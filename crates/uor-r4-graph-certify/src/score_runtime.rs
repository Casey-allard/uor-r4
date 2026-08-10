//! The Phase-4 reference scorer: the witness-replayable, integer-only
//! scoring model of the graph-compiler plan (§5 Phase 4, §6 runtime
//! contract, glossary "Scoring model") after the issue-#64 redesign.
//! Two rules, both scored per context:
//!
//! ```text
//! Rule 2 (D4 exact-context precedence): S(v) = B(v) + ΔX(X,v)
//!     when the EXCT probe resolves at the FULL graded code (level ==
//!     STAGES) with total evidence ≥ EXCT_SUPPORT_MIN — strict
//!     exact-context evidence dominates and graph residuals are skipped
//!     entirely (status ExactContext). An explicit supported NGRAM row
//!     takes the same most-specific precedence before the EXCT probe;
//!     since #380 the row hit carries the whole bounded row as the
//!     emission distribution (canonical argmax selection unchanged). A
//!     probe that resolves below full
//!     depth is prefix backoff, not exact context (#234, maintainer
//!     decision 2026-07-29): it is recorded in the witness but admits
//!     nothing, and the position falls through to Rule 1.
//! Rule 1 (chain-telescoped):   S_graph(v) = T_selected(v) + ΔT-offset
//!     with T_r(v) = B(v) + Σ_{n ∈ chain(r)} ΔE(n,v) over the covered
//!     refinement chain of the selected active region (status Graph, or
//!     Novel when no active region has a covered chain at all).
//! ```
//!
//! The redesign fixes the confirmed double counting of the literal
//! Σ-over-cloud formula `B + Σ_{n∈A} ΔE + Σ_{m∈F} ΔT + ΔX` (Gate C:
//! 0.3% vs 31.7% TLA3 baseline; one token took +91.7 nats from 13
//! sibling-subtree emission lists and won 98% of 2,000 probes): emission
//! residuals are applied only along ONE refinement chain per context, so
//! correlated residuals of sibling regions sharing a refinement parent
//! can no longer stack. The old formula is retained as
//! [`GraphScorer::score_candidates_legacy`] for the Gate C side-by-side
//! comparison only; it is not the shipping semantics.
//!
//! Every contribution is a [`ScoreQ`] (Q16.16 log-domain integer); the
//! accumulation core — candidate union, contribution application, the
//! canonical argmax — uses ScoreQ saturating add/sub, integer compares,
//! and table reads only: no float, no multiply, no divide, no modulo.
//! Legacy TLS1 compatibility keeps one compiler-side float helper delimited
//! by `BEGIN/END COMPILER-SIDE FLOAT` markers and documented at
//! [`quantize_exct`]. The deployed RX1 path never calls it; the machine-
//! checked source scan (`tests/score.rs`) asserts the accumulation core
//! carries no `f32`/`f64` and no `*` `/` `%` value arithmetic.
//!
//! # Scoring semantics (normative)
//!
//! - **Active cloud A**: the top-`TOP_M` masked-Hamming memberships at
//!   each depth 1..=max_depth — exactly the
//!   [`super::cover::ReferenceClassifier::binary_memberships`] semantics
//!   (top-M by distance, ties to the lower region id, within-radius
//!   filter, nearest-region fallback), recomputed here over the region
//!   parameters recovered from the artifact's NODE/ROUT sections so the
//!   artifact alone is the scoring authority.
//! - **Covered chain of an active region r** (Rule 1): r's refinement
//!   ancestor path from the depth-1 region down to r, truncated at the
//!   first node whose calibrated radius does not cover the context
//!   signature — the [`binary_top1_covered`] radius semantics applied
//!   per node (within-radius masked Hamming; the nearest-region fallback
//!   is a routing behavior and never makes a node covered). The chain is
//!   the contiguous covered prefix root → … → deepest covered ancestor
//!   of r, so the telescoped sum `B + Σ ΔE` composes the refinement
//!   corrections instead of stacking siblings (each chain node enters at
//!   most once per candidate — Theorem 10 by construction).
//! - **Cloud combination**: the selected region is the active region
//!   whose covered chain is DEEPEST (the graph analog of the baseline's
//!   deepest-populated-class rule); ties break by higher membership
//!   margin (`radius − distance` of the active region itself), then by
//!   the lowest region id. Only the selected chain's emission lists
//!   apply ΔE.
//! - **Predicted cloud F**: the union of E_f edge targets of the active
//!   regions. Each predicted region m keeps its single best incoming
//!   edge (highest `score_q`, ties to the lowest canonical edge id).
//!   Under Rule 1 the token-independent edge-weight part is folded into
//!   the scalar `transition_offset = Σ_m w(n_m→m)` added to every
//!   candidate once (the witness records the applied edges and the
//!   offset); F emission lists generate candidates but contribute no
//!   residuals.
//! - **Candidates**: under Rule 1, the union of the emission lists of A ∪ F
//!   ∪ chain plus the root prior's top-`root_top_b` tokens. Under Rule 2,
//!   only the EXCT probe's admitted local entries are scored; mixing global
//!   root candidates into that local distribution would change the graded-
//!   store argmax. Every scored candidate receives one root-prior contribution
//!   before its residual, with the baked smoothing floor for absent tokens.
//! - **EXCT precedence (Rule 2)**: the existing prefix probe from
//!   `runtime::predict_witness_plain` (deepest populated prefix). A
//!   resolution at the FULL graded code (level == STAGES, #234) with total
//!   evidence ≥ [`EXCT_SUPPORT_MIN`] ⇒ `S(v) = B(v) + ΔX(X,v)` over the
//!   admitted local entries, graph candidate generation skipped; a
//!   below-full-depth resolution or a total below the gate is recorded
//!   (admitted 0) and Rule 1 decides.
//! - **Status**: every prediction reports exactly one [`ScoreStatus`] —
//!   `ExactContext` (Rule 2 fired), `Graph` (Rule 1 with a non-empty
//!   selected chain), `Novel` (Rule 1 with no covered chain — Phase 5
//!   consumes this).
//! - **Selection**: the canonical tie-break — highest score, then the
//!   lowest token id (matches `runtime::predict_witness_plain`'s rule).
//!
//! # Theorem 10 by construction
//!
//! Contribution IDs are canonical: `RootPrior(token)`, `Emission(node,
//! token)`, `ExactContext(token)`, and one applied-edge id per predicted
//! region. Chain nodes are distinct refinement ancestors of one region,
//! so a node's emission list enters S(v) at most once per context — even
//! when the node is simultaneously active (ΔE) and predicted (ΔT), the
//! A ∪ F ∪ chain candidate union is deduplicated by node before any
//! emission is read, and only the selected chain applies. The induced
//! cover has no explicit overlap nodes, so there are no interaction
//! residuals this phase: the root-plus-residual decomposition attaches
//! every contribution to exactly one node. The witness verifier
//! independently rejects duplicate contribution IDs (belt-and-braces,
//! Theorem 10).
//!
//! # Witness and independent replay (Theorem 6)
//!
//! [`GraphScorer::score_candidates`] emits a bounded [`ScoreWitness`]:
//! graph CID, input code, the status (which rule fired), the selected
//! covered chain (empty under ExactContext/Novel), active regions +
//! margins, predicted cloud, applied transition edges + folded offset,
//! the EXCT probe record (when consulted), the selected token with its
//! full contribution list and score, the candidate count, and the op
//! census. [`verify_witness_replay`] rebuilds a fresh scorer **from the
//! validated R4G1 bytes** (plus the teacher TLA container when EXCT
//! evidence is present — checked against HEAD `teacher_cid`, so the
//! class-code derivation chains to pinned content), recomputes the
//! entire prediction without any compiler state, and requires bit-exact
//! equality of every witness field; duplicate contribution IDs are
//! rejected out of hand.
//!
//! # Op census
//!
//! Hamming membership runs through [`OpKernel`] (xor + popcount table +
//! add per byte), and emission/root/EXCT application, the candidate
//! scan, and the argmax are counted as adds / compares / table reads /
//! candidate scans. Bounded lookup structures (the root-prior map and
//! per-node emission maps) are built once at scorer construction — the
//! reference form of the deployed runtime's fixed tables; per-token
//! scoring itself performs no allocation-dependent work beyond the
//! candidate accumulation map (the deployed Phase-5 runtime replaces it
//! with fixed-capacity arrays).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use uor_r4_graph_format::{GraphView, ScoreQ, SectionId};

use uor_r4_core::transformerless::compiler::{self, Compiled, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::runtime::{self, OpKernel, Store};

/// Edge kind of refinement (parent/child) edges (cover/transitions).
pub const EDGE_KIND_REFINEMENT: u8 = 0;
/// Edge kind of lateral neighbor (co-activation) edges.
pub const EDGE_KIND_NEIGHBOR: u8 = 1;
/// Edge kind of forward transition edges (E_f).
pub const EDGE_KIND_FORWARD: u8 = 2;

/// Bounded multi-membership per depth (matches `cover::TOP_M` and the
/// runtime's top-M).
pub const TOP_M: usize = 3;

/// The shipped repetition-suppression penalty in raw ScoreQ units
/// (~-30.5 nats). Issue #381 made the magnitude sweepable at the
/// certifier level after the windowed Gate C measurement showed it costs
/// ~10pp top-1 on the fixture distribution.
pub const DEFAULT_REPETITION_PENALTY_RAW: i32 = -2_000_000;

/// EXCT section body marker for compile-time integer residual tables.
/// The four-byte storage descriptor still prefixes this body on disk.
pub const RESIDUAL_EXCT_MAGIC: [u8; 4] = *b"RX1\0";

/// Rule 2 (D4 EXCT precedence) support gate: the exact-context evidence
/// dominates the graph residuals only when the probe resolves at the
/// FULL graded code (#234 maintainer decision, 2026-07-29) AND its
/// total evidence count reaches this bound; otherwise the probe is
/// recorded (admitted 0) and the chain-telescoped Rule 1 score decides.
/// The value 5 matches the baseline's confident-prefix regime: a
/// context with fewer observations is too thin to outrank compressed
/// graph residuals (issue #64; the gate is witness-recorded, so the
/// threshold is part of the replayed semantics).
pub const EXCT_SUPPORT_MIN: u32 = 5;

/// Byte length of one EMIT entry: `(i32 token, i32 score_q)`.
pub const EMIT_ENTRY_BYTES: usize = 8;
/// Byte length of the EMIT root-prior block header:
/// `(u32 entry_count, u32 total_count, i32 floor_score_q, u32 reserved)`.
pub const EMIT_ROOT_HEADER_BYTES: usize = 16;

/// The region parameters the scoring path reads from the artifact:
/// node identity, multiresolution depth, calibrated radius, and the
/// packed sign-bit prototype. The f32 prototypes stay compiler-side;
/// membership here is the shipped binary (masked-Hamming) semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionParams {
    /// Artifact node id (region id = node − 1; the root is node 0).
    pub node: u32,
    /// Multiresolution depth (1..=max_depth).
    pub depth: u8,
    /// Calibrated acceptance radius (masked-Hamming bound).
    pub radius: u16,
    /// Binarized prototype signature.
    pub sig: [u8; SIG_BYTES],
    /// Parent region id (`None` at depth 1 — the parent is the root).
    pub parent: Option<u32>,
}

impl RegionParams {
    /// Region id of this node (index into the region vector).
    pub fn region_id(&self) -> u32 {
        self.node - 1
    }
}

/// Region parameters recovered from a validated artifact view: one entry
/// per region (node 1..=node_count−1), ascending node id. The parent of
/// a region is the source of its refinement edge (the root maps to
/// `None`); a region without a refinement parent maps to `None` as well
/// (every emitted region has exactly one).
///
/// Total: `None` when the view — already parse- and CID-validated by the
/// caller — lacks a section the recovery needs (HEAD, ROUT), declares zero
/// nodes, is missing a node record inside its declared count, or points a
/// prototype window outside ROUT. Every such case means the bytes are not a
/// scorer's graph (R5, #510).
pub fn regions_from_view(view: &GraphView) -> Option<Vec<RegionParams>> {
    let head = view.head()?;
    let rout = view.section(SectionId::ROUT)?;
    let node_count = head.node_count();
    if node_count == 0 {
        return None;
    }
    // Parents from refinement edges (dst node -> src node).
    let mut parent_of: BTreeMap<u32, u32> = BTreeMap::new();
    for edge in view.edges() {
        if edge.kind == EDGE_KIND_REFINEMENT {
            parent_of.insert(edge.dst.0, edge.src.0);
        }
    }
    let signature_bytes = head.signature_bytes() as usize;
    let mut regions = Vec::with_capacity((node_count - 1) as usize);
    let mut node_index = 1u32;
    while node_index < node_count {
        let node = view.node(node_index)?;
        let start = (node.prototype_word_start as usize) << 3;
        let window = rout.get(start..start + signature_bytes)?;
        let mut sig = [0u8; SIG_BYTES];
        sig.copy_from_slice(window);
        let parent =
            parent_of
                .get(&node_index)
                .and_then(|&src| if src == 0 { None } else { Some(src - 1) });
        regions.push(RegionParams {
            node: node_index,
            depth: node.depth.0,
            radius: node.radius.0,
            sig,
            parent,
        });
        node_index += 1;
    }
    Some(regions)
}

/// One structural edge recovered from the canonical array (E_r/E_o with
/// their stored scores — E_f is recompiled, not carried over).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralEdge {
    pub src: u32,
    pub kind: u8,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// The non-forward edges of a validated artifact view, in canonical
/// edge-id order.
pub fn structural_edges_from_view(view: &GraphView) -> Vec<StructuralEdge> {
    view.edges()
        .filter(|edge| edge.kind != EDGE_KIND_FORWARD)
        .map(|edge| StructuralEdge {
            src: edge.src.0,
            kind: edge.kind,
            dst: edge.dst.0,
            score_q: edge.score_q,
        })
        .collect()
}

/// Kernel-counted Hamming distance between two signatures.
fn hamming_counted(
    k: &mut OpKernel,
    pop: &[u8; 256],
    a: &[u8; SIG_BYTES],
    b: &[u8; SIG_BYTES],
) -> u32 {
    let mut dist = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let xored = k.xor(x, y);
        let ones = k.table_u8(pop, xored);
        dist = k.add(dist as i64, ones as i64) as u32;
    }
    dist
}

