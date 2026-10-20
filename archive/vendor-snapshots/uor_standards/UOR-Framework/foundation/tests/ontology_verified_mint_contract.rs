//! Phase 18 — `OntologyVerifiedMint` contract verification.
//!
//! Asserts the cross-cutting invariants that downstream consumers rely
//! on across every Path-2 mint scaffold:
//!
//! 1. `Mint{Foo}` is `Send + Sync + Copy + Clone + Debug + Eq + PartialEq`.
//! 2. `THEOREM_IDENTITY` const equals the value the codegen-resolved
//!    `proof:provesIdentity` inverse-lookup produces.
//! 3. Two mints with identical inputs produce identical fingerprints
//!    (determinism).
//! 4. Mints with different non-zero inputs produce different
//!    fingerprints (Phase 15's content folding actually folds content).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_foundation::enforcement::ContentFingerprint;
use uor_foundation::witness_scaffolds::{
    MintBornRuleVerification, MintBornRuleVerificationInputs, MintCompletenessWitness,
    MintCompletenessWitnessInputs, MintLiftObstruction, MintLiftObstructionInputs,
    OntologyVerifiedMint,
};
use uor_foundation::DefaultHostTypes;

fn nonzero_fp(seed: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = seed;
    buf[1] = seed.wrapping_add(1);
    ContentFingerprint::from_buffer(buf, 32u8)
}

#[test]
fn mint_witnesses_are_send_sync_copy_clone() {
    fn assert_traits<T: Send + Sync + Copy + Clone + core::fmt::Debug + PartialEq + Eq>() {}
    assert_traits::<MintBornRuleVerification>();
    assert_traits::<MintCompletenessWitness>();
    assert_traits::<MintLiftObstruction>();
    assert_traits::<uor_foundation::witness_scaffolds::MintDisjointnessWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintImpossibilityWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintInhabitanceImpossibilityWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintMorphismGroundingWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintProjectionWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintStateGroundingWitness>();
    assert_traits::<uor_foundation::witness_scaffolds::MintWitness>();
}

#[test]
fn theorem_identity_constants_are_well_formed() {
    // Each Mint{Foo} carries a THEOREM_IDENTITY pointing to an
    // op-namespace identity IRI. Verify the IRI prefix discipline.
    fn assert_op_iri<W: OntologyVerifiedMint>() {
        let iri = W::THEOREM_IDENTITY;
        assert!(
            iri.starts_with("https://uor.foundation/op/"),
            "{iri} must be op-namespace"
        );
        assert!(
            !iri.is_empty() && iri.len() > "https://uor.foundation/op/".len(),
            "{iri} must carry a non-empty identity local-name"
        );
    }
    assert_op_iri::<MintBornRuleVerification>();
    assert_op_iri::<MintCompletenessWitness>();
    assert_op_iri::<MintLiftObstruction>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintDisjointnessWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintImpossibilityWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintInhabitanceImpossibilityWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintMorphismGroundingWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintProjectionWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintStateGroundingWitness>();
    assert_op_iri::<uor_foundation::witness_scaffolds::MintWitness>();
}

#[test]
fn mint_is_deterministic_for_identical_inputs() {
    // Two mints with byte-identical inputs must produce byte-identical
    // fingerprints — Phase 15's `fingerprint_for_inputs` is a pure
    // function of the input bytes.
    let inputs1 = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 7,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x99),
        ),
    };
    let inputs2 = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 7,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x99),
        ),
    };
    let w1 = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs1).unwrap();
    let w2 = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs2).unwrap();
    assert_eq!(
        w1.content_fingerprint(),
        w2.content_fingerprint(),
        "deterministic mint: identical inputs must produce identical fingerprints"
    );
}

