use musicxml_midi2_convert::{
    self as convert, clip,
    model::{Rational, RenderingPolicy, Route},
    rules::Articulation,
    xml,
};
use rstest::rstest;
use std::fmt::Write as _;

fn score(body: &str) -> Vec<u8> {
    format!("<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>Violin</part-name></score-part></part-list><part id=\"P1\">{body}</part></score-partwise>").into_bytes()
}
fn note(step: &str, extra: &str) -> String {
    format!(
        "<note><pitch><step>{step}</step><octave>4</octave></pitch><duration>1</duration>{extra}</note>"
    )
}
fn parsed(body: &str) -> convert::model::Performance {
    xml::read(&score(body), &RenderingPolicy::default())
        .unwrap()
        .1
}

#[test]
fn independent_bwv267_facts() {
    // Independently inspected XML: 330 pitched written notes, 5 tie stops,
    // four backward-repeat declarations at written measure 4 (one per part).
    // The repeated pickup-through-4 adds 21+17+22+17=77 attacks; 330-5+77=402.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../music21/music21/corpus/bach/bwv267.mxl");
    let (_, s) = xml::read(&std::fs::read(root).unwrap(), &RenderingPolicy::default()).unwrap();
    assert_eq!(s.notes.len(), 402);
    let first = s
        .notes
        .iter()
        .filter(|n| n.onset == Rational::from_integer(0))
        .map(|n| (n.route.channel, n.pitch))
        .collect::<Vec<_>>();
    assert_eq!(
        first,
        [
            (0, 74.into()),
            (1, 67.into()),
            (2, 59.into()),
            (3, 55.into())
        ]
    );
}

#[test]
fn first_and_second_endings_are_not_dropped() {
    let body = format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes><barline location=\"left\"><repeat direction=\"forward\"/></barline>{}</measure><measure number=\"2\"><barline location=\"left\"><ending number=\"1\" type=\"start\"/></barline>{}<barline><ending number=\"1\" type=\"stop\"/><repeat direction=\"backward\"/></barline></measure><measure number=\"3\"><barline location=\"left\"><ending number=\"2\" type=\"start\"/></barline>{}<barline><ending number=\"2\" type=\"stop\"/></barline></measure>",
        note("C", ""),
        note("D", ""),
        note("E", "")
    );
    let s = parsed(&body);
    assert_eq!(
        s.notes
            .iter()
            .map(|n| n.pitch.to_integer())
            .collect::<Vec<_>>(),
        [60, 62, 60, 64]
    );
}

#[test]
fn time_only_filters_repeat_occurrences() {
    let n = note("C", "").replace("<note>", "<note time-only=\"2\">");
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{n}<barline><repeat direction=\"backward\"/></barline></measure>"
    ));
    assert_eq!(s.notes.len(), 1);
    assert_eq!(s.notes[0].onset, Rational::from_integer(1));
}

#[test]
fn sounding_ties_have_one_lifecycle() {
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}{}</measure>",
        note("C", "<tie type=\"start\"/>"),
        note("C", "<tie type=\"stop\"/>")
    ));
    assert_eq!(s.notes.len(), 1);
    assert_eq!(s.notes[0].end, Rational::from_integer(2));
}

#[test]
fn trill_defaults_are_an_independent_timed_oracle() {
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}</measure>",
        note(
            "C",
            "<notations><ornaments><trill-mark/></ornaments></notations>"
        )
    ));
    assert_eq!(
        s.notes
            .iter()
            .map(|n| n.pitch.to_integer())
            .collect::<Vec<_>>(),
        [62, 60, 62, 60]
    );
    assert_eq!(
        s.notes.iter().map(|n| n.onset).collect::<Vec<_>>(),
        [
            Rational::new(0, 1),
            Rational::new(1, 4),
            Rational::new(1, 2),
            Rational::new(3, 4)
        ]
    );
    assert!(
        s.notes
            .iter()
            .all(|n| n.articulation == Articulation::default())
    );
}

#[rstest]
#[case(
    "recorded hotel mute",
    "mute-identity",
    "urn:orchestraltools:berlin:hotel-mute"
)]
#[case("harmonic staccato", "production", "harmonic")]
#[case("expressive legato", "connection-style", "expressive")]
#[case("cinematic legato", "connection-style", "cinematic")]
#[case("evolving vibrato", "vibrato-envelope", "evolving")]
#[case("spiccato blurred", "sample-character", "blurred")]
fn berlin_conventions_survive(#[case] term: &str, #[case] key: &str, #[case] value: &str) {
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}</measure>",
        note(
            "C",
            &format!(
                "<notations><technical><other-technical>{term}</other-technical></technical></notations>"
            )
        )
    ));
    let (encoded, dictionary, _) = clip::encode(&s).unwrap();
    let decoded = clip::decode(&encoded, Some(&dictionary)).unwrap();
    assert_eq!(
        decoded.notes[0].descriptor.as_ref().unwrap().qualifiers
            [&format!("urn:midi2workbench:{key}")],
        value
    );
    let inverse = xml::write(&decoded).unwrap();
    let (_, back) = xml::read(inverse.as_bytes(), &RenderingPolicy::default()).unwrap();
    let (encoded, d, _) = clip::encode(&back).unwrap();
    assert_eq!(
        decoded.semantic_values(),
        clip::decode(&encoded, Some(&d)).unwrap().semantic_values()
    );
}

