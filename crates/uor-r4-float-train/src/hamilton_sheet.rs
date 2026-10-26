//! Hamilton gate offline-float sheet (`hamilton-gate-offline-float-sheet`, `#2824`).
//!
//! Executes the pre-registered 4-arm × 2-seed × 256-step D37 offline-float sheet in
//! `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md`:
//! 1. `Hamilton_Active`: continuous `Gate::Hamilton` (`q = g / sqrt(|g|^2 + 1e-12)`, `out = q (x) u`)
//!    with exact CUDA `hamilton_gate_fwd` and `hamilton_gate_bwd`.
//! 2. `Hamilton_IcosianHard`: hard-forward 120-element icosian snap (`q_snap in I_120`, the 120
//!    unit roots of `H4_ROOT_COEFFICIENTS`) with the straight-through `S^3` tangent-space bridge.
//! 3. `IdentityGate_Control`: un-gated pass-through control (`q = (1, 0, 0, 0)`, `out = u`).
//! 4. `RandomGate_Null`: deterministic norm-preserving random unit-icosian control (`q_rand in I_120`).
//!
//! Evaluates each arm on the `14,287` eligible response targets of `ft/dev` (`400` documents,
//! `42,856` payload tokens) in both continuous `f32` and D11-compatible integer export
//! (120-icosian gate snap + signed `i16` Q12 fixed-point `NativeVectorWriter`, verified against
//! an exact `i32` integer L1 oracle), compares against `TrainUnigram_CountTable` (`1,226 / 14,287`
//! top-1, `4,060 / 14,287` top-8), and records empirical gate signal suppression (`|g|_2`,
//! `rho(g, u) = ||out||_2 / ||u||_2`, `1 / r`, and downstream projection norm ratio
//! `||P_down(q (x) u)||_2 / ||P_down(u)||_2`).

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[cfg(feature = "cuda")]
use candle_core::backend::BackendStorage;
#[cfg(feature = "cuda")]
use candle_core::{CudaDevice, CudaStorage, DType, Layout, Shape};
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};

use crate::geometric_stack::{
    Gate, Objective, QuaternionWeights, RecGates, RotationGroup, StackAdamW, StackArch,
    StackConfig, StackModel,
};
use crate::native_vector_writer::{NativeVectorWriter, WriterSpec, CELLS};
use crate::{invalid, sha256_bytes, Result};

/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16`.
pub const PINNED_TRAIN_TOKENS_SHA256: &str =
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8`.
pub const PINNED_TRAIN_MASK_SHA256: &str =
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16`.
pub const PINNED_DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8`.
pub const PINNED_DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/tokenizer.json`.
pub const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
/// Pinned SHA-256 of `src/cuda_stack.cu`.
pub const CUDA_STACK_CU_SHA256: &str =
    "eb9a3fd870c74919e96bf6c2974717a5ca1914722f98f469bf3e5fb462d478bd";

pub const UORT_HEADER_BYTES: usize = 64;
pub const SHEET_WIDTH: usize = 256;
pub const SHEET_HEADS: usize = 4;
pub const SHEET_MLP_HIDDEN: usize = 384;
pub const SHEET_TRAIN_BATCH: usize = 4;
pub const SHEET_EVAL_BATCH: usize = 16;
pub const SHEET_CONTEXT: usize = 192;
pub const SHEET_VOCAB: usize = 4096;
pub const SHEET_BOS_ID: u32 = 0;
pub const SHEET_KEPT_CELLS: usize = 8;
pub const SHEET_RIVALS: usize = 8;
pub const SHEET_TRAIN_BATCHES: usize = 64;
pub const SHEET_STEPS: usize = 256;
pub const SHEET_BASE_LR: f64 = 0.002;
pub const SHEET_DECAY_LR_MULT: f64 = 20.0;
pub const SHEET_WEIGHT_DECAY: f64 = 0.1;
pub const SHEET_CLIP: f64 = 1.0;

pub const EXPECTED_DEV_TOKENS: usize = 42_856;
pub const EXPECTED_DEV_DOCS: usize = 400;
pub const EXPECTED_DEV_ELIGIBLE_TARGETS: usize = 14_287;
pub const EXPECTED_TRAIN_TOKENS: usize = 7_512_797;
pub const EXPECTED_TRAIN_ELIGIBLE_TARGETS: usize = 2_425_925;

pub const K16_NOISE_TOP1_ROWS: usize = 398;
pub const K16_NOISE_TOP8_ROWS: usize = 748;
pub const K16_NOISE_ADM_ROWS: usize = 930;

/// Q12 fixed-point scale (`4096 = 2^12`) and signed `i16` clamp bounds (`[-32768, 32767]`).
pub const Q12_SCALE: f32 = 4096.0;
pub const Q12_I16_MIN: f32 = -32768.0;
pub const Q12_I16_MAX: f32 = 32767.0;

/// Exact 120 signed H4 root coefficients (`[[[i8; 2]; 4]; 120]`), where `[a, b]` denotes `(a + b*phi)/2`.
/// Index `0` is `-1 = (-1, 0, 0, 0)` and index `1` is the identity `+1 = (+1, 0, 0, 0)`.
pub const H4_ROOT_COEFFICIENTS: [[[i8; 2]; 4]; 120] = historical_root_coefficients();

const EVEN_PERMUTATIONS: [[usize; 4]; 12] = [
    [0, 1, 2, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 2, 1, 0],
];

const fn historical_root_coefficients() -> [[[i8; 2]; 4]; 120] {
    let mut roots = [[[0i8; 2]; 4]; 120];
    let mut index = 0;
    let mut axis = 0;
    while axis < 4 {
        roots[index][axis] = [-2, 0];
        roots[index + 1][axis] = [2, 0];
        index += 2;
        axis += 1;
    }
    let mut signs = 0;
    while signs < 16 {
        axis = 0;
        while axis < 4 {
            roots[index][axis] = [if signs & (1 << axis) == 0 { -1 } else { 1 }, 0];
            axis += 1;
        }
        signs += 1;
        index += 1;
    }
    let mut permutation = 0;
    while permutation < 12 {
        signs = 0;
        while signs < 8 {
            let s1 = if signs & 4 == 0 { -1 } else { 1 };
            let s2 = if signs & 2 == 0 { -1 } else { 1 };
            let s3 = if signs & 1 == 0 { -1 } else { 1 };
            let signed = [[0, 0], [s1, 0], [0, s2], [-s3, s3]];
            axis = 0;
            while axis < 4 {
                roots[index][axis] = signed[EVEN_PERMUTATIONS[permutation][axis]];
                axis += 1;
            }
            index += 1;
            signs += 1;
        }
        permutation += 1;
    }
    roots
}

/// SHA-256 of the 960 signed coefficient bytes in `H4_ROOT_COEFFICIENTS`.
pub fn h4_coefficients_sha256() -> String {
    let mut bytes = Vec::with_capacity(960);
    for root in H4_ROOT_COEFFICIENTS {
        for coord in root {
            bytes.push(coord[0] as u8);
            bytes.push(coord[1] as u8);
        }
    }
    sha256_bytes(&bytes)
}

/// Evaluates the 120 unit icosians in `f64`: `(a + b * phi) / 2` where `phi = (1 + sqrt(5)) / 2`.
pub fn h4_unit_roots_f64() -> [[f64; 4]; 120] {
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    std::array::from_fn(|idx| {
        std::array::from_fn(|axis| {
            let [a, b] = H4_ROOT_COEFFICIENTS[idx][axis];
            0.5 * (f64::from(a) + f64::from(b) * phi)
        })
    })
}

/// Evaluates the 120 unit icosians in `f32`.
pub fn h4_unit_roots_f32() -> [[f32; 4]; 120] {
    let roots_f64 = h4_unit_roots_f64();
    std::array::from_fn(|idx| std::array::from_fn(|axis| roots_f64[idx][axis] as f32))
}

#[inline]
fn h4_family_candidates(lane: &[i32; 4]) -> [u8; 14] {
    let magnitudes = lane.map(|v| i64::from(v).unsigned_abs());
    let positive = lane.map(|v| u8::from(v > 0));
    let mut candidates = [0u8; 14];
    let mut largest_axis = 0;
    for axis in 1..4 {
        if magnitudes[axis] > magnitudes[largest_axis] {
            largest_axis = axis;
        }
    }
    candidates[0] = ((largest_axis as u8) << 1) + positive[largest_axis];
    candidates[1] = 8;
    for (axis, bit) in positive.iter().enumerate() {
        candidates[1] += *bit << axis;
    }
    let positive_mask = candidates[1] - 8;
    for (block, permutation) in EVEN_PERMUTATIONS.iter().enumerate() {
        let mut mask = positive_mask;
        let mut axis = 0;
        let mut signs = 0u8;
        while mask != 0 {
            if mask & 1 != 0 {
                let base_coord = permutation[axis];
                if base_coord != 0 {
                    signs |= 1 << (3 - base_coord);
                }
            }
            mask >>= 1;
            axis += 1;
        }
        candidates[block + 2] = 24 + ((block as u8) << 3) + signs;
    }
    candidates
}

#[inline]
fn h4_score_i32(lane: &[i32; 4], root: &[[i8; 2]; 4]) -> [i64; 2] {
    let mut a = 0i64;
    let mut b = 0i64;
    for (val, coeff) in lane.iter().zip(root) {
        a += i64::from(*val) * i64::from(coeff[0]);
        b += i64::from(*val) * i64::from(coeff[1]);
    }
    [a, b]
}

#[inline]
fn h4_score_diff_order([a, b]: [i64; 2]) -> Ordering {
    let rational = (a << 1) + b;
    if b == 0 {
        return rational.cmp(&0);
    }
    if rational == 0 {
        return b.cmp(&0);
    }
    if (rational < 0) == (b < 0) {
        return rational.cmp(&0);
    }
    let mag_sq = u128::from(rational.unsigned_abs()) * u128::from(rational.unsigned_abs());
    let b_sq = u128::from(b.unsigned_abs()) * u128::from(b.unsigned_abs());
    let irr_sq = (b_sq << 2) + b_sq;
    match mag_sq.cmp(&irr_sq) {
        Ordering::Greater => rational.cmp(&0),
        Ordering::Less => b.cmp(&0),
        Ordering::Equal => Ordering::Equal,
    }
}

/// Exact signed H4 root classification on common-scale `i32` 4-lanes (`uor_r4_integer::h4_classifier::signed_h4_code_i32`).
/// All-zero lane maps to identity code `1`; exact ties pick the lowest historical index in `0..120`.
#[inline]
pub fn signed_h4_code_i32(lane: [i32; 4]) -> u8 {
    if lane == [0; 4] {
        return 1;
    }
    let candidates = h4_family_candidates(&lane);
    let mut best_index = candidates[0];
    let mut best_score = h4_score_i32(&lane, &H4_ROOT_COEFFICIENTS[usize::from(best_index)]);
    for &index in candidates.iter().skip(1) {
        let candidate = h4_score_i32(&lane, &H4_ROOT_COEFFICIENTS[usize::from(index)]);
        let diff = [candidate[0] - best_score[0], candidate[1] - best_score[1]];
        if h4_score_diff_order(diff) == Ordering::Greater {
            best_index = index;
            best_score = candidate;
        }
    }
    best_index
}

/// Snaps an `f32` 4-block `g` to its nearest icosian root index `0..120` by Q12 integer classification.
#[inline]
pub fn snap_block_q12_h4_code(g: [f32; 4]) -> u8 {
    if g == [0.0; 4] {
        return 1;
    }
    let max_abs = g[0].abs().max(g[1].abs()).max(g[2].abs()).max(g[3].abs());
    if max_abs <= 1e-30 || !max_abs.is_finite() {
        return 1;
    }
    // Common positive scale so the largest coordinate maps near 2^20 in i32.
    let scale = (1_048_576.0_f64) / f64::from(max_abs);
    let lane = [
        (f64::from(g[0]) * scale).round() as i32,
        (f64::from(g[1]) * scale).round() as i32,
        (f64::from(g[2]) * scale).round() as i32,
        (f64::from(g[3]) * scale).round() as i32,
    ];
    signed_h4_code_i32(lane)
}

/// Snaps an `f32` 4-block `g` by direct `f32` inner product against `h4_unit_roots_f32()` (lowest-index tie-break, `0 -> 1`).
#[inline]
pub fn snap_block_f32_dot_code(g: [f32; 4], roots: &[[f32; 4]; 120]) -> u8 {
    if g == [0.0; 4] {
        return 1;
    }
    let mut best_k = 0u8;
    let mut best_dot = f32::NEG_INFINITY;
    for (k, r) in roots.iter().enumerate() {
        let dot = g[0] * r[0] + g[1] * r[1] + g[2] * r[2] + g[3] * r[3];
        if dot > best_dot {
            best_dot = dot;
            best_k = k as u8;
        }
    }
    best_k
}

/// Scalar 4-vector Hamilton product `p (x) q`.
#[inline]
pub fn hamilton_product_f64(p: [f64; 4], q: [f64; 4]) -> [f64; 4] {
    let ([a1, b1, c1, d1], [a2, b2, c2, d2]) = (p, q);
    [
        a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
        a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
        a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
        a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
    ]
}

/// Builds and verifies the 120x120 icosian Cayley table and 120-element inverse table (`I-Icosian-Closure`).
pub fn verify_icosian_closure() -> Result<IcosianClosureCheck> {
    let roots = h4_unit_roots_f64();
    let mut max_unit_norm_err = 0.0f64;
    for r in &roots {
        let norm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + r[3] * r[3]).sqrt();
        max_unit_norm_err = max_unit_norm_err.max((norm - 1.0).abs());
    }
    if max_unit_norm_err > 1e-12 {
        return Err(invalid(format!(
            "icosian unit norm error {max_unit_norm_err} exceeds 1e-12"
        )));
    }
    let mut compose = [[0u8; 120]; 120];
    let mut inverse = [0u8; 120];
    let mut max_closure_dist = 0.0f64;
    for i in 0..120 {
        let conj = [roots[i][0], -roots[i][1], -roots[i][2], -roots[i][3]];
        let mut best_inv = 0usize;
        let mut best_inv_dist = f64::INFINITY;
        for (k, rk) in roots.iter().enumerate() {
            let d = ((conj[0] - rk[0]).powi(2)
                + (conj[1] - rk[1]).powi(2)
                + (conj[2] - rk[2]).powi(2)
                + (conj[3] - rk[3]).powi(2))
            .sqrt();
            if d < best_inv_dist {
                best_inv_dist = d;
                best_inv = k;
            }
        }
        inverse[i] = best_inv as u8;
        max_closure_dist = max_closure_dist.max(best_inv_dist);
        for j in 0..120 {
            let prod = hamilton_product_f64(roots[i], roots[j]);
            let mut best_k = 0usize;
            let mut best_dist = f64::INFINITY;
            for (k, rk) in roots.iter().enumerate() {
                let d = ((prod[0] - rk[0]).powi(2)
                    + (prod[1] - rk[1]).powi(2)
                    + (prod[2] - rk[2]).powi(2)
                    + (prod[3] - rk[3]).powi(2))
                .sqrt();
                if d < best_dist {
                    best_dist = d;
                    best_k = k;
                }
            }
            compose[i][j] = best_k as u8;
            max_closure_dist = max_closure_dist.max(best_dist);
        }
    }
    if max_closure_dist > 1e-12 {
        return Err(invalid(format!(
            "icosian Cayley closure distance {max_closure_dist} exceeds 1e-12"
        )));
    }
    for i in 0..120 {
        let inv = usize::from(inverse[i]);
        if compose[1][i] != i as u8
            || compose[i][1] != i as u8
            || compose[i][inv] != 1
            || compose[inv][i] != 1
        {
            return Err(invalid(format!(
                "icosian group identity/inverse axiom failed at root {i}"
            )));
        }
    }
    // Verify signed_h4_code_i32 maps each scaled root back to its exact index 0..120.
    let mut exact_self_matches = 0usize;
    for (idx, r) in roots.iter().enumerate() {
        let lane = [
            (r[0] * 1_000_000.0).round() as i32,
            (r[1] * 1_000_000.0).round() as i32,
            (r[2] * 1_000_000.0).round() as i32,
            (r[3] * 1_000_000.0).round() as i32,
        ];
        if usize::from(signed_h4_code_i32(lane)) == idx {
            exact_self_matches += 1;
        }
    }
    if exact_self_matches != 120 {
        return Err(invalid(format!(
            "signed_h4_code_i32 self-recovery matched {exact_self_matches}/120 roots"
        )));
    }
    Ok(IcosianClosureCheck {
        root_count: 120,
        identity_index: 1,
        h4_coefficients_sha256: h4_coefficients_sha256(),
        max_unit_norm_error: max_unit_norm_err,
        max_cayley_closure_distance: max_closure_dist,
        exact_root_self_recovery: exact_self_matches,
        passed: true,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IcosianClosureCheck {
    pub root_count: usize,
    pub identity_index: u8,
    pub h4_coefficients_sha256: String,
    pub max_unit_norm_error: f64,
    pub max_cayley_closure_distance: f64,
    pub exact_root_self_recovery: usize,
    pub passed: bool,
}

/// The four registered gate arms in `docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HamiltonArm {
    HamiltonActive,
    HamiltonIcosianHard,
    IdentityGateControl,
    RandomGateNull,
}

