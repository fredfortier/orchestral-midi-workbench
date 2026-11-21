# Rendition file data

This documents the implemented converter schema, not the host execution contract.
[Controller architecture](../controller/ARCHITECTURE.md) owns the proposed runtime
boundaries; [the task workflow](../skills/WORKFLOW.md) owns musical authority.
A valid companion does not establish a loaded, configured or audible receiver.

## Companion and route assignments

[Companion](src/preserve.rs) version 1 preserves source/performance without a
rendition. Version 2 carries [Rendition](src/rendition.rs) version 1 and cannot
also carry competing legacy `receiver_bindings`.

| Collection | Material data |
| --- | --- |
| `tracks` | Stable ID, display name, optional mix intent |
| `instruments` | Stable ID, optional articulation label, absolute NKI execution path, player identity/version/format, declared settings, optional sample root |
| `slots` | Receiver ID, track, musical name, NKI resource ID, complete note input-channel set, instance settings |
| `bindings` | Encoded UMP route -> track, destination note channel, instrument ID and slot ID |

New plans declare explicit `slots`, separating receiver identity/configuration
from reusable NKI resources. A binding's `slot` must match its track, instrument
resource and admitted note channel. Multiple bindings may reach the same slot;
distinct slots may load the same NKI. Admission can include intentional live note
channels absent from the MIDI. Every declared slot still requires a binding.
Omitted slots retain the legacy one-binding-per-receiver interpretation;
`articulationName` remains a legacy resource label, not the new slot identity.

Destination channels are zero-based JSON, 1–16 in controller CLI. Articulated
tracks reserve displayed 1 for shared CC and use 2–16 for notes/keyswitches;
validation rejects destination channel 1 when a track uses multiple note channels.
A nonarticulated single-channel part may use 1 for everything. Explicit assignments
are validated rather than silently renumbered. Standard native slot ensure sets
outer All/Same, stock Filter admitting shared 1 plus the complete note-channel
set, then stock Map all → 1. The receiver receives notes and controls on 1.
Compatible controls use one channel-1 track MIDI lane.

Do not rewrite encoded UMP routes to channel 0: unrelated tracks can reuse the same
slot numbers, and portable receiver controls must remain distinguishable. Native
placement compares full control histories before consolidating; conflicting
independent curves require explicit reconciliation, never last-writer-wins merging.
MIDI values convert to observed native units only at placement. `midi decode`
returns portable performance, not an applied Bitwig routing schedule. No MIDI CC
sender device or automation-clip wrapper is part of this model.

UMP groups do not imply native groups. Encoded routes and track/note-channel
destinations are unique. Explicit slot identity determines whether bindings
share a receiver; an instrument resource ID alone does not.

Validation requires complete occupied-route coverage and rejects unused/unbound
references. There are at most 256 tracks/instruments/slots/bindings and 128 settings per
instrument or slot. IDs use 1–128 ASCII letters/digits/dots/underscores/hyphens; labels
are nonempty, NUL-free and at most 256 UTF-8 bytes. Track names are unique;
articulation labels are unique per destination track. Settings have unique control
IDs, non-null values and explicit units/vocabulary. A setting declaration does
not authorize customization or prove an operation supports it.

The file schema stores absolute `.nki` paths. Task patch identity remains the
relative path under `INSTRUMENTS_ROOT_DIR`; resolving an execution path does not
create a second identity or require a patch checksum.

`set_rendition`/`validate_rendition` compare decoded semantic performance with
`expected_midi_base64` and validate route bindings. Current types have no
`rendition_hash`, NKI hash or evidence SHA field. Optional `note_sources` retain
source/encoded-route and exact note-coordinate provenance, not load receipts.
Restoring original notation additionally requires the source/performance checks
in `preserve.rs`; a matching assignment alone is insufficient.

## Host-specific metadata is not an execution schedule

Optional `bitwig` data contains project ID/name, group name, decision revision and
native meter tick rate. Track `mix` can declare gain/pan. Player `initialName`
requires paired located `initialNameEvidence` (`path`, `locator`, `claims`). These
are existing schema fields, not a portable host model or proof of live state.
Detailed validation remains with `rendition.rs` rather than a copied field catalog.

Older planners derive Performance Controls, private settings or raw player
insertion from this data. Those schedules conflict with the current task boundary;
do not execute, silently prune or mutate their authority to force admission.
The controller/compiler needs repair where no supported independent content path
exists. Legacy loading/checkpoint instructions are not an alternative setup route.

Readable labels are musical intent. Naming-only revision requires preserving
performed messages and all patch/routing/control choices; use the typed supported
API and fresh outputs. Historical native plans do not authorize current writes.

## Conversion-owned receiver adaptation

### Reviewed passages and dynamic curves

New score realizations must review musical passages before constructing adaptation
rules. A conversion plan may carry `passages`, an array of independent assertions:
`id`, `sourcePrefixes`, `track`, `instrument`, `expectedNotes`, and `reason`. Prefixes select
performed source-note identities, including repeat occurrences. When present, the
passages must partition every performed note exactly once; each assertion must
select its exact positive note count and the track/instrument reached through the
encoded route's binding. Unknown instruments, empty/unused/overlapping selectors,
and mismatches fail conversion. Limits are 4096 passages and 4096 prefixes in
total. These assertions are reviewed expectations, never generated from the rules
or resulting routing. They do not choose a technique or prove sample suitability.
An omitted/empty array preserves historical plans without claiming passage review.
The assertions and their musical reasons remain internal conversion evidence;
MIDI plus the companion remain the executable delivery.

