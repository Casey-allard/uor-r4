//! # uor-r4-graph-format — R4G1 packed graph artifact container
//!
//! R4G1 is the versioned packed artifact container for the R⁴ holographic
//! graph compiler: a single little-endian, fixed-width, explicitly-aligned
//! binary format that carries a compiled semantic-region graph (sections
//! HEAD/CODE/NODE/EDGE/ROUT/EMIT plus optional
//! EXCT/NGRAM/FWDA/PROV/CERT/PTCH/SECT)
//! from the offline compiler to the deployed runtime. It succeeds the
//! ad-hoc TLA3/TLA4/TLS1 containers.
//!
//! The authoritative specification is `docs/transformerless/R4G1.md`
//! (wire-format RFC, DRAFT); terminology lives in
//! `docs/transformerless/GLOSSARY.md`. This crate implements the first
//! Phase-1 slices of `docs/r4_graph_compiler_implementation_plan.md`:
//!
//! - fixed-width domain newtypes ([`NodeId`], [`SectionOffset`],
//!   [`TokenId`], [`ScoreQ`], [`Depth`], [`Radius`], [`ArtifactCid`],
//!   [`SectionId`]);
//! - the stage-1 structural parser/validator (RFC §6): magic, version,
//!   endianness marker, alignment, `total_len`, section-table bounds,
//!   canonical (sorted) ordering, non-overlap, checked offset arithmetic,
//!   and rejection of unknown mandatory sections / feature bits;
//! - the stage-2 semantic validator (RFC §6 items 4–9): the fixed
//!   224-byte HEAD payload ([`Head`]), packed-range resolution for the
//!   v0 draft-line [`PackedNode`]/[`PackedEdge`] layouts, edge endpoints
//!   plus child/forward/reverse index consistency, edge-kind/profile
//!   validation, HEAD-bound honesty, the ROUT v0 bytecode set, and
//!   EMIT/EXCT [`StorageDescriptor`]s;
//! - the canonical serializer ([`ArtifactBuilder`], behind `alloc`):
//!   deterministic bytes for identical inputs (Gate E, RFC §1 rule 7);
//! - [`GraphView`], a zero-copy borrowed view over caller-owned (or
//!   memory-mapped) bytes, constructible only after successful stage-1
//!   validation plus stage-2 whenever a HEAD section is present, with
//!   typed decode-on-demand node/edge accessors and
//!   [`GraphView::verify_cids`] for the blake3 integrity CIDs (RFC §6
//!   invariant 9).
//!
//! ## CID hashing convention (normative for this crate)
//!
//! ```text
//! head_cid     := BLAKE3( HEAD section body bytes )
//! artifact_cid := BLAKE3( artifact_bytes[56 .. total_len] )
//! ```
//!
//! `artifact_cid` covers everything *after* its own field — i.e. the
//! `head_cid` field, the section table, padding, and all section bodies —
//! chaining both CIDs. The `artifact_cid` field itself (bytes 24..56) is
//! outside its own hash input; the serializer writes it as zeros before
//! hashing and patches the digest in afterwards, so the field's contents
//! never influence the digest. The verifier recomputes over the same
//! `[56 .. total_len]` range. Identical convention on both sides, always.
//!
//! ## no_std / features
//!
//! The parser, validator, and [`GraphView`] are `core`-only (no
//! allocation). The `alloc` feature additionally enables
//! [`ArtifactBuilder`] (it assembles into a `Vec<u8>`). The `std` feature
//! (default) enables `alloc` plus the `std::error::Error` impl for
//! [`FormatError`].

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod code;
mod error;
mod fmm;
mod fwda;
mod head;
mod header;
pub mod inference_contract;
pub mod invariant_ownership;
mod msa_selector;
mod ngram;
mod prov;
#[cfg(feature = "alloc")]
pub mod r4g1;
pub mod records;
mod rout;
pub mod route_attention;
pub mod sanctioned;
pub mod scoring_semantics;
#[cfg(feature = "alloc")]
mod ser;
mod stage2;
mod types;
mod view;

