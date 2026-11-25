package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Base64;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.UUID;

/** Build-specific document operations. Every entry executes on the document thread. */
public final class EditorAdapter61 implements AutoCloseable {
    private static final String[] EXPRESSIONS = {"PRESSURE", "TRANSPOSE", "GAIN", "PAN", "TIMBRE", "FORMANT"};
    private static final String[] WIRE_EXPRESSIONS = {"pressure", "pitch", "gain", "pan", "timbre", "formant"};
    private static final Set<String> GEOMETRY = Set.of("channel","key","start","duration");
    private static final Map<String,String> SCALARS = Map.of(
            "muted", "is_muted",
            "velocity", "note_on_velocity", "releaseVelocity", "note_off_velocity", "velocitySpread", "velocity_spread");
    private final Object proxy;
    private DocumentAccess continuation;
    private final Mix61 mixer=new Mix61();
    private final ExportAudio61 exports=new ExportAudio61();
    private final ProjectCues61 cues=new ProjectCues61();
    private final Instrument61 instruments=new Instrument61();
    private Object currentDocument;
    private String generation;
    private String frozenRevision;
    private ObjectNode frozenSnapshot;
    private JsonNode frozenPaths;
    private Object documentTarget;
    private Object documentListener;
    private boolean closed;
    private ProjectRevision61 nativeChanges;

    private record View(Object document, ObjectNode snapshot,
                        Map<String,Object> clips, Map<String,Object> notes, Map<String,String> payloads,
                        Map<String,AutomationAdapter61.Owner> curves, Set<String> arrangerTracks) {}

    public EditorAdapter61(Object proxy, HostBuild build) {
        build.requireVerified();
        this.proxy = proxy;
    }

    public EditorAdapter61(Object proxy,HostBuild build,DocumentAccess continuation){this(proxy,build);this.continuation=continuation;}
    public void initialize() throws Exception {observeTargetDocument();}

    /** Probe-owned document lifetime without serializing the complete project graph. */
    public String currentProjectGeneration() throws Exception {
        observeCurrentDocument();return generation;
    }

    public String mix(String request)throws Exception {
        exports.requireIdle();instruments.requireIdle();
        try{return mixer.run(proxy,this,Wire.parse(request));}
        catch(Exception failure){if(!mixer.dispatched)throw new NativeRefusal(failure);throw failure;}
    }
    public String exportAudio(String request)throws Exception {
        instruments.requireIdle();ObjectNode input=Wire.parse(request);
        try{return exports.run(proxy,this,mixer,input);}
        catch(Exception failure){if(!exports.contains(input.path("operation").asText()))throw new NativeRefusal(failure);throw failure;}
    }
    public String cues(String request)throws Exception {
        observeCurrentDocument();ObjectNode input=Wire.parse(request);
        if(!input.required("action").asText().equals("read")){instruments.requireIdle();exports.requireIdle();}
        try{return cues.run(proxy,this,input);}
        catch(Exception failure){if(!cues.admitted(input.path("operation").asText()))throw new NativeRefusal(failure);throw failure;}
    }
    public String instrument(String request) throws Exception {
        exports.requireIdle();observeCurrentDocument();
        return instruments.run(proxy,this,continuation,Wire.parse(request));
    }
    private final ObservedScope selectionScope=new ObservedScope();
    public java.util.concurrent.CompletableFuture<String> selection(String request)throws Exception {return selection(request,()->true);}
    public java.util.concurrent.CompletableFuture<String> selection(String request,java.util.function.BooleanSupplier validRequest)throws Exception {
        Object document=observeCurrentDocument();exports.requireIdle();instruments.requireIdle();
        return new Selection61(proxy,document,generation,selectionScope,()->validRequest.getAsBoolean() && !closed && currentDocument==document).run(Wire.parse(request));
    }
    private final ObservedScope launcherScope=new ObservedScope();
    public ObjectNode launcher(ObjectNode request) throws Exception {
        Object document=observeCurrentDocument();
        Object slot=Launcher61.selectedSlot(Launcher61.selection(proxy));
        ObjectNode before=Launcher61.observe(document,generation,slot,launcherScope);
        if(request.size()==1 && request.path("action").asText().equals("read"))return before;
        try {
            if(!request.path("action").asText().equals("create"))throw new IllegalArgumentException("Unknown launcher action");
            Launcher61.preflight(request,before);
            exports.requireIdle();instruments.requireIdle();
            requireIdleForContentEdit(document);
            requireIndependentUndo(document);
        }catch(Exception failure){throw new NativeRefusal(failure);}
        int beats=request.required("beats").intValue();
        invoke(document,"v33",new Class<?>[]{String.class,boolean.class},"Create launcher clip",false);
        try {
            Object recorder=call(call(call(document,"cuK"),"TcB"),"v33");
            Object step=call(recorder,"qKA");
            try {
                Launcher61.create(slot,beats);
                call(call(recorder,"v33"),"LHs");
                Launcher61.verify(slot,beats);
                Object actualSlot=Launcher61.selectedSlot(Launcher61.selection(proxy));
                if(actualSlot!=slot)throw new IllegalStateException("Launcher selection changed during clip creation");
                return Launcher61.observe(document,generation,actualSlot,launcherScope);
            }catch(Exception failure) {
                reverseStep(recorder,step,failure);
                if(Launcher61.clip(slot)!=null || Launcher61.selectedSlot(Launcher61.selection(proxy))!=slot)
                    throw new IllegalStateException("Native recovery did not restore the empty launcher slot",failure);
                discardRecoveredStep(step);
                throw new NativeRefusal(failure);
            }
        } finally {
            invoke(document,"v33",new Class<?>[]{boolean.class},false);
        }
    }
    /** All reads use the same complete clip representation. */
    public String info(String path) throws Exception {return info(path,false);}
    public String info(String path,boolean parameters) throws Exception {
        Object document=observeCurrentDocument();
        ObjectNode entity=new Entity61(document).observe(Wire.JSON.readTree(path),parameters);
        entity.set("selection",new Selection61(proxy,document,generation,selectionScope,()->currentDocument==document).read());
        return entity.toString();
    }
    public String capture(String path,boolean includeInactive)throws Exception {return new MidiCapture61(observeCurrentDocument(),this).read(Wire.JSON.readTree(path),includeInactive);}
    public String readMidi() throws Exception {return readSelection();}
    public String readClips(String paths) throws Exception {return observe(Wire.JSON.readTree(paths)).snapshot().toString();}
    public String freezePaths(JsonNode paths) throws Exception {
        exports.requireIdle();instruments.requireIdle();frozenRevision=null;frozenPaths=paths.deepCopy();
        View view=observe(frozenPaths);requireIndependentUndo(view.document());requireIsolatedSources(view);
        frozenRevision=view.snapshot().required("revision").asText();frozenSnapshot=view.snapshot().deepCopy();return view.snapshot().toString();
    }
    public String readSelection() throws Exception { return observe().snapshot().toString(); }
    public String readAppliedSelection() throws Exception {try{return observe(frozenPaths).snapshot().toString();}finally{frozenPaths=null;}}

    public String freeze() throws Exception {
        instruments.requireIdle();exports.requireIdle();
        frozenRevision = null;frozenPaths=null;
        View view = observe();
        requireIndependentUndo(view.document());
        requireIsolatedSources(view);
        String wire = view.snapshot().toString();
        frozenRevision = view.snapshot().required("revision").asText();frozenSnapshot=view.snapshot().deepCopy();
        return wire;
    }

