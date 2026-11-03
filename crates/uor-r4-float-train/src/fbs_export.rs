//! D11 export of a SpiralCore/FBS float trunk (d11-exporter-float-trunk,
//! claude-123): the converter from a [`StackModel`], as Stage 3 trains and
//! saves it, to the integer crate's [`FbsTrunkSnapshot`].
//!
//! A float FBS projection routes each output 8-block by a GPU argmin over
//! learned query points, so its routes are read back through the model's own
//! GPU projection: one coded carrier, whose input slot `i = atom*8 + c` holds
//! `i + 1`, comes out as `±(atom*8 + (code & 7) + 1)` in every output channel.
//! The atom, axis and sign of each route are then read exactly, and no host
//! argmin is computed. Everything else is copied by name.
//!
//! The snapshot is plain data (`uor_r4_integer::fbs_snapshot`), built without
//! the integer crate's serving build; `fbs-export` turns its JSON into the
//! `UORLUT01` artifact.

use std::path::{Path, PathBuf};

use candle_core::{Tensor, Var};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, spiralcore_signed_axes};
use uor_r4_integer::fbs_snapshot::{
    fixed_value, grid_code, grid_value, q12_value, write_new_file, FbsFloatPosition,
    FbsFloatReadout, FbsFloatSequence, FbsHammingRead, FbsHammingSnapshot, FbsMemoryEntry,
    FbsMemorySnapshot, FbsReadoutMode, FbsReplyTable, FbsRoutes, FbsTrunkLayer, FbsTrunkSnapshot,
    FBS_FIXED_EXP,
};

use crate::flock_read_hamming::{HammingReads, HAMMING_DEPTHS, HAMMING_READOUT_DEPTH};
use crate::flock_read_memory::TurnMemory;

use crate::geometric_stack::{
    Gate, QuaternionWeights, RecGates, RotationGroup, StackArch, StackConfig, StackModel,
    RMS_EPSILON,
};
use crate::native_vector_writer::WriterSpec;
use crate::{invalid, Result};

/// The six projections of one recurrence layer, in [`FbsTrunkLayer`] order.
pub const FBS_PROJECTIONS: [&str; 6] = [
    "rec.in", "rec.gate", "rec.out", "mlp.gate", "mlp.up", "mlp.down",
];

/// The integer artifact's Hamming exponents, as `uor-r4-integer`'s `stack/format.rs` sets them
/// (its native build does not expose `stack`); a test holds them to that file.
const HAMMING_ROTATION_EXP: i32 = -24;
const HAMMING_MARGIN_EXP: i32 = -16;
const HAMMING_WEIGHT_EXP: i32 = -24;
/// And its turn memory's move weights (claude-252), held to the same file.
const MEMORY_WEIGHT_EXP: i32 = -24;

/// Refuse a trunk the integer FBS session cannot serve, each for its own reason.
pub fn check_fbs_export_config(
    config: &StackConfig,
    weights: QuaternionWeights,
    head: Option<&WriterSpec>,
) -> Result<()> {
    let refusal = if config.arch != StackArch::Geometric {
        Some("the trunk is not geometric")
    } else if config.pattern.is_empty() || config.pattern.bytes().any(|c| c != b'r') {
        Some("the integer FBS trunk serves recurrence layers only")
    } else if config.rec_gates != RecGates::Geometric {
        Some("the recurrence gates are not geometric")
    } else if !config.rotation {
        Some("the recurrence has no learned rotations")
    } else if config.rotation_group != RotationGroup::Quaternion {
        Some("the U(1) rotation control is not served")
    } else if config.gate != Gate::Hamilton {
        Some("the feed-forward gate is not Hamilton")
    } else if weights != QuaternionWeights::SpiralcoreFbs {
        Some("the projections are not SpiralCore/FBS")
    } else if config.pointer.is_some() {
        Some("a pointer head is not served")
    } else if config.memory.is_some() || config.select.is_some() || config.read_router.is_some() {
        Some("a memory, read selection or read router is not served")
    } else if head.is_none_or(|h| h.vocabulary != config.vocab_size || h.width != config.width) {
        Some("the native vector head is absent or does not match the trunk")
    } else {
        None
    };
    refusal.map_or(Ok(()), |r| Err(invalid(r)))
}

/// Compare a float geometry's signed axes (64 states × 8, low three bits an
/// axis, bit 3 a negation) and orbit roots with the integer crate's, in order.
/// The integer session applies the integer table's codes, and the writer's
/// cell `a*16 + b` names roots by their order.
pub fn compare_geometry_tables(axes: &[u32], roots: &[[i8; 8]; 16]) -> Result<()> {
    let integer_axes = spiralcore_signed_axes();
    if axes.len() != integer_axes.len()
        || axes
            .iter()
            .zip(integer_axes)
            .any(|(&f, &i)| f != u32::from(i))
    {
        return Err(invalid(
            "the float and integer Cl(0,6) signed-axis tables differ",
        ));
    }
    if roots != spiralcore_orbit_roots() {
        return Err(invalid("the float and integer orbit roots differ"));
    }
    Ok(())
}

/// [`compare_geometry_tables`] on this crate's own SpiralCore/FBS geometry.
pub fn check_geometry_tables() -> Result<()> {
    let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?;
    compare_geometry_tables(&plan.geometry.axes, &plan.geometry.roots)
}

/// The coded carrier of a projection with `in_blocks` input 8-blocks: slot `i`
/// holds `i + 1`, an exact `f32` integer.
pub fn coded_carrier(in_blocks: usize) -> Vec<f32> {
    (1..=in_blocks * 8).map(|i| i as f32).collect()
}

/// Read the routes of a projection back from its output on the coded carrier.
///
/// Each output block must read one input atom through eight whole signed
/// axes, and the eight codes must be a state of the integer Cl(0,6) table;
/// anything else (a mix of atoms, a scaled or non-integer value, a code row
/// outside the group) is refused, because the integer session serves only a
/// gather and a sign.
pub fn decode_coded_routes(
    projected: &[f32],
    in_blocks: usize,
    out_blocks: usize,
) -> Result<FbsRoutes> {
    if in_blocks == 0 || in_blocks > usize::from(u16::MAX) + 1 || projected.len() != out_blocks * 8
    {
        return Err(invalid(format!(
            "a coded projection of {in_blocks} -> {out_blocks} blocks needs {} values, got {}",
            out_blocks * 8,
            projected.len()
        )));
    }
    let slots = (in_blocks * 8) as f32;
    let axes = spiralcore_signed_axes();
    let mut atoms = Vec::with_capacity(out_blocks);
    let mut states = Vec::with_capacity(out_blocks);
    for (o, block) in projected.chunks_exact(8).enumerate() {
        let mut atom = None;
        let mut codes = [0u8; 8];
        for (j, &v) in block.iter().enumerate() {
            let k = v.abs();
            if !(k >= 1.0 && k <= slots && k.fract() == 0.0) {
                return Err(invalid(format!(
                    "block {o} channel {j}: {v} is not a coded input slot"
                )));
            }
            let slot = k as usize - 1;
            if *atom.get_or_insert(slot / 8) != slot / 8 {
                return Err(invalid(format!(
                    "block {o} reads two atoms: the projection is not a signed gather"
                )));
            }
            codes[j] = (slot % 8) as u8 | if v < 0.0 { 8 } else { 0 };
        }
        let state = axes
            .chunks_exact(8)
            .position(|row| row == codes)
            .ok_or_else(|| {
                invalid(format!(
                    "block {o}: codes {codes:?} are not a state of the Cl(0,6) table"
                ))
            })?;
        atoms.push(atom.map_or(0, |a| a as u16));
        states.push(state as u8);
    }
    Ok(FbsRoutes {
        in_blocks,
        out_blocks,
        atoms,
        states,
    })
}

/// The `(input, output)` widths of projection `part` in a trunk of width `d`,
/// MLP width `m` and `g = d/4 + d` gate rows.
fn projection_widths(part: &str, d: usize, m: usize) -> (usize, usize) {
    match part {
        "rec.in" => (d, 2 * d),
        "rec.gate" => (d, d / 4 + d),
        "rec.out" => (d, d),
        "mlp.gate" | "mlp.up" => (d, m),
        _ => (m, d),
    }
}

/// Read the routes of one projection variable back through the model's own GPU
/// projection on the coded carrier.
fn read_routes(
    model: &StackModel,
    name: &str,
    in_width: usize,
    out_width: usize,
) -> Result<FbsRoutes> {
    let queries = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?
        .as_tensor();
    let carrier = Tensor::from_vec(coded_carrier(in_width / 8), (1, in_width), queries.device())?;
    let projected = model
        .linear(&carrier, queries)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    decode_coded_routes(&projected, in_width / 8, out_width / 8)
        .map_err(|e| invalid(format!("{name}: {e}")))
}

/// Refuse a copied table that holds a value that is not finite, naming it: a
/// diverged trunk must not reach the JSON handoff, which would write `null`.
pub fn finite_table(name: &str, values: Vec<f32>) -> Result<Vec<f32>> {
    match values.iter().position(|v| !v.is_finite()) {
        Some(i) => Err(invalid(format!(
            "{name}[{i}] = {} is not finite",
            values[i]
        ))),
        None => Ok(values),
    }
}

/// Refuse to read a trunk on a thread with a SpiralCore/FBS sheet arm set:
/// `linear` would route through the arm, and the snapshot would not say so.
fn check_no_sheet_arm() -> Result<()> {
    match crate::spiralcore_fbs_arms::fbs_arm() {
        Some(arm) => Err(invalid(format!(
            "a SpiralCore/FBS sheet arm ({arm:?}) is set on this thread; clear it before export"
        ))),
        None => Ok(()),
    }
}

/// Convert a SpiralCore/FBS float trunk to the integer crate's snapshot. GPU
/// only: the routes are read back through the model's CUDA projection.
pub fn fbs_trunk_snapshot(model: &StackModel) -> Result<FbsTrunkSnapshot> {
    let config = &model.config;
    let head = model.native_vector_head_spec();
    check_no_sheet_arm()?;
    check_fbs_export_config(config, model.quaternion_weights(), head)?;
    check_geometry_tables()?;
    model.validate_native_learning_path()?;
    let kept_cells = head.map_or(0, |h| h.kept_cells);
    let (d, m) = (config.width, config.mlp_hidden);
    let values = |name: &str| -> Result<Vec<f32>> {
        let values = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        finite_table(name, values)
    };
    let mut layers = Vec::with_capacity(config.layers());
    for layer in 0..config.layers() {
        let at = |suffix: &str| format!("layers.{layer:02}.{suffix}");
        let mut routes = Vec::with_capacity(FBS_PROJECTIONS.len());
        for part in FBS_PROJECTIONS {
            let (in_width, out_width) = projection_widths(part, d, m);
            routes.push(read_routes(
                model,
                &at(&format!("{part}.spiralcore_fbs")),
                in_width,
                out_width,
            )?);
        }
        let [rec_in, rec_gate, rec_out, mlp_gate, mlp_up, mlp_down]: [FbsRoutes; 6] = routes
            .try_into()
            .map_err(|_| invalid("six projections per layer"))?;
        layers.push(FbsTrunkLayer {
            rec_norm: values(&at("rec_norm.weight"))?,
            mlp_norm: values(&at("mlp_norm.weight"))?,
            rec_in,
            rec_gate,
            rec_out,
            mlp_gate,
            mlp_up,
            mlp_down,
            // `[CONVOLUTION_WIDTH, d]` row-major: tap 0 on the current position.
            conv_taps: values(&at("rec.conv.weight"))?,
            conv_bias: values(&at("rec.conv.bias"))?,
            gate_bias: values(&at("rec.gate.bias"))?,
            decay: values(&at("rec.decay"))?,
        });
    }
    Ok(FbsTrunkSnapshot {
        vocab: config.vocab_size,
        width: d,
        mlp: m,
        context: config.context,
        rms_eps: RMS_EPSILON,
        kept_cells,
        embed: values("embedding.weight")?,
        final_norm: values("final_norm.weight")?,
        layers,
        // The Hamming reads are not part of the `StackModel` either: `fbs-snapshot` loads them
        // and sets them with `fbs_hamming_snapshot` (claude-200).
        hamming: None,
        // The memory is not part of the `StackModel`: `fbs-snapshot` loads it from the checkpoint
        // directory and sets it with `fbs_memory_snapshot` (claude-198).
        memory: None,
        // Nor is the rate reading: `fbs-snapshot` sets it from the run's config (claude-226).
        retain_rates: false,
        // Nor the readout: `fbs-snapshot` sets it from the run's objective (claude-231).
        readout_nearest: false,
    })
}

