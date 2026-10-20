//! Test 11: Composition primitive (Amendment 12).
//!
//! Validates: `morphism:Composition`, `morphism:CompositionLaw`,
//! `morphism:criticalComposition`, `morphism:Identity`, `morphism:identityOn`.

/// Instance graph for Test 11: Composition primitive.
pub const TEST11_COMPOSITION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix morphism:   <https://uor.foundation/morphism/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix op:         <https://uor.foundation/op/> .

# The critical composition law: neg ∘ bnot = succ
morphism:criticalComposition
    a                       owl:NamedIndividual, morphism:CompositionLaw ;
    morphism:lawComponents  op:neg ;
    morphism:lawComponents  op:bnot ;
    morphism:lawResult      op:succ ;
    morphism:isAssociative  false ;
    morphism:isCommutative  false .

# A concrete composition transform
<https://uor.foundation/instance/compose-neg-bnot>
    a                           owl:NamedIndividual, morphism:Composition ;
    morphism:compositionComponents <https://uor.foundation/instance/transform-neg> ;
    morphism:compositionComponents <https://uor.foundation/instance/transform-bnot> ;
    morphism:compositionResult  <https://uor.foundation/instance/transform-succ> ;
    morphism:compositionOrder   "2"^^xsd:nonNegativeInteger ;
    morphism:preservesStructure "ring homomorphism" .

# Component transforms
<https://uor.foundation/instance/transform-neg>
    a                       owl:NamedIndividual, morphism:Isometry ;
    morphism:composesWith   <https://uor.foundation/instance/transform-bnot> .

<https://uor.foundation/instance/transform-bnot>
    a                       owl:NamedIndividual, morphism:Isometry .

<https://uor.foundation/instance/transform-succ>
    a                       owl:NamedIndividual, morphism:Transform .

# An identity transform on a type
<https://uor.foundation/instance/identity-u8>
    a                       owl:NamedIndividual, morphism:Identity ;
    morphism:identityOn     <https://uor.foundation/instance/type-u8> .

<https://uor.foundation/instance/type-u8>
    a                       owl:NamedIndividual, type:PrimitiveType ;
    type:bitWidth           "8"^^xsd:positiveInteger .
"#;
