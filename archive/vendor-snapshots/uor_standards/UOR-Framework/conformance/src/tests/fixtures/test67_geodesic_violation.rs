/// SHACL test 67: Geodesic violation — GeodesicViolation with violationReason
/// (Amendment 35).
pub const TEST67_GEODESIC_VIOLATION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. GeodesicViolation
observable:ex_gv_67 a owl:NamedIndividual, observable:GeodesicViolation ;
    observable:violationReason "Step entropy exceeds Landauer bound"^^xsd:string .
"#;
