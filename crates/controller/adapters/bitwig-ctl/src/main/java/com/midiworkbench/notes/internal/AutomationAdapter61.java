package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Admitted-build stored arranger and launcher source curves; invoked only on the document thread. */
final class AutomationAdapter61 {
    private final Object proxy;
    private final ClassLoader loader;
    record Owner(Object lane,Object clip,Object launcherSlot,Object source,Object timeline,Object valueType) {}
    record Prepared(Owner owner,ObjectNode edit,Map<String,Object> points) {}
    AutomationAdapter61(Object proxy) {
        this.proxy=proxy;this.loader=proxy.getClass().getClassLoader();
    }
    private Class<?> type(String name) throws Exception {return Class.forName(name,true,loader);}
    private static Object call(Object owner,String method) throws Exception {return Reflection.call(owner,method);}
    private static Object value(Object owner,String field) throws Exception {return ProjectAdapter61.value(owner,field);}
    private static String id(Object owner) throws Exception {return value(owner,"managed_object_ID").toString();}
    private static String key(String track,String lane,String clip) {return track+":"+lane+":"+clip;}
    private static double number(Object value) {return ((Number)value).doubleValue();}
    private static void set(Object owner,String field,Object value) throws Exception {
        Reflection.call(Reflection.call(owner,"Nln",new Class<?>[]{String.class},field),"setValue",new Class<?>[]{Object.class},value);
    }
    private static void keys(JsonNode node,Set<String> allowed) {
        if(!node.isObject())throw new IllegalArgumentException("Expected automation object");
        node.fieldNames().forEachRemaining(k->{if(!allowed.contains(k))throw new IllegalArgumentException("Unknown automation field "+k);});
        for(String k:allowed)node.required(k);
    }
    private static String text(JsonNode node,String field) {
        JsonNode v=node.required(field);if(!v.isTextual() || v.asText().isEmpty() || v.asText().length()>128 || v.asText().indexOf(0)>=0)throw new IllegalArgumentException("Invalid automation identity "+field);return v.asText();
    }
    private static double finite(JsonNode node,String field) {
        JsonNode n=node.required(field);if(!n.isNumber() || !Double.isFinite(n.doubleValue()))throw new IllegalArgumentException("Non-finite automation "+field);return n.doubleValue();
    }
    private static ObjectNode point(Object p) throws Exception {
        ObjectNode n=Wire.object();n.put("id",id(p));n.put("time",number(value(p,"time")));n.put("value",number(value(p,"value")));
        if(p.getClass().getName().equals("nCh")) {
            n.put("interpolation",value(p,"interpolation_mode").toString());n.put("curvature",number(value(p,"curvature")));n.put("spread",number(value(p,"spread")));
        } else if(p.getClass().getName().equals("Oss")) {n.put("interpolation","HOLD");n.put("curvature",0);n.put("spread",0);}
        else throw new IllegalStateException("Unsupported native automation point representation");
        return n;
    }
    ArrayNode readTrack(Object document,Object track,boolean includeArranger,List<Object> launcherClips,Map<String,Owner> owners) throws Exception {
        ArrayNode result=Wire.JSON.createArrayNode();
        String trackId=id(track);Object lanes=value(track,"automation_lanes");
        for(Object lane:(List<?>)Reflection.call(lanes,"XrN",new Class<?>[]{String.class},new Object[]{null})) {
            // Bitwig exposes the UI's "about to be created" preview beside stored
            // lanes. It is not document automation and refuses event insertion.
            if(type("owf").isInstance(lane))continue;
            String laneContext="track "+trackId+", lane "+id(lane);
            try {
                Object atom=value(lane,"value_atom"),reference=value(lane,"atom_reference");
                String context=laneContext;
                if(reference==null)throw new IllegalStateException("Unsupported stored automation owner ("+context+"): atom_reference is absent; complete read refused, no lanes omitted");
                context+=", reference "+id(reference);
                if(atom==null) {
                    // dsv.value_atom is a cached target. The persistent reference's
                    // hBi.qKA/vCm.Zlo getter reads its canonical stored target.
                    atom=call(reference,"qKA");
                    if(atom==null || !type("bXo").isInstance(atom))throw new IllegalStateException("Unsupported unresolved stored automation target ("+context+"): neither lane value_atom nor reference target resolves to a native value atom; complete read refused, no lanes omitted");
                }
                Object valueType=call(reference,"heu");
                if(valueType==null)throw new IllegalStateException("Unsupported stored automation target ("+context+"): reference value type is absent; complete read refused, no lanes omitted");
                if(includeArranger) {
                    Object arranger=call(lane,"XQF");List<?> clips=(List<?>)call(arranger,"v33");
                    Object outside=value(arranger,"outside_timeline");
                    addLane(result,document,track,lane,atom,reference,valueType,null,outside,clips,null,null,owners);
                    for(Object clip:clips) {
                        if(!type("I3W").isInstance(clip))throw new IllegalStateException("Unexpected native automation clip");
                        addLane(result,document,track,lane,atom,reference,valueType,clip,call(clip,"mQv"),clips,null,null,owners);
                    }
                }
                if(!launcherClips.isEmpty()) {
                    List<LauncherCurve> placements=new ArrayList<>();
                    for(Object noteClip:launcherClips) {
                        Object slot=launcherSlot(lane,noteClip,false),clip=Launcher61.clip(slot);
                        if(clip!=null)placements.add(new LauncherCurve(noteClip,slot,clip));
                    }
                    List<Object> clips=placements.stream().map(LauncherCurve::clip).toList();
                    for(LauncherCurve placement:placements) {
                        Object clip=placement.clip();
                        addLane(result,document,track,lane,atom,reference,valueType,clip,call(clip,"mQv"),clips,placement.noteClip(),placement.slot(),owners);
                    }
                }
            }catch(Exception error) {
                throw new IllegalStateException("Complete automation read failed ("+laneContext+"): "+error.getMessage(),error);
            }
        }
        return result;
    }
    private record LauncherCurve(Object noteClip,Object slot,Object clip) {}

