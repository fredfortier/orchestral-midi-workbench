import java.lang.reflect.*;
import java.nio.file.*;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Stream;

/** Version-specific research only. All document access executes in Java on its owner thread. */
public final class NotesDocumentProbe {
  private static final AtomicLong sequence = new AtomicLong();
  private static final Map<Long, String> results = new ConcurrentHashMap<>();
  private static final Path output = Path.of("/tmp/notes-transformer-java-probes.jsonl");
  private static Object fixtureDocument;
  private static List<?> fixtureLanes = List.of();
  private static Object stagedClip;
  private static Object stagedLane;
  private static String stagedSource;

  public static long submit(Object proxy, String operation, String[] input) throws Exception {
    final long id = sequence.incrementAndGet();
    final String[] args = input.clone();
    Method dispatcher = method(proxy.getClass(), "exec", Runnable.class);
    dispatcher.invoke(proxy, (Runnable) () -> {
      Map<String, Object> entry = map("id", id, "probe", operation, "boundary", "private-document-diagnostic",
          "at", Instant.now().toString(), "thread", Thread.currentThread().getName());
      try {
        if (!Boolean.TRUE.equals(call(proxy, "isDocumentThread"))) throw new IllegalStateException("Wrong thread");
        entry.put("result", execute(proxy, operation, args));
        entry.put("error", null);
      } catch (Throwable error) {
        while (error instanceof InvocationTargetException) error = ((InvocationTargetException) error).getTargetException();
        entry.put("error", error.toString());
      }
      String json = json(entry);
      results.put(id, json);
      results.keySet().removeIf(key -> key < id - 256);
      try { Files.writeString(output, json + "\n", StandardOpenOption.CREATE, StandardOpenOption.APPEND); }
      catch (Exception error) { results.put(id, json(map("id", id, "error", error.toString()))); }
    });
    return id;
  }

  public static String result(long id) { return results.get(id); }

