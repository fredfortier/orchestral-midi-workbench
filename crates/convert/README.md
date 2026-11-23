# MusicXML and MIDI 2.0 converter

`convert` validates MIDI 2.0 and converts between MusicXML/MXL and MIDI Clip Files.
Run `cargo run --manifest-path crates/Cargo.toml -p musicxml-midi2-convert -- --help`
from the repository root.

## Prepare an orchestrated rendition

Use the same working score and rendering policy for inspection and conversion.
Keep the original score alongside the project. The [working example](examples/violin.rendition.json)
compiles against [violin.musicxml](examples/violin.musicxml). Its placeholder
receiver path demonstrates offline encoding. Use your verified NKI paths for
native placement and setup.

```sh
convert working.mxl --inspect-rendition --policy policy.json --output score-notes.json
convert working.mxl --to midi2 --policy policy.json --rendition-plan plan.json \
  --output performance.midi2 --companion performance.companion.json --report conversion.json
convert performance.midi2 --validate --report validation.json
midi-bridge controller midi decode --input performance.midi2 \
  --companion performance.companion.json > decoded.json
```

`--inspect-rendition` defaults to JSON on stdout. Its `notes` are performed
occurrences, with exact `source`, `articulation` and `descriptor` values for
building rules. `sourceVoices` maps written note paths, before the `#occurrence`
suffix, to MusicXML voice strings. `--policy` is optional. For example,
`{"default_tempo":56}` sets quarter notes per minute for a score lacking tempo.

| `plan.json` field | Use |
| --- | --- |
| `rendition` | Name the tracks, exact NKI resources, slots and bindings. `bitwig.groupName` is optional and its absence targets the saved project root. Portable UMP group/channel and native destination channel are distinct and zero-based. |
| `rules` | Match each performed note exactly once with `sourcePrefix`, `articulation`, `descriptor` and optional `sourceVoice`. Route to a binding and choose `keyVelocity`, `fixedAttack` with `keyVelocity`, or the documented `crossfade` law. |
| `noteEdits` | Optionally set resolved endings for exact `{source,onset,key,end}` occurrences. Beat values are `[numerator,denominator]` quarter-note beats. Keep written notation in the source. |
| `rendition.controlRoutes` | Declare `{track,route}` for each track with shared MIDI controls. Use portable route channel `0` for native channel 1. Keep independent note routes in `bindings`. |
| `trackCcCurves` | Give `{route,controller,points:[{time,value}]}` on declared shared routes. Start each curve at beat zero, in increasing rational beat order. Values are literal unsigned 32-bit MIDI 2.0 values. Each point holds until the next event. Sample a continuous musical envelope explicitly. |
| `archiveScorePreview` | Set `true` when the planned rendition replaces the score viewer's initial volume and pan. The companion retains these source settings and channel-addressed direction prose in the archived score. |

For a sounding MIDI 1.0 input value `v` in 0–127, use the reviewed value
`round(v * 4294967295 / 127)` when generating a normalized controller curve.
The converter rejects unmatched rules, ambiguous routes and inconsistent timing.
Check the conversion report and the decoded MIDI against the intended note and
control decisions before opening the Bitwig import step. Re-use a new output
location for each attempt. `--force` explicitly replaces existing files.

[Architecture](ARCHITECTURE.md) · [Verification inputs](../../docs/references/README.md)
