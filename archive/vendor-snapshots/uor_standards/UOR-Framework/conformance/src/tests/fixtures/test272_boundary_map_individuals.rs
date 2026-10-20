//! SHACL test 272: morphism:GroundingMap and ProjectionMap individuals.

/// Instance graph for Test 272: All 6 boundary map individuals.
pub const TEST272_BOUNDARY_MAP_INDIVIDUALS: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix morphism: <https://uor.foundation/morphism/> .

morphism:IntegerGroundingMap  a owl:NamedIndividual, morphism:GroundingMap .
morphism:Utf8GroundingMap     a owl:NamedIndividual, morphism:GroundingMap .
morphism:JsonGroundingMap     a owl:NamedIndividual, morphism:GroundingMap .
morphism:IntegerProjectionMap a owl:NamedIndividual, morphism:ProjectionMap .
morphism:Utf8ProjectionMap    a owl:NamedIndividual, morphism:ProjectionMap .
morphism:JsonProjectionMap    a owl:NamedIndividual, morphism:ProjectionMap .
"#;
