# Atomically replace complete clip MIDI and automation

Input is one fresh complete clip snapshot, copied and modified offline. Preserve
its clip paths, native identities, source/loop metadata, authority and every
unrequested value. New notes/points use unique temporary IDs. Retained IDs mean
update; omitted notes/points mean delete. Do not send a statistics projection.

```sh
midi-bridge controller midi get-all --paths paths.json --output before.json
jq '(.clips[].notes[] | select(.note.key==60) | .note.velocity) = 0.6' before.json > desired.json
midi-bridge controller midi replace --expected before.json --input desired.json --output after.json
```

Use a fresh evidence filename/directory per attempt; output publication refuses
existing files. Check command success before parsing the retained file.

Before dispatch, quantize each new or changed automation-point time to the native
`2^-20` quarter-note-beat grid: `floor(time * 1048576 + 0.5) / 1048576`.
Record this representation-boundary rounding; preserve untouched point values.
New `HOLD` points require curvature exactly zero, including points copied from an
older parameter lane whose readback contains floating residue. Recheck point
ordering and equal-time intent after quantization; do not silently merge events.

Both files include all notes, expressions and automation. The adapter resolves
paths and checks current complete values, writes the batch in one native Undo
group and independently reads back all affected content. Rust checks the intended
result and preservation. A stale read requires a fresh read and recomputation,
not a changed revision pasted into the old payload. Native canonicalization must
be explicit in the result; unsupported semantics refuse before mutation.

Check the result independently, using a fresh read of the same paths. Compare
requested values and all preserved data. A timeout/disconnect is unknown, not
permission to repeat replacement or issue speculative Undo. Inspect the original
operation and current affected content first. Do not use UI edits, another
controller surface or screenshot parsing to complete or verify this operation.
