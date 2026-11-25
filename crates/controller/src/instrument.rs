//! Scoped instrument intents. Native identities are evidence, never caller addresses.
use crate::{control, controller::Command, project_address::Path, session::Sessions};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Read {
        target: Path,
    },
    EnsureLayer {
        operation: String,
        target: Path,
    },
    EnsureSlot {
        operation: String,
        target: Path,
        name: String,
        #[serde(rename = "inputChannels")]
        input_channels: Vec<u8>,
    },
    DeleteSlot {
        operation: String,
        target: Path,
    },
    Assign {
        operation: String,
        target: Path,
        nki: Option<PathBuf>,
        runtime: Option<crate::kontakt_process::Runtime>,
        output: Option<PathBuf>,
        #[serde(default, rename = "keepEditorOpen")]
        keep_editor_open: bool,
        #[serde(default, rename = "resumeBeforeLoad")]
        resume_before_load: Option<PathBuf>,
    },
    Status {
        operation: String,
    },
}
impl Request {
    pub(crate) fn bind_operation(&mut self, id: &str) {
        match self {
            Self::Read { .. } | Self::Status { .. } => {}
            Self::EnsureLayer { operation, .. }
            | Self::EnsureSlot { operation, .. }
            | Self::DeleteSlot { operation, .. }
            | Self::Assign { operation, .. } => *operation = id.into(),
        }
    }
    #[must_use]
    pub fn operation(&self) -> Option<&str> {
        match self {
            Self::Read { .. } => None,
            Self::EnsureLayer { operation, .. }
            | Self::EnsureSlot { operation, .. }
            | Self::DeleteSlot { operation, .. }
            | Self::Assign { operation, .. }
            | Self::Status { operation } => Some(operation),
        }
    }
    /// # Errors
    /// Refuses malformed paths, operation IDs and channel values.
    pub fn validate(&self) -> Result<(), String> {
        if self
            .operation()
            .is_some_and(|id| id.is_empty() || id.len() > 128 || id.contains('\0'))
        {
            return Err("A bounded nonempty operation ID is required".into());
        }
        let target = match self {
            Self::Read { target }
            | Self::EnsureLayer { target, .. }
            | Self::EnsureSlot { target, .. }
            | Self::DeleteSlot { target, .. }
            | Self::Assign { target, .. } => target,
            Self::Status { .. } => return Ok(()),
        };
        if let Self::Assign {
            nki,
            runtime,
            output,
            keep_editor_open,
            resume_before_load,
            ..
        } = self
        {
            if nki.is_some() != runtime.is_some()
                || nki.is_some() != output.is_some()
                || (nki.is_none() && (*keep_editor_open || resume_before_load.is_some()))
            {
                return Err(
                    "Loading requires --load-patch, Wine runtime and evidence output together"
                        .into(),
                );
            }
        }
        target.validate()?;
        let slot = matches!(
            target.elements.last(),
            Some(
                crate::project_address::Element::LayerChain { .. }
                    | crate::project_address::Element::Choice { .. }
            )
        );
        if !slot
            && !matches!(
                target.elements.last(),
                Some(
                    crate::project_address::Element::Layer { .. }
                        | crate::project_address::Element::Selector { .. }
                )
            )
        {
            return Err("Instrument target must be a qualified layer or slot path".into());
        }
        if matches!(self, Self::EnsureLayer { .. }) && slot {
            return Err("Container ensure requires a layer or selector path".into());
        }
        if !matches!(
            self,
            Self::Read { .. } | Self::EnsureLayer { .. } | Self::EnsureSlot { .. }
        ) && !slot
        {
            return Err("Instrument operation requires a slot path".into());
        }
        if let Self::EnsureSlot {
            name,
            input_channels,
            ..
        } = self
        {
            if !crate::project_address::name(name)
                || input_channels.is_empty()
                || input_channels.len() > 16
                || input_channels.iter().any(|c| *c > 15)
                || input_channels
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != input_channels.len()
            {
                return Err("Slot ensure requires a name and distinct input channels 0..15".into());
            }
        }
        Ok(())
    }
}

pub(crate) async fn native(
    state: &Sessions,
    adapter: Option<String>,
    request: Value,
) -> Result<Value, control::Error> {
    match crate::session::dispatch(state, adapter, Command::InstrumentNative { request }).await? {
        control::Response::InstrumentResult { result } => {
            if result["status"] == "rejected" {
                return Err(control::Error::invalid(result["error"].to_string()));
            }
            Ok(result)
        }
        _ => Err(control::Error::unknown("Unexpected instrument result")),
    }
}

