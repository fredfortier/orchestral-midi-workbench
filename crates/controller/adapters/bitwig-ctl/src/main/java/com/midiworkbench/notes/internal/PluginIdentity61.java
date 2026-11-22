package com.midiworkbench.notes.internal;

/** Address identity for native plug-in contents, independent of their editor window. */
record PluginIdentity61(String id, String version) {
    static PluginIdentity61 read(Object contents, String nativeDeviceId) throws Exception {
        if (contents == null) return null;
        ClassLoader loader = contents.getClass().getClassLoader();
        if (!Class.forName("ych", false, loader).isInstance(contents)) return null;

        boolean clap = nativeDeviceId.equals("clap plugin device contents");
        String idField = clap ? "clap_device_id" : "vst_device_id";
        String prefix = clap ? "clap-" : "vst3-";
        return new PluginIdentity61(prefix + ProjectAdapter61.value(contents, idField),
                ProjectAdapter61.value(contents, "device_version").toString());
    }
}
