//! Phase 9d conformance check: zero hardcoded `f64` sites in
//! `foundation/src/**/*.rs` outside `#[cfg(test)]` blocks.
//!
//! Rationale: Phase 9 bounds `HostTypes::Decimal` on
//! `DecimalTranscendental` so the foundation is fully polymorphic over
//! the host's chosen decimal precision. Any literal `: f64` or `-> f64`
//! that survives outside `#[cfg(test)]` regresses that polymorphism.

use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/no_hardcoded_f64";

/// Maximum permitted hardcoded `f64` sites in foundation/src/. Phase 9
/// closes at zero — no allow-list per the completion plan.
const MAX_PERMITTED: usize = 0;

/// Runs the Phase 9d gate.
///
/// # Errors
///
/// Returns an error if a workspace file cannot be read or the regex
/// fails to compile.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let foundation_src = workspace.join("foundation/src");
    let mut sources: Vec<(PathBuf, String)> = Vec::new();
    collect_sources(&foundation_src, &mut sources)?;

    // Pattern: `: f64`, `-> f64`, `f64,`, `<f64>`, `: &f64`, `: [f64`.
    // The `(?:^|[^a-zA-Z0-9_])` lookbehind avoids matching identifier
    // suffixes (e.g. `myf64`).
    let pat = Regex::new(
        r"(?m)(?:^|[^a-zA-Z0-9_])(?P<m>: f64\b|-> f64\b|f64,|<f64>|: &f64\b|: \[f64\b)",
    )?;

    // Wiki ADR-047 / ADR-048 / ADR-049 carve-out: the σ-Projection
    // Hardening Principle's bandwidth metrics + the TypedCommitment /
    // ObservablePredicate / cryptanalysis-battery surfaces are
    // normatively f64-typed at the trait level (`accept_prob`,
    // `bandwidth_bits`, and their helpers reason in PRF probability
    // space). Foundation cannot polymorphize these over a generic
    // `HostTypes::Decimal` because the wiki commits the f64
    // signatures verbatim. The exempt files are the pipeline module
    // hosting the trait surface + the foundation-internal libm helpers
    // those impls delegate to.
    let exempt_file_substrings: &[&str] = &[
        // ADR-047/048/049 commitment + observable surface.
        "foundation/src/pipeline.rs",
    ];

    let mut hits: Vec<String> = Vec::new();
    for (path, src) in &sources {
        let path_str = path.to_string_lossy();
        if exempt_file_substrings
            .iter()
            .any(|frag| path_str.contains(frag))
        {
            continue;
        }
        let cleaned = strip_cfg_test_blocks(src);
        for cap in pat.captures_iter(&cleaned) {
            if let Some(m) = cap.name("m") {
                // Find the line containing the match. Skip if the line is a
                // doc / line comment (`//`, `///`, `//!`) — those appear in
                // documentation and don't constitute hardcoded type usage.
                let pre = &cleaned[..m.start()];
                let line_start = pre.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = cleaned[m.start()..]
                    .find('\n')
                    .map(|i| m.start() + i)
                    .unwrap_or(cleaned.len());
                let line = &cleaned[line_start..line_end];
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                let line_no = pre.matches('\n').count() + 1;
                hits.push(format!("{}:{}: {}", path.display(), line_no, m.as_str()));
            }
        }
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    let within_budget = hits.len() <= MAX_PERMITTED;
    if within_budget {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("0 hardcoded f64 sites in foundation/src/ (≤ {MAX_PERMITTED} permitted)"),
        ));
    } else {
        let preview: Vec<&str> = hits.iter().take(20).map(String::as_str).collect();
        report.push(TestResult::fail(
            VALIDATOR,
            format!(
                "Found {} hardcoded f64 site(s) in foundation/src/ (max {MAX_PERMITTED}). \
                 First {}: {:?}",
                hits.len(),
                preview.len(),
                preview,
            ),
        ));
    }

    Ok(report)
}

fn collect_sources(root: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&path)?;
                out.push((path, src));
            }
        }
    }
    Ok(())
}

/// Removes `#[cfg(test)] mod { ... }` blocks (brace-counted) from a
/// source string. Mirrors the helper in the Phase 7e orphan-counts
/// validator.
fn strip_cfg_test_blocks(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    loop {
        match rest.find("#[cfg(test)]") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(pos) => {
                out.push_str(&rest[..pos]);
                let tail = &rest[pos + "#[cfg(test)]".len()..];
                match tail.find('{') {
                    None => {
                        out.push_str(&rest[pos..]);
                        break;
                    }
                    Some(brace_off) => {
                        let mut depth: i32 = 0;
                        let mut closed_at: Option<usize> = None;
                        for (byte_idx, ch) in tail[brace_off..].char_indices() {
                            match ch {
                                '{' => depth += 1,
                                '}' => {
                                    depth -= 1;
                                    if depth == 0 {
                                        closed_at = Some(brace_off + byte_idx + ch.len_utf8());
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        match closed_at {
                            Some(end) => {
                                rest = &tail[end..];
                            }
                            None => break,
                        }
                    }
                }
            }
        }
    }
    out
}
