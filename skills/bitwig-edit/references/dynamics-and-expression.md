# Plan dynamics and expression as separate musical decisions

Read when realizing or revising dynamics/expression. The exact patch's
INSTRUMENT.md controls the mapping; these sources explain the distinction, not
every library's defaults. This reference changes planning, not edit authorization
or the native read/replace contract.

## Evidence and scope

- Orchestral Tools, [Common MIDI CCs](https://orchestraltools.helpscoutdocs.com/article/85-common-midi-ccs),
  CC1/CC11/CC7 entries, updated 5 October 2021, consulted 19 November 2026:
  CAPSULE CC1 morphs dynamic layers when X-Fade is active; CC11 controls patch
  level; CC7 controls Kontakt volume for template balance. This distinction does
  not establish the mode of an unidentified loaded patch.
- Strezov Sampling, [AFFLATUS Chapter I Strings, update 1.2](https://www.strezov-sampling.com/learn/articles/stories/afflatus-v1-2),
  22 May 2019, Advanced Options / Dynamics / Expression: the engine exposes
  separate dynamics and expression assignments; learning another controller
  does not remove the original assignment. This is AFFLATUS evidence, not proof
  of ARVA, FREYJA, WOTAN or another engine's mappings. Do not change MIDI Learn
  or patch settings to manufacture the desired control law.
- Master The Score, [Realistic Orchestral Mockups](https://www.masterthescore.com/article/orchestral-mockups),
  “The Controllers: CC1, CC11, Velocity, and CC64”, consulted 19 November 2026:
  explains layer/timbre versus independent level, and warns that duplicating the
  same curve can exaggerate dynamics. Its separate-pass/selective-CC11 approach
  is production advice, not a requirement to automate every passage or instrument.

## Apply the distinction

1. Resolve the exact articulation and supported controls. Identify dynamic-layer
   crossfade, expression gain, velocity roles and any remapped/linked controls.
   A volume change alone does not prove an independent timbral control. Missing
   evidence returns to documentation-only intake; preserve that branch unless
   the user authorizes a labelled provisional interpretation.
2. Read the part's actual sounding passages, rests, sustained notes and shared
   receiver overlaps. In PERFORMANCE.md state who leads, where tension builds,
   where accompaniment yields, and how the phrase releases. Clip edges and
   bars are location aids, not automatic phrase boundaries.
3. Choose the sampled intensity with the documented dynamics control (often
   CC1 for longs, velocity for shorts). Retain attack/transition functions of
   velocity. Attenuated loud samples retain their loud timbre.
4. Make a separate CC11 pass where independently supported. Give it its own
   musical anchors and level range. For broad phrase shaping, use fewer, more
   widely spaced anchors, held plateaus and longer approaches/releases than
   local CC1 details. An entry, crest or release may coincide for musical
   reasons; every bend should not coincide by construction. Do not copy, scale,
   shift or low-pass CC1 and call the result an independent expression decision.
5. Consider the combined gain. Both controls can affect loudness, so full-range
   movement of both can overstate a crescendo or hollow out accompaniment.
   Leave useful upward room in CC11 when the phrase needs it. There is no
   universal starting value, floor, range, or mapping from CC units to dB.
   A flat chosen level is valid for a deliberately level span; maximum is a
   musical endpoint or an explicit exception, not a workflow default.
6. Prepare entry values without a jump on a sounding attack; recover in rests
   where possible. Do not swell an absent part toward another part's climax.
   Preserve tails, authored silences, pedals/resets and effective state beyond
   the requested interval. Do not manufacture a fade at a trimmed continuation.

Velocity-driven shorts may use a supported CC11 envelope for broad passage
balance, while velocity continues to select attack/dynamic layers. A verified
CC11-only patch needs no speculative CC1. Unsupported percussion or an
unidentified receiver must not gain CC automation merely for uniform coverage.

## Singing patches

Compose the documented dynamics control as changing vocal intensity. Initialize
CC1 before the first sung attack where it controls dynamics, then shape each
independent line toward stressed words, harmonic tension and phrase climaxes,
easing through resolutions. Carry direction across sustained vowels and melismas;
do not pump every note or syllable. Supporting lines yield as another emerges.
Reserve the strongest intensity for the intended climax. A held value requires a
musically level span, not a generic maximum or an undocumented patch default.

Where independently supported, CC11 supplies separately planned expression and
balance; it does not discharge the sampled-dynamics requirement. Missing numeric
layer thresholds do not make a documented controller function unknown: use
labelled editorial anchors and refine by listening. Keep genuinely unknown exact
patch assignments separate from that calibration gap. Reconcile dynamics when
changing between builder and fixed-vowel receivers; retaining the old curves is
not proof that the replacement receives its required controls. Do not declare a
singing realization complete while required dynamics are absent.

Strezov's [Storm Choir Ultimate manual](https://www.strezov-sampling.com/data/uploads/products/62/6601e7995bfc9ae6d5a854081f5735af.pdf),
“About Storm Choir Ultimate”, PDF p.3, explicitly assigns the three recorded
syllable dynamic layers to modwheel CC1. That establishes the builder function,
not numeric mf/f/ff anchors or every separate legato patch's law. Exact library
capabilities remain in INSTRUMENT.md; the phrasing guidance above is musical
interpretation, not a vendor-prescribed curve.

## Verify the result

Independently read native lanes and verify their identity, channel, initial state,
points and intended passage alignment. Compare CC1 and CC11 against the stated
musical roles, including a span where they deliberately differ; fewer points or
low numerical correlation alone do not establish good phrasing. Preserve notes,
routing and unrequested curves. When audio/audition is requested, check combined
level/timbre and starts within phrases; otherwise report stored-data verification
separately from audible response. Do not compensate for a failed receiver route
with copied controls or sender devices.
