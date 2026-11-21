//! Read-only capture and offline musical assembly. Native records stay in controller.
use crate::{entity::Entity, files::Error, protocol::Snapshot};
use musicxml_midi2_convert::{
    self as convert,
    model::{Event, Note, Performance, Rational, Route},
    remarks, rules, wire,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capture {
    pub version: u8,
    pub entities: Entity,
    pub midi: Snapshot,
    pub globals: Vec<Context>,
    pub excluded: Vec<crate::project_address::Path>,
    /// Editorial transforms applied to a copy, never native mutation authority.
    #[serde(default)]
    pub decisions: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub beat: f64,
    pub seconds: f64,
    pub numerator: u8,
    pub denominator: u16,
}
impl Capture {
    /// # Errors
    /// Rejects incomplete canonical content and nonmonotonic timing observations.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 2 {
            return Err("Unsupported capture version".into());
        }
        self.entities.validate()?;
        self.midi.validate()?;
        for owner in &self.midi.owners {
            let mut prefix = owner.clone();
            prefix.elements.truncate(self.entities.path.elements.len());
            if !self.entities.path.matches(&prefix) {
                return Err("Content owner is outside the captured entity scope".into());
            }
        }
        if self.globals.is_empty()
            || self.globals[0].beat != 0.0
            || self.globals.iter().any(|c| {
                !c.beat.is_finite()
                    || !c.seconds.is_finite()
                    || c.beat < 0.0
                    || c.numerator == 0
                    || !c.denominator.is_power_of_two()
            })
            || self
                .globals
                .windows(2)
                .any(|p| p[1].beat <= p[0].beat || p[1].seconds <= p[0].seconds)
        {
            return Err("Incomplete or nonmonotonic global timing context".into());
        }
        Ok(())
    }
}
const TPQ: f64 = 61_440.0;
fn beat(value: f64) -> Result<Rational, Error> {
    if !value.is_finite() || value.abs() > 1_000_000.0 {
        return Err("Musical coordinate outside export bounds".into());
    }
    // Checked coordinate bound above makes the rounded i64 conversion exact.
    #[allow(clippy::cast_possible_truncation)]
    let ticks = (value * TPQ).round() as i64;
    Ok(Rational::new(ticks, 61_440))
}
fn level(value: f64, bits: u32) -> Result<u32, Error> {
    Ok(rules::level(
        convert::model::decimal(&(value * 127.0).to_string())?,
        bits,
    )?)
}
fn visit<'a>(entity: &'a Entity, output: &mut Vec<&'a Entity>) {
    output.push(entity);
    for child in &entity.children {
        visit(child, output);
    }
}
fn event(time: Rational, words: Vec<u32>) -> Event {
    Event {
        time,
        words,
        order: 0,
    }
}

