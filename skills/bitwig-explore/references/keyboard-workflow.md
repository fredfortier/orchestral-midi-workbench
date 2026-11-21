# Native keyboard actions around the MIDI tools

Use this reference only for supporting Bitwig window/project/navigation/history
work and authorized legacy sender/lane cleanup under the shared workflow.
MIDI notes, expression and curve creation/editing stay on the complete-clip
read/replace route. No transport, mixer, window-management or general Bitwig
control subcommands are prerequisites for that route.

## Select an action before sending keys

Use Bitwig's **named native action** and its binding in the active Shortcuts
configuration. Verify that binding once on the actual host/profile; a familiar
shortcut from another DAW or another Bitwig profile is not evidence. The native
keymap contains distinct actions including Open, Save, Save as, Undo, Redo,
Select All, Unselect All, Rename, Activate item, Expand Item, Collapse Item and
Cancel Dialog. Their focus contexts matter.

For project tabs, panel focus, arranger clip selection, opening/saving a project,
and explicit audition/history requests:

1. **CONTROLLER** Resolve the intended project and relevant entity paths with `controller info`; read current selection. Select exact paths when the recipe requires selection.
2. **COMPUTER** Establish the required panel focus and object/time mode. Choose the qualified named action/binding for that host/profile.
3. **COMPUTER** Send that binding through **Use Computer** to the intended Bitwig window.
4. **CONTROLLER** Check the resulting native state: `info` for project/path changes; a complete
   read of the same owning-track paths for selection/content/history consequences.
   An absent, ambiguous or unexpected result stops dependent work. Do not try
   neighboring keys, repeat an uncertain action, or add a helper subcommand.
5. **CONTROLLER** Reobserve musical content before the next write. Never issue Undo to recover
   from an unknown controller outcome; it may undo somebody else's latest edit.

Keyboard actions are allowed surrounding agency, not a replacement for a failing
MIDI write, automation read, Instrument Layer configuration or Kontakt bootstrap.
Use Computer screenshots can diagnose an unexpected state or inspect Shortcuts
while qualifying a route. **Do not parse screenshots/OCR as systematic workflow
state or as musical acceptance evidence.** If the available computer/accessibility
integration cannot establish a necessary UI precondition, report it and leave
the affected UI action pending.

## Qualification scope

Resolve the current host/profile before reusing a binding. A historical bridge
outage is not a current capability test. Inspect the active Shortcuts profile
and use the named action with its focus precondition. When qualifying a new
repeatable action sequence, check its nearest wrong-focus case on a disposable
host before using it systematically. Retain the action name, binding/profile,
focus precondition, input and native before/after result with the project evidence;
do not requalify unchanged actions for every track.

Save was executed on Bitwig 6.1 with `Default keyboard mappings (My version)`:
Ctrl+S on the project window, saving indicator finished, dirty asterisk cleared,
file modification advanced, then complete clip readback matched. This establishes
that action/profile case, not Open, audition or arbitrary Undo sequences.

Development experiments and wrong-focus probes use an agent-managed disposable
fixture. An authorized Save on the user's project is ordinary task execution,
not a diagnostic campaign. The observed recipes and their limits follow below.

## Sources

