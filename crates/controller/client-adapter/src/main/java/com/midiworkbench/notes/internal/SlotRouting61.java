package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

/** Opinionated slot routing using built-in devices, never user presets. */
final class SlotRouting61 {
    static final String FILTER="c5a1bb2d-a589-4fda-b3cf-911cfd6297be";
    static final String MAP="0f003fa3-adcc-4684-81f7-f0e11c09c5b4";
    private SlotRouting61() {}
    static Object stock(ClassLoader loader,String id)throws Exception {
        return Class.forName("com.bitwig.flt.document.core.master.device.zfx",true,loader)
            .getMethod("tGW",UUID.class).invoke(null,UUID.fromString(id));
    }
    static boolean helper(String id){return FILTER.equals(id) || MAP.equals(id);}
    static Map<String,Object> controls(Object device)throws Exception {
        Map<String,Object> result=new LinkedHashMap<>();
        result.put("On",ProjectAdapter61.value(device,"enabled"));
        Object contents=Reflection.call(device,"AtZ");
        Class<?> atom=Class.forName("bXo",true,device.getClass().getClassLoader());
        String prefix=FILTER.equals(Reflection.call(device,"v33").toString())?"SELECT_CHANNEL_":"CHANNEL_";
        for(Object member:(List<?>)Reflection.call(contents,"qpF"))if(atom.isInstance(member)) {
            String key=ProjectAdapter61.value(member,"preset_identifier").toString();
            for(int c=1;c<=16;c++)if(key.equals(prefix+c))result.put("CHANNEL_"+c,member);
        }
        if(result.size()!=17)throw new IllegalStateException("Stock device missing its 16 channel controls");
        return result;
    }
    static void read(Object device,ObjectNode observation)throws Exception {
        String id=Reflection.call(device,"v33").toString();if(!helper(id))return;
        Map<String,Object> controls=controls(device);
        observation.put("routingEnabled",Boolean.TRUE.equals(ProjectAdapter61.value(controls.get("On"),"value")));
        observation.put("routingActive",Boolean.TRUE.equals(ProjectAdapter61.value(device,"active")));
        var values=observation.putArray(FILTER.equals(id)?"channelFilter":"channelMap");
        for(int c=1;c<=16;c++) {
            Object v=ProjectAdapter61.value(controls.get("CHANNEL_"+c),"value");
            if(FILTER.equals(id))values.add((Boolean)v);else values.add(((Number)v).intValue());
        }
    }
    record Prepared(Object filter,Object map) {}
    static Prepared prepare(Object slot,ClassLoader loader)throws Exception {
        Object filter=null,map=null;
        List<?> devices=slot==null?List.of():(List<?>)Reflection.call(Reflection.call(slot,"PPp"),"ATl");
        for(Object device:devices) {
            String id=Reflection.call(device,"v33").toString();
            if(FILTER.equals(id)){if(filter!=null)throw new IllegalArgumentException("Multiple Channel Filters in target slot");filter=device;}
            if(MAP.equals(id)){if(map!=null)throw new IllegalArgumentException("Multiple Channel Maps in target slot");map=device;}
        }
        boolean addFilter=filter==null,addMap=map==null;
        if(addFilter)filter=stock(loader,FILTER);
        if(addMap)map=stock(loader,MAP);
        // Resolve the complete fixed parameter vocabulary before touching the document.
        controls(filter);controls(map);
        return new Prepared(filter,map);
    }
    static void apply(Object slot,Prepared prepared,JsonNode channels)throws Exception {
        ClassLoader loader=slot.getClass().getClassLoader();Object chain=Reflection.call(slot,"PPp");
        Class<?> deviceType=Class.forName("com.bitwig.flt.document.core.master.device.ysX",true,loader);
        place(chain,deviceType,prepared.filter(),0);
        place(chain,deviceType,prepared.map(),1);
        Set<Integer> enabled=new java.util.HashSet<>();enabled.add(0);for(JsonNode c:channels)enabled.add(c.intValue());
        configure(prepared.filter(),enabled);configure(prepared.map(),enabled);
    }
    private static void place(Object chain,Class<?> type,Object device,int index)throws Exception {
        List<?> devices=(List<?>)Reflection.call(chain,"ATl");
        int current=devices.indexOf(device);if(current==index)return;
        if(current>=0)Reflection.call(chain,"v33",new Class<?>[]{type,boolean.class},device,false);
        Reflection.call(chain,"v33",new Class<?>[]{type,int.class},device,index);
    }
    private static void configure(Object device,Set<Integer> enabled)throws Exception {
        Map<String,Object> controls=controls(device);set(controls.get("On"),true);
        Reflection.call(Reflection.call(device,"Nln",new Class<?>[]{String.class},"active"),"setValue",new Class<?>[]{Object.class},true);
        boolean filter=FILTER.equals(Reflection.call(device,"v33").toString());
        for(int c=0;c<16;c++)set(controls.get("CHANNEL_"+(c+1)),filter?Boolean.valueOf(enabled.contains(c)):Integer.valueOf(1));
    }
    private static void set(Object parameter,Object value)throws Exception {
        Reflection.call(Reflection.call(parameter,"Nln",new Class<?>[]{String.class},"value"),"setValue",new Class<?>[]{Object.class},value);
    }
}
