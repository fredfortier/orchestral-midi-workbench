# Native placement around complete clip CRUD

For placement, use the native project actions and complete musical edits
described below. Confirm each required action is available before preparing
dependent writes.

1. Compile the complete performance using [conversion](conversion.md). Authenticate
   MIDI2/companion with `controller midi decode --input FILE --companion FILE`
   before native construction. The fixture for placement is the authenticated
   decoded notes, time, CC and companion ownership. Keep articulation and
   controller decisions in the converter input.
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
4. Resolve shared CC events through the companion's `controlRoutes`, then place
   native channel-1 track MIDI lanes via [automation edits](../../bitwig-edit/references/automation-edits.md).
   Preserve beat-zero values and event timing, divide u32 values by 4,294,967,295
   for native normalized values, and use HOLD with zero curvature. Sampled
   CC events hold to the next event. Use the existing controller no-op or
   replacement checks. Once musical placement is verified, configure requested
   receiver routing and NKI loading through kontakt-setup's integrated operation.
5. Compare complete actual notes, expressions and lanes against the intended
   performance in native units. Preserve source/occurrence/loop coordinates. Save
   through a separately qualified native action and verify actual saved identity.

## Route and compare the notes

For a patch-bound rendition, use the companion's binding for each decoded
route and the native path resolved for that binding's track. For a notation
rehearsal without bindings, pair decoded notes with `note_sources` from the
same authenticated companion in canonical order. Verify count, onset, end,
key and encoded route before using each source part and occurrence to resolve
the intended destination. Several written parts can share an encoded route,
and a single part can use several routes. Preserve the part ownership and any
independent simultaneous voices in the reviewed destination map.

On readback, compare note occurrences per exact destination path as
multisets. Include channel, pitch, quantized start and end, attack and release
velocities, and mapped expression. Ignore new native IDs and selection flags
for this comparison. Check clip geometry and preserved content separately.
Equal note totals can conceal a part swap or a dropped overlap.

If a required project/folder/clip creation or file-placement action has no
qualified route, leave that dependent placement pending and report the exact
missing action. Existing complete clips remain editable through their native
owner. Do not substitute private construction helpers, compensating manual
controls or a successful compilation for verified placement.
