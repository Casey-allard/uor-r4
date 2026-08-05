//! Hopf retrieval-quality harness (issue #422): the measurement issue
//! \#306 explicitly conditioned on and never ran. Issue #306 replaced
//! the magnitude-only Hopf projection (four block L2 norms — sign-lossy,
//! near-constant on evolved states) with a signed per-block projection
//! onto fixed Blake3-derived directions, and reported a 6.1x
//! sector-occupancy gain on the #303 D3 protocol, conditioning "treated
//! as an improvement" on a retrieval-quality / collision measurement.
//! This harness is that measurement, at D3 corpus scale, on the natural
//! stack (issue #421 M-R1 identified retrieval quality as the suspected
//! lever on the router-vs-unigram anchor gap).
//!
//! # Corpus, mapping, split
//!
//! Same natural stack as `memory_lift_corpus.rs` and the M-R1 harness
//! (`uor-r4-graph-certify/tests/router_reconnect.rs`): the D3 token
//! corpus via `R4_CORPUS_META` / `R4_CORPUS_RECS`, each token id
//! rendered as a synthetic word (`t00042`) — the issue-421 bijection
//! (the router consumes word identity and co-occurrence only, never
//! natural language). The split is the `anchor_infill.rs` law: the D3
//! hash partition when `R4_STORIES` is set, otherwise the sequential
//! eighty/twenty story cut. Only construction-split stories are
//! ingested; stored contexts are consecutive non-overlapping
//! eight-token windows (`compiler::WINDOW`) deduplicated to first
//! occurrence (the #423 store construction). Probe queries are the
//! \#423 held-out query form — a target window's even-offset tokens in
//! reverse order: half the words, different order, no novel words, and
//! the query string itself is never ingested.
//!
//! # Sector addressing under measurement
//!
//! Every stored window and every probe query is routed through the
//! production routing surface
//! (`route_query_to_manifold_native_with_hopf_input`, the #303/#306
//! occupancy surface) under the single store identity, WITHOUT session
//! evolution: a sector used as a store address must be a stable
//! function of content, and the routing path grounds content
//! deterministically against the fixed default session state (the
//! occupancy harness measured session dynamics instead; that is a
//! different question). The returned 512-d Hopf input is exactly the
//! vector `get_state_4d_projection` reduces, so each arm's sector is an
//! honest re-projection of identical inputs through
//! `uor_r4_core::assign_sector_hopf_transport_scalar` with the
//! identity's own control-plane lambda and chi bins (asserted constant,
//! and arm A's recomputed sector is asserted equal to the production
//! `sector_id` — the shim path is tied to the shipped path).
//!
//! # Arms (the #306 change is the only degree of freedom)
//!
//! - ARM A — post-remediation as shipped: the production signed
//!   projection (`get_state_4d_projection_native`).
//! - ARM B — sector-ablated: a test-local shim reproducing the
//!   pre-#306 `get_state_4d_projection` verbatim (four block L2 norms,
//!   L2-normalized, same `denom < 1e-12` fallback; commit `c8eab90`
//!   is the reference). This is the ideal ablation: identical Hopf
//!   input, identical binning, identical control plane — only the
//!   projection #306 changed differs.
//! - ARM C — shuffled control: arm A's stored-window sectors rotated
//!   across window positions by a fixed half-length rotation (the #423
//!   control construction); query sectors unrotated. Same occupancy
//!   histogram as arm A, sector-content correspondence destroyed.
//!
//! # Metrics
//!
//! Base retrieval is the #423 methodology: content-derived query
//! vectors through the router's own indexing surface (scratch
//! identities), cosine over stored state vectors, stable descending
//! sort, ties by ascending window index. Per arm:
//!
//! - (a) SECTOR-FILTERED retrieval MRR / top-1: candidates restricted
//!   to stored windows whose sector (under that arm's assignment)
//!   equals the query's sector; a probe whose target lies outside the
//!   query's sector scores reciprocal rank zero. This is what a sector
//!   index would serve, so it is the retrieval-quality lever.
//! - (a') sector-AGREEMENT slice of the unfiltered base MRR: mean
//!   reciprocal rank over probes where query and target sectors agree
//!   vs disagree — does the address carry retrieval signal at all?
//! - (b) COLLISION metrics over stored contexts: occupied sectors,
//!   same-cell collision rate (probability a random pair of distinct
//!   stored contexts shares a sector cell), nearest-cell collision
//!   rate (identical or lattice-adjacent cells; delta and alpha bins
//!   wrap, chi does not), and max cell load. Arm C's histogram equals
//!   arm A's by construction (label rotation) and is not repeated.
//!
//! # Caps (documented, printed, no silent truncation)
//!
//! Ingestion is capped at the first `R4_HOPF_CONSTR_STORIES`
//! construction stories (default two thousand, ascending story id) and
//! probes at `R4_HOPF_PROBES` target windows (default five hundred,
//! evenly strided). Both caps print in the report.
//!
//! # Pre-declared exit rule
//!
//! Issue #306's "improvement" claim is CONFIRMED if and only if arm A
//! (post-remediation) exceeds arm B (sector-ablated) on sector-filtered
//! MRR by at least an MRR margin of 0.020 AND exceeds arm C (shuffled).
//! Anything less is a recorded negative: occupancy spread without
//! retrieval value. Collision rates are reported regardless of the
//! exit outcome. Structural invariants gate; direction prints (the
//! \#423 convention).
//!
//! # Redesign candidates (issue #422 phase 2, `R4_HOPF_REDESIGN=1`)
//!
//! The phase-1 run recorded the pre-declared NEGATIVE: the shipped
//! signed projection is content-BLIND (a fixed Blake3 direction per
//! block), so sectors spread (456/512) but carry no retrieval
//! structure (filtered MRR 0.0045), while the sign-lossy magnitude
//! shim collapses occupancy (16/512) yet retrieves better (0.0743)
//! because block norms weakly correlate with content. Setting
//! `R4_HOPF_REDESIGN=1` additionally measures CONTENT-ALIGNED
//! candidates — can a projection get both spread and retrieval value?
//! With the flag unset the original three-arm measurement is
//! unchanged. Every candidate is a test-local shim (the arm B
//! pattern) fed the identical 512-d Hopf inputs and binned through
//! the same production `assign_sector_hopf_transport_scalar`; same
//! metrics, same caps.
//!
//! - ARM E — data-PCA: per 128-d block the direction is the top
//!   principal component of that block across the CONSTRUCTION-split
//!   stored windows' Hopf inputs (mean-centered covariance).
//!   Deterministic, no RNG: sequential accumulation in window order,
//!   power iteration from the uniform all-ones start vector for a
//!   fixed 64 iterations (convergence is not asserted — determinism,
//!   not optimality, is the requirement; a degenerate `< 1e-12`
//!   iterate keeps the previous vector), sign canonicalized so the
//!   largest-magnitude coordinate (lowest index on exact ties, via
//!   strict `>`) is positive. Component = signed dot of the raw block
//!   with the PC direction; the 4-vector is normalized exactly as the
//!   shipped code (same `denom < 1e-12` fallback). Probe queries
//!   reuse the construction-estimated directions — no held-out
//!   leakage.
//! - ARM F — magnitude+sign hybrid: component = the pre-#306 block
//!   L2 norm (arm B's quantity) SIGNED by the block's dot with the
//!   \#306 Blake3 direction (arm A's quantity; a zero dot counts
//!   positive), 4-vector normalized as shipped. Keeps the magnitude
//!   information arm B retrieves with and adds one content-derived
//!   sign bit per block for spread.
//! - ARM G — mean-direction: component = dot of the raw block with
//!   the L2-normalized construction mean of that block (a `< 1e-12`
//!   mean norm falls back to a zero component), 4-vector normalized
//!   as shipped.
//!
//! Per-candidate pre-declared rule (same margins as phase 1): the
//! candidate is CONFIRMED iff its sector-filtered MRR is at least arm
//! B's plus 0.020 AND exceeds arm C's (shuffled), both measured in
//! the same run at the same caps. Occupancy is reported alongside —
//! the goal is beating arm B on MRR while occupying substantially
//! more than arm B's sectors.
//!
//! Run (natural stack):
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl \
//!   cargo test --release -p uor-r4-router --test hopf_retrieval_quality -- \
//!   --ignored --nocapture
//! Add R4_HOPF_REDESIGN=1 for the phase-2 candidate arms.

