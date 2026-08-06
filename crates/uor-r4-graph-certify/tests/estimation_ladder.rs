//! Estimation ladder for interaction orders (issue #459).
//!
//! Front: reconstructability (the RA framing) — can the joint next-token
//! distribution be rebuilt from compact block/marginal structure instead
//! of exact-context memory? Before #458 invests in higher-order
//! interaction blocks, this harness measures which k-way marginals are
//! even ESTIMABLE at D3 corpus scale: TF1 D-4 warns that ρ̂ degrades once
//! atoms are sparse, so an order whose marginal cannot be estimated from
//! the available corpus cannot feed a block search, whatever its true
//! interaction strength.
//!
//! VARIABLES: token-position variables in the story-bounded context
//! window — the k-way joint over `(t_{-k+1}, …, t_{-1}, next)` for k in
//! {1, 2, 3, 4} (n-gram orders; [`story_bounded_window`] semantics, the
//! same boundary rule the #433 ladder obeyed). A position whose story
//! prefix is shorter than k-1 has no order-k atom and is skipped at that
//! k. These k-way marginals are exactly the higher-order marginals the
//! #458 block search would consume.
//!
//! LADDER: deterministic prefix subsamples at fractions
//! {1/16, 1/8, 1/4, 1/2, 1} — the FIRST ceil(f·N) records truncated back
//! to the last story-run boundary (story-id change) at or before the
//! target, the truncation semantics of `scripts/mc1_subsample_corpus.py`.
//! No RNG anywhere in the ladder.
//!
//! PER (k, f): the k-way marginal estimated from the subsample is compared
//! with the full-corpus marginal:
//!   (a) KL(full ‖ sub) in bits, add-one smoothed over the union of
//!       observed atoms — subsample atoms are a strict subset (prefix),
//!       so the union is the full-corpus atom set;
//!   (b) atom sparsity: fraction of full-corpus atoms whose subsample
//!       count is below 1 (unseen) and below 5;
//!   (c) unique-atom count.
//! Atom keys are exact u128 packings of the k u32 tokens (k ≤ 4 ⇒ ≤ 128
//! bits, collision-free per k), the #433 packing convention.
//!
//! NULL ARM (falsifier): the same ladder on the corpus with the `next`
//! stream shuffled — one fixed-seed xorshift64* Fisher–Yates permutation
//! (seed `0x9E3779B97F4A7C15`), story boundaries untouched, only the
//! context→next pairing broken. Pre-declared null requirements:
//!   N1. null KL(k, 1/1) == 0 for every k (the f=1 subsample IS the
//!       corpus — a harness identity);
//!   N2. the shuffle preserves the unigram marginal EXACTLY (same
//!       multiset of `next` tokens), so the null and observed k=1 full
//!       tables must have identical atom and position counts, and null
//!       KL(1, f) must agree with observed KL(1, f) within a factor of 4
//!       at every f < 1 — both are prefix-sampling error measurements of
//!       the SAME marginal, so any larger divergence means the estimator
//!       responds to the shuffle rather than to sampling (void);
//!   N3. the null ladder is the counting-noise FLOOR: under independence
//!       there is no context→next structure to mis-estimate, so its KL is
//!       pure subsampling noise, and the observed arm is read as EXCESS
//!       over the null at the same (k,f). Independence does NOT make the
//!       null's KL small at sparse rungs — it spreads each context's
//!       continuation mass over the whole unigram, so the null's k-way
//!       atoms are at least as sparse as the observed ones. "Consistent
//!       with independence" therefore means: the null's entire KL is
//!       counting noise, and where observed ≈ null the order is
//!       count-limited, not structure-limited.
//! If N1 or N2 fails, the estimator is broken and every number here is
//! void.
//!
//! FLOOR REGIME: at small fractions the add-one smoothing (over the full
//! atom set A, which grows with k) dominates the subsample distribution
//! whenever positions(sub) ≪ A, so small-f KL values are smoothing-floor
//! measurements and need not be monotone in k or f. Cross-order
//! comparisons and the verdict therefore read f = 1/2, where
//! positions(sub) ≈ A for the largest k.
//!
//! PRE-DECLARED FLOOR: 0.05 bits. Justification: recent Gate C arm
//! separations that changed decisions were ~0.3–0.5 bits/token; a
//! marginal known to within 0.05 bits KL sits an order of magnitude
//! below those effects, so block-search decisions made from it are not
//! estimation-noise-driven.
//!
//! PRE-DECLARED VERDICT RULE: the highest k whose OBSERVED KL at f=1/2
//! (half-corpus estimate vs full-corpus reference) is below the floor is
//! "reliably estimable at half D3 scale". For every order above it the
//! corpus multiplier follows the 1/N rule — rare-atom counts scale
//! linearly with corpus size N, so KL ≈ C/N, giving
//! m = KL(k, 1/2) / (2·floor) — with the measured ratio
//! KL(k, 1/4) / KL(k, 1/2) printed as the scaling check (≈2 supports the
//! 1/N rule; outside [1.5, 2.5] the multiplier is flagged unreliable).
//! m ≤ 1 is interpolation within the measured ladder (the order reaches
//! the floor inside the current corpus); m > 1 is extrapolation beyond
//! it. No curve fitting beyond that ratio.
//!
//! Interpretation guide: a verdict of "k=3 reliable, k=4 needs ~Mx" says
//! the #458 block search may consume marginals up to order 3 at D3 scale
//! and must either collect ~M× more corpus or smooth/back off for order
//! 4. If the observed KL at some (k, f) does not exceed the null floor,
//! the order is count-limited rather than structure-limited there.
//!
//! Run:
//!   cargo test -p uor-r4-graph-certify --test estimation_ladder -- --ignored --nocapture
//!
//! Env: R4_CORPUS_META / R4_CORPUS_RECS select the corpus (default:
//! checked-in fixture). Skips vacuously with a printed note when the
//! fixtures are absent (κ-test convention).

