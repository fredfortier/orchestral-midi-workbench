package com.midiworkbench.notes;

import com.bitwig.extension.controller.api.ControllerHost;
import com.bitwig.extension.controller.api.RemoteConnection;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.internal.DocumentAccess;
import com.midiworkbench.notes.internal.EditorAdapter61;
import com.midiworkbench.notes.internal.HostBuild;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Consumer;

/** One native editing session. All protocol transitions run on the controller thread. */
public final class EditorSession implements AutoCloseable,ConnectionControl.Target {
    private static final long DEADLINE=TimeUnit.SECONDS.toNanos(15);
    private final ControllerHost host;
    private final DocumentAccess document;
    private final EditorAdapter61 editor;
    private final Consumer<String> status;
    private final String adapter=UUID.randomUUID().toString();
    private final ArrayBlockingQueue<Runnable> completions=new ArrayBlockingQueue<>(16);
    private final ThreadPoolExecutor writer=new ThreadPoolExecutor(1,1,0,TimeUnit.SECONDS,
            new ArrayBlockingQueue<>(8),r->{Thread t=new Thread(r,"Notes session writer");t.setDaemon(true);return t;});
    private final AtomicLong writingSince=new AtomicLong();
    private static final long REPLAY_BYTES=32L*1024*1024;
    private record Receipt(com.fasterxml.jackson.databind.JsonNode plan,String result,long bytes) {}
    private final ReplayCache<Receipt> receipts=new ReplayCache<>(16,REPLAY_BYTES,Receipt::bytes,receipt->true);
    private volatile RemoteConnection connection;
    private volatile boolean active=true;
    private record NetworkFailure(String reason,boolean transport) {}
    private volatile NetworkFailure networkError;
    private volatile Operation pending;
    private boolean ready;
    private boolean connecting;
    private long commandHighWater;
    private final ReplayCache<DocumentRequest> documentRequests=new ReplayCache<>(16,REPLAY_BYTES,entry->entry.bytes,entry->entry.receipt!=null);
    private static final class DocumentRequest {
        final ObjectNode input;
        final long inputBytes;
        String receipt;
        long bytes;
        DocumentRequest(ObjectNode input) {this.input=input.deepCopy();inputBytes=input.toString().getBytes(StandardCharsets.UTF_8).length;bytes=inputBytes;}
        void complete(String result){receipt=result;bytes=inputBytes+result.getBytes(StandardCharsets.UTF_8).length;}
    }
    private long sequence;
    private long connectAt;
    private boolean initialConnect=true;
    private boolean recoveringConnection;
    private long reconnectAt=Long.MAX_VALUE;
    private long reconnectDelay=TimeUnit.MILLISECONDS.toNanos(250);
    private long connectionAttempt;
    private long connectAttempts;
    private String connectionError;
    private String epoch;

    private static final class Operation {
        final String id;
        final long deadline=System.nanoTime()+TimeUnit.SECONDS.toNanos(30);
        String request;
        ObjectNode snapshot;
        String plan;
        boolean applying;
        volatile String receipt;
        Operation(String id){this.id=id;}
    }

    public EditorSession(ControllerHost host,DocumentAccess document,EditorAdapter61 editor,
                         Consumer<String> status) {
        this.host=host;this.document=document;this.editor=editor;this.status=status;
    }

    /** Controller lifecycle/notification tick only; receive callbacks dispatch native work directly. */
    public void tick() {
        if(!active)return;
        for(Runnable completion;(completion=completions.poll())!=null;)completion.run();
        if(networkError!=null){
            NetworkFailure error=networkError;networkError=null;
            boolean recover=error.transport() && (ready || recoveringConnection);
            disconnect(error.reason());
            if(recover){recoveringConnection=true;scheduleReconnect();}
        }
        long now=System.nanoTime();
        if(writingSince.get()!=0 && now-writingSince.get()>DEADLINE)disconnect("Write deadline expired; reconnect to reconcile");
        if(connecting && now-connectAt>DEADLINE) {
            boolean retry=recoveringConnection && connection==null;
            disconnect("Connection/handshake deadline expired");
            if(retry)scheduleReconnect();
        }
        if(initialConnect){initialConnect=false;requestConnect();}
        if(recoveringConnection && !connecting && connection==null && now>=reconnectAt)connect();
        Operation operation=pending;
        if(operation!=null && !operation.applying && operation.receipt==null && now>operation.deadline && operation.request!=null)
            reject(operation,"Operation deadline expired before mutation");
    }

