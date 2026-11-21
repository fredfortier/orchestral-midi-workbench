package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.net.URLClassLoader;
import java.util.function.BiConsumer;
import java.util.function.DoubleConsumer;

/** The same Zv/TQb export task instantiated by DXq's native Export Audio action. */
final class ExportAudio61 {
    private static final class Job {
        Object document,task;final ObjectNode receipt;Boolean success;String error;double progress;boolean settled;
        Job(Object document,ObjectNode receipt){this.document=document;this.receipt=receipt;}
    }
    private URLClassLoader taskLoader;
    void close()throws Exception {jobs.clear();if(taskLoader!=null)taskLoader.close();taskLoader=null;}
    private final Map<String,Job> jobs=new LinkedHashMap<>();
    private static Object call(Object owner,String method)throws Exception{return Reflection.call(owner,method);}
    private static Object value(Object owner,String key)throws Exception{return ProjectAdapter61.value(owner,key);}
    private static Class<?> type(ClassLoader loader,String name)throws Exception{return Class.forName(name,true,loader);}
    private static Object field(Object owner,String name)throws Exception {
        for(Class<?> c=owner.getClass();c!=null;c=c.getSuperclass())try {var f=c.getDeclaredField(name);f.setAccessible(true);return f.get(owner);}catch(NoSuchFieldException ignored) {}
        throw new NoSuchFieldException(name);
    }
    private static void set(Object owner,String key,Object v)throws Exception {Reflection.call(Reflection.call(owner,"Nln",new Class<?>[]{String.class},key),"setValue",new Class<?>[]{Object.class},v);}
    boolean contains(String operation){return jobs.containsKey(operation);}
    void requireIdle()throws Exception {
        for(Job job:jobs.values())if(!job.settled)throw new IllegalStateException("Export remains pending; query its operation status before mutation");
    }
    String run(Object proxy,EditorAdapter61 editor,Mix61 mix,ObjectNode request)throws Exception {
        String operation=request.required("operation").asText();
        if(!operation.matches("[A-Za-z0-9:/-]{1,256}"))throw new IllegalArgumentException("Invalid export operation");
        String action=request.required("action").asText();
        if(action.equals("status"))return status(proxy,editor,operation);
        if(!action.equals("start"))throw new IllegalArgumentException("Unknown export action");
        Job previous=jobs.get(operation);
        if(previous!=null)throw new IllegalArgumentException("Export operation already exists; query its original status");
        requireIdle();Object document=Mix61.document(proxy);ClassLoader loader=document.getClass().getClassLoader();
        JsonNode options=request.required("options");
        if(!Mix61.sameScope(mix.read(proxy,editor),options.required("expected")))throw new IllegalArgumentException("Stale mixer scope before export; no job created");
        Mix61.requireIdleEngine(document);
        if(!((Enum<?>)call(document,"T1L")).name().equals("CONNECTED") || call(document,"Met")!=null || !Boolean.TRUE.equals(call(document,"GsP")) || call(document,"jst")!=null)throw new IllegalStateException("Native export requires healthy idle connected engine");
        if(((Number)call(call(document,"voC"),"HUL")).intValue()!=0)throw new IllegalStateException("Native project has missing files");
        for(Object container:(List<?>)call(call(document,"YR"),"UMc"))for(Object plugin:(List<?>)call(container,"AxB"))
            if(call(plugin,"heu")==type(loader,"aPz").getField("v33").get(null))throw new IllegalStateException("Native project has missing plugins; no export confirmation dialog requested");
        Object queue=call(document,"R3h");if(field(queue,"FHv")!=null || !((List<?>)field(queue,"jaQ")).isEmpty())throw new IllegalStateException("Native render queue not drained");
        double start=options.required("start").doubleValue(),end=options.required("end").doubleValue();
        double seconds=((Number)Reflection.call(document,"tGW",new Class<?>[]{double.class},end)).doubleValue()-((Number)Reflection.call(document,"tGW",new Class<?>[]{double.class},start)).doubleValue();
        if(!nativeBeat(start) || !nativeBeat(end) || end<=start || !Double.isFinite(seconds) || seconds<=0 || seconds>3600)throw new IllegalArgumentException("Export range must be positive native beats and at most one hour");
        int rate=options.required("sampleRate").intValue();if(!options.required("sampleRate").isIntegralNumber() || !List.of(44100,48000,88200,96000,192000).contains(rate))throw new IllegalArgumentException("Unsupported sample rate");
        String depth=options.required("bitDepth").asText();if(!List.of("INT16","INT24","FLOAT32").contains(depth))throw new IllegalArgumentException("Unsupported WAV bit depth");
        for(String key:List.of("dither","preFader","realTime"))if(!options.required(key).isBoolean())throw new IllegalArgumentException("Boolean export option required: "+key);
        if(depth.equals("FLOAT32") && options.required("dither").asBoolean())throw new IllegalArgumentException("Float analysis exports require dither off");
        JsonNode paths=options.required("tracks");if(!paths.isArray() || paths.isEmpty() || paths.size()>256)throw new IllegalArgumentException("Require 1–256 exact export owners");
        List<Object> targets=new ArrayList<>();HashSet<String> seen=new HashSet<>();
        for(JsonNode path:paths) {
            if(!seen.add(path.toString()))throw new IllegalArgumentException("Duplicate export owner");
            boolean observed=false;for(JsonNode track:options.required("expected").required("tracks"))if(path.equals(track.required("path")))observed=true;
            if(!observed)throw new IllegalArgumentException("Export owner absent from expected scope");targets.add(Mix61.resolve(document,path));
        }
        Path directory=Path.of(options.required("directory").asText());if(!directory.isAbsolute() || Files.exists(directory) || !Files.isDirectory(directory.getParent()))throw new IllegalArgumentException("Require a fresh absolute export directory under an existing parent");
        if(jobs.size()>=256)throw new IllegalStateException("Export operation capacity reached");
        String[] names=new String[targets.size()];ObjectNode receipt=Wire.object();receipt.put("operation",operation);receipt.put("document",editor.currentProjectGeneration());receipt.put("hostBuild",HostBuild.VERIFIED_61);receipt.put("expectedSeconds",seconds);receipt.set("options",options.deepCopy());receipt.withObject("options").remove("expected");receipt.put("sourceNativeRevision",options.required("expected").required("nativeRevision").asText());
        var files=receipt.putArray("files");
        for(int i=0;i<targets.size();i++) {
            JsonNode path=paths.get(i);String name=path.required("elements").isEmpty()?"Master":path.required("elements").get(path.required("elements").size()-1).required("name").asText();
            name=name.replaceAll("[^\\p{L}\\p{N}._-]","_");if(name.length()>80)name=name.substring(0,80);
            names[i]=directory.resolve(String.format(java.util.Locale.ROOT,"%03d-%s",i+1,name)).toString();
            ObjectNode f=files.addObject();f.set("track",path.deepCopy());f.put("path",names[i]+".wav");
        }
        // Fresh settings are task-owned; no persistent project/export preferences change.
        Object settings=type(loader,"TQb").getMethod("Iz2").invoke(null),encoding=call(settings,"SRI");
        set(settings,"bounce_point",type(loader,"yLG").getMethod("valueOf",String.class).invoke(null,options.required("preFader").asBoolean()?"PRE_FADER":"POST_FADER"));
        set(settings,"user_real_time",options.required("realTime").asBoolean());
        set(encoding,"render_wav_bit_depth",type(loader,"jnu").getMethod("valueOf",String.class).invoke(null,depth));
        set(encoding,"render_dither",options.required("dither").asBoolean());set(encoding,"encode_dither",options.required("dither").asBoolean());set(encoding,"resampling_frequency",rate);
        for(String key:List.of("enable_mp3","enable_flac","enable_vorbis","enable_opus"))set(encoding,key,false);set(encoding,"enable_wav",true);
        Files.createDirectory(directory);
        if(taskLoader==null)taskLoader=new URLClassLoader(new java.net.URL[]{ExportAudio61.class.getProtectionDomain().getCodeSource().getLocation()},proxy.getClass().getClassLoader());
        Job job=new Job(document,receipt);
        BiConsumer<Boolean,String> completed=(success,error)->{job.success=success;job.error=error;if(success)job.progress=1.0;};
        DoubleConsumer progress=amount->job.progress=amount;
        Object task=Class.forName("ExportTask61",true,taskLoader).getConstructor(type(loader,"com.bitwig.flt.document.core.master.DXq"),List.class,String[].class,double.class,double.class,BiConsumer.class,DoubleConsumer.class).newInstance(document,targets,names,start,end,completed,progress);
        job.task=task;jobs.put(operation,job);
        try {Reflection.call(task,"TUp",new Class<?>[]{type(loader,"TQb")},settings);}
        catch(Exception error) {
            receipt.put("dispatchError",error.toString());
            // A synchronous exception is not proof that nothing started.
            // Only a fresh, drained native engine proves a pre-dispatch failure.
            try {Mix61.requireIdleEngine(document);job.success=false;job.error=error.toString();}
            catch(Exception uncertain){receipt.put("outcomeUnknown",true);}
        }
        return status(proxy,editor,operation);
    }
    private static boolean nativeBeat(double v){return Double.isFinite(v) && v>=0 && v<Long.MAX_VALUE/1048576.0 && v==Math.round(v*1048576.0)/1048576.0;}
    private String status(Object proxy,EditorAdapter61 editor,String operation)throws Exception {
        Job job=jobs.get(operation);if(job==null)throw new IllegalArgumentException("Unknown export operation; do not replay");
        if(!Boolean.TRUE.equals(call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong export owner thread");
        ObjectNode result=job.receipt.deepCopy();
        if(!job.settled && job.task!=null) {
            Object progress=field(job.task,"jaQ");if(progress!=null)job.progress=((Number)call(progress,"Vdp")).doubleValue();
            if(Boolean.TRUE.equals(job.success))job.progress=1.0;
            if(Boolean.TRUE.equals(call(job.task,"zeN")))result.put("effectiveRealTime",(boolean)value(call(job.task,"FSQ"),"effective_real_time"));
        }
        boolean source=false;
        try {source=editor.currentProjectGeneration().equals(result.required("document").asText());}
        catch(Exception unavailable){result.put("sourceObservationError",unavailable.toString());}
        result.put("sourceMatches",source);result.put("nativeComplete",job.success!=null);result.set("nativeSuccess",Wire.JSON.valueToTree(job.success));
        boolean drained=job.settled;
        if(!job.settled) {
            try {Mix61.requireQueueIdle(job.document);drained=true;}
            catch(Exception busy){drained=false;}
            if(job.success==null && drained && result.has("dispatchError")){job.success=false;job.error=result.required("dispatchError").asText();}
        }
        boolean outputs=true;for(JsonNode file:result.required("files")) {
            Path path=Path.of(file.required("path").asText());boolean exists=Files.isRegularFile(path);long bytes=exists?Files.size(path):0;
            ((ObjectNode)file).put("exists",exists);((ObjectNode)file).put("bytes",bytes);outputs&=exists && bytes>44;
        }
        String state=job.success==null?"pending":!drained?"pending":Boolean.TRUE.equals(job.success)&&outputs?"completed":"failed";
        result.put("status",state);result.put("progress",job.progress);
        if(job.error!=null && !job.error.isEmpty())result.put("error",job.error);
        result.put("scope","Native final success/error callback, drained engine and output existence. Independently verify WAV format, frames, signal, tails and music. Leaf exports exclude ancestor/master processing.");
        if(!state.equals("pending")) {
            job.receipt.setAll(result);job.receipt.withObject("options").remove("expected");
            job.document=null;job.task=null;job.settled=true;
        }
        return result.toString();
    }
}
