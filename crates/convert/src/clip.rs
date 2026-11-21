//! M2-116 clip framing with a separate decoder and exact score clock.
use crate::{
    Diagnostic, Result, ci,
    extension::Dictionary,
    model::{Event, Note, Performance, Rational, Report, Route, round},
    rules::{Articulation, PROFILE_ID},
    wire,
};
use midi2::prelude::*;
use num_integer::Integer;
use std::collections::{BTreeMap, BTreeSet};

const START: [u32; 4] = [0xf020_0000, 0, 0, 0];
const END: [u32; 4] = [0xf021_0000, 0, 0, 0];
const MARKER: &str = "urn:midi2workbench:refinement:v2:";

/// # Errors
/// Returns a diagnostic for invalid framing, unsupported events, clock limits or unresolved refinements.
pub fn tpq(score: &Performance) -> Result<u16> {
    let mut ticks = 1_i64;
    let mut times = score
        .notes
        .iter()
        .flat_map(|n| [n.onset, n.end])
        .chain(score.events.iter().map(|e| e.time))
        .chain(score.configuration.iter().map(|e| e.time))
        .chain(std::iter::once(score.duration))
        .collect::<Vec<_>>();
    for n in &score.notes {
        if let Some(d) = &n.descriptor {
            for (offset, ..) in crate::extension::dynamic_curve(d)? {
                times.push(n.onset + offset);
            }
        }
    }
    for t in times {
        ticks = ticks
            .checked_mul(*t.denom() / ticks.gcd(t.denom()))
            .ok_or_else(|| Diagnostic::new("E_TPQ", "clock overflow"))?;
        if ticks > 65535 {
            return Err(Diagnostic::new(
                "E_TPQ",
                format!("exact clock requires {ticks} ticks per quarter"),
            ));
        }
    }
    let preferred = ticks.checked_mul(480 / ticks.gcd(&480));
    u16::try_from(preferred.filter(|t| *t <= 65535).unwrap_or(ticks))
        .map_err(|_| Diagnostic::new("E_TPQ", "clock out of range"))
}
fn append(out: &mut Vec<u8>, words: &[u32]) {
    for word in words {
        out.extend(word.to_be_bytes());
    }
}
fn stamp(out: &mut Vec<u8>, delta: u64) {
    let mut remaining = delta;
    let mut message = midi2::utility::DeltaClockstamp::<[u32; 1]>::new();
    while remaining > 0xfffff {
        message.set_time_data(midi2::ux::u20::new(0xfffff));
        append(out, message.data());
        remaining -= 0xfffff;
    }
    message.set_time_data(midi2::ux::u20::new(
        u32::try_from(remaining).expect("20-bit remainder"),
    ));
    append(out, message.data());
}

/// Source ownership projected through encoder route isolation. Timing remains exact beats.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NoteSource {
    pub source: String,
    pub source_route: Route,
    /// Original attack before explicit receiver adaptation, when it changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_velocity: Option<u16>,
    pub encoded_route: Route,
    pub onset: Rational,
    pub end: Rational,
    pub key: u8,
}
/// Encoded artifacts plus ownership in the decoder's canonical note order.
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub dictionary: Dictionary,
    pub report: Report,
    pub note_sources: Vec<NoteSource>,
    /// Original performance-note indices in the same canonical order as `note_sources`.
    pub source_indices: Vec<usize>,
}
/// Encode without requesting source-ownership metadata.
/// # Errors
/// Returns invalid performance, clock, control and route diagnostics.
pub fn encode(score: &Performance) -> Result<(Vec<u8>, Dictionary, Report)> {
    let encoded = encode_with_sources(score)?;
    Ok((encoded.bytes, encoded.dictionary, encoded.report))
}

