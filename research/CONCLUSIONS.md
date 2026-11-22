# Control boundaries after the programmatic follow-up

These are proposed skill routes for the tested Bitwig 6.1 and DrivenByMoss
26.6.5 configuration. Use the [worksheet](WORKSHEET.md) for the observed
results and the [research contract](BOUNDARIES.md) for qualification.
Production skill adoption is a separate change.[^production]

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
| Create a document clip at an exact arranger position | Open qualification | A configured **Create Empty Clip** action is a candidate. Recording a take answers a different question and supplies no qualification for this route. |
| Create an empty launcher clip | Native create with a freshly observed empty Inspector slot | This creates an exact stopped container. OSC create starts playback and enables overdub. Use it where that performance behavior is intended. |
| Insert a known device or reconcile a receiver | OSC for a qualified one-off insertion. Native ensure for named Layer/slot routing and exact Kontakt assignment | Repeating OSC EQ insertion produced two devices. Repeating native Layer and slot ensure preserved observed device and slot identities and routing. |

[Arranger inventory, seek and selection](WORKSHEET.md#7-arranger-document-navigation) ·
[Live comparison and evidence](WORKSHEET.md#6-programmatic-follow-up) ·
[Original launcher observations](WORKSHEET.md#3-sketch-audition-and-revise) ·
[Receiver routing](WORKSHEET.md#4-reuse-a-receiver)

## Skill changes to make

- **Explore** should use one scoped entity read to resolve clip identity and
  position. From that result, seek with OSC when the user wants the playhead,
  use native selection when they want the clip or lane to become the target,
  and establish visibility or actual focus only when requested. At a handoff
  to native launcher creation, require an actual `launcher read` result.
- **Edit** should distinguish writing stored content from placing or changing
  the clip container. Use a complete native transaction once the container
  exists. Qualify exact geometry independently. The native content-replacement
  refusal protects this boundary.
- **Setup** should hand off the returned Layer target immediately to slot
  ensure. Reuse the scoped routing read. Keep OSC one-off insertion available
  where that matches the request and call the native ensure route for
  convergent topology.
- **Shared session actions** should have one small OSC-first reference for
  qualified action routes, target checks and verification. Keep computer
  recipes for proven residuals such as actual focus. Keep new controller
  methods for operations with valuable native semantics such as exact
  identity, complete reads and preserved-state mutation.

[Skill procedure critique](WORKSHEET.md#procedure-critique)

## Remaining qualification

**Exact empty arranger placement remains open.** The installed host exposes
a named **Create Empty Clip** action. The matching extension dispatches
assigned actions through OSC. The tested slot did not establish an effective
binding. Qualification needs a known active binding, a controller-established
arranger range on an exact track, and native readback of the empty clip's
position and length. Keep that branch open and qualify split, move and resize
separately. The original UI-seeded fixture proves only that the later edit had
a clip to operate on.[^actions]

**Focus, reopen, and other action bindings need their own evidence.** The
panel toggle observed visibility and native editor context. Save wrote the
fixture file. The checks do not establish keyboard delivery or reopen
semantics. The follow-up used controller and OSC exclusively.

[^actions]: [Installed capability and runtime result](WORKSHEET.md#6-programmatic-follow-up).
[^production]: [Current production execution boundary](../skills/WORKFLOW.md#mandatory-execution-boundary).