    @Override public ObjectNode connectionStatus() {
        var state=Wire.object();state.put("adapter",adapter);state.put("wireVersion",Wire.VERSION);state.put("hostBuild",HostBuild.VERIFIED_61);
        state.put("state",!active?"closed":ready?"ready":connecting?"connecting":connectionError!=null?"failed":"disconnected");state.put("error",connectionError);state.put("attempt",connectAttempts);return state;
    }
    @Override public void requestDisconnect(){initialConnect=false;recoveringConnection=false;disconnect("Explicit disconnect; native outcomes require reconciliation");connectionError=null;}
    @Override public void requestConnect() {recoveringConnection=false;connect();}
    /** Isolated qualification may use a different listener without touching the desktop service. */
    private static int adapterPort() {
        String configured=System.getenv("MIDI_WORKBENCH_ADAPTER_PORT");
        if(configured==null)return 8788;
        int port=Integer.parseInt(configured);
        if(port<1 || port>65535)throw new IllegalArgumentException("Invalid MIDI_WORKBENCH_ADAPTER_PORT");
        return port;
    }
    private void connect() {
        if(!active || connection!=null || connecting)throw new IllegalStateException("Connection already active/connecting or controller closed; no attempt dispatched");
        reconnectAt=Long.MAX_VALUE;initialConnect=false;connecting=true;connectionError=null;connectAt=System.nanoTime();long attempt=++connectionAttempt;connectAttempts++;
        status.accept("Connecting once; attempt "+connectAttempts);
        try {host.connectToRemoteHost("127.0.0.1",adapterPort(),connected->{
            if(!active || !connecting || connectionAttempt!=attempt || connection!=null){connected.disconnect();return;}
            connection=connected;ready=false;
            connected.setDisconnectCallback(()->{
                if(connection!=connected)return;
                boolean admitted=ready;
                disconnect("Service disconnected; native outcomes require reconciliation");
                // Only an admitted session enables recovery. Protocol refusal before
                // Welcome remains fail-stop; reconnecting never grants edit authority.
                if(admitted){recoveringConnection=true;scheduleReconnect();}
            });
            connected.setReceiveCallback(bytes->receive(bytes,connected));
            ObjectNode hello=Wire.object();hello.put("type","hello");hello.put("version",Wire.VERSION);
            hello.put("adapter",adapter);hello.put("hostBuild",HostBuild.VERIFIED_61);send(hello.toString());
        });}catch(Exception error){disconnect("Connection attempt failed: "+error);if(recoveringConnection)scheduleReconnect();}
    }