  private static Object execute(Object proxy, String operation, String[] args) throws Exception {
    Object wrapper = call(proxy, "getDocumentGUIWrapper");
    Object document = call(proxy, "getDocument");
    if (wrapper == null || document == null) throw new IllegalStateException("No document");
    Object range = call(call(wrapper, "iDx"), "Iks");
    List<?> lanes = list(call(call(range, "jaQ"), "zeN"));
    double from = number(call(range, "zeN"));
    double to = number(call(range, "cSc"));
    switch (operation) {
      case "selection": {
        Map<String, Object> views = new LinkedHashMap<>();
        for (String[] view : new String[][]{{"arranger", "iDx"}, {"arrangerTrackEditor", "Zlo"},
            {"arrangerClipEditor", "XlL"}, {"launcherClipEditor", "yhG"}}) {
          Object editor = call(wrapper, view[1]);
          Object objects = call(editor, "jaQ");
          Object time = call(editor, "Iks");
          views.put(view[0], map("data", describe(call(editor, "mP_")),
              "objectBounds", List.of(call(objects, "zeN"), call(objects, "FSQ")),
              "objects", list(call(call(objects, "jaQ"), "jaQ")).stream().map(NotesDocumentProbe::describe).toList(),
              "rangeBounds", List.of(call(time, "zeN"), call(time, "cSc")),
              "lanes", list(call(call(time, "jaQ"), "zeN")).stream().map(NotesDocumentProbe::describe).toList()));
        }
        return map("document", describe(document), "views", views);
      }
      case "rangeContents": return inventory(lanes, from, to);
      case "transportInventory": {
        Object transport = call(document, "Nh");
        return map("object", describe(transport), "attributes", attributes(transport));
      }
      case "coverage": case "locations": return inventory(lanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
      case "contextAt": {
        Object transport = call(document, "Nh");
        double position = Double.parseDouble(args[0]);
        if (!Double.isFinite(position) || position < 0) throw new IllegalArgumentException("Invalid project beat position");
        Object beforePosition = call(call(transport, "jDA"), "getValue");
        double seconds = number(invoke(transport, "EwU", new Class<?>[]{double.class}, position));
        double roundTrip = number(invoke(transport, "jaQ", new Class<?>[]{double.class}, seconds));
        Object tempoParameter = call(transport, "pSK");
        Object automation = call(transport, "zNG");
        double tempo = number(call(transport, "QXt"));
        Object evaluation = null;
        if (automation != null) {
          double normalized = number(invoke(call(automation, "XQF"), "v33",
              new Class<?>[]{nativeClass(transport, "FA"), double.class, boolean.class}, tempoParameter, position, true));
          tempo = number(invoke(call(tempoParameter, "xNb"), "TUp", new Class<?>[]{double.class}, normalized));
          Object timeline = call(call(automation, "XQF"), "Iks");
          evaluation = map("normalized", normalized, "points", list(call(timeline, "v33")).stream().map(point -> {
            try { return attributes(point); } catch (Exception error) { throw new IllegalStateException(error); }
          }).toList());
        }
        Object signature = call(transport, "IFc");
        Object signaturePoint = invoke(signature, "j_", new Class<?>[]{double.class}, position);
        int packedSignature = (int)call(signaturePoint == null ? signature : signaturePoint, signaturePoint == null ? "tGW" : "v33");
        int numerator = (int)method(nativeClass(transport, "RYK"), "tGW", int.class).invoke(null, packedSignature);
        int denominator = (int)method(nativeClass(transport, "RYK"), "ncc", int.class).invoke(null, packedSignature);
        return map("projectBeats", position, "seconds", seconds, "roundTripBeats", roundTrip, "tempoBpm", tempo,
            "automationEvaluation", evaluation, "signature", describe(signature), "signaturePoint", signaturePoint == null ? null : attributes(signaturePoint),
            "numerator", numerator, "denominator", denominator, "currentNumerator", call(signature, "zeN"), "currentDenominator", call(signature, "heu"),
            "playheadBefore", beforePosition, "playheadAfter", call(call(transport, "jDA"), "getValue"));
      }
      case "tempoPoint": {
        requireFixture(document, lanes);
        double position = Double.parseDouble(args[0]), bpm = Double.parseDouble(args[1]);
        if (!Double.isFinite(position + bpm) || position < 0 || bpm < 20 || bpm > 300) throw new IllegalArgumentException("Invalid fixture tempo");
        Object transport = call(document, "Nh");
        Object parameter = call(transport, "pSK");
        begin(document, "tempo point");
        Object point;
        try {
          Object automation = invoke(call(transport, "oOg"), "TUp", new Class<?>[]{nativeClass(transport, "com.bitwig.flt.document.core.iface.bf")}, call(parameter, "NYl"));
          Object timeline = call(call(automation, "XQF"), "Iks");
          Object interpolation = nativeClass(transport, "CBo").getField("v33").get(null);
          point = invoke(timeline, "v33", new Class<?>[]{double.class, double.class, Double.class, boolean.class,
              nativeClass(transport, "CBo"), int.class, nativeClass(transport, "bD"), java.util.function.Consumer.class, boolean.class},
              position, invoke(call(parameter, "xNb"), "v33", new Class<?>[]{double.class}, bpm), null, false, interpolation, -1, call(parameter, "xNb"), null, true);
        } finally { end(document); }
        return map("requestedPosition", position, "requestedBpm", bpm, "point", attributes(point));
      }
      case "signaturePoint": {
        requireFixture(document, lanes);
        double position = Double.parseDouble(args[0]);
        int numerator = Integer.parseInt(args[1]), denominator = Integer.parseInt(args[2]);
        if (!Double.isFinite(position) || position < 0 || numerator < 1 || numerator > 32 || !List.of(1,2,4,8,16,32).contains(denominator))
          throw new IllegalArgumentException("Invalid fixture time signature");
        Object transport = call(document, "Nh"), parameter = call(transport, "IFc");
        int packed = (int)method(nativeClass(transport, "RYK"), "v33", int.class, int.class, int.class).invoke(null, numerator, denominator, 16);
        double normalized = number(invoke(call(parameter, "qKA"), "v33", new Class<?>[]{int.class}, packed));
        begin(document, "signature point");
        Object point;
        try {
          Object automation = invoke(call(transport, "oOg"), "TUp", new Class<?>[]{nativeClass(transport, "com.bitwig.flt.document.core.iface.bf")}, call(parameter, "NYl"));
          Object timeline = call(call(automation, "XQF"), "Iks");
          Object interpolation = nativeClass(transport, "CBo").getField("v33").get(null);
          point = invoke(timeline, "v33", new Class<?>[]{double.class, double.class, Double.class, boolean.class,
              nativeClass(transport, "CBo"), int.class, nativeClass(transport, "bD"), java.util.function.Consumer.class, boolean.class},
              position, normalized, null, false, interpolation, -1, call(parameter, "bU_"), null, true);
        } finally { end(document); }
        return map("requestedPosition", position, "numerator", numerator, "denominator", denominator, "packed", packed, "point", attributes(point));
      }
      case "undoState": {
        Object message = call(document, "cuK");
        Object manager = call(message, "TcB");
        Object recorder = call(manager, "v33");
        return map("enabled", call(message, "FSQ"), "recorder", describe(recorder),
            "undoLabel", call(recorder, "XOR"), "redoLabel", call(recorder, "NKN"),
            "suppressed", call(call(recorder, "v33"), "NKN"));
      }
      case "armFixture": {
        boolean empty = Arrays.equals(args, new String[]{"EMPTY FOUR-TRACK FIXTURE"});
        boolean existing = args.length == 3 && args[0].equals("EXISTING DISPOSABLE FIXTURE");
        if (!empty && !existing) throw new IllegalArgumentException("Explicit fixture guard required");
        if (lanes.size() != 4) throw new IllegalStateException("Expected four selected fixture lanes");
        List<String> clipIds = new ArrayList<>(), noteIds = new ArrayList<>();
        for (Object lane : lanes) for (Object clip : clips(lane, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY)) {
          clipIds.add(identity(clip));
          for (Object note : notes(clip)) noteIds.add(identity(note));
        }
        Collections.sort(clipIds); Collections.sort(noteIds);
        if (empty && !clipIds.isEmpty()) throw new IllegalStateException("Fixture is not empty");
        if (existing && (!String.join(",", clipIds).equals(args[1]) || !String.join(",", noteIds).equals(args[2])))
          throw new IllegalStateException("Fixture identities differ from the explicit manifest");
        fixtureDocument = document;
        fixtureLanes = List.copyOf(lanes);
        return map("armed", true, "document", describe(document), "laneCount", lanes.size());
      }
      case "fixtureClip": {
        requireFixture(document, lanes);
        int index = Integer.parseInt(args[0]);
        double start = Double.parseDouble(args[1]), duration = Double.parseDouble(args[2]);
        if (start < 0 || duration <= 0 || !Double.isFinite(start + duration)) throw new IllegalArgumentException("Invalid bounds");
        Object lane = lanes.get(index);
        if (!clips(lane, start, start + duration).isEmpty()) throw new IllegalStateException("Occupied destination");
        begin(document, "create clip");
        Object clip;
        try { clip = invoke(call(lane, "oYp"), "qKA", new Class<?>[]{double.class, double.class}, start, duration); }
        finally { end(document); }
        return map("lane", index, "requested", List.of(start, duration), "actual", attributes(clip));
      }
      case "createOne": {
        requireFixture(document, lanes);
        Object clip = clips(lanes.get(Integer.parseInt(args[0])), Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY).get(Integer.parseInt(args[1]));
        int channel = Integer.parseInt(args[2]), key = Integer.parseInt(args[3]);
        double onset = Double.parseDouble(args[4]), duration = Double.parseDouble(args[5]), velocity = Double.parseDouble(args[6]);
        if (channel < 0 || channel > 15 || key < 0 || key > 127 || duration <= 0 || velocity < 0 || velocity > 1
            || !Double.isFinite(onset + duration + velocity)) throw new IllegalArgumentException("Invalid note");
        for (Object note : notes(clip)) if ((int)call(note, "yhG") == channel && (int)call(note, "v33") == key
            && number(call(note, "T1L")) < onset + duration && number(call(note, "IVb")) > onset)
          throw new IllegalStateException("Occupied destination");
        Object before = inventory(lanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
        begin(document, "create note");
        Object note;
        try { note = invoke(clip, "TUp", new Class<?>[]{int.class, int.class, double.class, double.class, double.class}, channel, key, onset, duration, velocity); }
        finally { end(document); }
        return map("requested", Arrays.asList(args), "note", snapshot(note), "before", before,
            "after", inventory(lanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY));
      }
      case "read": {
        Object note = findNote(lanes, args[0]);
        return snapshot(note);
      }
      case "updateOne": case "channelOne": case "keyOne": case "deleteOne": {
        requireFixture(document, lanes);
        Object note = findNote(lanes, args[0]);
        Object before = inventory(lanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
        Object row = call(note, "YeF");
        Object beforeNote = snapshot(note);
        begin(document, operation);
        try {
          if (operation.equals("deleteOne")) {
            invoke(row, "v33", new Class<?>[]{nativeClass(note, "oHd"), boolean.class}, note, false);
          } else if (operation.equals("channelOne") || operation.equals("keyOne")) {
            int channel = (int)call(note, "yhG"), key = (int)call(note, "v33");
            int desired = Integer.parseInt(args[1]);
            if (operation.equals("channelOne")) channel = desired; else key = desired;
            if (channel < 0 || channel > 15 || key < 0 || key > 127) throw new IllegalArgumentException("Invalid row");
            Object timeline = call(row, "hk_");
            Object destination = invoke(timeline, "tGW", new Class<?>[]{int.class, int.class, boolean.class}, channel, key, true);
            List<?> others = list(invoke(destination, "YLV", new Class<?>[]{double.class, double.class}, call(note, "T1L"), call(note, "IVb")));
            if (others.stream().anyMatch(other -> other != note)) throw new IllegalStateException("Occupied destination row");
            invoke(row, "v33", new Class<?>[]{nativeClass(note, "oHd"), boolean.class}, note, false);
            try { invoke(destination, "jaQ", new Class<?>[]{nativeClass(note, "oHd")}, note); }
            catch (Exception failure) { invoke(row, "jaQ", new Class<?>[]{nativeClass(note, "oHd")}, note); throw failure; }
          } else {
            if (!List.of("note_on_velocity", "note_off_velocity", "velocity_spread", "chance", "repeat_curve", "repeat_velocity_end", "repeat_velocity_curve").contains(args[1]))
              throw new IllegalArgumentException("Scalar probe field not admitted");
            double desired = Double.parseDouble(args[2]);
            if (!Double.isFinite(desired) || desired < 0 || desired > 1) throw new IllegalArgumentException("Invalid scalar");
            Object attribute = call(note, "Nln", String.class, args[1]);
            invoke(attribute, "setValue", new Class<?>[]{Object.class}, desired);
          }
        } finally { end(document); }
        return map("beforeNote", beforeNote, "afterNote", operation.equals("deleteOne") ? null : snapshot(note),
            "before", before, "after", inventory(lanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY));
      }
      case "clipMappingOne": {
        requireFixture(document, lanes);
        Object clip = findClip(lanes, args[0]);
        double value = Double.parseDouble(args[2]);
        if (!Double.isFinite(value) || value < 0) throw new IllegalArgumentException("Invalid mapping position");
        Object before = inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
        begin(document, "clip mapping");
        try {
          if (args[1].equals("offset")) invoke(clip, "UMc", new Class<?>[]{double.class}, value);
          else if (args[1].equals("loop")) {
            double duration = Double.parseDouble(args[3]);
            if (!Double.isFinite(duration) || duration <= 0) throw new IllegalArgumentException("Invalid loop duration");
            Object region = call(clip, "fvB");
            invoke(call(region, "Nln", String.class, "time"), "setValue", new Class<?>[]{Object.class}, value);
            invoke(call(region, "Nln", String.class, "duration"), "setValue", new Class<?>[]{Object.class}, duration);
            invoke(call(call(clip, "heu"), "Nln", String.class, "is_loop_enabled"), "setValue", new Class<?>[]{Object.class}, true);
          } else throw new IllegalArgumentException("Unknown mapping attribute");
        } finally { end(document); }
        return map("before", before, "after", inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY));
      }
      case "occurrences": {
        Object clip = findClip(lanes, args[0]);
        List<Object> occurrences = new ArrayList<>();
        for (Object note : notes(clip)) for (int iteration = 0; iteration < 4; iteration++) {
          Object occurrence = method(nativeClass(clip, "fOD"), "v33", nativeClass(clip, "mGg"), nativeClass(clip, "bT"), int.class, nativeClass(clip, "exV"))
              .invoke(null, note, clip, iteration, null);
          occurrences.add(map("sourceId", identity(note), "iteration", iteration,
              "relativeOnset", call(occurrence, "Ouu"), "projectOnset", call(occurrence, "QmM"),
              "projectEnd", call(occurrence, "qpF"), "visibleOnset", call(occurrence, "heu"), "visibleEnd", call(occurrence, "TcB")));
        }
        return map("clipId", identity(clip), "occurrences", occurrences);
      }
      case "arrangerOccurrences": return arrangerOccurrences(lanes);
      case "stageCloneClipRange": {
        stagedClip = null; stagedLane = null; stagedSource = null;
        requireFixture(document, lanes);
        Object source = findClip(lanes, args[0]);
        double start = Double.parseDouble(args[1]), stop = Double.parseDouble(args[2]);
        double placement = number(call(source, "T1L")), end = number(call(source, "IVb"));
        if (!Double.isFinite(start + stop) || start < placement || stop > end || stop <= start) throw new IllegalArgumentException("Range must lie within the fixture clip");
        Object before = arrangerOccurrences(fixtureLanes);
        Class<?> filterType = nativeClass(source, "com.bitwig.ramona.core.kid");
        Object clone = invoke(source, "v33", new Class<?>[]{filterType}, filterType.getField("EwU").get(null));
        boolean shared = call(clone, "FSQ") == call(source, "FSQ");
        if (shared) {
          Object independentContent = call(call(source, "FSQ"), "cu_");
          Object reference = call(call(clone, "Zeh", String.class, "content_timeline_reference"), "getValue");
          invoke(call(reference, "Zeh", String.class, "timeline"), "setValue", new Class<?>[]{Object.class}, independentContent);
        }
        if (call(clone, "FSQ") == call(source, "FSQ")) throw new IllegalStateException("Staged source content remains shared");
        double sourceStart = number(call(source, "dD_"));
        invoke(clone, "UMc", new Class<?>[]{double.class}, sourceStart + start - placement);
        invoke(clone, "YXc", new Class<?>[]{double.class}, sourceStart + stop - placement);
        invoke(clone, "FSQ", new Class<?>[]{double.class}, start);
        List<Object> sourceNotes = new ArrayList<>();
        for (Object note : notes(clone)) sourceNotes.add(snapshot(note));
        Object after = arrangerOccurrences(fixtureLanes);
        if (!json(before).equals(json(after))) throw new IllegalStateException("Staging changed live material");
        stagedClip = clone;
        stagedLane = call(source, "rwW");
        stagedSource = json(after);
        return map("contentWasSharedBeforeDetach", shared, "contentIsShared", call(clone, "FSQ") == call(source, "FSQ"),
            "beforeOccurrences", before, "afterOccurrences", after,
            "staged", map("attributes", attributes(clone), "sourceStart", call(clone, "dD_"), "loopEnabled", call(clone, "YeF"),
                "loopRegion", attributes(call(clone, "fvB")), "notes", sourceNotes));
      }
      case "stageRange": {
        stagedClip = null; stagedLane = null; stagedSource = null;
        requireFixture(document, lanes);
        int laneIndex = Integer.parseInt(args[0]);
        double start = Double.parseDouble(args[1]), stop = Double.parseDouble(args[2]);
        if (!Double.isFinite(start + stop) || start < 0 || stop <= start) throw new IllegalArgumentException("Invalid staged range");
        Object before = arrangerOccurrences(fixtureLanes);
        Object options = nativeClass(document, "iJo").getField("v33").get(null);
        Object lane = lanes.get(laneIndex);
        Object clip = invoke(call(lane, "gJp"), "v33", new Class<?>[]{double.class, double.class, nativeClass(document, "Hkh")}, start, stop, options);
        List<Object> source = new ArrayList<>();
        for (Object note : notes(clip)) source.add(snapshot(note));
        Object after = arrangerOccurrences(fixtureLanes);
        if (!json(before).equals(json(after))) throw new IllegalStateException("Staging changed live source notes");
        stagedClip = clip;
        stagedLane = lane;
        stagedSource = json(after);
        return map("beforeOccurrences", before, "afterOccurrences", after, "staged", map("attributes", attributes(clip),
            "mapping", attributes(call(clip, "heu")), "notes", source));
      }
      case "commitStaged": {
        requireFixture(document, lanes);
        if (!Arrays.equals(args, new String[]{"INSERT STAGED DISPOSABLE RANGE"}) || stagedClip == null || !lanes.contains(stagedLane))
          throw new IllegalStateException("No admitted staged fixture range");
        Object before = arrangerOccurrences(fixtureLanes);
        if (!stagedSource.equals(json(before))) throw new IllegalStateException("Live material changed after staging");
        begin(document, "insert staged clip");
        try { invoke(call(stagedLane, "gJp"), "YLV", new Class<?>[]{nativeClass(document, "gnS"), nativeClass(document, "exV")}, stagedClip, null); }
        finally { end(document); }
        Object after = arrangerOccurrences(fixtureLanes);
        stagedClip = null;
        return map("beforeOccurrences", before, "afterOccurrences", after);
      }
      case "insertOverRange": {
        requireFixture(document, lanes);
        if (args.length != 4 || !args[3].equals("REPLACE DISPOSABLE CLIP RANGE")) throw new IllegalArgumentException("Explicit fixture replacement guard required");
        int laneIndex = Integer.parseInt(args[0]);
        double start = Double.parseDouble(args[1]), duration = Double.parseDouble(args[2]);
        if (!Double.isFinite(start + duration) || start < 0 || duration <= 0) throw new IllegalArgumentException("Invalid clip interval");
        Object before = inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
        Object beforeOccurrences = arrangerOccurrences(fixtureLanes);
        begin(document, "replace clip range");
        Object clip;
        try {
          clip = invoke(call(lanes.get(laneIndex), "oYp"), "qKA", new Class<?>[]{double.class, double.class}, start, duration);
          invoke(clip, "TUp", new Class<?>[]{int.class,int.class,double.class,double.class,double.class}, 15, 90, 0.125, 0.25, 0.67);
        } finally { end(document); }
        return map("range", List.of(start, start + duration), "replacementClip", identity(clip),
            "before", before, "after", inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY),
            "beforeOccurrences", beforeOccurrences, "afterOccurrences", arrangerOccurrences(fixtureLanes));
      }
      case "expressionPoint": {
        requireFixture(document, lanes);
        Object note = findNote(lanes, args[0]);
        double time = Double.parseDouble(args[2]), value = Double.parseDouble(args[3]), spread = Double.parseDouble(args[4]);
        if (time < 0 || !Double.isFinite(time + value + spread) || spread < 0) throw new IllegalArgumentException("Invalid expression point");
        Object kind = Arrays.stream(nativeClass(note, "ZkX").getEnumConstants()).filter(item -> item.toString().equals(args[1])).findFirst().orElseThrow();
        Object before = snapshot(note);
        begin(document, "expression point");
        try {
          Object curve = invoke(call(note, "Vdp"), "v33", new Class<?>[]{nativeClass(note, "ZkX"), boolean.class}, kind, true);
          invoke(curve, "v33", new Class<?>[]{double.class, double.class, double.class}, time, value, spread);
        } finally { end(document); }
        return map("beforeNote", before, "afterNote", snapshot(note));
      }
      case "replaceRange": {
        requireFixture(document, lanes);
        if (!Arrays.equals(args, new String[]{"REPLACE DISPOSABLE RANGE"})) throw new IllegalArgumentException("Explicit replacement guard required");
        if (to - from < 1) throw new IllegalStateException("Use a fixture range at least one beat long");
        List<Object> affected = new ArrayList<>();
        List<Object[]> destinations = new ArrayList<>();
        for (Object lane : lanes) for (Object clip : clips(lane, from, to)) {
          double placement = number(call(call(clip, "Nln", String.class, "time"), "getValue"));
          double length = number(call(call(clip, "Nln", String.class, "duration"), "getValue"));
          if (!(placement == 0 && length == 80 || placement == 84 && length == 8))
            throw new IllegalStateException("Only the isolated straight fixture mapping is established");
          double localFrom = Math.max(0, from - placement), localTo = Math.min(length, to - placement);
          if (localTo - localFrom < 1) throw new IllegalStateException("Clip intersection is too short for this probe");
          for (Object note : notes(clip)) {
            double onset = number(call(note, "T1L")), end = number(call(note, "IVb"));
            if (onset < localTo && end > localFrom) {
              if (onset < localFrom || end > localTo) throw new IllegalStateException("Boundary splitting not yet established; no writes admitted");
              affected.add(note);
            }
          }
          destinations.add(new Object[]{clip, localFrom});
        }
        Object before = inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY);
        List<Object> created = new ArrayList<>();
        begin(document, "replace range");
        try {
          for (Object note : affected) invoke(call(note, "YeF"), "v33", new Class<?>[]{nativeClass(note, "oHd"), boolean.class}, note, false);
          for (Object[] destination : destinations) for (int i = 0; i < 2; i++) {
            Object note = invoke(destination[0], "TUp", new Class<?>[]{int.class, int.class, double.class, double.class, double.class},
                i == 0 ? 15 : 1, 80 + i, (double)destination[1] + 0.125 + i * 0.375, 0.25, 0.61 + i * 0.1);
            created.add(snapshot(note));
          }
        } finally { end(document); }
        return map("from", from, "to", to, "removedIds", affected.stream().map(note -> {
          try { return identity(note); } catch (Exception error) { throw new IllegalStateException(error); }
        }).toList(), "created", created, "before", before,
            "after", inventory(fixtureLanes, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY));
      }
      default: throw new IllegalArgumentException("Unimplemented probe: " + operation);
    }
  }

