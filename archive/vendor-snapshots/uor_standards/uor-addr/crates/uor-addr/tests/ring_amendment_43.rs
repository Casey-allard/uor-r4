//! UOR-Framework Amendment 43 §2 conformance suite for the ring
//! realization.
//!
//! Pins [`uor_addr::ring::RingElement`] against the canonical-bytes
//! layout `header(k) || le_bytes(x, k+1)` documented at
//! <https://github.com/UOR-Foundation/UOR-Framework/wiki/Amendment-43>.

use uor_addr::ring::{address, AddressFailure, RingElement, MAX_WITT_LEVEL};

#[test]
fn amendment_43_canonical_bytes_layout_at_every_witt_level() {
    for k in 0..=MAX_WITT_LEVEL {
        // Coefficient 0x12_34_56_78 truncated to k+1 little-endian bytes.
        let coeff: u64 = 0x12_34_56_78;
        let element = RingElement::from_components(k, coeff).expect("valid");
        let bytes = element.tagged_bytes();
        // header byte is the Witt level itself.
        assert_eq!(bytes[0], k);
        // payload is (k+1) bytes little-endian.
        let expected_payload: alloc::vec::Vec<u8> = coeff
            .to_le_bytes()
            .iter()
            .take((k + 1) as usize)
            .copied()
            .collect();
        assert_eq!(&bytes[1..], expected_payload.as_slice());
        // Total width is 1 + (k+1).
        assert_eq!(bytes.len(), 1 + (k as usize + 1));
    }
}

#[test]
fn zero_coefficient_at_every_witt_level() {
    for k in 0..=MAX_WITT_LEVEL {
        let element = RingElement::from_components(k, 0).expect("valid");
        let bytes = element.tagged_bytes();
        assert_eq!(bytes[0], k);
        for &b in &bytes[1..] {
            assert_eq!(b, 0);
        }
    }
}

#[test]
fn maximum_coefficient_value_per_width() {
    // Each Witt level k admits coefficients up to 2^(8*(k+1)) - 1.
    let cases: &[(u8, u64)] = &[(0, 0xFF), (1, 0xFFFF), (2, 0xFFFFFF), (3, 0xFFFFFFFF)];
    for &(k, max_value) in cases {
        let element = RingElement::from_components(k, max_value).expect("valid max value");
        let bytes = element.tagged_bytes();
        assert_eq!(bytes[0], k);
        let payload = &bytes[1..];
        assert_eq!(payload.len(), (k as usize) + 1);
        // All payload bytes should be 0xFF (max).
        for &b in payload {
            assert_eq!(b, 0xFF);
        }
    }
}

#[test]
fn canonicalize_is_identity_amendment_43() {
    // Amendment 43 §2 pins canonical bytes at construction: re-parsing a
    // ring element's tagged bytes and re-emitting them is the identity.
    for k in 0..=MAX_WITT_LEVEL {
        let element = RingElement::from_components(k, 0xABCD).expect("valid");
        let bytes = element.tagged_bytes().to_vec();
        let reparsed = RingElement::parse(&bytes).expect("re-parse canonical bytes");
        assert_eq!(reparsed.tagged_bytes(), bytes);
    }
}

#[test]
fn distinct_witt_levels_distinct_kappa_labels() {
    let labels: alloc::vec::Vec<uor_addr::KappaLabel<71>> = (0..=MAX_WITT_LEVEL)
        .map(|k| {
            let e = RingElement::from_components(k, 0x42).expect("valid");
            address(e.tagged_bytes()).expect("κ-label").address
        })
        .collect();
    // Every pair must differ.
    for i in 0..labels.len() {
        for j in (i + 1)..labels.len() {
            assert_ne!(labels[i], labels[j], "k={i} κ-label collides with k={j}");
        }
    }
}

#[test]
fn distinct_coefficients_within_witt_level_distinct_kappa_labels() {
    for k in 0..=MAX_WITT_LEVEL {
        let a = RingElement::from_components(k, 0x01).expect("valid");
        let b = RingElement::from_components(k, 0x02).expect("valid");
        let la = address(a.tagged_bytes()).expect("κ-label").address;
        let lb = address(b.tagged_bytes()).expect("κ-label").address;
        assert_ne!(la, lb, "k={k}: coefficient 0x01 collides with 0x02");
    }
}

#[test]
fn rejects_witt_level_above_bound() {
    for over in [MAX_WITT_LEVEL + 1, MAX_WITT_LEVEL + 2, 127, 255] {
        match RingElement::from_components(over, 0) {
            Err(v) if v.constraint_iri.ends_with("/wittLevelBound") => {}
            other => panic!("expected wittLevelBound for k={over}: {other:?}"),
        }
    }
}

#[test]
fn rejects_truncated_canonical_bytes() {
    // k=2 requires 1+3 = 4 bytes total. Anything shorter rejects.
    for short_len in 1..4 {
        let mut bytes = [0u8; 4];
        bytes[0] = 2;
        let err = RingElement::parse(&bytes[..short_len]).expect_err("must reject truncated");
        assert!(
            err.constraint_iri.ends_with("/validCanonicalBytes")
                || err.constraint_iri.ends_with("/wittLevelBound")
        );
    }
}

#[test]
fn large_input_is_unbounded_no_size_cap() {
    // ADR-060 removed the fixed input size cap. A large buffer is no
    // longer rejected for its size; the only ground for rejection is
    // structural — this all-zero buffer declares witt level 0 (1+1 = 2
    // bytes expected) yet carries far more, so it surfaces the structural
    // InvalidRingElement, NOT a size-cap failure.
    let large = alloc::vec![0u8; uor_addr::ring::RING_VALUE_MAX_BYTES + 1];
    match address(&large) {
        Err(AddressFailure::InvalidRingElement) => {}
        other => panic!("expected InvalidRingElement (no size cap): {other:?}"),
    }
}

#[test]
fn rejects_empty_input() {
    match address(&[]) {
        Err(AddressFailure::InvalidRingElement) => {}
        other => panic!("expected InvalidRingElement: {other:?}"),
    }
}

#[test]
fn rejects_wrong_payload_width_for_witt_level() {
    // k=0 requires 1+1 = 2 bytes. Supplying 3 bytes rejects.
    match address(&[0, 0x42, 0x00]) {
        Err(AddressFailure::InvalidRingElement) => {}
        other => panic!("expected InvalidRingElement: {other:?}"),
    }
    // k=3 requires 1+4 = 5 bytes. Supplying 4 rejects.
    match address(&[3, 0x01, 0x02, 0x03]) {
        Err(AddressFailure::InvalidRingElement) => {}
        other => panic!("expected InvalidRingElement: {other:?}"),
    }
}

extern crate alloc;
