# Java host harness conventions

## Source order and admission

Start with a working source example, then the published manual for shared concepts,
then the installed implementation for the missing private signature/context:

- [Openwig document dispatch](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L657-L675):
  protected `exec(Runnable)`, asynchronous completion, never wait on the queue.
  Translate the task to **Java**, because this project's JS concurrency probe failed.
- [Openwig native insertion](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L985-L1025):
  command descriptor → explicit owner → typed argument list → result/readback.
  Prefer this complete command pattern over calling an attractive low-level method.
- [Openwig doctor](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L1420-L1510):
  distinctive sentinels in disposable material, independent descriptor readback,
  capabilities admitted individually. Discovery alone does not admit a capability.
- [DrivenByMoss ApplicationImpl](https://github.com/git-moss/DrivenByMoss/blob/93167eb82cd1792a6f4a1ce8e0917201aa1cebd0/src/main/java/de/mossgrabers/bitwig/framework/daw/ApplicationImpl.java#L235-L266):
  call the public Application Undo/Redo commands; do not bypass their execution context.
  This is a public-command example, not evidence for private note enumeration.
- [Bitwig's Java extensions](https://github.com/bitwig/bitwig-extensions):
  lifecycle, observer subscription, settings and host scheduling conventions.
  [Superpowers](https://github.com/kalifrn6-art/bitwig-superpowers) is a native-tool
  and build-gating precedent, but its public repository does not supply editing source.

For each new operation, record the source precedent, hypothesis, exact owner/thread,
ephemeral experiment, independent result and admitted capability before production code.
Use decompilation to answer that focused question, not to assemble a speculative API.

## One small surface

The following names define our adapter conventions, not Bitwig vendor interfaces.
Keep the production implementation in `crates/controller/adapters/bitwig-ctl`:

```java
// Controller init: SDK proxies/settings/observers are registered here.
var document = new DocumentAccess(proxy, admittedBuild);
// An immutable request goes in; immutable document data comes out asynchronously.
document.submit(() -> adapter.readSelection());
document.submit(() -> adapter.read(snapshotRequest));
document.submit(() -> adapter.apply(frozenSnapshot, absolutePlan));
// Controller exit: cancel queued work and close the session; late work cannot apply.
document.close();
```

| Familiar Java controller pattern | Harness equivalent |
| --- | --- |
| `ControllerExtension.init()` creates proxies | Create `DocumentState`/Application/settings and one admitted adapter; wait for a connected document |
| `markInterested` / `addValueObserver` | SDK values keep SDK subscription rules; private observers read the real document and emit immutable snapshots |
| `CursorClip` / `NoteStep` | Editor selection → editable layer → owning clip/content → source note/occurrence handle; no grid address as identity |
| `setX(value)` | Sparse absolute desired field in a typed plan, validated and applied on the document thread |
| Observer confirms a write | Independent full document reread confirms requested state and preservation |
| `host.scheduleTask` / `flush()` | Controller notifications and status updates only; never socket I/O or a wait for document completion |
| `Application.undo/redo` | Host commands through their owning execution path, followed by a separate reread |
| `exit()` | Invalidate generation, cancel pending work, remove private subscriptions, close network workers |

Bind effective selection to `getDocumentWidget()`'s `active_editor_selection`
attribute on the verified 6.1 build. That widget also has `active_selection` and
`inspector_selection`; these can differ. Read actual objects, not their `toString()`.
An editor can retain inactive selections. Preserve the last musical editor when
the controller settings panel takes focus according to the host's active-editor state.

Keep reflection names under `internal`. Do not expose arbitrary reflection, Java
evaluation, filesystem access or vendor objects on the production wire. The wire
contains the Detail Editor model, exact source timing, scoped identities and revisions.
Rust normalizes applicable fields to owned `midi2` types before processing; exact
values, curves, spread and operators remain lossless companion data.

## Development surface and evidence

Expose named, typed probes for `selection`, `read`, `create`, `update`, `delete`,
expression point CRUD, replacement, stale refusal, injected failure and Undo/Redo.
Use the same adapter as production. Fixture creation and failure injection belong
in a separate development harness/artifact, never an arbitrary production RPC.
Diagnostic writes use an agent-managed disposable fixture; do not ask the user
to prepare it. Ordinary authorized musical edits and Save use their task skills
on the intended project, with fresh authority and independent readback. They do
not rerun diagnostic campaigns.

Each invocation completes with `{operationId, hostBuild, thread, arguments,
before, observed, preservation, error}`. Errors stay errors; no empty-success fallback.
Retain failed experiments alongside successful ones. A fresh document generation
invalidates every previous handle and armed fixture.

The implemented development artifact is `bitwig-ctl`'s Maven `harness`
profile. Install with `bash scripts/install-extension.sh --harness`; the separate
controller registers named probes, public Application Undo/Redo and the same
document dispatcher. Use `midi-bridge adapter bitwig probe …` for correlated invocation.
Mutations require `arm --document <observed-generation>` on a disposable project.

For an adapter iteration, build, run `bash scripts/install-extension.sh --harness --code-only`,
then `midi-bridge adapter bitwig probe reload`. The stable harness loads our adapter from a
fresh uniquely named JAR, with a separate loader for our package.
Reload invalidates frozen snapshots and fixture admission. This avoids repeated
Bitwig restarts and the stale-ZIP/classloader failures caused by overwriting a
loaded archive. Ordinary controller installation uses atomic replacement; a
new harness entrypoint still needs a host/controller-library reload. No generic
Java evaluator or hot-loading endpoint exists in the production extension.

Use the optional isolated virtual display and PipeWire device documented in
controller/MANAGED_BITWIG.md. Use **Use Computer** for actual Bitwig UI work;
Xpra and substitute remote-desktop transports are forbidden. Native readiness is
established through fresh data, never process launch or a fixed delay. Screenshots
are troubleshooting only, never systematic control or musical verification.
Production absolute native selection is separate from keyboard focus.
Development history/project fixture actions remain in the disposable harness.

## Native constraints

- Native Pen interaction delegates through an editable editor layer to note
  insertion; the clip factory accepts channel/key/start/duration/velocity. These
  paths still require separate behavioral admission for boundary/collision semantics.
- Expression point insertion returns **void** in the tested overload. Find the
  created point by independent curve enumeration; a null return is not failure.
- The instantaneous-change boolean is a lifetime flag, not rollback.
- Direct recorder Undo can lack the host's replay context. A tested direct call
  failed while removing an expressive note. Use the Application command pattern
  above when testing Undo. Do not compose direct recorder calls into production.
- Change-buffer reversal must restore full modeled state and native payloads,
  then repair only a verified empty independent step before normal group close.
  [The recovery admission](note-crud.md#native-event-order-and-rejected-group-history)
  covers calibrated write families and both history cursors; new native writes
  require their own calibrated failure cases. It does not admit all lifecycle,
  shared-source or concurrency behavior.

See [execution](execution-compatibility.md), [selection](selection-ownership.md),
[CRUD](note-crud.md) and [verification](probe-verification.md) for the owning details.

Use `midi-bridge adapter bitwig probe authority` for construction-probe UUID/revision arguments. Production controller and harness revisions belong to their own adapter lifetimes; obtain mutation authority from the dispatcher that will execute it.
