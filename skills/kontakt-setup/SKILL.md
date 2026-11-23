---
name: kontakt-setup
description: Use the existing integrated midi-bridge instrument assignment operation to load exact NKIs into named Bitwig Instrument Layer/Selector slots. Controller owns host structure and Kontakt assignment; the repo-owned kontakt-ctl only loads the NKI into the automatically opened window. No customization, manual intervention, or alternate loading workflow.
---

# Load the specified instruments

Infer scope from the request and conversation; use
[bitwig-explore](../bitwig-explore/SKILL.md) to resolve the targets:

```text
Existing selection -> observe once atomically; preserve the user's selection
Requested owners/group/project -> discover owners and control selection as requested
  -> exact PERFORMANCE.md bindings for each owner
  -> initialization ready? reuse : instrument-intake -> return here
  -> each binding sequentially: reuse/create structure -> integrated assign/load/close
  -> independent routing readback + PERFORMANCE.md setup evidence -> STOP
```

## Resolve scope and specification

For “selected track” or similar contextual references, follow bitwig-explore's
existing-selection route in one atomic controller observation. For a request to
select owners yourself, follow its discovery/selection route and process all
resolved owners sequentially. Do not change an existing selection merely to
discover it. Use the observation's project identity and qualified owners; never
substitute a remembered cursor or the first matching display name. Ask only when
context and controller evidence leave a consequential ambiguity.

Read the intended project's PERFORMANCE.md. It owns **all orchestration choices**:
qualified tracks, layers, named slots, patches and input/output channels. Execute
its bindings mechanically; do not choose replacements or ask for choices already
settled there. A vendor/family request constrains these bindings. Missing or
conflicting musical choices route to score-realization to settle in PERFORMANCE.md
before assignment; do not silently reorchestrate during setup.

A patch is identified solely by its `.nki` relative file path beneath
`INSTRUMENTS_ROOT_DIR`. Resolve the
relative path against that root for the loader; an absolute execution path is
not another patch identity. Processing does not require rebinding an unchanged
relative path.

## Initialize dependencies and return

Read [eligibility](references/eligibility.md). Missing initialization requires
executing [instrument-intake](../instrument-intake/SKILL.md) as a dependency of
this request, not asking the user to invoke it separately. Pass the exact relative
patch paths and library roots, retained documentation, processing records and
any prior failed operation evidence. Intake reuses completed initialization and
completes only missing operationalization through its documented procedure.

A failed/partial/unknown prior attempt routes to intake's reconciliation branch,
not a blind resave. Intake returns INSTRUMENT.md, path-based processing coverage
and the actual operation outcome. Resume the same setup after verified success;
failure blocks dependent assignment and retains its original evidence. Neither
intake nor setup invents patch, channel or recovery-policy choices.

## Assign and verify

Read [assignment](references/assignment.md) for exact paths and assignment.
Use the [keyboard workflow](../bitwig-explore/references/keyboard-workflow.md)
for supporting project and focus actions.

1. Resolve exact project/Group Track/Track paths with `controller info`. Use
   `instrument read --target target.json` for the intended Instrument Layer or Layer.
   Its native engine must be connected before assignment. An observation never
   changes selection or opens a different project.
2. Apply the specified container and named-slot ensures in
   [assignment](references/assignment.md). Slot ensure reapplies the documented
   stock Filter/Map defaults and selects/verifies the resulting slot.
   Check native name, channel mask, mapping and selection. CLI channels are 1–16;
   JSON/native note channels are 0–15. Preserve unrelated devices and musical content.
3. Use integrated `instrument assign` for the slot. Without `--load-patch`, reuse
   its instrument configuration; an explicit NKI requests replacement. Assignment
   preserves routing and other devices. Loading owns the exact editor and closes
   it by default. For an authorized Syllabuilder handoff use `--keep-editor-open`
   and return its verified window. Conventional controls use native track MIDI;
   read INSTRUMENT.md's control map and return curve edits to bitwig-edit.
4. Process bindings serially. After an unknown/failure, query the original operation
   ID; never close, delete, recreate or replay its receiver to obtain apparent success.
   Retain actual load and closure results and independent native routing readback.

After all assignments complete, independently read back the requested slots and
routing. Do not require a whole-project capture, read the project once per track
or replay assignments to verify them. If scoped readback is unavailable, report
which slots or routing remain unverified. [Assignment](references/assignment.md) owns identifier types, timing
and the bounded routing-check procedure.

The loader only loads the specified stock NKI into the controller-owned window.
Receiver controls follow the shared necessity rule and assignment reference.
Return unsupported musical requirements to realization and report any missing
assignment operation.

Update PERFORMANCE.md's setup evidence with actual assignment coverage and limits;
preserve its orchestration and all unrelated sections. Loading, editor closure,
sample completeness, saved persistence, audible playback and musical approval are
separate facts. Use [session actions](../bitwig-explore/references/session-actions.md#direct-actions) when the requested delivery includes Save.

Keep the direct path tight: reuse selection observations, documentation, runtime,
initialized libraries and matching receivers. Do not export full note/project
graphs or research alternative patches for mechanical setup. Announce required
initialization and its actual processing scope before it starts. Automatically
retain wall-clock and monotonic start/end timing for each command, including
failures, and total elapsed time; a speed requirement is measured acceptance, not a
promise or a claim inferred from a later no-op rerun.
