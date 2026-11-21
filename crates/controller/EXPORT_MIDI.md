# MIDI2 export contract

Binding continuation of [ARCHITECTURE.md](ARCHITECTURE.md).

## Self-contained MIDI2 export

Export is a read-only, two-step operation: `controller midi capture` reads the
current project (or an exact `--path` subtree) into canonical JSON;
`controller midi assemble` interprets that JSON offline and writes a MIDI Clip
File. Capture schema 2 retains Entity and complete Snapshot records, with global
musical context. Automation is stored once in `midi.automation`,
alongside clips and explicit owner/scope paths. Schema 1 captures require a new
capture; there is no parallel nested-automation decoder.
It excludes inactive tracks and descendants of inactive groups before reading
clips unless `--include-inactive` is explicit. Mute/solo are not activity.
Included owners must be read completely; an excluded owner cannot break capture.
Capture runs on the document thread against one current document. It does not
stop transport, change selection, or edit the project.

### Musical interpretation, not a project backup

The capture is retained unchanged. An agent may transform a working copy before
assembly, using the same canonical note and automation records. That copy is
editorial input, never authority for native replacement. `decisions` records
concise musical choices and losses; it is not an executable transformation DSL.
Prefer standard encodings. Unknown plugin automation may be interpreted as a
conventional MIDI control when its actual assignment, units, polarity and musical
function support the mapping. An approximation is permitted when useful and
labelled; a matching knob name alone does not prove equivalence. Do not retain
both an interpreted curve and its redundant DAW implementation.

Controller owns host-to-musical interpretation and assembly. Convert owns wire
validation and musical-to-notation interpretation. The output is self-contained:
intermediate JSON and reports are not required companions. Preserve compact
musical residuals in declared in-file text conventions only when standard MIDI
cannot express them. Do not embed native IDs, object graphs, plugin state or
routing devices already explained by the following convention.

### Channels emulate orchestral articulation profiles

Displayed MIDI channel 1 (JSON 0) carries shared controls. Articulated tracks use
channels 2–16 for notes; a single-channel part may use 1 for notes and controls.
Instrument Layer slots use All → Same, then stock Channel Filter → Channel Map:
the Filter admits shared channel 1 and the slot's note channels; the Map sends
all messages to channel 1 **before** the instrument. These channels are receiver
addresses, not universal orchestral subclass numbers. Export observes the actual
Filter/Map settings and resolves the slot name through the converter's canonical
articulation vocabulary. It encodes supported meaning in Note On attributes and
uses a musical track route rather than one score part per slot. An ambiguous
name retains a remark; it does not silently become a standard articulation.
File profile declarations do not claim negotiation or plugin support.[^export-profile]

Lyric phrase slots use `[lyrics lang=la reset=20] et in ter-ra pax`, where the
language and reset key are optional, spaces separate words and hyphens separate
syllables. The prefix identifies lyric meaning; `reset` explicitly declares a
command note on that slot's admitted note channels. No Syllabuilder inspection
or pitch-range heuristic is used. With no per-syllable alignment, phrase text is
a timed performance remark, not a fabricated standard lyric sequence.[^export-lyrics]
Command notes are excluded from engraved pitches and retained as declared
playback commands. Unmarked slots are not guessed to contain lyrics.

### Assembly boundaries

Expand actual clip source/loop occurrences into arranger time; deduplicate repeated
track automation, not distinct tracks or distinct placements. Preserve stable
same-time event dependencies. Use 61,440 ticks per quarter for captured times,
rounding once and reporting maximum error; retain positive short notes for at
least one tick. Native note probability/repetition and unsupported expression
remain explicit musical residuals, not claimed deterministic playback.
Continuous controls are sampled with at most 1/1024 normalized value error;
HOLD boundaries retain their order. Tempo approximation must be checked against
the captured beat/seconds mapping. Unrepresentable behavior is reported rather
than confused with malformed input. Export need not reproduce the original audio.

Use the latest published `midi2` crate, currently 0.11.1. Reuse convert's clip and
wire helpers instead of another encoder; narrow checked internal types cover
specific standards gaps. All semantic data needed for interpretation travels in
the file, not MIDI-CI Property Exchange, which is excluded from Clip Files.[^export-clip]
Validate assembled bytes before atomic no-clobber publication.

