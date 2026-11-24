//! Process-only adapter for the repo-owned Wine Windows kontakt-ctl package.
use clap::Args;
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Args, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Runtime {
    #[arg(long)]
    pub wine_prefix: PathBuf,
    #[arg(long)]
    pub wine_runtime: PathBuf,
    #[arg(long)]
    pub display: String,
    #[arg(skip)]
    #[serde(skip)]
    native_host: Option<u32>,
}
impl Runtime {
    #[must_use]
    pub fn new(wine_prefix: PathBuf, wine_runtime: PathBuf, display: String) -> Self {
        Self {
            wine_prefix,
            wine_runtime,
            display,
            native_host: None,
        }
    }

    /// Bind runtime resources to the admitted native process's mount namespace.
    /// # Errors
    /// Refuses a different display/process or inaccessible native Wine prefix.
    pub(crate) fn for_host(mut self, pid: u32) -> Result<Self, crate::control::Error> {
        let fail = |e: std::io::Error| crate::control::Error::invalid(e.to_string());
        let process = PathBuf::from(format!("/proc/{pid}"));
        let executable = fs::read_link(process.join("exe")).map_err(fail)?;
        let command = fs::read(process.join("cmdline")).map_err(fail)?;
        let launched_as_bitwig = command.split(|byte| *byte == 0).next()
            == Some(b"/opt/bitwig-studio/bin/BitwigStudio".as_slice());
        if executable != Path::new("/opt/bitwig-studio/lib/jre/bin/java") || !launched_as_bitwig {
            return Err(crate::control::Error::invalid(
                "Admitted native process is no longer Bitwig",
            ));
        }
        let environment = fs::read(process.join("environ")).map_err(fail)?;
        let display = format!("DISPLAY={}", self.display);
        if !environment
            .split(|byte| *byte == 0)
            .any(|entry| entry == display.as_bytes())
        {
            return Err(crate::control::Error::invalid(
                "Loader display differs from the admitted Bitwig instance",
            ));
        }
        if !self.wine_prefix.is_absolute() {
            return Err(crate::control::Error::invalid(
                "Absolute Wine prefix required",
            ));
        }
        self.wine_prefix = process.join("root").join(
            self.wine_prefix
                .strip_prefix("/")
                .map_err(|e| crate::control::Error::invalid(e.to_string()))?,
        );
        fs::metadata(&self.wine_prefix).map_err(fail)?;
        self.native_host = Some(pid);
        Ok(self)
    }
}
const FILES: &[(&str, &[u8])] = &[
    (
        "assets/targets/remove-instrument-dark.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/remove-instrument-dark.png"),
    ),
    (
        "kontakt_ctl/rack.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/rack.py"),
    ),
    (
        "assets/targets/instrument-navigator.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/instrument-navigator.png"),
    ),
    (
        "assets/targets/remove-instrument.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/remove-instrument.png"),
    ),
    (
        ".python-version",
        include_bytes!("../adapters/kontakt-ctl/.python-version"),
    ),
    (
        "assets/targets/batch_resave_menu.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/batch_resave_menu.png"),
    ),
    (
        "assets/targets/browse_for_folder_button.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/browse_for_folder_button.png"),
    ),
    (
        "assets/targets/file__menu.png",
        include_bytes!("../adapters/kontakt-ctl/assets/targets/file__menu.png"),
    ),
    (
        "kontakt_ctl/__init__.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/__init__.py"),
    ),
    (
        "kontakt_ctl/batch_resave.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/batch_resave.py"),
    ),
    (
        "kontakt_ctl/capture.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/capture.py"),
    ),
    (
        "kontakt_ctl/cli.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/cli.py"),
    ),
    (
        "kontakt_ctl/commands.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/commands.py"),
    ),
    (
        "kontakt_ctl/config.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/config.py"),
    ),
    (
        "kontakt_ctl/input_ops.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/input_ops.py"),
    ),
    (
        "kontakt_ctl/inspect_rack.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/inspect_rack.py"),
    ),
    (
        "kontakt_ctl/launcher.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/launcher.py"),
    ),
    (
        "kontakt_ctl/load_nki.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/load_nki.py"),
    ),
    (
        "kontakt_ctl/logging.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/logging.py"),
    ),
    (
        "kontakt_ctl/ui_ops.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/ui_ops.py"),
    ),
    (
        "kontakt_ctl/window_ops.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/window_ops.py"),
    ),
    (
        "pyproject.toml",
        include_bytes!("../adapters/kontakt-ctl/pyproject.toml"),
    ),
    ("uv.lock", include_bytes!("../adapters/kontakt-ctl/uv.lock")),
    (
        "kontakt_ctl/resave_native.py",
        include_bytes!("../adapters/kontakt-ctl/kontakt_ctl/resave_native.py"),
    ),
];

