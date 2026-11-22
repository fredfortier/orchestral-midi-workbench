# Qui tollis composition exercise

This exercise tests the composition routes against a new saved Bitwig project
created from the supplied Bach BWV 232 movement. It pairs historical evidence
from the [composition stories](composition-stories.md) and
[control research](CONCLUSIONS.md) with independent live observations. The
project's `evidence/` directory holds full native reads. The repository retains
summaries and replay criteria so it does not need to package the score,
installed samples, or project files.

## Environment and fixtures

- Bitwig Studio 6.1 revision 162652, DrivenByMoss 26.6.5, bridge API 2 and
  native wire 74. The isolated project is named **Bach BWV232 - Qui tollis
  Composition Smoke**. The original fixture remains a separate project.
- The existing listener on the native port was replaced by the OSC-enabled
  bridge after confirming the standalone listener's PID and logfile. The active
  bridge owns the existing native/HTTP listeners and loopback UDP 9000. The
  extension receives commands on 8000. One `/refresh` returned the new project
  and stopped transport, and native `info` returned the exact saved path.
- A separate loopback peer verified the typed wire bytes for `i:-2 f:1.25
  's:a b'`, service source port, filtered bundled feedback, omission of a
  missing address, malformed feedback handling, rejection before dispatch
  of an invalid float, and busy refusal while observing. SIGTERM exited
  cleanly. No external OSC listener participates in the live route.

## Intake

**Prompt.** Admit the supplied MXL into a new disposable production workspace,
preserving its original bytes.

**Observation.** The displayed File menu supplied **New Project** and **Save**.
The first Save displayed a separately managed GTK chooser. Binding the chooser
window itself resolved the clipped parent-window view and verified the
configured Bitwig Projects root. The native read returned the live project at
its exact saved path. Copy comparison and both SHA-256 values matched
`f0fe9dbfe9c07971ab4cfafdd734e96ca7042af12908e648cd37439d4ccdd5f2`.
`SOURCE.md` identifies the supplied file and the embedded MuseScore metadata.
The fresh `PERFORMANCE.md` records the native path and source receipt.

**Iteration.** Distinguish a first save destination dialog from saving an
existing native path. Preserve the source as opaque bytes even when a later
converter pass reads its metadata. The chooser can be a distinct window.

## Configured actions and placement

| Probe | Live observation | Procedure consequence |
| --- | --- | --- |
| Slot 1, Create Empty Clip | No target left native structure unchanged. A native [8,12] range with editor `none` also left clips unchanged, including after audio-engine activation. With the same range and editor `arranger` established by slot 5, one clip appeared at beat 8, duration 4, source [0,4], looping off and zero notes. Transport stayed stopped. | Exact range and actual Arranger focus are separate preconditions. Qualify a following write against the resulting clip path. |
| Slot 1, compatible range batch | One exact [0,153] selection over two instrument tracks created a clip on each track. Audio and FX tracks acquired none. | One selected range and one action can supply shared full-movement placement. Independently check each owner. |
| Slot 2, Zoom Arranger to Selection | Native selection retained [8,12], while the ruler changed from measures 1–4 to the selected measure 3 and its subdivisions. | Use the viewport for display evidence. Native selection alone does not prove framing. A later empty-selection comparison preserved the viewport. |
| Slot 3, Toggle Automation Lanes for Selected Tracks | With Inst 1 as the exact primary track header, dispatch revealed its Volume lane row and left Audio 2 closed. Scoped structural read stayed identical. | Check disclosure visually. Select track headers for predictable scope. A later empty-selection toggle affected the last track. |
| Slot 4, Split | Range [12,14] left the notes, expression and curve array unchanged. Range [10,12] split [8,12] into placements at 8 and 10 of duration 2 each. Source windows became [0,2] and [2,4]. Both stored sources contained the same two note values, including right-note pressure 0.5. The right copy received fresh IDs. | Count audible occurrences within source windows. Counting all source notes after a split would double the result. Only this cut/geometry is qualified. |
| Slot 5, Focus/toggle Arranger Timeline Panel | With an unfocused visible Arranger, preserved [8,12] selected range and changed editor to `arranger`. Invoking while focused hid the timeline and returned `none`. Invocation while hidden showed it and returned `arranger`. | Observe before invoking. An already focused timeline requires zero actions. |
| Session history | Direct Undo removed the split and restored matching stored notes and expressions. Redo recreated both placements and complete content. Three individually checked Undo actions removed the remaining disposable split/seed/container changes. | Use direct OSC history only with the known adjacent document edit and read back each transition. |
| Initial meter | At beat zero of the fresh project, `/time/signature s:3/4` yielded `3 / 4` on a fresh peer report. | Verify initial global timing independently before treating it as part of the score. Boundary-local meter and tempo remain distinct probes. |

**Configuration correction.** Bitwig lists slot 5 under **Panel Management**.
The proposal named **Project**, which instead contains transport and track
actions. Slots 2–5 were configured through the active Bitwig controller
profile. The standalone sender was retired from current procedures.

