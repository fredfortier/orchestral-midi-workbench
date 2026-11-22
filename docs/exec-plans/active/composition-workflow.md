# Composition workflow and control boundaries

## Goal

Carry composition work across specialized skill owners, using one resolved
musical target throughout, and verify the resulting procedures on a new
Bach *Qui tollis* project.

## Scope and stages

1. Integrate generic DrivenByMoss OSC submission and bounded feedback into
   the existing bridge process. Preserve the native adapter protocol.
2. Qualify direct and configured actions in an isolated project. Check the
   closest wrong context and record native and UI observations separately.
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
- Use a separate saved project for the exercise. The original score remains
  the source of notation and is preserved byte for byte.

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
