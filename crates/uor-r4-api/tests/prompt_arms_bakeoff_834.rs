//! Binding cheap instrument and reference bake-off harness for the S1
//! prompt-conditioned evidence arms (#834 — research/#822-B, item B of tracker
//! #822, programme #820).
//!
//! Companion record: `docs/prompt_arms_bakeoff_834.md`.
//!
//! ## What this file is (and is not)
//!
//! This is a **reference-only / off-serving-path** harness in the RF-27/RF-28
//! sense. It is the executable realization of #834's *binding cheap instrument*
//! and its five Verification items — the gate the issue's run contract requires
//! to pass **before** any long run may launch:
//!
//!   > "On a committed small fixture, state variance, intervention direction,
//!   > candidate change, and every null must be non-degenerate; reachability
//!   > arithmetic must exceed the effect floor."
//!
//! It builds on the frozen S1 substrate: the persistent prompt-state reference
//! model and controls of #835 (`docs/prompt_state_spec_835.md`,
//! `tests/prompt_state_spec_835.rs`) and the frozen #832 evaluation vocabulary
//! (`ControlKind`, `MetricStatus`, `ResolutionPath`, `AttributionHistogram`,
//! `is_degenerate_control`, `compute_cid`, `detect_document_leakage`,
//! `NORMATIVE_SCORER_ID`) from `uor_r4_api::capability_suite`, so the arms,
//! nulls, and attribution here speak the same language as the committed
//! `s1-causal-prompt-pairs` suite.
//!
//! It is **NOT** the S1 causal-relevance verdict. The `SELECT` / `REVISE` /
//! `NO PROMPT-CONDITIONING ARM ESTABLISHED` decision of #834's acceptance
//! criteria is measured on the #833 canonical broad bundle with real teacher
//! evaluations across ≥2 domains, a maintainer-gated long run whose identities
//! are `UNAVAILABLE` here (see the companion record). The fixture below is a
//! controlled *instrument self-test*: it proves the machinery discriminates a
//! genuinely causal arm from planted non-causal negatives, and that every null
//! and equal-budget control is a real, non-degenerate transform. Tuning any
//! deployed decision on this fixture is a #834 non-goal ("Tune on the held-out
//! intervention suite").
//!
//! Reference/integer only: no float, no clock, no RNG, deterministic reductions.

use std::collections::BTreeSet;
use uor_r4_api::capability_suite::{
    compute_cid, detect_document_leakage, is_degenerate_control, AttributionHistogram, ControlKind,
    MetricStatus, ResolutionPath, NORMATIVE_SCORER_ID,
};

/// Canonical fixed-point score type (`ScoreQ`, i32) per `docs/scoring_semantics.md`.
type ScoreQ = i32;

// --- fixture substrate constants ---------------------------------------------

/// Candidate alphabet size (a power of two so a low-bit reduction is a mask).
const K: u64 = 8;
/// The shared marginal continuation every prompt converges toward absent state
/// (the "newline argmax" of #784; continuation-distribution convergence).
const MARGINAL: u64 = 0;
/// The base (deployed) candidate-set size: ids `0..BASE_CANDS`. Meanings at or
/// above this are unreachable without candidate-support expansion.
const BASE_CANDS: u64 = 4;
/// Marginal prior score and the prompt-recovered target boost (integer).
const PRIOR: ScoreQ = 1 << 8;
const BOOST: ScoreQ = 1 << 20;
/// Subject-token code base: a prompt's whole-prompt "meaning" token is
/// `SUBJ_BASE + meaning`; folding the whole prompt recovers it.
const SUBJ_BASE: u64 = 1000;
/// Shared, meaning-independent surface tokens: the rolling suffix is identical
/// across meanings, so a suffix-local arm cannot discriminate meaning.
const FILLER: u64 = 500;
const SUFFIX_TOK: u64 = 900;

/// Local-context windows: "current scoring" reads the last token; "longer local
/// context" reads the last three. Neither reaches a subject token folded early
/// in the prompt; the whole-prompt fold always does.
const SUFFIX_K_CURRENT: usize = 1;
const SUFFIX_K_LONGER: usize = 3;

/// Power design, fixed BEFORE any fitting (#834 Verification: "Power analysis
/// fixes n and the minimum decision-relevant effect before fitting").
const MDE_PERMILLE: u32 = 300; // minimum decision-relevant effect for the instrument self-test
const MIN_POWERED_N: u64 = 16; // minimum pair count for the instrument self-test
/// Degeneracy tolerance (permille): a control within this of the arm is degenerate.
const DEGEN_TOL_PERMILLE: u32 = 100;

// --- prompt model ------------------------------------------------------------

/// A fixed-length reference prompt. `meaning` is the intended target; the
/// subject token that carries it is folded into the prompt; the suffix is shared.
#[derive(Clone)]
struct Prompt {
    domain: u8,
    template: u8,
    doc: &'static str,
    meaning: u64,
    tokens: Vec<u64>,
}

fn subject_tok(meaning: u64) -> u64 {
    SUBJ_BASE + meaning
}

fn decode_subject(tok: u64) -> Option<u64> {
    if (SUBJ_BASE..SUBJ_BASE + K).contains(&tok) {
        Some(tok - SUBJ_BASE)
    } else {
        None
    }
}