/// Execute only through the service's admitted adapter connection.
pub(crate) async fn execute(
    state: &Sessions,
    adapter: Option<String>,
    mut request: Request,
) -> Result<Value, control::Error> {
    request.validate().map_err(control::Error::invalid)?;
    if let Request::EnsureSlot { target, name, .. } = &mut request {
        use crate::project_address::Element;
        let child = match target.elements.last() {
            Some(Element::Layer { .. }) => Some(Element::LayerChain {
                name: name.clone(),
                native_id: None,
            }),
            Some(Element::Selector { .. }) => Some(Element::Choice {
                name: name.clone(),
                native_id: None,
            }),
            _ => None,
        };
        if let Some(child) = child {
            target.elements.push(child);
        }
    }
    if let Request::Read { target } = request {
        return native(state, adapter, json!({"action":"read","target":target})).await;
    }
    if let Request::Status { operation } = request {
        return crate::operations::inspect(state, &operation).await;
    }
    let mut command =
        serde_json::to_value(&request).map_err(|e| control::Error::invalid(e.to_string()))?;
    if let Some(operation) = request.operation() {
        state.operations.progress(operation, "preparing", None)?;
    }
    let before = native(
        state,
        adapter.clone(),
        json!({"action":"read","target":command["target"]}),
    )
    .await?;
    command["document"] = before["document"].clone();
    command["expected"] = before;
    if matches!(request, Request::Assign { nki: None, .. }) {
        return complete(state, adapter, assignment_native_command(command)).await;
    }
    if matches!(request, Request::Assign { .. }) {
        return assign(state, adapter, request, command).await;
    }
    complete(state, adapter, command).await
}
async fn assign(
    state: &Sessions,
    adapter: Option<String>,
    request: Request,
    command: Value,
) -> Result<Value, control::Error> {
    let Request::Assign {
        operation,
        target,
        nki: Some(nki),
        runtime: Some(runtime),
        output: Some(output),
        keep_editor_open,
        resume_before_load,
    } = request
    else {
        return Err(control::Error::invalid("Expected instrument assignment"));
    };
    let acquisition = EditorAcquisition::for_receiver(&command["expected"])?;
    let runtime = runtime.for_host(before_host(&command["expected"])?)?;
    check_assignment_resources(&runtime, &nki, &output).await?;
    let (assigned, baseline) = begin_assignment(
        state,
        adapter.clone(),
        command,
        &runtime,
        &output,
        resume_before_load.as_deref(),
    )
    .await?;
    let owner = qualify_editor(
        state,
        adapter.clone(),
        &operation,
        &target,
        &runtime,
        &output,
        (&baseline, &assigned),
        acquisition,
    )
    .await
    .map_err(|e| {
        control::Error::unknown(format!(
            "{}; ownership qualification failed before any NKI submission",
            e.message
        ))
    })?;
    claim_preload_continuation(&assigned, &operation, &nki, &output)?;
    state
        .operations
        .progress(&operation, "loading", Some(&format!("{operation}:load")))?;
    *state
        .unresolved_loader
        .lock()
        .map_err(|_| control::Error::unavailable("Loader state unavailable"))? =
        Some(operation.clone());
    let loaded = load_owned_nki(&runtime, nki, &output, &operation, owner).await?;
    let current = native(
        state,
        adapter.clone(),
        json!({"action":"read","target":target}),
    )
    .await
    .map_err(|e| control::Error::unknown(e.to_string()))?;
    if receiver_structure(assigned["observation"].clone()) != receiver_structure(current.clone()) {
        return Err(control::Error::unknown(
            "Receiver ownership or routing changed during NKI load; no editor closed",
        ));
    }
    let closed = if keep_editor_open {
        await_windows(
            &runtime,
            &output.join("kept-open"),
            &baseline,
            WindowExpectation::Open(owner),
        )
        .await
        .map_err(|e| loaded_visibility_error("open", &e))?;
        Value::Null
    } else {
        close_loaded_editor(
            state, adapter, &runtime, &output, &baseline, &operation, current,
        )
        .await?
    };
    let result = json!({
        "operation":operation,
        "status":"verified",
        "assignment":assigned,
        "load":loaded,
        "closure":closed,
        "editorKeptOpen":keep_editor_open,
        "ownership":{"method":"scoped-window-appearance","window":owner},
    });
    crate::files::publish(&output.join("assignment.json"), &result)
        .map_err(|e| control::Error::unknown(e.to_string()))?;
    *state
        .unresolved_loader
        .lock()
        .map_err(|_| control::Error::unavailable("Loader state unavailable"))? = None;
    Ok(result)
}

