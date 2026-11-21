//! Complete selected arranger clips: notes, expression and all owning-track automation.
use crate::protocol;
use std::error::Error;
pub type Observation = protocol::Snapshot;
impl Observation {
    /// # Errors
    /// Refuses invalid content, clip paths or duplicate path ownership.
    pub fn validate(&self) -> Result<(), String> {
        let _: protocol::Document = self.clone().try_into()?;
        let mut paths = std::collections::BTreeSet::new();
        for clip in &self.clips {
            clip.path.validate()?;
            if !matches!(
                clip.path.elements.last(),
                Some(crate::project_address::Element::Clip { .. })
            ) || !paths.insert(&clip.path)
            {
                return Err("Clip paths must be unique and end in a native Note Clip".into());
            }
        }
        let mut owners = std::collections::BTreeSet::new();
        for owner in &self.owners {
            owner.validate()?;
            if !owners.insert(owner) {
                return Err("Duplicate content owner".into());
            }
        }
        let mut placements = std::collections::BTreeSet::new();
        let mut pending = std::collections::BTreeSet::new();
        for lane in &self.automation {
            lane.validate()?;
            if lane.is_new()
                && !pending.insert(
                    serde_json::to_string(&(&lane.owner, &lane.target))
                        .map_err(|e| e.to_string())?,
                )
            {
                return Err("Duplicate pending automation target".into());
            }
            if !owners.contains(&lane.owner)
                || (!lane.is_new() && !placements.insert((&lane.track, &lane.lane, &lane.clip)))
            {
                return Err("Unknown owner or duplicate automation placement".into());
            }
        }
        for clip in &self.clips {
            let mut owner = clip.path.clone();
            owner.elements.pop();
            if !owners.contains(&owner) {
                return Err("Unknown clip owner".into());
            }
        }
        Ok(())
    }
}
/// # Errors
/// Returns connection, selection, ownership and schema failures.
pub async fn read(port: u16, adapter: Option<String>) -> Result<Observation, Box<dyn Error>> {
    match crate::api::exchange(port, adapter, crate::controller::Command::Midi).await? {
        crate::control::Response::MidiResult { midi } => {
            midi.validate()?;
            Ok(*midi)
        }
        _ => Err("Unexpected clip content response".into()),
    }
}

/// Compile a modified complete clip payload to one native notes-and-automation transaction.
/// # Errors
/// Refuses changed clip identity/geometry, partial note scope or unsupported note attributes.
pub fn replacement(before: &Observation, after: &Observation) -> Result<protocol::Plan, String> {
    before.validate()?;
    after.validate()?;
    if before.document != after.document
        || before.revision != after.revision
        || before.selection != after.selection
        || before.clips.len() != after.clips.len()
        || before.owners != after.owners
        || before.paths != after.paths
    {
        return Err("Replacement must retain the original clip set and authority".into());
    }
    validate_shared_points(after)?;
    let mut changed_sources = std::collections::BTreeSet::new();
    let mut plan = protocol::Plan {
        automation_create: Vec::new(),
        automation_delete: Vec::new(),
        document: before.document.clone(),
        revision: before.revision.clone(),
        edits: Vec::new(),
        automation: Vec::new(),
    };
    for old in &before.clips {
        let desired = after
            .clips
            .iter()
            .find(|c| c.path == old.path)
            .ok_or("Missing clip path")?;
        let mut metadata = old.clone();
        metadata.notes.clone_from(&desired.notes);

        if &metadata != desired {
            return Err("Content replacement preserves clip metadata and native ownership".into());
        }
        compile_notes(old, desired, &mut plan)?;
    }
    compile_new_lanes(before, after, &mut plan)?;
    {
        for lane in &before.automation {
            let target = after
                .automation
                .iter()
                .find(|l| l.track == lane.track && l.lane == lane.lane && l.clip == lane.clip);
            let Some(target) = target else {
                if changed_sources.insert(&lane.source) {
                    if lane.placement == "arranger" {
                        return Err("Outside-clip automation lanes cannot be deleted; retain the lane and clear its points explicitly".into());
                    }
                    plan.automation_delete.push(lane.clone());
                }
                continue;
            };
            let mut expected = lane.clone();
            expected.points.clone_from(&target.points);
            if &expected != target {
                return Err(
                    "Replacement preserves automation ownership, target and placement".into(),
                );
            }
            if lane.points != target.points && changed_sources.insert(&lane.source) {
                let edit = crate::automation::Replacement {
                    track: lane.track.clone(),
                    lane: lane.lane.clone(),
                    clip: lane.clip.clone(),
                    aliases: lane.aliases.clone(),
                    points: target.points.clone(),
                };
                plan.automation.push(edit);
            }
        }
    }

    let doc: protocol::Document = before.clone().try_into()?;
    doc.validate_plan(&plan)?;
    Ok(plan)
}

fn compile_notes(
    old: &protocol::Clip,
    desired: &protocol::Clip,
    plan: &mut protocol::Plan,
) -> Result<(), String> {
    for note in &old.notes {
        let found = desired.notes.iter().find(|n| n.note.id == note.note.id);
        if found.is_none() {
            plan.edits.push(protocol::Edit::Delete {
                clip: old.id.clone(),
                note: note.note.id.clone(),
            });
            continue;
        }
        let target = found.ok_or("Missing note")?;
        if target.selected != note.selected {
            return Err("Replacement cannot alter selection flags".into());
        }
        let left = serde_json::to_value(&note.note).map_err(|e| e.to_string())?;
        let right = serde_json::to_value(&target.note).map_err(|e| e.to_string())?;
        let mut changes = Vec::new();
        for (field, value) in right.as_object().ok_or("Invalid note")? {
            if left[field] != *value {
                changes.push(
                    serde_json::from_value(serde_json::json!({"field":field,"value":value}))
                        .map_err(|_| format!("Unsupported changed note attribute: {field}"))?,
                );
            }
        }
        if !changes.is_empty() {
            plan.edits.push(protocol::Edit::Update {
                clip: old.id.clone(),
                note: note.note.id.clone(),
                changes,
            });
        }
    }
    for note in &desired.notes {
        if !old.notes.iter().any(|n| n.note.id == note.note.id) {
            plan.edits.push(protocol::Edit::Create {
                clip: old.id.clone(),
                temporary: note.note.id.clone(),
                note: note.note.clone(),
            });
        }
    }
    Ok(())
}

fn validate_shared_points(after: &Observation) -> Result<(), String> {
    let mut desired_sources = std::collections::BTreeMap::new();
    for lane in after.automation.iter().filter(|lane| !lane.is_new()) {
        if desired_sources
            .insert(&lane.source, &lane.points)
            .is_some_and(|points| points != &lane.points)
        {
            return Err(
                "Repeated/shared automation sources have conflicting desired points".into(),
            );
        }
    }
    Ok(())
}

fn compile_new_lanes(
    old: &Observation,
    desired: &Observation,
    plan: &mut protocol::Plan,
) -> Result<(), String> {
    if desired
        .automation
        .iter()
        .filter(|lane| !lane.is_new())
        .any(|lane| {
            !old.automation.iter().any(|original| {
                original.track == lane.track
                    && original.lane == lane.lane
                    && original.clip == lane.clip
            })
        })
    {
        return Err("Unknown automation placement in replacement".into());
    }
    for lane in desired.automation.iter().filter(|lane| lane.is_new()) {
        if !old.owners.contains(&lane.owner) {
            return Err("New CC owner must be the clip's native parent".into());
        }
        plan.automation_create.push(lane.clone());
    }
    Ok(())
}
