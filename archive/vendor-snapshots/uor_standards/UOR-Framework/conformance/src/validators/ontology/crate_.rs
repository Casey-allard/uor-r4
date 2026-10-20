//! Generated crate validator.
//!
//! Validates the generated `uor-foundation` crate source against the ontology
//! source of truth. Ensures trait completeness, method completeness, individual
//! completeness, and module structure.

use std::path::Path;

use anyhow::{Context, Result};

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "ontology/crate";

/// Validates the generated crate source in `foundation/src/`.
///
/// # Errors
///
/// Returns an error if source files cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let src_dir = workspace.join("foundation").join("src");

    if !src_dir.exists() {
        report.push(TestResult::fail(
            VALIDATOR,
            "foundation/src/ directory not found",
        ));
        return Ok(report);
    }

    let ontology = uor_ontology::Ontology::full();

    // 1. Module structure: 3 space directories + enums.rs + lib.rs
    validate_module_structure(&src_dir, &mut report)?;

    // 2. Trait completeness: every non-enum class has a `pub trait` declaration
    validate_trait_completeness(&src_dir, ontology, &mut report)?;

    // 3. Method completeness: every property with a domain has a method
    validate_method_completeness(&src_dir, ontology, &mut report)?;

    // 4. Individual completeness: every non-enum individual has a const module or enum variant
    validate_individual_completeness(&src_dir, ontology, &mut report)?;

    // 5. HostTypes trait exists
    validate_host_types_trait(&src_dir, &mut report)?;

    Ok(report)
}

/// Validates that the expected module files exist.
fn validate_module_structure(src_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let expected_files = [
        "lib.rs",
        "enforcement.rs",
        "enums.rs",
        "kernel/mod.rs",
        "kernel/address.rs",
        "kernel/schema.rs",
        "kernel/op.rs",
        "kernel/carry.rs",
        "kernel/reduction.rs",
        "kernel/convergence.rs",
        "kernel/division.rs",
        "kernel/monoidal.rs",
        "kernel/operad.rs",
        "kernel/effect.rs",
        "kernel/predicate.rs",
        "kernel/parallel.rs",
        "kernel/stream.rs",
        "kernel/failure.rs",
        "kernel/linear.rs",
        "kernel/recursion.rs",
        "kernel/region.rs",
        "bridge/mod.rs",
        "bridge/query.rs",
        "bridge/resolver.rs",
        "bridge/partition.rs",
        "bridge/observable.rs",
        "bridge/homology.rs",
        "bridge/cohomology.rs",
        "bridge/proof.rs",
        "bridge/derivation.rs",
        "bridge/trace.rs",
        "bridge/cert.rs",
        "bridge/interaction.rs",
        "bridge/boundary.rs",
        "bridge/conformance_.rs",
        "user/mod.rs",
        "user/type_.rs",
        "user/morphism.rs",
        "user/state.rs",
    ];

    let mut all_present = true;
    for file in &expected_files {
        if !src_dir.join(file).exists() {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("Missing expected file: foundation/src/{file}"),
            ));
            all_present = false;
        }
    }

    if all_present {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {} expected module files present", expected_files.len()),
        ));
    }

    Ok(())
}

