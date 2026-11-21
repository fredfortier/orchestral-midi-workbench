# Read UI selection and ownership

Use [Bitwig's note-selection guide](https://www.bitwig.com/userguide/latest/inspecting_note_clips/#selecting_notes)
for user semantics: object selection and Time Selection are different forms of
selection. Use [layered editing](https://www.bitwig.com/userguide/latest/working_with_note_events/#layered_editing_mode)
to understand multiple clips/tracks and editable layers. The public controller
`Clip` cursor is a scoped observer grid, not that whole editor selection.

## Establish the active editor

October 21: following Openwig's descriptor-read pattern, isolated reads identified
`getDocumentWidget()` → `Nln("active_editor_selection").getValue()` as the actual
active editor. Its `mP_()` is the selection settings object; read
`is_time_selection_active` there. `active_selection` is a different object.

Inspector text entry can make `active_editor_selection` null without clearing
the inspected musical selection. On this build, read `inspector_selection` and
resolve its `lW_()` owner only when the active editor is null. Native `aBI.oOg()`
uses that same `bhU.lW_()` ownership mapping for active selection, and its focus
handling retains the Inspector selection while editing an Inspector widget.
A document-thread probe before/after velocity text entry retained the identical
`bhS` selection and `biA` owner, while active selection became null. The fallback
read returned the complete 900-note graph with only velocity changed to 0.6;
native Undo restored every modeled field and selection. Do not substitute an
arbitrary wrapper's remembered selection. A null `lW_()` remains no editor.
This proves the arranger Inspector case; other editor topologies remain probes.

For an active note editor (`biD` on this build), the probed range query is
`editor.v33(from, to, true, (boA) null)`. It delegates through the editor's
editable layers and returns `fOD` note occurrences. An isolated clip-editor
range returned four occurrences; object selection returned one. Occurrence
`v33()` resolves the source note and `YXc()` resolves the owning clip.
Do not substitute a grid or manually merge inactive editors.

`editor.Iks().YPP()` returns selected **lanes**, not selected notes. This was
observed in the arranger and the clip editor. General loop, filtered/locked-layer
and partial-boundary cases remain fixture obligations.

Inspect arranger, arranger-track editor, arranger-clip editor and launcher-clip
editor separately. The [existing selection probe](../../../../controller/probes/NotesDocumentProbe.java)
reads all four. Its verified build uses these wrapper accessors:

| Editor | Wrapper method |
| --- | --- |
| Arranger | `iDx()` |
| Arranger track editor | `Zlo()` |
| Arranger clip editor | `XlL()` |
| Launcher clip editor | `yhG()` |

On each editor, `jaQ()` is object selection and `Iks()` is time selection. For
the inspected arranger time selection, `zeN()`/`cSc()` are bounds and
`jaQ().zeN()` yields lanes. Do not infer that all editor methods share this
schema. Determine the active edit owner, selected objects, locked/editable
layers and effective host selection before choosing the operation's scope.
Do not merge every editor's cached selection into one invented selection.

The original user selection yielded a two-track time range while the public
arranger cursor reported no selected cells. See [actual observations](../../../../controller/validation/editor-selection/original-selection.jsonl).
An empty object selection does not make its time selection empty.

## Enumerate the right content

Query a selected arranger lane's `v33(double from,double to)` timeline stream,
and close the stream. General `qry()` enumeration also includes launcher clips.
Range overlap identifies candidate clips; it is not the final effective note
selection. Enumerate source notes from the clip's content timeline, map loop
occurrences, and use the host's effective edit semantics at boundaries.

Use stable document IDs plus owner scope: project generation, track, clip
occurrence, content and note. An object path, display name, bank index or array
position is not identity. Grouping changes paths and inserts overview lanes;
our [grouped-selection fixture](../../../../controller/validation/editor-selection/grouped-selection.jsonl)
verified four leaf lanes without duplicate group summaries.

Distinguish:

- A source note stored once in a content timeline.
- Multiple audible/visible occurrences caused by looping a clip.
- Multiple clip references sharing source content.
- A selected part of an occurrence requiring host-defined boundary handling.

Do not edit a shared source when the request addresses only one occurrence
unless native semantics and explicit ownership produce exactly that effect.
Do not call a geometric intersection the effective edit set: host consolidation
can omit apparently intersecting source material outside the playable loop.
See [timeline mapping](timeline-transport.md).

## Snapshot and invalidate

Capture the selection descriptor, owner identities, source state and location
mapping on the document thread. Return immutable values. Freeze this snapshot
on Update; use a project generation and selection revision distinct from note
content revisions. Re-read and compare before admitting a result. Detect changes
in selection kind, lanes, editable layers, clips, shared references and relevant
tempo/loop context, not just changes in note values.

Never select, focus, scroll, launch or move the playhead to obtain selection.
Test tool **2** over multiple tracks, grouped children, clips and >64 beats,
then object selection in both arranger and launcher editors. Counts must prove
complete coverage, not a first page or a diagnostic sampling limit.

## Full-piece Bach counterexample and bounded admission (October 21)

The full imported Contrapunctus I fixture has four note clips, 900 notes, and a
312-beat duration. An explicit leaf-track arranger Time Selection returned all
900 notes, including the last note ends at beat 312. Eight rounds of native CRUD
and separate Undo/Redo preserved the compared note graph and mappings exactly;
see [retained evidence](../../../../controller/validation/bach/README.md).

**The original collapsed-group read failed; its Time Selection read is now corrected.** On this same fixture, both
object selection and tool-2 Time Selection over the group summary returned an
empty clip/note/lane set through the current `editor.jaQ()` / `editor.Iks()` paths,
while the UI showed the group's full musical range. The earlier four-leaf-lane
fixture did not establish group-overview coverage.

The read-only topology probe identified `pxE.selected_items` containing a `suB`
meta note lane. `pxE.zeN()` omitted that lane. In the installed native strategy
`fen`, range collection expands `GSp.v33(true,true,true,false)` and returns the
real clips via `pxE.uS()`. That collector returned four `UMj` note clips and eight
`I3W` automation clips for the imported group. The corrected adapter now uses
`pxE.uS()` for arranger Time Selection and records raw selected lane IDs via
`pxE.jaQ()`. An actual collapsed-group reread returned all 900 selected notes;
every clip/note field exactly equalled the independent original leaf-track
snapshot. See the linked corrected read and native topology evidence.

This admission covers the concrete group Time Selection read. Object-selection
summaries and broader grouped mutations remain unproven. Do not infer their
coverage from this correction or require UI expansion as a product workaround.

## Collapsed group object selection

On the admitted 6.1 build, arranger object selection `LCE.jaQ()` contains an
`NJf` meta clip. `LCE.EwU()` alone can return no note clips. Follow the native
object strategy: `LCE.vfN()` returns `nkb`; its `EwU(Collection)` expands the raw
selected objects recursively through the meta clip's children and deduplicates
actual clip events. Preserve raw object IDs in the selection observation, then
filter expanded events to `UMj` note clips. Do not substitute a Time Selection
or union cached selections. [Retained native topology and full readback](../../../../controller/validation/group-object/)
prove all 900 Bach notes match the independent four-leaf-clip graph. This does
not establish launcher, locked/filtered layer or shared-occurrence semantics.

## Deliberate arranger navigation

Explicit native navigation is available as `controller info` and
`controller selection {read,set}`; see the
[composer procedure](../../../bitwig-explore/references/arranger-navigation.md).
The prohibition above on selecting to obtain a read applies to observation
operations: reads remain selection-preserving. A deliberate selection command
is a separate, fenced UI action, followed by a fresh effective musical read.
Native maps and qualification are documented in the
[arranger contract](../../../../controller/ARCHITECTURE.md#native-selection).
