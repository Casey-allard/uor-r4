//! The runner of `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`
//! (spiralcore-fbs-offline-float-sheet, #2891; registered by antigravity-81, built by claude-119).
//!
//! The model is the sheet's rung-0 native geometric stack, `hamilton_sheet::build_hamilton_model`
//! (6 recurrence layers, width 256, 4 heads, MLP 384, context 192, vocabulary 4,096, Hamilton gate,
//! geometric recurrence gates, MarginHinge with 8 rivals, SpiralCore/FBS projections, an 8-cell
//! native vector head). The four arms are `spiralcore_fbs_arms::FbsArm`. Each arm and seed trains
//! for 1,024 `StackAdamW` steps on `mw-balp/train`, one fresh 4 x 192 batch per step, and is scored
//! on the 14,287-row `ft/dev` panel at steps 0, 256, 512 and 1,024, in float and in the Hamilton
//! sheet's D11-compatible export mode. That mode snaps the Hamilton gates to the 120 icosians in
//! both the loss and the top-1 evaluation; only the top-1 evaluation also rounds the writer table
//! and the scored hidden state to Q12. The rest of the stack stays float, and the SpiralCore/FBS
//! route itself is an exact (atom, action) gather. No softmax, squashing or dense matrix product
//! occurs.
//!
//! Learning rates, weight decay and clip are the Hamilton sheet's (the FBS sheet registers none of
//! its own); the frozen-query arm keeps its queries out of both updates.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};

use crate::geometric_stack::{StackAdamW, StackModel};
use crate::hamilton_sheet::{
    self as hs, CausalWindow, DevEvalMetrics, ExportGapMetrics, HamiltonArm, PairedDiscordance,
    TrainBatch, SHEET_CONTEXT, SHEET_TRAIN_BATCH,
};
use crate::spiralcore_fbs::ProjectionPlan;
use crate::spiralcore_fbs_arms::{self as arms, FbsArm};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.spiralcore-fbs-sheet-receipt/1";
pub const PREP_SHEET: &str = "docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md";
pub const ISSUE: u32 = 2891;
pub const STEPS: usize = 1_024;
pub const SEEDS: [u64; 2] = [1, 2];
pub const EVAL_STEPS: [usize; 4] = [0, 256, 512, 1_024];
/// One fresh training batch per step.
pub const TRAIN_BATCHES: usize = STEPS;
pub const PINNED_SPIRALCORE_FBS_CU_SHA256: &str =
    "517bffcc3c2029ddd171e75fd8fc1eb532755b30e683c66bd230382416480b0d";
pub const PINNED_SPIRALCORE_FBS_RS_SHA256: &str =
    "6eb2b09d5602322b7628a83f4cd3c2ee35c59d78786cb349fd92c4519ebbe3cd";
pub const PINNED_CUDA_STACK_CU_SHA256: &str =
    "c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17";
pub const PINNED_FRESH_TOKENS_SHA256: &str =
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8";
pub const PINNED_FRESH_MASK_SHA256: &str =
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e";
pub const EXPECTED_FRESH_TOKENS: usize = 44_678;
pub const EXPECTED_FRESH_TARGETS: usize = 14_479;
/// `I-Route-Mobility`: at least 1% of active query blocks change their nearest orbit root over the
/// run, while the last step changes fewer than half.
pub const ROUTE_MOBILITY_FLOOR: f64 = 0.01;
pub const ROUTE_CHURN_CEILING: f64 = 0.50;
/// Decode timing: a 128-token DEV prefix, then 64 tokens written one at a time (each step reruns
/// the stack over the whole window; the stack keeps no incremental state).
pub const DECODE_PREFIX: usize = 128;
pub const DECODE_TOKENS: usize = 64;
/// `F_I = max(24, ceil(2 sqrt(b + c)))`, the paired top-1 noise threshold.
pub const F_I_FLOOR: usize = 24;

/// Clears the arm hook when an arm's run ends, by any path.
struct ArmGuard;

impl ArmGuard {
    fn set(arm: FbsArm) -> Self {
        arms::set_fbs_arm(Some(arm));
        ArmGuard
    }
}

impl Drop for ArmGuard {
    fn drop(&mut self) {
        arms::set_fbs_arm(None);
    }
}

/// The sheet's model for `arm` at `seed`.
pub fn build_fbs_model(arm: FbsArm, seed: u64, device: &Device) -> Result<StackModel> {
    let mut model = hs::build_hamilton_model(seed, device)?;
    if arm == FbsArm::TwoAtomMix {
        model.widen_fbs_queries_two_path()?;
    }
    Ok(model)
}

fn is_query(name: &str) -> bool {
    name.ends_with(".spiralcore_fbs")
}

fn batch_loss(
    model: &StackModel,
    ids: &[u32],
    scored: &[u32],
    targets: &[u32],
    batch: usize,
) -> Result<Tensor> {
    let hidden = model.hidden(ids, batch, SHEET_CONTEXT)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    model.native_head_loss(&hidden.index_select(&idx, 0)?, targets, hs::SHEET_RIVALS)
}

/// The mean `MarginHinge(rivals = 8)` loss over every scored row of `windows`, in float or in the
/// D11-compatible export mode.
pub fn panel_loss(
    model: &StackModel,
    windows: &[CausalWindow],
    seed: u64,
    step: usize,
    export_mode: bool,
) -> Result<f64> {
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, export_mode, || {
        let mut total = 0.0f64;
        let mut rows = 0usize;
        for chunk in windows.chunks(hs::SHEET_EVAL_BATCH) {
            let mut ids = Vec::with_capacity(chunk.len() * SHEET_CONTEXT);
            let mut scored = Vec::new();
            let mut targets = Vec::new();
            for (b, window) in chunk.iter().enumerate() {
                ids.extend_from_slice(&window.input_ids);
                for (&offset, &target) in window.scored_offsets.iter().zip(&window.scored_targets) {
                    scored.push((b * SHEET_CONTEXT + offset) as u32);
                    targets.push(target);
                }
            }
            if targets.is_empty() {
                continue;
            }
            let loss = batch_loss(model, &ids, &scored, &targets, chunk.len())?;
            let value = f64::from(loss.to_scalar::<f32>()?);
            if !value.is_finite() {
                return Err(invalid("I-Finite-CUDA: non-finite panel loss"));
            }
            total += value * targets.len() as f64;
            rows += targets.len();
        }
        if rows == 0 {
            return Err(invalid("the panel has no scored rows"));
        }
        Ok(total / rows as f64)
    })
}