/// Validates that every non-enum OWL class has a `pub trait` declaration.
fn validate_trait_completeness(
    src_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    // Classes represented as enums — skip trait check for these
    let enum_classes = uor_ontology::Ontology::enum_class_names();

    // Read all generated source files
    let all_source = read_all_rs_files(src_dir)?;

    let mut missing = Vec::new();
    let mut found = 0usize;

    for module in &ontology.namespaces {
        for class in &module.classes {
            let local = uor_codegen::mapping::local_name(class.id);

            // Skip enum classes
            if enum_classes.contains(&local) {
                continue;
            }

            let pattern = format!("pub trait {local}");
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
            format!("All {found} class traits present in generated source"),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "{} classes missing trait declarations ({found} found)",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(())
}

/// Validates that every property with a domain has a corresponding method.
fn validate_method_completeness(
    src_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    let all_source = read_all_rs_files(src_dir)?;

    // Classes represented as enums — skip method check for properties on these
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

            // Skip cross-namespace domain properties (domain class in different
            // namespace than the property — not generated by the codegen)
            if let Some(domain) = prop.domain {
                if !domain.starts_with(ns_iri) {
                    continue;
                }
                // Skip properties whose domain is an enum class
                let domain_local = uor_codegen::mapping::local_name(domain);
                if enum_domain_classes.contains(&domain_local) {
                    continue;
                }
            }

            let method =
                uor_codegen::mapping::to_snake_case(uor_codegen::mapping::local_name(prop.id));

            // The method may appear as fn method_name or fn method_name_count
            let pattern_fn = format!("fn {method}(");
            let pattern_count = format!("fn {method}_count(");
            if all_source.contains(&pattern_fn) || all_source.contains(&pattern_count) {
                found += 1;
            } else {
                missing.push(format!("{} ({})", prop.id, method));
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {found} property methods present in generated source"),
        ));
        // Amendment 47: cross-check total against centralized count
        if found != uor_ontology::counts::METHODS {
            report.push_meta(TestResult::fail(
                VALIDATOR,
                format!(
                    "Method count drift: found {} methods but counts::METHODS = {}",
                    found,
                    uor_ontology::counts::METHODS
                ),
            ));
        }
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "{} properties missing methods ({found} found)",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(())
}

/// Validates that every named individual has a const module or enum variant.
fn validate_individual_completeness(
    src_dir: &Path,
    ontology: &uor_ontology::Ontology,
    report: &mut ConformanceReport,
) -> Result<()> {
    let all_source = read_all_rs_files(src_dir)?;

    // Individual types that map to enum variants (not const modules).
    // Includes both OWL enum classes and PrimitiveOp variant types.
    let ontology_enums = uor_ontology::Ontology::enum_class_names();
    let primitive_op_types: &[&str] = &["UnaryOp", "BinaryOp", "Involution"];
    let enum_types: Vec<&str> = primitive_op_types
        .iter()
        .chain(ontology_enums.iter())
        .copied()
        .collect();

    let mut missing = Vec::new();
    let mut found = 0usize;

    for module in &ontology.namespaces {
        for ind in &module.individuals {
            let local = uor_codegen::mapping::local_name(ind.id);
            let type_local = uor_codegen::mapping::local_name(ind.type_);

            if enum_types.contains(&type_local) {
                // Should exist as an enum variant
                let mut variant = capitalize_first(local);
                // The codegen strips common PascalCase-word suffixes to avoid
                // clippy::enum_variant_names (e.g., MetricAxis strips "Axis",
                // ComplexityClass strips "Time", RewriteRule strips "Rule",
                // TriadProjection strips "Projection")
                let suffix = match type_local {
                    "MetricAxis" => Some("Axis"),
                    "ComplexityClass" => Some("Time"),
                    "RewriteRule" => Some("Rule"),
                    "PhaseBoundaryType" => Some("Boundary"),
                    _ => None,
                };
                if let Some(sfx) = suffix {
                    if variant.ends_with(sfx) && variant.len() > sfx.len() {
                        variant.truncate(variant.len() - sfx.len());
                    }
                }
                if all_source.contains(&variant) {
                    found += 1;
                } else {
                    missing.push(format!("{} (enum variant {})", ind.id, variant));
                }
            } else {
                // Should exist as a const module
                let mod_name = uor_codegen::mapping::to_snake_case(local);
                let pattern = format!("pub mod {mod_name}");
                if all_source.contains(&pattern) {
                    found += 1;
                } else {
                    missing.push(format!("{} (mod {})", ind.id, mod_name));
                }
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!("All {found} individuals present in generated source"),
        ));
        // Amendment 47: cross-check total against centralized count
        if found != uor_ontology::counts::INDIVIDUALS {
            report.push_meta(TestResult::fail(
                VALIDATOR,
                format!(
                    "Individual count drift: found {} but counts::INDIVIDUALS = {}",
                    found,
                    uor_ontology::counts::INDIVIDUALS
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

/// Validates that the `HostTypes` trait exists in lib.rs.
fn validate_host_types_trait(src_dir: &Path, report: &mut ConformanceReport) -> Result<()> {
    let lib_rs = std::fs::read_to_string(src_dir.join("lib.rs"))
        .with_context(|| "Failed to read foundation/src/lib.rs")?;

    if lib_rs.contains("pub trait HostTypes") {
        report.push(TestResult::pass(
            VALIDATOR,
            "HostTypes type family trait present in lib.rs",
        ));
    } else {
        report.push(TestResult::fail(
            VALIDATOR,
            "HostTypes type family trait not found in lib.rs",
        ));
    }

    // Check expected associated types (3 slots: Decimal, HostString, WitnessBytes)
    let expected_types = ["Decimal", "HostString", "WitnessBytes"];
    let mut all_types = true;
    for t in &expected_types {
        let pattern = format!("type {t}");
        if !lib_rs.contains(&pattern) {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("HostTypes trait missing associated type: {t}"),
            ));
            all_types = false;
        }
    }
    if all_types {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "HostTypes trait has all {} expected associated types",
                expected_types.len()
            ),
        ));
    }

    Ok(())
}

/// Reads all `.rs` files in a directory tree and concatenates their contents.
fn read_all_rs_files(dir: &Path) -> Result<String> {
    let mut content = String::new();
    visit_rs_files(dir, &mut content)?;
    Ok(content)
}

/// Recursively visits all `.rs` files and appends their content.
fn visit_rs_files(dir: &Path, buf: &mut String) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, buf)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let file_content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read: {}", path.display()))?;
            buf.push_str(&file_content);
            buf.push('\n');
        }
    }
    Ok(())
}

/// Capitalizes the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result = c.to_uppercase().to_string();
            result.push_str(chars.as_str());
            result
        }
    }
}
