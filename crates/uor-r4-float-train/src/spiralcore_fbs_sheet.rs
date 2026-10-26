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
//! route itself is an exact (atom, action) gather. The arithmetic stays within the declared
//! geometric contract.
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
    q12_query: bool,
) -> Result<Tensor> {
    let hidden = model.hidden(ids, batch, SHEET_CONTEXT)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    let query = hidden.index_select(&idx, 0)?;
    let query = if q12_query {
        hs::q12_query(&query)?
    } else {
        query
    };
    model.native_head_loss(&query, targets, hs::SHEET_RIVALS)
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
        scored_panel_loss(model, windows, false)
    })
}

/// [`panel_loss`] in the D11 mode (claude-126): the trunk computed as the exported artifact
/// computes it (`hamilton_sheet::with_d11_mode`) and each scored query rounded to Q12 before the
/// head, on a trunk whose tables are rounded as exported (an unrounded trunk is refused).
pub fn panel_loss_d11(model: &StackModel, windows: &[CausalWindow]) -> Result<f64> {
    crate::fbs_export::check_tables_rounded_as_exported(model)?;
    hs::with_d11_mode(|| scored_panel_loss(model, windows, true))
}

/// The mean loss over every scored row of `windows` under the gate hook as it is set.
fn scored_panel_loss(model: &StackModel, windows: &[CausalWindow], q12_query: bool) -> Result<f64> {
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
        let loss = batch_loss(model, &ids, &scored, &targets, chunk.len(), q12_query)?;
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
    run_fbs_arm_with(
        arm,
        false,
        seed,
        steps,
        batches,
        dev_windows,
        eval_steps,
        device,
    )
}

/// [`run_fbs_arm`], with every training step in the D11 mode when `d11_train` (claude-130): the
/// loss is the D11 forward's, its scored query at Q12, through `hamilton_sheet::with_d11_training`'s
/// bridges, with the step's backward inside the same mode. The evaluations are unchanged.
#[allow(clippy::too_many_arguments)]
pub fn run_fbs_arm_with(
    arm: FbsArm,
    d11_train: bool,
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
        let loss_and_grads = || -> Result<(f64, candle_core::backprop::GradStore)> {
            let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH, d11_train)?;
            let value = f64::from(loss.to_scalar::<f32>()?);
            if !value.is_finite() {
                return Err(invalid(format!(
                    "I-Finite-CUDA: non-finite training loss at step {step} in {}",
                    arm.name()
                )));
            }
            Ok((value, loss.backward()?))
        };
        let (value, grads) = if d11_train {
            hs::with_d11_training(seed, step, loss_and_grads)?
        } else {
            loss_and_grads()?
        };
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
            let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH, false)?;
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

/// The pinned DEV split under `data_root` as its causal windows, and its document count.
fn load_dev(data_root: &Path) -> Result<(Vec<CausalWindow>, usize)> {
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
    hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)
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
    let (dev_windows, dev_docs) = load_dev(data_root)?;
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

