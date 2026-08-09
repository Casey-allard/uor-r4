//! Stage-2 semantic validation tests (RFC §6 items 4–9). Every stage-2
//! invariant has at least one rejection test; the positive path builds a
//! full HEAD+NODE+EDGE+ROUT+EMIT artifact through the canonical
//! serializer and checks the typed accessors, the HEAD getters, and the
//! integrity CIDs.

mod common;

use common::{
    edge_section, head_payload, node_section, op_halt, op_jmp_fwd, op_leaf, op_test_popcount_le,
    storage_section, EdgeFields, HeadFields, NodeFields,
};
use uor_r4_graph_format::{
    ArtifactBuilder, BoundKind, Depth, EdgeKind, EdgePayloadField, FormatError, GraphView,
    KappaError, NodeId, Radius, RangeField, ScoreQ, SectionId, FEATURE_EDGE_ALGEBRA_V1, HEADER_LEN,
};

/// The happy-path packed node records: two nodes whose ranges resolve
/// against `valid_edges` (2 edges), `valid_emit` (8-byte remainder), and
/// `valid_rout` (64 bytes = 8 u64 words, so the W=8 prototype/mask
/// extents starting at word 0 resolve exactly).
fn valid_nodes() -> [NodeFields; 2] {
    [
        NodeFields {
            child_start: 0,
            child_len: 1,
            forward_start: 0,
            forward_len: 0,
            emission_start: 0,
            emission_len: 4,
            prototype_word_start: 0,
            mask_word_start: 0,
            radius: 5,
            depth: 1,
            flags: 0,
        },
        NodeFields {
            child_start: 0,
            child_len: 0,
            forward_start: 0,
            forward_len: 2,
            emission_start: 4,
            emission_len: 4,
            prototype_word_start: 0,
            mask_word_start: 0,
            radius: 3,
            depth: 2,
            flags: 0,
        },
    ]
}

/// The happy-path canonical edges: refinement and predictive-forward over
/// the same `(src, dst)` pair.
fn valid_edges() -> [EdgeFields; 2] {
    [
        EdgeFields {
            src: 0,
            dst: 1,
            score_q: 100,
            kind: 0,
            flags: 0,
            reserved: 0,
        },
        EdgeFields {
            src: 0,
            dst: 1,
            score_q: -50,
            kind: 2,
            flags: 0,
            reserved: 0,
        },
    ]
}

/// The happy-path ROUT section: TEST, JMP over the LEAF, LEAF with a
/// 4-byte shortlist range, HALT — then the trailing shortlist table,
/// padded to 64 bytes (8 u64 words) so the W=8 prototype/mask extents
/// the nodes declare resolve within the section.
fn valid_rout() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&op_test_popcount_le(0, 0xFFFF, 3));
    out.extend_from_slice(&op_jmp_fwd(0));
    out.extend_from_slice(&op_leaf(0, 4));
    out.extend_from_slice(&op_halt());
    out.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // shortlist table
    out.resize(64, 0); // trailing table padding: W-word extents resolve
    out
}

/// Mutable view of one artifact's section payloads; each rejection test
/// perturbs exactly one piece of the valid fixture.
struct Fixture {
    head: HeadFields,
    head_override: Option<Vec<u8>>,
    nodes: Option<Vec<u8>>,
    edges: Option<Vec<u8>>,
    rout: Option<Vec<u8>>,
    emit: Option<Vec<u8>>,
    exct: Option<Vec<u8>>,
}

impl Fixture {
    fn valid() -> Self {
        Self {
            head: HeadFields {
                node_count: 2,
                edge_count: 2,
                depth_count: 4,
                ..HeadFields::default()
            },
            head_override: None,
            nodes: Some(node_section(&valid_nodes())),
            edges: Some(edge_section(&valid_edges(), &[0, 1])),
            rout: Some(valid_rout()),
            emit: Some(storage_section(1, 2, 0, &[0u8; 8])),
            exct: None,
        }
    }

