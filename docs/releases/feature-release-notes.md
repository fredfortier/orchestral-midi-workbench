# Feature release notes

## Unreleased

- In `midi-bridge controller`, replace `midi get-all` with `content read --selection`
  or `content read --paths PATHS.json`. Use `content replace` with the original
  and edited snapshots to apply changes. For device parameters, replace
  `automation read --target DEVICE.json` with `info --path DEVICE.json --parameters`.
- When restoring a score, `convert` checks that an explicit companion matches
  the MIDI clip. For custom receiver routing during conversion, supply a
  rendition plan with `--rendition-plan` in place of `--bindings`.
- The controller service can host DrivenByMoss OSC feedback. Use
  `midi-bridge osc` to send typed commands and observe selected feedback addresses
  from the same service process.

## v0.1.0 · Prototype

- Convert MusicXML and MXL to MIDI 2.0 Clip Files. The converter implements file encodings from the MIDI-CI Orchestral Articulation Profile and produces a companion file for source-aware restoration.
- Validate clips and inventory notes and articulations. Reconstruct MusicXML or MXL directly from the clip for notation and analysis.
- Use the production skills to realize a score, route articulations and shared expression to Kontakt in Bitwig Studio 6.1, and export an edited arrangement.
- Try the original two-measure violin example and run the pinned converter verification suite from a fresh checkout.

The demo covers file conversion. The live adapter targets the exact 6.1 build
and the Kontakt 7 workflow uses a provisioned Wine runtime on Linux.
[Verification and setup](../CICD.md)