#[test]
fn mute_collisions_allocate_separate_channels() {
    let input = format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}<backup><duration>1</duration></backup>{}</measure>",
        note("C", "<play><mute>cup</mute></play>"),
        note("E", "<play><mute>straight</mute></play>")
    );
    let s = parsed(&input);
    let (encoded, map, _) = clip::encode(&s).unwrap();
    let d = clip::decode(&encoded, Some(&map)).unwrap();
    assert_ne!(d.notes[0].route, d.notes[1].route);
}

#[test]
fn same_pitch_overlap_and_position_do_not_retag_a_neighbor() {
    let mut s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}{}</measure>",
        note("C", ""),
        note("C", "")
    ));
    s.notes[0].end = Rational::from_integer(2);
    s.notes[1].onset = Rational::new(1, 2);
    for (n, position) in s.notes.iter_mut().zip([0x0800_0000, 0xf000_0000_u32]) {
        n.descriptor = Some(convert::model::Descriptor {
            technique: "core.normal".into(),
            qualifiers: std::collections::BTreeMap::from([(
                "urn:midi2workbench:position".into(),
                serde_json::json!(position),
            )]),
        });
    }
    let (bytes, map, _) = clip::encode(&s).unwrap();
    let d = clip::decode(&bytes, Some(&map)).unwrap();
    assert_ne!(d.notes[0].route, d.notes[1].route);
    assert_ne!(d.notes[0].descriptor, d.notes[1].descriptor);
}

#[test]
fn independently_authored_clip_has_a_real_inverse() {
    let words = [
        0x0040_0000,
        0x0030_01e0,
        0x0040_0000,
        0xf020_0000,
        0,
        0,
        0,
        0x0040_0000,
        0xd010_0000,
        50_000_000,
        0,
        0,
        0x0040_0000,
        0x4090_3c11,
        0x1234_5000,
        0x0040_01e0,
        0x4080_3c10,
        0x4321_0000,
        0x0040_0000,
        0xf021_0000,
        0,
        0,
        0,
    ];
    let mut input = b"SMF2CLIP".to_vec();
    for word in words {
        input.extend(u32::to_be_bytes(word));
    }
    let expected = clip::decode(&input, None).unwrap();
    assert_eq!(expected.notes[0].velocity, 0x1234);
    assert_eq!(expected.notes[0].articulation.subclass, 5);
    let inverse = convert::midi2_to_xml(&input, None, false).unwrap();
    let (_, back) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let (bytes, map, _) = clip::encode(&back).unwrap();
    assert_eq!(
        expected.semantic_values(),
        clip::decode(&bytes, Some(&map)).unwrap().semantic_values()
    );
}

#[test]
fn editing_the_clip_defeats_stale_source_restoration() {
    let bytes = score(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}</measure>",
        note("C", "")
    ));
    let mut c = convert::xml_to_midi2(&bytes, &RenderingPolicy::default()).unwrap();
    let at = c
        .clip
        .windows(4)
        .position(|b| b == [0x40, 0x80, 0x3c, 0x10])
        .unwrap();
    c.clip[at - 4..at].copy_from_slice(&0x0040_03c0_u32.to_be_bytes());
    let inverse = convert::midi2_to_xml(&c.clip, Some(&c.companion), false).unwrap();
    let (_, s) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    assert_eq!(s.notes[0].end, Rational::from_integer(2));
}