    /** Preflight, apply and independently observe one native document batch. */
    public String apply(String json) throws Exception {
        instruments.requireIdle();exports.requireIdle();
        String request = frozenRevision;ObjectNode frozen=frozenSnapshot;
        frozenRevision = null;frozenSnapshot=null;
        if (request == null) throw new IllegalStateException("No frozen UI selection");
        View live = observe(frozenPaths);
        if(frozen==null || !sameDocumentState(frozen,live.snapshot()))throw new IllegalArgumentException("Complete clip scope or selection changed before write");
        ObjectNode plan = Wire.parse(json);
        if (!plan.required("document").asText().equals(live.snapshot().required("document").asText())
                || !plan.required("revision").asText().equals(request))
            throw new IllegalStateException("Selection, source content or clip mapping changed before apply");
        requireIsolatedSources(live);
        Set<String> planFields=new HashSet<>(Set.of("document","revision","edits"));for(String field:List.of("automation","automationCreate","automationDelete"))if(plan.has(field))planFields.add(field);requireKeys(plan,planFields);
        ObjectNode expected = live.snapshot().deepCopy();
        Set<String> touched = new HashSet<>();
        for (JsonNode edit : plan.required("edits")) {
            String operation = edit.required("type").asText(), clip = edit.required("clip").asText();
            ObjectNode owner = find(expected.withArray("clips"),clip,"id");
            if (!live.clips().containsKey(clip)) throw new IllegalArgumentException("Owner is outside UI selection");
            String id = edit.required(operation.equals("create")?"temporary":"note").asText();
            if (!touched.add(clip+":"+id)) throw new IllegalArgumentException("Duplicate edit target");
            ArrayNode notes = owner.withArray("notes");
            switch(operation) {
                case "create" -> {
                    requireKeys(edit,Set.of("type","clip","temporary","note"));
                    ObjectNode note = (ObjectNode)edit.required("note").deepCopy();
                    validateNote(note);
                    for(String field:List.of("start","duration","contentStart"))note.put(field,canonicalTime(finite(note,field)));
                    for(JsonNode curve:note.required("expressions"))for(JsonNode point:curve)((ObjectNode)point).put("time",canonicalTime(finite(point,"time")));
                    validateNote(note);
                    if (!id.equals(note.required("id").asText()) || live.notes().containsKey(clip+":"+id)) throw new IllegalArgumentException("Invalid creation identity");
                    ObjectNode item = Wire.object();item.set("note",note);
                    item.put("selected",!expected.required("selection").required("kind").asText().equals("notes"));notes.add(item);
                }
                case "update", "delete" -> {
                    ObjectNode item = findNote(notes,id);
                    if (!item.required("selected").asBoolean()) throw new IllegalArgumentException("Note is outside UI selection");
                    if (operation.equals("delete")) {
                        requireKeys(edit,Set.of("type","clip","note"));
                        notes.remove(indexOf(notes,item));
                        ArrayNode objects=(ArrayNode)expected.required("selection").required("objects");
                        for(int i=objects.size()-1;i>=0;i--)if(objects.get(i).asText().equals(clip+":"+id))objects.remove(i);
                    } else {
                        requireKeys(edit,Set.of("type","clip","note","changes"));
                        ObjectNode note = (ObjectNode)item.required("note");
                        Set<String> fields = new HashSet<>();
                        if (!edit.required("changes").isArray() || edit.required("changes").isEmpty()) throw new IllegalArgumentException("Empty changes");
                        for(JsonNode change:edit.required("changes")) {
                            requireKeys(change,Set.of("field","value"));
                            String field = change.required("field").asText();
                            if (!fields.add(field) || !(SCALARS.containsKey(field) || GEOMETRY.contains(field) || Set.of("operators","expressions").contains(field)))
                                throw new IllegalArgumentException("Unsupported or repeated note field");
                            if(field.equals("expressions")) {
                                if(!change.required("value").isObject())throw new IllegalArgumentException("Expressions must be an object");
                                var curves=change.required("value").fields();
                                while(curves.hasNext()) {
                                    var entry=curves.next();ArrayNode requested=(ArrayNode)entry.getValue().deepCopy();
                                    JsonNode original=note.required("expressions").required(entry.getKey());
                                    for(JsonNode point:requested) {
                                        JsonNode old=null;for(JsonNode candidate:original)if(candidate.required("id").equals(point.required("id"))){old=candidate;break;}
                                        if(old==null || !old.required("time").equals(point.required("time")))
                                            ((ObjectNode)point).put("time",canonicalTime(finite(point,"time")));
                                    }
                                    ((ObjectNode)note.required("expressions")).set(entry.getKey(),requested);
                                }
                            } else {
                                note.set(field,change.required("value").deepCopy());
                                if(field.equals("start") || field.equals("duration"))note.put(field,canonicalTime(finite(note,field)));
                            }
                        }
                        validateNote(note);
                    }
                }
                default -> throw new IllegalArgumentException("Unknown CRUD operation");
            }
        }
        AutomationAdapter61 automation=new AutomationAdapter61(proxy);
        List<AutomationAdapter61.Prepared> curves=automation.prepare(live.snapshot(),live.curves(),plan.has("automation")?plan.required("automation"):Wire.JSON.createArrayNode());
        List<AutomationAdapter61.Creation> createdCurves=automation.prepareCreates(live.snapshot(),live.clips(),live.arrangerTracks(),plan.has("automationCreate")?plan.required("automationCreate"):Wire.JSON.createArrayNode());
        List<AutomationAdapter61.Owner> deletedCurves=automation.prepareDeletes(live.snapshot(),live.curves(),plan.has("automationDelete")?plan.required("automationDelete"):Wire.JSON.createArrayNode());
        if(touched.isEmpty() && curves.isEmpty() && createdCurves.isEmpty() && deletedCurves.isEmpty())throw new IllegalArgumentException("Empty edit plan");
        requireIdleForContentEdit(live.document());
        preflightNewIdentities(live.snapshot(),plan);
        preflightGeometry(live.snapshot(),expected);
        Object document = live.document();
        requireIndependentUndo(document);
        // No network waits or other user events can interleave this instant group.
        invoke(document,"v33",new Class<?>[]{String.class,boolean.class},touched.isEmpty()?"Edit automation":"Edit notes and automation",false);
        Object recorder=null,step=null;
        Map<String,String> identities = new LinkedHashMap<>();
        try {
            recorder=call(call(call(document,"cuK"),"TcB"),"v33");
            step=call(recorder,"qKA");
            applyGeometry(live,expected,plan);
            for(JsonNode edit:plan.required("edits")) {
                String clipId = edit.required("clip").asText(), operation=edit.required("type").asText();
                Object clip = live.clips().get(clipId);
                if(operation.equals("create")) {
                    ObjectNode desired=(ObjectNode)findNote(find(expected.withArray("clips"),clipId,"id").withArray("notes"),edit.required("temporary").asText()).required("note");
                    Object note=createNote(clip,desired);
                    String temporary=edit.required("temporary").asText();
                    applyFields(note,desired,identities);
                    identities.put(temporary,identity(note));
                    findNote(find(expected.withArray("clips"),clipId,"id").withArray("notes"),temporary).withObject("note").put("id",identity(note));
                } else if(operation.equals("update")) {
                    String noteId=edit.required("note").asText(); Object note=live.notes().get(clipId+":"+noteId);
                    ObjectNode desired=(ObjectNode)findNote(find(expected.withArray("clips"),clipId,"id").withArray("notes"),noteId).required("note");
                    for(JsonNode change:edit.required("changes")) {
                        String field=change.required("field").asText();
                        if(!GEOMETRY.contains(field))applyField(note,field,desired,identities);
                    }
                }
            }
            ObjectNode pointIdentities=Wire.object();automation.deleteAll(deletedCurves);ObjectNode defaults=automation.writeAll(curves,pointIdentities);
            List<ObjectNode> created=automation.createAll(createdCurves,pointIdentities);
            pointIdentities.fields().forEachRemaining(entry->identities.put(entry.getKey(),entry.getValue().asText()));
            automation.project(expected,curves);
            call(call(recorder,"v33"),"LHs");
            View observed=observe(frozenPaths);
            automation.projectTopology(expected,observed.snapshot(),deletedCurves,created);
            verify(live,expected,observed,touched);
            ObjectNode result=Wire.object();result.set("snapshot",observed.snapshot());result.set("identities",Wire.JSON.valueToTree(identities));result.set("defaults",defaults);
            return result.toString();
        } catch(Exception failure) {
            // Recorder acquisition is read-only; even its failure must close the group.
            if(recorder==null || step==null)throw failure;
            // Same native change-buffer reversal used by the host's validation
            // recovery. Closing the group below is not itself rollback.
            reverseStep(recorder,step,failure);
            View recovered=observe(frozenPaths);
            if(!sameDocumentState(live.snapshot(),recovered.snapshot()) || !live.payloads().equals(recovered.payloads()))
                throw new IllegalStateException("Native recovery did not restore the frozen document",failure);
            discardRecoveredStep(step);
            throw new IllegalStateException("Edit refused; native state restored: "+failure.getMessage(),failure);
        } finally {
            invoke(document,"v33",new Class<?>[]{boolean.class},false);
        }
    }

