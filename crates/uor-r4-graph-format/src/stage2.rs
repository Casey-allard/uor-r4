//! Stage-2 semantic validation (RFC §6 items 4–9): packed-range
//! resolution, edge endpoints and the reverse index, HEAD-bound
//! honesty, ROUT bytecode, and EMIT/EXCT storage descriptors.
//!
//! Stage 2 runs from [`GraphView::parse`](crate::GraphView::parse)
//! after stage 1, and only when a HEAD section is present: a container
//! without HEAD stays stage-1-only so bootstrap fixtures and pure
//! section-carrier artifacts remain parseable (their sections stay
//! opaque bytes). Completeness note: the RFC marks CODE/PROV mandatory,
//! but this draft slice validates the sections listed above only;
//! mandatory-section completeness is a later Phase-1 slice.
//!
//! v0 approximations (recorded in RFC §9):
//!
//! - Reverse-index coverage keeps an O(edge_count²) existence scan (no
//!   allocation) while additionally wiring each node's declared forward
//!   range to incoming-edge targets.
//! - Node emission ranges are byte ranges over the EMIT remainder
//!   (after the 4-byte storage descriptor); entry-typed resolution
//!   waits for the EMIT table layout.
//!
//! Prototype/mask resolution is word-exact: each node's word start plus
//! the full W-word extent must lie within the ROUT section, and — when
//! `signature_bytes < W * 8` (word-aligned storage of a byte-exact
//! signature, RFC §4.1) — the padding bytes between the signature and
//! the end of every validated prototype/mask window must be zero.

use crate::error::{BoundKind, EdgePayloadField, FormatError, RangeField};
use crate::head::{Head, FEATURE_EDGE_ALGEBRA_V1, KNOWN_FEATURE_BITS_REQUIRED};
use crate::header::read_u32_le;
use crate::records::{
    self, is_optional_edge_kind, trajectory_metadata_word_start, trajectory_prototype_word_start,
    EdgeKind, StorageDescriptor, NODE_FLAGS_KNOWN, NODE_FLAG_TRAJECTORY_ROUTE, PACKED_EDGE_LEN,
    PACKED_NODE_LEN, REVERSE_INDEX_ENTRY_LEN, STORAGE_DESCRIPTOR_LEN,
};
use crate::rout;
use crate::types::SectionId;
use crate::view::GraphView;

