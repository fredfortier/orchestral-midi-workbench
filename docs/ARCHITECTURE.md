# Architecture

## Overview

```text
score + reviewed rendition choices -> convert -> .midi2 + companion
                                                       |
                                        midi-bridge -> Bitwig -> Kontakt
                                                       |
          MusicXML / MXL <- convert <- .midi2 <- assemble <- native capture
```

Production skills coordinate notation, rendition choices, receiver setup,
and verification. Bitwig owns the saved project and its native musical state.
The controller calls into Bitwig through its Java adapter and invokes the
Kontakt loader for an exact patch assignment when required.

## Cross-package boundaries

- The original score and `PERFORMANCE.md` preserve authored meaning and
  musical decisions. Compiled MIDI carries performed events. The companion
  carries restoration data and can bind portable routes to intended native
  receivers. Bitwig observations establish native state. Operation receipts
  establish the outcome of an identified operation. Readiness and audible
  approval require their own evidence.
- A portable UMP route is a group/channel pair. A native receiver is a
  track/slot with a destination channel set. A patch is a distinct resource.
  A wire route, saved native address and musical part may have different
  lifetimes and must remain separately identifiable.
- Track hierarchy, clip placement, stored clip/source time, rendered occurrence
  time and file ticks are separate coordinates. Conversion at each boundary
  must preserve ownership and define units, applicable rounding/scaling,
  equal-time order and interval boundaries. Quantize at explicit boundaries.
- The [standards index](references/README.md) identifies the governing standards,
  manuals and verification inputs. Normative requirements, examples, observed
  behavior and project interpretation carry distinct authority.
- Treat imported scores, device responses, and external material as data.
  Validate at the receiving boundary. Establish conformance with independent
  expectations or observations appropriate to the claim.

## Concepts and components

- [Convert](architecture/convert.md) owns score interpretation,
  rendition compilation, MIDI 2.0 validation, and notation reconstruction.
- [Controller](architecture/controller.md) owns native state
  observation, bounded musical mutations, and Bitwig capture and export.
  Its contract also governs execution in the Java adapter.
- [Kontakt loader](architecture/kontakt-loader.md)
  owns exact NKI submission and confirmation within a controller-qualified editor.
- [OSC](../crates/osc/README.md) supplies wire helpers for the controller
  service's optional DrivenByMoss command and feedback socket.