/// The shipped binary membership at `depth` — the
/// `ReferenceClassifier::binary_memberships` semantics over recovered
/// region parameters: scan regions in ascending node-id order keeping
/// the top-[`TOP_M`] by masked-Hamming distance (strict `<` insertion,
/// so ties go to the lower id), filter to those within their calibrated
/// radius, and fall back to the nearest region when nothing is in
/// range. Returns `(region_id, distance)` pairs in distance order.
pub fn binary_memberships(
    k: &mut OpKernel,
    pop: &[u8; 256],
    regions: &[RegionParams],
    depth: usize,
    sig: &[u8; SIG_BYTES],
) -> Vec<(u32, u32)> {
    let mut top: Vec<(u32, u32)> = Vec::with_capacity(TOP_M);
    for region in regions {
        if region.depth as usize != depth {
            continue;
        }
        let dist = hamming_counted(k, pop, sig, &region.sig);
        let mut inserted = false;
        for (idx, &(_, d0)) in top.iter().enumerate() {
            k.compares += 1;
            if dist < d0 {
                top.insert(idx, (region.region_id(), dist));
                inserted = true;
                break;
            }
        }
        if !inserted && top.len() < TOP_M {
            top.push((region.region_id(), dist));
        }
        if inserted && top.len() > TOP_M {
            top.pop();
        }
    }
    if top.is_empty() {
        return Vec::new();
    }
    let within: Vec<(u32, u32)> = top
        .iter()
        .filter(|&&(id, dist)| {
            k.compares += 1;
            dist <= u32::from(regions[id as usize].radius)
        })
        .map(|&(id, dist)| (id, dist))
        .collect();
    if within.is_empty() {
        // Nearest-region fallback (the backoff floor).
        vec![top[0]]
    } else {
        within
    }
}

/// Covered binary top-1 membership at `depth`: the nearest region when
/// — and only when — the context signature lies within its calibrated
/// radius. This is the *evidence* membership (region content), distinct
/// from the routing membership above: the nearest-region fallback is a
/// routing behavior, not region content. Without this rule, the backoff
/// floor would assign every observation to some region at every depth
/// and deep regions' distributions would collect the whole corpus.
pub fn binary_top1_covered(
    k: &mut OpKernel,
    pop: &[u8; 256],
    regions: &[RegionParams],
    depth: usize,
    sig: &[u8; SIG_BYTES],
) -> Option<(u32, u32)> {
    let (region, dist) = binary_memberships(k, pop, regions, depth, sig)
        .into_iter()
        .next()?;
    k.compares += 1;
    if dist <= u32::from(regions[region as usize].radius) {
        Some((region, dist))
    } else {
        None
    }
}

/// The covered refinement chain of `region` (Rule 1, module docs): the
/// region's ancestor path from the depth-1 region down to `region`,
/// truncated at the first node whose calibrated radius does not cover
/// the context signature — the [`binary_top1_covered`] radius semantics
/// per node (the nearest-region fallback never makes a node covered).
/// Returns the contiguous covered prefix as node ids in root-to-leaf
/// order; the root itself is implicit via B(v). An empty vector means no
/// covered chain exists for this region (the depth-1 ancestor is already
/// out of range). A malformed parent cycle terminates deterministically
/// at the region count.
fn covered_chain(
    k: &mut OpKernel,
    pop: &[u8; 256],
    regions: &[RegionParams],
    region: u32,
    sig: &[u8; SIG_BYTES],
) -> Vec<u32> {
    let mut path: Vec<u32> = Vec::new();
    let mut current = Some(region);
    while let Some(id) = current {
        if path.len() >= regions.len() {
            break;
        }
        path.push(id);
        current = regions[id as usize].parent;
    }
    let mut chain = Vec::with_capacity(path.len());
    for &id in path.iter().rev() {
        let dist = hamming_counted(k, pop, sig, &regions[id as usize].sig);
        k.compares += 1;
        if dist <= u32::from(regions[id as usize].radius) {
            chain.push(id + 1); // region id -> node id
        } else {
            break;
        }
    }
    chain
}

// BEGIN COMPILER-SIDE FLOAT QUANTIZATION ---------------------------------
// The one floating-point site of the reference scoring path (macOS-pinned
// libm, exactly the status of the existing κ baseline; the D2 canonical
// deterministic compile mode resolves cross-platform byte equality later).
// The deployed integer runtime never quantizes at probe time: residualized
// EXCT tables (the Phase-5 EXCT migration) bake these values at compile
// time. This helper is kept out of every accumulation function so the
// S(v) integer core stays machine-checkable by source scan.

/// ΔX probe-time quantization: `ScoreQ::from_logprob(ln P(v|X))` with
/// add-one smoothing over the compiled vocabulary,
/// `P(v|X) = (count + 1) / (total + V)`. The residual form used by the
/// scorer subtracts the stored root prior from this value with integer
/// saturating subtraction.
fn quantize_exct(count: u32, total: u32, vocab: u32) -> ScoreQ {
    let p = (f64::from(count) + 1.0) / (f64::from(total) + f64::from(vocab));
    ScoreQ::from_logprob(p.ln() as f32)
}

/// FWDA load-time quantization (issue #399): the forward-anchor section
/// stores raw counts plus the row total, and the residual
/// `ln((count + 0.5) / (total + vocab * 0.5))` is baked into ScoreQ once
/// at scorer construction — the measured Gate C fusion law's smoothing
/// (`fuse_forward_arm` in `score.rs`). Like [`quantize_exct`] this
/// helper is kept out of every accumulation function; the per-token
/// serving path reads only the pre-quantized integers.
fn quantize_fwda(count: u32, total: u32, vocab: u32) -> ScoreQ {
    let p = (f64::from(count) + 0.5) / (f64::from(total) + f64::from(vocab) * 0.5);
    ScoreQ::from_logprob(p.ln() as f32)
}

// END COMPILER-SIDE FLOAT QUANTIZATION -----------------------------------

/// Integer multiply for the `#80` candidate scoring variants, built only
/// from shift/add/compare (P-4: the accumulation core may not use a
/// literal `*` operator). Binary shift-and-add long multiplication,
/// magnitude-only with the sign folded back in at the end.
fn shift_mul_i128(a: i128, b: i128) -> i128 {
    let negative = (a < 0) != (b < 0);
    let mut multiplicand = a.unsigned_abs();
    let mut multiplier = b.unsigned_abs();
    let mut result: u128 = 0;
    while multiplier > 0 {
        if multiplier & 1 == 1 {
            result = result.saturating_add(multiplicand);
        }
        multiplier >>= 1;
        if multiplier > 0 {
            // Doubling `multiplicand` would silently wrap (rather than
            // panic) if its top bit is already set, so detect that case
            // up front and saturate instead of shifting into it.
            if multiplicand > u128::MAX >> 1 {
                result = u128::MAX;
                break;
            }
            multiplicand <<= 1;
        }
    }
    // Saturate before the signed cast: `result` may exceed `i128::MAX`
    // for extreme inputs, and casting an out-of-range `u128` to `i128`
    // would otherwise silently wrap.
    let magnitude = result.min(i128::MAX as u128) as i128;
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// Integer divide (truncating toward zero, like `/`) for the `#80`
/// candidate scoring variants, built only from shift/subtract/compare
/// (P-4: the accumulation core may not use a literal `/` operator).
/// Binary shift-and-subtract restoring long division, magnitude-only
/// with the sign folded back in at the end. Divide-by-zero returns 0
/// (the call sites already clamp divisors to a minimum of 1).
fn shift_div_i128(dividend: i128, divisor: i128) -> i128 {
    if divisor == 0 {
        return 0;
    }
    let negative = (dividend < 0) != (divisor < 0);
    let mut remainder = dividend.unsigned_abs();
    let d = divisor.unsigned_abs();
    if remainder < d {
        return 0;
    }
    let mut quotient: u128 = 0;
    let mut shift = (128 - remainder.leading_zeros()) - (128 - d.leading_zeros());
    loop {
        let shifted = d << shift;
        if shifted <= remainder {
            remainder -= shifted;
            quotient |= 1u128 << shift;
        }
        if shift == 0 {
            break;
        }
        shift -= 1;
    }
    let magnitude = quotient as i128;
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// Canonical identity of one score contribution (Theorem 10): no id may
/// appear twice in one candidate's contribution list. The transition
/// edge-weight contributions are folded into the scalar offset and
/// witnessed once per applied edge (see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContributionId {
    /// Root prior B(v) — exactly one per candidate.
    RootPrior { token: u32 },
    /// Emission residual ΔE(node, v) — at most one per node per context.
    Emission { node: u32, token: u32 },
    /// Exact-context residual ΔX(X, v) — at most one per context.
    ExactContext { token: u32 },
}

/// One contribution to a candidate's score: a canonical id and the
/// ScoreQ value applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contribution {
    pub id: ContributionId,
    pub value: ScoreQ,
}

/// One active region of the witness: region identity, depth, masked-
/// Hamming distance to the context signature, and the membership margin
/// (`radius − distance`; negative only on the nearest-region fallback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveRegion {
    pub region: u32,
    pub depth: u8,
    pub distance: u16,
    pub margin: i16,
}

/// One applied transition edge of the witness: the best incoming E_f
/// edge of one predicted region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeUse {
    pub edge_id: u32,
    pub src: u32,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// The EXCT probe record of the witness: the deepest populated graded
/// prefix, its evidence total, and the number of probe tokens admitted
/// to the candidate set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExctProbe {
    pub level: u8,
    pub key: Vec<u8>,
    pub total: u32,
    pub admitted: u32,
}

/// Which scoring rule produced the prediction (module docs; Phase 5
/// consumes this status per decision D4). Recorded in every witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreStatus {
    /// Rule 2 fired: an explicit NGRAM row matched, or the EXCT probe
    /// resolved at the FULL graded code (#234) with total evidence ≥
    /// [`EXCT_SUPPORT_MIN`];
    /// `S(v) = B(v) + ΔX(X,v)`, graph residuals skipped entirely.
    ExactContext,
    /// Rule 1 fired with a non-empty selected covered chain:
    /// `S(v) = B(v) + Σ_{chain} ΔE + ΔT-offset`.
    Graph,
    /// Rule 1 fired but no active region has a covered chain (every
    /// membership is the nearest-region fallback floor): the score is
    /// the root prior plus the folded transition offset, and the context
    /// is outside every calibrated radius.
    Novel,
}

/// The bounded, replayable record of one prediction (glossary
/// "Witness"; plan §24, Theorem 6). Everything the independent verifier
/// needs is recomputed from the validated artifact bytes; the witness
/// carries only claims.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreWitness {
    /// `artifact_cid` of the scored R4G1 container.
    pub graph_cid: [u8; 32],
    /// H(x): the context's sign-bit signature.
    pub input_sig: [u8; SIG_BYTES],
    /// The graded code the EXCT probe consumed (#243 Phase C, option A —
    /// maintainer decision 2026-07-30). Attested by the producer and
    /// verified-as-consumed on replay: under a sign-metric (TLA3/4/5)
    /// artifact it is re-derivable from `input_sig`; under TLA6 it
    /// derives from the integer bundle, which the witness does not
    /// carry — code DERIVATION is proven at certify time by the
    /// kernel==plain equality witnesses, and replay proves consistent
    /// CONSUMPTION. `None` when no artifact/EXCT surface is wired.
    pub input_code: Option<[u8; STAGES]>,
    /// Recent lexical tokens used by the explicit NGRAM backoff, retained so
    /// independent replay can reproduce the same context decision.
    pub context_tokens: Vec<u32>,
    /// Which rule fired (module docs).
    pub status: ScoreStatus,
    /// Active cloud A (ascending region id) with margins; empty under
    /// Rule 2 (graph candidate generation skipped).
    pub active: Vec<ActiveRegion>,
    /// The selected covered chain (node ids, root-to-leaf order); empty
    /// under Rule 2 and under `Novel`.
    pub chain: Vec<u32>,
    /// Predicted cloud F (ascending node id).
    pub predicted: Vec<u32>,
    /// Applied transition edges (ascending edge id).
    pub edges_applied: Vec<EdgeUse>,
    /// Σ of the applied edge weights (the folded ΔT edge part).
    pub transition_offset: ScoreQ,
    /// Whether predicted-cloud (ΔT) emissions were enabled for this
    /// witness. Replay must apply the same flag to recompute faithfully.
    pub f_emissions: bool,
    /// The EXCT probe, when exact-context evidence was consulted.
    pub exct: Option<ExctProbe>,
    /// The selected token and its score (canonical tie-break).
    pub selected: u32,
    pub selected_score: ScoreQ,
    /// The selected token's full contribution list, canonical order.
    pub selected_contributions: Vec<Contribution>,
    /// Size of the scored candidate set.
    pub candidate_count: u32,
    /// The op census of this prediction.
    pub census: OpKernel,
}

/// Which mechanism resolved an `ExactContext` selection (#362/#364
/// attribution). Since the NGRAM context rows landed, an explicit
/// bigram/trigram row hit and an EXCT full-depth probe resolution both
/// report [`ScoreStatus::ExactContext`]; measurements that compare the
/// exact-context slice across eras need to know which mechanism answered.
/// `None` on outcomes whose status is not `ExactContext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactContextSource {
    /// An explicit NGRAM (bigram/trigram) context row supplied the token.
    NgramRow,
    /// The EXCT probe resolved at the full graded code with sufficient
    /// support.
    ExctProbe,
}

/// The outcome of [`GraphScorer::score_candidates`]: the selection plus
/// every candidate's final score (ascending token order) — the certifier
/// reads the distribution for bits/token; the witness stands alone.
#[derive(Debug, Clone)]
pub struct ScoreOutcome {
    pub selected: u32,
    pub selected_score: ScoreQ,
    /// Every candidate `(token, score)` in ascending token order.
    pub candidates: Vec<(u32, ScoreQ)>,
    /// Per-candidate score decomposition, ascending token order:
    /// `(token, root_raw, residual_with_offset_raw, repetition_penalized)`.
    /// Measurement only — lets the certifier ask whether the graph residual
    /// changes any decision, or whether ranking is carried by the root prior.
    pub candidate_components: Vec<(u32, i32, i32, bool)>,
    pub witness: ScoreWitness,
    /// Attribution for `ExactContext` selections (`None` otherwise). Not
    /// part of the replayable witness: provenance is measurement metadata,
    /// so the witness bytes and kappa are untouched.
    pub exact_context_source: Option<ExactContextSource>,
}

/// The outcome of [`GraphScorer::score_candidates_legacy`]: the
/// selection plus every candidate's final score (ascending token order).
/// The old formula is comparison-only — it is not the shipping semantics
/// and emits no replayable witness.
#[derive(Debug, Clone)]
pub struct LegacyOutcome {
    pub selected: u32,
    pub selected_score: ScoreQ,
    /// Every candidate `(token, score)` in ascending token order.
    pub candidates: Vec<(u32, ScoreQ)>,
}

#[derive(Debug, Clone)]
struct ResidualExctContext {
    total: u32,
    entries: BTreeMap<u32, ScoreQ>,
}

