use musicxml_midi2_convert::{
    clip,
    model::{Note, Performance, Rational, RenderingPolicy, Route},
    preserve::Companion,
    rendition::Rendition,
    rules::Articulation,
};
use serde_json::{Value, json};

fn manifest() -> Value {
    json!({
        "version": 1,
        "tracks": [{"id":"oboe1","name":"Oboe I"},{"id":"oboe2","name":"Oboe II"}],
        "instruments": [{
            "id":"oboe-sustain", "nki":"/library/Solo Oboe Sustains.nki",
            "player":{"id":"kontakt7","name":"Kontakt 7","version":"7.10.1","format":"VST3"},
            "settings":[{"control":"MPE","value":false,"unit":"boolean"}]
        }],
        "bindings":[
            {"route":{"group":0,"channel":0},"track":"oboe1","channel":0,"instrument":"oboe-sustain"},
            {"route":{"group":1,"channel":0},"track":"oboe2","channel":0,"instrument":"oboe-sustain"}
        ]
    })
}
fn performance() -> Performance {
    Performance {
        duration: Rational::from_integer(1),
        notes: (0..2)
            .map(|group| Note {
                on_order: 0,
                off_order: 0,
                onset: Rational::from_integer(0),
                end: Rational::from_integer(1),
                pitch: Rational::from_integer(60),
                velocity: 32768,
                release_velocity: 16384,
                route: Route { group, channel: 0 },
                key: 60,
                articulation: Articulation::default(),
                descriptor: None,
                source: format!("source-oboe-{group}"),
            })
            .collect(),
        ..Performance::default()
    }
}
#[test]
fn independent_tracks_reuse_one_patch_definition_without_per_note_assignments() {
    let rendition: Rendition = serde_json::from_value(manifest()).unwrap();
    rendition.validate_performance(&performance()).unwrap();
    assert_eq!(rendition.instruments.len(), 1);
    assert_eq!(rendition.bindings.len(), 2);
    assert_ne!(rendition.bindings[0].track, rendition.bindings[1].track);
}
#[test]
fn malformed_missing_duplicate_conflicting_and_stale_bindings_fail() {
    let base = manifest();
    let mut cases = Vec::new();
    let mut missing = base.clone();
    missing["bindings"].as_array_mut().unwrap().pop();
    cases.push(missing);
    for (pointer, value) in [
        ("/version", json!(2)),
        ("/bindings/1/route/group", json!(0)),
        ("/bindings/1/route/channel", json!(16)),
        ("/bindings/1/track", json!("oboe1")),
        ("/bindings/1/instrument", json!("unknown")),
        ("/bindings/1/channel", json!(16)),
        ("/tracks/1/id", json!("oboe1")),
        ("/tracks/1/name", json!("Oboe I")),
        ("/instruments/0/nki", json!("relative.nki")),
        ("/instruments/0/nki", json!("")),
        ("/instruments/0/player/version", json!("")),
        ("/instruments/0/settings/0/value", Value::Null),
    ] {
        let mut case = base.clone();
        *case.pointer_mut(pointer).unwrap() = value;
        cases.push(case);
    }
    for case in cases {
        let parsed: Rendition = serde_json::from_value(case.clone()).unwrap();
        assert!(
            parsed.validate_performance(&performance()).is_err(),
            "{case}"
        );
    }
    let rendition: Rendition = serde_json::from_value(base).unwrap();
    let mut short = performance();
    short.notes.pop();
    assert!(
        rendition
            .validate_performance(&short)
            .unwrap_err()
            .message
            .contains("stale")
    );
    let mut unknown = performance();
    unknown.notes[0].route.group = 2;
    assert!(
        rendition
            .validate_performance(&unknown)
            .unwrap_err()
            .message
            .contains("no binding")
    );
}
#[test]
fn unknown_assignment_and_route_fields_are_not_silently_discarded() {
    for pointer in [
        "",
        "/bindings/0",
        "/bindings/0/route",
        "/instruments/0",
        "/instruments/0/settings/0",
    ] {
        let mut value = manifest();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("typo".into(), json!(true));
        assert!(serde_json::from_value::<Rendition>(value).is_err());
    }
}
#[test]
fn companion_detects_changed_assignments_and_inconsistent_versions() {
    let score = performance();
    let (bytes, dictionary, _) = clip::encode(&score).unwrap();
    let decoded = clip::decode(&bytes, Some(&dictionary)).unwrap();
    let mut companion = Companion::new(
        b"source",
        "source",
        &bytes,
        &decoded,
        dictionary,
        &RenderingPolicy::default(),
    );
    companion
        .set_rendition(serde_json::from_value(manifest()).unwrap(), &decoded)
        .unwrap();
    assert_eq!(companion.version, 2);
    companion.validate_rendition(&decoded).unwrap();
    let saved = serde_json::to_vec(&companion).unwrap();
    for field in ["instrument", "track"] {
        let mut changed: Companion = serde_json::from_slice(&saved).unwrap();
        let binding = &mut changed.rendition.as_mut().unwrap().bindings[0];
        if field == "instrument" {
            binding.instrument = "swapped-patch".into();
        } else {
            binding.track = "oboe2".into();
        }
        assert_eq!(
            changed.validate_rendition(&decoded).unwrap_err().code,
            "E_RENDITION"
        );
    }
    for version in [0, 1, 3] {
        let mut changed: Companion = serde_json::from_slice(&saved).unwrap();
        changed.version = version;
        assert_eq!(
            changed.validate_rendition(&decoded).unwrap_err().code,
            "E_COMPANION_VERSION"
        );
    }
    let mut changed: Companion = serde_json::from_slice(&saved).unwrap();
    changed.rendition = None;
    assert!(changed.validate_rendition(&decoded).is_err());
}

