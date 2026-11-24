//! Explicit receiver adaptation before MIDI encoding; native import does not own these choices.
use crate::{
    Diagnostic, Result,
    model::{Descriptor, Event, Note, Performance, Rational, Route},
    rendition::Rendition,
    rules::Articulation,
    wire,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod controls;
mod passages;
use controls::Controls;
pub use passages::Passage;

/// Internal conversion input, retained as evidence rather than a second delivery configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plan {
    pub rendition: Rendition,
    pub rules: Vec<Rule>,
    /// Preserve score preview mix and direction prose only in the archived source.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archive_score_preview: bool,
    /// Explicit beat-zero MIDI 2.0 CC values, independent of note dynamics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_controllers: Vec<InitialController>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub track_cc_curves: Vec<TrackCcCurve>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub note_edits: Vec<NoteEdit>,
    /// Reviewed musical expectations, independent of the adaptation rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passages: Vec<Passage>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackCcCurve {
    pub route: Route,
    pub controller: u8,
    pub points: Vec<CcPoint>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CcPoint {
    pub time: Rational,
    pub value: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteEdit {
    pub source: String,
    pub onset: Rational,
    pub key: u8,
    pub end: Rational,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialController {
    pub route: Route,
    pub controller: u8,
    /// Full unsigned 32-bit MIDI 2.0 value; no implicit 7-bit scaling.
    pub value: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    pub source_prefix: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_voice: Option<String>,
    pub articulation: Articulation,
    pub descriptor: Option<Descriptor>,
    pub route: Route,
    pub dynamics: Dynamics,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Dynamics {
    KeyVelocity,
    FixedAttack {
        key_velocity: u16,
    },
    Crossfade {
        controller: u8,
        key_velocity: u16,
        source_law: SourceLaw,
        expression: Option<ControllerValue>,
        /// Explicitly admit sampled dynamic curves as one shared channel axis.
        #[serde(default, skip_serializing_if = "CurvePolicy::is_reject")]
        curve_policy: CurvePolicy,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CurvePolicy {
    #[default]
    Reject,
    ChannelHold,
}
impl CurvePolicy {
    fn is_reject(&self) -> bool {
        *self == Self::Reject
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SourceLaw {
    RequireDynamic,
    DynamicOrVelocity16,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControllerValue {
    pub controller: u8,
    pub value: u32,
}
/// Values retained by source-note index before encoder canonical ordering.
#[derive(Clone, Debug)]
pub struct SourceValue {
    pub route: Route,
    pub velocity: u16,
}
fn fail(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_RECEIVER_ADAPTATION", message)
}

/// Adapt an interpreted performance with exact source/meaning predicates and encoded routes.
/// The caller retains the returned original values with encoded note provenance.
/// # Errors
/// Returns ambiguous/missing rules, conflicting channel dynamics, unsupported
/// required curves, or source messages without an explicit adaptation disposition.
pub fn render(source: &Performance, plan: &Plan) -> Result<(Performance, Vec<SourceValue>)> {
    render_with_voices(source, plan, &BTreeMap::new())
}

pub(crate) fn render_with_voices(
    source: &Performance,
    plan: &Plan,
    voices: &BTreeMap<String, String>,
) -> Result<(Performance, Vec<SourceValue>)> {
    plan.rendition.validate()?;
    validate_rules(plan)?;
    if !source.piano_routes.is_empty() {
        return Err(fail(
            "piano profile adaptation requires an explicit receiver policy",
        ));
    }
    let mut result = source.clone();
    // Initial program/name context is replaced by explicit assignments. Other
    // channel messages require their own source-to-destination association.
    retain_globals(&mut result, plan.archive_score_preview)?;
    let mut edits: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
    for edit in &plan.note_edits {
        if edits
            .entry(edit.source.as_str())
            .or_default()
            .insert((edit.onset, edit.key), edit.end)
            .is_some()
        {
            return Err(fail(format!("duplicate note edit: {}", edit.source)));
        }
    }
    let mut provenance = Vec::with_capacity(source.notes.len());
    let mut controls = Controls::default();
    for initial in &plan.initial_controllers {
        controls
            .initial
            .insert((initial.route, initial.controller), initial.value);
    }
    for note in &mut result.notes {
        if let Some(end) = edits
            .get_mut(note.source.as_str())
            .and_then(|occurrences| occurrences.remove(&(note.onset, note.key)))
        {
            note.end = end;
        }
        if note.onset < Rational::from_integer(0)
            || note.end <= note.onset
            || note.end > source.duration
        {
            return Err(fail(format!(
                "note outside performed lifetime: {}",
                note.source
            )));
        }
        let meaning = meaning(note)?;
        let source_path = note.source.split('#').next().unwrap_or(&note.source);
        let voice = voices.get(source_path);
        let mut matches = plan.rules.iter().filter(|rule| {
            note.source.starts_with(&rule.source_prefix)
                && rule.source_voice.as_ref().is_none_or(|v| voice == Some(v))
                && rule.articulation == note.articulation
                && rule.descriptor == meaning.descriptor
        });
        let rule = match (matches.next(), matches.count()) {
            (Some(rule), 0) => rule,
            (first, rest) => {
                let count = usize::from(first.is_some()) + rest;
                return Err(fail(format!(
                    "{count} matching complete rules for {}",
                    note.source
                )));
            }
        };
        provenance.push(SourceValue {
            route: note.route,
            velocity: note.velocity,
        });
        note.route = rule.route;
        controls.record(note, &rule.dynamics, &meaning)?;
        if !meaning.curve.is_empty() {
            if let Some(descriptor) = &mut note.descriptor {
                descriptor
                    .qualifiers
                    .remove(crate::extension::DYNAMIC_CURVE);
            }
            result.report.record(
                &note.source,
                "receiver dynamic curve",
                "samples rendered once as channel HOLD controls over [onset, end); note-off endpoints omitted; per-note curve consumed",
            );
        }
    }
    for (source, occurrences) in edits {
        if let Some(((onset, key), _)) = occurrences.first_key_value() {
            return Err(fail(format!(
                "unmatched note edit: {source} at {onset}, key {key}"
            )));
        }
    }
    passages::check(&mut result, plan)?;
    controls.finish(&mut result)?;
    write_track_cc_curves(&mut result, plan)?;
    plan.rendition.validate_performance(&result)?;
    Ok((result, provenance))
}

fn write_track_cc_curves(performance: &mut Performance, plan: &Plan) -> Result<()> {
    let mut points = 0usize;
    let mut occupied = BTreeSet::new();
    for event in performance.configuration.iter().chain(&performance.events) {
        if let Some(cc) = wire::channel_cc(&event.words) {
            occupied.insert(cc);
        }
    }
    for curve in &plan.track_cc_curves {
        points = points.saturating_add(curve.points.len());
        if points > 1_000_000
            || !plan
                .rendition
                .control_routes
                .iter()
                .any(|entry| entry.route == curve.route)
            || curve.controller > 119
            || matches!(curve.controller, 0 | 6 | 32 | 38 | 98..=101)
            || !occupied.insert((curve.route, curve.controller))
        {
            return Err(fail("invalid, unbound or duplicate track CC curve"));
        }
        let zero = Rational::from_integer(0);
        if curve.points.first().is_none_or(|p| p.time != zero)
            || curve
                .points
                .last()
                .is_some_and(|p| p.time > performance.duration)
            || curve.points.windows(2).any(|p| p[0].time >= p[1].time)
        {
            return Err(fail(
                "track CC points require beat zero and strictly increasing times within the rendition",
            ));
        }
        for point in &curve.points {
            let event = Event {
                time: point.time,
                order: 0,
                words: wire::cc(curve.route, curve.controller, point.value),
            };
            if point.time == zero {
                performance.configuration.push(event);
            } else {
                performance.events.push(event);
            }
        }
    }
    Ok(())
}

fn validate_rules(plan: &Plan) -> Result<()> {
    if plan.rules.is_empty() || plan.rules.len() > 4096 {
        return Err(fail("rule budget"));
    }
    let routes: BTreeSet<_> = plan
        .rendition
        .bindings
        .iter()
        .map(|binding| binding.route)
        .collect();
    let mut laws = BTreeMap::new();
    let mut initial = BTreeSet::new();
    if plan.initial_controllers.len() > 32_768 {
        return Err(fail("controller initialization budget"));
    }
    for value in &plan.initial_controllers {
        // M2-104-UM v1.1.2 §7.4.6 replaces these compound MIDI 1.0
        // selectors with unified program/registered/assignable messages.
        if value.controller > 127
            || matches!(value.controller, 0 | 6 | 32 | 38 | 98..=101)
            || !routes.contains(&value.route)
            || !initial.insert((value.route, value.controller))
        {
            return Err(fail(
                "invalid, unbound or duplicate controller initialization",
            ));
        }
    }
    for rule in &plan.rules {
        rule.articulation.validate()?;
        if rule.source_prefix.is_empty()
            || rule.source_prefix.len() > 4096
            || !routes.contains(&rule.route)
        {
            return Err(fail("missing source predicate or unbound rule route"));
        }
        if let Dynamics::Crossfade {
            controller,
            expression,
            ..
        } = &rule.dynamics
        {
            if *controller > 127
                || expression
                    .as_ref()
                    .is_some_and(|e| e.controller > 127 || e.controller == *controller)
            {
                return Err(fail(
                    "invalid or conflicting dynamics/expression controller",
                ));
            }
            if initial.contains(&(rule.route, *controller))
                || expression
                    .as_ref()
                    .is_some_and(|e| initial.contains(&(rule.route, e.controller)))
            {
                return Err(fail(
                    "explicit initialization conflicts with dynamics/expression ownership",
                ));
            }
        }
        if laws
            .insert(rule.route, &rule.dynamics)
            .is_some_and(|previous| previous != &rule.dynamics)
        {
            return Err(fail(format!(
                "conflicting receiver laws on {:?}",
                rule.route
            )));
        }
    }
    Ok(())
}
fn retain_globals(performance: &mut Performance, archive_score_preview: bool) -> Result<()> {
    let mut retained = Vec::new();
    for event in &performance.configuration {
        let head = *event
            .words
            .first()
            .ok_or_else(|| fail("empty configuration message"))?;
        if event.time == Rational::from_integer(0)
            && ((head >> 28 == 4 && (head >> 20) & 15 == 0xc)
                || (head >> 28 == 0xd && head & 0xff00 == 0x0100))
        {
            performance.report.record(
                "/",
                "receiver adaptation",
                "initial program/name context replaced by explicit rendition assignments",
            );
        } else if wire::route(&event.words).is_none() {
            retained.push(event.clone());
        } else if archive_score_preview
            && event.time == Rational::from_integer(0)
            && matches!(wire::channel_cc(&event.words), Some((_, 7 | 10)))
        {
            performance.report.record(
                "/",
                "receiver adaptation",
                "score preview volume/pan archived in the source",
            );
        } else {
            return Err(fail(format!(
                "unmapped required configuration event: {event:?}"
            )));
        }
    }
    if archive_score_preview {
        let before = performance.events.len();
        performance.events.retain(|event| {
            !matches!(
                crate::remarks::performance(&event.words),
                Some((Some(_), _))
            )
        });
        if performance.events.len() != before {
            performance.report.record(
                "/",
                "receiver adaptation",
                "channel-addressed score direction text archived in the source",
            );
        }
    }
    for event in &performance.events {
        if wire::route(&event.words).is_some() {
            return Err(fail(format!(
                "unmapped required timed channel event: {event:?}"
            )));
        }
    }
    performance.configuration = retained;
    Ok(())
}
type CurvePoint = (Rational, u32, u32);
struct Meaning {
    descriptor: Option<Descriptor>,
    level: Option<u32>,
    curve: Vec<CurvePoint>,
}
/// Return the exact source-note selectors used for receiver adaptation.
/// # Errors
/// Returns a diagnostic when an interpreted facet is invalid.
pub fn inspect_notes(notes: Vec<Note>) -> Result<Vec<Note>> {
    notes
        .into_iter()
        .map(|mut note| {
            note.descriptor = meaning(&note)?.descriptor;
            Ok(note)
        })
        .collect()
}
fn meaning(note: &crate::model::Note) -> Result<Meaning> {
    let mut descriptor = note.descriptor.clone();
    let mut level = None;
    let mut curve = Vec::new();
    if let Some(descriptor) = &mut descriptor {
        if descriptor
            .qualifiers
            .get(crate::extension::DYNAMIC_CURVE)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|points| points.len() > 65_536)
        {
            return Err(fail(format!("dynamic curve point budget: {}", note.source)));
        }
        curve = crate::extension::dynamic_curve(descriptor)?;
        descriptor
            .qualifiers
            .remove(crate::extension::DYNAMIC_CURVE);
        if let Some(value) = descriptor
            .qualifiers
            .remove(crate::extension::DYNAMIC_LEVEL)
        {
            level = Some(
                value
                    .as_u64()
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or_else(|| fail(format!("invalid dynamic level: {}", note.source)))?,
            );
        }
    }
    Ok(Meaning {
        descriptor,
        level,
        curve,
    })
}
