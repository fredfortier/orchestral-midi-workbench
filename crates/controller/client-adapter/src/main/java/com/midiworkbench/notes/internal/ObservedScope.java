package com.midiworkbench.notes.internal;

import com.fasterxml.jackson.databind.node.ObjectNode;
import java.util.UUID;

/** One bounded scope's expected values, owned by its adapter lifetime. */
final class ObservedScope {
    private ObjectNode expected;
    private String operation;
    String observe(ObjectNode actual) throws java.io.IOException {
        com.midiworkbench.notes.Wire.requireMessageCapacity(actual);
        ObjectNode values=actual.deepCopy();values.remove("revision");
        if(!values.equals(expected)) {expected=values;operation=UUID.randomUUID().toString();}
        return operation;
    }
    void clear() {expected=null;operation=null;}
}
