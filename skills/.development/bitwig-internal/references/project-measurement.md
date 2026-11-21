# Inspect native content without a project graph

[ARCHITECTURE.md](../../../../crates/controller/ARCHITECTURE.md#one-complete-musical-snapshot) owns the
schema. `controller info` reads typed folders; `controller midi get-all` reads
complete selected clips, or `--paths paths.json` reads explicit clips without UI
selection changes. Notes, expressions and all automation always travel together.

Use jq over the saved complete JSON for counts, extents, channels and curves.
Return clip paths with statistics. Distinguish stored source notes from expanded
loop playback; preserve source/arranger offsets and actual units. No whole-project
property graph, summary/paging protocol, helper CC device or data optimization
belongs here. Roughly 1 MB prompts inspection for junk, not a throughput project.

Offline `controller midi decode --input FILE --companion FILE` authenticates the
actual MIDI2 file plus companion. It does not establish native placement or audible
receiver behavior. Historic graph/export/import probes remain source evidence in
SOURCES.md; they are not current operations.
