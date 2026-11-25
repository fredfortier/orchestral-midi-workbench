//! Complete native note clips, expressions and stored automation.
use crate::project_address::{Element, Path};
use crate::protocol;
use std::collections::BTreeSet;
pub type Observation = protocol::Snapshot;
impl Observation {
    /// Check the resolved scope as well as the echoed locators.
    pub(crate) fn verify_scope(&self, paths: Option<&[Path]>) -> Result<(), String> {
        if self.paths.as_deref() != paths {
            return Err("Native content scope differs from the request".into());
        }
        let Some(paths) = paths else {
            return Ok(());
        };
        let mut owners = BTreeSet::new();
        let mut arranger_owners = BTreeSet::new();
        let mut track_scopes = BTreeSet::new();
        let mut clip_scopes = BTreeSet::new();
        for path in paths {
            let locator = path.track_path()?;
            let mut matches = self.owners.iter().filter(|owner| locator.matches(owner));
            let (Some(owner), None) = (matches.next(), matches.next()) else {
                return Err("Requested content owner is absent or ambiguous".into());
            };
            owners.insert(owner);
            if !matches!(path.elements.last(), Some(Element::LauncherClip { .. })) {
                arranger_owners.insert(owner);
            }
            if matches!(
                path.elements.last(),
                Some(Element::Clip { .. } | Element::LauncherClip { .. })
            ) {
                let mut matches = self.clips.iter().filter(|clip| path.matches(&clip.path));
                let (Some(clip), None) = (matches.next(), matches.next()) else {
                    return Err("Requested Note Clip is absent or ambiguous".into());
                };
                if !clip_scopes.insert(&clip.path) {
                    return Err("Multiple locators resolve to the same Note Clip".into());
                }
            } else if !track_scopes.insert(owner) {
                return Err("Multiple locators resolve to the same content track".into());
            }
        }
        if owners.len() != self.owners.len() {
            return Err("Native read returned an unrequested content owner".into());
        }
        for clip in &self.clips {
            let covered_by_track = matches!(clip.path.elements.last(), Some(Element::Clip { .. }))
                && track_scopes.contains(&clip.path.track_path()?);
            if covered_by_track == clip_scopes.contains(&clip.path) {
                return Err(
                    "Native Note Clip is outside the requested scope or covered twice".into(),
                );
            }
        }
        if self
            .automation
            .iter()
            .any(|lane| lane.placement != "launcher" && !arranger_owners.contains(&lane.owner))
        {
            return Err(
                "Native read returned Arranger automation outside the requested scope".into(),
            );
        }
        Ok(())
    }

    pub(crate) fn owns_automation(&self, lane: &crate::automation::Curve) -> bool {
        lane.note_clip.as_ref().is_none_or(|id| {
            self.clips.iter().any(|clip| {
                &clip.id == id
                    && matches!(
                        clip.path.elements.last(),
                        Some(crate::project_address::Element::LauncherClip { .. })
                    )
                    && clip
                        .path
                        .track_path()
                        .is_ok_and(|track| track == lane.owner)
            })
        })
    }

    /// # Errors
    /// Refuses invalid content, clip paths or duplicate path ownership.
    pub fn validate(&self) -> Result<(), String> {
        let _: protocol::Document = self.clone().try_into()?;
        let mut paths = std::collections::BTreeSet::new();
        for clip in &self.clips {
            clip.path.validate()?;
            if !matches!(
                clip.path.elements.last(),
                Some(
                    crate::project_address::Element::Clip { .. }
                        | crate::project_address::Element::LauncherClip { .. }
                )
            ) || !paths.insert(&clip.path)
            {
                return Err("Clip paths must be unique and end in a native Note Clip".into());
            }
            if clip
                .path
                .elements
                .last()
                .and_then(Element::native_id)
                .is_some_and(|id| id != clip.id)
            {
                return Err("Clip identity differs from its qualified path".into());
            }
        }
        let mut owners = std::collections::BTreeSet::new();
        for owner in &self.owners {
            owner.validate()?;
            if !matches!(
                owner.elements.last(),
                Some(Element::Track { .. } | Element::Group { .. })
            ) {
                return Err("Content owner must be a track or group channel".into());
            }
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
                    serde_json::to_string(&(&lane.owner, &lane.note_clip, &lane.target))
                        .map_err(|e| e.to_string())?,
                )
            {
                return Err("Duplicate pending automation target".into());
            }
            if !self.owns_automation(lane)
                || !owners.contains(&lane.owner)
                || (!lane.is_new() && track_id(&lane.owner).is_some_and(|id| id != lane.track))
                || (!lane.is_new() && !placements.insert((&lane.track, &lane.lane, &lane.clip)))
            {
                return Err("Unknown owner or duplicate automation placement".into());
            }
        }
        for clip in &self.clips {
            let mut owner = clip.path.clone();
            owner.elements.pop();
            if !owners.contains(&owner) || track_id(&owner).is_some_and(|id| id != clip.track) {
                return Err("Unknown clip owner".into());
            }
        }
        Ok(())
    }
}

fn track_id(path: &Path) -> Option<&str> {
    match path.elements.last() {
        Some(Element::Track { native_id, .. }) => native_id.as_deref(),
        _ => None,
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

#[cfg(test)]
mod scope_tests {
    use super::Observation;
    use serde_json::json;

    #[test]
    fn correct_path_echo_does_not_admit_wrong_owners_or_missing_clips() {
        let owner = json!({"project":"Piece","elements":[
            {"kind":"track","name":"Harp","nativeId":"1"}
        ]});
        let clip = json!({"project":"Piece","elements":[
            {"kind":"track","name":"Harp","nativeId":"1"},
            {"kind":"clip","name":"Harp part","nativeId":"2"}
        ]});
        let missing: Observation = serde_json::from_value(json!({
            "document":"piece","revision":"observed",
            "selection":{"editor":"paths","kind":"clips","from":0,"to":0,"lanes":[],"objects":[]},
            "clips":[],"automation":[],"owners":[owner],"paths":[clip]
        }))
        .unwrap();
        assert!(missing.validate().is_ok());
        assert_eq!(
            missing.verify_scope(missing.paths.as_deref()).unwrap_err(),
            "Requested Note Clip is absent or ambiguous"
        );

        let mut wrong_owner = missing;
        wrong_owner.paths = Some(wrong_owner.owners.clone());
        wrong_owner.owners[0].elements[0] = super::Element::Track {
            name: "Cello".into(),
            native_id: Some("3".into()),
        };
        assert!(wrong_owner.validate().is_ok());
        assert!(
            wrong_owner
                .verify_scope(wrong_owner.paths.as_deref())
                .is_err()
        );
    }
}
