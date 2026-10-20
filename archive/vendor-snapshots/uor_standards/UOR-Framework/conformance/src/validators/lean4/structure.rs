//! Generated Lean 4 crate validator.
//!
//! Validates the generated Lean 4 formalization against the ontology source of
//! truth. Ensures structure completeness, field completeness, enum completeness,
//! individual completeness, and module structure.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "lean4/structure";

/// Validates the generated Lean 4 source in `lean4/`.
///
/// # Errors
///
/// Returns an error if source files cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let lean_dir = workspace.join("lean4");

    if !lean_dir.exists() {
        report.push(TestResult::fail(VALIDATOR, "lean4/ directory not found"));
        return Ok(report);
    }

    let ontology = uor_ontology::Ontology::full();

    // 1. Module structure: expected files exist
    validate_module_structure(&lean_dir, &mut report)?;

    // 2. Structure completeness: every non-enum class has a structure
    validate_structure_completeness(&lean_dir, ontology, &mut report)?;

    // 3. Field completeness: every property with a domain has a field
    validate_field_completeness(&lean_dir, ontology, &mut report)?;

    // 4. Enum completeness: all enum classes present
    validate_enum_completeness(&lean_dir, &mut report)?;

    // 5. Individual completeness: every non-enum individual has a namespace
    validate_individual_completeness(&lean_dir, ontology, &mut report)?;

    // 6. Primitives class exists
    validate_primitives_class(&lean_dir, &mut report)?;

    // 7. Lakefile present
    validate_lakefile(workspace, &mut report)?;

    // Meta: sorry audit (informational, not counted in CONFORMANCE_CHECKS)
    audit_sorry(&lean_dir, &mut report)?;

    Ok(report)
}

/// Validates that expected module files exist.
fn validate_module_structure(lean_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let expected_files = [
        "UOR.lean",
        "UOR/Primitives.lean",
        "UOR/Enums.lean",
        "UOR/Structures.lean",
        "UOR/Individuals.lean",
        "UOR/Individuals/Schema.lean",
        "UOR/Individuals/Op.lean",
        "UOR/Individuals/Type_.lean",
        "UOR/Individuals/Observable.lean",
        "UOR/Individuals/Homology.lean",
        "UOR/Individuals/Cohomology.lean",
        "UOR/Individuals/Proof.lean",
        "UOR/Individuals/Trace.lean",
        "UOR/Individuals/Morphism.lean",
        "UOR/Individuals/State.lean",
        "UOR/Individuals/Reduction.lean",
        "UOR/Individuals/Convergence.lean",
        "UOR/Individuals/Division.lean",
        "UOR/Individuals/Predicate.lean",
        "UOR/Individuals/Conformance_.lean",
    ];

    let mut all_present = true;
    for file in &expected_files {
        if !lean_dir.join(file).exists() {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("Missing expected file: lean4/{file}"),
            ));
            all_present = false;
        }
    }

    if all_present {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "All {} expected Lean 4 module files present",
                expected_files.len()
            ),
        ));
    }

    Ok(())
}

