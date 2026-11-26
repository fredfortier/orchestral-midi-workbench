# Score and performance

## Turn the score into a playable ensemble

```text
saved project + original score + user direction
  -> resolve the orchestration checklist
     missing material preference -> ask once, continue independent research
     delegated choice -> research, choose and record the working default
  -> instrument intake for the intended patches
     review exact stock controls and baselines in INSTRUMENT.md
     required effect unresolved -> research or compare eligible patches
                              -> still unresolved: mark affected scope pending
  -> PERFORMANCE.md musical anchors -> convert -> validate and decode
     check every required control against the emitted MIDI2 + companion
  -> native import -> independent note and CC readback
     -> Kontakt assignment and routing readback ---+
     -> requested cues and marker readback -------+
                                                -> verify required effects, save and audition
```

1. Pass the supplied bytes and destination to [score intake](../../score-intake/SKILL.md).
   Require the saved native path, matching source bytes and project
   instructions. Reuse an existing matching workspace.
2. Resolve the orchestration checklist below. When the brief calls for a
   researched expressive design, use the
   [composition and orchestration framework](../../performance-cues/references/musicology-research.md#evaluate-a-proposed-expressive-design)
   with the source score and intended performed form. Carry the musical reasoning
   and listening criteria into receiver selection and realization. Live cue work
   uses the native MIDI after placement.
   Run or reuse verified
   [instrument intake](../../instrument-intake/SKILL.md) for every intended
   exact patch. Require its library `INSTRUMENT.md` to document the stock MIDI
   controls, including CC1, CC2, CC11, other relevant CCs, velocity and pitch
   bend, with separate stock dynamics and expression baselines and any unknowns
   explicit. Review the exact entries before committing receiver choices. If
   a required effect lacks control evidence, return to intake or compare an
   eligible stock alternative within the user's constraints.
3. Give the saved identity, working score, exact patch capabilities and decisions
   to [score realization](../../score-realization/SKILL.md). For every track and
   articulation, explain in `PERFORMANCE.md` how documented standard MIDI inputs
   support the intended dynamics, foreground, phrase direction and release.
   Anchor the required curves to the composer's markings and the musicological
   analysis in PERFORMANCE.md. Follow the shared
   [expression control specification](../../bitwig-edit/references/dynamics-and-expression.md#apply-the-distinction).
   Follow the [sparse curve procedure](../../bitwig-edit/references/automation-edits.md#draw-sparse-musical-curves)
   when encoding applicable shared channel-1 curves.
   Finish and review the converter's MIDI2 and companion before creating
   musical tracks, clips or receivers. The bundle holds performed note gates,
   attacks, note routes and track controls. Place it with the admitted native
   import route and verify the transfer before assigning instruments.
4. Give the exact documented receiver bindings to
   [Kontakt setup](../../kontakt-setup/SKILL.md). Use the returned Layer target
   for slot ensure. Read routing and assignment outcomes, and carry the actual
   patch/control map into listening. Give a settled patch binding to integrated
   assignment directly.
5. Once the native MIDI readback agrees with the bundle, place any requested
   markers through [performance cues](../../performance-cues/SKILL.md) and read
   them back. Marker placement can precede Kontakt loading.
6. Check every required musical effect against the documented capability, the
   PERFORMANCE.md anchors, the decoded curve or note data and native readback.
   An unresolved effect keeps that scope pending. Save the intended project if
   the request includes saved delivery. Verify
   the file and affected native scope. Use a representative excerpt and the
   requested listening or [audio export](../../bitwig-export-audio/SKILL.md)
   evidence to establish playback. A saved workspace, editable notes, installed
   receivers and an audible ensemble are distinct deliverables.

Use documented standard MIDI CCs for continuous receiver control in the MIDI2
orchestration. Record any stock mapping that remains unknown and its musical
effect on the rendition. Consider a patch macro or
other exceptional control only after the MIDI2 rendition is complete and the
specific musical context warrants a separately authorized refinement.

### Orchestration checklist

Use the user's brief, the complete score and any existing `PERFORMANCE.md`.
Record each decision that affects the rendition beside its evidence.

- [ ] **Edition and performed form.** Resolve repeats, cuts, Da Capo, Fine,
  pickups, fermatas and ties. Compare printed directions with playback directives.
  Identify editorial staves, independent lines and doublings.
- [ ] **Style, pulse and pitch.** Identify era, genre, forces and any reference
  conductor or recording. Choose pulse, subdivisions, tempo changes, pitch
  and tuning. Match edition and performed form before inferring tempo from audio.
- [ ] **Forces and texture.** Check sounding transpositions, tessitura,
  polyphony, continuo texture, foreground and support. Resolve vocal sections,
  language, vowels or text fidelity, and melismas.
- [ ] **Articulation and line.** Complete the
  [articulation research and map](../../score-realization/references/interpretation.md#research-and-map-articulations).
  Carry the score occurrences, cited evidence, playing intent and exact
  receiver choices into the rendition.
- [ ] **Intensity and expression.** Identify tension, arrivals, cadence and
  release. Choose sampled dynamics, attack velocity and independent expression
  separately. Review the patch control and baseline for each, and justify the
  initial level and phrase contour against the relevant musical evidence.
  Compile and verify the chosen articulation routes and control curves.
- [ ] **Verification excerpts.** Pick passages that expose difficult entries,
  transitions, simultaneous voices, range extremes and the ending.

Ask focused questions for missing preferences that materially change the result
and cannot be resolved from context. Examples include conflicting reference
performances, an editorial doubling, or text fidelity versus fixed vowels.
Offer a recommended default and continue independent research. Resolve delegated
choices using an applicable score edition, piece-specific scholarship, the
reference recording and exact patch documentation. Record the defaults and
their uncertainty. Follow [cue research](../../performance-cues/references/musicology-research.md)
for interpretation and [score realization](../../score-realization/references/interpretation.md)
for what to encode. Carry settled decisions forward directly.

## Direct the performance from its form

1. With the intended project open and the relevant MIDI present, pass its
   native clip scope and the musical question to
   [performance cues](../../performance-cues/SKILL.md). Keep the verified cue
   keys, source-to-arranger positions, phrasing and foreground relationships.
   For a proposal, return the directions and listening criteria. Place or update
   markers when that result is in scope.
2. When the brief also calls for an expression plan or implementation, pass the
   selected score events, performed occurrences, affected lines and listening
   criteria to [realization's interpretation procedure](../../score-realization/references/interpretation.md#translate-an-expressive-direction-into-receiver-behavior).
   Carry the resolved exact receiver and current routing into that handoff.
   The result specifies the controls and anchors for the intended sound.
   Use the full [score realization](../../score-realization/SKILL.md) workflow
   for an authorized generated rendition. Send a bounded native revision to
   [Bitwig Edit](../../bitwig-edit/SKILL.md) with qualified owners, the scoped
   control plan and the preservation boundary. If the request is a proposal,
   return the plan at this point. Keep the remaining effect pending if the
   required receiver behavior cannot be established.
3. For requested acoustic verification, carry the same criterion and passages
   into [audition in context](audition.md). If the effect concerns blend,
   foreground or climax hierarchy, include the relevant ensemble. A patch
   probe can settle an individual mapping. If the mapping works and the
   balance fails, return the result to the owner of the affected musical
   choice. A failed interpretation returns to Cues, a failed translation to
   Realization, and a mismatched native edit to Bitwig Edit.
4. For a requested balance or playback investigation, give
   [mixing](../../bitwig-mixing/SKILL.md) the same cue/passages, relationship and
   listening question. For a reported fault, identify the capture source and
   reproducible passage before changing routing or render settings. Compare
   results at the same passage and report audible and measured evidence.

## Configure and direct sung phrases

1. Resolve source lyric underlay, exact voices, independent controls and
   verified stock receiver bindings. Send missing capability facts to
   [instrument intake](../../instrument-intake/SKILL.md) and stock assignment
   to [Kontakt setup](../../kontakt-setup/SKILL.md).
2. Give each exact open receiver and its source phrase to
   [syllable builder](../../kontakt-syllable-builder/SKILL.md). Carry forward the
   resulting lyric coverage, reusable phrase names, loss notes, reset/hold
   behavior and control map.
3. Pass those facts to [realization](../../score-realization/SKILL.md) or
   [editing](../../bitwig-edit/SKILL.md) for scoped note/channel/control writes.
   Look for native note and curve readback and audition representative entrances
   and phrase restarts. Reconcile any old curve with the replacement patch's
   documented control law.