fn parse_residual_exct(body: &[u8]) -> Option<Vec<BTreeMap<Vec<u8>, ResidualExctContext>>> {
    if body.len() < 8 || body[..4] != RESIDUAL_EXCT_MAGIC {
        return None;
    }
    let levels = body[4] as usize;
    if levels != STAGES + 1 || body[5..8] != [0u8; 3] {
        return None;
    }
    let mut offset = 8usize;
    let mut parsed = Vec::with_capacity(levels);
    for level in 0..levels {
        let key_count = u32::from_le_bytes(body.get(offset..offset + 4)?.try_into().ok()?) as usize;
        offset += 4;
        let mut contexts = BTreeMap::new();
        for _ in 0..key_count {
            let key_len = usize::from(*body.get(offset)?);
            offset += 1;
            if key_len != level {
                return None;
            }
            let key = body.get(offset..offset + key_len)?.to_vec();
            offset += key_len;
            let total = u32::from_le_bytes(body.get(offset..offset + 4)?.try_into().ok()?);
            offset += 4;
            let entry_count =
                u32::from_le_bytes(body.get(offset..offset + 4)?.try_into().ok()?) as usize;
            offset += 4;
            let mut entries = BTreeMap::new();
            for _ in 0..entry_count {
                let token = u32::from_le_bytes(body.get(offset..offset + 4)?.try_into().ok()?);
                let residual =
                    i32::from_le_bytes(body.get(offset + 4..offset + 8)?.try_into().ok()?);
                offset += 8;
                entries.insert(token, ScoreQ::from_raw(residual));
            }
            contexts.insert(key, ResidualExctContext { total, entries });
        }
        parsed.push(contexts);
    }
    (offset == body.len()).then_some(parsed)
}

/// The reference scorer: validated R4G1 bytes (+ the teacher TLA
/// container when EXCT evidence is wired) parsed once into bounded
/// lookup structures. Construction fails closed — invalid bytes or CIDs
/// never yield a scorer.
/// Candidate scoring variant (issue #80).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoringVariant {
    /// Chain-telescoped scoring (default HEAD behavior: unweighted sum of chain region residuals).
    #[default]
    ChainTelescoped,
    /// Cloud-size normalized scoring (residual sum divided by chain length).
    CloudSizeNormalized,
    /// Margin-weighted residual stacking (residual scaled by normalized membership margin).
    MarginWeighted,
}

/// One loaded forward-anchor row (issue #399): pre-quantized ScoreQ
/// residuals under the measured product-fusion law, plus the row's
/// absent-token floor residual (the zero-count smoothing value under the
/// same denominator). Both are derived once at load time from the FWDA
/// section's raw counts; the serving path is integer-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardAnchorServingRow {
    /// Residual applied to tokens absent from the row's entries.
    pub absent_floor: ScoreQ,
    /// Bounded `(token, residual)` entries, ascending token order.
    pub entries: Vec<(u32, ScoreQ)>,
}

pub struct GraphScorer {
    graph_cid: [u8; 32],
    regions: Vec<RegionParams>,
    max_depth: usize,
    /// Forward edges grouped by source node, ascending edge id within.
    forward_by_src: BTreeMap<u32, Vec<EdgeUse>>,
    root_prior: BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
    root_top: Vec<u32>,
    /// Per-region emission maps (index = region id), token → ΔE.
    emissions: Vec<BTreeMap<u32, ScoreQ>>,
    /// Explicit bigram/trigram rows. The root prior remains the unigram
    /// fallback in EMIT.
    context_rows: BTreeMap<(u8, u32, u32), Vec<(u32, ScoreQ)>>,
    /// Forward-anchor rows keyed `(lookahead distance, anchor token)`,
    /// loaded from the optional FWDA section (empty when absent).
    fwd_rows: BTreeMap<(u8, u32), ForwardAnchorServingRow>,
    residual_exct: Option<Vec<BTreeMap<Vec<u8>, ResidualExctContext>>>,
    store: Option<Store>,
    artifacts: Option<Compiled>,
    vocab: u32,
    exct_top_x: usize,
    pop: [u8; 256],
    /// When false, the predicted cloud F is empty and ΔT contributes
    /// nothing. Default FALSE since the #66 ablation: the folded
    /// transition offset is argmax-neutral by construction (a uniform
    /// bias can only distort the distribution — Rule 1 bits/token
    /// 71.52 → 17.73 with ΔT off) and the EXCT-precedence path is
    /// F-invariant. Re-enable only with a measured per-token ΔT design.
    f_emissions: bool,
    scoring_variant: ScoringVariant,
    repetition_penalty_raw: i32,
    fallback_policy: uor_r4_core::transformerless::resolution_status::FallbackPolicy,
}

impl GraphScorer {
    /// Build the certifier-side low-rank far-field candidate for this graph.
    /// This deliberately does not alter deployed scoring semantics.
    ///
    /// Total: `None` when the low-rank factorization cannot be built from the
    /// graph parts (the sole path [`crate::fmm::FmmCandidateScorer::from_graph_parts`]
    /// already reports as `None`) — R5, #510.
    pub fn fmm_candidate(
        &self,
        config: crate::fmm::FmmConfig,
    ) -> Option<crate::fmm::FmmCandidateScorer> {
        crate::fmm::FmmCandidateScorer::from_graph_parts(
            &self.regions,
            &self.emissions,
            &self.root_prior,
            self.root_floor,
            self.vocab,
            config,
        )
    }

    /// Enable or disable predicted-cloud (F / ΔT) emissions. Default
    /// disabled since the #66 ablation decision; re-enabling requires a
    /// measured per-token ΔT design (the folded offset is argmax-neutral).
    pub fn set_f_emissions(&mut self, enabled: bool) {
        self.f_emissions = enabled;
    }

    /// Set the candidate scoring variant (issue #80).
    pub fn set_scoring_variant(&mut self, variant: ScoringVariant) {
        self.scoring_variant = variant;
    }

    /// Set the repetition-penalty magnitude (issue #381; raw ScoreQ units,
    /// must be <= 0). The shipped default is
    /// [`DEFAULT_REPETITION_PENALTY_RAW`]; the knob exists so the penalty's
    /// measured ~10pp top-1 cost on windowed evaluation can be swept rather
    /// than assumed. Serving keeps the default unless a maintainer decision
    /// changes it.
    pub fn set_repetition_penalty_raw(&mut self, raw: i32) {
        self.repetition_penalty_raw = raw.min(0);
    }

    /// The active candidate scoring variant.
    pub fn scoring_variant(&self) -> ScoringVariant {
        self.scoring_variant
    }

    /// The artifact-declared D4 fallback policy.
    pub fn fallback_policy(
        &self,
    ) -> &uor_r4_core::transformerless::resolution_status::FallbackPolicy {
        &self.fallback_policy
    }
}

impl GraphScorer {
    /// Build a scorer from validated R4G1 bytes. `teacher_container` is
    /// the TLA artifact the EXCT class codes derive from; when the
    /// artifact carries an EXCT section, supplying it enables
    /// exact-context evidence (its blake3 must equal HEAD `teacher_cid`
    /// — fail closed, so the evidence chains to the pinned teacher),
    /// and a `None` teacher runs the same artifact without EXCT.
    ///
    /// Total: `None` when the bytes are not a scorer — they fail two-stage
    /// parse (`NotAProduct`) or CID integrity (`KappaError`), lack a section
    /// the scorer needs, or carry a teacher container that does not match the
    /// pinned `teacher_cid`. The two sanctioned failure reasons are collapsed
    /// into `None` here because every caller either rebuilds from
    /// self-produced bytes (a `None` is then a defect it panics on) or only
    /// needs the yes/no answer (R5, #510).
    pub fn from_artifact(
        r4g1: &[u8],
        teacher_container: Option<&[u8]>,
        root_top_b: usize,
        exct_top_x: usize,
    ) -> Option<Self> {
        let view = GraphView::parse(r4g1).ok()?;
        view.verify_cids().ok()?;
        let head = view.head()?;
        let fallback_policy =
            uor_r4_core::transformerless::resolution_status::FallbackPolicy::from_bytes(
                head.fallback_policies(),
            );
        let graph_cid = view.header().artifact_cid.0;
        let regions = regions_from_view(&view)?;
        let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(0);

        let mut forward_by_src: BTreeMap<u32, Vec<EdgeUse>> = BTreeMap::new();
        for (edge_id, edge) in view.edges().enumerate() {
            if edge.kind == EDGE_KIND_FORWARD {
                forward_by_src.entry(edge.src.0).or_default().push(EdgeUse {
                    edge_id: edge_id as u32,
                    src: edge.src.0,
                    dst: edge.dst.0,
                    score_q: edge.score_q,
                });
            }
        }

        // EMIT: descriptor (stage-2 validated), the root-prior block,
        // and per-node emission lists wired by the packed ranges.
        let emit = view.section(SectionId::EMIT)?;
        let remainder = emit.get(uor_r4_graph_format::STORAGE_DESCRIPTOR_LEN..)?;
        let header = remainder.get(..EMIT_ROOT_HEADER_BYTES)?;
        let root_entry_count = u32::from_le_bytes(header[0..4].try_into().expect("4 bytes"));
        let root_floor = ScoreQ::from_raw(i32::from_le_bytes(
            header[8..12].try_into().expect("4 bytes"),
        ));
        let root_entry_bytes = (root_entry_count as usize) << 3;
        let root_block =
            remainder.get(EMIT_ROOT_HEADER_BYTES..EMIT_ROOT_HEADER_BYTES + root_entry_bytes)?;
        let mut root_prior = BTreeMap::new();
        for entry in root_block.chunks_exact(EMIT_ENTRY_BYTES) {
            let token = i32::from_le_bytes(entry[0..4].try_into().expect("4 bytes"));
            let value = i32::from_le_bytes(entry[4..8].try_into().expect("4 bytes"));
            let token = u32::try_from(token).ok()?;
            root_prior.insert(token, ScoreQ::from_raw(value));
        }
        // Top-B root candidates by (score desc, token asc) — context-
        // independent, precomputed once (module docs).
        let mut ranked: Vec<(u32, ScoreQ)> = root_prior.iter().map(|(&t, &s)| (t, s)).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let root_top: Vec<u32> = ranked
            .into_iter()
            .take(root_top_b)
            .map(|(t, _)| t)
            .collect();

        let node_count = head.node_count();
        let mut emissions = vec![BTreeMap::new(); regions.len()];
        let mut node_index = 1u32;
        while node_index < node_count {
            let node = view.node(node_index)?;
            let start = node.emission_start as usize;
            let byte_len = (node.emission_len as usize) << 3;
            let list = remainder.get(start..start + byte_len)?;
            let map = &mut emissions[(node_index - 1) as usize];
            for entry in list.chunks_exact(EMIT_ENTRY_BYTES) {
                let token = i32::from_le_bytes(entry[0..4].try_into().expect("4 bytes"));
                let value = i32::from_le_bytes(entry[4..8].try_into().expect("4 bytes"));
                let token = u32::try_from(token).ok()?;
                map.insert(token, ScoreQ::from_raw(value));
            }
            node_index += 1;
        }

        let mut context_rows = BTreeMap::new();
        if let Some(table) = view.ngram_table().ok()? {
            for row in table.rows() {
                let entries: Vec<(u32, ScoreQ)> = row
                    .entries()
                    .map(|entry| (entry.token, entry.score_q))
                    .collect();
                if !entries.is_empty() {
                    let key = (row.context_len(), row.key().0, row.key().1);
                    context_rows.insert(key, entries);
                }
            }
        }

        // FWDA (issue #399): the optional forward-anchor section stores
        // raw counts plus each row's full total; residuals are quantized
        // here, once, so the per-token infill fusion stays integer-only.
        let mut fwd_rows = BTreeMap::new();
        if let Some(table) = view.fwda_table().ok()? {
            let vocab = head.vocab_size();
            for row in table.rows() {
                let total = row.total();
                let entries: Vec<(u32, ScoreQ)> = row
                    .entries()
                    .map(|entry| (entry.token, quantize_fwda(entry.count, total, vocab)))
                    .collect();
                fwd_rows.insert(
                    (row.distance(), row.anchor()),
                    ForwardAnchorServingRow {
                        absent_floor: quantize_fwda(0, total, vocab),
                        entries,
                    },
                );
            }
        }

        // EXCT: prefer compile-time residual tables. They can be used by
        // the deployed integer-only runtime; raw TLS1 remains accepted for
        // legacy Gate C/converter artifacts and uses the old certifier-side
        // quantization path.
        let exct = view.section(SectionId::EXCT);
        let (residual_exct, store, artifacts) = match exct {
            Some(bytes) => {
                let body = bytes.get(uor_r4_graph_format::STORAGE_DESCRIPTOR_LEN..)?;
                if let Some(residuals) = parse_residual_exct(body) {
                    let artifacts = match teacher_container {
                        Some(teacher) => {
                            if blake3::hash(teacher).as_bytes() != &head.teacher_cid().0 {
                                return None;
                            }
                            Some(compiler::parse_artifacts(teacher)?)
                        }
                        None => None,
                    };
                    (Some(residuals), None, artifacts)
                } else {
                    // EXCT present but not residualized RX1: it must be a raw
                    // TLS1 store paired with the pinned teacher. A missing
                    // teacher container means these bytes are not a scorer.
                    let teacher = teacher_container?;
                    if blake3::hash(teacher).as_bytes() != &head.teacher_cid().0 {
                        return None;
                    }
                    let parsed = compiler::parse_artifacts(teacher)?;
                    #[allow(deprecated)]
                    let store = runtime::parse_store(body)
                        .or_else(|| runtime::parse_store_legacy_u16(body))?;
                    (None, Some(store), Some(parsed))
                }
            }
            None => (None, None, None),
        };

        Some(GraphScorer {
            graph_cid,
            regions,
            max_depth,
            forward_by_src,
            root_prior,
            root_floor,
            root_top,
            emissions,
            context_rows,
            fwd_rows,
            residual_exct,
            store,
            artifacts,
            vocab: head.vocab_size(),
            exct_top_x,
            pop: runtime::derive_popcount_table(),
            f_emissions: false,
            scoring_variant: ScoringVariant::ChainTelescoped,
            repetition_penalty_raw: DEFAULT_REPETITION_PENALTY_RAW,
            fallback_policy,
        })
    }

    /// True when exact-context evidence is wired.
    pub fn has_exct(&self) -> bool {
        self.residual_exct.is_some() || self.store.is_some()
    }

    /// The compiled vocabulary size (HEAD).
    pub fn vocab(&self) -> u32 {
        self.vocab
    }

    /// The root-prior smoothing floor (baked into EMIT).
    pub fn root_floor(&self) -> ScoreQ {
        self.root_floor
    }

