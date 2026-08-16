//! Pre-registered A/B comparison of `msa-structured-selector/1` against
//! `r4-route-attention/1` (issue #643): the SAME synthetic held-out
//! evaluation corpus and candidate set #605's replacement ladder
//! (`crate::route_fit_report`) measures `r4-route-attention/1` against,
//! now also driving `msa-structured-selector/1`'s deployed packed
//! kernel through the identical `support-restrict-renormalize/1`
//! teacher-forced evaluation (`crate::route_fit_report::scope_metrics`,
//! reused verbatim — never a reimplementation), plus a unigram floor
//! computed over the same corpus in the #457 "root null" convention.
//!
//! ## Why this harness needs no fitting step
//!
//! `r4-route-attention/1` requires an offline fit (`route-fit/1`:
//! project query/key hidden vectors to route codes, threshold, pack —
//! `uor_r4_graph_compiler::route_fit::fit_route_codes`) because its
//! selection is CONTENT-dependent. `msa-structured-selector/1` has no
//! analogous fit: its classification is `candidate_id mod 11` — a pure
//! function of POSITION, never of content
//! (`uor-r4-graph-format::msa_selector` module docs). So this harness
//! feeds `candidate_id = position` directly into the deployed packed
//! kernel over the SAME causal-prefix candidate table
//! `r4-route-attention/1` selects over — no projection, no threshold,
//! no learned parameter of any kind.
//!
//! ## Same evaluation semantics as #605
//!
//! Both arms restrict the teacher's own per-head softmax weights to
//! their selected top-M support and renormalize
//! (`crate::route_fit_report::preregistered_route_fit_contract`) —
//! every attention head of the fixture teacher. `msa-structured-
//! selector/1` selects identically for every head (position-only
//! classification), which is itself a measured property of the
//! operator, not a harness shortcut.
//!
//! ## The unigram floor (#457 convention)
//!
//! `#457` established the "unigram floor" (root-prior null) as the
//! bits/token and top-1 accuracy of a context-blind predictor that
//! always emits the corpus's own marginal next-token distribution —
//! computed HERE over this corpus's own `next` tokens (#457's own
//! numbers, `8.5635` bits / `0.0620` top-1, are pinned to a different
//! fixture and are not reused directly; the METHOD is reused).
//! Top-1 under a fixed marginal is the modal token's frequency;
//! bits/token is the marginal's own self-entropy (cross-entropy of the
//! empirical distribution against itself).
//!
//! ## Pre-registered exit rule (restated from `msa_selector_643.rs`)
//!
//! POSITIVE iff, on this corpus and candidate set, `msa-structured-
//! selector/1`'s whole-model top-1 >= `r4-route-attention/1`'s
//! whole-model top-1 + 0.02 absolute, AND does not increase bits/token
//! versus `r4-route-attention/1`, AND its bits/token beats (is strictly
//! lower than) the unigram floor computed above. NEGATIVE (including
//! "no measurable difference") closes #643 with the comparison table;
//! `msa-structured-selector-dormant` (already registered in
//! `model/ledger.toml`) is the disposition, unchanged.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use uor_r4_graph_compiler::route_fit::{
    fit_route_codes, FittedRouteCodes, RouteTraceCorpus, SyntheticRouteTeacher,
};
use uor_r4_graph_format::route_attention::{
    build_route_attention_instance, RouteAttentionView, RouteOpCensus, ROUTE_CODE_BYTES,
    ROUTE_MAX_CANDIDATES,
};
use uor_r4_graph_format::{
    build_msa_selector_instance, MsaSelectorOpCensus, MsaSelectorView, ScoreQ, MSA_MAX_TOP_M,
};
use uor_r4_graph_runtime::msa_selector::{msa_selector_step, MsaSelectorState};
use uor_r4_graph_runtime::route_attention::{route_attention_step, RouteState};
use uor_r4_model_source::SourceUnavailable;

use crate::route_fit_report::{
    preregistered_route_fit_contract, scope_metrics, ReplacedHead, RunContract, STAGE_WHOLE_MODEL,
};
use crate::score::GateCMetrics;

/// Schema tag of the canonical #643 A/B comparison report.
pub const MSA_AB_REPORT_SCHEMA: &str = "uor-r4-msa-ab-report/1";

/// One arm's measured numbers at the compared scope.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArmMetrics {
    /// Teacher-forced top-1 agreement with the recorded teacher argmax.
    pub top1_agreement: f64,
    /// Base-2 cross-entropy against the recorded targets (bits/token).
    pub bits_per_token: f64,
}

