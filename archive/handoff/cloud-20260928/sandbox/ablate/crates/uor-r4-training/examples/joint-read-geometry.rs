//! Train the D8 joint learner from scratch on u16 token files and report its
//! development likelihood with the read enabled and disabled, for one read
//! geometry. Two runs that differ only in `geometry=` compare the Dot and
//! Lorentz reads at an equal budget: same seed, initial shared arrays, sampled
//! windows and evaluation windows. Constant AdamW learning rate, as in the
//! frozen campaigns. Every evaluation window starts from a fresh state.
//!
//! This is an exploratory comparison tool, not a frozen campaign: no evaluator
//! manifest, source edits, generation panel, checkpoint selection or resume.
//!
//! `read_dropout=P` trains round(P*batch) windows of every batch with the read
//! disabled from their first position, and the rest with it enabled, in one
//! loss weighted by window count. It tests whether making the recurrent state
//! predict without its read prevents read dependence. It requires one shard.
//!
//! ```text
//! cargo run --release -p uor-r4-training --example joint-read-geometry -- \
//!   train=TRAIN.u16 valid=VALID.u16 out=NEW_REPORT_ROOT geometry=dot|lorentz \
//!   [lens=TOKEN_BYTES.u16] [seed=1] [width=256] [context=256] [batch=16] \
//!   [steps=1000] [lr=0.001] [shards=1] [transport=quaternion] \
//!   [eval_every=250] [eval_windows=64] [final_windows=256] [max_seconds=inf] \
//!   [save_model=false] [read_dropout=0]
//! ```
//!
//! `lens` holds the byte length of each token id (u16, vocabulary order); with
//! it the report adds bits per byte over the scored targets.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::backprop::GradStore;
use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::joint_model::{JointConfig, JointModel, ReadGeometry, ReadMode, Transport};
use uor_r4_training::joint_optimizer::{AdamConfig, NamedAdamW};
use uor_r4_training::joint_parallel::batch_gradients;
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

