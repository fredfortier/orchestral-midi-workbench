# Native placement around complete clip CRUD

For placement, use the native project actions and complete musical edits
described below. Confirm each required action is available before preparing
dependent writes.

1. Compile the explicit performance using [conversion](conversion.md). Authenticate
   MIDI2/companion with `controller midi decode --input FILE --companion FILE` when
   decoded values are needed; this is offline and performs no host placement.
2. Discover existing destination paths with bitwig-explore. Use
   [session actions](../../bitwig-explore/references/session-actions.md) for
   qualified track creation and exact empty clip placement. Reuse existing
   paths and content where they match. A first Save or Open still uses the
   named native project dialog.
3. Read complete destination clips. Map authenticated decoded notes to the
   exact destination binding and channel, then convert played arranger beats to
   that clip's source coordinates. For a nonlooping clip this is
   `sourceStart + arrangerBeat - clip.start`; prove both endpoints are in
   range. Divide decoded unsigned 16-bit velocity and release velocity by
   65,535 for the native normalized fields. Quantize source start and end
   once to the native `2^-20` beat grid, then subtract to obtain duration.
   Preserve simultaneous voices and verify pitch without dropping fractional
   tuning. Map controls and expression explicitly. Use bitwig-edit's complete
   [clip replacement](../../bitwig-edit/references/clip-replacement.md)
   before/desired/readback. Retain geometry, ownership and unrelated automation.
4. Configure Instrument Layer/Layer routing through kontakt-setup's native
   operations. Use native track-level MIDI automation for documented controls,
   shared across slots where verified routing permits. No MIDI CC sender or
   automation clip is required; report unsupported native operations before
   dependent execution. Load NKI only when requested, through
   its integrated assignment.
5. Compare complete actual notes, expressions and lanes against the intended
   performance in native units. Preserve source/occurrence/loop coordinates. Save
   through a separately qualified native action and verify actual saved identity.

If a required project/folder/clip creation or file-placement action has no
qualified route, leave that dependent placement pending and report the exact
missing action. Existing complete clips remain editable through their native
owner. Do not substitute private construction helpers, compensating manual
controls or a successful compilation for verified placement.
