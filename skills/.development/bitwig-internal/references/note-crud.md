# CRUD on stored note events

Use [Bitwig's note inspector](https://www.bitwig.com/userguide/latest/inspecting_note_clips/)
for musical meaning and units. The public `NoteStep` documents overlapping scalar
concepts, but does not establish exact source timing, private setters or curve
preservation. Read the actual host note schema and its own inspector command.
All following private names apply only to the host version in the retained probes.

## Read

The [snapshot implementation](../../../../crates/controller/probes/NotesDocumentProbe.java)
reads source notes with `clip.qKA().v33(-Infinity,+Infinity)`. On a source note:

| Value | Inspected method | Meaning |
| --- | --- | --- |
| Key | `v33()` | integer 0–127 |
| Channel | `yhG()` | integer 0–15; UI numbering may be 1–16 |
| Onset | `T1L()` | stored clip-source beats |
| End | `IVb()` | end, **not duration** |
| Schema attributes | `RR()` | entries named by `zeN()`, values by `getValue()` |
| Managed identity | `Nln("managed_object_ID").getValue()` | scoped source identity |

Snapshot the entire persistent payload and all expression points before editing.
The probe's `jaQ(CRe.YLV)` yields host serialization for preservation comparison;
it is opaque host data, not a portable wire representation or a proven restore
API. Preserve actual stored tick values, not the originally requested doubles.
A requested onset `.333333` was stored as `.33333301544189453` in the fixture.

## Update scalars and channel/key

For a supported scalar, resolve its descriptor by name and apply the correctly
typed value; validate the specific range and enable flags before writing.
Do not copy the diagnostic scalar whitelist's 0–1 bound onto unrelated properties
such as repeat count, curves, transpose or timing. Inventory operator dependencies
(chance/recurrence/repeat and their flags), enum values and integer widths first.

For channel/key changes, follow the host inspector's row-transfer semantics:
resolve the source row and destination channel/key row, detach the **same note
object**, and reinsert it. The probe uses `note.YeF()` for the row, `row.hk_()`
for its timeline, `tGW(channel,key,create)` for a row and `YLV(onset,end)` for
collision queries. Removal is `row.v33(oHd,false)`; insertion is `row.jaQ(oHd)`.
See tested channel/curve preservation (retired archive: `crates/controller/validation/editor-selection/expression-channel.jsonl`).

Preflight collisions without creating destination rows or otherwise mutating.
Prove overlap semantics for same-key/channel notes before composing batch moves;
the old diagnostic refuses occupied destinations, which is not final product
behavior. Preserve selection and identity explicitly. Never reconstruct a note
from public grid values just to change its channel.

## Create/delete/replace

Openwig's [native clip/note command example](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L1005-L1025)
passes channel, key, onset, duration and velocity to the host note-insert command.
Our isolated probe uses the inspected clip `TUp(int,int,double,double,double)`
factory and independently reads the returned source note. Resolve exact owner
and factory semantics on the admitted build before using either path.

Subsequent command research (retired archive: `crates/controller/validation/editor-harness/README.md`)
followed Openwig's descriptor → explicit owner → typed arguments pattern:
`clip.EVR().cSc().v33(clip, List.of(channel,key,start,duration,velocity))`.
On the admitted build this descriptor is operation 7349, `insert_note_command`.
The returned source note was independently enumerated; native deletion and
separate Application Undo/Redo were exercised. Focused inspection also located
the Pen interaction's editable-layer insertion path. Before promoting drawing
across all contexts, prove owner/layer routing, defaults/snap, off-grid geometry,
collisions, loops/shared content, selection and resulting identity behavior.
The user's `drw` capability lead still needs its own provenance/call-path match;
do not equate it with an unrelated obfuscated class of the same name.

Delete exact identities from the owning row. Replace explicit selected material
with explicit creations in one logical undo action. Carry identity remappings
when native boundary splitting clones source content. Do not use source IDs as
if they identify a particular loop occurrence.

Detached consolidation is **not** a lossless clone. Raw clip `cu_()` failed a
shared-content relationship lookup in our probe. Find the host's correct copy
context/factory; do not drop relationships or accept lost expressions to make
cloning work. Native overlapping clip insertion already demonstrated boundary
splitting and Undo/Redo; see retained evidence (retired archive: `crates/controller/validation/editor-selection/loop-boundary-replacement.jsonl`).

## Expressions

Read existing curves without creating them: the probe accesses
`note.Vdp().v33(ZkXKind,false)`. A curve's `v33()` lists points; inspect each
point's time, value, interpolation, curvature and spread descriptors.
The inspected enum includes PRESSURE, TRANSPOSE, GAIN, PAN, TIMBRE and FORMANT;
enum membership alone does not establish an editable UI field or unit contract.
Pressure and timbre reads and preservation during channel transfer are tested.
Creation/deletion/movement of every point kind still needs isolated validation.

A scalar pressure/gain/pitch value cannot stand in for a curve. Preserve point
times and interpolation during unrelated edits, including note onset/duration
changes. Probe what the host does to relative curve timing and boundaries.

## Atomic point commands

`midi-bridge controller point {create,update,delete,clear}` names the observed
revision, clip, selected note owner and one of pressure/pitch/gain/pan/timbre.
Rust compiles the intent into one existing `Edit::Update` expression graph;
there is no new native wire message or transaction mechanism. Creation requires a
full Point and returns its native identity. Sparse updates preserve omitted fields;
`--before` also permits a pure ordering edit without a changes file. Anchors must
belong to the same curve and have the same canonical native time. A new point
defaults after existing ties; retiming preserves stable tie order unless explicitly
anchored. Other curves and native note properties remain preservation context.

Canonicalize only changed/new point times before comparing anchors or ordering.
Pure Rust ordering must match the model's numerical equality: `+0.0` and `-0.0`
compare as equal times. `total_cmp` alone reversed that valid model order in a
value-only edit and in seeded variation. Both regressions fail before the shared
stable comparator and pass afterward. This is Rust model consistency evidence;
it does not admit noncanonical private host states or change native tick rules.
Reject wrong owners, unknown points/anchors, repeated fields, self anchors,
different-time anchors and no-op intents before native writes. Formant is preserved,
not exposed as an editable atomic point family. Full per-note graph Plans remain
the batch interface; never broadcast a point identity to another note/curve.

## Commit and verify

The inspected document groups an instant edit with `v33(label,false)` and closes
it with `v33(false)` in `finally`. The boolean distinguishes instant/long-running
changes; it is **not** a rollback switch. Do not hold an edit open over network
processing. Prove exception recovery before admitting multi-write production
operations. Independent full readback and native Undo/Redo are required; a setter
return or closing the edit group is not a successful commit acknowledgment.

## Operational batches and native history

The production CLI compiles explicit CRUD, selected-owner replacement, sparse
selection-wide property sets and transposition into the same absolute Plan.
CLI host evidence (retired archive: `crates/controller/validation/controller-cli/`) proves
complete independent preservation and one native Undo/Redo for the exercised
900-note property batches. The later adjacent-key chord counterexample falsified
general transposition admission; batch write ordering requires the two-phase
ownership pattern below, not merely a collision-free final graph.

`ApplicationProxy.doUndo` invokes document `eel()`; `doRedo` invokes `NMp()`.
The adapter's document-thread history entrypoint checks document/revision immediately
before invoking those same native methods, then independently observes the result.
This avoids the raw recorder Undo path that previously failed on expressive notes.
The CLI host test proves these methods restore full before/after graphs and selection
in the tested contexts. History is project-global: callers must observe its result,
not assume the next history entry belongs to an earlier bridge command.

Temporary note and new-point IDs are globally unique within a Plan, including
across owners/curves. Existing point IDs are scoped to their actual note/curve;
a point ID from another owner cannot be reused as a new temporary. Both Java and
Rust preflight this. Rust also requires an exact bijective new-ID mapping before
Complete. Put recorder acquisition inside the instant group's `try/finally`;
even an acquisition exception must close the group. Active user gestures/nested
long-running groups still require a separate native concurrency admission probe.

## Active native gesture isolation

The held-gesture counterexample (retired archive: `crates/controller/validation/undo-group/`)
proves that nesting an instant change inside a native long-running velocity drag
merges the separate CLI edit into the drag's Undo step. One Undo reverted both
900 velocity changes and the CLI mute. Do not infer isolation from begin/end alone.
On this build `document.cuK().TcB().v33()` gives `dgx`; `TUp()` reports an open
instant/long-running change and `qKA()` gives the current step. The isolated
read-only probe reports false/absent when idle and true/present during the actual
held Inspector drag. Require an enabled, unsuppressed, non-replaying recorder with
no open group before preparing/applying a separate edit or invoking history.
Selection reads remain valid while a group is open. Check again immediately
before native begin; never close, absorb or undo the user's existing group.

The corrected production host test passes the held-velocity scenario: read access
continues; edit/history refuse before writes; release permits an independent CLI
edit; its single Undo/Redo preserves the velocity gesture; a separate Undo restores
the 900-note gesture. Complete independent graphs/selection are exact at each
boundary. The read-only UndoTopologyProbe61 class and its temporary reader hook
are excluded from the production JAR. The guard runs at freeze, immediately before
native begin, and before native history. Broader native lock/scope cases remain
separate admission obligations.

## Native time must survive object reconstruction

The rich off-grid point test found an important counterexample: a new point at
0.4444447 initially reads back exactly, but Redo recreates it at
0.4444446563720703. Raw attribute/factory calls can bypass the host's canonical-time
invariant when assertions are disabled. `oHd.Xtv(double)` asserts `anB.TUp(time)`;
`oHd.TUp(iA)` canonicalizes on deserialization. `anB.v33(double)` is the native
nearest-time codec: `Math.round(1048576 * beats) * 2^-20 + 0.0`. Use that host codec
for explicitly changed/new source timing before writing. Do not rewrite untouched
notes/points or relax complete Undo/Redo comparisons. Initial exact readback of a
noncanonical value did not prove it was a valid native persistent state.

Native Gain UI routing `FWk`/`eVU` uses `FD2.UAA`, whose scale is `fu.CUBE_ROOT`
and display unit is dB. `fw` cubes native values to amplitude and then uses
20*log10: native gain v>0 displays 60*log10(v). Four independent Inspector values
(0, −6, −12, +6 dB) match; +30 dB input clamps to native 2. Preserve raw gain values
and spread; zero is silence, not a finite dB value. Inspector pitch uses
`FD2.heu` (−96..96 semitones); UI +48 reads as native 48. Read compatibility must
not be narrowed to the manual's ±24 editor display description.


## Native event order and rejected-group history

A cumulative rich-curve campaign exposed a point crossing another in time: its
values changed correctly, but descriptor-only retiming left the owner's `v33()`
event list in the old order. Do not hide this by sorting readback. Native `KKl`'s
retiming implementation detaches events with `v33(oHd,false)`, sets canonical
source time, and reinserts through sorted `jaQ(oHd)` / `YLV(oHd,index)`. This
preserves the event's managed identity and other properties. The focused crossing
and original 357-note batch both pass independent graph comparison and exact native
Undo/Redo preservation checks with that pattern. Use the same ownership rule for
note retiming; native note geometry across shared occurrences still needs its
separate mapping/isolation admission.

Expression arrays also carry meaningful order among equal-time points. Native
`VYn.v33(time,value,spread)` inserts through `KKl`'s append/sorted path, which
places new ties after existing points. Retiming an earlier point to an existing
time can therefore reverse the requested tie order; unchanged-time setters do
not reorder anything. After resolving creations, retained identities and exact
deletions, reconcile differing native positions by `curve.v33(point,false)` then
`curve.YLV(point,index)`. `KKl.YLV` accepts the index when its time neighbors are
valid; `pcS` records the indexed ownership change. Do not recreate retained
points, sort readback or sort ties by managed ID. A value-only randomization must
use a stable time sort so it preserves native tie order.

Isolated and production probes cover all five editable curve families, equal-time
creation/reordering/replacement/retiming, exact native Undo/Redo and payload
restoration. Stored points at negative one tick, zero, the note end and one tick
beyond the end survive reconstruction. Boundary values include pressure 0/1,
pan/timbre -1/1, pitch -96/96 semitones, gain 0/2, curvature -1/1 and zero spread.
Do not trim stored curves to the visible note duration. This is point-storage
admission, not a claim about shared-source occurrence isolation or audible
mapping outside the played interval. Injected failures after indexed removal and
insertion preserve complete data and both native history cursor directions.

Reversing a failed `deV` change buffer is insufficient for history preservation.
Its private `ncc` logical-empty flag remains false, so normal group close can add
an empty Undo entry and clear an existing Redo chain. On this admitted build, after
full graph **and opaque payload** restoration, the fresh independent step must
have an empty `tGW` buffer, false `FSQ` serialized-batch flag and false `XOR`
forced-history flag. `Iz2` is an allocation/count estimate: native scalar
coalescing removes records without decrementing it, so a nonzero value does not
prove remaining history effects. If `LHs()` already reports logical emptiness,
context-only/no-write failures need no flag repair; normal group close already
preserves history.
Only then may its `ncc` logical-empty flag be restored to true, verified by
`LHs()`, before ordinary balanced group close. These names are build-specific
bookkeeping mapped from `deV`/`dgx`, not a generic cancellation API. Never mark a
partially restored or merged step empty, clear user history, or compensate by
issuing a project Undo after closing. The isolated recovery test passes both
previous-Undo and existing-Redo preservation after row, dynamics, operator and
curve writes. The development-only sweep additionally passes 63 selected native
mutation boundaries across properties, note create/delete, point CRUD and clear;
see the fault audits (retired archive: `crates/controller/validation/native-faults/`).
Calibration must precede establishing the marker/Redo cursor: calibrating a valid
write after a user Undo would itself replace the Redo chain and invalidate the
experiment. Assert the injected checkpoint actually fired, not just that an error
was returned. Production `apply(String)` has no failure parameter or test hook.
The development loader instruments a private copy with Java 25's Class-File API;
its checkpoint controller is thread-local and disables further injections before
recovery. Instrument actual setter/timeline mutation calls, record checkpoint
labels, and use the loader's resource hierarchy when regenerating stack maps.
Production packaging and bytecode checks must exclude all instrumentation classes,
references and failure literals. Do not reintroduce a test switch on the musical
wire or a durable one-off fault-test subcommand.

See retained counterexamples and corrected observations in
the randomized editing evidence (retired archive: `crates/controller/validation/randomized-editing/`).

## Native batch ordering

Final collision-free geometry alone does not prove safe native write order.
The adjacent-key chord counterexample (retired archive: `crates/controller/validation/runtime-audit/verification.json`)
passes final-graph preflight but sequential transposition displaces a later source
note from its row. Recovery restores the full pre-state; a sort for one direction
cannot implement arbitrary swaps/cycles or create-before-delete Plans.

`Be.tGW(channel,key,true)` resolves/creates the destination row. `sZH` notifies
`Be.TUp(row)` when ownership changes; `q3G` defers empty-row cleanup to native
change processing. Capture each affected note's original timeline before writes.
After complete preflight and native group begin, remove exact deletions and
detach every note whose channel/key/start/duration actually changes. Only then
set canonical final timing on detached notes and insert the same objects into
final channel/key rows. Place remaining creations after vacating source slots;
apply other sparse fields to the final owners. Preserve retained identities and
expression graphs. No native references cross threads and no network wait splits
the group. Avoid detached/reinserted work for unchanged geometry.

The isolated native probe covers all six argument permutations of three-note
key cycles and all six time-cycle permutations, with two points in each of five
editable expression families. Every complete native Undo/Redo and final 900-note
fixture/payload restoration matches. Composed adapter probes cover four of those
cases. Production evidence (retired archive: `crates/controller/validation/batch-geometry/verification.json`)
adds two seeded eight-round cumulative campaigns across all four owners, including
channel/key/onset/duration cycles and create-before-delete replacement. The original
903-note CLI transposition regression passes. Thirteen calibrated write positions
pass failures with both native history cursors preserved (26 refusals), including
native factory return before identity mapping. Broader overlap/drawing and shared
occurrence behavior remain open in the workpad. Instrument actual row placement and native
factory calls in the private development copy, never in the production class.
