# Skill synthesis and evidence

The 2026-10-20 synthesis below is historical. Current admission still belongs to
the owning controller contract/workpad and retained probes; later findings are
recorded in the dated sections below. Research is not runtime instruction.

## Initial synthesis — 2026-10-20

Class: integration-documentation. Shape: reference-backed expert. Optional deep
categories require separate lookups; a single inline file would force irrelevant
loads. No delegation, generated workflow or provider-specific tool requirement.
The canonical source lives in `skills/.development/bitwig-internal` and is
exposed through its installed skill alias. Source code is studied, not copied from GPL
Openwig; source citations retain its provenance. No Bitwig vendor code is bundled.

| Source | Kind / contribution | Boundary |
| --- | --- | --- |
| [Agent Skills specification](https://agentskills.io/specification) | format authority; frontmatter and relative resources | not Bitwig behavior |
| [Existing Java guide](../../../../skills/bitwig-controller/references/java-extensions.md) | copied SDK docs/templates; bootstrap | public API only |
| [Bitwig note selection](https://www.bitwig.com/userguide/latest/inspecting_note_clips/) | vendor musical semantics, channel, timing, selection | no private method contract |
| [Layered editor](https://www.bitwig.com/userguide/latest/working_with_note_events/) | multiple clips/tracks, expressions | source/selection mapping still probed |
| [Local Java probe](../../../controller/probes/NotesDocumentProbe.java) | implementation trace, exact reflection signatures | one admitted build; diagnostics |
| [Runtime evidence](../../../controller/validation/editor-selection/README.md) | selection, CRUD, curves, loop splitting, context, Undo | bounded observations, open acceptance |
| [Openwig code](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js) | native command invocation, structural resolver, completion model | JS execution is not adopted; no claim of complete selected-note CRUD |
| [Openwig doctor](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/diagnostics.py) | build validation, sentinel probes | its verified builds differ from ours |
| [Superpowers disclosure](https://github.com/kalifrn6-art/bitwig-superpowers#bitwig-superpowers) | release/build gating, native workflow precedent | private source; no implementation inference |
| [Author's 6.1.1 breakage report](https://www.reddit.com/r/Bitwig/comments/1w7n2zq/quick_update_on_native_midi_tools/) | primary report of obfuscation churn | reported, not independently reproduced |
| [DrivenByMoss cursor paging](https://github.com/git-moss/DrivenByMoss/blob/93167eb82cd1792a6f4a1ce8e0917201aa1cebd0/src/main/java/de/mossgrabers/bitwig/framework/daw/CursorClipImpl.java#L1053-L1057) | mature public-grid comparison | not a private global-selection precedent |

Adopted: one Java host boundary, immutable snapshots, exact build admission,
probe-first capabilities, native row transfers, owner-aware source/occurrence
model. Rejected: assuming private objects resemble public proxy scopes, JS on
document thread, grid reconstruction, full-project unbounded logging, transparent
retry of host writes. Deferred: lossless detached shared-content staging,
collision behavior, all curve writes, active editor/locked-layer completeness,
full selection restoration and automatic recovery of multi-write failures.

Coverage: lifecycle/config/version variance in execution reference; selection,
source mappings and transport in their leaves; all CRUD/property categories in
note reference; eight downstream tasks and fourteen failure/fix cases in probe
reference; processing/session lifecycle in service reference. Each deferred item
has the existing probe/test plan as its next runtime investigation. Further broad
search is lower yield than tracing those exact host operations.

Precision: added a separate skill because private document ownership differs
from public proxy/grid contracts; narrowed the neighboring skill with a route to
this one. Kept source-only lineage here. All six references have explicit router
entries. No new validator machinery.

Trigger check: should trigger for cross-track tool-2 selection, native note
channel transfer, pressure curve CRUD, loop occurrence mapping, internal API
porting and Java note adapter. Should not trigger for hardware pad mappings,
ordinary transport controller scripts, Rust-only DSP or CLAP audio callbacks.
The skill remains specific to Bitwig private document access.



### Source authority and retrieval

Retrieved 2026-10-23. Official manuals describe supported musical behavior; API
inventories delimit public contracts; installed implementation establishes the
examined call path; named probes establish only exercised behavior. Forum posts
are dated reports, questions or workflow examples, never a shipping capability
contract. A search finding is a lead, not a behavioral admission.

| Source | Material evidence | Limit / resulting action |
| --- | --- | --- |
| [6.1 vendor release notes](https://downloads.bitwig.com/6.1/Release-Notes-6.1.html), released 2026-08-26 | Specifies the 6.1 release and documentation transition | Absence of a feature announcement is not proof of global absence; use the installed build and endpoint probes |
| [Vendor MIDI devices](https://www.bitwig.com/userguide/latest/midi/) | Channel Filter/Map, eight-knob MIDI CC, program/bank and song-select behavior | These are MIDI message/control devices, not an admitted UMP/profile receiver |
| [Vendor HW Instrument](https://www.bitwig.com/userguide/latest/hardware/), HW Instrument section | Fixed versus retained channels; MPE conversion of note expressions and bend-range configuration | Verify the selected output route/receiver mode; transport and protocol claims remain separate |
| [Plug-in options](https://www.bitwig.com/userguide/latest/vst_plug-in_handling_and_options/) | Plug-in isolation and optional MPE mode | Not proof of per-note parameter support in an arbitrary patch |
| [3.2.8 official notes](https://downloads.bitwig.com/stable/3.2.8/Release-Notes-3.2.8.html), MIDI import fixes | Channel-preserving CC import and pitch-bend import were introduced historically | Old forum claims that bends are never imported are stale; current preservation still needs readback |
| [5.3.2 vendor notes, indexed official result](https://www.bitwig.com/dl/Bitwig%20Studio/5.3.2/release_notes/) | Reports expanded SMF export of tempo, meter and channel-parameter automation | Direct download URLs were not accessible in this run; current exporter code independently supplies corroboration. Do not equate channel automation with every note expression |
| [Bitwig CLAP announcement](https://www.bitwig.com/stories/clap-the-new-audio-plug-in-standard-201/), 2022-06-15 | Describes per-note/per-voice modulation and standards-oriented extensibility | Format capability is not proof of raw UMP, file import, OS MIDI transport or MIDI-CI implementation |
| [CLAP event definitions](https://github.com/free-audio/clap/blob/main/include/clap/events.h) | Distinct native note/expression, raw MIDI and raw MIDI2 event types | Need installed port/dialect negotiation and observed events before claiming a particular host/plugin path carries MIDI2 |
| [MIDI Association plug-in work](https://midi.org/plugin-formats-open-source-support-for-midi-2-0) | Describes ongoing helper work translating external profile information into plug-in formats | Participation in the working group is not a released Bitwig orchestral-profile endpoint |

The online guide is rolling documentation; the 6.1 release page explicitly directs
readers to release notes plus earlier manuals while its guide is revised. Pin the
installed SDK/build for code decisions rather than treating `/latest/` as a
version-specific private API.

### MIDI support matrix for this exact build

| Surface | Established evidence | What remains unestablished |
| --- | --- | --- |
| Published controller MIDI I/O | SDK 25 short-message/SysEx API, installed byte/data bounds | No documented UMP or profile endpoint in that API; this does not exhaust private/native subsystems |
| Native document notes/expressions | Stored high-resolution note/expression CRUD and independent readback in existing controller evidence | An internal floating-point model is not itself the MIDI2 protocol or an external receiver guarantee |
| MPE | Vendor HW Instrument/plug-in options document expression-to-channel conversion, range and mode settings | This run did not negotiate an external MPE device or prove an arbitrary patch's channel/range behavior |
| Classic SMF | Installed main-reader call trace; actual File reader accepts type 0/1; detached native staging measured | Complete UI/document insertion, resource lifetime, resulting graph targets and persistence are separate gates |
| `.midi2` M2-116 clips | Frozen valid 224-byte vector is rejected by the File reader used on that SMF path | A different importer or raw UMP endpoint has not been admitted; no blanket absence claim |
| CLAP | Vendor describes per-note/per-voice modulation; CLAP headers define native note/expression, raw MIDI and raw MIDI2 events separately | Installed host/plug-in dialect negotiation and raw MIDI2 traffic were not measured |
| OS/hardware UMP | No endpoint evidence obtained in this Linux 6.1 investigation | Windows MIDI Services availability or a device's MIDI2 label cannot establish this host's Linux transport behavior |
| MIDI-CI/profile/property exchange | Local standards and converter semantics are available; no Bitwig orchestral-profile endpoint was admitted | Capability discovery/negotiation, receiver profile behavior and patch-specific realization remain independent |

A useful preflight names the endpoint, event dialect, receiver, channel ownership,
precision and measured conversion losses. A single “supports MIDI2” boolean would
hide exactly the distinctions needed to make a defensible import decision.

### Production-forum leads and their disposition

Search priority was KVR's Bitwig/host sections, VI-CONTROL's orchestral workflows,
ModWiggler and r/Bitwig. No useful ModWiggler result established a 6.1 import
contract. Several KVR full-page fetches returned HTTP 403; their indexed excerpts
were leads only. Accessible Reddit/VI-CONTROL posts were read in context.

| Forum/source and date | Observed report/question | Reconciliation |
| --- | --- | --- |
| [KVR: MIDI 2.0 and Bitiwg](https://www.kvraudio.com/forum/viewtopic.php?t=615457), 2024-10 onward; indexed April 2026 follow-ups | Users ask whether full MIDI2 support exists | Not affirmative evidence. Full page was blocked; do not invent an answer from the title |
| [KVR: Clarity over which DAWs support MIDI 2?](https://www.kvraudio.com/forum/viewtopic.php?t=629382), 2026-04 | Contains the ambiguous description of internal MIDI2 support | Internal floating-point expressions, CLAP dialects, UMP I/O and file support must be investigated separately; full-page fetch blocked |
| [KVR: Bitwig 5 drag/drop from VSTi](https://www.kvraudio.com/forum/viewtopic.php?t=598780), 2023 | Users report import-dialog differences between generated MIDI files | A metadata/dialog hypothesis, not permission to retry UI actions or evidence about native UMP import; full-page fetch blocked |
| [r/Bitwig: multichannel data forced to channel 1](https://www.reddit.com/r/Bitwig/comments/wjvh0j/multichannel_midi_data_being_forced_to_channel_1/), 2022 | Original poster resolves live input remapping through the track channel setting | The documented schema has input-filter range -1..15 and channel-override default 0. This corroborates a routing hazard; it does not prove stored-arranger playback follows the same path |
| [r/Bitwig: MIDI export and pitch bends](https://www.reddit.com/r/Bitwig/comments/1dkbfro/midi_importexport_option_that_preserves_pitch/), 2024, later edited | A claim of absent automation export is subsequently corrected for 5.3.2 | Version/date matters. Verify exporter behavior rather than repeating the superseded complaint |
| [KVR: 3.2.8](https://www.kvraudio.com/forum/viewtopic.php?t=552829), 2020-09 | A user's bend-import observation points to that release's fix | Corroborated by the official 3.2.8 notes, not used as present-day fidelity proof |
| [VI-CONTROL: orchestral/video workflow](https://vi-control.net/community/threads/bitwig-5-workflow-for-orchestral-projects-and-video.151075/page-2), 2024-07 | Describes articulation-label/keyswitch and multichannel workflows | Useful musical-use cases; not MIDI-CI, a native articulation map, or authorization to use that workaround in this importer |
| [r/Bitwig: overlapping MIDI events](https://www.reddit.com/r/Bitwig/comments/16caxzl), 2023 | Reports note/clip overlap difficulties | Distinguish clip overlap, same-key same-channel notes, other pitches, channels and release tails; probe exact cases rather than accepting a blanket inability |

Two additional full-page forum investigations sharpen the project-read question:

- [KVR, notation-to-Kontakt orchestral import](https://www.kvraudio.com/forum/viewtopic.php?t=446450),
  2015-09: a Finale composer's multi-instrument mock-up question establishes a
  longstanding use case, not present-day capability or fidelity.
- [r/Bitwig, Dorico import](https://www.reddit.com/r/Bitwig/comments/1vk2j4l/importing_midi_files/)
  and [the author's follow-up](https://www.reddit.com/r/Bitwig/comments/1vlxoj5/update_about_importing_midi/),
  2026-08: the apparent single-track import was resolved by revealing/ungrouping
  an imported group. Therefore count native children recursively before diagnosing
  lost parts; ungrouping is not an importer workaround admitted here.
- [6.1.1 Windows MIDI report](https://www.reddit.com/r/Bitwig/comments/1wk6tud/bitwig_611_breaks_windows_midi_in_win_11_for_me/):
  the author reports device failures while another user reports success on both
  Windows and Linux. This is a different platform/build and endpoint, not evidence
  of a Linux 6.1 file-reader bug or justification for an internal retry loop.

No new automatic retry or production workaround is admitted from these posts.
The current importer does not route MIDI2 through SMF or real-time injection.

### Installed-code exegesis: MIDI surfaces

1. **Public controller bytes.** `MidiOutPort.sendMidi` accepts a status through
   0xEF and two seven-bit data bytes, then delegates that triple. SysEx is a
   separate byte/string route; `MidiInPort` uses short-message/SysEx callbacks.
   API 25 has no documented UMP or MIDI-CI profile operation. This bounds the
   controller API, not all of Bitwig's C++/plug-in/OS subsystems.
2. **Actual standard-file path.** `Wfd` owns an `nY` reader. `nY` delegates to
   `wuw.v33(File,double)`, which calls `MidiSystem.getMidiFileFormat(File)` and
   `getSequence(File)`. Type 0 is distributed by channel, positive formats by
   track; PPQ and SMPTE divisions take different timing paths. Short messages
   go to `gVN`; tempo/meter and selected metadata have distinct handling. The
   loop does not establish SysEx preservation.
3. **A misleading candidate was rejected.** `Nk` recognizes MThd/MTrk and
   RIFF/RMID and accepts format numbers 0/1/2. Tracing its caller finds the
   `XFF` AXBW importer, not the main SMF path. `CVM` can materialize notes through
   an importer Factory, including release velocity. Its existence does not
   describe the main UI importer's behavior. Never admit a codec by symbol search
   without tracing the caller.
4. **Native staging policy.** In `gVN`, a new same-key/channel attack first closes
   an active note. Note-off processing does not pass its release data through
   to note creation. Positive durations have a 0.01-beat floor before persistent
   rounding. Same-time points in a controller list replace the prior value.
   These statements motivated, and were checked by, the named detached probes
   below. They are not universal limits of native note CRUD or expression editing.
5. **SMF export is another adaptation.** `WPw` creates a PPQ-960 Java Sequence and
   writes type 1. `ERB` converts attack/release to seven-bit ShortMessages and
   emits poly-pressure from native curves; related helpers export tempo, meter
   and channel controls. This is not a lossless serialization of every native
   expression/operator or a MIDI2 clip writer.
6. **Routing is separate.** `master.RYK` declares `filter_note_channel` default
   -1 and `override_note_channel` default 0, ranges -1..15. Layer/control schema
   `device.mkN` has its own filter/override defaults. A valid stored note channel
   does not prove the message delivered to a player. Each owner/path must be
   measured; do not treat a live-input forum fix as an arranger-import fix.

An exhaustive byte-string search found no `SMF2CLIP`, `midi2` or `MIDI-CI` literal
in the inspected Java JARs. This helped locate questions; it is **not** proof of
absence from native binaries, plug-in bridges or external extensions. The
positive file-reader trace and actual probes are stronger evidence.

### Named reader/staging probe and measured results

| Question | Quantitative observation | Admission boundary |
| --- | --- | --- |
| SMF 0 and 1 | Each 79-byte vector: PPQ 480, one track, two attacks/two releases, stored tick extent 480; channels 0 and 15; one CC, bend, poly-pressure, channel pressure, SysEx, tempo and meter each | Reader acceptance only, not document preservation |
| SMF 2 | Metadata recognizes type 2; Sequence read rejects it | Format recognition is not a successful import |
| M2-116 `.midi2` | Valid 224-byte one-note clip rejected as unsupported by the File reader | No direct UMP-clip admission through this SMF path |
| Release | Source Note Off velocity 17/127; detached result 100/127, matching attack velocity | Main staging helper does not preserve the requested release value in this vector |
| Same-key/channel overlap | Two attacks remain two notes, but output intervals are [0,0.5) and [0.5,1), not the original intended overlapping intervals | Counts alone conceal lifetime changes; MIDI1 pairing/state ambiguity is not repaired silently |
| Exactly zero duration | Requested zero-beat lifetime becomes 0.25 beat | A distinct zero-duration branch, not the positive-duration floor |
| Independent channels | Simultaneous key 60 on channels 0 and 15 remains two notes with 1 and 1.5 beat durations; poly pressure only on channel 0 | Same-key overlap loss is scoped to one channel; no blanket overlap prohibition |
| Channel controls | CC1 on channels 0/15 has separate native keys 1/3841 and values 20/127, 100/127; channel pressure has separate keys 0/15 | Detached staging preserves these channel identities; final automation targets/delivery still require admission |
| Pitch bend | Source 14-bit values 0/8192/16383 yield staged -1/0/+1 | Confirms asymmetric normalization endpoints, not arbitrary per-note pitch fidelity |
| Very short duration | Requested 0.001 beat becomes 0.010000228881835938 beat | The staging floor plus native 2^20 ticks/beat rounding is visible; not a native note-model minimum |
| Equal-time CC | Two source controller events become one staging point | Do not claim ordering survives from an event count alone |
| Active project | Complete before/after revisions identical | Probe did not mutate the active fixture |

The [expanded probe](../../../controller/validation/import-bwv248/midi-reader/expanded-file-reader-and-staging.json)
retains these additional positive channel/pressure/bend and zero-duration cases.
The same-time CC overwrite applies within a channel/controller key; native `gVN`
keys that map as `(channel << 8) | controller`. Bend and channel pressure maps
use channel keys. Poly pressure is added only while the keyed note is active.
Program-change handling in this helper supplies an initial GM-name candidate;
it is not an admitted exact sample-library recipe/configuration mechanism.

The actual file/project insertion path, resource lifetime, native Undo grouping,
MPE round trip, raw CLAP MIDI2 dialect negotiation, OS/device UMP transport and
MIDI-CI/profile negotiation remain separate unadmitted questions. Do not turn
these negative SMF results into a false claim that Bitwig lacks note expression,
MPE, CLAP or every form of MIDI2-related functionality. Do not call its internal
model MIDI2 protocol merely because it has high-resolution per-note values.

### Native project/audit lessons

The retained [implementation experiment](../../../controller/validation/import-bwv248/README.md)
distinguishes source-bundle checks, within-session preservation and restart
failures. Source and exact observations live there, not in runtime prose.

All remaining import construction, receiver matching, stored automation, project
lifecycle and audible/performance gates remain in controller/IMPORT.md. A named
probe, a parser success or a structural audit never closes unrelated gates.

## Construction continuation — 2026-10-23

The native importer classes `ApiProject`/`ApiFactory` looked attractive because
Bitwig supplies track, note, device and automation factories there. The installed
`GsR.TUp(ClipFileImporter,File)` call path creates a detached `DXq.rwW()`, invokes
the importer and disposes the wrapper in `finally`. This is the relevant source
lifecycle; it is not an arbitrary document-thread controller entrypoint.

**Rejected promotion:** `ApiRelatedObject.wrap` explicitly asserts
`!q3G.v33().isEventDispatchThread()`. The actual controller document task reports
both `isDocumentThread()==true` and `isEventDispatchThread()==true`. The earlier
factory probe mutated a disposable fixture successfully because host assertions
were disabled. That success does not override the source execution contract.
The wrapper implementation is retained as rejected development source under
[construction evidence](../../../controller/validation/import-construction/), not
promoted to production or offered as the current construction command.

Experiments before that rejection established useful musical questions:

- Two native factory tracks held eight notes with independent release values;
  positive duration 0.001 became 0.0010004043579101562 beat, without the SMF
  staging 0.01-beat floor. Same-key overlap in one channel shortened the earlier
  note; a separate channel preserved both lifetimes. Allocation must not undo the
  converter's isolation when combining notes into receiver tracks.
- Factory CC helper points were stored at arranger times 8 and 10, normalized
  values 20/127 and approximately 100/127, with channel-15/CC1 destinations. Its
  default interpolation was LINEAR: an event stream requiring held values needs
  explicit HOLD, not a convenient default ramp.
- Full detached automation enumeration failed at `xFO.display_settings`.
  `WDv` property 1212 delegates to `rWq.fvB`, which requires the connected
  document's `Msg.TUp(this)`/`Krq` context. This is an owner-context failure, not
  permission to omit the field and claim a complete detached observation.
- Read-only derived properties must use the canonical `RR`/attribute reader.
  Mutable `Nln(name)` lookup returned null for `midi_parameter_container`; the
  corrected reader identified `TSI` → `HQO` channel 15, CHANNEL_CONTROLLER,
  type_parameter 1, and linked that exact reference to the automation lane.
  Device/automation graphs alone did not expose that MIDI parameter owner.
- One explicit native Application Undo restored all nine prior tracks, every
  track payload, all 900 notes and the original revision. Only `is_modified`
  remained true. Redo restored both sentinel tracks' clip data and payloads.
  These tests characterize the bounded fixture, not every graph mutation family.

The replacement experiment follows Openwig's documented native command pattern
and the exact installed `ApplicationProxy.doCreateInstrumentTrack` owner path.
`I1I.CDr` is arranger-clip command 7350; the existing note command is 7349. Group
and track factories stay on the live document thread without importer wrappers.
Stored envelopes use the native `TSI` parameter, `oJn` reference, `I3W` clip and
`nCh` points, as in the examined native staging implementation; no SMF or MIDI
message conversion is inserted. `CBo.v33` is HOLD; `CBo.TUp` is LINEAR.

The first raw-command probe stopped at a signature mismatch after one track/four
notes. The automation-lane overload takes `core.iface.bf`, not the concrete
`core.master.oJn` reference type. A separate fresh read recorded the partial
state; one agent-invoked Undo restored the original native revision and all track
payloads. The corrected signature is resolved before the mutation group begins.
This is an explicit code correction plus independent recovery, not an internal
retry or admitted instability workaround. Quantitative reports and failed traces
are retained; subsequent admission belongs to the completed probe/readback.


The corrected raw-command probe creates a group/two tracks and eight notes using
commands 7350/7349 and native owner operations, retaining the intended isolated
note durations and release values. Direct `I3W`/`nCh` envelopes store channel-15
CC1 at beats 8/9/10, including `0x12345678 / 0xFFFFFFFF` exactly as a native double,
with HOLD interpolation. No seven-bit encoding, live MIDI injection or SMF
translation occurs. A separate default-value point at zero is also visible;
entry/exit policy must account for it, not discard it from the oracle.

### Native saving and destination ownership

The actual native Save As path is `Wfd.v33(aBq,File,boolean,boolean,Tl3)`.
It requires the event thread, resolves the live wrapper through `FHv(DXq)`, and
queues the host's `RQA` save task. `RQA` copies states/resources, serializes through
`document.file.hBi` and invokes `taskCompleted()` or `v33(Exception)`; our callback
ran on main. The controller must return pending and expose an operation receipt,
not wait on the document thread or equate request acceptance with save completion.
The prototype's receipt map is bounded and loader-scoped; reload makes an old
operation unknown and demands observation/reconciliation, not replay.

A rejected save hypothesis matters: `DXq.v33(boolean,Tl3)` activates the audio
engine; `DXq.v33(Tl3)` deactivates it. Neither is a save method. Following the
actual app command was necessary to find the right lifecycle.

First Save As succeeded but revealed native destination canonicalization:
a request for `/tmp/import-native-pilot/NativeSentinel.bwproject` in an unregistered
parent became `.../NativeSentinel/NativeSentinel.bwproject`. `RQA.run()` checks
`document.file.CRe.YLV(parent)`; if absent, it creates a project subdirectory from
the stem. The owning native `CRe.jaQ(directory)` creates `.bitwig-project` and is
the proper directory-initialization primitive. Do not guess folder/name identity,
silently rewrite SPEC paths or publish a successful manifest at a requested path
that the host did not use. Require exact native file readback after the callback.
A destination nested inside another project's root requires an explicit different
scope; the current probe refuses it before saving. Folder initialization is a
filesystem side effect and is outside musical Undo.


### Zero-scene correction and explicit factory insertion — 2026-10-23

The fresh named-empty constructor (`Wfd.v33(name,0,0,wbN,xAa)`) successfully
created a separate one-master-track, zero-note project. Native open/select
succeeded for the saved sentinel, but did not connect its engine; a separate
`DXq.YLV(Tl3)` activation callback plus `WAR.CONNECTED` established that state.
An earlier cold start crashed during the initial empty-project transition;
its harvested engine report states Object.cpp:245 reference-count failure.
The older Oct-21 yabridge EOF report was stale and is not evidence for this crash.
Opening the same saved file in an already running host and explicitly activating
it succeeded. This does not establish universal cold-start reliability or justify
an automatic restart/import retry. Preserve both outcomes.

The zero-scene experiment falsified the earlier envelope placement assumption.
`lane.TcB()` is `zsB`, the **launcher slots**, and `qKA(0)` selected a launcher
slot in the older fixture. It fails when there are no scenes. Activation did not
change that result. Thus earlier fractional HOLD point storage was real, but it
was not proof of arranger placement. Exact source `KBH`/`owf` identifies
`lane.XQF()` as the native `CVR` arranger timeline; inherited `Kmg.TUp(fza)`
attaches the envelope there. The corrected empty-project probe independently
reads four tracks, eight notes and both arranger envelopes (369 owned objects).
The failed partial write was separately undone to the exact 157-object/zero-note
baseline; no internal retry or silent fallback was added.

The native `DeviceChainInsertionPoint`/`Elr` asynchronous path follows installed
SDK `tLn.doInsert`, with explicit native chain ownership, accepted-result checking,
callback receipts and independent `zfx.v33()` device identity. Instrument Layer
and Selector each inserted exactly once. However UUID insertion honors user
saved defaults: the installed Selector default contained Kontakt 7. This is
unacceptable as a deterministic factory scaffold. The probe now uses the host's
`Wfd.nsT()` factory device-settings root and explicit `<UUID>/Default.bwpreset`
through the same native insertion pipeline. Do not edit user defaults, copy
observed receiver contents into a plan, or equate built-in UUID identity with an
empty device. Factory insertion remains under test until fresh native readback.


### Committed automation and source coordinates — 2026-10-23

Further readback falsified two apparently successful construction observations.
The raw graph's source points alone cannot establish performed placement. The
`I3W` clip was at arranger beat 8 with source interval [0,4), while the earlier
probe wrote points at source beats 8/9/10. Those points were outside the played
source interval. Correct source positions are 0/1/2, mapping to arranger 8/9/10.
`ProjectAdapter61` now exposes native `T1L/GaO/dD_/Fgm/YeF` and loop data on each
observed automation clip, just as for main note clips. Never infer source position
from an automation point's `time` alone.

Also, `Kmg.TUp(fza)` delegates to `QNO.v33(TJk)`, whose list is preview events.
A recorder-manager `LHs()` does **not** convert those into committed events.
The committed insertion is `Kmg.v33(fza)` → `pcS.jaQ(oHd)` → indexed events.
The corrected probe must show CVR.events contains the clip and preview_events is
empty, then survive native save/reopen. Earlier numeric point-storage claims
remain true but do not admit arranger playback or persistence. Do not hide preview
state in an audit or call a write successful merely because a full graph contains
matching values. The extra LHs follows the existing native edit flush convention;
it is not a retry or substitute for the correct timeline operation.

Selector Index is discrete: its parameter.qeL() is au, not floating bD. The first
control-envelope probe stopped after adding a lane at the wrong factory signature;
one explicit Undo restored the measured prior state. A parameter reference's
`heu()` returns the automation-compatible bD. Its `TUp(time,value,CBo,curvature)`
creates the appropriate native point type: Selector yields Oss integer events,
while MIDI CC value yields nCh floating points. Oss has no interpolation_mode;
that is discrete storage, not missing floating curve data. Retain this distinction
in native query/audit output. No float-to-int guessed conversion is admitted.


### Final native construction readback and production lifecycle — 2026-10-23

Unit distinction: `bD.TUp(time,value,CBo,curvature)` is the normalized-value point
factory (its base error explicitly says createEventWithNormalizedValue). For the
MIDI CC knob's am range [0,127], normalized .2, 0x12345678/0xFFFFFFFF and .8 store
physical 25.4, approximately 9.03111109534 and 101.6. The HQO track CC type uses
[0,1]. These are different encodings of the requested normalized values. Read the
actual value_type min/max/domain; do not apply one universal scaling convention.
Direct Nln(value) writes physical values: the static P2=.75 probe is .75 native
units and is **not** evidence of 75% expression. The source am.TUp(double) performs
the normalized-to-physical conversion. Stored precision is not receiver/MIDI wire
or audible precision.

Production wire v6 promotes native new/activate/save/close/open/status through the
existing bounded Java session, with typed Rust requests/results and independent
project measurements. Mutations take a caller-retained operation ID and current
controller_generation/revision. The shared editor's target observer invalidates
generation even for A→B→A; a live test returned to the same persisted project UUID
and refused its old controller generation with the fresh current revision. The
native named-empty constructor, factory file open, separate activation, exact
save and normal close paths were exercised through production CLI. Duplicate
operation IDs refused before additional construction and left the full snapshot
unchanged. Pending callbacks were observed as pending and later queried by status;
no native mutation replay occurred. Receipts are bounded to 16 per adapter lifetime
and exhaustion is a clear fail-stop condition. A service/adapter restart is not a
retry policy; an unknown receipt demands current project/file reconciliation.


### Complete note batches and the source-generation counterexample — 2026-10-23

The native content batch reuses the existing complete-note adapter after group,
track and arranger-clip creation. Rust preflight rejects same-channel/key overlap,
non-native timing, incomplete expression axes and invalid hierarchy before any
native command. The first owner-identity probe failed after two groups/one empty
track because UMj.qKA() returns the gEi note grid, not the content source. One
explicit Undo restored the prior revision and every prior track payload. Installed
UMj.FSQ() returns Dxc, the clip-content source and proper owner of the native
clip_content_timeline_id property. The corrected probe built all three source-
authored notes, including release velocity and pitch/pressure curves. The Rust
planned-content oracle compared all requested hierarchy, clip geometry, note
operators/scalars and expression fields with zero differences. This is distinct
from the raw all-graph preservation audit and from receiver/playback verification.

One complete batch Undo restored four tracks/eight pre-existing notes and every
prior native track payload exactly. Redo restored the music but changed the two
clip-content UUIDs. The first content audit failed on those two fields, with no
musical differences; that report is retained unchanged. Installed Gvh.EVR() returns
a random UUID, and Gvh.TUp(iA) explicitly replaces XQF with EVR() during load.
Gvh.gJp() can also regenerate it. This falsifies treating that UUID as a persisted
clip key. The construction receipt retains it as a **source generation at creation**;
raw project observations retain the actual current value.


### Production history observation boundary — 2026-10-23

Wire v7's production project construction built the source-authored three-note
batch in an independently created empty project (five tracks, three notes, 357
owned objects). Its Rust content oracle passed. Native document history uses the
same eel/NMp calls as ApplicationProxy.doUndo/doRedo; the Fit/qeL BooleanValue
attributes preflight availability, without requiring an active musical editor.
The first immediate Undo result restored one master/zero notes and the original
serialized master payload, but its raw channel_index was still 4 (formerly 0) and
prerequisite_tracks retained one added reference. This strict comparison failed
and was retained. A separate fresh read and operation status then restored the
**entire** original track graph and native revision exactly (is_modified remains
true). No second Undo, sleep, internal polling, normalization or mutation replay
was used. The subsequent Redo restored the complete three-note content oracle.

Thus a same-task history read is not a settled whole-graph preservation claim.
Keep the immediate result, then query status and independently measure/audit the
next stage. If the fresh state still differs, stop and reconcile. The observed
schema declarations are core.master.RYK.NKN channel_index and I1I.pSK
prerequisite_tracks; do not guess the flags' meaning from their numbers. The
specific deferred update mechanism is not yet admitted; the measured boundary is
sufficient to require separate status/readback in the agent workflow. This is an
agent-controlled observation sequence, not an internal instability workaround.

### Installed plug-in registry and native Kontakt insertion — 2026-10-23

The installed SDK's `tLn.insertVST3Device` resolves the class ID through metadata
reader Q1, takes metadata DXq.ij_() (the virtual `bf` plug-in file), and inserts
`Elr(bf)` through DeviceChainInsertionPoint with the normal copy flag. The private
probe used that same path, with an explicitly empty owned chain, no serialized
plug-in state, preset injection or MIDI input. Metadata master L2D/Gax/DXq are the
qualified registry classes; unrelated root-package obfuscated names are not
interchangeable.

The first name-filter probe used Gax.eAi(), which is category, and falsely
returned zero Kontakt matches. A bounded full native registry read returned 86
entries without changing the project. RR `device_name` / Gax.cfn() is the actual
name; vyF() is manufacturer. The corrected filter uses the named RR property.
Keep the false-negative evidence: a wrong metadata accessor is not proof that a
receiver is absent.

The registry identifies Kontakt 7, version 7.10.1, Native Instruments, class ID
5653544E694B376B6F6E74616B742037, opaque insertion ID
`vst3-5653544E694B376B6F6E74616B742037`, at
the installed Kontakt 7 VST3. Its nativePreferred flag describes
architecture preference, not successful loading. Insertion callback/readback
produced one WwV owning core.master.device.hFO with the same `vst_device_id` and
`device_version`. On an inactive project it was NOT_LOADED. A separate fenced
production project activation completed CONNECTED; separate observations showed
LOADING then LOADED, note input/audio output true, error_message empty. The
three source-authored notes remained unchanged. No internal polling/retry was
introduced; these were explicit agent-driven measurement boundaries.

This loaded instance reports supports_mpe=false, force_mpe=false,
plugin_wants_mpe=false and supports_vst3_note_expressions=false. These are observed
instance flags, not a general MIDI 2.0 conformance conclusion. No NKI was loaded
or private receiver setup/audio verified by this experiment. The disposable
project was saved with Bitwig's native writer before adapter restart.

Kontakt exposes roughly two thousand generic parameters and expands a complete
raw project observation substantially. Keep one full observation per bounded
response; receipt metadata must not duplicate the owned parameter graph. Measure
or explicitly redesign representation budgets for an orchestral graph; do not
silently truncate parameters or claim unobserved state is preserved.

Stored control interpolation uses installed CBo.v33=HOLD, CBo.TUp=LINEAR.
Discrete Selector events are Oss; CC events are nCh. For static normalized values,
the same native bD.TUp(time,normalized,CBo,curvature) factory produces a typed
event whose physical `value` can be assigned to the parameter. Do not call
am.TUp(double) on a discrete enum type, or assign a normalized decimal directly
to a physical 0..127 CC knob. Production admission still requires independent
readback of static values, interpolation, source coordinates and committed events.

### Production device controls, audit correction and reload handles — 2026-10-23

Wire 8's production interface created the empty Layer, two filtered slots (input
0 and 15, both remapped to output 0), a nested Manual Selector with two slots,
and a MIDI CC device inside the other Layer slot. The normalized static CC value
0.75 read back as physical 95.25. Three source-coordinate points read back as
25.400000000000002, 9.031111095340714 and 101.60000000000001, at source beats
0/1/2 inside the arranger clip at 8 with duration 4. Interpolation was explicitly
LINEAR/LINEAR/HOLD, curvature/spread zero. The independent Rust oracle passed
identity, label, CC11 assignment, physical values, one committed/no preview clip,
exact clip mapping and complete stored source points.

The first Selector oracle failed because its expectation serialized integral
values as JSON floats (0.0/1.0) while native Oss values are JSON integers (0/1).
The failed report remains intact. The corrected expected representation derives
an exact bounded integer slot from the authored normalized value; it neither
changes the native project nor copies observed values. The standalone check of
the same observation passes. A malformed/missing slot or changed value still
fails. Discrete Selector automation admits HOLD only; continuous CC interpolation
is explicit. These checks do not establish private NKI setup or audible behavior.

Reload of the native fixture retained the top-level Layer UUID, but regenerated
nested slot and nested Selector UUIDs. A stale Selector command was refused before
mutation. The slot's `preset_UUID` read back the former UUID, while its current
UUID differed. Do not treat every device/slot UUID as a persisted import identity.
The native sYf base owns preset_UUID; zfx is a hRv/ysX descendant. The exact
nested-preset deserialization remapping path remains to be established. Until
then, re-observe and explicitly resolve the owned container path/recipe after
reload; nested UUIDs are current mutation handles. No blind UUID substitution,
automatic retry or stripping of strict raw-graph differences is admitted.

During development, a newly built CLI sent an interpolation field to an older
running service with the same provisional wire version. That service closed the
connection before adapter dispatch; explicit status reported unknown operation.
Restarting the service invalidates the fail-stop adapter transport; it does not
automatically reconnect. The agent restarted the disposable saved fixture after
reconciliation, read it afresh, and issued a distinct operation only after
checking ownership and note preservation. Deploy matching CLI/service/adapter
builds together. A generic EOF is weaker diagnostics than a schema/version error;
never interpret it as proof of either successful mutation or safe replay.

### Loaded is not healthy: DXGI crash and pending save — 2026-10-23

The active production Kontakt insertion later set
is_hung_while_processing_audio=true. Bitwig's native save remained pending with
is_being_saved=false and no target file. A later mutation was refused because the
save callback had not completed. Development-only Rust-facade display capture
showed Wine's crash debugger and Bitwig's question about proceeding without a
plug-in's saved state. No Yes/skip-state action was accepted, no successful save
or completion manifest was claimed, and raw snapshots/logs were retained.

Relevant community/primary reports checked:

- [yabridge #356](https://github.com/robbert-vdh/yabridge/issues/356) reports Kontakt
  crashes after installing DXVK, with a different page-fault stack. It supports
  investigating graphics configuration, not assuming the same cause or remedy.
- [yabridge #398](https://github.com/robbert-vdh/yabridge/issues/398) reports X11
  embedding failure in Bitwig on a different host/Wine version. Our trace is a
  division, so that window error is not an admitted diagnosis here.
- [yabridge #502](https://github.com/robbert-vdh/yabridge/issues/502) reports a
  beginEdit/getState deadlock in Bitwig with other vendors. We did not reproduce
  that concurrent GUI-edit boundary; it does not justify a retry workaround.
- [yabridge #236](https://github.com/robbert-vdh/yabridge/issues/236) concerns
  Kontakt's repeated parameter-name invalidations and slow loading of large
  instruments. No NKI was loaded here, so that sample-load case is not this proof.

Production save preflight now rejects observed hung, loading, saving or errored
plug-ins before dispatch, with player/state context. This guard has a Java
regression test. Native project summary exposes plug-in identity/version/state,
hung/saving flags and MPE/note-expression flags. A previously healthy plug-in can
still fail during save: pending status is not completion, and the agent must
reconcile without replay. Development GUI capture/click/key operations remain
outside the durable score-realization acceptance workflow.


### Native unload-before-save and cancelled close semantics — 2026-10-23

A successful active-player save read is_modified=false. Native close then unloads
the player, closes its editor and captures another plug-in-state filename; the
project becomes modified and asks whether to save. The independent raw comparison
retains 2,181 differences: mostly shared-memory IDs/readiness, plus the actual
plug-in window/state filename change and modified flag. No note fields changed.
The development agent selected Cancel, never No/discard or skip state.

Installed app.Wfd.v33(DXq,Tl3) resolves the aBq wrapper and enters normal close.
It sets document closing, deactivates the engine and may ask the save question.
The cancel branch calls aBq.heu(): it clears document closing and the wrapper flag,
then clears completion runnables. Thus the supplied close callback can remain
unresolved after cancellation. The read-only native lifecycle-state probe measured
closing=false, closeDialogPending=false, source still present and modified=true.
Production status now records whether closing began and reconciles that native
cancellation as a failed operation; no blind replay or false closed receipt.

### Native visible device names — 2026-10-23

ApiDevice.setDisplayName delegates through modify to native ysX.XOR(String).
The owner-thread probe invoked that native setter inside an independent Undo group,
without invoking ApiDevice wrappers. Readback of the Layer's user_device_name,
display_title and device_name all became `ART | pilot.part.b01 | M2-123`;
observedBuiltinId stayed 5024be2e-65d6-4d40-bbfe-8b2ea993c445. Therefore mutable
device_name is a display projection, not a reliable factory/plugin identity.
Use actual built-in identity or plug-in contents ID/version, plus owned bindings.


### VST3 precision and controller interfaces — 2026-10-23

Primary source: Steinberg [NoteOnEvent](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1NoteOnEvent.html)
uses floating velocity in [0,1] and floating cent tuning. A MIDI 1.0 controller
route alone therefore does not justify reducing native note velocity to seven
bits. Actual Bitwig/yabridge/Kontakt delivery remains an empirical boundary.
[IMidiMapping](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/classSteinberg_1_1Vst_1_1IMidiMapping.html)
(released 3.0.1, optional) maps input bus/channel/controller to a parameter ID;
the host sends parameter changes. Do not describe
[LegacyMIDICCOutEvent](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1LegacyMIDICCOutEvent.html)
as the host-to-plug-in input contract: its direction is output.
[IMidiMapping2](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical%2BDocumentation/Change%2BHistory/3.8.0/IMidiMapping2.html)
is a newer optional MIDI1/MIDI2 mapping interface. Its existence in current SDK
documentation establishes neither implementation in installed Bitwig 6.1 nor
support in Kontakt 7.10.1. Keep interface availability, native stored precision,
actual receiver delivery and MIDI2 protocol conformance as separate claims.

### Stored tempo/meter trajectories and the overload counterexample — 2026-10-23

Source order: retained Openwig owner/command/readback pattern; installed
`ApiTransport.setTimeSignature` and `ApiValueAtom.setAutomation`, then native
`kkr`, `qNN`, `Tl3`, `RYK`, `am` and `dwv`. The API wrapper itself is not called
on the owner thread. Constant setters `tempo.d_(double)` and meter `NKN(int)`
read back exactly 72.000000288 BPM and 3/8 with tick rate 24; native Undo restored
the static values. Do not confuse the native metrical tick rate with note time's
2^20 ticks per quarter-note beat.

The first timeline probe chose `TUp(bXo)` by reflection. That exact overload is
`Tl3.TUp(bXo):void`, whereas the source's lane-returning calls use the declared
`bf`/`kJk` interfaces. A null return is not a usable lane. The probe failed at
`XQF`, retained the failure, independently read the resulting state and issued
one explicit Undo. Static values returned, but owned-object totals remained
164 versus baseline 157, so complete raw-graph restoration was not claimed.
The corrected experiment used a fresh disposable project; it did not replay the
failed operation or retry around instability.

The admitted lane creation pattern is the same as stored device controls:
`oJn.v33(bXo)` parameter reference → owner `YLV(bf)` → arranger `XQF().Iks()`.
Transport-global points belong to the outside-clip timeline. Clear/replacement is
admitted only after proving no pre-existing global automation. `xyU()` follows
the native automation setup path. The tempo type's physical-value point factory
`TUp(time,bpm,spread)` preserves exact physical BPM without normalized round-trip;
set interpolation explicitly. Native meter points use `dwv.v33(time,RYK.v33(...))`.
The native simple-meter encoding must be checked against the numeric numerator,
denominator, tick rate and all subnumerators; it is not a MIDI bit layout.

### Selecting an already-open native project — 2026-10-23

Installed `Wfd.v33(String,...Set...,xAa)` first finds an existing wrapper with
`zeN(File)`. Its successful-existing-project branch selects that wrapper through
`oSF.v33(bNT)`. `Wfd.aYK()` is the native main project window getter. A named,
fenced disposable probe therefore looked up exactly one target UUID in the
native open-project inventory and called `app.aYK().v33(bNT)` with that wrapper.
Independent application readback switched from the saved GlobalsTimeline project
to the unsaved ApplicationAuthorityPilot UUID, with no file load or content edit.
Do not use `Wfd.TUp(aBq)` as a guessed selection method: its source updates recent
application context, not this complete project-tab selection path.

### Quarter-note clock relation caught during global planning — 2026-10-23

Local normative authority: archived M2-104-UM v1.1.2 §7.5.4, Figure 71, printed
pp.63–64 (`research/standards/midi/...Protocol_Specification.txt` lines 2926–2970).
Flex Set Time Signature's third payload byte is the number of 1/32 notes in 24
MIDI Clocks. The converter had written 24, confusing the clock count with that
relation. Score and clip timing use quarter-note beats; the generated relation
must be 8. This is a converter fix, not importer score reinterpretation or a DAW
workaround. Independent vectors are 4/4 `D0100001 04020800 00000000 00000000`
and 7/8 `D0100001 07030800 00000000 00000000`. Do not equate this relation with
Bitwig's separate native meter tick-rate field.


### Receiver UI setup probe and input delivery — 2026-10-23

A separate `ReceiverRangeProbe` native project contains the independently checked
three-note/five-track batch and one exact Kontakt 7.10.1 VST3 on its independent
track. Activation and insertion were separately measured. Native state reached
LOADED with no hang/error; this alone did not establish an NKI. A development
capture exposed the earlier crash-recovery dialog despite usable native project
authority. Dismissing recovery restored no projects. Host SIGTERM is explicitly a
termination request, not a proven graceful application quit.

The development file chooser rejected the first NKI request because simulated
character typing omitted spaces. Restoring xdotool's documented default 12 ms
cadence did not fix the observed whitespace loss. No automatic retry was added.
A separately invoked clipboard paste preserved the visible path; submission was
a separate gesture after field inspection. `host type|paste` remain bounded,
development-only helpers and never enter the score-realization runtime workflow.
The unsuccessful and successful captures are retained in `development-ui/`.

### Render source entry points — investigation in progress

After the retained Openwig command/owner/readback precedent, installed vendor
`com.bitwig.flt.app.testing.tests.RenderProjectsTests.startRendering` gives an
actual non-GUI render pattern: current DXq → Fv().vZB() master track → LUl request
with explicit beat range/output paths → TQb settings → TUp(settings), asynchronous
completion. Its concrete test XWT subclass handles completion. The vendor test's
setup sleeps are test implementation details, not admitted waits/retries for this
adapter. Production must use observed engine/player readiness and separate status.
No render capability is admitted by discovering these classes.


### Native callback render and decoded-sample counterfactual — 2026-10-23

Installed `by1` owns the document queue (`DXq.R3h()`), its `TQb` settings,
queued `dOp` requests and active `wwO`. `pfG(master,start,end,context,base,true)`
adds `.wav` itself. Native `wwO` implements render success/progress/error through
`com.bitwig.flt.document.core.iface.XWT`; queue start also accepts engine `Tl3`.
Use this existing concrete renderer, not a synthesized bytecode subclass. All
creation/dispatch and observed callbacks were on `main`. The queue resets bit
depth from `document.e2N()` (a `jnu`, not sample rate). Explicit settings were
POST_FADER, 48 kHz, preferred INT24, WAV only, both dither options false.
`by1` catches/logs IOException during renderer start; this is an unresolved
production failure-path obligation, not permission to retry. Require separate
status and both callback channels; this remains a development probe.


### Complete parameter-graph transfer — wire12 live admission

Deployment/power cycling did not establish loaded classes; use actual negotiated
production observations. Public ControllerHost.restart is documented as restarting
that controller (API7+), not a guarantee of reloading extension JAR classes.
Development wheel gestures remain outside durable score-realization QA.


At four active empty Kontakt instances, complete native readback exceeded the
old socket limit:19880500decoded bytes crossed as1163962encoded bytes, with
42603owned objects,16tracks and14sentinel notes. Six instances subsequently
reached63451objects with all six state=LOADED, hung=false, error empty. The sixth
native insertion callback preceded parameter expansion: its first completed
status observed LOADING and53124objects, then a separate fresh readiness
observation saw63451and LOADED. This is normal independently measured lifecycle
state, not permission to advance on callback alone or an internal retry policy.
Packed receipt output retains typed requests and complete results while stdout
reports status, source authority, project totals, plug-in health and action audit.
The offline receipt reader validates request/result correlation and never dispatches.


The inactive construction path was then admitted separately: after native
NOT_CONNECTED, an exact ninth Kontakt wrapper completed identity/ownership audit
without engine activation; all nine remained NOT_LOADED with no hung/error flag.
A native save/close/open checkpoint preserved14notes/16tracks and the exact
104774-byte saved file.
Reopening inactive discarded the transient generic-parameter expansion (1781
owned objects); this is observed host cache lifetime, not data truncation.
Players10–14 were each separately inserted and checked while NOT_CONNECTED.
The complete fourteen-wrapper scaffold had2266owned objects. Source-authored
content checks passed before activation after the checkpoint. Native fourteen-
player save completed at119424bytes.

Activation completed before all players finished loading; the first status
explicitly retained LOADING/QUEUED_FOR_LOADING states. A subsequent complete
capture observed all14 LOADED, no hung/error flags,146844objects,16tracks and
14notes. Java→service68917279bytes crossed as3959775bytes; service→CLI68917223
crossed as5291512bytes. The retained complete observation is68917188decoded bytes,
5282669encoded bytes.
The authored14-note oracle passed against this exact capture. Native read transfer
began05:26:35UTC, native response arrived05:27:03, forwarded05:27:11; owner-read
performance still needs improvement before larger configured scaffolds. Frontend
process RSS4776236KiB, peak4939152KiB, swap0; these exclude plug-in child processes.
Fourteen **empty** players are established, not an audible BWV orchestra.

Initial note-monitor source investigation: public AbstractChannelProxy subscribes
to jLF.dDz()/IoD and forwards kOx(on,key,floatVelocity,extraBoolean). Public
playingNotes further quantizes velocity to127steps and merges state by key.
kOx exposes no MIDI channel or CC payload. This surface alone cannot establish
channel remapping, overlapping same-key identity or CC routing. Native remote
page candidates ZkM→NAf()/YuS→VYW.iDx() and fpG slot mapping are only source leads;
no remote-page mutation has been admitted from signatures alone.


### Method metadata cache: measured before admission

A new read-only `probe construction project-measure` reports owner-thread read
nanoseconds, complete native raw JSON byte count/SHA, revision and totals. Its
explicit read deadline defaults45seconds (bounded1–120); expiration names the
operation and result offset and authorizes no replay. The first14-player native
read measured24970218178ns with146844objects and68917125raw bytes. Source
inspection found repeated superclass method searches and NoSuchMethodException
creation for each native property access.

The cache retains Method metadata by exact runtime class/name/parameter signature
using ClassValue plus concurrent per-class maps. Zero-argument calls reuse empty
argument arrays. It retains neither document owners nor returned values, keeps
original superclass search/access rules, and unwraps native exception causes as
before. [Java25 ClassValue](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/ClassValue.html)
provides class-associated atomic publication; [getDeclaredMethod](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/Class.html#getDeclaredMethod(java.lang.String,java.lang.Class...))
remains the lookup authority. Tests cover inherited/private methods, separate
owners/live values, shadowed classes, overloads, absent methods and native causes.

A separately built/reloaded harness measured3283478739ns (7.6048× faster) against
matching byte count, project identity, revision and totals. Java22tests passed, including
both cache tests. This admits the cache itself; the currently running production
extension still predates it and needs a measured reload before claiming production
latency improvement. No graph projection, discarded field or broadened timeout
was used to manufacture the improvement.


A separate NativeRemoteProbe was created inactive and its three-note batch and
native MIDI CC factory insertion independently passed. Native track `id` is the
managed snapshot ID, while `properties.UUID` is the persisted operation handle;
summary output must expose both clearly. The actual6.1 MIDI CC factory defaults
are P1→CC1/MW, P2→CC2/BC, P3→CC11/XP, P4–P8→-1 (disabled), MIDI_CHANNEL=1.
Do not assume eight CC1–8 defaults, zero-based device channel units or that an
unused knob cannot send. A recipe that reassigns P2 to Expression must explicitly
disable the factory P3/CC11 duplicate and every other unused knob; the current
u8-only controller command cannot yet express-1 and needs native admission plus
a negotiated wire update. This is a newly measured planning obligation, not an
admitted receiver-performance claim.


### Native PERF-page probe and selection counterexample

Bytecode for VYW.iDx confirms fpG.mQv → TUp(fpG) → qhW, then an explicit
hne.qKA(Zj3) selection call. The next isolated probe uses the same native page
factory and owned insertion inside our admitted transaction, while omitting that
UI selection command. It is under test and not yet production admission.


### Receiver-range decision and monitor hypothesis, wire13 continuation

The local archive must precede new library research. Relevant primary OT articles:
`research/manuals/text/363-berlin-strings-notes.md` (VlnI/VlnII G2–F6,
Viola C2–C5), `354-berlin-woodwinds-soloists-i-notes.md` (Solo Oboe Bb2–F#5,
one mf layer), `362-berlin-percussion-the-timpani-notes.md` (Normal Single Hits
C2–B5, seven layers/RR5), `349-berlin-harpsichords-notes.md` (French8ft/Italian
registers). Octave labels and percussion keyboard mapping still need installed
receiver confirmation; do not turn those labels into an unmeasured MIDI interval.
Berlin Brass recorded sustains Gb2–C5 cannot cover the two high BWV parts under
the candidate Kontakt C3=60 convention. The first-party Audiobro chart offers a
clean candidate: Piccolo Trumpet1/2 recorded D4–F6, playableBb3–Bb6, explicitly
concert pitch/C4=middleC on https://audiobro.com/modern-scoring-brass/ . Retained
primary chart images are in the BWV research folder, sourced from
`https://audiobro.com/wp-content/uploads/2022/03/Piccolo-Trumpets-Reduc.png` and
`Trumpets-Chart-Reduc.png`. No sample range extension or score transposition is
admitted from a filename. Candidate player settings remain separate setup gates.

A sample-free Kontakt receiver monitor is being probed, not admitted: an empty
instrument KSP script exposes RXOn/RXOff/RXChan/RXKey/RXVel/RXCC1/RXCC11/RXCCN as
host automation IDs0–7, and only observes received callbacks. This is a disposable
receiver, not an importer or MIDI injection path. Official KSP Control Parameters,
Event Commands and Version History establish automation IDs0–2047, event channel
queries and their version constraints:
https://docs.native-instruments.com/ni-tech-manuals/ksp-manual/en/control-parameters
https://docs.native-instruments.com/ni-tech-manuals/ksp-manual/en/event-commands
https://docs.native-instruments.com/ni-tech-manuals/ksp-manual/en/version-history
The hypothesis is that native HDN readback reflects these receiver counters. It
must pass a real playback/render plus negative routing before use in durable QA;
script source and UI labels alone do not prove that host parameters update.

The first monitor hypothesis was falsified: real playback delivered four on/four
off callbacks (Kontakt UI showed last channel15/key62/velocity114), but all native
PID0–7 base/modulated values stayed0. `HostAutomation` names/values are therefore
not a receiver telemetry path merely because a KSP UI knob has an automation ID.
Files `/tmp/import-v13-monitor-{script-native-before,during-play,after-play}.json`
retain the native counterexample. No GUI observation is admitted as durable QA.
The next probe uses KSP's documented `save_array_str`, once from an explicitly
triggered `on ui_control` callback, after collecting a bounded in-memory event
array. No file I/O runs in note/controller callbacks. The official Load/Save
Commands manual restricts asynchronous save to ui_control/persistence/pgs callbacks
and supplies completion IDs/status. Source: https://docs.native-instruments.com/ni-tech-manuals/ksp-manual/en/load-save-commands . The retained diagnostic
`observe-array.ksp` is exploratory until native parameter-triggered capture and
independent NKA readback pass. `host paste --input` now supports bounded UTF-8
multiline editor input, preserving the Rust facade; file/dialog `--text` remains
single-line. This development helper is not a durable score import operation.

### Live native parameter edit and first quantitative receiver callback audit

Source exegesis: installed
`control_surface.values.ParameterProxy.set` → `RangedValueProxy` →
`DecimalValueAtomValueTarget` / `ParameterValueTarget.setCurrentValue` →
`FA.qKA(double)`. The owner is the parameter in **plugin contents**, not its WwV
wrapper. Merely writing the HDN stored `value` yielded matching base-value
readback but no live KSP callback/export (two distinct probes, no internal retry).
The SDK's native edit path produced `capture-101.nka`: complete empty baseline,
20490array values with schema/extent/sentinel. Native normalized0.00010125 drove
integer capture101; the quarter-step interior avoids an integer-boundary
ambiguity and is an explicit receiver quantization choice, not a retry policy.

Native transport probe lessons: SDK TransportProxy uses owner mfy (document
`transport`), `tGW(double)` for playback start, `Huv()` for Play and `ed_()` for Stop.
Immediate Play readback may still be stopped; native state requires a separate
measurement. A raw full-project revision changes while playing, so revision-fenced
Stop was refused without mutation. The development stop gesture was explicit.
The typed lifecycle now separates identity-fenced Stop from revision-fenced
stopped Play/Seek, with status reconciliation and an explicit superseded-Play
receipt. A first probe reused a single snapshot builder and its second read failed;
subsequent observations use fresh builders. None of these failures was normalized
away or converted into an internal retry loop.

ReceiverMonitor UUID `3dbb8a71-eb42-42f3-8369-2ac1733a93b0` was cleanly
reopened through the production adapter. Rust-facade seek2/seek0 completed with
independent native position readback. Play and Stop initially returned pending;
separate status reconciled both native `is_playing` and `is_playing_on_engine`.
No Play/Stop was replayed. Identity authority on Stop avoids the moving revision
race while binding the same project and controller lifetime.

`receiver inspect --baseline` validates the complete earlier callback prefix,
capture IDs and advancing engine clock before selecting appended events. It
never drops unexpected setup/Stop callbacks. An explicit timingEventCount may
limit timing assertions to the score prefix while still checking every later
callback's kind/channel/number/value/order and the complete event count. Output
reports timed and untimed counts. A matching cumulative prefix is continuity
evidence, not an independent proof of instrument identity; retain project/device
and operation receipts with every capture pair.


## Native parameter identifiers: hexadecimal, not decimal indices

The scoped parameter view of the loaded Berlin Strings Kontakt exposes2065native
parameters: PID0…PID7ff, PID800 and sixteen IDs PID4d318200…PID4d31820f. This falsifies
the earlier assumption that suffixes are decimal indices. PIDa has name#010;
PID10 has name#016. Only exact observed canonical lowercase PID0…PID7ff identifiers
are admitted for ordinary plugin writes; other observed IDs remain read-only until
their semantics are established. PID8/RXCapture evidence remains valid. No decoder
translates an identifier into a differently numbered parameter.

The loaded ordinary violin NKI exposes placeholder names#000 etc, rather than
private CAPSULE controls. Generic host parameter values cannot attest those
settings or NKI path. Retain explicit configuration provenance and independent
receiver/audio evidence. Scoped views retain physical base values and type metadata,
including missing fields. A reduced historical fixture supplies only type=am;
range127 must not be invented there. The complete saved strings capture supplies
CC0–127, P1=71.56455329213398, CC1=1, CC2=11, MIDI_CHANNEL=1 and CC3…CC8=−1.


## Separate build outputs for the development harness

Maven's harness profile previously used the same target directory as the main
extension. Building one profile cleaned the other's artifact. The profiles now
use target/main and target/harness; both artifacts coexist and the Rust façade
reports profile, artifact bytes/SHA and wire version. Clean wire16 host startup
loaded both adapters successfully. A harness-only code deployment/reload remains
sufficient for isolated probe iterations; it does not upgrade the production adapter.

## Sample-free native Layer routing and preset-state delivery probe (wire16)

Disposable ReceiverRouting16 UUIDec7c802f-96d4-4d87-ac61-6589c74807d7 contains
one five-note clip and a native Instrument Layer with two slots: input0→output0
and input4→output0. Each owns MIDI CC→the diagnostic Kontakt preset. Both native
Kontakt instances expose PID8/RXCapture after activation. Factory CC2/CC11 knobs
were disabled before authored curves. Slot A has CC1 .25/.5/.75 at beats0/2/4;
slot B .75/.25/1. Native observations independently match complete planned envelopes.

Playback contains simultaneous key60 attacks on input0 velocity.25 and input4
velocity.75 at beat2, a simultaneous input15/key62 sentinel, then input0/key64
velocity.5 and input4/key67 velocity1 at beat4. All durations1beat, release.4.
Captures301/302 establish each receiver's complete prior buffer;303/304 retain
unchanged prefixes plus exactly seven new callbacks per receiver. The Rust audit
compares authored expectations, not copied actual events. A receives key60/32,
key64/64; B receives key60/95,key67/127. All delivered channels are0. Each gets
only its CC1 values (A64/95 then Stop32; B32/127 then Stop95), note-offs51, and
neither receives key62. Six timed callbacks are within3ms of score-relative
expectations under an explicit5ms tolerance; the final Stop reset is value/order/count
checked with timing intentionally unspecified. Both audits pass.
This proves actual Layer note filtering/remapping and independent stored CC delivery
without samples. It does not establish an acoustic library's articulation behavior.
Persistence and service-independent replay are separate follow-up checks.

### Routing persistence and service-free playback

The old development Stop's whole-project revision fence refused a moving
playhead before dispatch. Its probe now explicitly uses disposable-project UUID
for Stop only; Play/Seek retain revision authority. Production Stop retains its
stronger project/controller-generation authority. Native post-command readback
is asynchronous: immediate playing=true did not mean Stop failed; a separate
read confirmed both playing flags=false. A development space-key attempt did
not stop playback and is retained as failed recovery, not acceptance evidence.

The development production-operation bridge also exposed a hot-loader defect:
ObjectNode from the stable harness loader is not ObjectNode in its child loader.
Use a String boundary and parse inside the owning adapter, as existing named
construction probes do. Source has this correction; the stable harness entrypoint
requires a full controller/host reload before live admission. Code-only reload
cannot replace that stable entrypoint. The original request refused before dispatch.

## Connection recovery investigation and corrected source reading

SIGINT-stop/restart of the owned service left the old production connection
unavailable; controller power off/on restored it. BitwigStudio.log contains
java.net.ConnectException at BitwigStudioHost.connectToRemoteHost. The initial
inference that this IOException escaped and killed the scheduled tick was wrong.
Installed bytecode (`javap -c -p`, verified bitwig.jar) at method bytecodes25–79
constructs Socket synchronously, catches IOException and logs it, then returns
without a ConnectionEstablishedCallback. On success it schedules that callback
through oGN.TUp. A caller therefore needs an observed callback/welcome deadline;
a request or stack trace alone does not establish the connection outcome. The
precise old recovery failure is not proven by that stack trace.

Official bitwig/bitwig-extensions commit43c1a00153072f7f9801d82335077906e504e512
has no connectToRemoteHost/reconnect example in src/main/java. DrivenByMoss
commit6486e0b12cf2fd80bab73faa4454408d5f4ba899,
`src/main/java/de/mossgrabers/controller/gamepad/controller/GamepadControllerInputThread.java`
lines98–131 uses connection-state-driven hardware reconnect and reports runtime
errors at its loop boundary. That is not proof of identical TCP behavior or
permission to retry musical operations. No automatic retry workaround was admitted.

## Explicit connection lifecycle: live wire16 admission

The single production extension now owns a narrow, owner-private lifecycle
mailbox; it cannot carry musical requests. `adapter bitwig connection
list|connect|disconnect|status` exposes it through the Rust façade. The extension
makes one boot connection attempt. Each explicit connect makes one more. Refusal,
missing callback/welcome, EOF, late callback and exit stop an attempt; they never
schedule another or replay a mutation. A generation token rejects late connection
callbacks. The actor chooses when to connect, then separately reconciles native
operations and fresh document state through the normal service wire.

The String-based stable/child-loader production-operation harness boundary is
also live-admitted: identity-fenced native Stop and its separate status both
return transported/targetMatches=true, without the previous ObjectNode class
identity error. Harness startup no longer depends on the old port17879 transport
experiment; that experiment is an explicit named transport-connect command with
no implicit reconnect. A further restart experiment found the old harness re-read
its previous command file: operation2982459-1792835188637895846 appears both
09:46:29 and10:16:54, with the second refusing No connected project. Source now
seeds its last-seen command at init so prior-process requests remain unknown and
are never replayed; its clean-restart admission is recorded below.

### Routing ownership and acoustic probe lessons, 2026-10-24

`controller project view routing --track UUID` reports the track filter/override
and every owned routing node. Native `kdt` nodes also own internal FX chains:
one articulation slot containing MIDI CC and Kontakt had three nodes, but only
one enabled input filter/output remap. Count enabled routing policies separately
from owned nodes; do not infer articulation count from all `kdt` instances.
The regression oracle explicitly checks three nodes and one filter/remap.

The Rust `controller audio pitch` command implements bounded, single-window
difference/CMND threshold and parabolic refinement from de Cheveigné & Kawahara,
YIN, JASA111(4),1917–1930(2002), DOI10.1121/1.1458024, sectionsII.B–E,
equations6/8. Primary-source references:
https://pubmed.ncbi.nlm.nih.gov/12002874/ and
https://iro.umontreal.ca/~pift6080/H09/documents/papers/yin_pitch_tracker.pdf .
The paper's unqualified global-minimum fallback/tracking are deliberately absent:
unreliable periodicity fails this audit rather than fabricating an estimate.
CMND locates/refines the qualified dip depth; the raw difference refines the period.
Analytic tones, missing-fundamental harmonics, noise/silence and invalid resource
budgets are tested. CLI tests distinguish correct pitch, a1200cent error, silence
and immutable output. These tests do not establish timpani perceived pitch.

Development-only window diagnosis: the strings Kontakt Load command did create a
visible Wine dialog, but it was behind the embedded plugin frame; focus was
8388611 while the Load window was23068693. The Rust host windows query exposed
both rather than assuming the load failed. One explicit raise/focus of23068693
measured before8388611/after23068693; the normal filename field then accepted the
exact NKI path. `adapter bitwig host windows|focus` are development X diagnostics,
not document APIs or durable realization/acceptance commands. Do not repeat a load
because a modal is hidden. The private setup boundary still requires the authorized
operator; automated native audits never inspect this GUI.

### Live private-setup gate and stored strings audio, 2026-10-24

The three actual Berlin Strings immediate-sustain NKIs were loaded normally into
P7/P8/P9, set to Port A/channel1, with XFADE/Romantic shown. The authorized agent
is the setup operator; no user confirmation is fabricated. Native observation
retained1374notes and three healthy Kontakt7.10.1 instances. LOADED also described
the empty third instance before its NKI load, reinforcing the private-state limit.
`import checklist` resolves slot ownership to device UUIDs independently of custom
names; native device_name contains the assigned RX name, not the factory name.
Its template leaves operator/time/save-receipt/sample confirmations null.

Timpani acoustics research: the indexed primary abstract of Craig A. Anderson and Thomas D. Rossing,
“The acoustics of timpani,” JASA66(S1),S18(1979), DOI10.1121/1.2017643,
https://pubs.aip.org/asa/jasa/article/66/S1/S18/731734 reports approximately4:3:2
frequencies for three principal modes. Full publisher fetch was blocked; an
academic experimental page https://www.mdw.ac.at/iwk/wiener-pauke/ also timed out.
This supports investigating a modal/perceptual distinction when strict periodicity
fails; it does not establish this sampler's mapping or justify changing the
existing25cent/0.1 thresholds. Searches for the installed2013 patch's numeric
keyboard mapping found no authoritative answer. The later archived OT article362
names a different1.1-style patch and cannot retroactively supply the old1.0 range.

### Cold restart, strict setup refusal and recovery, 2026-10-24

Checklist output now projects compact plugin facts while retaining the full native
capture:849243bytes became11171bytes. Facts include preparation/modified/newer-than-
saved flags, state resource name and parameter count; no2065-element reference-array
dump is needed for setup decisions. The shared summary projection preserves null
for unknown fields. Read-only workflow failures identify their stage/verify target
and distinguish them from a possibly dispatched mutation.

The development harness journal had grown to2.1GB. An operator whole-log parse
used approximately7.7GB resident memory; it completed but was unnecessary. Count
specific canonical operation-prefix rows with bounded streaming, not whole-log
JSON materialization. Rust harness invocation already seeks the current tail;
its per-response read is now bounded to128MiB and retains outcome-unknown/no-replay
semantics on an oversized or incomplete response. This is development evidence
handling, not a new production transport or durable score-realization dependency.

## Source ownership and note replacement probes — 2026-10-24

Precedent: the documented Openwig owner-thread/native-command and doctor patterns in
`references/java-harness.md`, plus the already admitted native EditorAdapter61
create/remove operations. Installed implementation exegesis resolves a different
question: how to detect actual shared content before applying those operations.
`UMj.FSQ()` returns Dxc. `SuS.bRk()` obtains its owning track through `Dee()`,
selects that owner's native `mvn()`/`nOc()` stream, then filters by exact
`bT.QSj() == source` identity. Its two predicate bodies both perform that exact
object comparison. This is not an arranger-count heuristic or a project-global
name match. `yyk.EwU(false)` includes `dEG()` and the `bOY()` container list;
`GSp.qry()` combines arranger events with native launcher references. Launcher
and cross-container behavioral coverage still require their own fixtures.

Read-only probes on the configured strings project observed one source user for
P7's528 and P8's454 notes, with before/after complete observations equal. In a
separate `IMPORT-SOURCE-PROBE-01` project, a native alias made two arranger clips
reference the same two-note source. The native usage stream returned both clip
objects. The Rust façade now reports `sourceAccounting`: five arranger clip-note
references versus three distinct native note objects in the complete fixture.
Those counts concern observed main arranger lanes, exclude launcher/take-only
material, and use current native IDs rather than persisted musical identities.
The older `sourceShared` content-oracle field only counted arranger references;
its scope text is corrected rather than promoting it to global uniqueness proof.

Launcher probe refinement: `ApiTrack.clipSlot` calls `yyk.cf_()`, whose
implementation is the public default interface method `kfE.cf_()` (delegates to
`T1L().et_()` or0). The old reflection resolver walked only class superclasses
and missed it. A final `Class.getMethod(name, exactParameterTypes)` fallback
resolves public inherited default methods without changing existing declared/
protected-method precedence. A focused Java regression covers inherited defaults,
most-specific overrides, exact overloaded parameters, cached metadata and native
failure causes. The first corrected probe then refused because this fixture had
zero launcher slots; creation must be explicit rather than silently filling slots.
`ApiTrack.clipSlot`'s native append pattern is `oKx.heu()` →
`track.dEG().oza().v33(ntl)`. `ApiLauncherSlot.setClip` delegates to
`ntl.TUp(fza)`. The development fixture requests at most one explicit append at
the current count; no allocation or retry loop is admitted by that pattern.

A separate bounded global probe walks the same native group/return/master owners
as ProjectAdapter61, then each physical channel's `nOc()` registry and exact
`QSj()` source identity. On the initial fixture it observed five channels/two
native clip references, exactly one matching the selected source, without changing
the complete observation. A cross-track launcher attachment saw three clip
references globally but still only one original-source user: the host made that
cross-track launcher content independent. Do not describe it as a successful
cross-track alias or infer that content UUIDs survive saving/reopening. Its whole
receiver/launcher owner payload is useful unrelated-state preservation evidence.

### Cross-track copied UUID counterexample

The first complete native-registry fixture falsified treating
`clip_content_timeline_id` as a physical-source identity. After cross-track
launcher attachment, the exact-object native lookup reports only one user of the
original Dxc, yet the copied independent Dxc has the SAME content UUID. Editing
the original changes its payload/note count while the destination's serialized
track payload and copied notes stay unchanged. The new Rust global validator
correctly rejected the conflicting UUID-keyed facts rather than publishing a
passing source inventory. Retain that failed interpretation: native content UUIDs
are copyable diagnostic generations, not unique physical object keys, even within
one loaded document. Managed source-object identity is being probed explicitly;
never fix this counterexample by suppressing conflicting payload checks.

The managed-ID probe passed without changing the complete observation. Native
source2152 and source2458 both reported content UUID
`3efd1e90-2cfe-4f73-84c8-50ec9641c2be`; exact `QSj()` identity was true only for
2152. Source2355 was the other original one-note source. The corrected registry
schema2 therefore keys physical sources by canonical positive managed IDs and
retains UUIDs only as copyable diagnostics. Rust wraps this key in validated
`SourceId`; UUID strings/zero/noncanonical decimal strings cannot deserialize as
source handles. The fresh fixture reports three physical sources, two UUID values,
six physical-source notes, and four main-arranger note references. Earlier
UUID-keyed schema1 observations and their failing tests remain retained.

During the wire17 transition, both fixture projects had completed native saves,
verified closes and an empty application inventory before normal UI window exit.
The outer dbus/Bitwig launcher returned0, but its child3032723 reported exit1.
The application tail explicitly says all controllers stopped, engine3032939
terminated with exit0, and the application event loop finished; it then reports
XIO fatal I/O error0 during X11Display shutdown. Preserve this distinction rather
than claiming a clean Java exit or a musical/import failure. Host-start reporting
now separates outer launcher exit and reported child-error lines from a bounded
final256KiB log read, returns a failed status when a child error is reported, and
never restarts automatically. A parser regression uses the observed exit line.

## Installed wire17 loaded-source replacement — 2026-10-24

Native Undo and Redo retained the initial and changed observations at16389227
and16389861decoded bytes respectively. There is no automatic rollback/retry.
This is loaded-player source/history evidence, not acoustic or full-orchestra
evidence.

### Receiver-expansion research continuation

The installed candidate Audio.Imperia.Nucleus.v1.3.0.KONTAKT contains dedicated
`Nucleus - 01 Timpani - Hit.nki` and roll/multi patches, but no local PDF was found
in its library root. The current official product page
https://www.audioimperia.com/product/nucleus/ advertises1.4 and separately lists
1.3 additions. It is not an edition-matched1.3 manual or a tested range assertion.
A VI-CONTROL historical discussion points to the vendor's original downloadable
manual: https://vi-control.net/community/threads/out-now-nucleus-the-orchestral-core-musictech-excellence-award-10-10.84366/page-10
(indexed lead, not primary authority for the installed patch). The vendor-CDN
manual URL surfaced through an accessibility walkthrough:
https://cdn.shopify.com/s/files/1/0855/2380/files/AudioImperia_NCLS_Documentation_V1.4.pdf.pdf
Do not infer exact1.3 key mapping, sample-start compensation, tuning or a successful
timpani receiver merely from that1.4 document. Probe the installed patch and
separate manual revision, native state and acoustic observations. Existing2013
TheTimpani failures and unchanged thresholds remain retained; no score transpose
or tolerance widening is authorized by this new candidate search.

## Nucleus acoustic probe and explicit analysis boundaries

A2-low and D3-low render/engine callbacks complete. Both files are320001frames,
48kHz/stereo24bit, against the initial prospective320000expectation. A2's RMS
is−79.19364/−78.14189dBFS, no clipping, first−90dBFS activity80865/81331 versus
planned80000. Both declared2.15..2.45second YIN windows refuse an estimate:
normalized best dips0.13916/0.14927 exceed unchanged0.1. No octave/range shift,
window search or tolerance relaxation follows. These are retained failures.

A new descriptive Rust `audio spectrum` command uses RustFFT6.4.1's recommended
FftPlanner/forward FFT API (https://docs.rs/rustfft/6.4.1/rustfft/), scalar features,
periodic-Hann coherent-gain bin amplitudes, mean removal and explicit bounded
window/band/peak controls. Four new lockfile packages are rustfft,primal-check,
strength_reduce,transpose; existing num-complex is reused. Controller MSRV1.87
check passes. Tests independently establish exact-bin amplitude/DC, off-bin
mixtures, silence, invalid bounds, channel/window ownership and immutable CLI
outputs without a host. Shared stable WAV window reading now also serves YIN;
its correct/wrong/silent regression remains passing. This is offline allocation,
not an audio-thread callback or an additional host integration path.

The same fixed diagnostic windows yield strongest peaks109.944Hz and147.097Hz,
with near-fifth/octave partials and additional inharmonic peaks. These observations
are descriptive; they do not pass the original YIN test or identify an NKI.
Primary acoustic context: IWK's2002 Wiener Pauke reprint
https://www.mdw.ac.at/iwk/wiener-pauke/ distinguishes sustained tonal modes from
rapidly decaying inharmonic components, and labels mode11 fundamental,21 fifth,
31 octave. Anderson/Rossing1979 https://doi.org/10.1121/1.2017643 reports near4:3:2
principal-mode relationships. These motivate an explicitly authored model; no
standard or vendor mapping is inferred from them. Primary/missing-fundamental
interpretations must be explicit, never selected to fit a convenient octave.

`audio partials` checks exact authored ratio bands, signal level, relative peak
floor and optional strongest-primary criterion; no free fundamental search.
Wrong key/octave, detuned/weak/missing partials, malformed/overlapping bands and
silent input refuse. New `audio inspect --expected` enforces exact WAV frame/
format/clipping and signal/digital-silence criteria; one below-threshold nonzero
sample is not digital silence, and a one-frame mismatch fails quantitatively.
Neither helper grants private setup, timbre, articulation or perceptual fidelity.

A separately specified0..6beat export gives native expectedSeconds5.0 and240001
frames. This falsifies a simple ceil-duration model. On the documented build, the
three current exports and older retained examples are consistent with a final
sample included under floor(durationSeconds*rate)+1. Java pfG→Dmk→dOp preserves
requested beat endpoints and records native converted seconds; GzF forwards the
range/context. The deeper engine's rounding implementation is not established.
Do not promote the observed formula to a public Bitwig guarantee. The independent
forward0..48beat/40second fixture predicts1920001frames BEFORE its render;
follow-up-01/SPEC.md binds exact frame and three-modal-band expectations for
previously unrendered medium/high strikes. The first failed criteria stay intact.
A separate72..78beat silent render is planned with240001frames. Results pending.

A targeted export-length search found no Bitwig primary specification of this
one-sample behavior. Official exporter documentation
https://www.bitwig.com/userguide/latest/exporting_audio/ explains range/rate/
encoding settings without a sample-endpoint guarantee. The Bitwig forum lead
https://www.kvraudio.com/forum/viewtopic.php?t=571895 concerns selectable export
rates, not proof of an endpoint rule. Reports about other DAWs' off-by-one exports
are not authority for this host; no workaround/retry policy is admitted here.

The forward40second render has exactly1920001frames and passes its prospectively authored WAV contract.
Three of four new partial-band cases pass. A2-high fails ONLY the authored
strongest-primary hypothesis:110.479Hz is+7.522cents,8.351dB below165.986Hz;
its218.620Hz octave is−10.895cents,19.647dB below the strongest peak. Allthree
25cent bands/20dB floors pass. Preserve that failed hypothesis and untouched
expectations. IWK's actual discussion of weaker mode11 in international timpani
provides concrete reason that mode identity need not mean amplitude rank.
The library already requires rank only when explicitly requested; no test or
criterion in the failed case is removed. A new synthetic regression retains
rank-required refusal, demonstrates explicit unrestricted-rank mode checks and
still rejects wrong key/octave under that separate model.

follow-up-02/SPEC.md prospectively declares exactlythree new native full renders,
all18strike windows, unchanged25cent/20dB/signal/frame criteria and explicit
unrestricted mode rank. Every case must pass, with separate wrong-key/octave
failures. There is no rerender-until-passing loop and no RR-identity claim.
The native72..78beat control completes at5seconds/240001frames and passes exact
digital silence on both decoded channels. Replication results remain pending.

## Native selected leaf export: forward admission, 2026-10-24

Rust exact WAV criteria now admit authored per-window signal/clipping contracts.
Missing/shifted windows and one-bit leakage are negative tests; expected windows
are read from the contract, never inferred from measured audio. Requirements
formalize the already-written prospective fixture; no post-observation threshold
or window changes were used. Production target support requires wire18; archived
wire17 artifacts remain unchanged and historical.


## Isolated original BWV248 strings excerpts: forward pitch/control audit

Review exposed two silent-loss paths. Key-velocity recipes now refuse an
independently authored dynamic level as well as a sustained curve, rather than
ignoring that axis while using attack velocity. Nonempty balance/space decisions
refuse E_MIX_UNREALIZED with the full witness until a native mix plan is admitted.
These are explicit capability gaps to implement, not authorization to discard
musical intent. New negative tests establish both refusal boundaries.

### Mixer units investigation (not mutation admission)

Read the documented Openwig volume().value()/displayedValue() observation precedent
(local `/tmp/notes-internal-research/openwig.js`,lines297–308) before the native
mapping. ProjectAdapter61 already resolves group master RN.DQP and owned mixer.
Installed CFRfw.java lines157–225 establishes domain→linear→domain and
physical-domain→display-unit overloads; fu.java identifies CUBE_ROOT/DECIBEL.
Cube-root native0.6812920690579614 is a candidate−10dB rather than a normalized
68% gain. A read-only `construction mixer-inspect --track UUID` probe now emits
actual native owner/type/value and exact fw conversions. Results and any setter
admission remain pending; do not infer pan law or expose a gain formula from
an enum label alone. No production mix request has been admitted.

Mixer read-only live results used wire18 and an explicit reload on main. Original strings leaf and parent-group probes both
measured nativevolume0.6812920690579614→engine0.31622776601683805→
−9.999999999999996DECIBEL. Pan was0,LINEAR/LINEAR,PERCENT,bounds−1..1. Exact
owned mixer/type/automation/modulation attributes are retained in
/tmp/import-v18-mixer-{leaf,group}-inspect-01.json pending archival.
A separately specified `validation/import-construction/mix-gain/SPEC.md` now
controls the setter experiment. Native SaveAs createdMixerProbe.bwproject,
2935677bytes,
from the preserved saved stem fixture. Native open/SaveAs left that saved source
engineNOT_CONNECTED and pluginsNOT_LOADED; an explicit activate operation is
therefore required before musical mutation/render. File open/save success is not
engine readiness. Setter/Undo/gain/silence/pan counterfactuals remain unadmitted.

Mixer setter probe: volume1 read back native1/engine1/display0dB. One nativeUndo
restored the entire complete observation exactly; oneRedo restored the entire
changed observation exactly. A stale expected value refused with complete snapshot
preservation. Nativevolume0 yielded exact digital silence in master and leaf
0..24beat exports, both960001frames; oneUndo restored the complete before state.
Nativepan−1 read back and oneUndo likewise restored the whole before state.
Remaining pan/audio cases and aggregate preservation still pending.

The sampled-source master-gain hypothesis failed its original10±1dB gate:
8.78783964/8.89673840dB. Corresponding leaf changes9.76892326/9.84997941 passed.
Both witnesses remain unchanged. Prior unchanged-gain fixed Nucleus replicates
already show first-strike window RMS spanning1.66225dB; RR/sample variation is a
plausible confound, not a demonstrated cause. Do not admit exact master gain by
widening the failed threshold or repeating the export until green.

For a separate controlled-source experiment, official Bitwig Utility userguide
https://www.bitwig.com/userguide/latest/utility/ documents Test Tone as a generator
requiring no input, including sine and explicit Gain/Mix. Consulted2026-10-24;
latest manual is informative, actual installed6.1 parameter readback is authority.
InstalledJAR string search located com.bitwig.flt.packaging.core.nj2; CFR output
shows XrN UUID20b72dbc-0fe1-47c5-867c-f0ab1510f723 registered as Test Tone and
`devices/Test Tone.bwdevice`. The same admitted factory-preset insertion pipeline
can probe `/opt/bitwig-studio/Library/device-settings/<UUID>/Default.bwpreset`
without editing that factory resource. This is a new deterministic-source
hypothesis, not a rerender/relaxation of the sampled-source experiment. Insertion,
parameter settings, signal determinism and gain proof are still pending.

The fixed sampled-source mixer experiment finished with master-gainFAIL retained,
leafgainPASS, exact master/leaf zero at nativevolume0, exact opposite-channel zero
at left/right pan, complete Undo/Redo equality for both pan setters and volume,
and stale-value refusal with full equality. After restoring pan0/volume−10dB,
complete native state equaled the initial before exactly (zero differences).
Final native save retained2935677bytes before close. The source stem was retained.
Indexed full raw/packed witnesses live under mix-gain/host-01,host-02; all failed criteria remain separate.
Rust WAV contracts now support channel-indexed signal/clipping criteria on whole
files and windows; wrong-side, missing/duplicate channel and one-bit leakage are
negative cases. They formalize the predeclared pan checks, not a new pan law.

## Continuous source: meter authority counterexample and wire19 admission

Test Tone default native parameters were FREQUENCY83.21309485364912 in PITCH/
HERTZ(1000Hz),GAIN0.3116666666666666 CUBE_ROOT/DECIBEL,MIX0.8250000000000003,
WAVEFORM0 resolving to native Sine/single-harmonic enum item,BIPOLARtrue.
Normal UI setup gestures were not accepted from images; native before/after showed
all parameters unchanged. A new named factory-parameter probe uses exact owned
track/device/factory/key/name/current value, continuous numeric am type/range,
healthy stopped engine/drained queue/no automation or modulation, one native Undo
and flush. Its first MIX request refused stale before writing.

Build ergonomics: old Rust binaries labeled new Java artifacts with the CLI's
wire constant. The builder now executes only its newly built workspace artifact's
small SDK-free AdapterMetadata entrypoint and verifies stable artifact bytes.
It reports actual artifact/CLI wire values and refuses mismatched deployment.
Loaded descriptor/native observations remain required; metadata is not host
readiness. Historical logs from old builder wrappers are retained as such.

### Deterministic native mixer forward controls and reusable Rust evidence

The independent `controller/validation/import-construction/mix-tone/SPEC.md`
fixed all source/format/stability/gain/silence/pan criteria before forward renders.
Production wire19 Test Tone master baselines both measured−53.38737018123997dBFS,
zero difference. Leaf nativevolume−10dB→0dB produced master+10.000000794092259dB
and leaf+10.000000966425958dB, both channels, within the fixed±0.01dB criterion.
Nativevolume0 yielded exact digital zero in complete master and leaf WAVs;
hardleft/right exports showed positive selected side and exact opposite-side zero.
All exports960001frames/48k/stereo24-bit/zero clipping. Each setter has independent
full before/after; one normal nativeUndo/Redo restores exact material revisions.

A separate group-volume extension was written prospectively, after leaf cases and
before group mutation. Group−10dB→0dB passes master10±0.01dB while isolated leaf
stays0±0.01dB. Groupvolume0 makes master digitally silent while isolated leaf stays
0±0.01dB. This establishes ancestor omission in isolated leaf exports; it does not
admit group pan or general stem summation. Stale expected value and unknown UUID
refuse before writes with clear native exceptions, no retry. These results do not
relabel the failed sampled Nucleus master10±1dB experiment; the same immutable
WAVs still fail under the new Rust `controller audio gain` command.

### Native import lessons and evidence index — 2026-10-24

The records below describe bounded experiments on the admitted 6.1 build.
The score-realization contract in controller/IMPORT.md determines acceptance scope;
experimental criteria do not automatically become product requirements.

| Finding | Evidence / source | Consequence |
| --- | --- | --- |
| Project creation can complete before engine initialization | `validation/import-construction/mix-workflow/host-01/`: baseline is_engine_ready=false/shared_memory_id=-1, later ready=true/initialized | Observe readiness when an operation needs it; retain stale refusal and take a reviewed fresh baseline. Do not broaden authority exclusions or internally retry |
| Native device identities can regenerate on reopen | `service-independent/host-02/`, `wire16-startup/`, prior source-ownership probes | Define musical targets by qualified named hierarchy/object kinds. Resolve temporary native handles inside the adapter; raw UUID/index equality is not persistent identity |
| Current primitive ownership checks use native handles/ordered paths | `controller/src/import/schedule.rs`, retained old-UUID refusal and six rebound checks | This establishes measured state, not the desired semantic facade. Fix the contract/resolver rather than institutionalizing UUID recovery |
| Active player unload may affect saved state | `mix-workflow/host-02/`, `service-independent/host-02/` | Measure deactivate, then normal save, then close. A preflight-refused Close has no registered lifecycle completion; unknown status is not success |
| Open can mark a project modified | `mix-workflow/reload-dirty-diagnostic-01.json`, `service-independent/host-02/` | Retain the condition and normal-save result. Never clear the flag in an oracle or treat historical save bytes as current proof |
| Lost/error acknowledgments need retained intent | Generic project/application lifecycle exchange and no-current-project negative | Publish request intent before dispatch; distinguish read-only failure from unknown mutation outcome. Reconcile actual state/status instead of replaying |
| Actual loaded metadata matters | adapter build/deploy metadata and connection descriptors | Build/deploy is not proof of a loaded compatible adapter; measure the running instance |

Mixer experiments: `mix-tone/` independently establishes the nominal native fader
mapping with a deterministic source, silence/pan behavior and preservation/history.
`mix-workflow/` retains the 1,374-note, 57-step, 64-check construction/mix fixture.
Exact artifact identities and full results are in their SPECs and evidence indexes.
Neither is a complete orchestral-performance proof. The sampled Nucleus master
comparison still fails its own original 10±1dB experiment; it changed a Bitwig
**audio track fader**, not MIDI CC. Independent sampled renders did not establish
source repeatability or the cause of the discrepancy. Native/display precision
is not an acoustic-precision guarantee. This does not establish a Bitwig bug or
block basic note/velocity/tempo transfer. Investigate further only for a relevant
requested mixing/control requirement. Keep the original failed measurements.

### Service-independent entry and clip-end admission, wire20

Evidence: `controller/validation/import-construction/service-independent/`,
including prospective SPEC/expected events, captures7001..7012, all three case
summaries and indexed host-01/host-02 receipts. Two independent sample-free KSP
receivers observe stored notes and CC1 through native filtered/remapped layers.
20BPM and delayed first attacks provide a declared pre-event disconnect window.
Rust serve is explicitly stopped and a query returns ConnectionRefused; reconnect
and operation status are separate agent-issued commands without mutation retries.

| Case | Observation | Conclusion |
| --- | --- | --- |
| 1 | Both receiver checks pass; agent Stop arrives at beat41.72 rather than before16 | Procedural test failure, not host instability. Final CC restoration occurred at clip-end, before Stop |
| 2 | Stop at beat6.83; exactly6new musical callbacks per receiver, all within3ms; no expected seventh reset | Early Stop does not restore CC base in this fixture. Preserve failed7event expectations |
| 3 | Initial checkpoint0ms, score events6000..15003ms, clip-end restoration48000ms; exactly8new events per receiver, max3ms against5ms | Stored initialization, routing and natural clip-boundary delivery pass with service absent. No sampled articulation or tail claim |

Case3 disconnect completes1.6484s after status-call start at beat0.16355; Stop at
17.4791 satisfies its prospectively distinct16..24 window. All13baseline events
remain intact. Each Kontakt ENGINE_UPTIME clock is local to its own instance;
compare relative offsets within an instance, not absolute times across players.

After earlyStop/Seek0, A base/gui/modulated31.75/95.25/95.25 and B95.25/127/127
remain held. Parameter automation_statePLAYING persists while both transport play
flags are false. Inspect effective values and actual flags; do not infer reset or
playback from that enum. Clip-end restoration creates a possible effect on still
sounding releases, not proof of an observed acoustic tail defect. Test a lifetime
policy when the requested expressive behavior requires it.

Installed source lineage: TransportProxy Stop→mfy.ed_(), Seek→mfy.tGW(double),
no explicit CC/default reset. Arranger-loop enablement is
mfy.NYl().XQF().getBoolValue(), outside the enumerated transport attribute tree.
Read-only probe left the material revision unchanged; wire21 full capture includes
the Boolean in authority. Missing legacy data is unknown, not false; no loop
setter is admitted by the getter probe. Retained source traces/probes are indexed
in service-independent/host-01/; exact native map is confined to Java.

Primary context: [Automation Basics](https://www.bitwig.com/userguide/bws44-504/automation/)
and [Automation Editor](https://www.bitwig.com/userguide/latest/the_automation_editor_panel/)
describe held values and clip automation, not a universal Stop reset. The attempted
latest automation_basics URL was inaccessible. Informative forum leads:
[KVR527145](https://www.kvraudio.com/forum/viewtopic.php?t=527145),
[restart/MPE report](https://www.reddit.com/r/Bitwig/comments/y4pzos/), and
[corrected tail diagnosis](https://www.reddit.com/r/Bitwig/comments/1bac6cz/).
They do not establish a6.1import bug; the last author corrected the cause to
plug-in deactivation, and the restart author's reimport is not an admitted fix.

### Direct workflow analogues and receiver source lineage

Working-pattern sources to inspect before semantic-boundary implementation:

- [Openwig documented source](https://github.com/Axyom/openwig/tree/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d):
  composition→tracks/clips/devices/automation→render, native command ownership,
  completion and doctor probes. documented README declares Windows6.0.6; adapt and probe
  the complete pattern on our host rather than dismissing it for language choice.
- [maxdahl/bitwig-mcp](https://github.com/maxdahl/bitwig-mcp): direct clip/note
  read/create/modify workflow, deliberately narrower than automation/mixing.
- [March7-code/Bitwig-mcp](https://github.com/March7-code/Bitwig-mcp): documents
  track/instrument targeting, tempo/meter, clip/note operations and name-addressed
  launcher use. Inspect implementation to distinguish real stored operations from
  scheduled ramps/grid-limited operations before adopting a pattern.
- [Reaticulate](https://github.com/jtackaberry/reaticulate): later-phase orchestral
  articulation/bank workflow precedent; does not itself admit a Bitwig operation.
- [ProjectConverter](https://github.com/git-moss/ProjectConverter) and
  [DAWproject](https://github.com/bitwig/dawproject): interchange/data-model
  precedent, not a replacement for native live workflow/preservation checks.

The latter candidates were documentation/source leads, not installed behavioral
admissions. No conclusion that a direct analogue cannot exist is supported.


### Semantic-boundary source inspection — 2026-10-24

Read the implementation, rather than the feature inventory, in these documented
sources before the next native addressing/construction iteration:

- [Openwig bridge d9ffed4](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js):
  `_runOnDocumentThread`, `_cmdResolve`, `_insertClip`, `_clipCreateAndFill`.
  Dispatch posts a task; immediate return is a queued count, and a separate
  completion notification follows execution. Clip construction uses the native
  command's returned clip as the note insertion owner. The public handler gets
  the current cursor track's deepest target. This is a useful construction and
  asynchronous lifecycle precedent, not a qualified-name resolution contract.
- [maxdahl/bitwig-mcp c13fccf](https://github.com/maxdahl/bitwig-mcp/tree/c13fccf3ffd5fd573260088d18676fb896b017c9/bitwig-extension/src/main/java/com/pxaudio/bitwigmcp):
  `ClipHandler.createClip`, `setClipName`, and
  `BitwigMCPExtension.getGroupName`. Creation uses a bank track/launcher slot;
  naming uses the cursor clip. When supplied, track/slot coordinates are checked
  against the current selection before naming to prevent a previous cursor
  target from being renamed. Group names provide disambiguation context. This
  narrower workflow does not establish native arranger naming, complete nested
  hierarchy resolution or independent mutation completion on our host.

Project interpretation: retain the returned native clip as the construction
owner and configure its declared name in the native adapter; do not implement
naming by moving a cursor and assuming it has settled. Resolve existing targets
inside their qualified semantic parents. Keep current document/revision authority
in addition to semantic addresses. These are design directions requiring local
native probes, not new capability admissions. No host was started or changed by
this inspection. The Rust content-audit tests separately reproduce the old false
pass for ambiguous root names and cover parent/kind/name resolution; they do not
admit adapter mutation or live reopen behavior.

Inspected source bytes (local development copies retained under `/tmp/`):

### Qualified device paths: native probe (wire22)

The adapter resolves typed path segments from the already fenced immutable
observation on the document owner thread, then reuses ProjectDevice61's native
prepare/apply path. Existing native handles are retained only in diagnostic
resolvedAction evidence. ProjectAdapter61 supplies observedKind so the independent
Rust resolver contains no new native class/built-in UUID mapping.

Control Undo and Redo returned complete raw pre/post observations exactly. Native
save/close/open changed the Independent chain UUID from
`6e18a6bd-5f1c-4ba6-959e-524bc855eac1` to
`8fecc038-367a-4c91-8816-3f5c3d03eb8a`; unchanged qualified paths still passed control
and reviewed-batch content audits. Inserting an extra device before the target
then setting P1 normalized0.375 changed the original named target to47.625 native
units and preserved the extra device's defaults. Separate Undo restored the
control and then the extra insertion with zero full-observation differences.
A stale old device name failed with E_ADDRESS_MATCH; independent before/after
observations were exactly equal. These admit the exercised named primitives and
index-shift case, not full score import, arbitrary reordering, receiver delivery,
expressive controls or project performance readiness.


### Wire23 named schedule review and controlled receiver, 2026-10-24

Schema2 schedule work is indexed in
[semantic-schedule](../../../controller/validation/import-evidence/semantic-schedule/README.md).
Three fresh-context review cycles produced failing counterexamples for empty
owner membership, devices lacking factory names, and aliased track handles.
Current audits consume adapter-owned device/component roles, cover empty owners
and all owned devices, and check recipe identity after unique musical-name lookup.
The native three-note probe passed independent insertion/name/control checks and
four separate complete raw Undo comparisons (including a Surge insertion).
These are boundary observations, not complete musical-delivery evidence.

The complete unchanged BWV248/64 run-04 bundle was subsequently constructed in a
new named disposable project:14parts,3984notes,273beats. The semantic content
comparison reported zero differences and the stored tempo/meter check passed.
That experiment's SPEC and README record the deliberately controlled phase1
receiver choice, partial operation statuses, and still-open acceptance work.


The completed [basic14 scaffold checkpoint](../../../controller/validation/import-evidence/basic14-scaffold/README.md)
subsequently passed all116 live scaffold checks before close, after reopening,
and after a small isolated render. All14 receiver-chain UUIDs regenerated on
reopen while qualified paths continued to resolve. The initial P1 beats0–8 WAV
passed its prospectively authored frame/format/silence/clipping gate and A4 pitch
check. Factory Init Saw velocity sensitivity remains0dB; no velocity response or
full-score performance readiness is claimed. The named-parameter repair contract
records the actual ID/index distinction and a bounded Surge-specific admission;
the existing native-PID reserved-control protections remain unchanged.

### Native external receiver-state export — wire25 controlled probe

The default-state investigation on Bitwig6.1 establishes that `device.xuO.FSQ(boolean)` can omit an unmodified instance with
an empty saved-state filename. `w2l()` returns null for that case; no per-instance
reference to another receiver's state is implied. The project packager
`app.RQA.Ouu()` includes the set of nonempty individual state paths. Thirteen
empty paths in the new basic14 fixture therefore require independent capture.

For that focused question, the native external-file analogue is
`device.xuO.v33(File, base.async.Tl3)`: owner-thread checks, connected engine,
loaded state, no current state save, a Jbj external-output branch, explicit
`cSc(instance,path)` request and native watchdog. `device.msf.v33(xuO)` forwards
`Tl3.taskCompleted()`; its `(xuO,String)` error overload forwards `Tl3.v33(String)`.
Its no-state `TUp(xuO)` throws and must never be treated as successful empty output.
The external success branch differs from the internal `qz2` project-cache path.
`HUR.EwU()` identifies the native VST3 preset suffix `vstpreset`, exposed through
`Ixq.jaQ()`; this is a format check, not plugin-version evidence.

The existing owner-thread lifecycle/callback/retained-receipt pattern is reused.
The first successful file measurement remains bound to the operation. Offline
Rust/Java tests establish validation only. The subsequent controlled live probe
exported14distinct receivers and independently established factory equality for
all558parameters per instance, original default0dB velocity sensitivity, preserved
complete3984note/clip data, routes, mixer/automation/transport and all10836exposed
parameter states, and unchanged native project bytes. P2 survived an interrupted
client acknowledgment plus a Rust service restart/explicit reconnect by its original
operation ID, without replay. Native callbacks reported `main`. Refusal and
artifact-boundary evidence is indexed in
`controller/validation/import-evidence/native-state-export/`; this does not admit
sound or configured-performance completion.
The governing contract is the final receiver-state export subsection in
`controller/INTERFACE_DECISION.md`.

### Named render boundary and explicit cursor restoration, 2026-10-25

Wire26 reuses the proven native `pfG` render queue with semantic master/leaf
intent. `project render --master-project NAME` resolves root.DQP directly;
`--track-path PATH_JSON` resolves exactly one qualified instrument leaf, excluding
the canonical master even when its runtime type is also yyk. Native target handles
are diagnostic readback. Historical UUID/implicit-master requests remain readable
but cannot dispatch. Successful status requires the original controller lifetime,
source project and matching currently resolved semantic target. Terminal drained
work releases its busy guard even when a rename makes target matching fail.

Final fresh scaffold116/setup50 checks passed. Detailed evidence is indexed at
`validation/import-evidence/named-render/README.md`; controlled named rendering
is not service-free/basic-delivery or expressive completion by itself.


### Configured basic14 service-free playback, 2026-10-25

The initial recorder exit0 assertion failed on SIGINT exit1. Archived official
PipeWire1.6.9 src/tools/pw-cat.c do_quit1088 quits without drained;2658 grants
success only when drained;2678 calls sf_close on cleanup. Independently finalized
WAV validation establishes completion; no audio criteria changed or re-recording.
Source: https://github.com/PipeWire/pipewire/blob/1.6.9/src/tools/pw-cat.c .

### Arranger clip naming source trace,2026-10-25 (probe pending)

Basic14 main arranger labels are all1: the current ProjectConstruction61 names
tracks but omits clip names. Source precedent read first: documented Openwig bridge
commit d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d, clip.set_name line2034 calls
CursorClip.setName. The installed build's ClipProxy.setName/doSetName250–260
switches to document thread then invokes boi.mT_().g_(String). ApiClip.setTitle
invokes gnS.g_ inside its modify wrapper; do not call that wrapper on our owner
thread. fza.g_44–45 delegates AxB().v33(String); UMj.AxB216–217 returns FSQ.
Thus the setter writes shared content naming, not an independently isolated
instance field. Existing complete managed-source registry/exclusive_source guard
is required for this first scoped operation. This is primary implementation
lineage, not yet a behavioral admission.

Focused decompilation is under /tmp/notes-internal-research/clip-name27/, including
ClipProxy,ApiNoteOrClipEvent,UMj,I3W,boi,VrC,whT,fza; existing ApiClip is in the
shared primary-source directory. Proposed owned-fixture/probe and wire27 production
contract is in INTERFACE_DECISION.md. At this trace-only checkpoint no clip name
had yet been mutated for it. Subsequent behavioral results follow below.

Further focused clip-title trace: UMj→VrC→bT→XPB→gnS; FSQ note source is
Dxc→Bkg→SuS→x1G→Gvh→BIr. BIr.SRI reads its YEn String; final v33(String)
delegates TUp(String). Gvh.TUp opens a native "Rename Clip" change, delegates
the BIr string update and closes via qhW. BIr.qKA(String)/FHv(String) checks
nonnull/change, records the attribute change, assigns YEn and notifies listeners.
The prototype must reuse this owning rename context, not assume another manual
nested group is necessary. Raw getter value must be retained across Undo. Opaque
changed-source payloads are not independently decoded by the current observation;
claim observed musical preservation/untouched payload equality, not full unknown
source-byte equality. The complete callback/Undo behavior remains to be probed.

Fresh review falsified reusing replacement's curve-point-ID normalization for a
rename: changing pitch-point2190 to999999 incorrectly passed. The shared payload
projection now preserves those IDs; only replacement separately normalizes its
newly allocated curve-point IDs. The retained regression and fresh review's
counterexample now fail the rename oracle. All434 Rust tests pass,22 existing
live tests remain ignored; fmt, clippy-Dwarnings, doctests, MSRV1.87 and release
build pass. Both Java main/harness builds pass including the new typed resolver
checks. These results alone do not admit the production wire27 command.

A cross-track arranger attachment from P1 into P2 at288beats produced source19047
with the SAME content UUID and payload as source1382, but a DIFFERENT physical
managed source. This extends the earlier cross-track launcher copied-UUID finding;
it does not establish cross-track physical sharing. The independently observed
same-parent new-name collision refused without any complete-observation change.
Native Undo restored all pre-attachment payloads. A separately appended launcher
slot on P1 did observe two references (1346 arranger,20181 launcher) to source1382;
this is the live physical-sharing premise for the naming refusal. Remaining
refusal results and production admission must be recorded separately after their
actual execution; the original copied-source experiment is not relabeled shared.

Independent review narrowed the initial naming result: all14 receivers were
NOT_LOADED, and every plugin parameters array was empty. Exact receiver-chain
payload equality is proven;10836 live parameter states were not observed. The
post-deactivation count guard found0 and stopped before saving/closing. Retain
unloaded-evidence-gap01.json; loaded-receiver qualification remains mandatory
before production admission. The basic fixture has zero expression points, so
live populated-expression preservation is not claimed; the offline changed-ID
counterexample exercises that oracle boundary.

Loaded qualification completed all11 refusal cases, including actual loaded
launcher-backed source sharing and main-lane same-name ambiguity, with the
cross-track independent copy supplying the name-collision premise. Every
attachment was explicitly undone and complete pre-attachment payloads restored.
Final loaded Undo restored the complete pre-rename observation, including all
10836 parameter states, under the declared meter/revision/modified-flag scope.
The native modified flag is still true. behavioral-index02-loaded.json indexes
41 independently read loaded snapshots,19 exact comparison pairs,11 refusal
results and strict native health/count checks. Generic production command,
construction default naming and original main14-clip migration remain pending.

## Scoped arranger MIDI and client boundary — 2026-11-05

The [selected arranger clip reader](../../../controller/client-adapter/src/main/java/com/midiworkbench/notes/internal/EditorAdapter61.java)
reuses the existing note snapshot and
[owned project-property reader](../../../controller/client-adapter/src/main/java/com/midiworkbench/notes/internal/ProjectAdapter61.java).
Computed project properties exposed through `RR()` do not necessarily have an
`Nln()` attribute; the initial direct `getValue` approach failed with a null
owner. Reusing `ProjectAdapter61.value` resolved that boundary. Do not invent a
second property-access model for the same native owners.

The [MIDI contract and bounded acceptance](../../../controller/ARCHITECTURE.md#musical-scope-and-note-edits) distinguish
complete stored source curves from visible occurrences and populated lanes from
empty or preview material. The raw native graph and value-type metadata survive
the [pure projection](../../../controller/src/midi.rs). Knob labels are not evidence
of MIDI CC assignments; no automation write authority follows from a note revision.

The [architecture](../../../controller/ARCHITECTURE.md) separates external HTTP commands from
the SDK adapter session. The published SDK determines incoming framing and callback
mechanics; JSON, compression and correlation remain application choices. The
[retained native callback evidence](../../../controller/validation/native-transport/README.md#actual-handler-and-thread-evidence)
supports the document-dispatch route, not undocumented vendor architecture intent.
