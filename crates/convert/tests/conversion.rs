mod support;
use musicxml_midi2_convert::{
    self as convert, ci, clip,
    extension::Dictionary,
    model::{Rational, RenderingPolicy},
    rules, wire, xml,
};
use rstest::rstest;
use std::{collections::BTreeMap, path::Path};
use support::assert_schema;

fn score(note: &str) -> Vec<u8> {
    format!("<score-partwise version=\"4.0\"><part-list><score-part id=\"P1\"><part-name>Violin</part-name></score-part></part-list><part id=\"P1\"><measure number=\"1\"><attributes><divisions>480</divisions></attributes>{note}</measure></part></score-partwise>").into_bytes()
}
fn note(body: &str) -> String {
    format!(
        "<note><pitch><step>C</step><octave>4</octave></pitch><duration>480</duration>{body}</note>"
    )
}

#[test]
fn catalog_is_single_source() {
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../mapping/orchestral-subclasses.json")).unwrap();
    if std::env::var_os("UPDATE_CONVERT_CATALOG").is_some() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut catalog = expected.clone();
        catalog["rows"] = serde_json::to_value(rules::VOCABULARY).unwrap();
        std::fs::write(
            root.join("mapping/orchestral-subclasses.json"),
            format!("{}\n", serde_json::to_string_pretty(&catalog).unwrap()),
        )
        .unwrap();
        return;
    }
    assert_eq!(
        serde_json::to_value(rules::VOCABULARY).unwrap(),
        expected["rows"]
    );
    let counts = (16..24)
        .map(|at| rules::VOCABULARY.iter().filter(|r| r.at == at).count())
        .collect::<Vec<_>>();
    assert_eq!(counts, [11, 16, 10, 14, 9, 14, 16, 16]);
}

#[rstest]
#[case(0, 0)]
#[case(64, 0x8000)]
#[case(70, 0x8c30)]
#[case(96, 0xc104)]
#[case(127, 0xffff)]
fn normative_scaling(#[case] input: u32, #[case] expected: u32) {
    assert_eq!(rules::upscale(input, 7, 16), expected);
}

#[rstest]
#[case("<articulations><staccato/></articulations>", 0x11, 0)]
#[case("<articulations><staccato/><tenuto/></articulations>", 0x11, 9)]
#[case("<articulations><spiccato/></articulations>", 0x11, 5)]
fn native_coordinate(#[case] marks: &str, #[case] at: u8, #[case] sub: u8) {
    let (_, s) = xml::read(
        &score(&note(&format!("<notations>{marks}</notations>"))),
        &RenderingPolicy::default(),
    )
    .unwrap();
    assert_eq!(
        (s.notes[0].articulation.at, s.notes[0].articulation.subclass),
        (at, sub)
    );
}

#[test]
fn all_named_coordinates_survive_both_directions() {
    for row in rules::VOCABULARY {
        let input = score(&note(&format!(
            "<notations><technical><other-technical>{}</other-technical></technical></notations>",
            xml::escape(row.term)
        )));
        let c = convert::xml_to_midi2(&input, &RenderingPolicy::default())
            .unwrap_or_else(|e| panic!("{}: {e}", row.id));
        let decoded = clip::decode(&c.clip, Some(&c.companion.dictionary)).unwrap();
        assert_eq!(
            (
                decoded.notes[0].articulation.at,
                decoded.notes[0].articulation.subclass
            ),
            (row.at, row.sub)
        );
        let inverse = xml::write(&decoded).unwrap();
        let (_, back) = xml::read(inverse.as_bytes(), &RenderingPolicy::default()).unwrap();
        let (encoded, d, _) = clip::encode(&back).unwrap();
        assert_eq!(
            decoded.semantic_values(),
            clip::decode(&encoded, Some(&d)).unwrap().semantic_values(),
            "{}",
            row.id
        );
    }
}