  private static Class<?> nativeClass(Object object, String name) throws ClassNotFoundException {
    return Class.forName(name, true, object.getClass().getClassLoader());
  }
  private static String identity(Object object) throws Exception { return call(call(object, "Nln", String.class, "managed_object_ID"), "getValue").toString(); }
  private static Object findClip(List<?> lanes, String id) throws Exception {
    for (Object lane : lanes) for (Object clip : clips(lane, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY)) if (identity(clip).equals(id)) return clip;
    throw new IllegalStateException("Source clip not found: " + id);
  }
  private static Object findNote(List<?> lanes, String id) throws Exception {
    Object found = null;
    for (Object lane : lanes) for (Object clip : clips(lane, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY)) for (Object note : notes(clip)) {
      if (identity(note).equals(id)) {
        if (found != null && found != note) throw new IllegalStateException("Ambiguous identity");
        found = note;
      }
    }
    if (found == null) throw new IllegalStateException("Source note not found: " + id);
    return found;
  }
  private static void requireFixture(Object document, List<?> lanes) {
    if (document != fixtureDocument || lanes.isEmpty() || !fixtureLanes.containsAll(lanes)) throw new IllegalStateException("Not the armed disposable fixture");
  }
  private static void begin(Object document, String name) throws Exception {
    invoke(document, "v33", new Class<?>[]{String.class, boolean.class}, "Notes Transformer probe: " + name, false);
  }
  private static void end(Object document) throws Exception {
    invoke(document, "v33", new Class<?>[]{boolean.class}, false);
  }
  private static List<?> clips(Object lane, double from, double to) throws Exception {
    try (Stream<?> stream = (Stream<?>) invoke(lane, "v33", new Class<?>[]{double.class, double.class}, from, to)) { return stream.toList(); }
  }
  private static List<?> notes(Object clip) throws Exception {
    return list(invoke(call(clip, "qKA"), "v33", new Class<?>[]{double.class, double.class}, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY));
  }
  private static Object arrangerOccurrences(List<?> lanes) throws Exception {
    List<Object> result = new ArrayList<>();
    for (Object lane : lanes) for (Object clip : clips(lane, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY)) {
      double duration = number(call(clip, "GaO"));
      Object mapping = call(clip, "heu");
      int lastIteration = Boolean.TRUE.equals(call(clip, "YeF"))
          ? Math.max(0, (int)invoke(mapping, "TUp", new Class<?>[]{double.class, boolean.class}, duration, true)) : 0;
      for (Object note : notes(clip)) for (int iteration = 0; iteration <= lastIteration; iteration++) {
        Object occurrence = method(nativeClass(clip, "fOD"), "v33", nativeClass(clip, "mGg"), nativeClass(clip, "bT"), int.class, nativeClass(clip, "exV"))
            .invoke(null, note, clip, iteration, null);
        double onset = number(call(occurrence, "heu")), end = number(call(occurrence, "TcB"));
        if (end > onset) result.add(map("clipId", identity(clip), "sourceId", identity(note), "iteration", iteration,
            "nominalOnset", call(occurrence, "QmM"), "nominalEnd", call(occurrence, "qpF"),
            "visibleOnset", onset, "visibleEnd", end, "note", snapshot(note)));
      }
    }
    return result;
  }
  private static Object inventory(List<?> lanes, double from, double to) throws Exception {
    List<Object> contents = new ArrayList<>();
    for (Object lane : lanes) {
      List<Object> events = new ArrayList<>();
      for (Object clip : clips(lane, from, to)) {
        Object reference = call(call(clip, "Zeh", String.class, "content_timeline_reference"), "getValue");
        Object content = call(call(reference, "Zeh", String.class, "timeline"), "getValue");
        List<Object> source = new ArrayList<>();
        for (Object note : notes(clip)) source.add(snapshot(note));
        Object mapping = call(clip, "heu");
        List<Object> samples = new ArrayList<>();
        for (double elapsed : new double[]{0, 1, 5, 64}) samples.add(map("elapsedBeats", elapsed,
            "sourceBeats", invoke(mapping, "v33", new Class<?>[]{double.class, boolean.class}, elapsed, false),
            "loopIteration", invoke(mapping, "TUp", new Class<?>[]{double.class, boolean.class}, elapsed, false)));
        events.add(map("object", describe(clip), "attributes", attributes(clip), "reference", attributes(reference),
            "content", attributes(content), "notes", source, "mapping", map("object", describe(mapping), "attributes", attributes(mapping),
            "sourceStart", call(clip, "dD_"), "loopEnabled", call(clip, "YeF"), "loopRegion", attributes(call(clip, "fvB")), "samples", samples)));
      }
      contents.add(map("object", describe(lane), "clips", events));
    }
    return map("from", from, "to", to, "lanes", contents,
        "interpretation", "Source inventory; occurrence mapping and boundary semantics are not yet established");
  }
  private static Object snapshot(Object note) throws Exception {
    Class<?> options = Class.forName("com.bitwig.ramona.serial.CRe", true, note.getClass().getClassLoader());
    byte[] bytes = (byte[])invoke(note, "jaQ", new Class<?>[]{options}, options.getField("YLV").get(null));
    return map("object", describe(note), "key", call(note, "v33"), "channel", call(note, "yhG"),
        "onset", call(note, "T1L"), "end", call(note, "IVb"), "attributes", attributes(note),
        "serialized", Base64.getEncoder().encodeToString(bytes), "expressions", expressions(note));
  }
  private static Object expressions(Object note) throws Exception {
    Map<String, Object> result = new LinkedHashMap<>();
    Class<?> kindType = nativeClass(note, "ZkX");
    for (Object kind : kindType.getEnumConstants()) {
      Object curve = invoke(call(note, "Vdp"), "v33", new Class<?>[]{kindType, boolean.class}, kind, false);
      List<Object> points = new ArrayList<>();
      if (curve != null) for (Object point : list(call(curve, "v33"))) points.add(attributes(point));
      result.put(kind.toString(), points);
    }
    return result;
  }
  private static Map<String, Object> attributes(Object object) throws Exception {
    Map<String, Object> result = new LinkedHashMap<>();
    for (Object attribute : list(call(object, "RR"))) {
      Object value = call(attribute, "getValue");
      result.put((String)call(attribute, "zeN"), value == null || value instanceof Number || value instanceof Boolean ? value : value.toString());
    }
    return result;
  }
  private static Object describe(Object object) {
    return object == null ? null : map("type", object.getClass().getName(), "path", object.toString());
  }
  private static Method method(Class<?> type, String name, Class<?>... signature) throws NoSuchMethodException {
    for (Class<?> candidate = type; candidate != null; candidate = candidate.getSuperclass()) {
      try { Method method = candidate.getDeclaredMethod(name, signature); method.setAccessible(true); return method; }
      catch (NoSuchMethodException ignored) { }
    }
    throw new NoSuchMethodException(type.getName() + "." + name + Arrays.toString(signature));
  }
  private static Object call(Object object, String name) throws Exception { return invoke(object, name, new Class<?>[0]); }
  private static Object call(Object object, String name, Class<?> type, Object argument) throws Exception { return invoke(object, name, new Class<?>[]{type}, argument); }
  private static Object invoke(Object object, String name, Class<?>[] types, Object... args) throws Exception {
    Method selected = method(object.getClass(), name, types);
    try { return selected.invoke(object, args); }
    catch (IllegalArgumentException error) {
      throw new IllegalArgumentException(selected + " arguments=" + Arrays.stream(args).map(value -> value == null ? "null" : value.getClass().getName()).toList(), error);
    }
  }
  private static List<?> list(Object object) { return (List<?>)object; }
  private static double number(Object object) { return ((Number)object).doubleValue(); }
  private static Map<String, Object> map(Object... pairs) {
    Map<String, Object> result = new LinkedHashMap<>();
    for (int i = 0; i < pairs.length; i += 2) result.put((String)pairs[i], pairs[i + 1]);
    return result;
  }
  private static String json(Object value) {
    if (value == null) return "null";
    if (value instanceof Number) return Double.isFinite(((Number)value).doubleValue()) ? value.toString() : "null";
    if (value instanceof Boolean) return value.toString();
    if (value instanceof Map<?, ?>) {
      List<String> fields = new ArrayList<>();
      ((Map<?, ?>)value).forEach((key, item) -> fields.add(json(key.toString()) + ":" + json(item)));
      return "{" + String.join(",", fields) + "}";
    }
    if (value instanceof Iterable<?>) {
      List<String> items = new ArrayList<>();
      for (Object item : (Iterable<?>)value) items.add(json(item));
      return "[" + String.join(",", items) + "]";
    }
    StringBuilder result = new StringBuilder("\"");
    for (char character : value.toString().toCharArray()) {
      switch (character) {
        case '"': result.append("\\\""); break;
        case '\\': result.append("\\\\"); break;
        case '\n': result.append("\\n"); break;
        case '\r': result.append("\\r"); break;
        case '\t': result.append("\\t"); break;
        default: if (character < 32) result.append(String.format("\\u%04x", (int)character)); else result.append(character);
      }
    }
    return result.append('"').toString();
  }
}
