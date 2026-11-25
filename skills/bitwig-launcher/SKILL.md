---
name: bitwig-launcher
description: Reuse the running Bitwig Studio on Linux, or launch it when no instance exists. Apply the single-instance preflight to production, controller development and smoke tests.
---

# Manage Bitwig Studio


## Single-instance preflight

**Reuse the running Bitwig Studio. At most one application instance may run on
this machine.** Count across all displays, profiles, services and launch
methods. Apply this before production work, adapter development, qualification
and smoke tests. Xpra is prohibited.

1. Inspect Bitwig application PIDs, start identities and displays before any
   launch or host change. Launcher wrappers, splash screens, audio engines
   and plug-in hosts do not count as application instances.
2. If one instance is running, verify its project through
   `midi-bridge controller info` and use that instance. A different display,
   disposable project, fixture, listener port, extension build or wire version
   does not permit a second instance.
3. If the running session cannot support the task, diagnose that session.
   Closing, reopening or switching the active project, or restarting Bitwig
   or its services, requires the user's explicit request for that action. When
   the needed action is unauthorized or unavailable, finish independent work
   and report the blocked live check.
4. If multiple application instances exist, pause live work until the conflict
   is resolved within the user's authorization. Stop only a specifically
   identified process when authorized.
5. Launch only after verifying zero application instances. Launch normally by
   default. Use the virtual-display procedure only when the user requests it.

Verify after launch that exactly one Bitwig application instance is running and
that it is on the intended display. Launcher wrappers, splash screens, audio
engines and plug-in hosts are not separate application instances.

## Prepare once; reuse on subsequent calls

### Dedicated Sink for Managed Launches 

Use a dedicated PipeWire sink and explicitly record its monitor when audio
analysis is requested. Verify engine stream links and the exact sink/monitor IDs;
do not change the user's default output device. A running engine or nonempty WAV
does not establish correct routing, signal or musical fidelity.

### Virtual Display Case

Use an Xorg dummy display with a measured nonzero refresh rate. The prior
Kontakt/DXVK experiment failed on Xvfb's zero-Hz mode. Bind managed processes to
their exact PID/start identity and display. Never terminate Bitwig by process
name or manipulate the user's desktop instance as a recovery shortcut.

## Ensure Controller Listener Service

Keep one controller listener running independently of agent turns. Observe actual
native application/project readiness; a successful launcher or fixed delay is
insufficient. 


## Open and select

Resolve the requested saved project using file metadata and native identity.
Record the chosen source. Use an isolated project copy for experiments that may
change content/history; do not save over the user's source without authorization.
Opening and selecting in the agent-owned instance is part of this workflow.

Use [bitwig-explore](../bitwig-explore/SKILL.md) for native arranger discovery
and deliberate selection. Independently verify native Object/Time/note selection
and every intended clip/track owner; screenshots and injected input are not a fallback.

Use `midi-bridge controller selection read` to confirm native project and
selection readiness for either kind of host. When the task requires a musical
read, use the selected note clips or pass exact clip paths to
`midi-bridge controller midi get-all --paths paths.json`. Count the actual
note and automation arrays with jq.

Open [troubleshooting](references/troubleshooting.md) whenever launch, project
open, controller access or recording fails. Fix the durable cause and verify the
ordinary command again; do not turn an unexplained setup ritual into the UX.
