# Inspect once; apply a known disposition

Input: immutable `capture.json`. Output: either that same file for direct assembly,
or an editorial `interpreted.json` with concise `decisions`. Return to assembly in
the skill entrypoint. All JSON channels below are zero-based (0 = displayed 1).
Use jq commands directly; do not generate Python, shell or other helper scripts.

## Compact inventory — run first

```sh
jq '{excluded, clips:(.midi.clips|length),
  notes:([.midi.clips[].notes|length]|add // 0),
  tracks:([.midi.clips[]|{track,path}]|unique_by(.track)),
  channels:([.midi.clips[].notes[].note.channel]|group_by(.)|map({channel:.[0],notes:length})),
  slots:([.entities|recurse(.children[]?)|select(.kind? == "layerChain" or .kind? == "choice")|{id:.metadata.id,name:.metadata.name}]),
  controls:([.midi.automation[]|{lane,target,points:(.points|length)}]|unique_by(.lane))}' capture.json
```

Read exclusions, note channels, exact slot names and nonempty control identities.
Do not dump every note or point. Use only the relevant detail filter below.

| Observation | Action/default | Detail or edit |
| --- | --- | --- |
| Shared channel 0 CC, known technique names | Assemble directly; existing encoder handles them | No extra analysis |
| Unmarked/ambiguous slot name | Retain as remark; no semantic guess | Rename only with established meaning, recipe below |
| User-declared lyric phrase/reset | Prefix the named slot; phrase becomes remark, reset becomes command metadata | Slot rename recipe |
| Unknown plugin parameter | Leave unchanged; assembler embeds compact parameter residual | Control detail only if user supplies a mapping |
| Established CC assignment + matching units | Normalize/convert in copy; record law and loss | CC recipe |
| Explicitly excluded device/control | Remove exact IDs from copy; retain raw capture | Omission recipe |
| Nonshared CC with points | Assembly refuses unresolved channel scope | Inspect exact lane and routing; only promote to shared if evidence establishes shared intent, otherwise report blocker |
| Multiple lanes for same CC destination | Check overlap and authority; never pick by order | Duplicate filter; omit one only with documented authority |
| Conditional notes, spread, curved automation | Nominal notes and reported residual/approximation are default | Exception counts; do not simulate playback |
| Missing spelling/slurs/lyric timing | Accept reported editorial notation limits | Do not infer one syllable per note or authored slurs |

