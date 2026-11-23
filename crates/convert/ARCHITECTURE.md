# Conversion architecture

Convert owns the transformation between authored notation, performed values,
portable MIDI 2.0 and reconstructed notation. Controller owns native capture
and the [interpretation of Bitwig routing](../controller/ARCHITECTURE.md#self-contained-midi2-export).
Project-specific musical choices and evidence belong to the score's production
workspace.

```text
MusicXML / MXL -> per-part score cursor + performed occurrences
               -> rational Performance + independent sound facets
                  -> optional Rendition Plan: exact note-to-receiver adaptation
               -> SMF2CLIP + embedded refinement dictionary
                  + optional companion/source for exact restoration or delivery
                                                           |
Bitwig capture -> controller assembly -> self-contained .midi2
                                                           |
                                        validate -> reconstruct MusicXML / MXL
```

## Performance grammar

```text
Route       = {group: 0..15, channel: 0..15}
Performance = {duration, notes[], events[], configuration[], piano_routes, report}
Note        = {source, route, key, onset, end, pitch, velocity, release_velocity,
               on_order, off_order, articulation, descriptor?}
Descriptor  = {technique, qualifiers: {semanticId -> JSON value}}
Event       = {time, order, words: UMP u32[]}
```

- [`model.rs`](src/model.rs) represents time and sounding pitch as exact
  rationals in quarter-note beats and MIDI semitones. Duration includes trailing
  rests. Encoded key, sounding pitch, displayed spelling and instrument
  trigger are distinct.
- [`xml.rs`](src/xml.rs) owns score parsing, written cursors and finite performed
  occurrence identity. Source paths identify absolute same-name sibling indices
  (1-based), optional attributes and repeat occurrences. They are identifiers.
- [`rules.rs`](src/rules.rs) owns articulation choices.
  [`extension.rs`](src/extension.rs) owns named descriptors and dictionary rules.
  [`clip.rs`](src/clip.rs) and [`wire.rs`](src/wire.rs) own allocation, clock,
  framing and message encoding. [`notation.rs`](src/notation.rs) owns measured
  reconstruction and [`preserve.rs`](src/preserve.rs) owns exact source restoration.
- Preserve lifecycle, production/attack, connection, timbre, pattern, dynamics,
  position/mute and release as independent facets with source provenance. Every
  explicit audible facet needs a standard encoding, exact binding or timed
  realization. Report any accepted approximation or unsupported requirement.
  A valid file establishes wire validity. Fidelity and audible receiver behavior
  require separate evidence.

## Score semantics and timing

- Admit the documented MusicXML partwise, timewise and MXL forms. Validate
  output against the pinned MusicXML 4.0 grammar and follow the container root.
  Retain the source XML in the companion and original MXL bytes as a separate
  source artifact. Ambiguous roots require selection. Local parsing and XSD
  checks avoid network and external-entity resolution.
- Apply divisions, duration, backup and forward at the written cursor. Chords
  share an onset with independent duration. Cue notes are silent and hidden
  engraving alone leaves playback intact. A contiguous sounding tie joins one
  equal-pitch voice/instrument lifecycle, retaining first attack, final release
  and intervening controllers. Graphical `tied` is a separate fact.
- Repeats, endings and sound navigation produce finite occurrence paths. A plain
  backward repeat defaults to two passes. Preserve performance state across
  jumps and apply the destination's written instructions, divisions and
  transposition. Diagnose unresolved/cyclic navigation and preserve note/pedal
  lifetimes explicitly.
- Resolve sounding pitch from spelling, alter and chromatic/octave transposition.
  Keep harmonic sounding/base/touching pitches distinct. Unpitched instruments
  need an explicit numeric trigger map. Apply supported attack/release offsets
  in active divisions. Reject negative onset and nonpositive duration.
- Numeric sound tempo takes precedence over metronome interpretation. Default
  is 120 quarter BPM. Flex tempo is `round(6e9 / BPM)` in 10 ns per quarter.
  Preserve meter, key and text where representable. Carry residual meter or
  spelling without changing durations to fit the wire.
- Keep numeric dynamics, attack velocity, sampled dynamics and expression
  separate. MusicXML percentage dynamics use forte 90 and M2-115 scaling,
  interpolating between adjacent encoded values for fractional input. Explicit
  standard-width values retain all bits. Report project interpretations.
- Orchestral nominal printed levels use `pppppp..ffffff` values
  `4,8,12,20,32,45,58,72,90,104,116,120,124,127`, with mf default.
  Preserve `sf`, `sfz`, `fp`, `rfz` and niente as envelope/attack intent.
  Numeric note/sound values override labels. The ordinary accent policy adds
  12 nominal units up to 127 when no sampled accent is specified.
- Preserve phrase-wide wedges as curves. Apply authored grace allocation,
  tremolo counts/intervals, ornament rates, and arpeggiation when determined.
  Default wedge interpolation uses explicit endpoints or one adjacent nominal
  level when the endpoint is absent. Niente ends at zero. Sample at endpoints
  and 1/96-quarter intervals, retaining the definition of the curve.
  With no grace allocation, allow 1/8 of the following principal duration for
  the group. The default arpeggio step is 1/32 quarter, with total spread capped
  at 1/4 of the shortest chord duration. Missing timing policy for a fermata,
  breath, caesura or swing yields `E_TIMING_POLICY`. A sampled gesture must
  satisfy the full intended timing and replace the equivalent generated attacks.
- Piano damper/sostenuto/soft pedal uses CC64/66/67. At a pedal change release
  before pressing again. Keep pitch-pedals and percussion damping distinct.
  Resolve note/score instrument identities before mapping one-based MusicXML
  program, bank and channel numbers to wire values. Require an explicit
  disposition for unsupported playback-bearing data.

## Articulation grammar

- Typed MusicXML and the finite `other-technical`/`other-articulation` vocabulary
  can supply the same meaning. Normalize text by NFC, whitespace and lowercase,
  then match exactly. `direction/words` techniques persist in their part/staff/voice
  scope. Numbered `other-notation` spans apply on [start, stop) by part, staff,
  voice, number and term. `note/play` is per-note. `sound/play` persists by
  instrument. Note pizzicato overrides persistent pizzicato.
- Merge compatible facets and absorb only the marks a compound represents.
  For dot compounds, prefer tenuto+dot, accent+dot, slurred dot, then plain dot.
  A refinement such as snap pizzicato keeps the generic production and supplies
  the narrower technique. Preserve remaining timing, pitch and sound constraints.
- Prefer a complete standard coordinate plus modifiers/controllers, then an
  exact full-descriptor variation in declared preference order, then a custom
  binding or exact timed realization. Equal-priority ambiguity or an unsupported
  required facet fails. Report `E_CONFLICT` or `E_UNREPRESENTABLE` with the source.
- The [catalog](mapping/orchestral-subclasses.json) covers all 106 named
  coordinates and is checked against `rules::VOCABULARY`. T accepts the exact
  named term/semantic ID. P additionally requires pattern/phrase constraints.
  Published Table 12 governs Lontano `14/7` and Singing `14/8`.
  Without/with vibrato is `10/8`/`10/9`. Reserved coordinates fail.
  Custom AT `1A..1F` uses 96 coordinates with 16 variations each per
  receiver/program context. A variation's meaning comes from its binding.
- Exact aliases and their resets are in the interpretation rules. `arco` clears
  pizzicato only. `ordinario` clears persistent production/color/connection
  and restores normal position while preserving mute, dynamics, program and
  tuning. Require explicit support for distinctive sample character, technique
  combinations or ordered multi-message recipes.

| Rule | Input → intent / candidate |
| --- | --- |
| N0 | No non-default production specified → `10/0`; natural percussion decay is not a short articulation. |
| N1 | Pitched orchestral note under a slur, without a short/tenuto compound → slurred connection, `10/2` default candidate. **Policy:** interpret such slurs as legato intent; keep independent timbre and phrase boundaries. Chords need polyphonic binding or separate voices; ties do not create new attacks. |
| N2 | Glissando/slide span → connected pitch gesture, candidate `10/4`; retain both endpoints and any specified trajectory. It does not imply a prerecorded run. |
| N3 | Strong-accent without a short compound → `10/6`. **Policy:** sustained marcato; ordinary accent is independent attack emphasis. |
| N4 | Plain staccato → `11/0`. **Policy:** off-string default on bowed strings, ordinary staccato elsewhere. Explicit on-string refines to `11/1`. |
| N5 | Bowed-string staccato under a slur without tenuto/accent → `11/2`. **Policy:** slurred staccato. Retain the span. Other families retain their own connection intent. |
| N6 | Staccato + accent/strong-accent → `11/3`, absorbing both component marks once. |
| N7 | Staccatissimo → `11/4`. |
| N8 | Spiccato → `11/5`, retaining its MusicXML stroke-symbol provenance. |
| N9 | Detached-legato OR tenuto+staccato OR combined glyph → `11/9`. **Policy:** portato, regardless of serialization of equivalent marks. |
| N10 | Effective pizzicato=yes → `11/A`, unless refined by snap. |
| N11 | Snap-pizzicato/glyph → `11/B`. |
| N12 | Explicit unmeasured type (normally value 0), or legacy single/0 → `12/0`. Paired start/stop is checked first, including zero marks. Measured tremolo retains rate/count and needs a matching sample binding or timed repeated notes. |
| N13 | Double-tongue → retain native production in the note descriptor. Do not trigger two attacks per written note merely from tonguing technique. Explicit two-repeat sound requests use `12/4`. |
| N14 | Triple-tongue → retain native production; explicit three-repeat sound requests use `12/5`, with the same distinction. |
| N15 | Trill mark/span → resolve principal/auxiliary sounding pitches, starting member, terminal turn and timing. Semitone/whole-tone and main/upper-first select `13/0..3`. Explicit accidental marks and trill-step determine the auxiliary. Pinned XSD defaults apply where absent; do not claim the implementation derives a diatonic auxiliary from key signatures. |
| N16 | Named two-pitch sample request spanning 3..12 semitones → `13/4..D`; starting member is additional information. Native paired-note tremolo currently receives a diagnostic. |
| N17 | harmonic/natural → `14/0`, with separately resolved sounding/base/touching pitch. |
| N18 | harmonic/artificial → `14/1`, same pitch obligation. |
| N19 | Falloff → `15/0`; retain any additional specified ending trajectory. |
| N20 | Plop → `15/1`; principal note is the destination. |
| N21 | Doit → `15/2`; do not duplicate it on Note Off. |
| N22 | Scoop → `15/3`; principal note is the destination. |
| N23 | Grace group → leading pitches and times; below/above supplies `15/6`/`15/7` when the complete group fits a bound sample, otherwise emit grace notes. Never both. |
| N24 | Shake ornament → `15/8` candidate with all explicit pitch/rhythm properties retained. |
| N25 | Wedges/dynamic directions → timed dynamic intent. A whole single-note sampled gesture can select `15/9..D` if shape/duration/endpoints fit. A phrase-wide crescendo remains one curve across notes, not a new sampled crescendo per attack. |

### Verified SMuFL identities

All names below exist in the archived 1.4 registry. Placement variants do not
change the production technique. A glyph is matched by canonical name, not by a
font codepoint or appearance.

| Glyph names | Normalization |
| --- | --- |
| `stringsJeteAbove`, `stringsJeteBelow` | short.jete, rule G1. |
| `pluckedSnapPizzicatoAbove`, `pluckedSnapPizzicatoBelow` | short.snap-pizzicato. |
| `pluckedLeftHandPizzicato` | Pizzicato + direction 2. |
| `stringsDownBow`, `stringsDownBowTurned` / `stringsUpBow`, `stringsUpBowTurned` | Direction 1 / 2. |
| `articAccentStaccatoAbove`, `articAccentStaccatoBelow`, `articMarcatoStaccatoAbove`, `articMarcatoStaccatoBelow` | Accented-staccato compound. |
| `articTenutoStaccatoAbove`, `articTenutoStaccatoBelow` | Same combined symbol as detached-legato; declared portato convention. |
| `stringsBowBehindBridge`, `stringsBowBehindBridgeOneString`, `stringsBowBehindBridgeTwoStrings`, `stringsBowBehindBridgeThreeStrings`, `stringsBowBehindBridgeFourStrings` | effect.behind-bridge, rule G2; preserve stated string count. Not the ordinary bridge position endpoint. |
| `stringsHarmonic` | Harmonic kind unspecified; do not infer natural/artificial. |
| `stringsHalfHarmonic`, `stringsOverpressureDownBow`, `stringsOverpressureUpBow`, `stringsBowOnTailpiece`, `stringsBowOnBridge` | Exact named technique plus direction where present; bound/custom descriptor. No exact dedicated profile subclass. |

Other glyphs retain their real identities and need a semantic binding when they
specify an audible effect. Never fabricate a SMuFL name to hide a vocabulary gap.

### Pattern constraints

- Trill `13/0..3` means one or two semitones with even root-first and odd
  upper-first. `13/4..D` represents 3–12 semitones with separate start/end/rate
  constraints. Exact lower, fractional and wider intervals need capable
  bindings or timed pitches. The profile trigger and original score pitches
  retain separate identity.
- Repetition `12/4..8` requests 2–6 strokes. `12/9` has no numeric rate. Retain
  measured subdivisions, finite counts and release-triggered strokes. A slur
  and legato sample selection carry different data. Runs `16/3..F` need root,
  direction, extent and rhythm supplied by an explicit recipe.
- Ordinary chords keep individual attacks. Polyphonic samples and prerecorded
  dynamic gestures require a binding that satisfies their complete constraints.

## Refinements and bindings

MusicXML carries a project-defined JSON descriptor under
`play/other-play@type=urn:midi2workbench:articulation:v2`. This example exercises
the independent XSD and vocabulary checks.

```xml
<note id="v1-blurred">
  <pitch><step>D</step><octave>5</octave></pitch>
  <duration>1</duration>
  <notations><articulations><spiccato/></articulations></notations>
  <play>
    <other-play type="urn:midi2workbench:articulation:v2">{"technique":"short.spiccato","qualifiers":{"urn:midi2workbench:ot-berlin:sample-character":"blurred"},"binding":"example-berlin-strings-violin-i-v1"}</other-play>
  </play>
</note>
```

- The [annotation schema](mapping/annotation.schema.json) defines accepted keys
  and ranges. Reject duplicate keys, unknown vocabulary and contradictions.
  `replace` names precisely the superseded source paths. Unknown qualifiers need
  an explicit binding. Generic metadata cannot stand in for an audible effect.
- Preserve distinctions between per-note position, mute identity/amount,
  sounding pitch, attack velocity, sampled dynamic level, amplitude,
  sample length and mute provenance. Numeric values have their declared units.
  A literal MIDI 2.0 Note On velocity of zero remains a Note On.
- [Binding](mapping/binding.schema.json) and [refinement](mapping/refinement.schema.json)
  schemas describe declared receiver meanings. Bind the complete descriptor to
  a versioned library/player/configuration and numeric keys/channels, range,
  velocity roles, switch/hold/release ordering and timing. Compare the required
  map identity and content before claiming receiver support. The
  [Berlin example](examples/berlin-receiver.example.json) illustrates a declared
  configuration with unverified status. Settings and descriptive names alone
  provide no evidence of loaded state, MIDI-CI or audible support.
- Property Exchange can advertise the custom `X-ArticulationMap` resource when
  a real receiver supports it. For ASCII camelCase PE JSON, serialize URI
  qualifiers as sorted unique `{id, valueJson}` entries. Apply negotiated
  SysEx7 chunk limits. The [ResourceList example](examples/resource-list.example.json)
  is resource data. MIDI Clip Files exclude PE resources.
- The embedded dictionary uses `urn:midi2workbench:refinement:v2:` in Unknown
  Metadata Text with `{version, controller_index, descriptors, parameters?}`.
  Its assigned per-note controller resolves a descriptor ID in the file.
  Keep dictionary identity, routing and note-association rules intact.

### Receiver binding constraints

- Use documented articulations, zones and trigger modes for the exact library
  edition and player. A logical program change is separate from an actual
  VST3 patch selection. Positions in a reordered articulation menu carry no
  stable identity.
- Keep connection style, sampled dynamics, expression, tuning and timing
  independent. Match monophonic legato with independent voices or reject
  interference. An overlap duration belongs to the binding's recipe.
- A sampled run, rolled-key trill, release-triggered multiple stroke or
  true-pedal instrument needs its documented numeric trigger/hold/release
  sequence. Bind duration, tempo and stretch explicitly when they affect
  the outcome. Keep recorded mute and simulated mute distinct where specified.
- Switching and live controller state can reach neighboring held or releasing
  notes. Use independent configured instances or refuse incompatible state.
  For each timed gesture, choose the bound realization or the generated events
  once and preserve the original authored intent.

## Clip and wire grammar

```text
SMF2CLIP | profile declarations* | DCS(0) + DCTPQ
         | configuration* | Start of Clip | timed UMP* | End of Clip
```

- Use big-endian UMP words. Initial broadcast Set Profile On declarations
  precede any DCS. Configuration and sequence use separate clocks. For exact
  score rendering take the LCM of rational time denominators and prefer an
  additional factor of 480 when the resulting TPQ fits u16. Reject unrepresentable
  exact clocks. Bitwig capture applies its [separate quantization policy](../controller/ARCHITECTURE.md#self-contained-midi2-export).
- Allocate a compatible route for the full audible, pedal and release lifetime.
  Same-pitch overlaps with independent per-note controls require independent
  addresses. Retain ending releases, pedal changes, channel state, mute
  type then amount, per-note initialization, attacks and explicit post-attack
  events in dependency order. Preserve authored same-time order. Apply stable
  source/route ordering for independent events.
- Validate UMP lengths, phases, clocks, SysEx and note lifecycles and reject
  trailing data. Split long gaps at the 20-bit Delta Clockstamp maximum.
  Publish final outputs only following successful validation.
- M2-123 ID is `7E 21 01 01 01`. Profile Details target `01` has two packed
  feature bytes then 128 variation counts. Feature bits 0–4 cover mute type,
  mute amount, position, release time and custom discovery. Target `40`
  has 96 custom counts. Counts range from 0 to 16 and
  describe availability with no semantic names. File declarations carry no
  live negotiation result. For a playback claim, verify discovery, enablement,
  optional controls and descriptor binding on the exact receiver.
- M2-126 Piano ID is `7E 20 00 01 01`. Ordinary Piano declares Piano alone and
  articulated Piano can declare both. Piano Attribute 04 and an orchestral
  attribute cannot share one slot. Report explicit conflict. For Piano the
  rendering defaults are mf=79 and printed f=92. Numeric MusicXML dynamics still
  use forte 90. Labels/sound IDs select piano interpretation independently of
  the preserved numeric program.

```text
On.word0 = 0x40000000 | g<<24 | 9<<20 | c<<16 | key<<8 | AT
On.data  = sub<<12 | variation<<8 | direction<<6 | resetRR<<3 | string
On.word1 = velocity16<<16 | On.data
Off.word0 uses status 8 and offAT
Off.data = offSub<<12 | offString
Off.word1 = releaseTime16<<16 | Off.data  (or ordinary release velocity off-profile)
```

- Group/channel, subclass and variation fit four bits. Key fits seven.
  Direction 0 lets the receiver select, 1 means down/right, 2 up/left, 3 is
  reserved. On bits 5–4 and Off bits 11–3 are zero. String 0 is unassigned,
  1–6 conventional and 7 other. Reset round-robin only on explicit intent.
  Balance each attack with the allocated Note Off even after natural decay.
- Orchestral release time is inverted (0 longest, `0xFFFF` shortest), with
  `0x8000` default. It differs from
  ordinary end-dynamics. Use the default receiver ending unless a specific
  ending is requested and avoid double application of release gestures.

### Note Off subclasses

| Off AT/sub | Meaning |
| --- | --- |
| `00/—` | Default ending, data zero. |
| `10/0` | Suppress Note Off sample. |
| `10/1` | Soft ending. |
| `10/2` | Hard ending. |
| `10/3` | Release pitch rise. |
| `10/4` | Release pitch fall. |
| `10/5..F` | Reserved. |
| `01/—` | Manufacturer-specific codec. |

### Controller encoding

- Mute type is registered channel index `0x20/0x22` with
  `slot * 0x08000000`, then amount at `0x20/0x23`. Type selection resets amount
  to max. Standard slots 0–15, reserved 16–25, and bound manufacturer slots
  26–31 retain their M2-123 identities. Simultaneous incompatible channel mute
  states need independent routes.
- Position uses Registered Per-Note Controller 0x0C. Bridge/center = 0,
  normal = `0x80000000`, nut/rim = `0xFFFFFFFF`. Plain ponticello/tasto use
  representative `0x08000000`/`0xF0000000`. Preserve the source region as well.
  Position alone cannot encode behind-bridge or overpressure production.
- Absolute Pitch 7.25 uses registered per-note index 3, address `floor(pitch)`,
  and `round(pitch * 2^25)` fitting u32 for pitch in [0,128). Reinitialize
  reused addresses. This is fixed-point, with M2-115 min/center/max scaling
  reserved for ordinary velocity/CC. Retain semantic pitch independently of
  the receiver's measured resolution.

## Rendition and companion grammar

```text
Companion v1 = source, policy, expected MIDI, dictionary, requirements, note_sources?
Companion v2 = v1 + Rendition v1 (legacy receiver_bindings excluded)
Rendition    = tracks[], instruments[], slots[]?, bindings[], bitwig?
Binding      = UMP Route -> track, destination channel, instrument, slot?
Slot         = id, track, musical name, instrument, inputChannels[], settings?
NoteSource   = source identity/route, source_velocity?, encoded route, onset/end/key
```

- [`preserve.rs`](src/preserve.rs) authenticates the companion against decoded
  semantic values from `expected_midi_base64`. Optional `note_sources` are
  canonical by `(onset, route, key, end)`, with exact ownership through encoder
  route isolation. Consumers verify count and coordinates. Exact restoration
  additionally validates the archived source against the rendering policy.
- [`rendition.rs`](src/rendition.rs) requires complete occupied-route coverage,
  unique portable routes and unique destination track/channel pairs, with
  no unused or unbound references. UMP groups do not imply native groups.
  Slot identity is separate from NKI resource identity. Multiple destinations
  may share a slot and distinct slots may reuse a resource. A binding must
  match the slot's track, instrument and admitted note channel. Every slot needs
  a binding. An omitted slot collection preserves legacy one-binding-per-receiver.
- Native destination channels are zero-based JSON and displayed 1–16. Articulated
  tracks reserve displayed 1 for shared controls. A single-channel part may
  use 1 for both notes and controls. Preserve encoded UMP routes. Converging
  curves require agreement over complete histories. For native Filter/Map
  settings use [the controller contract](../controller/ARCHITECTURE.md#integrated-instruments).
- Cap tracks, instruments, slots and bindings at 256 each and settings at 128
  per owner. IDs use 1–128 ASCII letters/digits/dots/underscores/hyphens. Labels
  are nonempty and NUL-free with at most 256 UTF-8 bytes. Track names and
  articulation labels per track must be unique. Settings require unique control
  IDs, a non-null value and declared units/vocabulary.
- Instruments carry an absolute execution NKI path, player identity/version/format,
  settings and optional sample root. Musical patch identity remains relative to
  `INSTRUMENTS_ROOT_DIR`. Optional Bitwig project/group, decision revision,
  meter tick rate, mix and located player-name evidence describe intent.
  Neither a companion field nor a historical schedule grants native write or
  customization authority.

### Reviewed passages and dynamic curves

- [`adapt::Plan`](src/adapt.rs) carries `rendition`, `rules`, optional
  `initialControllers`, `passages`, exact `noteEdits` and `trackCcCurves`.
  The plan is reviewed conversion input.
  The executable delivery is the MIDI and companion.
- Reviewed passages carry `id, sourcePrefixes, track, instrument, expectedNotes,
  reason`. Present nonempty assertions partition performed notes exactly once
  and agree with the bound destination and positive note count. Reject
  unused, unknown, overlapping or contradictory assertions. Cap passages and
  total prefixes at 4096 each. Omission preserves older plans without review
  evidence.
- Each interpreted note must match exactly one rule by source prefix, optional
  lexical `sourceVoice`, complete articulation and descriptor. A rule chooses
  an already bound encoded route and `keyVelocity`, `fixedAttack` or `crossfade`.
  Every rule on a route shares the
  control law. Crossfade declares a CC, fixed u16 key velocity, source law
  and optional independent expression default. `requireDynamic` requires an
  explicit level or admitted curve. `dynamicOrVelocity16` falls back to
  `attackVelocity16 * 65537` by project policy. Preserve unconsumed facets,
  note onsets and release velocity. Initialize controls at beat zero and write changes
  before attacks. Reject any unplanned route introduced by allocation.
- `channelHold` opts into explicit relative dynamic curves. Their first point
  is zero-offset and offsets strictly increase within the note lifetime.
  At the end of [onset, end) a terminal point is checked without a CC write.
  Overlapping notes must request identical values throughout their shared time.
  Use HOLD events with at most 65,536 points per note and 1,000,000 per conversion.
  Consume the rendered per-note descriptor curve once. Release tails share
  later channel control. Default `reject` and key-velocity rules refuse evolving
  curves.
- Additional beat-zero controller entries are unique bound route/index pairs
  with literal u32 values and index 0–127, distinct from rule-owned axes.
  Reject MIDI 1.0 compound selector/data-entry indexes 0, 6, 32, 38, 98–101.
  Preserve special controller bit layouts. A rule has no authority to guess
  a patch's meaning. Unassociated timed channel events and unsupported
  Piano adaptation fail explicitly. Initial program/name assignment changes
  retain original source provenance.
- Explicit note endings select one performed source ID, onset and key; preserve
  onset, pitch and release velocity. `rendition.controlRoutes` designates each
  track's shared channel-1 CC address separately from note bindings. Curve
  points are increasing, in-range rational beats, starting at zero with literal
  u32 values. Destination CC0–119 uses normal MIDI CC events. Retain the score
  preview mix and performance prose in source when `archiveScorePreview` is
  selected. Unrelated executable channel messages still require a disposition.

## Portable arrangement grammar

- Standard Note On attributes, per-note controls, CC, tempo/meter/key and Flex
  text/lyrics take precedence. `urn:midi2workbench:arrangement:v1:` carries
  `{version:1, parts:[{name, routes[]}], decisions[]}` in Unknown Metadata Text.
  Require one declaration, nonempty parts and unique route ownership.
  Auxiliary routes for independent note instances stay under their musical part.
- Unknown Metadata Text is bank 1/status 0 and timed Unknown Performance Text
  is bank 2/status 0. Keep scoped direction remarks under
  `urn:midi2workbench:direction-remark:v1:` and note remarks in `score-remarks`.
  Declared command notes use the arrangement command convention and stay outside
  engraved pitches. Piano Description is specific to Piano. Other instrument
  identities use `urn:midi2workbench:instrument:v1:`; programs remain independent.
- [`arrangement.rs`](src/arrangement.rs), [`remarks.rs`](src/remarks.rs) and
  [`extension.rs`](src/extension.rs) own the actual serialization. The embedded
  dictionary suffices for new files. An older companion is used only when
  explicitly supplied for compatible restoration.

## Validation and reconstruction

`convert FILE --validate --report FILE` checks the same decoder used by conversion.
Report wire validity separately from score admissibility: framing, packet lengths,
text reassembly, clocks, balanced notes, profiles, dictionaries and recognised
metadata; then parts, routes, counts/ranges, tempo/meter, control types, expression,
lyrics/remarks and unresolved musical meaning. Malformed input exits 1; valid
but inadmissible notation input exits 2; admissible input exits 0. Diagnostics
identify an event or owner, not just a boolean. Valid encoding does not prove
receiver support, original notation, or audible equivalence.

### Notation reconstruction

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

### Verification

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

## Standards and implementation boundaries

- Apply MusicXML 4.0, SMuFL 1.4, M2-123 v1.0, M2-126 v1.0, M2-104 v1.1.2
  and M2-116 v1.0. For CI use M2-101/102/103 and for scaling use M2-115.
  [Source and verification inputs](../../docs/references/README.md) records the
  pinned corpus and external evidence.
- Reuse the pinned `midi2` encoder/validators and the shared `musicxml` ScorePartwise
  parser/writer. Keep narrow local adapters for unsupported wire cases, the pinned
  Flex text reader's last-byte defect and fractional Semitones serialization.
  Use the same canonical XML serializer for exact and measured reconstruction.
- Use independent semantic and XSD checks. Coverage of named coordinates and
  pinned corpus round trips does not certify live negotiation, vendor playback
  or the interpretation of every legal MusicXML performance instruction.