#[test]
fn every_release_subclass_and_note_modifier_is_validated() {
    for release in 0..5 {
        for direction in 0..3 {
            for string in 0..8 {
                let a = Articulation {
                    at: 0x11,
                    subclass: 5,
                    variation: 15,
                    direction,
                    string,
                    reset_round_robin: true,
                    release,
                };
                let words = convert::wire::note(Route::default(), 60, 0x9000, a, true).unwrap();
                assert_eq!(
                    Articulation::from_data(
                        0x11,
                        u16::from_le_bytes([words[1].to_le_bytes()[0], words[1].to_le_bytes()[1]])
                    )
                    .unwrap()
                    .data()
                    .unwrap(),
                    a.data().unwrap()
                );
                convert::wire::validate(
                    &convert::wire::note(Route::default(), 60, 0, a, false).unwrap(),
                )
                .unwrap();
            }
        }
    }
    assert!(
        Articulation {
            direction: 3,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn crescendo_is_a_timed_dynamic_layer_curve_and_survives_edits() {
    let body = "<measure number=\"1\"><attributes><divisions>1</divisions></attributes><direction><direction-type><dynamics><p/></dynamics></direction-type></direction><direction><direction-type><wedge type=\"crescendo\"/></direction-type></direction><note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note><direction><direction-type><wedge type=\"stop\"/></direction-type></direction><direction><direction-type><dynamics><f/></dynamics></direction-type></direction></measure>";
    let source = score(body);
    let mut c = convert::xml_to_midi2(&source, &RenderingPolicy::default()).unwrap();
    let d = clip::decode(&c.clip, Some(&c.companion.dictionary)).unwrap();
    let curve = convert::extension::dynamic_curve(d.notes[0].descriptor.as_ref().unwrap()).unwrap();
    assert_eq!(curve.len(), 385);
    assert_eq!(curve[0].1, convert::rules::upscale(45, 7, 32));
    assert_eq!(curve.last().unwrap().1, convert::rules::upscale(90, 7, 32));
    assert_eq!(
        c.companion.dictionary.parameters[&126],
        convert::extension::DYNAMIC_LEVEL
    );
    let canonical = xml::write(&d).unwrap();
    let (_, back) = xml::read(canonical.as_bytes(), &RenderingPolicy::default()).unwrap();
    let (bytes, map, _) = clip::encode(&back).unwrap();
    assert_eq!(
        d.semantic_values(),
        clip::decode(&bytes, Some(&map)).unwrap().semantic_values()
    );
    let at = c
        .clip
        .windows(4)
        .position(|b| b == [0x40, 0x10, 0x3c, 0x7e])
        .unwrap();
    c.clip[at + 4..at + 8].copy_from_slice(&u32::MAX.to_be_bytes());
    let inverse = convert::midi2_to_xml(&c.clip, Some(&c.companion), false).unwrap();
    let (_, edited) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let actual =
        convert::extension::dynamic_curve(edited.notes[0].descriptor.as_ref().unwrap()).unwrap();
    assert_eq!(actual[0].1, u32::MAX);
}

#[rstest]
#[case::da_capo("<sound fine=\"yes\"/>","<sound dacapo=\"yes\"/>", vec![60,62,64,60])]
#[case::dal_segno("<sound segno=\"s\"/>","<sound dalsegno=\"s\"/>", vec![60,62,64,62,64])]
fn navigation_has_independent_attack_expectations(
    #[case] marker: &str,
    #[case] jump: &str,
    #[case] expected: Vec<i64>,
) {
    let body = format!(
        "<measure number=\"1\">{}{marker}</measure><measure number=\"2\">{}</measure><measure number=\"3\">{}{jump}</measure>",
        note("C", ""),
        note("D", ""),
        note("E", "")
    );
    let s = parsed(&body);
    assert_eq!(
        s.notes
            .iter()
            .map(|n| n.pitch.to_integer())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn navigation_to_coda_and_midmeasure_fine() {
    let body = format!(
        "<measure number=\"1\">{}<sound tocoda=\"c\"/>{}<sound fine=\"yes\"/>{}</measure><measure number=\"2\">{}<sound dacapo=\"yes\"/></measure><measure number=\"3\"><sound coda=\"c\"/>{}</measure>",
        note("C", ""),
        note("D", ""),
        note("E", ""),
        note("F", ""),
        note("G", "")
    );
    let s = parsed(&body);
    assert_eq!(
        s.notes
            .iter()
            .map(|n| n.pitch.to_integer())
            .collect::<Vec<_>>(),
        vec![60, 62, 64, 65, 60, 67]
    );
    let fine = format!(
        "<measure number=\"1\">{}<sound fine=\"yes\"/>{}</measure><measure number=\"2\">{}<sound dacapo=\"yes\"/></measure>",
        note("C", ""),
        note("D", ""),
        note("E", "")
    );
    assert_eq!(parsed(&fine).notes.len(), 4);
}

#[rstest]
#[case::following(
    "steal-time-following=\"25\"",
    Rational::new(1, 4),
    Rational::from_integer(2)
)]
#[case::inserted(
    "make-time=\"1\"",
    Rational::from_integer(1),
    Rational::from_integer(3)
)]
fn grace_keeps_articulation_and_explicit_timing(
    #[case] timing: &str,
    #[case] principal_onset: Rational,
    #[case] end: Rational,
) {
    let body = format!(
        "<measure number=\"1\"><note><grace {timing}/><pitch><step>D</step><octave>4</octave></pitch><notations><articulations><staccato/></articulations></notations></note>{}{}</measure>",
        note("C", ""),
        note("E", "")
    );
    let s = parsed(&body);
    assert_eq!(
        s.notes[0].articulation,
        Articulation::named("short.staccato-off").unwrap()
    );
    assert_eq!(s.notes[1].onset, principal_onset);
    assert_eq!(s.notes.last().unwrap().end, end);
    assert_eq!(s.notes[0].onset, Rational::from_integer(0));
}

#[test]
fn bank_bytes_and_simultaneous_controller_order_have_independent_vectors() {
    assert_eq!(
        convert::wire::program(Route::default(), 5, Some(130)),
        vec![0x40c0_0001, 0x0500_0102]
    );
    let words = [
        0x0040_0000,
        0x0030_01e0,
        0x0040_0000,
        0xf020_0000,
        0,
        0,
        0,
        0x0040_0000,
        0x4090_3c10,
        0x9000_0000,
        0x40b0_4000,
        u32::MAX,
        0x0040_01e0,
        0x4080_3c10,
        0x8000_0000,
        0x0040_0000,
        0xf021_0000,
        0,
        0,
        0,
    ];
    let mut input = b"SMF2CLIP".to_vec();
    for w in words {
        input.extend(w.to_be_bytes());
    }
    let expected = clip::decode(&input, None).unwrap();
    assert!(
        expected
            .events
            .iter()
            .any(|e| e.words[0] == 0x40b0_4000 && e.order > 1)
    );
    let inverse = convert::midi2_to_xml(&input, None, false).unwrap();
    let (_, s) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let (output, map, _) = clip::encode(&s).unwrap();
    assert_eq!(
        expected.semantic_values(),
        clip::decode(&output, Some(&map)).unwrap().semantic_values()
    );
}

#[test]
fn navigation_preserves_current_playback_state_and_note_overrides() {
    let body = format!(
        "<measure number=\"1\">{}<sound fine=\"yes\"/></measure><measure number=\"2\"><direction><direction-type><words>pizzicato</words></direction-type></direction>{}<sound dacapo=\"yes\"/></measure>",
        note("C", ""),
        note("D", "")
    );
    let s = parsed(&body);
    assert_eq!(
        s.notes
            .iter()
            .map(|n| (n.articulation.at, n.articulation.subclass))
            .collect::<Vec<_>>(),
        vec![(0x10, 0), (0x11, 10), (0x11, 10)]
    );
    let override_note = "<note pizzicato=\"no\"><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration></note>";
    let body = format!(
        "<measure number=\"1\"><direction><direction-type><words>pizzicato</words></direction-type></direction>{override_note}{}</measure>",
        note("D", "")
    );
    let s = parsed(&body);
    assert_eq!(s.notes[0].articulation.at, 0x10);
    assert_eq!(s.notes[1].articulation.subclass, 10);
}

#[test]
fn fine_decimal_boundaries_use_library_arithmetic() {
    let tempo =
        convert::wire::tempo_from_bpm(convert::model::decimal("60.000000000000001").unwrap())
            .unwrap();
    assert_eq!(tempo, vec![0xd010_0000, 100_000_000, 0, 0]);
    assert_eq!(
        convert::rules::level(convert::model::decimal("126.999999999999999").unwrap(), 32).unwrap(),
        u32::MAX
    );
    let source = score(
        "<measure number=\"1\"><note><pitch><step>G</step><alter>0.999999999999999</alter><octave>9</octave></pitch><duration>1</duration></note></measure>",
    );
    let normalized = xml::read(&source, &RenderingPolicy::default()).unwrap().1;
    assert_eq!(normalized.notes.len(), 1, "{normalized:?}");
    let converted = convert::xml_to_midi2(&source, &RenderingPolicy::default()).unwrap();
    let decoded = clip::decode(&converted.clip, Some(&converted.companion.dictionary)).unwrap();
    assert_eq!(
        decoded.notes[0].pitch,
        Rational::new(i64::from(u32::MAX), 1 << 25)
    );
    assert!(
        converted
            .report
            .decisions
            .iter()
            .any(|d| d.rule == "Pitch 7.25 rounding")
    );
}

/// A full final Flex payload can end in a multibyte UTF-8 character. Upstream's
/// iterator drops byte twelve, falsely rejecting that valid message. Preserve
/// external text bytes across inverse conversion, including both packet sizes.
#[test]
fn external_multi_packet_flex_text_survives_the_inverse() {
    use midi2::prelude::*;
    let mut text = midi2::flex_data::UnknownMetadataText::<Vec<u32>>::new();
    for value in [
        "An independently constructed text longer than one Flex Data packet.",
        "abcdefghijé",
        "abcdefghijklmnopqrstuvé",
    ] {
        text.set_text(value);
        let mut words = vec![
            0x0040_0000,
            0x0030_01e0,
            0x0040_0000,
            0xf020_0000,
            0,
            0,
            0,
            0x0040_0000,
        ];
        words.extend(text.data());
        words.extend([0x0040_0000, 0xf021_0000, 0, 0, 0]);
        let mut input = b"SMF2CLIP".to_vec();
        for word in words {
            input.extend(word.to_be_bytes());
        }
        let expected = clip::decode(&input, None).unwrap();
        let message = expected
            .events
            .iter()
            .find(|e| e.words[0] & 0xffff == 0x0100)
            .unwrap();
        assert_eq!(message.words.len(), value.len().div_ceil(12) * 4);
        assert_eq!(
            convert::remarks::metadata(&message.words).as_deref(),
            Some(value)
        );
        let inverse = convert::midi2_to_xml(&input, None, false).unwrap();
        let (_, s) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
        let (output, map, _) = clip::encode(&s).unwrap();
        assert_eq!(
            expected.semantic_values(),
            clip::decode(&output, Some(&map)).unwrap().semantic_values()
        );
    }
}

#[test]
fn typed_library_adapters_keep_fractional_durations_and_timewise_entities() {
    let source=b"<score-timewise version='4.0'><part-list><score-part id='P1'><part-name>Violin &amp; friends</part-name></score-part></part-list><measure number='1'><part id='P1'><attributes><divisions>1.5</divisions></attributes><note attack='0.125'><pitch><step>C</step><alter>0.5</alter><octave>4</octave></pitch><duration>1.5</duration><notations><technical><other-technical>spiccato blurred</other-technical></technical></notations></note></part></measure></score-timewise>";
    let normalized = xml::read(source, &RenderingPolicy::default()).unwrap().1;
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.notes[0].pitch, Rational::new(121, 2));
    assert_eq!(normalized.notes[0].onset, Rational::new(1, 12));
    assert_eq!(normalized.notes[0].end, Rational::from_integer(1));
    let converted = convert::xml_to_midi2(source, &RenderingPolicy::default()).unwrap();
    let decoded = clip::decode(&converted.clip, Some(&converted.companion.dictionary)).unwrap();
    let canonical = xml::write(&decoded).unwrap();
    let (_, reconstructed) = xml::read(canonical.as_bytes(), &RenderingPolicy::default()).unwrap();
    assert_eq!(reconstructed.notes[0].pitch, Rational::new(121, 2));
}

