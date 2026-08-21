//! `region-conditional` D2 reference instrument — S1 representation redesign
//! (#822).
//!
//! Mandated by the approved redesign RFC (`docs/s1_redesign_rfc_822.md`
//! §4-D2, §6 step 3); the run contract was posted to #822 before this run.
//! The holographic bet stated plainly: the compiled geometry already
//! summarizes the whole window (0.0% full-depth context-code collisions,
//! #784), but no evidence table conditions on it — the graph path consults
//! cover/score structures whose measured continuation distributions converge
//! (#784: 11/15 distinct full-depth codes argmax to the corpus-dominant
//! mode). This instrument tabulates region-conditional continuation evidence
//! keyed by the artifact's own graded code and asks whether granularity the
//! coarse convergence result hides carries teacher-predictive signal.
//!
//! Region assignment is read DIRECTLY from the released artifact per the Q2
//! approval: codes are derived for every corpus record by the deployed
//! quantization path — `runtime::bundle_plain` →
//! `runtime::assign_code_for_bundle`, the exact #784 instrument path — and
//! cached via the κ-gated code sidecar. No recompile.
//!
//! ## Arms (pre-registered on #822; λ fixed at 1.0 before evaluation)
//!
//!   * `base`      — suffix rate over suffix candidates (the suffix-local
//!     floor; #891 construction verbatim). Reproduction gates: 246.6‰
//!     (±0.05), minimal pairs 4,722 exact, base-follow 0.
//!   * `region[d]` for d ∈ {1,2,3,4} — score(c) = suffix_rate(c) +
//!     λ·(region_rate_d(c) − suffix_rate(c)), region key = the record's
//!     graded code truncated to depth d (cap-64 tables). PRIMARY =
//!     `region[4]` (full depth); the depth sweep is the pre-registered
//!     granularity sweep.
//!   * `rxs`       — region × suffix product key at full depth
//!     (code4, 2-token suffix): the geometric analog of D1's joint key.
//!   * `codeshuf`  — planted null: this position's suffix baseline with
//!     region evidence looked up under the swap partner's full-depth code
//!     (different-story rotation). Must change ≥1 prediction and sit below
//!     a positive arm.
//!
//! The pre-registered #784 convergence null is reported as a diagnostic:
//! the held-out-usage-weighted share of `region[4]` tables whose argmax
//! equals the corpus-marginal token, plus distinct-code and coverage stats
//! per depth.
//!
//! ## Pre-registered decision rule (posted to #822 before this run)
//!
//! Consultation order `region[4]` → `rxs` (the depth sweep is exploratory):
//!   * SELECT  iff the consulted arm's paired 95% lower bound ≥ 25.0‰
//!     (Q3 opening bar; the frozen 20‰ floor stays the promotion gate).
//!   * NO ARM  iff `region[4]` and `rxs` both have lower bound ≤ 0 with 0
//!     minimal-pair follows.
//!   * REVISE  otherwise; a lower bound in [20, 25) is recorded explicitly
//!     as floor-clearing-but-below-opening-bar.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test region_conditional_run_822 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus and artifact are read from it).

#![allow(clippy::doc_lazy_continuation)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::code_sidecar;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_compiler::induction;

const SUFFIX_K: usize = 2;
/// Per-key cap on retained argmax counts (bounded lane tables, #835).
const CAP: usize = 64;
/// Candidate-set widths: suffix baseline plus the region/rxs widenings.
const CAND_SUFFIX: usize = 32;
const CAND_REGION: usize = 32;
const CAND_RXS: usize = 32;
/// Pre-registered residual weight λ, fixed before evaluation.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// The frozen causal floor (‰) — the promotion gate (#887: the bar stands).
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// The Q3-adopted OPENING bar (‰) for a lowering track (RFC §1).
const OPENING_BAR_PERMILLE: f64 = 25.0;
/// Positions re-scored by the in-harness double-run determinism check.
const DOUBLE_RUN_N: usize = 2_000;
/// Reproduction gates shared with the #875/#891/D1 harnesses.
const REPRO_BASE_PERMILLE: f64 = 246.6;
const REPRO_MP_TOTAL: u64 = 4_722;

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
    /// Argmax by count, canonical tie-break (count desc, id asc).
    fn argmax_tok(&self) -> Option<u32> {
        self.map
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(&t, _)| t)
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

