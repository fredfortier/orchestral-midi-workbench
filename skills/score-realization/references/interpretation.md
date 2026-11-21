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

Choose variants with the relevant INSTRUMENT.md. Keep attack weight, sustained
timbre/dynamics, expression, vibrato, tuning and release distinct. Patch changes
may change the dynamics control law. Select documented stock behavior that fits
the rendition. Setup only assigns and loads the exact patch; it does not change
receiver settings. A catalogue claim is not acoustic proof.

Review register/transposition, simultaneous lines, same-key overlap, tied notes,
articulation transitions, tails and final cutoff. Never shorten ties to accommodate
a monophonic patch. Preserve independent control histories; channel sharing must
agree over the complete overlap, not only at attacks. For continuo, figures and
voice-leading require actual harmonic interpretation; sounding pitch classes do
not constitute a continuo realization. Do not add unrequested notes or doubles.

Keep a concise passage review of musical role, receiver, attack/connection/release
and dynamics. Use supported `adapt::Plan.passages` assertions to independently
check track/instrument ownership and note coverage when constructing new plans.
When present these partition every performed note including repeats; source
prefixes use actual converter identities, not printed bar numbers. Expectations
come from score review, not copied conversion results. See
[reviewed passages](../../../convert/RENDITION.md#reviewed-passages-and-dynamic-curves).

Authored dynamic curves need explicit supported rendering rules. The converter
does not invent smoothing, breaths, bowing or phrase contours. Store unsupported
requirements and accepted losses in PERFORMANCE.md. A conversion pass establishes
encoding, not musical taste. Requested audition comparisons use exploration;
research of the global performance arc belongs to performance-cues.

For revisions, compare the actual project with the last rendition before replacing
anything. Preserve intentional native changes in pitch, timing, articulation,
dynamics, tempo and mix; reconcile their relation to score and executable inputs.