    private void receive(byte[] bytes,RemoteConnection receivedOn) {
        if(!active || receivedOn!=connection)return;
        try {
            if(bytes.length>Wire.MAX_MESSAGE_BYTES)throw new IllegalArgumentException("Oversized service frame");
            ObjectNode message=Wire.parse(new String(Frame.decode(bytes),StandardCharsets.UTF_8));
            String type=message.required("type").asText();
            switch(type) {
                case "documentRequest" -> execute(message,receivedOn);
                case "welcome" -> {
                    keys(message,Set.of("type","version","epoch","maxMessageBytes"));
                    if(message.required("version").intValue()!=Wire.VERSION || message.required("maxMessageBytes").intValue()!=Wire.MAX_MESSAGE_BYTES)
                        throw new IllegalArgumentException("Incompatible service protocol");
                    String nextEpoch=message.required("epoch").asText();
                    boolean restarted=epoch!=null && !epoch.equals(nextEpoch);
                    if(!nextEpoch.equals(epoch)){commandHighWater=0;documentRequests.clear();}
                    epoch=nextEpoch;ready=true;connecting=false;connectionError=null;
                    recoveringConnection=false;reconnectDelay=TimeUnit.MILLISECONDS.toNanos(250);
                    Operation operation=pending;
                    if(restarted && operation!=null) {
                        // A new epoch never authorizes replay. A fresh Update is a new explicit request.
                        pending=null;status.accept("Service restarted; previous outcome requires fresh observation. Press Update for a new operation.");
                    } else if(operation!=null) {
                        if(operation.receipt!=null)send(operation.receipt);
                        else if(operation.request!=null) {
                            ObjectNode request=Wire.object();request.put("type","status");request.put("operation",operation.id);send(request.toString());
                        }
                    } else status.accept("Ready");
                }
                case "plan" -> {
                    keys(message,Set.of("type","operation","plan"));
                    String id=message.required("operation").asText(),plan=message.required("plan").toString();
                    Operation operation=pending;
                    if(operation==null || !id.equals(operation.id)) {
                        Receipt previous=receipts.get(id);
                        if(previous==null || previous.plan()==null || !previous.plan().equals(message.required("plan")))
                            throw new IllegalArgumentException("Unsolicited or conflicting plan");
                        send(previous.result());return;
                    }
                    if(operation.plan!=null && !Wire.JSON.readTree(operation.plan).equals(message.required("plan")))
                        throw new IllegalArgumentException("Conflicting plan for frozen operation");
                    operation.plan=plan;
                    if(operation.receipt!=null){send(operation.receipt);return;}
                    if(operation.applying)return;
                    if(System.nanoTime()>operation.deadline){reject(operation,"Operation deadline expired before mutation");return;}
                    operation.applying=true;status.accept("Applying "+id);host.println("Applying "+id);
                    // The admitted native receive callback invokes the document dispatcher directly.
                    document.submit(()->{
                        if(!active || connection!=receivedOn || pending!=operation || System.nanoTime()>operation.deadline)
                            throw new IllegalStateException("Session invalidated before native apply");
                        ObjectNode applied=Wire.parse(editor.apply(operation.snapshot,plan));
                        ObjectNode independent=editor.readContent(operation.snapshot.required("paths"));
                        ObjectNode expected=applied.required("snapshot").deepCopy();expected.set("revision",independent.required("revision"));
                        if(!independent.equals(expected))throw new IllegalStateException("Independent content differs from apply readback");
                        ObjectNode receipt=Wire.object();receipt.put("type","applied");receipt.put("operation",id);
                        receipt.set("snapshot",independent);receipt.set("identities",applied.required("identities"));return receipt.toString();
                    }).whenComplete((result,error)->{
                        if(error==null)operation.receipt=result;
                        else operation.receipt=rejection(operation,error.toString());
                        post(()->{
                            if(pending!=operation)return;
                            if(error!=null)status.accept("Native edit failed: "+error);
                            else status.accept("Observed native result; awaiting Rust preservation verification");
                            send(operation.receipt);
                        });
                    });
                }
                case "complete" -> {
                    keys(message,Set.of("type","operation"));
                    Operation operation=pending;
                    if(operation==null || !operation.id.equals(message.required("operation").asText()))return;
                    if(operation.receipt==null || !Wire.parse(operation.receipt).required("type").asText().equals("applied"))
                        throw new IllegalArgumentException("Completion without native observation");
                    retain(operation);
                    host.println("Verified "+operation.id);pending=null;status.accept("Verified "+operation.id);
                }
                case "error" -> {
                    keys(message,Set.of("type","operation","reason"));
                    Operation operation=pending;
                    if(message.required("operation").isNull()) {
                        disconnect("Service protocol refusal: "+message.required("reason").asText());return;
                    }
                    if(operation!=null && operation.id.equals(message.required("operation").asText())) {
                        // Service errors never convert a native observation into a success.
                        if(operation.receipt!=null)retain(operation);
                        pending=null;
                        status.accept("Service refused: "+message.required("reason").asText());
                    }
                }
                case "unknown" -> {
                    keys(message,Set.of("type","operation"));
                    Operation operation=pending;
                    if(operation!=null && operation.id.equals(message.required("operation").asText())) {
                        if(operation.receipt==null && !operation.applying && operation.request!=null
                                && !Wire.parse(operation.request).required("type").asText().equals("observation"))send(operation.request);
                        else {pending=null;status.accept("Outcome unknown to service; fresh observation is required before another Update");}
                    }
                }
                default -> throw new IllegalArgumentException("Unknown service message");
            }
        }catch(Exception error){disconnect("Protocol failure: "+error);}
    }

