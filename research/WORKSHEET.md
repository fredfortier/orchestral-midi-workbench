# Boundary Sketch worksheet

**Run** 2026-11-22, Bitwig Studio 6.1 revision 162652, native controller wire 74,
and the workspace `osc` harness bound to loopback UDP 9000. DrivenByMoss
received commands on UDP 8000. The [live archive and manifest check](evidence/06-installed.txt)
identified DrivenByMoss 26.6.5. The generated Bitwig OSC reference contained
empty method lists. External scope below refers to the pinned documentation
and the installed release's source.[^run][^osc]

**Fixture** One new project, `Boundary Sketch`, saved under
`tmp/Boundary Sketch/`. An arranger call ascends C–E–G and a launcher answer
descends G–E–C, both one bar long. A second instrument track provides a
multi-selection comparison. The CC11 lane and empty receiver slot exercise
storage and routing. No instrument patch was loaded, so audition checks
establish transport and launcher state only.

## 1. Find the answer

**Aim** Distinguish the document's owners from the remote's bank view.

**Setup and steps** Use `controller info` and `midi get-all --paths` for exact
clip paths. With the listener bound, send OSC `/refresh`. Send
`/track/add/instrument` once and compare native structure before and after.

**Result — observed.** OSC identified the project and banked tracks and exposed
launcher slot occupancy. The native entity read identified the arranger clip,
placement and lane metadata. The native content read distinguished the launcher
answer, arranger call and stored CC11 points. OSC track add created one native
instrument track, `Inst 3`. A complete qualified hierarchy and stored event/curve
read are not documented as OSC operations. This probe used unique labels.[^osc][^native]

**Evidence** [Native entities](evidence/01-entities.json), [stored content](evidence/03-final-content.json),
[refresh feedback](evidence/01-live-refresh.txt), [track-add comparison](evidence/01-osc-add-track.json).

## 2. Jump and select

**Aim** Select two owners for a quick action, then target the expression lane.

**Setup and steps** Use fresh `selection read` and `selection set --input`
with track IDs 2720 and 393, primary first. Send `/track/1/select i:1` after
checking the bank. Set the lane's time range to arranger beats 1–3. Submit a
deliberately incompatible track-plus-lane request in object mode.

**Result — observed.** Native readback contained both tracks with 2720 primary.
The OSC select produced only track 393 in that starting context. The native
time request selected lane 3124 with exact 1–3 bounds. The mixed object request
returned `Unsupported object selection target` before dispatch. Complete
selection readback was unchanged. Exact arbitrary membership, a primary
designation and qualified lane selection are not documented OSC commands.[^osc][^native]

**Evidence** [Multi-selection request](evidence/02-multi-request.json) and
[readback](evidence/02-multi-readback.json), [OSC selection readback](evidence/02-osc-select-readback.json),
[lane request](evidence/02-lane-request.json) and [readback](evidence/02-lane-readback.json),
[mixed-object refusal](evidence/02-mixed-refusal.txt).

## 3. Sketch, audition and revise

**Aim** Compare deterministic placement and editing with performance controls.

**Setup and steps** Select an empty launcher slot in Bitwig, then
`launcher read` and `launcher create --expected … --beats 4`. The arranger
container was seeded through the UI before checking all programmatic options.
Write both motifs with one
`midi replace` and verify with an exact-path `get-all`. Add CC11 with
`automation set`. Click the call's middle note in the Detail Editor, confirm
the one selected key, then `note transpose --semitones -1` with fresh authority.
With the fixture engine active, send OSC seek, indexed launch, stop, indexed
slot select, indexed create, and stop. The attached transcript records arguments.

**Result — observed.** Native launcher creation produced an empty four-beat
clip without starting transport. Native content readback showed both motifs.
CC11 creation preserved the arranger clip, and the selected-note edit changed
only E to E-flat while preserving the automation. OSC seek reached bar 2
in the 4/4 fixture. Launch started playback, and stop reported `/play 0`.
OSC slot 2 create produced a second four-beat clip, started playback and
enabled launcher overdub. The following stop reported playback and overdub
off. OSC had reported slot 2 selected while `launcher read` still returned
`position:null`. Clicking the slot in Bitwig exposed the native Inspector slot
and made the created clip available for explicit readback. This was observed
from an arranger lane time selection. **Exact empty arranger creation remains unresolved.**
The UI-seeded clip let the note-editing probe proceed. It supplied no evidence
that the UI was necessary. The reviewed controller CLI exposes launcher
creation and content replacement. The OSC reference documents launcher
creation and configured actions. Before assigning arranger creation to a
surface, check a configured action against exact empty-clip placement.
The follow-up below tested a duration change. Split and move remain
untested.[^osc][^edit]