/// Pack the first `d` stages of a graded code into a u32 key (little-endian
/// byte order; unused high bytes zero). Depth is 1..=STAGES.
fn pack_prefix(code: &[u8; STAGES], d: usize) -> u32 {
    let mut key = 0u32;
    for (i, &b) in code.iter().take(d).enumerate() {
        key |= (b as u32) << (8 * i);
    }
    key
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
    /// One region table per depth d = index + 1.
    region_next: [HashMap<u32, Counter>; STAGES],
    rxs_next: HashMap<(u32, (u32, u32)), Counter>,
    marginal_tok: u32,
}

impl Tables {
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

    fn suffix_rate(&self, w: &[u32], c: u32) -> f64 {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total
    }

    /// Top candidates of one bounded table by rate (count desc, id asc).
    fn table_top(counter: Option<&Counter>, take: usize) -> Vec<u32> {
        let Some(c) = counter else {
            return Vec::new();
        };
        let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.into_iter().take(take).map(|(k, _)| k).collect()
    }

    /// Single-key residual scoring shared by every region arm and the null:
    /// score(c) = base(c) + λ·(rate(c) − base(c)) when the key is supported,
    /// base fallback otherwise. `w` supplies the suffix baseline.
    fn keyed_scores(
        &self,
        w: &[u32],
        counter: Option<&Counter>,
        cands: &[u32],
        lam: f64,
    ) -> HashMap<u32, f64> {
        let mut m = HashMap::new();
        let tot = counter.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        for &c in cands {
            let base = self.suffix_rate(w, c);
            let s = match counter {
                Some(cn) => {
                    let rate = cn.map.get(&c).copied().unwrap_or(0) as f64 / tot;
                    base + lam * (rate - base)
                }
                None => base,
            };
            m.insert(c, s);
        }
        m
    }
}

/// One held-out position's predictions across every arm and control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Preds {
    base: u32,
    region: [u32; STAGES],
    rxs: u32,
    codeshuf: u32,
}

