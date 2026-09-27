//! M4a: a learned continuous cache over a frozen Llama checkpoint, with dot,
//! Euclidean and Lorentz (hyperbolic) scores of equal parameter count (see
//! `uor_r4_training::cache_memory`).
//!
//! ```text
//! cache-memory model=DIR train=X.u16 valid=Y.u16 out=NEW_ROOT
//!     [geometries=dot,euclid,lorentz] [seeds=1,2] [dim=32] [gap=256] [window=256] [segment=1024]
//!     [train_segments=256] [valid_segments=64] [site=head|attention:L|mlp:L]
//!     [steps=400] [batch=4] [lr=0.003] [eval_every=100] [eval_rows=4] [save=false]
//!     [device=cpu|metal]
//! ```
//!
//! With `save=true` every trained arm is written to
//! `NEW_ROOT/models/GEOMETRY-seedS.safetensors` with a `.json` beside it
//! (geometry, dimensions, gap and site), the input of `lut-tool mode=export`'s
//! `cache=` option.
//!
//! `device=metal` (with `--features metal`) runs the backbone on the GPU; the
//! cache always trains on the CPU. The frozen backbone runs once over evenly spaced segments of `segment + 1`
//! tokens, in windows of its trained context `window` with stride
//! `window / 2`; every position is read from a window in which it has at least
//! half a window of context (the first half window excepted). Its states at
//! `site` and its log-probability of every next token are kept in memory.
//! Every arm then trains on the same training segments in the same order (per
//! seed; each pass over the segments in a fresh random order) and is evaluated
//! on every validation segment:
//! - mixture and backbone NLL at the query positions `t >= gap`;
//! - the share of query positions whose next token is in the cache at all;
//! - read concentration, on every `eval_rows`-th query row: the share of
//!   readable entries holding 99% of the read mass, and the mass in the top 8
//!   and 32 entries (what a sparse index must recover);
//! - the gate, `beta` and the mean `|k|^2` (the Lorentz arm's curvature scale).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::cache_memory::{
    concentration, mean, segment_features, CacheBatch, CacheGeometry, CacheMemory,
};
use uor_r4_training::joint_optimizer::{AdamConfig, NamedAdamW};
use uor_r4_training::kappa_llama::{load_checkpoint, KappaLlama, ScoreKind, Site, Trainable};
use uor_r4_training::{Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
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
    fn list<T: std::str::FromStr>(&self, key: &str, default: &str) -> Result<Vec<T>> {
        self.text(key)
            .unwrap_or(default)
            .split(',')
            .map(|v| {
                v.trim()
                    .parse()
                    .map_err(|_| invalid(format!("invalid {key} entry {v}")))
            })
            .collect()
    }
}

struct Settings {
    model: PathBuf,
    train: PathBuf,
    valid: PathBuf,
    geometries: Vec<CacheGeometry>,
    seeds: Vec<u64>,
    dim: usize,
    gap: usize,
    window: usize,
    segment: usize,
    train_segments: usize,
    valid_segments: usize,
    site: SiteChoice,
    steps: usize,
    batch: usize,
    lr: f64,
    eval_every: usize,
    eval_rows: usize,
    save: bool,
    device: String,
}

#[derive(Clone, Copy, Debug)]
enum SiteChoice {
    Head,
    Attention(usize),
    Mlp(usize),
}

impl SiteChoice {
    fn parse(text: &str) -> Result<Self> {
        let layer = |rest: &str| -> Result<usize> {
            rest.parse()
                .map_err(|_| invalid(format!("invalid site layer {rest}")))
        };
        match text.split_once(':') {
            None if text == "head" => Ok(Self::Head),
            Some(("attention", rest)) => Ok(Self::Attention(layer(rest)?)),
            Some(("mlp", rest)) => Ok(Self::Mlp(layer(rest)?)),
            _ => Err(invalid("site must be head, attention:L or mlp:L")),
        }
    }

    fn site(self) -> Site {
        match self {
            Self::Head => Site::Head,
            Self::Attention(layer) => Site::Attention(layer),
            Self::Mlp(layer) => Site::Mlp(layer),
        }
    }

