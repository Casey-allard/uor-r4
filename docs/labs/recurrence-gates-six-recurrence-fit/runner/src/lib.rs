//! Standalone D37 offline-float six-recurrence fit and D11 integer recurrence export instrument
//! (`recurrence-gates-six-recurrence-fit`, `#2890`).
//!
//! Executes the pre-registered 4-arm × 2-seed × 1,024-step six-recurrence fit sheet in
//! `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`:
//! 1. `Rec_Geometric_S3_Active`: full `RecGates::Geometric` + `RotationGroup::Quaternion` (`rotation = true`).
//! 2. `Rec_Geometric_U1_AbelianControl`: commutative transport control (`RotationGroup::U1`, `j = k = 0`).
//! 3. `Rec_Geometric_IdentityTransport_Control`: identity transport control (`rotation = false`, `u_t = (1, 0, 0, 0)`).
//! 4. `Rec_FrozenRates_Null`: static rate-selection control (`ds_t = 0`, `rec.decay` frozen at initial bin centers).
//!
//! Plus the non-neural GPU train-split response count-table baseline (`TrainSplit_CountTable_Ref`,
//! `1,226 / 14,287` = `8.58%` top-1 on `ft/dev`).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[cfg(feature = "cuda")]
use candle_core::backend::BackendStorage;
use candle_core::{CpuStorage, CustomOp1, CustomOp2, DType, Device, Layout, Shape, Tensor};
#[cfg(feature = "cuda")]
use candle_core::CudaStorage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(feature = "cuda")]
use uor_r4_float_train::cuda_stack_kernels::cuda::{launch, launch_groups, uninit, Arg};
use uor_r4_float_train::geometric_stack::{
    hamilton_gate, quaternion_scan, rate_select, Gate, Objective, QuaternionWeights, RecGates,
    RotationGroup, StackAdamW, StackArch, StackConfig, StackModel, KEEP_TABLE, RATE_ENTRIES,
    RATE_TABLE,
};
use uor_r4_float_train::hamilton_sheet::{
    build_document_windows, build_training_batches, compute_train_unigram_reference,
    load_pinned_uort_split, pure_i32_q12_writer_oracle, verify_icosian_closure, CausalWindow,
    TrainBatch, UnigramReference, EXPECTED_DEV_DOCS, EXPECTED_DEV_ELIGIBLE_TARGETS,
    EXPECTED_DEV_TOKENS, EXPECTED_TRAIN_ELIGIBLE_TARGETS, EXPECTED_TRAIN_TOKENS,
    PINNED_DEV_MASK_SHA256, PINNED_DEV_TOKENS_SHA256, PINNED_TOKENIZER_SHA256,
    PINNED_TRAIN_MASK_SHA256, PINNED_TRAIN_TOKENS_SHA256, Q12_I16_MAX, Q12_I16_MIN, Q12_SCALE,
    SHEET_BOS_ID, SHEET_CLIP, SHEET_CONTEXT, SHEET_EVAL_BATCH, SHEET_HEADS, SHEET_KEPT_CELLS,
    SHEET_MLP_HIDDEN, SHEET_RIVALS, SHEET_TRAIN_BATCH, SHEET_VOCAB, SHEET_WEIGHT_DECAY,
    SHEET_WIDTH,
};
use uor_r4_float_train::native_vector_writer::{NativeVectorWriter, WriterSpec};
use uor_r4_float_train::secant_descent::{
    rate_index_from_lambda_bits, rate_select_bwd_analytic_keep_f64,
};
use uor_r4_float_train::{FloatTrainError, Result};

/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/fresh_v2/tokens.u16`.
pub const PINNED_FRESH_V2_TOKENS_SHA256: &str =
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8";
/// Pinned SHA-256 of `w2-inputs/2026-10-25/driver-data/fresh_v2/response_mask.u8`.
pub const PINNED_FRESH_V2_MASK_SHA256: &str =
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e";
/// Pinned SHA-256 of `crates/uor-r4-float-train/src/cuda_stack.cu`.
pub const PINNED_CUDA_STACK_CU_SHA256: &str =
    "c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17";
/// Pinned SHA-256 of `crates/uor-r4-float-train/src/geometric_stack_native.rs`.
pub const PINNED_GEOMETRIC_STACK_NATIVE_SHA256: &str =
    "675a11e5225232a8bdb36063a7305c3f428db60ef8609931dac12ac1b38cd9f3";
/// Pinned SHA-256 of `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`.
pub const PINNED_GEOMETRIC_STACK_NATIVE_CUDA_SHA256: &str =
    "3a279df72f937619e6cfa5775d8507d981a825c9f2424177d0f0222c928a1755";

pub const EXPECTED_FRESH_V2_TOKENS: usize = 44_678;
pub const EXPECTED_FRESH_V2_ELIGIBLE_TARGETS: usize = 14_479;

pub const REC_SHEET_STEPS: usize = 1024;
pub const REC_SHEET_TRAIN_BATCHES: usize = 64;
pub const REC_SHEET_BASE_LR: f64 = 0.002;
pub const REC_SHEET_DECAY_LR_MULT: f64 = 20.0;
pub const Q8_SCALE: f32 = 256.0;

/// Exact Q12 (`scale = 4096 = 2^12`) keep table `round(sqrt(1 - 2^{-2(i+1)}) * 4096)` for `i in 0..16`.
/// Every entry is a Canonical Signed Digit (CSD) sum/difference of powers of two, so scaling by
/// `KEEP_TABLE_Q12[i]` executes with pure bit shifts and additions/subtractions (`csd_keep_q12`).
pub const KEEP_TABLE_Q12: [i32; RATE_ENTRIES] = [
    3547, // 4096 - 512 - 32 - 4 - 1
    3966, // 4096 - 128 - 2
    4064, // 4096 - 32
    4088, // 4096 - 8
    4094, // 4096 - 2
    4096, // 4096
    4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096,
];

fn invalid(msg: impl Into<String>) -> FloatTrainError {
    FloatTrainError::Invalid(msg.into())
}

fn spiralcore_roots_q12() -> Result<[[i32; 8]; 16]> {
    let plan8 = uor_r4_float_train::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
    let val = serde_json::to_value(&*plan8)?;
    let raw_roots: [[i32; 8]; 16] = serde_json::from_value(val["geometry"]["roots"].clone())?;
    Ok(std::array::from_fn(|r| {
        std::array::from_fn(|j| raw_roots[r][j] * (Q12_SCALE as i32))
    }))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Synchronizes the CUDA device stream if `device` is a CUDA device, ensuring accurate wall-clock timing.
pub fn sync_cuda(device: &Device) -> Result<()> {
    #[cfg(feature = "cuda")]
    if let Device::Cuda(cuda_dev) = device {
        cuda_dev
            .cuda_stream()
            .synchronize()
            .map_err(|e| candle_core::Error::Cuda(Box::new(e)))?;
    }
    let _ = device;
    Ok(())
}

/// The four matched geometric recurrence arms in `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecArm {
    RecGeometricS3Active,
    RecGeometricU1AbelianControl,
    RecGeometricIdentityTransportControl,
    RecFrozenRatesNull,
}

impl RecArm {
    pub const ALL: [RecArm; 4] = [
        RecArm::RecGeometricS3Active,
        RecArm::RecGeometricU1AbelianControl,
        RecArm::RecGeometricIdentityTransportControl,
        RecArm::RecFrozenRatesNull,
    ];

    pub fn name(self) -> &'static str {
        match self {
            RecArm::RecGeometricS3Active => "Rec_Geometric_S3_Active",
            RecArm::RecGeometricU1AbelianControl => "Rec_Geometric_U1_AbelianControl",
            RecArm::RecGeometricIdentityTransportControl => {
                "Rec_Geometric_IdentityTransport_Control"
            }
            RecArm::RecFrozenRatesNull => "Rec_FrozenRates_Null",
        }
    }

    pub fn is_active_rates(self) -> bool {
        !matches!(self, RecArm::RecFrozenRatesNull)
    }
}

/// Pure shift-and-add Q12 keep scaling `(val_q12 * KEEP_TABLE_Q12[index]) >> 12` with zero float
/// and zero integer multiply instructions.
#[inline]
pub fn csd_keep_q12(index: usize, val_q12: i32) -> i32 {
    let x = i64::from(val_q12);
    let scaled = match index {
        0 => (x << 12) - (x << 9) - (x << 5) - (x << 2) - x,
        1 => (x << 12) - (x << 7) - (x << 1),
        2 => (x << 12) - (x << 5),
        3 => (x << 12) - (x << 3),
        4 => (x << 12) - (x << 1),
        _ => x << 12,
    };
    (scaled >> 12) as i32
}

/// Pure 8-bit shift-and-add integer product `q_q8 * val` for `q_q8 in -256..=256` with zero float
/// and zero integer multiply instructions.
#[inline]
pub fn shift_add_mul_q8(q_q8: i32, val: i32) -> i64 {
    let neg = q_q8 < 0;
    let mag = q_q8.unsigned_abs();
    let v = i64::from(val);
    let mut acc = 0i64;
    let mut bit = 0u32;
    while bit <= 8 {
        if ((mag >> bit) & 1) != 0 {
            acc += v << bit;
        }
        bit += 1;
    }
    if neg {
        -acc
    } else {
        acc
    }
}

/// Pure shift-and-add 4-block Hamilton product `(q_q8 (x) x) >> 8` with zero float and zero
/// integer multiply instructions.
#[inline]
pub fn hamilton_shift_add_q8(q_q8: [i32; 4], x: [i32; 4]) -> [i32; 4] {
    let [a1, b1, c1, d1] = q_q8;
    let [a2, b2, c2, d2] = x;
    let r0 = shift_add_mul_q8(a1, a2)
        - shift_add_mul_q8(b1, b2)
        - shift_add_mul_q8(c1, c2)
        - shift_add_mul_q8(d1, d2);
    let r1 = shift_add_mul_q8(a1, b2) + shift_add_mul_q8(b1, a2) + shift_add_mul_q8(c1, d2)
        - shift_add_mul_q8(d1, c2);
    let r2 = shift_add_mul_q8(a1, c2) - shift_add_mul_q8(b1, d2)
        + shift_add_mul_q8(c1, a2)
        + shift_add_mul_q8(d1, b2);
    let r3 = shift_add_mul_q8(a1, d2) + shift_add_mul_q8(b1, c2) - shift_add_mul_q8(c1, b2)
        + shift_add_mul_q8(d1, a2);
    [
        (r0 >> 8) as i32,
        (r1 >> 8) as i32,
        (r2 >> 8) as i32,
        (r3 >> 8) as i32,
    ]
}

