---
name: bitwig-export-midi2
description: Capture an active Bitwig project or track group, interpret its musical routing and automation, assemble a self-contained MIDI2 file, validate it, and optionally reconstruct an MXL score. Use for DAW-to-score export; excludes audio export, score import and live musical edits.
---

# Export Bitwig musical content

Use the [shared workflow](../WORKFLOW.md). Run from `crates/`, using the configured
`midi-bridge` service and `./target/debug/convert`. Build the existing locked
converter only if absent. Export is read-only; interpretation edits a JSON copy.

## Short execution path

1. **Scope and capture.** Run `midi-bridge controller info` only when a requested
   group/track needs resolving. Save its exact returned Path to `scope.json`;
   ambiguous names require resolution, not choosing the first match.

   ```sh
   midi-bridge controller midi capture --output capture.json
   ```

   Add `--path scope.json` for a group/track. Default excludes inactive tracks and
   descendants of inactive groups; mute/solo are separate facts. Add
   `--include-inactive` only when requested. Capture failure is not permission to
   drop an included owner's automation. Preserve this observation unchanged.
2. **Observe once, choose only necessary edits.** Open
   [inspection and decision recipes](references/interpretation.md), run its compact
   inventory, then only the filters selected by that inventory. Use its decision
   table and ready-made jq recipes. Direct assembly is the default. Unknown plugin
   controls remain residuals; unknown slot names remain remarks. Do not research
   speculative equivalences, inspect plugin state, write helper scripts or build a
   generic ETL pipeline. Use musical judgment only for the concrete choices the
   capture/user evidence supports. Record material edits in `decisions`.
3. **Assemble and inspect admission.** Use `capture.json` directly when no edits
   apply; otherwise use the final editorial copy as `interpreted.json`.

   ```sh
   midi-bridge controller midi assemble --input interpreted.json --output performance.midi2 --report assembly.json
   ./target/debug/convert performance.midi2 --validate --report validation.json
   jq '{valid,admissible,notes,parts,pitchRange,controllers,diagnostics}' validation.json
   ```

   Malformed input exits 1; valid but inadmissible score input exits 2. On failure,
   inspect only the named owner/field using the reference filters. Apply a known
   correction and retry to fresh destinations. If required meaning remains
   unknown, report that blocker; do not guess, silently omit it, or start an
   exploratory scripting/research loop. Host/wire failure requires restoring the
   exact supported controller deployment before recapture.
4. **Convert if a score was requested.** Admission must pass first.

   ```sh
   ./target/debug/convert performance.midi2 --to mxl -o score.mxl --report notation.json
   ```

   Check notation decisions and independent MusicXML schema validation. When
   assessing readability, render opening, one dense passage and closing using
   existing tools. Stop at representative checks; publication engraving is a
   separate task. Reports/working JSON are evidence, never required companions.
5. **Return.** Give artifact paths, scope/exclusions, counts, material decisions,
   checks and unresolved losses. If timing is requested, record wall time for
   capture, inspection/edits, assembly, validation and conversion separately;
   summarize the decision made at each step in one sentence.

For changed interpretation, revise the copy and assemble to fresh paths. Recapture
only when current host content is needed; never reuse capture as mutation authority.
No Syllabuilder extraction, live slot renaming or audible-equivalence claim follows
from export. Standard lyrics need actual syllable timing; named phrases use remarks.

For a concrete diagnostic not covered by the recipes, consult only its relevant
section in the [controller contract](../../controller/EXPORT_MIDI.md)
or [conversion contract](../../convert/ARCHITECTURE.md). These own schema and
representation policy; they are not mandatory research before routine export.
