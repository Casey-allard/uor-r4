/// SHACL test 96: GeodesicTrace at Q1 ring scale with GeodesicCertificate
/// (Amendment 40).
pub const TEST96_GEODESIC_TRACE_W16: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix trace: <https://uor.foundation/trace/> .
@prefix cert:  <https://uor.foundation/cert/> .

trace:ex_gt_96 a owl:NamedIndividual, trace:GeodesicTrace ;
    trace:isGeodesic          "true"^^xsd:boolean ;
    trace:isAR1Ordered        "true"^^xsd:boolean ;
    trace:isDC10Selected      "true"^^xsd:boolean ;
    trace:geodesicCertificate cert:ex_gc_96 .

cert:ex_gc_96 a owl:NamedIndividual, cert:GeodesicCertificate ;
    cert:certifiedGeodesic <https://uor.foundation/trace/geodesic_Q1> .
"#;
