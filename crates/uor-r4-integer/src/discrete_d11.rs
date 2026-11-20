//! Clean-room reader and integer-only evaluator for discrete SpiralCore/FBS
//! checkpoints exported in the `UORLUT01` (`uor-r4.lut-discrete/1`) container.
//!
//! Serving arithmetic uses only signed-axis permutation, $L_1$ and Hamming root
//! comparisons, ternary gate accumulation in `{-1, 0, 1}`, and exact `snap_n`
//! orbit-root projection by repeated addition over the 16-root `Cl(0,6)` anchor
//! orbit. No exponent or squashing table, no floating-point value, and no dense
//! matrix product exist on this path.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    invalid,
    ordered_role::{
        orbit_action_digest, RoleBank, RoleBindings, RoleFraming, RoleSites, ROLE_RECORD_BYTES,
    },
    paired_role::{PairedRoleDecision, RequestedContext, RoleSpan, ROLE_VOCAB},
    Result,
};

mod s2_expert;
pub use s2_expert::{s2_route_key, S2ExpertPrediction, S2ExpertRead, S2RankedToken, S2_MASS_TOTAL};
mod assertion_retrieval;
mod rival_admission;
#[path = "contextual_source.rs"]
pub mod contextual_source;
pub use assertion_retrieval::pack_assertion_retrieval;
pub use rival_admission::pack_rival_admission;

/// Container magic (`UORLUT01`).
pub const MAGIC: &[u8; 8] = b"UORLUT01";
#[cfg(feature = "p1-count-serving")]
#[path = "discrete_count.rs"]
pub mod count_deployment;
/// Header schema of a discrete SpiralCore/FBS `UORLUT01` artifact.
pub const DISCRETE_SCHEMA: &str = "uor-r4.lut-discrete/1";
/// Header schema of a 19-table discrete SpiralCore/FBS + post-rotation trunk `UORLUT01` artifact.
pub const DISCRETE_TRUNK_SCHEMA: &str = "uor-r4.lut-discrete-trunk/1";
/// Trunk with a frozen positional-prime successor-count section for P1 read-back.
pub const DISCRETE_S2_SCHEMA: &str = "uor-r4.lut-discrete-trunk-s2/1";
/// Opt-in longest-suffix retained-count prediction on the identical D11 section layout.
pub const DISCRETE_S2_EXPERT_SCHEMA: &str = "uor-r4.lut-discrete-trunk-s2/2";
/// The S2 format binds the base artifact, TRAIN and selected token stream (three SHA-256s).
pub const S2_SECTION: &str = "discrete.trunk.s2.top4.log2.v1";
const S2_HEADER_BYTES: usize = 96;
const S2_ROW_BYTES: usize = 20;
/// Learned read-only role selection, followed by the frozen span, WordStart and continuation operators.
pub const DISCRETE_ROLE_SCHEMA: &str = "uor-r4.lut-discrete-role-copy/1";
/// Frozen paired carrier with an explicit learned all-starts onset bank.
pub const DISCRETE_ALL_STARTS_SCHEMA: &str = "uor-r4.lut-discrete-all-starts/1";
const DISCRETE_REQUESTED_OCCURRENCE_SCHEMA: &str = "uor-r4.lut-discrete-requested-occurrence/1";
const DISCRETE_USER_TEXT_ONLY_SCHEMA: &str = "uor-r4.lut-discrete-requested-occurrence/2";
const DISCRETE_CLAUSE_TO_ENTITY_SCHEMA: &str = "uor-r4.lut-discrete-requested-occurrence/3";
/// A learned requested/excluded role bank, assertion-local value read and exact-copy agreement.
pub const DISCRETE_PAIRED_SCHEMA: &str = "uor-r4.lut-discrete-paired-copy/1";
/// Positional or identity Cl(0,6) role state with the frozen paired writer.
pub const DISCRETE_ORDERED_ROLE_SCHEMA: &str = "uor-r4.lut-discrete-paired-copy/2";
#[path = "paired_copy.rs"]
mod paired_copy;
pub use paired_copy::{PairedAssertionWitness, PairedCopyDecision};

/// Borrowed native inputs for a frozen-role backend replay.
pub struct FrozenRoleReplay<'a> {
    pub roles: &'a crate::paired_role::PairedRoleState,
    pub delimiters: &'a [u32],
    pub assertion_classes: &'a [u32],
    pub requested_framing: Option<&'a RoleFraming>,
    pub requested_context: Option<RequestedContext>,
}

#[derive(Serialize)]
pub struct RequestedRoleReplayTrace {
    #[serde(flatten)]
    pub roles: crate::paired_role::RequestedOccurrenceTrace,
    pub assertions: Vec<PairedAssertionWitness>,
    pub contextual_sources: Vec<ReplayContextualSource>,
}

#[derive(Serialize)]
pub struct ReplayContextualSource {
    pub fact_span: [usize; 2],
    pub selection: contextual_source::SourceSelection,
    pub candidates: Vec<(contextual_source::SourceCandidate, contextual_source::SourceScore)>,
}

/// Version 1 stops at a period, EOS, BOS or newline. A witnessed delimiter after a nonempty copy emits EOS.
pub const ROLE_COPY_STOPS: [u32; 4] = [16, 1, 0, 201];

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RolePolicy {
    #[serde(skip)]
    paired: Option<paired_copy::Policy>,
    #[serde(skip)]
    onset: Option<OnsetPolicy>,
    occurrence: [Vec<[u32; 6]>; 3],
    span: crate::span_selector::Bank,
    exact: crate::value_selector::ExactBank,
    legacy_root: u32,
}

#[derive(Clone)]
struct OnsetPolicy {
    bank: crate::all_starts::Bank,
    packed_roots: Vec<u32>,
    requested_policy: Option<RequestedPolicy>,
}

impl RolePolicy {
    fn requested_policy(&self) -> Option<&RequestedPolicy> {
        self.onset.as_ref()?.requested_policy.as_ref()
    }

    fn requested_framing(&self) -> Option<&RoleFraming> {
        Some(self.requested_policy()?.framing())
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum RequestedPolicy {
    AssertionAgreement(RoleFraming),
    UserTextOnly(RoleFraming),
    ClauseToEntity {
        framing: RoleFraming,
        separators: [u32; 2],
    },
}

impl RequestedPolicy {
    fn framing(&self) -> &RoleFraming {
        match self {
            Self::AssertionAgreement(framing) | Self::UserTextOnly(framing) => framing,
            Self::ClauseToEntity { framing, .. } => framing,
        }
    }

    fn context(&self) -> RequestedContext {
        match self {
            Self::AssertionAgreement(_) => RequestedContext::FullPrefix,
            Self::UserTextOnly(_) => RequestedContext::UserTextOnly,
            Self::ClauseToEntity { separators, .. } => {
                RequestedContext::ClauseToEntity(*separators)
            }
        }
    }
}

fn onset_schema(requested: Option<&RequestedPolicy>) -> &'static str {
    match requested {
        Some(RequestedPolicy::AssertionAgreement(_)) => DISCRETE_REQUESTED_OCCURRENCE_SCHEMA,
        Some(RequestedPolicy::UserTextOnly(_)) => DISCRETE_USER_TEXT_ONLY_SCHEMA,
        Some(RequestedPolicy::ClauseToEntity { .. }) => DISCRETE_CLAUSE_TO_ENTITY_SCHEMA,
        None => DISCRETE_ALL_STARTS_SCHEMA,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OnsetEnvelope {
    schema: String,
    parent_sha256: String,
    span_sha256: String,
    bank_sha256: String,
    span_bytes: usize,
    profile: crate::all_starts::OnsetProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    requested_policy: Option<RequestedPolicy>,
}

#[derive(Deserialize)]
struct OnsetSpanState {
    artifact_sha256: String,
    tokenizer_sha256: String,
    bank: crate::span_selector::Bank,
}

const MAX_ONSET_SPAN_BYTES: usize = 1 << 20;

impl std::fmt::Debug for RolePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RolePolicy")
            .field(
                "occurrence_sizes",
                &self.occurrence.each_ref().map(Vec::len),
            )
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RoleEnvelope {
    schema: String,
    base_sha256: String,
    tokenizer_sha256: String,
    policy: RolePolicy,
    #[serde(default)]
    paired: Option<paired_copy::Header>,
}

impl RolePolicy {
    fn validate(&self) -> Result<()> {
        self.span.validate()?;
        self.exact.validate()?;
        if self.legacy_root > 65535
            || self.occurrence.iter().any(|g| {
                g.is_empty()
                    || g.len() > 4096
                    || g.iter().flatten().any(|&v| v > 65535 && v != u32::MAX)
            })
        {
            return Err(invalid("invalid role-copy occurrence bank or legacy root"));
        }
        Ok(())
    }

    fn choose(&self, ids: &[u32], roots: &[u32]) -> Result<crate::content_selector::Choice> {
        use crate::content_selector::{Choice, WIDTH};
        use crate::span_selector::{bounds, matched};
        if ids.is_empty()
            || ids.len() > WIDTH
            || roots.len() != 4096
            || ids.iter().any(|&v| v >= 4096)
        {
            return Err(invalid("role selector needs a bounded V4096 prompt"));
        }
        let orbit = spiralcore_orbit_roots();
        let lut: [[u32; 16]; 16] = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                orbit[i]
                    .iter()
                    .zip(&orbit[j])
                    .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                    .sum()
            })
        });
        let distance = |a: u32, b: u32| -> u32 {
            if a == b {
                return 0;
            }
            if a == u32::MAX || b == u32::MAX {
                return 64;
            }
            (0..4)
                .map(|k| lut[((a >> (4 * k)) & 15) as usize][((b >> (4 * k)) & 15) as usize])
                .sum()
        };
        let mut prior = Vec::with_capacity(ids.len());
        let mut winner = None;
        for q in 0..ids.len() {
            let span = bounds(ids, &self.span.classes, q);
            let site: [u32; 6] = std::array::from_fn(|j| {
                let pos = if j < 3 {
                    span.0.checked_sub(3 - j)
                } else {
                    Some(span.1 + 1 + j - 3)
                };
                pos.and_then(|i| ids.get(i))
                    .map(|&id| roots[id as usize])
                    .unwrap_or(u32::MAX)
            });
            let near = |bank: &[[u32; 6]]| {
                bank.iter()
                    .map(|p| {
                        site.iter()
                            .zip(p)
                            .map(|(&x, &y)| distance(x, y))
                            .sum::<u32>()
                    })
                    .min()
                    .unwrap_or(0)
            };
            let role = near(&self.occurrence[0]) + 48u32.saturating_sub(near(&self.occurrence[1]));
            let fact = near(&self.occurrence[2]);
            for (f, &(other, cost)) in prior.iter().take(q.saturating_sub(1)).enumerate() {
                if matched(ids, q, f, span, other) {
                    let candidate = (role + cost, q, f);
                    if winner.is_none_or(|old| candidate < old) {
                        winner = Some(candidate);
                    }
                }
            }
            prior.push((span, fact));
        }
        let Some((_, q, f)) = winner else {
            return Ok(Choice::default());
        };
        let mut choice = Choice {
            query: Some(q as u32),
            fact: Some(f as u32),
            source: None,
        };
        let span =
            crate::span_selector::resolve(ids, roots, &self.span, self.legacy_root, choice)?.span;
        let value = crate::value_selector::decide(ids, roots, &self.span, &self.exact, span)?;
        choice.source =
            crate::continuation_selector::decide(ids, roots, &self.span, span, value)?.source;
        Ok(choice)
    }
}

/// Source snapshot schema recorded by `uor-r4-discrete`.
pub const SNAPSHOT_SCHEMA: &str = "uor-r4.discrete-spiralcore-fbs/2";

const ALIGN: usize = 64;
const MAX_HEADER: u64 = 64 << 20;
const MAX_CONTEXT: usize = 243;
const MAX_KEPT: usize = 8;
const MAX_SWEEPS: u32 = 5;
const EDGE_TYPES: usize = 5;

const AXES: [u8; 512] = [
    0, 1, 2, 3, 4, 5, 6, 7, 12, 10, 1, 14, 0, 7, 3, 13, 15, 11, 6, 1, 5, 12, 10, 0, 2, 12, 8, 13,
    1, 3, 15, 6, 14, 13, 15, 4, 11, 1, 0, 2, 5, 14, 11, 2, 7, 8, 1, 12, 13, 14, 11, 2, 15, 0, 1, 4,
    9, 0, 12, 7, 2, 6, 13, 11, 3, 7, 13, 8, 14, 2, 4, 9, 15, 3, 14, 9, 5, 12, 2, 0, 14, 5, 15, 12,
    3, 9, 0, 2, 10, 12, 0, 13, 1, 3, 7, 14, 4, 10, 1, 14, 8, 15, 3, 5, 15, 3, 6, 9, 13, 4, 10, 0,
    11, 15, 13, 0, 14, 2, 4, 1, 9, 0, 4, 7, 10, 14, 5, 11, 8, 9, 10, 11, 12, 13, 14, 15, 7, 3, 14,
    9, 13, 4, 2, 8, 10, 4, 0, 5, 9, 11, 7, 14, 6, 5, 7, 12, 3, 9, 8, 10, 13, 6, 3, 10, 15, 0, 9, 4,
    11, 7, 5, 8, 14, 2, 12, 1, 9, 8, 12, 15, 10, 14, 13, 11, 0, 9, 10, 11, 4, 5, 14, 7, 5, 14, 3,
    2, 15, 0, 9, 12, 6, 5, 15, 12, 11, 1, 0, 10, 2, 12, 0, 13, 9, 11, 7, 6, 5, 6, 3, 10, 7, 8, 9,
    12, 3, 15, 13, 0, 6, 10, 4, 9, 1, 0, 4, 7, 2, 6, 5, 3, 8, 1, 2, 3, 12, 13, 6, 15, 8, 1, 10, 3,
    4, 5, 14, 15, 9, 8, 4, 15, 2, 6, 5, 11, 3, 15, 5, 0, 14, 2, 12, 9, 1, 8, 4, 15, 10, 14, 5, 3,
    6, 13, 7, 4, 11, 1, 8, 10, 13, 6, 11, 10, 7, 8, 1, 4, 14, 13, 7, 4, 3, 9, 8, 2, 0, 9, 2, 11,
    12, 13, 6, 7, 1, 0, 12, 7, 10, 14, 13, 3, 4, 2, 1, 6, 0, 7, 3, 5, 11, 15, 5, 0, 6, 10, 12, 1,
    2, 4, 8, 5, 9, 11, 15, 6, 7, 11, 14, 1, 5, 12, 2, 8, 10, 4, 8, 5, 1, 3, 15, 14, 11, 7, 13, 8,
    6, 10, 4, 1, 12, 10, 9, 14, 8, 15, 11, 13, 7, 11, 6, 1, 13, 4, 10, 8, 12, 2, 9, 6, 0, 7, 11,
    13, 3, 7, 5, 8, 6, 10, 12, 9, 1, 8, 12, 15, 2, 6, 13, 3, 4, 2, 9, 6, 8, 15, 11, 5, 12, 2, 1, 6,
    8, 15, 3, 13, 10, 12, 8, 13, 9, 11, 15, 14, 6, 13, 15, 4, 3, 9, 0, 10, 4, 10, 9, 14, 0, 7, 11,
    5, 2, 4, 0, 5, 1, 3, 7, 6, 8, 9, 2, 11, 4, 5, 6, 15, 14, 5, 7, 12, 11, 1, 8, 2, 0, 1, 10, 3,
    12, 13, 14, 7, 13, 14, 3, 2, 7, 8, 9, 4, 5, 6, 11, 10, 15, 0, 1, 12, 15, 11, 14, 1, 13, 4, 2,
    0, 7, 3, 6, 9, 5, 12, 10, 8,
];

