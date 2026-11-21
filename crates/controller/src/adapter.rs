//! Local development CLI. Java alone owns document access and named probes.
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Invoke one focused development probe.
/// # Errors
/// Returns invalid input or an unavailable native outcome.
pub fn probe(
    command: &str,
    plan: Option<&Path>,
    document: Option<&str>,
) -> Result<Value, Box<dyn Error>> {
    if ![
        "reload",
        "selection",
        "freeze",
        "arm",
        "apply",
        "undo",
        "redo",
    ]
    .contains(&command)
    {
        return Err("Unknown named probe".into());
    }
    let mut request = json!({"type":command});
    if let Some(plan) = plan {
        request["plan"] = serde_json::from_slice(&fs::read(plan)?)?;
    }
    if let Some(document) = document {
        request["document"] = document.into();
    }
    if command == "arm" {
        if request.get("document").is_none() {
            return Err(
                "arm requires --document with the observed disposable project generation".into(),
            );
        }
        request["confirmation"] = "DISPOSABLE FIXTURE".into();
    }
    if command == "apply" && request.get("plan").is_none() {
        return Err("apply requires --plan".into());
    }
    invoke(request)
}

/// Create the disposable MIDI fixture.
/// # Errors
/// Requires current native authority.
pub fn construction(project_uuid: &str, revision: &str) -> Result<Value, Box<dyn Error>> {
    if project_uuid.is_empty() || !crate::protocol::valid_revision(revision) {
        return Err("Native construction requires the current project UUID and revision".into());
    }
    invoke(
        json!({"type":"construction","phase":"native","projectUuid":project_uuid,"revision":revision,"confirmation":"DISPOSABLE FIXTURE"}),
    )
}

/// Read native document authority for a disposable fixture.
/// # Errors
/// Returns unavailable host data.
pub fn authority_probe() -> Result<Value, Box<dyn Error>> {
    invoke(json!({"type":"authority"}))
}
fn invoke(request: Value) -> Result<Value, Box<dyn Error>> {
    invoke_with_deadline(request, Duration::from_secs(15))
}

fn invoke_with_deadline(mut request: Value, budget: Duration) -> Result<Value, Box<dyn Error>> {
    let operation = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    request["operation"] = operation.clone().into();
    let root = std::env::var_os("NOTES_HARNESS_DIRECTORY").map_or_else(
        || "/tmp/notes-editor-harness".into(),
        std::path::PathBuf::from,
    );
    fs::create_dir_all(&root)?;
    let output = root.join("results.jsonl");
    let mut offset = fs::metadata(&output).map_or(0, |metadata| metadata.len());
    let pending = root.join(format!("{operation}.tmp"));
    fs::write(&pending, serde_json::to_vec(&request)?)?;
    fs::rename(pending, root.join("command.json"))?;
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if let Ok(file) = fs::File::open(&output) {
            let mut reader = BufReader::new(file);
            reader.seek(SeekFrom::Start(offset))?;
            loop {
                let mut line = Vec::new();
                let size = Read::take(
                    &mut reader,
                    u64::try_from(crate::frame::MAX_DECODED_BYTES)? + 1,
                )
                .read_until(b'\n', &mut line)?;
                if size > crate::frame::MAX_DECODED_BYTES {
                    return Err(format!("Harness response exceeds16MiB for operation {operation}; outcome unknown, no replay authorized; reconcile {} from byte offset {offset}",output.display()).into());
                }
                // A writer may still be appending this line. Reread it next poll.
                if size == 0 || line.last() != Some(&b'\n') {
                    break;
                }
                offset += u64::try_from(size)?;
                let Ok(entry) = serde_json::from_slice::<Value>(&line) else {
                    continue;
                };
                if entry.get("operation").and_then(Value::as_str) == Some(&operation) {
                    return Ok(entry);
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    Err(format!("No Java outcome within {} seconds for operation {operation}; state unknown, no replay authorized. Reconcile {} from byte offset {offset}",budget.as_secs(),output.display()).into())
}

/// Inspect a workspace-built extension.
/// # Errors
/// Returns invalid metadata or unavailable Java.
pub fn artifact_wire_version(path: &Path) -> Result<u8, Box<dyn Error>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Metadata {
        schema_version: u32,
        wire_version: u8,
    }
    let before = crate::file_evidence::measure(path, 64 * 1024 * 1024)?;
    let output = std::process::Command::new("/opt/bitwig-studio/lib/jre/bin/java")
        .args(["-cp"])
        .arg(&before.path)
        .arg("com.midiworkbench.notes.AdapterMetadata")
        .output()?;
    if !output.status.success() || output.stdout.len() > 4096 || output.stderr.len() > 4096 {
        return Err("Built artifact metadata failed; rebuild the Java adapter with the current Rust facade before deployment".into());
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    let after = crate::file_evidence::measure(path, 64 * 1024 * 1024)?;
    if metadata.schema_version != 1
        || metadata.wire_version == 0
        || before.modified_ns != after.modified_ns
        || before.bytes != after.bytes
    {
        return Err("Built artifact identity/schema changed while measuring metadata".into());
    }
    Ok(metadata.wire_version)
}

/// Refresh the exact development adapter driver.
/// # Errors
/// Returns native refusal or unavailable harness.
pub fn reload_adapter(driver: Option<&str>, refresh: bool) -> Result<Value, Box<dyn Error>> {
    if driver.is_some_and(|id| {
        ![
            "08d96063-f7a1-4f66-bd72-6c0eec8e54db",
            "8a9470ba-b9cf-4ad8-a801-e48721e95d42",
        ]
        .contains(&id)
    }) {
        return Err(
            "Only the exact Notes adapter or development harness driver UUID is accepted".into(),
        );
    }
    if refresh && driver.is_none() {
        return Err("Library refresh requires the exact driver".into());
    }
    invoke(
        json!({"type":"construction","phase":"reload-adapter","driver":driver,"refresh":refresh}),
    )
}

/// Set a disposable native arranger selection.
/// # Errors
/// Returns invalid input or unavailable harness.
pub fn arranger_probe(request: &crate::selection::Request) -> Result<Value, Box<dyn Error>> {
    request.validate()?;
    invoke(json!({"type":"arranger","request":request}))
}
