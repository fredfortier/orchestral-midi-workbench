# Verification and CI

GitHub Actions runs formatting, locked workspace tests, and Clippy on Linux.
For a local run, install stable Rust with `rustfmt` and `clippy`, Python 3,
and `lxml` for the Python interpreter selected by `python3` on your PATH.
From the repository root, run:

```sh
python3 scripts/fetch-test-inputs.py
cargo fmt --manifest-path crates/Cargo.toml --all -- --check
cargo test --manifest-path crates/Cargo.toml --workspace --all-targets --locked
cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets --locked -- -D warnings
```

## Verification inputs

The fetch script retrieves the test scores from the pinned [music21
revision](https://github.com/cuthbertLab/music21/tree/7a3b55e13781e2f3d28a5411f17275d259684af4)
and three files from the [MusicXML 4.0 schema](https://github.com/w3c/musicxml/tree/v4.0/schema).
It verifies SHA-256 hashes against the [corpus manifest](../crates/convert/tests/corpus-manifest.json)
and the schema hashes in the script. Downloads go into ignored directories.
Cached files are checked again on subsequent runs.

The optional research audit, `node crates/convert/tests/check_research.mjs`,
requires a separately provisioned research archive with `sources.json`, Node,
`unzip`, and `xmllint`. The public test fetcher provides the inputs for the
Rust workspace and independent XSD oracle.

After changing `rules::VOCABULARY`, regenerate the catalog with
`UPDATE_CONVERT_CATALOG=1 cargo test --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --test conversion catalog_is_single_source`.

## Native verification

Bitwig and Kontakt checks use disposable fixtures outside unattended CI. The
[adapter instructions](../crates/controller/adapters/bitwig-ctl/AGENTS.md#build-checks)
cover Java builds and a version-matched native smoke test.
The [coverage table](QUALITY_SCORE.md) identifies the evidence needed for
native edits, patch loading, and playback.
