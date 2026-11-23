# Discover device parameters and Remote Controls

Use this read-only flow when asked to inspect a device, for the
[instrument macro fallback](../../bitwig-edit/references/automation-edits.md#instrument-macro-fallback),
or to identify legacy sender assignments during an authorized migration.
Discovery never implies sender insertion, automation creation or receiver loading.

## Resolve the exact device

**CONTROLLER** Read layout, narrow to the relevant track, then resolve the exact device.

```sh
midi-bridge controller info > entities.json
jq --arg track 'Soprano - ARVA Boys' --arg chain 'Ah legato' '
  [.. | objects | select(has("metadata"))
   | select(.kind == "device" and .metadata.name == "MIDI CC")
   | select(any(.path.elements[]; .kind == "track" and .name == $track))
   | select(any(.path.elements[]; .kind == "layerChain" and .name == $chain))
   | .path]
  | if length == 1 then .[0] else error("Expected one qualified device") end
' entities.json > device.json
```

`info` returns `Entity = {path, kind, metadata, children}`. `path` retains every
native Group Track, Track, device container and chain. Reuse its `nativeId`
discriminators; do not strip them, flatten ancestry or choose the first duplicate.
Include additional ancestor criteria when names occur in several groups. For
plug-ins select the discovered `plugin` entity with its registry ID/version.

## Read the available parameters

**CONTROLLER** Request this catalogue explicitly; structural `info` omits it.

```sh
midi-bridge controller automation read --target device.json > device-parameters.json
jq '.metadata.parameters[] | select(.midiController == 1)
    | {name:.target.name, parameter:.target.nativeParameterId,
       controller:.midiController, value, automatable,
       min:.target.min, max:.target.max, unit:.target.unit}' device-parameters.json
```

The CLI returns the complete device Entity and its parameter descriptors; the jq
command only formats the already retained local JSON. It does not reduce the
native read or replace a complete observation.

| Field | Meaning |
| --- | --- |
| `path` | Exact device containment and current identity discriminators |
| `metadata.parameters[].target` | Device path, native parameter ID, native name, physical range/domain/unit and point representation |
| `metadata.parameters[].value` | Current native parameter value; not a stored curve or acoustic measurement |
| `metadata.parameters[].automatable` | Native automation eligibility |
| `metadata.parameters[].midiController` | Actual controller assignment for a MIDI CC device value parameter; absent on unrelated parameters, −1 means disabled |
| `metadata.remoteControls[]` | Device/preset page, rendered control label, zero-based index and underlying native parameter ID |

Join `remoteControls[].nativeParameterId` to
`parameters[].target.nativeParameterId` to identify the underlying parameter.
A Remote Control is a mapping to that parameter, not another independent CC.
Parameter discovery includes available continuous controls without stored curves.
Do not infer assignments from parameter position, labels, factory defaults or the
presence of a MIDI CC device. Apply the same process to an instrument's exposed
parameters and Remote Controls; no MIDI CC helper is required for those controls.

For example, a result with `name: "Knob 2 Value"` and `midiController: 1`
establishes that the second value parameter sends CC1. It does not establish that
the receiving patch interprets CC1 as dynamics; use the user-specified binding and
documented receiver semantics for that musical claim.

## Inspect curves and hand off an edit

Resolve the owning track paths from the same entity tree, then read their
complete content once:

```sh
midi-bridge controller midi get-all --paths paths.json --output before.json > /dev/null
```

This includes every note, expression curve, owning-track automation curve and
source/placement/loop mapping. Match a parameter curve by its observed
`target.nativeParameterId`, not merely its display name. Preserve the full payload;
statistics and display projections are not edit inputs.

If an edit is requested, return the device path, exact parameter identity/name,
actual assignment, units and complete content to
[bitwig-edit](../../bitwig-edit/SKILL.md). Its atomic `automation set` convenience
command discovers/resolves targets, obtains a fresh complete clip snapshot and
uses the same complete-content replacement. Missing curves do not require setup.
Do not route ordinary CC automation to sender insertion. Exploration ends with
verified facts, not mutations.

## Inspect the exact Kontakt editor — observed 17 November 2026

Explicit user instruction authorized direct Kontakt UI inspection for the
Electronic Continuo transcription and requested repeatable practice. This is a
bounded observed exception to the ordinary setup workflow; no patch was loaded,
source remapped, or receiver control changed.

1. **CONTROLLER** Resolve the track and plug-in with `controller info`; read the exact Layer slot
   and discover device parameters first. The native parameter catalogue identifies
   controls, but this build did not expose the loaded NKI/snapshot or per-key map.
2. **COMPUTER** Bind the Bitwig project window with `cua.getApp({windowId})`. Its AX tree was
   empty (accessibility disabled); use fresh screenshots and window-relative input.
3. **CONTROLLER** Select the exact track/device path; **COMPUTER** establish device-panel focus. The historical recipe clicked the track header to show its device chain. Clip selection and
   device/track focus are different: the selected note clip initially left a
   different Cembalo receiver visible below. Focusing the track also changed
   record-arm/focus state under this profile; do not claim whole-UI preservation.
4. **CONTROLLER** Reread selection after the focus action and verify the intended
   track/device/slot context. **COMPUTER** Confirm the device header belongs to
   that exact receiver before opening it. In the Kontakt device, the small bottom-left square button opens **Show
   plug-in window** (observed at Bitwig-window coordinate 839,1297 in a 1654x1403
   screenshot). The top orange circular icon instead shows plug-in parameters;
   it is not the editor toggle. Coordinates are session-specific.
5. **COMPUTER** Refresh `cua.listWindows()` after opening. The qualified title included the
   exact project group / Electronic Continuo / Instrument Layer / Instrument /
   Kontakt 7. Bind that window separately and verify its project/group/track/slot title against
   the requested path before inspecting any map. A Bitwig-window screenshot showed a
   clipped/offset editor; the separately bound 1357x898 Kontakt screenshot showed
   the complete interface and reliable local coordinates.
6. **COMPUTER** Read the rack instrument title and snapshot row. Here: Analog Hybrid Ensemble
   Designer / Cinematic Hybrid Trailer Kit (Hyped). Closing the Quick-Load pane
   with its x revealed the bottom Stage/Source/Settings/MasterFX tabs. Scrolling
   alone was not needed to expose those tabs after that closure.
7. **COMPUTER** With Follow MIDI already on, clicking the **embedded instrument keybeds** on
   Stage selected keys and updated the displayed note/source name. The lower
   white-key area and upper black-key area must be distinguished. Settings retains
   those keybeds and reveals the selected key's Performance indicator alongside
   the global MIDI Vel, RR and velocity-random controls. Inspect each used key;
   a displayed Repeater tab and saved rate are not proof Performance is enabled.
8. Observed map: C2/48 Firm Kick, F#2/54 Slap Snare, E3/64 Big Snizzle Hat,
   C#4/73 Super Tom Mid Low, D4/74 Super Tom Mid High, D#4/75 Super Tom High.
   Each inspected Performance power indicator was unlit; RR lit, MIDI Vel and
   velocity randomization unlit. This is snapshot-specific visual evidence,
   not a universal library map, measured dynamics calibration or audio audition.
9. Source browser navigation was inspected but not used to load/remap anything;
   its list could show a different browsing category from the current mapped
   source. The browser's highlighted candidate is not the loaded source authority.
10. **COMPUTER** Return to the Bitwig window and toggle the same Show plug-in window button
    off; verify closure. **CONTROLLER** Reobserve complete musical content
    before mutation because focus/selection changed. Use native full readback for
    MIDI/automation verification. Do not use screenshot labels as note-readback.

Limit: this established this receiver's UI identity and map with direct user
authorization. It did not qualify arbitrary Kontakt loading, window recovery,
script mutation, source persistence, sample completeness or audible sound.
