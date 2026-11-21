use midi2::{ci::DeviceId, ux::u28};
use musicxml_midi2_convert::{
    ci::{self, Body, Message},
    extension::Dictionary,
    model::Descriptor,
    rules::PROFILE_ID,
};
use rstest::rstest;

fn message(body: Body) -> Message {
    let broadcast = matches!(
        body,
        Body::Report { .. } | Body::Added { .. } | Body::Removed { .. } | Body::Invalidate { .. }
    );
    let invalidate = matches!(body, Body::Invalidate { .. });
    let mut message = Message::channel(
        15,
        0x0123_4567,
        if broadcast {
            ci::BROADCAST
        } else {
            0x0076_5432
        },
        body,
    )
    .unwrap();
    if invalidate {
        message.address = DeviceId::FunctionBlock;
    }
    message
}

#[rstest]
#[case(Body::Inquiry)]
#[case(Body::Reply{enabled:vec![PROFILE_ID],disabled:vec![]})]
#[case(Body::Set{on:true,profile:PROFILE_ID,channels:1})]
#[case(Body::Set{on:false,profile:PROFILE_ID,channels:0})]
#[case(Body::Report{on:true,profile:PROFILE_ID,channels:1})]
#[case(Body::Report{on:false,profile:PROFILE_ID,channels:1})]
#[case(Body::Added{profile:PROFILE_ID})]
#[case(Body::Removed{profile:PROFILE_ID})]
#[case(Body::Details{profile:PROFILE_ID,target:1})]
#[case(Body::DetailsReply{profile:PROFILE_ID,target:0x40,data:vec![0;96]})]
#[case(Body::Specific{profile:PROFILE_ID,data:vec![1,2,3]})]
#[case(Body::Ack{negative:false,original:0x22,status:0,status_data:0,details:PROFILE_ID,data:vec![]})]
#[case(Body::Ack{negative:true,original:0x22,status:4,status_data:0,details:PROFILE_ID,data:b"unsupported".to_vec()})]
#[case(Body::Invalidate{muid:0x0123_4567})]
fn profile_family_codecs(#[case] body: Body) {
    let m = message(body);
    let data = m.encode().unwrap();
    assert_eq!(Message::decode(&data).unwrap(), m);
    for end in 0..data.len() {
        assert!(
            Message::decode(&data[..end]).is_err(),
            "accepted truncated family {:x} at {end}",
            m.status()
        );
    }
    let mut extra = data.clone();
    extra.push(0);
    assert!(Message::decode(&extra).is_err());
    let mut non7 = data;
    non7[5] |= 128;
    assert!(Message::decode(&non7).is_err());
}

#[test]
fn off_reserved_field_and_ack_details_follow_the_standard() {
    let mut off = message(Body::Set {
        on: false,
        profile: PROFILE_ID,
        channels: 0,
    })
    .encode()
    .unwrap();
    assert_eq!(&off[18..20], &[0, 0]);
    off[18] = 1;
    assert!(Message::decode(&off).is_err());
    let mut responder = ci::Engine::new(2).unwrap();
    let mut unknown = PROFILE_ID;
    unknown[2] = 2;
    let set = Message::channel(
        0,
        1,
        2,
        Body::Set {
            on: true,
            profile: unknown,
            channels: 1,
        },
    )
    .unwrap();
    let nak = responder.receive(&set).unwrap().remove(0).encode().unwrap();
    assert_eq!(&nak[13..21], &[0x22, 4, 0, 0x7e, 0x21, 2, 1, 1]);
}

#[test]
fn published_discovery_and_local_reply_interoperate() {
    let mut a = ci::Engine::new(1).unwrap();
    let mut b = ci::Engine::new(2).unwrap();
    a.identity.set_output_path_id(midi2::ux::u7::new(7));
    let reply = b.receive_payload(&a.discovery()).unwrap().remove(0);
    assert_eq!(&reply[..5], &[0x7e, 0x7f, 0x0d, 0x71, 2]);
    assert_eq!(reply[29], 7);
    assert_eq!(reply[30], 0x7f);
    assert!(a.receive_payload(&reply).unwrap().is_empty());
    let collision = b.receive_payload(&b.discovery()).unwrap();
    assert_eq!(collision.len(), 2);
    assert_ne!(u32::from(b.muid), 2);
    assert!(matches!(
        Message::decode(&collision[0]).unwrap().body,
        Body::Invalidate { muid: 2 }
    ));
}