#[test]
fn controllers_between_attacks_and_before_releases_keep_their_order() {
    let words = [
        0x0040_0000,
        0x0030_01e0,
        0x0040_0000,
        0xf020_0000,
        0,
        0,
        0,
        0x0040_0000,
        0x4090_4010,
        0x9000_0000,
        0x40b0_0b00,
        u32::MAX,
        0x4090_3c10,
        0x9000_0000,
        0x0040_01e0,
        0x40b0_4000,
        u32::MAX,
        0x4080_4010,
        0x8000_0000,
        0x4080_3c10,
        0x8000_0000,
        0x0040_0000,
        0xf021_0000,
        0,
        0,
        0,
    ];
    let mut input = b"SMF2CLIP".to_vec();
    for word in words {
        input.extend(word.to_be_bytes());
    }
    let expected = clip::decode(&input, None).unwrap();
    let expression = expected
        .events
        .iter()
        .find(|e| e.words[0] == 0x40b0_0b00)
        .unwrap();
    let c = expected.notes.iter().find(|n| n.key == 60).unwrap();
    let e = expected.notes.iter().find(|n| n.key == 64).unwrap();
    assert!(e.on_order < expression.order && expression.order < c.on_order);
    let pedal = expected
        .events
        .iter()
        .find(|e| e.words[0] == 0x40b0_4000)
        .unwrap();
    assert!(pedal.order < e.off_order && pedal.order < c.off_order);
    let inverse = convert::midi2_to_xml(&input, None, false).unwrap();
    let (_, s) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let (output, map, _) = clip::encode(&s).unwrap();
    assert_eq!(
        expected.semantic_values(),
        clip::decode(&output, Some(&map)).unwrap().semantic_values()
    );
}