[^export-profile]: M2-123 v1.0 §§5–7 defines articulation coordinates, not host
    channel assignments; M2-101 [MIDI-CI](https://midi.org/midi-ci-specification)
    describes bidirectional capability negotiation. This channel convention is
    project policy for receivers without that implementation.
[^export-lyrics]: M2-104 v1.1.2 §§7.5.9–11 defines performance text and syllable-timed
    lyrics/melisma; see the [UMP specification](https://midi.org/universal-midi-packet-ump-and-midi-2-0-protocol-specification).
[^export-clip]: M2-116 v1.0 §6.4 prohibits Property Exchange resources in a
    [MIDI Clip File](https://midi.org/midi-clip-file-specification-smf-midi-2-0).

### External model and interpretation precedents

Use DawProject's musical grammar as the model comparison, not as a new capture
format: [Clip](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Clip.java)
separates placement and content/loop coordinates;
[Note](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Note.java)
contains velocities and expression timelines; [Points](https://github.com/bitwig/dawproject/blob/ee4dcdde75940f30e14e55401a26955a58b8322b/src/main/java/com/bitwig/dawproject/timeline/Points.java)
associates values with a target and unit. Canonical controller records retain
these roles without a second model. The converter's
[precedent comparison](../convert/ARCHITECTURE.md#implementation-precedents-and-deliberate-differences)
records reconstruction choices and necessary differences.

Planetbiru's [DAWProject-to-MIDI parameter conversion](https://github.com/Planetbiru/MusicXML/blob/ea95586b86630e4f3ed7396fb2ed081d4de25eb0/src/DAWProject/DAWProjectToMIDI.php#L208)
illustrates normalized controls, bipolar pan and thresholded CC64. The
[official CC table](https://midi.org/midi-1-0-control-change-messages) establishes
controller names, while the supplied [practical cheatsheet](https://producersociety.com/midi-cc-cheatsheet/)
is a useful lookup. Neither proves plugin assignments. In particular, CC1 is
Modulation, CC11 is Expression and CC64 is Hold by standard; orchestral crossfade
and syllable holding are documented receiver conventions. Never apply a threshold
or rename a lane solely because the plugin uses a familiar word.

The Rust [dawproject-rs API](https://github.com/leichak/dawproject-rs/tree/35fe2a9c26235f3214f755b8597756aa26cc845c/src/api)
corroborates these boundaries: Track/Channel containment, Arrangement lanes plus
global tempo/signature points, and Note-local expression. Capture's `entities`,
`midi` and `globals` retain these three roles using existing canonical records;
mandatory observed values stay mandatory rather than copying an interchange
schema's optional fields. This is topology inspiration, not a DawProject importer.

Bound expanded work before publication: at most 100,000 clip loop segments,
100,000 sampled points per control, one million expanded musical events and
100,000 global timing samples. Native context is sampled at 1/16 quarter-note
beats plus exact stored meter changes; exported tempo uses interval-average
quarter duration derived from native beat/seconds. Check endpoint timing against
the decoded file; interior tempo remains an approximation, not an exact recovered
tempo-automation law. These limits reject the affected export rather than truncate.
Native gain `x` is cube-root amplitude. When interpreted under M2-113 §3's squared
Volume response, both CC7 and registered per-note Volume use `x^(3/2)`; do not
confuse a control value with linear amplitude. A different receiver response is
an explicit musical interpretation, not an assumed plugin capability.

Nonshared channel CC curves with actual points must be interpreted before
collapsing articulation channels. Assembly refuses them rather than silently
broadcasting receiver-specific state across the musical track. Empty legacy
lanes carry no curve. Resolve such a curve explicitly in the editorial copy as
shared control, independent routing, or a named musical residual.

Capture-to-score audit: clip placement/loop ends bound sounding note tails; report
clipping beyond one file tick rather than silently implying full stored durations
survive export. A stored fermata-length tail beyond the arranger clip end is not
permission to extend the arrangement. Compare current facts separately from older
performance notes. `examples/music21_analysis.py` independently checks imported
MXL attacks/pitches/endpoints against the editorial capture and optionally an
archived source score; its chord-slice labels are analysis, not recovered intent.
