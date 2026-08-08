//! #457 Arm B — the definitive test the #499 cheap-check deferred: does an
//! IPF-consistent reconstruction of the next-token distribution beat the current
//! backoff+sum on held-out bits/top-1?
//!
//! # The two reconstructions
//!
//! A held-out position belongs to several overlapping cover regions (the #499
//! probe measured a mean of 7.52). Each region `r` carries an emission marginal
//! `m_r(next)` — `softmax(root_prior + ΔE_r)` over the tokens it constrains. The
//! question #64's double-counting redesign left open is how to COMBINE the
//! marginals of the regions a position co-activates:
//!
//! - **Arm A — backoff+sum** (the deployed Rule 1 scoring, EXCT disabled):
//!   `P_A(t) ∝ exp(root_logprob(t) + Σ_r ΔE_r(t))`. The residual log-terms of
//!   every active region are added. Nothing constrains the result to be marginal-
//!   consistent with any single block.
//!
//! - **Arm B — IPF-consistent** (Reconstructability Analysis): the max-entropy
//!   distribution whose restriction to each region's support matches that
//!   region's marginal. Iterative proportional fitting / raking: start from the
//!   root prior, then cyclically rescale the working distribution on each
//!   region's residual support `S_r` to the SHAPE of `m_r` there (preserving the
//!   working mass on `S_r`), renormalising, until the per-region inconsistency
//!   stops falling. On overlapping supports over one variable this is exactly the
//!   "reconstruct the joint consistent with the blocks" operator RA prescribes,
//!   and it is a genuinely different combination rule from summing residuals.
//!
//! # Measured (default cover, 5000 held-out of the 500k fixture) — NEGATIVE
//!
//! ```text
//!   arm         bits/token   top-1
//!   root null     8.5635     0.0620   (the unigram floor)
//!   A sum        23.9196     0.0018   (naive Σ-residual — reproduces #64's collapse)
//!   B IPF         8.5995     0.0620   (max-ent consistent — lands AT the floor)
//!   per-region inconsistency (bits): A 5.3953  ->  B after IPF 0.0956
//! ```
//!
//! Two things, both decisive. (1) The naive additive backoff+sum blows up to
//! 23.92 bits / 0.18% top-1 — it reproduces the #64 double-counting collapse
//! (~0.3% top-1) almost exactly, which both validates the harness and shows why
//! the chain-telescoped redesign was needed. (2) IPF DOES its job: it drives the
//! per-region marginal inconsistency from 5.40 bits down to 0.096, i.e. it
//! reconciles the overlapping blocks into a genuinely consistent joint. But that
//! consistent joint lands EXACTLY at the unigram floor (8.60 bits, 0.0620 top-1 =
//! the null's 0.0620) — it recovers no next-token signal above the prior. The
//! overlapping regions carry nothing reconcilable; the consistency operator is
//! not the missing lever. This corroborates #456 (the region reconstruction is
//! sub-unigram) and #459 (k>=3 marginals are counting noise) from a third angle.
//!
//! Caveat, recorded for honesty: Arm A here is the NAIVE additive sum of every
//! active region's residual, not the deployed chain-telescoped/EXCT scorer — so
//! the "+15.3 bits B-over-A" is against the #64 strawman, not the shipped scorer.
//! The load-bearing number is B = floor, which needs no Arm A to interpret.
//!
//! # Pre-registered exit rule (issue #457)
//!
//! POSITIVE if Arm B lowers held-out bits/token over Arm A by at least
//! `BITS_MARGIN`, AND both arms clear the root-prior (unigram) floor (the
//! anti-vacuity control — a reconstruction that cannot beat the prior is not
//! reconstructing anything). Otherwise NEGATIVE, and the NEGATIVE-branch
//! deliverable is reported regardless: the mean per-region model-vs-empirical KL
//! `KL(m_r ‖ P_A|S_r)` — how marginal-INconsistent backoff+sum actually was, so
//! the next arm knows whether there was inconsistency for IPF to remove at all.
//!
//! # Prior (why this is very likely NEGATIVE, but must be settled with data)
//!
//! #456 measured the reconstruction that COMBINES these regions to be sub-unigram
//! (16.3 bits vs 8.7 unigram) with EXCT disabled; #459 that k≥3 marginals are
//! counting noise at D3 scale; #499 that the ~7.5 co-activated blocks concur only
//! ~35% of the time — disagreement among near-noise residuals. IPF would reconcile
//! that into a consistent-but-still-noisy joint. The number below decides it.
//!
//! `#[ignore]`d (needs the pinned fixtures). Run:
//!   R4_RECON_HELD=5000 \
//!   cargo test --release -p uor-r4-graph-cli --test marginal_reconstruction_457 -- --ignored --nocapture

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::{self as score, ScoreConfig};
use uor_r4_graph_cli::cover_sweep::load_inputs;
use uor_r4_graph_compiler::induction as cover;

