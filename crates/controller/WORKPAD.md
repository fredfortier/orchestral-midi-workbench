# Controller implementation and evidence

[ARCHITECTURE.md](ARCHITECTURE.md) is binding. **Overall completion and absolute
simplicity are not certified.** The remaining red flags below are outstanding
obligations, not architectural exceptions.

## Implemented

- One complete ClipSet schema across client, Rust and Java: native clip paths,
  every note/attribute/expression and all owning-track automation, in plain arrays.
- Actual selection and explicit paths share that model. Detail Editor cursor
  content is no longer silently added to selected clips. Explicit path reads do
  not change selection. Entity reads expose typed containment and explicit metadata.
- Complete replacement compiles note and lane changes into one native Undo group,
  checks fresh full expected values, verifies native readback and preserves untouched
  values. Shared automation copies must agree. Unsupported isolation refuses.
- Direct channel CC lane creation inserts no helper device. Exclusive committed
  automation placements can be deleted. Explicit clears verify Bitwig's native
  fallback. Outside-clip lane omission refuses rather than silently becoming clear.
- Deleted general project/import/lifecycle/render/mixer/audio/property-graph
  machinery and alternate Kontakt settings/window adapter. Deleted compression,
  paging, custom size reservation and intermediate MIDI packet processing.
- Deleted controller/loader durable unit tests and their explicit test dependency/
  plugin. Historical research data remain evidence, not executable acceptance.
- Production Java has no selection/history/project experiments; named disposable
  probes live in the separately packaged development harness.
- Kontakt insertion accepts exact installed native registry file or bundle. Scoped
  comparisons use consistent JSON integer types; visibility waits for native state.
  External Wine commands enter the owning Bitwig user/mount/IPC namespaces and
  inherit its synchronization settings. Running status reads do not dispatch
  competing native commands. An unresolved external load blocks further writes;
  post-load closure checks the original receiver identities and routing.
- Skills and their direct references use complete clip/path operations. Supporting
  native project/focus/history actions are keyboard-owned through Use Computer.

## Actual verification

Rust: package all-target check, Clippy with warnings denied and rustfmt.
Java: production and harness Maven packages, Java compiler warnings denied.
Python: environment helper compiled. Diff hygiene checked. No durable tests added
or run as a substitute for host evidence.

Native host: Bitwig6.1, isolated disposable New1 projects, virtual display :100 and
PipeWire target midi_workbench. User profile, project, extensions and default
service ports are independent. Current qualified native/socket wire39 and HTTP API2.
Latest disposable service ports8808/8809; earlier probes used8798/8799.

Ephemeral evidence (not durable tests):

- `/tmp/controller-namespace-smoke/`: combined note+CC update; note/point creation
  and deletion; expression creation; lane clear/default; clearing notes while
  preserving automation; verified no-op; stale complete expected values refused
  with zero musical change. Every success matched an independent complete read.
- `/tmp/controller-final-probes/`: actual arranger-selected two-clip combined
  replacement; one Undo and Redo restored both domains; failure injected after
  the first native write restored all complete content and left Undo/Redo history
  intact; new direct CC lane; committed automation-placement deletion and Undo/
  Redo with original native IDs; outside-lane omission refusal with unchanged music.
- Current final artifact also passed project and exact clip-path Entity reads,
  including generic devices with no device-contents object.
- Native Instrument Layer creation/ensure and Layer channel filtering/remap passed
  independent scoped readback. Native Kontakt insertion now succeeds without the
  Wine namespace engine crash found in the earlier disposable host.
- Independent focused review found no further concrete core defect after the
  addressed omission, selection, status, loader-fence and ownership findings.

## Retired receiver-local MIDI CC experiment — 2026-11-17

Historical evidence only. The commands and wire below are superseded by the
standard slot operations and native track MIDI workflow; do not execute them.

`--midi-cc KNOBS` on slot ensure/update and assign ensures a default sender in
that receiver's slot, immediately before Kontakt, and only the named changing
knob lanes (1 = CC1, 2 = CC2, 3 = CC11). Omit it for velocity-only dynamics and
static values. The wire field is an integer array, default `[]`; bare flags,
invalid or duplicate knobs refuse. Existing compatible devices/curves, routing
and other controls survive. Selectors/choices remain existing-only.

