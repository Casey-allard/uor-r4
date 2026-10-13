//! Shared byte-level canonicalize disciplines for the five categorical
//! operations on the Atlas image inside E₈ per wiki [ADR-061] §(3).
//!
//! Each function operates on the **raw digest bytes** of operand
//! κ-labels — the lowercase-hex digest body, decoded back to its 32- or
//! 64-byte raw form — and emits canonical-form bytes that the
//! composition shape's ψ-pipeline folds into the composed κ-label via
//! the bound σ-axis.
//!
//! Per CA-5, these are uor-addr's realization commitments under
//! ADR-061 §(3). The framework names each operation's algebraic
//! structure (commutative binary product, 2-element equivalence
//! relation, 2-class partition, 24-element equivalence relation,
//! identity); the realization commits the specific byte-level relation
//! that implements that algebraic structure.
//!
//! [ADR-061]: https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061

#![cfg(feature = "alloc")]

use alloc::vec::Vec;

use crate::composition::CompositionFailure;
use crate::label::KappaLabel;

extern crate alloc;

// ─── Operand parsing ─────────────────────────────────────────────────

/// Decode a κ-label's raw digest bytes (the lowercase-hex body after
/// the σ-axis prefix). Returns the σ-axis name (the prefix without
/// trailing `:`) and the raw digest bytes (32 or 64 bytes).
pub fn decode_operand<const N: usize>(
    operand: &KappaLabel<N>,
) -> Result<(&str, Vec<u8>), CompositionFailure> {
    let axis = operand
        .sigma_axis()
        .ok_or(CompositionFailure::MalformedOperand)?;
    let hex_digest = operand
        .sigma_axis_digest_hex()
        .ok_or(CompositionFailure::MalformedOperand)?;
    if hex_digest.len() % 2 != 0 {
        return Err(CompositionFailure::MalformedOperand);
    }
    let mut raw = Vec::with_capacity(hex_digest.len() / 2);
    let bytes = hex_digest.as_bytes();
    for pair in bytes.chunks_exact(2) {
        let hi = hex_nibble(pair[0]).ok_or(CompositionFailure::MalformedOperand)?;
        let lo = hex_nibble(pair[1]).ok_or(CompositionFailure::MalformedOperand)?;
        raw.push((hi << 4) | lo);
    }
    Ok((axis, raw))
}

/// Lowercase-hex nibble decoder. Returns `None` for any non-lowercase-hex
/// ASCII byte. (Pipeline-emitted κ-labels are lowercase-hex per
/// ADR-058's emission discipline.)
fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + b - b'a'),
        _ => None,
    }
}

/// Validate that an operand's σ-axis matches the expected axis (per
/// CA-3 σ-axis homogeneity).
pub fn check_axis(
    operand_axis: &str,
    expected_axis: &'static str,
) -> Result<(), CompositionFailure> {
    if operand_axis == expected_axis {
        Ok(())
    } else {
        // Convert the operand's borrowed axis name to a static str for
        // the failure variant. We compare against the five admissible
        // axes; anything else is rejected as MalformedOperand.
        let static_axis = match operand_axis {
            "sha256" => "sha256",
            "blake3" => "blake3",
            "sha3-256" => "sha3-256",
            "keccak256" => "keccak256",
            "sha512" => "sha512",
            _ => return Err(CompositionFailure::MalformedOperand),
        };
        Err(CompositionFailure::OperandSigmaAxisMismatch {
            expected_axis,
            operand_axis: static_axis,
        })
    }
}

// ─── CS-G2 — commutative binary product (lex-min-first ordering) ──────

/// CS-G2 canonicalize discipline: the commutative binary product of
/// two operand κ-labels. Per wiki ADR-061 §(3) the framework names the
/// algebraic structure (commutativity at the C-level per ADR-059); the
/// realization commits lex-min-first ordering as the byte-level rule
/// realizing that algebra.
///
/// Returns the concatenation `lo || hi` of the two operand byte
/// representations, where `(lo, hi) = (min(a, b), max(a, b))` under
/// bytewise lexicographic order. Commutativity is structural:
/// `canonicalize_g2(a, b)` and `canonicalize_g2(b, a)` produce
/// byte-identical canonical forms.
pub fn canonicalize_g2<const N: usize>(left: &KappaLabel<N>, right: &KappaLabel<N>) -> Vec<u8> {
    let l = left.as_bytes();
    let r = right.as_bytes();
    let mut out = Vec::with_capacity(N + N);
    if l <= r {
        out.extend_from_slice(l);
        out.extend_from_slice(r);
    } else {
        out.extend_from_slice(r);
        out.extend_from_slice(l);
    }
    out
}

// ─── CS-F4 — ± involution quotient (bitwise-complement mirror) ────────