/// The safetensors files of a checkpoint directory the export accounts for (claude-198): the
/// trunk, the optimizer's state (training only, never served), and the reads it carries, the
/// turn memory and the Hamming reads (claude-200). Any other is a read it does not carry: today
/// the Flock, content and opening reads and the prompt register (`in_space_long`'s final
/// checkpoint).
const CHECKPOINT_ACCOUNTED: [&str; 5] = [
    "model.safetensors",
    "optimizer.safetensors",
    "turn_memory.safetensors",
    "turn_memory_reply.safetensors",
    "hamming_read.safetensors",
];

/// Refuse a checkpoint directory that holds a read the export does not carry, naming every one,
/// so the export never serves the trunk without it (claude-198). A reply table without its
/// opening table is refused too:
/// `TurnMemory::load` reads neither without the opening file. A macOS `._*.safetensors` copy
/// artifact is refused like any unknown file, which is the safe direction.
pub fn check_checkpoint_reads(directory: &Path) -> Result<()> {
    if directory.join("turn_memory_reply.safetensors").exists()
        && !directory.join("turn_memory.safetensors").exists()
    {
        return Err(invalid(
            "the checkpoint holds turn_memory_reply.safetensors without turn_memory.safetensors, so the export would drop the reply table",
        ));
    }
    let mut refused = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".safetensors") && !CHECKPOINT_ACCOUNTED.contains(&name.as_str()) {
            refused.push(name);
        }
    }
    if refused.is_empty() {
        return Ok(());
    }
    refused.sort();
    Err(invalid(format!(
        "the checkpoint holds {}, which the export does not carry yet; exporting the trunk without it would serve another model",
        refused.join(", ")
    )))
}

/// Refuse float reference readouts for a checkpoint whose reads they would leave out
/// (claude-200): the turn memory moves the readout only at rows its caller names, which
/// `fbs_float_readout` does not compute, so readouts of a checkpoint with a memory would be the
/// trunk without it. The snapshot alone is still written.
pub fn check_readout_reads(directory: &Path, readouts: usize) -> Result<()> {
    if readouts > 0 && directory.join("turn_memory.safetensors").exists() {
        return Err(invalid(
            "the float readouts do not apply the turn memory, whose moves need the caller's rows; write the snapshot alone, without readouts",
        ));
    }
    Ok(())
}

/// The snapshot of a trunk's Hamming reads (claude-146) for a trunk of `layers` layers on points
/// of `width` coordinates (claude-200): each read's rotations, margin and weight, row-major, as
/// `hamming_read.safetensors` stores them, at the depths [`HAMMING_DEPTHS`].
///
/// It refuses three things:
/// - a read with the type spin (`step_rotation`, claude-151), which the integer engine does not
///   serve;
/// - a trunk whose layer count is not [`HAMMING_READOUT_DEPTH`]. The float read at that depth runs
///   on the readout whatever the layer count, and the integer engine takes the read at the layer
///   count as the readout (claude-199);
/// - a table of another shape, or one with a value that is not finite.
pub fn fbs_hamming_snapshot(
    reads: &HammingReads,
    layers: usize,
    width: usize,
) -> Result<FbsHammingSnapshot> {
    if layers != HAMMING_READOUT_DEPTH {
        return Err(invalid(format!(
            "the Hamming readout read runs at depth {HAMMING_READOUT_DEPTH}, but this trunk has {layers} layers; the export serves it only on a trunk of {HAMMING_READOUT_DEPTH}"
        )));
    }
    if reads.reads.len() != HAMMING_DEPTHS.len() || width == 0 || !width.is_multiple_of(4) {
        return Err(invalid(format!(
            "a Hamming read needs one read per depth {HAMMING_DEPTHS:?} on a width of whole quaternions"
        )));
    }
    let heads = reads.reads[0].margin.dims().first().copied().unwrap_or(0);
    let mut out = Vec::with_capacity(reads.reads.len());
    for (depth, read) in HAMMING_DEPTHS.iter().zip(&reads.reads) {
        if read.step_rotation.is_some() {
            return Err(invalid(format!(
                "hamming_read d{depth} has the type spin (step_rotation), which the integer engine does not serve yet"
            )));
        }
        let table = |name: &str, var: &Var, shape: &[usize]| -> Result<Vec<f32>> {
            let name = format!("hamming_read.d{depth}.{name}");
            if var.dims() != shape {
                return Err(invalid(format!(
                    "{name} has shape {:?}, not {shape:?}",
                    var.dims()
                )));
            }
            finite_table(&name, var.as_tensor().flatten_all()?.to_vec1::<f32>()?)
        };
        let quaternions = [width / 4, 4];
        out.push(FbsHammingRead {
            query_rotation: table("query_rotation", &read.query_rotation, &quaternions)?,
            key_rotation: table("key_rotation", &read.key_rotation, &quaternions)?,
            value_rotation: table("value_rotation", &read.value_rotation, &quaternions)?,
            margin: table("margin", &read.margin, &[heads])?,
            weight: table("weight", &read.weight, &[2, width])?,
        });
        // A head with a margin at or below 0 never reads in float; the integer read refuses it,
        // so it is refused here by name, before the JSON handoff.
        let margin = &out.last().expect("pushed").margin;
        if let Some(i) = margin.iter().position(|&m| m <= 0.0) {
            return Err(invalid(format!(
                "hamming_read.d{depth}.margin[{i}] = {} is not positive, and the integer read refuses it",
                margin[i]
            )));
        }
    }
    Ok(FbsHammingSnapshot {
        heads,
        depths: HAMMING_DEPTHS.to_vec(),
        reads: out,
        // The copy heads' key (claude-261), absent from a one-token snapshot's JSON, and each
        // head's own when they differ (claude-268), absent otherwise.
        copy_key: reads.copy_key(),
        copy_keys: reads.mixed_copy_keys(),
    })
}

/// Whether a run's recurrence read its rate table as retention (integer-retention-rates,
/// claude-226), from the run's `config.json`: its `retain_rates`, false when absent, as in every run
/// before 27 Oct 2026. A run with `fold_scores` or `reflect_floor` on is refused: those change the
/// rate score before its selection, and the integer artifact only clamps it.
pub fn run_rate_reading(run_config: &str) -> Result<bool> {
    let config: serde_json::Value = serde_json::from_str(run_config)?;
    let flag = |name: &str| config.get(name).and_then(serde_json::Value::as_bool).unwrap_or(false);
    for name in ["fold_scores", "reflect_floor"] {
        if flag(name) {
            return Err(invalid(format!(
                "the run trained with {name} on, whose integer form does not exist: its export is refused (integer-retention-rates)"
            )));
        }
    }
    Ok(flag("retain_rates"))
}

/// The snapshot of a turn memory (claude-161, claude-193): its tables in ascending key order, as
/// `TurnMemory::save` writes them, its weights, and the training window its keys were cut in
/// (claude-198).
pub fn fbs_memory_snapshot(memory: &TurnMemory, key_window: usize) -> Result<FbsMemorySnapshot> {
    let entries = |rows: Vec<(Vec<u32>, u32)>| -> Vec<FbsMemoryEntry> {
        rows.into_iter()
            .map(|(key, value)| FbsMemoryEntry { key, value })
            .collect()
    };
    Ok(FbsMemorySnapshot {
        k: memory.k(),
        key_window,
        opening: entries(memory.opening_entries()),
        opening_weight: finite_table("turn_memory.weight", memory.opening_weight()?)?,
        reply: match memory.reply_parts()? {
            None => None,
            Some((rows, weight)) => Some(FbsReplyTable {
                turn: crate::in_space_long::TURN_KEY,
                entries: entries(rows),
                weight: finite_table("turn_memory_reply.weight", weight)?,
            }),
        },
    })
}

/// Round the trunk's tables in place to the values the exported artifact holds
/// (claude-125): grid codes for the RMS gains and the convolution taps, Q16 for
/// the biases and decay offsets, and Q12 for the embedding, which is also the
/// writer table. The FBS routes are exact and stay as they are. With these
/// tables, [`FbsReadoutMode::D11`] computes what the artifact computes.
pub fn round_tables_as_exported(model: &StackModel) -> Result<()> {
    for (name, round) in exported_roundings(model) {
        let (var, _, rounded) = table_and_rounding(model, &name, round)?;
        let tensor = var.as_tensor();
        var.set(&Tensor::from_vec(rounded, tensor.shape(), tensor.device())?)?;
    }
    Ok(())
}

/// claude-235: round the Hamming reads' parameters as the integer artifact
/// holds them (`quantize_fixed_i32` on [`fbs_hamming_snapshot`]'s tables):
/// the rotations at `2^HAMMING_ROTATION_EXP`, the margins at
/// `2^HAMMING_MARGIN_EXP` and the weights at `2^HAMMING_WEIGHT_EXP`, so the
/// float read computes with the values the integer read serves. Only the
/// trace's rounded arms call it. Every rounded value is an exact `f32`: one
/// whose `f32` spacing is its step or coarser is on the step already, and a
/// finer one rounds to at most 24 significant bits. What the export or the
/// integer read refuses is refused here, before any value changes: the type
/// spin, a value that is not finite, a value past `i32` at its step, a margin
/// that rounds to 0 or below, and a read count other than one per depth.
pub fn round_hamming_as_exported(reads: &HammingReads) -> Result<()> {
    if reads.reads.len() != HAMMING_DEPTHS.len() {
        return Err(invalid(format!(
            "{} Hamming reads, not one at each depth of {HAMMING_DEPTHS:?}",
            reads.reads.len()
        )));
    }
    let mut rounded = Vec::new();
    for (depth, read) in HAMMING_DEPTHS.iter().zip(&reads.reads) {
        if read.step_rotation.is_some() {
            return Err(invalid(
                "a Hamming read with the type spin (step_rotation) has no exported form to round to",
            ));
        }
        for (table, var, exp) in [
            ("query_rotation", &read.query_rotation, HAMMING_ROTATION_EXP),
            ("key_rotation", &read.key_rotation, HAMMING_ROTATION_EXP),
            ("value_rotation", &read.value_rotation, HAMMING_ROTATION_EXP),
            ("margin", &read.margin, HAMMING_MARGIN_EXP),
            ("weight", &read.weight, HAMMING_WEIGHT_EXP),
        ] {
            let name = format!("hamming_read.d{depth}.{table}");
            let values = finite_table(&name, var.as_tensor().flatten_all()?.to_vec1::<f32>()?)?;
            let steps = |v: f32| (f64::from(v) * 2f64.powi(-exp)).round();
            if let Some(i) = values
                .iter()
                .position(|&v| steps(v).abs() > f64::from(i32::MAX))
            {
                return Err(invalid(format!(
                    "{name}[{i}] = {} overflows i32 at 2^{exp}, and the export refuses it",
                    values[i]
                )));
            }
            if table == "margin" {
                if let Some(i) = values.iter().position(|&v| steps(v) <= 0.0) {
                    return Err(invalid(format!(
                        "{name}[{i}] = {} rounds to {} at 2^{exp}, not positive, and the integer read refuses it",
                        values[i],
                        fixed_value(values[i], exp)
                    )));
                }
            }
            let values: Vec<f32> = values
                .into_iter()
                .map(|v| fixed_value(v, exp) as f32)
                .collect();
            rounded.push((var, values));
        }
    }
    for (var, values) in rounded {
        let tensor = var.as_tensor();
        var.set(&Tensor::from_vec(values, tensor.shape(), tensor.device())?)?;
    }
    Ok(())
}

/// claude-252: round the turn memory's move weights as the integer artifact holds them
/// (`quantize_fixed_i32` at `2^MEMORY_WEIGHT_EXP` on [`fbs_memory_snapshot`]'s weights): the
/// opening weight, and the reply weight when the memory has a reply table. Its keys and stored
/// tokens are exact and stay as they are. Every rounded value is an exact `f32`, as in
/// [`round_hamming_as_exported`]. A weight that is not finite, or past `i32` at its step, is
/// refused before any value changes, as the export refuses it.
pub fn round_memory_as_exported(memory: &TurnMemory) -> Result<()> {
    let mut weights = vec![("turn_memory.weight", &memory.weight)];
    if let Some(reply) = &memory.reply {
        weights.push(("turn_memory_reply.weight", &reply.weight));
    }
    let exp = MEMORY_WEIGHT_EXP;
    let mut rounded = Vec::new();
    for (name, var) in weights {
        let values = finite_table(name, var.as_tensor().to_vec1::<f32>()?)?;
        let steps = |v: f32| (f64::from(v) * 2f64.powi(-exp)).round();
        if let Some(i) = values
            .iter()
            .position(|&v| steps(v).abs() > f64::from(i32::MAX))
        {
            return Err(invalid(format!(
                "{name}[{i}] = {} overflows i32 at 2^{exp}, and the export refuses it",
                values[i]
            )));
        }
        let values: Vec<f32> = values
            .into_iter()
            .map(|v| fixed_value(v, exp) as f32)
            .collect();
        rounded.push((var, values));
    }
    for (var, values) in rounded {
        let tensor = var.as_tensor();
        var.set(&Tensor::from_vec(values, tensor.shape(), tensor.device())?)?;
    }
    Ok(())
}