    private static void requireIdleForContentEdit(Object document) throws Exception {
        Object transport=ProjectAdapter61.value(document,"transport");
        for(String field:List.of("is_playing","is_playing_on_engine","record","write_automation"))
            if(!Boolean.FALSE.equals(ProjectAdapter61.value(transport,field)))throw new IllegalArgumentException("Content edits require stopped transport and recording disabled");
        if(Boolean.TRUE.equals(ProjectAdapter61.value(document,"is_being_saved")) || call(document,"jst")!=null)throw new IllegalArgumentException("Save/render in progress");
    }

    private static void reverseStep(Object recorder,Object step,Exception failure) throws Exception {
        Object manager=call(recorder,"v33");call(manager,"LHs");
        for(int i=0;Boolean.TRUE.equals(call(step,"Vdp"));i++) {
            if(i>=16)throw new IllegalStateException("Native recovery did not converge",failure);
            call(step,"mQv");call(manager,"LHs");
        }
    }

    /** deV tracks logical emptiness separately from its drained change buffer. */
    void discardRecoveredStep(Object step) throws Exception {
        if(step.getClass()!=type("deV"))throw new IllegalStateException("Unexpected native change step");
        if(Boolean.TRUE.equals(call(step,"LHs")))return; // Native already considers a context-only/no-write step empty.
        var pending=step.getClass().getDeclaredField("tGW");pending.setAccessible(true);
        var serialized=step.getClass().getDeclaredField("FSQ");serialized.setAccessible(true);
        var forced=step.getClass().getDeclaredField("XOR");forced.setAccessible(true);
        Object buffer=pending.get(step);int records=buffer instanceof List<?> changes?changes.size():-1;
        if(records!=0 || serialized.getBoolean(step) || forced.getBoolean(step) || Boolean.TRUE.equals(call(step,"Vdp")))
            throw new IllegalStateException("Recovered native step still contains history effects: pending="+records+", serialized="+serialized.getBoolean(step)+", forced="+forced.getBoolean(step));
        var empty=step.getClass().getDeclaredField("ncc");empty.setAccessible(true);empty.setBoolean(step,true);
        if(!Boolean.TRUE.equals(call(step,"LHs")))throw new IllegalStateException("Recovered native step is not empty");
    }

    double canonicalTime(double requested) throws Exception {
        double resolved=((Number)Reflection.method(type("anB"),"v33",double.class).invoke(null,requested)).doubleValue();
        if(!Double.isFinite(resolved) || Math.abs(requested-resolved)>0x1.0p-21)
            throw new IllegalArgumentException("Requested time exceeds the native persistent coordinate range");
        return resolved;
    }

    static void requireIndependentUndo(Object document) throws Exception {
        Object messages=call(document,"cuK");
        Object recorder=call(call(messages,"TcB"),"v33");
        Object manager=call(recorder,"v33");
        if(!Boolean.TRUE.equals(call(messages,"FSQ"))
                || Boolean.TRUE.equals(call(recorder,"TUp"))
                || Boolean.TRUE.equals(call(manager,"NKN"))
                || Boolean.TRUE.equals(call(manager,"Fgm")))
            throw new IllegalStateException("Native Undo context is busy or unavailable; refused before writes. Finish the current gesture and observe again.");
    }

    private static Set<String> preflightNewIdentities(ObjectNode before,ObjectNode plan) {
        Set<String> known=new HashSet<>(),temporary=new HashSet<>();
        for(JsonNode clip:before.required("clips"))for(JsonNode item:clip.required("notes")) {
            JsonNode note=item.required("note");known.add(note.required("id").asText());
            note.required("expressions").elements().forEachRemaining(points->points.forEach(point->known.add(point.required("id").asText())));
        }
        for(JsonNode lane:before.required("automation"))for(JsonNode point:lane.required("points"))known.add(point.required("id").asText());
        java.util.function.Consumer<String> claim=id->{
            if(id.isEmpty() || known.contains(id) || !temporary.add(id))
                throw new IllegalArgumentException("New note/point temporary identities must be unique across the whole plan");
        };
        for(JsonNode edit:plan.required("edits")) {
            String action=edit.required("type").asText();
            if(action.equals("create")) {
                claim.accept(edit.required("temporary").asText());
                edit.required("note").required("expressions").elements().forEachRemaining(points->points.forEach(point->claim.accept(point.required("id").asText())));
            }else if(action.equals("update")) {
                JsonNode original=findNote(find(before.withArray("clips"),edit.required("clip").asText(),"id").withArray("notes"),edit.required("note").asText()).required("note");
                for(JsonNode change:edit.required("changes"))if(change.required("field").asText().equals("expressions"))
                    change.required("value").fields().forEachRemaining(curve->{for(JsonNode point:curve.getValue()) {
                        String id=point.required("id").asText();boolean exists=false;
                        for(JsonNode old:original.required("expressions").required(curve.getKey()))if(old.required("id").asText().equals(id)){exists=true;break;}
                        if(!exists)claim.accept(id);
                    }});
            }
        }
        if(plan.has("automation"))for(JsonNode edit:plan.required("automation")) {
            Set<String> own=new HashSet<>();
            for(JsonNode lane:before.required("automation"))
                if(lane.required("track").equals(edit.required("track")) && lane.required("lane").equals(edit.required("lane")) && lane.required("clip").equals(edit.required("clip")))for(JsonNode point:lane.required("points"))own.add(point.required("id").asText());
            for(JsonNode point:edit.required("points"))if(!own.contains(point.required("id").asText()))claim.accept(point.required("id").asText());
        }
        if(plan.has("automationCreate"))for(JsonNode curve:plan.required("automationCreate"))for(JsonNode point:curve.required("points"))claim.accept(point.required("id").asText());
        return temporary;
    }

