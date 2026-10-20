/// SHACL test 84: VerificationDomain individual with enumVariant annotation (Amendment 37, Gap 11).
pub const TEST84_ENUM_VARIANT: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix op:  <https://uor.foundation/op/> .

op:ex_vd_84 a owl:NamedIndividual, op:VerificationDomain ;
    op:enumVariant "Algebraic"^^xsd:string .
"#;