fn any_plan(model: &StackModel) -> Result<Arc<ProjectionPlan>> {
    model
        .variables()
        .keys()
        .find_map(|name| model.projection_plan_for_variable(name))
        .cloned()
        .ok_or_else(|| invalid("the model has no SpiralCore/FBS projection"))
}

/// Each query block's nearest orbit root, per `.spiralcore_fbs` variable, computed on the GPU
/// (broadcast differences and a sum over 8 axes; no matrix product).
fn route_codes(model: &StackModel) -> Result<Vec<Tensor>> {
    let plan = any_plan(model)?;
    let roots: Vec<f32> = plan
        .geometry
        .roots
        .iter()
        .flatten()
        .map(|&v| f32::from(v))
        .collect();
    let roots = Tensor::from_vec(roots, (1, 1, 16, 8), model.device())?;
    let mut codes = Vec::new();
    for (name, var) in model.variables() {
        if !is_query(name) {
            continue;
        }
        let q = var.as_tensor().unsqueeze(2)?;
        let distance = q.broadcast_sub(&roots)?.sqr()?.sum(3)?;
        codes.push(distance.argmin(2)?);
    }
    Ok(codes)
}

/// The fraction of query blocks whose code differs between `a` and `b`.
fn changed_fraction(a: &[Tensor], b: &[Tensor]) -> Result<(f64, usize)> {
    let mut changed = 0.0f64;
    let mut total = 0usize;
    for (x, y) in a.iter().zip(b) {
        changed += f64::from(
            x.ne(y)?
                .to_dtype(DType::F32)?
                .sum_all()?
                .to_scalar::<f32>()?,
        );
        total += x.elem_count();
    }
    Ok((changed / total.max(1) as f64, total))
}

fn query_credit_l2(model: &StackModel, grads: &candle_core::backprop::GradStore) -> Result<f64> {
    let mut sum = 0.0f64;
    for (name, var) in model.variables() {
        if is_query(name) {
            if let Some(grad) = grads.get(var.as_tensor()) {
                sum += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
            }
        }
    }
    Ok(sum.sqrt())
}

/// Decode rate: `DECODE_TOKENS` written one at a time after a `DECODE_PREFIX`-token prefix.
fn decode_tokens_per_second(model: &StackModel, window: &CausalWindow, seed: u64) -> Result<f64> {
    if window.input_ids.len() < DECODE_PREFIX + DECODE_TOKENS {
        return Err(invalid("the decode window is shorter than prefix + reply"));
    }
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let mut ids = window.input_ids[..DECODE_PREFIX].to_vec();
        let started = Instant::now();
        for _ in 0..DECODE_TOKENS {
            let next = model
                .native_next_token_ids(&ids, 1, ids.len())?
                .flatten_all()?
                .to_dtype(DType::U32)?
                .to_vec1::<u32>()?;
            let last = *next
                .last()
                .ok_or_else(|| invalid("the writer returned no token"))?;
            ids.push(last);
        }
        Ok(DECODE_TOKENS as f64 / started.elapsed().as_secs_f64().max(1e-9))
    })
}

/// One evaluation point of one arm and seed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FbsEvalPoint {
    pub step: usize,
    pub train_loss: Option<f64>,
    pub dev_loss_float: f64,
    pub dev_loss_int: f64,
    pub dev_float: DevEvalMetrics,
    pub dev_int: DevEvalMetrics,
}

/// One arm and seed's run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FbsArmRun {
    pub arm: String,
    pub seed: u64,
    pub steps: usize,
    pub eval: Vec<FbsEvalPoint>,
    pub train_wall_seconds: f64,
    pub train_tokens_per_second: f64,
    pub context_tokens_per_second: f64,
    pub decode_tokens_per_second: f64,
    pub step0_query_credit_l2: f64,
    pub query_blocks: usize,
    pub route_mobility_fraction: f64,
    pub route_churn_last_step: f64,
    pub export_gap_final: ExportGapMetrics,
    pub final_preds_float_sha256: String,
    pub final_preds_int_sha256: String,
}

impl FbsArmRun {
    pub fn final_point(&self) -> Result<&FbsEvalPoint> {
        self.eval
            .last()
            .ok_or_else(|| invalid("a run has no evaluation"))
    }
}

/// A run's final model and its final DEV predictions (float, integer export).
pub struct FbsArmOutput {
    pub run: FbsArmRun,
    pub model: StackModel,
    pub preds_float: Vec<u32>,
    pub preds_int: Vec<u32>,
}

fn preds_sha256(preds: &[u32]) -> String {
    let bytes: Vec<u8> = preds.iter().flat_map(|p| p.to_le_bytes()).collect();
    sha256_bytes(&bytes)
}

