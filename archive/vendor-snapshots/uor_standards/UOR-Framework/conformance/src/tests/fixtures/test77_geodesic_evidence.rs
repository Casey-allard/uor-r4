/// SHACL test 77: GeodesicEvidenceBundle linked from GeodesicCertificate (Amendment 37, Gap 9).
pub const TEST77_GEODESIC_EVIDENCE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_geb_77 a owl:NamedIndividual, cert:GeodesicEvidenceBundle .
cert:ex_gc_77 a owl:NamedIndividual, cert:GeodesicCertificate ;
    cert:certifiedGeodesic <https://uor.foundation/trace/geodesic_Q0> ;
    cert:evidenceBundle cert:ex_geb_77 .
"#;
