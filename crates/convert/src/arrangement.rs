//! Compact musical identities embedded in standard Unknown Metadata Text.
use crate::{
    Diagnostic, Result,
    model::{Performance, Route},
    remarks,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const PREFIX: &str = "urn:midi2workbench:arrangement:v1:";
pub const COMMAND: &str = "urn:midi2workbench:command:v1:";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub name: String,
    pub routes: Vec<Route>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arrangement {
    pub version: u8,
    pub parts: Vec<Part>,
    pub decisions: Vec<String>,
}
impl Arrangement {
    /// # Errors
    /// Rejects duplicate route owners, invalid routes and empty musical identities.
    pub fn validate(&self) -> Result<()> {
        let mut routes = BTreeSet::new();
        if self.version != 1 || self.parts.is_empty() || self.parts.len() > 256 {
            return Err(Diagnostic::new(
                "E_ARRANGEMENT",
                "Unsupported arrangement version or part count",
            ));
        }
        for part in &self.parts {
            if part.name.trim().is_empty()
                || part.routes.is_empty()
                || part
                    .routes
                    .iter()
                    .any(|r| r.group > 15 || r.channel > 15 || !routes.insert(*r))
            {
                return Err(Diagnostic::new(
                    "E_ARRANGEMENT",
                    "Missing name or ambiguous musical route ownership",
                ));
            }
        }
        Ok(())
    }
    /// # Errors
    /// Returns invalid musical metadata or serialization errors.
    pub fn words(&self) -> Result<Vec<u32>> {
        self.validate()?;
        Ok(remarks::text(
            None,
            &format!(
                "{PREFIX}{}",
                serde_json::to_string(self)
                    .map_err(|e| Diagnostic::new("E_ARRANGEMENT", e.to_string()))?
            ),
            true,
        ))
    }
}
/// # Errors
/// Rejects contradictory/unknown arrangement declarations rather than guessing owners.
pub fn read(score: &Performance) -> Result<Option<Arrangement>> {
    let mut found = None;
    for e in &score.configuration {
        if let Some(text) = remarks::metadata(&e.words) {
            if let Some(payload) = text.strip_prefix(PREFIX) {
                if found.is_some() {
                    return Err(Diagnostic::new(
                        "E_ARRANGEMENT",
                        "Multiple arrangement declarations",
                    ));
                }
                let a: Arrangement = serde_json::from_str(payload)
                    .map_err(|e| Diagnostic::new("E_ARRANGEMENT", e.to_string()))?;
                a.validate()?;
                found = Some(a);
            }
        }
    }
    Ok(found)
}
