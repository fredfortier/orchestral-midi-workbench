# Cargo workspace

- Use [Cargo.toml](Cargo.toml) and the affected package manifest for membership, edition, features and minimum Rust version. Check package overrides and inspect [Cargo.lock](Cargo.lock) when dependencies change.
- Read the [converter contract](convert/ARCHITECTURE.md) or [controller contract](controller/ARCHITECTURE.md) before changing a musical model, protocol, or host operation. For work inside an adapter, use its own instructions and validation path.

## Rust rules

- Use the `rust-skills` rule files relevant to the change. Start with `err-result-over-panic` for failure paths, `num-overflow-explicit` for protocol or timing arithmetic, `serde-try-from-validate` for deserialization, `async-cancel-safety` for session changes, or `unsafe-safety-comment` for unsafe code.
- Return recoverable errors with context for invalid input and host failures. Reserve assertions for proven invariants. Put validation at the boundary and preserve wire compatibility deliberately.
- Keep blocking work off async executor threads and release locks before awaiting. Document the safety argument for each unsafe operation. Measure the affected path before optimizing it.

## Verification

Run from `crates/`. Start with checks for the affected package. Converter tests need the pinned [score corpus](convert/tests/cli.rs) and the [MusicXML XSD and Python lxml](convert/tests/check_musicxml.py). For native Bitwig checks, use the [adapter instructions](controller/adapters/bitwig-ctl/AGENTS.md).

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Run doctests for changes to library examples or public documentation. Check affected feature combinations, supported toolchains and targets when they change. Run package-specific adapter checks for cross-language changes.