/// The export-gap bar, Stage 3's `G-Fbs-ExportGap` and Stage 4b's `G-D11-ExportGap` on one seed:
/// the exported computation's DEV loss within `noise` of the float loss, and the paired top-1
/// net gap (`paired` holds float against export) within `F_I`.
pub fn export_gap_within(
    noise: f64,
    loss_float: f64,
    loss_export: f64,
    paired: &PairedDiscordance,
) -> bool {
    let net = (paired.a_hit_b_miss_c as isize - paired.a_miss_b_hit_b as isize).unsigned_abs();
    (loss_export - loss_float).abs() <= noise
        && net <= f_i(paired.a_hit_b_miss_c, paired.a_miss_b_hit_b)
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
        export_ok &= export_gap_within(noise, r.loss_float, r.loss_int, &gap);
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

/// The arm Stage 4b exports (claude-126): only one-atom routes convert
/// (`fbs_export::decode_coded_routes`), and Stage 3's capacity gate kept one atom.
pub const D11_EXPORT_ARM: FbsArm = FbsArm::OneAtomActive;

/// Stage 4b's parity sample: every eighth `ft/dev` window, fixed before the run. The readouts
/// read these windows whole, so the parity with the integer session covers real text.
pub const PARITY_WINDOW_STRIDE: usize = 8;

/// The panel's D11 predictions against the D11 readout's next tokens at the same positions, over
/// every scored row: `next[w]` is window `w` read alone (`fbs_export::fbs_float_next_tokens`).
/// Returns `(rows compared, rows that differ)`. The panel batches windows and the readout reads
/// one, so this ties the panel to the readout path the integer parity covers.
fn panel_against_readout(
    windows: &[CausalWindow],
    panel_preds: &[u32],
    next: &[Vec<u32>],
) -> Result<(usize, usize)> {
    if next.len() != windows.len() {
        return Err(invalid("the readout did not read every window"));
    }
    let (mut row, mut differ) = (0usize, 0usize);
    for (w, window) in windows.iter().enumerate() {
        for &offset in &window.scored_offsets {
            let read = next[w]
                .get(offset)
                .ok_or_else(|| invalid(format!("the readout lacks window {w} offset {offset}")))?;
            let pred = panel_preds
                .get(row)
                .ok_or_else(|| invalid("fewer panel predictions than scored rows"))?;
            differ += usize::from(pred != read);
            row += 1;
        }
    }
    if row != panel_preds.len() {
        return Err(invalid("more panel predictions than scored rows"));
    }
    Ok((row, differ))
}

/// Stage 4b's run (`docs/harness/prep/d11-exporter-float-trunk-stage4b-2026-10-26.md`): the four
/// Stage 3 arms trained on both seeds, for the noise band; and, for [`D11_EXPORT_ARM`] on each
/// seed, the D11 export gap, with the trained model, its snapshot and its plain, export and D11
/// readouts saved under `save_dir/seed-<seed>/` (a directory that must not exist yet). GPU only.
pub fn run_d11_export_gap(
    data_root: &Path,
    save_dir: &Path,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::{
        fbs_float_next_tokens, fbs_float_readout_on, fbs_trunk_snapshot, round_tables_as_exported,
    };
    use uor_r4_integer::fbs_snapshot::{write_new_file, FbsReadoutMode};

    fs::create_dir(save_dir)?;
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, TRAIN_BATCHES)?;
    let targets: Vec<u32> = dev_windows
        .iter()
        .flat_map(|w| w.scored_targets.iter().copied())
        .collect();
    let sample: Vec<Vec<u32>> = dev_windows
        .iter()
        .step_by(PARITY_WINDOW_STRIDE)
        .map(|w| w.input_ids.clone())
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
    let gate_runs: BTreeMap<(FbsArm, u64), GateRun<'_>> = outputs
        .iter()
        .map(|(&key, out)| -> Result<_> {
            let last = out.run.final_point()?;
            Ok((
                key,
                GateRun {
                    loss0: out.run.eval[0].dev_loss_float,
                    loss_float: last.dev_loss_float,
                    loss_int: last.dev_loss_int,
                    preds_float: &out.preds_float,
                    preds_int: &out.preds_int,
                },
            ))
        })
        .collect::<Result<_>>()?;
    // The noise band, and Stage 3's export-mode readings, exactly as the Stage 3 sheet computes them.
    let stage3 = evaluate_gates(&gate_runs, &targets)?;

    let mut seeds = Vec::new();
    let mut gap_ok = true;
    let mut instruments_ok = true;
    for seed in SEEDS {
        let out = &outputs[&(D11_EXPORT_ARM, seed)];
        let last = out.run.final_point()?;
        let dir = save_dir.join(format!("seed-{seed}"));
        fs::create_dir(&dir)?;
        out.model.save(&dir.join("model"))?;
        let loaded = StackModel::load(&dir.join("model"), device)?;
        // I-Save-Load: the saved model scores the plain panel exactly as the trained one did.
        let (_, reloaded_preds) = hs::evaluate_dev_panel(
            &loaded,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            STEPS,
            false,
            false,
        )?;
        let reloaded_loss = panel_loss(&loaded, &dev_windows, seed, STEPS, false)?;
        let i_save_load = reloaded_preds == out.preds_float && reloaded_loss == last.dev_loss_float;
        hs::clear_active_arm();
        let snapshot = fbs_trunk_snapshot(&loaded)?;
        let plain = fbs_float_readout_on(&loaded, FbsReadoutMode::Plain, &sample)?;
        let export = fbs_float_readout_on(&loaded, FbsReadoutMode::Export, &sample)?;
        round_tables_as_exported(&loaded)?;
        let d11_readout = fbs_float_readout_on(&loaded, FbsReadoutMode::D11, &sample)?;
        let (d11, d11_preds) = hs::evaluate_dev_panel_d11(&loaded, &dev_windows)?;
        let (_, d11_again) = hs::evaluate_dev_panel_d11(&loaded, &dev_windows)?;
        let d11_loss = panel_loss_d11(&loaded, &dev_windows)?;
        // m1: the panel's D11 path against the readout's on every scored row; the integer
        // parity (m2, from `fbs-compare` on the sample) covers the readout's.
        let all_windows: Vec<Vec<u32>> = dev_windows.iter().map(|w| w.input_ids.clone()).collect();
        let d11_next = fbs_float_next_tokens(&loaded, FbsReadoutMode::D11, &all_windows)?;
        let (panel_rows, panel_differ) =
            panel_against_readout(&dev_windows, &d11_preds, &d11_next)?;
        let paired = hs::compute_paired_discordance(
            "float",
            &out.preds_float,
            "d11",
            &d11_preds,
            &targets,
            seed,
        );
        let within = export_gap_within(stage3.noise, last.dev_loss_float, d11_loss, &paired);
        let i_determinism = d11_again == d11_preds;
        let i_dev = d11.scored_rows == hs::EXPECTED_DEV_ELIGIBLE_TARGETS;
        let i_q12 = d11.q12_i32_oracle_checked_rows > 0 && d11.q12_i32_oracle_mismatches == 0;
        instruments_ok &= i_save_load && i_determinism && i_dev && i_q12;
        gap_ok &= within;
        for (name, bytes) in [
            ("snapshot.json", serde_json::to_vec(&snapshot)?),
            ("readout.json", serde_json::to_vec(&plain)?),
            ("readout-export.json", serde_json::to_vec(&export)?),
            ("readout-d11.json", serde_json::to_vec(&d11_readout)?),
            ("preds-float.json", serde_json::to_vec(&out.preds_float)?),
            ("preds-d11.json", serde_json::to_vec(&d11_preds)?),
        ] {
            write_new_file(&dir.join(name), &bytes)?;
        }
        let net = paired.a_hit_b_miss_c as isize - paired.a_miss_b_hit_b as isize;
        seeds.push(serde_json::json!({
            "seed": seed,
            "arm": D11_EXPORT_ARM.name(),
            "float": last.dev_float,
            "d11": d11,
            "export_mode": last.dev_int,
            "loss_float": last.dev_loss_float,
            "loss_d11": d11_loss,
            "loss_export_mode": last.dev_loss_int,
            "loss_gap_d11": d11_loss - last.dev_loss_float,
            "paired_float_vs_d11": paired,
            "net_gap_rows": net,
            "f_i": f_i(paired.a_hit_b_miss_c, paired.a_miss_b_hit_b),
            "within_band": within,
            "within_band_scope": "the GPU half on this seed; the gate also needs I-D11-Artifact-Agreement",
            "preds_float_sha256": preds_sha256(&out.preds_float),
            "preds_d11_sha256": preds_sha256(&d11_preds),
            "i_save_load": i_save_load,
            "reloaded_loss_float": reloaded_loss,
            "i_d_determinism": i_determinism,
            "i_dev_denominator": i_dev,
            "i_q12_oracle": i_q12,
            "panel_vs_readout_rows": panel_rows,
            "panel_vs_readout_differ": panel_differ,
        }));
    }
    let runs: Vec<&FbsArmRun> = outputs.values().map(|o| &o.run).collect();
    Ok(serde_json::json!({
        "sheet": "docs/harness/prep/d11-exporter-float-trunk-stage4b-2026-10-26.md",
        "kernels": kernel_identity(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "parity_window_stride": PARITY_WINDOW_STRIDE,
        "parity_windows": sample.len(),
        "parity_positions": sample.iter().map(Vec::len).sum::<usize>(),
        "noise_band_dev_loss": stage3.noise,
        "stage3_export_mode_paired": stage3.export_paired,
        "stage3_export_mode_arm": stage3.export_arm.name(),
        "seeds": seeds,
        "gpu_instruments_pass": instruments_ok,
        // Half of G-D11-ExportGap: the gate also needs I-D11-Artifact-Agreement, from
        // `fbs-compare` on each seed's outputs and `panel_vs_readout_differ`. A failed
        // instrument check is an instrument fault: the gap is not counted, never FAIL.
        "gap_inside_band_on_both_seeds": if instruments_ok {
            serde_json::json!(gap_ok)
        } else {
            serde_json::json!("not counted: an instrument check failed")
        },
        "runs": runs,
    }))
}

/// The DEV panel's windows as a sequences file for the GPU evaluator of the exported artifact
/// (`d11-fbs-eval run`, claude-128/129): every window's `SHEET_CONTEXT` input ids, in panel order.
pub fn d11_artifact_windows(data_root: &Path) -> Result<serde_json::Value> {
    let (dev_windows, dev_docs) = load_dev(data_root)?;
    let rows: usize = dev_windows.iter().map(|w| w.scored_targets.len()).sum();
    if rows != hs::EXPECTED_DEV_ELIGIBLE_TARGETS {
        return Err(invalid(
            "I-DEV-Denominator: the windows do not hold every DEV target",
        ));
    }
    let sequences: Vec<&Vec<u32>> = dev_windows.iter().map(|w| &w.input_ids).collect();
    if sequences.iter().any(|s| s.len() != SHEET_CONTEXT) {
        return Err(invalid("a DEV window is not SHEET_CONTEXT tokens long"));
    }
    Ok(serde_json::json!({
        "sequences": sequences,
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "scored_rows": rows,
    }))
}

/// The artifact's prediction and Q12 query (as `q / 4096`) at every scored row, in panel order:
/// window `w`'s row at `offset` is the evaluator's output at position `offset` of sequence `w`,
/// the step that has read `input_ids[..=offset]`, as the float panel scores the hidden state there.
fn artifact_rows(
    windows: &[CausalWindow],
    tokens: &[u32],
    queries: &[i32],
    len: usize,
    width: usize,
) -> Result<(Vec<u32>, Vec<f32>)> {
    let mut preds = Vec::new();
    let mut rows = Vec::new();
    for (w, window) in windows.iter().enumerate() {
        for &offset in &window.scored_offsets {
            if offset >= len {
                return Err(invalid(format!(
                    "window {w} offset {offset} lies past the run"
                )));
            }
            let at = w * len + offset;
            let (Some(&token), Some(query)) =
                (tokens.get(at), queries.get(at * width..(at + 1) * width))
            else {
                return Err(invalid(format!("the run lacks window {w} offset {offset}")));
            };
            preds.push(token);
            rows.extend(query.iter().map(|&q| q as f32 / hs::Q12_SCALE));
        }
    }
    Ok((preds, rows))
}

/// The per-position outputs of `d11-fbs-eval run --verify` (claude-128's evaluator).
#[derive(Deserialize)]
struct EvaluatorRun {
    artifact_sha256: String,
    sequences_sha256: String,
    sequences: usize,
    len: usize,
    width: usize,
    tokens: Vec<u32>,
    queries: Vec<i32>,
    verify: serde_json::Value,
    build: serde_json::Value,
}

/// `G-D11-ExportGap` with D the exported artifact itself (claude-129): the GPU evaluator's tokens
/// and Q12 queries at every scored row of the DEV panel, held against the saved float model's own
/// panel on the GPU. The artifact's top-1 is the evaluator's token at each scored offset; its loss
/// is the same `MarginHinge(rivals = 8)` the D11 mode scores (`panel_loss_d11`), applied to the
/// evaluator's Q12 query with the tables rounded as exported. `stage4b` is claude-126's receipt,
/// for the two-seed noise band and the stored float predictions' loss; `stored_preds` its
/// `preds-float.json` for this seed. GPU only.
#[allow(clippy::too_many_arguments)]
pub fn run_d11_artifact_gap(
    data_root: &Path,
    model_dir: &Path,
    evaluator: &Path,
    windows_sha256: &str,
    artifact_sha256: &str,
    stage4b: &Path,
    stored_preds: &Path,
    seed: u64,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::round_tables_as_exported;

    let (dev_windows, dev_docs) = load_dev(data_root)?;
    let targets: Vec<u32> = dev_windows
        .iter()
        .flat_map(|w| w.scored_targets.iter().copied())
        .collect();
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(stage4b)?)?;
    let noise = receipt["noise_band_dev_loss"]
        .as_f64()
        .ok_or_else(|| invalid("the Stage 4b receipt has no noise band"))?;
    let stage4b_seed = receipt["seeds"]
        .as_array()
        .and_then(|s| s.iter().find(|v| v["seed"].as_u64() == Some(seed)))
        .ok_or_else(|| invalid(format!("the Stage 4b receipt has no seed {seed}")))?;
    let stored_loss = stage4b_seed["loss_float"]
        .as_f64()
        .ok_or_else(|| invalid("the Stage 4b receipt has no float loss"))?;
    let stored: Vec<u32> = serde_json::from_slice(&fs::read(stored_preds)?)?;

    let loaded = StackModel::load(model_dir, device)?;
    // I-Save-Load: the saved model's plain panel is claude-126's, row for row.
    let (float, float_preds) = hs::evaluate_dev_panel(
        &loaded,
        &dev_windows,
        HamiltonArm::HamiltonActive,
        seed,
        STEPS,
        false,
        false,
    )?;
    let loss_float = panel_loss(&loaded, &dev_windows, seed, STEPS, false)?;
    hs::clear_active_arm();
    let i_save_load = float_preds == stored && (loss_float - stored_loss).abs() <= 1e-6;

    // The evaluator's run: the panel's windows, verified bit for bit against the Rust session.
    let run: EvaluatorRun = serde_json::from_slice(&fs::read(evaluator)?)?;
    let width = loaded.config.width;
    let shape_ok = run.sequences == dev_windows.len()
        && run.len == SHEET_CONTEXT
        && run.width == width
        && run.tokens.len() == run.sequences * run.len
        && run.queries.len() == run.tokens.len() * width;
    if !shape_ok {
        return Err(invalid(
            "the evaluator's run does not cover the DEV windows",
        ));
    }
    // I-Inputs: the evaluator read the sheet's windows and this seed's pinned artifact.
    let i_inputs = run.sequences_sha256 == windows_sha256 && run.artifact_sha256 == artifact_sha256;
    let model_sha256 = sha256_bytes(&fs::read(model_dir.join("model.safetensors"))?);
    let config_sha256 = sha256_bytes(&fs::read(model_dir.join("config.json"))?);
    let i_verify = run.verify.as_array().is_some_and(|checks| {
        checks.len() == 2
            && checks.iter().all(|c| c["equal"] == serde_json::json!(true))
            && checks
                .iter()
                .all(|c| c["positions"].as_u64() == Some((run.sequences * run.len) as u64))
    });
    let (preds, queries) = artifact_rows(&dev_windows, &run.tokens, &run.queries, run.len, width)?;
    let i_dev = preds.len() == hs::EXPECTED_DEV_ELIGIBLE_TARGETS;

    // The artifact's loss: its own Q12 queries through the D11 head on the exported tables.
    round_tables_as_exported(&loaded)?;
    let (mut total, mut rows) = (0.0f64, 0usize);
    for (q_chunk, t_chunk) in queries
        .chunks(hs::SHEET_EVAL_BATCH * SHEET_CONTEXT * width)
        .zip(targets.chunks(hs::SHEET_EVAL_BATCH * SHEET_CONTEXT))
    {
        let query = Tensor::from_vec(q_chunk.to_vec(), (t_chunk.len(), width), device)?;
        let loss =
            hs::with_d11_mode(|| loaded.native_head_loss(&query, t_chunk, hs::SHEET_RIVALS))?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid("I-Finite-CUDA: non-finite artifact loss"));
        }
        total += value * t_chunk.len() as f64;
        rows += t_chunk.len();
    }
    let loss_artifact = total / rows.max(1) as f64;
    let top1_artifact = preds.iter().zip(&targets).filter(|(p, t)| p == t).count();
    let paired =
        hs::compute_paired_discordance("float", &float_preds, "artifact", &preds, &targets, seed);
    let within = export_gap_within(noise, loss_float, loss_artifact, &paired);
    let instruments_ok = i_save_load && i_inputs && i_verify && i_dev;
    let net = paired.a_hit_b_miss_c as isize - paired.a_miss_b_hit_b as isize;
    Ok(serde_json::json!({
        "sheet": "docs/harness/prep/d11-exporter-float-trunk-artifact-gap-2026-10-26.md",
        "seed": seed,
        "arm": D11_EXPORT_ARM.name(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "artifact_sha256": run.artifact_sha256,
        "evaluator_build": run.build,
        "windows_sha256": windows_sha256,
        "artifact_sha256_pinned": artifact_sha256,
        "model_safetensors_sha256": model_sha256,
        "model_config_sha256": config_sha256,
        "float": float,
        "top1_artifact": top1_artifact,
        "loss_float": loss_float,
        "loss_float_stage4b": stored_loss,
        "loss_artifact": loss_artifact,
        "loss_gap": loss_artifact - loss_float,
        "noise_band_dev_loss": noise,
        "paired_float_vs_artifact": paired,
        "net_gap_rows": net,
        "f_i": f_i(paired.a_hit_b_miss_c, paired.a_miss_b_hit_b),
        "within_band": within,
        "preds_artifact_sha256": preds_sha256(&preds),
        "preds_float_sha256": preds_sha256(&float_preds),
        "i_save_load": i_save_load,
        "i_inputs": i_inputs,
        "i_verify_all": i_verify,
        "i_dev_denominator": i_dev,
        "instruments_pass": instruments_ok,
        "gap_inside_band": if instruments_ok {
            serde_json::json!(within)
        } else {
            serde_json::json!("not counted: an instrument check failed")
        },
    }))
}

