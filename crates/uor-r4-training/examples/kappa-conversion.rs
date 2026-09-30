//! Curvature-homotopy conversion of a Llama checkpoint (for example
//! SmolLM2-135M/360M-Instruct) to hyperbolic (Lorentz) attention.
//!
//! Offline float tool; see `uor_r4_training::kappa_llama` for the score family
//! and its flat-limit caveat. Curvature is always stated as the dimensionless
//! `t = kappa * mean |k|^2` of a head, which does not change when a head's keys
//! are rescaled. Modes (arguments are key=value):
//!
//! ```text
//! kappa-conversion mode=tokenize model=DIR text=IN.txt out=OUT.u16
//! kappa-conversion mode=probe model=DIR tokens=X.u16 out=NEW_ROOT
//!     [score=intrinsic|key_norm] [batch=1] [time=256] [windows=4] [t_grid=1e-8,1e-6,...,3]
//!     # or training-free flock attention, exactly one arm per invocation:
//!     [flock=1|7|16|64] [weights=softmax|rank] [seed=9001] [held_out_tokens=N]
//! kappa-conversion mode=drive model=DIR tokens=X.u16 out=NEW_ROOT
//!     [teacher=DIR] [batch=1] [time=256] [windows=16] [t_grid=0.1,1] [cost_layers=all|0,4,...]
//! kappa-conversion mode=train student=DIR train=X.u16 valid=Y.u16 out=NEW_ROOT
//!     [teacher=DIR] [score=dot|intrinsic|key_norm|intrinsic_linear|key_norm_linear]
//!     [init_t=1e-5] [trainable=scalars|query_key] [steps=500] [batch=1] [accumulate=4]
//!     [time=256] [lr=1e-5] [curv_lr=1e-2] [eval_every=100] [eval_windows=16] [seed=1]
//!     [max_seconds=0] [save_model=true] [anneal_t=T] [anneal_steps=steps/2] [anneal_hold=true]
//! kappa-conversion mode=sample model=DIR prompt=TEXT [variables=ROOT/model/variables.safetensors]
//!     [score=dot] [trainable=scalars] [system=TEXT] [tokens=64] [raw=true]
//! ```
//!
//! Every mode also accepts `device=cpu|metal` (metal needs `--features metal`).
//!
//! `drive` is the zero-training test to run before any curvature training. It
//! reports, per head of the checkpoint's own attention, the mean and median
//! `|k|^2`, the cosine between each query and its top key, and the attention
//! mass held by the top 2% and 5% of keys (the ceiling for any index scoring
//! that fraction). It then reports the flat-limit curvature drive
//! `g = dL/dt` at `t = 0` of both first-order score kinds (one backward pass per
//! window, next-token loss and, with `teacher=`, KL to the teacher), the
//! zero-shot loss change when one layer's heads are set to each `t`, and a
//! pre-registered rule per layer: curvature training is worth running only if
//! the first-order gain at `t = 1` (`-sum_h g_h`) exceeds the measured zero-shot
//! cost at `t = 1`.
//!
//! `probe flock=K` runs the dense baseline plus exactly one training-free flock
//! arm on the same windows: sink (position 0) + the 64 nearest causal positions
//! + the exact top-`K` of the rest by Lorentz rank, weighting the support either
//! by the checkpoint's scaled-dot softmax (`weights=softmax`; the learned
//! per-head `log_beta` is not applied, so this matches the checkpoint's own
//! softmax only at `log_beta == 0`) or by the fixed normalized
//! rank table `w_i = 1/(i+1)` (`weights=rank`, no exponentials). Ranking lifts
//! each head's post-RoPE q/k as `x_L = (sqrt(1+|x|^2), x)` and orders by
//! `-q0 k0 + q . k` (monotone; no `arcosh`). No weights change. With
//! `seed`/`held_out_tokens` the window starts are counter-drawn from the final
//! `held_out_tokens` of the token file and recorded.
//!
//! `train` minimizes KL(teacher || student) when `teacher=` is given (the
//! `score=dot` is the matched plain control and `*_linear` the matched
//! first-order control (dot plus the fixed quartic feature, free coefficient).
//! `anneal_t` raises a floor on every head's curvature linearly to `t = T`
//! over `anneal_steps`, recomputed from the current keys at every step so that
//! shrinking the keys cannot escape it, then holds it (`anneal_hold=false`
//! releases it). `accumulate` sums gradients over micro-batches of `batch`
//! windows. Memory: the curved scores keep about thirty `(batch, heads, time,
//! time)` tensors per layer for backward, so a 135M student with a 360M
//! teacher needs `batch=1` at `time=256` on a 16 GB machine; use `time=128` on
//! 8 GB.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use candle_core::backprop::GradStore;
use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::joint_optimizer::{AdamConfig, NamedAdamW};
use uor_r4_training::kappa_llama::{
    distillation_kl, held_out_starts, load_checkpoint, max_abs_difference, mean_key_sq,
    next_token_nll, rank_table, Checkpoint, FlockSpec, FlockStats, FlockWeights,
    HeadStatisticsAccumulator, KappaLlama, ScoreKind, Trainable, LAMBDA, LOG_BETA, LOG_EPS,
    NORM_SCALE,
};
use uor_r4_training::{Result, TrainingError};

/// SmolLM2's chat template inserts this system turn when none is given.
const SMOLLM2_SYSTEM: &str = "You are a helpful AI assistant named SmolLM, trained by Hugging Face";
/// log_eps used as "flat" (kappa = e^-24 ~ 4e-11).
const FLAT_LOG_EPS: f32 = -12.0;

type Chunk = (Vec<u32>, Vec<u32>, usize);

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
    fn flag(&self, key: &str, default: bool) -> Result<bool> {
        match self.text(key) {
            None => Ok(default),
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            Some(other) => Err(invalid(format!("{key} must be true or false, not {other}"))),
        }
    }
    fn list(&self, key: &str, default: &str) -> Result<Vec<f64>> {
        self.text(key)
            .unwrap_or(default)
            .split(',')
            .map(|v| {
                v.parse::<f64>()
                    .ok()
                    .filter(|x| x.is_finite() && *x > 0.0 && *x <= 100.0)
                    .ok_or_else(|| invalid(format!("{key} values must lie in (0, 100], not {v}")))
            })
            .collect()
    }
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