    fn build(&self) -> Vec<u8> {
        let mut b = ArtifactBuilder::new(3);
        let head = self
            .head_override
            .clone()
            .unwrap_or_else(|| head_payload(&self.head));
        b.add_section(SectionId::HEAD, 0, &head);
        if let Some(nodes) = &self.nodes {
            b.add_section(SectionId::NODE, 0, nodes);
        }
        if let Some(edges) = &self.edges {
            b.add_section(SectionId::EDGE, 0, edges);
        }
        if let Some(rout) = &self.rout {
            b.add_section(SectionId::ROUT, 0, rout);
        }
        if let Some(emit) = &self.emit {
            b.add_section(SectionId::EMIT, 0, emit);
        }
        if let Some(exct) = &self.exct {
            b.add_section(SectionId::EXCT, 0, exct);
        }
        b.build().expect("fixture serialization must succeed")
    }
}

fn err_of(bytes: &[u8]) -> FormatError {
    match GraphView::parse(bytes) {
        Ok(_) => panic!("expected rejection, but the artifact parsed"),
        Err(e) => e.reason,
    }
}

/// Assemble a container with an honest `total_len` but otherwise
/// unchecked contents (same shape as the stage-1 helper), for the
/// HEAD-less stage-1-only case the builder cannot produce.
fn assemble(entries: &[[u32; 4]], body: &[u8]) -> Vec<u8> {
    let total_len = (HEADER_LEN + 16 * entries.len() + body.len()) as u64;
    let mut out = Vec::new();
    out.extend_from_slice(b"R4G1");
    out.push(0); // major (draft line, RFC §8 version gate)
    out.push(0); // minor
    out.push(0x01); // endianness marker
    out.push(3); // alignment_log2
    out.extend_from_slice(&total_len.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // flags
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

// ── Positive paths ────────────────────────────────────────────────────

#[test]
fn full_fixture_parses_and_verifies() {
    let bytes = Fixture::valid().build();
    let view = GraphView::parse(&bytes).expect("valid stage-2 artifact must parse");
    assert_eq!(view.node_count(), Some(2));
    assert_eq!(view.edge_count(), Some(2));
    view.verify_cids().expect("freshly built CIDs verify");
}

#[test]
fn head_getters_round_trip() {
    let f = Fixture::valid();
    let bytes = f.build();
    let view = GraphView::parse(&bytes).unwrap();
    let head = view.head().expect("HEAD present");
    let want = &f.head;
    assert_eq!(head.teacher_cid().0, want.teacher_cid);
    assert_eq!(head.tokenizer_cid().0, want.tokenizer_cid);
    assert_eq!(
        head.corpus_construction_cid().0,
        want.corpus_construction_cid
    );
    assert_eq!(
        head.corpus_certification_cid().0,
        want.corpus_certification_cid
    );
    assert_eq!(head.hf_revision(), &want.hf_revision);
    assert_eq!(head.compiler_version_cid().0, want.compiler_version_cid);
    assert_eq!(head.max_frontier_width(), want.max_frontier_width);
    assert_eq!(head.max_candidates(), want.max_candidates);
    assert_eq!(head.signature_words(), want.signature_words);
    assert_eq!(head.shortlist_size(), want.shortlist_size);
    assert_eq!(head.max_emission_entries(), want.max_emission_entries);
    assert_eq!(head.max_program_steps(), want.max_program_steps);
    assert_eq!(head.node_count(), want.node_count);
    assert_eq!(head.edge_count(), want.edge_count);
    assert_eq!(head.depth_count(), want.depth_count);
    assert_eq!(head.fallback_policies(), want.fallback_policies);
    assert_eq!(head.reserved(), 0);
    assert_eq!(head.signature_bytes(), want.signature_bytes);
    assert_eq!(head.min_runtime_major(), want.min_runtime_major);
    assert_eq!(head.min_runtime_minor(), want.min_runtime_minor);
    assert_eq!(head.feature_bits_required(), want.feature_bits_required);
    assert_eq!(head.vocab_size(), want.vocab_size);
}

#[test]
fn typed_accessors_decode_on_demand() {
    let bytes = Fixture::valid().build();
    let view = GraphView::parse(&bytes).unwrap();

    let nodes: Vec<_> = view.nodes().collect();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].child_start, 0);
    assert_eq!(nodes[0].child_len, 1);
    assert_eq!(nodes[0].forward_start, 0);
    assert_eq!(nodes[0].forward_len, 0);
    assert_eq!(nodes[0].emission_start, 0);
    assert_eq!(nodes[0].emission_len, 4);
    assert_eq!(nodes[0].prototype_word_start, 0);
    assert_eq!(nodes[0].mask_word_start, 0);
    assert_eq!(nodes[0].radius, Radius(5));
    assert_eq!(nodes[0].depth, Depth(1));
    assert_eq!(nodes[0].flags, 0);
    assert_eq!(nodes[1].emission_start, 4);
    assert_eq!(nodes[1].forward_len, 2);
    assert_eq!(nodes[1].depth, Depth(2));
    assert_eq!(view.node(0), Some(nodes[0]));
    assert_eq!(view.node(1), Some(nodes[1]));
    assert_eq!(view.node(2), None);

    let edges: Vec<_> = view.edges().collect();
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].src, NodeId(0));
    assert_eq!(edges[0].dst, NodeId(1));
    assert_eq!(edges[0].score_q, ScoreQ::from_raw(100));
    assert_eq!(edges[0].kind, 0);
    assert_eq!(edges[1].src, NodeId(0));
    assert_eq!(edges[1].score_q, ScoreQ::from_raw(-50));
    assert_eq!(view.edge(0), Some(edges[0]));
    assert_eq!(view.edge(2), None);

    assert_eq!(view.reverse_edge_id(0), Some(0));
    assert_eq!(view.reverse_edge_id(1), Some(1));
    assert_eq!(view.reverse_edge_id(2), None);
}