impl HamiltonArm {
    pub const ALL: [HamiltonArm; 4] = [
        HamiltonArm::HamiltonActive,
        HamiltonArm::HamiltonIcosianHard,
        HamiltonArm::IdentityGateControl,
        HamiltonArm::RandomGateNull,
    ];

    pub fn name(self) -> &'static str {
        match self {
            HamiltonArm::HamiltonActive => "Hamilton_Active",
            HamiltonArm::HamiltonIcosianHard => "Hamilton_IcosianHard",
            HamiltonArm::IdentityGateControl => "IdentityGate_Control",
            HamiltonArm::RandomGateNull => "RandomGate_Null",
        }
    }
}

#[derive(Clone, Debug)]
struct GateHookState {
    active_arm: Option<HamiltonArm>,
    seed: u64,
    step: usize,
    fwd_call_idx: usize,
    bwd_call_idx: usize,
    icosian_export_mode: bool,
    capture_telemetry: bool,
    cuda_error_checks_passed: u64,
    // Intrinsic per-4-block suppression telemetry
    total_blocks_observed: u64,
    min_g_norm: f64,
    max_g_norm: f64,
    sum_g_norm: f64,
    g_norm_samples: Vec<f32>,
    min_rho: f64,
    sum_rho: f64,
    blocks_rho_le_0_5: u64,
    blocks_g_norm_le_1e3: u64,
    blocks_g_norm_le_guard: u64,
    max_inv_r: f64,
    max_d_gate_block_norm: f64,
    step0_d_gate_sq_sum: f64,
    icosian_histogram: [u64; 120],
    sum_icosian_cos: f64,
    icosian_gpu_vs_i32_checked: u64,
    icosian_gpu_vs_i32_matched: u64,
    // Downstream projection gain ratio telemetry: ||P_down(q (x) u)||_2 / ||P_down(u)||_2
    downstream_rows_observed: u64,
    sum_downstream_gain_ratio: f64,
    min_downstream_gain_ratio: f64,
    max_downstream_gain_ratio: f64,
    downstream_rows_suppressed_lt_0_8: u64,
    downstream_gain_samples: Vec<f32>,
}

impl Default for GateHookState {
    fn default() -> Self {
        Self {
            active_arm: None,
            seed: 0,
            step: 0,
            fwd_call_idx: 0,
            bwd_call_idx: 0,
            icosian_export_mode: false,
            capture_telemetry: false,
            cuda_error_checks_passed: 0,
            total_blocks_observed: 0,
            min_g_norm: f64::INFINITY,
            max_g_norm: 0.0,
            sum_g_norm: 0.0,
            g_norm_samples: Vec::new(),
            min_rho: f64::INFINITY,
            sum_rho: 0.0,
            blocks_rho_le_0_5: 0,
            blocks_g_norm_le_1e3: 0,
            blocks_g_norm_le_guard: 0,
            max_inv_r: 0.0,
            max_d_gate_block_norm: 0.0,
            step0_d_gate_sq_sum: 0.0,
            icosian_histogram: [0; 120],
            sum_icosian_cos: 0.0,
            icosian_gpu_vs_i32_checked: 0,
            icosian_gpu_vs_i32_matched: 0,
            downstream_rows_observed: 0,
            sum_downstream_gain_ratio: 0.0,
            min_downstream_gain_ratio: f64::INFINITY,
            max_downstream_gain_ratio: 0.0,
            downstream_rows_suppressed_lt_0_8: 0,
            downstream_gain_samples: Vec::new(),
        }
    }
}

impl GateHookState {
    fn reset_telemetry(&mut self) {
        self.total_blocks_observed = 0;
        self.min_g_norm = f64::INFINITY;
        self.max_g_norm = 0.0;
        self.sum_g_norm = 0.0;
        self.g_norm_samples.clear();
        self.min_rho = f64::INFINITY;
        self.sum_rho = 0.0;
        self.blocks_rho_le_0_5 = 0;
        self.blocks_g_norm_le_1e3 = 0;
        self.blocks_g_norm_le_guard = 0;
        self.max_inv_r = 0.0;
        self.icosian_histogram = [0; 120];
        self.sum_icosian_cos = 0.0;
        self.icosian_gpu_vs_i32_checked = 0;
        self.icosian_gpu_vs_i32_matched = 0;
        self.downstream_rows_observed = 0;
        self.sum_downstream_gain_ratio = 0.0;
        self.min_downstream_gain_ratio = f64::INFINITY;
        self.max_downstream_gain_ratio = 0.0;
        self.downstream_rows_suppressed_lt_0_8 = 0;
        self.downstream_gain_samples.clear();
    }
}

thread_local! {
    static GATE_HOOK: RefCell<GateHookState> = RefCell::new(GateHookState::default());
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn record_cuda_error_check_ok() {
    GATE_HOOK.with(|cell| {
        cell.borrow_mut().cuda_error_checks_passed += 1;
    });
}

pub(crate) fn is_capturing_gate_telemetry() -> bool {
    GATE_HOOK.with(|cell| cell.borrow().capture_telemetry)
}

/// Records row-wise L2 norm ratios `||down||_2 / ||unrotated||_2` after `mlp.down` or `rec.out`.
pub(crate) fn record_downstream_projection_norms(
    rotated: &Tensor,
    unrotated: &Tensor,
) -> Result<()> {
    let capture = GATE_HOOK.with(|cell| cell.borrow().capture_telemetry);
    if !capture {
        return Ok(());
    }
    let width = *rotated
        .dims()
        .last()
        .ok_or_else(|| invalid("missing width dim"))?;
    let rows = rotated.elem_count() / width;
    let rot_norms = rotated
        .reshape((rows, width))?
        .sqr()?
        .sum(1)?
        .sqrt()?
        .to_vec1::<f32>()?;
    let unrot_norms = unrotated
        .reshape((rows, width))?
        .sqr()?
        .sum(1)?
        .sqrt()?
        .to_vec1::<f32>()?;
    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        for (&rn, &un) in rot_norms.iter().zip(&unrot_norms) {
            if un > 1e-12 && rn.is_finite() && un.is_finite() {
                let ratio = f64::from(rn) / f64::from(un);
                st.downstream_rows_observed += 1;
                st.sum_downstream_gain_ratio += ratio;
                st.min_downstream_gain_ratio = st.min_downstream_gain_ratio.min(ratio);
                st.max_downstream_gain_ratio = st.max_downstream_gain_ratio.max(ratio);
                if ratio < 0.80 {
                    st.downstream_rows_suppressed_lt_0_8 += 1;
                }
                if st.downstream_gain_samples.len() < 16_384 {
                    st.downstream_gain_samples.push(ratio as f32);
                }
            }
        }
    });
    Ok(())
}

#[cfg(feature = "cuda")]
#[inline]
fn splitmix64_index(seed: u64, call_idx: usize, block_idx: usize) -> usize {
    let mut z = seed
        ^ (call_idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (block_idx as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) as usize) % 120
}