use std::collections::HashMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_certify::score::story_bounded_window;

/// Ladder fractions as (numerator, denominator) pairs, ascending.
const FRACTIONS: [(u64, u64); 5] = [(1, 16), (1, 8), (1, 4), (1, 2), (1, 1)];
/// Interaction orders on the ladder.
const KS: [usize; 4] = [1, 2, 3, 4];
/// Pre-declared KL floor (bits) for the verdict; see the module header.
const KL_FLOOR_BITS: f64 = 0.05;
/// Fixed seed for the null-arm permutation (documented in the header).
const NULL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Story-run-boundary truncation: the cut for a target record count is the
/// start of the story run straddling the target — the semantics of
/// `scripts/mc1_subsample_corpus.py`. `target >= n` cuts at the end.
fn run_cut(story: &[u32], target: usize) -> usize {
    let n = story.len();
    if target >= n {
        return n;
    }
    let mut cut = target;
    if story[target - 1] == story[target] {
        cut = target - 1;
        while cut > 0 && story[cut - 1] == story[target - 1] {
            cut -= 1;
        }
    }
    cut
}

/// Exact u128 packing of a fixed-length token tuple (k ≤ 4 ⇒ ≤ 128 bits;
/// collision-free within one k, and tables never mix orders).
fn pack_key(tokens: &[u32]) -> u128 {
    let mut key = 0u128;
    for &t in tokens {
        key = (key << 32) | u128::from(t);
    }
    key
}

/// Atom-count table for one order k over records `0..cut`, keyed by the
/// k-way tuple (story-bounded context of length k-1, then `next`).
struct Counts {
    table: HashMap<u128, u64>,
    /// Positions that contributed an atom (full story-bounded window).
    positions: u64,
}

fn count_atoms(c: &compiler::Corpus, next: &[u32], k: usize, cut: usize) -> Counts {
    let mut table: HashMap<u128, u64> = HashMap::new();
    let mut positions = 0u64;
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..cut {
        let mut key = 0u128;
        if k > 1 {
            let window = story_bounded_window(c, i, k - 1);
            if window.len() < k - 1 {
                continue;
            }
            key = pack_key(window);
        }
        key = (key << 32) | u128::from(next[i]);
        *table.entry(key).or_default() += 1;
        positions += 1;
    }
    Counts { table, positions }
}

/// Add-one-smoothed KL(full ‖ sub) in bits over the union of observed
/// atoms. The subsample is a prefix of the full corpus, so its atoms are
/// a subset and the union is the full-corpus atom set.
fn kl_bits(full: &Counts, sub: &Counts) -> f64 {
    let atoms = full.table.len() as f64;
    let p_norm = full.positions as f64 + atoms;
    let q_norm = sub.positions as f64 + atoms;
    let mut kl = 0.0f64;
    for (&key, &fc) in &full.table {
        let p = (fc as f64 + 1.0) / p_norm;
        let q = (sub.table.get(&key).copied().unwrap_or(0) as f64 + 1.0) / q_norm;
        kl += p * (p / q).log2();
    }
    kl
}

