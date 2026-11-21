# Common complete-clip editing recipes

Use these recipes for ordinary note and automation edits. Example values are
illustrative; substitute the authorized musical change. Run each command only
after its predecessor succeeds. Keep before/desired/result/readback in a fresh
evidence directory; output files must not already exist.

## 1. Choose scope and read complete content

For the actual selected clip(s), including a selection of notes inside a clip:

```sh
midi-bridge controller midi get-all --output before.json > /dev/null
jq '[.clips[].path]' before.json > paths.json
```

This returns **all** notes, all per-note expression and all owning-track automation
for every included clip. `.notes[].selected` identifies the selected-note subset;
selection bounds are not a request to omit other content. For a whole selected
clip edit, operate on all intended notes; for a selected-note edit, explicitly
test `.selected`. Verify the returned clip paths match the requested scope.

For named owners independent of UI selection, follow
[path discovery](../../bitwig-explore/references/arranger-navigation.md), retain
the exact discovered paths in `paths.json`, then:

```sh
midi-bridge controller midi get-all --paths paths.json --output before.json > /dev/null
```

For **structural information**, use `controller info` (or `info --path path.json`)
through bitwig-explore. For available device parameters/Remote Controls and actual
CC assignments, follow its [parameter discovery](../../bitwig-explore/references/device-parameters.md).
Neither structural read replaces the complete content read or establishes that
a parameter has a stored curve. Existing-curve and note edits need no parameter
discovery or instrument setup.

## 2. Inspect or isolate data locally

The complete file is the edit base. Derived files are convenient views, not
replacement payloads. Preserve all fields in any isolated record intended for
reinsertion; projections for display are not lossless edit records.

Inspect coverage and selected-note counts:

```sh
jq '[.clips[] | {path, notes:(.notes|length),
  selectedNotes:([.notes[]|select(.selected)]|length),
  automationSources:([.automation[].source]|unique)}]' before.json
```

Extract complete note records with their clip owners, without changing the base:

```sh
jq '[.clips[] as $c | $c.notes[]
     | select(.selected)
     | {path:$c.path, clip:$c.id, record:.}]' before.json > selected-notes.json
```

Locate a parameter's curves using its discovered native parameter identity:

```sh
jq --arg parameter "$parameter_id" '
  [.automation[]
   | select(.target.nativeParameterId == $parameter)]
  | unique_by([.track,.lane,.clip])
' before.json > parameter-curves.json
```

This local view retains target, source, placement, aliases, mapping and **all**
points. `[]` means no stored curve for that parameter in the observed scope; it
does not mean the parameter/device is absent. Outside-clip arranger automation
and committed automation clips can coexist for one parameter. Choose the intended
observed source; do not merge their arrays or mistake a remainder for a second
musical phrase. Source and arranger beats differ when the mapping has offsets or
loops. Shared-source edits affect every acknowledged placement.

## 3A. Change notes while preserving automation

Example: set velocity to 0.6 for selected note onsets in source beats `[24,32)`:

```sh
jq '
  (.clips[].notes[]
   | select(.selected and .note.start >= 24 and .note.start < 32)
   | .note.velocity) = 0.6
' before.json > desired.json
```

The result remains a complete snapshot. Untargeted notes, all other note fields,
expression, automation and clip geometry remain present. Adapt the predicate
to the user's exact scope, rather than assuming every selected clip's notes were
individually selected. Count matching, already-correct and changed records before
dispatch; zero matches and an already-correct result are different outcomes.

To replace a clip's entire note collection, `replacement-notes.json` must contain
complete `{note: NoteEvent, selected: boolean}` records:

```sh
jq --arg clip "$clip_id" --slurpfile notes replacement-notes.json '
  (.clips[] | select(.id == $clip) | .notes) = $notes[0]
' before.json > desired.json
```

Preserve IDs and selection flags for retained notes. New notes need unique
temporary IDs and every required note field, including operators and all expression
families. Omitted notes are intentionally deleted; an empty replacement clears
that clip's notes. Automation is untouched. Do not build replacement notes from
display projections, key/time/velocity summaries or a different observation.

## 3B. Change existing automation while preserving notes

Example: change an existing native CC point at arranger beat 192 to MIDI value 84/127:

```sh
jq --arg source "$source_id" '
  (.automation[] | select(.source == $source)
   | .points[] | select(.time == 192) | .value) = (84/127)
' before.json > desired.json
```

This updates every repeated copy of that source in the payload and retains all
point identities and untouched point fields. Check that the point exists and that
84/127 is within the observed target's physical range. Do not apply an empty change.

For a complete curve replacement, keep all other content and replace only its
`points` array. `desired-points.json` contains complete Point records:

```sh
jq --arg source "$source_id" --slurpfile points desired-points.json '
  (.automation[] | select(.source == $source) | .points) = $points[0]
' before.json > desired.json
```

Retained point IDs mean update; new points use unique temporary IDs; omitted
points mean delete. Retain stable equal-time order and explicit interpolation,
curvature and spread. Clearing points may produce a verified native fallback;
it does not delete an outside-clip lane.

For named complete curves or **new lanes**, use the same underlying transaction
through `controller automation set --input curves.json --output after.json`.
Its [input recipe](automation-edits.md#set-track-cc-curves) avoids
manual snapshot bookkeeping: specify the track/channel/CC from INSTRUMENT.md,
supply complete desired points in native CC units, and let the CLI capture the full
current content internally. Device discovery is explicit opt-in beyond MIDI only. New curves
need no preliminary lane setup. Do not additionally run `midi replace` for the
same edit after `automation set`.

## 4. Apply one complete replacement

For either `desired.json` recipe, use the same command:

```sh
midi-bridge controller midi replace \
  --expected before.json --input desired.json --output after.json > /dev/null
```

The expected state includes notes **and** automation. One native transaction
preflights identities, ranges, source ownership and preserved content, then applies
the batch and independently reads it back. Notes-only and automation-only edits
do not have different write authorities or reduced transfer schemas. See
[the replacement contract](clip-replacement.md) for stale/unknown outcomes.

## 5. Independently reread and check preservation

```sh
midi-bridge controller midi get-all \
  --paths paths.json --output verified.json > /dev/null
```

For the existing-record examples above (no creation or native canonicalization),
check the full expected clip records, including every unrequested domain:

```sh
jq -e -n --slurpfile wanted desired.json --slurpfile actual verified.json '
  $wanted[0].document == $actual[0].document
  and $wanted[0].clips == $actual[0].clips
'
```

The observation revision may advance; explicit-path and selection reads also have
different selection-envelope descriptions. Never patch authority fields merely
to make snapshots compare. Compare requested clip membership and complete clip
records, with native selection preservation also verified by the transaction.
For new objects or fallback canonicalization, `after.json` contains the resulting
native IDs/values; the transaction already validates its internal temporary-ID
mapping. Independently locate new curves by exact owner/parameter, compare the
requested point properties and check every untouched record exactly. Do not expect
temporary IDs to survive as native IDs or strip identities from preserved data.
Report counts, readback and preservation; save only when requested.
