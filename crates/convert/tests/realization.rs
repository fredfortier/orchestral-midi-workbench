use musicxml_midi2_convert::{
    adapt::{self, Plan},
    clip,
    extension::{DYNAMIC_CURVE, DYNAMIC_LEVEL},
    model::{Descriptor, Note, Performance, Rational, RenderingPolicy, Route},
    rules::Articulation,
};
use serde_json::json;
use std::collections::BTreeMap;

fn plan() -> Plan {
    serde_json::from_value(json!({
        "rendition": {"version":1,"tracks":[{"id":"strings","name":"Strings"}],
            "instruments":(["short","long"].map(|id| json!({"id":id,"nki":format!("/library/{id}.nki"),
                "player":{"id":"fixture","name":"Fixture","version":"1","format":"VST3"},
                "settings":[{"control":"MPE","value":false,"unit":"boolean"}]}))),
            "bindings":[{"route":{"group":0,"channel":0},"track":"strings","channel":1,"instrument":"short"},
                        {"route":{"group":0,"channel":1},"track":"strings","channel":2,"instrument":"long"}]},
        "rules":[{"sourcePrefix":"short/","articulation":Articulation::default(),"descriptor":null,
            "route":{"group":0,"channel":0},"dynamics":{"kind":"keyVelocity"}},
            {"sourcePrefix":"long/","articulation":Articulation::default(),"descriptor":{"technique":"core.normal"},
            "route":{"group":0,"channel":1},"dynamics":{"kind":"crossfade","controller":1,"keyVelocity":40000,
                "sourceLaw":"requireDynamic","expression":{"controller":11,"value":u32::MAX},"curvePolicy":"channelHold"}}],
        "passages":[{"id":"pulse","track":"strings","sourcePrefixes":["short/"],"instrument":"short","expectedNotes":1,
            "reason":"Reviewed detached accompaniment; preserve attack dynamics."},
            {"id":"held","track":"strings","sourcePrefixes":["long/"],"instrument":"long","expectedNotes":1,
            "reason":"Reviewed held phrase with within-note shaping."}]
    })).unwrap()
}
fn source() -> Performance {
    let note = |source: &str, key, onset, end, descriptor| Note {
        source: source.into(),
        key,
        pitch: Rational::from_integer(i64::from(key)),
        onset: Rational::from_integer(onset),
        end: Rational::from_integer(end),
        on_order: 2,
        off_order: 0,
        velocity: 12345,
        release_velocity: 23456,
        route: Route {
            group: 0,
            channel: 7,
        },
        articulation: Articulation::default(),
        descriptor,
    };
    Performance {
        duration: Rational::from_integer(4),
        notes: vec![
            note("short/1", 60, 0, 1, None),
            note(
                "long/1",
                62,
                1,
                4,
                Some(Descriptor {
                    technique: "core.normal".into(),
                    qualifiers: BTreeMap::from([
                        (DYNAMIC_LEVEL.into(), json!(100)),
                        (
                            DYNAMIC_CURVE.into(),
                            json!([
                                [[0, 1], 100, 0],
                                [[1, 1], 200, 0],
                                [[2, 1], 300, 0],
                                [[3, 1], 400, 0]
                            ]),
                        ),
                    ]),
                }),
            ),
        ],
        ..Performance::default()
    }
}

#[test]
fn compiled_gates_and_shared_ccs_preserve_the_note_routes() {
    let mut input = serde_json::to_value(plan()).unwrap();
    let shared = json!({"group":1,"channel":0});
    input["rendition"]["controlRoutes"] = json!([{"track":"strings","route":shared}]);
    input["trackCcCurves"] = json!([{"route":shared,"controller":11,"points":[
        {"time":[0,1],"value":123_456}, {"time":[1,1],"value":345_678}
    ]}]);
    input["noteEdits"] = json!([{"source":"short/1","onset":[0,1],"key":60,"end":[3,4]}]);
    let reviewed: Plan = serde_json::from_value(input.clone()).unwrap();
    let (result, _) = adapt::render(&source(), &reviewed).unwrap();
    let (bytes, dictionary, _) = clip::encode(&result).unwrap();
    let decoded = clip::decode(&bytes, Some(&dictionary)).unwrap();
    reviewed.rendition.validate_performance(&decoded).unwrap();
    assert_eq!(decoded.notes[0].end, Rational::new(3, 4));
    assert_eq!(
        decoded.notes[0].route,
        Route {
            group: 0,
            channel: 0
        }
    );
    for (beat, value) in [(0, 123_456), (1, 345_678)] {
        let expected = musicxml_midi2_convert::wire::cc(
            Route {
                group: 1,
                channel: 0,
            },
            11,
            value,
        );
        assert!(
            decoded
                .configuration
                .iter()
                .chain(&decoded.events)
                .any(|event| {
                    event.time == Rational::from_integer(beat) && event.words == expected
                })
        );
    }
    input["noteEdits"][0]["source"] = json!("stale");
    assert!(adapt::render(&source(), &serde_json::from_value(input).unwrap()).is_err());
}

