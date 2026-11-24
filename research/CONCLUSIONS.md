# Control boundaries after the programmatic follow-up

The [worksheet](WORKSHEET.md) records the initial boundary experiments on
Bitwig 6.1 and DrivenByMoss 26.6.5. The [Bach exercise](QUI-TOLLIS.md) records
subsequent checks against the bridge-owned OSC service, including initial
score placement and ending limits. Use the
[production session procedure](../skills/bitwig-explore/references/session-actions.md)
for the currently qualified routes.

```text
Intent + target + preservation requirements
  -> matching native or OSC route
  -> exact preflight -> action -> independent effect check
  -> unresolved candidate stays open
  -> proven remaining UI dependency gets a qualified computer recipe
```

| Intent | Preferred route | What the comparison establishes |
| --- | --- | --- |
| Print the arranger clip organization | Native entity inventory, scoped to the track or group when known | The tested inventory returned each clip's owner, identity, start, duration, source bounds and loop metadata. The reviewed OSC protocol exposes launcher slots and cursor feedback. An equivalent arranger inventory is not documented. |
| Jump to a known clip's start | OSC `/time` with the arranger beat from the existing inventory | Seeking to beat 16 moved the playhead while stopped and preserved selection on the clip at beat 8. It needs no duplicate controller seek API. |
| Select the exact arranger clip or lane | Native path selection or lane time range | Native readback confirmed selection of the requested arranger clip. OSC seek and launcher-slot addressing satisfy different intents. Reveal or focus is a separate check if the request calls for it. |
| Add or rename a simple track, toggle the Note Editor, save, or execute known Undo/Redo | OSC with a current target or known history action and independent readback | Indexed track rename, panel toggle, Save and history actions worked. Save was checked on disk. Reopen and keyboard focus remain separate checks. |
| Find the exact clip or lane, select a compatible set, or edit stored notes and curves | Native structure, selection and complete replacement | Native identifiers resolved duplicate labels. The same replacement route handled both motifs and named note changes. OSC bank selection did not supply native Inspector slot authority in the tested contexts. |
| Create a document clip at an exact arranger position | Native time range, qualified OSC focus, then configured **Create Empty Clip** | A focused [8,12] range created one empty clip. One batch later created exact containers on two, then ten, compatible tracks. Native ranges alone were insufficient when the editor was `none`. |
| Create an empty launcher clip | Native create with a freshly observed empty Inspector slot | This creates an exact stopped container. OSC create starts playback and enables overdub. Use it where that performance behavior is intended. |
| Insert a known device or reconcile a receiver | OSC for a qualified one-off insertion. Native ensure for named Layer/slot routing and exact Kontakt assignment | Repeating OSC EQ insertion produced two devices. Repeating native Layer and slot ensure preserved observed device and slot identities and routing. |

[Arranger inventory, seek and selection](WORKSHEET.md#7-arranger-document-navigation) ·
[Live comparison and evidence](WORKSHEET.md#6-programmatic-follow-up) ·
[Original launcher observations](WORKSHEET.md#3-sketch-audition-and-revise) ·
[Receiver routing](WORKSHEET.md#4-reuse-a-receiver)

## Rationale for the skill structure

- **Explore** uses one scoped entity read to resolve clip identity and
  position. From that result, seek with OSC when the user wants the playhead,
  use native selection when they want the clip or lane to become the target,
  and establish visibility or actual focus only when requested. At a handoff
  to native launcher creation, require an actual `launcher read` result.
- **Edit** distinguishes writing stored content from placing or changing
  the clip container. Use a complete native transaction once the container
  exists. Qualify exact geometry independently. The native content-replacement
  refusal protects this boundary.
- **Setup** hands off the returned Layer target immediately to slot
  ensure. Reuse the scoped routing read. Keep OSC one-off insertion available
  where that matches the request and call the native ensure route for
  convergent topology.
- **Shared session actions** uses one OSC-first reference for
  qualified action routes, target checks and verification. Keep computer
  recipes for proven residuals such as actual focus. Keep new controller
  methods for operations with valuable native semantics such as exact
  identity, complete reads and preserved-state mutation.

[Skill procedure critique](WORKSHEET.md#procedure-critique)

## Current boundaries

OSC handles the tested create, zoom, selected-track lane visibility, single
split context, and focus toggle through configured actions. Use native
readback to prove clip geometry and content. The split preserved the source
notes in both copies, so compare actual occurrences inside each clip's source
window. Global tempo and meter setters affected time before the stopped
playhead and cannot perform a boundary-local edit.

Use Computer for native project dialogs and the Launcher Inspector fallback.
OSC bank selection did not populate the native Launcher observation. Move,
resize, local tempo/meter, and other context-specific actions need qualification
for their actual effect. The [exercise record](QUI-TOLLIS.md) separates
completed probes from outstanding saved, audible, and musical results.