/// # Errors
/// Returns a diagnostic for invalid framing, unsupported events, clock limits or unresolved refinements.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
/// # Panics
/// Panics only if validated address and field-width invariants are inconsistent.
pub fn encode_with_sources(score: &Performance) -> Result<Encoded> {
    let ticks = tpq(score)?;
    let dictionary =
        Dictionary::from_descriptors(score.notes.iter().filter_map(|n| n.descriptor.as_ref()));
    dictionary.validate()?;
    let mut report = score.report.clone();
    let mut note_sources = Vec::with_capacity(score.notes.len());
    let mut schedule = Vec::<(Rational, u32, u8, usize, Vec<u32>)>::new();
    for (i, e) in score.events.iter().enumerate() {
        wire::validate_event(&e.words, false)?;
        if e.time < Rational::from_integer(0) {
            return Err(Diagnostic::new("E_TIME", "negative controller onset"));
        }
        schedule.push((e.time, e.order, 1, i, e.words.clone()));
    }
    let mut notes = score.notes.iter().enumerate().collect::<Vec<_>>();
    notes.sort_by_key(|(i, n)| (n.onset, n.on_order, *i));
    let mut busy = BTreeMap::<(Route, u8), Rational>::new();
    let mut mute_busy = BTreeMap::<Route, Vec<(Rational, serde_json::Value)>>::new();
    let mut last_mute = BTreeMap::<Route, serde_json::Value>::new();
    let uses_mute = score.notes.iter().any(|n| {
        n.descriptor
            .as_ref()
            .is_some_and(|d| d.qualifiers.contains_key("urn:midi2workbench:mute"))
    });
    let uses_position = score.notes.iter().any(|n| {
        n.descriptor
            .as_ref()
            .is_some_and(|d| d.qualifiers.contains_key("urn:midi2workbench:position"))
    });
    let reserved = score
        .notes
        .iter()
        .map(|n| n.route)
        .chain(score.piano_routes.iter().copied())
        .chain(
            score
                .configuration
                .iter()
                .chain(&score.events)
                .filter_map(|e| wire::route(&e.words)),
        )
        .collect::<BTreeSet<_>>();
    let mut projections = BTreeMap::<Route, BTreeSet<Route>>::new();
    // A channel's profile remains meaningful during rests and before its first
    // attack. Keep declared Piano routes even when this clip has no notes there.
    let mut routes = score.piano_routes.clone();
    let mut piano_routes = score.piano_routes.clone();
    for (i, n) in notes {
        if n.onset < Rational::from_integer(0)
            || n.end <= n.onset
            || n.pitch < Rational::from_integer(0)
            || n.pitch >= Rational::from_integer(128)
            || n.route.group > 15
            || n.route.channel > 15
            || n.key > 127
        {
            return Err(Diagnostic::new(
                "E_NOTE",
                format!("invalid lifecycle at {}", n.source),
            ));
        }
        let mut route = n.route;
        let mute = n
            .descriptor
            .as_ref()
            .and_then(|d| d.qualifiers.get("urn:midi2workbench:mute"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"slot":0,"amount":u32::MAX}));
        let compatible = |r: &Route| {
            busy.get(&(*r, n.key)).is_none_or(|end| *end <= n.onset)
                && mute_busy.get(r).is_none_or(|notes| {
                    notes
                        .iter()
                        .all(|(end, state)| *end <= n.onset || *state == mute)
                })
        };
        if !compatible(&route) {
            route = (0..16)
                .map(|channel| Route { channel, ..route })
                .filter(|r| *r == n.route || !reserved.contains(r))
                .find(compatible)
                .ok_or_else(|| {
                    Diagnostic::new(
                        "E_ROUTE",
                        format!("no same-key isolation slot for {}", n.source),
                    )
                })?;
            report.record(
                &n.source,
                "same-key isolation",
                format!("channel {}", route.channel),
            );
        }
        routes.insert(route);
        projections.entry(n.route).or_default().insert(route);
        note_sources.push((
            i,
            NoteSource {
                source: n.source.clone(),
                source_route: n.route,
                source_velocity: None,
                encoded_route: route,
                onset: n.onset,
                end: n.end,
                key: n.key,
            },
        ));
        busy.insert((route, n.key), n.end);
        let held = mute_busy.entry(route).or_default();
        held.retain(|(end, _)| *end > n.onset);
        held.push((n.end, mute.clone()));
        if score.piano_routes.contains(&n.route) {
            piano_routes.insert(route);
        }
        if uses_mute && last_mute.get(&route) != Some(&mute) {
            let slot = mute
                .get("slot")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| Diagnostic::new("E_MUTE", "missing mute slot"))?;
            if slot > 31 || (16..26).contains(&slot) {
                return Err(Diagnostic::new("E_MUTE", "reserved mute slot"));
            }
            let amount = mute
                .get("amount")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(u64::from(u32::MAX));
            if amount > u64::from(u32::MAX) {
                return Err(Diagnostic::new("E_MUTE", "amount out of range"));
            }
            if score.piano_routes.contains(&n.route) {
                if !matches!(slot, 0..=2) || ![0, u64::from(u32::MAX)].contains(&amount) {
                    return Err(Diagnostic::new(
                        "E_PIANO_MUTE",
                        "MusicXML brass/string mute type cannot be translated into a piano sordino",
                    ));
                }
                let message = wire::registered_channel(
                    route,
                    0x41,
                    0x20,
                    if slot == 0 || amount == 0 {
                        0
                    } else {
                        u32::MAX
                    },
                );
                if !score
                    .events
                    .iter()
                    .any(|e| e.time == n.onset && e.words == message)
                {
                    schedule.push((n.onset, n.on_order, 1, i, message));
                }
            } else {
                let type_word = wire::registered_channel(
                    route,
                    0x20,
                    0x22,
                    u32::try_from(slot).expect("validated mute slot") << 27,
                );
                if !score
                    .events
                    .iter()
                    .any(|e| e.time == n.onset && e.words == type_word)
                {
                    schedule.push((n.onset, n.on_order, 1, i, type_word));
                    // M2-123 mute type selection resets amount to its maximum.
                    if amount != u64::from(u32::MAX) {
                        schedule.push((
                            n.onset,
                            n.on_order,
                            1,
                            i,
                            wire::registered_channel(
                                route,
                                0x20,
                                0x23,
                                u32::try_from(amount).expect("validated mute amount"),
                            ),
                        ));
                    }
                }
            }
            last_mute.insert(route, mute);
        }
        let wide =
            num_rational::Ratio::new(i128::from(*n.pitch.numer()), i128::from(*n.pitch.denom()));
        let pitch = u32::try_from(
            (wide * num_rational::Ratio::from_integer(1_i128 << 25))
                .round()
                .to_integer()
                .clamp(0, i128::from(u32::MAX)),
        )
        .map_err(|_| Diagnostic::new("E_PITCH", "Pitch 7.25 overflow"))?;
        let encoded = Rational::new(i64::from(pitch), 1 << 25);
        report.quantized(
            &n.source,
            "pitch/7.25",
            wide * num_rational::Ratio::from_integer(1_i128 << 25),
            pitch,
        );
        if encoded != n.pitch {
            report.record(
                &n.source,
                "Pitch 7.25 rounding",
                format!("{} -> {}", n.pitch, encoded),
            );
        }
        schedule.push((n.onset, n.on_order, 2, i, wire::detach_reset(route, n.key)));
        schedule.push((
            n.onset,
            n.on_order,
            2,
            i,
            wire::controller(route, n.key, 3, pitch, true),
        ));
        if uses_position {
            let position = n
                .descriptor
                .as_ref()
                .and_then(|d| d.qualifiers.get("urn:midi2workbench:position"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0x8000_0000);
            let position = u32::try_from(position)
                .map_err(|_| Diagnostic::new("E_POSITION", "position out of range"))?;
            schedule.push((
                n.onset,
                n.on_order,
                2,
                i,
                wire::controller(route, n.key, 12, position, true),
            ));
        }
        if !dictionary.descriptors.is_empty() {
            let token = n
                .descriptor
                .as_ref()
                .map_or(Ok(0), |d| dictionary.token(d))?;
            schedule.push((
                n.onset,
                n.on_order,
                3,
                i,
                wire::controller(route, n.key, dictionary.controller_index, token, false),
            ));
        }
        if dictionary.parameters.contains_key(&126) {
            let curve = n
                .descriptor
                .as_ref()
                .map_or_else(|| Ok(vec![]), crate::extension::dynamic_curve)?;
            let curve = if curve.is_empty() {
                vec![(
                    Rational::from_integer(0),
                    crate::rules::upscale(u32::from(n.velocity), 16, 32),
                    0,
                )]
            } else {
                curve
            };
            let mut previous = None;
            for (offset, value, order) in curve {
                if offset < Rational::from_integer(0)
                    || offset > n.end - n.onset
                    || previous.is_some_and(|p| p >= (offset, order))
                {
                    return Err(Diagnostic::new(
                        "E_DYNAMIC_CURVE",
                        "curve points must be ordered within note lifetime",
                    ));
                }
                previous = Some((offset, order));
                schedule.push((
                    n.onset + offset,
                    if offset == Rational::from_integer(0) && order == 0 {
                        n.on_order
                    } else {
                        order
                    },
                    if offset == Rational::from_integer(0) {
                        3
                    } else {
                        0
                    },
                    i,
                    wire::controller(route, n.key, 126, value, false),
                ));
            }
        }
        let key_velocity = n
            .descriptor
            .as_ref()
            .and_then(|d| d.qualifiers.get("urn:midi2workbench:piano-key-velocity"))
            .and_then(serde_json::Value::as_u64);
        let piano_attack =
            score.piano_routes.contains(&n.route) && n.articulation == Articulation::default();
        if key_velocity.is_some() && !piano_attack {
            return Err(Diagnostic::new(
                "E_ATTRIBUTE_CONFLICT",
                "Piano key velocity and an orchestral articulation cannot share one Attribute slot",
            ));
        }
        let attack = if piano_attack {
            let key_velocity = key_velocity
                .map(u16::try_from)
                .transpose()
                .map_err(|_| Diagnostic::new("E_KEY_VELOCITY", "outside 16 bits"))?;
            crate::piano::note(route, n.key, n.velocity, key_velocity, true)?
        } else {
            wire::note(route, n.key, n.velocity, n.articulation, true)?
        };
        schedule.push((n.onset, n.on_order, 4, i, attack));
        schedule.push((
            n.end,
            n.off_order,
            0,
            i,
            if score.piano_routes.contains(&n.route)
                && n.articulation.release == 0
                && n.articulation.string == 0
            {
                crate::piano::note(route, n.key, n.release_velocity, None, false)?
            } else {
                wire::note(route, n.key, n.release_velocity, n.articulation, false)?
            },
        ));
    }
    // Channel state follows automatically isolated voices; source note-instance
    // controls need an explicit binding rather than an ambiguous broadcast.
    for (index, event) in score.events.iter().enumerate() {
        if let Some(source) = wire::route(&event.words) {
            if let Some(targets) = projections.get(&source) {
                for &target in targets.iter().filter(|r| **r != source) {
                    let status = (event.words[0] >> 20) & 15;
                    if event.words[0] >> 28 == 4 && matches!(status, 0 | 1 | 6 | 8 | 9 | 10 | 15) {
                        return Err(Diagnostic::new(
                            "E_ROUTE_CONTROL_BINDING",
                            "isolated note-instance controls require an explicit association",
                        ));
                    }
                    schedule.push((
                        event.time,
                        event.order,
                        1,
                        index,
                        wire::reroute(&event.words, target)?,
                    ));
                }
            }
        }
    }
    schedule.sort_by_key(|(time, order, priority, index, _)| (*time, *order, *priority, *index));
    let mut out = b"SMF2CLIP".to_vec();
    for route in routes {
        let needs_articulation = score.notes.iter().any(|n| {
            projections
                .get(&n.route)
                .is_some_and(|targets| targets.contains(&route))
                && n.needs_orchestral_profile()
        });
        if !piano_routes.contains(&route) || needs_articulation {
            append(
                &mut out,
                &ci::file_profile_declaration(route.group, route.channel)?,
            );
        }
        if piano_routes.contains(&route) {
            append(
                &mut out,
                &ci::file_profile(route.group, route.channel, crate::piano::PROFILE_ID)?,
            );
        }
    }
    stamp(&mut out, 0);
    let mut clock_message = midi2::utility::DeltaClockstampTpq::<[u32; 1]>::new();
    clock_message.set_time_data(ticks);
    append(&mut out, clock_message.data());
    let mut configuration_clock = 0_u64;
    let mut configuration = Vec::new();
    for event in &score.configuration {
        configuration.push(event.clone());
        if let Some(source) = wire::route(&event.words) {
            if let Some(targets) = projections.get(&source) {
                for &target in targets.iter().filter(|r| **r != source) {
                    configuration.push(Event {
                        words: wire::reroute(&event.words, target)?,
                        ..event.clone()
                    });
                }
            }
        }
    }
    let mut configuration_phase = 0;
    for e in &configuration {
        configuration_order(&e.words, e.time, &mut configuration_phase)?;
        wire::validate_event(&e.words, true)?;
        if e.time < Rational::from_integer(0) {
            return Err(Diagnostic::new("E_TIME", "negative configuration time"));
        }
        let tick = u64::try_from(round(e.time * Rational::from_integer(i64::from(ticks))))
            .map_err(|_| Diagnostic::new("E_TIME", "configuration time out of range"))?;
        if tick < configuration_clock {
            return Err(Diagnostic::new(
                "E_TIME",
                "configuration events out of order",
            ));
        }
        stamp(&mut out, tick - configuration_clock);
        append(&mut out, &e.words);
        configuration_clock = tick;
    }
    if !dictionary.descriptors.is_empty() {
        let mut marker = midi2::flex_data::UnknownMetadataText::<Vec<u32>>::new();
        marker.set_text(&format!(
            "{MARKER}{}",
            serde_json::to_string(&dictionary)
                .map_err(|e| Diagnostic::new("E_EXTENSION_MAP", e.to_string()))?
        ));
        stamp(&mut out, 0);
        append(&mut out, marker.data());
    }
    stamp(&mut out, 0);
    append(
        &mut out,
        midi2::ump_stream::StartOfClip::<[u32; 4]>::new().data(),
    );
    let mut clock = 0_u64;
    for (time, _, _, _, message) in schedule {
        let tick = u64::try_from(round(time * Rational::from_integer(i64::from(ticks))))
            .map_err(|_| Diagnostic::new("E_TIME", "sequence time out of range"))?;
        stamp(&mut out, tick - clock);
        append(&mut out, &message);
        clock = tick;
    }
    let duration = u64::try_from(round(
        score.duration * Rational::from_integer(i64::from(ticks)),
    ))
    .map_err(|_| Diagnostic::new("E_TIME", "performance duration out of range"))?;
    stamp(&mut out, duration.max(clock) - clock);
    append(
        &mut out,
        midi2::ump_stream::EndOfClip::<[u32; 4]>::new().data(),
    );
    note_sources.sort_by_key(|(_, n)| (n.onset, n.encoded_route, n.key, n.end));
    let (source_indices, note_sources) = note_sources.into_iter().unzip();
    Ok(Encoded {
        bytes: out,
        dictionary,
        report,
        note_sources,
        source_indices,
    })
}

