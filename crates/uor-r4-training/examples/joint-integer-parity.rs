//! Integer serving of a trained joint checkpoint, against its float forms.
//!
//! Loads a `JointModel` checkpoint (for example one saved by
//! `joint-read-geometry save_model=true`). A float checkpoint gets frozen dyadic
//! scales calibrated once (post-training quantization); a checkpoint trained
//! with `quantize_ramp` keeps its own. The model is packed, and the same evenly
//! spaced windows of a token file are scored three ways: the float weights
//! (for a quantization-aware checkpoint, its F32 shadows), the packed F32
//! emulator, and the integer runtime (`uor-r4-integer`), with the read enabled
//! and disabled.
//! It also compares the integer and emulator distributions and states
//! position by position. Every window starts from a fresh state. Dot and
//! Lorentz reads are both served; a Lorentz model needs a table root with the
//! arcosh table (`joint-integer-tables`).
//!
//! ```text
//! cargo run --release -p uor-r4-training --example joint-integer-parity -- \
//!   model=CHECKPOINT_DIR tables=TABLE_ROOT tokens=VALID.u16 out=NEW_REPORT_ROOT \
//!   [lens=TOKEN_BYTES.u16] [windows=32]
//! ```
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, IndexOp};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::joint_integer::{IntegerModel, PROBABILITY_TOTAL};
use uor_r4_training::joint_model::{JointModel, ReadMode};
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

const STATE_SCALE: f64 = 2048.0;

struct Args(BTreeMap<String, String>);
impl Args {
    fn required(&self, key: &str) -> Result<PathBuf> {
        self.0
            .get(key)
            .map(PathBuf::from)
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }
}

