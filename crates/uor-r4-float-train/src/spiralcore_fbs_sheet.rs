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

use candle_core::{backprop::GradStore, DType, Device, Tensor};
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
        TrainForward::Plain,
        seed,
        steps,
        batches,
        dev_windows,
        eval_steps,
        device,
    )
}

/// How an arm's training steps compute their loss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainForward {
    /// The plain forward, as Stage 3 and Stage 4b train.
    Plain,
    /// Every step in the D11 mode, its scored query at Q12, through
    /// `hamilton_sheet::with_d11_training`'s bridges (claude-130).
    D11,
    /// The plain loss plus the D11-mode loss, weighted 1:1, so both forwards are trained
    /// (claude-131, the decision review after claude-130).
    Dual,
}

impl TrainForward {
    pub fn name(self) -> &'static str {
        match self {
            TrainForward::Plain => "plain",
            TrainForward::D11 => "d11",
            TrainForward::Dual => "dual",
        }
    }

    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "plain" => Ok(TrainForward::Plain),
            "d11" => Ok(TrainForward::D11),
            "dual" => Ok(TrainForward::Dual),
            other => Err(invalid(format!("unknown training forward {other}"))),
        }
    }
}

/// `a + b`, tensor by tensor: a gradient in only one of them is kept as it is.
fn sum_grads(mut a: GradStore, b: &GradStore) -> Result<GradStore> {
    let ids: Vec<_> = b.get_ids().copied().collect();
    for id in ids {
        let Some(gb) = b.get_id(id) else { continue };
        let sum = match a.get_id(id) {
            Some(ga) => (ga + gb)?,
            None => gb.clone(),
        };
        a.insert_id(id, sum);
    }
    Ok(a)
}

/// One training step's loss and gradients as `forward` says, from `loss_and_grads(q12)`: the
/// batch's loss and its backward, the scored query at Q12 when `q12`.
fn train_step(
    forward: TrainForward,
    seed: u64,
    step: usize,
    loss_and_grads: impl Fn(bool) -> Result<(f64, GradStore)>,
) -> Result<(f64, GradStore)> {
    match forward {
        TrainForward::Plain => loss_and_grads(false),
        TrainForward::D11 => hs::with_d11_training(seed, step, || loss_and_grads(true)),
        TrainForward::Dual => {
            // claude-131: each forward's backward runs under its own gate mode (the icosian
            // gate's backward reads the hook), and the gradients are summed 1:1.
            let (plain, plain_grads) = loss_and_grads(false)?;
            let (d11, d11_grads) = hs::with_d11_training(seed, step, || loss_and_grads(true))?;
            Ok((plain + d11, sum_grads(plain_grads, &d11_grads)?))
        }
    }
}

/// [`run_fbs_arm`], with its training steps' loss as `forward` says. In the D11 mode the loss is
/// the D11 forward's, its scored query at Q12, through `hamilton_sheet::with_d11_training`'s
/// bridges, with the step's backward inside the same mode (claude-130). In the dual mode it is
/// that plus the plain loss, each with its own backward (claude-131). The evaluations are
/// unchanged.
#[allow(clippy::too_many_arguments)]
pub fn run_fbs_arm_with(
    arm: FbsArm,
    forward: TrainForward,
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
        let loss_and_grads = |q12: bool| -> Result<(f64, GradStore)> {
            let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH, q12)?;
            let value = f64::from(loss.to_scalar::<f32>()?);
            if !value.is_finite() {
                return Err(invalid(format!(
                    "I-Finite-CUDA: non-finite training loss at step {step} in {}",
                    arm.name()
                )));
            }
            Ok((value, loss.backward()?))
        };
        let (value, grads) = train_step(forward, seed, step, loss_and_grads)?;
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
        // The sheet the trained models answer to, as their training receipt names it (claude-131).
        "training_sheet": receipt["sheet"],
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

/// Per row, the largest distance in Q12 codes between two sets of scored queries (`code / 4096`,
/// rows × `width`).
fn q12_drift(a: &[f32], b: &[f32], width: usize) -> Result<Vec<u32>> {
    if width == 0 || a.len() != b.len() || !a.len().is_multiple_of(width) {
        return Err(invalid("the two query sets are not the same whole rows"));
    }
    let code = |v: f32| (v * hs::Q12_SCALE).round() as i32;
    Ok(a.chunks(width)
        .zip(b.chunks(width))
        .map(|(ra, rb)| {
            ra.iter()
                .zip(rb)
                .map(|(&x, &y)| code(x).abs_diff(code(y)))
                .max()
                .unwrap_or(0)
        })
        .collect())
}

/// How far two sets of queries drift, over all rows and over the rows `selected`.
fn drift_summary(drift: &[u32], selected: impl Fn(usize) -> bool) -> serde_json::Value {
    let summary = |rows: Vec<u32>| {
        serde_json::json!({
            "rows": rows.len(),
            "max_codes": rows.iter().copied().max().unwrap_or(0),
            "rows_exact": rows.iter().filter(|&&d| d == 0).count(),
            "rows_within_1": rows.iter().filter(|&&d| d <= 1).count(),
            "rows_within_2": rows.iter().filter(|&&d| d <= 2).count(),
        })
    };
    serde_json::json!({
        "all": summary(drift.to_vec()),
        "selected": summary(
            drift.iter().enumerate().filter(|(i, _)| selected(*i)).map(|(_, &d)| d).collect()
        ),
    })
}

/// The smallest size, in Q12 codes on both sides, at which an opposite sign counts as a sign flip
/// in [`drift_readings`] (1/64 of a unit): below it, rounding alone can cross zero.
const SIGN_FLIP_MIN_CODES: i64 = 64;

/// The drift readings the claude-132 code review asked for (claude-133), in Q12 codes, of the
/// queries `a` against the reference `r` (rows × `width`):
/// - the median, mean, 99th-percentile and largest absolute difference over every entry (by
///   nearest rank: the upper middle value when the count is even);
/// - the three columns that hold the most per-row maxima (a row whose largest difference is in
///   two columns counts toward the lower one), and the rows with no difference at all, which
///   count toward no column;
/// - the largest per-channel mean signed difference in size, and its channel (the higher one on a
///   tie);
/// - the entries saturated at the i16 ends on each side;
/// - the entries of opposite sign where both sides are at least [`SIGN_FLIP_MIN_CODES`] in size,
///   so that a sign flip shows;
/// - the per-row scale Σ|a| / Σ|r| (5th percentile, median, 95th percentile), so that a scale or
///   exponent error shows. It is an L1 ratio: no products.
fn drift_readings(a: &[f32], r: &[f32], width: usize) -> Result<serde_json::Value> {
    if width == 0 || a.len() != r.len() || a.is_empty() || !a.len().is_multiple_of(width) {
        return Err(invalid("the two query sets are not the same whole rows"));
    }
    let code = |v: f32| (v * hs::Q12_SCALE).round() as i64;
    let saturated = |c: i64| c >= i64::from(i16::MAX) || c <= i64::from(i16::MIN);
    let rows = a.len() / width;
    let mut abs = Vec::with_capacity(a.len());
    let mut channel_sum = vec![0i64; width];
    let mut max_column = vec![0usize; width];
    let (mut saturated_a, mut saturated_r) = (0usize, 0usize);
    let mut sign_flips = 0usize;
    let mut rows_no_gap = 0usize;
    let mut scales = Vec::with_capacity(rows);
    for (ra, rr) in a.chunks(width).zip(r.chunks(width)) {
        let (mut best, mut best_col) = (0u64, 0usize);
        let (mut l1_a, mut l1_r) = (0u64, 0u64);
        for (j, (&x, &y)) in ra.iter().zip(rr).enumerate() {
            let (cx, cy) = (code(x), code(y));
            let d = cx - cy;
            abs.push(d.unsigned_abs());
            channel_sum[j] += d;
            if d.unsigned_abs() > best {
                (best, best_col) = (d.unsigned_abs(), j);
            }
            saturated_a += usize::from(saturated(cx));
            saturated_r += usize::from(saturated(cy));
            sign_flips += usize::from(
                cx.signum() * cy.signum() < 0 && cx.abs().min(cy.abs()) >= SIGN_FLIP_MIN_CODES,
            );
            l1_a += cx.unsigned_abs();
            l1_r += cy.unsigned_abs();
        }
        // A row with no difference has no column of its largest one (claude-133 code review).
        if best > 0 {
            max_column[best_col] += 1;
        } else {
            rows_no_gap += 1;
        }
        if l1_r > 0 {
            scales.push(l1_a as f64 / l1_r as f64);
        }
    }
    abs.sort_unstable();
    scales.sort_by(f64::total_cmp);
    let at = |len: usize, q: f64| ((len.saturating_sub(1)) as f64 * q).round() as usize;
    let scale = |q: f64| scales.get(at(scales.len(), q)).copied();
    let mut columns: Vec<(usize, usize)> = max_column.iter().copied().enumerate().collect();
    columns.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    let (worst_channel, worst_mean) = channel_sum
        .iter()
        .enumerate()
        .map(|(j, &s)| (j, s as f64 / rows as f64))
        .max_by(|x, y| x.1.abs().total_cmp(&y.1.abs()))
        .unwrap_or((0, 0.0));
    Ok(serde_json::json!({
        "entries": abs.len(),
        "median_abs_codes": abs[at(abs.len(), 0.5)],
        "mean_abs_codes": abs.iter().sum::<u64>() as f64 / abs.len() as f64,
        "p99_abs_codes": abs[at(abs.len(), 0.99)],
        "max_abs_codes": abs[abs.len() - 1],
        "top_max_columns": columns.iter().take(3).map(|&(j, n)| [j, n]).collect::<Vec<_>>(),
        "rows_no_gap": rows_no_gap,
        "max_abs_channel_mean_codes": worst_mean,
        "max_abs_channel_mean_channel": worst_channel,
        "saturated_entries": {"queries": saturated_a, "reference": saturated_r},
        "sign_flip_entries": sign_flips,
        "row_l1_scale": {"p05": scale(0.05), "median": scale(0.5), "p95": scale(0.95)},
    }))
}