/// The #457-convention unigram (root-prior) floor of a corpus: the
/// bits/token and top-1 accuracy of a context-blind predictor emitting
/// the corpus's own marginal `next`-token distribution at every
/// position.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UnigramFloor {
    /// Modal token frequency (`max_count / total`).
    pub top1: f64,
    /// Marginal self-entropy in bits.
    pub bits_per_token: f64,
    /// Positions the marginal was fit over.
    pub positions: u64,
}

/// The three pre-registered gate evaluations, each independently named
/// so a reader never has to recompute them from the raw numbers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GateEvaluations {
    /// `msa.top1 >= route.top1 + 0.02`.
    pub top1_margin_gate: bool,
    /// `msa.bits_per_token <= route.bits_per_token`.
    pub bits_not_worse_gate: bool,
    /// `msa.bits_per_token < unigram.bits_per_token`.
    pub beats_unigram_floor_gate: bool,
}

/// The canonical #643 A/B comparison report: both arms' measured
/// numbers, the unigram floor, the three gate evaluations, and the
/// binding verdict, all computed on one held-out corpus generation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MsaAbReport {
    /// [`MSA_AB_REPORT_SCHEMA`].
    pub schema: String,
    /// κ of the merged observation-record bytes of the shared corpus.
    pub corpus_kappa: String,
    /// `r4-route-attention/1`, fitted (`route-fit/1`), whole-model scope.
    pub route_attention: ArmMetrics,
    /// `msa-structured-selector/1`, unfitted (position-only), same scope.
    pub msa_selector: ArmMetrics,
    /// The #457-convention unigram floor of this corpus.
    pub unigram: UnigramFloor,
    /// The three pre-registered gates.
    pub gates: GateEvaluations,
    /// `POSITIVE` iff every gate in `gates` holds; `NEGATIVE` otherwise
    /// (the exit rule's stated default, including "no measurable
    /// difference").
    pub verdict: String,
}

const FULL_MASK: [u8; ROUTE_CODE_BYTES] = [0xff; ROUTE_CODE_BYTES];

/// `[story][pos] -> selected candidate positions`, keyed by
/// `(layer, head)` — the same shape
/// `route_fit_report::scope_metrics` consumes.
type HeadSelections = BTreeMap<(u32, u32), Vec<Vec<Vec<u32>>>>;

/// Drive the deployed `r4-route-attention/1` packed kernel over one
/// fitted head's codes, returning `[story][pos] -> selected positions`.
/// Mirrors `route_fit_report`'s private `packed_selection_evidence`
/// selection loop (the witness/crosscheck arms there are cross-checked
/// exhaustively by the #605/#604 test suites already; this harness only
/// needs the resulting selections for a metrics comparison).
fn route_selections_for_head(
    key_codes: &[Vec<[u8; ROUTE_CODE_BYTES]>],
    query_codes: &[Vec<[u8; ROUTE_CODE_BYTES]>],
    top_m: u32,
) -> Result<Vec<Vec<Vec<u32>>>, SourceUnavailable> {
    let zeros = vec![ScoreQ::ZERO; ROUTE_MAX_CANDIDATES];
    let mut state = RouteState::new();
    let mut out = Vec::with_capacity(key_codes.len());
    for (story_keys, story_queries) in key_codes.iter().zip(query_codes.iter()) {
        let mut story_selected = Vec::with_capacity(story_keys.len());
        for (pos, query) in story_queries.iter().enumerate() {
            let candidates = &story_keys[..=pos];
            let n = candidates.len();
            let m = top_m.min(n as u32);
            let instance = build_route_attention_instance(&FULL_MASK, candidates, &zeros[..n], m)
                .map_err(|error| {
                SourceUnavailable::new(format!(
                    "route instance construction failed at pos {pos}: {error}"
                ))
            })?;
            let view = RouteAttentionView::parse(&instance).map_err(|error| {
                SourceUnavailable::new(format!("route instance parse failed: {error}"))
            })?;
            let mut census = RouteOpCensus::default();
            route_attention_step(&view, query, &mut state, &mut census).map_err(|error| {
                SourceUnavailable::new(format!("packed route step failed at pos {pos}: {error}"))
            })?;
            let mut selected = Vec::with_capacity(m as usize);
            let mut slot = 0usize;
            while let Some((candidate, _distance)) = state.selected(slot) {
                selected.push(candidate);
                slot += 1;
            }
            story_selected.push(selected);
        }
        out.push(story_selected);
    }
    Ok(out)
}

