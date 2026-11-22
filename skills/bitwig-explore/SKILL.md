---
name: bitwig-explore
description: Locate and select Bitwig tracks, clips, automation lanes, devices and slots; inspect musical content, instruments and cues; seek, reveal and audition resolved passages through qualified session actions. Musical edits and instrument setup retain their owning skills.
---

# Explore and navigate Bitwig

Follow [the shared workflow](../WORKFLOW.md). Choose a reference by the user's
intended work; **CONTROLLER** and **COMPUTER** mark execution steps within that
workflow, not separate categories of tasks.

| Intended work | Workflow |
| --- | --- |
| Locate/select an item, narrow a group or inspect current selection | [Native navigation](references/arranger-navigation.md) |
| Understand a passage or analyze tracks | [Musical queries](references/musical-queries.md) |
| Understand automation, its owners and coordinates | [Automation](references/automation-queries.md) |
| Inspect an instrument, exposed controls or the exact plug-in UI | [Instrument inspection](references/device-parameters.md) |
| Inspect performance landmarks | [Cue markers](references/cue-markers.md) |
| Seek, audition, Save, history, arranger visibility or configured actions | [Session actions](references/session-actions.md) |
| Project dialogs, focus or authorized legacy cleanup | [Computer recipes](references/keyboard-workflow.md) |

Resolve names within actual native containment; retain returned `nativeId`
discriminators for duplicate siblings. A displayed lane index locates a row in
its observed track and is not durable identity. Read project layout, narrow to
the relevant owners, then read complete musical content and analyze locally.
Structural and selection reads contain no notes, points or plug-in parameters.
Parameter discovery is explicit through `controller automation read`.

Find the exact object and its arranger geometry once. Seek with the returned
beat, select with the returned native path, and show or focus the relevant
panel only when the request or next action needs it. Explicit-path reads need
no selection. Track headers, clip objects and automation time ranges are
distinct native states. Check editor and membership separately.

For musical edits, pass resolved paths and complete content to
[bitwig-edit](../bitwig-edit/SKILL.md): read → construct change → replace →
independently reread and compare. For routing or instrument assignment, use
[kontakt-setup](../kontakt-setup/SKILL.md)'s existing integrated operation, verify
routing, then return to the musical task. Preserve Channel Filter → Channel Map
setup. Exploration adds no macro executor or alternate setup/editing mechanism.

Keep complete observations as edit authority; local query projections are reports.
A read failure requires diagnosis, not fabricated empty content. Reobserve after
navigation/history before writing. Service health is `/v2/health` on the HTTP API
port (8789 by default). Confirm project readiness with a native read.
