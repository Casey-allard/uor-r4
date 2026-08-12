//! `R4RouteAttentionV1` operator-instance substrate (#604): the canonical
//! wire layout, bounds, validation, and op-census vocabulary of the first
//! target (deployed-class) route-attention operator.
//!
//! This module is the shared substrate under both implementations of the
//! versioned `r4-route-attention/1` operator (registered in
//! `uor-r4-model-source::attention` next to the #602 source operators):
//! the scalar reference lives in `uor-r4-graph-certify::route_attention`,
//! the packed lowering in `uor-r4-graph-runtime::route_attention`. Both
//! consume the SAME validated borrowed-bytes view defined here, so the
//! bytes are the single authority — exactly the `GraphView` discipline.
//!
//! The operator is DORMANT: registered `open` in `model/ledger.toml` as
//! `r4-route-attention-dormant`, constructible and testable, referenced
//! by no serving path, and it changes no artifact bytes (instances are
//! separate serialized objects, never an R4G1 section — see the module
//! docs of the certify-side reference for the carriage decision).
//!
//! ## Route-code width: 288 bits, from the deployed signature substrate
//!
//! The route code width is pinned to the compiled signature width the
//! whole routing substrate already uses, not chosen fresh:
//!
//! - `uor-r4-core::transformerless::compiler` pins `D = 288` and
//!   `SIG_BYTES = D / 8 = 36`; every deployed signature is 36 bytes.
//! - HEAD declares `signature_bytes` (shipped artifacts declare 36) and
//!   stage 2 requires `(W-1)*8 < signature_bytes <= W*8` for the ROUT
//!   word storage (`head.rs`).
//! - ROUT carries the per-region prototype and mask windows at exactly
//!   that width (`PackedNode::prototype_word_start` /
//!   `mask_word_start`), and the ROUT bytecode's one relation primitive
//!   is the masked popcount (`OP_TEST_POPCOUNT_LE`:
//!   `popcount(signature[word] & mask)` — `rout.rs`).
//!
//! So a route code IS a signature-width bit vector and the masked
//! XOR+popcount relation below is the signature substrate's existing
//! metric, reused rather than invented.
//!
//! ## Canonical instance wire layout (version 1, little-endian)
//!
//! ```text
//! offset  size   field
//! 0       4      magic "RAT1"
//! 4       u16    instance version (= 1)
//! 6       u16    code_bytes (= 36; any other width is refused)
//! 8       u32    candidate_count N   (1 ..= 64)
//! 12      u16    top_m M             (1 ..= min(8, N))
//! 14      u16    reserved (= 0)
//! 16      36     mask (one route-code width)
//! 52      N*36   candidate route codes, index order
//! 52+36N  N*4    candidate contributions (i32 LE, raw ScoreQ Q16.16)
//! ```
//!
//! The layout is deterministic by construction (fixed offsets, no maps,
//! no padding beyond the fixed fields), so identical inputs produce
//! identical bytes and `blake3(bytes)` is the instance identity
//! ([`route_instance_digest`]).
//!
//! ## Declared bounds (hard caps, sanctioned refusal)
//!
//! - `candidate_count` ∈ `1..=`[`ROUTE_MAX_CANDIDATES`] — the candidate
//!   bound. 64 matches the packed frontier capacity precedent
//!   (`packed_kernels::PackedFrontier<64>`).
//! - `top_m` ∈ `1..=min(`[`ROUTE_MAX_TOP_M`]`, candidate_count)` — the
//!   selection bound. 8 matches the shortlist top-K precedent
//!   (`packed_kernels::StepOutput<8>`, trace `PRIMARY_TOP_K`).
//!
//! Violations are refused on the sanctioned R5 surface: a
//! [`NotAProduct`] whose [`FormatError`] reason carries the declared
//! value and the bound it crossed (the `RoutingProgramTooDeep`
//! precedent), never a panic and never a silent clamp.
//!
//! ## Op census vocabulary
//!
//! [`RouteOpCensus`] mirrors the `OpKernel` census style
//! (`uor-r4-core::transformerless::runtime::OpKernel`) with the fields
//! the operator's declared cost model needs. Both implementations are
//! REQUIRED to produce the same census for the same input — the per-step
//! counts are data-independent closed forms of `(N, M)` (see the
//! reference module), so an independent replayer can verify the census
//! without running the operator. There is no float field because there
//! is no float operation: the operator's whole op set is XOR, AND (part
//! of the masked popcount), popcount-table read, integer add/subtract
//! (saturating for ScoreQ), integer compare, and table reads.

