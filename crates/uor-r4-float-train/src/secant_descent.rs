//! Pre-registered secant surrogate descent check (`r-secant-surrogate-descent`, `#2886`).
//!
//! Executes the 6-arm × 2-seed × 256-step secant surrogate descent check for:
//! - Part A (`RecGates::Geometric`): `rate_select_bwd` on the 16-entry dyadic fade/keep table
//!   (`RecSecant_Active`, `RecZeroCredit_Null`, `RecRandomRefit_Null`), updating the recurrence
//!   rate parameters (`layers.*.rec.decay`) while holding non-rate parameters fixed at initialization.
//! - Part B (`QuaternionWeights::SpiralcoreFbs`): `spiralcore_fbs_bwd` on the 16-root E8 orbit and
//!   base-3 FBS binder tree (`FbsSecant_Active`, `FbsZeroCredit_Null`, `FbsRandomRefit_Null`),
//!   updating the `layers.*.*.spiralcore_fbs` query tensors while holding non-query parameters fixed
//!   at initialization and propagating exact inverse input adjoints `A_o^-1 dy_o` across all layers.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};

use crate::geometric_stack::{
    Gate, Objective, QuaternionWeights, RecGates, RotationGroup, StackArch, StackConfig,
    StackModel, KEEP_TABLE, RATE_ENTRIES, RATE_TABLE,
};
use crate::spiralcore_fbs::{ProjectionPlan, ABSENT};
use crate::{invalid, sha256_bytes, Result};

/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16`.
pub const PINNED_TOKENS_SHA256: &str =
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8`.
pub const PINNED_MASK_SHA256: &str =
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/tokenizer.json`.
pub const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
/// Pinned SHA-256 of `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` source module `cuda_stack.cu`.
pub const CUDA_STACK_CU_SHA256: &str =
    "eb9a3fd870c74919e96bf6c2974717a5ca1914722f98f469bf3e5fb462d478bd";

pub const UORT_HEADER_BYTES: usize = 64;
pub const PROBE_WIDTH: usize = 256;
pub const PROBE_HEADS: usize = 4;
pub const PROBE_MLP_HIDDEN: usize = 384;
pub const PROBE_BATCH: usize = 4;
pub const PROBE_CONTEXT: usize = 192;
pub const PROBE_VOCAB: usize = 4096;
pub const PROBE_KEPT_CELLS: usize = 8;
pub const PROBE_RIVALS: usize = 8;
pub const PROBE_BATCHES: usize = 64;
pub const PROBE_STEPS: usize = 256;

/// The six registered arms in `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecantArm {
    RecSecantActive,
    RecZeroCreditNull,
    RecRandomRefitNull,
    FbsSecantActive,
    FbsZeroCreditNull,
    FbsRandomRefitNull,
}

impl SecantArm {
    pub const ALL: [SecantArm; 6] = [
        SecantArm::RecSecantActive,
        SecantArm::RecZeroCreditNull,
        SecantArm::RecRandomRefitNull,
        SecantArm::FbsSecantActive,
        SecantArm::FbsZeroCreditNull,
        SecantArm::FbsRandomRefitNull,
    ];

    pub const PART_A: [SecantArm; 3] = [
        SecantArm::RecSecantActive,
        SecantArm::RecZeroCreditNull,
        SecantArm::RecRandomRefitNull,
    ];

    pub const PART_B: [SecantArm; 3] = [
        SecantArm::FbsSecantActive,
        SecantArm::FbsZeroCreditNull,
        SecantArm::FbsRandomRefitNull,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SecantArm::RecSecantActive => "RecSecant_Active",
            SecantArm::RecZeroCreditNull => "RecZeroCredit_Null",
            SecantArm::RecRandomRefitNull => "RecRandomRefit_Null",
            SecantArm::FbsSecantActive => "FbsSecant_Active",
            SecantArm::FbsZeroCreditNull => "FbsZeroCredit_Null",
            SecantArm::FbsRandomRefitNull => "FbsRandomRefit_Null",
        }
    }

    pub fn is_part_a(self) -> bool {
        matches!(
            self,
            SecantArm::RecSecantActive
                | SecantArm::RecZeroCreditNull
                | SecantArm::RecRandomRefitNull
        )
    }

    pub fn trainable_variable(self, name: &str) -> bool {
        if self.is_part_a() {
            name.ends_with("rec.decay")
        } else {
            name.ends_with(".spiralcore_fbs")
        }
    }
}

/// Deterministic SplitMix64 generator for Rademacher sign flips (`{-1.0, +1.0}`).
#[derive(Clone, Debug)]
pub struct RademacherRng(u64);