#[test]
/// Files copied without a companion used to fail despite containing the full
/// dictionary. Guard embedded decoding and reject a contradictory supplied map:
/// otherwise a stale companion could silently relabel each note's refinement.
fn refinement_is_per_note_and_checks_embedded_map() {
    let input = score(&note(
        "<notations><articulations><spiccato/></articulations></notations><play><other-play type=\"urn:midi2workbench:articulation:v2\">{\"technique\":\"short.spiccato\",\"qualifiers\":{\"urn:midi2workbench:sample-character\":\"blurred\"}}</other-play></play>",
    ));
    let c = convert::xml_to_midi2(&input, &RenderingPolicy::default()).unwrap();
    let embedded = clip::decode(&c.clip, None).unwrap();
    let mut stale = c.companion.dictionary.clone();
    stale.descriptors.clear();
    assert!(clip::decode(&c.clip, Some(&stale)).is_err());
    let decoded = clip::decode(&c.clip, Some(&c.companion.dictionary)).unwrap();
    assert_eq!(embedded.semantic_values(), decoded.semantic_values());
    assert_eq!(
        decoded.notes[0].descriptor.as_ref().unwrap().qualifiers["urn:midi2workbench:sample-character"],
        "blurred"
    );
    let inverse = xml::write(&decoded).unwrap();
    let (_, back) = xml::read(inverse.as_bytes(), &RenderingPolicy::default()).unwrap();
    assert_eq!(back.notes[0].descriptor, decoded.notes[0].descriptor);
}

#[test]
fn independent_note_wire_vector() {
    let a = rules::Articulation {
        at: 0x11,
        subclass: 10,
        variation: 0,
        direction: 1,
        string: 2,
        reset_round_robin: true,
        release: 0,
    };
    let words = wire::note(convert::model::Route::default(), 74, 0x9000, a, true).unwrap();
    assert_eq!(words, [0x4090_4a11, 0x9000_a04a]);
    assert!(midi2::UmpMessage::try_from(words.as_slice()).is_err());
    wire::validate(&words).unwrap();
    assert!(rules::Articulation::from_data(0x11, 0x0030).is_err());
}

#[test]
fn profile_codec_and_enable_exchange() {
    let mut initiator = ci::Engine::new(1).unwrap();
    let mut responder = ci::Engine::new(2).unwrap();
    let request = initiator.request_enable(2, 15, 100).unwrap();
    let bytes = request.encode().unwrap();
    assert_eq!(&bytes[..5], &[0x7e, 15, 13, 0x22, 2]);
    let decoded = ci::Message::decode(&bytes).unwrap();
    assert_eq!(request, decoded);
    assert!(!initiator.remote.contains_key(&(2, 15)));
    let response = responder.receive(&decoded).unwrap().remove(0);
    initiator.receive(&response).unwrap();
    assert!(initiator.remote[&(2, 15)]);
    assert!(responder.enabled.contains(&15));
    initiator.request_enable(2, 0, 10).unwrap();
    assert_eq!(initiator.expire(10), [(2, 0)]);
}

#[test]
fn details_validate_lengths_and_counts() {
    let f = ci::Features::default();
    assert_eq!(f.payload(1).unwrap().len(), 130);
    assert_eq!(f.payload(0x40).unwrap().len(), 96);
    let mut bad = f.payload(1).unwrap();
    bad[2] = 17;
    assert!(ci::Features::default().apply(1, &bad).is_err());
    for target in [1, 0x40] {
        let data = f.payload(target).unwrap();
        let m = ci::Message::channel(
            0,
            1,
            2,
            ci::Body::DetailsReply {
                profile: rules::PROFILE_ID,
                target,
                data,
            },
        )
        .unwrap();
        assert_eq!(ci::Message::decode(&m.encode().unwrap()).unwrap(), m);
    }
}