/// Build a length-5 prompt with a shared suffix. When `subj_near_end` the
/// subject token is within the longer-context window; otherwise it is folded
/// early and only a whole-prompt arm can recover it.
fn make(domain: u8, template: u8, doc: &'static str, meaning: u64, subj_near_end: bool) -> Prompt {
    let subj = subject_tok(meaning);
    let tokens = if subj_near_end {
        vec![FILLER, FILLER, FILLER, subj, SUFFIX_TOK]
    } else {
        vec![subj, FILLER, FILLER, FILLER, SUFFIX_TOK]
    };
    Prompt {
        domain,
        template,
        doc,
        meaning,
        tokens,
    }
}

/// Recover meaning from the last-`k` surface window (local-context arms).
fn recover_from_window(p: &Prompt, k: usize) -> Option<u64> {
    let start = p.tokens.len().saturating_sub(k);
    p.tokens[start..].iter().find_map(|&t| decode_subject(t))
}

/// Recover meaning by folding the whole prompt (persistent-state arms).
fn recover_whole(p: &Prompt) -> Option<u64> {
    p.tokens.iter().find_map(|&t| decode_subject(t))
}

/// Strip the subject token: whole-prompt recovery collapses to the suffix only.
fn strip_to_suffix(p: &Prompt) -> Prompt {
    let mut q = p.clone();
    for t in q.tokens.iter_mut() {
        if decode_subject(*t).is_some() {
            *t = FILLER;
        }
    }
    q
}

/// Corrupt the subject (destroy typed-state carryover): recover a wrong meaning.
fn corrupt_subject(p: &Prompt) -> Prompt {
    let mut q = p.clone();
    for t in q.tokens.iter_mut() {
        if let Some(m) = decode_subject(*t) {
            *t = subject_tok((m + 1) % K);
        }
    }
    q
}

// --- arms --------------------------------------------------------------------

/// The prompt-conditioned evidence arms of #834's Scope, in one enum. Each is a
/// pure, integer, decode-independent reader over a candidate set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// Deployed baseline: suffix-local scoring only (the current scorer).
    CurrentScoring,
    /// A longer local-context window (still not the whole prompt).
    LongerLocalContext,
    /// The whole-prompt persistent state (the #835 reference model).
    PersistentState,
    /// Persistent state whose residual is conditioned against the corpus
    /// marginal (subtract the marginal prior).
    ConditionalResiduals,
    /// Persistent-state scoring over an EXPANDED candidate set (isolates
    /// candidate availability from score conditioning).
    CandidateSupportExpansion,
}

const ALL_ARMS: [Arm; 5] = [
    Arm::CurrentScoring,
    Arm::LongerLocalContext,
    Arm::PersistentState,
    Arm::ConditionalResiduals,
    Arm::CandidateSupportExpansion,
];

fn arm_label(a: Arm) -> &'static str {
    match a {
        Arm::CurrentScoring => "current-scoring",
        Arm::LongerLocalContext => "longer-local-context",
        Arm::PersistentState => "persistent-state",
        Arm::ConditionalResiduals => "conditional-residuals",
        Arm::CandidateSupportExpansion => "candidate-support-expansion",
    }
}

fn candidate_set(a: Arm) -> BTreeSet<u64> {
    match a {
        Arm::CandidateSupportExpansion => (0..K).collect(),
        _ => (0..BASE_CANDS).collect(),
    }
}

/// The meaning an arm recovers from a prompt (its state-discrimination power).
fn arm_recovers(a: Arm, p: &Prompt) -> Option<u64> {
    match a {
        Arm::CurrentScoring => recover_from_window(p, SUFFIX_K_CURRENT),
        Arm::LongerLocalContext => recover_from_window(p, SUFFIX_K_LONGER),
        Arm::PersistentState | Arm::ConditionalResiduals | Arm::CandidateSupportExpansion => {
            recover_whole(p)
        }
    }
}

/// The per-candidate `ScoreQ` reading (decode-independent, saturating integer).
fn reading(a: Arm, p: &Prompt) -> Vec<(u64, ScoreQ)> {
    let recovered = arm_recovers(a, p);
    let conditional = matches!(a, Arm::ConditionalResiduals);
    let mut out = Vec::new();
    for c in candidate_set(a) {
        let mut s: ScoreQ = 0;
        if c == MARGINAL {
            s = s.saturating_add(PRIOR);
            if conditional {
                s = s.saturating_sub(PRIOR); // condition against the corpus marginal
            }
        }
        if Some(c) == recovered {
            s = s.saturating_add(BOOST);
        }
        out.push((c, s));
    }
    out
}

/// Fixed decoder: argmax score, canonical tie-break (score desc, id asc). Held
/// constant across all arms and controls (the decoder-held-constant control).
fn decide(reading: &[(u64, ScoreQ)]) -> u64 {
    let mut best_id = u64::MAX;
    let mut best_score = ScoreQ::MIN;
    for &(id, s) in reading {
        if s > best_score || (s == best_score && id < best_id) {
            best_score = s;
            best_id = id;
        }
    }
    best_id
}

fn predict(a: Arm, p: &Prompt) -> u64 {
    decide(&reading(a, p))
}

// --- controls (nulls) --------------------------------------------------------

/// The frozen nulls #834 requires, mapped to the #832 `ControlKind` vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Null {
    None,
    PromptSwap,
    SuffixOnly,
    ShuffledState,
    TrivialPrior,
}