    /// The stored root prior B(v), or the smoothing floor for tokens
    /// absent from the root block.
    /// Number of emission nodes (regions + 1 offset). Measurement-only.
    pub fn emission_node_count(&self) -> usize {
        self.emissions.len()
    }

    /// Is `token` in the global root prior's top-B list? Measurement-only.
    pub fn root_top_contains(&self, token: u32) -> bool {
        self.root_top.contains(&token)
    }

    /// Does `node` (1-based) emit `token`? Measurement-only accessor for
    /// asking which regions could have supplied a token, independent of which
    /// chain was selected.
    pub fn node_emits(&self, node: u32, token: u32) -> bool {
        self.emissions
            .get((node - 1) as usize)
            .is_some_and(|emissions| emissions.contains_key(&token))
    }

    fn root_score(&self, token: u32) -> ScoreQ {
        self.root_prior
            .get(&token)
            .copied()
            .unwrap_or(self.root_floor)
    }

    /// Apply the explicit lexical backoff rule and return the winning row's
    /// full entry list (ascending token order, as compiled). A row is
    /// supported only when it has at least one candidate; empty rows fall
    /// through to the next context order and finally to the normal
    /// root/graph scorer. Since #380 the whole bounded row is the emission
    /// distribution on a hit — the argmax alone made row hits
    /// single-candidate and collapsed bits/token on the exact-context slice.
    fn context_row(&self, recent_tokens: &[u32]) -> Option<&[(u32, ScoreQ)]> {
        let tokens = if recent_tokens.first().is_some_and(|&token| token <= 1) {
            &recent_tokens[1..]
        } else {
            recent_tokens
        };
        let keys = if tokens.len() >= 2 {
            [
                Some((2, tokens[tokens.len() - 2], tokens[tokens.len() - 1])),
                tokens.last().copied().map(|token| (1, token, 0)),
            ]
        } else {
            [None, tokens.last().copied().map(|token| (1, token, 0))]
        };
        for key in keys.into_iter().flatten() {
            if let Some(entries) = self.context_rows.get(&key) {
                if !entries.is_empty() {
                    return Some(entries);
                }
            }
        }
        None
    }

    /// The canonical row argmax: highest score, then lowest token id —
    /// byte-identical to the pre-#380 selection, so served tokens are
    /// unchanged by the distribution change.
    fn context_row_argmax(entries: &[(u32, ScoreQ)]) -> (u32, ScoreQ) {
        let mut best = entries[0];
        for &(token, score) in &entries[1..] {
            if score.raw() > best.1.raw() || (score.raw() == best.1.raw() && token < best.0) {
                best = (token, score);
            }
        }
        best
    }

    /// Score one context signature: compute the active cloud A, the
    /// predicted cloud F, the bounded candidate set, and S(v) for every
    /// candidate by integer ScoreQ accumulation; select by the canonical
    /// tie-break; emit the bounded witness. The arithmetic of this
    /// function is integer-only. Legacy raw TLS1 EXCT has a delimited
    /// certifier-side float fallback; deployed RX1 EXCT is already quantized.
    pub fn score_candidates(
        &self,
        sig: &[u8; SIG_BYTES],
        recent_tokens: &[u32],
    ) -> Option<ScoreOutcome> {
        self.score_candidates_coded(sig, None, recent_tokens)
    }

