//! #395 construction experiment: a graded store whose addressing geometry is
//! E8/icosian block codes instead of the shipped residual-VQ sign codes — at
//! MATCHED key budget (4 stages × 1 byte, 256-entry codebooks per stage,
//! identical store shape, backoff and evidence rules).
//!
//! Pipeline per record: the centered context bundle (the exact shipped
//! object) is split into 36 blocks of 8 dims; each block is decoded to its
//! nearest E8 lattice point (Conway–Sloane) at a per-block scale grid-picked
//! on construction data, mapped through a per-block id dictionary of the 255
//! most frequent construction lattice points plus OOV, and stage k of the
//! graded key is a 255-pattern codebook over the 9 block ids covering dims
//! 72k..72k+71 plus OOV. Both stores are then built with the SAME evidence
//! rule and graded by the SAME replica eval: top1, teacher agreement, WB
//! bits, and the free-position infill slice at stride 4 per issue 394.
//!
//! Certifier-side instrumentation only (f32/f64 and allocation permitted
//! here, never in the kernel).
//!
//! Run:
//!   cargo test --release -p uor-r4-graph-certify --test e8_store_experiment -- --ignored --nocapture

use std::collections::BTreeMap;
use std::collections::HashMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::compiler::{D, STAGES};
use uor_r4_core::transformerless::runtime;

const BLOCK: usize = 8;
const NBLK: usize = D / BLOCK; // 36
const GROUPS: usize = STAGES; // 4 stages
const BLK_PER_GROUP: usize = NBLK / GROUPS; // 9
const OOV: u8 = 255;

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

/// Nearest E8 point (best of D8 and D8+½), returned ×2 so half-integers are
/// exact in i16.
fn e8_decode(y: &[f64; BLOCK]) -> [i16; BLOCK] {
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
    let p = if da <= db { a } else { b };
    let mut out = [0i16; BLOCK];
    for i in 0..BLOCK {
        out[i] = (p[i] * 2.0).round() as i16;
    }
    out
}

type Store = Vec<BTreeMap<Vec<u8>, BTreeMap<u32, u32>>>;

/// Same evidence rule as `runtime::build_store`: construction records only,
/// weighted teacher top-k.
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

/// Replica of the certify `eval` metrics (deepest-argmax backoff, WB bits via
/// the λ-smoothed backoff mixture) plus the #394 free-position slice.
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

