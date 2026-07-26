//! Stage-1 validation, canonical serialization, and CID tests for the
//! R4G1 container. Every stage-1 invariant of RFC §6 has at least one
//! rejection test; positive paths round-trip through the canonical
//! serializer and `GraphView`.
//!
//! Positive-path fixtures carry stage-2-valid section payloads (a real
//! 224-byte HEAD, matching NODE/EMIT), because `GraphView::parse` now
//! runs stage-2 semantic validation whenever a HEAD section is present.
//! Payload builders live in `tests/common/`; stage-2 semantics proper
//! are covered in `tests/stage2.rs`.

mod common;

use common::{head_payload, node_section, storage_section, HeadFields, NodeFields};
use uor_r4_graph_format::{
    ArtifactBuilder, Depth, FormatError, GraphView, NodeId, Radius, ScoreQ, SectionId,
    SectionOffset, TokenId, HEADER_LEN,
};

/// Assemble a container with an honest `total_len` but otherwise
/// unchecked contents: raw 16-byte entries taken as given, `body`
/// appended immediately after the table. Lets each test violate exactly
/// one invariant. CIDs are left zeroed (stage 1 does not check them).
fn assemble(
    alignment_log2: u8,
    flags: u32,
    count_override: Option<u32>,
    entries: &[[u32; 4]],
    body: &[u8],
) -> Vec<u8> {
    let count = count_override.unwrap_or(entries.len() as u32);
    let total_len = (HEADER_LEN + 16 * entries.len() + body.len()) as u64;
    let mut out = Vec::new();
    out.extend_from_slice(b"R4G1");
    out.push(0); // major (draft line, RFC §8 version gate)
    out.push(0); // minor
    out.push(0x01); // endianness marker
    out.push(alignment_log2);
    out.extend_from_slice(&total_len.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&[0u8; 32]); // artifact_cid
    out.extend_from_slice(&[0u8; 32]); // head_cid
    for entry in entries {
        for word in entry {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out.extend_from_slice(body);
    out
}

/// Structurally valid two-section container: HEAD@[120,124),
/// CODE@[128,132), alignment 8.
fn valid_fixture() -> Vec<u8> {
    assemble(
        3,
        0,
        None,
        &[
            [0x01, 0, 120, 4], // HEAD
            [0x02, 0, 128, 4], // CODE
        ],
        &[1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8],
    )
}

/// Sample HEAD payload declaring a single node (matches `sample_node`).
fn sample_head() -> Vec<u8> {
    head_payload(&HeadFields {
        node_count: 1,
        ..HeadFields::default()
    })
}

/// Sample NODE payload: one zeroed 30-byte PackedNode (all ranges empty,
/// depth 0 — valid against `sample_head`).
fn sample_node() -> Vec<u8> {
    node_section(&[NodeFields::default()])
}

/// Canonically serialized sample artifact (HEAD/NODE/ROUT/EMIT + one
/// unknown optional section), valid CIDs included. All section payloads
/// are stage-2-valid so `GraphView::parse` accepts the whole artifact.
/// The zeroed 64-byte ROUT covers the zeroed node's W=8 prototype/mask
/// word extents and validates as a lone HALT op.
fn build_sample() -> Vec<u8> {
    let mut b = ArtifactBuilder::new(3);
    // Deliberately unsorted insertion order.
    b.add_section(SectionId::NODE, 7, &sample_node());
    b.add_section(SectionId::HEAD, 0, &sample_head());
    b.add_section(SectionId::ROUT, 0, &[0u8; 64]);
    b.add_section(SectionId::EMIT, 0, &storage_section(1, 0, 0, &[]));
    b.add_section(SectionId(0x8000_0042), 0, b"opaque");
    b.build().expect("sample artifact must build")
}

fn err_of(bytes: &[u8]) -> FormatError {
    match GraphView::parse(bytes) {
        Ok(_) => panic!("expected rejection, but the artifact parsed"),
        Err(e) => e,
    }
}

// ── Positive paths ────────────────────────────────────────────────────

#[test]
fn round_trip_sections() {
    let bytes = build_sample();
    let view = GraphView::parse(&bytes).expect("canonical bytes must validate");

    assert_eq!(view.section(SectionId::HEAD), Some(&sample_head()[..]));
    assert_eq!(view.section(SectionId::NODE), Some(&sample_node()[..]));
    assert_eq!(view.section(SectionId::ROUT), Some(&[0u8; 64][..]));
    assert_eq!(
        view.section(SectionId::EMIT),
        Some(&storage_section(1, 0, 0, &[])[..])
    );
    assert_eq!(view.section(SectionId(0x8000_0042)), Some(&b"opaque"[..]));
    assert_eq!(view.section(SectionId::EDGE), None);

    let header = view.header();
    assert_eq!(header.major, 0);
    assert_eq!(header.minor, 0);
    assert_eq!(header.alignment_log2, 3);
    assert_eq!(header.section_count, 5);
    assert_eq!(header.total_len, bytes.len() as u64);
    assert_eq!(view.as_bytes(), &bytes[..]);

    // Iteration is in canonical (sorted by id) order; the NODE entry
    // keeps its per-entry flags.
    let sections: Vec<_> = view.sections().collect();
    assert_eq!(sections.len(), 5);
    let ids: Vec<u32> = sections.iter().map(|s| s.id.raw()).collect();
    assert_eq!(ids, [0x01, 0x03, 0x05, 0x06, 0x8000_0042]);
    assert_eq!(sections[1].flags, 7);

    // Every section offset is aligned to 1 << alignment_log2.
    for s in &sections {
        let off = s.payload.as_ptr() as usize - bytes.as_ptr() as usize;
        assert_eq!(off % 8, 0, "section {:08x} misaligned", s.id.raw());
    }
}

#[test]
fn deterministic_canonical_bytes() {
    let mut a = ArtifactBuilder::new(3);
    a.add_section(SectionId::HEAD, 0, b"head-body-v1");
    a.add_section(SectionId::NODE, 7, &[0u8; 13]);
    a.add_section(SectionId::EMIT, 0, b"emit");
    a.add_section(SectionId(0x8000_0042), 0, b"opaque");
    let bytes_a1 = a.build().unwrap();
    let bytes_a2 = a.build().unwrap();

    // Same content, different insertion order.
    let mut b = ArtifactBuilder::new(3);
    b.add_section(SectionId(0x8000_0042), 0, b"opaque");
    b.add_section(SectionId::EMIT, 0, b"emit");
    b.add_section(SectionId::NODE, 7, &[0u8; 13]);
    b.add_section(SectionId::HEAD, 0, b"head-body-v1");
    let bytes_b = b.build().unwrap();

    assert_eq!(bytes_a1, bytes_a2, "same builder, two builds");
    assert_eq!(bytes_a1, bytes_b, "insertion order must not matter");
}

#[test]
fn verify_cids_ok() {
    let bytes = build_sample();
    let view = GraphView::parse(&bytes).unwrap();
    view.verify_cids().expect("freshly built CIDs verify");
}

#[test]
fn tokenizer_cid_mismatch_detected() {
    let bytes = build_sample();
    let view = GraphView::parse(&bytes).unwrap();
    let dummy_tokenizer = b"dummy tokenizer binary payload for testing";
    let actual_hash = blake3::hash(dummy_tokenizer);
    assert_ne!(
        view.head().unwrap().tokenizer_cid().0,
        *actual_hash.as_bytes()
    );
    assert_eq!(
        view.verify_tokenizer_cid(dummy_tokenizer),
        Err(FormatError::TokenizerCidMismatch)
    );
}

#[test]
fn head_cid_tamper_detected() {
    let bytes = build_sample();
    let view = GraphView::parse(&bytes).unwrap();
    let head_off =
        view.section(SectionId::HEAD).unwrap().as_ptr() as usize - bytes.as_ptr() as usize;

    let mut tampered = bytes.clone();
    tampered[head_off] ^= 0xFF;
    let view = GraphView::parse(&tampered).unwrap();
    assert_eq!(view.verify_cids(), Err(FormatError::HeadCidMismatch));
}

#[test]
fn artifact_cid_tamper_detected() {
    let bytes = build_sample();
    let view = GraphView::parse(&bytes).unwrap();
    let opaque_off =
        view.section(SectionId(0x8000_0042)).unwrap().as_ptr() as usize - bytes.as_ptr() as usize;

    // Flip a payload byte in the unknown optional section: stage-2 does
    // not inspect it, head_cid still matches, artifact_cid does not.
    let mut tampered = bytes.clone();
    tampered[opaque_off + 1] ^= 0xFF;
    let view = GraphView::parse(&tampered).unwrap();
    assert_eq!(view.verify_cids(), Err(FormatError::ArtifactCidMismatch));

    // Flip a section-table flags byte: same result.
    let mut tampered = bytes.clone();
    tampered[HEADER_LEN + 4] ^= 0xFF;
    let view = GraphView::parse(&tampered).unwrap();
    assert_eq!(view.verify_cids(), Err(FormatError::ArtifactCidMismatch));
}

#[test]
fn unknown_optional_section_skipped() {
    let head = head_payload(&HeadFields::default());
    let mut body = head.clone();
    body.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
    let bytes = assemble(
        3,
        0,
        None,
        &[
            [0x01, 0, 120, 224],      // HEAD
            [0x8000_0040, 0, 344, 4], // unknown optional
        ],
        &body,
    );
    let view = GraphView::parse(&bytes).expect("unknown optional section must be skipped");
    assert_eq!(view.section(SectionId::HEAD), Some(&head[..]));
    // Retained as opaque bytes, reachable by raw ID.
    assert_eq!(
        view.section(SectionId(0x8000_0040)),
        Some(&[0xAA, 0xBB, 0xCC, 0xDD][..])
    );
    assert_eq!(view.sections().count(), 2);
}

#[test]
fn unknown_optional_feature_bits_ignored() {
    // Bits 16..=31 are the optional feature space.
    let head = head_payload(&HeadFields::default());
    let bytes = assemble(3, 0x1234_0000, None, &[[0x01, 0, 104, 224]], &head);
    let view = GraphView::parse(&bytes).expect("optional feature bits must be ignored");
    assert_eq!(view.header().flags, 0x1234_0000);
}

// ── Stage-1 rejection cases (RFC §6) ──────────────────────────────────

#[test]
fn reject_bad_magic() {
    let mut bytes = valid_fixture();
    bytes[0] = b'X';
    assert_eq!(err_of(&bytes), FormatError::BadMagic);
}

#[test]
fn reject_unsupported_major_version() {
    // Draft-line gate (RFC §8): pre-freeze readers accept only major 0;
    // a stable-line claim (major >= 1) must be rejected until widths freeze.
    let mut bytes = valid_fixture();
    bytes[4] = 1;
    assert_eq!(err_of(&bytes), FormatError::UnsupportedMajorVersion(1));
    bytes[4] = 2;
    assert_eq!(err_of(&bytes), FormatError::UnsupportedMajorVersion(2));
}

#[test]
fn reject_bad_endianness_marker() {
    let mut bytes = valid_fixture();
    bytes[6] = 0x00;
    assert_eq!(err_of(&bytes), FormatError::UnsupportedEndianness(0x00));
}

#[test]
fn reject_unsupported_alignment() {
    let bytes = assemble(2, 0, None, &[[0x01, 0, 104, 4]], &[1, 2, 3, 4]);
    assert_eq!(err_of(&bytes), FormatError::UnsupportedAlignment(2));
}

#[test]
fn reject_total_len_mismatch() {
    let mut bytes = valid_fixture();
    let declared = bytes.len() as u64 + 8;
    bytes[8..16].copy_from_slice(&declared.to_le_bytes());
    assert_eq!(
        err_of(&bytes),
        FormatError::TotalLenMismatch {
            declared,
            actual: declared - 8,
        }
    );
}

#[test]
fn reject_truncated_header() {
    let bytes = valid_fixture();
    assert_eq!(err_of(&bytes[..80]), FormatError::TruncatedHeader);
}

#[test]
fn reject_section_table_out_of_bounds() {
    // Declares two entries but carries only one.
    let bytes = assemble(3, 0, Some(2), &[[0x01, 0, 104, 4]], &[]);
    assert_eq!(err_of(&bytes), FormatError::SectionTableOutOfBounds);
}

#[test]
fn reject_unsorted_section_ids() {
    let bytes = assemble(
        3,
        0,
        None,
        &[
            [0x03, 0, 120, 4], // NODE before HEAD: non-canonical order
            [0x01, 0, 128, 4],
        ],
        &[0u8; 12],
    );
    assert_eq!(err_of(&bytes), FormatError::SectionsNotSorted);
}

#[test]
fn reject_overlapping_sections() {
    let bytes = assemble(
        3,
        0,
        None,
        &[
            [0x01, 0, 120, 8], // HEAD [120,128)
            [0x02, 0, 120, 8], // CODE [120,128) — identical range
        ],
        &[0u8; 8],
    );
    assert_eq!(err_of(&bytes), FormatError::SectionsOverlap);
}

#[test]
fn reject_misaligned_section() {
    let bytes = assemble(3, 0, None, &[[0x01, 0, 121, 4]], &[1, 2, 3, 4]);
    assert_eq!(err_of(&bytes), FormatError::SectionMisaligned);
}

#[test]
fn reject_offset_length_overflow() {
    // offset + length overflows u32 even though both are in range and
    // the offset is aligned.
    let bytes = assemble(3, 0, None, &[[0x01, 0, 0xFFFF_FF00, 0xFFFF_FF00]], &[]);
    assert_eq!(err_of(&bytes), FormatError::OffsetOverflow);
}

#[test]
fn reject_section_out_of_bounds() {
    // No u32 overflow, but the range runs past total_len.
    let bytes = assemble(3, 0, None, &[[0x01, 0, 104, 0x1000]], &[]);
    assert_eq!(err_of(&bytes), FormatError::SectionOutOfBounds);
}

#[test]
fn reject_unknown_mandatory_section() {
    let bytes = assemble(
        3,
        0,
        None,
        &[
            [0x01, 0, 120, 4],
            [0x40, 0, 128, 4], // unknown, OPTIONAL_BIT clear => mandatory
        ],
        &[0u8; 12],
    );
    assert_eq!(err_of(&bytes), FormatError::UnknownMandatorySection(0x40));
}

#[test]
fn reject_unknown_mandatory_feature_bit() {
    // Bit 0 lies in the mandatory feature space and is undefined.
    let bytes = assemble(3, 0x0000_0001, None, &[[0x01, 0, 104, 4]], &[1, 2, 3, 4]);
    assert_eq!(err_of(&bytes), FormatError::UnknownMandatoryFeature(1));
}

// ── Serializer errors ─────────────────────────────────────────────────

#[test]
fn builder_requires_head() {
    let mut b = ArtifactBuilder::new(3);
    b.add_section(SectionId::NODE, 0, &[0u8; 4]);
    assert_eq!(b.build(), Err(FormatError::MissingHead));
}

#[test]
fn builder_rejects_duplicate_sections() {
    let mut b = ArtifactBuilder::new(3);
    b.add_section(SectionId::HEAD, 0, b"a");
    b.add_section(SectionId::HEAD, 0, b"b");
    assert_eq!(
        b.build(),
        Err(FormatError::DuplicateSection(SectionId::HEAD))
    );
}

#[test]
fn builder_rejects_unknown_mandatory_id() {
    let mut b = ArtifactBuilder::new(3);
    b.add_section(SectionId::HEAD, 0, b"head");
    b.add_section(SectionId(0x40), 0, b"nope");
    assert_eq!(b.build(), Err(FormatError::UnknownMandatorySection(0x40)));
}

// ── Newtype smoke ─────────────────────────────────────────────────────

#[test]
fn newtype_smoke() {
    assert_eq!(ScoreQ::from_raw(0x0001_0000).raw(), 0x0001_0000);
    assert!(Depth(1) < Depth(2));
    assert!(Radius(7) > Radius(3));
    let _ = (NodeId(0), TokenId(0), SectionOffset(0));

    // RFC §3 mandatory column, plus the unknown-ID OPTIONAL_BIT policy.
    assert!(SectionId::HEAD.mandatory());
    assert!(SectionId::CODE.mandatory());
    assert!(SectionId::NODE.mandatory());
    assert!(SectionId::EDGE.mandatory());
    assert!(SectionId::ROUT.mandatory());
    assert!(SectionId::EMIT.mandatory());
    assert!(SectionId::PROV.mandatory());
    assert!(!SectionId::EXCT.mandatory());
    assert!(!SectionId::CERT.mandatory());
    assert!(!SectionId::PTCH.mandatory());
    assert!(!SectionId::SECT.mandatory());
    assert!(!SectionId::RTNX.mandatory());
    assert!(SectionId(0x40).mandatory());
    assert!(!SectionId(0x8000_0040).mandatory());
    assert!(SectionId::SECT.is_known());
    assert!(SectionId::RTNX.is_known());
    assert!(!SectionId(0x0D).is_known());

    assert!(!FormatError::BadMagic.to_string().is_empty());
}