    /// Resolve the graded code the EXCT probe consumes (#243 Phase C,
    /// option A): an attested caller code wins; a sign-metric artifact
    /// derives it from the sig (that IS its declared metric); a TLA6
    /// artifact without a caller code is a contract error — the sig
    /// cannot reconstruct a dot-metric code.
    fn probe_code(
        art: &Compiled,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; STAGES]>,
    ) -> Option<[u8; STAGES]> {
        match input_code {
            Some(code) => Some(*code),
            None if art.dot_cb.is_empty() => Some(runtime::assign_plain(art, sig)),
            // TLA6 artifact without a caller code: the sig cannot reconstruct
            // the dot-metric graded code (witness contract, #243 Phase C A).
            None => None,
        }
    }

    /// [`GraphScorer::score_candidates`] with an explicitly attested
    /// graded code (#243 Phase C option A): bundle-holding callers
    /// (serving, harness) assign under the artifact's declared metric
    /// and pass the code through; replay passes the witnessed code.
    pub fn score_candidates_coded(
        &self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; STAGES]>,
        recent_tokens: &[u32],
    ) -> Option<ScoreOutcome> {
        let recent_tokens = if recent_tokens.len() > 32 {
            &recent_tokens[recent_tokens.len() - 32..]
        } else {
            recent_tokens
        };
        if let Some(entries) = self.context_row(recent_tokens) {
            // #380: the whole bounded row is the emission distribution; the
            // selection stays the canonical argmax, so served tokens are
            // byte-identical to the pre-#380 behavior — only the
            // distribution (bits/token, downstream sampling) changes.
            let (selected, selected_score) = Self::context_row_argmax(entries);
            let candidates: Vec<(u32, ScoreQ)> = entries.to_vec();
            let candidate_components: Vec<(u32, i32, i32, bool)> = entries
                .iter()
                .map(|&(token, score)| (token, score.raw(), 0, false))
                .collect();
            let census = OpKernel {
                table_reads: 1,
                ..Default::default()
            };
            let context_tokens = recent_tokens.to_vec();
            let witness = ScoreWitness {
                graph_cid: self.graph_cid,
                input_sig: *sig,
                input_code: input_code.copied(),
                context_tokens,
                status: ScoreStatus::ExactContext,
                active: Vec::new(),
                chain: Vec::new(),
                predicted: Vec::new(),
                edges_applied: Vec::new(),
                transition_offset: ScoreQ::ZERO,
                f_emissions: self.f_emissions,
                exct: None,
                selected,
                selected_score,
                selected_contributions: Vec::new(),
                candidate_count: candidates.len() as u32,
                census,
            };
            return Some(ScoreOutcome {
                selected,
                selected_score,
                candidates,
                candidate_components,
                witness,
                exact_context_source: Some(ExactContextSource::NgramRow),
            });
        }
        let mut k = OpKernel::default();

        // Active cloud A: top-M memberships at each depth — exactly the
        // ReferenceClassifier semantics, nearest-region fallback included.
        // A fallback member is routing evidence, not region content: the
        // region itself is uncovered, so it never enters a chain, but its
        // covered ancestors still compose the deepest covered prefix below.
        let mut active: Vec<ActiveRegion> = Vec::new();
        for depth in 1..=self.max_depth {
            for (region, dist) in binary_memberships(&mut k, &self.pop, &self.regions, depth, sig) {
                let radius = u32::from(self.regions[region as usize].radius);
                let margin = radius as i64 - dist as i64;
                active.push(ActiveRegion {
                    region,
                    depth: depth as u8,
                    distance: dist as u16,
                    margin: margin.clamp(i16::MIN as i64, i16::MAX as i64) as i16,
                });
            }
        }
        active.sort_by_key(|a| a.region);

        // Select one deepest covered refinement chain. Applying every active
        // region's residual stacks correlated sibling distributions and was
        // the source of the pathological high-confidence gibberish output.
        let mut selected_chain = Vec::new();
        let mut selected_chain_region = u32::MAX;
        let mut selected_chain_margin = i16::MIN;
        for member in &active {
            let chain = covered_chain(&mut k, &self.pop, &self.regions, member.region, sig);
            let better = chain.len() > selected_chain.len()
                || (chain.len() == selected_chain.len()
                    && (member.margin > selected_chain_margin
                        || (member.margin == selected_chain_margin
                            && member.region < selected_chain_region)));
            if better {
                selected_chain = chain;
                selected_chain_region = member.region;
                selected_chain_margin = member.margin;
            }
        }

        // Predicted cloud F: union of E_f targets of the active regions,
        // keeping the single best incoming edge per predicted region
        // (highest score_q, ties to the lowest canonical edge id).
        // Disabled entirely in the F-ablation measurement variant (#66).
        let active_nodes: BTreeSet<u32> = active.iter().map(|a| a.region + 1).collect();
        let (mut predicted, mut edges_applied, mut transition_offset) = if self.f_emissions {
            let mut best_edge: BTreeMap<u32, EdgeUse> = BTreeMap::new();
            for &node in &active_nodes {
                if let Some(edges) = self.forward_by_src.get(&node) {
                    for edge in edges {
                        k.table_reads += 1;
                        let better = match best_edge.get(&edge.dst) {
                            None => true,
                            Some(current) => {
                                k.compares += 1;
                                (edge.score_q, std::cmp::Reverse(edge.edge_id))
                                    > (current.score_q, std::cmp::Reverse(current.edge_id))
                            }
                        };
                        if better {
                            best_edge.insert(edge.dst, *edge);
                        }
                    }
                }
            }
            let predicted: Vec<u32> = best_edge.keys().copied().collect();
            let edges_applied: Vec<EdgeUse> = best_edge.values().copied().collect();
            let mut transition_offset = ScoreQ::ZERO;
            for edge in &edges_applied {
                transition_offset = transition_offset.saturating_add(edge.score_q);
                k.adds += 1;
            }
            (predicted, edges_applied, transition_offset)
        } else {
            (Vec::new(), Vec::new(), ScoreQ::ZERO)
        };

        // Candidate accumulation: active and predicted nodes contribute
        // candidate tokens, but only the selected covered chain contributes
        // residual scores. This keeps sibling emissions from stacking while
        // preserving their tokens as bounded candidates.
        let mut candidates: BTreeMap<u32, (ScoreQ, Vec<Contribution>)> = BTreeMap::new();
        let mut contributing: BTreeSet<u32> = active_nodes.clone();
        for &node in &predicted {
            contributing.insert(node);
        }
        contributing.extend(selected_chain.iter().copied());
        let chain_nodes: BTreeSet<u32> = selected_chain.iter().copied().collect();
        for &node in &contributing {
            let emissions = &self.emissions[(node - 1) as usize];
            for (&token, &value) in emissions {
                k.candidate_scans += 1;
                k.table_reads += 1;
                let entry = candidates
                    .entry(token)
                    .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
                if entry.1.is_empty() {
                    entry.1.push(Contribution {
                        id: ContributionId::RootPrior { token },
                        value: self.root_score(token),
                    });
                }
                if chain_nodes.contains(&node) {
                    let effective_val = match self.scoring_variant {
                        ScoringVariant::ChainTelescoped => value,
                        ScoringVariant::CloudSizeNormalized => {
                            // #80 candidate variant: residual sum divided by
                            // chain length, via the shift/subtract divider.
                            let n = (selected_chain.len().max(1)) as i128;
                            ScoreQ::from_raw(shift_div_i128(i128::from(value.raw()), n) as i32)
                        }
                        ScoringVariant::MarginWeighted => {
                            // #80 candidate variant: residual scaled by
                            // normalized membership margin, via the
                            // shift/add multiplier and shift/subtract
                            // divider (saturating the final cast).
                            let margin = active
                                .iter()
                                .find(|a| a.region + 1 == node)
                                .map(|a| a.margin.max(0) as i128)
                                .unwrap_or(1);
                            let radius = active
                                .iter()
                                .find(|a| a.region + 1 == node)
                                .map(|a| u32::from(self.regions[a.region as usize].radius) as i128)
                                .unwrap_or(1)
                                .max(1);
                            let scaled = shift_div_i128(
                                shift_mul_i128(i128::from(value.raw()), margin),
                                radius,
                            );
                            ScoreQ::from_raw(
                                scaled.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32,
                            )
                        }
                    };
                    entry.0 = entry.0.saturating_add(effective_val);
                    k.adds += 1;
                    entry.1.push(Contribution {
                        id: ContributionId::Emission { node, token },
                        value: effective_val,
                    });
                }
            }
        }
        // Root prior top-B tokens join the candidate set.
        for &token in &self.root_top {
            k.table_reads += 1;
            let entry = candidates
                .entry(token)
                .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
            if entry.1.is_empty() {
                entry.1.push(Contribution {
                    id: ContributionId::RootPrior { token },
                    value: self.root_score(token),
                });
            }
        }

        // EXCT: the existing prefix probe (deepest populated graded
        // prefix) over the exact-context store; ΔX(X,v) = quantized
        // probe log-prob minus the stored root prior (integer sub).
        let mut exct_probe: Option<ExctProbe> = None;
        let mut witness_code: Option<[u8; STAGES]> = None;
        let mut exact_residuals: Vec<(u32, ScoreQ)> = Vec::new();
        if let Some(art) = &self.artifacts {
            let code = Self::probe_code(art, sig, input_code)?;
            witness_code = Some(code);
            if let Some(residual_exct) = &self.residual_exct {
                for level in (0..=STAGES).rev() {
                    if let Some(context) = residual_exct[level].get(&code[..level]) {
                        // #234 maintainer decision (2026-07-29): Rule 2
                        // fires at FULL CODE ONLY. A probe that resolves
                        // below level STAGES is prefix backoff, not exact
                        // context; it is recorded in the witness but never
                        // admits evidence or preempts the graph.
                        let supported = level == STAGES && context.total >= EXCT_SUPPORT_MIN;
                        let admitted = if supported {
                            context.entries.len().min(self.exct_top_x) as u32
                        } else {
                            0
                        };
                        if supported {
                            exact_residuals.extend(
                                context.entries.iter().take(self.exct_top_x).map(
                                    |(&token, &residual)| {
                                        k.table_reads += 1;
                                        (token, residual)
                                    },
                                ),
                            );
                        }
                        exct_probe = Some(ExctProbe {
                            level: level as u8,
                            key: code[..level].to_vec(),
                            total: context.total,
                            admitted,
                        });
                        break;
                    }
                }
            } else if let Some(store) = &self.store {
                for level in (0..=STAGES).rev() {
                    if let Some(dist) = store[level].get(&code[..level]) {
                        let total: u32 = dist.values().sum();
                        let mut ranked: Vec<(u32, u32)> =
                            dist.iter().map(|(&t, &c)| (t, c)).collect();
                        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        // #234: full code only (see the RX1 branch above).
                        let supported = level == STAGES && total >= EXCT_SUPPORT_MIN;
                        let mut admitted = 0u32;
                        if supported {
                            for &(token, count) in ranked.iter().take(self.exct_top_x) {
                                k.table_reads += 1;
                                let value = quantize_exct(count, total, self.vocab)
                                    .saturating_sub(self.root_score(token));
                                exact_residuals.push((token, value));
                                admitted += 1;
                            }
                        }
                        exct_probe = Some(ExctProbe {
                            level: level as u8,
                            key: code[..level].to_vec(),
                            total,
                            admitted,
                        });
                        break;
                    }
                }
            }
        }

        // D4 EXCT precedence: once the exact prefix has enough evidence,
        // discard the graph and global-root candidate sets and rank only the
        // admitted local entries. Mixing global root candidates into a local
        // distribution lets a globally common token beat the locally dominant
        // exact-context token, which diverges from the graded-store semantics.
        let exact_context = exct_probe
            .as_ref()
            .is_some_and(|probe| probe.admitted > 0 && !exact_residuals.is_empty());
        if exact_context {
            candidates.clear();
            active.clear();
            selected_chain.clear();
            predicted.clear();
            edges_applied.clear();
            transition_offset = ScoreQ::ZERO;
            for (token, value) in exact_residuals {
                let entry = candidates
                    .entry(token)
                    .or_insert_with(|| (ScoreQ::ZERO, Vec::new()));
                if entry.1.is_empty() {
                    entry.1.push(Contribution {
                        id: ContributionId::RootPrior { token },
                        value: self.root_score(token),
                    });
                }
                entry.0 = entry.0.saturating_add(value);
                entry.1.push(Contribution {
                    id: ContributionId::ExactContext { token },
                    value,
                });
            }
        }

        // Final per-candidate scores: apply the baked root prior and the
        // folded token-independent transition edge weight (module docs:
        // ΔT(m,v) = w_m + ΔE(m,v)); canonicalize contribution order.
        // Also apply bounded integer repetition control via recent_tokens.
        let mut ranked_candidates: Vec<(u32, ScoreQ, Vec<Contribution>)> =
            Vec::with_capacity(candidates.len());
        let mut components: Vec<(u32, i32, i32, bool)> = Vec::with_capacity(candidates.len());
        for (token, (residual, mut contributions)) in candidates {
            let with_offset = residual.saturating_add(transition_offset);
            k.adds += 1;
            let base = self.root_score(token);
            let mut score = base.saturating_add(with_offset);
            k.adds += 1;
            if recent_tokens.contains(&token) {
                // Suppression penalty for repetition control (#381 knob).
                score = score.saturating_add(ScoreQ::from_raw(self.repetition_penalty_raw));
                k.adds += 1;
            }
            contributions.sort_by_key(|c| c.id);
            components.push((
                token,
                base.raw(),
                with_offset.raw(),
                recent_tokens.contains(&token),
            ));
            ranked_candidates.push((token, score, contributions));
        }
        if ranked_candidates.is_empty() {
            return None;
        }

        // Canonical argmax: highest score, then the lowest token id
        // (ascending-token iteration with a strict `>` keep).
        let mut best_index = 0usize;
        for (index, &(_, score, _)) in ranked_candidates.iter().enumerate() {
            k.candidate_scans += 1;
            k.compares += 1;
            if score > ranked_candidates[best_index].1 {
                best_index = index;
            }
        }
        let (selected, selected_score, _) = ranked_candidates[best_index];
        let selected_contributions = ranked_candidates[best_index].2.clone();
        let candidate_count = ranked_candidates.len() as u32;
        let candidates_out: Vec<(u32, ScoreQ)> = ranked_candidates
            .into_iter()
            .map(|(token, score, _)| (token, score))
            .collect();

        let witness = ScoreWitness {
            graph_cid: self.graph_cid,
            input_sig: *sig,
            input_code: witness_code,
            context_tokens: recent_tokens.to_vec(),
            status: if exact_context {
                ScoreStatus::ExactContext
            } else if selected_chain.is_empty() {
                ScoreStatus::Novel
            } else {
                ScoreStatus::Graph
            },
            active,
            chain: selected_chain,
            predicted,
            edges_applied,
            transition_offset,
            f_emissions: self.f_emissions,
            exct: exct_probe,
            selected,
            selected_score,
            selected_contributions,
            candidate_count,
            census: k,
        };
        Some(ScoreOutcome {
            selected,
            selected_score,
            candidates: candidates_out,
            candidate_components: components,
            witness,
            exact_context_source: exact_context.then_some(ExactContextSource::ExctProbe),
        })
    }

    /// The loaded forward-anchor row for `(distance, anchor)`, when the
    /// artifact carries a FWDA section with that key. Measurement and
    /// test accessor; serving goes through
    /// [`GraphScorer::score_candidates_infill`].
    pub fn forward_anchor_row(
        &self,
        distance: u8,
        anchor: u32,
    ) -> Option<&ForwardAnchorServingRow> {
        self.fwd_rows.get(&(distance, anchor))
    }

    /// Number of loaded forward-anchor rows (zero when the FWDA section
    /// is absent).
    pub fn forward_anchor_row_count(&self) -> usize {
        self.fwd_rows.len()
    }

    /// Infill serving entry point (issue #399): score the position with
    /// [`GraphScorer::score_candidates_coded`], then — when the caller
    /// knows the NEXT anchor token and its lookahead distance, and the
    /// artifact carries a forward-anchor row for that key — fuse the
    /// forward-anchor channel by the measured product law in the
    /// log domain: every candidate's ScoreQ gets a saturating add of its
    /// row residual (or the row's absent floor), and row tokens outside
    /// the base candidate set enter at the offset-shifted root floor
    /// (`root_floor + transition_offset`, the same accounting rule the
    /// Gate C `fuse_forward_arm` instrumentation measured) plus their
    /// residual. Selection is re-run under the canonical rule (highest
    /// score, ties to the lowest token). The fusion itself is
    /// integer-only; the residuals were quantized once at load time.
    ///
    /// A `None` anchor, a missing row, or an artifact without a FWDA
    /// section returns the base outcome unchanged — byte-identical to
    /// `score_candidates_coded`.
    ///
    /// Witness semantics: infill fusion is post-witness serving-side
    /// re-ranking. The returned witness (and `candidate_components`)
    /// keep the BASE outcome's contents, so independent replay covers
    /// the base prediction; only `selected`, `selected_score`, and
    /// `candidates` reflect the fused distribution.
    pub fn score_candidates_infill(
        &self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; STAGES]>,
        recent_tokens: &[u32],
        next_anchor: Option<(u32, u8)>,
    ) -> Option<ScoreOutcome> {
        let base = self.score_candidates_coded(sig, input_code, recent_tokens)?;
        let Some((anchor, distance)) = next_anchor else {
            return Some(base);
        };
        let Some(row) = self.fwd_rows.get(&(distance, anchor)) else {
            return Some(base);
        };
        let floor_base = self
            .root_floor
            .saturating_add(base.witness.transition_offset);
        let mut fused: BTreeMap<u32, ScoreQ> = BTreeMap::new();
        for &(token, score) in &base.candidates {
            let residual = match row.entries.binary_search_by_key(&token, |&(t, _)| t) {
                Ok(index) => row.entries[index].1,
                Err(_) => row.absent_floor,
            };
            fused.insert(token, score.saturating_add(residual));
        }
        for &(token, residual) in &row.entries {
            fused
                .entry(token)
                .or_insert_with(|| floor_base.saturating_add(residual));
        }
        let mut best: Option<(u32, ScoreQ)> = None;
        for (&token, &score) in &fused {
            if best.is_none_or(|(_, current)| score.raw() > current.raw()) {
                best = Some((token, score));
            }
        }
        let Some((selected, selected_score)) = best else {
            return Some(base);
        };
        let mut outcome = base;
        outcome.selected = selected;
        outcome.selected_score = selected_score;
        outcome.candidates = fused.into_iter().collect();
        Some(outcome)
    }

    /// The pre-#64 Σ-over-cloud formula `S(v) = B(v) + Σ_{n∈A} ΔE(n,v) +
    /// Σ_{m∈F} ΔT(m,v) + ΔX(X,v)` — retained ONLY as the Gate C
    /// comparison column (the correlated-sibling stacking it exhibits is
    /// what the chain rule fixes; module docs). Not the shipping
    /// semantics; emits no replayable witness. The active cloud uses the
    /// same ReferenceClassifier semantics the old formula scored
    /// (nearest-region fallback included).
    pub fn score_candidates_legacy(&self, sig: &[u8; SIG_BYTES]) -> Option<LegacyOutcome> {
        let mut k = OpKernel::default();

        // Active cloud A: top-M memberships at each depth.
        let mut active: Vec<u32> = Vec::new();
        for depth in 1..=self.max_depth {
            for (region, _) in binary_memberships(&mut k, &self.pop, &self.regions, depth, sig) {
                active.push(region);
            }
        }

        // Predicted cloud F: union of E_f targets of the active regions,
        // keeping the single best incoming edge per predicted region
        // (highest score_q, ties to the lowest canonical edge id).
        let mut best_edge: BTreeMap<u32, EdgeUse> = BTreeMap::new();
        for &region in &active {
            if let Some(edges) = self.forward_by_src.get(&(region + 1)) {
                for edge in edges {
                    let better = match best_edge.get(&edge.dst) {
                        None => true,
                        Some(current) => {
                            (edge.score_q, std::cmp::Reverse(edge.edge_id))
                                > (current.score_q, std::cmp::Reverse(current.edge_id))
                        }
                    };
                    if better {
                        best_edge.insert(edge.dst, *edge);
                    }
                }
            }
        }
        let mut transition_offset = ScoreQ::ZERO;
        for edge in best_edge.values() {
            transition_offset = transition_offset.saturating_add(edge.score_q);
        }

        // Candidate accumulation: every node of A ∪ F applies its
        // emission list — the correlated sibling stacking the redesign
        // removes lives HERE (all subtrees of one refinement parent
        // contribute at once).
        let mut candidates: BTreeMap<u32, ScoreQ> = BTreeMap::new();
        let mut contributing: BTreeSet<u32> = active.iter().map(|&r| r + 1).collect();
        contributing.extend(best_edge.keys().copied());
        for &node in &contributing {
            let emissions = &self.emissions[(node - 1) as usize];
            for (&token, &value) in emissions {
                let entry = candidates.entry(token).or_insert(ScoreQ::ZERO);
                *entry = entry.saturating_add(value);
            }
        }
        // Root prior top-B tokens join the candidate set.
        for &token in &self.root_top {
            candidates.entry(token).or_insert(ScoreQ::ZERO);
        }

        // EXCT: the existing prefix probe (deepest populated graded
        // prefix) over the exact-context evidence, applied additively
        // with no support gate — the stacking the precedence rule
        // replaces. ΔX(X,v) = quantized probe log-prob minus the stored
        // root prior (the RX1 table carries exactly this residual).
        if let Some(art) = &self.artifacts {
            let code = runtime::assign_plain(art, sig);
            if let Some(residual_exct) = &self.residual_exct {
                for level in (0..=STAGES).rev() {
                    if let Some(context) = residual_exct[level].get(&code[..level]) {
                        for (&token, &value) in context.entries.iter().take(self.exct_top_x) {
                            let entry = candidates.entry(token).or_insert(ScoreQ::ZERO);
                            *entry = entry.saturating_add(value);
                        }
                        break;
                    }
                }
            } else if let Some(store) = &self.store {
                for level in (0..=STAGES).rev() {
                    if let Some(dist) = store[level].get(&code[..level]) {
                        let total: u32 = dist.values().sum();
                        let mut ranked: Vec<(u32, u32)> =
                            dist.iter().map(|(&t, &c)| (t, c)).collect();
                        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        for &(token, count) in ranked.iter().take(self.exct_top_x) {
                            let value = quantize_exct(count, total, self.vocab)
                                .saturating_sub(self.root_score(token));
                            let entry = candidates.entry(token).or_insert(ScoreQ::ZERO);
                            *entry = entry.saturating_add(value);
                        }
                        break;
                    }
                }
            }
        }

        // Final per-candidate scores: the baked root prior plus the
        // folded token-independent transition edge weight; canonical
        // argmax (highest score, then the lowest token id).
        let mut ranked_candidates: Vec<(u32, ScoreQ)> = Vec::with_capacity(candidates.len());
        for (token, residual) in candidates {
            let with_offset = residual.saturating_add(transition_offset);
            let score = self.root_score(token).saturating_add(with_offset);
            ranked_candidates.push((token, score));
        }
        if ranked_candidates.is_empty() {
            return None;
        }
        let mut best_index = 0usize;
        for (index, &(_, score)) in ranked_candidates.iter().enumerate() {
            if score > ranked_candidates[best_index].1 {
                best_index = index;
            }
        }
        let (selected, selected_score) = ranked_candidates[best_index];
        Some(LegacyOutcome {
            selected,
            selected_score,
            candidates: ranked_candidates,
        })
    }
}

// ---------------------------------------------------------------------
// Deployed allocation-free scoring step (Phase 5 deployed path, issue
// #78). `score_candidates` is the witness-emitting reference: bounded
// but heap-based. The deployed HTTP adapter needs the same Rule 1 /
// Rule 2 semantics on fixed-capacity buffers — zero allocation per
// prediction in steady state (the status-policy allocation census
// asserts it). `score_step` recomputes a prediction into a caller-owned
// [`StepState`] and reports the selection, the [`ScoreStatus`], the
// candidate count, and the op census — no witness, no per-call heap.
// Parity with `score_candidates` (selected token, score, status,
// candidate count) is asserted per probe signature by the status-policy
// probe suite whenever the reference runs the deployed configuration
// (predicted-cloud emissions off, residualized RX1 exact-context
// evidence).
//
// Two deliberate fail-closed restrictions:
// - predicted-cloud (F / delta-T) emissions must be off — the deployed
//   default since the #66 ablation;
// - exact-context evidence must be the residualized RX1 table. The
//   legacy raw-TLS1 fallback quantizes at probe time (the one delimited
//   float site above), which the deployed integer contract forbids;
//   [`GraphScorer::has_legacy_exct`] lets the adapter detect that case
//   and keep those artifacts on the reference path.
// ---------------------------------------------------------------------

/// The membership widening bound of the D4 fallback policy: the deployed
/// adapter retries a Novel prediction once with the per-depth membership
/// set widened from [`TOP_M`] to twice that many entries, never more
/// (threat model: fallback denial-of-service is bounded by manifest
/// constants).
pub const WIDENED_TOP_M: usize = TOP_M + TOP_M;

/// Vocabulary bound for the fixed-capacity step buffers. HEAD declares
/// the vocabulary as u32 and the step state indexes accumulators by
/// token, so an artifact declaring an outlandish vocabulary would
/// otherwise force an outsized one-time allocation when the state is
/// built. Compiler-produced vocabularies are orders of magnitude below
/// this bound.
pub const STEP_MAX_VOCAB: u32 = 1 << 22;