fn rows_differ(a: &[u32], b: &[u32]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// The share of rows on which D and A may differ for a faithful artifact, in percent: three times
/// claude-127's measured integer residual at 6 layers (4 to 5 of 512 tokens, about 1%).
const RESIDUAL_FAITHFUL_PERCENT: usize = 3;
/// The share beyond which D and A differ as a different forward does, in percent: claude-127's
/// norm-bias defect changed 91 of 512 tokens (18%).
const RESIDUAL_DEPARTURE_PERCENT: usize = 10;

/// The residual check's classification of one model, as its sheet fixes it (claude-132):
/// - `faithful`: every instrument green, H and A identical on every row, D and A differing on at
///   most [`RESIDUAL_FAITHFUL_PERCENT`] of the rows, and D against A within the paired test;
/// - `departure`: H and A differ on any row, D and A differ on more than
///   [`RESIDUAL_DEPARTURE_PERCENT`] of the rows, or D against A is outside the paired test;
/// - `indeterminate`: otherwise, an instrument investigation;
/// - `not counted`: an instrument check failed.
fn classify_residual(
    instruments_ok: bool,
    head_differ: usize,
    d_a_differ: usize,
    rows: usize,
    paired_within: bool,
) -> &'static str {
    if !instruments_ok || rows == 0 {
        "not counted"
    } else if head_differ > 0
        || d_a_differ * 100 > RESIDUAL_DEPARTURE_PERCENT * rows
        || !paired_within
    {
        "departure"
    } else if d_a_differ * 100 <= RESIDUAL_FAITHFUL_PERCENT * rows {
        "faithful"
    } else {
        "indeterminate"
    }
}

/// The residual check of claude-131's pivot note (claude-132), for one saved model and its
/// artifact's stored, verified evaluator run, on the GPU.
/// - **F**, the plain-float panel, which must reproduce `stored_preds` (`I-Save-Load`);
/// - **E**, the export-mode panel, on the tables as trained;
/// - **D**, the D11 panel, with the tables rounded as exported;
/// - **H**, the writer head at Q12 on the artifact's own queries;
/// - **A**, the artifact's tokens at the scored offsets.
///
/// [`classify_residual`] decides whether the artifact computes its D11 forward: H against A for
/// the head, D against A row by row and under the paired test for the whole. The Q12 drift
/// between D's and A's queries says where a departure sits.
#[allow(clippy::too_many_arguments)]
pub fn run_d11_residual(
    data_root: &Path,
    model_dir: &Path,
    evaluator: &Path,
    windows_sha256: &str,
    artifact_sha256: &str,
    model_sha256_pin: &str,
    stored_preds: &Path,
    stored_d11_preds: Option<&Path>,
    seed: u64,
    label: &str,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::round_tables_as_exported;

    let stored_d11: Option<Vec<u32>> = match stored_d11_preds {
        Some(path) => Some(serde_json::from_slice(&fs::read(path)?)?),
        None => None,
    };
    let (dev_windows, dev_docs) = load_dev(data_root)?;
    let targets: Vec<u32> = dev_windows
        .iter()
        .flat_map(|w| w.scored_targets.iter().copied())
        .collect();
    let stored: Vec<u32> = serde_json::from_slice(&fs::read(stored_preds)?)?;
    let loaded = StackModel::load(model_dir, device)?;
    let model_sha256 = sha256_bytes(&fs::read(model_dir.join("model.safetensors"))?);
    let panel = |export_mode: bool| {
        hs::evaluate_dev_panel(
            &loaded,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            STEPS,
            export_mode,
            false,
        )
    };
    let (f_metrics, f_preds) = panel(false)?;
    let (e_metrics, e_preds) = panel(true)?;
    hs::clear_active_arm();
    let i_save_load = f_preds == stored;

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
    let i_inputs = run.sequences_sha256 == windows_sha256
        && run.artifact_sha256 == artifact_sha256
        && model_sha256 == model_sha256_pin;
    let i_verify = run.verify.as_array().is_some_and(|checks| {
        checks.len() == 2
            && checks.iter().all(|c| c["equal"] == serde_json::json!(true))
            && checks
                .iter()
                .all(|c| c["positions"].as_u64() == Some((run.sequences * run.len) as u64))
    });
    let (a_preds, a_queries) =
        artifact_rows(&dev_windows, &run.tokens, &run.queries, run.len, width)?;
    let i_dev = a_preds.len() == hs::EXPECTED_DEV_ELIGIBLE_TARGETS;

    round_tables_as_exported(&loaded)?;
    let (d_metrics, d_preds) = hs::evaluate_dev_panel_d11(&loaded, &dev_windows)?;
    let (_, d_again) = hs::evaluate_dev_panel_d11(&loaded, &dev_windows)?;
    let d_queries = hs::with_d11_mode(|| hs::dev_panel_q12_queries(&loaded, &dev_windows))?;
    // I-Head: the head path reproduces the D11 panel's own predictions from its own queries, so H
    // is the panel's head.
    let i_head = hs::write_q12_rows(&loaded, &d_queries)? == d_preds;
    let h_preds = hs::write_q12_rows(&loaded, &a_queries)?;
    // The Q16 control (claude-133): the D11 forward with its residual stream on the integer
    // session's 2^-16 grid.
    let (d16_metrics, d16_preds) =
        hs::with_q16_residual(|| hs::evaluate_dev_panel_d11(&loaded, &dev_windows))?;
    let d16_queries = hs::with_q16_residual(|| {
        hs::with_d11_mode(|| hs::dev_panel_q12_queries(&loaded, &dev_windows))
    })?;
    let i_head_q16 = hs::write_q12_rows(&loaded, &d16_queries)? == d16_preds;
    if [&d_preds, &d16_preds, &h_preds, &f_preds, &e_preds]
        .iter()
        .any(|p| p.len() != a_preds.len())
    {
        return Err(invalid("the panels do not cover the artifact's rows"));
    }
    let drift = q12_drift(&d_queries, &a_queries, width)?;
    let drift_q16 = q12_drift(&d16_queries, &a_queries, width)?;

    let pair = |name_a: &str, a: &[u32], name_b: &str, b: &[u32]| {
        let p = hs::compute_paired_discordance(name_a, a, name_b, b, &targets, seed);
        serde_json::json!({
            "b": p.a_hit_b_miss_c,
            "c": p.a_miss_b_hit_b,
            "net": p.net_advantage_rows,
            "f_i": f_i(p.a_hit_b_miss_c, p.a_miss_b_hit_b),
            "within": p.net_advantage_rows.unsigned_abs() <= f_i(p.a_hit_b_miss_c, p.a_miss_b_hit_b),
            "rows_differ": rows_differ(a, b),
        })
    };
    let top1 = |preds: &[u32]| preds.iter().zip(&targets).filter(|(p, t)| p == t).count();
    let d_vs_a = pair("d11", &d_preds, "artifact", &a_preds);
    // I-D-Stored: where the model's D11 panel was stored when it was trained (Stage 4b), the
    // recomputed panel is that one, row for row.
    let i_d_stored = stored_d11.as_ref().map(|s| *s == d_preds);
    let instruments_ok = i_save_load
        && i_inputs
        && i_verify
        && i_dev
        && d_preds == d_again
        && i_head
        && i_d_stored.unwrap_or(true);
    // The Q16 control's own head check gates only its own classification (claude-133 code review).
    let instruments_q16_ok = instruments_ok && i_head_q16;
    let classification = classify_residual(
        instruments_ok,
        rows_differ(&h_preds, &a_preds),
        rows_differ(&d_preds, &a_preds),
        a_preds.len(),
        d_vs_a["within"] == serde_json::json!(true),
    );
    // The same rule with the Q16 control as the reference (claude-133).
    let d16_vs_a = pair("d11_q16", &d16_preds, "artifact", &a_preds);
    let classification_q16 = classify_residual(
        instruments_q16_ok,
        rows_differ(&h_preds, &a_preds),
        rows_differ(&d16_preds, &a_preds),
        a_preds.len(),
        d16_vs_a["within"] == serde_json::json!(true),
    );
    Ok(serde_json::json!({
        "sheet": "docs/harness/prep/d11-exporter-float-trunk-investigation-2026-10-26.md",
        "residual_sheet": "docs/harness/prep/d11-exporter-float-trunk-residual-2026-10-26.md",
        "label": label,
        "seed": seed,
        "arm": D11_EXPORT_ARM.name(),
        "dev_documents": dev_docs,
        "dev_windows": dev_windows.len(),
        "model_safetensors_sha256": model_sha256,
        "artifact_sha256": run.artifact_sha256,
        "windows_sha256": run.sequences_sha256,
        "evaluator_build": run.build,
        "top1": {
            "float": f_metrics.top1_hits,
            "export_mode": e_metrics.top1_hits,
            "d11": d_metrics.top1_hits,
            "d11_q16": d16_metrics.top1_hits,
            "head_on_artifact_queries": top1(&h_preds),
            "artifact": top1(&a_preds),
        },
        "d11_q12_oracle_mismatches": d_metrics.q12_i32_oracle_mismatches,
        "pairs": {
            "d11_vs_artifact": d_vs_a,
            "head_vs_artifact": pair("head", &h_preds, "artifact", &a_preds),
            "d11_vs_head": pair("d11", &d_preds, "head", &h_preds),
            "export_mode_vs_artifact": pair("export_mode", &e_preds, "artifact", &a_preds),
            "export_mode_vs_d11": pair("export_mode", &e_preds, "d11", &d_preds),
            "float_vs_d11": pair("float", &f_preds, "d11", &d_preds),
            "float_vs_artifact": pair("float", &f_preds, "artifact", &a_preds),
            "d11_q16_vs_artifact": d16_vs_a,
            "d11_vs_d11_q16": pair("d11", &d_preds, "d11_q16", &d16_preds),
        },
        "q12_drift_d11_vs_artifact": drift_summary(&drift, |i| d_preds[i] != a_preds[i]),
        "q12_drift_d11_q16_vs_artifact": drift_summary(&drift_q16, |i| d16_preds[i] != a_preds[i]),
        "drift_readings": {
            "artifact_vs_d11": drift_readings(&a_queries, &d_queries, width)?,
            "artifact_vs_d11_q16": drift_readings(&a_queries, &d16_queries, width)?,
            "d11_q16_vs_d11": drift_readings(&d16_queries, &d_queries, width)?,
        },
        "d11_q16_q12_oracle_mismatches": d16_metrics.q12_i32_oracle_mismatches,
        "i_head_q16": i_head_q16,
        "classification_q16": classification_q16,
        "i_save_load": i_save_load,
        "i_inputs": i_inputs,
        "i_verify_all": i_verify,
        "i_dev_denominator": i_dev,
        "i_d_determinism": d_preds == d_again,
        "i_head": i_head,
        "i_d_stored": i_d_stored,
        "instruments_pass": instruments_ok,
        "instruments_pass_q16": instruments_q16_ok,
        "rule": {
            "faithful_percent": RESIDUAL_FAITHFUL_PERCENT,
            "departure_percent": RESIDUAL_DEPARTURE_PERCENT,
        },
        "classification": classification,
    }))
}

