//! Host-side, one-time, offline Prime-Rank Indexing and Complex-Plane Zeta-Phase
//! ($\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$) codebook
//! generator and exact integer quantization boundary.
//!
//! Authority: Decision D32 October 14 Addendum (#2398), mechanism brief
//! `docs/mechanisms/prime-zeta-complex-routing.md`, pre-registration sheet
//! `docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md` (TEST FITNESS: FIT,
//! #2433), and owner release on #2029 (comment 6170312990).
//!
//! No model forward, routing, scoring or learning executes on the host (D24/D32).
//! All floating-point arithmetic (`ln`, `sin`, `cos`, `sqrt` on `f64` Odlyzko
//! zeta-zero ordinates) is strictly confined to one-time, offline host codebook
//! construction up to the integer quantization boundary (`PhaseQ29`, `UnitS3Q30`,
//! $E_8$ anchor orbit `ROOTS: [[i8; 8]; 16]`, and `Snapshot.table`).
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::f64::consts::PI;
use uor_r4_discrete::{Error, Result, Spec};

pub mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

mod zeta_zeros {
    include!("../../uor-r4-core/src/zeta_zeros.rs");
}

pub use zeta_zeros::ZETA_ZEROS;

/// Scale factor $2^{29}$ for `PhaseQ29` signed radians in $[-\pi, \pi)$.
pub const PHASE_Q29_SCALE: f64 = (1u64 << 29) as f64;
/// Scale factor $2^{30}$ for `UnitS3Q30` unit 4-vectors on $S^3 \subset \mathbb{R}^4$.
pub const UNIT_S3_Q30_SCALE: f64 = (1u64 << 30) as f64;
/// Default critical-line phase shift $\delta = 1.0$ for the registered tangent null.
pub const TANGENT_NULL_DELTA: f64 = 1.0;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// Signed phase in radians quantized at $2^{-29}$ and wrapped into $[-\pi, \pi)$.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhaseQ29(pub i32);

impl PhaseQ29 {
    pub fn from_radians(radians: f64) -> Result<Self> {
        if !radians.is_finite() {
            return Err(invalid("pre-quantization phase must be finite"));
        }
        let wrapped = wrap_to_pi(radians);
        let scaled = (wrapped * PHASE_Q29_SCALE).round();
        if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
            return Err(invalid("PhaseQ29 quantization overflow"));
        }
        Ok(Self(scaled as i32))
    }

    pub const fn raw(self) -> i32 {
        self.0
    }
}

/// Canonical Q1.30 quantized point on $S^3 \subset \mathbb{R}^4$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnitS3Q30(pub [i32; 4]);

impl UnitS3Q30 {
    pub fn from_r4(values: [f64; 4]) -> Result<Self> {
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("UnitS3Q30 input must be finite"));
        }
        let norm_sq: f64 = values.iter().map(|v| v * v).sum();
        if norm_sq <= 1.0e-24 {
            return Err(invalid("zero R4 vector has no S3 direction"));
        }
        let norm = norm_sq.sqrt();
        let mut out = [0i32; 4];
        for (slot, val) in out.iter_mut().zip(values) {
            let scaled = ((val / norm) * UNIT_S3_Q30_SCALE).round();
            if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
                return Err(invalid("UnitS3Q30 quantization overflow"));
            }
            *slot = scaled as i32;
        }
        if out.iter().all(|&x| x == 0) {
            return Err(invalid("UnitS3Q30 quantization collapsed to zero"));
        }
        Ok(Self(out))
    }

    pub const fn raw(self) -> [i32; 4] {
        self.0
    }
}

fn wrap_to_pi(theta: f64) -> f64 {
    (theta + PI).rem_euclid(2.0 * PI) - PI
}

/// Deterministic prime table $p_1 = 2, p_2 = 3, \dots, p_V$ for $V \le 8192$.
/// Token $v \in \{0, \dots, V-1\}$ maps bijectively to $r(v) = p_{v+1}$.
pub fn first_n_primes(count: usize) -> Result<Vec<u32>> {
    if count == 0 || count > 8192 {
        return Err(invalid("prime table count must be in 1..=8192"));
    }
    let mut primes = Vec::with_capacity(count);
    let mut candidate = 2u32;
    while primes.len() < count {
        let mut is_prime = true;
        for &p in &primes {
            if p * p > candidate {
                break;
            }
            if candidate.is_multiple_of(p) {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            primes.push(candidate);
        }
        candidate += if candidate == 2 { 1 } else { 2 };
    }
    Ok(primes)
}

/// Which codebook construction mode is generated.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CodebookMode {
    /// Primary mechanism: $r(v) = p_{v+1}$, critical-line zeta ordinates $\gamma_k$.
    PrimeZeta,
    /// D22 §3 Tangent Null: critical-line phase shift $\gamma_k \to \gamma_k + \delta$.
    TangentNull { delta: f64 },
    /// D22 §3 Normal Null: seeded prime-rank permutation $\pi_{\text{seed}}(p)$ and
    /// off-critical-line amplitude perturbation $\sigma = \frac{1}{2} \pm \Delta\sigma$.
    NormalNull { seed: u32 },
    /// Pure-integer injective null without writer-cell quota: draws distinct $B$-block
    /// $E_8$ root vectors uniformly without replacement from a seeded integer generator.
    RandomInjective { seed: u32 },
    /// Pure-integer quota-balanced injective null: enforces the same writer-cell quota
    /// ($\lceil V / 256 \rceil$ tokens per cell) and unique interior block roots as
    /// `PrimeZeta`, but assigns tokens to cells and interior roots via a seeded integer
    /// permutation without primes or zeta phases.
    QuotaOnlyInjective { seed: u32 },
}

impl CodebookMode {
    pub fn label(&self) -> String {
        match self {
            Self::PrimeZeta => "prime_zeta".to_owned(),
            Self::TangentNull { delta } => format!("tangent_null_delta_{delta:.1}"),
            Self::NormalNull { seed } => format!("normal_null_seed_{seed}"),
            Self::RandomInjective { seed } => format!("random_injective_seed_{seed}"),
            Self::QuotaOnlyInjective { seed } => format!("quota_only_injective_seed_{seed}"),
        }
    }
}

/// Per-token quantized witness at the integer boundary.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenQuantizationWitness {
    pub token_id: u32,
    pub prime_atom: u32,
    pub phases_q29: Vec<i32>,
    pub spinors_s3_q30: Vec<[i32; 4]>,
    pub root_indices: Vec<u8>,
    pub table_vector: Vec<i32>,
}