    private Object launcherSlot(Object lane,Object noteClip,boolean createSlot) throws Exception {
        Object slots=call(lane,"TcB");
        int scene=Launcher61.scene(Launcher61.noteSlot(noteClip));
        if(scene>=((Number)call(slots,"FSQ")).intValue()) {
            if(!createSlot)return null;
            call(slots,"ncc",new Class<?>[]{int.class},scene+1);
        }
        return call(slots,"jaQ",new Class<?>[]{int.class},scene);
    }

    private void addLane(ArrayNode result,Object document,Object track,Object lane,Object atom,Object reference,
            Object valueType,Object clip,Object timeline,List<?> clips,Object noteClip,Object launcherSlot,
            Map<String,Owner> owners) throws Exception {
        String trackId=id(track);
        Object source=clip==null?timeline:call(clip,"dxV");String clipId=clip==null?id(timeline):id(clip);
        ObjectNode c=result.addObject();c.set("owner",ProjectAddress61.nativeTrackPath(document,track));c.put("placement",noteClip!=null?"launcher":clip==null?"arranger":"clip");
        if(noteClip!=null)c.put("noteClip",id(noteClip));
        c.put("track",trackId);c.put("lane",id(lane));c.put("clip",clipId);c.put("source",id(source));
        ArrayNode aliases=c.putArray("aliases");
        if(clip==null){aliases.add(clipId);c.put("referenceCount",1);}
        else {
            for(Object alias:clips)if(call(alias,"dxV")==source)aliases.add(id(alias));
            c.put("referenceCount",((Number)value(value(source,"shared_timeline_info"),"reference_count")).intValue());
        }
        ObjectNode mapping=c.putObject("mapping");
        if(clip!=null) {
            mapping.put("startBeats",noteClip!=null?0.0:number(call(clip,"T1L")));
            mapping.put("durationBeats",number(call(clip,noteClip!=null?"Fgm":"GaO")));
            mapping.put("sourceStartBeats",number(call(clip,"dD_")));mapping.put("sourceEndBeats",number(call(clip,"Fgm")));mapping.put("loopEnabled",(boolean)call(clip,"YeF"));
            Object loop=call(clip,"fvB");mapping.put("loopStart",number(value(loop,"time")));mapping.put("loopLength",number(value(loop,"duration")));
        }
        Object physical=value(atom,"value_type");
        if(physical==null)throw new IllegalStateException("Unsupported stored automation target (track "+trackId+", lane "+id(lane)+", reference "+id(reference)+", atom "+id(atom)+"): physical value type is absent; complete read refused, no lanes omitted");
        ObjectNode target=c.putObject("target");
        target.put("kind","parameter");target.put("nativeParameterId",id(atom));target.put("name",String.valueOf(value(atom,"name")));
        if(type("am").isInstance(valueType))for(String field:List.of("min","max","domain","unit"))target.set(field,Wire.JSON.valueToTree(value(physical,field)));
        else target.put("domain","discrete");
        target.put("pointType",type("am").isInstance(valueType)?"continuous":"unsupported");
        if(type("am").isInstance(valueType))target.put("fallbackValue",number(call(valueType,"TUp",new Class<?>[]{double.class},number(call(reference,"aV_")))));
        if(type("HQO").isInstance(atom)) {
            int status=((Number)call(atom,"tGW")).intValue();
            if((status&0xf0)==0xb0){target.put("kind","midiCc");target.put("channel",status&15);target.put("controller",((Number)call(atom,"Iz2")).intValue());}
        }
        ArrayNode points=c.putArray("points");for(Object point:(List<?>)call(timeline,"v33"))points.add(point(point));
        if(owners!=null)owners.put(key(trackId,id(lane),clipId),new Owner(lane,clip,launcherSlot,source,timeline,valueType));
    }
    private static List<JsonNode> curves(JsonNode snapshot) {
        List<JsonNode> result=new ArrayList<>();
        snapshot.required("automation").forEach(result::add);
        return result;
    }
    private Object call(Object owner,String method,Class<?>[] signature,Object... args) throws Exception {return Reflection.call(owner,method,signature,args);}
    List<Prepared> prepare(ObjectNode snapshot,Map<String,Owner> owners,JsonNode edits) throws Exception {
        if(!edits.isArray())throw new IllegalArgumentException("Automation edits must be an array");
        List<Prepared> prepared=new ArrayList<>();Set<Object> sources=java.util.Collections.newSetFromMap(new java.util.IdentityHashMap<>());
        Set<String> known=new HashSet<>(),temporary=new HashSet<>();int total=0;
        for(JsonNode c:curves(snapshot))for(JsonNode p:c.required("points"))known.add(p.required("id").asText());
        for(JsonNode raw:edits) {
            keys(raw,Set.of("track","lane","clip","aliases","points"));
            String k=key(text(raw,"track"),text(raw,"lane"),text(raw,"clip"));Owner owner=owners.get(k);
            if(owner==null || !sources.add(owner.source()))throw new IllegalArgumentException("Unknown automation owner or duplicate source edit");
            JsonNode curve=null;for(JsonNode c:curves(snapshot))if(k.equals(key(c.required("track").asText(),c.required("lane").asText(),c.required("clip").asText())))curve=c;
            ObjectNode edit=raw.deepCopy();validateCurve(edit,curve,known,temporary);edit.set("fallbackValue",curve.required("target").required("fallbackValue"));
            total+=edit.required("points").size();if(total>100000)throw new IllegalArgumentException("Automation result exceeds point budget");
            Map<String,Object> points=new LinkedHashMap<>();for(Object p:(List<?>)call(owner.timeline(),"v33"))points.put(id(p),p);
            prepared.add(new Prepared(owner,edit,points));
        }
        return prepared;
    }
    void project(ObjectNode expected,List<Prepared> edits) throws Exception {
        for(Prepared edit:edits)for(JsonNode lane:expected.required("automation"))
            if(lane.required("source").asText().equals(id(edit.owner().source())))((ObjectNode)lane).set("points",edit.edit().required("points"));
    }
    ObjectNode writeAll(List<Prepared> edits,ObjectNode identities) throws Exception {
        ObjectNode defaults=Wire.object();
        for(Prepared edit:edits) {
            write(edit,identities);
            if(edit.edit().required("points").isEmpty()) {
                call(call(call(call(call(call(proxy,"getDocument"),"cuK"),"TcB"),"v33"),"v33"),"LHs");
                var points=(List<?>)call(edit.owner().timeline(),"v33");
                if(points.size()!=1)throw new IllegalStateException("Native clear did not return one fallback point");
                ObjectNode generated=point(points.get(0));
                if(generated.required("time").doubleValue()!=0 || generated.required("value").doubleValue()!=edit.edit().required("fallbackValue").doubleValue() || !generated.required("interpolation").asText().equals("LINEAR") || generated.required("curvature").doubleValue()!=0 || generated.required("spread").doubleValue()!=0)throw new IllegalStateException("Unexpected native clear canonicalization; rolling back");
                defaults.set(id(edit.owner().source()),generated);
                identities.put("default:"+id(edit.owner().source()),generated.required("id").asText());
                edit.edit().withArray("points").add(generated);
            }
        }
        return defaults;
    }
    record Creation(Object track,Object noteClip,Object parameter,ObjectNode input) {}
    List<Creation> prepareCreates(ObjectNode snapshot,Map<String,Object> clips,Set<String> arrangerTracks,JsonNode inputs)throws Exception {
        List<Creation> result=new ArrayList<>();Set<String> known=new HashSet<>(),temporary=new HashSet<>(),targets=new HashSet<>();
        for(JsonNode curve:curves(snapshot))for(JsonNode p:curve.required("points"))known.add(p.required("id").asText());
        for(JsonNode input:inputs) {
            JsonNode target=input.required("target");boolean parameterTarget=target.path("kind").asText().equals("parameter");
            if(parameterTarget)keys(target,Set.of("kind","device","nativeParameterId","name","min","max","domain","unit","pointType"));
            else keys(target,Set.of("kind","channel","controller"));
            boolean launcher=input.required("placement").asText().equals("launcher");
            if(!launcher && !input.required("placement").asText().equals("arranger"))
                throw new IllegalArgumentException("Expected arranger or launcher automation");
            Object document=call(proxy,"getDocument");
            Object track=ProjectAddress61.resolveNative(input.required("owner"),document,loader);
            if(track.getClass().getName().equals("com.bitwig.flt.document.core.master.stI"))track=call(track,"DQP");
            Object noteClip=null;
            if(launcher) {
                noteClip=clips.get(text(input,"noteClip"));
                if(noteClip==null || Launcher61.noteSlot(noteClip)==null || call(Launcher61.noteSlot(noteClip),"eu_")!=track)
                    throw new IllegalArgumentException("Launcher automation requires an observed note clip on its owning track");
                if(arrangerTracks.contains(id(track)))throw new IllegalArgumentException("Create launcher automation through a launcher-only clip scope");
            } else if(!arrangerTracks.contains(id(track)))throw new IllegalArgumentException("Arranger automation requires an arranger scope");
            Object parameter=null;String identity;
            if(parameterTarget) {
                JsonNode devicePath=target.required("device");
                if(!ProjectAddress61.trackPath(devicePath).equals(input.required("owner")) || !Set.of("device","plugin","layer","selector").contains(devicePath.required("elements").get(devicePath.required("elements").size()-1).path("kind").asText()))throw new IllegalArgumentException("Knob device must belong to its observed track");
                parameter=DeviceParameters61.resolve(document,target);
                identity="parameter:"+id(parameter);
            } else {
                if(!target.required("controller").isIntegralNumber() || target.required("controller").intValue()<0 || target.required("controller").intValue()>119 || !target.required("kind").asText().equals("midiCc") || !target.required("channel").isIntegralNumber() || target.required("channel").intValue()<0 || target.required("channel").intValue()>15)throw new IllegalArgumentException("Expected native channel CC arranger data");
                identity="midiCc:"+target.required("channel")+":"+target.required("controller");
            }
            boolean owned=false;for(JsonNode path:snapshot.required("owners"))if(path.equals(input.required("owner")))owned=true;
            if(!owned || !targets.add(id(track)+":"+input.path("noteClip").asText()+":"+identity))throw new IllegalArgumentException("New automation owner is absent or repeated");
            for(JsonNode curve:curves(snapshot))if(curve.required("track").asText().equals(id(track)) && curve.path("noteClip").equals(input.path("noteClip")) && (parameterTarget?curve.required("target").path("nativeParameterId").equals(target.required("nativeParameterId")):curve.required("target").path("kind").asText().equals("midiCc") && curve.required("target").path("channel").equals(target.required("channel")) && curve.required("target").path("controller").equals(target.required("controller"))))throw new IllegalArgumentException("Automation lane already exists; edit its complete points");
            ObjectNode physical=Wire.object();physical.put("min",parameterTarget?target.required("min").doubleValue():0.0);physical.put("max",parameterTarget?target.required("max").doubleValue():1.0);physical.put("domain",parameterTarget?target.required("domain").asText():"LINEAR");physical.put("pointType",parameterTarget?target.required("pointType").asText():"continuous");
            JsonNode points=input.required("points");if(points.isEmpty() || points.get(0).required("time").doubleValue()!=0)throw new IllegalArgumentException("New automation must declare its beat-zero value");
            validatePoints(points,Wire.JSON.createArrayNode(),physical,known,temporary);
            result.add(new Creation(track,noteClip,parameter,input.deepCopy()));
        }
        return result;
    }
    List<Owner> prepareDeletes(ObjectNode snapshot,Map<String,Owner> owners,JsonNode inputs)throws Exception {
        List<Owner> result=new ArrayList<>();
        for(JsonNode input:inputs) {
            if(!Set.of("clip","launcher").contains(input.required("placement").asText()) || input.required("referenceCount").intValue()!=1 || input.required("aliases").size()!=1 || curves(snapshot).stream().noneMatch(input::equals))throw new IllegalArgumentException("Only an observed exclusive automation placement can be deleted");
            Owner owner=owners.get(key(input.required("track").asText(),input.required("lane").asText(),input.required("clip").asText()));
            if(owner==null || owner.clip()==null || result.contains(owner))throw new IllegalArgumentException("Missing or repeated automation deletion");result.add(owner);
        }
        return result;
    }
    void deleteAll(List<Owner> inputs)throws Exception {
        for(Owner owner:inputs) {
            if(owner.launcherSlot()!=null)call(owner.launcherSlot(),"Vdp");
            else call(call(owner.lane(),"XQF"),"v33",new Class<?>[]{type("oHd"),boolean.class},owner.clip(),false);
        }
    }
    List<ObjectNode> createAll(List<Creation> inputs,ObjectNode identities)throws Exception {
        List<ObjectNode> result=new ArrayList<>();Object document=call(proxy,"getDocument");
        for(Creation creation:inputs) {
            JsonNode target=creation.input().required("target");boolean parameterTarget=target.required("kind").asText().equals("parameter");
            Object parameter=creation.parameter();
            if(!parameterTarget)parameter=call(value(creation.track(),"midi_parameter_container"),"TUp",new Class<?>[]{int.class,int.class},target.required("channel").intValue(),target.required("controller").intValue());
            Object reference=type("com.bitwig.flt.document.core.master.oJn").getMethod("v33",type("bXo")).invoke(null,parameter),valueType=call(reference,"heu");
            Object lane=call(creation.track(),"YLV",new Class<?>[]{type("com.bitwig.flt.document.core.iface.bf")},reference);
            Object slot=null,clip=null,timeline;
            Object noteClip=creation.noteClip();
            if(noteClip==null)timeline=value(call(lane,"XQF"),"outside_timeline");
            else {
                slot=launcherSlot(lane,noteClip,true);
                if(Launcher61.clip(slot)!=null)throw new IllegalStateException("Launcher automation slot is occupied");
                clip=call(slot,"v33",new Class<?>[]{double.class,type("xpF")},number(call(noteClip,"Fgm")),lane);
                call(clip,"TUp",new Class<?>[]{type("gnS")},noteClip);
                call(slot,"TUp",new Class<?>[]{type("fza")},clip);
                timeline=call(clip,"mQv");
            }
            Map<String,Object> original=new LinkedHashMap<>();for(Object point:(List<?>)call(timeline,"v33"))original.put(id(point),point);
            ObjectNode edit=Wire.object();edit.set("points",creation.input().required("points").deepCopy());
            Object source=clip==null?timeline:call(clip,"dxV");
            write(new Prepared(new Owner(lane,clip,slot,source,timeline,valueType),edit,original),identities,true);
            ObjectNode observed=null;
            for(JsonNode curve:readTrack(document,creation.track(),noteClip==null,noteClip==null?List.of():List.of(noteClip),null))
                if(curve.required("lane").asText().equals(id(lane)) && curve.required("placement").equals(creation.input().required("placement")))observed=(ObjectNode)curve;
            if(observed==null || !observed.required("points").equals(edit.required("points")) || !observed.required("target").path("kind").equals(target.required("kind")) || (parameterTarget?!observed.required("target").path("nativeParameterId").equals(target.required("nativeParameterId")):!observed.required("target").path("channel").equals(target.required("channel")) || !observed.required("target").path("controller").equals(target.required("controller"))))throw new IllegalStateException("Native CC creation differs; rolling back");
            result.add(observed);
        }
        return result;
    }
    void projectTopology(ObjectNode expected,ObjectNode observed,List<Owner> deleted,List<ObjectNode> created)throws Exception {
        if(deleted.isEmpty() && created.isEmpty())return;
        ArrayNode curves=expected.withArray("automation");
        for(Owner owner:deleted)for(int i=curves.size()-1;i>=0;i--)
            if(curves.get(i).required("lane").asText().equals(id(owner.lane())) && curves.get(i).required("clip").asText().equals(id(owner.clip())))curves.remove(i);
        int index=0;
        for(JsonNode actual:observed.required("automation")) {
            for(ObjectNode lane:created)
                if(lane.required("lane").equals(actual.required("lane")) && lane.required("clip").equals(actual.required("clip")))curves.insert(index,lane);
            index++;
        }
    }
    static void validateCurve(ObjectNode edit,JsonNode curve,Set<String> known,Set<String> temporary) {
        Set<String> aliases=new HashSet<>();JsonNode a=edit.required("aliases");
        if(!a.isArray() || a.isEmpty() || a.size()>4096)throw new IllegalArgumentException("Explicit automation aliases required");
        for(JsonNode v:a)if(!v.isTextual() || !aliases.add(v.asText()))throw new IllegalArgumentException("Duplicate/invalid automation alias");
        Set<String> expected=new HashSet<>();for(JsonNode v:curve.required("aliases"))expected.add(v.asText());
        if(!aliases.equals(expected) || curve.required("referenceCount").intValue()!=expected.size())throw new IllegalArgumentException("Automation source has unacknowledged or unobserved aliases");
        validatePoints(edit.required("points"),curve.required("points"),curve.required("target"),known,temporary);
    }
    private static void validatePoints(JsonNode points,JsonNode original,JsonNode target,Set<String> known,Set<String> temporary) {
        double min=finite(target,"min"),max=finite(target,"max");
        if(!target.path("pointType").asText().equals("continuous") || !target.path("domain").asText().equals("LINEAR") || min>=max)throw new IllegalArgumentException("Unsupported automation physical domain/representation");
        if(!points.isArray() || points.size()>100000)throw new IllegalArgumentException("Automation point budget exceeded");
        Set<String> own=new HashSet<>(),ids=new HashSet<>();Map<String,JsonNode> originals=new HashMap<>();
        for(JsonNode p:original){own.add(p.required("id").asText());originals.put(p.required("id").asText(),p);}
        double previous=Double.NEGATIVE_INFINITY;
        for(JsonNode p:points) {
            keys(p,Set.of("id","time","value","interpolation","curvature","spread"));String id=text(p,"id");double time=finite(p,"time"),v=finite(p,"value"),c=finite(p,"curvature"),s=finite(p,"spread");
            if(!ids.add(id) || (!own.contains(id) && (known.contains(id) || !temporary.add(id))))throw new IllegalArgumentException("Duplicate/cross-source automation point identity");
            if(time<previous || v<min || v>max || c< -1 || c>1 || s<0 || (p.required("interpolation").asText().equals("HOLD") && c!=0) || !Set.of("HOLD","LINEAR").contains(p.required("interpolation").asText()))throw new IllegalArgumentException("Invalid automation point range/order/interpolation");previous=time;
            JsonNode old=originals.get(id);
            if(old==null || time!=old.required("time").doubleValue()) {
                double canonical=Math.round(time*1048576.0)*0x1.0p-20+0.0;
                if(time!=canonical)throw new IllegalArgumentException("Changed/new automation time must use native 2^-20 beat resolution");
            }
        }
        if(points.equals(original))throw new IllegalArgumentException("Automation edit makes no native change");
    }
    private void write(Prepared prepared,ObjectNode identities) throws Exception {write(prepared,identities,false);}
    private void write(Prepared prepared,ObjectNode identities,boolean creation) throws Exception {
        Object timeline=prepared.owner().timeline();List<Object> ordered=new ArrayList<>();
        Map<String,JsonNode> desiredById=new HashMap<>();
        for(JsonNode p:prepared.edit().required("points"))desiredById.put(p.required("id").asText(),p);
        Set<String> detached=new HashSet<>();
        for(var entry:prepared.points().entrySet()) {
            JsonNode desired=desiredById.get(entry.getKey());
            if(creation || desired==null || number(value(entry.getValue(),"time"))!=desired.required("time").doubleValue()) {
                call(timeline,"v33",new Class<?>[]{type("oHd"),boolean.class},entry.getValue(),false);detached.add(entry.getKey());
            }
        }
        for(JsonNode raw:prepared.edit().required("points")) {
            ObjectNode desired=(ObjectNode)raw;String oldId=desired.required("id").asText();Object p=creation?null:prepared.points().get(oldId);
            boolean created=p==null;
            if(created) {
                Object atom=value(prepared.owner().lane(),"value_atom");
                if(atom==null)atom=call(value(prepared.owner().lane(),"atom_reference"),"qKA");
                Object target=value(atom,"value_type");
                double min=number(value(target,"min")),max=number(value(target,"max"));
                p=call(prepared.owner().valueType(),"TUp",new Class<?>[]{double.class,double.class,type("CBo"),double.class},desired.required("time").doubleValue(),(desired.required("value").doubleValue()-min)/(max-min),type("CBo").getField(desired.required("interpolation").asText().equals("HOLD")?"v33":"TUp").get(null),desired.required("curvature").doubleValue());
                if(!p.getClass().getName().equals("nCh"))throw new IllegalStateException("Automation factory representation changed");
            }
            for(String field:List.of("time","value","curvature","spread"))if(number(value(p,field))!=desired.required(field).doubleValue())set(p,field,desired.required(field).doubleValue());
            Object mode=value(p,"interpolation_mode");
            if(!mode.toString().equals(desired.required("interpolation").asText())) {
                Object interpolation=java.util.Arrays.stream(mode.getClass().getEnumConstants()).filter(v->v.toString().equals(desired.required("interpolation").asText())).findFirst().orElseThrow();set(p,"interpolation_mode",interpolation);
            }
            if(created || detached.contains(oldId))call(timeline,"jaQ",new Class<?>[]{type("oHd")},p);
            if(created){identities.put(oldId,id(p));desired.put("id",id(p));}
            ordered.add(p);
        }
        for(int i=0;i<ordered.size();i++)if(((List<?>)call(timeline,"v33")).get(i)!=ordered.get(i)) {
            Object p=ordered.get(i);call(timeline,"v33",new Class<?>[]{type("oHd"),boolean.class},p,false);call(timeline,"YLV",new Class<?>[]{type("oHd"),int.class},p,i);
        }
    }
    static boolean samePersistent(ObjectNode a,ObjectNode b) {
        ObjectNode left=a.deepCopy(),right=b.deepCopy();left.remove("revision");right.remove("revision");return left.equals(right);
    }
}
