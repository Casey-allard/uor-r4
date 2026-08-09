//! Executable proof module: Deterministic top-K candidate sorting & canonical tie-breaking.

use uor_r4_core::transformerless::score_q::ScoreQ;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub token: u32,
    pub score: ScoreQ,
}

/// Sort candidate list according to canonical tie-breaking rule:
/// Highest score S(v) first, then lowest lexical TokenId second.
pub fn sort_candidates_canonical(candidates: &mut [Candidate]) {
    candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.token.cmp(&b.token)));
}

/// Verify that a candidate list satisfies canonical tie-breaking order. Total:
/// returns `None` when the order is canonical, or `Some(reason)` at the first
/// violation (R5 — a failed proof is a measured report, not a raised error).
pub fn verify_canonical_order(candidates: &[Candidate]) -> Option<String> {
    for i in 0..candidates.len().saturating_sub(1) {
        let a = &candidates[i];
        let b = &candidates[i + 1];
        if a.score < b.score {
            return Some(format!(
                "Canonical order violation at index {}: score {:?} < {:?}",
                i, a.score, b.score
            ));
        }
        if a.score == b.score && a.token >= b.token {
            return Some(format!(
                "Canonical tie-breaking violation at index {}: token {} >= {}",
                i, a.token, b.token
            ));
        }
    }
    None
}
