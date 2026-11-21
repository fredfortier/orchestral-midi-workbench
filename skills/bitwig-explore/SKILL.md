---
name: bitwig-explore
description: Locate and select Bitwig tracks, clips, automation lanes, devices and slots; inspect musical content, instruments and cues; compose native selection with supporting computer recipes. Musical edits and instrument setup retain their owning skills.
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
| Save, rename, navigate panels or perform authorized cleanup | [Supporting project actions](references/keyboard-workflow.md) |

Resolve names within actual native containment; retain returned `nativeId`
discriminators for duplicate siblings. A displayed lane index locates a row in
its observed track and is not durable identity. Read project layout, narrow to
the relevant owners, then read complete musical content and analyze locally.
Structural and selection reads contain no notes, points or plug-in parameters.
Parameter discovery is explicit through `controller automation read`.

For an explicit-path operation, selection is unnecessary. For surrounding UI
work, read context → **CONTROLLER** select exact paths → **COMPUTER** establish
the recipe's panel focus and invoke its named action → **CONTROLLER** reread.
Track headers, clip objects and automation time ranges are distinct native states.
Never infer keyboard focus from native selection.

For musical edits, pass resolved paths and complete content to
[bitwig-edit](../bitwig-edit/SKILL.md): read → construct change → replace →
independently reread and compare. For routing or instrument assignment, use
[kontakt-setup](../kontakt-setup/SKILL.md)'s existing integrated operation, verify
routing, then return to the musical task. Preserve Channel Filter → Channel Map
setup. Exploration adds no macro executor or alternate setup/editing mechanism.

Keep complete observations as edit authority; local query projections are reports.
A read failure requires diagnosis, not fabricated empty content. Reobserve after
navigation/history before writing. Health is `/v2/health` on the configured port.
