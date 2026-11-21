//! Instance-fenced local connection control. Musical requests use the normal service protocol.
use clap::{Args, Subcommand};
use notes_transformer::{file_evidence, files};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Args)]
pub struct Connection {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    /// List retained adapter instances with current host-process identity checks.
    List,
    /// Request exactly one SDK connection attempt. Read its result with status; never retries.
    Connect(Request),
    /// Explicitly disconnect/cancel a connection attempt without replaying native operations.
    Disconnect(Request),
    /// Read current state and optionally the retained result of one exact lifecycle operation.
    Status {
        #[arg(long)]
        adapter: String,
        #[arg(long)]
        operation: Option<String>,
    },
}
#[derive(Args)]
struct Request {
    /// Exact adapter-instance UUID returned by connection list; changes on controller restart.
    #[arg(long)]
    adapter: String,
    /// Caller-chosen unique operation ID; retained before one attempt, never reused.
    #[arg(long)]
    operation: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Descriptor {
    schema_version: u32,
    adapter: String,
    wire_version: u8,
    host_build: String,
    state: String,
    error: Option<String>,
    attempt: u64,
    host_pid: u32,
    host_start_ticks: String,
    closed: bool,
    pending_operation: Option<String>,
    control_error: Option<String>,
    retained_commands: u16,
}
fn root() -> Result<PathBuf, Box<dyn Error>> {
    let base = match std::env::var_os("XDG_STATE_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(std::env::var_os("HOME").ok_or("HOME is unavailable")?)
            .join(".local/state"),
    };
    if !base.is_absolute() {
        return Err("XDG_STATE_HOME must be absolute".into());
    }
    Ok(base.join("midi-workbench/connections"))
}
fn identity(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_digit() || (b'a'..=b'f').contains(&c)
            }
        })
}
fn operation(value: &str) -> Result<(), Box<dyn Error>> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c))
    {
        return Err("Operation ID must be1–128 ASCII letters/digits or _.:-".into());
    }
    Ok(())
}
#[cfg(unix)]
fn private(path: &Path, directory: bool) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;
    let m = fs::symlink_metadata(path)?;
    if m.file_type().is_symlink()
        || (if directory { !m.is_dir() } else { !m.is_file() })
        || m.permissions().mode() & 0o777 != if directory { 0o700 } else { 0o600 }
    {
        return Err(format!(
            "Lifecycle location must be private{} and not a symlink: {}",
            if directory {
                "0700 directory"
            } else {
                "0600 regular file"
            },
            path.display()
        )
        .into());
    }
    Ok(())
}
#[cfg(not(unix))]
fn private(_: &Path, _: bool) -> Result<(), Box<dyn Error>> {
    Err("Connection lifecycle currently requires the admitted Linux host".into())
}
fn read(path: &Path) -> Result<Value, Box<dyn Error>> {
    private(path, false)?;
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err("Lifecycle file exceeds4096bytes".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
fn descriptor(root: &Path, adapter: &str) -> Result<(Descriptor, Value, PathBuf), Box<dyn Error>> {
    let result = retained_descriptor(root, adapter)?;
    if !compatible(&result.0) {
        return Err("Unsupported/stale lifecycle descriptor; no request published".into());
    }
    Ok(result)
}
fn compatible(d: &Descriptor) -> bool {
    d.wire_version == notes_transformer::protocol::VERSION
        && d.host_build == notes_transformer::session::HOST_BUILD
}
fn retained_descriptor(
    root: &Path,
    adapter: &str,
) -> Result<(Descriptor, Value, PathBuf), Box<dyn Error>> {
    if !identity(adapter) {
        return Err("Expected canonical adapter-instance UUID from connection list".into());
    }
    private(root, true)?;
    let directory = root.join(adapter);
    private(&directory, true)?;
    private(&directory.join("requests"), true)?;
    let value = read(&directory.join("status.json"))?;
    let d: Descriptor = serde_json::from_value(value.clone())?;
    if d.schema_version != 1
        || d.adapter != adapter
        || !["ready", "connecting", "failed", "disconnected", "closed"].contains(&d.state.as_str())
        || d.retained_commands > 256
        || d.host_start_ticks.is_empty()
    {
        return Err("Unsupported/stale lifecycle descriptor; no request published".into());
    }
    Ok((d, value, directory))
}
fn alive(d: &Descriptor) -> bool {
    fs::read_to_string(format!("/proc/{}/stat", d.host_pid))
        .ok()
        .and_then(|s| {
            s.rsplit_once(") ").map(|(_, tail)| {
                tail.split_whitespace().nth(19) == Some(d.host_start_ticks.as_str())
            })
        })
        .unwrap_or(false)
}
fn list(root: &Path) -> Result<Value, Box<dyn Error>> {
    let mut rows = Vec::new();
    if root.exists() {
        private(root, true)?;
        let mut directories = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
        if directories.len() > 1024 {
            return Err("More than1024 retained lifecycle instances; nothing truncated".into());
        }
        directories.sort_by_key(std::fs::DirEntry::file_name);
        for entry in directories {
            let name = entry.file_name();
            let name = name.to_str().ok_or("Non-UTF8 lifecycle instance")?;
            let (descriptor, mut row, _) = retained_descriptor(root, name)?;
            row["hostProcessAlive"] = alive(&descriptor).into();
            row["compatible"] = compatible(&descriptor).into();
            rows.push(row);
        }
    }
    Ok(json!({"schemaVersion":1,"status":"measured","root":root,"instances":rows}))
}
fn validate_receipt(
    directory: &Path,
    adapter: &str,
    op: &str,
    receipt: &Value,
) -> Result<(), Box<dyn Error>> {
    let request_path = directory
        .join("requests")
        .join(format!("{op}.request.json"));
    let request = read(&request_path)?;
    let expected_state = match request["action"].as_str() {
        Some("connect") => "ready",
        Some("disconnect") => "disconnected",
        _ => return Err("Unknown retained lifecycle action".into()),
    };
    if request["schemaVersion"] != 1
        || request["adapter"] != adapter
        || request["operation"] != op
        || receipt["schemaVersion"] != 1
        || receipt["adapter"] != adapter
        || receipt["operation"] != op
        || receipt["request"] != request
        || receipt["wireVersion"] != notes_transformer::protocol::VERSION
        || receipt["hostBuild"] != notes_transformer::session::HOST_BUILD
    {
        return Err(
            "Lifecycle result contradicts retained intent/build authority; no replay".into(),
        );
    }
    match receipt["status"].as_str() {
        Some("passed")
            if receipt["state"] == expected_state && receipt["operationError"].is_null() =>
        {
            Ok(())
        }
        Some("failed")
            if receipt["operationError"]
                .as_str()
                .is_some_and(|v| !v.is_empty()) =>
        {
            Ok(())
        }
        _ => Err("Lifecycle result contradicts terminal state/error; no replay".into()),
    }
}
fn status(root: &Path, adapter: &str, op: Option<&str>) -> Result<Value, Box<dyn Error>> {
    let (descriptor, current, directory) = descriptor(root, adapter)?;
    let mut result = json!({"schemaVersion":1,"status":"measured","current":current,"hostProcessAlive":alive(&descriptor)});
    if let Some(op) = op {
        operation(op)?;
        let file = directory.join("requests").join(format!("{op}.result.json"));
        if file.exists() {
            let receipt = read(&file)?;
            validate_receipt(&directory, adapter, op, &receipt)?;
            result["status"] = receipt["status"].clone();
            result["result"] = receipt;
            result["artifact"] = serde_json::to_value(file_evidence::measure(&file, 4096)?)?;
        } else {
            let retained = directory
                .join("requests")
                .join(format!("{op}.request.json"));
            let request = if retained.exists() {
                read(&retained)?
            } else {
                read(&directory.join("command.json"))?
            };
            if request["schemaVersion"] != 1
                || request["adapter"] != adapter
                || request["operation"] != op
            {
                return Err("Unknown lifecycle operation; do not replay".into());
            }
            result["status"] = if descriptor.closed || !alive(&descriptor) {
                "outcome-unknown"
            } else {
                "pending"
            }
            .into();
        }
    }
    Ok(result)
}
fn request(root: &Path, verb: &str, args: &Request) -> Result<Value, Box<dyn Error>> {
    operation(&args.operation)?;
    let (descriptor, _, directory) = descriptor(root, &args.adapter)?;
    if descriptor.closed || !alive(&descriptor) {
        return Err("Controller instance is closed/dead; refresh connection list".into());
    }
    if let Some(error) = &descriptor.control_error {
        return Err(format!("Lifecycle mailbox requires review: {error}").into());
    }
    if descriptor.retained_commands >= 256 {
        return Err("Lifecycle receipt budget exhausted".into());
    }
    if verb == "connect"
        && (descriptor.state == "ready"
            || descriptor.state == "connecting"
            || descriptor.pending_operation.is_some())
    {
        return Err("Adapter already connected/connecting; no request published".into());
    }
    for suffix in ["request", "result"] {
        if directory
            .join("requests")
            .join(format!("{}.{suffix}.json", args.operation))
            .exists()
        {
            return Err("Lifecycle operation already retained; read status, never replay".into());
        }
    }
    let request =
        json!({"schemaVersion":1,"adapter":args.adapter,"operation":args.operation,"action":verb});
    let inbox = directory.join("command.json");
    let mut bytes = serde_json::to_vec_pretty(&request)?;
    bytes.push(b'\n');
    files::publish_bytes(&inbox, &bytes)?;
    Ok(
        json!({"schemaVersion":1,"status":"requested","request":request,"priorState":descriptor.state,"priorError":descriptor.error,"priorAttempts":descriptor.attempt,
        "intent":{"publishedPath":inbox,"bytes":bytes.len()},"next":"connection status --adapter <same UUID> --operation <same ID>; do not repeat the request"}),
    )
}
pub fn run(args: Connection) -> Result<bool, Box<dyn Error>> {
    let root = root()?;
    let result = match args.command {
        Action::List => list(&root)?,
        Action::Status { adapter, operation } => status(&root, &adapter, operation.as_deref())?,
        Action::Connect(args) => request(&root, "connect", &args)?,
        Action::Disconnect(args) => request(&root, "disconnect", &args)?,
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(result["status"] != "failed" && result["status"] != "outcome-unknown")
}
