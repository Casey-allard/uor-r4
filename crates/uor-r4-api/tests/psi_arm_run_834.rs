//! Ψ persistent-state segment-lane REFERENCE arm — teacher-grounded re-test
//! (#834 follow-up, respecting the run contract: reference-only, non-serving).
//!
//! The #834 run found the deployed model suffix-local (no prompt-conditioning
//! arm established for the fittable arms). Its three Ψ-family arms were
//! UNAVAILABLE because they are not lowered into the artifact. This harness
//! builds the #835 **segment lane** — whole-prompt content → candidate-support
//! contributions — as an offline reference arm and asks the decisive question:
//!
//!   Does whole-prompt content carry predictive power for the teacher's answer
//!   BEYOND the 2-token suffix?
//!
//! Faithful-to-#835 scope: on held-out wiki text the operative lanes are the
//! segment/entity lanes (whole-prompt content); the role/history/constraint
//! lanes are chat-session state that does not apply here. So the segment lane is
//! the right — and most favorable — lane to test on this corpus.
//!
//! Construction (document-disjoint by story; `split_positions` splits by story):
//!   * From TRAIN positions, count the recorded teacher argmax per key:
//!     `suffix_next[(a,b)]` is the 2-token suffix baseline (n-gram);
//!     `content_next[t]` is the segment-lane co-occurrence, one entry per
//!     whole-prompt content token t. Each key is capped to its top-`CAP` argmax
//!     (bounded lane, #835).
//!   * On HELD-OUT positions, score candidates two ways and take argmax:
//!     baseline uses the suffix rate only (the suffix-local floor); Ψ uses the
//!     suffix rate plus λ times the whole-prompt content rate.
//!   * Primary signal — MINIMAL PAIRS (same 2-token suffix, different story,
//!     different teacher argmax): the baseline predicts the SAME token for both
//!     (identical suffix); a working segment lane predicts DIFFERENTLY, tracking
//!     each teacher target. Baseline follow is 0 by construction; Ψ follow > 0
//!     (lower bound) is the persistent-state signal that would trigger #836.
//!
//! λ is fixed BEFORE evaluation (`LAMBDA_NUM/LAMBDA_DEN`, pre-registered); an
//! exploratory λ-sweep is reported but is NOT the verdict.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test psi_arm_run_834 -- --ignored --nocapture
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
/// Pre-registered content weight λ = LAMBDA_NUM / LAMBDA_DEN, fixed before eval.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// A Ψ effect clears this paired lower bound (‰) to count as a positive.
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;

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

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn psi_arm_run_834() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP psi_arm_run_834: no serving bundle at {}",
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
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);

    let lambda = LAMBDA_NUM / LAMBDA_DEN;

    // Score one held-out position: returns (baseline_pred, psi_pred).
    let score = |w: &[u32], lam: f64| -> (u32, u32) {
        let sk = suffix_key(w);
        let sfx = suffix_next.get(&sk);
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;

        // whole-prompt content aggregate
        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let mut uniq: Vec<u32> = w.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
        let ncontent = uniq.len().max(1) as f64;
        for t in &uniq {
            if let Some(cn) = content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *content_rate.entry(c).or_insert(0.0) += (cnt as f64 / tot) / ncontent;
                }
            }
        }

        // Baseline candidate set = suffix candidates + marginal only, so the
        // baseline prediction is a PURE function of the suffix key (a clean
        // suffix-local control: identical suffix ⇒ identical prediction).
        let mut suffix_cands: Vec<u32> = Vec::new();
        if let Some(c) = sfx {
            let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            suffix_cands.extend(v.into_iter().take(CAND_SUFFIX).map(|(k, _)| k));
        }
        suffix_cands.push(marginal_tok);
        suffix_cands.sort_unstable();
        suffix_cands.dedup();

        let mut base_scores: HashMap<u32, f64> = HashMap::new();
        for &c in &suffix_cands {
            let base = sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total;
            base_scores.insert(c, base);
        }

        // Ψ candidate set = suffix ∪ whole-prompt content (candidate-support
        // widening); score = suffix rate + λ · content rate.
        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.clone();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        let mut psi_scores: HashMap<u32, f64> = HashMap::new();
        for &c in &all_cands {
            let base = sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total;
            let cont = content_rate.get(&c).copied().unwrap_or(0.0);
            psi_scores.insert(c, base + lam * cont);
        }
        (argmax(&base_scores), argmax(&psi_scores))
    };

    // --- evaluate on HELD-OUT ----------------------------------------------
    let mut base_hits = 0u64;
    let mut psi_hits = 0u64;
    let n = held_out.len() as u64;
    let mut paired: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut windows: Vec<Vec<u32>> = Vec::with_capacity(held_out.len());
    let mut base_pred: Vec<u32> = Vec::with_capacity(held_out.len());
    let mut psi_pred: Vec<u32> = Vec::with_capacity(held_out.len());
    for &i in &held_out {
        let w = induction::context_window(&corpus, i);
        let (bp, pp) = score(&w, lambda);
        let target = corpus.t_argmax[i];
        let bh = bp == target;
        let ph = pp == target;
        if bh {
            base_hits += 1;
        }
        if ph {
            psi_hits += 1;
        }
        paired.push(i8::from(ph) - i8::from(bh));
        windows.push(w);
        base_pred.push(bp);
        psi_pred.push(pp);
    }

    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let (psi_r, psi_lo, psi_hi) = ci95_permille(psi_hits, n);
    let (delta_r, delta_lo, delta_hi) = paired_delta_permille(&paired);

    // --- minimal pairs: same suffix, different story, different teacher -----
    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, &i) in held_out.iter().enumerate() {
        let w = &windows[idx];
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
        let _ = i;
    }
    let mut mp_total = 0u64;
    let mut psi_follow = 0u64;
    let mut base_follow = 0u64;
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
                if psi_pred[xa] == ta && psi_pred[xb] == tb && psi_pred[xa] != psi_pred[xb] {
                    psi_follow += 1;
                }
                if base_pred[xa] == ta && base_pred[xb] == tb && base_pred[xa] != base_pred[xb] {
                    base_follow += 1;
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let (mp_rate, mp_lo, _mp_hi) = ci95_permille(psi_follow, mp_total.max(1));

    // --- exploratory λ-sweep (NOT the verdict) -----------------------------
    let sweep_lams = [0.5f64, 1.0, 2.0, 4.0, 8.0];
    let mut sweep: Vec<(f64, u64)> = Vec::new();
    for &lam in &sweep_lams {
        let mut h = 0u64;
        for (idx, &i) in held_out.iter().enumerate() {
            let (_, pp) = score(&windows[idx], lam);
            if pp == corpus.t_argmax[i] {
                h += 1;
            }
        }
        sweep.push((lam, h));
    }

    // --- verdict ------------------------------------------------------------
    let verdict = if delta_lo >= CAUSAL_FLOOR_PERMILLE || mp_lo > 0.0 {
        "SELECT: persistent-state (segment lane) — whole-prompt content beats the suffix floor; a positive that would trigger #836"
    } else if delta_lo <= 0.0 && psi_follow == 0 {
        "NO ARM — whole-prompt content adds no predictive signal over the suffix; the segment-lane mechanism does not help"
    } else {
        "REVISE — a weak/uncertain segment-lane effect below the causal floor"
    };

    let elapsed = started.elapsed();
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #834 Ψ segment-lane reference-arm re-test ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("train / held_out : {} / {}", train.len(), held_out.len());
    println!("lambda (fixed)   : {lambda}");
    println!("suffix baseline  : {base_r:.1}permille  (95% CI [{base_lo:.1}, {base_hi:.1}])");
    println!("Ψ (seg lane)     : {psi_r:.1}permille  (95% CI [{psi_lo:.1}, {psi_hi:.1}])");
    println!(
        "Ψ-DELTA          : {delta_r:.1}permille  (paired 95% CI [{delta_lo:.1}, {delta_hi:.1}])  psi-minus-suffix"
    );
    let sweep_str: String = sweep
        .iter()
        .map(|(l, h)| format!("λ{}={:.0} ", l, *h as f64 / n as f64 * 1000.0))
        .collect();
    println!("λ-sweep (explor.): {sweep_str}(baseline={base_r:.0})permille");
    println!(
        "minimal-pairs    : {mp_total} pairs; Ψ-follow {psi_follow} ({mp_rate:.1}permille, 95% lo {mp_lo:.1}); baseline-follow {base_follow}"
    );
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    let mut rec = Vec::new();
    for v in [
        n,
        base_hits,
        psi_hits,
        mp_total,
        psi_follow,
        base_follow,
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
            "  \"arm\": \"psi-segment-lane-reference\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"lambda\": {},\n",
            "  \"suffix_baseline_permille\": {:.1},\n",
            "  \"psi_permille\": {:.1},\n",
            "  \"psi_permille_ci\": [{:.1}, {:.1}],\n",
            "  \"psi_delta_permille\": {:.1},\n",
            "  \"psi_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"lambda_sweep_permille\": {},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"psi_follow\": {}, \"psi_follow_permille\": {:.1}, \"psi_follow_ci_lo\": {:.1}, \"baseline_follow\": {}}},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        train.len(),
        held_out.len(),
        lambda,
        base_r,
        psi_r,
        psi_lo,
        psi_hi,
        delta_r,
        delta_lo,
        delta_hi,
        sweep_json,
        mp_total,
        psi_follow,
        mp_rate,
        mp_lo,
        base_follow,
        result_cid,
        verdict,
    );
    let out = repo_root().join("docs").join("psi_arm_834_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());

    assert!(mp_total > 0, "expected minimal pairs to evaluate");
    assert_eq!(
        base_follow, 0,
        "suffix baseline cannot follow identical-suffix pairs"
    );
}
