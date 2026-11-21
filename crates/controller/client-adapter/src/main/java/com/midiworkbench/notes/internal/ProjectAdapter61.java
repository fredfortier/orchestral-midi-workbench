package com.midiworkbench.notes.internal;
import java.util.List;
/** Lookup one explicitly named native field; never export reflected properties. */
final class ProjectAdapter61 {
 private ProjectAdapter61() {}
 static Object value(Object target,String name)throws Exception {
  for(Object attribute:(List<?>)Reflection.call(target,"RR"))
   if(name.equals(Reflection.call(attribute,"zeN")))return Reflection.call(attribute,"getValue");
  throw new IllegalStateException("Missing required native field: "+name);
 }
}
