//! Writes website output files.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Writes content to a file, creating parent directories as needed.
///
/// # Errors
///
/// Returns an error if directories cannot be created or the file cannot be written.
pub fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create directory: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Cannot write file: {}", path.display()))?;
    Ok(())
}