/// Complete offline-quantized codebook and integer boundary diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantizedCodebook {
    pub mode: String,
    pub spec: Spec,
    pub seed: u32,
    pub primes: Vec<u32>,
    /// Flat `[vocabulary * blocks * 4]` `PhaseQ29` raw values.
    pub phases_q29: Vec<i32>,
    /// Flat `[vocabulary * blocks * 2]` `UnitS3Q30` 4-vectors.
    pub spinors_s3_q30: Vec<[i32; 4]>,
    /// Selected $E_8$ orbit root index $r \in 0..16$ per `(token, block)`.
    pub root_indices: Vec<u8>,
    /// Flat `[vocabulary * blocks * 8]` integer codebook ready for `Snapshot.table`.
    pub table: Vec<i32>,
    pub table_sha256: String,
    pub phases_sha256: String,
    pub spinors_sha256: String,
    pub unique_phase_vectors: usize,
    pub unique_spinor_vectors: usize,
    pub unique_table_vectors: usize,
    pub occupied_writer_cells: usize,
}

/// Structural Hamming and L1 comparison between two quantized codebooks of the same `Spec`.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralSeparation {
    pub left_mode: String,
    pub right_mode: String,
    pub vocabulary: u32,
    pub blocks: u32,
    /// Number of tokens whose full multi-block `Snapshot.table` vector differs.
    pub differing_tokens: usize,
    /// Total number of `(token, block)` $E_8$ root slots that differ (out of `vocabulary * blocks`).
    pub differing_root_blocks: usize,
    /// Total coordinate Hamming distance across all `vocabulary * blocks * 8` entries.
    pub coordinate_hamming_distance: u64,
    /// Total coordinate L1 distance across all `vocabulary * blocks * 8` entries.
    pub coordinate_l1_distance: u64,
    /// Number of tokens whose writer product cell `(first_block_root, last_block_root)` differs.
    pub differing_writer_cells: usize,
    /// Total absolute difference in `PhaseQ29` coordinates across `vocabulary * blocks * 4`.
    pub phase_q29_l1_distance: u128,
}

