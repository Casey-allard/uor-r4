//! `MsaStructuredSelectorV1` operator-instance substrate (#643): the
//! canonical wire layout, bounds, validation, and op-census vocabulary
//! of the second target (deployed-class) attention operator, alongside
//! `r4-route-attention/1` (#604).
//!
//! This module is the shared substrate under both implementations of
//! the versioned `msa-structured-selector/1` operator (registered in
//! `uor-r4-model-source::attention`): the scalar reference lives in
//! `uor-r4-graph-certify::msa_selector`, the packed lowering in
//! `uor-r4-graph-runtime::msa_selector`. Both consume the SAME
//! validated borrowed-bytes view defined here — exactly the
//! `route_attention` split, and for the same reason: this crate has no
//! dependency on either `uor-r4-graph-certify` (which depends on
//! `uor-r4-graph-runtime`) or `uor-r4-graph-runtime`, so it is the only
//! place both can share one wire format without a dependency cycle.
//!
//! ## Classification is precomputed, not recomputed on the deployed path
//!
//! The certify-side reference (`uor-r4-graph-certify::msa_selector`)
//! classifies a candidate id by `residue = candidate_id mod 11`, then a
//! table lookup for role/cascade-position (MSA7's "11-Theorem" for the
//! three proven residues, this project's own cascade-position-mod-3
//! extension for the rest — see that module's docs for the full
//! grounding and the explicit non-theorem caveat). That `mod 11` is
//! fine at REFERENCE/BUILD time (`uor-r4-graph-certify` is not
//! P-4-scanned), but the packed lowering in `uor-r4-graph-runtime` IS
//! P-4-scanned (`uor-r4-core::transformerless::mod.rs`,
//! `p4_contract_owned_graph_runtime_source_scan`): no value `*` `/` `%`
//! may appear there, so it cannot recompute a modulus at runtime.
//!
//! The registry record's own declared field already commits to the
//! answer: `permitted_operation_class` on
//! `AttentionOperatorSpec::msa_structured_selector_v1()` names
//! "deployed-integer-table-read-compare-add-no-runtime-modulo". So this
//! wire format carries the classification PRECOMPUTED, at build time,
//! into the instance bytes themselves — the packed lowering reads
//! `role_rank`/`cascade_position` as declared table entries, never
//! recomputing them. [`build_msa_selector_instance`] is the one place
//! `% 11` happens (via
//! [`uor_r4_core`]-free reuse of the same cascade-orbit table the
//! certify-side reference owns, duplicated here in `core`-only form so
//! this `no_std` crate needs no new dependency); [`MsaSelectorView`]
//! never computes it.
//!
//! ## Canonical instance wire layout (version 1, little-endian)
//!
//! ```text
//! offset  size   field
//! 0       4      magic "MSA1"
//! 4       u16    instance version (= 1)
//! 6       u16    reserved (= 0)
//! 8       u32    candidate_count N   (1 ..= 64)
//! 12      u16    top_m M             (1 ..= min(8, N))
//! 14      u16    reserved (= 0)
//! 16      N*10   candidate rows, index order, each:
//!                    4  candidate_id (u32 LE)
//!                    1  role_rank (u8; 0=Gen,1=Med,2=Man,3=Zero)
//!                    1  cascade_position (u8; 0..=9, or 10 sentinel)
//!                    4  contribution (i32 LE, raw ScoreQ Q16.16)
//! ```
//!
//! The layout is deterministic by construction (fixed offsets, no maps,
//! no padding beyond the fixed fields), so identical inputs produce
//! identical bytes and `blake3(bytes)` is the instance identity
//! ([`msa_selector_instance_digest`]).
//!
//! ## Declared bounds (hard caps, sanctioned refusal)
//!
//! Reuses the exact same caps as `r4-route-attention/1`
//! ([`MSA_MAX_CANDIDATES`] = [`crate::route_attention::ROUTE_MAX_CANDIDATES`],
//! [`MSA_MAX_TOP_M`] = [`crate::route_attention::ROUTE_MAX_TOP_M`]) —
//! both target operators share one packed-frontier/shortlist-top-K
//! capacity precedent, and a shared A/B harness over the same fixture
//! shapes needs matching bounds on both sides.
//!
//! Violations are refused on the sanctioned R5 surface: a
//! [`NotAProduct`] whose [`FormatError`] reason carries the declared
//! value and the bound it crossed, never a panic and never a silent
//! clamp.
//!
//! ## Op census vocabulary
//!
//! [`MsaSelectorOpCensus`] mirrors [`crate::route_attention::RouteOpCensus`]'s
//! style: counters only, incremented by the implementations and carried
//! verbatim in the witness. Per step over `N` candidates selecting `M`:
//!
//! ```text
//! table_reads         = N + M   (one classification row read per
//!                                candidate, one contribution read per
//!                                selected slot)
//! compares            = M*N     (fixed M slot comparisons/candidate)
//! adds                = M       (selection-order saturating fold)
//! candidates_examined = N
//! ```
//!
//! Every field is a closed form of `(N, M)` — deliberately
//! data-independent, so the census is replay-verifiable without running
//! the operator. There is no float field: the operator's whole op set
//! is table read, integer compare, and integer add (saturating for
//! `ScoreQ`).