/// Probe (b) of D11-mode training (claude-130): 32 D11-mode steps of [`D11_EXPORT_ARM`] on seed 1,
/// the DEV panel at steps 0 and 32, and the projected wall time of the full run (two seeds of
/// [`STEPS`] steps, each evaluated at [`EVAL_STEPS`]).
pub fn run_d11_train_probe(data_root: &Path, device: &Device) -> Result<serde_json::Value> {
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, 32)?;
    let out = run_fbs_arm_with(
        D11_EXPORT_ARM,
        true,
        1,
        32,
        &batches,
        &dev_windows,
        &[0, 1, 32],
        device,
    )?;
    let first = &out.run.eval[0];
    // The evaluation after step 1 carries step 0's D11-mode training loss.
    let after_first = &out.run.eval[1];
    let last = out.run.final_point()?;
    let per_step = out.run.train_wall_seconds / 32.0;
    let eval_seconds = last.dev_float.eval_wall_seconds + last.dev_int.eval_wall_seconds;
    let projected_seed = per_step * STEPS as f64 + eval_seconds * 2.0 * EVAL_STEPS.len() as f64;
    let train_learns = match (after_first.train_loss, last.train_loss) {
        (Some(a), Some(b)) => b < a,
        _ => false,
    };
    Ok(serde_json::json!({
        "probe": "d11-train-probe",
        "kernels": kernel_identity(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "train_loss_step_0": after_first.train_loss,
        "train_loss_step_31": last.train_loss,
        "train_learns": train_learns,
        "dev_loss_float_step_0": first.dev_loss_float,
        "dev_loss_float_step_32": last.dev_loss_float,
        "learns": last.dev_loss_float < first.dev_loss_float,
        "step0_query_credit_l2": out.run.step0_query_credit_l2,
        "seconds_per_step": per_step,
        "projected_seconds_per_seed": projected_seed,
        "projected_full_seconds": projected_seed * SEEDS.len() as f64,
        "under_one_hour_per_seed": projected_seed < 3600.0,
        "run": out.run,
    }))
}

