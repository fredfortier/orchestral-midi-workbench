# Controller architecture

## Entity model

Bitwig owns three related kinds of state. An **entity** is a named structural
object with a qualified path. **Selection** describes the native UI context and
references entities or items inside them. **Musical content** lives in note
and automation sources. An automation lane addresses a control on a track.
Its arranger timeline and automation clips can carry different curves for
that control.[^bitwig-automation] The Inspector can select a track, clip,
note, device or automation point, and the active editor has its own
meaning.[^bitwig-inspector]

```text
Project
  +-- Group [nested Group ...]
        +-- Track
              +-- Arranger Note Clip ---- note source ---- notes, expression
              +-- Launcher Note Clip ---- note source ---- notes, expression
              +-- Automation Lane ------- target identity
              |     +-- outside clips --- curve source --- points
              |     +-- automation clip - curve source --- points
              +-- Device
                    +-- Layer or Selector chain ---- nested devices

Selection ---- qualified entity path + optional content reference
Snapshot ----- explicit paths + complete musical content in that scope
```

The structural path ends at the lane or Note Clip. A point or note reference
carries an identity scoped by its lane or clip. A placement says where a
source plays. Several placements can share one source, so an edit to that
source must account for its aliases. This distinction determines what the
controller can preserve during replacement.

## Command grammar

Use the same qualified JSON `Path` for every operation that addresses a
structural entity. Entity kinds and names follow native containment. Display
order, focus, parameter labels and numeric controller numbers describe
different facts and cannot substitute for a path. Resolve names within the
already identified owner, then carry the observed path to the next operation.

| Command | Scope and result |
| --- | --- |
| `info [--path PATH.json]` | One structural subtree, or the project when omitted, with current native selection from the same document callback. |
| `info --path DEVICE.json --parameters` | Include metadata for the addressed device's parameters and Remote Controls, including current values. The explicit flag bounds this additional read to that device. |
| `selection read` | The same selection representation on its own. This avoids traversing the hierarchy during repeated checks. |
| `selection set --input REQUEST.json` | Absolute native selection request with fresh authority, paths and mode. Return the verified native selection. |
| `content read --paths PATHS.json` | One complete snapshot for a nonempty array of explicit track, group-channel or Note Clip paths. |
| `content read --selection` | The same snapshot grammar for the effective native musical selection, including selected note flags. |
| `content replace --expected BEFORE.json --input DESIRED.json` | Replace the same complete scope and return independently verified content. |

The prefix is `midi-bridge controller`. `--path` takes one path and `--paths`
takes an array. A mutation takes `--input` for its desired data and uses
`--expected` where the precondition is a separate complete snapshot. These
commands emit one JSON document on stdout and diagnostics on stderr. The shell
handles output routing. Use explicit paths for a known native target or
`--selection` when the selected clip, notes or time range define the task.
Replacement requires the same scope in both snapshots and rechecks that scope
against native state before writing.[^cli-grammar]

An explicit content observation verifies the returned owners, clips and
automation against the requested scope. Track and group-channel paths cover
their own Arranger content. Launcher content requires a Launcher Note Clip
path. A clip path selects one clip and its owning automation scope. Missing,
overlapping and unrequested content invalidate the observation.

```text
info -> qualified selection and hierarchy -> resolve target and owning path
  -> content read -> inspect or edit the JSON locally
  -> content replace -> content read of the same scope
```

A lane address identifies the curve family. Use its owning track path to read
the complete arranger snapshot, then filter the result by lane ID and, where
needed, by placement or source. This keeps both the requested scope and the
preservation scope visible. An ambiguous name requires a qualified path or
further user intent. A point ID without ownership cannot choose an edit target.

Use `info --parameters` for the device catalog and `automation set` to express a
complete curve in native units. Specialist commands use `Path` for structural
entities and keep operation IDs, channels, source IDs, and artifact filenames
in their respective domains.

## Purpose and execution boundaries

Bitwig owns the project, hierarchy, selection, notes, expression, automation and
Undo. Controller provides lightweight discovery, exact native targeting and
complete musical reads/replacement. Skills compose those primitives with
qualified OSC session actions and computer recipes. Native mixing, audio export and MIDI2 interpretation
are bounded operations owned here.

