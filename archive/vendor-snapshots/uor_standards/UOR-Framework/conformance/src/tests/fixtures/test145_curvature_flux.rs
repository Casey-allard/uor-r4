/// SHACL fixture for observable:CurvatureFlux.
pub const TEST145_CURVATURE_FLUX: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:curvature_flux_1> a owl:NamedIndividual , observable:CurvatureFlux ;
    observable:value "0.25"^^xsd:decimal .
"#;