const COMPOSE: [u8; 4096] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 1, 16, 6, 7, 8, 9, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, 51, 27, 34, 41, 47, 35, 42, 48, 43, 49, 50, 2, 10, 11, 12, 52, 53, 54, 3,
    28, 13, 14, 55, 56, 57, 4, 29, 36, 15, 58, 59, 5, 30, 37, 44, 0, 38, 39, 45, 31, 32, 46, 33,
    40, 62, 63, 61, 60, 2, 27, 16, 10, 11, 12, 1, 28, 29, 30, 18, 19, 20, 31, 32, 33, 17, 0, 35,
    42, 48, 7, 8, 9, 55, 56, 58, 51, 34, 41, 47, 43, 49, 50, 21, 3, 52, 53, 13, 14, 60, 22, 4, 38,
    54, 15, 61, 23, 5, 39, 45, 6, 24, 25, 26, 36, 37, 63, 44, 62, 46, 40, 57, 59, 3, 34, 35, 16,
    13, 14, 21, 1, 36, 37, 2, 38, 39, 19, 20, 40, 18, 10, 0, 43, 49, 27, 52, 53, 8, 9, 59, 28, 6,
    55, 56, 11, 12, 61, 51, 17, 41, 47, 42, 48, 50, 24, 31, 4, 57, 60, 15, 25, 32, 5, 46, 7, 29,
    30, 62, 22, 23, 26, 63, 44, 33, 45, 58, 54, 4, 41, 42, 43, 16, 15, 22, 24, 1, 44, 31, 2, 45, 3,
    46, 20, 19, 11, 13, 0, 50, 55, 27, 54, 34, 57, 9, 29, 52, 6, 58, 35, 60, 12, 36, 38, 7, 59, 10,
    61, 14, 51, 17, 18, 47, 48, 49, 26, 33, 40, 5, 8, 21, 63, 30, 28, 62, 37, 23, 25, 39, 32, 53,
    56, 5, 47, 48, 49, 50, 16, 23, 25, 26, 1, 32, 33, 2, 40, 3, 4, 20, 12, 14, 15, 0, 56, 58, 27,
    59, 34, 41, 30, 53, 54, 6, 61, 35, 42, 37, 39, 57, 7, 60, 10, 43, 44, 45, 46, 8, 11, 13, 51,
    17, 18, 19, 9, 62, 21, 22, 63, 28, 24, 29, 36, 31, 38, 55, 52, 6, 2, 51, 21, 22, 23, 16, 10,
    11, 12, 34, 41, 47, 52, 53, 54, 27, 1, 28, 29, 30, 18, 19, 20, 31, 32, 33, 0, 3, 4, 5, 36, 37,
    44, 35, 7, 38, 39, 24, 25, 62, 42, 8, 55, 45, 26, 63, 48, 9, 56, 58, 17, 43, 49, 50, 13, 14,
    60, 15, 61, 59, 57, 46, 40, 7, 3, 28, 51, 24, 25, 35, 16, 13, 14, 6, 55, 56, 41, 47, 57, 34,
    21, 1, 36, 37, 2, 38, 39, 19, 20, 40, 10, 17, 31, 32, 22, 23, 63, 0, 27, 4, 5, 29, 30, 44, 43,
    52, 8, 46, 62, 26, 49, 53, 9, 59, 18, 11, 12, 61, 42, 48, 50, 60, 15, 54, 58, 33, 45, 8, 4, 29,
    36, 51, 26, 42, 43, 16, 15, 52, 6, 58, 7, 59, 47, 41, 22, 24, 1, 44, 31, 2, 45, 3, 46, 20, 11,
    38, 17, 33, 28, 62, 23, 13, 55, 18, 40, 21, 63, 25, 0, 27, 34, 5, 30, 37, 50, 54, 57, 9, 19,
    35, 60, 12, 10, 61, 14, 48, 49, 56, 53, 39, 32, 9, 5, 30, 37, 44, 51, 48, 49, 50, 16, 53, 54,
    6, 57, 7, 8, 47, 23, 25, 26, 1, 32, 33, 2, 40, 3, 4, 12, 39, 45, 17, 63, 28, 29, 14, 56, 46,
    18, 62, 21, 36, 15, 58, 59, 19, 22, 24, 0, 27, 34, 41, 20, 61, 35, 42, 60, 10, 43, 11, 13, 52,
    55, 31, 38, 10, 21, 3, 17, 31, 32, 7, 27, 52, 53, 16, 13, 14, 42, 48, 60, 35, 18, 2, 38, 39,
    51, 24, 25, 29, 30, 62, 34, 1, 36, 37, 19, 20, 40, 6, 0, 22, 23, 4, 5, 45, 55, 43, 11, 63, 46,
    33, 56, 49, 12, 61, 28, 41, 47, 57, 8, 9, 58, 59, 54, 50, 15, 44, 26, 11, 22, 4, 38, 17, 33, 8,
    55, 27, 54, 43, 16, 15, 10, 61, 48, 42, 19, 31, 2, 45, 36, 51, 26, 21, 63, 30, 41, 24, 1, 44,
    3, 46, 20, 52, 13, 28, 62, 18, 40, 32, 6, 0, 35, 23, 5, 39, 58, 50, 60, 12, 29, 7, 59, 47, 34,
    57, 53, 9, 56, 14, 49, 25, 37, 12, 23, 5, 39, 45, 17, 9, 56, 58, 27, 49, 50, 16, 60, 10, 11,
    48, 20, 32, 33, 2, 37, 44, 51, 62, 21, 22, 47, 25, 26, 1, 40, 3, 4, 53, 14, 63, 28, 46, 18, 38,
    54, 15, 61, 29, 19, 31, 6, 0, 35, 42, 30, 57, 7, 8, 59, 34, 55, 41, 52, 43, 13, 36, 24, 13, 24,
    31, 4, 18, 40, 52, 8, 34, 57, 11, 35, 60, 16, 15, 49, 43, 38, 19, 3, 46, 22, 28, 62, 51, 26,
    37, 55, 29, 21, 63, 17, 33, 39, 41, 42, 1, 44, 2, 45, 20, 7, 10, 0, 25, 32, 5, 59, 61, 50, 14,
    36, 27, 54, 56, 6, 58, 47, 53, 9, 48, 12, 30, 23, 14, 25, 32, 5, 46, 18, 53, 9, 59, 34, 12, 61,
    35, 50, 16, 13, 49, 39, 20, 40, 3, 23, 63, 28, 44, 51, 24, 56, 30, 62, 21, 45, 17, 31, 47, 48,
    26, 1, 33, 2, 4, 57, 60, 15, 36, 38, 19, 7, 10, 0, 43, 37, 58, 27, 52, 54, 6, 8, 55, 41, 11,
    42, 22, 29, 15, 26, 33, 40, 5, 19, 54, 57, 9, 41, 60, 12, 42, 14, 43, 16, 50, 45, 46, 20, 4,
    62, 23, 29, 25, 36, 51, 58, 63, 30, 22, 32, 38, 17, 59, 61, 37, 24, 39, 31, 18, 47, 48, 49, 1,
    2, 3, 8, 11, 13, 0, 44, 53, 55, 27, 56, 52, 34, 6, 7, 35, 10, 28, 21, 16, 51, 17, 18, 19, 20,
    27, 34, 41, 47, 35, 42, 48, 43, 49, 50, 0, 2, 3, 4, 5, 28, 29, 30, 36, 37, 44, 6, 21, 22, 23,
    38, 39, 45, 7, 10, 24, 25, 31, 32, 46, 8, 11, 13, 26, 33, 40, 9, 12, 14, 15, 1, 55, 56, 58, 52,
    53, 59, 54, 57, 61, 60, 63, 62, 17, 6, 0, 35, 42, 48, 51, 21, 22, 23, 3, 4, 5, 38, 39, 45, 2,
    16, 10, 11, 12, 34, 41, 47, 52, 53, 54, 1, 7, 8, 9, 13, 14, 15, 28, 18, 55, 56, 43, 49, 61, 29,
    19, 31, 58, 50, 60, 30, 20, 32, 33, 27, 36, 37, 44, 24, 25, 62, 26, 63, 40, 46, 59, 57, 18, 7,
    10, 0, 43, 49, 28, 51, 24, 25, 17, 31, 32, 4, 5, 46, 3, 35, 16, 13, 14, 6, 55, 56, 41, 47, 57,
    21, 27, 52, 53, 42, 48, 60, 1, 2, 8, 9, 11, 12, 15, 36, 38, 19, 59, 61, 50, 37, 39, 20, 40, 34,
    22, 23, 63, 29, 30, 44, 62, 26, 45, 33, 54, 58, 19, 8, 11, 13, 0, 50, 29, 36, 51, 26, 38, 17,
    33, 18, 40, 5, 4, 42, 43, 16, 15, 52, 6, 58, 7, 59, 47, 22, 55, 27, 54, 10, 61, 48, 24, 31, 34,
    57, 35, 60, 49, 1, 2, 3, 9, 12, 14, 44, 45, 46, 20, 41, 28, 62, 23, 21, 63, 25, 30, 37, 32, 39,
    56, 53, 20, 9, 12, 14, 15, 0, 30, 37, 44, 51, 39, 45, 17, 46, 18, 19, 5, 48, 49, 50, 16, 53,
    54, 6, 57, 7, 8, 23, 56, 58, 27, 60, 10, 11, 25, 32, 59, 34, 61, 35, 13, 26, 33, 40, 41, 42,
    43, 1, 2, 3, 4, 47, 63, 28, 29, 62, 21, 36, 22, 24, 38, 31, 52, 55, 21, 35, 7, 27, 52, 53, 18,
    2, 38, 39, 51, 24, 25, 29, 30, 62, 28, 34, 6, 55, 56, 0, 43, 49, 11, 12, 61, 3, 16, 13, 14, 41,
    47, 57, 17, 1, 42, 48, 8, 9, 58, 31, 36, 22, 60, 59, 54, 32, 37, 23, 63, 10, 4, 5, 46, 19, 20,
    33, 40, 45, 44, 26, 15, 50, 22, 42, 8, 55, 27, 54, 19, 31, 2, 45, 36, 51, 26, 21, 63, 30, 29,
    41, 52, 6, 58, 13, 0, 50, 35, 60, 12, 4, 43, 16, 15, 7, 59, 47, 38, 24, 10, 61, 34, 57, 53, 17,
    1, 28, 48, 9, 56, 33, 44, 62, 23, 11, 18, 40, 5, 3, 46, 39, 20, 32, 25, 37, 49, 14, 23, 48, 9,
    56, 58, 27, 20, 32, 33, 2, 37, 44, 51, 62, 21, 22, 30, 47, 53, 54, 6, 14, 15, 0, 61, 35, 42, 5,
    49, 50, 16, 57, 7, 8, 39, 25, 60, 10, 59, 34, 55, 45, 26, 63, 11, 41, 52, 17, 1, 28, 29, 12,
    46, 18, 19, 40, 3, 31, 4, 38, 36, 24, 13, 43, 24, 43, 52, 8, 34, 57, 38, 19, 3, 46, 22, 28, 62,
    51, 26, 37, 36, 55, 41, 7, 59, 42, 10, 61, 0, 50, 14, 31, 11, 35, 60, 27, 54, 56, 4, 29, 16,
    15, 6, 58, 47, 18, 21, 1, 49, 53, 9, 40, 63, 44, 25, 13, 2, 45, 32, 17, 33, 5, 39, 20, 30, 23,
    12, 48, 25, 49, 53, 9, 59, 34, 39, 20, 40, 3, 23, 63, 28, 44, 51, 24, 37, 56, 47, 57, 7, 48,
    60, 10, 15, 0, 43, 32, 12, 61, 35, 58, 27, 52, 5, 30, 50, 16, 54, 6, 8, 46, 62, 26, 13, 55, 41,
    18, 21, 1, 36, 14, 33, 2, 38, 45, 17, 19, 31, 4, 22, 29, 42, 11, 26, 50, 54, 57, 9, 41, 45, 46,
    20, 4, 62, 23, 29, 25, 36, 51, 44, 58, 59, 47, 8, 61, 48, 11, 49, 13, 0, 33, 60, 12, 42, 53,
    55, 27, 40, 63, 14, 43, 56, 52, 34, 5, 30, 37, 16, 6, 7, 19, 22, 24, 1, 15, 39, 31, 2, 32, 38,
    3, 17, 18, 28, 21, 10, 35, 27, 17, 1, 28, 29, 30, 0, 35, 42, 48, 7, 8, 9, 55, 56, 58, 6, 51,
    21, 22, 23, 3, 4, 5, 38, 39, 45, 16, 18, 19, 20, 24, 25, 26, 10, 34, 31, 32, 36, 37, 63, 11,
    41, 52, 33, 44, 62, 12, 47, 53, 54, 2, 13, 14, 15, 43, 49, 61, 50, 60, 57, 59, 40, 46, 28, 10,
    34, 6, 55, 56, 3, 17, 31, 32, 1, 36, 37, 22, 23, 63, 21, 7, 27, 52, 53, 16, 13, 14, 42, 48, 60,
    18, 0, 43, 49, 8, 9, 59, 2, 51, 11, 12, 41, 47, 54, 38, 24, 29, 61, 57, 58, 39, 25, 30, 62, 35,
    19, 20, 40, 4, 5, 45, 46, 33, 26, 44, 50, 15, 29, 11, 41, 52, 6, 58, 4, 38, 17, 33, 24, 1, 44,
    28, 62, 23, 22, 8, 55, 27, 54, 43, 16, 15, 10, 61, 48, 19, 13, 0, 50, 34, 57, 9, 31, 36, 35,
    60, 7, 59, 56, 2, 51, 21, 12, 47, 53, 45, 26, 63, 30, 42, 3, 46, 20, 18, 40, 32, 5, 39, 37, 25,
    14, 49, 30, 12, 47, 53, 54, 6, 5, 39, 45, 17, 25, 26, 1, 63, 28, 29, 23, 9, 56, 58, 27, 49, 50,
    16, 60, 10, 11, 20, 14, 15, 0, 59, 34, 41, 32, 37, 61, 35, 57, 7, 52, 33, 44, 62, 42, 8, 55, 2,
    51, 21, 22, 48, 40, 3, 4, 46, 18, 38, 19, 31, 24, 36, 43, 13, 31, 55, 43, 11, 35, 60, 24, 29,
    21, 63, 19, 3, 46, 17, 33, 39, 38, 13, 42, 10, 61, 8, 34, 57, 6, 58, 53, 36, 41, 7, 59, 0, 50,
    14, 22, 4, 27, 54, 16, 15, 48, 28, 18, 2, 56, 49, 12, 62, 40, 45, 32, 52, 51, 26, 37, 1, 44,
    23, 25, 30, 5, 20, 47, 9, 32, 56, 49, 12, 61, 35, 25, 30, 62, 21, 20, 40, 3, 45, 17, 31, 39,
    14, 48, 60, 10, 9, 59, 34, 54, 6, 55, 37, 47, 57, 7, 15, 0, 43, 23, 5, 58, 27, 50, 16, 11, 63,
    46, 33, 52, 13, 42, 28, 18, 2, 38, 53, 44, 51, 24, 26, 1, 29, 36, 22, 19, 4, 8, 41, 33, 58, 50,
    60, 12, 42, 26, 63, 30, 22, 46, 20, 4, 32, 38, 17, 45, 15, 61, 48, 11, 57, 9, 41, 56, 52, 6,
    44, 59, 47, 8, 49, 13, 0, 62, 40, 53, 55, 14, 43, 35, 23, 5, 39, 27, 16, 10, 29, 19, 31, 2, 54,
    25, 36, 51, 37, 24, 21, 1, 28, 3, 18, 34, 7, 34, 18, 21, 1, 36, 37, 10, 0, 43, 49, 27, 52, 53,
    8, 9, 59, 7, 28, 51, 24, 25, 17, 31, 32, 4, 5, 46, 35, 2, 38, 39, 29, 30, 62, 16, 6, 19, 20,
    22, 23, 26, 13, 55, 41, 40, 63, 44, 14, 56, 47, 57, 3, 42, 48, 60, 11, 12, 15, 61, 50, 58, 54,
    45, 33, 35, 28, 18, 2, 38, 39, 34, 6, 55, 56, 0, 43, 49, 11, 12, 61, 10, 3, 17, 31, 32, 1, 36,
    37, 22, 23, 63, 7, 51, 24, 25, 4, 5, 46, 27, 16, 29, 30, 19, 20, 33, 52, 13, 42, 62, 40, 45,
    53, 14, 48, 60, 21, 8, 9, 59, 41, 47, 54, 57, 58, 15, 50, 26, 44, 36, 13, 55, 41, 7, 59, 31, 4,
    18, 40, 29, 21, 63, 1, 44, 25, 24, 52, 8, 34, 57, 11, 35, 60, 16, 15, 49, 38, 42, 10, 61, 6,
    58, 53, 19, 22, 0, 50, 27, 54, 9, 3, 28, 51, 14, 56, 47, 46, 62, 26, 37, 43, 17, 33, 39, 2, 45,
    20, 32, 5, 23, 30, 48, 12, 37, 14, 56, 47, 57, 7, 32, 5, 46, 18, 30, 62, 21, 26, 1, 36, 25, 53,
    9, 59, 34, 12, 61, 35, 50, 16, 13, 39, 48, 60, 10, 54, 6, 55, 20, 23, 15, 0, 58, 27, 41, 40,
    63, 44, 43, 52, 8, 3, 28, 51, 24, 49, 45, 17, 31, 33, 2, 4, 38, 19, 29, 22, 11, 42, 38, 52, 13,
    42, 10, 61, 36, 22, 28, 62, 4, 18, 40, 2, 45, 32, 31, 43, 11, 35, 60, 41, 7, 59, 27, 54, 56,
    24, 8, 34, 57, 16, 15, 49, 29, 19, 6, 58, 0, 50, 12, 21, 3, 17, 53, 14, 48, 63, 46, 33, 39, 55,
    1, 44, 25, 51, 26, 30, 37, 23, 20, 5, 9, 47, 39, 53, 14, 48, 60, 10, 37, 23, 63, 28, 5, 46, 18,
    33, 2, 38, 32, 49, 12, 61, 35, 47, 57, 7, 58, 27, 52, 25, 9, 59, 34, 50, 16, 13, 30, 20, 54, 6,
    15, 0, 42, 62, 40, 45, 55, 43, 11, 21, 3, 17, 31, 56, 26, 1, 36, 44, 51, 22, 24, 29, 4, 19, 41,
    8, 40, 59, 61, 50, 14, 43, 62, 26, 37, 24, 33, 39, 31, 20, 4, 18, 46, 60, 15, 49, 13, 58, 53,
    55, 9, 41, 7, 63, 54, 56, 52, 12, 42, 10, 44, 45, 47, 8, 48, 11, 0, 25, 32, 5, 34, 35, 16, 36,
    38, 19, 3, 57, 30, 22, 28, 23, 29, 51, 21, 1, 17, 2, 6, 27, 41, 19, 22, 24, 1, 44, 11, 13, 0,
    50, 55, 27, 54, 34, 57, 9, 8, 29, 36, 51, 26, 38, 17, 33, 18, 40, 5, 42, 31, 2, 45, 21, 63, 30,
    43, 52, 3, 46, 28, 62, 37, 16, 6, 7, 20, 23, 25, 15, 58, 59, 47, 4, 10, 61, 48, 35, 60, 49, 12,
    14, 53, 56, 32, 39, 42, 29, 19, 31, 2, 45, 41, 52, 6, 58, 13, 0, 50, 35, 60, 12, 11, 4, 38, 17,
    33, 24, 1, 44, 28, 62, 23, 8, 36, 51, 26, 18, 40, 5, 55, 43, 21, 63, 3, 46, 39, 27, 16, 10, 30,
    20, 32, 54, 15, 61, 48, 22, 34, 57, 9, 7, 59, 56, 47, 53, 49, 14, 37, 25, 43, 36, 38, 19, 3,
    46, 55, 41, 7, 59, 42, 10, 61, 0, 50, 14, 13, 31, 4, 18, 40, 29, 21, 63, 1, 44, 25, 52, 22, 28,
    62, 2, 45, 32, 8, 11, 51, 26, 17, 33, 5, 34, 35, 16, 37, 39, 20, 57, 60, 15, 49, 24, 6, 58, 53,
    27, 54, 9, 56, 47, 12, 48, 23, 30, 44, 15, 58, 59, 47, 8, 33, 40, 5, 19, 63, 30, 22, 37, 24, 1,
    26, 54, 57, 9, 41, 60, 12, 42, 14, 43, 16, 45, 61, 48, 11, 56, 52, 6, 46, 62, 49, 13, 53, 55,
    7, 20, 23, 25, 0, 27, 34, 4, 29, 36, 51, 50, 32, 38, 17, 39, 31, 18, 2, 3, 21, 28, 35, 10, 45,
    54, 15, 61, 48, 11, 44, 62, 23, 29, 40, 5, 19, 39, 31, 2, 33, 50, 60, 12, 42, 59, 47, 8, 53,
    55, 27, 26, 57, 9, 41, 14, 43, 16, 63, 46, 56, 52, 49, 13, 10, 30, 20, 32, 6, 0, 35, 22, 4, 38,
    17, 58, 37, 24, 1, 25, 36, 28, 51, 21, 18, 3, 7, 34, 46, 57, 60, 15, 49, 13, 63, 44, 25, 36,
    45, 32, 38, 5, 19, 3, 40, 61, 50, 14, 43, 54, 56, 52, 47, 8, 34, 62, 58, 53, 55, 48, 11, 35,
    26, 33, 9, 41, 12, 42, 16, 37, 39, 20, 7, 10, 0, 24, 31, 4, 18, 59, 23, 29, 21, 30, 22, 1, 28,
    51, 2, 17, 27, 6, 47, 20, 23, 25, 26, 1, 12, 14, 15, 0, 56, 58, 27, 59, 34, 41, 9, 30, 37, 44,
    51, 39, 45, 17, 46, 18, 19, 48, 32, 33, 2, 62, 21, 22, 49, 53, 40, 3, 63, 28, 24, 50, 54, 57,
    4, 29, 36, 16, 6, 7, 8, 5, 60, 10, 11, 61, 35, 13, 42, 43, 55, 52, 38, 31, 48, 30, 20, 32, 33,
    2, 47, 53, 54, 6, 14, 15, 0, 61, 35, 42, 12, 5, 39, 45, 17, 25, 26, 1, 63, 28, 29, 9, 37, 44,
    51, 46, 18, 19, 56, 49, 62, 21, 40, 3, 31, 58, 50, 60, 22, 4, 38, 27, 16, 10, 11, 23, 59, 34,
    41, 57, 7, 52, 8, 55, 13, 43, 24, 36, 49, 37, 39, 20, 40, 3, 56, 47, 57, 7, 48, 60, 10, 15, 0,
    43, 14, 32, 5, 46, 18, 30, 62, 21, 26, 1, 36, 53, 23, 63, 28, 33, 2, 38, 9, 12, 44, 51, 45, 17,
    19, 59, 61, 50, 24, 31, 4, 34, 35, 16, 13, 25, 54, 6, 55, 58, 27, 41, 52, 8, 42, 11, 29, 22,
    50, 44, 45, 46, 20, 4, 58, 59, 47, 8, 61, 48, 11, 49, 13, 0, 15, 33, 40, 5, 19, 63, 30, 22, 37,
    24, 1, 54, 62, 23, 29, 39, 31, 2, 57, 60, 25, 36, 32, 38, 3, 9, 12, 14, 51, 17, 18, 41, 42, 43,
    16, 26, 56, 52, 6, 53, 55, 7, 27, 34, 10, 35, 21, 28, 51, 0, 27, 34, 41, 47, 2, 3, 4, 5, 28,
    29, 30, 36, 37, 44, 1, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 35, 42, 48, 55, 56, 58, 18, 21,
    43, 49, 52, 53, 59, 19, 22, 24, 50, 54, 57, 20, 23, 25, 26, 16, 31, 32, 33, 38, 39, 40, 45, 46,
    63, 62, 60, 61, 52, 31, 36, 22, 28, 62, 43, 11, 35, 60, 41, 7, 59, 27, 54, 56, 55, 24, 29, 21,
    63, 19, 3, 46, 17, 33, 39, 13, 4, 18, 40, 1, 44, 25, 42, 8, 2, 45, 51, 26, 30, 10, 34, 6, 32,
    37, 23, 61, 57, 58, 53, 38, 0, 50, 14, 16, 15, 48, 49, 12, 9, 47, 5, 20, 53, 32, 37, 23, 63,
    28, 49, 12, 61, 35, 47, 57, 7, 58, 27, 52, 56, 25, 30, 62, 21, 20, 40, 3, 45, 17, 31, 14, 5,
    46, 18, 26, 1, 36, 48, 9, 33, 2, 44, 51, 22, 60, 59, 54, 38, 24, 29, 10, 34, 6, 55, 39, 15, 0,
    43, 50, 16, 11, 13, 42, 41, 8, 19, 4, 54, 33, 44, 62, 23, 29, 50, 60, 12, 42, 59, 47, 8, 53,
    55, 27, 58, 26, 63, 30, 22, 46, 20, 4, 32, 38, 17, 15, 40, 5, 19, 37, 24, 1, 61, 57, 39, 31,
    25, 36, 28, 48, 9, 56, 2, 51, 21, 11, 41, 52, 6, 45, 49, 13, 0, 14, 43, 35, 16, 10, 7, 34, 3,
    18, 55, 38, 24, 29, 21, 63, 13, 42, 10, 61, 8, 34, 57, 6, 58, 53, 52, 36, 22, 28, 62, 4, 18,
    40, 2, 45, 32, 43, 19, 3, 46, 51, 26, 37, 11, 41, 17, 33, 1, 44, 23, 35, 7, 27, 39, 25, 30, 60,
    59, 54, 56, 31, 16, 15, 49, 0, 50, 12, 14, 48, 47, 9, 20, 5, 56, 39, 25, 30, 62, 21, 14, 48,
    60, 10, 9, 59, 34, 54, 6, 55, 53, 37, 23, 63, 28, 5, 46, 18, 33, 2, 38, 49, 20, 40, 3, 44, 51,
    24, 12, 47, 45, 17, 26, 1, 29, 61, 57, 58, 31, 36, 22, 35, 7, 27, 52, 32, 50, 16, 13, 15, 0,
    42, 43, 11, 8, 41, 4, 19, 57, 40, 63, 44, 25, 36, 61, 50, 14, 43, 54, 56, 52, 47, 8, 34, 59,
    62, 26, 37, 24, 33, 39, 31, 20, 4, 18, 60, 45, 32, 38, 23, 29, 21, 15, 58, 5, 19, 30, 22, 1,
    49, 53, 9, 3, 28, 51, 13, 55, 41, 7, 46, 12, 42, 10, 48, 11, 0, 35, 16, 27, 6, 17, 2, 58, 45,
    26, 63, 30, 22, 15, 61, 48, 11, 57, 9, 41, 56, 52, 6, 54, 44, 62, 23, 29, 40, 5, 19, 39, 31, 2,
    50, 46, 20, 4, 25, 36, 51, 60, 59, 32, 38, 37, 24, 21, 12, 47, 53, 17, 1, 28, 42, 8, 55, 27,
    33, 14, 43, 16, 49, 13, 10, 0, 35, 34, 7, 18, 3, 59, 46, 62, 26, 37, 24, 60, 15, 49, 13, 58,
    53, 55, 9, 41, 7, 57, 63, 44, 25, 36, 45, 32, 38, 5, 19, 3, 61, 33, 39, 31, 30, 22, 28, 50, 54,
    20, 4, 23, 29, 51, 14, 56, 47, 18, 21, 1, 43, 52, 8, 34, 40, 48, 11, 35, 12, 42, 16, 10, 0, 6,
    27, 2, 17, 60, 62, 40, 45, 32, 38, 57, 58, 53, 55, 50, 14, 43, 48, 11, 35, 61, 46, 33, 39, 31,
    44, 25, 36, 30, 22, 28, 59, 26, 37, 24, 20, 4, 18, 54, 15, 23, 29, 5, 19, 2, 56, 49, 12, 21, 3,
    17, 52, 13, 42, 10, 63, 47, 8, 34, 9, 41, 6, 7, 27, 0, 16, 1, 51, 61, 63, 46, 33, 39, 31, 59,
    54, 56, 52, 15, 49, 13, 12, 42, 10, 60, 40, 45, 32, 38, 26, 37, 24, 23, 29, 21, 57, 44, 25, 36,
    5, 19, 3, 58, 50, 30, 22, 20, 4, 17, 53, 14, 48, 28, 18, 2, 55, 43, 11, 35, 62, 9, 41, 7, 47,
    8, 27, 34, 6, 16, 0, 51, 1, 62, 61, 57, 58, 53, 55, 46, 33, 39, 31, 44, 25, 36, 30, 22, 28, 63,
    59, 54, 56, 52, 15, 49, 13, 12, 42, 10, 40, 50, 14, 43, 47, 8, 34, 45, 26, 48, 11, 9, 41, 6,
    32, 37, 23, 35, 7, 27, 38, 24, 29, 21, 60, 5, 19, 3, 20, 4, 17, 18, 2, 1, 51, 16, 0, 63, 60,
    59, 54, 56, 52, 40, 45, 32, 38, 26, 37, 24, 23, 29, 21, 62, 57, 58, 53, 55, 50, 14, 43, 48, 11,
    35, 46, 15, 49, 13, 9, 41, 7, 33, 44, 12, 42, 47, 8, 27, 39, 25, 30, 10, 34, 6, 31, 36, 22, 28,
    61, 20, 4, 18, 5, 19, 2, 3, 17, 51, 1, 0, 16,
];

const ROOTS: [[i8; 8]; 16] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
];

const BFS: [u8; 256] = [
    0, 16, 7, 42, 3, 8, 9, 27, 5, 24, 4, 47, 2, 12, 1, 34, 16, 0, 34, 1, 12, 2, 47, 4, 24, 5, 27,
    9, 8, 3, 42, 7, 34, 7, 0, 26, 11, 9, 14, 10, 6, 4, 5, 2, 47, 1, 3, 16, 1, 42, 3, 0, 15, 4, 5,
    8, 13, 9, 2, 10, 27, 7, 16, 48, 12, 3, 1, 7, 0, 5, 6, 9, 2, 41, 13, 4, 35, 16, 15, 11, 2, 8,
    47, 27, 24, 0, 7, 1, 3, 12, 42, 34, 16, 5, 4, 9, 47, 9, 2, 10, 4, 34, 0, 3, 11, 1, 26, 16, 7,
    6, 5, 14, 4, 27, 5, 2, 13, 42, 48, 0, 7, 15, 16, 3, 1, 9, 8, 10, 35, 5, 4, 9, 41, 12, 1, 15, 0,
    16, 7, 11, 3, 2, 13, 6, 5, 35, 6, 13, 2, 3, 11, 7, 16, 0, 15, 1, 12, 41, 9, 4, 27, 4, 10, 8, 9,
    1, 3, 16, 15, 7, 0, 48, 42, 13, 2, 5, 9, 47, 14, 5, 6, 7, 16, 26, 1, 11, 3, 0, 34, 4, 10, 2, 8,
    2, 9, 4, 5, 16, 34, 42, 12, 3, 1, 7, 0, 24, 27, 47, 3, 12, 11, 15, 16, 35, 4, 13, 41, 2, 9, 6,
    5, 0, 7, 1, 42, 1, 48, 16, 7, 27, 10, 2, 9, 13, 8, 5, 4, 15, 0, 3, 7, 34, 16, 3, 1, 47, 2, 5,
    4, 6, 10, 14, 9, 11, 26, 0,
];

const BRANCH: [[u8; 3]; 16] = [
    [0, 14, 3],
    [1, 3, 14],
    [2, 13, 4],
    [3, 0, 1],
    [4, 2, 15],
    [5, 7, 10],
    [6, 9, 8],
    [7, 12, 5],
    [8, 6, 11],
    [9, 11, 6],
    [10, 5, 12],
    [11, 8, 9],
    [12, 10, 7],
    [13, 15, 2],
    [14, 1, 0],
    [15, 4, 13],
];

const IDENTITY: u8 = 0;
const OPERATOR_KAPPA: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
const COMPOSITION_KAPPA: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";

/// Read operator of a discrete checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteReadMode {
    Route,
    Flock {
        kept: u32,
    },
    Relation {
        kept: u32,
        sweeps: u32,
    },
    /// Hard copy read (device spec word `s[8] = 3`): picks the latest earlier
    /// position whose `(p - 1, p)` block codes are nearest the query pair by
    /// XOR + popcount and adds the gated successor blocks.
    Pick,
}

