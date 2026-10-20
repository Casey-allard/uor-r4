/// SHACL fixture for distributed grounding (MC_6/MC_7) — Amendment 48.
pub const TEST163_DISTRIBUTED_GROUNDING: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix xsd:       <http://www.w3.org/2001/XMLSchema#> .
@prefix state:     <https://uor.foundation/state/> .
@prefix partition: <https://uor.foundation/partition/> .

<urn:test:shared_ctx_163> a owl:NamedIndividual , state:SharedContext ;
    state:leaseSet <urn:test:lease_a_163> ;
    state:leaseSet <urn:test:lease_b_163> .

<urn:test:lease_a_163> a owl:NamedIndividual , state:ContextLease ;
    state:leaseHolder <urn:test:session_a_163> ;
    state:leasedSites <urn:test:budget_a_163> .

<urn:test:lease_b_163> a owl:NamedIndividual , state:ContextLease ;
    state:leaseHolder <urn:test:session_b_163> ;
    state:leasedSites <urn:test:budget_b_163> .

<urn:test:session_a_163> a owl:NamedIndividual , state:Session .
<urn:test:session_b_163> a owl:NamedIndividual , state:Session .
<urn:test:budget_a_163>  a owl:NamedIndividual , partition:FreeRank .
<urn:test:budget_b_163>  a owl:NamedIndividual , partition:FreeRank .

<urn:test:comp_163> a owl:NamedIndividual , state:SessionComposition ;
    state:composedFrom <urn:test:session_a_163> ;
    state:composedFrom <urn:test:session_b_163> ;
    state:compositionCompatible "true"^^xsd:boolean ;
    state:compositionResult <urn:test:saturated_163> ;
    state:towerConsistencyVerified "true"^^xsd:boolean .

<urn:test:saturated_163> a owl:NamedIndividual , state:GroundedContext ;
    state:isGrounded "true"^^xsd:boolean ;
    state:residualFreeRank "0"^^xsd:nonNegativeInteger .
"#;
