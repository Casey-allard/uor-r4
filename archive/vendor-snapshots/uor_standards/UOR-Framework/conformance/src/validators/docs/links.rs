//! Documentation internal link validator.
//!
//! Verifies that all internal links in generated HTML docs point to existing pages.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates internal links in all docs HTML files.
///
/// # Errors
///
/// Returns an error if the docs directory cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let base_path = std::env::var("PUBLIC_BASE_PATH").unwrap_or_default();
    let base_path = base_path.trim_end_matches('/').to_string();

    let docs_dir = artifacts.join("docs");
    if !docs_dir.exists() {
        report.push(TestResult::warn(
            "docs/links",
            "public/docs/ not found — skipping link check",
        ));
        return Ok(report);
    }

    // Collect ALL files in artifacts root (HTML, CSS, JS, JSON, etc.)
    // for checking absolute links (e.g., /css/style.css).
    let all_artifacts_files: HashSet<String> = WalkDir::new(artifacts)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(artifacts)
                .unwrap_or(e.path())
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    // Collect docs HTML files (paths relative to artifacts root)
    // for checking relative links within docs/.
    let docs_html_files: HashSet<String> = WalkDir::new(&docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
        .map(|e| {
            e.path()
                .strip_prefix(artifacts)
                .unwrap_or(e.path())
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    let mut broken: Vec<String> = Vec::new();

    for entry in WalkDir::new(&docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        let file_path = entry.path();
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_label = file_path
            .strip_prefix(artifacts)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        // Parent directory of this file, relative to artifacts root
        let parent_rel = file_path
            .parent()
            .and_then(|p| p.strip_prefix(artifacts).ok())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        // Extract href values from anchor and link tags
        for href in extract_hrefs(&content) {
            // Only check internal links (not http://, https://, mailto:, #fragments)
            if href.starts_with("http://")
                || href.starts_with("https://")
                || href.starts_with("mailto:")
                || href.starts_with('#')
                || href.is_empty()
            {
                continue;
            }

            let (resolved, check_all_artifacts) = if href.starts_with('/') {
                // Absolute site-root path — strip base_path prefix, then resolve against artifacts root
                let without_base = if !base_path.is_empty() {
                    href.strip_prefix(&base_path).unwrap_or(href.as_str())
                } else {
                    href.as_str()
                };
                let path = normalize_path(without_base.trim_start_matches('/'));
                (path, true)
            } else {
                // Relative path — resolve from this file's parent within docs
                let raw = if parent_rel.is_empty() {
                    href.clone()
                } else {
                    format!("{}/{}", parent_rel, href)
                };
                (normalize_path(&raw), false)
            };

            // Strip fragment
            let resolved = resolved.split('#').next().unwrap_or(&resolved).to_string();

            if resolved.is_empty() {
                continue;
            }

            let found = if check_all_artifacts {
                // For absolute paths: check the exact file or an index.html fallback
                all_artifacts_files.contains(&resolved)
                    || all_artifacts_files
                        .contains(&format!("{}/index.html", resolved.trim_end_matches('/')))
            } else {
                // For relative paths: check docs HTML files
                docs_html_files.contains(&resolved)
            };

            if !found {
                broken.push(format!("{}: broken link → {}", file_label, href));
            }
        }
    }

    if broken.is_empty() {
        report.push(TestResult::pass(
            "docs/links",
            "No broken internal links in documentation",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "docs/links",
            format!("{} broken internal link(s) in documentation", broken.len()),
            broken,
        ));
    }

    Ok(report)
}

/// Normalizes a path by resolving `.` and `..` components.
///
/// For example, `docs/concepts/../namespaces/u.html` → `docs/namespaces/u.html`.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

/// Extracts href attribute values from HTML content.
fn extract_hrefs(html: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut remaining = html;

    while let Some(idx) = remaining.find("href=\"") {
        remaining = &remaining[idx + 6..];
        if let Some(end) = remaining.find('"') {
            hrefs.push(remaining[..end].to_string());
            remaining = &remaining[end + 1..];
        }
    }

    hrefs
}
