/// SHACL test 99: MeasurementCertificate with BornRuleVerification at Q1 — 16
/// sites (Amendment 40). Validates QM_1 and QM_5 at Q1 site count.
pub const TEST99_MEASUREMENT_BORN_W16: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_mc_99 a owl:NamedIndividual, cert:MeasurementCertificate ;
    cert:vonNeumannEntropy "4.0"^^xsd:decimal ;
    cert:landauerCost      "2.773"^^xsd:decimal .

cert:ex_brv_99 a owl:NamedIndividual, cert:BornRuleVerification ;
    cert:bornRuleVerified "true"^^xsd:boolean .
"#;