    private Object createNote(Object clip,ObjectNode desired) throws Exception {
        Object command=call(call(clip,"EVR"),"cSc");
        return invoke(command,"v33",new Class<?>[]{type("com.bitwig.ramona.core.oJn"),List.class},clip,
                List.of(desired.required("channel").intValue(),desired.required("key").intValue(),desired.required("start").doubleValue(),desired.required("duration").doubleValue(),desired.required("velocity").doubleValue()));
    }

    void validateImportedNote(ObjectNode desired) throws Exception {
        validateNote(desired);
        for(String field:List.of("start","duration","contentStart")) {
            double requested=finite(desired,field);
            if(Double.doubleToLongBits(canonicalTime(requested))!=Double.doubleToLongBits(requested))throw new IllegalArgumentException("Import plan must resolve native timing before mutation: "+field);
        }
        for(JsonNode curve:desired.required("expressions"))for(JsonNode point:curve) {
            double requested=finite(point,"time");
            if(Double.doubleToLongBits(canonicalTime(requested))!=Double.doubleToLongBits(requested))throw new IllegalArgumentException("Import plan must resolve native expression timing before mutation");
        }
    }

    String createImportedNote(Object clip,ObjectNode desired) throws Exception {
        ObjectNode owned=desired.deepCopy();Object note=createNote(clip,owned);
        applyFields(note,owned,new LinkedHashMap<>());return identity(note);
    }

    void removeImportedNote(Object note) throws Exception { remove(note); }

    private record Relocation(Object note,Object timeline,ObjectNode desired) {}

    /** Vacate deleted/moving sources before placing the complete final geometry. */
    private void applyGeometry(View live,ObjectNode expected,ObjectNode plan) throws Exception {
        List<Relocation> moving=new ArrayList<>();
        for(JsonNode edit:plan.required("edits"))if(edit.required("type").asText().equals("update")) {
            String clip=edit.required("clip").asText(),id=edit.required("note").asText();
            ObjectNode original=(ObjectNode)findNote(find(live.snapshot().withArray("clips"),clip,"id").withArray("notes"),id).required("note");
            ObjectNode desired=(ObjectNode)findNote(find(expected.withArray("clips"),clip,"id").withArray("notes"),id).required("note");
            if(GEOMETRY.stream().anyMatch(field->!original.required(field).equals(desired.required(field)))) {
                Object note=live.notes().get(clip+":"+id);
                moving.add(new Relocation(note,call(call(note,"YeF"),"hk_"),desired));
            }
        }
        for(JsonNode edit:plan.required("edits"))if(edit.required("type").asText().equals("delete"))
            remove(live.notes().get(edit.required("clip").asText()+":"+edit.required("note").asText()));
        for(Relocation move:moving)remove(move.note());
        for(Relocation move:moving)placeGeometry(move);
    }

    private void placeGeometry(Relocation move) throws Exception {
        Object note=move.note();ObjectNode desired=move.desired();
        for(String field:List.of("start","duration")) {
            String descriptor=field.equals("start")?"time":field;
            double requested=desired.required(field).doubleValue();
            if(number(value(note,descriptor))!=requested)set(note,descriptor,requested);
        }
        Object row=invoke(move.timeline(),"tGW",new Class<?>[]{int.class,int.class,boolean.class},
                desired.required("channel").intValue(),desired.required("key").intValue(),true);
        invoke(row,"jaQ",new Class<?>[]{type("oHd")},note);
    }

    private void applyFields(Object note,ObjectNode desired,Map<String,String> identities) throws Exception {
        for(String field:SCALARS.keySet())applyField(note,field,desired,identities);
        set(note,"content_start_time",desired.required("contentStart").doubleValue());
        applyField(note,"operators",desired,identities);applyField(note,"expressions",desired,identities);
    }
    private void applyField(Object note,String field,ObjectNode desired,Map<String,String> identities) throws Exception {
        if(SCALARS.containsKey(field)) {
            JsonNode value=desired.required(field);set(note,SCALARS.get(field),value.isBoolean()?value.booleanValue():value.doubleValue());
        } else if(field.equals("operators")) {
            JsonNode o=desired.required("operators");
            for(String name:new String[]{"chance","occurrence","recurrence","repeat"})set(note,"enable_"+name,o.required(name+"Enabled").booleanValue());
            set(note,"chance",o.required("chance").doubleValue());
            Object old=value(note,"occurrence");
            Object condition=java.util.Arrays.stream(old.getClass().getEnumConstants()).filter(v->v.toString().equals(o.required("occurrence").asText())).findFirst().orElseThrow();
            set(note,"occurrence",condition);
            set(note,"recurrence",o.required("recurrenceLength").intValue()-1 | o.required("recurrenceMask").intValue()<<3);
            set(note,"repeat_count",o.required("repeatCount").intValue());
            set(note,"repeat_curve",o.required("repeatCurve").doubleValue());set(note,"repeat_velocity_end",o.required("repeatVelocityEnd").doubleValue());
            set(note,"repeat_velocity_curve",o.required("repeatVelocityCurve").doubleValue());
        } else if(field.equals("expressions")) {
            for(int i=0;i<EXPRESSIONS.length;i++) {
                ArrayNode desiredPoints=(ArrayNode)desired.required("expressions").required(WIRE_EXPRESSIONS[i]);
                Object curve=curve(note,i,!desiredPoints.isEmpty());
                if(curve==null)continue;
                Map<String,Object> points=new HashMap<>();for(Object p:list(call(curve,"v33")))points.put(identity(p),p);
                Set<String> retained=new HashSet<>();
                List<Object> ordered=new ArrayList<>();
                for(JsonNode p:desiredPoints) {
                    String id=p.required("id").asText();Object point=points.get(id);
                    if(point==null) {
                        Set<String> previous=new HashSet<>();for(Object oldPoint:list(call(curve,"v33")))previous.add(identity(oldPoint));
                        invoke(curve,"v33",new Class<?>[]{double.class,double.class,double.class},p.required("time").doubleValue(),p.required("value").doubleValue(),p.required("spread").doubleValue());
                        point=null;for(Object candidate:list(call(curve,"v33")))if(!previous.contains(identity(candidate))){point=candidate;break;}
                        if(point==null)throw new IllegalStateException("Expression insertion not observed");
                        identities.put(id,identity(point));((ObjectNode)p).put("id",identity(point));
                    }
                    retained.add(identity(point));
                    ordered.add(point);
                    moveTime(point,curve,p.required("time").doubleValue());set(point,"value",p.required("value").doubleValue());
                    set(point,"curvature",p.required("curvature").doubleValue());set(point,"spread",p.required("spread").doubleValue());
                    Object old=value(point,"interpolation_mode");
                    Object interpolation=java.util.Arrays.stream(old.getClass().getEnumConstants()).filter(v->v.toString().equals(p.required("interpolation").asText())).findFirst().orElseThrow();
                    set(point,"interpolation_mode",interpolation);
                }
                for(Map.Entry<String,Object> p:points.entrySet())if(!retained.contains(p.getKey()))invoke(curve,"v33",new Class<?>[]{type("oHd"),boolean.class},p.getValue(),false);
                orderPoints(curve,ordered);
            }
        }
    }
    /** Equal-time points have meaningful native list order; preserve the requested graph order. */
    private void orderPoints(Object curve,List<Object> ordered) throws Exception {
        List<?> actual=list(call(curve,"v33"));
        if(actual.size()!=ordered.size())throw new IllegalStateException("Expression point inventory differs");
        for(int index=0;index<ordered.size();index++) {
            Object point=ordered.get(index);
            if(list(call(curve,"v33")).get(index)==point)continue;
            invoke(curve,"v33",new Class<?>[]{type("oHd"),boolean.class},point,false);
            invoke(curve,"YLV",new Class<?>[]{type("oHd"),int.class},point,index);
        }
    }
    /** Native KKl's retiming pattern: preserve the event, maintain sorted ownership. */
    private void moveTime(Object event,Object timeline,double time) throws Exception {
        if(number(value(event,"time"))==time)return;
        invoke(timeline,"v33",new Class<?>[]{type("oHd"),boolean.class},event,false);
        set(event,"time",time);
        invoke(timeline,"jaQ",new Class<?>[]{type("oHd")},event);
    }
    private void remove(Object note) throws Exception { invoke(call(note,"YeF"),"v33",new Class<?>[]{type("oHd"),boolean.class},note,false); }
    private static void set(Object target,String name,Object v) throws Exception {invoke(invoke(target,"Nln",new Class<?>[]{String.class},name),"setValue",new Class<?>[]{Object.class},v);}

