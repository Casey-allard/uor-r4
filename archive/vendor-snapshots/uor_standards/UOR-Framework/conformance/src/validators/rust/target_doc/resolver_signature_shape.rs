//! Target-doc cross-ref A.2: every `resolver::<name>::certify` signature
//! matches target §4.2's prescribed shape:
//!
//! - First parameter: `&Validated<…>`
//! - Return type: `Result<Certified<…>, Certified<…>>`
//!
//! Exception: `multiplication::certify` takes `&MulContext` (a
//! pre-validated inline shape whose construction enforces the resolver's
//! admissibility preconditions).

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/target_doc/resolver_signature_shape";

/// Whitelist of resolver modules exempt from the §4.2 signature shape.
const SIGNATURE_EXEMPT_MODULES: &[&str] = &["multiplication"];

/// Runs the §4.2 resolver signature-shape check.
///
/// # Errors
///
/// Returns an error if `foundation/src/enforcement.rs` cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let path = workspace.join("foundation/src/enforcement.rs");
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", path.display()),
            ));
            return Ok(report);
        }
    };

    let mut violations: Vec<String> = Vec::new();
    let mut resolver_count = 0usize;
    for (module_name, certify_signature) in extract_resolver_certify_sigs(&content) {
        if SIGNATURE_EXEMPT_MODULES.contains(&module_name.as_str()) {
            continue;
        }
        resolver_count += 1;
        if !certify_signature.contains("&Validated<") {
            violations.push(format!(
                "resolver::{module_name}::certify parameter is not `&Validated<_>` — sig: {}",
                summarize(&certify_signature)
            ));
        }
        if !has_certified_error_side(&certify_signature) {
            violations.push(format!(
                "resolver::{module_name}::certify error type is not `Certified<_>` — sig: {}",
                summarize(&certify_signature)
            ));
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "target §4.2 resolver signature shape: {} resolvers conform (input `&Validated<_>`, error `Certified<_>`); `multiplication` exempt by design",
                resolver_count
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "target §4.2 resolver signature shape: {} deviations",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}

/// Scan for `pub mod <name> { ... pub fn certify(...)...}` blocks and
/// extract `(module_name, full_certify_signature_string)` pairs. Uses a
/// simple line-based walker that tracks brace depth for module nesting.
fn extract_resolver_certify_sigs(src: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    // Find every `pub mod <ident> {` line.
    let mut i = 0usize;
    while i < src.len() {
        let rem = &src[i..];
        let Some(off) = rem.find("pub mod ") else {
            break;
        };
        let start = i + off + "pub mod ".len();
        // Ident ends at whitespace or '{'.
        let ident_end = src[start..]
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map(|o| start + o)
            .unwrap_or(src.len());
        let ident = src[start..ident_end].to_string();
        // Find the `{` opening the module body.
        let Some(brace_off) = src[ident_end..].find('{') else {
            i = ident_end;
            continue;
        };
        let body_start = ident_end + brace_off + 1;
        // Find the matching closing brace via depth tracking.
        let mut depth: i32 = 1;
        let mut j = body_start;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        let body_end = j;
        // Within the module body, find `pub fn certify(` (not
        // `certify_at(` — we want the default-level entry point).
        let body = &src[body_start..body_end];
        if let Some(cert_off) = find_certify_sig(body) {
            // Capture from `pub fn certify` up through the `{` or `;`
            // that ends the signature.
            let abs = body_start + cert_off;
            if let Some(end_rel) = src[abs..].find('{') {
                let sig = src[abs..abs + end_rel].to_string();
                out.push((ident.clone(), sig));
            }
        }
        i = body_end;
    }
    out
}

/// Find `pub fn certify(` or `pub fn certify<` within a module body;
/// excludes `pub fn certify_at`.
fn find_certify_sig(body: &str) -> Option<usize> {
    let needles = ["pub fn certify<", "pub fn certify("];
    let mut best: Option<usize> = None;
    for n in &needles {
        if let Some(off) = body.find(n) {
            best = Some(best.map_or(off, |b| b.min(off)));
        }
    }
    best
}

/// Does the certify signature's return type carry `Certified<…>` on both
/// the success and error sides? Check for two `Certified<` substrings
/// within the `Result<...>` block.
fn has_certified_error_side(sig: &str) -> bool {
    let Some(result_idx) = sig.find("Result<") else {
        return false;
    };
    let tail = &sig[result_idx..];
    tail.matches("Certified<").count() >= 2
}

/// Collapse a multiline signature into a single-line summary for
/// violation messages. Truncate to 160 chars.
fn summarize(sig: &str) -> String {
    let one_line: String = sig
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.len() > 160 {
        format!("{}…", &one_line[..160])
    } else {
        one_line
    }
}
