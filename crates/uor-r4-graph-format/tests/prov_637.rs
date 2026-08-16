//! PROV/1 format-freeze tests (#637 phase 1): round-trip, bounds, order,
//! duplicate, and tamper coverage for `uor_r4_graph_format::Prov` /
//! `build_prov`. Format-only — no `stage2`/compiler wiring is exercised
//! here (that is #637 phases 2/3).

use uor_r4_graph_format::{build_prov, NotAProduct, Prov, ProvComponents, PROV_HEADER_LEN};

const fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

const ASCENDING_ROOTS: [[u8; 32]; 3] = [digest(0x10), digest(0x20), digest(0x30)];
const UNSORTED_ROOTS: [[u8; 32]; 3] = [digest(0x30), digest(0x10), digest(0x20)];
const DUPLICATE_ROOTS: [[u8; 32]; 3] = [digest(0x10), digest(0x20), digest(0x10)];

fn full_components() -> ProvComponents<'static> {
    ProvComponents {
        source_manifest_kappa: Some(digest(0x01)),
        geometry_digest: Some(digest(0x02)),
        tokenizer_adapter_digest: Some(digest(0x03)),
        attention_operator_digest: Some(digest(0x04)),
        dense_operator_digest: Some(digest(0x05)),
        license: Some("MIT"),
        evidence_roots: &ASCENDING_ROOTS,
    }
}

#[test]
fn round_trips_all_components_present() {
    let bytes = build_prov(&full_components()).expect("build");
    let prov = Prov::parse(&bytes).expect("parse");
    assert_eq!(prov.source_manifest_kappa(), Some(digest(0x01)));
    assert_eq!(prov.geometry_digest(), Some(digest(0x02)));
    assert_eq!(prov.tokenizer_adapter_digest(), Some(digest(0x03)));
    assert_eq!(prov.attention_operator_digest(), Some(digest(0x04)));
    assert_eq!(prov.dense_operator_digest(), Some(digest(0x05)));
    assert_eq!(prov.license(), Some("MIT"));
    assert_eq!(prov.evidence_root_count(), 3);
    let roots: Vec<_> = prov.evidence_roots().copied().collect();
    assert_eq!(roots, vec![digest(0x10), digest(0x20), digest(0x30)]);
}

#[test]
fn round_trips_all_components_absent() {
    let components = ProvComponents::default();
    let bytes = build_prov(&components).expect("build empty");
    assert_eq!(bytes.len(), PROV_HEADER_LEN);
    let prov = Prov::parse(&bytes).expect("parse empty");
    assert_eq!(prov.source_manifest_kappa(), None);
    assert_eq!(prov.geometry_digest(), None);
    assert_eq!(prov.tokenizer_adapter_digest(), None);
    assert_eq!(prov.attention_operator_digest(), None);
    assert_eq!(prov.dense_operator_digest(), None);
    assert_eq!(prov.license(), None);
    assert_eq!(prov.evidence_root_count(), 0);
    assert!(prov.evidence_roots().next().is_none());
}

#[test]
fn builder_sorts_evidence_roots_and_output_round_trips_ascending() {
    let components = ProvComponents {
        evidence_roots: &UNSORTED_ROOTS,
        ..full_components()
    };
    let bytes = build_prov(&components).expect("build unsorted input");
    let prov = Prov::parse(&bytes).expect("parse");
    let roots: Vec<_> = prov.evidence_roots().copied().collect();
    assert_eq!(roots, vec![digest(0x10), digest(0x20), digest(0x30)]);
}

#[test]
fn builder_rejects_duplicate_evidence_roots() {
    let components = ProvComponents {
        evidence_roots: &DUPLICATE_ROOTS,
        ..full_components()
    };
    assert!(build_prov(&components).is_err());
}

#[test]
fn builder_rejects_non_ascii_license() {
    let components = ProvComponents {
        license: Some("MIT\u{2764}"), // non-ASCII heart
        ..full_components()
    };
    assert!(build_prov(&components).is_err());
}

// --- bounds ---

#[test]
fn parse_rejects_too_short_header() {
    let bytes = vec![0u8; PROV_HEADER_LEN - 1];
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "shorter than its header"));
}

