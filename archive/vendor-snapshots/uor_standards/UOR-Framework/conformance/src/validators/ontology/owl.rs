//! OWL 2 DL validator.
//!
//! Validates OWL 2 DL constraints on the UOR Foundation ontology:
//! - Subclass targets must be known classes
//! - Domain/range targets must be known classes or datatypes
//! - Disjoint-with targets must be known classes
//! - Individual type assertions must reference known classes
//! - No circular imports between namespaces

use crate::report::{ConformanceReport, TestResult};

/// Validates OWL 2 DL constraints on the live spec ontology.
///
/// # Errors
///
/// Returns an error only if an unexpected internal error occurs (never on constraint violations).
pub fn validate() -> ConformanceReport {
    let mut report = ConformanceReport::new();
    let ontology = uor_ontology::Ontology::full();

    // Collect all known class IRIs
    let mut known_classes: std::collections::HashSet<&'static str> =
        std::collections::HashSet::new();
    for module in &ontology.namespaces {
        for class in &module.classes {
            known_classes.insert(class.id);
        }
    }

    // Collect all known property IRIs
    let mut known_properties: std::collections::HashSet<&'static str> =
        std::collections::HashSet::new();
    for module in &ontology.namespaces {
        for prop in &module.properties {
            known_properties.insert(prop.id);
        }
    }

    // Collect all known individual IRIs
    let mut known_individuals: std::collections::HashSet<&'static str> =
        std::collections::HashSet::new();
    for module in &ontology.namespaces {
        for ind in &module.individuals {
            known_individuals.insert(ind.id);
        }
    }

    // Known XSD datatypes and OWL primitives that are valid range/domain targets.
    // Include both prefixed forms (xsd:) and full IRIs for spec compatibility.
    let known_datatypes: std::collections::HashSet<&'static str> = [
        // Prefixed forms
        "xsd:string",
        "xsd:integer",
        "xsd:boolean",
        "xsd:anyURI",
        "xsd:nonNegativeInteger",
        "xsd:positiveInteger",
        "xsd:dateTimeStamp",
        "xsd:dateTime",
        "xsd:float",
        "xsd:double",
        "owl:Thing",
        "owl:Class",
        "rdf:List",
        "rdfs:Literal",
        // Full IRI forms
        "http://www.w3.org/2001/XMLSchema#string",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#boolean",
        "http://www.w3.org/2001/XMLSchema#anyURI",
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
        "http://www.w3.org/2001/XMLSchema#positiveInteger",
        "http://www.w3.org/2001/XMLSchema#dateTimeStamp",
        "http://www.w3.org/2001/XMLSchema#dateTime",
        "http://www.w3.org/2001/XMLSchema#float",
        "http://www.w3.org/2001/XMLSchema#double",
        "http://www.w3.org/2001/XMLSchema#decimal",
        "xsd:decimal",
        "http://www.w3.org/2001/XMLSchema#hexBinary",
        "xsd:hexBinary",
        "http://www.w3.org/2002/07/owl#Thing",
        "http://www.w3.org/2002/07/owl#Class",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#List",
        "http://www.w3.org/2000/01/rdf-schema#Literal",
    ]
    .into();

    let mut violations: Vec<String> = Vec::new();

    // Check subclass targets
    for module in &ontology.namespaces {
        for class in &module.classes {
            for parent in class.subclass_of {
                if !known_classes.contains(parent) && !known_datatypes.contains(parent) {
                    violations.push(format!(
                        "Class {} has unknown subClassOf target: {}",
                        class.id, parent
                    ));
                }
            }
            for disjoint in class.disjoint_with {
                if !known_classes.contains(disjoint) {
                    violations.push(format!(
                        "Class {} has unknown disjointWith target: {}",
                        class.id, disjoint
                    ));
                }
            }
        }
    }

    // Check property domain and range targets
    for module in &ontology.namespaces {
        for prop in &module.properties {
            if let Some(domain) = prop.domain {
                if !known_classes.contains(domain) && !known_datatypes.contains(domain) {
                    violations.push(format!(
                        "Property {} has unknown domain: {}",
                        prop.id, domain
                    ));
                }
            }
            if !known_classes.contains(prop.range)
                && !known_datatypes.contains(prop.range)
                && !prop.range.is_empty()
            {
                violations.push(format!(
                    "Property {} has unknown range: {}",
                    prop.id, prop.range
                ));
            }
        }
    }

    // Check individual type assertions
    for module in &ontology.namespaces {
        for ind in &module.individuals {
            if !known_classes.contains(ind.type_) && !known_datatypes.contains(ind.type_) {
                violations.push(format!(
                    "Individual {} has unknown type: {}",
                    ind.id, ind.type_
                ));
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            "ontology/owl",
            "All OWL 2 DL structural constraints satisfied",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "ontology/owl",
            "OWL 2 DL constraint violations detected",
            violations,
        ));
    }

    // Check for circular imports (simple: namespace A imports B, B imports A)
    check_circular_imports(ontology, &mut report);

    report
}

/// Checks for circular imports between namespace modules.
fn check_circular_imports(ontology: &uor_ontology::Ontology, report: &mut ConformanceReport) {
    let mut circular: Vec<String> = Vec::new();

    // Build import map
    let import_map: std::collections::HashMap<&str, &[&str]> = ontology
        .namespaces
        .iter()
        .map(|m| (m.namespace.iri, m.namespace.imports))
        .collect();

    for module in &ontology.namespaces {
        let iri = module.namespace.iri;
        for &imported in module.namespace.imports {
            // Check if the imported namespace also imports this one (direct circular)
            if let Some(&imported_imports) = import_map.get(imported) {
                if imported_imports.contains(&iri) {
                    circular.push(format!("{} <-> {} (circular import)", iri, imported));
                }
            }
        }
    }

    if circular.is_empty() {
        report.push(TestResult::pass(
            "ontology/owl",
            "No circular imports between namespaces",
        ));
    } else {
        report.push(TestResult::fail_with_details(
            "ontology/owl",
            "Circular imports detected between namespaces",
            circular,
        ));
    }
}