/// The whole-model selections of the fitted `r4-route-attention/1`
/// arm, keyed by `(layer, head)`.
fn route_attention_selections(
    fitted: &FittedRouteCodes,
) -> Result<HeadSelections, SourceUnavailable> {
    let mut out = BTreeMap::new();
    for head in &fitted.heads {
        let selections =
            route_selections_for_head(&head.key_codes, &head.query_codes, fitted.top_m)?;
        out.insert((head.layer, head.head), selections);
    }
    Ok(out)
}

/// Drive the deployed `msa-structured-selector/1` packed kernel over
/// the SAME causal-prefix candidate table (`candidate_id = position`,
/// no fitting), returning `[story][pos] -> selected positions`. One
/// selection set: the classification is position-only, so it is
/// identical for every `(layer, head)` by construction (module docs) —
/// duplicated across every scope head below, never recomputed.
fn msa_selector_selections(
    corpus: &RouteTraceCorpus,
) -> Result<Vec<Vec<Vec<u32>>>, SourceUnavailable> {
    let mut state = MsaSelectorState::new();
    let mut out = Vec::with_capacity(corpus.stories.len());
    for story in &corpus.stories {
        let mut story_selected = Vec::with_capacity(story.steps.len());
        for pos in 0..story.steps.len() {
            let n = pos + 1;
            let candidate_ids: Vec<u32> = (0..n as u32).collect();
            let contributions = vec![ScoreQ::ZERO; n];
            let top_m = (MSA_MAX_TOP_M as u32).min(n as u32);
            let instance = build_msa_selector_instance(&candidate_ids, &contributions, top_m)
                .map_err(|error| {
                    SourceUnavailable::new(format!(
                        "msa instance construction failed at pos {pos}: {error}"
                    ))
                })?;
            let view = MsaSelectorView::parse(&instance).map_err(|error| {
                SourceUnavailable::new(format!("msa instance parse failed: {error}"))
            })?;
            let mut census = MsaSelectorOpCensus::default();
            msa_selector_step(&view, &mut state, &mut census);
            let mut selected = Vec::with_capacity(top_m as usize);
            let mut slot = 0usize;
            while let Some((candidate, _role, _cascade)) = state.selected(slot) {
                selected.push(candidate);
                slot += 1;
            }
            story_selected.push(selected);
        }
        out.push(story_selected);
    }
    Ok(out)
}

/// The #457-convention unigram floor of a corpus: fit the empirical
/// marginal of `next` tokens over every story/position (fixed
/// story-ascending, position-ascending order — deterministic), then
/// report the modal token's frequency (top-1) and the marginal's own
/// self-entropy in bits (bits/token of a predictor that always emits
/// this fixed distribution).
fn unigram_floor(corpus: &RouteTraceCorpus) -> UnigramFloor {
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    let mut positions: u64 = 0;
    for story in &corpus.stories {
        for step in &story.steps {
            *counts.entry(step.next).or_insert(0) += 1;
            positions += 1;
        }
    }
    if positions == 0 {
        return UnigramFloor::default();
    }
    let total = positions as f64;
    let mut top1 = 0.0f64;
    // Marginal self-entropy: sum_j -p_j * log2(p_j) — the average
    // per-token surprisal of a predictor that always emits this fixed
    // distribution (NOT scaled by count again; p_j already carries the
    // 1/total weighting).
    let mut bits = 0.0f64;
    for &count in counts.values() {
        let p = count as f64 / total;
        if p > top1 {
            top1 = p;
        }
        bits += -p * p.log2();
    }
    UnigramFloor {
        top1,
        bits_per_token: bits,
        positions,
    }
}

