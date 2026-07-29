//! The CERTIFIER: measures the equivalence-class membership of the compiled
//! artifact, the price of every constraint, and the compression claims.
//! Instrumentation — floating point and division are permitted here and
//! appear nowhere in the runtime.
//!
//! Certificate contents:
//!   - teacher bounds (floor, ceiling) on the held-out stream;
//!   - equality witnesses between the runtime's kernel path and plain path:
//!     bundles, class codes, AND predictions, per sampled position;
//!   - the op census (kernel path, every operation counted);
//!   - the multiplication-free runtime's metrics (store built by the
//!     runtime's own path — key identity by construction);
//!   - the binarization ablation (f32 nearest-centroid assignment);
//!   - the bit-prefix coordinate (no codebook classes at all);
//!   - COMPRESSION (PROOF.md P5): container round-trip witness, the
//!     rate–distortion table of the shipped token representation, and the
//!     end-to-end artifact accounting against the source bytes.

use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, D, K, STAGES, V, WINDOW};
use uor_r4_core::transformerless::runtime::{
    self, build_store, bundle_kernel, bundle_plain, code_plain, Runtime, Store,
};
use uor_r4_model_source::TeacherOracle;

// Retained for the Phase-4 EXCT prefix-store path; not yet called.
#[allow(dead_code)]
fn build_prefix_store(
    art: &compiler::Compiled,
    rot: &[usize; WINDOW + 1],
    c: &compiler::Corpus,
    depths: usize,
) -> Store {
    let cut = (c.stories as f64 * 0.8) as u32;
    let mut levels: Store = (0..=depths).map(|_| BTreeMap::new()).collect();
    for i in 0..c.n {
        if c.story[i] >= cut {
            continue;
        }
        let code = code_plain(art, rot, c, i);
        for k_idx in 0..c.top_tokens[i].len() {
            let tok = c.top_tokens[i][k_idx];
            let weight = c.top_weights[i][k_idx];
            if weight > 0 {
                *levels[0].entry(vec![]).or_default().entry(tok).or_default() += weight;
                for d in 1..=depths {
                    *levels[d]
                        .entry(code[..d].to_vec())
                        .or_default()
                        .entry(tok)
                        .or_default() += weight;
                }
            }
        }
    }
    levels
}

fn deepest_argmax(store: &Store, key: &dyn Fn(usize) -> Vec<u8>, depths: usize) -> u32 {
    for d in (0..=depths).rev() {
        if let Some(dist) = store[d].get(&key(d)) {
            // canonical argmax: highest count, ties to smallest token id.
            let mut best_t = 0u32;
            let mut best_c = -1i64;
            for (&t, &cnt) in dist {
                if (cnt as i64) > best_c {
                    best_c = cnt as i64;
                    best_t = t;
                }
            }
            return best_t;
        }
    }
    unreachable!()
}

struct Metrics {
    top1: f64,
    agree: f64,
    wb_bits: f64,
    keys: usize,
}

fn eval(
    c: &compiler::Corpus,
    store: &Store,
    depths: usize,
    key: &dyn Fn(usize, usize) -> Vec<u8>,
) -> Metrics {
    let cut = (c.stories as f64 * 0.8) as u32;
    let test: Vec<usize> = (0..c.n).filter(|&i| c.story[i] >= cut).collect();
    let (mut top1, mut agree, mut bits) = (0u64, 0u64, 0f64);
    for &i in &test {
        let pred = deepest_argmax(store, &|d| key(i, d), depths);
        if pred == c.next[i] {
            top1 += 1;
        }
        if pred == c.t_argmax[i] {
            agree += 1;
        }
        let mut lams: Vec<(f64, &BTreeMap<u32, u32>, u32)> = Vec::new();
        for (d, level) in store.iter().enumerate().take(depths + 1) {
            if let Some(dist) = level.get(&key(i, d)) {
                let total: u32 = dist.values().sum();
                let lam = total as f64 / (total as f64 + dist.len() as f64);
                lams.push((lam, dist, total));
            }
        }
        let mut p = {
            let mut rem = 1.0f64;
            let mut acc = 0.0f64;
            for li in (0..lams.len()).rev() {
                let w = rem * lams[li].0;
                rem *= 1.0 - lams[li].0;
                if let Some(&cc) = lams[li].1.get(&c.next[i]) {
                    acc += w * cc as f64 / lams[li].2 as f64;
                }
            }
            acc + rem / 32000.0
        };
        if p <= 0.0 {
            p = 1e-30;
        }
        bits += -p.log2();
    }
    let n = test.len() as f64;
    Metrics {
        top1: 100.0 * top1 as f64 / n,
        agree: 100.0 * agree as f64 / n,
        wb_bits: bits / n,
        keys: store.iter().map(|l| l.len()).sum(),
    }
}