fn device(args: &Args) -> Result<Device> {
    match args.text("device").unwrap_or("cpu") {
        "cpu" => Ok(Device::Cpu),
        "metal" => Ok(Device::new_metal(0)?),
        other => Err(invalid(format!("device must be cpu or metal, not {other}"))),
    }
}

fn read_tokens(path: &Path, vocab: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid(format!(
            "{} has a token outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}

/// Windows of `time + 1` tokens: inputs and next-token targets.
fn windows(tokens: &[u32], starts: &[usize], time: usize) -> (Vec<u32>, Vec<u32>) {
    let mut inputs = Vec::with_capacity(starts.len() * time);
    let mut targets = Vec::with_capacity(starts.len() * time);
    for &start in starts {
        inputs.extend_from_slice(&tokens[start..start + time]);
        targets.extend_from_slice(&tokens[start + 1..start + time + 1]);
    }
    (inputs, targets)
}

fn evenly_spaced(len: usize, time: usize, count: usize) -> Result<Vec<usize>> {
    if len < time + 2 || count == 0 {
        return Err(invalid("token file shorter than one window"));
    }
    let span = len - time - 1;
    Ok((0..count).map(|i| i * span / count).collect())
}

/// Evaluation chunks of up to `batch` windows each.
fn chunks(tokens: &[u32], starts: &[usize], time: usize, batch: usize) -> Vec<Chunk> {
    starts
        .chunks(batch)
        .map(|chunk| {
            let (inputs, targets) = windows(tokens, chunk, time);
            (inputs, targets, chunk.len())
        })
        .collect()
}

fn scalar(tensor: &Tensor) -> Result<f64> {
    Ok(f64::from(tensor.to_scalar::<f32>()?))
}

fn summary(grid: &[Vec<f32>]) -> Value {
    let mut values: Vec<f32> = grid.iter().flatten().copied().collect();
    if values.is_empty() {
        return Value::Null;
    }
    values.sort_by(f32::total_cmp);
    let n = values.len();
    let above = |threshold: f32| values.iter().filter(|&&v| v > threshold).count();
    json!({
        "heads": n,
        "min": values[0],
        "median": values[n / 2],
        "max": values[n - 1],
        "above_0.01": above(0.01),
        "above_0.1": above(0.1),
        "above_0.3": above(0.3),
        "above_1": above(1.0),
    })
}

fn clone_checkpoint(checkpoint: &Checkpoint) -> Checkpoint {
    Checkpoint {
        shape: checkpoint.shape.clone(),
        tensors: checkpoint.tensors.clone(),
        weights_sha256: checkpoint.weights_sha256.clone(),
    }
}

/// `log_eps` per head that puts every head at dimensionless curvature `t`.
fn log_eps_for(t: f64, key_sq: &[Vec<f32>]) -> Vec<Vec<f32>> {
    key_sq
        .iter()
        .map(|row| {
            row.iter()
                .map(|m| (0.5 * (t / f64::from(m.max(1e-12))).ln()) as f32)
                .collect()
        })
        .collect()
}

/// Dimensionless curvature `t = kappa * mean |k|^2` per head.
fn dimensionless(kappa: &[Vec<f32>], key_sq: &[Vec<f32>]) -> Vec<Vec<f32>> {
    kappa
        .iter()
        .zip(key_sq)
        .map(|(k, m)| k.iter().zip(m).map(|(a, b)| a * b).collect())
        .collect()
}

/// Mean `|k|^2` per layer and head of `model` on the given chunks (no backward graph).
fn calibrate(model: &KappaLlama, chunks: &[Chunk], time: usize) -> Result<Vec<Vec<f32>>> {
    let shape = model.shape();
    let mut sums = vec![vec![0f64; shape.heads]; shape.layers];
    for (inputs, _, rows) in chunks {
        let mut probe = |layer: usize, _: &Tensor, key: &Tensor, _: &Tensor| -> Result<()> {
            for (head, value) in mean_key_sq(key)?.into_iter().enumerate() {
                sums[layer][head] += f64::from(value) / chunks.len() as f64;
            }
            Ok(())
        };
        model.forward_with_probe(inputs, *rows, time, true, &mut probe)?;
    }
    Ok(sums
        .into_iter()
        .map(|row| row.into_iter().map(|v| v as f32).collect())
        .collect())
}

fn tokenize(args: &Args) -> Result<()> {
    let model = PathBuf::from(args.required("model")?);
    let text = fs::read_to_string(args.required("text")?)?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; token files are never overwritten",
            out.display()
        )));
    }
    let tokenizer =
        HfBpeTokenizer::from_dir(&model).map_err(|error| invalid(format!("tokenizer: {error}")))?;
    if tokenizer.vocab_size() > usize::from(u16::MAX) + 1 {
        return Err(invalid("vocabulary does not fit u16 token files"));
    }
    let tokens = tokenizer.encode(&text);
    let mut bytes = Vec::with_capacity(tokens.len() * 2);
    for token in &tokens {
        let token = u16::try_from(*token).map_err(|_| invalid("token id exceeds u16"))?;
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    fs::write(&out, bytes)?;
    eprintln!(
        "tokenized {} bytes into {} tokens ({:.3} bytes/token)",
        text.len(),
        tokens.len(),
        text.len() as f64 / tokens.len().max(1) as f64
    );
    Ok(())
}

struct ProbeSettings {
    device: Device,
    model: PathBuf,
    tokens: PathBuf,
    score: ScoreKind,
    batch: usize,
    time: usize,
    windows: usize,
    grid: Vec<f64>,
    flock: Option<usize>,
    flock_weights: FlockWeights,
    seed: Option<u64>,
    held_out_tokens: Option<usize>,
}