/// A candidate block-projection shim: 512-d Hopf input to a 4-vector.
type ProjectionFn = Box<dyn Fn(&[f64]) -> Vec<f64>>;

use std::collections::{BTreeMap, HashMap, HashSet};

use uor_r4_core::transformerless::compiler;
use uor_r4_router::UorR4Router;

/// Identity scope of the corpus store (fixes the control plane: lambda
/// and chi bins are identity-derived and must be shared by store and
/// query for a sector index to function).
const ID: &str = "user:hopfrq";
/// Sector budget K at the production call site (#303 convention).
const SECTOR_CAP: usize = 512;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

/// Token id rendered as the synthetic router word (module docs).
fn token_word(token: u32) -> String {
    format!("t{token:05}")
}

/// An eight-token window rendered as the stored sentence form.
fn render_window(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    format!("{}.", words.join(" "))
}

/// The held-out probe query for a target window: even-offset tokens in
/// reverse order (the #423 query law).
fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

fn cosine(a: &[f64], b: &[f64], norm_a: f64, norm_b: f64) -> f64 {
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot / (norm_a * norm_b)
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// ARM B shim: the pre-#306 `get_state_4d_projection`, reproduced
/// verbatim from the parent of commit `c8eab90` ("Remediate Hopf
/// projection sign loss"): per-block L2 norms — non-negative, hence the
/// sign loss — normalized by their own L2, with the same near-zero
/// fallback. Test-local because the shipped router no longer contains
/// this code path; the reproduction is exact, so arm B IS the
/// pre-remediation sector assignment on identical inputs.
fn magnitude_only_projection(state_vector: &[f64]) -> Vec<f64> {
    let block_norm = |offset: usize| -> f64 {
        state_vector[offset..offset + 128]
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
    };
    let w = [
        block_norm(0),
        block_norm(128),
        block_norm(256),
        block_norm(384),
    ];
    let denom = w.iter().map(|value| value * value).sum::<f64>().sqrt();
    if denom < 1e-12 {
        vec![0.5, 0.5, 0.5, 0.5]
    } else {
        w.iter().map(|value| value / denom).collect()
    }
}

/// The shipped 4-vector normalization, shared by the redesign
/// candidates: L2-normalize the four components with the same
/// `denom < 1e-12` fallback `get_state_4d_projection` uses.
fn normalize4(components: [f64; 4]) -> Vec<f64> {
    let denom = components
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if denom < 1e-12 {
        vec![0.5, 0.5, 0.5, 0.5]
    } else {
        components.iter().map(|value| value / denom).collect()
    }
}

/// Test-local reproduction of the #306 Blake3 probe direction for one
/// 128-d block (`hopf_signed_projection_component`'s unit direction):
/// byte `index % 32` of the domain-separated digest mapped to
/// `[-0.5, 0.5]`, L2-normalized. Exact by construction — same domain
/// string, same little-endian block tag, same probe map.
fn blake3_direction(block: usize) -> [f64; 128] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4 issue-306 signed-hopf-projection");
    hasher.update(&(block as u64).to_le_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    let mut probe = [0.0f64; 128];
    for (index, value) in probe.iter_mut().enumerate() {
        *value = (bytes[index % bytes.len()] as f64 / 255.0) - 0.5;
    }
    let norm = probe.iter().map(|x| x * x).sum::<f64>().sqrt();
    for value in probe.iter_mut() {
        *value /= norm;
    }
    probe
}

/// ARM F shim (module docs): per block, the pre-#306 L2 magnitude
/// signed by the block's dot with the #306 Blake3 direction; a zero
/// dot counts positive (deterministic tie rule). Normalized as
/// shipped.
fn magnitude_sign_projection(state_vector: &[f64], dirs: &[[f64; 128]; 4]) -> Vec<f64> {
    let mut components = [0.0f64; 4];
    for (block, (component, dir)) in components.iter_mut().zip(dirs).enumerate() {
        let slice = &state_vector[block * 128..(block + 1) * 128];
        let magnitude = slice.iter().map(|x| x * x).sum::<f64>().sqrt();
        let dot: f64 = slice.iter().zip(dir).map(|(x, d)| x * d).sum();
        *component = if dot < 0.0 { -magnitude } else { magnitude };
    }
    normalize4(components)
}

/// ARM E / ARM G shim (module docs): per block, the signed dot of the
/// raw block with a per-block unit direction. Normalized as shipped.
fn directional_projection(state_vector: &[f64], dirs: &[[f64; 128]; 4]) -> Vec<f64> {
    let mut components = [0.0f64; 4];
    for (block, (component, dir)) in components.iter_mut().zip(dirs).enumerate() {
        let slice = &state_vector[block * 128..(block + 1) * 128];
        *component = slice.iter().zip(dir).map(|(x, d)| x * d).sum();
    }
    normalize4(components)
}

/// Per-block construction statistics for arms G and E: the
/// L2-normalized mean direction and the top principal component of
/// the mean-centered covariance, estimated from the construction
/// windows' Hopf inputs only. Fully deterministic (module docs): all
/// sums accumulate sequentially in window order, the power iteration
/// starts from the uniform all-ones unit vector and runs a fixed 64
/// iterations, and the eigenvector sign is canonicalized on the
/// largest-magnitude coordinate (strict `>`, so the lowest index wins
/// exact ties). No RNG anywhere. Returns `(mean_dirs, pc_dirs)`.
#[allow(clippy::type_complexity)]
fn construction_block_directions(inputs: &[Vec<f64>]) -> ([[f64; 128]; 4], [[f64; 128]; 4]) {
    let n = inputs.len();
    assert!(n > 1, "PCA premise: at least two construction vectors");
    let mut mean_dirs = [[0.0f64; 128]; 4];
    let mut pc_dirs = [[0.0f64; 128]; 4];
    for (block, (mean_slot, pc_slot)) in mean_dirs.iter_mut().zip(pc_dirs.iter_mut()).enumerate() {
        let offset = block * 128;
        let mut mu = [0.0f64; 128];
        for input in inputs {
            for (m, x) in mu.iter_mut().zip(&input[offset..offset + 128]) {
                *m += x;
            }
        }
        for m in mu.iter_mut() {
            *m /= n as f64;
        }
        // Mean-centered second-moment sums (the covariance up to the
        // 1/N scale, which is irrelevant to the eigenvector).
        let mut cov = vec![[0.0f64; 128]; 128];
        let mut centered = [0.0f64; 128];
        for input in inputs {
            for (d, (x, m)) in centered
                .iter_mut()
                .zip(input[offset..offset + 128].iter().zip(&mu))
            {
                *d = x - m;
            }
            for (row, &di) in cov.iter_mut().zip(&centered) {
                for (entry, &dj) in row.iter_mut().zip(&centered) {
                    *entry += di * dj;
                }
            }
        }
        // Deterministic power iteration (module docs).
        let mut v = [1.0 / (128.0f64).sqrt(); 128];
        for _ in 0..64 {
            let mut w = [0.0f64; 128];
            for (wi, row) in w.iter_mut().zip(&cov) {
                *wi = row.iter().zip(&v).map(|(c, x)| c * x).sum();
            }
            let nrm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
            if nrm < 1e-12 {
                break; // degenerate block: keep the previous iterate
            }
            for (vi, wi) in v.iter_mut().zip(&w) {
                *vi = wi / nrm;
            }
        }
        let mut lead = 0usize;
        for (index, value) in v.iter().enumerate().skip(1) {
            if value.abs() > v[lead].abs() {
                lead = index;
            }
        }
        if v[lead] < 0.0 {
            for value in v.iter_mut() {
                *value = -*value;
            }
        }
        *pc_slot = v;
        let mu_norm = mu.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mu_norm >= 1e-12 {
            for (d, m) in mean_slot.iter_mut().zip(&mu) {
                *d = m / mu_norm;
            }
        }
    }
    (mean_dirs, pc_dirs)
}

/// One Hopf sector cell: the flat sector id plus its lattice bins.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    sector: usize,
    chi_bin: usize,
    delta_bin: usize,
    alpha_bin: usize,
}