#[test]
fn ornaments_are_realized_after_ties_resolve_their_full_span() {
    let body = format!(
        "<measure number=\"1\">{}{}</measure>",
        note(
            "C",
            "<tie type=\"start\"/><notations><ornaments><trill-mark/></ornaments></notations>"
        ),
        note("C", "<tie type=\"stop\"/>")
    );
    let s = parsed(&body);
    assert_eq!(s.notes.len(), 4);
    assert_eq!(
        s.notes.iter().map(|n| n.onset).collect::<Vec<_>>(),
        vec![
            Rational::from_integer(0),
            Rational::new(1, 2),
            Rational::from_integer(1),
            Rational::new(3, 2)
        ]
    );
    assert_eq!(s.notes.last().unwrap().end, Rational::from_integer(2));
}

#[test]
fn native_compound_and_left_hand_pizzicato_keep_independent_meaning() {
    let body = "<measure number=\"1\"><note pizzicato=\"yes\"><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><notations><articulations><staccato/></articulations></notations></note><note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><notations><technical><left-hand-pizzicato/></technical></notations></note></measure>";
    let s = parsed(body);
    assert_eq!(s.notes[0].articulation.subclass, 10);
    assert!(
        s.notes[0].descriptor.as_ref().unwrap().qualifiers["urn:midi2workbench:facets"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("short.staccato-off"))
    );
    assert_eq!(s.notes[1].articulation.subclass, 10);
    assert_eq!(s.notes[1].articulation.direction, 2);
}

#[rstest]
#[case::forte_piano("fp", 90, 45)]
#[case::sforzando("sf", 90, 72)]
fn transient_dynamics_are_envelopes_and_do_not_change_later_attacks(
    #[case] kind: &str,
    #[case] peak: u32,
    #[case] later: u32,
) {
    let sound = if kind == "sf" {
        "<sound dynamics=\"100\"/>"
    } else {
        ""
    };
    let body = format!(
        "<measure number=\"1\"><direction><direction-type><dynamics><{kind}/></dynamics></direction-type>{sound}</direction>{}{}</measure>",
        note("C", ""),
        note("D", "")
    );
    let source = score(&body);
    let c = convert::xml_to_midi2(&source, &RenderingPolicy::default()).unwrap();
    let decoded = clip::decode(&c.clip, Some(&c.companion.dictionary)).unwrap();
    assert_eq!(
        u32::from(decoded.notes[0].velocity),
        convert::rules::upscale(peak, 7, 16)
    );
    assert_eq!(
        u32::from(decoded.notes[1].velocity),
        convert::rules::upscale(later, 7, 16)
    );
    let curve =
        convert::extension::dynamic_curve(decoded.notes[0].descriptor.as_ref().unwrap()).unwrap();
    assert_eq!(curve[0].1, convert::rules::upscale(peak, 7, 32));
    assert_eq!(curve.last().unwrap().0, Rational::new(1, 8));
    assert_eq!(
        curve.last().unwrap().1,
        convert::rules::upscale(later, 7, 32)
    );
    let canonical = xml::write(&decoded).unwrap();
    let (_, back) = xml::read(canonical.as_bytes(), &RenderingPolicy::default()).unwrap();
    let (bytes, map, _) = clip::encode(&back).unwrap();
    assert_eq!(
        decoded.semantic_values(),
        clip::decode(&bytes, Some(&map)).unwrap().semantic_values()
    );
}

#[test]
fn arpeggio_default_and_explicit_arrow_have_defined_pitch_order() {
    for down in [false, true] {
        let mut body = String::from("<measure number=\"1\">");
        for (i, step) in ["C", "E", "G"].iter().enumerate() {
            let chord = if i == 0 { "" } else { "<chord/>" };
            let direction = if down && i == 0 {
                " direction=\"down\""
            } else {
                ""
            };
            write!(body,"<note>{chord}<pitch><step>{step}</step><octave>4</octave></pitch><duration>1</duration><notations><arpeggiate{direction}/></notations></note>").unwrap();
        }
        body.push_str("</measure>");
        let s = parsed(&body);
        let expected = if down {
            [
                Rational::new(1, 8),
                Rational::new(1, 16),
                Rational::from_integer(0),
            ]
        } else {
            [
                Rational::from_integer(0),
                Rational::new(1, 16),
                Rational::new(1, 8),
            ]
        };
        assert_eq!(
            s.notes.iter().map(|n| n.onset).collect::<Vec<_>>(),
            expected
        );
        assert!(s.notes.iter().all(|n| n.end == Rational::from_integer(1)));
    }
}