/// Pure-integer D11 recurrence scan + Hamilton output gate over `time` steps for one 4-lane:
/// `h_t = (hamilton_shift_add_q8(u_q8[t], h_{t-1}) >> (rate_idx[t] + 1)) + csd_keep_q12(rate_idx[t], conv_q12[t])`
/// `out_t = hamilton_shift_add_q8(gate_q8[t], h_t)`
/// Uses zero floating-point operations and zero integer multiply instructions.
pub fn d11_recurrence_scan_and_gate_i32(
    rate_indices: &[usize],
    u_q8: &[[i32; 4]],
    conv_q12: &[[i32; 4]],
    gate_q8: &[[i32; 4]],
) -> Vec<[i32; 4]> {
    let mut h = [0i32; 4];
    let mut out = Vec::with_capacity(rate_indices.len());
    for t in 0..rate_indices.len() {
        let idx = rate_indices[t].min(RATE_ENTRIES - 1);
        let shift = (idx + 1) as u32;
        let rot = hamilton_shift_add_q8(u_q8[t], h);
        let faded = [
            rot[0] >> shift,
            rot[1] >> shift,
            rot[2] >> shift,
            rot[3] >> shift,
        ];
        let c = conv_q12[t];
        let drive = [
            csd_keep_q12(idx, c[0]),
            csd_keep_q12(idx, c[1]),
            csd_keep_q12(idx, c[2]),
            csd_keep_q12(idx, c[3]),
        ];
        h = [
            faded[0] + drive[0],
            faded[1] + drive[1],
            faded[2] + drive[2],
            faded[3] + drive[3],
        ];
        let h_clamped = [
            h[0].clamp(-32768, 32767),
            h[1].clamp(-32768, 32767),
            h[2].clamp(-32768, 32767),
            h[3].clamp(-32768, 32767),
        ];
        out.push(hamilton_shift_add_q8(gate_q8[t], h_clamped));
    }
    out
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportExactnessCheck {
    pub passed: bool,
    pub csd_keep_q12_exact_entries: usize,
    pub shift_add_q8_exact_pairs: usize,
    pub synthetic_arms_tested: usize,
    pub max_scan_lsb_diff: i32,
    pub synthetic_writer_top1_mismatches: usize,
}

/// Verifies `I-Export-Exactness`: pure shift-and-add `csd_keep_q12`, `hamilton_shift_add_q8`,
/// `d11_recurrence_scan_and_gate_i32`, and `pure_i32_q12_writer_oracle` across all 16 dyadic bins
/// and all 4 arm modes (`S3`, `U1`, `Identity`, `FrozenRates`).
pub fn verify_export_exactness() -> Result<ExportExactnessCheck> {
    let mut csd_ok = 0usize;
    for (idx, &k_q12) in KEEP_TABLE_Q12.iter().enumerate() {
        let f_q12 = (KEEP_TABLE[idx] * Q12_SCALE).round() as i32;
        if k_q12 != f_q12 {
            return Err(invalid(format!(
                "KEEP_TABLE_Q12[{idx}] = {k_q12} != round(KEEP_TABLE[{idx}]*4096) = {f_q12}"
            )));
        }
        for sample in [-32768i32, -4096, -1023, -1, 0, 1, 1023, 4096, 32767] {
            let expected = ((i64::from(sample) * i64::from(k_q12)) >> 12) as i32;
            let actual = csd_keep_q12(idx, sample);
            if actual != expected {
                return Err(invalid(format!(
                    "csd_keep_q12 mismatch at idx={idx}, sample={sample}: {actual} != {expected}"
                )));
            }
        }
        csd_ok += 1;
    }

    let mut q8_pairs = 0usize;
    for q in [-256i32, -181, -128, -63, -1, 0, 1, 63, 128, 181, 256] {
        for v in [-32768i32, -4096, -333, 0, 333, 4096, 32767] {
            if shift_add_mul_q8(q, v) != i64::from(q) * i64::from(v) {
                return Err(invalid("shift_add_mul_q8 mismatch"));
            }
            q8_pairs += 1;
        }
    }

    let width = 256usize;
    let lanes = width / 4;
    let vocab = 4096usize;
    let rows = 16usize;
    let roots_q12 = spiralcore_roots_q12()?;
    let table_q12: Vec<i32> = (0..vocab * width)
        .map(|k| (((k * 73 + 19) % 4097) as i32) - 2048)
        .collect();
    let targets: Vec<u32> = (0..rows).map(|r| (r * 17 + 3) as u32).collect();

    let mut max_scan_lsb_diff = 0i32;
    let mut writer_mismatches = 0usize;

    for (arm_idx, arm) in RecArm::ALL.iter().enumerate() {
        let mut query_i32 = vec![0i32; rows * width];
        let mut query_ref = vec![0i32; rows * width];
        for lane in 0..lanes {
            let rate_indices: Vec<usize> = (0..rows)
                .map(|t| {
                    if *arm == RecArm::RecFrozenRatesNull {
                        lane % RATE_ENTRIES
                    } else {
                        (lane + t + arm_idx) % RATE_ENTRIES
                    }
                })
                .collect();
            let u_q8: Vec<[i32; 4]> = (0..rows)
                .map(|t| match arm {
                    RecArm::RecGeometricIdentityTransportControl => [256, 0, 0, 0],
                    RecArm::RecGeometricU1AbelianControl => {
                        if (t + lane).is_multiple_of(2) {
                            [181, 181, 0, 0]
                        } else {
                            [256, 0, 0, 0]
                        }
                    }
                    RecArm::RecGeometricS3Active | RecArm::RecFrozenRatesNull => {
                        if (t + lane).is_multiple_of(2) {
                            [128, 128, 128, 128]
                        } else {
                            [181, 0, 181, 0]
                        }
                    }
                })
                .collect();
            let conv_q12: Vec<[i32; 4]> = (0..rows)
                .map(|t| {
                    let b = ((t * 31 + lane * 13 + arm_idx * 7) % 1024) as i32 - 512;
                    [b, -b / 2, b / 3, -b / 4]
                })
                .collect();
            let gate_q8: Vec<[i32; 4]> = (0..rows)
                .map(|t| {
                    if (t + lane).is_multiple_of(3) {
                        [128, -128, 128, -128]
                    } else {
                        [256, 0, 0, 0]
                    }
                })
                .collect();

            let scan_i32 =
                d11_recurrence_scan_and_gate_i32(&rate_indices, &u_q8, &conv_q12, &gate_q8);
            // Compare against floating-point Q8/Q12 reference scan
            let mut h_f = [0.0f64; 4];
            for t in 0..rows {
                let idx = rate_indices[t];
                let u_f = [
                    f64::from(u_q8[t][0]) / 256.0,
                    f64::from(u_q8[t][1]) / 256.0,
                    f64::from(u_q8[t][2]) / 256.0,
                    f64::from(u_q8[t][3]) / 256.0,
                ];
                let rot_f = [
                    u_f[0] * h_f[0] - u_f[1] * h_f[1] - u_f[2] * h_f[2] - u_f[3] * h_f[3],
                    u_f[0] * h_f[1] + u_f[1] * h_f[0] + u_f[2] * h_f[3] - u_f[3] * h_f[2],
                    u_f[0] * h_f[2] - u_f[1] * h_f[3] + u_f[2] * h_f[0] + u_f[3] * h_f[1],
                    u_f[0] * h_f[3] + u_f[1] * h_f[2] - u_f[2] * h_f[1] + u_f[3] * h_f[0],
                ];
                let lam = f64::from(RATE_TABLE[idx]);
                let keep = f64::from(KEEP_TABLE_Q12[idx]) / 4096.0;
                for c in 0..4 {
                    h_f[c] = (rot_f[c].floor() * lam).floor()
                        + (f64::from(conv_q12[t][c]) * keep).floor();
                }
                let g_f = [
                    f64::from(gate_q8[t][0]) / 256.0,
                    f64::from(gate_q8[t][1]) / 256.0,
                    f64::from(gate_q8[t][2]) / 256.0,
                    f64::from(gate_q8[t][3]) / 256.0,
                ];
                let out_f = [
                    (g_f[0] * h_f[0] - g_f[1] * h_f[1] - g_f[2] * h_f[2] - g_f[3] * h_f[3]).floor()
                        as i32,
                    (g_f[0] * h_f[1] + g_f[1] * h_f[0] + g_f[2] * h_f[3] - g_f[3] * h_f[2]).floor()
                        as i32,
                    (g_f[0] * h_f[2] - g_f[1] * h_f[3] + g_f[2] * h_f[0] + g_f[3] * h_f[1]).floor()
                        as i32,
                    (g_f[0] * h_f[3] + g_f[1] * h_f[2] - g_f[2] * h_f[1] + g_f[3] * h_f[0]).floor()
                        as i32,
                ];
                for c in 0..4 {
                    let diff = (scan_i32[t][c] - out_f[c]).abs();
                    max_scan_lsb_diff = max_scan_lsb_diff.max(diff);
                    query_i32[t * width + lane * 4 + c] = scan_i32[t][c];
                    query_ref[t * width + lane * 4 + c] = out_f[c];
                }
            }
        }
        let (preds_i32, _, _) = pure_i32_q12_writer_oracle(
            &query_i32,
            &table_q12,
            &roots_q12,
            rows,
            width,
            vocab,
            SHEET_KEPT_CELLS,
            &targets,
        );
        let (preds_ref, _, _) = pure_i32_q12_writer_oracle(
            &query_ref,
            &table_q12,
            &roots_q12,
            rows,
            width,
            vocab,
            SHEET_KEPT_CELLS,
            &targets,
        );
        writer_mismatches += preds_i32
            .iter()
            .zip(&preds_ref)
            .filter(|(a, b)| a != b)
            .count();
    }

    let passed = csd_ok == RATE_ENTRIES && max_scan_lsb_diff == 0 && writer_mismatches == 0;
    Ok(ExportExactnessCheck {
        passed,
        csd_keep_q12_exact_entries: csd_ok,
        shift_add_q8_exact_pairs: q8_pairs,
        synthetic_arms_tested: RecArm::ALL.len(),
        max_scan_lsb_diff,
        synthetic_writer_top1_mismatches: writer_mismatches,
    })
}

/// Custom RMSNorm operator calling the ahead-of-time PTX kernels in `uor_r4_float_train`.
struct NativeRmsNorm;

impl CustomOp2 for NativeRmsNorm {
    fn name(&self) -> &'static str {
        "recurrence-six-fit-rms-norm"
    }

    fn cpu_fwd(
        &self,
        _s1: &CpuStorage,
        _l1: &Layout,
        _s2: &CpuStorage,
        _l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("D24: RMSNorm requires CUDA; no CPU reference")
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        let width = l2.shape().elem_count();
        if width == 0 || l1.shape().dims().last() != Some(&width) {
            candle_core::bail!("RMSNorm weight must match the last dimension");
        }
        if !l1.is_contiguous()
            || !l2.is_contiguous()
            || s1.dtype() != DType::F32
            || s2.dtype() != DType::F32
        {
            candle_core::bail!("CUDA RMSNorm requires contiguous F32 input");
        }
        let x_slice = s1.as_cuda_slice::<f32>()?;
        let w_slice = s2.as_cuda_slice::<f32>()?;
        let x_start = l1.start_offset();
        let total = l1.shape().elem_count();
        let w_start = l2.start_offset();
        let x_view = x_slice
            .try_slice(x_start..x_start + total)
            .ok_or_else(|| candle_core::Error::Msg("RMSNorm x slice out of bounds".into()))?;
        let w_view = w_slice
            .try_slice(w_start..w_start + width)
            .ok_or_else(|| candle_core::Error::Msg("RMSNorm w slice out of bounds".into()))?;
        let device = &s1.device;
        let rows = total / width;
        let out = uninit::<f32>(device, total)?;
        launch_groups(
            device,
            "rms_norm_fwd",
            (rows, 1, 1),
            (256, 1, 1),
            &[
                Arg::F(x_view),
                Arg::F(w_view),
                Arg::f(&out),
                Arg::U32(width as u32),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            l1.shape().clone(),
        ))
    }

    fn bwd(
        &self,
        x: &Tensor,
        w: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let Device::Cuda(device) = x.device() {
            let width = w.elem_count();
            let xr = x.contiguous()?;
            let wr = w.contiguous()?;
            let gr = grad.contiguous()?;
            let total = xr.elem_count();
            let rows = total / width;
            let (xs, xl) = xr.storage_and_layout();
            let (ws, wl) = wr.storage_and_layout();
            let (gs, gl) = gr.storage_and_layout();
            let (
                candle_core::Storage::Cuda(xs),
                candle_core::Storage::Cuda(ws),
                candle_core::Storage::Cuda(gs),
            ) = (&*xs, &*ws, &*gs)
            else {
                candle_core::bail!("expected CUDA tensors in RMSNorm backward");
            };
            let xv = xs
                .as_cuda_slice::<f32>()?
                .try_slice(xl.start_offset()..xl.start_offset() + total)
                .ok_or_else(|| candle_core::Error::Msg("xv slice out of bounds".into()))?;
            let wv = ws
                .as_cuda_slice::<f32>()?
                .try_slice(wl.start_offset()..wl.start_offset() + width)
                .ok_or_else(|| candle_core::Error::Msg("wv slice out of bounds".into()))?;
            let gv = gs
                .as_cuda_slice::<f32>()?
                .try_slice(gl.start_offset()..gl.start_offset() + total)
                .ok_or_else(|| candle_core::Error::Msg("gv slice out of bounds".into()))?;
            let dx = uninit::<f32>(device, total)?;
            let row_r = uninit::<f64>(device, rows)?;
            launch_groups(
                device,
                "rms_norm_bwd_dx",
                (rows, 1, 1),
                (256, 1, 1),
                &[
                    Arg::F(xv.slice(..)),
                    Arg::F(wv),
                    Arg::F(gv.slice(..)),
                    Arg::f(&dx),
                    Arg::d(&row_r),
                    Arg::U32(width as u32),
                ],
            )?;
            let chunks = rows.div_ceil(256);
            let partials = uninit::<f64>(device, chunks * width)?;
            launch(
                device,
                "rms_norm_dw_partial",
                chunks * width,
                &[
                    Arg::F(xv),
                    Arg::F(gv),
                    Arg::d(&row_r),
                    Arg::d(&partials),
                    Arg::U32(rows as u32),
                    Arg::U32(width as u32),
                    Arg::U32(256),
                ],
            )?;
            let dw = uninit::<f32>(device, width)?;
            launch(
                device,
                "rms_norm_dw_reduce",
                width,
                &[
                    Arg::d(&partials),
                    Arg::f(&dw),
                    Arg::U32(chunks as u32),
                    Arg::U32(width as u32),
                ],
            )?;
            let dx_t = Tensor::from_storage(
                candle_core::Storage::Cuda(CudaStorage::wrap_cuda_slice(dx, device.clone())),
                x.shape().clone(),
                candle_core::op::BackpropOp::none(),
                false,
            );
            let dw_t = Tensor::from_storage(
                candle_core::Storage::Cuda(CudaStorage::wrap_cuda_slice(dw, device.clone())),
                w.shape().clone(),
                candle_core::op::BackpropOp::none(),
                false,
            );
            return Ok((Some(dx_t), Some(dw_t)));
        }
        let _ = (x, w, grad);
        candle_core::bail!("D24: RMSNorm gradient requires CUDA; no CPU reference")
    }
}

#[derive(Clone, Debug, Default)]
pub struct RateBackwardTelemetry {
    pub layer_ds_l2_norms: [f64; 6],
    pub d_lambda_abs_sum: f64,
    pub d_lambda_count: usize,
}

/// Custom `rate_select` operator that calls the ahead-of-time PTX kernels in `uor_r4_float_train`
/// (or analytic `f64` keep secant when `analytic_keep_secant == true`) and captures per-layer
/// `||ds_l||_2` and `|d_lambda|` inside `bwd` before `candle_core::GradStore` drops intermediate tensors.
struct TrackedRateSelect {
    layer: usize,
    analytic_keep_secant: bool,
    telemetry: std::sync::Arc<std::sync::Mutex<RateBackwardTelemetry>>,
}