/// Trains and scores one `(arm, seed)`.
pub fn run_fbs_arm(
    arm: FbsArm,
    seed: u64,
    steps: usize,
    batches: &[TrainBatch],
    dev_windows: &[CausalWindow],
    eval_steps: &[usize],
    device: &Device,
) -> Result<FbsArmOutput> {
    if batches.is_empty() {
        return Err(invalid("no training batches"));
    }
    let _guard = ArmGuard::set(arm);
    let model = build_fbs_model(arm, seed, device)?;
    let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;
    let trains_queries = arm.trains_queries();
    let codes_start = route_codes(&model)?;
    let mut codes_before_last = codes_start.clone();
    let mut eval = Vec::new();
    let mut last_preds = (Vec::new(), Vec::new());
    let mut evaluate = |model: &StackModel, step: usize, train_loss: Option<f64>| -> Result<()> {
        let (dev_float, preds_float) = hs::evaluate_dev_panel(
            model,
            dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            step,
            false,
            false,
        )?;
        let (dev_int, preds_int) = hs::evaluate_dev_panel(
            model,
            dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            step,
            true,
            false,
        )?;
        let dev_loss_float = panel_loss(model, dev_windows, seed, step, false)?;
        let dev_loss_int = panel_loss(model, dev_windows, seed, step, true)?;
        eval.push(FbsEvalPoint {
            step,
            train_loss,
            dev_loss_float,
            dev_loss_int,
            dev_float,
            dev_int,
        });
        last_preds = (preds_float, preds_int);
        Ok(())
    };
    if eval_steps.contains(&0) {
        evaluate(&model, 0, None)?;
    }
    let mut step0_credit = 0.0f64;
    let mut train_wall = 0.0f64;
    for step in 0..steps {
        let started = Instant::now();
        let (ids, scored, targets) = &batches[step % batches.len()];
        let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!(
                "I-Finite-CUDA: non-finite training loss at step {step} in {}",
                arm.name()
            )));
        }
        let grads = loss.backward()?;
        if step == 0 {
            step0_credit = query_credit_l2(&model, &grads)?;
        }
        if step + 1 == steps {
            codes_before_last = route_codes(&model)?;
        }
        opt_main.update_filtered(&model, &grads, hs::SHEET_BASE_LR, |name| {
            !name.ends_with("rec.decay") && (trains_queries || !is_query(name))
        })?;
        opt_decay.update_filtered(
            &model,
            &grads,
            hs::SHEET_BASE_LR * hs::SHEET_DECAY_LR_MULT,
            |name| name.ends_with("rec.decay"),
        )?;
        // The step's backward and updates are queued on the GPU; wait for them before timing.
        device.synchronize()?;
        train_wall += started.elapsed().as_secs_f64();
        if eval_steps.contains(&(step + 1)) {
            evaluate(&model, step + 1, Some(value))?;
        }
    }
    let codes_end = route_codes(&model)?;
    let (mobility, query_blocks) = changed_fraction(&codes_start, &codes_end)?;
    let (churn, _) = changed_fraction(&codes_before_last, &codes_end)?;
    let decode = decode_tokens_per_second(&model, &dev_windows[0], seed)?;
    let last = eval
        .last()
        .ok_or_else(|| invalid("the run evaluated no step"))?
        .clone();
    let (preds_float, preds_int) = last_preds;
    let export_gap_final =
        hs::compute_export_gap(&last.dev_float, &preds_float, &last.dev_int, &preds_int);
    let run = FbsArmRun {
        arm: arm.name().to_string(),
        seed,
        steps,
        train_wall_seconds: train_wall,
        train_tokens_per_second: (steps * SHEET_TRAIN_BATCH * SHEET_CONTEXT) as f64
            / train_wall.max(1e-9),
        context_tokens_per_second: last.dev_float.eval_tokens_per_second,
        decode_tokens_per_second: decode,
        step0_query_credit_l2: step0_credit,
        query_blocks,
        route_mobility_fraction: mobility,
        route_churn_last_step: churn,
        export_gap_final,
        final_preds_float_sha256: preds_sha256(&preds_float),
        final_preds_int_sha256: preds_sha256(&preds_int),
        eval,
    };
    Ok(FbsArmOutput {
        run,
        model,
        preds_float,
        preds_int,
    })
}

/// `I-Export-Exactness` on a synthetic plan with random (untrained) queries, for the one-atom
/// standard forward only: the route's `(atom, action)` table, read back from a coded-carrier
/// forward, must hold one atom and one lawful Cl(0,6) signed-axis action per output block (a row
/// of the pinned geometry's integer `axes` table), and the integer evaluation of that table must
/// equal the F32 kernel exactly on integer root-aligned carriers. The identity and two-atom arms'
/// integer forms are not exercised here, and the Q12 writer's top-1 is checked separately, by the
/// pure-`i32` writer oracle inside every export-mode DEV evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportExactness {
    pub output_blocks: usize,
    pub fixture_rows: usize,
    pub mixed_atom_blocks: usize,
    pub unlawful_actions: usize,
    pub integer_mismatches: usize,
    pub pass: bool,
}

pub fn export_exactness_check(device: &Device, seed: u64) -> Result<ExportExactness> {
    let plan = ProjectionPlan::new(32 * 8, 48 * 8)?;
    let [outputs, stages, eight] = plan.query_shape();
    let inputs = plan.input_blocks;
    let mut rng = seed | 1;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let q: Vec<f32> = (0..outputs * stages * eight)
        .map(|_| ((next() >> 40) as f32 / (1u64 << 23) as f32) - 1.0)
        .collect();
    let q = Tensor::from_vec(q, (outputs, stages, eight), device)?;
    // Coded carriers: atom a, axis k holds a * 8 + k + 1.
    let coded: Vec<f32> = (0..inputs * 8).map(|i| (i + 1) as f32).collect();
    let coded = Tensor::from_vec(coded, (1, inputs * 8), device)?;
    let y = plan.forward(&q, &coded)?.flatten_all()?.to_vec1::<f32>()?;
    let axes = &plan.geometry.axes;
    let mut table = Vec::with_capacity(outputs);
    let (mut mixed, mut unlawful) = (0usize, 0usize);
    for o in 0..outputs {
        let mut atom = None;
        let mut packed = [0u32; 8];
        let mut consistent = true;
        for (j, slot) in packed.iter_mut().enumerate() {
            let v = y[o * 8 + j];
            let index = (v.abs() as usize).saturating_sub(1);
            let a = index / 8;
            if atom.is_some_and(|prev| prev != a) {
                consistent = false;
            }
            atom = Some(a);
            *slot = (index % 8) as u32 | if v < 0.0 { 8 } else { 0 };
        }
        if !consistent {
            mixed += 1;
        }
        let state = (0..64).find(|&s| (0..8).all(|j| axes[s * 8 + j] == packed[j]));
        if state.is_none() {
            unlawful += 1;
        }
        table.push((atom.unwrap_or(0), state.unwrap_or(0)));
    }
    // Integer root-aligned carriers, scored through the table in i32 and through the F32 kernel.
    let rows = 8;
    let roots = &plan.geometry.roots;
    let carriers: Vec<i32> = (0..rows * inputs)
        .flat_map(|_| roots[(next() % 16) as usize].map(i32::from))
        .collect();
    let as_f32: Vec<f32> = carriers.iter().map(|&v| v as f32).collect();
    let x = Tensor::from_vec(as_f32, (rows, inputs * 8), device)?;
    let y = plan.forward(&q, &x)?.flatten_all()?.to_vec1::<f32>()?;
    let mut mismatches = 0usize;
    for r in 0..rows {
        for (o, &(atom, state)) in table.iter().enumerate() {
            for j in 0..8 {
                let code = axes[state * 8 + j];
                let v = carriers[r * inputs * 8 + atom * 8 + (code & 7) as usize];
                let want = if code & 8 != 0 { -v } else { v };
                if y[(r * outputs + o) * 8 + j] != want as f32 {
                    mismatches += 1;
                }
            }
        }
    }
    Ok(ExportExactness {
        output_blocks: outputs,
        fixture_rows: rows,
        mixed_atom_blocks: mixed,
        unlawful_actions: unlawful,
        integer_mismatches: mismatches,
        pass: mixed == 0 && unlawful == 0 && mismatches == 0,
    })
}

