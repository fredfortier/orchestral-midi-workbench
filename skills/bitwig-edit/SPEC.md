# Native editing design rationale and evidence

Authoring-only rationale. [SKILL.md](SKILL.md) and its directly routed references
own runtime editing; [controller architecture](../../controller/ARCHITECTURE.md)
describes the implementation contract.

The complete-clip model keeps note, expression, automation and source mapping in
one preservation scope. Sparse edit intents do not require a second adapter read
model. This avoids a projection losing untouched fields while appearing to make
a small edit. Supporting UI actions remain a separate surface for the same reason
as in exploration: focus is not content authority.

Historical controller/WORKPAD.md evidence includes selected and explicit paths,
multi-clip replacement, note/point CRUD, expressions, stale refusal, Undo/Redo and
injected-write rollback on Bitwig 6.1. These are implementation qualification
records, not a runtime instruction to repeat a probe campaign on each project.
