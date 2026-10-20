/// SHACL test 107: TowerCompletenessResolver — an incremental completeness
/// resolver operating across the full quantum level tower Q0→Q3.
pub const TEST107_TOWER_RESOLVER: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .
@prefix resolver: <https://uor.foundation/resolver/> .

# Tower resolver spanning all quantum levels
resolver:ex_tower_107 a owl:NamedIndividual, resolver:IncrementalCompletenessResolver ;
    rdfs:label "Tower completeness resolver Q0-Q3" .

# Lift refinement suggestion from the tower resolver
resolver:ex_suggestion_107 a owl:NamedIndividual, resolver:LiftRefinementSuggestion ;
    rdfs:label "Refinement at Q2 boundary" .

# WittLift resolved by the tower
type:ex_lift_107 a owl:NamedIndividual, type:WittLift ;
    type:sourceLevel   schema:Q0 ;
    type:targetLevel   schema:Q3 ;
    type:isFlat        "true"^^xsd:boolean .
"#;
