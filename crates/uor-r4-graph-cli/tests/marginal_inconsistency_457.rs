//! #457 — is the current scoring marginal-INconsistent enough for an IPF
//! reconstruction operator to matter? (the exit-criteria NEGATIVE-branch probe).
//!
//! IPF-consistent reconstruction reconciles overlapping block marginals into the
//! max-entropy joint whose marginal on every block equals the empirical block
//! marginal. Backoff-plus-sum (the current Rule 1 scoring) is one non-iterative
//! reconstruction with no such guarantee. The gain IPF could buy is bounded by
//! how much the blocks a position actually belongs to DISAGREE: if the regions a
//! held-out position activates already concur on the next-token distribution,
//! then any reasonable combination — backoff+sum or IPF — lands in the same place
//! and IPF cannot pay. This is the cheap check that decides whether building the
//! IPF Arm B is worth it (measurement discipline: the cheapest instrument first).
//!
//! For each held-out observation this measures the disagreement among its active
//! cover regions (the "blocks" it belongs to, `binary_memberships` across
//! depths): the pairwise Jensen–Shannon divergence of their emission marginals
//! and their argmax agreement, over the union of the regions' residual supports.
//! A shuffled-region null (random regions of the same count) is the falsifier —
//! real co-activated regions should agree MORE than random ones; and the ABSOLUTE
//! level of real disagreement is the number the exit criteria asks for.
//!
//! # Measured (default cover, 5000 held-out of the 500k fixture) — hypothesis REVERSED
//!
//! Positions activate a MEAN of 7.52 regions (heavy overlap, not nested-only).
//! Over the root-mass + residual support:
//!   real co-activated regions : mean pairwise JS 0.5511 bits, argmax agreement 0.3468
//!   null random regions       : mean pairwise JS 0.6019 bits, argmax agreement 0.2611
//!
//! The falsifier passes (real regions agree more than random), so the cover's
//! co-activation carries real structure. But the ABSOLUTE disagreement is LARGE:
//! the ~7.5 blocks a position belongs to concur on the next token only ~35% of
//! the time, barely above the random floor. So the marginal inconsistency is NOT
//! too small to matter — the hypothesis that overlapping regions concur (so IPF
//! is moot) is refuted. By this cheap check, an IPF Arm B is *warranted*, not
//! ruled out: there is real disagreement for a consistency operator to reconcile.
//!
//! The caveat that keeps #457 from being a positive: #456 measured the
//! reconstruction that COMBINES these regions to be sub-unigram, and #459 that
//! k≥3 marginals are counting noise at this scale. So the disagreement is very
//! likely disagreement among near-noise residuals, which IPF would reconcile into
//! a consistent-but-still-noisy joint rather than into signal. The definitive
//! test is Arm B (IPF vs backoff+sum on held-out bits/top-1) — likely negative,
//! but not settled here. #457 stays open for it.
//!
//! `#[ignore]`d (needs the pinned fixtures). Run:
//!   R4_INCONSIST_HELD=5000 \
//!   cargo test --release -p uor-r4-graph-cli --test marginal_inconsistency_457 -- --ignored --nocapture

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::{self as score, ScoreConfig};
use uor_r4_graph_cli::cover_sweep::load_inputs;
use uor_r4_graph_compiler::induction as cover;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Deterministic xorshift64* — no `rand`, reproducible per seed.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// P(next|region) over `support`, softmaxed from root prior + region ΔE
/// residuals (Q16.16 log-prob). Tokens outside the region's residual list back
/// off to the root prior (ΔE = 0), and tokens outside the root prior take the
/// root floor.
fn region_marginal(
    emissions: &score::EmissionTables,
    region_id: usize,
    support: &[u32],
) -> Vec<f64> {
    let deltas: HashMap<u32, f64> = emissions.region_lists[region_id]
        .iter()
        .map(|(t, dq)| (*t, dq.to_logprob() as f64))
        .collect();
    let floor = emissions.root_floor.to_logprob() as f64;
    let logs: Vec<f64> = support
        .iter()
        .map(|&t| {
            let root = emissions
                .root_prior
                .get(&t)
                .map(|q| q.to_logprob() as f64)
                .unwrap_or(floor);
            root + deltas.get(&t).copied().unwrap_or(0.0)
        })
        .collect();
    let max = logs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut exps: Vec<f64> = logs.iter().map(|l| (l - max).exp()).collect();
    let z: f64 = exps.iter().sum();
    if z > 0.0 {
        for e in &mut exps {
            *e /= z;
        }
    }
    exps
}

