# Controller entity and content addressing

Status: Accepted

## Context

Bitwig exposes track and clip placement, track automation and native Inspector
selection. The previous CLI exposed hierarchy and selection independently.
Its selection-based content read treated selected automation points as
unsupported Arranger material. A qualified owning-track read could still
return the complete curve.

## Decision

A qualified structural path is the reusable entity address. Add selection to
the structural overview and retain the narrow selection read for rechecks.
Selection members carry their owner's path and a typed content reference
where applicable. Complete content reads require explicit scope. Replacement
uses the same snapshot grammar and verification transaction.

A separate lane-only query was considered. It would introduce a second answer
to what constitutes a complete observation and could hide neighboring material
required for preservation. Local filtering uses the lane identity within the
complete snapshot and can distinguish multiple placements on the same lane.

## Consequences

Hierarchy and selection can be reconciled from one document-thread callback.
The new content commands can reuse the existing native operations and callers
can migrate from the older spellings. Selected content requires native owner
resolution. A shared source can have several placement candidates, and the
controller must expose that distinction. See the
[entity and CLI contract](../../crates/controller/ARCHITECTURE.md#entity-model)
for the external sources and command grammar.
