//! `joint-conditional` D1 reference instrument — S1 representation redesign
//! (#822).
//!
//! Mandated by the S1 REVISE verdict (#822, 2026-08-21) through the approved
//! redesign RFC (`docs/s1_redesign_rfc_822.md` §4-D1, §6 step 2); the run
//! contract was posted to #822 before this run. The #834 five-arm space
//! re-weighted evidence whose table keys were fixed (2-token-suffix tables +
//! unigram content tables): its best content arms measured +17.5‰ (Ψ, #875)
//! and +16.2‰ (conditional residuals against the GLOBAL marginal, #891), CI
//! upper bounds ≤ 19.0‰ < the frozen 20‰ floor. This instrument changes the
//! table key itself: joint conditional tables keyed by
//! `(content token, 2-token suffix)` learn `P(answer | t ∈ window, suffix)`
//! residuals relative to `P(answer | suffix)` — the suffix-conditional
//! baseline the #891 falsifier identified as the missing subtrahend
//! (subtracting the global marginal added nothing: CR-vs-Ψ −1.3‰
//! [−2.4, −0.1]).
//!
//! Per the Q1 answer recorded on #822: the recorded `t_argmax` labels
//! condition on the full article prefix, hard-capped at 128 positions
//! (`--sequence-length 128`), so label-borne conditioning between the 8-token
//! deployed key and 128 tokens exists for joint keys to reach; nothing beyond
//! 128 tokens is promised or reachable by this instrument.
//!
//! ## Arms (pre-registered on #822; λ fixed at 1.0 before evaluation)
//!
//! Reproduction arms on the exact #891 candidate sets (harness gates; their
//! recorded values in `docs/conditional_residuals_834_result.json` must
//! reproduce before the verdict binds):
//!   * `base`  — suffix rate over suffix candidates (the suffix-local floor).
//!   * `psi`   — the #875 segment lane (264.1‰ recorded).
//!   * `cr`    — the #891 conditional-residuals arm (262.8‰ recorded).
//!
//! New arms on the shared widened candidate set (suffix ∪ content-top-32 ∪
//! joint-top-32; identical across the new arms so their contrasts stay pure
//! scoring-rule ablations):
//!   * `joint` — PRIMARY. score(c) = suffix_rate(c) + λ·(mean over the
//!     window's unique tokens t with a SUPPORTED joint key (t, suffix) of
//!     [joint_rate_t(c) − suffix_rate(c)]); a position with no supported
//!     token keeps the suffix score unchanged (base fallback).
//!   * `mix`   — the RFC-named backed-off fallback: the joint residual where
//!     the joint key is supported, the verbatim #875 Ψ-bag contribution where
//!     not, normalized over all unique window tokens.
//!   * `joint-narrow` — joint scoring over the legacy #891 candidate set
//!     (separates score-conditioning from candidate availability).
//!   * `d4pos` / `d4skip` — RFC §4-D4 comparison arms: the same residual
//!     shape over (token, distance-from-end) keys and (token, last-token)
//!     skip-bigram keys.
//!
//! Nulls (planted; the instrument must be able to fail):
//!   * `swap`    — different-story content/joint evidence scored under this
//!     window's suffix (the #891 construction extended to joint tables).
//!   * `keyshuf` — this window's tokens looked up under the swap partner's
//!     suffix key: the conditioning-specificity null unique to D1. If the
//!     joint arm's edge does not depend on the SPECIFIC (token, suffix)
//!     pairing, keyshuf tracks the real arm and the D1 premise is refuted.
//!
//! ## Pre-registered decision rule (posted to #822 before this run)
//!
//! PRIMARY = paired joint-vs-base 95% lower bound over all held-out
//! positions.
//!   * SELECT  iff the lower bound ≥ 25.0‰ — the Q3-adopted lowering-track
//!     OPENING bar (the frozen 20‰ `CAUSAL_FLOOR_PERMILLE` stays the
//!     promotion gate). Consultation order: `joint` first; `mix` consulted
//!     only when `joint` is below the bar.
//!   * NO ARM  iff the joint lower bound ≤ 0 and joint follows 0 minimal
//!     pairs.
//!   * REVISE  otherwise; a lower bound in [20, 25) is recorded explicitly
//!     as floor-clearing-but-below-opening-bar (no lowering track opens).
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test joint_conditional_run_822 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus is read from it).

#![allow(clippy::doc_lazy_continuation)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

const SUFFIX_K: usize = 2;
/// Per-key cap on retained argmax counts (bounded lane tables, #835).
const CAP: usize = 64;
/// Candidate-set widths: suffix baseline, content widening (#891 verbatim),
/// and the joint-evidence widening introduced by this instrument.
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
const CAND_JOINT: usize = 32;
/// Pre-registered residual weight λ = LAMBDA_NUM / LAMBDA_DEN, fixed before
/// evaluation; the λ-sweep is exploratory, NOT the verdict.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// The frozen causal floor (‰) — the promotion gate (#887: the bar stands).
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// The Q3-adopted OPENING bar (‰) a paired lower bound must clear before a
/// lowering track opens (RFC §1; approval comment on #822, 2026-08-21).
const OPENING_BAR_PERMILLE: f64 = 25.0;
/// Positions re-scored by the in-harness double-run determinism check.
const DOUBLE_RUN_N: usize = 2_000;
/// Reproduction gates: recorded values this harness must reproduce
/// (`docs/conditional_residuals_834_result.json`, #891 / #875).
const REPRO_BASE_PERMILLE: f64 = 246.6;
const REPRO_PSI_PERMILLE: f64 = 264.1;
const REPRO_CR_PERMILLE: f64 = 262.8;
const REPRO_MP_TOTAL: u64 = 4_722;
const REPRO_PSI_FOLLOW: u64 = 10;
const REPRO_CR_FOLLOW: u64 = 13;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn bundle_root() -> PathBuf {
    match std::env::var_os("R4_CAUSAL_BUNDLE") {
        Some(v) => PathBuf::from(v),
        None => repo_root()
            .join(".uor-models")
            .join("compiled")
            .join("smollm2-360m-broad-clean"),
    }
}