fn argmax(dist: &[f64], support: &[u32]) -> u32 {
    support[dist
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0)]
}

/// Jensen–Shannon divergence (base 2, in [0,1]) between two distributions over
/// the same support.
fn js_divergence(p: &[f64], q: &[f64]) -> f64 {
    fn kl(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b)
            .map(|(&ai, &bi)| {
                if ai > 0.0 && bi > 0.0 {
                    ai * (ai / bi).log2()
                } else {
                    0.0
                }
            })
            .sum()
    }
    let m: Vec<f64> = p.iter().zip(q).map(|(&pi, &qi)| 0.5 * (pi + qi)).collect();
    0.5 * kl(p, &m) + 0.5 * kl(q, &m)
}

/// Support = the shared root-prior mass (`root_top`) UNION the regions' residual
/// tokens. Including the root mass is essential: two regions are both ≈ the root
/// prior and differ only on their few weak residual tokens, so a support of
/// residual tokens ALONE would exclude everywhere the regions agree and grossly
/// overstate their disagreement. This measures the full predictive distributions.
fn union_support(
    emissions: &score::EmissionTables,
    region_ids: &[usize],
    root_top: &[u32],
) -> Vec<u32> {
    let mut set: HashSet<u32> = root_top.iter().copied().collect();
    for &r in region_ids {
        for (t, _) in &emissions.region_lists[r] {
            set.insert(*t);
        }
    }
    let mut v: Vec<u32> = set.into_iter().collect();
    v.sort_unstable();
    v
}

/// Mean pairwise JS divergence + argmax-agreement fraction among a set of
/// regions' full marginals over the root-mass + residual support.
fn disagreement(
    emissions: &score::EmissionTables,
    region_ids: &[usize],
    root_top: &[u32],
) -> Option<(f64, f64)> {
    if region_ids.len() < 2 {
        return None;
    }
    let support = union_support(emissions, region_ids, root_top);
    if support.is_empty() {
        return None;
    }
    let marginals: Vec<Vec<f64>> = region_ids
        .iter()
        .map(|&r| region_marginal(emissions, r, &support))
        .collect();
    let argmaxes: Vec<u32> = marginals.iter().map(|m| argmax(m, &support)).collect();

    let mut js_sum = 0.0;
    let mut pairs = 0u64;
    for i in 0..marginals.len() {
        for j in (i + 1)..marginals.len() {
            js_sum += js_divergence(&marginals[i], &marginals[j]);
            pairs += 1;
        }
    }
    let mean_js = js_sum / pairs.max(1) as f64;

    // Argmax agreement: fraction of regions whose argmax equals the modal argmax.
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for &a in &argmaxes {
        *counts.entry(a).or_default() += 1;
    }
    let modal = counts.values().cloned().max().unwrap_or(0);
    let agree = modal as f64 / argmaxes.len() as f64;
    Some((mean_js, agree))
}

