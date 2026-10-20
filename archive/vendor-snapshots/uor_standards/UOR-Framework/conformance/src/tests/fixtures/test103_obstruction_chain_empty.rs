/// SHACL test 103: Empty ObstructionChain — a quantum lift with no
/// obstructions, confirming the chain is vacuously satisfied.
pub const TEST103_OBSTRUCTION_CHAIN_EMPTY: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

# WittLift with empty obstruction chain
type:ex_lift_empty_103 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel   schema:Q0 ;
    type:targetLevel   schema:Q1 ;
    type:liftedType    type:ex_flat_103 ;
    type:isFlat        "true"^^xsd:boolean .

type:ex_flat_103 a owl:NamedIndividual, type:FlatType ;
    rdfs:label "Unobstructed flat type" .

# LiftObstructionClass with trivial class (no non-trivial element)
observable:ex_oclass_empty_103 a owl:NamedIndividual, observable:LiftObstructionClass ;
    rdfs:label "Trivial obstruction class" .
"#;
