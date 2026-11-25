//! Source preservation checked against explicit performed values and original notation.
use crate::{
    extension::Dictionary,
    model::{Performance, RenderingPolicy},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Companion {
    pub version: u32,
    pub mapping_version: u32,
    pub policy: RenderingPolicy,
    pub expected_midi_base64: String,
    pub dictionary: Dictionary,
    pub source_xml: String,
    pub requirements: serde_json::Value,
    /// Version-2 intended receiver assignments, distinct from execution evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendition: Option<crate::rendition::Rendition>,
    /// Optional ownership in canonical decoded note order; older companions omit it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub note_sources: Vec<crate::clip::NoteSource>,
}
impl Companion {
    fn expected_values(&self) -> crate::Result<String> {
        use base64::Engine;
        if self.expected_midi_base64.len() > 128 * 1024 * 1024 {
            return Err(crate::Diagnostic::new(
                "E_COMPANION",
                "Expected MIDI exceeds resource budget",
            ));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.expected_midi_base64)
            .map_err(|e| crate::Diagnostic::new("E_COMPANION", e.to_string()))?;
        Ok(crate::clip::decode(&bytes, Some(&self.dictionary))?.semantic_values())
    }

    #[must_use]
    /// # Panics
    /// Panics if `decoded` contains invalid note fields; pass a successfully decoded performance.
    pub fn new(
        _source: &[u8],
        xml: &str,
        clip: &[u8],
        decoded: &Performance,
        dictionary: Dictionary,
        policy: &RenderingPolicy,
    ) -> Self {
        let required = decoded
            .required_features(None)
            .expect("decoded valid note fields");
        let channels = decoded
            .notes
            .iter()
            .map(|n| n.route)
            .chain(decoded.piano_routes.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        let profiles = channels
            .into_iter()
            .map(|route| {
                let piano = decoded.piano_routes.contains(&route);
                let orchestral = !piano
                    || decoded
                        .notes
                        .iter()
                        .any(|n| n.route == route && n.needs_orchestral_profile());
                let mut ids = Vec::new();
                if piano {
                    ids.push(crate::piano::PROFILE_ID);
                }
                if orchestral {
                    ids.push(crate::rules::PROFILE_ID);
                }
                serde_json::json!({"route":route,"profileIds":ids})
            })
            .collect::<Vec<_>>();
        let requirements = serde_json::json!({"profiles":profiles,"pianoChannels":decoded.piano_routes,"optionalFeatures":required.bits,"standardCounts":required.standard.to_vec(),"customCounts":required.custom.to_vec(),"releaseSamples":decoded.notes.iter().map(|n|n.articulation.release).filter(|v|*v!=0).collect::<std::collections::BTreeSet<_>>()});
        Self {
            requirements,
            rendition: None,
            note_sources: Vec::new(),
            version: 1,
            mapping_version: 2,
            policy: policy.clone(),
            expected_midi_base64: {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(clip)
            },
            dictionary,
            source_xml: xml.into(),
        }
    }
    /// Bind intended assignments to this already encoded performance.
    /// # Errors
    /// Returns invalid/stale assignments, performance mismatch or serialization errors.
    pub fn set_rendition(
        &mut self,
        rendition: crate::rendition::Rendition,
        decoded: &Performance,
    ) -> crate::Result<()> {
        if decoded.semantic_values() != self.expected_values()? {
            return Err(crate::Diagnostic::new(
                "E_RENDITION",
                "assignment performance differs from companion",
            ));
        }
        rendition.validate_performance(decoded)?;
        self.rendition = Some(rendition);
        self.version = 2;
        Ok(())
    }
    /// Authenticate the versioned assignment extension against decoded MIDI.
    /// Version 1 remains a source-preservation envelope without receiver assignments.
    /// # Errors
    /// Returns unsupported/inconsistent versions, changed assignments and invalid bindings.
    pub fn validate_rendition(&self, decoded: &Performance) -> crate::Result<()> {
        if self.mapping_version != 2 {
            return Err(crate::Diagnostic::new(
                "E_COMPANION",
                "Unsupported companion mapping",
            ));
        }
        self.dictionary.validate()?;
        if self.expected_values()? != decoded.semantic_values() {
            return Err(crate::Diagnostic::new(
                "E_RENDITION",
                "performed values differ from companion",
            ));
        }
        match (self.version, &self.rendition) {
            (1, None) => Ok(()),
            (2, Some(rendition)) => rendition.validate_performance(decoded),
            _ => Err(crate::Diagnostic::new(
                "E_COMPANION_VERSION",
                "unsupported version or inconsistent rendition extension",
            )),
        }
    }
    #[must_use]
    pub(crate) fn source_matches(&self, decoded: &Performance) -> bool {
        // The manifest's own claims are not an oracle for the archived source.
        // Re-render it under the recorded policy before restoring written structure.
        let source_matches = (|| {
            let (_, source) = crate::xml::read(self.source_xml.as_bytes(), &self.policy)?;
            let (bytes, map, _) = crate::clip::encode(&source)?;
            let rendered = crate::clip::decode(&bytes, Some(&map))?;
            Ok::<_, crate::Diagnostic>(rendered.semantic_values() == decoded.semantic_values())
        })();
        source_matches.unwrap_or(false)
    }
}