    private static ObjectNode find(ArrayNode items,String id,String field){for(JsonNode n:items)if(n.required(field).asText().equals(id))return (ObjectNode)n;throw new IllegalArgumentException("Unknown identity "+id);}
    private static ObjectNode findNote(ArrayNode notes,String id){for(JsonNode n:notes)if(n.required("note").required("id").asText().equals(id))return (ObjectNode)n;throw new IllegalArgumentException("Unknown source note "+id);}
    private static int indexOf(ArrayNode nodes,JsonNode node){for(int i=0;i<nodes.size();i++)if(nodes.get(i)==node)return i;throw new IllegalArgumentException("Missing source");}
    private static void requireKeys(JsonNode object,Set<String> keys){
        if(!object.isObject())throw new IllegalArgumentException("Expected object");
        Set<String> actual=new HashSet<>();object.fieldNames().forEachRemaining(actual::add);
        if(!actual.equals(keys))throw new IllegalArgumentException("Unexpected or missing fields: "+actual);
    }
    private static double finite(JsonNode object,String key){JsonNode v=object.required(key);if(!v.isNumber() || !Double.isFinite(v.doubleValue()))throw new IllegalArgumentException("Invalid "+key);return v.doubleValue();}
    private static void bounded(JsonNode o,String k,double low,double high){double v=finite(o,k);if(v<low || v>high)throw new IllegalArgumentException("Out of range "+k);}
    private static void integer(JsonNode o,String k,int low,int high){JsonNode v=o.required(k);if(!v.isIntegralNumber() || !v.canConvertToInt() || v.intValue()<low || v.intValue()>high)throw new IllegalArgumentException("Invalid integer "+k);}
    private static void validateNote(ObjectNode n){
        requireKeys(n,Set.of("id","channel","key","start","duration","muted","velocity","releaseVelocity","velocitySpread","contentStart","operators","expressions"));
        if(!n.required("id").isTextual() || n.required("id").asText().isEmpty() || !n.required("muted").isBoolean())throw new IllegalArgumentException("Invalid note identity/mute");
        integer(n,"channel",0,15);integer(n,"key",0,127);finite(n,"start");finite(n,"contentStart");
        if(finite(n,"duration")<=0 || !Double.isFinite(n.required("start").doubleValue()+n.required("duration").doubleValue()))throw new IllegalArgumentException("Invalid duration");
        for(String field:List.of("velocity","releaseVelocity","velocitySpread"))bounded(n,field,0,1);
        JsonNode o=n.required("operators");requireKeys(o,Set.of("chanceEnabled","chance","occurrenceEnabled","occurrence","recurrenceEnabled","recurrenceLength","recurrenceMask","repeatEnabled","repeatCount","repeatCurve","repeatVelocityEnd","repeatVelocityCurve"));
        for(String field:List.of("chanceEnabled","occurrenceEnabled","recurrenceEnabled","repeatEnabled"))if(!o.required(field).isBoolean())throw new IllegalArgumentException("Invalid operator enable");
        bounded(o,"chance",0,1);integer(o,"recurrenceLength",1,8);integer(o,"recurrenceMask",0,255);integer(o,"repeatCount",-127,127);
        for(String field:List.of("repeatCurve","repeatVelocityEnd","repeatVelocityCurve"))bounded(o,field,-1,1);
        if(!Set.of("ALWAYS","FIRST","NOT_FIRST","PREV","NOT_PREV","PREV_CHANNEL","NOT_PREV_CHANNEL","PREV_KEY","NOT_PREV_KEY","FILL","NOT_FILL").contains(o.required("occurrence").asText()))throw new IllegalArgumentException("Invalid occurrence");
        JsonNode curves=n.required("expressions");requireKeys(curves,Set.of(WIRE_EXPRESSIONS));
        for(String kind:WIRE_EXPRESSIONS){JsonNode points=curves.required(kind);if(!points.isArray())throw new IllegalArgumentException("Expected curve points");
            double previous=Double.NEGATIVE_INFINITY;Set<String> ids=new HashSet<>();
            for(JsonNode point:points){requireKeys(point,Set.of("id","time","value","interpolation","curvature","spread"));
                if(!point.required("id").isTextual() || point.required("id").asText().isEmpty() || !ids.add(point.required("id").asText()))throw new IllegalArgumentException("Invalid point identity");
                double t=finite(point,"time");if(t<previous)throw new IllegalArgumentException("Unsorted curve points");previous=t;finite(point,"value");bounded(point,"curvature",-1,1);
                if(finite(point,"spread")<0)throw new IllegalArgumentException("Invalid spread");
                if(!Set.of("HOLD","LINEAR").contains(point.required("interpolation").asText()))throw new IllegalArgumentException("Invalid interpolation");
            }
        }
    }
    private static void preflightGeometry(ObjectNode before,ObjectNode expected){
        for(JsonNode c:expected.required("clips")) {
            List<JsonNode> ns=new ArrayList<>();c.required("notes").forEach(n->ns.add(n.required("note")));
            ns.sort(Comparator.comparingInt((JsonNode n)->n.required("channel").intValue()).thenComparingInt(n->n.required("key").intValue()).thenComparingDouble(n->n.required("start").doubleValue()));
            for(int i=1;i<ns.size();i++){JsonNode a=ns.get(i-1),b=ns.get(i);
                if(a.required("channel").equals(b.required("channel")) && a.required("key").equals(b.required("key")) && a.required("start").doubleValue()+a.required("duration").doubleValue()>b.required("start").doubleValue()) {
                    JsonNode owner=find(before.withArray("clips"),c.required("id").asText(),"id");
                    for(JsonNode n:List.of(a,b)){ObjectNode old;
                        try{old=findNote((ArrayNode)owner.required("notes"),n.required("id").asText());}catch(IllegalArgumentException missing){throw new IllegalArgumentException("Creation would overwrite existing material");}
                        for(String field:List.of("channel","key","start","duration"))if(!old.required("note").required(field).equals(n.required(field)))throw new IllegalArgumentException("Destination overlaps existing material");
                    }
                }
            }
        }
    }
    private static boolean sameDocumentState(ObjectNode a,ObjectNode b){ObjectNode ac=a.deepCopy(),bc=b.deepCopy();ac.remove("revision");bc.remove("revision");return ac.equals(bc);}
    private static void verify(View before,ObjectNode expected,View observed,Set<String> touched) {
        ObjectNode actual=observed.snapshot().deepCopy();
        // Arrays use stable document IDs; row transfers can change native iteration order.
        for(ObjectNode root:List.of(expected,actual))for(JsonNode clip:root.required("clips")){
            List<JsonNode> ns=new ArrayList<>();clip.required("notes").forEach(ns::add);ns.sort(Comparator.comparing(n->n.required("note").required("id").asText()));
            ((ObjectNode)clip).set("notes",Wire.JSON.valueToTree(ns));
        }
        if(!sameDocumentState(expected,actual))throw new IllegalStateException("Independent readback differs from complete plan: "+differences(expected,actual));
        for(Map.Entry<String,String> p:before.payloads().entrySet())if(!touched.contains(p.getKey()) && !p.getValue().equals(observed.payloads().get(p.getKey())))throw new IllegalStateException("Unselected payload changed");
    }

