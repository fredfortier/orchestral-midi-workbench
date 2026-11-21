package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicReference;

/** Qualified layer/slot execution and evidence. No project export or loader orchestration. */
public final class Instrument61 {
    private static final String LAYER="5024be2e-65d6-4d40-bbfe-8b2ea993c445";
    private static final String SELECTOR="9588fbcf-721a-438b-8555-97e4231f7d2c";
    private static final String KONTAKT="vst3-5653544E694B376B6F6E74616B742037";
    private final Map<String,Entry> operations=new LinkedHashMap<>();
    private static final class Entry {
        final ObjectNode input;
        final DocumentAccess continuation;
        final AtomicReference<String> phase=new AtomicReference<>("pending");
        volatile String error;
        boolean dispatched;
        ObjectNode result;
        Entry(ObjectNode input,DocumentAccess continuation){this.input=input.deepCopy();this.continuation=continuation;}
    }
    private record Scope(Object document,Object track,Object parent,Object layer,Object container,Object slot,ObjectNode target,ObjectNode observation) {}

    void invalidate() {
        for(Entry entry:operations.values())if(entry.result==null){entry.error="Document lifetime changed";entry.phase.set("unknown");}
    }
    void requireIdle() {
        for(var entry:operations.entrySet())if(entry.getValue().result==null)throw new IllegalStateException("Unresolved instrument operation "+entry.getKey()+"; query original status");
    }
    public String run(Object proxy,EditorAdapter61 editor,DocumentAccess continuation,ObjectNode request)throws Exception {
        try{return execute(proxy,editor,continuation,request);}
        catch(Exception failure) {
            if(request.hasNonNull("operation") && operations.containsKey(request.required("operation").asText()))throw failure;
            ObjectNode rejected=Wire.object();rejected.put("status","rejected");rejected.put("error",failure.toString());
            if(request.hasNonNull("operation"))rejected.set("operation",request.required("operation"));
            return rejected.toString();
        }
    }
    private String execute(Object proxy,EditorAdapter61 editor,DocumentAccess continuation,ObjectNode request)throws Exception {
        HostBuild.loaded(proxy).requireVerified();
        if(!Boolean.TRUE.equals(Reflection.call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong document thread");
        String action=request.required("action").asText();
        if(action.equals("status"))return status(proxy,editor,request.required("operation").asText(),false).toString();
        if(action.equals("read"))return scope(proxy,editor,request.required("target")).observation().toString();
        if(!Set.of("ensureLayer","ensureSlot","deleteSlot","assignReceiver","openEditor","closeEditor").contains(action))throw new IllegalArgumentException("Unknown scoped instrument action");
        String operation=request.required("operation").asText();
        Entry old=operations.get(operation);
        if(old!=null){if(!old.input.equals(request))throw new IllegalArgumentException("Conflicting instrument operation ID");return status(proxy,editor,operation,false).toString();}
        for(Entry entry:operations.values())if(entry.result==null)throw new IllegalStateException("Unresolved native instrument operation; query its original ID");
        if(operations.size()>=4096)throw new IllegalStateException("Instrument receipt capacity reached; no mutation dispatched");
        Scope scope=scope(proxy,editor,request.required("target"));
        if(!scope.observation().required("document").equals(request.required("document")) || !scope.observation().equals(request.required("expected"))) {
            List<String> changed=new ArrayList<>();
            scope.observation().fieldNames().forEachRemaining(field->{if(!scope.observation().get(field).equals(request.required("expected").get(field)))changed.add(field);});
            throw new IllegalStateException("Stale instrument scope; no mutation dispatched; changed fields: "+changed);
        }
        requireStopped(scope.document());
        if(!action.equals("ensureLayer") && scope.layer()==null)throw new IllegalArgumentException("Named instrument container is absent");
        if(!Set.of("ensureLayer","ensureSlot").contains(action) && scope.slot()==null)throw new IllegalArgumentException("Named slot is absent");
        if(action.equals("ensureSlot")) {
            String name=request.required("name").asText();
            if(name.isBlank() || name.length()>256 || name.indexOf(0)>=0)throw new IllegalArgumentException("Musical slot name required");
            JsonNode channels=request.required("inputChannels");
            Set<Integer> unique=new java.util.HashSet<>();
            if(!channels.isArray() || channels.isEmpty() || channels.size()>16)throw new IllegalArgumentException("Complete input channel set required");
            for(JsonNode c:channels)if(!c.isIntegralNumber() || c.intValue()<0 || c.intValue()>15 || !unique.add(c.intValue()))throw new IllegalArgumentException("Distinct native input channels must be 0..15");
            if(scope.slot()==null || !name.equals(value(scope.slot(),"displayed_title")))
                for(Object slot:slots(scope.container()))if(slot!=scope.slot() && name.equals(value(slot,"displayed_title")))throw new IllegalArgumentException("Slot name already exists in this container");
        }
        if(action.equals("assignReceiver"))receiver(scope.slot(),false);
        if(Set.of("openEditor","closeEditor").contains(action))receiver(scope.slot(),true);
        if(action.equals("ensureSlot") && scope.slot()==null && slots(scope.container()).size()>=128)throw new IllegalArgumentException("Container slot capacity reached");
        SlotRouting61.Prepared routing=action.equals("ensureSlot")?SlotRouting61.prepare(scope.slot(),proxy.getClass().getClassLoader()):null;
        Entry entry=new Entry(request,continuation);operations.put(operation,entry);
        try {
            if(action.equals("ensureLayer")) {ensureLayer(scope);entry.phase.set("completed");}
            else if(action.equals("assignReceiver") && receiver(scope.slot(),false)==null)insert(proxy,editor,continuation,scope,entry,scope.slot());
            else if(action.equals("deleteSlot")) {editSlot(scope,request,routing);entry.phase.set("deleting");}
            else {
                if(action.equals("ensureSlot"))editSlot(scope,request,routing);
                if(action.equals("openEditor") || action.equals("closeEditor")) {
                    editor(proxy,scope.slot(),action.equals("openEditor"));entry.phase.set("visibility");
                }else entry.phase.set("completed");
            }
        } catch(Exception error) {if(!operations.containsKey(operation))throw error;entry.error=error.toString();entry.phase.set("unknown");}
        return status(proxy,editor,operation,true).toString();
    }
    private ObjectNode status(Object proxy,EditorAdapter61 editor,String operation,boolean finish)throws Exception {
        Entry entry=operations.get(operation);
        if(entry==null)throw new IllegalArgumentException("Unknown instrument operation; do not replay");
        if(entry.result!=null)return entry.result.deepCopy();
        String phase=entry.phase.get();
        ObjectNode result=Wire.object();result.put("operation",operation);result.put("status",phase);result.set("document",entry.input.required("document"));
        if(entry.error!=null)result.put("error",entry.error);
        if(phase.equals("pending") || phase.equals("unknown") || phase.equals("failed") || (phase.equals("inserted") && !finish)){if(phase.equals("inserted"))result.put("status","pending");return result;}
        if(!editor.currentProjectGeneration().equals(entry.input.required("document").asText())){entry.error="Document lifetime changed";entry.phase.set("unknown");result.put("status","unknown");result.put("error",entry.error);return result;}
        try {
            String action=entry.input.required("action").asText();
            requireStopped(Reflection.call(proxy,"getDocument"));
            ObjectNode target=(ObjectNode)entry.input.required("target").deepCopy();
            Object track=resolvedTrack(proxy,entry);
            String display=value(track,"displayed_title").toString();
            for(JsonNode element:target.required("elements"))if(element.path("kind").asText().equals("track") && !element.required("name").asText().equals(display)) {
                if(!value(track,"title").toString().isEmpty() || !entry.input.required("expected").required("trackTitle").asText().isEmpty())throw new IllegalStateException("Authored track name changed during native operation");
                result.putObject("automaticTrackName").set("before",element.required("name"));
                ((ObjectNode)result.required("automaticTrackName")).put("after",display);
                ((ObjectNode)element).put("name",display);
            }
            if(action.equals("ensureSlot")) {
                JsonNode elements=target.required("elements");
                ((ObjectNode)elements.get(elements.size()-1)).set("name",entry.input.required("name"));
            }
            Scope current=scope(proxy,editor,target,action.equals("deleteSlot"));
            if(phase.equals("deleting")) {
                String deleted=entry.input.required("expected").required("slot").required("id").asText();
                for(JsonNode slot:current.observation().required("slots"))if(slot.required("id").asText().equals(deleted)){result.put("status","pending");return result;}
            }
            ObjectNode after=current.observation();
            if(phase.equals("visibility") && device(receiver(current.slot(),true)).required("editorVisible").asBoolean()!=action.equals("openEditor")) {
                result.put("status","pending");return result;
            }
            if(action.equals("assignReceiver")) {
                if(current.slot()==null)throw new IllegalStateException("Assigned receiver is absent");
                receiver(current.slot(),true);
            }
            if(action.equals("ensureLayer") && current.layer()==null)throw new IllegalStateException("Named layer was not materialized");
            if(action.equals("ensureSlot")) {
                if(current.slot()==null)throw new IllegalStateException("Named slot readback absent");
                if(((List<?>)Reflection.call(trackSelection(current.document()),"jaQ")).size()!=1 || ((List<?>)Reflection.call(slotSelection(current.container()),"jaQ")).size()!=1 || !id(current.slot()).equals(after.path("selectedSlot").asText()))throw new IllegalStateException("Resulting slot is not selected");
                verifyRouting(entry.input,after.required("slot"));
            }
            preservation(entry,after,action);
            result.put("status","completed");result.set("observation",after);result.set("document",after.required("document"));entry.result=result.deepCopy();return result;
        } catch(Exception error){entry.error=error.toString();entry.phase.set("unknown");result.put("status","unknown");result.put("error",entry.error);return result;}
    }
    private static void preservation(Entry entry,ObjectNode after,String action) {
        ObjectNode expected=(ObjectNode)entry.input.required("expected").deepCopy();
        expected.set("target",after.required("target"));
        var beforeSlots=(com.fasterxml.jackson.databind.node.ArrayNode)expected.required("slots");
        JsonNode original=expected.get("slot");String selected=original==null || original.isNull()?null:original.required("id").asText();
        if(action.equals("ensureLayer") && !expected.required("layerPresent").asBoolean()) {
            var devices=(com.fasterxml.jackson.databind.node.ArrayNode)expected.required("parentDevices");JsonNode actual=after.required("parentDevices");
            if(actual.size()!=devices.size()+1 || !actual.get(actual.size()-1).required("id").equals(after.required("layerId")))throw new IllegalStateException("Unexpected layer insertion membership");
            devices.add(actual.get(actual.size()-1));if(expected.required("parentId").equals(expected.required("trackId")))expected.set("trackDevices",devices.deepCopy());expected.set("layerId",after.required("layerId"));expected.put("layerPresent",true);
        }
        if(action.equals("ensureSlot") && selected==null) {
            if(after.required("slot").isNull())throw new IllegalStateException("Created slot absent");
            beforeSlots.add(after.required("slot"));expected.set("slot",after.required("slot"));
        }
        if(selected!=null) {
            if(action.equals("deleteSlot")) {
                for(int i=beforeSlots.size()-1;i>=0;i--)if(beforeSlots.get(i).required("id").asText().equals(selected))beforeSlots.remove(i);
                expected.putNull("slot");
            }else{
                ObjectNode slot=(ObjectNode)expected.required("slot");
                if(action.equals("ensureSlot")) {
                    slot.set("name",entry.input.required("name"));slot.put("channel",-1);slot.put("outputChannel",-1);
                    JsonNode actual=after.required("slot").required("devices");
                    List<JsonNode> beforeOther=new ArrayList<>(),afterOther=new ArrayList<>();
                    for(JsonNode d:slot.required("devices"))if(!SlotRouting61.helper(d.path("nativeDeviceId").asText()))beforeOther.add(d);
                    for(JsonNode d:actual)if(!SlotRouting61.helper(d.path("nativeDeviceId").asText()))afterOther.add(d);
                    if(!beforeOther.equals(afterOther))throw new IllegalStateException("Slot ensure changed nonrouting devices");
                    for(JsonNode d:slot.required("devices"))if(SlotRouting61.helper(d.path("nativeDeviceId").asText())) {
                        ObjectNode old=(ObjectNode)d.deepCopy(),next=null;
                        for(JsonNode candidate:actual)if(candidate.required("id").equals(d.required("id")))next=(ObjectNode)candidate.deepCopy();
                        if(next==null)throw new IllegalStateException("Existing routing helper was replaced");
                        for(String key:List.of("channelFilter","channelMap","routingEnabled","routingActive")){old.remove(key);next.remove(key);}
                        if(!old.equals(next))throw new IllegalStateException("Routing helper identity changed");
                    }
                    slot.set("devices",actual);
                }
                if(action.equals("assignReceiver")) {
                    JsonNode actual=after.required("slot").required("devices"),before=slot.required("devices");
                    if(actual.size()!=before.size() && actual.size()!=before.size()+1)throw new IllegalStateException("Unexpected instrument membership");
                    for(int i=0;i<before.size();i++)if(!before.get(i).equals(actual.get(i)))throw new IllegalStateException("Existing devices changed during assignment");
                    slot.set("devices",actual);
                }
                if(action.equals("openEditor") || action.equals("closeEditor")) {
                    JsonNode actual=after.required("slot").required("devices");
                    for(int i=0;i<slot.required("devices").size();i++) {
                        ObjectNode d=(ObjectNode)slot.required("devices").get(i);
                        if(KONTAKT.equals(d.path("pluginId").asText()))d.put("editorVisible",action.equals("openEditor"));
                    }
                }
                for(int i=0;i<beforeSlots.size();i++)if(beforeSlots.get(i).required("id").asText().equals(selected))beforeSlots.set(i,slot.deepCopy());
            }
        }
        if(action.equals("ensureSlot") || (action.equals("deleteSlot") && selected!=null && expected.path("selectedSlot").asText().equals(selected)))expected.set("selectedSlot",after.required("selectedSlot"));
        if((action.equals("openEditor") || action.equals("assignReceiver")) && after.path("selectedSlot").asText().equals(selected))expected.set("selectedSlot",after.required("selectedSlot"));
        if(!expected.equals(after))throw new IllegalStateException("Independent instrument readback changed untouched identity, membership, order or routing");
    }
    private static Object resolvedTrack(Object proxy,Entry entry)throws Exception {
        Object document=Reflection.call(proxy,"getDocument");
        Object track=findTrack(Reflection.call(document,"peG"),entry.input.required("expected").required("trackId").asText(),proxy.getClass().getClassLoader(),0);
        if(track==null)throw new IllegalStateException("Resolved track no longer belongs to this document");
        return track;
    }
    private static Scope scope(Object proxy,EditorAdapter61 editor,JsonNode target)throws Exception {
        return scope(proxy,editor,target,false);
    }
    private static Scope scope(Object proxy,EditorAdapter61 editor,JsonNode target,boolean deletedSlotMayBeAbsent)throws Exception {
        Object document=Reflection.call(proxy,"getDocument");if(document==null)throw new IllegalStateException("No current project");
        ObjectNode trackPath=ProjectAddress61.trackPath(target);Object track=ProjectAddress61.resolveNative(trackPath,document,proxy.getClass().getClassLoader());
        JsonNode elements=target.required("elements");
        String last=elements.get(elements.size()-1).required("kind").asText();
        boolean slotTarget=last.equals("layerChain") || last.equals("choice");
        int index=elements.size()-(slotTarget?2:1);JsonNode layerElement=elements.get(index);
        String kind=layerElement.required("kind").asText();
        if(!kind.equals("layer") && !kind.equals("selector"))throw new IllegalArgumentException("Instrument container target required");
        boolean selector=kind.equals("selector");String layerName=layerElement.required("name").asText();
        JsonNode slotElement=slotTarget?elements.get(index+1):null;String slotName=slotTarget?slotElement.required("name").asText():null;
        ObjectNode parentPath=(ObjectNode)target.deepCopy();var parentElements=parentPath.putArray("elements");for(int i=0;i<index;i++)parentElements.add(elements.get(i));
        Object parent=ProjectAddress61.resolveNative(parentPath,document,proxy.getClass().getClassLoader());
        Object layer=null;
        for(Object device:chain(parent))if(layerName.equals(deviceName(device)) && (!layerElement.has("nativeId") || layerElement.required("nativeId").asText().equals(value(device,"managed_object_ID").toString()))) {
            if(layer!=null)throw new IllegalStateException("Ambiguous named container");
            if(!(selector?SELECTOR:LAYER).equals(Reflection.call(device,"v33").toString()))throw new IllegalStateException("Named device is not the requested instrument container");layer=device;
        }
        if(layer==null && layerElement.has("nativeId"))throw new IllegalArgumentException("Stale instrument container identity");
        ObjectNode observation=Wire.object();observation.set("target",target.deepCopy());observation.put("document",editor.currentProjectGeneration());
        observation.put("hostPid",Math.toIntExact(ProcessHandle.current().pid()));observation.put("trackTitle",value(track,"title").toString());observation.put("projectUuid",id(document));observation.put("trackId",id(track));observation.put("engine",((Enum<?>)Reflection.call(document,"T1L")).name());
        var trackDevices=observation.putArray("trackDevices");for(Object device:chain(track))trackDevices.add(device(device));
        observation.put("parentId",id(parent));var parentDevices=observation.putArray("parentDevices");for(Object device:chain(parent))parentDevices.add(device(device));
        Object container=layer==null?null:Reflection.call(Reflection.call(layer,"AtZ"),"v33",new Class<?>[]{String.class},selector?"CHAIN_SELECTOR":"CHAIN_LIST"),selected=null;
        if(layer==null || !editorPathActive(document,track,target,index))observation.putNull("selectedSlot");else {
            Object selectedChain=Reflection.call(slotSelection(container),"UAA");
            if(selectedChain==null || !soleSelection(slotSelection(container),selectedChain) || !deviceSelectionWithin(layer,selectedChain))observation.putNull("selectedSlot");else observation.put("selectedSlot",id(selectedChain));
        }
        observation.put("layerPresent",layer!=null);if(layer==null)observation.putNull("layerId");else observation.put("layerId",id(layer));
        var slots=observation.putArray("slots");
        if(container!=null)for(Object slot:slots(container)) {
            ObjectNode row=slot(slot);slots.add(row);
            if(slotName!=null && slotName.equals(value(slot,"displayed_title")) && (!slotElement.has("nativeId") || slotElement.required("nativeId").asText().equals(value(slot,"managed_object_ID").toString()))) {if(selected!=null)throw new IllegalStateException("Ambiguous named slot");selected=slot;}
        }
        if(selected==null && slotElement!=null && slotElement.has("nativeId") && !deletedSlotMayBeAbsent)throw new IllegalArgumentException("Stale instrument slot identity");
        if(selected==null)observation.putNull("slot");else observation.set("slot",slot(selected));
        Wire.requireMessageCapacity(observation);
        return new Scope(document,track,parent,layer,container,selected,(ObjectNode)target.deepCopy(),observation);
    }
    private static ObjectNode slot(Object slot)throws Exception {
        ObjectNode result=Wire.object();result.put("id",id(slot));result.put("name",value(slot,"displayed_title").toString());
        result.put("channel",((Number)value(slot,"filter_note_channel")).intValue());result.put("outputChannel",((Number)value(slot,"override_note_channel")).intValue());
        var devices=result.putArray("devices");Set<String> unique=new java.util.HashSet<>();
        for(Object device:chain(slot)){if(!unique.add(id(device)))throw new IllegalStateException("Duplicate native device membership");devices.add(device(device));}return result;
    }
    private static ObjectNode device(Object device)throws Exception {
        ObjectNode result=Wire.object();result.put("id",id(device));result.put("name",deviceName(device));
        result.put("nativeDeviceId",Reflection.call(device,"v33").toString());
        Object contents=value(device,"device_contents");
        PluginIdentity61 plugin=PluginIdentity61.read(contents);
        if(plugin!=null) {
            result.put("pluginId",plugin.id());result.put("version",plugin.version());
            result.put("editorVisible",Boolean.TRUE.equals(Reflection.call(Reflection.call(contents,"cfn"),"getBoolValue")));
        }
        SlotRouting61.read(device,result);
        return result;
    }
    private static String deviceName(Object device)throws Exception {String name=value(device,"user_device_name").toString();return name.isEmpty()?value(device,"device_name").toString():name;}
    private static List<?> chain(Object owner)throws Exception {
        List<?> devices=(List<?>)Reflection.call(Reflection.call(owner,"PPp"),"ATl");if(devices.size()>256)throw new IllegalStateException("Instrument chain exceeds 256 devices");return devices;
    }
    private static List<?> slots(Object container)throws Exception {List<?> slots=(List<?>)value(container,"nested_device_chains");if(slots.size()>128)throw new IllegalStateException("Instrument layer exceeds 128 slots");return slots;}
    private static Object value(Object owner,String key)throws Exception{return ProjectAdapter61.value(owner,key);}
    private static String id(Object owner)throws Exception{return value(owner,"UUID").toString();}
    private static void requireStopped(Object document)throws Exception {
        for(String field:List.of("is_playing","is_playing_on_engine","record","write_automation"))if(!Boolean.FALSE.equals(value(value(document,"transport"),field)))throw new IllegalStateException("Stopped nonrecording transport required");
        if(Boolean.TRUE.equals(value(document,"is_being_saved")) || Reflection.call(document,"jst")!=null)throw new IllegalStateException("Save/render in flight");
    }
    private void insert(Object proxy,EditorAdapter61 editor,DocumentAccess continuation,Scope scope,Entry entry,Object owner)throws Exception {
        if(continuation==null)throw new IllegalStateException("Native instrument continuation dispatcher unavailable");
        NativeInsertion prepared;
        try {prepared=prepareInsertion(proxy,owner);}
        catch(Exception failure){if(!entry.dispatched)operations.remove(entry.input.required("operation").asText());throw failure;}
        Class<?> callbackType=Class.forName("com.bitwig.base.async.Tl3",true,proxy.getClass().getClassLoader());
        Object callback=Proxy.newProxyInstance(callbackType.getClassLoader(),new Class<?>[]{callbackType},(receiver,method,args)-> {
            switch(method.getName()) {
                case "hashCode":return System.identityHashCode(receiver);
                case "equals":return receiver==args[0];
                case "toString":return "Scoped instrument insertion completion";
                default:
                    // Completion is dispatched back onto the document thread.
                    if(method.getName().equals("taskCompleted") || (method.getName().equals("v33") && method.getParameterTypes()[0]==Object.class)) {
                        entry.phase.set("inserted");
                        continuation.submit(()->status(proxy,editor,entry.input.required("operation").asText(),true))
                            .whenComplete((result,error)->{if(error!=null){entry.error=error.toString();entry.phase.set("unknown");}});
                    }
                    else {entry.error="Native insertion callback: "+method.getName();entry.phase.set("unknown");}
                    return null;
            }
        });
        entry.phase.set("pending");
        entry.dispatched=true;
        if(!applyInsertion(proxy,prepared,callback)){entry.error="Native insertion refused";entry.phase.set("failed");}
    }
    /** Only the exact installed Kontakt plug-in; stock routing uses its native factory. */
    private record NativeInsertion(Object point,Object data,Object options) {}
    private static NativeInsertion prepareInsertion(Object proxy,Object owner)throws Exception {
        ClassLoader loader=proxy.getClass().getClassLoader();
        Object app=type(loader,"com.bitwig.flt.app.iOF").getMethod("d3J").invoke(null),data;
            Object plugin=null;
            for(Object candidate:(List<?>)Reflection.call(app,"IVb")) {
                if(!KONTAKT.equals(Reflection.call(Reflection.call(candidate,"FSQ"),"v33").toString())
                        || !"7.10.1".equals(value(candidate,"device_version")))continue;
                java.nio.file.attribute.BasicFileAttributes backing;
                try {backing=java.nio.file.Files.readAttributes(((java.io.File)Reflection.call(candidate,"Zlo")).toPath(),java.nio.file.attribute.BasicFileAttributes.class);}
                catch(java.nio.file.NoSuchFileException absent){continue;}
                if(!backing.isRegularFile() && !backing.isDirectory())throw new IllegalStateException("Kontakt backing path is not a file or bundle");
                if(plugin!=null)throw new IllegalStateException("Ambiguous installed Kontakt registry entry");
                plugin=candidate;
            }
            if(plugin==null)throw new IllegalStateException("Exact installed Kontakt version is unavailable");
            data=type(loader,"Elr").getConstructor(type(loader,"com.bitwig.base.io.bf")).newInstance(Reflection.call(plugin,"ij_"));
        Object chain=Reflection.call(owner,"PPp");
        Object point=type(loader,"com.bitwig.flt.document.core.iface.clipboard.device.DeviceChainInsertionPoint")
            .getConstructor(type(loader,"Sx"),int.class,type(loader,"UNB")).newInstance(chain,chain(owner).size(),null);
        Object options=Reflection.call(type(loader,"hFO").getField("v33").get(null),"v33",new Class<?>[]{type(loader,"hLp"),boolean.class},type(loader,"hLp").getField("TUp").get(null),true);
        return new NativeInsertion(point,data,options);
    }
    private static boolean applyInsertion(Object proxy,NativeInsertion prepared,Object callback)throws Exception {
        ClassLoader loader=proxy.getClass().getClassLoader();
        return (boolean)Reflection.call(prepared.point(),"v33",new Class<?>[]{type(loader,"oR3"),type(loader,"hFO"),type(loader,"com.bitwig.base.async.Tl3")},prepared.data(),prepared.options(),callback);
    }
    private static Class<?> type(ClassLoader loader,String name)throws ClassNotFoundException{return Class.forName(name,true,loader);}
    private static Object findTrack(Object root,String uuid,ClassLoader loader,int depth)throws Exception {
        if(depth>64)throw new IllegalStateException("Track hierarchy depth limit");
        for(Object track:(List<?>)Reflection.call(root,"IFc")) {
            if(uuid.equals(id(track)))return track;
            if(type(loader,"com.bitwig.flt.document.core.iface.RN").isInstance(track)) {
                Object found=findTrack(track,uuid,loader,depth+1);if(found!=null)return found;
            }
        }
        return null;
    }
    private static void editSlot(Scope scope,ObjectNode request,SlotRouting61.Prepared routing)throws Exception {
        String action=request.required("action").asText();Object slot=scope.slot();
        if(action.equals("ensureSlot") && slot==null && slots(scope.container()).size()>=128)throw new IllegalArgumentException("Layer slot capacity reached");
        ClassLoader loader=scope.document().getClass().getClassLoader();
        EditorAdapter61.requireIndependentUndo(scope.document());Reflection.call(scope.document(),"v33",new Class<?>[]{String.class,boolean.class},"Instrument slot "+action,false);
        try {
            if(action.equals("deleteSlot"))Reflection.call(scope.container(),"v33",new Class<?>[]{List.class},List.of(slot));
            else {
                if(slot==null) {
                    slot=Class.forName("com.bitwig.flt.document.core.master.device.kdt",true,loader).getMethod("QT").invoke(null);
                    Object mixer=Class.forName("com.bitwig.flt.document.core.master.device.wt1",true,loader).getMethod("NC3").invoke(null);
                    Reflection.call(slot,"v33",new Class<?>[]{Class.forName("Ee1",true,loader)},mixer);
                    JsonNode elements=scope.target().required("elements");Reflection.call(slot,"d_",new Class<?>[]{String.class},elements.get(elements.size()-1).required("name").asText());
                }
                if(request.hasNonNull("name"))Reflection.call(slot,"d_",new Class<?>[]{String.class},request.required("name").asText());
                for(String key:List.of("filter_note_channel","override_note_channel"))Reflection.call(Reflection.call(slot,"Nln",new Class<?>[]{String.class},key),"setValue",new Class<?>[]{Object.class},-1);
                if(scope.slot()==null)Reflection.call(scope.container(),"Iz2",new Class<?>[]{Class.forName("com.bitwig.flt.document.core.master.device.kdt",true,loader)},slot);
                SlotRouting61.apply(slot,routing,request.required("inputChannels"));
                selectSlot(scope,slot);

            }
            Reflection.call(Reflection.call(Reflection.call(Reflection.call(Reflection.call(scope.document(),"cuK"),"TcB"),"v33"),"v33"),"LHs");
        }finally{Reflection.call(scope.document(),"v33",new Class<?>[]{boolean.class},false);}
    }
    private static void ensureLayer(Scope scope)throws Exception {
        if(scope.layer()!=null)return;
        JsonNode elements=scope.target().required("elements"),element=elements.get(elements.size()-1);
        Object device=SlotRouting61.stock(scope.document().getClass().getClassLoader(),element.path("kind").asText().equals("selector")?SELECTOR:LAYER);
        Reflection.call(device,"XOR",new Class<?>[]{String.class},element.required("name").asText());
        EditorAdapter61.requireIndependentUndo(scope.document());Reflection.call(scope.document(),"v33",new Class<?>[]{String.class,boolean.class},"Ensure instrument container",false);
        try {Reflection.call(Reflection.call(scope.parent(),"PPp"),"v33",new Class<?>[]{type(scope.document().getClass().getClassLoader(),"com.bitwig.flt.document.core.master.device.ysX"),int.class},device,chain(scope.parent()).size());}
        finally {Reflection.call(scope.document(),"v33",new Class<?>[]{boolean.class},false);}
    }
    static Object trackSelection(Object document)throws Exception {
        return Reflection.call(Reflection.call(document,"OOF"),"zeN");
    }
    static Object deviceSelection(Object device)throws Exception {
        ClassLoader loader=device.getClass().getClassLoader();
        Object owner=Reflection.call(device,"EwU",new Class<?>[]{Class.class},type(loader,"dmQ"));
        return Reflection.call(Reflection.call(owner,"QT"),"FSQ");
    }
    static void selectDevice(Object device)throws Exception {
        Reflection.call(deviceSelection(device),"qKA",new Class<?>[]{type(device.getClass().getClassLoader(),"com.bitwig.ramona.core.Zj3")},device);
    }
    private static boolean soleSelection(Object selection,Object target)throws Exception {
        List<?> selected=(List<?>)Reflection.call(selection,"jaQ");
        return selected.size()==1 && selected.getFirst().equals(target);
    }
    private static boolean deviceSelectionWithin(Object device,Object owner)throws Exception {
        List<?> selected=(List<?>)Reflection.call(deviceSelection(device),"jaQ");
        if(selected.size()!=1)return false;
        Object active=selected.getFirst();
        // DeviceProxy's flattened selection can point at an instrument inside
        // the container. Zj3.XQF follows native M_ ownership, not UI memory.
        return active.equals(device) || Boolean.TRUE.equals(Reflection.call(active,"XQF",new Class<?>[]{type(device.getClass().getClassLoader(),"com.bitwig.ramona.core.Zj3")},owner));
    }
    private static boolean editorPathActive(Object document,Object track,JsonNode target,int containerIndex)throws Exception {
        if(!soleSelection(trackSelection(document),track))return false;
        ClassLoader loader=document.getClass().getClassLoader();
        ObjectNode path=ProjectAddress61.nativeTrackPath(document,track);
        JsonNode elements=target.required("elements");
        // Check the same editor selections that selectSlot writes, including
        // every enclosing container/chain and the addressed container itself.
        for(int i=path.required("elements").size();i<=containerIndex;i++) {
            ((com.fasterxml.jackson.databind.node.ArrayNode)path.required("elements")).add(elements.get(i));
            Object ancestor=ProjectAddress61.resolveNative(path,document,loader);
            String kind=elements.get(i).required("kind").asText();
            if(kind.equals("layer") || kind.equals("selector")) {
                if(!deviceSelectionWithin(ancestor,ancestor))return false;
            }else if(!soleSelection(slotSelection(Reflection.call(ancestor,"Iks")),ancestor))return false;
        }
        return true;
    }
    private static void selectSlot(Scope scope,Object slot)throws Exception {
        selectPath(scope.document(),scope.target());
        Class<?> object=type(scope.document().getClass().getClassLoader(),"com.bitwig.ramona.core.Zj3");
        Reflection.call(slotSelection(scope.container()),"qKA",new Class<?>[]{object},slot);
        Object settings=value(scope.container(),"display_settings");
        Reflection.call(Reflection.call(settings,"Nln",new Class<?>[]{String.class},"should_show_cursor_chain_contents"),"setValue",new Class<?>[]{Object.class},true);
    }
    record SelectionStep(Object selection,Object target) {
        void apply()throws Exception {Reflection.call(selection,"qKA",new Class<?>[]{type(target.getClass().getClassLoader(),"com.bitwig.ramona.core.Zj3")},target);}
    }
    static List<SelectionStep> prepareSelectionPath(Object document,JsonNode target)throws Exception {
        ClassLoader loader=document.getClass().getClassLoader();Class<?> object=type(loader,"com.bitwig.ramona.core.Zj3");
        List<SelectionStep> steps=new java.util.ArrayList<>();
        ObjectNode path=(ObjectNode)target.deepCopy();path.putArray("elements");
        for(JsonNode element:target.required("elements")) {
            path.withArray("elements").add(element);
            Object ancestor=ProjectAddress61.resolveNative(path,document,loader);
            Object selection=switch(element.required("kind").asText()) {
                case "group","track" -> trackSelection(document);
                case "layerChain","choice" -> slotSelection(Reflection.call(ancestor,"Iks"));
                default -> deviceSelection(ancestor);
            };
            Reflection.method(selection.getClass(),"qKA",object);steps.add(new SelectionStep(selection,ancestor));
        }
        return steps;
    }
    static void selectPath(Object document,JsonNode target)throws Exception {
        for(SelectionStep step:prepareSelectionPath(document,target))step.apply();
    }
    static Object slotSelection(Object container)throws Exception {
        // Same owner as NestedDeviceChainProxy.getEditorChannelSelection;
        // AbstractChannelProxy.doSelectInEditor calls qKA(Zj3) on this selection.
        return value(value(container,"display_settings"),"selection");
    }
    private static void verifyRouting(ObjectNode input,JsonNode slot) {
        if(slot.path("channel").asInt()!=-1 || slot.path("outputChannel").asInt()!=-1 || !slot.required("name").equals(input.required("name")))throw new IllegalStateException("Slot name or All/Same routing differs");
        JsonNode devices=slot.required("devices");
        if(devices.size()<2 || !devices.get(0).path("nativeDeviceId").asText().equals(SlotRouting61.FILTER) || !devices.get(1).path("nativeDeviceId").asText().equals(SlotRouting61.MAP))throw new IllegalStateException("Filter/Map prefix differs");
        Set<Integer> enabled=new java.util.HashSet<>();enabled.add(0);for(JsonNode c:input.required("inputChannels"))enabled.add(c.intValue());
        for(int c=0;c<16;c++)if(devices.get(0).required("channelFilter").get(c).asBoolean()!=enabled.contains(c) || devices.get(1).required("channelMap").get(c).asInt()!=1)throw new IllegalStateException("Channel device readback differs");
        if(!devices.get(0).required("routingEnabled").asBoolean() || !devices.get(1).required("routingEnabled").asBoolean() || !devices.get(0).required("routingActive").asBoolean() || !devices.get(1).required("routingActive").asBoolean())throw new IllegalStateException("Channel devices disabled");
    }
    private static Object receiver(Object slot,boolean required)throws Exception {
        Object found=null;
        for(Object device:chain(slot))if(((Enum<?>)Reflection.call(device,"Nh")).name().equals("NOTE_TO_AUDIO")) {
            if(found!=null)throw new IllegalArgumentException("Multiple instruments in slot; precise instrument scope is ambiguous");found=device;
        }
        if(found==null){if(required)throw new IllegalArgumentException("Slot has no instrument");return null;}
        Object contents=value(found,"device_contents");
        PluginIdentity61 plugin=PluginIdentity61.read(contents);
        if(plugin==null || !KONTAKT.equals(plugin.id()))throw new IllegalArgumentException("Existing instrument is not Kontakt");
        return found;
    }
    private static void editor(Object proxy,Object slot,boolean visible)throws Exception {
        Object device=receiver(slot,true),contents=value(device,"device_contents"),window=Reflection.call(contents,"cfn");
        if(visible) {
            selectDevice(device);
        }
        Reflection.call(window,"setBoolValue",new Class<?>[]{boolean.class},visible);
    }
}