#[test]
fn shared_node_multikind_round_trip() {
    let mut b = ArtifactBuilder::new(3);
    let head = HeadFields {
        node_count: 3,
        edge_count: 4,
        depth_count: 4,
        feature_bits_required: FEATURE_EDGE_ALGEBRA_V1,
        ..HeadFields::default()
    };
    let nodes = [
        NodeFields {
            child_start: 0,
            child_len: 1,
            forward_start: 0,
            forward_len: 0,
            emission_start: 0,
            emission_len: 0,
            prototype_word_start: 0,
            mask_word_start: 0,
            radius: 1,
            depth: 0,
            flags: 0,
        },
        NodeFields {
            child_start: 0,
            child_len: 0,
            forward_start: 0,
            forward_len: 1,
            emission_start: 0,
            emission_len: 0,
            prototype_word_start: 0,
            mask_word_start: 0,
            radius: 1,
            depth: 1,
            flags: 0,
        },
        NodeFields {
            child_start: 0,
            child_len: 0,
            forward_start: 1,
            forward_len: 3,
            emission_start: 0,
            emission_len: 0,
            prototype_word_start: 0,
            mask_word_start: 0,
            radius: 1,
            depth: 1,
            flags: 0,
        },
    ];
    let edges = [
        EdgeFields {
            src: 0,
            dst: 1,
            score_q: 0,
            kind: EdgeKind::RefinementAbstraction as u8,
            flags: 0,
            reserved: 0,
        },
        EdgeFields {
            src: 0,
            dst: 2,
            score_q: 7,
            kind: EdgeKind::Similarity as u8,
            flags: 0,
            reserved: 0,
        },
        EdgeFields {
            src: 0,
            dst: 2,
            score_q: 3,
            kind: EdgeKind::EvidenceProvenance as u8,
            flags: 0,
            reserved: 101,
        },
        EdgeFields {
            src: 1,
            dst: 2,
            score_q: 5,
            kind: EdgeKind::PredictiveForward as u8,
            flags: 0,
            reserved: 102,
        },
    ];
    let reverse = [0, 1, 2, 3];
    b.add_section(SectionId::HEAD, 0, &head_payload(&head));
    b.add_section(SectionId::NODE, 0, &node_section(&nodes));
    b.add_section(SectionId::EDGE, 0, &edge_section(&edges, &reverse));
    b.add_section(SectionId::ROUT, 0, &valid_rout());
    b.add_section(SectionId::EMIT, 0, &storage_section(1, 0, 0, &[]));
    let bytes = b.build().expect("build shared-node fixture");
    let view = GraphView::parse(&bytes).expect("parse shared-node fixture");
    let kinds: Vec<u8> = view.edges().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        vec![
            EdgeKind::RefinementAbstraction as u8,
            EdgeKind::Similarity as u8,
            EdgeKind::EvidenceProvenance as u8,
            EdgeKind::PredictiveForward as u8,
        ]
    );
    assert_eq!(view.reverse_edge_id(0), Some(0));
    assert_eq!(view.reverse_edge_id(1), Some(1));
    assert_eq!(view.reverse_edge_id(2), Some(2));
    assert_eq!(view.reverse_edge_id(3), Some(3));
}

