//! Phase 15 verification — every Path-2 `Mint{Foo}` witness mints
//! successfully via `OntologyVerifiedMint::ontology_mint` when given
//! populated `Mint{Foo}Inputs<H>`. `Default::default()` inputs are
//! rejected with a typed `GenericImpossibilityWitness` whose IRI cites
//! the family-specific failure mode.
//!
//! Each test produces a non-zero fingerprint distinct across families
//! — the Phase-15 `fingerprint_for_inputs` folds the input bytes
//! together with the THEOREM_IDENTITY IRI.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::enforcement::ContentFingerprint;
use uor_foundation::witness_scaffolds::{
    MintBornRuleVerification, MintBornRuleVerificationInputs, MintCompletenessWitness,
    MintCompletenessWitnessInputs, MintDisjointnessWitness, MintDisjointnessWitnessInputs,
    MintImpossibilityWitness, MintImpossibilityWitnessInputs, MintInhabitanceImpossibilityWitness,
    MintInhabitanceImpossibilityWitnessInputs, MintLiftObstruction, MintLiftObstructionInputs,
    MintMorphismGroundingWitness, MintMorphismGroundingWitnessInputs, MintProjectionWitness,
    MintProjectionWitnessInputs, MintStateGroundingWitness, MintStateGroundingWitnessInputs,
    MintWitness, MintWitnessInputs, OntologyVerifiedMint,
};
use uor_foundation::DefaultHostTypes;

fn nonzero_fp(seed: u8) -> ContentFingerprint {
    // 32 = `<DefaultHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES`, the
    // default const-generic on `ContentFingerprint`.
    let mut buf = [0u8; 32];
    buf[0] = seed;
    buf[1] = seed.wrapping_add(1);
    ContentFingerprint::from_buffer(buf, 32u8)
}

fn assert_ok_with_fingerprint(witness_label: &str, fp: ContentFingerprint) {
    assert!(
        !fp.is_zero(),
        "{witness_label}: fingerprint must be non-zero (Phase-15 derives from input bytes)"
    );
    assert!(
        fp.width_bytes() > 0,
        "{witness_label}: fingerprint width_bytes must be > 0"
    );
}

fn assert_default_rejects_with_iri<W, I, F>(witness_label: &str, expected_iri_prefix: &str, mint: F)
where
    F: Fn(I) -> Result<W, uor_foundation::enforcement::GenericImpossibilityWitness>,
    I: Default,
    W: core::fmt::Debug,
{
    let err = match mint(I::default()) {
        Ok(_) => panic!("{witness_label}: Default inputs should be rejected by Phase-15 verify"),
        Err(e) => e,
    };
    let identity = err
        .identity()
        .unwrap_or_else(|| panic!("{witness_label}: error must carry an op-namespace IRI"));
    assert!(
        identity.starts_with(expected_iri_prefix),
        "{witness_label}: expected IRI prefix `{expected_iri_prefix}`, got `{identity}`"
    );
}

#[test]
fn br_populated_mint_succeeds() {
    let inputs = MintBornRuleVerificationInputs::<DefaultHostTypes> {
        born_rule_verified: true,
        certifies: "https://uor.foundation/op/QM_5",
        method: uor_foundation::ProofStrategy::default(),
        timestamp: &[0u8],
        verified: true,
        witt_length: 8,
    };
    let w = MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("BR family verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintBornRuleVerification", w.content_fingerprint());
}

#[test]
fn br_default_rejects_at_br_1() {
    assert_default_rejects_with_iri::<
        MintBornRuleVerification,
        MintBornRuleVerificationInputs<DefaultHostTypes>,
        _,
    >(
        "MintBornRuleVerification",
        "https://uor.foundation/op/BR_",
        MintBornRuleVerification::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn cc_populated_mint_succeeds() {
    let inputs = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 4,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x42),
        ),
    };
    let w = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("CC family verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintCompletenessWitness", w.content_fingerprint());
}

