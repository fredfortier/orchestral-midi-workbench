//! Plain bounded input and atomic JSON/file publication.
use serde_json::Value;
use std::{
    error::Error as StdError,
    fs::File,
    io::{Read, Write},
    path::Path,
};
pub type Error = Box<dyn StdError>;
pub use musicxml_midi2_convert::MAX_ARTIFACT_BYTES;

/// Refuse an unavailable evidence destination before native dispatch.
/// This checks current availability; publication still uses atomic no-clobber.
/// # Errors
/// Returns existing paths (including dangling symlinks) and unwritable/missing parents.
pub fn ensure_fresh_output(path: &Path) -> Result<(), Error> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => return Err(format!("E_EVIDENCE_EXISTS: {}", path.display()).into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let _probe = tempfile::NamedTempFile::new_in(parent)?;
    Ok(())
}

/// Read bounded offline artifacts without accepting an unlimited allocation.
/// # Errors
/// Returns file errors and oversized input diagnostics.
pub fn read_bounded(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_ARTIFACT_BYTES + 1).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())? > MAX_ARTIFACT_BYTES {
        return Err(format!("Artifact exceeds 64 MiB: {}", path.display()).into());
    }
    Ok(bytes)
}
/// Publish a report atomically without overwriting previous evidence.
/// # Errors
/// Returns I/O failures, including an existing destination.
pub fn publish(path: &Path, result: &Value) -> Result<(), Error> {
    let mut bytes = serde_json::to_vec_pretty(result)?;
    bytes.push(b'\n');
    publish_bytes(path, &bytes)
}
/// Publish bounded immutable bytes atomically, including packed native observations.
/// # Errors
/// Returns I/O, size and existing-destination failures; never overwrites earlier evidence.
pub fn publish_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    if u64::try_from(bytes.len())? > MAX_ARTIFACT_BYTES {
        return Err("Report exceeds 64 MiB; no partial evidence published".into());
    }
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    staged.persist_noclobber(path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
