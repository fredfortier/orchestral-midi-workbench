#!/usr/bin/env python3
"""Durable Linux development host. Never changes the desktop profile or default sink."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
STATE = Path(os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state"))) / "midi-workbench/managed"
DISPLAY = ":100"
SINK = "midi_workbench"


def run(*args, **kwargs):
    return subprocess.run(args, check=True, text=True, **kwargs)


def sink():
    sinks = json.loads(run("pactl", "-f", "json", "list", "sinks", capture_output=True).stdout)
    if not any(item["name"] == SINK for item in sinks):
        run("pactl", "load-module", "module-null-sink", f"sink_name={SINK}",
            "sink_properties=device.description=MIDI_Workbench", "rate=48000", "channels=2")


def route_audio():
    """React to graph changes; only the client on our reserved display is rerouted."""
    import select
    import time
    sink()
    with subprocess.Popen(["pw-link", "--monitor", "--id", "--links", "--input", "--output"], stdout=subprocess.PIPE) as monitor:
        while True:
            graph = json.loads(run("pw-dump", capture_output=True).stdout)
            props = {item["id"]: item.get("info", {}).get("props", {}) for item in graph}
            clients = {key for key, p in props.items() if p.get("window.x11.display") == DISPLAY
                       and p.get("application.process.binary", "").startswith("BitwigAudioEngine")}
            nodes = {key for key, p in props.items() if p.get("client.id") in clients}
            sinks = {key for key, p in props.items() if p.get("node.name") == SINK}
            inputs = {p.get("audio.channel"): key for key, p in props.items()
                      if p.get("node.id") in sinks and p.get("port.direction") == "in"}
            for key, p in props.items():
                if p.get("node.id") not in nodes or p.get("port.direction") != "out":
                    continue
                channel = {"out1": "FL", "out2": "FR"}.get(p.get("port.name"))
                target = inputs.get(channel)
                if target is None:
                    continue
                links = {link: lp for link, lp in props.items() if lp.get("link.output.port") == key}
                for link, lp in links.items():
                    if lp.get("link.input.port") != target:
                        run("pw-link", "--disconnect", str(link))
                if not any(lp.get("link.input.port") == target for lp in links.values()):
                    run("pw-link", str(key), str(target))
            # No polling during steady state. Batch graph events, including our own links.
            if not select.select([monitor.stdout], [], [], None)[0]:
                continue
            if not os.read(monitor.stdout.fileno(), 65536):
                raise RuntimeError("PipeWire monitor exited; inspect audio service")
            time.sleep(0.05)
            while select.select([monitor.stdout], [], [], 0)[0]:
                if not os.read(monitor.stdout.fileno(), 65536):
                    raise RuntimeError("PipeWire monitor exited; inspect audio service")


def launch():
    config = json.loads((STATE / "config.json").read_text())
    sink()
    # A private mount namespace preserves the same paths expected by plugins,
    # while profiles, Wine writes, POSIX shared memory and temporary locks are isolated.
    args = ["bwrap", "--bind", "/", "/", "--unshare-ipc", "--dev", "/dev",
            "--dev-bind-try", "/dev/dri", "/dev/dri",
            "--dev-bind-try", "/dev/nvidia0", "/dev/nvidia0",
            "--dev-bind-try", "/dev/nvidiactl", "/dev/nvidiactl",
            "--dev-bind-try", "/dev/nvidia-uvm", "/dev/nvidia-uvm",
            "--tmpfs", "/dev/shm",
            "--ro-bind", str(Path.home() / ".BitwigStudio/installed-packages"), str(STATE / "packages"),
            "--bind", str(STATE / "profile"), str(Path.home() / ".BitwigStudio"),
            "--ro-bind", str(STATE / "packages"), str(Path.home() / ".BitwigStudio/installed-packages"),
            "--bind", str(STATE / "tmp"), "/tmp/bitwig-user",
            "--overlay-src", str(Path.home() / "wine"), "--overlay", str(STATE / "wine-upper"),
            str(STATE / "wine-work"), str(Path.home() / "wine"),
            "--setenv", "DISPLAY", DISPLAY, "--setenv", "PULSE_SINK", SINK,
            "--setenv", "PIPEWIRE_PROPS", f"{{ target.object = {SINK} }}",
            "--unsetenv", "GTK_MODULES", "--unsetenv", "GTK3_MODULES",
            "dbus-run-session", "--", "bitwig-studio", config["project"]]
    # OSC requires a configured MIDI input even when no notes are injected.
    # Opt in only for a qualification host which needs the existing ALSA ports.
    if os.environ.get("MIDI_WORKBENCH_ALSA_DEVICES") == "1":
        index = args.index("dbus-run-session")
        args[index:index] = ["--dev-bind-try", "/dev/snd", "/dev/snd"]
    extensions = os.environ.get("MIDI_WORKBENCH_EXTENSIONS")
    if extensions:
        directory = Path(extensions).resolve(strict=True)
        if not directory.is_dir():
            raise ValueError("MIDI_WORKBENCH_EXTENSIONS must name an existing directory")
        index = args.index("dbus-run-session")
        args[index:index] = ["--ro-bind", str(directory), str(Path.home() / "Bitwig Studio/Extensions")]
    os.execvp(args[0], args)


def setup(project):
    project = project.resolve(strict=True)
    if project.suffix != ".bwproject" or not project.is_file():
        raise ValueError("expected an existing .bwproject")
    if (STATE / "config.json").exists():
        config = json.loads((STATE / "config.json").read_text())
        if config["source"] != str(project):
            raise ValueError("managed setup targets another source; inspect config before changing it")
        install()
        return
    for command in ("bwrap", "pactl", "systemctl", "Xorg", "dbus-run-session", "bitwig-studio", "i3", "xrandr", "pw-link", "pw-dump"):
        if shutil.which(command) is None:
            raise ValueError(f"missing dependency: {command}")
    for directory in ("profile", "profile/installed-packages", "packages", "tmp", "wine-upper", "wine-work", "project", "library"):
        (STATE / directory).mkdir(parents=True, exist_ok=True)
    desktop = Path.home() / ".BitwigStudio"
    # Copy only durable preferences/activation. Do not inherit process locks,
    # unsaved projects, Undo buffers or mutable plugin caches from another host.
    for entry in desktop.iterdir():
        if entry.is_file() and entry.name not in ("lock.pid", "server-port"):
            shutil.copy2(entry, STATE / "profile" / entry.name)
    for directory in ("prefs", "keyboard-shortcuts", "library"):
        shutil.copytree(desktop / directory, STATE / "profile" / directory, dirs_exist_ok=True)
    # Factory packages are immutable installation resources; preserve exact paths.
    project_directory = STATE / "project" / project.stem
    project_directory.mkdir(exist_ok=True)
    destination = project_directory / project.name
    shutil.copy2(project, destination)
    (project_directory / ".bitwig-project").touch(exist_ok=True)
    (STATE / "config.json").write_text(json.dumps({"source": str(project), "project": str(destination),
                                                "display": DISPLAY, "sink": SINK}, indent=2) + "\n")
    install()
    print(json.dumps({"status": "installed", "state": str(STATE), "source": str(project),
                      "next": "configure managed Locations once, then start midi-workbench-bitwig.service"}))


def install():
    """Refresh only our owned units and atomically replace the installed executable."""
    local_bin = Path.home() / ".local/bin"
    local_bin.mkdir(parents=True, exist_ok=True)
    import tempfile
    with tempfile.NamedTemporaryFile(dir=local_bin, prefix=".midi-bridge-", delete=False) as staged:
        staged_path = Path(staged.name)
    try:
        shutil.copy2(ROOT.parent / "target/debug/midi-bridge", staged_path)
        staged_path.replace(local_bin / "midi-bridge")
    finally:
        staged_path.unlink(missing_ok=True)
    units = Path.home() / ".config/systemd/user"
    units.mkdir(parents=True, exist_ok=True)
    common = "[Install]\nWantedBy=default.target\n"
    definitions = {
        "midi-workbench.service": f"[Unit]\nDescription=MIDI Workbench controller session\n[Service]\nExecStart={local_bin}/midi-bridge serve\nRestart=on-failure\n" + common,
        "midi-workbench-display.service": f"[Unit]\nDescription=MIDI Workbench virtual Xorg\n[Service]\nExecStart=/usr/lib/Xorg {DISPLAY} -noreset -nolisten tcp -ac -config {ROOT}/scripts/managed-xorg.conf -logfile {STATE}/Xorg.log\nRestart=on-failure\n" + common,
        "midi-workbench-wm.service": f"[Unit]\nDescription=MIDI Workbench display focus and window management\nRequires=midi-workbench-display.service\nAfter=midi-workbench-display.service\n[Service]\nEnvironment=DISPLAY={DISPLAY}\nExecStartPre=/usr/bin/python3 {Path(__file__).resolve()} wait-display\nExecStart=/usr/bin/i3 -c {ROOT}/scripts/managed-i3.conf\nRestart=on-failure\n" + common,
        "midi-workbench-audio.service": f"[Unit]\nDescription=Route managed Bitwig to its recording sink\nAfter=pipewire-pulse.service\n[Service]\nExecStart=/usr/bin/python3 {Path(__file__).resolve()} route-audio\nRestart=on-failure\nRestartSec=2\n" + common,
        "midi-workbench-bitwig.service": f"[Unit]\nDescription=Isolated MIDI Workbench Bitwig\nRequires=midi-workbench-wm.service midi-workbench-audio.service\nWants=midi-workbench.service\nAfter=midi-workbench-wm.service midi-workbench.service midi-workbench-audio.service\n[Service]\nExecStart=/usr/bin/python3 {Path(__file__).resolve()} run\nKillMode=control-group\nKillSignal=SIGTERM\nTimeoutStopSec=40\n" + common,
    }
    for name, content in definitions.items():
        path = units / name
        if path.exists() and "MIDI Workbench" not in path.read_text() and str(Path(__file__).resolve()) not in path.read_text():
            raise ValueError(f"refusing to overwrite unowned unit: {path}")
        path.write_text(content)
    run("systemctl", "--user", "daemon-reload")
    run("systemctl", "--user", "enable", *definitions)


def wait_display():
    import time
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        result = subprocess.run(["xrandr", "--display", DISPLAY, "--current"], capture_output=True, text=True)
        if result.returncode == 0 and "1600x1000" in result.stdout:
            return
        time.sleep(0.1)
    raise TimeoutError("managed Xorg mode did not become ready; inspect its unit and Xorg.log")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("setup").add_argument("--project", type=Path, required=True)
    for name in ("install", "run", "wait-display", "sink", "route-audio", "status"):
        sub.add_parser(name)
    args = parser.parse_args()
    if args.action == "setup":
        setup(args.project)
    elif args.action == "install":
        install()
    elif args.action == "run":
        launch()
    elif args.action == "wait-display":
        wait_display()
    elif args.action == "sink":
        sink()
    elif args.action == "route-audio":
        route_audio()
    else:
        print((STATE / "config.json").read_text())
        run("systemctl", "--user", "--no-pager", "status", "midi-workbench.service",
            "midi-workbench-display.service", "midi-workbench-bitwig.service")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError, TimeoutError) as error:
        print(f"managed Bitwig: {error}", file=sys.stderr)
        sys.exit(1)