The controller uses one correlated JSON envelope and one qualified Path model.
The Rust service compiles edit proposals from correlated native observations.
The adapter sends observations and operation results.
No generic action/macro executor, reflected property access, mirrored project,
second setup mechanism, transport API, general clip-geometry editing or project lifecycle
API. Instrument operations own integrated slot/routing and
Kontakt assignment, including Channel Filter → Channel Map → receiver. Java owns
host calls; Rust owns pure musical planning and external assignment orchestration.
Keyboard delivery and panel focus belong to computer use in the owning skill.

## Runtime and ownership

```text
CLI -> loopback HTTP -> Rust service <== native TCP ==> Bitwig Java extension
                        |                               |
                        |                           document thread
                        |                               |
                        +-- qualified load --> Kontakt editor     Bitwig state
```

- [`controller.rs`](../../crates/controller/src/controller.rs) defines admitted operations.
  [`protocol.rs`](../../crates/controller/src/protocol.rs), [`frame.rs`](../../crates/controller/src/frame.rs) and
  [`session.rs`](../../crates/controller/src/session.rs) own messages, framing and adapter epochs.
- [`project_address.rs`](../../crates/controller/src/project_address.rs), [`editor.rs`](../../crates/controller/src/editor.rs)
  and [`automation.rs`](../../crates/controller/src/automation.rs) own path, note and curve types.
  [`export_midi.rs`](../../crates/controller/src/export_midi.rs) owns capture interpretation and assembly.
- [`EditorSession`](../../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/EditorSession.java)
  connects the extension. The harness packages separately armed disposable
  probes. Probe commands and the filesystem mailbox stay in the harness.

## OSC session actions

The bridge can own one loopback UDP command and feedback socket for the local
DrivenByMoss extension. Its separate HTTP route submits raw OSC addresses and
typed arguments without changing the adapter wire. One active submission can
collect selected feedback addresses for a bounded window. UDP transmission
establishes local submission, and feedback describes asynchronous peer state.
Document effects still require native readback. Window or modal effects need
a corresponding observation. See [OSC transport](../../crates/osc/README.md).

## Native host execution

The [Bitwig Java adapter](../../crates/controller/adapters/bitwig-ctl/README.md) keeps published SDK
calls and private document mappings distinct. Qualify each operation against
its host build, owning thread and object lifetime. Host objects stay on the
admitted document thread. Socket waits, blocking external work and live
references stay out of that boundary. Timing-sensitive callbacks require
explicit allocation, blocking, locking and logging budgets.

## Paths and structural reads

`Path` consists of a project and a sequence of typed elements. It follows actual
native containment: groups → track → clip or automation lane; track/device chain
→ device → supported Layer/Selector slot → nested device. Plug-in elements carry
their exact registry identity. No fabricated folders or flattened group ancestry.
Names containing separators remain literal strings. A launcher Note Clip is a
`launcherClip` child of its track with a zero-based native scene index.
Clip resolution checks scene, name and any returned native identity.

Names/kinds resolve within their immediate parent. Returned `nativeId`
discriminators resolve duplicate siblings and must still match that named child.
Ambiguous, missing, renamed, moved or stale paths refuse. Native discriminators
are current observation handles, not durable musical identities. A displayed
lane index locates a row within its observed track; it is not identity.

`Entity = {path,kind,metadata,children}` contains name/native identity, track
activity, clip placement/source/loop geometry, automation-lane locator, device
registry identity and existing routing metadata. A lane's target descriptor
identifies its parameter or MIDI channel and controller. If the native target
cannot be resolved, structural reads return the lane's native identity, index
and `targetResolution: "unresolved"`. Complete automation reads still refuse
when a lane in their scope has an unresolved target. Structural `info`
adds a root-level `selection` to that entity tree. It returns identity and
structure without serializing notes, curve points or unrelated plug-in
parameters in its default form. Explicit `--parameters` adds the requested
device's parameter and Remote Control catalog. No native property bags, graphs,
plug-in payloads or speculative fields belong in Entity.

## Native selection

Only `selection read` and absolute `selection set` are admitted. Set carries fresh
`document`/`revision`, returned `paths`, `mode:objects|time` and a nullable range.
Object targets are track headers, arranger clip objects, devices and supported
slots. Time targets are track or automation lanes with increasing arranger-beat
bounds, normalized intentionally at native 2^-20 beat precision. Preflight every
target and owner combination before changing anything. Reject incompatible
families/owners, duplicates and stale authority. The first requested member is
primary where supported; verify that fact independently of membership.