fn sha256_i32_slice(values: &[i32]) -> String {
    let mut hasher = Sha256::new();
    for &v in values {
        hasher.update(v.to_le_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Deterministic xorshift64* permutation of prime atoms for the Normal Null.
/// Uses integer-only state transitions so each named seed (`0, 1, 2`) is 100%
/// bit-exact reproducible and distinct.
fn permute_primes_for_normal_null(primes: &[u32], seed: u32) -> Vec<u32> {
    let mut out = primes.to_vec();
    let mut state: u64 =
        (u64::from(seed) ^ 0x9e37_79b9_7f4a_7c15u64).wrapping_mul(0xbf58_476d_1ce4_e5b9u64) | 1;
    let n = out.len();
    for i in (1..n).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let word = state.wrapping_mul(0x2545_f491_4f6c_dd1du64);
        let j = (word as usize) % (i + 1);
        out.swap(i, j);
    }
    // Guarantee no fixed-order identity permutation even for tiny V.
    if n > 1 && out == primes {
        out.swap(0, n - 1);
    }
    out
}

/// Embed two `UnitS3Q30` 4-vectors into the 8D coordinate basis aligned with the
/// four orthogonal 2D root planes of `geometry::ROOTS`:
/// - Plane 0: axes (0, 1) <- (left[0], left[1])
/// - Plane 1: axes (2, 4) <- (left[2], left[3])
/// - Plane 2: axes (3, 7) <- (right[0], right[1])
/// - Plane 3: axes (5, 6) <- (right[2], right[3])
fn embed_s3_pair_into_e8_planes(left: UnitS3Q30, right: UnitS3Q30) -> [i32; 8] {
    let l = left.raw();
    let r = right.raw();
    [l[0], l[1], l[2], r[0], l[3], r[2], r[3], r[1]]
}

/// Exact integer inner product $\langle \mathbf{q}_b, \mathbf{r}_m \rangle$ in `i64`
/// between the quantized 8D vector `q_b` and `geometry::ROOTS[root_idx]`.
fn exact_root_alignment(q_b: &[i32; 8], root_idx: usize) -> i64 {
    let root = &geometry::ROOTS[root_idx];
    let mut sum = 0i64;
    for d in 0..8 {
        sum += i64::from(q_b[d]) * i64::from(root[d]);
    }
    sum
}

/// Deterministic SplitMix64 integer state step used by `RandomInjective` and
/// `QuotaOnlyInjective` so each named seed produces a bit-exact integer table
/// across macOS and Linux with zero non-integer operations.
fn next_seeded_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Pure-integer constructor for `CodebookMode::RandomInjective` and
/// `CodebookMode::QuotaOnlyInjective`.
///
/// Note on witness fields: `phases_q29` and `spinors_s3_q30` are populated with
/// deterministic integer XOR tags purely for `QuantizedCodebook` JSON schema
/// compatibility and uniqueness accounting (`unique_phase_vectors`,
/// `unique_spinor_vectors`). They are not geometric $S^1$ phases or unit-$S^3$
/// spinors; the discrete training and evaluation battery (`pinned_table_v1`)
/// reads only `root_indices` / `Snapshot.table`.
fn generate_integer_injective_null(
    spec: Spec,
    seed: u32,
    mode: CodebookMode,
    null_seed: u32,
    is_quota_only: bool,
) -> Result<QuantizedCodebook> {
    let vocab = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    if !(3..=16).contains(&blocks) {
        return Err(invalid(
            "integer injective nulls require spec.blocks in 3..=16",
        ));
    }
    let interior_cap: usize = if blocks == 3 { 16 } else { 256 };
    if is_quota_only {
        if vocab == 0 || !vocab.is_multiple_of(256) || vocab / 256 > interior_cap {
            return Err(invalid(
                "quota_only_injective requires vocabulary to be a positive multiple of 256 within per-cell interior capacity",
            ));
        }
    } else {
        let total_tuples = 1u128 << (4 * blocks);
        if vocab == 0 || (vocab as u128) * 2 > total_tuples {
            return Err(invalid(
                "random_injective requires 0 < vocabulary <= 16^blocks / 2 for uniform rejection sampling",
            ));
        }
    }

    let primes = permute_primes_for_normal_null(&first_n_primes(vocab)?, null_seed);
    let enforce_unique_full_vector = true;

    let mut rng_state = if is_quota_only {
        u64::from(null_seed) ^ 0xbb67_ae85_84ca_a73bu64
    } else {
        u64::from(null_seed) ^ 0x6a09_e667_f3bc_c908u64
    };

    let mut cell_counts = [0usize; 256];
    let mut used_vectors: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut root_indices = Vec::with_capacity(vocab * blocks);
    let mut table = Vec::with_capacity(vocab * blocks * 8);
    let mut phases_q29 = Vec::with_capacity(vocab * blocks * 4);
    let mut spinors_s3_q30 = Vec::with_capacity(vocab * blocks * 2);

    if is_quota_only {
        // Quota-only injective null:
        // 1. Build a uniform cell assignment where each of the 256 writer cells
        //    appears exactly V / 256 times (for V = 4096, 16 tokens in each of the
        //    256 cells), then shuffle across tokens via seeded Fisher-Yates.
        let mut cell_schedule: Vec<u8> = (0..vocab).map(|i| (i % 256) as u8).collect();
        for i in (1..vocab).rev() {
            let j = (next_seeded_u64(&mut rng_state) as usize) % (i + 1);
            cell_schedule.swap(i, j);
        }

        // 2. Pre-shuffle the interior root choices per writer cell so every token in a
        //    cell receives a unique interior root vector.
        let mut cell_interior_orders: Vec<Vec<u16>> = Vec::with_capacity(256);
        for _cell in 0..256 {
            let mut order: Vec<u16> = (0..interior_cap as u16).collect();
            for i in (1..interior_cap).rev() {
                let j = (next_seeded_u64(&mut rng_state) as usize) % (i + 1);
                order.swap(i, j);
            }
            cell_interior_orders.push(order);
        }

        // For blocks >= 5, pre-shuffle per-cell root permutations for each additional
        // interior block (3..blocks - 1) using SplitMix64 Fisher-Yates. When blocks <= 4,
        // this loop does not execute, keeping B = 4 outputs bit-identical.
        let extra_blocks = blocks.saturating_sub(4);
        let mut extra_interior_orders: Vec<Vec<Vec<u8>>> = Vec::with_capacity(extra_blocks);
        for _extra in 0..extra_blocks {
            let mut per_cell: Vec<Vec<u8>> = Vec::with_capacity(256);
            for _cell in 0..256 {
                let mut order: Vec<u8> = (0..interior_cap).map(|k| (k & 0x0f) as u8).collect();
                for i in (1..interior_cap).rev() {
                    let j = (next_seeded_u64(&mut rng_state) as usize) % (i + 1);
                    order.swap(i, j);
                }
                per_cell.push(order);
            }
            extra_interior_orders.push(per_cell);
        }

        for &cell_u8 in &cell_schedule {
            let cell = usize::from(cell_u8);
            let r_first = (cell / 16) as u8;
            let r_last = (cell % 16) as u8;
            let rank_in_cell = cell_counts[cell];
            let interior_code = cell_interior_orders[cell][rank_in_cell];

            let mut selected = Vec::with_capacity(blocks);
            selected.push(r_first);
            if blocks == 3 {
                selected.push((interior_code & 0x0f) as u8);
            } else {
                selected.push(((interior_code >> 4) & 0x0f) as u8);
                selected.push((interior_code & 0x0f) as u8);
                for (extra_idx, per_cell) in extra_interior_orders.iter().enumerate() {
                    let _ = extra_idx;
                    selected.push(per_cell[cell][rank_in_cell]);
                }
            }
            selected.push(r_last);

            if enforce_unique_full_vector && !used_vectors.insert(selected.clone()) {
                return Err(invalid(
                    "quota_only_injective produced duplicate full E8 root vector",
                ));
            }
            cell_counts[cell] += 1;
            for &r in &selected {
                root_indices.push(r);
                for &coord in &geometry::ROOTS[usize::from(r)] {
                    table.push(i32::from(coord));
                }
            }
        }
    } else {
        // Random injective null without writer-cell quota:
        // Draw V distinct B-block root tuples uniformly without replacement from {0..16}^B
        // via rejection sampling (guaranteed load factor <= 1/2 by the entry guard).
        for _t in 0..vocab {
            let mut chosen: Option<Vec<u8>> = None;
            for _attempt in 0..1024 {
                let word = next_seeded_u64(&mut rng_state);
                let mut candidate = Vec::with_capacity(blocks);
                for b in 0..blocks {
                    let shift = b * 4;
                    candidate.push(((word >> shift) & 0x0f) as u8);
                }
                if !used_vectors.contains(&candidate) {
                    chosen = Some(candidate);
                    break;
                }
            }
            let selected = chosen.ok_or_else(|| {
                invalid("random_injective failed to draw unique E8 multi-block root vector")
            })?;
            let cell = usize::from(selected[0]) * 16 + usize::from(selected[blocks - 1]);
            cell_counts[cell] += 1;
            if enforce_unique_full_vector {
                used_vectors.insert(selected.clone());
            }
            for &r in &selected {
                root_indices.push(r);
                for &coord in &geometry::ROOTS[usize::from(r)] {
                    table.push(i32::from(coord));
                }
            }
        }
    }

    // Populate integer schema witnesses (`phases_q29` and `spinors_s3_q30`) with
    // deterministic integer XOR tags derived from the selected roots, token index,
    // block index, and seed. The training and evaluation battery reads only
    // `root_indices` / `Snapshot.table`.
    let mode_tag: i32 = if is_quota_only {
        0x1357_2468
    } else {
        0x2468_1357
    };
    for t in 0..vocab {
        for b in 0..blocks {
            let r = usize::from(root_indices[t * blocks + b]);
            let coord = &geometry::ROOTS[r];
            let token_block_tag =
                ((t as i32) << 10) ^ ((b as i32) << 5) ^ (null_seed as i32) ^ mode_tag;
            for c in 0..4 {
                let plane_val = i32::from(coord[2 * c]) * 5 + i32::from(coord[2 * c + 1]) * 3;
                let phase_word = (plane_val << 22) ^ token_block_tag ^ (c as i32);
                phases_q29.push(phase_word);
            }
            let left_s3 = [
                (i32::from(coord[0]) << 24) ^ token_block_tag,
                (i32::from(coord[1]) << 24) ^ token_block_tag,
                (i32::from(coord[2]) << 24) ^ token_block_tag,
                (i32::from(coord[4]) << 24) ^ token_block_tag,
            ];
            let right_s3 = [
                (i32::from(coord[3]) << 24) ^ token_block_tag,
                (i32::from(coord[7]) << 24) ^ token_block_tag,
                (i32::from(coord[5]) << 24) ^ token_block_tag,
                (i32::from(coord[6]) << 24) ^ token_block_tag,
            ];
            spinors_s3_q30.push(left_s3);
            spinors_s3_q30.push(right_s3);
        }
    }

    let unique_phase_vectors = phases_q29
        .chunks_exact(blocks * 4)
        .collect::<BTreeSet<_>>()
        .len();
    let unique_spinor_vectors = spinors_s3_q30
        .chunks_exact(blocks * 2)
        .collect::<BTreeSet<_>>()
        .len();
    let unique_table_vectors = table
        .chunks_exact(blocks * 8)
        .collect::<BTreeSet<_>>()
        .len();
    let occupied_writer_cells = cell_counts.iter().filter(|&&c| c > 0).count();

    let table_sha256 = sha256_i32_slice(&table);
    let phases_sha256 = sha256_i32_slice(&phases_q29);
    let flat_spinors: Vec<i32> = spinors_s3_q30.iter().flatten().copied().collect();
    let spinors_sha256 = sha256_i32_slice(&flat_spinors);

    Ok(QuantizedCodebook {
        mode: mode.label(),
        spec,
        seed,
        primes,
        phases_q29,
        spinors_s3_q30,
        root_indices,
        table,
        table_sha256,
        phases_sha256,
        spinors_sha256,
        unique_phase_vectors,
        unique_spinor_vectors,
        unique_table_vectors,
        occupied_writer_cells,
    })
}

/// Generate the quantized codebook for `spec` and `seed` under `mode`.
///
/// All floating-point operations (`ln`, `sin`, `cos`, `sqrt`) occur strictly prior
/// to constructing `PhaseQ29` and `UnitS3Q30`. All subsequent root scoring,
/// multi-block non-aliasing assignment, and `Snapshot.table` population use
/// 100% exact integer arithmetic.
pub fn generate_codebook(spec: Spec, seed: u32, mode: CodebookMode) -> Result<QuantizedCodebook> {
    spec.validate()?;
    match mode {
        CodebookMode::RandomInjective { seed: null_seed } => {
            return generate_integer_injective_null(spec, seed, mode, null_seed, false);
        }
        CodebookMode::QuotaOnlyInjective { seed: null_seed } => {
            return generate_integer_injective_null(spec, seed, mode, null_seed, true);
        }
        CodebookMode::PrimeZeta
        | CodebookMode::TangentNull { .. }
        | CodebookMode::NormalNull { .. } => {}
    }
    let vocab = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    let base_primes = first_n_primes(vocab)?;
    let primes = match mode {
        CodebookMode::PrimeZeta | CodebookMode::TangentNull { .. } => base_primes,
        CodebookMode::NormalNull { seed: null_seed } => {
            permute_primes_for_normal_null(&base_primes, null_seed)
        }
        CodebookMode::RandomInjective { .. } | CodebookMode::QuotaOnlyInjective { .. } => {
            unreachable!("handled before phase loop")
        }
    };

    let mut phases_q29 = Vec::with_capacity(vocab * blocks * 4);
    let mut spinors_s3_q30 = Vec::with_capacity(vocab * blocks * 2);
    // Exact integer alignment table: per token, per block, 16 root scores.
    let mut block_alignments: Vec<Vec<[i64; 16]>> = Vec::with_capacity(vocab);

    for (token_idx, &prime) in primes.iter().enumerate() {
        let ln_p = f64::from(prime).ln();
        let mut token_alignments = Vec::with_capacity(blocks);
        for block in 0..blocks {
            let mut complex_re = [0.0f64; 4];
            let mut complex_im = [0.0f64; 4];
            for c in 0..4 {
                let channel = (block * 4 + c) % ZETA_ZEROS.len();
                let gamma = match mode {
                    CodebookMode::PrimeZeta => ZETA_ZEROS[channel],
                    CodebookMode::TangentNull { delta } => {
                        if !delta.is_finite() || delta == 0.0 {
                            return Err(invalid("tangent null delta must be finite and non-zero"));
                        }
                        ZETA_ZEROS[channel] + delta
                    }
                    CodebookMode::NormalNull { seed: null_seed } => {
                        // Off-critical-line phase + seeded harmonic scramble
                        let offset = f64::from((null_seed.wrapping_add(c as u32) % 7) + 1) * 0.37;
                        ZETA_ZEROS[(channel + 17 + (null_seed as usize % 31)) % ZETA_ZEROS.len()]
                            + offset
                    }
                    CodebookMode::RandomInjective { .. }
                    | CodebookMode::QuotaOnlyInjective { .. } => {
                        unreachable!("handled before phase loop")
                    }
                };
                let theta = wrap_to_pi(gamma * ln_p);
                let q29 = PhaseQ29::from_radians(theta)?;
                phases_q29.push(q29.raw());

                // Off-critical-line amplitude perturbation for NormalNull (sigma != 1/2);
                // unit modulus on the critical line (sigma = 1/2) for PrimeZeta and TangentNull.
                let modulus = match mode {
                    CodebookMode::PrimeZeta | CodebookMode::TangentNull { .. } => 1.0,
                    CodebookMode::NormalNull { seed: null_seed } => {
                        let sigma_shift = match (c + (token_idx % 4) + (null_seed as usize % 4)) % 4
                        {
                            0 => 0.35,
                            1 => -0.35,
                            2 => 0.20,
                            _ => -0.20,
                        };
                        f64::from(prime).powf(sigma_shift)
                    }
                    CodebookMode::RandomInjective { .. }
                    | CodebookMode::QuotaOnlyInjective { .. } => {
                        unreachable!("handled before phase loop")
                    }
                };
                complex_re[c] = modulus * theta.cos();
                complex_im[c] = modulus * theta.sin();
            }

            // Canonical realification C^4 -> R^4 x R^4 and immediate Q1.30 integer quantization:
            let left_s3 =
                UnitS3Q30::from_r4([complex_re[0], complex_im[0], complex_re[1], complex_im[1]])?;
            let right_s3 =
                UnitS3Q30::from_r4([complex_re[2], complex_im[2], complex_re[3], complex_im[3]])?;
            spinors_s3_q30.push(left_s3.raw());
            spinors_s3_q30.push(right_s3.raw());

            // Integer boundary crossed: all subsequent computations are exact integer operations.
            let q_b = embed_s3_pair_into_e8_planes(left_s3, right_s3);
            let mut scores = [0i64; 16];
            for (r, score) in scores.iter_mut().enumerate() {
                *score = exact_root_alignment(&q_b, r);
            }
            token_alignments.push(scores);
        }
        block_alignments.push(token_alignments);
    }

    // Exact integer E8^B orbit snap with writer-cell balance and multi-block non-aliasing:
    // - Writer product cell is (first_block_root, last_block_root) in 0..256; capacity is ceil(V / 256).
    // - When 16^B >= V (e.g., B >= 3 for V = 4096, or B >= 2 for V <= 256), every token is
    //   assigned a strictly unique B-block root vector (0 full-vector collisions).
    let cell_quota = vocab.div_ceil(256);
    let mut cell_counts = [0usize; 256];
    let enforce_unique_full_vector = blocks >= 3 || vocab <= (1usize << (4 * blocks));
    let mut used_vectors: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut root_indices = Vec::with_capacity(vocab * blocks);
    let mut table = Vec::with_capacity(vocab * blocks * 8);

    for token_alignments in &block_alignments {
        // Rank the 256 outer (first_block, last_block) writer cells by exact integer score.
        let mut outer_candidates: Vec<(i64, u8, u8)> = Vec::with_capacity(256);
        for r_first in 0u8..16 {
            for r_last in 0u8..16 {
                let score = token_alignments[0][usize::from(r_first)]
                    + token_alignments[blocks - 1][usize::from(r_last)];
                outer_candidates.push((score, r_first, r_last));
            }
        }
        outer_candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| (a.1, a.2).cmp(&(b.1, b.2))));

        // Pre-sort the 16 roots for each interior block b in 1..blocks-1.
        let mut interior_sorted: Vec<[u8; 16]> = Vec::with_capacity(blocks.saturating_sub(2));
        for alignment in token_alignments
            .iter()
            .take(blocks.saturating_sub(1))
            .skip(1)
        {
            let mut order = [0u8; 16];
            for (idx, slot) in order.iter_mut().enumerate() {
                *slot = idx as u8;
            }
            order.sort_by(|&ra, &rb| {
                alignment[usize::from(rb)]
                    .cmp(&alignment[usize::from(ra)])
                    .then_with(|| ra.cmp(&rb))
            });
            interior_sorted.push(order);
        }

        let interior_pairs: Vec<(i64, u8, u8)> = if blocks >= 4 {
            let mut pairs = Vec::with_capacity(256);
            for &r1 in &interior_sorted[0] {
                for &r2 in &interior_sorted[1] {
                    let s =
                        token_alignments[1][usize::from(r1)] + token_alignments[2][usize::from(r2)];
                    pairs.push((s, r1, r2));
                }
            }
            pairs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| (a.1, a.2).cmp(&(b.1, b.2))));
            pairs
        } else {
            Vec::new()
        };

        let mut chosen: Option<Vec<u8>> = None;
        for &(_outer_score, r_first, r_last) in &outer_candidates {
            let cell = usize::from(r_first) * 16 + usize::from(r_last);
            if cell_counts[cell] >= cell_quota {
                continue;
            }
            if blocks == 2 {
                let candidate = vec![r_first, r_last];
                if !enforce_unique_full_vector || !used_vectors.contains(&candidate) {
                    chosen = Some(candidate);
                    break;
                }
            } else if blocks == 3 {
                for &r1 in &interior_sorted[0] {
                    let candidate = vec![r_first, r1, r_last];
                    if !used_vectors.contains(&candidate) {
                        chosen = Some(candidate);
                        break;
                    }
                }
                if chosen.is_some() {
                    break;
                }
            } else {
                for &(_s, r1, r2) in &interior_pairs {
                    let mut candidate = Vec::with_capacity(blocks);
                    candidate.push(r_first);
                    candidate.push(r1);
                    candidate.push(r2);
                    for extra in 3..blocks - 1 {
                        candidate.push(interior_sorted[extra - 1][0]);
                    }
                    candidate.push(r_last);
                    if !used_vectors.contains(&candidate) {
                        chosen = Some(candidate);
                        break;
                    }
                }
                if chosen.is_some() {
                    break;
                }
            }
        }

        let selected = chosen.ok_or_else(|| {
            invalid(
                "failed to assign non-aliasing E8 multi-block root vector within orbit capacity",
            )
        })?;
        let cell = usize::from(selected[0]) * 16 + usize::from(selected[blocks - 1]);
        cell_counts[cell] += 1;
        if enforce_unique_full_vector {
            used_vectors.insert(selected.clone());
        }
        for &r in &selected {
            root_indices.push(r);
            for &coord in &geometry::ROOTS[usize::from(r)] {
                table.push(i32::from(coord));
            }
        }
    }

    let unique_phase_vectors = phases_q29
        .chunks_exact(blocks * 4)
        .collect::<BTreeSet<_>>()
        .len();
    let unique_spinor_vectors = spinors_s3_q30
        .chunks_exact(blocks * 2)
        .collect::<BTreeSet<_>>()
        .len();
    let unique_table_vectors = table
        .chunks_exact(blocks * 8)
        .collect::<BTreeSet<_>>()
        .len();
    let occupied_writer_cells = cell_counts.iter().filter(|&&c| c > 0).count();

    let table_sha256 = sha256_i32_slice(&table);
    let phases_sha256 = sha256_i32_slice(&phases_q29);
    let flat_spinors: Vec<i32> = spinors_s3_q30.iter().flatten().copied().collect();
    let spinors_sha256 = sha256_i32_slice(&flat_spinors);

    Ok(QuantizedCodebook {
        mode: mode.label(),
        spec,
        seed,
        primes,
        phases_q29,
        spinors_s3_q30,
        root_indices,
        table,
        table_sha256,
        phases_sha256,
        spinors_sha256,
        unique_phase_vectors,
        unique_spinor_vectors,
        unique_table_vectors,
        occupied_writer_cells,
    })
}

