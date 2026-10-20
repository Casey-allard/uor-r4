/// SHACL test 34: Completeness Certification Pathway — Amendment 25.
pub const TEST34_COMPLETENESS_CANDIDATE: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix resolver: <https://uor.foundation/resolver/> .

resolver:ex_compl_resolver_34 a owl:NamedIndividual, resolver:CompletenessResolver ;
    resolver:completenessTarget type:ex_candidate_34 .

type:ex_candidate_34 a owl:NamedIndividual, type:CompletenessCandidate ;
    type:completenessCandidate type:ex_constrained_34 ;
    type:candidateNerve resolver:ex_nerve_34 .

type:ex_constrained_34 a owl:NamedIndividual, type:ConstrainedType .
resolver:ex_nerve_34 a owl:NamedIndividual, resolver:CechNerve .

type:ex_witness_34 a owl:NamedIndividual, type:CompletenessWitness ;
    type:witnessConstraint type:ex_constr_34 ;
    type:sitesClosed "3"^^xsd:nonNegativeInteger .

type:ex_constr_34 a owl:NamedIndividual, type:Constraint .
"#;