    fn name(self) -> String {
        match self {
            Self::Head => "head".to_owned(),
            Self::Attention(l) => format!("attention:{l}"),
            Self::Mlp(l) => format!("mlp:{l}"),
        }
    }
}

fn settings(args: &Args) -> Result<Settings> {
    let geometries = args
        .text("geometries")
        .unwrap_or("dot,euclid,lorentz")
        .split(',')
        .map(|g| CacheGeometry::parse(g.trim()))
        .collect::<Result<Vec<_>>>()?;
    let s = Settings {
        model: PathBuf::from(args.required("model")?),
        train: PathBuf::from(args.required("train")?),
        valid: PathBuf::from(args.required("valid")?),
        geometries,
        seeds: args.list("seeds", "1,2")?,
        dim: args.number("dim", 32)?,
        gap: args.number("gap", 256)?,
        window: args.number("window", 256)?,
        segment: args.number("segment", 1024)?,
        train_segments: args.number("train_segments", 256)?,
        valid_segments: args.number("valid_segments", 64)?,
        site: SiteChoice::parse(args.text("site").unwrap_or("head"))?,
        steps: args.number("steps", 400)?,
        batch: args.number("batch", 4)?,
        lr: args.number("lr", 0.003)?,
        eval_every: args.number("eval_every", 100)?,
        eval_rows: args.number("eval_rows", 4)?,
        save: args.text("save") == Some("true"),
        device: args.text("device").unwrap_or("cpu").to_owned(),
    };
    if s.dim == 0
        || s.window < 2
        || !s.window.is_multiple_of(2)
        || s.segment < s.window
        || !s.segment.is_multiple_of(s.window / 2)
        || s.gap >= s.segment
        || s.gap == 0
        || s.train_segments == 0
        || s.valid_segments == 0
        || s.steps == 0
        || s.batch == 0
        || s.eval_every == 0
        || s.eval_rows == 0
        || s.seeds.is_empty()
        || s.geometries.is_empty()
        || !s.lr.is_finite()
        || s.lr <= 0.0
    {
        return Err(invalid(
            "inconsistent settings: need gap < segment, window even and <= segment, segment a multiple of window / 2, positive counts",
        ));
    }
    Ok(s)
}

fn read_tokens(path: &Path, vocab: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if bytes.len() % 2 != 0 || tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid(format!(
            "{} is not a valid u16 token file",
            path.display()
        )));
    }
    Ok(tokens)
}

/// Backbone output for `count` evenly spaced segments of a token file.
struct Features {
    states: Tensor,
    logp: Tensor,
    next: Vec<Vec<u32>>,
}

fn features(model: &KappaLlama, tokens: &[u32], count: usize, s: &Settings) -> Result<Features> {
    let span = s.segment + 1;
    if tokens.len() < span {
        return Err(invalid("token file shorter than one segment"));
    }
    let room = tokens.len() - span;
    let mut states = Vec::with_capacity(count);
    let mut logps = Vec::with_capacity(count);
    let mut nexts = Vec::with_capacity(count);
    for c in 0..count {
        let offset = if count == 1 {
            0
        } else {
            c * room / (count - 1)
        };
        let (state, logp, next) = segment_features(
            model,
            &tokens[offset..offset + span],
            s.window,
            s.site.site(),
        )?;
        // The cache trains on the CPU whatever device ran the backbone.
        states.push(state.to_device(&Device::Cpu)?);
        logps.push(logp.to_device(&Device::Cpu)?);
        nexts.push(next);
    }
    Ok(Features {
        states: Tensor::stack(&states, 0)?,
        logp: Tensor::stack(&logps, 0)?,
        next: nexts,
    })
}

/// Deterministic batch order for one seed (shared by every geometry): passes
/// over the training segments in a fresh random order each pass, so no
/// segment repeats before every segment has been read.
fn batch_order(seed: u64, steps: usize, batch: usize, segments: usize) -> Vec<Vec<usize>> {
    let mut state = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut pass: Vec<usize> = Vec::new();
    (0..steps)
        .map(|_| {
            (0..batch)
                .map(|_| {
                    if pass.is_empty() {
                        pass = (0..segments).collect();
                        for i in (1..segments).rev() {
                            let j = (next() % (i as u64 + 1)) as usize;
                            pass.swap(i, j);
                        }
                    }
                    pass.pop().unwrap_or(0)
                })
                .collect()
        })
        .collect()
}