/// Bin-lattice dimensions (kchi, kdelta, kalpha), shared by both arms.
type Dims = (usize, usize, usize);

/// Assign a 4-d Hopf projection to a sector cell through the production
/// core function with the identity's control-plane parameters.
fn assign_cell(v4: &[f64], lambda: f64, chi_bins: usize) -> (Cell, Dims) {
    let (sector, bins, _) =
        uor_r4_core::assign_sector_hopf_transport_scalar(v4, SECTOR_CAP, lambda, chi_bins);
    (
        Cell {
            sector,
            chi_bin: bins["chi_bin"],
            delta_bin: bins["delta_bin"],
            alpha_bin: bins["alpha_bin"],
        },
        (bins["chi_bins"], bins["delta_bins"], bins["alpha_bins"]),
    )
}

/// True when two cells are identical or lattice-adjacent (Chebyshev
/// distance at most one per axis; delta and alpha are phases and wrap,
/// chi does not).
fn near(a: &Cell, b: &Cell, dims: Dims) -> bool {
    let (_, kdelta, kalpha) = dims;
    let wrap = |x: usize, y: usize, k: usize| {
        let d = x.abs_diff(y);
        d <= 1 || (k > 2 && d == k - 1)
    };
    a.chi_bin.abs_diff(b.chi_bin) <= 1
        && wrap(a.delta_bin, b.delta_bin, kdelta)
        && wrap(a.alpha_bin, b.alpha_bin, kalpha)
}

