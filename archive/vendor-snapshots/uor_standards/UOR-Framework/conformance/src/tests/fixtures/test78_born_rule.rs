/// SHACL test 78: BornRuleVerification certificate (Amendment 37, Gap 10).
pub const TEST78_BORN_RULE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

cert:ex_brv_78 a owl:NamedIndividual, cert:BornRuleVerification ;
    cert:bornRuleVerified "true"^^xsd:boolean .
"#;
