---
name: bitwig-edit
description: Edit native Bitwig notes, channels, per-note expression and stored automation. Supports arranger edits and launcher Note Clip creation and editing. Excludes score import, instrument setup and cue research.
---

# Edit native musical content

An edit request authorizes its stated
musical change. Establish intended owners and preservation properties; keep work
bounded to that request. Use [bitwig-explore](../bitwig-explore/SKILL.md) to resolve
scope when needed. An open project is not a disposable history/random-edit fixture.

Use `midi-bridge controller` for every read, write, and verification of stored
automation and MIDI notes. Resolve the exact native owner and source through
controller reads. If the controller cannot establish the source or return a
valid complete snapshot, stop the affected edit and report the controller error.

Default to the arranger. When the request or current edit context identifies
the launcher, use the [launcher recipe](references/common-edits.md#launcher-clips).

For instrument automation, check the vendor manual for the exact instrument
and consult its `INSTRUMENT.md` when available. Verify the loaded preset's
mapping and prefer a native MIDI CC lane whenever it delivers the requested
effect. Otherwise use an existing, exposed instrument macro through the Bitwig
controller. Use computer interaction with a VST editor only when the user
explicitly requests it. [Automation procedure](references/automation-edits.md)

| Change | Read |
| --- | --- |
| Selected clip or named owners: read, inspect, isolate locally, edit notes/automation, replace and verify | [Common edit recipes](references/common-edits.md) |
| Structural paths; explicit beyond-MIDI parameter/Remote Control discovery | [bitwig-explore](../bitwig-explore/SKILL.md), then its relevant discovery reference |
| Note CRUD, transpose, channel, timing, velocity, bounded humanization | [Native edits](references/native-edits.md) |
| Per-note pressure/pitch/gain/pan/timbre points | [Native edits](references/native-edits.md#expression-and-automation-boundaries) |
| Stored automation point CRUD, complete curves, smoothing | [Automation edits](references/automation-edits.md) and its [curve-drawing procedure](references/automation-edits.md#draw-sparse-musical-curves) |

```text
resolve owners -> settle requested change (PERFORMANCE.md when in use)
 -> read complete selected clips OR resolve named paths and read complete clips
 -> notes / existing curve: inspect and edit that complete snapshot
 -> instrument automation: manual + live mapping -> native MIDI CC when supported
    -> otherwise discover and automate an existing instrument macro via controller
 -> replace once -> independent complete readback
 -> preservation check -> save when requested -> STOP
```

Keep exact units, owner IDs and document/revision from the appropriate observation.
For named tracks/clips, use explicit paths and complete replacement; selection is
not a prerequisite. Use sparse note commands only for an actual selected-note
request. Notes and automation share complete observations; reobserve and recompute
a stale plan. Omitted properties survive sparse updates;
do not replace notes from a lossy reporting projection. Skip already-correct
values, reporting no matches separately from already-correct material.

Read complete notes and automation together, even for a one-point change.
Structural reads and available-parameter discovery do not contain the stored
curves; the complete clip observation does. Filtering is local analysis: retain
the complete snapshot as the edit base and preserve every unrequested domain.
For a complete notes+automation replacement, read [complete clip replacement](references/clip-replacement.md).
For surrounding actions, use the
[shared session procedure](../bitwig-explore/references/session-actions.md).
Stored musical changes remain on the native editing route. Preserve loops, source ownership, unrequested notes,
expression, points and folder metadata. Refuse unsupported shared-source isolation.

On partial/unknown results, retain intent/receipts and reobserve before another
write. Never replay automatically or issue speculative Undo. Use a supported
qualified history action only when the adjacent history entry is established
and the request authorizes it. Native transaction/rollback mechanics
remain the adapter's responsibility and do not guarantee a safe user Undo target.

Finish with requested/eligible/already-correct/applied counts by owner, independent
readback and preservation status. If project content diverges from its generated
bundle, record the edit in project revision notes when that workflow is in use;
future reimport must reconcile it. Do not claim automatic score reconstruction.
Keep loaded patches. Verify MIDI delivery through the actual receiver route,
including Filter/Map where present. If routing fails, preserve the existing
curves and report the dependency.

This skill does not itself load patches, revise orchestration, create cue interpretations,
or promise general clip split/move/delete capabilities absent from the CLI.
