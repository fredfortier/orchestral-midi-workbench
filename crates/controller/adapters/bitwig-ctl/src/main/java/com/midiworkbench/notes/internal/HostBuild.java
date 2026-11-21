package com.midiworkbench.notes.internal;

/** Declared host version; capability probes own private-method compatibility. */
public record HostBuild(String version) {
    public static final String VERIFIED_61 = "6.1";

    public static HostBuild loaded(Object proxy) throws Exception {
        Object host=Reflection.call(proxy,"getHost");
        return new HostBuild((String)Reflection.call(host,"getHostVersion"));
    }

    public void requireVerified() {
        if (!VERIFIED_61.equals(version)) {
            throw new IllegalStateException("Unsupported Bitwig version " + version
                    + "; validate mappings before document access");
        }
    }
}