fn adaptation_plan() -> musicxml_midi2_convert::adapt::Plan {
    serde_json::from_value(json!({
        "rendition":manifest(),
        "rules":[
            {"sourcePrefix":"source-oboe-0", "articulation":Articulation::default(), "descriptor":null,
             "route":{"group":0,"channel":0},
             "dynamics":{"kind":"crossfade","controller":1,"keyVelocity":40000,
                          "sourceLaw":"dynamicOrVelocity16","expression":{"controller":11,"value":4_294_967_295_u32}}},
            {"sourcePrefix":"source-oboe-1", "articulation":Articulation::default(), "descriptor":null,
             "route":{"group":1,"channel":0},"dynamics":{"kind":"keyVelocity"}}
        ]
    })).unwrap()
}
#[test]
fn additional_initial_controllers_preserve_full_values_and_route_ownership() {
    use musicxml_midi2_convert::adapt::{self, InitialController};
    let mut plan = adaptation_plan();
    plan.initial_controllers = vec![
        InitialController {
            route: Route {
                group: 0,
                channel: 0,
            },
            controller: 3,
            value: 0,
        },
        InitialController {
            route: Route {
                group: 1,
                channel: 0,
            },
            controller: 3,
            value: u32::MAX,
        },
    ];
    let (rendered, _) = adapt::render(&performance(), &plan).unwrap();
    let words: Vec<_> = rendered
        .configuration
        .iter()
        .map(|event| event.words.clone())
        .collect();
    // Independent UMP vectors: MT4, group0/1, statusB, channel0, controller3.
    assert!(words.contains(&vec![0x40b0_0300, 0]));
    assert!(words.contains(&vec![0x41b0_0300, u32::MAX]));
    assert_eq!(words.len(), 4); // Two authored CC3 values plus CC1 and CC11.
    assert!(
        rendered
            .configuration
            .iter()
            .all(|event| event.time == Rational::from_integer(0))
    );
    assert_eq!(rendered.notes.len(), 2);
    assert_eq!(rendered.notes[1].velocity, performance().notes[1].velocity);
    let (bytes, dictionary, _) = clip::encode(&rendered).unwrap();
    let decoded = clip::decode(&bytes, Some(&dictionary)).unwrap();
    for expected in [vec![0x40b0_0300, 0], vec![0x41b0_0300, u32::MAX]] {
        assert!(
            decoded
                .configuration
                .iter()
                .any(|event| event.words == expected)
        );
    }
}
#[test]
fn initial_controllers_refuse_duplicates_unbound_routes_and_dynamic_collisions() {
    use musicxml_midi2_convert::adapt::{self, InitialController};
    let initial = InitialController {
        route: Route {
            group: 0,
            channel: 0,
        },
        controller: 3,
        value: 0,
    };
    let mut plan = adaptation_plan();
    for entries in [
        vec![initial.clone(), initial.clone()],
        vec![InitialController {
            controller: 128,
            ..initial.clone()
        }],
        vec![InitialController {
            route: Route {
                group: 2,
                channel: 0,
            },
            ..initial.clone()
        }],
        vec![InitialController {
            controller: 1,
            ..initial.clone()
        }],
        vec![InitialController {
            controller: 11,
            ..initial.clone()
        }],
    ] {
        plan.initial_controllers = entries;
        assert_eq!(
            adapt::render(&performance(), &plan).unwrap_err().code,
            "E_RECEIVER_ADAPTATION"
        );
    }
    for controller in [0, 6, 32, 38, 98, 99, 100, 101] {
        plan.initial_controllers = vec![InitialController {
            controller,
            ..initial.clone()
        }];
        assert!(adapt::render(&performance(), &plan).is_err());
    }
    let mut value = serde_json::to_value(adaptation_plan()).unwrap();
    for bad in [json!(-1), json!(4_294_967_296_u64), json!(true), json!("0")] {
        value["initialControllers"] =
            json!([{"route":{"group":0,"channel":0},"controller":3,"value":bad}]);
        assert!(serde_json::from_value::<adapt::Plan>(value.clone()).is_err());
    }
}
#[test]
fn conversion_adaptation_renders_routes_fixed_attack_and_controller_values() {
    use musicxml_midi2_convert::{adapt, model::Event, wire};
    let mut source = performance();
    for note in &mut source.notes {
        note.route = Route {
            group: 0,
            channel: 7,
        };
    }
    let mut later = source.notes[0].clone();
    later.source = "source-oboe-0-later".into();
    later.onset = Rational::from_integer(2);
    later.end = Rational::from_integer(3);
    later.velocity = 16384;
    source.notes.push(later);
    source.duration = Rational::from_integer(3);
    let (rendered, originals) = adapt::render(&source, &adaptation_plan()).unwrap();
    assert_eq!(
        rendered
            .notes
            .iter()
            .map(|note| note.velocity)
            .collect::<Vec<_>>(),
        [40000, 32768, 40000]
    );
    assert_eq!(
        originals
            .iter()
            .map(|original| original.velocity)
            .collect::<Vec<_>>(),
        [32768, 32768, 16384]
    );
    assert!(originals.iter().all(|original| original.route
        == Route {
            group: 0,
            channel: 7
        }));
    assert_eq!(
        rendered.notes[1].route,
        Route {
            group: 1,
            channel: 0
        }
    );
    assert_eq!(
        rendered.configuration,
        [
            Event {
                time: Rational::from_integer(0),
                order: 0,
                words: vec![0x40b0_0100, 2_147_516_416]
            },
            Event {
                time: Rational::from_integer(0),
                order: 0,
                words: vec![0x40b0_0b00, 4_294_967_295]
            },
        ]
    );
    assert_eq!(
        rendered.events,
        [Event {
            time: Rational::from_integer(2),
            order: 0,
            words: vec![0x40b0_0100, 1_073_758_208]
        }]
    );
    let (bytes, dictionary, _) = clip::encode(&rendered).unwrap();
    let decoded = clip::decode(&bytes, Some(&dictionary)).unwrap();
    assert_eq!(decoded.notes[0].velocity, 40000);
    assert!(decoded.configuration.iter().any(|event| event.words
        == wire::cc(
            Route {
                group: 0,
                channel: 0
            },
            1,
            2_147_516_416
        )));
}
#[test]
fn adaptation_refuses_ambiguous_rules_conflicting_dynamics_and_unmapped_events() {
    use musicxml_midi2_convert::{adapt, model::Event};
    let source = performance();
    let mut ambiguous = adaptation_plan();
    ambiguous.rules.push(ambiguous.rules[0].clone());
    assert!(
        adapt::render(&source, &ambiguous)
            .unwrap_err()
            .message
            .contains("2 matching")
    );
    let mut missing = source.clone();
    missing.notes[0].source = "unassigned-note".into();
    assert!(
        adapt::render(&missing, &adaptation_plan())
            .unwrap_err()
            .message
            .contains("unassigned-note")
    );
    let mut conflicting = source.clone();
    let mut overlapping = conflicting.notes[0].clone();
    overlapping.velocity = 123;
    overlapping.source = "source-oboe-0-overlap".into();
    conflicting.notes.push(overlapping);
    assert!(
        adapt::render(&conflicting, &adaptation_plan())
            .unwrap_err()
            .message
            .contains("conflicting simultaneous")
    );
    let mut required = adaptation_plan();
    if let musicxml_midi2_convert::adapt::Dynamics::Crossfade { source_law, .. } =
        &mut required.rules[0].dynamics
    {
        *source_law = musicxml_midi2_convert::adapt::SourceLaw::RequireDynamic;
    }
    assert!(
        adapt::render(&source, &required)
            .unwrap_err()
            .message
            .contains("missing required dynamic level")
    );
    let mut evolving = source.clone();
    evolving.notes[0].descriptor = Some(musicxml_midi2_convert::model::Descriptor {
        technique: String::new(),
        qualifiers: std::collections::BTreeMap::from([(
            musicxml_midi2_convert::extension::DYNAMIC_CURVE.into(),
            json!([[[0, 1], 123, 0]]),
        )]),
    });
    assert!(
        adapt::render(&evolving, &adaptation_plan())
            .unwrap_err()
            .message
            .contains("source-oboe-0")
    );
    let mut malformed = serde_json::to_value(adaptation_plan()).unwrap();
    malformed["rules"][0]["articulation"]["unsupportedFacet"] = json!(true);
    assert!(serde_json::from_value::<musicxml_midi2_convert::adapt::Plan>(malformed).is_err());
    let mut channel = source;
    channel.events.push(Event {
        time: Rational::from_integer(0),
        order: 0,
        words: vec![0x40b0_0100, 123],
    });
    assert!(
        adapt::render(&channel, &adaptation_plan())
            .unwrap_err()
            .message
            .contains("unmapped required timed")
    );
}
#[test]
fn production_conversion_keeps_original_route_and_changed_velocity_as_explicit_provenance() {
    use musicxml_midi2_convert::{adapt, xml_to_midi2, xml_to_rendition};
    let xml = br#"<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Oboe</part-name></score-part></part-list><part id="P1"><measure number="1"><attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes><note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;
    let mut plan = adaptation_plan();
    plan.rendition.tracks.pop();
    plan.rendition.bindings.pop();
    plan.rules.pop();
    plan.rules[0].source_prefix = "/score-partwise/part[@id='P1']/".into();
    plan.rules[0].route = Route {
        group: 7,
        channel: 3,
    };
    plan.rendition.bindings[0].route = plan.rules[0].route;
    let duplicated = std::str::from_utf8(xml).unwrap().replace(
        "</note></measure>",
        "</note><note><chord/><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure>"
    );
    assert_eq!(
        xml_to_rendition(duplicated.as_bytes(), &RenderingPolicy::default(), &plan)
            .err()
            .unwrap()
            .code,
        "E_RENDITION_ROUTE_ISOLATION"
    );
    let original = xml_to_midi2(xml, &RenderingPolicy::default()).unwrap();
    let rendition = xml_to_rendition(xml, &RenderingPolicy::default(), &plan).unwrap();
    assert_eq!(rendition.companion.version, 2);
    let decoded = clip::decode(&rendition.clip, Some(&rendition.companion.dictionary)).unwrap();
    assert_eq!(
        decoded.notes[0].route,
        Route {
            group: 7,
            channel: 3
        }
    );
    assert_eq!(decoded.notes[0].velocity, 40000);
    assert_eq!(
        decoded.notes[0].release_velocity,
        clip::decode(&original.clip, Some(&original.companion.dictionary))
            .unwrap()
            .notes[0]
            .release_velocity
    );
    let provenance = &rendition.companion.note_sources[0];
    assert_eq!(
        provenance.source_route,
        original.companion.note_sources[0].source_route
    );
    assert!(provenance.source_velocity.is_some());
    rendition.companion.validate_rendition(&decoded).unwrap();
    assert!(matches!(
        plan.rules[0].dynamics,
        adapt::Dynamics::Crossfade { .. }
    ));
}