#[test]
fn reviewed_passages_and_curve_samples_survive_encoding_without_duplicate_envelopes() {
    let input = source();
    let (rendered, original) = adapt::render(&input, &plan()).unwrap();
    assert_eq!(rendered.notes[0].velocity, 12345);
    assert_eq!(rendered.notes[1].velocity, 40000);
    assert_eq!(
        rendered.notes[0].route,
        Route {
            group: 0,
            channel: 0
        }
    );
    assert_eq!(
        rendered.notes[1].route,
        Route {
            group: 0,
            channel: 1
        }
    );
    for (before, after) in input.notes.iter().zip(&rendered.notes) {
        assert_eq!(
            (
                before.onset,
                before.end,
                before.pitch,
                before.release_velocity
            ),
            (after.onset, after.end, after.pitch, after.release_velocity)
        );
    }
    assert_eq!(original[1].velocity, 12345);
    assert!(
        input.notes[1]
            .descriptor
            .as_ref()
            .unwrap()
            .qualifiers
            .contains_key(DYNAMIC_CURVE)
    );
    assert!(
        !rendered.notes[1]
            .descriptor
            .as_ref()
            .unwrap()
            .qualifiers
            .contains_key(DYNAMIC_CURVE)
    );
    // Independent protocol words and absolute beat expectations. The endpoint at
    // note-off emits no CC and therefore cannot collide with the next attack.
    assert_eq!(
        rendered
            .configuration
            .iter()
            .map(|e| &e.words)
            .collect::<Vec<_>>(),
        vec![&vec![0x40b1_0100, 100], &vec![0x40b1_0b00, u32::MAX]]
    );
    assert_eq!(
        rendered
            .events
            .iter()
            .map(|e| (e.time, e.words.clone()))
            .collect::<Vec<_>>(),
        vec![
            (Rational::from_integer(2), vec![0x40b1_0100, 200]),
            (Rational::from_integer(3), vec![0x40b1_0100, 300])
        ]
    );
    let (bytes, dictionary, _) = clip::encode(&rendered).unwrap();
    assert!(!dictionary.parameters.contains_key(&126));
    let decoded = clip::decode(&bytes, Some(&dictionary)).unwrap();
    assert_eq!(
        decoded
            .events
            .iter()
            .filter(|e| e.words[0] >> 28 == 4)
            .map(|e| (e.time, e.words.clone()))
            .collect::<Vec<_>>(),
        vec![
            (Rational::from_integer(2), vec![0x40b1_0100, 200]),
            (Rational::from_integer(3), vec![0x40b1_0100, 300])
        ]
    );
    assert_eq!(decoded.notes.len(), 2);
}

#[test]
fn collapsing_contrasting_passages_to_one_receiver_fails_even_with_complete_rules() {
    let mut collapsed = plan();
    collapsed.rules[1].route = collapsed.rules[0].route;
    collapsed.rules[1].dynamics = adapt::Dynamics::KeyVelocity;
    let mut input = source();
    input.notes[1]
        .descriptor
        .as_mut()
        .unwrap()
        .qualifiers
        .remove(DYNAMIC_CURVE);
    let error = adapt::render(&input, &collapsed).unwrap_err();
    assert!(
        error
            .message
            .contains("held expected instrument long, observed short"),
        "{error}"
    );
}