fn function_block(body: Body, source: u32, dest: u32) -> Message {
    Message {
        address: DeviceId::FunctionBlock,
        version: 2,
        source: u28::new(source),
        destination: u28::new(dest),
        body,
    }
}
fn negotiated() -> (ci::Engine, ci::Engine, Dictionary) {
    let d = Descriptor {
        technique: "short.spiccato".into(),
        qualifiers: std::collections::BTreeMap::from([(
            "urn:midi2workbench:sample-character".into(),
            serde_json::json!("flûté 🎻".repeat(20)),
        )]),
    };
    let map = Dictionary::from_descriptors(std::iter::once(&d));
    let mut client = ci::Engine::new(1).unwrap();
    let mut server = ci::Engine::new(2).unwrap();
    client.identity.set_max_sysex_size(u28::new(512));
    client.set_extension_map(Dictionary::default()).unwrap();
    server.set_extension_map(map.clone()).unwrap();
    let discovery = server
        .receive_payload(&client.discovery())
        .unwrap()
        .remove(0);
    client.receive_payload(&discovery).unwrap();
    let request = function_block(
        Body::PeCapabilities {
            reply: false,
            simultaneous: 1,
            major: 0,
            minor: 0,
        },
        1,
        2,
    );
    let response = server.receive(&request).unwrap().remove(0);
    client.receive(&response).unwrap();
    (client, server, map)
}

#[test]
fn refinement_resource_is_ascii_chunked_and_schema_described() {
    let (mut client, mut server, map) = negotiated();
    for (id, resource, res_id) in [
        (1, "ResourceList", None),
        (2, "X-ArticulationMap", None),
        (3, "JSONSchema", Some("articulationDictionaryV1")),
    ] {
        let request = client
            .request_property(2, id, resource, res_id, 10_000)
            .unwrap();
        let replies = server.receive(&request).unwrap();
        if resource != "ResourceList" {
            assert!(replies.len() > 1);
        }
        for reply in &replies {
            let encoded = reply.encode().unwrap();
            assert!(encoded.iter().all(|b| *b < 128));
            assert!(encoded.len() + 2 <= 512);
            client.receive(&Message::decode(&encoded).unwrap()).unwrap();
        }
    }
    let received = &client.received_resources[&(2, 2)];
    assert_eq!(*received, map.property_data());
    assert_eq!(Dictionary::from_property_data(received).unwrap(), map);
    let schema = &client.received_resources[&(2, 3)];
    jsonschema::validator_for(schema)
        .unwrap()
        .validate(received)
        .unwrap();
    assert!(
        client.received_resources[&(2, 1)]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["resource"] == "X-ArticulationMap")
    );
}

#[test]
fn out_of_order_property_chunks_fail() {
    let (mut client, mut server, _) = negotiated();
    let request = client
        .request_property(2, 1, "JSONSchema", Some("articulationDictionaryV1"), 10_000)
        .unwrap();
    let replies = server.receive(&request).unwrap();
    assert!(replies.len() > 1);
    assert_eq!(
        client.receive(replies.last().unwrap()).unwrap_err().code,
        "E_PE_CHUNK"
    );
}

#[test]
fn normative_fallback_preserves_original_request() {
    let mut f = ci::Features::default();
    f.standard[0x10] = 1;
    f.standard[0x15] = 0;
    let original = musicxml_midi2_convert::rules::Articulation::named("short.spiccato").unwrap();
    let fallback = f.fallback(original).unwrap();
    assert_eq!((fallback.at, fallback.subclass), (0x11, 0));
    assert_eq!(original.subclass, 5);
    f.standard[0x10] = 0;
    assert_eq!(f.fallback(original).unwrap().at, 0x10);
}

#[test]
fn invalid_capabilities_do_not_mutate_cached_state() {
    let mut features = ci::Features::default();
    let original = features.payload(1).unwrap();
    let mut invalid = original.clone();
    invalid[2] = 17;
    assert!(features.apply(1, &invalid).is_err());
    assert_eq!(features.payload(1).unwrap(), original);
}

#[test]
fn wire_errors_receive_prescribed_nak_codes() {
    let mut engine = ci::Engine::new(2).unwrap();
    let mut request = Message::channel(0, 1, 2, Body::Inquiry)
        .unwrap()
        .encode()
        .unwrap();
    request[3] = 0x2a;
    let response = engine.receive_payload(&request).unwrap().remove(0);
    assert!(matches!(
        Message::decode(&response).unwrap().body,
        Body::Ack {
            negative: true,
            status: 1,
            ..
        }
    ));
    request[3] = 0x22; // Set On without its required profile and channel fields.
    let response = engine.receive_payload(&request).unwrap().remove(0);
    assert!(matches!(
        Message::decode(&response).unwrap().body,
        Body::Ack {
            negative: true,
            status: 0x41,
            ..
        }
    ));
}

