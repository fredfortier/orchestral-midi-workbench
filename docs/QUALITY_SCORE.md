# Verification coverage

| Area | Evidence and current limit |
| --- | --- |
| Score conversion | The pinned corpus and independent XSD oracle cover finite round trips. See [reference inputs](references/README.md). |
| Bitwig adapter | Rust checks and both Maven profiles establish buildability. Native behavior needs separate evidence at the admitted host build. See [qualification](../crates/controller/RUNBOOK.md). |
| Kontakt and playback | A load result covers the owned editor transaction. Samples, saved persistence and audible behavior require independent evidence. See the [loader contract](../crates/controller/adapters/kontakt-ctl/ARCHITECTURE.md). |