/// Merge the distributions of several candidate keys at one store level
/// (issue #244: query-time beam expansion — the read-side equivalent of
/// what `add_evidence_multi` materializes at write time).
pub fn merged_beam_distribution(
    level: &BTreeMap<Vec<u8>, BTreeMap<u32, u32>>,
    keys: &[Vec<u8>],
) -> BTreeMap<u32, u32> {
    let mut merged: BTreeMap<u32, u32> = BTreeMap::new();
    for key in keys {
        if let Some(dist) = level.get(key) {
            for (&tok, &count) in dist {
                *merged.entry(tok).or_default() += count;
            }
        }
    }
    merged
}

/// Evaluate a SINGLE-KEY store with query-time beam expansion (issue #244):
/// per position, the same multi-membership beam the shipped store-time path
/// materializes as extra keys is instead applied at read time — candidate
/// keys per depth from `assign_memberships_plain`, distributions merged
/// before argmax/backoff scoring. Store size stays at single-assignment.
fn eval_query_beam(
    c: &compiler::Corpus,
    store: &Store,
    art: &compiler::Compiled,
    rot: &[usize; WINDOW + 1],
) -> Metrics {
    let cut = (c.stories as f64 * 0.8) as u32;
    let test: Vec<usize> = (0..c.n).filter(|&i| c.story[i] >= cut).collect();
    let (mut top1, mut agree, mut bits) = (0u64, 0u64, 0f64);
    for &i in &test {
        let bundle = bundle_plain(art, rot, c, i);
        let sig = runtime::sig_plain(art, &bundle);
        let (_code, by_depth) = runtime::assign_memberships_plain(art, &sig);

        let mut lams: Vec<(f64, BTreeMap<u32, u32>, u32)> = Vec::new();
        for (d, level) in store.iter().enumerate().take(STAGES + 1) {
            let keys: &[Vec<u8>] = by_depth.get(d).map(|k| k.as_slice()).unwrap_or(&[]);
            let merged = merged_beam_distribution(level, keys);
            if !merged.is_empty() {
                let total: u32 = merged.values().sum();
                let lam = total as f64 / (total as f64 + merged.len() as f64);
                lams.push((lam, merged, total));
            }
        }

        // canonical argmax on the deepest populated merged distribution
        let pred = lams
            .last()
            .map(|(_, dist, _)| {
                let mut best_t = 0u32;
                let mut best_c = -1i64;
                for (&t, &cnt) in dist {
                    if (cnt as i64) > best_c {
                        best_c = cnt as i64;
                        best_t = t;
                    }
                }
                best_t
            })
            .unwrap_or(0);
        if pred == c.next[i] {
            top1 += 1;
        }
        if pred == c.t_argmax[i] {
            agree += 1;
        }

        let mut p = {
            let mut rem = 1.0f64;
            let mut acc = 0.0f64;
            for li in (0..lams.len()).rev() {
                let w = rem * lams[li].0;
                rem *= 1.0 - lams[li].0;
                if let Some(&cc) = lams[li].1.get(&c.next[i]) {
                    acc += w * cc as f64 / lams[li].2 as f64;
                }
            }
            acc + rem / 32000.0
        };
        if p <= 0.0 {
            p = 1e-30;
        }
        bits += -p.log2();
    }
    let n = test.len() as f64;
    Metrics {
        top1: 100.0 * top1 as f64 / n,
        agree: 100.0 * agree as f64 / n,
        wb_bits: bits / n,
        keys: store.iter().map(|l| l.len()).sum(),
    }
}

