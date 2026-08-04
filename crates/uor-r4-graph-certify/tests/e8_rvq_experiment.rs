//! #395 construction experiment v1: residual-E8 — the fair fight.
//!
//! v0 (PR #403) recorded a negative for flat spatial-group keying and traced
//! it to OOV flooding, explicitly deferring the residual shape. This
//! experiment isolates ONE variable: does icosian/E8 lattice structure in the
//! stage codebooks help or hurt, with training held equal.
//!
//! Arms, all with identical 4-stage x 256-centroid residual-VQ skeleton,
//! identical k-means training (deterministic subsample, 8 Lloyd iterations),
//! identical store build and eval:
//!
//! - `shipped-geometry`: the artifact's own compiled codes (reference row).
//! - `rvq-f32`: retrained plain residual VQ — the honest control for the
//!   retraining confound (loaded artifacts do not carry their f32 stage
//!   centroids, so a lattice arm can only be compared against a same-budget
//!   retrained baseline, never against the shipped compile directly).
//! - `rvq-e8@m`: the SAME trained centroids, each 8-dim block snapped to its
//!   nearest E8 lattice point at scale m x block-RMS, for m in 0.5 / 1 / 2 —
//!   a lattice-coarseness sweep.
//!
//! Certifier-side instrumentation only (f32/f64 and allocation permitted
//! here, never in the kernel).
//!
//! Run:
//!   cargo test --release -p uor-r4-graph-certify --test e8_rvq_experiment -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::compiler::{D, STAGES};
use uor_r4_core::transformerless::runtime;

const BLOCK: usize = 8;
const NBLK: usize = D / BLOCK; // 36
const K: usize = 256; // centroids per stage
const LLOYD_ITERS: usize = 8;
const TRAIN_STEP: usize = 8; // construction subsample stride for k-means

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Nearest D8 point (integer coords, even sum).
fn d8_round(y: &[f64; BLOCK]) -> [f64; BLOCK] {
    let mut f = [0f64; BLOCK];
    for i in 0..BLOCK {
        f[i] = y[i].round();
    }
    let sum: i64 = f.iter().map(|v| *v as i64).sum();
    if sum.rem_euclid(2) != 0 {
        let (mut bi, mut be) = (0usize, -1f64);
        for i in 0..BLOCK {
            let e = (y[i] - f[i]).abs();
            if e > be {
                be = e;
                bi = i;
            }
        }
        f[bi] += if y[bi] > f[bi] { 1.0 } else { -1.0 };
    }
    f
}

/// Nearest E8 point (best of D8 and D8+half).
fn e8_snap(y: &[f64; BLOCK]) -> [f64; BLOCK] {
    let a = d8_round(y);
    let mut yh = [0f64; BLOCK];
    for i in 0..BLOCK {
        yh[i] = y[i] - 0.5;
    }
    let b0 = d8_round(&yh);
    let mut b = [0f64; BLOCK];
    for i in 0..BLOCK {
        b[i] = b0[i] + 0.5;
    }
    let da: f64 = (0..BLOCK).map(|i| (y[i] - a[i]).powi(2)).sum();
    let db: f64 = (0..BLOCK).map(|i| (y[i] - b[i]).powi(2)).sum();
    if da <= db {
        a
    } else {
        b
    }
}

type Store = Vec<BTreeMap<Vec<u8>, BTreeMap<u32, u32>>>;

fn build_store_from_codes(
    c: &compiler::Corpus,
    cut: u32,
    key: &dyn Fn(usize, usize) -> Vec<u8>,
) -> Store {
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    for i in 0..c.n {
        if c.story[i] >= cut {
            continue;
        }
        for k_idx in 0..c.top_tokens[i].len() {
            let (tok, weight) = (c.top_tokens[i][k_idx], c.top_weights[i][k_idx]);
            if weight > 0 {
                for (d, level) in store.iter_mut().enumerate() {
                    *level.entry(key(i, d)).or_default().entry(tok).or_default() += weight;
                }
            }
        }
    }
    store
}

struct EvalOut {
    top1: f64,
    agree: f64,
    wb_bits: f64,
    keys: usize,
    infill_top1: f64,
}