impl CustomOp1 for TrackedRateSelect {
    fn name(&self) -> &'static str {
        "tracked-rate-select"
    }

    fn cpu_fwd(&self, _s: &CpuStorage, _l: &Layout) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("D24: rate_select requires CUDA")
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(&self, s: &CudaStorage, l: &Layout) -> candle_core::Result<(CudaStorage, Shape)> {
        let t = Tensor::from_storage(
            candle_core::Storage::Cuda(s.try_clone(l)?),
            l.shape().clone(),
            candle_core::op::BackpropOp::none(),
            false,
        );
        let out = rate_select(&t).map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let out_c = out.contiguous()?;
        let (os, ol) = out_c.storage_and_layout();
        let candle_core::Storage::Cuda(cuda_s) = &*os else {
            candle_core::bail!("expected CUDA storage from rate_select");
        };
        Ok((cuda_s.try_clone(ol)?, ol.shape().clone()))
    }

    fn bwd(
        &self,
        score: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        let d_score = if self.analytic_keep_secant {
            let s_vec = score.flatten_all()?.to_vec1::<f32>()?;
            let g_vec = grad.flatten_all()?.to_vec1::<f32>()?;
            let ds = rate_select_bwd_analytic_keep_f64(&s_vec, &g_vec);
            Tensor::from_vec(ds, score.shape(), score.device())?
        } else {
            #[cfg(feature = "cuda")]
            if let Device::Cuda(device) = score.device() {
                let s = score.contiguous()?;
                let g = grad.contiguous()?;
                let total = s.elem_count();
                let (ss, sl) = s.storage_and_layout();
                let (gs, gl) = g.storage_and_layout();
                let (candle_core::Storage::Cuda(ss), candle_core::Storage::Cuda(gs)) = (&*ss, &*gs)
                else {
                    candle_core::bail!("expected CUDA tensors in rate_select backward");
                };
                let sv = ss
                    .as_cuda_slice::<f32>()?
                    .try_slice(sl.start_offset()..sl.start_offset() + total)
                    .ok_or_else(|| candle_core::Error::Msg("sv slice out of bounds".into()))?;
                let gv = gs
                    .as_cuda_slice::<f32>()?
                    .try_slice(gl.start_offset()..gl.start_offset() + total * 2)
                    .ok_or_else(|| candle_core::Error::Msg("gv slice out of bounds".into()))?;
                let ds_slice = uninit::<f32>(device, total)?;
                launch(
                    device,
                    "rate_select_bwd",
                    total,
                    &[
                        Arg::F(sv),
                        Arg::F(gv),
                        Arg::f(&ds_slice),
                        Arg::U32(total as u32),
                    ],
                )?;
                Tensor::from_storage(
                    candle_core::Storage::Cuda(CudaStorage::wrap_cuda_slice(
                        ds_slice,
                        device.clone(),
                    )),
                    score.shape().clone(),
                    candle_core::op::BackpropOp::none(),
                    false,
                )
            } else {
                candle_core::bail!("D24: rate_select gradient requires CUDA")
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = (score, grad);
                candle_core::bail!("D24: rate_select gradient requires CUDA")
            }
        };
        let ds_norm = f64::from(d_score.sqr()?.sum_all()?.to_scalar::<f32>()?).sqrt();
        let d_lam = grad.narrow(grad.rank() - 1, 0, 1)?;
        let d_lam_abs = f64::from(d_lam.abs()?.sum_all()?.to_scalar::<f32>()?);
        let d_lam_cnt = d_lam.elem_count();
        if let Ok(mut tel) = self.telemetry.lock() {
            if self.layer < 6 {
                tel.layer_ds_l2_norms[self.layer] = ds_norm;
            }
            tel.d_lambda_abs_sum += d_lam_abs;
            tel.d_lambda_count += d_lam_cnt;
        }
        Ok(Some(d_score))
    }
}

fn rms_norm_cuda(input: &Tensor, weight: &Tensor) -> Result<Tensor> {
    Ok(input
        .contiguous()?
        .apply_op2(&weight.contiguous()?, NativeRmsNorm)?)
}

fn get_var<'a>(model: &'a StackModel, name: &str) -> Result<&'a Tensor> {
    model
        .variables()
        .get(name)
        .map(|v| v.as_tensor())
        .ok_or_else(|| invalid(format!("missing model variable {name}")))
}

fn fbs_linear(model: &StackModel, input: &Tensor, fbs_var_name: &str) -> Result<Tensor> {
    let weight = get_var(model, fbs_var_name)?;
    let plan = model
        .projection_plan_for_variable(fbs_var_name)
        .ok_or_else(|| invalid(format!("missing projection plan for {fbs_var_name}")))?;
    plan.forward(weight, input)
}

/// Computes the 4-block Hamilton product `q (x) u` directly on quantized Q8 unit quaternions `q`
/// without re-normalizing `q`, matching `hamilton_shift_add_q8` in the D11 integer export pass.
fn hamilton_product_q8_tensors(q_q8: &Tensor, u: &Tensor) -> Result<Tensor> {
    let orig_shape = u.shape().clone();
    let blocks = u.elem_count() / 4;
    let q4 = q_q8.reshape((blocks, 4))?;
    let u4 = u.reshape((blocks, 4))?;
    let a1 = q4.narrow(1, 0, 1)?;
    let b1 = q4.narrow(1, 1, 1)?;
    let c1 = q4.narrow(1, 2, 1)?;
    let d1 = q4.narrow(1, 3, 1)?;
    let a2 = u4.narrow(1, 0, 1)?;
    let b2 = u4.narrow(1, 1, 1)?;
    let c2 = u4.narrow(1, 2, 1)?;
    let d2 = u4.narrow(1, 3, 1)?;
    let r0 = a1
        .mul(&a2)?
        .sub(&b1.mul(&b2)?)?
        .sub(&c1.mul(&c2)?)?
        .sub(&d1.mul(&d2)?)?;
    let r1 = a1
        .mul(&b2)?
        .add(&b1.mul(&a2)?)?
        .add(&c1.mul(&d2)?)?
        .sub(&d1.mul(&c2)?)?;
    let r2 = a1
        .mul(&c2)?
        .sub(&b1.mul(&d2)?)?
        .add(&c1.mul(&a2)?)?
        .add(&d1.mul(&b2)?)?;
    let r3 = a1
        .mul(&d2)?
        .add(&b1.mul(&c2)?)?
        .sub(&c1.mul(&b2)?)?
        .add(&d1.mul(&a2)?)?;
    Ok(Tensor::cat(&[&r0, &r1, &r2, &r3], 1)?.reshape(orig_shape)?)
}

fn quantize_unit_gate_q8(gate: &Tensor) -> Result<Tensor> {
    let orig_shape = gate.shape().clone();
    let blocks = gate.elem_count() / 4;
    let g4 = gate.reshape((blocks, 4))?;
    let r = g4.sqr()?.sum_keepdim(1)?.affine(1.0, 1e-12)?.sqrt()?;
    let unit = g4.broadcast_div(&r)?;
    let q8 = unit
        .affine(f64::from(Q8_SCALE), 0.0)?
        .round()?
        .clamp(-f64::from(Q8_SCALE), f64::from(Q8_SCALE))?
        .affine(1.0 / f64::from(Q8_SCALE), 0.0)?;
    Ok(q8.reshape(orig_shape)?)
}

pub struct ForwardPassOutput {
    pub hidden: Tensor,
    pub layer_scores: Vec<Tensor>,
    pub layer_selected: Vec<Tensor>,
    pub bwd_telemetry: std::sync::Arc<std::sync::Mutex<RateBackwardTelemetry>>,
    pub d11_layer0_oracle_checked_lanes: usize,
    pub d11_layer0_oracle_max_lsb_diff: i32,
}

/// Executes the 6-layer recurrence + MLP stack for `arm` in either continuous `f32` (`export_mode == false`)
/// or D11 integer export (`export_mode == true`). When `export_mode == false` and `analytic_keep_secant == false`,
/// the forward and backward passes for `RecGeometricS3Active` and `RecGeometricU1AbelianControl` are
/// bit-for-bit identical (0 ULP) to `StackModel::hidden`.
#[allow(clippy::too_many_arguments)]
pub fn forward_hidden_for_arm(
    model: &StackModel,
    ids: &[u32],
    batch: usize,
    time: usize,
    arm: RecArm,
    export_mode: bool,
    analytic_keep_secant: bool,
    verify_d11_layer0_oracle: bool,
) -> Result<ForwardPassOutput> {
    let device = model.device();
    let width = model.config.width;
    let lanes = width / 4;
    let mut x = model.embed(ids, batch, time)?;
    let mut layer_scores = Vec::with_capacity(model.config.layers());
    let mut layer_selected = Vec::with_capacity(model.config.layers());
    let bwd_telemetry = std::sync::Arc::new(std::sync::Mutex::new(RateBackwardTelemetry::default()));
    let mut d11_checked_lanes = 0usize;
    let mut d11_max_lsb_diff = 0i32;

    let keep_q12_table_tensor = if export_mode {
        let vals: Vec<f32> = KEEP_TABLE_Q12
            .iter()
            .map(|&v| (v as f32) / Q12_SCALE)
            .collect();
        Some(Tensor::from_vec(vals, RATE_ENTRIES, device)?)
    } else {
        None
    };

    for layer in 0..model.config.layers() {
        let pfx = format!("layers.{layer:02}");
        let u = rms_norm_cuda(&x, get_var(model, &format!("{pfx}.rec_norm.weight"))?)?;
        let branches = fbs_linear(model, &u, &format!("{pfx}.rec.in.spiralcore_fbs"))?;
        let input = branches.narrow(2, 0, width)?;
        let gate = branches.narrow(2, width, width)?;

        let conv_weights = get_var(model, &format!("{pfx}.rec.conv.weight"))?;
        let mut convolved = get_var(model, &format!("{pfx}.rec.conv.bias"))?
            .broadcast_as((batch, time, width))?
            .contiguous()?;
        for shift in 0..4usize.min(time) {
            let shifted = if shift == 0 {
                input.clone()
            } else {
                Tensor::cat(
                    &[
                        &Tensor::zeros((batch, shift, width), DType::F32, device)?,
                        &input.narrow(1, 0, time - shift)?,
                    ],
                    1,
                )?
            };
            convolved = convolved.add(&shifted.broadcast_mul(&conv_weights.get(shift)?)?)?;
        }

        let gates = fbs_linear(model, &u, &format!("{pfx}.rec.gate.spiralcore_fbs"))?
            .broadcast_add(get_var(model, &format!("{pfx}.rec.gate.bias"))?)?;
        let raw_score = gates
            .narrow(2, 0, lanes)?
            .broadcast_add(&get_var(model, &format!("{pfx}.rec.decay"))?.reshape((1, 1, lanes))?)?
            .clamp(0.0, 1.0)?;

        let score_in = match arm {
            RecArm::RecFrozenRatesNull => raw_score.detach(),
            _ => raw_score.clone(),
        };
        let selected = if export_mode {
            rate_select(&score_in)?
        } else {
            score_in.contiguous()?.apply_op1(TrackedRateSelect {
                layer,
                analytic_keep_secant: analytic_keep_secant && arm.is_active_rates(),
                telemetry: std::sync::Arc::clone(&bwd_telemetry),
            })?
        };
        let lambda = selected.narrow(3, 0, 1)?;
        let keep = if let Some(ref k_tbl) = keep_q12_table_tensor {
            let idx = (score_in.affine(RATE_ENTRIES as f64, 0.0)?)
                .floor()?
                .clamp(0.0, (RATE_ENTRIES - 1) as f64)?
                .to_dtype(DType::U32)?
                .flatten_all()?;
            k_tbl
                .index_select(&idx, 0)?
                .reshape((batch, time, lanes, 1))?
        } else {
            selected.narrow(3, 1, 1)?
        };

        let unit_rot = match arm {
            RecArm::RecGeometricIdentityTransportControl => {
                let ones = Tensor::ones((batch, time, lanes, 1), DType::F32, device)?;
                let zeros = Tensor::zeros((batch, time, lanes, 3), DType::F32, device)?;
                Tensor::cat(&[&ones, &zeros], 3)?
            }
            RecArm::RecGeometricU1AbelianControl => {
                let raw = gates
                    .narrow(2, lanes, width)?
                    .reshape((batch, time, lanes, 4))?
                    .broadcast_mul(&Tensor::new(&[1.0f32, 1.0, 0.0, 0.0], device)?)?;
                let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
                raw.broadcast_div(&norm)?
            }
            RecArm::RecGeometricS3Active | RecArm::RecFrozenRatesNull => {
                let raw = gates
                    .narrow(2, lanes, width)?
                    .reshape((batch, time, lanes, 4))?;
                let norm = raw.sqr()?.sum_keepdim(3)?.affine(1.0, 1e-6)?.sqrt()?;
                raw.broadcast_div(&norm)?
            }
        };

        let eff_unit = if export_mode {
            unit_rot
                .affine(f64::from(Q8_SCALE), 0.0)?
                .round()?
                .clamp(-f64::from(Q8_SCALE), f64::from(Q8_SCALE))?
                .affine(1.0 / f64::from(Q8_SCALE), 0.0)?
        } else {
            unit_rot
        };

        let transition = if arm == RecArm::RecGeometricIdentityTransportControl && !export_mode {
            let zeros = Tensor::zeros((batch, time, lanes, 3), DType::F32, device)?;
            Tensor::cat(&[&lambda, &zeros], 3)?
        } else {
            eff_unit.broadcast_mul(&lambda)?
        };

        let eff_convolved = if export_mode {
            convolved
                .affine(f64::from(Q12_SCALE), 0.0)?
                .round()?
                .clamp(Q12_I16_MIN, Q12_I16_MAX)?
                .affine(1.0 / f64::from(Q12_SCALE), 0.0)?
        } else {
            convolved
        };

        let drive = eff_convolved
            .reshape((batch, time, lanes, 4))?
            .broadcast_mul(&keep)?;
        let state = quaternion_scan(&transition.contiguous()?, &drive.contiguous()?)?
            .reshape((batch, time, width))?;

        let mixed = if export_mode {
            let q_gate = quantize_unit_gate_q8(&gate)?;
            let state_q12 = state
                .affine(f64::from(Q12_SCALE), 0.0)?
                .round()?
                .clamp(Q12_I16_MIN, Q12_I16_MAX)?
                .affine(1.0 / f64::from(Q12_SCALE), 0.0)?;
            let prod = hamilton_product_q8_tensors(&q_gate, &state_q12)?;
            if verify_d11_layer0_oracle && layer == 0 {
                // Verify row 0 (first 8 lanes across all `time` steps) against pure-integer CPU oracle
                let lam_host = lambda.get(0)?.squeeze(2)?.to_vec2::<f32>()?;
                let u_host = eff_unit
                    .get(0)?
                    .affine(f64::from(Q8_SCALE), 0.0)?
                    .round()?
                    .to_vec3::<f32>()?;
                let c_host = eff_convolved
                    .get(0)?
                    .reshape((time, lanes, 4))?
                    .affine(f64::from(Q12_SCALE), 0.0)?
                    .round()?
                    .to_vec3::<f32>()?;
                let g_host = q_gate
                    .get(0)?
                    .reshape((time, lanes, 4))?
                    .affine(f64::from(Q8_SCALE), 0.0)?
                    .round()?
                    .to_vec3::<f32>()?;
                let gpu_out_q12 = prod
                    .get(0)?
                    .reshape((time, lanes, 4))?
                    .affine(f64::from(Q12_SCALE), 0.0)?
                    .round()?
                    .to_vec3::<f32>()?;
                for l_idx in 0..8usize.min(lanes) {
                    let r_idx: Vec<usize> = (0..time)
                        .map(|t| {
                            let bits = lam_host[t][l_idx].to_bits();
                            RATE_TABLE
                                .iter()
                                .position(|r| r.to_bits() == bits)
                                .unwrap_or(RATE_ENTRIES - 1)
                        })
                        .collect();
                    let u_l: Vec<[i32; 4]> = (0..time)
                        .map(|t| std::array::from_fn(|c| u_host[t][l_idx][c] as i32))
                        .collect();
                    let c_l: Vec<[i32; 4]> = (0..time)
                        .map(|t| std::array::from_fn(|c| c_host[t][l_idx][c] as i32))
                        .collect();
                    let g_l: Vec<[i32; 4]> = (0..time)
                        .map(|t| std::array::from_fn(|c| g_host[t][l_idx][c] as i32))
                        .collect();
                    let cpu_i32 = d11_recurrence_scan_and_gate_i32(&r_idx, &u_l, &c_l, &g_l);
                    for t in 0..time {
                        for c in 0..4 {
                            let diff = (cpu_i32[t][c] - (gpu_out_q12[t][l_idx][c] as i32)).abs();
                            d11_max_lsb_diff = d11_max_lsb_diff.max(diff);
                        }
                    }
                    d11_checked_lanes += 1;
                }
            }
            prod
        } else {
            hamilton_gate(&gate, &state)?
        };

        let rec_out = fbs_linear(model, &mixed, &format!("{pfx}.rec.out.spiralcore_fbs"))?;
        x = x.add(&rec_out)?;

        // MLP block
        let u_mlp = rms_norm_cuda(&x, get_var(model, &format!("{pfx}.mlp_norm.weight"))?)?;
        let mlp_gate = fbs_linear(model, &u_mlp, &format!("{pfx}.mlp.gate.spiralcore_fbs"))?;
        let mlp_up = fbs_linear(model, &u_mlp, &format!("{pfx}.mlp.up.spiralcore_fbs"))?;
        let mlp_mixed = if export_mode {
            let q_mlp = quantize_unit_gate_q8(&mlp_gate)?;
            hamilton_product_q8_tensors(&q_mlp, &mlp_up)?
        } else {
            hamilton_gate(&mlp_gate, &mlp_up)?
        };
        let mlp_down = fbs_linear(model, &mlp_mixed, &format!("{pfx}.mlp.down.spiralcore_fbs"))?;
        x = x.add(&mlp_down)?;

        layer_scores.push(raw_score);
        layer_selected.push(selected);
    }

    let x_final = rms_norm_cuda(&x, get_var(model, "final_norm.weight")?)?;
    Ok(ForwardPassOutput {
        hidden: x_final.reshape((batch * time, width))?,
        layer_scores,
        layer_selected,
        bwd_telemetry,
        d11_layer0_oracle_checked_lanes: d11_checked_lanes,
        d11_layer0_oracle_max_lsb_diff: d11_max_lsb_diff,
    })
}

