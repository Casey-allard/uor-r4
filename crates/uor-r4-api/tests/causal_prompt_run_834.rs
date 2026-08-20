//! Teacher-grounded S1 causal-influence RUN on the real #833 canonical bundle
//! (#834 — research/#822-B). Ignored heavy harness.
//!
//! ## What this measures (and why it is teacher-grounded without a live teacher)
//!
//! When the #833 broad bundle was compiled, the SmolLM2 teacher was run and its
//! per-position next-token answer was recorded: `Corpus::t_argmax[i]` is the
//! teacher argmax at held-out position `i`, over the real Simple-Wiki text in
//! `Corpus::input`. So the teacher-forced targets are already pinned in the
//! artifact — this harness reads them and drives the deployed engine, no live
//! teacher forward needed.
//!
//! The question is the S1 bottleneck: does prompt meaning move the deployed
//! model's prediction beyond suffix / exact-context memory? With EXCT disabled
//! at load (the serving-eval policy, #280), for each real held-out position we
//! compare the deployed model's top-1 agreement with the teacher argmax under:
//!
//!   * full-context  — the whole in-story context window (the longer-context arm),
//!   * suffix-only    — the last `SUFFIX_K` tokens (the current-scoring floor),
//!   * prompt-swap    — an unrelated story's window (a causal-influence null),
//!   * trivial-prior  — the corpus marginal argmax (the no-context floor).
//!
//! Primary statistic: causal-influence-delta = full-context minus suffix-only
//! top-1 agreement with the teacher (paired), with the prompt-swap and trivial
//! nulls held well below it. Plus a paired minimal-pairs test mined from the
//! corpus itself: positions in different stories with the SAME `SUFFIX_K` suffix
//! but DIFFERENT teacher argmax — a prompt-conditioned model predicts
//! differently and tracks each teacher target; a suffix-local model (identical
//! suffix ⇒ identical prediction) cannot, so its follow rate is zero by
//! construction.
//!
//! Scope: the runnable arms here are window-length variants of the ONE deployed
//! scorer (current-scoring vs longer-context). The three #835 Ψ-family arms
//! (persistent-state, conditional-residuals, candidate-support-expansion) are
//! the mechanisms #836 would build and are recorded UNAVAILABLE here — the
//! engine's public surface returns the selected token, not a candidate score
//! vector. A negative verdict is the #834 "if negative" path: build the #835
//! mechanisms (#836) and re-test.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test causal_prompt_run_834 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir), R4_CAUSAL_N (max sampled positions).

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

/// The suffix length that defines the "current-scoring / suffix-only" floor and
/// the minimal-pair key.
const SUFFIX_K: usize = 2;
/// Default cap on sampled held-out positions (a deterministic stride subsample).
const DEFAULT_N: usize = 20_000;
/// Minimum decision-relevant effect for the causal arm's paired lower bound (‰).
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// The prompt-swap null must sit at least this far below the full-context arm
/// for the reading to be non-vacuous (‰).
const SWAP_MARGIN_PERMILLE: f64 = 20.0;
/// Cap on minimal pairs formed per suffix key (bounds the paired test's work).
const PAIRS_PER_KEY: usize = 1;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/uor-r4-api
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

/// 95% normal-approximation confidence interval for a binomial rate, in
/// parts-per-thousand: returns (rate, low, high).
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

/// Paired difference-of-proportions with a 95% CI, in ‰: for each unit a
/// signed indicator `d ∈ {-1,0,1}` (a_hit - b_hit). Returns (delta, low, high).
fn paired_delta_permille(d: &[i8]) -> (f64, f64, f64) {
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let sum: f64 = d.iter().map(|&x| x as f64).sum();
    let mean = sum / n as f64;
    let var = d
        .iter()
        .map(|&x| {
            let c = x as f64 - mean;
            c * c
        })
        .sum::<f64>()
        / n as f64;
    let se = (var / n as f64).sqrt();
    let half = 1.96 * se;
    (
        (mean) * 1000.0,
        (mean - half) * 1000.0,
        (mean + half) * 1000.0,
    )
}