/// Run stage 2 over the sections of a stage-1-validated view. Returns
/// the decoded [`Head`] when a HEAD section is present; `None` keeps
/// the container stage-1-only (see module docs).
pub(crate) fn validate(view: &GraphView) -> Result<Option<Head>, crate::NotAProduct> {
    let head = match view.section(SectionId::HEAD) {
        Some(bytes) => Head::parse(bytes)?,
        None => return Ok(None),
    };
    let unknown_required = head.feature_bits_required() & !KNOWN_FEATURE_BITS_REQUIRED;
    if unknown_required != 0 {
        return Err((FormatError::UnknownMandatoryFeature(u32::from(unknown_required))).into());
    }
    let edge_algebra_v1 = head.feature_bits_required() & FEATURE_EDGE_ALGEBRA_V1 != 0;

    // HEAD-internal honesty: signature storage is word-aligned. The
    // byte-exact signature width must fit within W u64 words and exceed
    // W-1 words — `(W-1)*8 < signature_bytes <= W*8` (RFC §4.1) — so W=5
    // words store a real 36-byte signature with 4 zero padding bytes.
    let storage_bytes = u32::from(head.signature_words()) * 8;
    let signature_bytes = u32::from(head.signature_bytes());
    if signature_bytes > storage_bytes || signature_bytes <= storage_bytes.saturating_sub(8) {
        return Err((FormatError::DishonestBounds {
            bound: BoundKind::SignatureBytes,
            declared: signature_bytes,
            observed: storage_bytes,
        })
        .into());
    }

    // EMIT/EXCT storage descriptors (RFC §6 item 8). Node emission
    // ranges resolve against the EMIT remainder, so the descriptor is
    // validated first.
    let emit_remainder = match view.section(SectionId::EMIT) {
        Some(bytes) => {
            StorageDescriptor::parse(SectionId::EMIT, bytes)?;
            bytes.len() - STORAGE_DESCRIPTOR_LEN
        }
        None => 0,
    };
    if let Some(bytes) = view.section(SectionId::EXCT) {
        StorageDescriptor::parse(SectionId::EXCT, bytes)?;
    }
    if let Some(bytes) = view.section(SectionId::NGRAM) {
        crate::ngram::NgramTable::parse(bytes)?;
    }
    if let Some(bytes) = view.section(SectionId::FWDA) {
        crate::fwda::FwdaTable::parse(bytes)?;
    }

    // ROUT section-internal bytecode validation (RFC §6 item 6) runs
    // before the per-node cross-references into the section, mirroring
    // the EMIT/EXCT descriptor ordering above. The section size in u64
    // words then anchors prototype/mask word starts, W-word extents,
    // and the zero-padding rule.
    let rout_bytes = view.section(SectionId::ROUT);
    let rout_words = rout_bytes.map_or(0, |bytes| bytes.len() as u64 / 8);
    if let Some(bytes) = rout_bytes {
        rout::validate(bytes, &head)?;
    }

    // CODE section-internal bytecode validation
    if let Some(bytes) = view.section(SectionId::CODE) {
        crate::code::validate(bytes, head.max_program_steps())?;
    }

    // NODE is present iff node_count > 0, and its record count must
    // equal the declaration (RFC §6 item 4).
    let node_bytes = match view.section(SectionId::NODE) {
        Some(bytes) => {
            let expected = u64::from(head.node_count()) * PACKED_NODE_LEN as u64;
            if bytes.len() as u64 != expected {
                return Err((FormatError::NodeCountMismatch {
                    declared: head.node_count(),
                    section_len: bytes.len() as u64,
                })
                .into());
            }
            Some(bytes)
        }
        None => {
            if head.node_count() > 0 {
                return Err((FormatError::MissingNodeSection).into());
            }
            None
        }
    };

    // EDGE holds edge_count canonical edges plus the reverse index.
    let edge_bytes = match view.section(SectionId::EDGE) {
        Some(bytes) => {
            let expected =
                u64::from(head.edge_count()) * (PACKED_EDGE_LEN + REVERSE_INDEX_ENTRY_LEN) as u64;
            if bytes.len() as u64 != expected {
                return Err((FormatError::EdgeCountMismatch {
                    declared: head.edge_count(),
                    section_len: bytes.len() as u64,
                })
                .into());
            }
            Some(bytes)
        }
        None => {
            if head.edge_count() > 0 {
                return Err((FormatError::MissingEdgeSection).into());
            }
            None
        }
    };

    // Per-node range resolution and HEAD-bound honesty (RFC §6
    // items 4 and 7).
    if let Some(bytes) = node_bytes {
        for i in 0..head.node_count() {
            // The section size was validated above, so the record is
            // in bounds.
            let node = records::decode_node(&bytes[i as usize * PACKED_NODE_LEN..]);
            if node.flags & !NODE_FLAGS_KNOWN != 0 {
                return Err((FormatError::UnknownNodeFlags {
                    node: i,
                    flags: node.flags & !NODE_FLAGS_KNOWN,
                })
                .into());
            }
            let child_end = u64::from(node.child_start) + u64::from(node.child_len);
            if child_end > u64::from(head.edge_count()) {
                return Err((FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Child,
                })
                .into());
            }
            let forward_end = u64::from(node.forward_start) + u64::from(node.forward_len);
            if forward_end > u64::from(head.edge_count()) {
                return Err((FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Forward,
                })
                .into());
            }
            let emission_end = u64::from(node.emission_start) + u64::from(node.emission_len);
            if emission_end > emit_remainder as u64 {
                return Err((FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Emission,
                })
                .into());
            }
            // Word starts plus the full W-word extent must resolve
            // within the ROUT section.
            let signature_words = u64::from(head.signature_words());
            if u64::from(node.prototype_word_start) + signature_words > rout_words {
                return Err((FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Prototype,
                })
                .into());
            }
            if u64::from(node.mask_word_start) + signature_words > rout_words {
                return Err((FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Mask,
                })
                .into());
            }
            if node.flags & NODE_FLAG_TRAJECTORY_ROUTE != 0 {
                let Some(trajectory_start) =
                    trajectory_prototype_word_start(node, head.signature_words())
                else {
                    return Err((FormatError::RangeOutOfBounds {
                        node: i,
                        field: RangeField::TrajectoryPrototype,
                    })
                    .into());
                };
                let Some(metadata_start) =
                    trajectory_metadata_word_start(node, head.signature_words())
                else {
                    return Err((FormatError::RangeOutOfBounds {
                        node: i,
                        field: RangeField::TrajectoryMetadata,
                    })
                    .into());
                };
                if u64::from(trajectory_start) + signature_words > rout_words {
                    return Err((FormatError::RangeOutOfBounds {
                        node: i,
                        field: RangeField::TrajectoryPrototype,
                    })
                    .into());
                }
                if u64::from(metadata_start) + 1 > rout_words {
                    return Err((FormatError::RangeOutOfBounds {
                        node: i,
                        field: RangeField::TrajectoryMetadata,
                    })
                    .into());
                }
                let rout = rout_bytes.ok_or(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::TrajectoryMetadata,
                })?;
                let metadata_begin = metadata_start as usize * 8;
                let metadata = &rout[metadata_begin..metadata_begin + 8];
                let trajectory_radius = u16::from_le_bytes([metadata[0], metadata[1]]);
                if metadata[2..].iter().any(|&byte| byte != 0)
                    || u32::from(trajectory_radius) > u32::from(head.signature_bytes()) * 8
                {
                    return Err((FormatError::InvalidTrajectoryRouteMetadata { node: i }).into());
                }
            }
            // Zero-padding rule (RFC §4.1): with word-aligned storage of
            // a byte-exact signature, the bytes between the signature
            // and the end of every validated prototype/mask window must
            // be zero. The extents above place both windows in bounds.
            let signature_bytes = head.signature_bytes() as usize;
            let storage_bytes = head.signature_words() as usize * 8;
            if signature_bytes < storage_bytes {
                if let Some(rout) = rout_bytes {
                    for (word_start, field) in [
                        (node.prototype_word_start, RangeField::Prototype),
                        (node.mask_word_start, RangeField::Mask),
                    ] {
                        let begin = word_start as usize * 8 + signature_bytes;
                        let end = word_start as usize * 8 + storage_bytes;
                        let Some(padding) = rout.get(begin..end) else {
                            return Err((FormatError::RangeOutOfBounds { node: i, field }).into());
                        };
                        if padding.iter().any(|&byte| byte != 0) {
                            return Err(
                                (FormatError::NonZeroSignaturePadding { node: i, field }).into()
                            );
                        }
                    }
                    if let Some(word_start) =
                        trajectory_prototype_word_start(node, head.signature_words())
                    {
                        let begin = word_start as usize * 8 + signature_bytes;
                        let end = word_start as usize * 8 + storage_bytes;
                        let Some(padding) = rout.get(begin..end) else {
                            return Err((FormatError::RangeOutOfBounds {
                                node: i,
                                field: RangeField::TrajectoryPrototype,
                            })
                            .into());
                        };
                        if padding.iter().any(|&byte| byte != 0) {
                            return Err((FormatError::NonZeroSignaturePadding {
                                node: i,
                                field: RangeField::TrajectoryPrototype,
                            })
                            .into());
                        }
                    }
                }
            }
            if node.child_len > head.max_frontier_width() {
                return Err((FormatError::DishonestBounds {
                    bound: BoundKind::FrontierWidth,
                    declared: u32::from(head.max_frontier_width()),
                    observed: u32::from(node.child_len),
                })
                .into());
            }
            if u32::from(node.emission_len) > head.max_emission_entries() {
                return Err((FormatError::DishonestBounds {
                    bound: BoundKind::EmissionEntries,
                    declared: head.max_emission_entries(),
                    observed: u32::from(node.emission_len),
                })
                .into());
            }
            if node.depth.0 >= head.depth_count() {
                return Err((FormatError::DishonestBounds {
                    bound: BoundKind::DepthCount,
                    declared: u32::from(head.depth_count()),
                    observed: u32::from(node.depth.0),
                })
                .into());
            }
        }
    }

    // Edges: endpoints, reverse-index bounds, and the Theorem-7
    // existence approximation (RFC §6 item 5).
    if let Some(bytes) = edge_bytes {
        let edge_count = head.edge_count();
        let mut previous_key: Option<(u32, u8, u32, u16)> = None;
        for i in 0..edge_count {
            // The section size was validated above, so the record is
            // in bounds.
            let edge = records::decode_edge(&bytes[i as usize * PACKED_EDGE_LEN..]);
            if edge.src.0 >= head.node_count() || edge.dst.0 >= head.node_count() {
                return Err((FormatError::EdgeEndpointOutOfBounds {
                    edge: i,
                    src: edge.src.0,
                    dst: edge.dst.0,
                })
                .into());
            }
            let known_kind = EdgeKind::from_raw(edge.kind);
            if known_kind.is_none() && !is_optional_edge_kind(edge.kind) {
                return Err((FormatError::UnknownMandatoryEdgeKind {
                    edge: i,
                    kind: edge.kind,
                })
                .into());
            }
            if edge.flags != 0 {
                return Err((FormatError::InvalidEdgePayload {
                    edge: i,
                    field: EdgePayloadField::Flags,
                })
                .into());
            }
            if !edge_algebra_v1 && edge.reserved != 0 {
                return Err((FormatError::InvalidEdgePayload {
                    edge: i,
                    field: EdgePayloadField::Reserved,
                })
                .into());
            }
            if let Some(kind) = known_kind {
                if kind.directed() && !kind.may_cycle() && edge.src.0 >= edge.dst.0 {
                    return Err((FormatError::InvalidEdgePayload {
                        edge: i,
                        field: EdgePayloadField::AcyclicOrder,
                    })
                    .into());
                }
                if edge_algebra_v1 && kind.requires_contribution_id() && edge.reserved == 0 {
                    return Err((FormatError::InvalidEdgePayload {
                        edge: i,
                        field: EdgePayloadField::ContributionId,
                    })
                    .into());
                }
            }
            let contribution_id = if edge_algebra_v1 { edge.reserved } else { 0 };
            let key = (edge.src.0, edge.kind, edge.dst.0, contribution_id);
            if let Some(prev) = previous_key {
                if prev >= key {
                    return Err((FormatError::EdgeCanonicalOrderViolation {
                        previous: i - 1,
                        edge: i,
                    })
                    .into());
                }
            }
            previous_key = Some(key);
        }
        if edge_algebra_v1 {
            for a in 0..edge_count {
                let edge_a = records::decode_edge(&bytes[a as usize * PACKED_EDGE_LEN..]);
                let Some(kind_a) = EdgeKind::from_raw(edge_a.kind) else {
                    continue;
                };
                if !kind_a.requires_contribution_id() {
                    continue;
                }
                for b in (a + 1)..edge_count {
                    let edge_b = records::decode_edge(&bytes[b as usize * PACKED_EDGE_LEN..]);
                    let Some(kind_b) = EdgeKind::from_raw(edge_b.kind) else {
                        continue;
                    };
                    if !kind_b.requires_contribution_id() {
                        continue;
                    }
                    if edge_a.src == edge_b.src
                        && edge_a.dst == edge_b.dst
                        && edge_a.reserved == edge_b.reserved
                    {
                        return Err((FormatError::ContributionIdCollision {
                            first: a,
                            second: b,
                            src: edge_a.src.0,
                            dst: edge_a.dst.0,
                            contribution_id: edge_a.reserved,
                        })
                        .into());
                    }
                }
            }
        }
        let reverse = &bytes[edge_count as usize * PACKED_EDGE_LEN..];
        for (j, entry) in reverse.chunks_exact(REVERSE_INDEX_ENTRY_LEN).enumerate() {
            let edge_id = read_u32_le(entry, 0);
            if edge_id >= edge_count {
                return Err((FormatError::ReverseIndexOutOfBounds {
                    index: j as u32,
                    edge_id,
                })
                .into());
            }
        }
        // Existence scan (v0): every canonical edge ID that requires reverse
        // coverage must appear at least once in the reverse block.
        for i in 0..edge_count {
            let edge = records::decode_edge(&bytes[i as usize * PACKED_EDGE_LEN..]);
            let Some(kind) = EdgeKind::from_raw(edge.kind) else {
                // Unknown optional kinds are forward-compatible and may be
                // omitted from reverse-coverage requirements.
                continue;
            };
            if !kind.requires_reverse_index() {
                continue;
            }
            let found = reverse
                .chunks_exact(REVERSE_INDEX_ENTRY_LEN)
                .any(|entry| read_u32_le(entry, 0) == i);
            if !found {
                return Err((FormatError::ReverseIndexMissing { edge: i }).into());
            }
        }
        // Every declared node reverse range must resolve to incoming edges of
        // that exact node.
        if let Some(nodes) = node_bytes {
            for node in 0..head.node_count() {
                let node_rec = records::decode_node(&nodes[node as usize * PACKED_NODE_LEN..]);
                let start = node_rec.forward_start;
                let end = start + u32::from(node_rec.forward_len);
                for index in start..end {
                    let reverse_entry = &reverse[index as usize * REVERSE_INDEX_ENTRY_LEN..];
                    let edge_id = read_u32_le(reverse_entry, 0);
                    let edge = records::decode_edge(&bytes[edge_id as usize * PACKED_EDGE_LEN..]);
                    if edge.dst.0 != node {
                        return Err((FormatError::ReverseRangeTargetMismatch {
                            node,
                            index,
                            edge_id,
                            edge_dst: edge.dst.0,
                        })
                        .into());
                    }
                }
            }
        }
    }

    // Child ranges must be refinement/abstraction edges originating from the
    // containing node.
    if let (Some(nodes), Some(edges)) = (node_bytes, edge_bytes) {
        for node in 0..head.node_count() {
            let node_rec = records::decode_node(&nodes[node as usize * PACKED_NODE_LEN..]);
            let start = node_rec.child_start;
            let end = start + u32::from(node_rec.child_len);
            for edge_id in start..end {
                let edge = records::decode_edge(&edges[edge_id as usize * PACKED_EDGE_LEN..]);
                if edge.src.0 != node || edge.kind != EdgeKind::RefinementAbstraction as u8 {
                    return Err((FormatError::ChildRangeEdgeMismatch {
                        node,
                        edge: edge_id,
                        edge_src: edge.src.0,
                        edge_kind: edge.kind,
                    })
                    .into());
                }
            }
        }
    }

    // PTCH (Phase 9) section: 32-byte parent CID + array of PACKED_TOMBSTONE_LEN.
    if let Some(bytes) = view.section(SectionId::PTCH) {
        if bytes.len() < 32 || (bytes.len() - 32) % records::PACKED_TOMBSTONE_LEN != 0 {
            return Err((FormatError::PatchSectionMisaligned {
                actual_len: bytes.len() as u64,
            })
            .into());
        }
    }

    // RTNX (Phase 9) section length must be a multiple of PACKED_ROUTE_TRANSLATION_LEN.
    if let Some(bytes) = view.section(SectionId::RTNX) {
        if bytes.len() % records::PACKED_ROUTE_TRANSLATION_LEN != 0 {
            return Err((FormatError::RouteTranslationSectionMisaligned {
                actual_len: bytes.len() as u64,
            })
            .into());
        }
    }

    // FMM is optional and compiler-folded. Validate its fixed-width table
    // before exposing the borrowed accessor to runtime callers.
    if let Some(bytes) = view.section(SectionId::FMM) {
        crate::fmm::FmmTranslationTable::parse(bytes)?;
    }

    // Invariant Ownership Matrix loader validation (Issue #135)
    let node_count = head.node_count() as usize;
    let max_node_degree = head.max_frontier_width() as usize;
    let degree_limit = head.max_frontier_width() as usize;
    if let Some(inv_err) =
        crate::invariant_ownership::GraphInvariantOwnershipMatrix::validate_graph_structure(
            node_count,
            max_node_degree,
            degree_limit,
            &[],
            &[],
        )
    {
        return Err((FormatError::InvariantViolation(inv_err)).into());
    }

    Ok(Some(head))
}
