/// SHACL test 86: WittLift with trivial LiftObstruction — W16 lift (Amendment 39).
pub const TEST86_WITT_LIFT_TRIVIAL: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix schema:   <https://uor.foundation/schema/> .
@prefix resolver: <https://uor.foundation/resolver/> .

type:ex_base_86 a owl:NamedIndividual, type:CompleteType .

type:ex_obstruction_86 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionTrivial "true"^^xsd:boolean .

type:ex_lift_86 a owl:NamedIndividual, type:WittLift ;
    type:liftBase        type:ex_base_86 ;
    type:liftTargetLevel schema:W16 ;
    type:liftObstruction type:ex_obstruction_86 .

resolver:ex_icr_86 a owl:NamedIndividual, resolver:IncrementalCompletenessResolver ;
    resolver:liftTarget type:ex_lift_86 .
"#;