/// The shape of the snap trace (claude-134): per layer, `lanes` transition roots, `lanes` output
/// gate roots, then `ff_blocks` feed-forward gate roots, for each of `windows × len` positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SnapShape {
    windows: usize,
    len: usize,
    layers: usize,
    lanes: usize,
    ff_blocks: usize,
}

impl SnapShape {
    fn layer_bytes(self) -> usize {
        2 * self.lanes + self.ff_blocks
    }
    fn stride(self) -> usize {
        self.layers * self.layer_bytes()
    }
    fn blocks(self, site: usize) -> usize {
        if site == 2 {
            self.ff_blocks
        } else {
            self.lanes
        }
    }
    /// The trace offset of a snap call's first block at `row` (`window × len + position`).
    fn offset(self, row: usize, layer: usize, site: usize) -> usize {
        row * self.stride() + layer * self.layer_bytes() + site * self.lanes
    }
}

/// A snap disagreement's class (claude-134), by its zero sides. A zero block (all four raw
/// channels zero, 255 in a capture or a trace) is snapped by the float forward to the identity
/// (root 1) and by the integer path to root 0, which is −1, so a zero block on either side can
/// apply a different rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapClass {
    /// Both blocks non-zero, with different roots.
    Root,
    /// Both blocks zero: the identity against −1.
    ZeroBoth,
    /// The float block zero (the identity), the integer block non-zero.
    ZeroFloat,
    /// The integer block zero (−1), the float block non-zero.
    ZeroInteger,
}

impl SnapClass {
    const NAMES: [&'static str; 4] = ["root", "zero_both", "zero_float_only", "zero_integer_only"];

    /// The class of a float and an integer snap byte, or `None` when both apply the same root.
    fn of(float: u8, integer: u8) -> Option<Self> {
        let (float_zero, integer_zero) = (float == u8::MAX, integer == u8::MAX);
        let applied_float = if float_zero { 1 } else { float };
        let applied_integer = if integer_zero { 0 } else { integer };
        (applied_float != applied_integer).then_some(match (float_zero, integer_zero) {
            (false, false) => Self::Root,
            (true, true) => Self::ZeroBoth,
            (true, false) => Self::ZeroFloat,
            (false, true) => Self::ZeroInteger,
        })
    }
}

/// A window's first root disagreement in causal order (position, then layer, then the
/// transition, the output gate and the feed-forward gate), and D's relative margins at that
/// call's root disagreements: the window's primaries. Every later root disagreement in the window
/// comes after them.
#[derive(Clone, Debug, PartialEq)]
struct FirstFlip {
    position: usize,
    layer: usize,
    site: usize,
    primaries: Vec<f32>,
}

/// The D11 forward's snap choices against the artifact's (claude-134).
#[derive(Clone, Debug, Default, PartialEq)]
struct SnapComparison {
    /// Per layer, the disagreements at the transition, the output gate and the feed-forward gate.
    per_layer_site: Vec<[usize; 3]>,
    /// The disagreements by class, in [`SnapClass`] order.
    per_class: [usize; 4],
    /// Transition disagreements at position 0, where the state the root rotates is still zero on
    /// both sides: inert, so counted here and nowhere else.
    inert: usize,
    /// Every snap compared.
    sites: usize,
    /// Per position (`window × len + position`): a root disagreement there.
    position_root: Vec<bool>,
    /// Per position: a zero-block disagreement there.
    position_zero: Vec<bool>,
    /// D's relative margin at every root disagreement.
    root_margins: Vec<f32>,
    /// Per window: its first flip, and the position of its first zero-block disagreement.
    first: Vec<Option<FirstFlip>>,
    first_zero: Vec<Option<usize>>,
}

/// Compares the D11 forward's snap captures (three per layer, in call order, over `windows × len`
/// rows) with the artifact's trace (`shape.stride()` bytes per position).
fn compare_snaps(
    captures: &[hs::SnapCapture],
    trace: &[u8],
    shape: SnapShape,
) -> Result<SnapComparison> {
    let rows = shape.windows * shape.len;
    if captures.len() != 3 * shape.layers || trace.len() != rows * shape.stride() {
        return Err(invalid(
            "the captures or the trace do not fit the trunk's shape",
        ));
    }
    // Every snap byte names one of the 120 roots, or is 255 for a zero block.
    let named = |b: &u8| *b < 120 || *b == u8::MAX;
    if !trace.iter().all(named) || !captures.iter().flat_map(|c| &c.roots).all(named) {
        return Err(invalid("a snap byte names no root"));
    }
    let mut out = SnapComparison {
        per_layer_site: vec![[0; 3]; shape.layers],
        position_root: vec![false; rows],
        position_zero: vec![false; rows],
        first_zero: vec![None; shape.windows],
        ..SnapComparison::default()
    };
    // A snap call's causal key within its window: position, then layer, then site.
    let key =
        |position: usize, layer: usize, site: usize| (position * shape.layers + layer) * 3 + site;
    let mut first_key = vec![usize::MAX; shape.windows];
    for layer in 0..shape.layers {
        for site in 0..3 {
            let capture = &captures[layer * 3 + site];
            let blocks = shape.blocks(site);
            if capture.roots.len() != rows * blocks || capture.margins.len() != rows * blocks {
                return Err(invalid(format!(
                    "snap call {} does not cover the rows",
                    layer * 3 + site
                )));
            }
            for row in 0..rows {
                let base = shape.offset(row, layer, site);
                for block in 0..blocks {
                    out.sites += 1;
                    let at = row * blocks + block;
                    let Some(class) = SnapClass::of(capture.roots[at], trace[base + block]) else {
                        continue;
                    };
                    let (window, position) = (row / shape.len, row % shape.len);
                    if site == 0 && position == 0 {
                        out.inert += 1;
                        continue;
                    }
                    out.per_layer_site[layer][site] += 1;
                    out.per_class[class as usize] += 1;
                    if class == SnapClass::Root {
                        out.position_root[row] = true;
                        out.root_margins.push(capture.margins[at]);
                        first_key[window] = first_key[window].min(key(position, layer, site));
                    } else {
                        out.position_zero[row] = true;
                        let first = &mut out.first_zero[window];
                        *first = Some(first.map_or(position, |p| p.min(position)));
                    }
                }
            }
        }
    }
    out.first = first_key
        .iter()
        .enumerate()
        .map(|(window, &k)| {
            (k != usize::MAX).then(|| {
                let (site, layer, position) = (k % 3, k / 3 % shape.layers, k / 3 / shape.layers);
                let capture = &captures[layer * 3 + site];
                let blocks = shape.blocks(site);
                let row = window * shape.len + position;
                let base = shape.offset(row, layer, site);
                let primaries = (0..blocks)
                    .filter(|&block| {
                        SnapClass::of(capture.roots[row * blocks + block], trace[base + block])
                            == Some(SnapClass::Root)
                    })
                    .map(|block| capture.margins[row * blocks + block])
                    .collect();
                FirstFlip {
                    position,
                    layer,
                    site,
                    primaries,
                }
            })
        })
        .collect();
    Ok(out)
}

/// The fewest snaps that must agree at every (layer, site), in percent, for the trace's layout to
/// stand (`I-Layout`, claude-134). D and A agree on nearly every token, so nearly every snap; a
/// site read at the wrong offset would agree on about one root in 120.
const LAYOUT_MIN_AGREE_PERCENT: usize = 50;

/// `I-Layout`: each (layer, site) agrees on at least [`LAYOUT_MIN_AGREE_PERCENT`] of its snaps.
fn layout_agrees(c: &SnapComparison, shape: SnapShape) -> bool {
    let rows = shape.windows * shape.len;
    c.per_layer_site.iter().all(|sites| {
        (0..3).all(|site| {
            let snaps = rows * shape.blocks(site);
            100 * (snaps - sites[site]) >= LAYOUT_MIN_AGREE_PERCENT * snaps
        })
    })
}

/// The D panel's snap captures, one per call over each chunk of [`hs::SHEET_EVAL_BATCH`] windows
/// in panel order, joined into one per (layer, site) over all the windows.
fn join_chunk_captures(
    captures: Vec<hs::SnapCapture>,
    windows: usize,
    layers: usize,
) -> Result<Vec<hs::SnapCapture>> {
    let calls = 3 * layers;
    if captures.len() != windows.div_ceil(hs::SHEET_EVAL_BATCH) * calls {
        return Err(invalid("the D panel's snap calls do not cover its chunks"));
    }
    let mut joined = vec![hs::SnapCapture::default(); calls];
    for (i, capture) in captures.into_iter().enumerate() {
        joined[i % calls].roots.extend(capture.roots);
        joined[i % calls].margins.extend(capture.margins);
    }
    Ok(joined)
}

/// One mask's placement of the scored rows (claude-134): the rows where D and A differ with a
/// disagreement of the mask's kind in their window at or before their position; the same over
/// the rows where D and A agree (the base rate); and the windows whose first disagreement of that
/// kind is at or before their first row where D and A differ.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MaskReadings {
    differ_preceded: usize,
    agree_preceded: usize,
    lead_windows: usize,
}