/// Return the exact token count in each of the 256 writer product cells
/// `cell = root_0 * 16 + root_{B-1}` for a quantized codebook.
#[allow(dead_code)]
pub fn writer_cell_occupancy(codebook: &QuantizedCodebook) -> [usize; 256] {
    let vocab = codebook.spec.vocabulary as usize;
    let blocks = codebook.spec.blocks as usize;
    let mut counts = [0usize; 256];
    for t in 0..vocab {
        let roots = &codebook.root_indices[t * blocks..(t + 1) * blocks];
        let cell = usize::from(roots[0]) * 16 + usize::from(roots[blocks - 1]);
        counts[cell] += 1;
    }
    counts
}

/// Extract the exact quantization witness for `token_id`.
#[allow(dead_code)]
pub fn token_witness(
    codebook: &QuantizedCodebook,
    token_id: u32,
) -> Result<TokenQuantizationWitness> {
    let vocab = codebook.spec.vocabulary as usize;
    let blocks = codebook.spec.blocks as usize;
    let t = token_id as usize;
    if t >= vocab {
        return Err(invalid("token_id out of vocabulary range"));
    }
    Ok(TokenQuantizationWitness {
        token_id,
        prime_atom: codebook.primes[t],
        phases_q29: codebook.phases_q29[t * blocks * 4..(t + 1) * blocks * 4].to_vec(),
        spinors_s3_q30: codebook.spinors_s3_q30[t * blocks * 2..(t + 1) * blocks * 2].to_vec(),
        root_indices: codebook.root_indices[t * blocks..(t + 1) * blocks].to_vec(),
        table_vector: codebook.table[t * blocks * 8..(t + 1) * blocks * 8].to_vec(),
    })
}