#[cfg(feature = "cuda")]
fn cached_device_icosians(device: &CudaDevice) -> candle_core::Result<(Tensor, Tensor)> {
    type Cache = std::collections::HashMap<String, (Tensor, Tensor)>;
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Cache>> = std::sync::OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| candle_core::Error::Msg("icosian cache lock poisoned".into()))?;
    let key = format!("{:?}", device.id());
    if let Some(pair) = cache.get(&key) {
        return Ok(pair.clone());
    }
    let roots_flat: Vec<f32> = h4_unit_roots_f32().into_iter().flatten().collect();
    let dev = Device::Cuda(device.clone());
    let roots_120_4 = Tensor::from_vec(roots_flat, (120, 4), &dev)?;
    let roots_1_120_4 = roots_120_4.unsqueeze(0)?;
    cache.insert(key, (roots_120_4.clone(), roots_1_120_4.clone()));
    Ok((roots_120_4, roots_1_120_4))
}

/// Snaps a contiguous F32 tensor of shape `(..., 4 * K)` to its nearest unit icosians in `I_120` on CUDA.
#[cfg(feature = "cuda")]
pub fn icosian_snap_cuda_tensor(device: &CudaDevice, gate: &Tensor) -> candle_core::Result<Tensor> {
    let total = gate.elem_count();
    let blocks = total / 4;
    let gate_2d = gate.reshape((blocks, 4))?;
    let (roots_120_4, roots_1_120_4) = cached_device_icosians(device)?;
    let chunk_size = 8192usize;
    let mut chunks = Vec::with_capacity(blocks.div_ceil(chunk_size));
    let dev = Device::Cuda(device.clone());
    for start in (0..blocks).step_by(chunk_size) {
        let len = (blocks - start).min(chunk_size);
        let sub = gate_2d.narrow(0, start, len)?;
        let scores = sub.unsqueeze(1)?.broadcast_mul(&roots_1_120_4)?.sum(2)?;
        let raw_idx = scores.argmax(1)?;
        let is_zero = sub.abs()?.sum(1)?.eq(0.0f32)?;
        let ones = Tensor::ones(len, DType::U32, &dev)?;
        let idx = is_zero.where_cond(&ones, &raw_idx)?;
        let snapped = roots_120_4.index_select(&idx, 0)?;
        chunks.push(snapped);
    }
    let full = if chunks.len() == 1 {
        chunks.pop().unwrap()
    } else {
        Tensor::cat(&chunks, 0)?
    };
    full.reshape(gate.shape())
}

#[cfg(feature = "cuda")]
fn build_random_icosian_tensor(
    device: &CudaDevice,
    shape: &Shape,
    seed: u64,
    call_idx: usize,
) -> candle_core::Result<Tensor> {
    let total = shape.elem_count();
    let blocks = total / 4;
    let roots = h4_unit_roots_f32();
    let mut values = Vec::with_capacity(total);
    for b in 0..blocks {
        let code = splitmix64_index(seed, call_idx, b);
        values.extend_from_slice(&roots[code]);
    }
    Tensor::from_vec(values, shape.clone(), &Device::Cuda(device.clone()))
}

#[cfg(feature = "cuda")]
fn build_identity_gate_tensor(device: &CudaDevice, shape: &Shape) -> candle_core::Result<Tensor> {
    let total = shape.elem_count();
    let blocks = total / 4;
    let dev = Device::Cuda(device.clone());
    let unit = Tensor::from_vec(vec![1.0f32, 0.0, 0.0, 0.0], (1, 4), &dev)?;
    unit.broadcast_as((blocks, 4))?
        .contiguous()?
        .reshape(shape.clone())
}

#[cfg(feature = "cuda")]
fn observe_gate_blocks_telemetry(gate_host: &[f32], up_host: &[f32], out_host: &[f32]) {
    let roots = h4_unit_roots_f32();
    let guard_half = 1e-6_f64 / 3.0_f64.sqrt();
    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        let blocks = gate_host.len() / 4;
        let sample_stride = (blocks / 256).max(1);
        for b in 0..blocks {
            let i = b * 4;
            let g = [
                gate_host[i],
                gate_host[i + 1],
                gate_host[i + 2],
                gate_host[i + 3],
            ];
            let u = [up_host[i], up_host[i + 1], up_host[i + 2], up_host[i + 3]];
            let o = [
                out_host[i],
                out_host[i + 1],
                out_host[i + 2],
                out_host[i + 3],
            ];
            let g_sq = f64::from(g[0]) * f64::from(g[0])
                + f64::from(g[1]) * f64::from(g[1])
                + f64::from(g[2]) * f64::from(g[2])
                + f64::from(g[3]) * f64::from(g[3]);
            let g_norm = g_sq.sqrt();
            let r = (g_sq + 1e-12_f64).sqrt();
            let inv_r = 1.0 / r;
            let u_norm = (f64::from(u[0]) * f64::from(u[0])
                + f64::from(u[1]) * f64::from(u[1])
                + f64::from(u[2]) * f64::from(u[2])
                + f64::from(u[3]) * f64::from(u[3]))
            .sqrt();
            let o_norm = (f64::from(o[0]) * f64::from(o[0])
                + f64::from(o[1]) * f64::from(o[1])
                + f64::from(o[2]) * f64::from(o[2])
                + f64::from(o[3]) * f64::from(o[3]))
            .sqrt();
            let rho = if u_norm > 1e-20 {
                o_norm / u_norm
            } else {
                g_norm / r
            };
            st.total_blocks_observed += 1;
            st.min_g_norm = st.min_g_norm.min(g_norm);
            st.max_g_norm = st.max_g_norm.max(g_norm);
            st.sum_g_norm += g_norm;
            st.min_rho = st.min_rho.min(rho);
            st.sum_rho += rho;
            st.max_inv_r = st.max_inv_r.max(inv_r);
            if rho <= 0.5 {
                st.blocks_rho_le_0_5 += 1;
            }
            if g_norm <= 1e-3 {
                st.blocks_g_norm_le_1e3 += 1;
            }
            if g_norm <= guard_half {
                st.blocks_g_norm_le_guard += 1;
            }
            if b % sample_stride == 0 {
                if st.g_norm_samples.len() < 16_384 {
                    st.g_norm_samples.push(g_norm as f32);
                }
                let code_i32 = snap_block_q12_h4_code(g);
                let code_f32 = snap_block_f32_dot_code(g, &roots);
                st.icosian_histogram[usize::from(code_i32)] += 1;
                st.icosian_gpu_vs_i32_checked += 1;
                if code_i32 == code_f32 {
                    st.icosian_gpu_vs_i32_matched += 1;
                }
                let c = roots[usize::from(code_i32)];
                let dot = (f64::from(g[0]) * f64::from(c[0])
                    + f64::from(g[1]) * f64::from(c[1])
                    + f64::from(g[2]) * f64::from(c[2])
                    + f64::from(g[3]) * f64::from(c[3]))
                    / r;
                st.sum_icosian_cos += dot;
            }
        }
    });
}

#[cfg(feature = "cuda")]
pub(crate) fn try_hamilton_gate_fwd_cuda(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> candle_core::Result<Option<(CudaStorage, Shape)>> {
    let (active_arm, seed, call_idx, export_mode, capture) = GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        let c = st.fwd_call_idx;
        if st.active_arm.is_some() || st.icosian_export_mode || st.capture_telemetry {
            st.fwd_call_idx += 1;
        }
        (
            st.active_arm,
            st.seed,
            c,
            st.icosian_export_mode,
            st.capture_telemetry,
        )
    });
    if active_arm.is_none() && !export_mode && !capture {
        return Ok(None);
    }
    let device = &s1.device;
    let dev = Device::Cuda(device.clone());
    let shape = l1.shape().clone();
    let gate_tensor = Tensor::from_storage(
        candle_core::Storage::Cuda(s1.try_clone(l1)?),
        shape.clone(),
        candle_core::op::BackpropOp::none(),
        false,
    );
    let _ = &dev;
    let effective_gate = match (active_arm, export_mode) {
        (Some(HamiltonArm::IdentityGateControl), _) => build_identity_gate_tensor(device, &shape)?,
        (Some(HamiltonArm::RandomGateNull), _) => {
            build_random_icosian_tensor(device, &shape, seed, call_idx)?
        }
        (Some(HamiltonArm::HamiltonIcosianHard), _) | (_, true) => {
            // In export mode, quantize gate to Q12 integer steps before 120-icosian snap.
            let g_in = if export_mode {
                (gate_tensor.affine(f64::from(Q12_SCALE), 0.0))?
                    .round()?
                    .clamp(Q12_I16_MIN, Q12_I16_MAX)?
            } else {
                gate_tensor.clone()
            };
            icosian_snap_cuda_tensor(device, &g_in)?
        }
        _ => gate_tensor.clone(),
    };
    let eff_contig = effective_gate.contiguous()?;
    let (eff_s, eff_l) = eff_contig.storage_and_layout();
    let candle_core::Storage::Cuda(eff_cuda) = &*eff_s else {
        candle_core::bail!("expected CUDA storage for effective gate");
    };
    let (out_storage, out_shape) =
        crate::geometric_stack::cuda_ops::raw_hamilton_gate_fwd(eff_cuda, eff_l, s2, l2)?;
    if capture {
        let up_tensor = Tensor::from_storage(
            candle_core::Storage::Cuda(s2.try_clone(l2)?),
            l2.shape().clone(),
            candle_core::op::BackpropOp::none(),
            false,
        );
        let out_layout = Layout::contiguous(out_shape.clone());
        let out_tensor = Tensor::from_storage(
            candle_core::Storage::Cuda(out_storage.try_clone(&out_layout)?),
            out_shape.clone(),
            candle_core::op::BackpropOp::none(),
            false,
        );
        let g_vec = gate_tensor.flatten_all()?.to_vec1::<f32>()?;
        let u_vec = up_tensor.flatten_all()?.to_vec1::<f32>()?;
        let o_vec = out_tensor.flatten_all()?.to_vec1::<f32>()?;
        observe_gate_blocks_telemetry(&g_vec, &u_vec, &o_vec);
    }
    Ok(Some((out_storage, out_shape)))
}

#[cfg(feature = "cuda")]
pub(crate) fn try_hamilton_gate_bwd_cuda(
    device: &CudaDevice,
    gate: &Tensor,
    up: &Tensor,
    grad: &Tensor,
) -> candle_core::Result<Option<(Tensor, Tensor)>> {
    let (active_arm, seed, step, call_idx, capture) = GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        let c = st.bwd_call_idx;
        if st.active_arm.is_some() || st.capture_telemetry {
            st.bwd_call_idx += 1;
        }
        (st.active_arm, st.seed, st.step, c, st.capture_telemetry)
    });
    let Some(arm) = active_arm else {
        return Ok(None);
    };
    let (d_gate, d_up) = match arm {
        HamiltonArm::HamiltonActive => {
            crate::geometric_stack::cuda_ops::raw_hamilton_gate_bwd(device, gate, up, grad)?
        }
        HamiltonArm::HamiltonIcosianHard => {
            let (d_gate_cont, _) =
                crate::geometric_stack::cuda_ops::raw_hamilton_gate_bwd(device, gate, up, grad)?;
            let q_snap = icosian_snap_cuda_tensor(device, gate)?;
            let (_, d_up_snap) =
                crate::geometric_stack::cuda_ops::raw_hamilton_gate_bwd(device, &q_snap, up, grad)?;
            (d_gate_cont, d_up_snap)
        }
        HamiltonArm::IdentityGateControl => {
            let zero_dg = Tensor::zeros(gate.shape(), DType::F32, gate.device())?;
            (zero_dg, grad.clone())
        }
        HamiltonArm::RandomGateNull => {
            let zero_dg = Tensor::zeros(gate.shape(), DType::F32, gate.device())?;
            let q_rand = build_random_icosian_tensor(device, gate.shape(), seed, call_idx)?;
            let (_, d_up_rand) =
                crate::geometric_stack::cuda_ops::raw_hamilton_gate_bwd(device, &q_rand, up, grad)?;
            (zero_dg, d_up_rand)
        }
    };
    if step == 0 || capture {
        let dg_sq = f64::from(d_gate.sqr()?.sum_all()?.to_scalar::<f32>()?);
        let max_dg_elem = f64::from(d_gate.abs()?.max_all()?.to_scalar::<f32>()?);
        GATE_HOOK.with(|cell| {
            let mut st = cell.borrow_mut();
            if step == 0 {
                st.step0_d_gate_sq_sum += dg_sq;
            }
            st.max_d_gate_block_norm = st.max_d_gate_block_norm.max(max_dg_elem * 2.0);
        });
    }
    Ok(Some((d_gate, d_up)))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateSuppressionTelemetry {
    pub total_blocks_observed: u64,
    pub min_g_norm: f64,
    pub p01_g_norm: f64,
    pub median_g_norm: f64,
    pub mean_g_norm: f64,
    pub max_g_norm: f64,
    pub min_amplitude_ratio_rho: f64,
    pub mean_amplitude_ratio_rho: f64,
    pub blocks_rho_le_0_5: u64,
    pub frac_blocks_rho_le_0_5: f64,
    pub blocks_g_norm_le_1e3: u64,
    pub frac_blocks_g_norm_le_1e3: f64,
    pub blocks_g_norm_le_guard: u64,
    pub max_backward_scale_inv_r: f64,
    pub max_d_gate_block_norm: f64,
    pub active_icosian_roots_used: usize,
    pub mean_icosian_alignment_cos: f64,
    pub icosian_gpu_vs_i32_agreement_rate: f64,
    pub downstream_rows_observed: u64,
    pub mean_downstream_gain_ratio: f64,
    pub min_downstream_gain_ratio: f64,
    pub p10_downstream_gain_ratio: f64,
    pub median_downstream_gain_ratio: f64,
    pub p90_downstream_gain_ratio: f64,
    pub max_downstream_gain_ratio: f64,
    pub frac_rows_downstream_suppressed_lt_0_8: f64,
    pub intrinsic_suppression_observed: bool,
    pub classification: String,
}

