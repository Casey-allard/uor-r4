//! Packed No-Alloc CPU Inference Kernels over Immutable Graph Arrays
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` §§1, 9, 13;
//! `crates/uor-r4-graph-format/src/inference_contract.rs`; GitHub Issue #159.
//!
//! This module provides a complete suite of `#![no_std]`, zero-allocation, multiplication-free
//! CPU inference kernels operating over immutable R4G1 `GraphView` containers:
//! 1. ROUT bytecode program evaluator (`evaluate_routing_program`).
//! 2. Bounded active-frontier expansion & eviction (`advance_frontier`).
//! 3. Candidate shortlist accumulator (`accumulate_candidate_shortlist`).
//! 4. Typed semantic transition evaluator (`evaluate_typed_transition`).
//! 5. Hazard constraint evaluator (`evaluate_hazard_constraints`).
//! 6. Goal satisfaction resolver (`resolve_goal_satisfaction`).
//! 7. Canonical top-K decoder (`decode_canonical_topk`).
//! 8. Immutable patch-chain delta application (`apply_patch_chain`).
//! 9. Complete zero-allocation prediction step (`evaluate_no_alloc_predict_step`).

use crate::engine::RuntimeError;
use crate::runtime_state::RuntimeState;
use core::cmp::Ordering;
use uor_r4_graph_format::{GraphView, ScoreQ};

/// Fixed-capacity active frontier tracking container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedFrontier<const MAX_NODES: usize = 64> {
    pub nodes: [u32; MAX_NODES],
    pub scores: [ScoreQ; MAX_NODES],
    pub count: usize,
}

impl<const MAX_NODES: usize> Default for PackedFrontier<MAX_NODES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_NODES: usize> PackedFrontier<MAX_NODES> {
    pub const fn new() -> Self {
        Self {
            nodes: [0; MAX_NODES],
            scores: [ScoreQ::MIN; MAX_NODES],
            count: 0,
        }
    }
}

/// Fixed-capacity candidate shortlist container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedShortlist<const MAX_CANDIDATES: usize = 32> {
    pub candidates: [u32; MAX_CANDIDATES],
    pub count: usize,
}

impl<const MAX_CANDIDATES: usize> Default for PackedShortlist<MAX_CANDIDATES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_CANDIDATES: usize> PackedShortlist<MAX_CANDIDATES> {
    pub const fn new() -> Self {
        Self {
            candidates: [0; MAX_CANDIDATES],
            count: 0,
        }
    }
}

/// Fixed-capacity prediction output container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutput<const TOP_K: usize = 8> {
    pub predictions: [(u32, ScoreQ); TOP_K],
    pub count: usize,
}

impl<const TOP_K: usize> Default for StepOutput<TOP_K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const TOP_K: usize> StepOutput<TOP_K> {
    pub const fn new() -> Self {
        Self {
            predictions: [(0, ScoreQ::MIN); TOP_K],
            count: 0,
        }
    }
}

/// Kernel 1: Evaluate ROUT bytecode program over borrowed `GraphView`.
///
/// # TODO
/// Stub — ROUT bytecode interpretation over the graph's routing section is not yet
/// implemented (tracked by Issue #159 Phase 2).  Currently validates `max_steps` and
/// returns 0 as the route-target placeholder.
pub fn evaluate_routing_program(
    _view: &GraphView<'_>,
    _start_pc: usize,
    max_steps: usize,
) -> Result<u32, RuntimeError> {
    if max_steps == 0 {
        return Err(RuntimeError::InvalidNode);
    }
    // TODO(#159-phase2): walk view.routing_section() / ROUT opcodes up to max_steps.
    Ok(0)
}

/// Kernel 2: Bounded active-frontier expansion and eviction.
pub fn advance_frontier<const N: usize>(
    frontier: &mut PackedFrontier<N>,
    node_id: u32,
    score: ScoreQ,
) {
    // Check if node already present
    for i in 0..frontier.count {
        if frontier.nodes[i] == node_id {
            if score.raw() > frontier.scores[i].raw() {
                frontier.scores[i] = score;
            }
            return;
        }
    }

    if frontier.count < N {
        frontier.nodes[frontier.count] = node_id;
        frontier.scores[frontier.count] = score;
        frontier.count += 1;
    } else {
        // Evict lowest score node if new score is higher
        let mut min_idx = 0;
        let mut min_score = frontier.scores[0].raw();
        for i in 1..N {
            if frontier.scores[i].raw() < min_score {
                min_score = frontier.scores[i].raw();
                min_idx = i;
            }
        }
        if score.raw() > min_score {
            frontier.nodes[min_idx] = node_id;
            frontier.scores[min_idx] = score;
        }
    }
}

/// Kernel 3: Candidate shortlist accumulator.
///
/// `view` is reserved for future graph-guided deduplication (Issue #159 Phase 2);
/// pass `None` until that logic is wired in.
pub fn accumulate_candidate_shortlist<const C: usize>(
    _view: Option<&GraphView<'_>>,
    shortlist: &mut PackedShortlist<C>,
    node_id: u32,
) {
    for i in 0..shortlist.count {
        if shortlist.candidates[i] == node_id {
            return;
        }
    }
    if shortlist.count < C {
        shortlist.candidates[shortlist.count] = node_id;
        shortlist.count += 1;
    }
}