An observation member has the fields of `Path` and, for selected content, an
additional `item`. A selected note uses the owning Note Clip path and
`item: {kind:"note", id}`. A selected automation point uses the lane path and
`item: {kind:"automationPoint", id, source, placement?}`. The placement is
present when the native context identifies it. For a shared source with no
resolved placement, retain the source identity and require placement resolution
for occurrence-specific work. Structural members are plain paths. Reuse the
same path resolver and selection reader in project overview and selection-only
reads. Selection observation does not imply write support for every member kind.

Empty object paths clear the active supported selection. Readback distinguishes
membership, primary, active editor and object/time mode; unsupported membership
must be explicit. Reads never move selection or keyboard focus. Membership,
primary and mode describe
the native Inspector target; active editor is observed independently from real
focus. Selection completes its Inspector handoff and readback after native
end-of-event mode/focus reconciliation, with request/lifetime fences rechecked.
Never inject `active_selection` to imitate keyboard focus. Native selection
does not establish panel focus: Delete, Rename, arrows and object/time commands
require their owning computer recipe's focus, selection and binding prerequisites.
Track, device and slot selection use the native selection helpers. Arranger
selection uses the native collection and range mechanisms.

## Launcher slot creation

`launcher read` observes the Inspector's single native launcher note slot and
returns its qualified track, zero-based scene, native slot ID and nullable clip
path. `launcher create` requires that exact fresh observation for an empty
slot and creates an empty Note Clip of 1–1024 whole quarter-note beats in one
native Undo step. Verify the slot, the created clip and its source and loop
length before returning authority. Slot creation and subsequent MIDI replacement
are independent Undo steps.

## One complete musical snapshot

```text
Snapshot = {document,revision,selection,paths,owners,clips:[Clip],automation:[Lane]}
paths = [Path] | null  // exact explicit read scope; null means effective selection
owners = [Path]       // qualified native track/group-channel owners
selection = {editor,kind,from,to,lanes:[nativeId],objects:[nativeId]}
Clip = {path,id,lane,track,content,start,duration,sourceStart,sourceEnd,
        loopStart,loopLength,loopEnabled,notes:[{note:Note,selected:boolean}]}
Note = {id,channel,key,start,duration,muted,velocity,releaseVelocity,
        velocitySpread,contentStart,operators,expressions}
expressions = {pressure:[Point],pitch:[Point],gain:[Point],pan:[Point],
               timbre:[Point],formant:[Point]}
operators = {chanceEnabled,chance,occurrenceEnabled,occurrence,
             recurrenceEnabled,recurrenceLength,recurrenceMask,
             repeatEnabled,repeatCount,repeatCurve,repeatVelocityEnd,repeatVelocityCurve}
Point = {id,time,value,interpolation:HOLD|LINEAR,curvature,spread}
Lane = {owner:Path,placement:arranger|clip|launcher,track,lane,clip,source,noteClip?,
        aliases:[nativePlacementId],referenceCount,mapping,target,points:[Point]}
mapping = {} | {startBeats,durationBeats,sourceStartBeats,sourceEndBeats,
                loopEnabled,loopStart,loopLength}
target = {kind:midiCc|parameter,nativeParameterId,name,pointType,domain,
          min?,max?,unit?,fallbackValue?,channel?,controller?}
```

Read every note, complete expression graph and complete owning-track automation.
Automation appears once alongside clips, including tracks without note clips;
never nest duplicate copies or invent note clips as automation owners. Distinct
placements remain distinct even when sources are shared. `aliases` and
`referenceCount` preserve shared-source authority. Explicit track/group paths
include their own content; child tracks are separately addressed. Clip paths read
whole sources, not just selected notes. `selected` retains effective note intent.
Selected launcher slots and explicit launcher clip paths use the same note and
expression representation. Launcher placement starts at zero in clip-local
beats. Launcher automation is scoped to the observed note clips and identified
by `noteClip` in addition to the automation placement ID. Track/group MIDI
reads and MIDI2 arrangement capture retain their arranger scope.

Keep ordinary arrays, complete values and stable equal-time order. No pagination,
compression, sparse views, note-only/automation-only observation protocols,
subscription caches or adaptive payload machinery. About 1 MB is a prompt to
inspect junk fields, not permission to truncate or impose a musical size ceiling.
Frames reject above 16 MiB and offline inputs above 64 MiB as input safeguards.

The adapter emits automation points in increasing source-time order and keeps
native enumeration order among equal-time points. A remaining order failure
identifies the track, lane, source and offending point pair. Apply the same
canonical order to ordinary reads, the edit preflight and independent readback.