/// The pinned source hashes and this module's kernel hashes.
pub fn kernel_identity() -> serde_json::Value {
    let fbs_cu = sha256_bytes(include_bytes!("spiralcore_fbs.cu"));
    let fbs_rs = sha256_bytes(include_bytes!("spiralcore_fbs.rs"));
    let stack_cu = sha256_bytes(include_bytes!("cuda_stack.cu"));
    #[cfg(feature = "cuda")]
    let arms_ptx = arms::arms_ptx_sha256();
    #[cfg(not(feature = "cuda"))]
    let arms_ptx = String::from("not built (no cuda feature)");
    serde_json::json!({
        "spiralcore_fbs_cu_sha256": fbs_cu,
        "spiralcore_fbs_rs_sha256": fbs_rs,
        "cuda_stack_cu_sha256": stack_cu,
        "pins_match_sheet": fbs_cu == PINNED_SPIRALCORE_FBS_CU_SHA256
            && fbs_rs == PINNED_SPIRALCORE_FBS_RS_SHA256
            && stack_cu == PINNED_CUDA_STACK_CU_SHA256,
        "arms_kernel_sha256": arms::arms_kernel_sha256(),
        "arms_prelude_sha256": arms::arms_prelude_sha256(),
        "arms_ptx_sha256": arms_ptx,
    })
}

/// Deterministic synthetic batches for probe (a): random tokens, every position scored.
fn synthetic_batches(count: usize, seed: u64) -> Vec<TrainBatch> {
    let mut rng = seed | 1;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..count)
        .map(|_| {
            let ids: Vec<u32> = (0..SHEET_TRAIN_BATCH * SHEET_CONTEXT)
                .map(|_| 1 + (next() % (hs::SHEET_VOCAB as u64 - 1)) as u32)
                .collect();
            let mut scored = Vec::new();
            let mut targets = Vec::new();
            for b in 0..SHEET_TRAIN_BATCH {
                for t in 0..SHEET_CONTEXT - 1 {
                    scored.push((b * SHEET_CONTEXT + t) as u32);
                    targets.push(ids[b * SHEET_CONTEXT + t + 1]);
                }
            }
            (ids, scored, targets)
        })
        .collect()
}

/// Probe (a): `I-Export-Exactness`, and an 8-step descent check of all four arms on synthetic
/// batches (`I-Finite-CUDA`: finite losses, nonzero step-0 query credit in the trained arms).
pub fn run_synthetic_probe(device: &Device) -> Result<serde_json::Value> {
    let exactness = export_exactness_check(device, 0x5eed)?;
    let batches = synthetic_batches(4, 0x51ab);
    let mut arms_out = Vec::new();
    let mut finite = true;
    for arm in FbsArm::ALL {
        let _guard = ArmGuard::set(arm);
        let model = build_fbs_model(arm, 1, device)?;
        let mut opt = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
        let trains_queries = arm.trains_queries();
        let mut losses = Vec::new();
        let mut credit = 0.0f64;
        for step in 0..8 {
            let (ids, scored, targets) = &batches[step % batches.len()];
            let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH)?;
            let value = f64::from(loss.to_scalar::<f32>()?);
            finite &= value.is_finite();
            losses.push(value);
            let grads = loss.backward()?;
            if step == 0 {
                credit = query_credit_l2(&model, &grads)?;
            }
            opt.update_filtered(&model, &grads, hs::SHEET_BASE_LR, |name| {
                trains_queries || !is_query(name)
            })?;
        }
        finite &= credit.is_finite();
        arms_out.push(serde_json::json!({
            "arm": arm.name(), "losses": losses, "step0_query_credit_l2": credit,
            "credit_nonzero_when_trained": !trains_queries || credit > 0.0,
        }));
    }
    let credit_ok = arms_out
        .iter()
        .all(|a| a["credit_nonzero_when_trained"] == serde_json::json!(true));
    let exact = exactness.pass;
    Ok(serde_json::json!({
        "probe": "synthetic-probe",
        "kernels": kernel_identity(),
        "i_export_exactness": exactness,
        "arms": arms_out,
        "i_finite_cuda_pass": finite && credit_ok,
        "pass": finite && credit_ok && exact,
    }))
}

/// The pinned TRAIN and DEV splits under `data_root`, and the training batches.
fn load_inputs(
    data_root: &Path,
    batches: usize,
) -> Result<(Vec<TrainBatch>, Vec<CausalWindow>, usize)> {
    let (train_tokens, train_mask) = hs::load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        hs::PINNED_TRAIN_TOKENS_SHA256,
        hs::PINNED_TRAIN_MASK_SHA256,
        hs::EXPECTED_TRAIN_TOKENS,
        hs::EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (dev_windows, dev_docs) =
        hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let train = hs::build_training_batches(
        &train_tokens,
        &train_mask,
        SHEET_TRAIN_BATCH,
        SHEET_CONTEXT,
        batches,
        hs::SHEET_BOS_ID,
    )?;
    Ok((train, dev_windows, dev_docs))
}

/// Probe (b): 32 steps of the two trained-route arms on `mw-balp/train`, the full DEV panel in
/// both modes (`I-DEV-Denominator`), route mobility, and the projected full-run wall time.
pub fn run_throughput_probe(data_root: &Path, device: &Device) -> Result<serde_json::Value> {
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, 32)?;
    let mut runs = Vec::new();
    for arm in [FbsArm::OneAtomActive, FbsArm::TwoAtomMix] {
        let out = run_fbs_arm(arm, 1, 32, &batches, &dev_windows, &[0, 32], device)?;
        runs.push(out.run);
    }
    let last = runs[0].final_point()?;
    let denominators = runs.iter().all(|run| {
        run.eval.iter().all(|p| {
            p.dev_float.scored_rows == hs::EXPECTED_DEV_ELIGIBLE_TARGETS
                && p.dev_int.scored_rows == hs::EXPECTED_DEV_ELIGIBLE_TARGETS
        })
    });
    let per_step = runs
        .iter()
        .map(|r| r.train_wall_seconds / 32.0)
        .fold(0.0f64, f64::max);
    let eval_seconds = last.dev_float.eval_wall_seconds + last.dev_int.eval_wall_seconds;
    let projected_arm = per_step * STEPS as f64 + eval_seconds * 2.0 * EVAL_STEPS.len() as f64;
    Ok(serde_json::json!({
        "probe": "throughput-probe",
        "kernels": kernel_identity(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "i_dev_denominator_pass": denominators,
        "runs": runs,
        "projected_seconds_per_arm_seed": projected_arm,
        "projected_full_seconds": projected_arm * (FbsArm::ALL.len() * SEEDS.len()) as f64,
        "under_one_hour_per_arm": projected_arm < 3600.0,
    }))
}

