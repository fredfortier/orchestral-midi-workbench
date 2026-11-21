package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Scoped device parameters and mapped Remote Controls, never an object-graph export. */
final class DeviceParameters61 {
    private DeviceParameters61() {}
    private static Object value(Object owner,String field)throws Exception{return ProjectAdapter61.value(owner,field);}
    private static Object call(Object owner,String method)throws Exception{return Reflection.call(owner,method);}
    private static String id(Object owner)throws Exception{return value(owner,"managed_object_ID").toString();}
    private static List<?> remotePages(Object device)throws Exception {
        Object remotes=value(device,"remote_controls");
        try(var pages=(java.util.stream.Stream<?>)call(remotes,"cQ_")) {
            List<?> result=pages.limit(257).toList();
            if(result.size()>256)throw new IllegalStateException("Remote Controls page limit exceeded");
            return result;
        }
    }
    private static void add(Map<String,Object> parameters,Object candidate,Class<?> atom)throws Exception {
        if(candidate!=null && atom.isInstance(candidate)) {
            if(parameters.size()>=100000)throw new IllegalStateException("Device parameter limit exceeded");
            parameters.put(id(candidate),candidate);
        }
    }
    static Map<String,Object> parameters(Object device)throws Exception {
        Map<String,Object> result=new LinkedHashMap<>();Class<?> atom=Class.forName("bXo",true,device.getClass().getClassLoader());
        Object contents=value(device,"device_contents");
        // Inspect only this device's immediate parameter containers. References
        // and nested devices are not recursively exported or silently retargeted.
        for(Object owner:List.of(device,contents))for(Object attribute:(List<?>)call(owner,"RR")) {
            Object item=call(attribute,"getValue");add(result,item,atom);
            if(item instanceof List<?> list)for(Object member:list)add(result,member,atom);
        }
        for(Object page:remotePages(device))for(Object slot:(List<?>)call(page,"AxB"))add(result,call(slot,"XQF"),atom);
        return result;
    }
    static ObjectNode target(Object parameter,JsonNode device)throws Exception {
        ObjectNode result=Wire.object();result.put("kind","parameter");result.set("device",device.deepCopy());
        result.put("nativeParameterId",id(parameter));result.put("name",String.valueOf(value(parameter,"name")));
        Object physical=value(parameter,"value_type");
        for(String field:List.of("min","max","domain","unit"))result.set(field,Wire.JSON.valueToTree(value(physical,field)));
        result.put("pointType",physical.getClass().getName().equals("am")?"continuous":"unsupported");
        return result;
    }
    static ObjectNode read(Object device,JsonNode path)throws Exception {
        ObjectNode result=Wire.object();ArrayNode rows=result.putArray("parameters");
        for(Object parameter:parameters(device).values()) {
            Object physical=value(parameter,"value_type");
            if(!physical.getClass().getName().equals("am"))continue;
            ObjectNode row=rows.addObject();row.set("target",target(parameter,path));
            row.set("value",Wire.JSON.valueToTree(value(parameter,"value")));
            row.put("automatable",Boolean.TRUE.equals(call(parameter,"AZY")));
            if("a0b8f27a-128e-4f72-b9fc-a277060b87ee".equals(call(device,"v33").toString())) {
                String key=String.valueOf(value(parameter,"preset_identifier"));
                if(key.matches("P[1-8]")) {
                    Object assignment=Reflection.call(value(device,"device_contents"),"v33",new Class<?>[]{String.class},"CC"+key.substring(1));
                    row.set("midiController",Wire.JSON.valueToTree(value(assignment,"value")));
                }
            }
        }
        ArrayNode remotes=result.putArray("remoteControls");
        for(Object page:remotePages(device))for(Object slot:(List<?>)call(page,"AxB")) {
            Object parameter=call(slot,"XQF");if(parameter==null)continue;
            ObjectNode row=remotes.addObject();row.put("page",String.valueOf(call(page,"zeN")));
            row.put("name",String.valueOf(call(call(slot,"EwU"),"get")));row.put("index",((Number)call(slot,"Iz2")).intValue());
            row.put("nativeParameterId",id(parameter));
        }
        return result;
    }
    static Object resolve(Object document,JsonNode target)throws Exception {
        JsonNode path=target.required("device");Object device=ProjectAddress61.resolveNative(path,document,document.getClass().getClassLoader());
        Object parameter=parameters(device).get(target.required("nativeParameterId").asText());
        if(parameter==null || !target(parameter,path).equals(target) || !Boolean.TRUE.equals(call(parameter,"AZY")))throw new IllegalArgumentException("Device parameter identity, domain or automation eligibility changed; rediscover parameters");
        return parameter;
    }
}
