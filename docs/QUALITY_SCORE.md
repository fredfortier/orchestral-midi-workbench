# Verification coverage

| Area | Evidence and current limit |
| --- | --- |
| Score conversion | The pinned corpus and independent XSD oracle cover finite round trips. See [reference inputs](references/README.md). |
| Bitwig adapter | Rust checks and both Maven profiles establish buildability. A version-matched native cue read establishes live smoke coverage. Other native behavior requires separate evidence. [Verification contract](architecture/controller.md#wire-deployment-and-verification) |
| OSC and composition | The [Bach exercise](../research/QUI-TOLLIS.md) verifies bridge-owned delivery, scoped host effects, score placement, cues and reverse export. It records the still-open cases and durability limits. |
| Kontakt and playback | A load result covers the owned editor transaction. Samples, saved persistence and audible behavior require independent evidence. See the [loader contract](architecture/kontakt-loader.md). |
