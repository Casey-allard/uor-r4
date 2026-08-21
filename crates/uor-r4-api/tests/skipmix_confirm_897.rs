//! `skipmix` phase-0 key-family confirmation — S1 redesign lowering (#897).
//!
//! Pre-registered on #897 (ACTIVATED comment, 2026-08-21) as phase 0 of the
//! D1-triggered `#836`-shaped lowering: a single cheap confirmation run, in the
//! D1 harness mold (`joint_conditional_run_822.rs`), that decides the lowered
//! KEY FAMILY before any lowering engineering. It changes nothing about
//! deployed behaviour; it selects which backed-off mix the lowering will lower.
//!
//! The maintainer §6-4 decision on #822 (2026-08-21) recorded the D1
//! comparison arm `d4skip` (1-token conditioning, denser support) at
//! +49.6permille [46.8, 52.4] — stronger than the D1-SELECTED 2-token `mix`
//! (+30.6permille [28.6, 32.5]) — but `d4skip` sat OUTSIDE D1's pre-registered
//! consultation order, so a fresh pre-registered run with the 1-token mix as
//! PRIMARY keeps the selection disciplined.
//!
//! ## Arms (pre-registered on #897; lambda fixed at 1.0 before evaluation)
//!
//! Reproduction arms recomputed on their ORIGINAL D1 constructions
//! (`docs/joint_conditional_822_result.json`) as harness-correctness gates:
//!   * `base` — suffix rate over suffix candidates (the suffix-local floor).
//!     Recorded 246.6permille.
//!   * `mix` — the D1-SELECTED backed-off mix: joint residual keyed by
//!     (content token, 2-token suffix) where supported, verbatim Psi-bag
//!     fallback where not, normalized over all unique window tokens. Recorded
//!     277.2permille.
//!   * `d4skip` — the D1 pure skip residual keyed by (earlier token, last
//!     token), base fallback. Recorded 296.2permille.
//!
//! New PRIMARY arm on the shared D1 widened candidate set (identical to `mix`
//! and `d4skip`, so the contrast is a pure scoring-rule / key-family ablation):
//!   * `skipmix` — the backed-off mix with 1-token conditioning: joint tables
//!     keyed by (content token, LAST window token) where supported, verbatim
//!     Psi-bag fallback where not, normalized over all unique window tokens;
//!     residual against the full 2-token `suffix_rate`; cap 64 per key. This is
//!     `mix` with the join key's suffix component narrowed from the 2-token
//!     suffix to the single last token — nothing else differs.
//!
//! Null (planted; the instrument must be able to fail):
//!   * `skipmix_null` — this window's tokens looked up under the SWAP PARTNER's
//!     last-token key ((t, partner_last)), baseline rates unchanged. The
//!     conditioning-specificity null for the 1-token key: if `skipmix`'s edge
//!     does not depend on the SPECIFIC last token, this null tracks it and the
//!     confirmation is refuted.
//!
//! ## Pre-registered decision rule (posted to #897 before this run)
//!
//! SELECT-1-token iff the `skipmix` paired-vs-`base` 95% lower bound >= 25.0permille
//! (the Q3 lowering-track opening bar) AND the paired `skipmix`-vs-`mix` 95%
//! lower bound > 0. Otherwise the lowering proceeds with the D1-SELECTED
//! 2-token `mix` as measured. The branches differ ONLY in the lowered key
//! family; the lowering itself proceeds either way. The frozen 20permille
//! end-to-end floor remains the promotion gate (unchanged here).
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test skipmix_confirm_897 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus is read from it). Defaults to the
//! attested #833 bundle `.uor-models/compiled/smollm2-360m-broad-clean`.

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
/// and the joint-evidence widening (#822 D1) — identical construction so the
/// new arm sees exactly the D1 candidate set.
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
const CAND_JOINT: usize = 32;
/// Pre-registered residual weight lambda = 1.0, fixed before evaluation.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// The Q3-adopted OPENING bar (permille): the `skipmix` paired lower bound must
/// clear this to select the 1-token key family (RFC §1; #822 approval).
const OPENING_BAR_PERMILLE: f64 = 25.0;
/// The frozen causal floor (permille) — the END-TO-END promotion gate, recorded
/// here for context; phase 0 does not evaluate against it (#887: the bar
/// stands).
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// Positions re-scored by the in-harness double-run determinism check.
const DOUBLE_RUN_N: usize = 2_000;
/// Reproduction gates: recorded D1 values this harness must reproduce
/// (`docs/joint_conditional_822_result.json`).
const REPRO_BASE_PERMILLE: f64 = 246.6;
const REPRO_MIX_PERMILLE: f64 = 277.2;
const REPRO_D4SKIP_PERMILLE: f64 = 296.2;
const REPRO_MP_TOTAL: u64 = 4_722;
/// The attested #833 bundle's corpus.meta CID prefix — the pinned identity this
/// confirmation is only valid on (guards against a wrong bundle at
/// R4_CAUSAL_BUNDLE; a missing bundle SKIPs, a wrong bundle FAILS loudly).
const ATTESTED_CORPUS_CID_PREFIX: &str = "blake3:aa9d1767";

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
/// `CAP` by count (ties broken by the smaller token id).
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

