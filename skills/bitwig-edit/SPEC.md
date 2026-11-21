# Native editing design rationale and evidence

Authoring rationale for the [native editing procedure](SKILL.md).

The complete-clip model keeps note, expression, automation and source mapping in
one preservation scope. Sparse edit intents do not require a second adapter read
model. This avoids a projection losing untouched fields while appearing to make
a small edit. Supporting UI actions remain a separate surface for the same reason
as in exploration: focus is not content authority.
