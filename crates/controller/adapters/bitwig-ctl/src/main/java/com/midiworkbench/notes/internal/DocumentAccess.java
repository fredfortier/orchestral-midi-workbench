package com.midiworkbench.notes.internal;

import java.lang.reflect.Method;
import java.util.Set;
import java.util.concurrent.Callable;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;

/** Owns Java-only document dispatch; no network or JS callbacks execute here. */
public final class DocumentAccess implements AutoCloseable {
    private final Object proxy;
    private final Method dispatch;
    private final AtomicBoolean closed = new AtomicBoolean();
    private final Set<CompletableFuture<?>> pending = ConcurrentHashMap.newKeySet();

    public DocumentAccess(Object documentProxy, HostBuild build) throws Exception {
        build.requireVerified();
        proxy = documentProxy;
        dispatch = Reflection.method(proxy.getClass(), "exec", Runnable.class);
    }

    public <T> CompletableFuture<T> submit(Callable<T> operation) {
        CompletableFuture<T> result = new CompletableFuture<>();
        pending.add(result);
        result.whenComplete((value, error) -> pending.remove(result));
        if (closed.get()) {
            result.completeExceptionally(new IllegalStateException("Adapter exited"));
            return result;
        }
        try {
            dispatch.invoke(proxy, (Runnable) () -> {
                if (closed.get() || result.isDone()) {
                    result.cancel(false);
                    return;
                }
                try {
                    if (!Boolean.TRUE.equals(Reflection.call(proxy, "isDocumentThread"))) {
                        throw new IllegalStateException("Wrong document thread");
                    }
                    result.complete(operation.call());
                } catch (Exception error) {
                    result.completeExceptionally(error);
                }
            });
        } catch (ReflectiveOperationException error) {
            result.completeExceptionally(error);
        }
        return result;
    }

    @Override public void close() {
        closed.set(true);
        pending.forEach(result -> result.cancel(false));
    }

    /** Cancel queued work now, then dispose native subscriptions on their owner. */
    public CompletableFuture<Void> close(AutoCloseable resource) {
        close();
        CompletableFuture<Void> disposed=new CompletableFuture<>();
        try {
            dispatch.invoke(proxy,(Runnable)()->{
                try {
                    if(!Boolean.TRUE.equals(Reflection.call(proxy,"isDocumentThread")))throw new IllegalStateException("Wrong document thread");
                    resource.close();disposed.complete(null);
                }catch(Exception error){disposed.completeExceptionally(error);}
            });
        }catch(ReflectiveOperationException error){disposed.completeExceptionally(error);}
        return disposed;
    }
}