#[test]
fn cc_default_rejects_at_cc_1() {
    assert_default_rejects_with_iri::<
        MintCompletenessWitness,
        MintCompletenessWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintCompletenessWitness",
        "https://uor.foundation/op/CC_",
        MintCompletenessWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn dp_populated_mint_succeeds() {
    let inputs = MintDisjointnessWitnessInputs::<DefaultHostTypes> {
        disjointness_left:
            uor_foundation::kernel::effect::EffectTargetHandle::<DefaultHostTypes>::new(nonzero_fp(
                0x10,
            )),
        disjointness_right:
            uor_foundation::kernel::effect::EffectTargetHandle::<DefaultHostTypes>::new(nonzero_fp(
                0x20,
            )),
    };
    let w = MintDisjointnessWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("DP family verify must succeed for populated, distinct inputs");
    assert_ok_with_fingerprint("MintDisjointnessWitness", w.content_fingerprint());
}

#[test]
fn dp_default_rejects_at_fx_4() {
    assert_default_rejects_with_iri::<
        MintDisjointnessWitness,
        MintDisjointnessWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintDisjointnessWitness",
        "https://uor.foundation/op/FX_4",
        MintDisjointnessWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn ih_impossibility_populated_mint_succeeds() {
    let inputs = MintImpossibilityWitnessInputs::<DefaultHostTypes> {
        achievability_status: uor_foundation::AchievabilityStatus::default(),
        depends_on: &[],
        formal_derivation:
            uor_foundation::bridge::proof::DerivationTermHandle::<DefaultHostTypes>::new(
                nonzero_fp(0x33),
            ),
        impossibility_domain: uor_foundation::VerificationDomain::default(),
        impossibility_reason: "missing-witness",
        proves_identity: uor_foundation::kernel::op::IdentityHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x44),
        ),
        strategy: uor_foundation::ProofStrategy::default(),
        timestamp: &[0u8],
        verified: true,
        verified_at_level: &[],
        witness: &[],
    };
    let w = MintImpossibilityWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("IH/ImpossibilityWitness verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintImpossibilityWitness", w.content_fingerprint());
}

#[test]
fn ih_impossibility_default_rejects_at_ih_1() {
    assert_default_rejects_with_iri::<
        MintImpossibilityWitness,
        MintImpossibilityWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintImpossibilityWitness",
        "https://uor.foundation/op/IH_",
        MintImpossibilityWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn ih_inhabitance_populated_mint_succeeds() {
    let inputs =
        MintInhabitanceImpossibilityWitnessInputs::<DefaultHostTypes> {
            achievability_status: uor_foundation::AchievabilityStatus::default(),
            contradiction_proof: "carrier(T) = empty",
            depends_on: &[],
            formal_derivation: uor_foundation::bridge::proof::DerivationTermHandle::<
                DefaultHostTypes,
            >::new(nonzero_fp(0x55)),
            grounded: uor_foundation::user::type_::ConstrainedTypeHandle::<DefaultHostTypes>::new(
                nonzero_fp(0x66),
            ),
            impossibility_domain: uor_foundation::VerificationDomain::default(),
            impossibility_reason: "carrier-empty",
            proves_identity: uor_foundation::kernel::op::IdentityHandle::<DefaultHostTypes>::new(
                nonzero_fp(0x77),
            ),
            search_trace: uor_foundation::bridge::trace::InhabitanceSearchTraceHandle::<
                DefaultHostTypes,
            >::new(nonzero_fp(0x88)),
            strategy: uor_foundation::ProofStrategy::default(),
            timestamp: &[0u8],
            verified: true,
            verified_at_level: &[],
            witness: &[],
        };
    let w = MintInhabitanceImpossibilityWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("IH/InhabitanceImpossibilityWitness verify must succeed for populated inputs");
    assert_ok_with_fingerprint(
        "MintInhabitanceImpossibilityWitness",
        w.content_fingerprint(),
    );
}

#[test]
fn ih_inhabitance_default_rejects_at_ih_1() {
    assert_default_rejects_with_iri::<
        MintInhabitanceImpossibilityWitness,
        MintInhabitanceImpossibilityWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintInhabitanceImpossibilityWitness",
        "https://uor.foundation/op/IH_",
        MintInhabitanceImpossibilityWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn lo_populated_mint_succeeds_non_trivial() {
    let inputs = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: false,
        obstruction_site:
            uor_foundation::bridge::partition::SiteIndexHandle::<DefaultHostTypes>::new(nonzero_fp(
                0xAA,
            )),
    };
    let w = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("LO family verify must succeed for non-trivial obstruction with site");
    assert_ok_with_fingerprint("MintLiftObstruction (non-trivial)", w.content_fingerprint());
}

#[test]
fn lo_populated_mint_succeeds_trivial() {
    let inputs = MintLiftObstructionInputs::<DefaultHostTypes>::default();
    // Default has obstruction_trivial = false AND zero site → rejects at WLS_2.
    // Populate with trivial=true + zero site → succeeds.
    let inputs = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: true,
        obstruction_site: inputs.obstruction_site,
    };
    let w = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("LO family verify must succeed for trivial obstruction with zero site");
    assert_ok_with_fingerprint("MintLiftObstruction (trivial)", w.content_fingerprint());
}

#[test]
fn lo_default_rejects_at_wls() {
    assert_default_rejects_with_iri::<
        MintLiftObstruction,
        MintLiftObstructionInputs<DefaultHostTypes>,
        _,
    >(
        "MintLiftObstruction",
        "https://uor.foundation/op/WLS_",
        MintLiftObstruction::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn oa_morphism_grounding_populated_mint_succeeds() {
    let inputs = MintMorphismGroundingWitnessInputs::<DefaultHostTypes> {
        grounded_address: uor_foundation::kernel::address::ElementHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x11),
        ),
        surface_symbol:
            uor_foundation::kernel::schema::SurfaceSymbolHandle::<DefaultHostTypes>::new(
                nonzero_fp(0x12),
            ),
    };
    let w = MintMorphismGroundingWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("OA/morphism::GroundingWitness verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintMorphismGroundingWitness", w.content_fingerprint());
}