/// Collision metrics over stored-context cells (module docs, metric b):
/// (occupied sectors, same-cell pair rate, nearest-cell pair rate, max
/// cell load). Pair rates are over ordered pairs of distinct stored
/// contexts, so they read as collision probabilities.
fn collision_report(cells: &[Cell], dims: Dims) -> (usize, f64, f64, usize) {
    let mut loads: BTreeMap<usize, (Cell, usize)> = BTreeMap::new();
    for cell in cells {
        let entry = loads.entry(cell.sector).or_insert((*cell, 0));
        entry.1 += 1;
    }
    let n = cells.len() as f64;
    let denom = n * (n - 1.0);
    let same: f64 = loads
        .values()
        .map(|&(_, count)| (count * (count - 1)) as f64)
        .sum();
    let mut near_pairs = same;
    let entries: Vec<(Cell, usize)> = loads.values().copied().collect();
    for (i, (cell_a, load_a)) in entries.iter().enumerate() {
        for (cell_b, load_b) in entries.iter().skip(i + 1) {
            if near(cell_a, cell_b, dims) {
                near_pairs += 2.0 * (load_a * load_b) as f64;
            }
        }
    }
    let max_load = loads.values().map(|&(_, count)| count).max().unwrap_or(0);
    (loads.len(), same / denom, near_pairs / denom, max_load)
}

/// Rank of `target` among the stored windows sharing `query_sector`
/// under `sector_of` (stable descending cosine, ties by ascending
/// window index — the #423 sort law). `None` when the target itself is
/// outside the query's sector (a sector-index miss).
fn filtered_rank(
    sims: &[f64],
    target: usize,
    query_sector: usize,
    sector_of: &dyn Fn(usize) -> usize,
) -> Option<usize> {
    if sector_of(target) != query_sector {
        return None;
    }
    let target_sim = sims[target];
    let mut rank = 1usize;
    for (position, &sim) in sims.iter().enumerate() {
        if sector_of(position) != query_sector {
            continue;
        }
        if sim > target_sim || (sim == target_sim && position < target) {
            rank += 1;
        }
    }
    Some(rank)
}

