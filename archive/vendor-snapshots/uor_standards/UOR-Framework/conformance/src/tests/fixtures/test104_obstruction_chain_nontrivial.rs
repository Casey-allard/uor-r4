/// SHACL test 104: Non-trivial ObstructionChain — a lift with genuine
/// obstructions classified by a non-trivial LiftObstructionClass.
pub const TEST104_OBSTRUCTION_CHAIN_NONTRIVIAL: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

# Non-trivial obstruction at Q2
type:ex_obstruction_104 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionLevel schema:Q2 ;
    rdfs:label "Cohomological obstruction at Q2" .

# LiftObstructionClass classifying the obstruction
observable:ex_oclass_104 a owl:NamedIndividual, observable:LiftObstructionClass ;
    observable:obstructionElement type:ex_obstruction_104 ;
    rdfs:label "Non-trivial cohomological class" .

# The blocked lift
type:ex_lift_blocked_104 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel      schema:Q0 ;
    type:targetLevel      schema:Q3 ;
    type:isFlat           "false"^^xsd:boolean ;
    type:liftObstruction  type:ex_obstruction_104 .
"#;