fn select(features: &Features, rows: &[usize]) -> Result<(Tensor, Tensor, Vec<Vec<u32>>)> {
    let index = Tensor::from_vec(
        rows.iter().map(|&r| r as u32).collect::<Vec<u32>>(),
        rows.len(),
        features.states.device(),
    )?;
    Ok((
        features.states.index_select(&index, 0)?,
        features.logp.index_select(&index, 0)?,
        rows.iter().map(|&r| features.next[r].clone()).collect(),
    ))
}

/// Validation metrics of one arm; `full` adds concentration and cache coverage.
fn evaluate(memory: &CacheMemory, valid: &Features, s: &Settings, full: bool) -> Result<Value> {
    let count = valid.next.len();
    let (mut mixture, mut backbone, mut gate, mut key_sq, mut batches) =
        (0.0, 0.0, 0.0, 0.0, 0usize);
    let (mut fraction, mut top8, mut top32, mut rows) = (0.0, 0.0, 0.0, 0usize);
    let (mut covered, mut queries) = (0usize, 0usize);
    let chunk = s.batch.max(1);
    for first in (0..count).step_by(chunk) {
        let rows_here: Vec<usize> = (first..(first + chunk).min(count)).collect();
        let (states, logp, next) = select(valid, &rows_here)?;
        let out = memory.forward(&CacheBatch {
            states: &states,
            logp: &logp,
            next: &next,
        })?;
        mixture += mean(&out.nll)?;
        backbone += mean(&out.backbone_nll)?;
        gate += mean(&out.gate)?;
        key_sq += mean(&out.key_norm_sq)?;
        batches += 1;
        if full {
            let attention = out.attention.to_vec3::<f32>()?;
            for (b, segment) in attention.iter().enumerate() {
                for (row, values) in segment.iter().enumerate() {
                    let target = next[b][row + s.gap];
                    queries += 1;
                    if next[b][..=row].contains(&target) {
                        covered += 1;
                    }
                    if row % s.eval_rows == 0 {
                        let (share, tops) = concentration(values, row + 1, 0.99, &[8, 32]);
                        fraction += share;
                        top8 += tops[0];
                        top32 += tops[1];
                        rows += 1;
                    }
                }
            }
        }
    }
    let n = batches.max(1) as f64;
    let mut report = json!({
        "mixture_nll": mixture / n,
        "backbone_nll": backbone / n,
        "delta_nats": (mixture - backbone) / n,
        "mean_gate": gate / n,
        "mean_key_norm_sq": key_sq / n,
        "beta": memory.beta()?,
    });
    if full {
        let r = rows.max(1) as f64;
        report["cache_covers_target"] = json!(covered as f64 / queries.max(1) as f64);
        report["read_share_for_99_percent"] = json!(fraction / r);
        report["top8_mass"] = json!(top8 / r);
        report["top32_mass"] = json!(top32 / r);
        report["concentration_rows"] = json!(rows);
    }
    Ok(report)
}