/// Assemble from canonical records, without consulting live state or a companion.
/// # Errors
/// Returns malformed capture, route-capacity, numeric and semantic encoding errors.
pub fn assemble(capture: &Capture) -> Result<(Vec<u8>, serde_json::Value), Error> {
    capture.validate()?;
    let mut score = Performance::default();
    let mut entities = Vec::new();
    visit(&capture.entities, &mut entities);
    let mut tracks = BTreeMap::new();
    let mut names = Vec::new();
    let mut losses = capture.decisions.clone();
    let mut max_rounding = 0.0_f64;
    let mut selectors = 0;

    let mut clips: Vec<_> = capture.midi.clips.iter().collect();
    clips.sort_by_key(|clip| {
        entities
            .iter()
            .position(|e| e.metadata["id"].as_str() == Some(&clip.track))
            .unwrap_or(usize::MAX)
    });
    for clip in clips {
        score.duration = score.duration.max(beat(clip.start + clip.duration)?);
        if clip.notes.is_empty() {
            continue;
        }
        let index = tracks.len();
        let index = *tracks.entry(clip.track.clone()).or_insert(index);
        let route = Route {
            group: u8::try_from(index / 16)?,
            channel: u8::try_from(index % 16)?,
        };
        if route.group > 15 {
            return Err("More than 256 musical tracks".into());
        }
        if names.len() == index {
            let name = clip
                .path
                .elements
                .iter()
                .rev()
                .nth(1)
                .map_or("Part", crate::project_address::Element::name);
            names.push((route, name.to_owned()));
        }
        let slots = slot_channels(&entities, &clip.track)?;
        let (commands, rounding) = assemble_notes(
            clip,
            route,
            &slots,
            &names[index].1,
            &mut score,
            &mut losses,
        )?;
        selectors += commands;
        max_rounding = max_rounding.max(rounding);
    }
    score.duration = score.duration.max(beat(
        capture.globals.last().ok_or("Missing capture extent")?.beat,
    )?);
    assemble_automation(capture, &mut tracks, &mut names, &mut score, &mut losses)?;
    coalesce_remarks(&mut score);
    add_globals(capture, &mut score)?;
    if score.notes.len() + score.events.len() > 1_000_000 {
        return Err("Expanded performance exceeds one million events".into());
    }
    // Existing encoder owns same-key isolation. Its source rows reveal auxiliary
    // routes; describe these once in musical metadata, without native identities.
    let encoded = convert::clip::encode_with_sources(&score)?;
    // Bind expression after the canonical encoder has isolated overlapping keys.
    // The second pass then has explicit note-instance routes, not ambiguous broadcasts.
    score = convert::clip::decode(&encoded.bytes, None)?;
    encode_expression(&mut score, &mut losses)?;
    let parts = musical_parts(names, &encoded.note_sources);
    losses.sort();
    losses.dedup();
    let metadata = convert::arrangement::Arrangement {
        version: 1,
        parts,
        decisions: losses.clone(),
    };
    score
        .configuration
        .push(event(Rational::from_integer(0), metadata.words()?));
    let (bytes, _, _) = convert::clip::encode(&score)?;
    let analysis = convert::validation::inspect(&bytes);
    if !analysis.valid {
        return Err(format!(
            "Assembled MIDI2 failed validation: {:?}",
            analysis.diagnostics
        )
        .into());
    }
    let report = serde_json::json!({"notes":score.notes.len(),"selectorCommands":selectors,"maxRoundingBeats":max_rounding,"excluded":capture.excluded,"decisions":losses,"validation":analysis});
    Ok((bytes, report))
}
fn musical_parts(
    names: Vec<(Route, String)>,
    sources: &[convert::clip::NoteSource],
) -> Vec<convert::arrangement::Part> {
    let mut parts = Vec::new();
    for (base, name) in names {
        let mut routes = BTreeSet::from([base]);
        for row in sources {
            if row.source_route == base {
                routes.insert(row.encoded_route);
            }
        }
        parts.push(convert::arrangement::Part {
            name,
            routes: routes.into_iter().collect(),
        });
    }
    parts
}

fn onset_q_to_f64(value: Rational) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let value = *value.numer() as f64 / *value.denom() as f64;
    value
}

fn slot_channels(entities: &[&Entity], track: &str) -> Result<BTreeMap<u8, Vec<String>>, Error> {
    let owner = entities
        .iter()
        .find(|e| e.metadata["id"].as_str() == Some(track))
        .ok_or("Captured track absent from entities")?;
    let mut children = Vec::new();
    visit(owner, &mut children);
    let mut channels: BTreeMap<u8, Vec<String>> = BTreeMap::new();
    for slot in children
        .iter()
        .filter(|e| matches!(e.kind.as_str(), "layerChain" | "choice"))
    {
        let label = slot.metadata["name"]
            .as_str()
            .ok_or("Unnamed receiver slot")?;
        let filter = slot
            .children
            .iter()
            .find_map(|d| d.metadata["channelFilter"].as_array().map(|v| (d, v)));
        let map = slot
            .children
            .iter()
            .find(|d| d.metadata["channelMap"].is_array());
        if let Some((device, filter)) = filter {
            if filter.len() != 16
                || device.metadata["routingEnabled"] != true
                || device.metadata["routingActive"] != true
                || map.is_none_or(|d| {
                    d.metadata["routingEnabled"] != true
                        || d.metadata["routingActive"] != true
                        || d.metadata["channelMap"].as_array().is_none_or(|m| {
                            m.len() != 16 || m.iter().any(|v| v.as_u64() != Some(1))
                        })
                })
            {
                return Err(format!("Unresolved Filter/Map routing for {label}").into());
            }
            for (i, enabled) in filter.iter().enumerate() {
                // Shared channel 1 carries controls, unless it is the slot's sole admitted channel.
                if enabled == true && (i != 0 || filter.iter().skip(1).all(|v| v == false)) {
                    channels
                        .entry(u8::try_from(i)?)
                        .or_default()
                        .push(label.to_owned());
                }
            }
        } else if let Some(channel) = slot.metadata["channel"].as_i64().filter(|c| *c >= 0) {
            channels
                .entry(u8::try_from(channel)?)
                .or_default()
                .push(label.to_owned());
        }
    }
    for labels in channels.values_mut() {
        labels.sort();
        labels.dedup();
    }
    Ok(channels)
}