/// IPF passes over the region constraints per position. Convergence on a single
/// shared variable with overlapping supports is fast; the harness reports the
/// residual inconsistency so the cap can be seen to be enough.
const IPF_PASSES: usize = 32;
/// Pre-registered bits/token margin for Arm B to be a POSITIVE over Arm A.
const BITS_MARGIN: f64 = 0.05;

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

/// Natural-log-prob of one token under the root prior, backing off to the floor.
fn root_logprob(emissions: &score::EmissionTables, floor: f64, t: u32) -> f64 {
    emissions
        .root_prior
        .get(&t)
        .map(|q| q.to_logprob() as f64)
        .unwrap_or(floor)
}

fn softmax(logs: &[f64]) -> Vec<f64> {
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

/// Arm A: backoff + sum of every active region's residual, softmaxed over the
/// support. This is the deployed Rule 1 combination with EXCT disabled.
fn backoff_sum(
    emissions: &score::EmissionTables,
    floor: f64,
    active: &[usize],
    support: &[u32],
) -> Vec<f64> {
    let deltas: Vec<HashMap<u32, f64>> = active
        .iter()
        .map(|&r| {
            emissions.region_lists[r]
                .iter()
                .map(|(t, dq)| (*t, dq.to_logprob() as f64))
                .collect()
        })
        .collect();
    let logs: Vec<f64> = support
        .iter()
        .map(|&t| {
            let mut l = root_logprob(emissions, floor, t);
            for d in &deltas {
                l += d.get(&t).copied().unwrap_or(0.0);
            }
            l
        })
        .collect();
    softmax(&logs)
}

/// One region's marginal `m_r` over the support (softmax(root + ΔE_r)) and the
/// index set of the support tokens the region actually constrains (its residual
/// support, where a raking step acts).
fn region_marginal_and_support(
    emissions: &score::EmissionTables,
    floor: f64,
    region_id: usize,
    support: &[u32],
) -> (Vec<f64>, Vec<usize>) {
    let deltas: HashMap<u32, f64> = emissions.region_lists[region_id]
        .iter()
        .map(|(t, dq)| (*t, dq.to_logprob() as f64))
        .collect();
    let logs: Vec<f64> = support
        .iter()
        .map(|&t| root_logprob(emissions, floor, t) + deltas.get(&t).copied().unwrap_or(0.0))
        .collect();
    let idx: Vec<usize> = support
        .iter()
        .enumerate()
        .filter(|(_, t)| deltas.contains_key(t))
        .map(|(i, _)| i)
        .collect();
    (softmax(&logs), idx)
}

/// Arm B: IPF / raking to marginal consistency. Rakes the working distribution
/// onto each region's residual support to match that region's marginal SHAPE
/// there, preserving the working mass on the support; iterates over regions.
/// Returns the reconstructed distribution and the mean residual inconsistency
/// (post-fit KL of each region's target vs the working restriction) so the
/// convergence and the negative-branch number are both visible.
fn ipf_reconstruct(
    emissions: &score::EmissionTables,
    floor: f64,
    active: &[usize],
    support: &[u32],
) -> (Vec<f64>, f64) {
    let targets: Vec<(Vec<f64>, Vec<usize>)> = active
        .iter()
        .map(|&r| region_marginal_and_support(emissions, floor, r, support))
        .collect();
    // Start from the root prior over the support.
    let root_logs: Vec<f64> = support
        .iter()
        .map(|&t| root_logprob(emissions, floor, t))
        .collect();
    let mut q = softmax(&root_logs);

    for _ in 0..IPF_PASSES {
        for (target, idx) in &targets {
            if idx.is_empty() {
                continue;
            }
            // Working mass on this region's support, and the target's mass there.
            let q_mass: f64 = idx.iter().map(|&i| q[i]).sum();
            let t_mass: f64 = idx.iter().map(|&i| target[i]).sum();
            if q_mass <= 0.0 || t_mass <= 0.0 {
                continue;
            }
            // Rake q on S_r to the target's SHAPE (target[i]/t_mass), scaled back
            // to q's current mass q_mass on S_r — the IPF multiplicative update.
            for &i in idx {
                q[i] = q_mass * (target[i] / t_mass);
            }
            let z: f64 = q.iter().sum();
            if z > 0.0 {
                for v in &mut q {
                    *v /= z;
                }
            }
        }
    }

    // Residual inconsistency: mean KL(target_r|S_r ‖ q|S_r) after fitting.
    let mut kl_sum = 0.0;
    let mut kl_n = 0u64;
    for (target, idx) in &targets {
        if idx.is_empty() {
            continue;
        }
        let q_mass: f64 = idx.iter().map(|&i| q[i]).sum();
        let t_mass: f64 = idx.iter().map(|&i| target[i]).sum();
        if q_mass <= 0.0 || t_mass <= 0.0 {
            continue;
        }
        let mut kl = 0.0;
        for &i in idx {
            let p = target[i] / t_mass;
            let g = q[i] / q_mass;
            if p > 0.0 && g > 0.0 {
                kl += p * (p / g).log2();
            }
        }
        kl_sum += kl;
        kl_n += 1;
    }
    (q, kl_sum / kl_n.max(1) as f64)
}

/// Mean KL(m_r|S_r ‖ P|S_r) of the deployed backoff+sum distribution against
/// each region's marginal — how marginal-INconsistent Arm A was (the #457
/// negative-branch deliverable: was there inconsistency for IPF to remove?).
fn arm_a_inconsistency(
    emissions: &score::EmissionTables,
    floor: f64,
    active: &[usize],
    support: &[u32],
    p_a: &[f64],
) -> Option<f64> {
    let mut kl_sum = 0.0;
    let mut kl_n = 0u64;
    for &r in active {
        let (target, idx) = region_marginal_and_support(emissions, floor, r, support);
        if idx.is_empty() {
            continue;
        }
        let p_mass: f64 = idx.iter().map(|&i| p_a[i]).sum();
        let t_mass: f64 = idx.iter().map(|&i| target[i]).sum();
        if p_mass <= 0.0 || t_mass <= 0.0 {
            continue;
        }
        let mut kl = 0.0;
        for &i in &idx {
            let p = target[i] / t_mass;
            let g = p_a[i] / p_mass;
            if p > 0.0 && g > 0.0 {
                kl += p * (p / g).log2();
            }
        }
        kl_sum += kl;
        kl_n += 1;
    }
    if kl_n == 0 {
        None
    } else {
        Some(kl_sum / kl_n as f64)
    }
}

fn bits_of(dist: &[f64], support: &[u32], next: u32) -> f64 {
    match support.iter().position(|&t| t == next) {
        Some(i) if dist[i] > 0.0 => -dist[i].log2(),
        _ => 24.0, // vocab-scale miss ceiling (2^24 >> vocab), same role as a floor
    }
}

fn argmax_token(dist: &[f64], support: &[u32]) -> u32 {
    support[dist
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0)]
}

