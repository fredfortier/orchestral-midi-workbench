# Vocal realization

Load only for sung text or vocal receivers. Read relevant initialized library
documentation; exact inventories and manual facts belong there, not in this skill.

Establish the requested fidelity: literal intelligible text, phonetic approximation,
vowel-only or wordless singing. Preserve the original lyrics in all cases. An Ah
patch does not pronounce Latin; humming changes text. Record losses explicitly in
PERFORMANCE.md, retaining unsupported required diction as pending.

Recover each sung unit from actual part/staff/voice/lyric line and note/occurrence
identities. Preserve `syllabic`, `extend`, `elision`, ties and relevant slurs. A
MusicXML word-position label is not pronunciation. Distinguish another pitch on
the same syllable from a new syllable; ties/melismas preserve continuity. Missing
text does not authorize copying another voice or advancing a shared word counter.
Review ambiguous grouping against the edition. Pronunciation tradition and vowel
mapping require an explicit source-backed decision, not English token spelling.

Keep independent vocal entries, text, register, stress, breath and dynamics. Tempo
alone cannot select syllabic versus legato behavior. Velocity may trigger a special
sample rather than accent loudness. Do not switch receivers within a melisma by
default: another receiver does not inherit the preceding legato/syllable state.

For each receiver, follow [singing dynamics](../../bitwig-edit/references/dynamics-and-expression.md#singing-patches).
Specify initialized vocal-intensity contours separately from supported expression
gain. Reconcile the control law when switching builder/fixed-vowel patches;
syllable correctness or preserved old curves alone does not complete the dynamics.

For supported stateful instruments, required selector/HLS/reset behavior must be
documented and encoded with exact ordering, numeric control keys and units.
Arbitrary seek, loops and release state require their own evidence. Do not invent
selector lead times or assume pedal CC meanings are shared across libraries.

For requested Syllabuilder performance, route to
[kontakt-syllable-builder](../../kontakt-syllable-builder/SKILL.md) with the exact
voices, underlay and fidelity policy. It returns configured phrase definitions,
occurrences and verified controls; resume realization only through supported
MIDI/control operations. Its planning JSON is project evidence, not an executable
companion extension. Do not invent JSON fields or unverified low-note selectors.
Stock assignment remains kontakt-setup's responsibility. Read
the local Strezov research index (`.research/strezov/SOURCES.md`) when available only for the specific
capability question. Piece-specific instrument choices and phonetic candidate
tables belong in project evidence, not reusable default policy.
