package com.midiworkbench.notes.internal;

/** Proven admission failure before this native operation dispatched any writes. */
public final class NativeRefusal extends Exception {
    private static final long serialVersionUID=1L;
    public NativeRefusal(Exception cause){super(cause.toString(),cause);}
}
