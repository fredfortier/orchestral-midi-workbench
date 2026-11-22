# Apply the composer's intended edit

Before mutation, establish the target owners and intended change, plus what must
survive: timing, duration, routing, dynamics, operators, expression and untouched
notes. Ask only when unresolved scope changes musical behavior or preservation.
Keep work within existing authorization. An open user project is not a disposable
test fixture; do not run random campaigns or history experiments on it.

## Observe, compile, apply, read back

1. For named owners, resolve exact paths and use
   [complete replacement](clip-replacement.md): read once, edit the full JSON,
   replace once. This covers notes and automation without changing selection.
2. For an actual selected-note request, read the effective selection without
   `--paths`. Sparse commands use only that selection's IDs/document/revision;
   explicit-path IDs do not authorize a selection-scoped sparse command.
3. Review affected counts, IDs, ranges and preservation before dispatch. Retain
   before/desired/result, then independently reread the same scope and compare
   requested changes plus all untouched content.

On stale refusal, reobserve and recompute against the user's current scope; do
not merely substitute a new revision into an old plan. On disconnect/unknown
outcome, inspect current state and reconcile evidence before any further write.
Do not automatically repeat a command or issue Undo after an uncertain result.
Undo affects the project's current native history entry, possibly another edit.

## Choose the operation

| Intent | Command and scope |
| --- | --- |
| Transpose the complete effective selection | `controller note transpose --document … --revision … --semitones …` |
| Sparse change to one selected note | `controller note update --document … --revision … --clip … --note … --changes changes.json` |
| Same sparse change to every selected note | `controller note set --document … --revision … --changes changes.json` |
| Different changes to a queried subset | `controller apply --plan plan.json` |
| Insert/delete/replace notes | Inspect `controller note create/delete/replace --help`; creation requires complete `NoteEvent`, replacement affects selected notes in its owner |
| Intentional bounded humanization | Inspect `controller note randomize --help`; choose explicit fields, ranges, count and seed from the musical intent |
| Per-note expression point CRUD | Inspect `controller point --help` and its action's help; requires clip/note/family ownership |
| Authorized user Undo/Redo | [Session history](../../bitwig-explore/references/session-actions.md#direct-actions) for an established adjacent history entry. |

Sparse changes are typed arrays, e.g. `[{"field":"velocity","value":0.6}]`.
Native channel bounds are 0–15. Do not round-trip a reporting projection through
`note replace`: it loses omitted properties, operators and expressions. Use exact
native values in plans; do not turn presentation rounding into quantization.

## Example: compile a phrase velocity plan offline

Run this filter with `jq -f plan.jq selection.json > plan.json`. It sets velocity
to 0.6 for selected note onsets in source beats `[24,32)`. Adapt to the request.

```jq
. as $s
| {document:$s.document, revision:$s.revision,
   edits:[$s.clips[] as $c
     | $c.notes[]
     | select(.selected and .note.start >= 24 and .note.start < 32)
     | select(.note.velocity != 0.6)
     | {type:"update", clip:$c.id, note:.note.id,
        changes:[{field:"velocity", value:0.6}]}]}
```

Review the affected count with `jq '{document,revision,edits:(.edits|length)}'
before executing `midi-bridge controller apply --plan plan.json` once. If no edits
remain, distinguish already-correct values from no matching notes; do not apply
an empty plan. IDs/revisions expire after intervening changes.

For duration-filtered channel edits, inspect exact durations first: a literal
sixteenth is 0.25 quarter-note beats; user channel 6 is native 5. Humanized/gated
lengths need an explicit matching policy. Skip already-correct notes and report
eligible/already-correct/changed totals per owner. Stop after a failed command or
preservation mismatch before parsing dependent output, navigating or writing.

## Expression and automation boundaries

Readable note expressions include pressure, pitch, gain, pan, timbre and formant.
Point-edit commands expose pressure, pitch, gain, pan and timbre; do not infer a
formant write operation from its presence in JSON. Point time is in the note's
expression content, with `contentStart` preserved. Pitch uses semitones; gain
uses native cube-root amplitude. Preserve equal-time point order, curvature and
spread unless the edit intends to change them. Randomizing point properties
requires existing curves; ordinary humanization should not randomize MIDI
channels or techniques unless requested.

`controller point` edits per-note expression. For track automation point CRUD or
smoothing, use [automation edits](automation-edits.md): `controller midi replace`
has its own complete observation, revision fence and atomic source-curve plan.
`get-all` is one complete observation for notes, expression and automation. Its
document, revision, full content and source ownership are the expected state for
the combined `midi replace` transaction; there is no separate automation authority.
