# Controller entity selection and content reads

## Goal

Resolve musical targets in Bitwig by qualified path from one structural
overview. Read and replace the complete explicit content scope. Rework and
independently verify the live project’s Harp Punish curve after qualification.

## Scope and checks

1. Ground hierarchy, selection, timeline, ownership and CLI grammar in
   Bitwig's guide, the adapter and the accepted controller contract.
2. Use one selection projection, one path grammar and one snapshot transaction.
   Update callers to the same command surface.
3. Verify the Rust CLI and both Java profiles. Qualify the current Bitwig 6.1
   host with explicit paths, selected points and a complete automation read.
4. Apply the original curve edit and reread the exact scope. Compare every
   unrelated curve and note record to the original snapshot.

## Progress

- [x] Traced hierarchy, selection, complete snapshots and point-order rejection.
- [x] Added the canonical metadata and content command contract.
- [x] Passed the controller Rust tests and Clippy, and the production and
  harness Maven builds.
- [x] Verified wire 75 and the requested edit in the sole running Bitwig instance.

Wire 74 reproduced the ordering failure on the explicit Cembalo track read.
With user authorization, the bridge was restarted and the controller
extension reloaded in the existing Bitwig process. Wire 75 resolved all 57
selected points to the Punish lane and returned a complete track snapshot.
The combined overview and narrow selection responses agreed. The native
cue smoke and a no-op replacement passed. A subsequent replacement and
independent read confirmed 46 Punish points at 30–46%, preserved 1,289
notes and every unrelated automation source in the scope, and left the
cello's 50 CC1 points identical to their earlier readback.

Project evidence lives in `evidence/punish-curve-tfwftYvP/`. This live
check covers an outside-clip automation selection. Qualification of selected
points in automation clips and shared sources remains separate.

The [decision record](../../adrs/controller-entity-and-content-addressing.md)
records the scope and rationale. The live project lifecycle remains under the
project instructions.
