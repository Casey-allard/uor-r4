/// SHACL test 65: Geodesic trace — GeodesicTrace with isGeodesic,
/// geodesicCertificate, stepEntropyCost, cumulativeEntropyCost (Amendment 35).
pub const TEST65_GEODESIC_TRACE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix cert:       <https://uor.foundation/cert/> .

# 1. GeodesicTrace with properties
trace:ex_gt_65 a owl:NamedIndividual, trace:GeodesicTrace ;
    trace:isGeodesic "true"^^xsd:boolean ;
    trace:geodesicCertificate cert:ex_gc_65 ;
    trace:stepEntropyCost "0.5"^^xsd:decimal ;
    trace:cumulativeEntropyCost "2.5"^^xsd:decimal .

# 2. Referenced certificate
cert:ex_gc_65 a owl:NamedIndividual, cert:GeodesicCertificate .
"#;