/// Route trace ABI shared with `discrete.cu`.
pub const DISCRETE_ROUTE_TRACE_HEADER_WORDS: usize = 4;
pub const DISCRETE_ROUTE_TRACE_STAGE_WORDS: usize = 6;
/// Active-leaf trace ABI shared with `discrete.cu`.
pub const DISCRETE_ACTIVE_TRACE_HEADER_WORDS: usize = 4;
pub const DISCRETE_ACTIVE_TRACE_LEVEL_WORDS: usize = 28;

/// Objective policy recorded in the discrete header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteObjectivePolicy {
    NearestAddressMarginV1,
    OccupiedCellRankHingeV1,
}

/// Shape and read/writer metadata of a discrete `UORLUT01` artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutShape {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
    pub stages: u32,
    pub read_mode: String,
    pub read_kept: u32,
    pub read_sweeps: u32,
    /// Position transport word (device `s[14]`): 0 legacy BFS, 1 causal prefix
    /// holonomy, 2 matched endpoint, 3 identity. Absent in the header means 0.
    #[serde(default, skip_serializing_if = "is_zero_word")]
    pub read_position: u32,
    pub objective_policy: String,
    pub read_initialization: String,
    pub seed: u32,
    pub epoch: u64,
}

/// Provenance recorded in `source` of a discrete `UORLUT01` artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutSource {
    pub snapshot_schema: String,
    pub snapshot_sha256: String,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SectionSpan {
    offset: u64,
    bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SectionKind {
    I16,
    I32,
    U32,
}

impl SectionKind {
    fn width(self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I32 | Self::U32 => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SectionSpec {
    name: String,
    kind: SectionKind,
    len: usize,
    span: SectionSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DiscreteLutHeader {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tokenizer_sha256: Option<String>,
    schema: String,
    shape: DiscreteLutShape,
    matrices: Vec<serde_json::Value>,
    tables: Vec<SectionSpec>,
    source: DiscreteLutSource,
}

/// Output of evaluating a causal prefix on an [`IntegerDiscreteModel`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscretePrefixEvaluation {
    /// Geometric trunk read vector (empty for the count mixer).
    pub read_vector: Vec<i32>,
    /// Occupied geometric writer cells in distance order (empty for the count mixer).
    pub admitted_cells: Vec<u32>,
    /// Up to eight original IDs, ordered by trunk distance or decreasing count mass, then ID.
    pub top8: Vec<u32>,
    /// Top-1 predicted token (`top8[0]`).
    pub prediction: u32,
    /// Per-node relation scores (`blocks * nodes_count`) when `read_mode` is `Relation`.
    pub relation_scores: Option<Vec<i32>>,
}

/// Persistent u32 words of the opt-in occurrence-addressed writer. The source
/// is fixed when a rollout starts; cursor and source index the immutable
/// original prompt, even after generated tokens are appended to the prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCopyState {
    pub active: u32,
    pub source: u32,
    pub cursor: u32,
    pub prompt_end: u32,
}

impl ExactCopyState {
    pub fn new(source: Option<u32>, prompt_end: u32) -> Self {
        let position = source.unwrap_or(u32::MAX);
        Self {
            active: u32::from(source.is_some()),
            source: position,
            cursor: position,
            prompt_end,
        }
    }

    /// Raw serving and CUDA ABI, suitable for lossless replay.
    pub fn words(self) -> [u32; 4] {
        [self.active, self.source, self.cursor, self.prompt_end]
    }

    pub fn from_words([active, source, cursor, prompt_end]: [u32; 4]) -> Self {
        Self {
            active,
            source,
            cursor,
            prompt_end,
        }
    }

    pub fn valid_for(self, prefix_length: usize) -> bool {
        self.prompt_end as usize <= prefix_length
            && self.active <= 1
            && ((self.source == u32::MAX && self.cursor == u32::MAX && self.active == 0)
                || (self.source < self.prompt_end
                    && self.source <= self.cursor
                    && self.cursor <= self.prompt_end
                    && self.cursor - self.source <= 8))
    }

    /// Apply the final writer to a scored causal prefix. A terminator releases
    /// the cursor on its own step, which emits the ordinary native prediction.
    pub fn write(self, prefix: &[u32], prediction: u32, stops: [u32; 4]) -> Result<ExactCopyStep> {
        if !self.valid_for(prefix.len()) {
            return Err(invalid("invalid exact-copy state or original prompt fence"));
        }
        let mut step = ExactCopyStep {
            state: self,
            emitted: prediction,
            exact_id: u32::MAX,
            copied: 0,
            release: 0,
        };
        if self.active == 0 {
            return Ok(step);
        }
        step.release = if self.cursor - self.source == 8 {
            3
        } else if self.cursor == self.prompt_end {
            2
        } else if stops.contains(&prefix[self.cursor as usize]) {
            1
        } else {
            let id = prefix[self.cursor as usize];
            step.state.cursor += 1;
            step.emitted = id;
            step.exact_id = id;
            step.copied = 1;
            0
        };
        if step.release != 0 {
            step.state.active = 0;
        }
        Ok(step)
    }
}

/// One committed writer action. On Generate, exact_id is u32::MAX. Release is
/// 0 (none), 1 (delimiter), 2 (original prompt fence), or 3 (eight-token cap).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCopyStep {
    pub state: ExactCopyState,
    pub emitted: u32,
    pub exact_id: u32,
    pub copied: u32,
    pub release: u32,
}

impl ExactCopyStep {
    /// A delimiter was observed after at least one exact source token. A fence,
    /// eight-token cap, inactive cursor or delimiter at the start is insufficient.
    pub fn completed_span(self) -> bool {
        self.release == 1 && self.state.cursor > self.state.source
    }

    /// Opt-in final emission for bounded short answers after a completed copy.
    pub fn finish_with_eos(mut self, enabled: bool, eos: u32) -> Self {
        if enabled && self.completed_span() {
            self.emitted = eos;
        }
        self
    }

    pub fn words(self) -> [u32; 8] {
        [
            self.state.active,
            self.state.source,
            self.state.cursor,
            self.state.prompt_end,
            self.emitted,
            self.exact_id,
            self.copied,
            self.release,
        ]
    }

    pub fn from_words(
        [active, source, cursor, prompt_end, emitted, exact_id, copied, release]: [u32; 8],
    ) -> Self {
        Self {
            state: ExactCopyState::from_words([active, source, cursor, prompt_end]),
            emitted,
            exact_id,
            copied,
            release,
        }
    }
}

/// A production-prefix evaluation plus the exact read-decision words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteTracedPrefixEvaluation {
    pub evaluation: DiscretePrefixEvaluation,
    /// Raw base trace words in `(block, stage)` order. For a count mixer, the trace
    /// is the previous original ID, conditional weight, winning ID and Q32 mass (low, high).
    pub trace_words: Vec<u32>,
    /// Relation scores padded to the artifact context's maximum binder-node
    /// count, exactly as CUDA stores its batch tail. `u32::MAX` marks padding.
    pub relation_trace_words: Option<Vec<u32>>,
}

/// Target-supervised writer loss and rival counts for one prefix evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscreteTargetObservation {
    pub evaluation: DiscretePrefixEvaluation,
    pub token_rivals: u32,
    pub address_rivals: u32,
    pub token_hinge: u64,
    pub address_hinge: u64,
    pub rank_hinge: u64,
    pub loss_ta: u64,
    pub loss_tr: u64,
    pub selected_loss: u64,
}

/// Sparse integer count table keyed by `u32` (16-bit or 28-bit state keys) -> `u16` token -> `u32` count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DiscreteSparseCountTable {
    entries: BTreeMap<u32, BTreeMap<u16, u32>>,
}

impl DiscreteSparseCountTable {
    fn from_triples(words: &[u32], vocabulary: usize) -> Result<Self> {
        if words.len() % 3 != 0 {
            return Err(invalid(
                "sparse trunk count table word length must be a multiple of 3",
            ));
        }
        let mut entries: BTreeMap<u32, BTreeMap<u16, u32>> = BTreeMap::new();
        for chunk in words.chunks_exact(3) {
            let key = chunk[0];
            let tok_u32 = chunk[1];
            let cnt = chunk[2];
            if (tok_u32 as usize) >= vocabulary || cnt == 0 {
                return Err(invalid(
                    "sparse trunk count table contains invalid token or zero count",
                ));
            }
            let tok = u16::try_from(tok_u32)
                .map_err(|_| invalid("sparse trunk count table token overflows u16"))?;
            let map = entries.entry(key).or_default();
            if map.insert(tok, cnt).is_some() {
                return Err(invalid(
                    "duplicate (key, token) entry in sparse trunk count table",
                ));
            }
        }
        Ok(Self { entries })
    }

    fn get(&self, key: u32, tok: u16) -> u32 {
        self.entries
            .get(&key)
            .and_then(|m| m.get(&tok).copied())
            .unwrap_or(0)
    }

    fn entries(&self, key: u32) -> Option<&BTreeMap<u16, u32>> {
        self.entries.get(&key)
    }
}

pub(crate) fn pack_4block_roots(roots: [u8; 4]) -> u16 {
    u16::from(roots[0] & 0xF)
        | (u16::from(roots[1] & 0xF) << 4)
        | (u16::from(roots[2] & 0xF) << 8)
        | (u16::from(roots[3] & 0xF) << 12)
}

fn orbit_root_causal_successor(prefix: &[u32], codebook_roots: &[[u8; 4]]) -> Result<Option<u16>> {
    if prefix.len() < 2 {
        return Ok(None);
    }
    let query_tok = prefix[prefix.len() - 1] as usize;
    let query_roots = codebook_roots
        .get(query_tok)
        .copied()
        .ok_or_else(|| invalid("query_tok out of codebook_roots bounds"))?;
    let mut matched_succ = None;
    for p in 0..(prefix.len() - 1) {
        let p_tok = prefix[p] as usize;
        let p_roots = codebook_roots
            .get(p_tok)
            .copied()
            .ok_or_else(|| invalid("prefix token out of codebook_roots bounds"))?;
        if p_roots == query_roots {
            let succ_tok = u16::try_from(prefix[p + 1])
                .map_err(|_| invalid("successor token overflows u16"))?;
            if usize::from(succ_tok) >= codebook_roots.len() {
                return Err(invalid("successor token out of codebook_roots bounds"));
            }
            matched_succ = Some(succ_tok);
        }
    }
    Ok(matched_succ)
}

/// Validated on load, then searched directly in its serialized representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenS2Table {
    bytes: Box<[u8]>,
}

impl FrozenS2Table {
    fn load(bytes: &[u8], vocabulary: usize) -> Result<Self> {
        let rows = bytes
            .get(S2_HEADER_BYTES..)
            .ok_or_else(|| invalid("short S2 header"))?;
        if !rows.len().is_multiple_of(S2_ROW_BYTES) {
            return Err(invalid("partial S2 record"));
        }
        let mut previous = None;
        let mut successors = 0;
        for row in rows.chunks_exact(S2_ROW_BYTES) {
            let (key, token, rank) = Self::fields(row);
            let order = (key >> 120) as u32;
            if order > 10 || key & (((1u128 << 120) - 1) >> (12 * order)) != 0 {
                return Err(invalid("invalid S2 prime-route key"));
            }
            if token == 0
                || usize::from(token) >= vocabulary
                || !(1..=31).contains(&rank)
                || row[19] != 0
            {
                return Err(invalid("invalid S2 token, rank or padding"));
            }
            if previous.is_some_and(|prior| prior >= (key, token)) {
                return Err(invalid("unordered or duplicate S2 record"));
            }
            successors = if previous.is_some_and(|(prior, _)| prior == key) {
                successors + 1
            } else {
                1
            };
            if successors > 4 {
                return Err(invalid("S2 context exceeds four retained successors"));
            }
            previous = Some((key, token));
        }
        Ok(Self {
            bytes: bytes.into(),
        })
    }

    fn fields(row: &[u8]) -> (u128, u16, u8) {
        let mut key = [0; 16];
        key.copy_from_slice(&row[..16]);
        (
            u128::from_le_bytes(key),
            u16::from_le_bytes([row[16], row[17]]),
            row[18],
        )
    }

    /// Three concatenated SHA-256 digests: original trunk, full TRAIN file, sampled token IDs.
    pub fn source_digests(&self) -> &[u8] {
        &self.bytes[..S2_HEADER_BYTES]
    }

    /// Canonical (ordered key, token, declared count) stream.
    pub fn triples(&self) -> impl ExactSizeIterator<Item = (u128, u16, u32)> + '_ {
        self.bytes[S2_HEADER_BYTES..]
            .chunks_exact(S2_ROW_BYTES)
            .map(|row| {
                let (key, token, rank) = Self::fields(row);
                (key, token, 1u32 << rank)
            })
    }

    /// Read a declared count using the exact packed PrimeRouteKey and original token ID.
    pub fn count(&self, key: u128, token: u16) -> u32 {
        let rows = &self.bytes[S2_HEADER_BYTES..];
        let mut low = 0;
        let mut high = rows.len() / S2_ROW_BYTES;
        while low < high {
            let mid = low + (high - low) / 2;
            let (found_key, found_token, rank) = Self::fields(&rows[mid * S2_ROW_BYTES..]);
            match (found_key, found_token).cmp(&(key, token)) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return 1u32 << rank,
            }
        }
        0
    }
}

/// Optional 15-table post-rotation joint-pointer trunk (`uor-r4.lut-discrete-trunk/1`).
#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscreteTrunkTables {
    s2: Option<FrozenS2Table>,
    active_s2: bool,
    step_candidates: [u8; 64],
    norm_candidates: [u8; 4],
    unigram_best_global: u16,
    unigram_counts: Vec<u32>,
    read_table: DiscreteSparseCountTable,
    rec_table: DiscreteSparseCountTable,
    read_rec_table: DiscreteSparseCountTable,
    rec_shuf_table: DiscreteSparseCountTable,
    read_rec_shuf_table: DiscreteSparseCountTable,
    rec_unit_table: DiscreteSparseCountTable,
    read_rec_unit_table: DiscreteSparseCountTable,
    query_table: DiscreteSparseCountTable,
    query_read_table: DiscreteSparseCountTable,
    query_rec_table: DiscreteSparseCountTable,
    query_rec_shuf_table: DiscreteSparseCountTable,
    query_rec_unit_table: DiscreteSparseCountTable,
}

impl DiscreteTrunkTables {
    fn level3_joint_pointer_score(
        &self,
        query_token: u16,
        read_code: u16,
        rec_code: u16,
        pointer_succ: Option<u16>,
        tok: u16,
    ) -> u64 {
        let q_key = u32::from(query_token);
        let r_key = u32::from(read_code);
        let qr_key = (u32::from(query_token) << 16) | u32::from(read_code);
        let qc_key = (u32::from(query_token) << 16) | u32::from(rec_code);
        let rc_key = (u32::from(read_code) << 16) | u32::from(rec_code);
        let c_q = u64::from(self.query_table.get(q_key, tok));
        let c_qr = u64::from(self.query_read_table.get(qr_key, tok));
        let c_r = u64::from(self.read_table.get(r_key, tok));
        let c_qc = u64::from(self.query_rec_table.get(qc_key, tok));
        let c_rr = u64::from(self.read_rec_table.get(rc_key, tok));
        let ptr_boost = if pointer_succ == Some(tok) {
            1u64 << 8
        } else {
            0u64
        };
        (c_qr << 6) + (c_qc << 5) + (c_rr << 3) + (c_r << 1) + c_q + ptr_boost
    }

    fn select_level3_joint_pointer_global(
        &self,
        query_token: u16,
        read_code: u16,
        rec_code: u16,
        pointer_succ: Option<u16>,
    ) -> u16 {
        let q_key = u32::from(query_token);
        let r_key = u32::from(read_code);
        let c_key = u32::from(rec_code);
        let vocab = self.unigram_counts.len();
        let mut best_tok = self.unigram_best_global;
        let mut best_s = 0u64;
        let mut best_u = 0u32;

        let mut consider = |tok: u16| {
            if usize::from(tok) >= vocab {
                return;
            }
            let s = self.level3_joint_pointer_score(
                query_token,
                read_code,
                rec_code,
                pointer_succ,
                tok,
            );
            let u = self.unigram_counts[usize::from(tok)];
            if s > best_s
                || (s == best_s && s > 0 && u > best_u)
                || (s == best_s && s > 0 && u == best_u && tok < best_tok)
            {
                best_s = s;
                best_u = u;
                best_tok = tok;
            }
        };

        for map_opt in [
            self.query_table.entries(q_key),
            self.read_table.entries(r_key),
            self.rec_table.entries(c_key),
        ]
        .into_iter()
        .flatten()
        {
            for &tok in map_opt.keys() {
                consider(tok);
            }
        }
        if let Some(ptr_tok) = pointer_succ {
            consider(ptr_tok);
        }
        best_tok
    }

    fn compute_rec_code(&self, prefix: &[u32], codebook_roots: &[[u8; 4]]) -> Result<u16> {
        let len = prefix.len();
        let last_pos = len - 1;
        let prev_pos = last_pos.saturating_sub(1);
        let mid_pos = last_pos / 2;
        let mut out_roots = [0u8; 4];
        let mut prefix_acc = [0usize; MAX_CONTEXT];
        for b in 0..4 {
            let mut acc = usize::from(IDENTITY);
            for (pos, &tok) in prefix.iter().enumerate() {
                let r = usize::from(codebook_roots[tok as usize][b]);
                let step = usize::from(self.step_candidates[(b << 4) + r]);
                acc = compose(step, acc);
                prefix_acc[pos] = acc;
            }
            let f_last = prefix_acc[last_pos];
            let h_prev = compose(f_last, cl06_inverse(prefix_acc[prev_pos]));
            let h_mid = compose(f_last, cl06_inverse(prefix_acc[mid_pos]));

            let r_start = usize::from(codebook_roots[prefix[0] as usize][b]);
            let r_last = usize::from(codebook_roots[prefix[last_pos] as usize][b]);
            let r_prev = usize::from(codebook_roots[prefix[prev_pos] as usize][b]);
            let r_mid = usize::from(codebook_roots[prefix[mid_pos] as usize][b]);

            let q_coords = orbit_root_coords(r_last);
            let prev_rot = action(h_prev, &orbit_root_coords(r_prev));
            let mid_rot = action(h_mid, &orbit_root_coords(r_mid));
            let full_rot = action(f_last, &orbit_root_coords(r_start));

            let mut sum_block = [0i32; 8];
            for d in 0..8 {
                sum_block[d] = q_coords[d] + prev_rot[d] + mid_rot[d] + full_rot[d];
            }
            let snapped_root = snap(&sum_block, 4);
            let snapped_coords = orbit_root_coords(snapped_root);
            let align_frame = usize::from(self.norm_candidates[b]);
            let normalized_block = action(align_frame, &snapped_coords);
            let norm_root = root_id(&normalized_block)
                .ok_or_else(|| invalid("normalized recurrence block outside 16-root orbit"))?
                as u8;
            out_roots[b] = norm_root;
        }
        Ok(pack_4block_roots(out_roots))
    }

    fn apply_joint_pointer_gate(
        &self,
        prefix: &[u32],
        q_blocks: &mut [[i32; 8]],
        codebook_roots: &[[u8; 4]],
    ) -> Result<()> {
        if q_blocks.len() != 4 {
            return Err(invalid("joint-pointer trunk gate requires 4 blocks"));
        }
        let mut read_roots = [0u8; 4];
        for b in 0..4 {
            read_roots[b] = root_id(&q_blocks[b])
                .ok_or_else(|| invalid("read block outside 16-root orbit"))?
                as u8;
        }
        let read_code = pack_4block_roots(read_roots);
        let rec_code = self.compute_rec_code(prefix, codebook_roots)?;
        let pointer_succ = orbit_root_causal_successor(prefix, codebook_roots)?;
        let query_token = u16::try_from(prefix[prefix.len() - 1])
            .map_err(|_| invalid("query token overflows u16"))?;
        let chosen_tok =
            self.select_level3_joint_pointer_global(query_token, read_code, rec_code, pointer_succ);
        let target_roots = codebook_roots
            .get(usize::from(chosen_tok))
            .copied()
            .ok_or_else(|| invalid("chosen_tok out of codebook_roots bounds"))?;
        for b in 0..4 {
            let src = usize::from(read_roots[b]);
            let dst = usize::from(target_roots[b]);
            let frame = usize::from(BFS[(src << 4) + dst]);
            let rotated = action(frame, &q_blocks[b]);
            let snapped_root = snap(&rotated, 1);
            q_blocks[b] = orbit_root_coords(snapped_root);
        }
        Ok(())
    }
}

/// Loaded discrete SpiralCore/FBS model served from a `UORLUT01` (`uor-r4.lut-discrete/1`
/// or 19-table `uor-r4.lut-discrete-trunk/1`) integer container.
#[derive(Clone, Debug)]
pub struct IntegerDiscreteModel {
    count_mixer: Option<crate::count_mixer::CountMixer>,
    role_policy: Option<RolePolicy>,
    contextual_source: Option<contextual_source::SourcePolicy>,
    assertion_arm: Option<crate::relation_retrieval::Arm>,
    tokenizer_sha256: Option<String>,
    shape: DiscreteLutShape,
    source: DiscreteLutSource,
    read_mode: DiscreteReadMode,
    objective_policy: DiscreteObjectivePolicy,
    artifact_sha256: String,
    /// `[token][block]` 8D orbit-root coordinates.
    token_blocks: Vec<Vec<[i32; 8]>>,
    /// Precomputed 4-block orbit root indices `[token][0..4]` when `blocks == 4`.
    codebook_roots: Vec<[u8; 4]>,
    /// Pre-verified writer cell `0..256` per token.
    cells: Vec<u32>,
    /// `[block][stage]` route action index `0..64`.
    routes: Vec<Vec<u32>>,
    /// `[block][stage]` level gate trits in `-1..=1`.
    level_gates: Vec<Vec<i32>>,
    /// `[stage][edge_type]` relation-edge gate trits in `-1..=1` (empty unless `Relation`).
    relation_gates: Vec<[i32; EDGE_TYPES]>,
    /// Optional 15-table post-rotation joint-pointer trunk (`uor-r4.lut-discrete-trunk/1`).
    trunk: Option<DiscreteTrunkTables>,
    /// When `true`, projects intermediate tree/frame state transport to the identity frame
    /// (`A_0 = I_8`) on the local query token block and zeroes all level and relation gates.
    state_transport_knockout: bool,
    /// Position transport of the Pick read (`None` unless `Pick`).
    pick_transport: Option<PickTransport>,
}

fn expected_stages(context: u32) -> Option<u32> {
    if !(1..=MAX_CONTEXT as u32).contains(&context) {
        return None;
    }
    let mut stages = 1u32;
    let mut span = 3u32;
    while span <= context {
        stages += 1;
        span += span + span;
    }
    Some(stages)
}

fn root_id(block: &[i32; 8]) -> Option<usize> {
    (0..16).find(|&r| (0..8).all(|d| block[d] == i32::from(ROOTS[r][d])))
}

fn action(a: usize, x: &[i32; 8]) -> [i32; 8] {
    let mut y = [0i32; 8];
    let base = a << 3;
    for d in 0..8 {
        let axis = usize::from(AXES[base + d]);
        let v = x[axis & 7];
        y[d] = if (axis & 8) != 0 { -v } else { v };
    }
    y
}

fn compose(a: usize, b: usize) -> usize {
    usize::from(COMPOSE[(a << 6) + b])
}

fn root_cost(x: &[i32; 8], root: usize) -> (u32, u32) {
    let mut distance = 0u32;
    let mut hamming = 0u32;
    for d in 0..8 {
        let y = i32::from(ROOTS[root][d]);
        distance += (x[d] - y).unsigned_abs();
        hamming += u32::from(x[d] != y);
    }
    (distance, hamming)
}

fn snap(x: &[i32; 8], n: usize) -> usize {
    let mut best = 0usize;
    let mut best_distance = u32::MAX;
    let mut best_hamming = u32::MAX;
    for r in 0..16 {
        let mut distance = 0u32;
        let mut hamming = 0u32;
        for d in 0..8 {
            let mut y = 0i32;
            let v = i32::from(ROOTS[r][d]);
            for _ in 0..n {
                y += v;
            }
            distance += (x[d] - y).unsigned_abs();
            hamming += u32::from(x[d] != y);
        }
        if distance < best_distance || (distance == best_distance && hamming < best_hamming) {
            best = r;
            best_distance = distance;
            best_hamming = hamming;
        }
    }
    best
}

