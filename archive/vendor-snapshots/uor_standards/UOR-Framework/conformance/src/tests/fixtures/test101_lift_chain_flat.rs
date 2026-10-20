/// SHACL test 101: Flat LiftChain Q0→Q3 — a complete lift chain traversing
/// all quantum levels with trivial (flat) holonomy at each stage.
pub const TEST101_LIFT_CHAIN_FLAT: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix op:     <https://uor.foundation/op/> .
@prefix resolver: <https://uor.foundation/resolver/> .

# Flat LiftChain from Q0 to Q3 with no obstructions
type:ex_lc_flat_101 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel   schema:Q0 ;
    type:targetLevel   schema:Q3 ;
    type:liftedType    type:ex_flat_type_101 ;
    type:isFlat        "true"^^xsd:boolean .

type:ex_flat_type_101 a owl:NamedIndividual, type:FlatType ;
    rdfs:label "FlatType at Q3" .

resolver:ex_resolver_flat_101 a owl:NamedIndividual, resolver:IncrementalCompletenessResolver ;
    resolver:resolvedLift type:ex_lc_flat_101 .
"#;
