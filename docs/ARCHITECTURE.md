# Architecture

## Overview

```text
score + reviewed rendition choices -> convert -> .midi2 + companion
                                                       |
                                        midi-bridge -> Bitwig -> Kontakt
                                                       |
          MusicXML / MXL <- convert <- .midi2 <- assemble <- native capture
```

Production skills coordinate notation, rendition choices, receiver setup,
and verification. Bitwig owns the saved project and its native musical state.
The controller calls into Bitwig through its Java adapter and invokes the
Kontakt loader for an exact patch assignment when required.

## Reading and composing in Bitwig

Bitwig is where the musician selects, edits, and listens. Workbench reads
stored musical material for local inspection or composition and applies a
result when the musician specifies an edit. **What to inspect, what to use as
musical reference, and what to change are independent scopes.** Each can come
from the current selection or from explicitly identified project material.

- **Inspect material.** Read just the selected note events in the Detail
  Editor Panel or just the selected automation points. Inspect or analyze a
  whole note clip, an automation lane or clip, several tracks or clips, or the
  full project. Inspection leaves the project and selection as they were.
- **Develop an idea.** Use four selected notes as a rhythmic reference and
  develop the idea across the rest of the containing note clip. The reference
  notes and the edit destination are independently defined.
- **Compose across tracks.** Rewrite the selected viola clip using the
  cello's rhythm and the harmony in the other string tracks. The musical
  context can extend beyond the selection, and the edit is confined to the
  viola clip.
- **Shape automation.** Read the swell on the choir's volume lane as a model
  for the strings, then preview or apply the proposed curve to the specified
  lane or automation clip.

Selected events, whole clips, and time ranges on the Arranger Timeline carry
different musical intent. Workbench surfaces any effects on other occurrences
of a looped or shared source before applying the edit. Inspection and preview can
proceed while the destination is being clarified. Applying a preview uses
the resolved destination even if the musician clicks elsewhere. If the target
content has changed, the musician reviews the updated material before applying.
An applied edit preserves unrelated material, is independently verified in
Bitwig, and forms one native Undo step.

