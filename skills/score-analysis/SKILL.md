---
name: score-analysis
description: Analyze a current Bitwig composition or supplied MusicXML/MXL for orchestration, tessitura, melodic contour, rhythmic activity, tonal movement, Roman numerals, suspensions and voice leading. Use for musical diagnosis and basic composition analysis; excludes live musical edits, audio mixing, score realization and literature-led performance cue creation.
---

# Analyze a composition with music21

Answer the musical question from an identified score, using the smallest relevant
analysis. This is symbolic analysis; it does not establish audible balance,
receiver behavior, historical correctness or artistic quality.

## Acquire → load → select → report

1. **Acquire current evidence.** For the active Bitwig piece/group, open
   `~/.agents/skills/bitwig-export-midi2/SKILL.md`
   ([repository equivalent](../bitwig-export-midi2/SKILL.md)). Invoke its capture,
   interpretation, MIDI2 admission and MXL steps for the requested scope; return
   here with the fresh MXL, assembly/notation reports and editorial capture path.
   Carry forward user exclusions and established interpretations. Do not assume an
   older case-study export is current. An MXL produced from a fresh observation
   in this same task may be reused. For a supplied score or explicitly historical
   analysis, use that file without exporting or launching Bitwig.
2. **Load once.** Open [loading and common objects](references/loading.md).
   Use its ephemeral `uv run --with` context. Check part identities, duration,
   attack/segment counts and known export losses before interpreting results.
   A failed import or mismatched scope blocks dependent musical conclusions.
   Report the affected parts and continue with any verified scope.
3. **Choose by intent below.** Read only the selected leaves; combine them only
   when the question needs it. For “basic analysis,” use the loading inventory,
   register/texture table and one harmony excerpt, with anomalies first. For a
   vague “is this working?”, start with integrity and texture; ask only if a
   musical goal is needed to choose further analysis. Unmatched requests stay
   within these observations or return to the owning workflow; no automatic
   literature search, comprehensive theory survey or new analysis framework.
4. **Finish.** Give a brief musical finding, its part/measure/beat and evidence,
   then the material limit or plausible alternative. State whether a number
   counts attacks, segments, pitch-beats, elapsed beats or sampled verticalities.
   Save artifacts only when requested or needed to substantiate a finding; name
   the input and music21 version. Do not write back to the MXL/DAW. Report blockers
   and stop unsupported branches while completing independent requested checks.

## Intent router — open the matching reference

| Musical question / language | Route and expected answer |
| --- | --- |
| “Did export lose notes?”, “wrong bar count”, “tiny note”, “ties or clipped fermata?”, “compare with source” | [Integrity](references/integrity.md): counts, boundaries and located discrepancies before theory |
| “Too dense/thin?”, “who carries the texture?”, “tessitura”, “range”, “doubling”, “does the climax actually build?”, “orchestration overview” | [Register and texture](references/register-texture.md): weighted register, occupancy and concurrent-part timeline; no acoustic masking verdict |
| “Is this line singable?”, “awkward leap”, “contour”, “stepwise motion”, “melodic peak” | [Melody](references/melody.md): one identified line, interval distribution and located leaps; range alone cannot prove playability |
| “Static/busy?”, “attack density”, “pulse”, “rhythmic variety”, “strong/weak beats”, “syncopation” | [Rhythm](references/rhythm.md): attacks, durations and metric placement; syncopation needs contextual review |
| “What key?”, “tonal plan”, “modulation”, “Roman numerals”, “applied dominant”, “cadence”, “harmonic rhythm” | [Harmony](references/harmony.md): key alternatives and selected bass/chord readings; separate observed sonority from inferred function |
| “Why this tension?”, “suspension/preparation/resolution”, “crossing”, “parallel fifths/octaves”, “independent voices”, “contrary motion” | [Counterpoint](references/counterpoint.md): named voices and successive verticalities; candidates, not automatic rule violations |
| “Show it visually” | Use the plot recipe in [register/texture](references/register-texture.md) or the timeline from the relevant leaf; generate one informative plot, not every available graph |

Lyrics, slurs and articulations are evidence only when actually encoded; phrase
remarks do not supply syllable timing. Dynamics/CC/velocity alone do not supply
acoustic loudness. For audible orchestral balance use `bitwig-mixing`; for
researched interpretive cues use `performance-cues`; return live edits to
`bitwig-edit` only within the user's requested scope.
