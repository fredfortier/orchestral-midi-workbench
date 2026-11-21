package com.midiworkbench.notes.internal;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;

/** The single reflective invocation boundary for build-specific mappings. */
final class Reflection {
    private Reflection() {}
    private static final Class<?>[] NO_TYPES={};
    private static final Object[] NO_ARGUMENTS={};
    private record Signature(String name,List<Class<?>> parameters) {}
    private static final class Methods {
        final ConcurrentHashMap<String,Method> noArguments=new ConcurrentHashMap<>();
        final ConcurrentHashMap<Signature,Method> parameterized=new ConcurrentHashMap<>();
    }
    // Class-scoped method metadata, never a document owner or a returned host value.
    // ClassValue avoids a permanent global map of retired extension/proxy classes.
    private static final ClassValue<Methods> METHODS=new ClassValue<>() {
        @Override protected Methods computeValue(Class<?> type) {return new Methods();}
    };

    static Method method(Class<?> type, String name, Class<?>... signature)
            throws NoSuchMethodException {
        Methods methods=METHODS.get(type);
        if(signature.length==0) {
            Method cached=methods.noArguments.get(name);if(cached!=null)return cached;
            Method resolved=resolve(type,name,NO_TYPES);
            Method prior=methods.noArguments.putIfAbsent(name,resolved);
            return prior==null?resolved:prior;
        }
        Signature key=new Signature(name,List.of(signature));
        Method cached=methods.parameterized.get(key);if(cached!=null)return cached;
        Method resolved=resolve(type,name,signature);
        Method prior=methods.parameterized.putIfAbsent(key,resolved);
        return prior==null?resolved:prior;
    }
    private static Method resolve(Class<?> type,String name,Class<?>[] signature) throws NoSuchMethodException {
        for (Class<?> owner = type; owner != null; owner = owner.getSuperclass()) {
            try {
                Method result = owner.getDeclaredMethod(name, signature);
                result.setAccessible(true);
                return result;
            } catch (NoSuchMethodException ignored) {
                // The public proxy inherits its protected document dispatcher.
            }
        }
        // Native owners also inherit public default methods from interface hierarchies.
        // Class.getMethod applies Java's most-specific-interface resolution; keep exact parameters.
        Method result = type.getMethod(name, signature);
        result.setAccessible(true);
        return result;
    }

    static Object call(Object target, String name, Class<?>[] signature, Object... arguments)
            throws Exception {
        if(target==null)throw new IllegalStateException("Null native owner for method "+name);
        try {
            return method(target.getClass(), name, signature).invoke(target, arguments);
        } catch (InvocationTargetException error) {
            Throwable cause = error.getCause();
            if (cause instanceof Exception exception) throw exception;
            if (cause instanceof Error fatal) throw fatal;
            throw new IllegalStateException(cause);
        } catch (IllegalArgumentException error) {
            String types=java.util.Arrays.stream(arguments).map(value->value==null?"null":value.getClass().getName()).collect(java.util.stream.Collectors.joining(", "));
            throw new IllegalArgumentException("Native invocation argument mismatch: "+target.getClass().getName()+"."+name+" received ["+types+"]",error);
        }
    }

    static Object call(Object target, String name) throws Exception {
        return call(target, name, NO_TYPES,NO_ARGUMENTS);
    }
}
