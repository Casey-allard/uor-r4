/// SHACL fixture for state:SharedContext and state:ContextLease — Amendment 48.
pub const TEST160_SHARED_CONTEXT: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix state:     <https://uor.foundation/state/> .
@prefix partition: <https://uor.foundation/partition/> .

<urn:test:shared_ctx_160> a owl:NamedIndividual , state:SharedContext ;
    state:leaseSet <urn:test:lease_a_160> ;
    state:leaseSet <urn:test:lease_b_160> .

<urn:test:lease_a_160> a owl:NamedIndividual , state:ContextLease ;
    state:leaseHolder <urn:test:session_a_160> ;
    state:leasedSites <urn:test:budget_a_160> .

<urn:test:lease_b_160> a owl:NamedIndividual , state:ContextLease ;
    state:leaseHolder <urn:test:session_b_160> ;
    state:leasedSites <urn:test:budget_b_160> .

<urn:test:session_a_160> a owl:NamedIndividual , state:Session .
<urn:test:session_b_160> a owl:NamedIndividual , state:Session .
<urn:test:budget_a_160>  a owl:NamedIndividual , partition:FreeRank .
<urn:test:budget_b_160>  a owl:NamedIndividual , partition:FreeRank .
"#;