/// Unfiltered rank of `target` (the #423 base retrieval).
fn base_rank(sims: &[f64], target: usize) -> usize {
    let target_sim = sims[target];
    let mut rank = 1usize;
    for (position, &sim) in sims.iter().enumerate() {
        if sim > target_sim || (sim == target_sim && position < target) {
            rank += 1;
        }
    }
    rank
}

/// (top-1 hit rate, MRR) where `None` (sector miss) scores zero.
fn filtered_metrics(ranks: &[Option<usize>]) -> (f64, f64) {
    let n = ranks.len() as f64;
    let hits = ranks.iter().filter(|r| **r == Some(1)).count() as f64;
    let mrr: f64 = ranks
        .iter()
        .map(|r| r.map_or(0.0, |rank| 1.0 / rank as f64))
        .sum();
    (hits / n, mrr / n)
}

/// Content-derived query vectors through the router's own indexing
/// surface (scratch identities — the #255/#423 pattern).
fn query_vectors(router: &mut UorR4Router, queries: &[String]) -> Vec<Vec<f64>> {
    queries
        .iter()
        .enumerate()
        .map(|(qi, q)| {
            let scratch = format!("user:hq{qi}");
            router.index_sentence(q, &scratch);
            let items = router.corpus_items_for(&scratch);
            assert_eq!(items.len(), 1, "one stored item per probe query");
            items[0].state_vector.clone()
        })
        .collect()
}

/// Stored state vectors re-ordered into window order (sentence text is
/// the key; store iteration order is HashMap-dependent).
fn aligned_vectors<'a>(router: &'a UorR4Router, windows: &[String]) -> Vec<&'a [f64]> {
    let items = router.corpus_items_for(ID);
    assert_eq!(
        items.len(),
        windows.len(),
        "store holds every deduplicated window (no silent truncation)"
    );
    let by_sentence: HashMap<&str, &[f64]> = items
        .iter()
        .map(|item| (item.sentence.as_str(), item.state_vector.as_slice()))
        .collect();
    windows
        .iter()
        .map(|w| {
            *by_sentence
                .get(w.as_str())
                .expect("every window is stored under its own text")
        })
        .collect()
}

/// Route `text` under the store identity and return its arm A and arm B
/// cells plus the shared lattice dims, control-plane parameters, and
/// the routed 512-d Hopf input (the vector every arm re-projects; the
/// redesign candidates consume it after direction estimation).
/// Asserts arm A's recomputed sector equals the production `sector_id`.
#[allow(clippy::type_complexity)]
fn address(router: &mut UorR4Router, text: &str) -> (Cell, Cell, Dims, f64, usize, Vec<f64>) {
    let (routing, hopf_input) = router.route_query_to_manifold_native_with_hopf_input(text, ID);
    let hopf = &routing.routed.hopf;
    let lambda = hopf.phase_transport_lambda;
    let chi_bins = hopf.hopf_chi_bins as usize;

    let v4_shipped = router.get_state_4d_projection_native(&hopf_input);
    let (cell_a, dims_a) = assign_cell(&v4_shipped, lambda, chi_bins);
    assert_eq!(
        cell_a.sector, hopf.sector_id as usize,
        "arm A recomputation must match the production sector assignment"
    );

    let v4_ablated = magnitude_only_projection(&hopf_input);
    let (cell_b, dims_b) = assign_cell(&v4_ablated, lambda, chi_bins);
    assert_eq!(dims_a, dims_b, "arms share one bin lattice");

    (cell_a, cell_b, dims_a, lambda, chi_bins, hopf_input)
}