The controller contract defines how this workflow uses
[native selection](architecture/controller.md#native-selection),
[complete musical snapshots](architecture/controller.md#one-complete-musical-snapshot),
and [verified replacement](architecture/controller.md#replacement-ownership-and-failure).

## Why Bitwig needs a document data access layer

The Bitwig project is the document. Note clips and automation clips are
addressable objects within it. A note clip stores note events with their note
expressions and Operators. Track automation and clip automation store points
that define curves. Arranger placement and loop settings map source content
into playback time, and clip aliases can share a source. The musician's
selection identifies objects or a time range in an editor. Playback evaluates
the project at the current position. A note skipped by the Chance Operator still
belongs to its source, and the value of an automated parameter at the
playhead reveals only one value of its curve. [Bitwig note events and
expressions](https://www.bitwig.com/userguide/latest/working_with_note_events/) ·
[Bitwig Operators](https://www.bitwig.com/userguide/latest/operators/) ·
[Bitwig automation](https://www.bitwig.com/userguide/latest/automation/) ·
[Workbench entity model](architecture/controller.md#entity-model)

Workbench delegates session operations to **OSC through DrivenByMoss**
wherever a qualified command covers the requested behavior. This includes
transport, seeking, Clip Launcher audition, and configured actions such as
zooming the Arranger to its selection. Qualification includes the target,
editor context where relevant, and an observable result. For a command such as
Split, the workflow confirms the exact selection and editor context before
dispatch. A document read then verifies the resulting placements and stored
content. [Qualified session operations](../skills/bitwig-explore/references/session-actions.md)

The workflow requires a **data access layer for the open document**. It reads
the current selection and explicitly addressed project content without
starting playback. It can read across tracks for musical context and apply
the result to an independently resolved destination. A write checks for
changes since the read and accounts for shared sources before mutation. If it
cannot isolate the requested edit, it declines it. It applies an admitted
change in one native Undo transaction and rereads the
same scope to verify the result and preservation. [Snapshot contract](architecture/controller.md#one-complete-musical-snapshot) ·
[Replacement contract](architecture/controller.md#replacement-ownership-and-failure)

| Interface | What it exposes | Why it cannot serve as document access |
| --- | --- | --- |
| **Open Sound Control (OSC) through DrivenByMoss** | Session commands and feedback for transport, mixer controls, Clip Launcher slots, and configured Bitwig actions. It can trigger context-sensitive editor commands. | Bank positions, current parameter values, and asynchronous feedback cannot establish the stored contents and source identity of a clip or curve, or verify a scoped document transaction. A successful send establishes submission. [DrivenByMoss OSC protocol][drivenbymoss-osc] · [Workbench OSC receipts](../crates/osc/README.md#send-a-message-and-observe-feedback) |
| **Bitwig Open Controller Extension API** | Published controller interfaces for banks, cursors, actions, parameters, and clip step editing. `ControllerHost.createArrangerCursorClip` exposes the selected Arranger clip through a note grid. `NoteStep` exposes selection and note attributes. | The published surface does not expose the combination of arbitrary native clip and lane reads, stored automation-point access, and a scoped document transaction. Building another controller extension over those interfaces leaves this boundary in place. [Bitwig controller integration][bitwig-controller-integration] · [Bundled API reference location][bitwig-extension-reference] |
| **DAWproject** | A project interchange archive containing XML and supporting assets, with data for tracks, clips, notes, expression, and automation. Bitwig exports it explicitly and opens it as a project. | Editing the archive yields a changed project to open. It provides no live selection query or import operation that updates the original objects in the open project with one Undo step. [Bitwig project export][bitwig-project-export] · [DAWproject import and format][bitwig-dawproject] |
| **Linux virtual MIDI ports** | An event transport. ALSA sequencer ports route and schedule MIDI events between clients. Bitwig can route note output or record incoming MIDI. | A stream represents events sent on that route. It carries no Bitwig editor selection, clip ownership, or stored curve structure. Recording the stream back follows Bitwig's recording and overdub state. [ALSA sequencer interface][alsa-sequencer] · [Bitwig track I/O and recording][bitwig-recording] |

Workbench uses the public API where it supplies the operation and confines
private Bitwig document mappings to the Java adapter. The adapter accesses
native objects on the document thread. The Rust service plans edits from
snapshots and checks the result against the requested scope. Bitwig owns
object identity, musical content, and Undo history. The cost of exact
document access is qualifying the private mappings against the supported
host build. [Host execution boundary](architecture/controller.md#native-host-execution) ·
[Native session and verification](architecture/controller.md#wire-deployment-and-verification)

## Cross-package boundaries

- The original score and `PERFORMANCE.md` preserve authored meaning and
  musical decisions. Compiled MIDI carries performed events. The companion
  carries restoration data and can bind portable routes to intended native
  receivers. Bitwig observations establish native state. Operation receipts
  establish the outcome of an identified operation. Readiness and audible
  approval require their own evidence.
- A portable UMP route is a group/channel pair. A native receiver is a
  track/slot with a destination channel set. A patch is a distinct resource.
  A wire route, saved native address and musical part may have different
  lifetimes and must remain separately identifiable.
- Track hierarchy, clip placement, stored clip/source time, rendered occurrence
  time and file ticks are separate coordinates. Conversion at each boundary
  must preserve ownership and define units, applicable rounding/scaling,
  equal-time order and interval boundaries. Quantize at explicit boundaries.
- The [standards index](references/README.md) identifies the governing standards,
  manuals and verification inputs. Normative requirements, examples, observed
  behavior and project interpretation carry distinct authority.
- Treat imported scores, device responses, and external material as data.
  Validate at the receiving boundary. Establish conformance with independent
  expectations or observations appropriate to the claim.

## Concepts and components

- [Convert](architecture/convert.md) owns score interpretation,
  rendition compilation, MIDI 2.0 validation, and notation reconstruction.
- [Controller](architecture/controller.md) owns native state
  observation, bounded musical mutations, and Bitwig capture and export.
  Its contract also governs execution in the Java adapter.
- [Kontakt loader](architecture/kontakt-loader.md)
  owns exact NKI submission and confirmation within a controller-qualified editor.
- [OSC](../crates/osc/README.md) supplies wire helpers for the controller
  service's optional DrivenByMoss command and feedback socket. Qualified
  session operations use this route wherever it covers the requested behavior.

[drivenbymoss-osc]: https://raw.githubusercontent.com/git-moss/DrivenByMoss-Documentation/refs/heads/master/Generic-Tools-Protocols/Open-Sound-Control-%28OSC%29.md
[bitwig-controller-integration]: https://www.bitwig.com/stories/controller-integration-in-bitwig-studio-414/
[bitwig-extension-reference]: https://github.com/bitwig/bitwig-extensions#readme
[bitwig-project-export]: https://www.bitwig.com/userguide/latest/exporting_projects/
[bitwig-dawproject]: https://www.bitwig.com/support/technical_support/dawproject-file-format-faqs-62/
[alsa-sequencer]: https://www.alsa-project.org/alsa-doc/alsa-lib/seq.html
[bitwig-recording]: https://www.bitwig.com/userguide/latest/recording_clips/