#[test]
fn malformed_clip_cannot_publish_a_performance() {
    for data in [b"".as_slice(), b"SMF2CLIP", b"SMF2CLIP\0\0\0\0"] {
        assert!(clip::decode(data, None).is_err());
    }
    let c = convert::xml_to_midi2(&score(&note("")), &RenderingPolicy::default()).unwrap();
    assert!(clip::decode(&c.clip[..c.clip.len() - 4], Some(&Dictionary::default())).is_err());
    let mut trailing = c.clip.clone();
    trailing.extend([0; 4]);
    assert!(clip::decode(&trailing, None).is_err());
}

#[test]
fn exact_decimals_and_tuplets_use_libraries() {
    assert_eq!(
        convert::model::decimal("0.125").unwrap(),
        Rational::new(1, 8)
    );
    assert_eq!(
        convert::model::decimal("1.20").unwrap(),
        Rational::new(6, 5)
    );
    assert!(convert::model::decimal("NaN").is_err());
}

#[rstest]
#[case::numerator_overflow("-9223372036854775808/-1", Err("E_PRECISION"))]
#[case::denominator_overflow("1/-9223372036854775808", Err("E_PRECISION"))]
#[case::reducible_limits("-9223372036854775808/-9223372036854775808", Ok(1.into()))]
#[case::zero("0/-9223372036854775808", Ok(0.into()))]
#[case::negative_denominator("1/-2", Ok(Rational::new(-1, 2)))]
#[case::zero_denominator("1/0", Err("E_NUMBER"))]
fn rational_normalization_checks_signed_limits(
    #[case] input: &str,
    #[case] expected: Result<Rational, &str>,
) {
    assert_eq!(convert::model::decimal(input).map_err(|e| e.code), expected);
}