use serde::{Deserialize, Serialize};

use crate::error::FormatError;
use crate::header::{read_u16_le, read_u32_le};
use crate::route_attention::{ROUTE_MAX_CANDIDATES, ROUTE_MAX_TOP_M};
use crate::sanctioned::{NotAProduct, ObjectKind};
use crate::types::ScoreQ;

/// Registry id of the target operator (`uor-r4-model-source::attention`).
pub const MSA_SELECTOR_OPERATOR_ID: &str = "msa-structured-selector";
/// Registry version of the target operator.
pub const MSA_SELECTOR_OPERATOR_VERSION: u32 = 1;

/// Instance wire magic.
pub const MSA_INSTANCE_MAGIC: [u8; 4] = *b"MSA1";
/// Instance wire version accepted by this reader.
pub const MSA_INSTANCE_VERSION: u16 = 1;

/// Hard cap on `candidate_count`, shared with `r4-route-attention/1`
/// (module docs).
pub const MSA_MAX_CANDIDATES: usize = ROUTE_MAX_CANDIDATES;
/// Hard cap on `top_m`, shared with `r4-route-attention/1`.
pub const MSA_MAX_TOP_M: usize = ROUTE_MAX_TOP_M;

/// Fixed instance header length in bytes (before the candidate rows).
pub const MSA_INSTANCE_HEADER_LEN: usize = 16;
/// Bytes of one candidate row: `candidate_id` (4) + `role_rank` (1) +
/// `cascade_position` (1) + `contribution` (4).
pub const MSA_CANDIDATE_ROW_BYTES: usize = 10;

/// Role rank of the γ anchor (`mod_11(γ) = 2`, cascade position 0) —
/// MSA7, "The 11-Theorem".
pub const ROLE_GEN: u8 = 0;
/// Role rank of the μ anchor (`mod_11(μ) = 4`, cascade position 1) —
/// MSA7.
pub const ROLE_MED: u8 = 1;
/// Role rank of the ε anchor (`mod_11(ε) = 8`, cascade position 2) —
/// MSA7.
pub const ROLE_MAN: u8 = 2;
/// Role rank of residue 0 — outside `(ℤ/11ℤ)*`, so outside every role
/// MSA7 assigns; sentinel class, sorts last.
pub const ROLE_ZERO: u8 = 3;