pub use code::{OP_CLEAR_SLOT, OP_HALT as CODE_OP_HALT, OP_SHIFT_SLOTS, OP_UPDATE_SLOT};
pub use error::{BoundKind, EdgePayloadField, FormatError, RangeField};
pub use fmm::{
    FmmCoefficientRow, FmmRows, FmmScoreIter, FmmTokenIter, FmmTranslationTable, FMM_HEADER_LEN,
    FMM_MAGIC, FMM_VERSION,
};
pub use fwda::{
    FwdaEntries, FwdaEntry, FwdaRow, FwdaRows, FwdaTable, FWDA_ENTRY_LEN, FWDA_HEADER_LEN,
    FWDA_MAGIC, FWDA_MAX_DISTANCE, FWDA_ROW_LEN, FWDA_VERSION,
};
pub use head::{
    Head, FALLBACK_POLICY_COUNT, FEATURE_EDGE_ALGEBRA_V1, HEAD_PAYLOAD_LEN,
    KNOWN_FEATURE_BITS_REQUIRED,
};
pub use header::{
    Header, ARTIFACT_CID_OFFSET, ARTIFACT_HASH_START, ENDIANNESS_LITTLE, FORMAT_VERSION_MAJOR,
    FORMAT_VERSION_MINOR, HEADER_LEN, HEAD_CID_OFFSET, MAGIC, SECTION_ENTRY_LEN,
};
pub use inference_contract::{
    owner_for_activity, ActivityOwner, AllowedOperationClass, BoundaryActivity, ContractVersion,
    ExplicitExclusion, ForbiddenOperationClass, InferenceContractError, ACTIVITY_OWNERS,
    ALLOWED_OPERATION_CLASSES, BOUNDARY_ACTIVITIES, EXPLICIT_EXCLUSIONS,
    FORBIDDEN_OPERATION_CLASSES, INFERENCE_OPERATION_CONTRACT_VERSION,
};
pub use invariant_ownership::{
    GraphInvariantId, GraphInvariantOwnershipMatrix, InvariantOwner, InvariantOwnershipEntry,
    InvariantOwnershipRow, InvariantValidationError, INVARIANT_OWNERSHIP_ROWS, MATRIX_VERSION,
    OPERATION_SET_CONFORMANCE_ROW,
};
#[cfg(feature = "alloc")]
pub use msa_selector::build_msa_selector_instance;
pub use msa_selector::{
    msa_selector_instance_digest, MsaSelectorOpCensus, MsaSelectorView, CASCADE_SENTINEL_POSITION,
    MSA_CANDIDATE_ROW_BYTES, MSA_INSTANCE_HEADER_LEN, MSA_INSTANCE_MAGIC, MSA_INSTANCE_VERSION,
    MSA_MAX_CANDIDATES, MSA_MAX_TOP_M, MSA_SELECTOR_OPERATOR_ID, MSA_SELECTOR_OPERATOR_VERSION,
    ROLE_GEN, ROLE_MAN, ROLE_MED, ROLE_ZERO,
};
pub use ngram::{
    NgramEntries, NgramEntry, NgramRow, NgramRows, NgramTable, NGRAM_ENTRY_LEN, NGRAM_HEADER_LEN,
    NGRAM_MAGIC, NGRAM_ROW_LEN, NGRAM_VERSION,
};
#[cfg(feature = "alloc")]
pub use prov::{build as build_prov, ProvComponents};
pub use prov::{
    parse_digest_hex, EvidenceRoots, Prov, PROV_DIGEST_LEN, PROV_HEADER_LEN, PROV_MAGIC,
    PROV_VERSION,
};
pub use records::{
    EdgeKind, PackedEdge, PackedNode, StorageDescriptor, EDGE_KIND_OPTIONAL_BIT, PACKED_EDGE_LEN,
    PACKED_NODE_LEN, STORAGE_DESCRIPTOR_LEN,
};
pub use rout::{OP_HALT, OP_JMP_FWD, OP_LEAF, OP_TEST_POPCOUNT_LE};
#[cfg(feature = "alloc")]
pub use route_attention::build_route_attention_instance;
pub use route_attention::{
    route_instance_digest, RouteAttentionView, RouteOpCensus, ROUTE_ATTENTION_OPERATOR_ID,
    ROUTE_ATTENTION_OPERATOR_VERSION, ROUTE_CODE_BITS, ROUTE_CODE_BYTES, ROUTE_MAX_CANDIDATES,
    ROUTE_MAX_TOP_M, ROUTE_POPCOUNT_TABLE,
};
pub use sanctioned::{KappaError, NotAProduct, ObjectKind, ObservedBound};
#[cfg(feature = "alloc")]
pub use ser::ArtifactBuilder;
pub use types::{ArtifactCid, Depth, NodeId, Radius, ScoreQ, SectionId, SectionOffset, TokenId};
pub use view::{Edges, GraphView, Nodes, SectionRef, Sections};