fn is_zero_word(word: &u32) -> bool {
    *word == 0
}

/// Position transport admitted by the Pick read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PickTransport {
    /// `s[14] = 1`: `F_{p,b} = COMPOSE[step(b, root(p, b)), F_{p-1,b}]`,
    /// transport `H_{t <- p, b} = COMPOSE[F_{t,b}, INVERSE[F_{p,b}]]`.
    /// `frames[b * 16 + root]` is the step frame index `0..64`.
    PrefixHolonomy { frames: Vec<u8> },
    /// `s[14] = 3`: identity at every position.
    Identity,
}

/// One row of the Pick read: the output blocks, the picked position and its distance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PickRead {
    pub(crate) blocks: Vec<[i32; 8]>,
    pub(crate) pick: Option<usize>,
    pub(crate) distance: Option<u32>,
    pub(crate) ties: u32,
}

/// Exact read observations from the explicit identity-transport diagnostic.
#[doc(hidden)]
pub struct IdentityPickDiagnostic {
    pub vector: Vec<i32>,
    pub block_words: Vec<[u32; 5]>,
    pub successor: Option<u32>,
}

/// 16-bit code of an anchor-orbit block: bit `2d` is `x[d] > 0`, bit `2d + 1` is `x[d] < 0`.
/// For two orbit roots, `L1(a - b) == 2 * popcount(code(a) ^ code(b))`.
pub(crate) fn root_code(x: &[i32; 8]) -> u16 {
    let mut code = 0u16;
    for (d, &v) in x.iter().enumerate() {
        if v > 0 {
            code |= 1 << (2 * d);
        } else if v < 0 {
            code |= 1 << (2 * d + 1);
        }
    }
    code
}

/// The Pick read for one causal prefix (`t = prefix.len() - 1`). Every block of
/// `tok(p)` is transported from `p` to `t`; the pick is the latest `p` in
/// `1..t` minimizing `sum_b popcount(code(p-1,b) ^ code(t-1,b)) +
/// popcount(code(p,b) ^ code(t,b))`; block `b` reads
/// `snap_{1+|g_b|}(query_b + g_b * successor_b)` with the level-0 gate `g_b`.
pub(crate) fn pick_read(
    token_blocks: &[Vec<[i32; 8]>],
    gates0: &[i32],
    transport: &PickTransport,
    prefix: &[u32],
) -> Result<PickRead> {
    if prefix.is_empty() {
        return Err(invalid("Pick read needs a non-empty prefix"));
    }
    let t = prefix.len() - 1;
    let query = &token_blocks[prefix[t] as usize];
    let blocks = query.len();
    if gates0.len() != blocks {
        return Err(invalid("Pick read gate count differs from block count"));
    }
    if t < 2 {
        return Ok(PickRead {
            blocks: query.clone(),
            pick: None,
            distance: None,
            ties: 0,
        });
    }
    // transported[p][b]: block b of tok(p) carried from p to t.
    let mut transported = vec![vec![[0i32; 8]; blocks]; t + 1];
    for b in 0..blocks {
        match transport {
            PickTransport::Identity => {
                for (p, slot) in transported.iter_mut().enumerate() {
                    slot[b] = token_blocks[prefix[p] as usize][b];
                }
            }
            PickTransport::PrefixHolonomy { frames } => {
                let mut prefix_frames = Vec::with_capacity(t + 1);
                let mut acc = usize::from(IDENTITY);
                for &tok in prefix {
                    let root = root_id(&token_blocks[tok as usize][b])
                        .ok_or_else(|| invalid("Pick block outside SpiralCore orbit"))?;
                    acc = compose(usize::from(frames[b * 16 + root]), acc);
                    prefix_frames.push(acc);
                }
                for (p, slot) in transported.iter_mut().enumerate() {
                    let h = compose(prefix_frames[t], cl06_inverse(prefix_frames[p]));
                    slot[b] = action(h, &token_blocks[prefix[p] as usize][b]);
                }
            }
        }
    }
    let codes: Vec<Vec<u16>> = transported
        .iter()
        .map(|row| row.iter().map(root_code).collect())
        .collect();
    let mut pick = 1usize;
    let mut best = u32::MAX;
    let mut ties = 0;
    for p in 1..t {
        let mut d = 0u32;
        for b in 0..blocks {
            d += (codes[p - 1][b] ^ codes[t - 1][b]).count_ones();
            d += (codes[p][b] ^ codes[t][b]).count_ones();
        }
        if d < best {
            ties = 0;
        }
        if d <= best {
            pick = p;
            best = d;
            ties += 1;
        }
    }
    let mut out = Vec::with_capacity(blocks);
    for b in 0..blocks {
        let gate = gates0[b];
        let mut u = query[b];
        let payload = &transported[pick + 1][b];
        for d in 0..8 {
            u[d] += if gate > 0 {
                payload[d]
            } else if gate < 0 {
                -payload[d]
            } else {
                0
            };
        }
        let n = 1 + gate.unsigned_abs() as usize;
        out.push(ROOTS[snap(&u, n)].map(i32::from));
    }
    Ok(PickRead {
        blocks: out,
        pick: Some(pick),
        distance: Some(best),
        ties,
    })
}

fn checked_scaled_add(mut value: u64, mut times: u32) -> Option<u64> {
    let mut total = 0u64;
    while times != 0 {
        if (times & 1) != 0 {
            total = total.checked_add(value)?;
        }
        times >>= 1;
        if times != 0 {
            if value > (u64::MAX >> 1) {
                return None;
            }
            value <<= 1;
        }
    }
    Some(total)
}

#[derive(Clone, Debug)]
struct BinderNode {
    children: Option<[usize; 3]>,
    atom: Option<usize>,
    level: usize,
    first: usize,
    last: usize,
}

#[derive(Clone, Debug)]
struct RowPlan {
    nodes: Vec<BinderNode>,
    frontier: Vec<usize>,
}
type FlockReadParts = (Vec<[i32; 8]>, Option<Vec<i32>>, Option<Vec<u32>>);

fn build_row_plan(length: usize) -> Result<RowPlan> {
    if !(1..=MAX_CONTEXT).contains(&length) {
        return Err(invalid("prefix length must be in 1..=context"));
    }
    let mut nodes: Vec<BinderNode> = Vec::new();
    let mut frontier: Vec<usize> = Vec::new();
    for atom in 0..length {
        frontier.push(nodes.len());
        nodes.push(BinderNode {
            children: None,
            atom: Some(atom),
            level: 0,
            first: atom,
            last: atom,
        });
        loop {
            let n = frontier.len();
            if n < 3 {
                break;
            }
            let c0 = frontier[n - 3];
            let c1 = frontier[n - 2];
            let c2 = frontier[n - 1];
            let lv = nodes[c0].level;
            if nodes[c1].level != lv || nodes[c2].level != lv {
                break;
            }
            let parent = nodes.len();
            let first = nodes[c0].first;
            let last = nodes[c2].last;
            nodes.push(BinderNode {
                children: Some([c0, c1, c2]),
                atom: None,
                level: lv + 1,
                first,
                last,
            });
            frontier.truncate(n - 3);
            frontier.push(parent);
        }
    }
    Ok(RowPlan { nodes, frontier })
}

pub(crate) fn orbit_root_coords(root: usize) -> [i32; 8] {
    let r = &ROOTS[root];
    [
        i32::from(r[0]),
        i32::from(r[1]),
        i32::from(r[2]),
        i32::from(r[3]),
        i32::from(r[4]),
        i32::from(r[5]),
        i32::from(r[6]),
        i32::from(r[7]),
    ]
}

pub(crate) fn orbit_root_id(block: &[i32; 8]) -> Option<usize> {
    root_id(block)
}

pub(crate) fn orbit_snap(block: &[i32; 8], n: usize) -> usize {
    snap(block, n)
}

pub(crate) fn cl06_action(a: usize, x: &[i32; 8]) -> [i32; 8] {
    action(a, x)
}

/// The 64-state Cl(0,6) signed-axis table, state `a` in row `a*8..a*8 + 8`:
/// `code & 7` is the source axis, `code & 8` negates. A converter checks a
/// float geometry against it (d11-exporter-float-trunk).
pub fn spiralcore_signed_axes() -> &'static [u8; 512] {
    &AXES
}

/// The 16 orbit roots of the Cl(0,6) anchor `[2, 2, 0, 0, 0, 0, 0, 0]`, anchor
/// first. The native vector writer's cell `a*16 + b` pairs roots `a` and `b`.
pub fn spiralcore_orbit_roots() -> &'static [[i8; 8]; 16] {
    &ROOTS
}

/// The eight signed-axis codes of Cl(0,6) state `a`: `code & 7` is the source
/// axis, `code & 8` negates.
#[allow(dead_code)]
pub(crate) fn cl06_axes(a: usize) -> [u8; 8] {
    let mut codes = [0u8; 8];
    codes.copy_from_slice(&AXES[a * 8..a * 8 + 8]);
    codes
}

pub(crate) fn cl06_compose(a: usize, b: usize) -> usize {
    compose(a, b)
}

pub(crate) fn cl06_bfs(src: usize, dst: usize) -> usize {
    usize::from(BFS[(src << 4) + dst])
}

pub(crate) fn cl06_inverse(a: usize) -> usize {
    compose(a, compose(a, a))
}

pub(crate) fn fbs_binder_summary(length: usize) -> Result<(usize, usize, u32)> {
    let plan = build_row_plan(length)?;
    let stages =
        expected_stages(length as u32).ok_or_else(|| invalid("invalid FBS binder length"))?;
    Ok((plan.nodes.len(), plan.frontier.len(), stages))
}

fn read_role_digest(text: &str) -> Result<[u8; 32]> {
    let mut digest = [0; 32];
    hex::decode_to_slice(text, &mut digest).map_err(|_| invalid("invalid ordered-role SHA-256"))?;
    if text != hex::encode(digest) {
        return Err(invalid("noncanonical ordered-role SHA-256"));
    }
    Ok(digest)
}

fn role_bindings(
    base: &str,
    tokenizer: &str,
    classes: [u8; 32],
    framing: &RoleFraming,
    train: [u8; 32],
) -> Result<RoleBindings> {
    Ok([
        read_role_digest(base)?,
        read_role_digest(tokenizer)?,
        classes,
        orbit_action_digest(),
        train,
        framing.operator_digest(),
    ])
}

/// Pack an explicitly bound onset bank around an exact frozen paired-copy/1 carrier.
/// All source bytes needed to verify the frozen site profile survive cold reload.
pub fn pack_all_starts_artifact(
    frozen_paired: &[u8],
    span_state: &[u8],
    bank: &[u8],
    profile: &crate::all_starts::OnsetProfile,
) -> Result<Vec<u8>> {
    let parent = read_onset_parent(frozen_paired)?;
    pack_all_starts(&parent, frozen_paired, span_state, bank, profile, None)
}

/// Package assertion-agreement role selection over the original ordered value bank.
#[cfg(feature = "serving-wasm")]
pub fn pack_assertion_agreement_artifact(
    frozen_paired: &[u8],
    span_state: &[u8],
    bank: &[u8],
    profile: &crate::all_starts::OnsetProfile,
    tokenizer: &[u8],
) -> Result<Vec<u8>> {
    pack_requested_occurrence_artifact(
        frozen_paired,
        span_state,
        bank,
        profile,
        tokenizer,
        RequestedContext::FullPrefix,
    )
}

/// Package an explicitly scoped assertion-agreement ranker with the frozen ordered value bank.
#[cfg(feature = "serving-wasm")]
pub fn pack_requested_occurrence_artifact(
    frozen_paired: &[u8],
    span_state: &[u8],
    bank: &[u8],
    profile: &crate::all_starts::OnsetProfile,
    tokenizer: &[u8],
    context: RequestedContext,
) -> Result<Vec<u8>> {
    let parent = read_onset_parent(frozen_paired)?;
    if parent.tokenizer_sha256() != Some(hex::encode(Sha256::digest(tokenizer)).as_str()) {
        return Err(invalid(
            "requested occurrence pack requires the original tokenizer",
        ));
    }
    let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer)
        .ok_or_else(|| invalid("invalid requested occurrence tokenizer"))?;
    if tokenizer.vocab_size() != ROLE_VOCAB {
        return Err(invalid(
            "requested occurrence tokenizer vocabulary mismatch",
        ));
    }
    let framing = RoleFraming::from_tokenizer(&tokenizer)?;
    let requested_policy = match context {
        RequestedContext::FullPrefix => RequestedPolicy::AssertionAgreement(framing),
        RequestedContext::UserTextOnly => RequestedPolicy::UserTextOnly(framing),
        RequestedContext::ClauseToEntity(separators) => {
            if separators.map(|id| tokenizer.decode_bytes(&[id])) != [b",".to_vec(), b";".to_vec()]
            {
                return Err(invalid(
                    "clause-to-entity separators must be comma and semicolon",
                ));
            }
            RequestedPolicy::ClauseToEntity {
                framing,
                separators,
            }
        }
    };
    pack_all_starts(
        &parent,
        frozen_paired,
        span_state,
        bank,
        profile,
        Some(requested_policy),
    )
}

fn pack_all_starts(
    parent: &IntegerDiscreteModel,
    frozen_paired: &[u8],
    span_state: &[u8],
    bank: &[u8],
    profile: &crate::all_starts::OnsetProfile,
    requested_policy: Option<RequestedPolicy>,
) -> Result<Vec<u8>> {
    validate_onset_bank(parent, span_state, bank, profile, requested_policy.clone())?;
    let header = serde_json::to_vec(&OnsetEnvelope {
        schema: onset_schema(requested_policy.as_ref()).to_owned(),
        parent_sha256: parent.artifact_sha256.clone(),
        span_sha256: hex::encode(Sha256::digest(span_state)),
        bank_sha256: hex::encode(Sha256::digest(bank)),
        span_bytes: span_state.len(),
        profile: profile.clone(),
        requested_policy,
    })?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend(header);
    bytes.resize(bytes.len().next_multiple_of(ALIGN), 0);
    bytes.extend_from_slice(bank);
    bytes.extend_from_slice(span_state);
    bytes.extend_from_slice(frozen_paired);
    Ok(bytes)
}

fn read_onset_parent(bytes: &[u8]) -> Result<IntegerDiscreteModel> {
    let prefix = bytes
        .get(..16)
        .ok_or_else(|| invalid("missing frozen onset carrier"))?;
    if &prefix[..8] != MAGIC {
        return Err(invalid("missing frozen onset carrier magic"));
    }
    let length = u64::from_le_bytes(
        <[u8; 8]>::try_from(&prefix[8..])
            .map_err(|_| invalid("invalid frozen onset carrier header"))?,
    );
    if length > MAX_HEADER {
        return Err(invalid("frozen onset carrier header too large"));
    }
    #[derive(Deserialize)]
    struct Schema {
        schema: String,
    }
    let header = bytes
        .get(16..16 + length as usize)
        .ok_or_else(|| invalid("truncated frozen onset carrier header"))?;
    if serde_json::from_slice::<Schema>(header)?.schema != DISCRETE_PAIRED_SCHEMA {
        return Err(invalid(
            "onset carrier requires its frozen paired-copy/1 parent",
        ));
    }
    IntegerDiscreteModel::read_container(bytes, true)
}

fn validate_onset_bank(
    parent: &IntegerDiscreteModel,
    span_state: &[u8],
    raw: &[u8],
    profile: &crate::all_starts::OnsetProfile,
    requested_policy: Option<RequestedPolicy>,
) -> Result<OnsetPolicy> {
    if let Some(policy) = &requested_policy {
        if !matches!(
            profile.mode,
            crate::all_starts::OnsetMode::Ordered | crate::all_starts::OnsetMode::OrderedWholeWord
        ) {
            return Err(invalid(
                "assertion agreement requires the original ordered value policy",
            ));
        }
        policy.framing().validate()?;
    }
    if span_state.len() > MAX_ONSET_SPAN_BYTES {
        return Err(invalid("frozen onset span state too large"));
    }
    let span: OnsetSpanState = serde_json::from_slice(span_state)?;
    let policy = parent
        .role_policy
        .as_ref()
        .ok_or_else(|| invalid("missing onset parent policy"))?;
    let Some(paired) = &policy.paired else {
        return Err(invalid("missing onset parent paired policy"));
    };
    if let Some(RequestedPolicy::ClauseToEntity { separators, .. }) = &requested_policy {
        if separators[0] == separators[1]
            || separators
                .iter()
                .any(|id| !(2..ROLE_VOCAB as u32).contains(id) || paired.delimiters.contains(id))
        {
            return Err(invalid("invalid clause-to-entity separators"));
        }
    }
    let paired_copy::Roles::Frozen(roles) = &paired.roles else {
        return Err(invalid("onset parent must use frozen roles"));
    };
    let bindings = roles.bindings();
    if read_role_digest(&span.artifact_sha256)? != bindings[0]
        || read_role_digest(&span.tokenizer_sha256)? != bindings[1]
        || span.bank.classes != policy.span.classes
        || span.bank.relations != policy.span.relations
    {
        return Err(invalid("onset span state differs from frozen parent"));
    }
    let packed_roots = parent.packed_codebook_roots();
    let mut roots = Sha256::new();
    for &word in &packed_roots {
        roots.update((word as u16).to_le_bytes());
    }
    let mut classes = Sha256::new();
    for &class in &policy.span.classes {
        classes.update([class as u8]);
    }
    let mut sites = Sha256::new();
    sites.update(b"uor-r4/all-starts-sites/1");
    sites.update(read_role_digest(&parent.artifact_sha256)?);
    sites.update(roots.finalize());
    sites.update(classes.finalize());
    sites.update(Sha256::digest(span_state));
    for root in spiralcore_orbit_roots() {
        sites.update(root.map(|coordinate| coordinate as u8));
    }
    let bank = crate::all_starts::Bank::from_profile(
        raw,
        bindings[0],
        bindings[1],
        sites.finalize().into(),
        profile,
    )?;
    Ok(OnsetPolicy {
        bank,
        packed_roots,
        requested_policy,
    })
}

/// Pack a checked bank into a versioned D11 paired-copy container. Frozen
/// writer policy, fact delimiters, classes and the embedded trunk are reused.
#[cfg(feature = "serving-wasm")]
pub fn pack_ordered_role_artifact(
    legacy_paired: &[u8],
    tokenizer: &[u8],
    bank: &RoleBank,
    train_digest: [u8; 32],
) -> Result<Vec<u8>> {
    let parent = IntegerDiscreteModel::from_bytes(legacy_paired)?;
    if parent.artifact_schema() != DISCRETE_PAIRED_SCHEMA
        || parent.tokenizer_sha256() != Some(hex::encode(Sha256::digest(tokenizer)).as_str())
    {
        return Err(invalid(
            "ordered-role pack requires its frozen paired carrier and original tokenizer",
        ));
    }
    let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer)
        .ok_or_else(|| invalid("invalid ordered-role tokenizer JSON"))?;
    if tokenizer.vocab_size() != ROLE_VOCAB {
        return Err(invalid("ordered-role tokenizer vocabulary mismatch"));
    }
    let framing = RoleFraming::from_tokenizer(&tokenizer)?;
    bank.verify_bindings(&parent.ordered_role_bindings(&framing, train_digest)?)?;
    let classes: Vec<_> = parent.ordered_role_sites()?.word_classes().collect();
    let header_len = u64::from_le_bytes(
        legacy_paired[8..16]
            .try_into()
            .map_err(|_| invalid("legacy paired header length mismatch"))?,
    ) as usize;
    let end = 16 + header_len;
    let mut envelope: RoleEnvelope = serde_json::from_slice(&legacy_paired[16..end])?;
    let header = envelope
        .paired
        .as_mut()
        .ok_or_else(|| invalid("missing frozen paired header"))?;
    let base = &legacy_paired[end.next_multiple_of(ALIGN) + header.bank_bytes..];
    let raw = bank.to_bytes();
    envelope.schema = DISCRETE_ORDERED_ROLE_SCHEMA.to_owned();
    header.bank_bytes = ROLE_RECORD_BYTES;
    header.bank_sha256 = hex::encode(Sha256::digest(raw));
    header.ordered = Some(paired_copy::OrderedHeader {
        framing,
        train_sha256: hex::encode(train_digest),
    });
    let encoded = serde_json::to_vec(&envelope)?;
    let mut file = MAGIC.to_vec();
    file.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    file.extend_from_slice(&encoded);
    file.resize(file.len().next_multiple_of(ALIGN), 0);
    file.extend_from_slice(&raw);
    file.extend_from_slice(&classes);
    file.extend_from_slice(base);
    let reload = IntegerDiscreteModel::from_bytes(&file)?;
    if reload.ordered_role_bank().map(RoleBank::to_bytes) != Some(raw) {
        return Err(invalid("ordered-role artifact reload mismatch"));
    }
    Ok(file)
}

