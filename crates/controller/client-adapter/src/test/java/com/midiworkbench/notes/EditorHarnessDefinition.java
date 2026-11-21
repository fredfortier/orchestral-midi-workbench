package com.midiworkbench.notes;

import com.bitwig.extension.api.PlatformType;
import com.bitwig.extension.controller.AutoDetectionMidiPortNamesList;
import com.bitwig.extension.controller.ControllerExtension;
import com.bitwig.extension.controller.ControllerExtensionDefinition;
import com.bitwig.extension.controller.api.ControllerHost;
import java.util.UUID;

/** Only packaged by -Pharness. Never part of the production service surface. */
public final class EditorHarnessDefinition extends ControllerExtensionDefinition {
    @Override public String getName(){return "Notes Editor Harness";}
    @Override public String getAuthor(){return "MIDI2.0Workbench";}
    @Override public String getVersion(){return "0.3.0";}
    @Override public UUID getId(){return UUID.fromString("8a9470ba-b9cf-4ad8-a801-e48721e95d42");}
    @Override public String getHardwareVendor(){return "MIDI2.0Workbench";}
    @Override public String getHardwareModel(){return getName();}
    @Override public int getRequiredAPIVersion(){return 25;}
    @Override public boolean isUsingBetaAPI(){return true;}
    @Override public boolean shouldFailOnDeprecatedUse(){return true;}
    @Override public int getNumMidiInPorts(){return 0;}
    @Override public int getNumMidiOutPorts(){return 0;}
    @Override public void listAutoDetectionMidiPortNames(AutoDetectionMidiPortNamesList names,PlatformType platform){}
    @Override public ControllerExtension createInstance(ControllerHost host){return new EditorHarness(this,host);}
}
