//! Bitwig's project hierarchy as typed folders with explicit metadata.
use crate::project_address::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub path: Path,
    pub kind: String,
    pub metadata: Value,
    pub children: Vec<Entity>,
}
impl Entity {
    /// # Errors
    /// Refuses malformed hierarchy records rather than accepting arbitrary native graphs.
    pub fn validate(&self) -> Result<(), String> {
        self.path.validate()?;
        if ![
            "project",
            "group",
            "track",
            "clip",
            "launcherClip",
            "automationLane",
            "layer",
            "layerChain",
            "selector",
            "choice",
            "plugin",
            "device",
        ]
        .contains(&self.kind.as_str())
            || !self.metadata.is_object()
        {
            return Err("Invalid native entity".into());
        }
        let fields = [
            "active",
            "index",
            "routingEnabled",
            "routingActive",
            "channelFilter",
            "channelMap",
            "name",
            "id",
            "start",
            "duration",
            "sourceStart",
            "sourceEnd",
            "loopStart",
            "loopLength",
            "loopEnabled",
            "pluginId",
            "version",
            "channel",
            "outputChannel",
            "file",
            "muted",
            "type",
            "tempo",
            "meter",
            "cues",
            "parameters",
            "remoteControls",
        ];
        if self
            .metadata
            .as_object()
            .ok_or("Missing metadata")?
            .keys()
            .any(|key| !fields.contains(&key.as_str()))
        {
            return Err("Unknown entity metadata".into());
        }
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}
