//! `meta/required_property_coverage` — fast drift guard.
//!
//! For every `Property` marked `required: true`, verify that every
//! named individual whose `rdf:type` is the property's domain (or a
//! transitive subclass of the domain) has at least one assertion of
//! that property. Runs in memory against `Ontology::full()` and
//! completes in milliseconds — the fastest possible feedback loop
//! for ontology-layer drift.
//!
//! Layered with `lean4/individual_proof`: this validator catches
//! drift at the ontology layer BEFORE the Lean codegen runs, while
//! `individual_proof` catches anything this validator misses by
//! actually type-checking the struct literals. Both checks pass →
//! conformance passes.

use anyhow::Result;
use uor_ontology::model::{Class, Ontology};

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "meta/required_property_coverage";

/// Validates that every required property is asserted on every
/// individual whose class matches its domain.
///
/// # Errors
///
/// Returns `Ok` on both success and failure — the failure mode is
/// encoded as a `TestResult::Failure` in the report.
pub fn validate(ontology: &Ontology) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // Build an index: class IRI -> Class (for subclass walks).
    let class_by_iri: std::collections::HashMap<&str, &Class> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .map(|c| (c.id, c))
        .collect();

    // For each required property, gather its domain + domain's
    // subclasses, then check every matching individual for an assertion.
    let mut missing: Vec<String> = Vec::new();
    let mut required_count = 0usize;
    let mut checked_assertions = 0usize;

    for module in &ontology.namespaces {
        for prop in &module.properties {
            if !prop.required {
                continue;
            }
            required_count += 1;
            let Some(domain_iri) = prop.domain else {
                // Required without a domain is meaningless — flag and skip.
                missing.push(format!(
                    "{} (property has `required: true` but no domain)",
                    prop.id
                ));
                continue;
            };
            let domain_closure = subclass_closure(domain_iri, &class_by_iri);

            for m2 in &ontology.namespaces {
                for ind in &m2.individuals {
                    if !domain_closure.contains(ind.type_) {
                        continue;
                    }
                    checked_assertions += 1;
                    let has_assertion = ind.properties.iter().any(|(p, _)| *p == prop.id);
                    if !has_assertion {
                        missing.push(format!(
                            "{} :: {} (required by domain <{}>)",
                            ind.id, prop.label, domain_iri
                        ));
                    }
                }
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "All {} required properties asserted on every matching individual \
                 ({} instance-property checks)",
                required_count, checked_assertions
            ),
        ));
    } else {
        report.push(TestResult::fail_with_details(
            VALIDATOR,
            format!(
                "{} required-property assertions missing across the ontology",
                missing.len()
            ),
            missing,
        ));
    }

    Ok(report)
}

/// Returns the set of IRIs for the given domain class and every
/// transitive subclass of it. Used to determine which individuals
/// (via their `rdf:type`) are subject to the required-property
/// assertion requirement.
fn subclass_closure<'a>(
    root: &'a str,
    class_by_iri: &std::collections::HashMap<&'a str, &'a Class>,
) -> std::collections::HashSet<&'a str> {
    let mut result: std::collections::HashSet<&'a str> = std::collections::HashSet::new();
    result.insert(root);
    let mut changed = true;
    while changed {
        changed = false;
        for (child_iri, child) in class_by_iri {
            if result.contains(child_iri) {
                continue;
            }
            if child.subclass_of.iter().any(|p| result.contains(p)) {
                result.insert(child_iri);
                changed = true;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::expect_used)]
    fn full_ontology_has_no_required_drift() {
        let ontology = uor_ontology::Ontology::full();
        let report = validate(ontology).expect("validator never errors");
        // Assert the primary check passes. Any failure prints details
        // so the test output surfaces the specific gaps.
        for result in &report.results {
            assert!(
                !result.is_failure(),
                "required_property_coverage check failed: {} — details: {:?}",
                result.message,
                result.details
            );
        }
    }
}