/// Theorem M4 ("The 11-Cascade Theorem"): the doubling cascade starting
/// at 2 in `(ℤ/11ℤ)*`, in orbit order. Position 0 = γ's residue (2,
/// Gen), position 1 = μ's residue (4, Med), position 2 = ε's residue
/// (8, Man) — MSA7. Positions 3..9 are this project's own mod-3 role
/// extension (see `uor-r4-graph-certify::msa_selector` module docs for
/// the full grounding), not a paper theorem. Duplicated verbatim from
/// that module so this `no_std`, dependency-free crate can precompute
/// classification at build time without depending on the certify crate
/// (which depends on this one).
///
/// `cfg(feature = "alloc")`: this table and every helper below it exist
/// solely to support [`build_msa_selector_instance`] (also
/// alloc-gated) — [`MsaSelectorView`] never recomputes classification
/// (module docs), so a `no_std`, no-`alloc` build has no use for the
/// mod-11 machinery at all.
#[cfg(feature = "alloc")]
const CASCADE_ORBIT_11: [u8; 10] = [2, 4, 8, 5, 10, 9, 7, 3, 6, 1];
/// The modulus this operator is pinned to (MSA7/M4 are both stated for
/// p = 11 specifically).
#[cfg(feature = "alloc")]
const CASCADE_MODULUS: u32 = 11;
/// Residue 0 is outside `(ℤ/11ℤ)*` and has no orbit position; this
/// sentinel sorts after every real position (0..=9).
pub const CASCADE_SENTINEL_POSITION: u8 = 10;

/// This residue's position in the cascade orbit, or `None` for residue
/// 0 (not in the multiplicative group).
#[cfg(feature = "alloc")]
const fn cascade_position(residue: u8) -> Option<u8> {
    let mut index = 0usize;
    while index < CASCADE_ORBIT_11.len() {
        if CASCADE_ORBIT_11[index] == residue {
            return Some(index as u8);
        }
        index += 1;
    }
    None
}

/// This residue's role rank (module docs: proven MSA7 anchors at
/// cascade positions 0/1/2, every other nonzero residue's role is its
/// cascade position mod 3, residue 0 is [`ROLE_ZERO`]).
#[cfg(feature = "alloc")]
const fn role_rank(residue: u8) -> u8 {
    match cascade_position(residue) {
        Some(position) => position % 3,
        None => ROLE_ZERO,
    }
}

/// Classify a candidate id at BUILD time: `residue = candidate_id mod
/// 11`, then table lookup for role and cascade position. This is the
/// one place this crate computes a modulus — never on the packed
/// lowering's read path (module docs). Total: every `u32` classifies to
/// something.
#[cfg(feature = "alloc")]
const fn classify_at_build_time(candidate_id: u32) -> (u8, u8) {
    let residue = (candidate_id % CASCADE_MODULUS) as u8;
    let role = role_rank(residue);
    let position = match cascade_position(residue) {
        Some(position) => position,
        None => CASCADE_SENTINEL_POSITION,
    };
    (role, position)
}

/// Op census of MSA-selector execution (module docs table).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsaSelectorOpCensus {
    /// Table reads: one classification row per candidate, one
    /// contribution row per selected slot.
    #[serde(default)]
    pub table_reads: u64,
    /// Ordered `(role_rank, cascade_position, index)` slot comparisons
    /// of the selection.
    #[serde(default)]
    pub compares: u64,
    /// Saturating `ScoreQ` adds during the aggregate fold.
    #[serde(default)]
    pub adds: u64,
    /// Candidates examined (the candidate bound's measured side).
    #[serde(default)]
    pub candidates_examined: u64,
}

/// Zero-copy validated view over canonical instance bytes — the
/// [`GraphView`](crate::GraphView) discipline: constructible only by
/// [`MsaSelectorView::parse`], so every accessor's invariants (widths,
/// bounds, region lengths) hold by construction.
#[derive(Debug, Clone, Copy)]
pub struct MsaSelectorView<'a> {
    bytes: &'a [u8],
    rows: &'a [u8],
    candidate_count: u32,
    top_m: u16,
}