#[test]
fn numbered_trill_continues_to_its_resolution_and_accidentals_change_auxiliary_pitch() {
    let body = format!(
        "<measure number=\"1\">{}{}{}</measure>",
        note(
            "C",
            "<notations><ornaments><trill-mark/><wavy-line type=\"start\"/></ornaments></notations>"
        ),
        note(
            "D",
            "<notations><ornaments><wavy-line type=\"continue\"/></ornaments></notations>"
        ),
        note(
            "E",
            "<notations><ornaments><wavy-line type=\"stop\"/></ornaments></notations>"
        )
    );
    let s = parsed(&body);
    assert_eq!(s.notes.len(), 9);
    assert_eq!(s.notes[4].pitch, Rational::from_integer(64));
    assert_eq!(s.notes[8].onset, Rational::from_integer(2));
    let altered = parsed(&format!(
        "<measure number=\"1\">{}</measure>",
        note(
            "G",
            "<notations><ornaments><trill-mark/><accidental-mark>flat</accidental-mark></ornaments></notations>"
        )
    ));
    assert_eq!(altered.notes[0].pitch, Rational::from_integer(68));
}

#[test]
fn sound_offset_and_time_only_change_the_actual_dynamic_scope() {
    let body = format!(
        "<measure number=\"1\"><direction><direction-type><words/></direction-type><offset sound=\"no\">1</offset><sound dynamics=\"100\"><offset>1</offset></sound></direction>{}{}</measure>",
        note("C", ""),
        note("D", "")
    );
    let s = parsed(&body);
    assert_eq!(
        u32::from(s.notes[0].velocity),
        convert::rules::upscale(72, 7, 16)
    );
    assert_eq!(
        u32::from(s.notes[1].velocity),
        convert::rules::upscale(90, 7, 16)
    );
    let repeated = format!(
        "<measure number=\"1\"><sound dynamics=\"0\" time-only=\"2\"/>{}<barline location=\"right\"><repeat direction=\"backward\"/></barline></measure>",
        note("C", "")
    );
    let s = parsed(&repeated);
    assert_eq!(s.notes.len(), 1);
    assert_eq!(s.duration, Rational::from_integer(2));
}

#[test]
fn literal_zero_midi2_attacks_and_trailing_silence_have_a_real_inverse() {
    let words = [
        0x0040_0000_u32,
        0x0030_01e0,
        0x0040_0000,
        0xf020_0000,
        0,
        0,
        0,
        0x0040_0000,
        0x4090_3c10,
        0,
        0x0040_01e0,
        0x4080_3c10,
        0x8000_0000,
        0x0040_01e0,
        0xf021_0000,
        0,
        0,
        0,
    ];
    let mut input = b"SMF2CLIP".to_vec();
    for word in words {
        input.extend(word.to_be_bytes());
    }
    let expected = clip::decode(&input, None).unwrap();
    assert_eq!(expected.notes.len(), 1);
    assert_eq!(expected.notes[0].velocity, 0);
    assert_eq!(expected.duration, Rational::from_integer(2));
    let inverse = convert::midi2_to_xml(&input, None, false).unwrap();
    let (_, back) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let (bytes, map, _) = clip::encode(&back).unwrap();
    assert_eq!(
        expected.semantic_values(),
        clip::decode(&bytes, Some(&map)).unwrap().semantic_values()
    );
}

#[rstest]
#[case("none", 0)]
#[case("soft", 1)]
#[case("hard", 2)]
#[case("rise", 3)]
#[case("fall", 4)]
fn release_subclasses_survive_canonical_musicxml(#[case] name: &str, #[case] code: u8) {
    let body = format!(
        "<measure number=\"1\">{}</measure>",
        note(
            "C",
            &format!(
                "<play><other-play type=\"urn:midi2workbench:articulation:v2\">{{\"technique\":\"core.normal\",\"release\":\"{name}\"}}</other-play></play>"
            )
        )
    );
    let s = parsed(&body);
    let (bytes, map, _) = clip::encode(&s).unwrap();
    let decoded = clip::decode(&bytes, Some(&map)).unwrap();
    assert_eq!(decoded.notes[0].articulation.release, code);
    let canonical = xml::write(&decoded).unwrap();
    let (_, back) = xml::read(canonical.as_bytes(), &RenderingPolicy::default()).unwrap();
    assert_eq!(back.notes[0].articulation.release, code);
}

#[test]
fn explicit_swing_uses_the_musicxml_ratio_and_eligibility_rules() {
    let eighth = |step: &str, extra: &str| {
        format!(
            "<note {extra}><pitch><step>{step}</step><octave>4</octave></pitch><duration>1</duration><type>eighth</type></note>"
        )
    };
    let body = format!(
        "<measure number=\"1\"><attributes><divisions>2</divisions></attributes><sound><swing><first>2</first><second>1</second></swing></sound>{}{}</measure>",
        eighth("C", ""),
        eighth("D", "attack=\"0.1\" release=\"0.1\"")
    );
    let s = parsed(&body);
    assert_eq!(s.notes[0].end, Rational::new(2, 3));
    assert_eq!(s.notes[1].onset, Rational::new(2, 3) + Rational::new(1, 20));
    assert_eq!(s.notes[1].end, Rational::new(21, 20));
    let body = format!(
        "<measure number=\"1\"><attributes><divisions>2</divisions></attributes><sound><swing><first>2</first><second>1</second></swing></sound>{}{}</measure>",
        note("C", ""),
        note("D", "")
    );
    let unchanged = parsed(&body); // No type element: MusicXML explicitly excludes these notes.
    assert_eq!(unchanged.notes[0].end, Rational::new(1, 2));
    assert_eq!(unchanged.notes[1].onset, Rational::new(1, 2));
}