impl RademacherRng {
    pub fn new(seed: u64, step: usize, call_idx: usize, domain: u64) -> Self {
        let mut s = seed
            ^ ((step as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((call_idx as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
            ^ domain;
        let _ = Self::mix(&mut s);
        Self(s)
    }

    #[inline]
    fn mix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns `+1.0f32` or `-1.0f32` with equal probability, preserving Euclidean norm bit-identically.
    #[inline]
    pub fn next_sign(&mut self) -> f32 {
        if (Self::mix(&mut self.0) & 1) == 0 {
            1.0f32
        } else {
            -1.0f32
        }
    }

    pub fn signs(&mut self, len: usize) -> Vec<f32> {
        (0..len).map(|_| self.next_sign()).collect()
    }
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
#[derive(Clone, Debug, Default)]
struct HookState {
    active_arm: Option<SecantArm>,
    seed: u64,
    step: usize,
    rate_bwd_calls: usize,
    fbs_bwd_calls: usize,
    analytic_keep_secant: bool,
    capture_rate_fwd: bool,
    captured_rate_indices: Vec<u8>,
    cuda_error_checks_passed: u64,
    finite_bwd_checks_passed: u64,
    step0_ds_sq_sum: f64,
    step0_dq_sq_sum: f64,
    step0_d_lambda_abs_sum: f64,
    step0_d_lambda_count: u64,
}

thread_local! {
    static HOOK_STATE: RefCell<HookState> = RefCell::new(HookState::default());
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn record_cuda_error_check_ok() {
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.cuda_error_checks_passed += 1;
    });
}

/// Recovers the dyadic table index `i in 0..16` from `lambda_i = 2^{-(i+1)}` emitted by `rate_select_fwd`.
#[inline]
pub fn rate_index_from_lambda_bits(lambda: f32) -> usize {
    for (idx, &entry) in RATE_TABLE.iter().enumerate() {
        if lambda.to_bits() == entry.to_bits() {
            return idx;
        }
    }
    let exp = ((lambda.to_bits() >> 23) & 0xff) as i32;
    (126 - exp).clamp(0, (RATE_ENTRIES - 1) as i32) as usize
}

/// Analytic dyadic secant backward on `[0, 1]` using exact `f64` keep differences
/// `sqrt(1 - 2^{-2(j+1)}) - sqrt(1 - 2^{-2(i+1)})` so `KEEP_TABLE[12..15]` has nonzero
/// keep-channel secant slope if `I-Keep-Plateau` triggers.
pub fn rate_select_bwd_analytic_keep_f64(score: &[f32], grad: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; score.len()];
    let keep_f64: [f64; RATE_ENTRIES] = std::array::from_fn(|idx| {
        let lam = f64::from(RATE_TABLE[idx]);
        (1.0f64 - lam * lam).sqrt()
    });
    for (id, &s) in score.iter().enumerate() {
        let index = ((s * (RATE_ENTRIES as f32)).max(0.0) as usize).min(RATE_ENTRIES - 1);
        let d_lambda = f64::from(grad[2 * id]);
        let d_keep = f64::from(grad[2 * id + 1]);
        let mut credit = 0.0f64;
        let mut sides = 0.0f64;
        if index > 0 {
            let dl = f64::from(RATE_TABLE[index - 1]) - f64::from(RATE_TABLE[index]);
            let dk = keep_f64[index - 1] - keep_f64[index];
            credit -= d_lambda * dl + d_keep * dk;
            sides += 1.0;
        }
        if index + 1 < RATE_ENTRIES {
            let dl = f64::from(RATE_TABLE[index + 1]) - f64::from(RATE_TABLE[index]);
            let dk = keep_f64[index + 1] - keep_f64[index];
            credit += d_lambda * dl + d_keep * dk;
            sides += 1.0;
        }
        if sides > 0.0 {
            credit /= sides;
        }
        out[id] = (credit * (RATE_ENTRIES as f64)) as f32;
    }
    let _ = KEEP_TABLE;
    out
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn on_rate_select_fwd_values(pairs: &[f32]) {
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        if !st.capture_rate_fwd {
            return;
        }
        st.captured_rate_indices.reserve(pairs.len() / 2);
        for chunk in pairs.chunks_exact(2) {
            st.captured_rate_indices
                .push(rate_index_from_lambda_bits(chunk[0]) as u8);
        }
    });
}

/// Run `f` with every rate selection's table index recorded, in call order (`in_space_long`).
pub(crate) fn with_rate_indices<T>(f: impl FnOnce() -> Result<T>) -> Result<(T, Vec<u8>)> {
    /// Turns the capture off on every way out of [`with_rate_indices`], a panic included.
    struct CaptureOff;
    impl Drop for CaptureOff {
        fn drop(&mut self) {
            HOOK_STATE.with(|cell| cell.borrow_mut().capture_rate_fwd = false);
        }
    }
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        if st.capture_rate_fwd {
            return Err(invalid(
                "rate-index capture is already on; nested captures are refused",
            ));
        }
        st.capture_rate_fwd = true;
        st.captured_rate_indices.clear();
        Ok(())
    })?;
    let off = CaptureOff;
    let out = f();
    drop(off);
    let captured =
        HOOK_STATE.with(|cell| std::mem::take(&mut cell.borrow_mut().captured_rate_indices));
    Ok((out?, captured))
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn is_capturing_rate_fwd() -> bool {
    HOOK_STATE.with(|cell| cell.borrow().capture_rate_fwd)
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn transform_rate_select_bwd(
    score: &Tensor,
    grad: &Tensor,
    raw_d_score: Tensor,
) -> candle_core::Result<Tensor> {
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let Some(arm) = st.active_arm else {
            return Ok(raw_d_score);
        };
        let d_score = if st.analytic_keep_secant {
            let score_vec = score.flatten_all()?.to_vec1::<f32>()?;
            let grad_vec = grad.flatten_all()?.to_vec1::<f32>()?;
            let analytic = rate_select_bwd_analytic_keep_f64(&score_vec, &grad_vec);
            Tensor::from_vec(analytic, score.shape(), score.device())?
        } else {
            raw_d_score
        };

        let call_idx = st.rate_bwd_calls;
        st.rate_bwd_calls += 1;
        st.finite_bwd_checks_passed += 1;

        if st.step == 0 {
            let sq = d_score.sqr()?.sum_all()?.to_scalar::<f32>()?;
            if !sq.is_finite() {
                candle_core::bail!("I-Finite-CUDA failed: non-finite d_score at step 0");
            }
            st.step0_ds_sq_sum += f64::from(sq);
            let last_dim = grad.rank() - 1;
            let d_lam = grad.narrow(last_dim, 0, 1)?;
            let abs_sum = d_lam.abs()?.sum_all()?.to_scalar::<f32>()?;
            st.step0_d_lambda_abs_sum += f64::from(abs_sum);
            st.step0_d_lambda_count += d_lam.elem_count() as u64;
        }

        match arm {
            SecantArm::RecZeroCreditNull => {
                Tensor::zeros(score.shape(), DType::F32, score.device())
            }
            SecantArm::RecRandomRefitNull => {
                let dims = score.dims();
                let lanes = *dims
                    .last()
                    .ok_or_else(|| candle_core::Error::Msg("score lacks lane axis".into()))?;
                let mut rng = RademacherRng::new(st.seed, st.step, call_idx, 0x5245_435F_5241_4E44);
                let lane_signs = rng.signs(lanes);
                let mut bcast_shape = vec![1usize; dims.len()];
                bcast_shape[dims.len() - 1] = lanes;
                let xi = Tensor::from_vec(lane_signs, bcast_shape.as_slice(), score.device())?;
                d_score.broadcast_mul(&xi)
            }
            SecantArm::RecSecantActive
            | SecantArm::FbsSecantActive
            | SecantArm::FbsZeroCreditNull
            | SecantArm::FbsRandomRefitNull => Ok(d_score),
        }
    })
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
pub(crate) fn transform_spiralcore_fbs_bwd(
    q: &Tensor,
    raw_dq: Tensor,
    raw_dx: Tensor,
) -> candle_core::Result<(Tensor, Tensor)> {
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let Some(arm) = st.active_arm else {
            return Ok((raw_dq, raw_dx));
        };
        let call_idx = st.fbs_bwd_calls;
        st.fbs_bwd_calls += 1;
        st.finite_bwd_checks_passed += 1;

        if st.step == 0 {
            let sq = raw_dq.sqr()?.sum_all()?.to_scalar::<f32>()?;
            let dx_sq = raw_dx
                .to_dtype(DType::F32)?
                .sqr()?
                .sum_all()?
                .to_scalar::<f32>()?;
            if !sq.is_finite() || !dx_sq.is_finite() {
                candle_core::bail!("I-Finite-CUDA failed: non-finite dq or dx at step 0");
            }
            st.step0_dq_sq_sum += f64::from(sq);
        }

        match arm {
            SecantArm::FbsZeroCreditNull => {
                let zero_dq = Tensor::zeros(q.shape(), DType::F32, q.device())?;
                Ok((zero_dq, raw_dx))
            }
            SecantArm::FbsRandomRefitNull => {
                let mut rng = RademacherRng::new(st.seed, st.step, call_idx, 0x4642_535F_5241_4E44);
                let signs = rng.signs(q.elem_count());
                let xi = Tensor::from_vec(signs, q.shape(), q.device())?;
                Ok((raw_dq.mul(&xi)?, raw_dx))
            }
            SecantArm::FbsSecantActive
            | SecantArm::RecSecantActive
            | SecantArm::RecZeroCreditNull
            | SecantArm::RecRandomRefitNull => Ok((raw_dq, raw_dx)),
        }
    })
}