use serde::{Deserialize, Serialize};

use crate::error::FormatError;
use crate::header::{read_u16_le, read_u32_le};
use crate::sanctioned::{NotAProduct, ObjectKind};
use crate::types::ScoreQ;

/// Registry id of the target operator (`uor-r4-model-source::attention`).
pub const ROUTE_ATTENTION_OPERATOR_ID: &str = "r4-route-attention";
/// Registry version of the target operator. A behavioral change is a new
/// version, never an in-place edit (#600/#601/#602/#603 discipline).
pub const ROUTE_ATTENTION_OPERATOR_VERSION: u32 = 1;

/// Instance wire magic.
pub const ROUTE_INSTANCE_MAGIC: [u8; 4] = *b"RAT1";
/// Instance wire version accepted by this reader.
pub const ROUTE_INSTANCE_VERSION: u16 = 1;

/// Route-code width in bits: the deployed 288-bit signature width
/// (`compiler::D`), justified in the module docs.
pub const ROUTE_CODE_BITS: usize = 288;
/// Route-code width in bytes (`ROUTE_CODE_BITS / 8` = `SIG_BYTES`).
pub const ROUTE_CODE_BYTES: usize = 36;

/// Hard cap on `candidate_count` (packed-frontier capacity precedent).
pub const ROUTE_MAX_CANDIDATES: usize = 64;
/// Hard cap on `top_m` (shortlist top-K precedent).
pub const ROUTE_MAX_TOP_M: usize = 8;

/// Fixed instance header length in bytes (before the mask).
pub const ROUTE_INSTANCE_HEADER_LEN: usize = 16;
/// Bytes of one contribution entry (i32 LE raw ScoreQ).
pub const ROUTE_CONTRIBUTION_BYTES: usize = 4;

/// Build the 256-entry byte-popcount table at compile time — the same
/// derived table the transformerless runtime uses as its only metric
/// arithmetic (`derive_popcount_table`), reproduced here so the `no_std`
/// packed lowering and the certify-side reference read one shared table.
const fn build_popcount_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut value = 0usize;
    while value < 256 {
        let mut bits = value;
        let mut ones = 0u8;
        while bits != 0 {
            ones += (bits & 1) as u8;
            bits >>= 1;
        }
        table[value] = ones;
        value += 1;
    }
    table
}

/// The byte-popcount table both operator implementations read
/// ("popcount via table" — one table read per masked byte).
pub const ROUTE_POPCOUNT_TABLE: [u8; 256] = build_popcount_table();

