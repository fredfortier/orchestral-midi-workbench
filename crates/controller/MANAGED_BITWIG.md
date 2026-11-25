# Disposable Linux Bitwig host

[managed-bitwig.py](scripts/managed-bitwig.py) provisions an isolated project/profile,
Xorg display and audio sink for native qualification. This is optional test
infrastructure, not a prerequisite for operating the user's existing Bitwig.

From `crates/`, inspect the script's help before provisioning:

```sh
python3 controller/scripts/managed-bitwig.py --help
```

`setup --project /absolute/disposable-source.bwproject` copies the project/profile
under `$XDG_STATE_HOME/midi-workbench/managed` (default `~/.local/state/...`) and
writes user-systemd units. It requires bubblewrap overlay support, dummy Xorg,
i3, Xorg tools, PipeWire/Pulse tools and installed Bitwig. Source paths and installed
packages remain outside the writable profile. Repeated setup refuses a different
source rather than replacing the managed project.

The script reserves display `:100` and sink `midi_workbench`; inspect its definitions
before starting units. Configure the managed profile's Locations and controller
once; preserve recovery inputs. Build/deploy matching artifacts explicitly.
A successful unit start is not proof of project/editor readiness.

Use the script's `status` for inspection. Close Bitwig through a qualified native
action using Use Computer before stopping the disposable service. This helper
has no Bitwig UI-control or close command. Service termination is environment
cleanup for the explicitly disposable fixture; it can force-clean its process
group after its timeout. Never stop the user's DAW or shared Wine server.

Use one Bitwig application instance. For a dedicated qualification listener,
configure its native and HTTP API ports explicitly.
`MIDI_WORKBENCH_EXTENSIONS` overrides the managed extension directory;
`MIDI_WORKBENCH_ADAPTER_PORT` selects the native port from Bitwig. The script's explicit
`MIDI_WORKBENCH_ALSA_DEVICES=1` option exposes ALSA devices when OSC needs a virtual
input; do not commandeer physical ports or the desktop feedback socket.

Validate actual host generation/owners before native writes. Valid silent audio,
a display connection or a service health response is not musical qualification.
See the [native verification contract](../../docs/architecture/controller.md#wire-deployment-and-verification).
