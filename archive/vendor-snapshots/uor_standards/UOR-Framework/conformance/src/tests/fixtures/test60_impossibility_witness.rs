/// SHACL test 60: Impossibility witness — ImpossibilityWitness with
/// forbidsSignature, impossibilityReason, impossibilityDomain (Amendment 34).
pub const TEST60_IMPOSSIBILITY_WITNESS: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix proof:      <https://uor.foundation/proof/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. ImpossibilityWitness
proof:ex_iw_60 a owl:NamedIndividual, proof:ImpossibilityWitness ;
    proof:forbidsSignature observable:ex_sig_60 ;
    proof:impossibilityReason "Violates prime factorization uniqueness"^^xsd:string ;
    proof:impossibilityDomain "arithmetic"^^xsd:string .

# 2. Referenced signature
observable:ex_sig_60 a owl:NamedIndividual, observable:SynthesisSignature .
"#;