/// Each segment maps a source interval to one arranger occurrence.
fn segments(
    start: f64,
    duration: f64,
    source_start: f64,
    source_end: f64,
    loop_start: f64,
    loop_length: f64,
    looped: bool,
) -> Result<Vec<(f64, f64, f64)>, Error> {
    let mut output = Vec::new();
    let mut elapsed = 0.0;
    let mut source = source_start;
    while elapsed < duration {
        if output.len() >= 100_000 {
            return Err("Loop expansion exceeds 100000 segments".into());
        }
        let boundary = if looped {
            loop_start + loop_length
        } else {
            source_end
        };
        let length = (boundary - source).min(duration - elapsed);
        if length <= 0.0 || !length.is_finite() {
            return Err("Unsupported clip source/loop mapping".into());
        }
        output.push((start + elapsed, source, source + length));
        elapsed += length;
        if !looped {
            break;
        }
        source = loop_start;
    }
    Ok(output)
}
fn occurrences(
    clip: &crate::protocol::Clip,
    start: f64,
    duration: f64,
) -> Result<Vec<(f64, f64)>, Error> {
    Ok(segments(
        clip.start,
        clip.duration,
        clip.source_start,
        clip.source_end,
        clip.loop_start,
        clip.loop_length,
        clip.loop_enabled,
    )?
    .into_iter()
    .filter_map(|(position, from, to)| {
        if start >= from && start < to {
            Some((
                position + start - from,
                position + (start + duration).min(to) - from,
            ))
        } else {
            None
        }
    })
    .collect())
}
fn assemble_automation(
    capture: &Capture,
    tracks: &mut BTreeMap<String, usize>,
    names: &mut Vec<(Route, String)>,
    score: &mut Performance,
    losses: &mut Vec<String>,
) -> Result<(), Error> {
    for lane in &capture.midi.automation {
        let index = tracks.len();
        let index = *tracks.entry(lane.track.clone()).or_insert(index);
        let route = Route {
            group: u8::try_from(index / 16)?,
            channel: u8::try_from(index % 16)?,
        };
        if route.group > 15 {
            return Err("More than 256 musical tracks".into());
        }
        if names.len() == index {
            names.push((
                route,
                lane.owner
                    .elements
                    .last()
                    .map_or("Part", crate::project_address::Element::name)
                    .to_owned(),
            ));
        }
        assemble_lane(lane, route, score, losses)?;
    }
    Ok(())
}
fn assemble_lane(
    lane: &crate::automation::Curve,
    route: Route,
    score: &mut Performance,
    losses: &mut Vec<String>,
) -> Result<(), Error> {
    let cc = lane.target["controller"]
        .as_u64()
        .filter(|_| lane.target["kind"] == "midiCc");
    if lane.target["kind"] == "midiCc" && cc.is_none() {
        return Err("MIDI CC lane requires a controller number".into());
    }
    if cc.is_none() {
        if !lane.points.is_empty() {
            let name = lane.target["name"].as_str().unwrap_or("Unmapped parameter");
            losses.push(format!(
                "{name}: no established MIDI equivalent; retained as a musical parameter curve"
            ));
            let mut points = Vec::new();
            for (position, from, to) in lane_segments(lane, score.duration)? {
                for p in &lane.points {
                    if p.time >= from && p.time <= to {
                        points.push(serde_json::json!([
                            position + p.time - from,
                            p.value,
                            p.interpolation,
                            p.curvature,
                            p.spread
                        ]));
                    }
                }
            }
            score.configuration.push(event(Rational::from_integer(0),remarks::text(Some(route),&format!("urn:midi2workbench:parameter:v1:{}",serde_json::json!({"name":name,"unit":lane.target["unit"],"points":points,"timeUnit":"quarter-note beats"})),true)));
        }
        return Ok(());
    }
    let channel = lane.target["channel"]
        .as_u64()
        .filter(|c| *c < 16)
        .ok_or("MIDI CC requires a channel in 0..15")?;
    if channel != 0 && !lane.points.is_empty() {
        return Err("Receiver-specific CC must be interpreted before collapsing articulation channels; shared controls use channel 1".into());
    }
    let cc = u8::try_from(cc.ok_or("Missing CC")?)?;
    if cc > 119 {
        return Err("Channel-mode messages cannot be exported as continuous CC lanes".into());
    }
    if lane
        .points
        .iter()
        .any(|p| p.spread != 0.0 || p.curvature != 0.0)
    {
        losses.push(format!(
            "CC{cc}: curvature/spread approximated by straight segments"
        ));
    }
    let mut samples = Vec::new();
    for (i, p) in lane.points.iter().enumerate() {
        if !(0.0..=1.0).contains(&p.value) {
            return Err(
                format!("CC{cc} values must be normalized to 0..1 in interpreted capture").into(),
            );
        }
        if samples.len() > crate::automation::MAX_POINTS {
            return Err("Sampled control exceeds 100000 points".into());
        }
        samples.push((p.time, p.value));
        if let Some(next) = lane.points.get(i + 1) {
            if p.interpolation == crate::editor::Interpolation::Linear && next.time > p.time {
                let count = ((next.value - p.value).abs() * 1024.0).ceil().max(1.0);
                if count > 100_000.0 {
                    return Err("Control sampling budget exceeded".into());
                }
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let count = count as u32;
                for step in 1..count {
                    let fraction = f64::from(step) / f64::from(count);
                    samples.push((
                        p.time + (next.time - p.time) * fraction,
                        p.value + (next.value - p.value) * fraction,
                    ));
                }
            }
        }
    }
    let mapped = lane_segments(lane, score.duration)?;
    for (position, from, to) in mapped {
        if let Some((_, v)) = samples.iter().rev().find(|(t, _)| *t < from) {
            score
                .events
                .push(event(beat(position)?, wire::cc(route, cc, level(*v, 32)?)));
        }
        for &(t, v) in &samples {
            if t >= from && t <= to {
                score.events.push(event(
                    beat(position + t - from)?,
                    wire::cc(route, cc, level(v, 32)?),
                ));
            }
        }
    }
    Ok(())
}
fn add_globals(capture: &Capture, score: &mut Performance) -> Result<(), Error> {
    let mut last_tempo = None;
    let mut last_meter = None;
    for (i, c) in capture.globals.iter().enumerate() {
        let time = beat(c.beat)?;
        let meter = (c.numerator, c.denominator);
        if last_meter != Some(meter) {
            let words = wire::meter(c.numerator, u8::try_from(c.denominator.ilog2())?);
            score.events.push(event(time, words.clone()));
            if i == 0 {
                score.configuration.push(event(time, words));
            }
            last_meter = Some(meter);
        }
        if let Some(next) = capture.globals.get(i + 1) {
            let bpm = 60.0 * (next.beat - c.beat) / (next.seconds - c.seconds);
            let words = wire::tempo_from_bpm(convert::model::decimal(&format!("{bpm:.8}"))?)?;
            if last_tempo.as_ref() != Some(&words) {
                score.events.push(event(time, words.clone()));
                if i == 0 {
                    score.configuration.push(event(time, words.clone()));
                }
                last_tempo = Some(words);
            }
        }
    }
    // Tempo/meter precede descriptive metadata in the configuration header.
    score.configuration.sort_by_key(|e| {
        if e.words[0].trailing_zeros() >= 16 {
            0
        } else if e.words[0] & 0xffff == 1 {
            1
        } else {
            2
        }
    });
    Ok(())
}

