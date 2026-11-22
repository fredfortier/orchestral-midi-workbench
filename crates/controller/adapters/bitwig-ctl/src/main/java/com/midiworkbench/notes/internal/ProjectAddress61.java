package com.midiworkbench.notes.internal;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
/** Native path containment and exact sibling-name resolution. */
final class ProjectAddress61 {
 private ProjectAddress61() {}
 private static final Set<String> CHAINS=Set.of("layerChain","choice");
    static ObjectNode trackPath(JsonNode path) {
        validate(path);ObjectNode result=((ObjectNode)path).deepCopy();
        var elements=result.putArray("elements");boolean found=false;
        for(JsonNode element:path.required("elements")) {
            String kind=element.required("kind").asText();
            if(!kind.equals("group") && !kind.equals("track"))break;
            elements.add(element.deepCopy());found=true;
        }
        if(!found)throw new IllegalArgumentException("E_ADDRESS: receiver has no owning track");
        return result;
    }
    static ObjectNode qualified(JsonNode path,Object document)throws Exception {
        ObjectNode result=((ObjectNode)path).deepCopy();result.putArray("elements");
        for(JsonNode element:path.required("elements")) {
            result.withArray("elements").add(element.deepCopy());Object owner=resolveNative(result,document,document.getClass().getClassLoader());
            ((ObjectNode)result.withArray("elements").get(result.withArray("elements").size()-1)).put("nativeId",ProjectAdapter61.value(owner,"managed_object_ID").toString());
        }
        return result;
    }
    /** Resolve only named native owners; never inspect note sources or plugin byte payloads. */
    static Object resolveNative(JsonNode path,Object document,ClassLoader loader) throws Exception {
        validate(path);
        if(!path.required("project").asText().equals(ProjectAdapter61.value(document,"name")))
            throw new IllegalArgumentException("E_ADDRESS_PROJECT: project name differs");
        Object owner=Reflection.call(document,"peG");
        for(JsonNode element:path.required("elements")) {
            String kind=element.required("kind").asText(),name=element.required("name").asText();
            List<Object> matches=new ArrayList<>();
            if(kind.equals("group") || kind.equals("track")) {
                String expected=kind.equals("group")?"com.bitwig.flt.document.core.master.stI":"yyk";
                List<Object> candidates=new ArrayList<>((List<?>)Reflection.call(owner,"IFc"));
                if(kind.equals("track"))candidates.addAll((List<?>)Reflection.call(owner,"UQt"));
                for(Object candidate:candidates)
                    if(candidate.getClass().getName().equals(expected) && name.equals(ProjectAdapter61.value(candidate,"displayed_title")))matches.add(candidate);
            } else if(kind.equals("clip")) {
                Object clipOwner=owner.getClass().getName().equals("com.bitwig.flt.document.core.master.stI")?Reflection.call(owner,"DQP"):owner;
                Object lane=ProjectAdapter61.value(clipOwner,"main_note_clip_lane");
                try(var stream=(java.util.stream.Stream<?>)Reflection.call(lane,"v33",new Class<?>[]{double.class,double.class},Double.NEGATIVE_INFINITY,Double.POSITIVE_INFINITY)) {
                    var iterator=stream.iterator();while(iterator.hasNext()){Object candidate=iterator.next();if(candidate.getClass().getName().equals("UMj") && name.equals(ProjectAdapter61.value(candidate,"displayed_title")))matches.add(candidate);}
                }
            } else if(kind.equals("automationLane")) {
                Object track=owner.getClass().getName().equals("com.bitwig.flt.document.core.master.stI")?Reflection.call(owner,"DQP"):owner;
                for(Object candidate:Entity61.automationLanes(track))if(name.equals(Entity61.laneName(candidate)))matches.add(candidate);
            } else if(CHAINS.contains(kind)) {
                Object container=Reflection.call(Reflection.call(owner,"AtZ"),"v33",new Class<?>[]{String.class},kind.equals("layerChain")?"CHAIN_LIST":"CHAIN_SELECTOR");
                if(container==null)throw new IllegalArgumentException("E_ADDRESS: missing native chain owner");
                for(Object candidate:(List<?>)ProjectAdapter61.value(container,"nested_device_chains"))
                    if(name.equals(ProjectAdapter61.value(candidate,"displayed_title")))matches.add(candidate);
            } else {
                Object chain=Reflection.call(owner,owner.getClass().getName().equals("com.bitwig.flt.document.core.master.device.kdt")?"PPp":"Vdp");
                for(Object candidate:(List<?>)Reflection.call(chain,"ATl")) {
                    String display=String.valueOf(ProjectAdapter61.value(candidate,"user_device_name"));
                    if(display.isEmpty())display=String.valueOf(ProjectAdapter61.value(candidate,"device_name"));
                    if(!name.equals(display))continue;
                    if(kind.equals("device")){matches.add(candidate);}
                    else if(kind.equals("plugin")) {
                        PluginIdentity61 plugin=PluginIdentity61.read(ProjectAdapter61.value(candidate,"device_contents"),
                                Reflection.call(candidate,"v33").toString());
                        if(plugin!=null && element.required("id").asText().equals(plugin.id())
                            && element.required("version").asText().equals(plugin.version()))matches.add(candidate);
                    } else {
                        String expected=switch(kind) {
                            case "layer" -> "5024be2e-65d6-4d40-bbfe-8b2ea993c445";
                            case "selector" -> "9588fbcf-721a-438b-8555-97e4231f7d2c";
                            default -> throw new IllegalArgumentException("E_ADDRESS: unsupported native target");
                        };
                        if(expected.equals(Reflection.call(candidate,"v33").toString()))matches.add(candidate);
                    }
                }
            }
            if(element.has("nativeId")) {
                List<Object> identified=new ArrayList<>();
                for(Object candidate:matches)if(element.required("nativeId").asText().equals(ProjectAdapter61.value(candidate,"managed_object_ID").toString()))identified.add(candidate);
                matches=identified;
            }
            if(matches.size()!=1)throw new IllegalArgumentException("E_ADDRESS_MATCH: expected one "+element+", found "+matches.size());
            owner=matches.get(0);
        }
        return path.required("elements").isEmpty()?document:owner;
    }
    /** Native containment gives the path; no exported object graph is involved. */
    static ObjectNode nativeTrackPath(Object document,Object track) throws Exception {
        ObjectNode path=com.midiworkbench.notes.Wire.object();path.put("project",ProjectAdapter61.value(document,"name").toString());
        var elements=path.putArray("elements");
        if(!findTrack(Reflection.call(document,"peG"),track,elements))throw new IllegalArgumentException("Track is outside project hierarchy");
        return path;
    }
    private static boolean findTrack(Object parent,Object wanted,com.fasterxml.jackson.databind.node.ArrayNode elements) throws Exception {
        for(Object track:(List<?>)Reflection.call(parent,"IFc")) {
            boolean group=track.getClass().getName().equals("com.bitwig.flt.document.core.master.stI");
            elements.addObject().put("kind",group?"group":"track").put("name",ProjectAdapter61.value(track,"displayed_title").toString()).put("nativeId",ProjectAdapter61.value(track,"managed_object_ID").toString());
            if(track==wanted || (group && Reflection.call(track,"DQP")==wanted))return true;
            if(group && findTrack(track,wanted,elements))return true;
            elements.remove(elements.size()-1);
        }
        for(Object track:(List<?>)Reflection.call(parent,"UQt"))if(track==wanted) {
            elements.addObject().put("kind","track").put("name",ProjectAdapter61.value(track,"displayed_title").toString()).put("nativeId",ProjectAdapter61.value(track,"managed_object_ID").toString());return true;
        }
        return false;
    }
    static boolean matches(JsonNode wanted,JsonNode actual) {
        if(!wanted.required("project").equals(actual.required("project")) || wanted.required("elements").size()!=actual.required("elements").size())return false;
        for(int i=0;i<wanted.required("elements").size();i++) {
            ObjectNode observed=actual.required("elements").get(i).deepCopy();
            JsonNode requested=wanted.required("elements").get(i);
            if(!requested.has("nativeId"))observed.remove("nativeId");
            if(!requested.equals(observed))return false;
        }
        return true;
    }
    static void validate(JsonNode path) {
        if (!path.isObject() || path.size() != 2 || !validName(path.path("project"), 128)
            || !path.path("elements").isArray() || path.required("elements").size() > 64)
            throw new IllegalArgumentException("E_ADDRESS: invalid named project/path bounds");
        boolean track = false;
        String previous = "";
        for (JsonNode element : path.required("elements")) {
            String kind = element.path("kind").asText();
            if (!element.isObject() || element.size() != (kind.equals("plugin") ? 4 : 2) + (element.has("nativeId")?1:0) || !validName(element.path("name"), 256))
                throw new IllegalArgumentException("E_ADDRESS: invalid named element");
            if(element.has("nativeId") && !validName(element.required("nativeId"),128))throw new IllegalArgumentException("E_ADDRESS: invalid native identity discriminator");
            boolean valid = switch (kind) {
                case "group", "track" -> !track && (previous.isEmpty() || previous.equals("group"));
                case "layerChain" -> previous.equals("layer");
                case "choice" -> previous.equals("selector");
                case "clip", "automationLane" -> previous.equals("track") || previous.equals("group");
                case "device", "layer", "selector", "plugin" -> (track || previous.equals("group")) && (previous.equals("group") || previous.equals("track") || CHAINS.contains(previous));
                default -> false;
            };
            if (!valid) throw new IllegalArgumentException("E_ADDRESS: invalid group/track/device hierarchy");
            if (kind.equals("plugin") && (!validName(element.path("id"), 256) || !validName(element.path("version"), 128)))
                throw new IllegalArgumentException("E_ADDRESS: exact plugin ID/version required");
            track |= kind.equals("track");
            previous = kind;
        }
    }
    private static boolean validName(JsonNode node, int max) {
        return node.isTextual() && !node.asText().isBlank() && node.asText().indexOf('\0') < 0
            && node.asText().getBytes(StandardCharsets.UTF_8).length <= max;
    }
}