/// CS-F4 canonicalize discipline: the 2-element equivalence relation
/// on operands under the ± mirror involution. Per wiki ADR-061 §(3)
/// the framework names the algebraic structure (a 2-element
/// equivalence relation per ADR-059); the realization commits
/// bitwise-complement of the raw digest as the partition-inducing
/// mirror.
///
/// The canonical representative is the lex-min of the 2-element class:
/// compare the operand's raw digest with its bitwise-complement; the
/// canonical form is the σ-axis prefix concatenated with the lex-min
/// raw digest re-encoded as lowercase hex.
pub fn canonicalize_f4<const N: usize>(
    operand: &KappaLabel<N>,
) -> Result<Vec<u8>, CompositionFailure> {
    let (axis, raw) = decode_operand(operand)?;
    let complement: Vec<u8> = raw.iter().map(|b| !b).collect();
    let canon_raw: &[u8] = if raw[..] <= complement[..] {
        &raw[..]
    } else {
        &complement[..]
    };
    Ok(emit_canonical(axis, canon_raw))
}

// ─── CS-E6 — degree-partition filtration (mod-9 partition) ────────────

/// The two degree-partition tag values per ADR-059's 64:8 vertex
/// partition of the Atlas. degree-5 vertices outnumber degree-6 by 8:1.
const DEGREE_5_TAG: u8 = 0x05;
const DEGREE_6_TAG: u8 = 0x06;

/// CS-E6 canonicalize discipline: the 2-class partition with 8:1
/// population ratio per ADR-059. Per wiki ADR-061 §(3) the framework
/// names the algebraic structure (a 2-class partition with population
/// ratio 8:1 per ADR-059); the realization commits the partition
/// derived from `first_raw_digest_byte mod 9`.
///
/// The canonical form is `[degree_class_tag] || operand.as_bytes()` —
/// a one-byte tag (`0x05` for degree-5, `0x06` for degree-6) prepended
/// to the operand's full κ-label bytes. Total width = `N + 1` per
/// wiki ADR-061 §(2).
pub fn canonicalize_e6<const N: usize>(
    operand: &KappaLabel<N>,
) -> Result<Vec<u8>, CompositionFailure> {
    let (_axis, raw) = decode_operand(operand)?;
    if raw.is_empty() {
        return Err(CompositionFailure::MalformedOperand);
    }
    let tag = match raw[0] % 9 {
        0..=7 => DEGREE_5_TAG,
        8 => DEGREE_6_TAG,
        _ => unreachable!("u8 % 9 is in 0..=8"),
    };
    let mut out = Vec::with_capacity(1 + N);
    out.push(tag);
    out.extend_from_slice(operand.as_bytes());
    Ok(out)
}

// ─── CS-E7 — S₄-orbit augmentation (quarter-permutation orbit) ────────

/// The 24 permutations of S₄ as quarter-index arrays. Generated
/// lexicographically — the canonical-form lex-min property below
/// depends on enumerating every member of the orbit, not on the
/// enumeration order.
const S4_PERMUTATIONS: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// CS-E7 canonicalize discipline: the 24-element equivalence relation
/// on operands under the S₄ quarter-permutation orbit. Per wiki
/// ADR-061 §(3) the framework names the algebraic structure (a
/// 24-element equivalence relation per ADR-059); the realization
/// commits the S₄ action by quarter-permutation of the raw digest.
///
/// The raw digest is partitioned into 4 equal-width quarters; the 24
/// permutations of S₄ generate 24 candidate raw-digest byte sequences;
/// the canonical representative is the lex-min of the 24-element
/// orbit. The σ-axis prefix is preserved.
pub fn canonicalize_e7<const N: usize>(
    operand: &KappaLabel<N>,
) -> Result<Vec<u8>, CompositionFailure> {
    let (axis, raw) = decode_operand(operand)?;
    if raw.len() % 4 != 0 || raw.is_empty() {
        return Err(CompositionFailure::MalformedOperand);
    }
    let q = raw.len() / 4;
    let quarters: [&[u8]; 4] = [
        &raw[0..q],
        &raw[q..2 * q],
        &raw[2 * q..3 * q],
        &raw[3 * q..],
    ];

    let mut canon: Option<Vec<u8>> = None;
    for perm in S4_PERMUTATIONS.iter() {
        let mut candidate = Vec::with_capacity(raw.len());
        for &idx in perm.iter() {
            candidate.extend_from_slice(quarters[idx]);
        }
        match &canon {
            None => canon = Some(candidate),
            Some(current) if candidate < *current => canon = Some(candidate),
            _ => {}
        }
    }
    let canon_raw = canon.expect("S4_PERMUTATIONS is non-empty");
    Ok(emit_canonical(axis, &canon_raw))
}

// ─── CS-E8 — direct embedding (identity on canonical-form bytes) ──────

/// CS-E8 canonicalize discipline: the identity. Per wiki ADR-061 §(3)
/// the framework names the algebraic structure (the identity relation
/// per ADR-059 — every operand is its own equivalence class); the
/// realization commits identity on canonical-form bytes.
///
/// The composed κ-label is distinguished from the operand's κ-label
/// by realization-IRI provenance, not by digest bytes.
pub fn canonicalize_e8<const N: usize>(operand: &KappaLabel<N>) -> Vec<u8> {
    operand.as_bytes().to_vec()
}

