# Translation defenses and adversarial review

This converter acts as a virtual performer preparing vendor-neutral requests for
an instrument renderer. It does not prove a particular sample library can realize
those requests. Names such as Berlin and Strezov identify possible receivers, not
new MIDI-CI profiles or portable keyswitch addresses.

## Authority and burden of proof

The controlling orchestral document is [M2-123-UM v1.0, 2024-03-28](../../research/standards/midi/M2-123-UM_v1-0_Note-On_Orchestral_Articulation_Profile.pdf).
Read §§2.4, 4–9 and Appendix A together. §5 standardizes names, while sound
design remains the receiver's decision. The tables' alternative descriptions are
informative hints; Appendix A explicitly says its instrument applications are
non-mandatory. Neither establishes an exact duration, bow speed, trill rate or
sample implementation. The introduction says eight instrument categories, but
Appendix A Table 26 prints **nine** columns: strings, woodwinds, brass, unpitched
percussion, tuned percussion, guitar/plucked, harp, keyboard/organ, choir/vocals.
The eight **articulation classifications** 0x10–0x17 are a different enumeration.

The controlling notation document is the complete [MusicXML 4.0 release](../../research/standards/musicxml/musicxml-v4.0.zip), especially its
[XSD annotations](../../research/standards/musicxml/musicxml-4.0/schema/musicxml.xsd)
and [official element reference](https://www.w3.org/2021/06/musicxml40/musicxml-reference/).
The user's formats.music link is a 4.1 draft, useful supplementary context but
not substituted silently for the pinned 4.0 contract.

Piano uses [M2-126-UM v1.0](../../research/standards/midi/M2-126-UM_v1-0_Piano_Profile.pdf), document dated 2026-07-08,
[published 2026-07-22](https://midi.org/midi-ci-profile-for-piano).
Its [implementation guide](../../research/standards/midi/M2-126-IG_v1-0_Piano_Profile_Implementation_Guide.pdf)
is informative. M2-104, M2-101/102, M2-115 and M2-116 govern transport, negotiation,
numbers and files respectively; exact archived references are in the workpad.

[Open Music Theory, Other Aspects of Notation](https://viva.pressbooks.pub/openmusictheory/chapter/other-aspects-of-notation/)
is supplementary musical explanation of connection/separation, attack emphasis,
relative dynamics, tempo and prolongation. It does not supply universal velocity
increments, gate percentages or fermata multipliers. Vendor manuals substantiate
vendor refinements only. JavaScript in this repository is archaeology and a
cross-check, never authority over a published specification. Searches of publisher
material and public MusicXML/MIDI-CI mapping references did not locate an existing
complete authoritative crosswalk; that is a bounded research finding, not proof
that no such work exists anywhere.

## Rule hierarchy implemented in Rust

1. Preserve authored playback numbers and explicit structured semantic input.
   Sounding pitch, attack/release offsets and exact velocity outrank a renderer's
   defaults. Explicit incompatible primary techniques are errors, not XML-order
   winners. Recognized extension terms are a **declared convention**, not W3C enums.
2. Read native notation as independent evidence. Compose compatible production,
   accent, connection, harmonic, position, mute and release facets. Local note
   parameters override inherited playback settings; numbered spans are scoped by
   part/staff/voice/number. The stop boundary is half-open.
3. Use instrument identity to select the standard profile/physical interpretation.
   An explicit MusicXML sound ID is stronger evidence than a label. A numeric
   program without a declared sound set does not prove piano, violin, or synth.
4. Use actual durations, sounding intervals, authored attack/release, dynamics and
   context where they constrain the result. Measured ornaments use duration and
   tuplets; trill accidentals and start-note constrain pitches; hairpins operate on
   the correct instrument response. Do not fabricate a speed threshold separating
   staccato, spiccato and sautillé. Short or loud alone cannot establish bow mechanics.
5. When evidence only establishes a broader intention, choose its documented
   ordinary representative and retain the residual distinction/provenance. Unknown
   prose remains scoped text for explicit human/library bindings. Ambiguity does
   not authorize replacing the instrument or discarding its instruction.

These are ordinary `match`/pure Rust predicates over upstream MusicXML nodes,
MIDI messages, exact library numbers and the existing descriptor map. There is no
second score AST, custom decimal implementation, runtime inference language or
new third-party profile. The finite term dictionary is not a general prose parser.

## Defenses, reverse projection and attempted defeat

The rule numbers below correspond to the catalog and README. Some catalog labels
name capabilities, rather than a native automatic inference; that distinction
matters. “Named” always means an explicit semantic request in the documented
extension vocabulary. Inverse notation is generated only when its extra semantics
are justified; otherwise a standard `other-technical`/`other-play` carrier retains
the exact named request.

| Rule | Source → target; musical defense | Defeater and reverse consequence |
| --- | --- | --- |
| N0 | Unmarked note → ordinary sustain/strike, 10/0. M2-123 §5.2.1 includes naturally decaying percussion. | A short duration or naturally short drum does not establish deliberately choked sound. No synthetic staccato. |
| N1 | Slur → 10/2 legato request, unless a compatible short/other production takes priority. | A phrase slur need not prescribe exact overlap or one breath/bow. No invented overlap percentage; standalone inverse cannot invent original slur endpoints. |
| N2 | Glissando/slide → 10/4 sound request. Both standards describe connected pitch motion. | A tag does not encode an exact trajectory. Source endpoints/layout survive in companion; a standalone sound request stays an extension, not invented paired endpoints. |
| N3 | Strong-accent → 10/6 as a declared marcato representative; ordinary horizontal accent affects attack emphasis. | Vertical mark alone does not prove a sustained loud tail. An early release supplies gate timing, not proof of a different marcato attack; no universal duration cutoff. |
| N4 | Staccato → default normal-short 11/0. | A dot does not prove off-string mechanics. Off-string is the selected default name, not a recovered historical bowing fact. Explicit on-string requests remain 11/1. Inverse dot uses the same convention. |
| N5 | Slur plus staccato → 11/2 slurred-staccato representative. | Outside strings, do not claim a physical single bow. Preserve musical connection independently; profile permits analogous use across instruments. |
| N6 | Accent plus staccato → 11/3 combines emphasis and separation. | Neither component is applied a second time by the sampled-articulation selector. Explicit numeric attack overrides the default accent increment. |
| N7 | Staccatissimo → 11/4. | No universal fraction of written value is encoded by either standard. Receiver articulation response is distinct from nominal Note Off time. |
| N8 | MusicXML spiccato stroke mark → 11/5 as a declared musical reading. | MusicXML defines a stroke-shaped mark, not guaranteed modern bouncing technique. Do not derive it from shortness alone. |
| N9 | Detached-legato or dot+tenuto → 11/9 portato representative. | Portato/louré usage varies; no exact duration inferred. Explicit long/mezzo-staccato term uses 11/8 instead. |
| N10–11 | Pizzicato=yes, snap-pizzicato and corresponding SMuFL → 11/A or 11/B. Snap is a stronger production request. | Generic pizzicato does not imply snap. Left-hand glyph requires both plucking and direction 2; direction alone was insufficient and is corrected. |
| N12 | Explicit unmeasured tremolo (normally zero marks) or legacy single/zero → 12/0. Measured strokes → timed repetitions using duration/type/tuplet ratio. | Paired start/stop is checked before zero marks, so a paired tremolo cannot become two single-note rolls. Already expanded notes must not retrigger a sampled repeat. `consume_pattern` records the consumed request. |
| N13–14 | Native double/triple-tongue → preserve the production symbol in the per-note refinement. | Two/three tongue strokes in a technique do not necessarily mean two/three attacks per written note. No automatic 12/4–5 trigger. Explicit named two/three-repeat requests can select those sounds. |
| N15 | Trill: explicit interval/start and timing → explicit alternating pitches; named sample requests retain 13/0–3. | Main/upper/below are distinct. A bare historical period is not permission to override authored start-note. Exact schedule consumes sample trigger. |
| N16 | Named wider two-pitch tremolos → 13/4–D by requested interval. | Paired native two-note tremolo is currently diagnosed, not falsely claimed implemented. A fast written passage is not a sampled tremolo. |
| N17–18 | Natural/artificial harmonic production → 14/0–1. | Base/touching pitch cannot be sounded as the resulting partial. Explicit such pitch roles need a realization and receive E_HARMONIC_PITCH; an unspecified harmonic kind stays a residual qualifier. Inverse uses sounding-pitch only for the sounding pitch it actually has. |
| N19–22 | Falloff/plop/doit/scoop → corresponding fall/rise at end/start, 15/0–3. | Principal note is the correct start/destination; no invented pitch span/rate. Note On ending gesture is not duplicated by an invented Note Off gesture. |
| N23 | Explicit grace notes → authored pitches and allocated time, not a sampled grace layered on top. | Missing grace timing is a documented performer choice. Named grace-from-below/above sounds remain 15/6–7; no invented second attack. |
| N24 | Named brass-style shake → 15/8. Native MusicXML shake uses its ornament playback attributes and is expanded. | Identical English names are insufficient: MusicXML shake has a specific trill-sound schedule. Counterexample now tests three actual attacks. |
| N25 | Hairpin → timed dynamic interpretation. Named sampled crescendo/etc. → 15/9–D. | A hairpin is not automatically a prerecorded swell. Piano affects subsequent hammer attacks; sustained orchestral controllers require the declared receiver binding. |
| T/P | Exact named standard request → same coordinate, retaining any phrase parameters. | Names do not provide unspecified run extent, trill rate or voicing. No scale/run inferred merely because notes happen to form a scale. |
| G1/G2 | Actual registered jeté/behind-bridge glyph → corresponding sound; retain specified string count. | Behind-bridge sound is not merely ordinary sul ponticello at an endpoint. Unknown glyphs survive for binding. |
| D1 (rejected inference) | Authored attack/release offsets determine exact sounded timing. | Early release alone does not prove staccato, staccatissimo or martellato production: ordinary phrasing may also end early. An attempted automatic short-sample rule was removed after this counterexample. Written brevity or dynamics alone likewise cannot prove bow mechanics. |
| A1 | Stress/accent increases attack emphasis; unstress/soft-accent decreases it. The corpus-v1 performer convention adjusts the nominal level by 12, clamping non-silent attacks to 1–127; native symbols survive as residuals. | No standard prescribes 12. Explicit numerical dynamics defeats this convention; simultaneous stronger/weaker marks are diagnosed. Tenuto alone does not become a louder attack or invented overlap. |
| Dynamics | Numeric note/sound percentage → nominal MIDI-1 value 0.9×percent → M2-115 scaling. Explicit numbers outrank letters. | MusicXML's numeric default forte is 90; Piano's textual f center is 92. They must not be conflated. |
| Piano | Textual dynamic → M2-126 Table 6 center; inverse uses its inclusive ranges and retains exact 16-bit value. | No nearest-letter quantization of transport data. Silent key depression is not Note Off. Electric piano is outside this profile. |
| Release | MusicXML end-dynamics is Note Off velocity percentage, not loudness of the final attack. | M2-123 interprets that field as relative envelope release time; M2-126 as relative damper response. Scaled code values are preserved; equal milliseconds/audio cannot be claimed. |
| Pedals/mutes | Authored sound damper/sostenuto/soft pedals → CC64/66/67. Piano una corda/tre corde words → CC67 on/off. Explicit piano play/mute on/off/practice → M2-126 RC41/20. | Piano `con sordino` can have historical damper meanings; it is retained as text rather than equated with a muffler or una corda. Orchestral mute identity/amount remains separate. Explicit brass mute names and continuous muffler amounts cannot silently become a binary piano mute. |
| Position | Explicit sul ponticello/tasto → declared representative position value and literal request. | These words identify a region, not a canonical numeric point. The exact representative values are conventions, not M2-123 requirements. |
| Timing | Fermata lengthens, breath separates, caesura inserts silence; scope uses shared score clock. | Fixed 3/2, gap and grace defaults are reproducible renderer choices, not universal musical truths. Simultaneous fermatas do not add once per part. Original notation survives in source-aware inverse. |
| Remarks | Direction words/rehearsal/editorial notes → standard Flex text plus declared role/scope convention. Note remarks/other-* → hash-bound note descriptor. Note voice/staff membership accompanies scoped directions, so the address retains its target. | Prose is not silently executable. Native voice/staff are not native UMP fields. Canonical inverse retains native text and exact carrier; companion preserves original typography. |

## Complete named articulation meanings

The following glossary states what each request is asking the receiver to do,
without pretending that a name standardizes its acoustic implementation. Every
row is accepted as a semantic ID or canonical term in the existing catalog.
The executable tests check both directions for all rows, and separately repeat
reachability over every Appendix A instrument column. These checks establish
coverage and encoding, not that a physical instrument can perform every request.


### Classification 0x10 — M2-123 §5.2.1

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `10/0` `core.normal` | Instrument's ordinary sustain or natural strike; no universal vibrato assumption. |
| `10/1` `core.normal-extra` | Second bank of the same ordinary sound, providing more variation slots; not a new technique. |
| `10/2` `core.legato` | Connected transitions between notes; a slur supplies phrasing evidence but no universal overlap time. |
| `10/3` `core.molto-legato` | More continuous/lyrical connection than ordinary legato; not a measured slide rate. |
| `10/4` `core.glissando` | Continuous connecting pitch motion, commonly called string portamento by sample libraries. |
| `10/5` `core.detache` | Separately articulated bow strokes; detached bowing does not by itself mean staccato length. |
| `10/6` `core.marcato` | Marked accented sustained sound; distinct from a brief attack accent or a clipped stroke. |
| `10/7` `core.martele` | Hammered, clearly articulated string attack; not an inference from loudness alone. |
| `10/8` `core.senza-vibrato` | Explicitly suppress vibrato; ordinary sound may already have none. |
| `10/9` `core.con-vibrato` | Explicitly request vibrato; does not specify rate, width or synchrony. |
| `10/A` `core.synchronized-vibrato` | Vibrato coordinated in its timing; greater vibrato amount is insufficient evidence. |

### Classification 0x11 — M2-123 §5.2.2

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `11/0` `short.staccato-off` | Ordinary short articulation; chosen default is off-string for strings, ordinary staccato elsewhere. |
| `11/1` `short.staccato-on` | Ordinary short articulation with the bow remaining on the string when applicable. |
| `11/2` `short.slurred-staccato` | Separated notes within a slurred/single-bow gesture; connection and separation coexist. |
| `11/3` `short.accented-staccato` | Separated short notes with attack emphasis. |
| `11/4` `short.staccatissimo` | Very detached/shortened sound; no standardized gate fraction. |
| `11/5` `short.spiccato` | Distinct bouncing-bow short stroke on strings, analogous short attack elsewhere. |
| `11/6` `short.sautille` | Rapid springing-bow stroke; fast note duration alone does not establish its mechanics. |
| `11/7` `short.martellato` | Shortened marked/hammered attack, contrasted with a sustained marcato sound. |
| `11/8` `short.long-staccato` | Relatively long short strokes, including mezzo-staccato/louré usage. |
| `11/9` `short.portato` | Gently separated connected articulation; string portato or analogous light tonguing. |
| `11/A` `short.pizzicato` | String plucked rather than ordinarily bowed; exact hand is separate. |
| `11/B` `short.snap-pizzicato` | Forceful snapped/plucked attack, or analogous harsh noisy attack on another family. |
| `11/C` `short.col-legno-battuto` | String struck using the wood of the bow; percussive production, not sustained wood bowing. |
| `11/D` `short.col-legno-gestrichen` | The standard's named short col-legno-gestrichen request; do not conflate with sustained tratto. |
| `11/E` `short.string-hand-tap` | Pitched string-hand tap; unlike a generic body-tap noise. |
| `11/F` `short.jete` | Thrown/rebounding bow gesture; bounce count and timing are not encoded by the name alone. |

### Classification 0x12 — M2-123 §5.2.3

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `12/0` `repeat.tremolo` | Rapid same-pitch reiteration: string tremolo, flutter tonguing, percussion roll, or analogous technique. |
| `12/1` `repeat.growl` | Growled/razz production; distinct from simply flutter tonguing or playing louder. |
| `12/2` `repeat.coloristic-tremolo` | Alternative color of same-pitch tremolo; retain the specified color as a refinement. |
| `12/3` `repeat.timbral-trill` | Timbral alternation on one pitch, such as alternate fingerings or equivalent string timbre changes. |
| `12/4` `repeat.two` | A sound containing a two-repeat gesture; not every double-tongue marking proves this trigger. |
| `12/5` `repeat.three` | A sound containing a three-repeat gesture; native triple-tonguing need not triple written attacks. |
| `12/6` `repeat.four` | A sound containing a four-repeat gesture. |
| `12/7` `repeat.five` | A sound containing a five-repeat gesture. |
| `12/8` `repeat.six` | A sound containing a six-repeat gesture. |
| `12/9` `repeat.fast` | Faster reiteration variant; no universal beats-per-second value follows. |

### Classification 0x13 — M2-123 §5.2.4

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `13/0` `trill.semitone-main-first` | Alternation a semitone above the principal, principal first. |
| `13/1` `trill.semitone-upper-first` | Alternation a semitone above the principal, upper auxiliary first. |
| `13/2` `trill.tone-main-first` | Alternation a whole tone above the principal, principal first. |
| `13/3` `trill.tone-upper-first` | Alternation a whole tone above the principal, upper auxiliary first. |
| `13/4` `trill.minor-third` | Alternation separated by three semitones; starting member is not prescribed here. |
| `13/5` `trill.major-third` | Alternation separated by four semitones; starting member remains an independent constraint. |
| `13/6` `trill.fourth` | Alternation separated by five semitones. |
| `13/7` `trill.tritone` | Alternation separated by six semitones. |
| `13/8` `trill.fifth` | Alternation separated by seven semitones. |
| `13/9` `trill.minor-sixth` | Alternation separated by eight semitones. |
| `13/A` `trill.major-sixth` | Alternation separated by nine semitones. |
| `13/B` `trill.minor-seventh` | Alternation separated by ten semitones. |
| `13/C` `trill.major-seventh` | Alternation separated by eleven semitones. |
| `13/D` `trill.octave` | Alternation separated by twelve semitones; not two independently sustained octave notes. |

### Classification 0x14 — M2-123 §5.2.5

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `14/0` `color.natural-harmonic` | Natural harmonic production, involving a harmonic of the open string/air column as applicable; sounding pitch still needs resolution. |
| `14/1` `color.artificial-harmonic` | Artificial harmonic production using a stopped/fingered fundamental where applicable; touching pitch is not the sounded partial. |
| `14/2` `color.col-legno-tratto` | Sustained bowing using the wood side of the bow, unlike battuto striking. |
| `14/3` `color.flautando` | Flute-like softened string color; bow-hair/contact details are receiver implementation choices. |
| `14/4` `color.polyphonic-octaves` | One triggered sound containing octave layers; not ordinary explicit octave polyphony. |
| `14/5` `color.polyphonic-chord` | One triggered sound containing intervals/chords; chord voicing is an independent parameter. |
| `14/6` `color.cuivre` | Brassy/metallic tone color, especially cuivré horn; not an automatic volume or instrument replacement. |
| `14/7` `color.lontano` | Distant-sounding performance/color; not a standardized physical distance or reverb value. |
| `14/8` `color.singing` | Singing while exciting/playing the instrument, yielding combined timbre; not an ordinary choir patch. |

### Classification 0x15 — M2-123 §5.2.6

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `15/0` `gesture.fall-end` | Begin at the stated note pitch, then fall away at its end. |
| `15/1` `gesture.fall-start` | Begin above and fall into the stated note pitch. |
| `15/2` `gesture.rise-end` | Begin at the stated note pitch, then rise away at its end. |
| `15/3` `gesture.rise-start` | Begin below and rise into the stated note pitch. |
| `15/4` `gesture.blue-down` | Depart downward from the stated pitch and return to it. |
| `15/5` `gesture.blue-up` | Depart upward from the stated pitch and return to it. |
| `15/6` `gesture.grace-below` | Leading ornamental/grace gesture arriving from below; do not duplicate explicit grace notes. |
| `15/7` `gesture.grace-above` | Leading ornamental/grace gesture arriving from above; historical label does not override authored pitches. |
| `15/8` `gesture.shake` | Shaking pitch gesture such as a brass shake; not automatically MusicXML's trill-sound shake ornament. |
| `15/9` `gesture.crescendo` | A selected sound with an internal increase in dynamics. |
| `15/A` `gesture.decrescendo` | A selected sound with an internal decrease in dynamics. |
| `15/B` `gesture.swell` | A selected sound that grows and then fades. |
| `15/C` `gesture.reverse-swell` | A selected sound that fades and then grows. |
| `15/D` `gesture.sfz-crescendo` | Emphatic sforzando attack followed by crescendo; not interchangeable with a simple sfz transient. |

### Classification 0x16 — M2-123 §5.2.7

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `16/0` `run.playable` | Sound designed for explicitly played fast runs; does not itself prove or specify a recorded scale. |
| `16/1` `run.playable-tremolo` | Sound designed for explicitly played tremolo sequences. |
| `16/2` `run.playable-glissando` | Sound designed for explicitly played glissando sequences. |
| `16/3` `run.major` | Triggered major-scale/run sound; extent, rate and direction remain additional information. |
| `16/4` `run.minor` | Triggered minor-scale/run sound; the minor form is not fully specified by this code. |
| `16/5` `run.dominant-seventh` | Triggered dominant-seventh scale/run sound; do not infer it from one dominant chord. |
| `16/6` `run.diminished-whole-half` | Triggered diminished whole-half collection or the standard's library-specific Other 1 alternative. |
| `16/7` `run.diminished-half-whole` | Triggered diminished half-whole collection or the standard's library-specific Other 2 alternative. |
| `16/8` `run.whole-tone-c` | Triggered whole-tone collection containing C, one of the two transposition classes. |
| `16/9` `run.whole-tone-c-sharp` | Triggered whole-tone collection containing C-sharp, the other transposition class. |
| `16/A` `run.pentatonic-major` | Triggered major-pentatonic collection/run. |
| `16/B` `run.pentatonic-minor` | Triggered minor-pentatonic collection/run. |
| `16/C` `run.lydian` | Triggered Lydian collection/run, characterized by raised fourth relative to major. |
| `16/D` `run.lydian-flat-seven` | Triggered Lydian-flat-seven collection/run, retaining the raised fourth with lowered seventh. |
| `16/E` `run.chromatic` | Triggered chromatic collection/run. |
| `16/F` `run.other` | Other specified scale/run sound; the code does not identify its collection without refinement. |

### Classification 0x17 — M2-123 §5.2.8

| Coordinate / semantic ID | Musical meaning and limit |
| --- | --- |
| `17/0` `effect.sustained-noise` | Sustained noise/air or unconventional effect; not automatically a pitched ordinary sustain. |
| `17/1` `effect.behind-bridge` | Sustained bowing behind the bridge; distinct contact region from ordinary bridge-side bowing. |
| `17/2` `effect.random-pizzicato` | An extended irregular/random-pitch pizzicato texture; not one random plucked attack. |
| `17/3` `effect.harmonic-glissando` | Harmonic gliding effect; ordinary continuous stopped-pitch glissando is not sufficient evidence. |
| `17/4` `effect.random-glissando` | Irregular/random glissando texture; not a single explicitly measured slide. |
| `17/5` `effect.short-air` | Short air/chiff/squeak/lip-pizz-type effect; distinct from sustained air noise. |
| `17/6` `effect.mechanical-long` | Continuous/extended mechanical noise texture. |
| `17/7` `effect.mechanical-short` | Short key/valve/other mechanical action noise. |
| `17/8` `effect.finger-glide` | Audible finger-gliding noise; distinguish from an intended pitched slide. |
| `17/9` `effect.fret-noise` | Fret/contact noise; not a separately chosen melodic pitch. |
| `17/A` `effect.thump` | Dull thumping impact. |
| `17/B` `effect.knock` | Knocking impact. |
| `17/C` `effect.slap` | Slapping impact on the instrument/body or equivalent effect. |
| `17/D` `effect.pop` | Popping impact/attack such as an appropriate mouthpiece effect. |
| `17/E` `effect.tap` | Tapped body/contact noise; distinct from the pitched string-hand-tap subclass. |
| `17/F` `effect.click` | Short bright clicking noise. |

## Review record and completion limit

The adversarial round inspects source semantics, target semantics, directionality,
missing evidence, inverse implications and independent examples for every rule
family above. The glossary accounts for every non-reserved named coordinate.
`interpretation.rs` supplies negative and boundary cases in addition to the older
musical/transport suite. In particular it defeated the first left-hand-pizzicato
implementation (direction without plucking), the lower-start ornament branch,
and the assumption that a MusicXML shake was merely a sampled brass shake. An attempted early-release-to-short-production inference was rejected: exact gate timing survives without inventing a sample-production change.
These are corrected in code. A rest-only Piano case additionally ensures profile declarations survive independently of note attacks. Repeated freehand notation retains its original source ID in provenance without duplicating XML IDs on canonical elements. Whitespace introduced by the upstream XML writer
also exposed an editorial-remark round-trip defect; lexical text nodes no longer
become musical remark entries.

Remaining distinctions are explicit limits, not proofs of completion: a symbolic
name cannot certify a physical technique, library support, sample duration or
acoustic result. Accelerated/terminal ornament schedules, paired native tremolo,
and explicit harmonic base/touching pitch realization need further performance
binding and currently produce diagnostics. The finite theorem includes complete
**named M2-123 vocabulary**, standard Piano conversion, the selected corpus and
specified counterexamples; it does not claim complete interpretation of every
legal MusicXML performance instruction or universal orchestration expertise.

A newly found unsupported instruction or contradictory musical example is a new
counterexample to the relevant broader claim. Record it, narrow an unjustified
rule or extend the implementation and rerun calibration. Never turn an omitted
audible effect into a pass by calling the source archive its performance.
