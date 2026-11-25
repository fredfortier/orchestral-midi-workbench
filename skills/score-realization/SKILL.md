---
name: score-realization
description: Convert a score with the Workbench converter, import a MIDI2 rendition into Bitwig, or realize an orchestrated performance with explicit rendition decisions. Excludes workspace intake, Kontakt loading/configuration and researched cue sheets.
---

# Score realization

For conversion or import with the converter's defaults, follow [default score
import](references/default-import.md). That procedure also owns placement of
an existing Workbench MIDI2 bundle when no musical changes are requested.

## Orchestrated realization

Own the musical decisions and their
translation into native editable material. Stop at verified content/routing and
save; do not automatically load Kontakt, research an arc, render or seek approval.

```text
existing workspace + source + musical direction
 -> review score gestures, continuations and neighboring notes
 -> assign exact receiver and channel to every reviewed occurrence
 -> PERFORMANCE.md + independent passage expectations
 -> convert -> compare decoded routes to expectations
 -> native placement -> verify channels, receiver filters and note coverage
```

| Need | Read |
| --- | --- |
| New or changed musical interpretation, including a cue-led expression plan | [Interpretation](references/interpretation.md) |
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
   from initialized library documentation. Read the exact articulation's
   Conventional MIDI control map and stock dynamics and expression baselines.
   Missing/stale controls return to documentation-only intake. If a required
   effect remains unresolved, evaluate eligible patches or mark the effect pending.
3. Own PERFORMANCE.md's realization, binding and approximation sections using
   the [shared template](../score-intake/assets/PERFORMANCE.md). Specify each
   track/channel's musical purpose, patch identity, documented stock behavior
   and supported control law. Anchor its curves to the score and musicological
   decisions using the shared
   [expression control specification](../bitwig-edit/references/dynamics-and-expression.md#apply-the-distinction).
   Receiver setup is limited to native routing and exact NKI assignment/loading;
   it does not customize patch settings.
   Carry the intended audible result into the plan for each material cue-led
   change. Preserve cue/configuration sections. Stable binding keys link readable
   native track/layer-chain names to details. Identify patches solely by NKI paths
   relative to INSTRUMENTS_ROOT_DIR.
4. Compile with convert. Check the required effects against the plan before
   conversion and against the decoded bundle before import. A valid bundle or
   audible stem is insufficient evidence for an omitted effect.
   Required routes, velocities, gates, controller histories
   and supported expression must be encoded before import. Preserve original
   notation and working-score changes. Unsupported musical requirements remain
   explicit; never flatten curves or substitute articulations to obtain a pass.
   Understand the exact receiver for every track and articulation. Use its
   documented standard MIDI controls for their intended musical roles and
   explain the resulting curves in PERFORMANCE.md. Consider patch-specific
   macros only in a separately authorized refinement after MIDI2 orchestration.
   A successful rule match proves encoding. Before placement, reconcile every
   performed occurrence against the [reviewed gestures](references/interpretation.md),
   including continuation spans, repeated sections and channel boundaries.
5. Freshly observe destination, review the plan and preservation scope, then use
   admitted native operations. Use the destination group declared in the
   companion, or the project root when no group is specified. Declare named
   receiver slots independently of NKI resources; each admits its
   complete note-channel set through the shared standard Filter/Map topology.
   Reserve channel 1 for shared controls on articulated tracks; use 2–16 for notes
   and keyswitches. A single-channel nonarticulated part may use 1 for everything.
   Incompatible independent histories require explicit routing reconciliation.
6. Check intended effects against the decoded bundle and independently compare
   notes, expressions, stored controls, tempo/meter, clip mappings and routing.
   Counts alone are insufficient. Verify save/path/content
   through supported operations. Report completed versus pending scopes and stop.

Review the complete device schedule against the shared execution boundary before
its first write. Use [native import](references/native-import.md) for content and
[kontakt-setup](../kontakt-setup/SKILL.md) for integrated NKI loading when requested.
Leave any placement that conflicts with the shared execution boundary pending.
Preserve existing music and keep interpretation within the requested scope.