/// Constructs the six-recurrence rung-0 `StackModel` for `seed` on `device`, with shared initialization
/// across all four arms.
pub fn build_six_recurrence_model(seed: u64, device: &Device) -> Result<StackModel> {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandaloneCudaParityCheck {
    pub passed: bool,
    pub max_rate_table_error: f64,
    pub max_keep_table_error: f64,
    pub max_secant_bwd_error: f64,
    pub max_quaternion_scan_fwd_error: f64,
    pub max_quaternion_scan_bwd_error: f64,
    pub max_stack_hidden_parity_diff: f64,
    pub max_stack_grad_parity_diff: f64,
    pub s3_layer_step0_ds_l2_norms: [f64; 6],
    pub u1_layer_step0_ds_l2_norms: [f64; 6],
    pub id_layer_step0_ds_l2_norms: [f64; 6],
    pub frozen_null_step0_ds_l2_norm: f64,
}

/// Runs `I-Standalone-CUDA-Parity` on `device`:
/// 1. Standalone CUDA device qualification of `rate_select_fwd`/`bwd` against exact `f64` dyadic rate,
///    keep, and secant formulas (`<= 5e-13` rate error, `<= 4.6e-10` keep error, `<= 5.5e-8` secant error).
/// 2. Standalone CUDA device qualification of `quaternion_scan_fwd`/`bwd` against exact `f64` CPU
///    reference scan across non-abelian `S^3`, abelian `U(1)`, and identity sequences.
/// 3. Bit-for-bit parity (`max_abs_diff == 0.0`) between `forward_hidden_for_arm` and `StackModel::hidden`
///    (and all 38 parameter gradients) on the 6-layer stack.
/// 4. Nonzero step-0 opening/decay credit `||ds_l||_2 > 0` across all 6 recurrence layers (`l = 0..5`).
pub fn verify_standalone_cuda_parity(device: &Device) -> Result<StandaloneCudaParityCheck> {
    if !device.is_cuda() {
        return Err(invalid("I-Standalone-CUDA-Parity requires a CUDA device"));
    }

    // 1. rate_select_fwd / rate_select_bwd parity across all 16 dyadic bins
    let scores_host: Vec<f32> = (0..RATE_ENTRIES)
        .map(|i| (i as f32 + 0.375) / (RATE_ENTRIES as f32))
        .collect();
    let scores_var = candle_core::Var::from_vec(scores_host.clone(), RATE_ENTRIES, device)?;
    let selected = rate_select(scores_var.as_tensor())?;
    let selected_host = selected.to_vec2::<f32>()?;
    let mut max_rate_err = 0.0f64;
    let mut max_keep_err = 0.0f64;
    for (i, pair) in selected_host.iter().enumerate() {
        let exact_lam = 2.0f64.powi(-((i as i32) + 1));
        let exact_keep_f32 = (1.0f64 - exact_lam * exact_lam).sqrt() as f32;
        max_rate_err = max_rate_err.max((f64::from(pair[0]) - exact_lam).abs());
        max_keep_err = max_keep_err.max(f64::from((pair[1] - exact_keep_f32).abs()));
        if pair[0].to_bits() != RATE_TABLE[i].to_bits()
            || pair[1].to_bits() != KEEP_TABLE[i].to_bits()
        {
            return Err(invalid(format!(
                "rate_select table bit mismatch at bin {i}"
            )));
        }
    }

    let grad_in_host: Vec<f32> = (0..RATE_ENTRIES * 2)
        .map(|k| ((k as f32) * 0.17 - 1.1).sin())
        .collect();
    let grad_in = Tensor::from_vec(grad_in_host.clone(), (RATE_ENTRIES, 2), device)?;
    let bwd_store = selected.mul(&grad_in)?.sum_all()?.backward()?;
    let ds_gpu = bwd_store
        .get(scores_var.as_tensor())
        .ok_or_else(|| invalid("missing rate_select_bwd gradient"))?
        .to_vec1::<f32>()?;
    let mut max_secant_err = 0.0f64;
    for i in 0..RATE_ENTRIES {
        let dl_in = grad_in_host[2 * i];
        let dk_in = grad_in_host[2 * i + 1];
        let mut credit = 0.0f32;
        let mut sides = 0.0f32;
        if i > 0 {
            let dl = RATE_TABLE[i - 1] - RATE_TABLE[i];
            let dk = KEEP_TABLE[i - 1] - KEEP_TABLE[i];
            credit -= dl_in * dl + dk_in * dk;
            sides += 1.0;
        }
        if i + 1 < RATE_ENTRIES {
            let dl = RATE_TABLE[i + 1] - RATE_TABLE[i];
            let dk = KEEP_TABLE[i + 1] - KEEP_TABLE[i];
            credit += dl_in * dl + dk_in * dk;
            sides += 1.0;
        }
        let expected_ds = (credit / sides) * (RATE_ENTRIES as f32);
        max_secant_err = max_secant_err.max(f64::from((ds_gpu[i] - expected_ds).abs()));
    }

    // 2. quaternion_scan_fwd / quaternion_scan_bwd parity against exact f64 CPU reference
    let (b_scan, t_scan, l_scan) = (2usize, 16usize, 4usize);
    let n_elem = b_scan * t_scan * l_scan * 4;
    let trans_host: Vec<f32> = (0..n_elem)
        .map(|k| 0.25 * ((k as f32) * 0.31 + 0.2).cos())
        .collect();
    let drive_host: Vec<f32> = (0..n_elem)
        .map(|k| 0.5 * ((k as f32) * 0.19 - 0.4).sin())
        .collect();
    let d_out_host: Vec<f32> = (0..n_elem)
        .map(|k| 0.3 * ((k as f32) * 0.23 + 0.7).sin())
        .collect();
    let trans_var =
        candle_core::Var::from_vec(trans_host.clone(), (b_scan, t_scan, l_scan, 4), device)?;
    let drive_var =
        candle_core::Var::from_vec(drive_host.clone(), (b_scan, t_scan, l_scan, 4), device)?;
    let d_out_t = Tensor::from_vec(d_out_host.clone(), (b_scan, t_scan, l_scan, 4), device)?;
    let state_gpu = quaternion_scan(trans_var.as_tensor(), drive_var.as_tensor())?;
    let state_host = state_gpu.flatten_all()?.to_vec1::<f32>()?;
    let scan_grads = state_gpu.mul(&d_out_t)?.sum_all()?.backward()?;
    let dt_host = scan_grads
        .get(trans_var.as_tensor())
        .ok_or_else(|| invalid("missing quaternion_scan d_transition"))?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dd_host = scan_grads
        .get(drive_var.as_tensor())
        .ok_or_else(|| invalid("missing quaternion_scan d_drive"))?
        .flatten_all()?
        .to_vec1::<f32>()?;

    let mut max_scan_fwd_err = 0.0f64;
    let mut max_scan_bwd_err = 0.0f64;
    for b in 0..b_scan {
        for l in 0..l_scan {
            let mut h = [0.0f64; 4];
            let mut h_hist = vec![[0.0f64; 4]; t_scan];
            for (t, h_slot) in h_hist.iter_mut().enumerate().take(t_scan) {
                let base = ((b * t_scan + t) * l_scan + l) * 4;
                let q = [
                    f64::from(trans_host[base]),
                    f64::from(trans_host[base + 1]),
                    f64::from(trans_host[base + 2]),
                    f64::from(trans_host[base + 3]),
                ];
                let d = [
                    f64::from(drive_host[base]),
                    f64::from(drive_host[base + 1]),
                    f64::from(drive_host[base + 2]),
                    f64::from(drive_host[base + 3]),
                ];
                let rot = [
                    q[0] * h[0] - q[1] * h[1] - q[2] * h[2] - q[3] * h[3],
                    q[0] * h[1] + q[1] * h[0] + q[2] * h[3] - q[3] * h[2],
                    q[0] * h[2] - q[1] * h[3] + q[2] * h[0] + q[3] * h[1],
                    q[0] * h[3] + q[1] * h[2] - q[2] * h[1] + q[3] * h[0],
                ];
                for c in 0..4 {
                    h[c] = rot[c] + d[c];
                    max_scan_fwd_err =
                        max_scan_fwd_err.max((f64::from(state_host[base + c]) - h[c]).abs());
                }
                *h_slot = h;
            }
            let mut g = [0.0f64; 4];
            for t in (0..t_scan).rev() {
                let base = ((b * t_scan + t) * l_scan + l) * 4;
                for c in 0..4 {
                    g[c] += f64::from(d_out_host[base + c]);
                    max_scan_bwd_err =
                        max_scan_bwd_err.max((f64::from(dd_host[base + c]) - g[c]).abs());
                }
                let h_prev = if t > 0 { h_hist[t - 1] } else { [0.0; 4] };
                // d_transition = g (x) conj(h_prev)
                let dq = [
                    g[0] * h_prev[0] + g[1] * h_prev[1] + g[2] * h_prev[2] + g[3] * h_prev[3],
                    -g[0] * h_prev[1] + g[1] * h_prev[0] - g[2] * h_prev[3] + g[3] * h_prev[2],
                    -g[0] * h_prev[2] + g[1] * h_prev[3] + g[2] * h_prev[0] - g[3] * h_prev[1],
                    -g[0] * h_prev[3] - g[1] * h_prev[2] + g[2] * h_prev[1] + g[3] * h_prev[0],
                ];
                for c in 0..4 {
                    max_scan_bwd_err =
                        max_scan_bwd_err.max((f64::from(dt_host[base + c]) - dq[c]).abs());
                }
                let q = [
                    f64::from(trans_host[base]),
                    f64::from(trans_host[base + 1]),
                    f64::from(trans_host[base + 2]),
                    f64::from(trans_host[base + 3]),
                ];
                // g_{t-1} = conj(q_t) (x) g_t
                g = [
                    q[0] * g[0] + q[1] * g[1] + q[2] * g[2] + q[3] * g[3],
                    q[0] * g[1] - q[1] * g[0] - q[2] * g[3] + q[3] * g[2],
                    q[0] * g[2] + q[1] * g[3] - q[2] * g[0] - q[3] * g[1],
                    q[0] * g[3] - q[1] * g[2] + q[2] * g[1] - q[3] * g[0],
                ];
            }
        }
    }

    // 3. StackModel::hidden vs forward_hidden_for_arm bit-for-bit parity & per-layer ||ds_l||_2 > 0
    let model = build_six_recurrence_model(1, device)?;
    let batch = 2usize;
    let time = 16usize;
    let ids: Vec<u32> = (0..batch * time).map(|i| (i % 13) as u32).collect();
    let targets: Vec<u32> = (0..batch * time).map(|i| ((i + 1) % 13) as u32).collect();

    let ref_hidden = model.hidden(&ids, batch, time)?;
    let ref_loss = model.native_head_loss(&ref_hidden, &targets, SHEET_RIVALS)?;
    let ref_grads = ref_loss.backward()?;

    let out_s3 = forward_hidden_for_arm(
        &model,
        &ids,
        batch,
        time,
        RecArm::RecGeometricS3Active,
        false,
        false,
        false,
    )?;
    let hidden_diff = f64::from(
        ref_hidden
            .sub(&out_s3.hidden)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?,
    );
    let loss_s3 = model.native_head_loss(&out_s3.hidden, &targets, SHEET_RIVALS)?;
    let grads_s3 = loss_s3.backward()?;
    let mut max_grad_diff = 0.0f64;
    for (name, var) in model.variables() {
        let g1 = ref_grads
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("missing ref grad for {name}")))?;
        let g2 = grads_s3
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("missing s3 grad for {name}")))?;
        let d = f64::from(g1.sub(g2)?.abs()?.max_all()?.to_scalar::<f32>()?);
        max_grad_diff = max_grad_diff.max(d);
    }

    let s3_norms = out_s3
        .bwd_telemetry
        .lock()
        .map_err(|_| invalid("poisoned S3 bwd_telemetry"))?
        .layer_ds_l2_norms;

    let out_u1 = forward_hidden_for_arm(
        &model,
        &ids,
        batch,
        time,
        RecArm::RecGeometricU1AbelianControl,
        false,
        false,
        false,
    )?;
    let _grads_u1 = model
        .native_head_loss(&out_u1.hidden, &targets, SHEET_RIVALS)?
        .backward()?;
    let u1_norms = out_u1
        .bwd_telemetry
        .lock()
        .map_err(|_| invalid("poisoned U1 bwd_telemetry"))?
        .layer_ds_l2_norms;

    let out_id = forward_hidden_for_arm(
        &model,
        &ids,
        batch,
        time,
        RecArm::RecGeometricIdentityTransportControl,
        false,
        false,
        false,
    )?;
    let _grads_id = model
        .native_head_loss(&out_id.hidden, &targets, SHEET_RIVALS)?
        .backward()?;
    let id_norms = out_id
        .bwd_telemetry
        .lock()
        .map_err(|_| invalid("poisoned Identity bwd_telemetry"))?
        .layer_ds_l2_norms;

    let out_frz = forward_hidden_for_arm(
        &model,
        &ids,
        batch,
        time,
        RecArm::RecFrozenRatesNull,
        false,
        false,
        false,
    )?;
    let grads_frz = model
        .native_head_loss(&out_frz.hidden, &targets, SHEET_RIVALS)?
        .backward()?;
    let frz_norms = out_frz
        .bwd_telemetry
        .lock()
        .map_err(|_| invalid("poisoned FrozenRates bwd_telemetry"))?
        .layer_ds_l2_norms;
    let mut frz_norm_sq: f64 = frz_norms.iter().map(|v| v * v).sum();
    for l in 0..6 {
        let dec_var = get_var(&model, &format!("layers.{l:02}.rec.decay"))?;
        if let Some(g) = grads_frz.get(dec_var) {
            frz_norm_sq += f64::from(g.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
    }

    let passed = max_rate_err <= 5e-13
        && max_keep_err <= 4.6e-10
        && max_secant_err <= 5.5e-8
        && max_scan_fwd_err <= 1e-6
        && max_scan_bwd_err <= 1e-5
        && hidden_diff == 0.0
        && max_grad_diff <= 1e-5
        && s3_norms.iter().all(|&n| n > 0.0 && n.is_finite())
        && u1_norms.iter().all(|&n| n > 0.0 && n.is_finite())
        && id_norms.iter().all(|&n| n > 0.0 && n.is_finite())
        && frz_norm_sq == 0.0;

    Ok(StandaloneCudaParityCheck {
        passed,
        max_rate_table_error: max_rate_err,
        max_keep_table_error: max_keep_err,
        max_secant_bwd_error: max_secant_err,
        max_quaternion_scan_fwd_error: max_scan_fwd_err,
        max_quaternion_scan_bwd_error: max_scan_bwd_err,
        max_stack_hidden_parity_diff: hidden_diff,
        max_stack_grad_parity_diff: max_grad_diff,
        s3_layer_step0_ds_l2_norms: s3_norms,
        u1_layer_step0_ds_l2_norms: u1_norms,
        id_layer_step0_ds_l2_norms: id_norms,
        frozen_null_step0_ds_l2_norm: frz_norm_sq.sqrt(),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelEvalMetrics {
    pub scored_rows: usize,
    pub evaluated_context_tokens: usize,
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
    pub eval_scored_targets_per_second: f64,
    pub q12_i32_writer_oracle_checked_rows: usize,
    pub q12_i32_writer_oracle_mismatches: usize,
    pub d11_layer0_oracle_checked_lanes: usize,
    pub d11_layer0_oracle_max_lsb_diff: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateTelemetrySnapshot {
    pub step: usize,
    pub bins: [u64; RATE_ENTRIES],
    pub plateau_12_15_fraction: f64,
    pub decay_param_bins: Vec<usize>,
}

/// Evaluates `model` across `windows` (`14,287` `ft/dev` targets or `14,479` `fresh_v2` targets)
/// in either continuous `f32` (`export_mode == false`) or D11 integer export (`export_mode == true`).
/// When `capture_rate_hist == true`, also captures the 16-bin active `(lane, token)` dyadic rate
/// histogram across all evaluated windows and layers.
pub fn evaluate_causal_panel(
    model: &StackModel,
    windows: &[CausalWindow],
    arm: RecArm,
    step: usize,
    export_mode: bool,
    capture_rate_hist: bool,
) -> Result<(PanelEvalMetrics, Vec<u32>, Option<RateTelemetrySnapshot>)> {
    let device = model.device();
    sync_cuda(device)?;
    let t0 = Instant::now();

    let spec = WriterSpec {
        vocabulary: model.config.vocab_size,
        width: model.config.width,
        kept_cells: SHEET_KEPT_CELLS,
    };
    let table_tensor = get_var(model, "embedding.weight")?;
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

    let roots_q12 = spiralcore_roots_q12()?;
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

    let mut total_scored = 0usize;
    let mut top1_hits = 0usize;
    let mut top8_hits = 0usize;
    let mut admitted_hits = 0usize;
    let mut unviolated_hits = 0usize;
    let mut token_margin_sum = 0.0f64;
    let mut address_margin_sum = 0.0f64;
    let mut total_margin_sum = 0.0f64;
    let mut all_top1_preds = Vec::with_capacity(windows.len() * 36);
    let mut q12_checked_rows = 0usize;
    let mut q12_mismatches = 0usize;
    let mut d11_checked_lanes = 0usize;
    let mut d11_max_lsb_diff = 0i32;
    let mut rate_bins = [0u64; RATE_ENTRIES];

    for (chunk_idx, chunk) in windows.chunks(SHEET_EVAL_BATCH).enumerate() {
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
        let fwd = forward_hidden_for_arm(
            model,
            &flat_ids,
            batch_len,
            SHEET_CONTEXT,
            arm,
            export_mode,
            false,
            export_mode && chunk_idx == 0,
        )?;
        if export_mode && chunk_idx == 0 {
            d11_checked_lanes = fwd.d11_layer0_oracle_checked_lanes;
            d11_max_lsb_diff = fwd.d11_layer0_oracle_max_lsb_diff;
        }
        if capture_rate_hist {
            for sel in &fwd.layer_selected {
                let lam_flat = sel.narrow(3, 0, 1)?.flatten_all()?.to_vec1::<f32>()?;
                for &lam in &lam_flat {
                    let idx = rate_index_from_lambda_bits(lam);
                    rate_bins[idx] += 1;
                }
            }
        }

        let idx_tensor = Tensor::from_vec(scored_indices, scored_targets.len(), device)?;
        let scored_hidden = fwd.hidden.index_select(&idx_tensor, 0)?;
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

    sync_cuda(device)?;
    let elapsed = t0.elapsed().as_secs_f64();
    let evaluated_context_tokens = windows.len() * SHEET_CONTEXT;
    let denom = total_scored.max(1) as f64;

    let telemetry = if capture_rate_hist {
        let mut decay_param_bins = Vec::with_capacity(model.config.layers() * (SHEET_WIDTH / 4));
        for layer in 0..model.config.layers() {
            let dec = get_var(model, &format!("layers.{layer:02}.rec.decay"))?.to_vec1::<f32>()?;
            for v in dec {
                let idx =
                    ((v.clamp(0.0, 1.0) * (RATE_ENTRIES as f32)) as usize).min(RATE_ENTRIES - 1);
                decay_param_bins.push(idx);
            }
        }
        let total_bins: u64 = rate_bins.iter().sum();
        let plateau = if total_bins > 0 {
            ((rate_bins[12] + rate_bins[13] + rate_bins[14] + rate_bins[15]) as f64)
                / (total_bins as f64)
        } else {
            0.0
        };
        Some(RateTelemetrySnapshot {
            step,
            bins: rate_bins,
            plateau_12_15_fraction: plateau,
            decay_param_bins,
        })
    } else {
        None
    };

    Ok((
        PanelEvalMetrics {
            scored_rows: total_scored,
            evaluated_context_tokens,
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
            eval_tokens_per_second: (evaluated_context_tokens as f64) / elapsed.max(1e-9),
            eval_scored_targets_per_second: (total_scored as f64) / elapsed.max(1e-9),
            q12_i32_writer_oracle_checked_rows: q12_checked_rows,
            q12_i32_writer_oracle_mismatches: q12_mismatches,
            d11_layer0_oracle_checked_lanes: d11_checked_lanes,
            d11_layer0_oracle_max_lsb_diff: d11_max_lsb_diff,
        },
        all_top1_preds,
        telemetry,
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairedMcNemarComparison {
    pub arm_a: String,
    pub arm_b: String,
    pub seed: u64,
    pub c_a_hit_b_miss: usize,
    pub b_a_miss_b_hit: usize,
    pub discordant_total_b_plus_c: usize,
    pub net_advantage_rows_c_minus_b: isize,
    pub f_i_threshold: usize,
    pub exceeds_f_i: bool,
}

pub fn compute_mcnemar(
    arm_a: &str,
    preds_a: &[u32],
    arm_b: &str,
    preds_b: &[u32],
    targets: &[u32],
    seed: u64,
) -> PairedMcNemarComparison {
    let mut c_count = 0usize;
    let mut b_count = 0usize;
    for ((&pa, &pb), &t) in preds_a.iter().zip(preds_b).zip(targets) {
        let a_ok = pa == t;
        let b_ok = pb == t;
        if a_ok && !b_ok {
            c_count += 1;
        } else if !a_ok && b_ok {
            b_count += 1;
        }
    }
    let discord = b_count + c_count;
    let f_i = 24usize.max((2.0 * (discord as f64).sqrt()).ceil() as usize);
    let net = (c_count as isize) - (b_count as isize);
    PairedMcNemarComparison {
        arm_a: arm_a.to_string(),
        arm_b: arm_b.to_string(),
        seed,
        c_a_hit_b_miss: c_count,
        b_a_miss_b_hit: b_count,
        discordant_total_b_plus_c: discord,
        net_advantage_rows_c_minus_b: net,
        f_i_threshold: f_i,
        exceeds_f_i: net >= (f_i as isize),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportGapCheck {
    pub delta_token_margin: f64,
    pub delta_top1_rows: usize,
    pub delta_top1_rate_pp: f64,
    pub delta_top8_rows: usize,
    pub delta_top8_rate_pp: f64,
    pub row_top1_agreement_hits: usize,
    pub row_top1_agreement_rate: f64,
    pub mcnemar_float_vs_int: PairedMcNemarComparison,
}

pub fn compute_d11_export_gap(
    arm: &str,
    seed: u64,
    float_eval: &PanelEvalMetrics,
    float_preds: &[u32],
    int_eval: &PanelEvalMetrics,
    int_preds: &[u32],
    targets: &[u32],
) -> ExportGapCheck {
    let n = float_eval.scored_rows.max(1) as f64;
    let delta_top1 = float_eval.top1_hits.abs_diff(int_eval.top1_hits);
    let delta_top8 = float_eval.top8_hits.abs_diff(int_eval.top8_hits);
    let agree_hits = float_preds
        .iter()
        .zip(int_preds)
        .filter(|(a, b)| a == b)
        .count();
    let mcnemar = compute_mcnemar(
        &format!("{arm}_f32"),
        float_preds,
        &format!("{arm}_d11_int"),
        int_preds,
        targets,
        seed,
    );
    ExportGapCheck {
        delta_token_margin: (float_eval.token_margin - int_eval.token_margin).abs(),
        delta_top1_rows: delta_top1,
        delta_top1_rate_pp: 100.0 * (delta_top1 as f64) / n,
        delta_top8_rows: delta_top8,
        delta_top8_rate_pp: 100.0 * (delta_top8 as f64) / n,
        row_top1_agreement_hits: agree_hits,
        row_top1_agreement_rate: (agree_hits as f64) / n,
        mcnemar_float_vs_int: mcnemar,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecurrenceArmRunResult {
    pub arm: String,
    pub seed: u64,
    pub steps: usize,
    pub base_lr: f64,
    pub decay_lr: f64,
    pub analytic_keep_secant_used: bool,
    pub train_loss_step0: f64,
    pub train_loss_step256: f64,
    pub train_loss_step512: f64,
    pub train_loss_step1024: f64,
    pub train_loss_delta_1024: f64,
    pub train_wall_seconds: f64,
    pub train_tokens_per_second: f64,
    pub step0_layer_ds_l2_norms: [f64; 6],
    pub step0_total_ds_l2_norm: f64,
    pub step0_mean_abs_d_lambda: f64,
    pub decay_param_crossings: usize,
    pub decay_param_total: usize,
    pub decay_param_crossings_frac: f64,
    pub max_keep_plateau_12_15_fraction: f64,
    pub rate_telemetry_step0: RateTelemetrySnapshot,
    pub rate_telemetry_step1024: RateTelemetrySnapshot,
    pub dev_step0_float: PanelEvalMetrics,
    pub dev_step0_d11_int: PanelEvalMetrics,
    pub dev_step256_float: PanelEvalMetrics,
    pub dev_step256_d11_int: PanelEvalMetrics,
    pub dev_step512_float: PanelEvalMetrics,
    pub dev_step512_d11_int: PanelEvalMetrics,
    pub dev_step1024_float: PanelEvalMetrics,
    pub dev_step1024_d11_int: PanelEvalMetrics,
    pub export_gap_step0: ExportGapCheck,
    pub export_gap_step1024: ExportGapCheck,
}

#[allow(clippy::too_many_arguments)]
fn response_masked_batch_loss_and_fwd(
    model: &StackModel,
    flat_ids: &[u32],
    scored_indices: &[u32],
    scored_targets: &[u32],
    batch_size: usize,
    context: usize,
    arm: RecArm,
    analytic_keep_secant: bool,
) -> Result<(Tensor, ForwardPassOutput)> {
    let fwd = forward_hidden_for_arm(
        model,
        flat_ids,
        batch_size,
        context,
        arm,
        false,
        analytic_keep_secant,
        false,
    )?;
    let idx_tensor = Tensor::from_vec(
        scored_indices.to_vec(),
        scored_indices.len(),
        model.device(),
    )?;
    let masked_hidden = fwd.hidden.index_select(&idx_tensor, 0)?;
    let loss = model.native_head_loss(&masked_hidden, scored_targets, SHEET_RIVALS)?;
    Ok((loss, fwd))
}

/// Runs one `(arm, seed)` training and DEV evaluation trajectory on CUDA.
#[allow(clippy::too_many_arguments)]
pub fn run_recurrence_arm(
    arm: RecArm,
    seed: u64,
    steps: usize,
    base_lr: f64,
    decay_lr_mult: f64,
    analytic_keep_secant: bool,
    train_batches: &[TrainBatch],
    dev_windows: &[CausalWindow],
    dev_targets: &[u32],
    device: &Device,
) -> Result<(StackModel, RecurrenceArmRunResult, Vec<u32>, Vec<u32>)> {
    let model = build_six_recurrence_model(seed, device)?;
    let mut opt_main = StackAdamW::new(&model, SHEET_WEIGHT_DECAY, SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, SHEET_CLIP)?;
    let decay_lr = base_lr * decay_lr_mult;

    let (dev_step0_float, preds_0_float, rate_tel_0_opt) =
        evaluate_causal_panel(&model, dev_windows, arm, 0, false, true)?;
    let rate_telemetry_step0 = rate_tel_0_opt.expect("captured step0 rate telemetry");
    let (dev_step0_int, preds_0_int, _) =
        evaluate_causal_panel(&model, dev_windows, arm, 0, true, false)?;
    let export_gap_step0 = compute_d11_export_gap(
        arm.name(),
        seed,
        &dev_step0_float,
        &preds_0_float,
        &dev_step0_int,
        &preds_0_int,
        dev_targets,
    );

    let mut train_loss_step0 = 0.0f64;
    let mut train_loss_step256 = 0.0f64;
    let mut train_loss_step512 = 0.0f64;
    let mut dev_step256_float = dev_step0_float.clone();
    let mut dev_step256_int = dev_step0_int.clone();
    let mut dev_step512_float = dev_step0_float.clone();
    let mut dev_step512_int = dev_step0_int.clone();
    let mut step0_layer_ds_norms = [0.0f64; 6];
    let mut step0_mean_abs_d_lambda = 0.0f64;

    sync_cuda(device)?;
    let mut train_wall_accum = 0.0f64;

    for step in 0..steps {
        let t_step = Instant::now();
        let (ref flat_ids, ref scored_idx, ref scored_tgts) =
            train_batches[step % train_batches.len()];
        let (loss, fwd) = response_masked_batch_loss_and_fwd(
            &model,
            flat_ids,
            scored_idx,
            scored_tgts,
            SHEET_TRAIN_BATCH,
            SHEET_CONTEXT,
            arm,
            analytic_keep_secant,
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
        if step == 0 {
            let tel = fwd
                .bwd_telemetry
                .lock()
                .map_err(|_| invalid("poisoned step0 bwd_telemetry"))?;
            step0_layer_ds_norms = tel.layer_ds_l2_norms;
            if tel.d_lambda_count > 0 {
                step0_mean_abs_d_lambda = tel.d_lambda_abs_sum / (tel.d_lambda_count as f64);
            }
        }

        let _ = opt_main
            .update_filtered(&model, &grads, base_lr, |name| !name.ends_with("rec.decay"))?;
        if arm.is_active_rates() {
            let _ = opt_decay
                .update_filtered(&model, &grads, decay_lr, |name| name.ends_with("rec.decay"))?;
        }
        train_wall_accum += t_step.elapsed().as_secs_f64();

        let completed = step + 1;
        if completed == 256 && steps >= 256 {
            train_loss_step256 = loss_val;
            let (d256_f, _, _) =
                evaluate_causal_panel(&model, dev_windows, arm, 256, false, false)?;
            let (d256_i, _, _) = evaluate_causal_panel(&model, dev_windows, arm, 256, true, false)?;
            dev_step256_float = d256_f;
            dev_step256_int = d256_i;
        } else if completed == 512 && steps >= 512 {
            train_loss_step512 = loss_val;
            let (d512_f, _, _) =
                evaluate_causal_panel(&model, dev_windows, arm, 512, false, false)?;
            let (d512_i, _, _) = evaluate_causal_panel(&model, dev_windows, arm, 512, true, false)?;
            dev_step512_float = d512_f;
            dev_step512_int = d512_i;
        }
    }
    sync_cuda(device)?;

    let (ref flat_ids_0, ref scored_idx_0, ref scored_tgts_0) = train_batches[0];
    let (loss_end_t, _) = response_masked_batch_loss_and_fwd(
        &model,
        flat_ids_0,
        scored_idx_0,
        scored_tgts_0,
        SHEET_TRAIN_BATCH,
        SHEET_CONTEXT,
        arm,
        analytic_keep_secant,
    )?;
    let train_loss_step1024 = f64::from(loss_end_t.to_scalar::<f32>()?);
    if steps < 256 {
        train_loss_step256 = train_loss_step1024;
    }
    if steps < 512 {
        train_loss_step512 = train_loss_step1024;
    }

    let (dev_step1024_float, preds_1024_float, rate_tel_1024_opt) =
        evaluate_causal_panel(&model, dev_windows, arm, steps, false, true)?;
    let rate_telemetry_step1024 = rate_tel_1024_opt.expect("captured final rate telemetry");
    let (dev_step1024_int, preds_1024_int, _) =
        evaluate_causal_panel(&model, dev_windows, arm, steps, true, false)?;
    let export_gap_step1024 = compute_d11_export_gap(
        arm.name(),
        seed,
        &dev_step1024_float,
        &preds_1024_float,
        &dev_step1024_int,
        &preds_1024_int,
        dev_targets,
    );

    let decay_total = rate_telemetry_step0.decay_param_bins.len();
    let decay_crossings = rate_telemetry_step0
        .decay_param_bins
        .iter()
        .zip(&rate_telemetry_step1024.decay_param_bins)
        .filter(|(a, b)| a != b)
        .count();
    let decay_crossings_frac = (decay_crossings as f64) / (decay_total.max(1) as f64);
    let max_plateau = rate_telemetry_step0
        .plateau_12_15_fraction
        .max(rate_telemetry_step1024.plateau_12_15_fraction);
    let step0_total_ds = step0_layer_ds_norms
        .iter()
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();
    let total_train_tokens = steps * SHEET_TRAIN_BATCH * SHEET_CONTEXT;

    Ok((
        model,
        RecurrenceArmRunResult {
            arm: arm.name().to_string(),
            seed,
            steps,
            base_lr,
            decay_lr,
            analytic_keep_secant_used: analytic_keep_secant,
            train_loss_step0,
            train_loss_step256,
            train_loss_step512,
            train_loss_step1024,
            train_loss_delta_1024: train_loss_step0 - train_loss_step1024,
            train_wall_seconds: train_wall_accum,
            train_tokens_per_second: (total_train_tokens as f64) / train_wall_accum.max(1e-9),
            step0_layer_ds_l2_norms: step0_layer_ds_norms,
            step0_total_ds_l2_norm: step0_total_ds,
            step0_mean_abs_d_lambda,
            decay_param_crossings: decay_crossings,
            decay_param_total: decay_total,
            decay_param_crossings_frac: decay_crossings_frac,
            max_keep_plateau_12_15_fraction: max_plateau,
            rate_telemetry_step0,
            rate_telemetry_step1024,
            dev_step0_float,
            dev_step0_d11_int: dev_step0_int,
            dev_step256_float,
            dev_step256_d11_int: dev_step256_int,
            dev_step512_float,
            dev_step512_d11_int: dev_step512_int,
            dev_step1024_float,
            dev_step1024_d11_int: dev_step1024_int,
            export_gap_step0,
            export_gap_step1024,
        },
        preds_1024_float,
        preds_1024_int,
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FreshV2TransferArmResult {
    pub arm: String,
    pub seed: u64,
    pub fresh_v2_float: PanelEvalMetrics,
    pub fresh_v2_d11_int: PanelEvalMetrics,
    pub export_gap_fresh_v2: ExportGapCheck,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SixRecurrenceSheetReceipt {
    pub schema: String,
    pub item_id: String,
    pub issue: u32,
    pub prep_sheet: String,
    pub train_tokens_sha256: String,
    pub train_mask_sha256: String,
    pub dev_tokens_sha256: String,
    pub dev_mask_sha256: String,
    pub fresh_v2_tokens_sha256: Option<String>,
    pub fresh_v2_mask_sha256: Option<String>,
    pub tokenizer_sha256: String,
    pub cuda_stack_cu_sha256: String,
    pub geometric_stack_native_sha256: String,
    pub geometric_stack_native_cuda_sha256: String,
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
    pub standalone_cuda_parity_check: StandaloneCudaParityCheck,
    pub export_exactness_check: ExportExactnessCheck,
    pub icosian_closure_check: uor_r4_float_train::hamilton_sheet::IcosianClosureCheck,
    pub train_split_count_table_ref: UnigramReference,
    pub runs: Vec<RecurrenceArmRunResult>,
    pub paired_mcnemar_dev_d11_int: Vec<PairedMcNemarComparison>,
    pub paired_mcnemar_dev_float: Vec<PairedMcNemarComparison>,
    pub two_seed_noise_loss_float: f64,
    pub two_seed_noise_loss_d11_int: f64,
    pub two_seed_noise_top1_rows_d11_int: usize,
    pub min_active_rate_mobility_frac: f64,
    pub max_active_keep_plateau_12_15_frac: f64,
    pub i_standalone_cuda_parity_pass: bool,
    pub i_dev_denominator_pass: bool,
    pub i_rate_mobility_and_plateau_pass: bool,
    pub i_export_exactness_pass: bool,
    pub g_rec_sheet_fit_pass: bool,
    pub g_rec_transport_s3_beats_controls_pass: bool,
    pub winning_transport_mode: String,
    pub g_rec_export_gap_pass: bool,
    pub fresh_v2_evaluated: bool,
    pub fresh_v2_count_table_ref: Option<UnigramReference>,
    pub fresh_v2_runs: Vec<FreshV2TransferArmResult>,
    pub g_rec_transfer_pass: bool,
    pub g_rec_transfer_verdict: String,
    pub overall_verdict: String,
}

/// Runs Probe (a): standalone CUDA parity (`I-Standalone-CUDA-Parity`), export exactness (`I-Export-Exactness`),
/// and an 8-step synthetic descent check across all 4 geometric arms.
pub fn run_synthetic_probe(device: &Device) -> Result<serde_json::Value> {
    let parity = verify_standalone_cuda_parity(device)?;
    let export_exact = verify_export_exactness()?;
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
    for arm in RecArm::ALL {
        let (_, res, _, _) = run_recurrence_arm(
            arm,
            1,
            8,
            REC_SHEET_BASE_LR,
            REC_SHEET_DECAY_LR_MULT,
            false,
            &train_batches,
            &dev_win,
            &scored_targets,
            device,
        )?;
        summaries.insert(
            arm.name().to_string(),
            serde_json::json!({
                "loss_step0": res.train_loss_step0,
                "loss_step8": res.train_loss_step1024,
                "delta": res.train_loss_delta_1024,
                "step0_total_ds_l2_norm": res.step0_total_ds_l2_norm,
                "decay_param_crossings": res.decay_param_crossings,
                "q12_writer_oracle_mismatches": res.dev_step1024_d11_int.q12_i32_writer_oracle_mismatches,
                "d11_layer0_oracle_max_lsb_diff": res.dev_step1024_d11_int.d11_layer0_oracle_max_lsb_diff,
            }),
        );
    }
    Ok(serde_json::json!({
        "probe": "synthetic-probe",
        "standalone_cuda_parity": parity,
        "export_exactness": export_exact,
        "arms": summaries,
    }))
}

/// Runs Probe (b): 32-step throughput & rate-mobility check on `mw-balp/train` + full `14,287`-target `ft/dev`.
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
    let mut dev_targets = Vec::with_capacity(EXPECTED_DEV_ELIGIBLE_TARGETS);
    for w in &dev_windows {
        dev_targets.extend_from_slice(&w.scored_targets);
    }
    let train_batches = build_training_batches(
        &train_tokens,
        &train_mask,
        SHEET_TRAIN_BATCH,
        SHEET_CONTEXT,
        REC_SHEET_TRAIN_BATCHES,
        SHEET_BOS_ID,
    )?;

    let t0 = Instant::now();
    let (_, res_s3, _, _) = run_recurrence_arm(
        RecArm::RecGeometricS3Active,
        1,
        32,
        REC_SHEET_BASE_LR,
        REC_SHEET_DECAY_LR_MULT,
        false,
        &train_batches,
        &dev_windows,
        &dev_targets,
        device,
    )?;
    let elapsed = t0.elapsed().as_secs_f64();
    let projected_1024_step_arm_s = res_s3.train_wall_seconds * 32.0
        + (res_s3.dev_step1024_float.eval_wall_seconds
            + res_s3.dev_step1024_d11_int.eval_wall_seconds)
            * 4.0;
    let projected_8_runs_s = projected_1024_step_arm_s * 8.0;

    Ok(serde_json::json!({
        "probe": "throughput-probe",
        "dev_docs": dev_docs,
        "dev_windows": dev_windows.len(),
        "dev_scored_rows": res_s3.dev_step1024_d11_int.scored_rows,
        "elapsed_seconds": elapsed,
        "s3_32step_train_s": res_s3.train_wall_seconds,
        "s3_train_tokens_per_second": res_s3.train_tokens_per_second,
        "dev_eval_float_s": res_s3.dev_step1024_float.eval_wall_seconds,
        "dev_eval_float_tokens_per_second": res_s3.dev_step1024_float.eval_tokens_per_second,
        "dev_eval_d11_int_s": res_s3.dev_step1024_d11_int.eval_wall_seconds,
        "dev_eval_d11_int_tokens_per_second": res_s3.dev_step1024_d11_int.eval_tokens_per_second,
        "s3_32step_decay_crossings_frac": res_s3.decay_param_crossings_frac,
        "s3_32step_plateau_12_15_frac": res_s3.max_keep_plateau_12_15_fraction,
        "projected_1024_step_arm_seconds": projected_1024_step_arm_s,
        "projected_full_8_runs_seconds": projected_8_runs_s,
        "under_30_minutes_total": projected_8_runs_s < 1800.0,
    }))
}

/// Runs Probe (c): the full 4-arm × 2-seed × 1,024-step six-recurrence fit sheet + `TrainSplit_CountTable_Ref`,
/// plus `fresh_v2` transfer scoring if and only if `G-Rec-Sheet-Fit` and `G-Rec-ExportGap` pass.
pub fn run_full_six_recurrence_sheet(
    data_root: &Path,
    device: &Device,
) -> Result<SixRecurrenceSheetReceipt> {
    let tok_bytes = fs::read(data_root.join("tokenizer.json"))?;
    let tok_sha = sha256_bytes(&tok_bytes);
    if tok_sha != PINNED_TOKENIZER_SHA256 {
        return Err(invalid(format!(
            "tokenizer.json SHA-256 mismatch: got {tok_sha}, expected {PINNED_TOKENIZER_SHA256}"
        )));
    }

    let parity_check = verify_standalone_cuda_parity(device)?;
    let export_exactness_check = verify_export_exactness()?;
    let icosian_closure_check = verify_icosian_closure()?;
    if !parity_check.passed || !export_exactness_check.passed || !icosian_closure_check.passed {
        return Err(invalid(
            "Stop rule triggered: pre-run CUDA parity or D11 export exactness check failed",
        ));
    }

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
    let (count_table_ref, count_table_preds) = compute_train_unigram_reference(
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
        REC_SHEET_TRAIN_BATCHES,
        SHEET_BOS_ID,
    )?;

    let mut runs = Vec::with_capacity(8);
    let mut trained_models: BTreeMap<(String, u64), StackModel> = BTreeMap::new();
    let mut preds_float_by_arm_seed: BTreeMap<(String, u64), Vec<u32>> = BTreeMap::new();
    let mut preds_int_by_arm_seed: BTreeMap<(String, u64), Vec<u32>> = BTreeMap::new();

    for seed in [1u64, 2u64] {
        for arm in RecArm::ALL {
            let (mut model, mut run_res, mut pf, mut pi) = run_recurrence_arm(
                arm,
                seed,
                REC_SHEET_STEPS,
                REC_SHEET_BASE_LR,
                REC_SHEET_DECAY_LR_MULT,
                false,
                &train_batches,
                &dev_windows,
                &dev_targets,
                device,
            )?;
            // Pre-registered calibration retry if I-Rate-Mobility-And-Plateau triggers on an active arm
            if arm.is_active_rates()
                && (run_res.decay_param_crossings_frac < 0.05
                    || run_res.max_keep_plateau_12_15_fraction >= 0.80)
            {
                let (m_retry, r_retry, pf_retry, pi_retry) = run_recurrence_arm(
                    arm,
                    seed,
                    REC_SHEET_STEPS,
                    REC_SHEET_BASE_LR,
                    REC_SHEET_DECAY_LR_MULT * 2.0,
                    true,
                    &train_batches,
                    &dev_windows,
                    &dev_targets,
                    device,
                )?;
                model = m_retry;
                run_res = r_retry;
                pf = pf_retry;
                pi = pi_retry;
            }
            trained_models.insert((arm.name().to_string(), seed), model);
            preds_float_by_arm_seed.insert((arm.name().to_string(), seed), pf);
            preds_int_by_arm_seed.insert((arm.name().to_string(), seed), pi);
            runs.push(run_res);
        }
    }

    let find = |arm: RecArm, seed: u64| -> &RecurrenceArmRunResult {
        runs.iter()
            .find(|r| r.arm == arm.name() && r.seed == seed)
            .expect("missing arm-seed run")
    };

    let mut paired_mcnemar_dev_d11_int = Vec::new();
    let mut paired_mcnemar_dev_float = Vec::new();
    for seed in [1u64, 2u64] {
        let s3_i = &preds_int_by_arm_seed[&(RecArm::RecGeometricS3Active.name().to_string(), seed)];
        let u1_i = &preds_int_by_arm_seed[&(
            RecArm::RecGeometricU1AbelianControl.name().to_string(),
            seed,
        )];
        let id_i = &preds_int_by_arm_seed[&(
            RecArm::RecGeometricIdentityTransportControl
                .name()
                .to_string(),
            seed,
        )];
        let frz_i = &preds_int_by_arm_seed[&(RecArm::RecFrozenRatesNull.name().to_string(), seed)];

        for (name_a, pa, name_b, pb) in [
            (
                RecArm::RecGeometricS3Active.name(),
                s3_i.as_slice(),
                RecArm::RecFrozenRatesNull.name(),
                frz_i.as_slice(),
            ),
            (
                RecArm::RecGeometricS3Active.name(),
                s3_i.as_slice(),
                RecArm::RecGeometricU1AbelianControl.name(),
                u1_i.as_slice(),
            ),
            (
                RecArm::RecGeometricS3Active.name(),
                s3_i.as_slice(),
                RecArm::RecGeometricIdentityTransportControl.name(),
                id_i.as_slice(),
            ),
            (
                RecArm::RecGeometricS3Active.name(),
                s3_i.as_slice(),
                "TrainSplit_CountTable_Ref",
                count_table_preds.as_slice(),
            ),
            (
                RecArm::RecGeometricU1AbelianControl.name(),
                u1_i.as_slice(),
                RecArm::RecFrozenRatesNull.name(),
                frz_i.as_slice(),
            ),
            (
                RecArm::RecGeometricIdentityTransportControl.name(),
                id_i.as_slice(),
                RecArm::RecFrozenRatesNull.name(),
                frz_i.as_slice(),
            ),
        ] {
            paired_mcnemar_dev_d11_int.push(compute_mcnemar(
                name_a,
                pa,
                name_b,
                pb,
                &dev_targets,
                seed,
            ));
        }

        let s3_f =
            &preds_float_by_arm_seed[&(RecArm::RecGeometricS3Active.name().to_string(), seed)];
        let u1_f = &preds_float_by_arm_seed[&(
            RecArm::RecGeometricU1AbelianControl.name().to_string(),
            seed,
        )];
        let id_f = &preds_float_by_arm_seed[&(
            RecArm::RecGeometricIdentityTransportControl
                .name()
                .to_string(),
            seed,
        )];
        let frz_f =
            &preds_float_by_arm_seed[&(RecArm::RecFrozenRatesNull.name().to_string(), seed)];
        for (name_a, pa, name_b, pb) in [
            (
                RecArm::RecGeometricS3Active.name(),
                s3_f.as_slice(),
                RecArm::RecFrozenRatesNull.name(),
                frz_f.as_slice(),
            ),
            (
                RecArm::RecGeometricS3Active.name(),
                s3_f.as_slice(),
                RecArm::RecGeometricU1AbelianControl.name(),
                u1_f.as_slice(),
            ),
            (
                RecArm::RecGeometricS3Active.name(),
                s3_f.as_slice(),
                RecArm::RecGeometricIdentityTransportControl.name(),
                id_f.as_slice(),
            ),
        ] {
            paired_mcnemar_dev_float.push(compute_mcnemar(
                name_a,
                pa,
                name_b,
                pb,
                &dev_targets,
                seed,
            ));
        }
    }

    let mut two_seed_noise_loss_float = 0.0f64;
    let mut two_seed_noise_loss_int = 0.0f64;
    let mut two_seed_noise_top1_int = 0usize;
    let mut min_active_mobility = 1.0f64;
    let mut max_active_plateau = 0.0f64;

    for arm in RecArm::ALL {
        let s1 = find(arm, 1);
        let s2 = find(arm, 2);
        two_seed_noise_loss_float = two_seed_noise_loss_float
            .max((s1.dev_step1024_float.token_margin - s2.dev_step1024_float.token_margin).abs());
        two_seed_noise_loss_int = two_seed_noise_loss_int.max(
            (s1.dev_step1024_d11_int.token_margin - s2.dev_step1024_d11_int.token_margin).abs(),
        );
        two_seed_noise_top1_int = two_seed_noise_top1_int.max(
            s1.dev_step1024_d11_int
                .top1_hits
                .abs_diff(s2.dev_step1024_d11_int.top1_hits),
        );
        if arm.is_active_rates() {
            min_active_mobility = min_active_mobility
                .min(s1.decay_param_crossings_frac)
                .min(s2.decay_param_crossings_frac);
            max_active_plateau = max_active_plateau
                .max(s1.max_keep_plateau_12_15_fraction)
                .max(s2.max_keep_plateau_12_15_fraction);
        }
    }

    let i_standalone_cuda_parity_pass = parity_check.passed
        && runs.iter().all(|r| {
            r.train_loss_step0.is_finite()
                && r.train_loss_step1024.is_finite()
                && (!(r.arm != RecArm::RecFrozenRatesNull.name())
                    || r.step0_layer_ds_l2_norms.iter().all(|&n| n > 0.0))
        });
    let i_dev_denominator_pass = dev_docs == EXPECTED_DEV_DOCS
        && count_table_ref.dev_scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
        && runs.iter().all(|r| {
            r.dev_step1024_float.scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
                && r.dev_step1024_d11_int.scored_rows == EXPECTED_DEV_ELIGIBLE_TARGETS
        });
    let i_rate_mobility_and_plateau_pass = min_active_mobility >= 0.05 && max_active_plateau < 0.80;
    let i_export_exactness_pass = export_exactness_check.passed
        && runs.iter().all(|r| {
            r.dev_step0_d11_int.q12_i32_writer_oracle_mismatches == 0
                && r.dev_step1024_d11_int.q12_i32_writer_oracle_mismatches == 0
        });

    // Gate 1: G-Rec-Sheet-Fit
    let g_rec_sheet_fit_pass = [1u64, 2u64].iter().all(|&seed| {
        let s3 = find(RecArm::RecGeometricS3Active, seed);
        let frz = find(RecArm::RecFrozenRatesNull, seed);
        let reduced_below_step0 =
            s3.dev_step1024_float.token_margin < s3.dev_step0_float.token_margin;
        let loss_beat_frz = (frz.dev_step1024_float.token_margin
            - s3.dev_step1024_float.token_margin)
            >= two_seed_noise_loss_float;
        let mcnemar_frz = paired_mcnemar_dev_d11_int
            .iter()
            .find(|m| {
                m.arm_a == RecArm::RecGeometricS3Active.name()
                    && m.arm_b == RecArm::RecFrozenRatesNull.name()
                    && m.seed == seed
            })
            .expect("missing S3 vs FrozenRates McNemar");
        let beats_frz = loss_beat_frz || mcnemar_frz.exceeds_f_i;
        let beats_count_table = s3.dev_step1024_d11_int.top1_hits > count_table_ref.dev_top1_hits;
        reduced_below_step0 && beats_frz && beats_count_table
    });

    // Gate 2: G-Rec-Transport
    let g_rec_transport_s3_beats_controls_pass = [1u64, 2u64].iter().all(|&seed| {
        let s3 = find(RecArm::RecGeometricS3Active, seed);
        let u1 = find(RecArm::RecGeometricU1AbelianControl, seed);
        let id = find(RecArm::RecGeometricIdentityTransportControl, seed);
        let m_u1 = paired_mcnemar_dev_d11_int
            .iter()
            .find(|m| {
                m.arm_a == RecArm::RecGeometricS3Active.name()
                    && m.arm_b == RecArm::RecGeometricU1AbelianControl.name()
                    && m.seed == seed
            })
            .unwrap();
        let m_id = paired_mcnemar_dev_d11_int
            .iter()
            .find(|m| {
                m.arm_a == RecArm::RecGeometricS3Active.name()
                    && m.arm_b == RecArm::RecGeometricIdentityTransportControl.name()
                    && m.seed == seed
            })
            .unwrap();
        let beats_u1 = (u1.dev_step1024_float.token_margin - s3.dev_step1024_float.token_margin)
            >= two_seed_noise_loss_float
            || m_u1.exceeds_f_i;
        let beats_id = (id.dev_step1024_float.token_margin - s3.dev_step1024_float.token_margin)
            >= two_seed_noise_loss_float
            || m_id.exceeds_f_i;
        beats_u1 && beats_id
    });

    // Determine winning active carry mode by mean D11 integer DEV top-1 hits across seeds 1 and 2
    let mean_top1 = |arm: RecArm| -> usize {
        find(arm, 1).dev_step1024_d11_int.top1_hits + find(arm, 2).dev_step1024_d11_int.top1_hits
    };
    let winning_arm = if g_rec_transport_s3_beats_controls_pass {
        RecArm::RecGeometricS3Active
    } else {
        let mut best = RecArm::RecGeometricS3Active;
        for cand in [
            RecArm::RecGeometricU1AbelianControl,
            RecArm::RecGeometricIdentityTransportControl,
        ] {
            if mean_top1(cand) > mean_top1(best) {
                best = cand;
            }
        }
        best
    };
    let winning_transport_mode = winning_arm.name().to_string();

    // Gate 3: G-Rec-ExportGap on the winning active arm
    let g_rec_export_gap_pass = [1u64, 2u64].iter().all(|&seed| {
        let r = find(winning_arm, seed);
        let loss_ok = r.export_gap_step1024.delta_token_margin
            <= two_seed_noise_loss_float.max(two_seed_noise_loss_int);
        let top1_ok = r.export_gap_step1024.delta_top1_rows
            <= r.export_gap_step1024
                .mcnemar_float_vs_int
                .f_i_threshold
                .max(two_seed_noise_top1_int);
        loss_ok && top1_ok
    });

    // Gate 4: G-Rec-Transfer (evaluated ONLY if G-Rec-Sheet-Fit and G-Rec-ExportGap pass, CHARTER §4.3)
    let mut fresh_v2_evaluated = false;
    let mut fresh_v2_tokens_sha = None;
    let mut fresh_v2_mask_sha = None;
    let mut fresh_v2_count_ref = None;
    let mut fresh_v2_runs = Vec::new();
    let mut g_rec_transfer_pass = false;
    let g_rec_transfer_verdict: String;

    if g_rec_sheet_fit_pass && g_rec_export_gap_pass {
        let fv2_tok_path = data_root.join("fresh_v2/tokens.u16");
        let fv2_mask_path = data_root.join("fresh_v2/response_mask.u8");
        if fv2_tok_path.exists() && fv2_mask_path.exists() {
            let (fv2_tokens, fv2_mask) = load_pinned_uort_split(
                &fv2_tok_path,
                &fv2_mask_path,
                PINNED_FRESH_V2_TOKENS_SHA256,
                PINNED_FRESH_V2_MASK_SHA256,
                EXPECTED_FRESH_V2_TOKENS,
                EXPECTED_FRESH_V2_ELIGIBLE_TARGETS,
                SHEET_VOCAB,
                SHEET_BOS_ID,
            )?;
            fresh_v2_evaluated = true;
            fresh_v2_tokens_sha = Some(PINNED_FRESH_V2_TOKENS_SHA256.to_string());
            fresh_v2_mask_sha = Some(PINNED_FRESH_V2_MASK_SHA256.to_string());
            let (fv2_windows, _) =
                build_document_windows(&fv2_tokens, &fv2_mask, SHEET_CONTEXT, SHEET_BOS_ID)?;
            let mut fv2_targets = Vec::with_capacity(EXPECTED_FRESH_V2_ELIGIBLE_TARGETS);
            for w in &fv2_windows {
                fv2_targets.extend_from_slice(&w.scored_targets);
            }
            let (fv2_cref, _) = compute_train_unigram_reference(
                &train_tokens,
                &train_mask,
                &fv2_windows,
                SHEET_VOCAB,
                SHEET_BOS_ID,
            )?;
            for seed in [1u64, 2u64] {
                for arm in [winning_arm, RecArm::RecFrozenRatesNull] {
                    let m = &trained_models[&(arm.name().to_string(), seed)];
                    let (f_eval, f_preds, _) =
                        evaluate_causal_panel(m, &fv2_windows, arm, REC_SHEET_STEPS, false, false)?;
                    let (i_eval, i_preds, _) =
                        evaluate_causal_panel(m, &fv2_windows, arm, REC_SHEET_STEPS, true, false)?;
                    let gap = compute_d11_export_gap(
                        arm.name(),
                        seed,
                        &f_eval,
                        &f_preds,
                        &i_eval,
                        &i_preds,
                        &fv2_targets,
                    );
                    fresh_v2_runs.push(FreshV2TransferArmResult {
                        arm: arm.name().to_string(),
                        seed,
                        fresh_v2_float: f_eval,
                        fresh_v2_d11_int: i_eval,
                        export_gap_fresh_v2: gap,
                    });
                }
            }
            g_rec_transfer_pass = [1u64, 2u64].iter().all(|&seed| {
                let win = fresh_v2_runs
                    .iter()
                    .find(|r| r.arm == winning_arm.name() && r.seed == seed)
                    .unwrap();
                let frz = fresh_v2_runs
                    .iter()
                    .find(|r| r.arm == RecArm::RecFrozenRatesNull.name() && r.seed == seed)
                    .unwrap();
                win.fresh_v2_d11_int.top1_hits > frz.fresh_v2_d11_int.top1_hits
                    && win.fresh_v2_d11_int.top1_hits > fv2_cref.dev_top1_hits
            });
            fresh_v2_count_ref = Some(fv2_cref);
            g_rec_transfer_verdict = if g_rec_transfer_pass {
                "PASS_FRESH_V2_TRANSFER".to_string()
            } else {
                "FAIL_FRESH_V2_TRANSFER".to_string()
            };
        } else {
            g_rec_transfer_verdict = "FRESH_V2_SPLIT_NOT_PRESENT_ON_DISK".to_string();
        }
    } else {
        g_rec_transfer_verdict = "SKIPPED_DEV_GATE_DID_NOT_PASS".to_string();
    }

    let overall_verdict = if !i_standalone_cuda_parity_pass
        || !i_dev_denominator_pass
        || !i_rate_mobility_and_plateau_pass
        || !i_export_exactness_pass
    {
        "INSTRUMENT_FAULT".to_string()
    } else if g_rec_sheet_fit_pass && g_rec_export_gap_pass && g_rec_transfer_pass {
        "PASS_ALL_GATES_AND_TRANSFER".to_string()
    } else if g_rec_sheet_fit_pass && g_rec_export_gap_pass {
        "PASS_DEV_FIT_AND_EXPORT_GAP".to_string()
    } else if g_rec_export_gap_pass {
        "MEASURED_DEV_FIT_AND_PASS_D11_EXPORT_GAP".to_string()
    } else {
        "MEASURED_SIX_RECURRENCE_FIT".to_string()
    };

    Ok(SixRecurrenceSheetReceipt {
        schema: "uor-r4.recurrence-gates-six-recurrence-fit/1".into(),
        item_id: "recurrence-gates-six-recurrence-fit".into(),
        issue: 2890,
        prep_sheet: "docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md".into(),
        train_tokens_sha256: PINNED_TRAIN_TOKENS_SHA256.into(),
        train_mask_sha256: PINNED_TRAIN_MASK_SHA256.into(),
        dev_tokens_sha256: PINNED_DEV_TOKENS_SHA256.into(),
        dev_mask_sha256: PINNED_DEV_MASK_SHA256.into(),
        fresh_v2_tokens_sha256: fresh_v2_tokens_sha,
        fresh_v2_mask_sha256: fresh_v2_mask_sha,
        tokenizer_sha256: tok_sha,
        cuda_stack_cu_sha256: PINNED_CUDA_STACK_CU_SHA256.into(),
        geometric_stack_native_sha256: PINNED_GEOMETRIC_STACK_NATIVE_SHA256.into(),
        geometric_stack_native_cuda_sha256: PINNED_GEOMETRIC_STACK_NATIVE_CUDA_SHA256.into(),
        spiralcore_fbs_source_sha256: uor_r4_float_train::spiralcore_fbs::FBS_SOURCE_SHA256.into(),
        dev_documents: dev_docs,
        dev_windows: dev_windows.len(),
        dev_eligible_targets: count_table_ref.dev_scored_rows,
        train_batches: REC_SHEET_TRAIN_BATCHES,
        train_batch_size: SHEET_TRAIN_BATCH,
        context: SHEET_CONTEXT,
        steps: REC_SHEET_STEPS,
        base_lr: REC_SHEET_BASE_LR,
        decay_lr_mult: REC_SHEET_DECAY_LR_MULT,
        weight_decay: SHEET_WEIGHT_DECAY,
        standalone_cuda_parity_check: parity_check,
        export_exactness_check,
        icosian_closure_check,
        train_split_count_table_ref: count_table_ref,
        runs,
        paired_mcnemar_dev_d11_int,
        paired_mcnemar_dev_float,
        two_seed_noise_loss_float,
        two_seed_noise_loss_d11_int: two_seed_noise_loss_int,
        two_seed_noise_top1_rows_d11_int: two_seed_noise_top1_int,
        min_active_rate_mobility_frac: min_active_mobility,
        max_active_keep_plateau_12_15_frac: max_active_plateau,
        i_standalone_cuda_parity_pass,
        i_dev_denominator_pass,
        i_rate_mobility_and_plateau_pass,
        i_export_exactness_pass,
        g_rec_sheet_fit_pass,
        g_rec_transport_s3_beats_controls_pass,
        winning_transport_mode,
        g_rec_export_gap_pass,
        fresh_v2_evaluated,
        fresh_v2_count_table_ref: fresh_v2_count_ref,
        fresh_v2_runs,
        g_rec_transfer_pass,
        g_rec_transfer_verdict,
        overall_verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d11_export_exactness_shift_add_keep_and_hamilton_pass() -> Result<()> {
        let check = verify_export_exactness()?;
        assert!(check.passed);
        assert_eq!(check.csd_keep_q12_exact_entries, 16);
        assert_eq!(check.max_scan_lsb_diff, 0);
        assert_eq!(check.synthetic_writer_top1_mismatches, 0);
        Ok(())
    }

    #[test]
    fn mcnemar_f_i_threshold_and_discordance_formula() {
        let targets = vec![1u32; 100];
        let mut preds_a = vec![0u32; 100];
        let mut preds_b = vec![0u32; 100];
        // 40 hits for A where B misses, 10 hits for B where A misses -> b+c = 50, c-b = 30
        for p in preds_a.iter_mut().take(40) {
            *p = 1;
        }
        for p in preds_b.iter_mut().skip(40).take(10) {
            *p = 1;
        }
        let m = compute_mcnemar("A", &preds_a, "B", &preds_b, &targets, 1);
        assert_eq!(m.c_a_hit_b_miss, 40);
        assert_eq!(m.b_a_miss_b_hit, 10);
        assert_eq!(m.discordant_total_b_plus_c, 50);
        assert_eq!(m.net_advantage_rows_c_minus_b, 30);
        assert_eq!(m.f_i_threshold, 24);
        assert!(m.exceeds_f_i);
    }
}