/// Run the pre-registered #643 A/B comparison: generate the shared
/// synthetic corpus (the #605 production trace pipeline, into `dir`),
/// fit `r4-route-attention/1` (`route-fit/1`), drive both deployed
/// packed kernels over the identical whole-model scope, and evaluate
/// the three pre-registered gates. Deterministic: two runs (fresh
/// `dir`) produce a byte-identical report.
pub fn run_msa_ab_comparison(dir: &Path) -> Result<MsaAbReport, SourceUnavailable> {
    use uor_r4_graph_compiler::route_fit::{
        generate_synthetic_route_trace, load_route_trace_corpus, synthetic_capture_geometry,
    };
    use uor_r4_model_source::TeacherOracle;

    generate_synthetic_route_trace(dir)?;
    let mut teacher = SyntheticRouteTeacher::new();
    let corpus = load_route_trace_corpus(
        dir,
        synthetic_capture_geometry(),
        teacher.bos_token() as u32,
    )?;
    let fitted = fit_route_codes(&corpus)?;

    let contract: RunContract = preregistered_route_fit_contract();
    let whole_model_scope: Vec<ReplacedHead> = contract
        .stages
        .iter()
        .find(|stage| stage.stage == STAGE_WHOLE_MODEL)
        .map(|stage| stage.replaced.clone())
        .ok_or_else(|| SourceUnavailable::new("contract declares no whole-model stage"))?;
    let scope_heads: Vec<(u32, u32)> = whole_model_scope
        .iter()
        .map(|head| (head.layer, head.head))
        .collect();

    let route_selections = route_attention_selections(&fitted)?;
    let msa_positions = msa_selector_selections(&corpus)?;
    let mut msa_selections: HeadSelections = BTreeMap::new();
    for key in &scope_heads {
        msa_selections.insert(*key, msa_positions.clone());
    }

    let route_metrics: GateCMetrics = scope_metrics(
        &mut teacher,
        &corpus,
        &whole_model_scope,
        &route_selections,
        &contract,
    )
    .replaced;
    let msa_metrics: GateCMetrics = scope_metrics(
        &mut teacher,
        &corpus,
        &whole_model_scope,
        &msa_selections,
        &contract,
    )
    .replaced;
    let unigram = unigram_floor(&corpus);

    let route_arm = ArmMetrics {
        top1_agreement: route_metrics.top1_agreement,
        bits_per_token: route_metrics.bits_per_token,
    };
    let msa_arm = ArmMetrics {
        top1_agreement: msa_metrics.top1_agreement,
        bits_per_token: msa_metrics.bits_per_token,
    };

    let gates = GateEvaluations {
        top1_margin_gate: msa_arm.top1_agreement >= route_arm.top1_agreement + 0.02,
        bits_not_worse_gate: msa_arm.bits_per_token <= route_arm.bits_per_token,
        beats_unigram_floor_gate: msa_arm.bits_per_token < unigram.bits_per_token,
    };
    let verdict =
        if gates.top1_margin_gate && gates.bits_not_worse_gate && gates.beats_unigram_floor_gate {
            "POSITIVE"
        } else {
            "NEGATIVE"
        }
        .to_owned();

    Ok(MsaAbReport {
        schema: MSA_AB_REPORT_SCHEMA.to_owned(),
        corpus_kappa: corpus.records_kappa.clone(),
        route_attention: route_arm,
        msa_selector: msa_arm,
        unigram,
        gates,
        verdict,
    })
}

/// Canonical report bytes (ciborium, struct-declaration field order —
/// the certify crate's existing serde byte-format convention).
pub fn canonical_msa_ab_report_bytes(report: &MsaAbReport) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(report, &mut bytes).expect("msa-ab report serializes to canonical bytes");
    bytes
}

