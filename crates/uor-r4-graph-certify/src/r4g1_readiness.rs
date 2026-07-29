//! Certifier-side readiness probe for the R4G1 graph evaluation (issue #232).
//!
//! `certify` converts the legacy store to an R4G1 container and evaluates it
//! over the full held-out set. When the graph is untrained or unscored, that
//! evaluation runs for hours and terminates in an all-zero row — output that
//! reads as a measured failure when it is actually an absent measurement.
//!
//! The probe runs the same prediction call over a small, deterministic sample
//! of held-out positions first and classifies the graph's output shape. Two
//! degenerate shapes are detected:
//!
//! - **No scored emission**: every probe prediction returns `ScoreQ::MIN`,
//!   i.e. no emission list was ever reached with a real score — the constant
//!   root-fallback path in `predict_distribution` is the only thing running.
//! - **Constant prediction**: every probe position yields the same token
//!   regardless of context. A trained graph queried with varied contexts
//!   does not produce a single constant token across the whole probe.
//!
//! Either shape means the full multi-hour evaluation would produce a vacuous
//! row; the caller should skip it and say so explicitly.
//!
//! Certifier-side code: allocation and iterators are permitted here (this is
//! not the deployed runtime kernel).

use std::time::{Duration, Instant};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;

/// Number of held-out positions sampled by the probe. Deterministic: callers
/// take the first `PROBE_POSITIONS` positions of the held-out set in order.
pub const PROBE_POSITIONS: usize = 64;

/// Outcome of the readiness probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R4g1EvalReadiness {
    /// The graph produced varied, scored predictions on the probe; the full
    /// evaluation is meaningful. `scored` counts probe positions whose best
    /// score exceeded `ScoreQ::MIN`.
    Ready { scored: usize },
    /// Every probe prediction came back at `ScoreQ::MIN`: no scored emission
    /// was reachable from any probe context.
    NoScoredEmission,
    /// Every probe position predicted the same token — the constant
    /// root-fallback shape of an untrained or unscored graph.
    ConstantPrediction { token: u32 },
    /// The probe could not finish inside its wall-clock budget (issue
    /// #278): per-position prediction cost on the current graph size makes
    /// the full evaluation infeasible. `probed` counts positions completed
    /// before the budget check tripped.
    BudgetExceeded { probed: usize, elapsed: Duration },
}

impl R4g1EvalReadiness {
    /// True when the full evaluation should run.
    pub fn is_ready(&self) -> bool {
        matches!(self, R4g1EvalReadiness::Ready { .. })
    }
}

/// Classify the graph's output shape over `probe_contexts`.
///
/// `node_scores` must be at least `runtime.node_count()` long; it is reset
/// before every prediction and left dirty afterwards (callers reset it again
/// before reuse). An empty probe iterator classifies as `NoScoredEmission`
/// (there is no evidence the graph can emit).
pub fn r4g1_eval_readiness<'a, I>(
    runtime: &R4G1Runtime,
    probe_contexts: I,
    node_scores: &mut [ScoreQ],
) -> R4g1EvalReadiness
where
    I: IntoIterator<Item = &'a [u32]>,
{
    r4g1_eval_readiness_within(runtime, probe_contexts, node_scores, Duration::MAX)
}

/// Budgeted variant of [`r4g1_eval_readiness`] (issue #278): checks
/// wall-clock elapsed time before each position and classifies as
/// [`R4g1EvalReadiness::BudgetExceeded`] once `budget` is spent. The check
/// runs between positions, so a single in-flight prediction bounds the
/// overshoot past the budget.
pub fn r4g1_eval_readiness_within<'a, I>(
    runtime: &R4G1Runtime,
    probe_contexts: I,
    node_scores: &mut [ScoreQ],
    budget: Duration,
) -> R4g1EvalReadiness
where
    I: IntoIterator<Item = &'a [u32]>,
{
    let start = Instant::now();
    let mut scored = 0usize;
    let mut first_token: Option<u32> = None;
    let mut constant = true;
    let mut probed = 0usize;

    for ctx in probe_contexts {
        if start.elapsed() >= budget {
            return R4g1EvalReadiness::BudgetExceeded {
                probed,
                elapsed: start.elapsed(),
            };
        }
        probed += 1;
        node_scores.fill(ScoreQ::MIN);
        let (token, score) = runtime.predict_distribution(ctx, None, node_scores);
        if score.raw() > ScoreQ::MIN.raw() {
            scored += 1;
        }
        match first_token {
            None => first_token = Some(token),
            Some(t) if t != token => constant = false,
            Some(_) => {}
        }
    }

    if probed == 0 || scored == 0 {
        return R4g1EvalReadiness::NoScoredEmission;
    }
    if constant {
        // `probed > 0` guarantees `first_token` is set.
        return R4g1EvalReadiness::ConstantPrediction {
            token: first_token.unwrap_or(0),
        };
    }
    R4g1EvalReadiness::Ready { scored }
}