/// Argmax by score, canonical tie-break (score desc, id asc) — order
/// independent, so the run is deterministic despite HashMap iteration.
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

/// Per-position joint evidence used only to build the D1 candidate set: summed
/// supported-key rates per candidate, and the number of supported keys.
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
    d4skip_next: HashMap<(u32, u32), Counter>,
    marginal_tok: u32,
}

impl Tables {
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
    /// content candidates by content rate.
    fn widened_cands(&self, suffix_cands: &[u32], content_rate: &HashMap<u32, f64>) -> Vec<u32> {
        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.to_vec();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        all_cands
    }

    /// D1 widening: the legacy #891 set ∪ top `CAND_JOINT` candidates by summed
    /// supported joint rates — the shared candidate set every new arm sees.
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

    /// Joint evidence for `tokens` under suffix key `sfx`: per-candidate summed
    /// `joint_rate_t` over the unique tokens whose (t, sfx) key is supported.
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

    /// D4 skip-bigram evidence: unique (earlier-token, last-token) pairs — the
    /// pure D1 `d4skip` residual arm (base fallback).
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

    /// Mean-supported-residual scoring for the `d4skip` reproduction arm:
    /// score(c) = base(c) + λ·(mean_t skip_rate_t(c) − base(c)) over supported
    /// keys; base fallback when nothing is supported.
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

    /// The D1-SELECTED backed-off mix (2-token join key): joint residual where
    /// the (t, sfx) key is supported, verbatim #875 Ψ-bag contribution where
    /// not, normalized over all unique window tokens.
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

