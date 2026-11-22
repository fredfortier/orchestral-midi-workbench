# Score-to-Bitwig workflow

This contract governs the task skills below. The user's requested scope controls
which tasks run. Each skill returns after its own result; dependencies return to
their caller. A multi-stage request composes these tasks without a new approval
at every handoff. Missing capability does not authorize a workaround or work outside
the owning existing operation. Complete independent musical work and report the
affected scope when a tool cannot perform a required action. Use only the owning
production skills, project/library documents, musical sources and current tool
help to carry out these workflows.

## Bitwig project layout

- The project root is the directory containing the saved `.bwproject`. Keep `PERFORMANCE.md` there.
- Keep the original score and its provenance record in the project's established source location. For a new project without an agreed source location, use the project root.
- Use project-root `references/` for external listening references and `evidence/` for new evidence. Honor the project's existing evidence location.
- Preserve existing project files and paths. Create folders as needed and leave `recordings/`, `master-recordings/`, and other DAW-managed directories to Bitwig. Follow the [checkpoint procedure](#experimental-project-backups) for manual backups.

## Experimental project backups

Before an authorized experiment on an existing project, save the current project
through [session actions](bitwig-explore/references/session-actions.md#direct-actions)
and confirm completion. Resolve its actual saved `.bwproject` path with
`controller info`; do not copy an older on-disk state while edits remain unsaved.

Keep manual checkpoints in `backups/` beside the main project file:

```text
<project>/
  <name>.bwproject
  backups/
    <name>.20261118T214600Z.before-syllable-builder.bwproject
  auto-backups/
```

Use the actual project stem, current UTC timestamp and a short experiment label.
Copy only the saved `.bwproject` into this folder, as a single-file checkpoint
analogous to Bitwig's auto-backups. Do not copy the project directory, assets,
samples, exports or companion files. Leave Bitwig's `auto-backups/` untouched.
Choose a fresh filename, never overwrite an existing checkpoint, and verify the
copy is byte-identical to the saved main file before starting the experiment.
If saving or copying cannot be confirmed, stop dependent mutations.

Continue working in the open main `<name>.bwproject` at its original path; it is
the experimental fork. Do not use Save As to move work into the backup or create
a second project workspace. Record the checkpoint's relative path in the existing
project evidence. Further checkpoints follow the same convention.

This checkpoint restores project-file state only; external files remain shared.
Keep required external resources in place and preserve them under their owning
workflow. If rollback is requested, preserve the experimental main file as another
checkpoint, restore the selected backup to the original main path without an
open document overwriting it, then reopen and verify the restored project through
the qualified native workflow.

## Mandatory execution boundary

Use the [editing skill](bitwig-edit/SKILL.md) for notes, expression and
automation and the [assignment skill](kontakt-setup/SKILL.md) for Instrument
Layer and Kontakt operations. If a required operation fails, preserve current
musical state and reconcile its outcome before further edits.

For mixing observations/balance and native audio export, use the explicitly
admitted [mixing](bitwig-mixing/SKILL.md) and [export](bitwig-export-audio/SKILL.md)
operations. This extension does not restore general project control.

For exact native selection, use `controller selection read/set` through
[exploration](bitwig-explore/references/arranger-navigation.md). For surrounding
project/window/focus/audition/history actions, use the
[shared session procedure](bitwig-explore/references/session-actions.md).
It routes qualified OSC actions and the remaining computer actions. Native
observations establish document effects. Computer observations establish actual
window, viewport or focus effects where needed. Keep surrounding actions out
of the native document command vocabulary.

Stock instrument assignment has two owners:

* `controller/` owns idempotent Instrument Layer and named-slot materialization,
  channel filtering/remapping, native MIDI automation and explicitly requested
  device-parameter automation, and Kontakt instrument assignment. Assignment
  automatically opens the intended Kontakt window and invokes the loader as
  one existing integrated operation.
* The repo-owned `controller/adapters/kontakt-ctl` has one job: load the exact NKI
  into that existing Kontakt window. It does not create/select host structure,
  launch a replacement player, customize instruments, or change receiver settings.

After stock assignment, [kontakt-syllable-builder](kontakt-syllable-builder/SKILL.md)
owns authorized Syllabuilder dashboard configuration through computer use in the
exact open editor. Its screenshots establish Kontakt UI state, not native Bitwig
note/routing state. It returns phrase definitions and verified controls to the
existing realization/edit owner. This adds no loader, native editing operation
or kontakt-ctl customization responsibility.

Use the [integrated assignment procedure](kontakt-setup/references/assignment.md)
for the exact receiver and NKI.
Conventional MIDI controls use native **track-level MIDI automation** by default:
MW CC1, BC CC2, XP CC11, other documented CCs, and pitch bend (a distinct MIDI
message, never a CC number). Read the exact patch's control table in
INSTRUMENT.md; missing or stale facts route to instrument-intake documentation-only
review. Downstream tasks do not guess mappings or parse manuals during execution.

Standard slot anatomy is **All → Same**, containing stock **Channel Filter →
Channel Map → instrument**. Slot ensure requires a musical name and complete note
input-channel set, reapplies these defaults, and selects/verifies the slot in
controller code. The Filter enables shared channel 1 plus those note channels;
the Map sends all channels to 1. Articulated parts reserve channel 1 for shared
CC and use 2–16 for musical/keyswitch notes. A nonarticulated single-channel part
may use 1 for notes and controls. Multiple admitted note channels share one
receiver's state. Use one channel-1 native track lane per shared control and verify
actual receiver response. Failed passage routes to setup/routing repair, not
copied curves, sender devices, macros or Grid broadcasters. Preserve existing
customizations unless their standardizing operation is requested. Assignment
independently inserts/reuses the instrument and preserves surrounding devices;
it does not require this topology. See [assignment](kontakt-setup/references/assignment.md).

Device parameter/Remote Control discovery is an explicit opt-in for a requested
control beyond the documented conventional MIDI palette. It is not an automatic
fallback for missing MIDI support. Use `controller automation read --target
device.json`, then the parameter form of `automation set` only for that explicit
scope. Legacy MIDI CC device discovery is also admitted for an authorized migration:
read actual assignments and channel, translate physical units, preserve curve
shape and receiver behavior, verify direct MIDI lanes, then remove redundant
senders and their automation through the qualified native cleanup below. Never
infer an assignment from a knob index.

For an authorized legacy migration, whole-sender/whole-lane deletion may use
[qualified native cleanup](bitwig-explore/references/keyboard-workflow.md#automation-lane-and-sender-cleanup), with exact owner selection and
independent native before/after reads. Curve creation/editing stays with the
controller; deleting a sender must preserve its receiver and unrelated content.

### Expression control specification

Before realization or setup, obtain each exact patch/articulation's documented
default from instrument-intake's INSTRUMENT.md and specify its project use in
PERFORMANCE.md. Keep that default: velocity-controlled dynamics require note
velocity; dynamic crossfade requires its documented controller, commonly CC1.
When CC11 independently controls level, plan it separately from sampled dynamics:
use it for the part's broader phrase envelope, foreground/support relationship
and releases while CC1 retains the intended timbre. Do not default CC11 to 127,
copy the CC1 curve, or generate CC11 by scaling/smoothing CC1. A deliberately
level passage may hold a chosen value; justify that musical decision rather than
applying a global maximum. For control planning, read
[dynamics and expression](bitwig-edit/references/dynamics-and-expression.md).
A verified CC11-only patch uses that supported control without an invented CC1
lane. Preserve documented velocity roles such as attack/transition selection.

For singing, apply the reference's vocal-intensity guidance: author each line's
documented dynamics through its phrases and melismas, separately from expression
gain. A missing calibration is not a missing controller assignment. After a
patch change, reconcile the controls required by the new receiver; preserved old
automation and successful syllable/HLS delivery do not establish complete dynamics.

Record the sampled named dynamics and the numeric velocity/CC ranges or anchors
for p, mf, ff and other required levels, with units and manual revision/section.
Use labelled approximate anchors and interpolation when exact calibration is
unnecessary; distinguish these editorial targets from vendor layer bounds and
measured calibration. Specify CC1/velocity timbre targets and CC11 level targets
separately, including their passage anchors and interaction. Do not equate
attenuation with a softer sampled timbre or claim a universal CC-to-dB law.
Missing control-function evidence leaves that branch unresolved; preserve its
current behavior unless the user authorizes a labelled provisional interpretation.
Verify native MIDI lane identity/channel, receiver routing, initial values and
actual track automation readback before claiming execution. A
missing required operation routes to its controller owner, then returns to this
task with fresh observations; it does not authorize an alternate control surface.

## Route by outcome

```text
REQUEST
 +-- compose across workflows --------> bitwig-compose -------> STOP after requested stages
 +-- establish score workspace ------> score-intake ----------> STOP
 +-- document installed library -----> instrument-intake -----> STOP
 +-- configure vocal syllable phrases -> kontakt-syllable-builder -> STOP
 |    +-- missing patch facts --------> instrument-intake --> return
 |    +-- stock receiver assignment --> kontakt-setup -----> return
 |    `-- native note/control edits --> bitwig-edit -------> return
 +-- realize/revise score -----------> score-realization
 |    +-- missing workspace ---------> score-intake -------> return
 |    +-- missing relevant docs -----> instrument-intake --> return
 |    `-- specify -> convert -> native import -> verify ------> STOP
 +-- research/place cues ------------> performance-cues ------> STOP
 |    `-- requires intended project open + relevant MIDI loaded
 +-- assign/load instruments --------> kontakt-setup ---------> STOP
 |    +-- contextual scope -> bitwig-explore -> resolved owners -> return
 |    +-- missing/stale initialization -> instrument-intake -> return
 |    `-- requires settled receiver specification; setup owns layer/slot materialization
 +-- mix / diagnose balance ---------> bitwig-mixing
 |    `-- audio evidence ------------> bitwig-export-audio --> return
 +-- export native audio -----------> bitwig-export-audio ----> STOP
 +-- export MIDI2 / reconstruct score -> bitwig-export-midi2 --> STOP
 +-- analyze score / composition ----> score-analysis --------> STOP
 |    `-- current DAW score needed --> bitwig-export-midi2 --> return
 +-- inspect/navigate ---------------> bitwig-explore --------> STOP
 `-- edit notes/expression/curves ----> bitwig-edit -> verify --> STOP
```

Resolve skill paths relative to this directory and check optional references exist.
Use the saved musical project's own instructions to resolve its project paths.
Use the owning production skill and current CLI help to establish available operations.
Do not invent command names or install another control surface to fill a gap.

## Meaning and authority

```text
original score + user direction + documented library capabilities
                              |
                       PERFORMANCE.md
                              |
                  explicit supported conversion inputs
                              |
                           convert
                              |
                       MIDI2 + companion
                              |
               native placement and clip replacement
                              |
                     independent native readback
```

| Authority | Owns | Does not prove |
| --- | --- | --- |
| User direction | Outcome, scope, accepted departures | Successful execution |
| Original score | Authored notation, source identity | Every editorial decision |
| Official manuals + observed inventory | Documented behavior + available files | Suitability for this passage |
| Library INSTRUMENT.md | Cited capability description; observed exceptions distinguished | Project instrument selection or loaded state |
| Project PERFORMANCE.md | Normative musical choices, receiver bindings, approximations, cues | Actual host state |
| MIDI2 + companion | Executable rendition + exact intended patch/routing bindings | Audible support or current host state |
| Fresh observations / retained receipts | Measured state / historical operation evidence | Intent or subjective approval |

The agent translates musical choices into supported inputs. Neither CLI executes
arbitrary prose. Before conversion/import/setup, compare the affected musical
specification with the actual typed inputs and companion. Record source locations and declared revisions in operation evidence. This is an agent review obligation;
there is no general automatic PERFORMANCE.md compiler/agreement checker today.
Conflicts require reconciliation, not silently choosing the easiest authority.

PERFORMANCE.md is the current musical specification. Preserve relevant older
commentary as context; do not create a competing assignment ledger. Exploration
and bounded native edits do not require adopting the rendition workflow.

## Domain identities

```text
score part / voice               authored musical identity
sample library                   installed collection; one INSTRUMENT.md per root
Kontakt instrument patch (.nki)  sampled/scripted resource, possibly multi-articulation
receiver binding                 project track/channel -> stock patch + native routing
Instrument Layer                 Bitwig device containing layer chains (CLI: slots)
Arranger Cue Marker              native landmark linked by permanent cue key
```

A MIDI channel is an address, not an instrument. A UMP group is not a Bitwig group.
CLI instrument channels are 1..16; companion/native note channels are 0..15.
Kontakt input A/1 is a separate receiver address. Track identity, binding key,
patch identity, channel, and temporary native IDs are distinct. Resolve qualified
owners afresh; duplicate display names are possible. Musical names such as
`Violins I` / `Pizzicato` link to stable binding keys in PERFORMANCE.md; file slugs,
bank positions and UUIDs do not substitute for musical labels.

PERFORMANCE.md owns all orchestration choices. Kontakt patch identity is solely
the `.nki` path relative to `INSTRUMENTS_ROOT_DIR`.
Identify resources by
source location, relative instrument path and declared version; establish edit
authority through qualified native owners, explicit expected values, native
change information and operation IDs. Reuse completed initialization by covered
paths and observed operation outcome.

Per-note expression and track/device automation have separate ownership, units,
and write authority. Source beats, arranger beats, bar numbers and seconds differ.
Stored notes are not expanded loop occurrences. Sampled legato is not a synonym
for ties, slurs, overlap, or an instruction to connect every note.

## Artifact and mutation ownership

| Skill | Owns | Completion evidence |
| --- | --- | --- |
| [bitwig-compose](bitwig-compose/SKILL.md) | Requested multi-stage composition outcome | Owner evidence for each requested result |
| [score-intake](score-intake/SKILL.md) | Saved workspace, original score, project instructions | Actual saved path + unaltered source + recorded project identity |
| [instrument-intake](instrument-intake/SKILL.md) | Library documentation and verified one-time batch preprocessing | Completed path-based processing coverage + INSTRUMENT.md marker + cited capabilities/unknowns |
| [score-realization](score-realization/SKILL.md) | Musical realization, patch selection, conversion, native notes/controls/channel layers | Intended and imported content/routing agree; save confirmed |
| [performance-cues](performance-cues/SKILL.md) | Scholarship, arc, cue sheet, markers | Current MIDI scope + cited decisions + independent marker readback |
| [kontakt-setup](kontakt-setup/SKILL.md) | Existing controller assignment and exact NKI loading | Integrated assignment/load result and native routing readback; no customization |
| [bitwig-mixing](bitwig-mixing/SKILL.md) | Musical balance, scoped mixer changes and audio-backed diagnosis | Native readback, musical preservation, verified exports and listening limits |
| [bitwig-export-audio](bitwig-export-audio/SKILL.md) | Exact native channel/range/options export | Native final outcome plus independent file/audio checks |
| [bitwig-explore](bitwig-explore/SKILL.md) | Discovery, deliberate selection, reads, requested audition | Complete intended scope; retained observation; bounded finding |
| [bitwig-edit](bitwig-edit/SKILL.md) | Authorized notes/expression/automation changes | Requested result + independent full preservation readback |

PERFORMANCE.md uses the [shared template](score-intake/assets/PERFORMANCE.md).
Intake establishes identity; realization owns realization/bindings; cues owns
interpretation/cue sections; setup owns configuration evidence. Every writer
preserves the other sections and user commentary. Setup may report a required
specification correction but cannot silently change musical intent. Cues do not
automatically enact their directions as notes, automation or tempo changes.

## Readiness and change propagation

```text
workspace     musical spec     native content     receiver setup     cue placement
   [ ]            [ ]              [ ]                 [ ]               [ ]

audible behavior [ ]     user musical approval [ ]     (separate evidence)
```

These are independent facts, not one global READY flag. Require only those needed
by the requested task. Relevant MIDI can exist without this workflow's companion;
exploration, cues and bounded native edits do not require regenerating it.

| Change / condition | Route |
| --- | --- |
| Same intent/resources and matching current state | Verify/reuse; no duplicate project, group, slot or load |
| Generated rendition changes | Realization; reconvert; supported reimport only |
| Authorized native notes/expression revision | Editing; required setup returns to editing; record bundle divergence without automatic reconversion |
| Patch changes control law | Revise bindings + conversion; reconvert; reconcile setup |
| Patch changes, performed messages unchanged | Revise companion/specification; reconcile setup; no automatic reconversion |
| Cue-only interpretation/placement change | Cues; musical changes require realization/editing scope |
| Native user edits diverge from imported bundle | Preserve and reconcile before replacement; no automatic reverse conversion |
| Unknown operation outcome | Original operation status + fresh read; no blind replay or Undo |
| Missing/ambiguous owner, shared-source or loop conflict | Preserve state; resolve scope or report precise unsupported operation |
| Missing required capability | Keep affected result pending; continue independent authorized work |

Changed executable decisions invalidate affected plans. Changed arrangement can
invalidate cue placement. Changed patches can invalidate acoustic evidence.
An updated library document is evidence to review affected choices, not permission
to reorchestrate existing projects. Audition uses exploration; corrections route
to editing, realization or setup. A full listening review is not an automatic task.

## Supported production operations

Use the owning skill and current CLI help to determine which actions are available.

* General public New/Save/Open and transport/history commands are not exposed.
  Use the [native keyboard workflow](bitwig-explore/references/keyboard-workflow.md)
  for supported lifecycle actions. Confirm that the intended project was actually
  created and saved.
* A generated importer schedule may contain devices outside the mandatory
  execution boundary. Such a schedule is inadmissible. Use the existing bounded
  content operations where they preserve the complete intended content. Assign
  instruments through the setup skill. If the available commands cannot preserve
  the intended scope, leave the affected placement pending.
* General reconciliation/migration of populated projects is not implemented.
  Notes-only reimport requires unchanged receiver/control/global/clip topology and
  its ownership evidence. It does not overwrite arbitrary native edits.
* Marker creation is additive. Existing-key move/rename/delete has no production
  command here; report the exact requested revision as pending.
* Native track MIDI CC creation covers CC0–119 with an explicit beat-zero value;
  existing track automation is readable/editable. Pitch-bend creation and general
  track-lane deletion require owning-controller qualification before use. Existing
  committed sources remain preservation obligations, not workflow prerequisites.
* Kontakt assignment is not a universal idempotent loader. Unknown populated
  racks require provenance/reconciliation; load completion does not prove samples,
  persistence or audible behavior.

## Finish or reconcile

Each task ends after its owned result and independent verification. An unknown
operation outcome requires the original operation's status and fresh target
readback before any further write. Preserve the original intent and receipt.
