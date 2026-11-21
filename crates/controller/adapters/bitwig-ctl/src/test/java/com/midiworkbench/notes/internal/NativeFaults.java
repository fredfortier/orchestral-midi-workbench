package com.midiworkbench.notes.internal;

import com.midiworkbench.notes.Wire;
import java.util.ArrayList;
import java.util.List;

/** Development-loader instrumentation only; never referenced by production bytecode. */
public final class NativeFaults {
    private static final ThreadLocal<State> CURRENT=new ThreadLocal<>();
    private static final class State {
        final int after;
        int count;
        boolean triggered;
        final List<String> checkpoints=new ArrayList<>();
        State(int after){this.after=after;}
    }
    private NativeFaults() {}
    public static void arm(int after){
        if(after<1 || CURRENT.get()!=null)throw new IllegalArgumentException("Fresh positive native failure checkpoint required");
        CURRENT.set(new State(after));
    }
    public static void checkpoint(String family){
        State state=CURRENT.get();if(state==null || state.triggered)return;
        state.count++;if(state.checkpoints.size()<256)state.checkpoints.add(family);
        if(state.count==state.after){
            state.triggered=true;
            throw new IllegalStateException("Injected failure after native write: "+family+" checkpoint="+state.count);
        }
    }
    public static String disarm(){
        State state=CURRENT.get();CURRENT.remove();if(state==null)return null;
        var result=Wire.object();result.put("after",state.after);result.put("count",state.count);
        result.put("triggered",state.triggered);result.set("checkpoints",Wire.JSON.valueToTree(state.checkpoints));
        return result.toString();
    }
}