/// The fixed-capacity scratch of one deployed scoring step: every
/// buffer sized once (see [`GraphScorer::step_state`]), epoch-stamped
/// so per-prediction reuse never re-zeroes a vocabulary-sized buffer.
/// Construction allocates; [`GraphScorer::score_step`] itself performs
/// no allocation.
pub struct StepState {
    /// Residual accumulators indexed by token; valid where
    /// `cand_epoch[token] == epoch`.
    residuals: Vec<i32>,
    /// Per-token touch stamps (candidate dedup without re-zeroing).
    cand_epoch: Vec<u64>,
    /// Candidate tokens in first-touch order.
    touched: Vec<u32>,
    /// The active cloud (region, depth, distance, margin).
    active: Vec<ActiveRegion>,
    /// Membership scratch: the per-depth top insertion list (one extra
    /// slot for the insertion transient).
    member_top: Vec<(u32, u32)>,
    /// Membership scratch: the within-radius filter output.
    member_within: Vec<(u32, u32)>,
    /// Ancestor-path scratch of the covered-chain walk.
    path: Vec<u32>,
    /// Current member's covered chain (node ids, root-to-leaf).
    chain: Vec<u32>,
    /// The selected covered chain.
    selected_chain: Vec<u32>,
    /// Node-in-selected-chain stamps (indexed by node id).
    chain_epoch: Vec<u64>,
    /// Contributing-node dedup stamps (indexed by node id).
    node_epoch: Vec<u64>,
    /// Current epoch counter.
    epoch: u64,
    /// Largest membership width this state supports.
    max_top_m: usize,
}

/// The outcome of one deployed scoring step: the selection and status
/// of [`GraphScorer::score_candidates`] without the witness.
#[derive(Debug, Clone)]
pub struct StepOutcome {
    /// Selected token (canonical tie-break: highest score, lowest id).
    pub selected: u32,
    /// Its final score.
    pub selected_score: ScoreQ,
    /// Which rule produced the selection.
    pub status: ScoreStatus,
    /// Size of the scored candidate set.
    pub candidate_count: u32,
    /// The op census of this step.
    pub census: OpKernel,
    /// Attribution for `ExactContext` selections (`None` otherwise);
    /// see [`ExactContextSource`].
    pub exact_context_source: Option<ExactContextSource>,
}

/// Advance the step state's epoch, re-zeroing the stamp buffers on the
/// (practically unreachable) u64 wrap so a stale stamp can never alias
/// the fresh epoch.
fn step_next_epoch(state: &mut StepState) -> u64 {
    state.epoch = state.epoch.wrapping_add(1);
    if state.epoch == 0 {
        state.cand_epoch.fill(0);
        state.chain_epoch.fill(0);
        state.node_epoch.fill(0);
        state.epoch = 1;
    }
    state.epoch
}

/// Allocation-free class-code assignment for the deployed step's
/// exact-context probe: the nearest class per stage (ties to the lowest
/// class id, by strict-`<` replacement over ascending class ids) —
/// exactly the `code` half of `runtime::assign_memberships_plain`,
/// without its heap-built membership lists.
fn assign_code_plain(art: &Compiled, sig: &[u8; SIG_BYTES]) -> [u8; STAGES] {
    let mut code = [0u8; STAGES];
    for (st_code, sigs) in code.iter_mut().zip(art.class_sigs.iter()) {
        let mut best = u32::MAX;
        let mut best_class = 0u8;
        for (kk, cs) in sigs.chunks_exact(SIG_BYTES).enumerate() {
            let mut dist = 0u32;
            for (&x, &y) in sig.iter().zip(cs.iter()) {
                dist += (x ^ y).count_ones();
            }
            if dist < best {
                best = dist;
                best_class = kk as u8;
            }
        }
        *st_code = best_class;
    }
    code
}

impl GraphScorer {
    /// Allocation-free form of [`binary_memberships`] with a
    /// caller-chosen membership width: identical insertion order, tie
    /// rule, within-radius filter, and nearest-region fallback, writing
    /// the result into caller-owned scratch. `member_top` needs room
    /// for `top_m + 1` entries (the insertion transient); `out`
    /// receives at most `top_m`.
    fn memberships_into(
        &self,
        k: &mut OpKernel,
        depth: usize,
        sig: &[u8; SIG_BYTES],
        top_m: usize,
        member_top: &mut Vec<(u32, u32)>,
        out: &mut Vec<(u32, u32)>,
    ) {
        member_top.clear();
        out.clear();
        for region in &self.regions {
            if region.depth as usize != depth {
                continue;
            }
            let dist = hamming_counted(k, &self.pop, sig, &region.sig);
            let mut inserted = false;
            for (idx, &(_, d0)) in member_top.iter().enumerate() {
                k.compares += 1;
                if dist < d0 {
                    member_top.insert(idx, (region.region_id(), dist));
                    inserted = true;
                    break;
                }
            }
            if !inserted && member_top.len() < top_m {
                member_top.push((region.region_id(), dist));
            }
            if inserted && member_top.len() > top_m {
                member_top.pop();
            }
        }
        for &(id, dist) in member_top.iter() {
            k.compares += 1;
            if dist <= u32::from(self.regions[id as usize].radius) {
                out.push((id, dist));
            }
        }
        if out.is_empty() {
            if let Some(&nearest) = member_top.first() {
                // Nearest-region fallback (the backoff floor).
                out.push(nearest);
            }
        }
    }

    /// Allocation-free form of [`covered_chain`]: the contiguous covered
    /// refinement prefix of `region` (node ids, root-to-leaf), written
    /// into caller-owned scratch. `path` and `out` are bounded by the
    /// region count (the malformed-cycle guard of the reference); a
    /// parent index outside the region table terminates the walk, which
    /// only malformed artifacts can trigger.
    fn covered_chain_into(
        &self,
        k: &mut OpKernel,
        region: u32,
        sig: &[u8; SIG_BYTES],
        path: &mut Vec<u32>,
        out: &mut Vec<u32>,
    ) {
        path.clear();
        out.clear();
        let mut current = Some(region);
        while let Some(id) = current {
            if path.len() >= self.regions.len() {
                break;
            }
            let Some(params) = self.regions.get(id as usize) else {
                break;
            };
            path.push(id);
            current = params.parent;
        }
        for &id in path.iter().rev() {
            let Some(params) = self.regions.get(id as usize) else {
                break;
            };
            let dist = hamming_counted(k, &self.pop, sig, &params.sig);
            k.compares += 1;
            if dist <= u32::from(params.radius) {
                out.push(id + 1); // region id -> node id
            } else {
                break;
            }
        }
    }

    /// True when exact-context evidence is wired through the legacy raw
    /// TLS1 store (probe-time quantization) rather than the residualized
    /// RX1 table. The deployed step rejects such scorers (fail closed);
    /// the adapter keeps those artifacts on the reference path.
    pub fn has_legacy_exct(&self) -> bool {
        self.store.is_some()
    }

    /// Allocate the fixed-capacity step state for this scorer.
    /// Construction is the one allocating step; every later
    /// [`GraphScorer::score_step`] call with the state is
    /// allocation-free.
    pub fn step_state(&self, max_top_m: usize) -> Result<StepState, String> {
        if self.vocab == 0 || self.vocab > STEP_MAX_VOCAB {
            return Err(format!(
                "HEAD vocabulary {} outside the deployed step bound {}",
                self.vocab, STEP_MAX_VOCAB
            ));
        }
        if max_top_m == 0 {
            return Err("step state requires a nonzero membership bound".to_owned());
        }
        let vocab = self.vocab as usize;
        let depth_slots = self.max_depth.max(1);
        let mut active_cap = 0usize;
        let mut i = 0usize;
        while i < max_top_m {
            active_cap = active_cap.saturating_add(depth_slots);
            i += 1;
        }
        let region_count = self.regions.len();
        Ok(StepState {
            residuals: vec![0i32; vocab],
            cand_epoch: vec![0u64; vocab],
            touched: Vec::with_capacity(vocab),
            active: Vec::with_capacity(active_cap),
            member_top: Vec::with_capacity(max_top_m + 1),
            member_within: Vec::with_capacity(max_top_m),
            path: Vec::with_capacity(region_count),
            chain: Vec::with_capacity(region_count),
            selected_chain: Vec::with_capacity(region_count),
            chain_epoch: vec![0u64; region_count + 1],
            node_epoch: vec![0u64; region_count + 1],
            epoch: 0,
            max_top_m,
        })
    }

    /// Accumulate one contributing node's emission list into the
    /// candidate buffers (Rule 1): the node contributes candidate tokens
    /// exactly once per epoch; its residual scores apply only when the
    /// node sits on the selected covered chain. Fails closed on an
    /// emission token outside the HEAD vocabulary (artifact integrity).
    fn step_accumulate_node(
        &self,
        node: u32,
        epoch: u64,
        state: &mut StepState,
        k: &mut OpKernel,
    ) -> Result<(), String> {
        if state.node_epoch[node as usize] == epoch {
            return Ok(());
        }
        state.node_epoch[node as usize] = epoch;
        let in_chain = state.chain_epoch[node as usize] == epoch;
        let Some(emissions) = self.emissions.get((node - 1) as usize) else {
            return Err("active node outside the emission tables".to_owned());
        };
        for (&token, &value) in emissions {
            k.candidate_scans += 1;
            k.table_reads += 1;
            let idx = token as usize;
            if idx >= state.residuals.len() {
                return Err("emission token outside the HEAD vocabulary".to_owned());
            }
            if state.cand_epoch[idx] != epoch {
                state.cand_epoch[idx] = epoch;
                state.residuals[idx] = 0;
                state.touched.push(token);
            }
            if in_chain {
                let effective_val = match self.scoring_variant {
                    ScoringVariant::ChainTelescoped => value,
                    ScoringVariant::CloudSizeNormalized => {
                        // #80 candidate variant: residual sum divided by
                        // chain length, via the shift/subtract divider.
                        let n = (state.selected_chain.len().max(1)) as i128;
                        ScoreQ::from_raw(shift_div_i128(i128::from(value.raw()), n) as i32)
                    }
                    ScoringVariant::MarginWeighted => {
                        // #80 candidate variant: residual scaled by
                        // normalized membership margin, via the
                        // shift/add multiplier and shift/subtract
                        // divider (saturating the final cast).
                        let margin = state
                            .active
                            .iter()
                            .find(|a| a.region + 1 == node)
                            .map(|a| a.margin.max(0) as i128)
                            .unwrap_or(1);
                        let radius = state
                            .active
                            .iter()
                            .find(|a| a.region + 1 == node)
                            .map(|a| u32::from(self.regions[a.region as usize].radius) as i128)
                            .unwrap_or(1)
                            .max(1);
                        let scaled =
                            shift_div_i128(shift_mul_i128(i128::from(value.raw()), margin), radius);
                        ScoreQ::from_raw(
                            scaled.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
                        )
                    }
                };
                state.residuals[idx] = ScoreQ::from_raw(state.residuals[idx])
                    .saturating_add(effective_val)
                    .raw();
                k.adds += 1;
            }
        }
        Ok(())
    }

    /// One deployed scoring step over `sig` with per-depth membership
    /// width `top_m` (bounded by the state's limit; the adapter uses
    /// [`TOP_M`] and, on a Novel first pass, [`WIDENED_TOP_M`]): the
    /// Rule 1 / Rule 2 semantics of [`GraphScorer::score_candidates`]
    /// recomputed on fixed-capacity buffers — integer-only, no per-call
    /// allocation. Selection, score, status, and candidate count match
    /// the reference exactly in the deployed configuration
    /// (predicted-cloud emissions off, residualized RX1 exact-context
    /// evidence or none); the status-policy probe suite asserts that
    /// parity per probe signature.
    pub fn score_step(
        &self,
        sig: &[u8; SIG_BYTES],
        top_m: usize,
        state: &mut StepState,
    ) -> Result<StepOutcome, String> {
        self.score_step_with_recent(sig, top_m, state, &[])
    }

    /// Deployed scoring step with the bounded repetition penalty applied to
    /// candidates present in `recent_tokens`.
    pub fn score_step_with_recent(
        &self,
        sig: &[u8; SIG_BYTES],
        top_m: usize,
        state: &mut StepState,
        recent_tokens: &[u32],
    ) -> Result<StepOutcome, String> {
        self.score_step_coded_with_recent(sig, None, top_m, state, recent_tokens)
    }