/// The marker of an evaluation copy rounded as exported ([`write_rounded_copy`]), in its step
/// directory, written last.
pub const ROUNDED_COPY_MARKER: &str = "rounded-as-exported.json";
pub const ROUNDED_COPY_SCHEMA: &str = "uor-r4.rounded-as-exported/1";

/// The step files a copy rounded as exported takes, each with what the copy does to it. The copy
/// refuses any other file, which it would otherwise drop.
const ROUNDED_COPY_FILES: [(&str, &str); 5] = [
    ("config.json", "none: copied byte for byte"),
    (
        "model.safetensors",
        "grid codes for the RMS gains, the convolution taps and the final norm; Q16 for the \
         convolution and gate biases and the decay offsets; Q12 for the embedding; the FBS \
         query points as they are, since the routes are exact",
    ),
    (
        "hamming_read.safetensors",
        "the rotations at 2^-24, the margins at 2^-16, the weights at 2^-24",
    ),
    (
        "turn_memory.safetensors",
        "the opening weight at 2^-24; the keys and stored tokens as they are",
    ),
    (
        "turn_memory_reply.safetensors",
        "the reply weight at 2^-24; the keys and stored tokens as they are",
    ),
];

/// One file of a [`RoundedCopy`]: its name, its sha256 in the source and in the copy, and what
/// the copy did to it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopiedFile {
    pub name: String,
    pub source_sha256: String,
    pub copy_sha256: String,
    pub rounding: String,
}

/// claude-252, Q5's side T: the marker of an evaluation copy of a checkpoint whose parameters are
/// rounded as `fbs-export --raw-gates --raw-transition` rounds them, so the float path computes
/// with the values R's artifact holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundedCopy {
    pub schema: String,
    /// The source step directory, canonical.
    pub source: String,
    /// The run's `config.json`, two levels above the step, copied byte for byte.
    pub run_config: CopiedFile,
    /// The step's files, in [`ROUNDED_COPY_FILES`] order.
    pub files: Vec<CopiedFile>,
}

fn file_sha256(path: &Path) -> Result<String> {
    Ok(hex::encode(Sha256::digest(std::fs::read(path)?)))
}

/// The run directory of a step directory, `RUN/models/step-N`.
fn run_of(step: &Path) -> Result<PathBuf> {
    step.parent()
        .and_then(Path::parent)
        .filter(|run| !run.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .ok_or_else(|| invalid(format!("{} is not a run's models/step-N", step.display())))
}

/// A table's values as `f32::to_bits`, so a comparison is bit for bit.
fn bits(var: &Var) -> Result<Vec<u32>> {
    Ok(var
        .as_tensor()
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(f32::to_bits)
        .collect())
}

/// Each parameter of the reads and the memory, in a fixed order, bit for bit.
fn read_values(reads: Option<&HammingReads>, memory: Option<&TurnMemory>) -> Result<Vec<Vec<u32>>> {
    let mut vars: Vec<&Var> = Vec::new();
    for read in reads.map_or(&[][..], |r| r.reads.as_slice()) {
        vars.extend([
            &read.query_rotation,
            &read.key_rotation,
            &read.value_rotation,
            &read.margin,
            &read.weight,
        ]);
    }
    vars.extend(memory.map(TurnMemory::variables).unwrap_or_default());
    vars.into_iter().map(bits).collect()
}

/// A memory's key length, opening table and reply table, which a copy keeps as they are.
type MemoryTables = Option<(usize, Vec<(Vec<u32>, u32)>, Option<Vec<(Vec<u32>, u32)>>)>;

fn memory_tables(memory: Option<&TurnMemory>) -> Result<MemoryTables> {
    memory
        .map(|m| Ok((m.k(), m.opening_entries(), m.reply_parts()?.map(|(e, _)| e))))
        .transpose()
}

/// claude-252: write an evaluation copy of the checkpoint `model_dir` (`RUN/models/step-N`) to
/// `out_dir` (`OUT_RUN/models/step-N`, the same step), with every parameter the integer artifact
/// holds rounded as the export rounds it: the trunk's tables ([`round_tables_as_exported`]), the
/// Hamming reads ([`round_hamming_as_exported`]) and the turn memory's weights
/// ([`round_memory_as_exported`]). The step's `config.json` and the run's are copied byte for
/// byte. Everything is read, checked and rounded on the CPU before any file is written:
/// - an `OUT_RUN` that exists is refused. The staging folder `OUT_RUN.partial` is claimed with
///   one `create_dir`, which refuses a folder that exists, so two copies never share one;
/// - a step file the copy does not know, and whatever the export refuses in the step, the run's
///   rate reading or the trunk's configuration, is refused;
/// - the copy is written to the staging folder, read back with every check the loaders make, and
///   refused unless it holds the rounded values bit for bit, re-rounding it changes nothing, its
///   memory tables are the source's, and it holds the source's files;
/// - the marker [`ROUNDED_COPY_MARKER`] is written last, `OUT_RUN` is checked again, and the
///   staging folder is then renamed to it. On any refusal after the claim, the staging folder is
///   removed, so no partial copy is ever left; a folder the claim refused is never touched.
pub fn write_rounded_copy(model_dir: &Path, out_dir: &Path) -> Result<RoundedCopy> {
    let (run, out_run) = (run_of(model_dir)?, run_of(out_dir)?);
    if out_dir.file_name() != model_dir.file_name() {
        return Err(invalid(
            "the copy keeps its step's name, from which the scorer reads the step",
        ));
    }
    let mut staging = out_run.clone().into_os_string();
    staging.push(".partial");
    let staging = PathBuf::from(staging);
    if out_run.symlink_metadata().is_ok() {
        return Err(exists(&out_run));
    }
    if let Some(parent) = staging.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::create_dir(&staging) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return Err(exists(&staging)),
        claimed => claimed?,
    }
    let written = copy_into(model_dir, out_dir, &run, &out_run, &staging);
    if written.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    written
}

/// The copy's refusal of an output or staging folder that exists.
fn exists(place: &Path) -> crate::FloatTrainError {
    invalid(format!(
        "{} exists, and the copy never replaces one",
        place.display()
    ))
}

/// [`write_rounded_copy`] once its staging folder is claimed: read, check and round the step,
/// stage the copy, and rename it into place.
fn copy_into(
    model_dir: &Path,
    out_dir: &Path,
    run: &Path,
    out_run: &Path,
    staging: &Path,
) -> Result<RoundedCopy> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(model_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !entry.file_type()?.is_file() || !ROUNDED_COPY_FILES.iter().any(|(n, _)| *n == name) {
            return Err(invalid(format!(
                "the step holds {name}, which the copy does not know and would drop"
            )));
        }
        names.push(name);
    }
    names.sort();
    check_checkpoint_reads(model_dir)?;
    let run_config = std::fs::read(run.join("config.json"))?;
    run_rate_reading(&String::from_utf8_lossy(&run_config))?;
    let device = candle_core::Device::Cpu;
    let model = StackModel::load(model_dir, &device)?;
    check_fbs_export_config(
        &model.config,
        model.quaternion_weights(),
        model.native_vector_head_spec(),
    )?;
    let reads = HammingReads::load(model_dir, &device)?;
    let memory = TurnMemory::load(model_dir, &device)?;
    round_tables_as_exported(&model)?;
    if let Some(reads) = &reads {
        round_hamming_as_exported(reads)?;
    }
    if let Some(memory) = &memory {
        round_memory_as_exported(memory)?;
    }
    let step = staging.join(
        out_dir
            .strip_prefix(out_run)
            .map_err(|e| invalid(e.to_string()))?,
    );
    let marker = StagedCopy {
        model_dir,
        run,
        staging,
        step: &step,
        run_config: &run_config,
        names: &names,
        model: &model,
        reads: reads.as_ref(),
        memory: memory.as_ref(),
    }
    .write()?;
    // Checked again: a rename replaces an empty folder made since the first check.
    if out_run.symlink_metadata().is_ok() {
        return Err(exists(out_run));
    }
    std::fs::rename(staging, out_run)?;
    Ok(marker)
}

/// A checkpoint read and rounded on the CPU, and where [`write_rounded_copy`] stages its copy.
struct StagedCopy<'a> {
    model_dir: &'a Path,
    run: &'a Path,
    staging: &'a Path,
    step: &'a Path,
    run_config: &'a [u8],
    /// The step's files, sorted.
    names: &'a [String],
    model: &'a StackModel,
    reads: Option<&'a HammingReads>,
    memory: Option<&'a TurnMemory>,
}

impl StagedCopy<'_> {
    /// Write the copy to the staging folder, read it back with every check the loaders make, and
    /// write its marker last.
    fn write(&self) -> Result<RoundedCopy> {
        let (step, device) = (self.step, candle_core::Device::Cpu);
        std::fs::create_dir_all(step)?;
        std::fs::write(self.staging.join("config.json"), self.run_config)?;
        std::fs::copy(self.model_dir.join("config.json"), step.join("config.json"))?;
        let tensors: std::collections::HashMap<String, Tensor> = self
            .model
            .variables()
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        candle_core::safetensors::save(&tensors, step.join("model.safetensors"))?;
        if let Some(reads) = self.reads {
            reads.save(step)?;
        }
        if let Some(memory) = self.memory {
            memory.save(step)?;
        }
        // Read back: the copy holds the rounded values bit for bit, they are their own rounding,
        // and its memory tables and files are the source's.
        let trunk = |m: &StackModel| -> Result<Vec<(String, Vec<u32>)>> {
            m.variables()
                .iter()
                .map(|(name, var)| Ok((name.clone(), bits(var)?)))
                .collect()
        };
        let copy = StackModel::load(step, &device)?;
        check_tables_rounded_as_exported(&copy)?;
        let reads = HammingReads::load(step, &device)?;
        let memory = TurnMemory::load(step, &device)?;
        let held = read_values(reads.as_ref(), memory.as_ref())?;
        if let Some(reads) = &reads {
            round_hamming_as_exported(reads)?;
        }
        if let Some(memory) = &memory {
            round_memory_as_exported(memory)?;
        }
        if trunk(&copy)? != trunk(self.model)?
            || held != read_values(self.reads, self.memory)?
            || held != read_values(reads.as_ref(), memory.as_ref())?
            || memory_tables(memory.as_ref())? != memory_tables(self.memory)?
        {
            return Err(invalid(
                "the copy does not read back as the rounded checkpoint (an instrument fault)",
            ));
        }
        let mut copied: Vec<String> = std::fs::read_dir(step)?
            .map(|e| Ok(e?.file_name().to_string_lossy().into_owned()))
            .collect::<Result<_>>()?;
        copied.sort();
        if copied != self.names {
            return Err(invalid(format!(
                "the copy holds {copied:?}, not the step's {:?}",
                self.names
            )));
        }
        let file = |name: &str, source: &Path, copy: &Path, rounding: &str| -> Result<CopiedFile> {
            Ok(CopiedFile {
                name: name.to_owned(),
                source_sha256: file_sha256(source)?,
                copy_sha256: file_sha256(copy)?,
                rounding: rounding.to_owned(),
            })
        };
        let marker = RoundedCopy {
            schema: ROUNDED_COPY_SCHEMA.to_owned(),
            source: std::fs::canonicalize(self.model_dir)?
                .to_string_lossy()
                .into_owned(),
            run_config: file(
                "config.json",
                &self.run.join("config.json"),
                &self.staging.join("config.json"),
                "none: copied byte for byte",
            )?,
            files: ROUNDED_COPY_FILES
                .iter()
                .filter(|(name, _)| self.names.iter().any(|n| n == name))
                .map(|(name, rounding)| {
                    file(name, &self.model_dir.join(name), &step.join(name), rounding)
                })
                .collect::<Result<_>>()?,
        };
        write_new_file(
            &step.join(ROUNDED_COPY_MARKER),
            &serde_json::to_vec_pretty(&marker)?,
        )?;
        Ok(marker)
    }
}

