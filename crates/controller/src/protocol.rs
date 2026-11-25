//! Versioned messages for a bidirectional document-adapter session.
use crate::editor::{Change, MidiNote, NoteEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const VERSION: u8 = 75;

/// Validate an adapter-owned operation token or bounded native history cursor.
#[must_use]
pub fn valid_revision(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-:/".contains(&b))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Selection {
    pub editor: String,
    pub kind: String,
    pub from: f64,
    pub to: f64,
    pub lanes: Vec<String>,
    pub objects: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub document: String,
    /// Adapter-owned authority for the observed selection, content and mappings.
    pub revision: String,
    pub selection: Selection,
    pub clips: Vec<Clip>,
    pub owners: Vec<crate::project_address::Path>,
    pub paths: Option<Vec<crate::project_address::Path>>,
    pub automation: Vec<crate::automation::Curve>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Clip {
    pub path: crate::project_address::Path,
    pub id: String,
    pub lane: String,
    pub track: String,
    pub content: String,
    pub start: f64,
    pub duration: f64,
    pub source_start: f64,
    pub source_end: f64,
    pub loop_start: f64,
    pub loop_length: f64,
    pub loop_enabled: bool,
    pub notes: Vec<SelectedNote>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectedNote {
    pub note: NoteEvent,
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Edit {
    Create {
        clip: String,
        temporary: String,
        note: NoteEvent,
    },
    Update {
        clip: String,
        note: String,
        changes: Vec<Change>,
    },
    Delete {
        clip: String,
        note: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plan {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub automation_create: Vec<crate::automation::Curve>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub automation_delete: Vec<crate::automation::Curve>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub automation: Vec<crate::automation::Replacement>,
    pub document: String,
    pub revision: String,
    pub edits: Vec<Edit>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ClientMessage {
    LauncherObservation {
        request: String,
        result: crate::launcher::Observation,
    },
    SelectionObservation {
        request: String,
        result: crate::selection::Observation,
    },
    CaptureObservation {
        request: String,
        capture: Box<crate::export_midi::Capture>,
    },
    MixObservation {
        request: String,
        result: crate::mix::Outcome,
    },
    ExportAudioObservation {
        request: String,
        result: serde_json::Value,
    },
    EntityObservation {
        request: String,
        entity: crate::entity::Entity,
    },
    CuesObservation {
        request: String,
        result: crate::cues::Outcome,
    },
    InstrumentObservation {
        request: String,
        result: serde_json::Value,
    },
    MidiObservation {
        request: String,
        midi: Box<crate::midi::Observation>,
    },
    Observation {
        request: String,
        operation: Option<String>,
        snapshot: Snapshot,
    },
    CommandRefused {
        request: String,
        reason: String,
    },
    CommandError {
        request: String,
        reason: String,
    },
    #[serde(rename_all = "camelCase")]
    Hello {
        version: u8,
        adapter: String,
        host_build: String,
    },
    /// Internal proposal compiled from a correlated native observation.
    #[serde(skip)]
    Propose {
        operation: String,
        snapshot: Snapshot,
        plan: Plan,
    },
    Applied {
        operation: String,
        snapshot: Snapshot,
        identities: BTreeMap<String, String>,
    },
    Rejected {
        operation: String,
        reason: String,
    },
    Status {
        operation: String,
    },
}

#[cfg(test)]
mod wire_tests {
    use super::ClientMessage;

    #[test]
    fn adapter_cannot_supply_an_internal_edit_proposal() {
        let error = serde_json::from_str::<ClientMessage>(r#"{"type":"propose"}"#).unwrap_err();
        assert!(error.to_string().contains("unknown variant `propose`"));
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ServerMessage {
    DocumentRequest {
        request: String,
        command: crate::controller::DocumentCommand,
    },
    #[serde(rename_all = "camelCase")]
    Welcome {
        version: u8,
        epoch: String,
        max_message_bytes: usize,
    },
    Plan {
        operation: String,
        plan: Plan,
    },
    Complete {
        operation: String,
    },
    Error {
        operation: Option<String>,
        reason: String,
    },
    Unknown {
        operation: String,
    },
}

pub struct Document {
    pub snapshot: Snapshot,
    /// Every note is normalized on receipt, including unselected collision context.
    pub notes: Vec<(String, MidiNote, bool)>,
}

impl TryFrom<Snapshot> for Document {
    type Error = String;
    fn try_from(snapshot: Snapshot) -> Result<Self, String> {
        let selection = &snapshot.selection;
        if snapshot.document.is_empty()
            || snapshot.revision.is_empty()
            || !selection.from.is_finite()
            || !selection.to.is_finite()
            || selection.to < selection.from
        {
            return Err("Invalid document/selection context".into());
        }
        let mut clips = HashSet::new();
        let mut notes = Vec::new();
        for clip in &snapshot.clips {
            if clip.id.is_empty()
                || !clips.insert(&clip.id)
                || clip.lane.is_empty()
                || clip.track.is_empty()
                || clip.content.is_empty()
                || [
                    clip.start,
                    clip.duration,
                    clip.source_start,
                    clip.source_end,
                    clip.loop_start,
                    clip.loop_length,
                ]
                .iter()
                .any(|v| !v.is_finite())
                || clip.duration <= 0.0
                || clip.source_end <= clip.source_start
                || (clip.loop_enabled && clip.loop_length <= 0.0)
            {
                return Err("Invalid clip ownership/mapping".into());
            }
            let mut ids = HashSet::new();
            for n in &clip.notes {
                if !ids.insert(&n.note.id) {
                    return Err("Duplicate source identity".into());
                }
                notes.push((clip.id.clone(), n.note.clone().try_into()?, n.selected));
            }
        }
        Ok(Self { snapshot, notes })
    }
}

impl Document {
    fn validate_new_identities(&self, plan: &Plan) -> Result<HashSet<String>, String> {
        let mut known = HashSet::new();
        for (_, note, _) in &self.notes {
            known.insert(note.detail().id.as_str());
            for point in note.detail().expressions.values().flatten() {
                known.insert(point.id.as_str());
            }
        }
        {
            for lane in &self.snapshot.automation {
                for point in &lane.points {
                    known.insert(point.id.as_str());
                }
            }
        }
        let mut temporary = HashSet::new();
        let mut claim = |id: &str| {
            if id.is_empty() || known.contains(id) || !temporary.insert(id.to_owned()) {
                Err(
                    "New note/point temporary identities must be unique across the whole plan"
                        .to_owned(),
                )
            } else {
                Ok(())
            }
        };
        for edit in &plan.edits {
            match edit {
                Edit::Create {
                    temporary, note, ..
                } => {
                    claim(temporary)?;
                    for point in note.expressions.values().flatten() {
                        claim(&point.id)?;
                    }
                }
                Edit::Update {
                    clip,
                    note,
                    changes,
                } => {
                    let original = self
                        .notes
                        .iter()
                        .find(|(owner, value, _)| owner == clip && value.detail().id == *note)
                        .ok_or("Unknown note")?
                        .1
                        .detail();
                    for change in changes {
                        if let Change::Expressions(curves) = change {
                            for (kind, points) in curves {
                                for point in points {
                                    if !original.expressions.get(kind).is_some_and(|old| {
                                        old.iter().any(|value| value.id == point.id)
                                    }) {
                                        claim(&point.id)?;
                                    }
                                }
                            }
                        }
                    }
                }
                Edit::Delete { .. } => {}
            }
        }
        for edit in &plan.automation {
            let curve = self
                .snapshot
                .automation
                .iter()
                .find(|c| c.track == edit.track && c.lane == edit.lane && c.clip == edit.clip)
                .ok_or("Missing automation owner")?;
            if edit.points.is_empty() {
                claim(&format!("default:{}", curve.source))?;
            }
            for point in &edit.points {
                if !curve.points.iter().any(|p| p.id == point.id) {
                    claim(&point.id)?;
                }
            }
        }
        for lane in &plan.automation_create {
            for point in &lane.points {
                claim(&point.id)?;
            }
        }
        Ok(temporary)
    }

    #[allow(clippy::too_many_lines, clippy::float_cmp)] // One complete desired graph, reused by result verification and byte admission.
    fn projected_snapshot(
        &self,
        plan: &Plan,
        identities: &BTreeMap<String, String>,
        observed: &Snapshot,
    ) -> Result<Snapshot, String> {
        let mut expected = self.snapshot.clone();
        let mut expression_edits = HashSet::new();
        for edit in &plan.edits {
            let clip_id = match edit {
                Edit::Create { clip, .. }
                | Edit::Update { clip, .. }
                | Edit::Delete { clip, .. } => clip,
            };
            let clip = expected
                .clips
                .iter_mut()
                .find(|c| &c.id == clip_id)
                .ok_or("Missing result owner")?;
            match edit {
                Edit::Create {
                    temporary, note, ..
                } => {
                    let id = identities
                        .get(temporary)
                        .ok_or("Missing creation identity mapping")?
                        .clone();
                    if id.is_empty() || clip.notes.iter().any(|n| n.note.id == id) {
                        return Err("Invalid created identity".into());
                    }
                    let mut note = note.clone();
                    note.id.clone_from(&id);
                    note.start = native_time(note.start)?;
                    note.duration = native_time(note.duration)?;
                    note.content_start = native_time(note.content_start)?;
                    for point in note.expressions.values_mut().flatten() {
                        point.time = native_time(point.time)?;
                    }
                    expression_edits.insert((clip_id.clone(), id));
                    clip.notes.push(SelectedNote {
                        note,
                        selected: expected.selection.kind != "notes",
                    });
                }
                Edit::Update { note, changes, .. } => {
                    let target = clip
                        .notes
                        .iter_mut()
                        .find(|n| &n.note.id == note)
                        .ok_or("Missing updated identity")?;
                    let mut normalized: MidiNote = target.note.clone().try_into()?;
                    for change in changes {
                        normalized.change(change)?;
                        match change {
                            Change::Start(_) => {
                                let value = native_time(normalized.detail().start)?;
                                normalized.change(&Change::Start(value))?;
                            }
                            Change::Duration(_) => {
                                let value = native_time(normalized.detail().duration)?;
                                normalized.change(&Change::Duration(value))?;
                            }
                            Change::Expressions(_) => {
                                expression_edits.insert((clip_id.clone(), note.clone()));
                            }
                            _ => {}
                        }
                    }
                    let mut desired = normalized.detail().clone();
                    if expression_edits.contains(&(clip_id.clone(), note.clone())) {
                        for (kind, points) in &mut desired.expressions {
                            for point in points {
                                let old =
                                    target.note.expressions.get(kind).and_then(|points| {
                                        points.iter().find(|p| p.id == point.id)
                                    });
                                if old.is_none_or(|old| old.time != point.time) {
                                    point.time = native_time(point.time)?;
                                }
                            }
                        }
                    }
                    target.note = desired;
                }
                Edit::Delete { note, .. } => {
                    clip.notes.retain(|n| &n.note.id != note);
                    expected
                        .selection
                        .objects
                        .retain(|id| id != &format!("{clip_id}:{note}"));
                }
            }
        }
        for clip in &mut expected.clips {
            for note in &mut clip.notes {
                if expression_edits.contains(&(clip.id.clone(), note.note.id.clone())) {
                    for points in note.note.expressions.values_mut() {
                        for point in points {
                            if let Some(id) = identities.get(&point.id) {
                                point.id = id.clone();
                            }
                        }
                    }
                }
            }
            clip.notes.sort_by(|a, b| a.note.id.cmp(&b.note.id));
        }
        for edit in &plan.automation {
            let source = &self
                .snapshot
                .automation
                .iter()
                .find(|c| c.track == edit.track && c.lane == edit.lane && c.clip == edit.clip)
                .ok_or("Missing automation owner")?
                .source;
            let mut points = edit.points.clone();
            if points.is_empty() {
                let lane = self
                    .snapshot
                    .automation
                    .iter()
                    .find(|l| &l.source == source)
                    .ok_or("Missing cleared lane")?;
                points.push(crate::editor::Point {
                    id: identities
                        .get(&format!("default:{source}"))
                        .ok_or("Missing native clear default identity")?
                        .clone(),
                    time: 0.0,
                    value: lane.target["fallbackValue"]
                        .as_f64()
                        .ok_or("Missing fallback value")?,
                    interpolation: crate::editor::Interpolation::Linear,
                    curvature: 0.0,
                    spread: 0.0,
                });
            }
            for p in &mut points {
                if let Some(id) = identities.get(&p.id) {
                    p.id.clone_from(id);
                }
            }
            {
                for lane in &mut expected.automation {
                    if &lane.source == source {
                        lane.points.clone_from(&points);
                    }
                }
            }
        }
        for deleted in &plan.automation_delete {
            {
                expected.automation.retain(|lane| {
                    !(lane.track == deleted.track
                        && lane.lane == deleted.lane
                        && lane.clip == deleted.clip)
                });
            }
        }
        let mut created_lanes = Vec::new();
        for created in &plan.automation_create {
            let actual = observed
                .automation
                .iter()
                .find(|lane| lane.placement == created.placement && created.matches_target(lane))
                .ok_or("Missing created MIDI CC lane")?;
            let mut points = created.points.clone();
            for point in &mut points {
                point.id = identities
                    .get(&point.id)
                    .ok_or("Missing created CC point identity")?
                    .clone();
            }
            if actual.points != points
                || (created.placement == "arranger" && actual.mapping != serde_json::json!({}))
                || actual.reference_count != 1
                || actual.aliases != [actual.clip.clone()]
                || actual.target["min"]
                    != if created.target["kind"] == "parameter" {
                        created.target["min"].clone()
                    } else {
                        serde_json::json!(0.0)
                    }
                || actual.target["max"]
                    != if created.target["kind"] == "parameter" {
                        created.target["max"]
                            .as_f64()
                            .ok_or("Missing parameter maximum")?
                    } else {
                        1.0
                    }
                || actual.target["domain"] != "LINEAR"
            {
                return Err("Created CC lane differs from requested MIDI data".into());
            }
            created_lanes.push(actual.clone());
        }
        for (index, lane) in observed.automation.iter().enumerate() {
            if created_lanes
                .iter()
                .any(|c| c.track == lane.track && c.lane == lane.lane && c.clip == lane.clip)
            {
                if index > expected.automation.len() {
                    return Err("Unexpected automation lane during creation".into());
                }
                expected.automation.insert(index, lane.clone());
            }
        }
        Ok(expected)
    }

    /// Verify the full observed document against the admitted absolute plan.
    /// Host tick rounding is permitted only for explicitly changed timing.
    ///
    /// # Errors
    /// Refuses any mismatch in requested changes, identity mappings or preserved state.
    pub fn verify_result(
        &self,
        plan: &Plan,
        observed: &Snapshot,
        identities: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        self.validate_plan(plan)?;
        let required = self.validate_new_identities(plan)?;
        let native: HashSet<_> = identities.values().collect();
        if required.len() != identities.len()
            || !required.iter().all(|id| identities.contains_key(id))
            || native.len() != identities.len()
            || native.iter().any(|id| id.is_empty())
        {
            return Err("Missing, duplicate or unexpected native identity mappings".into());
        }
        let mut expected = self.projected_snapshot(plan, identities, observed)?;
        let mut observed = observed.clone();
        for clip in &mut observed.clips {
            clip.notes.sort_by(|a, b| a.note.id.cmp(&b.note.id));
        }
        expected.revision.clone_from(&observed.revision);
        if expected != observed {
            return Err(
                "Observed document differs from admitted plan or preserved material".into(),
            );
        }
        Ok(())
    }

    /// Validate ownership, selection, fields and the complete desired collision graph.
    ///
    /// # Errors
    /// Refuses stale, ambiguous, invalid or colliding edits before host mutation.
    #[allow(clippy::too_many_lines, clippy::float_cmp)] // Exact preservation is the contract.
    pub fn validate_plan(&self, plan: &Plan) -> Result<(), String> {
        if plan.document != self.snapshot.document
            || plan.revision != self.snapshot.revision
            || (plan.edits.is_empty()
                && plan.automation.is_empty()
                && plan.automation_create.is_empty()
                && plan.automation_delete.is_empty())
        {
            return Err("Stale or empty plan".into());
        }
        self.validate_new_identities(plan)?;
        if !plan.automation.is_empty() {
            self.snapshot.preflight_automation(&plan.automation)?;
        }
        for created in &plan.automation_create {
            created.validate()?;
            if !created.is_new()
                || !self.snapshot.owners.contains(&created.owner)
                || !self.snapshot.owns_automation(created)
                || self
                    .snapshot
                    .automation
                    .iter()
                    .any(|lane| created.matches_target(lane))
            {
                return Err("New CC target is absent from the clip scope or already exists".into());
            }
        }
        for deleted in &plan.automation_delete {
            if !["clip", "launcher"].contains(&deleted.placement.as_str())
                || deleted.reference_count != 1
                || deleted.aliases.len() != 1
                || !self.snapshot.automation.iter().any(|c| c == deleted)
                || plan.automation.iter().any(|e| {
                    e.track == deleted.track && e.lane == deleted.lane && e.clip == deleted.clip
                })
            {
                return Err("Deletion requires an observed exclusive automation placement".into());
            }
        }
        // Model the complete desired result before any host writes. Original notes
        // remain in place until every operation and every destination is validated.
        let mut final_notes: Vec<_> = self
            .notes
            .iter()
            .map(|(c, n, _)| (c.clone(), n.detail().clone()))
            .collect();
        let mut touched = HashSet::new();
        for edit in &plan.edits {
            let (clip, id) = match edit {
                Edit::Create {
                    clip, temporary, ..
                } => (clip, temporary),
                Edit::Update { clip, note, .. } | Edit::Delete { clip, note } => (clip, note),
            };
            if !touched.insert((clip, id)) {
                return Err("Duplicate edit target".into());
            }
            let owner = self
                .snapshot
                .clips
                .iter()
                .find(|c| &c.id == clip)
                .ok_or("Unknown clip")?;
            match edit {
                Edit::Create {
                    temporary, note, ..
                } => {
                    if temporary.is_empty()
                        || note.id != *temporary
                        || owner.notes.iter().any(|n| n.note.id == *temporary)
                    {
                        return Err("Invalid creation identity".into());
                    }
                    let normalized: MidiNote = note.clone().try_into()?;
                    final_notes.push((clip.clone(), normalized.detail().clone()));
                }
                Edit::Update { note, changes, .. } => {
                    let (_, original, selected) = self
                        .notes
                        .iter()
                        .find(|(c, n, _)| c == clip && n.detail().id == *note)
                        .ok_or("Unknown note")?;
                    if !selected || changes.is_empty() {
                        return Err("Edit outside UI selection".into());
                    }
                    let mut normalized: MidiNote = original.detail().clone().try_into()?;
                    let mut fields = HashSet::new();
                    for change in changes {
                        if !fields.insert(std::mem::discriminant(change)) {
                            return Err("Repeated update field".into());
                        }
                        normalized.change(change)?;
                    }
                    let target = final_notes
                        .iter_mut()
                        .find(|(c, n)| c == clip && n.id == *note)
                        .ok_or("Missing update target")?;
                    target.1 = normalized.detail().clone();
                }
                Edit::Delete { note, .. } => {
                    if !self
                        .notes
                        .iter()
                        .any(|(c, n, s)| c == clip && n.detail().id == *note && *s)
                    {
                        return Err("Deletion outside UI selection".into());
                    }
                    final_notes.retain(|(c, n)| c != clip || n.id != *note);
                }
            }
        }
        // Native overlap behavior may trim/delete notes. Admit only plans whose
        // resulting same-row intervals coexist; do not silently overwrite others.
        final_notes.sort_by(|(ca, a), (cb, b)| {
            (ca, a.channel, a.key)
                .cmp(&(cb, b.channel, b.key))
                .then(a.start.total_cmp(&b.start))
        });
        for pair in final_notes.windows(2) {
            let (ca, a) = &pair[0];
            let (cb, b) = &pair[1];
            if ca == cb
                && a.channel == b.channel
                && a.key == b.key
                && a.start + a.duration > b.start
            {
                // Preserve pre-existing overlaps unless this plan changes either note's geometry.
                let changed = |c: &String, n: &NoteEvent| {
                    self.notes
                        .iter()
                        .find(|(o, v, _)| o == c && v.detail().id == n.id)
                        .is_none_or(|(_, v, _)| {
                            v.detail().channel != n.channel
                                || v.detail().key != n.key
                                || v.detail().start != n.start
                                || v.detail().duration != n.duration
                        })
                };
                if changed(ca, a) || changed(cb, b) {
                    return Err("Destination overlaps existing material".into());
                }
            }
        }
        Ok(())
    }
}

/// Bitwig 6.1 persistent musical coordinates (anB.v33), including negative ties.
pub(crate) fn native_time(value: f64) -> Result<f64, String> {
    let scaled = value * 1_048_576.0;
    let floor = scaled.floor();
    // Adding 0.5 before floor can round a just-below-half double upwards.
    let rounded = if scaled - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    // Java Math.round saturates its long result; converting Long.MAX_VALUE to
    // double yields the same upper endpoint as 2^63.
    let resolved = rounded.clamp(-9_223_372_036_854_775_808.0, 9_223_372_036_854_775_808.0)
        / 1_048_576.0
        + 0.0;
    if !resolved.is_finite() || (value - resolved).abs() > 0.5 / 1_048_576.0 {
        return Err("Requested time exceeds the native persistent coordinate range".into());
    }
    Ok(resolved)
}
