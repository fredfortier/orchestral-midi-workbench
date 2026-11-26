# Bitwig session actions

Use OSC for qualified session actions, [native navigation](arranger-navigation.md)
for exact target identity and selection, and the [computer recipes](keyboard-workflow.md)
for project dialogs, configuration and residual focus or menu dependencies.
Use the active project and its saved path from native `info`. Carry an already
resolved native path and arranger beat forward.

## Preflight and verify

Use the existing bridge with OSC enabled. Resolve its configured HTTP port and
the DrivenByMoss profile. If this installation has not qualified the profile,
use the installed controller's **Settings → Controllers → Open Sound Control
OSC** to inspect its ports and numbered actions. Confirm current project and
transport with the feedback addresses relevant to the action.

```sh
midi-bridge osc /refresh --observe /project/name --observe /play --observe /beat/str
```

A submission receipt confirms local UDP dispatch. With `--observe`, the
result contains the last argument array seen per requested address in a
bounded window. A field can be absent when it did not change. Send `/refresh`
when a fresh peer observation is required. Verify the actual effect using
native state for document effects, current viewport for display changes,
and audio for listening. A lost response requires fresh observation before
another send. A bind conflict or absent OSC route needs a matching service
configuration, not a second listener.

## Direct actions

| Intent | Target and action | Verify |
| --- | --- | --- |
| Seek | From the resolved position in quarter-note arranger beats, send `/time f:BEAT` | Read `/beat/str` with the current meter and compare to the requested location. Check preserved selection if it matters. |
| Play/stop | Check engine, stopped/recording state and the intended location. Send `/play` or `/stop` | Read `/play` and `/beat/str`. Playback state establishes transport. |
| Initialize a fresh project's global pulse | At beat zero set the agreed meter with `/time/signature s:3/4` and raw quarter-note BPM with `/tempo/raw f:55`, substituting the intended values. | Refresh both values and independently check native timing across the range. These setters affected earlier time even when sent at a later beat. Use only for intended project-wide changes. |
| Activate audio in the intended tab | Check the active project and engine state, then send `/project/engine i:1` | Observe engine active for the same project. Account for the previously active tab if the request needs it restored. |
| Audition Launcher | Confirm exact owner and slot/clip. Match the current OSC bank track and scene with project, name, position and slot content. The OSC slot name can be blank for a native default clip name. Use the resolved scene position and occupied slot in that case. Send `/track/T/clip/S/launch i:1` using one-based OSC bank indexes. | Observe that slot's `isPlaying` and transport. Listen if audition is requested. Stop, then return the affected track to Arrangement with `/track/T/clip/returntoarrangement`. Stopped transport and `isPlaying: 0` establish that Launcher playback ended. Verify the Arrangement source independently before further listening. |
| Save to an existing native path | Verify current project and saved path, then send `/project/save`. For the first save use the native Save dialog. | Verify the write to the exact project file and independently reread the affected content in the current session. |
| Undo/Redo | Verify the current project and known, immediately adjacent history entry, then send `/undo` or `/redo` once. | Independently reread affected geometry/content and check preservation. An unknown mutation outcome needs reconciliation first. |
| Add or rename a simple instrument track | For child creation, enter the intended group in Bitwig before `/track/add/instrument`. Group header selection can leave creation at the parent level. Resolve `/track/T/name s:NAME` against current OSC names. If the track is outside the bank, page with `/track/bank/page/+` or `/track/bank/page/-` and refresh. | Verify the first child's native ancestry before repeating creation. Compare IDs and exact names after rename. Reuse the returned paths. |
| Show or hide an Arranger section | Read visibility, then send the toggle only if the current value differs, e.g. `/arranger/timeLineVisibility`. | Observe visibility. Establish focus separately if needed. |

Use [editing](../../bitwig-edit/SKILL.md) for stopped Launcher clip creation and
stored notes, expression, and curves. Use [Kontakt setup](../../kontakt-setup/SKILL.md)
for receiver ensure. Indexed session control does not supply the native identity or
preservation needed by those operations.