Times are quarter-note beats with native source/placement/loop coordinates;
expression time uses its source plus `contentStart`. MIDI channels are 0–15,
velocities/pressure 0–1, pitch semitones, pan/timbre −1–1, gain native cube-root
amplitude. Keep exact stored precision; quantize only intentional new/changed
native times. Operators and unsupported domains remain explicit, never silently
flattened. Loops/aliases are not extra authored notes.

## Replacement, ownership and failure

```text
observed Snapshot + desired Snapshot -> native prepare/freeze -> Rust Plan
      Plan = document + revision + note CRUD + complete curve changes
      Plan -> one native Undo transaction -> independent Snapshot + ID mappings
```

Preserve paths, owners, selection, clip/source/loop geometry,
untouched attributes and unrelated curves. Store the original explicit scope for
freeze and reread; reconstructing it from clips loses automation-only tracks and
can widen clip-only requests. Unchanged replacement independently rereads and
compares the same scope without writes or an Undo step.

Native prepare freezes current complete values. Rust compares expected content
and compiles note create/update/delete and complete curve replacements into one
Plan. Revision is a freshness/transaction cursor, not sufficient proof of content.
Preflight source aliases, temporary identities, collisions, domains, ordering and
native precision before mutation. Reject unsupported occurrence isolation. Complete
readback and point identity mappings verify intended changes and preservation;
never treat an echoed request as independent verification.

Native mutation uses an independent Undo transaction and calibrated
rollback behavior.
Known refusal, partial failure and unknown outcome remain distinct. Recover lost
responses through the original operation identity and readback; never blindly
retry or issue keyboard Undo against an unknown outcome.

A retained lane with `points:[]` clears its stored points; Bitwig materializes a
beat-zero fallback point, whose exact value and native identity must be verified.
Omitting an observed exclusive committed automation placement deletes that
placement. Omitting an outside-clip lane refuses; whole-lane UI deletion is a
separate authorized computer recipe. Shared sources require consistent desired
points across aliases. Preserve discrete lanes completely and label unsupported
mutation domains. New/changed points retain source-time order and explicit ties.
New notes/points use unique temporary identities replaced by native readback IDs;
retained identities update existing content. Clearing notes preserves automation.

Conventional controls use documented native track MIDI automation. Direct CC
values/spread are normalized 0–1; sender-device values may be 0–127. New CC lanes
admit 0–119; channel-mode messages 120–127 and pitch-bend creation are outside
that operation. Device parameters and macros require explicit beyond-MIDI scope
or inspection of a sender that the user requests to remove. Committed sources require an
explicit source; overlapping content refuses an implicit track edit. New parameter
curves carry the qualified device and physical target descriptor; resolve identity,
range/domain and automation eligibility before writes. Already-correct batches
produce no history mutation. New launcher curves use `placement: "launcher"`
and name the observed `noteClip` in a launcher-only snapshot for that track.
The automation placement is created in the same scene and uses that clip's loop
and source coordinates. If its target has no track-wide lane yet, the native
lane factory also initializes that lane's arranger fallback at beat zero.

## Integrated instruments

Layer ensure creates/reuses the named stock Layer/Selector; conflicting types
refuse. Slot ensure takes a musical name and complete nonempty note-channel set.
After a rename, resolve the unchanged enclosing container path and select the
resolved slot object. Verify its new name, preserved identity and routing in
the result.
It reapplies outer All → Same and stock Filter admitting shared displayed channel
1 plus the requested notes, then Map sending all sixteen channels to displayed 1.
Reuse helpers through the native factory; move a sole existing helper to the
prefix while preserving other device identity/order. Duplicate helpers refuse.
One native Undo group covers creation/routing/name changes. Select and independently
verify the resulting slot, all filter flags/map destinations and enabled states.
Omitting ensure preserves manual customization and Selector performance policy.

Assignment independently reuses the sole Kontakt anywhere in the slot or inserts
at the end when no instrument exists; multiple instruments refuse. It does not
require or modify routing helpers. Without an explicit NKI, preserve patch state.
With an explicit NKI, record the Win32 window inventory before assignment.
Bind the single new window that Bitwig opens automatically on insertion and
load directly into it after a scoped native read. For an existing hidden
receiver, open its editor once and qualify the resulting window the same way.
An already-visible existing editor cannot be correlated from that inventory and
the request refuses before changing it.

