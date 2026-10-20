/// SHACL test 56: Domain grounding record — DomainGroundingRecord with
/// groundedContext, groundedDomain, domainFreeRank (Amendment 33).
pub const TEST56_DOMAIN_GROUNDING_RECORD: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix state:      <https://uor.foundation/state/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. DomainGroundingRecord
observable:ex_dsr_56 a owl:NamedIndividual, observable:DomainGroundingRecord ;
    observable:groundedContext state:ex_sc_56 ;
    observable:groundedDomain "arithmetic"^^xsd:string ;
    observable:domainFreeRank "0"^^xsd:integer .

# 2. Referenced GroundedContext
state:ex_sc_56 a owl:NamedIndividual, state:GroundedContext .
"#;
