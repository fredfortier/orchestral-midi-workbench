# Orchestral MIDI Workbench

AI companion tools for composing and editing orchestral music directly in an open Bitwig Studio project.

> “Read the cello rhythm and the harmony across the strings. Use them to
> develop the selected viola clip. Change only that clip.”

With playback stopped, you can inspect just the notes or automation points you
selected, or read across the whole project for musical context. When you ask
for an edit, you choose exactly which material changes. The result stays
editable in Bitwig and you can undo it in one step.

Workbench also brings MusicXML scores into this workflow. It carries
articulations such as staccato and pizzicato in MIDI 2.0, plays them through
sampled instrument patches, and recovers them from edited performances for
export.

[Work with an open Bitwig project](#work-with-an-open-bitwig-project) · [Articulations and channel routing](#orchestral-articulations-and-channel-routing) · [Try the standalone converter](#try-the-converter)

## Orchestral articulations and channel routing

The MIDI Association's **MIDI-CI Profile for Note On Selection of Orchestral
Articulation** (M2-123) defines how a MIDI 2.0 Note On carries a playing
technique such as pizzicato. In a plain MusicXML conversion, `convert` sets
the MIDI 2.0 Note On **Attribute Type and Attribute Data** for the recognized
articulations. [MIDI Association profile](https://midi.org/midi-ci-profile-for-note-on-selection-of-orchestral-articulation)

A virtual instrument supplies the sound. The Kontakt edition of **Orchestral
Tools Berlin Strings** provides single-articulation patches as Kontakt
Instrument (`.nki`) files. Workbench can load the part's pizzicato patch
into the Kontakt sampler plug-in to realize the profile's pizzicato
articulation with that library's samples.
[Berlin Strings for Kontakt](https://www.native-instruments.com/products/berlin-strings) · [Orchestral Tools articulation patches](https://orchestraltools.helpscoutdocs.com/article/22-single-vs-multi-artic-patches) · [Kontakt file formats](https://docs.native-instruments.com/ni-tech-manuals/kontakt-manual/en/file-formats)

The Bitwig and Kontakt workflow does not have an end-to-end path for
selecting patches from M2-123 Note On attributes. To carry the articulation
through Bitwig's editable note clips and play the intended samples, Workbench
uses this **channel convention on each articulated track**. Channel numbers
below are the numbers shown in Bitwig.

| Where | Convention |
| --- | --- |
| **Note clip, channels 2–16** | Assign each articulation a specific channel on the track. For example, channel **2** can select sustains, **3** staccato, and **4** pizzicato. The assignments are per track. Musical notes and any patch-specific keyswitch notes use the assigned channel. |
| **Track automation, channel 1** | Reserve channel **1** for shared MIDI Control Change (CC) automation. One curve per control can follow the phrase across articulation changes. For patches that support them, CC1 can crossfade sampled dynamic layers and CC11 can shape phrase level. |
| **Kontakt input, channel 1** | Each receiving layer or slot has a **Channel Filter** that admits its assigned note channels and shared channel **1**. A **Channel Map** then remaps every admitted message to **1** before it reaches the Kontakt instance that hosts the articulation patch. |

[Bitwig Channel Filter and Channel Map](https://www.bitwig.com/userguide/latest/midi/) · [Workbench receiver routing](docs/architecture/controller.md#integrated-instruments) · [Score realization](skills/score-realization/SKILL.md#orchestrated-realization) · [Dynamics and expression](skills/bitwig-edit/references/dynamics-and-expression.md#apply-the-distinction)

A **rendition plan** is a JSON file that assigns notes to MIDI 2.0 routes and binds each route to a Bitwig track, note channel, and instrument patch. Pass it to `convert` with `--rendition-plan` to opt into patch-bound routing. The same Note On then carries both the articulation attribute and the group and channel assigned by the plan. The companion records the Bitwig binding. In the violin example, a pizzicato note carries the pizzicato attribute and lands on Bitwig channel **2**, where the Shorts slot receives it. [Example rendition plan](crates/convert/examples/violin.rendition.json) · [Rendition inputs](skills/score-realization/references/conversion.md#rendition-inputs)

On export, `midi-bridge` reads the slot routing and encodes recognized
articulations back into MIDI 2.0 Note On attributes on the musical part. It
records an ambiguous slot name as a performance remark. Once Bitwig can
preserve and deliver these attributes to an instrument that supports the
profile, that playback path can select articulations directly and dispense
with Workbench's channel convention. [Capture and MIDI 2.0 export](docs/architecture/controller.md#self-contained-midi2-export)

## Work with an open Bitwig project

You can ask what the selected notes are doing harmonically and leave the project
as it is. You can also use a choir volume swell as a model and apply the new
curve to a specific strings automation lane. What you inspect, what you use as
musical reference, and what you change are independent choices.

`midi-bridge` and the Bitwig Java extension read stored note events, note
expressions, and automation directly from the open project. The edit stays tied
to the clip or lane you previewed, even if you click elsewhere. Workbench checks
whether a looped or shared clip source would carry the change to other
occurrences and whether the destination content has changed since the read.
After applying a supported edit, it reads the same scope again to verify the
result. [Selection, snapshots, and verified edits](docs/ARCHITECTURE.md#reading-and-composing-in-bitwig)

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

Printed dynamics inform attack velocity or continuous controls as part of
the rendition. Reconstructing notation from edited performance timing
requires choices such as quantization and voice assignment. Keep the source
score when you need the original engraving. [Rendition and reconstruction](docs/architecture/convert.md)

## Project status

The converter runs independently of Bitwig or Kontakt. Document access uses private mappings qualified against the supported Bitwig Studio 6.1 host build on Linux. Score realization with Kontakt uses a provisioned Kontakt 7 setup under Wine. See the [verification guide](docs/CICD.md) for reproducible converter checks and the scope of native testing.

[Contributing](CONTRIBUTING.md) · [Architecture](docs/ARCHITECTURE.md) · [MIT License](LICENSE)
