# Execute on the owner thread and admit the build

## Bootstrap

Build `crates/controller/client-adapter` as a Java-only `.bwextension`, using its
Maven wrapper and Java 25 target. Compile against `com.bitwig:extension-api:25`
with **provided** scope. Never bundle Bitwig application classes. Register the
concrete definition in `META-INF/services/com.bitwig.extension.ExtensionDefinition`.
Set API 25, beta true, zero MIDI input/output ports and fail-on-deprecation true.
Use [the SDK lifecycle guide](../../../../../skills/bitwig-controller/references/java-extensions.md)
for metadata and settings; these SDK calls do not implement document note CRUD.

The inspected Bitwig 6.1 runtime is Java 25.0.2 LTS. Its trimmed image lacks
`java.net.http`. Check the actual installed runtime with `java --list-modules`
before choosing libraries; a desktop JDK's modules do not establish host support.
Package private dependencies in the extension, avoid native libraries, and test
class loading and actual connection against the host runtime.

### Messaging inventory (6.1, October 21, 2026)

Inspect the installed `bin/bitwig.jar`, `bin/libs.jar`, additional classpath
directory and runtime modules before assuming a transport dependency is provided.
This build has no `org.zeromq`/`zmq` implementation in those JARs. Its Log4j
`core/appender/mom/jeromq` classes are an optional integration, not the JeroMQ
implementation. An unrelated obfuscated `zMQ` class is not ZeroMQ evidence.

The published `ControllerHost.connectToRemoteHost`/`RemoteConnection` API provides
TCP, disconnect callbacks, and incoming message framing (32-bit big-endian length,
stripped before the receive callback). This is not a ZeroMQ/ZMTP endpoint. Do not
assume outbound framing is symmetric: `send(byte[])` sends the supplied bytes.