fn f_i(b: usize, c: usize) -> usize {
    let bound = (2.0 * ((b + c) as f64).sqrt()).ceil() as usize;
    bound.max(F_I_FLOOR)
}

/// Whether arm `a` beats arm `c` on one seed: a strictly lower final DEV loss by at least the
/// noise band, or a paired top-1 advantage (rows `a` hits and `c` misses, less the reverse) of at
/// least `F_I`.
fn wins(loss_a: f64, loss_c: f64, noise: f64, paired: &PairedDiscordance) -> bool {
    let advantage = paired.a_hit_b_miss_c as isize - paired.a_miss_b_hit_b as isize;
    let threshold = f_i(paired.a_hit_b_miss_c, paired.a_miss_b_hit_b) as isize;
    (loss_c - loss_a >= noise && loss_c > loss_a) || advantage >= threshold
}

/// What the gates read from one arm and seed: the step-0 and final float DEV loss, the final
/// export-mode DEV loss, and the final float and export-mode top-1 predictions.
pub struct GateRun<'a> {
    pub loss0: f64,
    pub loss_float: f64,
    pub loss_int: f64,
    pub preds_float: &'a [u32],
    pub preds_int: &'a [u32],
}

/// The sheet's section 4 verdicts on DEV, computed from the final numbers alone.
#[derive(Clone, Debug)]
pub struct GateVerdicts {
    pub noise: f64,
    /// Active arm against each control, float top-1, per seed (G-Fbs-Sheet-Fit).
    pub paired: Vec<PairedDiscordance>,
    /// The exported arm's float against export-mode top-1, per seed (G-Fbs-ExportGap).
    pub export_paired: Vec<PairedDiscordance>,
    pub fit_arm: Option<FbsArm>,
    pub capacity_winner: FbsArm,
    pub export_arm: FbsArm,
    pub export_ok: bool,
}

/// `G-Fbs-Sheet-Fit`, `G-Fbs-Capacity` and `G-Fbs-ExportGap` over every arm and seed.
pub fn evaluate_gates(
    runs: &BTreeMap<(FbsArm, u64), GateRun<'_>>,
    targets: &[u32],
) -> Result<GateVerdicts> {
    let run = |arm: FbsArm, seed: u64| {
        runs.get(&(arm, seed))
            .ok_or_else(|| invalid(format!("no run for {} seed {seed}", arm.name())))
    };
    // The two-seed noise band: the largest seed-to-seed gap in final float DEV loss, over arms.
    let mut noise = 0.0f64;
    for arm in FbsArm::ALL {
        let gap = run(arm, SEEDS[0])?.loss_float - run(arm, SEEDS[1])?.loss_float;
        noise = noise.max(gap.abs());
    }
    // G-Fbs-Sheet-Fit: an active arm descends and beats both controls on both seeds.
    let mut paired = Vec::new();
    let mut fitting = Vec::new();
    for active in [FbsArm::OneAtomActive, FbsArm::TwoAtomMix] {
        let mut all = true;
        for seed in SEEDS {
            let a = run(active, seed)?;
            let mut ok = a.loss_float < a.loss0;
            for control in [FbsArm::FrozenQueries, FbsArm::IdentityTransport] {
                let c = run(control, seed)?;
                let p = hs::compute_paired_discordance(
                    active.name(),
                    a.preds_float,
                    control.name(),
                    c.preds_float,
                    targets,
                    seed,
                );
                ok &= wins(a.loss_float, c.loss_float, noise, &p);
                paired.push(p);
            }
            all &= ok;
        }
        if all {
            fitting.push(active);
        }
    }
    // G-Fbs-Capacity: two paths win only by the noise band on both seeds.
    let mut two_wins = true;
    for seed in SEEDS {
        let one = run(FbsArm::OneAtomActive, seed)?.loss_float;
        two_wins &= run(FbsArm::TwoAtomMix, seed)?.loss_float <= one - noise;
    }
    let capacity_winner = if two_wins {
        FbsArm::TwoAtomMix
    } else {
        FbsArm::OneAtomActive
    };
    // The exported arm is the one fitting arm when exactly one fits, else the capacity winner.
    let export_arm = match fitting.as_slice() {
        [only] => *only,
        _ => capacity_winner,
    };
    // G-Fbs-ExportGap: the exported arm's integer export within the noise band on both seeds.
    let mut export_paired = Vec::new();
    let mut export_ok = true;
    for seed in SEEDS {
        let r = run(export_arm, seed)?;
        let gap = hs::compute_paired_discordance(
            "float",
            r.preds_float,
            "int",
            r.preds_int,
            targets,
            seed,
        );
        let net = (gap.a_hit_b_miss_c as isize - gap.a_miss_b_hit_b as isize).unsigned_abs();
        export_ok &= (r.loss_int - r.loss_float).abs() <= noise
            && net <= f_i(gap.a_hit_b_miss_c, gap.a_miss_b_hit_b);
        export_paired.push(gap);
    }
    Ok(GateVerdicts {
        noise,
        paired,
        export_paired,
        fit_arm: fitting.first().copied(),
        capacity_winner,
        export_arm,
        export_ok,
    })
}

