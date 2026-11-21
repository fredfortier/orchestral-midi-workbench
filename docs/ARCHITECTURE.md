# Architecture

Use `$dexlabs:durable-documentation` to maintain this map and the package contracts.

## Overview

```text
MusicXML / MXL --> convert --> MIDI 2.0 rendition --> midi-bridge
                     ^                                    |
                     |                                  Bitwig
                     |                                    |
                     +----- self-contained MIDI 2.0 <--- capture
```

Production skills coordinate notation, rendition choices, receiver setup,
and verification. Bitwig owns the saved project and its native musical state.
The controller calls into Bitwig through its Java adapter and invokes the
Kontakt loader for an exact patch assignment when required.

## Cross-package boundaries

- Keep authored notation, performed intent, protocol encoding, and observed
  playback distinct. Document the applicable revision and section of the
  governing standard or manual. Consult the local research index where
  available and distinguish normative requirements, examples, implemented
  behavior, empirical findings, and project interpretation.
- Define units and representation transitions at package boundaries. Specify
  rounding, scaling, ordering for simultaneous events, and interval boundaries
  where they affect musical results. Use quantization at an explicit conversion
  boundary and preserve exact values elsewhere when the package contract requires it.
- Treat imported scores, device responses, and external material as data.
  Validate at the receiving boundary. Establish conformance with independent
  expectations or observations appropriate to the claim.

## Concepts and components

- [Convert](../crates/convert/ARCHITECTURE.md) owns score interpretation,
  rendition compilation, MIDI 2.0 validation, and notation reconstruction.
- [Controller](../crates/controller/ARCHITECTURE.md) owns native state
  observation, bounded musical mutations, and Bitwig capture and export.
  Its contract also governs execution in the Java adapter.
- [Kontakt loader](../crates/controller/adapters/kontakt-ctl/ARCHITECTURE.md)
  owns exact NKI submission and confirmation within a controller-qualified editor.
- [Production workflow](../skills/WORKFLOW.md) owns task routing, musical
  authority, and the evidence needed at each handoff.