/// claude-252: the decision-trace side of the checkpoint `dir`: `T` when it is an evaluation copy
/// rounded as exported ([`write_rounded_copy`]), `F` when it holds no marker. A marker that does
/// not parse, or a copy whose files or run config are not the ones its marker names, is refused.
/// The rounding itself is then checked again, not taken from the marker: the trunk's tables, and
/// the reads and memory unchanged by re-rounding ([`check_rounded_as_exported`]). So a trace on
/// side `T` is always of the rounded parameters, whoever wrote the marker.
pub fn checkpoint_trace_side(dir: &Path) -> Result<&'static str> {
    let path = dir.join(ROUNDED_COPY_MARKER);
    if path.symlink_metadata().is_err() {
        return Ok("F");
    }
    let marker: RoundedCopy = serde_json::from_slice(&std::fs::read(&path)?)?;
    if marker.schema != ROUNDED_COPY_SCHEMA {
        return Err(invalid(format!(
            "{}'s schema is {}, not {ROUNDED_COPY_SCHEMA}",
            path.display(),
            marker.schema
        )));
    }
    let mut held: Vec<String> = std::fs::read_dir(dir)?
        .map(|e| Ok(e?.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_>>()?;
    held.retain(|name| name != ROUNDED_COPY_MARKER);
    held.sort();
    let mut named: Vec<String> = marker.files.iter().map(|f| f.name.clone()).collect();
    named.sort();
    if held != named {
        return Err(invalid(format!(
            "the rounded copy holds {held:?}, not the {named:?} its marker names"
        )));
    }
    let run = run_of(&std::fs::canonicalize(dir)?)?;
    for (file, at) in marker
        .files
        .iter()
        .map(|f| (f, dir.join(&f.name)))
        .chain(std::iter::once((
            &marker.run_config,
            run.join("config.json"),
        )))
    {
        if file_sha256(&at)? != file.copy_sha256 {
            return Err(invalid(format!(
                "{} is not the file the rounded copy's marker names",
                at.display()
            )));
        }
    }
    check_rounded_as_exported(dir)?;
    Ok("T")
}

/// Refuse the checkpoint `dir` unless every parameter the artifact holds is its own export
/// rounding: its trunk's tables ([`check_tables_rounded_as_exported`]), and its Hamming reads and
/// turn memory's weights, which re-rounding must leave bit for bit as they are. The copy's own
/// read-back makes the same checks and more ([`StagedCopy::write`]).
fn check_rounded_as_exported(dir: &Path) -> Result<()> {
    let device = candle_core::Device::Cpu;
    check_tables_rounded_as_exported(&StackModel::load(dir, &device)?)?;
    let reads = HammingReads::load(dir, &device)?;
    let memory = TurnMemory::load(dir, &device)?;
    let held = read_values(reads.as_ref(), memory.as_ref())?;
    if let Some(reads) = &reads {
        round_hamming_as_exported(reads)?;
    }
    if let Some(memory) = &memory {
        round_memory_as_exported(memory)?;
    }
    if held != read_values(reads.as_ref(), memory.as_ref())? {
        return Err(invalid(format!(
            "{}'s reads or memory are not as the export rounds them",
            dir.display()
        )));
    }
    Ok(())
}

/// Refuse a trunk whose tables are not as the exported artifact holds them
/// ([`round_tables_as_exported`]): only on those does the D11 mode compute
/// what the artifact computes.
pub fn check_tables_rounded_as_exported(model: &StackModel) -> Result<()> {
    for (name, round) in exported_roundings(model) {
        let (_, values, rounded) = table_and_rounding(model, &name, round)?;
        if let Some(i) = (0..values.len()).find(|&i| values[i] != rounded[i]) {
            return Err(invalid(format!(
                "{name}[{i}] is {}, not its exported {}: the D11 mode needs the tables rounded \
                 as exported first (round_tables_as_exported)",
                values[i], rounded[i]
            )));
        }
    }
    Ok(())
}

/// A table value's exported rounding.
type Rounding = fn(f32) -> Result<f32>;

/// A trunk table, its values and their exported rounding.
type RoundedTable = (candle_core::Var, Vec<f32>, Vec<f32>);

/// Each table the exported artifact holds rounded, with its rounding: grid
/// codes for the RMS gains and the convolution taps, Q16 for the biases and
/// decay offsets, Q12 for the embedding.
fn exported_roundings(model: &StackModel) -> Vec<(String, Rounding)> {
    fn grid(v: f32) -> Result<f32> {
        Ok(grid_value(grid_code(f64::from(v), false).map_err(invalid)?) as f32)
    }
    fn fixed(v: f32) -> Result<f32> {
        Ok(fixed_value(v, FBS_FIXED_EXP) as f32)
    }
    fn q12(v: f32) -> Result<f32> {
        q12_value(v)
            .map(|q| q as f32)
            .ok_or_else(|| invalid(format!("embedding value {v} does not fit Q12 in i16")))
    }
    let mut tables: Vec<(String, Rounding)> = Vec::new();
    for layer in 0..model.config.layers() {
        for name in ["rec_norm.weight", "mlp_norm.weight", "rec.conv.weight"] {
            tables.push((format!("layers.{layer:02}.{name}"), grid));
        }
        for name in ["rec.conv.bias", "rec.gate.bias", "rec.decay"] {
            tables.push((format!("layers.{layer:02}.{name}"), fixed));
        }
    }
    tables.push(("final_norm.weight".to_owned(), grid));
    tables.push(("embedding.weight".to_owned(), q12));
    tables
}

/// The trunk's table `name`, its values and their exported rounding.
fn table_and_rounding(model: &StackModel, name: &str, round: Rounding) -> Result<RoundedTable> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?
        .clone();
    let values = finite_table(name, var.as_tensor().flatten_all()?.to_vec1::<f32>()?)?;
    let rounded = values.iter().map(|&v| round(v)).collect::<Result<_>>()?;
    Ok((var, values, rounded))
}

/// The float GPU trunk's own output for every single-token input and for
/// [`READOUT_SEQUENCES`] fixed full-context sequences: its final hidden
/// vector and its native writer's next token. `fbs-compare` holds the
/// converted snapshot against it. GPU only.
/// - [`FbsReadoutMode::Export`]: both Hamilton gates in the float sheets' D11
///   export mode (Q12-rounded, then snapped to the 120 icosians).
/// - [`FbsReadoutMode::D11`]: the gates and the transition snapped to the 120
///   icosians directly, as the integer path does, and the final hidden
///   vector rounded to Q12 before the writer. The caller rounds the tables
///   first ([`round_tables_as_exported`]); an unrounded trunk is refused.
pub fn fbs_float_readout(model: &StackModel, mode: FbsReadoutMode) -> Result<FbsFloatReadout> {
    // Eight fixed full-context sequences: past the first position they reach
    // the transition, the later convolution taps and the carried state.
    let (vocab, time) = (model.config.vocab_size, model.config.context);
    let sequences: Vec<Vec<u32>> = (0..READOUT_SEQUENCES)
        .map(|s| {
            (0..time)
                .map(|t| ((s * 37 + t * 11 + 1) % vocab) as u32)
                .collect()
        })
        .collect();
    fbs_float_readout_on(model, mode, &sequences)
}

/// [`fbs_float_readout`] over chosen `sequences` in place of the fixed ones
/// (claude-126: a sample of DEV windows, so the parity covers real text).
pub fn fbs_float_readout_on(
    model: &StackModel,
    mode: FbsReadoutMode,
    sequences: &[Vec<u32>],
) -> Result<FbsFloatReadout> {
    check_no_sheet_arm()?;
    if mode == FbsReadoutMode::D11 {
        check_tables_rounded_as_exported(model)?;
    }
    let read = || read_float_outputs(model, mode == FbsReadoutMode::D11, sequences);
    let readout = match mode {
        FbsReadoutMode::Plain => read(),
        FbsReadoutMode::Export => crate::hamilton_sheet::with_d11_export_gates(read),
        FbsReadoutMode::D11 => crate::hamilton_sheet::with_d11_mode(read),
    }?;
    Ok(FbsFloatReadout { mode, ..readout })
}

/// Run a diagnostic forward under the complete served surrogate contract:
/// unsnapped gates and transition, Q12 query, Q16 residual, dynamic i16
/// activations and Q16 projection outputs. Every flag is restored after
/// success, error or panic by the underlying guard.
pub fn with_served_readout_contract<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    crate::hamilton_sheet::with_served_surrogate_training(0, 0, f)
}

/// The served-contract D11 readout over chosen sequences. The caller must
/// round the trunk's tables as exported first, exactly as for D11.
pub fn fbs_float_readout_served_on(
    model: &StackModel,
    sequences: &[Vec<u32>],
) -> Result<FbsFloatReadout> {
    let mut readout = with_served_readout_contract(|| {
        fbs_float_readout_on(model, FbsReadoutMode::D11, sequences)
    })?;
    readout.served_readout = true;
    Ok(readout)
}

/// [`fbs_float_readout_served_on`] on the fixed readout sequences.
pub fn fbs_float_readout_served(model: &StackModel) -> Result<FbsFloatReadout> {
    let (vocab, time) = (model.config.vocab_size, model.config.context);
    let sequences: Vec<Vec<u32>> = (0..READOUT_SEQUENCES)
        .map(|s| {
            (0..time)
                .map(|t| ((s * 37 + t * 11 + 1) % vocab) as u32)
                .collect()
        })
        .collect();
    fbs_float_readout_served_on(model, &sequences)
}

/// The native writer's next token at every position of each of `sequences`,
/// each read alone, as [`fbs_float_readout_on`] reads them but without the
/// hidden vectors (claude-126: the D11 tokens of every DEV window, held
/// against the panel's). GPU only.
pub fn fbs_float_next_tokens(
    model: &StackModel,
    mode: FbsReadoutMode,
    sequences: &[Vec<u32>],
) -> Result<Vec<Vec<u32>>> {
    check_no_sheet_arm()?;
    if mode == FbsReadoutMode::D11 {
        check_tables_rounded_as_exported(model)?;
    }
    let q12_query = mode == FbsReadoutMode::D11;
    let read = || -> Result<Vec<Vec<u32>>> {
        sequences
            .iter()
            .map(|tokens| Ok(hidden_and_next(model, tokens, q12_query)?.1))
            .collect()
    };
    match mode {
        FbsReadoutMode::Plain => read(),
        FbsReadoutMode::Export => crate::hamilton_sheet::with_d11_export_gates(read),
        FbsReadoutMode::D11 => crate::hamilton_sheet::with_d11_mode(read),
    }
}

/// The hidden vectors of `tokens` (one sequence), rounded to Q12 when
/// `q12_query`, with the native writer's next token for each.
fn hidden_and_next(
    model: &StackModel,
    tokens: &[u32],
    q12_query: bool,
) -> Result<(Vec<Vec<f32>>, Vec<u32>)> {
    let mut hidden = model.hidden(tokens, 1, tokens.len())?;
    if q12_query {
        hidden = hidden
            .affine(4096.0, 0.0)?
            .round()?
            .clamp(-32768f32, 32767f32)?
            .affine(1.0 / 4096.0, 0.0)?;
    }
    let next = model
        .native_vector_writer()?
        .write(&hidden)?
        .flatten_all()?
        .to_vec1::<u32>()?;
    if next.len() != tokens.len() {
        return Err(invalid(
            "the native writer returned other than one token per position",
        ));
    }
    let hidden = hidden
        .to_vec2::<f32>()?
        .into_iter()
        .map(|h| finite_table("hidden", h))
        .collect::<Result<_>>()?;
    Ok((hidden, next))
}

fn read_float_outputs(
    model: &StackModel,
    q12_query: bool,
    sequences: &[Vec<u32>],
) -> Result<FbsFloatReadout> {
    let (vocab, width) = (model.config.vocab_size, model.config.width);
    let mut positions = Vec::with_capacity(vocab);
    for token in 0..vocab as u32 {
        let (mut hidden, next) = hidden_and_next(model, &[token], q12_query)?;
        positions.push(FbsFloatPosition {
            token,
            next: next[0],
            hidden: hidden.remove(0),
        });
    }
    let mut read = Vec::with_capacity(sequences.len());
    for tokens in sequences {
        let (hidden, next) = hidden_and_next(model, tokens, q12_query)?;
        read.push(FbsFloatSequence {
            tokens: tokens.clone(),
            next,
            hidden,
        });
    }
    Ok(FbsFloatReadout {
        vocab,
        width,
        mode: FbsReadoutMode::Plain,
        served_readout: false,
        positions,
        sequences: read,
    })
}

