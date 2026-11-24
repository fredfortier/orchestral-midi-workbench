---
name: score-intake
description: Intake a supplied score into an idempotent Bitwig project workspace, preserving its original bytes and establishing project instructions. Mechanical setup only; no musical realization or patch selection.
---

# Score intake

Produce a reusable saved project
workspace and stop. Intake accepts a score as an opaque source, including MXL,
MusicXML or PDF; acceptance here does not imply converter support for that format.

1. Read destination instructions. Use the requested destination or configured
   Bitwig Projects root. Existing projects keep their observed saved directory.
   If a new project has no established destination, ask for it.
2. Read `midi-bridge controller info` for the intended open project. Match actual
   saved location and path identity. Reuse that target. For an unsaved or
   unopened project use the named **New Project**, **Open**, and first **Save**
   actions in the [computer recipes](../bitwig-explore/references/keyboard-workflow.md).
   Choose the intended destination in the native dialog and verify the actual
   saved path before copying the source. Subsequent saves to that path route
   through [session actions](../bitwig-explore/references/session-actions.md#direct-actions).
3. Follow the [Bitwig project layout](references/project-layout.md#bitwig-project-layout).
   Establish identity in [PERFORMANCE.md](assets/PERFORMANCE.md), leaving
   musical sections unfilled in a new file and preserving existing content.
4. Copy the original to the project's source location without altering it.
   Record its original location, stored path and known source/edition in a
   concise adjacent source record. Reuse an existing copy from the same
   recorded source. If the destination is ambiguous or occupied, retain both
   under distinct names.
   Never silently replace the project source.
5. Verify native save/path, original bytes, project identity and usable
   instruction links. A second pass against matching state must require no
   creation or replacement.
   Report created/reused/conflicting items and stop.

Do not create musical tracks, import notes, normalize/edit the score, select NKIs,
research the piece, load players, or change host-global project-folder settings.
A populated target is reused, never cleared to satisfy a later import precondition.
