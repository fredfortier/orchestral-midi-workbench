//! Intended track/receiver assignments. These declarations are never load receipts.
use crate::{
    Diagnostic, Result,
    model::{Performance, Route},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

/// Versioned reusable assignment model carried inside the single companion.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rendition {
    pub version: u32,
    pub tracks: Vec<Track>,
    pub instruments: Vec<Instrument>,
    pub bindings: Vec<Binding>,
    /// Explicit receiver instances; omit only for legacy one-binding-per-slot manifests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<Slot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_routes: Vec<ControlRoute>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitwig: Option<Bitwig>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<Mix>,
}
/// Portable CC address for native MIDI channel 1 on the named track.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRoute {
    pub track: String,
    pub route: Route,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Instrument {
    pub id: String,
    /// Human-readable slot label, independent of the instrument's stable ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub articulation_name: Option<String>,
    /// Exact absolute source file, independent of the companion's location.
    pub nki: PathBuf,
    pub player: Player,
    /// Intended private settings; successful observation belongs in evidence.
    pub settings: Vec<Setting>,
    /// Explicit library directory for the admitted missing-content resolver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_root: Option<PathBuf>,
}
/// A named receiver instance, independent of its reusable NKI resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub id: String,
    pub track: String,
    pub name: String,
    pub instrument: String,
    /// Complete admitted note-channel set, zero-based. Shared CC channel 0 is implicit.
    pub input_channels: Vec<u8>,
    /// Private configuration of this instance, such as its Syllabuilder phrase.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<Setting>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_name_evidence: Option<Evidence>,
}
/// Located observation/manual evidence, separate from musical intent.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Evidence {
    pub path: PathBuf,
    pub locator: String,
    pub claims: Vec<String>,
}
/// Properties with no suitable MIDI representation. Destination is the companion's folder.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bitwig {
    pub project_id: String,
    pub project_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    pub decision_revision: u64,
    pub native_meter_tick_rate: u8,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Mix {
    pub gain_db: Option<f64>,
    pub pan: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Setting {
    /// Exact control identity in the receiver's documented vocabulary.
    pub control: String,
    pub value: serde_json::Value,
    /// Unit or enumerated vocabulary; never inferred from a bare number.
    pub unit: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    /// UMP group/channel, both zero-based 0..15.
    pub route: Route,
    pub track: String,
    /// Zero-based Bitwig destination note channel. Articulated tracks use 1..15;
    /// 0 is reserved for shared CC. A single-channel part may use 0 for notes too.
    pub channel: u8,
    pub instrument: String,
    /// Named receiver instance. Required when the manifest declares explicit slots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
}
fn fail(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_RENDITION", message)
}
fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.contains('\0')
}

