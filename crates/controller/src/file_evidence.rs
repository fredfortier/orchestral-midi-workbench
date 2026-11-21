//! Observed file location, size and native modification time; never modifies source files.
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileEvidence {
    pub path: PathBuf,
    pub bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_ns: Option<String>,
}
/// Observe one regular file under an explicit byte budget.
/// # Errors
/// Returns non-files, budget overflow, unavailable timestamps and I/O failures.
pub fn measure(path: &Path, max_bytes: u64) -> Result<FileEvidence, Box<dyn Error>> {
    Ok(open_measured(path, max_bytes)?.1)
}
/// Return the opened source together with its observed metadata.
/// # Errors
/// Returns non-files, budget overflow, unavailable timestamps and I/O failures.
pub fn open_measured(
    path: &Path,
    max_bytes: u64,
) -> Result<(File, FileEvidence, std::fs::Metadata), Box<dyn Error>> {
    let canonical = path.canonicalize()?;
    let file = File::open(&canonical)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err("Evidence requires a regular file within the declared byte budget".into());
    }
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let evidence = FileEvidence {
        path: canonical,
        bytes: metadata.len(),
        modified_ns: Some(modified),
    };
    Ok((file, evidence, metadata))
}

/// Require a fresh native output in an existing canonical parent without following an entry symlink.
/// # Errors
/// Refuses relative/noncanonical destinations, wrong suffixes and existing entries.
pub fn fresh_destination(path: &Path, suffix: &str) -> Result<(), Box<dyn Error>> {
    let parent = path.parent().ok_or("Missing native output parent")?;
    if !path.is_absolute()
        || path.as_os_str().len() > 4096
        || path.to_str().is_none_or(|p| p.contains('\0'))
        || path.extension().is_none_or(|e| e != suffix)
        || !parent.is_dir()
        || parent.canonicalize()? != parent
    {
        return Err("Native output requires an absolute fresh path in an existing canonical parent with the expected suffix".into());
    }
    match path.symlink_metadata() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => Err(
            "Native output destination already exists; preserve it and choose a fresh artifact"
                .into(),
        ),
    }
}

/// Read the source's current native modification time.
/// # Errors
/// Returns metadata and byte-budget failures.
pub fn modification(path: &Path, max_bytes: u64) -> Result<String, Box<dyn Error>> {
    measure(path, max_bytes)?
        .modified_ns
        .ok_or_else(|| "File modification time unavailable".into())
}