#[test]
fn container_without_head_is_stage1_only() {
    // Single NODE section, no HEAD: stage 2 is skipped, typed accessors
    // report nothing, structural parsing still works.
    let bytes = assemble(&[[0x03, 0, 104, 4]], &[1, 2, 3, 4]);
    let view = GraphView::parse(&bytes).expect("stage-1-only container must parse");
    assert_eq!(view.head(), None);
    assert_eq!(view.node_count(), None);
    assert_eq!(view.edge_count(), None);
    assert_eq!(view.nodes().count(), 0);
    assert_eq!(view.edges().count(), 0);
    assert_eq!(view.node(0), None);
    assert_eq!(view.edge(0), None);
    assert_eq!(view.reverse_edge_id(0), None);
    assert_eq!(view.section(SectionId::NODE), Some(&[1, 2, 3, 4][..]));
    assert_eq!(view.verify_cids(), Err(KappaError::MissingHead));
}

// ── HEAD payload (RFC §4) ─────────────────────────────────────────────

#[test]
fn reject_head_too_short() {
    let mut f = Fixture::valid();
    f.head_override = Some(head_payload(&f.head)[..223].to_vec());
    assert_eq!(
        err_of(&f.build()),
        FormatError::HeadTooShort { actual: 223 }
    );
}

#[test]
fn reject_head_trailing_bytes() {
    // Draft-line policy: trailing bytes past the 224-byte prefix are an
    // error, not silently ignored.
    let mut f = Fixture::valid();
    let mut head = head_payload(&f.head);
    head.push(0);
    f.head_override = Some(head);
    assert_eq!(err_of(&f.build()), FormatError::HeadTooLong { actual: 225 });
}

// ── NODE/EDGE section presence and record counts (RFC §6 item 4) ──────

#[test]
fn reject_node_count_mismatch() {
    let mut f = Fixture::valid();
    f.head.node_count = 3; // NODE still carries 2 records (60 bytes)
    assert_eq!(
        err_of(&f.build()),
        FormatError::NodeCountMismatch {
            declared: 3,
            section_len: 60,
        }
    );
}

#[test]
fn reject_missing_node_section() {
    let mut f = Fixture::valid();
    f.nodes = None; // head still declares node_count = 2
    assert_eq!(err_of(&f.build()), FormatError::MissingNodeSection);
}

#[test]
fn reject_edge_count_mismatch() {
    let mut f = Fixture::valid();
    f.head.edge_count = 3; // EDGE still carries 2 edges + 2 reverse entries
    assert_eq!(
        err_of(&f.build()),
        FormatError::EdgeCountMismatch {
            declared: 3,
            section_len: 40,
        }
    );
}

#[test]
fn reject_missing_edge_section() {
    let mut f = Fixture::valid();
    f.edges = None; // head still declares edge_count = 2
    assert_eq!(err_of(&f.build()), FormatError::MissingEdgeSection);
}

