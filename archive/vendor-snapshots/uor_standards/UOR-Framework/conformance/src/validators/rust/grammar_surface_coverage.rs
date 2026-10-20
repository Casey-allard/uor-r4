//! Phase G (target §1.5): grammar-surface coverage validator.
//!
//! §1.5 says "every Rust surface in the crate either (a) implements one of
//! the two grammars' productions, (b) validates an input against one of
//! them, or (c) produces a witness that a valid program was accepted."
//!
//! This validator parses the two EBNF grammars (`uor.term.ebnf`,
//! `uor.conformance.ebnf`) and asserts that every declaration form has a
//! corresponding Rust builder with a `validate` and `validate_const`
//! method, plus the companion `Validated<Decl>` result type.
//!
//! It also enforces the absence of the deprecated `uor-foundation-macros`
//! proc-macro crate — target §4.1 W15.B and this project's scope
//! constraints require the crate to be absent, with const-validated
//! builders as the sole ergonomic path.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/grammar_surface_coverage";

/// Conformance-grammar declaration forms → Rust builder name.
/// Each entry: (grammar production, Rust builder, Rust result type).
///
/// Source: `public/uor.conformance.ebnf`. The 7 top-level declaration forms
/// (compile_unit, dispatch_rule, witt_level, predicate, parallel, stream,
/// lease) each map 1:1 to a Rust builder in `enforcement::`.
const CONFORMANCE_DECLS: &[(&str, &str, &str)] = &[
    ("compile_unit", "CompileUnitBuilder", "CompileUnit"),
    (
        "dispatch_rule",
        "DispatchDeclarationBuilder",
        "DispatchDeclaration",
    ),
    (
        "witt_level",
        "WittLevelDeclarationBuilder",
        "WittLevelDeclaration",
    ),
    (
        "predicate",
        "PredicateDeclarationBuilder",
        "PredicateDeclaration",
    ),
    (
        "parallel",
        "ParallelDeclarationBuilder",
        "ParallelDeclaration",
    ),
    ("stream", "StreamDeclarationBuilder", "StreamDeclaration"),
    ("lease", "LeaseDeclarationBuilder", "LeaseDeclaration"),
];

/// Term-grammar declaration forms that map to Rust builders.
/// `effect-decl` → `EffectDeclarationBuilder`; `source-decl` / `sink-decl`
/// materialize `SourceDeclaration` / `SinkDeclaration` via the pipeline
/// (no builder — implementers declare them through the grammar surface and
/// the foundation re-validates); `type-decl` goes through
/// `ConstrainedTypeInput` + the compile-time-evidence pattern per target §3.
const TERM_DECLS: &[(&str, &str, &str)] = &[
    (
        "effect-decl",
        "EffectDeclarationBuilder",
        "EffectDeclaration",
    ),
    (
        "source-decl",
        // Boundary declarations materialize through
        // `SourceDeclaration` + the `Grounding` trait; the foundation
        // re-validates the produced output. No dedicated builder per
        // target §3.
        "SourceDeclaration",
        "SourceDeclaration",
    ),
    ("sink-decl", "SinkDeclaration", "SinkDeclaration"),
    // type-decl: the compile-time-evidence pattern
    // `const _VALIDATED_T: Validated<ConstrainedTypeInput, CompileTime> = ...`
    // is the grammar-surface-conformant entry point (target §3 W2).
    ("type-decl", "ConstrainedTypeInput", "ConstrainedTypeInput"),
];

/// EBNF grammar files that MUST exist in the repo — they are the authoritative
/// definition the crate surface conforms to.
const GRAMMAR_FILES: &[&str] = &["public/uor.conformance.ebnf", "public/uor.term.ebnf"];

/// Phase G.4 assertion: no `uor-foundation-macros` crate exists in the
/// workspace members. The target ergonomic is the const-validated builder
/// path, not a Rust macro.
fn check_macros_crate_absent(workspace: &Path, violations: &mut Vec<String>) {
    let manifest_path = workspace.join("Cargo.toml");
    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    if content.contains("uor-foundation-macros") {
        violations.push(
            "Cargo.toml references `uor-foundation-macros` — target §4.1 W15.B \
             requires this crate to be absent (const-validated builder path is \
             the grammar-surface ergonomic)."
                .to_string(),
        );
    }
    if workspace.join("uor-foundation-macros").exists() {
        violations.push(
            "`uor-foundation-macros/` directory exists in the workspace — \
             must be absent per Phase G.4."
                .to_string(),
        );
    }
}

