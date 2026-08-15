//! #661/#720 finite Octeract conformance. These tests establish the bounded
//! byte algebra only; they make no attention-quality or formal-proof claim.

use uor_r4_graph_certify::octeract::{
    anchor_for_shell, distance_from_oriented, folded_class, masked_byte_distance,
    masked_weight_lower_bound, octeract_closed_form, octeract_closed_form_from_weight,
    octeract_sort_subtract, oriented_class, BlockDistance, ByteWeight, FullByteShell,
    OrientedClass, OCTERACT_ANCHORS, OCTERACT_CYPHER_SOURCE, OCTERACT_VALIDATION_SOURCE,
};

fn block_distance(distance: u8, active_bits: u8) -> BlockDistance {
    BlockDistance::new(distance, active_bits).expect("test inputs are bounded byte blocks")
}

fn byte_shell(shell: u8) -> FullByteShell {
    FullByteShell::new(shell).expect("test inputs are full-byte shells")
}

fn byte_weight(weight: u8) -> ByteWeight {
    ByteWeight::new(weight).expect("test inputs are byte weights")
}

#[test]
fn supplied_source_identities_are_exact_and_nonredistributable() {
    assert_eq!(
        OCTERACT_CYPHER_SOURCE.metadata_title,
        OCTERACT_CYPHER_SOURCE.displayed_title
    );
    assert_eq!(OCTERACT_CYPHER_SOURCE.filename, "Octeract_Cypher_Paper.pdf");
    assert_eq!(OCTERACT_CYPHER_SOURCE.byte_len, 54_969);
    assert_eq!(
        OCTERACT_CYPHER_SOURCE.sha256,
        "44bab09a20253437aeef43057ae316fcded5b00fd9f6b180f83843f06d2bbb3c"
    );
    assert_eq!(OCTERACT_VALIDATION_SOURCE.byte_len, 262_762);
    assert_eq!(
        OCTERACT_VALIDATION_SOURCE.metadata_title,
        "Validating Octeract Cypher Mathlib"
    );
    assert_eq!(
        OCTERACT_VALIDATION_SOURCE.displayed_title,
        "Formal Validation and Mechanized Verification of the Octeract Cypher: A Base-2 Kaprekar Adjunction Framework"
    );
    assert_eq!(
        OCTERACT_VALIDATION_SOURCE.filename,
        "Validating Octeract Cypher Mathlib.pdf"
    );
    assert_eq!(
        OCTERACT_VALIDATION_SOURCE.sha256,
        "5322c519fa872ca836e2ad23d523ecf655defedd3dd17589ba290dec62a93a5e"
    );
    for source in [OCTERACT_CYPHER_SOURCE, OCTERACT_VALIDATION_SOURCE] {
        assert_eq!(source.license, "NOASSERTION");
        assert_eq!(source.redistribution, "not-authorized");
        assert_eq!(
            source.provenance,
            "supplied-out-of-band-to-repository-maintainer"
        );
        assert_eq!(source.audited_on, "2026-08-14");
    }
}

#[test]
fn direct_oracle_closed_form_and_published_map_agree_exhaustively() {
    // The paper's named equal-weight example: both arrangements have four
    // ones and therefore route to the same published anchor.
    assert_eq!(octeract_sort_subtract(0b1010_1010), 225);
    assert_eq!(octeract_sort_subtract(0b1111_0000), 225);

    let published = [0u8, 127, 189, 217, 225, 217, 189, 127, 0];
    for (weight, &expected) in published.iter().enumerate() {
        assert_eq!(
            octeract_closed_form_from_weight(byte_weight(weight as u8)),
            expected
        );
    }
    assert_eq!(ByteWeight::new(9), None);

    let mut observed = [false; 256];
    for value in u8::MIN..=u8::MAX {
        let direct = octeract_sort_subtract(value);
        let closed = octeract_closed_form(value);
        assert_eq!(direct, closed, "input {value:#04x}");
        observed[direct as usize] = true;

        // Complement symmetry and one-step idempotency are finite facts over
        // the complete byte domain, including both endpoint weights.
        assert_eq!(closed, octeract_closed_form(!value));
        assert_eq!(closed, octeract_closed_form(closed));

        let weight = value.count_ones() as u8;
        let shell = folded_class(block_distance(weight, 8));
        assert_eq!(closed, anchor_for_shell(byte_shell(shell)));
    }
    let outputs: Vec<u8> = observed
        .iter()
        .enumerate()
        .filter_map(|(value, &present)| present.then_some(value as u8))
        .collect();
    assert_eq!(outputs, OCTERACT_ANCHORS);
}