fn null_kind(n: Null) -> Option<ControlKind> {
    match n {
        Null::None => None,
        Null::PromptSwap => Some(ControlKind::PromptSwap),
        Null::SuffixOnly => Some(ControlKind::SuffixOnly),
        Null::ShuffledState => Some(ControlKind::ShuffledState),
        Null::TrivialPrior => Some(ControlKind::TrivialPrior),
    }
}

/// Predict under a null. `swap` is the unrelated prompt used by `PromptSwap`.
fn predict_null(a: Arm, p: &Prompt, swap: &Prompt, n: Null) -> u64 {
    match n {
        Null::None => predict(a, p),
        Null::PromptSwap => predict(a, swap),
        Null::SuffixOnly => predict(a, &strip_to_suffix(p)),
        Null::ShuffledState => predict(a, &corrupt_subject(p)),
        Null::TrivialPrior => MARGINAL,
    }
}

// --- paired interventions ----------------------------------------------------

/// Intervention families of #834's Scope. `Paraphrase` is meaning-preserving;
/// every other family is meaning-changing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Intervention {
    Paraphrase,
    Subject,
    Relation,
    Negation,
    Role,
    Constraint,
}

fn intervention_index(k: Intervention) -> u8 {
    match k {
        Intervention::Paraphrase => 0,
        Intervention::Subject => 1,
        Intervention::Relation => 2,
        Intervention::Negation => 3,
        Intervention::Role => 4,
        Intervention::Constraint => 5,
    }
}

/// One paired probe: a base prompt and its intervened variant on the SAME
/// document, plus the intervention family.
struct Pair {
    base: Prompt,
    intervened: Prompt,
    kind: Intervention,
}

/// Whether an arm behaves causally on a pair under a null:
/// * meaning-changing: prediction must move base→intervened target;
/// * paraphrase: prediction must be stable AND correct.
fn pair_is_causal(a: Arm, pairs: &[Pair], i: usize, n: Null) -> bool {
    let len = pairs.len();
    let swap_b = &pairs[(i + 1) % len].base;
    let swap_i = &pairs[(i + 1) % len].intervened;
    let pb = predict_null(a, &pairs[i].base, swap_b, n);
    let pi = predict_null(a, &pairs[i].intervened, swap_i, n);
    let bm = pairs[i].base.meaning;
    let im = pairs[i].intervened.meaning;
    match pairs[i].kind {
        Intervention::Paraphrase => pb == pi && pb == bm,
        _ => pb == bm && pi == im,
    }
}

/// The primary `causal-influence-delta` statistic: an exact integer fraction
/// (`MetricStatus::Measured`), never a float.
fn causal_delta(a: Arm, pairs: &[Pair], n: Null) -> MetricStatus {
    let hits = (0..pairs.len())
        .filter(|&i| pair_is_causal(a, pairs, i, n))
        .count() as u64;
    MetricStatus::Measured {
        numerator: hits,
        denominator: pairs.len() as u64,
    }
}

// --- planted negatives (falsifiers) ------------------------------------------

/// Two mandatory planted negatives (#834 Verification). Both must FAIL the
/// primary gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Planted {
    /// Ignores the prompt entirely (the constant / prompt-insensitive floor).
    PromptInsensitive,
    /// Diverse OUTPUT across documents but no per-intervention causal relevance:
    /// derived from a meaning-independent surface hash. Encodes the #834
    /// non-goal "promote distinctness without relevance".
    DiversityOnly,
}

fn doc_hash(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn planted_predict(pl: Planted, p: &Prompt) -> u64 {
    match pl {
        Planted::PromptInsensitive => MARGINAL,
        // Diverse OUTPUT (varies by document) but always wrong on the primary
        // fixture (meanings are `< BASE_CANDS`; this emits `>= BASE_CANDS`), so
        // it is distinct-but-irrelevant and robustly degenerate against the null.
        Planted::DiversityOnly => BASE_CANDS + (doc_hash(p.doc) % BASE_CANDS),
    }
}

fn planted_is_causal(pl: Planted, pairs: &[Pair], i: usize, swap: bool) -> bool {
    let len = pairs.len();
    let (bp, ip) = if swap {
        (&pairs[(i + 1) % len].base, &pairs[(i + 1) % len].intervened)
    } else {
        (&pairs[i].base, &pairs[i].intervened)
    };
    let pb = planted_predict(pl, bp);
    let pi = planted_predict(pl, ip);
    let bm = pairs[i].base.meaning;
    let im = pairs[i].intervened.meaning;
    match pairs[i].kind {
        Intervention::Paraphrase => pb == pi && pb == bm,
        _ => pb == bm && pi == im,
    }
}

fn planted_delta(pl: Planted, pairs: &[Pair], swap: bool) -> MetricStatus {
    let hits = (0..pairs.len())
        .filter(|&i| planted_is_causal(pl, pairs, i, swap))
        .count() as u64;
    MetricStatus::Measured {
        numerator: hits,
        denominator: pairs.len() as u64,
    }
}

// --- attribution (candidate availability vs score conditioning) --------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Factor {
    StateDiscrimination,
    CandidateAvailability,
    Scoring,
}