/// `G-Fbs-Transfer` on the sealed `fresh_v2` draw: the exported arm against the frozen null on
/// both seeds. Every failure, a missing draw included, comes back as an error for the caller to
/// record as blocked.
fn score_fresh_v2(
    data_root: &Path,
    outputs: &BTreeMap<(FbsArm, u64), FbsArmOutput>,
    arm: FbsArm,
    noise: f64,
) -> Result<(bool, serde_json::Value)> {
    let (fresh_tokens, fresh_mask) = hs::load_pinned_uort_split(
        &data_root.join("fresh_v2/tokens.u16"),
        &data_root.join("fresh_v2/response_mask.u8"),
        PINNED_FRESH_TOKENS_SHA256,
        PINNED_FRESH_MASK_SHA256,
        EXPECTED_FRESH_TOKENS,
        EXPECTED_FRESH_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (fresh_windows, _) =
        hs::build_document_windows(&fresh_tokens, &fresh_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let fresh_targets: Vec<u32> = fresh_windows
        .iter()
        .flat_map(|w| w.scored_targets.iter().copied())
        .collect();
    let mut rows = Vec::new();
    let mut all = true;
    for seed in SEEDS {
        let mut scored = BTreeMap::new();
        for which in [arm, FbsArm::FrozenQueries] {
            let out = outputs
                .get(&(which, seed))
                .ok_or_else(|| invalid(format!("no run for {} seed {seed}", which.name())))?;
            let _guard = ArmGuard::set(which);
            let (metrics, preds) = hs::evaluate_dev_panel(
                &out.model,
                &fresh_windows,
                HamiltonArm::HamiltonActive,
                seed,
                STEPS,
                false,
                false,
            )?;
            let l = panel_loss(&out.model, &fresh_windows, seed, STEPS, false)?;
            scored.insert(which, (metrics, preds, l));
        }
        let (wm, wp, wl) = &scored[&arm];
        let (fm, fp, fl) = &scored[&FbsArm::FrozenQueries];
        let p = hs::compute_paired_discordance(
            arm.name(),
            wp,
            FbsArm::FrozenQueries.name(),
            fp,
            &fresh_targets,
            seed,
        );
        let won = wins(*wl, *fl, noise, &p);
        all &= won;
        rows.push(serde_json::json!({
            "seed": seed, "winner_loss": wl, "frozen_loss": fl,
            "winner_top1": wm.top1_hits, "frozen_top1": fm.top1_hits,
            "paired": p, "winner_beats_frozen": won,
        }));
    }
    Ok((
        all,
        serde_json::json!({"targets": fresh_targets.len(), "seeds": rows}),
    ))
}

/// The receipt of the full run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FbsSheetReceipt {
    pub schema: String,
    pub item_id: String,
    pub issue: u32,
    pub prep_sheet: String,
    pub kernels: serde_json::Value,
    pub steps: usize,
    pub seeds: Vec<u64>,
    pub train_batches: usize,
    pub base_lr: f64,
    pub decay_lr_mult: f64,
    pub weight_decay: f64,
    pub dev_documents: usize,
    pub dev_windows: usize,
    pub runs: Vec<FbsArmRun>,
    pub noise_band_dev_loss: f64,
    pub paired: Vec<PairedDiscordance>,
    /// The exported arm's float against export-mode top-1 discordance, per seed.
    #[serde(default)]
    pub export_paired: Vec<PairedDiscordance>,
    pub i_finite_cuda_pass: bool,
    pub i_dev_denominator_pass: bool,
    pub i_route_mobility_pass: bool,
    pub i_export_exactness: ExportExactness,
    pub g_sheet_fit: bool,
    pub g_sheet_fit_arm: Option<String>,
    pub g_capacity_winner: String,
    /// The arm `G-Fbs-ExportGap` and `G-Fbs-Transfer` score.
    #[serde(default)]
    pub g_export_gap_arm: String,
    pub g_export_gap: bool,
    pub g_transfer: Option<bool>,
    pub fresh_v2: Option<serde_json::Value>,
    pub results_sha256: String,
}

/// Probe (c): the full 4-arm x 2-seed x 1,024-step sheet, its gates, and `fresh_v2` only when
/// `G-Fbs-Sheet-Fit` and `G-Fbs-ExportGap` pass.
pub fn run_full_sheet(data_root: &Path, device: &Device) -> Result<FbsSheetReceipt> {
    let exactness = export_exactness_check(device, 0x5eed)?;
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, TRAIN_BATCHES)?;
    let targets: Vec<u32> = dev_windows
        .iter()
        .flat_map(|w| w.scored_targets.iter().copied())
        .collect();
    let mut outputs: BTreeMap<(FbsArm, u64), FbsArmOutput> = BTreeMap::new();
    for seed in SEEDS {
        for arm in FbsArm::ALL {
            let out = run_fbs_arm(
                arm,
                seed,
                STEPS,
                &batches,
                &dev_windows,
                &EVAL_STEPS,
                device,
            )?;
            outputs.insert((arm, seed), out);
        }
    }
    let mut gate_runs = BTreeMap::new();
    for (&key, out) in &outputs {
        let last = out.run.final_point()?;
        gate_runs.insert(
            key,
            GateRun {
                loss0: out.run.eval[0].dev_loss_float,
                loss_float: last.dev_loss_float,
                loss_int: last.dev_loss_int,
                preds_float: &out.preds_float,
                preds_int: &out.preds_int,
            },
        );
    }
    let gates = evaluate_gates(&gate_runs, &targets)?;
    let runs: Vec<FbsArmRun> = outputs.values().map(|o| o.run.clone()).collect();
    let i_dev = runs.iter().all(|run| {
        run.eval.iter().all(|p| {
            p.dev_float.scored_rows == hs::EXPECTED_DEV_ELIGIBLE_TARGETS
                && p.dev_int.scored_rows == hs::EXPECTED_DEV_ELIGIBLE_TARGETS
        })
    });
    let i_finite = runs.iter().all(|run| {
        let trained = run.arm != FbsArm::FrozenQueries.name();
        run.step0_query_credit_l2.is_finite() && (!trained || run.step0_query_credit_l2 > 0.0)
    });
    let i_mobility = runs
        .iter()
        .filter(|run| {
            run.arm == FbsArm::OneAtomActive.name() || run.arm == FbsArm::TwoAtomMix.name()
        })
        .all(|run| {
            run.route_mobility_fraction >= ROUTE_MOBILITY_FLOOR
                && run.route_churn_last_step < ROUTE_CHURN_CEILING
        });
    // Any failure on the sealed draw blocks G-Fbs-Transfer: it is recorded, never a negative, and
    // never discards the DEV results above.
    let (transfer, fresh) = if gates.fit_arm.is_some() && gates.export_ok {
        match score_fresh_v2(data_root, &outputs, gates.export_arm, gates.noise) {
            Ok((won, value)) => (Some(won), Some(value)),
            Err(error) => (
                None,
                Some(serde_json::json!({
                    "status": "blocked: G-Fbs-Transfer could not be scored on the sealed fresh_v2 draw",
                    "error": error.to_string(),
                })),
            ),
        }
    } else {
        (None, None)
    };
    let mut digest = Vec::new();
    for run in &runs {
        digest.extend_from_slice(run.arm.as_bytes());
        digest.extend_from_slice(&run.seed.to_le_bytes());
        digest.extend_from_slice(run.final_preds_float_sha256.as_bytes());
        digest.extend_from_slice(run.final_preds_int_sha256.as_bytes());
        for point in &run.eval {
            digest.extend_from_slice(&point.dev_loss_float.to_le_bytes());
            digest.extend_from_slice(&point.dev_loss_int.to_le_bytes());
        }
    }
    let receipt = FbsSheetReceipt {
        schema: SCHEMA.to_string(),
        item_id: "spiralcore-fbs-offline-float-sheet".to_string(),
        issue: ISSUE,
        prep_sheet: PREP_SHEET.to_string(),
        kernels: kernel_identity(),
        steps: STEPS,
        seeds: SEEDS.to_vec(),
        train_batches: batches.len(),
        base_lr: hs::SHEET_BASE_LR,
        decay_lr_mult: hs::SHEET_DECAY_LR_MULT,
        weight_decay: hs::SHEET_WEIGHT_DECAY,
        dev_documents: dev_docs,
        dev_windows: dev_windows.len(),
        runs,
        noise_band_dev_loss: gates.noise,
        paired: gates.paired,
        export_paired: gates.export_paired,
        i_finite_cuda_pass: i_finite,
        i_dev_denominator_pass: i_dev,
        i_route_mobility_pass: i_mobility,
        i_export_exactness: exactness,
        g_sheet_fit: gates.fit_arm.is_some(),
        g_sheet_fit_arm: gates.fit_arm.map(|a| a.name().to_string()),
        g_capacity_winner: gates.capacity_winner.name().to_string(),
        g_export_gap_arm: gates.export_arm.name().to_string(),
        g_export_gap: gates.export_ok,
        g_transfer: transfer,
        fresh_v2: fresh,
        results_sha256: sha256_bytes(&digest),
    };
    Ok(receipt)
}

