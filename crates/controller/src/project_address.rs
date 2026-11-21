//! Qualified musical addresses. Native identities are resolved separately for each observation.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Path {
    pub project: String,
    pub elements: Vec<Element>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Element {
    Device {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    AutomationLane {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Clip {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Group {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Track {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Layer {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    LayerChain {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Selector {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Choice {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
    },
    Plugin {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "nativeId")]
        native_id: Option<String>,
        id: String,
        version: String,
    },
}
impl Element {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Device { name, .. }
            | Self::AutomationLane { name, .. }
            | Self::Clip { name, .. }
            | Self::Group { name, .. }
            | Self::Track { name, .. }
            | Self::Layer { name, .. }
            | Self::LayerChain { name, .. }
            | Self::Selector { name, .. }
            | Self::Choice { name, .. }
            | Self::Plugin { name, .. } => name,
        }
    }
    fn native_id(&self) -> Option<&str> {
        match self {
            Self::Device { native_id, .. }
            | Self::AutomationLane { native_id, .. }
            | Self::Clip { native_id, .. }
            | Self::Group { native_id, .. }
            | Self::Track { native_id, .. }
            | Self::Layer { native_id, .. }
            | Self::LayerChain { native_id, .. }
            | Self::Selector { native_id, .. }
            | Self::Choice { native_id, .. }
            | Self::Plugin { native_id, .. } => native_id.as_deref(),
        }
    }
    fn clear_native_id(&mut self) {
        match self {
            Self::Device { native_id, .. }
            | Self::AutomationLane { native_id, .. }
            | Self::Clip { native_id, .. }
            | Self::Group { native_id, .. }
            | Self::Track { native_id, .. }
            | Self::Layer { native_id, .. }
            | Self::LayerChain { native_id, .. }
            | Self::Selector { native_id, .. }
            | Self::Choice { native_id, .. }
            | Self::Plugin { native_id, .. } => *native_id = None,
        }
    }
    pub(crate) fn chain(&self) -> bool {
        matches!(self, Self::LayerChain { .. } | Self::Choice { .. })
    }
}
pub(crate) fn name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.contains('\0')
}
impl Path {
    /// # Errors
    /// Rejects malformed names, excessive depth and paths that violate the musical hierarchy.
    pub fn validate(&self) -> Result<(), String> {
        if !name(&self.project) || self.project.len() > 128 || self.elements.len() > 64 {
            return Err("E_ADDRESS: expected a named project and 1–64 qualified elements".into());
        }
        let mut track = false;
        let mut previous: Option<&Element> = None;
        for element in &self.elements {
            if element
                .native_id()
                .is_some_and(|id| !name(id) || id.len() > 128)
            {
                return Err("E_ADDRESS: invalid native identity discriminator".into());
            }
            if !name(element.name()) {
                return Err(
                    "E_ADDRESS: names must be nonempty, NUL-free and at most 256 UTF-8 bytes"
                        .into(),
                );
            }
            let valid = match element {
                Element::Group { .. } | Element::Track { .. } => {
                    !track && previous.is_none_or(|p| matches!(p, Element::Group { .. }))
                }
                Element::LayerChain { .. } => matches!(previous, Some(Element::Layer { .. })),
                Element::Choice { .. } => matches!(previous, Some(Element::Selector { .. })),
                Element::Clip { .. } | Element::AutomationLane { .. } => matches!(
                    previous,
                    Some(Element::Track { .. } | Element::Group { .. })
                ),
                _ => {
                    (track || previous.is_some_and(|p| matches!(p, Element::Group { .. })))
                        && previous.is_some_and(|e: &Element| {
                            matches!(e, Element::Track { .. } | Element::Group { .. }) || e.chain()
                        })
                }
            };
            if !valid {
                return Err("E_ADDRESS: expected groups → track → device/chain hierarchy".into());
            }
            if let Element::Plugin { id, version, .. } = element {
                if id.is_empty()
                    || id.len() > 256
                    || version.is_empty()
                    || version.len() > 128
                    || id.contains('\0')
                    || version.contains('\0')
                {
                    return Err("E_ADDRESS: exact plugin registry ID/version required".into());
                }
            }
            track |= matches!(element, Element::Track { .. });
            previous = Some(element);
        }
        Ok(())
    }
    /// Whether a discovered path satisfies this locator, including supplied identities.
    #[must_use]
    pub fn matches(&self, discovered: &Self) -> bool {
        self.project == discovered.project
            && self.elements.len() == discovered.elements.len()
            && self
                .elements
                .iter()
                .zip(&discovered.elements)
                .all(|(wanted, actual)| {
                    let mut actual = actual.clone();
                    if wanted.native_id().is_none() {
                        actual.clear_native_id();
                    }
                    wanted == &actual
                })
    }
    /// Return the qualified owning track, retaining the authored group hierarchy.
    /// # Errors
    /// Refuses a malformed path or absent track.
    pub fn track_path(&self) -> Result<Self, String> {
        self.validate()?;
        let end = self
            .elements
            .iter()
            .take_while(|e| matches!(e, Element::Track { .. } | Element::Group { .. }))
            .count();
        if end == 0 {
            return Err("E_ADDRESS: missing qualified track/group owner".into());
        }
        Ok(Self {
            project: self.project.clone(),
            elements: self.elements[..end].to_vec(),
        })
    }
}