    /** Bounded contract diagnostics, retaining the exact field and both values. */
    private static String differences(ObjectNode expected,ObjectNode actual) {
        ObjectNode a=expected.deepCopy(),b=actual.deepCopy();a.remove("revision");b.remove("revision");
        List<String> result=new ArrayList<>();differences(a,b,"",result);return String.join("; ",result);
    }
    private static void differences(JsonNode expected,JsonNode actual,String path,List<String> result) {
        if(result.size()>=8 || Objects.equals(expected,actual))return;
        if(expected!=null && actual!=null && expected.isObject() && actual.isObject()) {
            Set<String> fields=new java.util.TreeSet<>();expected.fieldNames().forEachRemaining(fields::add);actual.fieldNames().forEachRemaining(fields::add);
            for(String field:fields)differences(expected.get(field),actual.get(field),path+"/"+field,result);
        } else if(expected!=null && actual!=null && expected.isArray() && actual.isArray() && expected.size()==actual.size()) {
            for(int i=0;i<expected.size();i++)differences(expected.get(i),actual.get(i),path+"/"+i,result);
        } else {
            result.add(path+": expected "+compact(expected)+", observed "+compact(actual));
        }
    }
    private static String compact(JsonNode value) {
        if(value==null)return "<missing>";
        if(value.isContainerNode())return value.getNodeType()+"["+value.size()+"]";
        String text=value.toString();return text.length()>96?text.substring(0,96)+"…":text;
    }