/// Writes `value` as pretty JSON to `path`, never over an existing file (the file is created
/// exclusively, so a concurrent writer cannot be overwritten either).
pub fn write_receipt(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| invalid(e.to_string()))?;
    text.push('\n');
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            invalid(format!(
                "{}: {e}; receipts are never overwritten",
                path.display()
            ))
        })?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_noise_threshold_follows_the_sheet() {
        assert_eq!(f_i(0, 0), 24);
        assert_eq!(f_i(100, 100), 29); // ceil(2 sqrt 200) = ceil(28.28)
        assert_eq!(f_i(50, 50), 24); // ceil(2 sqrt 100) = 20, floored at 24
    }

    const ROWS: usize = 2_000;

    /// Predictions over targets `0..ROWS` that hit the first `hits` rows and miss the rest.
    fn hitting(hits: usize) -> Vec<u32> {
        (0..ROWS as u32)
            .map(|t| if (t as usize) < hits { t } else { u32::MAX })
            .collect()
    }

    /// The gates over a table where every arm and seed starts at loss 3.0 and ends at
    /// `cell(arm, seed) = (float loss, export loss, float hits, export hits)`.
    fn gates_for(cell: impl Fn(FbsArm, u64) -> (f64, f64, usize, usize)) -> GateVerdicts {
        let targets: Vec<u32> = (0..ROWS as u32).collect();
        let mut table = BTreeMap::new();
        for seed in SEEDS {
            for arm in FbsArm::ALL {
                let (loss, loss_int, hits, hits_int) = cell(arm, seed);
                table.insert(
                    (arm, seed),
                    (loss, loss_int, hitting(hits), hitting(hits_int)),
                );
            }
        }
        let runs = table
            .iter()
            .map(|(&key, (loss, loss_int, float, int))| {
                let run = GateRun {
                    loss0: 3.0,
                    loss_float: *loss,
                    loss_int: *loss_int,
                    preds_float: float,
                    preds_int: int,
                };
                (key, run)
            })
            .collect();
        evaluate_gates(&runs, &targets).expect("gates")
    }

    fn is_active(arm: FbsArm) -> bool {
        matches!(arm, FbsArm::OneAtomActive | FbsArm::TwoAtomMix)
    }

    #[test]
    fn the_fit_gate_needs_both_controls_on_both_seeds() {
        // Equal losses (noise 0); the active arms hit 500 rows, the controls 100.
        let base = |arm: FbsArm, _seed: u64| {
            let hits = if is_active(arm) { 500 } else { 100 };
            (2.0, 2.0, hits, hits)
        };
        let gates = gates_for(base);
        assert_eq!(gates.fit_arm, Some(FbsArm::OneAtomActive));
        assert_eq!(
            gates.paired.len(),
            8,
            "2 active arms x 2 seeds x 2 controls"
        );
        // One control out-hits both active arms on one seed, either seed: no arm fits.
        for bad in SEEDS {
            let gates = gates_for(|arm, seed| match arm {
                FbsArm::IdentityTransport if seed == bad => (2.0, 2.0, 600, 600),
                _ => base(arm, seed),
            });
            assert_eq!(gates.fit_arm, None, "a control wins on seed {bad}");
        }
        // Winning on top-1 is not enough without descent below step 0 (3.0).
        let gates = gates_for(|arm, seed| {
            let (_, _, hits, hits_int) = base(arm, seed);
            (3.5, 3.5, hits, hits_int)
        });
        assert_eq!(gates.fit_arm, None, "no arm descends");
    }

    #[test]
    fn the_fit_gate_counts_top1_in_the_active_arms_favour_only() {
        // The identity control loses on loss either way. Against the frozen null only top-1 can
        // decide, and the active arms hit fewer rows: a reversed count would pass the gate.
        let gates = gates_for(|arm, _| match arm {
            FbsArm::FrozenQueries => (2.0, 2.0, 500, 500),
            FbsArm::IdentityTransport => (2.5, 2.5, 100, 100),
            _ => (2.0, 2.0, 100, 100),
        });
        assert_eq!(gates.fit_arm, None);
    }

    #[test]
    fn a_loss_win_needs_the_full_noise_band_and_a_strict_gap() {
        // Seed 2 runs 0.1 above seed 1 on every arm, so the noise band is 0.1; top-1 is tied.
        let offset = |seed: u64| if seed == 2 { 0.1 } else { 0.0 };
        let gates = gates_for(|arm, seed| {
            let loss = if is_active(arm) { 1.85 } else { 2.0 } + offset(seed);
            (loss, loss, 300, 300)
        });
        assert!((gates.noise - 0.1).abs() < 1e-9);
        assert_eq!(
            gates.fit_arm,
            Some(FbsArm::OneAtomActive),
            "0.15 below both controls"
        );
        let gates = gates_for(|arm, seed| {
            let loss = if is_active(arm) { 1.95 } else { 2.0 } + offset(seed);
            (loss, loss, 300, 300)
        });
        assert_eq!(gates.fit_arm, None, "0.05 is inside the band");
        // A zero band never turns a tie into a win.
        let gates = gates_for(|_, _| (2.0, 2.0, 300, 300));
        assert_eq!(gates.noise, 0.0);
        assert_eq!(gates.fit_arm, None);
    }

    #[test]
    fn capacity_picks_two_paths_only_by_the_band_and_a_lone_fitting_arm_is_exported() {
        // Two paths 0.2 below one path on both seeds (band 0.1): capacity picks two paths.
        let offset = |seed: u64| if seed == 2 { 0.1 } else { 0.0 };
        let cell = |arm: FbsArm, seed: u64| {
            let loss = match arm {
                FbsArm::TwoAtomMix => 1.6,
                FbsArm::OneAtomActive => 1.8,
                _ => 2.0,
            } + offset(seed);
            // Only the one-atom arm out-hits the controls.
            let hits = if arm == FbsArm::OneAtomActive {
                900
            } else {
                100
            };
            (loss, loss, hits, hits)
        };
        let gates = gates_for(cell);
        assert_eq!(gates.capacity_winner, FbsArm::TwoAtomMix);
        // Both arms beat the controls on loss, so both fit and the capacity winner is exported.
        assert_eq!(gates.export_arm, FbsArm::TwoAtomMix);
        // Give the frozen null the lowest loss: the two-path arm (100 hits, like the controls) no
        // longer fits, while the one-atom arm still fits on top-1. Capacity still picks two paths
        // on loss, but the lone fitting one-atom arm is the one exported.
        let gates = gates_for(|arm, seed| match arm {
            FbsArm::FrozenQueries => (1.55 + offset(seed), 1.55 + offset(seed), 100, 100),
            _ => cell(arm, seed),
        });
        assert_eq!(gates.fit_arm, Some(FbsArm::OneAtomActive));
        assert_eq!(gates.capacity_winner, FbsArm::TwoAtomMix);
        assert_eq!(gates.export_arm, FbsArm::OneAtomActive);
    }

    #[test]
    fn the_export_gap_reads_the_loss_gap_and_the_paired_top1_gap() {
        let offset = |seed: u64| if seed == 2 { 0.1 } else { 0.0 };
        let cell = |arm: FbsArm, seed: u64, int_loss_gap: f64, int_hits: usize| {
            let loss = if arm == FbsArm::OneAtomActive {
                1.8
            } else {
                2.0
            } + offset(seed);
            let hits = if arm == FbsArm::OneAtomActive {
                900
            } else {
                100
            };
            let hits_int = if arm == FbsArm::OneAtomActive {
                int_hits
            } else {
                hits
            };
            (loss, loss + int_loss_gap, hits, hits_int)
        };
        let gates = gates_for(|arm, seed| cell(arm, seed, 0.05, 890));
        assert!(
            gates.export_ok,
            "a 0.05 loss gap and 10 rows are inside the band and F_I"
        );
        assert_eq!(gates.export_paired.len(), 2);
        assert_eq!(gates.export_paired[0].a_hit_b_miss_c, 10);
        let gates = gates_for(|arm, seed| cell(arm, seed, 0.05, 500));
        assert!(!gates.export_ok, "400 rows exceed F_I = 40");
        let gates = gates_for(|arm, seed| cell(arm, seed, 0.2, 900));
        assert!(!gates.export_ok, "a 0.2 loss gap exceeds the 0.1 band");
        // An export loss 0.2 below the float loss, on seed 1 only: the gap is absolute, and one
        // failing seed fails the gate.
        let gates = gates_for(|arm, seed| {
            let (loss, _, hits, hits_int) = cell(arm, seed, 0.0, 900);
            let gap = if seed == 1 { -0.2 } else { 0.0 };
            (loss, loss + gap, hits, hits_int)
        });
        assert!(
            !gates.export_ok,
            "a 0.2 loss gap below the float loss on seed 1"
        );
    }

    #[test]
    fn capacity_keeps_one_atom_unless_two_paths_win_by_the_band_on_both_seeds() {
        // The frozen null's 0.3 seed gap is the band (the largest over arms); the one-atom and
        // control arms move by 0.1, and both active arms beat the controls on top-1.
        let with_two = |two: [f64; 2]| {
            gates_for(move |arm, seed| {
                let s = (seed - 1) as usize;
                let (loss, hits) = match arm {
                    FbsArm::TwoAtomMix => (two[s], 500),
                    FbsArm::OneAtomActive => ([1.8, 1.9][s], 500),
                    FbsArm::FrozenQueries => ([2.0, 2.3][s], 100),
                    FbsArm::IdentityTransport => ([2.0, 2.1][s], 100),
                };
                (loss, loss, hits, hits)
            })
        };
        let gates = with_two([1.35, 1.45]);
        assert!(
            (gates.noise - 0.3).abs() < 1e-9,
            "the band is the largest arm gap"
        );
        assert_eq!(
            gates.capacity_winner,
            FbsArm::TwoAtomMix,
            "0.45 below on both"
        );
        assert_eq!(
            with_two([1.6, 1.7]).capacity_winner,
            FbsArm::OneAtomActive,
            "0.2 below on both seeds is inside the 0.3 band"
        );
        assert_eq!(
            with_two([1.7, 1.55]).capacity_winner,
            FbsArm::OneAtomActive,
            "by the band on seed 2 only"
        );
        assert_eq!(
            with_two([1.45, 1.7]).capacity_winner,
            FbsArm::OneAtomActive,
            "by the band on seed 1 only"
        );
    }

    #[test]
    fn the_schedule_is_the_sheets() {
        assert_eq!(STEPS * SHEET_TRAIN_BATCH * SHEET_CONTEXT, 786_432);
        assert_eq!(EVAL_STEPS, [0, 256, 512, 1_024]);
        assert_eq!(SEEDS, [1, 2]);
        const { assert!(DECODE_PREFIX + DECODE_TOKENS <= SHEET_CONTEXT) };
    }

    #[test]
    fn the_pins_are_the_sheets_and_still_match_the_sources() {
        let identity = kernel_identity();
        assert_eq!(identity["pins_match_sheet"], serde_json::json!(true));
    }

    #[test]
    fn synthetic_batches_score_next_tokens() {
        let batches = synthetic_batches(2, 7);
        let (ids, scored, targets) = &batches[0];
        assert_eq!(ids.len(), SHEET_TRAIN_BATCH * SHEET_CONTEXT);
        assert_eq!(scored.len(), targets.len());
        assert_eq!(targets[0], ids[1]);
        assert!(ids
            .iter()
            .all(|&t| t != hs::SHEET_BOS_ID && (t as usize) < hs::SHEET_VOCAB));
    }
}
