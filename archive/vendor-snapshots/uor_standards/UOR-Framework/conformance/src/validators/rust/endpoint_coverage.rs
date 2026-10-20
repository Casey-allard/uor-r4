//! Layer 4: endpoint-coverage gate.
//!
//! Cross-references `foundation/tests/public-api.snapshot` against the
//! audit mapping at `conformance/endpoint_coverage.toml`. Every public
//! symbol must be either (a) mapped to a `behavior_*.rs` test file, or
//! (b) declared in an exemption category. Unmapped symbols fail the
//! conformance run.
//!
//! This gate closes the "new public endpoint without a behavior test"
//! loophole: every PR that adds a symbol to the snapshot must also add
//! the coverage mapping, which forces a review.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/endpoint_coverage";

/// Runs the endpoint-coverage gate.
///
/// # Errors
///
/// Returns an error if the snapshot or coverage TOML can't be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // Load the public-api snapshot.
    let snapshot_path = workspace.join("foundation/tests/public-api.snapshot");
    let snapshot = match fs::read_to_string(&snapshot_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read {}: {e}", snapshot_path.display()),
            ));
            return Ok(report);
        }
    };

    // Extract the set of symbol names from the snapshot. Each line has
    // the format `<path>: <kind> <name>`. We only care about names.
    let mut snapshot_symbols: BTreeSet<String> = BTreeSet::new();
    for line in snapshot.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Split at the first ': ' — the RHS is `kind name`.
        let rhs = match trimmed.split_once(": ") {
            Some((_, r)) => r,
            None => continue,
        };
        // Split off the kind token ('struct', 'fn', 'trait', etc.).
        let name = match rhs.split_once(' ') {
            Some((_, n)) => n.trim(),
            None => continue,
        };
        if !name.is_empty() {
            snapshot_symbols.insert(name.to_string());
        }
    }

    // Load the coverage TOML.
    let toml_path = workspace.join("conformance/endpoint_coverage.toml");
    let toml_src = match fs::read_to_string(&toml_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read {}: {e}", toml_path.display()),
            ));
            return Ok(report);
        }
    };

    // We do a string-based extraction of all `symbols = [...]` arrays.
    // This avoids pulling a TOML dependency into the conformance crate.
    // The extraction is rough but stable for the TOML format we control.
    let mut mapped: BTreeSet<String> = BTreeSet::new();
    let mut in_array = false;
    let mut current: String = String::new();
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("symbols = [") {
            in_array = true;
            current.clear();
            current.push_str(trimmed);
            if trimmed.ends_with(']') || trimmed.ends_with("],") {
                // Single-line array.
                in_array = false;
                extract_quoted_into(&current, &mut mapped);
                current.clear();
            }
            continue;
        }
        if in_array {
            current.push_str(trimmed);
            if trimmed.ends_with(']') || trimmed.ends_with("],") {
                in_array = false;
                extract_quoted_into(&current, &mut mapped);
                current.clear();
            }
        }
    }

    // Verify every snapshot symbol is either mapped or exempt. Since the
    // TOML merges coverage and exempt entries into the same `mapped`
    // set, any unmapped symbol is a gap.
    let mut unmapped: Vec<String> = Vec::new();
    for sym in &snapshot_symbols {
        if !mapped.contains(sym) {
            unmapped.push(sym.clone());
        }
    }

    // Also check that every mapped symbol corresponds to a real snapshot
    // entry — catches stale mappings after a symbol is removed.
    let mut stale: Vec<String> = Vec::new();
    for sym in &mapped {
        if !snapshot_symbols.contains(sym) {
            stale.push(sym.clone());
        }
    }

    if unmapped.is_empty() && stale.is_empty() {
        let total = snapshot_symbols.len();
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "endpoint_coverage: all {total} public symbols in public-api.snapshot \
                 are either mapped to a behavior_*.rs test or declared as an exemption \
                 in conformance/endpoint_coverage.toml"
            ),
        ));
    } else {
        let mut details: Vec<String> = Vec::new();
        for sym in &unmapped {
            details.push(format!(
                "unmapped public symbol `{sym}` \u{2014} add a coverage mapping \
                 in endpoint_coverage.toml or a behavior_{}.rs test",
                sym.to_lowercase()
            ));
        }
        for sym in &stale {
            details.push(format!(
                "stale mapping for `{sym}` \u{2014} symbol no longer in public-api.snapshot",
            ));
        }
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "endpoint_coverage: {} unmapped, {} stale",
                unmapped.len(),
                stale.len()
            ),
            details,
        ));
    }

    Ok(report)
}

/// Extracts quoted string values from a snippet like `["Foo", "Bar", "Baz"]`
/// and inserts them into `out`.
fn extract_quoted_into(snippet: &str, out: &mut BTreeSet<String>) {
    let bytes = snippet.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            if j > start && j < bytes.len() {
                if let Ok(s) = core::str::from_utf8(&bytes[start..j]) {
                    out.insert(s.to_string());
                }
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
}
