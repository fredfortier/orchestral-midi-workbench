//! Additive performance landmarks on the project Arranger timeline.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    pub name: String,
    /// Arranger quarter-note beats, never clip-source time or a bar number.
    pub time: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub markers: Vec<Marker>,
}

fn key(name: &str) -> Result<&str, String> {
    let (key, label) = name.split_once(" · ").ok_or("Cue requires KEY · Label")?;
    let digits = key.strip_prefix("PC").ok_or("Cue key must start PC")?;
    if !(2..=4).contains(&digits.len())
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || label.trim().is_empty()
        || label.trim() != label
        || name.len() > 64
        || name.chars().any(char::is_control)
    {
        return Err("Cue requires PC plus 2–4 digits and a short label (64 UTF-8 bytes)".into());
    }
    Ok(key)
}
impl Create {
    /// # Errors
    /// Rejects duplicate keys/times, malformed labels and non-native arranger times.
    pub fn validate(&self) -> Result<(), String> {
        if self.markers.is_empty() || self.markers.len() > 128 {
            return Err("Create 1–128 cue markers per batch".into());
        }
        let mut keys = BTreeSet::new();
        let mut times = BTreeSet::new();
        for marker in &self.markers {
            if !keys.insert(key(&marker.name)?)
                || !marker.time.is_finite()
                || marker.time.is_sign_negative()
                || !crate::protocol::native_time(marker.time)
                    .is_ok_and(|t| t.to_bits() == marker.time.to_bits())
                || !times.insert(marker.time.to_bits())
            {
                return Err("Duplicate cue key/time or unresolved nonnegative native beat".into());
            }
        }
        Ok(())
    }
}

/// Marker-only authority and readback. Native IDs distinguish existing objects during preservation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    pub document: String,
    pub project_uuid: String,
    pub revision: String,
    pub markers: Vec<ObservedMarker>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ObservedMarker {
    pub id: String,
    #[serde(flatten)]
    pub marker: Marker,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Read {},
    Create {
        operation: String,
        document: String,
        revision: String,
        cues: Create,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub before: Option<Scope>,
    pub observation: Scope,
    pub error: Option<String>,
}
impl Scope {
    /// # Errors
    /// Refuses incomplete or duplicate native marker identity and invalid beats.
    pub fn validate(&self) -> Result<(), String> {
        if self.document.is_empty()
            || self.project_uuid.is_empty()
            || self.revision.is_empty()
            || self.markers.len() > 4096
        {
            return Err("Invalid scoped cue observation".into());
        }
        let mut ids = BTreeSet::new();
        for marker in &self.markers {
            if marker.id.is_empty()
                || !ids.insert(&marker.id)
                || !marker.marker.time.is_finite()
                || marker.marker.time < 0.0
            {
                return Err("Invalid native marker identity/time".into());
            }
        }
        Ok(())
    }
}
/// Verify actual additions and untouched marker identity/order against independent native reads.
/// # Errors
/// Refuses stale authority, collisions, partial effects or any unrelated marker change.
pub fn verify_scoped(request: &Request, result: &Outcome) -> Result<(), String> {
    result.observation.validate()?;
    if let Some(error) = &result.error {
        return Err(format!(
            "Cue execution has partial/unknown effects: {error}"
        ));
    }
    let Request::Create {
        document,
        revision,
        cues,
        ..
    } = request
    else {
        return if result.before.is_none() {
            Ok(())
        } else {
            Err("Read returned mutation evidence".into())
        };
    };
    cues.validate()?;
    let before = result
        .before
        .as_ref()
        .ok_or("Missing original marker scope")?;
    before.validate()?;
    if &before.document != document
        || &before.revision != revision
        || before.document != result.observation.document
        || before.project_uuid != result.observation.project_uuid
    {
        return Err("Cue document/authority differs".into());
    }
    for wanted in &cues.markers {
        let k = key(&wanted.name)?;
        if before
            .markers
            .iter()
            .any(|m| m.marker.name == k || m.marker.name.starts_with(&format!("{k} · ")))
        {
            return Err(format!("Cue key collision: {k}"));
        }
    }
    let retained: Vec<_> = result
        .observation
        .markers
        .iter()
        .filter(|m| before.markers.iter().any(|old| old.id == m.id))
        .collect();
    if retained != before.markers.iter().collect::<Vec<_>>() {
        return Err("Existing cue identity, order or value changed".into());
    }
    let added: Vec<_> = result
        .observation
        .markers
        .iter()
        .filter(|m| !before.markers.iter().any(|old| old.id == m.id))
        .collect();
    if added.len() != cues.markers.len()
        || cues
            .markers
            .iter()
            .any(|wanted| added.iter().filter(|m| &m.marker == wanted).count() != 1)
    {
        return Err("Native cue additions differ from intent".into());
    }
    Ok(())
}
