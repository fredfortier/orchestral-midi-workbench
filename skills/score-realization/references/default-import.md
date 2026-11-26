# Default score import into the current Bitwig project

Use the installed Workbench `convert` and `midi-bridge` for this workflow.
Keep the requested scope to conversion, placement and verification. Retain
the encoded channels and notes, use empty instrument tracks, and preserve
existing devices and routing. Apply any source reductions the user asks for
to a separate working score.

## Convert and preflight

1. Resolve the current native project and its document identity with the
   [single-instance preflight](../../bitwig-launcher/SKILL.md#single-instance-preflight).
   A request to import into the current project uses that document. For a
   conversion-only request, complete the file work without contacting Bitwig.
2. Resolve the full path of the installed Workbench `convert` and confirm
   its help. Use fresh filenames in the project's established working
   location. For an existing Workbench bundle, begin at decode.

   ```sh
   "$score_converter" "$working_score" --to midi2 -o "$midi2"
   "$score_converter" "$midi2" --validate
   midi-bridge controller midi decode --input "$midi2" --companion "$companion" > decoded.json
   ```

   The default companion uses the same stem with `.convert.json`. Omit
   rendition, binding and policy overrides for a converter-default request.
   Stop at the requested file output for conversion-only work. Report converter
   or decode failures at their owning step. The Workbench decode takes a
   MIDI 2.0 Clip File plus its companion. A Standard MIDI File (`.mid`) is a
   different input format. If supplied alone, report the missing Workbench
   SMF import capability.
3. Before the first live write, retain the decoded bundle and a placement
   plan identifying the document, each source part, its encoded group/channel,
   target track and clip, exact arranger range and expected note occurrences.
   Check one destination per intended part, including when parts share an
   encoded channel. Bitwig stores a channel on each note. Check for note
   collisions and conflicting control histories if distinct UMP groups share
   a channel on the same destination track. Preserve per-track controls where
   the encoded route and source metadata identify an unambiguous owner. Determine which encoded
   controls, expression and timing events the native path can represent.
   Record any unrepresentable event and keep that scope pending. If an essential
   destination action or mapping is unavailable, stop before creating its tracks
   or clips. Change project-wide tempo or meter only
   within the [qualified timing action](../../bitwig-explore/references/session-actions.md#direct-actions)
   and when the import scope permits it.
4. Use [qualified session actions](../../bitwig-explore/references/session-actions.md)
   for empty track and Arranger Note Clip creation. Verify stopped transport,
   destination ownership, Arranger focus, and absence of overlap. Verify
   the document immediately before each structural write. Retain its observed
   result and the created native path with the plan before proceeding.
   Keep the intended beat range explicit, including initial rests.

## Place and verify

Follow [native placement](native-import.md#route-and-compare-the-notes) for
mapping and quantization. Read all destination paths together with
`controller content read --paths PATHS.json` and use one complete
`controller content replace --expected BEFORE.json --input DESIRED.json`
for the supported note and automation scope. Reread those paths and compare
owner, geometry, note occurrences, channels and supported controls with
the plan. Save through the qualified action when requested and verify the
saved identity.

## Resume an interrupted import

Reidentify the same document and reread all planned destination paths
before writing. Reuse an empty track or clip only when its native identity
matches the retained creation receipt and its geometry still matches the
plan. Skip a populated destination only if complete readback matches the
intended placement. For an unknown replacement outcome, inspect the
[original operation and native readback](../../bitwig-edit/references/clip-replacement.md)
before proceeding. Leave any conflicting or ambiguously owned destination
pending. When a structural action has an unknown outcome and no verified
receipt, compare fresh hierarchy and geometry with the retained pre-action
observation. Reuse a target only if that comparison identifies its creation
unambiguously. Create only confirmed missing items and compare the entire
import scope after the final write.