fn encode_expression(score: &mut Performance, losses: &mut Vec<String>) -> Result<(), Error> {
    let note_count = score.notes.len();
    for n in &mut score.notes {
        let Some(d) = n.descriptor.as_mut() else {
            continue;
        };
        let Some(curves) = d
            .qualifiers
            .get_mut("urn:midi2workbench:expression")
            .and_then(serde_json::Value::as_object_mut)
        else {
            continue;
        };
        for kind in ["pressure", "pitch", "pan", "gain"] {
            let Some(points) = curves.remove(kind) else {
                continue;
            };
            let points = points.as_array().ok_or("Malformed expression points")?;
            if kind == "gain"
                && points.iter().any(|p| {
                    p["value"]
                        .as_f64()
                        .is_none_or(|v| !(0.0..=1.0).contains(&v))
                })
            {
                curves.insert(kind.into(), serde_json::Value::Array(points.clone()));
                losses.push("Per-note gain above unity retained as a musical residual".into());
                continue;
            }
            for (i, p) in points.iter().enumerate() {
                let offset = p["time"].as_f64().ok_or("Missing expression time")?;
                let value = p["value"].as_f64().ok_or("Missing expression value")?;
                let mut samples = vec![(offset, value)];
                if let Some(next) = points.get(i + 1).filter(|_| p["interpolation"] == "LINEAR") {
                    let to = next["time"].as_f64().ok_or("Missing expression end")?;
                    let v = next["value"]
                        .as_f64()
                        .ok_or("Missing expression end value")?;
                    let scale = if kind == "pitch" {
                        128.0
                    } else if kind == "pan" {
                        2.0
                    } else {
                        1.0
                    };
                    let count = ((v - value).abs() / scale * 1024.0)
                        .ceil()
                        .clamp(1.0, 100_000.0);
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let count = count as u32;
                    for step in 1..count {
                        let f = f64::from(step) / f64::from(count);
                        samples.push((offset + (to - offset) * f, value + (v - value) * f));
                    }
                }
                for (offset, value) in samples {
                    let time = (n.onset + beat(offset)?).max(n.onset);
                    if time >= n.end {
                        continue;
                    }
                    if p["curvature"] != 0.0 || p["spread"] != 0.0 {
                        losses.push(format!(
                            "{kind}: expression curvature/spread approximated by straight segments"
                        ));
                    }
                    let words = expression_message(kind, n.route, n.key, value)?;
                    if note_count + score.events.len() > 1_000_000 {
                        return Err("Expanded expression exceeds one million events".into());
                    }
                    score.events.push(event(time, words));
                }
            }
        }
        if curves.is_empty() {
            d.qualifiers.remove("urn:midi2workbench:expression");
        } else {
            losses.push("Expression without a documented standard equivalent retained as a musical residual".into());
        }
    }
    Ok(())
}