#[rstest]
#[case::bach_piano(0)]
#[case::cpebach_ornaments(1)]
#[case::joplin_syncopation(2)]
#[case::schoenberg_articulation(3)]
#[case::schumann_voice_pedal(4)]
#[case::weber_transposing_wind(5)]
#[case::beethoven_largest(6)]
#[case::beethoven_fugue_harmonic(7)]
#[case::mozart_classical_quartet(8)]
#[case::bach_chorale_baseline(9)]
#[case::bach_orchestral_case_study(10)]
#[case::bach_large_orchestration(11)]
fn corpus_roundtrip(#[case] index: usize) {
    let sample: serde_json::Value =
        serde_json::from_str(include_str!("corpus-sample.json")).unwrap();
    let relative = sample["files"][index]["path"].as_str().unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../music21/music21/corpus")
        .join(relative);
    let bytes = std::fs::read(&path).unwrap();
    let converted = convert::xml_to_midi2(&bytes, &RenderingPolicy::default())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let decoded = clip::decode(&converted.clip, Some(&converted.companion.dictionary)).unwrap();
    let inverse =
        convert::midi2_to_xml(&converted.clip, Some(&converted.companion), false).unwrap();
    let (_, back) = xml::read(&inverse, &RenderingPolicy::default()).unwrap();
    let (encoded, dictionary, _) = clip::encode(&back).unwrap();
    assert_eq!(
        decoded.semantic_values(),
        clip::decode(&encoded, Some(&dictionary))
            .unwrap()
            .semantic_values(),
        "{}",
        path.display()
    );
    let canonical = xml::write(&decoded).unwrap();
    assert_schema(&[&inverse, canonical.as_bytes()], &path);
    let (_, reconstructed) = xml::read(canonical.as_bytes(), &RenderingPolicy::default()).unwrap();
    let (encoded, map, _) = clip::encode(&reconstructed).unwrap();
    let reparsed = clip::decode(&encoded, Some(&map)).unwrap();
    assert_eq!(
        decoded.semantic_values(),
        reparsed.semantic_values(),
        "canonical standalone inverse: {}",
        path.display()
    );
    let mut coordinates = BTreeMap::<String, usize>::new();
    for n in &decoded.notes {
        *coordinates
            .entry(format!(
                "{:02X}/{:X}",
                n.articulation.at, n.articulation.subclass
            ))
            .or_default() += 1;
    }
    let canonical_doc = roxmltree::Document::parse(&canonical).unwrap();
    let mut canonical_marks = BTreeMap::<String, usize>::new();
    for n in canonical_doc.descendants().filter(|n| {
        n.is_element()
            && [
                "staccato",
                "staccatissimo",
                "slur",
                "tie",
                "fermata",
                "trill-mark",
                "inverted-mordent",
                "other-play",
                "strong-accent",
                "pedal",
                "harmonic",
            ]
            .contains(&n.tag_name().name())
    }) {
        *canonical_marks
            .entry(n.tag_name().name().into())
            .or_default() += 1;
    }
    let words = converted.clip[8..]
        .chunks_exact(4)
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .collect::<Vec<_>>();
    let mut wire_attributes = BTreeMap::<String, usize>::new();
    let mut index = 0;
    while index < words.len() {
        let head = words[index];
        if head >> 28 == 4 && head >> 20 & 15 == 9 {
            *wire_attributes
                .entry(format!("{:02X}", head & 255))
                .or_default() += 1;
        }
        index += wire::packet_len(head);
    }
    let mut observation = serde_json::json!({"pianoChannels":decoded.piano_routes,"profiles":converted.companion.requirements["profiles"],"noteOnAttributeTypes":wire_attributes,"path":relative,"performedAttacks":decoded.notes.len(),"durationQuarters":decoded.duration.to_string(),"noteOnCoordinates":coordinates,"canonicalMarks":canonical_marks,"dictionaryDescriptors":converted.companion.dictionary.descriptors.len(),"bothInversePerformanceHashesMatch":true,"bothInverseXsdValid":true});
    if relative == "bach/bwv29.8.mxl" {
        let (_, normalized) = xml::read(&bytes, &RenderingPolicy::default()).unwrap();
        let traces=normalized.notes.iter().filter(|n|n.descriptor.as_ref().is_some_and(|d|d.qualifiers.contains_key("urn:midi2workbench:expanded-pattern"))).take(6).map(|n|serde_json::json!({"source":n.source,"pitch":n.pitch.to_string(),"onset":n.onset.to_string(),"end":n.end.to_string(),"route":n.route,"descriptor":n.descriptor,"noteOnWords":wire::note(n.route,n.key,n.velocity,n.articulation,true).unwrap().iter().map(|w|format!("{w:08X}")).collect::<Vec<_>>()})).collect::<Vec<_>>();
        observation["ornamentTrace"] = serde_json::json!(traces);
    }
    println!("CORPUS_OBSERVATION {observation}");
}

