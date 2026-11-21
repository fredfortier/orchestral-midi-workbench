package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.List;
import java.util.Set;

/** Bitwig's folder-like hierarchy. No native property graphs or musical payloads. */
final class Entity61 {
    private final Object document;
    Entity61(Object document){this.document=document;}
    private static Object call(Object o,String method)throws Exception{return Reflection.call(o,method);}
    private static Object value(Object o,String field)throws Exception{return ProjectAdapter61.value(o,field);}
    private static String title(Object o)throws Exception{return value(o,"displayed_title").toString();}
    private static ObjectNode childPath(JsonNode parent,String kind,String name) {
        ObjectNode path=parent.deepCopy();path.withArray("elements").addObject().put("kind",kind).put("name",name);return path;
    }
    private static ObjectNode entity(JsonNode path,String kind,String name,Object owner)throws Exception {
        if(!path.required("elements").isEmpty())((ObjectNode)path.required("elements").get(path.required("elements").size()-1)).put("nativeId",value(owner,"managed_object_ID").toString());
        ObjectNode result=Wire.object();result.set("path",path);result.put("kind",kind);
        ObjectNode metadata=result.putObject("metadata");metadata.put("name",name);metadata.put("id",value(owner,"managed_object_ID").toString());result.putArray("children");
        switch(kind) {
            case "clip" -> {
                metadata.put("start",((Number)call(owner,"T1L")).doubleValue());metadata.put("duration",((Number)call(owner,"GaO")).doubleValue());
                metadata.put("sourceStart",((Number)call(owner,"dD_")).doubleValue());metadata.put("sourceEnd",((Number)call(owner,"Fgm")).doubleValue());metadata.put("loopEnabled",(boolean)call(owner,"YeF"));
                Object loop=call(owner,"fvB");metadata.put("loopStart",((Number)value(loop,"time")).doubleValue());metadata.put("loopLength",((Number)value(loop,"duration")).doubleValue());
            }
            case "layerChain","choice" -> {metadata.put("channel",((Number)value(owner,"filter_note_channel")).intValue());metadata.put("outputChannel",((Number)value(owner,"override_note_channel")).intValue());}
            case "plugin","device","layer","selector" -> {
                SlotRouting61.read(owner,metadata);
                if(kind.equals("plugin")){Object contents=value(owner,"device_contents");metadata.put("pluginId","vst3-"+value(contents,"vst_device_id"));metadata.put("version",value(contents,"device_version").toString());}
            }
            default -> { }
        }
        return result;
    }
    String read(JsonNode requested)throws Exception {return read(requested,false);}
    String read(JsonNode requested,boolean parameters)throws Exception {
        ObjectNode path;
        Object owner;
        String kind;
        if(requested.isNull() || requested.required("elements").isEmpty()) {
            if(!requested.isNull())ProjectAddress61.resolveNative(requested,document,document.getClass().getClassLoader());
            path=Wire.object();path.put("project",value(document,"name").toString());path.putArray("elements");
            owner=document;kind="project";
        } else {
            owner=ProjectAddress61.resolveNative(requested,document,document.getClass().getClassLoader());
            path=ProjectAddress61.qualified(requested,document);kind=path.required("elements").get(path.required("elements").size()-1).required("kind").asText();
        }
        String name=kind.equals("project")?value(document,"name").toString():path.required("elements").get(path.required("elements").size()-1).required("name").asText();
        ObjectNode node=entity(path,kind,name,owner);
        if(kind.equals("project")) {
            String file=(String)call(document,"jDA");if(file!=null && !file.isBlank())node.withObject("metadata").put("file",file);
            tracks(call(document,"peG"),node);
        } else if(kind.equals("track") || kind.equals("group"))track(owner,node,kind.equals("group"));
        else if(kind.equals("layerChain") || kind.equals("choice"))devices(owner,node);
        else if(Set.of("device","plugin","layer","selector").contains(kind)) {
            deviceChildren(owner,node,kind);
            if(parameters)node.withObject("metadata").setAll(DeviceParameters61.read(owner,node.required("path")));
        } else if(parameters)throw new IllegalArgumentException("Parameter discovery requires a device path");
        return node.toString();
    }
    private void tracks(Object parent,ObjectNode root)throws Exception {
        java.util.LinkedHashSet<Object> tracks=new java.util.LinkedHashSet<>((List<?>)call(parent,"IFc"));tracks.addAll((List<?>)call(parent,"UQt"));
        for(Object track:tracks) {
            boolean group=track.getClass().getName().equals("com.bitwig.flt.document.core.master.stI");String kind=group?"group":"track";
            ObjectNode path=childPath(root.required("path"),kind,title(track));ObjectNode node=entity(path,kind,title(track),track);root.withArray("children").add(node);
            track(track,node,group);
        }
    }
    private void track(Object track,ObjectNode node,boolean group)throws Exception {
        JsonNode path=node.required("path");
        Object channel=group?call(track,"DQP"):track;
        node.withObject("metadata").put("active",(boolean)value(channel,"active"));
        Object lane=value(channel,"main_note_clip_lane");
        if(lane!=null)try(var stream=(java.util.stream.Stream<?>)Reflection.call(lane,"v33",new Class<?>[]{double.class,double.class},Double.NEGATIVE_INFINITY,Double.POSITIVE_INFINITY)) {
            var clips=stream.iterator();while(clips.hasNext()) {
                Object clip=clips.next();if(!clip.getClass().getName().equals("UMj"))continue;
                ObjectNode c=entity(childPath(path,"clip",title(clip)),"clip",title(clip),clip);node.withArray("children").add(c);
            }
        }
        int index=0;
        for(Object automation:automationLanes(channel)) {
            ObjectNode c=entity(childPath(path,"automationLane",laneName(automation)),"automationLane",laneName(automation),automation);
            c.withObject("metadata").put("index",index++);node.withArray("children").add(c);
        }
        devices(channel,node);if(group)tracks(track,node);
    }
    static List<?> automationLanes(Object track)throws Exception {
        Object lanes=value(track,"automation_lanes");
        Class<?> preview=Class.forName("owf",true,track.getClass().getClassLoader());
        return ((List<?>)Reflection.call(lanes,"XrN",new Class<?>[]{String.class},new Object[]{null})).stream().filter(l->!preview.isInstance(l)).toList();
    }
    static String laneName(Object lane)throws Exception {
        Object atom=value(lane,"value_atom");if(atom==null)atom=call(value(lane,"atom_reference"),"qKA");
        if(atom==null)throw new IllegalStateException("Unresolved automation lane target");
        return value(atom,"name")+" automation";
    }
    private void devices(Object owner,ObjectNode parent)throws Exception {
        Object chain=call(owner,owner.getClass().getName().equals("com.bitwig.flt.document.core.master.device.kdt")?"PPp":"Vdp");
        for(Object device:(List<?>)call(chain,"ATl")) {
            String name=value(device,"user_device_name").toString();if(name.isEmpty())name=value(device,"device_name").toString();
            String builtin=call(device,"v33").toString();Object contents=value(device,"device_contents");
            String kind=builtin.equals("5024be2e-65d6-4d40-bbfe-8b2ea993c445")?"layer":builtin.equals("9588fbcf-721a-438b-8555-97e4231f7d2c")?"selector":contents!=null && Class.forName("ych",true,contents.getClass().getClassLoader()).isInstance(contents)?"plugin":"device";
            ObjectNode path=childPath(parent.required("path"),kind,name);
            if(kind.equals("plugin")) {ObjectNode e=(ObjectNode)path.withArray("elements").get(path.withArray("elements").size()-1);e.put("id","vst3-"+value(contents,"vst_device_id"));e.put("version",value(contents,"device_version").toString());}
            ObjectNode node=entity(path,kind,name,device);parent.withArray("children").add(node);
            deviceChildren(device,node,kind);
        }
    }
    private void deviceChildren(Object device,ObjectNode node,String kind)throws Exception {
            JsonNode path=node.required("path");
            Object contents=value(device,"device_contents");
            if(kind.equals("layer") || kind.equals("selector")) {
                Object container=Reflection.call(contents,"v33",new Class<?>[]{String.class},kind.equals("layer")?"CHAIN_LIST":"CHAIN_SELECTOR");
                for(Object layer:(List<?>)value(container,"nested_device_chains")) {
                    String layerKind=kind.equals("layer")?"layerChain":"choice";ObjectNode child=entity(childPath(path,layerKind,title(layer)),layerKind,title(layer),layer);node.withArray("children").add(child);
                    devices(layer,child);
                }
            }
    }
}
