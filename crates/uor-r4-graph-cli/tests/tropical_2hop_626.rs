//! #626 — tropical (max-plus) multi-hop route composition over the compiled
//! forward-transition graph: does a token-dependent 2-hop max-plus term carry
//! held-out next-token signal the deployed configuration leaves on the table?
//!
//! # Why this seat is empty
//!
//! E_f edges are compiled, quantized, and shipped in every artifact, but the
//! deployed scorer's transition offset is token-independent and therefore
//! argmax-neutral (score_runtime.rs "f_emissions" note) — forward transitions
//! contribute exactly zero to serving predictions today. The tropical arm makes
//! the term token-DEPENDENT by composing routes through destination-region
//! emission residuals (route-then-emit):
//!
//! ```text
//!   trop1(t) = max over u∈A, edge u→w        of  wq(u→w)             + ΔE_w(t)
//!   trop2(t) = max over u∈A, paths u→v→w     of  wq(u→v) + wq(v→w)  + ΔE_w(t)
//!   S(t)     = root(t) + trop(t)      (ΔE absent ⇒ 0; no route ⇒ root alone)
//! ```
//!
//! `⊗` is integer add on raw Q16.16 (i64 — saturation unreachable), `⊕` is
//! max with a -inf identity, and the argmax is witness-bearing by construction
//! (the selection retains WHICH route won — a ring accumulation cannot).
//!
//! # Pre-registered exit rule (issue #626, first comment — binding)
//!
//! POSITIVE iff top-1(2H) >= top-1(1H) + 0.02 absolute AND bits(2H) <= bits(1H)
//! AND top-1(2H) > top-1(NULL). The NULL row must reproduce the #457 unigram
//! floor (0.0620 / 8.5635 bits) on these fixtures or the instrument is broken.
//! Instrument null N (deranged edge destinations) must NOT beat 1H or 2H.
//! Secondary, reported either way: 1H vs NULL — whether the E_f seat carries
//! any one-hop signal at all. NEGATIVE ⇒ close #626 with the table and register
//! the dormant lane in model/ledger.toml.
//!
//! # Semantics parity (binding)
//!
//! `max_plus_fold` — the fold both arms use — must agree exactly with
//! `uor_matmul_core::dot_tropical_ref` (dev-dependency, rev-pinned) including
//! -inf absorption and empty folds. `tropical_parity_with_uor_matmul` below
//! runs un-ignored in every `cargo test`.
//!
//! `#[ignore]`d (needs the pinned fixtures). Run:
//!   R4_TROP_HELD=5000 \
//!   cargo test --release -p uor-r4-graph-cli --test tropical_2hop_626 -- --ignored --nocapture

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::{self as score, ScoreConfig};
use uor_r4_graph_cli::cover_sweep::load_inputs;
use uor_r4_graph_compiler::induction as cover;

/// The tropical semiring zero: no route. `None` is -inf; `Some(q)` is a finite
/// raw-Q16.16 route score.
type Trop = Option<i64>;

/// `⊕` over one offered value: `acc := acc ⊕ v`. Max with -inf identity.
fn offer(acc: &mut Trop, v: i64) {
    match acc {
        Some(a) if *a >= v => {}
        _ => *acc = Some(v),
    }
}

/// The max-plus fold: `⊕_i (a_i ⊗ w_i)` with `⊗` = add, `-inf` absorbing.
/// This is the exact fold the arms use, kept free-standing so the parity test
/// below can pin it to `uor_matmul_core::dot_tropical_ref`.
fn max_plus_fold(a: &[Trop], w: &[Trop]) -> Trop {
    let k = a.len().min(w.len());
    let mut acc: Trop = None;
    for i in 0..k {
        if let (Some(x), Some(y)) = (a[i], w[i]) {
            offer(&mut acc, x + y);
        }
    }
    acc
}

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