    private View observe() throws Exception {return observe(null);}
    private View observe(JsonNode paths) throws Exception {
        Object document = observeCurrentDocument();
        Map<String,Object> tracks=new LinkedHashMap<>();Map<String,Object> clips=new LinkedHashMap<>();Map<String,Set<String>> selected=new HashMap<>();Set<String> selectedObjects=new HashSet<>();
        Object launcherSelection=paths==null?Launcher61.selection(proxy):null;
        Object editor=null;boolean time=false,detail=false,launcher=false;double from=0,to=0;List<?> lanes=List.of();
        if(paths!=null) {
            if(!paths.isArray() || paths.isEmpty())throw new IllegalArgumentException("Explicit clip paths must be a nonempty array");
            for(JsonNode path:paths) {
                Object owner=ProjectAddress61.resolveNative(path,document,proxy.getClass().getClassLoader());
                String kind=path.required("elements").get(path.required("elements").size()-1).required("kind").asText();
                List<Object> requested=new ArrayList<>();
                if(kind.equals("clip") || kind.equals("launcherClip"))requested.add(owner);
                else if(kind.equals("track") || kind.equals("group")) {
                    Object track=kind.equals("group")?call(owner,"DQP"):owner;tracks.put(identity(track),track);
                    Object lane=ProjectAdapter61.value(track,"main_note_clip_lane");
                    if(lane!=null)try(var stream=(java.util.stream.Stream<?>)invoke(lane,"v33",new Class<?>[]{double.class,double.class},Double.NEGATIVE_INFINITY,Double.POSITIVE_INFINITY)) {
                        var iterator=stream.iterator();while(iterator.hasNext()){Object candidate=iterator.next();if(type("UMj").isInstance(candidate))requested.add(candidate);}
                    }
                } else throw new IllegalArgumentException("Content paths must address tracks, groups or Note Clips");
                for(Object clip:requested) {
                    if(!type("UMj").isInstance(clip) || clips.put(identity(clip),clip)!=null)throw new IllegalArgumentException("Expected nonoverlapping content paths");
                    selectedObjects.add(identity(clip));for(Object note:notes(clip))selected.computeIfAbsent(identity(clip),ignored->new HashSet<>()).add(identity(note));
                }
            }
        } else if(launcherSelection!=null) {
            launcher=true;
            for(Object slot:list(call(launcherSelection,"jaQ"))) {
                Object clip=Launcher61.clip(slot);
                if(clip==null)continue;
                if(!type("UMj").isInstance(clip))throw new IllegalArgumentException("Selected launcher slot contains unsupported material");
                clips.put(identity(clip),clip);
                selectedObjects.add(identity(clip));
                for(Object note:notes(clip))selected.computeIfAbsent(identity(clip),ignored->new HashSet<>()).add(identity(note));
            }
        } else {
        Object widget = call(proxy,"getDocumentWidget");
        if (widget == null) throw new IllegalStateException("No connected project widget");
        editor = value(widget,"active_editor_selection");
        if (editor == null) {
            // Inspector text entry releases active focus, but aBI retains its
            // inspected bhU selection. Resolve that selection's native owner;
            // never combine cached selections from other editor wrappers.
            Object inspected = value(widget,"inspector_selection");
            if (inspected != null) editor = call(inspected,"lW_");
        }
        if(editor==null)editor=call(call(proxy,"getDocumentGUIWrapper"),"iDx");
        Object settings = call(editor,"mP_");
        time = (boolean)value(settings,"is_time_selection_active");
        Object range = call(editor,"Iks");
        from = number(call(time?range:call(editor,"jaQ"),"zeN"));
        to = number(call(time?range:call(editor,"jaQ"),time?"cSc":"FSQ"));
        lanes = list(call(call(range,"jaQ"),"zeN"));
        detail = type("biD").isInstance(editor);
        if (detail) {
            List<?> occurrences = time
                    ? list(invoke(editor,"v33",new Class<?>[]{double.class,double.class,boolean.class,type("boA")},from,to,true,null))
                    : list(call(call(call(editor,"jaQ"),"jaQ"),"jaQ"));
            for (Object occurrence : occurrences) {
                if (!type("fOD").isInstance(occurrence)) throw new IllegalStateException("Unsupported selected note-editor object: "+occurrence.getClass().getName());
                Object note = call(occurrence,"v33"), clip = call(occurrence,"YXc");
                clips.put(identity(clip),clip);
                selected.computeIfAbsent(identity(clip), ignored -> new HashSet<>()).add(identity(note));
                selectedObjects.add(identity(clip)+":"+identity(note));
            }
        } else if (type("biA").isInstance(editor)) {
            if (time) {
                // pxE.uS delegates to the native arranger selection strategy,
                // which expands selected meta/group lanes into their real clips.
                // pxE.zeN omits those group summaries and cannot define this scope.
                Object nativeSelection = call(range,"jaQ");
                lanes = list(call(nativeSelection,"jaQ"));
                List<?> candidates=list(call(nativeSelection,"uS"));
                for (Object clip : candidates) {
                    if (type("UMj").isInstance(clip)) clips.put(identity(clip),clip);
                    else if (!type("I3W").isInstance(clip)) throw new IllegalStateException("Unsupported selected arranger material: "+clip.getClass().getName());
                }

            } else {
                Object nativeSelection = call(call(editor,"jaQ"),"jaQ");
                List<?> raw = list(call(nativeSelection,"jaQ"));
                for (Object object : raw) {
                    if(type("I3W").isInstance(object))throw new IllegalStateException("Unsupported explicit automation-clip selection: stored-note operations require note material");
                    selectedObjects.add(identity(object));
                }
                if (!raw.isEmpty()) {
                    // The native arranger object strategy expands meta/group clips
                    // recursively, with its own deduplication semantics.
                    Object strategy = call(nativeSelection,"vfN");
                    for (Object clip : (java.util.Collection<?>)invoke(strategy,"EwU",new Class<?>[]{java.util.Collection.class},raw)) {
                        if (type("UMj").isInstance(clip)) clips.put(identity(clip),clip);
                    else if (!type("I3W").isInstance(clip)) throw new IllegalStateException("Unsupported selected arranger material: "+clip.getClass().getName());
                    }
                }
            }
            for (Object clip : clips.values()) for (Object note : notes(clip)) {
                if (!time || intersects(clip,note,from,to)) selected.computeIfAbsent(identity(clip),ignored->new HashSet<>()).add(identity(note));
            }
        } else throw new IllegalStateException("Unsupported active editor: "+editor.getClass().getName());
        }
        ObjectNode selection = Wire.object();
        selection.put("editor", launcher?"launcher":editor==null?"paths":detail?"detailEditor":"arranger");
        selection.put("kind", time ? "time" : detail ? "notes" : "clips");
        selection.put("from",from); selection.put("to",to);
        ArrayNode laneIds = selection.putArray("lanes");
        List<String> laneKeys = new ArrayList<>();
        if(time){for (Object lane : lanes) laneKeys.add(identity(lane));}
        else{for(Object clip:clips.values())laneKeys.add(identity(call(clip,"rwW")));}
        laneKeys.stream().sorted().forEach(laneIds::add);
        ArrayNode objectIds = selection.putArray("objects");
        if (!time) selectedObjects.stream().sorted().forEach(objectIds::add);
        ObjectNode root = Wire.object(); root.put("document",generation); root.set("selection",selection);root.set("paths",paths==null?com.fasterxml.jackson.databind.node.NullNode.instance:paths.deepCopy());
        ArrayNode clipNodes = root.putArray("clips");
        for(Object lane:lanes) {
            Object track=invoke(lane,"EwU",new Class<?>[]{Class.class},type("com.bitwig.flt.document.core.master.kkr"));
            if(track!=null)tracks.put(identity(track),track);
        }
        Map<String,Object> sourceNotes = new HashMap<>(); Map<String,String> payloads = new HashMap<>();
        Set<String> arrangerTracks=new HashSet<>(tracks.keySet());
        Map<String,List<Object>> launcherClips=new HashMap<>();
        for (String id : clips.keySet().stream().sorted().toList()) {
            Object clip = clips.get(id); ObjectNode c = Wire.object();
            c.put("id",id); c.put("lane",identity(call(clip,"rwW")));
            Object track = invoke(clip,"EwU",new Class<?>[]{Class.class},type("com.bitwig.flt.document.core.master.kkr"));
            if (track == null) throw new IllegalStateException("Clip has no owning track");
            ObjectNode path=Launcher61.clipPath(document,track,clip);
            c.set("path",path);
            tracks.put(identity(track),track);
            c.put("track",identity(track)); c.put("content",identity(call(clip,"FSQ")));
            boolean inLauncher=Launcher61.noteSlot(clip)!=null;
            if(inLauncher)launcherClips.computeIfAbsent(identity(track),ignored->new ArrayList<>()).add(clip);
            else arrangerTracks.add(identity(track));
            c.put("start",inLauncher?0.0:number(call(clip,"T1L")));
            c.put("duration",number(call(clip,inLauncher?"Fgm":"GaO")));
            c.put("sourceStart",number(call(clip,"dD_"))); c.put("sourceEnd",number(call(clip,"Fgm")));
            c.put("loopEnabled",(boolean)call(clip,"YeF"));
            Object loop = call(clip,"fvB"); c.put("loopStart",number(value(loop,"time"))); c.put("loopLength",number(value(loop,"duration")));
            ArrayNode ns = c.putArray("notes");
            List<?> sources = notes(clip);
            List<Object> ordered = new ArrayList<>(sources);
            ordered.sort(Comparator.comparing(n -> {try{return identity(n);}catch(Exception e){throw new IllegalStateException(e);}}));
            for (Object note : ordered) {
                String key = id+":"+identity(note);
                sourceNotes.put(key,note); payloads.put(key,payload(note));
                ObjectNode item = Wire.object(); item.set("note",readNote(note));
                item.put("selected",selected.getOrDefault(id,Set.of()).contains(identity(note))); ns.add(item);
            }
            clipNodes.add(c);
        }
        ArrayNode owners=root.putArray("owners"),automation=root.putArray("automation");
        AutomationAdapter61 automationAdapter=new AutomationAdapter61(proxy);
        Map<String,AutomationAdapter61.Owner> curveOwners=new HashMap<>();
        for(String trackId:tracks.keySet().stream().sorted().toList()) {
            Object track=tracks.get(trackId);owners.add(ProjectAddress61.nativeTrackPath(document,track));
            automation.addAll(automationAdapter.readTrack(document,track,arrangerTracks.contains(trackId),launcherClips.getOrDefault(trackId,List.of()),curveOwners));
        }
        root.put("revision",ProjectRevision61.read(proxy,document));
        return new View(document,root,Map.copyOf(clips),Map.copyOf(sourceNotes),Map.copyOf(payloads),Map.copyOf(curveOwners),Set.copyOf(arrangerTracks));
    }

    /** Native usage enumeration is independent of selection; aliases cannot be silently edited. */
    private void requireIsolatedSources(View view) throws Exception {
        String kind=view.snapshot().required("selection").required("kind").asText();
        for(Object clip:view.clips().values()) {
            if(!kind.equals("clips") && (boolean)call(clip,"YeF"))
                throw new IllegalStateException("Unsupported occurrence isolation: partial/note selection in a looped clip; no writes");
            Object source=call(clip,"FSQ");
            int count=0;
            try(java.util.stream.Stream<?> users=(java.util.stream.Stream<?>)call(source,"bRk")) {
                var iterator=users.iterator();
                while(iterator.hasNext()) {
                    Object user=iterator.next();
                    if(call(user,"QSj")!=source || ++count>1)
                        throw new IllegalStateException("Unsupported shared source isolation: aliased clip content; no writes");
                    if(user!=clip)throw new IllegalStateException("Native source owner differs; no writes");
                }
            }
            if(count!=1)throw new IllegalStateException("Native source uniqueness is unproven; no writes");
        }
    }

