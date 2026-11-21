package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;

/** Stopped-document mixer facts. Host values never leave the document thread. */
final class Mix61 {
    private final ObservedScope authority=new ObservedScope();
    boolean dispatched;
    static boolean sameScope(JsonNode a,JsonNode b){ObjectNode aa=a.deepCopy(),bb=b.deepCopy();aa.remove("captureTime");bb.remove("captureTime");return aa.equals(bb);}
    private static Object value(Object owner,String key)throws Exception{return ProjectAdapter61.value(owner,key);}
    private static Object call(Object owner,String method)throws Exception{return Reflection.call(owner,method);}
    static Object optional(Object owner,String key)throws Exception {
        for(Object attribute:(List<?>)call(owner,"RR"))if(key.equals(call(attribute,"zeN")))return call(attribute,"getValue");
        return null;
    }
    static Object document(Object proxy)throws Exception {
        HostBuild.loaded(proxy).requireVerified();
        if(!Boolean.TRUE.equals(call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong mixer owner thread");
        Object document=call(proxy,"getDocument");if(document==null)throw new IllegalStateException("No current project");
        Object transport=value(document,"transport");
        for(String key:List.of("is_playing","is_playing_on_engine","record","write_automation"))
            if(Boolean.TRUE.equals(value(transport,key)))throw new IllegalStateException("Mixer snapshot/export requires stopped, nonrecording transport");
        if(Boolean.TRUE.equals(value(document,"is_being_saved")))throw new IllegalStateException("Project save in progress");
        return document;
    }
    static void requireIdleEngine(Object document)throws Exception {
        if(!((Enum<?>)call(document,"T1L")).name().equals("CONNECTED") || call(document,"jst")!=null)throw new IllegalStateException("Requires connected idle engine");
        requireQueueIdle(document);
    }
    static void requireQueueIdle(Object document)throws Exception {
        if(call(document,"jst")!=null)throw new IllegalStateException("Native document render active");
        Object queue=call(document,"R3h");
        for(String key:List.of("FHv","jaQ")) {
            var f=queue.getClass().getDeclaredField(key);f.setAccessible(true);Object actual=f.get(queue);
            if(key.equals("jaQ")?!((List<?>)actual).isEmpty():actual!=null)throw new IllegalStateException("Requires drained native render queue");
        }
    }
    static Object resolve(Object document,JsonNode path)throws Exception {
        ProjectAddress61.validate(path);
        if(!path.required("project").asText().equals(value(document,"name")))throw new IllegalArgumentException("Wrong mixer project");
        if(path.required("elements").isEmpty())return call(call(document,"peG"),"DQP");
        Object track=ProjectAddress61.resolveNative(path,document,document.getClass().getClassLoader());
        if(track.getClass().getName().equals("com.bitwig.flt.document.core.master.stI"))return call(track,"DQP");
        if(!track.getClass().getName().equals("yyk"))throw new IllegalArgumentException("Expected track/group/master path");
        return track;
    }
    ObjectNode read(Object proxy,EditorAdapter61 editor)throws Exception {
        Object document=document(proxy);
        ObjectNode result=Wire.object();result.put("document",editor.currentProjectGeneration());result.put("project",value(document,"name").toString());result.put("hostBuild",HostBuild.VERIFIED_61);result.put("nativeRevision",ProjectRevision61.read(proxy,document));
        Object transport=value(document,"transport");result.put("positionBeats",((Number)value(transport,"playback_position")).doubleValue());result.put("positionSeconds",((Number)value(transport,"position_in_seconds")).doubleValue());
        var beats=new java.util.TreeSet<Double>();beats.add(0.0);beats.add(((Number)value(transport,"playback_position")).doubleValue());
        Object markers=value(transport,"cue_marker_timeline");List<?> events=(List<?>)call(markers,"v33");if(events.size()>4096)throw new IllegalStateException("Cue time-context bound exceeded");
        for(Object event:events)beats.add(((Number)value(event,"time")).doubleValue());
        ArrayNode times=result.putArray("beatSeconds");for(double beat:beats)times.addObject().put("beat",beat).put("seconds",((Number)Reflection.call(document,"tGW",new Class<?>[]{double.class},beat)).doubleValue());
        ObjectNode root=Wire.object();root.put("project",value(document,"name").toString());root.putArray("elements");
        ArrayNode tracks=result.putArray("tracks");walk(document,call(document,"peG"),root,tracks,0);
        track(document,call(call(document,"peG"),"DQP"),root,"master",tracks);
        result.putArray("limitations").add("Stored stopped-document mixer settings; no effective playback values, meters or audio analysis.")
            .add("Pan is the native PANORAMA parameter; stereo balance law/mode is not inferred.")
            .add("Opaque plugin state, Kontakt patch/microphones and complete automation histories are outside this snapshot.");
        result.put("revision",authority.observe(result));result.put("captureTime",java.time.Instant.now().toString());return result;
    }
    private static ObjectNode child(JsonNode parent,Object owner,String kind)throws Exception {
        ObjectNode path=parent.deepCopy();path.withArray("elements").addObject().put("kind",kind).put("name",value(owner,"displayed_title").toString()).put("nativeId",value(owner,"managed_object_ID").toString());return path;
    }
    private void walk(Object document,Object parent,JsonNode path,ArrayNode output,int depth)throws Exception {
        if(depth>64)throw new IllegalStateException("Mixer hierarchy exceeds depth bound");
        for(Object owner:(List<?>)call(parent,"IFc")) {
            boolean group=owner.getClass().getName().equals("com.bitwig.flt.document.core.master.stI");
            ObjectNode next=child(path,owner,group?"group":"track");track(document,group?call(owner,"DQP"):owner,next,group?"group":"track",output);
            if(group)walk(document,owner,next,output,depth+1);
        }
        for(Object owner:(List<?>)call(parent,"UQt"))track(document,owner,child(path,owner,"track"),"fx",output);
    }
    private void track(Object document,Object owner,JsonNode path,String kind,ArrayNode output)throws Exception {
        if(output.size()>=4096)throw new IllegalStateException("Mixer track bound exceeded; nothing truncated");
        Object mixer=value(owner,"mixer");ObjectNode row=output.addObject();row.set("path",path);row.put("kind",kind);
        ObjectNode state=row.putObject("state");state.put("active",(boolean)value(owner,"active"));state.put("mute",(boolean)value(value(mixer,"mute"),"value"));
        row.put("effectivelyMuted",(boolean)value(mixer,"is_effectively_muted"));state.put("solo",(boolean)value(mixer,"solo"));
        row.set("volume",parameter(value(mixer,"volume"),true));row.set("pan",parameter(value(mixer,"pan"),false));
        Object destination=value(owner,"track_destination");ObjectNode route=row.putObject("output");
        Object endpoint=value(destination,"destination");route.set("destination",endpoint(document,endpoint));
        route.put("hasAudioDestination",(boolean)value(destination,"has_selected_audio_destination"));
        ArrayNode sends=row.putArray("sends");for(Object send:(List<?>)value(mixer,"sends")) {
            ObjectNode s=sends.addObject();s.put("nativeId",value(send,"managed_object_ID").toString());
            s.put("enabled",(boolean)value(send,"enable"));s.put("active",(boolean)value(send,"is_active"));
            s.put("mode",value(send,"send_mode").toString());s.put("preFader",(boolean)value(send,"pre_fader_result"));
            Object target=value(send,"channel_to_send_to");s.set("destination",endpoint(document,target));
            s.set("amount",parameter(value(send,"send_amount"),true));
        }
        devices(owner,row.putArray("devices"),0);
    }
    private static JsonNode endpoint(Object document,Object endpoint)throws Exception {
        if(endpoint==null)return Wire.JSON.nullNode();
        ObjectNode result=Wire.object();result.put("nativeId",value(endpoint,"managed_object_ID").toString());
        Object channel=endpoint;
        Class<?> track=Class.forName("com.bitwig.flt.document.core.master.kkr",true,document.getClass().getClassLoader());
        if(!track.isInstance(channel))channel=Reflection.call(endpoint,"EwU",new Class<?>[]{Class.class},track);
        if(channel!=null) {
            if(channel==call(call(document,"peG"),"DQP")){ObjectNode path=Wire.object();path.put("project",value(document,"name").toString());path.putArray("elements");result.set("owner",path);}
            else result.set("owner",ProjectAddress61.nativeTrackPath(document,channel));
            result.put("status","projectChannel");
        } else {result.put("status","externalOrDevice");result.put("limitation","Endpoint identity only; external hardware/device port semantics are not inferred.");}
        return result;
    }
    private static void devices(Object owner,ArrayNode rows,int depth)throws Exception {
        if(depth>64)throw new IllegalStateException("Device chain depth exceeded");
        Object chain=call(owner,owner.getClass().getName().equals("com.bitwig.flt.document.core.master.device.kdt")?"PPp":"Vdp");
        for(Object device:(List<?>)call(chain,"ATl")) {
            if(rows.size()>=4096)throw new IllegalStateException("Device inventory bound exceeded");
            ObjectNode row=rows.addObject();String name=value(device,"user_device_name").toString();if(name.isEmpty())name=value(device,"device_name").toString();
            row.put("name",name);row.put("nativeId",value(device,"managed_object_ID").toString());String id=call(device,"v33").toString();row.put("deviceId",id);
            Object active=optional(device,"active");if(active!=null && !(active instanceof Boolean))throw new IllegalStateException("Unqualified device activation type");row.set("active",Wire.JSON.valueToTree(active));
            if(active==null)row.put("limitation","Device activation/bypass not exposed by this qualified field.");
            if(id.equals("5024be2e-65d6-4d40-bbfe-8b2ea993c445") || id.equals("9588fbcf-721a-438b-8555-97e4231f7d2c")) {
                Object contents=value(device,"device_contents"),container=Reflection.call(contents,"v33",new Class<?>[]{String.class},id.equals("5024be2e-65d6-4d40-bbfe-8b2ea993c445")?"CHAIN_LIST":"CHAIN_SELECTOR");
                ArrayNode layers=row.putArray("layers");for(Object layer:(List<?>)value(container,"nested_device_chains")) {
                    ObjectNode l=layers.addObject();l.put("name",value(layer,"displayed_title").toString());devices(layer,l.putArray("devices"),depth+1);
                }
            }
        }
    }
    private static ObjectNode parameter(Object parameter,boolean volume)throws Exception {
        Object physical=value(parameter,"value_type");String domain=value(physical,"domain").toString(),unit=value(physical,"unit").toString();
        double nativeValue=((Number)value(parameter,"value")).doubleValue();
        if(!domain.equals(volume?"CUBE_ROOT":"LINEAR") || !unit.equals(volume?"DECIBEL":"PERCENT") || !value(physical,"engine_domain").toString().equals("LINEAR"))throw new IllegalStateException("Unqualified mixer domain");
        ClassLoader loader=parameter.getClass().getClassLoader();Class<?> conversion=Class.forName("fw",true,loader),domainType=Class.forName("fu",true,loader),unitType=Class.forName("fv",true,loader);
        double display=((Number)conversion.getMethod("v33",double.class,domainType,unitType).invoke(null,nativeValue,value(physical,"domain"),value(physical,"unit"))).doubleValue();
        double amplitude=((Number)conversion.getMethod("v33",double.class,domainType,domainType).invoke(null,nativeValue,value(physical,"domain"),value(physical,"engine_domain"))).doubleValue();
        ObjectNode p=Wire.object();p.put("nativeValue",nativeValue);p.put("min",((Number)value(physical,"min")).doubleValue());p.put("max",((Number)value(physical,"max")).doubleValue());p.put("domain",domain);p.put("unit",unit);
        if(Double.isFinite(display))p.put("displayValue",display);else p.putNull("displayValue");if(volume)p.put("linearAmplitude",amplitude);else p.putNull("linearAmplitude");
        p.put("automated",(boolean)value(parameter,"has_automation"));p.put("modulated",(boolean)value(parameter,"has_modulations"));return p;
    }
    String run(Object proxy,EditorAdapter61 editor,ObjectNode request)throws Exception {
        dispatched=false;String action=request.required("action").asText();ObjectNode before=read(proxy,editor),result=Wire.object();result.set("observation",before);result.putNull("error");
        if(action.equals("read"))return result.toString();
        if(!action.equals("set") || !sameScope(before,request.required("expected")))throw new IllegalArgumentException("Unknown action or stale complete mixer observation; no mutation");
        Object document=document(proxy);requireIdleEngine(document);
        JsonNode changes=request.required("changes");if(!changes.isArray() || changes.isEmpty() || changes.size()>4096)throw new IllegalArgumentException("Require bounded changes");
        List<Object> attributes=new ArrayList<>();List<Double> desired=new ArrayList<>();HashSet<String> paths=new HashSet<>();
        for(JsonNode change:changes) {
            JsonNode path=change.required("path");if(!paths.add(path.toString()))throw new IllegalArgumentException("Repeated mixer owner");
            Object owner=resolve(document,path),mixer=value(owner,"mixer");
            boolean present=false;for(String key:List.of("volumeDb","pan"))if(change.hasNonNull(key)) {
                present=true;if(key.equals("pan") && !path.required("elements").isEmpty() && path.required("elements").get(path.required("elements").size()-1).required("kind").asText().equals("group"))throw new IllegalArgumentException("Group pan is not admitted");Object parameter=value(mixer,key.equals("volumeDb")?"volume":"pan");ObjectNode observed=parameter(parameter,key.equals("volumeDb"));
                if(observed.required("automated").asBoolean() || observed.required("modulated").asBoolean())throw new IllegalStateException("Automated/modulated mixer parameter refuses static write");
                double input=change.required(key).doubleValue();if(!change.required(key).isNumber() || !Double.isFinite(input) || (key.equals("volumeDb")?(input < -120 || input > 6):(input < -1 || input > 1)))throw new IllegalArgumentException("Invalid mixer target");
                double target=key.equals("volumeDb")?Math.pow(10,input/60.0):input;
                if(target<observed.required("min").doubleValue() || target>observed.required("max").doubleValue())throw new IllegalArgumentException("Mixer target outside native range");
                if(Double.doubleToLongBits(target)!=Double.doubleToLongBits(observed.required("nativeValue").doubleValue())) {attributes.add(Reflection.call(parameter,"Nln",new Class<?>[]{String.class},"value"));desired.add(target);}
            }
            if(!present)throw new IllegalArgumentException("Empty mixer change");
        }
        if(attributes.isEmpty())return result.toString();EditorAdapter61.requireIndependentUndo(document);
        dispatched=true;Reflection.call(document,"v33",new Class<?>[]{String.class,boolean.class},"Mix track balance",false);
        try {for(int i=0;i<attributes.size();i++)Reflection.call(attributes.get(i),"setValue",new Class<?>[]{Object.class},desired.get(i));
            call(call(call(call(call(document,"cuK"),"TcB"),"v33"),"v33"),"LHs");
            for(int i=0;i<attributes.size();i++)if(Double.doubleToLongBits(((Number)call(attributes.get(i),"getValue")).doubleValue())!=Double.doubleToLongBits(desired.get(i)))throw new IllegalStateException("Native mixer readback differs");
        }catch(Exception error){result.put("error","Mixer batch may be partial; reconcile readback/history: "+error);}
        finally {Reflection.call(document,"v33",new Class<?>[]{boolean.class},false);}
        result.set("observation",read(proxy,editor));return result.toString();
    }
}