#[test]
fn reject_nonzero_reserved_without_edge_algebra_v1() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[1].reserved = 9;
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidEdgePayload {
            edge: 1,
            field: EdgePayloadField::Reserved,
        }
    );
}

// ── Packed-node ranges (RFC §6 item 4) ────────────────────────────────

#[test]
fn reject_child_range_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[0].child_start = 1;
    nodes[0].child_len = 2; // 1 + 2 = 3 > edge_count 2
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::RangeOutOfBounds {
            node: 0,
            field: RangeField::Child,
        }
    );
}

#[test]
fn reject_forward_range_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[1].forward_start = 1;
    nodes[1].forward_len = 2; // 1 + 2 = 3 > edge_count 2
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::RangeOutOfBounds {
            node: 1,
            field: RangeField::Forward,
        }
    );
}

#[test]
fn reject_emission_range_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[1].emission_len = 5; // 4 + 5 = 9 > 8-byte EMIT remainder
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::RangeOutOfBounds {
            node: 1,
            field: RangeField::Emission,
        }
    );
}

#[test]
fn reject_prototype_word_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[0].prototype_word_start = 4; // ROUT is 64 bytes = 8 words; 4 + W(8) = 12 > 8
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::RangeOutOfBounds {
            node: 0,
            field: RangeField::Prototype,
        }
    );
}

#[test]
fn reject_mask_word_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[0].mask_word_start = 1; // 1 + W(8) = 9 > 8 words
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::RangeOutOfBounds {
            node: 0,
            field: RangeField::Mask,
        }
    );
}

// ── Edge endpoints and the reverse index (RFC §6 item 5) ──────────────

#[test]
fn reject_edge_endpoint_out_of_bounds() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[0].src = 5; // >= node_count 2
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::EdgeEndpointOutOfBounds {
            edge: 0,
            src: 5,
            dst: 1,
        }
    );

    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[1].dst = 2; // == node_count 2: still out of bounds
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::EdgeEndpointOutOfBounds {
            edge: 1,
            src: 0,
            dst: 2,
        }
    );
}

#[test]
fn reject_reverse_index_entry_out_of_bounds() {
    let mut f = Fixture::valid();
    f.edges = Some(edge_section(&valid_edges(), &[0, 2])); // 2 >= edge_count
    assert_eq!(
        err_of(&f.build()),
        FormatError::ReverseIndexOutOfBounds {
            index: 1,
            edge_id: 2,
        }
    );
}

#[test]
fn reject_reverse_index_missing_edge() {
    // Both entries point at edge 1; edge 0 has no reverse entry (v0
    // existence approximation of Theorem 7).
    let mut f = Fixture::valid();
    f.edges = Some(edge_section(&valid_edges(), &[1, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::ReverseIndexMissing { edge: 0 }
    );
}

#[test]
fn allow_unknown_optional_kind_without_reverse_coverage() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[1].kind = 0x80;
    f.edges = Some(edge_section(&edges, &[0, 0]));
    GraphView::parse(&f.build())
        .expect("unknown optional kinds may be skipped by reverse coverage");
}

#[test]
fn reject_child_range_non_refinement_edge() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[0].kind = 1; // similarity in child range
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::ChildRangeEdgeMismatch {
            node: 0,
            edge: 0,
            edge_src: 0,
            edge_kind: 1,
        }
    );
}

#[test]
fn reject_forward_range_target_mismatch() {
    let mut f = Fixture::valid();
    // Node 1 forward range starts at 0 and spans both entries. Keep the first
    // reverse slot pointing to edge 0 whose dst=1 is valid, and make the second point to a
    // synthetic edge targeting node 0.
    let mut edges = valid_edges();
    edges[1].dst = 0;
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::ReverseRangeTargetMismatch {
            node: 1,
            index: 1,
            edge_id: 1,
            edge_dst: 0,
        }
    );
}

#[test]
fn reject_unknown_mandatory_edge_kind() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[1].kind = 0x40;
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::UnknownMandatoryEdgeKind {
            edge: 1,
            kind: 0x40,
        }
    );
}

