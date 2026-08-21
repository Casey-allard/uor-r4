//! `conditional-residuals` REFERENCE arm — teacher-grounded run (#834, S1-B).
//!
//! The last unmeasured arm of #834's five-arm scope. `current-scoring` and
//! `longer-local-context` measured negative (bake-off record §6.1, PR #874:
//! the deployed model is suffix-local). `persistent-state` — the Ψ segment
//! lane — was positive off-serving (§6.2, PR #875: +17.5‰, CI [15.9, 19.0])
//! but was retired sub-floor after lowering (#836 reachability ceiling
//! 19.0‰ < the 20‰ floor; #886 lowering-fidelity gap; #887: the bar stands).
//! `candidate-support-expansion` (#888) is closed off. This harness builds
//! and measures the remaining arm per the maintainer decision recorded on
//! #834 (2026-08-21): `conditional-residuals` — persistent state minus the
//! corpus marginal ("evidence beyond the marginal", record §2).
//!
//! ## Mechanism under test
//!
//! The Ψ segment lane promotes whatever the whole-prompt content co-occurs
//! with — including tokens that co-occur with EVERYTHING (the corpus-marginal
//! pathology: #784 found 11/15 distinct rows favoring newline). The
//! conditional-residuals arm subtracts the corpus-marginal rate from the
//! content evidence, so a candidate is promoted only where the prompt's
//! conditional evidence EXCEEDS the corpus prior:
//!
//!   score(c) = suffix_rate(c) + λ · (content_rate(c) − marginal_rate(c))
//!
//! against the Ψ comparator score(c) = suffix_rate(c) + λ · content_rate(c),
//! over the IDENTICAL candidate set and the identical tables — so the CR-vs-Ψ
//! contrast is a pure scoring-rule ablation, and CR-vs-baseline is the arm's
//! headline paired delta.
//!
//! ## Construction (identical to the #875 precedent where shared)
//!
//! Fit on the 288,794 document-disjoint TRAIN positions of the #833 canonical
//! bundle (`split_positions` splits by story): `suffix_next[(a,b)]` (2-token
//! suffix → teacher-argmax counts, top-64 per key), `content_next[t]`
//! (whole-prompt content token → teacher-argmax counts, top-64 per key), and
//! the UNCAPPED corpus `marginal` (teacher-argmax counts over all of TRAIN).
//! Scored teacher-grounded on all 72,130 held-out positions via the recorded
//! teacher argmax — no live teacher forward, deterministic.
//!
//! Arms/controls evaluated in one pass with a fixed argmax decoder
//! (score desc, id asc — decoder-held-constant):
//!   * `base`      — suffix rate over suffix candidates (pure function of the
//!     suffix key; the suffix-local floor).
//!   * `psi`       — the #875 segment lane, recomputed as a harness-correctness
//!     reproduction gate (its §6.2 numbers must reproduce exactly).
//!   * `cr`        — PRIMARY: conditional residuals over the Ψ-widened
//!     candidate set.
//!   * `cr-narrow` — the same residual scoring over the suffix candidate set
//!     only (separates score-conditioning from candidate-availability inside
//!     the CR mechanism).
//!   * `swap`      — prompt-swap null: CR scored with the content evidence of
//!     a different-story held-out window (must sit below the real arm).
//!   * `rshuf`     — residual-shuffle null: CR with each candidate's
//!     subtracted marginal taken from the NEXT candidate id in the sorted
//!     candidate set (the specific marginal alignment must matter).
//!   * `trivial`   — the corpus-marginal argmax (no-context floor).
//!
//! Primary signal: paired CR-minus-base delta on all held-out positions, plus
//! the minimal-pairs test (same 2-token suffix, different story, different
//! teacher argmax — the suffix baseline follows 0 by construction).
//!
//! ## Pre-registered decision rule (posted to #834 before this run)
//!
//! λ is fixed at 1.0 before evaluation; the λ-sweep is exploratory, NOT the
//! verdict. Following the #887 governance verdict (the 20‰ floor STANDS for
//! this arm class), the SELECT rule is STRICTER than the #875 rule: a
//! minimal-pairs lower bound alone can no longer produce SELECT.
//!   * SELECT  iff the CR paired-delta 95% lower bound ≥ 20‰ (the frozen
//!     `CAUSAL_FLOOR_PERMILLE`).
//!   * NO ARM  iff the lower bound ≤ 0 and CR follows 0 minimal pairs.
//!   * REVISE  otherwise (real-but-sub-floor signal; no lowering — the #887
//!     bar stands).
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test conditional_residuals_run_834 \
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
/// Per-key cap on retained argmax (bounded segment-lane residual table, #835).
const CAP: usize = 64;
/// Candidate-set widths from the suffix baseline and the content evidence.
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
/// Pre-registered residual weight λ = LAMBDA_NUM / LAMBDA_DEN, fixed before eval.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// The frozen causal floor (‰) the paired lower bound must clear for SELECT
/// (#887: the bar stands for this arm class).
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// Positions re-scored by the in-harness double-run determinism check.
const DOUBLE_RUN_N: usize = 2_000;
/// Reproduction gate: the #875 §6.2 recorded values this harness must
/// reproduce exactly (same corpus, same split, same construction).
const REPRO_BASE_PERMILLE: f64 = 246.6;
const REPRO_PSI_PERMILLE: f64 = 264.1;
const REPRO_MP_TOTAL: u64 = 4_722;
const REPRO_PSI_FOLLOW: u64 = 10;

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
/// `CAP` by count (ties broken by the smaller token id) with the total retained.
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