// ─── Helpers ─────────────────────────────────────────────────────────

/// Re-emit `<axis>:<lowercase-hex-of-raw>` from the σ-axis name and
/// the raw digest bytes. Used by CS-F4 and CS-E7 to produce
/// canonical-form bytes for the composition's ψ-pipeline input.
fn emit_canonical(axis: &str, raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(axis.len() + 1 + 2 * raw.len());
    out.extend_from_slice(axis.as_bytes());
    out.push(b':');
    for &byte in raw {
        out.push(hex_lo(byte >> 4));
        out.push(hex_lo(byte & 0x0F));
    }
    out
}

/// Lowercase-hex digit for a nibble (`0..=15`). Matches the
/// foundation's ψ_9 σ-projection emission discipline.
fn hex_lo(nibble: u8) -> u8 {
    match nibble {
        0..=9 => b'0' + nibble,
        10..=15 => b'a' + (nibble - 10),
        _ => unreachable!("nibble is `& 0x0F`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label<const N: usize>(s: &str) -> KappaLabel<N> {
        KappaLabel::from_bytes(s.as_bytes()).expect("test label admits")
    }

    #[test]
    fn g2_is_commutative() {
        let a =
            label::<71>("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let b =
            label::<71>("sha256:1111111111111111111111111111111111111111111111111111111111111111");
        let ab = canonicalize_g2(&a, &b);
        let ba = canonicalize_g2(&b, &a);
        assert_eq!(ab, ba, "CS-G2 commutativity is structural");
        assert_eq!(ab.len(), 142, "G2 canonical form is 2N bytes");
    }

    #[test]
    fn f4_mirror_collapses() {
        // An operand and its mirror produce byte-identical canonical
        // forms (the 2-element class collapses to its lex-min member).
        let a =
            label::<71>("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        // The mirror (all-FF raw) is its bitwise complement.
        let m =
            label::<71>("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let ca = canonicalize_f4(&a).expect("a canonicalizes");
        let cm = canonicalize_f4(&m).expect("m canonicalizes");
        assert_eq!(ca, cm, "CS-F4 mirror collapses to lex-min representative");
    }

    #[test]
    fn e6_prepends_degree_tag() {
        let a =
            label::<71>("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let canon = canonicalize_e6(&a).expect("canonicalizes");
        assert_eq!(canon.len(), 72, "CS-E6 canonical form is N + 1 bytes");
        assert!(canon[0] == DEGREE_5_TAG || canon[0] == DEGREE_6_TAG);
        assert_eq!(&canon[1..], a.as_bytes());
    }

    #[test]
    fn e6_partition_distinguishes_classes() {
        // first_raw_digest_byte = 0x00 → 0x00 % 9 = 0 → degree-5
        let a =
            label::<71>("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let ca = canonicalize_e6(&a).expect("canonicalizes");
        assert_eq!(ca[0], DEGREE_5_TAG, "first byte 0x00 → degree-5");
        // first_raw_digest_byte = 0x08 → 0x08 % 9 = 8 → degree-6
        let b =
            label::<71>("sha256:0800000000000000000000000000000000000000000000000000000000000000");
        let cb = canonicalize_e6(&b).expect("canonicalizes");
        assert_eq!(cb[0], DEGREE_6_TAG, "first byte 0x08 → degree-6");
    }

    #[test]
    fn e7_preserves_width_and_collapses_orbit() {
        // CS-E7 canonical form is the lex-min over the 24-orbit; it
        // preserves the σ-axis prefix + digest width, and any
        // quarter-permutation of an operand lands on the same canonical
        // representative.
        let a =
            label::<71>("sha256:0102030405060708090a0b0c0d0e0f1011121314151617181920212223242526");
        let ca = canonicalize_e7(&a).expect("canonicalizes");
        assert_eq!(ca.len(), a.as_bytes().len(), "CS-E7 preserves width");
        // Re-canonicalizing the canonical form is a fixed point (the
        // lex-min member's orbit lex-min is itself).
        let ca_label = KappaLabel::<71>::from_bytes(&ca).expect("canonical is a label");
        let ca2 = canonicalize_e7(&ca_label).expect("canonicalizes");
        assert_eq!(ca, ca2, "CS-E7 lex-min is an orbit fixed point");
    }

    #[test]
    fn e8_is_identity() {
        let a =
            label::<71>("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let canon = canonicalize_e8(&a);
        assert_eq!(
            canon,
            a.as_bytes(),
            "CS-E8 is identity on canonical-form bytes"
        );
    }

    #[test]
    fn axis_check_admits_match() {
        assert!(check_axis("sha256", "sha256").is_ok());
    }

    #[test]
    fn axis_check_rejects_mismatch() {
        match check_axis("blake3", "sha256") {
            Err(CompositionFailure::OperandSigmaAxisMismatch {
                expected_axis,
                operand_axis,
            }) => {
                assert_eq!(expected_axis, "sha256");
                assert_eq!(operand_axis, "blake3");
            }
            _ => panic!("σ-axis mismatch must be reported"),
        }
    }
}
