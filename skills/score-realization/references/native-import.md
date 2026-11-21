# Native placement around complete clip CRUD

For placement, use the native project actions and complete musical edits
described below. Confirm each required action is available before preparing
dependent writes.

1. Compile the explicit performance using [conversion](conversion.md). Authenticate
   MIDI2/companion with `controller midi decode --input FILE --companion FILE` when
   decoded values are needed; this is offline and performs no host placement.
2. Discover existing destination paths with bitwig-explore. For surrounding project,
   folder or clip creation/import actions, first use the named native action route
   in [keyboard workflow](../../bitwig-explore/references/keyboard-workflow.md).
   No unqualified key sequence or screenshot-driven systematic workflow is allowed.
3. Read complete destination clips, convert the settled performed values into the
   documented native Note/Point arrays, and replace using bitwig-edit's
   [clip replacement](../../bitwig-edit/references/clip-replacement.md). Retain
   geometry/ownership and all unrelated automation. Never assume native MIDI file
   import preserves MIDI2 expression or companion meaning; verify actual values.
4. Configure Instrument Layer/Layer routing through kontakt-setup's native
   operations. Use native track-level MIDI automation for documented controls,
   shared across slots where verified routing permits. No MIDI CC sender or
   automation clip is required; report unsupported native operations before
   dependent execution. Load NKI only when requested, through
   its integrated assignment.
5. Compare complete actual notes, expressions and lanes against the intended
   performance in native units. Preserve source/occurrence/loop coordinates. Save
   through a separately qualified native action and verify actual saved identity.

If a required project/folder/clip creation or file-placement action has no qualified
keyboard sequence, leave that dependent placement pending and report the exact
missing action. Existing complete clips remain editable through their native
owner. Do not substitute private construction helpers, compensating manual
controls or a successful compilation for verified placement.