#[test]
fn reject_invalid_edge_flags_payload() {
    let mut f = Fixture::valid();
    let mut edges = valid_edges();
    edges[0].flags = 1;
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidEdgePayload {
            edge: 0,
            field: EdgePayloadField::Flags,
        }
    );
}

#[test]
fn reject_missing_contribution_id_for_v1_evidence_edge() {
    let mut f = Fixture::valid();
    f.head.feature_bits_required = FEATURE_EDGE_ALGEBRA_V1;
    let mut edges = valid_edges();
    edges[1].kind = EdgeKind::EvidenceProvenance as u8;
    edges[1].reserved = 0; // required in v1 for evidence-bearing kinds
    f.edges = Some(edge_section(&edges, &[0, 1]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidEdgePayload {
            edge: 1,
            field: EdgePayloadField::ContributionId,
        }
    );
}

#[test]
fn reject_contribution_id_collision_across_kinds() {
    let mut f = Fixture::valid();
    f.head.feature_bits_required = FEATURE_EDGE_ALGEBRA_V1;
    let edges = [
        EdgeFields {
            src: 0,
            dst: 1,
            score_q: 1,
            kind: EdgeKind::PredictiveForward as u8,
            flags: 0,
            reserved: 42,
        },
        EdgeFields {
            src: 0,
            dst: 1,
            score_q: 2,
            kind: EdgeKind::EvidenceProvenance as u8,
            flags: 0,
            reserved: 42,
        },
    ];
    f.edges = Some(edge_section(&edges, &[0, 1]));
    let mut nodes = valid_nodes();
    nodes[0].child_len = 0;
    nodes[1].forward_len = 2;
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::ContributionIdCollision {
            first: 0,
            second: 1,
            src: 0,
            dst: 1,
            contribution_id: 42,
        }
    );
}

#[test]
fn reject_undeclared_cycle_for_acyclic_kind() {
    let mut f = Fixture::valid();
    f.head.feature_bits_required = FEATURE_EDGE_ALGEBRA_V1;
    let mut edges = valid_edges();
    edges[1].kind = EdgeKind::Causal as u8;
    edges[1].src = 1;
    edges[1].dst = 0; // causal kind is declared acyclic and must follow src<dst
    edges[1].reserved = 7;
    f.edges = Some(edge_section(&edges, &[1, 0]));
    let mut nodes = valid_nodes();
    nodes[0].forward_start = 1;
    nodes[0].forward_len = 1;
    nodes[1].forward_start = 0;
    nodes[1].forward_len = 1;
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidEdgePayload {
            edge: 1,
            field: EdgePayloadField::AcyclicOrder,
        }
    );
}

// ── HEAD-bound honesty (RFC §6 item 7) ────────────────────────────────

#[test]
fn reject_child_len_exceeds_frontier_width() {
    let mut f = Fixture::valid();
    f.head.max_frontier_width = 1;
    let mut nodes = valid_nodes();
    nodes[0].child_len = 2; // range still resolves (0 + 2 <= 2 edges)
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::DishonestBounds {
            bound: BoundKind::FrontierWidth,
            declared: 1,
            observed: 2,
        }
    );
}

#[test]
fn reject_emission_len_exceeds_max_entries() {
    let mut f = Fixture::valid();
    f.head.max_emission_entries = 4;
    let mut nodes = valid_nodes();
    nodes[0].emission_len = 5; // range still resolves (0 + 5 <= 8 bytes)
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::DishonestBounds {
            bound: BoundKind::EmissionEntries,
            declared: 4,
            observed: 5,
        }
    );
}

#[test]
fn reject_node_depth_exceeds_depth_count() {
    let mut f = Fixture::valid();
    let mut nodes = valid_nodes();
    nodes[1].depth = 4; // depth_count is 4: depth must be strictly below
    f.nodes = Some(node_section(&nodes));
    assert_eq!(
        err_of(&f.build()),
        FormatError::DishonestBounds {
            bound: BoundKind::DepthCount,
            declared: 4,
            observed: 4,
        }
    );
}