**Editing friction.** The first fixture seed built flattened note objects from an
old evidence projection. The current CLI rejected the file before dispatch.
A complete native NoteEvent inside `{selected,note}` worked, including a
pressure point. Production documentation needs a minimal complete example
and an explicit source-beat conversion at the owner, so score placement does
not depend on reconstructing schema from old receipts.

## Full-score placement and cue study

- Created ten additional instrument tracks through the indexed session surface.
  After creation, the OSC bank had scrolled to Inst 9–13. Queried the bank
  rather than deriving indexes from track order. Renamed those five, paged
  back, checked the eight names (including the template's audio track), and
  renamed the first seven musical tracks. Native IDs established the exact
  owners. One shared range action created the ten missing [0,153] clips;
  the existing two were reused. The Audio and FX tracks stayed empty.
- Authenticated the source conversion and retained score-note provenance.
  Decode yielded 2,173 events, duration 152.5 beats. The organ had two encoded
  source routes and seven simultaneous same-pitch pairs. Collapsing by track
  alone would have lost that distinction. Preserved both native channels.
  Quantized source onset/end once, maximum error 0.000000344 beats. One batch
  replacement took about 1.50 s. Independent complete read took about 0.35 s
  and matched clip geometry and every decoded pitch, channel, onset, duration,
  velocity and release velocity. No curves existed in this scope.
- The first well-shaped note batch omitted expression families and got a Java
  field error with outcome unknown. Independent readback proved all 12
  containers were still empty. Supplying all six arrays succeeded. The changed
  Rust validation refused the same missing-family defect before dispatch
  against a fresh live clip snapshot. The editing reference now includes the
  full note shape and the placement reference specifies conversions.
- The fresh native capture established 3/4 and elapsed time consistent with
  quarter = 55 throughout. Setting 2/4 and 50 BPM at beat 144 changed the
  native meter and time-to-beat relation at beat zero and across all 153 beats.
  Restored 3/4 and 55. These OSC operations qualify for initial global timing
  and must be excluded from local transitions.
- Paired source text and harmonic evidence with independent native notes.
  René Pérez Torres's analysis of Bach's use of BWV 46 supplies the
  tonic-to-dominant structure and the B-to-F-sharp upper line. The project
  reserves the principal performance intensity for the late choral
  supplication. Eight cue names and beats matched independent controller
  readback. Saved through OSC after both note placement and cue creation.
  [Pérez Torres, pp. 17–30, especially example 1.8](https://digital.library.unt.edu/ark:/67531/metadc4945/m2/1/high_res_d/thesis.pdf)

The project contains an editable notation rehearsal. Its plain converter
artifact retains the source score information, including late tempo events
and lyrics. The native rehearsal has constant tempo and is an intermediate
deliverable for the receiver, editing and listening exercise.

A later comparison derived part ownership from the MXL part list and
companion provenance, independently of the native replacement payload.
Flauto traverso I and Violoncello share encoded route 0/0 while Organo uses
two routes. The source-aware multiset matched all 2,173 occurrences in both
the independent note read and the later export capture, including destination
part and channel, pitch, native-grid start and end, and attack and release
velocity. The later cue read still matched all eight intended names and beats.
This strengthened the production placement recipe with a provenance check
for notation rehearsals.

## Receiver and Launcher handoff

- Violino I Layer and slot ensure reused the returned Layer target. Both
  repeated ensures preserved IDs, Channel Filter and Channel Map. Native read
  independently matched. Mutation receipts and independent observations have
  different JSON nesting, now shown in the assignment reference.
- The integrated loader inserted the exact Kontakt instance, then failed
  before showing its owned Load dialog. The retained owner records and loader
  process exit were checked before recovery. The first render was visible
  but the subsequent immediate screenshot was black. The loader now waits
  for a visible post-preparation render and uses the existing typed protocol
  for provable pre-submission dialog failures. A legacy receipt requires the
  archived producer to establish that the exact dialog failure preceded
  submission.
- OSC selected Violino I Scene 1 in its bank while native `launcher read`
  still reported no Inspector position. One computer click on the empty
  slot cell selected it natively. A subsequent create correctly refused
  while the unknown external assignment remained fenced.

- The retained pre-load receipt passed the revised continuation gate. The
  subsequent native/Win32 visibility challenge could not hide the same
  Kontakt window. Bitwig reported an unresponsive plug-in host, and both
  ordinary Close with Save and reversible OSC engine deactivation left the
  project and engine active without advancing the saved file. Only the earlier
  score/cue saves are proved. The continuation stopped before claiming or
  submitting the NKI. Shared-engine termination was considered before the
  orphaned-dialog recovery below.
- The failed save eventually showed an explicit Kontakt state timeout. After
  dismissing the prompt without proceeding, the native clip operations
  responded despite the audio-engine alert. A fresh empty Scene 1 observation
  created an eight-beat Violino I Launcher clip. Its first candidate reused
  the eight notes from arranger beats [60,68), rebased their onsets to clip
  coordinates, and applied a small attack-velocity arch. Independent readback
  matched all eight candidate events. The 12 Arranger clips, complete note
  values, expression and automation matched their prior snapshot. The bank
  reported Launcher content while transport remained stopped.
- With the Bach project active, recording off and the corresponding bank
  slot occupied, OSC launch set both slot playing and global play to 1
  and advanced the beat. Stop and the Arrangement return command left the
  transport and slot stopped. The OSC bank's clip-name field was blank for
  native default name `S1`. A seek to beat 60 reported bar 21 beat 1 in
  3/4 without changing the native Launcher selection. A fresh mixer read
  joined cues 60 and 141 to 65.454545 and 153.818182 seconds. The 12
  complete Arranger clips matched the pre-audition snapshot.
- An explicit attempt to create into occupied Scene 1 returned
  `InvalidRequest / NotDispatched`, and an independent read still matched the
  exact Launcher clip, its geometry and all eight notes. These checks establish
  transport and edit mechanics. Audible audition and promotion remain open.

### Wrong-context display probes

- Clearing the unsupported Launcher selection by sending empty native paths
  refused before dispatch. Selecting the exact Violino I Arranger clip worked.
  Clearing from the Detail Editor did not clear the Arranger's own saved
  selection. Slot 5 restored the Arranger selection on focus. Clearing again
  with the Arranger active produced a confirmed empty target.
- Slot 2 with that empty Arranger target preserved both the ruler framing and
  the independent native selection.
- Slot 3 with the same empty Arranger target revealed Violino I's Volume lane
  while leaving native selection empty. The earlier focused track retained
  influence over the action. Explicit exact header membership is a production
  prerequisite, even when a null primary otherwise looks harmless.

### Dialog recovery and disk verification

- A later native window inventory exposed one visible `Load` dialog.
  Observation-only Win32 inspection matched its owner HWND and PID to the
  retained Violino I loader request. Its filename was empty. Canceling only
  that dialog released the queued project chooser and cleared the plug-in
  alert. No patch path was supplied or submitted. No shared-engine reset
  was needed for this recovery.
- A byte-identical copy of the earlier save opened in Bitwig without
  activating audio. Independent native reads matched all 12 Arranger clips
  and 2,173 notes, plus all eight cue names and beats. Native identifiers
  were regenerated and were excluded from the musical value comparison.
- After returning to the original tab, the visible project and the remote
  controller cursor disagreed. Native `info` and fresh OSC feedback pointed
  at another project. The visible **File → Save** action changed the original
  project file from SHA-256 prefix `e670b80c` to `ea2ac435`. No remote write
  was issued against the mismatched target. A fresh copy of the latter bytes
  opened in Bitwig and the exact Scene 1 Launcher read matched all eight
  notes, clip geometry and loop settings. This verifies the saved data.
  Closing and reopening the original path and retrying its integrated
  assignment remain open.
- The saved copy also reopened with the Violino I Channel Filter, Channel
  Map and Kontakt 7 shell. Activating audio once from the verified project
  changed the native engine state from `NOT_CONNECTED` to `CONNECTING`.
  Subsequent native reads still reported `CONNECTING`, fresh OSC feedback
  reported engine `0`, and the window inventory showed no plug-in dialog.
  The assignment requires a connected engine. No new NKI load was dispatched.

### Occupied placement and append

- In the saved candidate copy, slot 1 on occupied Violino I beats [60,68)
  split the original into [0,60) and [68,153) and inserted an empty clip
  between them. Focus and correct range were insufficient when the range
  contained music. One known-adjacent Undo restored the full original
  geometry and all 2,173 notes. The production recipe now calls out the
  replacement consequence as well as the required empty-range check.
- An adjacent empty range [153,159) on the same track produced one clip
  without touching the prior close. Copied the eight-note figure from
  [63,69) into that clip's source coordinates and independently verified
  position, values and preservation of all 12 preceding clips. Native
  timing samples confirmed 3/4 and 55 BPM across the join. Two separately
  verified history operations reversed the insertion and clip creation.
  This qualifies unchanged-pulse append mechanics. Local timing changes
  and the audible join remain separate.

## Fresh reverse export

The full live scope captured 12 clips and 2,173 notes without exclusions,
including the track with its inserted Kontakt shell. Direct assembly retained
the ambiguous Sustains slot as a remark. MIDI2 matched 12 parts and 2,173 notes
and passed admission. The reconstructed 12-part MXL passed independent
MusicXML 4.0 XSD validation. Capture took 0.35 s, assembly 0.16 s, admission
0.004 s, and notation conversion 0.13 s in this run. These measurements are
tool wall time, exclusive of interpretation and schema checking. The
reconstructed score represents captured native state and its notation losses.
