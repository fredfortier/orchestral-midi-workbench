use super::{CurvePolicy, Dynamics, Meaning, fail};
use crate::{
    Result,
    model::{Event, Note, Performance, Rational, Route},
    wire,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Controls {
    notes: BTreeMap<(Route, u8), Vec<DynamicState>>,
    pub(super) initial: BTreeMap<(Route, u8), u32>,
    point_count: usize,
}
struct DynamicState {
    end: Rational,
    points: Vec<(Rational, u32, u32)>,
    source: String,
}
#[derive(Default)]
struct Changes {
    ends: Vec<usize>,
    points: Vec<(usize, u32, u32)>,
}
impl Controls {
    pub(super) fn record(
        &mut self,
        note: &mut Note,
        dynamics: &Dynamics,
        meaning: &Meaning,
    ) -> Result<()> {
        let Dynamics::Crossfade {
            controller,
            key_velocity,
            source_law,
            expression,
            curve_policy,
        } = dynamics
        else {
            if !meaning.curve.is_empty() {
                return Err(fail(format!(
                    "key-velocity receiver cannot render evolving dynamics: {}",
                    note.source
                )));
            }
            if let Dynamics::FixedAttack { key_velocity } = dynamics {
                if *key_velocity == 0 {
                    return Err(fail("fixed attack velocity must be positive"));
                }
                note.velocity = *key_velocity;
            }
            return Ok(());
        };
        let points = if meaning.curve.is_empty() {
            let value = meaning
                .level
                .or_else(|| match source_law {
                    super::SourceLaw::RequireDynamic => None,
                    super::SourceLaw::DynamicOrVelocity16 => Some(u32::from(note.velocity) * 65537),
                })
                .ok_or_else(|| {
                    fail(format!(
                        "missing required dynamic level for {}",
                        note.source
                    ))
                })?;
            vec![(note.onset, value, note.on_order)]
        } else {
            if *curve_policy != CurvePolicy::ChannelHold {
                return Err(fail(format!(
                    "required evolving dynamic curve needs explicit channelHold policy: {}",
                    note.source
                )));
            }
            curve_points(note, meaning)?
        };
        if points.len() > 1_000_000 - self.point_count {
            return Err(fail("receiver control-state point budget"));
        }
        self.point_count += points.len();
        note.velocity = *key_velocity;
        self.notes
            .entry((note.route, *controller))
            .or_default()
            .push(DynamicState {
                end: note.end,
                points,
                source: note.source.clone(),
            });
        if let Some(expression) = expression {
            let key = (note.route, expression.controller);
            if self
                .initial
                .insert(key, expression.value)
                .is_some_and(|old| old != expression.value)
            {
                return Err(fail(format!(
                    "conflicting controller initialization at {key:?}"
                )));
            }
        }
        Ok(())
    }

    pub(super) fn finish(mut self, performance: &mut Performance) -> Result<()> {
        for ((route, controller), notes) in self.notes {
            let mut timeline = BTreeMap::<Rational, Changes>::new();
            for (index, note) in notes.iter().enumerate() {
                timeline.entry(note.end).or_default().ends.push(index);
                for &(time, value, order) in &note.points {
                    timeline
                        .entry(time)
                        .or_default()
                        .points
                        .push((index, value, order));
                }
            }
            let mut active = BTreeMap::<usize, u32>::new();
            let mut levels = BTreeMap::<u32, BTreeSet<usize>>::new();
            let mut previous = None;
            for (time, changes) in timeline {
                for index in changes.ends {
                    remove(index, &mut active, &mut levels);
                }
                let order = changes.points.iter().map(|p| p.2).min().unwrap_or(0);
                for (index, value, _) in changes.points {
                    remove(index, &mut active, &mut levels);
                    active.insert(index, value);
                    levels.entry(value).or_default().insert(index);
                }
                if levels.len() > 1 {
                    let sources: Vec<_> = levels
                        .values()
                        .filter_map(|s| s.first())
                        .map(|i| notes[*i].source.as_str())
                        .take(2)
                        .collect();
                    return Err(fail(format!(
                        "conflicting simultaneous dynamics at {route:?} CC{controller}, beat {time}: {}",
                        sources.join(" / ")
                    )));
                }
                if let Some((&value, _)) = levels.first_key_value() {
                    if previous.is_none() {
                        self.initial.insert((route, controller), value);
                    } else if previous != Some(value) {
                        performance.events.push(Event {
                            time,
                            order,
                            words: wire::cc(route, controller, value),
                        });
                    }
                    previous = Some(value);
                }
            }
        }
        for ((route, controller), value) in self.initial {
            performance.configuration.push(Event {
                time: Rational::from_integer(0),
                order: 0,
                words: wire::cc(route, controller, value),
            });
        }
        Ok(())
    }
}

fn remove(
    index: usize,
    active: &mut BTreeMap<usize, u32>,
    levels: &mut BTreeMap<u32, BTreeSet<usize>>,
) {
    if let Some(old) = active.remove(&index) {
        if let Some(indices) = levels.get_mut(&old) {
            indices.remove(&index);
            if indices.is_empty() {
                levels.remove(&old);
            }
        }
    }
}

fn curve_points(note: &Note, meaning: &Meaning) -> Result<Vec<(Rational, u32, u32)>> {
    let zero = Rational::from_integer(0);
    if meaning.curve.first().is_none_or(|p| p.0 != zero) {
        return Err(fail(format!(
            "dynamic curve must start at offset zero: {}",
            note.source
        )));
    }
    let mut points = Vec::with_capacity(meaning.curve.len());
    let mut previous = None;
    for &(offset, value, order) in &meaning.curve {
        if offset < zero || previous.is_some_and(|p| p >= offset) {
            return Err(fail(format!(
                "dynamic curve offsets must strictly increase: {}",
                note.source
            )));
        }
        previous = Some(offset);
        // Wide arithmetic avoids intermediate overflow before checking storage precision.
        let wide =
            |r: Rational| num_rational::Ratio::new(i128::from(*r.numer()), i128::from(*r.denom()));
        let time = wide(note.onset) + wide(offset);
        let time = Rational::new(
            i64::try_from(*time.numer()).map_err(|_| fail("curve time overflow"))?,
            i64::try_from(*time.denom()).map_err(|_| fail("curve time precision overflow"))?,
        );
        if time > note.end || note.onset < zero || note.end <= note.onset {
            return Err(fail(format!(
                "dynamic curve outside note lifetime: {}",
                note.source
            )));
        }
        if time < note.end {
            points.push((
                time,
                value,
                if offset == zero && order == 0 {
                    note.on_order
                } else {
                    order
                },
            ));
        }
    }
    Ok(points)
}
