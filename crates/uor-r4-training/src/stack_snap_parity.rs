//! The D11 snap parity report: a snapped model directory's exported
//! multiplier-free artifact against two float comparators, root selection by
//! root selection and logit by logit, on evenly spaced windows of given
//! tokens.
//!
//! This is the S1.4 parity vehicle, decomposed as in
//! `docs/integration/s1-stack-serving-measurements-2026-09-28.md`:
//!
//! - `kernel`: the D11 engine against the artifact's own dequantized float
//!   reference (same weights, snap set). Weights and snap are shared, so a
//!   divergence comes from the integer arithmetic of the kernel or from a
//!   near-tie at a snap boundary; `kernel_first_divergence` isolates each
//!   window's first one, cascade-free. Mismatches after a window has
//!   diverged are not classified here: they may be cascade through the
//!   recurrence or a kernel error on a rare branch. No numeric acceptance
//!   threshold is coded. Near-ties are counted from the same-weights
//!   reference margin (float margin < 1e-4).
//! - `serving_profile.weight_rounding`: the reference against the trained
//!   unquantized float model — the 4-bit weight-rounding cost. Near-ties use
//!   the trained model's margin.
//! - `serving_profile.end_to_end`: the D11 engine against the trained float
//!   model — the served total. Near-ties use the trained model's margin.
//!
//! Floating point is used here only, offline, as the comparison reference.

use std::path::Path;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_lut::format::StackArtifact;

use crate::geometric_stack::{SnapSelection, StackModel};
use crate::stack_export::{export_stack, stack_grid_reference};
use crate::{invalid, Result};

/// Accumulated comparison of one engine's selections/logits against a float
/// comparator's.
#[derive(Default)]
struct Comparison {
    selections_compared: usize,
    near_ties: usize,
    root_mismatches: usize,
    top1_agree: usize,
    max_gap: f64,
    sum_abs_gap: f64,
    logit_samples: usize,
    /// The largest comparator margin among the root mismatches, i.e. the
    /// arithmetic difference the selection had to be resolved within.
    mismatch_margin_max: f64,
    mismatch_margin_below_1e3: usize,
    mismatch_margin_below_1e2: usize,
}

impl Comparison {
    fn note_mismatch(&mut self, margin: f32) {
        self.root_mismatches += 1;
        self.mismatch_margin_max = self.mismatch_margin_max.max(f64::from(margin));
        if margin < 1e-3 {
            self.mismatch_margin_below_1e3 += 1;
        }
        if margin < 1e-2 {
            self.mismatch_margin_below_1e2 += 1;
        }
    }

    fn note_gap(&mut self, a: f64, b: f64) {
        let gap = (a - b).abs();
        self.max_gap = self.max_gap.max(gap);
        self.sum_abs_gap += gap;
        self.logit_samples += 1;
    }

    fn report(&self, targets: usize, margin_source: &str) -> Value {
        let mean = if self.logit_samples == 0 {
            0.0
        } else {
            self.sum_abs_gap / self.logit_samples as f64
        };
        let agreement = self.top1_agree as f64 / targets as f64;
        json!({
            "selections_compared": self.selections_compared,
            "near_ties": self.near_ties,
            "near_tie_margin_source": margin_source,
            "root_mismatches": self.root_mismatches,
            "top1_agreement": agreement,
            "top1_flip_rate": 1.0 - agreement,
            "max_abs_logit_gap_nats": self.max_gap,
            "mean_abs_logit_gap_nats": mean,
            "mismatch_margin_max": self.mismatch_margin_max,
            "mismatch_margin_below_1e-3": self.mismatch_margin_below_1e3,
            "mismatch_margin_below_1e-2": self.mismatch_margin_below_1e2,
        })
    }
}