- [Window anatomy](https://www.bitwig.com/userguide/latest/anatomy_of_the_bitwig_studio_window/): project tabs and controller focus are distinct from musical clip content.
- [Menus and transport](https://www.bitwig.com/userguide/latest/the_window_menus_transport_area/): menu items display configured shortcuts; transport state has its own native semantics.
- [Note events](https://www.bitwig.com/userguide/latest/working_with_note_events/): notes are stored in clips; the Detail Editor supports multi-clip/track work.

These sources establish native semantics, not tested key bindings on the active
profile. New native supporting actions require the same focused qualification;
they do not expand the controller's production operation vocabulary.

## Selection and focus prerequisites

| Named action | Required state | Expected result |
| --- | --- | --- |
| Save | Intended project window; no modal dialog or text edit intercepting binding | Save completes, dirty indicator clears, saved file advances; complete affected content matches |
| Rename | Exact selected track header, clip object, device or slot; its panel focused; object mode | Only that item's name changes; resolve its new path with the same native discriminator |
| Arrow navigation | Qualified binding in the intended panel; known primary object; object mode unless a time action is requested | Native reread shows the intended adjacent target; do not infer from scrolling |
| Delete selected points | Automation editor/lane focused with exact point or time-range selection | Points change; lane remains; this belongs to the musical edit owner |
| Clear Arranger Automation | Exact automation owner/context | Curve clearing may retain fallback state; it does not delete the lane |
| Delete automation lane | Controller identifies exact lane path; COMPUTER focuses that lane header and opens its context menu | Only that lane disappears; other curves and all notes remain |
| Object/time selection action | Intended arranger panel, configured binding and known starting mode | Native reread confirms mode, lanes/objects and bounds separately |

Only Save and the specific observed recipes below have historical profile evidence.
Arrow/object-time recipes require disposable qualification of the exact binding,
focus and wrong-focus behavior before systematic use. No generic Delete or arrow
sequence is admitted by familiarity.

## Automation lane and sender cleanup

1. **CONTROLLER** Resolve the exact lane/device and owning track. Retain complete
   track content before cleanup, including all `.automation` and clip notes and
   expression. A track with no note clips uses the same track-path read.
2. **CONTROLLER** Establish exact supported native context. A lane is a time-mode
   target with explicit bounds; it is not an object-selection target. This does
   not establish the lane-header context menu.
3. **COMPUTER** Focus the exact lane header/device inspector, verify its displayed
   owner against the resolved path, and open the named context action. Do not
   equate selected time/points with a selected lane header.
4. **CONTROLLER** Independently reread the same track paths. Compare every
   remaining curve/point and note/expression, plus structure/routing. Verify that
   only the authorized lane or sender disappeared. An unexpected difference stops
   further cleanup; do not batch across unverified focus changes.


### Observed computer recipes

Task evidence, not a second controller API. Host/profile-specific; recheck focus
and bindings before reuse. Use native reads for musical verification.

| Action | Recipe | Observed check |
| --- | --- | --- |
| Save | **COMPUTER** Bitwig project window → named Save action, Ctrl+S in `Default keyboard mappings (My version)` | Saving finishes, dirty asterisk clears, file mtime advances; complete clip reread matches |
| Rename | **CONTROLLER** select exact path → **COMPUTER** focus its panel → named Rename, Ctrl+R in that profile → text → Return | Resolve the renamed item again through `info`; names are not IDs |
| Set a fixed knob value | **CONTROLLER** resolve exact receiver/device → **COMPUTER** focus numeric editor → triple-click knob → Ctrl+A → type number → Return; no automation lane | Portato CC11 status displayed 127.00; complete readback still had only knob 1’s curve, then Save cleared dirty state |
| Reveal automation | **COMPUTER** track's lower-left disclosure arrow | Inspect current layout after expansion; rows and scroll position move |
| Remove an unwanted automation lane | **CONTROLLER** resolve lane → **COMPUTER** exact lane header → context menu → **Delete automation lane** | Complete owning-track read of `.automation` confirms lane absent and all changing curves preserved; **Clear Arranger Automation** retains a fallback and is different |
| Remove a sender, explicitly authorized cleanup only | **CONTROLLER** resolve exact device path → **COMPUTER** focus exact track → exact receiver layer → select MIDI CC in that layer's **inspector device list** → Delete | `info` confirms only that sender is gone; Kontakt, routing, notes and unrelated curves survive |

On this Linux integration, native drag is unsupported; Ctrl+click did not open
the numeric editor. Triple-click did. Use `typeText` for the focused numeric field;
the optional `typeCharacters` method was absent. Do not infer success from a
changed knob outline or create temporary static automation to set a constant.

A layer selection can leave the bottom device panel showing another layer's
previous device. Its visible device header is not the selected receiver's authority.
Resolve the device in the selected layer's inspector before deletion. This caused
a portato sender/curve removal during cleanup; recovery restored the curve through
the controller and independent readback. Never batch destructive actions across
unverified layer changes. Lane removal near the bottom can scroll the arrangement;
reobserve before using another row coordinate. Coordinates are session-local.

Keyboard delivery can affect transport: check stopped state before a mutation.
If export/save or user activity starts, leave it undisturbed; reobserve authority
and focus after it finishes before resuming dependent native work.
Screenshots diagnose focus/layout; they do not establish note or curve correctness.
Unknown controller outcomes require original operation status and native readback,
not keyboard Undo. A known UI cleanup error needs its exact history boundary and
readback; a familiar shortcut alone does not establish a safe recovery target.

Native selection supplies exact targets; panel focus and keys stay on the computer. Controller-owned MIDI/automation edits and
integrated Kontakt loading keep their existing owners. These cleanup recipes are
not permission to use UI edits to bypass failed controller CRUD.

The 2026-11-17 qualification is in local Codex thread
`01a1eb0d-afa2-7943-ac4a-3177ea9bd389`. Its artifacts remain in the Bach project's
original paths `assets/evidence/expression-realization-20261117T195858Z/` and
`assets/evidence/cc-necessity-cleanup-20261117/`. No general Open/Undo/Redo or
wrong-focus qualification is claimed.

## Select All on track headers — 20 November 2026

**CONTROLLER** Resolve/select the intended track/group path. **COMPUTER** click
its visible header label to establish actual header-panel focus, confirm the
native Edit menu shows Select All → Ctrl+A, dismiss the menu and send Ctrl+A.
**CONTROLLER** reread membership and primary. On Bitwig 6.1 this selected the
fixture's two top-level headers and ten headers in the active SATB-context case;
complete native content remained unchanged and original supported selection was
restored. This qualifies that binding/context, not points, clips, Delete or arrows.
A native Inspector target does not establish real keyboard focus.
