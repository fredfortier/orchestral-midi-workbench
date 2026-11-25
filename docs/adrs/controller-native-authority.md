# Controller native authority and qualification

Status: Accepted (recovered)

## Context

Recovered from `e0e0e1d:crates/controller/WORKPAD.md`. Original qualification
reports and earlier validation files are in the retired MIDI2.0Workbench
commit `9f549d9d69453de4501744913d342a95c3c516de` under
`crates/controller/qualification/` and `crates/controller/validation/`.
Retired OSC/automation task accounts and earlier failed observations are at
`2b705a53484dc6472f17f9572b2dea9666c54aa7:crates/.agent/` in that repository.

## Decision

Use one native authority for live musical state. Plan changes against complete
owned snapshots and apply them in one native Undo group. Use absolute Paths
for selection. Retain automation alongside clips so owners without notes
remain addressable. Native selection, keyboard focus and content authority
have separate scopes.

## Consequences and evidence

- Isolated Java probes separated SDK lifecycle and transport from document-thread
  access. Published proxy state exposed different scope from the native editor.
  Build-specific evidence established authority for each native operation.
- Receiver-local CC devices were tried and retired in favor of native track
  MIDI for conventional controls. Standard slots use stock Filter/Map.
  Separate slot ensure and assignment allowed an existing receiver to retain
  its identity and surrounding devices.
- A native-visible but unavailable Wine editor left a receiver inserted before
  NKI submission. The retained pre-submit evidence justified one-shot
  continuation under unchanged authority. It supplies no general replay path.
  Failed editor, render and keyboard trials describe their original fixtures.
- Standalone Kontakt preprocessing retained path-based coverage after a clean
  exit. This kept completed library work reusable across patch library updates
  where the same paths were already covered. Hosted NKI loading remains a
  separate transaction. Earlier rack-image calibration records describe retired
  settings experiments and their original rendering conditions.
- Native mixer and WAV export entered as bounded exceptions to the earlier
  MIDI-only scope. A failed native render left a partial WAV, establishing
  why native completion must accompany independent file checks.
- The retained navigation report records wire 70, adapter 0.4.29 and harness
  0.3.0. It qualified scoped selection, rollback and preservation on covered
  fixtures, with explicit focus and evidence limits. It describes the observed
  builds and projects. Current qualification follows the package contract.

[Current controller contract](../architecture/controller.md)
