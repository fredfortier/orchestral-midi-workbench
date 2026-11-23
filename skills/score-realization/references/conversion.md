# Compile the musical rendition

Use the installed musical converter and check its `--help`. Invoke its exact
path because other tools also use the name `convert`.

Inputs: working MusicXML/MXL, explicit supported interpretation policy and a
reviewed rendition plan. Output: MIDI2, one structured companion, report and
source-preservation artifacts. Internal inputs are retained evidence; playback
does not execute them or PERFORMANCE.md prose.

```sh
"$score_converter" "$working_score" --inspect-rendition \
  --policy "$policy" --output "$run/source-notes.json"
"$score_converter" "$working_score" --to midi2 \
  --policy "$policy" --rendition-plan "$rendition_plan" \
  --output "$project/performance.midi2" \
  --companion "$project/performance.companion.json" \
  --report "$run/conversion-report.json"
midi-bridge controller midi decode \
  --input "$project/performance.midi2" \
  --companion "$project/performance.companion.json" > "$run/preflight.json"
```

Variables are resolved task paths, not literal placeholders to execute. Preserve
prior affected artifacts/inputs in a fresh archive before publication. Never add
`--force` automatically. A plain score-to-MIDI2 conversion can test parsing but
does not supply a complete patch-bound rendition without its explicit plan.

## Rendition inputs

- The plan contains `rendition` and `rules`, with optional
  `initialControllers` and `passages`. In `rendition`, use version 1 and
  declare `tracks`, `instruments`, `slots`, and `bindings`. Name each track
  and slot, and give each a stable ID. Give each instrument its ID, exact
  absolute `nki`, player identity/version/format, and `settings`. The slot
  identifies its track, instrument, and full `inputChannels` set. Each
  binding identifies its `route` (group and channel), `track`, destination
  `channel`, `instrument` and `slot`. JSON channels start at zero.
- Match the intended musical track, named receiver slot, exact patch, and
  admitted note channels. Each occupied MIDI route needs a binding. Several
  note channels can share a declared slot when its complete channel set admits
  them. Preserve original portable routes and use [assignment](../../kontakt-setup/references/assignment.md)
  for the native mapping.
- Optional `passages` entries record `id`, `sourcePrefixes`, `track`,
  `instrument`, `expectedNotes` and `reason`. When supplied, they must cover
  every performed occurrence exactly once. Establish expected counts against the
  score before compiling.
- Match each note to exactly one musical rule. Choose plain key velocity or
  a documented crossfade law. Each rule matches `sourcePrefix`,
  `articulation` and `descriptor`, then sets `route` and `dynamics`.
  Read actual source identities from the companion's `note_sources`.
  For `dynamics`, use `kind: "keyVelocity"` or `kind: "crossfade"` with
  the documented `controller`, `keyVelocity` and `sourceLaw`. Add the
  independent `expression` controller and value only when intended.
  Beat-zero `initialControllers` entries use a bound `route`, controller
  number and literal MIDI 2.0 value. With explicit `channelHold` support, dynamics
  must start at note onset and overlapping notes that share a channel must agree
  throughout the overlap. Keep sampled dynamics and expression separate.

- Build rule selectors from `--inspect-rendition` on the same score and policy.
  Use `sourceVoice` for distinct written voices on a stave. Specify any
  reviewed note endings in `noteEdits` and any channel-1 curves in
  `trackCcCurves` with their companion `controlRoutes`. Match the edited
  occurrences and CC points against the decoded performance. For a fixed attack
  with a separately planned track curve, use `fixedAttack` and its documented
  velocity. Archive preview pan/volume and direction prose with
  `archiveScorePreview` when the realization replaces the preview mix.

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
supported operations and revalidate its assignments and musical content.
If the installed tools cannot express the required change, record it as pending.

Before any host write, inspect the complete generated device schedule against
the shared mandatory execution boundary. Encoded controllers or companion settings
must agree with documented INSTRUMENT.md controls and native track-level MIDI
automation, including channel routing and units; standard slot Filter/Map routing is owned by kontakt-setup. They do not
authorize MIDI CC sender devices or receiver customization. Preserve unsupported
musical intent explicitly and leave the affected placement pending.