    private void execute(ObjectNode message,RemoteConnection receivedOn) throws Exception {
        keys(message,Set.of("type","request","command"));
        String request=message.required("request").asText();
        String prefix=epoch+":command:";
        if(!ready || !request.startsWith(prefix))throw new IllegalArgumentException("Wrong service generation for document request");
        long requestSequence=Long.parseLong(request.substring(prefix.length()));
        if(requestSequence<=0)throw new IllegalArgumentException("Invalid document request identity");
        DocumentRequest cached=documentRequests.get(request);
        if(cached!=null) {
            if(!cached.input.equals(message))throw new IllegalArgumentException("Conflicting document request identity");
            if(cached.receipt!=null)send(cached.receipt);
            return; // An in-progress duplicate shares the original native action.
        }
        if(requestSequence<=commandHighWater){commandError(request,"Expired document request; fresh observation required");return;}
        ObjectNode command=(ObjectNode)message.required("command");
        String action=command.required("action").asText();
        boolean edit=switch(action) {
            case "selection","launcher","capture","mix","exportAudio","info","cues","instrument","readMidi","readClips" -> false;
            case "prepareEdit" -> true;
            default -> throw new IllegalArgumentException("Unknown native document action");
        };
        keys(command,action.equals("capture")?Set.of("action","path","includeInactive"):action.equals("info")?Set.of("action","path","parameters"):action.equals("readClips")?Set.of("action","paths"):(action.equals("selection") || action.equals("launcher") || action.equals("mix") || action.equals("exportAudio") || action.equals("cues") || action.equals("instrument"))?Set.of("action","request"):(action.equals("readMidi"))?Set.of("action"):command.has("paths")?Set.of("action","document","revision","paths"):Set.of("action","document","revision"));
        commandHighWater=requestSequence;
        DocumentRequest entry=new DocumentRequest(message);documentRequests.put(request,entry);
        if(edit && pending!=null){entry.complete(commandErrorReceipt(request,"An edit handshake already owns this session"));documentRequests.trim();send(entry.receipt);return;}
        long commandDeadline=System.nanoTime()+DEADLINE;
        Operation operation=edit?new Operation(adapter+":"+(++sequence)):null;
        if(edit)pending=operation;
        java.util.concurrent.CompletableFuture<String> execution;
        if(action.equals("selection")) {
            execution=document.submit(()->{
                if(!active || receivedOn!=connection || System.nanoTime()>commandDeadline)throw new IllegalStateException("Session changed or deadline expired before selection");
                return editor.selection(command.required("request").toString(),()->active && receivedOn==connection && System.nanoTime()<=commandDeadline);
            }).thenCompose(stage->stage).thenApply(observed->{
                try {ObjectNode result=Wire.object();result.put("type","selectionObservation");result.put("request",request);result.set("result",Wire.parse(observed));return result.toString();}
                catch(java.io.IOException error){throw new java.util.concurrent.CompletionException(error);}
            });
        } else execution=document.submit(()->{
            if(!active || receivedOn!=connection || System.nanoTime()>commandDeadline)throw new IllegalStateException("Session changed or deadline expired before command");
            if(action.equals("mix") || action.equals("exportAudio")) {
                ObjectNode result=Wire.object();result.put("type",action.equals("mix")?"mixObservation":"exportAudioObservation");result.put("request",request);
                result.set("result",Wire.parse(action.equals("mix")?editor.mix(command.required("request").toString()):editor.exportAudio(command.required("request").toString())));Wire.requireMessageCapacity(result);return result.toString();
            }
            if(action.equals("launcher")) {
                ObjectNode result=Wire.object().put("type","launcherObservation").put("request",request);
                result.set("result",editor.launcher((ObjectNode)command.required("request")));
                return result.toString();
            }
            if(action.equals("capture")) {
                ObjectNode result=Wire.object();result.put("type","captureObservation");result.put("request",request);
                result.set("capture",Wire.parse(editor.capture(command.required("path").toString(),command.required("includeInactive").asBoolean())));Wire.requireMessageCapacity(result);return result.toString();
            }
            if(action.equals("info")) {
                ObjectNode result=Wire.object();result.put("type","entityObservation");result.put("request",request);result.set("entity",Wire.parse(editor.info(command.required("path").toString(),command.required("parameters").asBoolean())));return result.toString();
            }
            if(action.equals("cues")) {
                ObjectNode result=Wire.object();result.put("type","cuesObservation");result.put("request",request);
                result.set("result",Wire.parse(editor.cues(command.required("request").toString())));Wire.requireMessageCapacity(result);return result.toString();
            }
            if(action.equals("instrument")) {
                ObjectNode result=Wire.object();result.put("type","instrumentObservation");result.put("request",request);
                result.set("result",Wire.parse(editor.instrument(command.required("request").toString())));Wire.requireMessageCapacity(result);return result.toString();
            }
            if(action.equals("readMidi") || action.equals("readClips")) {
                ObjectNode result=Wire.object();result.put("type","midiObservation");result.put("request",request);
                result.set("midi",Wire.parse(action.equals("readClips")?editor.readClips(command.required("paths").toString()):editor.readMidi()));Wire.requireMessageCapacity(result);return result.toString();
            }
            ObjectNode snapshot=editor.prepareEdit(command.has("paths") ? command.required("paths")
                    : com.fasterxml.jackson.databind.node.NullNode.getInstance());
            if(!snapshot.required("document").equals(command.required("document"))
                    || (!command.required("revision").isNull() && !snapshot.required("revision").equals(command.required("revision"))))
                throw new IllegalArgumentException("Stale document or revision before preparing edit");
            operation.snapshot=snapshot;
            ObjectNode result=Wire.object().put("type","observation").put("request",request).put("operation",operation.id);
            result.set("snapshot",snapshot);
            return result.toString();
        });
        execution.whenComplete((result,error)->post(()->{
            if(error!=null){if(pending==operation)pending=null;entry.complete(commandErrorReceipt(request,error));}
            else {if(edit)operation.request=result;entry.complete(result);}
            documentRequests.trim();
            if(receivedOn!=connection){if(pending==operation)pending=null;return;}
            send(entry.receipt);
        }));
    }
    private void commandError(String request,String reason) {
        send(commandErrorReceipt(request,reason));
    }