// M2-116 §§6.1.1–6.1.2: configuration tempo/meter precede receiver setup.
fn configuration_order(words: &[u32], time: Rational, phase: &mut u8) -> Result<()> {
    let header = words[0];
    if header >> 28 == 0xd && header.trailing_zeros() >= 16 {
        if *phase != 0 || time != Rational::from_integer(0) {
            return Err(Diagnostic::new(
                "E_CLIP_CONFIGURATION",
                "configuration tempo must be first at clock zero",
            ));
        }
        *phase = 1;
    } else if header >> 28 == 0xd && header & 0xffff == 1 {
        if *phase != 1 || time != Rational::from_integer(0) {
            return Err(Diagnostic::new(
                "E_CLIP_CONFIGURATION",
                "configuration meter must immediately follow tempo at clock zero",
            ));
        }
        *phase = 2;
    } else {
        *phase = 3;
    }
    Ok(())
}

/// Parse packet lengths before indexing, reassemble SysEx/text, and account notes.
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for invalid framing, unsupported events, clock limits or unresolved refinements.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
pub fn decode(bytes: &[u8], dictionary: Option<&Dictionary>) -> Result<Performance> {
    if !bytes.starts_with(b"SMF2CLIP") || !(bytes.len() - 8).is_multiple_of(4) {
        return Err(Diagnostic::new(
            "E_CLIP_HEADER",
            "expected complete SMF2CLIP words",
        ));
    }
    if let Some(d) = dictionary {
        d.validate()?;
    }
    let mut dictionary = dictionary.cloned();
    let words = bytes[8..]
        .chunks_exact(4)
        .map(|b| u32::from_be_bytes(b.try_into().expect("four bytes")))
        .collect::<Vec<_>>();
    let mut out = Performance::default();
    let mut at = 0;
    let mut ticks = None;
    let mut clock = 0_i64;
    let mut started = false;
    let mut ended = false;
    let mut stamped = false;
    let mut sysex = BTreeMap::<u8, Vec<u8>>::new();
    let mut text = BTreeMap::<u32, (Rational, bool, Vec<u32>)>::new();
    let mut pitch = BTreeMap::<(Route, u8), Rational>::new();
    let mut tokens = BTreeMap::<(Route, u8), u32>::new();
    let mut positions = BTreeMap::<(Route, u8), u32>::new();
    let mut dynamic_state = BTreeMap::<(Route, u8), u32>::new();
    let mut dynamic_curves = BTreeMap::<(Route, u8), Vec<(Rational, u32, u32)>>::new();
    let mut active = BTreeMap::<(Route, u8), Note>::new();
    let mut refinement = false;
    let mut ordinal = 0u32;
    let mut configuration_phase = 0;
    while at < words.len() {
        let first = words[at];
        let len = wire::packet_len(first);
        let packet = words
            .get(at..at + len)
            .ok_or_else(|| Diagnostic::new("E_UMP", "truncated packet"))?;
        at += len;
        if ended {
            return Err(Diagnostic::new("E_CLIP_END", "data after End of Clip"));
        }
        if first & 0xfff0_0000 == 0x0040_0000 {
            if first & 0xfffff != 0 {
                ordinal = 0;
            }
            clock = clock
                .checked_add(i64::from(first & 0xfffff))
                .ok_or_else(|| Diagnostic::new("E_TIME", "clock overflow"))?;
            stamped = true;
            continue;
        }
        if let Some(ticks) = ticks {
            if !started && packet != START {
                configuration_order(
                    packet,
                    Rational::new(clock, i64::from(ticks)),
                    &mut configuration_phase,
                )?;
            }
        }
        if first >> 28 == 3 {
            if ticks.is_some() && !stamped {
                return Err(Diagnostic::new(
                    "E_CLIP_CLOCK",
                    "SysEx has no Delta Clockstamp",
                ));
            }
            let group = ((first >> 24) & 15) as u8;
            let form = ((first >> 20) & 15) as u8;
            let count = ((first >> 16) & 15) as usize;
            if count > 6 || form > 3 {
                return Err(Diagnostic::new("E_SYSEX", "invalid SysEx7 header"));
            }
            let raw = packet
                .iter()
                .flat_map(|w| w.to_be_bytes())
                .collect::<Vec<_>>();
            let data = &raw[2..2 + count];
            if data.iter().any(|v| *v > 127) {
                return Err(Diagnostic::new("E_SYSEX", "non-7bit data"));
            }
            if raw[2 + count..].iter().any(|b| *b != 0) {
                return Err(Diagnostic::new("E_SYSEX", "nonzero unused bytes"));
            }
            if let Some(ticks) = ticks {
                if started {
                    ordinal += 1;
                }
                let event = Event {
                    order: if started { ordinal } else { 0 },
                    time: Rational::new(clock, i64::from(ticks)),
                    words: packet.to_vec(),
                };
                if started {
                    out.events.push(event);
                } else {
                    out.configuration.push(event);
                }
            }
            if form == 0 || form == 1 {
                if sysex.contains_key(&group) {
                    return Err(Diagnostic::new("E_SYSEX", "unclosed message"));
                }
                sysex.insert(group, Vec::new());
            }
            sysex
                .get_mut(&group)
                .ok_or_else(|| Diagnostic::new("E_SYSEX", "continuation without start"))?
                .extend(data);
            if form == 0 || form == 3 {
                let payload = sysex.remove(&group).expect("open message");
                if ticks.is_none() {
                    if stamped || payload.len() < 13 || payload[5..13].iter().any(|b| *b != 127) {
                        return Err(Diagnostic::new(
                            "E_CLIP_PROFILE",
                            "file profile declarations require both broadcast MUIDs and no preceding clock",
                        ));
                    }
                    let mut decoded = payload.clone();
                    decoded[5..13].fill(0);
                    let message = ci::Message::decode(&decoded)?;
                    if let ci::Body::Set {
                        on: true,
                        profile,
                        channels: 1,
                    } = message.body
                    {
                        if profile == crate::piano::PROFILE_ID {
                            if let midi2::ci::DeviceId::Channel(channel) = message.address {
                                out.piano_routes.insert(Route {
                                    group,
                                    channel: channel.into(),
                                });
                            } else {
                                return Err(Diagnostic::new(
                                    "E_CLIP_PROFILE",
                                    "Piano is a single-channel profile",
                                ));
                            }
                        } else if profile != PROFILE_ID {
                            return Err(Diagnostic::new(
                                "E_CLIP_PROFILE",
                                "unsupported standard profile",
                            ));
                        }
                    } else {
                        return Err(Diagnostic::new(
                            "E_CLIP_PROFILE",
                            "unsupported or misplaced profile declaration",
                        ));
                    }
                } else if payload.first() == Some(&0x7e)
                    && payload.get(2) == Some(&0x0d)
                    && payload.get(3).is_some_and(|s| (0x30..=0x3f).contains(s))
                {
                    return Err(Diagnostic::new(
                        "E_CLIP_PE",
                        "Property Exchange is forbidden in clips",
                    ));
                }
                if ticks.is_some()
                    && payload.first() == Some(&0x7e)
                    && payload.get(2) == Some(&0x0d)
                    && payload.get(3) == Some(&0x22)
                    && payload.get(5..9) == Some(&[127, 127, 127, 127])
                {
                    return Err(Diagnostic::new(
                        "E_CLIP_PROFILE",
                        "file profile declaration must precede the clock declaration",
                    ));
                }
            }
            continue;
        }
        if !stamped {
            return Err(Diagnostic::new(
                "E_CLIP_CLOCK",
                "message has no Delta Clockstamp",
            ));
        }
        if first & 0xffff_0000 == 0x0030_0000 {
            if ticks.is_some()
                || started
                || clock != 0
                || u16::from_le_bytes([first.to_le_bytes()[0], first.to_le_bytes()[1]]) == 0
            {
                return Err(Diagnostic::new("E_TPQ", "invalid clock declaration"));
            }
            ticks = Some(u16::from_le_bytes([
                first.to_le_bytes()[0],
                first.to_le_bytes()[1],
            ]));
            continue;
        }
        if packet == START {
            if started || ticks.is_none() {
                return Err(Diagnostic::new(
                    "E_CLIP_START",
                    "missing clock or duplicate start",
                ));
            }
            started = true;
            clock = 0;
            ordinal = 0;
            stamped = false;
            continue;
        }
        if packet == END {
            if !started || !active.is_empty() || !sysex.is_empty() || !text.is_empty() {
                return Err(Diagnostic::new(
                    "E_CLIP_END",
                    "unbalanced notes/messages or missing start",
                ));
            }
            out.duration = Rational::new(clock, i64::from(ticks.expect("start requires clock")));
            ended = true;
            continue;
        }
        if first == 0 {
            continue;
        }
        if first >> 28 == 0xd && (first >> 8) & 255 != 0 {
            let form = (first >> 22) & 3;
            let key = first & !0x00c0_0000;
            let time =
                Rational::new(
                    clock,
                    i64::from(ticks.ok_or_else(|| {
                        Diagnostic::new("E_TPQ", "text precedes clock declaration")
                    })?),
                );
            if form < 2 && text.insert(key, (time, started, Vec::new())).is_some() {
                return Err(Diagnostic::new("E_FLEX", "overlapping text message"));
            }
            let (initial_time, initial_phase, buffer) = text
                .get_mut(&key)
                .ok_or_else(|| Diagnostic::new("E_FLEX", "continuation without text start"))?;
            if *initial_time != time || *initial_phase != started {
                return Err(Diagnostic::new(
                    "E_FLEX",
                    "text packets cross a clock or sequence boundary",
                ));
            }
            buffer.extend(packet);
            if form == 0 || form == 3 {
                let (_, _, buffer) = text.remove(&key).expect("open text");
                wire::validate(&buffer)?;
                let marker = if !started && first & 0xffff == 0x0100 {
                    Some(
                        crate::remarks::flex_text(&buffer)
                            .ok_or_else(|| Diagnostic::new("E_FLEX", "Invalid metadata text"))?,
                    )
                } else {
                    None
                };
                if let Some(value) = marker.as_deref().and_then(|s| s.strip_prefix(MARKER)) {
                    let embedded = serde_json::from_str::<Dictionary>(value)
                        .map_err(|e| Diagnostic::new("E_EXTENSION_MAP", e.to_string()))?;
                    embedded.validate()?;
                    if dictionary.as_ref().is_some_and(|d| *d != embedded) {
                        return Err(Diagnostic::new(
                            "E_EXTENSION_MAP",
                            "dictionary entries differ",
                        ));
                    }
                    dictionary = Some(embedded);
                    refinement = true;
                } else {
                    if started {
                        ordinal += 1;
                    }
                    let event = Event {
                        order: if started { ordinal } else { 0 },
                        time,
                        words: buffer,
                    };
                    if started {
                        out.events.push(event);
                    } else {
                        out.configuration.push(event);
                    }
                }
            }
            continue;
        }
        if !started {
            wire::validate(packet)?;
            out.configuration.push(Event {
                order: 0,
                time: Rational::new(
                    clock,
                    i64::from(ticks.ok_or_else(|| {
                        Diagnostic::new("E_TPQ", "configuration event precedes DCTPQ")
                    })?),
                ),
                words: packet.to_vec(),
            });
            continue;
        }
        let time = Rational::new(clock, i64::from(ticks.expect("start requires TPQ")));
        let kind = first >> 28;
        let status = (first >> 20) & 15;
        let route = Route {
            group: ((first >> 24) & 15) as u8,
            channel: ((first >> 16) & 15) as u8,
        };
        let key = ((first >> 8) & 127) as u8;
        if kind == 4 && status == 15 {
            wire::validate(packet)?;
            if active.contains_key(&(route, key)) {
                return Err(Diagnostic::new(
                    "E_NOTE_OVERLAP",
                    "detaching an active address needs an explicit note-instance mapping",
                ));
            }
            if first & 2 != 0 {
                pitch.remove(&(route, key));
                positions.remove(&(route, key));
                tokens.remove(&(route, key));
                dynamic_state.remove(&(route, key));
            }
            continue;
        }
        if kind == 4 && status == 0 && first.to_le_bytes()[0] == 3 {
            if active.contains_key(&(route, key)) {
                ordinal += 1;
                out.events.push(Event {
                    order: ordinal,
                    time,
                    words: packet.to_vec(),
                });
            }
            pitch.insert((route, key), Rational::new(i64::from(packet[1]), 1 << 25));
            continue;
        }
        if kind == 4 && status == 0 && first.to_le_bytes()[0] == 12 {
            if active.contains_key(&(route, key)) {
                ordinal += 1;
                out.events.push(Event {
                    order: ordinal,
                    time,
                    words: packet.to_vec(),
                });
            }
            positions.insert((route, key), packet[1]);
            continue;
        }
        if refinement
            && kind == 4
            && status == 1
            && Some(first.to_le_bytes()[0]) == dictionary.as_ref().map(|d| d.controller_index)
        {
            if active.contains_key(&(route, key)) {
                return Err(Diagnostic::new(
                    "E_EXTENSION_LIFETIME",
                    "descriptor tokens must be prepared before attack; use declared timed parameters for live refinements",
                ));
            }
            tokens.insert((route, key), packet[1]);
            continue;
        }
        if refinement
            && kind == 4
            && status == 1
            && first.to_le_bytes()[0] == 126
            && dictionary.as_ref().is_some_and(|d| {
                d.parameters
                    .get(&126)
                    .is_some_and(|id| id == crate::extension::DYNAMIC_LEVEL)
            })
        {
            if let Some(n) = active.get(&(route, key)) {
                let curve = dynamic_curves.entry((route, key)).or_insert_with(|| {
                    vec![(
                        Rational::from_integer(0),
                        *dynamic_state.get(&(route, key)).unwrap_or(&0),
                        0,
                    )]
                });
                ordinal += 1;
                curve.push((time - n.onset, packet[1], ordinal));
            }
            dynamic_state.insert((route, key), packet[1]);
            continue;
        }
        if kind == 4 && (status == 8 || status == 9) {
            wire::validate(packet)?;
            if status == 9 {
                let art = if matches!(first.to_le_bytes()[0], 0 | 3 | 4) {
                    Articulation::default()
                } else {
                    Articulation::from_data(
                        first.to_le_bytes()[0],
                        u16::from_le_bytes([
                            packet[1].to_le_bytes()[0],
                            packet[1].to_le_bytes()[1],
                        ]),
                    )?
                };
                let p = if first.to_le_bytes()[0] == 3 {
                    Rational::new(i64::from(packet[1] & 0xffff), 512)
                } else {
                    *pitch
                        .get(&(route, key))
                        .unwrap_or(&Rational::from_integer(i64::from(key)))
                };
                let mut descriptor = if refinement {
                    dictionary
                        .as_ref()
                        .expect("checked map")
                        .lookup(*tokens.get(&(route, key)).unwrap_or(&0))?
                } else {
                    None
                };
                if first.to_le_bytes()[0] == 4 {
                    descriptor
                        .get_or_insert_with(|| crate::model::Descriptor {
                            technique: "core.normal".into(),
                            ..Default::default()
                        })
                        .qualifiers
                        .insert(
                            "urn:midi2workbench:piano-key-velocity".into(),
                            serde_json::json!(packet[1] & 0xffff),
                        );
                }
                if let Some(value) = positions.get(&(route, key)).filter(|v| **v != 0x8000_0000) {
                    descriptor
                        .get_or_insert_with(|| crate::model::Descriptor {
                            technique: crate::rules::by_coordinate(art.at, art.subclass)
                                .map_or("core.normal", |r| r.id)
                                .into(),
                            ..Default::default()
                        })
                        .qualifiers
                        .insert(
                            "urn:midi2workbench:position".into(),
                            serde_json::json!(value),
                        );
                }
                if descriptor
                    .as_ref()
                    .is_some_and(|d| d.qualifiers.contains_key(crate::extension::DYNAMIC_CURVE))
                {
                    let value = *dynamic_state.get(&(route, key)).ok_or_else(|| {
                        Diagnostic::new("E_DYNAMIC_INITIAL", "missing dynamic initialization")
                    })?;
                    dynamic_curves
                        .insert((route, key), vec![(Rational::from_integer(0), value, 0)]);
                }
                ordinal += 1;
                let n = Note {
                    on_order: ordinal,
                    off_order: 0,
                    onset: time,
                    end: time,
                    pitch: p,
                    velocity: (packet[1] >> 16) as u16,
                    release_velocity: 0,
                    route,
                    key,
                    articulation: art,
                    descriptor,
                    source: String::new(),
                };
                if active.insert((route, key), n).is_some() {
                    return Err(Diagnostic::new(
                        "E_NOTE_OVERLAP",
                        "ambiguous same-key note lifecycle",
                    ));
                }
            } else {
                let mut n = active
                    .remove(&(route, key))
                    .ok_or_else(|| Diagnostic::new("E_NOTE_OFF", "unmatched release"))?;
                ordinal += 1;
                n.off_order = ordinal;
                n.end = time;
                n.release_velocity = (packet[1] >> 16) as u16;
                if first.to_le_bytes()[0] == 0x10 {
                    n.articulation.release = ((packet[1] >> 12) & 15) as u8;
                }
                if n.end <= n.onset {
                    return Err(Diagnostic::new("E_NOTE_TIME", "nonpositive duration"));
                }
                if let Some(curve) = dynamic_curves.remove(&(route, key)) {
                    let d = n
                        .descriptor
                        .get_or_insert_with(|| crate::model::Descriptor {
                            technique: crate::rules::by_coordinate(
                                n.articulation.at,
                                n.articulation.subclass,
                            )
                            .map_or("core.normal", |r| r.id)
                            .into(),
                            ..Default::default()
                        });
                    d.qualifiers.insert(
                        crate::extension::DYNAMIC_CURVE.into(),
                        serde_json::to_value(curve).expect("curve JSON"),
                    );
                }
                out.notes.push(n);
            }
            continue;
        }
        wire::validate(packet)?;
        ordinal += 1;
        out.events.push(Event {
            order: ordinal,
            time,
            words: packet.to_vec(),
        });
    }
    if !ended {
        return Err(Diagnostic::new("E_CLIP_END", "missing End of Clip"));
    }
    if !out
        .events
        .iter()
        .any(|e| e.time == Rational::from_integer(0) && e.words.first() == Some(&0xd010_0000))
    {
        for n in &mut out.notes {
            if n.onset == Rational::from_integer(0) {
                n.on_order += 1;
            }
        }
        for e in &mut out.events {
            if e.time == Rational::from_integer(0) {
                e.order += 1;
            }
        }
        out.events.insert(
            0,
            Event {
                order: 1,
                time: Rational::from_integer(0),
                words: wire::tempo(50_000_000),
            },
        );
        out.report
            .record("/", "implicit sequence tempo", "120 quarter BPM");
    }
    out.notes.sort_by_key(|n| (n.onset, n.route, n.key, n.end));
    Ok(out)
}
