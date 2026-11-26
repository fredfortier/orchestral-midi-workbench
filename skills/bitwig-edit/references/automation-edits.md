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

## Draw sparse musical curves

1. Read the existing source and align it with the sounding notes and relevant
   performance cues. Identify what each gesture does before smoothing it.
   Describe the initial level, onset of the rise, crest or plateau, release and
   return in musical terms, then give each necessary anchor a time and a value
   in the target's units. Resolve cue or bar references to the actual played
   occurrence. Preserve intentional local accents and the relative weight of
   crests, rests, resets, tails and boundary values. A cue can explain a gesture
   without requiring a breakpoint.
2. Start with the required anchors. Use straight ramps and constant spans
   wherever they serve the gesture. For an eased rise or release, use a
   shape-preserving reference that passes through the anchors and stays within
   the intended range. Add only the linear breakpoints needed to approximate it
   at the same beats within a stated value tolerance. Remove collinear and
   redundant flat points where their removal preserves timing and held state.
   Use HOLD for intentional steps and switches, with zero curvature. Keep their
   exact times, boundaries and equal-time order outside any fitting pass.
3. For portable continuous MIDI CC, use LINEAR with zero curvature and spread
   and confirm destination channel, controller and receiver behavior. Plan in
   MIDI 1.0 CC units (0–127) and store native CC values as level / 127 with full
   precision. Budget curve approximation in CC steps, then separately check
   7-bit rounding and any audible change from discrete event timing. Start with
   a tolerance of roughly one CC step for a gentle continuous control if the
   receiver permits it. Preserve seek and initial state. Sparse Bitwig points
   define the shape. A MIDI renderer samples the ramps into discrete CC events
   at its delivery cadence. A device-parameter curve uses its own value domain.
   It requires a verified receiver mapping before translation to MIDI CC.
4. Check the final ramps after [native time quantization](clip-replacement.md).
   Confirm exact required anchors, range, intended direction, boundary
   continuity and approximation error between anchors. Account for every
   extra point by its musical or approximation role. Apply through the complete
   native edit and compare fresh readback with the plan and preserved content.

### When a helper is worthwhile

Use direct anchor placement for flats and simple ramps. For repeated eased
phrases or dense existing curves, an offline helper can take resolved beats,
CC levels, protected anchors and a shape such as hold, ramp or eased swell.
Keep the agent responsible for choosing those musical anchors. The helper
would return proposed points and an error report for review before replacement.

[SCAMP's expenvelope](https://scamp.marcevanstein.com/expenvelope/expenvelope.envelope.Envelope.html)
models musical envelopes with levels, durations and segment shapes.
[SciPy PCHIP](https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.PchipInterpolator.html)
is an alternative for smooth, shape-preserving interpolation from points alone.
Either can supply the reference shape. For minimum point count on a fixed
candidate grid, find the shortest path through links whose straight segments
meet the same-beat value tolerance, splitting at protected anchors and steps.
Check the worst error between sample times as well when claiming a continuous
bound. Report the candidate grid and tolerance alongside any optimality claim.
The helper would export LINEAR or HOLD point proposals for the existing native
transaction. Event sampling and receiver routing remain part of MIDI delivery.

## Set track CC curves

```sh
midi-bridge controller automation set --input curves.json > after.json
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
notes/expression/automation, and uses the canonical `content replace` transaction.
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
midi-bridge controller info --path device.json --parameters > parameters.json
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