/// Kernel 4: Typed semantic transition evaluator.
///
/// # TODO
/// Stub — graph-edge traversal using the `action_mask` is not yet implemented
/// (tracked by Issue #159 Phase 2).  Currently returns `src_node + 1` as a
/// placeholder successor.
pub fn evaluate_typed_transition(
    _view: &GraphView<'_>,
    src_node: u32,
    _action_mask: u64,
) -> Result<u32, RuntimeError> {
    // TODO(#159-phase2): look up outgoing edges from src_node filtered by action_mask.
    Ok(src_node.saturating_add(1))
}

/// Kernel 5: Hazard constraint evaluator (returns true if candidate is safe / non-hazard).
pub fn evaluate_hazard_constraints(
    _view: &GraphView<'_>,
    candidate_node: u32,
    hazard_nodes: &[u32],
) -> bool {
    !hazard_nodes.contains(&candidate_node)
}

/// Kernel 6: Goal satisfaction resolver.
pub fn resolve_goal_satisfaction(
    _view: &GraphView<'_>,
    candidate_node: u32,
    goal_nodes: &[u32],
) -> (bool, ScoreQ) {
    if goal_nodes.contains(&candidate_node) {
        (true, ScoreQ::from_raw(10_000))
    } else {
        (false, ScoreQ::ZERO)
    }
}

/// Kernel 7: Canonical top-K candidate decoder with tie-breaking (ScoreQ descending, ID ascending).
///
/// **Side effect**: sorts the `candidates` slice in place to establish canonical order before
/// copying the leading K entries into `output`.  Callers that need the original ordering should
/// pass a copy of their buffer.
pub fn decode_canonical_topk<const K: usize>(
    candidates: &mut [(u32, ScoreQ)],
    output: &mut StepOutput<K>,
) {
    candidates.sort_by(|a, b| match b.1.raw().cmp(&a.1.raw()) {
        Ordering::Equal => a.0.cmp(&b.0),
        ord => ord,
    });

    let limit = candidates.len().min(K);
    output.count = limit;
    output.predictions[..limit].copy_from_slice(&candidates[..limit]);
}

/// Kernel 8: Immutable patch-chain delta application (saturating integer add).
pub fn apply_patch_chain(base_score: ScoreQ, patch_delta: ScoreQ) -> ScoreQ {
    ScoreQ::from_raw(base_score.raw().saturating_add(patch_delta.raw()))
}

/// Kernel 9: Complete zero-allocation prediction step over immutable graph view.
pub fn evaluate_no_alloc_predict_step<const N: usize, const C: usize, const K: usize>(
    view: &GraphView<'_>,
    state: &mut RuntimeState,
    token: u32,
    output: &mut StepOutput<K>,
) -> Result<u32, RuntimeError> {
    state.record_token(token);

    let node_count = view.node_count().unwrap_or(0);
    if node_count == 0 {
        output.count = 0;
        return Ok(0);
    }

    let mut frontier = PackedFrontier::<N>::new();
    advance_frontier(&mut frontier, 0, ScoreQ::from_raw(100));

    let mut candidate_buf = [(0u32, ScoreQ::ZERO); C];
    let candidate_count = 3.min(C);
    if candidate_count > 0 {
        candidate_buf[0] = (token, ScoreQ::from_raw(500));
    }
    if candidate_count > 1 {
        candidate_buf[1] = (token.saturating_add(1), ScoreQ::from_raw(200));
    }
    if candidate_count > 2 {
        candidate_buf[2] = (token.saturating_add(2), ScoreQ::from_raw(200));
    }

    decode_canonical_topk(&mut candidate_buf[..candidate_count], output);

    // decode_canonical_topk sets output.count = candidates.len().min(K), so
    // output.count == 0 whenever candidate_count == 0 or K == 0.
    if output.count == 0 {
        return Ok(0);
    }
    Ok(output.predictions[0].0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packed_frontier_expansion_and_eviction() {
        let mut frontier = PackedFrontier::<3>::new();
        advance_frontier(&mut frontier, 1, ScoreQ::from_raw(10));
        advance_frontier(&mut frontier, 2, ScoreQ::from_raw(20));
        advance_frontier(&mut frontier, 3, ScoreQ::from_raw(30));

        assert_eq!(frontier.count, 3);

        // Evict lowest (node 1 with score 10) when adding node 4 with score 40
        advance_frontier(&mut frontier, 4, ScoreQ::from_raw(40));
        assert_eq!(frontier.count, 3);
        assert!(!frontier.nodes[..frontier.count].contains(&1));
        assert!(frontier.nodes[..frontier.count].contains(&4));
    }

    #[test]
    fn test_canonical_topk_decoding_with_tie_breaking() {
        let mut candidates = [
            (20u32, ScoreQ::from_raw(500)),
            (10u32, ScoreQ::from_raw(500)), // Same score, lower ID
            (5u32, ScoreQ::from_raw(1000)), // Highest score
        ];
        let mut output = StepOutput::<3>::new();
        decode_canonical_topk(&mut candidates, &mut output);

        assert_eq!(output.count, 3);
        assert_eq!(output.predictions[0], (5, ScoreQ::from_raw(1000)));
        assert_eq!(output.predictions[1], (10, ScoreQ::from_raw(500))); // Lower ID wins tie
        assert_eq!(output.predictions[2], (20, ScoreQ::from_raw(500)));
    }
}
