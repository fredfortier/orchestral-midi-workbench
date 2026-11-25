package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.List;

import static com.midiworkbench.notes.internal.Reflection.call;
import static com.midiworkbench.notes.internal.ProjectAdapter61.value;

/** Native note-slot selection and creation on the admitted document thread. */
final class Launcher61 {
    private Launcher61() {}

    static Object selection(Object proxy) throws Exception {
        Object widget = call(proxy,"getDocumentWidget");
        Object wrapper = call(proxy,"getDocumentGUIWrapper");
        if (widget == null || wrapper == null) return null;
        Object launcher = call(wrapper,"NC3");
        return value(widget,"inspector_selection") == launcher ? call(launcher,"qKA") : null;
    }

    static Object selectedSlot(Object selection) throws Exception {
        if (selection == null) return null;
        List<?> members = (List<?>)call(selection,"jaQ");
        return members.size() == 1 && isNoteSlot(members.getFirst()) ? members.getFirst() : null;
    }

    private static boolean isNoteSlot(Object slot) throws Exception {
        return slot != null && Class.forName("oKx",false,slot.getClass().getClassLoader()).isInstance(slot);
    }

    static Object slot(Object track,int scene) throws Exception {
        Object slots = call(track,"DSu");
        if (slots == null || scene < 0 || scene >= ((Number)call(slots,"et_")).intValue()) return null;
        Object slot = call(track,"GTj",new Class<?>[]{int.class},scene);
        return isNoteSlot(slot) ? slot : null;
    }

    static Object noteSlot(Object clip) throws Exception {
        return call(clip,"dA_");
    }

    static Object clip(Object slot) throws Exception {
        return slot == null ? null : call(slot,"qKA");
    }

    static int scene(Object slot) throws Exception {
        return ((Number)call(slot,"Zlo")).intValue();
    }

    static String clipName(Object clip,int scene) throws Exception {
        String title=value(clip,"displayed_title").toString();
        // A new unnamed clip receives its displayed slot label at the end of the event.
        return title.isBlank() ? "S"+(scene+1) : title;
    }

    static ObjectNode clipPath(Object document,Object track,Object clip) throws Exception {
        ObjectNode path = ProjectAddress61.nativeTrackPath(document,track);
        Object slot = noteSlot(clip);
        ObjectNode element = path.withArray("elements").addObject()
                .put("kind",slot == null ? "clip" : "launcherClip")
                .put("name",slot==null ? value(clip,"displayed_title").toString() : clipName(clip,scene(slot)))
                .put("nativeId",id(clip));
        if (slot != null) element.put("scene",scene(slot));
        return path;
    }

    private static String id(Object owner) throws Exception {
        return value(owner,"managed_object_ID").toString();
    }

    static ObjectNode observe(Object document,String generation,Object slot) throws Exception {
        ObjectNode result = Wire.object().put("document",generation);
        result.putNull("position");
        if (slot != null) {
            Object track = call(slot,"eu_");
            ObjectNode position = result.putObject("position").put("scene",scene(slot)).put("slot",id(slot));
            position.set("track",ProjectAddress61.nativeTrackPath(document,track));
            Object clip = clip(slot);
            if (clip == null) position.putNull("clip");
            else position.set("clip",clipPath(document,track,clip));
        }
        result.put("revision",ObservedScope.revision(result));
        return result;
    }

    static void preflight(ObjectNode request,ObjectNode before) {
        if (!request.has("expected") || !request.has("beats") || request.size() != 3
                || !request.required("expected").equals(before)
                || before.required("position").isNull()
                || !before.required("position").required("clip").isNull()
                || !request.required("beats").isIntegralNumber()
                || !request.required("beats").canConvertToLong()
                || request.required("beats").longValue() < 1 || request.required("beats").longValue() > 0xffff_ffffL)
            throw new IllegalArgumentException("Creation requires the current empty launcher note slot and a positive unsigned 32-bit beat length");
    }

    static void create(Object slot,long beats) throws Exception {
        call(slot,"v33",new Class<?>[]{double.class},(double)beats);
    }

    static void verify(Object slot,long beats) throws Exception {
        Object created = clip(slot);
        if (created == null || noteSlot(created) != slot
                || ((Number)value(call(created,"fvB"),"duration")).doubleValue() != beats
                || ((Number)call(created,"Fgm")).doubleValue() != beats)
            throw new IllegalStateException("Native launcher clip creation readback differs");
    }
}
