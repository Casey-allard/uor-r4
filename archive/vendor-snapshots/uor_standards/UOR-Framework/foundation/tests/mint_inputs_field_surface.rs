//! Phase 14 verification: every `Mint{Foo}Inputs<H>` exposes per-property
//! public fields matching its ontology class's properties (own + inherited
//! from `subclass_of` chain). Each field is reachable via field access;
//! the struct is `Debug + Clone + Copy + Default`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::enforcement::ContentFingerprint;
use uor_foundation::witness_scaffolds::{
    MintBornRuleVerificationInputs, MintCompletenessWitnessInputs, MintDisjointnessWitnessInputs,
    MintImpossibilityWitnessInputs, MintInhabitanceImpossibilityWitnessInputs,
    MintLiftObstructionInputs, MintMorphismGroundingWitnessInputs, MintProjectionWitnessInputs,
    MintStateGroundingWitnessInputs, MintWitnessInputs,
};
use uor_foundation::DefaultHostTypes;

#[test]
fn born_rule_verification_inputs_field_surface() {
    let inputs = MintBornRuleVerificationInputs::<DefaultHostTypes> {
        born_rule_verified: true,
        certifies: "https://uor.foundation/op/QM_5",
        method: uor_foundation::ProofStrategy::default(),
        timestamp: &[0u8],
        verified: true,
        witt_length: 8,
    };
    // Every field reachable.
    assert!(inputs.born_rule_verified);
    assert_eq!(inputs.certifies, "https://uor.foundation/op/QM_5");
    assert_eq!(inputs.witt_length, 8);
    assert!(inputs.verified);

    // Default fills sentinels.
    let d = MintBornRuleVerificationInputs::<DefaultHostTypes>::default();
    assert!(!d.born_rule_verified);
    assert_eq!(d.witt_length, 0);
    assert!(!d.verified);
}

#[test]
fn disjointness_witness_inputs_field_surface() {
    let fp = ContentFingerprint::default();
    let inputs = MintDisjointnessWitnessInputs::<DefaultHostTypes> {
        disjointness_left:
            uor_foundation::kernel::effect::EffectTargetHandle::<DefaultHostTypes>::new(fp),
        disjointness_right:
            uor_foundation::kernel::effect::EffectTargetHandle::<DefaultHostTypes>::new(fp),
    };
    let _l = inputs.disjointness_left;
    let _r = inputs.disjointness_right;

    // Default fills handles with zero fingerprint.
    let d = MintDisjointnessWitnessInputs::<DefaultHostTypes>::default();
    assert!(d.disjointness_left.fingerprint.is_zero());
    assert!(d.disjointness_right.fingerprint.is_zero());
}

#[test]
fn morphism_grounding_witness_inputs_field_surface() {
    let d = MintMorphismGroundingWitnessInputs::<DefaultHostTypes>::default();
    assert!(d.grounded_address.fingerprint.is_zero());
    assert!(d.surface_symbol.fingerprint.is_zero());
}

#[test]
fn projection_witness_inputs_field_surface() {
    let d = MintProjectionWitnessInputs::<DefaultHostTypes>::default();
    assert!(d.projection_output.fingerprint.is_zero());
    // projection_source maps to crate::enforcement::PartitionHandle (AlreadyImplemented).
    let _ = d.projection_source;
}

#[test]
fn morphism_witness_inputs_is_phantom_only() {
    // Abstract supertype — no own/inherited properties.
    let _d = MintWitnessInputs::<DefaultHostTypes>::default();
}

#[test]
fn impossibility_witness_inputs_field_surface() {
    let d = MintImpossibilityWitnessInputs::<DefaultHostTypes>::default();
    assert_eq!(d.depends_on.len(), 0);
    assert_eq!(d.witness.len(), 0);
    assert_eq!(d.verified_at_level.len(), 0);
    assert!(!d.verified);
    assert!(d.formal_derivation.fingerprint.is_zero());
    assert!(d.proves_identity.fingerprint.is_zero());
}

#[test]
fn inhabitance_impossibility_witness_inputs_field_surface() {
    let d = MintInhabitanceImpossibilityWitnessInputs::<DefaultHostTypes>::default();
    // 3 own + 12 inherited.
    assert!(d.contradiction_proof.is_empty());
    assert!(d.grounded.fingerprint.is_zero());
    assert!(d.search_trace.fingerprint.is_zero());
    // Inherited from ImpossibilityWitness:
    assert_eq!(d.depends_on.len(), 0);
    assert!(!d.verified);
}

#[test]
fn state_grounding_witness_inputs_field_surface() {
    let d = MintStateGroundingWitnessInputs::<DefaultHostTypes>::default();
    assert_eq!(d.witness_step, 0);
    assert_eq!(d.witness_binding.len(), 0);
}

#[test]
fn completeness_witness_inputs_field_surface() {
    let d = MintCompletenessWitnessInputs::<DefaultHostTypes>::default();
    assert_eq!(d.sites_closed, 0);
    assert!(d.witness_constraint.fingerprint.is_zero());
}

#[test]
fn lift_obstruction_inputs_field_surface() {
    let d = MintLiftObstructionInputs::<DefaultHostTypes>::default();
    assert!(!d.obstruction_trivial);
    assert!(d.obstruction_site.fingerprint.is_zero());
}

#[test]
fn all_inputs_are_copy_clone_debug_default() {
    fn assert_traits<T: Copy + Clone + core::fmt::Debug + Default>() {}

    assert_traits::<MintBornRuleVerificationInputs<DefaultHostTypes>>();
    assert_traits::<MintDisjointnessWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintMorphismGroundingWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintProjectionWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintImpossibilityWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintInhabitanceImpossibilityWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintStateGroundingWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintCompletenessWitnessInputs<DefaultHostTypes>>();
    assert_traits::<MintLiftObstructionInputs<DefaultHostTypes>>();
}