#[test]
fn simultaneous_unison_ties_keep_two_note_lifecycles() {
    let tied = |kind: &str, chord: bool| {
        format!(
            "<note>{}<pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><tie type=\"{kind}\"/></note>",
            if chord { "<chord/>" } else { "" }
        )
    };
    let body = format!(
        "<measure number=\"1\">{}{}{}{}</measure>",
        tied("start", false),
        tied("start", true),
        tied("stop", false),
        tied("stop", true)
    );
    let s = parsed(&body);
    assert_eq!(s.notes.len(), 2);
    assert!(
        s.notes
            .iter()
            .all(|n| n.onset == Rational::from_integer(0) && n.end == Rational::from_integer(2))
    );
    let (bytes, map, _) = clip::encode(&s).unwrap();
    let decoded = clip::decode(&bytes, Some(&map)).unwrap();
    assert_ne!(decoded.notes[0].route, decoded.notes[1].route);
}

#[test]
fn numbered_articulation_spans_are_half_open_and_voice_scoped() {
    let start = "<notations><other-notation type=\"start\" number=\"2\">spiccato</other-notation></notations>";
    let stop = "<notations><other-notation type=\"stop\" number=\"2\">spiccato</other-notation></notations>";
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}{}{}</measure>",
        note("C", start),
        note("D", ""),
        note("E", stop)
    ));
    assert_eq!(
        s.notes
            .iter()
            .map(|n| (n.articulation.at, n.articulation.subclass))
            .collect::<Vec<_>>(),
        [(0x11, 5), (0x11, 5), (0x10, 0)]
    );
}

#[test]
fn source_pitch_replacement_is_explicit_and_rounding_is_accounted() {
    let body = note(
        "C",
        "<play><other-play type=\"urn:midi2workbench:articulation:v2\">{\"technique\":\"core.normal\",\"qualifiers\":{\"urn:midi2workbench:sounding-pitch\":\"241/4\"},\"replace\":[\"/score-partwise/part[@id='P1']/measure[1]/note[1]/pitch[1]\"]}</other-play></play>",
    );
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{body}</measure>"
    ));
    assert_eq!(s.notes[0].pitch, Rational::new(241, 4));
    assert!(
        s.report
            .decisions
            .iter()
            .any(|d| d.rule == "explicit replacement")
    );
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}</measure>",
        note("C", "").replace("<note>", "<note dynamics=\"71.234567\">")
    ));
    let (_, _, report) = clip::encode(&s).unwrap();
    assert_ne!(report.numeric_changes, Vec::<serde_json::Value>::new());
    for change in report.numeric_changes {
        let exact: num_rational::Ratio<i128> =
            change["exactMagnitude"].as_str().unwrap().parse().unwrap();
        let encoded =
            num_rational::Ratio::from_integer(i128::from(change["encoded"].as_u64().unwrap()));
        assert!(
            (num_rational::Ratio::new(-1, 2)..=num_rational::Ratio::new(1, 2))
                .contains(&(exact - encoded))
        );
    }
    assert!(s.report.source_constructs.contains_key("note@dynamics"));
    assert!(s.report.source_constructs.keys().all(|key| {
        s.report
            .decisions
            .iter()
            .any(|d| d.source == format!("construct:{key}"))
    }));
}

#[test]
fn instrument_ids_and_isolated_channels_inherit_initial_programs() {
    let source=b"<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>Strings</part-name><score-instrument id=\"I1\"><instrument-name>Violin</instrument-name></score-instrument><midi-instrument id=\"I1\"><midi-channel>1</midi-channel><midi-program>41</midi-program><volume>70</volume></midi-instrument><score-instrument id=\"I2\"><instrument-name>Viola</instrument-name></score-instrument><midi-instrument id=\"I2\"><midi-channel>3</midi-channel><midi-program>42</midi-program></midi-instrument></score-part></part-list><part id=\"P1\"><measure number=\"1\"><attributes><divisions>1</divisions></attributes><note><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration><instrument id=\"I1\"/></note><backup><duration>2</duration></backup><note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><instrument id=\"I1\"/></note><note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><instrument id=\"I2\"/></note></measure></part></score-partwise>";
    let (_, s) = xml::read(source, &RenderingPolicy::default()).unwrap();
    assert_eq!(
        s.notes
            .iter()
            .find(|n| n.pitch == 62.into())
            .unwrap()
            .route
            .channel,
        2
    );
    let (bytes, map, _) = clip::encode(&s).unwrap();
    let d = clip::decode(&bytes, Some(&map)).unwrap();
    let aliases = d
        .notes
        .iter()
        .filter(|n| n.pitch == 60.into())
        .map(|n| n.route.channel)
        .collect::<Vec<_>>();
    assert_ne!(aliases[0], aliases[1]);
    assert!(!aliases.contains(&2));
    for channel in aliases {
        assert!(d.configuration.iter().any(|e| e.words[0] >> 28 == 4
            && (e.words[0] >> 20) & 15 == 12
            && (e.words[0] >> 16) & 15 == u32::from(channel)
            && e.words[1] >> 24 == 40));
    }
    let inverse = xml::write(&d).unwrap();
    let (_, back) = xml::read(inverse.as_bytes(), &RenderingPolicy::default()).unwrap();
    let (bytes, map, _) = clip::encode(&back).unwrap();
    assert_eq!(
        d.semantic_values(),
        clip::decode(&bytes, Some(&map)).unwrap().semantic_values()
    );
}