/// Fraction of full-corpus atoms whose subsample count is below `below`.
fn sparsity(full: &Counts, sub: &Counts, below: u64) -> f64 {
    let n = full.table.len().max(1) as f64;
    let sparse = full
        .table
        .keys()
        .filter(|k| sub.table.get(*k).copied().unwrap_or(0) < below)
        .count() as f64;
    sparse / n
}

/// Deterministic shuffle of the `next` stream: one fixed-seed xorshift64*
/// Fisher–Yates permutation of the position indices. Story boundaries are
/// untouched; only the context→next pairing is broken.
fn shuffled_next(next: &[u32]) -> Vec<u32> {
    let mut state = NULL_SEED;
    let mut rand = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut perm: Vec<u32> = (0..next.len() as u32).collect();
    for i in (1..perm.len()).rev() {
        let j = (rand() % (i as u64 + 1)) as usize;
        perm.swap(i, j);
    }
    perm.iter().map(|&p| next[p as usize]).collect()
}

/// One ladder row: every metric for one (k, fraction).
struct Row {
    k: usize,
    frac_label: String,
    positions: u64,
    unique_atoms: usize,
    /// Atom count and counted positions of the full-corpus reference.
    full_atoms: usize,
    full_positions: u64,
    kl: f64,
    pct_below_1: f64,
    pct_below_5: f64,
}

/// Run the ladder under one `next` stream and print it under `label`.
/// Returns the rows (observed arm feeds the verdict).
fn run_arm(c: &compiler::Corpus, next: &[u32], label: &str) -> Vec<Row> {
    println!("\n==== ladder: {label} ====");
    println!(
        "{:>2} | {:>5} | {:>9} | {:>11} | {:>10} | {:>11} | {:>11}",
        "k", "frac", "positions", "atoms(sub)", "KL(bits)", "atoms<1(%)", "atoms<5(%)"
    );
    let mut rows = Vec::new();
    for &k in &KS {
        let full = count_atoms(c, next, k, c.n);
        println!(
            "{:>2} | {:>5} | atoms(full) = {}, positions(full) = {}",
            k,
            "ref",
            full.table.len(),
            full.positions
        );
        for &(num, den) in &FRACTIONS {
            let target = (c.n as u64 * num).div_ceil(den) as usize;
            let cut = run_cut(&c.story, target);
            let sub = count_atoms(c, next, k, cut);
            let row = Row {
                k,
                frac_label: format!("{num}/{den}"),
                positions: sub.positions,
                unique_atoms: sub.table.len(),
                full_atoms: full.table.len(),
                full_positions: full.positions,
                kl: kl_bits(&full, &sub),
                pct_below_1: 100.0 * sparsity(&full, &sub, 1),
                pct_below_5: 100.0 * sparsity(&full, &sub, 5),
            };
            println!(
                "{:>2} | {:>5} | {:>9} | {:>11} | {:>10.4} | {:>11.2} | {:>11.2}",
                row.k,
                row.frac_label,
                row.positions,
                row.unique_atoms,
                row.kl,
                row.pct_below_1,
                row.pct_below_5
            );
            rows.push(row);
        }
    }
    rows
}

/// KL at one (k, fraction) out of a finished arm's rows.
fn kl_at(rows: &[Row], k: usize, frac: &str) -> f64 {
    rows.iter()
        .find(|r| r.k == k && r.frac_label == frac)
        .map(|r| r.kl)
        .expect("ladder row")
}

