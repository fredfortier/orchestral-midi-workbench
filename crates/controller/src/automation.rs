//! Automation point edits over complete clip observations, including all notes.
use crate::editor::Point;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_POINTS: usize = 100_000;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Curve {
    pub owner: crate::project_address::Path,
    pub placement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_clip: Option<String>,
    #[serde(default)]
    pub track: String,
    #[serde(default)]
    pub lane: String,
    #[serde(default)]
    pub clip: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub reference_count: usize,
    #[serde(default)]
    pub mapping: Value,
    pub target: Value,
    pub points: Vec<Point>,
}

pub type Observation = crate::protocol::Snapshot;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub track: String,
    pub lane: String,
    pub clip: String,
    /// Complete set of committed placements sharing this source, explicitly acknowledged.
    pub aliases: Vec<String>,
    pub points: Vec<Point>,
}

fn identity(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && !id.contains('\0')
}

fn points_valid(points: &[Point]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    if points.len() > MAX_POINTS {
        return Err("Automation source exceeds 100000 points".into());
    }
    for p in points {
        if !identity(&p.id)
            || !ids.insert(&p.id)
            || !p.time.is_finite()
            || !p.value.is_finite()
            || !p.curvature.is_finite()
            || !(-1.0..=1.0).contains(&p.curvature)
            || !p.spread.is_finite()
            || p.spread < 0.0
        {
            return Err("Invalid automation point identity/time/value/curvature/spread".into());
        }
    }
    if let Some(pair) = points.windows(2).find(|p| p[0].time > p[1].time) {
        return Err(format!(
            "Automation points must retain stable source-time order ({} at {} before {} at {})",
            pair[0].id, pair[0].time, pair[1].id, pair[1].time
        ));
    }
    Ok(())
}

fn validate_replacements(edits: &[Replacement]) -> Result<(), String> {
    if edits.is_empty() || edits.len() > 256 {
        return Err("Automation batch requires 1–256 source edits".into());
    }
    let mut owners = BTreeSet::new();
    let mut total = 0usize;
    for edit in edits {
        if [&edit.track, &edit.lane, &edit.clip]
            .iter()
            .any(|id| !identity(id))
            || !owners.insert((&edit.track, &edit.lane, &edit.clip))
            || edit.aliases.is_empty()
            || edit.aliases.len() > 4096
            || edit.aliases.iter().any(|id| !identity(id))
            || edit.aliases.iter().collect::<BTreeSet<_>>().len() != edit.aliases.len()
        {
            return Err("Invalid/duplicate automation owner or alias acknowledgement".into());
        }
        points_valid(&edit.points)?;
        if edit
            .points
            .iter()
            .any(|p| p.interpolation == crate::editor::Interpolation::Hold && p.curvature != 0.0)
        {
            return Err("Native HOLD points require zero curvature for stable Undo/Redo".into());
        }
        total = total
            .checked_add(edit.points.len())
            .ok_or("Automation point count overflow")?;
    }
    if total > MAX_POINTS {
        return Err("Automation batch exceeds 100000 resulting points".into());
    }
    Ok(())
}

impl Observation {
    fn curves(&self) -> impl Iterator<Item = &Curve> {
        self.automation.iter()
    }
    fn native_identities(&self) -> BTreeSet<String> {
        self.curves()
            .flat_map(|c| c.points.iter().map(|p| p.id.clone()))
            .collect()
    }
    /// # Errors
    /// Refuses stale scope, hidden aliases, cross-source identities and invalid target values.
    pub fn preflight_automation(&self, edits: &[Replacement]) -> Result<(), String> {
        self.validate()?;
        validate_replacements(edits)?;
        let known = self.native_identities();
        let mut temporary = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for edit in edits {
            let curve = self
                .curves()
                .find(|c| c.track == edit.track && c.lane == edit.lane && c.clip == edit.clip)
                .ok_or("Unknown automation owner")?;
            if !sources.insert(&curve.source) {
                return Err("Duplicate automation source edit".into());
            }
            if curve.aliases.iter().collect::<BTreeSet<_>>()
                != edit.aliases.iter().collect::<BTreeSet<_>>()
                || curve.reference_count != curve.aliases.len()
            {
                return Err("Automation aliases differ or source has unobserved references".into());
            }
            let min = curve.target["min"]
                .as_f64()
                .ok_or("Missing physical minimum")?;
            let max = curve.target["max"]
                .as_f64()
                .ok_or("Missing physical maximum")?;
            if curve.target["pointType"] != "continuous"
                || curve.target["domain"] != "LINEAR"
                || !min.is_finite()
                || !max.is_finite()
                || min >= max
            {
                return Err("Unsupported automation physical domain".into());
            }
            let originals: BTreeMap<_, _> =
                curve.points.iter().map(|p| (p.id.as_str(), p)).collect();
            for p in &edit.points {
                if p.value < min || p.value > max {
                    return Err("Automation point outside physical target range".into());
                }
                if !originals.contains_key(p.id.as_str())
                    && (known.contains(&p.id) || !temporary.insert(&p.id))
                {
                    return Err("Automation temporary identity collides with another source".into());
                }
                let old = originals.get(p.id.as_str());
                if old.is_none_or(|old| old.time.to_bits() != p.time.to_bits()) {
                    let canonical = crate::protocol::native_time(p.time)?;
                    if canonical.to_bits() != p.time.to_bits() {
                        return Err(
                            "New/changed automation times must use native 2^-20 beat resolution"
                                .into(),
                        );
                    }
                }
            }
            if edit.points == curve.points {
                return Err("Automation edit makes no native change".into());
            }
        }
        Ok(())
    }
}

