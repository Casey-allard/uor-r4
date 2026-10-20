/// SHACL test 97: GeodesicEvidenceBundle with isAR1Ordered — exercises new
/// Amendment 38 property (Amendment 40).
pub const TEST97_EVIDENCE_BUNDLE_AR1: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_geb_97 a owl:NamedIndividual, cert:GeodesicEvidenceBundle ;
    cert:isAR1Ordered "true"^^xsd:boolean .
"#;
