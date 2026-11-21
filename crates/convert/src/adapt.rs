//! Explicit receiver adaptation before MIDI encoding; native import does not own these choices.
use crate::{
    Diagnostic, Result,
    model::{Descriptor, Performance, Rational, Route},
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
    /// Explicit beat-zero MIDI 2.0 CC values, independent of note dynamics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_controllers: Vec<InitialController>,
    /// Reviewed musical expectations, independent of the adaptation rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passages: Vec<Passage>,
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
    pub articulation: Articulation,
    pub descriptor: Option<Descriptor>,
    pub route: Route,
    pub dynamics: Dynamics,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Dynamics {
    KeyVelocity,
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceLaw {
    RequireDynamic,
    DynamicOrVelocity16,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    retain_globals(&mut result)?;
    let mut provenance = Vec::with_capacity(source.notes.len());
    let mut controls = Controls::default();
    for initial in &plan.initial_controllers {
        controls
            .initial
            .insert((initial.route, initial.controller), initial.value);
    }
    for note in &mut result.notes {
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
        let matches: Vec<_> = plan
            .rules
            .iter()
            .filter(|rule| {
                note.source.starts_with(&rule.source_prefix)
                    && rule.articulation == note.articulation
                    && rule.descriptor == meaning.descriptor
            })
            .collect();
        if matches.len() != 1 {
            return Err(fail(format!(
                "{} matching complete rules for {}",
                matches.len(),
                note.source
            )));
        }
        let rule = matches[0];
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
    passages::check(&mut result, plan)?;
    controls.finish(&mut result)?;
    plan.rendition.validate_performance(&result)?;
    Ok((result, provenance))
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
        let law = serde_json::to_vec(&rule.dynamics).map_err(|e| fail(e.to_string()))?;
        if laws
            .insert(rule.route, law.clone())
            .is_some_and(|previous| previous != law)
        {
            return Err(fail(format!(
                "conflicting receiver laws on {:?}",
                rule.route
            )));
        }
    }
    Ok(())
}
fn retain_globals(performance: &mut Performance) -> Result<()> {
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
        } else {
            return Err(fail(format!(
                "unmapped required configuration event: {event:?}"
            )));
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