The [Kontakt loader](kontakt-loader.md) loads into the
qualified handle and PID. Verify receiver membership and routing after loading,
ignoring transient native editor visibility. With keep-editor-open, verify that
the same Win32 editor remains visible, record `editorKeptOpen`, and report no
closure. Otherwise close the editor after loading and verify its disappearance.
External loading is outside native Undo. Native assignment, NKI submission,
loaded patch, samples and audible playback are distinct facts.

Retain only outstanding external work while it may still act or its result is
unknown; later writes refuse until reconciliation. Status does not compete with
an active loader. A native editor-visibility operation invalidated by a document
lifetime change retains a terminal unknown receipt. It releases its write guard
because it has no deferred document edit. Resume-before-load continues the same
operation only after a retained receipt and fresh identity/membership/routing
evidence prove no NKI
submission. It never reinserts. An atomic one-shot receipt claim fences submission;
possible submission, consumed claims or changed scope refuse. Loader activation/
render preparation does not establish dialog submission or permit customization.
## Mixing and audio export

```text
MixerObservation = document + revision + host/project context + beatSeconds[]
                   + tracks[] + limitations[]
MixerTrack       = Path + kind + state + volume/pan + output + sends[] + devices[]
MixSet          = expected MixerObservation + [{path, volumeDb?, pan?}]
AudioStart      = operation + {expected, tracks: Path[], directory, start, end,
                              sampleRate, bitDepth, dither, preFader, realTime}
AudioStatus     = same operation
```

- `mix read` captures one stopped-document snapshot with host identity, generation,
  revision, hierarchy, master, routing, sends and ordered device identities and
  activation. `beatSeconds` maps zero, the current beat and cue beats through
  the native tempo map. Capture time is envelope metadata. Volume reports native
  physical value, linear amplitude and dB (null at silence). Pan records range
  and mode. Distinguish stored and effective values and report unsupported fields.
- `mix set` accepts a fresh complete expected snapshot and absolute volume/pan
  changes for exact Paths. Require stopped transport and unchanged mixer scope.
  Refuse recording, automation writing, render, save and automated/modulated
  targets. Preflight the whole batch, write under one native Undo group and
  independently reread. Preserve musical content and unrequested mixer state.
- `export-audio start` takes fresh scope, exact track/group/return/master Paths,
  a quarter-note range, a fresh output directory and admitted WAV options
  (sample rate, depth, dither, pre/post-fader and real-time). Resolve every path
  before changing the queue or settings. Reject missing targets, unsupported
  options and existing output paths. Use task-owned settings and preserve
  persistent export preferences. [`export_audio.rs`](../../crates/controller/src/export_audio.rs) owns
  the sample-rate allowlist. Depth is INT16, INT24 or FLOAT32. Float WAV rejects
  dither. Range endpoints must be exact native beats.
- Return an operation ID immediately and observe progress, native outcome and
  exact paths via `export-audio status`. The document thread remains available.
  Independent checks establish file structure, isolation, signal and tails.
  Native completion and musical approval remain separate.

## Self-contained MIDI2 export

```text
Capture    = {entities, midi: Snapshot, globals[], excluded[], decisions[]?}
Context    = {beat, seconds, numerator, denominator}
native read -> retained capture -> optional editorial copy -> assemble -> .midi2
```

- Capture reads one current document on its owning thread, optionally under an
  exact Path. Exclude inactive tracks and descendants of inactive groups before
  reading their clips unless explicitly included. Mute and solo are separate
  state. Preserve selection, transport and project. Capture uses the complete
  snapshot grammar, including one automation
  collection and explicit owners/scope. The capture stays intact. An editorial
  copy may record musical decisions and losses and carries no native write
  authority. See [the capture types](../../crates/controller/src/export_midi.rs).
- Plug-in paths and capture inventory read identity from the native serialized
  contents. CLAP contents use `clap_device_id` and the `clap-` prefix, including
  inside Instrument Layer and Selector slots.
- For articulated tracks, use actual stock Filter/Map routing to interpret note
  channels and shared displayed channel 1 (JSON 0). Resolve slot names against
  the converter vocabulary. Encode supported meaning as Note On attributes
  under one musical track identity. Retain an explicit remark for an ambiguous
  slot name. Receiver channels carry local addresses. Require interpretation of
  nonshared CC curves with points before collapsing routes. Use a documented
  mapping for a plug-in parameter's function, units and polarity. Remove a
  redundant DAW curve once it is represented by the interpreted control.