fn probe_settings(args: &Args) -> Result<ProbeSettings> {
    let flock = match args.text("flock") {
        None => None,
        Some(text) => {
            let k: usize = text
                .parse()
                .map_err(|_| invalid(format!("flock must be a positive integer, not {text}")))?;
            if !matches!(k, 1 | 7 | 16 | 64) {
                return Err(invalid("flock k must be one of 1, 7, 16, 64"));
            }
            Some(k)
        }
    };
    let score = ScoreKind::parse(args.text("score").unwrap_or("intrinsic"))?;
    if flock.is_none() && !score.is_curved() {
        return Err(invalid(
            "probe compares a curved score (intrinsic or key_norm) with dot, or takes flock=K",
        ));
    }
    let flock_weights = FlockWeights::parse(args.text("weights").unwrap_or("softmax"))?;
    let seed: Option<u64> = match args.text("seed") {
        None => None,
        Some(text) => Some(
            text.parse()
                .map_err(|_| invalid(format!("seed must be an integer, not {text}")))?,
        ),
    };
    let held_out_tokens: Option<usize> = match args.text("held_out_tokens") {
        None => None,
        Some(text) => Some(
            text.parse()
                .map_err(|_| invalid(format!("held_out_tokens must be an integer, not {text}")))?,
        ),
    };
    if seed.is_some() != held_out_tokens.is_some() {
        return Err(invalid(
            "seed and held_out_tokens are given together (the held-out split is drawn from the token file tail)",
        ));
    }
    let settings = ProbeSettings {
        device: device(args)?,
        model: PathBuf::from(args.required("model")?),
        tokens: PathBuf::from(args.required("tokens")?),
        score,
        batch: args.number("batch", 1)?,
        time: args.number("time", 256)?,
        windows: args.number("windows", 4)?,
        grid: args.list("t_grid", "1e-8,1e-6,1e-4,1e-3,1e-2,0.1,0.3,1,3")?,
        flock,
        flock_weights,
        seed,
        held_out_tokens,
    };
    if settings.batch == 0 || settings.time < 2 || settings.windows == 0 {
        return Err(invalid("batch and windows must be positive and time >= 2"));
    }
    Ok(settings)
}

fn probe(settings: &ProbeSettings, out: &Path) -> Result<()> {
    if let Some(k) = settings.flock {
        return probe_flock(settings, k, out);
    }
    probe_curved(settings, out)
}

/// Peak resident set of this process, sampled from `/bin/ps` while a run is live.
fn resident_bytes() -> Option<u64> {
    let process = std::process::id().to_string();
    let output = std::process::Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &process])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    text.split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?
        .checked_mul(1024)
}