Wire 43 / extension 0.4.2 is installed and live. Immutable wire-named installation,
CLI/service wire checks and adapter handshakes follow the single deployment rule
in [AGENTS.md](../AGENTS.md#verification-and-completion).

The earlier 28-sender/84-lane Bach result was excessive and is superseded.
Saved native result: 20 necessary senders, 20 changing curves; eight velocity-only
senders and 64 unnecessary lanes removed. All 4,599 notes and other clip fields
match; dynamic points retain timing/value/interpolation/curvature/spread. A UI
layer/device selection mismatch removed the portato sender; the existing setup
operation restored it with only knob 1, and complete replacement restored its
original musical curve with fresh native IDs. Post-save complete readback matches. Portato CC11 is a fixed 127 entered through
the native numeric editor, without an automation lane.
Native `--midi-cc 1` created exactly one lane; an independent scoped read verified
receiver order. Audio calibration and listener approval are separate.
Evidence: Bach project `assets/evidence/cc-necessity-cleanup-20261117/verification.json`;
original expression evidence remains under `expression-realization-20261117T195858Z/`.

Failures retained and reconciled: a cached earlier adapter stopped after one
upstream insertion; that sender was removed. One full expression attempt refused
an `owf` UI preview lane and restored exact native content. Reads now exclude
nonstored preview lanes; ensure materializes actual arranger lanes. The desktop
loader's same-namespace `nsenter` failure occurred before receiver insertion;
joining only differing namespaces repaired the actual integrated load. Original
operations were not blindly replayed. No musical Undo experiment was run here.

Checks: workspace tests; Rust package all-target check, workspace Clippy with
warnings denied and rustfmt; production/harness Maven verify; diff hygiene.
Ephemeral smoke proved same-wire changed-artifact refusal with installed bytes
unchanged, receiver-flag encoding, rejection of the upstream layer flag, and
stale-service refusal before POST. No new dependencies or durable tests added.

## Skill repair from session evidence — 2026-11-17

Read-only codex-session query: current thread `01a1eb0d-afa2-7943-ac4a-3177ea9bd389`,
all 16 persisted user-role records at query time, plus four relevant assistant
messages. Source: `sessions/2026/11/17/rollout-2026-11-17T11-48-25-01a1eb0d-afa2-7943-ac4a-3177ea9bd389.jsonl`.
User anchors: lines 85 (execute/refine), 282 (direct knob curves), 525 (fixture
handoff rejected), 965 (per-receiver placement), 975 (single deployment flow).
Assistant anchors: 430 (selection-scoped sparse write rejected explicit-path IDs),
460 (qualification displaced completion), 940 (cached adapter), 1581 (lane/loader
failures). These are evidence, not additional authority. Global and lineage queries
failed on an unrelated empty March rollout; coverage is the current local thread,
not an exhaustive cross-thread review. Raw history was not added to the repository.

Replaced conflicting selection-first/direct-CC guidance with named-path complete
replacement and receiver-parameter curves. Narrowed fixture requirements to
integration diagnostics; agent prepares fixtures, dependencies return to the
unfinished musical task. Replaced obsolete blanket keyboard-outage claims with
per-action/profile evidence. Latest user corrections replace static initialization
as a device/lane trigger: only changing controls justify them. No new control
surface, wrapper or parameter-processing layer. UI recipes and pitfalls live in
[SCRATCHPAD.md](../SCRATCHPAD.md), outside skill runtime prose.

Validation: four skill frontmatter/fence checks passed; 47 local references
resolved; installed symlinks still select the repository sources. CLI stub smoke
passed empty/single/multiple knob arrays, bare/invalid/duplicate knob refusal,
upstream-layer flag refusal and stale-service rejection before POST. Workspace
all-target tests, doctests, Clippy, rustfmt and production/harness Maven verify
passed. No fresh-context model comparison was run.

Author walkthrough: named-track expression enters complete replacement; selected
note edits retain sparse commands; read-only exploration does not invoke setup;
velocity/static cases omit MIDI CC; changing CC1/CC11 select only their knobs;
unknown outcomes reconcile original operations. This is prose inspection plus
actual CLI/native evidence, not a fresh-context model comparison or measured
behavioral-improvement claim. Invocation metadata and installed skill symlinks
are unchanged.

## Native mixing and export — 2026-11-18

The user's explicit mixing request admits the bounded extension in [MIXING.md](MIXING.md).
Added native mixer snapshots and absolute volume/pan batches, plus asynchronous
exact-owner WAV export through the native Export Audio task. No generic property
surface or persistent export-preference mutation was introduced. Production
qualification is Bitwig Studio 6.1, wire 53, adapter 0.4.12; matching CLI/service
and immutable extension were installed and live wire identity checked before use.

Disposable-project qualification proved independent volume/pan readback, one-batch
Undo/Redo, stale refusal before mutation/destination creation, filtered master,
track and return isolation, silent-channel output, pre-fader and real-time export,
and FLOAT32/INT16/INT24 output at explicit rates. Cancelling a real-time render
produced a failed native completion despite an existing partial WAV; a subsequent
mixer read succeeded. Export success uses the final native callback, never file
presence or completed render counts. The extension contains the callback subclass,
without bundling host classes. Cross-rate output can differ by a few milliseconds
through native resampling; synchronized analysis used an engine-matched rate.

Independent review findings were resolved: final encoder completion, connected
and drained engine guards, missing receiver preflight, dispatch-failure eligibility,
terminal native-reference release, project-independent terminal status and retained
mixer failure receipts. The follow-up source review found no remaining blocker.

The authorized project balance pass retained before/after snapshots and complete
musical reads, rendered 27 selected owners twice, and verified native completion,
WAV structure, finite samples, synchronized frame counts, signal and tails. Waveform,
RMS, Welch spectra, shared-scale spectrograms, band comparisons and loudness estimates
are retained in the project's `assets/evidence/mixing-20261118-01/` bundle. Native
cue-to-seconds mapping superseded a provisional constant-tempo diagnostic. Five
faders changed in one native Undo group; all unrequested mixer facts and complete
musical content were preserved. Native Save completed; file modification time and
independent post-save mixer/content reads verified the result. This remains a
measurement-led pass, without listening or artist approval.

Added and registered generalized [bitwig-mixing](../skills/bitwig-mixing/SKILL.md)
and [bitwig-export-audio](../skills/bitwig-export-audio/SKILL.md). Their SPEC/SOURCES
separate official mechanics, industry guidance and editorial defaults; they contain
no case-project details or universal masking thresholds. Validation: workspace
all-target tests, doctests, Clippy with warnings denied, rustfmt, both Maven verify
profiles, skill validators and diff hygiene. The existing unrelated SCRATCHPAD
edits were preserved. No Git commit was made.

The historical computer-access red flag below is superseded for this task's
observed Save, transport and disposable Undo/Redo actions; it remains historical
evidence for its original session, not a current blanket prohibition.

## Remaining red flags

1. **Earlier disposable end-to-end NKI probe remains unqualified.** Current production load/closure evidence is recorded above; the earlier fixture below retains its own failed outcome. The most recent integrated
   probe reached editor ownership qualification: native Bitwig reported visible,
   while the Windows loader enumerated no visible owned Win32 editor. Its timeout
   reported that no NKI was submitted. Do not claim a loaded patch or successful
   closure, replay into these occupied receivers, or compensate manually. Evidence:
   `/tmp/controller-namespace-kontakt-6/` and its process result/error files.
2. **Surrounding keyboard workflow is unqualified.** Use Computer reports no
   apps/browsers and `native pipe path unavailable`. Named native keymap actions
   were inspected, but delivery/focus/readback and wrong-focus probes cannot run.
   Target skills' SPEC.md and keyboard reference explicitly preserve this gap.
   Restore Use Computer; do not use Xpra, CLI UI controls, screenshot parsing or
   guessed bindings. This also prevents visual troubleshooting of the editor gap.

Current development host is `/tmp/controller-namespace-native`; the earlier
`/tmp/controller-architecture-native` instance retained the namespace crash
failure evidence. Both are task-owned disposable hosts. No user project was used
as a fault fixture. No Git commit was made. Continue from these facts after
computer access is restored; do not turn an unverified behavior into a certificate.

## Syllabuilder assignment handoff repair — 2026-11-18

User scope: retain the existing layer/slot ensure -> integrated assign -> exact
kontakt-ctl load workflow, adding `--keep-editor-open` for subsequent GUI builder
configuration. Use actual lyric syllables for slot names. No parallel loader or
customization path is introduced.

Reproduction: a freshly inserted yabridge receiver exposed a visible OS window
while the native editor-visible value was false. The old initial-hidden check
therefore timed out before NKI submission. Repair establishes native open, then
identifies the exact window through one disappearance on close and one same-process
reappearance on reopen, preserving unrelated windows. Keep-open completion verifies
native visibility and the exact owned window after confirmed NKI load.

The original receiver must survive repair. `assign --resume-before-load STATUS`
continues only a retained completed-insertion/ownership-failure receipt explicitly
stating no NKI submission. It validates fresh project, host, track/layer/slot/device
identity, membership and routing (only adapter document generation and transient
visibility may differ), claims the receipt once without overwrite, skips insertion,
and rejoins the same ownership/load/completion stages. Generic occupied slots,
possible prior submission, changed scope and repeated claims refuse. The explicit
NKI remains the caller's settled intent. No raw loader or native command is exposed.

Acceptance pending: matching wire 54 / adapter 0.4.13 deployment, same receiver
continuation, loaded/open handoff, normal fresh assignment, default closure,
negative receipt guards and independent native preservation readback. Build-only
results do not qualify live loading. No kontakt-ctl syllable-control code is added.


## Track MIDI default — 19 November 2026

The automation CLI now accepts track/controller/points directly, defaulting to
channel 1 (JSON 0), through the existing complete-content transaction. Device
parameter discovery remains explicit; implicit writes refuse committed sources.
Canonical owner resolution, every-owner clip coverage and duplicate resolved
sources were independently reviewed and exercised by nine ephemeral offline
facade cases. Slot creation defaults output notes to channel 1. No native wire or
Java implementation changed; adapter/service remain wire 55.

Authorized case study: Bitwig 6.1, Vivaldi RV589 II - Corboz. Removed 16 legacy CC
senders and replaced their controls with 22 channel-1 track MIDI lanes; no new
wrapped clips. Independent native reads preserve all 5,872 notes and note-clip
geometry, and compare every planned curve. Extra primary Soprano anchors, held
tails, the HLS pedal pulse and cello's outside remainder were retained. Existing
Volume and Close Vol parameter curves remain. Evidence:
`/home/user/Shared/Music/Bitwig/Projects/Vivaldi RV589 II - Corboz/evidence/track-midi-20261118T235715Z/verification.json`.
This user-project migration is not an Undo/Redo qualification or measured audio
equivalence test. A pre-existing render-before-NKI-submission loader failure was
reconciled from its terminal error, exited process and unchanged native owner;
its failed slot was retained without replay before restarting the service.


## Owned render preflight and retained empty receiver continuation

The integrated loader captured a black embedded render before opening any Load
dialog when another editor retained foreground. Repair the existing loader to
activate and redraw only its qualified owner before the initial render capture.
Preflight errors carry a distinct no-submission exception type. Resume remains
inside the integrated assignment, skipping insertion only from conclusive original
terminal evidence and the same persistent receiver/routing. The earlier wire-55
render-timeout signature needs a narrowly recognized migration path; arbitrary
loading failures, deadlines and possible submissions remain inadmissible.
Authorized removal of legacy MIDI CC helpers must not invalidate the retained
Kontakt identity, but receiver replacement, routing changes or added devices must
still refuse. Verify negative guards and actual load before dependent realization.
Deploy a new immutable wire/version pair and refresh live authority before writes.
# Standard instrument slots

Three existing operations own distinct results:

* `instrument layer ensure`: target kind (`layer`/`selector`) chooses the container. Create if absent, reuse exact type; refuse conflicting type.
* `instrument slot ensure`: required name and complete nonempty `inputChannels` (native 0..15). Target can end at container (find/create that name) or existing slot (rename/update that precise slot). Always set outer routing -1/-1 (All/Same), ensure Channel Filter then Channel Map at chain start; admit shared channel 0 plus requested note channels, map every channel to displayed 1. Reuse helpers by native type, preserve all other devices and order. Reject duplicated routing helpers before mutation; move a sole existing Filter/Map to the prefix while preserving every other device and their relative order. Select and verify resulting slot in controller code. Retain slot identity on updates. No partial update operation or configurable outer-channel flags.
* `instrument assign`: target slot, optional `loadPatch` plus existing loader runtime/evidence when loading. Reuse sole instrument wherever in chain; insert Kontakt at end if absent. Never require helper topology or alter routing. Multiple instruments refuse. Explicit NKI replaces current Kontakt rack contents with one instrument using integrated loader; no NKI preserves patch/configuration. External loading is a reported stage with retained unknown outcomes, never automatic replay.

Converter destination bindings admit multiple note channels to one explicitly named slot, separate from NKI resource identity. Destination channels are 0..15 JSON / 1..16 CLI. Multi-slot/articulated tracks reserve displayed 1 for shared CC; nonarticulated single-channel tracks may use it for notes. Never rewrite portable UMP identities. Skills use direct shared-channel track MIDI automation and this slot topology by default.

Evidence: complete readback includes all filter enable flags, map destinations and selected slot, persistent instrument identity and sibling membership/order. Verify creation, update, reapplication, selector, existing bare instrument/custom effects, refusal before mutation and Undo in disposable fixture; then migrate active project with complete MIDI preservation and save/reopen.


## Standard routing qualification — wire 62 / adapter 0.4.21

Bitwig 6.1 disposable fixture verified stock factory construction without presets,
repeat/update defaults, nested Layer/Selector paths, full mask replacement,
helper identity/order preservation around an existing Kontakt, one-step Undo,
selected-slot native-ID deletion and Undo, cross-track selection and same-track
inactive-container exclusion. Native flattened device selection follows descendant
ownership; remembered layer selection alone is insufficient. Repositioning uses
detach/insert, never duplicate insertion of an attached native object. Helpers
must be both enabled and active.

Integrated first NKI load and same-NKI sole-instrument replacement passed, with
rack count1, retained native Kontakt identity and default editor closure. Rack
counting requires independent scrollbar-top readback; textured Remove button
background is ignored while its glyph/qualified header position are checked.
No builder control API was added. Retained failure receipts preceded repairs;
unknown operations were never blindly replayed.

The active authorized case standardized existing slots and independently preserved
10 readable clips/5,210 notes except requested destination-channel edits. Inactive
Electronic Continuo has a pre-existing unresolved stored automation reference
(lane21647/reference21663). Its routing has native instrument/sibling preservation,
but its complete MIDI cannot be certified. Reads now recover a null cache only
through the canonical stored target; unresolved targets refuse contextually and
never disappear from a purported complete capture. Disposable probes were removed;
case-specific receipts remain in the project evidence, outside reusable skills.


## Navigation implementation — 20 November 2026

The approved minimal-controller plan replaces delta arranger selection with one
absolute native path operation and moves automation once alongside clips. The
current evidence is recorded in the qualification report below. The known
wire-55 zero-render continuation incident formerly described in ARCHITECTURE.md
is historical evidence only: its retained producer/response proved the render
wait preceded dialog input. That comparison admitted removal of the stock MIDI CC
helper before the same Kontakt while preserving qualified project/host/track/
layer/slot/receiver identity and routing. It does not authorize general historical
scope relaxation. Current one-shot resume-before-load requirements still apply.


Final navigation implementation and qualification passed on wire **70**, adapter
**0.4.29**, harness **0.3.0**. See the
[qualification report](qualification/navigation-20261120/README.md) for build,
disposable native, active-project and documentation checks, retained failure/
Undo evidence, measured payloads and bounded live-source limits. The active
project's 6,043 notes and 43 automation placements are unchanged; original
supported Basso selection is restored. Keyboard focus remains in track headers
and the time-selection smoke left the playback cursor at beat zero.
