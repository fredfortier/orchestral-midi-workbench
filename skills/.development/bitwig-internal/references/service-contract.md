# Service/adapter development

The [architecture data contracts](../../../../crates/controller/ARCHITECTURE.md#purpose-and-execution-boundaries)
own application intent, domain planning, native execution and recovery. Read the
[historical qualification](../../../../docs/adrs/controller-native-authority.md) before treating an existing
session/DTO as the desired contract.

Rust owns plain musical computation and volatile request completion. Java owns native resolution,
preflight, Undo and independent scoped readback; it does not hold an Undo group
while waiting on Rust. Native objects never cross threads or JSON boundaries.
A prepared musical scope is complete for its operation, not a project dump.

Inspect current [Rust sessions](../../../../crates/controller/src/session.rs),
[application DTOs](../../../../crates/controller/src/control.rs) and
[Java session](../../../../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/EditorSession.java).
Do not copy historical protocol numbers, output budgets or history commands from
old probes. Use operation IDs to reconcile original execution; a receipt after
Undo is historical evidence, not current native state.

Keep controller status/counts in application preferences: document settings
would add musical Undo entries. Qualify changed dispatch/result paths with
independent preservation, history, stale-state and disconnect fixtures under
[RUNBOOK.md](../../../../crates/controller/RUNBOOK.md). Do not infer live qualification
from peer mocks or serialization round trips.
