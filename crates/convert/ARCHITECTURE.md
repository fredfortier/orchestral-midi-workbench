# Conversion architecture

Convert separates musical meaning, MIDI encoding and reconstructed notation.
Controller owns DAW capture and offline host-to-MIDI2 assembly; the
[controller export contract](../controller/EXPORT_MIDI.md#self-contained-midi2-export)
owns active-track filtering, the channel/Filter/Map convention, lyric slot names
and interpretation of captured automation. Do not duplicate that host model here.

## Musical representation

Preserve authored distinctions and provenance where the conversion contract
requires them. Make ambiguous, unsupported and lossy mappings explicit. A
receiver-specific keyswitch or an editorial inference needs an explicit binding
or interpretation, respectively. An instrument's available techniques alone
provide no evidence of MIDI-CI support.

Keep score time, performed beats, ticks, pitch units and protocol resolution
explicit. Define scaling, rounding and equal-time ordering at encoding
boundaries. Account for tempo and repeat occurrences when mapping score time
to performed time. Keep validation and reconstruction outside real-time host
callbacks.

## Self-contained MIDI2

Use `midi2` 0.11.1 (latest published version verified during this change), with
small checked adapters only where current standards exceed upstream support.
Reuse the existing Performance, clip framing, orchestral dictionary and wire
helpers. An embedded dictionary must suffice without an external duplicate.
An explicitly supplied older companion remains usable for source restoration;
new export and reconstruction require no companion and do not discover one
implicitly for an exported file.

Standard Note On attributes, per-note controls, CC, tempo/meter/key and Flex Data
text/lyrics take precedence over private conventions. A namespaced Unknown
Metadata Text record may identify musical parts and compact unresolved musical
facts. Timed Unknown Performance Text carries remarks. These are project
conventions, not new registered MIDI profiles. Once routing has been interpreted,
do not serialize native IDs or a duplicate of the routing graph. Property
Exchange is not an alternative in a MIDI Clip File.[^clip]

## Validation is a usable analysis

`convert FILE --validate --report FILE` checks the same decoder used by conversion.
Report wire validity separately from score admissibility: framing, packet lengths,
text reassembly, clocks, balanced notes, profiles, dictionaries and recognised
metadata; then parts, routes, counts/ranges, tempo/meter, control types, expression,
lyrics/remarks and unresolved musical meaning. Malformed input exits 1; valid
but inadmissible notation input exits 2; admissible input exits 0. Diagnostics
identify an event or owner, not just a boolean. Valid encoding does not prove
receiver support, original notation, or audible equivalence.

## Notation reconstruction

For exported files, group notes by embedded musical track identity, including
auxiliary routes for independent overlapping note instances. Do not turn
articulation channels into extra score parts. Construct measures, rests, chord
members, deterministic voices and ties across barlines. Keep tempo/meter changes.
Infer binary/triplet rhythms only within 1/96 quarter-note beat by default;
record approximations. A tiny note is not automatically a keyswitch or grace.
Missing key context uses deterministic editorial spelling, not claimed source
spelling. The measured writer currently retains key messages as playback metadata;
it does not engrave key signatures or perform key-aware enharmonic spelling.
This is an explicit reconstruction limit even when the MIDI2 contains a key.
Missing meter/unaligned changes require an explicit reported policy.

Standard articulations become their MusicXML equivalents. A legato articulation
does not establish slur endpoints. CC1 does not universally mean orchestral
crossfade; CC11 does not uniquely identify a printed dynamic. Retain numeric
control meaning and use explicit musical interpretation when applicable.

Standard lyric messages become MusicXML lyrics with word/melisma boundaries.
Unaligned lyric phrases remain visible `direction/words`, preserving text without
inventing syllable alignment. MusicXML's `other-play` carries playback details
without a more specific standard representation; it is not a place to pretend
unknown playback has been realised.[^xml]

No new mapping sidecar is required. Add an opt-in mapping input only after a
concrete, documented ambiguity cannot be resolved from the standards, embedded
musical metadata or declared conventions. Reports and intermediate captures are
not reconstruction dependencies.

## Verification and economy

Use existing tests and independent MusicXML XSD checks. Add durable tests only
for plausible regressions; every added test docstring must name the scenario,
likely failure mechanism and observable protection. Use ephemeral native probes
for controller integration. Retain a piece-specific case study outside production
logic; distinguish capture, file validation, readable notation and audio evidence.

[^clip]: M2-104 v1.1.2 §§7.5.9–11 defines text and lyrics; M2-116 v1.0 §6.4
    excludes Property Exchange resources. See [UMP](https://midi.org/universal-midi-packet-ump-and-midi-2-0-protocol-specification)
    and [Clip Files](https://midi.org/midi-clip-file-specification-smf-midi-2-0).
[^xml]: MusicXML 4.0 distinguishes [notes and playback](https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/note/),
    [text directions](https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/words/)
    and [other playback](https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/other-play/).

## Implementation precedents and deliberate differences

Standards govern encodings. Where they leave reconstruction policy open, the
following inspected implementations supply mechanisms, not new requirements.

| Decision | Inspected example and adaptation |
| --- | --- |
| Musical entity grammar | DawProject revision `ee4dcdde`: [Note](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Note.java) holds time/duration/key/channel/normalized velocities and child expression; [Points](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Points.java) holds a target, typed points and units. Existing controller records already express these facts. Reuse them rather than introducing another object grammar or claiming DawProject serialization compatibility. |
| Placement versus source | DawProject [Clip](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Clip.java) separates time/duration from playStart/playStop and loopStart/loopEnd. Canonical sourceStart/sourceEnd and loopLength retain those roles, using explicit quarter-note beats. |
| Reconstruction order | music21 revision `7a3b55e1`, [MIDI translation](https://github.com/cuthbertLab/music21/blob/7a3b55e13781e2f3d28a5411f17275d259684af4/music21/midi/translate.py#L2225): quantize, insert conductor/meter, make measures, voices, ties and rests. Use this sequencing with the existing Rust note payload writer; do not add a Python runtime or convert through MIDI 1.0. |
| Rhythm defaults | music21 [Stream.quantize](https://github.com/cuthbertLab/music21/blob/7a3b55e13781e2f3d28a5411f17275d259684af4/music21/stream/base.py#L9329) compares rational subdivisions and records signed errors. MuseScore's [quantizer](https://github.com/musescore/MuseScore/blob/main/src/importexport/midi/internal/midiimport/importmidi_quant.cpp) uses rational grids and avoids shortening beneath observed note lengths. Our bounded binary/triplet snapping is deliberately more conservative than music21's default sixteenth/triplet grid. Retain positive exceptional durations rather than treating them as noise. |
| Small conversion pipeline | Planetbiru revision `ea95586b`, [MusicConverter](https://github.com/Planetbiru/MusicXML/blob/ea95586b86630e4f3ed7396fb2ed081d4de25eb0/src/MusicXML/MusicConverter.php#L235) delegates MIDI parsing and score writing. Adopt that direct orchestration, not its rendering backend or a competing MIDI-1 model. Its [DAWProjectToMIDI](https://github.com/Planetbiru/MusicXML/blob/ea95586b86630e4f3ed7396fb2ed081d4de25eb0/src/DAWProject/DAWProjectToMIDI.php) demonstrates parameter-to-CC unit conversion; use full MIDI2 resolution and explicit receiver-specific exceptions. |

These are mechanism-level adaptations, not copied implementations. MIDI2
attributes, embedded metadata and preservation of non-default receiver meanings
are necessary differences from MIDI-1 importers. Existing crate input/output and
error conventions remain the integration examples for CLI, framing and publication.

### Canonical Rust implementations

Read the pinned crate source before replacing a supported operation. `midi2`
0.11.1's `channel_voice2/key_pressure.rs` and
`registered_per_note_controller.rs` demonstrate typed setters; `Controller`
provides Volume, Pan and Pitch7_25. Reuse those layouts. The narrow text reader
repairs a demonstrated `flex_data/text.rs` defect: `finished()` returns true at
byte 11 of the last packet, excluding the twelfth payload byte. Keep upstream
builders and message validation; do not build another MIDI message framework.
The shared canonical XML serializer uses `musicxml`'s ScorePartwise reader/writer
and `musicxml_internal::XmlElement`/serializer traits, with the existing lexical
adapter for integer-backed Semitones. Both exact and measured writers pass
through that serializer. These are pinned code facts, not speculative API gaps.

The [dawproject-rs API](https://github.com/leichak/dawproject-rs/tree/35fe2a9c26235f3214f755b8597756aa26cc845c/src/api)
was also inspected: `track.rs` contains channels, `arrangement.rs` contains
lanes/points/tempo/signature sequences, and `timeline_mods/note.rs` contains time,
duration, channel, key, normalized velocities and child expression timelines.
Use those conceptual boundaries. Do not import its pervasive optional numeric
fields or fake-data machinery into a validated native capture. This comparison
requires no new runtime dependency or substitute for canonical controller records.

Initial clef follows MuseScore's inspected
[`clefTypeFromAveragePitch`](https://github.com/musescore/MuseScore/blob/main/src/importexport/midi/internal/midiimport/importmidi_clef.cpp#L108):
mean MIDI pitch below 60 selects bass, otherwise treble. This deliberately omits
adaptive clef changes and keyboard staff inference. Emit divisions/clef initially
and time signatures only initially or when the actual meter changes.
