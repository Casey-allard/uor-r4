//! Interaction information over context variable groups (issue #458, first
//! slice).
//!
//! Front: reconstructability (the RA framing) — can the joint next-token
//! distribution be rebuilt from compact block/marginal structure instead
//! of exact-context memory? This harness is the BLOCK-SELECTION EVIDENCE
//! for #458: it measures which groups of context-position variables
//! `(t_{-i})` carry dependency with `next` beyond what a left-context
//! cover captures, at exactly the orders the #459 estimation ladder
//! proved estimable. The spectrally-seeded cover experiment is the
//! follow-up consumer of these blocks, gated on PR #462's reconstruction
//! metric landing.
//!
//! #459's estimability bound (same fixture corpus): k=2 joints are
//! well-estimated at full D3 scale (KL 0.0626 bits at half corpus, and
//! structure CONCENTRATES atoms — the observed joint beats an
//! independence table of the same cardinality), while k=3 estimation
//! error sits at the shuffled-null floor, i.e. is pure counting noise.
//! Consequences pre-registered here:
//!
//! - Pairwise quantities (item 1) are reported in absolute bits under
//!   Laplace (add-one) smoothing over the product alphabet of the TYPES
//!   OBSERVED IN THE TABLE (Kx × Ky; every counted type occurs in the
//!   table, so the smoothed joint's marginals are exact and MI >= 0 is
//!   arithmetically guaranteed). Token ids never co-occurring in the
//!   table carry no evidence, and uniform smoothing mass over them would
//!   crush every signal — that is why the alphabet is per-table, not the
//!   tokenizer's vocab. Under independence the Laplace-smoothed joint
//!   factorizes up to finite-sample noise, so the null's pairwise MI is
//!   genuinely near zero and an absolute floor is pre-declared (N1).
//! - Three-way quantities (item 2) CANNOT use product-alphabet Laplace
//!   smoothing (a vocab^3 smoothing mass would crush the signal along
//!   with the noise), so they use observed-support add-one smoothing
//!   (the #459 scheme), whose sparse plug-in bias grows with table
//!   cardinality and does NOT vanish under the null. Per #459's N3
//!   pattern, the shuffled-null value IS the counting-noise floor: every
//!   triple is reported as raw / null / excess (observed − null), and
//!   any triple with |observed| ≤ 2×|null| is flagged "count-noise
//!   regime, per #459". We explicitly do NOT pre-declare II_null ≈ 0 in
//!   absolute bits — that expectation is false for sparse plug-in
//!   estimation; the 2× flag is the pre-declared noise rule.
//!
//! VARIABLES: token-position variables in the story-bounded context
//! window — `t_{-o}` for offsets o = 1..8 (the input token o positions
//! back, valid only while the story id matches, the
//! `story_bounded_window` boundary rule of #433/#459) — and `next`. A
//! position contributes to a table only when every variable in the table
//! is valid there, so per-table N shrinks with the largest offset.
//!
//! MEASUREMENTS (both arms: observed `c.next`, and NULL — one fixed-seed
//! xorshift64* Fisher–Yates permutation of the `next` stream, story
//! boundaries untouched, seed 0x9E3779B97F4A7C15):
//!
//! 1. Pairwise mutual information (bits, Laplace): MI(t_{-i}; next) for
//!    i = 1..8, and MI(t_{-i}; t_{-j}) for all pairs i < j in 1..8 —
//!    ranked tables (ties break to the lowest indices).
//! 2. Three-way interaction information
//!    II(t_{-i}; t_{-j}; next) = MI((t_{-i},t_{-j}); next) − MI(t_{-i};
//!    next) − MI(t_{-j}; next). Positive II = SYNERGY (the pair predicts
//!    next better jointly than separately); negative = REDUNDANCY. All
//!    three component MIs use observed-support add-one smoothing; the
//!    same triples are evaluated under the null. Two rankings are
//!    printed, each top 10: (a) RAW — top pairs by MI((t_{-i},t_{-j});
//!    next), observed arm (the literal #458 brief; note this ranking is
//!    cardinality-biased: wider-offset pairs have more pair atoms, hence
//!    more plug-in bias, under BOTH arms); (b) NULL-CORRECTED — top pairs
//!    by MI excess over the null (the #459-consistent read; the ranking
//!    bias cancels).
//! 3. Null requirements (pre-declared):
//!    N1. null MI(t_{-i}; next) ≤ 0.05 bits for every i (Laplace
//!    smoothing factorizes under independence; 0.05 bits is the #459
//!    floor convention);
//!    N2. context-context MI(t_{-i}; t_{-j}) is bit-identical across
//!    arms (the shuffle touches only `next`) — a harness identity;
//!    N3. the null II is the per-triple counting-noise floor, applied
//!    via the 2× flag above.
//!    If N1 or N2 fails the estimator is broken and every number here
//!    is void.
//!
//! PRE-DECLARED EXIT RULE. A position pair is block-selection evidence
//! for #458 iff its II is POSITIVE (synergy) AND clears the count-noise
//! regime (|observed| > 2×|null|), in EITHER ranking. Adjacent-position
//! pairs (|i−j| = 1) "dominate" iff at least half of a ranking's top 10
//! are adjacent. The verdict names the strongest synergistic pair
//! clearing the noise regime (or states that none does — in which case
//! the honest reading is that pairwise context synergy is not measurable
//! at D3 scale, and #458's block search must wait for more corpus or
//! exploit the redundant structure instead).
//!
//! Determinism: every MI sum iterates atoms in sorted key order (std
//! HashMap seeds are per-instance random, so unsorted iteration would
//! make f64 sums instance-dependent — this is what N2 actually checks).
//!
//! Run:
//!   cargo test -p uor-r4-graph-certify --test interaction_information -- --ignored --nocapture
//!
//! Env: R4_CORPUS_META / R4_CORPUS_RECS select the corpus (default:
//! checked-in fixture). Skips vacuously with a printed note when the
//! fixtures are absent (κ-test convention).

