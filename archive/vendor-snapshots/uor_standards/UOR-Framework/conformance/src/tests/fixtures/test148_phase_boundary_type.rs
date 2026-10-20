/// SHACL fixture for observable:PhaseBoundaryType (enum class).
pub const TEST148_PHASE_BOUNDARY_TYPE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:PeriodBoundary a owl:NamedIndividual , observable:PhaseBoundaryType .
observable:PowerOfTwoBoundary a owl:NamedIndividual , observable:PhaseBoundaryType .
"#;