#[test]
fn all_full_mask_pairs_obey_fold_orientation_and_safe_bound() {
    let mut complement_collisions = 0u64;
    for query in u8::MIN..=u8::MAX {
        for key in u8::MIN..=u8::MAX {
            let difference = query ^ key;
            let distance = masked_byte_distance(query, key, u8::MAX);
            let block = block_distance(distance, 8);
            let shell = folded_class(block);
            let oriented = oriented_class(block);
            assert_eq!(distance_from_oriented(oriented), distance);
            assert_eq!(
                octeract_sort_subtract(difference),
                anchor_for_shell(byte_shell(shell))
            );

            let complement_distance = masked_byte_distance(query, !key, u8::MAX);
            assert_eq!(complement_distance, 8 - distance);
            assert_eq!(
                folded_class(block_distance(complement_distance, 8)),
                folded_class(block)
            );
            if complement_distance != distance {
                complement_collisions += 1;
                assert_ne!(
                    oriented_class(block_distance(complement_distance, 8)),
                    oriented_class(block)
                );
            }

            let lower = masked_weight_lower_bound(query, key, u8::MAX);
            assert!(lower <= distance);
        }
    }
    // Exactly the C(8,4) * 256 equator pairs do not change distance
    // under key complementation; every other pair is a lossy fold collision.
    assert_eq!(complement_collisions, 65_536 - 70 * 256);
}

#[test]
fn partial_masks_and_wide_block_composition_are_bounded_and_nonvacuous() {
    // Exhaust the complete canonical oriented state space, including every
    // active width not represented by the fixed mask sample below.
    let mut canonical_states = 0u8;
    for active_bits in 0..=8u8 {
        for distance in 0..=active_bits {
            let class = oriented_class(block_distance(distance, active_bits));
            assert_eq!(distance_from_oriented(class), distance);
            canonical_states += 1;
        }
    }
    assert_eq!(canonical_states, 45);

    const MASKS: [u8; 8] = [0x00, 0x01, 0x0f, 0x33, 0x55, 0xaa, 0xfe, 0xff];
    let mut strict_lower_bounds = 0u64;
    for mask in MASKS {
        let active_bits = mask.count_ones() as u8;
        for query in u8::MIN..=u8::MAX {
            for key in u8::MIN..=u8::MAX {
                let distance = masked_byte_distance(query, key, mask);
                let lower = masked_weight_lower_bound(query, key, mask);
                assert!(distance <= active_bits);
                assert!(lower <= distance);
                if lower < distance {
                    strict_lower_bounds += 1;
                }
                let class = oriented_class(block_distance(distance, active_bits));
                assert_eq!(distance_from_oriented(class), distance);
            }
        }
    }
    assert!(
        strict_lower_bounds > 100_000,
        "the bound is not an equality"
    );

    // Fixed 288-bit example: summing the 36 ordinary block weights and
    // reconstructing those weights from oriented shells are exactly the same
    // relation. The unoriented fold is deliberately lossy.
    let mut exact_sum = 0u16;
    let mut reconstructed_sum = 0u16;
    let mut folded_sum = 0u16;
    for block in 0..36u8 {
        let query = block.wrapping_mul(73).rotate_left(1);
        let key = block.wrapping_mul(29).wrapping_add(0x5a);
        let distance = masked_byte_distance(query, key, u8::MAX);
        exact_sum += u16::from(distance);
        reconstructed_sum += u16::from(distance_from_oriented(oriented_class(block_distance(
            distance, 8,
        ))));
        folded_sum += u16::from(folded_class(block_distance(distance, 8)));
    }
    assert_eq!(exact_sum, reconstructed_sum);
    assert_ne!(exact_sum, folded_sum);
}

#[test]
fn anchor_integers_are_only_a_bijective_shell_relabeling() {
    let arbitrary_labels = [91u8, 4, 250, 33, 118];
    for left in 0..=8u8 {
        for right in 0..=8u8 {
            let left_shell = folded_class(block_distance(left, 8));
            let right_shell = folded_class(block_distance(right, 8));
            assert_eq!(
                anchor_for_shell(byte_shell(left_shell))
                    == anchor_for_shell(byte_shell(right_shell)),
                arbitrary_labels[left_shell as usize] == arbitrary_labels[right_shell as usize]
            );
        }
    }
}

#[test]
fn malformed_bounded_classes_fail_closed() {
    assert_eq!(BlockDistance::new(9, 8), None);
    assert_eq!(BlockDistance::new(0, 9), None);
    assert_eq!(FullByteShell::new(5), None);
    assert_eq!(OrientedClass::new(4, true, 8), None);
    assert_eq!(OrientedClass::new(3, false, 4), None);

    let canonical = OrientedClass::new(3, true, 8).expect("high-side shell is canonical");
    assert_eq!(canonical.shell(), 3);
    assert!(canonical.high_side());
    assert_eq!(canonical.active_bits(), 8);
    assert_eq!(distance_from_oriented(canonical), 5);
}