#[test]
fn reject_signature_bytes_mismatch() {
    // Word-aligned storage rule (RFC §4.1): (W-1)*8 < signature_bytes
    // <= W*8. With W = 8 the honest range is 57..=64.
    let mut f = Fixture::valid();
    f.head.signature_bytes = 56; // one full word short of the W = 8 storage
    assert_eq!(
        err_of(&f.build()),
        FormatError::DishonestBounds {
            bound: BoundKind::SignatureBytes,
            declared: 56,
            observed: 64,
        }
    );

    let mut f = Fixture::valid();
    f.head.signature_bytes = 65; // past the W = 8 storage width
    assert_eq!(
        err_of(&f.build()),
        FormatError::DishonestBounds {
            bound: BoundKind::SignatureBytes,
            declared: 65,
            observed: 64,
        }
    );
}

#[test]
fn signature_bytes_within_word_of_storage_parses() {
    // 63 bytes of signature in 64 bytes of storage: word-aligned with
    // one byte of (zero) padding — accepted.
    let mut f = Fixture::valid();
    f.head.signature_bytes = 63;
    GraphView::parse(&f.build()).expect("byte-exact signature within one word must parse");
}

// ── Word-aligned signature storage: W=5, 36-byte signatures ──────────

/// A one-node fixture with the migration geometry: W = 5 words of
/// storage per 36-byte signature (4 padding bytes), the node's
/// prototype/mask windows inside an 11-word ROUT section laid out as
/// `[program][prototype words][mask words]`.
fn word_aligned_fixture() -> Fixture {
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_halt());
    rout.extend_from_slice(&[0u8; 7]); // program padding to 8 bytes
    let mut prototype = [0u8; 40];
    prototype[..36].fill(0xA5); // 36-byte signature, 4 zero padding bytes
    rout.extend_from_slice(&prototype);
    let mut mask = [0u8; 40];
    mask[..36].fill(0xFF); // 36 one-bits, then 4 zero padding bytes
    rout.extend_from_slice(&mask);
    Fixture {
        head: HeadFields {
            signature_words: 5,
            signature_bytes: 36,
            node_count: 1,
            edge_count: 0,
            depth_count: 2,
            ..HeadFields::default()
        },
        head_override: None,
        nodes: Some(node_section(&[NodeFields {
            prototype_word_start: 1, // word 0 is the padded HALT program
            mask_word_start: 6,
            depth: 1,
            ..NodeFields::default()
        }])),
        edges: None,
        rout: Some(rout),
        emit: None,
        exct: None,
    }
}

#[test]
fn word_aligned_signature_storage_parses() {
    let bytes = word_aligned_fixture().build();
    GraphView::parse(&bytes).expect("36-byte signature in 5 words must parse");
}

#[test]
fn reject_nonzero_prototype_padding() {
    let mut f = word_aligned_fixture();
    let mut rout = f.rout.take().unwrap();
    rout[8 + 36] = 1; // first padding byte of the prototype window
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::NonZeroSignaturePadding {
            node: 0,
            field: RangeField::Prototype,
        }
    );
}

#[test]
fn reject_nonzero_mask_padding() {
    let mut f = word_aligned_fixture();
    let mut rout = f.rout.take().unwrap();
    rout[8 + 40 + 39] = 0x80; // last padding byte of the mask window
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::NonZeroSignaturePadding {
            node: 0,
            field: RangeField::Mask,
        }
    );
}

// ── ROUT bytecode (RFC §6 item 6) ─────────────────────────────────────

#[test]
fn reject_unknown_rout_opcode() {
    let mut f = Fixture::valid();
    let mut rout = vec![0x7F];
    rout.extend_from_slice(&op_halt());
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::UnknownRoutingOp {
            offset: 0,
            opcode: 0x7F,
        }
    );
}

#[test]
fn reject_rout_program_too_deep() {
    let mut f = Fixture::valid();
    f.head.max_program_steps = 2; // the valid program has 4 ops
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingProgramTooDeep { ops: 4, max: 2 }
    );
}