#[test]
fn reviewed_track_ownership_cannot_be_changed_behind_an_unchanged_instrument() {
    let mut value = serde_json::to_value(plan()).unwrap();
    value["rendition"]["tracks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"other","name":"Other"}));
    value["rendition"]["bindings"][1]["track"] = json!("other");
    let changed: Plan = serde_json::from_value(value).unwrap();
    let error = adapt::render(&source(), &changed).unwrap_err();
    assert!(
        error
            .message
            .contains("expected track strings, observed other"),
        "{error}"
    );
}

#[test]
fn passage_review_refuses_incomplete_ambiguous_stale_or_malformed_expectations() {
    for mode in 0..10 {
        let mut p = plan();
        match mode {
            0 => {
                p.passages.pop();
            }
            1 => p.passages.push(p.passages[0].clone()),
            2 => p.passages[0].expected_notes = 2,
            3 => p.passages[0].source_prefixes.push("missing/".into()),
            4 => p.passages[0].source_prefixes.push("short/1".into()),
            5 => p.passages[0].instrument = "missing".into(),
            6 => p.passages[0].reason = " ".into(),
            7 => p.passages[0].expected_notes = 0,
            8 => p.passages[0].source_prefixes.clear(),
            _ => p.passages[0].source_prefixes[0].clear(),
        }
        assert!(adapt::render(&source(), &p).is_err(), "mode {mode}");
    }
    let mut value = serde_json::to_value(plan()).unwrap();
    value["passages"][0]["guessedBowing"] = json!(true);
    assert!(serde_json::from_value::<Plan>(value).is_err());
}

#[test]
fn curve_policy_is_explicit_and_rejects_velocity_only_receivers_and_invalid_offsets() {
    for curve in [
        json!([[[1, 1], 100, 0]]),
        json!([[[-1, 1], 100, 0]]),
        json!([[[0, 1], 100, 0], [[0, 1], 200, 1]]),
        json!([[[0, 1], 100, 0], [[4, 1], 200, 0]]),
    ] {
        let mut input = source();
        input.notes[1]
            .descriptor
            .as_mut()
            .unwrap()
            .qualifiers
            .insert(DYNAMIC_CURVE.into(), curve);
        assert!(adapt::render(&input, &plan()).is_err());
    }
    let mut p = plan();
    if let adapt::Dynamics::Crossfade { curve_policy, .. } = &mut p.rules[1].dynamics {
        *curve_policy = adapt::CurvePolicy::Reject;
    }
    assert!(
        adapt::render(&source(), &p)
            .unwrap_err()
            .message
            .contains("channelHold")
    );
    p.rules[1].dynamics = adapt::Dynamics::KeyVelocity;
    assert!(
        adapt::render(&source(), &p)
            .unwrap_err()
            .message
            .contains("key-velocity")
    );
    let mut value = serde_json::to_value(plan()).unwrap();
    value["rules"][1]["dynamics"]
        .as_object_mut()
        .unwrap()
        .remove("curvePolicy");
    let historical: Plan = serde_json::from_value(value).unwrap();
    assert!(adapt::render(&source(), &historical).is_err());
}

#[test]
fn shared_channel_curves_must_agree_at_every_interior_change() {
    let mut p = plan();
    p.passages[1].expected_notes = 2;
    let mut input = source();
    let mut second = input.notes[1].clone();
    second.key = 65;
    second.pitch = Rational::from_integer(65);
    second.source = "long/2".into();
    input.notes.push(second);
    let (rendered, _) = adapt::render(&input, &p).unwrap();
    assert_eq!(rendered.events.len(), 2); // One merged axis, not two copies.
    input.notes[2]
        .descriptor
        .as_mut()
        .unwrap()
        .qualifiers
        .insert(
            DYNAMIC_CURVE.into(),
            json!([[[0, 1], 100, 0], [[1, 1], 201, 0], [[2, 1], 300, 0]]),
        );
    let error = adapt::render(&input, &p).unwrap_err();
    assert!(error.message.contains("beat 2"), "{error}");
    assert!(error.message.contains("long/1") && error.message.contains("long/2"));
}

#[test]
fn channel_ownership_ends_at_note_off_and_initializes_before_a_late_entry() {
    let mut p = plan();
    p.passages[1].expected_notes = 2;
    let mut input = source();
    input.notes[1].end = Rational::from_integer(2);
    input.notes[1]
        .descriptor
        .as_mut()
        .unwrap()
        .qualifiers
        .insert(
            DYNAMIC_CURVE.into(),
            json!([[[0, 1], 100, 0], [[1, 1], 999, 0]]),
        );
    let mut second = input.notes[1].clone();
    second.source = "long/2".into();
    second.onset = Rational::from_integer(2);
    second.end = Rational::from_integer(4);
    second.descriptor.as_mut().unwrap().qualifiers.insert(
        DYNAMIC_CURVE.into(),
        json!([[[0, 1], 200, 0], [[1, 1], 300, 0]]),
    );
    input.notes.push(second);
    let (rendered, _) = adapt::render(&input, &p).unwrap();
    assert_eq!(rendered.configuration[0].time, Rational::from_integer(0));
    assert_eq!(rendered.configuration[0].words, vec![0x40b1_0100, 100]);
    assert_eq!(rendered.events[0].time, Rational::from_integer(2));
    assert_eq!(rendered.events[0].words, vec![0x40b1_0100, 200]);
    assert_eq!(rendered.events.len(), 2);
}

#[test]
fn explicit_gates_and_cross_bar_slurs_keep_distinct_receivers_through_conversion() {
    let score = br#"<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Strings</part-name></score-part></part-list>
    <part id="P1"><measure number="10"><attributes><divisions>4</divisions><time><beats>2</beats><beat-type>4</beat-type></time></attributes>
    <note release="-1"><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration><type>eighth</type></note>
    <note><pitch><step>D</step><octave>4</octave></pitch><duration>6</duration><type>quarter</type><dot/><notations><slur type="start" number="1"/></notations></note>
    </measure><measure number="11"><note><pitch><step>E</step><octave>4</octave></pitch><duration>8</duration><type>half</type><notations><slur type="stop" number="1"/></notations></note></measure></part></score-partwise>"#;
    let mut p = plan();
    let first = "/score-partwise/part[@id='P1']/measure[1]/note[1]";
    let long = "/score-partwise/part[@id='P1']/measure[1]/note[2]";
    let second_bar = "/score-partwise/part[@id='P1']/measure[2]/";
    p.rules[0].source_prefix = first.into();
    p.rules[1].source_prefix = "/score-partwise/part[@id='P1']/".into();
    p.rules[1].articulation = Articulation::named("core.legato").unwrap();
    p.rules[1].descriptor = None;
    if let adapt::Dynamics::Crossfade { source_law, .. } = &mut p.rules[1].dynamics {
        *source_law = adapt::SourceLaw::DynamicOrVelocity16;
    }
    p.passages[0].source_prefixes = vec![first.into()];
    p.passages[1].source_prefixes = vec![long.into(), second_bar.into()];
    p.passages[1].expected_notes = 2;
    let converted =
        musicxml_midi2_convert::xml_to_rendition(score, &RenderingPolicy::default(), &p).unwrap();
    let decoded = clip::decode(&converted.clip, Some(&converted.companion.dictionary)).unwrap();
    let short = decoded.notes.iter().find(|n| n.key == 60).unwrap();
    assert_eq!(short.end, Rational::new(1, 4));
    assert_eq!(short.route.channel, 0);
    for key in [62, 64] {
        let note = decoded.notes.iter().find(|n| n.key == key).unwrap();
        assert_eq!(note.route.channel, 1);
        assert_eq!(
            note.articulation,
            Articulation::named("core.legato").unwrap()
        );
    }
    assert_eq!(
        decoded.notes.iter().find(|n| n.key == 64).unwrap().onset,
        Rational::from_integer(2)
    );
    assert!(converted.companion.source_xml.contains("release=\"-1\""));
    assert!(
        converted
            .report
            .decisions
            .iter()
            .any(|d| d.rule == "reviewed passage")
    );
}

#[test]
fn curve_budgets_and_unrepresentable_absolute_times_refuse() {
    let mut input = source();
    input.notes[1]
        .descriptor
        .as_mut()
        .unwrap()
        .qualifiers
        .insert(
            DYNAMIC_CURVE.into(),
            json!(vec![json!([[0, 1], 100, 0]); 65_537]),
        );
    assert!(
        adapt::render(&input, &plan())
            .unwrap_err()
            .message
            .contains("point budget")
    );
    let mut input = source();
    input.notes[1].onset = Rational::new(1, i64::MAX - 1);
    input.notes[1]
        .descriptor
        .as_mut()
        .unwrap()
        .qualifiers
        .insert(
            DYNAMIC_CURVE.into(),
            json!([[[0, 1], 100, 0], [[1, i64::MAX], 200, 0]]),
        );
    assert!(
        adapt::render(&input, &plan())
            .unwrap_err()
            .message
            .contains("curve time overflow")
    );
}
