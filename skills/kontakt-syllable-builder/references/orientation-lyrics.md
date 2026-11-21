# Resolve the project and underlay

Use `midi-bridge controller info` to establish the active saved project and exact
voice/clip paths. Resolve the directory from its native saved file, not the shell's
working directory. Read local instructions and PERFORMANCE.md. Before an experiment,
follow the shared workflow's Save and single-file checkpoint procedure.

Use the caller's specified underlay; otherwise expect `performance.companion.json`
beside the saved project. Check source/rendition identity against the intended
piece and native notes. A stale bundle can still preserve lyrics, but is not the
current note/control authority. Record later native revisions before joining it.

```sh
jq -e '(.source_xml | type == "string" and length > 0) and
       (.note_sources | type == "array")' "$project_dir/performance.companion.json"
jq -r '.source_xml' "$project_dir/performance.companion.json" > "$evidence/source.musicxml"
jq '.note_sources[] | {source,onset,end,key,source_route,encoded_route}' \
  "$project_dir/performance.companion.json" > "$evidence/note-associations.json"
```

Parse MusicXML as XML, not with lyric regexes. The bundled extractor retains
part, measure index/label, note ordinal, voice/staff, lyric line, ordered lyric
children, ties and slurs; it attaches existing source-to-performance associations.

```sh
python "$skill_dir/scripts/phrase_plan.py" extract \
  --companion "$project_dir/performance.companion.json" \
  --output "$evidence/underlay.json"
```

Inspect the complete requested parts. Reconstruct word grouping and continuation
from text, `syllabic`, `extend`, elisions, ties and score context. Missing lyric text
does not automatically mean either a new syllable or a melisma. Review malformed
grouping and sparse extenders; retain the source and document editorial resolution.
Keep performed repetitions and independent lyric lines distinct.

Match performed occurrences to fresh native notes by documented part binding,
pitch, rational onset/end and any retained adaptation. Resolve ambiguity before
edits; do not use array order, timestamp coincidence alone or a different voice's
lyrics. If no authoritative underlay exists, report the missing input and stop
mapping. An explicitly supplied equivalent score/underlay may replace the companion.
