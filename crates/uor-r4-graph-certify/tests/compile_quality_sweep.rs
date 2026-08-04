//! #407 step 2: attribute the +2pp retrained-RVQ delta (PR #404's control
//! finding) between the two training choices that differ from the shipped
//! compile path.
//!
//! Step-1 diff (recorded on the issue): the shipped compile trains its
//! context codebooks via `sampled_kmeans_rvq(CTX_SAMPLE = 6_000, CTX_ITERS
//! = 6)` — about 23 samples per centroid across 4 stages x 256 — while the
//! harness control used ~50k samples and 8 iterations.
//!
//! Arms (identical machinery, store build and eval; one batched run per
//! Rule 1): (samples, iters) in {6k, 50k} x {6, 8}, plus the shipped
//! artifact codes as the reference row.
//!
//! Run:
//!   cargo test --release -p uor-r4-graph-certify --test compile_quality_sweep -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::compiler::{D, STAGES};
use uor_r4_core::transformerless::runtime;

const K: usize = 256;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
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

fn eval_store(
    c: &compiler::Corpus,
    cut: u32,
    store: &Store,
    key: &dyn Fn(usize, usize) -> Vec<u8>,
) -> (f64, f64, f64, usize) {
    let (mut top1, mut agree, mut bits, mut n) = (0u64, 0u64, 0f64, 0u64);
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
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
    (
        100.0 * top1 as f64 / n as f64,
        100.0 * agree as f64 / n as f64,
        bits / n as f64,
        store.iter().map(|l| l.len()).sum(),
    )
}

fn kmeans(train: &[f32], dim: usize, k: usize, iters: usize) -> Vec<f32> {
    let n = train.len() / dim;
    let mut centroids = vec![0f32; k * dim];
    for j in 0..k {
        let src = (j * n / k) * dim;
        centroids[j * dim..(j + 1) * dim].copy_from_slice(&train[src..src + dim]);
    }
    let mut assign = vec![0usize; n];
    for _ in 0..iters {
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

fn train_rvq(train_src: &[f32], iters: usize) -> Vec<Vec<f32>> {
    let mut train = train_src.to_vec();
    let mut stage_cbs = Vec::with_capacity(STAGES);
    for _ in 0..STAGES {
        let cb = kmeans(&train, D, K, iters);
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
        stage_cbs.push(cb);
    }
    stage_cbs
}

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

#[test]
#[ignore = "measurement sweep; run explicitly with --ignored"]
fn compile_quality_sweep() {
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let art = compiler::load_artifacts_from(&fixture("tless_artifacts.bin"))
        .expect("checked-in fixture artifacts");
    let cut = (c.stories as f64 * 0.8) as u32;
    let rot = compiler::derive_rotations();
    println!("compile-quality sweep (#407): {} records", c.n);

    let mut bundles: Vec<f32> = Vec::with_capacity(c.n * D);
    let mut shipped_codes: Vec<[u8; STAGES]> = Vec::with_capacity(c.n);
    for i in 0..c.n {
        let b = runtime::bundle_plain(&art, &rot, &c, i);
        shipped_codes.push(runtime::assign_for_bundle(&art, &b));
        let w = runtime::centered_work(&art, &b);
        bundles.extend(w.iter().map(|&v| v as f32));
    }
    println!("bundle pass done");

    let ship_key = |i: usize, d: usize| shipped_codes[i][..d].to_vec();
    let store = build_store_from_codes(&c, cut, &ship_key);
    let (t1, ag, wb, keys) = eval_store(&c, cut, &store, &ship_key);
    println!("shipped-artifact       top1 {t1:.1}% | agree {ag:.1}% | WB {wb:.4} | {keys} keys");
    drop(store);

    // deterministic strided construction samples at two sizes
    let make_train = |target: usize| -> Vec<f32> {
        let constr: Vec<usize> = (0..c.n).filter(|&i| c.story[i] < cut).collect();
        let step = (constr.len() / target).max(1);
        let mut train = Vec::with_capacity((constr.len() / step + 1) * D);
        for &i in constr.iter().step_by(step) {
            train.extend_from_slice(&bundles[i * D..(i + 1) * D]);
        }
        train
    };

    for (samples, iters) in [(6_000usize, 6usize), (6_000, 8), (50_000, 6), (50_000, 8)] {
        let train = make_train(samples);
        let cbs = train_rvq(&train, iters);
        let codes = rvq_assign(&bundles, c.n, &cbs);
        let key = |i: usize, d: usize| codes[i][..d].to_vec();
        let store = build_store_from_codes(&c, cut, &key);
        let (t1, ag, wb, keys) = eval_store(&c, cut, &store, &key);
        println!(
            "rvq s={samples:<6} it={iters} top1 {t1:.1}% | agree {ag:.1}% | WB {wb:.4} | {keys} keys (trained on {} vecs)",
            train.len() / D
        );
    }
}
