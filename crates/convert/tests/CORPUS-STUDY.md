# Twelve-score conversion study

The user requested a diversified sample of twelve, replacing routine conversion
of the entire corpus. All 535 MXL containers were surveyed before selection.
[corpus-sample.json](corpus-sample.json) pins the music21 revision, exact hashes,
selection rationale, instrument labels and source-feature counts. Scores were
selected for musical coverage, including piano, voice, transposing wind, quartet,
large orchestration and the largest/most intricate material. Note-element counts
include rests; they must not be mistaken for the number of sounded attacks.

The executable cases independently XSD-validate **both** inverse modes and compare
actual decoded performances. [acceptance.json](acceptance.json) records current
attack counts, exact duration, actual wire Attribute Types, semantic coordinates,
Piano channels and declared profiles per score. Semantic coordinate 10/0 in the
internal normalized model does not mean every Piano packet carries Attribute 10:
ordinary Piano uses Attribute 00 and its Piano Profile declaration.

## Each score, both directions

“Canonical inverse” means interpreting actual MIDI performance without restoring
archived source. A matching companion permits source-aware reconstruction only
after the source has been rendered again and compared with the clip. Canonical
notation is not claimed to recover original barlines, note spelling or layout.

| Selected score | MusicXML → MIDI performance | MIDI performance → MusicXML |
| --- | --- | --- |
| J. S. Bach, BWV846 | Piano identity selects M2-126. Voice timing and tied sound lifecycles are preserved; fermatas use the declared shared-clock interpretation. Ordinary notes use Piano hammer dynamics. | Piano profile/description restores Piano identity and dynamic range labels. Canonical notes express played durations; a companion restores original ties, voices and fermata notation. |
| C. P. E. Bach, H186 | Acoustic Grand Piano instrument label supplies the older export's missing sound ID. Staccatissimo selects 11/4 with compatible Orchestral Profile; slurred connection uses 10/2. Turns, mordents, trills and grace use resolved attacks rather than duplicate sampled patterns. | Piano identity survives; 11/4 becomes native staccatissimo, exact velocities remain. Expanded ornaments remain actual played notes plus provenance; source-aware mode restores the original ornamental signs. |
| Joplin, Maple Leaf Rag | Piano Profile; chords, syncopation, ties and written repeat navigation determine attacks and timing. No short bow technique inferred from rhythmic brevity. | Canonical performance retains syncopation, polyphony and repeat-expanded attacks. Companion restores repeat signs and written tie structure. |
| Schoenberg, Op.19/2 | Piano plus applicable articulation profile. Staccato and slur context combine; dynamic wedges change future hammer attacks. Tenuto remains independent intent. | Native staccato where justified, Piano dynamic labels plus exact attacks. Canonical inverse does not turn hammer dynamics into a sustained gain swell; source-aware mode restores original wedges and engraving. |
| Robert Schumann, Op.48/2 | Separate voice and Piano routes; staccato, accent, slur, pedal, fermata and dynamic scopes apply to their source parts. Voice is not inferred to be piano merely from a numeric program. | Voice/Piano identity and routing retained. Canonical notes/controllers/text preserve performance; matching companion restores lyrics/layout and exact native pedal/wedge/phrase notation. |
| Weber, Clarinet Concertino | Clarinet and Piano remain distinct; instrument IDs and transposition affect sounding pitch. Staccato/accent/slur, ornaments, grace and hairpins provide interacting evidence. | Sounding pitches, routing and standard articulation requests survive. Canonical score does not pretend to recover the original transposing spelling. Companion restores written clarinet transposition and original piano/solo engraving. |
| Beethoven, Op.132 | Quartet; **20,361 written note elements**, largest in the survey. Numbered slur/trill spans, grace, ties, wedges, repeats and pizzicato directions determine the performance. | Standard requests reverse to native signs where unambiguous, otherwise named extensions. Played timing and pattern provenance survive. Companion restores the large written form, phrase spans and repeat notation. |
| Beethoven, Op.133 | Quartet; dense fugue texture, 139 trill marks, 172 grace notes, ties, fermatas and harmonic indication exercise combination and note binding. | Explicit performed ornaments remain notes; harmonic/specific sound requests retain identity rather than an invented fingering. Companion recovers the source signs and written fugue structure. |
| Mozart, K458/I | Quartet; 466 written staccato marks, numbered trill/wavy spans, slurs, grace and repeats distinguish separated, connected and ornamented attacks. | Native staccato for ordinary-short requests; slur/trill sample labels cannot invent original span endpoints. Performance equality and source-aware notation restoration are separately checked. |
| J. S. Bach, BWV267 | Four-part **chorale**, retained as the previous CLI/independent oracle. 330 written pitched notes, five sounding tie stops, and 77 repeated attacks give **402** attacks. | Independent clip events reconstruct four routes with the actual pitches and timing. Companion restores written repeats/ties. This score is not presented as the orchestral case study. |
| J. S. Bach, BWV29.8 | Trumpets, SATB and timpani. Mordents, shared fermatas, sound ties and repeat navigation produce the detailed trace below. No violin or piano is invented. | Canonical notes reconstruct actual attacks, relative release and text/refinements. Companion restores the original mordent signs, ties, repeat bars and fermatas. |
| J. S. Bach, BWV248.64-6 | Fourteen-part ensemble: trumpets, timpani, oboes, strings, SATB and continuo. Large routing/polyphonic/tie case; sparse explicit short articulations remain sparse. | Instrument descriptions and programs remain separate facts. Canonical performance preserves ensemble routing and attacks. Companion restores original instrumental/choral notation and tie chains. |