/// Counter-based window sampler (SplitMix64).
struct Windows(u64);
impl Windows {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

struct Args(BTreeMap<String, String>);
impl Args {
    fn text(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
    fn required(&self, key: &str) -> Result<&str> {
        self.text(key)
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }
    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.text(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

struct Settings {
    train: PathBuf,
    valid: PathBuf,
    lens: Option<PathBuf>,
    config: JointConfig,
    batch: usize,
    steps: usize,
    learning_rate: f64,
    shards: usize,
    eval_every: usize,
    eval_windows: usize,
    final_windows: usize,
    max_seconds: f64,
    save_model: bool,
    read_dropout: f64,
    dropped_windows: usize,
}

fn settings(args: &Args) -> Result<Settings> {
    let known = [
        "train",
        "valid",
        "out",
        "geometry",
        "lens",
        "seed",
        "width",
        "context",
        "batch",
        "steps",
        "lr",
        "shards",
        "transport",
        "eval_every",
        "eval_windows",
        "final_windows",
        "max_seconds",
        "save_model",
        "read_dropout",
    ];
    if let Some(key) = args.0.keys().find(|key| !known.contains(&key.as_str())) {
        return Err(invalid(format!("unknown argument {key}=")));
    }
    let read_geometry = match args.required("geometry")? {
        "dot" => ReadGeometry::Dot,
        "lorentz" => ReadGeometry::Lorentz,
        other => {
            return Err(invalid(format!(
                "geometry must be dot or lorentz, not {other}"
            )))
        }
    };
    let transport = match args.text("transport").unwrap_or("quaternion") {
        "quaternion" => Transport::Quaternion,
        "householder_pair" => Transport::HouseholderPair,
        other => return Err(invalid(format!("unknown transport {other}"))),
    };
    let config = JointConfig {
        vocab_size: 4096,
        width: args.number("width", 256)?,
        read_width: 64,
        context: args.number("context", 256)?,
        transport,
        seed: args.number("seed", 1)?,
        read_geometry,
    };
    let batch: usize = args.number("batch", 16)?;
    let read_dropout: f64 = args.number("read_dropout", 0.0)?;
    let dropped_windows = (read_dropout * batch as f64).round() as usize;
    let shards: usize = args.number("shards", 1)?;
    if !(0.0..1.0).contains(&read_dropout)
        || dropped_windows >= batch.max(1)
        || (dropped_windows > 0 && shards != 1)
    {
        return Err(invalid(
            "read_dropout must lie in [0,1), leave a read-enabled window and use one shard",
        ));
    }
    let settings = Settings {
        train: PathBuf::from(args.required("train")?),
        valid: PathBuf::from(args.required("valid")?),
        lens: args.text("lens").map(PathBuf::from),
        batch,
        steps: args.number("steps", 1000)?,
        learning_rate: args.number("lr", 1e-3)?,
        shards,
        eval_every: args.number("eval_every", 250)?,
        eval_windows: args.number("eval_windows", 64)?,
        final_windows: args.number("final_windows", 256)?,
        max_seconds: args.number("max_seconds", f64::INFINITY)?,
        save_model: args.number("save_model", false)?,
        read_dropout,
        dropped_windows,
        config,
    };
    if settings.batch == 0
        || settings.batch > 64
        || settings.eval_every == 0
        || settings.eval_windows == 0
        || settings.final_windows == 0
    {
        return Err(invalid(
            "batch must be 1..64; evaluation counts must be positive",
        ));
    }
    Ok(settings)
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

struct Evaluation {
    nll: [f64; 2],
    bytes: f64,
    no_read_mass: f64,
    targets: usize,
}

impl Evaluation {
    fn report(&self) -> Value {
        let per_byte =
            |nats: f64| (self.bytes > 0.0).then(|| nats / std::f64::consts::LN_2 / self.bytes);
        let targets = self.targets as f64;
        json!({
            "targets": self.targets,
            "nll_read": self.nll[0] / targets,
            "nll_no_read": self.nll[1] / targets,
            "read_effect_nats": (self.nll[1] - self.nll[0]) / targets,
            "bits_per_byte_read": per_byte(self.nll[0]),
            "bits_per_byte_no_read": per_byte(self.nll[1]),
            "no_read_mass": self.no_read_mass / targets,
        })
    }
}

/// Fixed evenly spaced windows; Read and NoRead score the same targets.
fn evaluate(
    model: &JointModel,
    valid: &[u32],
    lens: Option<&[u32]>,
    windows: usize,
) -> Result<Evaluation> {
    let time = model.config.context;
    let stride = (valid.len() - time - 1) / windows;
    let mut result = Evaluation {
        nll: [0.0; 2],
        bytes: 0.0,
        no_read_mass: 0.0,
        targets: 0,
    };
    let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
    for group in starts.chunks(32) {
        let batch = group.len();
        let mut ids = Vec::with_capacity(batch * time);
        let mut targets = Vec::with_capacity(batch * time);
        for &start in group {
            ids.extend_from_slice(&valid[start..start + time]);
            targets.extend_from_slice(&valid[start + 1..start + time + 1]);
        }
        if let Some(lens) = lens {
            result.bytes += targets
                .iter()
                .map(|&id| f64::from(lens[id as usize]))
                .sum::<f64>();
        }
        result.targets += targets.len();
        let index = Tensor::from_vec(targets, (batch, time, 1), model.device())?;
        for (slot, mode) in [ReadMode::Enabled, ReadMode::NoRead]
            .into_iter()
            .enumerate()
        {
            let output = model.forward(&ids, batch, time, mode, false)?;
            let picked = output.probabilities.gather(&index, 2)?.flatten_all()?;
            result.nll[slot] -= picked
                .to_vec1::<f32>()?
                .iter()
                .map(|&p| f64::from(p).ln())
                .sum::<f64>();
            if mode == ReadMode::Enabled {
                result.no_read_mass += output
                    .no_read_mass
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|&mass| f64::from(mass))
                    .sum::<f64>();
            }
        }
    }
    Ok(result)
}

/// One combined loss: read-enabled windows first, then `dropped` NoRead
/// windows, each part weighted by its share of the batch.
fn read_dropout_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    batch: usize,
    dropped: usize,
) -> Result<(f32, GradStore)> {
    let time = model.config.context;
    let enabled = batch - dropped;
    let split = enabled * time;
    let read = model
        .forward(&inputs[..split], enabled, time, ReadMode::Enabled, true)?
        .loss(&targets[..split])?;
    let no_read = model
        .forward(&inputs[split..], dropped, time, ReadMode::NoRead, true)?
        .loss(&targets[split..])?;
    let loss = read
        .affine(enabled as f64 / batch as f64, 0.0)?
        .add(&no_read.affine(dropped as f64 / batch as f64, 0.0)?)?;
    let mean = loss.to_scalar::<f32>()?;
    if !mean.is_finite() {
        return Err(invalid("nonfinite read-dropout training loss"));
    }
    Ok((mean, loss.backward()?))
}

fn scalars(model: &JointModel) -> Result<Value> {
    let mut values = serde_json::Map::new();
    for (name, variable) in model.variables() {
        if name.contains("lorentz") {
            values.insert(
                name.clone(),
                json!(variable.as_tensor().flatten_all()?.to_vec1::<f32>()?),
            );
        }
    }
    Ok(Value::Object(values))
}

fn run(settings: &Settings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let vocabulary = settings.config.vocab_size;
    let train = read_tokens(&settings.train, vocabulary)?;
    let valid = read_tokens(&settings.valid, vocabulary)?;
    let lens = match &settings.lens {
        Some(path) => {
            let lens = read_tokens(path, u16::MAX as usize + 1)?;
            if lens.len() != vocabulary {
                return Err(invalid("lens must hold one byte length per token id"));
            }
            Some(lens)
        }
        None => None,
    };
    let time = settings.config.context;
    if train.len() <= time + 1 || valid.len() <= time + settings.final_windows {
        return Err(invalid("token files are too short for the context/windows"));
    }
    let model = JointModel::new(settings.config.clone(), &Device::Cpu)?;
    let mut optimizer = NamedAdamW::new(
        model.variables(),
        AdamConfig {
            learning_rate: settings.learning_rate,
            ..AdamConfig::default()
        },
    )?;
    let lens_slice = lens.as_deref();
    let initial = evaluate(&model, &valid, lens_slice, settings.eval_windows)?;
    let mut curve =
        vec![json!({"step":0,"development":initial.report(),"lorentz":scalars(&model)?})];
    let mut windows = Windows(settings.config.seed ^ 0x5EED_0FD8);
    let mut train_seconds = 0.0;
    let mut losses = Vec::new();
    let mut completed = 0;
    for step in 0..settings.steps {
        if started.elapsed().as_secs_f64() >= settings.max_seconds {
            break;
        }
        let step_started = Instant::now();
        let mut inputs = Vec::with_capacity(settings.batch * time);
        let mut targets = Vec::with_capacity(settings.batch * time);
        for _ in 0..settings.batch {
            let start = (windows.next() % (train.len() - time - 1) as u64) as usize;
            inputs.extend_from_slice(&train[start..start + time]);
            targets.extend_from_slice(&train[start + 1..start + time + 1]);
        }
        let (mean_nll, gradients) = if settings.dropped_windows == 0 {
            let gradients = batch_gradients(
                &model,
                &inputs,
                &targets,
                settings.batch,
                time,
                settings.shards,
            )?;
            (gradients.mean_nll, gradients.gradients)
        } else {
            read_dropout_gradients(
                &model,
                &inputs,
                &targets,
                settings.batch,
                settings.dropped_windows,
            )?
        };
        let update = optimizer.step(model.variables(), &gradients)?;
        train_seconds += step_started.elapsed().as_secs_f64();
        losses.push(f64::from(mean_nll));
        completed = step + 1;
        if completed % settings.eval_every == 0 && completed != settings.steps {
            let development = evaluate(&model, &valid, lens_slice, settings.eval_windows)?;
            let mean = losses.iter().sum::<f64>() / losses.len() as f64;
            eprintln!(
                "step {completed}: train {mean:.4}, {}",
                development.report()
            );
            curve.push(json!({
                "step": completed,
                "train_nll_since_previous": mean,
                "global_grad_norm": update.global_grad_norm,
                "development": development.report(),
                "lorentz": scalars(&model)?,
            }));
            losses.clear();
        }
    }
    let evaluation_started = Instant::now();
    let last = evaluate(&model, &valid, lens_slice, settings.final_windows)?;
    let report = json!({
        "schema": "uor-r4.joint-read-geometry-comparison/1",
        "scope": "Exploratory from-scratch fit; equal budget per geometry; not a frozen campaign or capability qualification",
        "config": settings.config,
        "numerical_contract": model.numerical_contract(),
        "parameters": model.parameter_count(),
        "inputs": {
            "train": identity(&settings.train)?,
            "valid": identity(&settings.valid)?,
            "lens": settings.lens.as_deref().map(identity).transpose()?,
            "train_tokens": train.len(),
            "valid_tokens": valid.len(),
        },
        "optimizer": optimizer.config(),
        "batch": settings.batch,
        "shards": settings.shards,
        "read_dropout": settings.read_dropout,
        "read_dropout_windows_per_step": settings.dropped_windows,
        "training_objective": if settings.dropped_windows == 0 {
            "Population-mean next-token NLL with the read enabled"
        } else {
            "Window-count-weighted mean of read-enabled and NoRead next-token NLL"
        },
        "steps_requested": settings.steps,
        "steps_completed": completed,
        "sampled_target_visits": completed * settings.batch * time,
        "window_sampler": "SplitMix64 counter seeded with seed ^ 0x5EED0FD8; uniform window starts",
        "evaluation": "Fresh state per window; evenly spaced starts; Read and NoRead on the same targets",
        "final_windows": settings.final_windows,
        "final": last.report(),
        "lorentz": scalars(&model)?,
        "curve": curve,
        "seconds": {
            "training_updates": train_seconds,
            "final_evaluation": evaluation_started.elapsed().as_secs_f64(),
            "wall": started.elapsed().as_secs_f64(),
        },
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    if settings.save_model {
        model.save(&out.join("model"))?;
    }
    eprintln!("final: {}", last.report());
    Ok(())
}

fn main() -> Result<()> {
    let mut pairs = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| invalid("arguments are key=value"))?;
        pairs.insert(key.to_string(), value.to_string());
    }
    let args = Args(pairs);
    let settings = settings(&args)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = run(&settings, &out);
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