#[test]
fn simultaneous_notes_keep_their_own_source_velocities_after_canonical_key_sorting() {
    use musicxml_midi2_convert::{xml_to_midi2, xml_to_rendition};
    let source=br#"<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Oboe</part-name></score-part></part-list><part id="P1"><measure number="1"><attributes><divisions>1</divisions></attributes><note dynamics="90"><pitch><step>E</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note><note dynamics="40"><chord/><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration><type>whole</type></note></measure></part></score-partwise>"#;
    let mut plan = adaptation_plan();
    plan.rendition.tracks.pop();
    plan.rendition.bindings.pop();
    plan.rules.pop();
    plan.rules[0].source_prefix = "/score-partwise/part[@id='P1']/".into();
    // Equal simultaneous channel dynamics are required; vary source attack while
    // giving both notes an explicit shared performed dynamic level.
    let source=std::str::from_utf8(source).unwrap().replace("<type>whole</type>",r#"<type>whole</type><play><other-play type="urn:midi2workbench:articulation:v2">{"technique":"core.normal","qualifiers":{"urn:midi2workbench:dynamic-level":2147483648}}</other-play></play>"#);
    plan.rules[0].descriptor = Some(musicxml_midi2_convert::model::Descriptor {
        technique: "core.normal".into(),
        qualifiers: std::collections::BTreeMap::new(),
    });
    let baseline = xml_to_midi2(source.as_bytes(), &RenderingPolicy::default()).unwrap();
    let original = clip::decode(&baseline.clip, Some(&baseline.companion.dictionary)).unwrap();
    let rendered = xml_to_rendition(source.as_bytes(), &RenderingPolicy::default(), &plan).unwrap();
    assert_ne!(original.notes[0].velocity, original.notes[1].velocity);
    for (note, provenance) in original.notes.iter().zip(&rendered.companion.note_sources) {
        assert_eq!(note.key, provenance.key);
        assert_eq!(provenance.source_velocity, Some(note.velocity));
        let source = &baseline
            .companion
            .note_sources
            .iter()
            .find(|source| source.key == note.key)
            .unwrap()
            .source;
        assert_eq!(&provenance.source, source);
    }
}

#[test]
fn articulation_display_names_are_validated_without_replacing_internal_ids() {
    let mut value = manifest();
    value["instruments"][0]["articulationName"] = json!("Sustain");
    let rendition: Rendition = serde_json::from_value(value.clone()).unwrap();
    rendition.validate().unwrap();
    assert_eq!(rendition.instruments[0].id, "oboe-sustain");
    for invalid in ["", "  ", "bad\0name"] {
        value["instruments"][0]["articulationName"] = json!(invalid);
        let rendition: Rendition = serde_json::from_value(value.clone()).unwrap();
        assert!(rendition.validate().is_err());
    }
}

#[test]
fn duplicate_articulation_labels_within_one_track_refuse_before_native_planning() {
    let mut value = manifest();
    value["instruments"][0]["articulationName"] = json!("Sustain");
    let mut second = value["instruments"][0].clone();
    second["id"] = json!("other-sustain");
    value["instruments"].as_array_mut().unwrap().push(second);
    value["bindings"][0]["channel"] = json!(1);
    value["bindings"].as_array_mut().unwrap().push(json!({"route":{"group":0,"channel":1},"track":"oboe1","channel":2,"instrument":"other-sustain"}));
    let rendition: Rendition = serde_json::from_value(value).unwrap();
    assert!(
        rendition
            .validate()
            .unwrap_err()
            .to_string()
            .contains("duplicate articulation display name")
    );
}

#[test]
fn explicit_slots_share_resources_and_admit_multiple_note_channels() {
    let mut value = manifest();
    value["bindings"][0]["channel"] = json!(1);
    value["bindings"][0]["slot"] = json!("first");
    value["bindings"][1]["slot"] = json!("second");
    value["slots"] = json!([
        {"id":"first","track":"oboe1","name":"Phrase one","instrument":"oboe-sustain","inputChannels":[1,2]},
        {"id":"second","track":"oboe2","name":"Phrase two","instrument":"oboe-sustain","inputChannels":[0]}
    ]);
    let rendition: Rendition = serde_json::from_value(value.clone()).unwrap();
    rendition.validate_performance(&performance()).unwrap();
    for (pointer, bad) in [
        ("/slots/0/inputChannels", json!([0, 1])),
        ("/slots/0/inputChannels", json!([1, 1])),
        ("/slots/0/inputChannels", json!([])),
        ("/bindings/0/slot", json!("second")),
        ("/bindings/0/channel", json!(3)),
    ] {
        let mut invalid = value.clone();
        *invalid.pointer_mut(pointer).unwrap() = bad;
        assert!(
            serde_json::from_value::<Rendition>(invalid)
                .unwrap()
                .validate()
                .is_err(),
            "{pointer}"
        );
    }
    value["bindings"].as_array_mut().unwrap().push(json!({"route":{"group":0,"channel":1},"track":"oboe1","channel":2,"instrument":"oboe-sustain","slot":"first"}));
    serde_json::from_value::<Rendition>(value)
        .unwrap()
        .validate()
        .unwrap();
}

#[test]
fn articulated_destinations_reserve_shared_control_channel_without_changing_ump_routes() {
    let mut value = manifest();
    value["bindings"][1]["track"] = json!("oboe1");
    value["bindings"][1]["channel"] = json!(1);
    value["tracks"].as_array_mut().unwrap().pop();
    assert!(
        serde_json::from_value::<Rendition>(value.clone())
            .unwrap()
            .validate()
            .unwrap_err()
            .message
            .contains("shared CC")
    );
    value["bindings"][0]["channel"] = json!(2);
    let rendition: Rendition = serde_json::from_value(value).unwrap();
    rendition.validate().unwrap();
    assert_eq!(rendition.bindings[0].route.channel, 0);
    assert_eq!(rendition.bindings[1].route.group, 1);
}