async fn close_loaded_editor(
    state: &Sessions,
    adapter: Option<String>,
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    baseline: &[OwnedWindow],
    operation: &str,
    current: Value,
) -> Result<Value, control::Error> {
    let close_operation = format!("{operation}:close");
    state
        .operations
        .progress(operation, "closingEditor", Some(&close_operation))?;
    let closed = complete(
        state,
        adapter,
        json!({
            "action":"closeEditor",
            "operation":close_operation,
            "target":current["target"],
            "document":current["document"],
            "expected":current,
        }),
    )
    .await
    .map_err(|e| loaded_visibility_error("closed", &e))?;
    await_windows(
        runtime,
        &output.join("closed"),
        baseline,
        WindowExpectation::Closed,
    )
    .await
    .map_err(|e| loaded_visibility_error("closed", &e))?;
    Ok(closed)
}

fn loaded_visibility_error(expected: &str, error: &control::Error) -> control::Error {
    control::Error::unknown(format!(
        "Exact NKI load completed but its editor was not confirmed {expected}: {}; do not reload",
        error.message
    ))
}

fn assignment_native_command(mut command: Value) -> Value {
    command["action"] = "assignReceiver".into();
    let fields = command.as_object_mut().expect("serialized request object");
    for field in [
        "nki",
        "runtime",
        "output",
        "keepEditorOpen",
        "resumeBeforeLoad",
    ] {
        fields.remove(field);
    }
    command
}