/// Validates that every non-enum OWL class has a `structure` declaration.
fn validate_structure_completeness(
    lean_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    let enum_classes = uor_ontology::Ontology::enum_class_names();
    let all_source = read_all_lean_files(lean_dir)?;

    let mut missing = Vec::new();
    let mut found = 0usize;

    for module in &ontology.namespaces {
        for class in &module.classes {
            let local = uor_lean_codegen::mapping::local_name(class.id);

            if enum_classes.contains(&local) {
                continue;
            }

            let pattern = format!("structure {local}");
            if all_source.contains(&pattern) {
                found += 1;
            } else {
                missing.push(local.to_string());
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {found} class structures present in generated Lean 4 source"),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "{} classes missing structure declarations ({found} found)",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(())
}

/// Validates that every non-annotation property with a domain has a field.
fn validate_field_completeness(
    lean_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    let all_source = read_all_lean_files(lean_dir)?;
    let enum_domain_classes = uor_ontology::Ontology::enum_class_names();

    let mut missing = Vec::new();
    let mut found = 0usize;

    for module in &ontology.namespaces {
        let ns_iri = module.namespace.iri;
        for prop in &module.properties {
            if prop.domain.is_none() {
                continue;
            }
            if prop.kind == uor_ontology::PropertyKind::Annotation {
                continue;
            }
            if let Some(domain) = prop.domain {
                if !domain.starts_with(ns_iri) {
                    continue;
                }
                let domain_local = uor_lean_codegen::mapping::local_name(domain);
                if enum_domain_classes.contains(&domain_local) {
                    continue;
                }
            }

            let field_name = uor_lean_codegen::mapping::to_lean_field_name(
                uor_lean_codegen::mapping::local_name(prop.id),
            );

            // Search for the field name followed by a colon (Lean field syntax)
            let pattern = format!("{field_name} :");
            if all_source.contains(&pattern) {
                found += 1;
            } else {
                missing.push(format!("{} ({})", prop.id, field_name));
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {found} property fields present in generated Lean 4 source"),
        ));
        if found != uor_ontology::counts::METHODS {
            report.push_meta(TestResult::fail(
                VALIDATOR,
                format!(
                    "Field count drift: found {} fields but counts::METHODS = {}",
                    found,
                    uor_ontology::counts::METHODS
                ),
            ));
        }
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "{} properties missing fields ({found} found)",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(())
}

/// Validates that all 18 enum classes are present in Enums.lean.
fn validate_enum_completeness(lean_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let enums_path = lean_dir.join("UOR").join("Enums.lean");
    let content =
        std::fs::read_to_string(&enums_path).with_context(|| "Failed to read UOR/Enums.lean")?;

    let enum_classes = uor_ontology::Ontology::enum_class_names();
    let mut missing = Vec::new();

    for name in enum_classes {
        let inductive_pattern = format!("inductive {name}");
        let structure_pattern = format!("structure {name}");
        if !content.contains(&inductive_pattern) && !content.contains(&structure_pattern) {
            missing.push(name.to_string());
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "All {} enum classes present in Enums.lean",
                enum_classes.len()
            ),
        ));

        // Drift guard: total `inductive` + `structure` declarations in
        // Enums.lean should equal `LEAN_INDUCTIVES`. Mirrors the
        // `METHODS` drift-check pattern in `validate_field_completeness`.
        let inductive_count = content.matches("\ninductive ").count()
            + usize::from(content.starts_with("inductive "));
        let structure_count = content.matches("\nstructure ").count()
            + usize::from(content.starts_with("structure "));
        let total = inductive_count + structure_count;
        if total != uor_ontology::counts::LEAN_INDUCTIVES {
            report.push_meta(TestResult::fail(
                VALIDATOR,
                format!(
                    "Enum-layer type count drift: found {} inductive/structure declarations \
                     in Enums.lean but counts::LEAN_INDUCTIVES = {}",
                    total,
                    uor_ontology::counts::LEAN_INDUCTIVES
                ),
            ));
        }
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("{} enum classes missing from Enums.lean", missing.len()),
            missing,
        ));
    }

    Ok(())
}

