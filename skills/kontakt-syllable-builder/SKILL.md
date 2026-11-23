---
name: kontakt-syllable-builder
description: Configure reusable Strezov-style Kontakt Syllabuilder phrases from vocal lyric underlay, minimizing phrase receivers and documenting phonetic losses. Begins with the intended patch open; uses computer interaction for builder editing. Also supplies the syllable-documentation procedure for instrument intake.
---

# Configure vocal phrases

For library documentation only, follow [document syllables](references/document-syllables.md)
and return to intake. For a wider performance request, use the [sung-phrase handoff](../bitwig-compose/references/score-and-performance.md).
Begin with the intended Kontakt editor open on an initialized full-builder patch,
with exactly that one instrument in its Instrument Navigator;
loading and native routing belong to [kontakt-setup](../kontakt-setup/SKILL.md).
That owner uses the existing integrated assignment with `--keep-editor-open`.

## Direct execution

Use one settled plan in PERFORMANCE.md, including its linked dictionaries and
occurrence tables. On a continuation, read that plan and retained completion
receipts, observe the current affected state, and resume the first unfinished
binding. Replan only the decisions invalidated by new evidence or user direction;
do not rerun extraction, optimization, intake or correct programming merely to
repeat their evidence.

```text
resolve active project + existing plan
  → settle only missing phrase/control decisions; record them once in PERFORMANCE.md
  → serially reuse or load → program → reread → close each unfinished receiver
  → one fresh complete-clip edit after all affected receivers are ready
  → independent readback → Save/reopen and representative audition → report
```

Keep the control flow tight. Reuse unchanged patch qualification and native
keyboard qualification within their established host/patch scope. Batch independent
reads and offline comparisons; use the directly relevant reference at its action.
Do not add approval stages, duplicate ledgers, whole-project reads per receiver,
or repeated validation of unchanged material. Keep checks that establish target
identity, musical preservation, uncertain outcomes and the requested result.
A failure blocks its dependent action; reconcile that outcome without restarting
completed work or expanding the task into a diagnostic campaign.

1. Follow [orientation and lyrics](references/orientation-lyrics.md): establish the
   saved project, requested voices, original underlay and performed occurrences.
   Default to its `performance.companion.json`; native notes are not lyric authority.
2. Read the exact NKI's INSTRUMENT.md entry. Missing vocabulary/control evidence
   invokes intake's documentation-only route, then returns here. Match the open
   patch to that entry. A fixed-vowel or automatic Performance patch does not
   inherit full-builder eligibility.
3. Follow [phrase planning](references/phrase-planning.md): default to the closest
   evidenced sampled syllables with documented phonetic losses, considering
   fixed-vowel legato for suitable complete melismas; honor explicit user policy
   without asking again. Settle pronunciation,
   gestures, safe cuts and fixed-vowel passages; optimize the reusable phrase
   dictionary. Record definitions, occurrences, losses and receiver bindings in
   PERFORMANCE.md before programming. Each phrase owns a named receiver slot and its note-channel binding, using
   the shared standard Filter/Map topology and channel-1 track controls. Verify
   actual delivery before relying on HLS.
   Review prefix reuse and irreducibility before programming or accepting the
   bank. Name slots from source lyric syllables, never mapped sample labels;
   retain sampled equivalents, text aliases and opaque IDs in evidence.
4. Follow [builder programming](references/builder-programming.md). Reuse correct
   content. Capture the baseline, edit through computer use and reread the result.
   Do not write stock NKIs or manipulate preset binaries.
5. Before rerouting notes, require every affected receiver to be loaded, configured
   and verified. Return exact phrase/channel bindings and verified reset/HLS/control schedules
   to realization or [bitwig-edit](../bitwig-edit/SKILL.md). Preserve musical note
   attributes except the specified receiver assignment. Use complete-clip reads
   and replacement; this skill does not supply another native editing surface.
   Reconcile every replacement receiver's documented dynamics with the project's
   existing controls. Follow [singing dynamics](../bitwig-edit/references/dynamics-and-expression.md#singing-patches);
   return missing required curves to the edit owner before completion. Preserving
   existing automation alone does not establish the new patch's vocal intensity.
6. Compare every occurrence against the plan, including phrase entries, melismas,
   repeated attacks and rests. Verify saved configuration and an audible passage
   before claiming persistence or synchronized singing. Report completed scope
   and remaining losses; a programmed dashboard alone proves neither.

An ambiguous source association, unsupported required phoneme or unverified
control blocks only its dependent action. Resolve from retained sources and the
user's established approximation policy; ask only for a consequential missing
decision. After uncertain UI input or partial writes, reobserve and reconcile
the original intent before continuing. Never reset a cursor by assumption.