async fn begin_assignment(
    state: &Sessions,
    adapter: Option<String>,
    mut command: Value,
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    resume_before_load: Option<&std::path::Path>,
) -> Result<(Value, Vec<OwnedWindow>), control::Error> {
    let operation = command["operation"]
        .as_str()
        .ok_or_else(|| control::Error::invalid("Missing assignment operation"))?
        .to_owned();
    command = assignment_native_command(command);
    std::fs::create_dir(output).map_err(|e| control::Error::invalid(e.to_string()))?;
    let inventory_runtime = runtime.clone();
    let inventory_output = output.join("windows-before");
    let baseline = tokio::task::spawn_blocking(move || {
        crate::kontakt_process::windows(&inventory_runtime, &inventory_output)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| control::Error::invalid(e.to_string()))?
    .map_err(control::Error::invalid)?;
    let baseline = window_rows(&baseline).map_err(|e| control::Error::invalid(e.message))?;
    state
        .operations
        .progress(&operation, "assigningReceiver", Some(&operation))?;
    let assigned = if let Some(receipt) = resume_before_load {
        let prior = resume_observation(receipt, &command["expected"])?;
        let claim = preload_claim_path(runtime, &command["expected"])?;
        crate::files::ensure_fresh_output(&claim)
            .map_err(|e| control::Error::invalid(e.to_string()))?;
        json!({"operation":operation,"status":"completed","document":command["document"],"observation":command["expected"],"resumedBeforeLoad":prior,"continuationClaim":claim})
    } else {
        complete(state, adapter.clone(), command).await?
    };
    Ok((assigned, baseline))
}

fn receiver_device(observation: &Value) -> Option<&Value> {
    observation["slot"]["devices"]
        .as_array()?
        .iter()
        .find(|d| d["pluginId"] == "vst3-5653544E694B376B6F6E74616B742037")
}

fn claim_preload_continuation(
    assigned: &Value,
    operation: &str,
    nki: &std::path::Path,
    output: &std::path::Path,
) -> Result<(), control::Error> {
    if let Some(claim) = assigned["continuationClaim"].as_str() {
        crate::files::publish(std::path::Path::new(claim),
            &json!({"originalRequest":assigned["resumedBeforeLoad"],"continuation":operation,"nki":nki,"output":output}))
            .map_err(|e| control::Error::invalid(format!("Pre-load continuation already claimed or unavailable: {e}")))?;
    }
    Ok(())
}

fn preload_claim_path(
    runtime: &crate::kontakt_process::Runtime,
    observation: &Value,
) -> Result<PathBuf, control::Error> {
    let devices = receiver_device(observation)
        .ok_or_else(|| control::Error::invalid("Missing resumed receiver"))?;
    let ids = [
        &observation["projectUuid"],
        &observation["slot"]["id"],
        &devices["id"],
    ];
    let mut key = String::new();
    for id in ids {
        let id = id
            .as_str()
            .filter(|id| {
                !id.is_empty()
                    && id.len() <= 128
                    && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            })
            .ok_or_else(|| control::Error::invalid("Invalid persistent continuation identity"))?;
        key.push_str(id);
        key.push('_');
    }
    let root = runtime
        .wine_prefix
        .canonicalize()
        .map_err(|e| control::Error::invalid(e.to_string()))?
        .join("midi-workbench-assignment-claims");
    std::fs::create_dir_all(&root).map_err(|e| control::Error::invalid(e.to_string()))?;
    Ok(root.join(format!("{key}.json")))
}

async fn load_owned_nki(
    runtime: &crate::kontakt_process::Runtime,
    nki: PathBuf,
    output: &std::path::Path,
    operation: &str,
    owner: OwnedWindow,
) -> Result<Value, control::Error> {
    let runtime = runtime.clone();
    let output = output.join("load");
    let operation = format!("{operation}:load");
    tokio::task::spawn_blocking(move || {
        crate::kontakt_process::load(&runtime, &nki, &output, &operation, owner.hwnd, owner.pid)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| control::Error::unknown(e.to_string()))?
    .map_err(control::Error::unknown)
}

// Admit the evidence-checked continuation before another NKI submission.
// Fresh native owner/routing checks and the one-use claim still run in assignment.
pub(crate) fn resumes_blocked_loader(request: &Request, blocked: &str) -> bool {
    let Request::Assign {
        resume_before_load: Some(receipt),
        nki: Some(_),
        ..
    } = request
    else {
        return false;
    };
    let Ok(bytes) = crate::files::read_bounded(receipt) else {
        return false;
    };
    let Ok(record) = serde_json::from_slice::<Value>(&bytes) else {
        return false;
    };
    let prior = &record["result"];
    prior["operationId"] == blocked
        && resume_observation(receipt, &prior["native"]["observation"]).is_ok()
}

// Explicit continuation of a proven pre-submission failure. Loading, ownership
// qualification and completion remain the same integrated assignment operation.
fn resume_observation(
    receipt: &std::path::Path,
    current: &Value,
) -> Result<String, control::Error> {
    let bytes =
        crate::files::read_bounded(receipt).map_err(|e| control::Error::invalid(e.to_string()))?;
    let record: Value =
        serde_json::from_slice(&bytes).map_err(|e| control::Error::invalid(e.to_string()))?;
    let prior = &record["result"];
    let request_id = prior["requestId"].as_str().filter(|id| !id.is_empty());
    if record["type"] != "instrumentResult"
        || prior["state"] != "outcomeUnknown"
        || prior["failure"]["outcome"] != "unknown"
        || !proven_before_submission(prior)
        || prior["native"]["status"] != "completed"
        || prior["operationId"].as_str().is_none_or(str::is_empty)
        || prior["native"]["operation"] != prior["operationId"]
        || request_id.is_none()
    {
        return Err(control::Error::invalid(
            "Continuation requires original completed-insertion status and explicit proof of no NKI submission",
        ));
    }
    let original = &prior["native"]["observation"];
    let observation: Observation = serde_json::from_value(original.clone())
        .map_err(|e| control::Error::invalid(e.to_string()))?;
    observation.validate().map_err(control::Error::invalid)?;
    if original["slot"].is_null() || prior["native"]["document"] != original["document"] {
        return Err(control::Error::invalid(
            "Original receiver insertion evidence is incomplete",
        ));
    }
    if !same_resume_owner(original, current) {
        return Err(control::Error::invalid(
            "Original receiver identity, routing, project or host changed; no continuation dispatched",
        ));
    }
    request_id
        .map(str::to_owned)
        .ok_or_else(|| control::Error::invalid("Missing original request ID"))
}

fn same_resume_owner(original: &Value, current: &Value) -> bool {
    let owner = |value: &Value| {
        let mut scoped = json!({});
        for key in [
            "projectUuid",
            "hostPid",
            "engine",
            "target",
            "trackId",
            "trackTitle",
            "trackDevices",
            "parentId",
            "parentDevices",
            "layerId",
            "layerPresent",
            "slot",
        ] {
            scoped[key] = value[key].clone();
        }
        // Receipts before nested-container support belonged directly to the track.
        if scoped["parentId"].is_null() {
            scoped["parentId"] = value["trackId"].clone();
            scoped["parentDevices"] = value["trackDevices"].clone();
        }
        receiver_structure(scoped)
    };
    let mut prior = owner(original);
    let current = owner(current);
    if let (Some(old), Some(now)) = (
        prior["slot"]["devices"].as_array_mut(),
        current["slot"]["devices"].as_array(),
    ) {
        // The admitted legacy migration can remove only the stock sender before
        // the exact same Kontakt. It cannot replace/add/reorder receiver devices.
        if old.len() == 2
            && now.len() == 1
            && old[0]["name"] == "MIDI CC"
            && old[0]["nativeDeviceId"] == "a0b8f27a-128e-4f72-b9fc-a277060b87ee"
            && old[1] == now[0]
        {
            old.remove(0);
        }
    }
    prior == current
}

fn proven_before_submission(prior: &Value) -> bool {
    let message = prior["failure"]["message"].as_str().unwrap_or_default();
    if prior["progress"]["stage"] == "assigningReceiver" {
        return message.contains("no NKI submitted")
            || message.contains("before any NKI submission");
    }
    if prior["progress"]["stage"] != "loading"
        || prior["progress"]["childOperation"].as_str()
            != prior["operationId"]
                .as_str()
                .map(|id| format!("{id}:load"))
                .as_deref()
    {
        return false;
    }
    let Some((body, directory)) = message
        .strip_prefix("kontakt-ctl failed (exit status: 1): ")
        .and_then(|value| value.rsplit_once("; see "))
    else {
        return false;
    };
    let Ok(response) = serde_json::from_str::<Value>(body) else {
        return false;
    };
    let directory = std::path::Path::new(directory);
    let read_json = |name| {
        crate::files::read_bounded(&directory.join(name))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    };
    let Some(request) = read_json("request.json") else {
        return false;
    };
    if !directory.is_absolute()
        || response["schema"] != "kontakt-ctl.result/v1"
        || response["ok"] != false
        || read_json("response.json").as_ref() != Some(&response)
        || request["request"]["operationId"] != prior["progress"]["childOperation"]
        || request["command"][0] != "load-nki"
    {
        return false;
    }
    response["error"]["type"] == "NkiNotSubmitted"
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditorAcquisition {
    AutomaticOnInsertion,
    OpenExisting,
}

impl EditorAcquisition {
    fn for_receiver(before: &Value) -> Result<Self, control::Error> {
        match receiver_device(before) {
            None => Ok(Self::AutomaticOnInsertion),
            Some(device) if device["editorVisible"] == false => Ok(Self::OpenExisting),
            Some(_) => Err(control::Error::invalid(
                "Existing Kontakt editor must be hidden to identify its window when opened; no editor changed",
            )),
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn qualify_editor(
    state: &Sessions,
    adapter: Option<String>,
    operation: &str,
    target: &Path,
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    observations: (&[OwnedWindow], &Value),
    acquisition: EditorAcquisition,
) -> Result<OwnedWindow, control::Error> {
    let (baseline, assigned) = observations;
    let expected = &assigned["observation"];
    // An insertion opens the hosted editor itself. Its native visibility flag can
    // lag the actual Win32 window, so observe appearance without toggling it.
    if acquisition == EditorAcquisition::OpenExisting {
        let current = native(
            state,
            adapter.clone(),
            json!({"action":"read","target":target}),
        )
        .await?;
        if &current != expected {
            return Err(control::Error::unknown(
                "Receiver changed before opening its editor; no editor action dispatched",
            ));
        }
        complete(
            state,
            adapter.clone(),
            json!({
                "action":"openEditor",
                "operation":format!("{operation}:open"),
                "target":target,
                "document":assigned["document"],
                "expected":current,
            }),
        )
        .await?;
    }
    let WindowObservation::Open(owner) = await_windows(
        runtime,
        &output.join("owner-appeared"),
        baseline,
        WindowExpectation::New,
    )
    .await?
    else {
        return Err(control::Error::unknown(
            "Owned editor window was not observed",
        ));
    };
    let current = native(state, adapter, json!({"action":"read","target":target})).await?;
    if receiver_structure(expected.clone()) != receiver_structure(current) {
        return Err(control::Error::unknown(
            "Receiver changed during ownership qualification; no NKI submitted",
        ));
    }
    Ok(owner)
}

async fn complete(
    state: &Sessions,
    adapter: Option<String>,
    command: Value,
) -> Result<Value, control::Error> {
    let operation = command["operation"].clone();
    let mut result = native(state, adapter.clone(), command).await?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while result["status"] == "pending" {
        if tokio::time::Instant::now() >= deadline {
            return Err(control::Error::unknown(format!(
                "Instrument operation {operation} remains pending; reconcile its original ID"
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        result = native(
            state,
            adapter.clone(),
            json!({"action":"status","operation":operation}),
        )
        .await?;
    }
    if result["status"] == "rejected" {
        return Err(control::Error::invalid(result["error"].to_string()));
    }
    if result["status"] != "completed" {
        return Err(control::Error::unknown(result.to_string()));
    }
    Ok(result)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub target: Path,
    pub document: String,
    pub project_uuid: String,
    pub track_id: String,
    pub track_title: String,
    pub host_pid: u32,
    pub engine: String,
    pub layer_id: Option<String>,
    pub layer_present: bool,
    pub track_devices: Vec<Device>,
    #[serde(default)]
    pub parent_id: String,
    #[serde(default)]
    pub parent_devices: Vec<Device>,
    pub slots: Vec<Slot>,
    pub slot: Option<Slot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_slot: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub plugin_id: Option<String>,
    pub version: Option<String>,
    pub editor_visible: Option<bool>,
    pub native_device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_filter: Option<Vec<bool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Stock Map destination values are displayed channels 1..16, unlike note channels 0..15.
    pub channel_map: Option<Vec<u8>>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub id: String,
    pub name: String,
    pub channel: i8,
    pub output_channel: i8,
    pub devices: Vec<Device>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeOutcome {
    operation: String,
    status: String,
    document: Option<String>,
    observation: Option<Observation>,
    error: Option<String>,
    automatic_track_name: Option<AutomaticName>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AutomaticName {
    before: String,
    after: String,
}
impl Observation {
    /// # Errors
    /// Refuses incomplete ownership, duplicate identities and unbounded structural payloads.
    pub fn validate(&self) -> Result<(), String> {
        self.target.validate()?;
        let id = |id: &str| !id.is_empty() && id.len() <= 128 && !id.contains('\0');
        if !id(&self.document)
            || !id(&self.project_uuid)
            || !id(&self.track_id)
            || self.slots.len() > 128
            || self.track_devices.len() > 256
            || (!self.parent_id.is_empty() && !id(&self.parent_id))
            || self.layer_present != self.layer_id.is_some()
            || (!self.layer_present && (!self.slots.is_empty() || self.slot.is_some()))
        {
            return Err("Invalid scoped instrument ownership/capacity".into());
        }
        let devices = |devices: &[Device]| -> bool {
            let mut ids = std::collections::BTreeSet::new();
            devices.len() <= 256
                && devices.iter().all(|device| {
                    id(&device.id)
                        && ids.insert(&device.id)
                        && device.name.len() <= 256
                        && device.plugin_id.is_some() == device.version.is_some()
                        && device.plugin_id.is_some() == device.editor_visible.is_some()
                        && match device.native_device_id.as_deref() {
                            Some("c5a1bb2d-a589-4fda-b3cf-911cfd6297be") => {
                                device
                                    .channel_filter
                                    .as_ref()
                                    .is_some_and(|v| v.len() == 16)
                                    && device.channel_map.is_none()
                                    && device.routing_enabled.is_some()
                                    && device.routing_active.is_some()
                            }
                            Some("0f003fa3-adcc-4684-81f7-f0e11c09c5b4") => {
                                device.channel_map.as_ref().is_some_and(|v| {
                                    v.len() == 16 && v.iter().all(|c| (1..=16).contains(c))
                                }) && device.channel_filter.is_none()
                                    && device.routing_enabled.is_some()
                                    && device.routing_active.is_some()
                            }
                            _ => {
                                device.channel_filter.is_none()
                                    && device.channel_map.is_none()
                                    && device.routing_enabled.is_none()
                                    && device.routing_active.is_none()
                            }
                        }
                })
        };
        let mut ids = std::collections::BTreeSet::new();
        if !devices(&self.track_devices)
            || !devices(&self.parent_devices)
            || self.slots.iter().any(|slot| {
                !id(&slot.id)
                    || !ids.insert(&slot.id)
                    || !crate::project_address::name(&slot.name)
                    || !(-1..=15).contains(&slot.channel)
                    || !(-1..=15).contains(&slot.output_channel)
                    || !devices(&slot.devices)
            })
            || self
                .selected_slot
                .as_ref()
                .is_some_and(|selected| !self.slots.iter().any(|slot| &slot.id == selected))
            || self.slot.as_ref().is_some_and(|slot| {
                self.slots
                    .iter()
                    .filter(|candidate| *candidate == slot)
                    .count()
                    != 1
            })
        {
            return Err("Invalid scoped slot membership/routing".into());
        }
        Ok(())
    }
}
pub(crate) fn validate_native(request: &Value, result: &Value) -> Result<(), String> {
    if request["action"] == "read" {
        let observation: Observation =
            serde_json::from_value(result.clone()).map_err(|e| e.to_string())?;
        observation.validate()?;
        if observation.parent_id.is_empty() {
            return Err("Missing native container parent identity".into());
        }
        if serde_json::to_value(&observation.target).map_err(|e| e.to_string())?
            != request["target"]
        {
            return Err("Instrument read addressed a different musical path".into());
        }
    } else {
        let outcome: NativeOutcome =
            serde_json::from_value(result.clone()).map_err(|e| e.to_string())?;
        if request["operation"] != outcome.operation
            || !["pending", "completed", "unknown", "failed", "rejected"]
                .contains(&outcome.status.as_str())
        {
            return Err("Invalid instrument operation correlation/state".into());
        }
        if outcome.status == "completed" {
            let observation = outcome
                .observation
                .ok_or("Missing independent scoped readback")?;
            observation.validate()?;
            if observation.parent_id.is_empty() {
                return Err("Missing native container parent identity".into());
            }
            if outcome.document.as_deref() != Some(&observation.document) || outcome.error.is_some()
            {
                return Err("Instrument completion authority/error differs".into());
            }
        }
        if let Some(name) = outcome.automatic_track_name {
            if name.before.is_empty() || name.after.is_empty() {
                return Err("Incomplete native automatic-name evidence".into());
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct OwnedWindow {
    hwnd: u64,
    pid: u32,
}
fn window_rows(inventory: &Value) -> Result<Vec<OwnedWindow>, control::Error> {
    let mut windows: Vec<OwnedWindow> =
        serde_json::from_value(inventory["data"]["windows"].clone())
            .map_err(|e| control::Error::unknown(e.to_string()))?;
    if windows.len() > 256
        || windows
            .iter()
            .any(|window| window.hwnd == 0 || window.pid == 0)
    {
        return Err(control::Error::unknown(
            "Unbounded or invalid Win32 window inventory",
        ));
    }
    windows.sort_unstable();
    if windows.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(control::Error::unknown("Duplicate Win32 window identity"));
    }
    Ok(windows)
}
#[derive(Clone, Copy)]
enum WindowExpectation {
    New,
    Open(OwnedWindow),
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowObservation {
    Open(OwnedWindow),
    Closed,
}

impl WindowExpectation {
    fn observe(
        self,
        baseline: &[OwnedWindow],
        windows: &[OwnedWindow],
    ) -> Option<WindowObservation> {
        if !baseline.iter().all(|window| windows.contains(window)) {
            return None;
        }
        if matches!(self, Self::Closed) {
            return (windows.len() == baseline.len()).then_some(WindowObservation::Closed);
        }
        let mut added = windows.iter().filter(|window| !baseline.contains(window));
        let owner = *added.next()?;
        if added.next().is_some() || matches!(self, Self::Open(expected) if owner != expected) {
            return None;
        }
        Some(WindowObservation::Open(owner))
    }
}

async fn await_windows(
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    baseline: &[OwnedWindow],
    expectation: WindowExpectation,
) -> Result<WindowObservation, control::Error> {
    std::fs::create_dir(output).map_err(|e| control::Error::unknown(e.to_string()))?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut previous = None;
    let mut attempt = 0;
    loop {
        let runtime = runtime.clone();
        let output = output.join(format!("observation-{attempt}"));
        attempt += 1;
        let observed = tokio::task::spawn_blocking(move || {
            crate::kontakt_process::windows(&runtime, &output).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| control::Error::unknown(e.to_string()))?
        .map_err(control::Error::unknown)?;
        let windows = window_rows(&observed)?;
        let outcome = expectation.observe(baseline, &windows);
        if let Some(owner) = outcome {
            if previous == outcome {
                return Ok(owner);
            }
        }
        previous = outcome;
        if tokio::time::Instant::now() >= deadline {
            return Err(control::Error::unknown(
                "Expected editor window inventory did not stabilize; retain the original operation",
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn before_host(observation: &Value) -> Result<u32, control::Error> {
    observation["hostPid"]
        .as_u64()
        .and_then(|pid| u32::try_from(pid).ok())
        .filter(|pid| *pid > 0)
        .ok_or_else(|| control::Error::invalid("Native host process identity is missing"))
}

// Window visibility is asynchronous native UI state. Identity/routing remain exact.
fn receiver_structure(mut observation: Value) -> Value {
    fn remove_visibility(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("editorVisible");
                for child in fields.values_mut() {
                    remove_visibility(child);
                }
            }
            Value::Array(values) => {
                for value in values {
                    remove_visibility(value);
                }
            }
            _ => {}
        }
    }
    remove_visibility(&mut observation);
    observation
}

async fn check_assignment_resources(
    runtime: &crate::kontakt_process::Runtime,
    nki: &std::path::Path,
    output: &std::path::Path,
) -> Result<(), control::Error> {
    let checked_runtime = runtime.clone();
    let checked_nki = nki.to_path_buf();
    let checked_output = output.to_path_buf();
    tokio::task::spawn_blocking(move || {
        crate::kontakt_process::preflight(&checked_runtime, &checked_nki, &checked_output)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| control::Error::invalid(e.to_string()))?
    .map_err(control::Error::invalid)
}

#[cfg(test)]
mod editor_tests {
    use super::{EditorAcquisition, OwnedWindow, WindowExpectation, WindowObservation};
    use serde_json::json;

    #[test]
    fn new_receiver_uses_automatic_open_and_hidden_existing_receiver_opens_once() {
        let observation = |devices| json!({"slot":{"devices":devices}});
        let kontakt = |visible| {
            json!([{
                "pluginId":"vst3-5653544E694B376B6F6E74616B742037",
                "editorVisible":visible
            }])
        };
        assert_eq!(
            EditorAcquisition::for_receiver(&observation(json!([]))).unwrap(),
            EditorAcquisition::AutomaticOnInsertion
        );
        assert_eq!(
            EditorAcquisition::for_receiver(&observation(kontakt(false))).unwrap(),
            EditorAcquisition::OpenExisting
        );
        assert!(EditorAcquisition::for_receiver(&observation(kontakt(true))).is_err());
    }

    #[test]
    fn automatic_open_requires_one_new_window_and_preserves_existing_windows() {
        let unrelated = OwnedWindow { hwnd: 10, pid: 20 };
        let editor = OwnedWindow { hwnd: 11, pid: 20 };
        let replacement = OwnedWindow { hwnd: 12, pid: 20 };
        let baseline = [unrelated];
        assert_eq!(
            WindowExpectation::New.observe(&baseline, &[unrelated, editor]),
            Some(WindowObservation::Open(editor))
        );
        assert_eq!(WindowExpectation::New.observe(&baseline, &baseline), None);
        assert_eq!(WindowExpectation::New.observe(&baseline, &[editor]), None);
        assert_eq!(
            WindowExpectation::New.observe(&baseline, &[unrelated, editor, replacement]),
            None
        );
        assert_eq!(
            WindowExpectation::Open(editor).observe(&baseline, &[unrelated, replacement]),
            None
        );
        assert_eq!(
            WindowExpectation::Closed.observe(&baseline, &[unrelated, editor]),
            None
        );
        assert_eq!(
            WindowExpectation::Closed.observe(&baseline, &baseline),
            Some(WindowObservation::Closed)
        );
    }
}
