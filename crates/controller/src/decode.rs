//! Decode actual performance bytes; never reinterpret archived notation.
use crate::files::{Error, read_bounded};
use musicxml_midi2_convert::{clip, model::Performance, preserve::Companion};
use std::path::Path;

/// Decode and authenticate portable encoded routes, without applying Bitwig slot routing.
/// # Errors
/// Returns malformed clips, inconsistent performed values and unsupported companion versions.
pub fn read(input: &Path, companion: &Path) -> Result<(Performance, Companion), Error> {
    let bytes = read_bounded(input)?;
    let envelope: Companion = serde_json::from_slice(&read_bounded(companion)?)?;
    if !matches!(envelope.version, 1 | 2) || envelope.mapping_version != 2 {
        return Err("E_BUNDLE_VERSION: unsupported companion version".into());
    }
    let performance = clip::decode(&bytes, Some(&envelope.dictionary))?;
    envelope.validate_rendition(&performance)?;
    if !envelope.note_sources.is_empty() {
        if envelope.note_sources.len() != performance.notes.len() {
            return Err("E_PROVENANCE: ownership note count differs".into());
        }
        for (note, source) in performance.notes.iter().zip(&envelope.note_sources) {
            if note.onset != source.onset
                || note.end != source.end
                || note.route != source.encoded_route
                || note.key != source.key
                || source.source.is_empty()
            {
                return Err(
                    "E_PROVENANCE: ownership does not identify the actual decoded note".into(),
                );
            }
        }
    }
    Ok((performance, envelope))
}
