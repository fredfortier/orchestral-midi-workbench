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

Resolve articulation over the complete gesture before writing routing rules.
Read words and signs such as *simile* or *segue*, opening examples whose marks
continue, changes of bowing, and shared-staff instrument labels. Use the
engraved score or another appropriate source to settle uncertain starts and
ends. Compare neighbors with the same rhythm and musical role at every
receiver switch, on both sides of the boundary. Default legato can give a
fresh attack with appropriate note separation. A switch to a short sample
requires its own musical justification. Trace shared staves back to the
named instruments and distinguish intended doublings from duplicated exports.

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

For a new realization or a full channel review, partition every performed
note, including repeated occurrences, into reviewed gestures. For each
gesture record the score location, intended attack/connection/release, exact
receiver and channel, and musical reason. Group notes that share a decision.
Account explicitly for redundant source copies and other intentional omissions.
Use the supported `passages` assertions to check the reviewed track, instrument
and note coverage independently of the routing rules. Source prefixes use
actual converter identities. Inspect adjacent notes and all route boundaries
against that review, and verify exact channel/slot separately where multiple
slots share a patch. See [rendition inputs](conversion.md#rendition-inputs).

Authored dynamic curves need explicit supported rendering rules. The converter
does not invent smoothing, breaths, bowing or phrase contours. Store unsupported
requirements and accepted losses in PERFORMANCE.md. A conversion pass establishes
encoding, not musical taste. Requested audition comparisons use exploration;
research of the global performance arc belongs to performance-cues.

For revisions, compare the actual project with the last rendition before replacing
anything. Preserve intentional native changes in pitch, timing, articulation,
dynamics, tempo and mix; reconcile their relation to score and executable inputs.
