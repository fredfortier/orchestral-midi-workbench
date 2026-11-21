package com.midiworkbench.notes;

import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.PosixFilePermissions;
import java.util.Set;
import java.util.UUID;

/** Owner-private lifecycle commands only. Session recovery belongs to EditorSession. */
final class ConnectionControl implements AutoCloseable {
    interface Target {
        ObjectNode connectionStatus();
        void requestConnect();
        void requestDisconnect();
    }
    private final Target target;
    private final Path directory;
    private final String adapter;
    private final long hostPid;
    private final String hostStartTicks;
    private String pending;
    private String previous;
    private String controlError;
    private int commands;
    private final java.util.Map<String,ObjectNode> requests=new java.util.HashMap<>();
    private boolean closed;
    ConnectionControl(Target target,Path root) throws Exception {
        this.target=target;adapter=target.connectionStatus().required("adapter").asText();
        hostPid=ProcessHandle.current().pid();hostStartTicks=startTicks(hostPid);
        UUID.fromString(adapter);privateDirectory(root);directory=root.resolve(adapter);
        Files.createDirectory(directory,PosixFilePermissions.asFileAttribute(PosixFilePermissions.fromString("rwx------")));
        privateDirectory(directory.resolve("requests"));publishStatus();
    }
    static Path root() {
        String state=System.getenv("XDG_STATE_HOME");
        Path base=state==null || state.isBlank()?Path.of(System.getProperty("user.home"),".local","state"):Path.of(state);
        if(!base.isAbsolute())throw new IllegalArgumentException("XDG_STATE_HOME must be absolute");
        return base.resolve("midi-workbench").resolve("connections");
    }
    void tick() {
        if(closed)return;
        if(controlError==null)try {
            settle();Path inbox=directory.resolve("command.json");
            if(Files.exists(inbox,LinkOption.NOFOLLOW_LINKS)) {
                privateFile(inbox);
                byte[] bytes;
                try(var input=Files.newInputStream(inbox)){bytes=input.readNBytes(4097);}
                if(bytes.length>4096)throw new IOException("Lifecycle command exceeds4096bytes; retained inbox requires review");
                var request=Wire.parse(new String(bytes,java.nio.charset.StandardCharsets.UTF_8));
                Set<String> keys=new java.util.HashSet<>();request.fieldNames().forEachRemaining(keys::add);
                if(!keys.equals(Set.of("schemaVersion","adapter","operation","action")) || request.required("schemaVersion").asInt()!=1 || !request.required("adapter").asText().equals(adapter))throw new IOException("Invalid/stale lifecycle command; retained inbox requires review");
                String operation=request.required("operation").asText(),action=request.required("action").asText();
                if(!operation.matches("[A-Za-z0-9_.:-]{1,128}") || !Set.of("connect","disconnect").contains(action))throw new IOException("Invalid lifecycle operation/action; no attempt dispatched");
                if(commands>=256)throw new IOException("Lifecycle receipt budget exhausted; no attempt dispatched");
                if(Files.exists(directory.resolve("requests").resolve(operation+".result.json"),LinkOption.NOFOLLOW_LINKS))throw new IOException("Lifecycle result already retained; no attempt dispatched");
                Path retained=directory.resolve("requests").resolve(operation+".request.json");
                Files.createLink(retained,inbox);Files.delete(inbox);commands++;
                try(var input=Files.newInputStream(retained)){if(!java.util.Arrays.equals(bytes,input.readNBytes(4097)))throw new IOException("Lifecycle intent changed before dispatch");}
                requests.put(operation,request.deepCopy());
                try {
                    if(action.equals("connect")) {
                        if(pending!=null)throw new IllegalStateException("A connection attempt is pending; request disconnect to cancel");
                        target.requestConnect();pending=operation;
                    } else {
                        if(pending!=null){finish(pending,"failed","Explicitly cancelled by disconnect "+operation);pending=null;}
                        target.requestDisconnect();finish(operation,"passed",null);
                    }
                }catch(Exception error){finish(operation,"failed",error.toString());}
                settle();
            }
        }catch(Exception error){controlError=error.toString();}
        try{publishStatus();}catch(Exception error){System.err.println("Connection lifecycle status publication failed: "+error);}
    }
    private void settle() throws Exception {
        if(pending==null)return;
        var state=target.connectionStatus();String phase=state.required("state").asText();
        if(phase.equals("ready")){finish(pending,"passed",null);pending=null;}
        else if(!phase.equals("connecting")){finish(pending,"failed",state.path("error").asText("Connection attempt stopped"));pending=null;}
    }
    private void finish(String operation,String status,String error) throws Exception {
        var result=target.connectionStatus();result.put("schemaVersion",1);result.put("operation",operation);result.put("status",status);result.put("operationError",error);result.set("request",requests.get(operation));
        publish(directory.resolve("requests").resolve(operation+".result.json"),result.toString(),false);
    }
    private void publishStatus() throws Exception {
        var state=target.connectionStatus();state.put("schemaVersion",1);state.put("hostPid",hostPid);state.put("hostStartTicks",hostStartTicks);state.put("closed",closed);state.put("pendingOperation",pending);state.put("controlError",controlError);state.put("retainedCommands",commands);
        String json=state.toString();if(!json.equals(previous)){publish(directory.resolve("status.json"),json,true);previous=json;}
    }
    static String startTicks(long pid) throws IOException {
        String stat=Files.readString(Path.of("/proc",Long.toString(pid),"stat"));
        return stat.substring(stat.lastIndexOf(')')+2).split(" ")[19];
    }
    static void privateDirectory(Path path) throws IOException {
        if(!Files.exists(path,LinkOption.NOFOLLOW_LINKS))Files.createDirectories(path,PosixFilePermissions.asFileAttribute(PosixFilePermissions.fromString("rwx------")));
        if(Files.isSymbolicLink(path) || !Files.isDirectory(path,LinkOption.NOFOLLOW_LINKS) || !Files.getPosixFilePermissions(path).equals(PosixFilePermissions.fromString("rwx------")))throw new IOException("Lifecycle directory must be private0700 and not a symlink: "+path);
    }
    static void privateFile(Path path) throws IOException {
        if(Files.isSymbolicLink(path) || !Files.isRegularFile(path,LinkOption.NOFOLLOW_LINKS) || !Files.getPosixFilePermissions(path).equals(PosixFilePermissions.fromString("rw-------")))throw new IOException("Lifecycle file must be private0600 and regular: "+path);
    }
    private static void publish(Path path,String json,boolean replace) throws IOException {
        byte[] bytes=json.getBytes(java.nio.charset.StandardCharsets.UTF_8);if(bytes.length>4096)throw new IOException("Lifecycle status exceeds4096bytes");
        Path temporary=Files.createTempFile(path.getParent(),".lifecycle-",".tmp",PosixFilePermissions.asFileAttribute(PosixFilePermissions.fromString("rw-------")));
        try {Files.write(temporary,bytes);if(replace)Files.move(temporary,path,StandardCopyOption.ATOMIC_MOVE,StandardCopyOption.REPLACE_EXISTING);else Files.createLink(path,temporary);}
        finally{Files.deleteIfExists(temporary);}
    }
    @Override public void close(){closed=true;try{if(pending!=null){finish(pending,"failed","Controller exited; no replay");pending=null;}publishStatus();}catch(Exception error){System.err.println("Lifecycle exit publication failed: "+error);}}
}