/// The fitted tables, shared by every arm and control.
struct Tables {
    suffix_next: HashMap<(u32, u32), Counter>,
    content_next: HashMap<u32, Counter>,
    marginal: Counter,
    marginal_total: u64,
    marginal_tok: u32,
}

impl Tables {
    fn marginal_rate(&self, c: u32) -> f64 {
        self.marginal.map.get(&c).copied().unwrap_or(0) as f64 / self.marginal_total.max(1) as f64
    }

    /// Whole-prompt content aggregate of a window — the #875 construction,
    /// verbatim: mean over the window's unique tokens of each token's
    /// capped argmax rate.
    fn content_rates(&self, w: &[u32]) -> HashMap<u32, f64> {
        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let mut uniq: Vec<u32> = w.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
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
    /// deduped) — a pure function of the suffix key, as in #875.
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

    /// Ψ-widened candidate set: suffix candidates ∪ top `CAND_CONTENT`
    /// content candidates by content rate — identical widening for Ψ and CR,
    /// so their contrast is a pure scoring-rule ablation.
    fn widened_cands(&self, suffix_cands: &[u32], content_rate: &HashMap<u32, f64>) -> Vec<u32> {
        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.to_vec();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        all_cands
    }

    fn suffix_rate(&self, w: &[u32], c: u32) -> f64 {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total
    }
}

/// One held-out position's predictions across every arm and control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Preds {
    base: u32,
    psi: u32,
    cr: u32,
    narrow: u32,
    rshuf: u32,
}

/// Score one position's non-swap arms. `lam` is the residual weight.
fn score_position(t: &Tables, w: &[u32], lam: f64) -> Preds {
    let content_rate = t.content_rates(w);
    let suffix_cands = t.suffix_cands(w);
    let all_cands = t.widened_cands(&suffix_cands, &content_rate);

    let mut base_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &suffix_cands {
        base_scores.insert(c, t.suffix_rate(w, c));
    }

    let mut psi_scores: HashMap<u32, f64> = HashMap::new();
    let mut cr_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &all_cands {
        let base = t.suffix_rate(w, c);
        let cont = content_rate.get(&c).copied().unwrap_or(0.0);
        psi_scores.insert(c, base + lam * cont);
        cr_scores.insert(c, base + lam * (cont - t.marginal_rate(c)));
    }

    let mut narrow_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &suffix_cands {
        let base = t.suffix_rate(w, c);
        let cont = content_rate.get(&c).copied().unwrap_or(0.0);
        narrow_scores.insert(c, base + lam * (cont - t.marginal_rate(c)));
    }

    // Residual-shuffle null: subtract the NEXT sorted candidate's marginal
    // (rotate by one within the sorted widened candidate set).
    let mut rshuf_scores: HashMap<u32, f64> = HashMap::new();
    for (i, &c) in all_cands.iter().enumerate() {
        let rot = all_cands[(i + 1) % all_cands.len()];
        let base = t.suffix_rate(w, c);
        let cont = content_rate.get(&c).copied().unwrap_or(0.0);
        rshuf_scores.insert(c, base + lam * (cont - t.marginal_rate(rot)));
    }

    Preds {
        base: argmax(&base_scores),
        psi: argmax(&psi_scores),
        cr: argmax(&cr_scores),
        narrow: argmax(&narrow_scores),
        rshuf: argmax(&rshuf_scores),
    }
}

