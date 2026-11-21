# Place and reconcile performance landmarks

[Architecture](../../../controller/ARCHITECTURE.md#purpose-and-execution-boundaries) owns
intended marker scope. The cue map and current native note/source mapping supply
musical positions; score bar numbers are not native beat coordinates.

## Identity and placement

Reuse established musical keys. The current CLI convention is `PC01 · Opening`:
`PC` plus 2–4 digits, middle-dot separator, and a short label within 64 UTF-8 bytes.
Keys are permanent; do not renumber them to restore chronology. Put reasoning,
citations and part-specific directions in PERFORMANCE.md, not marker names.

Place preparations where action begins, arrivals where they occur, and releases
where useful. Do not quantize all landmarks to barlines. Equal-time functions
normally share a cue. Derive beat positions from the actual arranger/metrical map.

## Execute and verify

Inspect `controller cues --help` and each subcommand's help. Establish the exact
project/generation, current markers and intended missing keys before mutation.
The current `create` operation is additive (1–128 missing keys); it refuses
collisions and unsupported host state. Move/rename/delete are different operations,
not permission to delete and recreate all markers.

Author the requested `{markers:[{name,time}]}` values independently of readback.
Use a fresh operation ID and retain intent/result. Independently verify requested
names/positions and preservation of existing markers. Update PERFORMANCE.md only
for actual verified coverage. A read or receipt does not prove saved persistence.

Cue reads and writes use their native scoped marker list and document/revision;
they do not export project graphs. Preserve every pre-existing marker and verify
new exact names/beat positions through independent `controller cues read`.

On failure or lost acknowledgment, reconcile the original operation ID through
supported status/readback. Insertion may be partial; no automatic rollback or
replay is implied. Reobserve after arrangement or history changes before revising
placement. An offline comparison attests its retained inputs, not current state.

## Performance use

[Bitwig's marker documentation](https://www.bitwig.com/userguide/latest/playing_back_the_arranger/#cue_markers)
explains ruler/Sections navigation and triggering. Placement and quantized playback
jumps are separate outcomes. Creating cues does not authorize starting playback
or changing launch quantization.
