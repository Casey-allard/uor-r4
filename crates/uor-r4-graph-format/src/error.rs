//! The single focused error type for parsing, validation, serialization,
//! and CID verification.

use core::fmt;

use crate::types::SectionId;

/// Which packed-node range field failed to resolve within its target
/// section (stage 2, RFC §6 item 4). Targets per the v0 draft line:
/// `Child` → the canonical edge array, `Forward` → the EDGE reverse
/// index, `Emission` → the EMIT remainder (bytes after the storage
/// descriptor), `Prototype`/`Mask` → the ROUT section size in u64 words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeField {
    /// `child_start`/`child_len` over the canonical edge array.
    Child,
    /// `forward_start`/`forward_len` over the reverse index.
    Forward,
    /// `emission_start`/`emission_len` over the EMIT remainder.
    Emission,
    /// `prototype_word_start` into the ROUT section (u64 words).
    Prototype,
    /// `mask_word_start` into the ROUT section (u64 words).
    Mask,
}

impl fmt::Display for RangeField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RangeField::Child => "child",
            RangeField::Forward => "forward",
            RangeField::Emission => "emission",
            RangeField::Prototype => "prototype",
            RangeField::Mask => "mask",
        };
        write!(f, "{name}")
    }
}

/// Which HEAD-declared bound was smaller than the maximum observed in
/// the sections (stage 2, RFC §6 item 7: bounds must be honest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundKind {
    /// `A` (max frontier width) vs. the observed max `child_len`.
    FrontierWidth,
    /// `E` (max emission entries per region) vs. the observed max
    /// `emission_len`.
    EmissionEntries,
    /// `depth_count` vs. the observed max node `depth`.
    DepthCount,
    /// `signature_bytes` vs. the W-word storage width (cross-check:
    /// `(W-1)*8 < signature_bytes <= W*8`, RFC §4.1).
    SignatureBytes,
}

impl fmt::Display for BoundKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BoundKind::FrontierWidth => "max frontier width A",
            BoundKind::EmissionEntries => "max emission entries E",
            BoundKind::DepthCount => "depth_count",
            BoundKind::SignatureBytes => "signature_bytes",
        };
        write!(f, "{name}")
    }
}

/// Which packed-edge payload field violated the v0/v1 edge-algebra rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgePayloadField {
    /// `flags` (no bits defined in v0/v1).
    Flags,
    /// `reserved` (must be zero unless the edge-algebra-v1 contribution-id
    /// rules apply).
    Reserved,
    /// `reserved` contribution id is required but missing.
    ContributionId,
    /// Directed acyclic edge carries a non-increasing node order.
    AcyclicOrder,
}

impl fmt::Display for EdgePayloadField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EdgePayloadField::Flags => "flags",
            EdgePayloadField::Reserved => "reserved",
            EdgePayloadField::ContributionId => "contribution_id",
            EdgePayloadField::AcyclicOrder => "acyclic_order",
        };
        write!(f, "{name}")
    }
}