#[test]
fn pe_rejects_unsolicited_replies_and_expired_transactions() {
    let (mut client, mut server, _) = negotiated();
    let inquiry = client
        .request_property(2, 1, "ResourceList", None, 100)
        .unwrap();
    let replies = server.receive(&inquiry).unwrap();
    client.expire(100);
    assert_eq!(
        client.receive(&replies[0]).unwrap_err().code,
        "E_PE_TRANSACTION"
    );
    client
        .request_property(2, 1, "ResourceList", None, 200)
        .unwrap();
    assert!(
        client
            .request_property(2, 2, "ResourceList", None, 200)
            .is_err()
    );
    for reply in replies {
        client.receive(&reply).unwrap();
    }
    assert!(client.received_resources.contains_key(&(2, 1)));
}

#[test]
fn unknown_chunk_count_is_declared_by_the_final_chunk() {
    let (mut client, mut server, _) = negotiated();
    let inquiry = client
        .request_property(2, 1, "ResourceList", None, 100)
        .unwrap();
    let mut replies = server.receive(&inquiry).unwrap();
    let last = replies.len() - 1;
    for (i, message) in replies.iter_mut().enumerate() {
        if let Body::Property { total, .. } = &mut message.body {
            if i != last {
                *total = 0;
            }
        }
        client.receive(message).unwrap();
    }
    assert!(client.received_resources.contains_key(&(2, 1)));
}

#[test]
fn optional_capabilities_require_a_live_transaction_and_program_changes_invalidate_them() {
    let mut a = ci::Engine::new(1).unwrap();
    let mut b = ci::Engine::new(2).unwrap();
    let reply = b.receive_payload(&a.begin_discovery()).unwrap().remove(0);
    a.receive_payload(&reply).unwrap();
    a.finish_discovery();
    let enable = a.request_enable(2, 0, 100).unwrap();
    let report = b.receive(&enable).unwrap().remove(0);
    a.receive(&report).unwrap();
    let source=b"<score-partwise version=\"4.0\"><part-list><score-part id=\"P\"><part-name>Violin</part-name></score-part></part-list><part id=\"P\"><measure number=\"1\"><note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><notations><articulations><spiccato/></articulations></notations></note></measure></part></score-partwise>";
    let performance = musicxml_midi2_convert::xml::read(
        source,
        &musicxml_midi2_convert::model::RenderingPolicy::default(),
    )
    .unwrap()
    .1;
    let route = musicxml_midi2_convert::model::Route::default();
    assert_eq!(
        a.require_standard(2, route, &performance).unwrap_err().code,
        "E_PROFILE_DETAILS_REQUIRED"
    );
    b.features.standard[0x15] = 1;
    let inquiry = a.request_details(2, 0, 1, 100).unwrap();
    let detail = b.receive(&inquiry).unwrap().remove(0);
    a.receive(&detail).unwrap();
    a.require_standard(2, route, &performance).unwrap();
    assert_eq!(a.receive(&detail).unwrap_err().code, "E_CI_TRANSACTION");
    a.invalidate_program(2, 0);
    assert_eq!(
        a.require_standard(2, route, &performance).unwrap_err().code,
        "E_PROFILE_DETAILS_REQUIRED"
    );
    let other_group = musicxml_midi2_convert::model::Route { group: 1, ..route };
    assert_eq!(
        a.require_standard(2, other_group, &performance)
            .unwrap_err()
            .code,
        "E_PROFILE_GROUP"
    );
}

#[test]
fn acknowledgements_extend_timeouts_without_claiming_enablement() {
    let mut a = ci::Engine::new(1).unwrap();
    a.request_enable(2, 0, 100).unwrap();
    let wait = Message::channel(
        0,
        2,
        1,
        Body::Ack {
            negative: false,
            original: 0x22,
            status: 0x10,
            status_data: 5,
            details: PROFILE_ID,
            data: vec![],
        },
    )
    .unwrap();
    a.receive_at(&wait, 80).unwrap();
    assert!(a.expire(579).is_empty());
    assert_eq!(a.expire(580), vec![(2, 0)]);
    assert!(!a.remote.get(&(2, 0)).copied().unwrap_or(false));
}

