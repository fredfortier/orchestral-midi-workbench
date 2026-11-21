# Compile the musical rendition

[RENDITION.md](../../../convert/RENDITION.md) and actual CLI help own accepted
schemas/limits. Run from crates with matching workspace binaries; bare `convert`
may resolve to ImageMagick. Use `./target/debug/convert` or the explicit installed
workspace binary. Build only when necessary; no new parser or ad hoc MIDI writer.

Inputs: working MusicXML/MXL, explicit supported interpretation policy and
`adapt::Plan` rendition plan. Output: MIDI2, one structured companion, report and
source-preservation artifacts. Internal inputs are retained evidence; playback
does not execute them or PERFORMANCE.md prose.

```sh
./target/debug/convert "$working_score" --to midi2 \
  --policy "$policy" --rendition-plan "$rendition_plan" \
  --output "$project/performance.midi2" \
  --companion "$project/performance.companion.json" \
  --report "$run/conversion-report.json"
./target/debug/midi-bridge import inspect \
  --authority "$project/performance.companion.json" \
  --input "$project/performance.midi2" \
  --companion "$project/performance.companion.json" \
  --output "$run/preflight.json"
```

Variables are resolved task paths, not literal placeholders to execute. Preserve
prior affected artifacts/inputs in a fresh archive before publication. Never add
`--force` automatically. A plain score-to-MIDI2 conversion can test parsing but
does not supply a complete patch-bound rendition without its explicit plan.

The converter resolves supported score semantics and applies declared source-note
rules to receiver routes/control laws. Fixed-attack + CC-dynamics adaptation occurs
here, not during native import. Required initialized controllers and evolving
dynamic curves must use supported explicit rules. No universal CC1 meaning, gate,
breath, bow stroke or smoothing is inferred. The importer transfers encoded values.

Compare PERFORMANCE.md's binding purposes, stock variants and native routing with
the exact rendition inputs and emitted companion. Verify resource locations and declared versions,
route coverage, source-part provenance, repeated occurrences, performed timing,
attack/release and remaining expression. Review independent passage assertions
and all loss/unsupported diagnostics. Musical suitability and audible support require independent evidence. The companion's project name/parent determine
native destination and must match the actual intended saved path.

Reconvert when musical messages or receiver control laws change. For a patch-only
revision with unchanged performed messages, revise the typed companion through
supported APIs and revalidate its assignments and musical content. New schema or
unsupported interpretation mechanisms require implementation, not guessed fields.

Before any host write, inspect the complete generated device schedule against
the shared mandatory execution boundary. Encoded controllers or companion settings
must agree with documented INSTRUMENT.md controls and native track-level MIDI
automation, including channel routing and units; standard slot Filter/Map routing is owned by kontakt-setup. They do not
authorize MIDI CC sender devices or receiver customization. Preserve unsupported musical intent explicitly and repair its
owning compiler/controller affordance within scope; do not flatten it or execute
a forbidden schedule. Host operations use existing midi-bridge commands only.