/// The report κ: `blake3:<hex>` over the canonical report bytes.
pub fn msa_ab_report_kappa(report: &MsaAbReport) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_msa_ab_report_bytes(report)).to_hex()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("uor-r4-{name}-{nanos}"))
    }

    #[test]
    fn unigram_floor_is_the_modal_frequency_and_self_entropy() {
        // Three tokens, hand counted: 5x token 0, 3x token 1, 2x token 2.
        let corpus = RouteTraceCorpus {
            geometry: uor_r4_graph_compiler::route_fit::synthetic_capture_geometry(),
            declared_layers: vec![0],
            support_size: 1,
            trace_profile: uor_r4_graph_compiler::route_fit::synthetic_trace_profile(),
            stories: vec![uor_r4_graph_compiler::route_fit::StoryTrace {
                story: 0,
                tokens: vec![0; 10],
                steps: [0u32, 0, 0, 0, 0, 1, 1, 1, 2, 2]
                    .into_iter()
                    .enumerate()
                    .map(|(pos, next)| uor_r4_graph_compiler::route_fit::StepTrace {
                        pos: pos as u32,
                        input_token: 0,
                        next,
                        top_tokens: [0; 8],
                        target_logprob_nats: 0.0,
                        q_rows: vec![],
                        k_rows: vec![],
                        supports: vec![],
                    })
                    .collect(),
            }],
            records: 10,
            records_kappa: String::new(),
            trace_kappa: String::new(),
            identity_bundle_digest: String::new(),
        };
        let floor = unigram_floor(&corpus);
        assert_eq!(floor.positions, 10);
        assert!(
            (floor.top1 - 0.5).abs() < 1e-12,
            "5/10 is the modal frequency"
        );
        let expected_bits = -(0.5 * 0.5f64.log2() + 0.3 * 0.3f64.log2() + 0.2 * 0.2f64.log2());
        assert!((floor.bits_per_token - expected_bits).abs() < 1e-12);
    }

    #[test]
    fn ab_comparison_is_deterministic_and_reports_every_field() {
        let dir_a = unique_path("msa-ab-a");
        let report_a = run_msa_ab_comparison(&dir_a).expect("comparison a");
        let _ = std::fs::remove_dir_all(&dir_a);

        let dir_b = unique_path("msa-ab-b");
        let report_b = run_msa_ab_comparison(&dir_b).expect("comparison b");
        let _ = std::fs::remove_dir_all(&dir_b);

        assert_eq!(
            report_a, report_b,
            "two independent runs are byte-identical"
        );
        assert_eq!(report_a.schema, MSA_AB_REPORT_SCHEMA);
        assert!(!report_a.corpus_kappa.is_empty());
        assert!(report_a.route_attention.top1_agreement > 0.0);
        assert!(report_a.msa_selector.top1_agreement >= 0.0);
        assert!(report_a.unigram.positions > 0);
        assert!(report_a.verdict == "POSITIVE" || report_a.verdict == "NEGATIVE");
        // The verdict is exactly the conjunction of the three gates.
        let expected_verdict = if report_a.gates.top1_margin_gate
            && report_a.gates.bits_not_worse_gate
            && report_a.gates.beats_unigram_floor_gate
        {
            "POSITIVE"
        } else {
            "NEGATIVE"
        };
        assert_eq!(report_a.verdict, expected_verdict);
        assert_eq!(
            msa_ab_report_kappa(&report_a),
            msa_ab_report_kappa(&report_b)
        );
    }

    /// Measured values at pin time (2026-08-16 fixture, seed constants as
    /// shipped) — the actual #643 pre-registered A/B run:
    ///
    /// ```text
    /// r4-route-attention/1 (fitted, whole-model): top1 0.990234  bits/token 4.971638
    /// msa-structured-selector/1 (position-only):  top1 0.642578  bits/token 5.532591
    /// unigram floor (this corpus):                top1 0.035156  bits/token 5.854087
    /// gates: top1_margin=false  bits_not_worse=false  beats_unigram=true
    /// verdict: NEGATIVE
    /// ```
    ///
    /// This is the actual, honest result of the pre-registered #643 exit
    /// rule: `msa-structured-selector/1`'s classification is position-only
    /// (`candidate_id mod 11`, module docs) and carries no signal about
    /// this synthetic teacher's content-directional attention, so it
    /// clears the trivial unigram floor by a wide margin but falls far
    /// short of the fitted, content-aware `r4-route-attention/1` arm —
    /// consistent with #643's own pre-registered mod-12 probe-chain
    /// warning (posted to the issue before implementation): a class's
    /// representation capacity does not, by itself, survive selection
    /// dynamics without content-aware encoding. The pinned
    /// assertions below reflect THESE measured numbers; the consistency
    /// assertions above (determinism, gate/verdict conjunction) would
    /// hold under either outcome.
    #[test]
    fn msa_selector_measured_negative_against_fitted_route_attention() {
        let dir = unique_path("msa-ab-pinned");
        let report = run_msa_ab_comparison(&dir).expect("comparison");
        let _ = std::fs::remove_dir_all(&dir);

        let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(close(report.route_attention.top1_agreement, 0.990234));
        assert!(close(report.route_attention.bits_per_token, 4.971638));
        assert!(close(report.msa_selector.top1_agreement, 0.642578));
        assert!(close(report.msa_selector.bits_per_token, 5.532591));
        assert!(close(report.unigram.top1, 0.035156));
        assert!(close(report.unigram.bits_per_token, 5.854087));
        assert!(!report.gates.top1_margin_gate);
        assert!(!report.gates.bits_not_worse_gate);
        assert!(report.gates.beats_unigram_floor_gate);
        assert_eq!(report.verdict, "NEGATIVE");
    }
}
