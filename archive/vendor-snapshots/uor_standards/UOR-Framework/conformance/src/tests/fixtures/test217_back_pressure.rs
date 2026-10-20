//! SHACL test 217: `reduction:BackPressureSignal` instance.

/// Instance graph for Test 217: BackPressureSignal with pressure level.
pub const TEST217_BACK_PRESSURE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:bp_example a owl:NamedIndividual, reduction:BackPressureSignal ;
    reduction:pressureLevel "High"^^xsd:string ;
    reduction:pressureThreshold "0.9"^^xsd:string .
"#;
