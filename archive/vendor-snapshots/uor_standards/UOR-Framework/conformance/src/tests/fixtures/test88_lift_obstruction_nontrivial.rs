/// SHACL test 88: Non-trivial LiftObstruction with TwistedType and holonomyClassified
/// — Q1 scale (Amendment 39). Validates QLS_2 + MN_8.
pub const TEST88_LIFT_OBSTRUCTION_NONTRIVIAL: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix schema:     <https://uor.foundation/schema/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix partition:  <https://uor.foundation/partition/> .

type:ex_twisted_88 a owl:NamedIndividual, type:TwistedType ;
    type:holonomyClassified "true"^^xsd:boolean ;
    type:holonomyGroup      observable:ex_holonomy_88 .

observable:ex_holonomy_88 a owl:NamedIndividual, observable:HolonomyGroup ;
    observable:holonomyGroupOrder "4"^^xsd:positiveInteger .

type:ex_obstruction_88 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionTrivial "false"^^xsd:boolean ;
    type:obstructionSite   partition:ex_site_88 .

partition:ex_site_88 a owl:NamedIndividual, partition:SiteIndex .

type:ex_lift_88 a owl:NamedIndividual, type:WittLift ;
    type:liftBase        type:ex_twisted_88 ;
    type:liftTargetLevel schema:Q1 ;
    type:liftObstruction type:ex_obstruction_88 .
"#;
