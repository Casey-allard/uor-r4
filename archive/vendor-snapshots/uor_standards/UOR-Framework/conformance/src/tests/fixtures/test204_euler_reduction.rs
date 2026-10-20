//! SHACL test 204: `reduction:EulerReduction` instance.

/// Instance graph for Test 204: EulerReduction with phaseParameter and stageCount.
pub const TEST204_EULER_REDUCTION: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:euler_reduction_instance a owl:NamedIndividual, reduction:EulerReduction ;
    reduction:phaseParameter "e^{i*pi/6}" ;
    reduction:stageCount "6"^^xsd:nonNegativeInteger ;
    reduction:convergenceAngle "pi" .
"#;
