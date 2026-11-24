---
name: bitwig-internal
description: Build and probe Java integrations with Bitwig's private in-process document API. Use for UI-owned note selection across arranger tracks/clips, direct note CRUD, channel/key row transfers, expression curves, clip/source/loop mapping, location tempo context, native Undo, reflection mapping and host-build compatibility. Uses a Java-only extension and JSON contract with Rust; excludes MIDI hardware mappings, CLAP audio processing and ordinary published-controller grid editing.
---

# Bitwig internal document API

Use one Java `.bwextension` as the host adapter. Use the published SDK only for
extension lifecycle, discovery, settings and scheduling notifications. Access
selection, stored notes and location context through the document adapter. Do
not add a JavaScript bridge, grid reconstruction path, MIDI interface or a
second competing note model.

Read the [controller architecture](../../../crates/controller/ARCHITECTURE.md) for
intended authority/data boundaries and [historical qualification](../../../docs/adrs/controller-native-authority.md)
for build-specific evidence. [Probe inventory](../../../crates/controller/probes/README.md)
and retained qualifications establish only their original host/fixture scope.
For ordinary musical tasks use [the focused workflow](../../bitwig-compose/SKILL.md).
This skill does not authorize native mutation or an implementation change by itself.
Start new integration work with [the Java harness conventions](references/java-harness.md).

## Choose a category

| Open when you need to… | Read |
| --- | --- |
| Translate Java controller patterns or expose a repeatable development harness | [Java harness](references/java-harness.md) |
| Build/load the Java extension, dispatch document work, or admit a host build | [execution and compatibility](references/execution-compatibility.md) |
| Read object/time selection, grouped arranger lanes or editable editor layers | [selection and ownership](references/selection-ownership.md) |
| Create/read/update/delete notes, reassign channel/key or edit expression points | [note CRUD](references/note-crud.md) |
| Map clip source/loop occurrences or query tempo/meter at a location | [timeline and transport](references/timeline-transport.md) |
| Read/create Arranger Cue Markers for a performance arc | [cue markers](references/cue-markers.md) |
| Connect the Rust processor, freeze snapshots, apply plans or recover acknowledgments | [service contract](references/service-contract.md) |
| Inspect complete clips or distinguish native/MIDI file boundaries | [project measurement](references/project-measurement.md) |
| Create a disposable MIDI fixture or identify retired construction scope | [native construction](references/project-construction.md) |
| Investigate a failed probe or establish acceptance | [probe and verification](references/probe-verification.md) |

Resolve references relative to this skill. Retained host observations are
build-specific executable evidence, not public API contracts. Research, source
lineage and failed hypotheses live in [SOURCES.md](SOURCES.md).
Use the published Java SDK for lifecycle signatures. Qualify private methods
and selection scope against the actual admitted host build.

## Work in isolation before composition

1. Identify the owning document/editor/content objects and the exact question.
   Read the applicable GitHub example **first** (see the harness reference),
   formulate a concrete hypothesis, then cross-reference the installed implementation.
   An obfuscated name or plausible signature alone is not a usage pattern.
2. Read the relevant reference and implementation. Check the declared host version and admitted capability before invoking
   mapped methods.
3. Manually invoke one named Java probe; log arguments, host identity, owner
   thread, result/error and independent before/after snapshots.
4. On failure, inspect the same operation in Bitwig's own inspector/command
   implementation, then Openwig, official docs and applicable DrivenByMoss code.
   Check overloads, ownership, units, selection mode and transaction context.
   Do not turn a misunderstood call into an accepted product limit.
5. Admit a capability only after its host readback and preservation checks pass.
   Compose tested capabilities behind the Java adapter; keep reflection and
   obfuscated names inside its build-specific implementation.

Never ship an unknown build by best-effort reflection. Structural discovery
produces candidates; behavioral probes decide whether to admit them. Never send
live document references across threads or over JSON. Keep all host access on
the host's document thread; pass immutable snapshots and desired edits across
boundaries. Never block that thread waiting for a socket, a future or user input.
