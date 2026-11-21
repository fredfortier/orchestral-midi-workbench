# Understand track automation

Default to track-level MIDI automation identified by channel and CC number.
Obtain musical meanings from INSTRUMENT.md, not knob discovery. Use
[device parameter discovery](device-parameters.md) only for an explicit beyond-MIDI
request or legacy-sender migration. This reference queries stored content.

1. **CONTROLLER** Resolve the relevant track paths with scoped `info` and read
   complete content through `midi get-all --paths paths.json`.
2. Analyze the retained snapshot locally. `.automation[]` stores every placement
   once alongside `.clips[]`; it does not require a note-clip owner.

```sh
jq '[.automation[] | {
  owner,target,placement,mapping,source,aliases,referenceCount,
  points:(.points|length), range:([.points[].value]|[min,max])}]' before.json
```

CC identity is `target.kind == "midiCc"` with native zero-based `channel` and
`controller`. Other Bitwig parameters retain their physical value domain and
units. A parameter label is not a CC assignment or proof of audible support.

Points retain exact time, value, interpolation, curvature, spread and identity.
Arranger points use arranger beats; committed points use source beats with the
stored placement/source offset and loop mapping. Do not trim points to the note
clip's visible range: an earlier held/interpolated value can affect that range.
Shared source changes affect all acknowledged placements; unsupported isolation
must refuse. Empty/uneditable domains must be distinguished from absent content.

Use [bitwig-edit](../../bitwig-edit/SKILL.md) for point or complete-payload edits.
Do not wrap automation as clips or insert CC helper devices to make the tooling
work. Such changes are not observation and are not prerequisites for MIDI CRUD.

For authorized whole-lane removal, use the distinct selection/focus recipe in
[supporting project actions](keyboard-workflow.md#automation-lane-and-sender-cleanup).