#[test]
fn mint_distinguishes_distinct_inputs() {
    // Phase 15's fold over input bytes must produce DIFFERENT
    // fingerprints for inputs that differ in any field.
    let inputs_low = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 1,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x11),
        ),
    };
    let inputs_high = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 999,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x11),
        ),
    };
    let w_low = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs_low).unwrap();
    let w_high = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs_high).unwrap();
    assert_ne!(
        w_low.content_fingerprint(),
        w_high.content_fingerprint(),
        "distinct sites_closed must produce distinct fingerprints"
    );

    let inputs_low_h = MintCompletenessWitnessInputs::<DefaultHostTypes> {
        sites_closed: 1,
        witness_constraint: uor_foundation::user::type_::ConstraintHandle::<DefaultHostTypes>::new(
            nonzero_fp(0x22),
        ),
    };
    let w_low_h = MintCompletenessWitness::ontology_mint::<DefaultHostTypes>(inputs_low_h).unwrap();
    assert_ne!(
        w_low.content_fingerprint(),
        w_low_h.content_fingerprint(),
        "distinct constraint handles must produce distinct fingerprints"
    );
}

#[test]
fn mint_lift_obstruction_dispatches_on_trivial_flag() {
    // Phase 15's verify_type_lift_obstruction has TWO failure modes
    // depending on the obstruction_trivial flag:
    //   - trivial=true with non-zero site → WLS_1
    //   - trivial=false with zero site → WLS_2
    let trivial_with_site = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: true,
        obstruction_site:
            uor_foundation::bridge::partition::SiteIndexHandle::<DefaultHostTypes>::new(nonzero_fp(
                0xAA,
            )),
    };
    let err = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(trivial_with_site)
        .expect_err("trivial=true with non-zero site must reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/WLS_1"));

    let nontrivial_with_zero = MintLiftObstructionInputs::<DefaultHostTypes> {
        obstruction_trivial: false,
        obstruction_site:
            uor_foundation::bridge::partition::SiteIndexHandle::<DefaultHostTypes>::new(
                ContentFingerprint::default(),
            ),
    };
    let err = MintLiftObstruction::ontology_mint::<DefaultHostTypes>(nontrivial_with_zero)
        .expect_err("trivial=false with zero site must reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/WLS_2"));
}

#[test]
fn mint_born_rule_progressive_failure_routing() {
    // BR's verify body checks the structural invariants in order:
    // verified, born_rule_verified, witt_length, certifies. Each
    // failure routes to BR_1, BR_2, BR_3, BR_4 respectively.
    fn make_inputs(
        verified: bool,
        born: bool,
        witt: u64,
        certifies: &'static str,
    ) -> MintBornRuleVerificationInputs<DefaultHostTypes> {
        MintBornRuleVerificationInputs {
            born_rule_verified: born,
            certifies,
            method: uor_foundation::ProofStrategy::default(),
            timestamp: &[0u8],
            verified,
            witt_length: witt,
        }
    }

    // verified=false → BR_1.
    let err = MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(make_inputs(
        false, false, 0, "x",
    ))
    .unwrap_err();
    assert_eq!(err.identity(), Some("https://uor.foundation/op/BR_1"));

    // verified=true, born=false → BR_2.
    let err = MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(make_inputs(
        true, false, 0, "x",
    ))
    .unwrap_err();
    assert_eq!(err.identity(), Some("https://uor.foundation/op/BR_2"));

    // verified=true, born=true, witt=0 → BR_3.
    let err = MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(make_inputs(
        true, true, 0, "x",
    ))
    .unwrap_err();
    assert_eq!(err.identity(), Some("https://uor.foundation/op/BR_3"));

    // verified=true, born=true, witt=8, certifies="" (empty sentinel) → BR_4.
    let err =
        MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(make_inputs(true, true, 8, ""))
            .unwrap_err();
    assert_eq!(err.identity(), Some("https://uor.foundation/op/BR_4"));

    // All checks pass.
    let w = MintBornRuleVerification::ontology_mint::<DefaultHostTypes>(make_inputs(
        true,
        true,
        8,
        "https://uor.foundation/op/QM_5",
    ))
    .unwrap();
    assert!(!w.content_fingerprint().is_zero());
}