/// Score one position. `code` is this record's graded code; `partner_code`
/// drives the codeshuf null.
fn score_position(
    t: &Tables,
    w: &[u32],
    code: &[u8; STAGES],
    partner_code: &[u8; STAGES],
    lam: f64,
) -> Preds {
    let sfx = suffix_key(w);
    let suffix_cands = t.suffix_cands(w);

    let full_key = pack_prefix(code, STAGES);
    let region4 = t.region_next[STAGES - 1].get(&full_key);
    let rxs = t.rxs_next.get(&(full_key, sfx));

    // Shared candidate set for every D2 arm: suffix ∪ region4-top ∪ rxs-top.
    let mut cands = suffix_cands.clone();
    cands.extend(Tables::table_top(region4, CAND_REGION));
    cands.extend(Tables::table_top(rxs, CAND_RXS));
    cands.sort_unstable();
    cands.dedup();

    let mut base_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &suffix_cands {
        base_scores.insert(c, t.suffix_rate(w, c));
    }

    let mut region_preds = [0u32; STAGES];
    for d in 1..=STAGES {
        let key = pack_prefix(code, d);
        let counter = t.region_next[d - 1].get(&key);
        let scores = t.keyed_scores(w, counter, &cands, lam);
        region_preds[d - 1] = argmax(&scores);
    }

    let rxs_scores = t.keyed_scores(w, rxs, &cands, lam);

    // Planted null: the swap partner's full-depth code under this window's
    // suffix baseline, with the foreign table's own candidate widening.
    let foreign_key = pack_prefix(partner_code, STAGES);
    let foreign = t.region_next[STAGES - 1].get(&foreign_key);
    let mut shuf_cands = suffix_cands.clone();
    shuf_cands.extend(Tables::table_top(foreign, CAND_REGION));
    shuf_cands.sort_unstable();
    shuf_cands.dedup();
    let codeshuf_scores = t.keyed_scores(w, foreign, &shuf_cands, lam);

    Preds {
        base: argmax(&base_scores),
        region: region_preds,
        rxs: argmax(&rxs_scores),
        codeshuf: argmax(&codeshuf_scores),
    }
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus + artifact; run with --ignored"]
fn region_conditional_run_822() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP region_conditional_run_822: no serving bundle at {}",
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
    let artifacts = compiler::load_artifacts_from(bundle.teacher.to_str().expect("teacher utf8"))
        .expect("load artifacts");
    let (train, held_out) = induction::split_positions(&corpus);
    assert!(!train.is_empty() && !held_out.is_empty(), "non-empty split");

    let started = Instant::now();

    // --- derive every record's graded code via the deployed quantization
    // path (Q2 approval: direct read-only artifact reads; #784's exact
    // instrument path), cached through the κ-gated sidecar ----------------
    let rotations = runtime::derive_rotations();
    let codes: Vec<[u8; STAGES]> = code_sidecar::corpus_codes_cached(&artifacts, &corpus, || {
        let mut out: Vec<[u8; STAGES]> = Vec::with_capacity(corpus.n);
        for i in 0..corpus.n {
            let bundle_vec = runtime::bundle_plain(&artifacts, &rotations, &corpus, i);
            out.push(runtime::assign_code_for_bundle(&artifacts, &bundle_vec));
            if i % 50_000 == 0 {
                eprintln!("codes {i}/{}", corpus.n);
            }
        }
        out
    });
    assert_eq!(codes.len(), corpus.n, "one code per record");
    let code_pass_s = started.elapsed().as_secs_f64();

    // --- build tables from TRAIN (document-disjoint by story) --------------
    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut region_next: [HashMap<u32, Counter>; STAGES] = Default::default();
    let mut rxs_next: HashMap<(u32, (u32, u32)), Counter> = HashMap::new();
    let mut marginal = Counter::default();
    for &i in &train {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        let sfx = suffix_key(&w);
        marginal.bump(target);
        suffix_next.entry(sfx).or_default().bump(target);
        for d in 1..=STAGES {
            region_next[d - 1]
                .entry(pack_prefix(&codes[i], d))
                .or_default()
                .bump(target);
        }
        rxs_next
            .entry((pack_prefix(&codes[i], STAGES), sfx))
            .or_default()
            .bump(target);
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for table in region_next.iter_mut() {
        for c in table.values_mut() {
            c.cap_to_top(CAP);
        }
    }
    for c in rxs_next.values_mut() {
        c.cap_to_top(CAP);
    }
    let marginal_total = marginal.total();
    let marginal_tok = marginal.argmax_tok().unwrap_or(0);
    let region_entries: Vec<u64> = region_next.iter().map(|t| t.len() as u64).collect();
    let rxs_entries = rxs_next.len() as u64;
    for (d, entries) in region_entries.iter().enumerate() {
        assert!(
            *entries > 0,
            "region depth {} table must be non-empty",
            d + 1
        );
    }
    assert!(rxs_entries > 0, "rxs table must be non-empty");

    let tables = Tables {
        suffix_next,
        region_next,
        rxs_next,
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
    let mut depth_covered = [0u64; STAGES];
    let mut rxs_covered = 0u64;
    let mut distinct_code4: HashMap<u32, u32> = HashMap::new();
    let mut converged_positions = 0u64;
    for (idx, w) in windows.iter().enumerate() {
        let i = held_out[idx];
        let partner_code = &codes[held_out[swap_partner[idx]]];
        preds.push(score_position(&tables, w, &codes[i], partner_code, lambda));
        let full_key = pack_prefix(&codes[i], STAGES);
        *distinct_code4.entry(full_key).or_insert(0) += 1;
        for d in 1..=STAGES {
            if tables.region_next[d - 1].contains_key(&pack_prefix(&codes[i], d)) {
                depth_covered[d - 1] += 1;
            }
        }
        if tables.rxs_next.contains_key(&(full_key, suffix_key(w))) {
            rxs_covered += 1;
        }
        if let Some(cn) = tables.region_next[STAGES - 1].get(&full_key) {
            if cn.argmax_tok() == Some(tables.marginal_tok) {
                converged_positions += 1;
            }
        }
    }
    assert!(
        distinct_code4.len() > 1,
        "held-out full-depth codes must not be a single region"
    );

    let mut base_hits = 0u64;
    let mut region_hits = [0u64; STAGES];
    let mut rxs_hits = 0u64;
    let mut codeshuf_hits = 0u64;
    let mut trivial_hits = 0u64;
    let mut region_vs_base: Vec<Vec<i8>> = (0..STAGES)
        .map(|_| Vec::with_capacity(held_out.len()))
        .collect();
    let mut rxs_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut codeshuf_vs_base: Vec<i8> = Vec::with_capacity(held_out.len());
    let mut codeshuf_changed = 0u64;
    for (idx, &i) in held_out.iter().enumerate() {
        let target = corpus.t_argmax[i];
        let p = preds[idx];
        let bh = p.base == target;
        base_hits += u64::from(bh);
        for d in 0..STAGES {
            let rh = p.region[d] == target;
            region_hits[d] += u64::from(rh);
            region_vs_base[d].push(i8::from(rh) - i8::from(bh));
        }
        let xh = p.rxs == target;
        rxs_hits += u64::from(xh);
        rxs_vs_base.push(i8::from(xh) - i8::from(bh));
        let sh = p.codeshuf == target;
        codeshuf_hits += u64::from(sh);
        codeshuf_vs_base.push(i8::from(sh) - i8::from(bh));
        trivial_hits += u64::from(tables.marginal_tok == target);
        codeshuf_changed += u64::from(p.codeshuf != p.region[STAGES - 1]);
    }

    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let mut region_r = [0f64; STAGES];
    let mut region_delta = [(0f64, 0f64, 0f64); STAGES];
    for d in 0..STAGES {
        let (r, _, _) = ci95_permille(region_hits[d], n);
        region_r[d] = r;
        region_delta[d] = paired_delta_permille(&region_vs_base[d]);
    }
    let (rxs_r, _, _) = ci95_permille(rxs_hits, n);
    let (rxs_delta, rxs_delta_lo, rxs_delta_hi) = paired_delta_permille(&rxs_vs_base);
    let (codeshuf_r, _, _) = ci95_permille(codeshuf_hits, n);
    let (codeshuf_delta, codeshuf_delta_lo, codeshuf_delta_hi) =
        paired_delta_permille(&codeshuf_vs_base);
    let (trivial_r, _, _) = ci95_permille(trivial_hits, n);
    let (r4_delta, r4_delta_lo, _r4_delta_hi) = region_delta[STAGES - 1];

    // --- minimal pairs: same suffix, different story, different teacher -----
    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, w) in windows.iter().enumerate() {
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
    }
    let mut mp_total = 0u64;
    let mut base_follow = 0u64;
    let mut region4_follow = 0u64;
    let mut rxs_follow = 0u64;
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
                let (ra, rb) = (pa.region[STAGES - 1], pb.region[STAGES - 1]);
                if ra == ta && rb == tb && ra != rb {
                    region4_follow += 1;
                }
                if pa.rxs == ta && pb.rxs == tb && pa.rxs != pb.rxs {
                    rxs_follow += 1;
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let (r4_mp_rate, r4_mp_lo, _) = ci95_permille(region4_follow, mp_total.max(1));

    // --- exploratory λ-sweep for region[4] (NOT the verdict) ---------------
    let sweep_lams = [0.5f64, 1.0, 2.0, 4.0, 8.0];
    let mut sweep: Vec<(f64, u64)> = Vec::new();
    for &lam in &sweep_lams {
        let mut h = 0u64;
        for (idx, &i) in held_out.iter().enumerate() {
            let partner_code = &codes[held_out[swap_partner[idx]]];
            let p = score_position(&tables, &windows[idx], &codes[i], partner_code, lam);
            if p.region[STAGES - 1] == corpus.t_argmax[i] {
                h += 1;
            }
        }
        sweep.push((lam, h));
    }

    // --- double-run determinism check --------------------------------------
    let check_n = DOUBLE_RUN_N.min(held_out.len());
    for idx in 0..check_n {
        let i = held_out[idx];
        let partner_code = &codes[held_out[swap_partner[idx]]];
        let again = score_position(&tables, &windows[idx], &codes[i], partner_code, lambda);
        assert_eq!(again, preds[idx], "double-run drift at held-out idx {idx}");
    }

    // --- reproduction gates -------------------------------------------------
    assert!(
        (base_r - REPRO_BASE_PERMILLE).abs() < 0.05,
        "suffix baseline {base_r:.1}permille does not reproduce the recorded {REPRO_BASE_PERMILLE}permille"
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
        codeshuf_changed > 0,
        "code-shuffle null must change at least one prediction"
    );
    if r4_delta_lo > 0.0 {
        assert!(
            codeshuf_delta < r4_delta,
            "code-shuffle null ({codeshuf_delta:.1}permille) must sit below a positive arm ({r4_delta:.1}permille)"
        );
    }
    assert!(
        region_hits[STAGES - 1] > trivial_hits,
        "the arm must beat the trivial no-context floor"
    );

    // --- verdict (pre-registered rule; see module docs) ---------------------
    let verdict = if r4_delta_lo >= OPENING_BAR_PERMILLE {
        "SELECT (region4) — the paired lower bound clears the 25permille opening bar; the geometric key joins the lowering-track design space"
    } else if rxs_delta_lo >= OPENING_BAR_PERMILLE {
        "SELECT (region x suffix) — the product-key arm clears the 25permille opening bar; the geometric key joins the lowering-track design space"
    } else if r4_delta_lo <= 0.0 && region4_follow == 0 && rxs_delta_lo <= 0.0 && rxs_follow == 0 {
        "NO ARM — region-conditional evidence adds no teacher-predictive signal over the suffix floor; the #784 convergence result stands at corpus scale"
    } else if r4_delta_lo >= CAUSAL_FLOOR_PERMILLE {
        "REVISE (floor-clearing, below opening bar) — region4 clears the frozen 20permille floor but not the Q3 25permille opening bar; no lowering track opens"
    } else {
        "REVISE — real but sub-floor region-conditional signal; below the frozen 20permille floor, so no lowering track opens"
    };

    let elapsed = started.elapsed();
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #822 D2 region-conditional reference instrument ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("train / held_out : {} / {}", train.len(), held_out.len());
    println!("lambda (fixed)   : {lambda}");
    println!(
        "code pass        : {:.1}s (sidecar-cached thereafter)",
        code_pass_s
    );
    println!(
        "region tables    : d1 {} / d2 {} / d3 {} / d4 {} keys; rxs {rxs_entries} keys",
        region_entries[0], region_entries[1], region_entries[2], region_entries[3]
    );
    println!(
        "held-out coverage: d1 {} / d2 {} / d3 {} / d4 {} of {n}; rxs {rxs_covered}; distinct code4 {}",
        depth_covered[0],
        depth_covered[1],
        depth_covered[2],
        depth_covered[3],
        distinct_code4.len()
    );
    println!(
        "convergence diag : {converged_positions}/{} covered positions whose region4 argmax IS the corpus mode (#784 null)",
        depth_covered[STAGES - 1]
    );
    println!("suffix baseline  : {base_r:.1}permille  (95% CI [{base_lo:.1}, {base_hi:.1}])");
    for d in 0..STAGES {
        let (delta, lo, hi) = region_delta[d];
        let marker = if d == STAGES - 1 { " (PRIMARY)" } else { "" };
        println!(
            "region[{}]{marker:9}: {:.1}permille  (delta {delta:.1} [{lo:.1}, {hi:.1}])",
            d + 1,
            region_r[d]
        );
    }
    println!(
        "rxs (product)    : {rxs_r:.1}permille  (delta {rxs_delta:.1} [{rxs_delta_lo:.1}, {rxs_delta_hi:.1}]; covered {rxs_covered})"
    );
    println!(
        "codeshuf null    : {codeshuf_r:.1}permille  (delta {codeshuf_delta:.1} [{codeshuf_delta_lo:.1}, {codeshuf_delta_hi:.1}]; changed {codeshuf_changed})"
    );
    println!("trivial prior    : {trivial_r:.1}permille");
    let sweep_str: String = sweep
        .iter()
        .map(|(l, h)| format!("λ{}={:.0} ", l, *h as f64 / n as f64 * 1000.0))
        .collect();
    println!("λ-sweep (explor.): {sweep_str}(baseline={base_r:.0})permille");
    println!(
        "minimal-pairs    : {mp_total} pairs; region4-follow {region4_follow} ({r4_mp_rate:.1}permille, 95% lo {r4_mp_lo:.1}); rxs-follow {rxs_follow}; baseline-follow {base_follow}"
    );
    println!("double-run       : {check_n} positions identical");
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    let mut rec = Vec::new();
    for v in [
        n,
        base_hits,
        region_hits[0],
        region_hits[1],
        region_hits[2],
        region_hits[3],
        rxs_hits,
        codeshuf_hits,
        trivial_hits,
        mp_total,
        region4_follow,
        rxs_follow,
        base_follow,
        codeshuf_changed,
        region_entries[0],
        region_entries[1],
        region_entries[2],
        region_entries[3],
        rxs_entries,
        depth_covered[0],
        depth_covered[1],
        depth_covered[2],
        depth_covered[3],
        rxs_covered,
        distinct_code4.len() as u64,
        converged_positions,
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
    let region_json: String = {
        let mut s = String::from("[");
        for d in 0..STAGES {
            if d > 0 {
                s.push(',');
            }
            let (delta, lo, hi) = region_delta[d];
            s.push_str(&format!(
                "{{\"depth\": {}, \"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"table_entries\": {}, \"held_out_covered\": {}}}",
                d + 1,
                region_r[d],
                delta,
                lo,
                hi,
                region_entries[d],
                depth_covered[d]
            ));
        }
        s.push(']');
        s
    };
    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 822,\n",
            "  \"arm\": \"region-conditional-d2-reference\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"lambda\": {},\n",
            "  \"suffix_baseline_permille\": {:.1},\n",
            "  \"regions\": {},\n",
            "  \"rxs\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"table_entries\": {}, \"held_out_covered\": {}}},\n",
            "  \"codeshuf_null\": {{\"permille\": {:.1}, \"delta_permille\": {:.1}, \"delta_ci\": [{:.1}, {:.1}], \"changed\": {}}},\n",
            "  \"trivial_prior_permille\": {:.1},\n",
            "  \"convergence_diagnostic\": {{\"region4_covered\": {}, \"argmax_is_corpus_mode\": {}, \"distinct_code4_held_out\": {}}},\n",
            "  \"lambda_sweep_permille\": {},\n",
            "  \"minimal_pairs\": {{\"total\": {}, \"region4_follow\": {}, \"region4_follow_permille\": {:.1}, \"region4_follow_ci_lo\": {:.1}, \"rxs_follow\": {}, \"baseline_follow\": {}}},\n",
            "  \"double_run\": {{\"checked\": {}, \"identical\": true}},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        train.len(),
        held_out.len(),
        lambda,
        base_r,
        region_json,
        rxs_r,
        rxs_delta,
        rxs_delta_lo,
        rxs_delta_hi,
        rxs_entries,
        rxs_covered,
        codeshuf_r,
        codeshuf_delta,
        codeshuf_delta_lo,
        codeshuf_delta_hi,
        codeshuf_changed,
        trivial_r,
        depth_covered[STAGES - 1],
        converged_positions,
        distinct_code4.len(),
        sweep_json,
        mp_total,
        region4_follow,
        r4_mp_rate,
        r4_mp_lo,
        rxs_follow,
        base_follow,
        check_n,
        result_cid,
        verdict,
    );
    let out = repo_root()
        .join("docs")
        .join("region_conditional_822_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