fn coalesce_remarks(score: &mut Performance) {
    score.events.sort_by_key(|e| e.time);
    let mut last = BTreeMap::new();
    score.events.retain(|e| {
        let Some((Some(route), text)) = remarks::performance(&e.words) else {
            return true;
        };
        if text.starts_with(convert::arrangement::COMMAND) {
            last.remove(&route);
            return true;
        }
        if last.get(&route) == Some(&text) {
            return false;
        }
        last.insert(route, text);
        true
    });
}

fn assemble_notes(
    clip: &crate::protocol::Clip,
    route: Route,
    slots: &BTreeMap<u8, Vec<String>>,
    name: &str,
    score: &mut Performance,
    losses: &mut Vec<String>,
) -> Result<(usize, f64), Error> {
    let mut selectors = 0;
    let mut max_rounding = 0.0_f64;
    for selected in &clip.notes {
        let n = &selected.note;
        if n.muted {
            losses.push(format!("{name}: muted note omitted"));
            continue;
        }
        let labels = slots.get(&n.channel).cloned().unwrap_or_default();
        let mut articulation = rules::Articulation::default();
        let mut descriptor = None;
        let mut lyric = None;
        for label in &labels {
            if let Some(phrase) = label.strip_prefix("[lyrics") {
                let (options, text) = phrase.split_once(']').ok_or("Unclosed lyric slot prefix")?;
                let text = options
                    .split_whitespace()
                    .find_map(|v| v.strip_prefix("lang="))
                    .map_or_else(
                        || text.trim().to_owned(),
                        |lang| format!("[{lang}] {}", text.trim()),
                    );
                let reset = options
                    .split_whitespace()
                    .find_map(|v| v.strip_prefix("reset="));
                if let Some(key) = reset {
                    if key.parse::<u8>()? == n.key {
                        selectors += 1;
                        lyric = Some((true, text.trim().to_owned()));
                        break;
                    }
                }
                lyric = Some((false, text.trim().to_owned()));
            } else if let Some((art, desc)) = rules::term(label) {
                (articulation, descriptor) = rules::combine(articulation, descriptor, art, desc)?;
            } else {
                losses.push(format!("{name}: slot '{label}' retained as a remark"));
            }
        }
        for (onset, end) in occurrences(clip, n.start, n.duration)? {
            if end + 1.0 / TPQ < onset + n.duration {
                losses.push(format!(
                    "{name}: note tails clipped at arranger/loop boundary"
                ));
            }
            if score.notes.len() + score.events.len() > 1_000_000 {
                return Err("Expanded performance exceeds one million events".into());
            }
            let onset_q = beat(onset)?;
            let mut end_q = beat(end)?;
            max_rounding = max_rounding
                .max((onset_q_to_f64(onset_q) - onset).abs())
                .max((onset_q_to_f64(end_q) - end).abs());
            if end_q <= onset_q {
                end_q = onset_q + Rational::new(1, 61_440);
                losses.push("Positive note extended to one file tick".into());
            }
            if let Some((reset, text)) = &lyric {
                if *reset {
                    score.events.push(event(onset_q, remarks::text(Some(route), &format!("{}{}", convert::arrangement::COMMAND, serde_json::json!({"key":n.key,"duration":end_q-onset_q,"velocity":level(n.velocity,16)?})), false)));
                    continue;
                }
                score.events.push(event(
                    onset_q,
                    remarks::text(Some(route), &format!("Lyrics phrase: {text}"), false),
                ));
            } else {
                for label in &labels {
                    if rules::term(label).is_none() {
                        score
                            .events
                            .push(event(onset_q, remarks::text(Some(route), label, false)));
                    }
                }
            }
            let descriptor = note_descriptor(n, descriptor.clone(), losses)?;
            score.notes.push(Note {
                onset: onset_q,
                end: end_q,
                pitch: Rational::from_integer(i64::from(n.key)),
                velocity: u16::try_from(level(n.velocity, 16)?)?,
                release_velocity: u16::try_from(level(n.release_velocity, 16)?)?,
                route,
                key: n.key,
                articulation,
                descriptor,
                source: name.into(),
                on_order: 0,
                off_order: 0,
            });
        }
    }
    Ok((selectors, max_rounding))
}
fn note_descriptor(
    n: &crate::editor::NoteEvent,
    mut descriptor: Option<convert::model::Descriptor>,
    losses: &mut Vec<String>,
) -> Result<Option<convert::model::Descriptor>, Error> {
    let o = &n.operators;
    let mut behavior = serde_json::Map::new();
    if o.chance_enabled && o.chance < 1.0 {
        behavior.insert("chance".into(), serde_json::json!(o.chance));
    }
    if o.occurrence_enabled && o.occurrence != crate::editor::Occurrence::Always {
        behavior.insert("occurrence".into(), serde_json::json!(o.occurrence));
    }
    if o.recurrence_enabled && u16::from(o.recurrence_mask) != (1_u16 << o.recurrence_length) - 1 {
        behavior.insert(
            "recurrence".into(),
            serde_json::json!([o.recurrence_length, o.recurrence_mask]),
        );
    }
    if o.repeat_enabled {
        behavior.insert("repeat".into(),serde_json::json!({"count":o.repeat_count,"curve":o.repeat_curve,"velocityEnd":o.repeat_velocity_end,"velocityCurve":o.repeat_velocity_curve}));
    }
    if n.velocity_spread != 0.0 {
        behavior.insert(
            "attackVelocitySpread".into(),
            serde_json::json!(n.velocity_spread),
        );
    }
    if !behavior.is_empty() {
        let d = descriptor.get_or_insert_with(|| convert::model::Descriptor {
            technique: "core.normal".into(),
            ..Default::default()
        });
        d.qualifiers
            .insert("urn:midi2workbench:note-behavior".into(), behavior.into());
        losses.push(
            "Note conditions or attack variation retained; file contains nominal attacks".into(),
        );
    }
    if n.expressions.values().any(|v| !v.is_empty()) {
        let d = descriptor.get_or_insert_with(|| convert::model::Descriptor {
            technique: "core.normal".into(),
            ..Default::default()
        });
        let curves: BTreeMap<_,_>=n.expressions.iter().filter(|(_,p)| !p.is_empty()).map(|(kind,points)|(kind,points.iter().map(|p|serde_json::json!({"time":p.time-n.content_start,"value":p.value,"interpolation":p.interpolation,"curvature":p.curvature,"spread":p.spread})).collect::<Vec<_>>())).collect();
        d.qualifiers.insert(
            "urn:midi2workbench:expression".into(),
            serde_json::to_value(curves)?,
        );
    }
    Ok(descriptor)
}