/// Compute exact integer structural separation metrics between two quantized codebooks.
#[allow(dead_code)]
pub fn compare_codebooks(
    left: &QuantizedCodebook,
    right: &QuantizedCodebook,
) -> Result<StructuralSeparation> {
    if left.spec != right.spec {
        return Err(invalid(
            "codebook structural comparison requires matching Spec",
        ));
    }
    let vocab = left.spec.vocabulary as usize;
    let blocks = left.spec.blocks as usize;
    let width = blocks * 8;

    let mut differing_tokens = 0usize;
    let mut differing_root_blocks = 0usize;
    let mut coordinate_hamming_distance = 0u64;
    let mut coordinate_l1_distance = 0u64;
    let mut differing_writer_cells = 0usize;

    for t in 0..vocab {
        let l_vec = &left.table[t * width..(t + 1) * width];
        let r_vec = &right.table[t * width..(t + 1) * width];
        if l_vec != r_vec {
            differing_tokens += 1;
        }
        let l_roots = &left.root_indices[t * blocks..(t + 1) * blocks];
        let r_roots = &right.root_indices[t * blocks..(t + 1) * blocks];
        for b in 0..blocks {
            if l_roots[b] != r_roots[b] {
                differing_root_blocks += 1;
            }
        }
        let l_cell = usize::from(l_roots[0]) * 16 + usize::from(l_roots[blocks - 1]);
        let r_cell = usize::from(r_roots[0]) * 16 + usize::from(r_roots[blocks - 1]);
        if l_cell != r_cell {
            differing_writer_cells += 1;
        }
        for (&a, &b) in l_vec.iter().zip(r_vec.iter()) {
            if a != b {
                coordinate_hamming_distance += 1;
            }
            coordinate_l1_distance += u64::from(a.abs_diff(b));
        }
    }

    let mut phase_q29_l1_distance = 0u128;
    for (&a, &b) in left.phases_q29.iter().zip(right.phases_q29.iter()) {
        phase_q29_l1_distance += u128::from(a.abs_diff(b));
    }

    Ok(StructuralSeparation {
        left_mode: left.mode.clone(),
        right_mode: right.mode.clone(),
        vocabulary: left.spec.vocabulary,
        blocks: left.spec.blocks,
        differing_tokens,
        differing_root_blocks,
        coordinate_hamming_distance,
        coordinate_l1_distance,
        differing_writer_cells,
        phase_q29_l1_distance,
    })
}

