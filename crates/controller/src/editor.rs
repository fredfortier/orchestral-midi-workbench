//! Native note values, validation and explicit field changes.
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoteEvent {
    pub id: String,
    pub channel: u8,
    pub key: u8,
    pub start: f64,
    pub duration: f64,
    pub muted: bool,
    pub velocity: f64,
    pub release_velocity: f64,
    pub velocity_spread: f64,
    pub content_start: f64,
    pub operators: Operators,
    pub expressions: BTreeMap<Expression, Vec<Point>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
// These four independent native enable flags must remain separate on the wire.
#[allow(clippy::struct_excessive_bools)]
pub struct Operators {
    pub chance_enabled: bool,
    pub chance: f64,
    pub occurrence_enabled: bool,
    pub occurrence: Occurrence,
    pub recurrence_enabled: bool,
    pub recurrence_length: u8,
    pub recurrence_mask: u8,
    pub repeat_enabled: bool,
    pub repeat_count: i8,
    pub repeat_curve: f64,
    pub repeat_velocity_end: f64,
    pub repeat_velocity_curve: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Occurrence {
    Always,
    First,
    NotFirst,
    Prev,
    NotPrev,
    PrevChannel,
    NotPrevChannel,
    PrevKey,
    NotPrevKey,
    Fill,
    NotFill,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum Expression {
    Pressure,
    Pitch,
    Gain,
    Pan,
    Timbre,
    Formant,
}

impl Expression {
    const FAMILIES: [Self; 6] = [
        Self::Pressure,
        Self::Pitch,
        Self::Gain,
        Self::Pan,
        Self::Timbre,
        Self::Formant,
    ];
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Point {
    pub id: String,
    /// Beats in the note's expression content, with `contentStart` kept explicit.
    pub time: f64,
    /// Native values: pitch in semitones, pressure in `0..=1`, pan/timbre in `-1..=1`.
    /// Gain is cube-root amplitude: positive values display as `60 * log10(value)` dB;
    /// zero represents silence. Keep stored values independent of UI display bounds.
    pub value: f64,
    pub interpolation: Interpolation,
    pub curvature: f64,
    pub spread: f64,
}

/// Stable native ties include numerically equal signed zeros; IDs never order ties.
#[allow(clippy::float_cmp)]
pub(crate) fn compare_point_times(a: &Point, b: &Point) -> Ordering {
    if a.time == b.time {
        Ordering::Equal
    } else {
        a.time.total_cmp(&b.time)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Interpolation {
    Hold,
    Linear,
}

/// Typed sparse edits: omission preserves, an empty expression vector clears.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "field",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Change {
    Channel(u8),
    Key(u8),
    Start(f64),
    Duration(f64),
    Muted(bool),
    Velocity(f64),
    ReleaseVelocity(f64),
    VelocitySpread(f64),
    Operators(Operators),
    Expressions(BTreeMap<Expression, Vec<Point>>),
}

/// A validated native note. No intermediate MIDI encoding or quantization.
pub struct MidiNote {
    detail: NoteEvent,
}

fn bounded(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

impl TryFrom<NoteEvent> for MidiNote {
    type Error = String;
    fn try_from(detail: NoteEvent) -> Result<Self, String> {
        let o = &detail.operators;
        if detail.id.is_empty()
            || detail.channel > 15
            || detail.key > 127
            || !detail.start.is_finite()
            || !detail.duration.is_finite()
            || detail.duration <= 0.0
            || !(detail.start + detail.duration).is_finite()
            || !detail.content_start.is_finite()
            || !bounded(detail.velocity, 0.0, 1.0)
            || !bounded(detail.release_velocity, 0.0, 1.0)
            || !bounded(detail.velocity_spread, 0.0, 1.0)
            || !bounded(o.chance, 0.0, 1.0)
            || !(1..=8).contains(&o.recurrence_length)
            || !bounded(o.repeat_curve, -1.0, 1.0)
            || !bounded(o.repeat_velocity_end, -1.0, 1.0)
            || !bounded(o.repeat_velocity_curve, -1.0, 1.0)
        {
            return Err(format!("Invalid note event {}", detail.id));
        }
        for kind in Expression::FAMILIES {
            if !detail.expressions.contains_key(&kind) {
                return Err(format!(
                    "Complete note event {} is missing {kind:?} expression; use an empty array when it has no points",
                    detail.id
                ));
            }
        }
        for (&kind, points) in &detail.expressions {
            let mut ids = std::collections::HashSet::new();
            let mut previous = f64::NEG_INFINITY;
            for p in points {
                let valid_value = match kind {
                    Expression::Pressure => bounded(p.value, 0.0, 1.0),
                    Expression::Pan | Expression::Timbre => bounded(p.value, -1.0, 1.0),
                    Expression::Pitch | Expression::Gain | Expression::Formant => {
                        p.value.is_finite()
                    }
                };
                if p.id.is_empty()
                    || !ids.insert(&p.id)
                    || !p.time.is_finite()
                    || p.time < previous
                    || !valid_value
                    || !bounded(p.curvature, -1.0, 1.0)
                    || !p.spread.is_finite()
                    || p.spread < 0.0
                {
                    return Err(format!("Invalid {kind:?} point on {}", detail.id));
                }
                previous = p.time;
            }
        }
        Ok(Self { detail })
    }
}

impl MidiNote {
    #[must_use]
    pub fn detail(&self) -> &NoteEvent {
        &self.detail
    }

    /// Change explicit native values, then validate the complete resulting note.
    /// Full validation happens before replacing this note.
    ///
    /// # Errors
    /// Refuses invalid field values or an invalid resulting note without replacing it.
    pub fn change(&mut self, change: &Change) -> Result<(), String> {
        let mut next = self.detail.clone();
        match change {
            Change::Channel(v) if *v <= 15 => next.channel = *v,
            Change::Key(v) if *v <= 127 => next.key = *v,
            Change::Channel(_) | Change::Key(_) => return Err("Invalid MIDI address".into()),
            Change::Start(v) => next.start = *v,
            Change::Duration(v) => next.duration = *v,
            Change::Muted(v) => next.muted = *v,
            Change::Velocity(v) => next.velocity = *v,
            Change::ReleaseVelocity(v) => next.release_velocity = *v,
            Change::VelocitySpread(v) => next.velocity_spread = *v,
            Change::Operators(v) => next.operators = v.clone(),
            Change::Expressions(v) => next.expressions.extend(v.clone()),
        }
        *self = next.try_into()?;
        Ok(())
    }
}
