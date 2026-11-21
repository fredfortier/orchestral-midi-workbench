# Verification and CI

This checkout uses local checks. Bitwig and Kontakt qualification use disposable
fixtures outside unattended checks. See the
[workspace checks](../crates/AGENTS.md#verification) and [adapter checks](../crates/controller/adapters/bitwig-ctl/AGENTS.md#build-checks).

## Verification inputs

Conversion checks use the pinned MusicXML 4.0 XSD in
`.research/standards/musicxml/musicxml-4.0/schema/` and the pinned `music21`
corpus in `music21/music21/corpus/`. The
[corpus manifest](../crates/convert/tests/corpus-manifest.json) records the
revision and sample. The research archive's `sources.json` records source
revisions and hashes for MIDI, MusicXML, SMuFL and instrument manuals.
Original documents retain their own licenses.

From this repository root, to recover the inputs from a local copy of the
retired checkout:

```sh
git -C ../MIDI2.0Workbench archive --prefix=.research/ \
  9f549d9d69453de4501744913d342a95c3c516de:crates/research | tar -x
git clone ../MIDI2.0Workbench/music21 music21
git -C music21 checkout --detach 7a3b55e13781e2f3d28a5411f17275d259684af4
node crates/convert/tests/check_research.mjs
cargo test --manifest-path crates/Cargo.toml --workspace --all-targets --locked
```

Run the extraction and clone only for missing inputs. The corpus also comes
from the [pinned upstream revision](https://github.com/cuthbertLab/music21/tree/7a3b55e13781e2f3d28a5411f17275d259684af4).
The Node check needs `unzip` and `xmllint`. Rust's independent XSD oracles use
Python with `lxml`.

After changing `rules::VOCABULARY`, regenerate the JSON catalog with
`UPDATE_CONVERT_CATALOG=1 cargo test --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --test conversion catalog_is_single_source`.