#[test]
fn oa_morphism_grounding_default_rejects_at_surface_symmetry() {
    assert_default_rejects_with_iri::<
        MintMorphismGroundingWitness,
        MintMorphismGroundingWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintMorphismGroundingWitness",
        "https://uor.foundation/op/surfaceSymmetry",
        MintMorphismGroundingWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn oa_projection_populated_mint_succeeds() {
    let inputs = MintProjectionWitnessInputs::<DefaultHostTypes> {
        projection_output:
            uor_foundation::user::morphism::SymbolSequenceHandle::<DefaultHostTypes>::new(
                nonzero_fp(0x13),
            ),
        projection_source: uor_foundation::PartitionHandle::<DefaultHostTypes>::from_fingerprint(
            nonzero_fp(0x14),
        ),
    };
    let w = MintProjectionWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("OA/morphism::ProjectionWitness verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintProjectionWitness", w.content_fingerprint());
}

#[test]
fn oa_projection_default_rejects_at_surface_symmetry() {
    assert_default_rejects_with_iri::<
        MintProjectionWitness,
        MintProjectionWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintProjectionWitness",
        "https://uor.foundation/op/surfaceSymmetry",
        MintProjectionWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn oa_state_grounding_populated_mint_succeeds() {
    // Phase-15 invariants: witness_step > 0 AND witness_binding non-empty.
    // BindingHandle::new isn't const over a runtime fingerprint, so use a
    // function-local lifetime via Box::leak (test-only).
    let bindings: &'static [uor_foundation::user::state::BindingHandle<DefaultHostTypes>] =
        Box::leak(Box::new([uor_foundation::user::state::BindingHandle::<
            DefaultHostTypes,
        >::new(nonzero_fp(0xBB))]));
    let inputs = MintStateGroundingWitnessInputs::<DefaultHostTypes> {
        witness_binding: bindings,
        witness_step: 3,
    };
    let w = MintStateGroundingWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("OA/state::GroundingWitness verify must succeed for populated inputs");
    assert_ok_with_fingerprint("MintStateGroundingWitness", w.content_fingerprint());
}

#[test]
fn oa_state_grounding_default_rejects_at_surface_symmetry() {
    assert_default_rejects_with_iri::<
        MintStateGroundingWitness,
        MintStateGroundingWitnessInputs<DefaultHostTypes>,
        _,
    >(
        "MintStateGroundingWitness",
        "https://uor.foundation/op/surfaceSymmetry",
        MintStateGroundingWitness::ontology_mint::<DefaultHostTypes>,
    );
}

#[test]
fn oa_morphism_witness_default_succeeds() {
    // Abstract MintWitness has zero fields → no invariants → always Ok.
    let inputs = MintWitnessInputs::<DefaultHostTypes>::default();
    let w = MintWitness::ontology_mint::<DefaultHostTypes>(inputs)
        .expect("OA/morphism::Witness (abstract) verify must always succeed");
    assert_ok_with_fingerprint("MintWitness", w.content_fingerprint());
}

#[test]
fn fingerprints_distinguish_witnesses_across_families() {
    let mut fps = std::collections::HashSet::new();
    fps.insert(
        MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(
            MintBornRuleVerificationInputs::<DefaultHostTypes> {
                born_rule_verified: true,
                certifies: "qm5",
                method: uor_foundation::ProofStrategy::default(),
                timestamp: &[1u8],
                verified: true,
                witt_length: 8,
            },
        )
        .unwrap()
        .content_fingerprint(),
    );
    fps.insert(
        MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(
            MintCompletenessWitnessInputs::<DefaultHostTypes> {
                sites_closed: 1,
                witness_constraint:
                    uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
                        nonzero_fp(0xC1),
                    ),
            },
        )
        .unwrap()
        .content_fingerprint(),
    );
    fps.insert(
        MintDisjointnessWitness::ontology_mint::<DefaultHostTypes>(
            MintDisjointnessWitnessInputs::<DefaultHostTypes> {
                disjointness_left: uor_foundation::kernel::effect::EffectTargetHandle::<
                    DefaultHostTypes,
                >::new(nonzero_fp(0xD1)),
                disjointness_right: uor_foundation::kernel::effect::EffectTargetHandle::<
                    DefaultHostTypes,
                >::new(nonzero_fp(0xD2)),
            },
        )
        .unwrap()
        .content_fingerprint(),
    );
    fps.insert(
        MintWitness::ontology_mint::<DefaultHostTypes>(
            MintWitnessInputs::<DefaultHostTypes>::default(),
        )
        .unwrap()
        .content_fingerprint(),
    );
    assert!(
        fps.len() >= 4,
        "Population sample of 4 distinct families must produce 4 distinct fingerprints; got {}",
        fps.len()
    );
}
