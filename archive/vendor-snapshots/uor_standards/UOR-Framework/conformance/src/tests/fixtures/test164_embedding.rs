//! SHACL test 164: `morphism:Embedding` — quantum-level injective transform.
//!
//! An `Embedding` is an injective, structure-preserving transform from a ring
//! at one quantum level into a larger ring at a higher quantum level.

/// Instance graph for Test 164: morphism:Embedding.
pub const TEST164_EMBEDDING: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix morphism: <https://uor.foundation/morphism/> .

<https://uor.foundation/instance/embed_3_to_4>
    a                        owl:NamedIndividual, morphism:Embedding ;
    morphism:sourceQuantum   "3"^^xsd:positiveInteger ;
    morphism:targetQuantum   "4"^^xsd:positiveInteger .
"#;
