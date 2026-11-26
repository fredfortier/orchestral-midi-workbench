# Interpret before encoding

Read the complete musical texture, not only the opening or a note-count summary.
Use the score rendering/edition when notation is ambiguous. Preserve provenance
for part, voice, source note and performed repeat occurrence.

```text
SOURCE: authored notation / text / pitch / articulation / dynamics
INTENT: chosen attack / connection / release / contour, with musical reason
RECIPE: documented patch + required controls that can realize that intent
```

Unmarked notation does not mean sustain sampling; short values do not prove
staccato/spiccato. Do not impose a minimum patch count or apply one gate percentage
to all parts. Identify connected line, separate attacks, repeated pulse, held
harmony, entrances and cadences. Slurs, ties, sampled legato and portamento are
different. Phrases cross barlines; do not reset legato, lyrics or dynamics there.

## Research and map articulations

Complete this review before recommending an articulation patch or writing its
routing or note changes. It applies to new and revised articulation decisions
in proposals and realizations. Use it for the requested passage, part, clip or
movement.

1. **Resolve the occurrences.** Identify the edition or score source, part and
   voice, and complete requested range. For native material, identify the actual
   track and clip owner and reconcile written locations with performed
   occurrences. Distinguish an Arranger clip from a Launcher clip, a selected
   subset from a whole clip, and shared-staff lines from doublings or export
   duplicates. Read slurs, ties, rests, ornament and articulation marks, and
   repeated attacks in the complete gesture, including neighboring notes.
   Check directions that continue beyond their first example.
   Where a parsed marking or voice assignment is uncertain, inspect the original
   notation or a suitable edition and identify the source used. When the score
   is unavailable, report what the native notes establish and leave score
   markings unverified.
2. **Research the consequential choices.** Use the [musicology research
   procedure](../../performance-cues/references/musicology-research.md) to seek
   work- and passage-specific analysis or critical commentary. If it leaves the
   technique open, check applicable period, instrument or vocal practice.
   Cite the source and exact location and state the inference it supports. When
   an available recording is specified as a reference, align its performed
   form to the score and cite timestamps for the relevant contrasts. Describe
   the audible attack, connection and release. Attribute a physical technique
   only if the evidence can identify it. Record gaps or conflicting evidence
   and make a reasoned provisional choice when the sources leave room.
3. **Choose the sound, then the receiver.** Describe the intended attack,
   connection, body and release in playing terms. Use [instrument
   selection](instrument-selection.md) to compare eligible installed variants
   against that intent. Read their exact entries in `INSTRUMENT.md` and the
   applicable manual sections when the choice depends on a specific behavior.
   Check register, retriggering, transitions, release, dynamics control and
   other behavior relevant to the passage. Cite what distinguishes the chosen
   variant from plausible alternatives. Qualify a filename-only candidate as
   unverified. If acoustic checking is in scope, use the [matched audition
   procedure](../../bitwig-compose/references/audition.md) at the passage's
   register and pulse, including the relevant transition or release.
4. **Return the articulation map.** For each gesture, give the score location
   and performed occurrence, observed notation, cited evidence or its limit,
   intended sound, exact relative NKI, displayed Bitwig note channel and
   Kontakt input, and verification status. Label proposed routing, measured
   behavior and editorial gate or control targets explicitly. Group equivalent
   occurrences where they share the rationale and binding. For a complete
   rendition or full-clip review, account for every performed note and
   intentional omission. Return the map for a proposal or record it in the
   realization evidence linked from `PERFORMANCE.md`. Use the owning edit
   and setup workflows when implementation is requested.

Keep attack weight, sustained timbre, dynamics, expression, vibrato, tuning
and release distinct. Patch changes may change the dynamics control law.
Select documented stock behavior that fits the rendition. Setup assigns and
loads the exact patch with its stock settings. Establish acoustic behavior
through audition or measurement.

## Check the performed mapping

Review register/transposition, simultaneous lines, same-key overlap, tied notes,
articulation transitions, tails and final cutoff. Never shorten ties to accommodate
a monophonic patch. Preserve independent control histories; channel sharing must
agree over the complete overlap, not only at attacks. For continuo, figures and
voice-leading require actual harmonic interpretation; sounding pitch classes do
not constitute a continuo realization. Do not add unrequested notes or doubles.

When compiling, use the supported `passages` assertions to check the map's
track, instrument and note coverage independently of the routing rules.
Source prefixes use actual converter identities. Inspect adjacent notes
and every receiver switch on both sides of its boundary. Verify the exact
channel and slot even where multiple slots share a patch. See
[rendition inputs](conversion.md#rendition-inputs).

Authored dynamic curves need explicit supported rendering rules. The converter
does not invent smoothing, breaths, bowing or phrase contours. Store unsupported
requirements and accepted losses in PERFORMANCE.md. A conversion pass establishes
encoding, not musical taste. Requested audition comparisons use exploration;
research of the global performance arc belongs to performance-cues.

For revisions, compare the actual project with the last rendition before replacing
anything. Preserve intentional native changes in pitch, timing, articulation,
dynamics, tempo and mix; reconcile their relation to score and executable inputs.

## Translate an expressive direction into receiver behavior

Use the cue's score event, occurrence, named lines and audible criterion where a
researched direction exists. For a score before native placement, take the
equivalent anchors from the score and intended performed form. Return ambiguous
musical meaning to its owner. Resolve feasibility through the exact patch and
articulation, informed by the documented control law and any applicable patch
tests.

For each affected binding, specify the attack or within-note change, the parts
that need independent shaping, and the initial state, anchors and release that
will make the gesture audible. Name the supported control, its units and the
applicable note or control-channel route. Follow the shared
[dynamics and expression procedure](../../bitwig-edit/references/dynamics-and-expression.md#apply-the-distinction)
for velocity, sampled intensity and independent expression. Receiver capability
governs the choice of control. Mark numeric targets as editorial unless they
describe documented bounds or measured response. Group equivalent gestures when
they share a purpose, binding and intended behavior. Keep different occurrences
distinct when the return calls for a different treatment.

Record this translation in the existing binding and curve-anchor sections of
`PERFORMANCE.md` for a rendition. A proposal can describe the same choices
without encoding them. When applying to the generated rendition, verify the
decoded and native values, alignment and routing against the intended gesture.
For an authorized native-only revision, hand the scoped plan to
[Bitwig Edit](../../bitwig-edit/SKILL.md) and use its readback checks. If the
scope includes acoustic verification, carry the cue's criterion into a
representative ensemble comparison. A matched stem can establish a receiver's
response, while hearing the intended hierarchy requires the relevant parts
together. Keep unsupported effects pending and report the acoustic evidence
actually obtained.