/// What the snap-trace bar reads (claude-134), over one model's windows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SnapReadings {
    rows_differ: usize,
    rows_agree: usize,
    /// Windows with a row where D and A differ.
    differ_windows: usize,
    /// The root disagreements' placement, and the zero blocks'.
    root: MaskReadings,
    zero: MaskReadings,
    /// Over those windows' primaries: their count, those at an exact tie (D's margin 0), and
    /// D's upper median margin at the others.
    root_primaries: usize,
    exact_ties: usize,
    median_margin: Option<f32>,
}

/// The snap-trace bar's thresholds (the sheet's §3).
const SNAP_CARRIED_MIN_PERCENT: f64 = 50.0;
const NEAR_TIE_MIN_PERCENT: f64 = 90.0;
const MIN_LIFT_POINTS: f64 = 20.0;
const NEAR_TIE_MAX_MARGIN: f32 = 1e-3;
const DECISIVE_MIN_MARGIN: f32 = 1e-2;

/// One model's snap-trace outcome, the first that applies (the sheet's §3 precedence).
fn classify_snap_trace(r: &SnapReadings) -> &'static str {
    let percent = |part: usize, whole: usize| {
        if whole == 0 {
            0.0
        } else {
            100.0 * part as f64 / whole as f64
        }
    };
    if r.rows_differ == 0 {
        return "no-rows-differ";
    }
    // Per mask: the share preceded, its base rate, and the share of windows it leads.
    let place = |m: &MaskReadings| {
        (
            percent(m.differ_preceded, r.rows_differ),
            percent(m.agree_preceded, r.rows_agree),
            percent(m.lead_windows, r.differ_windows),
        )
    };
    let (zero_p, zero_b, zero_l) = place(&r.zero);
    if zero_p >= SNAP_CARRIED_MIN_PERCENT
        && zero_l >= SNAP_CARRIED_MIN_PERCENT
        && zero_p - zero_b >= MIN_LIFT_POINTS
    {
        return "zero-convention";
    }
    let (p, b, l) = place(&r.root);
    if p < SNAP_CARRIED_MIN_PERCENT || l < SNAP_CARRIED_MIN_PERCENT {
        return "not-snap-carried";
    }
    if r.exact_ties > 0 && 2 * r.exact_ties >= r.root_primaries {
        return "tie-break";
    }
    match r.median_margin {
        Some(m) if m >= DECISIVE_MIN_MARGIN => "localise-upstream",
        Some(m)
            if m < NEAR_TIE_MAX_MARGIN
                && p >= NEAR_TIE_MIN_PERCENT
                && l >= NEAR_TIE_MIN_PERCENT
                && p - b >= MIN_LIFT_POINTS =>
        {
            "snap-flip-near-tie"
        }
        _ => "indeterminate",
    }
}

/// The primaries of `windows`' first flips: their count, those at an exact tie, and D's margins
/// at the others. Returns the readings' primary fields and their record.
fn primary_summary(
    first: &[Option<FirstFlip>],
    windows: impl IntoIterator<Item = usize>,
) -> ((usize, usize, Option<f32>), serde_json::Value) {
    let (mut count, mut exact_ties, mut windows_with) = (0usize, 0usize, 0usize);
    let mut margins = Vec::new();
    for flip in windows.into_iter().filter_map(|w| first[w].as_ref()) {
        windows_with += 1;
        for &m in &flip.primaries {
            count += 1;
            if m > 0.0 {
                margins.push(m);
            } else {
                exact_ties += 1;
            }
        }
    }
    margins.sort_by(f32::total_cmp);
    let median = margins.get(margins.len() / 2).copied();
    let below = |t: f32| margins.iter().filter(|&&m| m < t).count();
    let record = serde_json::json!({
        "windows_with_a_flip": windows_with,
        "primaries": count,
        "exact_ties": exact_ties,
        "margins_off_a_tie": {
            "count": margins.len(),
            "upper_median": median,
            "max": margins.last(),
            "below_1e-6": below(1e-6),
            "below_1e-4": below(1e-4),
            "below_1e-3": below(1e-3),
            "below_1e-2": below(1e-2),
        },
    });
    ((count, exact_ties, median), record)
}

/// The trace file of `d11-fbs-eval trace` (claude-134).
#[derive(Deserialize)]
struct EvaluatorTrace {
    artifact_sha256: String,
    sequences_sha256: String,
    sequences: usize,
    len: usize,
    width: usize,
    mlp: usize,
    layers: usize,
    trace_stride: usize,
    tokens: Vec<u32>,
    outputs_host_equal: bool,
    trace_host_equal: bool,
    trace_hex: String,
    build: serde_json::Value,
}