The visible project tab and controller target can differ after opening or
closing a project. Reobserve both before issuing a write. An OSC project
name alone cannot distinguish duplicate opens or prove a saved path. If a
remote Save would target a different project, use the named File → Save
action in the visible, previously identified project and verify the exact
file on disk. Resolve the remote target before resuming remote writes.

For a known occupied Launcher clip after reopening, combine the freshly
resolved native track element with the known `launcherClip` name and
zero-based `scene`, omitting the stale clip ID. An explicit-path native
`content read` resolves the current ID and verifies the candidate content
before addressing the matching OSC bank slot. An empty slot still requires
the [native slot observation](keyboard-workflow.md) before creation.

## Audition an existing manual selector

Use this route when an authorized comparison identifies a selector already in
the playing path. Resolve its exact native path and choices with
`controller instrument read --target selector.json`. Confirm that the intended
notes reach it. In Bitwig's device panel, verify Manual mode and record the
playing choice from the selector's active indicator. The native
`selectedSlot` can name the editor selection while another choice plays.

With playback and recording stopped, use the exact choice's **Set as active
layer** control in the Bitwig host device panel. Verify the active label before
each take. Use the [audio export contract](audio-export.md) for renders. After
the comparison, restore the previous playing choice and verify its active
label. Preserve any authored selector mode or automation. If the playing
choice is controlled by the arrangement or a different selector mode, use
that existing control path within the authorized scope.

## Qualified configured profile

The following controller assignments were exercised in Bitwig 6.1 with
DrivenByMoss 26.6.5. Send `midi-bridge osc /action/N` with zero arguments.
A configured action can apply in the active editor, so read the preconditions
and verify the effect before dependent actions.

If slot 5 leaves the native editor `none`, focus the visible Arranger through
the computer, reread the editor and establish the exact range before slot 1.

| Slot and binding | Preconditions and checks |
| --- | --- |
| **1 · General · Create Empty Clip** | Prove the selected time range has no overlap on every intended instrument track. An occupied selection can replace the audible middle of an existing clip with an empty clip. Require stopped transport and a live native selection read with exact range, track membership and `editor: arranger`. If focus is missing use slot 5 as described below and reread. Dispatch once, then independently verify one empty arranger clip per requested track with exact owner, start and duration, plus preservation of adjacent material. Pass its path to editing. |
| **2 · Arranger · Zoom Arranger to Selection** | Set the exact arranger object/range and observe viewport before sending. Compare the resulting bounds and verify native selection stays on the target. |
| **3 · Arranger · Toggle Automation Lanes for Selected Tracks** | Explicitly select the exact track headers before toggling. An empty primary and member list can still toggle the last track's lanes. Observe current disclosure, then check target and neighboring track visibility. For lane time selection or curve edits, use the native procedures. |
| **4 · General · Split** | Use a stopped, focused Arranger with one track time-range selection whose first bound is inside the intended nonlooping clip and whose second bound is the clip end. Verify the selected owners and absence of overlapping clips. After dispatch, verify both placements/source windows and surviving note/expression occurrences. Each side can retain source notes outside its playback window. Other geometry cases require qualification. |
| **5 · Panel Management · Focus/toggle Arranger Timeline Panel** | Observe independent native editor and timeline visibility. If the timeline is visible but focus is elsewhere, or the timeline is hidden, send once and verify `editor: arranger` and visible timeline. The Arranger can restore its own previous selection when focus changes. Reread and establish the exact target before dispatching the next action. If already focused, skip the action. Sending it then hides the timeline. |

The starting focus, selected owners and exact geometry are part of the
qualification. For another action binding or context, establish the live
assignment and check its effect and closest wrong context on a disposable
project. Keep unsupported geometry, tempo-point and meter operations open
until their actual result is verified.