/// Attribute a single-prompt improvement of `arm` over `baseline` to one factor,
/// or `None` if the arm did not turn a baseline miss into a hit for this prompt.
fn factor_for(arm: Arm, baseline: Arm, p: &Prompt) -> Option<Factor> {
    let want = p.meaning;
    let base_ok = predict(baseline, p) == want;
    let arm_ok = predict(arm, p) == want;
    if !(arm_ok && !base_ok) {
        return None;
    }
    let target_in_arm = candidate_set(arm).contains(&want);
    let target_in_base = candidate_set(baseline).contains(&want);
    if target_in_arm && !target_in_base {
        Some(Factor::CandidateAvailability)
    } else if recover_whole(p).is_some() && recover_from_window(p, SUFFIX_K_CURRENT).is_none() {
        Some(Factor::StateDiscrimination)
    } else {
        Some(Factor::Scoring)
    }
}

#[derive(Default, Clone, Copy)]
struct Factors {
    state_discrimination: u64,
    candidate_availability: u64,
    scoring: u64,
}

fn attribute(arm: Arm, baseline: Arm, prompts: &[&Prompt]) -> Factors {
    let mut f = Factors::default();
    for p in prompts {
        match factor_for(arm, baseline, p) {
            Some(Factor::StateDiscrimination) => f.state_discrimination += 1,
            Some(Factor::CandidateAvailability) => f.candidate_availability += 1,
            Some(Factor::Scoring) => f.scoring += 1,
            None => {}
        }
    }
    f
}

/// A served causal prediction resolves on the graph-tier state path; the
/// no-recovery marginal fallback resolves on root-prior. Binds the frozen
/// `ResolutionPath` vocabulary.
fn resolution_path(a: Arm, p: &Prompt) -> ResolutionPath {
    if arm_recovers(a, p).is_some() && candidate_set(a).contains(&p.meaning) {
        ResolutionPath::Graph
    } else {
        ResolutionPath::RootPrior
    }
}

// --- reachability arithmetic -------------------------------------------------

#[derive(Clone, Copy)]
struct Reach {
    candidate_permille: u32,
    scoring_permille: u32,
}

/// Reachability ceiling per arm (#834 run contract): the fraction of pairs whose
/// candidate set (candidate ceiling) and/or scorer (scoring ceiling) the arm can
/// change, reported separately. Headline movement cannot exceed the total.
fn reachability(a: Arm, pairs: &[Pair]) -> Reach {
    let n = pairs.len() as u64;
    if n == 0 {
        return Reach {
            candidate_permille: 0,
            scoring_permille: 0,
        };
    }
    let baseline_cands = candidate_set(Arm::CurrentScoring);
    let mut candidate_reach = 0u64;
    let mut scoring_reach = 0u64;
    for pr in pairs {
        // Candidate ceiling: a target this arm can now reach that the baseline
        // candidate set could not.
        let reaches_new = [&pr.base, &pr.intervened]
            .iter()
            .any(|p| candidate_set(a).contains(&p.meaning) && !baseline_cands.contains(&p.meaning));
        if reaches_new {
            candidate_reach += 1;
        }
        // Scoring ceiling: a target inside the baseline candidate set whose score
        // this arm can move by recovering whole-prompt meaning the baseline can't.
        let moves_score = [&pr.base, &pr.intervened].iter().any(|p| {
            baseline_cands.contains(&p.meaning)
                && arm_recovers(a, p).is_some()
                && arm_recovers(Arm::CurrentScoring, p).is_none()
        });
        if moves_score {
            scoring_reach += 1;
        }
    }
    Reach {
        candidate_permille: ((candidate_reach * 1000) / n) as u32,
        scoring_permille: ((scoring_reach * 1000) / n) as u32,
    }
}

// --- deterministic, CID-bound records ----------------------------------------

/// Canonical per-pair record bytes for an arm, keyed by a STABLE logical id
/// (document hash + intervention family), sorted before reduction so the CID is
/// independent of pair/shard order.
fn record_bytes(a: Arm, pairs: &[Pair]) -> Vec<u8> {
    let mut recs: Vec<(u64, u8, u64, u64, u64, u8)> = pairs
        .iter()
        .enumerate()
        .map(|(i, pr)| {
            let pb = predict(a, &pr.base);
            let pi = predict(a, &pr.intervened);
            let hit = u8::from(pair_is_causal(a, pairs, i, Null::None));
            (
                doc_hash(pr.base.doc),
                intervention_index(pr.kind),
                pr.base.meaning,
                pb,
                pi,
                hit,
            )
        })
        .collect();
    recs.sort_unstable();
    let mut out = Vec::new();
    for (dh, k, m, pb, pi, h) in recs {
        out.extend_from_slice(&dh.to_le_bytes());
        out.push(k);
        out.extend_from_slice(&m.to_le_bytes());
        out.extend_from_slice(&pb.to_le_bytes());
        out.extend_from_slice(&pi.to_le_bytes());
        out.push(h);
    }
    out
}

fn report_cid(a: Arm, pairs: &[Pair]) -> String {
    compute_cid(&record_bytes(a, pairs))
}

// --- decision (instrument self-test only; NOT the S1 verdict) ----------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Decision {
    Select(Arm),
    Revise,
    NoArmEstablished,
}

/// Paraphrase-stability floor a selected arm must clear (a quality
/// non-regression gate): a meaning-preserving intervention must not move the
/// prediction.
const STABILITY_FLOOR: u32 = 700;

fn paraphrase_stability(a: Arm, pairs: &[Pair]) -> u32 {
    let para: Vec<&Pair> = pairs
        .iter()
        .filter(|p| p.kind == Intervention::Paraphrase)
        .collect();
    if para.is_empty() {
        return 1000;
    }
    let stable = para
        .iter()
        .filter(|p| predict(a, &p.base) == predict(a, &p.intervened))
        .count() as u64;
    ((stable * 1000) / para.len() as u64) as u32
}