fn predict_token(engine: &mut R4Engine, window: &[u32]) -> (Option<u32>, bool) {
    match engine.predict_decision(window) {
        Ok(PredictDecision::Serve(outcome)) => (Some(outcome.token), outcome.ngram_hit),
        Ok(PredictDecision::Abstain(a)) => (None, a.ngram_hit),
        Err(_) => (None, false),
    }
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle; run explicitly with --ignored"]
// Several passes index parallel per-sample arrays (sample/full/suffix/swap) by
// the same position index; enumerate() over one does not remove the others.
#[allow(clippy::needless_range_loop)]
fn causal_prompt_run_834() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP causal_prompt_run_834: no serving bundle at {} (set R4_CAUSAL_BUNDLE)",
            root.display()
        );
        return;
    };

    let graph_bytes = std::fs::read(&bundle.graph).expect("graph bytes");
    let teacher_bytes = std::fs::read(&bundle.teacher).expect("teacher bytes");
    let score_report = bundle
        .graph
        .parent()
        .and_then(|p| std::fs::read(p.join("score_report.json")).ok())
        .filter(|b| serde_json::from_slice::<serde_json::Value>(b).is_ok());
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();

    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (_, held_out) = induction::split_positions(&corpus);
    assert!(!held_out.is_empty(), "held-out partition must be non-empty");

    let n_cap = std::env::var("R4_CAUSAL_N")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_N)
        .max(1);
    let stride = (held_out.len() / n_cap).max(1);
    let sample: Vec<usize> = held_out.iter().copied().step_by(stride).collect();
    let n = sample.len();
    assert!(n >= 100, "need a powered sample, got {n}");

    // Measurement, not deployment: accept the artifact regardless of the
    // deploy-time quality gate (the graph runtime top-1 sits below the TLA
    // baseline by construction; we are measuring that artifact, not shipping it).
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &teacher_bytes,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: score_report.as_deref(),
    })
    .expect("engine load");

    // Precompute the full in-story window for each sampled position.
    let windows: Vec<Vec<u32>> = sample
        .iter()
        .map(|&i| induction::context_window(&corpus, i))
        .collect();
    // The full-vs-suffix contrast is only meaningful where windows exceed the
    // suffix length; record how many do.
    let win_over_k = windows.iter().filter(|w| w.len() > SUFFIX_K).count() as u64;
    let win_max = windows.iter().map(|w| w.len()).max().unwrap_or(0);
    let win_mean = windows.iter().map(|w| w.len()).sum::<usize>() as f64 / n as f64;

    let started = Instant::now();

    // Pass 1 — full context.
    engine.reset();
    let mut full_pred: Vec<Option<u32>> = Vec::with_capacity(n);
    let mut served = 0u64;
    let mut ngram_hits = 0u64;
    for w in &windows {
        let (tok, ng) = predict_token(&mut engine, w);
        if tok.is_some() {
            served += 1;
        }
        if ng {
            ngram_hits += 1;
        }
        full_pred.push(tok);
    }

    // Pass 2 — suffix-only (last SUFFIX_K tokens of the window).
    engine.reset();
    let mut suffix_pred: Vec<Option<u32>> = Vec::with_capacity(n);
    for w in &windows {
        let start = w.len().saturating_sub(SUFFIX_K);
        let (tok, _) = predict_token(&mut engine, &w[start..]);
        suffix_pred.push(tok);
    }

    // Pass 3 — prompt-swap: predict target i from an unrelated story's window.
    engine.reset();
    let mut swap_pred: Vec<Option<u32>> = Vec::with_capacity(n);
    for s in 0..n {
        let i = sample[s];
        let mut d = (s + n / 2) % n;
        // ensure a different story (bounded linear probe)
        let mut guard = 0;
        while corpus.story[sample[d]] == corpus.story[i] && guard < n {
            d = (d + 1) % n;
            guard += 1;
        }
        let (tok, _) = predict_token(&mut engine, &windows[d]);
        swap_pred.push(tok);
    }

    // Context-saturation sweep — top-1 agreement with the teacher at increasing
    // suffix lengths. If it is flat up to the full window, additional prompt
    // context adds no predictive signal (the airtight form of the negative).
    let sweep_ks: [usize; 5] = [1, 2, 3, 4, 6];
    let mut sweep: Vec<(usize, u64)> = Vec::new();
    for &k in &sweep_ks {
        engine.reset();
        let mut hits = 0u64;
        for (s, w) in windows.iter().enumerate() {
            let start = w.len().saturating_sub(k);
            let (tok, _) = predict_token(&mut engine, &w[start..]);
            if tok == Some(corpus.t_argmax[sample[s]]) {
                hits += 1;
            }
        }
        sweep.push((k, hits));
    }

    // Trivial prior — the marginal teacher argmax over the sample (a constant).
    let mut freq: HashMap<u32, u64> = HashMap::new();
    for &i in &sample {
        *freq.entry(corpus.t_argmax[i]).or_insert(0) += 1;
    }
    let marginal = freq
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);

    // Rates vs the recorded teacher argmax.
    let mut full_hits = 0u64;
    let mut suffix_hits = 0u64;
    let mut swap_hits = 0u64;
    let mut trivial_hits = 0u64;
    let mut paired: Vec<i8> = Vec::with_capacity(n);
    // Attribution: context-helped / suffix-sufficed / miss.
    let (mut ctx_helped, mut suffix_sufficed, mut miss) = (0u64, 0u64, 0u64);
    for s in 0..n {
        let target = corpus.t_argmax[sample[s]];
        let f = full_pred[s] == Some(target);
        let su = suffix_pred[s] == Some(target);
        if f {
            full_hits += 1;
        }
        if su {
            suffix_hits += 1;
        }
        if swap_pred[s] == Some(target) {
            swap_hits += 1;
        }
        if marginal == target {
            trivial_hits += 1;
        }
        paired.push(i8::from(f) - i8::from(su));
        match (f, su) {
            (true, false) => ctx_helped += 1,
            (true, true) => suffix_sufficed += 1,
            _ => miss += 1,
        }
    }

    let (full_r, full_lo, _full_hi) = ci95_permille(full_hits, n as u64);
    let (suffix_r, _suffix_lo, _suffix_hi) = ci95_permille(suffix_hits, n as u64);
    let (swap_r, _swap_lo, swap_hi) = ci95_permille(swap_hits, n as u64);
    let (trivial_r, ..) = ci95_permille(trivial_hits, n as u64);
    let (delta_r, delta_lo, delta_hi) = paired_delta_permille(&paired);

    // Minimal-pairs test: same SUFFIX_K suffix, different story, different
    // teacher argmax. A prompt-conditioned model follows the differing meaning.
    let mut by_suffix: HashMap<Vec<u32>, Vec<usize>> = HashMap::new();
    for s in 0..n {
        let w = &windows[s];
        if w.len() < SUFFIX_K {
            continue;
        }
        let key = w[w.len() - SUFFIX_K..].to_vec();
        by_suffix.entry(key).or_default().push(s);
    }
    let mut mp_total = 0u64;
    let mut mp_follow = 0u64;
    let mut mp_suffix_identical = 0u64;
    for group in by_suffix.values() {
        let mut made = 0usize;
        'outer: for a in 0..group.len() {
            for b in (a + 1)..group.len() {
                let (sa, sb) = (group[a], group[b]);
                let (ia, ib) = (sample[sa], sample[sb]);
                if corpus.story[ia] == corpus.story[ib] {
                    continue;
                }
                let (ta, tb) = (corpus.t_argmax[ia], corpus.t_argmax[ib]);
                if ta == tb {
                    continue;
                }
                mp_total += 1;
                // suffix-only control: identical suffix ⇒ identical prediction.
                if suffix_pred[sa] == suffix_pred[sb] {
                    mp_suffix_identical += 1;
                }
                // full-context follows: each tracks its own teacher target and
                // the two predictions differ.
                if full_pred[sa] == Some(ta)
                    && full_pred[sb] == Some(tb)
                    && full_pred[sa] != full_pred[sb]
                {
                    mp_follow += 1;
                }
                made += 1;
                if made >= PAIRS_PER_KEY {
                    break 'outer;
                }
            }
        }
    }
    let (mp_rate, mp_lo, _mp_hi) = ci95_permille(mp_follow, mp_total.max(1));

    // --- verdict (the runnable window arms) --------------------------------
    let non_vacuous = full_r - swap_r >= SWAP_MARGIN_PERMILLE && full_r > trivial_r;
    let verdict = if !non_vacuous {
        "VACUOUS — full-context does not clear the prompt-swap/trivial nulls; reading licenses nothing"
    } else if delta_lo >= CAUSAL_FLOOR_PERMILLE || mp_lo > 0.0 {
        "SELECT: longer-context arm — prompt context causally beats the suffix-only floor (teacher-grounded)"
    } else if delta_lo <= 0.0 && mp_follow == 0 {
        "NO PROMPT-CONDITIONING ARM ESTABLISHED — the deployed model is suffix-local; build the #835 mechanisms (#836) and re-test"
    } else {
        "REVISE — a weak/uncertain positive prompt effect below the causal floor"
    };

    let elapsed = started.elapsed();

    // --- report ------------------------------------------------------------
    let artifact_cid = compute_cid(&graph_bytes);
    let corpus_cid = compute_cid(&meta_bytes);
    println!("=== #834 teacher-grounded causal-influence run ===");
    println!("bundle           : {}", bundle.root.display());
    println!("artifact_cid     : {artifact_cid}");
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("held_out         : {} positions", held_out.len());
    println!("sample_n         : {n} (stride {stride}), served {served}, ngram_hits {ngram_hits}");
    println!("suffix_k         : {SUFFIX_K}");
    println!(
        "windows          : mean {win_mean:.1} tokens, max {win_max}, > suffix_k: {win_over_k}/{n}"
    );
    println!(
        "full-context     : {:.1}permille  (95% CI [{:.1}, {:.1}])",
        full_r, full_lo, _full_hi
    );
    println!("suffix-only      : {suffix_r:.1}permille");
    let sweep_str: String = sweep
        .iter()
        .map(|(k, h)| format!("k{}={:.0} ", k, *h as f64 / n as f64 * 1000.0))
        .collect();
    println!("context sweep    : {sweep_str}full(8)={full_r:.0}  permille (flat = suffix-local)");
    println!("prompt-swap null : {swap_r:.1}permille  (95% hi {swap_hi:.1})");
    println!("trivial-prior    : {trivial_r:.1}permille (marginal token {marginal})");
    println!(
        "CAUSAL-DELTA     : {:.1}permille  (paired 95% CI [{:.1}, {:.1}])  full-minus-suffix",
        delta_r, delta_lo, delta_hi
    );
    println!(
        "attribution      : context-helped {ctx_helped}, suffix-sufficed {suffix_sufficed}, miss {miss}"
    );
    println!(
        "minimal-pairs    : {mp_total} pairs, follow {mp_follow} ({mp_rate:.1}permille, 95% lo {mp_lo:.1}); suffix-identical {mp_suffix_identical}/{mp_total}"
    );
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    // --- CID-bound canonical record ---------------------------------------
    let mut rec = Vec::new();
    for v in [
        n as u64,
        full_hits,
        suffix_hits,
        swap_hits,
        trivial_hits,
        served,
        ngram_hits,
        mp_total,
        mp_follow,
        mp_suffix_identical,
        marginal as u64,
        SUFFIX_K as u64,
        win_over_k,
        win_max as u64,
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    for (_, h) in &sweep {
        rec.extend_from_slice(&h.to_le_bytes());
    }
    rec.extend_from_slice(artifact_cid.as_bytes());
    rec.extend_from_slice(corpus_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid       : {result_cid}");

    let mut sweep_json = String::from("{");
    for (idx, (k, h)) in sweep.iter().enumerate() {
        if idx > 0 {
            sweep_json.push(',');
        }
        sweep_json.push_str(&format!("\"{}\":{:.1}", k, *h as f64 / n as f64 * 1000.0));
    }
    sweep_json.push_str(&format!(",\"8\":{full_r:.1}}}"));

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 834,\n",
            "  \"bundle\": \"{}\",\n",
            "  \"artifact_cid\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"held_out\": {},\n",
            "  \"sample_n\": {},\n",
            "  \"suffix_k\": {},\n",
            "  \"context_sweep_permille\": {},\n",
            "  \"served\": {},\n",
            "  \"ngram_hits\": {},\n",
            "  \"full_context_permille\": {:.1},\n",
            "  \"full_context_ci\": [{:.1}, {:.1}],\n",
            "  \"suffix_only_permille\": {:.1},\n",
            "  \"prompt_swap_permille\": {:.1},\n",
            "  \"trivial_prior_permille\": {:.1},\n",
            "  \"causal_delta_permille\": {:.1},\n",
            "  \"causal_delta_ci\": [{:.1}, {:.1}],\n",
            "  \"attribution\": {{\"context_helped\": {}, \"suffix_sufficed\": {}, \"miss\": {}}},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"follow\": {}, \"follow_permille\": {:.1}, \"follow_ci_lo\": {:.1}, \"suffix_identical\": {}}},\n",
            "  \"arms_unavailable\": [\"persistent-state\", \"conditional-residuals\", \"candidate-support-expansion\"],\n",
            "  \"arms_unavailable_reason\": \"#835 mechanisms not lowered; engine exposes selected token only (built by #836)\",\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        artifact_cid,
        corpus_cid,
        held_out.len(),
        n,
        SUFFIX_K,
        sweep_json,
        served,
        ngram_hits,
        full_r,
        full_lo,
        _full_hi,
        suffix_r,
        swap_r,
        trivial_r,
        delta_r,
        delta_lo,
        delta_hi,
        ctx_helped,
        suffix_sufficed,
        miss,
        mp_total,
        mp_follow,
        mp_rate,
        mp_lo,
        mp_suffix_identical,
        result_cid,
        verdict,
    );
    let out = repo_root().join("docs").join("causal_run_834_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());

    // Structural guards (the measurement is non-vacuous machinery, not a
    // vacuous green): the sample served real predictions, and the suffix-only
    // control is degenerate-by-construction on the minimal pairs.
    assert!(served > 0, "engine served no predictions");
    if mp_total > 0 {
        assert_eq!(
            mp_suffix_identical, mp_total,
            "suffix-only must be identical on identical-suffix minimal pairs (control degeneracy)"
        );
    }
}
