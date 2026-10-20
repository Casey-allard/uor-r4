//! SHACL test 268: `ViolationKind` enum individuals.

/// Instance graph for Test 268: All 5 ViolationKind individuals.
pub const TEST268_VIOLATION_KIND: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix conformance: <https://uor.foundation/conformance/> .

conformance:Missing a owl:NamedIndividual, conformance:ViolationKind .
conformance:TypeMismatch a owl:NamedIndividual, conformance:ViolationKind .
conformance:CardinalityViolation a owl:NamedIndividual, conformance:ViolationKind .
conformance:ValueCheck a owl:NamedIndividual, conformance:ViolationKind .
conformance:LevelMismatch a owl:NamedIndividual, conformance:ViolationKind .
"#;
