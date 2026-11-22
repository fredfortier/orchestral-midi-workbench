# Controller

`midi-bridge` serves the Bitwig Java extension and exposes bounded musical
operations. Run `cargo run --manifest-path crates/Cargo.toml -p notes-transformer -- --help`
from the repository root.

[Architecture](ARCHITECTURE.md) · [Java adapter](adapters/bitwig-ctl/README.md)

## Launcher clips

Select one note slot in the Bitwig launcher, then use the slot observation to
create a four-beat clip.

```sh
midi-bridge controller launcher read > slot.json
midi-bridge controller launcher create --expected slot.json --beats 4 > created.json
jq '[.position.clip]' created.json > paths.json
midi-bridge controller midi get-all --paths paths.json > before.json
```

With a launcher slot selected, `midi get-all` also reads its contents directly.
Copy the snapshot, edit its notes or automation points, and submit the complete
desired snapshot with `midi replace --expected before.json --input after.json`.
To add a CC curve, append an automation entry with the track `owner`,
`"placement": "launcher"`, the observed note clip ID in `noteClip`, a
`midiCc` target, and points beginning at beat zero. Leave the native automation
IDs out of a new entry. Subsequent reads return those IDs for point replacement.
When the target is new to the track, Bitwig also initializes its track-wide
automation lane with a beat-zero arranger fallback.

Launcher scene indices start at zero. Clip and automation times use local
quarter-note beats. Track-scoped reads and MIDI2 capture use the arranger.
