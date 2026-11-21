# Clip MIDI controller

[ARCHITECTURE.md](ARCHITECTURE.md) is binding. The service reads and replaces
complete native Bitwig clips: notes, per-note expressions and every owning-track
automation lane. Project structure is a named, typed folder hierarchy. Bounded native `mix read|set` and `export-audio start|status` are specified in
[MIXING.md](MIXING.md). General Bitwig control/import/render/audio-analysis and
reflected property graphs remain retired.
[WORKPAD.md](WORKPAD.md) records actual evidence and remaining gaps.

From `crates/`:

```sh
cargo build -p notes-transformer
(cd controller/adapters/bitwig-ctl && ./mvnw verify)
./target/debug/midi-bridge serve
./target/debug/midi-bridge controller info
./target/debug/midi-bridge controller midi get-all --paths paths.json --output before.json
jq '(.clips[].notes[].note.velocity) = 0.6' before.json > after.json
./target/debug/midi-bridge controller midi replace --expected before.json --input after.json
```

Omit `--paths` for actual native selection. Reads always include all clip notes and
automation; use jq offline for stats and edits. Replacement preserves geometry,
ownership and selection, validates full expected content and returns independently
read complete content. Native IDs correlate existing notes/points; new values use
unique temporary IDs. Empty notes clears notes while preserving automation.
Read the architecture for exact fields, units and native fallback semantics.

Use `controller automation set` with track/channel/controller for native track
MIDI automation (channel 0 for displayed channel 1; values 0–1). Instrument Layer
slot ensure sets All/Same and stock Channel Filter → Channel Map, admitting
shared channel 1 plus the complete requested note-channel set and mapping all
to 1. Assignment independently reuses/inserts the instrument. Shared controls
use one channel-1 track lane after delivery qualification. Device parameter/Remote
Control discovery is explicit opt-in beyond conventional MIDI. The legacy
MIDI CC insertion helper is not needed for conventional MIDI automation.

`controller selection read` inspects native membership, primary, editor and mode.
`controller selection set --input request.json` replaces selection absolutely
using returned qualified paths and fresh authority; time mode supplies arranger
beat bounds. It does not establish keyboard focus. See the
[navigation workflow](../../skills/bitwig-explore/references/arranger-navigation.md).
Complete snapshots store `.automation` once beside `.clips`, with `.owners` and
exact explicit `.paths`; a track need not contain note clips to own automation.
Structural `info --path` resolves its subtree directly and omits parameter catalogues.

Other core commands: `controller note`, `point`, `instrument`, `cues` and original
`operation` status. `controller midi decode` authenticates offline MIDI2/companion;
it does not orchestrate host placement. Current `--help` owns exact CLI arguments.

HTTP defaults to loopback8789; native TCP to8788. Configure them independently with
`serve --api-port`/`--port`. CLI `controller --port` selects HTTP. Deploy matching
Java/Rust wire versions with [install-extension.sh](scripts/install-extension.sh);
installation alone does not reload running extension classes. Connection lifecycle
is explicit via `adapter bitwig connection`; never replay a mutation after timeout.

[skills/WORKFLOW.md](../../skills/WORKFLOW.md) owns musical task routing and the
qualified native keyboard workflow around the core tools. [RUNBOOK.md](RUNBOOK.md)
owns ephemeral qualification; no durable unit tests are retained. Optional isolated
virtual display/PipeWire setup is in [MANAGED_BITWIG.md](MANAGED_BITWIG.md). Use
Computer for Bitwig UI work; Xpra is forbidden. [kontakt-ctl](adapters/kontakt-ctl/README.md)
only loads the exact NKI into the controller-owned editor.

## Mixing and native export

```sh
midi-bridge controller mix read --output mix.json
midi-bridge controller mix set --input balance.json --output mix-after.json
midi-bridge controller export-audio start --input export.json --operation export-01 --output started.json
midi-bridge controller export-audio status --operation export-01 --output status-01.json
```

The [export skill](../../skills/bitwig-export-audio/SKILL.md) owns option selection,
exact-path track filtering, status reconciliation and independent audio checks.
The [mixing skill](../../skills/bitwig-mixing/SKILL.md) owns musical technique and
acceptance; native settings alone do not establish audible quality.

The 6.1 native export callback subclass compiles against the installed host JAR
as a provided build input. Override its location with
`-Dbitwig.runtime.jar=PATH_TO_INSTALLED_BITWIG_JAR` when needed. Application classes
are not packaged. The subclass runs the host's Export Audio task and captures its
final success/error callback; it does not inspect UI pixels.

## Standalone MIDI2 and score export

```sh
midi-bridge controller midi capture --output capture.json
midi-bridge controller midi assemble --input capture.json --output performance.midi2 --report assembly.json
./target/debug/convert performance.midi2 --validate --report validation.json
./target/debug/convert performance.midi2 --to mxl -o score.mxl --report notation.json
```

Capture excludes inactive tracks by default. The [export contract](EXPORT_MIDI.md)
owns scope, routing and losses; the [export skill](../../skills/bitwig-export-midi2/SKILL.md)
guides interpretation of a separate working JSON copy. No companion is required.