fn ci95_permille(hits: u64, n: u64) -> (f64, f64, f64) {
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let p = hits as f64 / n as f64;
    let half = 1.96 * (p * (1.0 - p) / n as f64).sqrt();
    (
        p * 1000.0,
        (p - half).max(0.0) * 1000.0,
        (p + half).min(1.0) * 1000.0,
    )
}

fn paired_delta_permille(d: &[i8]) -> (f64, f64, f64) {
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mean = d.iter().map(|&x| x as f64).sum::<f64>() / n as f64;
    let var = d.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / n as f64;
    let half = 1.96 * (var / n as f64).sqrt();
    (
        mean * 1000.0,
        (mean - half) * 1000.0,
        (mean + half) * 1000.0,
    )
}

/// Counts of teacher argmax under one key, kept sparse and capped to the top
/// `CAP` by count (ties broken by the smaller token id) with the total
/// retained.
#[derive(Default, Clone)]
struct Counter {
    map: HashMap<u32, u32>,
}

impl Counter {
    fn bump(&mut self, tok: u32) {
        *self.map.entry(tok).or_insert(0) += 1;
    }
    fn cap_to_top(&mut self, cap: usize) {
        if self.map.len() <= cap {
            return;
        }
        let mut v: Vec<(u32, u32)> = self.map.drain().collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(cap);
        self.map = v.into_iter().collect();
    }
    fn total(&self) -> u64 {
        self.map.values().map(|&c| c as u64).sum()
    }
}

fn suffix_key(window: &[u32]) -> (u32, u32) {
    let n = window.len();
    if n >= 2 {
        (window[n - 2], window[n - 1])
    } else if n == 1 {
        (u32::MAX, window[0])
    } else {
        (u32::MAX, u32::MAX)
    }
}

/// Argmax by score, canonical tie-break (score desc, id asc).
fn argmax(scores: &HashMap<u32, f64>) -> u32 {
    let mut best_id = u32::MAX;
    let mut best = f64::NEG_INFINITY;
    for (&id, &s) in scores {
        if s > best || (s == best && id < best_id) {
            best = s;
            best_id = id;
        }
    }
    best_id
}

fn uniq_tokens(w: &[u32]) -> Vec<u32> {
    let mut uniq: Vec<u32> = w.to_vec();
    uniq.sort_unstable();
    uniq.dedup();
    uniq
}

/// Per-position evidence under one key family: summed supported-key rates per
/// candidate token, the number of supported keys, and the number of keys the
/// window offered (for coverage reporting).
struct Evidence {
    acc: HashMap<u32, f64>,
    supported: usize,
    offered: usize,
}

/// The fitted tables, shared by every arm and control.
struct Tables {
    suffix_next: HashMap<(u32, u32), Counter>,
    content_next: HashMap<u32, Counter>,
    joint_next: HashMap<(u32, (u32, u32)), Counter>,
    d4pos_next: HashMap<(u32, u8), Counter>,
    d4skip_next: HashMap<(u32, u32), Counter>,
    marginal: Counter,
    marginal_total: u64,
    marginal_tok: u32,
}

impl Tables {
    fn marginal_rate(&self, c: u32) -> f64 {
        self.marginal.map.get(&c).copied().unwrap_or(0) as f64 / self.marginal_total.max(1) as f64
    }

