# Clip mappings and context at a location

Read [clip looping concepts](https://www.bitwig.com/userguide/latest/working_with_clips/)
and [note editing](https://www.bitwig.com/userguide/latest/working_with_note_events/)
for musical semantics. Keep four coordinate systems explicit: source-note time,
clip-local elapsed time, arranger project beats and elapsed project seconds.
Never substitute current transport tempo for location tempo.

## Inspected clip model

| Method | Meaning established by probes |
| --- | --- |
| `T1L()` | arranger placement |
| `GaO()` / `IVb()` | occurrence duration / end |
| `dD_()` / `Fgm()` | source play start / stop markers |
| `heu()` | cue/play/loop mapping object |
| `fvB()` | source loop region, with time/duration attributes |

Use the host mapping object to convert elapsed clip time to source position and
loop iteration. The [mapping probe](../../../../controller/validation/editor-selection/clip-mapping.jsonl)
exercises elapsed 0, 1, 5 and 64, including offset .375 and a two-beat loop.
Do not assume `source = project - placement` or clamp all notes to one loop.
Changing the start marker with `UMc(double)` changed duration because the stop
marker stayed fixed; it did not perform a duration-preserving content slide.

The native occurrence wrapper in the probe distinguishes nominal onset/end from
visible clipped onset/end. Geometric overlap alone does not prove that source
material participates in the effective selection. The consolidation experiment
omitted material before the loop region despite apparently intersecting geometry.
Trace native selection/copy behavior for the exact editor operation.

When inserting over a clip range, Bitwig can split the original into left and
right clips, cloning right-hand content and changing its identities. Preserve
all material outside the requested range by time/owner/payload comparison and
return old-to-new identity mappings. Do not mistake a native ID change for an
unrequested payload change, or use this exception to hide altered music.

## Location transport

The [context probe](../../../../controller/probes/NotesDocumentProbe.java)
resolves the inspected document transport with `document.Nh()`. In that build:

- `EwU(projectBeats)` maps beats to seconds; `jaQ(seconds)` maps back.
- Read tempo automation at the requested beat, then convert its normalized value
  through the tempo parameter's mapping (`TUp(normalized)`). Raw physical BPM and
  normalized automation values are different domains.
- Read the held signature point at that beat, decode numerator/denominator through
  the host signature helper, and retain bar/beat context where established.
- Keep playhead-before/after in the evidence to prove the query did not seek.

The [retained context observations](../../../../controller/validation/editor-selection/context.jsonl)
verify BPM 110 before beat 32, BPM 140 and 3/4 at/after beat 32, with exact
beat/second round trips. They do not yet validate every ramp, pickup or meter case.
Test ramps and boundaries independently before claiming complete map coverage.

If constructing a disposable tempo fixture, use the parameter's actual automation
address, physical-to-normalized conversion and the sorted insertion index.
The earlier index `0` appended points in the wrong order; `-1` requested automatic
sorted insertion. Wrong insertion order and reversed time-conversion calls are
retained comprehension failures, not host limits.
