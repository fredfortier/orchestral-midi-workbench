# Bitwig qualification probe notes

Use the [adapter instructions](adapters/bitwig-ctl/AGENTS.md) for deployment and
the [verification contract](ARCHITECTURE.md#wire-deployment-and-verification)
for acceptance. Select probe cases applicable to the change and fixture.

## Candidate probe scopes

| Concern | Evidence |
| --- | --- |
| Discovery | Collapsed groups, duplicate names, empty owners, complete native paths; round-trip discovered CLAP and VST plug-in paths through `controller info --path` |
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