Crossfade rules optionally declare `curvePolicy: "channelHold"`; omission or
`"reject"` retains the historical refusal of evolving dynamics. Opt-in renders
the source's explicit relative dynamic-curve samples to MIDI 2.0 CC at rational
quarter-note times, with HOLD interpolation and no invented smoothing. Each curve
must start at zero and have strictly increasing offsets within its note lifetime.
A terminal point exactly at note-off is checked but emits no controller change:
the note's control ownership is the half-open interval [onset, end). There are at
most 65,536 points per note and 1,000,000 control-state points in one conversion.
All simultaneously sounding notes on a route must demand identical controller
values throughout their overlap, including at interior curve changes; disagreement
fails with source witnesses. Compatible demands merge into one axis, initialized
at beat zero. Adjacent notes may change level at their shared boundary. Release
tails remain subject to that channel's subsequent controls; no isolation is implied.

The rendered curve is removed from the performed note's private per-note dynamic
descriptor to avoid applying the same envelope twice; original XML remains
archived. Key-velocity rules still refuse evolving curves. No gate, bowing,
articulation, breath, or curve is inferred from duration or patch names. Source
annotation and MusicXML attack/release support remain the explicit interpretation
surface. Older strict readers reject these new optional plan fields; deploy the
matching converter when using them. Companion and native wire schemas do not change.

`xml_to_rendition` and the CLI's `--rendition-plan` option consume an internal
`adapt::Plan`: the rendition plus source-prefix/complete-articulation/descriptor
rules. This conversion input may be retained in an evidence/archive area. It is
not an additional current configuration the composer must maintain to reproduce
playback. The resulting MIDI2 and companion contain the performed values and
intended assignments.

The optional `initialControllers` array declares additional beat-zero MIDI CCs:
each entry contains a bound `route`, a controller index 0–127, and an unsigned
32-bit `value` 0–4294967295. Omitting the array means no additional initialization.
Values are already in MIDI 2.0 resolution; the converter does not infer 7-bit
scaling or an instrument's interpretation. Every route/controller appears once
and must be distinct from dynamics/expression controllers owned by the rules.
Duplicate, unbound, out-of-range and conflicting entries fail. The converter
encodes them in the MIDI configuration before notes; the importer transfers
those values without choosing musical behavior. Receiver CC mapping and observed
playback remain separate evidence requirements.
MIDI 1.0 compound selector/data-entry indexes 0, 6, 32, 38 and 98–101 are refused:
M2-104-UM v1.1.2 §7.4.6 recommends unified MIDI 2.0 program/registered/assignable
messages instead and recommends receivers ignore those CC indexes. Special CC
values retain their protocol bit layout; this field does not infer their meaning.

Each interpreted source note must match exactly one rule. Rules choose an already
bound encoded route and either retain attack velocity or specify a crossfade law.
A crossfade law declares its CC index, fixed unsigned-16 attack velocity, explicit
dynamic-source policy and optional independent expression-controller default.
Rules sharing a route must share the same law. The converter writes route-wide
controller initialization at beat zero and writes subsequent level changes before
the corresponding note attacks. Existing note timing, pitch, release velocity,
articulation and unconsumed descriptor facets survive. Opted-in dynamic curves
are rendered and consumed as described above. Source routes and changed attack velocities
are retained in the note-source records. Encoder isolation may not
silently move a note to another receiver: an unplanned route is an error.

`requireDynamic` requires an explicit performed dynamic level or an admitted
curve whose first point supplies it.
`dynamicOrVelocity16` uses that level when present and otherwise multiplies
unsigned-16 source attack velocity by 65537. This is the existing project's
linear full-range interpretation policy, not the M2-115 min/center/max protocol
rescaling rule and not an assertion that CC1 has a universal dynamic meaning.
The patch's documented configuration supplies that interpretation. MIDI CC words
are constructed by the existing `wire::cc` helper using pinned midi2 0.11.1;
[M2-104 v1.1.2 §7.4.6](../research/standards/midi/M2-104-UM_v1-1-2_UMP_and_MIDI_2-0_Protocol_Specification.pdf)
defines the 32-bit Control Change representation.

Notes overlapping on a shared route must request equal channel dynamic levels;
otherwise conversion reports both source identities. A channel change also
reaches that player's sounding release tails: no per-note isolation or tail reset
is invented. Initial source program/name context is explicitly replaced by the
rendition assignments and retained in archived source/report provenance. Existing
timed channel messages require an explicit source association and currently fail
with the exact event; they are never broadcast by guessed ownership. Piano-profile
adaptation and evolving curves without an admitted crossfade `channelHold` policy
fail with a scoped unsupported-policy diagnostic. These limits remain outstanding where required;
they do not authorize partial-performance completion claims.
