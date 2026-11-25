package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;

/** Arranger-only titled events, traced through ApiArrangementCueMarkerTimeline. */
final class ProjectCues61 {
    String run(Object proxy,EditorAdapter61 editor,ObjectNode request)throws Exception {
        String action=request.required("action").asText();
        if(action.equals("read")){ObjectNode result=Wire.object();result.putNull("before");result.set("observation",read(proxy,editor));result.putNull("error");return result.toString();}
        ObjectNode before;
        Prepared prepared;
        try {
            if(!action.equals("create"))throw new IllegalArgumentException("Unknown cue action");
            request.required("operation");
            before=read(proxy,editor);
            if(!request.required("document").equals(before.required("document")) || !request.required("revision").equals(before.required("revision")))throw new IllegalStateException("Stale marker scope; no mutation");
            prepared=prepare(proxy,request.required("cues"));
        }catch(Exception failure){throw new NativeRefusal(failure);}
        ObjectNode result=Wire.object();result.set("before",before);result.set("observation",before.deepCopy());result.putNull("error");
        try {apply(prepared);result.set("observation",read(proxy,editor));}
        catch(Exception error){result.put("error",error.toString());try{result.set("observation",read(proxy,editor));}catch(Exception readFailure){error.addSuppressed(readFailure);}}
        return result.toString();
    }
    private ObjectNode read(Object proxy,EditorAdapter61 editor)throws Exception {
        if(!Boolean.TRUE.equals(Reflection.call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong cue owner thread");
        Object document=Reflection.call(proxy,"getDocument");if(document==null)throw new IllegalStateException("No current project");
        Object timeline=ProjectAdapter61.value(ProjectAdapter61.value(document,"transport"),"cue_marker_timeline");
        for(String field:List.of("edit_events","obliterated_events"))if(!((List<?>)ProjectAdapter61.value(timeline,field)).isEmpty())throw new IllegalStateException("Pending native cue edits");
        List<?> events=(List<?>)Reflection.call(timeline,"v33");if(events.size()>4096)throw new IllegalStateException("Cue inventory exceeds bound");
        ObjectNode result=Wire.object();result.put("document",editor.currentProjectGeneration());result.put("projectUuid",ProjectAdapter61.value(document,"UUID").toString());
        var markers=result.putArray("markers");
        for(Object event:events) {
            if(!event.getClass().getName().equals("Wnp"))throw new IllegalStateException("Unsupported native cue event");
            ObjectNode marker=markers.addObject();marker.put("id",ProjectAdapter61.value(event,"managed_object_ID").toString());
            marker.put("name",Reflection.call(event,"v33").toString());marker.put("time",((Number)ProjectAdapter61.value(event,"time")).doubleValue());
        }
        result.put("revision",ObservedScope.revision(result));Wire.requireMessageCapacity(result);return result;
    }
    record Prepared(Object document,Object timeline,Class<?> eventType,List<Object> events) {}

    static void validate(JsonNode request) {
        if(!request.isObject() || request.size()!=1 || !request.has("markers"))throw new IllegalArgumentException("Expected markers only");
        JsonNode markers=request.required("markers");
        if(!markers.isArray() || markers.isEmpty() || markers.size()>128)throw new IllegalArgumentException("Require 1–128 markers");
        var keys=new HashSet<String>();var times=new HashSet<Double>();
        for(JsonNode marker:markers) {
            if(!marker.isObject() || marker.size()!=2 || !marker.path("name").isTextual() || !marker.path("time").isNumber())throw new IllegalArgumentException("Expected marker name/time");
            String name=marker.required("name").asText();double time=marker.required("time").doubleValue();
            if(!name.matches("PC[0-9]{2,4} · \\S(?:.*\\S)?") || name.codePoints().anyMatch(Character::isISOControl)
                || name.getBytes(StandardCharsets.UTF_8).length>64 || !keys.add(key(name))
                || !Double.isFinite(time) || Double.doubleToRawLongBits(time)<0 || time>Long.MAX_VALUE/1048576.0
                || time!=Math.round(time*1048576.0)/1048576.0 || !times.add(time))throw new IllegalArgumentException("Invalid/duplicate cue key or native beat");
        }
    }
    private static String key(String name) {return name.substring(0,name.indexOf(" · "));}
    static Prepared prepare(Object proxy,JsonNode request) throws Exception {
        validate(request);HostBuild.loaded(proxy).requireVerified();
        if(!Boolean.TRUE.equals(Reflection.call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong cue owner thread");
        Object document=Reflection.call(proxy,"getDocument");
        if(document==null || !((Enum<?>)Reflection.call(document,"T1L")).name().equals("CONNECTED"))throw new IllegalStateException("Bitwig must have an active project");
        Object transport=ProjectAdapter61.value(document,"transport");
        for(String field:List.of("is_playing","is_playing_on_engine","record","write_automation"))
            if(Boolean.TRUE.equals(ProjectAdapter61.value(transport,field)))throw new IllegalStateException("Cue creation requires stopped nonrecording transport");
        if(Boolean.TRUE.equals(ProjectAdapter61.value(document,"is_being_saved")) || Reflection.call(document,"jst")!=null)throw new IllegalStateException("Save/render in flight");
        Object timeline=ProjectAdapter61.value(transport,"cue_marker_timeline");
        // API wrapper getEvents uses this complete committed list (not a bank or cursor).
        List<?> existing=(List<?>)Reflection.call(timeline,"v33");
        if(existing.size()+request.required("markers").size()>4096)throw new IllegalStateException("Cue inventory exceeds bound");
        for(String field:List.of("edit_events","obliterated_events"))
            if(!((List<?>)ProjectAdapter61.value(timeline,field)).isEmpty())throw new IllegalStateException("Pending cue edits");
        var loader=proxy.getClass().getClassLoader();Class<?> event=Class.forName("Wnp",true,loader),base=Class.forName("oHd",true,loader);
        Reflection.method(timeline.getClass(),"jaQ",base);EditorAdapter61.requireIndependentUndo(document);
        var created=new ArrayList<Object>();
        for(JsonNode marker:request.required("markers")) {
            String name=marker.required("name").asText(),key=key(name);
            for(Object old:existing) {
                if(!event.isInstance(old))throw new IllegalStateException("Unknown cue event kind");
                String title=(String)Reflection.call(old,"v33");
                if(title.equals(name) || title.equals(key) || title.startsWith(key+" · "))throw new IllegalArgumentException("Cue key collision: "+key);
            }
            Object point=Reflection.method(event,"EwU",double.class).invoke(null,marker.required("time").doubleValue());
            Reflection.call(point,"v33",new Class<?>[]{String.class},name);created.add(point);
        }
        return new Prepared(document,timeline,base,List.copyOf(created));
    }
    static void apply(Prepared p) throws Exception {
        Reflection.call(p.document(),"v33",new Class<?>[]{String.class,boolean.class},"Create performance cue markers",false);
        try {
            for(Object point:p.events())Reflection.call(p.timeline(),"jaQ",new Class<?>[]{p.eventType()},point);
            Reflection.call(Reflection.call(Reflection.call(Reflection.call(Reflection.call(p.document(),"cuK"),"TcB"),"v33"),"v33"),"LHs");
        } finally {Reflection.call(p.document(),"v33",new Class<?>[]{boolean.class},false);}
    }
}