/// One batch of `(ids, targets)` of shape `batch * context`.
#[derive(Clone, Debug)]
pub struct BatchWindow {
    pub ids: Vec<u32>,
    pub targets: Vec<u32>,
}

/// Pinned 64-batch (`256`-row) diagnostic slice of `tokens.u16`.
#[derive(Clone, Debug)]
pub struct DiagnosticSlice {
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokenizer_sha256: String,
    pub payload_tokens: u64,
    pub batches: Vec<BatchWindow>,
}

impl DiagnosticSlice {
    /// Loads and verifies the pinned `UORT` corpus slice from disk.
    pub fn load_pinned(
        tokens_path: &Path,
        mask_path: &Path,
        tokenizer_path: &Path,
        num_batches: usize,
    ) -> Result<Self> {
        let tokens_bytes = fs::read(tokens_path)?;
        let mask_bytes = fs::read(mask_path)?;
        let tokenizer_bytes = fs::read(tokenizer_path)?;

        let tokens_sha256 = sha256_bytes(&tokens_bytes);
        let mask_sha256 = sha256_bytes(&mask_bytes);
        let tokenizer_sha256 = sha256_bytes(&tokenizer_bytes);

        if tokens_sha256 != PINNED_TOKENS_SHA256 {
            return Err(invalid(format!(
                "tokens.u16 SHA-256 mismatch: expected {PINNED_TOKENS_SHA256}, got {tokens_sha256}"
            )));
        }
        if mask_sha256 != PINNED_MASK_SHA256 {
            return Err(invalid(format!(
                "response_mask.u8 SHA-256 mismatch: expected {PINNED_MASK_SHA256}, got {mask_sha256}"
            )));
        }
        if tokenizer_sha256 != PINNED_TOKENIZER_SHA256 {
            return Err(invalid(format!(
                "tokenizer.json SHA-256 mismatch: expected {PINNED_TOKENIZER_SHA256}, got {tokenizer_sha256}"
            )));
        }

        Self::from_uort_bytes(
            &tokens_bytes,
            &mask_bytes,
            tokens_sha256,
            mask_sha256,
            tokenizer_sha256,
            num_batches,
            PROBE_BATCH,
            PROBE_CONTEXT,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_uort_bytes(
        tokens_bytes: &[u8],
        mask_bytes: &[u8],
        tokens_sha256: String,
        mask_sha256: String,
        tokenizer_sha256: String,
        num_batches: usize,
        batch_size: usize,
        context: usize,
    ) -> Result<Self> {
        if tokens_bytes.len() < UORT_HEADER_BYTES || &tokens_bytes[..4] != b"UORT" {
            return Err(invalid("tokens.u16 lacks canonical 64-byte UORT header"));
        }
        let version = u32::from_le_bytes([
            tokens_bytes[4],
            tokens_bytes[5],
            tokens_bytes[6],
            tokens_bytes[7],
        ]);
        let mut count_bytes = [0u8; 8];
        count_bytes.copy_from_slice(&tokens_bytes[8..16]);
        let payload_tokens = u64::from_le_bytes(count_bytes);
        let vocab = u32::from_le_bytes([
            tokens_bytes[16],
            tokens_bytes[17],
            tokens_bytes[18],
            tokens_bytes[19],
        ]);
        if version != 1
            || vocab as usize != PROBE_VOCAB
            || tokens_bytes[20..64].iter().any(|&b| b != 0)
        {
            return Err(invalid("UORT header version/vocab/reserved check failed"));
        }
        if payload_tokens * 2 + (UORT_HEADER_BYTES as u64) != tokens_bytes.len() as u64 {
            return Err(invalid("UORT payload byte length mismatch"));
        }
        if mask_bytes.len() as u64 != payload_tokens {
            return Err(invalid("response_mask.u8 length differs from UORT payload"));
        }

        let needed_rows = num_batches * batch_size;
        let needed_tokens = needed_rows * context + 1;
        if (payload_tokens as usize) < needed_tokens {
            return Err(invalid(
                "UORT corpus shorter than requested diagnostic slice",
            ));
        }

        let token_slice: Vec<u32> = tokens_bytes
            [UORT_HEADER_BYTES..UORT_HEADER_BYTES + needed_tokens * 2]
            .chunks_exact(2)
            .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
            .collect();
        if token_slice.iter().any(|&id| id as usize >= PROBE_VOCAB) {
            return Err(invalid("token id out of vocabulary range"));
        }

        let mut batches = Vec::with_capacity(num_batches);
        for b in 0..num_batches {
            let mut ids = Vec::with_capacity(batch_size * context);
            let mut targets = Vec::with_capacity(batch_size * context);
            for r in 0..batch_size {
                let row_idx = b * batch_size + r;
                let start = row_idx * context;
                ids.extend_from_slice(&token_slice[start..start + context]);
                targets.extend_from_slice(&token_slice[start + 1..start + context + 1]);
            }
            batches.push(BatchWindow { ids, targets });
        }

        Ok(Self {
            tokens_sha256,
            mask_sha256,
            tokenizer_sha256,
            payload_tokens,
            batches,
        })
    }

    /// Deterministic synthetic slice for Probe (a) and finite tests.
    pub fn synthetic(num_batches: usize, batch_size: usize, context: usize) -> Self {
        let cycle = [0u32, 1, 0, 2, 1, 3, 2, 4, 3, 5, 4, 6, 5, 7, 6, 8];
        let mut batches = Vec::with_capacity(num_batches);
        for b in 0..num_batches {
            let mut ids = Vec::with_capacity(batch_size * context);
            let mut targets = Vec::with_capacity(batch_size * context);
            for r in 0..batch_size {
                let offset = (b * batch_size + r) * 3;
                for t in 0..context {
                    ids.push(cycle[(offset + t) % cycle.len()]);
                    targets.push(cycle[(offset + t + 1) % cycle.len()]);
                }
            }
            batches.push(BatchWindow { ids, targets });
        }
        Self {
            tokens_sha256: "synthetic".into(),
            mask_sha256: "synthetic".into(),
            tokenizer_sha256: "synthetic".into(),
            payload_tokens: (num_batches * batch_size * context + 1) as u64,
            batches,
        }
    }
}

/// Active route choices `(goal[0..count], atom, state)` for one output block of a `ProjectionPlan`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FbsOutputRoute {
    pub goals: Vec<u32>,
    pub choices: Vec<u32>,
    pub atom: u32,
    pub state: u32,
}

/// Evaluates the exact route choices (`goal[stage]`, `choice[stage]`, `atom`, `state`) for every
/// output block of `plan` given query tensor slice `queries` (`[output_blocks, stages, 8]`).
pub fn extract_fbs_routes(plan: &ProjectionPlan, queries: &[f32]) -> Result<Vec<FbsOutputRoute>> {
    let expected = plan.output_blocks * plan.stages * 8;
    if queries.len() != expected {
        return Err(invalid("query slice length disagrees with ProjectionPlan"));
    }
    let nodes = plan.node_words();
    let g = &plan.geometry;
    let roots_f32: Vec<f32> = g.roots.iter().flatten().map(|&x| f32::from(x)).collect();
    let mut routes = Vec::with_capacity(plan.output_blocks);

    for output in 0..plan.output_blocks {
        let q = &queries[output * plan.stages * 8..(output + 1) * plan.stages * 8];
        let mut state = g.identity;
        let mut node = ABSENT;
        let mut goals = Vec::with_capacity(plan.stages);
        let mut choices = Vec::with_capacity(plan.stages);
        let mut final_atom = ABSENT;

        for stage in 0..plan.stages {
            let root = g.root_of_state[state as usize];
            let count = if stage == 0 { plan.frontier.len() } else { 3 };
            let mut selected = 0u32;
            let mut best = f32::MAX;
            let q_stage = &q[stage * 8..(stage + 1) * 8];
            for candidate in 0..(count as u32) {
                let goal = if stage == 0 {
                    candidate
                } else {
                    g.branch_targets[(root * 3 + candidate) as usize]
                };
                let r_slice = &roots_f32[(goal as usize) * 8..(goal as usize + 1) * 8];
                let dist: f32 = q_stage
                    .iter()
                    .zip(r_slice)
                    .map(|(a, b)| {
                        let d = a - b;
                        d * d
                    })
                    .sum();
                if dist < best {
                    best = dist;
                    selected = candidate;
                }
            }
            let goal = if stage == 0 {
                selected
            } else {
                g.branch_targets[(root * 3 + selected) as usize]
            };
            goals.push(goal);
            choices.push(selected);
            let from = g.root_of_state[state as usize];
            let action = g.bfs_actions[(from * 16 + goal) as usize];
            state = g.compose[(action * 64 + state) as usize];
            node = if stage == 0 {
                plan.frontier[selected as usize]
            } else {
                nodes[(node * 4 + selected) as usize]
            };
            let atom = nodes[(node * 4 + 3) as usize];
            if atom != ABSENT {
                final_atom = atom;
                break;
            }
        }
        routes.push(FbsOutputRoute {
            goals,
            choices,
            atom: final_atom,
            state,
        });
    }
    Ok(routes)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeepHistogramSnapshot {
    pub step: usize,
    pub loss: f64,
    pub bins: [u64; RATE_ENTRIES],
    pub plateau_12_15_fraction: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArmRunReceipt {
    pub arm: String,
    pub seed: u64,
    pub steps: usize,
    pub lr: f64,
    pub loss_step_0: f64,
    pub loss_step_64: f64,
    pub loss_step_128: f64,
    pub loss_step_256: f64,
    pub delta_loss_0_to_256: f64,
    pub step0_ds_l2_norm: f64,
    pub step0_dq_l2_norm: f64,
    pub step0_mean_abs_d_lambda: f64,
    pub cuda_error_checks_passed: u64,
    pub finite_bwd_checks_passed: u64,
    pub all_steps_finite: bool,
    pub rec_active_lane_token_crossings: u64,
    pub rec_active_lane_token_total: u64,
    pub rec_active_lane_token_crossings_frac: f64,
    pub rec_decay_param_crossings: usize,
    pub rec_decay_param_total: usize,
    pub rec_decay_param_crossings_frac: f64,
    pub fbs_query_stage_crossings: usize,
    pub fbs_query_stage_total: usize,
    pub fbs_query_stage_crossings_frac: f64,
    pub fbs_output_route_flips: usize,
    pub fbs_output_route_total: usize,
    pub fbs_output_route_flip_rate: f64,
    pub keep_histograms: Vec<KeepHistogramSnapshot>,
    pub wall_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentSummary {
    pub i_finite_cuda_pass: bool,
    pub i_boundary_crossings_rec_pass: bool,
    pub i_boundary_crossings_fbs_pass: bool,
    pub i_keep_plateau_pass: bool,
    pub max_rec_plateau_12_15_fraction: f64,
    pub min_rec_active_crossings_frac: f64,
    pub min_fbs_active_crossings_frac: f64,
    pub max_fbs_active_route_flip_rate: f64,
    pub analytic_keep_secant_used: bool,
    pub lr_rec_used: f64,
    pub lr_fbs_used: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateSummary {
    pub delta_noise_rec: f64,
    pub delta_noise_fbs: f64,
    pub rec_secant_seed1_delta: f64,
    pub rec_secant_seed2_delta: f64,
    pub rec_zero_seed1_delta: f64,
    pub rec_zero_seed2_delta: f64,
    pub rec_rand_seed1_delta: f64,
    pub rec_rand_seed2_delta: f64,
    pub rec_margin_over_controls_seed1: f64,
    pub rec_margin_over_controls_seed2: f64,
    pub g_rec_descent_pass: bool,
    pub g_rec_verdict: String,
    pub fbs_secant_seed1_delta: f64,
    pub fbs_secant_seed2_delta: f64,
    pub fbs_zero_seed1_delta: f64,
    pub fbs_zero_seed2_delta: f64,
    pub fbs_rand_seed1_delta: f64,
    pub fbs_rand_seed2_delta: f64,
    pub fbs_margin_over_controls_seed1: f64,
    pub fbs_margin_over_controls_seed2: f64,
    pub g_fbs_descent_pass: bool,
    pub g_fbs_verdict: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecantDescentSuiteReceipt {
    pub schema: String,
    pub item: String,
    pub issue: u32,
    pub prep_sheet: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokenizer_sha256: String,
    pub cuda_stack_cu_sha256: String,
    pub spiralcore_fbs_cu_sha256: String,
    pub spiralcore_fbs_compiler_sha256: String,
    pub shape: serde_json::Value,
    pub instruments: InstrumentSummary,
    pub gates: GateSummary,
    pub runs: Vec<ArmRunReceipt>,
    pub total_wall_ms: u64,
}

/// Constructs the registered rung-0 micro-probe `StackModel` for `seed`.
pub fn build_probe_model(seed: u64, device: &Device) -> Result<StackModel> {
    let mut config = StackConfig::transformer_control(seed);
    config.arch = StackArch::Geometric;
    config.pattern = "rrrrrr".into();
    config.width = PROBE_WIDTH;
    config.heads = PROBE_HEADS;
    config.mlp_hidden = PROBE_MLP_HIDDEN;
    config.context = PROBE_CONTEXT;
    config.vocab_size = PROBE_VOCAB;
    config.rotation = true;
    config.rotation_group = RotationGroup::Quaternion;
    config.gate = Gate::Hamilton;
    config.rec_gates = RecGates::Geometric;
    config.objective = Objective::MarginHinge {
        rivals: PROBE_RIVALS,
    };
    let mut model = StackModel::new(config, device)?;
    model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
    model.set_native_vector_head(PROBE_KEPT_CELLS)?;
    Ok(model)
}

fn snapshot_decay_indices(model: &StackModel) -> Result<Vec<usize>> {
    let mut indices = Vec::new();
    for (name, var) in model.variables() {
        if name.ends_with("rec.decay") {
            for v in var.as_tensor().to_vec1::<f32>()? {
                let idx =
                    ((v.clamp(0.0, 1.0) * (RATE_ENTRIES as f32)) as usize).min(RATE_ENTRIES - 1);
                indices.push(idx);
            }
        }
    }
    Ok(indices)
}

fn snapshot_fbs_routes(model: &StackModel) -> Result<BTreeMap<String, Vec<FbsOutputRoute>>> {
    let mut out = BTreeMap::new();
    for (name, var) in model.variables() {
        if name.ends_with(".spiralcore_fbs") {
            let plan = model
                .projection_plan_for_variable(name)
                .ok_or_else(|| invalid(format!("missing ProjectionPlan for {name}")))?;
            let flat = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            out.insert(name.clone(), extract_fbs_routes(plan, &flat)?);
        }
    }
    Ok(out)
}

fn evaluate_slice(
    model: &StackModel,
    slice: &DiagnosticSlice,
    step: usize,
) -> Result<(f64, KeepHistogramSnapshot, Vec<u8>)> {
    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.capture_rate_fwd = true;
        st.captured_rate_indices.clear();
    });

    let mut total_loss = 0.0f64;
    for batch in &slice.batches {
        let loss = model.loss(&batch.ids, &batch.targets, PROBE_BATCH, PROBE_CONTEXT)?;
        let val = loss.to_scalar::<f32>()?;
        if !val.is_finite() {
            return Err(invalid(format!(
                "non-finite evaluation loss at step {step}"
            )));
        }
        total_loss += f64::from(val);
    }
    let mean_loss = total_loss / (slice.batches.len() as f64);

    let captured = HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.capture_rate_fwd = false;
        std::mem::take(&mut st.captured_rate_indices)
    });

    let mut bins = [0u64; RATE_ENTRIES];
    for &idx in &captured {
        if (idx as usize) < RATE_ENTRIES {
            bins[idx as usize] += 1;
        }
    }
    let total: u64 = bins.iter().sum();
    let plateau = if total > 0 {
        ((bins[12] + bins[13] + bins[14] + bins[15]) as f64) / (total as f64)
    } else {
        0.0
    };

    Ok((
        mean_loss,
        KeepHistogramSnapshot {
            step,
            loss: mean_loss,
            bins,
            plateau_12_15_fraction: plateau,
        },
        captured,
    ))
}

/// Runs a single `(arm, seed)` configuration for `steps` optimizer steps on `slice`.
pub fn run_single_arm(
    arm: SecantArm,
    seed: u64,
    steps: usize,
    lr: f64,
    analytic_keep_secant: bool,
    slice: &DiagnosticSlice,
    device: &Device,
) -> Result<ArmRunReceipt> {
    let start = Instant::now();
    let model = build_probe_model(seed, device)?;
    let mut optimizer = crate::geometric_stack::StackAdamW::new(&model, 0.0, 1.0)?;

    let decay_0 = snapshot_decay_indices(&model)?;
    let fbs_routes_0 = snapshot_fbs_routes(&model)?;

    let (loss_0, hist_0, rate_indices_0) = evaluate_slice(&model, slice, 0)?;
    let mut keep_histograms = vec![hist_0];
    let mut loss_64 = loss_0;
    let mut loss_128 = loss_0;

    HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        *st = HookState {
            active_arm: Some(arm),
            seed,
            step: 0,
            rate_bwd_calls: 0,
            fbs_bwd_calls: 0,
            analytic_keep_secant,
            capture_rate_fwd: false,
            captured_rate_indices: Vec::new(),
            cuda_error_checks_passed: 0,
            finite_bwd_checks_passed: 0,
            step0_ds_sq_sum: 0.0,
            step0_dq_sq_sum: 0.0,
            step0_d_lambda_abs_sum: 0.0,
            step0_d_lambda_count: 0,
        };
    });

