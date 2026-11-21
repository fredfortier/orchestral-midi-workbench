//! Point intents compile to one ordered native curve in the ordinary note Plan.
use crate::{
    editor::{Interpolation, Point},
    protocol::native_time,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum PointEdit {
    Create {
        point: Point,
        before: Option<String>,
    },
    Update {
        point: String,
        changes: Vec<PointChange>,
        before: Option<String>,
    },
    Delete {
        point: String,
    },
    Clear,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "field",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum PointChange {
    Time(f64),
    Value(f64),
    Interpolation(Interpolation),
    Curvature(f64),
    Spread(f64),
}

impl PointEdit {
    /// Preserve native identities and stable ties; an explicit anchor reorders ties.
    /// # Errors
    /// Refuses unknown identities, repeated fields, invalid timing/anchors and no-ops.
    #[allow(clippy::float_cmp)] // Native persistent times and complete graph equality are exact.
    pub fn curve(&self, original: &[Point]) -> Result<Vec<Point>, String> {
        let mut points = original.to_vec();
        let (target, before) = match self {
            Self::Create { point, before } => {
                if points.iter().any(|old| old.id == point.id) {
                    return Err("Point already exists in this curve".into());
                }
                let mut point = point.clone();
                point.time = native_time(point.time)?;
                let target = point.id.clone();
                points.push(point);
                (Some(target), before.as_deref())
            }
            Self::Update {
                point,
                changes,
                before,
            } => {
                let target = points
                    .iter_mut()
                    .find(|p| p.id == *point)
                    .ok_or("Unknown point in this curve")?;
                let mut fields = HashSet::new();
                for change in changes {
                    if !fields.insert(std::mem::discriminant(change)) {
                        return Err("Repeated point update field".into());
                    }
                    match change {
                        PointChange::Time(value) => target.time = native_time(*value)?,
                        PointChange::Value(value) => target.value = *value,
                        PointChange::Interpolation(value) => target.interpolation = *value,
                        PointChange::Curvature(value) => target.curvature = *value,
                        PointChange::Spread(value) => target.spread = *value,
                    }
                }
                (Some(point.clone()), before.as_deref())
            }
            Self::Delete { point } => {
                let index = points
                    .iter()
                    .position(|p| p.id == *point)
                    .ok_or("Unknown point in this curve")?;
                points.remove(index);
                (None, None)
            }
            Self::Clear => {
                points.clear();
                (None, None)
            }
        };
        points.sort_by(crate::editor::compare_point_times);
        if let (Some(target), Some(anchor)) = (target, before) {
            if target == anchor {
                return Err("A point cannot precede itself".into());
            }
            let index = points
                .iter()
                .position(|p| p.id == target)
                .ok_or("Missing edited point")?;
            let anchor_time = points
                .iter()
                .find(|p| p.id == anchor)
                .ok_or("Unknown before-point in this curve")?
                .time;
            if points[index].time != anchor_time {
                return Err("Before-point must have the same native time".into());
            }
            let point = points.remove(index);
            let index = points
                .iter()
                .position(|p| p.id == anchor)
                .ok_or("Missing anchor")?;
            points.insert(index, point);
        }
        if points == original {
            return Err("Point intent makes no native change".into());
        }
        Ok(points)
    }
}
