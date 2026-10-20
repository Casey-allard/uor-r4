//! Product/Coproduct Completion Amendment §D1.4 validation:
//! the three witness types implement the partition-algebra traits
//! (`PartitionProduct<H>`, `PartitionCoproduct<H>`,
//! `CartesianPartitionProduct<H>`) fully generic over `H: HostTypes`,
//! returning a `NullPartition<H>` from each accessor whose fingerprint
//! matches the witness's stored operand fingerprint.
//!
//! Also verifies `NullPartition<H>` itself implements `Partition<H>`
//! with the expected resolver-absent defaults: empty sub-components,
//! zero witt length, true is_exhaustive, `HostTypes::EMPTY_*`-derived
//! return values for the host-typed accessors.

use uor_foundation::bridge::partition::{
    CartesianPartitionProduct, Complement, Component, FreeRank, Partition, PartitionCoproduct,
    PartitionProduct, SiteIndex, TagSite,
};
use uor_foundation::{
    CartesianProductMintInputs, CartesianProductWitness, ContentFingerprint, DefaultHostTypes,
    NullPartition, PartitionCoproductMintInputs, PartitionCoproductWitness,
    PartitionProductMintInputs, PartitionProductWitness, VerifiedMint,
};

fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

type H = DefaultHostTypes;

// -- PartitionProduct<H> for PartitionProductWitness ---------------------

#[test]
fn partition_product_witness_implements_trait_with_null_partition_assoc() {
    let witness = PartitionProductWitness::mint_verified(PartitionProductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA0),
        right_fingerprint: fp(0xB0),
        left_site_budget: 2,
        right_site_budget: 3,
        left_total_site_count: 2,
        right_total_site_count: 3,
        left_euler: 0,
        right_euler: 0,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        combined_site_budget: 5,
        combined_site_count: 5,
        combined_euler: 0,
        combined_entropy_nats_bits: 0_u64,
        combined_fingerprint: fp(0xC0),
    })
    .expect("minting a valid PartitionProductWitness");

    // By-value accessor returns a NullPartition<H> whose fingerprint
    // round-trips the witness's stored operand fingerprint.
    let left = <PartitionProductWitness as PartitionProduct<H>>::left_factor(&witness);
    let right = <PartitionProductWitness as PartitionProduct<H>>::right_factor(&witness);
    assert_eq!(left.fingerprint(), fp(0xA0));
    assert_eq!(right.fingerprint(), fp(0xB0));
}

// -- PartitionCoproduct<H> for PartitionCoproductWitness -----------------

#[test]
fn partition_coproduct_witness_implements_trait_with_null_partition_assoc() {
    const TAG_COEFFS: [i64; uor_foundation::pipeline::AFFINE_MAX_COEFFS] = {
        let mut a = [0i64; uor_foundation::pipeline::AFFINE_MAX_COEFFS];
        a[3] = 1;
        a
    };
    const TAG_COEFF_COUNT: u32 = 4;
    static CONSTRAINTS: [uor_foundation::pipeline::ConstraintRef; 7] = [
        uor_foundation::pipeline::ConstraintRef::Site { position: 0 },
        uor_foundation::pipeline::ConstraintRef::Site { position: 1 },
        uor_foundation::pipeline::ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: 0,
        },
        uor_foundation::pipeline::ConstraintRef::Site { position: 0 },
        uor_foundation::pipeline::ConstraintRef::Carry { site: 1 },
        uor_foundation::pipeline::ConstraintRef::Site { position: 2 },
        uor_foundation::pipeline::ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1,
        },
    ];

    let witness = PartitionCoproductWitness::mint_verified(PartitionCoproductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA1),
        right_fingerprint: fp(0xB1),
        left_site_budget: 2,
        right_site_budget: 3,
        left_total_site_count: 2,
        right_total_site_count: 3,
        left_euler: 0,
        right_euler: 0,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        left_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        right_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        combined_site_budget: 3,
        combined_site_count: 4,
        combined_euler: 0,
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_betti: [0; uor_foundation::enforcement::MAX_BETTI_DIMENSION],
        combined_fingerprint: fp(0xC1),
        combined_constraints: &CONSTRAINTS,
        left_constraint_count: 3,
        tag_site: 3,
    })
    .expect("minting a valid PartitionCoproductWitness");

    let left = <PartitionCoproductWitness as PartitionCoproduct<H>>::left_summand(&witness);
    let right = <PartitionCoproductWitness as PartitionCoproduct<H>>::right_summand(&witness);
    assert_eq!(left.fingerprint(), fp(0xA1));
    assert_eq!(right.fingerprint(), fp(0xB1));
}

