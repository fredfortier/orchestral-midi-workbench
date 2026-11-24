---
name: bitwig-export-midi2
description: Capture an active Bitwig project or track group, interpret its musical routing and automation, assemble a self-contained MIDI2 file, validate it, and optionally reconstruct an MXL score. Use for DAW-to-score export; excludes audio export, score import and live musical edits.
---

# Export Bitwig musical content

Use the configured `midi-bridge` service
and the installed musical converter. Set `score_converter` to its exact
executable path. Export is read-only. Make interpretation edits in a JSON copy.

## Required capture for the requested scope

A complete, successful native capture of the exact requested scope is required
before assembly, validation, conversion, or dependent score analysis. If capture
fails, retain the command, scope, and exact diagnostic. Narrow reads may isolate
the failing owner for diagnosis only. Report the failed scope and affected
deliverables explicitly, then resume only after the owning operation is fixed
and the same full-scope capture succeeds.

Never splice partial track captures or scores, substitute archived material,
synthesize a capture from other reads, or remove an included owner or automation
to get past a failed capture. A matching note count or event comparison does
not qualify a substitute. A narrower export is a different deliverable and
requires explicit user direction. Disclose all authorized exclusions,
interpretation decisions, unresolved losses, and failed checks. Deliver an MXL
as verified only after admission, conversion, and the independent schema check
all pass for that scope. A time target does not change these requirements.

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
   "$score_converter" performance.midi2 --validate --report validation.json
   jq '{valid,admissible,notes,parts,pitchRange,controllers,diagnostics}' validation.json
   ```

   Malformed input exits 1; valid but inadmissible score input exits 2. On failure,
   inspect only the named owner/field using the reference filters. Apply a known
   correction and retry to fresh destinations. If required meaning remains
   unknown, report that blocker; do not guess, silently omit it, or start an
   exploratory scripting/research loop. If the controller is unavailable,
   retain the results already obtained and report the missing prerequisite.
4. **Convert if a score was requested.** Admission must pass first.

   ```sh
   "$score_converter" performance.midi2 --to mxl -o score.mxl --report notation.json
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

Use the [interpretation recipes](references/interpretation.md), the actual
diagnostic, and the current CLI help to resolve an export failure. If the
available operation cannot preserve the intended music, report the affected
tracks or events and retain the original capture.
