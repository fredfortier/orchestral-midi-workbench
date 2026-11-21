package com.midiworkbench.notes.internal;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.midiworkbench.notes.Wire;
import java.util.List;
/** Disposable two-clip MIDI and automation fixture; never a production control API. */
public final class ConstructionProbe61 {
 private static final String GROUP="IMPORT-PROBE native group";
 public static String run(Object proxy,String json)throws Exception {
  ObjectNode request=Wire.parse(json);
  if(request.path("phase").asText().equals("reload-adapter"))return AdapterReloadProbe61.run(proxy,request);
  if(!request.required("phase").asText().equals("native") || !request.required("confirmation").asText().equals("DISPOSABLE FIXTURE"))throw new IllegalArgumentException("Only disposable MIDI fixture creation is supported");
  ClassLoader loader=proxy.getClass().getClassLoader();Object document=Reflection.call(proxy,"getDocument"),root=Reflection.call(document,"peG");
  if(!request.required("projectUuid").asText().equals(ProjectAdapter61.value(document,"UUID").toString()) || !request.required("revision").asText().equals(ProjectRevision61.read(proxy,document)))throw new IllegalStateException("Stale fixture authority");
  for(Object child:(List<?>)Reflection.call(root,"IFc"))if(GROUP.equals(ProjectAdapter61.value(child,"displayed_title")))throw new IllegalStateException("Fixture already exists; no replay");
  EditorAdapter61.requireIndependentUndo(document);
  Reflection.call(document,"v33",new Class<?>[]{String.class,boolean.class},"Create disposable MIDI fixture",false);
  try {
                Object group=Reflection.call(root,"niE");title(group,GROUP);append(root,group,loader);
                for(int i=0;i<2;i++) {
                    // ApplicationProxy.doCreateInstrumentTrack uses this factory
                    // and the explicit insertion owner, on the document thread.
                    Object track=Reflection.call(group,"v33",new Class<?>[]{type(loader,"fn")},type(loader,"fn").getField("v33").get(null));
                    title(track,"IMPORT-PROBE native part "+i);append(group,track,loader);
                    set(track,"override_note_channel",-1);
                    Object command=Reflection.call(type(loader,"I1I").getField("tGW").get(null),"CDr");
                    Object clip=Reflection.call(command,"v33",new Class<?>[]{type(loader,"com.bitwig.ramona.core.oJn"),List.class},track,List.of(8.0,4.0));
                    Reflection.call(clip,"qKA",new Class<?>[]{boolean.class},false);
                    note(clip,loader,0,60,0.0,1.0,0.8,0.2);
                    note(clip,loader,i==0?0:1,60,0.5,1.0,0.7,0.3);
                    note(clip,loader,15,60,0.0,2.0,0.9,0.4);
                    note(clip,loader,3,72,2.25,0.001,10001.0/65535.0,23117.0/65535.0);
                    controls(track,loader);
                }
                flush(document);
  }finally {Reflection.call(document,"v33",new Class<?>[]{boolean.class},false);}
  return Wire.object().put("status","measured").toString();
 }
    private static void append(Object owner,Object track,ClassLoader loader) throws Exception {
        Reflection.call(owner,"v33",new Class<?>[]{List.class,int.class,type(loader,"com.bitwig.base.async.xAa"),type(loader,"KIT")},List.of(track),((Number)Reflection.call(owner,"TFo")).intValue(),null,null);
    }
    private static void title(Object track,String title) throws Exception {Reflection.call(track,"d_",new Class<?>[]{String.class},title);}
    private static void set(Object target,String key,Object value) throws Exception {
        Reflection.call(Reflection.call(target,"Nln",new Class<?>[]{String.class},key),"setValue",new Class<?>[]{Object.class},value);
    }
    private static void note(Object clip,ClassLoader loader,int channel,int key,double start,double duration,double velocity,double release) throws Exception {
        Object command=Reflection.call(Reflection.call(clip,"EVR"),"cSc");
        Object note=Reflection.call(command,"v33",new Class<?>[]{type(loader,"com.bitwig.ramona.core.oJn"),List.class},clip,List.of(channel,key,start,duration,velocity));
        Reflection.call(note,"TUp",new Class<?>[]{double.class},release);
    }
    private static void controls(Object track,ClassLoader loader) throws Exception {
        Object container=Reflection.call(track,"xel");
        Object parameter=Reflection.call(container,"v33",new Class<?>[]{int.class,type(loader,"tsU"),int.class},15,type(loader,"tsU").getField("YLV").get(null),1);
        Object reference=type(loader,"com.bitwig.flt.document.core.master.oJn").getMethod("v33",type(loader,"bXo")).invoke(null,parameter);
        Object lane=Reflection.call(track,"YLV",new Class<?>[]{type(loader,"com.bitwig.flt.document.core.iface.bf")},reference);
        Object clip=type(loader,"I3W").getMethod("v33",type(loader,"bD"),double.class,double.class).invoke(null,Reflection.call(parameter,"qeL"),8.0,4.0);
        double fractional=(double)0x12345678L/(double)0xFFFFFFFFL;
        // I3W's arranger position is 8, but its source starts at zero.
        double[][] points={{0.0,20.0/127.0},{1.0,fractional},{2.0,100.0/127.0}};
        for(double[] point:points) {
            Object p=type(loader,"nCh").getMethod("v33",double.class,double.class).invoke(null,point[0],point[1]);
            Reflection.call(p,"v33",new Class<?>[]{type(loader,"CBo")},type(loader,"CBo").getField("v33").get(null));
            Reflection.call(Reflection.call(clip,"mQv"),"jaQ",new Class<?>[]{type(loader,"oHd")},p);
        }
        Object timeline=Reflection.call(lane,"XQF");
        Reflection.call(timeline,"v33",new Class<?>[]{type(loader,"fza")},clip);
    }
    private static Class<?> type(ClassLoader loader,String name) throws ClassNotFoundException {return Class.forName(name,true,loader);}
    static void flush(Object document) throws Exception {Reflection.call(Reflection.call(Reflection.call(Reflection.call(Reflection.call(document,"cuK"),"TcB"),"v33"),"v33"),"LHs");}
}
