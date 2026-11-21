---
name: bitwig-export-audio
description: Export an open Bitwig arrangement as synchronized audio for listening, mixing analysis or delivery, with exact track/group/return/master filtering, range, sample rate, WAV bit depth, dither, pre/post-fader and real-time options. Uses the existing native controller export operation and verifies its outputs.
---

# Export native Bitwig audio

Read [the shared workflow](../WORKFLOW.md). The procedure and limits below own
this skill’s export behavior; [controller MIXING.md](../../controller/MIXING.md)
owns the executable operation schema.
Use `controller export-audio`, which invokes Bitwig's native Export Audio task.
Do not emulate rendering, bounce clips destructively or solo tracks one by one.

## Control flow

```text
Resolve intended project + export purpose/range/owners
  -> fresh controller mix read
  -> filter exact observed paths (or explicitly choose all wanted channels)
  -> select supported export options + fresh destination
  -> export-audio start once; retain original operation
       pending -> export-audio status, same operation, fresh receipt filename
       failed/unknown -> inspect original status, native state and output; no replay
       completed -> independently verify files -> return to caller
```

Resolve project instructions and confirm current project with `controller info`.
Playback must be stopped; surrounding transport/window actions use the qualified
native keyboard route. A request for advice alone does not dispatch an export.

```sh
midi-bridge controller mix read --output mix.json
midi-bridge controller export-audio start --input export.json --operation export-01 --output export-start.json
midi-bridge controller export-audio status --operation export-01 --output export-status-01.json
```

Use a new output filename for every status receipt. Poll without resubmitting the
start command. Choose operation IDs made of ASCII letters/digits, colon, slash
or hyphen; preserve the original ID through timeouts/disconnection. On adapter
restart, retained native operation state may be unavailable: reconcile files and
host state rather than dispatch the old intent again.

`export.json` contains all these fields:

| Field | Meaning / selection |
| --- | --- |
| `expected` | Complete fresh mixer observation, unchanged |
| `tracks` | Nonempty array of exact `.tracks[].path` values from that observation |
| `directory` | New absolute directory under an existing parent; no overwrite |
| `start`, `end` | Nonnegative quarter-note arranger beats, end greater than start; native tick resolution; at most one hour |
| `sampleRate` | Explicit supported rate: 44100, 48000, 88200, 96000 or 192000 Hz |
| `bitDepth` | `INT16`, `INT24` or `FLOAT32`; output format is WAV |
| `dither` | Boolean; off for FLOAT32; choose integer delivery dither deliberately |
| `preFader` | Boolean: false preserves the selected channel's mixer balance; true selects its pre-fader tap |
| `realTime` | Boolean; true when external processing requires real time |

Filter by exact names/hierarchy offline, then pass the original qualified paths,
including native discriminators. Do not use substring matches silently or assume
group selection means export every child. A group is its combined output; a
master is the complete master output; return tracks are separate choices. In the
mix snapshot only, the project-root path with empty `elements` names the master.
For a sounding-mix analysis, filter deactivated tracks and descendants of
deactivated ancestors using `state.active`; mute state alone is insufficient.
Keep explicitly requested inactive exports separate from sounding comparisons.

For mixing analysis default to post-fader FLOAT32, no dither, engine-matched
explicit sample rate, and a common start/end including complete release/reverb
tails. These are workflow defaults, not delivery requirements. Do not normalize
tracks independently. Only select pre-fader when that different measurement tap
is intended. Pre-fader ignores mixer volume automation; it does not mean raw
instrument output. Leaf exports exclude ancestor/master processing; retain the
master and relevant groups/returns separately and do not double-count them.

After native completion, independently inspect actual file headers, channels,
sample rate, encoding, frame count/duration, signal/peak and tail behavior. Use
the receipt's exact owner-to-file mapping. A file's presence or a completed task
does not establish correct instrument sound. Sample-internal variation can make
repeated renders differ; waveform byte identity is not an acceptance test. Silent
parts may be musically correct; unexplained silence routes back to source/routing
diagnosis.

Return filenames, options, range, completion/errors, measurement findings and
limits to the caller. Do not claim an automatic tail, normalized output, MP3/
FLAC/Vorbis/Opus export, arbitrary sample rate, open-folder preference control,
or generic render support: those are outside the current command contract.
