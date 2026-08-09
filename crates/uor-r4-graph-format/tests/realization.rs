//! Conformance vectors for the representation-level R4G1 address.

mod common;

use common::{head_payload, node_section, storage_section, HeadFields, NodeFields};
use uor_r4_graph_format::{r4g1, ArtifactBuilder, SectionId};

fn sample(alignment_log2: u8, reverse_insertion: bool) -> Vec<u8> {
    let mut builder = ArtifactBuilder::new(alignment_log2);
    let head = head_payload(&HeadFields::default());
    let node = node_section(&[] as &[NodeFields]);
    let rout = [0u8; 64];
    let emit = storage_section(1, 0, 0, &[]);
    if reverse_insertion {
        builder.add_section(SectionId::EMIT, 0, &emit);
        builder.add_section(SectionId::ROUT, 0, &rout);
        builder.add_section(SectionId::NODE, 7, &node);
        builder.add_section(SectionId::HEAD, 0, &head);
    } else {
        builder.add_section(SectionId::HEAD, 0, &head);
        builder.add_section(SectionId::NODE, 7, &node);
        builder.add_section(SectionId::ROUT, 0, &rout);
        builder.add_section(SectionId::EMIT, 0, &emit);
    }
    builder.build().expect("stage-2-valid sample must build")
}

#[test]
fn equivalent_reserializations_share_the_realization_address() {
    let first = r4g1::address(&sample(3, false)).expect("valid sample");
    let second = r4g1::address(&sample(5, true)).expect("valid sample");

    assert_eq!(first.skeleton, second.skeleton);
    assert_eq!(first.artifact_kappa, second.artifact_kappa);
    assert_eq!(first.sections, second.sections);
}

#[test]
fn payload_changes_change_the_section_and_artifact_addresses() {
    let original = sample(3, false);
    let mut changed_builder = ArtifactBuilder::new(3);
    let head = head_payload(&HeadFields::default());
    let node = node_section(&[] as &[NodeFields]);
    let rout = [0u8; 64];
    let changed_emit = storage_section(2, 0, 0, &[]);
    changed_builder.add_section(SectionId::HEAD, 0, &head);
    changed_builder.add_section(SectionId::NODE, 7, &node);
    changed_builder.add_section(SectionId::ROUT, 0, &rout);
    changed_builder.add_section(SectionId::EMIT, 0, &changed_emit);
    let changed_bytes = changed_builder
        .build()
        .expect("stage-2-valid sample must build");

    let original = r4g1::address(&original).expect("valid sample");
    let changed = r4g1::address(&changed_bytes).expect("valid sample");
    assert_ne!(original.artifact_kappa, changed.artifact_kappa);
    assert_ne!(
        r4g1::section_kappa(&sample(3, false), SectionId::EMIT).expect("valid sample"),
        r4g1::section_kappa(&changed_bytes, SectionId::EMIT).expect("valid sample")
    );
}

#[test]
fn malformed_or_tampered_artifacts_are_rejected() {
    // Not a valid artifact -> not addressable.
    assert!(r4g1::address(b"not an R4G1 artifact").is_none());

    let mut tampered = sample(3, false);
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    // Flipping a content byte leaves the structure valid but breaks the
    // artifact CID, so it is likewise not addressable.
    assert!(r4g1::address(&tampered).is_none());
}