fn expression_message(kind: &str, route: Route, key: u8, value: f64) -> Result<Vec<u32>, Error> {
    Ok(match kind {
        "pressure" => wire::pressure(route, key, level(value, 32)?),
        "pitch" => {
            let pitch = f64::from(key) + value;
            if !(0.0..128.0).contains(&pitch) {
                return Err("Absolute per-note pitch outside Pitch 7.25".into());
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let value = (pitch * 33_554_432.0).round().min(f64::from(u32::MAX)) as u32;
            wire::controller(route, key, 3, value, true)
        }
        "pan" => {
            if !(-1.0..=1.0).contains(&value) {
                return Err("Per-note pan outside -1..1".into());
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let value = ((value + 1.0) * 2_147_483_648.0)
                .round()
                .min(f64::from(u32::MAX)) as u32;
            wire::controller(route, key, 10, value, true)
        }
        "gain" => {
            // Native cube-root amplitude versus M2-113's squared volume response.
            let control = value.powf(1.5);
            wire::controller(route, key, 7, level(control, 32)?, true)
        }
        _ => return Err("Unknown expression".into()),
    })
}

fn lane_segments(
    lane: &crate::automation::Curve,
    duration: Rational,
) -> Result<Vec<(f64, f64, f64)>, Error> {
    Ok(if lane.placement == "arranger" {
        vec![(0.0, 0.0, onset_q_to_f64(duration))]
    } else {
        let m = &lane.mapping;
        segments(
            m["startBeats"]
                .as_f64()
                .ok_or("Missing automation placement")?,
            m["durationBeats"]
                .as_f64()
                .ok_or("Missing automation duration")?,
            m["sourceStartBeats"]
                .as_f64()
                .ok_or("Missing automation source start")?,
            m["sourceEndBeats"]
                .as_f64()
                .ok_or("Missing automation source end")?,
            m["loopStart"].as_f64().unwrap_or(0.0),
            m["loopLength"].as_f64().unwrap_or(0.0),
            m["loopEnabled"].as_bool().unwrap_or(false),
        )?
    })
}