fn snapshot_suppression_telemetry() -> GateSuppressionTelemetry {
    GATE_HOOK.with(|cell| {
        let st = cell.borrow();
        let mut g_samples = st.g_norm_samples.clone();
        g_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let p01_g = if g_samples.is_empty() {
            0.0
        } else {
            f64::from(g_samples[g_samples.len() / 100])
        };
        let med_g = if g_samples.is_empty() {
            0.0
        } else {
            f64::from(g_samples[g_samples.len() / 2])
        };
        let mut d_samples = st.downstream_gain_samples.clone();
        d_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let p10_d = if d_samples.is_empty() {
            1.0
        } else {
            f64::from(d_samples[d_samples.len() / 10])
        };
        let med_d = if d_samples.is_empty() {
            1.0
        } else {
            f64::from(d_samples[d_samples.len() / 2])
        };
        let p90_d = if d_samples.is_empty() {
            1.0
        } else {
            f64::from(d_samples[(d_samples.len() * 9) / 10])
        };
        let n_blocks = st.total_blocks_observed.max(1) as f64;
        let n_rows = st.downstream_rows_observed.max(1) as f64;
        let n_checked = st.icosian_gpu_vs_i32_checked.max(1) as f64;
        let active_roots = st.icosian_histogram.iter().filter(|&&c| c > 0).count();
        let intrinsic = st.blocks_rho_le_0_5 > 0 || st.blocks_g_norm_le_1e3 > 0;
        let classification = if intrinsic {
            "H_GuardCoreHazard".to_string()
        } else {
            "H_NoIntrinsicSuppression_DirectionalSteeringOnly".to_string()
        };
        GateSuppressionTelemetry {
            total_blocks_observed: st.total_blocks_observed,
            min_g_norm: if st.min_g_norm.is_finite() {
                st.min_g_norm
            } else {
                0.0
            },
            p01_g_norm: p01_g,
            median_g_norm: med_g,
            mean_g_norm: st.sum_g_norm / n_blocks,
            max_g_norm: st.max_g_norm,
            min_amplitude_ratio_rho: if st.min_rho.is_finite() {
                st.min_rho
            } else {
                1.0
            },
            mean_amplitude_ratio_rho: st.sum_rho / n_blocks,
            blocks_rho_le_0_5: st.blocks_rho_le_0_5,
            frac_blocks_rho_le_0_5: (st.blocks_rho_le_0_5 as f64) / n_blocks,
            blocks_g_norm_le_1e3: st.blocks_g_norm_le_1e3,
            frac_blocks_g_norm_le_1e3: (st.blocks_g_norm_le_1e3 as f64) / n_blocks,
            blocks_g_norm_le_guard: st.blocks_g_norm_le_guard,
            max_backward_scale_inv_r: st.max_inv_r,
            max_d_gate_block_norm: st.max_d_gate_block_norm,
            active_icosian_roots_used: active_roots,
            mean_icosian_alignment_cos: st.sum_icosian_cos / n_checked,
            icosian_gpu_vs_i32_agreement_rate: (st.icosian_gpu_vs_i32_matched as f64) / n_checked,
            downstream_rows_observed: st.downstream_rows_observed,
            mean_downstream_gain_ratio: st.sum_downstream_gain_ratio / n_rows,
            min_downstream_gain_ratio: if st.min_downstream_gain_ratio.is_finite() {
                st.min_downstream_gain_ratio
            } else {
                1.0
            },
            p10_downstream_gain_ratio: p10_d,
            median_downstream_gain_ratio: med_d,
            p90_downstream_gain_ratio: p90_d,
            max_downstream_gain_ratio: st.max_downstream_gain_ratio,
            frac_rows_downstream_suppressed_lt_0_8: (st.downstream_rows_suppressed_lt_0_8 as f64)
                / n_rows,
            intrinsic_suppression_observed: intrinsic,
            classification,
        }
    })
}

/// One document-bounded causal window of length `context` (`192`), with the window offsets
/// `0..context` that correspond to scored response positions (`response_mask[p] == 1 && tokens[p] != bos_id`).
#[derive(Clone, Debug)]
pub struct CausalWindow {
    pub doc_id: usize,
    pub input_ids: Vec<u32>,
    pub scored_offsets: Vec<usize>,
    pub scored_targets: Vec<u32>,
    pub scored_corpus_positions: Vec<usize>,
}

