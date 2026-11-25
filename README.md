# Orchestral MIDI Workbench

AI companion tools for inspecting, composing, and editing orchestral music in Bitwig Studio, with MusicXML and MIDI 2.0 conversion.

Point to selected notes, automation points, or clips in Bitwig and ask an agent to inspect them or use them as musical reference. For example, use a cello rhythm and the harmony across the string tracks to rewrite just the viola clip. Workbench keeps the material it reads and the destination it edits independent. You can also start from a score, realize it with Kontakt instruments, and export the edited arrangement for analysis or notation.

[Work with an open Bitwig project](#work-with-an-open-bitwig-project) · [Try the standalone converter](#try-the-converter) · [Architecture](docs/ARCHITECTURE.md)

## Work with an open Bitwig project

`midi-bridge` connects to a Java extension in Bitwig Studio. Together they provide document access for stored notes, note expressions, and automation in the Arranger and Clip Launcher. The controller resolves the exact destination, checks it against the content previously read, and verifies the stored result after applying a supported edit in one native Undo step. That lets an agent read broadly and edit narrowly while Bitwig owns the project and its history.

Workbench delegates session operations to DrivenByMoss over OSC wherever a qualified command covers the task, such as play, stop, seek, or clip audition. When a session action changes stored material, the workflow verifies the effect with a document read. [Why the stack uses a document data access layer](docs/ARCHITECTURE.md#why-bitwig-needs-a-document-data-access-layer)

Follow the [Linux adapter setup](crates/controller/adapters/bitwig-ctl/AGENTS.md#update-and-smoke-test-on-linux). With the adapter connected, select notes in Bitwig's Detail Editor Panel and run a read from the repository root.

```sh
./crates/target/debug/midi-bridge controller content read --selection
```

The command returns a JSON snapshot of the owning clips, with flags for the selected notes. For analysis and editing through an agent, open this checkout in Codex or another agent that supports `SKILL.md` and use the [exploration](skills/bitwig-explore/SKILL.md), [score analysis](skills/score-analysis/SKILL.md), or [editing](skills/bitwig-edit/SKILL.md) skill.

## Try the converter

![A violin phrase goes from MusicXML through a validated MIDI 2.0 clip and back to notation.](docs/media/violin-round-trip.gif)

From the repository root, with the stable Rust toolchain installed, run:

```sh
out=$(mktemp -d)
cargo install --locked --path crates/convert --root "$out"
"$out/bin/convert" crates/convert/examples/violin.musicxml --to midi2 -o "$out/violin.midi2"
"$out/bin/convert" "$out/violin.midi2" --validate
"$out/bin/convert" "$out/violin.midi2" --to mxl -o "$out/violin.mxl"
```

The validation report inventories seven notes and the encoded pizzicato and staccato articulations. Open the resulting `violin.mxl` in a notation editor. Keep the generated `.convert.json` companion when you want source-aware restoration of a matching clip. Move it aside to see what the converter reconstructs from the MIDI performance alone. The forward conversion also saves the original source bytes alongside the clip.

## Realize and edit a score in Bitwig

Start with the [production workflow](skills/bitwig-compose/SKILL.md) to choose documented patches, plan articulations and phrasing, and realize a score with editable notes and automation in Bitwig. Export can capture a whole arrangement or a track group, validate a self-contained MIDI 2.0 clip, and reconstruct MusicXML or compressed MXL for harmonic, rhythmic, and orchestration analysis. [Kontakt loader setup](crates/controller/adapters/kontakt-ctl/README.md) · [MIDI 2.0 export](skills/bitwig-export-midi2/SKILL.md)

## How articulations and expression travel

**In the file**, the converter prototypes the MIDI-CI Orchestral Articulation Profile. A supported technique such as pizzicato is stored in a MIDI 2.0 Note On attribute. The clip declares the profile and carries the performed notes and controls. A printed dynamic can inform attack velocity or continuous controls as part of the rendition. Profile declarations describe the clip encoding. Live negotiation and playback depend on the receiving instrument. [Orchestral Articulation Profile](https://midi.org/midi-ci-profile-for-note-on-selection-of-orchestral-articulation) · [MIDI Clip File specification](https://midi.org/midi-clip-file-specification-smf-midi-2-0)

**In Bitwig**, articulated parts use channels 2–16 for musical and keyswitch notes. The note channel selects the Kontakt instrument or articulation slot. Channel 1 carries a single shared automation lane per control. Each slot's Channel Filter admits its own note channels and the shared channel 1. Its Channel Map then sends those messages to channel 1 at Kontakt. That lets one dynamic curve follow the phrase as it moves between patches. For example, when a patch supports it, CC1 crossfades sampled dynamic layers while CC11 shapes phrase level. [Bitwig MIDI devices](https://www.bitwig.com/userguide/latest/midi/) · [Routing and expression conventions](skills/bitwig-edit/references/dynamics-and-expression.md#apply-the-distinction)

**On export**, the bridge reads that routing and translates recognized articulation slots into MIDI 2.0 note attributes. An ambiguous slot name becomes a performance remark. Reconstructing a score from edited DAW timing involves notation choices such as quantization and voice assignment. Keep the source score when you need the original engraving. [Capture and reconstruction](docs/architecture/controller.md#self-contained-midi2-export)

## Project status

The converter runs independently of Bitwig or Kontakt. Document access uses private mappings qualified against the supported Bitwig Studio 6.1 host build on Linux. Score realization with Kontakt uses a provisioned Kontakt 7 setup under Wine. See the [verification guide](docs/CICD.md) for reproducible converter checks and the scope of native testing.

[Contributing](CONTRIBUTING.md) · [Architecture](docs/ARCHITECTURE.md) · [MIT License](LICENSE)