fn bits_of(dist: &[f64], support: &[u32], next: u32) -> f64 {
    match support.iter().position(|&t| t == next) {
        Some(i) if dist[i] > 0.0 => -dist[i].log2(),
        _ => 24.0, // vocab-scale miss ceiling, same convention as the #457 harness
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

/// A composed route: destination REGION index, raw-Q16.16 path score, and the
/// witness the tropical selection retains (source region; mid region for 2-hop).
struct Route {
    dst_region: usize,
    path_q: i64,
    witness_src: usize,
    witness_mid: Option<usize>,
}

/// Forward index over compiled edges: node id (region + 1) → (dst node, raw q).
fn forward_index(edges: &[(u32, u32, i64)]) -> HashMap<u32, Vec<(u32, i64)>> {
    let mut fwd: HashMap<u32, Vec<(u32, i64)>> = HashMap::new();
    for &(src, dst, q) in edges {
        fwd.entry(src).or_default().push((dst, q));
    }
    fwd
}

/// 1-hop routes from the active regions.
fn routes_1hop(
    active: &[usize],
    fwd: &HashMap<u32, Vec<(u32, i64)>>,
    n_regions: usize,
) -> Vec<Route> {
    let mut out = Vec::new();
    for &u in active {
        let src_node = u as u32 + 1;
        if let Some(list) = fwd.get(&src_node) {
            for &(dst_node, q) in list {
                let w = dst_node as usize;
                if w >= 1 && w - 1 < n_regions {
                    out.push(Route {
                        dst_region: w - 1,
                        path_q: q,
                        witness_src: u,
                        witness_mid: None,
                    });
                }
            }
        }
    }
    out
}

/// 2-hop routes: `⊗` composes the two edge scores; per (src, dst) the caller's
/// per-token `⊕` keeps only the best, so no dedup is needed here.
fn routes_2hop(
    active: &[usize],
    fwd: &HashMap<u32, Vec<(u32, i64)>>,
    n_regions: usize,
) -> Vec<Route> {
    let mut out = Vec::new();
    for &u in active {
        let src_node = u as u32 + 1;
        if let Some(first) = fwd.get(&src_node) {
            for &(mid_node, q1) in first {
                if let Some(second) = fwd.get(&mid_node) {
                    for &(dst_node, q2) in second {
                        let w = dst_node as usize;
                        let m = mid_node as usize;
                        if w >= 1 && w - 1 < n_regions && m >= 1 {
                            out.push(Route {
                                dst_region: w - 1,
                                path_q: q1 + q2,
                                witness_src: u,
                                witness_mid: Some(m - 1),
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

/// Per-token tropical term over the support: `trop(t) = ⊕_routes (path ⊗ ΔE)`,
/// with ΔE contributions from the route's destination-region residual list and
/// 0 for tokens the destination does not constrain. Returns the term per
/// support token (raw Q16.16, `None` = no route at all) and, for the winning
/// contribution per token, whether a residual-constrained route won.
fn tropical_term(
    routes: &[Route],
    delta_maps: &[HashMap<u32, i64>],
    max_pos_delta: &[i64],
    support: &[u32],
) -> (Vec<Trop>, Vec<bool>, Vec<Option<usize>>) {
    // Routes sorted by path desc so the plain-route bound admits early exit.
    // The global max positive ΔE bounds what any remaining route could add.
    let global_max_pos: i64 = routes
        .iter()
        .map(|r| max_pos_delta[r.dst_region].max(0))
        .max()
        .unwrap_or(0);
    let mut order: Vec<usize> = (0..routes.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(routes[i].path_q));

    let mut term: Vec<Trop> = vec![None; support.len()];
    let mut via_delta: Vec<bool> = vec![false; support.len()];
    let mut winner: Vec<Option<usize>> = vec![None; support.len()];
    for (si, &t) in support.iter().enumerate() {
        let mut best: Trop = None;
        for &ri in &order {
            let r = &routes[ri];
            // Upper bound for this and all remaining (smaller-path) routes.
            if let Some(b) = best
                && r.path_q + global_max_pos <= b
            {
                break;
            }
            let (contribution, delta_hit) = match delta_maps[r.dst_region].get(&t) {
                Some(&dq) => (r.path_q + dq, true),
                None => (r.path_q, false),
            };
            let improved = match best {
                Some(b) => contribution > b,
                None => true,
            };
            if improved {
                best = Some(contribution);
                via_delta[si] = delta_hit;
                winner[si] = Some(ri);
            }
        }
        term[si] = best;
    }
    (term, via_delta, winner)
}

/// Arm distribution: softmax over `root(t) + trop(t)` (raw Q16.16 → ln via the
/// ScoreQ scale), falling back to root alone where no route exists.
fn arm_distribution(
    emissions: &score::EmissionTables,
    floor: f64,
    support: &[u32],
    term: &[Trop],
) -> Vec<f64> {
    const SCALE: f64 = 65536.0;
    let logs: Vec<f64> = support
        .iter()
        .zip(term)
        .map(|(&t, trop)| {
            let base = root_logprob(emissions, floor, t);
            match trop {
                Some(q) => base + (*q as f64) / SCALE,
                None => base,
            }
        })
        .collect();
    softmax(&logs)
}

/// Deterministic instrument null: derange edge destinations by rotating the
/// dst column of the (already canonically sorted) edge list by ⌊len/2⌋. No RNG.
fn derange_edges(edges: &[(u32, u32, i64)]) -> Vec<(u32, u32, i64)> {
    let n = edges.len();
    if n < 2 {
        return edges.to_vec();
    }
    let dsts: Vec<u32> = edges.iter().map(|e| e.1).collect();
    edges
        .iter()
        .enumerate()
        .map(|(i, &(src, _, q))| (src, dsts[(i + n / 2) % n], q))
        .collect()
}

/// Parity with the ecosystem tropical reference (pre-registered as binding):
/// `max_plus_fold` must agree exactly with `uor_matmul_core::dot_tropical_ref`
/// on finite values, -inf absorption, empty folds, and length mismatch
/// (truncation to the shorter operand ≡ -inf padding, their CK-17/A-6).
#[test]
fn tropical_parity_with_uor_matmul() {
    use uor_matmul_core::{Trop as MTrop, as_alphabet_tropical, dot_tropical_ref};

    fn ours(a: &[Option<i32>], w: &[Option<i32>]) -> Option<i64> {
        let at: Vec<Trop> = a.iter().map(|x| x.map(i64::from)).collect();
        let wt: Vec<Trop> = w.iter().map(|x| x.map(i64::from)).collect();
        max_plus_fold(&at, &wt)
    }
    fn theirs(a: &[Option<i32>], w: &[Option<i32>]) -> Option<i64> {
        let at: Vec<MTrop<i32>> = a
            .iter()
            .map(|x| x.map(MTrop::finite).unwrap_or(MTrop::NEG_INF))
            .collect();
        let wt: Vec<MTrop<i32>> = w
            .iter()
            .map(|x| x.map(MTrop::finite).unwrap_or(MTrop::NEG_INF))
            .collect();
        dot_tropical_ref(as_alphabet_tropical(&at), as_alphabet_tropical(&wt))
            .get()
            .map(|v| i64::try_from(v).expect("i32 sums fit i64"))
    }

    // Deterministic corpus: finite mixes, -inf in every slot pattern, empty,
    // singleton, length mismatch, extreme magnitudes, and an exhaustive small
    // grid. No RNG anywhere (repo determinism discipline).
    type Vec32 = Vec<Option<i32>>;
    let inf: Option<i32> = None;
    let cases: Vec<(Vec32, Vec32)> = vec![
        (vec![], vec![]),
        (vec![Some(3)], vec![]),
        (vec![Some(3)], vec![Some(4)]),
        (vec![inf], vec![Some(4)]),
        (vec![Some(3)], vec![inf]),
        (vec![inf, inf], vec![inf, inf]),
        (
            vec![Some(1), Some(-2), Some(3), inf],
            vec![Some(5), Some(6), inf, Some(8)],
        ),
        (
            vec![Some(i32::MAX), Some(i32::MIN + 1)],
            vec![Some(i32::MAX), Some(i32::MIN + 1)],
        ),
        // Length mismatch: their fold truncates to the shorter operand, which
        // must equal ours (and equal -inf padding of the shorter side).
        (vec![Some(7), Some(8), Some(9)], vec![Some(1), Some(2)]),
    ];
    for (a, w) in &cases {
        assert_eq!(ours(a, w), theirs(a, w), "parity break on a={a:?} w={w:?}");
    }
    // Exhaustive small grid over {-2,-1,0,1,2,-inf}^2 per slot, length 2.
    let vals: [Option<i32>; 6] = [Some(-2), Some(-1), Some(0), Some(1), Some(2), inf];
    for &a0 in &vals {
        for &a1 in &vals {
            for &w0 in &vals {
                for &w1 in &vals {
                    let a = vec![a0, a1];
                    let w = vec![w0, w1];
                    assert_eq!(ours(&a, &w), theirs(&a, &w), "grid a={a:?} w={w:?}");
                }
            }
        }
    }
    // Truncation ≡ -inf padding on their side too (CK-17/A-6 restated here).
    let long = vec![Some(7), Some(8), Some(9)];
    let short = vec![Some(1), Some(2)];
    let padded = vec![Some(1), Some(2), inf];
    assert_eq!(theirs(&long, &short), theirs(&long, &padded));
}

#[test]
#[ignore = "#626 tropical route-composition experiment; needs fixtures — run with --ignored"]
fn tropical_2hop_vs_1hop_on_held_out() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("tropical_2hop_626: fixtures absent, skipping");
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
    let floor = emissions.root_floor.to_logprob() as f64;

    // Forward transitions — the same compile the score pipeline's stage 4 runs.
    let (t_edges, _quant) = score::compile_transitions_with_quantization(
        &inputs.corpus,
        &regions,
        &inputs.train,
        max_depth,
        score::DEFAULT_TRANSITION_OUT_DEGREE,
    );
    let edges: Vec<(u32, u32, i64)> = t_edges
        .iter()
        .map(|e| (e.src, e.dst, e.score.raw() as i64))
        .collect();
    assert!(
        !edges.is_empty(),
        "no forward transitions compiled — vacuous"
    );
    let fwd = forward_index(&edges);
    let fwd_null = forward_index(&derange_edges(&edges));

    // Per-region residual maps (token → raw ΔE) and per-region max positive ΔE
    // (the early-exit bound in `tropical_term`).
    let delta_maps: Vec<HashMap<u32, i64>> = emissions
        .region_lists
        .iter()
        .map(|list| list.iter().map(|(t, dq)| (*t, dq.raw() as i64)).collect())
        .collect();
    let max_pos_delta: Vec<i64> = emissions
        .region_lists
        .iter()
        .map(|list| {
            list.iter()
                .map(|(_, dq)| dq.raw() as i64)
                .max()
                .unwrap_or(0)
        })
        .collect();

    // Shared root mass (same rule and default as the #457 harness).
    let root_top_k = env_usize("R4_TROP_ROOT_TOP", 2_000);
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
    let held_cap = env_usize("R4_TROP_HELD", 5_000);
    let held = &inputs.held_out[..held_cap.min(inputs.held_out.len())];
    let mut kernel = runtime::OpKernel::default();

    let (mut bits_null, mut bits_1h, mut bits_2h, mut bits_n) = (0.0f64, 0.0, 0.0, 0.0);
    let (mut hit_null, mut hit_1h, mut hit_2h, mut hit_n) = (0u64, 0u64, 0u64, 0u64);
    let (mut have_1h, mut have_2h) = (0u64, 0u64);
    let (mut moved_2h, mut witness_delta_wins) = (0.0f64, 0u64);
    let mut witness_examples: Vec<(usize, usize, usize, u32)> = Vec::new();
    let mut graded = 0u64;

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

        let r1 = routes_1hop(&active, &fwd, regions.len());
        let r2 = routes_2hop(&active, &fwd, regions.len());
        let rn = routes_2hop(&active, &fwd_null, regions.len());
        if !r1.is_empty() {
            have_1h += 1;
        }
        if !r2.is_empty() {
            have_2h += 1;
        }

        // Support: shared root mass ∪ every route destination's residual
        // support (all arms' destinations, so every arm scores the same set)
        // ∪ the true next token.
        let mut set: HashSet<u32> = root_top.iter().copied().collect();
        for r in r1.iter().chain(r2.iter()).chain(rn.iter()) {
            for (t, _) in &emissions.region_lists[r.dst_region] {
                set.insert(*t);
            }
        }
        set.insert(obs.next);
        let mut support: Vec<u32> = set.into_iter().collect();
        support.sort_unstable();

        // NULL — root prior alone (the unigram floor).
        let null_logs: Vec<f64> = support
            .iter()
            .map(|&t| root_logprob(&emissions, floor, t))
            .collect();
        let p_null = softmax(&null_logs);
        bits_null += bits_of(&p_null, &support, obs.next);
        if argmax_token(&p_null, &support) == obs.next {
            hit_null += 1;
        }

        // 1H and 2H tropical arms, and the deranged-instrument null.
        let (term1, _, _) = tropical_term(&r1, &delta_maps, &max_pos_delta, &support);
        let p_1h = arm_distribution(&emissions, floor, &support, &term1);
        bits_1h += bits_of(&p_1h, &support, obs.next);
        if argmax_token(&p_1h, &support) == obs.next {
            hit_1h += 1;
        }

        let (term2, via2, win2) = tropical_term(&r2, &delta_maps, &max_pos_delta, &support);
        let p_2h = arm_distribution(&emissions, floor, &support, &term2);
        bits_2h += bits_of(&p_2h, &support, obs.next);
        let am_2h = argmax_token(&p_2h, &support);
        if am_2h == obs.next {
            hit_2h += 1;
        }
        if let Some(i) = support.iter().position(|&t| t == am_2h)
            && via2[i]
        {
            witness_delta_wins += 1;
            // The tropical selection is witness-bearing: the winning route
            // (src → mid → dst) is retained by the fold itself.
            if let Some(ri) = win2[i]
                && witness_examples.len() < 3
            {
                let r = &r2[ri];
                witness_examples.push((
                    r.witness_src,
                    r.witness_mid.unwrap_or(usize::MAX),
                    r.dst_region,
                    am_2h,
                ));
            }
        }

        let (term_n, _, _) = tropical_term(&rn, &delta_maps, &max_pos_delta, &support);
        let p_n = arm_distribution(&emissions, floor, &support, &term_n);
        bits_n += bits_of(&p_n, &support, obs.next);
        if argmax_token(&p_n, &support) == obs.next {
            hit_n += 1;
        }

        let l1: f64 = p_2h.iter().zip(&p_null).map(|(a, b)| (a - b).abs()).sum();
        moved_2h += l1;
        graded += 1;
    }

    let n = graded.max(1) as f64;
    let row = |b: f64, h: u64| (b / n, h as f64 / n);
    let (mb_null, t_null) = row(bits_null, hit_null);
    let (mb_1h, t_1h) = row(bits_1h, hit_1h);
    let (mb_2h, t_2h) = row(bits_2h, hit_2h);
    let (mb_n, t_n) = row(bits_n, hit_n);

    println!("#626 tropical route composition — held-out arms ({graded} positions)");
    println!(
        "  routes present: 1-hop at {have_1h} positions, 2-hop at {have_2h}; \
         2H argmax won via a residual-constrained route at {witness_delta_wins}"
    );
    for (src, mid, dst, tok) in &witness_examples {
        println!("  witness example: region {src} -> {mid} -> {dst} selected token {tok}");
    }
    println!(
        "  arm            bits/token   top-1\n  \
         NULL (root)    {mb_null:>8.4}   {t_null:.4}\n  \
         1H tropical    {mb_1h:>8.4}   {t_1h:.4}\n  \
         2H tropical    {mb_2h:>8.4}   {t_2h:.4}\n  \
         N  deranged    {mb_n:>8.4}   {t_n:.4}"
    );

    // Anti-vacuity (Rule 5): the instrument must be able to fail.
    assert!(have_2h > 0, "no position composed a 2-hop route — vacuous");
    assert!(
        moved_2h / n > 1e-9,
        "the tropical term never moved the distribution (mean L1 {:.2e}) — vacuous",
        moved_2h / n
    );
    // Floor cross-check against the #457 table on the same fixtures/split.
    assert!(
        (mb_null - 8.5635).abs() < 0.1 && (t_null - 0.0620).abs() < 0.01,
        "NULL row does not reproduce the #457 unigram floor (got {mb_null:.4}/{t_null:.4}) — \
         instrument broken"
    );

    println!("\n  ==== verdict (pre-registered #626 exit rule) ====");
    let instrument_valid = t_n <= t_1h && t_n <= t_2h;
    if !instrument_valid {
        println!(
            "  INSTRUMENT INVALID: the deranged null ({t_n:.4}) beats a legitimate arm \
             (1H {t_1h:.4} / 2H {t_2h:.4}) — no claim issues from this run."
        );
        return;
    }
    let positive = t_2h >= t_1h + 0.02 && mb_2h <= mb_1h && t_2h > t_null;
    if positive {
        println!(
            "  POSITIVE: 2H clears 1H by {:+.4} top-1 without a bits regression \
             ({mb_2h:.4} vs {mb_1h:.4}) and clears the floor. Next (separate, \
             pre-registered): runtime-source kernel adoption behind the P-4 scan.",
            t_2h - t_1h
        );
    } else {
        println!(
            "  NEGATIVE against the exit rule: 2H-1H top-1 {:+.4} (gate +0.02), \
             bits {mb_2h:.4} vs {mb_1h:.4}, floor {t_null:.4}.",
            t_2h - t_1h
        );
        println!(
            "  Secondary: 1H-NULL top-1 {:+.4}, bits {:+.4} — {}",
            t_1h - t_null,
            mb_1h - mb_null,
            if t_1h <= t_null {
                "the E_f seat carries no one-hop token signal on this cover; the \
                 argmax-neutral deployed configuration is corroborated from a second angle"
            } else {
                "the seat carries some one-hop signal but the second hop does not \
                 clear the pre-registered margin"
            }
        );
    }
}