/// The fixed sequences [`fbs_float_readout`] reads, each the full context.
pub const READOUT_SEQUENCES: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::Objective;

    /// claude-226: a run's rate reading is its `retain_rates`, false when absent. A run with
    /// `fold_scores` or `reflect_floor` on is refused, since the integer artifact only clamps the rate
    /// score.
    #[test]
    fn the_run_rate_reading_is_retain_rates_and_refuses_the_other_score_forms() {
        assert!(run_rate_reading(r#"{"retain_rates": true, "fold_scores": false, "reflect_floor": false}"#).unwrap());
        assert!(!run_rate_reading(r#"{"steps": 16384}"#).unwrap(), "absent, as before 27 Oct");
        assert!(run_rate_reading(r#"{"retain_rates": true, "fold_scores": true}"#).is_err(), "folded scores");
        assert!(run_rate_reading(r#"{"reflect_floor": true}"#).is_err(), "a reflected floor");
        assert!(run_rate_reading("not json").is_err());
    }

    /// The Stage 3 trunk's configuration at a small size.
    fn small_config(seed: u64) -> StackConfig {
        let mut config = StackConfig::transformer_control(seed);
        config.arch = StackArch::Geometric;
        config.pattern = "rr".into();
        config.width = 32;
        config.heads = 4;
        config.mlp_hidden = 32;
        config.context = 8;
        config.vocab_size = 256;
        config.rotation = true;
        config.rotation_group = RotationGroup::Quaternion;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 4 };
        config
    }

    fn small_head() -> WriterSpec {
        WriterSpec {
            vocabulary: 256,
            width: 32,
            kept_cells: 2,
        }
    }

    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }
    }

    /// Apply `routes` to `x` (`in_blocks * 8` values) on the host with the
    /// integer table: the gather the integer session serves.
    fn host_gather(routes: &FbsRoutes, x: &[f32]) -> Vec<f32> {
        let axes = spiralcore_signed_axes();
        let mut y = Vec::with_capacity(routes.out_blocks * 8);
        for (&atom, &state) in routes.atoms.iter().zip(&routes.states) {
            for &code in &axes[usize::from(state) * 8..usize::from(state) * 8 + 8] {
                let v = x[usize::from(atom) * 8 + usize::from(code & 7)];
                y.push(if code & 8 != 0 { -v } else { v });
            }
        }
        y
    }

    /// claude-198: a saved turn memory snapshots exactly as it loads: both tables in ascending key
    /// order, both weights, the key window, and the same JSON; a memory without a reply table has
    /// none, and a weight that is not finite is refused.
    #[test]
    fn a_saved_memory_snapshots_as_it_loads() -> Result<()> {
        use std::collections::HashMap;
        let device = candle_core::Device::Cpu;
        let scratch = Scratch::new("claude-198-memory");
        let dir = &scratch.0;
        let pad = uor_r4_integer::fbs_snapshot::MEMORY_PAD;
        let openings: HashMap<Vec<u32>, u32> =
            [(vec![9, 4], 6), (vec![3], 7), (vec![3, 4], 8)].into_iter().collect();
        let replies: HashMap<Vec<u32>, u32> = [
            (vec![3, 4, pad, 1, pad, 6], 7),
            (vec![pad, 2, pad, 9], 8),
            (vec![3, pad, 5, pad, 2], 1),
        ]
        .into_iter()
        .collect();
        let memory = TurnMemory::new(2, openings, 4, &device)?.with_reply(replies)?;
        memory
            .weight
            .set(&Tensor::new(&[0.25f32, -0.5, 1.0, 0.0], &device)?)?;
        memory
            .reply
            .as_ref()
            .expect("a reply table")
            .weight
            .set(&Tensor::new(&[-1.0f32, 0.125, 0.0, 2.0], &device)?)?;
        memory.save(dir)?;
        let loaded = TurnMemory::load(dir, &device)?.expect("saved");
        let before = fbs_memory_snapshot(&memory, 192)?;
        let after = fbs_memory_snapshot(&loaded, 192)?;
        assert_eq!(before, after);
        assert_eq!(serde_json::to_vec(&before)?, serde_json::to_vec(&after)?);
        assert_eq!((after.k, after.key_window), (2, 192));
        let keys: Vec<&[u32]> = after.opening.iter().map(|e| e.key.as_slice()).collect();
        assert_eq!(keys, [&[3][..], &[3, 4], &[9, 4]]);
        assert_eq!(after.opening[1].value, 8);
        assert_eq!(after.opening_weight, [0.25, -0.5, 1.0, 0.0]);
        let reply = after.reply.as_ref().expect("the reply table");
        assert!(reply.entries.windows(2).all(|w| w[0].key < w[1].key));
        assert_eq!((reply.turn, reply.entries.len()), (8, 3));
        assert_eq!(reply.weight, [-1.0, 0.125, 0.0, 2.0]);
        let parsed: FbsMemorySnapshot = serde_json::from_slice(&serde_json::to_vec(&after)?)?;
        assert_eq!(parsed, after);

        let plain = TurnMemory::new(2, HashMap::new(), 4, &device)?;
        assert!(fbs_memory_snapshot(&plain, 192)?.reply.is_none());
        plain
            .weight
            .set(&Tensor::new(&[0.0f32, f32::NAN, 0.0, 0.0], &device)?)?;
        assert!(fbs_memory_snapshot(&plain, 192).is_err());
        Ok(())
    }

    /// A test's scratch directory, emptied before use and removed when dropped, so a failed run
    /// leaves nothing behind for the next.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// claude-235: the exponents the rounding uses are the integer format's own; claude-252 adds
    /// the memory's.
    #[test]
    fn the_hamming_exponents_are_the_integer_formats() {
        let format = include_str!("../../uor-r4-integer/src/stack/format.rs");
        for (name, exp) in [
            ("HAMMING_ROTATION_EXP", HAMMING_ROTATION_EXP),
            ("HAMMING_MARGIN_EXP", HAMMING_MARGIN_EXP),
            ("HAMMING_WEIGHT_EXP", HAMMING_WEIGHT_EXP),
            ("MEMORY_WEIGHT_EXP", MEMORY_WEIGHT_EXP),
        ] {
            let line = format!("pub const {name}: i32 = {exp};");
            assert!(format.contains(&line), "stack/format.rs has no `{line}`");
        }
    }

    /// A Hamming read's five tables, in the order the rounding takes them, and their exponents.
    fn hamming_tables(read: &crate::flock_read_hamming::HammingRead) -> [(&Var, i32); 5] {
        [
            (&read.query_rotation, HAMMING_ROTATION_EXP),
            (&read.key_rotation, HAMMING_ROTATION_EXP),
            (&read.value_rotation, HAMMING_ROTATION_EXP),
            (&read.margin, HAMMING_MARGIN_EXP),
            (&read.weight, HAMMING_WEIGHT_EXP),
        ]
    }

    /// claude-235: rounding the reads as exported puts every parameter on its export step, the
    /// nearest one, half away from zero as `quantize_fixed_i32` rounds; leaves a value already on
    /// its step alone (a table value of 1 or more at `2^-24` is); holds the result exactly in
    /// `f32` (a margin of 1 or more still rounds at `2^-16`); and is idempotent. A read with the
    /// type spin, a value that is not finite, one past `i32` at its step and a margin that rounds
    /// to 0 are refused, with no value changed.
    #[test]
    fn rounding_the_hamming_reads_holds_the_exported_values() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let (heads, width) = (4, 32);
        let reads = HammingReads::new(heads, width, &device)?;
        let mut rng = Lcg(235);
        let values = |var: &Var| var.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let all = |reads: &HammingReads| -> Vec<Vec<f32>> {
            reads
                .reads
                .iter()
                .flat_map(|read| hamming_tables(read).map(|(var, _)| values(var)))
                .collect()
        };
        for read in &reads.reads {
            for (var, exp) in hamming_tables(read) {
                let margin = exp == HAMMING_MARGIN_EXP;
                let mut random: Vec<f32> = (0..var.elem_count())
                    .map(|_| (rng.next() % 2_000_001) as f32 / 1_000_003.0 - 1.0)
                    .map(|v| if margin { v.abs() + 0.001 } else { v })
                    .collect();
                // One value exactly half a step past a step, which rounds away from 0.
                random[0] = if margin { 3.5 } else { -3.5 } * 2f32.powi(exp);
                // One past 1: three quarters of a margin step past 1, or a table value one f32
                // spacing past 1, already on its 2^-24 step.
                random[1] = if margin {
                    1.0 + 3.0 * 2f32.powi(exp - 2)
                } else {
                    1.0 + 2f32.powi(-23)
                };
                var.set(&Tensor::from_vec(random, var.shape(), &device)?)?;
            }
        }
        let before = all(&reads);
        round_hamming_as_exported(&reads)?;
        let after = all(&reads);
        let exps = reads
            .reads
            .iter()
            .flat_map(|read| hamming_tables(read).map(|(_, exp)| exp));
        let mut moved = 0;
        for ((old, new), exp) in before.iter().zip(&after).zip(exps) {
            let step = 2f64.powi(exp);
            for (&o, &n) in old.iter().zip(new) {
                let (o, n) = (f64::from(o), f64::from(n));
                assert_eq!((n / step).fract(), 0.0, "{n} is off its 2^{exp} step");
                assert!(
                    (n - o).abs() <= step / 2.0,
                    "{o} went to {n}, past its nearest 2^{exp} step"
                );
                moved += usize::from(o != n);
            }
            let away = if exp == HAMMING_MARGIN_EXP { 4.0 } else { -4.0 };
            assert_eq!(
                new[0],
                away * 2f32.powi(exp),
                "half a step rounds away from 0"
            );
            if exp == HAMMING_MARGIN_EXP {
                assert_eq!(
                    new[1],
                    1.0 + 2f32.powi(exp),
                    "three quarters of a step past 1"
                );
            } else {
                assert_eq!(new[1], old[1], "on its step already");
            }
        }
        assert!(moved > 0, "no value moved, so the rounding had no work");
        round_hamming_as_exported(&reads)?;
        assert_eq!(all(&reads), after, "idempotent");

        let spun = HammingReads::new(heads, width, &device)?.with_type_spin()?;
        assert!(round_hamming_as_exported(&spun).is_err());
        let mut short = HammingReads::new(heads, width, &device)?;
        short.reads.pop();
        let err = round_hamming_as_exported(&short)
            .expect_err("a read short")
            .to_string();
        assert!(err.contains("not one at each depth"), "{err}");
        // Each refusal comes before any value changes: the first read's values sit off their
        // steps, and the planted value is in the last read.
        for (table, at, value, text) in [
            (1, 3, 200.0, "overflows i32"),
            (3, 2, 2f32.powi(HAMMING_MARGIN_EXP - 2), "not positive"),
            (4, 5, f32::NAN, "is not finite"),
        ] {
            let planted = HammingReads::new(heads, width, &device)?;
            for (r, read) in planted.reads.iter().enumerate() {
                for (k, (var, exp)) in hamming_tables(read).into_iter().enumerate() {
                    let mut v = vec![
                        if exp == HAMMING_MARGIN_EXP {
                            0.5
                        } else {
                            2f32.powi(exp - 2)
                        };
                        var.elem_count()
                    ];
                    if r + 1 == planted.reads.len() && k == table {
                        v[at] = value;
                    }
                    var.set(&Tensor::from_vec(v, var.shape(), &device)?)?;
                }
            }
            let held = all(&planted);
            let err = round_hamming_as_exported(&planted)
                .expect_err(text)
                .to_string();
            assert!(err.contains(text), "{err}");
            let unchanged = all(&planted).iter().zip(&held).all(|(a, b)| {
                a.iter()
                    .zip(b)
                    .all(|(x, y)| x == y || (x.is_nan() && y.is_nan()))
            });
            assert!(unchanged, "{text}: a value changed before the refusal");
        }
        Ok(())
    }

    /// claude-252: rounding the turn memory as exported puts its opening and reply weights on
    /// their `2^-24` step, the nearest one, half away from zero; leaves a weight already on its
    /// step alone; holds the result exactly in `f32`; leaves the tables as they are; and is
    /// idempotent. A memory without a reply table rounds its opening weight alone. A weight that
    /// is not finite, or past `i32` at its step, is refused in either table with no value changed.
    #[test]
    fn rounding_the_turn_memory_holds_the_exported_weights() -> Result<()> {
        use std::collections::HashMap;
        let device = candle_core::Device::Cpu;
        let (pad, width) = (uor_r4_integer::fbs_snapshot::MEMORY_PAD, 32);
        let memory = |reply: bool| -> Result<TurnMemory> {
            let openings: HashMap<Vec<u32>, u32> =
                [(vec![9, 4], 6), (vec![3], 7)].into_iter().collect();
            let memory = TurnMemory::new(2, openings, width, &device)?;
            if !reply {
                return Ok(memory);
            }
            memory.with_reply(
                [(vec![3, 4, pad, 1], 7), (vec![pad, 2], 8)]
                    .into_iter()
                    .collect(),
            )
        };
        let weights = |m: &TurnMemory| -> Vec<Vec<f32>> {
            m.variables()
                .into_iter()
                .map(|v| v.as_tensor().to_vec1::<f32>().unwrap())
                .collect()
        };
        let set = |m: &TurnMemory, values: &[Vec<f32>]| -> Result<()> {
            for (var, v) in m.variables().into_iter().zip(values) {
                var.set(&Tensor::from_vec(v.clone(), width, &device)?)?;
            }
            Ok(())
        };
        let step = 2f32.powi(MEMORY_WEIGHT_EXP);
        let mut rng = Lcg(252);
        // A `k/1_000_003 - 1` in f32 is on the 2^-24 grid already; a third of one mostly is not.
        let random: Vec<Vec<f32>> = (0..2)
            .map(|_| {
                let mut v: Vec<f32> = (0..width)
                    .map(|_| ((rng.next() % 2_000_001) as f32 / 1_000_003.0 - 1.0) / 3.0)
                    .collect();
                // Half a step past a step, which rounds away from 0 (to even, it would go to -2);
                // and one f32 spacing past 1, on its 2^-24 step already.
                v[0] = -2.5 * step;
                v[1] = 1.0 + 2f32.powi(-23);
                v
            })
            .collect();
        let full = memory(true)?;
        set(&full, &random)?;
        let tables = memory_tables(Some(&full))?;
        round_memory_as_exported(&full)?;
        let after = weights(&full);
        assert_eq!(after.len(), 2, "the opening and the reply weight");
        let mut moved = 0;
        for (old, new) in random.iter().zip(&after) {
            for (&o, &n) in old.iter().zip(new) {
                let (o, n, s) = (f64::from(o), f64::from(n), f64::from(step));
                assert_eq!((n / s).fract(), 0.0, "{n} is off its 2^-24 step");
                assert!(
                    (n - o).abs() <= s / 2.0,
                    "{o} went to {n}, past its nearest step"
                );
                moved += usize::from(o != n);
            }
            assert_eq!(new[0], -3.0 * step, "half a step rounds away from 0");
            assert_eq!(new[1], old[1], "on its step already");
        }
        assert!(moved > 0, "no weight moved, so the rounding had no work");
        assert_eq!(
            memory_tables(Some(&full))?,
            tables,
            "the tables are as they were"
        );
        round_memory_as_exported(&full)?;
        assert_eq!(weights(&full), after, "idempotent");
        let plain = memory(false)?;
        set(&plain, &random[..1])?;
        round_memory_as_exported(&plain)?;
        assert_eq!(
            weights(&plain),
            after[..1].to_vec(),
            "the opening weight alone"
        );
        // Each refusal, in either table, comes before any value changes: every weight sits off
        // its step.
        for (table, value, text) in [
            (0, f32::NAN, "is not finite"),
            (1, 200.0, "overflows i32"),
            (1, f32::INFINITY, "is not finite"),
        ] {
            let planted = memory(true)?;
            let mut held = vec![vec![step / 4.0; width]; 2];
            held[table][5] = value;
            set(&planted, &held)?;
            let err = round_memory_as_exported(&planted)
                .expect_err(text)
                .to_string();
            assert!(err.contains(text), "{err}");
            let unchanged = weights(&planted)
                .iter()
                .flatten()
                .zip(held.iter().flatten())
                .all(|(a, b)| a == b || (a.is_nan() && b.is_nan()));
            assert!(unchanged, "{text}: a weight changed before the refusal");
        }
        Ok(())
    }

    /// Each file of `dir` and its sha256, sorted.
    fn sums(dir: &Path) -> Result<Vec<(String, String)>> {
        let mut sums = std::fs::read_dir(dir)?
            .map(|e| {
                let e = e?;
                Ok((
                    e.file_name().to_string_lossy().into_owned(),
                    file_sha256(&e.path())?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        sums.sort();
        Ok(sums)
    }

    /// claude-252, Q5's side T: a copy rounded as exported holds every rounding. Its trunk tables,
    /// Hamming reads and memory weights are the source's rounded as exported, bit for bit, each
    /// group's rounding had work, and rounding the copy again changes nothing. Its memory tables,
    /// both configs and the source's files are as they were, and its marker names each file
    /// before and after. The source's side is F and the copy's T. The copy refuses an existing
    /// output or staging folder (leaving it as it was), another step's name, a step file it does
    /// not know and a value the export refuses, each leaving nothing behind. The copy's side is
    /// refused once a file or the run config is not the one its marker names, once it holds a
    /// file the marker does not name, and once the marker's schema is another.
    #[test]
    fn a_copy_rounded_as_exported_holds_every_rounding_and_is_side_t() -> Result<()> {
        use std::collections::HashMap;
        let device = candle_core::Device::Cpu;
        let scratch = Scratch::new("claude-252-copy");
        let root = &scratch.0;
        let source = root.join("run/models/step-000004");
        let mut rng = Lcg(252);
        // Off the 2^-24 grid, as in the memory's test, so every group's rounding has work.
        let mut random = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| ((rng.next() % 2_000_001) as f32 / 1_000_003.0 - 1.0) / 3.0)
                .collect()
        };
        let mut model = StackModel::new(small_config(252), &device)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(2)?;
        for (name, var) in model.variables() {
            if !name.ends_with(".spiralcore_fbs") {
                var.set(&Tensor::from_vec(
                    random(var.elem_count()),
                    var.shape(),
                    &device,
                )?)?;
            }
        }
        model.save(&source)?;
        let reads = HammingReads::new(4, 32, &device)?;
        for read in &reads.reads {
            for (var, exp) in hamming_tables(read) {
                let mut v = random(var.elem_count());
                if exp == HAMMING_MARGIN_EXP {
                    v.iter_mut().for_each(|x| *x = x.abs() + 0.001);
                }
                var.set(&Tensor::from_vec(v, var.shape(), &device)?)?;
            }
        }
        reads.save(&source)?;
        let pad = uor_r4_integer::fbs_snapshot::MEMORY_PAD;
        let openings: HashMap<Vec<u32>, u32> =
            [(vec![9, 4], 6), (vec![3], 7)].into_iter().collect();
        let memory = TurnMemory::new(2, openings, 32, &device)?
            .with_reply([(vec![3, 4, pad, 1], 7)].into_iter().collect())?;
        for var in memory.variables() {
            var.set(&Tensor::from_vec(random(32), 32, &device)?)?;
        }
        memory.save(&source)?;
        let run_config = br#"{"steps": 4, "retain_rates": true}"#;
        std::fs::write(root.join("run/config.json"), run_config)?;
        let source_sums = sums(&source)?;
        assert_eq!(checkpoint_trace_side(&source)?, "F");

        let out = root.join("t/models/step-000004");
        let marker = write_rounded_copy(&source, &out)?;
        assert_eq!(checkpoint_trace_side(&out)?, "T");
        assert_eq!(sums(&source)?, source_sums, "the source is as it was");
        assert!(
            !root.join("t.partial").exists(),
            "no staging folder is left"
        );
        let names: Vec<&str> = marker.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ROUNDED_COPY_FILES.map(|(name, _)| name));
        let saved: RoundedCopy =
            serde_json::from_slice(&std::fs::read(out.join(ROUNDED_COPY_MARKER))?)?;
        assert_eq!(saved, marker);
        for file in &marker.files {
            assert_eq!(file.copy_sha256, file_sha256(&out.join(&file.name))?);
            assert_eq!(file.source_sha256, file_sha256(&source.join(&file.name))?);
        }
        assert_eq!(
            std::fs::read(out.join("config.json"))?,
            std::fs::read(source.join("config.json"))?
        );
        assert_eq!(std::fs::read(root.join("t/config.json"))?, run_config);
        // The copy is the source rounded as exported, bit for bit, and every group moved.
        let trunk = |m: &StackModel| -> Result<Vec<Vec<u32>>> {
            m.variables().values().map(bits).collect()
        };
        let rounded = StackModel::load(&source, &device)?;
        let rounded_reads = HammingReads::load(&source, &device)?.expect("saved reads");
        let rounded_memory = TurnMemory::load(&source, &device)?.expect("a saved memory");
        let (unrounded_trunk, unrounded) = (
            trunk(&rounded)?,
            read_values(Some(&rounded_reads), Some(&rounded_memory))?,
        );
        round_tables_as_exported(&rounded)?;
        round_hamming_as_exported(&rounded_reads)?;
        round_memory_as_exported(&rounded_memory)?;
        let copy = StackModel::load(&out, &device)?;
        assert_eq!(trunk(&copy)?, trunk(&rounded)?);
        assert_ne!(
            trunk(&rounded)?,
            unrounded_trunk,
            "the trunk's rounding had work"
        );
        let copy_reads = HammingReads::load(&out, &device)?.expect("copied reads");
        let copy_memory = TurnMemory::load(&out, &device)?.expect("a copied memory");
        let held = read_values(Some(&copy_reads), Some(&copy_memory))?;
        assert_eq!(
            held,
            read_values(Some(&rounded_reads), Some(&rounded_memory))?
        );
        assert_eq!(held.len(), 5 * HAMMING_DEPTHS.len() + 2);
        for (i, (h, u)) in held.iter().zip(&unrounded).enumerate() {
            assert_ne!(h, u, "parameter {i}'s rounding had work");
        }
        check_tables_rounded_as_exported(&copy)?;
        round_hamming_as_exported(&copy_reads)?;
        round_memory_as_exported(&copy_memory)?;
        assert_eq!(
            read_values(Some(&copy_reads), Some(&copy_memory))?,
            held,
            "re-rounding the copy changes nothing"
        );
        assert_eq!(
            memory_tables(Some(&copy_memory))?,
            memory_tables(Some(&memory))?
        );

        // Refusals.
        let refused = |from: &Path, to: &Path, text: &str| {
            let err = write_rounded_copy(from, to).expect_err(text).to_string();
            assert!(err.contains(text), "{text}: {err}");
        };
        refused(&source, &out, "exists");
        assert_eq!(
            checkpoint_trace_side(&out)?,
            "T",
            "an existing copy is left as it was"
        );
        std::fs::create_dir_all(root.join("v.partial"))?;
        std::fs::write(root.join("v.partial/keep"), b"x")?;
        refused(&source, &root.join("v/models/step-000004"), "exists");
        assert!(
            root.join("v.partial/keep").exists(),
            "a staging folder it did not make stays"
        );
        let clone = |name: &str| -> Result<PathBuf> {
            let step = root.join(name).join("models/step-000004");
            std::fs::create_dir_all(&step)?;
            std::fs::write(root.join(name).join("config.json"), run_config)?;
            for (file, _) in &source_sums {
                std::fs::copy(source.join(file), step.join(file))?;
            }
            Ok(step)
        };
        let left =
            |name: &str| root.join(name).exists() || root.join(format!("{name}.partial")).exists();
        refused(
            &source,
            &root.join("w/models/step-000008"),
            "keeps its step's name",
        );
        assert!(!left("w"));
        let unknown = clone("run-unknown")?;
        std::fs::write(unknown.join("transport.json"), b"{}")?;
        refused(
            &unknown,
            &root.join("x/models/step-000004"),
            "does not know",
        );
        assert!(!left("x"));
        let diverged = clone("run-nan")?;
        memory
            .weight
            .set(&Tensor::from_vec(vec![f32::NAN; 32], 32, &device)?)?;
        memory.save(&diverged)?;
        refused(
            &diverged,
            &root.join("y/models/step-000004"),
            "is not finite",
        );
        assert!(!left("y"));
        let folded = clone("run-folded")?;
        std::fs::write(
            root.join("run-folded/config.json"),
            br#"{"fold_scores": true}"#,
        )?;
        refused(&folded, &root.join("z/models/step-000004"), "fold");
        assert!(!left("z"));

        // The side, once the copy is not the one its marker names.
        let side_refused = |text: &str| {
            let err = checkpoint_trace_side(&out).expect_err(text).to_string();
            assert!(err.contains(text), "{text}: {err}");
        };
        let tensors = out.join("model.safetensors");
        let bytes = std::fs::read(&tensors)?;
        let mut changed = bytes.clone();
        *changed.last_mut().expect("bytes") ^= 1;
        std::fs::write(&tensors, &changed)?;
        side_refused("is not the file");
        std::fs::write(&tensors, &bytes)?;
        std::fs::write(root.join("t/config.json"), b"{}")?;
        side_refused("is not the file");
        std::fs::write(root.join("t/config.json"), run_config)?;
        std::fs::write(out.join("extra.json"), b"{}")?;
        side_refused("holds");
        std::fs::remove_file(out.join("extra.json"))?;
        let marker_path = out.join(ROUNDED_COPY_MARKER);
        let text = std::fs::read_to_string(&marker_path)?;
        std::fs::write(
            &marker_path,
            text.replace(ROUNDED_COPY_SCHEMA, "uor-r4.rounded-as-exported/0"),
        )?;
        side_refused("schema");
        std::fs::write(&marker_path, &text)?;
        // A marker naming an unrounded file by its true sha256 is refused all the same: the side
        // checks the rounding itself.
        for (name, text_refused) in [
            ("model.safetensors", "not its exported"),
            ("turn_memory.safetensors", "not as the export rounds them"),
        ] {
            let kept = std::fs::read(out.join(name))?;
            std::fs::copy(source.join(name), out.join(name))?;
            let mut forged: RoundedCopy = serde_json::from_str(&text)?;
            for file in forged.files.iter_mut().filter(|f| f.name == name) {
                file.copy_sha256 = file_sha256(&out.join(name))?;
            }
            std::fs::write(&marker_path, serde_json::to_vec_pretty(&forged)?)?;
            side_refused(text_refused);
            std::fs::write(out.join(name), &kept)?;
            std::fs::write(&marker_path, &text)?;
        }
        assert_eq!(checkpoint_trace_side(&out)?, "T", "restored");
        Ok(())
    }

    /// claude-200: saved Hamming reads snapshot exactly as they load, row-major, at the depths
    /// `HAMMING_DEPTHS`, and the JSON reads back equal. A trunk of another depth, a read with the
    /// type spin, a table of another width and a value that is not finite are refused by text.
    #[test]
    fn saved_hamming_reads_snapshot_as_they_load() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let (heads, width) = (4, 32);
        let scratch = Scratch::new("claude-200-hamming");
        let reads = HammingReads::new(heads, width, &device)?;
        let mut rng = Lcg(200);
        for read in &reads.reads {
            for var in [
                &read.query_rotation,
                &read.key_rotation,
                &read.value_rotation,
                &read.margin,
                &read.weight,
            ] {
                let mut values: Vec<f32> = (0..var.elem_count())
                    .map(|_| (rng.next() % 2001) as f32 / 1000.0 - 1.0)
                    .collect();
                // Margins are in bits and positive, as the integer read requires.
                if std::ptr::eq(var, &read.margin) {
                    values.iter_mut().for_each(|m| *m = 1.0 + 8.0 * m.abs());
                }
                var.set(&Tensor::from_vec(values, var.shape(), &device)?)?;
            }
        }
        reads.save(&scratch.0)?;
        let loaded = HammingReads::load(&scratch.0, &device)?.expect("saved");
        let before = fbs_hamming_snapshot(&reads, HAMMING_READOUT_DEPTH, width)?;
        let after = fbs_hamming_snapshot(&loaded, HAMMING_READOUT_DEPTH, width)?;
        assert_eq!(before, after);
        assert_eq!((after.heads, after.depths.as_slice()), (heads, &HAMMING_DEPTHS[..]));
        for (snap, read) in after.reads.iter().zip(&reads.reads) {
            let values = |var: &Var| var.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            assert_eq!(snap.query_rotation, values(&read.query_rotation));
            assert_eq!(snap.key_rotation, values(&read.key_rotation));
            assert_eq!(snap.value_rotation, values(&read.value_rotation));
            assert_eq!(snap.margin, values(&read.margin));
            assert_eq!(snap.weight, values(&read.weight));
            assert_eq!(
                (snap.query_rotation.len(), snap.margin.len(), snap.weight.len()),
                (width, heads, 2 * width)
            );
        }
        let parsed: FbsHammingSnapshot = serde_json::from_slice(&serde_json::to_vec(&after)?)?;
        assert_eq!(parsed, after);

        let refused = |reads: &HammingReads, layers: usize, width: usize, reason: &str| {
            let message = fbs_hamming_snapshot(reads, layers, width)
                .expect_err(reason)
                .to_string();
            assert!(message.contains(reason), "{message}");
        };
        refused(&loaded, 4, width, "this trunk has 4 layers");
        refused(&loaded, HAMMING_READOUT_DEPTH, 64, "hamming_read.d2.query_rotation has shape");
        let spun = loaded.clone().with_type_spin()?;
        refused(&spun, HAMMING_READOUT_DEPTH, width, "the type spin (step_rotation)");
        let mut margin = loaded.reads[0].margin.flatten_all()?.to_vec1::<f32>()?;
        let kept = margin.clone();
        margin[2] = 0.0;
        loaded.reads[0]
            .margin
            .set(&Tensor::from_vec(margin, heads, &device)?)?;
        refused(&loaded, HAMMING_READOUT_DEPTH, width, "hamming_read.d2.margin[2] = 0 is not positive");
        loaded.reads[0]
            .margin
            .set(&Tensor::from_vec(kept, heads, &device)?)?;
        let mut weight = loaded.reads[1].weight.flatten_all()?.to_vec1::<f32>()?;
        weight[3] = f32::NAN;
        loaded.reads[1]
            .weight
            .set(&Tensor::from_vec(weight, (2, width), &device)?)?;
        refused(&loaded, HAMMING_READOUT_DEPTH, width, "hamming_read.d6.weight[3]");
        Ok(())
    }

    /// claude-198: a checkpoint of the trunk, its optimizer state and its turn memory exports;
    /// every other read `in_space_long` saves beside them is refused by name, because the export
    /// does not carry it yet.
    #[test]
    fn a_checkpoint_read_the_export_does_not_carry_is_refused() -> Result<()> {
        let accounted = [
            "model.safetensors",
            "optimizer.safetensors",
            "turn_memory.safetensors",
            "turn_memory_reply.safetensors",
            "hamming_read.safetensors",
            "config.json",
        ];
        // The four reads `in_space_long` saves that the export does not carry, and a read no code
        // writes yet: the check is an allow-list, so a read added later is refused too.
        let reads = [
            "flock_read.safetensors",
            "content_read.safetensors",
            "opening_read.safetensors",
            "prompt_register.safetensors",
            "future_read.safetensors",
        ];
        for read in reads {
            let scratch = Scratch::new("claude-198-reads");
            std::fs::create_dir_all(&scratch.0)?;
            for file in accounted {
                std::fs::write(scratch.0.join(file), b"")?;
            }
            check_checkpoint_reads(&scratch.0)?;
            std::fs::write(scratch.0.join(read), b"")?;
            let message = check_checkpoint_reads(&scratch.0)
                .expect_err(read)
                .to_string();
            assert!(message.contains(read), "{message}");
        }
        // Three reads at once, written in reverse order, are all named in name order.
        let scratch = Scratch::new("claude-198-reads");
        std::fs::create_dir_all(&scratch.0)?;
        for read in [
            "prompt_register.safetensors",
            "opening_read.safetensors",
            "flock_read.safetensors",
        ] {
            std::fs::write(scratch.0.join(read), b"")?;
        }
        let message = check_checkpoint_reads(&scratch.0)
            .expect_err("three reads")
            .to_string();
        assert!(
            message.contains(
                "flock_read.safetensors, opening_read.safetensors, prompt_register.safetensors"
            ),
            "{message}"
        );
        // Readouts of a checkpoint with a turn memory are refused (claude-200); the snapshot alone
        // is not, and a checkpoint without a memory may have readouts.
        let scratch = Scratch::new("claude-200-readouts");
        std::fs::create_dir_all(&scratch.0)?;
        check_readout_reads(&scratch.0, 3)?;
        std::fs::write(scratch.0.join("turn_memory.safetensors"), b"")?;
        check_readout_reads(&scratch.0, 0)?;
        let message = check_readout_reads(&scratch.0, 1)
            .expect_err("readouts beside a memory")
            .to_string();
        assert!(message.contains("do not apply the turn memory"), "{message}");
        // A reply table without its opening table is refused, not dropped.
        let scratch = Scratch::new("claude-198-reads");
        std::fs::create_dir_all(&scratch.0)?;
        for file in ["model.safetensors", "turn_memory_reply.safetensors"] {
            std::fs::write(scratch.0.join(file), b"")?;
        }
        let message = check_checkpoint_reads(&scratch.0)
            .expect_err("a reply table alone")
            .to_string();
        assert!(message.contains("without turn_memory.safetensors"), "{message}");
        Ok(())
    }

    #[test]
    fn geometry_tables_match_the_integer_crate() {
        check_geometry_tables().expect("the float geometry is the integer crate's");
        let plan = crate::spiralcore_fbs::ProjectionPlan::new(8, 8).expect("plan");
        let (axes, roots) = (&plan.geometry.axes, plan.geometry.roots);
        // A changed code and a reordered root are each refused.
        let mut changed = axes.clone();
        changed[9] ^= 8;
        assert!(compare_geometry_tables(&changed, &roots).is_err());
        let mut reordered = roots;
        reordered.swap(1, 2);
        assert!(compare_geometry_tables(axes, &reordered).is_err());
        assert!(compare_geometry_tables(&axes[..504], &roots).is_err());
    }

    #[test]
    fn coded_carrier_decode_recovers_a_gather_exactly() {
        let mut rng = Lcg(20261026);
        for (in_blocks, out_blocks) in [(1usize, 3usize), (4, 8), (4, 5), (32, 4)] {
            let routes = FbsRoutes {
                in_blocks,
                out_blocks,
                atoms: (0..out_blocks)
                    .map(|_| (rng.next() % in_blocks as u64) as u16)
                    .collect(),
                states: (0..out_blocks).map(|_| (rng.next() % 64) as u8).collect(),
            };
            let projected = host_gather(&routes, &coded_carrier(in_blocks));
            assert_eq!(
                decode_coded_routes(&projected, in_blocks, out_blocks).expect("decode"),
                routes
            );
        }
    }

    #[test]
    fn coded_carrier_decode_refuses_what_is_not_a_signed_gather() {
        let routes = FbsRoutes {
            in_blocks: 4,
            out_blocks: 2,
            atoms: vec![1, 3],
            states: vec![5, 40],
        };
        let good = host_gather(&routes, &coded_carrier(4));
        assert!(decode_coded_routes(&good, 4, 2).is_ok());
        type Mutation = fn(&mut Vec<f32>);
        let refused: [(&str, Mutation, &str); 6] = [
            (
                "two atoms in a block",
                |p| p[3] = if p[3] < 0.0 { -1.0 } else { 1.0 },
                "block 0 reads two atoms",
            ),
            (
                "a slot past the carrier",
                |p| p[9] = 33.0,
                "block 1 channel 1: 33 is not a coded input slot",
            ),
            ("a fractional value", |p| p[2] += 0.25, "block 0 channel 2:"),
            (
                "NaN",
                |p| p[0] = f32::NAN,
                "block 0 channel 0: NaN is not a coded input slot",
            ),
            (
                "zero",
                |p| p[15] = 0.0,
                "block 1 channel 7: 0 is not a coded input slot",
            ),
            (
                "a code row outside the group",
                |p| {
                    // Every channel reads axis 0 of atom 1 (slot 9): one atom, no permutation.
                    for v in &mut p[..8] {
                        *v = 9.0;
                    }
                },
                "block 0: codes [0, 0, 0, 0, 0, 0, 0, 0] are not a state of the Cl(0,6) table",
            ),
        ];
        for (what, mutate, reason) in refused {
            let mut projected = good.clone();
            mutate(&mut projected);
            match decode_coded_routes(&projected, 4, 2) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(r) => panic!("{what} must be refused, decoded {r:?}"),
            }
        }
        assert!(decode_coded_routes(&good[..8], 4, 2).is_err());
        assert!(decode_coded_routes(&good, 0, 2).is_err());
    }

    #[test]
    fn round_tables_as_exported_holds_the_exported_values() -> Result<()> {
        // A CPU-built trunk: the rounding reads and writes the tables only.
        let mut model = StackModel::new(small_config(5), &candle_core::Device::Cpu)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        let values = |m: &StackModel, name: &str| -> Result<Vec<f32>> {
            Ok(m.variables()[name]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?)
        };
        // A fresh trunk's gains are ones, already on the grid: give every
        // table random values off every grid first, so each rounding has work.
        let mut rng = Lcg(31);
        for (name, var) in model.variables() {
            if name.ends_with(".spiralcore_fbs") {
                continue;
            }
            let n = var.as_tensor().elem_count();
            let random: Vec<f32> = (0..n)
                .map(|_| (rng.next() % 2_000_001) as f32 / 1_000_003.0 - 1.0)
                .collect();
            var.set(&Tensor::from_vec(
                random,
                var.as_tensor().shape(),
                &candle_core::Device::Cpu,
            )?)?;
        }
        let before: Vec<Vec<f32>> = model
            .variables()
            .keys()
            .map(|name| values(&model, name))
            .collect::<Result<_>>()?;
        // Every D11 entry refuses the trunk until its tables are rounded; the
        // refusal comes before any tensor operation, so it holds on the host.
        use crate::hamilton_sheet::{evaluate_dev_panel_d11, is_snapping_transition};
        let unrounded =
            |e: crate::FloatTrainError| e.to_string().contains("rounded as exported first");
        assert!(check_tables_rounded_as_exported(&model).is_err_and(unrounded));
        assert!(fbs_float_readout(&model, FbsReadoutMode::D11).is_err_and(unrounded));
        assert!(evaluate_dev_panel_d11(&model, &[]).is_err_and(unrounded));
        round_tables_as_exported(&model)?;
        check_tables_rounded_as_exported(&model)?;
        // Rounded, the panel gets past its guard and stops at its first tensor
        // operation (CPU model compute is refused), leaving the hook as it was.
        let past_guard = evaluate_dev_panel_d11(&model, &[]).expect_err("no CPU compute");
        assert!(!unrounded(past_guard));
        assert!(!is_snapping_transition());
        for (name, b) in model.variables().keys().zip(&before) {
            // Every table changes; the FBS routes do not.
            let a = values(&model, name)?;
            assert_eq!(name.ends_with(".spiralcore_fbs"), &a == b, "{name}");
        }
        let grid = |v: f32| grid_value(grid_code(f64::from(v), false).expect("grid")) as f32;
        let fixed = |v: f32| fixed_value(v, FBS_FIXED_EXP) as f32;
        let q12 = |v: f32| q12_value(v).expect("q12") as f32;
        let mut tables: Vec<(String, &dyn Fn(f32) -> f32)> = vec![
            ("final_norm.weight".into(), &grid),
            ("embedding.weight".into(), &q12),
        ];
        for layer in 0..model.config.layers() {
            for (suffix, f) in [
                ("rec_norm.weight", &grid as &dyn Fn(f32) -> f32),
                ("mlp_norm.weight", &grid),
                ("rec.conv.weight", &grid),
                ("rec.conv.bias", &fixed),
                ("rec.gate.bias", &fixed),
                ("rec.decay", &fixed),
            ] {
                tables.push((format!("layers.{layer:02}.{suffix}"), f));
            }
        }
        for (name, f) in tables {
            // Each rounded value is its own exported rounding.
            for v in values(&model, &name)? {
                assert_eq!(f(v).to_bits(), v.to_bits(), "{name}: {v}");
            }
        }
        // An embedding value that does not fit Q12 in i16 is refused.
        let var = &model.variables()["embedding.weight"];
        let mut big = values(&model, "embedding.weight")?;
        big[0] = 9.0;
        var.set(&Tensor::from_vec(
            big,
            var.as_tensor().shape(),
            &candle_core::Device::Cpu,
        )?)?;
        assert!(round_tables_as_exported(&model).is_err_and(|e| e.to_string().contains("Q12")));
        Ok(())
    }

    #[test]
    fn d11_mode_snaps_the_transition_only_inside_its_closure() -> Result<()> {
        use crate::hamilton_sheet::{is_snapping_transition, with_d11_mode};
        assert!(!is_snapping_transition());
        assert!(with_d11_mode(|| Ok(is_snapping_transition()))?);
        assert!(!is_snapping_transition());
        let failed: Result<()> = with_d11_mode(|| Err(invalid("inside")));
        assert!(failed.is_err() && !is_snapping_transition());
        Ok(())
    }

    #[test]
    fn served_readout_contract_sets_and_restores_every_flag() -> Result<()> {
        use crate::hamilton_sheet::{
            is_bridging_activation_i16, is_bridging_d11, is_bridging_q16_residual,
            is_rounding_residual_q16, is_snapping_transition,
        };
        let state = || {
            (
                is_rounding_residual_q16(),
                is_bridging_q16_residual(),
                is_bridging_activation_i16(),
                is_bridging_d11(),
                is_snapping_transition(),
            )
        };
        let before = state();
        assert_eq!(
            with_served_readout_contract(|| Ok(state()))?,
            (true, true, true, true, false)
        );
        assert_eq!(state(), before);
        let failed: Result<()> = with_served_readout_contract(|| Err(invalid("inside")));
        assert!(failed.is_err());
        assert_eq!(state(), before);
        Ok(())
    }

    #[test]
    fn copied_tables_refuse_non_finite_values() {
        assert_eq!(
            finite_table("t", vec![1.0, -2.5]).expect("finite"),
            vec![1.0, -2.5]
        );
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let e = finite_table("layers.01.rec.decay", vec![0.5, bad]).expect_err("refused");
            assert!(e.to_string().contains("layers.01.rec.decay[1]"), "{e}");
        }
    }

    #[test]
    fn export_config_refuses_what_the_integer_session_cannot_serve() {
        let head = small_head();
        let fbs = QuaternionWeights::SpiralcoreFbs;
        assert!(check_fbs_export_config(&small_config(1), fbs, Some(&head)).is_ok());
        type Mutation = fn(&mut StackConfig);
        let refused: [(&str, Mutation, &str); 9] = [
            (
                "a transformer",
                |c| c.arch = StackArch::Transformer,
                "the trunk is not geometric",
            ),
            (
                "a read layer",
                |c| c.pattern = "ra".into(),
                "the integer FBS trunk serves recurrence layers only",
            ),
            (
                "historical gates",
                |c| c.rec_gates = RecGates::Historical,
                "the recurrence gates are not geometric",
            ),
            (
                "no rotations",
                |c| c.rotation = false,
                "the recurrence has no learned rotations",
            ),
            (
                "the U(1) control",
                |c| c.rotation_group = RotationGroup::U1,
                "the U(1) rotation control is not served",
            ),
            (
                "a SwiGLU gate",
                |c| c.gate = Gate::Swiglu,
                "the feed-forward gate is not Hamilton",
            ),
            (
                "a pointer",
                |c| {
                    c.pointer = Some(
                        serde_json::from_value(serde_json::json!({"dim": 8})).expect("pointer"),
                    )
                },
                "a pointer head is not served",
            ),
            (
                "a memory",
                |c| c.memory = Some(serde_json::json!({})),
                "a memory, read selection or read router is not served",
            ),
            (
                "a wider vocabulary",
                |c| c.vocab_size = 512,
                "the native vector head is absent or does not match the trunk",
            ),
        ];
        for (what, mutate, reason) in refused {
            let mut config = small_config(1);
            mutate(&mut config);
            match check_fbs_export_config(&config, fbs, Some(&head)) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
        let config = small_config(1);
        for (what, weights, head, reason) in [
            (
                "dense projections",
                QuaternionWeights::None,
                Some(&head),
                "the projections are not SpiralCore/FBS",
            ),
            (
                "no native head",
                fbs,
                None,
                "the native vector head is absent or does not match the trunk",
            ),
        ] {
            match check_fbs_export_config(&config, weights, head) {
                Err(e) => assert!(e.to_string().contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
    }

    /// On one GPU: the read-back routes reproduce the model's projection bit for
    /// bit, and the trunk converts and reads out. With `UOR_FBS_CONVERTER_OUT`
    /// set to a directory, it writes `snapshot.json` and `readout.json` there for
    /// the serving build's `fbs-compare` (export, parse, serve, and a reading).
    /// The serving build is never a dependency of this crate (D29).
    #[cfg(feature = "cuda")]
    #[test]
    fn gpu_converted_trunk_reproduces_the_projection() -> Result<()> {
        let device = candle_core::Device::new_cuda(0)?;
        let mut model = StackModel::new(small_config(20261026), &device)?;
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(2)?;
        let snap = fbs_trunk_snapshot(&model)?;
        let (d, m) = (snap.width, snap.mlp);

        // Every projection of every layer, on random carriers: bit-identical.
        let mut rng = Lcg(7);
        for (layer, l) in snap.layers.iter().enumerate() {
            let routes = [
                &l.rec_in,
                &l.rec_gate,
                &l.rec_out,
                &l.mlp_gate,
                &l.mlp_up,
                &l.mlp_down,
            ];
            for (part, routes) in FBS_PROJECTIONS.into_iter().zip(routes) {
                let (in_width, _) = projection_widths(part, d, m);
                let name = format!("layers.{layer:02}.{part}.spiralcore_fbs");
                let queries = model.variables().get(&name).expect("queries").as_tensor();
                for _ in 0..3 {
                    let x: Vec<f32> = (0..in_width)
                        .map(|_| (rng.next() % 20001) as f32 / 1000.0 - 10.0)
                        .collect();
                    let carrier = Tensor::from_vec(x.clone(), (1, in_width), &device)?;
                    let gpu = model
                        .linear(&carrier, queries)?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let host = host_gather(routes, &x);
                    assert_eq!(gpu.len(), host.len(), "{name}: output width");
                    assert!(
                        gpu.iter()
                            .zip(&host)
                            .all(|(a, b)| a.to_bits() == b.to_bits()),
                        "{name}: the read-back routes differ from the GPU projection"
                    );
                }
            }
        }

        // The trunk reads out, and an active sheet arm is refused.
        let readout = fbs_float_readout(&model, FbsReadoutMode::Plain)?;
        let export = fbs_float_readout(&model, FbsReadoutMode::Export)?;
        crate::spiralcore_fbs_arms::set_fbs_arm(Some(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
        ));
        let armed = fbs_trunk_snapshot(&model);
        crate::spiralcore_fbs_arms::set_fbs_arm(None);
        assert!(armed.is_err_and(|e| e.to_string().contains("sheet arm")));
        // The D11 mode (claude-125): round the tables as exported, then read.
        // The rounded trunk must convert to the artifact the original exports
        // to; the serving build checks that from the two snapshots written below.
        round_tables_as_exported(&model)?;
        let rounded = fbs_trunk_snapshot(&model)?;
        assert_ne!(rounded.embed, snap.embed, "the rounding reached the tables");
        let d11 = fbs_float_readout(&model, FbsReadoutMode::D11)?;
        for r in [&readout, &export, &d11] {
            r.check(snap.vocab, snap.width, snap.context, r.mode)
                .map_err(invalid)?;
        }
        assert!(d11
            .positions
            .iter()
            .flat_map(|p| &p.hidden)
            .all(|h| (h * 4096.0).fract() == 0.0));
        // Causality, in every mode: a token read alone from a zero state is
        // that token at the first position of a sequence. The export-mode gate
        // hook once broke this at time 1 (claude-124).
        for r in [&readout, &export, &d11] {
            for q in &r.sequences {
                let alone = &r.positions[q.tokens[0] as usize];
                let gap = alone
                    .hidden
                    .iter()
                    .zip(&q.hidden[0])
                    .map(|(a, b)| (a - b).abs())
                    .fold(0f32, f32::max);
                assert!(
                    gap <= 1e-5 && alone.next == q.next[0],
                    "mode {:?}: token {} alone differs from position 0 of its sequence (gap {gap:e})",
                    r.mode,
                    q.tokens[0]
                );
            }
        }
        // Each mode reaches the forward: some hidden vector changes. This does
        // not isolate the transition snap, because the rounded tables and the
        // unrounded gates already change every position; the run's set-aside
        // 2 (the snap turned off) is its pin.
        assert!(readout
            .positions
            .iter()
            .zip(&export.positions)
            .any(|(a, b)| a.hidden != b.hidden));
        assert!(export
            .sequences
            .iter()
            .zip(&d11.sequences)
            .any(|(a, b)| a.hidden != b.hidden));
        if let Ok(dir) = std::env::var("UOR_FBS_CONVERTER_OUT") {
            let dir = std::path::Path::new(&dir);
            for (name, json) in [
                ("snapshot.json", serde_json::to_vec(&snap)?),
                ("snapshot-rounded.json", serde_json::to_vec(&rounded)?),
                ("readout.json", serde_json::to_vec(&readout)?),
                ("readout-export.json", serde_json::to_vec(&export)?),
                ("readout-d11.json", serde_json::to_vec(&d11)?),
            ] {
                uor_r4_integer::fbs_snapshot::write_new_file(&dir.join(name), &json)?;
            }
            eprintln!(
                "fbs converter: wrote the snapshots and the plain, export and D11 readouts to {}",
                dir.display()
            );
        }
        Ok(())
    }
}
