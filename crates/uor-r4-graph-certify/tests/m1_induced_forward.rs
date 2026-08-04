//! #399 M1 (substrate scope): forward-anchor conditioning over the INDUCED
//! cover — the geometry that measurably works on natural text (#393 record).
//!
//! Arms:
//! 1. `induced-causal`: region-path-keyed store (cover paths as graded keys,
//!    observed-continuation evidence) — the harness-protocol reproduction of
//!    the score pipeline's substrate; exit rule: within ~2pp of 36.5%.
//! 2. `induced-fwd`: (distance, anchor-emitting region at depth P) → target
//!    token table over induced regions; anchor resolved to emitting regions
//!    via train emission counts (top-k weighted mixture).
//! 3. `fuse:causal×fwd`: log-domain product per the measured composition law.
//!
//! Held-out routing: cosine descent down the region tree using the
//! compiler-side f32 prototypes (root → best child per depth).
//!
//! Certifier-side instrumentation only.
//!
//! Run (natural stack):
//!   R4_CORPUS_META=/tmp/wiki-obs/state.bin R4_CORPUS_RECS=/tmp/wiki-obs/merged.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl \
//!   cargo test --release -p uor-r4-graph-certify --test m1_induced_forward -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction::{self, CoverConfig};

const STRIDE: usize = 4;
const M1_P: usize = 2; // region-path prefix depth for the forward table
const M1_TOPR: usize = 4;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
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

fn argmax64(dist: &BTreeMap<u32, u64>) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
}

fn smoothed_ln(dist: &BTreeMap<u32, u64>, total: u64, t: u32) -> f64 {
    let c = dist.get(&t).copied().unwrap_or(0) as f64;
    ((c + 0.5) / (total as f64 + 16_000.0)).ln()
}

fn fuse2(a: Option<&BTreeMap<u32, u64>>, b: Option<&BTreeMap<u32, u64>>) -> Option<u32> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => argmax64(x),
        (Some(x), Some(y)) => {
            let (xt, yt): (u64, u64) = (x.values().sum(), y.values().sum());
            x.keys()
                .chain(y.keys())
                .map(|&t| (t, smoothed_ln(x, xt, t) + smoothed_ln(y, yt, t)))
                .max_by(|p, q| p.1.partial_cmp(&q.1).unwrap().then(q.0.cmp(&p.0)))
                .map(|(t, _)| t)
        }
    }
}

/// Cosine descent: route a unit vector down the region tree, returning the
/// region-id path (depth 1..=max reached).
fn route_path(regions: &[induction::CoverRegion], depth1: &[u32], vector: &[f32]) -> Vec<u32> {
    let dot = |rid: u32| -> f32 {
        let p = &regions[rid as usize].prototype;
        p.iter().zip(vector).map(|(a, b)| a * b).sum()
    };
    let mut path = Vec::new();
    let mut candidates: Vec<u32> = depth1.to_vec();
    while !candidates.is_empty() {
        let best = candidates
            .iter()
            .copied()
            .max_by(|&a, &b| dot(a).partial_cmp(&dot(b)).unwrap())
            .unwrap();
        path.push(best);
        candidates = regions[best as usize].children.clone();
    }
    path
}