use std::collections::{BTreeMap, HashMap};

use uor_r4_core::transformerless::compiler;

/// Context offsets measured.
const OFFSETS: [usize; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
/// Fixed seed for the null-arm permutation (documented in the header).
const NULL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
/// Pre-declared Laplace-MI null floor (bits); see N1 in the header.
const MI_NULL_FLOOR_BITS: f64 = 0.05;
/// How many top pairs per ranking get II evaluated.
const TOP_PAIRS: usize = 10;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// A variable aligned to corpus positions: `None` where the story-bounded
/// offset is unavailable (story prefix shorter than the offset).
type Var = Vec<Option<u32>>;

/// Offset variable: the input token `offset` positions back within the
/// same story run (story-id equality — the `story_bounded_window` rule).
fn offset_var(c: &compiler::Corpus, offset: usize) -> Var {
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    (0..c.n)
        .map(|i| {
            if i >= offset && c.story[i - offset] == c.story[i] {
                Some(c.input[i - offset])
            } else {
                None
            }
        })
        .collect()
}

/// Joint counts of two variables over positions where both are valid.
/// Keys pack (x, y) into a u64 (collision-free u32 fields).
struct Table2 {
    counts: HashMap<u64, u32>,
    n: u64,
}

fn build2(xv: &Var, yv: &Var) -> Table2 {
    let mut counts: HashMap<u64, u32> = HashMap::new();
    let mut n = 0u64;
    for (x, y) in xv.iter().zip(yv.iter()) {
        if let (Some(x), Some(y)) = (x, y) {
            *counts
                .entry((u64::from(*x) << 32) | u64::from(*y))
                .or_default() += 1;
            n += 1;
        }
    }
    Table2 { counts, n }
}

/// Joint counts of three variables (pair (x,y) plus z), packed into a
/// u128 (collision-free u32 fields, the #433/#459 convention).
struct Table3 {
    counts: HashMap<u128, u32>,
    n: u64,
}

fn build3(xv: &Var, yv: &Var, zv: &Var) -> Table3 {
    let mut counts: HashMap<u128, u32> = HashMap::new();
    let mut n = 0u64;
    for ((x, y), z) in xv.iter().zip(yv.iter()).zip(zv.iter()) {
        if let (Some(x), Some(y), Some(z)) = (x, y, z) {
            let key = (u128::from(*x) << 64) | (u128::from(*y) << 32) | u128::from(*z);
            *counts.entry(key).or_default() += 1;
            n += 1;
        }
    }
    Table3 { counts, n }
}

/// Marginal counts of a (x, y)-keyed count table (top 32 key bits are x).
fn marginals2(counts: &HashMap<u64, u32>) -> (BTreeMap<u32, u64>, BTreeMap<u32, u64>) {
    let mut cx: BTreeMap<u32, u64> = BTreeMap::new();
    let mut cy: BTreeMap<u32, u64> = BTreeMap::new();
    for (&key, &c) in counts {
        *cx.entry((key >> 32) as u32).or_default() += u64::from(c);
        *cy.entry(key as u32).or_default() += u64::from(c);
    }
    (cx, cy)
}

/// Mutual information (bits) under Laplace (add-one) smoothing over the
/// product alphabet of the types OBSERVED IN THE TABLE (Kx × Ky). Every
/// counted type occurs in the table, so the smoothed joint's marginals
/// are exact and MI >= 0 holds arithmetically. The unobserved-cell sum
/// is evaluated in closed form (O(Kx + Ky + A)): with p0 = 1/p_norm,
/// Σ_unobs log2 px = Ky·Σ_x log2 px − Σ_atoms log2 px (and likewise for
/// y), since every x pairs with all Ky y's. Atom iteration is in sorted
/// key order (deterministic f64 summation; see the header).
fn mi_laplace(t: &Table2) -> f64 {
    let (cx, cy) = marginals2(&t.counts);
    let kx = cx.len() as u64;
    let ky = cy.len() as u64;
    let atoms = t.counts.len() as u64;
    let unobs = kx * ky - atoms;
    let p_norm = t.n as f64 + (kx * ky) as f64;
    let px_of = |c: u64| (c + ky) as f64 / p_norm;
    let py_of = |c: u64| (c + kx) as f64 / p_norm;
    let sum_log_px: f64 = cx.values().map(|&c| px_of(c).log2()).sum();
    let sum_log_py: f64 = cy.values().map(|&c| py_of(c).log2()).sum();
    let p0 = 1.0 / p_norm;
    let mut keys: Vec<u64> = t.counts.keys().copied().collect();
    keys.sort_unstable();
    let mut mi = 0.0f64;
    let mut obs_log_px = 0.0f64;
    let mut obs_log_py = 0.0f64;
    for key in keys {
        let c = t.counts[&key];
        let p = (f64::from(c) + 1.0) / p_norm;
        let px = px_of(cx[&((key >> 32) as u32)]);
        let py = py_of(cy[&(key as u32)]);
        mi += p * (p / (px * py)).log2();
        obs_log_px += px.log2();
        obs_log_py += py.log2();
    }
    if unobs > 0 {
        let u = unobs as f64;
        mi += p0
            * (u * p0.log2()
                - (ky as f64 * sum_log_px - obs_log_px)
                - (kx as f64 * sum_log_py - obs_log_py));
    }
    mi
}

/// Mutual information (bits) under observed-support add-one smoothing
/// (the #459 scheme): p(x,y) = (c+1)/(N+A) over observed atoms only,
/// marginals by marginalization of the smoothed joint (row x carries
/// c_x + m_x, where m_x is its distinct-y count). Sparse plug-in MI has
/// a cardinality-proportional positive bias; used only where
/// product-alphabet Laplace is infeasible (three-way tables) or as a
/// cross-check, always read against the same-scheme null. Atom iteration
/// is in sorted key order (deterministic f64 summation).
fn mi_support(counts: &HashMap<u64, u32>, n: u64) -> f64 {
    let atoms = counts.len() as f64;
    let p_norm = n as f64 + atoms;
    let (cx, cy) = marginals2(counts);
    let mut mx: BTreeMap<u32, u64> = BTreeMap::new();
    let mut my: BTreeMap<u32, u64> = BTreeMap::new();
    for &key in counts.keys() {
        *mx.entry((key >> 32) as u32).or_default() += 1;
        *my.entry(key as u32).or_default() += 1;
    }
    let mut keys: Vec<u64> = counts.keys().copied().collect();
    keys.sort_unstable();
    let mut mi = 0.0f64;
    for key in keys {
        let c = counts[&key];
        let (x, y) = ((key >> 32) as u32, key as u32);
        let p = (f64::from(c) + 1.0) / p_norm;
        let px = (cx[&x] + mx[&x]) as f64 / p_norm;
        let py = (cy[&y] + my[&y]) as f64 / p_norm;
        mi += p * (p / (px * py)).log2();
    }
    mi
}

/// MI of a three-way table read as MI((x,y); z): the pair is one joint
/// variable (u64 value (x<<32)|y), z the low u32. Observed-support
/// smoothing (see `mi_support`); sorted atom iteration.
fn mi_pair_z_support(t: &Table3) -> f64 {
    let atoms = t.counts.len() as f64;
    let p_norm = t.n as f64 + atoms;
    let mut cp: BTreeMap<u64, u64> = BTreeMap::new();
    let mut cz: BTreeMap<u32, u64> = BTreeMap::new();
    let mut mp: BTreeMap<u64, u64> = BTreeMap::new();
    let mut mz: BTreeMap<u32, u64> = BTreeMap::new();
    for (&key, &c) in &t.counts {
        let pair = (key >> 32) as u64; // (x << 32) | y, fits u64
        let z = key as u32;
        *cp.entry(pair).or_default() += u64::from(c);
        *cz.entry(z).or_default() += u64::from(c);
        *mp.entry(pair).or_default() += 1;
        *mz.entry(z).or_default() += 1;
    }
    let mut keys: Vec<u128> = t.counts.keys().copied().collect();
    keys.sort_unstable();
    let mut mi = 0.0f64;
    for key in keys {
        let c = t.counts[&key];
        let pair = (key >> 32) as u64;
        let z = key as u32;
        let p = (f64::from(c) + 1.0) / p_norm;
        let px = (cp[&pair] + mp[&pair]) as f64 / p_norm;
        let py = (cz[&z] + mz[&z]) as f64 / p_norm;
        mi += p * (p / (px * py)).log2();
    }
    mi
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

/// Everything one arm needs for the II comparison, per (offset pair).
/// Pair maps are indexed by 0-based positions into OFFSETS, i < j only.
struct Arm {
    /// Laplace MI(t_{-i}; next) per offset (parallel to OFFSETS).
    mi_off_laplace: Vec<f64>,
    /// Observed-support MI(t_{-i}; next) per offset (II components).
    mi_off_support: Vec<f64>,
    /// Observed-support MI((t_i,t_j); next) per pair.
    mi_pair_support: BTreeMap<(usize, usize), f64>,
    /// Laplace MI(t_{-i}; t_{-j}) per pair (arm-invariant; N2 checks it).
    mi_ctx_laplace: BTreeMap<(usize, usize), f64>,
}

fn pairs() -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..OFFSETS.len() {
        for j in (i + 1)..OFFSETS.len() {
            v.push((i, j));
        }
    }
    v
}

fn run_arm(next: &Var, off_vars: &[Var]) -> Arm {
    let mut mi_off_laplace = Vec::new();
    let mut mi_off_support = Vec::new();
    for ov in off_vars {
        let t = build2(ov, next);
        mi_off_laplace.push(mi_laplace(&t));
        mi_off_support.push(mi_support(&t.counts, t.n));
    }
    let mut mi_pair_support = BTreeMap::new();
    let mut mi_ctx_laplace = BTreeMap::new();
    for &(i, j) in &pairs() {
        let t3 = build3(&off_vars[i], &off_vars[j], next);
        mi_pair_support.insert((i, j), mi_pair_z_support(&t3));
        let t2 = build2(&off_vars[i], &off_vars[j]);
        mi_ctx_laplace.insert((i, j), mi_laplace(&t2));
    }
    Arm {
        mi_off_laplace,
        mi_off_support,
        mi_pair_support,
        mi_ctx_laplace,
    }
}

/// One evaluated triple: II under both arms plus the noise verdict.
struct Triple {
    i: usize,
    j: usize,
    mi_pair_obs: f64,
    ii_obs: f64,
    ii_null: f64,
    flagged: bool,
}

fn eval_triple(obs: &Arm, nul: &Arm, i: usize, j: usize) -> Triple {
    let mi_pair_obs = obs.mi_pair_support[&(i, j)];
    let ii_obs = mi_pair_obs - obs.mi_off_support[i] - obs.mi_off_support[j];
    let ii_null = nul.mi_pair_support[&(i, j)] - nul.mi_off_support[i] - nul.mi_off_support[j];
    Triple {
        i,
        j,
        mi_pair_obs,
        ii_obs,
        ii_null,
        flagged: ii_obs.abs() <= 2.0 * ii_null.abs(),
    }
}

fn print_triple(t: &Triple) {
    let reading = if t.flagged {
        "count-noise regime, per #459"
    } else if t.ii_obs > 0.0 {
        "SYNERGY"
    } else {
        "redundancy"
    };
    println!(
        "{:>7} | {:>11.4} | {:>10.4} | {:>10.4} | {:>10.4} | {:>10} | {}",
        format!("({},{})", OFFSETS[t.i], OFFSETS[t.j]),
        t.mi_pair_obs,
        t.ii_obs,
        t.ii_null,
        t.ii_obs - t.ii_null,
        if t.flagged { "FLAGGED" } else { "clears" },
        reading
    );
}

#[test]
#[ignore = "measurement harness (issue #458); run explicitly with --ignored"]
fn interaction_information() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path}); vacuous green");
        return;
    };
    println!("corpus: {meta_path} + {recs_path}");
    println!(
        "interaction information (#458): {} records, {} stories; offsets 1..8",
        c.n, c.stories
    );

    let off_vars: Vec<Var> = OFFSETS.iter().map(|&o| offset_var(&c, o)).collect();
    let next_obs: Var = c.next.iter().map(|&t| Some(t)).collect();
    let shuffled = shuffled_next(&c.next);
    let next_null: Var = shuffled.iter().map(|&t| Some(t)).collect();

    let obs = run_arm(&next_obs, &off_vars);
    let nul = run_arm(&next_null, &off_vars);

    // ---- item 1: pairwise MI, ranked ----
    println!("\n==== MI(t_-i; next), bits (Laplace) — ranked ====");
    println!(
        "{:>4} | {:>10} | {:>10} | {:>10}",
        "off", "observed", "null", "excess"
    );
    let mut off_rank: Vec<usize> = (0..OFFSETS.len()).collect();
    off_rank.sort_by(|&a, &b| {
        obs.mi_off_laplace[b]
            .total_cmp(&obs.mi_off_laplace[a])
            .then(a.cmp(&b))
    });
    for &oi in &off_rank {
        println!(
            "{:>4} | {:>10.4} | {:>10.4} | {:>10.4}",
            OFFSETS[oi],
            obs.mi_off_laplace[oi],
            nul.mi_off_laplace[oi],
            obs.mi_off_laplace[oi] - nul.mi_off_laplace[oi]
        );
    }

    println!("\n==== MI(t_-i; t_-j), bits (Laplace, arm-invariant) — ranked top 15 ====");
    println!("{:>7} | {:>10}", "(i,j)", "observed");
    let mut ctx_rank: Vec<(usize, usize)> = pairs();
    ctx_rank.sort_by(|a, b| {
        obs.mi_ctx_laplace[b]
            .total_cmp(&obs.mi_ctx_laplace[a])
            .then(a.cmp(b))
    });
    for &(i, j) in ctx_rank.iter().take(15) {
        println!(
            "{:>7} | {:>10.4}",
            format!("({},{})", OFFSETS[i], OFFSETS[j]),
            obs.mi_ctx_laplace[&(i, j)]
        );
    }

    // ---- item 2: three-way II, two rankings ----
    let header = || {
        println!(
            "{:>7} | {:>11} | {:>10} | {:>10} | {:>10} | {:>10} | reading",
            "(i,j)", "MI(ij;next)", "II obs", "II null", "excess", "noise flag"
        );
    };
    // (a) raw ranking (the literal brief; cardinality-biased).
    let mut raw_rank: Vec<(usize, usize)> = pairs();
    raw_rank.sort_by(|a, b| {
        obs.mi_pair_support[b]
            .total_cmp(&obs.mi_pair_support[a])
            .then(a.cmp(b))
    });
    let raw_top: Vec<(usize, usize)> = raw_rank.iter().copied().take(TOP_PAIRS).collect();
    println!(
        "\n==== II(t_-i; t_-j; next), bits (observed-support smoothing) — ranking (a) RAW: top {TOP_PAIRS} by MI((t_i,t_j); next) ===="
    );
    header();
    let raw_triples: Vec<Triple> = raw_top
        .iter()
        .map(|&(i, j)| eval_triple(&obs, &nul, i, j))
        .collect();
    for t in &raw_triples {
        print_triple(t);
    }
    // (b) null-corrected ranking (excess of MI((t_i,t_j); next) over null).
    let mut cor_rank: Vec<(usize, usize)> = pairs();
    cor_rank.sort_by(|a, b| {
        let ea = obs.mi_pair_support[a] - nul.mi_pair_support[a];
        let eb = obs.mi_pair_support[b] - nul.mi_pair_support[b];
        eb.total_cmp(&ea).then(a.cmp(b))
    });
    let cor_top: Vec<(usize, usize)> = cor_rank.iter().copied().take(TOP_PAIRS).collect();
    println!(
        "\n==== ranking (b) NULL-CORRECTED: top {TOP_PAIRS} by MI((t_i,t_j); next) excess over null ===="
    );
    header();
    let cor_triples: Vec<Triple> = cor_top
        .iter()
        .map(|&(i, j)| eval_triple(&obs, &nul, i, j))
        .collect();
    for t in &cor_triples {
        print_triple(t);
    }

    // ---- item 3: pre-declared null requirements ----
    println!("\n==== null checks (pre-declared) ====");
    let n1 = nul
        .mi_off_laplace
        .iter()
        .all(|&mi| mi <= MI_NULL_FLOOR_BITS);
    println!(
        "N1 null MI(t_-i; next) <= {MI_NULL_FLOOR_BITS} bits for all i (max null: {:.4}): {}",
        nul.mi_off_laplace.iter().copied().fold(0.0f64, f64::max),
        if n1 {
            "PASS"
        } else {
            "FAIL — estimator void"
        }
    );
    let n2 = pairs()
        .iter()
        .all(|p| obs.mi_ctx_laplace[p] == nul.mi_ctx_laplace[p]);
    println!(
        "N2 context-context MI bit-identical across arms: {}",
        if n2 {
            "PASS"
        } else {
            "FAIL — harness broken"
        }
    );
    println!("N3 null II is the per-triple counting-noise floor (2x flags above); sparse-scheme MI(t_-i; next) for scale:");
    for (oi, &o) in OFFSETS.iter().enumerate() {
        println!(
            "  off {o}: observed {:.4} bits | null {:.4} bits",
            obs.mi_off_support[oi], nul.mi_off_support[oi]
        );
    }

    // ---- item 5: verdict (over the union of both rankings) ----
    println!("\n==== VERDICT (#458) ====");
    let mut all: Vec<&Triple> = raw_triples.iter().collect();
    for t in &cor_triples {
        if !all.iter().any(|u| u.i == t.i && u.j == t.j) {
            all.push(t);
        }
    }
    let clearing: Vec<&&Triple> = all
        .iter()
        .filter(|t| !t.flagged && t.ii_obs > 0.0)
        .collect();
    match clearing
        .iter()
        .max_by(|a, b| (a.ii_obs - a.ii_null).total_cmp(&(b.ii_obs - b.ii_null)))
    {
        Some(t) => println!(
            "strongest synergistic pair clearing the noise regime: (t_-{}, t_-{}) — II {:.4} bits (null {:.4}, excess {:.4})",
            OFFSETS[t.i],
            OFFSETS[t.j],
            t.ii_obs,
            t.ii_null,
            t.ii_obs - t.ii_null
        ),
        None => println!(
            "no synergistic pair clears the count-noise regime — pairwise context synergy is not measurable at D3 scale"
        ),
    }
    for (label, triples) in [("raw", &raw_triples), ("null-corrected", &cor_triples)] {
        let adjacent = triples
            .iter()
            .filter(|t| OFFSETS[t.j] - OFFSETS[t.i] == 1)
            .count();
        println!(
            "adjacent-position pairs (|i-j| = 1) in {label} top {TOP_PAIRS}: {adjacent} — {}",
            if adjacent * 2 >= TOP_PAIRS {
                "adjacency DOMINATES (pre-declared rule: >= half)"
            } else {
                "adjacency does not dominate"
            }
        );
    }
    println!(
        "exit rule (#458): a pair is block-selection evidence iff II > 0 AND clears the 2x noise flag; {} of {} evaluated pairs qualify",
        clearing.len(),
        all.len()
    );
}
