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
    let document = assigned["document"].clone();
    let (owner, hidden_baseline) = qualify_editor(
        state,
        adapter.clone(),
        &operation,
        &target,
        &runtime,
        &output,
        (&baseline, &assigned),
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
        if receiver_editor_visible(&current) != Some(true)
            || await_windows(
                &runtime, &output.join("kept-open"), &hidden_baseline,
                WindowExpectation::Visible(Some(owner.pid)),
            ).await.map_err(|e|control::Error::unknown(format!("Exact NKI load completed but editor-open verification failed: {}; do not reload",e.message)))?
                != Some(owner)
        {
            return Err(control::Error::unknown(
                "Loaded receiver editor is not independently confirmed open; do not reload",
            ));
        }
        Value::Null
    } else {
        state.operations.progress(
            &operation,
            "closingEditor",
            Some(&format!("{operation}:close")),
        )?;
        complete(state,adapter,json!({"action":"closeEditor","operation":format!("{operation}:close"),"target":target,"document":document,"expected":current})).await.map_err(|e|control::Error::unknown(e.to_string()))?
    };
    let result = json!({"operation":operation,"status":"verified","assignment":assigned,"load":loaded,"closure":closed,"editorKeptOpen":keep_editor_open,"ownership":{"method":"native-visibility-handshake","window":owner}});
    crate::files::publish(&output.join("assignment.json"), &result)
        .map_err(|e| control::Error::unknown(e.to_string()))?;
    *state
        .unresolved_loader
        .lock()
        .map_err(|_| control::Error::unavailable("Loader state unavailable"))? = None;
    Ok(result)
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
) -> Result<(Value, Value), control::Error> {
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
fn receiver_editor_visible(observation: &Value) -> Option<bool> {
    receiver_device(observation)?
        .get("editorVisible")?
        .as_bool()
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

// Admit only the existing, evidence-checked continuation through the write gate.
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
    if response["error"]["type"] == "NkiNotSubmitted" {
        return true;
    }
    legacy_initial_render_failure(directory, &response)
}

fn legacy_initial_render_failure(directory: &std::path::Path, response: &Value) -> bool {
    if response["error"]["type"] != "RuntimeError"
        || response["error"]["message"]
            != "Timed out waiting for the Kontakt window to render: last_mean=0.00"
    {
        return false;
    }
    // Wire-55 compatibility only: retain the exact producer source and establish
    // that its sole render wait precedes all Load-dialog interaction.
    let Ok(bytes) =
        crate::files::read_bounded(&directory.join("kontakt-ctl/kontakt_ctl/load_nki.py"))
    else {
        return false;
    };
    let Ok(source) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let Some(body) = source.split_once("def run_load_nki(").map(|(_, body)| body) else {
        return false;
    };
    let Some((preflight, after)) =
        body.split_once("    before_image = wait_for_window_render(render_window, config)")
    else {
        return false;
    };
    !preflight.contains("open_headless_load_dialog(")
        && !preflight.contains("set_dialog_text(")
        && after.contains("    open_headless_load_dialog(render_window, config)")
        && body.matches("wait_for_window_render(").count() == 1
}

async fn qualify_editor(
    state: &Sessions,
    adapter: Option<String>,
    operation: &str,
    target: &Path,
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    observations: (&Value, &Value),
) -> Result<(OwnedWindow, Vec<OwnedWindow>), control::Error> {
    let (baseline, assigned) = observations;
    let document = assigned["document"].clone();
    let before = window_rows(baseline)?;
    // Native visibility is not an OS handle. Qualify that association before any NKI input.
    // A freshly inserted hosted plug-in can expose an OS editor while its native
    // visibility value is still false. Establish open, then challenge hidden;
    // never infer the initial OS state from that native value alone.
    let current = assigned["observation"].clone();
    let opened = visibility(
        state,
        adapter.clone(),
        operation,
        "owner-open",
        target,
        &document,
        current,
        true,
    )
    .await?;
    let opened_windows =
        await_open_inventory(runtime, &output.join("owner-opened-inventory"), &before).await?;
    let closed = visibility(
        state,
        adapter.clone(),
        operation,
        "owner-challenge",
        target,
        &document,
        opened["observation"].clone(),
        false,
    )
    .await?;
    let first = await_windows(
        runtime,
        &output.join("owner-hidden"),
        &opened_windows,
        WindowExpectation::OneRemoved,
    )
    .await?
    .ok_or_else(|| control::Error::unknown("Owned editor window was not observed"))?;
    let hidden_baseline: Vec<_> = opened_windows
        .into_iter()
        .filter(|window| *window != first)
        .collect();
    let reopened = visibility(
        state,
        adapter.clone(),
        operation,
        "open",
        target,
        &document,
        closed["observation"].clone(),
        true,
    )
    .await?;
    let owner = await_windows(
        runtime,
        &output.join("owner-final"),
        &hidden_baseline,
        WindowExpectation::Visible(Some(first.pid)),
    )
    .await?
    .ok_or_else(|| control::Error::unknown("Owned editor did not reappear"))?;
    let observed = native(
        state,
        adapter.clone(),
        json!({"action":"read","target":target}),
    )
    .await
    .map_err(|e| control::Error::unknown(e.to_string()))?;
    if observed != reopened["observation"] {
        return Err(control::Error::unknown(
            "Receiver/editor changed during ownership qualification; no NKI submitted",
        ));
    }
    Ok((owner, hidden_baseline))
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
enum WindowExpectation {
    OneRemoved,
    Visible(Option<u32>),
}

async fn await_open_inventory(
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    baseline: &[OwnedWindow],
) -> Result<Vec<OwnedWindow>, control::Error> {
    std::fs::create_dir(output).map_err(|e| control::Error::unknown(e.to_string()))?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut previous = None;
    let mut attempt = 0;
    loop {
        let runtime = runtime.clone();
        let observation = output.join(format!("observation-{attempt}"));
        attempt += 1;
        let observed = tokio::task::spawn_blocking(move || {
            crate::kontakt_process::windows(&runtime, &observation).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| control::Error::unknown(e.to_string()))?
        .map_err(control::Error::unknown)?;
        let windows = window_rows(&observed)?;
        let count_matches = windows.len() == baseline.len() + 1
            || (!windows.is_empty() && windows.len() == baseline.len());
        if count_matches && baseline.iter().all(|window| windows.contains(window)) {
            if previous.as_ref() == Some(&windows) {
                return Ok(windows);
            }
            previous = Some(windows);
        } else {
            previous = None;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(control::Error::unknown(
                "Owned editor opening did not stabilize",
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn await_windows(
    runtime: &crate::kontakt_process::Runtime,
    output: &std::path::Path,
    baseline: &[OwnedWindow],
    visible: WindowExpectation,
) -> Result<Option<OwnedWindow>, control::Error> {
    std::fs::create_dir(output).map_err(|e| control::Error::unknown(e.to_string()))?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
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
        if matches!(visible, WindowExpectation::OneRemoved)
            && windows.len() + 1 == baseline.len()
            && windows.iter().all(|window| baseline.contains(window))
        {
            return Ok(baseline
                .iter()
                .find(|window| !windows.contains(window))
                .copied());
        }
        if baseline.iter().all(|window| windows.contains(window)) {
            let added: Vec<_> = windows
                .iter()
                .filter(|window| !baseline.contains(window))
                .copied()
                .collect();
            match visible {
                WindowExpectation::Visible(pid)
                    if added.len() == 1 && pid.is_none_or(|pid| added[0].pid == pid) =>
                {
                    return Ok(Some(added[0]));
                }
                _ => {}
            }
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(control::Error::unknown(
                "Editor ownership visibility challenge did not stabilize; retain the original operation",
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
#[allow(clippy::too_many_arguments)]
async fn visibility(
    state: &Sessions,
    adapter: Option<String>,
    operation: &str,
    phase: &str,
    target: &Path,
    document: &Value,
    expected: Value,
    visible: bool,
) -> Result<Value, control::Error> {
    let current = native(
        state,
        adapter.clone(),
        json!({"action":"read","target":target}),
    )
    .await?;
    if receiver_structure(expected) != receiver_structure(current.clone()) {
        return Err(control::Error::unknown(
            "Receiver structure changed before editor action; no input dispatched",
        ));
    }
    let result=complete(state,adapter,json!({"action":if visible{"openEditor"}else{"closeEditor"},"operation":format!("{operation}:{phase}"),"target":target,"document":document,"expected":current})).await.map_err(|e|control::Error::unknown(e.to_string()))?;
    if receiver_editor_visible(&result["observation"]) != Some(visible) {
        return Err(control::Error::unknown(
            "Native editor visibility readback differs",
        ));
    }
    Ok(result)
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