impl Rendition {
    /// Check complete reusable identities and one-to-one route/destination bindings.
    /// File contents and actual receiver settings require separate measurement.
    /// # Errors
    /// Returns unsupported versions, invalid resources, duplicate identities and
    /// missing/ambiguous/conflicting track, route or instrument bindings.
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.tracks.is_empty()
            || self.tracks.len() > 256
            || self.instruments.is_empty()
            || self.instruments.len() > 256
            || self.bindings.is_empty()
            || self.bindings.len() > 256
            || self.slots.len() > 256
        {
            return Err(fail(
                "unsupported version or track/instrument/binding budget",
            ));
        }
        if let Some(native) = &self.bitwig {
            if !identity(&native.project_id)
                || !label(&native.project_name)
                || native.project_name.contains(['/', '\\'])
                || native
                    .group_name
                    .as_deref()
                    .is_some_and(|name| !label(name))
                || native.decision_revision == 0
                || native.native_meter_tick_rate == 0
            {
                return Err(fail(
                    "invalid native project identity/revision/meter subdivision",
                ));
            }
        }
        let mut tracks = BTreeSet::new();
        let mut names = BTreeSet::new();
        for track in &self.tracks {
            if !identity(&track.id)
                || !tracks.insert(track.id.as_str())
                || !label(&track.name)
                || !names.insert(&track.name)
            {
                return Err(fail(format!(
                    "invalid/duplicate track identity or display name: {}",
                    track.id
                )));
            }
        }
        for track in &self.tracks {
            if track.mix.as_ref().is_some_and(|mix| {
                mix.gain_db.is_some_and(|v| !(-120.0..=6.0).contains(&v))
                    || mix.pan.is_some_and(|v| !(-1.0..=1.0).contains(&v))
            }) {
                return Err(fail(format!("invalid static mix: {}", track.id)));
            }
        }
        let mut instruments = BTreeSet::new();
        for instrument in &self.instruments {
            if !instruments.insert(instrument.id.as_str()) {
                return Err(fail(format!("duplicate instrument: {}", instrument.id)));
            }
            validate_instrument(instrument)?;
        }
        self.validate_bindings(&tracks, &instruments)
    }

    fn validate_slots<'a>(
        &'a self,
        tracks: &BTreeSet<&str>,
        instruments: &BTreeSet<&str>,
    ) -> Result<BTreeMap<&'a str, &'a Slot>> {
        let mut explicit_slots = BTreeMap::new();
        let mut named_slots = BTreeSet::new();
        let mut admitted = BTreeSet::new();
        for slot in &self.slots {
            if !identity(&slot.id)
                || explicit_slots.insert(slot.id.as_str(), slot).is_some()
                || !tracks.contains(slot.track.as_str())
                || !instruments.contains(slot.instrument.as_str())
                || !label(&slot.name)
                || !named_slots.insert((&slot.track, &slot.name))
                || slot.input_channels.is_empty()
                || slot.input_channels.len() > 16
            {
                return Err(fail("invalid, duplicate or unbound receiver slot"));
            }
            for channel in &slot.input_channels {
                if *channel > 15 || !admitted.insert((&slot.track, *channel)) {
                    return Err(fail("slot note-channel admission overlaps or is invalid"));
                }
            }
            validate_settings(&slot.settings)?;
        }
        let mut track_channels: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
        for slot in &self.slots {
            track_channels
                .entry(&slot.track)
                .or_default()
                .extend(&slot.input_channels);
        }
        for binding in &self.bindings {
            track_channels
                .entry(&binding.track)
                .or_default()
                .insert(binding.channel);
        }
        if track_channels
            .values()
            .any(|channels| channels.len() > 1 && channels.contains(&0))
        {
            return Err(fail(
                "articulated destination tracks reserve channel 1 for shared CC; use note channels 2..16",
            ));
        }
        Ok(explicit_slots)
    }

    fn validate_bindings(
        &self,
        tracks: &BTreeSet<&str>,
        instruments: &BTreeSet<&str>,
    ) -> Result<()> {
        let definitions: BTreeMap<_, _> = self
            .instruments
            .iter()
            .map(|i| (i.id.as_str(), i))
            .collect();
        let explicit_slots = self.validate_slots(tracks, instruments)?;
        let mut used_slots = BTreeSet::new();
        let mut slot_names = BTreeSet::new();
        let mut routes = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        let mut bound_tracks = BTreeSet::new();
        let mut bound_instruments = BTreeSet::new();
        for binding in &self.bindings {
            if binding.route.group > 15
                || binding.route.channel > 15
                || binding.channel > 15
                || !routes.insert(binding.route)
                || !destinations.insert((&binding.track, binding.channel))
                || !tracks.contains(binding.track.as_str())
                || !instruments.contains(binding.instrument.as_str())
            {
                return Err(fail(format!(
                    "missing, duplicate or conflicting route/destination/instrument: {binding:?}"
                )));
            }
            if self.slots.is_empty() {
                if binding.slot.is_some() {
                    return Err(fail("binding names a slot without a slot definition"));
                }
            } else {
                let slot = explicit_slots
                    .get(binding.slot.as_deref().unwrap_or(""))
                    .ok_or_else(|| fail("binding requires a declared receiver slot"))?;
                if slot.track != binding.track
                    || slot.instrument != binding.instrument
                    || !slot.input_channels.contains(&binding.channel)
                {
                    return Err(fail(
                        "binding does not match its slot track, resource or note channels",
                    ));
                }
                used_slots.insert(slot.id.as_str());
            }
            if self.slots.is_empty() {
                if let Some(name) = &definitions[binding.instrument.as_str()].articulation_name {
                    if !slot_names.insert((&binding.track, name)) {
                        return Err(fail("duplicate articulation display name within a track"));
                    }
                }
            }
            bound_tracks.insert(binding.track.as_str());
            bound_instruments.insert(binding.instrument.as_str());
        }
        if used_slots.len() != self.slots.len() {
            return Err(fail("unbound receiver slot"));
        }
        if *tracks != bound_tracks || *instruments != bound_instruments {
            return Err(fail("unbound track or instrument definition"));
        }
        let mut control_tracks = BTreeSet::new();
        let mut control_addresses = BTreeSet::new();
        for control in &self.control_routes {
            if control.route.group > 15
                || control.route.channel != 0
                || !tracks.contains(control.track.as_str())
                || !control_tracks.insert(&control.track)
                || !control_addresses.insert(control.route)
                || self.bindings.iter().any(|binding| {
                    binding.route == control.route
                        && (binding.track != control.track || binding.channel != 0)
                })
            {
                return Err(fail("invalid or conflicting shared track CC route"));
            }
        }
        Ok(())
    }

    /// Validate every musical-note and channel-event route against this rendition.
    /// Unused bindings are stale, rather than an implicit request for extra players.
    /// # Errors
    /// Returns invalid assignments, missing routes and stale/unused bindings.
    pub fn validate_performance(&self, performance: &Performance) -> Result<()> {
        self.validate()?;
        let bindings: BTreeMap<_, _> = self
            .bindings
            .iter()
            .map(|binding| (binding.route, binding))
            .collect();
        let control_routes: BTreeSet<_> = self.control_routes.iter().map(|c| c.route).collect();
        let mut occupied: BTreeSet<_> = performance
            .notes
            .iter()
            .map(|note| note.route)
            .chain(performance.piano_routes.iter().copied())
            .collect();
        for event in performance.configuration.iter().chain(&performance.events) {
            let Some(route) = crate::wire::route(&event.words) else {
                continue;
            };
            if control_routes.contains(&route)
                && matches!(crate::wire::channel_cc(&event.words), Some((_, 0..=119)))
            {
                continue;
            }
            occupied.insert(route);
        }
        for route in &occupied {
            if !bindings.contains_key(route) {
                return Err(fail(format!(
                    "occupied encoded route has no binding: {route:?}"
                )));
            }
        }
        for route in bindings.keys() {
            if !occupied.contains(route) {
                return Err(fail(format!(
                    "stale binding has no performed route: {route:?}"
                )));
            }
        }
        Ok(())
    }
}