    /// Whole-prompt content aggregate of a window — the #875 construction,
    /// verbatim: mean over the window's unique tokens of each token's capped
    /// argmax rate.
    fn content_rates(&self, w: &[u32]) -> HashMap<u32, f64> {
        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let uniq = uniq_tokens(w);
        let ncontent = uniq.len().max(1) as f64;
        for t in &uniq {
            if let Some(cn) = self.content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *content_rate.entry(c).or_insert(0.0) += (cnt as f64 / tot) / ncontent;
                }
            }
        }
        content_rate
    }

    /// Suffix candidate set (top `CAND_SUFFIX` + the marginal token, sorted,
    /// deduped) — a pure function of the suffix key, as in #875/#891.
    fn suffix_cands(&self, w: &[u32]) -> Vec<u32> {
        let mut suffix_cands: Vec<u32> = Vec::new();
        if let Some(c) = self.suffix_next.get(&suffix_key(w)) {
            let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            suffix_cands.extend(v.into_iter().take(CAND_SUFFIX).map(|(k, _)| k));
        }
        suffix_cands.push(self.marginal_tok);
        suffix_cands.sort_unstable();
        suffix_cands.dedup();
        suffix_cands
    }

    /// The #891 widened candidate set: suffix candidates ∪ top `CAND_CONTENT`
    /// content candidates by content rate. Reproduction arms use exactly this.
    fn widened_cands(&self, suffix_cands: &[u32], content_rate: &HashMap<u32, f64>) -> Vec<u32> {
        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.to_vec();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        all_cands
    }

    /// D1 widening: the legacy #891 set ∪ top `CAND_JOINT` candidates by
    /// summed supported joint rates — shared by every new arm.
    fn joint_widened(&self, legacy: &[u32], ev: &Evidence) -> Vec<u32> {
        let mut jr: Vec<(u32, f64)> = ev.acc.iter().map(|(&k, &v)| (k, v)).collect();
        jr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all = legacy.to_vec();
        all.extend(jr.into_iter().take(CAND_JOINT).map(|(k, _)| k));
        all.sort_unstable();
        all.dedup();
        all
    }

    fn suffix_rate(&self, w: &[u32], c: u32) -> f64 {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total
    }

    /// Joint evidence for `tokens` under suffix key `sfx`: per-candidate
    /// summed `joint_rate_t`, over the unique tokens whose (t, sfx) key is
    /// supported in TRAIN.
    fn joint_evidence(&self, tokens: &[u32], sfx: (u32, u32)) -> Evidence {
        let uniq = uniq_tokens(tokens);
        let offered = uniq.len();
        let mut acc: HashMap<u32, f64> = HashMap::new();
        let mut supported = 0usize;
        for t in &uniq {
            if let Some(cn) = self.joint_next.get(&(*t, sfx)) {
                supported += 1;
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *acc.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            }
        }
        Evidence {
            acc,
            supported,
            offered,
        }
    }

    /// D4 positional evidence: unique (token, distance-from-end) pairs.
    fn d4pos_evidence(&self, w: &[u32]) -> Evidence {
        let n = w.len();
        let mut pairs: Vec<(u32, u8)> = w
            .iter()
            .enumerate()
            .map(|(i, &t)| (t, (n - i) as u8))
            .collect();
        pairs.sort_unstable();
        pairs.dedup();
        let offered = pairs.len();
        let mut acc: HashMap<u32, f64> = HashMap::new();
        let mut supported = 0usize;
        for key in &pairs {
            if let Some(cn) = self.d4pos_next.get(key) {
                supported += 1;
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *acc.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            }
        }
        Evidence {
            acc,
            supported,
            offered,
        }
    }

    /// D4 skip-bigram evidence: unique (earlier-token, last-token) pairs.
    fn d4skip_evidence(&self, w: &[u32]) -> Evidence {
        let n = w.len();
        let mut acc: HashMap<u32, f64> = HashMap::new();
        let mut supported = 0usize;
        let mut offered = 0usize;
        if n >= 2 {
            let last = w[n - 1];
            let mut pairs: Vec<(u32, u32)> = w[..n - 1].iter().map(|&t| (t, last)).collect();
            pairs.sort_unstable();
            pairs.dedup();
            offered = pairs.len();
            for key in &pairs {
                if let Some(cn) = self.d4skip_next.get(key) {
                    supported += 1;
                    let tot = cn.total().max(1) as f64;
                    for (&c, &cnt) in &cn.map {
                        *acc.entry(c).or_insert(0.0) += cnt as f64 / tot;
                    }
                }
            }
        }
        Evidence {
            acc,
            supported,
            offered,
        }
    }

    /// Mean-supported-residual scoring shared by `joint`, `d4pos`, `d4skip`,
    /// `swap`, and `keyshuf`: score(c) = base(c) + λ·(mean_t joint_rate_t(c)
    /// − base(c)) over supported keys; base fallback when nothing is
    /// supported. `w` supplies the suffix baseline.
    fn residual_scores(
        &self,
        w: &[u32],
        ev: &Evidence,
        cands: &[u32],
        lam: f64,
    ) -> HashMap<u32, f64> {
        let mut m = HashMap::new();
        for &c in cands {
            let base = self.suffix_rate(w, c);
            let s = if ev.supported > 0 {
                let mean = ev.acc.get(&c).copied().unwrap_or(0.0) / ev.supported as f64;
                base + lam * (mean - base)
            } else {
                base
            };
            m.insert(c, s);
        }
        m
    }

    /// Backed-off mix: joint residual where the (t, sfx) key is supported,
    /// the verbatim #875 Ψ-bag contribution where not, normalized over all
    /// unique window tokens.
    fn mix_scores(&self, w: &[u32], sfx: (u32, u32), cands: &[u32], lam: f64) -> HashMap<u32, f64> {
        let uniq = uniq_tokens(w);
        let ntot = uniq.len().max(1) as f64;
        let mut contrib: HashMap<u32, f64> = HashMap::new();
        let mut sup_ts = 0usize;
        for t in &uniq {
            if let Some(cn) = self.joint_next.get(&(*t, sfx)) {
                sup_ts += 1;
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *contrib.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            } else if let Some(cn) = self.content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *contrib.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            }
        }
        let mut m = HashMap::new();
        for &c in cands {
            let base = self.suffix_rate(w, c);
            let resid = (contrib.get(&c).copied().unwrap_or(0.0) - sup_ts as f64 * base) / ntot;
            m.insert(c, base + lam * resid);
        }
        m
    }
}

/// One held-out position's predictions across every arm and control except
/// the separately-scored swap null.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Preds {
    base: u32,
    psi: u32,
    cr: u32,
    joint: u32,
    mix: u32,
    jnarrow: u32,
    d4pos: u32,
    d4skip: u32,
    keyshuf: u32,
}