#[test]
#[ignore = "measurement harness (issue #459); run explicitly with --ignored"]
fn estimation_ladder() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path}); vacuous green");
        return;
    };
    println!("corpus: {meta_path} + {recs_path}");
    println!(
        "estimation ladder (#459): {} records, {} stories; orders k = {:?}; fractions = 1/16, 1/8, 1/4, 1/2, 1",
        c.n, c.stories, KS
    );

    let observed = run_arm(&c, &c.next, "observed (c.next)");
    let shuffled = shuffled_next(&c.next);
    let null = run_arm(
        &c,
        &shuffled,
        "NULL: next shuffled (fixed-seed permutation)",
    );

    // ---- pre-declared null requirements ----
    println!("\n==== null checks (pre-declared) ====");
    let n1 = KS.iter().all(|&k| kl_at(&null, k, "1/1") == 0.0);
    println!(
        "N1 null KL(k, 1/1) == 0 for all k: {}",
        if n1 {
            "PASS"
        } else {
            "FAIL — estimator void"
        }
    );
    // N2: the shuffle preserves the unigram marginal exactly, so the k=1
    // full tables must match and the two arms' KL(1, f) — two prefix
    // sampling-error measurements of the SAME marginal — must agree
    // within a factor of 4 at every f < 1.
    fn row_at<'a>(rows: &'a [Row], frac: &str) -> Option<&'a Row> {
        rows.iter().find(|r| r.k == 1 && r.frac_label == frac)
    }
    let n2_table = match (row_at(&observed, "1/1"), row_at(&null, "1/1")) {
        (Some(o), Some(n)) => o.full_atoms == n.full_atoms && o.full_positions == n.full_positions,
        _ => false,
    };
    let mut n2_ratio = true;
    for &(num, den) in &FRACTIONS[..FRACTIONS.len() - 1] {
        let frac = format!("{num}/{den}");
        let (o, n) = (kl_at(&observed, 1, &frac), kl_at(&null, 1, &frac));
        let ratio = if n > 0.0 { o / n } else { f64::INFINITY };
        println!("  k=1 {frac:>5}: observed {o:.4} bits | null {n:.4} bits | ratio {ratio:.2}");
        if !(0.25..=4.0).contains(&ratio) {
            n2_ratio = false;
        }
    }
    println!(
        "N2 unigram marginal invariant under shuffle (table match: {}; KL ratios within [0.25, 4]: {}): {}",
        if n2_table { "yes" } else { "NO" },
        if n2_ratio { "yes" } else { "NO" },
        if n2_table && n2_ratio {
            "PASS"
        } else {
            "FAIL — estimator void"
        }
    );
    println!("N3 null ladder is the counting-noise floor; observed excess over null per (k, 1/2):");
    for &k in &KS {
        println!(
            "  k={k}: observed {:.4} bits | null {:.4} bits | excess {:.4} bits",
            kl_at(&observed, k, "1/2"),
            kl_at(&null, k, "1/2"),
            kl_at(&observed, k, "1/2") - kl_at(&null, k, "1/2")
        );
    }

    // ---- pre-declared verdict ----
    println!("\n==== VERDICT (#459) ====");
    println!(
        "floor: {KL_FLOOR_BITS} bits KL at f=1/2 (half-corpus estimate vs full-corpus reference)"
    );
    let mut k_star = 0usize;
    for &k in &KS {
        let kl = kl_at(&observed, k, "1/2");
        println!(
            "  k={k}: KL(1/2) = {kl:.4} bits — {}",
            if kl < KL_FLOOR_BITS {
                "below floor"
            } else {
                "ABOVE floor"
            }
        );
        if kl < KL_FLOOR_BITS {
            k_star = k;
        }
    }
    if k_star == 0 {
        println!("no order meets the floor — even the unigram is not estimable at D3 scale");
        return;
    }
    println!("highest reliably estimable order at half D3 scale (KL(1/2) < floor): k = {k_star}");
    // Corpus multiplier for every order above k_star, by the pre-declared
    // 1/N rule: m = KL(k, 1/2) / (2·floor). m ≤ 1 is INTERPOLATION within
    // the measured ladder (the order reaches the floor inside the current
    // corpus); m > 1 is EXTRAPOLATION beyond it.
    println!("corpus multipliers above k* (1/N rule: m = KL(1/2) / (2·floor)):");
    for &k in KS.iter().filter(|&&k| k > k_star) {
        let kl_half = kl_at(&observed, k, "1/2");
        let kl_quarter = kl_at(&observed, k, "1/4");
        let multiplier = kl_half / (2.0 * KL_FLOOR_BITS);
        let ratio = kl_quarter / kl_half;
        let reach = if multiplier <= 1.0 {
            "reachable WITHIN the current corpus (interpolation)"
        } else {
            "needs more than the current corpus (EXTRAPOLATION)"
        };
        let scaling = if (1.5..=2.5).contains(&ratio) {
            "~2.0 supports the 1/N rule"
        } else {
            "outside [1.5, 2.5] — multiplier UNRELIABLE"
        };
        println!(
            "  k = {k}: KL(1/2) = {kl_half:.4} bits ⇒ ~{multiplier:.2}x — {reach}; scaling check KL(1/4)/KL(1/2) = {ratio:.2} ({scaling})"
        );
    }
    if k_star == KS[KS.len() - 1] {
        println!(
            "k = {} is the top of the ladder; no higher order measured (add k=5 for the k+1 extrapolation)",
            KS[KS.len() - 1]
        );
    }
}
