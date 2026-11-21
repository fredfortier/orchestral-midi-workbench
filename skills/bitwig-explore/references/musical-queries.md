# Query complete clip MIDI with jq

These filters operate on the same complete `get-all` JSON used for replacement.
They do not change the adapter scope or strip the retained file.

**CONTROLLER** Resolve relevant tracks, then read their complete content once
using [native navigation](arranger-navigation.md). Analyze locally below.

Per-clip statistics, including empty clips:

```sh
jq '[.clips[] | {path, notes:(.notes|length),
  keys:([.notes[].note.key]|unique),
  channels:([.notes[].note.channel]|unique),
  velocityRange:([.notes[].note.velocity]|[min,max])}]' before.json
```

Notes overlapping a source-beat interval (keep original note geometry):

```sh
jq --argjson from 24 --argjson to 32 '[.clips[] as $c |
  $c.notes[].note | select(.start < $to and .start+.duration > $from) |
  {path:$c.path,id,start,duration,key,channel,velocity,releaseVelocity,muted}]' before.json
```

Expression point counts:

```sh
jq '[.clips[] | {path, expressions:([.notes[].note.expressions |
  to_entries[] | {kind:.key,points:(.value|length)}] |
  group_by(.kind) | map({kind:.[0].kind,points:(map(.points)|add)}))}]' before.json
```

Keys do not prove spelling/harmony; channel numbers do not prove instrument
articulation. Constant velocity does not prove flat dynamics: inspect expression,
automation and documented receiver behavior. These count stored source events;
looped playback and shared sources require their native mappings. Keep complete
values for edits; presentation rounding is not quantization permission.