fn eval_store(
    c: &compiler::Corpus,
    cut: u32,
    positions: &[usize],
    store: &Store,
    key: &dyn Fn(usize, usize) -> Vec<u8>,
) -> EvalOut {
    let (mut top1, mut agree, mut bits) = (0u64, 0u64, 0f64);
    let (mut n, mut inf_hits, mut inf_n) = (0u64, 0u64, 0u64);
    #[allow(clippy::needless_range_loop)] // i indexes parallel corpus arrays
    for i in 0..c.n {
        if c.story[i] < cut {
            continue;
        }
        n += 1;
        let mut pred = None;
        for d in (0..=STAGES).rev() {
            if let Some(dist) = store[d].get(&key(i, d)) {
                pred = dist
                    .iter()
                    .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                    .map(|(&t, _)| t);
                break;
            }
        }
        let pred = pred.unwrap_or(0);
        if pred == c.next[i] {
            top1 += 1;
        }
        if pred == c.t_argmax[i] {
            agree += 1;
        }
        let target_pos = positions[i] + 1;
        if !target_pos.is_multiple_of(4) {
            inf_n += 1;
            if pred == c.next[i] {
                inf_hits += 1;
            }
        }
        let mut lams: Vec<(f64, &BTreeMap<u32, u32>, u32)> = Vec::new();
        for (d, level) in store.iter().enumerate().take(STAGES + 1) {
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
    EvalOut {
        top1: 100.0 * top1 as f64 / n as f64,
        agree: 100.0 * agree as f64 / n as f64,
        wb_bits: bits / n as f64,
        keys: store.iter().map(|l| l.len()).sum(),
        infill_top1: 100.0 * inf_hits as f64 / inf_n as f64,
    }
}

fn story_positions(c: &compiler::Corpus) -> Vec<usize> {
    let mut positions = Vec::with_capacity(c.n);
    let (mut cur, mut pos) = (u32::MAX, 0usize);
    for i in 0..c.n {
        if c.story[i] != cur {
            cur = c.story[i];
            pos = 0;
        } else {
            pos += 1;
        }
        positions.push(pos);
    }
    positions
}

/// Deterministic k-means: stride-sampled init, Lloyd iterations, empty
/// clusters reseeded from the worst-fit training vector.
fn kmeans(train: &[f32], dim: usize, k: usize, iters: usize) -> Vec<f32> {
    let n = train.len() / dim;
    let mut centroids = vec![0f32; k * dim];
    for j in 0..k {
        let src = (j * n / k) * dim;
        centroids[j * dim..(j + 1) * dim].copy_from_slice(&train[src..src + dim]);
    }
    let mut assign = vec![0usize; n];
    for _ in 0..iters {
        // assignment
        for i in 0..n {
            let v = &train[i * dim..(i + 1) * dim];
            let (mut bj, mut bd) = (0usize, f32::MAX);
            for j in 0..k {
                let cent = &centroids[j * dim..(j + 1) * dim];
                let mut d2 = 0f32;
                for t in 0..dim {
                    let r = v[t] - cent[t];
                    d2 += r * r;
                }
                if d2 < bd {
                    bd = d2;
                    bj = j;
                }
            }
            assign[i] = bj;
        }
        // update
        let mut sums = vec![0f64; k * dim];
        let mut counts = vec![0u64; k];
        for i in 0..n {
            counts[assign[i]] += 1;
            for t in 0..dim {
                sums[assign[i] * dim + t] += train[i * dim + t] as f64;
            }
        }
        for j in 0..k {
            if counts[j] == 0 {
                // reseed deterministically from a spread position
                let src = ((j * 2_654_435_761) % n) * dim;
                centroids[j * dim..(j + 1) * dim].copy_from_slice(&train[src..src + dim]);
                continue;
            }
            for t in 0..dim {
                centroids[j * dim + t] = (sums[j * dim + t] / counts[j] as f64) as f32;
            }
        }
    }
    centroids
}

/// Assign the full corpus through a 4-stage residual VQ, returning codes.
fn rvq_assign(bundles: &[f32], n: usize, stage_cbs: &[Vec<f32>]) -> Vec<[u8; STAGES]> {
    let mut codes = vec![[0u8; STAGES]; n];
    for i in 0..n {
        let mut r = [0f32; D];
        r.copy_from_slice(&bundles[i * D..(i + 1) * D]);
        for (s, cb) in stage_cbs.iter().enumerate() {
            let (mut bj, mut bd) = (0usize, f32::MAX);
            for j in 0..K {
                let cent = &cb[j * D..(j + 1) * D];
                let mut d2 = 0f32;
                for t in 0..D {
                    let e = r[t] - cent[t];
                    d2 += e * e;
                }
                if d2 < bd {
                    bd = d2;
                    bj = j;
                }
            }
            codes[i][s] = bj as u8;
            let cent = &cb[bj * D..(bj + 1) * D];
            for t in 0..D {
                r[t] -= cent[t];
            }
        }
    }
    codes
}

/// Snap every centroid's 8-dim blocks to the E8 lattice at scale
/// `mult x block RMS` (RMS over the centroid set, per block).
fn snap_codebooks(stage_cbs: &[Vec<f32>], mult: f64) -> Vec<Vec<f32>> {
    stage_cbs
        .iter()
        .map(|cb| {
            let mut out = cb.clone();
            for blk in 0..NBLK {
                let mut sq = 0f64;
                for j in 0..K {
                    for t in 0..BLOCK {
                        let v = cb[j * D + blk * BLOCK + t] as f64;
                        sq += v * v;
                    }
                }
                let scale = (sq / (K * BLOCK) as f64).sqrt().max(1e-9) * mult;
                for j in 0..K {
                    let mut y = [0f64; BLOCK];
                    for t in 0..BLOCK {
                        y[t] = cb[j * D + blk * BLOCK + t] as f64 / scale;
                    }
                    let s = e8_snap(&y);
                    for t in 0..BLOCK {
                        out[j * D + blk * BLOCK + t] = (s[t] * scale) as f32;
                    }
                }
            }
            out
        })
        .collect()
}

#[test]
#[ignore = "construction experiment; run explicitly with --ignored"]
fn e8_residual_vq_vs_controls() {
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let art = compiler::load_artifacts_from(&fixture("tless_artifacts.bin"))
        .expect("checked-in fixture artifacts");
    let cut = (c.stories as f64 * 0.8) as u32;
    let positions = story_positions(&c);
    let rot = compiler::derive_rotations();
    println!("e8-rvq experiment (#395 v1): {} records, cut {}", c.n, cut);

    // ---- single bundle pass ----
    let mut bundles: Vec<f32> = Vec::with_capacity(c.n * D);
    let mut shipped_codes: Vec<[u8; STAGES]> = Vec::with_capacity(c.n);
    for i in 0..c.n {
        let b = runtime::bundle_plain(&art, &rot, &c, i);
        shipped_codes.push(runtime::assign_for_bundle(&art, &b));
        let w = runtime::centered_work(&art, &b);
        bundles.extend(w.iter().map(|&v| v as f32));
    }
    println!("bundle pass done");

    // ---- k-means residual training on construction subsample ----
    let mut stage_cbs: Vec<Vec<f32>> = Vec::with_capacity(STAGES);
    let train_idx: Vec<usize> = (0..c.n)
        .step_by(TRAIN_STEP)
        .filter(|&i| c.story[i] < cut)
        .collect();
    let mut train: Vec<f32> = Vec::with_capacity(train_idx.len() * D);
    for &i in &train_idx {
        train.extend_from_slice(&bundles[i * D..(i + 1) * D]);
    }
    println!("k-means training on {} vectors", train_idx.len());
    for s in 0..STAGES {
        let cb = kmeans(&train, D, K, LLOYD_ITERS);
        // residual update of the training set
        let tn = train.len() / D;
        for i in 0..tn {
            let v = train[i * D..(i + 1) * D].to_vec();
            let (mut bj, mut bd) = (0usize, f32::MAX);
            for j in 0..K {
                let cent = &cb[j * D..(j + 1) * D];
                let mut d2 = 0f32;
                for t in 0..D {
                    let e = v[t] - cent[t];
                    d2 += e * e;
                }
                if d2 < bd {
                    bd = d2;
                    bj = j;
                }
            }
            for t in 0..D {
                train[i * D + t] -= cb[bj * D + t];
            }
        }
        println!("stage {s} trained");
        stage_cbs.push(cb);
    }
    drop(train);

    // ---- arms ----
    let ship_key = |i: usize, d: usize| shipped_codes[i][..d].to_vec();
    let ship_store = build_store_from_codes(&c, cut, &ship_key);
    let m = eval_store(&c, cut, &positions, &ship_store, &ship_key);
    println!(
        "shipped-geometry     top1 {:.1}% | agree {:.1}% | WB {:.4} bits | {} keys | infill-free top1 {:.1}%",
        m.top1, m.agree, m.wb_bits, m.keys, m.infill_top1
    );
    drop(ship_store);

    let mut arms: Vec<(String, Vec<Vec<f32>>)> = vec![("rvq-f32".to_owned(), stage_cbs.clone())];
    for mult in [0.5f64, 1.0, 2.0] {
        arms.push((
            format!("rvq-e8@{mult}rms"),
            snap_codebooks(&stage_cbs, mult),
        ));
    }
    for (name, cbs) in &arms {
        let codes = rvq_assign(&bundles, c.n, cbs);
        let key = |i: usize, d: usize| codes[i][..d].to_vec();
        let store = build_store_from_codes(&c, cut, &key);
        let m = eval_store(&c, cut, &positions, &store, &key);
        println!(
            "{name:<20} top1 {:.1}% | agree {:.1}% | WB {:.4} bits | {} keys | infill-free top1 {:.1}%",
            m.top1, m.agree, m.wb_bits, m.keys, m.infill_top1
        );
    }
}