#[test]
#[ignore = "#457 Arm B reconstruction test; needs fixtures — run with --ignored"]
fn ipf_vs_backoff_sum_on_held_out() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("marginal_reconstruction_457: fixtures absent, skipping");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load inputs");
    let config = ScoreConfig::default();
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
    for (i, r) in regions.iter().enumerate() {
        assert_eq!(
            r.region_id() as usize,
            i,
            "region_id() == region_lists index"
        );
    }
    let floor = emissions.root_floor.to_logprob() as f64;

    // Shared root mass in every support (the #499 rule: the regions agree here and
    // differ on residuals; excluding it overstates disagreement and starves bits).
    let root_top_k = env_usize("R4_RECON_ROOT_TOP", 2_000);
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
    let held_cap = env_usize("R4_RECON_HELD", 5_000);
    let held = &inputs.held_out[..held_cap.min(inputs.held_out.len())];
    let mut kernel = runtime::OpKernel::default();

    let (mut bits_a, mut bits_b, mut bits_null) = (0.0f64, 0.0f64, 0.0f64);
    let (mut hit_a, mut hit_b, mut hit_null) = (0u64, 0u64, 0u64);
    let (mut incon_a, mut incon_b) = (0.0f64, 0.0f64);
    let (mut moved, mut graded, mut multi) = (0.0f64, 0u64, 0u64);

    for obs in held {
        let mut active: Vec<usize> = Vec::new();
        for depth in 1..=max_depth {
            for (rid, _d) in score::binary_memberships(&mut kernel, &pop, &regions, depth, &obs.sig)
            {
                active.push(rid as usize);
            }
        }
        active.sort_unstable();
        active.dedup();

        // Support = shared root mass ∪ active residual supports ∪ the true next
        // token (so bits are always defined, never silently a miss-ceiling).
        let mut set: HashSet<u32> = root_top.iter().copied().collect();
        for &r in &active {
            for (t, _) in &emissions.region_lists[r] {
                set.insert(*t);
            }
        }
        set.insert(obs.next);
        let mut support: Vec<u32> = set.into_iter().collect();
        support.sort_unstable();

        // Null: root prior alone — the unigram floor, the anti-vacuity control.
        let null = softmax(
            &support
                .iter()
                .map(|&t| root_logprob(&emissions, floor, t))
                .collect::<Vec<_>>(),
        );
        bits_null += bits_of(&null, &support, obs.next);
        if argmax_token(&null, &support) == obs.next {
            hit_null += 1;
        }

        let p_a = backoff_sum(&emissions, floor, &active, &support);
        bits_a += bits_of(&p_a, &support, obs.next);
        if argmax_token(&p_a, &support) == obs.next {
            hit_a += 1;
        }
        if let Some(kl) = arm_a_inconsistency(&emissions, floor, &active, &support, &p_a) {
            incon_a += kl;
        }

        let (p_b, resid) = ipf_reconstruct(&emissions, floor, &active, &support);
        bits_b += bits_of(&p_b, &support, obs.next);
        if argmax_token(&p_b, &support) == obs.next {
            hit_b += 1;
        }
        incon_b += resid;

        // L1 distance A→B, so a "no-op IPF" cannot masquerade as agreement.
        let l1: f64 = p_a.iter().zip(&p_b).map(|(a, b)| (a - b).abs()).sum();
        moved += l1;
        if active.len() >= 2 {
            multi += 1;
        }
        graded += 1;
    }

    let n = graded.max(1) as f64;
    println!("#457 Arm B — IPF-consistent reconstruction vs backoff+sum");
    println!(
        "  graded {graded} held-out positions, {multi} with >=2 active regions, \
         mean A->B L1 move {:.4}",
        moved / n
    );
    println!(
        "  arm         bits/token   top-1\n  \
         root null   {:>8.4}   {:.4}\n  \
         A sum       {:>8.4}   {:.4}\n  \
         B IPF       {:>8.4}   {:.4}",
        bits_null / n,
        hit_null as f64 / n,
        bits_a / n,
        hit_a as f64 / n,
        bits_b / n,
        hit_b as f64 / n,
    );
    println!(
        "  marginal inconsistency (mean per-region KL to block marginal, bits): \
         A backoff+sum {:.4} -> B after IPF {:.4}",
        incon_a / n,
        incon_b / n,
    );

    let (mbits_a, mbits_b, mbits_null) = (bits_a / n, bits_b / n, bits_null / n);
    let delta = mbits_a - mbits_b; // positive = B improves (fewer bits)

    // Anti-vacuity (Rule 5): the reconstructions must be doing something. If IPF
    // never moved the distribution, say so rather than passing on an equal tie.
    assert!(
        moved / n > 1e-9,
        "IPF never changed the distribution (mean L1 {:.2e}) — vacuous",
        moved / n
    );

    println!("\n  ==== verdict (pre-registered #457 exit rule) ====");
    println!(
        "  Arm B moves bits/token by {:+.4} vs Arm A [margin {:.2}]; floor is {:.4}",
        delta, BITS_MARGIN, mbits_null
    );
    if delta >= BITS_MARGIN && mbits_b < mbits_null && mbits_a < mbits_null {
        println!(
            "  POSITIVE: IPF consistency lowers held-out bits by {:+.4} over backoff+sum, both \
             below the unigram floor. File the compiler-side follow-up to make the consistency \
             operator the scoring semantics (EMIT residual semantics, RFC amendment).",
            delta
        );
    } else {
        let why = if mbits_a >= mbits_null {
            "both reconstructions are ABOVE the unigram floor — the combined regions carry no \
             next-token signal to reconcile (consistent with #456's sub-unigram reconstruction \
             and #459's counting-noise k>=3)"
        } else {
            "backoff+sum already clears the floor and IPF does not improve it by the margin — \
             the marginal inconsistency is real but reconciling it does not recover signal, \
             because it is disagreement among near-noise residuals (#499)"
        };
        println!("  NEGATIVE against the exit rule: {why}.");
        println!(
            "  Recorded for retrace: A inconsistency {:.4} bits collapses to {:.4} after IPF, so \
             IPF DID enforce consistency; it simply bought no held-out bits. The consistency \
             operator is not the missing lever on this cover at D3 scale.",
            incon_a / n,
            incon_b / n
        );
    }
}