/// Probability the corpus recorded for the observed next token, from the
/// integer-percent quantized top-3 weights (issue #231).
///
/// Weights are stored as whole percents (`softmax_top3_sample`,
/// compiler-side), so a top-3 token whose renormalized probability fell
/// below 0.5% records weight 0 — while the corpus `next` is sampled from
/// the full CDF and can land on exactly that token. The raw `0/100`
/// probability would make the teacher floor `-ln(0)` = +inf, so a
/// zero-weight collision clamps to half a quantization step (0.005) —
/// the same clamp discipline the HF evaluate path already applies.
/// Returns `(prob, zero_weight_collision)`.
pub fn recorded_next_prob(top_tokens: &[u32; 8], top_weights: &[u32; 8], next: u32) -> (f64, bool) {
    for k in 0..3 {
        if top_tokens[k] == next {
            let w = top_weights[k];
            return if w == 0 {
                (0.005, true)
            } else {
                (w as f64 / 100.0, false)
            };
        }
    }
    (0.01, false)
}

/// Evaluation context for position `i` (issue #237): the last `WINDOW`
/// tokens in chronological order — oldest first, ending with `input[i]` as
/// the most recent — bounded to the position's own story, matching the
/// compiler's observation semantics (`runtime::history_token`).
///
/// Replaces the defective construction that indexed the corpus with the
/// per-slot *vector rotation offsets* (j·17 mod D) as if they were time
/// lags — sampling stride-17 positions, in reversed order, across story
/// boundaries, with a phantom trailing zero token.
pub fn eval_context(c: &compiler::Corpus, i: usize) -> Vec<u32> {
    let mut hist = Vec::with_capacity(WINDOW);
    for j in (1..=WINDOW).rev() {
        if let Some(t) = runtime::history_token(c, i, j) {
            hist.push(t);
        }
    }
    hist
}

