/// SHACL test 38: Session lifecycle with BindingAccumulator and SessionQuery — Amendment 27.
pub const TEST38_SESSION_LIFECYCLE: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix state:    <https://uor.foundation/state/> .
@prefix resolver: <https://uor.foundation/resolver/> .
@prefix query:    <https://uor.foundation/query/> .

state:ex_session_38 a owl:NamedIndividual, state:Session ;
    state:sessionBindings  state:ex_context_38 ;
    state:sessionQueries   "3"^^xsd:nonNegativeInteger .

state:ex_accumulator_38 a owl:NamedIndividual, state:BindingAccumulator ;
    state:accumulatedBindings state:ex_binding1_38 ;
    state:accumulatedBindings state:ex_binding2_38 .

resolver:ex_session_resolver_38 a owl:NamedIndividual, resolver:SessionResolver ;
    resolver:sessionAccumulator state:ex_accumulator_38 .

query:ex_session_query_38 a owl:NamedIndividual, query:SessionQuery ;
    query:sessionMembership state:ex_session_38 .

state:ex_context_38  a owl:NamedIndividual, state:Context .
state:ex_binding1_38 a owl:NamedIndividual, state:Binding .
state:ex_binding2_38 a owl:NamedIndividual, state:Binding .
"#;
