//! SHACL test 274: morphism:Witness and related types.

/// Instance graph for Test 274: Witness, GroundingWitness,
/// ProjectionWitness, SymbolSequence, and SequenceElement instances.
pub const TEST274_WITNESS_TYPES: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix morphism: <https://uor.foundation/morphism/> .

morphism:ex_witness_274             a owl:NamedIndividual, morphism:Witness .
morphism:ex_grounding_witness_274   a owl:NamedIndividual, morphism:GroundingWitness .
morphism:ex_projection_witness_274  a owl:NamedIndividual, morphism:ProjectionWitness .
morphism:ex_symbol_sequence_274     a owl:NamedIndividual, morphism:SymbolSequence .
morphism:ex_sequence_element_274    a owl:NamedIndividual, morphism:SequenceElement .
"#;
