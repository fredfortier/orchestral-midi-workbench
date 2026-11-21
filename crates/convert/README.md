# MusicXML ↔ MIDI 2.0: articulation mapping

The evolving companion assignment format and its implemented boundary are documented
in [RENDITION.md](RENDITION.md).

**Revision 2: mapping contract and Rust implementation.**
The finite executable acceptance domain is recorded in
[`tests/acceptance.json`](tests/acceptance.json).
The first draft is preserved in commit `17b933d`.
The current implementation also supports the published **M2-126 acoustic Piano
Profile**. [Translation defenses and adversarial review](mapping/TRANSLATIONS.md)
distinguish executable behavior, musical interpretations and explicit limits;
[the 12-score study](tests/CORPUS-STUDY.md) walks through both directions.
Earlier design requirements below do not imply every legal MusicXML performance
instruction is implemented; diagnostics and current evidence are explicit.

The contract is to preserve every specified musical property, express every
M2-123 subclass, and distinguish standardized meaning from the performance
conventions needed where a score leaves choices open. Exactness means that no
specified property disappears or is falsely claimed to fit a standard field.
It cannot mean identical audio from independent libraries: M2-123 §5 standardizes
articulation names and leaves their sounds to their designers.

The practical basis is the publisher's complete 30-article Berlin Series manual
index and 60 shared SINE/Kontakt guide articles, archived with the standards in
[`../research/`](../research/README.md). See the collection-by-collection
[Berlin audit](../research/BERLIN-AUDIT.md). Berlin collections run in SINEplayer
or Kontakt; they are not independent VST implementations of MIDI-CI. This is a
manual-grounded mapping contract, not an audition or conformance certification.

## 1. Authority and exactness