Source precedent: [Openwig's socket setup and send](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L377-L409)
uses a Bitwig listener, receive/disconnect callbacks, and newline-delimited UTF-8
JSON outbound. Its inbound framing is supplied by Bitwig. A new protocol can
instead prepend its own outbound u32 length to make framing symmetric; this is
a proposed convention, not Openwig's existing wire format.

After reading that precedent, inspection of the admitted 6.1 implementation
(`BitwigStudioHost`, `RemoteConnectionSection` and its input thread) established:
Bitwig starts a receiver thread, queues receive/disconnect callbacks through the
control-surface callback path, and disposes the receiver/socket with the section.
`connectToRemoteHost` performs a synchronous `new Socket(host,port)` on the
controller path, reports successful connection via a queued callback, and logs
IOException without a failure callback. `send` directly writes to the socket;
it is not an asynchronous bounded writer. Neither path implements automatic
reconnect or application acknowledgment. Keep private document access behind
`DocumentAccess`; controller callbacks are not document-thread permission.

One further build-specific trap: `RemoteSocketSection.getPort()` discards the
socket's local-port result and returns -1 (confirmed by bytecode), despite the
published documentation. Do not rely on fallback-port discovery on this build.
Before choosing listener/client topology, probe real connection establishment,
failure/reconnect, large sends/backpressure and extension disposal. Research
inspection alone does not admit a production transport.

### Admitted native receive/document context

The [actual host PoC and complete wire evidence](../../../../controller/validation/native-transport/README.md)
subsequently passed: a Java `RemoteConnection.setReceiveCallback` handler runs
on `Control Surface Session` with `isControlSurfaceThread() == true`. It can
call the admitted `DocumentAccess.submit`/native `exec(Runnable)` directly.
That task runs on `main` with `isDocumentThread() == true`, and successfully
reads/edits stored notes through `EditorAdapter61`. No polling, synthetic UI
event, `scheduleTask` trampoline, or private callback queue is needed to deliver
the message. Use this standard internal dispatcher; do not access the document
directly from the receive callback. Send immutable results from an asynchronous
completion worker, never from the document thread.

Verified over the actual Rust/native TCP channel: UTF-8/framed messages, full
independent post-edit readback, stale refusal, bounded injected-failure recovery,
exact fixture restoration and explicit reconnection. This meets the required
handler-context condition; packaging JeroMQ is not required for that condition.
Connection readiness did not guarantee an active musical editor: the first
read returned a readiness error, retained in the evidence. General backpressure,
large-payload, pending-work shutdown and production-session admission remain open.

[JeroMQ](https://github.com/zeromq/jeromq) is pure Java and can be packaged privately
in a `.bwextension`. An ephemeral JeroMQ 0.6.0 probe packaged its classes and a
test entrypoint in one archive, loaded it through an isolated URLClassLoader with
the platform loader as parent, and ran on `/opt/bitwig-studio/lib/jre/bin/java`
(25.0.2). Loopback TCP ROUTER/DEALER exchange in both directions and context close
passed. This demonstrates runtime/archive feasibility, **not** actual controller
lifecycle or Rust interoperability. The JAR was approximately 527 KiB; its Maven
dependency includes pure-Java `jnacl` for crypto (the unencrypted probe did not
exercise that dependency). Package/relocate runtime dependencies if adopted;
no native `libzmq` or JNI is necessary for JeroMQ. Test bounded queues, reconnection,
extension shutdown and Rust interoperability separately before admission.

## Dispatch

The existing [Java probe](../../../../controller/probes/NotesDocumentProbe.java)
resolves protected `exec(Runnable)` on the document-state proxy, checking
`isDocumentThread()` inside a **Java-created** Runnable. On that thread,
`getDocumentGUIWrapper()` and `getDocument()` expose editor and document roots.
Look up declared methods through superclasses and verify exact parameter types.
Unwrap `InvocationTargetException` to retain the real failure.

```java
// Conceptual shape; use the adapter's checked dispatcher, not a second copy.
documentAccess.submit(() -> {
    // Resolve live owners and build an immutable snapshot here.
    return snapshotSelection();
}).thenAcceptAsync(snapshot -> session.send(snapshot), networkExecutor);
```

Keep document operations and host proxies inside the task. Do not let a
completion callback silently run network work on the completing document thread.
Do not call `join()` on the controller/document thread. Return errors as errors,
not empty selections. Cancel/ignore tasks from an exited extension or obsolete
project generation. Close network workers in `exit()` and prevent late results
from touching the next extension instance.

The previous JS-created Runnable collided with scheduled JS callbacks. The
[retained failure and Java-only correction](../../../../controller/validation/editor-selection/README.md#execution-ownership)
are evidence for this Java-only design. Openwig uses a JS Runnable and passive
completion notifications; study its ownership mechanics but do not copy its
cross-thread JS execution.

## Native project invalidation and disposal

A read-time project-pointer comparison cannot invalidate a prepared Plan across
A → B → A between reads. Polling is also insufficient while a session is pending.
On the admitted 6.1 build, `ControlSurfaceProxy.getTargetValue()` exposes
an observable whose `addListener(ums)` / `removeListener(ums)` methods accept
`eventOccurred(Object)`. The native disconnect/connect path emits null/new target
transitions synchronously on the document thread. Keep these private names inside
`EditorAdapter61`; the public project-name observer does not establish identity.

[The isolated probe and production regressions](../../../../controller/validation/project-lifecycle/)
prove callback ownership, all four round-trip transitions and listener removal
from one listener to zero with no subsequent callbacks. Every transition clears
the frozen revision without editing the document. Prepared direct-switch and
round-trip Plans are then refused with independent full payload preservation.

Close native subscriptions on their owner thread. `DocumentAccess.close(resource)`
cancels queued musical work immediately, then dispatches resource disposal through
the native executor without the cancelled-task guard and without waiting. Report
asynchronous disposal failures. The harness must close its old adapter before
replacing the class loader. Actual extension power-off/on rejects old prepared
authority and allows fresh native history-verified work. These probes do not
admit shutdown during native writes or general lifecycle/backpressure behavior.

## Build compatibility

Record the declared host version and capability results in
[host evidence](../../../../controller/validation/editor-selection/README.md).
Keep version-specific method/type mappings in one implementation. Refuse an
unsupported version or missing capability before document access.

When porting:

1. Discover candidate methods by signature, object/descriptor shape, stable
   property identifiers and host command operation IDs.
2. Validate read paths against known sentinels, without mutating the user's work.
3. Validate each write in a disposable fixture with preservation and Undo/Redo.
4. Record the declared version and admitted capabilities only after those pass.

Openwig's [structural resolver](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L677-L800)
uses operation IDs to find command hosts. Treat its claims of stable identifiers
as a discovery strategy to verify on each host, not a vendor compatibility promise.


## Explicit connection lifecycle

Use `adapter bitwig connection list|connect|disconnect|status`; the Rust façade
works even when the service connection is down. Select the live instance by
adapter UUID and host PID/start identity. Require matching wire/build, an alive
host, closed=false and state=ready. Connect publishes one intent; inspect its
terminal status separately. A refusal, missing SDK callback/welcome, EOF or exit
requires a new explicit decision, never a mutation replay. Reconnect success does
not complete an earlier native operation: reconcile that operation and reread the
current document. Controller restart invalidates the adapter-instance UUID.

Installed connectToRemoteHost catches Socket IOException and logs it without
calling the connection callback. Therefore absence of an exception is not success;
the explicit callback/welcome deadline is authoritative. This lifecycle mailbox
is scoped to the admitted Linux host and cannot carry document operations.