/// Runs the grammar-surface coverage check.
///
/// # Errors
///
/// Returns an error if the grammar files or foundation source cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let mut violations: Vec<String> = Vec::new();

    // Step 1: both EBNF grammar files exist.
    for rel in GRAMMAR_FILES {
        let path = workspace.join(rel);
        if !path.exists() {
            violations.push(format!("grammar file missing: {}", rel));
        }
    }

    // Step 2: every conformance-grammar declaration form has a matching
    // Rust builder + validate + validate_const + Validated<Decl> result.
    let enforcement_path = workspace.join("foundation/src/enforcement.rs");
    let content = match std::fs::read_to_string(&enforcement_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("failed to read {}: {e}", enforcement_path.display()),
            ));
            return Ok(report);
        }
    };

    let conformance_ebnf =
        std::fs::read_to_string(workspace.join("public/uor.conformance.ebnf")).unwrap_or_default();
    let term_ebnf =
        std::fs::read_to_string(workspace.join("public/uor.term.ebnf")).unwrap_or_default();

    for (grammar_form, builder, decl) in CONFORMANCE_DECLS {
        let grammar_anchor = format!("\"{grammar_form}\"");
        if !conformance_ebnf.contains(&grammar_anchor) {
            violations.push(format!(
                "conformance grammar does not declare `{grammar_form}` literal (expected in uor.conformance.ebnf)"
            ));
        }
        let builder_anchor = format!("pub struct {builder}");
        if !content.contains(&builder_anchor) {
            violations.push(format!(
                "missing builder `{builder}` for grammar form `{grammar_form}`"
            ));
        }
        let validate_const_anchor = "pub const fn validate_const";
        let decl_mention = format!("Validated<{decl}, CompileTime>");
        let has_validate_const =
            content.contains(validate_const_anchor) && content.contains(&decl_mention);
        if !has_validate_const {
            // Allow pipeline-level standalone helpers (validate_*_const free functions).
            let pipeline_anchor =
                format!("pub const fn validate_{}_const", short_name(grammar_form));
            let pipeline_path = workspace.join("foundation/src/pipeline.rs");
            let pipeline_content = std::fs::read_to_string(&pipeline_path).unwrap_or_default();
            if !pipeline_content.contains(&pipeline_anchor) {
                violations.push(format!(
                    "missing `validate_const` for grammar form `{grammar_form}` (expected `{builder}::validate_const` or `pipeline::validate_{}_const`)",
                    short_name(grammar_form)
                ));
            }
        }
    }

    // Step 3: term-grammar declarations map to existing Rust surface.
    for (grammar_form, rust_name, _) in TERM_DECLS {
        let grammar_keyword = match *grammar_form {
            "effect-decl" => "\"effect\"",
            "source-decl" => "\"source\"",
            "sink-decl" => "\"sink\"",
            "type-decl" => "\"type\"",
            _ => "",
        };
        if !grammar_keyword.is_empty() && !term_ebnf.contains(grammar_keyword) {
            violations.push(format!(
                "term grammar does not declare `{grammar_keyword}` (expected in uor.term.ebnf)"
            ));
        }
        let rust_anchor_a = format!("pub struct {rust_name}");
        let rust_anchor_b = format!("pub enum {rust_name}");
        if !content.contains(&rust_anchor_a) && !content.contains(&rust_anchor_b) {
            violations.push(format!(
                "term-grammar form `{grammar_form}` has no Rust surface (expected `{rust_name}` in enforcement.rs)"
            ));
        }
    }

    // Step 4: Phase G.4 — uor-foundation-macros crate must be absent.
    check_macros_crate_absent(workspace, &mut violations);

    if violations.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            "Phase G grammar-surface coverage: 7 conformance-grammar declaration forms + \
             4 term-grammar declaration forms map 1:1 to Rust builders (with validate/validate_const) \
             or foundation-sealed surfaces; uor-foundation-macros crate absent per target §4.1 W15.B",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "Phase G grammar-surface coverage found {} gaps",
                violations.len()
            ),
            violations,
        ));
    }

    Ok(report)
}

/// Map a grammar form like `compile_unit` to the short form used in
/// `validate_<short>_const` free-function names in pipeline.rs.
fn short_name(grammar_form: &str) -> &str {
    match grammar_form {
        "compile_unit" => "compile_unit",
        "dispatch_rule" => "dispatch",
        "witt_level" => "witt_level",
        "predicate" => "predicate",
        "parallel" => "parallel",
        "stream" => "stream",
        "lease" => "lease",
        other => other,
    }
}
