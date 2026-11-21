package com.midiworkbench.notes;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.function.Predicate;
import java.util.function.ToLongFunction;

/** Bounded replay evidence; expiration never removes an unfinished action. */
final class ReplayCache<T> {
    private final Map<String,T> entries=new LinkedHashMap<>();
    private final int maximumEntries;
    private final long maximumBytes;
    private final ToLongFunction<T> bytes;
    private final Predicate<T> finished;
    ReplayCache(int maximumEntries,long maximumBytes,ToLongFunction<T> bytes,Predicate<T> finished) {
        this.maximumEntries=maximumEntries;this.maximumBytes=maximumBytes;this.bytes=bytes;this.finished=finished;
    }
    T get(String key){return entries.get(key);}
    void put(String key,T value){entries.put(key,value);trim();}
    void clear(){entries.clear();}
    void trim(){
        while(entries.size()>maximumEntries || retainedBytes()>maximumBytes) {
            String expired=entries.entrySet().stream().filter(entry->finished.test(entry.getValue()))
                    .map(Map.Entry::getKey).findFirst().orElse(null);
            if(expired==null)throw new IllegalStateException("Unfinished replay evidence exceeds admitted cache capacity");
            entries.remove(expired);
        }
    }
    long retainedBytes(){return entries.values().stream().mapToLong(bytes).sum();}
    int size(){return entries.size();}
}