No runtime standards survey is needed. The owning architecture already records
MIDI2/MusicXML and MuseScore/music21 precedents. Consult a specific cited section
only for a concrete unsupported decision; otherwise preserve the residual and
finish. The [official CC list](https://midi.org/midi-1-0-control-change-messages)
and [cheatsheet](https://producersociety.com/midi-cc-cheatsheet/) identify controller
functions, not plugin knob assignments.

## Selective observation filters

Exact lane, including placement/source mapping, endpoint values and interpolation:

```sh
jq --arg id 'LANE_ID' '[.midi.automation[]|select(.lane==$id)]|unique_by([.lane,.clip])|map({lane,track,clip,placement,mapping,target,points:(.points|length),first:.points[0],last:.points[-1],shapes:([.points[]|{interpolation,curvature,spread}]|unique)})' capture.json
```

Exact track's slot routing (read actual Filter/Map, not All/Same slot properties):

```sh
jq --arg id 'TRACK_ID' '.entities|recurse(.children[]?)|select(.kind? == "track" and .metadata.id==$id)|recurse(.children[]?)|select(.kind? == "layerChain" or .kind? == "choice")|{id:.metadata.id,name:.metadata.name,channel:.metadata.channel,routing:[.children[]|select(.metadata.channelFilter? != null or .metadata.channelMap? != null)|.metadata]}' capture.json
```

Nonshared CC lanes with points; these require a scope decision before collapse:

```sh
jq '[.midi.automation[]|select(.target.kind=="midiCc" and .target.channel!=0 and (.points|length)>0)|{track,lane,target,first:.points[0],last:.points[-1],count:(.points|length)}]|unique_by(.lane)' capture.json
```

Potential duplicate destinations (placement/source may explain apparent duplication):

```sh
jq '[.midi.automation[]|select(.target.kind=="midiCc" and (.points|length)>0)]|unique_by([.track,.lane,.clip])|group_by([.track,.target.channel,.target.controller])|map(select(length>1)|map({track,lane,clip,placement,mapping,target,first:.points[0],last:.points[-1]}))' capture.json
```

Exception counts, not a playback reconstruction:

```sh
jq '{muted:([.midi.clips[].notes[].note|select(.muted)]|length),
  velocitySpread:([.midi.clips[].notes[].note|select(.velocitySpread!=0)]|length),
  conditional:([.midi.clips[].notes[].note.operators|select((.chanceEnabled and .chance<1) or (.occurrenceEnabled and .occurrence!="ALWAYS") or (.recurrenceEnabled and (.recurrenceLength!=1 or .recurrenceMask!=1)) or (.repeatEnabled and .repeatCount>0))]|length),
  shapedLanes:([.midi.automation[]|select(any(.points[];.curvature!=0 or .spread!=0))|.lane]|unique|length)}' capture.json
```

## Bounded edit recipes

Use exact IDs from this capture; substitute placeholders via `--arg`. Write to a
fresh file, then use it as the next recipe's input. Never redirect into the input
file. Preserve required fields, placement/source mapping and all untouched values.

### Rename established slot meanings

Use an exact supported technique name, or a user-declared lyric prefix such as
`[lyrics lang=la reset=20] et in ter-ra pax`. Spaces separate words; hyphens mark
syllables. Language/reset are optional. Reset key is an explicit declaration,
never a low-pitch heuristic. No plugin/Syllabuilder inspection or syllable timing
inference. Unknown names can simply stay remarks.

```sh
jq --argjson names '{"SLOT_ID":"DECLARED_NAME"}' --arg why 'Established slot meaning; literal phrase retained without inferred note alignment.' '
  .entities |= walk(if type=="object" and (.kind?=="layerChain" or .kind?=="choice") and .metadata.id?!=null then .metadata.id as $id | if $names|has($id) then .metadata.name=$names[$id] else . end else . end)
  | .decisions += [$why]' capture.json > interpreted.json
```

Put all established ID/name pairs in the same object to handle several slots in
one pass. Prior editorial names may be reused only after their source IDs and
raw names match the current capture; reuse interpretations, never old notes or
automation.

### Omit an explicitly excluded device and its identified automation

Set lane IDs to exactly the controls the user excluded. Removing the entity alone
is insufficient. Ordinary musical CC11 is not automatically an expression-pedal
parameter and must remain unless separately excluded.

```sh
jq --arg device 'DEVICE_ID' --argjson lanes '["LANE_ID"]' --arg why 'User-requested omission of identified device and automation; raw capture preserved.' '
  .entities |= walk(if type=="object" and has("children") then .children |= map(select(.metadata.id != $device)) else . end)
  | .midi.automation |= map(select(.lane as $id | $lanes|index($id)|not))
  | .decisions += [$why]' capture.json > interpreted.json
```

For a redundant lane only, use the same automation filter and decision append,
omitting the `.entities` clause. A single zero reset is removable only when its
role is established as redundant, not merely because its value is zero.

### Assign an established normalized control to a shared CC

Requires observed/documented musical equivalence, normalized 0–1 values and shared
intent. Do not map a vague “Dynamics” label to CC1 or CC11. Do not equate CC1 with
printed dynamics, or choir CC64 with piano pedalling. Preserve the source control
law: dB is not linear amplitude; bipolar pan and pitch semitones need other laws.

```sh
jq --arg lane 'LANE_ID' --argjson cc 11 --arg why 'Established shared expression assignment; existing normalized 0–1 values retained.' '
  .midi.automation |= map(if .lane==$lane then .target += {kind:"midiCc",channel:0,controller:$cc} else . end)
  | .decisions += [$why]' capture.json > interpreted.json
```

For a justified linear range `[lo, hi]`, additionally transform selected points
with `.points |= map(.value = ((.value-$lo)/($hi-$lo)))`, passing numeric `--argjson
lo`/`hi`. Check endpoints/range using the lane filter; reject values outside 0–1
rather than clamping. Matching a knob's range alone does not establish CC meaning.

For native cube-root amplitude `x`, amplitude is `x³`; M2-113 §3 squared CC7
response requires `y=x^(3/2)`. Only when that response and change of gain location
are acceptable, change the same selected lane to CC7 and add
`.points |= map(.value = pow(.value;1.5))`. Values must be 0–1. This preserves
anchors, not curved interpolation. Default for an unqualified nonlinear curve is
to retain the parameter residual; an explicitly accepted anchor-only approximation
must be recorded. Do not create a curve-fitting/densification script.

For a native CC already established to be shared, change only
`.target.channel=0` in the selected-lane recipe. Resolve duplicate destinations
before assembly. Do not promote receiver-specific controls just to bypass refusal.

## Completion boundary

Record one concise `decisions` entry per material interpretation: source function,
chosen meaning/control law and loss. Do not record every unchanged field. Assembly
handles bounded timing quantization, loops, nominal operators, linear sampling,
articulation names and residual encoding; agents must not reimplement these steps.
Return to assembly after this pass. Unavailable facts remain stated limits.