// -- CartesianPartitionProduct<H> for CartesianProductWitness ------------

#[test]
fn cartesian_product_witness_implements_trait_with_null_partition_assoc() {
    let witness = CartesianProductWitness::mint_verified(CartesianProductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA2),
        right_fingerprint: fp(0xB2),
        left_site_budget: 2,
        right_site_budget: 3,
        left_total_site_count: 2,
        right_total_site_count: 3,
        left_euler: 1,
        right_euler: 1,
        left_betti: [1, 0, 0, 0, 0, 0, 0, 0],
        right_betti: [1, 0, 0, 0, 0, 0, 0, 0],
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        combined_site_budget: 5,
        combined_site_count: 5,
        combined_euler: 1,
        combined_betti: [1, 0, 0, 0, 0, 0, 0, 0],
        combined_entropy_nats_bits: 0_u64,
        combined_fingerprint: fp(0xC2),
    })
    .expect("minting a valid CartesianProductWitness");

    let left =
        <CartesianProductWitness as CartesianPartitionProduct<H>>::left_cartesian_factor(&witness);
    let right =
        <CartesianProductWitness as CartesianPartitionProduct<H>>::right_cartesian_factor(&witness);
    assert_eq!(left.fingerprint(), fp(0xA2));
    assert_eq!(right.fingerprint(), fp(0xB2));
}

// -- Partition<H> for NullPartition<H> -----------------------------------

#[test]
fn null_partition_implements_partition_trait_with_resolver_absent_defaults() {
    let np = NullPartition::<H>::from_fingerprint(fp(0xFF));

    // Reference-returning component accessors return empty / zero-
    // cardinality sub-components per the resolver-absent defaults.
    let irreducibles = <NullPartition<H> as Partition<H>>::irreducibles(&np);
    let reducibles = <NullPartition<H> as Partition<H>>::reducibles(&np);
    let units = <NullPartition<H> as Partition<H>>::units(&np);
    assert_eq!(Component::<H>::cardinality(irreducibles), 0);
    assert_eq!(Component::<H>::cardinality(reducibles), 0);
    assert_eq!(Component::<H>::cardinality(units), 0);
    let exterior = <NullPartition<H> as Partition<H>>::exterior(&np);
    let source_type = <NullPartition<H> as Partition<H>>::source_type(&np);
    let free_rank = <NullPartition<H> as Partition<H>>::site_budget(&np);
    let tag = <NullPartition<H> as Partition<H>>::tag_site_of(&np);

    // Value-returning accessors produce the resolver-absent defaults.
    assert_eq!(<NullPartition<H> as Partition<H>>::density(&np), 0.0);
    assert_eq!(<NullPartition<H> as Partition<H>>::witt_length(&np), 0);
    assert!(
        <NullPartition<H> as Partition<H>>::is_exhaustive(&np),
        "empty partition is trivially exhaustive"
    );
    assert_eq!(
        <NullPartition<H> as Partition<H>>::product_category_level(&np),
        <H as uor_foundation::HostTypes>::EMPTY_HOST_STRING
    );

    // Sub-trait accessors return amendment-documented defaults.
    assert_eq!(Component::<H>::cardinality(exterior), 0);
    assert!(Component::<H>::member(exterior).is_empty());
    // Reference validity: the trait returns a live reference into
    // `np.exterior.term`. Use the concrete NullTermExpression type when
    // casting to a pointer so clippy doesn't reject an inferable raw
    // pointer type.
    let ext_criteria_ref = Complement::<H>::exterior_criteria(exterior);
    let ext_criteria_ptr: *const _ = core::ptr::from_ref(ext_criteria_ref);
    assert!(
        !ext_criteria_ptr.is_null(),
        "exterior_criteria must point into np.exterior.term"
    );
    let _ = source_type;
    assert_eq!(FreeRank::<H>::total_sites(free_rank), 0);
    assert_eq!(FreeRank::<H>::pinned_count(free_rank), 0);
    assert_eq!(FreeRank::<H>::free_rank(free_rank), 0);
    assert!(FreeRank::<H>::is_closed(free_rank));
    assert_eq!(SiteIndex::<H>::site_position(tag), 0);
    assert!(!TagSite::<H>::tag_value(tag));

    // Fingerprint round-trips.
    assert_eq!(np.fingerprint(), fp(0xFF));
}