    /// [`GraphScorer::score_step_with_recent`] with an explicitly
    /// attested graded code (#243 Phase C option A) — the serving
    /// surface assigns under the artifact's declared metric from the
    /// live bundle and passes the code through; sign-metric artifacts
    /// may omit it (derived from the sig, which IS their metric).
    pub fn score_step_coded_with_recent(
        &self,
        sig: &[u8; SIG_BYTES],
        input_code: Option<&[u8; STAGES]>,
        top_m: usize,
        state: &mut StepState,
        recent_tokens: &[u32],
    ) -> Result<StepOutcome, String> {
        if top_m == 0 || top_m > state.max_top_m {
            return Err(format!(
                "membership width {top_m} outside the step state bound {}",
                state.max_top_m
            ));
        }
        if self.f_emissions {
            return Err(
                "deployed step requires predicted-cloud emissions disabled (#66 ablation)"
                    .to_owned(),
            );
        }
        if self.store.is_some() {
            return Err(
                "deployed step requires residualized RX1 exact-context evidence; legacy TLS1 store present"
                    .to_owned(),
            );
        }
        if state.residuals.len() != self.vocab as usize {
            return Err("step state does not match this scorer".to_owned());
        }
        if let Some(entries) = self.context_row(recent_tokens) {
            let (selected, selected_score) = Self::context_row_argmax(entries);
            let census = OpKernel {
                table_reads: 1,
                ..Default::default()
            };
            return Ok(StepOutcome {
                selected,
                selected_score,
                status: ScoreStatus::ExactContext,
                candidate_count: entries.len() as u32,
                census,
                exact_context_source: Some(ExactContextSource::NgramRow),
            });
        }
        let mut k = OpKernel::default();
        // Fresh epoch: stale stamps compare older and are overwritten on
        // first touch, so no vocabulary-sized buffer is re-zeroed.
        let epoch = step_next_epoch(state);
        state.touched.clear();
        state.active.clear();
        state.selected_chain.clear();

        // Active cloud A (module docs): the per-depth membership lists,
        // nearest-region fallback included.
        for depth in 1..=self.max_depth {
            self.memberships_into(
                &mut k,
                depth,
                sig,
                top_m,
                &mut state.member_top,
                &mut state.member_within,
            );
            for &(region, dist) in state.member_within.iter() {
                let radius = u32::from(self.regions[region as usize].radius);
                let margin = radius as i64 - dist as i64;
                state.active.push(ActiveRegion {
                    region,
                    depth: depth as u8,
                    distance: dist as u16,
                    margin: margin.clamp(i16::MIN as i64, i16::MAX as i64) as i16,
                });
            }
        }
        state.active.sort_unstable_by_key(|a| a.region);

        // Deepest covered refinement chain (ties: higher margin, then
        // the lowest region id) — the reference selection loop exactly.
        let mut selected_chain_region = u32::MAX;
        let mut selected_chain_margin = i16::MIN;
        for idx in 0..state.active.len() {
            let member = state.active[idx];
            self.covered_chain_into(
                &mut k,
                member.region,
                sig,
                &mut state.path,
                &mut state.chain,
            );
            let better = state.chain.len() > state.selected_chain.len()
                || (state.chain.len() == state.selected_chain.len()
                    && (member.margin > selected_chain_margin
                        || (member.margin == selected_chain_margin
                            && member.region < selected_chain_region)));
            if better {
                state.selected_chain.clear();
                state.selected_chain.extend_from_slice(&state.chain);
                selected_chain_region = member.region;
                selected_chain_margin = member.margin;
            }
        }

        // Candidate accumulation: active and chain nodes contribute
        // tokens; only the selected chain's nodes apply their emission
        // residuals (Rule 1 chain telescoping). Predicted-cloud
        // emissions are off in the deployed configuration, so the
        // folded transition offset is zero (reference parity).
        for &node in state.selected_chain.iter() {
            state.chain_epoch[node as usize] = epoch;
        }
        for idx in 0..state.active.len() {
            let node = state.active[idx].region + 1;
            self.step_accumulate_node(node, epoch, state, &mut k)?;
        }
        for idx in 0..state.selected_chain.len() {
            let node = state.selected_chain[idx];
            self.step_accumulate_node(node, epoch, state, &mut k)?;
        }
        // Root prior top-B tokens join the candidate set.
        for &token in &self.root_top {
            k.table_reads += 1;
            let idx = token as usize;
            if idx >= state.residuals.len() {
                return Err("root-prior token outside the HEAD vocabulary".to_owned());
            }
            if state.cand_epoch[idx] != epoch {
                state.cand_epoch[idx] = epoch;
                state.residuals[idx] = 0;
                state.touched.push(token);
            }
        }

        // Rule 2 (D4 EXCT precedence): the deepest-populated-prefix
        // probe over the residualized table; total evidence >=
        // EXCT_SUPPORT_MIN discards the graph candidate sets and ranks
        // only the admitted local entries.
        let mut exact_context = false;
        if let (Some(art), Some(residual_exct)) = (&self.artifacts, &self.residual_exct) {
            let code = match input_code {
                Some(code) => *code,
                None if art.dot_cb.is_empty() => assign_code_plain(art, sig),
                None => {
                    return Err(
                        "TLA6 artifact: the EXCT probe requires the assigned graded code \
                         (witness contract, #243 Phase C option A)"
                            .to_owned(),
                    );
                }
            };
            for level in (0..=STAGES).rev() {
                if let Some(context) = residual_exct[level].get(&code[..level]) {
                    // #234 maintainer decision: Rule 2 fires at FULL
                    // CODE ONLY (reference parity with score_candidates).
                    let supported = level == STAGES && context.total >= EXCT_SUPPORT_MIN;
                    let admitted = if supported {
                        context.entries.len().min(self.exct_top_x)
                    } else {
                        0
                    };
                    if admitted > 0 {
                        // Reset the candidate set to the admitted local
                        // entries under a fresh epoch.
                        let epoch = step_next_epoch(state);
                        state.touched.clear();
                        for (&token, &value) in context.entries.iter().take(self.exct_top_x) {
                            k.table_reads += 1;
                            let idx = token as usize;
                            if idx >= state.residuals.len() {
                                return Err(
                                    "exact-context token outside the HEAD vocabulary".to_owned()
                                );
                            }
                            if state.cand_epoch[idx] != epoch {
                                state.cand_epoch[idx] = epoch;
                                state.residuals[idx] = 0;
                                state.touched.push(token);
                            }
                            state.residuals[idx] = ScoreQ::from_raw(state.residuals[idx])
                                .saturating_add(value)
                                .raw();
                            k.adds += 1;
                        }
                        exact_context = true;
                    }
                    break;
                }
            }
        }

        if state.touched.is_empty() {
            return Err("scoring produced an empty candidate set".to_owned());
        }
        // Final per-candidate scores: the baked root prior plus the
        // accumulated residual; canonical argmax (highest score, then
        // the lowest token id) — the ascending-strict-`>` reference
        // scan computed order-free.
        let mut best = state.touched[0];
        let mut best_score = self
            .root_score(best)
            .saturating_add(ScoreQ::from_raw(state.residuals[best as usize]));
        k.adds += 1;
        if recent_tokens.contains(&best) {
            best_score = best_score.saturating_add(ScoreQ::from_raw(self.repetition_penalty_raw));
            k.adds += 1;
        }
        for &token in state.touched.iter().skip(1) {
            k.candidate_scans += 1;
            k.compares += 1;
            let mut score = self
                .root_score(token)
                .saturating_add(ScoreQ::from_raw(state.residuals[token as usize]));
            k.adds += 1;
            if recent_tokens.contains(&token) {
                score = score.saturating_add(ScoreQ::from_raw(self.repetition_penalty_raw));
                k.adds += 1;
            }
            if score > best_score || (score == best_score && token < best) {
                best = token;
                best_score = score;
            }
        }

        let status = if exact_context {
            ScoreStatus::ExactContext
        } else if state.selected_chain.is_empty() {
            ScoreStatus::Novel
        } else {
            ScoreStatus::Graph
        };
        Ok(StepOutcome {
            selected: best,
            selected_score: best_score,
            status,
            candidate_count: state.touched.len() as u32,
            census: k,
            exact_context_source: exact_context.then_some(ExactContextSource::ExctProbe),
        })
    }
}

/// Anchor stride of the gated two-pass generation path (issue #399):
/// the same stride-four grid the #394 infill protocol and the Gate C
/// forward-anchor instrumentation use. A generated position is on the
/// anchor grid when its emitted token lands on a story position that is
/// a multiple of this stride (see [`two_pass_infill_generate`] for the
/// grid definition in generated-stream coordinates).
pub const INFILL_STRIDE: usize = 4;

/// Result of one gated two-pass generation run
/// ([`two_pass_infill_generate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoPassGeneration {
    /// The pass-1 greedy draft, one token per generated position.
    pub draft: Vec<u32>,
    /// The pass-2 output: gated free positions re-ranked under the
    /// forward-anchor channel, every other position identical to the
    /// draft.
    pub final_tokens: Vec<u32>,
    /// Positions where the pass-2 re-rank changed the token.
    pub changed_positions: usize,
    /// Free positions that passed the confidence gate (an in-stream
    /// next anchor whose pass-1 selection resolved as `ExactContext`)
    /// and were therefore re-scored. `changed_positions` is bounded by
    /// this count.
    pub gated_positions: usize,
    /// Positions on the anchor grid (kept verbatim from the draft).
    pub anchor_positions: usize,
}

/// Everything pass 2 needs to re-rank a pass-1 position without
/// re-generating: the exact context inputs pass 1 scored with, plus the
/// selected token and its confidence status.
struct Pass1Record {
    token: u32,
    exact_context: bool,
    sig: [u8; SIG_BYTES],
    code: [u8; STAGES],
    recent: Vec<u32>,
}

/// Gated two-pass generation (issue #399, serving step 2): pass 1 runs
/// the ordinary greedy loop (window bundle → sig → attested code →
/// [`GraphScorer::score_candidates_coded`], sliding eight-token window
/// and 32-token recent deque — the `generate_greedy_repetition_rate`
/// pattern), recording for every generated position the selected token,
/// whether it resolved as [`ScoreStatus::ExactContext`], and the exact
/// context inputs (sig, code, recent tokens) it was scored with. Pass 2
/// then re-ranks each free position through
/// [`GraphScorer::score_candidates_infill`] with the DRAFT token at the
/// next anchor position as the forward anchor — but only where that
/// anchor's pass-1 selection resolved as `ExactContext` (the measured
/// B′ confidence gate, the strongest teacher-forced arm: live slice
/// 45.0% vs 39.4%).
///
/// Anchor grid: generated position `i` occupies story position
/// `seed_len + i` (the seed occupies story positions zero through
/// `seed_len - 1`), and it is an anchor when `(seed_len + i + 1)` is a
/// multiple of [`INFILL_STRIDE`] — the emitted token lands on a
/// stride-multiple story slot, mirroring the corpus-side law
/// (`(story_pos + 1) % stride == 0`) of the #394 infill protocol and
/// `compile_forward_anchor_rows`. The lookahead distance from a free
/// position to its next anchor is the position difference, matching the
/// FWDA row key `(distance, anchor)`.
///
/// Pass 2 is a PURE RE-RANK: every re-score consumes the stored pass-1
/// context (sig, code, recent tokens), so a changed token never feeds
/// back into later positions' context. A full iterative re-draft — where
/// pass-2 corrections propagate into subsequent windows — is future
/// work; this shape isolates the falsifier question (how much of the
/// gated lift survives draft-anchor context) from compounding effects.
///
/// One deliberate divergence from `generate_greedy_repetition_rate`:
/// when the seed is shorter than the bundle window, the window here
/// GROWS as tokens are generated (that probe's seeds always fill the
/// window, so its non-growing short-seed branch is never exercised).
pub fn two_pass_infill_generate(
    scorer: &GraphScorer,
    artifacts: &Compiled,
    rotations: &[usize; compiler::WINDOW + 1],
    seed: &[u32],
    tokens_to_generate: usize,
) -> Option<TwoPassGeneration> {
    let seed_len = seed.len();

    // ---- Pass 1: greedy draft, recording per-position context. ----
    let mut window = [0u32; compiler::WINDOW];
    let mut w_len = seed_len.min(compiler::WINDOW);
    window[..w_len].copy_from_slice(&seed[seed_len - w_len..]);
    let mut recent_tokens = std::collections::VecDeque::with_capacity(32);
    let r_len = seed_len.min(32);
    for &t in &seed[seed_len - r_len..] {
        recent_tokens.push_back(t);
    }

    let mut records: Vec<Pass1Record> = Vec::with_capacity(tokens_to_generate);
    for _ in 0..tokens_to_generate {
        let bundle = runtime::bundle_window_plain(artifacts, rotations, &window[..w_len]);
        let sig = runtime::sig_plain(artifacts, &bundle);
        // #243 Phase C option A: attest the metric-respecting code.
        let code = runtime::assign_for_bundle(artifacts, &bundle);
        let recent: Vec<u32> = recent_tokens.iter().copied().collect();
        let outcome = scorer.score_candidates_coded(&sig, Some(&code), &recent)?;
        let token = outcome.selected;
        records.push(Pass1Record {
            token,
            exact_context: outcome.witness.status == ScoreStatus::ExactContext,
            sig,
            code,
            recent,
        });

        if w_len < compiler::WINDOW {
            window[w_len] = token;
            w_len += 1;
        } else {
            window.copy_within(1.., 0);
            window[compiler::WINDOW - 1] = token;
        }
        if recent_tokens.len() == 32 {
            recent_tokens.pop_front();
        }
        recent_tokens.push_back(token);
    }

    // ---- Pass 2: gated re-rank against the drafted next anchor. ----
    let draft: Vec<u32> = records.iter().map(|record| record.token).collect();
    let mut final_tokens = draft.clone();
    let mut changed_positions = 0usize;
    let mut gated_positions = 0usize;
    let mut anchor_positions = 0usize;
    for (i, record) in records.iter().enumerate() {
        // Story slot of the token this position emits.
        let emitted_slot = seed_len + i + 1;
        if emitted_slot.is_multiple_of(INFILL_STRIDE) {
            anchor_positions += 1;
            continue;
        }
        let distance = emitted_slot.next_multiple_of(INFILL_STRIDE) - emitted_slot;
        let anchor_index = i + distance;
        let Some(anchor_record) = records.get(anchor_index) else {
            // The next anchor falls beyond the generated stream.
            continue;
        };
        if !anchor_record.exact_context {
            // B′ confidence gate: the draft anchor is trusted only where
            // pass 1 resolved it as exact context.
            continue;
        }
        gated_positions += 1;
        let outcome = scorer.score_candidates_infill(
            &record.sig,
            Some(&record.code),
            &record.recent,
            Some((anchor_record.token, distance as u8)),
        )?;
        if outcome.selected != record.token {
            final_tokens[i] = outcome.selected;
            changed_positions += 1;
        }
    }

    Some(TwoPassGeneration {
        draft,
        final_tokens,
        changed_positions,
        gated_positions,
        anchor_positions,
    })
}

/// The nearest GIVEN skeleton slot strictly after `index`, within the
/// FWDA lookahead range: `Some((token, distance))` when a `Some` slot
/// exists at `index + d` for the smallest `d` in one through
/// [`uor_r4_graph_format::FWDA_MAX_DISTANCE`], `None` when every slot
/// in that range is free or past the end. This is the anchor-selection
/// rule of [`infill_fill`], exposed so callers (the CLI summary line)
/// can reproduce the per-position anchor choice without re-running the
/// fill.
pub fn next_skeleton_anchor(skeleton: &[Option<u32>], index: usize) -> Option<(u32, u8)> {
    for distance in 1..=usize::from(uor_r4_graph_format::FWDA_MAX_DISTANCE) {
        match skeleton.get(index + distance) {
            None => return None,
            Some(&Some(token)) => return Some((token, distance as u8)),
            Some(&None) => {}
        }
    }
    None
}