/// D11-mode training and export (claude-130): [`D11_EXPORT_ARM`] trained on both seeds with every
/// step in the D11 mode (`run_fbs_arm_with`, `d11_train`), each saved model reloaded and checked
/// (`I-Save-Load`), and its snapshot and plain float predictions written under
/// `save_dir/seed-<seed>/` (a directory that must not exist yet) for `fbs-export` and the
/// artifact-gap scorer. The receipt carries Stage 4b's noise band, read from `stage4b`, and each
/// seed's float loss, in the shape `run_d11_artifact_gap` reads as its `stage4b` receipt.
pub fn run_d11_train_export(
    data_root: &Path,
    save_dir: &Path,
    stage4b: &Path,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::fbs_trunk_snapshot;
    use uor_r4_integer::fbs_snapshot::write_new_file;

    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(stage4b)?)?;
    let noise = receipt["noise_band_dev_loss"]
        .as_f64()
        .ok_or_else(|| invalid("the Stage 4b receipt has no noise band"))?;
    fs::create_dir(save_dir)?;
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, TRAIN_BATCHES)?;
    let mut seeds = Vec::new();
    let mut instruments_ok = true;
    for seed in SEEDS {
        let out = run_fbs_arm_with(
            D11_EXPORT_ARM,
            true,
            seed,
            STEPS,
            &batches,
            &dev_windows,
            &EVAL_STEPS,
            device,
        )?;
        let last = out.run.final_point()?;
        let dir = save_dir.join(format!("seed-{seed}"));
        fs::create_dir(&dir)?;
        out.model.save(&dir.join("model"))?;
        let loaded = StackModel::load(&dir.join("model"), device)?;
        let (_, reloaded_preds) = hs::evaluate_dev_panel(
            &loaded,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            STEPS,
            false,
            false,
        )?;
        let reloaded_loss = panel_loss(&loaded, &dev_windows, seed, STEPS, false)?;
        let i_save_load = reloaded_preds == out.preds_float && reloaded_loss == last.dev_loss_float;
        hs::clear_active_arm();
        instruments_ok &= i_save_load;
        let snapshot = fbs_trunk_snapshot(&loaded)?;
        write_new_file(&dir.join("snapshot.json"), &serde_json::to_vec(&snapshot)?)?;
        write_new_file(
            &dir.join("preds-float.json"),
            &serde_json::to_vec(&out.preds_float)?,
        )?;
        seeds.push(serde_json::json!({
            "seed": seed,
            "arm": D11_EXPORT_ARM.name(),
            "d11_trained": true,
            "float": last.dev_float,
            "export_mode": last.dev_int,
            "loss_float": last.dev_loss_float,
            "loss_export_mode": last.dev_loss_int,
            "preds_float_sha256": preds_sha256(&out.preds_float),
            "i_save_load": i_save_load,
            "reloaded_loss_float": reloaded_loss,
        }));
    }
    Ok(serde_json::json!({
        "sheet": "docs/harness/prep/d11-exporter-float-trunk-d11-train-2026-10-26.md",
        "kernels": kernel_identity(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "noise_band_dev_loss": noise,
        "noise_band_source": "claude-126's Stage 4b receipt (d11-gap.json)",
        "seeds": seeds,
        "instruments_pass": instruments_ok,
    }))
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

    #[test]
    fn the_export_gap_bar_holds_the_loss_band_and_the_paired_threshold() {
        let paired = |b: usize, c: usize| PairedDiscordance {
            arm_a: "float".into(),
            arm_b: "d11".into(),
            seed: 1,
            a_hit_b_miss_c: b,
            a_miss_b_hit_b: c,
            net_advantage_rows: b as isize - c as isize,
        };
        // Below 144 discordant rows F_I is its floor, 24: a net gap of 24 is inside, 25 is not,
        // in either direction.
        assert!(export_gap_within(0.25, 2.0, 2.25, &paired(60, 36)));
        assert!(export_gap_within(0.25, 2.0, 1.75, &paired(36, 60)));
        assert!(!export_gap_within(0.25, 2.0, 2.0, &paired(61, 36)));
        assert!(!export_gap_within(0.25, 2.0, 2.0, &paired(36, 61)));
        // The loss gap counts in either direction, up to the band.
        assert!(!export_gap_within(0.25, 2.0, 2.3, &paired(0, 0)));
        assert!(!export_gap_within(0.25, 2.0, 1.7, &paired(0, 0)));
        // Above it F_I grows: 400 discordant rows give 40, 402 give 41.
        assert!(export_gap_within(0.0, 1.0, 1.0, &paired(220, 180)));
        assert!(export_gap_within(0.0, 1.0, 1.0, &paired(180, 220)));
        assert!(!export_gap_within(0.0, 1.0, 1.0, &paired(222, 180)));
    }

    #[test]
    fn the_panel_is_held_against_the_readout_on_every_scored_row() -> Result<()> {
        let window = |scored: Vec<usize>| CausalWindow {
            doc_id: 0,
            input_ids: vec![1; 4],
            scored_offsets: scored.clone(),
            scored_targets: vec![0; scored.len()],
            scored_corpus_positions: vec![0; scored.len()],
        };
        let windows = vec![window(vec![1, 3]), window(vec![0]), window(vec![2])];
        let next = vec![vec![0, 5, 0, 6], vec![9, 0, 0, 0], vec![0, 0, 7, 0]];
        // Rows, in panel order: window 0 offsets 1 and 3, window 1 offset 0, window 2 offset 2.
        assert_eq!(
            panel_against_readout(&windows, &[5, 6, 9, 7], &next)?,
            (4, 0)
        );
        assert_eq!(
            panel_against_readout(&windows, &[5, 4, 9, 1], &next)?,
            (4, 2)
        );
        // Every window's rows count, the middle one's too.
        assert_eq!(
            panel_against_readout(&windows, &[5, 6, 0, 7], &next)?,
            (4, 1)
        );
        assert!(panel_against_readout(&windows, &[5, 6, 9], &next).is_err());
        assert!(panel_against_readout(&windows, &[5, 6, 9, 7, 1], &next).is_err());
        assert!(panel_against_readout(&windows, &[5, 6, 9, 7], &next[..2]).is_err());
        Ok(())
    }

    #[test]
    fn the_artifact_is_read_at_each_scored_offset_in_panel_order() -> Result<()> {
        // claude-129: the evaluator's output at position `offset` of window `w` is row
        // (w, offset), and its query is read as `q / 4096`.
        let window = |scored: Vec<usize>| CausalWindow {
            doc_id: 0,
            input_ids: vec![1; 4],
            scored_offsets: scored.clone(),
            scored_targets: vec![0; scored.len()],
            scored_corpus_positions: vec![0; scored.len()],
        };
        let windows = vec![window(vec![1, 3]), window(vec![0]), window(vec![2])];
        let (len, width) = (4, 2);
        let tokens: Vec<u32> = (0..12).collect();
        let queries: Vec<i32> = (0..24).map(|i| i * 4096).collect();
        let (preds, rows) = artifact_rows(&windows, &tokens, &queries, len, width)?;
        // Window 0 offsets 1 and 3, window 1 offset 0, window 2 offset 2.
        assert_eq!(preds, vec![1, 3, 4, 10]);
        assert_eq!(rows, vec![2.0, 3.0, 6.0, 7.0, 8.0, 9.0, 20.0, 21.0]);
        // A run that stops short, or an offset past the run's length, is refused.
        assert!(artifact_rows(&windows, &tokens[..10], &queries, len, width).is_err());
        assert!(artifact_rows(&[window(vec![4])], &tokens, &queries, len, width).is_err());
        Ok(())
    }

    #[test]
    fn the_d11_loss_refuses_an_unrounded_trunk() -> Result<()> {
        // A CPU-built trunk: the guard reads the tables only, before any tensor operation.
        let model = build_fbs_model(D11_EXPORT_ARM, 1, &Device::Cpu)?;
        let unrounded =
            |e: crate::FloatTrainError| e.to_string().contains("rounded as exported first");
        assert!(panel_loss_d11(&model, &[]).is_err_and(unrounded));
        crate::fbs_export::round_tables_as_exported(&model)?;
        // Rounded, it gets past the guard and stops at the empty panel, the hook restored.
        assert!(panel_loss_d11(&model, &[])
            .is_err_and(|e| e.to_string().contains("the panel has no scored rows")));
        assert!(!hs::is_snapping_transition());
        Ok(())
    }

    #[test]
    fn a_saved_trunk_loads_back_exactly() -> Result<()> {
        // The run reloads each exported model from disk; the reload must be the trained trunk.
        let model = build_fbs_model(D11_EXPORT_ARM, 2, &Device::Cpu)?;
        let root = std::path::PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .ok_or_else(|| invalid("owned artifact directory required"))?,
        )
        .join(format!(
            "fbs-save-load-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        model.save(&root)?;
        let loaded = StackModel::load(&root, &Device::Cpu)?;
        assert_eq!(loaded.variables().len(), model.variables().len());
        for (name, var) in model.variables() {
            let bits = |t: &Tensor| -> Result<Vec<u32>> {
                Ok(t.flatten_all()?
                    .to_dtype(DType::F32)?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|v| v.to_bits())
                    .collect())
            };
            let other = loaded
                .variables()
                .get(name)
                .ok_or_else(|| invalid(format!("{name} did not load")))?;
            assert_eq!(bits(var.as_tensor())?, bits(other.as_tensor())?, "{name}");
        }
        assert_eq!(loaded.quaternion_weights(), model.quaternion_weights());
        assert_eq!(
            loaded.native_vector_head_spec(),
            model.native_vector_head_spec()
        );
        assert_eq!(
            model.native_vector_head_spec().map(|h| h.kept_cells),
            Some(hs::SHEET_KEPT_CELLS)
        );
        fs::remove_dir_all(&root)?;
        Ok(())
    }
}