/// Score one position's non-swap arms. `partner_sfx` drives the keyshuf null.
fn score_position(t: &Tables, w: &[u32], partner_sfx: (u32, u32), lam: f64) -> Preds {
    let sfx = suffix_key(w);
    let content_rate = t.content_rates(w);
    let suffix_cands = t.suffix_cands(w);
    let legacy_cands = t.widened_cands(&suffix_cands, &content_rate);
    let joint_ev = t.joint_evidence(w, sfx);
    let d1_cands = t.joint_widened(&legacy_cands, &joint_ev);

    let mut base_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &suffix_cands {
        base_scores.insert(c, t.suffix_rate(w, c));
    }

    let mut psi_scores: HashMap<u32, f64> = HashMap::new();
    let mut cr_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &legacy_cands {
        let base = t.suffix_rate(w, c);
        let cont = content_rate.get(&c).copied().unwrap_or(0.0);
        psi_scores.insert(c, base + lam * cont);
        cr_scores.insert(c, base + lam * (cont - t.marginal_rate(c)));
    }

    let joint_scores = t.residual_scores(w, &joint_ev, &d1_cands, lam);
    let jnarrow_scores = t.residual_scores(w, &joint_ev, &legacy_cands, lam);
    let mix_scores = t.mix_scores(w, sfx, &d1_cands, lam);

    let d4pos_ev = t.d4pos_evidence(w);
    let d4skip_ev = t.d4skip_evidence(w);
    let d4pos_scores = t.residual_scores(w, &d4pos_ev, &d1_cands, lam);
    let d4skip_scores = t.residual_scores(w, &d4skip_ev, &d1_cands, lam);

    // Conditioning-specificity null: this window's tokens under the swap
    // partner's suffix key (baseline rates stay this window's own).
    let keyshuf_ev = t.joint_evidence(w, partner_sfx);
    let keyshuf_cands = t.joint_widened(&legacy_cands, &keyshuf_ev);
    let keyshuf_scores = t.residual_scores(w, &keyshuf_ev, &keyshuf_cands, lam);

    Preds {
        base: argmax(&base_scores),
        psi: argmax(&psi_scores),
        cr: argmax(&cr_scores),
        joint: argmax(&joint_scores),
        mix: argmax(&mix_scores),
        jnarrow: argmax(&jnarrow_scores),
        d4pos: argmax(&d4pos_scores),
        d4skip: argmax(&d4skip_scores),
        keyshuf: argmax(&keyshuf_scores),
    }
}

