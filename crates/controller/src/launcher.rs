//! Selected launcher slot authority and bounded empty Note Clip creation.
use crate::project_address::{Element, Path};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Position {
    pub track: Path,
    pub scene: u32,
    pub slot: String,
    pub clip: Option<Path>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub document: String,
    pub revision: String,
    pub position: Option<Position>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Read {},
    Create { expected: Observation, beats: u32 },
}

impl Position {
    fn validate(&self) -> Result<(), String> {
        self.track.validate()?;
        if !matches!(self.track.elements.last(), Some(Element::Track { .. }))
            || self.slot.is_empty()
            || self.slot.len() > 128
        {
            return Err("Expected a native launcher note slot".into());
        }
        if let Some(clip) = &self.clip {
            clip.validate()?;
            if clip.track_path()? != self.track
                || !matches!(clip.elements.last(), Some(Element::LauncherClip { scene, native_id: Some(_), .. }) if *scene == self.scene)
            {
                return Err("Launcher clip does not belong to the observed position".into());
            }
        }
        Ok(())
    }
}

impl Observation {
    fn validate(&self) -> Result<(), String> {
        if self.document.is_empty()
            || self.document.len() > 128
            || !crate::protocol::valid_revision(&self.revision)
        {
            return Err("Invalid launcher observation authority".into());
        }
        if let Some(position) = &self.position {
            position.validate()?;
        }
        Ok(())
    }
}

impl Request {
    /// # Errors
    /// Refuses creation without fresh authority for one empty note slot.
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Create { expected, beats } = self {
            expected.validate()?;
            if !(1..=1024).contains(beats)
                || expected.position.as_ref().is_none_or(|p| p.clip.is_some())
            {
                return Err(
                    "Creation requires an empty selected launcher slot and 1–1024 beats".into(),
                );
            }
        }
        Ok(())
    }

    /// # Errors
    /// Refuses invalid observation data or a creation result outside the requested slot.
    pub fn verify(&self, actual: &Observation) -> Result<(), String> {
        actual.validate()?;
        if let Self::Create { expected, .. } = self {
            let mut observed_slot = actual.position.clone().ok_or("Missing created slot")?;
            if observed_slot.clip.take().is_none()
                || expected.position.as_ref() != Some(&observed_slot)
                || expected.document != actual.document
            {
                return Err("Created clip differs from the requested launcher slot".into());
            }
        }
        Ok(())
    }
}