    for step in 0..steps {
        HOOK_STATE.with(|cell| {
            let mut st = cell.borrow_mut();
            st.step = step;
            st.rate_bwd_calls = 0;
            st.fbs_bwd_calls = 0;
        });

        let batch = &slice.batches[step % slice.batches.len()];
        let loss = model.loss(&batch.ids, &batch.targets, PROBE_BATCH, PROBE_CONTEXT)?;
        let loss_scalar = loss.to_scalar::<f32>()?;
        if !loss_scalar.is_finite() {
            return Err(invalid(format!(
                "I-Finite-CUDA failed: non-finite training loss at step {step}"
            )));
        }
        let grads = loss.backward()?;
        let grad_norm =
            optimizer.update_filtered(&model, &grads, lr, |name| arm.trainable_variable(name))?;
        if !grad_norm.is_finite() {
            return Err(invalid(format!(
                "I-Finite-CUDA failed: non-finite gradient norm at step {step}"
            )));
        }

        let completed = step + 1;
        if completed == 64 && steps >= 64 {
            let (l64, h64, _) = evaluate_slice(&model, slice, 64)?;
            loss_64 = l64;
            keep_histograms.push(h64);
        } else if completed == 128 && steps >= 128 {
            let (l128, h128, _) = evaluate_slice(&model, slice, 128)?;
            loss_128 = l128;
            keep_histograms.push(h128);
        }
    }

