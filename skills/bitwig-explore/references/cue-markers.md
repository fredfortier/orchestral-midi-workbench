# Read current Arranger Cue Markers

**CONTROLLER** Use the marker-only read for listing existing cues; no clip capture, UI selection
change or performance assessment is needed.

```sh
cue_read_dir=$(mktemp -d /tmp/bitwig-cues.XXXXXX)
midi-bridge controller cues read --output "$cue_read_dir/cues.json" > /dev/null
jq -e '.type == "cuesResult" and .result.error == null and
  (.result.observation.markers | type == "array")' "$cue_read_dir/cues.json"
jq '.result.observation | {document, projectUuid, revision,
  markers: [.markers[] | {id, name, time}]}' "$cue_read_dir/cues.json"
```

`--output` is required and must name a fresh file. The CLI also prints the full
result, so redirect stdout when retaining the file; keep stderr visible. Stop if
the command or result check fails; do not report a failed read as zero markers.
Use the configured controller port and an explicit adapter when required by the
connection; inspect `midi-bridge controller --help` for those options.

The observation identifies the current native document, project UUID and revision.
Each marker has its native `id`, exact `name` and `time` in arranger quarter-note
beats from arrangement start. Preserve all names, including markers without a
`PC` key. An empty `markers` array in a successful observation means no markers.
Report the count, names and positions with units. Convert to bars only with the
actual metrical map; beats are neither bar numbers nor seconds.

This read establishes current stored markers, not saved persistence, playback
jumps or musical interpretation. For requested cue creation or interpretation,
return the observation to [performance-cues](../../performance-cues/SKILL.md).
Do not create, rename, move or delete markers as part of a list request.