#[test]
fn corpus_sample_is_pinned_and_diversified() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../music21/music21/corpus");
    let sample: serde_json::Value =
        serde_json::from_str(include_str!("corpus-sample.json")).unwrap();
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("corpus-manifest.json")).unwrap();
    let files = sample["files"].as_array().unwrap();
    assert_eq!(files.len(), 12);
    let paths = files
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(paths.len(), 12);
    assert!(paths.contains("beethoven/opus132.mxl"));
    assert!(paths.contains("beethoven/opus133.mxl"));
    assert!(paths.contains("bach/bwv29.8.mxl"));
    assert_eq!(files.iter().filter(|f| f["role"] == "piano").count(), 4);
    assert_eq!(files.iter().filter(|f| f["role"] == "strings").count(), 3);
    for file in files {
        let path = file["path"].as_str().unwrap();
        let pinned = inventory["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["path"] == path)
            .unwrap();
        assert_eq!(file["path"], pinned["path"]);
        assert!(root.join(path).is_file());
        assert!(
            !inventory["invalidInputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["path"] == path)
        );
    }
}

#[test]
fn xml_and_archive_boundaries_reject_independent_adversarial_inputs() {
    use std::io::{Cursor, Write};
    let negative = String::from_utf8(score(&note("")))
        .unwrap()
        .replace("<note>", "<sound dynamics=\"-1.11\"/><note>");
    assert_eq!(
        xml::read(negative.as_bytes(), &RenderingPolicy::default())
            .unwrap_err()
            .code,
        "E_INPUT_DYNAMIC"
    );
    let bad_program = String::from_utf8(score(&note(""))).unwrap().replace(
        "</score-part>",
        "<midi-instrument id=\"I1\"><midi-program>0</midi-program></midi-instrument></score-part>",
    );
    assert_eq!(
        xml::read(bad_program.as_bytes(), &RenderingPolicy::default())
            .unwrap_err()
            .code,
        "E_INPUT_PROGRAM"
    );
    let entity=b"<!DOCTYPE score-partwise [<!ENTITY attack SYSTEM 'file:///etc/passwd'>]><score-partwise>&attack;</score-partwise>";
    assert!(xml::read(entity, &RenderingPolicy::default()).is_err());
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file(
            "META-INF/container.xml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    archive
        .write_all(
            b"<container><rootfiles><rootfile full-path=\"../score.xml\"/></rootfiles></container>",
        )
        .unwrap();
    archive
        .start_file("../score.xml", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"<score-partwise/>").unwrap();
    assert!(
        xml::extract(&archive.finish().unwrap().into_inner())
            .unwrap_err()
            .message
            .contains("unsafe rootfile")
    );
    assert!(
        xml::read(
            &score(&note(
                "<play><other-play type=\"urn:unknown\">unknown</other-play></play>"
            )),
            &RenderingPolicy::default()
        )
        .is_err()
    );
    let dictionary = Dictionary {
        version: 999,
        ..Dictionary::default()
    };
    assert!(dictionary.validate().is_err());
    assert!(Dictionary::default().lookup(1).is_err());
}

/// An exported part can occupy auxiliary MIDI routes when equal keys overlap.
/// Route-based reconstruction would split it into spurious instruments; ignoring
/// meter would also leave its long note untied across the bar. This guards both
/// observable failures using explicit expected part/measure/tie counts and XSD.
#[test]
fn standalone_arrangement_groups_routes_and_ties_across_bars() {
    use convert::{
        arrangement::{Arrangement, Part},
        model::{Event, Route},
    };
    let input = score(
        "<note><pitch><step>C</step><octave>4</octave></pitch><duration>2400</duration></note><backup><duration>2400</duration></backup><note><pitch><step>C</step><octave>4</octave></pitch><duration>480</duration></note>",
    );
    let conversion = convert::xml_to_midi2(&input, &RenderingPolicy::default()).unwrap();
    let mut performance = clip::decode(&conversion.clip, None).unwrap();
    let routes = performance
        .notes
        .iter()
        .map(|n| n.route)
        .collect::<std::collections::BTreeSet<Route>>()
        .into_iter()
        .collect();
    let declaration = Arrangement {
        version: 1,
        parts: vec![Part {
            name: "Violins".into(),
            routes,
        }],
        decisions: vec![],
    };
    performance.configuration.push(Event {
        time: Rational::from_integer(0),
        order: 0,
        words: declaration.words().unwrap(),
    });
    let (bytes, _, _) = clip::encode(&performance).unwrap();
    assert!(convert::validation::inspect(&bytes).admissible);
    let xml = convert::midi2_to_xml(&bytes, None, false).unwrap();
    let doc = roxmltree::Document::parse(std::str::from_utf8(&xml).unwrap()).unwrap();
    assert_eq!(
        doc.descendants().filter(|n| n.has_tag_name("part")).count(),
        1
    );
    assert_eq!(
        doc.descendants()
            .filter(|n| n.has_tag_name("measure"))
            .count(),
        2
    );
    assert_eq!(
        doc.descendants()
            .filter(|n| n.has_tag_name("tie") && n.attribute("type") == Some("start"))
            .count(),
        1
    );
    assert_eq!(
        doc.descendants()
            .filter(|n| n.has_tag_name("tie") && n.attribute("type") == Some("stop"))
            .count(),
        1
    );
    assert_schema(&[&xml], Path::new("standalone-arrangement"));
}
