# Exact path assignment

Pass [eligibility](eligibility.md), then execute the settled PERFORMANCE.md schedule.
[ARCHITECTURE.md](../../../controller/ARCHITECTURE.md#purpose-and-execution-boundaries)
owns the data and execution contract. This reference owns the control flow.

Create a target JSON path containing project, native groups, track, Instrument Layer
and optionally Layer. For example:

```json
{"project":"Piece","elements":[{"kind":"group","name":"Orchestra"},{"kind":"track","name":"Violin"},{"kind":"layer","name":"Instruments"},{"kind":"layerChain","name":"Sustain"}]}
```

`layer`/`layerChain` address an Instrument Layer and its slot;
`selector`/`choice` address an Instrument Selector and its slot. The target's kind
selects the container type. Existing Selectors retain their active-choice policy;
channel admission does not select the performing choice.

```sh
midi-bridge controller instrument layer ensure --target container.json --operation "$container_operation"
midi-bridge controller instrument slot ensure --target container.json --operation "$slot_operation" \
  --name "$musical_name" --input-channel 2 --input-channel 3
midi-bridge controller instrument assign --target slot.json --operation "$assignment_operation" \
  --load-patch "$absolute_nki" --wine-prefix "$wine_prefix" --wine-runtime "$windows_runtime" \
  --display "$display" --output "$fresh_evidence"
midi-bridge controller instrument read --target slot.json
```

Slot ensure targets a container to find/create the required name, or an exact
existing slot to rename/update it. Repeated `--input-channel` flags replace the
complete note admission set. Each call applies All → Same, stock Channel Filter
(shared 1 plus note channels; others off), stock Channel Map (all → 1), and verified
slot selection. Existing helper/device identities survive; other devices retain
relative order. No shipped or user presets are used. Omit ensure to preserve
manual routing tweaks. Duplicate helpers or ambiguous names refuse before mutation.

`--wine-runtime` takes the provisioned runtime's root directory containing
`.venv/Scripts/uv.exe`, not the `.venv` directory or the executable itself.
Validate that exact path before assignment; a preflight `notDispatched` result
does not mean a receiver was inserted or an NKI submitted.

Assignment works independently of routing topology. It reuses the sole Kontakt
wherever in the slot chain, or inserts Kontakt at the end when no instrument
exists. Multiple instruments refuse. Omit `--load-patch` and the loader runtime/
output options to preserve the current patch and configuration. An explicit patch
uses the integrated loader; never load merely to verify an existing configuration.
Use the slot path returned by ensure, refreshing native handles after reopening.
Conventional controls use native track-level MIDI automation; missing patch facts
return to intake. Setup returns curve edits to bitwig-edit.

Each operation ID identifies one intent. `instrument status --operation ID` queries
it without replay. A response loss is not evidence that the operation did not run.
The service performs a fresh full target read before mutation and validates native
readback. Runtime must belong to the admitted Bitwig PID/display/Wine prefix.

No selection helper, first-title window match, screenshot parsing, empty-player
insertion, alternate loader or manual intervention is part of this flow. The
controller qualifies the exact editor through native visibility and Win32 window
identity before sending NKI input. The external loader only loads that exact NKI.
The relative NKI beneath INSTRUMENTS_ROOT_DIR remains the musical patch identity.

For an authorized Syllabuilder caller, append `--keep-editor-open` to that same
integrated `instrument assign` command. Require `editorKeptOpen: true`, confirmed
load and exact owned-window/native visibility evidence; return the active editor
and settled binding to kontakt-syllable-builder. Do not run the loader separately.
Default assignment still closes the editor.

For a completed-insertion ownership failure or retained initial-render preflight
failure conclusively proving no NKI submission, retain `instrument status --operation ID` as a fresh JSON receipt.
After owner repair, the same integrated `assign` accepts `--resume-before-load
STATUS_JSON` with the original target and settled NKI, fresh operation/output and
optional keep-open flag. It validates the same persistent owner, receiver and routing, skips insertion,
requalifies ownership and claims that receiver once before loading. Authorized
removal of its stock legacy MIDI CC helper is admitted; replacement/additional
devices are not. Historical sibling edits do not relax fresh-scope preservation.
Changed scope, possible submission or a consumed claim refuses; do not edit the
receipt, recreate the receiver or use this branch for ordinary occupied slots.

Success means factual assignment, loader confirmation and the requested editor
disposition: verified closure by default, verified open for the builder handoff.
It does not prove missing samples are absent, saving, playback or musical approval.
An unknown load outcome requires reconciliation before another load.