/// Op census of route-attention execution, in the `OpKernel` census
/// style: counters only, incremented by the implementations and carried
/// verbatim in the witness. Per step over `N` candidates selecting `M`:
///
/// ```text
/// adds                = 36*N + M   (distance accumulation + aggregation)
/// xors                = 36*N       (query XOR candidate, per byte)
/// popcounts           = 36*N       (masked-byte popcount-table reads;
///                                   the mask AND is part of this op)
/// compares            = M*N        (fixed M slot comparisons/candidate)
/// table_reads         = 2*N + M    (code + mask windows, then one
///                                   contribution row per selected)
/// bytes_read          = 72*N + 4*M (window bytes + contribution bytes;
///                                   the caller-owned query is an input,
///                                   not a table, and is not counted)
/// candidates_examined = N
/// ```
///
/// Every field is a closed form of `(N, M)` — deliberately
/// data-independent, so the census is replay-verifiable without running
/// the operator and a step's cost cannot depend on code contents.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteOpCensus {
    /// Integer additions (distance accumulation, ScoreQ saturating adds).
    #[serde(default)]
    pub adds: u64,
    /// Byte XORs of the compatibility relation.
    #[serde(default)]
    pub xors: u64,
    /// Popcounts, each one read of [`ROUTE_POPCOUNT_TABLE`].
    #[serde(default)]
    pub popcounts: u64,
    /// Ordered `(distance, index)` slot comparisons of the selection.
    #[serde(default)]
    pub compares: u64,
    /// Table reads other than the popcount table: candidate code
    /// windows, mask windows, contribution rows.
    #[serde(default)]
    pub table_reads: u64,
    /// Bytes fetched from the instance's borrowed regions.
    #[serde(default)]
    pub bytes_read: u64,
    /// Candidates examined (the candidate bound's measured side).
    #[serde(default)]
    pub candidates_examined: u64,
}

/// Zero-copy validated view over canonical instance bytes — the
/// [`GraphView`](crate::GraphView) discipline: constructible only by
/// [`RouteAttentionView::parse`], so every accessor's invariants
/// (widths, bounds, region lengths) hold by construction.
#[derive(Debug, Clone, Copy)]
pub struct RouteAttentionView<'a> {
    bytes: &'a [u8],
    mask: &'a [u8],
    codes: &'a [u8],
    contributions: &'a [u8],
    candidate_count: u32,
    top_m: u16,
}

impl<'a> RouteAttentionView<'a> {
    /// Parse and validate canonical instance bytes. Fail-closed on the
    /// sanctioned R5 surface: every malformation or crossed bound is a
    /// [`NotAProduct`] naming [`ObjectKind::RouteAttentionInstance`]
    /// with the focused [`FormatError`] reason.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        let refuse =
            |reason: FormatError| NotAProduct::new(ObjectKind::RouteAttentionInstance, reason);
        if bytes.len() < ROUTE_INSTANCE_HEADER_LEN + ROUTE_CODE_BYTES {
            return Err(refuse(FormatError::RouteInstanceTooShort {
                actual: bytes.len() as u64,
            }));
        }
        if bytes[0..4] != ROUTE_INSTANCE_MAGIC {
            return Err(refuse(FormatError::RouteInstanceBadMagic));
        }
        let version = read_u16_le(bytes, 4);
        if version != ROUTE_INSTANCE_VERSION {
            return Err(refuse(FormatError::RouteInstanceUnsupportedVersion(
                version,
            )));
        }
        let code_bytes = read_u16_le(bytes, 6);
        if usize::from(code_bytes) != ROUTE_CODE_BYTES {
            return Err(refuse(FormatError::RouteCodeWidthMismatch {
                declared: code_bytes,
            }));
        }
        let candidate_count = read_u32_le(bytes, 8);
        if candidate_count == 0 || candidate_count as usize > ROUTE_MAX_CANDIDATES {
            return Err(refuse(FormatError::RouteCandidateCountOutOfBounds {
                declared: candidate_count,
                max: ROUTE_MAX_CANDIDATES as u32,
            }));
        }
        let top_m = read_u16_le(bytes, 12);
        let top_m_max = if (candidate_count as usize) < ROUTE_MAX_TOP_M {
            candidate_count
        } else {
            ROUTE_MAX_TOP_M as u32
        };
        if top_m == 0 || u32::from(top_m) > top_m_max {
            return Err(refuse(FormatError::RouteTopMOutOfBounds {
                declared: u32::from(top_m),
                max: top_m_max,
            }));
        }
        if read_u16_le(bytes, 14) != 0 {
            return Err(refuse(FormatError::RouteNonZeroReserved));
        }
        // Region lengths via shift/add only: 36*N = (N<<5) + (N<<2),
        // 4*N = N<<2. N <= 64 was established above, so nothing here
        // can overflow.
        let n = candidate_count as usize;
        let codes_len = (n << 5) + (n << 2);
        let contributions_len = n << 2;
        let mask_start = ROUTE_INSTANCE_HEADER_LEN;
        let codes_start = mask_start + ROUTE_CODE_BYTES;
        let contributions_start = codes_start + codes_len;
        let expected = contributions_start + contributions_len;
        if bytes.len() != expected {
            return Err(refuse(FormatError::RouteInstanceLengthMismatch {
                expected: expected as u64,
                actual: bytes.len() as u64,
            }));
        }
        Ok(Self {
            bytes,
            mask: &bytes[mask_start..codes_start],
            codes: &bytes[codes_start..contributions_start],
            contributions: &bytes[contributions_start..expected],
            candidate_count,
            top_m,
        })
    }

    /// The full canonical instance bytes (digest input).
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Declared candidate count `N`.
    pub const fn candidate_count(&self) -> u32 {
        self.candidate_count
    }

    /// Declared selection width `M`.
    pub const fn top_m(&self) -> u16 {
        self.top_m
    }

    /// The declared relation mask (one route-code width).
    pub const fn mask(&self) -> &'a [u8] {
        self.mask
    }

    /// The candidate route-code region (`N * 36` bytes, index order).
    pub const fn codes(&self) -> &'a [u8] {
        self.codes
    }

    /// The contribution region (`N * 4` bytes, i32 LE raw ScoreQ).
    pub const fn contributions(&self) -> &'a [u8] {
        self.contributions
    }

    /// Candidate `index`'s route code window, `None` past `N`.
    /// Constant-stride addressing by shift/add: `36*i = (i<<5)+(i<<2)`.
    pub fn candidate_code(&self, index: u32) -> Option<&'a [u8]> {
        if index >= self.candidate_count {
            return None;
        }
        let i = index as usize;
        let start = (i << 5) + (i << 2);
        self.codes.get(start..start + ROUTE_CODE_BYTES)
    }

    /// Candidate `index`'s declared ScoreQ contribution, `None` past `N`.
    pub fn contribution(&self, index: u32) -> Option<ScoreQ> {
        if index >= self.candidate_count {
            return None;
        }
        let start = (index as usize) << 2;
        let window = self.contributions.get(start..start + 4)?;
        Some(ScoreQ::from_raw(i32::from_le_bytes([
            window[0], window[1], window[2], window[3],
        ])))
    }
}