#[test]
fn parse_rejects_license_length_past_end_of_section() {
    let mut bytes = build_prov(&ProvComponents {
        license: Some("MIT"),
        ..ProvComponents::default()
    })
    .expect("build");
    // Header declares license_len at offset 12..16; inflate it past the
    // actual remaining bytes without extending the buffer.
    let inflated = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) + 1_000_000;
    bytes[12..16].copy_from_slice(&inflated.to_le_bytes());
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "out of bounds"));
}

#[test]
fn parse_rejects_evidence_root_count_past_end_of_section() {
    let mut bytes = build_prov(&full_components()).expect("build");
    let len = bytes.len();
    // Truncate one byte off the last evidence root so the declared count
    // no longer matches the available bytes exactly.
    bytes.truncate(len - 1);
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "out of bounds"));
}

// --- order / duplicate (hand-built bytes: the builder always emits
// canonical order, so exercising the reader's own check requires
// constructing bytes directly rather than going through `build_prov`) ---

/// Build a minimal valid PROV/1 header (all components absent) followed
/// by the given raw evidence-root bytes, with `evidence_root_count` set
/// to `count`. Lets a test hand-craft an out-of-order or duplicate tail
/// that the builder itself would never produce.
fn header_with_raw_roots(count: u32, root_bytes: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0u8; PROV_HEADER_LEN];
    bytes[0..4].copy_from_slice(b"PRV1");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    // presence = 0, reserved = 0, license_len = 0 (all already zeroed).
    bytes[8..12].copy_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(root_bytes);
    bytes
}

fn is_reason(err: &NotAProduct, needle: &str) -> bool {
    std::format!("{err}").contains(needle)
}

#[test]
fn parse_rejects_out_of_order_evidence_roots() {
    let mut roots = Vec::new();
    roots.extend_from_slice(&digest(0x20));
    roots.extend_from_slice(&digest(0x10)); // descending: out of order
    let bytes = header_with_raw_roots(2, &roots);
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "canonically sorted"));
}

#[test]
fn parse_rejects_duplicate_evidence_roots() {
    let mut roots = Vec::new();
    roots.extend_from_slice(&digest(0x10));
    roots.extend_from_slice(&digest(0x10)); // exact duplicate
    let bytes = header_with_raw_roots(2, &roots);
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "canonically sorted"));
}

// --- tamper ---

#[test]
fn parse_rejects_bad_magic() {
    let mut bytes = build_prov(&ProvComponents::default()).expect("build");
    bytes[0] = b'X';
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "magic"));
}

#[test]
fn parse_rejects_unsupported_version() {
    let mut bytes = build_prov(&ProvComponents::default()).expect("build");
    bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "version"));
}

#[test]
fn parse_rejects_nonzero_reserved_byte() {
    let mut bytes = build_prov(&ProvComponents::default()).expect("build");
    bytes[7] = 0x01;
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "reserved"));
}

#[test]
fn parse_rejects_nonzero_unused_presence_bits() {
    let mut bytes = build_prov(&ProvComponents::default()).expect("build");
    bytes[6] = 0b1100_0000; // bits 6-7 are reserved
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "reserved"));
}

#[test]
fn parse_rejects_presence_bit_set_with_zeroed_slot() {
    let mut bytes = build_prov(&ProvComponents::default()).expect("build");
    bytes[6] = 0b0000_0001; // claim source_manifest_kappa present
                            // ...but leave its digest slot (offset 16..48) all-zero.
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "presence"));
}

#[test]
fn parse_rejects_presence_bit_clear_with_nonzero_slot() {
    let mut bytes = build_prov(&full_components()).expect("build");
    bytes[6] &= !0b0000_0001; // clear the source_manifest_kappa presence bit
                              // ...but its digest slot (offset 16..48) is still non-zero.
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "presence"));
}

#[test]
fn parse_rejects_non_ascii_license_bytes() {
    let mut bytes = header_with_raw_roots(0, &[]);
    // Declare a 1-byte license and set the presence bit, then splice in
    // a non-ASCII byte directly (bypassing the builder's own check).
    bytes[6] = 0b0010_0000;
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes.insert(PROV_HEADER_LEN, 0xFF);
    let err = Prov::parse(&bytes).unwrap_err();
    assert!(is_reason(&err, "ASCII"));
}
