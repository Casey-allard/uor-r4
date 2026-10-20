//! SHACL test 258: `stream` namespace types.

/// Instance graph for Test 258: Stream types.
pub const TEST258_STREAM_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix stream: <https://uor.foundation/stream/> .

stream:ex_productive_258 a owl:NamedIndividual, stream:ProductiveStream .
stream:ex_prefix_258 a owl:NamedIndividual, stream:StreamPrefix .
stream:ex_morphism_258 a owl:NamedIndividual, stream:StreamMorphism .
stream:ex_unfold_258 a owl:NamedIndividual, stream:Unfold .
"#;