    let (loss_end, hist_end, rate_indices_end) = evaluate_slice(&model, slice, steps)?;
    keep_histograms.push(hist_end);

    let (
        cuda_error_checks_passed,
        finite_bwd_checks_passed,
        step0_ds_l2_norm,
        step0_dq_l2_norm,
        step0_mean_abs_d_lambda,
    ) = HOOK_STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.active_arm = None;
        let mean_dlam = if st.step0_d_lambda_count > 0 {
            st.step0_d_lambda_abs_sum / (st.step0_d_lambda_count as f64)
        } else {
            0.0
        };
        (
            st.cuda_error_checks_passed,
            st.finite_bwd_checks_passed,
            st.step0_ds_sq_sum.sqrt(),
            st.step0_dq_sq_sum.sqrt(),
            mean_dlam,
        )
    });

    let rec_active_lane_token_total = rate_indices_0.len() as u64;
    let rec_active_lane_token_crossings = rate_indices_0
        .iter()
        .zip(&rate_indices_end)
        .filter(|(a, b)| a != b)
        .count() as u64;
    let rec_active_lane_token_crossings_frac = if rec_active_lane_token_total > 0 {
        (rec_active_lane_token_crossings as f64) / (rec_active_lane_token_total as f64)
    } else {
        0.0
    };

    let decay_end = snapshot_decay_indices(&model)?;
    let rec_decay_param_total = decay_0.len();
    let rec_decay_param_crossings = decay_0
        .iter()
        .zip(&decay_end)
        .filter(|(a, b)| a != b)
        .count();
    let rec_decay_param_crossings_frac = if rec_decay_param_total > 0 {
        (rec_decay_param_crossings as f64) / (rec_decay_param_total as f64)
    } else {
        0.0
    };

    let fbs_routes_end = snapshot_fbs_routes(&model)?;
    let mut fbs_query_stage_crossings = 0usize;
    let mut fbs_query_stage_total = 0usize;
    let mut fbs_output_route_flips = 0usize;
    let mut fbs_output_route_total = 0usize;

    for (name, r0_list) in &fbs_routes_0 {
        let rend_list = &fbs_routes_end[name];
        for (r0, rend) in r0_list.iter().zip(rend_list) {
            fbs_output_route_total += 1;
            if r0.atom != rend.atom || r0.state != rend.state || r0.goals != rend.goals {
                fbs_output_route_flips += 1;
            }
            let stages = r0.goals.len().min(rend.goals.len());
            for s in 0..stages {
                fbs_query_stage_total += 1;
                if r0.goals[s] != rend.goals[s] {
                    fbs_query_stage_crossings += 1;
                }
            }
        }
    }

    let fbs_query_stage_crossings_frac = if fbs_query_stage_total > 0 {
        (fbs_query_stage_crossings as f64) / (fbs_query_stage_total as f64)
    } else {
        0.0
    };
    let fbs_output_route_flip_rate = if fbs_output_route_total > 0 {
        (fbs_output_route_flips as f64) / (fbs_output_route_total as f64)
    } else {
        0.0
    };

    Ok(ArmRunReceipt {
        arm: arm.name().into(),
        seed,
        steps,
        lr,
        loss_step_0: loss_0,
        loss_step_64: loss_64,
        loss_step_128: loss_128,
        loss_step_256: loss_end,
        delta_loss_0_to_256: loss_0 - loss_end,
        step0_ds_l2_norm,
        step0_dq_l2_norm,
        step0_mean_abs_d_lambda,
        cuda_error_checks_passed,
        finite_bwd_checks_passed,
        all_steps_finite: true,
        rec_active_lane_token_crossings,
        rec_active_lane_token_total,
        rec_active_lane_token_crossings_frac,
        rec_decay_param_crossings,
        rec_decay_param_total,
        rec_decay_param_crossings_frac,
        fbs_query_stage_crossings,
        fbs_query_stage_total,
        fbs_query_stage_crossings_frac,
        fbs_output_route_flips,
        fbs_output_route_total,
        fbs_output_route_flip_rate,
        keep_histograms,
        wall_ms: start.elapsed().as_millis() as u64,
    })
}

