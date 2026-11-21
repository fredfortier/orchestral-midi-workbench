---
name: bitwig-mixing
description: Mix an open Bitwig arrangement using musical foreground, native mixer observations and rendered audio. Covers balance, pan, masking, depth, dynamics and verification, especially orchestral and contrapuntal music. Use for mixing or mix diagnosis; excludes score realization, patch loading and mastering-only requests.
---

# Mix a Bitwig arrangement

Read [the shared workflow](../WORKFLOW.md), then
[mixing techniques and measurement](references/mixing-techniques.md) before
making musical decisions. That reference separates
industry guidance, native mechanics and this workflow's editorial defaults.
Use the existing controller; do not scrape the mixer or create another bridge.

## Enter, decide, execute, return

| Request / finding | Action and return |
| --- | --- |
| General mixing advice, no project requested | Explain the relevant technique in the routed reference; no host action |
| Review only | Observe and render if authorized; report supported diagnoses without changing the mix |
| Mix the current project | Resolve it through `controller info`; establish foreground, reference/delivery intent and preserved content; proceed below |
| Need audio evidence | Invoke [bitwig-export-audio](../bitwig-export-audio/SKILL.md) with exact owners, range and analysis settings; return with completed receipts and verified audio |
| Missing/misconfigured receiver or wrong articulation | Record the acoustic finding; route exact setup to kontakt-setup or musical changes to bitwig-edit only within authorized scope; return after fresh observations |
| Unsupported native field/processing operation | Report the specific gap; do not invent a value, substitute UI automation or silently omit a required result |
| Uncertain mutation/export outcome | Query the original operation, independently reread/reconcile; never replay or Undo blindly |

1. Resolve project instructions and current `PERFORMANCE.md`, when present.
   Separate current musical intent, historical receipts and fresh host facts.
   For a literature-led interpretation, check piece-specific scholarship and the
   relevant score passages before deciding dynamics. Record source, locator,
   musical implication and editorial realization separately. Compare current
   curves with that interpretation; historical verification does not establish
   that a contour still serves the intended phrase or climax hierarchy.
   State routine assumptions; ask only if an unresolved choice materially changes
   musical intent, preservation or delivery compatibility. Do not reorchestrate.
2. Stop playback through the qualified native keyboard workflow if necessary.
   Read `midi-bridge controller mix read --output mix-before.json`. Retain complete
   owning clips through bitwig-explore for content/automation preservation. Read
   cues separately; join their beat positions to the mixer snapshot's `beatSeconds`
   for native tempo-aware audio windows. Existing parameter discovery owns receiver controls.
   Build the sounding scope from `state.active`, ancestor activation and routing,
   as well as mute/solo state. `effectivelyMuted: false` alone does not establish
   activation. Exclude deactivated tracks from mix decisions, edits and analysis
   exports unless the user requests them; preserve their complete captured content.
3. Collect an unnormalized full master and synchronized post-fader track/section
   exports through bitwig-export-audio. Include relevant returns and tails. Record
   exact processing taps: leaf exports do not include ancestor/master processing.
   Do not add overlapping leaves and group exports as if independent stems.
   Apply the routed reference's quantitative method: compare waveform/envelopes,
   spectra and
   relevant time-frequency views, target/comparison band levels and full-mix
   loudness/headroom before and after. Retain settings and evidence; spectral
   overlap by itself never triggers an automatic cut.
4. Build a phrase-level priority map: foreground, counterline, harmonic/rhythmic
   support, bass foundation, intended blends and room. Inspect sources, dynamic
   control laws, existing microphone perspective and processing before treating
   a missing attack or timbral defect as a fader problem.
5. Work in context: rough balance, justified pan/depth, phrase handoffs, then
   targeted EQ/dynamics only for demonstrated residual problems. Apply the
   smallest useful batch. Static native volume/pan uses:

   ```sh
   midi-bridge controller mix set --input balance.json --output mix-after.json
   ```

   `balance.json` contains `{ "expected": <complete fresh mixer observation>,
   "changes": [{ "path": <observed exact track path>, "volumeDb": -12.0,
   "pan": -0.15 }] }`. Omit the property not changing. Volume is dB; pan is native
   −1..1, not degrees. Numbers here illustrate syntax, not target balances.
   Automated/modulated mixer controls refuse static writes. Existing device
   automation changes use bitwig-edit's complete owning-clip contract. Performance
   controls default to shared track MIDI lanes from the INSTRUMENT.md control map. Do not
   overwrite performance dynamics with generic gain contours.
6. Independently reread the mixer and complete affected clips. Compare requested
   balance against actual readback and prove musical preservation. Render again
   after consequential changes. Compare at matched playback loudness, including
   mono/quiet playback and representative sections. If listening is unavailable,
   label the pass measurement-led and leave perceptual approval pending.
7. Save through the qualified native Save action; verify persistence separately.
   Record project-specific decisions/evidence in the project's existing musical
   document, never here. Return actual changes, before/after measurements,
   processing/export scope, preservation, save and listening limits. Do not call
   a source defect, a successful render or an aesthetically provisional pass a
   fully approved mix.