/// `sum / n`, or zero for no samples.
fn mean(sum: f64, n: usize) -> f64 {
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

/// The top-1 index of a float logit row.
fn top_f32(row: &[f32]) -> Result<usize> {
    row.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .ok_or_else(|| invalid("an empty float logit row"))
}

/// `ln softmax(row)[target]` for a row of `i32` fixed-point logits scaled by
/// `2^-16`, without materializing a float row.
fn log_softmax_fixed(row: &[i32], target: usize) -> Result<f64> {
    if row.is_empty() || target >= row.len() {
        return Err(invalid("target outside the vocabulary"));
    }
    let scale = 1.0 / 65536.0;
    let max = row
        .iter()
        .map(|&v| f64::from(v) * scale)
        .fold(f64::NEG_INFINITY, f64::max);
    let target_value = f64::from(row[target]) * scale;
    let sum: f64 = row
        .iter()
        .map(|&v| (f64::from(v) * scale - max).exp())
        .sum();
    Ok(target_value - max - sum.ln())
}

/// `ln softmax(row)[target]` for a float row, in f64.
fn log_softmax_f32(row: &[f32], target: usize) -> Result<f64> {
    if row.is_empty() || target >= row.len() {
        return Err(invalid("target outside the vocabulary"));
    }
    let max = row
        .iter()
        .map(|&v| f64::from(v))
        .fold(f64::NEG_INFINITY, f64::max);
    let target_value = f64::from(row[target]);
    let sum: f64 = row.iter().map(|&v| (f64::from(v) - max).exp()).sum();
    Ok(target_value - max - sum.ln())
}

/// Run the snap parity report for the snapped model saved at `model_dir`
/// over `windows` evenly spaced windows of `tokens` (each of the model's
/// context length): export the artifact in memory, serve it with the D11
/// engine with the snap trace on, build the artifact's dequantized float
/// reference with the snap set, and report the kernel and serving-profile
/// comparisons of the module documentation.
pub fn snap_parity_report(model_dir: &Path, tokens: &[u32], windows: usize) -> Result<Value> {
    let mut model = StackModel::load(model_dir, &Device::Cpu)?;
    let snap = StackModel::saved_transport_snap(model_dir)?
        .ok_or_else(|| invalid("snap parity needs a snapped save (transport_snap.json)"))?;
    model.set_transport_snap(Some(snap))?;
    let time = model.config.context;
    if windows == 0 || tokens.len() <= time + windows {
        return Err(invalid("too few tokens for the requested windows"));
    }
    let (bytes, _) = export_stack(
        &model,
        json!({"snap_parity": model_dir.display().to_string()}),
        None,
        Some(snap),
    )?;
    let integer = IntegerStackModel::parse(&bytes).map_err(|e| invalid(e.to_string()))?;
    // The artifact's own dequantized float reference: the serving parse
    // refuses the snap record, so the offline-only reference parse is used.
    let artifact = StackArtifact::parse_for_reference(bytes).map_err(|e| invalid(e.to_string()))?;
    let mut reference = stack_grid_reference(&model, &artifact)?;
    reference.model.set_transport_snap(Some(snap))?;
    let lanes = model.config.width / 4;
    let stride = (tokens.len() - time - 1) / windows;
    let mut session = integer.session();
    session.enable_snap_trace();
    let mut kernel = Comparison::default();
    let mut weight_rounding = Comparison::default();
    let mut end_to_end = Comparison::default();
    let (mut integer_nll, mut reference_nll, mut trained_nll, mut nll_targets) =
        (0f64, 0f64, 0f64, 0usize);
    let mut per_window = Vec::with_capacity(windows);
    let mut first_divergence = Vec::with_capacity(windows);
    let mut first_margin_max = 0f32;
    for window in 0..windows {
        let start = window * stride;
        let ids = &tokens[start..start + time];
        let trained_logits = model.forward(ids, 1, time)?.to_vec2::<f32>()?;
        let trained_selections = model.snap_selections(ids, 1, time)?;
        let reference_logits = reference.logits(ids, 1, time)?.to_vec2::<f32>()?;
        let reference_selections = reference.model.snap_selections(ids, 1, time)?;
        session.reset();
        session.enable_snap_trace();
        let mut window_mismatch = 0usize;
        for (t, &id) in ids.iter().enumerate() {
            let logits = session.step(id).map_err(|e| invalid(e.to_string()))?;
            let top_i = logits
                .iter()
                .enumerate()
                .max_by_key(|&(_, &v)| v)
                .map(|(i, _)| i)
                .ok_or_else(|| invalid("an empty logit row"))?;
            kernel.top1_agree += usize::from(top_i == top_f32(&reference_logits[t])?);
            end_to_end.top1_agree += usize::from(top_i == top_f32(&trained_logits[t])?);
            weight_rounding.top1_agree +=
                usize::from(top_f32(&reference_logits[t])? == top_f32(&trained_logits[t])?);
            for (&a, &b) in logits.iter().zip(&reference_logits[t]) {
                kernel.note_gap(f64::from(a) / 65536.0, f64::from(b));
            }
            for (&a, &b) in logits.iter().zip(&trained_logits[t]) {
                end_to_end.note_gap(f64::from(a) / 65536.0, f64::from(b));
            }
            for (&a, &b) in reference_logits[t].iter().zip(&trained_logits[t]) {
                weight_rounding.note_gap(f64::from(a), f64::from(b));
            }
            let target = tokens[start + t + 1] as usize;
            integer_nll += log_softmax_fixed(&logits, target)?;
            reference_nll += log_softmax_f32(&reference_logits[t], target)?;
            trained_nll += log_softmax_f32(&trained_logits[t], target)?;
            nll_targets += 1;
        }
        let trace = session
            .snap_trace()
            .ok_or_else(|| invalid("the snap trace is not enabled"))?;
        let trace_roots: std::collections::HashMap<(usize, usize, usize), usize> = trace
            .iter()
            .map(|e| ((e.layer, e.position, e.lane), e.root))
            .collect();
        // The first mismatch in (layer, position) order precedes every cascade,
        // so its reference margin is measured on a rollout whose selected roots
        // have matched up to that point.
        let mut first: Option<(usize, usize, f32)> = None;
        for (layer, selected) in &reference_selections {
            for (k, selection) in selected.iter().enumerate() {
                let (position, lane) = (k / lanes, k % lanes);
                let root = *trace_roots
                    .get(&(*layer, position, lane))
                    .ok_or_else(|| invalid("the trace missed a selection"))?;
                if root != selection.index {
                    first = Some((*layer, position, selection.margin));
                    break;
                }
            }
            if first.is_some() {
                break;
            }
        }
        first_divergence.push(match first {
            Some((layer, position, margin)) => {
                first_margin_max = first_margin_max.max(margin);
                json!({
                    "window": window,
                    "layer": layer,
                    "position": position,
                    "reference_margin": margin,
                })
            }
            None => json!({"window": window, "exact_match": true}),
        });
        // Compare one selection triple (kernel: trace vs reference; profile:
        // reference vs trained and trace vs trained). Near-tie margins come
        // from the comparator named in each block's `near_tie_margin_source`.
        let compare = |acc: &mut Comparison,
                       actual: &mut dyn FnMut(usize, usize, usize) -> Result<usize>,
                       expected: &std::collections::BTreeMap<usize, Vec<SnapSelection>>,
                       count_mismatch: bool|
         -> Result<usize> {
            let mut mismatched = 0usize;
            for (layer, selected) in expected {
                for (k, selection) in selected.iter().enumerate() {
                    let root = actual(*layer, k / lanes, k % lanes)?;
                    if selection.margin < 1e-4 {
                        acc.near_ties += 1;
                    } else {
                        acc.selections_compared += 1;
                        if root != selection.index {
                            acc.note_mismatch(selection.margin);
                            if count_mismatch {
                                mismatched += 1;
                            }
                        }
                    }
                }
            }
            Ok(mismatched)
        };
        window_mismatch += compare(
            &mut kernel,
            &mut |layer, position, lane| {
                trace_roots
                    .get(&(layer, position, lane))
                    .copied()
                    .ok_or_else(|| invalid("the trace missed a selection"))
            },
            &reference_selections,
            true,
        )?;
        compare(
            &mut weight_rounding,
            &mut |layer, position, lane| {
                Ok(reference_selections
                    .get(&layer)
                    .and_then(|s| s.get(position * lanes + lane))
                    .map(|s| s.index)
                    .ok_or_else(|| invalid("the reference missed a selection"))?)
            },
            &trained_selections,
            false,
        )?;
        compare(
            &mut end_to_end,
            &mut |layer, position, lane| {
                trace_roots
                    .get(&(layer, position, lane))
                    .copied()
                    .ok_or_else(|| invalid("the trace missed a selection"))
            },
            &trained_selections,
            false,
        )?;
        per_window.push(json!({
            "window": window,
            "start": start,
            "root_mismatches": window_mismatch,
        }));
    }
    let targets = windows * time;
    Ok(json!({
        "schema": "uor-r4.stack-snap-parity/2",
        "model": model_dir.display().to_string(),
        "transport_snap": snap.record(),
        "windows": windows,
        "positions": targets,
        "kernel": kernel.report(targets, "reference"),
        "serving_profile": {
            "weight_rounding": weight_rounding.report(targets, "trained_float"),
            "end_to_end": end_to_end.report(targets, "trained_float"),
        },
        "mean_log_prob_over_window_targets": {
            "targets": nll_targets,
            "trained_float": mean(trained_nll, nll_targets),
            "artifact_reference": mean(reference_nll, nll_targets),
            "integer": mean(integer_nll, nll_targets),
            "integer_minus_reference": mean(integer_nll - reference_nll, nll_targets),
            "reference_minus_trained": mean(reference_nll - trained_nll, nll_targets),
            "integer_minus_trained": mean(integer_nll - trained_nll, nll_targets),
        },
        "kernel_first_divergence": {
            "max_reference_margin": first_margin_max,
            "per_window": first_divergence,
        },
        "per_window": per_window,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{ReadScore, StackArch, StackConfig, TransportSnap};
    use std::fs;

    #[test]
    fn a_tiny_snapped_model_reports_parity() {
        let dir = std::env::temp_dir().join(format!("uor-snap-parity-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut config = StackConfig::transformer_control(0x5A9);
        config.arch = StackArch::Geometric;
        config.vocab_size = 64;
        config.width = 32;
        config.heads = 2;
        config.mlp_hidden = 32;
        config.context = 8;
        config.pattern = "ra".to_owned();
        config.read = ReadScore::Lorentz;
        config.rotation = true;
        let mut model = StackModel::new(config, &Device::Cpu).expect("model");
        model
            .set_transport_snap(Some(TransportSnap::Icosian))
            .expect("snap");
        model.save(&dir).expect("save");
        let tokens: Vec<u32> = (0..40).map(|i| (i * 5 + 1) % 64).collect();
        let report = snap_parity_report(&dir, &tokens, 2).expect("report");
        assert_eq!(report["schema"], "uor-r4.stack-snap-parity/2");
        // Same weights and snap: on this tiny model the kernel block shows no
        // root mismatch and full top-1 agreement.
        let kernel = &report["kernel"];
        assert_eq!(kernel["root_mismatches"], 0);
        assert_eq!(kernel["top1_agreement"], 1.0);
        assert!(kernel["selections_compared"].as_u64().expect("compared") > 0);
        // The serving profile includes weight rounding, which may
        // legitimately perturb a tiny random model; no cleanliness asserted.
        assert!(
            report["serving_profile"]["weight_rounding"]["selections_compared"]
                .as_u64()
                .expect("compared")
                > 0
        );
        assert!(
            report["serving_profile"]["end_to_end"]["selections_compared"]
                .as_u64()
                .expect("compared")
                > 0
        );
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
