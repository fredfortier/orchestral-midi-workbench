package com.midiworkbench.notes;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.nio.charset.StandardCharsets;

/** JSON is confined to immutable strings at the host/network boundary. */
public final class Wire {
    public static final int VERSION = 77;
    public static final int MAX_MESSAGE_BYTES = 16 * 1024 * 1024;
    public static final ObjectMapper JSON = new ObjectMapper()
            .enable(com.fasterxml.jackson.core.JsonParser.Feature.STRICT_DUPLICATE_DETECTION)
            .enable(com.fasterxml.jackson.databind.DeserializationFeature.FAIL_ON_TRAILING_TOKENS);
    private Wire() {}
    /** Check our actual UTF-8 encoding and reader constraints before native writes. */
    public static void requireMessageCapacity(ObjectNode message) throws IOException {
        String json = message.toString();
        Frame.encode(json.getBytes(StandardCharsets.UTF_8));
        parse(json);
    }
    public static ObjectNode object() { return JSON.createObjectNode(); }
    public static ObjectNode parse(String text) throws IOException {
        JsonNode node = JSON.readTree(text);
        if (!(node instanceof ObjectNode object)) throw new IOException("Expected JSON object");
        return object;
    }
}
