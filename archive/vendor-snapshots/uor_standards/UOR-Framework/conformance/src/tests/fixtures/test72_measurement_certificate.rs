/// SHACL test 72: Measurement certificate — MeasurementCertificate with
/// certifiedMeasurement, vonNeumannEntropy, landauerCost (Amendment 36).
pub const TEST72_MEASUREMENT_CERTIFICATE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix resolver:   <https://uor.foundation/resolver/> .

# 1. MeasurementCertificate
cert:ex_mc_72 a owl:NamedIndividual, cert:MeasurementCertificate ;
    cert:certifiedMeasurement resolver:ex_mr_72 ;
    cert:vonNeumannEntropy "2.1"^^xsd:decimal ;
    cert:landauerCost "0.693"^^xsd:decimal .

# 2. Referenced MeasurementResolver
resolver:ex_mr_72 a owl:NamedIndividual, resolver:MeasurementResolver .
"#;
