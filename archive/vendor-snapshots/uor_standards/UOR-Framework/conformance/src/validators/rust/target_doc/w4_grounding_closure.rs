//! Target-doc cross-ref A.4: W4 closure is structurally enforced.
//!
//! Authority: target §4.3 + §9 criterion 1. Asserts:
//!
//! - `pub trait Grounding { ... }` does NOT contain `fn ground(` as a
//!   required or provided method.
//! - `pub trait GroundingExt` exists and requires `Grounding` + a sealed
//!   supertrait.
//! - A blanket `impl<G: Grounding> GroundingExt for G` exists.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/target_doc/w4_grounding_closure";

/// Runs the W4-closure structural check.
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

    // 1) `pub trait Grounding {` body must not contain `fn ground(`.
    if let Some(body) = extract_trait_body(&content, "Grounding") {
        if body.contains("fn ground(") {
            violations.push(
                "`Grounding` trait still declares `fn ground(...)` — W4 closure requires this to live on `GroundingExt` only"
                    .to_string(),
            );
        }
    } else {
        violations.push(
            "could not locate `pub trait Grounding {` in foundation/src/enforcement.rs".to_string(),
        );
    }

    // 2) `pub trait GroundingExt: Grounding` must exist.
    let has_ext_trait = content.contains("pub trait GroundingExt")
        && content.contains("GroundingExt:")
        && content.contains("Grounding +");
    if !has_ext_trait {
        violations.push(
            "`pub trait GroundingExt: Grounding + <sealed>::Sealed` declaration not found"
                .to_string(),
        );
    }

    // 3) Sealed supertrait — the ext trait must inherit from a `Sealed`
    // marker in a crate-private module.
    let has_sealed_supertrait =
        content.contains("grounding_ext_sealed::Sealed") || content.contains("sealed::Sealed");
    if !has_sealed_supertrait {
        violations.push(
            "`GroundingExt` does not have a sealed supertrait — downstream could impl `GroundingExt` directly"
                .to_string(),
        );
    }

    // 4) Blanket `impl<G: Grounding> GroundingExt for G`.
    let has_blanket = content.contains("impl<G: Grounding> GroundingExt for G")
        || content.contains("impl<G> GroundingExt for G\nwhere\n    G: Grounding");
    if !has_blanket {
        violations.push(
            "foundation blanket `impl<G: Grounding> GroundingExt for G` not found".to_string(),
        );
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "target §9 criterion 1: W4 closure structurally enforced — `Grounding` has no `fn ground`, `GroundingExt` is sealed, foundation blanket impl supplies the body",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "W4 closure not complete: {} structural gap(s)",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}

/// Extract the body (between `{` and matching `}`) of a top-level
/// `pub trait <name>` declaration. Returns None if not found.
fn extract_trait_body(src: &str, trait_name: &str) -> Option<String> {
    let anchor = format!("pub trait {trait_name}");
    // Skip `GroundingExt` if we're searching for `Grounding` (prefix match).
    let mut search_from = 0;
    loop {
        let rest = &src[search_from..];
        let off = rest.find(&anchor)?;
        let abs = search_from + off;
        // Ensure this isn't `GroundingExt` when searching for `Grounding`.
        let after = abs + anchor.len();
        if let Some(ch) = src[after..].chars().next() {
            if ch.is_alphanumeric() || ch == '_' {
                // False match (e.g. `pub trait GroundingExt` when searching `Grounding`).
                search_from = after;
                continue;
            }
        }
        // Find the opening `{` of the trait body.
        let brace_rel = src[abs..].find('{')?;
        let body_start = abs + brace_rel + 1;
        let bytes = src.as_bytes();
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
        return Some(src[body_start..j.saturating_sub(1)].to_string());
    }
}