fn validate_instrument(instrument: &Instrument) -> Result<()> {
    if !identity(&instrument.id)
        || instrument
            .articulation_name
            .as_deref()
            .is_some_and(|name| !label(name))
        || !instrument.nki.is_absolute()
        || instrument.nki.extension().is_none_or(|e| e != "nki")
    {
        return Err(fail(format!(
            "invalid/duplicate instrument or exact NKI resource: {}",
            instrument.id
        )));
    }
    if instrument
        .sample_root
        .as_ref()
        .is_some_and(|root| !root.is_absolute())
    {
        return Err(fail(format!(
            "sample root must be absolute: {}",
            instrument.id
        )));
    }
    let player = &instrument.player;
    if [&player.id, &player.name, &player.version, &player.format]
        .iter()
        .any(|v| !label(v))
    {
        return Err(fail(format!(
            "incomplete player requirements: {}",
            instrument.id
        )));
    }
    if player.initial_name.is_some() != player.initial_name_evidence.is_some()
        || player
            .initial_name
            .as_ref()
            .is_some_and(|name| !label(name))
    {
        return Err(fail(format!(
            "initial player name requires paired evidence: {}",
            instrument.id
        )));
    }
    if let Some(evidence) = &player.initial_name_evidence {
        if !evidence.path.is_absolute()
            || !label(&evidence.locator)
            || evidence.claims.is_empty()
            || evidence.claims.len() > 128
            || evidence.claims.iter().any(|claim| !label(claim))
        {
            return Err(fail(format!(
                "invalid player insertion evidence: {}",
                instrument.id
            )));
        }
    }
    validate_settings(&instrument.settings)
}

fn validate_settings(settings: &[Setting]) -> Result<()> {
    if settings.len() > 128 {
        return Err(fail("receiver settings budget exceeded"));
    }
    let mut controls = BTreeSet::new();
    for setting in settings {
        if !label(&setting.control)
            || !label(&setting.unit)
            || setting.value.is_null()
            || !controls.insert(&setting.control)
        {
            return Err(fail(format!(
                "invalid/duplicate required setting: {}",
                setting.control
            )));
        }
    }
    Ok(())
}
