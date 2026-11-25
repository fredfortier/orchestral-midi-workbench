# Bitwig adapter state authority

## Goal

Bitwig supplies the current state for admitting project reads and writes. Adapter receipts describe requests and outcomes. They do not lock the project.

## Scope

- Remove project-wide gates based on unfinished instrument or export receipts.
- Derive scoped observation identifiers from Bitwig observations instead of the order in which the adapter sees them.
- Preserve request correlation, native Undo and readback, and the ownership of native asynchronous work.

## Validation

Build the production and harness profiles and check the version-matched controller. Qualify live behavior only in the admitted Bitwig session and disposable material. Use no mocked host tests.

## Progress

- [x] Reproduce the production launcher refusal and identify its source.
- [x] Remove receipt-based project admission, derive scoped revisions from observations, and carry content snapshots with the edit.
- [x] Build both Maven profiles and the matching controller. Verify controller Clippy, workspace formatting, and the patch diff.
- [x] Load the matching adapter and listener in the running Bitwig 6.1 instance.
- [x] Verify live cue readback, stale-slot refusal with both slots preserved, repeatable observation after switching A → B → A, creation at 1,025 beats, and exact 115-note replacement in the disposable Navigation qualification project.
- [x] Verify the requested 64-beat Cembalo launcher clip and exact 115-note readback in the production project. Compare the track's Arranger clips and automation before and after.

Native evidence is retained in `/tmp/bitwig-harpsichord.sKRHxP`. The production
project remains open with the new clip available for audition. Playback and
saving are outside this validation.
