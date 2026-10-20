//! Writes generated HTML files and the machine-generated README.md.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Writes an HTML page to the given path, creating parent directories as needed.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the file cannot be written.
pub fn write_html(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write HTML: {}", path.display()))?;
    Ok(())
}

/// Writes a text file (Markdown, JSON, etc.) to the given path.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(())
}
