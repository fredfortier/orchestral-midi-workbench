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
The outcome is the active project's `PERFORMANCE.md` and actual Bitwig **Arranger
Cue Markers**, linked by persistent cue keys. A document or marker JSON alone is
not completed marker creation.

## Entry and intent routing

**First establish an already open project** with `controller info` and a fresh
complete clip read through bitwig-explore. Match actual project/path identity and
require relevant nonempty MIDI content. Missing host/service/project is a missing
prerequisite, not permission to launch or navigate elsewhere. With multiple
adapters identify the intended instance explicitly. Do not substitute cached music.

| Intent, after the entry condition passes | Required reading and action |
| --- | --- |
| Assess a piece / create its performance cues | Read [musical assessment](references/musical-assessment.md), [musicology research](references/musicology-research.md), [performance file](references/performance-file.md), then [cue markers](references/cue-markers.md). Observe → assess → cross-reference → shape → place → verify. |
| Revise an existing arc after musical changes | Read the existing PERFORMANCE.md and all four references above. Reobserve music and markers; retain cue keys whose musical functions survive; revise positions and interpretation from current evidence. |
| Research or refine a disputed interpretation | Read assessment, research and performance-file references. Investigate the specific performance decision; update affected cue explanations. Do not mutate markers unless placement/naming changes are requested or needed by the requested cue revision. |
| Place an already agreed cue sheet | Read performance-file and cue-marker references. Verify its musical anchors against a fresh note observation; preserve the agreed interpretation unless evidence contradicts it. |

Open [bitwig-explore](../bitwig-explore/SKILL.md) before musical reads; use its
`controller midi get-all --paths paths.json --output fresh.json` command for
explicit scope, or omit paths for the actual native selection. Use `controller info`
to establish every intended part/clip. Do not confuse an empty or partial selection
with the whole piece. Resolve written passages to actual arranger occurrences
using the [musical read procedure](../bitwig-explore/references/musical-queries.md).
Use current `controller cues --help` for cue syntax.
Resolve the active project's workspace from native project identity; the output
belongs beside its `.bwproject`, not in this skill or the source repository. For
an unsaved project without a durable workspace, ask for the destination while
continuing observation and analysis; do not invent a permanent directory.

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
native state before retrying. Finish only with a concise arc, a useful cue-key
map, relevant citations and separately reported marker readback status.