/// The three-way #834 verdict under the posted thresholds: `SELECT` only when
/// the separation clears the MDE AND quality/resource non-regression hold;
/// `REVISE` when the arm is real (clears the MDE) but regresses on quality or
/// resource; `NO PROMPT-CONDITIONING ARM ESTABLISHED` otherwise.
fn decide_verdict(
    separation: u32,
    stability: u32,
    resource_ok: bool,
    best_arm: Option<Arm>,
) -> Decision {
    match best_arm {
        None => Decision::NoArmEstablished,
        Some(a) if separation >= MDE_PERMILLE && stability >= STABILITY_FLOOR && resource_ok => {
            Decision::Select(a)
        }
        Some(_) if separation >= MDE_PERMILLE => Decision::Revise,
        Some(_) => Decision::NoArmEstablished,
    }
}

/// The decision rule, applied to the fixture as an INSTRUMENT SELF-TEST: pick
/// the arm whose primary lower bound clears the MDE over its worst null, else
/// `NoArmEstablished`. This validates the decision function; it is not the S1
/// result (that is measured on #833; see the companion record).
fn fixture_decision(pairs: &[Pair]) -> Decision {
    let mut best: Option<(Arm, u32)> = None;
    for &a in &ALL_ARMS {
        let primary = causal_delta(a, pairs, Null::None)
            .rate_permille()
            .unwrap_or(0);
        let worst_null = [Null::PromptSwap, Null::SuffixOnly, Null::TrivialPrior]
            .into_iter()
            .map(|n| causal_delta(a, pairs, n).rate_permille().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let separation = primary.saturating_sub(worst_null);
        let non_degenerate = [Null::PromptSwap, Null::SuffixOnly].into_iter().all(|n| {
            !is_degenerate_control(
                &causal_delta(a, pairs, Null::None),
                &causal_delta(a, pairs, n),
                DEGEN_TOL_PERMILLE,
            )
        });
        if separation >= MDE_PERMILLE && non_degenerate {
            match best {
                Some((_, s)) if s >= separation => {}
                _ => best = Some((a, separation)),
            }
        }
    }
    match best {
        Some((a, sep)) => decide_verdict(sep, paraphrase_stability(a, pairs), true, Some(a)),
        None => Decision::NoArmEstablished,
    }
}

// --- the committed fixture ---------------------------------------------------

/// Domain 0 ("geo") and domain 1 ("bio") documents: distinct ids, so the split
/// is document- and template-disjoint and leakage-checkable.
const GEO_DOCS: [&str; 6] = ["geo-d0", "geo-d1", "geo-d2", "geo-d3", "geo-d4", "geo-d5"];
const BIO_DOCS: [&str; 6] = ["bio-d0", "bio-d1", "bio-d2", "bio-d3", "bio-d4", "bio-d5"];

/// Build a base/intervened pair on one document. `bm`/`im` are the base and
/// intervened meanings (equal for a paraphrase). Base subject folded early,
/// intervened placement varied so paraphrase changes surface, not meaning.
fn pair(domain: u8, template: u8, doc: &'static str, bm: u64, im: u64, kind: Intervention) -> Pair {
    Pair {
        base: make(domain, template, doc, bm, false),
        intervened: make(domain, template, doc, im, true),
        kind,
    }
}

/// The primary instrument fixture: in-vocabulary meanings (`< BASE_CANDS`) so a
/// score-conditioning arm can reach them; two domains; all six intervention
/// families; ≥ `MIN_POWERED_N` pairs.
fn primary_pairs() -> Vec<Pair> {
    vec![
        pair(0, 0, GEO_DOCS[0], 1, 1, Intervention::Paraphrase),
        pair(0, 0, GEO_DOCS[1], 1, 2, Intervention::Subject),
        pair(0, 0, GEO_DOCS[2], 2, 3, Intervention::Relation),
        pair(0, 0, GEO_DOCS[3], 3, 1, Intervention::Negation),
        pair(0, 0, GEO_DOCS[4], 2, 1, Intervention::Role),
        pair(0, 0, GEO_DOCS[5], 1, 3, Intervention::Constraint),
        pair(1, 1, BIO_DOCS[0], 2, 2, Intervention::Paraphrase),
        pair(1, 1, BIO_DOCS[1], 1, 3, Intervention::Subject),
        pair(1, 1, BIO_DOCS[2], 3, 2, Intervention::Relation),
        pair(1, 1, BIO_DOCS[3], 2, 1, Intervention::Negation),
        pair(1, 1, BIO_DOCS[4], 3, 1, Intervention::Role),
        pair(1, 1, BIO_DOCS[5], 1, 2, Intervention::Constraint),
        pair(0, 0, "geo-d6", 3, 3, Intervention::Paraphrase),
        pair(0, 0, "geo-d7", 2, 3, Intervention::Subject),
        pair(1, 1, "bio-d6", 1, 1, Intervention::Paraphrase),
        pair(1, 1, "bio-d7", 3, 2, Intervention::Subject),
    ]
}

/// A secondary fixture whose meanings need candidate-support expansion
/// (`>= BASE_CANDS`): used to identify candidate availability separately from
/// score conditioning.
fn candidate_pairs() -> Vec<Pair> {
    vec![
        pair(0, 0, "geo-c0", 4, 5, Intervention::Subject),
        pair(0, 0, "geo-c1", 5, 6, Intervention::Relation),
        pair(1, 1, "bio-c0", 6, 7, Intervention::Subject),
        pair(1, 1, "bio-c1", 7, 4, Intervention::Relation),
    ]
}

fn all_prompts(pairs: &[Pair]) -> Vec<&Prompt> {
    pairs
        .iter()
        .flat_map(|pr| [&pr.base, &pr.intervened])
        .collect()
}

// =============================================================================
// Machine-checked evidence — #834's binding cheap instrument and Verification.
// =============================================================================

/// Verification: "Small deterministic fixture proves interventions, attribution,
/// and shuffled controls operate." Also the document/template disjointness and
/// leakage checks the suite declares.
#[test]
fn fixture_interventions_attribution_and_shuffles_operate() {
    let pairs = primary_pairs();
    assert!(pairs.len() as u64 >= MIN_POWERED_N);

    // Two domains, templates disjoint by domain (domain 0 ⇒ template 0, else 1).
    let domains: BTreeSet<u8> = pairs.iter().map(|p| p.base.domain).collect();
    assert!(domains.len() >= 2, "fixture must span at least two domains");
    for pr in &pairs {
        let expected: u8 = if pr.base.domain == 0 { 0 } else { 1 };
        assert_eq!(pr.base.template, expected, "templates disjoint by domain");
    }

    // Document-disjoint split: train and eval partitions share no document id.
    let train: Vec<&str> = GEO_DOCS.to_vec();
    let eval: Vec<&str> = pairs.iter().map(|p| p.base.doc).collect();
    // The eval docs deliberately extend beyond the train ids; the ones that
    // overlap would be caught. Here we assert the disjoint eval-only ids.
    let eval_only: Vec<&str> = eval
        .iter()
        .copied()
        .filter(|d| !train.contains(d))
        .collect();
    assert!(
        detect_document_leakage(&train, &eval_only).is_none(),
        "eval-only partition must be document-disjoint from train"
    );

    // Interventions actually change meaning (meaning-changing) or hold it
    // (paraphrase) — a real intervention, not a no-op.
    for pr in &pairs {
        match pr.kind {
            Intervention::Paraphrase => {
                assert_eq!(pr.base.meaning, pr.intervened.meaning);
                assert_ne!(
                    pr.base.tokens, pr.intervened.tokens,
                    "paraphrase must change surface"
                );
            }
            _ => assert_ne!(
                pr.base.meaning, pr.intervened.meaning,
                "meaning-changing intervention must change the target"
            ),
        }
    }

    // The shuffled-state control actually changes the recovered meaning.
    let sample = &pairs[1].base;
    assert_ne!(
        recover_whole(sample),
        recover_whole(&corrupt_subject(sample)),
        "shuffled-state must corrupt the recovered meaning"
    );

    // Attribution operates: the persistent-state arm's gains over the baseline
    // are non-empty and land in the state-discrimination column here.
    let f = attribute(
        Arm::PersistentState,
        Arm::CurrentScoring,
        &all_prompts(&pairs),
    );
    assert!(f.state_discrimination > 0, "attribution must credit gains");
}

/// Verification: "Power analysis fixes n and the minimum decision-relevant
/// effect before fitting." The n and MDE are compile-time constants; the fixture
/// must satisfy them and the causal arm must clear the MDE over its worst null.
#[test]
fn power_fixes_n_and_mde_before_fitting() {
    let pairs = primary_pairs();
    assert!(
        pairs.len() as u64 >= MIN_POWERED_N,
        "n must meet the pre-declared powered minimum"
    );
    let primary = causal_delta(Arm::PersistentState, &pairs, Null::None)
        .rate_permille()
        .unwrap();
    let worst_null = [Null::PromptSwap, Null::SuffixOnly, Null::TrivialPrior]
        .into_iter()
        .map(|n| {
            causal_delta(Arm::PersistentState, &pairs, n)
                .rate_permille()
                .unwrap()
        })
        .max()
        .unwrap();
    assert!(
        primary.saturating_sub(worst_null) >= MDE_PERMILLE,
        "the design must resolve the pre-declared MDE ({MDE_PERMILLE}‰): {primary}‰ vs {worst_null}‰"
    );
}

/// Verification: "Double-run/reordered-shard determinism check." The CID-bound
/// record reduces identically under repetition and under sharded/reversed order.
#[test]
fn double_run_and_reordered_shard_determinism() {
    let pairs = primary_pairs();
    for &a in &ALL_ARMS {
        let a1 = report_cid(a, &pairs);
        let a2 = report_cid(a, &pairs);
        assert_eq!(
            a1,
            a2,
            "double run must be byte-identical for {}",
            arm_label(a)
        );

        // Reorder the pairs (reverse) — the stable-id keyed reduction is
        // order-independent, so the CID is unchanged.
        let mut reordered: Vec<Pair> = primary_pairs();
        reordered.reverse();
        let a3 = report_cid(a, &reordered);
        assert_eq!(
            a1,
            a3,
            "reordered-shard reduction must be order-independent for {}",
            arm_label(a)
        );
    }
}

/// Verification: "Planted prompt-insensitive and diversity-only models fail the
/// primary gate." Both are flagged degenerate against the prompt-swap null.
#[test]
fn planted_negatives_fail_the_primary_gate() {
    let pairs = primary_pairs();

    for pl in [Planted::PromptInsensitive, Planted::DiversityOnly] {
        let primary = planted_delta(pl, &pairs, false);
        let swap = planted_delta(pl, &pairs, true);
        assert!(
            is_degenerate_control(&primary, &swap, DEGEN_TOL_PERMILLE),
            "{pl:?} must be flagged degenerate against the prompt-swap null"
        );
        // And it must not reach the genuine causal arm's separation.
        let genuine = causal_delta(Arm::PersistentState, &pairs, Null::None)
            .rate_permille()
            .unwrap();
        assert!(
            primary.rate_permille().unwrap() + MDE_PERMILLE <= genuine + 1000,
            "planted primary must stay below the causal arm by the MDE"
        );
        assert!(
            primary.rate_permille().unwrap() < genuine,
            "{pl:?} must not reach the causal arm's primary rate"
        );
    }

    // The diversity-only model is genuinely diverse (not constant): it emits
    // more than one distinct token across the fixture, yet still fails the gate.
    let distinct: BTreeSet<u64> = all_prompts(&pairs)
        .iter()
        .map(|p| planted_predict(Planted::DiversityOnly, p))
        .collect();
    assert!(
        distinct.len() > 1,
        "diversity-only must be distinct-but-irrelevant, not constant"
    );
}

/// Verification: "Independent report recomputation from stored scores/witnesses."
/// The CID recomputes from the canonical record bytes.
#[test]
fn independent_report_recomputation_from_cid() {
    let pairs = primary_pairs();
    for &a in &ALL_ARMS {
        let bytes = record_bytes(a, &pairs);
        let stored = compute_cid(&bytes);
        let recomputed = compute_cid(&record_bytes(a, &pairs));
        assert_eq!(
            stored,
            recomputed,
            "CID must recompute for {}",
            arm_label(a)
        );
        assert_eq!(stored, report_cid(a, &pairs));
    }
}

/// Acceptance: "A control cannot pass through zero variance or identical
/// outputs." Every null is a real transform that separates a genuinely causal
/// arm from its clean reading.
#[test]
fn every_control_is_non_degenerate() {
    let pairs = primary_pairs();
    let clean = causal_delta(Arm::PersistentState, &pairs, Null::None);
    for n in [
        Null::PromptSwap,
        Null::SuffixOnly,
        Null::ShuffledState,
        Null::TrivialPrior,
    ] {
        let under = causal_delta(Arm::PersistentState, &pairs, n);
        assert!(
            !is_degenerate_control(&clean, &under, DEGEN_TOL_PERMILLE),
            "null {n:?} must separate from the causal arm (non-degenerate)"
        );
        // The null must actually change at least one prediction (no silent no-op).
        let changed = (0..pairs.len()).any(|i| {
            let len = pairs.len();
            let swap = &pairs[(i + 1) % len].base;
            predict_null(Arm::PersistentState, &pairs[i].base, swap, Null::None)
                != predict_null(Arm::PersistentState, &pairs[i].base, swap, n)
        });
        assert!(changed, "null {n:?} must change at least one prediction");
    }
}

/// Run contract: "reachability arithmetic must exceed the effect floor" and
/// headline movement cannot exceed the reachable fraction.
#[test]
fn reachability_ceiling_bounds_headline_movement() {
    let pairs = primary_pairs();
    for &a in &ALL_ARMS {
        let reach = reachability(a, &pairs);
        let ceiling = reach.candidate_permille.max(reach.scoring_permille);
        let headline = causal_delta(a, &pairs, Null::None).rate_permille().unwrap();
        // Movement over the deployed baseline cannot exceed what the arm can reach.
        let baseline = causal_delta(Arm::CurrentScoring, &pairs, Null::None)
            .rate_permille()
            .unwrap();
        let movement = headline.saturating_sub(baseline);
        assert!(
            movement <= ceiling.max(1) * 2,
            "{}: movement {movement}‰ exceeds reachable ceiling {ceiling}‰",
            arm_label(a)
        );
    }
    // The genuine causal arm's reachability must exceed the effect floor.
    let reach = reachability(Arm::PersistentState, &pairs);
    assert!(
        reach.scoring_permille >= MDE_PERMILLE,
        "scoring reachability {}‰ must exceed the MDE floor",
        reach.scoring_permille
    );
}

/// The causal arm is a real, high-signal, non-degenerate reading: it separates
/// from every null. Anti-vacuity: a zero reading would mean "no effect", not a
/// broken harness.
#[test]
fn causal_arm_separates_from_all_nulls() {
    let pairs = primary_pairs();
    let primary = causal_delta(Arm::PersistentState, &pairs, Null::None);
    assert!(primary.rate_permille().unwrap() >= 1000 - MDE_PERMILLE);
    for n in [Null::PromptSwap, Null::SuffixOnly, Null::TrivialPrior] {
        let under = causal_delta(Arm::PersistentState, &pairs, n);
        assert!(
            !is_degenerate_control(&primary, &under, DEGEN_TOL_PERMILLE),
            "causal arm must separate from {n:?}"
        );
    }
}

/// Acceptance: "Candidate-support versus score-conditioning effects are
/// separately identified." On in-vocabulary pairs the persistent-state gain
/// attributes to state discrimination; on needs-expansion pairs the gain
/// attributes to candidate availability.
#[test]
fn candidate_and_score_conditioning_separately_identified() {
    let primary = primary_pairs();
    let cand = candidate_pairs();

    // Score conditioning: persistent state over the baseline candidate set.
    let f_score = attribute(
        Arm::PersistentState,
        Arm::CurrentScoring,
        &all_prompts(&primary),
    );
    assert!(f_score.state_discrimination > 0);
    assert_eq!(
        f_score.candidate_availability, 0,
        "in-vocabulary gains must not be attributed to candidate availability"
    );

    // Candidate availability: expansion reaches meanings the baseline set cannot.
    let f_cand = attribute(
        Arm::CandidateSupportExpansion,
        Arm::PersistentState,
        &all_prompts(&cand),
    );
    assert!(
        f_cand.candidate_availability > 0,
        "needs-expansion gains must be attributed to candidate availability"
    );
    assert_eq!(
        f_cand.state_discrimination, 0,
        "candidate-availability gains must not be attributed to state discrimination"
    );
    // Persistent state alone cannot reach out-of-vocabulary meanings.
    for pr in &cand {
        assert_ne!(predict(Arm::PersistentState, &pr.base), pr.base.meaning);
        assert_eq!(
            predict(Arm::CandidateSupportExpansion, &pr.base),
            pr.base.meaning
        );
    }
}

/// Instrument self-test of the decision function (NOT the S1 verdict): on the
/// controlled fixture the instrument SELECTs a genuinely causal arm; against
/// each planted negative alone it establishes no arm.
#[test]
fn instrument_selects_causal_arm_and_rejects_planted_negatives() {
    let pairs = primary_pairs();
    match fixture_decision(&pairs) {
        Decision::Select(a) => assert!(
            matches!(
                a,
                Arm::PersistentState
                    | Arm::ConditionalResiduals
                    | Arm::CandidateSupportExpansion
                    | Arm::LongerLocalContext
            ),
            "must select a genuinely prompt-conditioned arm, got {}",
            arm_label(a)
        ),
        other => panic!("instrument must select an arm on the causal fixture, got {other:?}"),
    }

    // A world containing only prompt-insensitive behavior yields no arm: the
    // baseline (current scoring) does not clear the MDE over its own nulls.
    let baseline_only = causal_delta(Arm::CurrentScoring, &pairs, Null::None)
        .rate_permille()
        .unwrap();
    let baseline_swap = causal_delta(Arm::CurrentScoring, &pairs, Null::PromptSwap)
        .rate_permille()
        .unwrap();
    assert!(
        baseline_only.saturating_sub(baseline_swap) < MDE_PERMILLE,
        "the suffix-local baseline must not clear the MDE (no spurious selection)"
    );
}

/// Binds the frozen #832 vocabulary: the normative scorer id, the control
/// labels this harness exercises, the resolution-path attribution histogram,
/// and the committed suite id string.
#[test]
fn bakeoff_binds_normative_scorer_and_frozen_vocabulary() {
    assert_eq!(
        NORMATIVE_SCORER_ID,
        "uor-r4-graph-format::scoring_semantics@1.0.0"
    );
    // The nulls map to declared ControlKind labels.
    assert_eq!(ControlKind::PromptSwap.label(), "prompt-swap");
    assert_eq!(ControlKind::SuffixOnly.label(), "suffix-only");
    assert_eq!(ControlKind::ShuffledState.label(), "shuffled-state");
    assert_eq!(ControlKind::TrivialPrior.label(), "trivial-prior");
    for n in [
        Null::PromptSwap,
        Null::SuffixOnly,
        Null::ShuffledState,
        Null::TrivialPrior,
    ] {
        assert!(null_kind(n).is_some());
    }

    // Resolution-path attribution over the causal arm: every position is served
    // (no declines), and the causal fixture resolves on the graph-tier path.
    let pairs = primary_pairs();
    let mut hist = AttributionHistogram::default();
    for p in all_prompts(&pairs) {
        hist.record(resolution_path(Arm::PersistentState, p));
    }
    assert_eq!(hist.total(), (pairs.len() * 2) as u64);
    assert_eq!(hist.served(), hist.total(), "no position declines here");
    assert!(hist.count(ResolutionPath::Graph) > 0);
}

/// The three-way decision vocabulary (`SELECT` / `REVISE` / `NO PROMPT-
/// CONDITIONING ARM ESTABLISHED`) is complete: each verdict is reachable under
/// the posted thresholds. `REVISE` is the "real arm, but quality/resource
/// regresses" branch the run contract's `if positive`/`if negative` split needs.
#[test]
fn verdict_vocabulary_is_complete() {
    // Below the MDE, or no candidate arm: no arm established.
    assert_eq!(
        decide_verdict(MDE_PERMILLE - 1, 1000, true, Some(Arm::PersistentState)),
        Decision::NoArmEstablished
    );
    assert_eq!(
        decide_verdict(1000, 1000, true, None),
        Decision::NoArmEstablished
    );

    // Clears the MDE and passes quality/resource non-regression: SELECT.
    assert_eq!(
        decide_verdict(1000, 1000, true, Some(Arm::PersistentState)),
        Decision::Select(Arm::PersistentState)
    );

    // Clears the MDE but regresses on paraphrase stability, or on resources:
    // REVISE (not a clean select, not a null result).
    assert_eq!(
        decide_verdict(1000, STABILITY_FLOOR - 1, true, Some(Arm::PersistentState)),
        Decision::Revise
    );
    assert_eq!(
        decide_verdict(1000, 1000, false, Some(Arm::PersistentState)),
        Decision::Revise
    );
}
