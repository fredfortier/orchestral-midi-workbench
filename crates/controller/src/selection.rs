//! Absolute native selection, separate from musical content and keyboard focus.
use crate::project_address::Path;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Read {},
    Set {
        document: String,
        revision: String,
        paths: Vec<Path>,
        mode: Mode,
        range: Option<Range>,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Objects,
    Time,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    pub from: f64,
    pub to: f64,
}
impl Request {
    /// # Errors
    /// Rejects malformed authority, duplicate paths and incompatible range/mode inputs.
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Set {
            document,
            revision,
            paths,
            mode,
            range,
        } = self
        {
            if document.is_empty()
                || document.len() > 128
                || !crate::protocol::valid_revision(revision)
                || paths.len() > 1024
            {
                return Err("Fresh selection authority and at most 1024 paths required".into());
            }
            let mut unique = std::collections::BTreeSet::new();
            for path in paths {
                path.validate()?;
                if path.elements.is_empty() || !unique.insert(path) {
                    return Err("Distinct native selection targets required".into());
                }
            }
            match (mode, range) {
                (Mode::Objects, None) => {},
                (Mode::Time, Some(r)) if !paths.is_empty() && r.from.is_finite() && r.to.is_finite() && r.from >= 0.0 && r.to > r.from && crate::protocol::native_time(r.from)? < crate::protocol::native_time(r.to)? => {},
                _ => return Err("Time mode requires lanes and increasing arranger-beat bounds; object mode has no range".into()),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Observation, Request};
    use serde_json::json;

    #[test]
    fn selection_point_uses_the_same_qualified_lane_address() {
        let member = json!({
            "project": "Piece",
            "elements": [
                {"kind": "track", "name": "Cello", "nativeId": "10"},
                {"kind": "automationLane", "name": "Modulation", "nativeId": "20"}
            ],
            "item": {"kind": "automationPoint", "id": "30", "source": "40", "placement": "40"}
        });
        let input = json!({
            "document": "document",
            "revision": "0e240cca-45e8-44b6-8b35-d67b0e6d0fcf",
            "editor": "arranger",
            "members": [member],
            "primary": member,
            "mode": "objects",
            "range": null,
            "unsupportedMembers": []
        });
        let observation: Observation = serde_json::from_value(input.clone()).unwrap();
        Request::Read {}.verify(&observation).unwrap();
        assert_eq!(serde_json::to_value(observation).unwrap(), input);

        let mut invalid = input;
        invalid["members"][0]["elements"][1]["kind"] = json!("clip");
        let observation = serde_json::from_value(invalid).unwrap();
        assert!(Request::Read {}.verify(&observation).is_err());
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub document: String,
    pub revision: String,
    pub editor: String,
    pub members: Vec<Member>,
    pub primary: Option<Member>,
    pub mode: Mode,
    pub range: Option<Range>,
    pub unsupported_members: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_primary: Option<String>,
}

/// A structural address with an optional reference to content at that address.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(flatten)]
    pub path: Path,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Item {
    Note {
        id: String,
    },
    AutomationPoint {
        id: String,
        source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placement: Option<String>,
    },
}

impl Member {
    fn validate(&self) -> Result<(), String> {
        use crate::project_address::Element;
        self.path.validate()?;
        let valid_id = |id: &str| crate::project_address::name(id) && id.len() <= 128;
        match (self.path.elements.last(), &self.item) {
            (_, None) => Ok(()),
            (
                Some(Element::Clip { .. } | Element::LauncherClip { .. }),
                Some(Item::Note { id }),
            ) if valid_id(id) => Ok(()),
            (
                Some(Element::AutomationLane { .. }),
                Some(Item::AutomationPoint {
                    id,
                    source,
                    placement,
                }),
            ) if valid_id(id) && valid_id(source) && placement.as_deref().is_none_or(valid_id) => {
                Ok(())
            }
            _ => Err("Selected content has an invalid identity or owner path".into()),
        }
    }
}
impl Request {
    /// Verify independently returned native selection, including primary and normalized bounds.
    /// # Errors
    /// Refuses malformed observations or selection that differs from the absolute request.
    pub fn verify(&self, observation: &Observation) -> Result<(), String> {
        if observation.document.is_empty()
            || !crate::protocol::valid_revision(&observation.revision)
            || !["none", "arranger", "detailEditor", "unsupported"]
                .contains(&observation.editor.as_str())
        {
            return Err("Invalid native selection authority/editor".into());
        }
        for member in observation.members.iter().chain(observation.primary.iter()) {
            member.validate()?;
        }
        if let Self::Set {
            document,
            paths,
            mode,
            range,
            ..
        } = self
        {
            if document != &observation.document
                || observation.members.len() != paths.len()
                || !observation.unsupported_members.is_empty()
                || paths.iter().any(|path| {
                    !observation
                        .members
                        .iter()
                        .any(|actual| actual.item.is_none() && path.matches(&actual.path))
                })
                || paths.first().is_some_and(|path| {
                    observation
                        .primary
                        .as_ref()
                        .is_none_or(|actual| actual.item.is_some() || !path.matches(&actual.path))
                })
                || (!paths.is_empty() && *mode != observation.mode)
            {
                return Err("Native selection differs from absolute request; reread before changing selection".into());
            }
            if let Some(requested) = range {
                let actual = observation.range.ok_or("Missing native selection range")?;
                if crate::protocol::native_time(requested.from)?.to_bits() != actual.from.to_bits()
                    || crate::protocol::native_time(requested.to)?.to_bits() != actual.to.to_bits()
                {
                    return Err("Native selection bounds differ".into());
                }
            }
        }
        Ok(())
    }
}