/// Decodes a `UORT` token file and its companion binary `response_mask.u8`, verifying SHA-256 pins.
#[allow(clippy::too_many_arguments)]
pub fn load_pinned_uort_split(
    tokens_path: &Path,
    mask_path: &Path,
    expected_tokens_sha: &str,
    expected_mask_sha: &str,
    expected_payload_tokens: usize,
    expected_eligible_targets: usize,
    vocab_size: usize,
    bos_id: u32,
) -> Result<(Vec<u32>, Vec<u8>)> {
    let token_bytes = fs::read(tokens_path)?;
    let actual_tok_sha = sha256_bytes(&token_bytes);
    if actual_tok_sha != expected_tokens_sha {
        return Err(invalid(format!(
            "tokens.u16 SHA-256 mismatch at {}: got {actual_tok_sha}, expected {expected_tokens_sha}",
            tokens_path.display()
        )));
    }
    let mask_bytes = fs::read(mask_path)?;
    let actual_mask_sha = sha256_bytes(&mask_bytes);
    if actual_mask_sha != expected_mask_sha {
        return Err(invalid(format!(
            "response_mask.u8 SHA-256 mismatch at {}: got {actual_mask_sha}, expected {expected_mask_sha}",
            mask_path.display()
        )));
    }
    if token_bytes.len() < UORT_HEADER_BYTES || &token_bytes[..4] != b"UORT" {
        return Err(invalid("tokens.u16 missing 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(token_bytes[8..16].try_into().unwrap()) as usize;
    let payload = &token_bytes[UORT_HEADER_BYTES..];
    if count != expected_payload_tokens || payload.len() != count * 2 || mask_bytes.len() != count {
        return Err(invalid(format!(
            "UORT payload length mismatch: count={count}, expected={expected_payload_tokens}"
        )));
    }
    let tokens: Vec<u32> = payload
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first().copied() != Some(bos_id) || tokens.iter().any(|&t| t as usize >= vocab_size) {
        return Err(invalid("UORT tokens out of range or missing initial BOS"));
    }
    let eligible_count = tokens
        .iter()
        .zip(&mask_bytes)
        .enumerate()
        .filter(|&(p, (&t, &m))| p > 0 && t != bos_id && m == 1)
        .count();
    if eligible_count != expected_eligible_targets {
        return Err(invalid(format!(
            "eligible target count {eligible_count} != expected {expected_eligible_targets}"
        )));
    }
    Ok((tokens, mask_bytes))
}

/// Partitions a decoded `UORT` split at `<|bos|>` document boundaries into causal windows of
/// length `context` (`192`), scoring each eligible target position (`p > doc_start && tokens[p] != bos_id && mask[p] == 1`)
/// exactly once (`I-DEV-Denominator`).
pub fn build_document_windows(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos_id: u32,
) -> Result<(Vec<CausalWindow>, usize)> {
    let doc_starts: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(idx, &t)| (t == bos_id).then_some(idx))
        .collect();
    let mut windows = Vec::with_capacity(doc_starts.len());
    for (doc_id, &doc_start) in doc_starts.iter().enumerate() {
        let doc_end = doc_starts.get(doc_id + 1).copied().unwrap_or(tokens.len());
        let doc_len = doc_end - doc_start;
        if doc_len <= 1 {
            continue;
        }
        // Partition target positions p in (doc_start + 1)..doc_end into chunks of at most `context`.
        let mut target_cursor = doc_start + 1;
        while target_cursor < doc_end {
            let target_end = (target_cursor + context).min(doc_end);
            // Targets p in target_cursor..target_end are predicted from inputs at (p - 1) in (target_cursor - 1)..(target_end - 1).
            // Include up to `context` causal prefix tokens ending at `target_end - 2`, never crossing `doc_start`.
            let win_start = (target_end - 1).saturating_sub(context).max(doc_start);
            let active_len = (target_end - 1) - win_start;
            let mut input_ids = vec![bos_id; context];
            input_ids[..active_len].copy_from_slice(&tokens[win_start..win_start + active_len]);
            let mut scored_offsets = Vec::new();
            let mut scored_targets = Vec::new();
            let mut scored_corpus_positions = Vec::new();
            for p in target_cursor..target_end {
                let offset = p - 1 - win_start;
                if tokens[p] != bos_id && mask[p] == 1 {
                    scored_offsets.push(offset);
                    scored_targets.push(tokens[p]);
                    scored_corpus_positions.push(p);
                }
            }
            if !scored_offsets.is_empty() {
                windows.push(CausalWindow {
                    doc_id,
                    input_ids,
                    scored_offsets,
                    scored_targets,
                    scored_corpus_positions,
                });
            }
            target_cursor = target_end;
        }
    }
    Ok((windows, doc_starts.len()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnigramReference {
    pub train_eligible_targets: usize,
    pub dev_scored_rows: usize,
    pub top1_token_id: u32,
    pub top8_token_ids: Vec<u32>,
    pub dev_top1_hits: usize,
    pub dev_top1_rate: f64,
    pub dev_top8_hits: usize,
    pub dev_top8_rate: f64,
}

/// Computes the response-masked target frequency count table on `mw-balp/train` (`CHARTER.md` §4.4)
/// and scores `top-1` and `top-8` on the `14,287` eligible DEV targets.
pub fn compute_train_unigram_reference(
    train_tokens: &[u32],
    train_mask: &[u8],
    dev_windows: &[CausalWindow],
    vocab_size: usize,
    bos_id: u32,
) -> Result<(UnigramReference, Vec<u32>)> {
    let mut counts = vec![0u64; vocab_size];
    let mut train_eligible = 0usize;
    for p in 1..train_tokens.len() {
        let tok = train_tokens[p];
        if tok != bos_id && train_mask[p] == 1 {
            counts[tok as usize] += 1;
            train_eligible += 1;
        }
    }
    let mut ranked: Vec<u32> = (0..vocab_size as u32).collect();
    ranked.sort_by(|&a, &b| {
        counts[b as usize]
            .cmp(&counts[a as usize])
            .then_with(|| a.cmp(&b))
    });
    let top1_id = ranked[0];
    let top8_ids = ranked[..8].to_vec();
    let mut dev_targets = Vec::with_capacity(EXPECTED_DEV_ELIGIBLE_TARGETS);
    for w in dev_windows {
        dev_targets.extend_from_slice(&w.scored_targets);
    }
    let top1_hits = dev_targets.iter().filter(|&&t| t == top1_id).count();
    let top8_hits = dev_targets
        .iter()
        .filter(|&&t| top8_ids.contains(&t))
        .count();
    let n = dev_targets.len();
    let preds = vec![top1_id; n];
    Ok((
        UnigramReference {
            train_eligible_targets: train_eligible,
            dev_scored_rows: n,
            top1_token_id: top1_id,
            top8_token_ids: top8_ids,
            dev_top1_hits: top1_hits,
            dev_top1_rate: (top1_hits as f64) / (n.max(1) as f64),
            dev_top8_hits: top8_hits,
            dev_top8_rate: (top8_hits as f64) / (n.max(1) as f64),
        },
        preds,
    ))
}

/// Pure `i32` integer L1 oracle for `NativeVectorWriter` on Q12-quantized `i16` coordinates (`I-Q12-Int-Parity`).
#[allow(clippy::too_many_arguments)]
pub fn pure_i32_q12_writer_oracle(
    query_q12: &[i32],
    table_q12: &[i32],
    roots_q12: &[[i32; 8]; 16],
    rows: usize,
    width: usize,
    vocab: usize,
    kept_cells: usize,
    targets: &[u32],
) -> (Vec<u32>, usize, usize) {
    let slots = vocab.div_ceil(CELLS);
    let mut top1_preds = Vec::with_capacity(rows);
    let mut top1_hits = 0usize;
    let mut admitted_hits = 0usize;
    for r in 0..rows {
        let q_row = &query_q12[r * width..(r + 1) * width];
        let q_first = &q_row[0..8];
        let q_last = &q_row[width - 8..width];
        let mut port_a = [0i64; 16];
        let mut port_b = [0i64; 16];
        for root_idx in 0..16 {
            let mut l1_a = 0i64;
            let mut l1_b = 0i64;
            for j in 0..8 {
                l1_a += i64::from(q_first[j] - roots_q12[root_idx][j]).abs();
                l1_b += i64::from(q_last[j] - roots_q12[root_idx][j]).abs();
            }
            port_a[root_idx] = -l1_a;
            port_b[root_idx] = -l1_b;
        }
        let mut cell_scores = [(0i64, 0usize); CELLS];
        for (a, &pa) in port_a.iter().enumerate() {
            for (b, &pb) in port_b.iter().enumerate() {
                let cell = a * 16 + b;
                cell_scores[cell] = (pa + pb, cell);
            }
        }
        // Highest score first, smallest cell id on tie.
        cell_scores.sort_by(|x, y| y.0.cmp(&x.0).then_with(|| x.1.cmp(&y.1)));
        let kept = &cell_scores[..kept_cells];
        let mut best_tok = u32::MAX;
        let mut best_score = i64::MIN;
        let mut target_admitted = false;
        for &(_, cell) in kept {
            for slot in 0..slots {
                let tok = cell + CELLS * slot;
                if tok >= vocab {
                    continue;
                }
                if tok as u32 == targets[r] {
                    target_admitted = true;
                }
                let t_row = &table_q12[tok * width..(tok + 1) * width];
                let mut l1 = 0i64;
                for j in 0..width {
                    l1 += i64::from(q_row[j] - t_row[j]).abs();
                }
                let score = -l1;
                if score > best_score || (score == best_score && (tok as u32) < best_tok) {
                    best_score = score;
                    best_tok = tok as u32;
                }
            }
        }
        if best_tok == targets[r] {
            top1_hits += 1;
        }
        if target_admitted {
            admitted_hits += 1;
        }
        top1_preds.push(best_tok);
    }
    (top1_preds, top1_hits, admitted_hits)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DevEvalMetrics {
    pub scored_rows: usize,
    pub top1_hits: usize,
    pub top1_rate: f64,
    pub top8_hits: usize,
    pub top8_rate: f64,
    pub admitted_target_hits: usize,
    pub admitted_target_rate: f64,
    pub token_unviolated_hits: usize,
    pub token_unviolated_rate: f64,
    pub token_margin: f64,
    pub address_margin: f64,
    pub total_margin: f64,
    pub eval_wall_seconds: f64,
    pub eval_tokens_per_second: f64,
    pub q12_i32_oracle_checked_rows: usize,
    pub q12_i32_oracle_mismatches: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportGapMetrics {
    pub delta_top1_rows: usize,
    pub delta_top1_rate_pp: f64,
    pub delta_top8_rows: usize,
    pub delta_top8_rate_pp: f64,
    pub delta_admitted_rows: usize,
    pub delta_token_margin: f64,
    pub row_top1_agreement_hits: usize,
    pub row_top1_agreement_rate: f64,
    pub within_k16_noise_band: bool,
}

/// Evaluates `model` across all `dev_windows` (`14,287` scored positions) in either continuous
/// `f32` (`export_mode == false`) or 120-icosian + `i16` Q12 fixed-point export (`export_mode == true`).
pub fn evaluate_dev_panel(
    model: &StackModel,
    dev_windows: &[CausalWindow],
    arm: HamiltonArm,
    seed: u64,
    step: usize,
    export_mode: bool,
    capture_telemetry: bool,
) -> Result<(DevEvalMetrics, Vec<u32>)> {
    let t0 = Instant::now();
    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        st.active_arm = Some(arm);
        st.seed = seed;
        st.step = step;
        st.fwd_call_idx = 0;
        st.icosian_export_mode = export_mode;
        st.capture_telemetry = capture_telemetry;
        if capture_telemetry {
            st.reset_telemetry();
        }
    });

    let spec = WriterSpec {
        vocabulary: model.config.vocab_size,
        width: model.config.width,
        kept_cells: SHEET_KEPT_CELLS,
    };
    let table_tensor = model
        .variables()
        .get("embedding.weight")
        .ok_or_else(|| invalid("missing embedding.weight"))?
        .as_tensor();
    let eval_table = if export_mode {
        table_tensor
            .affine(f64::from(Q12_SCALE), 0.0)?
            .round()?
            .clamp(Q12_I16_MIN, Q12_I16_MAX)?
            .affine(1.0 / f64::from(Q12_SCALE), 0.0)?
    } else {
        table_tensor.clone()
    };
    let writer = NativeVectorWriter::new(&eval_table, spec)?;

    let mut total_scored = 0usize;
    let mut top1_hits = 0usize;
    let mut top8_hits = 0usize;
    let mut admitted_hits = 0usize;
    let mut unviolated_hits = 0usize;
    let mut token_margin_sum = 0.0f64;
    let mut address_margin_sum = 0.0f64;
    let mut total_margin_sum = 0.0f64;
    let mut all_top1_preds = Vec::with_capacity(EXPECTED_DEV_ELIGIBLE_TARGETS);
    let mut q12_checked_rows = 0usize;
    let mut q12_mismatches = 0usize;

    let plan8 = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
    let roots_q12: [[i32; 8]; 16] = std::array::from_fn(|r| {
        std::array::from_fn(|j| i32::from(plan8.geometry.roots[r][j]) * (Q12_SCALE as i32))
    });
    let table_q12_host: Option<Vec<i32>> = if export_mode {
        Some(
            table_tensor
                .affine(f64::from(Q12_SCALE), 0.0)?
                .round()?
                .clamp(Q12_I16_MIN, Q12_I16_MAX)?
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(|v| v as i32)
                .collect(),
        )
    } else {
        None
    };

    for chunk in dev_windows.chunks(SHEET_EVAL_BATCH) {
        let batch_len = chunk.len();
        let mut flat_ids = Vec::with_capacity(batch_len * SHEET_CONTEXT);
        let mut scored_indices = Vec::new();
        let mut scored_targets = Vec::new();
        for (b_idx, win) in chunk.iter().enumerate() {
            flat_ids.extend_from_slice(&win.input_ids);
            for (&off, &tgt) in win.scored_offsets.iter().zip(&win.scored_targets) {
                scored_indices.push((b_idx * SHEET_CONTEXT + off) as u32);
                scored_targets.push(tgt);
            }
        }
        if scored_indices.is_empty() {
            continue;
        }
        let hidden = model.hidden(&flat_ids, batch_len, SHEET_CONTEXT)?;
        let idx_tensor = Tensor::from_vec(scored_indices, scored_targets.len(), model.device())?;
        let scored_hidden = hidden.index_select(&idx_tensor, 0)?;
        let eval_hidden = if export_mode {
            scored_hidden
                .affine(f64::from(Q12_SCALE), 0.0)?
                .round()?
                .clamp(Q12_I16_MIN, Q12_I16_MAX)?
                .affine(1.0 / f64::from(Q12_SCALE), 0.0)?
        } else {
            scored_hidden
        };
        let candidates = writer.candidates(&eval_hidden)?;
        let metrics = writer.metrics(&candidates, &scored_targets, SHEET_RIVALS, 8)?;
        let batch_preds = writer.write(&eval_hidden)?.to_vec1::<u32>()?;

        if let Some(ref tbl_i32) = table_q12_host {
            if q12_checked_rows == 0 {
                let q_i32: Vec<i32> = eval_hidden
                    .affine(f64::from(Q12_SCALE), 0.0)?
                    .round()?
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(|v| v as i32)
                    .collect();
                let (oracle_preds, oracle_top1, oracle_adm) = pure_i32_q12_writer_oracle(
                    &q_i32,
                    tbl_i32,
                    &roots_q12,
                    scored_targets.len(),
                    model.config.width,
                    model.config.vocab_size,
                    SHEET_KEPT_CELLS,
                    &scored_targets,
                );
                q12_checked_rows = scored_targets.len();
                let gpu_top1 = metrics.top1_count.to_scalar::<u32>()? as usize;
                let gpu_adm = metrics.admitted_target_count.to_scalar::<u32>()? as usize;
                let pred_diffs = batch_preds
                    .iter()
                    .zip(&oracle_preds)
                    .filter(|(a, b)| a != b)
                    .count();
                q12_mismatches = pred_diffs
                    + usize::from(gpu_top1 != oracle_top1)
                    + usize::from(gpu_adm != oracle_adm);
            }
        }

        let n = scored_targets.len();
        total_scored += n;
        top1_hits += metrics.top1_count.to_scalar::<u32>()? as usize;
        top8_hits += metrics.recall_at_k_count.to_scalar::<u32>()? as usize;
        admitted_hits += metrics.admitted_target_count.to_scalar::<u32>()? as usize;
        unviolated_hits += metrics.token_unviolated_count.to_scalar::<u32>()? as usize;
        token_margin_sum += f64::from(metrics.token_margin.to_scalar::<f32>()?) * (n as f64);
        address_margin_sum += f64::from(metrics.address_margin.to_scalar::<f32>()?) * (n as f64);
        total_margin_sum += f64::from(metrics.total_margin.to_scalar::<f32>()?) * (n as f64);
        all_top1_preds.extend_from_slice(&batch_preds);
    }

    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        st.icosian_export_mode = false;
        st.capture_telemetry = false;
    });

    let elapsed = t0.elapsed().as_secs_f64();
    let total_tokens_processed = dev_windows.len() * SHEET_CONTEXT;
    let denom = total_scored.max(1) as f64;
    Ok((
        DevEvalMetrics {
            scored_rows: total_scored,
            top1_hits,
            top1_rate: (top1_hits as f64) / denom,
            top8_hits,
            top8_rate: (top8_hits as f64) / denom,
            admitted_target_hits: admitted_hits,
            admitted_target_rate: (admitted_hits as f64) / denom,
            token_unviolated_hits: unviolated_hits,
            token_unviolated_rate: (unviolated_hits as f64) / denom,
            token_margin: token_margin_sum / denom,
            address_margin: address_margin_sum / denom,
            total_margin: total_margin_sum / denom,
            eval_wall_seconds: elapsed,
            eval_tokens_per_second: (total_tokens_processed as f64) / elapsed.max(1e-9),
            q12_i32_oracle_checked_rows: q12_checked_rows,
            q12_i32_oracle_mismatches: q12_mismatches,
        },
        all_top1_preds,
    ))
}

pub fn compute_export_gap(
    float_eval: &DevEvalMetrics,
    float_preds: &[u32],
    int_eval: &DevEvalMetrics,
    int_preds: &[u32],
) -> ExportGapMetrics {
    let n = float_eval.scored_rows.max(1) as f64;
    let delta_top1 = float_eval.top1_hits.abs_diff(int_eval.top1_hits);
    let delta_top8 = float_eval.top8_hits.abs_diff(int_eval.top8_hits);
    let delta_adm = float_eval
        .admitted_target_hits
        .abs_diff(int_eval.admitted_target_hits);
    let delta_margin = (float_eval.token_margin - int_eval.token_margin).abs();
    let agree_hits = float_preds
        .iter()
        .zip(int_preds)
        .filter(|(a, b)| a == b)
        .count();
    ExportGapMetrics {
        delta_top1_rows: delta_top1,
        delta_top1_rate_pp: 100.0 * (delta_top1 as f64) / n,
        delta_top8_rows: delta_top8,
        delta_top8_rate_pp: 100.0 * (delta_top8 as f64) / n,
        delta_admitted_rows: delta_adm,
        delta_token_margin: delta_margin,
        row_top1_agreement_hits: agree_hits,
        row_top1_agreement_rate: (agree_hits as f64) / n,
        within_k16_noise_band: delta_top1 <= K16_NOISE_TOP1_ROWS
            && delta_top8 <= K16_NOISE_TOP8_ROWS
            && delta_adm <= K16_NOISE_ADM_ROWS,
    }
}

/// One response-masked training batch: `(flat_input_ids, scored_batch_token_indices, scored_targets)`.
pub type TrainBatch = (Vec<u32>, Vec<u32>, Vec<u32>);

/// Builds the 64 response-masked training batches (`batch = 4, context = 192`) from `mw-balp/train`.
pub fn build_training_batches(
    train_tokens: &[u32],
    train_mask: &[u8],
    batch_size: usize,
    context: usize,
    num_batches: usize,
    bos_id: u32,
) -> Result<Vec<TrainBatch>> {
    let (all_windows, _) = build_document_windows(train_tokens, train_mask, context, bos_id)?;
    let usable: Vec<&CausalWindow> = all_windows
        .iter()
        .filter(|w| w.scored_offsets.len() >= 4)
        .take(num_batches * batch_size)
        .collect();
    if usable.len() < num_batches * batch_size {
        return Err(invalid(format!(
            "insufficient training windows with >=4 response targets: got {}, need {}",
            usable.len(),
            num_batches * batch_size
        )));
    }
    let mut batches = Vec::with_capacity(num_batches);
    for chunk in usable.chunks_exact(batch_size) {
        let mut flat_ids = Vec::with_capacity(batch_size * context);
        let mut scored_indices = Vec::new();
        let mut scored_targets = Vec::new();
        for (b_idx, win) in chunk.iter().enumerate() {
            flat_ids.extend_from_slice(&win.input_ids);
            for (&off, &tgt) in win.scored_offsets.iter().zip(&win.scored_targets) {
                scored_indices.push((b_idx * context + off) as u32);
                scored_targets.push(tgt);
            }
        }
        batches.push((flat_ids, scored_indices, scored_targets));
    }
    Ok(batches)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HamiltonArmRunResult {
    pub arm: String,
    pub seed: u64,
    pub steps: usize,
    pub train_loss_step0: f64,
    pub train_loss_step128: f64,
    pub train_loss_step256: f64,
    pub train_loss_delta_256: f64,
    pub train_wall_seconds: f64,
    pub train_tokens_per_second: f64,
    pub step0_d_gate_l2_norm: f64,
    pub cuda_error_checks_passed: u64,
    pub dev_step0_float: DevEvalMetrics,
    pub dev_step0_int_export: DevEvalMetrics,
    pub dev_step128_float: DevEvalMetrics,
    pub dev_step128_int_export: DevEvalMetrics,
    pub dev_step256_float: DevEvalMetrics,
    pub dev_step256_int_export: DevEvalMetrics,
    pub export_gap_step0: ExportGapMetrics,
    pub export_gap_step256: ExportGapMetrics,
    pub suppression_step0: GateSuppressionTelemetry,
    pub suppression_step256: GateSuppressionTelemetry,
}

pub fn build_hamilton_model(seed: u64, device: &Device) -> Result<StackModel> {
    let mut config = StackConfig::transformer_control(seed);
    config.arch = StackArch::Geometric;
    config.pattern = "rrrrrr".into();
    config.width = SHEET_WIDTH;
    config.heads = SHEET_HEADS;
    config.mlp_hidden = SHEET_MLP_HIDDEN;
    config.context = SHEET_CONTEXT;
    config.vocab_size = SHEET_VOCAB;
    config.rotation = true;
    config.rotation_group = RotationGroup::Quaternion;
    config.gate = Gate::Hamilton;
    config.rec_gates = RecGates::Geometric;
    config.objective = Objective::MarginHinge {
        rivals: SHEET_RIVALS,
    };
    let mut model = StackModel::new(config, device)?;
    model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
    model.set_native_vector_head(SHEET_KEPT_CELLS)?;
    Ok(model)
}

/// Runs `f` with the gate hook on `arm`, in the D11-compatible export mode (Q12 gate, icosian
/// snap) or in float, then turns the export mode off again. The SpiralCore/FBS sheet scores its DEV
/// loss through it (`spiralcore_fbs_sheet`, claude-119).
pub fn with_gate_mode<T>(
    arm: HamiltonArm,
    seed: u64,
    step: usize,
    export_mode: bool,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    // The arm, seed and step stay set afterwards, as `evaluate_dev_panel` leaves them; the two
    // flags this call changes only for its own duration are restored.
    let prior = GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        let prior = (st.icosian_export_mode, st.capture_telemetry);
        st.active_arm = Some(arm);
        st.seed = seed;
        st.step = step;
        st.fwd_call_idx = 0;
        st.icosian_export_mode = export_mode;
        st.capture_telemetry = false;
        prior
    });
    let out = f();
    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        (st.icosian_export_mode, st.capture_telemetry) = prior;
    });
    out
}