impl<'a> MsaSelectorView<'a> {
    /// Parse and validate canonical instance bytes. Fail-closed on the
    /// sanctioned R5 surface: every malformation or crossed bound is a
    /// [`NotAProduct`] naming [`ObjectKind::MsaSelectorInstance`] with
    /// the focused [`FormatError`] reason.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        let refuse =
            |reason: FormatError| NotAProduct::new(ObjectKind::MsaSelectorInstance, reason);
        if bytes.len() < MSA_INSTANCE_HEADER_LEN {
            return Err(refuse(FormatError::MsaInstanceTooShort {
                actual: bytes.len() as u64,
            }));
        }
        if bytes[0..4] != MSA_INSTANCE_MAGIC {
            return Err(refuse(FormatError::MsaInstanceBadMagic));
        }
        let version = read_u16_le(bytes, 4);
        if version != MSA_INSTANCE_VERSION {
            return Err(refuse(FormatError::MsaInstanceUnsupportedVersion(version)));
        }
        if read_u16_le(bytes, 6) != 0 {
            return Err(refuse(FormatError::MsaNonZeroReserved));
        }
        let candidate_count = read_u32_le(bytes, 8);
        if candidate_count == 0 || candidate_count as usize > MSA_MAX_CANDIDATES {
            return Err(refuse(FormatError::MsaCandidateCountOutOfBounds {
                declared: candidate_count,
                max: MSA_MAX_CANDIDATES as u32,
            }));
        }
        let top_m = read_u16_le(bytes, 12);
        let top_m_max = if (candidate_count as usize) < MSA_MAX_TOP_M {
            candidate_count
        } else {
            MSA_MAX_TOP_M as u32
        };
        if top_m == 0 || u32::from(top_m) > top_m_max {
            return Err(refuse(FormatError::MsaTopMOutOfBounds {
                declared: u32::from(top_m),
                max: top_m_max,
            }));
        }
        if read_u16_le(bytes, 14) != 0 {
            return Err(refuse(FormatError::MsaNonZeroReserved));
        }
        // Constant-stride row addressing: 10*N = (N<<3) + (N<<1).
        let n = candidate_count as usize;
        let rows_len = (n << 3) + (n << 1);
        let expected = MSA_INSTANCE_HEADER_LEN + rows_len;
        if bytes.len() != expected {
            return Err(refuse(FormatError::MsaInstanceLengthMismatch {
                expected: expected as u64,
                actual: bytes.len() as u64,
            }));
        }
        Ok(Self {
            bytes,
            rows: &bytes[MSA_INSTANCE_HEADER_LEN..expected],
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

    /// The candidate row region (`N * 10` bytes, index order).
    pub const fn rows(&self) -> &'a [u8] {
        self.rows
    }

    /// Candidate `index`'s declared row — `(candidate_id, role_rank,
    /// cascade_position, contribution)` — `None` past `N`.
    /// Constant-stride addressing by shift/add: `10*i = (i<<3)+(i<<1)`.
    pub fn candidate_row(&self, index: u32) -> Option<(u32, u8, u8, ScoreQ)> {
        if index >= self.candidate_count {
            return None;
        }
        let i = index as usize;
        let start = (i << 3) + (i << 1);
        let row = self.rows.get(start..start + MSA_CANDIDATE_ROW_BYTES)?;
        let candidate_id = u32::from_le_bytes([row[0], row[1], row[2], row[3]]);
        let role_rank = row[4];
        let cascade_position = row[5];
        let contribution = ScoreQ::from_raw(i32::from_le_bytes([row[6], row[7], row[8], row[9]]));
        Some((candidate_id, role_rank, cascade_position, contribution))
    }
}

/// blake3 digest of canonical instance bytes — the instance identity.
pub fn msa_selector_instance_digest(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

/// Serialize a canonical version-1 instance (compiler/certifier side).
/// Classification is computed HERE, once, at build time (module docs),
/// and baked into the row bytes; the packed lowering never recomputes
/// it. Validates exactly what [`MsaSelectorView::parse`] enforces, plus
/// the builder-only shape rule that the id and contribution tables
/// declare the same candidate count; identical inputs produce identical
/// bytes.
#[cfg(feature = "alloc")]
pub fn build_msa_selector_instance(
    candidate_ids: &[u32],
    contributions: &[ScoreQ],
    top_m: u32,
) -> Result<alloc::vec::Vec<u8>, NotAProduct> {
    let refuse = |reason: FormatError| NotAProduct::new(ObjectKind::MsaSelectorInstance, reason);
    if candidate_ids.len() != contributions.len() {
        return Err(refuse(FormatError::MsaTableShapeMismatch {
            candidate_ids: candidate_ids.len() as u64,
            contributions: contributions.len() as u64,
        }));
    }
    if candidate_ids.is_empty() || candidate_ids.len() > MSA_MAX_CANDIDATES {
        return Err(refuse(FormatError::MsaCandidateCountOutOfBounds {
            declared: candidate_ids.len() as u32,
            max: MSA_MAX_CANDIDATES as u32,
        }));
    }
    let top_m_max = if candidate_ids.len() < MSA_MAX_TOP_M {
        candidate_ids.len() as u32
    } else {
        MSA_MAX_TOP_M as u32
    };
    if top_m == 0 || top_m > top_m_max {
        return Err(refuse(FormatError::MsaTopMOutOfBounds {
            declared: top_m,
            max: top_m_max,
        }));
    }
    let mut bytes = alloc::vec::Vec::with_capacity(
        MSA_INSTANCE_HEADER_LEN + candidate_ids.len() * MSA_CANDIDATE_ROW_BYTES,
    );
    bytes.extend_from_slice(&MSA_INSTANCE_MAGIC);
    bytes.extend_from_slice(&MSA_INSTANCE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(candidate_ids.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(top_m as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for (&candidate_id, &contribution) in candidate_ids.iter().zip(contributions.iter()) {
        let (role_rank, cascade_position) = classify_at_build_time(candidate_id);
        bytes.extend_from_slice(&candidate_id.to_le_bytes());
        bytes.push(role_rank);
        bytes.push(cascade_position);
        bytes.extend_from_slice(&contribution.raw().to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_instance() -> alloc::vec::Vec<u8> {
        // ids chosen so residues mod 11 hit both proven anchors (2, 4,
        // 8) and unproven residues (0, 3), exercising every role class.
        let ids = [2u32, 4, 8, 11, 14];
        let contributions = [
            ScoreQ::from_raw(100),
            ScoreQ::from_raw(-200),
            ScoreQ::from_raw(300),
            ScoreQ::from_raw(-400),
            ScoreQ::from_raw(500),
        ];
        build_msa_selector_instance(&ids, &contributions, 3).expect("valid instance")
    }

    #[test]
    fn classify_at_build_time_matches_the_paper_anchors() {
        assert_eq!(classify_at_build_time(2), (ROLE_GEN, 0));
        assert_eq!(classify_at_build_time(4), (ROLE_MED, 1));
        assert_eq!(classify_at_build_time(8), (ROLE_MAN, 2));
        assert_eq!(
            classify_at_build_time(0),
            (ROLE_ZERO, CASCADE_SENTINEL_POSITION)
        );
        // 11 -> residue 0 (same zero class); 14 -> residue 3, cascade
        // position 7, role 7 % 3 = 1.
        assert_eq!(
            classify_at_build_time(11),
            (ROLE_ZERO, CASCADE_SENTINEL_POSITION)
        );
        assert_eq!(classify_at_build_time(14), (1, 7));
    }

    #[test]
    fn canonical_bytes_round_trip_and_pin_the_layout() {
        let bytes = small_instance();
        assert_eq!(bytes.len(), 16 + 5 * 10);
        let view = MsaSelectorView::parse(&bytes).expect("parses");
        assert_eq!(view.candidate_count(), 5);
        assert_eq!(view.top_m(), 3);
        assert_eq!(
            view.candidate_row(0),
            Some((2, ROLE_GEN, 0, ScoreQ::from_raw(100)))
        );
        assert_eq!(
            view.candidate_row(1),
            Some((4, ROLE_MED, 1, ScoreQ::from_raw(-200)))
        );
        assert_eq!(
            view.candidate_row(2),
            Some((8, ROLE_MAN, 2, ScoreQ::from_raw(300)))
        );
        assert_eq!(
            view.candidate_row(3),
            Some((
                11,
                ROLE_ZERO,
                CASCADE_SENTINEL_POSITION,
                ScoreQ::from_raw(-400)
            ))
        );
        assert!(view.candidate_row(5).is_none());
        assert_eq!(view.as_bytes(), bytes.as_slice());
        assert_eq!(small_instance(), bytes);
        assert_eq!(
            msa_selector_instance_digest(&small_instance()),
            msa_selector_instance_digest(&bytes)
        );
    }

    #[test]
    fn parse_refuses_every_malformation_by_name() {
        let bytes = small_instance();

        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(matches!(
            MsaSelectorView::parse(&bad_magic).expect_err("bad magic refused"),
            NotAProduct {
                object: ObjectKind::MsaSelectorInstance,
                reason: FormatError::MsaInstanceBadMagic,
            }
        ));

        let mut bad_version = bytes.clone();
        bad_version[4] = 9;
        assert!(matches!(
            MsaSelectorView::parse(&bad_version).expect_err("bad version refused"),
            NotAProduct {
                reason: FormatError::MsaInstanceUnsupportedVersion(9),
                ..
            }
        ));

        let mut bad_reserved = bytes.clone();
        bad_reserved[6] = 1;
        assert!(matches!(
            MsaSelectorView::parse(&bad_reserved).expect_err("reserved refused"),
            NotAProduct {
                reason: FormatError::MsaNonZeroReserved,
                ..
            }
        ));

        let mut bad_reserved_tail = bytes.clone();
        bad_reserved_tail[14] = 1;
        assert!(matches!(
            MsaSelectorView::parse(&bad_reserved_tail).expect_err("tail reserved refused"),
            NotAProduct {
                reason: FormatError::MsaNonZeroReserved,
                ..
            }
        ));

        let truncated = &bytes[..bytes.len() - 1];
        assert!(matches!(
            MsaSelectorView::parse(truncated).expect_err("truncation refused"),
            NotAProduct {
                reason: FormatError::MsaInstanceLengthMismatch { .. },
                ..
            }
        ));

        assert!(matches!(
            MsaSelectorView::parse(&bytes[..8]).expect_err("short buffer refused"),
            NotAProduct {
                reason: FormatError::MsaInstanceTooShort { .. },
                ..
            }
        ));
    }

    #[test]
    fn caps_are_refused_with_observed_and_bound() {
        let ids_over_cap: alloc::vec::Vec<u32> = (0..(MSA_MAX_CANDIDATES as u32 + 1)).collect();
        let contributions_over_cap = alloc::vec![ScoreQ::ZERO; MSA_MAX_CANDIDATES + 1];
        assert!(matches!(
            build_msa_selector_instance(&ids_over_cap, &contributions_over_cap, 1)
                .expect_err("candidate cap refused"),
            NotAProduct {
                reason: FormatError::MsaCandidateCountOutOfBounds {
                    declared,
                    max: 64,
                },
                ..
            } if declared == MSA_MAX_CANDIDATES as u32 + 1
        ));
        assert!(matches!(
            build_msa_selector_instance(&[], &[], 1).expect_err("empty refused"),
            NotAProduct {
                reason: FormatError::MsaCandidateCountOutOfBounds {
                    declared: 0,
                    max: 64
                },
                ..
            }
        ));
        let ids = alloc::vec![2u32; 16];
        let contributions = alloc::vec![ScoreQ::ZERO; 16];
        assert!(matches!(
            build_msa_selector_instance(&ids, &contributions, 9).expect_err("top-M cap refused"),
            NotAProduct {
                reason: FormatError::MsaTopMOutOfBounds {
                    declared: 9,
                    max: 8
                },
                ..
            }
        ));
        let ids = alloc::vec![2u32; 2];
        let contributions = alloc::vec![ScoreQ::ZERO; 2];
        assert!(matches!(
            build_msa_selector_instance(&ids, &contributions, 3)
                .expect_err("top-M above N refused"),
            NotAProduct {
                reason: FormatError::MsaTopMOutOfBounds {
                    declared: 3,
                    max: 2
                },
                ..
            }
        ));
        assert!(matches!(
            build_msa_selector_instance(&ids, &contributions, 0).expect_err("zero top-M refused"),
            NotAProduct {
                reason: FormatError::MsaTopMOutOfBounds { declared: 0, .. },
                ..
            }
        ));
        let contributions_short = alloc::vec![ScoreQ::ZERO; 1];
        assert!(matches!(
            build_msa_selector_instance(&ids, &contributions_short, 1)
                .expect_err("shape mismatch refused"),
            NotAProduct {
                reason: FormatError::MsaTableShapeMismatch {
                    candidate_ids: 2,
                    contributions: 1,
                },
                ..
            }
        ));
    }
}
