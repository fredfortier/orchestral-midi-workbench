//! Bounded native mixer observations and absolute balance changes.
use crate::project_address::Path;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Parameter {
    pub native_value: f64,
    pub min: f64,
    pub max: f64,
    pub domain: String,
    pub unit: String,
    pub display_value: Option<f64>,
    pub linear_amplitude: Option<f64>,
    pub automated: bool,
    pub modulated: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TrackState {
    pub active: bool,
    pub mute: bool,
    pub solo: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    pub path: Path,
    pub kind: String,
    pub state: TrackState,
    pub effectively_muted: bool,
    pub volume: Parameter,
    pub pan: Parameter,
    pub output: serde_json::Value,
    pub sends: Vec<serde_json::Value>,
    pub devices: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BeatSeconds {
    pub beat: f64,
    pub seconds: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub document: String,
    pub revision: String,
    pub project: String,
    pub host_build: String,
    pub native_revision: String,
    pub capture_time: String,
    pub position_beats: f64,
    pub position_seconds: f64,
    pub beat_seconds: Vec<BeatSeconds>,
    pub tracks: Vec<Track>,
    pub limitations: Vec<String>,
}
impl Observation {
    /// # Errors
    /// Rejects invalid identities, duplicate owners and invalid physical values.
    pub fn validate(&self) -> Result<(), String> {
        if self.document.is_empty()
            || self.revision.is_empty()
            || self.tracks.is_empty()
            || self.capture_time.is_empty()
            || !self.position_beats.is_finite()
            || !self.position_seconds.is_finite()
        {
            return Err("Incomplete mixer observation".into());
        }
        if self.beat_seconds.len() > 4098
            || self
                .beat_seconds
                .iter()
                .any(|p| !p.beat.is_finite() || !p.seconds.is_finite())
        {
            return Err("Invalid beat-to-seconds context".into());
        }
        let mut owners = std::collections::BTreeSet::new();
        for track in &self.tracks {
            track.path.validate()?;
            if track.path.project != self.project
                || !owners.insert(serde_json::to_string(&track.path).map_err(|e| e.to_string())?)
            {
                return Err("Duplicate or mismatched mixer owner".into());
            }
            for p in [&track.volume, &track.pan] {
                if !p.native_value.is_finite()
                    || !p.min.is_finite()
                    || !p.max.is_finite()
                    || p.min > p.max
                    || p.native_value < p.min
                    || p.native_value > p.max
                    || p.display_value.is_some_and(|v| !v.is_finite())
                    || p.linear_amplitude
                        .is_some_and(|v| !v.is_finite() || v < 0.0)
                {
                    return Err("Invalid native mixer parameter".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Change {
    pub path: Path,
    pub volume_db: Option<f64>,
    pub pan: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Read {},
    Set {
        expected: Box<Observation>,
        changes: Vec<Change>,
    },
}
impl Request {
    /// # Errors
    /// Rejects empty, duplicate, nonfinite or out-of-scope balance requests.
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Set { expected, changes } = self {
            expected.validate()?;
            if changes.is_empty() || changes.len() > expected.tracks.len() {
                return Err("Require nonempty bounded mixer changes".into());
            }
            let mut paths = std::collections::BTreeSet::new();
            for c in changes {
                c.path.validate()?;
                if !paths.insert(serde_json::to_string(&c.path).map_err(|e| e.to_string())?)
                    || !expected.tracks.iter().any(|t| t.path == c.path)
                    || (c.volume_db.is_none() && c.pan.is_none())
                    || c.volume_db
                        .is_some_and(|v| !v.is_finite() || !(-120.0..=6.0).contains(&v))
                    || c.pan
                        .is_some_and(|v| !v.is_finite() || !(-1.0..=1.0).contains(&v))
                {
                    return Err("Invalid or duplicate mixer change".into());
                }
            }
        }
        Ok(())
    }

    /// Independently check requested balance and preservation of all other mixer facts.
    /// # Errors
    /// Rejects incomplete or mismatched native readback, without implying rollback.
    pub fn verify(&self, outcome: &Outcome) -> Result<(), String> {
        outcome.observation.validate()?;
        let Self::Set { expected, changes } = self else {
            return Ok(());
        };
        if outcome.error.is_some() {
            return Ok(());
        }
        let mut preserved = expected.as_ref().clone();
        preserved
            .capture_time
            .clone_from(&outcome.observation.capture_time);
        preserved.revision.clone_from(&outcome.observation.revision);
        preserved
            .native_revision
            .clone_from(&outcome.observation.native_revision);
        for change in changes {
            let actual = outcome
                .observation
                .tracks
                .iter()
                .find(|t| t.path == change.path)
                .ok_or("Mixer readback target disappeared; reconcile without replay")?;
            let wanted = preserved
                .tracks
                .iter_mut()
                .find(|t| t.path == change.path)
                .ok_or("Mixer target absent from expected scope")?;
            if let Some(db) = change.volume_db {
                if actual
                    .volume
                    .display_value
                    .is_none_or(|v| (v - db).abs() > 1e-10)
                    || actual
                        .volume
                        .linear_amplitude
                        .is_none_or(|v| (v - 10.0_f64.powf(db / 20.0)).abs() > 1e-10)
                {
                    return Err(
                        "Native volume differs from requested dB; reconcile without replay".into(),
                    );
                }
                wanted.volume.native_value = actual.volume.native_value;
                wanted.volume.display_value = actual.volume.display_value;
                wanted.volume.linear_amplitude = actual.volume.linear_amplitude;
            }
            if let Some(pan) = change.pan {
                if (actual.pan.native_value - pan).abs() > 1e-12 {
                    return Err("Native pan differs; reconcile without replay".into());
                }
                wanted.pan.native_value = actual.pan.native_value;
                wanted.pan.display_value = actual.pan.display_value;
            }
        }
        if preserved != outcome.observation {
            return Err(
                "Unrequested mixer facts changed; reconcile readback without replay".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub observation: Observation,
    pub error: Option<String>,
}
