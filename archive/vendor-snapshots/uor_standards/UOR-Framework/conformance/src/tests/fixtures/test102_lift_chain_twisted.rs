/// SHACL test 102: Twisted LiftChain Q0→Q2 with non-trivial obstruction —
/// the lift encounters a twist at Q1 producing a LiftObstruction.
pub const TEST102_LIFT_CHAIN_TWISTED: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

# Twisted LiftChain Q0→Q2 with obstruction at Q1
type:ex_lc_twisted_102 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel    schema:Q0 ;
    type:targetLevel    schema:Q2 ;
    type:liftedType     type:ex_twisted_type_102 ;
    type:isFlat         "false"^^xsd:boolean ;
    type:liftObstruction type:ex_obstruction_102 .

type:ex_twisted_type_102 a owl:NamedIndividual, type:TwistedType ;
    rdfs:label "TwistedType at Q2" .

type:ex_obstruction_102 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionLevel schema:Q1 ;
    rdfs:label "Non-trivial obstruction at Q1" .
"#;