impl IntegerDiscreteModel {
    /// Load and validate a `UORLUT01` (`uor-r4.lut-discrete/1` or `uor-r4.lut-discrete-trunk/1`) file from disk.
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    /// Parse and validate a `UORLUT01` (`uor-r4.lut-discrete/1` or `uor-r4.lut-discrete-trunk/1`) byte buffer.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::read_container(bytes, true)
    }

    fn read_container(bytes: &[u8], allow_role: bool) -> Result<Self> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err(invalid("missing UORLUT01 magic"));
        }
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&bytes[8..16]);
        let header_len = u64::from_le_bytes(len_bytes);
        if header_len > MAX_HEADER || 16 + (header_len as usize) > bytes.len() {
            return Err(invalid("UORLUT01 header length out of bounds"));
        }
        let end_header = 16 + header_len as usize;
        let encoded_header = &bytes[16..end_header];
        #[derive(Deserialize)]
        struct Schema {
            schema: String,
        }
        let schema = serde_json::from_slice::<Schema>(encoded_header)?.schema;
        if schema == rival_admission::SCHEMA {
            return rival_admission::read(bytes, encoded_header, end_header, allow_role);
        }
        if schema == assertion_retrieval::SCHEMA {
            return assertion_retrieval::read(bytes, encoded_header, end_header, allow_role);
        }
        if matches!(
            schema.as_str(),
            contextual_source::SCHEMA | contextual_source::RECURRENCE_SCHEMA
        ) {
            return contextual_source::read_carrier(bytes, encoded_header, end_header, allow_role);
        }
        if matches!(
            schema.as_str(),
            DISCRETE_ALL_STARTS_SCHEMA
                | DISCRETE_REQUESTED_OCCURRENCE_SCHEMA
                | DISCRETE_USER_TEXT_ONLY_SCHEMA
                | DISCRETE_CLAUSE_TO_ENTITY_SCHEMA
        ) {
            if !allow_role {
                return Err(invalid("nested onset artifact"));
            }
            let envelope: OnsetEnvelope = serde_json::from_slice(encoded_header)?;
            if schema != onset_schema(envelope.requested_policy.as_ref()) {
                return Err(invalid(
                    "requested policy and onset artifact schema disagree",
                ));
            }
            let start = end_header.next_multiple_of(ALIGN);
            let payload = bytes
                .get(start..)
                .ok_or_else(|| invalid("missing onset artifact payload"))?;
            if envelope.span_bytes > MAX_ONSET_SPAN_BYTES
                || bytes[end_header..start].iter().any(|&byte| byte != 0)
            {
                return Err(invalid("invalid onset span length or padding"));
            }
            let (raw, attachment) = payload
                .split_at_checked(crate::all_starts::BANK_BYTES)
                .ok_or_else(|| invalid("truncated onset bank"))?;
            let (span, paired) = attachment
                .split_at_checked(envelope.span_bytes)
                .ok_or_else(|| invalid("truncated onset span state"))?;
            if Sha256::digest(raw).as_slice() != read_role_digest(&envelope.bank_sha256)?
                || Sha256::digest(span).as_slice() != read_role_digest(&envelope.span_sha256)?
                || Sha256::digest(paired).as_slice() != read_role_digest(&envelope.parent_sha256)?
            {
                return Err(invalid("onset artifact attachment checksum mismatch"));
            }
            let mut parent = read_onset_parent(paired)?;
            let onset = validate_onset_bank(
                &parent,
                span,
                raw,
                &envelope.profile,
                envelope.requested_policy,
            )?;
            parent
                .role_policy
                .as_mut()
                .ok_or_else(|| invalid("missing onset role policy"))?
                .onset = Some(onset);
            parent.artifact_sha256 = hex::encode(Sha256::digest(bytes));
            return Ok(parent);
        }
        if schema == crate::count_mixer::SCHEMA || schema == crate::count_mixer::CONTEXT_SCHEMA {
            if !allow_role {
                return Err(invalid("nested count mixer"));
            }
            let env: crate::count_mixer::Envelope = serde_json::from_slice(encoded_header)?;
            let start = end_header.next_multiple_of(ALIGN);
            let data = bytes
                .get(start..)
                .ok_or_else(|| invalid("missing count mixer payload"))?;
            if bytes[end_header..start].iter().any(|&v| v != 0) || env.state_bytes > data.len() {
                return Err(invalid("invalid count mixer bounds or padding"));
            }
            let (state, base) = data.split_at(env.state_bytes);
            if hex::encode(Sha256::digest(state)) != env.state_sha256 {
                return Err(invalid("count mixer digest mismatch"));
            }
            let mut model = Self::read_container(base, false)?;
            if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA
                || model.artifact_sha256() != env.base_sha256
                || model.tokenizer_sha256() != Some(env.tokenizer_sha256.as_str())
                || model.shape.vocabulary != 4096
            {
                return Err(invalid("count mixer carrier identity mismatch"));
            }
            model.count_mixer = Some(if schema == crate::count_mixer::CONTEXT_SCHEMA {
                crate::count_mixer::CountMixer::from_context_words(state)?
            } else {
                crate::count_mixer::CountMixer::from_words(state)?
            });
            model.tokenizer_sha256 = Some(env.tokenizer_sha256);
            model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
            return Ok(model);
        }
        if matches!(
            schema.as_str(),
            DISCRETE_ROLE_SCHEMA | DISCRETE_PAIRED_SCHEMA | DISCRETE_ORDERED_ROLE_SCHEMA
        ) {
            if !allow_role {
                return Err(invalid("nested role-copy artifact"));
            }
            let mut envelope: RoleEnvelope = serde_json::from_slice(encoded_header)?;
            let start = end_header.next_multiple_of(ALIGN);
            let payload = bytes
                .get(start..)
                .ok_or_else(|| invalid("missing embedded role-copy trunk"))?;
            if (envelope.schema != DISCRETE_ROLE_SCHEMA) != envelope.paired.is_some()
                || bytes[end_header..start].iter().any(|&v| v != 0)
            {
                return Err(invalid("invalid role-copy envelope"));
            }
            envelope.policy.validate()?;
            let mut packed_ordered = None;
            let base = if let Some(header) = envelope.paired {
                if header.bank_bytes > payload.len()
                    || (schema == DISCRETE_ORDERED_ROLE_SCHEMA
                        && header.bank_bytes != ROLE_RECORD_BYTES)
                    || header.delimiters.is_empty()
                    || header.delimiters.len() > ROLE_VOCAB
                    || header
                        .delimiters
                        .iter()
                        .any(|&id| id as usize >= ROLE_VOCAB)
                    || header.delimiters.windows(2).any(|ids| ids[0] >= ids[1])
                {
                    return Err(invalid("invalid paired bank bounds or delimiter IDs"));
                }
                let (raw, attachment) = payload.split_at(header.bank_bytes);
                if hex::encode(Sha256::digest(raw)) != header.bank_sha256 {
                    return Err(invalid("paired role bank SHA-256 mismatch"));
                }
                match (envelope.schema.as_str(), header.ordered) {
                    (DISCRETE_PAIRED_SCHEMA, None) => {
                        let roles = crate::paired_role::PairedRoleState::from_bytes(raw)?;
                        let binding = roles.bindings();
                        if binding[0] != read_role_digest(&envelope.base_sha256)?
                            || binding[1] != read_role_digest(&envelope.tokenizer_sha256)?
                        {
                            return Err(invalid(
                                "paired role fit belongs to a different model or tokenizer",
                            ));
                        }
                        envelope.policy.paired = Some(paired_copy::Policy {
                            roles: paired_copy::Roles::Frozen(roles),
                            delimiters: header.delimiters,
                        });
                        attachment
                    }
                    (DISCRETE_ORDERED_ROLE_SCHEMA, Some(ordered)) => {
                        let bank = RoleBank::from_bytes(raw)?;
                        ordered.framing.validate()?;
                        let (classes, base) = attachment
                            .split_at_checked(ROLE_VOCAB)
                            .ok_or_else(|| invalid("missing ordered-role word classes"))?;
                        if classes
                            .iter()
                            .any(|&class| u32::from(class) > crate::span_selector::CONT)
                        {
                            return Err(invalid("invalid ordered-role word class"));
                        }
                        bank.verify_bindings(&role_bindings(
                            &envelope.base_sha256,
                            &envelope.tokenizer_sha256,
                            Sha256::digest(classes).into(),
                            &ordered.framing,
                            read_role_digest(&ordered.train_sha256)?,
                        )?)?;
                        packed_ordered =
                            Some((bank, ordered.framing, classes.to_vec(), header.delimiters));
                        base
                    }
                    _ => return Err(invalid("mismatched paired role schema and header")),
                }
            } else {
                payload
            };
            let mut model = Self::read_container(base, false)?;
            if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA
                || model.artifact_sha256() != envelope.base_sha256
                || model.tokenizer_sha256() != Some(&envelope.tokenizer_sha256)
                || model.shape.vocabulary != 4096
            {
                return Err(invalid("role-copy carrier or tokenizer identity mismatch"));
            }
            if let Some((bank, framing, classes, delimiters)) = packed_ordered {
                envelope.policy.paired = Some(paired_copy::Policy {
                    roles: paired_copy::Roles::Ordered {
                        bank,
                        framing,
                        classes,
                        packed_roots: model.packed_codebook_roots(),
                    },
                    delimiters,
                });
            }
            model.tokenizer_sha256 = Some(envelope.tokenizer_sha256);
            model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
            model.role_policy = Some(envelope.policy);
            return Ok(model);
        }
        let header: DiscreteLutHeader = serde_json::from_slice(encoded_header)?;
        let is_trunk = match header.schema.as_str() {
            DISCRETE_SCHEMA => false,
            DISCRETE_TRUNK_SCHEMA | DISCRETE_S2_SCHEMA | DISCRETE_S2_EXPERT_SCHEMA => true,
            _ => {
                return Err(invalid(
                    "discrete UORLUT01 artifact requires schema uor-r4.lut-discrete/1 or uor-r4.lut-discrete-trunk/1",
                ));
            }
        };
        if !header.matrices.is_empty() {
            return Err(invalid(
                "discrete UORLUT01 artifact requires empty matrices",
            ));
        }
        if header.source.snapshot_schema != SNAPSHOT_SCHEMA
            || header.source.operator_kappa != OPERATOR_KAPPA
            || header.source.composition_kappa != COMPOSITION_KAPPA
        {
            return Err(invalid(
                "discrete UORLUT01 snapshot_schema or geometry kappa mismatch",
            ));
        }
        let unaligned = 16 + (header_len as usize);
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        if data_start > bytes.len() || bytes[unaligned..data_start].iter().any(|&b| b != 0) {
            return Err(invalid("UORLUT01 header padding is missing or non-zero"));
        }
        let data = &bytes[data_start..];
        let s = &header.shape;
        if !(2..=8192).contains(&s.vocabulary)
            || !(2..=16).contains(&s.blocks)
            || !(1..=MAX_CONTEXT as u32).contains(&s.context)
            || !(1..=16).contains(&s.kept_cells)
            || s.kept_cells > s.vocabulary
            || expected_stages(s.context) != Some(s.stages)
        {
            return Err(invalid("inconsistent or unsupported discrete shape"));
        }
        let read_mode = match (s.read_mode.as_str(), s.read_kept, s.read_sweeps) {
            ("route", 0, 0) => DiscreteReadMode::Route,
            ("flock", kept, 0) if (1..=MAX_KEPT as u32).contains(&kept) => {
                DiscreteReadMode::Flock { kept }
            }
            ("relation", kept, sweeps)
                if (1..=MAX_KEPT as u32).contains(&kept) && (1..=MAX_SWEEPS).contains(&sweeps) =>
            {
                DiscreteReadMode::Relation { kept, sweeps }
            }
            ("pick", 0, 0) => DiscreteReadMode::Pick,
            _ => return Err(invalid("unsupported discrete read mode or parameters")),
        };
        match (read_mode, s.read_position) {
            (DiscreteReadMode::Pick, 1 | 3) | (DiscreteReadMode::Route, 0) => {}
            (DiscreteReadMode::Flock { .. } | DiscreteReadMode::Relation { .. }, 0) => {}
            (DiscreteReadMode::Pick, _) => {
                return Err(invalid(
                    "Pick read admits only position transport 1 (prefix holonomy) or 3 (identity)",
                ))
            }
            _ => {
                return Err(invalid(
                    "position transport is served only for the Pick read",
                ))
            }
        }
        let objective_policy = match s.objective_policy.as_str() {
            "nearest_address_margin_v1" => DiscreteObjectivePolicy::NearestAddressMarginV1,
            "occupied_cell_rank_hinge_v1" => DiscreteObjectivePolicy::OccupiedCellRankHingeV1,
            _ => return Err(invalid("unsupported discrete objective policy")),
        };
        let init_ok = match s.read_initialization.as_str() {
            "zero_gates_v1" => true,
            "active_leaf_v1" => !matches!(read_mode, DiscreteReadMode::Route),
            _ => false,
        };
        if !init_ok {
            return Err(invalid("unsupported discrete read initialization"));
        }

        // A Pick read with prefix holonomy carries one more table, its step
        // frames (`discrete.position_frames`); every other artifact is unchanged.
        let plain_tables = if matches!(read_mode, DiscreteReadMode::Pick) && s.read_position == 1 {
            5
        } else {
            4
        };
        let active_s2 = header.schema == DISCRETE_S2_EXPERT_SCHEMA;
        let has_s2 = active_s2 || header.schema == DISCRETE_S2_SCHEMA;
        if active_s2 && (!allow_role || s.vocabulary != 4096) {
            return Err(invalid(
                "active S2 expert requires a standalone 4096-token trunk",
            ));
        }
        let trunk_tables = if has_s2 { 20 } else { 19 };
        if (!is_trunk && header.tables.len() != plain_tables)
            || (is_trunk && (header.tables.len() != trunk_tables || s.blocks != 4))
        {
            return Err(invalid(
                "discrete UORLUT01 requires 4 tables (uor-r4.lut-discrete/1) or 19 tables with 4 blocks (uor-r4.lut-discrete-trunk/1)",
            ));
        }
        if is_trunk {
            let mut prev_end = 0usize;
            for entry in &header.tables {
                let start = usize::try_from(entry.span.offset)
                    .map_err(|_| invalid("table offset overflow"))?;
                let span_bytes = usize::try_from(entry.span.bytes)
                    .map_err(|_| invalid("table bytes overflow"))?;
                let end = start
                    .checked_add(span_bytes)
                    .ok_or_else(|| invalid("table end overflow"))?;
                if start % ALIGN != 0
                    || start < prev_end
                    || start - prev_end >= ALIGN
                    || end > data.len()
                    || data[prev_end..start].iter().any(|&b| b != 0)
                {
                    return Err(invalid(
                        "discrete UORLUT01 section ordering, alignment gap, or zero-padding invalid",
                    ));
                }
                prev_end = end;
            }
            if prev_end != data.len() {
                return Err(invalid("discrete UORLUT01 has unexpected trailing bytes"));
            }
        }

        let read_section =
            |name: &str, expected_kind: SectionKind, expected_len: usize| -> Result<&[u8]> {
                let entry = header
                    .tables
                    .iter()
                    .find(|t| t.name == name)
                    .ok_or_else(|| invalid(format!("missing table {name}")))?;
                if entry.kind != expected_kind || entry.len != expected_len {
                    return Err(invalid(format!("table {name} kind or length mismatch")));
                }
                let expected_bytes = expected_len
                    .checked_mul(expected_kind.width())
                    .ok_or_else(|| invalid("table byte size overflow"))?;
                if (entry.span.offset as usize) % ALIGN != 0
                    || entry.span.bytes != expected_bytes as u64
                {
                    return Err(invalid(format!(
                        "table {name} alignment or byte span mismatch"
                    )));
                }
                let start = usize::try_from(entry.span.offset)
                    .map_err(|_| invalid("table offset overflow"))?;
                let end = start
                    .checked_add(expected_bytes)
                    .ok_or_else(|| invalid("table end overflow"))?;
                data.get(start..end)
                    .ok_or_else(|| invalid(format!("table {name} span out of bounds")))
            };

        let vocab = s.vocabulary as usize;
        let blocks = s.blocks as usize;
        let stages = s.stages as usize;
        let base_gates = blocks * stages;
        let total_gates = if matches!(read_mode, DiscreteReadMode::Relation { .. }) {
            base_gates + EDGE_TYPES * stages
        } else {
            base_gates
        };

        let raw_table: Vec<i32> =
            read_section("discrete.table", SectionKind::I32, vocab * blocks * 8)?
                .chunks_exact(4)
                .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
        let cells: Vec<u32> = read_section("discrete.cells", SectionKind::U32, vocab)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let raw_routes: Vec<u32> = read_section("discrete.routes", SectionKind::U32, base_gates)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let raw_gates: Vec<i32> = read_section("discrete.gates", SectionKind::I32, total_gates)?
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        if raw_routes.iter().any(|&r| r >= 64) {
            return Err(invalid("discrete.routes contains action index >= 64"));
        }
        if raw_gates.iter().any(|&g| !(-1..=1).contains(&g)) {
            return Err(invalid("discrete.gates contains trit outside -1..=1"));
        }

        let mut token_blocks = Vec::with_capacity(vocab);
        let mut codebook_roots = if blocks == 4 {
            Vec::with_capacity(vocab)
        } else {
            Vec::new()
        };
        for token in 0..vocab {
            let mut tb = Vec::with_capacity(blocks);
            let mut first_root = 0u32;
            let mut last_root = 0u32;
            let mut roots4 = [0u8; 4];
            for block in 0..blocks {
                let start = (token * blocks + block) * 8;
                let mut coords = [0i32; 8];
                coords.copy_from_slice(&raw_table[start..start + 8]);
                let root = root_id(&coords).ok_or_else(|| {
                    invalid("discrete.table block is outside the 16-root SpiralCore orbit")
                })? as u32;
                if block == 0 {
                    first_root = root;
                }
                if block + 1 == blocks {
                    last_root = root;
                }
                if block < 4 {
                    roots4[block] = root as u8;
                }
                tb.push(coords);
            }
            let expected_cell = (first_root << 4) + last_root;
            if cells[token] != expected_cell {
                return Err(invalid(
                    "discrete.cells entry disagrees with discrete.table endpoint roots",
                ));
            }
            if blocks == 4 {
                codebook_roots.push(roots4);
            }
            token_blocks.push(tb);
        }

        let routes: Vec<Vec<u32>> = (0..blocks)
            .map(|b| raw_routes[b * stages..(b + 1) * stages].to_vec())
            .collect();
        let level_gates: Vec<Vec<i32>> = (0..blocks)
            .map(|b| raw_gates[b * stages..(b + 1) * stages].to_vec())
            .collect();
        let mut relation_gates = Vec::new();
        if matches!(read_mode, DiscreteReadMode::Relation { .. }) {
            for stage in 0..stages {
                let mut rg = [0i32; EDGE_TYPES];
                let base = base_gates + stage * EDGE_TYPES;
                rg.copy_from_slice(&raw_gates[base..base + EDGE_TYPES]);
                relation_gates.push(rg);
            }
        }
        let pick_transport = match (read_mode, s.read_position) {
            (DiscreteReadMode::Pick, 3) => Some(PickTransport::Identity),
            (DiscreteReadMode::Pick, _) => {
                let frames: Vec<u8> =
                    read_section("discrete.position_frames", SectionKind::U32, blocks * 16)?
                        .chunks_exact(4)
                        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                        .map(|f| u8::try_from(f).ok().filter(|&f| f < 64))
                        .collect::<Option<_>>()
                        .ok_or_else(|| {
                            invalid("discrete.position_frames contains frame index >= 64")
                        })?;
                Some(PickTransport::PrefixHolonomy { frames })
            }
            _ => None,
        };

        let trunk = if is_trunk {
            let read_u32_any = |idx: usize, expected_name: &str| -> Result<Vec<u32>> {
                let entry = header
                    .tables
                    .get(idx)
                    .ok_or_else(|| invalid(format!("missing trunk table index {idx}")))?;
                if entry.name != expected_name || entry.kind != SectionKind::U32 {
                    return Err(invalid(format!(
                        "trunk table {expected_name} name or kind mismatch"
                    )));
                }
                let expected_bytes = entry
                    .len
                    .checked_mul(4)
                    .ok_or_else(|| invalid("trunk table byte size overflow"))?;
                if (entry.span.offset as usize) % ALIGN != 0
                    || entry.span.bytes != expected_bytes as u64
                {
                    return Err(invalid(format!(
                        "trunk table {expected_name} alignment or byte span mismatch"
                    )));
                }
                let start = usize::try_from(entry.span.offset)
                    .map_err(|_| invalid("trunk table offset overflow"))?;
                let end = start
                    .checked_add(expected_bytes)
                    .ok_or_else(|| invalid("trunk table end overflow"))?;
                let slice = data
                    .get(start..end)
                    .ok_or_else(|| invalid(format!("trunk table {expected_name} out of bounds")))?;
                Ok(slice
                    .chunks_exact(4)
                    .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .collect())
            };

            let step_words = read_u32_any(4, "discrete.trunk.step_candidates")?;
            if step_words.len() != 64 || step_words.iter().any(|&w| w >= 16) {
                return Err(invalid(
                    "discrete.trunk.step_candidates requires 64 candidates in 0..16",
                ));
            }
            let mut step_candidates = [0u8; 64];
            for (dst, &w) in step_candidates.iter_mut().zip(&step_words) {
                *dst = w as u8;
            }

            let norm_words = read_u32_any(5, "discrete.trunk.norm_candidates")?;
            if norm_words.len() != 4 || norm_words.iter().any(|&w| w >= 16) {
                return Err(invalid(
                    "discrete.trunk.norm_candidates requires 4 candidates in 0..16",
                ));
            }
            let mut norm_candidates = [0u8; 4];
            for (dst, &w) in norm_candidates.iter_mut().zip(&norm_words) {
                *dst = w as u8;
            }

            let unigram_words = read_u32_any(6, "discrete.trunk.unigram")?;
            if unigram_words.len() != 1 + vocab
                || (unigram_words[0] as usize) >= vocab
                || unigram_words[1..].iter().all(|&c| c == 0)
            {
                return Err(invalid(
                    "discrete.trunk.unigram length, best_global, or counts invalid",
                ));
            }
            let unigram_best_global = u16::try_from(unigram_words[0])
                .map_err(|_| invalid("unigram_best_global overflows u16"))?;
            let unigram_counts = unigram_words[1..].to_vec();

            let read_sparse = |idx: usize, name: &str| -> Result<DiscreteSparseCountTable> {
                let words = read_u32_any(idx, name)?;
                DiscreteSparseCountTable::from_triples(&words, vocab)
            };

            let s2 = if has_s2 {
                let entry = &header.tables[19];
                if entry.name != S2_SECTION {
                    return Err(invalid("missing final S2 section"));
                }
                Some(FrozenS2Table::load(
                    read_section(S2_SECTION, SectionKind::U32, entry.len)?,
                    vocab,
                )?)
            } else {
                None
            };
            Some(DiscreteTrunkTables {
                s2,
                active_s2,
                step_candidates,
                norm_candidates,
                unigram_best_global,
                unigram_counts,
                read_table: read_sparse(7, "discrete.trunk.read_table")?,
                rec_table: read_sparse(8, "discrete.trunk.rec_table")?,
                read_rec_table: read_sparse(9, "discrete.trunk.read_rec_table")?,
                rec_shuf_table: read_sparse(10, "discrete.trunk.rec_shuf_table")?,
                read_rec_shuf_table: read_sparse(11, "discrete.trunk.read_rec_shuf_table")?,
                rec_unit_table: read_sparse(12, "discrete.trunk.rec_unit_table")?,
                read_rec_unit_table: read_sparse(13, "discrete.trunk.read_rec_unit_table")?,
                query_table: read_sparse(14, "discrete.trunk.query_table")?,
                query_read_table: read_sparse(15, "discrete.trunk.query_read_table")?,
                query_rec_table: read_sparse(16, "discrete.trunk.query_rec_table")?,
                query_rec_shuf_table: read_sparse(17, "discrete.trunk.query_rec_shuf_table")?,
                query_rec_unit_table: read_sparse(18, "discrete.trunk.query_rec_unit_table")?,
            })
        } else {
            None
        };

        let artifact_sha256 = hex::encode(Sha256::digest(bytes));
        Ok(Self {
            count_mixer: None,
            role_policy: None,
            contextual_source: None,
            assertion_arm: None,
            tokenizer_sha256: header.tokenizer_sha256,
            shape: header.shape,
            source: header.source,
            read_mode,
            objective_policy,
            artifact_sha256,
            token_blocks,
            codebook_roots,
            cells,
            routes,
            level_gates,
            relation_gates,
            trunk,
            state_transport_knockout: false,
            pick_transport,
        })
    }

    /// Count-state read-back for a model carrying the optional frozen S2 family.
    pub fn frozen_s2(&self) -> Option<&FrozenS2Table> {
        self.trunk.as_ref()?.s2.as_ref()
    }

    /// Returns the retained counts only when their artifact selects active prediction.
    pub fn active_s2(&self) -> Option<&FrozenS2Table> {
        self.trunk
            .as_ref()
            .filter(|trunk| trunk.active_s2)?
            .s2
            .as_ref()
    }

    /// Returns `true` when this model was loaded from a 19-table `uor-r4.lut-discrete-trunk/1` container.
    pub fn is_trunk_d11(&self) -> bool {
        self.trunk.is_some()
    }

    /// Return a copy of this model with intermediate state transport knocked out:
    /// every binder-route action is projected to the identity element (`A_0 = 0`),
    /// level transport gates and relation transport gates are zeroed, and `Route`
    /// reads return the un-transported local query token blocks on the 16-root orbit.
    pub fn with_state_transport_knockout(&self) -> Self {
        let mut copy = self.clone();
        for b in &mut copy.routes {
            for r in b.iter_mut() {
                *r = u32::from(IDENTITY);
            }
        }
        for b in &mut copy.level_gates {
            for g in b.iter_mut() {
                *g = 0;
            }
        }
        for s in &mut copy.relation_gates {
            *s = [0; EDGE_TYPES];
        }
        copy.state_transport_knockout = true;
        copy
    }

    pub fn is_state_transport_knocked_out(&self) -> bool {
        self.state_transport_knockout
    }

    pub fn shape(&self) -> &DiscreteLutShape {
        &self.shape
    }

    pub fn count_mixer(&self) -> Option<&crate::count_mixer::CountMixer> {
        self.count_mixer.as_ref()
    }

    /// Schema of the artifact loaded by this instance.
    pub fn artifact_schema(&self) -> &'static str {
        if self.assertion_arm.is_some() {
            return assertion_retrieval::SCHEMA;
        }
        if let Some(arm) = self.contextual_source_arm() {
            arm.schema()
        } else if let Some(mixer) = &self.count_mixer {
            mixer.schema()
        } else if let Some(policy) = &self.role_policy {
            if let Some(onset) = &policy.onset {
                if onset.bank.has_negative_calibration() {
                    return rival_admission::SCHEMA;
                }
                return onset_schema(onset.requested_policy.as_ref());
            }
            match policy.paired.as_ref().map(|paired| &paired.roles) {
                Some(paired_copy::Roles::Frozen(_)) => DISCRETE_PAIRED_SCHEMA,
                Some(paired_copy::Roles::Ordered { .. }) => DISCRETE_ORDERED_ROLE_SCHEMA,
                None => DISCRETE_ROLE_SCHEMA,
            }
        } else if self.active_s2().is_some() {
            DISCRETE_S2_EXPERT_SCHEMA
        } else if self.frozen_s2().is_some() {
            DISCRETE_S2_SCHEMA
        } else if self.is_trunk_d11() {
            DISCRETE_TRUNK_SCHEMA
        } else {
            DISCRETE_SCHEMA
        }
    }

    pub fn onset_mode(&self) -> Option<crate::all_starts::OnsetMode> {
        Some(self.role_policy.as_ref()?.onset.as_ref()?.bank.mode())
    }

    /// A loaded role-copy schema uses the exact-copy cursor.
    pub fn is_role_copy_carrier(&self) -> bool {
        self.role_policy.is_some()
    }

    /// Select an occurrence from the immutable prompt. A role carrier returns an empty
    /// choice on a disabled read or on abstention; legacy models have no such policy.
    pub fn role_choice(
        &self,
        prompt: &[u32],
        read: bool,
    ) -> Result<Option<crate::content_selector::Choice>> {
        if let Some(result) = self.paired_copy_decision(prompt, read)? {
            return Ok(Some(result.choice));
        }
        self.role_policy
            .as_ref()
            .map(|policy| {
                if read {
                    policy.choose(prompt, &self.packed_codebook_roots())
                } else {
                    Ok(crate::content_selector::Choice::default())
                }
            })
            .transpose()
    }

    /// Trace the learned role, count of complete supporting values and exact accepted source.
    pub fn paired_copy_decision(
        &self,
        prompt: &[u32],
        read: bool,
    ) -> Result<Option<PairedCopyDecision>> {
        let Some((policy, paired)) = self.paired_policy() else {
            return Ok(None);
        };
        if !read {
            return Ok(Some(PairedCopyDecision::default()));
        }
        paired_copy::Policy::validate_ids(prompt)?;
        let roles = match &paired.roles {
            paired_copy::Roles::Frozen(_) => {
                self.paired_roles_diagnostic(prompt, 0..prompt.len(), false)?
            }
            paired_copy::Roles::Ordered { bank, framing, .. } => self
                .ordered_role_sites()?
                .prompt(prompt, framing.content(prompt)?)?
                .predict(bank.mode, &bank.prototypes, 0..prompt.len())?,
        };
        self.resolve_paired_roles(policy, paired, prompt, roles, |_| {})
            .map(Some)
    }

    fn paired_policy(&self) -> Option<(&RolePolicy, &paired_copy::Policy)> {
        let policy = self.role_policy.as_ref()?;
        Some((policy, policy.paired.as_ref()?))
    }

    pub fn ordered_role_sites(&self) -> Result<RoleSites<'_>> {
        let (_, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("ordered-role sites require a paired artifact"))?;
        Ok(match &paired.roles {
            paired_copy::Roles::Frozen(roles) => {
                RoleSites::from_frozen(&self.codebook_roots, roles.sites())
            }
            paired_copy::Roles::Ordered { classes, .. } => {
                RoleSites::from_d11(&self.codebook_roots, classes)
            }
        })
    }

    pub fn ordered_role_tables(&self) -> Result<(Vec<u32>, Vec<u32>)> {
        let sites = self.ordered_role_sites()?;
        Ok((
            sites.packed_roots().collect(),
            sites.word_classes().map(u32::from).collect(),
        ))
    }

    pub fn ordered_role_bank(&self) -> Option<&RoleBank> {
        match &self.paired_policy()?.1.roles {
            paired_copy::Roles::Ordered { bank, .. } => Some(bank),
            paired_copy::Roles::Frozen(_) => None,
        }
    }

    pub fn ordered_role_framing(&self) -> Option<&RoleFraming> {
        match &self.paired_policy()?.1.roles {
            paired_copy::Roles::Ordered { framing, .. } => Some(framing),
            paired_copy::Roles::Frozen(_) => None,
        }
    }

    pub fn ordered_role_bindings(
        &self,
        framing: &RoleFraming,
        train_digest: [u8; 32],
    ) -> Result<RoleBindings> {
        framing.validate()?;
        let tokenizer = self
            .tokenizer_sha256()
            .ok_or_else(|| invalid("paired artifact lacks a bound tokenizer"))?;
        let sites = self.ordered_role_sites()?;
        let classes: Vec<_> = sites.word_classes().collect();
        let (_, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("missing paired role policy"))?;
        let mut bindings = match &paired.roles {
            paired_copy::Roles::Frozen(legacy) => role_bindings(
                &hex::encode(legacy.bindings()[0]),
                tokenizer,
                Sha256::digest(&classes).into(),
                framing,
                train_digest,
            )?,
            paired_copy::Roles::Ordered { bank, .. } => *bank.bindings(),
        };
        bindings[4] = train_digest;
        bindings[5] = framing.operator_digest();
        Ok(bindings)
    }

    /// Diagnostic legacy ranking keeps full-prefix features even when candidate
    /// membership is bounded. Ordered roles clip features in the supplied region.
    pub fn paired_roles_diagnostic(
        &self,
        prompt: &[u32],
        region: core::ops::Range<usize>,
        lexical_only: bool,
    ) -> Result<PairedRoleDecision> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("paired role diagnostic requires a paired artifact"))?;
        paired_copy::Policy::validate_ids(prompt)?;
        match &paired.roles {
            paired_copy::Roles::Frozen(roles) => {
                if !lexical_only {
                    if let Some(requested) = policy.requested_policy() {
                        return roles.predict_requested_occurrence(
                            prompt,
                            requested.framing().content(prompt)?,
                            region,
                            &paired.delimiters,
                            &policy.span.classes,
                            requested.context(),
                        );
                    }
                }
                roles.predict_where(prompt, region, |span| {
                    !lexical_only
                        || paired
                            .assertion_sites(policy, prompt, span)
                            .next()
                            .is_some()
                })
            }
            paired_copy::Roles::Ordered { bank, framing, .. } => {
                if lexical_only {
                    return Err(invalid("lexical eligibility is a frozen-role diagnostic"));
                }
                self.ordered_role_sites()?
                    .prompt(prompt, framing.content(prompt)?)?
                    .predict(bank.mode, &bank.prototypes, region)
            }
        }
    }

    pub fn requested_role_framing(&self) -> Option<&RoleFraming> {
        self.role_policy.as_ref()?.requested_framing()
    }

    pub fn frozen_role_replay(&self) -> Result<FrozenRoleReplay<'_>> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("frozen role replay requires a paired artifact"))?;
        let paired_copy::Roles::Frozen(roles) = &paired.roles else {
            return Err(invalid("frozen role replay requires frozen roles"));
        };
        Ok(FrozenRoleReplay {
            roles,
            delimiters: &paired.delimiters,
            assertion_classes: &policy.span.classes,
            requested_framing: policy.requested_framing(),
            requested_context: policy.requested_policy().map(RequestedPolicy::context),
        })
    }

    /// Compare role scope and candidate enumeration using the sealed bank, then
    /// hand the result to the ordinary assertion and source reader.
    pub fn requested_role_replay(
        &self,
        prompt: &[u32],
        arm: crate::paired_role::RequestedReplayArm,
    ) -> Result<(PairedCopyDecision, RequestedRoleReplayTrace)> {
        let replay = self.frozen_role_replay()?;
        let framing = replay
            .requested_framing
            .ok_or_else(|| invalid("role replay requires literal-role framing"))?;
        let context = replay
            .requested_context
            .ok_or_else(|| invalid("role replay requires an occurrence policy"))?;
        let (decision, roles) = replay.roles.requested_occurrence_replay(
            prompt,
            framing.content(prompt)?,
            0..prompt.len(),
            replay.delimiters,
            replay.assertion_classes,
            context,
            arm,
        )?;
        let (copy, assertions) = self.paired_copy_witness(prompt, decision, None)?;
        let mut contextual_sources = Vec::new();
        if let Some(query) = decision.requested {
            for witness in &assertions {
                if let Some(expected) = witness.contextual_source {
                    let [start, end] = witness.fact_span;
                    let (selection, candidates) =
                        self.trace_bound_contextual_source(prompt, RoleSpan { start, end }, query)?;
                    if selection != expected {
                        return Err(invalid(
                            "role replay source trace differs from ordinary reader",
                        ));
                    }
                    contextual_sources.push(ReplayContextualSource {
                        fact_span: witness.fact_span,
                        selection,
                        candidates,
                    });
                }
            }
        }
        Ok((
            copy,
            RequestedRoleReplayTrace {
                roles,
                assertions,
                contextual_sources,
            },
        ))
    }

    pub fn frozen_role_trace(
        &self,
        prompt: &[u32],
        region: core::ops::Range<usize>,
        lexical_only: bool,
    ) -> Result<Vec<crate::paired_role::FrozenRoleCandidateTrace>> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("frozen role trace requires its paired artifact"))?;
        let paired_copy::Roles::Frozen(roles) = &paired.roles else {
            return Err(invalid("frozen role trace requires its paired artifact"));
        };
        roles.trace_where(prompt, region, |span| {
            !lexical_only
                || paired
                    .assertion_sites(policy, prompt, span)
                    .next()
                    .is_some()
        })
    }

    pub fn ordered_role_trace(
        &self,
        prompt: &[u32],
        region: core::ops::Range<usize>,
    ) -> Result<Vec<crate::ordered_role::RoleCandidateTrace>> {
        let bank = self
            .ordered_role_bank()
            .ok_or_else(|| invalid("missing ordered-role bank"))?;
        let framing = self
            .ordered_role_framing()
            .ok_or_else(|| invalid("missing ordered-role framing"))?;
        self.ordered_role_sites()?
            .prompt(prompt, framing.content(prompt)?)?
            .trace(bank.mode, &bank.prototypes, region)
    }

    /// Run the actual assertion fence, consensus and read admission from
    /// separately computed diagnostic roles. An override changes only requested
    /// endpoints and must denote a complete original candidate word.
    pub fn paired_copy_diagnostic(
        &self,
        prompt: &[u32],
        roles: PairedRoleDecision,
        requested_override: Option<RoleSpan>,
    ) -> Result<PairedCopyDecision> {
        self.paired_copy_diagnostic_inner(prompt, roles, requested_override, |_| {})
    }

    pub fn paired_copy_witness(
        &self,
        prompt: &[u32],
        roles: PairedRoleDecision,
        requested_override: Option<RoleSpan>,
    ) -> Result<(PairedCopyDecision, Vec<PairedAssertionWitness>)> {
        let mut witnesses = Vec::new();
        let decision =
            self.paired_copy_diagnostic_inner(prompt, roles, requested_override, |item| {
                witnesses.push(item);
            })?;
        Ok((decision, witnesses))
    }

    fn paired_copy_diagnostic_inner(
        &self,
        prompt: &[u32],
        mut roles: PairedRoleDecision,
        requested_override: Option<RoleSpan>,
        visit: impl FnMut(PairedAssertionWitness),
    ) -> Result<PairedCopyDecision> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("paired copy diagnostic requires a paired artifact"))?;
        if let Some(span) = requested_override {
            let content = self
                .ordered_role_framing()
                .map_or_else(|| Ok(1..prompt.len()), |framing| framing.content(prompt))?;
            if !self
                .ordered_role_sites()?
                .prompt(prompt, content)?
                .candidates()
                .any(|candidate| candidate == span)
            {
                return Err(invalid(
                    "requested override must be a complete original candidate",
                ));
            }
            roles.requested = Some(span);
        }
        self.resolve_paired_roles(policy, paired, prompt, roles, visit)
    }

    fn resolve_paired_roles(
        &self,
        policy: &RolePolicy,
        paired: &paired_copy::Policy,
        prompt: &[u32],
        roles: PairedRoleDecision,
        visit: impl FnMut(PairedAssertionWitness),
    ) -> Result<PairedCopyDecision> {
        let selected = match (self.assertion_arm, roles.requested) {
            (Some(arm), Some(person)) => Some(
                self.assertion_retrieval_plan(prompt, person, roles.excluded)?
                    .select(arm)
                    .selected_facts,
            ),
            _ => None,
        };
        self.resolve_assertion_subset(policy, paired, prompt, roles, selected.as_deref(), visit)
    }

    fn resolve_assertion_subset(
        &self,
        policy: &RolePolicy,
        paired: &paired_copy::Policy,
        prompt: &[u32],
        roles: PairedRoleDecision,
        selected: Option<&[RoleSpan]>,
        visit: impl FnMut(PairedAssertionWitness),
    ) -> Result<PairedCopyDecision> {
        let roots;
        let packed = if let Some(onset) = &policy.onset {
            onset.packed_roots.as_slice()
        } else if let paired_copy::Roles::Ordered { packed_roots, .. } = &paired.roles {
            packed_roots.as_slice()
        } else {
            roots = self.packed_codebook_roots();
            &roots
        };
        paired.choose_with_roles(
            policy,
            prompt,
            packed,
            roles,
            selected,
            visit,
            |fact, query| self.contextual_selection(prompt, fact, query),
        )
    }

    /// Replay only the named original assertion heads. The same native binder
    /// still requires complete, agreeing values and owns the copy address.
    pub fn paired_copy_selected_witness(
        &self,
        prompt: &[u32],
        roles: PairedRoleDecision,
        selected: &[RoleSpan],
    ) -> Result<(PairedCopyDecision, Vec<PairedAssertionWitness>)> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("missing paired reader"))?;
        let mut witnesses = Vec::new();
        let result =
            self.resolve_assertion_subset(policy, paired, prompt, roles, Some(selected), |w| {
                witnesses.push(w)
            })?;
        if witnesses.len() != selected.len() {
            return Err(invalid(
                "assertion selection includes an inadmissible or repeated head",
            ));
        }
        Ok((result, witnesses))
    }

    pub fn source(&self) -> &DiscreteLutSource {
        &self.source
    }

    pub fn read_mode(&self) -> DiscreteReadMode {
        self.read_mode
    }

    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }

    /// Tokenizer-byte identity for serving this model from text. The two released
    /// M1 files predate the header field and are bound by their full-file digests.
    pub fn tokenizer_sha256(&self) -> Option<&str> {
        if let Some(digest) = &self.tokenizer_sha256 {
            return Some(digest);
        }
        match self.artifact_sha256.as_str() {
            "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d" => {
                Some("a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377")
            }
            "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de" => {
                Some("f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e")
            }
            _ => None,
        }
    }

    pub fn cells(&self) -> &[u32] {
        &self.cells
    }

    pub fn packed_codebook_roots(&self) -> Vec<u32> {
        self.codebook_roots
            .iter()
            .map(|&r| u32::from(pack_4block_roots(r)))
            .collect()
    }

    /// Compile the trained four-block Identity Pick codes once for A1 serving.
    #[cfg(feature = "p1-count-serving")]
    pub fn identity_pick_codes(&self) -> Result<Vec<[u16; 4]>> {
        if !self.is_trunk_d11()
            || !matches!(self.read_mode, DiscreteReadMode::Relation { .. })
            || self.shape.read_position != 0
            || self.shape.blocks != 4
        {
            return Err(invalid(
                "A1 requires a four-block Relation trunk with its original token codes",
            ));
        }
        self.token_blocks
            .iter()
            .map(|blocks| {
                let row: &[[i32; 8]; 4] = blocks
                    .as_slice()
                    .try_into()
                    .map_err(|_| invalid("wrong A1 token width"))?;
                Ok(row.map(|root| root_code(&root)))
            })
            .collect()
    }

    /// Reuse only the underlying Relation trunk's common orbit table, routes
    /// and level gates. Extra relation gates and relation transport are excluded.
    #[doc(hidden)]
    pub fn identity_pick_inventory(&self) -> Result<(Vec<i32>, Vec<u32>, Vec<i32>)> {
        if !self.is_trunk_d11()
            || !matches!(self.read_mode, DiscreteReadMode::Relation { .. })
            || self.shape.read_position != 0
        {
            return Err(invalid(
                "identity Pick diagnostic requires a Relation trunk",
            ));
        }
        Ok((
            self.token_blocks
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect(),
            self.routes.iter().flatten().copied().collect(),
            self.level_gates.iter().flatten().copied().collect(),
        ))
    }

    /// This raw availability diagnostic supplies identity transport explicitly.
    /// The successor is the stored original ID and is independent of gate trits.
    #[doc(hidden)]
    pub fn observe_identity_pick(&self, prefix: &[u32]) -> Result<IdentityPickDiagnostic> {
        if !self.is_trunk_d11()
            || !matches!(self.read_mode, DiscreteReadMode::Relation { .. })
            || self.shape.read_position != 0
            || prefix.is_empty()
            || prefix.len() > self.shape.context as usize
            || prefix
                .iter()
                .enumerate()
                .any(|(at, &id)| id >= self.shape.vocabulary || id == 1 || (id == 0 && at != 0))
        {
            return Err(invalid("invalid causal prefix or Relation Pick source"));
        }
        let gates: Vec<_> = self.level_gates.iter().map(|g| g[0]).collect();
        let read = pick_read(&self.token_blocks, &gates, &PickTransport::Identity, prefix)?;
        let block_words = read
            .blocks
            .iter()
            .zip(gates)
            .map(|(block, gate)| {
                Ok([
                    read.pick.map_or(u32::MAX, |p| p as u32),
                    read.distance.unwrap_or(u32::MAX),
                    read.ties,
                    (gate + 1) as u32,
                    root_id(block).ok_or_else(|| invalid("Pick output outside orbit"))? as u32,
                ])
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(IdentityPickDiagnostic {
            vector: read.blocks.into_iter().flatten().collect(),
            block_words,
            successor: read.pick.map(|p| prefix[p + 1]),
        })
    }

    fn route_read(
        &self,
        prefix: &[u32],
        plan: &RowPlan,
        mut trace: Option<&mut Vec<u32>>,
    ) -> Result<Vec<[i32; 8]>> {
        let blocks = self.shape.blocks as usize;
        let stages = self.shape.stages as usize;
        let last_token = prefix[prefix.len() - 1] as usize;
        if self.state_transport_knockout {
            let out = (0..blocks)
                .map(|block| self.token_blocks[last_token][block])
                .collect::<Vec<_>>();
            if let Some(words) = trace {
                let stride =
                    DISCRETE_ROUTE_TRACE_HEADER_WORDS + stages * DISCRETE_ROUTE_TRACE_STAGE_WORDS;
                for block in &out {
                    let start = words.len();
                    words.resize(start + stride, u32::MAX);
                    words[start] = plan.frontier.len() as u32;
                    words[start + 1] = root_id(block).unwrap_or(usize::MAX) as u32;
                    words[start + 2] = 0;
                }
            }
            return Ok(out);
        }
        let mut out = Vec::with_capacity(blocks);
        for block in 0..blocks {
            let query = &self.token_blocks[last_token][block];
            let mut node = 0usize;
            let mut root = 0usize;
            let mut total = usize::from(IDENTITY);
            let mut resolved = None;
            let trace_start = trace.as_ref().map(|words| words.len());
            if let Some(words) = trace.as_deref_mut() {
                words.resize(
                    words.len()
                        + DISCRETE_ROUTE_TRACE_HEADER_WORDS
                        + stages * DISCRETE_ROUTE_TRACE_STAGE_WORDS,
                    u32::MAX,
                );
            }
            for stage in 0..stages {
                let frame = self.routes[block][stage] as usize;
                if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                    let at = start
                        + DISCRETE_ROUTE_TRACE_HEADER_WORDS
                        + stage * DISCRETE_ROUTE_TRACE_STAGE_WORDS;
                    words[at] = frame as u32;
                    words[at + 1] = root as u32;
                }
                let q = action(frame, query);
                let count = if stage == 0 { plan.frontier.len() } else { 3 };
                let mut best = 0usize;
                let mut best_distance = u32::MAX;
                let mut best_hamming = u32::MAX;
                let mut goal = 0usize;
                for port in 0..count {
                    let r = if stage == 0 {
                        port
                    } else {
                        usize::from(BRANCH[root][port])
                    };
                    let (distance, hamming) = root_cost(&q, r);
                    if distance < best_distance
                        || (distance == best_distance && hamming < best_hamming)
                    {
                        best = port;
                        best_distance = distance;
                        best_hamming = hamming;
                        goal = r;
                    }
                }
                total = compose(usize::from(BFS[(root << 4) + goal]), total);
                root = goal;
                let next_node = if stage == 0 {
                    plan.frontier[best]
                } else {
                    let children = plan.nodes[node]
                        .children
                        .ok_or_else(|| invalid("route stepped past an atom leaf"))?;
                    children[best]
                };
                if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                    let at = start
                        + DISCRETE_ROUTE_TRACE_HEADER_WORDS
                        + stage * DISCRETE_ROUTE_TRACE_STAGE_WORDS;
                    words[at + 2] = best as u32;
                    words[at + 3] = goal as u32;
                    words[at + 4] = total as u32;
                    words[at + 5] = next_node as u32;
                }
                if let Some(atom) = plan.nodes[next_node].atom {
                    let carrier_token = prefix[atom] as usize;
                    let carrier = &self.token_blocks[carrier_token][block];
                    resolved = Some(action(total, carrier));
                    if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                        words[start] = plan.frontier.len() as u32;
                        words[start + 2] = (stage + 1) as u32;
                        words[start + 3] = atom as u32;
                    }
                    break;
                }
                node = next_node;
            }
            let resolved = resolved.ok_or_else(|| invalid("route did not reach an atom leaf"))?;
            if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                words[start + 1] = root_id(&resolved)
                    .ok_or_else(|| invalid("Route output outside SpiralCore orbit"))?
                    as u32;
            }
            out.push(resolved);
        }
        Ok(out)
    }

    fn flock_or_relation_read(
        &self,
        prefix: &[u32],
        plan: &RowPlan,
        kept: usize,
        sweeps: u32,
        relation_active: bool,
        mut trace: Option<&mut Vec<u32>>,
    ) -> Result<FlockReadParts> {
        let context = self.shape.context as usize;
        let blocks = self.shape.blocks as usize;
        let stages = self.shape.stages as usize;
        let length = prefix.len();
        let nodes_count = plan.nodes.len();
        let last_token = prefix[length - 1] as usize;
        let mut out = Vec::with_capacity(blocks);
        let mut all_rel_scores = if relation_active {
            Some(Vec::with_capacity(blocks * nodes_count))
        } else {
            None
        };
        let max_nodes_count = (relation_active && trace.is_some())
            .then(|| build_row_plan(context).map(|plan| plan.nodes.len()))
            .transpose()?;
        let mut all_rel_trace =
            max_nodes_count.map(|max_nodes_count| Vec::with_capacity(blocks * max_nodes_count));

        for b in 0..blocks {
            let trace_start = trace.as_ref().map(|words| words.len());
            if let Some(words) = trace.as_deref_mut() {
                words.resize(
                    words.len()
                        + DISCRETE_ACTIVE_TRACE_HEADER_WORDS
                        + stages * DISCRETE_ACTIVE_TRACE_LEVEL_WORDS,
                    u32::MAX,
                );
            }
            let f = if b == 0 { blocks - 1 } else { 0 };
            let query_frame = &self.token_blocks[last_token][f];
            let mut u = self.token_blocks[last_token][b];
            let fq = root_id(query_frame)
                .ok_or_else(|| invalid("query frame outside SpiralCore orbit"))?;
            if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                words[start] = fq as u32;
                words[start + 3] = root_id(&u)
                    .ok_or_else(|| invalid("query payload outside SpiralCore orbit"))?
                    as u32;
            }

            let mut frame = vec![0usize; nodes_count];
            let mut payload = vec![[0i32; 8]; nodes_count];
            for (node_idx, node) in plan.nodes.iter().enumerate() {
                if let Some(atom) = node.atom {
                    let tok = prefix[atom] as usize;
                    frame[node_idx] = root_id(&self.token_blocks[tok][f])
                        .ok_or_else(|| invalid("atom frame outside SpiralCore orbit"))?;
                    payload[node_idx] = self.token_blocks[tok][b];
                } else {
                    let children = node
                        .children
                        .ok_or_else(|| invalid("non-atom node missing children"))?;
                    let c0 = children[0];
                    frame[node_idx] = frame[c0];
                    let mut sum = [0i32; 8];
                    for &child in &children {
                        let act = usize::from(BFS[(frame[child] << 4) + frame[c0]]);
                        let carried = action(act, &payload[child]);
                        for d in 0..8 {
                            sum[d] += carried[d];
                        }
                    }
                    let r = snap(&sum, 3);
                    payload[node_idx] = ROOTS[r].map(i32::from);
                }
            }

            let mut rel = vec![0i32; nodes_count];
            if relation_active {
                let mut prev: Vec<i32> = (0..nodes_count)
                    .map(|n| 8 - (root_cost(query_frame, frame[n]).0 as i32))
                    .collect();
                for _ in 0..sweeps {
                    for i in 0..nodes_count {
                        if plan.nodes[i].last >= length {
                            continue;
                        }
                        let li = plan.nodes[i].level;
                        for e in 0..EDGE_TYPES {
                            let gt = self.relation_gates[li][e];
                            if gt == 0 {
                                continue;
                            }
                            let mut esum = 0i32;
                            if e == 0 {
                                if let Some(children) = plan.nodes[i].children {
                                    for ch in children {
                                        if plan.nodes[ch].last < length {
                                            esum += prev[ch];
                                        }
                                    }
                                }
                                for p in (i + 1)..nodes_count {
                                    if let Some(children) = plan.nodes[p].children {
                                        if plan.nodes[p].last < length
                                            && (children[0] == i
                                                || children[1] == i
                                                || children[2] == i)
                                        {
                                            esum += prev[p];
                                        }
                                    }
                                }
                            } else if e == 1 {
                                for p in (i + 1)..nodes_count {
                                    let Some([c0, c1, c2]) = plan.nodes[p].children else {
                                        continue;
                                    };
                                    if plan.nodes[p].last >= length {
                                        continue;
                                    }
                                    if i == c0 && plan.nodes[c1].last < length {
                                        esum += prev[c1];
                                    } else if i == c1 {
                                        if plan.nodes[c0].last < length {
                                            esum += prev[c0];
                                        }
                                        if plan.nodes[c2].last < length {
                                            esum += prev[c2];
                                        }
                                    } else if i == c2 && plan.nodes[c1].last < length {
                                        esum += prev[c1];
                                    }
                                }
                            } else if e == 2 {
                                for j in 0..i {
                                    if plan.nodes[j].last < length
                                        && plan.nodes[j].level == li
                                        && plan.nodes[j].last + 1 == plan.nodes[i].first
                                    {
                                        esum += prev[j];
                                    }
                                }
                            } else if e == 3 {
                                if li == 0 {
                                    let tid = prefix[plan.nodes[i].last];
                                    for j in 0..i {
                                        if plan.nodes[j].last < length
                                            && plan.nodes[j].level == 0
                                            && prefix[plan.nodes[j].last] == tid
                                        {
                                            esum = prev[j];
                                        }
                                    }
                                }
                            } else if e == 4 {
                                for j in 0..i {
                                    if plan.nodes[j].last < length
                                        && plan.nodes[j].level == li
                                        && frame[j] == frame[i]
                                    {
                                        esum = prev[j];
                                    }
                                }
                            }
                            rel[i] += if gt > 0 { esum } else { -esum };
                        }
                    }
                    prev.copy_from_slice(&rel);
                }
                if let Some(scores) = &mut all_rel_scores {
                    scores.extend_from_slice(&rel);
                }
                if let (Some(scores), Some(max_nodes_count)) = (&mut all_rel_trace, max_nodes_count)
                {
                    scores.extend(rel.iter().map(|&score| score as u32));
                    scores.resize((b + 1) * max_nodes_count, u32::MAX);
                }
            }

            let mut terms = 1usize;
            for l in 0..stages {
                let gate = self.level_gates[b][l];
                if gate == 0 && trace.is_none() {
                    continue;
                }
                let mut top = [0usize; MAX_KEPT];
                let mut key_r = [0i32; MAX_KEPT];
                let mut key_d = [0u32; MAX_KEPT];
                let mut key_h = [0u32; MAX_KEPT];
                let mut found = 0usize;
                let mut eligible = 0usize;
                for node in 0..nodes_count {
                    if plan.nodes[node].level != l || plan.nodes[node].last + 1 >= length {
                        continue;
                    }
                    eligible += 1;
                    let (distance, hamming) = root_cost(query_frame, frame[node]);
                    let rscore = rel[node];
                    let mut at = found;
                    while at > 0 {
                        let o = top[at - 1];
                        let better = rscore > key_r[at - 1]
                            || (rscore == key_r[at - 1]
                                && (distance < key_d[at - 1]
                                    || (distance == key_d[at - 1]
                                        && (hamming < key_h[at - 1]
                                            || (hamming == key_h[at - 1]
                                                && (plan.nodes[node].last
                                                    > plan.nodes[o].last
                                                    || (plan.nodes[node].last
                                                        == plan.nodes[o].last
                                                        && node < o)))))));
                        if !better {
                            break;
                        }
                        if at < kept {
                            top[at] = top[at - 1];
                            key_r[at] = key_r[at - 1];
                            key_d[at] = key_d[at - 1];
                            key_h[at] = key_h[at - 1];
                        }
                        at -= 1;
                    }
                    if at < kept {
                        top[at] = node;
                        key_r[at] = rscore;
                        key_d[at] = distance;
                        key_h[at] = hamming;
                        if found < kept {
                            found += 1;
                        }
                    }
                }
                if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                    let at = start
                        + DISCRETE_ACTIVE_TRACE_HEADER_WORDS
                        + l * DISCRETE_ACTIVE_TRACE_LEVEL_WORDS;
                    words[at] = (gate + 1) as u32;
                    words[at + 1] = found as u32;
                    words[at + 3] = eligible as u32;
                }
                if found == 0 {
                    continue;
                }
                let mut sum = [0i32; 8];
                for &node in &top[..found] {
                    let act = usize::from(BFS[(frame[node] << 4) + fq]);
                    let carried = action(act, &payload[node]);
                    for d in 0..8 {
                        sum[d] += carried[d];
                    }
                }
                let m = snap(&sum, found);
                if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                    let at = start
                        + DISCRETE_ACTIVE_TRACE_HEADER_WORDS
                        + l * DISCRETE_ACTIVE_TRACE_LEVEL_WORDS;
                    words[at] = (gate + 1) as u32;
                    words[at + 1] = found as u32;
                    words[at + 2] = m as u32;
                    words[at + 3] = eligible as u32;
                    for k in 0..found {
                        words[at + 4 + k] = top[k] as u32;
                        let act = usize::from(BFS[(frame[top[k]] << 4) + fq]);
                        let carried = action(act, &payload[top[k]]);
                        words[at + 12 + k] = act as u32;
                        words[at + 20 + k] = root_id(&carried)
                            .ok_or_else(|| invalid("carried payload outside SpiralCore orbit"))?
                            as u32;
                    }
                }
                if gate == 0 {
                    continue;
                }
                for d in 0..8 {
                    let v = i32::from(ROOTS[m][d]);
                    u[d] += if gate > 0 { v } else { -v };
                }
                terms += 1;
            }
            let out_root = snap(&u, terms);
            if let (Some(words), Some(start)) = (trace.as_deref_mut(), trace_start) {
                words[start + 1] = out_root as u32;
                words[start + 2] = terms as u32;
            }
            out.push(ROOTS[out_root].map(i32::from));
        }
        Ok((out, all_rel_scores, all_rel_trace))
    }

    /// Run the Pick read on one causal prefix with this model's blocks, level-0
    /// gates and position transport, returning the pick beside the read blocks.
    pub(crate) fn pick_row(&self, prefix: &[u32]) -> Result<PickRead> {
        let transport = self
            .pick_transport
            .as_ref()
            .ok_or_else(|| invalid("Pick read requested on a model without Pick transport"))?;
        let gates0: Vec<i32> = self.level_gates.iter().map(|g| g[0]).collect();
        pick_read(&self.token_blocks, &gates0, transport, prefix)
    }

    fn address_distance(&self, q: &[[i32; 8]], cell: u32) -> u32 {
        let a = root_cost(&q[0], (cell >> 4) as usize).0;
        let b = root_cost(&q[q.len() - 1], (cell & 15) as usize).0;
        a + b
    }

    fn token_distance(&self, q: &[[i32; 8]], token: usize) -> u32 {
        let mut sum = 0u32;
        for (block_q, block_v) in q.iter().zip(&self.token_blocks[token]) {
            for d in 0..8 {
                sum += (block_q[d] - block_v[d]).unsigned_abs();
            }
        }
        sum
    }

    /// Evaluate a causal token prefix (`1..=context`) and return the read vector,
    /// admitted cells, top-8 ranked tokens, and top-1 prediction.
    pub fn evaluate_prefix(&self, prefix: &[u32]) -> Result<DiscretePrefixEvaluation> {
        self.evaluate_prefix_internal(prefix, false)
            .map(|traced| traced.evaluation)
    }

    /// Score through the native geometric generator, then commit either its
    /// prediction or the addressed occurrence's exact vocabulary ID. Callers
    /// feed the returned step's emitted ID and state into the next step.
    pub fn evaluate_prefix_with_exact_copy(
        &self,
        prefix: &[u32],
        state: ExactCopyState,
        stops: [u32; 4],
    ) -> Result<(DiscretePrefixEvaluation, ExactCopyStep)> {
        let evaluation = self.evaluate_prefix(prefix)?;
        let step = state.write(prefix, evaluation.prediction, stops)?;
        Ok((evaluation, step))
    }

    /// Evaluate a causal token prefix and retain the exact read decisions made
    /// by the same production path. This is a qualification observation only.
    pub fn evaluate_prefix_traced(&self, prefix: &[u32]) -> Result<DiscreteTracedPrefixEvaluation> {
        self.evaluate_prefix_internal(prefix, true)
    }

    /// The causal geometric read shared by prediction and the contextual
    /// source projector. It does not invoke any role or source selector.
    fn causal_geometric_read(
        &self,
        prefix: &[u32],
        trace: Option<&mut Vec<u32>>,
    ) -> Result<FlockReadParts> {
        let plan = build_row_plan(prefix.len())?;
        let (mut blocks, relation_scores, relation_trace) = match self.read_mode {
            DiscreteReadMode::Route => (self.route_read(prefix, &plan, trace)?, None, None),
            DiscreteReadMode::Flock { kept } => {
                self.flock_or_relation_read(prefix, &plan, kept as usize, 0, false, trace)?
            }
            DiscreteReadMode::Relation { kept, sweeps } => {
                self.flock_or_relation_read(prefix, &plan, kept as usize, sweeps, true, trace)?
            }
            DiscreteReadMode::Pick => (self.pick_row(prefix)?.blocks, None, None),
        };
        if !self.state_transport_knockout {
            if let Some(ref trunk) = self.trunk {
                trunk.apply_joint_pointer_gate(prefix, &mut blocks, &self.codebook_roots)?;
            }
        }
        Ok((blocks, relation_scores, relation_trace))
    }

    fn evaluate_prefix_internal(
        &self,
        prefix: &[u32],
        retain_trace: bool,
    ) -> Result<DiscreteTracedPrefixEvaluation> {
        let vocab = self.shape.vocabulary;
        let context = self.shape.context as usize;
        if prefix.is_empty() || prefix.len() > context {
            return Err(invalid("prefix length must be in 1..=context"));
        }
        if prefix.iter().any(|&tok| tok >= vocab) {
            return Err(invalid("prefix token ID outside vocabulary"));
        }
        if let Some(table) = self.active_s2() {
            let read = table.predict_expert(prefix)?;
            let best = read.top8[0];
            let mut trace_words = Vec::new();
            if retain_trace {
                let key = read.matched_key.unwrap_or(0);
                trace_words.extend([u32::from(read.matched_key.is_some()), u32::from(read.order)]);
                trace_words.extend((0..4).map(|i| (key >> (i * 32)) as u32));
                trace_words.push(read.reads().len() as u32);
                for entry in read.reads() {
                    trace_words.extend([u32::from(entry.token), entry.count]);
                }
                trace_words.extend([
                    u32::from(best.token),
                    best.mass as u32,
                    (best.mass >> 32) as u32,
                ]);
            }
            return Ok(DiscreteTracedPrefixEvaluation {
                evaluation: DiscretePrefixEvaluation {
                    prediction: u32::from(best.token),
                    top8: read
                        .top8
                        .iter()
                        .map(|entry| u32::from(entry.token))
                        .collect(),
                    read_vector: Vec::new(),
                    admitted_cells: Vec::new(),
                    relation_scores: None,
                },
                trace_words,
                relation_trace_words: None,
            });
        }
        if let Some(mixer) = self.count_mixer() {
            let previous = prefix[prefix.len() - 1] as u16;
            let ranked = mixer.top8_for_prefix(prefix);
            let winner = ranked[0];
            let trace_words = if retain_trace {
                let mass = mixer.mass_q32_for_prefix(prefix, winner);
                let mut trace = vec![
                    u32::from(previous),
                    mixer.conditional_weight(),
                    u32::from(winner),
                    mass as u32,
                    (mass >> 32) as u32,
                ];
                if let Some(weights) = mixer.context_weights() {
                    trace.extend(weights);
                    for route in mixer.context_routes(prefix) {
                        trace.extend([route as u32, (route >> 32) as u32]);
                    }
                }
                trace
            } else {
                Vec::new()
            };
            return Ok(DiscreteTracedPrefixEvaluation {
                evaluation: DiscretePrefixEvaluation {
                    prediction: u32::from(winner),
                    top8: ranked.into_iter().map(u32::from).collect(),
                    read_vector: Vec::new(),
                    admitted_cells: Vec::new(),
                    relation_scores: None,
                },
                trace_words,
                relation_trace_words: None,
            });
        }
        let mut trace_words = Vec::new();
        let (q_blocks, relation_scores, relation_trace_words) =
            self.causal_geometric_read(prefix, retain_trace.then_some(&mut trace_words))?;

        let mut occupied = [false; 256];
        for &cell in &self.cells {
            occupied[cell as usize] = true;
        }
        let mut used = [false; 256];
        let mut admitted_cells = Vec::with_capacity(self.shape.kept_cells as usize);
        for _ in 0..self.shape.kept_cells {
            let mut best_cell = None;
            let mut best_cost = u32::MAX;
            for c in 0..256u32 {
                if occupied[c as usize] && !used[c as usize] {
                    let d = self.address_distance(&q_blocks, c);
                    if d < best_cost || (d == best_cost && best_cell.is_none_or(|bc| c < bc)) {
                        best_cell = Some(c);
                        best_cost = d;
                    }
                }
            }
            let Some(cell) = best_cell else {
                break;
            };
            used[cell as usize] = true;
            admitted_cells.push(cell);
        }

        let mut ranked = [u32::MAX; 8];
        let mut distance = [u32::MAX; 8];
        let mut found = 0usize;
        for token in 0..vocab as usize {
            let cell = self.cells[token];
            if !admitted_cells.contains(&cell) {
                continue;
            }
            let d = self.token_distance(&q_blocks, token);
            let tok = token as u32;
            let mut at = found;
            while at > 0 {
                let prev = at - 1;
                if !(d < distance[prev] || (d == distance[prev] && tok < ranked[prev])) {
                    break;
                }
                if at < 8 {
                    ranked[at] = ranked[prev];
                    distance[at] = distance[prev];
                }
                at -= 1;
            }
            if at < 8 {
                ranked[at] = tok;
                distance[at] = d;
                if found < 8 {
                    found += 1;
                }
            }
        }
        if found == 0 {
            return Err(invalid("writer admitted zero tokens"));
        }
        let top8 = ranked[..found].to_vec();
        let prediction = top8[0];
        let read_vector = q_blocks.iter().flatten().copied().collect();
        Ok(DiscreteTracedPrefixEvaluation {
            evaluation: DiscretePrefixEvaluation {
                read_vector,
                admitted_cells,
                top8,
                prediction,
                relation_scores,
            },
            trace_words,
            relation_trace_words,
        })
    }

    /// Evaluate a causal token prefix and compute the exact writer loss numerators
    /// and rival counts against `target` (`0..vocabulary`).
    pub fn observe_target(&self, prefix: &[u32], target: u32) -> Result<DiscreteTargetObservation> {
        if self.count_mixer().is_some() || self.active_s2().is_some() {
            return Err(invalid(
                "count experts use target mass instead of trunk hinge loss",
            ));
        }
        if target >= self.shape.vocabulary {
            return Err(invalid("target token ID outside vocabulary"));
        }
        let evaluation = self.evaluate_prefix(prefix)?;
        let blocks = self.shape.blocks as usize;
        let mut q_blocks = Vec::with_capacity(blocks);
        for b in 0..blocks {
            let mut coords = [0i32; 8];
            coords.copy_from_slice(&evaluation.read_vector[(b << 3)..((b + 1) << 3)]);
            q_blocks.push(coords);
        }

        let target_cell = self.cells[target as usize];
        let mut token_rivals = 0u32;
        let mut address_rivals = 0u32;
        let mut rival_distance = u32::MAX;
        let mut rival_address = u32::MAX;
        for &cell in &evaluation.admitted_cells {
            let cost = self.address_distance(&q_blocks, cell);
            if cell != target_cell {
                address_rivals += 1;
                if cost < rival_address {
                    rival_address = cost;
                }
            }
            for token in 0..self.shape.vocabulary as usize {
                if self.cells[token] == cell {
                    let d = self.token_distance(&q_blocks, token);
                    if (token as u32) != target {
                        token_rivals += 1;
                        if d < rival_distance {
                            rival_distance = d;
                        }
                    }
                }
            }
        }

        let width = self.shape.blocks << 3;
        let td = self.token_distance(&q_blocks, target as usize);
        let ad = self.address_distance(&q_blocks, target_cell);
        let token_hinge = if rival_distance == u32::MAX || width + td <= rival_distance {
            0
        } else {
            u64::from(width + td - rival_distance)
        };
        let address_hinge = if rival_address == u32::MAX || 8 + ad <= rival_address {
            0
        } else {
            u64::from(8 + ad - rival_address)
        };

        let mut occupied = [false; 256];
        for &cell in &self.cells {
            occupied[cell as usize] = true;
        }
        let mut rank = 1u32;
        for cell in 0..256u32 {
            if occupied[cell as usize] {
                let d = self.address_distance(&q_blocks, cell);
                if d < ad || (d == ad && cell < target_cell) {
                    rank += 1;
                }
            }
        }
        let rank_hinge = if rank > self.shape.kept_cells {
            u64::from(rank - self.shape.kept_cells)
        } else {
            0
        };

        let weighted_token = checked_scaled_add(token_hinge, 8)
            .ok_or_else(|| invalid("weighted token hinge overflow"))?;
        let weighted_address = checked_scaled_add(address_hinge, width)
            .ok_or_else(|| invalid("weighted address hinge overflow"))?;
        let weighted_rank = checked_scaled_add(rank_hinge, width)
            .ok_or_else(|| invalid("weighted rank hinge overflow"))?;
        let loss_ta = weighted_token
            .checked_add(weighted_address)
            .ok_or_else(|| invalid("loss_ta overflow"))?;
        let loss_tr = weighted_token
            .checked_add(weighted_rank)
            .ok_or_else(|| invalid("loss_tr overflow"))?;
        let selected_loss = match self.objective_policy {
            DiscreteObjectivePolicy::NearestAddressMarginV1 => loss_ta,
            DiscreteObjectivePolicy::OccupiedCellRankHingeV1 => loss_tr,
        };

        Ok(DiscreteTargetObservation {
            evaluation,
            token_rivals,
            address_rivals,
            token_hinge,
            address_hinge,
            rank_hinge,
            loss_ta,
            loss_tr,
            selected_loss,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COPY_STOPS: [u32; 4] = [16, 1, 0, 201];

    #[test]
    fn exact_copy_preserves_id_and_replays_the_address_until_release() {
        let mut prefix = vec![0, 677, 1999, 16, 677, 878, 16];
        let state = ExactCopyState::new(Some(1), prefix.len() as u32);
        let first = state.write(&prefix, 1155, COPY_STOPS).unwrap();
        assert_eq!(first.words(), [1, 1, 2, 7, 677, 677, 1, 0]);
        prefix.push(first.emitted);
        let restored = ExactCopyState::from_words(first.state.words());
        let second = restored.write(&prefix, 878, COPY_STOPS).unwrap();
        assert_eq!(second.words(), [1, 1, 3, 7, 1999, 1999, 1, 0]);
        assert_eq!(ExactCopyStep::from_words(second.words()), second);
        prefix.push(second.emitted);
        let release = second.state.write(&prefix, 41, COPY_STOPS).unwrap();
        assert_eq!(release.words(), [0, 1, 3, 7, 41, u32::MAX, 0, 1]);
        assert_eq!(
            release.state.write(&prefix, 1, COPY_STOPS).unwrap().words(),
            [0, 1, 3, 7, 1, u32::MAX, 0, 0]
        );
    }

    #[test]
    fn exact_copy_stops_at_each_delimiter_cap_and_original_fence() {
        for delimiter in COPY_STOPS {
            let step = ExactCopyState::new(Some(0), 1)
                .write(&[delimiter], 42, COPY_STOPS)
                .unwrap();
            assert_eq!(step.words(), [0, 0, 0, 1, 42, u32::MAX, 0, 1]);
        }
        let prefix = [7; 12];
        let mut state = ExactCopyState::new(Some(1), 12);
        for _ in 0..8 {
            state = state.write(&prefix, 42, COPY_STOPS).unwrap().state;
        }
        assert_eq!(
            state.write(&prefix, 42, COPY_STOPS).unwrap().words(),
            [0, 1, 9, 12, 42, u32::MAX, 0, 3]
        );
        let first = ExactCopyState::new(Some(0), 1)
            .write(&[7], 42, COPY_STOPS)
            .unwrap();
        assert_eq!(
            first.state.write(&[7, 7], 42, COPY_STOPS).unwrap().words(),
            [0, 0, 1, 1, 42, u32::MAX, 0, 2]
        );
        let off = ExactCopyState::new(None, 1)
            .write(&[7], 42, COPY_STOPS)
            .unwrap();
        assert_eq!(off.words(), [0, u32::MAX, u32::MAX, 1, 42, u32::MAX, 0, 0]);
    }

    #[test]
    fn exact_copy_rejects_corrupt_restored_words() {
        for corrupted in [
            [1, 2, 2, 32],
            [1, 1, 0, 12],
            [2, 0, 2, 12],
            [1, 0, 9, 12],
            [1, 11, 13, 12],
            [1, 12, 12, 12],
            [1, u32::MAX, u32::MAX, 12],
            [0, u32::MAX, 1, 12],
        ] {
            assert!(ExactCopyState::from_words(corrupted)
                .write(&[7; 12], 42, COPY_STOPS)
                .is_err());
        }
    }

    const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
    const C_ROOT: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
    const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
    const D_ROOT: [i32; 8] = [2, -2, 0, 0, 0, 0, 0, 0];

    fn push_section(blob: &mut Vec<u8>, bytes: &[u8]) -> SectionSpan {
        let pad = (ALIGN - blob.len() % ALIGN) % ALIGN;
        blob.resize(blob.len() + pad, 0);
        let offset = blob.len() as u64;
        blob.extend_from_slice(bytes);
        SectionSpan {
            offset,
            bytes: bytes.len() as u64,
        }
    }

    fn build_discrete_uorlut01(
        shape: DiscreteLutShape,
        tokens: &[[[i32; 8]; 2]],
        routes: &[u32],
        gates: &[i32],
    ) -> Vec<u8> {
        let table: Vec<i32> = tokens.iter().flatten().flatten().copied().collect();
        let cells: Vec<u32> = tokens
            .iter()
            .map(|blocks| {
                let r0 = root_id(&blocks[0]).unwrap() as u32;
                let r1 = root_id(&blocks[1]).unwrap() as u32;
                (r0 << 4) + r1
            })
            .collect();
        let mut blob = Vec::new();
        let mut tables = Vec::new();
        let table_bytes: Vec<u8> = table.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.table".to_owned(),
            kind: SectionKind::I32,
            len: table.len(),
            span: push_section(&mut blob, &table_bytes),
        });
        let cells_bytes: Vec<u8> = cells.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.cells".to_owned(),
            kind: SectionKind::U32,
            len: cells.len(),
            span: push_section(&mut blob, &cells_bytes),
        });
        let routes_bytes: Vec<u8> = routes.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.routes".to_owned(),
            kind: SectionKind::U32,
            len: routes.len(),
            span: push_section(&mut blob, &routes_bytes),
        });
        let gates_bytes: Vec<u8> = gates.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.gates".to_owned(),
            kind: SectionKind::I32,
            len: gates.len(),
            span: push_section(&mut blob, &gates_bytes),
        });
        let header = DiscreteLutHeader {
            tokenizer_sha256: None,
            schema: DISCRETE_SCHEMA.to_owned(),
            shape,
            matrices: Vec::new(),
            tables,
            source: DiscreteLutSource {
                snapshot_schema: SNAPSHOT_SCHEMA.to_owned(),
                snapshot_sha256: "00".repeat(32),
                operator_kappa: OPERATOR_KAPPA.to_owned(),
                composition_kappa: COMPOSITION_KAPPA.to_owned(),
                kernel_sha256: "11".repeat(32),
                source_sha256: "22".repeat(32),
                geometry_sha256: "33".repeat(32),
            },
        };
        let json = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::with_capacity(16 + json.len() + ALIGN + blob.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(json.len() as u64).to_le_bytes());
        out.extend_from_slice(&json);
        let pad = (ALIGN - out.len() % ALIGN) % ALIGN;
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(&blob);
        out
    }

    #[test]
    fn route_writer_and_context_transport_match_cuda_qualification_witnesses() {
        // 1. Two-token exact writer witness (`qualification.rs::exact_writer`).
        let shape_2 = DiscreteLutShape {
            vocabulary: 2,
            blocks: 2,
            context: 1,
            kept_cells: 2,
            stages: 1,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            read_position: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let bytes_2 =
            build_discrete_uorlut01(shape_2.clone(), &[[A, A], [NEG_A, NEG_A]], &[0, 0], &[0, 0]);
        let model_2 = IntegerDiscreteModel::from_bytes(&bytes_2).unwrap();

        let obs0 = model_2.observe_target(&[0], 0).unwrap();
        assert_eq!(obs0.evaluation.prediction, 0);
        assert_eq!(obs0.evaluation.admitted_cells, vec![0, 17]);
        assert_eq!(obs0.evaluation.top8, vec![0, 1]);
        assert_eq!(obs0.token_rivals, 1);
        assert_eq!(obs0.address_rivals, 1);
        assert_eq!(obs0.selected_loss, 0);

        let obs1 = model_2.observe_target(&[1], 0).unwrap();
        assert_eq!(obs1.evaluation.prediction, 1);
        assert_eq!(obs1.evaluation.admitted_cells, vec![17, 0]);
        assert_eq!(obs1.evaluation.top8, vec![1, 0]);
        assert_eq!(obs1.token_rivals, 1);
        assert_eq!(obs1.address_rivals, 1);
        assert_eq!(obs1.selected_loss, 640);

        // Negative control: editing `discrete.table` (replacing token 0's endpoint block
        // with `NEG_A` and token 1's with `A`) changes the integer read vector, admitted cells,
        // and loss on the same prefix `[0]` and target `0`.
        let edited_bytes_2 =
            build_discrete_uorlut01(shape_2, &[[A, NEG_A], [NEG_A, A]], &[0, 0], &[0, 0]);
        let edited_model_2 = IntegerDiscreteModel::from_bytes(&edited_bytes_2).unwrap();
        let edited_obs0 = edited_model_2.observe_target(&[0], 0).unwrap();
        assert_ne!(
            edited_obs0.evaluation.read_vector,
            obs0.evaluation.read_vector
        );
        assert_eq!(edited_obs0.evaluation.admitted_cells, vec![1, 16]);
        assert_ne!(
            edited_obs0.evaluation.admitted_cells,
            obs0.evaluation.admitted_cells
        );

        // 2. Three-token context transport witness (`qualification.rs::context_transport`).
        let shape_3 = DiscreteLutShape {
            vocabulary: 3,
            blocks: 2,
            context: 3,
            kept_cells: 3,
            stages: 2,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            read_position: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let bytes_3 = build_discrete_uorlut01(
            shape_3,
            &[[A, A], [NEG_A, NEG_A], [B, B]],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
        );
        let model_3 = IntegerDiscreteModel::from_bytes(&bytes_3).unwrap();

        let eval_a = model_3.evaluate_prefix(&[0, 1, 2]).unwrap();
        let expected_a: Vec<i32> = [NEG_B, NEG_B].iter().flatten().copied().collect();
        assert_eq!(eval_a.read_vector, expected_a);

        let eval_b = model_3.evaluate_prefix(&[0, 0, 2]).unwrap();
        let expected_b: Vec<i32> = [B, B].iter().flatten().copied().collect();
        assert_eq!(eval_b.read_vector, expected_b);
    }

    #[test]
    fn flock_and_relation_copy_successor_match_cuda_qualification_witnesses() {
        // C2 copy-successor witness (`relation_qualification.rs::relation_copy_successor_fixture`).
        let tokens = [
            [B, A],
            [A, D_ROOT],
            [NEG_A, NEG_B],
            [C_ROOT, A],
            [D_ROOT, A],
        ];
        let prefix = [0u32, 1, 2, 3, 4, 1];

        let flock_shape = DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "flock".to_owned(),
            read_kept: 1,
            read_sweeps: 0,
            read_position: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "active_leaf_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let flock_bytes =
            build_discrete_uorlut01(flock_shape, &tokens, &[0, 0, 0, 0], &[0, 0, 1, 0]);
        let flock_model = IntegerDiscreteModel::from_bytes(&flock_bytes).unwrap();
        let flock_obs = flock_model.observe_target(&prefix, 2).unwrap();
        let expected_base_vec: Vec<i32> = [A, D_ROOT].iter().flatten().copied().collect();
        assert_eq!(flock_obs.evaluation.read_vector, expected_base_vec);
        assert_eq!(flock_obs.selected_loss, 640);

        let rel_shape = DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "relation".to_owned(),
            read_kept: 1,
            read_sweeps: 1,
            read_position: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "active_leaf_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let mut base_gates = [0i32; 14];
        base_gates[2] = 1;
        let rel_base_bytes =
            build_discrete_uorlut01(rel_shape.clone(), &tokens, &[0, 0, 0, 0], &base_gates);
        let rel_base_model = IntegerDiscreteModel::from_bytes(&rel_base_bytes).unwrap();
        let rel_base_obs = rel_base_model.observe_target(&prefix, 2).unwrap();
        assert_eq!(rel_base_obs.evaluation.read_vector, expected_base_vec);
        assert_eq!(rel_base_obs.selected_loss, 640);

        // Activate SequentialSuccessor at level 0 (`gates[6] = +1`): selects node 2 (`tok_B`),
        // transports to `[A, B]`, and reduces loss on target 2 from 640 to 448.
        let mut active_gates = base_gates;
        active_gates[6] = 1;
        let rel_active_bytes =
            build_discrete_uorlut01(rel_shape, &tokens, &[0, 0, 0, 0], &active_gates);
        let rel_active_model = IntegerDiscreteModel::from_bytes(&rel_active_bytes).unwrap();
        let rel_active_obs = rel_active_model.observe_target(&prefix, 2).unwrap();
        let expected_active_vec: Vec<i32> = [A, B].iter().flatten().copied().collect();
        assert_eq!(rel_active_obs.evaluation.read_vector, expected_active_vec);
        assert_eq!(rel_active_obs.selected_loss, 448);
        let scores = rel_active_obs
            .evaluation
            .relation_scores
            .as_ref()
            .expect("relation scores");
        let b1 = &scores[8..16];
        assert_eq!([b1[0], b1[1], b1[2], b1[4], b1[5]], [0, 0, 8, 0, 4]);
    }

    #[test]
    fn loader_and_evaluator_refuse_corrupted_containers_and_invalid_prefixes() {
        let tokens = [[A, A], [NEG_A, NEG_A]];
        let shape = DiscreteLutShape {
            vocabulary: 2,
            blocks: 2,
            context: 2,
            kept_cells: 2,
            stages: 1,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            read_position: 0,
            objective_policy: "occupied_cell_rank_hinge_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        };
        let valid = build_discrete_uorlut01(shape.clone(), &tokens, &[0, 0], &[0, 0]);
        let model = IntegerDiscreteModel::from_bytes(&valid).unwrap();
        assert!(model.evaluate_prefix(&[]).is_err());
        assert!(model.evaluate_prefix(&[0, 1, 0]).is_err());
        assert!(model.evaluate_prefix(&[2]).is_err());
        assert!(model.observe_target(&[0], 2).is_err());

        // Bad magic.
        let mut bad_magic = valid.clone();
        bad_magic[0] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&bad_magic).is_err());

        // Non-zero header padding.
        let header_len = u64::from_le_bytes(valid[8..16].try_into().unwrap()) as usize;
        let unaligned = 16 + header_len;
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        if unaligned < data_start {
            let mut bad_pad = valid.clone();
            bad_pad[unaligned] = 1;
            assert!(IntegerDiscreteModel::from_bytes(&bad_pad).is_err());
        }

        // Corrupted cell index in discrete.cells.
        let mut bad_cells = valid.clone();
        let header: DiscreteLutHeader =
            serde_json::from_slice(&valid[16..16 + header_len]).unwrap();
        let cells_offset = data_start + (header.tables[1].span.offset as usize);
        bad_cells[cells_offset] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&bad_cells).is_err());

        // Invalid stages in shape.
        let mut bad_shape = shape;
        bad_shape.stages = 2;
        let bad_stages = build_discrete_uorlut01(bad_shape, &tokens, &[0, 0], &[0, 0]);
        assert!(IntegerDiscreteModel::from_bytes(&bad_stages).is_err());
    }

    #[test]
    fn count_prediction_uses_the_last_committed_id_and_preserves_native_mass() {
        use crate::count_mixer::{CountMixer, VOCAB};
        let shape = DiscreteLutShape {
            vocabulary: VOCAB as u32,
            read_mode: "route".to_owned(),
            ..pick_shape(0)
        };
        let base = build_discrete_uorlut01(shape, &vec![[A, A]; VOCAB], &[0; 4], &[0; 4]);
        let mut model = IntegerDiscreteModel::from_bytes(&base).unwrap();
        let mut words = vec![0; 4 * (1 + VOCAB)];
        words[..4].copy_from_slice(&250u32.to_le_bytes());
        words[8..12].copy_from_slice(&10u32.to_le_bytes()); // Most frequent unigram is ID 1.
        for (previous, target) in [(1u16, 18u16), (18, 7)] {
            words.extend(previous.to_le_bytes());
            words.extend(target.to_le_bytes());
            words.extend(3u32.to_le_bytes());
        }
        let counts = CountMixer::from_words(&words).unwrap();
        let mass = counts.mass_q32(1, 18);
        model.count_mixer = Some(counts);
        let first = model.evaluate_prefix_traced(&[0, 1]).unwrap();
        assert_eq!(first.evaluation.prediction, 18);
        assert_eq!(first.evaluation.top8[0], 18);
        assert_eq!(first.evaluation.top8.len(), 8);
        assert!(first.evaluation.admitted_cells.is_empty());
        assert_eq!(
            first.trace_words,
            vec![1, 250, 18, mass as u32, (mass >> 32) as u32]
        );
        assert_eq!(model.evaluate_prefix(&[99, 1]).unwrap(), first.evaluation);
        assert_eq!(model.evaluate_prefix(&[0, 1, 18]).unwrap().prediction, 7);
        assert_eq!(model.evaluate_prefix(&[999]).unwrap().prediction, 1);
        assert!(model.observe_target(&[0, 1], 18).is_err());
        assert!(model.evaluate_prefix(&[VOCAB as u32, 1]).is_err());
    }

    #[test]
    fn trunk_d11_19_table_loader_evaluates_joint_pointer_and_rejects_planted_faults() {
        let shape = DiscreteLutShape {
            vocabulary: 3,
            blocks: 4,
            context: 4,
            kept_cells: 3,
            stages: 2,
            read_mode: "route".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            read_position: 0,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 1,
        };
        let tok0 = [A, A, A, A];
        let tok1 = [NEG_A, NEG_A, NEG_A, NEG_A];
        let tok2 = [B, B, B, B];
        let tokens = [tok0, tok1, tok2];
        let table: Vec<i32> = tokens.iter().flatten().flatten().copied().collect();
        let cells: Vec<u32> = tokens
            .iter()
            .map(|blocks| {
                let r0 = root_id(&blocks[0]).unwrap() as u32;
                let r3 = root_id(&blocks[3]).unwrap() as u32;
                (r0 << 4) + r3
            })
            .collect();
        let routes = [0u32; 8];
        let gates = [0i32; 8];

        let mut blob = Vec::new();
        let mut tables = Vec::new();
        let push_u32 = |blob: &mut Vec<u8>, name: &str, words: &[u32]| -> SectionSpec {
            let bytes: Vec<u8> = words.iter().flat_map(|x| x.to_le_bytes()).collect();
            SectionSpec {
                name: name.to_owned(),
                kind: SectionKind::U32,
                len: words.len(),
                span: push_section(blob, &bytes),
            }
        };
        let table_bytes: Vec<u8> = table.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.table".to_owned(),
            kind: SectionKind::I32,
            len: table.len(),
            span: push_section(&mut blob, &table_bytes),
        });
        tables.push(push_u32(&mut blob, "discrete.cells", &cells));
        tables.push(push_u32(&mut blob, "discrete.routes", &routes));
        let gates_bytes: Vec<u8> = gates.iter().flat_map(|x| x.to_le_bytes()).collect();
        tables.push(SectionSpec {
            name: "discrete.gates".to_owned(),
            kind: SectionKind::I32,
            len: gates.len(),
            span: push_section(&mut blob, &gates_bytes),
        });

        let step_words = [1u32; 64];
        tables.push(push_u32(
            &mut blob,
            "discrete.trunk.step_candidates",
            &step_words,
        ));
        let norm_words = [0u32; 4];
        tables.push(push_u32(
            &mut blob,
            "discrete.trunk.norm_candidates",
            &norm_words,
        ));
        let unigram_words = [0u32, 10, 5, 3];
        tables.push(push_u32(
            &mut blob,
            "discrete.trunk.unigram",
            &unigram_words,
        ));

        // Direct query_table for query_token = 0 toward token 2 (`[B, B, B, B]`) with count 5.
        for name in [
            "discrete.trunk.read_table",
            "discrete.trunk.rec_table",
            "discrete.trunk.read_rec_table",
            "discrete.trunk.rec_shuf_table",
            "discrete.trunk.read_rec_shuf_table",
            "discrete.trunk.rec_unit_table",
            "discrete.trunk.read_rec_unit_table",
            "discrete.trunk.query_table",
            "discrete.trunk.query_read_table",
            "discrete.trunk.query_rec_table",
            "discrete.trunk.query_rec_shuf_table",
            "discrete.trunk.query_rec_unit_table",
        ] {
            let words: &[u32] = if name == "discrete.trunk.query_table" {
                &[0, 2, 5]
            } else {
                &[]
            };
            tables.push(push_u32(&mut blob, name, words));
        }

        let header = DiscreteLutHeader {
            tokenizer_sha256: None,
            schema: DISCRETE_TRUNK_SCHEMA.to_owned(),
            shape,
            matrices: Vec::new(),
            tables,
            source: DiscreteLutSource {
                snapshot_schema: SNAPSHOT_SCHEMA.to_owned(),
                snapshot_sha256: "00".repeat(32),
                operator_kappa: OPERATOR_KAPPA.to_owned(),
                composition_kappa: COMPOSITION_KAPPA.to_owned(),
                kernel_sha256: "11".repeat(32),
                source_sha256: "22".repeat(32),
                geometry_sha256: "33".repeat(32),
            },
        };
        let json = serde_json::to_vec(&header).unwrap();
        let mut container = Vec::with_capacity(16 + json.len() + ALIGN + blob.len());
        container.extend_from_slice(MAGIC);
        container.extend_from_slice(&(json.len() as u64).to_le_bytes());
        container.extend_from_slice(&json);
        let pad = (ALIGN - container.len() % ALIGN) % ALIGN;
        container.resize(container.len() + pad, 0);
        container.extend_from_slice(&blob);

        let model = IntegerDiscreteModel::from_bytes(&container).unwrap();
        assert!(model.is_trunk_d11());
        let eval = model.evaluate_prefix(&[0]).unwrap();
        let expected_b4: Vec<i32> = tok2.iter().flatten().copied().collect();
        assert_eq!(eval.read_vector, expected_b4);
        assert_eq!(eval.prediction, 2);

        // The recurrence source sees only the same causal tape passed to the
        // trunk, even when the later assertion and query tokens are changed.
        let recurrence = contextual_source::SourceArm::WideRecurrence;
        let trunk = model.trunk.as_ref().unwrap();
        let code = trunk.compute_rec_code(&[0], &model.codebook_roots).unwrap();
        let expected: Vec<i32> = (0..4)
            .flat_map(|block| orbit_root_coords(usize::from((code >> (4 * block)) & 15)))
            .collect();
        let a = model
            .contextual_source_coordinates(&[0, 1, 2, 1], 1, Some(recurrence))
            .unwrap();
        let b = model
            .contextual_source_coordinates(&[0, 2, 0, 2], 1, Some(recurrence))
            .unwrap();
        assert_eq!(a.as_slice(), expected);
        assert_eq!(a, b);
        assert_eq!(
            model
                .contextual_source_coordinates(&[0, 1, 2, 1], 1, None)
                .unwrap()
                .as_slice(),
            expected_b4,
        );

        // Causal pointer successor `[0, 1, 2, 0]` points to successor token `1` (`ptr_boost = 256 > 5`),
        // rotating `q_blocks` to `tok1` (`[NEG_A, NEG_A, NEG_A, NEG_A]`) and predicting `1`.
        let eval_ptr = model.evaluate_prefix(&[0, 1, 2, 0]).unwrap();
        let expected_neg_a4: Vec<i32> = tok1.iter().flatten().copied().collect();
        assert_eq!(eval_ptr.read_vector, expected_neg_a4);
        assert_eq!(eval_ptr.prediction, 1);

        // Planted fault 1: trailing byte is rejected.
        let mut bad_trailing = container.clone();
        bad_trailing.push(0);
        assert!(IntegerDiscreteModel::from_bytes(&bad_trailing).is_err());

        // Planted fault 2: corrupted tail token in query_table (`2 -> 99 >= vocab`) is rejected.
        let header_len = u64::from_le_bytes(container[8..16].try_into().unwrap()) as usize;
        let unaligned = 16 + header_len;
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        let q_offset = data_start + (header.tables[14].span.offset as usize) + 4;
        let mut bad_tail = container;
        bad_tail[q_offset..q_offset + 4].copy_from_slice(&99u32.to_le_bytes());
        assert!(IntegerDiscreteModel::from_bytes(&bad_tail).is_err());
    }

    fn pick_shape(read_position: u32) -> DiscreteLutShape {
        DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "pick".to_owned(),
            read_kept: 0,
            read_sweeps: 0,
            read_position,
            objective_policy: "nearest_address_margin_v1".to_owned(),
            read_initialization: "zero_gates_v1".to_owned(),
            seed: 7,
            epoch: 0,
        }
    }

    /// Five tokens whose block-0 roots and block-1 roots are pairwise distinct.
    const PICK_TOKENS: [[[i32; 8]; 2]; 5] = [
        [A, B],
        [C_ROOT, NEG_B],
        [D_ROOT, A],
        [NEG_A, D_ROOT],
        [B, C_ROOT],
    ];

    /// Identity-transport Pick model with level-0 gates `[gate_b0, gate_b1]`.
    fn pick_model(gate_b0: i32, gate_b1: i32) -> IntegerDiscreteModel {
        let bytes = build_discrete_uorlut01(
            pick_shape(3),
            &PICK_TOKENS,
            &[0, 0, 0, 0],
            &[gate_b0, 0, gate_b1, 0],
        );
        IntegerDiscreteModel::from_bytes(&bytes).unwrap()
    }

    #[test]
    fn pick_code_xor_popcount_is_half_the_l1_root_distance() {
        for a in 0..16 {
            for b in 0..16 {
                let ra = ROOTS[a].map(i32::from);
                let rb = ROOTS[b].map(i32::from);
                let l1: u32 = (0..8).map(|d| (ra[d] - rb[d]).unsigned_abs()).sum();
                let xor = root_code(&ra) ^ root_code(&rb);
                assert_eq!(l1, 2 * xor.count_ones(), "roots {a} and {b}");
            }
        }
    }

    #[test]
    fn pick_copies_the_successor_of_the_one_earlier_query_pair() {
        // The query pair (tok 0, tok 1) at (4, 5) occurs earlier only at (0, 1).
        let prefix = [0u32, 1, 2, 3, 0, 1];
        let query = PICK_TOKENS[1];
        let successor = PICK_TOKENS[2];
        let expected: Vec<[i32; 8]> = (0..2)
            .map(|b| {
                let mut sum = query[b];
                for d in 0..8 {
                    sum[d] += successor[b][d];
                }
                ROOTS[snap(&sum, 2)].map(i32::from)
            })
            .collect();
        assert_ne!(expected, query.to_vec(), "fixture must move the read");

        let gated = pick_model(1, 1);
        let read = gated.pick_row(&prefix).unwrap();
        assert_eq!(read.pick, Some(1));
        assert_eq!(read.distance, Some(0));
        assert_eq!(read.blocks, expected);
        let served = gated.evaluate_prefix(&prefix).unwrap();
        let flat: Vec<i32> = expected.iter().flatten().copied().collect();
        assert_eq!(served.read_vector, flat);

        let open = pick_model(0, 0);
        let read = open.pick_row(&prefix).unwrap();
        assert_eq!(read.pick, Some(1));
        assert_eq!(read.blocks, query.to_vec());

        let short = gated.pick_row(&[0, 1]).unwrap();
        assert_eq!((short.pick, short.distance), (None, None));
        assert_eq!(short.blocks, query.to_vec());
    }

    #[test]
    fn pick_breaks_a_distance_tie_toward_the_latest_position() {
        // (tok 0, tok 1) at (0, 1) and (2, 3) both match the query pair at (4, 5).
        let read = pick_model(1, 1).pick_row(&[0, 1, 0, 1, 0, 1]).unwrap();
        assert_eq!(read.distance, Some(0));
        assert_eq!(read.pick, Some(3));
    }

    #[test]
    fn pick_refuses_legacy_and_matched_endpoint_transport() {
        for word in [0u32, 2] {
            let bytes =
                build_discrete_uorlut01(pick_shape(word), &PICK_TOKENS, &[0; 4], &[1, 0, 1, 0]);
            let error = IntegerDiscreteModel::from_bytes(&bytes)
                .unwrap_err()
                .to_string();
            assert!(error.contains("Pick read admits only"), "{word}: {error}");
        }
        let mut flock = pick_shape(3);
        flock.read_mode = "flock".to_owned();
        flock.read_kept = 1;
        let bytes = build_discrete_uorlut01(flock, &PICK_TOKENS, &[0; 4], &[0; 4]);
        let error = IntegerDiscreteModel::from_bytes(&bytes)
            .unwrap_err()
            .to_string();
        assert!(error.contains("served only for the Pick read"), "{error}");
    }

    #[test]
    fn pick_prefix_holonomy_with_identity_steps_equals_identity_transport() {
        for a in 0..64 {
            assert_eq!(
                compose(a, cl06_inverse(a)),
                usize::from(IDENTITY),
                "frame {a}"
            );
        }
        let token_blocks: Vec<Vec<[i32; 8]>> = PICK_TOKENS.iter().map(|t| t.to_vec()).collect();
        let prefix = [0u32, 1, 2, 3, 0, 1];
        let holonomy = PickTransport::PrefixHolonomy {
            frames: vec![IDENTITY; 32],
        };
        let by_holonomy = pick_read(&token_blocks, &[1, -1], &holonomy, &prefix).unwrap();
        let by_identity =
            pick_read(&token_blocks, &[1, -1], &PickTransport::Identity, &prefix).unwrap();
        assert_eq!(by_holonomy, by_identity);
        assert_eq!(by_identity.pick, Some(1));
    }
}