/// Every fallible operation in this crate returns this error.
///
/// Variants map one-to-one onto the stage-1 structural invariants of
/// RFC §6 plus the serializer/CID failure modes. All data carried is
/// `Copy`; no allocation, no source chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// Bytes do not start with the `R4G1` magic.
    BadMagic,
    /// `format_version.major` is not one this reader supports.
    UnsupportedMajorVersion(u8),
    /// Endianness marker is not `0x01` (little-endian).
    UnsupportedEndianness(u8),
    /// `alignment_log2` is outside the supported range `3..=31`
    /// (RFC §2 requires ≥ 3; > 31 is meaningless under u32 offsets).
    UnsupportedAlignment(u8),
    /// Fewer bytes than the fixed 88-byte header.
    TruncatedHeader,
    /// Declared `total_len` does not equal the actual buffer length.
    TotalLenMismatch {
        /// `total_len` as declared in the header.
        declared: u64,
        /// Actual buffer length.
        actual: u64,
    },
    /// A set header flag bit lies in the mandatory feature space but is
    /// not defined by this format version (RFC §6 stage-1 rule 2).
    /// Carries the offending bit mask.
    UnknownMandatoryFeature(u32),
    /// The section table (`section_count` × 16 bytes) extends past
    /// `total_len`.
    SectionTableOutOfBounds,
    /// Section table entries are not in strictly increasing
    /// `section_id` order (canonical ordering, RFC §2). Also covers
    /// duplicate IDs.
    SectionsNotSorted,
    /// An unknown section ID without [`SectionId::OPTIONAL_BIT`] —
    /// i.e. an unknown *mandatory* section (RFC §6 stage-1 rule 2).
    UnknownMandatorySection(u32),
    /// A section offset is not a multiple of `1 << alignment_log2`.
    SectionMisaligned,
    /// `offset + length` overflowed `u32` under checked arithmetic.
    OffsetOverflow,
    /// A section's `[offset, offset + length)` range extends past
    /// `total_len`.
    SectionOutOfBounds,
    /// Two section bodies overlap, or a section body overlaps the
    /// header / section-table region.
    SectionsOverlap,
    /// Serializer: the same section ID was added twice.
    DuplicateSection(SectionId),
    /// Serializer or CID verifier: the mandatory HEAD section is absent,
    /// so `head_cid` cannot be computed or checked.
    MissingHead,
    /// Serializer: a section payload exceeds the u32 length ceiling
    /// (RFC §9.1, ≤ 4 GiB per section). Carries the payload length.
    SectionTooLarge(u64),
    /// `head_cid` does not recompute to the HEAD section body.
    HeadCidMismatch,
    /// `artifact_cid` does not recompute to `artifact_bytes[56..]`.
    ArtifactCidMismatch,
    /// `tokenizer_cid` does not match the loaded tokenizer BLAKE3 hash.
    TokenizerCidMismatch,
    /// HEAD section body is shorter than the fixed 224-byte v0 prefix
    /// (RFC §4 draft-line layout).
    HeadTooShort {
        /// Actual HEAD payload length.
        actual: u64,
    },
    /// HEAD section body carries trailing bytes past the fixed 224-byte
    /// v0 prefix. Rejected (not ignored) so a future HEAD extension must
    /// arrive with a format minor-version bump (RFC §8).
    HeadTooLong {
        /// Actual HEAD payload length.
        actual: u64,
    },
    /// HEAD declares `node_count > 0` but the NODE section is absent
    /// (stage 2 requires NODE iff `node_count > 0`).
    MissingNodeSection,
    /// NODE section byte length ≠ `node_count × 30` (RFC §6 item 4:
    /// record count must equal the declared count).
    NodeCountMismatch {
        /// `node_count` declared in HEAD.
        declared: u32,
        /// Actual NODE section length in bytes.
        section_len: u64,
    },
    /// HEAD declares `edge_count > 0` but the EDGE section is absent.
    MissingEdgeSection,
    /// EDGE section byte length ≠ `edge_count × (16 + 4)` — canonical
    /// edges plus the reverse index (RFC §5 EDGE).
    EdgeCountMismatch {
        /// `edge_count` declared in HEAD.
        declared: u32,
        /// Actual EDGE section length in bytes.
        section_len: u64,
    },
    /// NGRAM has no complete fixed header.
    NgramTooShort,
    /// NGRAM magic is not `NGR1`.
    NgramBadMagic,
    /// NGRAM version is not supported.
    NgramUnsupportedVersion,
    /// NGRAM reserved bytes are non-zero.
    NgramNonZeroReserved,
    /// NGRAM row or entry offset arithmetic is invalid.
    NgramBounds,
    /// NGRAM row keys are not in canonical order.
    NgramRowsNotSorted,
    /// NGRAM entry tokens are not in canonical order.
    NgramEntriesNotSorted,
    /// NGRAM row metadata is invalid.
    NgramInvalidRow,
    /// FWDA has no complete fixed header.
    FwdaTooShort,
    /// FWDA magic is not `FWA1`.
    FwdaBadMagic,
    /// FWDA version is not supported.
    FwdaUnsupportedVersion,
    /// FWDA reserved bytes are non-zero.
    FwdaNonZeroReserved,
    /// FWDA row or entry offset arithmetic is invalid.
    FwdaBounds,
    /// FWDA row keys are not in canonical order.
    FwdaRowsNotSorted,
    /// FWDA entry tokens are not in canonical order.
    FwdaEntriesNotSorted,
    /// FWDA row metadata is invalid.
    FwdaInvalidRow,
    /// A packed-node range field does not resolve within its target
    /// section under checked arithmetic (RFC §6 item 4). For
    /// `Prototype`/`Mask` the full W-word extent from the word start
    /// must lie within the ROUT section.
    RangeOutOfBounds {
        /// Node (record) index carrying the bad range.
        node: u32,
        /// Which range field failed to resolve.
        field: RangeField,
    },
    /// A prototype/mask window's padding bytes — between the byte-exact
    /// `signature_bytes` and the end of its W-word storage extent — are
    /// not all zero (RFC §4.1 word-aligned signature storage).
    NonZeroSignaturePadding {
        /// Node (record) index carrying the bad window.
        node: u32,
        /// Which window (`Prototype` or `Mask`) carried non-zero padding.
        field: RangeField,
    },
    /// An edge endpoint is ≥ `node_count` (RFC §6 item 5).
    EdgeEndpointOutOfBounds {
        /// Edge (canonical array) index.
        edge: u32,
        /// Decoded `src` field.
        src: u32,
        /// Decoded `dst` field.
        dst: u32,
    },
    /// Edge kind is unknown and in the mandatory kind space.
    UnknownMandatoryEdgeKind {
        /// Edge (canonical array) index.
        edge: u32,
        /// Unknown kind value.
        kind: u8,
    },
    /// Edge payload violates the kind/profile validation rules.
    InvalidEdgePayload {
        /// Edge (canonical array) index.
        edge: u32,
        /// Which payload field violated the rule.
        field: EdgePayloadField,
    },
    /// Canonical edge array is not strictly sorted in canonical order.
    EdgeCanonicalOrderViolation {
        /// Previous edge index.
        previous: u32,
        /// Current edge index.
        edge: u32,
    },
    /// Two evidence-carrying edges share the same contribution id on the same
    /// `(src, dst)` pair.
    ContributionIdCollision {
        /// Earlier edge index.
        first: u32,
        /// Later edge index.
        second: u32,
        /// Shared source node.
        src: u32,
        /// Shared destination node.
        dst: u32,
        /// Colliding contribution id (`PackedEdge.reserved`).
        contribution_id: u16,
    },
    /// A reverse-index entry is ≥ `edge_count` (RFC §6 item 5).
    ReverseIndexOutOfBounds {
        /// Reverse-index position.
        index: u32,
        /// Offending edge ID stored there.
        edge_id: u32,
    },
    /// A canonical edge has no entry anywhere in the reverse index —
    /// the v0 existence approximation of Theorem 7 (RFC §6 item 5;
    /// full per-node range wiring comes later).
    ReverseIndexMissing {
        /// Canonical edge index with no reverse entry.
        edge: u32,
    },
    /// A reverse-index entry appears in a node's declared forward range, but
    /// the referenced canonical edge targets a different node.
    ReverseRangeTargetMismatch {
        /// Node whose forward range is being checked.
        node: u32,
        /// Reverse-index position.
        index: u32,
        /// Canonical edge id read from the reverse index.
        edge_id: u32,
        /// Target node (`dst`) of that canonical edge.
        edge_dst: u32,
    },
    /// A canonical edge in a node's declared child range is not a refinement
    /// edge from that node.
    ChildRangeEdgeMismatch {
        /// Node whose child range is being checked.
        node: u32,
        /// Canonical edge id in the child range.
        edge: u32,
        /// Source node (`src`) of that canonical edge.
        edge_src: u32,
        /// Kind of that canonical edge.
        edge_kind: u8,
    },
    /// A HEAD-declared bound is smaller than the maximum observed in
    /// the sections (RFC §6 item 7: bounds must be honest).
    DishonestBounds {
        /// Which bound was understated.
        bound: BoundKind,
        /// Value declared in HEAD.
        declared: u32,
        /// Maximum actually observed (for `SignatureBytes`: the storage
        /// width `W * 8`; the declared value must satisfy
        /// `(W-1)*8 < signature_bytes <= W*8`).
        observed: u32,
    },
    /// ROUT bytecode opcode outside the v0 set (RFC §6 item 6).
    UnknownRoutingOp {
        /// Byte offset of the opcode within the ROUT section.
        offset: u32,
        /// The unknown opcode byte.
        opcode: u8,
    },
    /// A ROUT op's fixed operands run past the section end.
    TruncatedRoutingOp {
        /// Byte offset of the op within the ROUT section.
        offset: u32,
        /// The opcode whose operands are truncated.
        opcode: u8,
    },
    /// ROUT static op count exceeds HEAD `D` (with forward-only jumps,
    /// the static count bounds every execution path — RFC §6 item 6).
    RoutingProgramTooDeep {
        /// Ops parsed before the terminator.
        ops: u32,
        /// HEAD `D` (max decision-program steps).
        max: u32,
    },
    /// ROUT program ends neither at `HALT` nor (at section end) at a
    /// `LEAF` — the v0 form of "at least one LEAF or HALT reachable".
    RoutingProgramUnterminated,
    /// `JMP_FWD` target op index lies outside the program (jumps are
    /// forward-only by construction; this is the in-bounds half of
    /// RFC §6 item 6).
    RoutingJumpOutOfBounds {
        /// Index of the jumping op.
        op_index: u32,
        /// Computed target op index.
        target: u64,
    },
    /// `TEST_POPCOUNT_LE` operand out of range: `word` ≥ HEAD `W` or
    /// `threshold` > 64 (popcount ceiling of a u64).
    RoutingOperandOutOfBounds {
        /// Index of the offending op.
        op_index: u32,
    },
    /// `LEAF` shortlist range does not resolve within the trailing
    /// shortlist table — or no table is present and `shortlist_len ≠ 0`.
    RoutingShortlistOutOfBounds {
        /// Index of the offending LEAF op.
        op_index: u32,
    },
    /// CODE bytecode opcode outside the set.
    UnknownCodeOp {
        /// Byte offset of the opcode within the CODE section.
        offset: u32,
        /// The unknown opcode byte.
        opcode: u8,
    },
    /// A CODE op's fixed operands run past the section end.
    TruncatedCodeOp {
        /// Byte offset of the op within the CODE section.
        offset: u32,
        /// The opcode whose operands are truncated.
        opcode: u8,
    },
    /// CODE static op count exceeds maximum steps.
    CodeProgramTooDeep {
        /// Ops parsed before the terminator.
        ops: u32,
        /// Max decision-program steps.
        max: u32,
    },
    /// CODE program does not end at `HALT`.
    CodeProgramUnterminated,
    /// CODE operand out of range (level > 2).
    CodeOperandOutOfBounds {
        /// Index of the offending op.
        op_index: u32,
    },
    /// EMIT/EXCT storage descriptor invalid (RFC §6 item 8): fewer than
    /// 4 bytes, `width ∉ {0,1,2}`, or `|shift| > 31`.
    InvalidStorageDescriptor {
        /// Section carrying the bad descriptor (EMIT or EXCT).
        section: SectionId,
    },
    /// PTCH section size not a multiple of PACKED_TOMBSTONE_LEN
    PatchSectionMisaligned {
        /// Actual length
        actual_len: u64,
    },
    /// RTNX section size not a multiple of PACKED_ROUTE_TRANSLATION_LEN
    RouteTranslationSectionMisaligned {
        /// Actual length
        actual_len: u64,
    },
    /// FMM section is shorter than its fixed header.
    FmmSectionTooShort {
        /// Actual length.
        actual: u64,
    },
    /// FMM section does not begin with `FMM1`.
    FmmSectionBadMagic,
    /// FMM section version is not supported by this reader.
    FmmSectionUnsupportedVersion(u16),
    /// FMM dimensions or scale descriptor are invalid.
    FmmSectionInvalidDimensions {
        /// Signature-bit coordinate count.
        dimension: u16,
        /// Compiler rank metadata.
        rank: u16,
        /// Candidate-token count.
        token_count: u32,
        /// Compiler factor fractional-bit count.
        factor_fraction_bits: u8,
    },
    /// FMM section length arithmetic overflowed.
    FmmSectionLengthOverflow,
    /// FMM section length does not match its dimensions.
    FmmSectionLengthMismatch {
        /// Expected length from the header dimensions.
        expected: u64,
        /// Actual section length.
        actual: u64,
    },
    /// Loader invariant validation failure
    InvariantViolation(crate::invariant_ownership::InvariantValidationError),
    /// A node's actual degree — derived from the edge list, never from a
    /// caller-supplied value — exceeds the declared structural bound
    /// (`invariant_ownership` invariant 1: bounded node degree).
    NodeDegreeExceeded {
        /// Node whose degree exceeds the limit.
        node: u32,
        /// Actual degree observed from the edge list.
        degree: u32,
        /// Declared maximum degree.
        limit: u32,
    },
    /// Duplicate evidence entry detected in a contribution list
    /// (`invariant_ownership` invariant 4: evidence non-duplication).
    DuplicateEvidence {
        /// The evidence ID that appears more than once.
        evidence_id: u32,
    },
    /// Route-attention instance shorter than its fixed header + mask
    /// (#604, `route_attention` module).
    RouteInstanceTooShort {
        /// Actual buffer length.
        actual: u64,
    },
    /// Route-attention instance does not begin with `RAT1`.
    RouteInstanceBadMagic,
    /// Route-attention instance version is not supported by this reader.
    RouteInstanceUnsupportedVersion(u16),
    /// Route-attention `code_bytes` is not the pinned 288-bit width
    /// (36 bytes — the deployed signature substrate).
    RouteCodeWidthMismatch {
        /// Declared code width in bytes.
        declared: u16,
    },
    /// Route-attention `candidate_count` is outside `1..=64` — the
    /// declared candidate hard cap (sanctioned refusal carries the
    /// observed value and the bound it crossed).
    RouteCandidateCountOutOfBounds {
        /// Declared candidate count.
        declared: u32,
        /// Permitted maximum.
        max: u32,
    },
    /// Route-attention `top_m` is outside `1..=min(8, candidate_count)`
    /// — the declared selection hard cap.
    RouteTopMOutOfBounds {
        /// Declared top-M.
        declared: u32,
        /// Permitted maximum.
        max: u32,
    },
    /// Route-attention reserved header bytes are non-zero.
    RouteNonZeroReserved,
    /// Route-attention instance length does not equal the exact layout
    /// implied by its declared candidate count.
    RouteInstanceLengthMismatch {
        /// Expected byte length.
        expected: u64,
        /// Actual byte length.
        actual: u64,
    },
    /// Route-attention builder inputs declare differing candidate counts
    /// (code table vs contribution table).
    RouteTableShapeMismatch {
        /// Code-table candidate count.
        codes: u64,
        /// Contribution-table candidate count.
        contributions: u64,
    },
    /// A route-attention query is not one route-code width (36 bytes).
    RouteQueryWidthMismatch {
        /// Actual query length in bytes.
        actual: u64,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::BadMagic => write!(f, "bad magic: not an R4G1 artifact"),
            FormatError::UnsupportedMajorVersion(v) => {
                write!(f, "unsupported format major version {v}")
            }
            FormatError::UnsupportedEndianness(m) => {
                write!(
                    f,
                    "unsupported endianness marker 0x{m:02x} (expected 0x01 = little)"
                )
            }
            FormatError::UnsupportedAlignment(a) => {
                write!(f, "unsupported alignment_log2 {a} (supported: 3..=31)")
            }
            FormatError::TruncatedHeader => write!(f, "buffer shorter than the 88-byte header"),
            FormatError::TotalLenMismatch { declared, actual } => write!(
                f,
                "total_len mismatch: header declares {declared} bytes, buffer has {actual}"
            ),
            FormatError::UnknownMandatoryFeature(mask) => {
                write!(f, "unknown mandatory feature bit(s) set: 0x{mask:08x}")
            }
            FormatError::SectionTableOutOfBounds => {
                write!(f, "section table extends past total_len")
            }
            FormatError::SectionsNotSorted => {
                write!(f, "section table entries not sorted by section_id")
            }
            FormatError::UnknownMandatorySection(id) => {
                write!(f, "unknown mandatory section id 0x{id:08x}")
            }
            FormatError::SectionMisaligned => {
                write!(f, "section offset not aligned to 1 << alignment_log2")
            }
            FormatError::OffsetOverflow => write!(f, "section offset + length overflowed u32"),
            FormatError::SectionOutOfBounds => {
                write!(f, "section body range extends past total_len")
            }
            FormatError::SectionsOverlap => {
                write!(
                    f,
                    "section bodies overlap each other or the header/table region"
                )
            }
            FormatError::DuplicateSection(id) => {
                write!(f, "duplicate section id 0x{:08x}", id.raw())
            }
            FormatError::MissingHead => write!(f, "mandatory HEAD section is absent"),
            FormatError::SectionTooLarge(len) => {
                write!(
                    f,
                    "section payload of {len} bytes exceeds the u32 length ceiling"
                )
            }
            FormatError::HeadCidMismatch => {
                write!(f, "head_cid does not match the HEAD section body")
            }
            FormatError::ArtifactCidMismatch => {
                write!(
                    f,
                    "artifact_cid does not match artifact_bytes[56..total_len]"
                )
            }
            FormatError::TokenizerCidMismatch => {
                write!(f, "tokenizer_cid does not match loaded tokenizer.bin")
            }
            FormatError::HeadTooShort { actual } => write!(
                f,
                "HEAD payload of {actual} bytes is shorter than the fixed 224-byte prefix"
            ),
            FormatError::HeadTooLong { actual } => write!(
                f,
                "HEAD payload of {actual} bytes has trailing bytes past the 224-byte prefix"
            ),
            FormatError::MissingNodeSection => {
                write!(f, "HEAD declares node_count > 0 but the NODE section is absent")
            }
            FormatError::NodeCountMismatch {
                declared,
                section_len,
            } => write!(
                f,
                "NODE section holds {section_len} bytes, not node_count {declared} x 30"
            ),
            FormatError::MissingEdgeSection => {
                write!(f, "HEAD declares edge_count > 0 but the EDGE section is absent")
            }
            FormatError::EdgeCountMismatch {
                declared,
                section_len,
            } => write!(
                f,
                "EDGE section holds {section_len} bytes, not edge_count {declared} x 20"
            ),
            FormatError::NgramTooShort => write!(f, "NGRAM section is shorter than its header"),
            FormatError::NgramBadMagic => write!(f, "NGRAM magic is not NGR1"),
            FormatError::NgramUnsupportedVersion => write!(f, "NGRAM version is unsupported"),
            FormatError::NgramNonZeroReserved => write!(f, "NGRAM reserved bytes are non-zero"),
            FormatError::NgramBounds => write!(f, "NGRAM row or entry range is out of bounds"),
            FormatError::NgramRowsNotSorted => write!(f, "NGRAM rows are not canonically sorted"),
            FormatError::NgramEntriesNotSorted => {
                write!(f, "NGRAM entries are not canonically sorted")
            }
            FormatError::NgramInvalidRow => write!(f, "NGRAM row metadata is invalid"),
            FormatError::FwdaTooShort => write!(f, "FWDA section is shorter than its header"),
            FormatError::FwdaBadMagic => write!(f, "FWDA magic is not FWA1"),
            FormatError::FwdaUnsupportedVersion => write!(f, "FWDA version is unsupported"),
            FormatError::FwdaNonZeroReserved => write!(f, "FWDA reserved bytes are non-zero"),
            FormatError::FwdaBounds => write!(f, "FWDA row or entry range is out of bounds"),
            FormatError::FwdaRowsNotSorted => write!(f, "FWDA rows are not canonically sorted"),
            FormatError::FwdaEntriesNotSorted => {
                write!(f, "FWDA entries are not canonically sorted")
            }
            FormatError::FwdaInvalidRow => write!(f, "FWDA row metadata is invalid"),
            FormatError::RangeOutOfBounds { node, field } => write!(
                f,
                "node {node}: {field} range does not resolve within its target section"
            ),
            FormatError::NonZeroSignaturePadding { node, field } => write!(
                f,
                "node {node}: {field} padding bytes past signature_bytes are not all zero"
            ),
            FormatError::EdgeEndpointOutOfBounds { edge, src, dst } => write!(
                f,
                "edge {edge}: endpoint (src {src}, dst {dst}) is not below node_count"
            ),
            FormatError::UnknownMandatoryEdgeKind { edge, kind } => write!(
                f,
                "edge {edge}: unknown mandatory edge kind 0x{kind:02x}"
            ),
            FormatError::InvalidEdgePayload { edge, field } => write!(
                f,
                "edge {edge}: invalid edge payload field {field}"
            ),
            FormatError::EdgeCanonicalOrderViolation { previous, edge } => write!(
                f,
                "canonical edges out of order at edge {edge} (previous edge {previous})"
            ),
            FormatError::ContributionIdCollision {
                first,
                second,
                src,
                dst,
                contribution_id,
            } => write!(
                f,
                "edges {first} and {second} collide on contribution_id {contribution_id} for ({src}->{dst})"
            ),
            FormatError::ReverseIndexOutOfBounds { index, edge_id } => write!(
                f,
                "reverse index entry {index} references edge {edge_id}, not below edge_count"
            ),
            FormatError::ReverseIndexMissing { edge } => {
                write!(f, "canonical edge {edge} has no reverse-index entry")
            }
            FormatError::ReverseRangeTargetMismatch {
                node,
                index,
                edge_id,
                edge_dst,
            } => write!(
                f,
                "node {node}: reverse index entry {index} points to edge {edge_id} targeting node {edge_dst}"
            ),
            FormatError::ChildRangeEdgeMismatch {
                node,
                edge,
                edge_src,
                edge_kind,
            } => write!(
                f,
                "node {node}: child-range edge {edge} has src {edge_src} and kind 0x{edge_kind:02x}"
            ),
            FormatError::DishonestBounds {
                bound,
                declared,
                observed,
            } => write!(
                f,
                "dishonest HEAD bound: {bound} declared {declared} but observed {observed}"
            ),
            FormatError::UnknownRoutingOp { offset, opcode } => write!(
                f,
                "ROUT offset {offset}: unknown routing opcode 0x{opcode:02x}"
            ),
            FormatError::TruncatedRoutingOp { offset, opcode } => write!(
                f,
                "ROUT offset {offset}: operands of opcode 0x{opcode:02x} run past the section end"
            ),
            FormatError::RoutingProgramTooDeep { ops, max } => write!(
                f,
                "ROUT program of {ops} ops exceeds HEAD D (max {max} steps)"
            ),
            FormatError::RoutingProgramUnterminated => {
                write!(f, "ROUT program ends at neither a HALT nor a final LEAF op")
            }
            FormatError::RoutingJumpOutOfBounds { op_index, target } => write!(
                f,
                "ROUT op {op_index}: jump target op {target} is outside the program"
            ),
            FormatError::RoutingOperandOutOfBounds { op_index } => write!(
                f,
                "ROUT op {op_index}: operand out of range (word < HEAD W, threshold <= 64)"
            ),
            FormatError::RoutingShortlistOutOfBounds { op_index } => write!(
                f,
                "ROUT op {op_index}: LEAF shortlist range does not resolve within the trailing table"
            ),
            FormatError::UnknownCodeOp { offset, opcode } => write!(
                f,
                "CODE offset {offset}: unknown code opcode 0x{opcode:02x}"
            ),
            FormatError::TruncatedCodeOp { offset, opcode } => write!(
                f,
                "CODE offset {offset}: operands of opcode 0x{opcode:02x} run past the section end"
            ),
            FormatError::CodeProgramTooDeep { ops, max } => write!(
                f,
                "CODE program of {ops} ops exceeds max {max} steps"
            ),
            FormatError::CodeProgramUnterminated => {
                write!(f, "CODE program does not end at a HALT op")
            }
            FormatError::CodeOperandOutOfBounds { op_index } => write!(
                f,
                "CODE op {op_index}: operand out of range (level > 2)"
            ),
            FormatError::InvalidStorageDescriptor { section } => write!(
                f,
                "section 0x{:08x}: invalid storage descriptor (4 bytes, width in {{0,1,2}}, |shift| <= 31)",
                section.raw()
            ),
            FormatError::PatchSectionMisaligned { actual_len } => write!(
                f,
                "PTCH section holds {actual_len} bytes, not a multiple of 8"
            ),
            FormatError::RouteTranslationSectionMisaligned { actual_len } => write!(
                f,
                "RTNX section holds {actual_len} bytes, not a multiple of 12"
            ),
            FormatError::FmmSectionTooShort { actual } => {
                write!(f, "FMM section is {actual} bytes, shorter than its 20-byte header")
            }
            FormatError::FmmSectionBadMagic => write!(f, "FMM section has bad magic"),
            FormatError::FmmSectionUnsupportedVersion(version) => {
                write!(f, "unsupported FMM section version {version}")
            }
            FormatError::FmmSectionInvalidDimensions {
                dimension,
                rank,
                token_count,
                factor_fraction_bits,
            } => write!(
                f,
                "invalid FMM dimensions: dimension={dimension}, rank={rank}, tokens={token_count}, factor_fraction_bits={factor_fraction_bits}"
            ),
            FormatError::FmmSectionLengthOverflow => write!(f, "FMM section length overflow"),
            FormatError::FmmSectionLengthMismatch { expected, actual } => write!(
                f,
                "FMM section length mismatch: expected {expected}, got {actual}"
            ),
            FormatError::InvariantViolation(err) => write!(f, "graph invariant violation: {err}"),
            FormatError::NodeDegreeExceeded { node, degree, limit } => write!(
                f,
                "node {node}: degree {degree} exceeds limit {limit}"
            ),
            FormatError::DuplicateEvidence { evidence_id } => {
                write!(f, "duplicate evidence ID {evidence_id} detected")
            },
            FormatError::RouteInstanceTooShort { actual } => write!(
                f,
                "route-attention instance shorter than header + mask: {actual} bytes"
            ),
            FormatError::RouteInstanceBadMagic => {
                write!(f, "route-attention instance magic is not RAT1")
            }
            FormatError::RouteInstanceUnsupportedVersion(version) => {
                write!(f, "unsupported route-attention instance version {version}")
            }
            FormatError::RouteCodeWidthMismatch { declared } => write!(
                f,
                "route-code width {declared} bytes is not the pinned 36-byte (288-bit) width"
            ),
            FormatError::RouteCandidateCountOutOfBounds { declared, max } => write!(
                f,
                "route-attention candidate count {declared} outside 1..={max}"
            ),
            FormatError::RouteTopMOutOfBounds { declared, max } => {
                write!(f, "route-attention top-M {declared} outside 1..={max}")
            }
            FormatError::RouteNonZeroReserved => {
                write!(f, "route-attention reserved header bytes are non-zero")
            }
            FormatError::RouteInstanceLengthMismatch { expected, actual } => write!(
                f,
                "route-attention instance length mismatch: expected {expected} bytes, got {actual}"
            ),
            FormatError::RouteTableShapeMismatch {
                codes,
                contributions,
            } => write!(
                f,
                "route-attention tables disagree: {codes} codes vs {contributions} contributions"
            ),
            FormatError::RouteQueryWidthMismatch { actual } => write!(
                f,
                "route-attention query is {actual} bytes, not one 36-byte route code"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FormatError {}
