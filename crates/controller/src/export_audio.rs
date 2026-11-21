//! Native Export Audio options, explicit owners and asynchronous status.
use crate::{mix::Observation, project_address::Path};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub expected: Observation,
    pub tracks: Vec<Path>,
    pub directory: std::path::PathBuf,
    pub start: f64,
    pub end: f64,
    pub sample_rate: u32,
    pub bit_depth: String,
    pub dither: bool,
    pub pre_fader: bool,
    pub real_time: bool,
}
impl Options {
    /// # Errors
    /// Refuses unsupported options, invalid ranges and unobserved/duplicate targets.
    pub fn validate(&self) -> Result<(), String> {
        self.expected.validate()?;
        if !self.directory.is_absolute()
            || self.directory.to_str().is_none()
            || self.tracks.is_empty()
            || self.tracks.len() > 256
            || ![44_100, 48_000, 88_200, 96_000, 192_000].contains(&self.sample_rate)
            || !["INT16", "INT24", "FLOAT32"].contains(&self.bit_depth.as_str())
            || (self.bit_depth == "FLOAT32" && self.dither)
            || !self.start.is_finite()
            || !self.end.is_finite()
            || self.start < 0.0
            || self.end <= self.start
            || !crate::protocol::native_time(self.start)
                .is_ok_and(|v| v.to_bits() == self.start.to_bits())
            || !crate::protocol::native_time(self.end)
                .is_ok_and(|v| v.to_bits() == self.end.to_bits())
        {
            return Err("Invalid native WAV export options (explicit sample rate and native beats required)".into());
        }
        let mut paths = std::collections::BTreeSet::new();
        for path in &self.tracks {
            path.validate()?;
            if !self.expected.tracks.iter().any(|t| &t.path == path)
                || !paths.insert(serde_json::to_string(path).map_err(|e| e.to_string())?)
            {
                return Err("Export target absent or duplicated".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Start {
        operation: String,
        options: Box<Options>,
    },
    Status {
        operation: String,
    },
}
impl Request {
    /// # Errors
    /// Refuses malformed operation identities/options before native dispatch.
    pub fn validate(&self) -> Result<(), String> {
        let (Self::Start { operation, .. } | Self::Status { operation }) = self;
        if !crate::protocol::valid_revision(operation) {
            return Err("Invalid export operation identity".into());
        }
        if let Self::Start { options, .. } = self {
            options.validate()?;
        }
        Ok(())
    }

    /// # Errors
    /// Rejects mismatched native operation metadata and premature success claims.
    pub fn verify(&self, result: &serde_json::Value) -> Result<(), String> {
        let (Self::Start { operation, .. } | Self::Status { operation }) = self;
        if result["operation"].as_str() != Some(operation)
            || !matches!(
                result["status"].as_str(),
                Some("pending" | "completed" | "failed")
            )
            || result["document"].as_str().is_none_or(str::is_empty)
            || !result["nativeComplete"].is_boolean()
            || result["expectedSeconds"]
                .as_f64()
                .is_none_or(|v| !v.is_finite() || v <= 0.0 || v > 3600.0)
        {
            return Err("Invalid native export result identity/status".into());
        }
        let options = &result["options"];
        if let Self::Start {
            options: requested, ..
        } = self
        {
            let mut requested = serde_json::to_value(requested).map_err(|e| e.to_string())?;
            requested
                .as_object_mut()
                .ok_or("Invalid export options")?
                .remove("expected");
            if &requested != options {
                return Err("Native export options differ".into());
            }
        }
        let directory = options["directory"]
            .as_str()
            .map(std::path::Path::new)
            .ok_or("Missing export directory")?;
        let paths = options["tracks"]
            .as_array()
            .ok_or("Missing export owners")?;
        let files = result["files"].as_array().ok_or("Missing export files")?;
        if files.is_empty() || files.len() != paths.len() || files.len() > 256 {
            return Err("Export files/owners mismatch".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for (file, path) in files.iter().zip(paths) {
            let output = file["path"]
                .as_str()
                .map(std::path::Path::new)
                .ok_or("Missing WAV destination")?;
            if &file["track"] != path
                || output.parent() != Some(directory)
                || output.extension().is_none_or(|v| v != "wav")
                || !seen.insert(output)
            {
                return Err("Native export file target differs".into());
            }
        }
        if result["status"] == "completed"
            && (result["nativeComplete"] != true
                || result["nativeSuccess"] != true
                || files
                    .iter()
                    .any(|f| f["exists"] != true || f["bytes"].as_u64().is_none_or(|v| v <= 44)))
        {
            return Err("Export completed without native success and complete outputs".into());
        }
        Ok(())
    }
}