#[test]
#[ignore = "construction experiment; run explicitly with --ignored"]
fn e8_store_vs_shipped() {
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let art = compiler::load_artifacts_from(&fixture("tless_artifacts.bin"))
        .expect("checked-in fixture artifacts");
    let cut = (c.stories as f64 * 0.8) as u32;
    let positions = story_positions(&c);
    let rot = compiler::derive_rotations();
    println!("e8-store experiment (#395): {} records, cut {}", c.n, cut);

    // ---- single bundle pass: centered f32 bundles + shipped codes ----
    let mut bundles: Vec<f32> = Vec::with_capacity(c.n * D);
    let mut shipped_codes: Vec<[u8; STAGES]> = Vec::with_capacity(c.n);
    for i in 0..c.n {
        let b = runtime::bundle_plain(&art, &rot, &c, i);
        shipped_codes.push(runtime::assign_for_bundle(&art, &b));
        let w = runtime::centered_work(&art, &b);
        bundles.extend(w.iter().map(|&v| v as f32));
    }
    println!("bundle pass done");

    // ---- per-block scales: grid on construction subsample ----
    let mut scales = [0f64; NBLK];
    for blk in 0..NBLK {
        let mut sq = 0f64;
        let mut cnt = 0u64;
        for i in (0..c.n).step_by(50) {
            if c.story[i] >= cut {
                continue;
            }
            for d in 0..BLOCK {
                let v = bundles[i * D + blk * BLOCK + d] as f64;
                sq += v * v;
            }
            cnt += BLOCK as u64;
        }
        let rms = (sq / cnt as f64).sqrt().max(1e-9);
        let mut best = (f64::INFINITY, rms);
        for mult in [0.25, 0.5, 1.0, 2.0, 4.0] {
            let s = rms * mult;
            let mut mse = 0f64;
            for i in (0..c.n).step_by(500) {
                if c.story[i] >= cut {
                    continue;
                }
                let mut y = [0f64; BLOCK];
                for d in 0..BLOCK {
                    y[d] = bundles[i * D + blk * BLOCK + d] as f64 / s;
                }
                let q = e8_decode(&y);
                for d in 0..BLOCK {
                    let r = y[d] - q[d] as f64 / 2.0;
                    mse += r * r;
                }
            }
            if mse < best.0 {
                best = (mse, s);
            }
        }
        scales[blk] = best.1;
    }

    // ---- E8 decode all records; per-block id dictionaries (construction) ----
    let mut lattice: Vec<[i16; BLOCK]> = Vec::with_capacity(c.n * NBLK);
    for i in 0..c.n {
        for blk in 0..NBLK {
            let mut y = [0f64; BLOCK];
            for d in 0..BLOCK {
                y[d] = bundles[i * D + blk * BLOCK + d] as f64 / scales[blk];
            }
            lattice.push(e8_decode(&y));
        }
    }
    drop(bundles);
    let mut block_ids: Vec<u8> = vec![OOV; c.n * NBLK];
    for blk in 0..NBLK {
        let mut freq: HashMap<[i16; BLOCK], u32> = HashMap::new();
        for i in 0..c.n {
            if c.story[i] < cut {
                *freq.entry(lattice[i * NBLK + blk]).or_default() += 1;
            }
        }
        let mut by_freq: Vec<_> = freq.into_iter().collect();
        by_freq.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        let dict: HashMap<[i16; BLOCK], u8> = by_freq
            .iter()
            .take(255)
            .enumerate()
            .map(|(id, (pt, _))| (*pt, id as u8))
            .collect();
        for i in 0..c.n {
            block_ids[i * NBLK + blk] = dict.get(&lattice[i * NBLK + blk]).copied().unwrap_or(OOV);
        }
    }
    drop(lattice);

    // ---- per-stage group codebooks: top-255 patterns of 9 block ids ----
    let group_pattern = |i: usize, g: usize| -> [u8; BLK_PER_GROUP] {
        let mut p = [0u8; BLK_PER_GROUP];
        for (k, item) in p.iter_mut().enumerate() {
            *item = block_ids[i * NBLK + g * BLK_PER_GROUP + k];
        }
        p
    };
    let mut e8_codes: Vec<[u8; GROUPS]> = vec![[OOV; GROUPS]; c.n];
    let mut oov_rate = [0u64; GROUPS];
    for g in 0..GROUPS {
        let mut freq: HashMap<[u8; BLK_PER_GROUP], u32> = HashMap::new();
        for i in 0..c.n {
            if c.story[i] < cut {
                *freq.entry(group_pattern(i, g)).or_default() += 1;
            }
        }
        let mut by_freq: Vec<_> = freq.into_iter().collect();
        by_freq.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        let dict: HashMap<[u8; BLK_PER_GROUP], u8> = by_freq
            .iter()
            .take(255)
            .enumerate()
            .map(|(id, (pt, _))| (*pt, id as u8))
            .collect();
        #[allow(clippy::needless_range_loop)] // i indexes parallel per-record arrays
        for i in 0..c.n {
            match dict.get(&group_pattern(i, g)) {
                Some(&id) => e8_codes[i][g] = id,
                None => {
                    e8_codes[i][g] = OOV;
                    oov_rate[g] += 1;
                }
            }
        }
    }
    println!("stage OOV rates (all records): {:?} / {}", oov_rate, c.n);

    // ---- build + grade both stores with identical machinery ----
    let ship_key = |i: usize, d: usize| shipped_codes[i][..d].to_vec();
    let e8_key = |i: usize, d: usize| e8_codes[i][..d].to_vec();
    let ship_store = build_store_from_codes(&c, cut, &ship_key);
    let e8_store = build_store_from_codes(&c, cut, &e8_key);
    for (name, store, key) in [
        (
            "shipped-geometry",
            &ship_store,
            &ship_key as &dyn Fn(usize, usize) -> Vec<u8>,
        ),
        (
            "e8-icosian-geometry",
            &e8_store,
            &e8_key as &dyn Fn(usize, usize) -> Vec<u8>,
        ),
    ] {
        let m = eval_store(&c, cut, &positions, store, key);
        println!(
            "{name:<20} top1 {:.1}% | agree {:.1}% | WB {:.4} bits | {} keys | infill-free top1 {:.1}%",
            m.top1, m.agree, m.wb_bits, m.keys, m.infill_top1
        );
    }
}