/// Sample RSS every 100 ms until the returned flag is set; join for the peak.
fn rss_sampler() -> (Arc<AtomicBool>, Arc<AtomicU64>, std::thread::JoinHandle<()>) {
    let stop = Arc::new(AtomicBool::new(false));
    let peak = Arc::new(AtomicU64::new(0));
    let thread_stop = Arc::clone(&stop);
    let thread_peak = Arc::clone(&peak);
    let handle = std::thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            if let Some(bytes) = resident_bytes() {
                thread_peak.fetch_max(bytes, Ordering::Relaxed);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    (stop, peak, handle)
}

/// Dense baseline plus exactly one training-free flock arm over the same windows.
fn probe_flock(settings: &ProbeSettings, k: usize, out: &Path) -> Result<()> {
    let started = Instant::now();
    let (stop, peak, sampler) = rss_sampler();
    let device = &settings.device;
    let checkpoint = load_checkpoint(&settings.model, device)?;
    let weights_sha256 = checkpoint.weights_sha256.clone();
    let tokens = read_tokens(&settings.tokens, checkpoint.shape.vocab)?;
    let tokens_sha256 = uor_r4_training::sha256_file(&settings.tokens)?;
    let starts = match (settings.seed, settings.held_out_tokens) {
        (Some(seed), Some(held_out)) => held_out_starts(
            tokens.len(),
            settings.time,
            settings.windows,
            held_out,
            seed,
        )?,
        _ => evenly_spaced(tokens.len(), settings.time, settings.windows)?,
    };
    let chunks = chunks(&tokens, &starts, settings.time, settings.batch);
    let dot = KappaLlama::new(
        clone_checkpoint(&checkpoint),
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        device,
    )?;
    let spec = FlockSpec {
        k,
        window: 64,
        weights: settings.flock_weights,
    };
    let mut flock = KappaLlama::new(checkpoint, ScoreKind::Dot, 0.0, Trainable::Scalars, device)?;
    flock.set_flock(Some(spec))?;
    let share = 1.0 / chunks.len() as f64;
    let (mut dot_nll, mut flock_nll, mut agree) = (0.0, 0.0, 0.0);
    let mut stats = FlockStats::default();
    for (inputs, targets, n) in &chunks {
        let reference = dot.forward(inputs, *n, settings.time, true)?;
        dot_nll += scalar(&next_token_nll(&reference, targets)?)? * share;
        let logits = flock.forward(inputs, *n, settings.time, true)?;
        flock_nll += scalar(&next_token_nll(&logits, targets)?)? * share;
        let same = logits
            .argmax(2)?
            .eq(&reference.argmax(2)?)?
            .to_dtype(candle_core::DType::F32)?
            .mean_all()?;
        agree += scalar(&same)? * share;
        stats.add(&flock.take_flock_stats());
    }
    stop.store(true, Ordering::Relaxed);
    sampler
        .join()
        .map_err(|_| invalid("rss sampler panicked"))?;
    let peak_rss = peak.load(Ordering::Relaxed);
    let after = uor_r4_training::sha256_file(&settings.model.join("model.safetensors"))?;
    let weights_unchanged = after == weights_sha256;
    let table = match settings.flock_weights {
        FlockWeights::Rank => Some(rank_table(1 + spec.window + k)),
        FlockWeights::Softmax => None,
    };
    let table_sha256 = table.as_ref().map(|values| {
        let bytes: Vec<u8> = values.iter().flat_map(|w| w.to_le_bytes()).collect();
        uor_r4_training::sha256_bytes(&bytes)
    });
    let dense_bits = dot_nll / std::f64::consts::LN_2;
    let report = json!({
        "schema": "uor-r4.kappa-conversion-probe/2",
        "mode": "flock",
        "identity": {
            "model": settings.model,
            "weights_sha256": weights_sha256,
            "tokens": settings.tokens,
            "tokens_sha256": tokens_sha256,
            "shape": flock.shape(),
        },
        "flock": {
            "k": k,
            "window": spec.window,
            "weights": settings.flock_weights,
            "weight_rule": "arm R: w_i = 1/(i+1) normalized over the selected support, i the Lorentz-rank position; arm S: the checkpoint's scaled-dot softmax over the same support, without the learned per-head log_beta (matches the model's own softmax only at log_beta == 0)",
            "weight_table": table,
            "weight_table_sha256": table_sha256,
            "weight_table_scope": "nominal maximum support (1 + window + k); per-row support is shorter at short prefixes and at top-k-short rows",
            "raw_rank_scale": "unnormalized a_i = 1/(i+1) via uor_r4_training::flock::raw_rank_weights, the B2 hybrid scale; not the normalized table above",
        },
        "batch": settings.batch,
        "time": settings.time,
        "windows": settings.windows,
        "seed": settings.seed,
        "held_out_tokens": settings.held_out_tokens,
        "window_starts": starts,
        "dot_nll": dot_nll,
        "flock_nll": flock_nll,
        "delta_nll": flock_nll - dot_nll,
        "top1_agreement": agree,
        "support": {
            "queries": stats.queries,
            "mean_selected": stats.mean_selected(),
            "sink_share": stats.sink_share(),
            "window_share": stats.window_share(),
            "top_k_share": stats.top_k_share(),
            "candidates_scanned": stats.candidates_scanned,
            "short_prefix_rows": stats.short_prefix,
            "top_k_short_rows": stats.top_k_short,
            "cutoff_tie_rows": stats.cutoff_ties,
            "selector": uor_r4_training::flock::FLOCK_SELECTOR_VERSION,
        },
        "weights_sha256": after,
        "weights_unchanged": weights_unchanged,
        "peak_rss_bytes": peak_rss,
        "g2_crosscheck": {
            "dense_bits_per_token": dense_bits,
            "external_reference_bits_per_token": 3.8425,
            "delta_bits_per_token": dense_bits - 3.8425,
            "reference": "docs/transformerless/EXTERNAL_REFEREE_300.md: 596 held-out articles / 71,714 target tokens, first 128 tokens per article, HF tokenizer, no BOS/chat template; a different pipeline",
        },
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("probe.json"), serde_json::to_vec_pretty(&report)?)?;
    if !weights_unchanged {
        return Err(invalid(
            "the checkpoint weights file changed during the flock probe",
        ));
    }
    Ok(())
}

fn probe_curved(settings: &ProbeSettings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let device = &settings.device;
    let checkpoint = load_checkpoint(&settings.model, device)?;
    let identity = json!({"model": settings.model, "weights_sha256": checkpoint.weights_sha256, "shape": checkpoint.shape});
    let tokens = read_tokens(&settings.tokens, checkpoint.shape.vocab)?;
    let starts = evenly_spaced(tokens.len(), settings.time, settings.windows)?;
    let chunks = chunks(&tokens, &starts, settings.time, settings.batch);
    let dot = KappaLlama::new(
        clone_checkpoint(&checkpoint),
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        device,
    )?;
    let key_sq = calibrate(&dot, &chunks, settings.time)?;
    let curved = KappaLlama::new(
        checkpoint,
        settings.score,
        FLAT_LOG_EPS,
        Trainable::Scalars,
        device,
    )?;
    let mut rows = Vec::new();
    let mut dot_nll = 0.0;
    let share = 1.0 / chunks.len() as f64;
    for (i, &t) in settings.grid.iter().enumerate() {
        curved.set_log_eps_grid(&log_eps_for(t, &key_sq))?;
        let (mut max_diff, mut kl, mut agree, mut nll) = (0f32, 0.0, 0.0, 0.0);
        for (inputs, targets, n) in &chunks {
            let reference = dot.forward(inputs, *n, settings.time, true)?;
            if i == 0 {
                dot_nll += scalar(&next_token_nll(&reference, targets)?)? * share;
            }
            let logits = curved.forward(inputs, *n, settings.time, true)?;
            max_diff = max_diff.max(max_abs_difference(&logits, &reference)?);
            kl += scalar(&distillation_kl(&logits, &reference)?)? * share;
            let same = logits
                .argmax(2)?
                .eq(&reference.argmax(2)?)?
                .to_dtype(candle_core::DType::F32)?
                .mean_all()?;
            agree += scalar(&same)? * share;
            nll += scalar(&next_token_nll(&logits, targets)?)? * share;
        }
        let row = json!({
            "t": t,
            "max_abs_logit_difference": max_diff,
            "mean_kl_dot_to_curved": kl,
            "top1_agreement": agree,
            "nll": nll,
        });
        eprintln!("{row}");
        rows.push(row);
    }
    let report = json!({
        "schema": "uor-r4.kappa-conversion-probe/2",
        "identity": identity,
        "score": settings.score,
        "batch": settings.batch,
        "time": settings.time,
        "window_starts": starts,
        "mean_key_sq": key_sq,
        "dot_nll": dot_nll,
        "rows": rows,
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("probe.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

struct DriveSettings {
    device: Device,
    model: PathBuf,
    tokens: PathBuf,
    teacher: Option<PathBuf>,
    batch: usize,
    time: usize,
    windows: usize,
    grid: Vec<f64>,
    cost_layers: Option<Vec<usize>>,
}

fn drive_settings(args: &Args) -> Result<DriveSettings> {
    let grid = args.list("t_grid", "0.1,1")?;
    if !grid.contains(&1.0) {
        return Err(invalid(
            "t_grid must contain 1 (the pre-registered rule is stated at t = 1)",
        ));
    }
    let cost_layers = match args.text("cost_layers").unwrap_or("all") {
        "all" => None,
        text => Some(
            text.split(',')
                .map(|v| {
                    v.parse()
                        .map_err(|_| invalid(format!("invalid cost layer {v}")))
                })
                .collect::<Result<Vec<usize>>>()?,
        ),
    };
    let settings = DriveSettings {
        device: device(args)?,
        model: PathBuf::from(args.required("model")?),
        tokens: PathBuf::from(args.required("tokens")?),
        teacher: args.text("teacher").map(PathBuf::from),
        batch: args.number("batch", 1)?,
        time: args.number("time", 256)?,
        windows: args.number("windows", 16)?,
        grid,
        cost_layers,
    };
    if settings.batch == 0 || settings.time < 33 || settings.windows < 2 {
        return Err(invalid(
            "batch must be positive, time >= 33 and windows >= 2",
        ));
    }
    Ok(settings)
}

fn gradient_grid(gradients: &GradStore, variable: &Tensor) -> Result<Vec<Vec<f32>>> {
    Ok(gradients
        .get(variable)
        .ok_or_else(|| invalid("no gradient reached the curvature coefficient"))?
        .to_vec2()?)
}

/// Mean and sample standard deviation over samples of per-head grids.
fn mean_sd(samples: &[Vec<Vec<f32>>]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = samples.len().max(1) as f64;
    let zero = |s: &Vec<Vec<f32>>| {
        s.iter()
            .map(|row| vec![0f64; row.len()])
            .collect::<Vec<_>>()
    };
    let mut mean = samples.first().map(zero).unwrap_or_default();
    let mut sd = mean.clone();
    for sample in samples {
        for (l, row) in sample.iter().enumerate() {
            for (h, v) in row.iter().enumerate() {
                mean[l][h] += f64::from(*v) / n;
            }
        }
    }
    for sample in samples {
        for (l, row) in sample.iter().enumerate() {
            for (h, v) in row.iter().enumerate() {
                sd[l][h] += (f64::from(*v) - mean[l][h]).powi(2) / (n - 1.0).max(1.0);
            }
        }
    }
    for row in &mut sd {
        for v in row.iter_mut() {
            *v = v.sqrt();
        }
    }
    (mean, sd)
}

fn drive(settings: &DriveSettings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let (device, time) = (&settings.device, settings.time);
    let checkpoint = load_checkpoint(&settings.model, device)?;
    let shape = checkpoint.shape.clone();
    let mut identity = json!({"model": settings.model, "weights_sha256": checkpoint.weights_sha256, "shape": shape});
    let tokens = read_tokens(&settings.tokens, shape.vocab)?;
    identity["tokens_sha256"] = json!(uor_r4_training::sha256_file(&settings.tokens)?);
    let starts = evenly_spaced(tokens.len(), time, settings.windows)?;
    let chunks = chunks(&tokens, &starts, time, settings.batch);
    let share = 1.0 / chunks.len() as f64;
    let teacher = match &settings.teacher {
        None => None,
        Some(dir) => {
            let loaded = load_checkpoint(dir, device)?;
            if loaded.shape.vocab != shape.vocab {
                return Err(invalid("teacher and model vocabularies differ"));
            }
            identity["teacher"] = json!(dir);
            identity["teacher_sha256"] = json!(loaded.weights_sha256);
            Some(KappaLlama::new(
                loaded,
                ScoreKind::Dot,
                0.0,
                Trainable::Scalars,
                device,
            )?)
        }
    };
    let cost_layers: Vec<usize> = match &settings.cost_layers {
        None => (0..shape.layers).collect(),
        Some(layers) => {
            if layers.iter().any(|&l| l >= shape.layers) {
                return Err(invalid("cost layer out of range"));
            }
            layers.clone()
        }
    };

    // 1. The checkpoint's own attention: statistics and reference losses.
    let dot = KappaLlama::new(
        clone_checkpoint(&checkpoint),
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        device,
    )?;
    let mut accumulator = HeadStatisticsAccumulator::new(&shape, &[0.02, 0.05], 32)?;
    let (mut dot_nll, mut dot_kd) = (0.0, 0.0);
    for (inputs, targets, n) in &chunks {
        let logits = accumulator.add(&dot, inputs, *n, time)?;
        dot_nll += scalar(&next_token_nll(&logits, targets)?)? * share;
        if let Some(teacher) = &teacher {
            let target = teacher.forward(inputs, *n, time, true)?;
            dot_kd += scalar(&distillation_kl(&logits, &target)?)? * share;
        }
    }
    let statistics = accumulator.finish()?;
    let key_sq = statistics.mean_key_sq.clone();
    eprintln!("statistics done at {:.0}s", started.elapsed().as_secs_f64());

    // 2. Flat-limit drive dL/dt per head (one backward pass per chunk).
    let mut drives = serde_json::Map::new();
    let mut nll_drive: BTreeMap<&str, Vec<Vec<f64>>> = BTreeMap::new();
    for (name, kind) in [
        ("intrinsic", ScoreKind::IntrinsicLinear),
        ("key_norm", ScoreKind::KeyNormLinear),
    ] {
        let mut linear = KappaLlama::new(
            clone_checkpoint(&checkpoint),
            kind,
            0.0,
            Trainable::Scalars,
            device,
        )?;
        linear.set_norm_scale(&key_sq)?;
        let lambda = linear
            .variables()
            .get(LAMBDA)
            .ok_or_else(|| invalid("first-order model lacks its coefficient"))?
            .as_tensor()
            .clone();
        let (mut nll_samples, mut kd_samples, mut self_max) = (Vec::new(), Vec::new(), 0f32);
        for (inputs, targets, n) in &chunks {
            let logits = linear.forward(inputs, *n, time, false)?;
            let loss = next_token_nll(&logits, targets)?;
            nll_samples.push(gradient_grid(&loss.backward()?, &lambda)?);
            if let Some(teacher) = &teacher {
                let target = teacher.forward(inputs, *n, time, true)?;
                let logits = linear.forward(inputs, *n, time, false)?;
                let loss = distillation_kl(&logits, &target)?;
                kd_samples.push(gradient_grid(&loss.backward()?, &lambda)?);
            }
            let reference = dot.forward(inputs, *n, time, true)?;
            let logits = linear.forward(inputs, *n, time, false)?;
            let loss = distillation_kl(&logits, &reference)?;
            let sanity = gradient_grid(&loss.backward()?, &lambda)?;
            self_max = sanity
                .iter()
                .flatten()
                .fold(self_max, |m, v| m.max(v.abs()));
        }
        let (nll_mean, nll_sd) = mean_sd(&nll_samples);
        let kd = if kd_samples.is_empty() {
            Value::Null
        } else {
            let (mean, sd) = mean_sd(&kd_samples);
            json!({"mean": mean, "sd": sd})
        };
        drives.insert(
            name.to_owned(),
            json!({
                "nll": {"mean": nll_mean, "sd": nll_sd},
                "kd": kd,
                "self_distillation_max_abs": self_max,
            }),
        );
        nll_drive.insert(name, nll_mean);
        eprintln!(
            "{name} drive done at {:.0}s",
            started.elapsed().as_secs_f64()
        );
    }

    // 3. Zero-shot cost of curvature, one layer at a time, and the pre-registered rule.
    let mut costs = serde_json::Map::new();
    let mut rules = serde_json::Map::new();
    let mut any_pass = false;
    let flat_grid = vec![vec![FLAT_LOG_EPS; shape.heads]; shape.layers];
    for (name, kind) in [
        ("intrinsic", ScoreKind::Intrinsic),
        ("key_norm", ScoreKind::KeyNorm),
    ] {
        let curved = KappaLlama::new(
            clone_checkpoint(&checkpoint),
            kind,
            FLAT_LOG_EPS,
            Trainable::Scalars,
            device,
        )?;
        let mean_nll = |model: &KappaLlama| -> Result<f64> {
            let mut total = 0.0;
            for (inputs, targets, n) in &chunks {
                let logits = model.forward(inputs, *n, time, true)?;
                total += scalar(&next_token_nll(&logits, targets)?)? * share;
            }
            Ok(total)
        };
        let base = mean_nll(&curved)?;
        let drive = nll_drive
            .get(name)
            .ok_or_else(|| invalid("missing first-order drive"))?;
        let (mut rows, mut rule_rows) = (Vec::new(), Vec::new());
        for &layer in &cost_layers {
            let mut at_one = f64::NAN;
            for &t in &settings.grid {
                let mut grid = flat_grid.clone();
                grid[layer] = log_eps_for(t, &key_sq[layer..layer + 1])
                    .into_iter()
                    .next()
                    .ok_or_else(|| invalid("empty layer grid"))?;
                curved.set_log_eps_grid(&grid)?;
                let change = mean_nll(&curved)? - base;
                if t == 1.0 {
                    at_one = change;
                }
                rows.push(json!({"layer": layer, "t": t, "delta_nll": change}));
            }
            let gain: f64 = -drive[layer].iter().sum::<f64>();
            let pass = gain > at_one;
            any_pass |= pass;
            rule_rows.push(json!({
                "layer": layer,
                "first_order_gain_at_t1": gain,
                "measured_change_at_t1": at_one,
                "pass": pass,
            }));
        }
        costs.insert(name.to_owned(), json!({"flat_nll": base, "rows": rows}));
        rules.insert(name.to_owned(), json!(rule_rows));
        eprintln!(
            "{name} costs done at {:.0}s",
            started.elapsed().as_secs_f64()
        );
    }

    let report = json!({
        "schema": "uor-r4.kappa-conversion-drive/1",
        "identity": identity,
        "settings": {"batch": settings.batch, "time": time, "windows": settings.windows,
                     "t_grid": settings.grid, "cost_layers": cost_layers, "window_starts": starts},
        "units": "nats per token; drive is dL/dt per head at t = 0 with t = kappa * mean |k|^2",
        "statistics": statistics,
        "dot_nll": dot_nll,
        "dot_kl_to_teacher": teacher.as_ref().map(|_| dot_kd),
        "drive": drives,
        "zero_shot_cost": costs,
        "rule": {
            "statement": "per layer, curvature training is warranted only if -sum_h dL/dt_h exceeds the measured zero-shot NLL change at t = 1",
            "per_layer": rules,
            "any_layer_passes": any_pass,
        },
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("drive.json"), serde_json::to_vec_pretty(&report)?)?;
    eprintln!("rule: any layer passes = {any_pass}");
    Ok(())
}

struct Evaluation {
    nll: f64,
    kl: Option<f64>,
}

fn evaluate(
    student: &KappaLlama,
    teacher: Option<&KappaLlama>,
    chunks: &[Chunk],
    time: usize,
) -> Result<Evaluation> {
    let (mut nll, mut kl, mut rows) = (0.0, 0.0, 0usize);
    for (inputs, targets, n) in chunks {
        let logits = student.forward(inputs, *n, time, true)?;
        nll += scalar(&next_token_nll(&logits, targets)?)? * *n as f64;
        if let Some(teacher) = teacher {
            let target = teacher.forward(inputs, *n, time, true)?;
            kl += scalar(&distillation_kl(&logits, &target)?)? * *n as f64;
        }
        rows += n;
    }
    Ok(Evaluation {
        nll: nll / rows as f64,
        kl: teacher.map(|_| kl / rows as f64),
    })
}

/// Rising lower bound on every head's dimensionless curvature.
#[derive(Clone, Copy, Debug)]
struct Anneal {
    t: f64,
    steps: usize,
    hold: bool,
}

impl Anneal {
    fn floor(&self, step: usize) -> Option<f64> {
        if step > self.steps && !self.hold {
            return None;
        }
        Some(self.t * (step as f64 / self.steps as f64).min(1.0))
    }
}

struct TrainSettings {
    device: Device,
    student: PathBuf,
    teacher: Option<PathBuf>,
    train: PathBuf,
    valid: PathBuf,
    score: ScoreKind,
    trainable: Trainable,
    init_t: f64,
    steps: usize,
    batch: usize,
    accumulate: usize,
    time: usize,
    lr: f64,
    curv_lr: f64,
    eval_every: usize,
    eval_windows: usize,
    seed: u64,
    max_seconds: f64,
    save_model: bool,
    anneal: Option<Anneal>,
}

fn train_settings(args: &Args) -> Result<TrainSettings> {
    let steps: usize = args.number("steps", 500)?;
    let score = ScoreKind::parse(args.text("score").unwrap_or("intrinsic"))?;
    let anneal = match args.text("anneal_t") {
        None => None,
        Some(text) => {
            let t: f64 = text
                .parse()
                .ok()
                .filter(|t: &f64| t.is_finite() && *t > 0.0 && *t <= 100.0)
                .ok_or_else(|| invalid(format!("anneal_t must lie in (0, 100], not {text}")))?;
            if !score.is_curved() {
                return Err(invalid(
                    "anneal_t needs a curved score (intrinsic or key_norm)",
                ));
            }
            let anneal_steps: usize = args.number("anneal_steps", steps.div_ceil(2))?;
            if anneal_steps == 0 {
                return Err(invalid("anneal_steps must be positive"));
            }
            Some(Anneal {
                t,
                steps: anneal_steps,
                hold: args.flag("anneal_hold", true)?,
            })
        }
    };
    let settings = TrainSettings {
        device: device(args)?,
        student: PathBuf::from(args.required("student")?),
        teacher: args.text("teacher").map(PathBuf::from),
        train: PathBuf::from(args.required("train")?),
        valid: PathBuf::from(args.required("valid")?),
        score,
        trainable: Trainable::parse(args.text("trainable").unwrap_or("scalars"))?,
        init_t: args.number("init_t", 1e-5)?,
        steps,
        batch: args.number("batch", 1)?,
        accumulate: args.number("accumulate", 4)?,
        time: args.number("time", 256)?,
        lr: args.number("lr", 1e-5)?,
        curv_lr: args.number("curv_lr", 1e-2)?,
        eval_every: args.number("eval_every", 100)?,
        eval_windows: args.number("eval_windows", 16)?,
        seed: args.number("seed", 1)?,
        max_seconds: args.number("max_seconds", 0.0)?,
        save_model: args.flag("save_model", true)?,
        anneal,
    };
    if settings.steps == 0
        || settings.batch == 0
        || settings.accumulate == 0
        || settings.time < 2
        || settings.eval_every == 0
        || settings.eval_windows == 0
        || !(settings.lr > 0.0 && settings.curv_lr > 0.0)
        || !(settings.init_t > 0.0 && settings.init_t <= 100.0)
        || !settings.max_seconds.is_finite()
        || settings.max_seconds < 0.0
    {
        return Err(invalid(
            "steps, batch, accumulate, eval_every, eval_windows, lr and curv_lr must be positive; init_t in (0, 100]; time >= 2",
        ));
    }
    Ok(settings)
}

fn train(settings: &TrainSettings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let device = &settings.device;
    let (score, time, batch) = (settings.score, settings.time, settings.batch);
    let checkpoint = load_checkpoint(&settings.student, device)?;
    let vocab = checkpoint.shape.vocab;
    let (layers, heads) = (checkpoint.shape.layers, checkpoint.shape.heads);
    let mut identity = json!({
        "student": settings.student, "student_sha256": checkpoint.weights_sha256, "shape": checkpoint.shape,
    });
    let teacher = match &settings.teacher {
        None => None,
        Some(dir) => {
            let loaded = load_checkpoint(dir, device)?;
            if loaded.shape.vocab != vocab {
                return Err(invalid("teacher and student vocabularies differ"));
            }
            identity["teacher"] = json!(dir);
            identity["teacher_sha256"] = json!(loaded.weights_sha256);
            Some(KappaLlama::new(
                loaded,
                ScoreKind::Dot,
                0.0,
                Trainable::Scalars,
                device,
            )?)
        }
    };
    let mut student = KappaLlama::new(checkpoint, score, FLAT_LOG_EPS, settings.trainable, device)?;
    let train_tokens = read_tokens(&settings.train, vocab)?;
    let valid_tokens = read_tokens(&settings.valid, vocab)?;
    identity["train_sha256"] = json!(uor_r4_training::sha256_file(&settings.train)?);
    identity["valid_sha256"] = json!(uor_r4_training::sha256_file(&settings.valid)?);
    if train_tokens.len() < time + 2 {
        return Err(invalid("train file shorter than one window"));
    }
    let valid_starts = evenly_spaced(valid_tokens.len(), time, settings.eval_windows)?;
    let valid_chunks = chunks(&valid_tokens, &valid_starts, time, batch);

    // Calibrate the dimensionless curvature unit on validation windows; the
    // flat student is the checkpoint's own attention.
    let calibration = calibrate(&student, &valid_chunks[..valid_chunks.len().min(4)], time)?;
    if score.is_curved() {
        student.set_log_eps_grid(&log_eps_for(settings.init_t, &calibration))?;
    }
    if score.is_linear() {
        student.set_norm_scale(&calibration)?;
    }

    // Curvature/temperature scalars and checkpoint weights get separate step
    // sizes and no weight decay (decay would pull log_eps toward kappa = 1).
    let scalar_names = [LOG_EPS, LOG_BETA, LAMBDA];
    let split = |scalars: bool| -> BTreeMap<String, candle_core::Var> {
        student
            .variables()
            .iter()
            .filter(|(name, _)| scalar_names.contains(&name.as_str()) == scalars)
            .map(|(name, var)| (name.clone(), var.clone()))
            .collect()
    };
    let scalar_vars = split(true);
    let weight_vars = split(false);
    let config = |learning_rate: f64| AdamConfig {
        learning_rate,
        weight_decay: 0.0,
        ..AdamConfig::default()
    };
    let mut scalar_opt = NamedAdamW::new(&scalar_vars, config(settings.curv_lr))?;
    let mut weight_opt = if weight_vars.is_empty() {
        None
    } else {
        Some(NamedAdamW::new(&weight_vars, config(settings.lr))?)
    };

    let mut key_sq = calibration.clone();
    let mut progress = Vec::new();
    let record = |step: usize,
                  recent: Option<f64>,
                  model: &KappaLlama,
                  key_sq: &[Vec<f32>]|
     -> Result<Value> {
        let eval = evaluate(model, teacher.as_ref(), &valid_chunks, time)?;
        let kappa = model.curvature()?;
        let t = if kappa.is_empty() {
            Vec::new()
        } else {
            dimensionless(&kappa, key_sq)
        };
        let row = json!({
            "step": step,
            "train_objective_recent": recent,
            "valid_nll": eval.nll,
            "valid_kl_to_teacher": eval.kl,
            "t": summary(&t),
            "t_floor": settings.anneal.and_then(|a| a.floor(step)),
            "first_order_t": summary(&model.first_order_coefficient()?),
            "temperature": summary(&model.temperature()?),
            "wall": started.elapsed().as_secs_f64(),
        });
        eprintln!("{row}");
        Ok(row)
    };
    progress.push(record(0, None, &student, &key_sq)?);
    let mut sampler = Windows(settings.seed);
    let mut recent = Vec::new();
    let mut completed = 0usize;
    for step in 1..=settings.steps {
        if settings.max_seconds > 0.0 && started.elapsed().as_secs_f64() > settings.max_seconds {
            break;
        }
        let mut sums: BTreeMap<String, Tensor> = BTreeMap::new();
        let mut last: Option<GradStore> = None;
        let mut objective = 0.0;
        for _ in 0..settings.accumulate {
            let starts: Vec<usize> = (0..batch)
                .map(|_| (sampler.next() % (train_tokens.len() - time - 1) as u64) as usize)
                .collect();
            let (inputs, targets) = windows(&train_tokens, &starts, time);
            let mut probe = |layer: usize, _: &Tensor, key: &Tensor, _: &Tensor| -> Result<()> {
                key_sq[layer] = mean_key_sq(&key.detach())?;
                Ok(())
            };
            let logits = student.forward_with_probe(&inputs, batch, time, false, &mut probe)?;
            let loss = match &teacher {
                Some(teacher) => {
                    distillation_kl(&logits, &teacher.forward(&inputs, batch, time, true)?)?
                }
                None => next_token_nll(&logits, &targets)?,
            };
            objective += scalar(&loss)? / settings.accumulate as f64;
            let gradients = loss
                .affine(1.0 / settings.accumulate as f64, 0.0)?
                .backward()?;
            for (name, var) in student.variables() {
                if let Some(g) = gradients.get(var.as_tensor()) {
                    let sum = match sums.remove(name) {
                        Some(previous) => previous.add(g)?,
                        None => g.clone(),
                    };
                    sums.insert(name.clone(), sum);
                }
            }
            last = Some(gradients);
        }
        let mut gradients = last.ok_or_else(|| invalid("no micro-batch ran"))?;
        for (name, var) in student.variables() {
            if let Some(sum) = sums.remove(name) {
                gradients.insert(var.as_tensor(), sum);
            }
        }
        recent.push(objective);
        scalar_opt.step(&scalar_vars, &gradients)?;
        if let Some(optimizer) = weight_opt.as_mut() {
            optimizer.step(&weight_vars, &gradients)?;
        }
        if let Some(floor) = settings.anneal.and_then(|a| a.floor(step)) {
            if floor > 0.0 {
                student.floor_log_eps_grid(&log_eps_for(floor, &key_sq))?;
            }
        }
        completed = step;
        if step % settings.eval_every == 0 || step == settings.steps {
            let window = recent.len().min(settings.eval_every);
            let mean = recent[recent.len() - window..].iter().sum::<f64>() / window as f64;
            progress.push(record(step, Some(mean), &student, &key_sq)?);
            fs::write(
                out.join("progress.json"),
                serde_json::to_vec_pretty(&progress)?,
            )?;
        }
    }
    if progress.last().and_then(|row| row["step"].as_u64()) != Some(completed as u64) {
        let window = recent.len().clamp(1, settings.eval_every);
        let mean = recent.iter().rev().take(window).sum::<f64>() / window as f64;
        progress.push(record(completed, Some(mean), &student, &key_sq)?);
    }
    if settings.save_model {
        let directory = out.join("model");
        fs::create_dir(&directory)?;
        let mut tensors: HashMap<String, Tensor> = student
            .variables()
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        if score.is_linear() {
            let scale: Vec<f32> = calibration.iter().flatten().copied().collect();
            tensors.insert(
                NORM_SCALE.to_owned(),
                Tensor::from_vec(scale, (layers, heads), device)?,
            );
        }
        candle_core::safetensors::save(&tensors, directory.join("variables.safetensors"))?;
    }
    let kappa = student.curvature()?;
    let t_final = if kappa.is_empty() {
        Vec::new()
    } else {
        dimensionless(&kappa, &key_sq)
    };
    let report = json!({
        "schema": "uor-r4.kappa-conversion-train/2",
        "identity": identity,
        "settings": {
            "score": score, "trainable": settings.trainable, "init_t": settings.init_t,
            "steps": settings.steps, "completed_steps": completed, "batch": batch,
            "accumulate": settings.accumulate, "time": time, "lr": settings.lr,
            "curv_lr": settings.curv_lr, "eval_every": settings.eval_every,
            "eval_windows": settings.eval_windows, "seed": settings.seed,
            "anneal": settings.anneal.map(|a| json!({"t": a.t, "steps": a.steps, "hold": a.hold})),
            "objective": if teacher.is_some() { "kl_to_dot_teacher" } else { "next_token_nll" },
        },
        "calibration_mean_key_sq": calibration,
        "curvature_final": kappa,
        "t_final": t_final,
        "first_order_t_final": student.first_order_coefficient()?,
        "temperature_final": student.temperature()?,
        "progress": progress,
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn sample(args: &Args) -> Result<()> {
    let device = device(args)?;
    let model_dir = PathBuf::from(args.required("model")?);
    let score = ScoreKind::parse(args.text("score").unwrap_or("dot"))?;
    let trainable = Trainable::parse(args.text("trainable").unwrap_or("scalars"))?;
    let count: usize = args.number("tokens", 64)?;
    let prompt = args.required("prompt")?;
    let system = args.text("system").unwrap_or(SMOLLM2_SYSTEM);
    let checkpoint = load_checkpoint(&model_dir, &device)?;
    let tokenizer = HfBpeTokenizer::from_dir(&model_dir)
        .map_err(|error| invalid(format!("tokenizer: {error}")))?;
    let mut model = KappaLlama::new(checkpoint, score, FLAT_LOG_EPS, trainable, &device)?;
    match args.text("variables") {
        Some(path) => {
            let saved = candle_core::safetensors::load(path, &device)?;
            let mut expected: BTreeSet<String> = model.variables().keys().cloned().collect();
            if score.is_linear() {
                expected.insert(NORM_SCALE.to_owned());
            }
            let found: BTreeSet<String> = saved.keys().cloned().collect();
            if found != expected {
                return Err(invalid(format!(
                    "{path} holds {found:?} but score={score:?} trainable={trainable:?} expects {expected:?}; pass the run's score= and trainable="
                )));
            }
            for (name, var) in model.variables() {
                let tensor = saved
                    .get(name)
                    .ok_or_else(|| invalid(format!("{path} lacks {name}")))?;
                var.set(tensor)?;
            }
            if let Some(scale) = saved.get(NORM_SCALE) {
                model.set_norm_scale(&scale.to_vec2::<f32>()?)?;
            }
        }
        None if score != ScoreKind::Dot => {
            eprintln!(
                "note: no variables= given, so the {score:?} heads are at the flat limit (the checkpoint's own attention)"
            );
        }
        None => {}
    }
    let raw = args.text("raw") == Some("true");
    let chat = if raw {
        prompt.to_owned()
    } else {
        format!(
            "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n"
        )
    };
    let mut ids = tokenizer.encode(&chat);
    let stop = tokenizer.encode("<|im_end|>");
    let prompt_len = ids.len();
    for _ in 0..count {
        let logits = model.forward(&ids, 1, ids.len(), true)?;
        let last = logits.narrow(1, ids.len() - 1, 1)?.flatten_all()?;
        let next = last.argmax(0)?.to_scalar::<u32>()?;
        ids.push(next);
        if stop.len() == 1 && next == stop[0] {
            break;
        }
    }
    println!("{}", tokenizer.decode(&ids[prompt_len..]));
    Ok(())
}

enum Run {
    Probe(ProbeSettings),
    Drive(DriveSettings),
    Train(Box<TrainSettings>),
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
    let run = match args.required("mode")? {
        "tokenize" => return tokenize(&args),
        "sample" => return sample(&args),
        "probe" => Run::Probe(probe_settings(&args)?),
        "drive" => Run::Drive(drive_settings(&args)?),
        "train" => Run::Train(Box::new(train_settings(&args)?)),
        other => return Err(invalid(format!("unknown mode {other}"))),
    };
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = match &run {
        Run::Probe(settings) => probe(settings, &out),
        Run::Drive(settings) => drive(settings, &out),
        Run::Train(settings) => train(settings, &out),
    };
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
