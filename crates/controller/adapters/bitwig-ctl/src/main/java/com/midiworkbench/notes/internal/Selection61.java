package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/** Absolute native selection. Does not send keys or establish keyboard focus. */
final class Selection61 {
    private final Object proxy,document;
    private final String generation;
    private final ObservedScope scope;
    private final java.util.function.BooleanSupplier current;
    private boolean dispatched;
    private final java.util.Map<String,JsonNode> knownPaths=new java.util.HashMap<>();
    Selection61(Object proxy,Object document,String generation,ObservedScope scope,java.util.function.BooleanSupplier current) {
        this.proxy=proxy;this.document=document;this.generation=generation;this.scope=scope;this.current=current;
    }
    private static Object call(Object o,String method)throws Exception{return Reflection.call(o,method);}
    private static Object value(Object o,String field)throws Exception{return ProjectAdapter61.value(o,field);}
    private static String id(Object o)throws Exception{return value(o,"managed_object_ID").toString();}
    private Class<?> type(String name)throws Exception{return Class.forName(name,true,proxy.getClass().getClassLoader());}
    private Object widget()throws Exception{return call(proxy,"getDocumentWidget");}
    private Object arranger()throws Exception{return call(call(proxy,"getDocumentGUIWrapper"),"iDx");}
    java.util.concurrent.CompletableFuture<String> run(ObjectNode request) {
        try {
            String action=request.required("action").asText();
            Set<String> allowed=action.equals("read")?Set.of("action"):Set.of("action","document","revision","paths","mode","range");
            request.fieldNames().forEachRemaining(k->{if(!allowed.contains(k))throw new IllegalArgumentException("Unknown selection field "+k);});
            if(!Set.of("read","set").contains(action))throw new IllegalArgumentException("Unknown selection action");
            ObjectNode before=read();if(action.equals("read"))return java.util.concurrent.CompletableFuture.completedFuture(before.toString());
            if(!before.required("document").equals(request.required("document")) || !before.required("revision").equals(request.required("revision")))throw new IllegalArgumentException("Stale selection authority; no selection changed");
            JsonNode paths=request.required("paths");String mode=request.required("mode").asText();
            if(!paths.isArray() || paths.size()>1024 || !Set.of("objects","time").contains(mode))throw new IllegalArgumentException("Invalid selection paths/mode");
            boolean time=mode.equals("time");JsonNode range=request.required("range");
            double from=0,to=0;
            if(time) {
                if(paths.isEmpty() || !range.isObject() || range.size()!=2 || !range.path("from").isNumber() || !range.path("to").isNumber())throw new IllegalArgumentException("Time selection requires lanes and arranger-beat bounds");
                from=range.required("from").asDouble();to=range.required("to").asDouble();
                if(!Double.isFinite(from) || !Double.isFinite(to) || from<0 || to<=from || Math.round(from*1048576.0)>=Math.round(to*1048576.0))throw new IllegalArgumentException("Invalid arranger-beat range");
            } else if(!range.isNull())throw new IllegalArgumentException("Object selection has no time range");
            List<Object> desired=new ArrayList<>();List<JsonNode> expected=new ArrayList<>();Set<String> seen=new HashSet<>();Object selection=null,wrapper=null;String family=null;
            for(JsonNode path:paths) {
                ProjectAddress61.validate(path);if(path.required("elements").isEmpty())throw new IllegalArgumentException("Cannot select project root");
                Object target=ProjectAddress61.resolveNative(path,document,proxy.getClass().getClassLoader());
                String kind=path.required("elements").get(path.required("elements").size()-1).required("kind").asText();
                Object owner;String current;
                if(time) {
                    if(kind.equals("track") || kind.equals("group"))target=value(kind.equals("group")?call(target,"DQP"):target,"main_note_clip_lane");
                    else if(!kind.equals("automationLane"))throw new IllegalArgumentException("Time selection targets track or automation lanes");
                    if(target==null)throw new IllegalArgumentException("Track has no arranger lane");
                    owner=call(call(arranger(),"Iks"),"qKA");current="time";
                } else if(kind.equals("clip")) {owner=call(call(arranger(),"jaQ"),"qKA");current="clips";}
                else if(kind.equals("track") || kind.equals("group")) {owner=Instrument61.trackSelection(document);current="tracks";}
                else if(kind.equals("layerChain") || kind.equals("choice")) {owner=Instrument61.slotSelection(call(target,"Iks"));current="slots";}
                else if(Set.of("device","plugin","layer","selector").contains(kind)) {owner=Instrument61.deviceSelection(target);current="devices";}
                else throw new IllegalArgumentException("Unsupported object selection target");
                if(selection!=null && (selection!=owner || !current.equals(family)))throw new IllegalArgumentException("Selection targets require one compatible native owner");
                if(!seen.add(id(target)))throw new IllegalArgumentException("Duplicate resolved selection target");
                selection=owner;family=current;
                if(wrapper==null)wrapper=switch(current) {
                    case "time" -> call(arranger(),"Iks");
                    case "clips" -> call(arranger(),"jaQ");
                    case "tracks" -> call(call(proxy,"getDocumentGUIWrapper"),"dDz");
                    case "devices" -> Reflection.call(call(proxy,"getDocumentGUIWrapper"),"v33",new Class<?>[]{type("dmQ")},Reflection.call(target,"EwU",new Class<?>[]{Class.class},type("dmQ")));
                    case "slots" -> type("bin").getConstructor(type("z33")).newInstance(call(target,"Iks"));
                    default -> throw new IllegalArgumentException("Unsupported native selection owner");
                };
desired.add(target);expected.add(pathFor(target));
            }
            if(selection==null) {
                Object inspected=value(widget(),"inspector_selection");
                if(inspected==null)throw new IllegalArgumentException("No supported active selection to clear");
                if(!before.required("unsupportedMembers").isEmpty())throw new IllegalArgumentException("Active selection contains unsupported members; cannot clear");
                selection=call(inspected,"qKA");wrapper=inspected;
                // Reading all current members above must succeed before clearing them.
            }
            Reflection.method(selection.getClass(),"v33",Collection.class);
            if(!desired.isEmpty())Reflection.method(selection.getClass(),"qKA",type("com.bitwig.ramona.core.Zj3"));
            Object arranger=arranger(),settings=call(arranger,"mP_");
            Object flag=Reflection.call(settings,"Nln",new Class<?>[]{String.class},"is_time_selection_active");
            Reflection.method(flag.getClass(),"setValue",Object.class);
            Reflection.method(widget().getClass(),"EwU",type("bhU"));
            if(time)Reflection.method(call(arranger,"Iks").getClass(),"v33",double.class,double.class);
            List<Instrument61.SelectionStep> ancestors=family!=null && (family.equals("devices") || family.equals("slots"))?Instrument61.prepareSelectionPath(document,paths.get(0)):List.of();
            Object scheduler=type("doS").getMethod("RzK").invoke(null);
            Reflection.method(scheduler.getClass(),"runAtTheVeryEndOfCurrentEvent",Runnable.class);
            Object notifications=call(call(call(call(document,"cuK"),"TcB"),"v33"),"v33");
            Reflection.method(notifications.getClass(),"LHs");
            dispatched=true;
            for(Instrument61.SelectionStep step:ancestors)step.apply();
            if(!desired.isEmpty())Reflection.call(selection,"qKA",new Class<?>[]{type("com.bitwig.ramona.core.Zj3")},desired.getFirst());
            Reflection.call(selection,"v33",new Class<?>[]{Collection.class},desired);
            if(family!=null && (family.equals("clips") || time)) {
                if(time)Reflection.call(call(arranger,"Iks"),"v33",new Class<?>[]{double.class,double.class},from,to);
                Reflection.call(flag,"setValue",new Class<?>[]{Object.class},time);

            }
            // Mode observers reconcile real focus at end-of-event. Hand off the
            // Inspector only after that native boundary, without forging focus.
            call(notifications,"LHs");
            return finish(scheduler,wrapper,expected,mode,time,from,to);
        } catch(Exception error) {return java.util.concurrent.CompletableFuture.failedFuture(dispatched?error:new NativeRefusal(error));}
    }
    private java.util.concurrent.CompletableFuture<String> finish(Object scheduler,Object wrapper,List<JsonNode> expected,String mode,boolean time,double from,double to)throws Exception {
        var result=new java.util.concurrent.CompletableFuture<String>();
        Reflection.call(scheduler,"runAtTheVeryEndOfCurrentEvent",new Class<?>[]{Runnable.class},(Runnable)()->{
            try {
                if(!current.getAsBoolean() || call(proxy,"getDocument")!=document)throw new IllegalStateException("Selection document/adapter changed before native readback; outcome unknown");
                if(!Boolean.TRUE.equals(call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong selection completion thread");
                Reflection.call(widget(),"EwU",new Class<?>[]{type("bhU")},wrapper);
                ObjectNode after=read();Set<JsonNode> actual=new HashSet<>();after.required("members").forEach(actual::add);
                if(!actual.equals(new HashSet<>(expected)) || (!expected.isEmpty() && !after.required("primary").equals(expected.getFirst())) || (!expected.isEmpty() && !after.required("mode").asText().equals(mode)))throw new IllegalStateException("Selection readback differs; outcome unknown, reread selection");
                if(time && (after.required("range").path("from").asDouble()!=Math.round(from*1048576.0)/1048576.0 || after.required("range").path("to").asDouble()!=Math.round(to*1048576.0)/1048576.0))throw new IllegalStateException("Selection range differs; outcome unknown");
                result.complete(after.toString());
            } catch(Exception error) {result.completeExceptionally(error);}
        });
        return result;
    }
    ObjectNode read()throws Exception {
        ObjectNode result=Wire.object();result.put("document",generation);
        Object active=value(widget(),"active_editor_selection"),inspected=value(widget(),"inspector_selection");
        result.put("editor",active==null?"none":active==arranger()?"arranger":type("biD").isInstance(active)?"detailEditor":"unsupported");
        ArrayNode members=result.putArray("members"),unsupported=result.putArray("unsupportedMembers");
        boolean time=inspected!=null && type("bhV").isInstance(inspected);
        result.put("mode",time?"time":"objects");result.putNull("range");result.putNull("primary");
        if(inspected!=null) {
            Object nativeSelection=call(inspected,"qKA");
            for(Object target:(List<?>)call(nativeSelection,"jaQ")) { try { members.add(member(target)); } catch(IllegalArgumentException outside) { unsupported.add(id(target)); } }
            Object primary=call(nativeSelection,"XQF");if(primary!=null) { try {result.set("primary",member(primary));}catch(IllegalArgumentException outside){result.put("unsupportedPrimary",id(primary));} }
            if(time) {ObjectNode range=result.putObject("range");range.put("from",((Number)call(inspected,"zeN")).doubleValue());range.put("to",((Number)call(inspected,"cSc")).doubleValue());}
        }
        result.put("revision",scope.observe(result));return result;
    }
    private JsonNode member(Object target)throws Exception {
        if(type("fOD").isInstance(target)) {
            Object clip=call(target,"YXc"),note=call(target,"v33");
            Object track=ancestor(clip,"com.bitwig.flt.document.core.master.kkr");
            if(track==null)throw new IllegalArgumentException("Selected note has no track");
            ObjectNode path=Launcher61.clipPath(document,track,clip);
            path.putObject("item").put("kind","note").put("id",id(note));
            return path;
        }
        if(type("nCh").isInstance(target) || type("Oss").isInstance(target))return automationPoint(target);
        return pathFor(target);
    }
    private Object ancestor(Object target,String name)throws Exception {
        return Reflection.call(target,"EwU",new Class<?>[]{Class.class},type(name));
    }
    private ObjectNode automationPoint(Object point)throws Exception {
        Object timeline=ancestor(point,"BAd");
        Object source=ancestor(point,"US");
        if(timeline==null && source!=null)timeline=call(source,"DQP");
        if(timeline==null)throw new IllegalArgumentException("Selected automation point has no timeline");
        Object arrangement=call(timeline,"Vdp");
        Object lane=arrangement==null?null:ancestor(arrangement,"dsv");
        if(lane==null)lane=ancestor(point,"dsv");
        if(lane==null)throw new IllegalArgumentException("Selected automation point has no lane");
        ObjectNode path=(ObjectNode)pathFor(lane);
        ObjectNode item=path.putObject("item").put("kind","automationPoint").put("id",id(point));
        if(arrangement!=null && timeline==value(arrangement,"outside_timeline")) {
            item.put("source",id(timeline)).put("placement",id(timeline));
        } else {
            Object clip=ancestor(point,"I3W");
            if(clip!=null) {
                item.put("source",id(call(clip,"dxV"))).put("placement",id(clip));
            } else if(source!=null) {
                item.put("source",id(source));
            } else throw new IllegalArgumentException("Selected automation point has no source");
        }
        return path;
    }
    private JsonNode pathFor(Object target)throws Exception {
        if(type("com.bitwig.flt.document.core.master.kkr").isInstance(target) || type("com.bitwig.flt.document.core.master.stI").isInstance(target))return ProjectAddress61.nativeTrackPath(document,target);
        Object track=Reflection.call(target,"EwU",new Class<?>[]{Class.class},type("com.bitwig.flt.document.core.master.kkr"));
        if(track==null)throw new IllegalArgumentException("Unsupported native selection member: "+target.getClass().getName());
        ObjectNode owner=ProjectAddress61.nativeTrackPath(document,track);
        if(target==value(track,"main_note_clip_lane"))return owner;
        String targetId=id(target);
        if(type("UMj").isInstance(target)) {
            owner.withArray("elements").addObject().put("kind","clip").put("name",value(target,"displayed_title").toString()).put("nativeId",targetId);return owner;
        }
        if(Entity61.automationLanes(track).contains(target)) {
            owner.withArray("elements").addObject().put("kind","automationLane").put("name",Entity61.laneName(target)).put("nativeId",targetId);return owner;
        }
        String trackId=id(track);
        if(!knownPaths.containsKey(trackId))index(Wire.parse(new Entity61(document).read(owner)));
        JsonNode found=knownPaths.get(targetId);
        if(found==null)throw new IllegalArgumentException("Selected member is outside supported containment: "+target.getClass().getName());
        return found;
    }
    private void index(JsonNode node) {
        knownPaths.put(node.required("metadata").required("id").asText(),node.required("path"));
        for(JsonNode child:node.required("children"))index(child);
    }
}
