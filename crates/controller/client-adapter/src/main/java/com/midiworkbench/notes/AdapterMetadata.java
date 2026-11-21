package com.midiworkbench.notes;

/** Build-time artifact identity; requires no Bitwig process or SDK initialization. */
public final class AdapterMetadata {
    private AdapterMetadata() {}
    public static void main(String[] args) {
        System.out.println("{\"schemaVersion\":1,\"wireVersion\":"+Wire.VERSION+"}");
    }
}
