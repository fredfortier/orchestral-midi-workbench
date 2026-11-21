---
name: bitwig-launcher
description: Launches or reuses an agent-managed Bitwig Studio on Linux.
---

# Manage Bitwig Studio


## Single-instance constraint

**Multiple Bitwig Studio instances are unsupported. At most one instance may run
across all displays and launch modes.** A virtual-display instance counts as a
running instance when the user requests normal desktop Bitwig, and vice versa.
Never launch another instance to satisfy a different display request.

Before every launch, inspect all Bitwig application processes and their exact
PID/start identities and displays:

- If one instance is running in the requested mode, reuse it.
- If any instance is running in another mode, stop: explain the conflict and ask
  the user to quit it, unless the user has already authorized stopping it.
- If multiple instances are found, do not launch. Resolve them first within the
  user's authorization.
- When stopping is authorized, terminate only the identified processes, verify
  that all Bitwig application instances have exited, then launch exactly one.
  Never terminate by process name or start a competing instance while waiting.
- If no instance is running, launch `bitwig-studio` normally by default. Use the
  virtual-display procedure below only when the user requests it.

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
