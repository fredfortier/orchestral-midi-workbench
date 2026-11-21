# Controller qualification

Use [ARCHITECTURE.md](ARCHITECTURE.md) for the intended contracts and
[WORKPAD.md](WORKPAD.md) for implementation gaps. This file defines evidence to
collect; it does not grant permission to mutate an open project.

## Fixture and observation

Use an explicitly disposable project. Record host build, adapter/service/CLI
identities, saved fixture path, document generation and tested owners. Keep the
user's project, extensions, ports and unresolved operations independent.
[MANAGED_BITWIG.md](MANAGED_BITWIG.md) documents the optional isolated Linux host.

Choose the smallest current read from CLI help. Preserve the original observation,
operation intent, actual result and independently authored expectations. Display
pages and counts cannot stand in for complete affected scope. Do not acquire a
whole-project dump just because a previous test did.

## Native acceptance matrix

| Concern | Evidence |
| --- | --- |
| Discovery | Collapsed groups, duplicate names, empty owners, complete native paths; no reflected graph |
| Selection | Object/time/note membership, active versus retained selection, locked layers, explicit unsupported editors |
| Notes/expression | Changed values plus untouched geometry, routing, operators, complete ordered curves and selection |
| Sources/occurrences | Offsets, loops, aliases and explicit rejection where isolation is unsupported |
| Automation | Physical domains, source/placement mapping, equal-time order, interpolation, canonicalization, other curves preserved |
| Instruments | Container/slot ensure and selection, complete Filter/Map readback, independent instrument assignment, load/closure facts and scoped preservation |
| Cues | Exact beat positions, collision/stale refusal and unchanged existing markers |
| History | One musical Undo action; native Undo/Redo restores the affected logical state |
| Failure | Stale refusal before writes; admitted partial-write rollback or honest partial/unknown result |
| Disconnect | Original-ID status after lost response; no repeated mutation or accidental retargeting |
| Artifacts | Native completion and independent file validation; audio/samples/persistence remain separate claims |

Use independent expected values. A serializer/deserializer pair or a receipt
repeating the submitted values is not independent native readback. Fault injection
and history probes belong only to the development harness. Do not treat an offline
pass as admitted live behavior or use automatic Undo to resolve an unknown outcome.

## Build checks and ephemeral probes

From `crates/`, for relevant Rust/Java implementation changes:

```sh
cargo fmt --all -- --check
cargo check -p notes-transformer --all-targets
cargo clippy -p notes-transformer --all-targets -- -D warnings
controller/client-adapter/mvnw -f controller/client-adapter/pom.xml verify
controller/client-adapter/mvnw -f controller/client-adapter/pom.xml -P harness verify
```

Durable unit tests are prohibited. Run small ephemeral probes during each change. Use manifests for supported toolchains
and features. Documentation-only changes require references/content/diff checks;
unrelated builds add no evidence. Live tests remain separate from unattended runs.

Report exact checks, fixture scope, readback/history evidence and unresolved limits.
[qualification/](qualification/) contains historical results, not universal admission.
Current help and ARCHITECTURE.md determine executable syntax; historical recipes
are not current execution instructions.