/// Prompt-swap null: CR scored with the suffix of `w_suffix` but the content
/// evidence (and content widening) of the different-story `w_content`.
fn score_swap(t: &Tables, w_suffix: &[u32], w_content: &[u32], lam: f64) -> u32 {
    let content_rate = t.content_rates(w_content);
    let suffix_cands = t.suffix_cands(w_suffix);
    let all_cands = t.widened_cands(&suffix_cands, &content_rate);
    let mut scores: HashMap<u32, f64> = HashMap::new();
    for &c in &all_cands {
        let base = t.suffix_rate(w_suffix, c);
        let cont = content_rate.get(&c).copied().unwrap_or(0.0);
        scores.insert(c, base + lam * (cont - t.marginal_rate(c)));
    }
    argmax(&scores)
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn conditional_residuals_run_834() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP conditional_residuals_run_834: no serving bundle at {}",
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
    let mut marginal = Counter::default();
    for &i in &train {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        marginal.bump(target);
        suffix_next.entry(suffix_key(&w)).or_default().bump(target);
        let mut seen: Vec<u32> = w.clone();
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            content_next.entry(t).or_default().bump(target);
        }
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in content_next.values_mut() {
        c.cap_to_top(CAP);
    }
    // The corpus marginal stays UNCAPPED: it is the prior being conditioned
    // against, not a bounded lane table.
    let marginal_total = marginal.total();
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);
    let marginal_entries = marginal.map.len();

    let tables = Tables {
        suffix_next,
        content_next,
        marginal,
        marginal_total,
        marginal_tok,
    };

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
    for (idx, w) in windows.iter().enumerate() {
        preds.push(score_position(&tables, w, lambda));
        swap_preds.push(score_swap(&tables, w, &windows[swap_partner[idx]], lambda));
    }

    let mut base_hits = 0u64;
    let mut psi_hits = 0u64;
    let mut cr_hits = 0u64;
    let mut narrow_hits = 0u64;
    let mut swap_hits = 0u64;
    let mut rshuf_hits = 0u64;
    let mut trivial_hits = 0u64;
    let mut cr_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut cr_vs_psi: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut psi_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut narrow_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut swap_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut rshuf_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut swap_changed = 0u64;
    let mut rshuf_changed = 0u64;
    for (idx, &i) in held_out.iter().enumerate() {
        let target = corpus.t_argmax[i];
        let p = preds[idx];
        let bh = p.base == target;
        let ph = p.psi == target;
        let ch = p.cr == target;
        let nh = p.narrow == target;
        let sh = swap_preds[idx] == target;
        let rh = p.rshuf == target;
        base_hits += u64::from(bh);
        psi_hits += u64::from(ph);
        cr_hits += u64::from(ch);
        narrow_hits += u64::from(nh);
        swap_hits += u64::from(sh);
        rshuf_hits += u64::from(rh);
        trivial_hits += u64::from(tables.marginal_tok == target);
        cr_vs_base.push(i8::from(ch) - i8::from(bh));
        cr_vs_psi.push(i8::from(ch) - i8::from(ph));
        psi_vs_base.push(i8::from(ph) - i8::from(bh));
        narrow_vs_base.push(i8::from(nh) - i8::from(bh));
        swap_vs_base.push(i8::from(sh) - i8::from(bh));
        rshuf_vs_base.push(i8::from(rh) - i8::from(bh));
        swap_changed += u64::from(swap_preds[idx] != p.cr);
        rshuf_changed += u64::from(p.rshuf != p.cr);
    }

    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let (psi_r, _psi_lo, _psi_hi) = ci95_permille(psi_hits, n);
    let (cr_r, cr_lo, cr_hi) = ci95_permille(cr_hits, n);
    let (narrow_r, _, _) = ci95_permille(narrow_hits, n);
    let (swap_r, _, _) = ci95_permille(swap_hits, n);
    let (rshuf_r, _, _) = ci95_permille(rshuf_hits, n);
    let (trivial_r, _, _) = ci95_permille(trivial_hits, n);
    let (cr_delta, cr_delta_lo, cr_delta_hi) = paired_delta_permille(&cr_vs_base);
    let (crpsi_delta, crpsi_lo, crpsi_hi) = paired_delta_permille(&cr_vs_psi);
    let (psi_delta, psi_delta_lo, psi_delta_hi) = paired_delta_permille(&psi_vs_base);
    let (narrow_delta, narrow_delta_lo, narrow_delta_hi) = paired_delta_permille(&narrow_vs_base);
    let (swap_delta, swap_delta_lo, swap_delta_hi) = paired_delta_permille(&swap_vs_base);
    let (rshuf_delta, rshuf_delta_lo, rshuf_delta_hi) = paired_delta_permille(&rshuf_vs_base);

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
    let mut narrow_follow = 0u64;
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
                if pa.narrow == ta && pb.narrow == tb && pa.narrow != pb.narrow {
                    narrow_follow += 1;
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let (cr_mp_rate, cr_mp_lo, _cr_mp_hi) = ci95_permille(cr_follow, mp_total.max(1));

    // --- exploratory λ-sweep for CR (NOT the verdict) ----------------------
    let sweep_lams = [0.5f64, 1.0, 2.0, 4.0, 8.0];
    let mut sweep: Vec<(f64, u64)> = Vec::new();
    for &lam in &sweep_lams {
        let mut h = 0u64;
        for (idx, &i) in held_out.iter().enumerate() {
            let p = score_position(&tables, &windows[idx], lam);
            if p.cr == corpus.t_argmax[i] {
                h += 1;
            }
        }
        sweep.push((lam, h));
    }

    // --- double-run determinism check --------------------------------------
    let check_n = DOUBLE_RUN_N.min(held_out.len());
    for idx in 0..check_n {
        let again = score_position(&tables, &windows[idx], lambda);
        assert_eq!(again, preds[idx], "double-run drift at held-out idx {idx}");
        let swap_again = score_swap(&tables, &windows[idx], &windows[swap_partner[idx]], lambda);
        assert_eq!(
            swap_again, swap_preds[idx],
            "double-run swap drift at held-out idx {idx}"
        );
    }

    // --- reproduction gate: the #875 §6.2 numbers must reproduce -----------
    assert!(
        (base_r - REPRO_BASE_PERMILLE).abs() < 0.05,
        "suffix baseline {base_r:.1}permille does not reproduce the recorded {REPRO_BASE_PERMILLE}permille"
    );
    assert!(
        (psi_r - REPRO_PSI_PERMILLE).abs() < 0.05,
        "psi arm {psi_r:.1}permille does not reproduce the recorded {REPRO_PSI_PERMILLE}permille"
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
        base_follow, 0,
        "suffix baseline cannot follow identical-suffix pairs"
    );

    // --- control teeth ------------------------------------------------------
    assert!(mp_total > 0, "expected minimal pairs to evaluate");
    assert!(
        swap_changed > 0,
        "prompt-swap null must change at least one prediction"
    );
    assert!(
        rshuf_changed > 0,
        "residual-shuffle null must change at least one prediction"
    );
    // The swap-null separation binds only when the arm reads positive: with a
    // null-effect arm the comparison is noise and must not mask a legitimate
    // NO-ARM verdict.
    if cr_delta_lo > 0.0 {
        assert!(
            swap_delta < cr_delta,
            "prompt-swap null ({swap_delta:.1}permille) must sit below a positive arm ({cr_delta:.1}permille)"
        );
    }
    assert!(
        cr_hits > trivial_hits,
        "the arm must beat the trivial no-context floor"
    );

    // --- verdict (pre-registered rule; see module docs) ---------------------
    let verdict = if cr_delta_lo >= CAUSAL_FLOOR_PERMILLE {
        "SELECT: conditional-residuals — the paired lower bound clears the frozen 20permille causal floor off-serving"
    } else if cr_delta_lo <= 0.0 && cr_follow == 0 {
        "NO ARM — conditional evidence beyond the corpus marginal adds no teacher-predictive signal over the suffix floor"
    } else {
        "REVISE — real but sub-floor conditional-residuals signal; below the frozen 20permille floor (#887: the bar stands), so no lowering track opens"
    };

    let elapsed = started.elapsed();
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #834 conditional-residuals reference-arm run ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("train / held_out : {} / {}", train.len(), held_out.len());
    println!("lambda (fixed)   : {lambda}");
    println!(
        "marginal table   : {marginal_entries} entries, total {marginal_total}, argmax tok {marginal_tok}"
    );
    println!("suffix baseline  : {base_r:.1}permille  (95% CI [{base_lo:.1}, {base_hi:.1}])");
    println!("psi (reproduced) : {psi_r:.1}permille  (delta {psi_delta:.1} [{psi_delta_lo:.1}, {psi_delta_hi:.1}])");
    println!("CR (PRIMARY)     : {cr_r:.1}permille  (95% CI [{cr_lo:.1}, {cr_hi:.1}])");
    println!(
        "CR-DELTA vs base : {cr_delta:.1}permille  (paired 95% CI [{cr_delta_lo:.1}, {cr_delta_hi:.1}])"
    );
    println!(
        "CR vs psi        : {crpsi_delta:.1}permille  (paired 95% CI [{crpsi_lo:.1}, {crpsi_hi:.1}])"
    );
    println!(
        "CR-narrow        : {narrow_r:.1}permille  (delta {narrow_delta:.1} [{narrow_delta_lo:.1}, {narrow_delta_hi:.1}])"
    );
    println!(
        "swap null        : {swap_r:.1}permille  (delta {swap_delta:.1} [{swap_delta_lo:.1}, {swap_delta_hi:.1}]; changed {swap_changed})"
    );
    println!(
        "rshuf null       : {rshuf_r:.1}permille  (delta {rshuf_delta:.1} [{rshuf_delta_lo:.1}, {rshuf_delta_hi:.1}]; changed {rshuf_changed})"
    );
    println!("trivial prior    : {trivial_r:.1}permille");
    let sweep_str: String = sweep
        .iter()
        .map(|(l, h)| format!("λ{}={:.0} ", l, *h as f64 / n as f64 * 1000.0))
        .collect();
    println!("λ-sweep (explor.): {sweep_str}(baseline={base_r:.0})permille");
    println!(
        "minimal-pairs    : {mp_total} pairs; CR-follow {cr_follow} ({cr_mp_rate:.1}permille, 95% lo {cr_mp_lo:.1}); psi-follow {psi_follow}; narrow-follow {narrow_follow}; baseline-follow {base_follow}"
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
        narrow_hits,
        swap_hits,
        rshuf_hits,
        trivial_hits,
        mp_total,
        cr_follow,
        psi_follow,
        narrow_follow,
        base_follow,
        swap_changed,
        rshuf_changed,
        marginal_total,
        marginal_entries as u64,
        marginal_tok as u64,
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
            "  \"issue\": 834,\n",
            "  \"arm\": \"conditional-residuals-reference\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"lambda\": {},\n",
            "  \"marginal_table\": {{\"entries\": {}, \"total\": {}, \"argmax_tok\": {}}},\n",
            "  \"suffix_baseline_permille\": {:.1},\n",
            "  \"psi_reproduction\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"mp_total\": {}, \"psi_follow\": {}}},\n",
            "  \"cr_permille\": {:.1},\n",
            "  \"cr_permille_ci\": [{:.1}, {:.1}],\n",
            "  \"cr_delta_permille\": {:.1},\n",
            "  \"cr_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"cr_vs_psi_delta_permille\": {:.1},\n",
            "  \"cr_vs_psi_ci\": [{:.1}, {:.1}],\n",
            "  \"cr_narrow_permille\": {:.1},\n",
            "  \"cr_narrow_delta_permille\": {:.1},\n",
            "  \"cr_narrow_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"swap_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"rshuf_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"trivial_prior_permille\": {:.1},\n",
            "  \"lambda_sweep_permille\": {},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"cr_follow\": {}, \"cr_follow_permille\": {:.1}, \"cr_follow_ci_lo\": {:.1}, \"psi_follow\": {}, \"narrow_follow\": {}, \"baseline_follow\": {}}},\n",
            "  \"double_run\": {{\"checked\": {}, \"identical\": true}},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        train.len(),
        held_out.len(),
        lambda,
        marginal_entries,
        marginal_total,
        marginal_tok,
        base_r,
        psi_r,
        psi_delta,
        psi_delta_lo,
        psi_delta_hi,
        mp_total,
        psi_follow,
        cr_r,
        cr_lo,
        cr_hi,
        cr_delta,
        cr_delta_lo,
        cr_delta_hi,
        crpsi_delta,
        crpsi_lo,
        crpsi_hi,
        narrow_r,
        narrow_delta,
        narrow_delta_lo,
        narrow_delta_hi,
        swap_r,
        swap_delta,
        swap_delta_lo,
        swap_delta_hi,
        swap_changed,
        rshuf_r,
        rshuf_delta,
        rshuf_delta_lo,
        rshuf_delta_hi,
        rshuf_changed,
        trivial_r,
        sweep_json,
        mp_total,
        cr_follow,
        cr_mp_rate,
        cr_mp_lo,
        psi_follow,
        narrow_follow,
        base_follow,
        check_n,
        result_cid,
        verdict,
    );
    let out = repo_root()
        .join("docs")
        .join("conditional_residuals_834_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
