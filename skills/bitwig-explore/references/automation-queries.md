# Understand track automation

Default to track-level MIDI automation identified by channel and CC number.
Obtain musical meanings from INSTRUMENT.md, not knob discovery. Use
[device parameter discovery](device-parameters.md) only for an explicit beyond-MIDI
request or legacy-sender migration. This reference queries stored content.

1. **CONTROLLER** Read `midi-bridge controller info > entities.json`. Resolve
   the named track or use `.selection.primary` to identify its owner. A selected
   automation point has the lane path and an `item` with its source and point ID.
   An Arranger Note Clip, Launcher Note Clip or automation lane has the owning
   track or group-channel path as its parent. If the prompt and metadata leave
   several plausible matches, ask the user which target they mean.
2. Find the requested lane within that owner, then read the complete track
   snapshot and filter it locally by the observed lane ID.

```sh
# For a selected track, Note Clip, automation lane or automation point.
jq -e '
  .selection.primary
  | {project, elements}
  | if (.elements[-1].kind | IN("clip", "launcherClip", "automationLane"))
    then .elements |= .[:-1] else . end
  | select(.elements[-1].kind | IN("track", "group"))
  | [.]
' entities.json > paths.json

jq -e --slurpfile owners paths.json --arg name 'Punish automation' '
  $owners[0][0] as $owner
  | [.. | objects | select(.path? == $owner) | .children[]
     | select(.kind == "automationLane" and .metadata.name == $name)]
  | if length == 1 then .[0] else error("Expected one matching lane") end
' entities.json > lane.json

midi-bridge controller content read --paths paths.json > before.json
jq --slurpfile lane lane.json '
  [.automation[] | select(.lane == $lane[0].metadata.id)]
' before.json > lane-curves.json
```

For channel 1 CC1, replace the lane-name predicate with
`.metadata.target.kind == "midiCc" and .metadata.target.channel == 0 and
.metadata.target.controller == 1`, keeping the automation-lane kind check.
`.automation[]` retains every placement once alongside `.clips[]`. For multiple
placements, resolve the intended one by source, placement and mapping.

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