#[test]
#[ignore = "issue #422 measurement harness; run explicitly with --ignored"]
fn hopf_retrieval_quality_three_arms() {
    // ---- natural stack ----
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    // ---- split: D3 hash partition when R4_STORIES is set, else the
    // sequential eighty/twenty story cut (anchor_infill.rs law) ----
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
            println!(
                "partition: D3 hash split from {path} ({} construction / {} held-out stories)",
                v.iter().filter(|&&b| b).count(),
                v.iter().filter(|&&b| !b).count()
            );
            v
        }
        Err(_) => (0..c.stories).map(|sid| sid < u64::from(cut)).collect(),
    };

    // ---- per-story token streams (stream index zero = first input) ----
    let mut streams: Vec<(u32, Vec<u32>)> = Vec::new();
    for ((&sid, &input), &next) in c.story.iter().zip(&c.input).zip(&c.next).take(c.n) {
        if streams.last().map(|(last, _)| *last) != Some(sid) {
            streams.push((sid, vec![input]));
        }
        let (_, stream) = streams.last_mut().expect("just pushed");
        stream.push(next);
    }

    // ---- CAPS (module docs; printed, never silent) ----
    let story_cap = env_usize("R4_HOPF_CONSTR_STORIES", 2_000);
    let probe_cap = env_usize("R4_HOPF_PROBES", 500).max(1);
    let constr_total = streams.iter().filter(|(s, _)| constr[*s as usize]).count();
    let capped: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(story_cap)
        .collect();
    println!(
        "CAPS: ingesting {} of {} construction stories; up to {} probes \
         (R4_HOPF_CONSTR_STORIES / R4_HOPF_PROBES override)",
        capped.len(),
        constr_total,
        probe_cap
    );

    // ---- eight-token windows, deduplicated to first occurrence ----
    let mut windows: Vec<String> = Vec::new();
    let mut window_tokens: Vec<Vec<u32>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (_, stream) in &capped {
        for chunk in stream.chunks_exact(compiler::WINDOW) {
            let sentence = render_window(chunk);
            if seen.insert(sentence.clone()) {
                windows.push(sentence);
                window_tokens.push(chunk.to_vec());
            }
        }
    }
    println!("windows: {} distinct eight-token windows", windows.len());
    assert!(
        windows.len() > probe_cap,
        "corpus-scale premise: more windows than probes"
    );

    // ---- probes: evenly strided targets, held-out query form ----
    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    let queries: Vec<String> = targets
        .iter()
        .map(|&t| render_query(&window_tokens[t]))
        .collect();
    println!(
        "probes: {} targets, window stride {stride}, query = even-offset tokens reversed",
        targets.len()
    );

    // ---- one store, production bulk ingestion (#423 pattern) ----
    let mut router = UorR4Router::new(0.5);
    let corpus_text: String = windows.join(" ");
    let indexed = router.index_corpus(&corpus_text, ID);
    assert_eq!(
        indexed,
        windows.len(),
        "production bulk surface indexed every distinct window"
    );
    let qv = query_vectors(&mut router, &queries);

    // ---- sector addressing: every stored window and every query
    // through the production routing surface (module docs) ----
    // Phase-2 flag (module docs): stash the routed Hopf inputs so the
    // redesign candidates re-project the identical vectors. With the
    // flag unset the stashes stay empty and the phase-1 measurement is
    // unchanged.
    let redesign = std::env::var("R4_HOPF_REDESIGN").is_ok_and(|value| value == "1");
    if redesign {
        println!("redesign candidates ENABLED (R4_HOPF_REDESIGN=1): arms E, F, G");
    }
    let mut window_inputs: Vec<Vec<f64>> = Vec::new();
    let mut query_inputs: Vec<Vec<f64>> = Vec::new();
    let mut store_a: Vec<Cell> = Vec::with_capacity(windows.len());
    let mut store_b: Vec<Cell> = Vec::with_capacity(windows.len());
    let mut dims: Option<Dims> = None;
    let mut control: Option<(f64, usize)> = None;
    let mut check = |cells: (Cell, Cell, Dims, f64, usize),
                     store_a: &mut Vec<Cell>,
                     store_b: &mut Vec<Cell>| {
        let (cell_a, cell_b, d, lambda, chi_bins) = cells;
        assert_eq!(*dims.get_or_insert(d), d, "one bin lattice for the run");
        assert_eq!(
            *control.get_or_insert((lambda, chi_bins)),
            (lambda, chi_bins),
            "identity-derived control plane is constant"
        );
        store_a.push(cell_a);
        store_b.push(cell_b);
    };
    for w in &windows {
        let (cell_a, cell_b, d, lambda, chi_bins, hopf_input) = address(&mut router, w);
        check(
            (cell_a, cell_b, d, lambda, chi_bins),
            &mut store_a,
            &mut store_b,
        );
        if redesign {
            window_inputs.push(hopf_input);
        }
    }
    let mut query_a: Vec<Cell> = Vec::with_capacity(queries.len());
    let mut query_b: Vec<Cell> = Vec::with_capacity(queries.len());
    for q in &queries {
        let (cell_a, cell_b, d, lambda, chi_bins, hopf_input) = address(&mut router, q);
        check(
            (cell_a, cell_b, d, lambda, chi_bins),
            &mut query_a,
            &mut query_b,
        );
        if redesign {
            query_inputs.push(hopf_input);
        }
    }
    let dims = dims.expect("at least one sample");
    let (lambda, chi_bins) = control.expect("at least one sample");
    println!(
        "control plane: lambda {lambda:.4}, chi_bins {chi_bins}; lattice {dims:?}; K {SECTOR_CAP}"
    );

    // ---- redesign candidates (module docs): estimate directions from
    // the construction stores only, then re-project every stashed Hopf
    // input through the same production binning ----
    let mut candidates: Vec<(&str, Vec<Cell>, Vec<Cell>)> = Vec::new();
    if redesign {
        let blake3_dirs = [
            blake3_direction(0),
            blake3_direction(1),
            blake3_direction(2),
            blake3_direction(3),
        ];
        let (mean_dirs, pc_dirs) = construction_block_directions(&window_inputs);
        let projections: Vec<(&str, ProjectionFn)> = vec![
            (
                "E data-PCA",
                Box::new(move |input: &[f64]| directional_projection(input, &pc_dirs)),
            ),
            (
                "F mag+sign",
                Box::new(move |input: &[f64]| magnitude_sign_projection(input, &blake3_dirs)),
            ),
            (
                "G mean-dir",
                Box::new(move |input: &[f64]| directional_projection(input, &mean_dirs)),
            ),
        ];
        for (name, project) in projections {
            let assign = |input: &Vec<f64>| {
                let (cell, d) = assign_cell(&project(input), lambda, chi_bins);
                assert_eq!(d, dims, "candidate {name} shares the run's bin lattice");
                cell
            };
            let store: Vec<Cell> = window_inputs.iter().map(&assign).collect();
            let query: Vec<Cell> = query_inputs.iter().map(&assign).collect();
            candidates.push((name, store, query));
        }
    }
    drop(window_inputs);
    drop(query_inputs);

    // ---- arm C: fixed half-length rotation of arm A's store labels ----
    let rotation = windows.len() / 2;
    let sector_c = |position: usize| store_a[(position + rotation) % windows.len()].sector;

    // ---- base retrieval (arm-independent) + per-arm sector metrics ----
    let vectors = aligned_vectors(&router, &windows);
    let norms: Vec<f64> = vectors.iter().map(|v| norm(v)).collect();

    let mut base_rr: Vec<f64> = Vec::with_capacity(targets.len());
    let mut ranks_a: Vec<Option<usize>> = Vec::with_capacity(targets.len());
    let mut ranks_b: Vec<Option<usize>> = Vec::with_capacity(targets.len());
    let mut ranks_c: Vec<Option<usize>> = Vec::with_capacity(targets.len());
    let mut agree_a: Vec<bool> = Vec::with_capacity(targets.len());
    let mut agree_b: Vec<bool> = Vec::with_capacity(targets.len());
    let mut agree_c: Vec<bool> = Vec::with_capacity(targets.len());
    let mut ranks_x: Vec<Vec<Option<usize>>> = candidates
        .iter()
        .map(|_| Vec::with_capacity(targets.len()))
        .collect();
    let mut agree_x: Vec<Vec<bool>> = candidates
        .iter()
        .map(|_| Vec::with_capacity(targets.len()))
        .collect();
    for (probe, (&t, q)) in targets.iter().zip(&qv).enumerate() {
        let qn = norm(q);
        let sims: Vec<f64> = vectors
            .iter()
            .zip(&norms)
            .map(|(v, &vn)| cosine(q, v, qn, vn))
            .collect();
        base_rr.push(1.0 / base_rank(&sims, t) as f64);
        let sec_a = |position: usize| store_a[position].sector;
        let sec_b = |position: usize| store_b[position].sector;
        ranks_a.push(filtered_rank(&sims, t, query_a[probe].sector, &sec_a));
        ranks_b.push(filtered_rank(&sims, t, query_b[probe].sector, &sec_b));
        ranks_c.push(filtered_rank(&sims, t, query_a[probe].sector, &sector_c));
        agree_a.push(store_a[t].sector == query_a[probe].sector);
        agree_b.push(store_b[t].sector == query_b[probe].sector);
        agree_c.push(sector_c(t) == query_a[probe].sector);
        for ((_, store, query), (ranks, agree)) in candidates
            .iter()
            .zip(ranks_x.iter_mut().zip(agree_x.iter_mut()))
        {
            let sec = |position: usize| store[position].sector;
            ranks.push(filtered_rank(&sims, t, query[probe].sector, &sec));
            agree.push(store[t].sector == query[probe].sector);
        }
    }

    let base_mrr = base_rr.iter().sum::<f64>() / base_rr.len() as f64;
    let (top1_a, mrr_a) = filtered_metrics(&ranks_a);
    let (top1_b, mrr_b) = filtered_metrics(&ranks_b);
    let (top1_c, mrr_c) = filtered_metrics(&ranks_c);

    let slice = |agree: &[bool]| -> (usize, f64, f64) {
        let mut on = (0usize, 0.0f64);
        let mut off = (0usize, 0.0f64);
        for (&a, &rr) in agree.iter().zip(&base_rr) {
            let side = if a { &mut on } else { &mut off };
            side.0 += 1;
            side.1 += rr;
        }
        let mean = |(count, sum): (usize, f64)| if count == 0 { 0.0 } else { sum / count as f64 };
        (on.0, mean(on), mean(off))
    };
    let (na, sa_on, sa_off) = slice(&agree_a);
    let (nb, sb_on, sb_off) = slice(&agree_b);
    let (nc, sc_on, sc_off) = slice(&agree_c);

    // ---- collision metrics (metric b; arm C's histogram = arm A's) ----
    let (occ_a, same_a, near_a, max_a) = collision_report(&store_a, dims);
    let (occ_b, same_b, near_b, max_b) = collision_report(&store_b, dims);

    println!(
        "hopf retrieval quality (issue #422): {} windows, {} probes, base MRR {base_mrr:.4}",
        windows.len(),
        targets.len()
    );
    println!(
        "  arm A post-remediation (shipped signed): top1 {top1_a:.3} | filtered MRR {mrr_a:.4}"
    );
    println!(
        "  arm B sector-ablated (pre-#306 shim):    top1 {top1_b:.3} | filtered MRR {mrr_b:.4}"
    );
    println!(
        "  arm C shuffled-sector control:           top1 {top1_c:.3} | filtered MRR {mrr_c:.4}"
    );
    println!(
        "  agreement slice A: {na}/{} agree | base MRR agree {sa_on:.4} vs disagree {sa_off:.4}",
        targets.len()
    );
    println!(
        "  agreement slice B: {nb}/{} agree | base MRR agree {sb_on:.4} vs disagree {sb_off:.4}",
        targets.len()
    );
    println!(
        "  agreement slice C: {nc}/{} agree | base MRR agree {sc_on:.4} vs disagree {sc_off:.4}",
        targets.len()
    );
    println!(
        "  collisions A: {occ_a}/{SECTOR_CAP} sectors occupied | same-cell {same_a:.4} | \
         near-cell {near_a:.4} | max load {max_a}"
    );
    println!(
        "  collisions B: {occ_b}/{SECTOR_CAP} sectors occupied | same-cell {same_b:.4} | \
         near-cell {near_b:.4} | max load {max_b}"
    );

    // Structural invariants gate; direction prints (module docs).
    for (name, m) in [
        ("base", base_mrr),
        ("arm A", mrr_a),
        ("arm B", mrr_b),
        ("arm C", mrr_c),
    ] {
        assert!((0.0..=1.0).contains(&m), "{name} MRR out of range: {m:.4}");
    }
    for (name, rate) in [
        ("A same", same_a),
        ("A near", near_a),
        ("B same", same_b),
        ("B near", near_b),
    ] {
        assert!(
            (0.0..=1.0).contains(&rate),
            "collision rate {name} out of range: {rate:.4}"
        );
    }

    // ---- pre-declared exit rule (module docs) ----
    let confirmed = mrr_a >= mrr_b + 0.020 && mrr_a > mrr_c;
    println!(
        "exit rule (#422): arm A filtered MRR {mrr_a:.4} vs arm B {mrr_b:.4} ({:+.4}, need at \
         least plus 0.020) and vs arm C {mrr_c:.4} ({:+.4}, need positive) -> {}",
        mrr_a - mrr_b,
        mrr_a - mrr_c,
        if confirmed {
            "CONFIRMED (#306 improvement claim holds for retrieval)"
        } else {
            "recorded NEGATIVE (occupancy spread without retrieval value)"
        }
    );

    // ---- phase-2 candidate report + per-candidate pre-declared rule
    // (module docs; runs only under R4_HOPF_REDESIGN=1) ----
    for ((name, store, _), (ranks, agree)) in candidates.iter().zip(ranks_x.iter().zip(&agree_x)) {
        let (top1, mrr) = filtered_metrics(ranks);
        let (n_on, on_mrr, off_mrr) = slice(agree);
        let (occ, same, near_rate, max_load) = collision_report(store, dims);
        println!("  arm {name}: top1 {top1:.3} | filtered MRR {mrr:.4}");
        println!(
            "  agreement slice {name}: {n_on}/{} agree | base MRR agree {on_mrr:.4} vs \
             disagree {off_mrr:.4}",
            targets.len()
        );
        println!(
            "  collisions {name}: {occ}/{SECTOR_CAP} sectors occupied | same-cell {same:.4} | \
             near-cell {near_rate:.4} | max load {max_load}"
        );
        assert!(
            (0.0..=1.0).contains(&mrr),
            "arm {name} MRR out of range: {mrr:.4}"
        );
        for (which, rate) in [("same", same), ("near", near_rate)] {
            assert!(
                (0.0..=1.0).contains(&rate),
                "collision rate {name} {which} out of range: {rate:.4}"
            );
        }
        let candidate_confirmed = mrr >= mrr_b + 0.020 && mrr > mrr_c;
        println!(
            "  exit rule (#422 phase 2) arm {name}: filtered MRR {mrr:.4} vs arm B {mrr_b:.4} \
             ({:+.4}, need at least plus 0.020) and vs arm C {mrr_c:.4} ({:+.4}, need positive) \
             -> {}; occupancy {occ}/{SECTOR_CAP} vs arm B {occ_b}/{SECTOR_CAP}",
            mrr - mrr_b,
            mrr - mrr_c,
            if candidate_confirmed {
                "CONFIRMED (content-aligned spread with retrieval value)"
            } else {
                "recorded NEGATIVE"
            }
        );
    }
}