/// Evaluates `InstrumentSummary` and `GateSummary` from the 12 completed arm-seed runs.
pub fn summarize_suite(
    runs: &[ArmRunReceipt],
    lr_rec: f64,
    lr_fbs: f64,
    analytic_keep_secant: bool,
    slice: &DiagnosticSlice,
    total_wall_ms: u64,
) -> Result<SecantDescentSuiteReceipt> {
    let find = |arm: SecantArm, seed: u64| -> Result<&ArmRunReceipt> {
        runs.iter()
            .find(|r| r.arm == arm.name() && r.seed == seed)
            .ok_or_else(|| invalid(format!("missing run for {} seed {seed}", arm.name())))
    };

    let rec_s1 = find(SecantArm::RecSecantActive, 1)?;
    let rec_s2 = find(SecantArm::RecSecantActive, 2)?;
    let rec_z1 = find(SecantArm::RecZeroCreditNull, 1)?;
    let rec_z2 = find(SecantArm::RecZeroCreditNull, 2)?;
    let rec_r1 = find(SecantArm::RecRandomRefitNull, 1)?;
    let rec_r2 = find(SecantArm::RecRandomRefitNull, 2)?;

    let fbs_s1 = find(SecantArm::FbsSecantActive, 1)?;
    let fbs_s2 = find(SecantArm::FbsSecantActive, 2)?;
    let fbs_z1 = find(SecantArm::FbsZeroCreditNull, 1)?;
    let fbs_z2 = find(SecantArm::FbsZeroCreditNull, 2)?;
    let fbs_r1 = find(SecantArm::FbsRandomRefitNull, 1)?;
    let fbs_r2 = find(SecantArm::FbsRandomRefitNull, 2)?;

    let i_finite_cuda_pass = runs.iter().all(|r| {
        r.all_steps_finite
            && r.cuda_error_checks_passed > 0
            && r.step0_ds_l2_norm > 0.0
            && r.step0_dq_l2_norm > 0.0
    });

    let rec_cross_s1 = rec_s1
        .rec_active_lane_token_crossings_frac
        .max(rec_s1.rec_decay_param_crossings_frac);
    let rec_cross_s2 = rec_s2
        .rec_active_lane_token_crossings_frac
        .max(rec_s2.rec_decay_param_crossings_frac);
    let min_rec_active_crossings_frac = rec_cross_s1.min(rec_cross_s2);
    let i_boundary_crossings_rec_pass = min_rec_active_crossings_frac >= 0.01;

    let min_fbs_active_crossings_frac = fbs_s1
        .fbs_query_stage_crossings_frac
        .min(fbs_s2.fbs_query_stage_crossings_frac);
    let max_fbs_active_route_flip_rate = fbs_s1
        .fbs_output_route_flip_rate
        .max(fbs_s2.fbs_output_route_flip_rate);
    let i_boundary_crossings_fbs_pass = min_fbs_active_crossings_frac >= 0.01;

    let max_rec_plateau_12_15_fraction = [rec_s1, rec_s2]
        .into_iter()
        .flat_map(|r| r.keep_histograms.iter().map(|h| h.plateau_12_15_fraction))
        .fold(0.0f64, f64::max);
    let i_keep_plateau_pass = !(max_rec_plateau_12_15_fraction > 0.80
        && rec_s1
            .step0_mean_abs_d_lambda
            .min(rec_s2.step0_mean_abs_d_lambda)
            < 1e-7);

    let delta_noise_rec = [
        (rec_s1.delta_loss_0_to_256 - rec_s2.delta_loss_0_to_256).abs(),
        (rec_z1.delta_loss_0_to_256 - rec_z2.delta_loss_0_to_256).abs(),
        (rec_r1.delta_loss_0_to_256 - rec_r2.delta_loss_0_to_256).abs(),
    ]
    .into_iter()
    .fold(0.0f64, f64::max);

    let delta_noise_fbs = [
        (fbs_s1.delta_loss_0_to_256 - fbs_s2.delta_loss_0_to_256).abs(),
        (fbs_z1.delta_loss_0_to_256 - fbs_z2.delta_loss_0_to_256).abs(),
        (fbs_r1.delta_loss_0_to_256 - fbs_r2.delta_loss_0_to_256).abs(),
    ]
    .into_iter()
    .fold(0.0f64, f64::max);

    let rec_ctrl_min_s1 = rec_z1.loss_step_256.min(rec_r1.loss_step_256);
    let rec_ctrl_min_s2 = rec_z2.loss_step_256.min(rec_r2.loss_step_256);
    let rec_margin_s1 = rec_ctrl_min_s1 - rec_s1.loss_step_256;
    let rec_margin_s2 = rec_ctrl_min_s2 - rec_s2.loss_step_256;
    let g_rec_descent_pass = i_finite_cuda_pass
        && i_boundary_crossings_rec_pass
        && i_keep_plateau_pass
        && rec_s1.loss_step_256 < rec_s1.loss_step_0
        && rec_s2.loss_step_256 < rec_s2.loss_step_0
        && rec_margin_s1 >= delta_noise_rec
        && rec_margin_s2 >= delta_noise_rec;

    let fbs_ctrl_min_s1 = fbs_z1.loss_step_256.min(fbs_r1.loss_step_256);
    let fbs_ctrl_min_s2 = fbs_z2.loss_step_256.min(fbs_r2.loss_step_256);
    let fbs_margin_s1 = fbs_ctrl_min_s1 - fbs_s1.loss_step_256;
    let fbs_margin_s2 = fbs_ctrl_min_s2 - fbs_s2.loss_step_256;
    let g_fbs_descent_pass = i_finite_cuda_pass
        && i_boundary_crossings_fbs_pass
        && fbs_s1.loss_step_256 < fbs_s1.loss_step_0
        && fbs_s2.loss_step_256 < fbs_s2.loss_step_0
        && fbs_margin_s1 >= delta_noise_fbs
        && fbs_margin_s2 >= delta_noise_fbs
        && max_fbs_active_route_flip_rate < 0.50;

    let plan_probe = ProjectionPlan::new(PROBE_WIDTH, PROBE_WIDTH)?;

    Ok(SecantDescentSuiteReceipt {
        schema: "r-secant-surrogate-descent-receipt/v1".into(),
        item: "r-secant-surrogate-descent".into(),
        issue: 2886,
        prep_sheet: "docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md".into(),
        tokens_sha256: slice.tokens_sha256.clone(),
        mask_sha256: slice.mask_sha256.clone(),
        tokenizer_sha256: slice.tokenizer_sha256.clone(),
        cuda_stack_cu_sha256: sha256_bytes(include_bytes!("cuda_stack.cu")),
        spiralcore_fbs_cu_sha256: plan_probe.kernel_sha256.clone(),
        spiralcore_fbs_compiler_sha256: plan_probe.compiler_sha256.clone(),
        shape: serde_json::json!({
            "pattern": "rrrrrr",
            "width": PROBE_WIDTH,
            "heads": PROBE_HEADS,
            "mlp_hidden": PROBE_MLP_HIDDEN,
            "batch": PROBE_BATCH,
            "context": PROBE_CONTEXT,
            "vocab": PROBE_VOCAB,
            "kept_cells": PROBE_KEPT_CELLS,
            "rivals": PROBE_RIVALS,
            "batches": slice.batches.len(),
            "steps": rec_s1.steps,
        }),
        instruments: InstrumentSummary {
            i_finite_cuda_pass,
            i_boundary_crossings_rec_pass,
            i_boundary_crossings_fbs_pass,
            i_keep_plateau_pass,
            max_rec_plateau_12_15_fraction,
            min_rec_active_crossings_frac,
            min_fbs_active_crossings_frac,
            max_fbs_active_route_flip_rate,
            analytic_keep_secant_used: analytic_keep_secant,
            lr_rec_used: lr_rec,
            lr_fbs_used: lr_fbs,
        },
        gates: GateSummary {
            delta_noise_rec,
            delta_noise_fbs,
            rec_secant_seed1_delta: rec_s1.delta_loss_0_to_256,
            rec_secant_seed2_delta: rec_s2.delta_loss_0_to_256,
            rec_zero_seed1_delta: rec_z1.delta_loss_0_to_256,
            rec_zero_seed2_delta: rec_z2.delta_loss_0_to_256,
            rec_rand_seed1_delta: rec_r1.delta_loss_0_to_256,
            rec_rand_seed2_delta: rec_r2.delta_loss_0_to_256,
            rec_margin_over_controls_seed1: rec_margin_s1,
            rec_margin_over_controls_seed2: rec_margin_s2,
            g_rec_descent_pass,
            g_rec_verdict: if g_rec_descent_pass {
                "PASS (mechanism descent)".into()
            } else {
                "FAIL (mechanism no-descent on 1-step dyadic secant configuration)".into()
            },
            fbs_secant_seed1_delta: fbs_s1.delta_loss_0_to_256,
            fbs_secant_seed2_delta: fbs_s2.delta_loss_0_to_256,
            fbs_zero_seed1_delta: fbs_z1.delta_loss_0_to_256,
            fbs_zero_seed2_delta: fbs_z2.delta_loss_0_to_256,
            fbs_rand_seed1_delta: fbs_r1.delta_loss_0_to_256,
            fbs_rand_seed2_delta: fbs_r2.delta_loss_0_to_256,
            fbs_margin_over_controls_seed1: fbs_margin_s1,
            fbs_margin_over_controls_seed2: fbs_margin_s2,
            g_fbs_descent_pass,
            g_fbs_verdict: if g_fbs_descent_pass {
                "PASS (mechanism descent)".into()
            } else {
                "FAIL (mechanism no-descent on unnormalized root-edge secant configuration)".into()
            },
        },
        runs: runs.to_vec(),
        total_wall_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rademacher_rng_preserves_l2_norm_and_is_deterministic() {
        let mut r1 = RademacherRng::new(1, 0, 0, 0x1234);
        let mut r2 = RademacherRng::new(1, 0, 0, 0x1234);
        let s1 = r1.signs(64);
        let s2 = r2.signs(64);
        assert_eq!(s1, s2);
        assert!(s1.iter().all(|&v| v == 1.0 || v == -1.0));
        assert!(s1.contains(&1.0) && s1.contains(&-1.0));

        let grad = (0..64)
            .map(|i| (i as f32 - 20.5) * 0.13)
            .collect::<Vec<_>>();
        let flipped: Vec<f32> = grad.iter().zip(&s1).map(|(g, s)| g * s).collect();
        let norm_orig: f32 = grad.iter().map(|x| x * x).sum();
        let norm_flip: f32 = flipped.iter().map(|x| x * x).sum();
        assert_eq!(norm_orig.to_bits(), norm_flip.to_bits());
    }

    #[test]
    fn rate_index_from_lambda_bits_recovers_all_sixteen_entries() {
        for (i, &lam) in RATE_TABLE.iter().enumerate() {
            assert_eq!(rate_index_from_lambda_bits(lam), i);
        }
    }

    #[test]
    fn analytic_keep_secant_supplies_nonzero_slope_across_keep_plateau() {
        let scores = [12.5f32 / 16.0, 13.5 / 16.0, 14.5 / 16.0, 15.5 / 16.0];
        let mut grads = Vec::new();
        for _ in 0..4 {
            grads.push(0.0f32); // d_lambda = 0
            grads.push(1.0f32); // d_keep = 1
        }
        let d_score = rate_select_bwd_analytic_keep_f64(&scores, &grads);
        for (k, &ds) in d_score.iter().enumerate() {
            assert!(
                ds > 0.0 && ds.is_finite(),
                "entry {} should have positive analytic keep slope, got {ds}",
                k + 12
            );
        }
    }

    #[test]
    fn uort_slice_loader_validates_header_and_builds_windows() -> Result<()> {
        let count = 400usize;
        let mut bytes = vec![0u8; UORT_HEADER_BYTES + count * 2];
        bytes[0..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(count as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&(PROBE_VOCAB as u32).to_le_bytes());
        for i in 0..count {
            let tok = (i % 100) as u16;
            bytes[UORT_HEADER_BYTES + i * 2..UORT_HEADER_BYTES + (i + 1) * 2]
                .copy_from_slice(&tok.to_le_bytes());
        }
        let mask = vec![1u8; count];
        let slice = DiagnosticSlice::from_uort_bytes(
            &bytes,
            &mask,
            "tok_sha".into(),
            "mask_sha".into(),
            "tkz_sha".into(),
            2,
            2,
            16,
        )?;
        assert_eq!(slice.batches.len(), 2);
        assert_eq!(slice.batches[0].ids.len(), 32);
        assert_eq!(slice.batches[0].targets.len(), 32);
        assert_eq!(slice.batches[0].ids[0], 0);
        assert_eq!(slice.batches[0].targets[0], 1);
        Ok(())
    }

    #[test]
    fn extract_fbs_routes_tracks_root_boundary_crossings() -> Result<()> {
        let plan = ProjectionPlan::new(256, 256)?;
        let n = plan.output_blocks * plan.stages * 8;
        let q0 = vec![0.1f32; n];
        let mut q1 = q0.clone();
        q1[0] = -4.0;
        q1[1] = -4.0;
        let r0 = extract_fbs_routes(&plan, &q0)?;
        let r1 = extract_fbs_routes(&plan, &q1)?;
        assert_eq!(r0.len(), plan.output_blocks);
        assert_ne!(r0[0].goals[0], r1[0].goals[0]);
        assert_eq!(r0[1].goals, r1[1].goals);
        Ok(())
    }
}
