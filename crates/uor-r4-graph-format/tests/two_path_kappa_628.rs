//! #628 — the two-path κ certificate (atlas `canonical_kappa_certificate`
//! discipline, ported): an INDEPENDENT serialization of the r4g1 header +
//! CID chain, written from the format documentation alone (header.rs §2
//! layout comment) and never calling the production writer's emission code,
//! must reproduce the production artifact byte-for-byte and κ-for-κ.
//!
//! Why this exists: verify-on-parse re-derives κ over whatever bytes the
//! writer produced, and CI's deterministic-rebuild reruns the SAME writer —
//! both are blind to the writer drifting from the documented canonical form.
//! This test is the second path. The mutation arm proves it can fail.

use uor_r4_graph_format::{ArtifactBuilder, SectionId};

// header.rs layout constants, restated FROM THE DOCUMENTATION (the module is
// private — restating them is the point: this path must not import the
// production layout).
const FORMAT_VERSION_MAJOR: u8 = 0;
const FORMAT_VERSION_MINOR: u8 = 0;

/// The independent writer: documentation-only reconstruction. Inputs are the
/// section (id, flags, payload) triples — the semantic content — and the
/// alignment; every byte below comes from the header.rs layout table, not
/// from ser.rs.
fn independent_serialize(alignment_log2: u8, sections: &[(u32, u32, Vec<u8>)]) -> Vec<u8> {
    let align = 1u64 << alignment_log2;
    let table_len = 88 + 16 * sections.len() as u64;
    // Section offsets: each aligned up from the running end.
    let mut offsets = Vec::new();
    let mut cursor = table_len;
    for (_, _, p) in sections {
        cursor = cursor.div_ceil(align) * align;
        offsets.push(cursor);
        cursor += p.len() as u64;
    }
    let total_len = cursor;
    let mut out = Vec::new();
    out.extend_from_slice(b"R4G1"); // 0..4 magic
    out.push(FORMAT_VERSION_MAJOR); // 4
    out.push(FORMAT_VERSION_MINOR); // 5
    out.push(0x01); // 6 endianness little
    out.push(alignment_log2); // 7
    out.extend_from_slice(&total_len.to_le_bytes()); // 8..16
    out.extend_from_slice(&(sections.len() as u32).to_le_bytes()); // 16..20
    out.extend_from_slice(&0u32.to_le_bytes()); // 20..24 flags
    out.extend_from_slice(&[0u8; 64]); // 24..88 artifact_cid + head_cid, zeroed
    for ((id, flags, p), off) in sections.iter().zip(&offsets) {
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&(*off as u32).to_le_bytes());
        out.extend_from_slice(&(p.len() as u32).to_le_bytes());
    }
    for ((_, _, p), off) in sections.iter().zip(&offsets) {
        while (out.len() as u64) < *off {
            out.push(0);
        }
        out.extend_from_slice(p);
    }
    // CID chain per the documented convention: head_cid = blake3(HEAD body);
    // artifact_cid = blake3(bytes[56..]) with its own field zeroed.
    let head = sections
        .iter()
        .find(|(id, _, _)| *id == SectionId::HEAD.0)
        .map(|(_, _, p)| p.as_slice())
        .expect("HEAD section");
    let head_cid = blake3::hash(head);
    out[56..88].copy_from_slice(head_cid.as_bytes());
    let artifact_cid = blake3::hash(&out[56..]);
    out[24..56].copy_from_slice(artifact_cid.as_bytes());
    out
}

#[test]
fn two_path_kappa_certificate() {
    // A small artifact with two sections through the PRODUCTION writer.
    let head_payload = b"two-path head body #628".to_vec();
    let other_payload = vec![0xABu8; 37];
    let other_id = SectionId::NODE; // a real registered section (stage1.rs pattern)
    let mut b = ArtifactBuilder::new(4);
    b.add_section(SectionId::HEAD, 0, &head_payload);
    b.add_section(other_id, 7, &other_payload);
    let production = b.build().expect("production serialize");

    // The INDEPENDENT path, same semantic inputs.
    let ours = independent_serialize(
        4,
        &[
            (SectionId::HEAD.0, 0, head_payload.clone()),
            (other_id.0, 7, other_payload.clone()),
        ],
    );

    // The certificate: bytes equal, therefore κ equal — asserted separately
    // so a κ-only report still names which one broke.
    let prod_kappa = format!("blake3:{}", blake3::hash(&production).to_hex());
    let our_kappa = format!("blake3:{}", blake3::hash(&ours).to_hex());
    assert_eq!(
        ours, production,
        "independent bytes diverge from production"
    );
    assert_eq!(our_kappa, prod_kappa, "two-path κ certificate broken");

    // The instrument can fail: a one-byte semantic mutation must break the
    // certificate (different payload ⇒ different bytes ⇒ different κ).
    let mut mutated = head_payload.clone();
    mutated[0] ^= 0x01;
    let drifted = independent_serialize(
        4,
        &[
            (SectionId::HEAD.0, 0, mutated),
            (other_id.0, 7, other_payload),
        ],
    );
    assert_ne!(
        drifted, production,
        "mutation invisible — certificate vacuous"
    );
}