/// Computes the response-masked margin loss for one training batch on CUDA.
fn response_masked_batch_loss(
    model: &StackModel,
    flat_ids: &[u32],
    scored_indices: &[u32],
    scored_targets: &[u32],
    batch_size: usize,
    context: usize,
) -> Result<Tensor> {
    let hidden = model.hidden(flat_ids, batch_size, context)?;
    let idx_tensor = Tensor::from_vec(
        scored_indices.to_vec(),
        scored_indices.len(),
        model.device(),
    )?;
    let masked_hidden = hidden.index_select(&idx_tensor, 0)?;
    model.native_head_loss(&masked_hidden, scored_targets, SHEET_RIVALS)
}

/// Runs one `(arm, seed)` training and DEV evaluation trajectory on CUDA.
pub fn run_hamilton_arm(
    arm: HamiltonArm,
    seed: u64,
    steps: usize,
    train_batches: &[TrainBatch],
    dev_windows: &[CausalWindow],
    device: &Device,
) -> Result<(HamiltonArmRunResult, Vec<u32>)> {
    let model = build_hamilton_model(seed, device)?;
    let mut opt_main = StackAdamW::new(&model, SHEET_WEIGHT_DECAY, SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, SHEET_CLIP)?;

    // Step-0 DEV evaluation (continuous float + integer export + suppression telemetry)
    let (dev_step0_float, preds_0_float) =
        evaluate_dev_panel(&model, dev_windows, arm, seed, 0, false, true)?;
    let suppression_step0 = snapshot_suppression_telemetry();
    let (dev_step0_int, preds_0_int) =
        evaluate_dev_panel(&model, dev_windows, arm, seed, 0, true, false)?;
    let export_gap_step0 = compute_export_gap(
        &dev_step0_float,
        &preds_0_float,
        &dev_step0_int,
        &preds_0_int,
    );

    let mut train_loss_step0 = 0.0f64;
    let mut train_loss_step128 = 0.0f64;
    let mut dev_step128_float = dev_step0_float.clone();
    let mut dev_step128_int = dev_step0_int.clone();

    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        st.step0_d_gate_sq_sum = 0.0;
        st.cuda_error_checks_passed = 0;
    });

    let t_train = Instant::now();
    for step in 0..steps {
        GATE_HOOK.with(|cell| {
            let mut st = cell.borrow_mut();
            st.active_arm = Some(arm);
            st.seed = seed;
            st.step = step;
            st.fwd_call_idx = 0;
            st.bwd_call_idx = 0;
            st.icosian_export_mode = false;
            st.capture_telemetry = false;
        });
        let (ref flat_ids, ref scored_idx, ref scored_tgts) =
            train_batches[step % train_batches.len()];
        let loss = response_masked_batch_loss(
            &model,
            flat_ids,
            scored_idx,
            scored_tgts,
            SHEET_TRAIN_BATCH,
            SHEET_CONTEXT,
        )?;
        let loss_val = f64::from(loss.to_scalar::<f32>()?);
        if !loss_val.is_finite() {
            return Err(invalid(format!(
                "non-finite training loss at step {step} in {}",
                arm.name()
            )));
        }
        if step == 0 {
            train_loss_step0 = loss_val;
        }
        let grads = loss.backward()?;
        let _ = opt_main.update_filtered(&model, &grads, SHEET_BASE_LR, |name| {
            !name.ends_with("rec.decay")
        })?;
        let _ = opt_decay.update_filtered(
            &model,
            &grads,
            SHEET_BASE_LR * SHEET_DECAY_LR_MULT,
            |name| name.ends_with("rec.decay"),
        )?;

        if step + 1 == 128 {
            train_loss_step128 = loss_val;
            let (d128_f, _) =
                evaluate_dev_panel(&model, dev_windows, arm, seed, 128, false, false)?;
            let (d128_i, _) = evaluate_dev_panel(&model, dev_windows, arm, seed, 128, true, false)?;
            dev_step128_float = d128_f;
            dev_step128_int = d128_i;
        }
    }
    let train_wall = t_train.elapsed().as_secs_f64();
    let total_train_tokens = steps * SHEET_TRAIN_BATCH * SHEET_CONTEXT;

    let (ref flat_ids_0, ref scored_idx_0, ref scored_tgts_0) = train_batches[0];
    GATE_HOOK.with(|cell| {
        let mut st = cell.borrow_mut();
        st.active_arm = Some(arm);
        st.seed = seed;
        st.step = steps;
        st.fwd_call_idx = 0;
        st.icosian_export_mode = false;
        st.capture_telemetry = false;
    });
    let train_loss_step256 = f64::from(
        response_masked_batch_loss(
            &model,
            flat_ids_0,
            scored_idx_0,
            scored_tgts_0,
            SHEET_TRAIN_BATCH,
            SHEET_CONTEXT,
        )?
        .to_scalar::<f32>()?,
    );
    if steps < 128 {
        train_loss_step128 = train_loss_step256;
    }

    let (step0_dg_norm, cuda_checks) = GATE_HOOK.with(|cell| {
        let st = cell.borrow();
        (st.step0_d_gate_sq_sum.sqrt(), st.cuda_error_checks_passed)
    });

    // Step-256 DEV evaluation (continuous float + integer export + suppression telemetry)
    let (dev_step256_float, preds_256_float) =
        evaluate_dev_panel(&model, dev_windows, arm, seed, steps, false, true)?;
    let suppression_step256 = snapshot_suppression_telemetry();
    let (dev_step256_int, preds_256_int) =
        evaluate_dev_panel(&model, dev_windows, arm, seed, steps, true, false)?;
    let export_gap_step256 = compute_export_gap(
        &dev_step256_float,
        &preds_256_float,
        &dev_step256_int,
        &preds_256_int,
    );

    GATE_HOOK.with(|cell| {
        cell.borrow_mut().active_arm = None;
    });

    Ok((
        HamiltonArmRunResult {
            arm: arm.name().to_string(),
            seed,
            steps,
            train_loss_step0,
            train_loss_step128,
            train_loss_step256,
            train_loss_delta_256: train_loss_step0 - train_loss_step256,
            train_wall_seconds: train_wall,
            train_tokens_per_second: (total_train_tokens as f64) / train_wall.max(1e-9),
            step0_d_gate_l2_norm: step0_dg_norm,
            cuda_error_checks_passed: cuda_checks,
            dev_step0_float,
            dev_step0_int_export: dev_step0_int,
            dev_step128_float,
            dev_step128_int_export: dev_step128_int,
            dev_step256_float,
            dev_step256_int_export: dev_step256_int,
            export_gap_step0,
            export_gap_step256,
            suppression_step0,
            suppression_step256,
        },
        preds_256_int,
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairedDiscordance {
    pub arm_a: String,
    pub arm_b: String,
    pub seed: u64,
    pub a_hit_b_miss_c: usize,
    pub a_miss_b_hit_b: usize,
    pub net_advantage_rows: isize,
}

pub fn compute_paired_discordance(
    arm_a: &str,
    preds_a: &[u32],
    arm_b: &str,
    preds_b: &[u32],
    targets: &[u32],
    seed: u64,
) -> PairedDiscordance {
    let mut b_count = 0usize;
    let mut c_count = 0usize;
    for ((&pa, &pb), &t) in preds_a.iter().zip(preds_b).zip(targets) {
        let a_ok = pa == t;
        let b_ok = pb == t;
        if a_ok && !b_ok {
            c_count += 1;
        } else if !a_ok && b_ok {
            b_count += 1;
        }
    }
    PairedDiscordance {
        arm_a: arm_a.to_string(),
        arm_b: arm_b.to_string(),
        seed,
        a_hit_b_miss_c: c_count,
        a_miss_b_hit_b: b_count,
        net_advantage_rows: (c_count as isize) - (b_count as isize),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HamiltonSheetReceipt {
    pub schema: String,
    pub item_id: String,
    pub issue: u32,
    pub prep_sheet: String,
    pub train_tokens_sha256: String,
    pub train_mask_sha256: String,
    pub dev_tokens_sha256: String,
    pub dev_mask_sha256: String,
    pub tokenizer_sha256: String,
    pub cuda_stack_cu_sha256: String,
    pub spiralcore_fbs_source_sha256: String,
    pub dev_documents: usize,
    pub dev_windows: usize,
    pub dev_eligible_targets: usize,
    pub train_batches: usize,
    pub train_batch_size: usize,
    pub context: usize,
    pub steps: usize,
    pub base_lr: f64,
    pub decay_lr_mult: f64,
    pub weight_decay: f64,
    pub icosian_closure_check: IcosianClosureCheck,
    pub unigram_reference: UnigramReference,
    pub runs: Vec<HamiltonArmRunResult>,
    pub paired_discordance: Vec<PairedDiscordance>,
    pub in_run_two_seed_noise_top1_rows: usize,
    pub in_run_two_seed_noise_top8_rows: usize,
    pub effective_noise_band_top1_rows: usize,
    pub effective_noise_band_top8_rows: usize,
    pub i_finite_cuda_pass: bool,
    pub i_dev_denominator_pass: bool,
    pub i_icosian_closure_pass: bool,
    pub i_q12_int_parity_pass: bool,
    pub g_hamilton_fit_pass_unigram: bool,
    pub g_hamilton_beats_random_null: bool,
    pub g_hamilton_icosian_hard_matches_or_beats_active: bool,
    pub g_hamilton_export_gap_pass: bool,
    pub gate_suppressed_intrinsic_amplitude: bool,
    pub gate_suppression_verdict: String,
    pub overall_verdict: String,
}

/// Runs Probe (a): synthetic 4-step CUDA check verifying all 4 arms, `I-Icosian-Closure`, and `I-Q12-Int-Parity`.
pub fn run_synthetic_probe(device: &Device) -> Result<serde_json::Value> {
    let closure = verify_icosian_closure()?;
    let cycle = [0u32, 1, 2, 3, 4, 5, 6, 7];
    let mut win_ids = vec![0u32; SHEET_CONTEXT];
    let mut scored_offsets = Vec::new();
    let mut scored_targets = Vec::new();
    let mut scored_positions = Vec::new();
    for t in 0..32 {
        win_ids[t] = cycle[t % cycle.len()];
        scored_offsets.push(t);
        scored_targets.push(cycle[(t + 1) % cycle.len()]);
        scored_positions.push(t + 1);
    }
    let dev_win = vec![CausalWindow {
        doc_id: 0,
        input_ids: win_ids.clone(),
        scored_offsets: scored_offsets.clone(),
        scored_targets: scored_targets.clone(),
        scored_corpus_positions: scored_positions,
    }];
    let mut flat_train = Vec::with_capacity(SHEET_TRAIN_BATCH * SHEET_CONTEXT);
    let mut train_idx = Vec::new();
    let mut train_tgts = Vec::new();
    for b in 0..SHEET_TRAIN_BATCH {
        flat_train.extend_from_slice(&win_ids);
        for (&off, &tgt) in scored_offsets.iter().zip(&scored_targets) {
            train_idx.push((b * SHEET_CONTEXT + off) as u32);
            train_tgts.push(tgt);
        }
    }
    let train_batches = vec![(flat_train, train_idx, train_tgts)];
    let mut summaries = BTreeMap::new();
    for arm in HamiltonArm::ALL {
        let (res, _) = run_hamilton_arm(arm, 1, 4, &train_batches, &dev_win, device)?;
        summaries.insert(
            arm.name().to_string(),
            serde_json::json!({
                "loss_step0": res.train_loss_step0,
                "loss_step4": res.train_loss_step256,
                "delta": res.train_loss_delta_256,
                "step0_d_gate_l2_norm": res.step0_d_gate_l2_norm,
                "q12_oracle_mismatches": res.dev_step256_int_export.q12_i32_oracle_mismatches,
                "min_rho": res.suppression_step256.min_amplitude_ratio_rho,
                "mean_downstream_gain": res.suppression_step256.mean_downstream_gain_ratio,
            }),
        );
    }
    Ok(serde_json::json!({
        "probe": "synthetic-probe",
        "icosian_closure": closure,
        "arms": summaries,
    }))
}

/// Runs Probe (b): 16-step throughput + full `14,287`-row DEV evaluation timing check.
pub fn run_throughput_probe(data_root: &Path, device: &Device) -> Result<serde_json::Value> {
    let (train_tokens, train_mask) = load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        PINNED_TRAIN_TOKENS_SHA256,
        PINNED_TRAIN_MASK_SHA256,
        EXPECTED_TRAIN_TOKENS,
        EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (dev_tokens, dev_mask) = load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        PINNED_DEV_TOKENS_SHA256,
        PINNED_DEV_MASK_SHA256,
        EXPECTED_DEV_TOKENS,
        EXPECTED_DEV_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (dev_windows, dev_docs) =
        build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, SHEET_BOS_ID)?;
    let train_batches = build_training_batches(
        &train_tokens,
        &train_mask,
        SHEET_TRAIN_BATCH,
        SHEET_CONTEXT,
        SHEET_TRAIN_BATCHES,
        SHEET_BOS_ID,
    )?;
    let t0 = Instant::now();
    let (res_active, _) = run_hamilton_arm(
        HamiltonArm::HamiltonActive,
        1,
        16,
        &train_batches,
        &dev_windows,
        device,
    )?;
    let (res_icosian, _) = run_hamilton_arm(
        HamiltonArm::HamiltonIcosianHard,
        1,
        16,
        &train_batches,
        &dev_windows,
        device,
    )?;
    let elapsed = t0.elapsed().as_secs_f64();
    let projected_per_arm_256 = res_active
        .train_wall_seconds
        .max(res_icosian.train_wall_seconds)
        * 16.0
        + res_active.dev_step256_float.eval_wall_seconds * 6.0;
    Ok(serde_json::json!({
        "probe": "throughput-probe",
        "dev_docs": dev_docs,
        "dev_windows": dev_windows.len(),
        "dev_scored_rows": res_active.dev_step256_int_export.scored_rows,
        "elapsed_seconds": elapsed,
        "active_16step_train_s": res_active.train_wall_seconds,
        "icosian_16step_train_s": res_icosian.train_wall_seconds,
        "dev_eval_float_s": res_active.dev_step256_float.eval_wall_seconds,
        "dev_eval_int_s": res_active.dev_step256_int_export.eval_wall_seconds,
        "projected_256_step_arm_seconds": projected_per_arm_256,
        "projected_full_8_runs_seconds": projected_per_arm_256 * 8.0,
        "under_one_hour_per_arm": projected_per_arm_256 < 3600.0,
    }))
}

/// Runs Probe (c): the full 4-arm × 2-seed × 256-step Hamilton gate offline-float sheet.
pub fn run_full_hamilton_sheet(data_root: &Path, device: &Device) -> Result<HamiltonSheetReceipt> {
    let tok_bytes = fs::read(data_root.join("tokenizer.json"))?;
    let tok_sha = sha256_bytes(&tok_bytes);
    if tok_sha != PINNED_TOKENIZER_SHA256 {
        return Err(invalid(format!(
            "tokenizer.json SHA-256 mismatch: got {tok_sha}, expected {PINNED_TOKENIZER_SHA256}"
        )));
    }
    let closure = verify_icosian_closure()?;
    let (train_tokens, train_mask) = load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        PINNED_TRAIN_TOKENS_SHA256,
        PINNED_TRAIN_MASK_SHA256,
        EXPECTED_TRAIN_TOKENS,
        EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (dev_tokens, dev_mask) = load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        PINNED_DEV_TOKENS_SHA256,
        PINNED_DEV_MASK_SHA256,
        EXPECTED_DEV_TOKENS,
        EXPECTED_DEV_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (dev_windows, dev_docs) =
        build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, SHEET_BOS_ID)?;
    let (unigram_ref, unigram_preds) = compute_train_unigram_reference(
        &train_tokens,
        &train_mask,
        &dev_windows,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let mut dev_targets = Vec::with_capacity(EXPECTED_DEV_ELIGIBLE_TARGETS);
    for w in &dev_windows {
        dev_targets.extend_from_slice(&w.scored_targets);
    }
    let train_batches = build_training_batches(
        &train_tokens,
        &train_mask,
        SHEET_TRAIN_BATCH,
        SHEET_CONTEXT,
        SHEET_TRAIN_BATCHES,
        SHEET_BOS_ID,
    )?;

    let mut runs = Vec::with_capacity(8);
    let mut preds_by_arm_seed: BTreeMap<(String, u64), Vec<u32>> = BTreeMap::new();
    for seed in [1u64, 2u64] {
        for arm in HamiltonArm::ALL {
            let (run_res, preds_int) =
                run_hamilton_arm(arm, seed, SHEET_STEPS, &train_batches, &dev_windows, device)?;
            preds_by_arm_seed.insert((arm.name().to_string(), seed), preds_int);
            runs.push(run_res);
        }
    }

    let mut paired_discordance = Vec::new();
    for seed in [1u64, 2u64] {
        let p_active = &preds_by_arm_seed[&("Hamilton_Active".to_string(), seed)];
        let p_icosian = &preds_by_arm_seed[&("Hamilton_IcosianHard".to_string(), seed)];
        let p_id = &preds_by_arm_seed[&("IdentityGate_Control".to_string(), seed)];
        let p_rand = &preds_by_arm_seed[&("RandomGate_Null".to_string(), seed)];
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_Active",
            p_active,
            "IdentityGate_Control",
            p_id,
            &dev_targets,
            seed,
        ));
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_Active",
            p_active,
            "RandomGate_Null",
            p_rand,
            &dev_targets,
            seed,
        ));
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_Active",
            p_active,
            "Hamilton_IcosianHard",
            p_icosian,
            &dev_targets,
            seed,
        ));
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_Active",
            p_active,
            "TrainUnigram_CountTable",
            &unigram_preds,
            &dev_targets,
            seed,
        ));
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_IcosianHard",
            p_icosian,
            "IdentityGate_Control",
            p_id,
            &dev_targets,
            seed,
        ));
        paired_discordance.push(compute_paired_discordance(
            "Hamilton_IcosianHard",
            p_icosian,
            "RandomGate_Null",
            p_rand,
            &dev_targets,
            seed,
        ));
    }

    let find = |arm: HamiltonArm, seed: u64| -> &HamiltonArmRunResult {
        runs.iter()
            .find(|r| r.arm == arm.name() && r.seed == seed)
            .expect("missing arm-seed run")
    };

    let mut in_run_noise_top1 = 0usize;
    let mut in_run_noise_top8 = 0usize;
    for arm in HamiltonArm::ALL {
        let s1 = find(arm, 1);
        let s2 = find(arm, 2);
        in_run_noise_top1 = in_run_noise_top1.max(
            s1.dev_step256_int_export
                .top1_hits
                .abs_diff(s2.dev_step256_int_export.top1_hits),
        );
        in_run_noise_top8 = in_run_noise_top8.max(
            s1.dev_step256_int_export
                .top8_hits
                .abs_diff(s2.dev_step256_int_export.top8_hits),
        );
    }
    let eff_noise_top1 = in_run_noise_top1.max(K16_NOISE_TOP1_ROWS);
    let eff_noise_top8 = in_run_noise_top8.max(K16_NOISE_TOP8_ROWS);

    let i_finite_cuda_pass = runs.iter().all(|r| {
        r.train_loss_step0.is_finite()
            && r.train_loss_step256.is_finite()
            && r.cuda_error_checks_passed > 0
            && ((r.arm != "Hamilton_Active" && r.arm != "Hamilton_IcosianHard")
                || r.step0_d_gate_l2_norm > 0.0)
    });
    let i_dev_denominator_pass = dev_docs == EXPECTED_DEV_DOCS
        && unigram_ref.dev_scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
        && runs.iter().all(|r| {
            r.dev_step256_float.scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
                && r.dev_step256_int_export.scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
        });
    let i_icosian_closure_pass = closure.passed;
    let i_q12_int_parity_pass = runs.iter().all(|r| {
        r.dev_step0_int_export.q12_i32_oracle_mismatches == 0
            && r.dev_step256_int_export.q12_i32_oracle_mismatches == 0
    });

    let g_hamilton_export_gap_pass = [
        HamiltonArm::HamiltonActive,
        HamiltonArm::HamiltonIcosianHard,
    ]
    .iter()
    .all(|&arm| {
        [1u64, 2u64].iter().all(|&seed| {
            let r = find(arm, seed);
            r.export_gap_step256.delta_top1_rows <= eff_noise_top1
                && r.export_gap_step256.delta_top8_rows <= eff_noise_top8
        })
    });

    let g_hamilton_beats_random_null = [1u64, 2u64].iter().all(|&seed| {
        let act = find(HamiltonArm::HamiltonActive, seed);
        let ico = find(HamiltonArm::HamiltonIcosianHard, seed);
        let rnd = find(HamiltonArm::RandomGateNull, seed);
        (act.dev_step256_int_export.token_margin < rnd.dev_step256_int_export.token_margin
            || act.dev_step256_int_export.top1_hits > rnd.dev_step256_int_export.top1_hits)
            && (ico.dev_step256_int_export.token_margin < rnd.dev_step256_int_export.token_margin
                || ico.dev_step256_int_export.top1_hits > rnd.dev_step256_int_export.top1_hits)
    });

    let g_hamilton_icosian_hard_matches_or_beats_active = [1u64, 2u64].iter().all(|&seed| {
        let act = find(HamiltonArm::HamiltonActive, seed);
        let ico = find(HamiltonArm::HamiltonIcosianHard, seed);
        ico.dev_step256_int_export
            .top1_hits
            .abs_diff(act.dev_step256_int_export.top1_hits)
            <= eff_noise_top1
            || ico.dev_step256_int_export.top1_hits >= act.dev_step256_int_export.top1_hits
    });

    let g_hamilton_fit_pass_unigram = [1u64, 2u64].iter().all(|&seed| {
        let act = find(HamiltonArm::HamiltonActive, seed);
        let ico = find(HamiltonArm::HamiltonIcosianHard, seed);
        let id = find(HamiltonArm::IdentityGateControl, seed);
        let rnd = find(HamiltonArm::RandomGateNull, seed);
        let best_ham = act
            .dev_step256_int_export
            .top1_hits
            .max(ico.dev_step256_int_export.top1_hits);
        let best_ctrl = id
            .dev_step256_int_export
            .top1_hits
            .max(rnd.dev_step256_int_export.top1_hits);
        best_ham > unigram_ref.dev_top1_hits && best_ham > best_ctrl + in_run_noise_top1
    });

    let gate_suppressed_intrinsic_amplitude = [
        HamiltonArm::HamiltonActive,
        HamiltonArm::HamiltonIcosianHard,
    ]
    .iter()
    .any(|&arm| {
        [1u64, 2u64].iter().any(|&seed| {
            let r = find(arm, seed);
            r.suppression_step0.intrinsic_suppression_observed
                || r.suppression_step256.intrinsic_suppression_observed
        })
    });

    let gate_suppression_verdict = if gate_suppressed_intrinsic_amplitude {
        "INTRINSIC_SUPPRESSION_OBSERVED".to_string()
    } else {
        "NO_INTRINSIC_AMPLITUDE_SUPPRESSION_DIRECTIONAL_MODULATION_ONLY".to_string()
    };

    let overall_verdict = if !i_finite_cuda_pass
        || !i_dev_denominator_pass
        || !i_icosian_closure_pass
        || !i_q12_int_parity_pass
    {
        "INSTRUMENT_FAULT".to_string()
    } else if g_hamilton_fit_pass_unigram && g_hamilton_export_gap_pass {
        "PASS_PROMOTE_CANDIDATE".to_string()
    } else if g_hamilton_export_gap_pass {
        "MEASURED_FIT_AND_EXPORT_GAP_WITHIN_NOISE_BAND".to_string()
    } else {
        "EXPORT_GAP_EXCEEDED".to_string()
    };

    Ok(HamiltonSheetReceipt {
        schema: "uor-r4.hamilton-gate-offline-float-sheet/1".into(),
        item_id: "hamilton-gate-offline-float-sheet".into(),
        issue: 2824,
        prep_sheet: "docs/harness/prep/hamilton-gate-offline-float-sheet-2026-10-25.md".into(),
        train_tokens_sha256: PINNED_TRAIN_TOKENS_SHA256.into(),
        train_mask_sha256: PINNED_TRAIN_MASK_SHA256.into(),
        dev_tokens_sha256: PINNED_DEV_TOKENS_SHA256.into(),
        dev_mask_sha256: PINNED_DEV_MASK_SHA256.into(),
        tokenizer_sha256: tok_sha,
        cuda_stack_cu_sha256: CUDA_STACK_CU_SHA256.into(),
        spiralcore_fbs_source_sha256: crate::spiralcore_fbs::FBS_SOURCE_SHA256.into(),
        dev_documents: dev_docs,
        dev_windows: dev_windows.len(),
        dev_eligible_targets: unigram_ref.dev_scored_rows,
        train_batches: SHEET_TRAIN_BATCHES,
        train_batch_size: SHEET_TRAIN_BATCH,
        context: SHEET_CONTEXT,
        steps: SHEET_STEPS,
        base_lr: SHEET_BASE_LR,
        decay_lr_mult: SHEET_DECAY_LR_MULT,
        weight_decay: SHEET_WEIGHT_DECAY,
        icosian_closure_check: closure,
        unigram_reference: unigram_ref,
        runs,
        paired_discordance,
        in_run_two_seed_noise_top1_rows: in_run_noise_top1,
        in_run_two_seed_noise_top8_rows: in_run_noise_top8,
        effective_noise_band_top1_rows: eff_noise_top1,
        effective_noise_band_top8_rows: eff_noise_top8,
        i_finite_cuda_pass,
        i_dev_denominator_pass,
        i_icosian_closure_pass,
        i_q12_int_parity_pass,
        g_hamilton_fit_pass_unigram,
        g_hamilton_beats_random_null,
        g_hamilton_icosian_hard_matches_or_beats_active,
        g_hamilton_export_gap_pass,
        gate_suppressed_intrinsic_amplitude,
        gate_suppression_verdict,
        overall_verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosian_closure_and_signed_h4_classifier_pass() -> Result<()> {
        let check = verify_icosian_closure()?;
        assert!(check.passed);
        assert_eq!(check.root_count, 120);
        assert_eq!(check.identity_index, 1);
        assert_eq!(check.exact_root_self_recovery, 120);
        assert!(check.max_unit_norm_error <= 1e-12);
        assert!(check.max_cayley_closure_distance <= 1e-12);

        // Check zero, axes, signed halves, and exact ties matching uor_r4_integer::h4_classifier.
        assert_eq!(signed_h4_code_i32([0, 0, 0, 0]), 1);
        assert_eq!(signed_h4_code_i32([1, 1, 0, 0]), 62);
        assert_eq!(signed_h4_code_i32([-1, -1, 0, 0]), 56);
        for axis in 0..4 {
            let mut pos = [0i32; 4];
            pos[axis] = 1000;
            assert_eq!(signed_h4_code_i32(pos), (2 * axis + 1) as u8);
            let mut neg = [0i32; 4];
            neg[axis] = -1000;
            assert_eq!(signed_h4_code_i32(neg), (2 * axis) as u8);
        }
        Ok(())
    }

    #[test]
    fn document_window_partition_scores_every_eligible_target_once() -> Result<()> {
        // Two synthetic documents:
        // Doc 0 starts at 0, length 6 (tokens [0, 10, 11, 12, 13, 14]), mask [0, 0, 1, 1, 0, 1] -> 3 eligible targets
        // Doc 1 starts at 6, length 5 (tokens [0, 20, 21, 22, 23]), mask [1, 1, 0, 1, 1] -> 3 eligible targets (mask at BOS is ignored)
        let tokens = vec![0u32, 10, 11, 12, 13, 14, 0, 20, 21, 22, 23];
        let mask = vec![0u8, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1];
        // Context = 3 forces Doc 0 (5 transitions) to split across 2 windows!
        let (windows, doc_count) = build_document_windows(&tokens, &mask, 3, 0)?;
        assert_eq!(doc_count, 2);
        let mut all_positions = Vec::new();
        let mut all_targets = Vec::new();
        for w in &windows {
            assert_eq!(w.input_ids.len(), 3);
            for (&off, (&pos, &tgt)) in w
                .scored_offsets
                .iter()
                .zip(w.scored_corpus_positions.iter().zip(&w.scored_targets))
            {
                assert!(off < 3);
                assert_eq!(tokens[pos], tgt);
                assert_eq!(mask[pos], 1);
                assert_ne!(tgt, 0);
                all_positions.push(pos);
                all_targets.push(tgt);
            }
        }
        assert_eq!(all_positions, vec![2, 3, 5, 7, 9, 10]);
        assert_eq!(all_targets, vec![11, 12, 14, 20, 22, 23]);
        Ok(())
    }

    #[test]
    fn pure_i32_q12_oracle_scores_exact_l1_candidates() {
        let roots_q12 = [[0i32; 8]; 16];
        let width = 8usize;
        let vocab = 256usize;
        let mut table_q12 = vec![0i32; vocab * width];
        for tok in 0..vocab {
            for j in 0..width {
                table_q12[tok * width + j] = (tok as i32) * 16 + (j as i32);
            }
        }
        // Query row 0 equals token 3 exactly; query row 1 equals token 5 + 1.
        let mut query_q12 = vec![0i32; 2 * width];
        for j in 0..width {
            query_q12[j] = 3 * 16 + (j as i32);
            query_q12[width + j] = 5 * 16 + (j as i32) + 1;
        }
        let targets = [3u32, 5u32];
        let (preds, top1, adm) = pure_i32_q12_writer_oracle(
            &query_q12, &table_q12, &roots_q12, 2, width, vocab, 8, &targets,
        );
        assert_eq!(preds, vec![3, 5]);
        assert_eq!(top1, 2);
        assert_eq!(adm, 2);
    }
}