fn read_tokens(path: &Path, vocabulary: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    if tokens.iter().any(|&id| id as usize >= vocabulary) {
        return Err(invalid(format!(
            "{} has ids outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path":path,"bytes":fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}

/// Sums over scored targets for one read mode.
#[derive(Default)]
struct Scores {
    float_nll: f64,
    emulator_nll: f64,
    integer_nll: f64,
    total_variation: f64,
    maximum_probability_delta: f64,
    maximum_state_delta: f64,
    top1_agreement_emulator: usize,
    top1_agreement_float: usize,
    integer_no_read_mass: f64,
    emulator_no_read_mass: f64,
    integer_seconds: f64,
}

fn argmax(values: impl Iterator<Item = f64>) -> usize {
    let mut best = (0usize, f64::NEG_INFINITY);
    for (index, value) in values.enumerate() {
        if value > best.1 {
            best = (index, value);
        }
    }
    best.0
}

fn main() -> Result<()> {
    let args = Args(
        std::env::args()
            .skip(1)
            .map(|arg| {
                arg.split_once('=')
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .ok_or_else(|| invalid(format!("expected key=value, not {arg}")))
            })
            .collect::<Result<_>>()?,
    );
    for key in args.0.keys() {
        if !matches!(
            key.as_str(),
            "model" | "tables" | "tokens" | "out" | "lens" | "windows"
        ) {
            return Err(invalid(format!("unknown argument {key}=")));
        }
    }
    let checkpoint = args.required("model")?;
    let tables = args.required("tables")?;
    let tokens_path = args.required("tokens")?;
    let out = args.required("out")?;
    let lens_path = args.0.get("lens").map(PathBuf::from);
    let windows: usize = match args.0.get("windows") {
        None => 32,
        Some(value) => value
            .parse()
            .map_err(|_| invalid(format!("invalid windows={value}")))?,
    };
    if windows == 0 {
        return Err(invalid("windows must be positive"));
    }
    report_output::claim(&out)?;
    let started = Instant::now();

    let device = Device::Cpu;
    let loaded = JointModel::load(&checkpoint, &device)?;
    let trained_scales = loaded.quantization().is_some();
    let (float, quantized) = if trained_scales {
        (loaded.without_quantization()?, loaded)
    } else {
        let mut quantized = JointModel::load(&checkpoint, &device)?;
        quantized.configure_quantization(0, 1)?;
        (loaded, quantized)
    };
    let config = float.config.clone();
    let vocabulary = config.vocab_size;
    let tokens = read_tokens(&tokens_path, vocabulary)?;
    let lens = lens_path
        .as_deref()
        .map(|path| read_tokens(path, u16::MAX as usize + 1))
        .transpose()?;
    if lens.as_ref().is_some_and(|lens| lens.len() != vocabulary) {
        return Err(invalid("lens must hold one byte length per token id"));
    }
    let time = config.context;
    if tokens.len() <= windows * (time + 1) {
        return Err(invalid("token file is too short for the requested windows"));
    }

    let packed = out.join("packed");
    fs::create_dir(&packed)?;
    quantized.save_hard(&packed)?;
    let emulator = JointModel::load_hard(&packed, &device)?;
    let integer = IntegerModel::load_with_tables(&packed, &tables)?;
    let prepared = started.elapsed().as_secs_f64();

    let stride = (tokens.len() - time - 1) / windows;
    let mut modes = [Scores::default(), Scores::default()];
    let mut bytes = 0.0f64;
    for window in 0..windows {
        let start = window * stride;
        let inputs = &tokens[start..start + time];
        let targets = &tokens[start + 1..start + time + 1];
        if let Some(lens) = &lens {
            bytes += targets
                .iter()
                .map(|&id| f64::from(lens[id as usize]))
                .sum::<f64>();
        }
        for (slot, mode) in [ReadMode::Enabled, ReadMode::NoRead]
            .into_iter()
            .enumerate()
        {
            let scores = &mut modes[slot];
            let float_output = float.forward(inputs, 1, time, mode, false)?;
            let emulator_output = emulator.forward(inputs, 1, time, mode, false)?;
            let float_rows = float_output.probabilities.i(0)?.to_vec2::<f32>()?;
            let emulator_rows = emulator_output.probabilities.i(0)?.to_vec2::<f32>()?;
            let emulator_states = emulator_output.states.i(0)?.to_vec2::<f32>()?;
            let emulator_no_read = emulator_output
                .no_read_mass
                .flatten_all()?
                .to_vec1::<f32>()?;
            let mut session = integer.new_session();
            for position in 0..time {
                let clock = Instant::now();
                let step = integer.step(&mut session, inputs[position], mode)?;
                scores.integer_seconds += clock.elapsed().as_secs_f64();
                let target = targets[position] as usize;
                let probability = step.probabilities[target] as f64 / PROBABILITY_TOTAL as f64;
                if probability <= 0.0 {
                    return Err(invalid("integer target probability is zero"));
                }
                scores.integer_nll -= probability.ln();
                scores.float_nll -= f64::from(float_rows[position][target]).ln();
                scores.emulator_nll -= f64::from(emulator_rows[position][target]).ln();
                let mut variation = 0.0f64;
                for (&code, &value) in step.probabilities.iter().zip(&emulator_rows[position]) {
                    let delta = (code as f64 / PROBABILITY_TOTAL as f64 - f64::from(value)).abs();
                    variation += delta / 2.0;
                    scores.maximum_probability_delta = scores.maximum_probability_delta.max(delta);
                }
                scores.total_variation += variation;
                for (&code, &value) in step.state.iter().zip(&emulator_states[position]) {
                    scores.maximum_state_delta = scores
                        .maximum_state_delta
                        .max((f64::from(code) / STATE_SCALE - f64::from(value)).abs());
                }
                let integer_top = argmax(step.probabilities.iter().map(|&p| p as f64));
                let emulator_top = argmax(emulator_rows[position].iter().map(|&p| f64::from(p)));
                let float_top = argmax(float_rows[position].iter().map(|&p| f64::from(p)));
                scores.top1_agreement_emulator += usize::from(integer_top == emulator_top);
                scores.top1_agreement_float += usize::from(integer_top == float_top);
                scores.integer_no_read_mass += step.no_read_mass as f64 / PROBABILITY_TOTAL as f64;
                scores.emulator_no_read_mass += f64::from(emulator_no_read[position]);
            }
        }
        eprintln!(
            "window {}/{windows}: integer read {:.4} no-read {:.4} nats/target so far",
            window + 1,
            modes[0].integer_nll / ((window + 1) * time) as f64,
            modes[1].integer_nll / ((window + 1) * time) as f64
        );
    }

    let targets = (windows * time) as f64;
    let bits = |nll: f64| (bytes > 0.0).then(|| nll / bytes / std::f64::consts::LN_2);
    let summary = |scores: &Scores| {
        json!({
            "float_nll":scores.float_nll / targets,
            "emulator_nll":scores.emulator_nll / targets,
            "integer_nll":scores.integer_nll / targets,
            "float_bits_per_byte":bits(scores.float_nll),
            "emulator_bits_per_byte":bits(scores.emulator_nll),
            "integer_bits_per_byte":bits(scores.integer_nll),
            "integer_minus_emulator_nll":(scores.integer_nll - scores.emulator_nll) / targets,
            "emulator_minus_float_nll":(scores.emulator_nll - scores.float_nll) / targets,
            "mean_total_variation_integer_emulator":scores.total_variation / targets,
            "maximum_probability_delta_integer_emulator":scores.maximum_probability_delta,
            "maximum_state_delta_integer_emulator":scores.maximum_state_delta,
            "top1_agreement_integer_emulator":scores.top1_agreement_emulator as f64 / targets,
            "top1_agreement_integer_float":scores.top1_agreement_float as f64 / targets,
            "mean_no_read_mass_integer":scores.integer_no_read_mass / targets,
            "mean_no_read_mass_emulator":scores.emulator_no_read_mass / targets,
            "integer_seconds_per_step":scores.integer_seconds / targets,
        })
    };
    let effect = |select: fn(&Scores) -> f64| (select(&modes[1]) - select(&modes[0])) / targets;
    let report = json!({
        "schema":"uor-r4.joint-integer-parity/1",
        "scope":"Post-training quantization of a float checkpoint and its integer execution, scored on evenly spaced windows of the given token file. Development measurement: no fine-tuning, checkpoint selection, final holdout or language qualification.",
        "inputs":{
            "checkpoint_config":identity(&checkpoint.join("config.json"))?,
            "checkpoint_weights":identity(&checkpoint.join("model.safetensors"))?,
            "tokens":identity(&tokens_path)?,
            "lens":lens_path.as_deref().map(identity).transpose()?,
            "tables_metadata":identity(&tables.join("tables.json"))?,
            "arcosh_metadata":tables.join("arcosh.json").try_exists()?.then(|| identity(&tables.join("arcosh.json"))).transpose()?,
            "packed_manifest":identity(&packed.join("hard-model.json"))?
        },
        "config":config,
        "quantization":if trained_scales {
            "Quantization-aware checkpoint: its own frozen dyadic scales; float scores use its F32 shadow weights; signed4 weights, signed16 additive and Lorentz scalars; quantized interfaces"
        } else {
            "Post-training: frozen dyadic scales calibrated once on the float checkpoint (configure_quantization(0, 1)); signed4 weights, signed16 additive and Lorentz scalars; quantized interfaces; no quantization-aware training"
        },
        "windows":windows,"window_tokens":time,"stride":stride,"targets":windows * time,
        "evaluation":"Fresh state per window; evenly spaced starts; Read and NoRead on the same targets; float and emulator by full-window forward, integer by incremental steps",
        "read":summary(&modes[0]),
        "no_read":summary(&modes[1]),
        "read_effect_nats":{
            "float":effect(|s| s.float_nll),
            "emulator":effect(|s| s.emulator_nll),
            "integer":effect(|s| s.integer_nll),
        },
        "seconds":{"preparation":prepared,"total":started.elapsed().as_secs_f64()},
        "timing_scope":"Integer step time is this machine's CPU in this build; it is not an optimized serving, throughput or energy measurement"
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