**Evidence** [Empty slot](evidence/03-empty-slot.json), [native create](evidence/03-created-slot.json),
[CC input](evidence/03-cc-input.json), [transpose readback](evidence/03-transpose.json),
[OSC exchange](evidence/03-osc-audition.txt), [native read of the OSC-created clip](evidence/03-osc-created-native-clip.json),
[final stored contents](evidence/03-final-content.json).

## 4. Reuse a receiver

**Aim** Converge on a named, channel-qualified receiver path.

**Setup and steps** Run `instrument layer ensure` for Sketch Instruments.
Run `instrument slot ensure --name Answer --input-channel 2`. Recover from
the stale track name using the returned target and run both ensures again
with distinct operation IDs. Compare independent `instrument read` results.

**Result — observed.** Initial Layer ensure auto-renamed the default track,
keeping native track ID 393. Slot ensure with the old path was refused before dispatch. With the returned target, the two routing observations matched.
They contained one named slot in All → Same, an enabled stock Filter admitting
displayed channels 1 and 2, and an enabled Map sending all inputs to channel 1,
with the same device and slot identities in both runs. OSC browsing, plug-in
insertion and device/layer actions are documented only here. The checked OSC
surface does not describe a comparable desired-state graph or exact NKI
assignment. NKI loading, arbitrary chain reconciliation and audibility were
outside this check.[^osc][^setup]

**Evidence** [Original target](evidence/04-layer-target.json), [successor target](evidence/04-layer-returned-target.json),
[refusal](evidence/04-target-refusal.txt), [routing readback](evidence/04-routing.json).

## 5. Work at the boundary

**Aim** Identify computer steps a remote action can shorten.

**Setup and steps** Start with the Detail Editor showing the selected E-flat.
Send `/panel/noteEditor` to close and again to reopen. Observe panel visibility
and read native editor state each time. On the dirty test document, send
`/project/save` and check the saved file and native content before and after.

**Result — observed.** The panel closed and reopened. The native editor changed
from `detailEditor` to `none` and back. That evidence establishes panel
visibility and editor context. Keyboard delivery to the reopened panel was
not tested. OSC Save advanced the fixture file's mtime and changed its hash,
and the three clips and automation matched before/after. Reopening from disk
was not exercised. Window targeting, actual focus for named actions, and
verified menu actions such as Rename and lane-header cleanup remain computer
work under the current production skills. The follow-up below exercised direct
OSC history, indexed track rename and a configured action slot.[^osc][^keyboard]

**Evidence** [Panel readback](evidence/05-panel.json), [Save command and comparison](evidence/05-save.txt),
[final stopped feedback](evidence/01-live-refresh.txt).

## 6. Programmatic follow-up

**Aim** Replace assumptions from UI-seeded probes with verified programmatic
routes and explicit limits.

**Setup and steps** Reuse the stopped fixture. Use its second instrument track
and unused timeline intervals. Probe OSC arranger recording, indexed rename,
device insertion and history. Independently read stored state through the
controller. Test a native content-replacement payload that changes only clip
duration. Inspect the installed action catalogue and the matching extension
source before considering empty arranger creation.

**Result**

- **Out-of-scope recording observation.** Arm only the target track, seek, send `/record`,
  start `/play i:1` when record is armed but the transport is stopped, send
  note-on/off pairs, and stop. The controller found three pitches in a new
  arranger clip at beat 8. Its duration was about 1.914 beats. Repeats at beats
  16 and 24 also created clips. The original instrument track's CC11 curve
  acquired three additional points at each tested recording span, even with
  that track disarmed. `/record` enabled Arranger automation write on this
  host. Turning it off before playback still left the curve side effect.
  Native replacement restored the original curve. This result carries no
  weight in choosing a document-centric clip creation or editing route.
  [OSC exchange](evidence/06-record-osc.txt) · [Recorded clip](evidence/06-recorded-clip.json) ·
  [Repeat, guard and restoration](evidence/06-preservation.txt).
