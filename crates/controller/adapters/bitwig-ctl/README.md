# Bitwig Java adapter

This extension contains host-specific native execution. Its intended authority is
specified in the [native host contract](../../ARCHITECTURE.md#native-host-execution).

## Build and packaging

From this directory, with a JDK accepted by [pom.xml](pom.xml):

```sh
./mvnw verify
./mvnw -P harness verify
```

The POM targets Java 25 and SDK API 25. Production and harness builds use separate
`target/main` and `target/harness` directories. The normal artifact is
`target/main/NotesTransformer.bwextension`; SDK classes are provided by the host.
Deploy with `controller/scripts/install-extension.sh` from crates/; installation does not reload
classes in a running host. Inspect command help and actual connection identity.

## Native boundary

[EditorSession](src/main/java/com/midiworkbench/notes/EditorSession.java) owns the
SDK connection; [internal/](src/main/java/com/midiworkbench/notes/internal/) contains
native mappings. Host objects remain on their owning document thread/lifetime.
The document thread never waits on network I/O. Build admission, scoped authority,
Undo and independent native readback are separate obligations.

The development harness provides explicitly armed disposable probes. Its methods,
fixture constants and historical observations do not establish production support.
Use [RUNBOOK.md](../../RUNBOOK.md) for qualification and
[WORKPAD.md](../../WORKPAD.md) for repair gaps. The development filesystem mailbox exposes only named disposable probes. Production
messages expose complete clip content and typed folders, never a property graph.

## Device parameters and automation

```sh
midi-bridge controller automation read --target device.json
midi-bridge controller automation set --input curves.json --output after.json
```

Discover the named device's parameters and mapped Remote Controls, then submit
named curves as one atomic batch. Native track CC inputs need no device discovery; explicit instrument plug-in
parameters and mapped macros use their discovered physical domains. See [automation edits](../../../../skills/bitwig-edit/references/automation-edits.md)
for inputs and source-coordinate rules. Existing controller assignments survive.

`instrument slot update --target receiver.json --operation helper-01 --midi-cc`
is an explicit legacy device-placement helper; conventional track MIDI needs no
sender and the helper creates no curves.

Deployment follows the [wire and deployment contract](../../ARCHITECTURE.md#wire-deployment-and-verification). The installer uses immutable
wire-named artifacts and refuses changed content under an installed wire number.
Require the exact installed extension version and matching live wire before writing.
