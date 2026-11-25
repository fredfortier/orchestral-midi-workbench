package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

/** Revision of a freshly constructed native observation. */
final class ObservedScope {
    private ObservedScope() {}

    static String revision(ObjectNode observation) throws IOException {
        var json = com.midiworkbench.notes.Wire.JSON;
        com.midiworkbench.notes.Wire.requireMessageCapacity(observation);
        try {
            byte[] digest = MessageDigest.getInstance("SHA-256").digest(json.writeValueAsBytes(observation));
            return HexFormat.of().formatHex(digest);
        } catch (NoSuchAlgorithmException unavailable) {
            throw new IllegalStateException("SHA-256 is unavailable", unavailable);
        }
    }
}