| Locally archived authority | Governs |
| --- | --- |
| [M2-123 v1.0](../research/standards/midi/M2-123-UM_v1-0_Note-On_Orchestral_Articulation_Profile.pdf), §§3–9, Tables 7–25 | Profile coordinates, optional controllers, and discovery payloads. |
| [M2-126 v1.0](../research/standards/midi/M2-126-UM_v1-0_Piano_Profile.pdf), §§3–10 | Acoustic Piano identity, dynamics, physical note/pedal response, optional controls and 31-byte details. |
| [MusicXML 4.0 release](../research/standards/musicxml/musicxml-v4.0.zip), [XSD](../research/standards/musicxml/musicxml-4.0/schema/musicxml.xsd), [official reference](https://www.w3.org/2021/06/musicxml40/) | Typed notation/playback semantics, scope, and extension carriers. |
| [M2-104 v1.1.2](../research/standards/midi/M2-104-UM_v1-1-2_UMP_and_MIDI_2-0_Protocol_Specification.pdf) | UMP, controllers, absolute Pitch 7.25, and Flex Data. |
| [M2-101 v1.2](../research/standards/midi/M2-101-UM_v1-2_MIDI-CI_Specification.pdf), [M2-102 v1.1](../research/standards/midi/M2-102-U_v1-1_Common_Rules_for_MIDI-CI_Profiles.pdf) | Discovery, profile enablement, identities, and controller assignment rules. |
| [M2-103 v1.2](../research/standards/midi/M2-103-UM_v1-2_Common_Rules_for_MIDI-CI_Property_Exchange.pdf), §§6.1.4, 8.4, 14.4 | Custom `X-` resources, `x-` properties, and JSON Schemas. |
| [M2-113 v1.0](../research/standards/midi/M2-113-UM_1-0_Default_Control_Change_Mapping_Profile.pdf) | Standard controller meanings; CC1 is not standardized as orchestral dynamic crossfade. |
| [M2-115 v1.0.2](../research/standards/midi/M2-115-U_v1-0-2_MIDI_2.0_Bit_Scaling_and_Resolution.pdf) | Min-center-max scaling versus fixed-point zero extension. |
| [M2-116 v1.0](../research/standards/midi/M2-116-U_v1-0_MIDI_Clip_File_Specification.pdf) | `.midi2` files; §6.4 excludes Property Exchange resources from clips. |
| [SMuFL 1.4](../research/standards/smufl/smufl-v1.4.zip), [glyph registry](../research/standards/smufl/glyphnames-v1.4.json) | Actual glyph identities used by MusicXML extension attributes. |

MusicXML 4.1 and SMuFL 1.5 are drafts at the research snapshot. Pin 4.0 and 1.4.
Mappings marked **policy** below are explicit project conventions, not newly
claimed W3C/MMA requirements. `shall` defines a requirement of this converter.

Handle published inconsistencies explicitly:

- Table 12: `14/7` is Lontano, `14/8` Singing into Instrument. Non-normative
  Appendix A.5 omits Lontano and shifts the latter; use the normative table.
- §6.1's fallback to “Classification 0x01” means classification number 1, whose
  Attribute Type is `0x10`, not manufacturer-specific Attribute Type 1.
- Note Off Figure 2 reserves bits 11–3, although §7.1 prose mentions variation.
  Emit those bits as zero and reject off-variation requests pending clarification.
- With-vibrato is `10/9`, without-vibrato `10/8`, despite an explanatory typo.
- MusicXML's generated trill reference has conflicting shared attribute
  descriptions; the pinned XSD supplies defaults for normalization.

Three guarantees are separate: **valid encoding**, **complete semantic coverage**,
and **verified playback**. Valid fields alone do not prove a sample implements a
specified transition, rate, or timbral refinement. M2-123 Appendix A is explicitly
non-normative and shall not restrict a technique to the listed instrument families.

## 2. Normalize musical intent, then encode it

An event shall retain independent facets, including their source paths:

| Facet | Required data |
| --- | --- |
| Lifecycle | Instrument/voice/staff, repeat occurrence, sounding pitch(es), onset/end, tie chain. |
| Production/attack | Ordinary, short, accented, harmonic, pizzicato, noise; attack character. |
| Connection | Predecessor/successor IDs, slurred/fingered/retongued/portamento, speed/style. |
| Timbre | Vibrato kind/envelope, blurred/exposed character, mallet/stop/registration. |
| Pattern | Trill interval/start/end/rate/count; run scale/direction/extent; grace pitches/times. |
| Dynamics | Attack velocity, dynamic layer, amplitude/expression, independent timed curves. |
| Position/mute | Position region/value, mute identity/amount, sampled/emulated provenance if specified. |
| Release | Ending sample, release time, pedal/damping, ending pitch gesture. |

Every specified property shall have a disposition: standard field, bound
variation/controller, custom sound, timed realization, or notation-only metadata.
An audible property cannot be hidden in the metadata disposition. Compatible
facets combine: harmonic staccato, muted legato, fluttertongue legato and sul-tasto
portato are legitimate requests, not automatic articulation conflicts.

A successful conversion produces the `.midi2` clip, a versioned performance/binding
manifest, a report, and the original MusicXML/MXL archive. The bundle preserves
notation and custom semantic identities absent from a bare UMP stream. It is a
project bundle, not a claim to implement the still-separate SMF2 Container format.

## 3. MusicXML carriers, scope, text, and glyphs

| Carrier | Exact rule |
| --- | --- |
| Typed `notations/articulations`, `technical`, `ornaments` | Normalize their actual meanings and compounds. Tenuto+staccato and detached-legato have the same combined-symbol intent. |
| `other-technical`, `other-articulation` | Accept the canonical terms in §5 and aliases below as a declared project vocabulary in standard extension elements. The words themselves are not W3C-standardized enums. |
| `direction-type/words` | The same finite vocabulary declares persistent techniques. Policy: explicit staff/voice restricts the target; absent staff applies to the whole part. Multi-instrument targets must resolve through their instrument binding. |
| `other-notation type=start/stop/single` | Match spans by part/staff/voice/number/term (number defaults to 1). Apply to attacks in the half-open start/stop interval. Independent facets may overlap. |
| `note/play` | MusicXML-defined per-note scope. |
| `sound/play` | MusicXML-defined persistent scope, targeted by instrument `id`; materialize into every Note On. |
| `sound@pizzicato` / `note@pizzicato` | Persistent setting / note override. Snap refines generic pizzicato. |

Text normalization is Unicode NFC, trim/collapse whitespace, lowercase, then
exact lookup. No substring or language-model interpretation. Typed data and
refinements shall agree; contradictory values fail `E_CONFLICT`. Explicit
replacement must identify the superseded source paths and preserve that decision.

### 3.1 Additional exact aliases and resets

| Alias | Normalization |
| --- | --- |
| `legato slurred` | core.legato, slurred connection. |
| `glissando legato` | core.glissando; keep interval/trajectory constraints. |
| `detache`, `martele`, `sautille`, `jete`, `gettato`, `cuivre` | The corresponding accented canonical spelling; gettato means short.jete. |
| `senza vib.`, `non vibrato`, `non-vibrato` | No vibrato; core.senza-vibrato when primary. |
| `con vib.`, `vibrato` | With vibrato; core.con-vibrato when primary. |
| `bartók pizzicato`, `bartok pizzicato`, `snap pizz.` | short.snap-pizzicato. |
| `louré`, `loure`, `mezzo staccato` | short.long-staccato, distinct from the default portato convention for dot+tenuto. |
| `flutter tongue`, `fluttertongue`, `flutter-tongue`, `bisbigliando` | repeat.tremolo plus the named production technique. |
| `razz` | repeat.growl. |
| `sul ponticello`, `sul tasto` | Bridge-side/nut-side position region; §7 supplies declared representative values. |
| `con sordino`, `con sord.` / `senza sordino`, `senza sord.` | Standard mute on / off, resolved by instrument family. |
| `arco` | Clear pizzicato only; preserve mute, position and dynamics. |
| `ordinario`, `ord.` | Clear persistent production/color/connection refinements and restore normal position; preserve mute, dynamics, program and tuning. |
| `sustains soft`, `sustains accented` | Ordinary sustained production + soft/accented attack qualifier; not automatically sustained marcato. |
| `staccato short`, `portato short`, `portato long` | The named base articulation + relative-length qualifier; do not substitute another subclass solely from a vendor length label. |
| `fluttertongue legato`, `fluttertongue portato`, `pizzicato tremolo` | Independent production, connection/attack and repetition facets, requiring their complete combination. |
| `molto vibrato`, `romantic vibrato`, `evolving vibrato` | With-vibrato + amount/style/envelope qualifier; never synchronized-vibrato merely because it is stronger. |

Bare col legno, harmonic/flageolet, tap, rip, or glissando can omit a distinction
that affects the sound. Retain the generic intent; use a declared instrument
default or additional semantic qualifier. A generic request may use its declared
default, but a specific request may not be reduced to a generic one.

### 3.2 Verified SMuFL identities

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
| `stringsHalfHarmonic`, `stringsOverpressureDownBow`, `stringsOverpressureUpBow`, `stringsBowOnTailpiece`, `stringsBowOnBridge` | Exact named technique plus direction where present; bound/custom descriptor under §8. No exact dedicated profile subclass. |

Other glyphs retain their real identities and need a semantic binding when they
specify an audible effect. Never fabricate a SMuFL name to hide a vocabulary gap.

## 4. Native predicates and complete realization

**T** means the row's canonical term or semantic ID directly requests that sound.
**P** adds explicit pattern/phrase constraints to T. G1/G2 are the glyph rules.
Every row accepts its canonical term and structured semantic ID; none is reachable
only through a numeric escape hatch. A profile candidate does not erase residual
pitch, timing, or connection constraints.

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

### 4.1 Deterministic constraint-preserving selection

1. Unify compatible facets. A refinement absorbs its generic marker: snap plus
   pizzicato is one request, not a conflict.
   For dot compounds, normalize tenuto+dot first (N9), then accent+dot (N6),
   then slurred dot (N5), then plain dot (N4). A selected compound absorbs only
   its own marks; extra attack/connection properties remain for binding selection.
2. Use a standard coordinate plus modifiers/controllers when those express all
   specified sound properties. Retain relational/timing constraints for playback.
3. Otherwise match a bound variation against the **complete** remaining descriptor.
   Choose the first match in the binding's declared preference order. Equal-priority
   ambiguity fails. A variation number has no global timbral meaning.
4. Otherwise use an exact custom binding or timed realization. Generated ordinary
   notes do not reproduce a specifically requested fluttertongue/harmonic production.
5. If no implementation satisfies a specified property, return `E_UNREPRESENTABLE`
   identifying it. A deliberately accepted approximation has its own reported mode;
   it is never labelled exact through a silent fallback.

Unspecified details follow the receiver's standard freedom or a labelled project
policy. Specified details are never overwritten by that freedom.

## 5. All 106 standardized Note On subclasses

`AT/sub` uses hexadecimal wire numbers, not the one-based UI numbers. The table is
generated from the checked [catalog](mapping/orchestral-subclasses.json). Canonical
terms are accepted in §3's standard MusicXML extension carriers. All rows accept
the named semantic ID through §8. Variation defaults to 0 unless a complete binding
selects another. T/P are positive named-input rules, not missing mappings.

<!-- BEGIN PROFILE CATALOG -->
| AT/sub | Sound | Named semantic input | Canonical MusicXML extension term | Additional native rule |
| --- | --- | --- | --- | --- |
| `10/0` | Normal, first bank of variations | `core.normal` | ordinary | N0 |
| `10/1` | Normal, second bank of variations | `core.normal-extra` | ordinary additional variations | T |
| `10/2` | Legato / slurred legato | `core.legato` | legato | N1 |
| `10/3` | Molto legato | `core.molto-legato` | molto legato | T |
| `10/4` | Glissando / string portamento sound | `core.glissando` | portamento | N2 |
| `10/5` | Detaché | `core.detache` | detaché | T |
| `10/6` | Marcato sustain | `core.marcato` | marcato | N3 |
| `10/7` | Martelé | `core.martele` | martelé | T |
| `10/8` | Explicitly without vibrato | `core.senza-vibrato` | senza vibrato | T |
| `10/9` | Explicitly with vibrato | `core.con-vibrato` | con vibrato | T |
| `10/A` | Synchronized vibrato | `core.synchronized-vibrato` | synchronized vibrato | T |
| `11/0` | Ordinary staccato, off-string for strings | `short.staccato-off` | staccato off string | N4 |
| `11/1` | Ordinary staccato, on-string | `short.staccato-on` | staccato on string | T |
| `11/2` | Single-bow slurred staccato | `short.slurred-staccato` | slurred staccato | N5 |
| `11/3` | Accented staccato | `short.accented-staccato` | accented staccato | N6 |
| `11/4` | Staccatissimo / deliberately damped short | `short.staccatissimo` | staccatissimo | N7 |
| `11/5` | Spiccato | `short.spiccato` | spiccato | N8 |
| `11/6` | Sautillé | `short.sautille` | sautillé | T |
| `11/7` | Martellato | `short.martellato` | martellato | T |
| `11/8` | Long / mezzo staccato, louré | `short.long-staccato` | long staccato | T |
| `11/9` | Portato | `short.portato` | portato | N9 |
| `11/A` | Pizzicato | `short.pizzicato` | pizzicato | N10 |
| `11/B` | Snap / Bartók pizzicato | `short.snap-pizzicato` | snap pizzicato | N11 |
| `11/C` | Col legno battuto | `short.col-legno-battuto` | col legno battuto | T |
| `11/D` | Col legno gestrichen | `short.col-legno-gestrichen` | col legno gestrichen | T |
| `11/E` | Pitched string-hand tap | `short.string-hand-tap` | string hand tap | T |
| `11/F` | Jeté | `short.jete` | jeté | G1 |
| `12/0` | Tremolo / flutter tongue / equivalent roll | `repeat.tremolo` | unmeasured tremolo | N12 |
| `12/1` | Growl / razz | `repeat.growl` | growl | T |
| `12/2` | Coloristic tremolo | `repeat.coloristic-tremolo` | coloristic tremolo | T |
| `12/3` | Same-pitch timbral trill | `repeat.timbral-trill` | timbral trill | T |
| `12/4` | Two-repeat sound | `repeat.two` | two repeats | N13 |
| `12/5` | Three-repeat sound | `repeat.three` | three repeats | N14 |
| `12/6` | Four-repeat sound | `repeat.four` | four repeats | P |
| `12/7` | Five-repeat sound | `repeat.five` | five repeats | P |
| `12/8` | Six-repeat sound | `repeat.six` | six repeats | P |
| `12/9` | Faster repeat sound | `repeat.fast` | fast repeats | P |
| `13/0` | Semitone trill, main note first | `trill.semitone-main-first` | semitone trill main first | N15 |
| `13/1` | Semitone trill, upper note first | `trill.semitone-upper-first` | semitone trill upper first | N15 |
| `13/2` | Whole-tone trill, main note first | `trill.tone-main-first` | whole tone trill main first | N15 |
| `13/3` | Whole-tone trill, upper note first | `trill.tone-upper-first` | whole tone trill upper first | N15 |
| `13/4` | Minor-third alternation | `trill.minor-third` | minor third trill | N16 |
| `13/5` | Major-third alternation | `trill.major-third` | major third trill | N16 |
| `13/6` | Perfect-fourth alternation | `trill.fourth` | perfect fourth trill | N16 |
| `13/7` | Tritone alternation | `trill.tritone` | tritone trill | N16 |
| `13/8` | Perfect-fifth alternation | `trill.fifth` | perfect fifth trill | N16 |
| `13/9` | Minor-sixth alternation | `trill.minor-sixth` | minor sixth trill | N16 |
| `13/A` | Major-sixth alternation | `trill.major-sixth` | major sixth trill | N16 |
| `13/B` | Minor-seventh alternation | `trill.minor-seventh` | minor seventh trill | N16 |
| `13/C` | Major-seventh alternation | `trill.major-seventh` | major seventh trill | N16 |
| `13/D` | Octave alternation | `trill.octave` | octave trill | N16 |
| `14/0` | Natural harmonic | `color.natural-harmonic` | natural harmonic | N17 |
| `14/1` | Artificial harmonic | `color.artificial-harmonic` | artificial harmonic | N18 |
| `14/2` | Col legno tratto | `color.col-legno-tratto` | col legno tratto | T |
| `14/3` | Flautando | `color.flautando` | flautando | T |
| `14/4` | Polyphonic octave sound | `color.polyphonic-octaves` | polyphonic octaves | P |
| `14/5` | Polyphonic interval/chord sound | `color.polyphonic-chord` | polyphonic chord | P |
| `14/6` | Cuivré | `color.cuivre` | cuivré | T |
| `14/7` | Lontano | `color.lontano` | lontano | T |
| `14/8` | Singing into the instrument | `color.singing` | singing into instrument | T |
| `15/0` | Fall at end, starting at note pitch | `gesture.fall-end` | fall at end | N19 |
| `15/1` | Fall into note from above | `gesture.fall-start` | fall into note | N20 |
| `15/2` | Rise at end, starting at note pitch | `gesture.rise-end` | rise at end | N21 |
| `15/3` | Rise into note from below | `gesture.rise-start` | rise into note | N22 |
| `15/4` | Dip and return to pitch | `gesture.blue-down` | blue note down and return | P |
| `15/5` | Rise and return to pitch | `gesture.blue-up` | blue note up and return | P |
| `15/6` | Leading grace sound from below | `gesture.grace-below` | grace from below | N23 |
| `15/7` | Leading grace sound from above | `gesture.grace-above` | grace from above | N23 |
| `15/8` | Shake sound | `gesture.shake` | shake | N24 |
| `15/9` | Prerecorded crescendo | `gesture.crescendo` | sampled crescendo | N25 |
| `15/A` | Prerecorded decrescendo | `gesture.decrescendo` | sampled decrescendo | N25 |
| `15/B` | Prerecorded swell then fade | `gesture.swell` | sampled crescendo decrescendo | N25 |
| `15/C` | Prerecorded fade then swell | `gesture.reverse-swell` | sampled decrescendo crescendo | N25 |
| `15/D` | Sforzando then crescendo sound | `gesture.sfz-crescendo` | sampled sforzando crescendo | N25 |
| `16/0` | Playable runs | `run.playable` | playable runs | T |
| `16/1` | Playable tremolos | `run.playable-tremolo` | playable tremolos | T |
| `16/2` | Playable glissando | `run.playable-glissando` | playable glissando | T |
| `16/3` | Major scale/run | `run.major` | sampled major run | P |
| `16/4` | Minor scale/run | `run.minor` | sampled minor run | P |
| `16/5` | Dominant-seventh scale/run | `run.dominant-seventh` | sampled dominant seventh run | P |
| `16/6` | Diminished whole–half / Other 1 | `run.diminished-whole-half` | sampled diminished whole half run | P |
| `16/7` | Diminished half–whole / Other 2 | `run.diminished-half-whole` | sampled diminished half whole run | P |
| `16/8` | Whole-tone collection containing C | `run.whole-tone-c` | sampled whole tone run containing c | P |
| `16/9` | Whole-tone collection containing C♯ | `run.whole-tone-c-sharp` | sampled whole tone run containing c sharp | P |
| `16/A` | Major pentatonic scale/run | `run.pentatonic-major` | sampled major pentatonic run | P |
| `16/B` | Minor pentatonic scale/run | `run.pentatonic-minor` | sampled minor pentatonic run | P |
| `16/C` | Lydian scale/run | `run.lydian` | sampled lydian run | P |
| `16/D` | Lydian ♭7 scale/run | `run.lydian-flat-seven` | sampled lydian flat seven run | P |
| `16/E` | Chromatic scale/run | `run.chromatic` | sampled chromatic run | P |
| `16/F` | Other scale/run | `run.other` | sampled other run | P |
| `17/0` | Sustained assorted noise / air effect | `effect.sustained-noise` | sustained noise | T |
| `17/1` | Sustained behind-bridge effect | `effect.behind-bridge` | bow behind bridge | G2 |
| `17/2` | Sustained random pizzicato | `effect.random-pizzicato` | random pizzicato | P |
| `17/3` | Harmonic glissando effect | `effect.harmonic-glissando` | harmonic glissando effect | P |
| `17/4` | Random glissando effect | `effect.random-glissando` | random glissando effect | P |
| `17/5` | Short air / chiff / squeak / lip-pizz effect | `effect.short-air` | short air noise | T |
| `17/6` | Sustained mechanical noise | `effect.mechanical-long` | sustained mechanical noise | T |
| `17/7` | Short mechanical noise | `effect.mechanical-short` | short mechanical noise | T |
| `17/8` | Finger glide noise | `effect.finger-glide` | finger glide noise | T |
| `17/9` | Fret noise | `effect.fret-noise` | fret noise | T |
| `17/A` | Thump | `effect.thump` | thump | T |
| `17/B` | Knock | `effect.knock` | knock | T |
| `17/C` | Slap | `effect.slap` | body slap | T |
| `17/D` | Pop | `effect.pop` | pop | T |
| `17/E` | Tap | `effect.tap` | body tap | T |
| `17/F` | Click | `effect.click` | click | T |
<!-- END PROFILE CATALOG -->

### 5.1 Required interval/phrase interpretation

For `13/0,1` the auxiliary is 1 semitone above the profile root; for `13/2,3`,
2 semitones. Even subclasses start on root, odd on upper. `13/4..D` correspond to
intervals 3,4,5,6,7,8,9,10,11,12 semitones; their first member is receiver-defined.
Specified rate/count/start/end/turn/acceleration needs binding support or event
realization. Keep original score pitch and profile trigger-root identity distinct.
“Baroque” here sets starting member, not a score-wide historical style.
Unison rearticulation is repeat intent; use 12/3 only when alternating timbre,
fingering or strings is actually specified. Fractional intervals, intervals above
an octave and lower-neighbor patterns that cannot preserve start/end semantics
need exact timed pitches or a custom binding; do not round them into 13/0..D.

`12/4..8` request 2..6 repeats; `12/9` has no numerical rate. Continuous versus
finite patterns, measured subdivisions, and release-triggered strokes remain
separate. `16/3..F` do not encode root key, direction, octave extent, rate or
internal rhythm. Those are binding parameters, never bow-direction bits.
For generated/adapter recipes the count policy is total strokes, matching Berlin
Doubles/Triples. M2-123's names alone do not define a sampled attack algorithm.

`10/1` extends normal variations to 32 with `10/0`; it is not a second instrument.
`14/4,5` select a polyphonic sound; ordinary written chords still produce individual
notes. `15/9..D` name prerecorded dynamic shapes, not calibrated endpoints.

### 5.2 All reserved and custom coordinates

| AT | Subclasses | Rule |
| --- | --- | --- |
| `10` | `B..F` | Reserved; reject. |
| `12` | `A..F` | Reserved; reject. |
| `13` | `E..F` | Reserved; reject. |
| `14` | `9..F` | Reserved; reject. |
| `15` | `E..F` | Reserved; reject. |
| `18..19` | `0..F` | Reserved classifications; reject. |
| `1A` | `0..F` | Exact custom binding required. |
| `1B` | `0..F` | Exact custom binding required. |
| `1C` | `0..F` | Exact custom binding required. |
| `1D` | `0..F` | Exact custom binding required. |
| `1E` | `0..F` | Exact custom binding required. |
| `1F` | `0..F` | Exact custom binding required. |

`11`, `16`, `17` have sixteen named subclasses each. Custom capacity is
96 subclass coordinates ×16 variations per instrument/program context. Exhaustion
requires explicit context changes or fails; reserved bits/classes remain reserved.

## 6. Note On/Off, modifiers and wire values

Single Channel Profile ID: `7E 21 01 01 01`; levels 2..127 are reserved. Every
attack carries its complete effective articulation, independent of earlier notes.

```text
on.word0 = 0x40000000 | (g<<24) | (9<<20) | (c<<16) | (note<<8) | AT
on.data  = (sub<<12) | (variation<<8) | (direction<<6) | (resetRR<<3) | string
on.word1 = (velocity16<<16) | on.data
```

Group/channel 0..15, note 0..127, subclass/variation 0..15, direction 0..2,
resetRR 0/1, string 0..7. Bits 5..4 zero; direction 3 reserved. Direction
1=down/right-hand, 2=up/left-hand, 0=receiver-selected. Bow marks map directly;
fretting finger numbers do not specify a hand. String 1..6 maps directly,
7=other, 0=no assignment. Exact identities beyond that need a binding; do not
collapse distinct specified strings silently. RR reset is explicit, not inferred
from repeat barlines. Profile variation is not a round-robin index.

Example: group/channel 0, D5=74, velocity 0x9000, pizzicato variation 0,
down/right direction, reset RR, string 2:

```text
on.word0 = 0x40904A11
on.data  = 0xA04A
on.word1 = 0x9000A04A
```

Generic profile-specific Attribute Type `0x02` is not this orchestral encoding.

### 6.1 Every Note Off subclass

| Off AT/sub | Meaning / named release intent |
| --- | --- |
| `00/—` | Default receiver ending, data=0; distinct from suppressing a release sample. |
| `10/0` | No Note Off sample: `none`. |
| `10/1` | Soft ending: `soft`. |
| `10/2` | Hard ending: `hard`. |
| `10/3` | Pitch rise at release: `rise`. |
| `10/4` | Pitch fall at release: `fall`. |
| `10/5..F` | Reserved; reject. |
| `01/—` | Manufacturer-specific attribute; separate vendor codec, not this layout. |

```text
off.word0 = 0x40000000 | (g<<24) | (8<<20) | (c<<16) | (note<<8) | offAT
off.data  = (offSub<<12) | offString  # bits 11..3 zero, Figure 2
off.word1 = (releaseTime16<<16) | off.data
```

Always balance the logical attack with a Note Off, even if its sample has already
finished. Match allocated address/string. Default release time is `0x8000`,
0=longest, 0xFFFF=shortest. MusicXML end-dynamics describes ordinary release
velocity, not this inverted release-time parameter; preserve it through an exact
binding. Release-sample support and variable release-time support need separate
evidence; the latter's discovery bit does not advertise every release sample.
Do not duplicate a Note On fall/rise at Note Off.

## 7. Mute, position and tuning controllers

### 7.1 Every mute slot

Registered channel controller `0x20/0x22` selects type using `slot*0x08000000`.
Then send amount on `0x20/0x23`; every type message resets amount to maximum.

| Slot | Type | MusicXML source or named request |
| --- | --- | --- |
| 0 | None | play/mute=off, senza sordino. |
| 1 | Standard/straight | straight; on/con sordino for bowed strings or brass, with family-specific standard type. |
| 2 | Practice | practice. |
| 3 | Cup | cup. |
| 4 | Harmon stem in | harmon-stem. |
| 5 | Harmon stem extended | Named stem-extended request; preserve extension amount if specified. |
| 6 | Harmon stem removed | harmon-no-stem. |
| 7 | Plunger | plunger. |
| 8 | Bucket | bucket. |
| 9 | Mica | Named mica mute. |
| 10 | Solotone | solotone. |
| 11 | Whisper | Named whisper mute. |
| 12 | Hat | hat. |
| 13 | Hand | stop-hand. |
| 14 | Stopped/transposing mute | stop-mute; no double transposition of an already sounding MusicXML pitch. |
| 15 | Into stand | Named into-stand request. |
| 16..25 | Reserved | Reject. |
| 26..31 | Manufacturer mute 1..6 | Exact bound identity; do not invent a generic hotel/echo/palm equivalence. |

Amount 0=open, 0xFFFFFFFF=closed; fixed mutes use maximum. String-mute on/off
maps to slot 1/0 on bowed strings. Stopped/open/half-muted/harmon-mute symbols
need instrument and glyph context; a guitar open-string circle is not horn mute
removal. A qualitative half-muted instruction uses the binding's halfway setting,
not a claim of physically linear closure. Different simultaneous mute states need
independent channel state.

### 7.2 Position

Registered Per-Note Controller 0x0C: strings bridge=0, normal=0x80000000,
nut=0xFFFFFFFF; drums center/normal/rim use the same endpoints. Preserve a source
position region separately from its representative encoded value.

**Policy:** plain sul ponticello uses 0x08000000, within M2-123's suggested region;
plain sul tasto uses 0xF0000000, a project representative. Neither is a standardized
physical measurement. More specific positions need calibrated bindings. Initialize
the effective position per attack. Position alone does not express flautando,
overpressure, bow on tailpiece, or behind-bridge production.
Emit/reset this optional controller when the receiver supports it and it is used.
A receiver without it can satisfy a position request through an exact patch binding;
ordinary default position does not gratuitously require an unsupported controller.

### 7.3 Pitch and controller encoding

Resolve sounding pitch with exact rationals:
`12*(octave+1)+stepOffset+alter+chromaticTranspose+12*octaveTranspose`.
Do not double-apply key accidentals or display octave shifts. Harmonic sounding,
base and touching pitches remain distinct; a diamond notehead alone is insufficient.
Unpitched staff placement is not a trigger key.

Keep the articulation attribute and initialize **Registered Per-Note Controller 3,
absolute Pitch 7.25** before the attack: `round(pitch*2^25)`, note address
`floor(pitch)`, pitch in [0,128), encoded value fitting u32. Reinitialize reused
addresses, including integer pitches. Receiver tuning precision must be verified;
25 fractional bits on the wire do not guarantee that acoustic precision.

```text
registered channel word0 = 0x40000000|(g<<24)|(2<<20)|(c<<16)|(bank<<8)|index
registered per-note word0 = 0x40000000|(g<<24)|(c<<16)|(note<<8)|index
word1 = full unsigned value
```

Pitch and position address note number, not string ID. Same-pitch overlaps with
independent controls require separate allocated channels. Pitch 7.25 is fixed-point
and uses zero-extension/rounding translation, not min-center-max scaling.

## 8. Finer articulations: mechanisms authorized by the standards

| Need | Mechanism and limit |
| --- | --- |
| Missing score notation | MusicXML other-technical/other-articulation/other-notation; use smufl only for a real glyph. No invented typed elements. |
| Playback-only semantics | MusicXML play/other-play@type permits an agreed string payload, with standard per-note/persistent scope. The semantic schema below is project-defined. |
| Refinement inside a subclass | M2-123 variation 0..15, normal banks 10/0 and 10/1. Publish exact meaning/version; count-only discovery cannot identify expressive/blurred/fingered variants. |
| Sound outside the classifications | M2-123 §§5.2.10, 9.2 custom AT 1A..1F, subclass/variation, scoped by receiver/program. Keep direction/string/reset meanings unchanged. |
| Additional parameters | M2-104 assignable channel/per-note controllers, explicitly bound to endpoint-specific meanings. Do not repurpose registered controllers or CC1 while claiming default CC semantics. |
| Semantic-map discovery | M2-103 permits a custom X-ArticulationMap resource, advertised in `ResourceList` with JSON Schema. Standard-resource extension properties use x-; custom resource names require X-. The articulation schema itself is not standardized. |
| Proprietary profile | M2-102 §2.2 allows the implementer's assigned SysEx Manufacturer ID and manufacturer-defined profile bytes. Never borrow OT's identity or reserved standard IDs/levels. A new profile is unnecessary just to publish a custom resource/binding. |
| File preservation | Keep map/schema/hash beside the clip. M2-116 §6.4 excludes PE resources from the clip; load the manifest or query live. Target 0x40 remains its fixed 96 count bytes, never a private JSON container. |

### 8.1 A named semantic annotation

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

The qualifier URI is a project-defined identity for an OT-described property,
not a vendor-registered identifier. Typed spiccato and named spiccato agree;
blurred adds a specified property. It cannot be dropped by playing ordinary spiccato.

JSON requires unique keys and permits: technique (one §5 ID or bound custom URI),
qualifiers (URI→JSON value map), binding (versioned map ID), variation (0..15),
direction (0..2), string (0..7), resetRoundRobin (boolean), release
(none/soft/hard/rise/fall), releaseTime (0..65535), and replace (source-path array).
Unknown top-level keys or contradictions fail. Replacement names only those source
properties intentionally superseded; it never suppresses unrelated instructions.
Canonical paths are absolute XML paths with one-based same-name sibling indices;
attributes append /@name. They are identifiers, not executable XPath.

The machine-readable [annotation schema](mapping/annotation.schema.json) constrains
payload shape and field ranges. Admission also requires catalog membership or a
bound custom URI, unique JSON keys, agreement with native notation, and support
for every qualifier; JSON Schema alone cannot establish those semantic properties.

The following qualifier keys have project-defined typed meanings. Prefix all IDs
in this table with `urn:midi2workbench:`. Other IDs require their binding's schema.

| Qualifier suffix | Exact value domain and action |
| --- | --- |
| `position` | Integer 0..4294967295 → profile RPNC 0x0C. |
| `position-region` | `bridge-side` or `nut-side`; retain the region alongside §7's representative coordinate. |
| `mute` | Object with slot 0..15 or 26..31, amount 0..4294967295 (default maximum), and identity URI for a custom slot; emit type then amount. |
| `sounding-pitch` | Rational string n/d, d>0, pitch in [0,128); overrides only a source pitch interpretation explicitly identified for replacement, e.g. a harmonic base pitch. |
| `attack-velocity` | Integer 0..65535, literal MIDI-2 attack velocity; zero remains a legal MIDI-2 attack, not MIDI-1 Note Off. Conflicting source silence requires an explicit replacement. |
| `dynamic-level` | Integer 0..4294967295, desired normalized sampled-dynamic level; requires a declared assignable-controller/receiver binding, not a reassigned standard CC1. |
| `amplitude` | Integer 0..4294967295 → standard expression CC11 when applicable; distinct from dynamic-layer timbre. |
| `sample-length` | `short` or `long`, a relative sample identity, not a standardized duration or gate multiplier. |
| `mute-provenance` | `recorded` or `emulated`, requiring the corresponding realization when specified. |

Native facets retain those meanings without requiring duplicate annotations.
Connection styles, pattern constraints and vendor sample-character values use
versioned binding schemas; unknown values never acquire a meaning by coincidence
with a registered controller number.

### 8.2 Required binding and a workable allocation

A binding identifies adapter/product/revision, library/edition/revision,
instrument/section, saved patch-instance hash, and a complete semantic descriptor.
It specifies numeric trigger keys/channels, latched/momentary switches, control
assignments, switch-before-attack delay, release actions, velocity roles, host-tempo
behavior, ranges and parameter precision. Manual octave names alone are insufficient;
store numeric MIDI addresses.

For a configured Berlin Strings Violin I receiver, an **example, not an official
OT assignment**, could bind 1A/0 variation 0 to blurred spiccato, 1A/1 variation 0
to a specific ostinato/fingered connection recipe, and 1A/2 variation 0 to a fully
specified harmonic-staccato descriptor. Prefer a bound parent-subclass variation
when it expresses the same complete request. Preserve semantic identity either way.

Before playback compare map identity/version/content hash. Missing or changed map
fails `E_BINDING_MISMATCH`. Another library may negotiate different coordinates for
the same descriptor, which the player remaps and reports explicitly. Profile
Details counts do not do that semantic negotiation. Unknown qualifiers remain
intact and require binding support; they never degrade to ordinary sound.

#### Property Exchange serialization and concrete example

M2-103 §6.1 requires camelCase ASCII property names. Therefore, the URI-keyed
qualifier map used inside MusicXML shall **not** be copied verbatim into a PE
resource. Serialize it as a sorted array of `{id, valueJson}` records: the URI is
an `id` string value, and `valueJson` preserves the qualifier's JSON value as text.
Reject duplicate qualifier IDs; compare parsed values without losing decimal
precision. Default PE JSON is ASCII with non-ASCII characters escaped, chunked
according to the negotiated maximum SysEx size. It is not raw UTF-8 in SysEx7.

The [example receiver map](examples/berlin-receiver.example.json) and its
[schema](mapping/binding.schema.json) give a concrete channel-per-articulation
setup for Sustains, Spiccato and Spiccato Blurred on output channels 0,1,2.
These are **chosen configuration addresses**, not alleged factory defaults.
The example has `status=example-unverified` and a zero placeholder setup hash;
a player shall refuse to activate it until real versions/configuration/evidence
replace those placeholders. Setting a status flag alone is not verification.

The example schema specifies the demonstrated fixed-channel/direct-pitch recipe,
not every possible plugin recipe. Multi-key, damping, timing and advanced controller
recipes require their own complete declared adapter schema and versioned map,
satisfying the obligations in §§8–10. Unknown recipe fields never execute.
This permits extension without claiming that a generic JSON object is executable.
For its crossfade recipe, the most recent normalized dynamic-level value supplies
the target controller; absent one, upscale the incoming attack velocity using
M2-115 and use that level. Downscale that level to the target's 7-bit CC, set it
before the attack, then send the independently fixed style-selection key velocity.
CC11 amplitude remains separate. The declared transfer is a configured baseline,
not a universal acoustic calibration. Identity-pitch recipes accept only exactly
representable target pitches; fractional tuning requires an additional recipe.

The [Resource list example](examples/resource-list.example.json) advertises the
custom resource and a `midi+jsonschema` reference using M2-103 §14.4.2. A real
responder must also provide that schema through the standard `JSONSchema` resource.
The example JSON files are resource data, not complete MIDI-CI SysEx transactions
and not evidence that SINE itself exposes these resources. The adapter owns them.

### 8.3 Documented Berlin distinctions that need refinement

| Actual distinction | Required treatment |
| --- | --- |
| Blurred/exposed/serial/bold staccato or spiccato | Bound sample-character qualifier; ordinary 11/5 alone cannot identify blurred spiccato. |
| Fingered/rebowed/retongued/agile/pattern/rapid/expressive/cinematic legato | Independent connection descriptor, velocity-zone/forced-style settings and transition recipe. 10/2 alone does not encode them. |
| Evolving/romantic/strong/molto vibrato | 10/9 expresses presence, not the specific envelope/style; retain refinement. |
| Harmonic staccato/tremolo; fluttertongue portato/legato | Full compound descriptor, without dropping a facet or pretending two Note On attributes exist. |
| Long/short/one-bar/two-bar sampled gestures | Shape/duration/rate constraints and a validated matching/time-stretch recipe. |
| Hotel mute, recorded con sordino versus emulation | Actual mute identity/provenance; no invented equivalence to practice mute or to another recording process. |
| Harp près de la table, bisbigliando, true pedals | Contact/production/pedal spelling. A profile-family correspondence does not encode every physical string or trigger. |
| Specific multiphonics, jet whistle, tongue ram | Exact named effect/fingering/spectrum when specified; a broad effects/polyphonic subclass is only a category. |
| Aleatoric clusters, random staccati, Doppler | Voice count, pitch extent, rate, recorded/random behavior, transform parameters; not a round-robin-reset seed. |
| Mallet/stick/snare-wire state, key-velocity percussion, harpsichord stops | Instrument/program/parameter refinements, not universal subclass variations or attack velocities. |

## 9. Timing, dynamics, and the remaining score

Parse numbers as exact rationals and maintain quarter-note coordinates until file
serialization. Nonnegative rounding is nearest integer, half upward. Report every
non-exact numeric encoding with original and encoded values. Never silently alter
an explicit pitch, note time or event count to make a sample fit.

| MusicXML operation | Required mapping |
| --- | --- |
| Partwise/timewise/MXL | Accept the pinned corpus's 1.0–4.0 forms (absent version defaults to 1.0); normalize to identical logical parts and emit 4.0. Validate locally, follow container root selection, preserve original members. Opus/multiple roots need explicit selection. No external-entity/network fetching during conversion. |
| Divisions/duration/backup/forward | Exact cursor scheduling per part/voice/staff, including mid-measure divisions changes. Chord notes share preceding onset with independent duration. Do not multiply encoded durations by time-modification again. Cue notes are silent; print-object=no is not silence. |
| Tie/tied | Sounding tie combines contiguous equal-pitch notes of one voice/instrument into one lifecycle. Tied alone is graphical. Keep intermediate controllers, first attack and final release; never swallow a specified reattack. |
| Repeats/navigation/time-only | Finite occurrence graph for repeats/endings and sound da capo/dal segno/coda/fine. Plain backward repeat defaults to two passes; respect explicit repeat/jump filters. Preserve current performance state on jumps, applying destination instructions; resolve score divisions/transposition at written destination. Notes/pedals crossing a jump need explicit lifetime handling. Cyclic/unresolved navigation fails. |
| Pitch/alter/transpose | Exact sounding pitch; preserve spelling separately. Key signatures, clefs and octave-shift display instructions do not re-transpose already encoded pitches. Resolve transpose-double, staff-specific transposition, scordatura and harp pitch pedals in the instrument model. Unpitched notes need a numeric percussion map, not display-step/octave as pitch. |
| Attack/release/offset | Honor note offsets in active divisions; sound offset overrides direction offset, honoring offset@sound. No negative performed onset or nonpositive duration. |
| Tempo | Numeric sound tempo governs playback; otherwise resolve numeric metronome beat units/dots/modulation. Policy default=120 quarter BPM. Flex tempo=round(6e9/BPM), 10 ns/quarter, positive u32. Prompting tempo=0 or unresolved modulation needs explicit input. |
| Meter/key/metadata | Encode representable meter, key, titles, creators and lyrics with applicable M2-104 Flex Data; archive exact layout/spelling. Additive/nonbinary meter that does not fit remains structured metadata; do not change note durations to fit it. |
| Numeric dynamics | 100% means MIDI-1 forte 90. Let x=90*percent/100; require 0..127. At integer x use M2-115's exact 7→16 min-center-max lookup. At fractional x linearly interpolate adjacent lookup entries, then round. This preserves decimal source detail and the defined center; simple x/127 scaling does not. Zero suppresses attack. Explicit high-resolution values retain their full bits. |
| Printed dynamics | Policy nominal MIDI-1 levels: pppppp=4, ppppp=8, pppp=12, ppp=20, pp=32, p=45, mp=58, mf=72, f=90, ff=104, fff=116, ffff=120, fffff=124, ffffff=127; default mf. Explicit note/sound values override labels. Attack/envelope labels sf/sfz/fp/rfz and niente retain their separate meaning, not a steady velocity substitution. |
| Wedges/envelopes | Phrase-wide dynamic curve. Policy: interpolate nominal dynamic level linearly between explicit endpoints; without a terminal level, use one named level above/below start, bounded to 0..127 and reported as a default. Niente ends at zero. Sample endpoints and every 1/96 quarter, retaining the curve definition. More specific authored curves override this policy. Dynamic-layer and amplitude curves remain separate. |
| Accent/tenuto/shorts | Compounds use §4. Policy ordinary accent without a sampled accent adds 12 nominal velocity units, capped at 127 and reported as interpretation. Strong-accent uses its sound candidate. Tenuto/short marks retain written gate unless performance offsets or the selected binding change it. Do not shorten an already short sample twice. |
| Measured tremolo | Pulse = 2^-(baseFlags+tremoloStrokes) quarter notes × cumulative normal-notes/actual-notes ratio. `baseFlags` derives from note type (eighth=1, sixteenth=2), not merely the presence of beam XML. For two-note tremolo remove the synthetic 2:1 tremolo ratio from the pulse calculation; its span is the sum of the two encoded durations. Validate pulse/count against the span. Keep rate/count constraints for a sampled realization or generate the exact attacks. |
| Trills/turns/mordents | Keep all trill-sound/accidental attributes. Shared XSD defaults: upper start, whole step, no turn, no acceleration, 4 attacks, second=25%, last=75%; mordents have their documented main-start/3-attack defaults. Policy for integral attack count: first at 0, second at second-beat, last at last-beat, intervening attacks evenly spaced, final release at source end. A terminal turn/acceleration/fractional count not determined by these constraints requires a complete authored timing schedule or capable binding. |
| Grace | Honor explicit make-time or steal-time allocation. Without allocation, policy takes 1/8 of following principal duration for the whole group, divided equally, preserving the common score timeline. Make-time inserts shared time and requires consistent part alignment. Sampled grace realization must replace, not duplicate, generated grace notes. |
| Arpeggiate/non-arpeggiate | Keep all chord pitches. Policy staggers by 1/32 quarter in marked direction, limiting total spread to 1/4 of the shortest chord duration by scaling evenly. Non-arpeggiate forbids staggering. Explicit timing wins. |
| Slur/glissando/slide/bend | Retain note relationships, endpoints and specified trajectories. Indeterminate pitch shape stays indeterminate; a measured bend requires exact events or matching binding. Legato selection is independent of a plugin's overlap recipe. |
| Fermata/breath/caesura/swing | Keep the named timing intent. A versioned rendering policy or explicit schedule supplies any factor/gap/ratio absent from the source; no claim that a glyph standardizes a numeric duration. Missing policy fails `E_TIMING_POLICY` at the actual construct, rather than dropping it. |
| Piano pedals | sound damper/sostenuto/soft pedal → CC64/66/67; yes/no=max/zero; percentage → normalized full-width value. Engraved start/stop/change keeps order; change releases then presses. Harp pitch pedals and percussion damping are different operations. |
| Instruments/programs | Resolve score-instrument, note instrument, instrument-change and device IDs. MusicXML program/channel/bank are one-based: decrement program/channel, decrement combined bank then split MSB/LSB. Resolve endpoints and vendor patches through bindings. |
| Volume/pan/elevation | Preserve volume and expression separately. Pan is piecewise -180→0, 0→0x80000000, +180→0xFFFFFFFF. Elevation needs an explicit spatial parameter binding. Preferred midi-instrument values override deprecated sound fields. |
| Other playing instructions | Retain IPA, semi-pitched, fingering, string/hole/harp-pedal/beater/handbell/registration meaning through named descriptors and bindings. A fingering that only describes an already resolved pitch can be notation-only. Unknown playback-bearing data errors; it never becomes ordinary sound silently. |
| Harmony/figured bass/layout | Harmony annotations do not automatically generate accompaniment. Archive lyrics, spelling, editorial/listening/layout/credit information. Requested accompaniment needs authored voicing/time rules. |

Numerical interpretation policies fill details the score does not specify; the
report shall identify each use. They may not override a specified musical property.
Defaults are not misrepresented as requirements of either standard.

For MIDI-1 translation use M2-115 scaling, not an arbitrary floating-point rescale.
Its examples include 7→16: 64→0x8000, 70→0x8C30, 96→0xC104, 127→0xFFFF.
Fixed-point pitch uses zero-extension/rounding; ordinary velocity/CC uses
min-center-max. Report precision loss and a receiver's narrower significant bits.
M2-104 §D.2.1 additionally requires a downscaled Note On velocity of zero to become
1: MIDI-2 velocity zero is a legal attack, while MIDI-1 zero means Note Off.
This exception applies to translation, not to MusicXML's explicit zero-dynamics
silence decision.

## 10. Berlin adapter behavior required by the manuals

The [collection audit](../research/BERLIN-AUDIT.md) supplies source-specific examples.
The adapter shall:

1. Pin player format/version, library edition/revision, saved instrument layout
   and assignments. SINE auto-maps keys/CCs/programs as articulations are reordered;
   ordinal positions are not universal articulation identities.
2. Use fixed keyswitches, CC values or channels for SINE VST3 articulation selection.
   The loaded-articulations guide explicitly says VST3 does not react to Program
   Change switching. File-level logical programs need adapter translation there.
3. Treat legato as a property of compatible articulations, as SINE documents.
   Enabled legato is monophonic. Preserve independent voices with independent
   state, fixed style/zones and a validated transition/overlap recipe. A universal
   20 ms overlap is not specified by these standards or manuals.
4. Separate dynamic level from key velocity. Berlin longs commonly use CC1 crossfading,
   while velocity selects attack or portamento. Solo Strings documents expressive
   zones 1–99/100–127 and cinematic zones 1–99/100–119/120–127. Raising loudness
   shall not accidentally switch connection style. Bind those controls separately.
5. Keep CC11 amplitude/expression separate from CC1 sample-layer dynamics. If the
   upstream intent needs a full-resolution layer curve, bind an assignable MIDI-2
   parameter to the adapter's CC1 output. Declare its private meaning explicitly;
   it is not a newly registered orchestral control or a redefinition of CC11.
6. Implement multi-message recipes. Trills Orchestrator expects rolled/ordered held
   keys and says simultaneous keys do not work. Bind a positive inter-key delay,
   mode and balanced releases. Doubles play stroke two on key release; Triples add
   a third after an equal interval. One trigger per desired stroke duplicates them.
7. Configure host/manual tempo, auto-tempo, stretch ratios, Lock to Beat and phrase
   duration. A Note On cannot convey them all. A patch named long/short is not a
   synchronization guarantee. Validate actual range and duration constraints.
8. Distinguish instrument mute from audio muting and true damping from gate length.
   Berlin Percussion's enabled True Damping uses CC64 to let notes ring; muted hits
   can use separate samples. Legacy con-sordino simulation and separately recorded
   Con Sordino Strings are distinct when the source requests that distinction.
9. Compute trigger from instrument mode, not only sounding pitch. True-pedal harp
   maps white keys to strings; percussion menus map technique/hand to keys; key-
   velocity instruments put layers on keys. Preserve score spelling and pitch while
   translating trigger identities. Store numeric keys, not ambiguous octave names.
10. Isolate switching/controller/legato/release state across sounding voices. Capsule
    documents Mono/Poly KS and Switch Imm behaviors; configurations that alter held
    neighbors require independent instances or incompatible-state rejection.
    SINE's Articulation Options guide says many GUI option changes interrupt audio;
    per-note behavior must use preconfigured copies or supported MIDI controls,
    not repeated GUI edits during playback.

A reproducible baseline is to pin the required SINE articulations, use its
channel-per-articulation scheme, record numeric channels, duplicate independent
legato voices, and route complete descriptors through those fixed recipes. This
is an actionable configuration path, not a claim that every Berlin patch has a
public universal keyswitch or already implements MIDI-CI.

## 11. Routing, clip serialization and live negotiation

- Each instrument has an ordered pool of (endpoint, group, channel) slots. Each
  wire (group,channel) identifies one endpoint in a file. Use the lowest compatible
  slot for the entire audible/pedal/release lifetime. Same-pitch independent per-
  note controls need separate addresses. No hidden borrowing or dropped notes.
- Same-tick dependencies: ending releases; pedal changes; program/channel state;
  mute type then amount; pitch/position initialization; new attacks; explicitly
  post-attack automation. Keep authored controller order, rejecting contradictions;
  independent events use stable route/source/occurrence ordering.
- TPQ=lcm(480, denominators of all rational event times), requiring TPQ≤65535.
  File: big-endian SMF2CLIP; initial broadcast-MUID Set Profile On SysEx7 per used
  channel; DCS(0)+DCTPQ; configuration tempo/meter; Start of Clip; timed UMPs;
  End of Clip. Initial profile declarations have no preceding DCS. Configuration
  and sequence clocks are distinct; initialize sequence tempo/meter again.
- Gaps above 0xFFFFF ticks use DCS(max)+Null Utility before the remaining delta
  (M2-116 §3.2.2). Require complete UMP lengths, balanced notes/pedals/SysEx, and
  no trailing data after End of Clip. On conversion error publish no final clip.
- File declarations are not negotiation. Before playback discover current MUIDs,
  inquire/enable/verify the profile on each channel, establish required controller
  semantics, and verify descriptor bindings and optional capabilities.
- Details target 0x01 is 130 bytes: two 7-bit-packed feature bytes followed by
  8×16 counts for classes 10..17/subclasses 0..F. D0=mute type, D1=mute amount,
  D2=position, D3=release time, D4=custom discovery; remaining bits reserved.
  Target 0x40 is 96 counts for classes 1A..1F. Counts are 0..16 and carry no names.
- Unknown optional support is not confirmed support. Profile fallback tries another
  variation of the same subclass, then subclass 0 of that class, then 10/0.
  Keep authored intent and report substitution. Such a fallback does not prove
  that a specific requested effect played or that exact playback succeeded.

## 12. Acceptance and current verification limits

Before claiming an implemented converter, verify parsing/normalization, every
catalog row/native predicate, compound facets, residual constraints, modifiers,
controllers, tuning/timing, custom-map identity, concrete adapter recipes,
independent clip decoding, and live negotiation success/failure. Preserve original
bytes/hash, canonical source paths, repeat occurrences and numerical decisions.

Current executable checks include the converter and its independent corpus/CLI oracles:

```sh
node crates/research/check.mjs
cargo check --manifest-path crates/Cargo.toml --workspace --locked
cargo test --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --locked
cargo fmt --manifest-path crates/Cargo.toml --all --check
cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets -- -D warnings
cargo doc --manifest-path crates/Cargo.toml --workspace --no-deps
```

These checks cover catalog completeness, source hashes, all 30 manual records,
real SMuFL names, schema-valid annotation examples, and wire/scaling examples.
They do not instantiate, audition, or certify the Berlin libraries. The library exposes `xml_to_midi2` and `midi2_to_xml`; its `convert` binary uses
`--to midi2|musicxml|mxl`. The typed score and writer come from `musicxml` 1.1.2;
UMP builders and primitives come from `midi2` 0.11.1. Narrow local adapters cover
missing CI profile messages, orchestral note attributes, registered position
controller 12, Piano Attribute 04/position controllers 13–15 and Piano Description Flex text, and the upstream program-bank byte-order defect. Decimal parsing
uses `rust_decimal`, timeline arithmetic uses `num-rational`, archives use `zip`,
and extension validation uses `jsonschema`. The converter needs Rust 1.87; the
independent offline XSD oracle needs `python3-lxml`.


### Running the converter

```sh
cargo run --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --bin convert -- \
  music21/music21/corpus/bach/bwv267.mxl --to midi2 -o /tmp/bwv267.midi2
cargo run --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --bin convert -- \
  /tmp/bwv267.midi2 --to musicxml -o /tmp/bwv267.musicxml
```

Default artifacts are the clip, `.convert.json` companion, `.report.json`, and
original `.source.mxl`/`.source.musicxml` archive. Outputs are staged before
publication, with the companion published last. Existing paths require `--force`.
`--policy` supplies the versioned rendering JSON. `--bindings` validates and
retains a verified receiver/program recipe in the companion for a future live
adapter; file conversion uses the portable category and refinement convention.
It does not run sample-library recipes. Unverified example bindings are rejected.

Without a companion, reconstruction emits canonical notation from actual clip
events. A matching companion restores source structure only after the archived
source is rendered again and compared with the actual decoded performance.
Edited clips use their current events. Declared private refinement tokens use
the embedded dictionary; an explicitly supplied dictionary must agree.

Canonical scores use one part per route, deterministic sharp spelling, exact
onsets/durations and the MIDI Clip ticks-per-quarter grid. That grid is the least
common multiple of all exact time denominators, preferably also divisible by 480
when the 16-bit field permits it. Legacy zero-length forward/backup cursor operations are harmless no-ops.
There is no timing quantization. A larger grid
fails explicitly. Pitch 7.25 uses nearest representable values, clamping at the field endpoints;
rounding records the original and encoded pitch.
The performance comparator sorts notes by onset/end/route/key, omits source paths,
and retains other messages, order, configuration time, and their relation to attacks.
This defines the current executable comparison, not proof of every source rule.

The finite acceptance domain and current evidence are recorded in
`tests/acceptance.json`. Re-evaluate with `python3 crates/convert/tests/calibrate.py`.
This establishes the pinned corpus and named synthetic cases, not all possible
MusicXML semantics or third-party receiver behavior. Explicit accelerated/terminal
ornament schedules, paired-note tremolo, numeric Fine duration schedules and
unresolved multi-instrument or note-instance bindings produce diagnostics.
Terminal grace notes support preceding allocation; a terminal following/make-time
allocation without a following boundary is rejected. Unknown private playback
carriers and dictionary versions are rejected. No hardware/audio certification
or installed Berlin library playback is claimed.


Upstream `musicxml` 1.1.2 integer-backed semitones/divisions and silently filtered
collection errors require a lexical adapter: structural placeholders enter its
typed view, while exact decimal values remain authoritative for the performance.
Each note is deserialized explicitly so a parser failure cannot silently remove
it. Fractional alterations are restored after writing; timewise containers are
rearranged using the upstream `XmlElement` type rather than a temporary serialized
score. These are local compatibility gaps, not additional MusicXML model types.

The Rust `rules::VOCABULARY` declaration is authoritative for the catalog. After
a reviewed mapping change, regenerate its JSON and Markdown projections with
`UPDATE_CONVERT_CATALOG=1 cargo test --manifest-path crates/Cargo.toml -p musicxml-midi2-convert --test conversion catalog_is_single_source`.
Ordinary acceptance compares all three projections without writing files.


### Acoustic Piano and composer remarks

Piano Profile ID is `[7E,20,00,01,01]`; Orchestral Articulation Profile ID remains
`[7E,21,01,01,01]`. They are existing standard profiles. A pure ordinary Piano
channel declares Piano only; articulated Piano can declare both under M2-102's
compatible-profile rules. Live use must negotiate receiver support for both.
The CI engine has `Engine::new_piano` for Piano transactions; an instance manages
one configured profile. It does not simulate a reproducing piano or claim support
for optional custom velocity-curve transactions, which receive an unsupported NAK.

Piano labels/sound IDs select profile interpretation; program numbers are preserved
without assuming a GM sound set. Default unmarked Piano dynamics use mf, center
79, as a stated rendering convention. Textual Piano f is 92, while authored MusicXML
numeric dynamics remain percent of its default forte 90. The inverse labels Piano
velocities using the published inclusive ranges, retaining exact numerical values.
Piano key-velocity Attribute 04 and a nontrivial orchestral articulation cannot
occupy the same Attribute slot; explicit conflict is diagnosed.

MusicXML has no generic standardized “comment field.” Score metadata, timed
`direction/words`/rehearsal, editorial `footnote`, and `other-*` have different
scopes. Timed text uses M2-104 Unknown Performance Text (bank2/status0); global text
uses Unknown Metadata Text (bank1/status0). Voice/staff/role has no native UMP field,
so scoped text uses `urn:midi2workbench:direction-remark:v1:` plus JSON in that
standard UTF-8 carrier. Note details use the existing `score-remarks` qualifier
and explicit per-note dictionary. These are declared conventions, not additional
MIDI-CI profiles. Native text is visible again on canonical MusicXML export.

M2-126's bank1/status0D is specifically **Instrument Used: Piano Description**,
not a universal instrument-name message. Other instruments retain identity in
Unknown Metadata Text with `urn:midi2workbench:instrument:v1:` plus a JSON string.
Original numeric programs remain independent. `remarks::literal_values` allows a
future explicit regex/sample-library binding to inspect note remarks; conversion
does not execute arbitrary prose or invent vendor keyswitches.

## Note ownership for native realization

Companions retain the encoded MIDI in `expected_midi_base64` and optionally
include `note_sources`. Each row records the original source path and route,
encoded route, exact onset/end and key in canonical `(onset, route, key, end)`
order. `clip::encode_with_sources` returns these rows alongside MIDI bytes,
dictionary and report. The refinement marker carries the complete dictionary.

These values preserve part ownership when the encoder isolates overlapping
same-key voices onto different routes. Consumers compare decoded musical values,
row counts and encoded note coordinates directly. Original notation restoration
also re-renders the archived source under its declared policy before reuse.

### Self-contained controller exports

[ARCHITECTURE.md](ARCHITECTURE.md) owns validation and measured score reconstruction.
`convert FILE --validate --report analysis.json` inventories admissibility without
requiring a companion. Files with embedded musical part identities use measured
notation with bounded rhythm inference; legacy canonical route reconstruction
remains exact. Newly exported arrangements do not discover adjacent companions.
See [the export workflow](../skills/bitwig-export-midi2/SKILL.md).
