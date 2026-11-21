package com.midiworkbench.notes.internal;
import com.midiworkbench.notes.Wire;
import com.fasterxml.jackson.databind.node.ObjectNode;
/** Only disposable fixture authority and native selection experiments. */
public final class EditorProbe61 {
 public static String run(Object proxy,Object rawEditor,String json)throws Exception {
  EditorAdapter61 editor=(EditorAdapter61)rawEditor;ObjectNode request=Wire.parse(json);
  String generation=editor.currentProjectGeneration();
  if(!request.required("type").asText().equals("authority"))throw new IllegalArgumentException("Unknown selection probe");
  Object document=Reflection.call(proxy,"getDocument");ObjectNode result=Wire.object(),project=result.putObject("project");
  project.put("controller_generation",generation);project.put("UUID",ProjectAdapter61.value(document,"UUID").toString());project.put("name",ProjectAdapter61.value(document,"name").toString());
  result.put("revision",ProjectRevision61.read(proxy,document));return result.toString();
 }
}