    private Object observeCurrentDocument() throws Exception {
        Object document=observeTargetDocument();if(document==null)throw new IllegalStateException("No connected project; navigate through DrivenByMoss OSC");return document;
    }
    private Object observeTargetDocument() throws Exception {
        if(!Boolean.TRUE.equals(call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong document thread");
        observeDocumentChanges();
        Object document=call(proxy,"getDocument");
        nativeChanges.bind(document);
        if(currentDocument!=document || generation==null) {currentDocument=document;generation=UUID.randomUUID().toString();frozenRevision=null;}
        return document;
    }

    /** The proxy target emits every switch, including A -> B -> A between reads. */
    private void observeDocumentChanges() throws Exception {
        if(closed)throw new IllegalStateException("Adapter exited");
        if(documentListener!=null)return;
        nativeChanges=new ProjectRevision61(proxy);
        try {
        Object target=call(proxy,"getTargetValue");Class<?> listenerType=type("ums");
        Object listener=java.lang.reflect.Proxy.newProxyInstance(listenerType.getClassLoader(),new Class<?>[]{listenerType},(receiver,method,args)->switch(method.getName()) {
            case "eventOccurred" -> {instruments.invalidate();nativeChanges.invalidate();frozenRevision=null;frozenSnapshot=null;currentDocument=null;generation=null;yield null;}
            case "hashCode" -> System.identityHashCode(receiver);
            case "equals" -> receiver==args[0];
            case "toString" -> "Notes document invalidation";
            default -> throw new UnsupportedOperationException(method.getName());
        });
        invoke(target,"addListener",new Class<?>[]{listenerType},listener);
        documentTarget=target;documentListener=listener;
        }catch(Exception error){try{nativeChanges.close();}catch(Exception cleanup){error.addSuppressed(cleanup);}nativeChanges=null;throw error;}
    }

    /** DocumentAccess dispatches disposal on the same owner thread as observation. */
    @Override public void close() {
        try {
            if(!Boolean.TRUE.equals(call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong document thread");
            instruments.invalidate();exports.close();
            if(nativeChanges!=null)nativeChanges.close();nativeChanges=null;

            closed=true;frozenRevision=null;currentDocument=null;generation=null;
            if(documentListener!=null)invoke(documentTarget,"removeListener",new Class<?>[]{type("ums")},documentListener);
            documentTarget=null;documentListener=null;
        }catch(Exception error){throw new IllegalStateException("Native observer disposal failed",error);}
    }

    private boolean intersects(Object clip, Object note, double from, double to) throws Exception {
        double duration = number(call(clip,"GaO")); Object mapping = call(clip,"heu");
        int last = (boolean)call(clip,"YeF") ? Math.max(0,((Number)invoke(mapping,"TUp",new Class<?>[]{double.class,boolean.class},duration,true)).intValue()) : 0;
        for (int i=0;i<=last;i++) {
            Object occurrence = Reflection.method(type("fOD"),"v33",type("mGg"),type("bT"),int.class,type("exV")).invoke(null,note,clip,i,null);
            double start = number(call(occurrence,"heu")), end = number(call(occurrence,"TcB"));
            if (end>start && start<to && end>from) return true;
        }
        return false;
    }

    ObjectNode readNote(Object note) throws Exception {
        ObjectNode n = Wire.object(); n.put("id",identity(note));
        n.put("channel",((Number)call(note,"yhG")).intValue()); n.put("key",((Number)call(note,"v33")).intValue());
        n.put("start",number(call(note,"T1L"))); n.put("duration",number(value(note,"duration")));
        n.put("muted",(boolean)value(note,"is_muted"));
        n.put("velocity",number(value(note,"note_on_velocity"))); n.put("releaseVelocity",number(value(note,"note_off_velocity")));
        n.put("velocitySpread",number(value(note,"velocity_spread"))); n.put("contentStart",number(value(note,"content_start_time")));
        ObjectNode operators = n.putObject("operators");
        for (String name : new String[]{"chance","occurrence","recurrence","repeat"}) operators.put(name+"Enabled",(boolean)value(note,"enable_"+name));
        operators.put("chance",number(value(note,"chance"))); operators.put("occurrence",value(note,"occurrence").toString());
        int recurrence = ((Number)value(note,"recurrence")).intValue();
        operators.put("recurrenceLength",(recurrence&7)+1); operators.put("recurrenceMask",(recurrence>>3)&255);
        operators.put("repeatCount",((Number)value(note,"repeat_count")).intValue());
        operators.put("repeatCurve",number(value(note,"repeat_curve"))); operators.put("repeatVelocityEnd",number(value(note,"repeat_velocity_end")));
        operators.put("repeatVelocityCurve",number(value(note,"repeat_velocity_curve")));
        ObjectNode expressions = n.putObject("expressions");
        for (int i=0;i<EXPRESSIONS.length;i++) {
            ArrayNode points = expressions.putArray(WIRE_EXPRESSIONS[i]);
            Object curve = curve(note,i,false);
            if (curve == null) continue;
            for (Object point : list(call(curve,"v33"))) {
                ObjectNode p = Wire.object(); p.put("id",identity(point)); p.put("time",number(value(point,"time")));
                p.put("value",number(value(point,"value"))); p.put("interpolation",value(point,"interpolation_mode").toString());
                p.put("curvature",number(value(point,"curvature"))); p.put("spread",number(value(point,"spread"))); points.add(p);
            }
        }
        return n;
    }

    private Object curve(Object note,int index,boolean create) throws Exception {
        Object kind = java.util.Arrays.stream(type("ZkX").getEnumConstants()).filter(k->k.toString().equals(EXPRESSIONS[index])).findFirst().orElseThrow();
        return invoke(call(note,"Vdp"),"v33",new Class<?>[]{type("ZkX"),boolean.class},kind,create);
    }

    private String payload(Object note) throws Exception {
        Class<?> options=type("com.bitwig.ramona.serial.CRe");
        byte[] bytes=(byte[])invoke(note,"jaQ",new Class<?>[]{options},options.getField("YLV").get(null));
        return Base64.getEncoder().encodeToString(bytes);
    }
    private List<?> notes(Object clip) throws Exception {
        return list(invoke(call(clip,"qKA"),"v33",new Class<?>[]{double.class,double.class},Double.NEGATIVE_INFINITY,Double.POSITIVE_INFINITY));
    }
    private Class<?> type(String name) throws ClassNotFoundException { return Class.forName(name,true,proxy.getClass().getClassLoader()); }
    private static Object call(Object target,String name) throws Exception { return Reflection.call(target,name); }
    private static Object invoke(Object target,String name,Class<?>[] types,Object...args) throws Exception { return Reflection.call(target,name,types,args); }
    private static Object value(Object target,String name) throws Exception { return call(invoke(target,"Nln",new Class<?>[]{String.class},name),"getValue"); }
    private static String identity(Object target) throws Exception { return value(target,"managed_object_ID").toString(); }
    private static double number(Object v) { return ((Number)v).doubleValue(); }
    private static List<?> list(Object v) { return (List<?>)v; }
}