#[test]
fn persistent_mutes_use_sound_play_and_respect_instrument_scope() {
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes><sound><play><mute>practice</mute></play></sound>{}</measure>",
        note("C", "")
    ));
    assert_eq!(
        s.notes[0].descriptor.as_ref().unwrap().qualifiers["urn:midi2workbench:mute"]["slot"],
        2
    );
}

#[test]
fn unpitched_instrument_binding_uses_midi_key_not_display_pitch() {
    let source=b"<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>Snare</part-name><score-instrument id=\"I1\"><instrument-name>Snare</instrument-name></score-instrument><midi-device id=\"I1\" port=\"2\"/><midi-instrument id=\"I1\"><midi-channel>10</midi-channel><midi-unpitched>39</midi-unpitched></midi-instrument></score-part></part-list><part id=\"P1\"><measure number=\"1\"><attributes><divisions>1</divisions></attributes><note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><instrument id=\"I1\"/></note></measure></part></score-partwise>";
    let (_, s) = xml::read(source, &RenderingPolicy::default()).unwrap();
    assert_eq!(s.notes[0].pitch, 38.into());
    assert_eq!(
        s.notes[0].route,
        Route {
            group: 1,
            channel: 9
        }
    );
    let (bytes, map, _) = clip::encode(&s).unwrap();
    assert_eq!(
        clip::decode(&bytes, Some(&map)).unwrap().notes[0].pitch,
        38.into()
    );
}

#[test]
fn header_tempo_and_meter_have_prescribed_order_and_multiplicity() {
    let tempo = [0xd010_0000_u32, 50_000_000, 0, 0];
    let meter = [0xd010_0001, 0x0402_1808, 0, 0];
    for (configuration, valid) in [
        (vec![tempo, meter], true),
        (vec![tempo, tempo], false),
        (vec![meter], false),
        (vec![[0x40b0_0700, 0, 0, 0], tempo], false),
    ] {
        let mut words = vec![0x0040_0000, 0x0030_01e0];
        for event in configuration {
            words.push(0x0040_0000);
            words.extend(event);
        }
        words.extend([
            0x0040_0000,
            0xf020_0000,
            0,
            0,
            0,
            0x0040_0000,
            0xf021_0000,
            0,
            0,
            0,
        ]);
        let mut bytes = b"SMF2CLIP".to_vec();
        for word in words {
            bytes.extend(word.to_be_bytes());
        }
        assert_eq!(clip::decode(&bytes, None).is_ok(), valid);
    }
}

#[test]
fn independent_parts_sharing_a_route_keep_their_expression_scope() {
    let source=b"<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>Upper</part-name><midi-instrument id=\"I1\"><midi-channel>1</midi-channel></midi-instrument></score-part><score-part id=\"P2\"><part-name>Lower</part-name><midi-instrument id=\"I2\"><midi-channel>1</midi-channel></midi-instrument></score-part></part-list><part id=\"P1\"><measure number=\"1\"><attributes><divisions>1</divisions></attributes><direction><direction-type><dynamics><f/></dynamics></direction-type></direction><note><pitch><step>C</step><octave>5</octave></pitch><duration>1</duration></note></measure></part><part id=\"P2\"><measure number=\"1\"><attributes><divisions>1</divisions></attributes><direction><direction-type><dynamics><p/></dynamics></direction-type></direction><note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration></note></measure></part></score-partwise>";
    let (_, s) = xml::read(source, &RenderingPolicy::default()).unwrap();
    assert_eq!(
        s.notes
            .iter()
            .find(|n| n.pitch == 72.into())
            .unwrap()
            .velocity,
        u16::try_from(convert::rules::level(90.into(), 16).unwrap()).unwrap()
    );
    assert_eq!(
        s.notes
            .iter()
            .find(|n| n.pitch == 60.into())
            .unwrap()
            .velocity,
        u16::try_from(convert::rules::level(45.into(), 16).unwrap()).unwrap()
    );
}

#[test]
fn explicit_grace_dynamics_record_the_numeric_boundary() {
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes><note dynamics=\"71.234567\"><grace steal-time-following=\"10\"/><pitch><step>D</step><octave>4</octave></pitch><type>eighth</type></note>{}</measure>",
        note("C", "")
    ));
    assert_eq!(s.notes.len(), 2);
    assert!(
        s.report
            .numeric_changes
            .iter()
            .any(|v| v["source"].as_str().unwrap().ends_with("note[1]"))
    );
    assert_eq!(
        s.notes
            .iter()
            .find(|n| n.pitch == 62.into())
            .unwrap()
            .velocity,
        u16::try_from(
            convert::rules::level(
                convert::model::decimal("71.234567").unwrap() * Rational::new(9, 10),
                16
            )
            .unwrap()
        )
        .unwrap()
    );
}

#[test]
fn a_span_stop_on_a_later_chord_member_excludes_the_whole_boundary() {
    let start = "<notations><other-notation type=\"start\">spiccato</other-notation></notations>";
    let stop = "<notations><other-notation type=\"stop\">spiccato</other-notation></notations>";
    let last = note("E", stop).replace("<note>", "<note><chord/>");
    let s = parsed(&format!(
        "<measure number=\"1\"><attributes><divisions>1</divisions></attributes>{}{}{last}</measure>",
        note("C", start),
        note("D", "")
    ));
    assert_eq!(
        s.notes
            .iter()
            .map(|n| (n.articulation.at, n.articulation.subclass))
            .collect::<Vec<_>>(),
        [(0x11, 5), (0x10, 0), (0x10, 0)]
    );
}