fn run(s: &Settings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let backbone_device = match s.device.as_str() {
        "cpu" => Device::Cpu,
        "metal" => Device::new_metal(0)?,
        other => return Err(invalid(format!("device must be cpu or metal, not {other}"))),
    };
    let device = Device::Cpu;
    let checkpoint = load_checkpoint(&s.model, &backbone_device)?;
    let weights_sha256 = checkpoint.weights_sha256.clone();
    let vocab = checkpoint.shape.vocab;
    let width = checkpoint.shape.width;
    let model = KappaLlama::new(
        checkpoint,
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        &backbone_device,
    )?;
    let train_tokens = read_tokens(&s.train, vocab)?;
    let valid_tokens = read_tokens(&s.valid, vocab)?;
    let train = features(&model, &train_tokens, s.train_segments, s)?;
    let valid = features(&model, &valid_tokens, s.valid_segments, s)?;
    let feature_seconds = started.elapsed().as_secs_f64();
    eprintln!("backbone features in {feature_seconds:.1}s");
    let mut arms = Vec::new();
    for &seed in &s.seeds {
        let order = batch_order(seed, s.steps, s.batch, s.train_segments);
        for &geometry in &s.geometries {
            let arm_started = Instant::now();
            let memory = CacheMemory::new(geometry, width, s.dim, s.gap, seed, &device)?;
            let mut optimizer = NamedAdamW::new(
                memory.variables(),
                AdamConfig {
                    learning_rate: s.lr,
                    weight_decay: 0.0,
                    ..AdamConfig::default()
                },
            )?;
            let mut curve = vec![json!({"step": 0, "valid": evaluate(&memory, &valid, s, false)?})];
            let mut recent = Vec::new();
            for (step, rows) in order.iter().enumerate() {
                let (states, logp, next) = select(&train, rows)?;
                let loss = memory
                    .forward(&CacheBatch {
                        states: &states,
                        logp: &logp,
                        next: &next,
                    })?
                    .nll
                    .mean_all()?;
                recent.push(f64::from(loss.to_scalar::<f32>()?));
                let gradients = loss.backward()?;
                optimizer.step(memory.variables(), &gradients)?;
                if (step + 1) % s.eval_every == 0 {
                    let train_nll = recent.iter().sum::<f64>() / recent.len().max(1) as f64;
                    recent.clear();
                    curve.push(json!({
                        "step": step + 1,
                        "train_mixture_nll": train_nll,
                        "valid": evaluate(&memory, &valid, s, false)?,
                    }));
                }
            }
            let final_report = evaluate(&memory, &valid, s, true)?;
            eprintln!(
                "{} seed {seed}: delta {:+.4} nats/token, 99% of read in {:.3} of entries ({:.1}s)",
                geometry.name(),
                final_report["delta_nats"].as_f64().unwrap_or(f64::NAN),
                final_report["read_share_for_99_percent"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                arm_started.elapsed().as_secs_f64()
            );
            if s.save {
                let directory = out.join("models");
                fs::create_dir_all(&directory)?;
                let stem = format!("{}-seed{seed}", geometry.name());
                let tensors: std::collections::HashMap<String, Tensor> = memory
                    .variables()
                    .iter()
                    .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
                    .collect();
                candle_core::safetensors::save(
                    &tensors,
                    directory.join(format!("{stem}.safetensors")),
                )?;
                fs::write(
                    directory.join(format!("{stem}.json")),
                    serde_json::to_vec_pretty(&json!({
                        "schema": "uor-r4.cache-memory-model/1",
                        "geometry": geometry.name(), "width": width, "dim": s.dim,
                        "gap": s.gap, "window": s.window, "site": s.site.name(), "seed": seed,
                        "backbone_weights_sha256": weights_sha256,
                        "variables": ["query (width x dim)", "key (width x dim)", "log_beta (1)", "gate_weight (width x 1)", "gate_bias (1)"],
                        "held_out": final_report,
                    }))?,
                )?;
            }
            arms.push(json!({
                "geometry": geometry.name(),
                "seed": seed,
                "final": final_report,
                "curve": curve,
                "seconds": arm_started.elapsed().as_secs_f64(),
            }));
        }
    }
    let report = json!({
        "schema": "uor-r4.cache-memory/1",
        "identity": {
            "model": s.model, "weights_sha256": weights_sha256,
            "train": s.train, "train_sha256": uor_r4_training::sha256_file(&s.train)?,
            "valid": s.valid, "valid_sha256": uor_r4_training::sha256_file(&s.valid)?,
        },
        "settings": {
            "dim": s.dim, "gap": s.gap, "window": s.window, "segment": s.segment,
            "train_segments": s.train_segments, "valid_segments": s.valid_segments,
            "site": s.site.name(), "steps": s.steps, "batch": s.batch, "lr": s.lr,
            "eval_every": s.eval_every, "eval_rows": s.eval_rows, "save": s.save,
        },
        "units": "nats per token at query positions t >= gap",
        "feature_seconds": feature_seconds,
        "seconds": started.elapsed().as_secs_f64(),
        "arms": arms,
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
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
    let s = settings(&args)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    fs::write(
        out.join("attempt.json"),
        serde_json::to_vec_pretty(&json!({"argv": std::env::args().collect::<Vec<_>>()}))?,
    )?;
    let result = run(&s, &out);
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
