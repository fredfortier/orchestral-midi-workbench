//! Development-only selection probe; production uses native paths.
use notes_transformer::selection::Request;
use std::error::Error;
pub fn probe(input: &std::path::Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let request: Request = serde_json::from_slice(&notes_transformer::files::read_bounded(input)?)?;
    request.validate()?;
    notes_transformer::adapter::arranger_probe(&request)
}
