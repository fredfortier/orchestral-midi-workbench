# Assess the music for performance

Use this reference when deriving or revising the interpretation. Inputs are the
active project's identity, complete relevant native note capture, existing cue
map and, when available, its score and edition. Output is a small set of audible
landmarks and actions to test against literature, then put into PERFORMANCE.md.

## Read and compare parts

Follow [the exploration procedure](../../bitwig-explore/references/arranger-navigation.md) from the project workspace:

```sh
run_dir=$(mktemp -d /tmp/performance-cues.XXXXXX)
midi-bridge controller midi get-all --output "$run_dir/notes.json" > /dev/null
```

Confirm movement, tracks, clips and musical extent from the result. Retain exact
data on disk. Reduce it with bounded jq queries by owner and passage, preserving
total match counts; do not dump the entire piece into context. Read source/loop
mapping before comparing parts on the arranger. Notes use source beats; cue
positions use arranger quarter-note beats. A pickup, changed meter, offset clip
or repeated source invalidates naive `(bar - 1) * 4` placement.

A complete clip read includes stored source notes outside its current audible
placement. Establish the intended excerpt from the actual clip start/duration,
source boundaries and loop mapping before choosing the performance arc. Preserve
the complete capture, but base this excerpt's analysis on events intersecting its
mapped playback interval. Do not extend clips, include a hidden later section or
import its climax/closure into the visible excerpt. A trimmed ending may interrupt
an unresolved phrase; describe that boundary honestly. Keep existing cue keys for
excluded sections reserved and unplaced rather than assigning them new functions.

Trace, across all relevant parts:

1. **Form and phrase structure:** introductions, thematic or textural returns,
   imitative entries, antecedent/consequent relations where applicable,
   phrase extensions, cadences and elisions. Do not force a periodic or sonata
   model onto music whose syntax does not support it.
2. **Harmonic direction:** bass motion, tonal centres, sequential patterns,
   chromatic intensification, pedal points, cadential preparation and resolution.
   Infer harmonic function from complete sounding verticalities and voice-leading,
   including tied/sustained notes, not just onset histograms. Use a score to
   establish spelling; qualify Roman numerals and inversion labels if uncertain.
3. **Counterpoint:** subject/motivic entries, imitation, suspensions and their
   preparation–dissonance–resolution, voice exchanges, contrary/parallel motion,
   rhythmic independence and changing foreground. Name a 9–8 or 4–3 suspension
   only after checking bass, preparation, metrical dissonance and resolution.
4. **Register, rhythm and texture:** local tessitura and registral apices,
   rhythmic diminution/augmentation, harmonic rhythm, sustained lines against
   moving accompaniment, density, orchestration and text prosody when known.
   Distinguish a registral apex from a harmonic or rhetorical culmination.
5. **Performance trajectory:** identify which processes intensify together,
   which arrival is structurally decisive, and which peaks remain subordinate.
   Describe preparation → arrival → release and the continuity across each.

Treat stored velocity, expression and automation as the current rendition, not
an authority for the work's intended character. Stored note lengths cannot prove
slurs, breath marks, articulation or legato in the score. A mute/chance/repeat
operator or instrument response may change what is heard; label the evidence.

## Translate analysis into decisions

For every proposed cue complete this thought: “Because **musical process**, the
performer should **action in named part(s)** toward **arrival/release**.”
Prefer `sustain the alto suspension; let the resolution release the intensity`
to `emotional peak`. Use *cantabile*, *tenuto*, *crescendo*, *diminuendo*,
*agogic broadening*, *dominant prolongation*, *cadential punctuation* and similar
terms only with their normal musical meanings. *Diminution* means shorter-value
figuration/ornamentation; *diminuendo* means decreasing loudness.

Mark phrase destinations, entry preparation, contrapuntal balance and releases,
not every analytical event. Retain room for the principal climax: a subordinate
registral ascent can have a local crescendo without exhausting dynamic scope.
Do not prescribe rubato at every landmark or confuse intensity with tempo/loudness.
Make actions playable: state the pulse to maintain, voice to bring forward,
continuo/accompaniment to restrain and point at which breath/intensity renews.

Piece-specific hypotheses belong in the project's interpretive decisions, with
their source and verification status, rather than as reusable skill defaults.
