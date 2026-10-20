//! SHACL test 255: `effect` namespace types.

/// Instance graph for Test 255: Effect subclasses and related types.
pub const TEST255_EFFECT_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix effect: <https://uor.foundation/effect/> .

effect:ex_reversible_255 a owl:NamedIndividual, effect:ReversibleEffect .
effect:ex_pinning_255 a owl:NamedIndividual, effect:PinningEffect .
effect:ex_unbinding_255 a owl:NamedIndividual, effect:UnbindingEffect .
effect:ex_phase_255 a owl:NamedIndividual, effect:PhaseEffect .
effect:ex_composite_255 a owl:NamedIndividual, effect:CompositeEffect .
effect:ex_external_255 a owl:NamedIndividual, effect:ExternalEffect .
effect:ex_target_255 a owl:NamedIndividual, effect:EffectTarget .
effect:ex_disjointness_witness_255 a owl:NamedIndividual, effect:DisjointnessWitness .
"#;
