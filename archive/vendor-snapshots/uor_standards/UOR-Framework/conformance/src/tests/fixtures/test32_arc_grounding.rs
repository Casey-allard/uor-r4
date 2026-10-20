/// SHACL test 32: ARC grounding — GroundingMap, ProjectionMap, and RelationQuery.
pub const TEST32_ARC_GROUNDING: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix morphism: <https://uor.foundation/morphism/> .
@prefix query:    <https://uor.foundation/query/> .
@prefix u:        <https://uor.foundation/u/> .
@prefix type:     <https://uor.foundation/type/> .
@prefix partition:<https://uor.foundation/partition/> .
@prefix state:    <https://uor.foundation/state/> .

# ARC grid cell: (row=1, col=2, color=5) encoded as 8-bit address 101
morphism:ex_groundingMap_arc a owl:NamedIndividual, morphism:GroundingMap ;
    morphism:groundedAddress  u:addr_101 ;
    morphism:symbolConstraints type:ex_constraint_arc .

# ProjectionMap for the same frame
morphism:ex_projectionMap_arc a owl:NamedIndividual, morphism:ProjectionMap ;
    morphism:projectionFrame   state:ex_frame_arc ;
    morphism:projectionSource  partition:ex_partition_arc ;
    morphism:roundTripCoherence "true"^^xsd:boolean .

# RelationQuery: given source address 101 and shift relation, find target
query:ex_relationQuery_arc a owl:NamedIndividual, query:RelationQuery ;
    query:sourceAddress u:addr_101 ;
    query:relationType  type:ex_constraint_arc ;
    query:targetSite   partition:ex_siteBudget_open ;
    query:groundingMap  morphism:ex_groundingMap_arc ;
    query:projectionMap morphism:ex_projectionMap_arc .
"#;
