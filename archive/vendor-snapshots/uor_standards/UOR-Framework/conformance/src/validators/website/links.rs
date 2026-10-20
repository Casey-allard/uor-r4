//! Website internal link validator.
//!
//! Checks that all internal links in `public/` HTML files point to existing resources.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::report::{ConformanceReport, TestResult};

/// Validates internal links across all website HTML files.
///
/// # Errors
///
/// Returns an error if HTML files cannot be read.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    if !artifacts.exists() {
        report.push(TestResult::warn(
            "website/links",
            "Artifacts directory not found — skipping link check",
        ));
        return Ok(report);
    }

    // Collect all files in the artifacts dir (both HTML and other resources)
    let all_files: HashSet<String> = WalkDir::new(artifacts)
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

    // Read the base path so we can resolve prefixed links correctly
    let base_path = std::env::var("PUBLIC_BASE_PATH").unwrap_or_default();
    let base_path = base_path.trim_end_matches('/').to_string();

    let mut broken: Vec<String> = Vec::new();

    for entry in WalkDir::new(artifacts)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "html").unwrap_or(false))
    {
        let file_path = entry.path();
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_dir = file_path
            .parent()
            .and_then(|p| p.strip_prefix(artifacts).ok())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        let file_label = file_path
            .strip_prefix(artifacts)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        for href in extract_hrefs(&content) {
            // Skip external, protocol-relative, and fragment-only links
            if href.starts_with("http://")
                || href.starts_with("https://")
                || href.starts_with("//")
                || href.starts_with("mailto:")
                || href.starts_with('#')
                || href.is_empty()
            {
                continue;
            }

            // Resolve relative link (stripping base_path prefix for root-relative links)
            let resolved = resolve_href(&rel_dir, &href, &base_path);

            // Strip fragment
            let resolved = resolved.split('#').next().unwrap_or(&resolved).to_string();

            if !resolved.is_empty() && !all_files.contains(&resolved) {
                // Try with index.html appended (directory links)
                let with_index = if resolved.ends_with('/') {
                    format!("{}index.html", resolved)
                } else {
                    format!("{}/index.html", resolved)
                };

                if !all_files.contains(&with_index) {
                    broken.push(format!("{}: broken link → {}", file_label, href));
                }
            }
        }
    }

    if broken.is_empty() {
        report.push(TestResult::pass(
            "website/links",
            "No broken internal links in website",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "website/links",
            format!("{} broken internal link(s) in website", broken.len()),
            broken,
        ));
    }

    Ok(report)
}

/// Extracts all `href` attribute values from HTML.
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

    // Also check src attributes for CSS/JS resources
    remaining = html;
    while let Some(idx) = remaining.find("src=\"") {
        remaining = &remaining[idx + 5..];
        if let Some(end) = remaining.find('"') {
            hrefs.push(remaining[..end].to_string());
            remaining = &remaining[end + 1..];
        }
    }

    hrefs
}

/// Resolves a relative href against a base directory.
///
/// Strips `PUBLIC_BASE_PATH` from root-relative links so that generated sites
/// that use a subpath prefix (e.g. `/UOR-Framework`) still resolve correctly
/// against the local `public/` directory.
fn resolve_href(base_dir: &str, href: &str, base_path: &str) -> String {
    if href.starts_with('/') {
        // Strip the base-path prefix if present, then resolve from root
        let without_base = if !base_path.is_empty() {
            href.strip_prefix(base_path).unwrap_or(href)
        } else {
            href
        };
        without_base.trim_start_matches('/').to_string()
    } else if base_dir.is_empty() {
        href.to_string()
    } else {
        normalize_path(&format!("{}/{}", base_dir, href))
    }
}

/// Normalizes a path by resolving `.` and `..` components.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "." | "" if !parts.is_empty() => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(segment),
        }
    }
    parts.join("/")
}
