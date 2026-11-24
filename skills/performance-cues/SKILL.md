---
name: performance-cues
description: Assess an active Bitwig piece from its stored notes and piece-specific musicological literature, shape a practical performance arc, and create Arranger Cue Markers mapped by stable keys to PERFORMANCE.md. Use for interpretive cue sheets, rehearsal landmarks, climax hierarchy, phrasing and performance-arc revisions; requires Bitwig Studio open with relevant MIDI loaded. Excludes general literature surveys and score import.
---

# Performance cues

Own PERFORMANCE.md's interpretation,
cue map and cue evidence; preserve its realization/bindings and configuration
sections. Cue directions do not automatically authorize note, automation or tempo
changes. Use the shared document template, not a competing whole-file format.

Turn musical evidence into decisions a performer can use at rehearsal and playback.
A proposal returns recommendations and listening checks. A requested cue-sheet
revision updates the active project's `PERFORMANCE.md`. Requested marker placement
requires actual Bitwig **Arranger Cue Markers**, linked by persistent cue keys and
verified by readback.

## Entry and intent routing

**First establish an already open project** with `controller info` and a fresh
complete clip read through bitwig-explore. Match actual project/path identity and
require relevant nonempty MIDI content. Missing host/service/project is a missing
prerequisite, not permission to launch or navigate elsewhere. With multiple
adapters identify the intended instance explicitly. Do not substitute cached music.

| Intent, after the entry condition passes | Required reading and action |
| --- | --- |
| Create performance cues and place markers | Read [musical assessment](references/musical-assessment.md), [musicology research](references/musicology-research.md), [performance file](references/performance-file.md), then [cue markers](references/cue-markers.md). Observe → assess → cross-reference → shape → place → verify. |
| Propose or review a researched performance or expression arc | Read [musical assessment](references/musical-assessment.md), [musicology research](references/musicology-research.md), and the [performance file](references/performance-file.md). Return the requested proposal or review. Open [cue markers](references/cue-markers.md) for requested placement or placement changes. |
| Revise an existing arc after musical changes | Read the existing PERFORMANCE.md and the assessment, research and performance-file references. Reobserve music and markers. Retain cue keys whose musical functions survive and revisit their positions and interpretation. Use [cue markers](references/cue-markers.md) when applying placement changes in scope. |
| Research or refine a disputed interpretation | Read assessment, research and performance-file references. Investigate the specific performance decision. Return a review or revise the affected cue explanations as requested. Change markers only when placement is in scope. |
| Place an already agreed cue sheet | Read performance-file and cue-marker references. Verify its musical anchors against a fresh note observation; preserve the agreed interpretation unless evidence contradicts it. |

Open [bitwig-explore](../bitwig-explore/SKILL.md) before musical reads; use its
`controller midi get-all --paths paths.json --output fresh.json` command for
explicit scope, or omit paths for the actual native selection. Use `controller info`
to establish every intended part/clip. Do not confuse an empty or partial selection
with the whole piece. Resolve written passages to actual arranger occurrences
using the [musical read procedure](../bitwig-explore/references/musical-queries.md).
Use current `controller cues --help` when operating on markers.
For a project-file deliverable, resolve the workspace from native project
identity and use the directory beside its `.bwproject`. For an unsaved project
without a durable workspace, resolve the destination while continuing observation
and analysis. A proposal in conversation needs no output path.

## Shared musical obligations

- Assess the whole movement's direction before placing local cues. Establish a
  hierarchy of principal climax, subordinate climaxes, preparation, relaxation
  and cadential closure; a high note, loud passage or dense texture alone does
  not establish the principal climax.
- Give each cue a conventional musical function and a concrete performance
  action: whose line leads, where the phrase goes, what dissonance is sustained,
  what resolves, what pulse/articulation continues, and where intensity releases.
  Use technical terms accurately, in their stylistic and historical context.
- Cross-reference actual notes, score/edition when available, and literature.
  Separate authored notation, source claims, native observations and editorial
  performance choices. Do not turn MIDI key numbers into enharmonic spelling,
  dynamic commands into composer markings, or an attractive narrative into fact.
- For material changes of contrast, balance or timbre, state the musical event and
  performed occurrence, the role of each affected line, the proposed musical
  action and what a listener should hear. Use composition and orchestration
  literature to test that reasoning where applicable. Carry the chosen criterion
  into realization or audition when the request includes either stage.
- Research only questions that change phrasing, balance, articulation, tempo,
  agogics or climax hierarchy. Stop collecting when the decisions have sufficient
  support and remaining uncertainty is explicit.
- Cue keys are the primary anchors. Bar references are optional, sparse aids;
  they never define identity. Preserve existing interpretation and markers outside
  the requested scope. This skill does not authorize note, automation, tempo,
  mixer, instrument or project-save changes.

If score, note evidence and scholarship disagree, revisit the affected passage
and edition/coordinate mapping. Retain the uncertainty and a usable provisional
reading if it cannot be resolved; do not manufacture agreement. If marker
execution fails, retain the draft with explicit unplaced status and reconcile
native state before retrying. Finish with the requested arc or cue map, relevant
citations, and the evidence status for any requested marker work.