The sample includes no invalid source documents. Independent negative tests still
cover the corpus's observed negative dynamics and out-of-range one-based program
values; invalid files are not quietly accepted because the sample is smaller.

## BWV29.8: an actual orchestral case

The source part list says Trumpet1, Trumpet2, Trumpet3, Soprano, Alto, Tenor, Bass,
Timpani. It contains **676 note elements: 515 pitched notes and 161 rests**.
There are six sound-tie stops, three inverted-mordent signs, 56 fermata signs and
one backward-repeat boundary printed in all eight parts. Those repeated declarations
refer to one shared structural boundary, not eight nested repetitions.

Program values are preserved. A program number alone does not establish the sound
set; a value 1 in an export is not sufficient evidence to reclassify Trumpet as
acoustic Piano. Instrument identity survives in standard metadata carriers using
our declared identity convention, separately from numeric Program Change.

### Trumpet2, written measure 3, second note

The archived source has divisions 10080 and this essential content:

```xml
<note>
  <pitch><step>E</step><octave>5</octave></pitch>
  <duration>20160</duration>
  <type>half</type>
  <notations><ornaments><inverted-mordent/></ornaments></notations>
</note>
```

This is a two-quarter-note span. MusicXML distinguishes the mordent sign with a
vertical line from the inverted-mordent without it; SMuFL naming differs, so
matching the English word alone is unsafe. The pinned XSD defaults for this
ornament supply main-note start, three attacks, whole-tone auxiliary, and landing
percentages 12% and 24%. These are MusicXML playback defaults, not a claim about
all historical Bach interpretation. Explicit attributes/accidentals override them.

The resulting pitches are **E5–F♯5–E5**, MIDI pitches **76–78–76**.
In this performed occurrence the first onset is quarter position 8:

| Attack | Pitch | Onset | End | MIDI2 Note On words (hex) |
| --- | --- | --- | --- | --- |
| 1 | E5 / 76 | 8 | 206/25 | `40914C10 90410000` |
| 2 | F♯5 / 78 | 206/25 | 212/25 | `40914E10 90410000` |
| 3 | E5 / 76 | 212/25 | 10 | `40914C10 90410000` |

The first word addresses group0/channel1, with Attribute10. The attribute data
selects ordinary sustain/strike 10/0, because the ornament has already been
realized as attacks. The velocity is the declared unmarked orchestral default
72, scaled to 9041 by M2-115. These performance messages are **not MIDI-CI
transactions**. The file's MIDI-CI profile declaration uses the real Orchestral
Profile ID `[7E,21,01,01,01]`; a live receiver must negotiate that profile before
using its articulation semantics.

Each attack retains the `expanded-pattern` refinement: inverted-mordent, attack
index, count3, logical duration2. Asking a receiver for a sampled intervallic
trill as well would duplicate the performance. M2-123 does have intervallic trills,
but it does not define a dedicated “this already-expanded mordent” subclass.

Reverse conversion decodes the attacks first. Canonical MusicXML contains their
exact pitches/times and the explicit provenance/playback carrier; it does not
invent proof that every E–F♯–E pattern was originally notated with a mordent sign.
With a matching companion, re-rendering verifies the original source before
restoring its one half note and inverted-mordent sign. Edited attacks invalidate
that restoration and the inverse follows the edited performance.

### Ties, fermatas and repeats

A Trumpet1 A5 quarter in written measure31 ties into an A5 eighth in measure32.
The MusicXML `tie` elements declare a continuous **sound**; `tied` records the
printed tie. The performance has one attack and one ending, with the appropriate
combined duration. It is not a repeated-note articulation. The canonical inverse
can write one played duration; source-aware inverse restores the written tie chain.

Fermatas lengthen the shared clock. The reproducible corpus-v1 performer choice
uses factor3/2; MusicXML does not prescribe that number. Simultaneous marks across
parts do not multiply the hold eight times. The inverse retains the played
lengthening; only source evidence justifies reinstating the original fermata
symbols and written durations.

The repeated opening contributes 128 additional pitched attacks before ornament
expansion. The three written mordents occur five times after repeats, adding ten
attacks. The independent total is therefore:

```text
515 pitched notes - 6 sounding tie stops + 128 repeated attacks
+ 5 ornament occurrences × 2 additional attacks = 647 performed attacks.
```

No explicit staccato/spiccato/bowing instruction appears in this score. There is
no musically defensible reason to invent one merely because a trumpet/timpani note
is short. Broader orchestral subclass coverage comes from the independent named
and adversarial tests, not an embellished description of this Bach source.

## Remarks and granular library distinctions

The focused tests include note-level `footnote`, unknown `other-technical`,
`other-notation` and `other-ornament`, plus a direction restricted to voice2 in the
second instrumental part. Those tests reconstruct from decoded MIDI data without
archived-source restoration. The text remains visible in the correct MusicXML
part and its note/voice scope remains in the declared carrier. Freehand comments
are candidates for explicit user regex/library mappings, not automatically executable
musical commands. A composer can therefore retain a comment such as “irregular
glass shimmer” without the converter falsely certifying a particular library patch.