/// Inspect PTX text for zero floating-point / tensor-core instructions and exact 11-entry inventory.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub fn verify_zero_float_ptx(ptx: &str) -> Result<usize> {
    const ENTRIES: [&str; 11] = [
        "discrete_init",
        "discrete_encode",
        "discrete_forward",
        "discrete_counterfactual",
        "discrete_choose",
        "discrete_apply",
        "discrete_validate",
        "discrete_evaluate",
        "discrete_evaluation_aggregate",
        "discrete_count",
        "discrete_count_score",
    ];
    let mut entries = BTreeSet::new();
    let mut expect_entry = false;
    for (line_number, line) in ptx.lines().enumerate() {
        let instruction = line.split("//").next().unwrap_or_default();
        for token in instruction
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            .filter(|token| !token.is_empty())
        {
            let components: Vec<_> = token.split('.').collect();
            if components.iter().any(|component| {
                matches!(
                    *component,
                    "f16"
                        | "f16x2"
                        | "f32"
                        | "f64"
                        | "bf16"
                        | "bf16x2"
                        | "tf32"
                        | "e4m3"
                        | "e5m2"
                        | "mma"
                        | "wmma"
                        | "wgmma"
                        | "tcgen05"
                        | "tensor"
                        | "tensormap"
                )
            }) || token.starts_with("ex2.approx")
            {
                return Err(invalid(format!(
                    "integer PTX admission rejected {token} at line {}",
                    line_number + 1
                )));
            }
            if expect_entry {
                if token.contains('.') || !entries.insert(token.to_owned()) {
                    return Err(invalid(format!("invalid or duplicate PTX entry {token}")));
                }
                expect_entry = false;
            } else if token == ".entry" {
                expect_entry = true;
            }
        }
    }
    if expect_entry {
        return Err(invalid("unterminated PTX entry declaration"));
    }
    let expected: BTreeSet<_> = ENTRIES.iter().map(|e| (*e).to_owned()).collect();
    if entries != expected {
        return Err(invalid(
            "PTX entry inventory differs from expected 11 kernels",
        ));
    }
    Ok(entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_rank_bijection_maps_v4096_to_exact_primes() -> Result<()> {
        let primes = first_n_primes(4096)?;
        assert_eq!(primes.len(), 4096);
        assert_eq!(primes[0], 2);
        assert_eq!(primes[1], 3);
        assert_eq!(primes[2], 5);
        assert_eq!(primes[3], 7);
        assert_eq!(primes[255], 1619);
        assert_eq!(primes[4095], 38873);
        Ok(())
    }

    #[test]
    fn v4096_codebook_is_bit_exact_across_named_seeds_and_collision_free() -> Result<()> {
        let spec_b4 = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 64,
            kept_cells: 8,
        };
        let cb0 = generate_codebook(spec_b4, 0, CodebookMode::PrimeZeta)?;
        let cb1 = generate_codebook(spec_b4, 1, CodebookMode::PrimeZeta)?;
        let cb2 = generate_codebook(spec_b4, 2, CodebookMode::PrimeZeta)?;

        // 1. Bit-exact across named seeds 0, 1, 2
        assert_eq!(cb0.table, cb1.table);
        assert_eq!(cb1.table, cb2.table);
        assert_eq!(cb0.table_sha256, cb1.table_sha256);
        assert_eq!(cb0.phases_sha256, cb2.phases_sha256);
        assert_eq!(cb0.spinors_sha256, cb2.spinors_sha256);

        // 2. Strict non-aliasing across all 4,096 vocabulary tokens (0 collisions)
        assert_eq!(cb0.unique_phase_vectors, 4096);
        assert_eq!(cb0.unique_spinor_vectors, 4096);
        assert_eq!(cb0.unique_table_vectors, 4096);
        assert_eq!(cb0.occupied_writer_cells, 256);

        // 3. Every 8-int block in Snapshot.table is an exact E8 orbit root in geometry::ROOTS
        for block_slice in cb0.table.chunks_exact(8) {
            let root_i8: [i8; 8] = block_slice
                .iter()
                .map(|&v| i8::try_from(v).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            assert!(geometry::ROOTS.contains(&root_i8));
        }

        // Also check B=2 (fixture-tokens V4096 B2 context64 from pre-registration sheet)
        let spec_b2 = Spec {
            vocabulary: 4096,
            blocks: 2,
            context: 64,
            kept_cells: 8,
        };
        let b2_0 = generate_codebook(spec_b2, 0, CodebookMode::PrimeZeta)?;
        let b2_1 = generate_codebook(spec_b2, 1, CodebookMode::PrimeZeta)?;
        let b2_2 = generate_codebook(spec_b2, 2, CodebookMode::PrimeZeta)?;
        assert_eq!(b2_0.table, b2_1.table);
        assert_eq!(b2_1.table, b2_2.table);
        assert_eq!(b2_0.unique_phase_vectors, 4096);
        assert_eq!(b2_0.unique_spinor_vectors, 4096);
        // At B=2, ROOTS^2 has 16^2 = 256 total orbit root pairs; all 256 are populated
        assert_eq!(b2_0.unique_table_vectors, 256);
        assert_eq!(b2_0.occupied_writer_cells, 256);

        Ok(())
    }

    #[test]
    fn tangent_and_normal_nulls_separate_structurally_from_prime_zeta() -> Result<()> {
        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 64,
            kept_cells: 8,
        };
        let prime_zeta = generate_codebook(spec, 0, CodebookMode::PrimeZeta)?;
        let tangent = generate_codebook(
            spec,
            0,
            CodebookMode::TangentNull {
                delta: TANGENT_NULL_DELTA,
            },
        )?;
        let normal0 = generate_codebook(spec, 0, CodebookMode::NormalNull { seed: 0 })?;
        let normal1 = generate_codebook(spec, 1, CodebookMode::NormalNull { seed: 1 })?;
        let normal2 = generate_codebook(spec, 2, CodebookMode::NormalNull { seed: 2 })?;

        // Both nulls also maintain 0 full-vector collisions on V4096 B4
        assert_eq!(tangent.unique_table_vectors, 4096);
        assert_eq!(normal0.unique_table_vectors, 4096);
        assert_eq!(normal1.unique_table_vectors, 4096);
        assert_eq!(normal2.unique_table_vectors, 4096);

        // Normal null is bit-exact reproducible per seed and distinct across seeds 0, 1, 2
        let normal0_again = generate_codebook(spec, 0, CodebookMode::NormalNull { seed: 0 })?;
        assert_eq!(normal0.table, normal0_again.table);
        assert_ne!(normal0.table, normal1.table);
        assert_ne!(normal1.table, normal2.table);

        let vs_tangent = compare_codebooks(&prime_zeta, &tangent)?;
        let vs_normal0 = compare_codebooks(&prime_zeta, &normal0)?;
        let tangent_vs_normal0 = compare_codebooks(&tangent, &normal0)?;

        // Non-trivial structural separation (>95% of 4,096 tokens move across all pairs)
        assert!(
            vs_tangent.differing_tokens > 3900,
            "expected >3900 differing tokens vs tangent null, got {}",
            vs_tangent.differing_tokens
        );
        assert!(
            vs_normal0.differing_tokens > 3900,
            "expected >3900 differing tokens vs normal null, got {}",
            vs_normal0.differing_tokens
        );
        assert!(
            tangent_vs_normal0.differing_tokens > 3900,
            "expected >3900 differing tokens between tangent and normal nulls, got {}",
            tangent_vs_normal0.differing_tokens
        );
        assert!(vs_tangent.coordinate_hamming_distance > 40_000);
        assert!(vs_normal0.coordinate_hamming_distance > 40_000);
        Ok(())
    }

    #[test]
    fn random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants(
    ) -> Result<()> {
        let spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        let prime_zeta = generate_codebook(spec, 0, CodebookMode::PrimeZeta)?;
        let tangent = generate_codebook(
            spec,
            0,
            CodebookMode::TangentNull {
                delta: TANGENT_NULL_DELTA,
            },
        )?;
        let normal0 = generate_codebook(spec, 0, CodebookMode::NormalNull { seed: 0 })?;

        let random0 = generate_codebook(spec, 0, CodebookMode::RandomInjective { seed: 0 })?;
        let random1 = generate_codebook(spec, 1, CodebookMode::RandomInjective { seed: 1 })?;
        let random2 = generate_codebook(spec, 2, CodebookMode::RandomInjective { seed: 2 })?;

        let quota0 = generate_codebook(spec, 0, CodebookMode::QuotaOnlyInjective { seed: 0 })?;
        let quota1 = generate_codebook(spec, 1, CodebookMode::QuotaOnlyInjective { seed: 1 })?;
        let quota2 = generate_codebook(spec, 2, CodebookMode::QuotaOnlyInjective { seed: 2 })?;

        // 1. Bit-exact reproducibility per seed and distinctness across seeds 0, 1, 2
        let random0_again = generate_codebook(spec, 0, CodebookMode::RandomInjective { seed: 0 })?;
        let quota0_again =
            generate_codebook(spec, 0, CodebookMode::QuotaOnlyInjective { seed: 0 })?;
        assert_eq!(random0.table, random0_again.table);
        assert_eq!(random0.table_sha256, random0_again.table_sha256);
        assert_eq!(quota0.table, quota0_again.table);
        assert_eq!(quota0.table_sha256, quota0_again.table_sha256);
        assert_ne!(random0.table, random1.table);
        assert_ne!(random1.table, random2.table);
        assert_ne!(quota0.table, quota1.table);
        assert_ne!(quota1.table, quota2.table);

        // 2. Full-vector injectivity (4096 / 4096 distinct vectors) and valid E8 roots
        for cb in [&random0, &random1, &random2, &quota0, &quota1, &quota2] {
            assert_eq!(cb.unique_table_vectors, 4096);
            assert_eq!(cb.unique_phase_vectors, 4096);
            assert_eq!(cb.unique_spinor_vectors, 4096);
            for block_slice in cb.table.chunks_exact(8) {
                let root_i8: [i8; 8] = block_slice
                    .iter()
                    .map(|&v| i8::try_from(v).unwrap())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                assert!(geometry::ROOTS.contains(&root_i8));
            }
        }

        // 3. Writer-cell occupancy: QuotaOnlyInjective has exactly 16 tokens in each of
        //    the 256 writer cells (matching PrimeZeta), whereas RandomInjective has no
        //    cell quota and exhibits natural multinomial occupancy dispersion.
        assert_eq!(writer_cell_occupancy(&prime_zeta), [16usize; 256]);
        for q_cb in [&quota0, &quota1, &quota2] {
            assert_eq!(q_cb.occupied_writer_cells, 256);
            assert_eq!(writer_cell_occupancy(q_cb), [16usize; 256]);
        }
        for r_cb in [&random0, &random1, &random2] {
            let occ = writer_cell_occupancy(r_cb);
            let total: usize = occ.iter().sum();
            let min_occ = *occ.iter().min().unwrap();
            let max_occ = *occ.iter().max().unwrap();
            assert_eq!(total, 4096);
            assert!(
                min_occ < 16 && max_occ > 16,
                "random_injective must not enforce flat 16 quota: min={min_occ}, max={max_occ}"
            );
        }

        // 4. Structural separation (>3900 / 4096 differing tokens, >40000 coordinate Hamming)
        for other in [&prime_zeta, &tangent, &normal0] {
            let sep_random = compare_codebooks(other, &random0)?;
            let sep_quota = compare_codebooks(other, &quota0)?;
            assert!(sep_random.differing_tokens > 3900);
            assert!(sep_quota.differing_tokens > 3900);
            assert!(sep_random.coordinate_hamming_distance > 40_000);
            assert!(sep_quota.coordinate_hamming_distance > 40_000);
        }
        let random_vs_quota = compare_codebooks(&random0, &quota0)?;
        assert!(random_vs_quota.differing_tokens > 3900);
        assert!(random_vs_quota.coordinate_hamming_distance > 40_000);

        // 5. Coverage census invariance proof: raw alias ceiling and within-cell tie mass
        //    are algebraically identical across all 4096-injective codebooks, whereas
        //    exported cell crowding separates QuotaOnlyInjective (30,720 collision pairs,
        //    max occupancy 16) from RandomInjective (>30,720 collision pairs, max > 16).
        let synthetic_dev_targets: Vec<u32> = (0..14_287u32).map(|i| (i * 73) % 4096).collect();
        let pz_cells: Vec<u16> = (0..4096)
            .map(|t| {
                (u16::from(prime_zeta.root_indices[t * 4]) * 16)
                    + u16::from(prime_zeta.root_indices[t * 4 + 3])
            })
            .collect();
        let q0_cells: Vec<u16> = (0..4096)
            .map(|t| {
                (u16::from(quota0.root_indices[t * 4]) * 16)
                    + u16::from(quota0.root_indices[t * 4 + 3])
            })
            .collect();
        let r0_cells: Vec<u16> = (0..4096)
            .map(|t| {
                (u16::from(random0.root_indices[t * 4]) * 16)
                    + u16::from(random0.root_indices[t * 4 + 3])
            })
            .collect();

        let pz_census = uor_r4_discrete::coverage_census::census(
            &prime_zeta.table,
            spec,
            &synthetic_dev_targets,
            Some(&pz_cells),
            Some(500),
        )?;
        let q0_census = uor_r4_discrete::coverage_census::census(
            &quota0.table,
            spec,
            &synthetic_dev_targets,
            Some(&q0_cells),
            Some(500),
        )?;
        let r0_census = uor_r4_discrete::coverage_census::census(
            &random0.table,
            spec,
            &synthetic_dev_targets,
            Some(&r0_cells),
            Some(500),
        )?;

        for report in [&pz_census, &q0_census, &r0_census] {
            assert_eq!(report["distinct_vectors"], 4096);
            assert_eq!(report["dev_alias_ceiling_rows"], 14_287);
            assert_eq!(report["within_cell_tie_mass"]["tokens_in_alias_groups"], 0);
            assert_eq!(
                report["within_cell_tie_mass"]["dev_rows_targeting_non_representative"],
                0
            );
        }
        assert_eq!(pz_census["cells"]["max_cell_occupancy"], 16);
        assert_eq!(pz_census["cells"]["cell_collision_pairs"], 30_720);
        assert_eq!(q0_census["cells"]["max_cell_occupancy"], 16);
        assert_eq!(q0_census["cells"]["cell_collision_pairs"], 30_720);
        assert!(r0_census["cells"]["max_cell_occupancy"].as_u64().unwrap() > 16);
        assert!(r0_census["cells"]["cell_collision_pairs"].as_u64().unwrap() > 30_720);

        // 6. Pinned B=4 table_sha256 preservation and fairness-guard / B=5 shuffle checks
        assert_eq!(
            quota0.table_sha256,
            "0098d4c3df8ce101e4a2dda8d8d28a29ac64f6ed3d521430504d0bb03ffbc18f"
        );
        assert_eq!(
            random0.table_sha256,
            "35c8da6250a5d986ffa5f6758cbf761b45af6fa24d3893114b9693c1b0d9ee37"
        );
        let non_multiple_spec = Spec {
            vocabulary: 500,
            ..spec
        };
        assert!(generate_codebook(
            non_multiple_spec,
            0,
            CodebookMode::QuotaOnlyInjective { seed: 0 }
        )
        .is_err());
        let two_block_spec = Spec {
            vocabulary: 256,
            blocks: 2,
            ..spec
        };
        assert!(generate_codebook(
            two_block_spec,
            0,
            CodebookMode::QuotaOnlyInjective { seed: 0 }
        )
        .is_err());
        assert!(
            generate_codebook(two_block_spec, 0, CodebookMode::RandomInjective { seed: 0 })
                .is_err()
        );
        let five_block_spec = Spec { blocks: 5, ..spec };
        let quota_b5 = generate_codebook(
            five_block_spec,
            0,
            CodebookMode::QuotaOnlyInjective { seed: 0 },
        )?;
        assert_eq!(quota_b5.unique_table_vectors, 4096);
        assert_eq!(writer_cell_occupancy(&quota_b5), [16usize; 256]);

        Ok(())
    }

    #[test]
    fn zero_float_ptx_verifier_accepts_integer_entries_and_rejects_float_tokens() -> Result<()> {
        let valid_ptx = "\
            .visible .entry discrete_init() { add.s32 %r1, %r2, %r3; ret; }\n\
            .visible .entry discrete_encode() { ret; }\n\
            .visible .entry discrete_forward() { ret; }\n\
            .visible .entry discrete_counterfactual() { ret; }\n\
            .visible .entry discrete_choose() { ret; }\n\
            .visible .entry discrete_apply() { ret; }\n\
            .visible .entry discrete_validate() { ret; }\n\
            .visible .entry discrete_evaluate() { ret; }\n\
            .visible .entry discrete_evaluation_aggregate() { ret; }\n\
            .visible .entry discrete_count() { ret; }\n\
            .visible .entry discrete_count_score() { ret; }\n";
        assert_eq!(verify_zero_float_ptx(valid_ptx)?, 11);

        let invalid_ptx = format!("{valid_ptx}\nmul.f32 %f1, %f2, %f3;\n");
        assert!(verify_zero_float_ptx(&invalid_ptx).is_err());
        Ok(())
    }

    #[test]
    fn driver_source_identity_changes_with_rank_objective_metadata() {
        assert_eq!(
            uor_r4_discrete::training::driver_source_sha256(),
            uor_r4_discrete::driver_source_pin::DRIVER_SOURCE_SHA256
        );
    }
}