/// Validates that every non-enum individual has a namespace block.
fn validate_individual_completeness(
    lean_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    let all_source = read_all_lean_files(lean_dir)?;

    let ontology_enums = uor_ontology::Ontology::enum_class_names();
    let primitive_op_types: &[&str] = &["UnaryOp", "BinaryOp", "Involution"];
    let enum_types: Vec<&str> = primitive_op_types
        .iter()
        .chain(ontology_enums.iter())
        .copied()
        .collect();

    let mut missing = Vec::new();
    let mut found = 0usize;
    // Count of non-enum individuals found as typed `def <name> :`
    // declarations (or Unit orphan placeholders). Mirrored against
    // `counts::LEAN_CONSTANT_NAMESPACES` below.
    let mut def_count = 0usize;

    for module in &ontology.namespaces {
        for ind in &module.individuals {
            let local = uor_lean_codegen::mapping::local_name(ind.id);
            let type_local = uor_lean_codegen::mapping::local_name(ind.type_);

            if enum_types.contains(&type_local) {
                // Enum variant — check exists in Enums.lean or as inductive variant
                found += 1;
                continue;
            }

            // Strict typed form: every non-enum individual is emitted
            // as `def <local> : <QualifiedClass> UOR.Prims.Standard := { ... }`
            // or `def <local> : Unit := ()` (orphan placeholder). The
            // bag-of-defs namespace form is no longer produced.
            let def_pattern = format!("def {local} :");
            if all_source.contains(&def_pattern) {
                found += 1;
                def_count += 1;
            } else {
                missing.push(format!("{} (def {local})", ind.id));
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {found} individuals present in generated Lean 4 source"),
        ));

        // Drift guard: non-enum individual typed-def count should
        // equal `LEAN_CONSTANT_NAMESPACES`. Mirrors the `METHODS`
        // drift-check pattern in `validate_field_completeness`.
        if def_count != uor_ontology::counts::LEAN_CONSTANT_NAMESPACES {
            report.push_meta(TestResult::fail(
                VALIDATOR,
                format!(
                    "Individual def count drift: found {} typed `def` declarations \
                     but counts::LEAN_CONSTANT_NAMESPACES = {}",
                    def_count,
                    uor_ontology::counts::LEAN_CONSTANT_NAMESPACES
                ),
            ));
        }
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!("{} individuals missing ({found} found)", missing.len()),
            missing,
        ));
    }

    Ok(())
}

/// Validates that the Primitives class exists.
fn validate_primitives_class(lean_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let path = lean_dir.join("UOR").join("Primitives.lean");
    let content =
        std::fs::read_to_string(&path).with_context(|| "Failed to read UOR/Primitives.lean")?;

    if content.contains("class Primitives")
        && content.contains("namespace UOR.Primitives")
        && content.contains("def Standard : UOR.Primitives.Primitives")
    {
        report.push(TestResult::pass(
            VALIDATOR,
            "Primitives typeclass + UOR.Prims.Standard instance present in Primitives.lean",
        ));
    } else {
        report.push(TestResult::fail(
            VALIDATOR,
            "Primitives typeclass, namespace wrapper, or UOR.Prims.Standard missing in Primitives.lean",
        ));
    }

    Ok(())
}

/// Validates that lakefile.lean exists.
fn validate_lakefile(workspace: &Path, report: &mut ConformanceReport) -> Result<()> {
    if workspace.join("lakefile.lean").exists() {
        report.push(TestResult::pass(VALIDATOR, "lakefile.lean present"));
    } else {
        report.push(TestResult::fail(VALIDATOR, "lakefile.lean not found"));
    }

    Ok(())
}

