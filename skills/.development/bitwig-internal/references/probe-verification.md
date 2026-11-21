# Probe and establish correctness

Use the existing [probe inventory](../../../../controller/probes/README.md),
[native editing runbook](../../../../controller/RUNBOOK.md) and
[retained host evidence](../../../../controller/validation/editor-selection/README.md).
Preserve a failed probe and its input before changing the implementation.

## Repeatable use cases

| Task | Isolate and establish |
| --- | --- |
| Read tool-2 selection across tracks | active editor, range, leaf lanes, complete effective notes |
| Read selected notes in one launcher clip | object/time scope, owning clip/content, no unrelated cached selection |
| Change channel/key | same source object, correct row, collisions, preserved curves and selection |
| Edit velocity/operator | descriptor type/range, dependent enable flags, exact unrelated payload |
| Create/delete/replace | exact owners, boundary/shared-content behavior, remapping, one Undo/Redo |
| Edit pressure/timbre points | point ownership/time/interpolation/spread, independent curve readback |
| Query context beyond beat 64 | source/occurrence mapping, local tempo/meter, no playhead movement |
| Reconnect/retry a processing session | same immutable input/result, stale refusal, no duplicate application |

## Failure / repair guide

| Observation | Investigate / correct |
| --- | --- |
| Cursor selection empty while UI has range | inspect active editor time selection, not just object/cell flags |
| Launcher clips leak into arranger inventory | use arranger timeline query rather than general lane children |
| Group overview duplicates notes | identify leaf note lanes and source identity; do not count overview objects |
| Timing looks stretched | verify end versus duration, host ticks, source versus project coordinates |
| Channel change loses expression | transfer original source note; do not clear/recreate from grid scalars |
| Detached clone has missing relationship | trace copy context/shared-content factory; retain failure, do not strip relationship |
| Consolidation changes apparent selection | establish native playable/loop semantics; consolidation is not a clone |
| Tempo is wrong | distinguish physical/normalized values, address mapping, conversion direction and point order |
| JavaScript multithread exception | keep all document execution in Java; no cross-thread JS Runnable |
| Synchronous wait hangs | never block the queue whose Runnable must complete; use async immutable results |
| Undo appears ineffective | verify focus, proxy readiness, undo label and actual independent state restoration |
| Class not found after host update | check declared host version, then remap and revalidate capabilities |
| Failed stage followed by commit uses old material | invalidate stage before attempting a replacement; refuse absent/new-invalid state |

Start mutations only in explicitly disposable fixture material. Read the user's
current selection without modifying it. Manually invoke named Java methods;
console convenience must not reintroduce a JavaScript implementation layer.
Use structured logs with operation/project/generation IDs, arguments, host build,
thread, requested and observed values. Avoid logging entire user projects when
summaries plus scoped snapshots suffice.

For every write, compare all unrequested properties and unselected material,
then Undo and Redo and reread. Match source payloads, expression points, ownership,
selection and occurrence geometry. Test same-key collisions, shared content,
loops and partial boundaries separately. Reject a setter-only demonstration.

A synthetic unit test proves only its boundary. A JAR packaging test proves
packaging; a host-runtime load proves loadability; actual Bitwig selection and
mutation evidence proves those exercised operations. Keep unresolved acceptance
items visible instead of describing them as completed coverage.