/// A-mode infill serving (issue #399): fill the free positions of a
/// GIVEN anchor skeleton left to right. This is the measured, validated
/// serving mode — anchors are inputs, not drafts, so the channel is
/// immune to draft-anchor drift by construction (the falsifier the
/// two-pass probe quantified).
///
/// `skeleton` is the anchor grid: `Some(token)` at given positions
/// (any pattern of givens is accepted — the stride-four grid is the
/// typical shape, not a requirement), `None` at free positions. At a
/// given position the given token is consumed verbatim. At a free
/// position the ordinary context pipeline runs (window bundle → sig →
/// attested code → [`GraphScorer::score_candidates_infill`]) with
/// `next_anchor` chosen by [`next_skeleton_anchor`]: the nearest
/// following given within the FWDA lookahead range, or `None` when the
/// next given is farther than [`uor_r4_graph_format::FWDA_MAX_DISTANCE`]
/// or absent — in which case the fusion entry point degrades to base
/// scoring byte-identically.
///
/// Window and recent-token handling mirror [`two_pass_infill_generate`]:
/// the sliding [`compiler::WINDOW`]-token bundle window grows until
/// full, and the bounded recent deque feeds repetition control. Both
/// consume given and filled tokens alike, so every position is scored
/// with the exact context the emitted stream implies.
pub fn infill_fill(
    scorer: &GraphScorer,
    artifacts: &Compiled,
    rotations: &[usize; compiler::WINDOW + 1],
    skeleton: &[Option<u32>],
) -> Option<Vec<u32>> {
    let mut window = [0u32; compiler::WINDOW];
    let mut w_len = 0usize;
    let mut recent_tokens = std::collections::VecDeque::with_capacity(32);
    let mut filled: Vec<u32> = Vec::with_capacity(skeleton.len());
    for (index, slot) in skeleton.iter().enumerate() {
        let token = match slot {
            Some(given) => *given,
            None => {
                let bundle = runtime::bundle_window_plain(artifacts, rotations, &window[..w_len]);
                let sig = runtime::sig_plain(artifacts, &bundle);
                // #243 Phase C option A: attest the metric-respecting code.
                let code = runtime::assign_for_bundle(artifacts, &bundle);
                let recent: Vec<u32> = recent_tokens.iter().copied().collect();
                let outcome = scorer.score_candidates_infill(
                    &sig,
                    Some(&code),
                    &recent,
                    next_skeleton_anchor(skeleton, index),
                )?;
                outcome.selected
            }
        };
        filled.push(token);
        if w_len < compiler::WINDOW {
            window[w_len] = token;
            w_len += 1;
        } else {
            window.copy_within(1.., 0);
            window[compiler::WINDOW - 1] = token;
        }
        if recent_tokens.len() == 32 {
            recent_tokens.pop_front();
        }
        recent_tokens.push_back(token);
    }
    Some(filled)
}

/// Rejection reasons of the independent witness-replay verifier
/// (Theorems 6 and 10). Each variant names exactly one inconsistency
/// between the witness's claims and the recomputation from the
/// validated artifact bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    /// The artifact bytes failed two-stage validation or CID integrity.
    Artifact(String),
    /// The witness's graph CID does not match the artifact's
    /// `artifact_cid`.
    GraphCidMismatch,
    /// EXCT evidence was claimed but no teacher container was provided.
    TeacherMissing,
    /// The teacher container does not match HEAD `teacher_cid`.
    TeacherCidMismatch,
    /// The recomputed active cloud (regions, distances, margins)
    /// differs.
    ActiveMismatch,
    /// The recomputed scoring rule differs.
    StatusMismatch,
    /// The recomputed selected covered chain differs.
    ChainMismatch,
    /// The recomputed predicted cloud differs.
    PredictedMismatch,
    /// The recomputed applied-edge set or folded offset differs.
    EdgesMismatch,
    /// The recomputed EXCT probe differs.
    ExctMismatch,
    /// Theorem 10: a contribution id appears twice in the witness.
    DuplicateContributionId,
    /// Theorem 10: an applied edge id appears twice in the witness.
    DuplicateEdgeId,
    /// The selected token's contribution list differs from the
    /// recomputation (ids, order, or values).
    ContributionsMismatch,
    /// The selected token or its score differs from the recomputation.
    SelectedMismatch,
    /// The recorded candidate count differs from the recomputation.
    CandidateCountMismatch,
    /// The recorded op census differs from the recomputation.
    CensusMismatch,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::Artifact(e) => write!(f, "artifact validation failed: {e}"),
            ReplayError::GraphCidMismatch => write!(f, "graph CID mismatch"),
            ReplayError::TeacherMissing => write!(f, "EXCT claimed but no teacher container"),
            ReplayError::TeacherCidMismatch => write!(f, "teacher CID mismatch"),
            ReplayError::ActiveMismatch => write!(f, "active cloud mismatch"),
            ReplayError::StatusMismatch => write!(f, "score status mismatch"),
            ReplayError::ChainMismatch => write!(f, "selected chain mismatch"),
            ReplayError::PredictedMismatch => write!(f, "predicted cloud mismatch"),
            ReplayError::EdgesMismatch => write!(f, "applied edges mismatch"),
            ReplayError::ExctMismatch => write!(f, "EXCT probe mismatch"),
            ReplayError::DuplicateContributionId => {
                write!(f, "duplicate contribution id (Theorem 10)")
            }
            ReplayError::DuplicateEdgeId => write!(f, "duplicate applied edge id (Theorem 10)"),
            ReplayError::ContributionsMismatch => write!(f, "contribution list mismatch"),
            ReplayError::SelectedMismatch => write!(f, "selected token or score mismatch"),
            ReplayError::CandidateCountMismatch => write!(f, "candidate count mismatch"),
            ReplayError::CensusMismatch => write!(f, "op census mismatch"),
        }
    }
}

impl std::error::Error for ReplayError {}

/// Independent witness replay (Theorem 6): rebuild a scorer from the
/// validated R4G1 bytes — with the teacher container when the witness
/// claims EXCT evidence — recompute the prediction of
/// `witness.input_sig` without any compiler state, and require bit-exact
/// equality of every witness field. Duplicate contribution ids are
/// rejected before any comparison (Theorem 10).
pub fn verify_witness_replay(
    r4g1: &[u8],
    teacher_container: Option<&[u8]>,
    witness: &ScoreWitness,
    root_top_b: usize,
    exct_top_x: usize,
) -> Option<ReplayError> {
    // Theorem 10: reject duplicate contribution ids out of hand.
    let mut seen: BTreeSet<ContributionId> = BTreeSet::new();
    for contribution in &witness.selected_contributions {
        if !seen.insert(contribution.id) {
            return Some(ReplayError::DuplicateContributionId);
        }
    }
    let mut seen_edges: BTreeSet<u32> = BTreeSet::new();
    for edge in &witness.edges_applied {
        if !seen_edges.insert(edge.edge_id) {
            return Some(ReplayError::DuplicateEdgeId);
        }
    }

    if witness.exct.is_some() && teacher_container.is_none() {
        return Some(ReplayError::TeacherMissing);
    }
    // `from_artifact` is now total (`Option`), so the teacher-cid mismatch is
    // detected here rather than sniffed from an error string: a supplied
    // teacher whose blake3 does not reproduce HEAD `teacher_cid` is the one
    // distinguished replay failure, and any other `None` is a generic
    // artifact defect.
    if let Some(teacher) = teacher_container {
        if let Some(head) = GraphView::parse(r4g1).ok().and_then(|view| view.head()) {
            if blake3::hash(teacher).as_bytes() != &head.teacher_cid().0 {
                return Some(ReplayError::TeacherCidMismatch);
            }
        }
    }
    let scorer = match GraphScorer::from_artifact(r4g1, teacher_container, root_top_b, exct_top_x) {
        Some(mut scorer) => {
            scorer.set_f_emissions(witness.f_emissions);
            scorer
        }
        None => {
            return Some(ReplayError::Artifact(
                "artifact bytes are not a scorer on replay".to_owned(),
            ));
        }
    };
    if scorer.graph_cid != witness.graph_cid {
        return Some(ReplayError::GraphCidMismatch);
    }
    let outcome = match scorer.score_candidates_coded(
        &witness.input_sig,
        witness.input_code.as_ref(),
        &witness.context_tokens,
    ) {
        Some(outcome) => outcome,
        None => {
            return Some(ReplayError::Artifact(
                "score_candidates_coded produced no outcome on replay".to_owned(),
            ));
        }
    };
    let recomputed = &outcome.witness;

    if recomputed.input_code != witness.input_code {
        return Some(ReplayError::Artifact(
            "input_code mismatch on replay (witness contract, #243 Phase C)".to_owned(),
        ));
    }
    if recomputed.context_tokens != witness.context_tokens {
        return Some(ReplayError::Artifact(
            "context_tokens mismatch on replay".to_owned(),
        ));
    }
    if recomputed.active != witness.active {
        return Some(ReplayError::ActiveMismatch);
    }
    if recomputed.status != witness.status {
        return Some(ReplayError::StatusMismatch);
    }
    if recomputed.chain != witness.chain {
        return Some(ReplayError::ChainMismatch);
    }
    if recomputed.predicted != witness.predicted {
        return Some(ReplayError::PredictedMismatch);
    }
    if recomputed.edges_applied != witness.edges_applied
        || recomputed.transition_offset != witness.transition_offset
    {
        return Some(ReplayError::EdgesMismatch);
    }
    if recomputed.exct != witness.exct {
        return Some(ReplayError::ExctMismatch);
    }
    if recomputed.selected_contributions != witness.selected_contributions {
        return Some(ReplayError::ContributionsMismatch);
    }
    if recomputed.selected != witness.selected
        || recomputed.selected_score != witness.selected_score
    {
        return Some(ReplayError::SelectedMismatch);
    }
    if recomputed.candidate_count != witness.candidate_count {
        return Some(ReplayError::CandidateCountMismatch);
    }
    if recomputed.census != witness.census {
        return Some(ReplayError::CensusMismatch);
    }
    None
}

// `shift_mul_i128`/`shift_div_i128` are exercised below against literal,
// hand-computed expectations (not the native `*`/`/` operators) so this
// test module itself stays clear of the P-4 source scan above.
#[cfg(test)]
mod shift_arithmetic_tests {
    use super::{shift_div_i128, shift_mul_i128};

    #[test]
    fn shift_mul_matches_hand_computed_products() {
        let cases: &[(i128, i128, i128)] = &[
            (0, 0, 0),
            (0, 5, 0),
            (5, 0, 0),
            (7, 6, 42),
            (-7, 6, -42),
            (7, -6, -42),
            (-7, -6, 42),
            (1, i128::from(i32::MAX), i128::from(i32::MAX)),
            (-1, i128::from(i32::MAX), -i128::from(i32::MAX)),
            (i128::from(i32::MIN), 3, -6_442_450_944),
        ];
        for &(a, b, expected) in cases {
            assert_eq!(shift_mul_i128(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn shift_mul_saturates_instead_of_overflowing() {
        // i128::MAX doubled overflows i128; the helper must saturate
        // rather than panic or silently wrap.
        assert_eq!(shift_mul_i128(i128::MAX, 2), i128::MAX);
        assert_eq!(shift_mul_i128(i128::MIN, 2), -i128::MAX);
    }

    #[test]
    fn shift_div_matches_hand_computed_truncating_quotients() {
        let cases: &[(i128, i128, i128)] = &[
            (0, 5, 0),
            (7, 2, 3),
            (-7, 2, -3),
            (7, -2, -3),
            (-7, -2, 3),
            (1, 5, 0),
            (-1, 5, 0),
            (i128::from(i32::MIN), 3, -715_827_882),
            (i128::from(i32::MAX), 7, 306_783_378),
        ];
        for &(dividend, divisor, expected) in cases {
            assert_eq!(
                shift_div_i128(dividend, divisor),
                expected,
                "dividend={dividend} divisor={divisor}"
            );
        }
    }

    #[test]
    fn shift_div_by_zero_returns_zero() {
        assert_eq!(shift_div_i128(42, 0), 0);
        assert_eq!(shift_div_i128(-42, 0), 0);
        assert_eq!(shift_div_i128(0, 0), 0);
    }
}

#[cfg(test)]
mod ngram_tests {
    use super::*;

    fn scorer(rows: BTreeMap<(u8, u32, u32), Vec<(u32, ScoreQ)>>) -> GraphScorer {
        GraphScorer {
            graph_cid: [0; 32],
            regions: Vec::new(),
            max_depth: 0,
            forward_by_src: BTreeMap::new(),
            root_prior: BTreeMap::new(),
            root_floor: ScoreQ::ZERO,
            root_top: Vec::new(),
            emissions: Vec::new(),
            context_rows: rows,
            fwd_rows: BTreeMap::new(),
            residual_exct: None,
            store: None,
            artifacts: None,
            vocab: 32,
            exct_top_x: 0,
            pop: [0; 256],
            f_emissions: false,
            scoring_variant: ScoringVariant::ChainTelescoped,
            repetition_penalty_raw: DEFAULT_REPETITION_PENALTY_RAW,
            fallback_policy: Default::default(),
        }
    }

    #[test]
    fn context_backoff_prefers_trigram_then_bigram() {
        let mut rows = BTreeMap::new();
        rows.insert((1, 20, 0), vec![(6, ScoreQ::from_raw(100))]);
        rows.insert(
            (2, 10, 20),
            vec![(4, ScoreQ::from_raw(10)), (5, ScoreQ::from_raw(20))],
        );
        let mut scorer = scorer(rows);
        let row_argmax = |scorer: &GraphScorer, tokens: &[u32]| {
            scorer
                .context_row(tokens)
                .map(GraphScorer::context_row_argmax)
        };
        assert_eq!(
            row_argmax(&scorer, &[10, 20]),
            Some((5, ScoreQ::from_raw(20)))
        );
        scorer.context_rows.remove(&(2, 10, 20));
        assert_eq!(
            row_argmax(&scorer, &[10, 20]),
            Some((6, ScoreQ::from_raw(100)))
        );
        assert_eq!(row_argmax(&scorer, &[30]), None);
    }

    /// #362 attribution: an NGRAM-row selection reports `NgramRow`
    /// provenance, and a probe-free non-exact selection reports `None` —
    /// the exact-context bucket stays attributable after context rows.
    #[test]
    fn exact_context_source_attributes_ngram_hits() {
        let mut rows = BTreeMap::new();
        rows.insert((1, 20, 0), vec![(6, ScoreQ::from_raw(100))]);
        let scorer = scorer(rows);
        let sig = [0u8; SIG_BYTES];
        let outcome = scorer
            .score_candidates_coded(&sig, None, &[10, 20])
            .expect("ngram row scores");
        assert_eq!(outcome.witness.status, ScoreStatus::ExactContext);
        assert_eq!(
            outcome.exact_context_source,
            Some(ExactContextSource::NgramRow)
        );
        assert!(outcome.witness.exct.is_none());
    }

    /// #380: a row hit carries the whole bounded row as the emission
    /// distribution, while the selection stays the canonical argmax — so
    /// served tokens are unchanged and only the distribution widens.
    #[test]
    fn context_row_hit_yields_the_full_bounded_distribution() {
        let mut rows = BTreeMap::new();
        rows.insert(
            (1, 20, 0),
            vec![(4, ScoreQ::from_raw(10)), (6, ScoreQ::from_raw(100))],
        );
        let scorer = scorer(rows);
        let sig = [0u8; SIG_BYTES];
        let outcome = scorer
            .score_candidates_coded(&sig, None, &[10, 20])
            .expect("ngram row scores");
        assert_eq!(outcome.selected, 6, "argmax selection is unchanged");
        assert_eq!(
            outcome.candidates,
            vec![(4, ScoreQ::from_raw(10)), (6, ScoreQ::from_raw(100))],
            "the full row is the distribution, ascending token order"
        );
        assert_eq!(outcome.witness.candidate_count, 2);
        assert_eq!(outcome.candidate_components.len(), 2);
    }
}
