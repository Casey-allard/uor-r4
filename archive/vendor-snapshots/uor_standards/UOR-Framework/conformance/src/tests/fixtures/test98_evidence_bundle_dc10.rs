/// SHACL test 98: GeodesicEvidenceBundle with isDC10Selected — exercises new
/// Amendment 38 property (Amendment 40).
pub const TEST98_EVIDENCE_BUNDLE_DC10: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_geb_98 a owl:NamedIndividual, cert:GeodesicEvidenceBundle ;
    cert:isDC10Selected "true"^^xsd:boolean .
"#;
