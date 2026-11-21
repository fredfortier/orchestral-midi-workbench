---
name: score-realization
description: Interpret a supplied score, choose documented instrument patches, compile an explicit MIDI rendition, and place verified editable notes, automation and channel-layer routing in its Bitwig project. Excludes workspace intake, Kontakt loading/configuration and researched cue sheets.
---

# Score realization

Follow [the shared workflow](../WORKFLOW.md). Own the musical decisions and their
translation into native editable material. Stop at verified content/routing and
save; do not automatically load Kontakt, research an arc, render or seek approval.

```text
existing workspace + source + musical direction
 -> read complete texture -> select documented variants
 -> PERFORMANCE.md + supported conversion inputs
 -> convert -> MIDI2 + companion -> native placement -> independent audit
```

| Need | Read |
| --- | --- |
| New or changed musical interpretation | [Interpretation](references/interpretation.md) |
| Discover libraries / choose NKI variants | [Instrument selection](references/instrument-selection.md) |
| Lyrics or vocal patches | [Vocal realization](references/vocal-realization.md) |
| Compile and validate the rendition | [Conversion](references/conversion.md) |
| Construct, reconcile, revise or audit native content | [Native import](references/native-import.md) |

1. Reuse project-local instructions, actual saved project identity and original
   source receipt. Missing workspace -> [score-intake](../score-intake/SKILL.md)
   and return. Unsupported score format needs an explicit supported conversion
   path; intake of a PDF does not make it MusicXML.
2. Read all relevant parts through the ending. Establish source instructions,
   editorial choices and receiver adaptations separately. Select exact variants
   from initialized library documentation and its Conventional MIDI control map;
   missing/stale controls return to documentation-only intake.
3. Own PERFORMANCE.md's realization, binding and approximation sections using
   the [shared template](../score-intake/assets/PERFORMANCE.md). Specify each
   track/channel's musical purpose, patch identity, documented stock behavior and supported control law.
   Receiver setup is limited to native routing and exact NKI assignment/loading;
   it does not customize patch settings.
   Preserve cue/configuration sections. Stable binding keys link readable native
   track/layer-chain names to details. Identify patches solely by NKI paths relative
   to INSTRUMENTS_ROOT_DIR.
4. Compile with convert. Required routes, velocities, gates, controller histories
   and supported expression must be encoded before import. Preserve original
   notation and working-score changes. Unsupported musical requirements remain
   explicit; never flatten curves or substitute articulations to obtain a pass.
5. Freshly observe destination, review the plan and preservation scope, then use
   admitted native operations. One top-level realization group contains musical
   tracks. Declare named receiver slots independently of NKI resources; each admits its
   complete note-channel set through the shared standard Filter/Map topology.
   Reserve channel 1 for shared controls on articulated tracks; use 2–16 for notes
   and keyswitches. A single-channel nonarticulated part may use 1 for everything.
   Incompatible independent histories require explicit routing reconciliation.
6. Independently compare notes, expressions, stored controls, tempo/meter, clip
   mappings and routing; counts alone are insufficient. Verify save/path/content
   through supported operations. Report completed versus pending scopes and stop.

Review the complete device schedule against the shared execution boundary before
its first write. Use [native import](references/native-import.md) for content and
[kontakt-setup](../kontakt-setup/SKILL.md) for integrated NKI loading when requested.
A conflicting schedule requires repair before dispatch. Preserve existing music;
do not add humanization or a listening campaign without a request.
