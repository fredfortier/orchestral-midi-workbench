# Composition workflow and control boundaries

## Goal

Carry composition work across specialized skill owners, using one resolved
musical target throughout, and verify the resulting procedures on a new
Bach *Qui tollis* project.

## Scope and stages

1. Integrate generic DrivenByMoss OSC submission and bounded feedback into
   the existing bridge process. Preserve the native adapter protocol.
2. Qualify direct and configured actions in the canonical Bach project. Use
   the production checkpoint procedure for experiments and verify the closest
   wrong context without creating a second workspace.
3. Route the seven composition stories through a short skill and specialized
   procedures. Consolidate recurring session actions and preserve the owners
   of notes, curves, routing, and Kontakt setup.
4. Exercise admission, realization, cues, navigation, audition and repair,
   receiver setup, transitions, and export on the Bach project. Save, reopen,
   and verify the project. Revisit code or procedures when an observation
   exposes friction or an ambiguous handoff.

## Validation and evidence

- Workspace build, formatting, tests, and Clippy. Exercise the OSC route with a
  loopback peer and with the installed DrivenByMoss configuration.
- Use the production skills for the Bach exercise and retain operation
  outcomes, timings, preservation checks, and outstanding limits in research.
- Demonstrate the closest wrong target before admitting context-sensitive
  OSC actions for repeatable production use.
- Review the evidence against every composition story before closing the plan.

## Decisions

- Run OSC in the existing bridge service with a separate loopback UDP socket.
  Keep generic host actions out of the native document protocol.
- Use the existing **Bach BWV232 - Qui tollis Composition Smoke** project for
  the entire exercise. Save and verify its main `.bwproject`, then make a
  [file-only checkpoint](../../../skills/score-intake/references/project-layout.md#experimental-project-backups)
  under its `backups/` directory when an experiment requires one. Work in the
  main file throughout. Preserve the original score byte for byte.

## Outstanding OSC and workflow repairs

| Edge | Repair and acceptance check |
| --- | --- |
| Single project | Close the Disk Check, Candidate Check, Transition Check, and Curve Check tabs if present and remove their temporary workspaces. Verify the main project and unrelated projects remain in place. Replace disposable-project instructions in [OSC qualification](../../../crates/osc/ACTIONS.md) and [session actions](../../../skills/bitwig-explore/references/session-actions.md) with the canonical checkpoint rule. Keep experiment receipts in the main project's `evidence/` directory. |
| Project navigation and Save | Qualify `/project/+` and `/project/-` between the canonical project and an already-open project. Check OSC project feedback and the exact native saved path after each switch. Verify a Save updates the intended file, including the known visible-tab and remote-cursor mismatch. Keep New, Open, Close, and first Save in the documented computer procedure. |
| Plug-in window | `/device/window` opened the exact Kontakt editor after native selection. Qualify close and an already-correct visibility state, and verify the preflight rejects a pinned or mismatched cursor. Check the exact native device and actual editor window. Use the integrated Kontakt operation for assignment. |
| Launcher targeting | Recheck whether OSC selection of an empty slot yields the exact track and scene in native `launcher read`. Earlier OSC selection did not supply the native Inspector position. Retain the computer cell selection until the native observation agrees. Use the qualified explicit-path read for a known occupied clip. |
| Timing and geometry | Use the OSC global setters for initialization. Use the verified stored timing-point recipe for a barline-local meter and tempo change, and keep Split within the qualified focused range and clip geometry. An expanded route must preserve the earlier timing, adjacent clips, and sounding occurrences. |

Add repeatable tests for the OSC CLI and service using a loopback peer. Cover
typed arguments, missing and malformed feedback, concurrent submissions, and
late feedback from an earlier request. `submitted` attests to local UDP send.
Keep host-effect verification in the owning procedure.

After cleanup and qualification, run the remaining workflow in the main Bach
file. Complete receiver realization, verify the sung entry and audition, apply
the authorized edits, then save and reopen that exact path. Verify the saved
project's content, timing, cues, and loaded receivers. Evidence from earlier
copies remains useful for individual probes. The main project must supply the
end-to-end evidence. Add a controller operation only if a capability gap remains
after testing the existing OSC, native, and computer routes.

## Progress

- Bridge-owned OSC landed in `67f888c`. Workspace tests, Clippy, and a
  standalone loopback fixture verified submission and feedback.
- The Bach workspace contains byte-verified source, 12 exact clip placements,
  2,173 independently reread note occurrences, and eight saved cues. The
  [exercise record](../../../research/QUI-TOLLIS.md) tracks qualified contexts,
  remaining cases, and fixes arising from the trial.
- The disposable append verified a held tempo and meter transition at a barline
  before and after a byte-identical open. The qualified procedure combines
  OSC framing, native timing evidence, and the host's stored timing controls.
