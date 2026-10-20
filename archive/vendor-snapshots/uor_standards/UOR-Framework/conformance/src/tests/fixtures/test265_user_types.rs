//! SHACL test 265: user-space types from `type` and `morphism` namespaces.

/// Instance graph for Test 265: User-space types.
pub const TEST265_USER_TYPES: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix morphism: <https://uor.foundation/morphism/> .

type:ex_inclusion_265 a owl:NamedIndividual, type:TypeInclusion .
type:ex_variance_265 a owl:NamedIndividual, type:VarianceAnnotation .
type:ex_lattice_265 a owl:NamedIndividual, type:SubtypingLattice .
morphism:ex_datum_265 a owl:NamedIndividual, morphism:ComputationDatum .
morphism:ex_app_morph_265 a owl:NamedIndividual, morphism:ApplicationMorphism .
morphism:ex_partial_app_265 a owl:NamedIndividual, morphism:PartialApplication .
morphism:ex_transform_265 a owl:NamedIndividual, morphism:TransformComposition .
"#;
