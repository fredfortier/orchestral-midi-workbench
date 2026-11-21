//! Seeded musical variation; native execution uses the ordinary absolute-plan path.
use crate::{
    editor::{Change, Expression, Interpolation, MidiNote},
    protocol::{Document, Edit},
};
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Randomize {
    pub seed: u64,
    pub count: usize,
    pub changes_per_note: usize,
    /// Eligible fields with inclusive native-value bounds. Empty uses scalar defaults.
    pub ranges: BTreeMap<String, [f64; 2]>,
}

fn bounds(field: &str) -> Option<(f64, f64, bool)> {
    Some(match field {
        "channel" => (0.0, 15.0, true),
        "key" => (0.0, 127.0, true),
        "muted" | "chanceEnabled" | "occurrenceEnabled" | "recurrenceEnabled" | "repeatEnabled" => {
            (0.0, 1.0, true)
        }
        "velocity" | "releaseVelocity" | "velocitySpread" | "chance" => (0.0, 1.0, false),
        "start" => (-8_796_093_022_208.0, 8_796_093_022_208.0, false),
        "duration" => (1.0 / 1_048_576.0, 8_796_093_022_208.0, false),
        "occurrence" => (0.0, 10.0, true),
        "recurrenceLength" => (1.0, 8.0, true),
        "recurrenceMask" => (0.0, 255.0, true),
        "repeatCount" => (-127.0, 127.0, true),
        "repeatCurve" | "repeatVelocityEnd" | "repeatVelocityCurve" => (-1.0, 1.0, false),
        _ => {
            let (kind, property) = field.split_once('.')?;
            expression(kind)?;
            match property {
                "value" => match kind {
                    "pressure" => (0.0, 1.0, false),
                    "pan" | "timbre" => (-1.0, 1.0, false),
                    "pitch" => (-96.0, 96.0, false),
                    // Native cube-root amplitude, not UI dB.
                    "gain" => (0.0, 2.0, false),
                    _ => return None,
                },
                "time" => (-8_796_093_022_208.0, 8_796_093_022_208.0, false),
                "curvature" => (-1.0, 1.0, false),
                "spread" => (0.0, f64::MAX, false),
                "interpolation" => (0.0, 1.0, true),
                _ => return None,
            }
        }
    })
}
fn expression(name: &str) -> Option<Expression> {
    match name {
        "pressure" => Some(Expression::Pressure),
        "pan" => Some(Expression::Pan),
        "timbre" => Some(Expression::Timbre),
        "pitch" => Some(Expression::Pitch),
        "gain" => Some(Expression::Gain),
        _ => None,
    }
}
impl Randomize {
    /// Compile an exact random subset without replacement. Invalid/colliding batches
    /// are refused whole by the caller; no notes or requested fields are skipped.
    /// # Errors
    /// Refuses invalid ranges, unavailable curves and impossible subset sizes.
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::float_cmp
    )]
    pub fn edits(&self, document: &Document) -> Result<Vec<Edit>, String> {
        let ranges = if self.ranges.is_empty() {
            BTreeMap::from([
                ("channel".into(), [0.0, 15.0]),
                ("velocity".into(), [0.0, 1.0]),
                ("releaseVelocity".into(), [0.0, 1.0]),
                ("velocitySpread".into(), [0.0, 1.0]),
                ("muted".into(), [0.0, 1.0]),
            ])
        } else {
            self.ranges.clone()
        };
        for (field, [low, high]) in &ranges {
            let (min, max, integer) =
                bounds(field).ok_or_else(|| format!("Unknown randomization field {field}"))?;
            if !low.is_finite()
                || !high.is_finite()
                || !(*high - *low).is_finite()
                || low > high
                || *low < min
                || *high > max
                || (integer && (low.fract() != 0.0 || high.fract() != 0.0))
            {
                return Err(format!(
                    "Invalid {field} range; native bounds {min}..{max}, integer={integer}"
                ));
            }
        }
        let mut notes: Vec<_> = document
            .notes
            .iter()
            .filter(|(_, _, selected)| *selected)
            .map(|(clip, note, _)| (clip, note))
            .collect();
        // Native row moves can reorder enumeration. Identity order fixes seed meaning.
        notes.sort_by(|a, b| (a.0, &a.1.detail().id).cmp(&(b.0, &b.1.detail().id)));
        if self.count == 0
            || self.count > notes.len()
            || self.changes_per_note == 0
            || self.changes_per_note > ranges.len()
        {
            return Err("Randomization requires a nonempty subset within selection and 1..eligible-fields changes per note".into());
        }
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        notes.shuffle(&mut rng);
        let mut edits = Vec::with_capacity(self.count);
        for (clip, midi) in notes.into_iter().take(self.count) {
            let note = midi.detail();
            let mut eligible: Vec<_> = ranges.iter().collect();
            eligible.shuffle(&mut rng);
            let mut changes = Vec::new();
            let mut operators = note.operators.clone();
            let mut changed_operators = false;
            let mut curves = BTreeMap::new();
            let mut chosen_fields = Vec::with_capacity(self.changes_per_note);
            for (field, [low, high]) in eligible.into_iter().take(self.changes_per_note) {
                chosen_fields.push(field.as_str());
                let integer = bounds(field).ok_or("Unknown field")?.2;
                let value =
                    if integer {
                        {
                            let old = match field.as_str() {
                                "channel" => f64::from(note.channel),
                                "key" => f64::from(note.key),
                                "muted" => f64::from(u8::from(note.muted)),
                                "chanceEnabled" => f64::from(u8::from(operators.chance_enabled)),
                                "occurrenceEnabled" => {
                                    f64::from(u8::from(operators.occurrence_enabled))
                                }
                                "recurrenceEnabled" => {
                                    f64::from(u8::from(operators.recurrence_enabled))
                                }
                                "repeatEnabled" => f64::from(u8::from(operators.repeat_enabled)),
                                "recurrenceLength" => f64::from(operators.recurrence_length),
                                "recurrenceMask" => f64::from(operators.recurrence_mask),
                                "repeatCount" => f64::from(operators.repeat_count),
                                "occurrence" => {
                                    use crate::editor::Occurrence::{
                                        Always, Fill, First, NotFill, NotFirst, NotPrev,
                                        NotPrevChannel, NotPrevKey, Prev, PrevChannel, PrevKey,
                                    };
                                    f64::from(match operators.occurrence {
                                        Always => 0,
                                        First => 1,
                                        NotFirst => 2,
                                        Prev => 3,
                                        NotPrev => 4,
                                        PrevChannel => 5,
                                        NotPrevChannel => 6,
                                        PrevKey => 7,
                                        NotPrevKey => 8,
                                        Fill => 9,
                                        NotFill => 10,
                                    })
                                }
                                // Point interpolation is sampled after choosing its point below.
                                _ => -999.0,
                            };
                            let mut choices: Vec<_> = (*low as i32..=*high as i32)
                                .filter(|v| f64::from(*v) != old)
                                .collect();
                            choices.shuffle(&mut rng);
                            f64::from(*choices.first().ok_or_else(|| {
                                format!("{field} range cannot change {}", note.id)
                            })?)
                        }
                    } else {
                        rng.random_range(*low..=*high)
                    };
                match field.as_str() {
                    "channel" => changes.push(Change::Channel(value as u8)),
                    "key" => changes.push(Change::Key(value as u8)),
                    "start" => changes.push(Change::Start(crate::protocol::native_time(value)?)),
                    "duration" => {
                        changes.push(Change::Duration(crate::protocol::native_time(value)?));
                    }
                    "muted" => changes.push(Change::Muted(value != 0.0)),
                    "velocity" => changes.push(Change::Velocity(value)),
                    "releaseVelocity" => changes.push(Change::ReleaseVelocity(value)),
                    "velocitySpread" => changes.push(Change::VelocitySpread(value)),
                    field if field.contains('.') => {
                        let (kind, property) =
                            field.split_once('.').ok_or("Invalid expression field")?;
                        let kind = expression(kind).ok_or("Unknown curve")?;
                        let points = curves.entry(kind).or_insert_with(|| {
                            note.expressions.get(&kind).cloned().unwrap_or_default()
                        });
                        if points.is_empty() {
                            return Err(format!(
                                "{} has no {kind:?} points; randomization does not invent curves",
                                note.id
                            ));
                        }
                        let index = rng.random_range(0..points.len());
                        let point = &mut points[index];
                        match property {
                            "value" => point.value = value,
                            "time" => point.time = crate::protocol::native_time(value)?,
                            "curvature" => point.curvature = value,
                            "spread" => point.spread = value,
                            "interpolation" => {
                                let old = if point.interpolation == Interpolation::Hold {
                                    0.0
                                } else {
                                    1.0
                                };
                                let next = if value != old {
                                    value
                                } else if *low != old {
                                    *low
                                } else if *high != old {
                                    *high
                                } else {
                                    return Err(format!(
                                        "{field} range cannot change the chosen point on {}",
                                        note.id
                                    ));
                                };
                                point.interpolation = if next == 0.0 {
                                    Interpolation::Hold
                                } else {
                                    Interpolation::Linear
                                };
                            }
                            _ => return Err("Unknown point property".into()),
                        }
                        // Native equal-time order is part of the curve. Preserve
                        // it unless a time change explicitly moves a point.
                        points.sort_by(crate::editor::compare_point_times);
                    }
                    field => {
                        changed_operators = true;
                        match field {
                            "chanceEnabled" => operators.chance_enabled = value != 0.0,
                            "chance" => operators.chance = value,
                            "occurrenceEnabled" => operators.occurrence_enabled = value != 0.0,
                            "occurrence" => {
                                use crate::editor::Occurrence::{
                                    Always, Fill, First, NotFill, NotFirst, NotPrev,
                                    NotPrevChannel, NotPrevKey, Prev, PrevChannel, PrevKey,
                                };
                                operators.occurrence = [
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
                                ][value as usize];
                            }
                            "recurrenceEnabled" => operators.recurrence_enabled = value != 0.0,
                            "recurrenceLength" => operators.recurrence_length = value as u8,
                            "recurrenceMask" => operators.recurrence_mask = value as u8,
                            "repeatEnabled" => operators.repeat_enabled = value != 0.0,
                            "repeatCount" => operators.repeat_count = value as i8,
                            "repeatCurve" => operators.repeat_curve = value,
                            "repeatVelocityEnd" => operators.repeat_velocity_end = value,
                            "repeatVelocityCurve" => operators.repeat_velocity_curve = value,
                            _ => return Err("Unknown operator".into()),
                        }
                    }
                }
            }
            if changed_operators {
                changes.push(Change::Operators(operators));
            }
            if !curves.is_empty() {
                changes.push(Change::Expressions(curves));
            }
            let mut desired: MidiNote = note.clone().try_into()?;
            for change in &changes {
                desired.change(change)?;
            }
            // A successful target must realize every sampled variable, including
            // after persistent-time quantization. Another changed field must not
            // hide a no-op from a singleton range or an unchanged curve point.
            let old = serde_json::to_value(note).map_err(|error| error.to_string())?;
            let new = serde_json::to_value(desired.detail()).map_err(|error| error.to_string())?;
            for field in chosen_fields {
                let effective = if let Some((kind, property)) = field.split_once('.') {
                    old["expressions"][kind].as_array().is_some_and(|points| {
                        points.iter().any(|point| {
                            new["expressions"][kind].as_array().is_some_and(|actual| {
                                actual.iter().any(|p| {
                                    p["id"] == point["id"] && p[property] != point[property]
                                })
                            })
                        })
                    })
                } else {
                    old.get(field).or_else(|| old["operators"].get(field))
                        != new.get(field).or_else(|| new["operators"].get(field))
                };
                if !effective {
                    return Err(format!(
                        "{field} range produced no native change on {}; no subset targets were skipped",
                        note.id
                    ));
                }
            }
            edits.push(Edit::Update {
                clip: clip.clone(),
                note: note.id.clone(),
                changes,
            });
        }
        Ok(edits)
    }
}
