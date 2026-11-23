# Export audio for a resolved passage

The **controller** renders audio through `midi-bridge controller export-audio`.
Use [session actions](session-actions.md) for OSC transport and playhead changes.
For an actual render, invoke [bitwig-export-audio](../../bitwig-export-audio/SKILL.md)
with the qualified project, Arranger bounds, mixer channel paths and intended
options. Export reads the specified range and channels independently of the
current cursor or Arranger selection. Confirm the intended saved project and
stopped, nonrecording transport before taking the mixer observation.

## Resolve the range and channels

- **Range.** Pass zero-based quarter-note **Arranger beats**. `start` is the
  first included boundary and `end` is the end boundary. In constant 4/4
  starting at Arranger bar 1, bars 1 through 3 are `start: 0, end: 12`.
  For other meters, pickups, loops or performed repeats, resolve the actual
  native occurrence and meter before converting measure labels. Use the
  [cue marker's native beat](cue-markers.md) when a cue names the boundary.
  Source clip offsets and score measure numbers require this resolution.
  Bitwig converts the beats to elapsed time using the project's tempo map.
- **Channels.** Run `midi-bridge controller mix read --output mix.json`.
  Pass a nonempty array of original `.tracks[].path` objects from that
  snapshot, including hierarchy and native IDs. Resolve duplicates before
  dispatch. A leaf path selects that track, a group path selects its combined
  bus, and a return needs its own path. The master is the mixer row whose
  `path.elements` is empty. Include the master when the listening question
  concerns the whole arrangement. Check active, mute and solo state for
  the intended comparison.

## Dispatch and verify

The input includes the complete fresh mixer observation as `expected`, the
resolved `tracks`, an unused absolute `directory` under an existing parent,
the bounds, and explicit audio options. This example selects one exact leaf
path for the first three bars of a 4/4 passage. Supply the observed native ID,
exact hierarchy and a fresh output directory.

```sh
jq -e --arg id "$track_native_id" --arg dir "$fresh_export_directory" '
  [.tracks[].path | select(
    (.elements | map(.name)) == ["Continuo", "Cellos"]
    and .elements[-1].nativeId == $id
  )] as $paths
  | if ($paths | length) != 1 then error("expected one cello track")
    else {expected: ., tracks: $paths, directory: $dir, start: 0, end: 12,
      sampleRate: 48000, bitDepth: "FLOAT32", dither: false,
      preFader: false, realTime: false}
    end
' mix.json > export.json

midi-bridge controller export-audio start \
  --input export.json --operation "$operation_id" --output export-start.json
```

If pending, use `export-audio status --operation "$operation_id" --output
export-status-01.json` with a fresh receipt filename for each poll. Reconcile
an unknown outcome before another start. Use the completed receipt's exact
channel-to-file mapping. Independently verify the WAV files, duration and
expected signal, then return that evidence to the listening task. The
[export owner](../../bitwig-export-audio/SKILL.md) gives the supported rates,
sample formats, tap semantics and failure checks.
