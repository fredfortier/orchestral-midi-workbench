package com.midiworkbench.notes;

import com.bitwig.extension.api.PlatformType;
import com.bitwig.extension.controller.AutoDetectionMidiPortNamesList;
import com.bitwig.extension.controller.ControllerExtensionDefinition;
import com.bitwig.extension.controller.api.ControllerHost;
import java.util.UUID;

public final class NotesAdapterDefinition extends ControllerExtensionDefinition {
    private static final UUID ID = UUID.fromString("08d96063-f7a1-4f66-bd72-6c0eec8e54db");

    @Override public String getName() { return "Notes Document Adapter"; }
    @Override public String getAuthor() { return "MIDI2.0Workbench"; }
    @Override public String getVersion() { return "0.4.31"; }
    @Override public UUID getId() { return ID; }
    @Override public String getHardwareVendor() { return "MIDI2.0Workbench"; }
    @Override public String getHardwareModel() { return "Notes Document Adapter"; }
    @Override public int getRequiredAPIVersion() { return 25; }
    @Override public boolean isUsingBetaAPI() { return true; }
    @Override public boolean shouldFailOnDeprecatedUse() { return true; }
    @Override public int getNumMidiInPorts() { return 0; }
    @Override public int getNumMidiOutPorts() { return 0; }
    @Override public void listAutoDetectionMidiPortNames(
            AutoDetectionMidiPortNamesList names, PlatformType platform) {}
    @Override public NotesAdapterExtension createInstance(ControllerHost host) {
        return new NotesAdapterExtension(this, host);
    }
}