    /// The PRIMARY `skipmix` arm — `mix_scores` with the join key's suffix
    /// component narrowed from the 2-token suffix to the single `cond_last`
    /// token: joint tables keyed by (t, cond_last) where supported, verbatim
    /// Ψ-bag fallback where not, normalized over all unique window tokens,
    /// residual against the full 2-token `suffix_rate`. `cond_last` is this
    /// window's own last token for the real arm; the swap partner's last token
    /// for the conditioning-specificity null.
    fn skipmix_scores(
        &self,
        w: &[u32],
        cond_last: u32,
        cands: &[u32],
        lam: f64,
    ) -> HashMap<u32, f64> {
        let uniq = uniq_tokens(w);
        let ntot = uniq.len().max(1) as f64;
        let mut contrib: HashMap<u32, f64> = HashMap::new();
        let mut sup_ts = 0usize;
        for t in &uniq {
            if let Some(cn) = self.d4skip_next.get(&(*t, cond_last)) {
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

/// One held-out position's predictions across every arm and control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Preds {
    base: u32,
    mix: u32,
    d4skip: u32,
    skipmix: u32,
    skipmix_null: u32,
}

/// Score one position's arms. `partner_last` drives the skipmix conditioning-
/// specificity null.
fn score_position(t: &Tables, w: &[u32], partner_last: u32, lam: f64) -> Preds {
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

    let mix_scores = t.mix_scores(w, sfx, &d1_cands, lam);

    let d4skip_ev = t.d4skip_evidence(w);
    let d4skip_scores = t.residual_scores(w, &d4skip_ev, &d1_cands, lam);

    let own_last = w.last().copied().unwrap_or(u32::MAX);
    let skipmix_scores = t.skipmix_scores(w, own_last, &d1_cands, lam);
    let skipmix_null_scores = t.skipmix_scores(w, partner_last, &d1_cands, lam);

    Preds {
        base: argmax(&base_scores),
        mix: argmax(&mix_scores),
        d4skip: argmax(&d4skip_scores),
        skipmix: argmax(&skipmix_scores),
        skipmix_null: argmax(&skipmix_null_scores),
    }
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn skipmix_confirm_897() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP skipmix_confirm_897: no serving bundle at {}",
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

    let corpus_cid = compute_cid(&meta_bytes);
    assert!(
        corpus_cid.starts_with(ATTESTED_CORPUS_CID_PREFIX),
        "phase-0 confirmation is only valid on the attested #833 bundle \
         (corpus.meta CID prefix {ATTESTED_CORPUS_CID_PREFIX}); got {corpus_cid}"
    );

    let started = Instant::now();

    // --- build tables from TRAIN (document-disjoint by story) --------------
    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut content_next: HashMap<u32, Counter> = HashMap::new();
    let mut joint_next: HashMap<(u32, (u32, u32)), Counter> = HashMap::new();
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
    for c in d4skip_next.values_mut() {
        c.cap_to_top(CAP);
    }
    let marginal_total = marginal.total();
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);
    let d4skip_entries = d4skip_next.len() as u64;
    let d4skip_mass: u64 = d4skip_next.values().map(Counter::total).sum();

    let tables = Tables {
        suffix_next,
        content_next,
        joint_next,
        d4skip_next,
        marginal_tok,
    };
    assert!(d4skip_entries > 0, "skip tables must be non-empty");

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
    let mut covered_positions = 0u64;
    let mut supported_sum = 0u64;
    let mut offered_sum = 0u64;
    let mut known_last_positions = 0u64;
    for (idx, w) in windows.iter().enumerate() {
        let partner_last = windows[swap_partner[idx]]
            .last()
            .copied()
            .unwrap_or(u32::MAX);
        preds.push(score_position(&tables, w, partner_last, lambda));
        let ev = tables.d4skip_evidence(w);
        if ev.supported > 0 {
            covered_positions += 1;
        }
        supported_sum += ev.supported as u64;
        offered_sum += ev.offered as u64;
        if w.len() >= 2 {
            known_last_positions += 1;
        }
    }

    let mut base_hits = 0u64;
    let mut mix_hits = 0u64;
    let mut d4skip_hits = 0u64;
    let mut skipmix_hits = 0u64;
    let mut skipmix_null_hits = 0u64;
    let mut skipmix_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut mix_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut d4skip_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut skipmix_vs_mix: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut skipmix_null_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut skipmix_null_changed = 0u64;
    for (idx, &i) in held_out.iter().enumerate() {
        let target = corpus.t_argmax[i];
        let p = preds[idx];
        let bh = p.base == target;
        let mh = p.mix == target;
        let dsh = p.d4skip == target;
        let smh = p.skipmix == target;
        let snh = p.skipmix_null == target;
        base_hits += u64::from(bh);
        mix_hits += u64::from(mh);
        d4skip_hits += u64::from(dsh);
        skipmix_hits += u64::from(smh);
        skipmix_null_hits += u64::from(snh);
        skipmix_vs_base.push(i8::from(smh) - i8::from(bh));
        mix_vs_base.push(i8::from(mh) - i8::from(bh));
        d4skip_vs_base.push(i8::from(dsh) - i8::from(bh));
        skipmix_vs_mix.push(i8::from(smh) - i8::from(mh));
        skipmix_null_vs_base.push(i8::from(snh) - i8::from(bh));
        skipmix_null_changed += u64::from(p.skipmix_null != p.skipmix);
    }

    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let (mix_r, _, _) = ci95_permille(mix_hits, n);
    let (d4skip_r, _, _) = ci95_permille(d4skip_hits, n);
    let (skipmix_r, skipmix_ci_lo, skipmix_ci_hi) = ci95_permille(skipmix_hits, n);
    let (skipmix_null_r, _, _) = ci95_permille(skipmix_null_hits, n);
    let (skipmix_delta, skipmix_delta_lo, skipmix_delta_hi) =
        paired_delta_permille(&skipmix_vs_base);
    let (mix_delta, mix_delta_lo, mix_delta_hi) = paired_delta_permille(&mix_vs_base);
    let (d4skip_delta, d4skip_delta_lo, d4skip_delta_hi) = paired_delta_permille(&d4skip_vs_base);
    let (sm_vs_mix_delta, sm_vs_mix_lo, sm_vs_mix_hi) = paired_delta_permille(&skipmix_vs_mix);
    let (skipmix_null_delta, skipmix_null_delta_lo, skipmix_null_delta_hi) =
        paired_delta_permille(&skipmix_null_vs_base);

    // --- minimal pairs: same suffix, different story, different teacher -----
    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, w) in windows.iter().enumerate() {
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
    }
    let mut mp_total = 0u64;
    let mut base_follow = 0u64;
    let mut skipmix_follow = 0u64;
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
                if pa.skipmix == ta && pb.skipmix == tb && pa.skipmix != pb.skipmix {
                    skipmix_follow += 1;
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
    let (skipmix_mp_rate, skipmix_mp_lo, _) = ci95_permille(skipmix_follow, mp_total.max(1));

    // --- double-run determinism check --------------------------------------
    let check_n = DOUBLE_RUN_N.min(held_out.len());
    for idx in 0..check_n {
        let partner_last = windows[swap_partner[idx]]
            .last()
            .copied()
            .unwrap_or(u32::MAX);
        let again = score_position(&tables, &windows[idx], partner_last, lambda);
        assert_eq!(again, preds[idx], "double-run drift at held-out idx {idx}");
    }

    // --- reproduction gates: D1 recorded values must reproduce -------------
    assert!(
        (base_r - REPRO_BASE_PERMILLE).abs() < 0.05,
        "suffix baseline {base_r:.1}permille does not reproduce the recorded {REPRO_BASE_PERMILLE}permille"
    );
    assert!(
        (mix_r - REPRO_MIX_PERMILLE).abs() < 0.05,
        "D1 mix arm {mix_r:.1}permille does not reproduce the recorded {REPRO_MIX_PERMILLE}permille"
    );
    assert!(
        (d4skip_r - REPRO_D4SKIP_PERMILLE).abs() < 0.05,
        "D1 d4skip arm {d4skip_r:.1}permille does not reproduce the recorded {REPRO_D4SKIP_PERMILLE}permille"
    );
    assert_eq!(
        mp_total, REPRO_MP_TOTAL,
        "minimal-pair mining does not reproduce the recorded pair count"
    );
    assert_eq!(
        base_follow, 0,
        "suffix baseline cannot follow identical-suffix pairs"
    );

    // --- control teeth ------------------------------------------------------
    assert!(mp_total > 0, "expected minimal pairs to evaluate");
    assert!(
        covered_positions > 0,
        "skip evidence must cover at least one held-out position"
    );
    assert!(
        skipmix_null_changed > 0,
        "conditioning-specificity null must change at least one prediction"
    );
    // The null separation binds only when the arm reads positive: with a
    // null-effect arm the comparison is noise and must not mask a legitimate
    // negative confirmation.
    if skipmix_delta_lo > 0.0 {
        assert!(
            skipmix_null_delta < skipmix_delta,
            "conditioning-specificity null ({skipmix_null_delta:.1}permille) must sit below the positive skipmix arm ({skipmix_delta:.1}permille)"
        );
    }

    // --- verdict (pre-registered rule; see module docs) ---------------------
    let select_1token = skipmix_delta_lo >= OPENING_BAR_PERMILLE && sm_vs_mix_lo > 0.0;
    let verdict = if select_1token {
        "SELECT-1-token — skipmix clears the 25permille opening bar AND beats the D1-selected 2-token mix (paired lower bound > 0); the lowering lowers the 1-token-conditioning skip-mix key family. The frozen 20permille end-to-end floor remains the promotion gate."
    } else if skipmix_delta_lo >= OPENING_BAR_PERMILLE {
        "LOWER-2-token-mix — skipmix clears the 25permille opening bar but does NOT beat the D1-selected 2-token mix (skipmix-vs-mix lower bound <= 0); the lowering proceeds with the D1-selected 2-token mix as measured."
    } else {
        "LOWER-2-token-mix — skipmix does not clear the 25permille opening bar; the lowering proceeds with the D1-selected 2-token mix as measured."
    };

    let elapsed = started.elapsed();

    println!("=== #897 phase-0 skipmix (1-token) key-family confirmation ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("train / held_out : {} / {}", train.len(), held_out.len());
    println!("lambda (fixed)   : {lambda}");
    println!("skip tables      : {d4skip_entries} keys, mass {d4skip_mass}");
    println!(
        "skip coverage    : {covered_positions}/{n} positions with support; mean supported tokens {:.2}/{:.2}; known-last positions {known_last_positions}",
        supported_sum as f64 / n as f64,
        offered_sum as f64 / n as f64
    );
    println!("suffix baseline  : {base_r:.1}permille  (95% CI [{base_lo:.1}, {base_hi:.1}])  [repro gate 246.6]");
    println!(
        "mix (D1, repro)  : {mix_r:.1}permille  (delta {mix_delta:.1} [{mix_delta_lo:.1}, {mix_delta_hi:.1}])  [repro gate 277.2]"
    );
    println!(
        "d4skip(D1, repro): {d4skip_r:.1}permille  (delta {d4skip_delta:.1} [{d4skip_delta_lo:.1}, {d4skip_delta_hi:.1}])  [repro gate 296.2]"
    );
    println!(
        "SKIPMIX (PRIMARY): {skipmix_r:.1}permille  (95% CI [{skipmix_ci_lo:.1}, {skipmix_ci_hi:.1}])"
    );
    println!(
        "SKIPMIX-vs-base  : {skipmix_delta:.1}permille  (paired 95% CI [{skipmix_delta_lo:.1}, {skipmix_delta_hi:.1}])  [bar {OPENING_BAR_PERMILLE}]"
    );
    println!(
        "SKIPMIX-vs-mix   : {sm_vs_mix_delta:.1}permille  (paired 95% CI [{sm_vs_mix_lo:.1}, {sm_vs_mix_hi:.1}])  [must be > 0]"
    );
    println!(
        "skipmix null     : {skipmix_null_r:.1}permille  (delta {skipmix_null_delta:.1} [{skipmix_null_delta_lo:.1}, {skipmix_null_delta_hi:.1}]; changed {skipmix_null_changed})"
    );
    println!(
        "minimal-pairs    : {mp_total} pairs; skipmix-follow {skipmix_follow} ({skipmix_mp_rate:.1}permille, 95% lo {skipmix_mp_lo:.1}); mix-follow {mix_follow}; baseline-follow {base_follow}"
    );
    println!("double-run       : {check_n} positions identical");
    println!("frozen floor     : {CAUSAL_FLOOR_PERMILLE}permille (end-to-end promotion gate; not evaluated in phase 0)");
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    let mut rec = Vec::new();
    for v in [
        n,
        base_hits,
        mix_hits,
        d4skip_hits,
        skipmix_hits,
        skipmix_null_hits,
        mp_total,
        skipmix_follow,
        mix_follow,
        base_follow,
        skipmix_null_changed,
        d4skip_entries,
        d4skip_mass,
        covered_positions,
        supported_sum,
        offered_sum,
        known_last_positions,
        marginal_total,
        train.len() as u64,
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    rec.extend_from_slice(corpus_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid       : {result_cid}");

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 897,\n",
            "  \"arm\": \"skipmix-1token-confirm-phase0\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"lambda\": {},\n",
            "  \"skip_tables\": {{\"entries\": {}, \"mass\": {}}},\n",
            "  \"skip_coverage\": {{\"covered_positions\": {}, \"mean_supported\": {:.2}, \"mean_offered\": {:.2}, \"known_last_positions\": {}}},\n",
            "  \"reproduction\": {{\"base_permille\": {:.1}, \"mix_permille\": {:.1}, \"d4skip_permille\": {:.1}, \"mp_total\": {}, \"base_follow\": {}}},\n",
            "  \"skipmix_permille\": {:.1},\n",
            "  \"skipmix_permille_ci\": [{:.1}, {:.1}],\n",
            "  \"skipmix_delta_permille\": {:.1},\n",
            "  \"skipmix_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"mix_delta_permille\": {:.1},\n",
            "  \"mix_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"d4skip_delta_permille\": {:.1},\n",
            "  \"d4skip_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"skipmix_vs_mix_delta_permille\": {:.1},\n",
            "  \"skipmix_vs_mix_ci\": [{:.1}, {:.1}],\n",
            "  \"skipmix_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"skipmix_follow\": {}, \"skipmix_follow_permille\": {:.1}, \"skipmix_follow_ci_lo\": {:.1}, \"mix_follow\": {}, \"baseline_follow\": {}}},\n",
            "  \"double_run\": {{\"checked\": {}, \"identical\": true}},\n",
            "  \"opening_bar_permille\": {:.1},\n",
            "  \"causal_floor_permille\": {:.1},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        train.len(),
        held_out.len(),
        lambda,
        d4skip_entries,
        d4skip_mass,
        covered_positions,
        supported_sum as f64 / n as f64,
        offered_sum as f64 / n as f64,
        known_last_positions,
        base_r,
        mix_r,
        d4skip_r,
        mp_total,
        base_follow,
        skipmix_r,
        skipmix_ci_lo,
        skipmix_ci_hi,
        skipmix_delta,
        skipmix_delta_lo,
        skipmix_delta_hi,
        mix_delta,
        mix_delta_lo,
        mix_delta_hi,
        d4skip_delta,
        d4skip_delta_lo,
        d4skip_delta_hi,
        sm_vs_mix_delta,
        sm_vs_mix_lo,
        sm_vs_mix_hi,
        skipmix_null_r,
        skipmix_null_delta,
        skipmix_null_delta_lo,
        skipmix_null_delta_hi,
        skipmix_null_changed,
        mp_total,
        skipmix_follow,
        skipmix_mp_rate,
        skipmix_mp_lo,
        mix_follow,
        base_follow,
        check_n,
        OPENING_BAR_PERMILLE,
        CAUSAL_FLOOR_PERMILLE,
        result_cid,
        verdict,
    );
    let out = repo_root()
        .join("docs")
        .join("skipmix_confirm_897_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
