package com.midiworkbench.notes;

import java.io.IOException;

/** Plain JSON bodies; the socket length prefix bounds the complete message. */
public final class Frame {
    public static final int MAX_DECODED_BYTES=Wire.MAX_MESSAGE_BYTES;
    private Frame() {}
    public static byte[] encode(byte[] bytes) throws IOException {
        if(bytes.length==0 || bytes.length>Wire.MAX_MESSAGE_BYTES)throw new IOException("JSON message exceeds socket limit");
        return bytes;
    }
    public static byte[] decode(byte[] bytes) throws IOException {return encode(bytes);}
}