#[test]
#[ignore = "M1 measurement; run explicitly with --ignored"]
fn m1_induced_forward() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let art = compiler::load_artifacts_from(&art_path).expect("artifacts");
    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = match std::env::var("R4_STORIES") {
        Ok(path) => {
            let text = std::fs::read_to_string(&path).expect("stories.jsonl");
            let mut v = vec![true; c.stories as usize];
            for line in text.lines() {
                let Some(story_pos) = line.find("\"story\":") else {
                    continue;
                };
                let story: usize = line[story_pos + 8..]
                    .split(',')
                    .next()
                    .and_then(|x| x.trim().parse().ok())
                    .expect("story id");
                if story < v.len() {
                    v[story] = !line.contains("\"partition\":\"HeldOut\"");
                }
            }
            v
        }
        Err(_) => (0..c.stories).map(|sid| sid < u64::from(cut)).collect(),
    };
    let is_constr = |sid: u32| constr[sid as usize];
    let positions = story_positions(&c);
    println!(
        "M1 induced-forward (#399): {} records, {} construction stories",
        c.n,
        constr.iter().filter(|&&b| b).count()
    );

    let train_pos: Vec<usize> = (0..c.n).filter(|&i| is_constr(c.story[i])).collect();
    let held_pos: Vec<usize> = (0..c.n).filter(|&i| !is_constr(c.story[i])).collect();

    let train_obs = induction::build_observations_with_threads(&art, &c, &train_pos, 2)
        .expect("train observations");
    println!("train observations built: {}", train_obs.len());

    let config = CoverConfig {
        threads: 2,
        ..CoverConfig::default()
    };
    let induced = induction::induce_cover(&train_obs, &config, "m1-artifact", "m1-corpus")
        .expect("cover induction");
    let cover = &induced.cover;
    println!(
        "cover induced: {} regions, max depth {}",
        cover.regions.len(),
        cover.max_depth
    );

    let depth1 = cover.regions_at_depth(1);

    // ---- causal region-path store (observed evidence) ----
    let mut store: Vec<BTreeMap<Vec<u32>, BTreeMap<u32, u64>>> =
        (0..=cover.max_depth).map(|_| BTreeMap::new()).collect();
    for (obs_idx, &i) in train_pos.iter().enumerate() {
        let path = &cover.paths[obs_idx];
        for d in 0..=path.len() {
            *store[d]
                .entry(path[..d].to_vec())
                .or_default()
                .entry(c.next[i])
                .or_default() += 1;
        }
    }

    // ---- forward table over induced regions ----
    // emit_index: token -> depth-P region prefix (as path vec) -> count
    let mut emit_index: BTreeMap<u32, BTreeMap<Vec<u32>, u32>> = BTreeMap::new();
    let mut fwd_walk: BTreeMap<(usize, Vec<u32>), BTreeMap<u32, u64>> = BTreeMap::new();
    for (obs_idx, &j) in train_pos.iter().enumerate() {
        let path = &cover.paths[obs_idx];
        let prefix = path[..M1_P.min(path.len())].to_vec();
        *emit_index
            .entry(c.next[j])
            .or_default()
            .entry(prefix.clone())
            .or_default() += 1;
        let emit_pos = positions[j] + 1;
        if emit_pos.is_multiple_of(STRIDE) {
            for d in 1..STRIDE {
                if j >= d && c.story[j - d] == c.story[j] {
                    *fwd_walk
                        .entry((d, prefix.clone()))
                        .or_default()
                        .entry(c.next[j - d])
                        .or_default() += 1;
                }
            }
        }
    }

    // ---- held-out routing + eval ----
    let held_obs =
        induction::build_observations_with_threads(&art, &c, &held_pos, 2).expect("held obs");
    let mut unigram: BTreeMap<u32, u64> = BTreeMap::new();
    for &i in &train_pos {
        *unigram.entry(c.next[i]).or_default() += 1;
    }
    let unigram_pred = argmax64(&unigram);

    let (mut n_all, mut hit_causal_all) = (0u64, 0u64);
    let (mut n_free, mut hit_causal, mut hit_fwd, mut hit_fuse, mut hit_uni) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    for (obs_idx, &i) in held_pos.iter().enumerate() {
        let vector = &held_obs[obs_idx].vector;
        let path = route_path(&cover.regions, &depth1, vector);
        // causal: deepest populated path prefix
        let mut causal_dist: Option<&BTreeMap<u32, u64>> = None;
        for d in (0..=path.len().min(cover.max_depth)).rev() {
            if let Some(dist) = store[d].get(&path[..d]) {
                causal_dist = Some(dist);
                break;
            }
        }
        let causal_pred = causal_dist.and_then(argmax64).or(unigram_pred);
        n_all += 1;
        if causal_pred == Some(c.next[i]) {
            hit_causal_all += 1;
        }

        let target_pos = positions[i] + 1;
        if target_pos.is_multiple_of(STRIDE) {
            continue; // anchors are given in the infill protocol
        }
        n_free += 1;
        let truth = c.next[i];
        if causal_pred == Some(truth) {
            hit_causal += 1;
        }
        if unigram_pred == Some(truth) {
            hit_uni += 1;
        }

        // forward: next anchor token -> emitting region mixture -> walk table
        let next_anchor_pos = target_pos.next_multiple_of(STRIDE);
        let lookahead = next_anchor_pos - target_pos;
        let j = i + lookahead;
        let fwd_dist: Option<BTreeMap<u32, u64>> = if j < c.n && c.story[j] == c.story[i] {
            emit_index.get(&c.next[j]).map(|regions_of| {
                let mut top: Vec<(&Vec<u32>, &u32)> = regions_of.iter().collect();
                top.sort_by_key(|(_, &cnt)| std::cmp::Reverse(cnt));
                let mut mix: BTreeMap<u32, u64> = BTreeMap::new();
                for (prefix, &rcnt) in top.into_iter().take(M1_TOPR) {
                    if let Some(dist) = fwd_walk.get(&(lookahead, prefix.clone())) {
                        let total: u64 = dist.values().sum();
                        for (&t, &cnt) in dist {
                            *mix.entry(t).or_default() +=
                                (cnt * u64::from(rcnt) * 1000) / total.max(1);
                        }
                    }
                }
                mix
            })
        } else {
            None
        };
        let fwd_ref = fwd_dist.as_ref().filter(|m| !m.is_empty());
        if fwd_ref.and_then(argmax64).or(unigram_pred) == Some(truth) {
            hit_fwd += 1;
        }
        if fuse2(causal_dist, fwd_ref).or(unigram_pred) == Some(truth) {
            hit_fuse += 1;
        }
    }
    let pct = |h: u64, t: u64| 100.0 * h as f64 / t.max(1) as f64;
    println!(
        "induced-causal ALL-positions top1 {:.1}% (n={n_all}) [exit rule: ~36.5%]",
        pct(hit_causal_all, n_all)
    );
    println!(
        "free targets (n={n_free}): unigram {:.1}% | induced-causal {:.1}% | induced-fwd {:.1}% | fuse {:.1}%",
        pct(hit_uni, n_free),
        pct(hit_causal, n_free),
        pct(hit_fwd, n_free),
        pct(hit_fuse, n_free)
    );
}
