package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.List;

/** Named development setup probe: inspect controller drivers or restart the unique Notes Document Adapter. */
public final class AdapterReloadProbe61 {
    private AdapterReloadProbe61() {}
    private static volatile String refreshStatus;
    public static String run(Object proxy,ObjectNode request)throws Exception {
        HostBuild.loaded(proxy).requireVerified();
        if(!Boolean.TRUE.equals(Reflection.call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong document thread");
        Object app=Class.forName("com.bitwig.flt.app.iOF",true,proxy.getClass().getClassLoader()).getMethod("d3J").invoke(null);
        Object manager=Reflection.call(Reflection.call(app,"kt"),"fVT");
        ObjectNode result=Wire.object();result.put("refreshStatus",refreshStatus);var rows=result.putArray("controllers");Object target=null;
        for(Object controller:(List<?>)Reflection.call(manager,"XlL")) {
            var row=rows.addObject();String id=Reflection.call(controller,"QXt").toString();row.put("driver",id);
            for(String key:List.of("name","vendor","model","is_running","is_running_and_initialized"))
                row.set(key,Wire.JSON.valueToTree(ProjectAdapter61.value(controller,key)));
            if(List.of("08d96063-f7a1-4f66-bd72-6c0eec8e54db","8a9470ba-b9cf-4ad8-a801-e48721e95d42").contains(id)) {
                Object definition=Reflection.call(Reflection.call(controller,"bTk"),"iE");
                row.put("version",Reflection.call(definition,"getVersion").toString());
                row.put("librarySource",((java.io.File)Reflection.call(Reflection.call(Reflection.call(controller,"bTk"),"FSQ"),"SRI")).getCanonicalPath());
                Object available=Reflection.call(Reflection.call(app,"Xtv"),"v33",new Class<?>[]{java.util.UUID.class},java.util.UUID.fromString(id));
                row.put("availableVersion",Reflection.call(Reflection.call(available,"iE"),"getVersion").toString());
            }
            if(request.path("driver").asText().equals(id)) {
                if(target!=null)throw new IllegalArgumentException("Ambiguous native controller driver");target=controller;
            }
        }
        if(request.hasNonNull("driver")) {
            if(!List.of("08d96063-f7a1-4f66-bd72-6c0eec8e54db","8a9470ba-b9cf-4ad8-a801-e48721e95d42").contains(request.required("driver").asText()))throw new IllegalArgumentException("Only the Notes adapter or development harness may be restarted by this setup probe");
            if(target==null)throw new IllegalArgumentException("Native controller driver absent; no restart requested");
            if(request.path("refresh").asBoolean()) {
                if("queued".equals(refreshStatus))throw new IllegalStateException("Native library refresh is already queued");
                Object definitions=Reflection.call(app,"Xtv");
                var field=definitions.getClass().getDeclaredField("TUp");field.setAccessible(true);
                Object library=field.get(definitions);
                Object source=Reflection.call(Reflection.call(target,"bTk"),"FSQ");
                Class<?> pathType=Class.forName("kjs",true,proxy.getClass().getClassLoader());
                java.io.File currentFile=(java.io.File)Reflection.call(source,"SRI");
                java.io.File candidate=new java.io.File(currentFile.getParentFile(),(request.required("driver").asText().equals("8a9470ba-b9cf-4ad8-a801-e48721e95d42")?"NotesHarness-wire":"NotesTransformer-wire")+Wire.VERSION+".bwextension");
                if(!candidate.isFile())throw new IllegalStateException("Exact immutable wire artifact is absent: "+candidate);
                Object nextSource=pathType.getMethod("TUp",java.io.File.class).invoke(null,candidate);
                Object definition=Reflection.call(Reflection.call(target,"bTk"),"iE");
                Class<?> definitionType=Class.forName("com.bitwig.extension.ExtensionDefinition",true,proxy.getClass().getClassLoader());
                Object previous=Reflection.call(library,"TUp",new Class<?>[]{definitionType},definition);
                if(previous==null)throw new IllegalStateException("Current extension library is not registered");
                Reflection.method(library.getClass(),"v33",pathType,java.util.Set.class,List.class);refreshStatus="queued";
                // The native file-change path reloads the registered bHT, then queues
                // replacement of old/new definitions on main. TUp(kjs) only adds libraries.
                java.util.concurrent.CompletableFuture.runAsync(()->{
                    try {Reflection.call(library,"v33",new Class<?>[]{pathType,java.util.Set.class,List.class},nextSource,null,List.of(previous));refreshStatus="native-loader-returned";}
                    catch(Exception error){refreshStatus="failed: "+error;}
                });
                result.put("status","library-refresh-requested");
            } else {Reflection.call(target,"HUL");result.put("status","restart-requested");}
        } else result.put("status","observed");
        return result.toString();
    }
}