/// Snap-flip localisation (claude-134), on the GPU: over the first `windows_count` DEV windows,
/// the D11 forward's snap choices against the artifact's trace, and where the rows that D and A
/// disagree on sit relative to the snap disagreements.
#[allow(clippy::too_many_arguments)]
pub fn run_d11_snap_trace(
    data_root: &Path,
    model_dir: &Path,
    trace_path: &Path,
    windows_count: usize,
    windows_sha256: &str,
    artifact_sha256: &str,
    model_sha256_pin: &str,
    stored_run: &Path,
    label: &str,
    seed: u64,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::round_tables_as_exported;

    let (dev_windows, dev_docs) = load_dev(data_root)?;
    let windows = dev_windows
        .get(..windows_count)
        .ok_or_else(|| invalid("fewer DEV windows than asked for"))?;
    let trace: EvaluatorTrace = serde_json::from_slice(&fs::read(trace_path)?)?;
    // I-Old-Run: on these windows the traced build's tokens are the stored, qualified run's
    // (claude-128's evaluator), so the trace describes the artifact that was scored.
    let old: EvaluatorRun = serde_json::from_slice(&fs::read(stored_run)?)?;
    let i_old_run = old.artifact_sha256 == trace.artifact_sha256
        && old.len == SHEET_CONTEXT
        && old.tokens.get(..windows_count * SHEET_CONTEXT) == Some(&trace.tokens[..]);
    let loaded = StackModel::load(model_dir, device)?;
    let model_sha256 = sha256_bytes(&fs::read(model_dir.join("model.safetensors"))?);
    let config = &loaded.config;
    let shape = SnapShape {
        windows: windows_count,
        len: SHEET_CONTEXT,
        layers: config.layers(),
        lanes: config.width / 4,
        ff_blocks: config.mlp_hidden / 4,
    };
    let trace_bytes = hex_bytes(&trace.trace_hex)?;
    // I-Inputs and I-Trace: the trace is the pinned artifact's, on these windows, with the GPU's
    // outputs and trace equal to the host twin's, and in this trunk's shape.
    let i_inputs = trace.artifact_sha256 == artifact_sha256
        && trace.sequences_sha256 == windows_sha256
        && model_sha256 == model_sha256_pin;
    let i_trace = trace.outputs_host_equal
        && trace.trace_host_equal
        && trace.sequences == windows_count
        && trace.len == SHEET_CONTEXT
        && trace.width == config.width
        && trace.mlp == config.mlp_hidden
        && trace.layers == shape.layers
        && trace.trace_stride == shape.stride()
        && trace.tokens.len() == windows_count * SHEET_CONTEXT;
    if !i_trace {
        return Err(invalid(
            "the trace does not fit these windows or this trunk (I-Trace)",
        ));
    }

    round_tables_as_exported(&loaded)?;
    // D's tokens and D's snap choices come from one forward: the D11 panel, captured.
    let ((_, d_preds), chunk_captures) =
        hs::with_snap_capture(|| hs::evaluate_dev_panel_d11(&loaded, windows))?;
    hs::clear_active_arm();
    let captures = join_chunk_captures(chunk_captures, windows_count, shape.layers)?;
    let compared = compare_snaps(&captures, &trace_bytes, shape)?;

    // Each scored row, D's token against A's, placed relative to its window's root and zero-block
    // disagreements. `score_dev_panel` reads each window's scored offsets in this same order.
    #[derive(Default)]
    struct Placed {
        rows: usize,
        preceded_by_root: usize,
        preceded_by_zero: usize,
        root_at_position: usize,
    }
    let (mut differ, mut agree) = (Placed::default(), Placed::default());
    let mut first_differ: Vec<Option<usize>> = vec![None; windows_count];
    let mut a_preds = Vec::new();
    for (w, window) in windows.iter().enumerate() {
        let start = w * SHEET_CONTEXT;
        for &offset in &window.scored_offsets {
            let at = start + offset;
            let a = trace.tokens[at];
            let d = *d_preds
                .get(a_preds.len())
                .ok_or_else(|| invalid("the D11 panel does not cover the windows"))?;
            a_preds.push(a);
            let placed = if d == a {
                &mut agree
            } else {
                first_differ[w] = Some(first_differ[w].map_or(offset, |p| p.min(offset)));
                &mut differ
            };
            placed.rows += 1;
            placed.preceded_by_root +=
                usize::from(compared.position_root[start..=at].contains(&true));
            placed.preceded_by_zero +=
                usize::from(compared.position_zero[start..=at].contains(&true));
            placed.root_at_position += usize::from(compared.position_root[at]);
        }
    }
    if a_preds.len() != d_preds.len() {
        return Err(invalid(
            "the D11 panel's rows are not the windows' scored rows",
        ));
    }
    let differ_windows: Vec<usize> = (0..windows_count)
        .filter(|&w| first_differ[w].is_some())
        .collect();
    let leads = |first: &dyn Fn(usize) -> Option<usize>| {
        differ_windows
            .iter()
            .filter(|&&w| matches!((first(w), first_differ[w]), (Some(f), Some(r)) if f <= r))
            .count()
    };
    let root_lead = leads(&|w| compared.first[w].as_ref().map(|f| f.position));
    let zero_lead = leads(&|w| compared.first_zero[w]);
    let ((root_primaries, exact_ties, median_margin), primaries_differ) =
        primary_summary(&compared.first, differ_windows.iter().copied());
    let (_, primaries_all) = primary_summary(&compared.first, 0..windows_count);
    let readings = SnapReadings {
        rows_differ: differ.rows,
        rows_agree: agree.rows,
        differ_windows: differ_windows.len(),
        root: MaskReadings {
            differ_preceded: differ.preceded_by_root,
            agree_preceded: agree.preceded_by_root,
            lead_windows: root_lead,
        },
        zero: MaskReadings {
            differ_preceded: differ.preceded_by_zero,
            agree_preceded: agree.preceded_by_zero,
            lead_windows: zero_lead,
        },
        root_primaries,
        exact_ties,
        median_margin,
    };
    // I-Finite on the margins: a NaN would otherwise read as an exact tie.
    let i_finite_margins = compared.root_margins.iter().all(|m| m.is_finite());
    let i_layout = layout_agrees(&compared, shape);
    let instruments_pass = i_inputs && i_trace && i_old_run && i_finite_margins && i_layout;
    let mut root_margins = compared.root_margins.clone();
    root_margins.sort_by(f32::total_cmp);
    let below = |t: f32| root_margins.iter().filter(|&&m| m < t).count();
    let first_flips: Vec<serde_json::Value> = compared
        .first
        .iter()
        .map(|f| {
            f.as_ref().map_or(serde_json::Value::Null, |f| {
                let site = ["transition", "output_gate", "ff_gate"][f.site];
                serde_json::json!({
                    "position": f.position,
                    "layer": f.layer,
                    "site": site,
                    "primaries": f.primaries.len(),
                })
            })
        })
        .collect();
    Ok(serde_json::json!({
        "sheet": "docs/harness/prep/d11-exporter-float-trunk-snap-trace-2026-10-26.md",
        "label": label,
        "seed": seed,
        "dev_documents": dev_docs,
        "windows": windows_count,
        "model_safetensors_sha256": model_sha256,
        "artifact_sha256": trace.artifact_sha256,
        "windows_sha256": trace.sequences_sha256,
        "evaluator_build": trace.build,
        "scored_rows": a_preds.len(),
        "rows_differ": differ.rows,
        "rows_differ_preceded_by_a_root_disagreement": differ.preceded_by_root,
        "rows_differ_preceded_by_a_zero_block": differ.preceded_by_zero,
        "rows_differ_with_a_root_disagreement_at_their_position": differ.root_at_position,
        "rows_agree": agree.rows,
        "rows_agree_preceded_by_a_root_disagreement": agree.preceded_by_root,
        "rows_agree_preceded_by_a_zero_block": agree.preceded_by_zero,
        "rows_agree_with_a_root_disagreement_at_their_position": agree.root_at_position,
        "windows_with_rows_differ": differ_windows.len(),
        "windows_whose_first_flip_leads": root_lead,
        "windows_whose_first_zero_block_leads": zero_lead,
        "first_flip_per_window": first_flips,
        "first_zero_block_per_window": compared.first_zero,
        "first_row_differ_per_window": first_differ,
        "primaries_windows_with_rows_differ": primaries_differ,
        "primaries_all_windows": primaries_all,
        "snap_sites": compared.sites,
        "snap_inert_transitions_at_position_0": compared.inert,
        "snap_disagreements_per_class": SnapClass::NAMES
            .iter()
            .zip(compared.per_class)
            .collect::<BTreeMap<_, _>>(),
        "snap_disagreements_per_layer_site": compared.per_layer_site,
        "positions_with_a_root_disagreement": compared.position_root.iter().filter(|&&f| f).count(),
        "positions_with_a_zero_block": compared.position_zero.iter().filter(|&&f| f).count(),
        "root_disagreement_margins": {
            "count": root_margins.len(),
            "exact_ties": root_margins.iter().filter(|&&m| m == 0.0).count(),
            "upper_median": root_margins.get(root_margins.len() / 2),
            "max": root_margins.last(),
            "below_1e-6": below(1e-6),
            "below_1e-4": below(1e-4),
            "below_1e-3": below(1e-3),
            "below_1e-2": below(1e-2),
        },
        "i_inputs": i_inputs,
        "i_trace": i_trace,
        "i_old_run": i_old_run,
        "i_finite_margins": i_finite_margins,
        "i_layout": i_layout,
        "instruments_pass": instruments_pass,
        "classification": if instruments_pass {
            classify_snap_trace(&readings)
        } else {
            "unclassified"
        },
    }))
}

/// Lowercase or uppercase hex to bytes.
fn hex_bytes(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return Err(invalid("odd-length hex"));
    }
    let digit = |c: u8| -> Result<u8> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(invalid("a non-hex digit")),
        }
    };
    text.as_bytes()
        .chunks(2)
        .map(|p| Ok((digit(p[0])? << 4) | digit(p[1])?))
        .collect()
}

/// The prep sheet a D11-trained run answers to.
fn d11_train_sheet(forward: TrainForward) -> Result<&'static str> {
    match forward {
        TrainForward::D11 => {
            Ok("docs/harness/prep/d11-exporter-float-trunk-d11-train-2026-10-26.md")
        }
        TrainForward::Dual => {
            Ok("docs/harness/prep/d11-exporter-float-trunk-dual-train-2026-10-26.md")
        }
        TrainForward::Plain => Err(invalid(
            "plain training is Stage 4b's (d11-export-gap), not a D11-trained run",
        )),
    }
}