#[test]
fn reject_rout_jump_out_of_bounds() {
    // JMP_FWD at op index 1 with delta 5 targets op 1 + 1 + 5 = 7, past
    // the 4-op program. (Backward jumps are unrepresentable: the unsigned
    // forward delta makes the program acyclic by construction.)
    let mut f = Fixture::valid();
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_test_popcount_le(0, 0xFFFF, 3));
    rout.extend_from_slice(&op_jmp_fwd(5));
    rout.extend_from_slice(&op_leaf(0, 0));
    rout.extend_from_slice(&op_halt());
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingJumpOutOfBounds {
            op_index: 1,
            target: 7,
        }
    );
}

#[test]
fn reject_rout_leaf_shortlist_unresolvable() {
    // No trailing table (HALT is the last byte) but the LEAF declares a
    // non-zero shortlist length.
    let mut f = Fixture::valid();
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_leaf(0, 4));
    rout.extend_from_slice(&op_halt());
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingShortlistOutOfBounds { op_index: 0 }
    );

    // A table is present but the range runs past it.
    let mut f = Fixture::valid();
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_leaf(2, 4)); // 2 + 4 = 6 > 4 table bytes
    rout.extend_from_slice(&op_halt());
    rout.extend_from_slice(&[0u8; 4]);
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingShortlistOutOfBounds { op_index: 0 }
    );
}

#[test]
fn reject_rout_unterminated_program() {
    // Bytes run out after a TEST: no HALT, and the last op is not a LEAF.
    let mut f = Fixture::valid();
    f.rout = Some(op_test_popcount_le(0, 1, 1).to_vec());
    assert_eq!(err_of(&f.build()), FormatError::RoutingProgramUnterminated);
}

#[test]
fn reject_rout_truncated_op() {
    // TEST_POPCOUNT_LE declares 12 bytes; only 2 are present.
    let mut f = Fixture::valid();
    f.rout = Some(vec![0x01, 0x00]);
    assert_eq!(
        err_of(&f.build()),
        FormatError::TruncatedRoutingOp {
            offset: 0,
            opcode: 0x01,
        }
    );
}

#[test]
fn reject_rout_operand_out_of_bounds() {
    // word >= HEAD W (8).
    let mut f = Fixture::valid();
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_test_popcount_le(8, 0, 0));
    rout.extend_from_slice(&op_halt());
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingOperandOutOfBounds { op_index: 0 }
    );

    // threshold > 64 (popcount ceiling of a u64).
    let mut f = Fixture::valid();
    let mut rout = Vec::new();
    rout.extend_from_slice(&op_test_popcount_le(0, 0, 65));
    rout.extend_from_slice(&op_halt());
    f.rout = Some(rout);
    assert_eq!(
        err_of(&f.build()),
        FormatError::RoutingOperandOutOfBounds { op_index: 0 }
    );
}

// ── EMIT/EXCT storage descriptors (RFC §6 item 8) ─────────────────────

#[test]
fn reject_emit_invalid_width() {
    let mut f = Fixture::valid();
    f.emit = Some(storage_section(3, 0, 0, &[0u8; 8]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidStorageDescriptor {
            section: SectionId::EMIT,
        }
    );
}

#[test]
fn reject_emit_invalid_shift() {
    let mut f = Fixture::valid();
    f.emit = Some(storage_section(1, 32, 0, &[0u8; 8]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidStorageDescriptor {
            section: SectionId::EMIT,
        }
    );

    let mut f = Fixture::valid();
    f.emit = Some(storage_section(1, -32, 0, &[0u8; 8]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidStorageDescriptor {
            section: SectionId::EMIT,
        }
    );
}

#[test]
fn reject_exct_invalid_descriptor() {
    let mut f = Fixture::valid();
    f.exct = Some(storage_section(9, 0, 0, &[]));
    assert_eq!(
        err_of(&f.build()),
        FormatError::InvalidStorageDescriptor {
            section: SectionId::EXCT,
        }
    );
}