/// Validate process resources before the caller creates its receiver.
/// # Errors
/// Refuses absent Windows runtime, incompatible library mapping and occupied output.
pub fn preflight(runtime: &Runtime, nki: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    let nki = nki.canonicalize()?;
    if !nki.is_file() {
        return Err("NKI must be a regular file".into());
    }
    let root = runtime.wine_prefix.join("dosdevices/x:").canonicalize()?;
    nki.strip_prefix(root)
        .map_err(|_| "NKI must be beneath the Wine X: library mapping")?;
    if !runtime.wine_runtime.join(".venv/Scripts/uv.exe").is_file() {
        return Err("Windows uv.exe is absent from the supplied Kontakt runtime".into());
    }
    if output.exists() {
        return Err("Kontakt output directory must be fresh".into());
    }
    let mut command = wine_command(runtime)?;
    environment(runtime, &mut command)?;
    Ok(())
}

/// Observe visible Win32 host windows without changing their visibility.
/// # Errors
/// Refuses runtime/process failures; takes no window input.
pub fn windows(runtime: &Runtime, output: &Path) -> Result<Value, Box<dyn Error>> {
    run_tool(
        runtime,
        output,
        &["windows".into()],
        &json!({"kind":"window-inventory"}),
    )
}
/// Load only into the exact Win32 editor qualified by the assignment.
/// # Errors
/// Retains process/dialog failures without closing or replaying the receiver.
pub fn load(
    runtime: &Runtime,
    nki: &Path,
    output: &Path,
    operation: &str,
    owner_window: u64,
    owner_pid: u32,
) -> Result<Value, Box<dyn Error>> {
    preflight(runtime, nki, output)?;
    let nki = nki.canonicalize()?;
    let root = runtime.wine_prefix.join("dosdevices/x:").canonicalize()?;
    let relative = nki
        .strip_prefix(root)
        .map_err(|_| "NKI must be beneath the Wine X: library mapping")?;
    let args = vec![
        "load-nki".into(),
        "--owner-window".into(),
        owner_window.to_string(),
        "--owner-pid".into(),
        owner_pid.to_string(),
        relative
            .to_str()
            .ok_or("UTF-8 NKI path required")?
            .replace('/', "\\"),
    ];
    let response = run_tool(
        runtime,
        output,
        &args,
        &json!({"operationId":operation,"relativePath":relative,"ownerWindow":owner_window,"ownerPid":owner_pid}),
    )?;
    if response["data"]["load_confirmed"] != true || response["data"]["rack_instrument_count"] != 1
    {
        return Err(
            "Exact NKI load is unconfirmed; retain original transaction and do not close or replay"
                .into(),
        );
    }
    Ok(response)
}
fn run_tool(
    runtime: &Runtime,
    output: &Path,
    args: &[String],
    request: &Value,
) -> Result<Value, Box<dyn Error>> {
    let uv = runtime
        .wine_runtime
        .join(".venv/Scripts/uv.exe")
        .canonicalize()?;
    fs::create_dir(output)?;
    let source = output.canonicalize()?.join("kontakt-ctl");
    fs::create_dir(&source)?;
    for (name, bytes) in FILES {
        let path = source.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
    }
    fs::write(
        source.join("source-provenance.json"),
        serde_json::to_vec_pretty(&json!({
            "schema":"kontakt-ctl.source/v1",
            "owner":"MIDI2.0Workbench/controller",
            "files":FILES.iter().map(|(name,bytes)|json!({
                "path":name,"bytes":bytes.len()
            })).collect::<Vec<_>>()
        }))?,
    )?;
    let mut command = wine_command(runtime)?;
    command
        .args(["start", "/wait", "/b", "/d"])
        .arg(windows_path(&source)?)
        .arg("/unix")
        .arg(uv)
        .args(["run", "--no-sync", "kontakt-ctl", "--json"])
        .args(args)
        .env(
            "UV_PROJECT_ENVIRONMENT",
            windows_path(&runtime.wine_runtime.join(".venv").canonicalize()?)?,
        )
        .env("PYTHONPATH", windows_path(&source)?)
        .current_dir(&source)
        .stdin(Stdio::null())
        .stdout(fs::File::create(output.join("response.json"))?)
        .stderr(fs::File::create(output.join("wine.log"))?);
    environment(runtime, &mut command)?;
    crate::files::publish(
        &output.join("request.json"),
        &json!({"request":request,"command":args,"runtime":runtime}),
    )?;
    let mut process = command.spawn()?;
    crate::files::publish(
        &output.join("process.json"),
        &json!({"pid":process.id(),"deadlineSeconds":360,"timeoutIsCancellation":false}),
    )?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(360);
    let status = loop {
        if let Some(status) = process.try_wait()? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!("Loader deadline expired; process {} may still act; retain original operation and block dependent changes",process.id()).into());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    let response: Value = serde_json::from_slice(&fs::read(output.join("response.json"))?)?;
    if !status.success() || response["schema"] != "kontakt-ctl.result/v1" || response["ok"] != true
    {
        return Err(format!(
            "kontakt-ctl failed ({status}): {response}; see {}",
            output.display()
        )
        .into());
    }
    Ok(response)
}
fn windows_path(path: &Path) -> Result<String, Box<dyn Error>> {
    if !path.is_absolute() {
        return Err("Absolute runtime path required".into());
    }
    Ok(format!(
        "Z:{}",
        path.to_str()
            .ok_or("UTF-8 path required")?
            .replace('/', "\\")
    ))
}
fn wine_command(runtime: &Runtime) -> Result<Command, Box<dyn Error>> {
    let pid = runtime
        .native_host
        .ok_or("Loader requires an admitted native host")?;
    // Join only differing namespaces. Linux rejects reassociating the current
    // user namespace with itself on an ordinary desktop Bitwig launch.
    let mut namespaces = Vec::new();
    for (namespace, option) in [("user", "--user"), ("mnt", "--mount"), ("ipc", "--ipc")] {
        if fs::read_link(format!("/proc/self/ns/{namespace}"))?
            != fs::read_link(format!("/proc/{pid}/ns/{namespace}"))?
        {
            namespaces.push(option);
        }
    }
    if namespaces.is_empty() {
        return Ok(Command::new("wine"));
    }
    let mut command = Command::new("nsenter");
    command.args(["--target", &pid.to_string()]);
    if namespaces.contains(&"--user") {
        command.arg("--preserve-credentials");
    }
    command.args(namespaces).args(["--", "wine"]);
    Ok(command)
}
fn environment(runtime: &Runtime, command: &mut Command) -> Result<(), Box<dyn Error>> {
    let pid = runtime
        .native_host
        .ok_or("Loader requires an admitted native host")?;
    let environment = fs::read(format!("/proc/{pid}/environ"))?;
    for name in ["WINEESYNC", "WINEFSYNC"] {
        command.env_remove(name);
        let prefix = format!("{name}=");
        if let Some(value) = environment
            .split(|byte| *byte == 0)
            .filter_map(|entry| std::str::from_utf8(entry).ok())
            .find_map(|entry| entry.strip_prefix(&prefix))
        {
            command.env(name, value);
        }
    }
    let root = PathBuf::from(format!("/proc/{pid}/root"));
    let native_prefix = Path::new("/").join(runtime.wine_prefix.strip_prefix(root)?);
    command
        .env("DISPLAY", &runtime.display)
        .env("WINEPREFIX", native_prefix);
    Ok(())
}
