# Controller architecture

Status: **Binding contract.** The owner's instructions govern. Existing code and
historical qualification cannot relax this contract.

## Purpose and execution boundaries

Bitwig owns the project, hierarchy, selection, notes, expression, automation and
Undo. Controller provides lightweight discovery, exact native targeting and
complete musical reads/replacement. Skills compose those primitives with
qualified OSC session actions and computer recipes. Native mixing, audio export and MIDI2 interpretation
are bounded operations owned here.

Retain one correlated JSON request/response envelope and qualified Path model.
No generic action/macro executor, reflected property access, mirrored project,
second setup mechanism, transport API, general clip-geometry editing or project lifecycle
API. Existing specialized instrument operations own integrated slot/routing and
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

- [`controller.rs`](src/controller.rs) defines admitted operations.
  [`protocol.rs`](src/protocol.rs), [`frame.rs`](src/frame.rs) and
  [`session.rs`](src/session.rs) own messages, framing and adapter epochs.
- [`project_address.rs`](src/project_address.rs), [`editor.rs`](src/editor.rs)
  and [`automation.rs`](src/automation.rs) own path, note and curve types.
  [`export_midi.rs`](src/export_midi.rs) owns capture interpretation and assembly.
- [`EditorSession`](adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/EditorSession.java)
  connects the extension. The harness packages separately armed disposable
  probes. Probe commands and the filesystem mailbox stay in the harness.

## OSC session actions

The bridge can own one loopback UDP command and feedback socket for the local
DrivenByMoss extension. Its separate HTTP route submits raw OSC addresses and
typed arguments without changing the adapter wire. One active submission can
collect selected feedback addresses for a bounded window. UDP transmission
establishes local submission, and feedback describes asynchronous peer state.
Document effects still require native readback. Window or modal effects need
a corresponding observation. See [setup](../osc/README.md) and the
[architecture decision](../../docs/adrs/osc-through-the-bridge.md).

## Native host execution

The [Bitwig Java adapter](adapters/bitwig-ctl/README.md) keeps published SDK
calls and private document mappings distinct. Qualify each operation against
its host build, owning thread and object lifetime. Host objects stay on the
admitted document thread. Socket waits, blocking external work and live
references stay out of that boundary. Timing-sensitive callbacks require
explicit allocation, blocking, locking and logging budgets.

## Paths and structural reads

`Path = {project, elements:[{kind,name,nativeId?,id?,version?}]}` follows actual
native containment: groups → track → clip or automation lane; track/device chain
→ device → supported Layer/Selector slot → nested device. Plug-ins retain exact
registry `id` and `version`. No fabricated folders or flattened group ancestry.
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
registry identity and existing routing metadata. Structural `info` reads the
project or resolves the requested subtree directly, then reads only descendants.
It never reads notes, points or unrelated plug-in parameters. Explicit
`automation read --target` adds the requested device's parameter/Remote Control
catalogue; ordinary structural reads never do so. No native property bags, graphs,
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
Use existing track/device/slot helpers and arranger collection/range mechanisms;
remove superseded delta/clear selection implementations.

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

Native mutation uses the existing independent Undo transaction and calibrated
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
that operation. Device parameters/macros require explicit beyond-MIDI scope, or
legacy-sender discovery for authorized migration. Committed sources require an
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
The existing integrated operation qualifies the exact native/Win32 editor, invokes
the [Kontakt loader](adapters/kontakt-ctl/ARCHITECTURE.md) and verifies receiver
membership/routing before closing. Only transient editor visibility is
excluded from that comparison. Keep-editor-open
verifies native visibility and the same owned window, records `editorKeptOpen`,
and reports no closure; the loader never closes the editor. External loading is
outside native Undo. Native assignment, NKI submission, loaded patch, samples and
audible playback are distinct facts.

Retain only outstanding external work while it may still act or its result is
unknown; later writes refuse until reconciliation. Status does not compete with
an active loader. Resume-before-load continues the same operation only after a
retained receipt and fresh identity/membership/routing evidence prove no NKI
submission. It never reinserts. An atomic one-shot receipt claim fences submission;
possible prior load, consumed claims or changed scope refuse. Loader activation/
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
  persistent export preferences. [`export_audio.rs`](src/export_audio.rs) owns
  the sample-rate allowlist. Depth is INT16, INT24 or FLOAT32. Float WAV rejects
  dither. Range endpoints must be exact native beats.
- Return an operation ID immediately and observe progress, native outcome and
  exact paths via `export-audio status`. The document thread remains available.
  Independent checks establish file structure, isolation, signal and tails.
  Native completion and musical approval remain separate.

## Self-contained MIDI2 export

```text
Capture v2 = {version: 2, entities, midi: Snapshot, globals[], excluded[], decisions[]?}
Context    = {beat, seconds, numerator, denominator}
native read -> retained capture -> optional editorial copy -> assemble -> .midi2
```

- Capture reads one current document on its owning thread, optionally under an
  exact Path. Exclude inactive tracks and descendants of inactive groups before
  reading their clips unless explicitly included. Mute and solo are separate
  state. Preserve selection, transport and project. Schema 1 requires a new
  capture. Use the same complete snapshot grammar, including one automation
  collection and explicit owners/scope. The capture stays intact. An editorial
  copy may record musical decisions and losses and carries no native write
  authority. See [the capture types](src/export_midi.rs).
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

Loopback HTTP v2 uses `/v2/health` and `/v2/commands` with
`{requestId,adapter?,command}`. The native wire is separately versioned and uses
`{type:documentRequest,request,command}` plus correlated typed observations.
Native frames are plain JSON preceded by a big-endian u32 byte length.
The adapter `hello` identifies wire, adapter and host build. The service
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
Bump wire/adapter version → build → install immutable CLI/service/adapter → reload
exact version → verify live handshake → reread → mutate. Never overwrite a loaded
artifact or admit a mismatched/unknown host build by best-effort reflection.

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
