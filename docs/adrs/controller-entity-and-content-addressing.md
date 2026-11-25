# Controller entity and content addressing

Status: Accepted

## Context

Bitwig exposes track and clip placement, track automation and native Inspector
selection. Inspector membership identifies selected automation points. A
qualified owning-track content read supplies the complete curve and its
preservation scope.

## Decision

A qualified structural path is the reusable entity address. Add selection to
the structural overview and retain the narrow selection read for rechecks.
Selection members carry their owner's path and a typed content reference
where applicable. A complete content read takes qualified paths or an explicit
request to use the effective native selection. Replacement uses the same
snapshot grammar and verification transaction for both scopes.

A separate lane-only query was considered. It would introduce a second answer
to what constitutes a complete observation and could hide neighboring material
required for preservation. Local filtering uses the lane identity within the
complete snapshot and can distinguish multiple placements on the same lane.

## Consequences

Hierarchy and selection can be reconciled from one document-thread callback.
The content commands use the existing native operations. Selected content
requires native owner resolution. A shared source can have several placement
candidates, and the controller exposes that distinction. See the
[entity and CLI contract](../architecture/controller.md#entity-model)
for the external sources and command grammar.
