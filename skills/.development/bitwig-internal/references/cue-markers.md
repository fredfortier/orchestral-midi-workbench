# Arranger cue markers

For the intended scope, read the
[landmark contract](../../../../crates/controller/ARCHITECTURE.md#purpose-and-execution-boundaries).
Current help and source establish implemented limits and operations. Use
`controller cues read/create`; read help for the exact arguments and independently
read markers again to verify creation. Native
transport `cue_marker_timeline` owns these markers; clip-source cue timelines
are different objects. Keep the Java adapter on the document thread and match
the admitted host capability before accessing it. Never derive insertion
signatures from a controller cue bank or expose generic reflection on the wire.

The [performance-cues skill](../../../performance-cues/SKILL.md) owns musical
assessment, targeted literature research and the project's PERFORMANCE.md.
Its mandatory prerequisite is an already open Bitwig Studio with an active
project; stop if it cannot be established. A source code trace establishes a
hypothesis; disposable native insertion, preservation and Undo readback establish
host behavior. Live qualification is separate from ordinary composition.