impl Curve {
    /// Match a requested creation to independently observed native automation.
    #[must_use]
    pub fn matches_target(&self, actual: &Self) -> bool {
        self.owner == actual.owner
            && self.note_clip == actual.note_clip
            && self.target["kind"] == actual.target["kind"]
            && if self.target["kind"] == "parameter" {
                self.target["nativeParameterId"] == actual.target["nativeParameterId"]
            } else {
                self.target["channel"] == actual.target["channel"]
                    && self.target["controller"] == actual.target["controller"]
            }
    }
    #[must_use]
    pub fn is_new(&self) -> bool {
        self.lane.is_empty()
    }
    /// # Errors
    /// Rejects incomplete native data or malformed new channel-CC automation.
    pub fn validate(&self) -> Result<(), String> {
        self.owner.validate()?;
        points_valid(&self.points).map_err(|error| {
            format!(
                "Track {}, lane {}, source {}: {error}",
                self.track, self.lane, self.source
            )
        })?;
        if (self.placement == "launcher") != self.note_clip.is_some()
            || self.note_clip.as_deref().is_some_and(|id| !identity(id))
        {
            return Err("Launcher automation requires its native note clip identity".into());
        }
        if self.is_new() {
            let (minimum, maximum) = if self.target["kind"] == "parameter" {
                let device: crate::project_address::Path =
                    serde_json::from_value(self.target["device"].clone())
                        .map_err(|_| "New parameter automation requires a qualified device path")?;
                device.validate()?;
                let mut parent = device.clone();
                parent.elements.truncate(self.owner.elements.len());
                if parent != self.owner
                    || !matches!(
                        device.elements.last(),
                        Some(
                            crate::project_address::Element::Device { .. }
                                | crate::project_address::Element::Plugin { .. }
                                | crate::project_address::Element::Layer { .. }
                                | crate::project_address::Element::Selector { .. }
                        )
                    )
                    || self.target["nativeParameterId"]
                        .as_str()
                        .is_none_or(|v| !identity(v))
                {
                    return Err("New parameter automation requires its owning track and observed parameter identity".into());
                }
                let min = self.target["min"]
                    .as_f64()
                    .ok_or("Missing parameter minimum")?;
                let max = self.target["max"]
                    .as_f64()
                    .ok_or("Missing parameter maximum")?;
                if !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || self.target["domain"] != "LINEAR"
                    || self.target["pointType"] != "continuous"
                {
                    return Err("Unsupported parameter automation domain".into());
                }
                (min, max)
            } else if self.target["kind"] == "midiCc"
                && self.target["channel"].as_u64().is_some_and(|c| c <= 15)
                && self.target["controller"].as_u64().is_some_and(|c| c <= 119)
            {
                (0.0, 1.0)
            } else {
                return Err("Unsupported new automation target".into());
            };
            if !["arranger", "launcher"].contains(&self.placement.as_str())
                || !self.track.is_empty()
                || !self.clip.is_empty()
                || !self.source.is_empty()
                || !self.aliases.is_empty()
                || self.reference_count != 0
                || !(self.mapping.is_null() || self.mapping == serde_json::json!({}))
                || self.points.first().is_none_or(|p| p.time != 0.0)
                || self
                    .points
                    .iter()
                    .any(|p| p.time < 0.0 || !(minimum..=maximum).contains(&p.value))
            {
                return Err("New automation requires an observed owner and in-range points starting at beat zero".into());
            }
        } else if !["clip", "arranger", "launcher"].contains(&self.placement.as_str())
            || [&self.track, &self.lane, &self.clip, &self.source]
                .iter()
                .any(|id| !identity(id))
        {
            return Err("Invalid native automation owner/placement".into());
        }
        Ok(())
    }
}