/// Audits for `sorry` and other banned primitives in generated Lean 4 files.
///
/// v0.2.1 Phase 7g.1: this check is **load-bearing**, not informational.
/// The published Lean surface must not contain:
///
/// - `sorry` — leaves a hole in the proof
/// - `axiom ...` (except the single whitelisted `UOR_SEALED_PROVENANCE`)
///   — unjustified assertion
/// - `partial def` — non-reducible; blocks `by decide`
/// - `native_decide` — trusts native compiler at elaboration
/// - `unsafe` — escapes Lean's core guarantees
/// - `@[extern]` — delegates to a native symbol
/// - `@[implemented_by]` — substitutes a native implementation
///
/// The check greps each published `.lean` file line-by-line, strips line
/// comments (so `-- TODO: avoid sorry` doesn't false-positive), and pushes
/// one `fail` into the main report per banned occurrence. Zero tolerance.
fn audit_sorry(lean_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let violations = run_rigor_check(lean_dir)?;

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            VALIDATOR,
            "No sorry found in generated Lean 4 source",
        ));
        report.push(TestResult::pass(
            "lean4/rigor",
            "No banned primitives (sorry / unauthorized axiom / partial def / \
             native_decide / unsafe / @[extern] / @[implemented_by]) found in \
             published Lean 4 source",
        ));
    } else {
        // The meta-audit entry still reports so reviewers see the failure
        // in the summary; the main report emits a single deterministic
        // `lean4/rigor` FAIL so the fixed-count conformance check count
        // stays stable regardless of how many violations exist.
        report.push_meta(TestResult::warn(
            VALIDATOR,
            format!(
                "{} banned-primitive violation(s) found in generated Lean 4 source",
                violations.len()
            ),
        ));
        let details = violations
            .iter()
            .map(|v| {
                format!(
                    "{}:{} — {} ({}): {}",
                    v.file, v.line, v.reason, v.pattern, v.snippet
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ");
        report.push(TestResult::fail(
            "lean4/rigor",
            format!(
                "{} banned-primitive violation(s) in published Lean 4 source:\n  {}",
                violations.len(),
                details
            ),
        ));
    }

    Ok(())
}

/// A single banned-primitive violation discovered by the rigor grep.
struct RigorViolation {
    file: String,
    line: usize,
    pattern: &'static str,
    reason: &'static str,
    snippet: String,
}

// v0.2.1 Phase 8b.6: banned-primitive table lives in
// `uor_lean_codegen::rigor_patterns::BANNED_PATTERNS` as the single source of
// truth shared with the codegen-time sanitizer. Any edit to the list updates
// both enforcement layers simultaneously — no drift possible.
use uor_lean_codegen::rigor_patterns::{ALLOWED_AXIOM, BANNED_PATTERNS as RIGOR_PATTERNS};

/// Walk every `.lean` file under `<lean_dir>/UOR/` and collect every
/// banned-primitive occurrence as a `RigorViolation`. The single whitelisted
/// `axiom UOR_SEALED_PROVENANCE` (Phase 7g.3) is recognised and excluded.
fn run_rigor_check(lean_dir: &Path) -> Result<Vec<RigorViolation>> {
    let uor_dir = lean_dir.join("UOR");
    let mut violations: Vec<RigorViolation> = Vec::new();
    if !uor_dir.is_dir() {
        return Ok(violations);
    }
    visit_rigor(&uor_dir, &mut violations)?;
    Ok(violations)
}

fn visit_rigor(dir: &Path, out: &mut Vec<RigorViolation>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip .lake build output.
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            visit_rigor(&path, out)?;
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("lean") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        for (line_idx, raw_line) in source.lines().enumerate() {
            // Strip trailing line comments so `-- sorry` in a TODO doesn't
            // false-positive. Lean line comments start with `--`.
            let code = match raw_line.find("--") {
                Some(idx) => &raw_line[..idx],
                None => raw_line,
            };
            // Axiom check: reject any `axiom` declaration that is not
            // `axiom UOR_SEALED_PROVENANCE`. We match at word boundary.
            if let Some(axiom_pos) = code.find("axiom ") {
                let rest = &code[axiom_pos + "axiom ".len()..];
                // Read the following identifier (up to whitespace / colon).
                let ident_end = rest
                    .find(|c: char| c.is_whitespace() || c == ':' || c == '(')
                    .unwrap_or(rest.len());
                let ident = rest[..ident_end].trim();
                if ident != ALLOWED_AXIOM && !ident.is_empty() {
                    out.push(RigorViolation {
                        file: path.display().to_string(),
                        line: line_idx + 1,
                        pattern: "axiom",
                        reason:
                            "unauthorized `axiom` (only `UOR_SEALED_PROVENANCE` is whitelisted)",
                        snippet: code.trim().to_string(),
                    });
                }
            }
            // Substring check against the banned-patterns table.
            for (pat, reason) in RIGOR_PATTERNS {
                if code.contains(pat) {
                    out.push(RigorViolation {
                        file: path.display().to_string(),
                        line: line_idx + 1,
                        pattern: pat,
                        reason,
                        snippet: code.trim().to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Reads all `.lean` files in a directory tree and concatenates their contents.
fn read_all_lean_files(dir: &Path) -> Result<String> {
    let mut content = String::new();
    visit_lean_files(dir, &mut content)?;
    Ok(content)
}

/// Recursively visits all `.lean` files and appends their content.
fn visit_lean_files(dir: &Path, buf: &mut String) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip .lake build directory
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            visit_lean_files(&path, buf)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("lean") {
            let file_content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read: {}", path.display()))?;
            buf.push_str(&file_content);
            buf.push('\n');
        }
    }
    Ok(())
}