/// Prompt-swap null: the joint arm scored with this window's suffix baseline
/// but the content/joint evidence of the different-story `w_content`.
fn score_swap(t: &Tables, w_suffix: &[u32], w_content: &[u32], lam: f64) -> u32 {
    let sfx = suffix_key(w_suffix);
    let content_rate = t.content_rates(w_content);
    let suffix_cands = t.suffix_cands(w_suffix);
    let legacy_cands = t.widened_cands(&suffix_cands, &content_rate);
    let ev = t.joint_evidence(w_content, sfx);
    let cands = t.joint_widened(&legacy_cands, &ev);
    let scores = t.residual_scores(w_suffix, &ev, &cands, lam);
    argmax(&scores)
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn joint_conditional_run_822() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP joint_conditional_run_822: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (train, held_out) = induction::split_positions(&corpus);
    assert!(!train.is_empty() && !held_out.is_empty(), "non-empty split");

    let started = Instant::now();

    // --- build tables from TRAIN (document-disjoint by story) --------------
    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut content_next: HashMap<u32, Counter> = HashMap::new();
    let mut joint_next: HashMap<(u32, (u32, u32)), Counter> = HashMap::new();
    let mut d4pos_next: HashMap<(u32, u8), Counter> = HashMap::new();
    let mut d4skip_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut marginal = Counter::default();
    for &i in &train {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        let sfx = suffix_key(&w);
        marginal.bump(target);
        suffix_next.entry(sfx).or_default().bump(target);
        for t in uniq_tokens(&w) {
            content_next.entry(t).or_default().bump(target);
            joint_next.entry((t, sfx)).or_default().bump(target);
        }
        let n = w.len();
        let mut pos_pairs: Vec<(u32, u8)> = w
            .iter()
            .enumerate()
            .map(|(idx, &t)| (t, (n - idx) as u8))
            .collect();
        pos_pairs.sort_unstable();
        pos_pairs.dedup();
        for key in pos_pairs {
            d4pos_next.entry(key).or_default().bump(target);
        }
        if n >= 2 {
            let last = w[n - 1];
            let mut skip_pairs: Vec<(u32, u32)> = w[..n - 1].iter().map(|&t| (t, last)).collect();
            skip_pairs.sort_unstable();
            skip_pairs.dedup();
            for key in skip_pairs {
                d4skip_next.entry(key).or_default().bump(target);
            }
        }
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in content_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in joint_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in d4pos_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in d4skip_next.values_mut() {
        c.cap_to_top(CAP);
    }
    // The corpus marginal stays UNCAPPED: it is the prior the reproduced CR
    // arm conditions against, not a bounded lane table.
    let marginal_total = marginal.total();
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);
    let joint_entries = joint_next.len() as u64;
    let joint_mass: u64 = joint_next.values().map(Counter::total).sum();
    let d4pos_entries = d4pos_next.len() as u64;
    let d4skip_entries = d4skip_next.len() as u64;

    let tables = Tables {
        suffix_next,
        content_next,
        joint_next,
        d4pos_next,
        d4skip_next,
        marginal,
        marginal_total,
        marginal_tok,
    };
    assert!(joint_entries > 0, "joint tables must be non-empty");

    let lambda = LAMBDA_NUM / LAMBDA_DEN;
    let n = held_out.len() as u64;

    // --- precompute windows, then swap partners (different story) ----------
    let windows: Vec<Vec<u32>> = held_out
        .iter()
        .map(|&i| induction::context_window(&corpus, i))
        .collect();
    let half = held_out.len() / 2;
    let swap_partner: Vec<usize> = (0..held_out.len())
        .map(|idx| {
            let mut p = (idx + half) % held_out.len();
            while corpus.story[held_out[p]] == corpus.story[held_out[idx]] {
                p = (p + 1) % held_out.len();
            }
            p
        })
        .collect();

    // --- evaluate on HELD-OUT ----------------------------------------------
    let mut preds: Vec<Preds> = Vec::with_capacity(held_out.len());
    let mut swap_preds: Vec<u32> = Vec::with_capacity(held_out.len());
    let mut covered_positions = 0u64;
    let mut supported_sum = 0u64;
    let mut offered_sum = 0u64;
    let mut known_suffix_positions = 0u64;
    for (idx, w) in windows.iter().enumerate() {
        let partner_sfx = suffix_key(&windows[swap_partner[idx]]);
        preds.push(score_position(&tables, w, partner_sfx, lambda));
        swap_preds.push(score_swap(&tables, w, &windows[swap_partner[idx]], lambda));
        let ev = tables.joint_evidence(w, suffix_key(w));
        if ev.supported > 0 {
            covered_positions += 1;
        }
        supported_sum += ev.supported as u64;
        offered_sum += ev.offered as u64;
        if tables.suffix_next.contains_key(&suffix_key(w)) {
            known_suffix_positions += 1;
        }
    }

    let mut base_hits = 0u64;
    let mut psi_hits = 0u64;
    let mut cr_hits = 0u64;
    let mut joint_hits = 0u64;
    let mut mix_hits = 0u64;
    let mut jnarrow_hits = 0u64;
    let mut d4pos_hits = 0u64;
    let mut d4skip_hits = 0u64;
    let mut swap_hits = 0u64;
    let mut keyshuf_hits = 0u64;
    let mut trivial_hits = 0u64;
    let mut joint_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut mix_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut jnarrow_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut joint_vs_psi: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut d4pos_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut d4skip_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut swap_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut keyshuf_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut psi_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut cr_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut swap_changed = 0u64;
    let mut keyshuf_changed = 0u64;
    for (idx, &i) in held_out.iter().enumerate() {
        let target = corpus.t_argmax[i];
        let p = preds[idx];
        let bh = p.base == target;
        let ph = p.psi == target;
        let ch = p.cr == target;
        let jh = p.joint == target;
        let mh = p.mix == target;
        let nh = p.jnarrow == target;
        let dph = p.d4pos == target;
        let dsh = p.d4skip == target;
        let sh = swap_preds[idx] == target;
        let kh = p.keyshuf == target;
        base_hits += u64::from(bh);
        psi_hits += u64::from(ph);
        cr_hits += u64::from(ch);
        joint_hits += u64::from(jh);
        mix_hits += u64::from(mh);
        jnarrow_hits += u64::from(nh);
        d4pos_hits += u64::from(dph);
        d4skip_hits += u64::from(dsh);
        swap_hits += u64::from(sh);
        keyshuf_hits += u64::from(kh);
        trivial_hits += u64::from(tables.marginal_tok == target);
        joint_vs_base.push(i8::from(jh) - i8::from(bh));
        mix_vs_base.push(i8::from(mh) - i8::from(bh));
        jnarrow_vs_base.push(i8::from(nh) - i8::from(bh));
        joint_vs_psi.push(i8::from(jh) - i8::from(ph));
        d4pos_vs_base.push(i8::from(dph) - i8::from(bh));
        d4skip_vs_base.push(i8::from(dsh) - i8::from(bh));
        swap_vs_base.push(i8::from(sh) - i8::from(bh));
        keyshuf_vs_base.push(i8::from(kh) - i8::from(bh));
        psi_vs_base.push(i8::from(ph) - i8::from(bh));
        cr_vs_base.push(i8::from(ch) - i8::from(bh));
        swap_changed += u64::from(swap_preds[idx] != p.joint);
        keyshuf_changed += u64::from(p.keyshuf != p.joint);
    }

    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let (psi_r, _, _) = ci95_permille(psi_hits, n);
    let (cr_r, _, _) = ci95_permille(cr_hits, n);
    let (joint_r, joint_ci_lo, joint_ci_hi) = ci95_permille(joint_hits, n);
    let (mix_r, _, _) = ci95_permille(mix_hits, n);
    let (jnarrow_r, _, _) = ci95_permille(jnarrow_hits, n);
    let (d4pos_r, _, _) = ci95_permille(d4pos_hits, n);
    let (d4skip_r, _, _) = ci95_permille(d4skip_hits, n);
    let (swap_r, _, _) = ci95_permille(swap_hits, n);
    let (keyshuf_r, _, _) = ci95_permille(keyshuf_hits, n);
    let (trivial_r, _, _) = ci95_permille(trivial_hits, n);
    let (joint_delta, joint_delta_lo, joint_delta_hi) = paired_delta_permille(&joint_vs_base);
    let (mix_delta, mix_delta_lo, mix_delta_hi) = paired_delta_permille(&mix_vs_base);
    let (jnarrow_delta, jnarrow_delta_lo, jnarrow_delta_hi) =
        paired_delta_permille(&jnarrow_vs_base);
    let (jpsi_delta, jpsi_lo, jpsi_hi) = paired_delta_permille(&joint_vs_psi);
    let (d4pos_delta, d4pos_delta_lo, d4pos_delta_hi) = paired_delta_permille(&d4pos_vs_base);
    let (d4skip_delta, d4skip_delta_lo, d4skip_delta_hi) = paired_delta_permille(&d4skip_vs_base);
    let (swap_delta, swap_delta_lo, swap_delta_hi) = paired_delta_permille(&swap_vs_base);
    let (keyshuf_delta, keyshuf_delta_lo, keyshuf_delta_hi) =
        paired_delta_permille(&keyshuf_vs_base);
    let (psi_delta, psi_delta_lo, psi_delta_hi) = paired_delta_permille(&psi_vs_base);
    let (cr_delta, cr_delta_lo, cr_delta_hi) = paired_delta_permille(&cr_vs_base);

    // --- minimal pairs: same suffix, different story, different teacher -----
    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, w) in windows.iter().enumerate() {
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
    }
    let mut mp_total = 0u64;
    let mut base_follow = 0u64;
    let mut psi_follow = 0u64;
    let mut cr_follow = 0u64;
    let mut joint_follow = 0u64;
    let mut mix_follow = 0u64;
    for group in by_suffix.values() {
        let mut made = 0usize;
        'outer: for a in 0..group.len() {
            for b in (a + 1)..group.len() {
                let (xa, xb) = (group[a], group[b]);
                let (ia, ib) = (held_out[xa], held_out[xb]);
                if corpus.story[ia] == corpus.story[ib] {
                    continue;
                }
                let (ta, tb) = (corpus.t_argmax[ia], corpus.t_argmax[ib]);
                if ta == tb {
                    continue;
                }
                mp_total += 1;
                let (pa, pb) = (preds[xa], preds[xb]);
                if pa.base == ta && pb.base == tb && pa.base != pb.base {
                    base_follow += 1;
                }
                if pa.psi == ta && pb.psi == tb && pa.psi != pb.psi {
                    psi_follow += 1;
                }
                if pa.cr == ta && pb.cr == tb && pa.cr != pb.cr {
                    cr_follow += 1;
                }
                if pa.joint == ta && pb.joint == tb && pa.joint != pb.joint {
                    joint_follow += 1;
                }
                if pa.mix == ta && pb.mix == tb && pa.mix != pb.mix {
                    mix_follow += 1;
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let (joint_mp_rate, joint_mp_lo, _) = ci95_permille(joint_follow, mp_total.max(1));

    // --- exploratory λ-sweep for the joint arm (NOT the verdict) -----------
    let sweep_lams = [0.5f64, 1.0, 2.0, 4.0, 8.0];
    let mut sweep: Vec<(f64, u64)> = Vec::new();
    for &lam in &sweep_lams {
        let mut h = 0u64;
        for (idx, &i) in held_out.iter().enumerate() {
            let partner_sfx = suffix_key(&windows[swap_partner[idx]]);
            let p = score_position(&tables, &windows[idx], partner_sfx, lam);
            if p.joint == corpus.t_argmax[i] {
                h += 1;
            }
        }
        sweep.push((lam, h));
    }

    // --- double-run determinism check --------------------------------------
    let check_n = DOUBLE_RUN_N.min(held_out.len());
    for idx in 0..check_n {
        let partner_sfx = suffix_key(&windows[swap_partner[idx]]);
        let again = score_position(&tables, &windows[idx], partner_sfx, lambda);
        assert_eq!(again, preds[idx], "double-run drift at held-out idx {idx}");
        let swap_again = score_swap(&tables, &windows[idx], &windows[swap_partner[idx]], lambda);
        assert_eq!(
            swap_again, swap_preds[idx],
            "double-run swap drift at held-out idx {idx}"
        );
    }

    // --- reproduction gates: #891/#875 recorded values must reproduce ------
    assert!(
        (base_r - REPRO_BASE_PERMILLE).abs() < 0.05,
        "suffix baseline {base_r:.1}permille does not reproduce the recorded {REPRO_BASE_PERMILLE}permille"
    );
    assert!(
        (psi_r - REPRO_PSI_PERMILLE).abs() < 0.05,
        "psi arm {psi_r:.1}permille does not reproduce the recorded {REPRO_PSI_PERMILLE}permille"
    );
    assert!(
        (cr_r - REPRO_CR_PERMILLE).abs() < 0.05,
        "cr arm {cr_r:.1}permille does not reproduce the recorded {REPRO_CR_PERMILLE}permille"
    );
    assert_eq!(
        mp_total, REPRO_MP_TOTAL,
        "minimal-pair mining does not reproduce the recorded pair count"
    );
    assert_eq!(
        psi_follow, REPRO_PSI_FOLLOW,
        "psi minimal-pair follow does not reproduce the recorded value"
    );
    assert_eq!(
        cr_follow, REPRO_CR_FOLLOW,
        "cr minimal-pair follow does not reproduce the recorded value"
    );
    assert_eq!(
        base_follow, 0,
        "suffix baseline cannot follow identical-suffix pairs"
    );

    // --- control teeth ------------------------------------------------------
    assert!(mp_total > 0, "expected minimal pairs to evaluate");
    assert!(
        covered_positions > 0,
        "joint evidence must cover at least one held-out position"
    );
    assert!(
        swap_changed > 0,
        "prompt-swap null must change at least one prediction"
    );
    assert!(
        keyshuf_changed > 0,
        "key-shuffle null must change at least one prediction"
    );
    // Null separations bind only when the arm reads positive: with a
    // null-effect arm the comparison is noise and must not mask a legitimate
    // NO-ARM verdict.
    if joint_delta_lo > 0.0 {
        assert!(
            swap_delta < joint_delta,
            "prompt-swap null ({swap_delta:.1}permille) must sit below a positive arm ({joint_delta:.1}permille)"
        );
        assert!(
            keyshuf_delta < joint_delta,
            "key-shuffle null ({keyshuf_delta:.1}permille) must sit below a positive arm ({joint_delta:.1}permille)"
        );
    }
    assert!(
        joint_hits > trivial_hits,
        "the arm must beat the trivial no-context floor"
    );

    // --- verdict (pre-registered rule; see module docs) ---------------------
    let verdict = if joint_delta_lo >= OPENING_BAR_PERMILLE {
        "SELECT (joint) — the paired lower bound clears the 25permille opening bar; an #836-shaped lowering track may open (the 20permille floor stays the promotion gate)"
    } else if mix_delta_lo >= OPENING_BAR_PERMILLE {
        "SELECT (backed-off mix) — the pre-declared fallback arm clears the 25permille opening bar; an #836-shaped lowering track may open (the 20permille floor stays the promotion gate)"
    } else if joint_delta_lo <= 0.0 && joint_follow == 0 {
        "NO ARM — joint conditional evidence adds no teacher-predictive signal over the suffix floor"
    } else if joint_delta_lo >= CAUSAL_FLOOR_PERMILLE {
        "REVISE (floor-clearing, below opening bar) — the paired lower bound clears the frozen 20permille floor but not the Q3 25permille opening bar; no lowering track opens"
    } else {
        "REVISE — real but sub-floor joint-conditional signal; below the frozen 20permille floor, so no lowering track opens"
    };

    let elapsed = started.elapsed();
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #822 D1 joint-conditional reference instrument ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("train / held_out : {} / {}", train.len(), held_out.len());
    println!("lambda (fixed)   : {lambda}");
    println!(
        "joint tables     : {joint_entries} keys, mass {joint_mass}; d4pos {d4pos_entries} keys, d4skip {d4skip_entries} keys"
    );
    println!(
        "joint coverage   : {covered_positions}/{n} positions with support; mean supported tokens {:.2}/{:.2}; known-suffix positions {known_suffix_positions}",
        supported_sum as f64 / n as f64,
        offered_sum as f64 / n as f64
    );
    println!("suffix baseline  : {base_r:.1}permille  (95% CI [{base_lo:.1}, {base_hi:.1}])");
    println!(
        "psi (reproduced) : {psi_r:.1}permille  (delta {psi_delta:.1} [{psi_delta_lo:.1}, {psi_delta_hi:.1}])"
    );
    println!(
        "cr (reproduced)  : {cr_r:.1}permille  (delta {cr_delta:.1} [{cr_delta_lo:.1}, {cr_delta_hi:.1}])"
    );
    println!(
        "JOINT (PRIMARY)  : {joint_r:.1}permille  (95% CI [{joint_ci_lo:.1}, {joint_ci_hi:.1}])"
    );
    println!(
        "JOINT-DELTA      : {joint_delta:.1}permille  (paired 95% CI [{joint_delta_lo:.1}, {joint_delta_hi:.1}])"
    );
    println!(
        "mix (fallback)   : {mix_r:.1}permille  (delta {mix_delta:.1} [{mix_delta_lo:.1}, {mix_delta_hi:.1}])"
    );
    println!(
        "joint-narrow     : {jnarrow_r:.1}permille  (delta {jnarrow_delta:.1} [{jnarrow_delta_lo:.1}, {jnarrow_delta_hi:.1}])"
    );
    println!(
        "joint vs psi     : {jpsi_delta:.1}permille  (paired 95% CI [{jpsi_lo:.1}, {jpsi_hi:.1}]; cross-candidate-set, exploratory)"
    );
    println!(
        "d4pos            : {d4pos_r:.1}permille  (delta {d4pos_delta:.1} [{d4pos_delta_lo:.1}, {d4pos_delta_hi:.1}])"
    );
    println!(
        "d4skip           : {d4skip_r:.1}permille  (delta {d4skip_delta:.1} [{d4skip_delta_lo:.1}, {d4skip_delta_hi:.1}])"
    );
    println!(
        "swap null        : {swap_r:.1}permille  (delta {swap_delta:.1} [{swap_delta_lo:.1}, {swap_delta_hi:.1}]; changed {swap_changed})"
    );
    println!(
        "keyshuf null     : {keyshuf_r:.1}permille  (delta {keyshuf_delta:.1} [{keyshuf_delta_lo:.1}, {keyshuf_delta_hi:.1}]; changed {keyshuf_changed})"
    );
    println!("trivial prior    : {trivial_r:.1}permille");
    let sweep_str: String = sweep
        .iter()
        .map(|(l, h)| format!("λ{}={:.0} ", l, *h as f64 / n as f64 * 1000.0))
        .collect();
    println!("λ-sweep (explor.): {sweep_str}(baseline={base_r:.0})permille");
    println!(
        "minimal-pairs    : {mp_total} pairs; joint-follow {joint_follow} ({joint_mp_rate:.1}permille, 95% lo {joint_mp_lo:.1}); mix-follow {mix_follow}; psi-follow {psi_follow}; cr-follow {cr_follow}; baseline-follow {base_follow}"
    );
    println!("double-run       : {check_n} positions identical");
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    let mut rec = Vec::new();
    for v in [
        n,
        base_hits,
        psi_hits,
        cr_hits,
        joint_hits,
        mix_hits,
        jnarrow_hits,
        d4pos_hits,
        d4skip_hits,
        swap_hits,
        keyshuf_hits,
        trivial_hits,
        mp_total,
        joint_follow,
        mix_follow,
        psi_follow,
        cr_follow,
        base_follow,
        swap_changed,
        keyshuf_changed,
        joint_entries,
        joint_mass,
        d4pos_entries,
        d4skip_entries,
        covered_positions,
        supported_sum,
        offered_sum,
        known_suffix_positions,
        marginal_total,
        train.len() as u64,
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    for (_, h) in &sweep {
        rec.extend_from_slice(&h.to_le_bytes());
    }
    rec.extend_from_slice(corpus_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid       : {result_cid}");

    let sweep_json: String = {
        let mut s = String::from("{");
        for (idx, (l, h)) in sweep.iter().enumerate() {
            if idx > 0 {
                s.push(',');
            }
            s.push_str(&format!("\"{}\":{:.1}", l, *h as f64 / n as f64 * 1000.0));
        }
        s.push('}');
        s
    };
    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 822,\n",
            "  \"arm\": \"joint-conditional-d1-reference\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"lambda\": {},\n",
            "  \"joint_tables\": {{\"entries\": {}, \"mass\": {}, \"d4pos_entries\": {}, \"d4skip_entries\": {}}},\n",
            "  \"joint_coverage\": {{\"covered_positions\": {}, \"mean_supported\": {:.2}, \"mean_offered\": {:.2}, \"known_suffix_positions\": {}}},\n",
            "  \"suffix_baseline_permille\": {:.1},\n",
            "  \"psi_reproduction\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"mp_total\": {}, \"psi_follow\": {}}},\n",
            "  \"cr_reproduction\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"cr_follow\": {}}},\n",
            "  \"joint_permille\": {:.1},\n",
            "  \"joint_permille_ci\": [{:.1}, {:.1}],\n",
            "  \"joint_delta_permille\": {:.1},\n",
            "  \"joint_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"mix_permille\": {:.1},\n",
            "  \"mix_delta_permille\": {:.1},\n",
            "  \"mix_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"joint_narrow_permille\": {:.1},\n",
            "  \"joint_narrow_delta_permille\": {:.1},\n",
            "  \"joint_narrow_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"joint_vs_psi_delta_permille\": {:.1},\n",
            "  \"joint_vs_psi_ci\": [{:.1}, {:.1}],\n",
            "  \"d4pos_permille\": {:.1},\n",
            "  \"d4pos_delta_permille\": {:.1},\n",
            "  \"d4pos_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"d4skip_permille\": {:.1},\n",
            "  \"d4skip_delta_permille\": {:.1},\n",
            "  \"d4skip_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"swap_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"keyshuf_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"trivial_prior_permille\": {:.1},\n",
            "  \"lambda_sweep_permille\": {},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"joint_follow\": {}, \"joint_follow_permille\": {:.1}, \"joint_follow_ci_lo\": {:.1}, \"mix_follow\": {}, \"psi_follow\": {}, \"cr_follow\": {}, \"baseline_follow\": {}}},\n",
            "  \"double_run\": {{\"checked\": {}, \"identical\": true}},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        train.len(),
        held_out.len(),
        lambda,
        joint_entries,
        joint_mass,
        d4pos_entries,
        d4skip_entries,
        covered_positions,
        supported_sum as f64 / n as f64,
        offered_sum as f64 / n as f64,
        known_suffix_positions,
        base_r,
        psi_r,
        psi_delta,
        psi_delta_lo,
        psi_delta_hi,
        mp_total,
        psi_follow,
        cr_r,
        cr_delta,
        cr_delta_lo,
        cr_delta_hi,
        cr_follow,
        joint_r,
        joint_ci_lo,
        joint_ci_hi,
        joint_delta,
        joint_delta_lo,
        joint_delta_hi,
        mix_r,
        mix_delta,
        mix_delta_lo,
        mix_delta_hi,
        jnarrow_r,
        jnarrow_delta,
        jnarrow_delta_lo,
        jnarrow_delta_hi,
        jpsi_delta,
        jpsi_lo,
        jpsi_hi,
        d4pos_r,
        d4pos_delta,
        d4pos_delta_lo,
        d4pos_delta_hi,
        d4skip_r,
        d4skip_delta,
        d4skip_delta_lo,
        d4skip_delta_hi,
        swap_r,
        swap_delta,
        swap_delta_lo,
        swap_delta_hi,
        swap_changed,
        keyshuf_r,
        keyshuf_delta,
        keyshuf_delta_lo,
        keyshuf_delta_hi,
        keyshuf_changed,
        trivial_r,
        sweep_json,
        mp_total,
        joint_follow,
        joint_mp_rate,
        joint_mp_lo,
        mix_follow,
        psi_follow,
        cr_follow,
        base_follow,
        check_n,
        result_cid,
        verdict,
    );
    let out = repo_root()
        .join("docs")
        .join("joint_conditional_822_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
