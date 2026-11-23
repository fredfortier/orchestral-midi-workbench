# Edit track MIDI automation

Default to native arranger track MIDI automation. For launcher scope, use the
[launcher recipe](common-edits.md#launcher-clips). Check the vendor manual and
the exact patch's **Conventional MIDI control map** in INSTRUMENT.md when
available. Confirm the current assignment and response. Distinguish documented
stock mappings from MIDI Learn options and user mappings. A CC name or number
alone does not establish a receiver response.

Use the [standard slot topology](../../kontakt-setup/references/assignment.md#exact-path-assignment):
shared controls use track channel 1 (JSON 0), explicitly admitted by each slot's
stock Filter and mapped with notes to receiver channel 1. Verify actual passage;
failed delivery returns to setup. Preserve authored contours and explicit
exceptions. Arranger creation uses track lanes. Pitch bend is a separate message;
its creation requires an admitted native operation and cannot be encoded as a CC.

Before shaping musical controls, read
[dynamics and expression](dynamics-and-expression.md). Settle each part's sampled
dynamics and independent expression envelope before writing. CC11 is not a
fixed-127 baseline by default, and its curve must not be derived mechanically
from CC1. Preserve unsupported controls and unrelated histories.

## Set track CC curves

```sh
midi-bridge controller automation set --input curves.json --output after.json > /dev/null
```

`curves.json` is one atomic batch:

```json
[{"track":{"project":"Piece","elements":[{"kind":"track","name":"Choir"}]},"channel":0,"controller":1,"points":[{"time":0,"value":0.3779527559055118},{"time":12,"value":0.5039370078740157}]}]
```

Channel defaults to zero (displayed channel 1); explicit channels are 0–15.
CC is 0–119. Native CC values/spread are normalized
0–1: MIDI 48/127 and 64/127 in this example. Preserve full floating precision;
quantization belongs at the MIDI output boundary. Times are quarter-note arranger
beats for track lanes. New lanes require an explicit beat-zero point. LINEAR,
zero curvature and zero spread are defaults. Record required initial held values
as track MIDI state when needed; no sender device is required for static controls.

The command resolves the owning track and its existing note clips, captures all
notes/expression/automation, and uses the canonical `midi replace` transaction.
It creates missing track lanes or edits their complete points. Device discovery,
selection changes and receiver reloads are unnecessary. Existing committed content
refuses an implicit edit: preserve it, and specify its observed `source` only when
that existing source is the requested scope. Source coordinates and loops survive;
the command never prefers wrapped clips over track automation.

Independently read the same clip paths after writing. Compare lane owner/channel/CC,
all point properties, notes, expression, geometry and every unrelated curve.
Repeated owning-track data under several note clips must remain consistent.

## Existing complete snapshots and migration

For combined note/curve changes use [complete replacement](clip-replacement.md).
Retain the complete snapshot, modify points offline and replace once. Preserve
native IDs for retained points; new points need unique temporary IDs. Preserve
equal-time ordering, interpolation, curvature, spread and all shared-source copies.

For an authorized legacy-sender migration, discover actual CC assignments and
sender channels using [device discovery](../../bitwig-explore/references/device-parameters.md).
Compare the full curves and patch semantics before consolidating. Sender physical
0–127 values and spread convert to normalized CC units by division by 127; other
parameter domains require their evidenced conversion. Curves that differ are not
duplicates: resolve their musical passage scope or retain independent behavior.
Check placement offsets/loops before translating to arranger time. Preserve
initial states, tails, pedals, resets and switches. Verify receiver response to the
new shared MIDI controls, then remove redundant sender automation/devices within
the authorized scope. Use [qualified cleanup](../../bitwig-explore/references/keyboard-workflow.md#automation-lane-and-sender-cleanup) with exact owner selection and independent native readback. Clearing points creates a fallback and does not delete a lane.

## Instrument macro fallback

If a verified MIDI CC cannot deliver the requested sound change, find an existing
instrument macro through native discovery. Use the controller for discovery and
writes. Open or manipulate a VST editor through computer interaction only on
explicit user request.

```sh
midi-bridge controller automation read --target device.json > parameters.json
```

Descriptors provide physical ranges, units, identity and mapped Remote Controls.
Then use the device form of the same atomic command:

```json
[{"device":{"project":"Piece","elements":[{"kind":"track","name":"Choir"},{"kind":"device","name":"Instrument"}]},"parameter":"Documented beyond-MIDI control","points":[{"time":0,"value":0.5}]}]
```

A unique observed name, mapped Remote Control name or native parameter ID identifies
the target. Preserve its physical units and confirm the macro does what the
requested curve needs. Keep existing macro assignments and sender devices.

Native message/channel choices follow [Bitwig's MIDI automation lanes](https://www.bitwig.com/userguide/latest/automation/).