#[test]
#[ignore = "#457 marginal-inconsistency probe; needs fixtures — run with --ignored"]
fn overlapping_region_marginals_disagree_so_ipf_arm_b_is_warranted() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("marginal_inconsistency_457: fixtures absent, skipping");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load inputs");
    let config = ScoreConfig::default();

    // Default cover + its emission tables (the fitted block marginals).
    let induced = cover::induce_cover(
        &inputs.train,
        &cover::CoverConfig::default(),
        &inputs.artifact_kappa,
        &inputs.corpus_kappa,
    )
    .expect("induce cover");
    let regions = score::regions_from_cover(&induced.cover);
    let max_depth = induced.cover.max_depth;
    let vocab = u32::try_from(
        inputs.artifacts.token_codes.len() / uor_r4_core::transformerless::compiler::STAGES,
    )
    .expect("vocab");
    let emissions = score::compile_emissions(
        &inputs.corpus,
        &inputs.store,
        &regions,
        &inputs.train,
        max_depth,
        vocab,
        &config,
    );

    // region_id() must equal the region_lists index (score.rs:1612) — assert it
    // so a future re-ordering can't silently misalign the marginals.
    for (i, r) in regions.iter().enumerate() {
        assert_eq!(
            r.region_id() as usize,
            i,
            "region_id() must equal the regions/region_lists index"
        );
    }
    assert_eq!(
        regions.len(),
        emissions.region_lists.len(),
        "regions ↔ region_lists"
    );

    // The shared root-prior mass every region marginal builds on: the top-K root
    // tokens by score. Including these in the support is what makes the JS honest
    // (the regions agree on this mass; they differ only on their residuals).
    let root_top_k = env_usize("R4_INCONSIST_ROOT_TOP", 2_000);
    let root_top: Vec<u32> = {
        let mut v: Vec<(u32, i32)> = emissions
            .root_prior
            .iter()
            .map(|(t, q)| (*t, q.raw()))
            .collect();
        v.sort_by_key(|x| std::cmp::Reverse(x.1));
        v.truncate(root_top_k);
        v.into_iter().map(|(t, _)| t).collect()
    };

    let pop = runtime::derive_popcount_table();
    let held_cap = env_usize("R4_INCONSIST_HELD", 5_000);
    let held = &inputs.held_out[..held_cap.min(inputs.held_out.len())];

    let mut kernel = runtime::OpKernel::default();
    let mut real_js = 0.0;
    let mut real_agree = 0.0;
    let mut real_n = 0u64;
    let mut null_js = 0.0;
    let mut null_agree = 0.0;
    let mut null_n = 0u64;
    let mut frontier_widths: Vec<usize> = Vec::new();
    let mut rng = Rng(0x5711_2233_4455);

    for obs in held {
        // Active regions across every depth = the blocks this position belongs to.
        let mut active: Vec<usize> = Vec::new();
        for depth in 1..=max_depth {
            for (rid, _dist) in
                score::binary_memberships(&mut kernel, &pop, &regions, depth, &obs.sig)
            {
                active.push(rid as usize);
            }
        }
        active.sort_unstable();
        active.dedup();
        frontier_widths.push(active.len());
        if active.len() < 2 {
            continue;
        }

        if let Some((js, agree)) = disagreement(&emissions, &active, &root_top) {
            real_js += js;
            real_agree += agree;
            real_n += 1;
        }

        // Null: the SAME number of regions, chosen at random.
        let mut rand_regions: Vec<usize> = Vec::with_capacity(active.len());
        while rand_regions.len() < active.len() {
            let r = rng.below(regions.len());
            if !rand_regions.contains(&r) {
                rand_regions.push(r);
            }
        }
        if let Some((js, agree)) = disagreement(&emissions, &rand_regions, &root_top) {
            null_js += js;
            null_agree += agree;
            null_n += 1;
        }
    }

    let fw_mean =
        frontier_widths.iter().sum::<usize>() as f64 / frontier_widths.len().max(1) as f64;
    let fw_max = frontier_widths.iter().cloned().max().unwrap_or(0);
    let overlap_positions = frontier_widths.iter().filter(|&&w| w >= 2).count();

    println!(
        "#457 marginal inconsistency — {} held-out positions",
        held.len()
    );
    println!(
        "  active-region count: mean {:.2}, max {}, {} of {} positions activate >=2 regions",
        fw_mean,
        fw_max,
        overlap_positions,
        held.len()
    );
    println!(
        "  REAL co-activated regions : mean pairwise JS {:.4} bits, argmax agreement {:.4} (n={})",
        real_js / real_n.max(1) as f64,
        real_agree / real_n.max(1) as f64,
        real_n
    );
    println!(
        "  NULL random regions       : mean pairwise JS {:.4} bits, argmax agreement {:.4} (n={})",
        null_js / null_n.max(1) as f64,
        null_agree / null_n.max(1) as f64,
        null_n
    );

    assert!(
        real_n > 0,
        "no position activated >=2 regions — cannot measure inconsistency"
    );

    // Falsifier: real co-activated regions must agree MORE (lower JS) than random
    // regions, or the frontier structure carries no coherence and the measurement
    // is void.
    let real_mean_js = real_js / real_n as f64;
    let null_mean_js = null_js / null_n.max(1) as f64;
    assert!(
        real_mean_js < null_mean_js,
        "real co-activated regions ({:.4} JS) do not agree more than random ({:.4}) — void",
        real_mean_js,
        null_mean_js
    );
}