/// blake3 digest of canonical instance bytes — the instance identity.
pub fn route_instance_digest(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// Serialize a canonical version-1 instance (compiler/certifier side).
/// Validates exactly what [`RouteAttentionView::parse`] enforces, plus
/// the builder-only shape rule that the code and contribution tables
/// declare the same candidate count; identical inputs produce identical
/// bytes.
#[cfg(feature = "alloc")]
pub fn build_route_attention_instance(
    mask: &[u8; ROUTE_CODE_BYTES],
    codes: &[[u8; ROUTE_CODE_BYTES]],
    contributions: &[ScoreQ],
    top_m: u32,
) -> Result<alloc::vec::Vec<u8>, NotAProduct> {
    let refuse = |reason: FormatError| NotAProduct::new(ObjectKind::RouteAttentionInstance, reason);
    if codes.len() != contributions.len() {
        return Err(refuse(FormatError::RouteTableShapeMismatch {
            codes: codes.len() as u64,
            contributions: contributions.len() as u64,
        }));
    }
    if codes.is_empty() || codes.len() > ROUTE_MAX_CANDIDATES {
        return Err(refuse(FormatError::RouteCandidateCountOutOfBounds {
            declared: codes.len() as u32,
            max: ROUTE_MAX_CANDIDATES as u32,
        }));
    }
    let top_m_max = if codes.len() < ROUTE_MAX_TOP_M {
        codes.len() as u32
    } else {
        ROUTE_MAX_TOP_M as u32
    };
    if top_m == 0 || top_m > top_m_max {
        return Err(refuse(FormatError::RouteTopMOutOfBounds {
            declared: top_m,
            max: top_m_max,
        }));
    }
    let mut bytes = alloc::vec::Vec::new();
    bytes.extend_from_slice(&ROUTE_INSTANCE_MAGIC);
    bytes.extend_from_slice(&ROUTE_INSTANCE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(ROUTE_CODE_BYTES as u16).to_le_bytes());
    bytes.extend_from_slice(&(codes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(top_m as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(mask);
    for code in codes {
        bytes.extend_from_slice(code);
    }
    for contribution in contributions {
        bytes.extend_from_slice(&contribution.raw().to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_code(seed: usize) -> [u8; ROUTE_CODE_BYTES] {
        let mut code = [0u8; ROUTE_CODE_BYTES];
        for (position, byte) in code.iter_mut().enumerate() {
            *byte = ((position
                .wrapping_mul(37)
                .wrapping_add(seed.wrapping_mul(13)))
                & 0xff) as u8;
        }
        code
    }

    fn small_instance() -> alloc::vec::Vec<u8> {
        let mask = [0xffu8; ROUTE_CODE_BYTES];
        let codes = [ramp_code(1), ramp_code(2), ramp_code(3), ramp_code(4)];
        let contributions = [
            ScoreQ::from_raw(100),
            ScoreQ::from_raw(-200),
            ScoreQ::from_raw(300),
            ScoreQ::from_raw(-400),
        ];
        build_route_attention_instance(&mask, &codes, &contributions, 2).expect("valid instance")
    }

    #[test]
    fn popcount_table_matches_definition_on_all_bytes() {
        let mut stratum_sizes = [0usize; 9];
        for value in 0..=255u8 {
            assert_eq!(
                ROUTE_POPCOUNT_TABLE[value as usize],
                value.count_ones() as u8
            );
            stratum_sizes[ROUTE_POPCOUNT_TABLE[value as usize] as usize] += 1;
        }
        assert_eq!(stratum_sizes, [1, 8, 28, 56, 70, 56, 28, 8, 1]);
    }

    #[test]
    fn canonical_bytes_round_trip_and_pin_the_layout() {
        let bytes = small_instance();
        assert_eq!(bytes.len(), 16 + 36 + 4 * 36 + 4 * 4);
        let view = RouteAttentionView::parse(&bytes).expect("parses");
        assert_eq!(view.candidate_count(), 4);
        assert_eq!(view.top_m(), 2);
        assert_eq!(view.mask(), &[0xffu8; ROUTE_CODE_BYTES]);
        assert_eq!(view.candidate_code(0).expect("code 0"), &ramp_code(1));
        assert_eq!(view.candidate_code(3).expect("code 3"), &ramp_code(4));
        assert!(view.candidate_code(4).is_none());
        assert_eq!(view.contribution(1), Some(ScoreQ::from_raw(-200)));
        assert!(view.contribution(4).is_none());
        assert_eq!(view.as_bytes(), bytes.as_slice());
        // Deterministic: rebuilding yields identical bytes and digest.
        assert_eq!(small_instance(), bytes);
        assert_eq!(
            route_instance_digest(&small_instance()),
            route_instance_digest(&bytes)
        );
    }

    #[test]
    fn parse_refuses_every_malformation_by_name() {
        let bytes = small_instance();

        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(matches!(
            RouteAttentionView::parse(&bad_magic).expect_err("bad magic refused"),
            NotAProduct {
                object: ObjectKind::RouteAttentionInstance,
                reason: FormatError::RouteInstanceBadMagic,
            }
        ));

        let mut bad_version = bytes.clone();
        bad_version[4] = 9;
        assert!(matches!(
            RouteAttentionView::parse(&bad_version).expect_err("bad version refused"),
            NotAProduct {
                reason: FormatError::RouteInstanceUnsupportedVersion(9),
                ..
            }
        ));

        let mut bad_width = bytes.clone();
        bad_width[6] = 40;
        assert!(matches!(
            RouteAttentionView::parse(&bad_width).expect_err("bad width refused"),
            NotAProduct {
                reason: FormatError::RouteCodeWidthMismatch { declared: 40 },
                ..
            }
        ));

        let mut bad_reserved = bytes.clone();
        bad_reserved[14] = 1;
        assert!(matches!(
            RouteAttentionView::parse(&bad_reserved).expect_err("reserved refused"),
            NotAProduct {
                reason: FormatError::RouteNonZeroReserved,
                ..
            }
        ));

        let truncated = &bytes[..bytes.len() - 1];
        assert!(matches!(
            RouteAttentionView::parse(truncated).expect_err("truncation refused"),
            NotAProduct {
                reason: FormatError::RouteInstanceLengthMismatch { .. },
                ..
            }
        ));

        assert!(matches!(
            RouteAttentionView::parse(&bytes[..8]).expect_err("short buffer refused"),
            NotAProduct {
                reason: FormatError::RouteInstanceTooShort { .. },
                ..
            }
        ));
    }

    #[test]
    fn caps_are_refused_with_observed_and_bound() {
        // Candidate cap: 65 declared, 64 permitted.
        let mask = [0u8; ROUTE_CODE_BYTES];
        let codes = alloc::vec![[0u8; ROUTE_CODE_BYTES]; ROUTE_MAX_CANDIDATES + 1];
        let contributions = alloc::vec![ScoreQ::ZERO; ROUTE_MAX_CANDIDATES + 1];
        assert!(matches!(
            build_route_attention_instance(&mask, &codes, &contributions, 1)
                .expect_err("candidate cap refused"),
            NotAProduct {
                reason: FormatError::RouteCandidateCountOutOfBounds {
                    declared: 65,
                    max: 64,
                },
                ..
            }
        ));
        // Zero candidates.
        assert!(matches!(
            build_route_attention_instance(&mask, &[], &[], 1).expect_err("empty refused"),
            NotAProduct {
                reason: FormatError::RouteCandidateCountOutOfBounds {
                    declared: 0,
                    max: 64,
                },
                ..
            }
        ));
        // top-M cap: 9 declared, 8 permitted at N = 16.
        let codes = alloc::vec![[0u8; ROUTE_CODE_BYTES]; 16];
        let contributions = alloc::vec![ScoreQ::ZERO; 16];
        assert!(matches!(
            build_route_attention_instance(&mask, &codes, &contributions, 9)
                .expect_err("top-M cap refused"),
            NotAProduct {
                reason: FormatError::RouteTopMOutOfBounds {
                    declared: 9,
                    max: 8,
                },
                ..
            }
        ));
        // top-M above N: 3 declared, N = 2.
        let codes = alloc::vec![[0u8; ROUTE_CODE_BYTES]; 2];
        let contributions = alloc::vec![ScoreQ::ZERO; 2];
        assert!(matches!(
            build_route_attention_instance(&mask, &codes, &contributions, 3)
                .expect_err("top-M above N refused"),
            NotAProduct {
                reason: FormatError::RouteTopMOutOfBounds {
                    declared: 3,
                    max: 2,
                },
                ..
            }
        ));
        // Zero top-M.
        assert!(matches!(
            build_route_attention_instance(&mask, &codes, &contributions, 0)
                .expect_err("zero top-M refused"),
            NotAProduct {
                reason: FormatError::RouteTopMOutOfBounds { declared: 0, .. },
                ..
            }
        ));
        // Shape mismatch between the two tables.
        let contributions_short = alloc::vec![ScoreQ::ZERO; 1];
        assert!(matches!(
            build_route_attention_instance(&mask, &codes, &contributions_short, 1)
                .expect_err("shape mismatch refused"),
            NotAProduct {
                reason: FormatError::RouteTableShapeMismatch {
                    codes: 2,
                    contributions: 1,
                },
                ..
            }
        ));
    }

    // Serde round-trip (including serde-default backfill of absent
    // fields) is covered in `uor-r4-graph-certify::route_attention`,
    // which owns a serde byte format (ciborium); this crate keeps the
    // derive-only serde surface of its peers (ScoreQ).
}
