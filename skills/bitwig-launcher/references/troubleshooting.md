# Controller and managed-host failures

Use actual errors and current PID/start identity. Preserve failed read evidence
and diagnose its owning boundary before retrying. Never automatically replay a
mutation: reconcile its original operation ID and current native state first.

| Symptom | Diagnosis and recovery |
| --- | --- |
| Connection refused on 8788 | Inspect the listener and durable service status/log. Start the single owned listener, then verify native readiness. Do not start a competing service. |
| Handshake failure or disconnected adapter | Inspect `adapter bitwig connection list`; ignore dead host descriptors. Start the listener first. An admitted session recovers a lost connection with bounded backoff; protocol/handshake refusal remains fail-stop. For initial setup failure, request one connect for the intended live instance and inspect its result. Keep the service persistent. |
| Version mismatch | Identify the CLI, connected service, loaded adapter and Bitwig versions. Report the mismatch as a missing prerequisite for dependent production actions. |
| Bitwig exits or restarts with the listener | Reconfirm the intended instance and project before reconnecting. Reconcile pending operations by their original IDs. If the restart repeats, report the unstable session prerequisite. |
| Multiple adapters | Something went very wrong. State your reasoning, stop and start again. |
| No active editor or wrong selection | Inspect the managed display and native selection. Select the arranger note clip itself with Object Selection. Preserve the native adapter's Inspector fallback; do not invent cached selection. |
| Ownership/geometry/reference error | Escalate. |
| Virtual-display plug-in crash | Measure actual mode/refresh and read Xorg/Bitwig/Wine logs. Use dummy Xorg with nonzero refresh for the admitted Kontakt/DXVK environment. |
| Silent/wrong recording | Inspect exact sink/monitor IDs and engine stream links. Route only the managed engine to its sink; explicitly record that monitor and check frames, format and signal independently. |
| `Null native owner for method getValue` in the scoped read | Escalate. |
| No editor after clicking a visible clip | A bare X server can have keyboard focus on the root window. Run the managed window manager before Bitwig; verify focus and then native selection. Clicking an unfocused renderer did not establish the active editor in the initial probe. |
| Project-folder prompt after copying a project | Create a named project directory containing the same-named `.bwproject` and its `.bitwig-project` marker. Bitwig can relocate a loose file into a native project folder when saving; observe the actual saved path and update the managed launch binding once. Setup uses a named folder. Do not relocate the user's source project. |
| Recovery dialog after stopping the managed service | Use computer to complete as reasonable. |
| `No space left on device` for inotify, with free disk space | Escalate. |

The native PipeWire backend initially kept its saved System Out connections even
with `target.object=midi_workbench`. The managed audio service therefore watches
graph changes and links only the engine client whose `window.x11.display` is
the managed host's observed display to the dedicated sink. It never changes the default output device.
`pw-link --monitor --id` also needs a listing selector: use `--links --input
--output`; without one the monitor exits instead of subscribing.

If startup or recovery cannot be verified, report the missing production
prerequisite and leave dependent actions pending.