#[test]
fn invalidation_and_discovery_collisions_use_injected_entropy() {
    use rand::SeedableRng;
    let mut a = ci::Engine::new(1).unwrap();
    a.enabled.insert(0);
    let invalidation = function_block(Body::Invalidate { muid: 1 }, 2, ci::BROADCAST)
        .encode()
        .unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let messages = a.receive_payload_with_rng(&invalidation, &mut rng).unwrap();
    assert_eq!(messages.len(), 1);
    assert_ne!(u32::from(a.muid), 1);
    assert_eq!(messages[0][3], 0x70);
    assert!(a.enabled.contains(&0));
    let mut client = ci::Engine::new(3).unwrap();
    let mut peer = ci::Engine::new(4).unwrap();
    let query = client.begin_discovery();
    let reply = peer.receive_payload(&query).unwrap().remove(0);
    client.receive_payload(&reply).unwrap();
    let collision = client.receive_payload(&reply).unwrap();
    assert!(matches!(
        Message::decode(&collision[0]).unwrap().body,
        Body::Invalidate { muid: 4 }
    ));
}

#[rstest]
#[case::channel(DeviceId::Channel(midi2::ux::u4::new(5)), vec![5])]
#[case::group(DeviceId::Group, (0..16).collect())]
#[case::function_block(DeviceId::FunctionBlock, (0..16).collect())]
fn profile_inquiries_report_each_addressed_channel(
    #[case] address: DeviceId,
    #[case] channels: Vec<u8>,
) {
    for (mut engine, profile) in [
        (ci::Engine::new(2).unwrap(), PROFILE_ID),
        (
            ci::Engine::new_piano(2).unwrap(),
            musicxml_midi2_convert::piano::PROFILE_ID,
        ),
    ] {
        engine.enabled.insert(5);
        let mut query = Message::channel(5, 1, 2, Body::Inquiry).unwrap();
        query.address = address;
        let replies = engine.receive(&query).unwrap();
        assert_eq!(replies.len(), channels.len());
        for (reply, channel) in replies.iter().zip(&channels) {
            assert_eq!(
                reply.address,
                DeviceId::Channel(midi2::ux::u4::new(*channel))
            );
            assert_eq!(
                (u32::from(reply.source), u32::from(reply.destination)),
                (2, 1)
            );
            let (enabled, disabled) = if *channel == 5 {
                (vec![profile], vec![])
            } else {
                (vec![], vec![profile])
            };
            assert_eq!(reply.body, Body::Reply { enabled, disabled });
        }
    }
}

#[test]
fn all_profile_state_notifications_and_disable_flow_have_explicit_effects() {
    let mut a = ci::Engine::new(1).unwrap();
    let mut b = ci::Engine::new(2).unwrap();
    let request = a.request_enable(2, 0, 100).unwrap();
    a.receive(&b.receive(&request).unwrap().remove(0)).unwrap();
    assert!(a.remote[&(2, 0)]);
    let off = Message::channel(
        0,
        1,
        2,
        Body::Set {
            on: false,
            profile: PROFILE_ID,
            channels: 0,
        },
    )
    .unwrap();
    a.receive(&b.receive(&off).unwrap().remove(0)).unwrap();
    assert!(!a.remote[&(2, 0)]);
    for body in [
        Body::Added {
            profile: PROFILE_ID,
        },
        Body::Removed {
            profile: PROFILE_ID,
        },
    ] {
        let report = Message::channel(0, 2, ci::BROADCAST, body).unwrap();
        a.receive(&report).unwrap();
    }
    assert!(!a.remote.contains_key(&(2, 0)));
    let specific = Message::channel(
        0,
        1,
        2,
        Body::Specific {
            profile: PROFILE_ID,
            data: vec![1],
        },
    )
    .unwrap();
    assert!(matches!(
        b.receive(&specific).unwrap()[0].body,
        Body::Ack {
            negative: true,
            status: 1,
            ..
        }
    ));
    assert!(
        Message::channel(0, 1, ci::BROADCAST, Body::Inquiry)
            .unwrap()
            .encode()
            .is_err()
    );
}

#[test]
fn generated_flex_meter_keeps_eight_thirty_seconds_per_quarter_note() {
    // Independent M2-104 v1.1.2 §7.5.4 / Figure 71 vectors; 24 clocks equal
    // one quarter note, containing eight 1/32 notes, regardless of meter.
    assert_eq!(
        musicxml_midi2_convert::wire::meter(4, 2),
        vec![0xd010_0001, 0x0402_0800, 0, 0]
    );
    assert_eq!(
        musicxml_midi2_convert::wire::meter(7, 3),
        vec![0xd010_0001, 0x0703_0800, 0, 0]
    );
}