pub fn certify(oracle: &dyn TeacherOracle) {
    let c = compiler::load_corpus().expect("corpus incomplete: run `transformerless gen` first");
    let cut = (c.stories as f64 * 0.8) as u32;
    let test: Vec<usize> = (0..c.n).filter(|&i| c.story[i] >= cut).collect();
    let ntest = test.len();
    println!(
        "corpus: {} tokens, {} stories, held-out {}",
        c.n, c.stories, ntest
    );

    let (mut floor, mut ceil) = (0f64, 0u64);
    let mut zero_clamped = 0usize;
    for i in 0..c.n {
        if c.story[i] < cut {
            continue;
        }
        let (prob, clamped) = recorded_next_prob(&c.top_tokens[i], &c.top_weights[i], c.next[i]);
        if clamped {
            zero_clamped += 1;
        }
        floor += -prob.ln() / std::f64::consts::LN_2;
        if c.top_tokens[i][0] == c.next[i] {
            ceil += 1;
        }
    }
    println!(
        "teacher floor {:.4} bits/token | teacher ceiling {:.1}% | zero-quantized-weight collisions: {}/{}",
        floor / ntest as f64,
        100.0 * ceil as f64 / ntest as f64,
        zero_clamped,
        ntest
    );

    let art = compiler::compile(oracle, &c);
    compiler::save_artifacts(&art);

    // ---- store, by the runtime's own path (key identity by construction)
    let (store, codes) = build_store(&art, &c);

    // ---- equality witnesses: kernel path == plain path, three stages deep
    let mut rt = Runtime::new(&art);
    let rot = rt.rot;
    let sample_n = 512usize;
    let stride = c.n / sample_n;
    for s in 0..sample_n {
        let i = s * stride;
        let bk = bundle_kernel(&mut rt.kernel, &art, &rot, &c, i);
        let bp = bundle_plain(&art, &rot, &c, i);
        assert_eq!(bk, bp, "bundle kernel/plain divergence at {}", i);
        let ck = rt.assign(&c, i);
        let cp = code_plain(&art, &rot, &c, i);
        assert_eq!(ck, cp, "code kernel/plain divergence at {}", i);

        rt.state.clear_token_state();
        let (_, by_depth_k) =
            runtime::assign_memberships_plain(&art, &runtime::sig_plain(&art, &bk));
        let (_, by_depth_p) =
            runtime::assign_memberships_plain(&art, &runtime::sig_plain(&art, &bp));
        assert_eq!(
            rt.predict_witness_beam(&store, &by_depth_k).token,
            runtime::predict_witness_plain_beam(&store, &by_depth_p).token,
            "beam prediction kernel/plain divergence at {}",
            i
        );
    }
    println!(
        "equality witness: bundles, codes, predictions — kernel path == plain path on {}/{} sampled positions",
        sample_n, sample_n
    );
    let k = &rt.kernel;
    println!(
        "per-token op census (kernel path, n={}): add {:.0} | xor {:.0} | shift {:.0} | compare {:.0} | table-read {:.0} | multiply 0 (no such operation exists in the kernel)",
        sample_n,
        k.adds as f64 / sample_n as f64,
        k.xors as f64 / sample_n as f64,
        k.shifts as f64 / sample_n as f64,
        k.compares as f64 / sample_n as f64,
        k.table_reads as f64 / sample_n as f64
    );

    // ---- A: the shipped runtime (issue #281 — the #244 decision):
    // single-key store, read-time query-beam.
    let m = eval_query_beam(&c, &store, &art, &rot);
    println!(
        "A (shipped: single-key store + query-beam): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- A-multi ablation: the pre-#281 write-time fan-out (kept for
    // matrix continuity with the recorded #244 rows).
    let (store_multi, _) = runtime::build_store_multi(&art, &c);
    let m = eval(&c, &store_multi, STAGES, &|i, d| codes[i][..d].to_vec());
    println!(
        "A-multi (ablation, write-time fan-out): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );
    drop(store_multi);

    // ---- A-f32 ablation: nearest-centroid assignment (certifier-side)
    let bundles: Vec<[i64; D]> = (0..c.n).map(|i| bundle_plain(&art, &rot, &c, i)).collect();
    let codes_f32: Vec<[u8; STAGES]> = (0..c.n)
        .map(|i| {
            let b = &bundles[i];
            let mut work = [0f32; D];
            let mut nn = 0f32;
            for d in 0..D {
                let x = (b[d] - art.thresholds[d]) as f32;
                work[d] = x;
                nn += x * x;
            }
            let nn = nn.sqrt().max(1e-9);
            let mut work: Vec<f32> = work.iter().map(|x| x / nn).collect();
            let mut code = [0u8; STAGES];
            for (st, cb) in art.ctx_cb.iter().enumerate() {
                let (mut bd, mut bk) = (f32::MAX, 0usize);
                for kk in 0..K {
                    let cent = &cb[kk * D..(kk + 1) * D];
                    let mut d2 = 0f32;
                    for j in 0..D {
                        let t = work[j] - cent[j];
                        d2 += t * t;
                    }
                    if d2 < bd {
                        bd = d2;
                        bk = kk;
                    }
                }
                code[st] = bk as u8;
                for j in 0..D {
                    work[j] -= cb[bk * D + j];
                }
            }
            code
        })
        .collect();
    let store_f32 = build_store_generic(&c, STAGES, &|i, d| codes_f32[i][..d].to_vec());
    let m = eval(&c, &store_f32, STAGES, &|i, d| codes_f32[i][..d].to_vec());
    println!(
        "A-f32 (ablation, multiplies at assignment): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ============ Phase A decomposition rows (issue #243) ============
    // Certifier-side instrumentation ONLY (f32 permitted here, never in
    // the kernel): attribute the A-f32 ceiling's gap among the three
    // losses — normalization (#3), magnitude (#2), residuals (#1) —
    // before committing to a design. docs/graded_signature_address_design.md.

    let centered: Vec<[f32; D]> = bundles
        .iter()
        .map(|b| {
            let mut w = [0f32; D];
            for d in 0..D {
                w[d] = (b[d] - art.thresholds[d]) as f32;
            }
            w
        })
        .collect();
    let norms: Vec<f32> = centered
        .iter()
        .map(|w| w.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9))
        .collect();
    let nearest = |work: &[f32], cb: &[f32]| -> usize {
        let (mut bd, mut bk) = (f32::MAX, 0usize);
        for kk in 0..K {
            let cent = &cb[kk * D..(kk + 1) * D];
            let mut d2 = 0f32;
            for j in 0..D {
                let t = work[j] - cent[j];
                d2 += t * t;
            }
            if d2 < bd {
                bd = d2;
                bk = kk;
            }
        }
        bk
    };

    // ---- A-norm-only: "normalize → sign-bits, no residuals". Signs are
    // scale-invariant, so per-vector normalization survives only through
    // the assignment geometry: the sign-unit vector (±1/√D) is assigned
    // by Euclidean distance against the f32 codebook, per stage, with no
    // residual update. vs A-single this attributes the assignment
    // metric/codebook at sign-only information; vs A-f32 it leaves
    // magnitude+residual jointly unrecovered.
    let sign_scale = 1.0f32 / (D as f32).sqrt();
    let codes_norm: Vec<[u8; STAGES]> = (0..c.n)
        .map(|i| {
            let mut q = [0f32; D];
            for d in 0..D {
                q[d] = if centered[i][d] >= 0.0 {
                    sign_scale
                } else {
                    -sign_scale
                };
            }
            let mut code = [0u8; STAGES];
            for (st, cb) in art.ctx_cb.iter().enumerate() {
                code[st] = nearest(&q, cb) as u8;
            }
            code
        })
        .collect();
    let st_row = build_store_generic(&c, STAGES, &|i, d| codes_norm[i][..d].to_vec());
    let m = eval(&c, &st_row, STAGES, &|i, d| codes_norm[i][..d].to_vec());
    println!(
        "A-norm-only (#243 instrumentation, f32: sign-unit Euclidean assignment, no residuals): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- A-resid-only: residual VQ in RAW centered space — no
    // per-vector normalization anywhere. Isolates the value of residual
    // refinement (loss #1) without loss #3's fix.
    let codes_resid: Vec<[u8; STAGES]> = (0..c.n)
        .map(|i| {
            let mut work = centered[i];
            let mut code = [0u8; STAGES];
            for (st, cb) in art.ctx_cb.iter().enumerate() {
                let bk = nearest(&work, cb);
                code[st] = bk as u8;
                for j in 0..D {
                    work[j] -= cb[bk * D + j];
                }
            }
            code
        })
        .collect();
    let st_row = build_store_generic(&c, STAGES, &|i, d| codes_resid[i][..d].to_vec());
    let m = eval(&c, &st_row, STAGES, &|i, d| codes_resid[i][..d].to_vec());
    println!(
        "A-resid-only (#243 instrumentation, f32: raw-space residual VQ, no normalization): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- A-G(b): thermometer-graded assignment, no residuals. Ladders
    // are per-dimension quantiles of the NORMALIZED centered values
    // (deterministic stride-8 corpus sample) — normalization folds into
    // the ladder per the design doc. Prototypes are the f32 centroids
    // graded through the same ladder; stage>0 centroids are
    // residual-space objects graded in original-space ladders, so these
    // rows LOWER-BOUND Design G (Phase B retrains prototypes in graded
    // space). Hamming on thermometer codes equals L1 on bucket indices;
    // L1 is computed directly.
    for bq in [2usize, 3, 4] {
        let ladder: Vec<Vec<f32>> = (0..D)
            .map(|d| {
                let mut vals: Vec<f32> = (0..c.n)
                    .step_by(8)
                    .map(|i| centered[i][d] / norms[i])
                    .collect();
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                (1..=bq).map(|j| vals[j * vals.len() / (bq + 1)]).collect()
            })
            .collect();
        let grade_val =
            |v: f32, lad: &[f32]| -> i32 { lad.iter().filter(|&&t| v > t).count() as i32 };
        let proto: Vec<Vec<i32>> = art
            .ctx_cb
            .iter()
            .map(|cb| {
                let mut g = vec![0i32; K * D];
                for kk in 0..K {
                    for d in 0..D {
                        g[kk * D + d] = grade_val(cb[kk * D + d], &ladder[d]);
                    }
                }
                g
            })
            .collect();
        let codes_g: Vec<[u8; STAGES]> = (0..c.n)
            .map(|i| {
                let g: Vec<i32> = (0..D)
                    .map(|d| grade_val(centered[i][d] / norms[i], &ladder[d]))
                    .collect();
                let mut code = [0u8; STAGES];
                for (st, pr) in proto.iter().enumerate() {
                    let (mut bd, mut bk) = (i64::MAX, 0usize);
                    for kk in 0..K {
                        let mut l1 = 0i64;
                        for d in 0..D {
                            l1 += (g[d] - pr[kk * D + d]).abs() as i64;
                        }
                        if l1 < bd {
                            bd = l1;
                            bk = kk;
                        }
                    }
                    code[st] = bk as u8;
                }
                code
            })
            .collect();
        let st_row = build_store_generic(&c, STAGES, &|i, d| codes_g[i][..d].to_vec());
        let m = eval(&c, &st_row, STAGES, &|i, d| codes_g[i][..d].to_vec());
        println!(
            "A-G({bq}) (#243 instrumentation: thermometer-graded assignment [normalized-space ladder], no residuals): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
            m.top1, m.agree, m.wb_bits, m.keys
        );
    }

    // ---- A-R: Design R as buildable today — integer centroid copies
    // scaled ONCE by the corpus-mean norm (a compile-time constant, so
    // the query loop is add/sub + sign re-threshold + Hamming only), and
    // per-stage class_sigs as trained (NOT retrained in residual space —
    // the doc's recorded risk, measured as-is). Re-signing reuses
    // sig_plain via a re-centered bundle so bit packing is canonical.
    let mean_norm = norms.iter().sum::<f32>() / norms.len() as f32;
    let cent_int: Vec<Vec<i64>> = art
        .ctx_cb
        .iter()
        .map(|cb| cb.iter().map(|&x| (x * mean_norm).round() as i64).collect())
        .collect();
    let hamming =
        |a: &[u8], b: &[u8]| -> u32 { a.iter().zip(b).map(|(x, y)| (x ^ y).count_ones()).sum() };
    let assign_r = |i: usize| -> [u8; STAGES] {
        let mut r: [i64; D] = std::array::from_fn(|d| bundles[i][d] - art.thresholds[d]);
        let mut code = [0u8; STAGES];
        for st in 0..STAGES {
            let mut fake = [0i64; D];
            for d in 0..D {
                fake[d] = r[d] + art.thresholds[d];
            }
            let sig = runtime::sig_plain(&art, &fake);
            let sigs_st = &art.class_sigs[st];
            let (mut bd, mut bk) = (u32::MAX, 0usize);
            for kk in 0..K {
                let cs = &sigs_st[kk * runtime::SIG_BYTES..(kk + 1) * runtime::SIG_BYTES];
                let h = hamming(&sig, cs);
                if h < bd {
                    bd = h;
                    bk = kk;
                }
            }
            code[st] = bk as u8;
            for d in 0..D {
                r[d] -= cent_int[st][bk * D + d];
            }
        }
        code
    };
    let codes_r: Vec<[u8; STAGES]> = (0..c.n).map(assign_r).collect();
    let st_row = build_store_generic(&c, STAGES, &|i, d| codes_r[i][..d].to_vec());
    let m = eval(&c, &st_row, STAGES, &|i, d| codes_r[i][..d].to_vec());
    println!(
        "A-R (#243 buildable shape: constant-scaled integer centroids + sign re-threshold, class_sigs unretrained): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- A-R∘G(b): graded signature OF the residual. Per-stage ladders
    // from the R-loop's residual distributions (deterministic stride-8
    // sample); prototypes are the integer centroid copies graded through
    // the SAME stage ladder. Same lower-bound caveat as A-G: prototypes
    // are not retrained in graded space.
    for bq in [2usize, 3, 4] {
        let mut stage_vals: Vec<Vec<Vec<f32>>> = vec![vec![Vec::new(); D]; STAGES];
        for i in (0..c.n).step_by(8) {
            let mut r: [i64; D] = std::array::from_fn(|d| bundles[i][d] - art.thresholds[d]);
            for st in 0..STAGES {
                for d in 0..D {
                    stage_vals[st][d].push(r[d] as f32);
                }
                let code = codes_r[i][st] as usize;
                for d in 0..D {
                    r[d] -= cent_int[st][code * D + d];
                }
            }
        }
        let ladders: Vec<Vec<Vec<f32>>> = stage_vals
            .iter()
            .map(|per_dim| {
                per_dim
                    .iter()
                    .map(|vals| {
                        let mut v = vals.clone();
                        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        (1..=bq).map(|j| v[j * v.len() / (bq + 1)]).collect()
                    })
                    .collect()
            })
            .collect();
        let grade_val =
            |v: f32, lad: &[f32]| -> i32 { lad.iter().filter(|&&t| v > t).count() as i32 };
        let proto: Vec<Vec<i32>> = (0..STAGES)
            .map(|st| {
                let mut g = vec![0i32; K * D];
                for kk in 0..K {
                    for d in 0..D {
                        g[kk * D + d] = grade_val(cent_int[st][kk * D + d] as f32, &ladders[st][d]);
                    }
                }
                g
            })
            .collect();
        let codes_rg: Vec<[u8; STAGES]> = (0..c.n)
            .map(|i| {
                let mut r: [i64; D] = std::array::from_fn(|d| bundles[i][d] - art.thresholds[d]);
                let mut code = [0u8; STAGES];
                for st in 0..STAGES {
                    let g: Vec<i32> = (0..D)
                        .map(|d| grade_val(r[d] as f32, &ladders[st][d]))
                        .collect();
                    let pr = &proto[st];
                    let (mut bd, mut bk) = (i64::MAX, 0usize);
                    for kk in 0..K {
                        let mut l1 = 0i64;
                        for d in 0..D {
                            l1 += (g[d] - pr[kk * D + d]).abs() as i64;
                        }
                        if l1 < bd {
                            bd = l1;
                            bk = kk;
                        }
                    }
                    code[st] = bk as u8;
                    for d in 0..D {
                        r[d] -= cent_int[st][bk * D + d];
                    }
                }
                code
            })
            .collect();
        let st_row = build_store_generic(&c, STAGES, &|i, d| codes_rg[i][..d].to_vec());
        let m = eval(&c, &st_row, STAGES, &|i, d| codes_rg[i][..d].to_vec());
        println!(
            "A-R∘G({bq}) (#243 buildable shape: graded residual assignment [residual-space ladders], prototypes unretrained): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
            m.top1, m.agree, m.wb_bits, m.keys
        );
    }
    // ============ end Phase A decomposition rows (issue #243) ============

    // ---- B: bit-prefix coordinate — signature bytes, no classes
    let sigs: Vec<[u8; runtime::SIG_BYTES]> = (0..c.n)
        .map(|i| runtime::sig_plain(&art, &bundles[i]))
        .collect();
    let bdepths = 6usize;
    let key_b = |i: usize, d: usize| -> Vec<u8> { sigs[i][..d].to_vec() };
    let store_b = build_store_generic(&c, bdepths, &key_b);
    let m = eval(&c, &store_b, bdepths, &key_b);
    println!(
        "B bit-prefix (mul-free, no codebook classes; depths 8..48 bits): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- A-single ablation: primary-key queries against the shipped
    // single-key store (no read-time beam) — measures what the beam buys.
    // The former standalone query-beam row is now the shipped A row above.
    let m = eval(&c, &store, STAGES, &|i, d| codes[i][..d].to_vec());
    println!(
        "A-single (ablation, no query beam): top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token | {} keys",
        m.top1, m.agree, m.wb_bits, m.keys
    );

    // ---- C: serving surface (issue #280)
    // The former C row here measured the `convert_r4g1` scaffold, which
    // was never a functional prediction path (issue #280 diagnosis); its
    // recorded 0.0% stands in the issue as the reason it left the
    // measurement matrix. The C row now measures the configuration that
    // actually serves — `uor-r4-api`'s `R4Engine` over a compiled
    // bundle's `score.r4g1` with the D4 policy — and lives in
    // `uor_r4_api::serving_eval`, driven by the `r4 certify` command
    // (this crate sits below `uor-r4-api` in the dependency graph).

    // ================= COMPRESSION (PROOF.md P5) =================

    // (a) container round-trip: load what was saved; byte- and κ-identity.
    let saved = std::fs::read(compiler::ART_PATH).unwrap();
    let reloaded = compiler::load_artifacts().expect("reload");
    assert_eq!(reloaded.token_codes, art.token_codes);
    assert_eq!(reloaded.stage_books, art.stage_books);
    assert_eq!(reloaded.thresholds, art.thresholds);
    assert_eq!(reloaded.class_sigs, art.class_sigs);
    compiler::save_artifacts(&reloaded);
    let resaved = std::fs::read(compiler::ART_PATH).unwrap();
    assert_eq!(
        saved, resaved,
        "container round-trip must be byte-identical"
    );
    println!(
        "compression witness (container): save → load → save is byte-identical ({} bytes, κ stable)",
        saved.len()
    );

    // (b) rate–distortion of the shipped token representation: decode at
    // prefix depth d (i8 book sums — the exact bytes the runtime reads)
    // against the source's centered, normalized embedding rows, read
    // through the same oracle surface the compiler used.
    let seed_string = format!(
        "{}{}{}",
        oracle.kappa(),
        oracle.tokenizer_address(),
        "r4-geometric-projection-v1"
    );
    let seed_hash = blake3::hash(seed_string.as_bytes());
    let seed_bytes = seed_hash.as_bytes();
    let source_dim = oracle.source_dimension();
    let src = compiler::deterministic_project(seed_bytes, V, source_dim, D, oracle);
    let src_bytes = V * D * 4;
    println!(
        "compression (representation): source embedding table {} bytes (f32 {}×{})",
        src_bytes, V, D
    );
    for depth in 1..=STAGES {
        let mut acc = 0f64;
        for t in 0..V {
            let mut rec = [0i32; D];
            runtime::decode_row_prefix_plain(&art, t as u32, depth, &mut rec);
            let s = &src[t * D..(t + 1) * D];
            let (mut dot, mut na, mut nb) = (0f64, 0f64, 0f64);
            for j in 0..D {
                dot += rec[j] as f64 * s[j] as f64;
                na += (rec[j] as f64) * (rec[j] as f64);
                nb += (s[j] as f64) * (s[j] as f64);
            }
            acc += dot / (na.sqrt() * nb.sqrt()).max(1e-12);
        }
        let bytes = V * depth + depth * K * D; // codes at depth + books at depth
        println!(
            "  depth {}: {} bytes total ({:.1}× vs source) | mean cosine to source rows {:.4}",
            depth,
            bytes,
            src_bytes as f64 / bytes as f64,
            acc / V as f64
        );
    }

    // (c) end-to-end artifact accounting.
    let runtime_bytes = art.token_codes.len()
        + art.stage_books.iter().map(|b| b.len()).sum::<usize>()
        + art.thresholds.len() * 8
        + art.class_sigs.iter().map(|s| s.len()).sum::<usize>();
    let store_bytes: usize = store
        .iter()
        .flat_map(|l| l.iter())
        .map(|(k, v)| k.len() + v.len() * 6)
        .sum();
    println!(
        "compression (artifact): runtime tables {} bytes + store ≈ {} bytes = {} vs source checkpoint {} bytes ({:.1}×), at the residual certified above",
        runtime_bytes,
        store_bytes,
        runtime_bytes + store_bytes,
        oracle.source_bytes(),
        oracle.source_bytes() as f64 / (runtime_bytes + store_bytes) as f64
    );

    crate::long_context::certify_long_context();
}

fn build_store_generic(
    c: &compiler::Corpus,
    depths: usize,
    key: &dyn Fn(usize, usize) -> Vec<u8>,
) -> Store {
    let cut = (c.stories as f64 * 0.8) as u32;
    let mut levels: Store = (0..=depths).map(|_| BTreeMap::new()).collect();
    for i in 0..c.n {
        if c.story[i] >= cut {
            continue;
        }
        for k_idx in 0..c.top_tokens[i].len() {
            let tok = c.top_tokens[i][k_idx];
            let weight = c.top_weights[i][k_idx];
            if weight > 0 {
                *levels[0].entry(vec![]).or_default().entry(tok).or_default() += weight;
                for (d, level) in levels.iter_mut().enumerate().take(depths + 1).skip(1) {
                    *level.entry(key(i, d)).or_default().entry(tok).or_default() += weight;
                }
            }
        }
    }
    levels
}
