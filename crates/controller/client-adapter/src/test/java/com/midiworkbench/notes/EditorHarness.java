package com.midiworkbench.notes;

import com.bitwig.extension.controller.ControllerExtension;
import com.bitwig.extension.controller.api.Application;
import com.bitwig.extension.controller.api.ControllerHost;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.internal.DocumentAccess;
import com.midiworkbench.notes.internal.HostBuild;
import java.util.concurrent.atomic.AtomicReference;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.net.URL;
import java.net.URLClassLoader;
import java.lang.reflect.InvocationTargetException;
import java.time.Instant;
import java.util.concurrent.ConcurrentLinkedQueue;

/** Named local development probes. SDK calls stay on the controller thread. */
public final class EditorHarness extends ControllerExtension {
    private static final Path DIRECTORY=Path.of(System.getProperty("notes.harness.directory","/tmp/notes-editor-harness"));
    private final ConcurrentLinkedQueue<String> completed=new ConcurrentLinkedQueue<>();
    private DocumentAccess document;
    private Object editor;
    private URLClassLoader code;
    private Object proxy;
    private Application application;
    private String last="";
    private boolean active;
    private boolean busy;
    private String armedDocument;

    EditorHarness(EditorHarnessDefinition definition,ControllerHost host){super(definition,host);}
    @Override public void init(){
        try{
            Files.createDirectories(DIRECTORY);
            proxy=getHost().getDocumentState();
            application=getHost().createApplication();
            HostBuild build=HostBuild.loaded(proxy);
            document=new DocumentAccess(proxy,build); reload();
            // A prior process's command is outcome-unknown, never a new request.
            Path previous=DIRECTORY.resolve("command.json");
            last=Files.exists(previous)?Files.readString(previous):"";
            active=true; tick();
        }catch(Exception error){getHost().println("Harness unavailable: "+error);try{Files.writeString(DIRECTORY.resolve("initialization-error.txt"),error.toString());}catch(Exception retained){error.addSuppressed(retained);}}
    }
    private void tick(){
        if(!active)return;
        try{
            Path input=DIRECTORY.resolve("command.json");
            if(!busy && Files.exists(input)){
                String text=Files.readString(input);
                if(!last.equals(text)){
                    last=text;
                    ObjectNode request=Wire.parse(text);
                    String operation=request.required("operation").asText();
                    String command=request.required("type").asText();
                    if(command.equals("undo") || command.equals("redo")){
                        if(command.equals("undo"))application.undo();else application.redo();
                        finish(operation,command,"dispatched",null);
                    }else if(command.equals("arranger")) {
                        busy=true;
                        document.submit(()->(java.util.concurrent.CompletableFuture<?>)editor.getClass().getMethod("selection",String.class).invoke(editor,request.required("request").toString()))
                            .thenCompose(stage->stage.thenApply(Object::toString)).whenComplete((result,error)->finish(operation,command,result,error));
                    }else{
                        busy=true;
                        AtomicReference<String> faultAudit=new AtomicReference<>();
                        document.submit(()-> {editor.getClass().getMethod("initialize").invoke(editor);return switch(command){
                            case "authority" -> {
                                Class<?> probe=editor.getClass().getClassLoader().loadClass("com.midiworkbench.notes.internal.EditorProbe61");
                                yield (String)probe.getMethod("run",Object.class,Object.class,String.class).invoke(null,proxy,editor,request.toString());
                            }
                            case "reload" -> {reload();editor.getClass().getMethod("initialize").invoke(editor);yield "reloaded; freeze and arm the current disposable document again";}
                            case "construction" -> {
                                Class<?> probe=editor.getClass().getClassLoader().loadClass("com.midiworkbench.notes.internal.ConstructionProbe61");
                                yield (String)probe.getMethod("run",Object.class,String.class).invoke(null,proxy,request.toString());
                            }
                            case "selection" -> invoke("readSelection");
                            case "freeze" -> invoke("freeze");
                            case "arm" -> {
                                ObjectNode state=Wire.parse(invoke("readSelection"));
                                if(!request.required("confirmation").asText().equals("DISPOSABLE FIXTURE")
                                        || !request.required("document").equals(state.required("document")))
                                    throw new IllegalArgumentException("Explicit current disposable document required");
                                armedDocument=state.required("document").asText();yield state.toString();
                            }
                            case "apply", "fail" -> {
                                ObjectNode state=Wire.parse(invoke("readSelection"));
                                if(!state.required("document").asText().equals(armedDocument))throw new IllegalStateException("Arm this disposable fixture first");
                                Class<?> faults=editor.getClass().getClassLoader().loadClass("com.midiworkbench.notes.internal.NativeFaults");
                                if(command.equals("fail"))faults.getMethod("arm",int.class).invoke(null,request.path("after").asInt(1));
                                try {yield (String)editor.getClass().getMethod("apply",String.class).invoke(editor,request.required("plan").toString());}
                                finally {faultAudit.set((String)faults.getMethod("disarm").invoke(null));}
                            }
                            default -> throw new IllegalArgumentException("Unknown named probe: "+command);
                        };}).whenComplete((result,error)->finish(operation,command,result,error,faultAudit.get()));
                    }
                }
            }
        }catch(Exception error){finish("input","parse",null,error);}
        getHost().scheduleTask(this::tick,100);
    }
    private void reload() throws Exception {
        Path built=DIRECTORY.resolve("code.bwextension");
        if(!Files.exists(built)) {
            if(editor!=null)((AutoCloseable)editor).close();
            if(code!=null)code.close();code=null;
            editor=new com.midiworkbench.notes.internal.EditorAdapter61(proxy,HostBuild.loaded(proxy));
            armedDocument=null;return;
        }
        byte[] bytes=Files.readAllBytes(built);
        Path immutable=Files.createTempFile(DIRECTORY,"code-",".jar");
        Files.write(immutable,bytes);
        URLClassLoader next=new URLClassLoader(new URL[]{immutable.toUri().toURL()},getClass().getClassLoader()){
            @Override protected Class<?> findClass(String name) throws ClassNotFoundException {
                if(!name.equals("com.midiworkbench.notes.internal.EditorAdapter61") && !name.equals("com.midiworkbench.notes.internal.AutomationAdapter61"))return super.findClass(name);
                URL resource=findResource(name.replace('.','/')+".class");
                if(resource==null)throw new ClassNotFoundException(name);
                try(var input=resource.openStream()){
                    Class<?> transformer=loadClass("com.midiworkbench.notes.internal.NativeFaultTransformer");
                    byte[] transformed=(byte[])transformer.getMethod("instrument",byte[].class,ClassLoader.class).invoke(null,input.readAllBytes(),this);
                    return defineClass(name,transformed,0,transformed.length);
                }catch(Exception error){throw new ClassNotFoundException("Development instrumentation failed",error);}
            }
            @Override protected Class<?> loadClass(String name,boolean resolve) throws ClassNotFoundException {
                synchronized(getClassLoadingLock(name)){
                    Class<?> found=findLoadedClass(name);
                    if(found==null && name.startsWith("com.midiworkbench.notes.")){
                        found=findClass(name);
                    }
                    if(found==null)found=super.loadClass(name,false);
                    if(resolve)resolveClass(found);return found;
                }
            }
        };
        try{
            Class<?> buildType=next.loadClass("com.midiworkbench.notes.internal.HostBuild");
            Object build=buildType.getConstructor(String.class).newInstance(HostBuild.VERIFIED_61);
            Object loaded=next.loadClass("com.midiworkbench.notes.internal.EditorAdapter61").getConstructor(Object.class,buildType).newInstance(proxy,build);
            if(editor!=null)((AutoCloseable)editor).close();
            if(code!=null)code.close();code=next;editor=loaded;armedDocument=null;
        }catch(Exception error){next.close();Files.deleteIfExists(immutable);throw error;}
    }
    private String invoke(String method) throws Exception {
        try{return (String)editor.getClass().getMethod(method).invoke(editor);}
        catch(InvocationTargetException error){if(error.getCause() instanceof Exception cause)throw cause;throw error;}
    }
    private void finish(String operation,String command,String result,Throwable error){finish(operation,command,result,error,null);}
    private void finish(String operation,String command,String result,Throwable error,String faultAudit){
        ObjectNode entry=Wire.object();entry.put("operation",operation);entry.put("type",command);
        entry.put("at",Instant.now().toString());entry.put("hostBuild",HostBuild.VERIFIED_61);
        entry.put("thread",Thread.currentThread().getName());
        try{if(result!=null)entry.set("observed",Wire.JSON.readTree(result));}
        catch(Exception ignored){entry.put("observed",result);}
        try{if(faultAudit!=null)entry.set("faultAudit",Wire.JSON.readTree(faultAudit));}
        catch(Exception invalid){throw new IllegalStateException("Invalid development fault audit",invalid);}
        while(error instanceof InvocationTargetException invocation)error=invocation.getCause();
        if(error!=null){var trace=new java.io.StringWriter();error.printStackTrace(new java.io.PrintWriter(trace));entry.put("error",trace.toString());}else entry.putNull("error");
        completed.add(entry.toString());
    }
    @Override public void flush(){
        for(String line;(line=completed.poll())!=null;){
            busy=false;
            try{Files.writeString(DIRECTORY.resolve("results.jsonl"),line+"\n",StandardOpenOption.CREATE,StandardOpenOption.APPEND);}
            catch(Exception error){getHost().println("Harness result write failed: "+error);}
        }
    }
    @Override public void exit(){active=false;if(document!=null && editor!=null)document.close((AutoCloseable)editor).exceptionally(error->{System.err.println("Harness disposal failed: "+error);return null;});completed.clear();try{if(code!=null)code.close();}catch(Exception ignored){}}
}
