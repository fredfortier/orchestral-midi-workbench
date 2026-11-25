# Bitwig composition boundary exercises

This temporary research contract governs disposable projects and worksheets in
this directory. The production workflow remains the authority for live project
work. Use the existing native service and start `osc listen` manually as a
standalone process. Service integration is outside this experiment.[^workflow]

The subject is document-centric MIDI. Compare inventory, exact targeting,
navigation and stored edits against the same expected result. Live recording
does not satisfy an exercise for deterministic clip placement or stored MIDI
editing.

## Fixture and exercises

Use a new Bitwig project with a short call-and-answer motif, two owners, an
empty launcher slot, an arranger clip, a cue or known seek point, and a native
CC lane. Add only the fixture needed for each comparison. Qualify the document
and the endpoint before writing. Keep existing project tabs out of mutation
scope. Leave the test project stopped and retain its native save separately
from the worksheets.

| Exercise | Musical question | Comparison and acceptance check |
| --- | --- | --- |
| 1. Find the answer | Which clip and lane own the second phrase? | Compare an exact arranger clip inventory with OSC feedback. Require track containment, clip identity and placement. Cross-check launcher slots and duplicate names as distinct cases. |
| 2. Jump and select | Can I reach the answer and select both parts for a quick action? | Compare playhead seek, exact arranger clip selection, lane time-range selection and panel reveal independently. Verify whether each operation can use a known native path or beat without repeating discovery. |
| 3. Sketch and revise | Can I place a motif and edit it as stored content? | Compare deterministic empty-clip placement and complete note/curve edits. Check placement, source/loop geometry, untouched notes and curves. Test arranger placement separately from launcher creation. |
| 4. Reuse a receiver | Does running the same setup converge on the same sound-producing path? | Run named Layer/slot ensure twice and compare identities/order and routing. Compare documented OSC insertion/browsing/layer controls with the native desired-state operation. Exact Kontakt loading requires an identified patch and remains a separate check. |
| 5. Work at the boundary | Which rehearsal controls could reduce keyboard steps? | Compare seek/play/stop and a documented save with qualified computer actions. Distinguish panel visibility from keyboard focus, and verify save via file and native readback. Inspect configured action slots before considering any action probe. |

## Rules

- Start each operation with controller and OSC capability checks. Compare
  the routes against the same target, result, and acceptable side effects.
  Use a matching controller or OSC route whenever one is available. Where
  neither route is established, record what was checked and what remains
  unresolved before considering Use Computer. An unresolved candidate keeps
  the boundary open. A fixture created through the UI establishes the starting
  state only.
- Follow [explore](../skills/bitwig-explore/SKILL.md), [edit](../skills/bitwig-edit/SKILL.md)
  and [assignment](../skills/kontakt-setup/references/assignment.md) for the
  native legs. Consult the [keyboard recipes](../skills/bitwig-explore/references/keyboard-workflow.md)
  for the UI leg. OSC mutations here are research probes on the disposable
  fixture only.[^workflow]
- Before a negative conclusion, cross-check the pinned DrivenByMoss reference
  in both directions, including banked and cursor variants, browser commands,
  MIDI input and configured actions. If Bitwig exposes an installed-version OSC
  reference, check it for discrepancies. Check actual controller CLI help and
  its owning contract before declaring a controller gap. A configured action
  must have a known binding and an independent effect check.[^osc][^controller]
- Use these result labels in the worksheet: **observed**, **documented only**,
  **not documented in the checked surface**, **refused** with its exact error,
  or **blocked** with the missing prerequisite. Silence, a failed probe, and an
  absent search hit do not establish impossibility. Attribute success to the
  command and fixture that produced it. An uncertain mutation requires fresh
  observation before another write.[^workflow]
- Verify stored document claims through qualified native readback. Record OSC
  arguments and relevant decoded messages for remote-state claims. Check the
  actual focused context for keyboard claims. Record preserved state and side
  effects. Audible behavior, file persistence and content are separate checks.
- Keep worksheets short. Each exercise records **Aim**, **Setup**, **Steps**,
  **Result**, and **Evidence**. Include host/protocol versions, project identity,
  exact input or command, relevant before/after, result label and scope. Keep
  raw logs and saves in ignored `tmp/`. Retain small fixture-only evidence
  extracts under `evidence/` when they are needed to recover a conclusion.
- Apply evidence-grounding to each claim. Give intended behavior its owning
  rule, external capabilities their primary reference, and observations their
  actual evidence. End the worksheet with a skill critique tied to observed
  steps. Identify the shortest reproducible route, redundant reads, missing
  handoffs and any remaining computer dependency. Keep suggested skill changes
  in the worksheet until the evidence supports a production change.

[^workflow]: The [production execution boundary and owners](../skills/bitwig-compose/references/workflow-stories.md#artifact-and-mutation-ownership)
    and [supported production operations](../skills/bitwig-compose/references/workflow-stories.md#route-by-outcome)
    govern current skill execution.
[^osc]: The [pinned DrivenByMoss OSC protocol](https://github.com/git-moss/DrivenByMoss-Documentation/blob/8aab247a5643082f9706b36c6fd091ab7125aae4/Generic-Tools-Protocols/Open-Sound-Control-(OSC).md)
    defines documented feedback, commands and the configured action slots.
[^controller]: The [controller contract](../docs/architecture/controller.md)
    defines native ownership and verification.
