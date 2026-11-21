# Locate items and select native context

1. **CONTROLLER** Read `midi-bridge controller info > entities.json`, or use
   `info --path path.json` for an already resolved subtree. The latter resolves
   that owner directly and reads only its descendants. Neither reads musical
   arrays or parameter catalogues.
2. **CONTROLLER** Narrow by exact names and containment locally. Retain returned
   typed paths and `nativeId` discriminators; names containing separators remain
   literal strings. Missing, ambiguous or stale paths refuse. A lane's displayed
   `metadata.index` is a locator in that observed track, not identity.
3. Choose the next operation below. No selection is needed for an explicit read.

Actual containment can include:

```text
Orchestra / Strings / Violino I
Orchestra / Strings / Violino I / Pleni
Orchestra / Strings / Violino I / Expression automation
Orchestra / Strings / Violino I / Instrument Layer / Portato
```

## Understand musical content

**CONTROLLER** Save an array of requested track/group-channel or clip paths, then:

```sh
midi-bridge controller midi get-all --paths paths.json --output before.json > /dev/null
```

Omit `--paths` only to read the effective native musical selection. Reads never
move selection. A track/group path reads its own complete clips and automation;
use explicit descendants for child tracks. Tracks without note clips can own
content. `.owners` lists track owners; `.paths` retains explicit scope, or null
for a selection read. `.clips` contains complete notes/expression; `.automation`
contains complete Lane records once, alongside clips. Preserve this entire file.
See [musical queries](musical-queries.md) or [automation](automation-queries.md).

## Deliberate native selection

**CONTROLLER** `midi-bridge controller selection read > selection.json` returns
native Inspector membership, primary item, actual active editor, object/time
mode, range and fresh
`document`/`revision` authority. Unsupported members are explicit and cannot be
silently interpreted as an empty supported selection.

Build one absolute request from the returned paths and current authority:

```json
{"action":"set","document":"OBSERVED","revision":"OBSERVED",
 "paths":[],"mode":"objects","range":null}
```

**CONTROLLER** `controller selection set --input request.json` replaces membership;
an empty object set clears the active supported selection. There are no add,
remove or separate clear modes. Put the intended primary first. Object targets
are track headers, arranger clip objects, devices or supported slots. Multiple
targets must share a compatible native selection owner. Time mode accepts track
or automation-lane paths and `range:{"from":8,"to":12}` in arranger beats.
Bounds use native 2^-20 beat precision. Incompatible combinations refuse before
selection changes. Stale authority requires a fresh read, not automatic retargeting.

**CONTROLLER** Reread selection independently. Confirm membership, primary,
editor and mode separately. **COMPUTER** Establish keyboard focus only when the
next [recipe](keyboard-workflow.md) requires it. A selected clip does not imply a
selected track header; a selected lane does not imply a point/time selection.
Unknown outcomes require readback before further action.

The reported editor may remain `none` while a clip or time range is inspected;
that is a separate native fact, not missing membership. Set finishes after native
mode/focus reconciliation. Establish the recipe's real panel focus through
**COMPUTER**, then reread before its keyboard action. Time selection can move the
playback cursor to its start; disclose that side effect when preserving context.
