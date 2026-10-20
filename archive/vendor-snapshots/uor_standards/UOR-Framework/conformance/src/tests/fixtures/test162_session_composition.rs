/// SHACL fixture for state:SessionComposition — Amendment 48.
pub const TEST162_SESSION_COMPOSITION: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix state: <https://uor.foundation/state/> .

<urn:test:comp_162> a owl:NamedIndividual , state:SessionComposition ;
    state:composedFrom <urn:test:session_a_162> ;
    state:composedFrom <urn:test:session_b_162> ;
    state:compositionCompatible "true"^^xsd:boolean ;
    state:compositionResult <urn:test:merged_ctx_162> ;
    state:towerConsistencyVerified "true"^^xsd:boolean .

<urn:test:session_a_162>  a owl:NamedIndividual , state:Session .
<urn:test:session_b_162>  a owl:NamedIndividual , state:Session .
<urn:test:merged_ctx_162> a owl:NamedIndividual , state:Context .
"#;
