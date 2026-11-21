# Tech debt

| Area | Remaining work |
| --- | --- |
| Reference inputs | Make the pinned XSD, standards and corpus retrievable without a local retired checkout. [Current recovery](../references/README.md). |
| Native qualification | Establish a maintained retrieval location for the retired native qualification reports. [Provenance](../adrs/controller-native-authority.md). |
| Bitwig deployment | Smoke-test the production wire 71 artifact and round-trip discovered CLAP and VST paths through `controller info --path` in a disposable Bitwig 6.1 project. [Deployment procedure](../../crates/controller/adapters/bitwig-ctl/AGENTS.md#update-and-smoke-test-on-linux) |