    private static String commandErrorReceipt(String request,Throwable error) {
        Throwable cause=error;
        while(cause.getCause()!=null && cause.getCause()!=cause)cause=cause.getCause();
        // NativeRefusal preserves its cause, so inspect the entire chain for the admission marker.
        boolean refused=false;for(Throwable e=error;e!=null;e=e.getCause())if(e.getClass().getName().equals("com.midiworkbench.notes.internal.NativeRefusal")){refused=true;break;}
        ObjectNode result=Wire.object();result.put("type",refused?"commandRefused":"commandError");result.put("request",request);result.put("reason",cause.toString());return result.toString();
    }
    private static String commandErrorReceipt(String request,String reason) {
        ObjectNode result=Wire.object();result.put("type","commandError");result.put("request",request);result.put("reason",reason);return result.toString();
    }

    private void retain(Operation operation) throws Exception {
        var plan=operation.plan==null?null:Wire.parse(operation.plan);
        long planBytes=operation.plan==null?0:operation.plan.getBytes(StandardCharsets.UTF_8).length;
        receipts.put(operation.id,new Receipt(plan,operation.receipt,planBytes+operation.receipt.getBytes(StandardCharsets.UTF_8).length));
        operation.request=null;
    }
    private void reject(Operation operation,String reason){operation.receipt=rejection(operation,reason);send(operation.receipt);}
    private static String rejection(Operation operation,String reason){ObjectNode r=Wire.object();r.put("type","rejected");r.put("operation",operation.id);r.put("reason",reason);return r.toString();}
    private static void keys(ObjectNode object,Set<String> expected){java.util.Set<String> actual=new java.util.HashSet<>();object.fieldNames().forEachRemaining(actual::add);if(!actual.equals(expected))throw new IllegalArgumentException("Invalid message fields");}
    private void post(Runnable completion){if(active && !completions.offer(completion))networkError=new NetworkFailure("Completion queue capacity exceeded",false);}
    private void send(String json) {
        RemoteConnection target=connection;if(!active || target==null)return;
        byte[] payload=json.getBytes(StandardCharsets.UTF_8);
        if(payload.length>Frame.MAX_DECODED_BYTES){disconnect("Complete operation exceeds decoded frame resource limit");return;}
        try {
            writer.execute(()->{
                if(!active || target!=connection)return;
                writingSince.set(System.nanoTime());
                try{byte[] encoded=Frame.encode(payload);target.send(ByteBuffer.allocate(4+encoded.length).putInt(encoded.length).put(encoded).array());}
                catch(Exception failure){networkError=new NetworkFailure("Send failed: "+failure,true);}
                finally{writingSince.set(0);}
            });
        }catch(java.util.concurrent.RejectedExecutionException failure){disconnect("Writer queue capacity exceeded");}
    }
    private void scheduleReconnect(){
        reconnectAt=System.nanoTime()+reconnectDelay;
        reconnectDelay=Math.min(TimeUnit.SECONDS.toNanos(5),reconnectDelay*2);
    }
    private void disconnect(String reason){reconnectAt=Long.MAX_VALUE;ready=false;connecting=false;RemoteConnection old=connection;connection=null;if(old!=null)old.disconnect();connectionAttempt++;connectionError=reason;status.accept(reason);}
    @Override public void close(){active=false;ready=false;document.close(editor).exceptionally(error->{System.err.println("Notes adapter disposal failed: "+error);return null;});RemoteConnection old=connection;connection=null;if(old!=null)old.disconnect();writer.shutdownNow();completions.clear();}
}
