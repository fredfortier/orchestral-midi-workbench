package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.List;

/** One document-thread capture; canonical entities and complete clips, never a second note model. */
final class MidiCapture61 {
    private final Object document;
    private final EditorAdapter61 editor;
    MidiCapture61(Object document,EditorAdapter61 editor){this.document=document;this.editor=editor;}
    private static Object call(Object owner,String method)throws Exception{return Reflection.call(owner,method);}
    private static Object value(Object owner,String key)throws Exception{return ProjectAdapter61.value(owner,key);}
    private static double number(Object value){double n=((Number)value).doubleValue();if(!Double.isFinite(n))throw new IllegalStateException("Non-finite timing context");return n;}
    String read(JsonNode path,boolean includeInactive)throws Exception {
        ObjectNode entities=Wire.parse(new Entity61(document).read(path));
        if(!List.of("project","group","track").contains(entities.required("kind").asText()))throw new IllegalArgumentException("Capture requires a project, group or track path");
        ObjectNode result=Wire.object();result.put("version",2);result.set("entities",entities);
        ArrayNode paths=Wire.JSON.createArrayNode(),excluded=result.putArray("excluded");
        // An explicitly requested descendant still inherits inactive ancestor groups.
        boolean ancestors=true;
        if(!path.isNull()) {
            ObjectNode prefix=path.deepCopy();prefix.set("elements",Wire.JSON.createArrayNode());
            for(JsonNode element:path.required("elements")) {
                if(!element.path("kind").asText().equals("group"))break;
                prefix.withArray("elements").add(element);
                Object group=ProjectAddress61.resolveNative(prefix,document,document.getClass().getClassLoader());
                ancestors &= (boolean)value(call(group,"DQP"),"active");
            }
        }
        collect(entities,ancestors,includeInactive,paths,excluded);
        if(paths.isEmpty())throw new IllegalArgumentException("No included content owners in capture scope");
        ObjectNode midi=Wire.parse(editor.readClips(paths.toString()));result.set("midi",midi);
        double end=0;for(JsonNode clip:midi.required("clips"))end=Math.max(end,clip.required("start").asDouble()+clip.required("duration").asDouble());
        for(JsonNode lane:midi.required("automation")) {
            if(lane.required("placement").asText().equals("clip"))end=Math.max(end,lane.required("mapping").required("startBeats").asDouble()+lane.required("mapping").required("durationBeats").asDouble());
            else for(JsonNode point:lane.required("points"))end=Math.max(end,point.required("time").asDouble());
        }
        result.set("globals",globals(end));result.putArray("decisions");
        return result.toString();
    }
    private static void collect(JsonNode node,boolean active,boolean includeInactive,ArrayNode paths,ArrayNode excluded) {
        String kind=node.required("kind").asText();
        if(kind.equals("track") || kind.equals("group"))active &= node.required("metadata").required("active").asBoolean();
        if(!includeInactive && !active){excluded.add(node.required("path"));return;}
        if(kind.equals("track") || kind.equals("group"))paths.add(node.required("path"));
        for(JsonNode child:node.required("children"))collect(child,active,includeInactive,paths,excluded);
    }
    private ArrayNode globals(double end)throws Exception {
        if(end<0 || end*16>100_000)throw new IllegalArgumentException("Capture extent exceeds supported timing range");
        Object transport=call(document,"Nh"),signature=call(transport,"IFc");
        ClassLoader loader=document.getClass().getClassLoader();Class<?> helper=Class.forName("RYK",true,loader);
        ArrayNode result=Wire.JSON.createArrayNode();
        // Native beat/seconds is authoritative and integrates the actual tempo curve.
        // Include exact stored signature boundaries; tempo is sampled independently.
        var beats=new java.util.TreeSet<Double>();beats.add(0.0);beats.add(end);
        for(double beat=0;beat<end;beat+=0.0625)beats.add(beat);
        Object timeline=call(transport,"qYu");
        for(Object point:(List<?>)call(timeline,"v33")) {double beat=number(value(point,"time"));if(beat>=0 && beat<=end)beats.add(beat);}
        if(beats.size()>100_000)throw new IllegalArgumentException("Global timing context exceeds 100000 samples");
        for(double beat:beats) {
            Object point=Reflection.call(signature,"j_",new Class<?>[]{double.class},beat);
            int packed=((Number)call(point==null?signature:point,point==null?"tGW":"v33")).intValue();
            ObjectNode event=result.addObject();event.put("beat",beat);
            event.put("seconds",number(Reflection.call(transport,"EwU",new Class<?>[]{double.class},beat)));
            event.put("numerator",((Number)Reflection.method(helper,"tGW",int.class).invoke(null,packed)).intValue());
            event.put("denominator",((Number)Reflection.method(helper,"ncc",int.class).invoke(null,packed)).intValue());
        }
        return result;
    }
}