- Lyric slots use `[lyrics lang=la reset=20] et in ter-ra pax`. Language and reset
  key are optional. Spaces separate words and hyphens separate syllables.
  The reset key marks command notes on admitted channels. Keep them as playback
  commands outside engraved pitch. Unaligned text remains a timed performance
  remark. Unmarked slots carry no inferred lyric semantics.
- Expand exact source/loop occurrences into arranger time, clipping tails at
  placement/loop ends and reporting clipping over one tick. Deduplicate repeated
  observations of track automation while preserving distinct placements and
  owners. Use 61,440 ticks per quarter, round once and preserve positive notes
  for at least one tick. Preserve equal-time dependencies and report maximum
  error. Keep unsupported operators/expression as explicit residuals.
- Sample continuous controls within 1/1024 normalized error and preserve HOLD
  boundaries. Native timing samples use 1/16 quarter-note spacing plus exact
  stored meter changes. Tempo uses interval-average quarter duration. Check
  decoded timing against sampled endpoints and report interior approximation.
  Bound work to 100,000 loop segments, points per control and timing samples
  respectively, and one million expanded events. Reject excess rather than
  truncate. For M2-113 squared Volume, convert native cube-root amplitude `x`
  using `x^(3/2)`, consistently for CC7 and per-note Volume.
- Reuse the converter's wire and clip model. Put portable musical identities and
  any residuals in the file. Leave native IDs, object graphs, plug-in state and
  consumed routing in the capture. Validate bytes before atomic, no-clobber
  publication. Reconstruction needs only the file.

## Wire, deployment and verification

The [loopback HTTP service](../../crates/controller/src/api.rs) exposes health and dispatches
`{requestId,adapter?,command}`. The native connection uses
`{type:documentRequest,request,command}` plus correlated typed observations.
Native frames are plain JSON preceded by a big-endian u32 byte length.
The adapter `hello` identifies the protocol, adapter and host build. The service
`welcome` supplies the session epoch and maximum message size. Reject empty
or oversized frames. Request correlation, operation identity, document revision
and session epoch have distinct lifetimes.
Bound externally supplied frames and queues. Define connection cancellation,
shutdown and partial-frame behavior while preserving original operation identity
for recovery.
For a repeated mutation request ID, the service compares the complete intent and
returns the retained result or state of the original operation. A conflicting
intent fails. This bounded record lives in the service process. After loss of
the record, use current native readback and retained evidence to reconcile.
The CLI, service and adapter use the same protocol contract. Deployment uses
immutable artifacts. A live handshake admits only a matching stack and a
qualified host build. A fresh native read establishes mutation authority.

Durable nominal unit tests are prohibited. Use workspace formatting/tests/Clippy,
production/harness Java verification and focused ephemeral probes. Native claims
require disposable material: duplicate names, groups, multiple/empty selections,
devices/slots, automation-only owners, primary/mode/range, stale refusal, failure
readback, full content preservation and relevant Undo/Redo. Active-project smoke
requires identity, saved checkpoint, baseline content/selection, no-op replacement,
selection restoration and unchanged musical/routing/geometry/name readback.
Record elapsed time and payload sizes for scoped structure, selection and content;
reject redundant full-project reads and unnecessary payloads.

Compilation establishes buildability. Native behavior, semantic fidelity,
instrument support and audible playback require their own evidence.

[^bitwig-automation]: The Bitwig user guide distinguishes track and clip
    automation editing modes, including Launcher clips.
    [Automation Editor Panel](https://www.bitwig.com/userguide/latest/the_automation_editor_panel/)

[^bitwig-inspector]: Bitwig describes the Inspector as following the active
    panel's selection and enumerates tracks, clips, notes, devices and
    automation points. The [native adapter](../../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/internal/Selection61.java)
    reads Inspector membership and editor focus from distinct native fields.
    [Meet Inspector Panel](https://www.bitwig.com/userguide/latest/meet_inspector_panel/)

[^cli-grammar]: The authors of the [Command Line Interface Guidelines](https://clig.dev/)
    recommend composable commands, consistent names and options, machine-readable
    stdout and actionable diagnostics. These principles support reusing JSON
    paths and snapshots directly in pipelines. The preservation boundary and
    explicit write preconditions are local controller decisions.
    [Simple parts](https://clig.dev/#simple-parts-that-work-together)
    · [Output](https://clig.dev/#output)
    · [Subcommands](https://clig.dev/#subcommands)
