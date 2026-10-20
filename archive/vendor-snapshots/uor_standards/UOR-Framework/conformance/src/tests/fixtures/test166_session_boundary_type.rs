//! SHACL test 166: `state:SessionBoundaryType` vocabulary — Amendment 27.
//!
//! `SessionBoundaryType` is a typed controlled vocabulary for session boundary
//! reasons. The three named individuals are `ExplicitReset`, `ConvergenceBoundary`,
//! and `ContradictionBoundary`.

/// Instance graph for Test 166: state:SessionBoundaryType.
pub const TEST166_SESSION_BOUNDARY_TYPE: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix state: <https://uor.foundation/state/> .

state:ExplicitReset
    a owl:NamedIndividual, state:SessionBoundaryType .
"#;
