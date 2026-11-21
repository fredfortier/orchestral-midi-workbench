package com.midiworkbench.notes;

import com.bitwig.extension.controller.ControllerExtension;
import com.bitwig.extension.controller.api.ControllerHost;
import com.bitwig.extension.controller.api.SettableStringValue;
import com.midiworkbench.notes.internal.DocumentAccess;
import com.midiworkbench.notes.internal.EditorAdapter61;
import com.midiworkbench.notes.internal.HostBuild;

/** A native read/write message processor for the Rust MIDI service. */
public final class NotesAdapterExtension extends ControllerExtension {
    private EditorSession session;
    private ConnectionControl connectionControl;
    public NotesAdapterExtension(NotesAdapterDefinition definition,ControllerHost host){super(definition,host);}
    @Override public void init() {
        ControllerHost host=getHost();
        // Telemetry belongs to application preferences. DocumentState setting writes
        // are native project edits and would pollute the user's musical Undo history.
        var settings=host.getPreferences();
        SettableStringValue status=settings.getStringSetting("Status","Notes",512,"Connecting…");
        ((com.bitwig.extension.controller.api.Setting)status).disable();
        try {
            Object proxy=host.getDocumentState();HostBuild build=HostBuild.loaded(proxy);
            DocumentAccess document=new DocumentAccess(proxy,build);
            session=new EditorSession(host,document,new EditorAdapter61(proxy,build,document),status::set);
            connectionControl=new ConnectionControl(session,ConnectionControl.root());
            tick();
        }catch(Exception error){status.set("Adapter unavailable: "+error);host.println("Adapter unavailable: "+error);}
    }
    private void tick(){if(session==null)return;try{session.tick();connectionControl.tick();}finally{if(session!=null)getHost().scheduleTask(this::tick,100);}}
    @Override public void flush() {}
    @Override public void exit(){if(session!=null){if(connectionControl!=null)connectionControl.close();session.close();session=null;}}
}
