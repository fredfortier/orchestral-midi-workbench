//! Bidirectional score/performance conversion with explicit orchestral semantics.
pub mod adapt;
pub mod arrangement;
pub mod ci;
pub mod clip;
pub mod extension;
pub mod model;
pub mod notation;
pub mod piano;
pub mod preserve;
pub mod remarks;
pub mod rendition;
pub mod rules;
pub mod validation;
pub mod wire;
pub mod xml;
use thiserror::Error;
/// A stable error code and concrete witness.
#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}
impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
pub type Result<T> = std::result::Result<T, Diagnostic>;
/// Encoded clip and its preservation envelope.
pub struct Conversion {
    pub clip: Vec<u8>,
    pub companion: preserve::Companion,
    pub report: model::Report,
}
/// Normalize and encode using explicit interpretation policy.
/// # Errors
/// Returns a diagnostic if input validation, normalization, encoding or reconstruction fails.
pub fn xml_to_midi2(bytes: &[u8], policy: &model::RenderingPolicy) -> Result<Conversion> {
    render_score(bytes, policy, None)
}
/// Render score meaning and explicit receiver adaptation before publishing MIDI.
/// The plan is an internal conversion input; the output MIDI and companion carry
/// the complete deterministic assignment and performed values.
/// # Errors
/// Returns conversion errors, incomplete bindings, ambiguous adaptation rules,
/// unsupported required controls and receiver-state conflicts.
pub fn xml_to_rendition(
    bytes: &[u8],
    policy: &model::RenderingPolicy,
    plan: &adapt::Plan,
) -> Result<Conversion> {
    render_score(bytes, policy, Some(plan))
}
fn render_score(
    bytes: &[u8],
    policy: &model::RenderingPolicy,
    plan: Option<&adapt::Plan>,
) -> Result<Conversion> {
    let (source, score) = xml::read(bytes, policy)?;
    let (score, provenance) = if let Some(plan) = plan {
        let voices = if plan.rules.iter().any(|rule| rule.source_voice.is_some()) {
            xml::note_voices(&source)?
        } else {
            std::collections::BTreeMap::new()
        };
        let (adapted, provenance) = adapt::render_with_voices(&score, plan, &voices)?;
        (adapted, Some(provenance))
    } else {
        (score, None)
    };
    let clip::Encoded {
        bytes: clip,
        dictionary,
        report,
        mut note_sources,
        source_indices,
    } = clip::encode_with_sources(&score)?;
    if let Some(provenance) = provenance {
        for (source, index) in note_sources.iter_mut().zip(source_indices) {
            let note = &score.notes[index];
            if source.encoded_route != note.route {
                return Err(Diagnostic::new(
                    "E_RENDITION_ROUTE_ISOLATION",
                    format!(
                        "encoder requires an unplanned isolation route for {}",
                        note.source
                    ),
                ));
            }
            source.source_route = provenance[index].route;
            source.source_velocity =
                (provenance[index].velocity != note.velocity).then_some(provenance[index].velocity);
        }
    }
    let decoded = clip::decode(&clip, Some(&dictionary))?;
    let mut companion =
        preserve::Companion::new(bytes, &source, &clip, &decoded, dictionary, policy);
    companion.note_sources = note_sources;
    if let Some(plan) = plan {
        companion.set_rendition(plan.rendition.clone(), &decoded)?;
    }
    Ok(Conversion {
        clip,
        companion,
        report,
    })
}
/// Decode actual events before consulting source-aware preservation data.
/// # Errors
/// Returns a diagnostic if input validation, normalization, encoding or reconstruction fails.
pub fn midi2_to_xml(
    bytes: &[u8],
    companion: Option<&preserve::Companion>,
    compressed: bool,
) -> Result<Vec<u8>> {
    let score = clip::decode(bytes, companion.map(|c| &c.dictionary))?;
    let source = if let Some(c) = companion.filter(|c| c.matches(bytes, &score)) {
        xml::source_aware_xml(&c.source_xml)?
    } else {
        notation::write(&score)?
    };
    if compressed {
        xml::compress(&source)
    } else {
        Ok(source.into_bytes())
    }
}