/// Probe (b) of D11-trained training (claude-130, claude-131): 32 steps of [`D11_EXPORT_ARM`] on
/// seed 1 trained as `forward` says, the DEV panel at steps 0 and 32, and the projected wall time of
/// the full run (two seeds of [`STEPS`] steps, each evaluated at [`EVAL_STEPS`]). In the dual mode
/// the training loss is the sum of the plain and D11-mode losses.
pub fn run_d11_train_probe(
    data_root: &Path,
    forward: TrainForward,
    device: &Device,
) -> Result<serde_json::Value> {
    let sheet = d11_train_sheet(forward)?;
    let (batches, dev_windows, dev_docs) = load_inputs(data_root, 32)?;
    let out = run_fbs_arm_with(
        D11_EXPORT_ARM,
        forward,
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
        "sheet": sheet,
        "train_forward": forward.name(),
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

/// D11-trained training and export (claude-130, claude-131): [`D11_EXPORT_ARM`] trained on both
/// seeds as `forward` says (`run_fbs_arm_with`), each saved model reloaded and checked
/// (`I-Save-Load`), and its snapshot and plain float predictions written under
/// `save_dir/seed-<seed>/` (a directory that must not exist yet) for `fbs-export` and the
/// artifact-gap scorer. The receipt carries Stage 4b's noise band, read from `stage4b`, and each
/// seed's float loss and training trajectory, in the shape `run_d11_artifact_gap` reads as its
/// `stage4b` receipt.
pub fn run_d11_train_export(
    data_root: &Path,
    save_dir: &Path,
    stage4b: &Path,
    forward: TrainForward,
    device: &Device,
) -> Result<serde_json::Value> {
    use crate::fbs_export::fbs_trunk_snapshot;
    use uor_r4_integer::fbs_snapshot::write_new_file;

    let sheet = d11_train_sheet(forward)?;
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
            forward,
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
            "train_forward": forward.name(),
            "float": last.dev_float,
            "export_mode": last.dev_int,
            "loss_float": last.dev_loss_float,
            "loss_export_mode": last.dev_loss_int,
            "preds_float_sha256": preds_sha256(&out.preds_float),
            "i_save_load": i_save_load,
            "reloaded_loss_float": reloaded_loss,
            "run": out.run,
        }));
    }
    Ok(serde_json::json!({
        "sheet": sheet,
        "train_forward": forward.name(),
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

    #[test]
    fn the_training_forwards_name_their_sheets() -> Result<()> {
        for forward in [TrainForward::Plain, TrainForward::D11, TrainForward::Dual] {
            assert_eq!(TrainForward::parse(forward.name())?, forward);
        }
        assert!(TrainForward::parse("both").is_err());
        assert!(d11_train_sheet(TrainForward::D11)?.ends_with("d11-train-2026-10-26.md"));
        assert!(d11_train_sheet(TrainForward::Dual)?.ends_with("dual-train-2026-10-26.md"));
        // Plain training of the export arm is Stage 4b's run, not a D11-trained one.
        assert!(d11_train_sheet(TrainForward::Plain).is_err());
        Ok(())
    }

    /// On one GPU (claude-131): the dual step's gradient is the plain backward's plus the D11
    /// backward's, tensor by tensor, and a tensor in only one of them keeps its gradient.
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_dual_gradients_add_tensor_by_tensor() -> Result<()> {
        use candle_core::Var;
        let device = Device::new_cuda(0)?;
        let shared = Var::from_vec(vec![1.0f32, -2.0, 0.5], 3, &device)?;
        let plain_only = Var::from_vec(vec![3.0f32, 4.0], 2, &device)?;
        let d11_only = Var::from_vec(vec![-1.0f32], 1, &device)?;
        let a = ((shared.as_tensor().affine(2.0, 0.0)?.sum_all()?
            + plain_only.as_tensor().sum_all()?)?)
        .backward()?;
        let b = ((shared.as_tensor().affine(3.0, 0.0)?.sum_all()?
            + d11_only.as_tensor().affine(5.0, 0.0)?.sum_all()?)?)
        .backward()?;
        let sum = sum_grads(a, &b)?;
        let grad = |v: &Var| -> Result<Vec<f32>> {
            Ok(sum
                .get(v.as_tensor())
                .ok_or_else(|| invalid("a gradient is missing"))?
                .to_vec1::<f32>()?)
        };
        assert_eq!(grad(&shared)?, vec![5.0, 5.0, 5.0]);
        assert_eq!(grad(&plain_only)?, vec![1.0, 1.0]);
        assert_eq!(grad(&d11_only)?, vec![5.0]);
        Ok(())
    }

    /// On one GPU (claude-131): a dual step of the export arm is a plain step plus a D11 step,
    /// each taken on its own, loss and gradient. So the plain half's backward runs outside the D11
    /// mode and the D11 half's inside it; with either half in the other's mode the sum differs.
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_a_dual_step_is_a_plain_step_plus_a_d11_step() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let _guard = ArmGuard::set(D11_EXPORT_ARM);
        let model = build_fbs_model(D11_EXPORT_ARM, 1, &device)?;
        let batches = synthetic_batches(1, 3);
        let (ids, scored, targets) = &batches[0];
        let loss_and_grads = |q12: bool| -> Result<(f64, GradStore)> {
            let loss = batch_loss(&model, ids, scored, targets, SHEET_TRAIN_BATCH, q12)?;
            Ok((f64::from(loss.to_scalar::<f32>()?), loss.backward()?))
        };
        let (plain, plain_grads) = train_step(TrainForward::Plain, 1, 0, loss_and_grads)?;
        let (d11, d11_grads) = train_step(TrainForward::D11, 1, 0, loss_and_grads)?;
        let (dual, dual_grads) = train_step(TrainForward::Dual, 1, 0, loss_and_grads)?;
        assert!(plain.is_finite() && d11.is_finite());
        assert_eq!(dual, plain + d11);
        let max_abs =
            |t: &Tensor| -> Result<f32> { Ok(t.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?) };
        let mut differs = false;
        for (name, var) in model.variables() {
            let get = |store: &GradStore| store.get(var.as_tensor()).cloned();
            let want = match (get(&plain_grads), get(&d11_grads)) {
                (Some(p), Some(d)) => {
                    differs |= max_abs(&(&p - &d)?)? > 0.0;
                    Some((&p + &d)?)
                }
                (Some(g), None) | (None, Some(g)) => Some(g),
                (None, None) => None,
            };
            match (get(&dual_grads), want) {
                (Some(s), Some(w)) => {
                    let gap = max_abs(&(&s - &w)?)?;
                    let tol = 1e-5 * (1.0 + max_abs(&w)?);
                    assert!(gap <= tol, "{name}: dual differs from plain + d11 by {gap}");
                }
                (None, None) => {}
                _ => panic!("{name}: dual and plain + d11 disagree on having a gradient"),
            }
        }
        // The two halves are different gradients, so the test can tell them apart.
        assert!(differs);
        hs::clear_active_arm();
        Ok(())
    }

    #[test]
    fn the_q12_drift_is_each_rows_largest_code_distance() -> Result<()> {
        let q = |codes: &[i32]| -> Vec<f32> { codes.iter().map(|&c| c as f32 / 4096.0).collect() };
        // Three rows of width 2: equal, one code apart, three codes apart (in the second column).
        let a = q(&[10, -20, 0, 5, 7, 100]);
        let b = q(&[10, -20, 1, 5, 7, 97]);
        let drift = q12_drift(&a, &b, 2)?;
        assert_eq!(drift, vec![0, 1, 3]);
        let summary = drift_summary(&drift, |i| i != 0);
        assert_eq!(summary["all"]["rows"], serde_json::json!(3));
        assert_eq!(summary["all"]["max_codes"], serde_json::json!(3));
        assert_eq!(summary["all"]["rows_exact"], serde_json::json!(1));
        assert_eq!(summary["all"]["rows_within_1"], serde_json::json!(2));
        assert_eq!(summary["all"]["rows_within_2"], serde_json::json!(2));
        assert_eq!(summary["selected"]["rows"], serde_json::json!(2));
        assert_eq!(summary["selected"]["rows_exact"], serde_json::json!(0));
        assert!(q12_drift(&a, &b[..4], 2).is_err());
        assert!(q12_drift(&a, &b, 4).is_err());
        assert_eq!(rows_differ(&[1, 2, 3], &[1, 0, 3]), 1);
        Ok(())
    }

    #[test]
    fn the_residual_rule_is_the_sheets() {
        let rows = 14_287;
        // 3% of 14,287 is 428.61, so 428 rows differ at most for a faithful artifact; 10% is
        // 1,428.7, so 1,429 or more is a departure.
        assert_eq!(classify_residual(true, 0, 428, rows, true), "faithful");
        assert_eq!(classify_residual(true, 0, 0, rows, true), "faithful");
        assert_eq!(classify_residual(true, 0, 429, rows, true), "indeterminate");
        assert_eq!(
            classify_residual(true, 0, 1_428, rows, true),
            "indeterminate"
        );
        assert_eq!(classify_residual(true, 0, 1_429, rows, true), "departure");
        // Any head difference, or a paired gap outside F_I, is a departure however few rows differ.
        assert_eq!(classify_residual(true, 1, 0, rows, true), "departure");
        assert_eq!(classify_residual(true, 0, 10, rows, false), "departure");
        // A failed instrument check is never classified.
        assert_eq!(classify_residual(false, 0, 0, rows, true), "not counted");
        assert_eq!(classify_residual(true, 0, 0, 0, true), "not counted");
    }

    /// On one GPU (claude-132): the residual check's head, given the D11 panel's own scored
    /// queries, writes exactly the D11 panel's predictions, so H is the panel's head (`I-Head`).
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_the_head_on_the_d11_queries_is_the_d11_panel() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let model = build_fbs_model(D11_EXPORT_ARM, 1, &device)?;
        crate::fbs_export::round_tables_as_exported(&model)?;
        let batches = synthetic_batches(1, 11);
        let (ids, _, _) = &batches[0];
        let windows: Vec<CausalWindow> = ids
            .chunks(SHEET_CONTEXT)
            .take(4)
            .enumerate()
            .map(|(doc_id, input)| {
                let scored_offsets: Vec<usize> = (0..SHEET_CONTEXT - 1).step_by(5).collect();
                CausalWindow {
                    doc_id,
                    input_ids: input.to_vec(),
                    scored_targets: scored_offsets.iter().map(|&o| input[o + 1]).collect(),
                    scored_corpus_positions: scored_offsets.clone(),
                    scored_offsets,
                }
            })
            .collect();
        let (_, d_preds) = hs::evaluate_dev_panel_d11(&model, &windows)?;
        let d_queries = hs::with_d11_mode(|| hs::dev_panel_q12_queries(&model, &windows))?;
        assert_eq!(d_queries.len(), d_preds.len() * model.config.width);
        assert!(!d_preds.is_empty());
        assert_eq!(hs::write_q12_rows(&model, &d_queries)?, d_preds);
        // Every query is a whole Q12 code.
        assert!(d_queries
            .iter()
            .all(|&v| (v * 4096.0 - (v * 4096.0).round()).abs() == 0.0));
        hs::clear_active_arm();
        Ok(())
    }

    #[test]
    fn the_drift_readings_count_codes_columns_saturation_and_scale() -> Result<()> {
        let q = |codes: &[i32]| -> Vec<f32> { codes.iter().map(|&c| c as f32 / 4096.0).collect() };
        // Two rows of width 3. Row 0: A doubles the reference (scale 2), largest gap in column 2.
        // Row 1: A saturates at the i16 top in column 0, its largest gap.
        let r = q(&[10, 20, 30, 100, 0, -50]);
        let a = q(&[20, 40, 60, 32767, 0, -50]);
        let v = drift_readings(&a, &r, 3)?;
        // No entry is a sign flip: the opposite-sign pairs are all below 64 codes or absent.
        assert_eq!(v["sign_flip_entries"], serde_json::json!(0));
        // A third row with no difference counts toward no column (claude-133 code review).
        let with_equal_row = drift_readings(
            &q(&[20, 40, 60, 32767, 0, -50, 7, 7, 7]),
            &q(&[10, 20, 30, 100, 0, -50, 7, 7, 7]),
            3,
        )?;
        assert_eq!(with_equal_row["rows_no_gap"], serde_json::json!(1));
        assert_eq!(
            with_equal_row["top_max_columns"],
            serde_json::json!([[0, 1], [2, 1], [1, 0]])
        );
        let flipped = drift_readings(&q(&[-100, 63, 5, 64]), &q(&[100, -64, -5, -64]), 4)?;
        // The first and last entries are flips: both sides at least 64 codes, of opposite sign.
        assert_eq!(flipped["sign_flip_entries"], serde_json::json!(2));
        assert_eq!(v["entries"], serde_json::json!(6));
        // The absolute gaps are 10, 20, 30, 32667, 0, 0; sorted 0, 0, 10, 20, 30, 32667.
        assert_eq!(v["median_abs_codes"], serde_json::json!(20));
        assert_eq!(v["max_abs_codes"], serde_json::json!(32667));
        assert_eq!(
            v["top_max_columns"],
            serde_json::json!([[0, 1], [2, 1], [1, 0]])
        );
        // Column 0's mean signed gap is (10 + 32667) / 2.
        assert_eq!(v["max_abs_channel_mean_channel"], serde_json::json!(0));
        assert_eq!(v["max_abs_channel_mean_codes"], serde_json::json!(16338.5));
        assert_eq!(
            v["saturated_entries"],
            serde_json::json!({"queries": 1, "reference": 0})
        );
        // Row 0's L1 scale is 120 / 60 = 2; row 1's is 32817 / 150.
        assert_eq!(v["row_l1_scale"]["p05"], serde_json::json!(2.0));
        assert!(drift_readings(&a, &r[..3], 3).is_err());
        assert!(drift_readings(&[], &[], 3).is_err());
        Ok(())
    }

    /// On one GPU (claude-133): the Q16 control puts the residual on the 2^-16 grid only when it is
    /// on, and off it returns the tensor bit for bit; under it the trunk's output changes.
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_the_q16_control_holds_the_residual_on_its_grid_and_is_off_by_default() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let x = Tensor::from_vec(vec![0.1f32, -2.5001, 3.000_01, 1.0 / 3.0], 4, &device)?;
        let bits = |t: &Tensor| -> Result<Vec<u32>> {
            Ok(t.to_vec1::<f32>()?.iter().map(|v| v.to_bits()).collect())
        };
        assert_eq!(bits(&hs::q16_residual(x.clone())?)?, bits(&x)?);
        let on = hs::with_q16_residual(|| hs::q16_residual(x.clone()))?;
        assert!(on
            .to_vec1::<f32>()?
            .iter()
            .all(|&v| (v * 65536.0).fract() == 0.0));
        assert_ne!(bits(&on)?, bits(&x)?);

        let model = build_fbs_model(D11_EXPORT_ARM, 1, &device)?;
        let batches = synthetic_batches(1, 13);
        let (ids, _, _) = &batches[0];
        let plain = model.hidden(ids, SHEET_TRAIN_BATCH, SHEET_CONTEXT)?;
        let again = model.hidden(ids, SHEET_TRAIN_BATCH, SHEET_CONTEXT)?;
        let control =
            hs::with_q16_residual(|| model.hidden(ids, SHEET_TRAIN_BATCH, SHEET_CONTEXT))?;
        let flat = |t: &Tensor| -> Result<Vec<f32>> { Ok(t.flatten_all()?.to_vec1::<f32>()?) };
        assert_eq!(flat(&plain)?, flat(&again)?);
        assert_ne!(flat(&plain)?, flat(&control)?);
        assert!(flat(&control)?.iter().all(|v| v.is_finite()));
        Ok(())
    }

    #[test]
    fn the_snap_comparison_places_each_disagreement() -> Result<()> {
        // One window of 2 positions, 1 layer, 1 lane and 1 feed-forward block: 3 roots per
        // position, as (transition, output gate, feed-forward gate).
        let shape = SnapShape {
            windows: 1,
            len: 2,
            layers: 1,
            lanes: 1,
            ff_blocks: 1,
        };
        assert_eq!((shape.layer_bytes(), shape.stride()), (3, 3));
        let capture = |roots: [u8; 2], margins: [f32; 2]| hs::SnapCapture {
            roots: roots.to_vec(),
            margins: margins.to_vec(),
        };
        // Position 0 agrees everywhere. Position 1: the transition disagrees (7 against 9, margin
        // 0.5), and the feed-forward gate is zero on both sides: the identity against −1.
        let captures = [
            capture([3, 7], [1.0, 0.5]),
            capture([4, 4], [1.0, 1.0]),
            capture([5, u8::MAX], [1.0, 0.0]),
        ];
        let trace = [3, 4, 5, 9, 4, u8::MAX];
        let c = compare_snaps(&captures, &trace, shape)?;
        assert_eq!(c.sites, 6);
        assert_eq!(c.per_layer_site, vec![[1, 0, 1]]);
        assert_eq!((c.per_class, c.inert), ([1, 1, 0, 0], 0));
        assert_eq!(c.position_root, vec![false, true]);
        assert_eq!(c.position_zero, vec![false, true]);
        assert_eq!(c.root_margins, vec![0.5]);
        assert_eq!(
            c.first,
            vec![Some(FirstFlip {
                position: 1,
                layer: 0,
                site: 0,
                primaries: vec![0.5],
            })]
        );
        assert_eq!(c.first_zero, vec![Some(1)]);
        // A transition disagreement at position 0 rotates a zero state: inert. A float zero block
        // against an integer root 1 applies the identity on both sides: no disagreement.
        let other = [8, 4, 5, 7, 4, 1];
        let c = compare_snaps(&captures, &other, shape)?;
        assert_eq!((c.per_class, c.inert), ([0; 4], 1));
        assert_eq!((c.first, c.first_zero), (vec![None], vec![None]));
        assert_eq!(c.position_root, vec![false, false]);
        // The wrong number of calls, or a short trace, is refused.
        assert!(compare_snaps(&captures[..2], &trace, shape).is_err());
        assert!(compare_snaps(&captures, &trace[..5], shape).is_err());
        // A byte that names no root (the host prefill 0xFE, say) is refused, on either side.
        assert!(compare_snaps(&captures, &[3, 4, 5, 9, 4, 0xFE], shape).is_err());
        let mut bad = captures.clone();
        bad[0].roots[0] = 0xFE;
        assert!(compare_snaps(&bad, &trace, shape).is_err());
        // I-Layout: the transition agrees on 1 snap of 2 (50%), the ff gate (a zero block on both
        // sides) on 1 of 2; a site read at a wrong offset, disagreeing on both, fails.
        assert!(layout_agrees(
            &compare_snaps(&captures, &trace, shape)?,
            shape
        ));
        let shifted = [3, 4, 5, 9, 9, u8::MAX];
        let c = compare_snaps(&captures, &shifted, shape)?;
        assert_eq!(c.per_layer_site, vec![[1, 1, 1]]);
        assert!(layout_agrees(&c, shape));
        let wrong = [9, 4, 9, 9, 4, u8::MAX];
        let c = compare_snaps(&captures, &wrong, shape)?;
        assert_eq!(c.per_layer_site, vec![[1, 0, 2]]);
        assert!(!layout_agrees(&c, shape));
        assert_eq!(hex_bytes("00ff7A")?, vec![0, 255, 122]);
        assert!(hex_bytes("abc").is_err() && hex_bytes("zz").is_err());
        Ok(())
    }

    #[test]
    fn a_snap_class_reads_the_applied_roots() {
        // Root 0 is −1 and root 1 the identity: a zero block's two conventions.
        assert_eq!(
            hs::H4_ROOT_COEFFICIENTS[0],
            [[-2, 0], [0, 0], [0, 0], [0, 0]]
        );
        assert_eq!(
            hs::H4_ROOT_COEFFICIENTS[1],
            [[2, 0], [0, 0], [0, 0], [0, 0]]
        );
        let z = u8::MAX;
        assert_eq!(SnapClass::of(3, 3), None);
        assert_eq!(SnapClass::of(3, 4), Some(SnapClass::Root));
        assert_eq!(SnapClass::of(z, z), Some(SnapClass::ZeroBoth));
        assert_eq!(SnapClass::of(z, 0), Some(SnapClass::ZeroFloat));
        assert_eq!(SnapClass::of(z, 1), None);
        assert_eq!(SnapClass::of(1, z), Some(SnapClass::ZeroInteger));
        assert_eq!(SnapClass::of(0, z), None);
    }

    #[test]
    fn a_windows_first_flip_is_first_in_causal_order() -> Result<()> {
        // One window of 2 positions and 2 layers. Layer 1's output gate disagrees at position 0,
        // layer 0's transition at position 1: position comes before layer. Layer 0's
        // feed-forward gate at position 0 is an integer zero block, earlier still, and it is
        // the window's first zero block but not its first flip.
        let shape = SnapShape {
            windows: 1,
            len: 2,
            layers: 2,
            lanes: 1,
            ff_blocks: 1,
        };
        let captures: Vec<hs::SnapCapture> = (1..=6)
            .map(|r| hs::SnapCapture {
                roots: vec![r, r],
                margins: vec![0.25, 0.75],
            })
            .collect();
        let trace = [1, 2, u8::MAX, 4, 9, 6, 8, 2, 3, 4, 5, 6];
        let c = compare_snaps(&captures, &trace, shape)?;
        assert_eq!(c.per_layer_site, vec![[1, 0, 1], [0, 1, 0]]);
        assert_eq!(c.per_class, [2, 0, 0, 1]);
        assert_eq!(
            c.first,
            vec![Some(FirstFlip {
                position: 0,
                layer: 1,
                site: 1,
                primaries: vec![0.25],
            })]
        );
        assert_eq!(c.first_zero, vec![Some(0)]);
        // The D panel's chunks of 16 windows join per call, in panel order.
        let chunked: Vec<hs::SnapCapture> = (0..6u8)
            .map(|i| hs::SnapCapture {
                roots: vec![i],
                margins: vec![f32::from(i)],
            })
            .collect();
        let joined = join_chunk_captures(chunked.clone(), 20, 1)?;
        assert_eq!(
            joined.iter().map(|c| c.roots.clone()).collect::<Vec<_>>(),
            vec![vec![0, 3], vec![1, 4], vec![2, 5]]
        );
        assert_eq!(joined[2].margins, vec![2.0, 5.0]);
        assert!(join_chunk_captures(chunked[..5].to_vec(), 20, 1).is_err());
        Ok(())
    }

    #[test]
    fn the_snap_trace_outcome_takes_the_first_that_applies() {
        let near_tie = SnapReadings {
            rows_differ: 40,
            rows_agree: 1000,
            differ_windows: 20,
            root: MaskReadings {
                differ_preceded: 39,
                agree_preceded: 500,
                lead_windows: 19,
            },
            zero: MaskReadings::default(),
            root_primaries: 30,
            exact_ties: 2,
            median_margin: Some(1e-4),
        };
        let with = |f: fn(&mut SnapReadings)| {
            let mut r = near_tie;
            f(&mut r);
            classify_snap_trace(&r)
        };
        // Zero blocks that sit before most partings, well above their base rate.
        let leading_zeros = MaskReadings {
            differ_preceded: 30,
            agree_preceded: 100,
            lead_windows: 15,
        };
        assert_eq!(classify_snap_trace(&near_tie), "snap-flip-near-tie");
        // The lift over the agreeing rows' base rate must reach 20 points.
        assert_eq!(with(|r| r.root.agree_preceded = 800), "indeterminate");
        assert_eq!(with(|r| r.root.differ_preceded = 15), "not-snap-carried");
        assert_eq!(with(|r| r.root.lead_windows = 9), "not-snap-carried");
        // Zero blocks before every row, partings or not (a structural zero lane), lift nothing.
        assert_eq!(
            with(|r| {
                r.zero = MaskReadings {
                    differ_preceded: 40,
                    agree_preceded: 1000,
                    lead_windows: 20,
                }
            }),
            "snap-flip-near-tie"
        );
        // The zero convention comes first, then not snap-carried, then the tie-break.
        let mut zeros = near_tie;
        zeros.zero = leading_zeros;
        assert_eq!(classify_snap_trace(&zeros), "zero-convention");
        zeros.root.lead_windows = 9;
        zeros.exact_ties = 30;
        assert_eq!(classify_snap_trace(&zeros), "zero-convention");
        assert_eq!(with(|r| r.exact_ties = 15), "tie-break");
        assert_eq!(with(|r| r.median_margin = Some(0.05)), "localise-upstream");
        assert_eq!(with(|r| r.median_margin = Some(5e-3)), "indeterminate");
        assert_eq!(with(|r| r.median_margin = None), "indeterminate");
        assert_eq!(with(|r| r.rows_differ = 0), "no-rows-differ");
    }

    /// On one GPU (claude-134): the snap capture of a D11-mode forward records, per layer, the
    /// transition, the output gate and the feed-forward gate, in that order, one root per 4-block
    /// (each below 120, or 255 for a zero block) with a non-negative margin, and changes nothing.
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_the_snap_capture_records_each_site_and_changes_nothing() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let model = build_fbs_model(D11_EXPORT_ARM, 1, &device)?;
        crate::fbs_export::round_tables_as_exported(&model)?;
        let batches = synthetic_batches(1, 17);
        let (ids, _, _) = &batches[0];
        let forward = || hs::with_d11_mode(|| model.hidden(ids, SHEET_TRAIN_BATCH, SHEET_CONTEXT));
        let plain = forward()?;
        let (captured_out, captures) = hs::with_snap_capture(forward)?;
        let flat = |t: &Tensor| -> Result<Vec<f32>> { Ok(t.flatten_all()?.to_vec1::<f32>()?) };
        assert_eq!(flat(&plain)?, flat(&captured_out)?);
        let config = &model.config;
        let rows = SHEET_TRAIN_BATCH * SHEET_CONTEXT;
        let lanes = config.width / 4;
        assert_eq!(captures.len(), 3 * config.layers());
        for (call, capture) in captures.iter().enumerate() {
            let blocks = if call % 3 == 2 {
                config.mlp_hidden / 4
            } else {
                lanes
            };
            assert_eq!(capture.roots.len(), rows * blocks, "call {call}");
            assert_eq!(capture.margins.len(), rows * blocks, "call {call}");
            assert!(capture.roots.iter().all(|&r| r < 120 || r == u8::MAX));
            assert!(capture.margins.iter().all(|&m| m >= 0.0));
        }
        // Outside the capture, nothing is recorded.
        assert!(hs::with_snap_capture(|| Ok(()))?.1.is_empty());
        hs::clear_active_arm();
        Ok(())
    }
}