- **Geometry refused.** Native `midi replace` rejected the duration-only
  change as `NotDispatched`. Fresh readback matched the original clip and
  automation. The host has a named **Create Empty Clip** action. DrivenByMoss
  26.6.5 dispatches an assigned action via `/action/N`. The tested slot's
  saved labels suggested Undo but the command produced no observed effect;
  direct `/undo` removed the known last insertion. No effective empty-clip
  assignment was established. Exact empty arranger creation remains an open
  OSC qualification, and split, move and resize are separate questions.
  [Refusal](evidence/06-geometry.txt) · [Installed action](evidence/06-installed.txt) ·
  [Action dispatcher](https://github.com/git-moss/DrivenByMoss/blob/7348cda4e5ae8aebd59bb99b8d967a1f4b87677e/src/main/java/de/mossgrabers/controller/osc/module/ActionModule.java#L53-L74) ·
  [Configured action and direct control](evidence/06-configured-action.txt).
- **Launcher selection bounded.** Indexed OSC slot select left native
  `launcher read` at `position:null` from the Detail Editor context and
  again after the native controller selected the intended track. This adds
  two observed starting states to exercise 3. Other contexts remain open.
  [Selection and launcher readback](evidence/06-slot-navigation.txt).
- **Rename, device insertion and history observed.** OSC renamed track 3,
  creating duplicate display names. Both qualified paths resolved their
  original IDs and the name-only path returned an ambiguity error. OSC
  restored the name. Two `/eq/add` calls produced two EQ+ identities. Direct
  OSC Undo/Redo also reversed and reapplied one known native note edit with
  complete target readback. Store exact native scope and a known history
  boundary before using that route.
  [Duplicate-name probe](evidence/06-rename.txt) · [EQ insertion](evidence/06-device.txt) ·
  [Undo and Redo](evidence/06-history.txt).

**Evidence** Each probe above names its input, independent observation and
limit. The fixture was saved through OSC after native verification of the
restored original content. This follow-up used controller and OSC throughout.

## 7. Arranger document navigation

**Aim** Determine whether OSC can replace native arranger inventory or exact
clip selection, and where it can reuse an already-resolved target.

**Setup and steps** Read three arranger clips on one track with native
`info`. Compare that inventory with OSC `/refresh` and the installed
release's clip command implementation. Select the first clip by native path.
Seek to the second clip's recorded start through OSC `/time f:16`. Compare
transport feedback and native selection. Then select the second clip by
native path and reread.

**Result** Native inventory returned the three clips at beats 8, 16 and 24
with qualified paths, durations, source bounds and loops. OSC indexed clip
feedback described launcher slots. Its cursor step implementation also
addresses the launcher slot bank. The reviewed release documents no
equivalent arranger inventory. Seek reached bar 5 with transport stopped
and left clip 4895 at beat 8 selected. Native selection then selected clip
9573 at beat 16. Use the same resolved path and beat for the two intents.
Reveal and keyboard focus were outside this check.

**Evidence** [Inventory, seek and selection](evidence/07-document-navigation.txt) ·
[Installed cursor and slot command implementation](https://github.com/git-moss/DrivenByMoss/blob/7348cda4e5ae8aebd59bb99b8d967a1f4b87677e/src/main/java/de/mossgrabers/controller/osc/module/ClipModule.java#L106-L158).

## Procedure critique

- **Explore** Start with the complete scoped content read when the clip path is
  already known. Use structure for discovery, selection for user-visible context
  and a focus step only when the next action needs it. A launcher workflow
  should hand off at **native** `launcher read` readiness. OSC bank selection
  alone failed that handoff in exercise 3.
- **Edit** Keep the explicit-path replacement and selected-note branches
  separate. One complete payload carried both motifs. The one-note transpose
  needed only the effective selected-note snapshot, its fenced operation and
  independent reread. Give arranger-container creation its own capability
  check before assigning it to any surface.
- **Setup** Put the returned target immediately between Layer ensure and slot
  ensure in the command example. The initial default-track rename made the
  original path invalid, even with the same native ID.
- **Remote** Qualify seek, launch, stop and Save first, followed by known-history
  Undo/Redo and indexed track rename. Keep
  project/slot identity and native verification at each handoff. Treat panel
  visibility, Inspector selection and keyboard focus as separate checks.
  The listener is a manually started standalone process. Its PID and raw
  log are in ignored `tmp/` alongside the saved fixture.

[^run]: [Generated reference inspection](evidence/01-generated-reference.txt)
    and [live refresh](evidence/01-live-refresh.txt) establish the local OSC
    observations. The host revision came from Bitwig's last-run metadata and
    the wire version from the active service's `/v2/health`. [Run evidence](evidence/00-run.txt).
[^osc]: [Pinned DrivenByMoss reference](https://github.com/git-moss/DrivenByMoss-Documentation/blob/8aab247a5643082f9706b36c6fd091ab7125aae4/Generic-Tools-Protocols/Open-Sound-Control-(OSC).md).
    Both directions were checked, including transport, layout, track, launcher,
    browser, virtual MIDI, markers and configured actions.
[^native]: [Native structure and selection](../crates/controller/ARCHITECTURE.md#paths-and-structural-reads)
    and the [navigation recipe](../skills/bitwig-explore/references/arranger-navigation.md).
[^edit]: [Launcher and replacement recipes](../skills/bitwig-edit/references/common-edits.md)
    and [supported production operations](../skills/WORKFLOW.md#supported-production-operations).
[^setup]: [Integrated instrument contract](../crates/controller/ARCHITECTURE.md#integrated-instruments)
    and [assignment procedure](../skills/kontakt-setup/references/assignment.md).
[^keyboard]: [Qualified keyboard workflow](../skills/bitwig-explore/references/keyboard-workflow.md).
