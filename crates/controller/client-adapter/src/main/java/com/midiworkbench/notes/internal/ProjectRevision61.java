package com.midiworkbench.notes.internal;

import java.lang.ref.WeakReference;
import java.lang.reflect.Proxy;
import java.util.Map;
import java.util.WeakHashMap;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicLong;

/** Lifetime-owned notifications for propagated native document changes. */
public final class ProjectRevision61 implements AutoCloseable {
    public static final String SCOPE="native-changes-v1";
    private static final Map<Object,WeakReference<ProjectRevision61>> OWNERS=new WeakHashMap<>();
    private final WeakReference<Object> proxy;
    private final String lifetime=UUID.randomUUID().toString();
    private final AtomicLong changes=new AtomicLong();
    private WeakReference<Object> document=new WeakReference<>(null);
    private Object listener;
    private Class<?> listenerType;
    private volatile boolean poisoned;
    private boolean closed;

    ProjectRevision61(Object proxy) {
        this.proxy=new WeakReference<>(proxy);
        synchronized(OWNERS) {
            var old=OWNERS.get(proxy);
            if(old!=null && old.get()!=null)throw new IllegalStateException("Native change observer already owned");
            OWNERS.put(proxy,new WeakReference<>(this));
        }
    }
    void bind(Object next) throws Exception {
        Object owner=proxy.get();
        bind(next,Class.forName("com.bitwig.ramona.core.BkN",false,owner.getClass().getClassLoader()));
    }
    void bind(Object next,Class<?> nativeListenerType) throws Exception {
        requireThread();
        if(closed || poisoned)throw new IllegalStateException("Native change observer unavailable");
        if(document.get()==next)return;
        detach();changes.incrementAndGet();
        if(next==null)return;
        listenerType=nativeListenerType;
        listener=Proxy.newProxyInstance(listenerType.getClassLoader(),new Class<?>[]{listenerType},(receiver,method,args)-> {
            switch(method.getName()) {
                case "hashCode": return System.identityHashCode(receiver);
                case "equals": return receiver==args[0];
                case "toString": return "Native document change observer";
                default:
                    try {
                        requireThread();
                        if(storedChange(method.getName(),args))changed();
                    } catch(Throwable error) {poisoned=true;}
                    return null;
            }
        });
        Reflection.method(next.getClass(),"v33",listenerType);
        document=new WeakReference<>(next);
        try {Reflection.call(next,"v33",new Class<?>[]{listenerType},listener);}
        catch(Exception error) {poisoned=true;try{detach();}catch(Exception cleanup){error.addSuppressed(cleanup);}throw error;}
    }
    void invalidate() {try{requireThread();detach();changed();}catch(Throwable error){poisoned=true;}}
    // ddx visitation requires nQ_(); serial.kid additionally excludes XQF().
    private static boolean storedChange(String event,Object[] args) throws Exception {
        return !event.equals("signalFired") && Boolean.TRUE.equals(Reflection.call(args[0],"nQ_"))
            && Boolean.FALSE.equals(Reflection.call(args[0],"XQF"));
    }
    void changed() {changes.updateAndGet(value->{if(value==Long.MAX_VALUE){poisoned=true;return value;}return value+1;});}
    private void requireThread() throws Exception {
        Object owner=proxy.get();
        if(owner==null || !Boolean.TRUE.equals(Reflection.call(owner,"isDocumentThread")))
            throw new IllegalStateException("Wrong native observer thread");
    }
    public static String read(Object proxy,Object document) throws Exception {
        ProjectRevision61 owner;
        synchronized(OWNERS) {var found=OWNERS.get(proxy);owner=found==null?null:found.get();}
        if(owner==null)throw new IllegalStateException("Native document observation has no lifetime owner");
        owner.requireThread();
        if(owner.closed || owner.poisoned || owner.document.get()!=document)
            throw new IllegalStateException("Native document observation is stale or unavailable");
        return owner.lifetime+":"+owner.changes.get();
    }
    private void detach() throws Exception {
        Object root=document.get();
        if(root!=null && listener!=null)Reflection.call(root,"TUp",new Class<?>[]{listenerType},listener);
        listener=null;document.clear();
    }
    @Override public void close() {
        try {requireThread();detach();}catch(Exception error){throw new IllegalStateException("Native change observer disposal failed",error);}
        closed=true;
        synchronized(OWNERS) {Object key=proxy.get();var found=OWNERS.get(key);if(found!=null && found.get()==this)OWNERS.remove(key);}
    }
}
