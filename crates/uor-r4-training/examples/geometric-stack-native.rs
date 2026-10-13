//! Admitted CUDA native rung0 training and masked evaluation. Historical CLI remains inactive.
use candle_core::Device;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_training::geometric_stack::{
    Gate, Objective, QuaternionWeights, ReadScore, RecGates, StackAdamW, StackArch, StackConfig,
    StackModel,
};
use uor_r4_training::{sha256_file, Result, TrainingError};
#[path = "geometric-stack/native_rung0.rs"]
mod native_rung0;
fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}
struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }

    fn optional(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn uniform(&mut self) -> f64 {
        ((self.next() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
}

fn read_tokens(path: &Path, vocabulary: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"UORT") {
        let reader = uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader::open(path)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        if reader.vocab_size() as usize > vocabulary {
            return Err(invalid(format!(
                "{} declares a larger vocabulary",
                path.display()
            )));
        }
        let tokens: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        if tokens.iter().any(|&id| id as usize >= vocabulary) {
            return Err(invalid(format!(
                "{} has ids outside the vocabulary",
                path.display()
            )));
        }
        return Ok(tokens);
    }
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
    Ok(json!({"path": path, "bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
}

fn native_bos_identity(path: &Path) -> Result<u32> {
    let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
        .ok_or_else(|| {
            invalid("native dev BOS identity needs the pinned readable tokenizer.json")
        })?;
    Ok(DialogueProtocol::literal_roles_v2(&tokenizer)
        .map_err(|e| invalid(format!("native dev tokenizer special-token identity: {e}")))?
        .bos_id)
}

const NATIVE_COUNT_REFERENCE_POLICY: &str =
    "train-response-masked-targets-with-within-document-predecessor/v1";
const NATIVE_COUNT_REFERENCE_FILE: &str = "native_count_reference.json";

fn native_input_content(path: &Path) -> Result<Value> {
    Ok(json!({"bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
}

/// Input admission only: no model, frequency counts, ranking or score arithmetic.
fn native_train_reference_options(
    streams: usize,
    mask: Option<&Path>,
    expected_targets: usize,
) -> Result<()> {
    if streams != 1 || mask.is_none() || expected_targets == 0 {
        return Err(invalid("native rung0 requires exactly one train=TRAIN.uort stream, train_mask=TRAIN.response_mask.u8 and positive expected_train_targets=2425925 (or the pinned panel's count); no DEV fallback"));
    }
    Ok(())
}

fn native_reference_binding(
    train_content: &Value,
    mask_content: &Value,
    tokenizer_content: &Value,
    reference: &native_rung0::MaskedTargets,
    expected_targets: usize,
    bos_id: u32,
) -> Result<Value> {
    if expected_targets == 0 || reference.len() != expected_targets {
        return Err(invalid(format!("TRAIN masked response targets differ from expected_train_targets: {} != {expected_targets}", reference.len())));
    }
    Ok(json!({
        "schema": "uor-r4.native-count-reference/1", "source_split": "train",
        "frequency_policy": NATIVE_COUNT_REFERENCE_POLICY,
        "train": train_content, "train_mask": mask_content, "tokenizer": tokenizer_content,
        "payload_tokens": reference.payload_tokens(), "reference_targets": reference.len(),
        "expected_train_targets": expected_targets, "bos_id": bos_id,
    }))
}

fn require_bound_native_reference(saved: &Value, requested: &Value) -> Result<()> {
    if saved != requested
        || saved["schema"] != "uor-r4.native-count-reference/1"
        || saved["source_split"] != "train"
        || saved["frequency_policy"] != NATIVE_COUNT_REFERENCE_POLICY
    {
        return Err(invalid("native count-reference binding is missing, stale or differs from TRAIN tokens/mask/tokenizer/policy/counts; no DEV reference fallback or old native lineage admission"));
    }
    Ok(())
}

fn save_native_reference_binding(model_dir: &Path, lineage: &Value) -> Result<()> {
    if let Some(binding) = lineage.get("native_count_reference") {
        fs::write(
            model_dir.join(NATIVE_COUNT_REFERENCE_FILE),
            serde_json::to_vec_pretty(binding)?,
        )?;
    }
    Ok(())
}

fn native_process_options(
    sample_tokens: usize,
    max_seconds: f64,
    host_nan_trace: bool,
) -> Result<()> {
    if sample_tokens != 0 || max_seconds.is_nan() || max_seconds <= 0.0 || host_nan_trace {
        return Err(invalid("native rung0 refuses host sampling and host tensor diagnostics; sample_tokens=0, positive max_seconds estimate, UOR_NAN_TRACE must not be1 (healthy work extends a self-set estimate)"));
    }
    Ok(())
}

/// Metadata admission before evaluation/update: state lineage alone cannot
/// authenticate a separately swapped model config or optimizer configuration.
fn require_loaded_runtime_metadata(
    loaded: &StackConfig,
    requested: &StackConfig,
    optimizer_decay: f64,
    optimizer_clip: f64,
    requested_decay: f64,
    requested_clip: f64,
) -> Result<()> {
    if loaded != requested || optimizer_decay != requested_decay || optimizer_clip != requested_clip
    {
        return Err(invalid("loaded model configuration or optimizer decay/clip differs from requested checkpoint lineage"));
    }
    Ok(())
}

fn require_native_training_config(
    config: &StackConfig,
    native_head: bool,
    projection: Option<QuaternionWeights>,
    native_read: bool,
) -> Result<()> {
    if !native_head
        || projection != Some(QuaternionWeights::SpiralcoreFbs)
        || config.arch != StackArch::Geometric
        || config.gate != Gate::Hamilton
        || config.rec_gates != RecGates::Geometric
        || !matches!(config.objective, Objective::MarginHinge { rivals: 1 | 8 })
        || config.pointer.is_some()
        || config.select.is_some()
        || config.memory.is_some()
        || config.pattern.is_empty()
        || config
            .pattern
            .chars()
            .any(|c| c != 'r' && !(c == 'a' && native_read))
        || (native_read && !config.pattern.contains('a'))
    {
        return Err(invalid("new train runs require the native path: arch=geometric, pattern of r (plus a only with native_read=starling_native:M:K:E), gate=hamilton, rec_gates=geometric, projection=spiralcore_fbs, head=spiralcore_product, objective=hinge|hinge8; no pointer/historical read/memory or historical fallback"));
    }
    uor_r4_training::native_vector_writer::WriterSpec {
        vocabulary: config.vocab_size,
        width: config.width,
        kept_cells: uor_r4_training::native_vector_writer::DEFAULT_KEPT_CELLS,
    }
    .validate()?;
    Ok(())
}

struct Progress {
    step: usize,
    rng: Rng,
    curve: Vec<Value>,
    train_seconds: f64,
}

fn save_checkpoint(
    out: &Path,
    model: &StackModel,
    optimizer: &StackAdamW,
    progress: &Progress,
    lineage: &Value,
) -> Result<()> {
    let staging = out.join("checkpoint.partial");
    let _ = fs::remove_dir_all(&staging);
    model.save(&staging.join("model"))?;
    save_native_reference_binding(&staging.join("model"), lineage)?;
    optimizer.save(&staging.join("optimizer"))?;
    fs::write(
        staging.join("state.json"),
        serde_json::to_vec_pretty(&json!({
            "step": progress.step, "rng": progress.rng.0.to_string(), "curve": progress.curve,
            "train_seconds": progress.train_seconds, "lineage": lineage,
        }))?,
    )?;
    let final_path = out.join("checkpoint");
    let _ = fs::remove_dir_all(&final_path);
    fs::rename(&staging, &final_path)?;
    Ok(())
}

/// The model, optimizer and progress a stopped run saved, if its lineage is
/// `lineage`; with the identity of its state file.
fn load_checkpoint(
    checkpoint: &Path,
    lineage: &Value,
    device: &Device,
) -> Result<(StackModel, StackAdamW, Progress, Value)> {
    let state: Value = serde_json::from_slice(&fs::read(checkpoint.join("state.json"))?)?;
    if &state["lineage"] != lineage {
        let keys = |value: &Value| -> Vec<String> {
            value
                .as_object()
                .map(|object| object.keys().cloned().collect())
                .unwrap_or_default()
        };
        let mut differing: Vec<String> = keys(lineage)
            .into_iter()
            .chain(keys(&state["lineage"]))
            .filter(|key| state["lineage"][key.as_str()] != lineage[key.as_str()])
            .collect();
        differing.sort();
        differing.dedup();
        return Err(invalid(format!(
            "the resume differs from the checkpoint's lineage in: {}",
            differing.join(", ")
        )));
    }
    let model = StackModel::load(&checkpoint.join("model"), device)?;
    let optimizer = StackAdamW::load(&checkpoint.join("optimizer"), &model)?;
    let progress = Progress {
        step: state["step"]
            .as_u64()
            .ok_or_else(|| invalid("checkpoint step"))? as usize,
        rng: Rng(state["rng"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| invalid("checkpoint rng"))?),
        curve: state["curve"].as_array().cloned().unwrap_or_default(),
        train_seconds: state["train_seconds"].as_f64().unwrap_or(0.0),
    };
    if optimizer.step != progress.step {
        return Err(invalid("checkpoint optimizer and progress steps differ"));
    }
    Ok((
        model,
        optimizer,
        progress,
        identity(&checkpoint.join("state.json"))?,
    ))
}

fn cosine_rate(lr: f64, warmup: usize, min_lr: f64, steps: usize, step: usize) -> f64 {
    if step < warmup {
        return lr * (step + 1) as f64 / warmup as f64;
    }
    let span = (steps - warmup).max(1) as f64;
    let progress = ((step - warmup) as f64 / span).min(1.0);
    let floor = lr * min_lr;
    floor + (lr - floor) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
}

// Only finished CUDA scalar receipts reach this metadata wrapper.
enum Evaluation {
    Native(Value),
}
impl Evaluation {
    fn record(&self) -> Value {
        match self {
            Self::Native(v) => v.clone(),
        }
    }
}

const TRAIN_ARGS: &[&str] = &[
    "train",
    "train_mask",
    "expected_train_targets",
    "valid",
    "valid_mask",
    "tokenizer",
    "expected_dev_targets",
    "out",
    "arch",
    "pattern",
    "width",
    "heads",
    "layers",
    "mlp",
    "stack_mlp",
    "context",
    "seed",
    "gate",
    "rec_gates",
    "projection",
    "head",
    "objective",
    "device",
    "precision",
    "data_parallel",
    "tf32",
    "read",
    "rotation",
    "rotation_group",
    "steps",
    "batch",
    "lr",
    "warmup",
    "min_lr",
    "weight_decay",
    "clip",
    "eval_every",
    "eval_windows",
    "final_windows",
    "checkpoint_every",
    "resume",
    "init",
    "max_seconds",
    "sample_tokens",
];
fn cuda_device(args: &Args) -> Result<Device> {
    if args.optional("device").as_deref() != Some("cuda") {
        return Err(invalid(
            "native execution requires explicit device=cuda; no host fallback",
        ));
    }
    for (key, want) in [
        ("precision", "f32"),
        ("data_parallel", "1"),
        ("tf32", "false"),
        ("read", "lorentz"),
        ("rotation", "true"),
        ("rotation_group", "quaternion"),
    ] {
        if args.optional(key).is_some_and(|v| v != want) {
            return Err(invalid(format!("native rung0 requires {key}={want}")));
        }
    }
    Device::new_cuda(0).map_err(Into::into)
}
fn config_from_args(args: &Args) -> Result<StackConfig> {
    let init = args.optional("init").map(PathBuf::from);
    let mut config = if let Some(ref dir) = init {
        serde_json::from_slice::<StackConfig>(&fs::read(dir.join("config.json"))?)?
    } else {
        let mut c = StackConfig::transformer(
            args.number("width", 256)?,
            args.number("heads", 4)?,
            args.number("layers", 6)?,
            args.number("mlp", 768)?,
            args.number("context", 384)?,
            args.number("seed", 1)?,
        )?;
        c.arch = StackArch::Geometric;
        c.pattern = args
            .optional("pattern")
            .unwrap_or_else(|| "r".repeat(c.layers()));
        c.mlp_hidden = args.number("stack_mlp", 384)?;
        c.read = ReadScore::Lorentz;
        c.rotation = true;
        c.gate = Gate::Hamilton;
        c.rec_gates = RecGates::Geometric;
        c.objective = match args.optional("objective").as_deref() {
            Some("hinge") => Objective::MarginHinge { rivals: 1 },
            Some("hinge8") => Objective::MarginHinge { rivals: 8 },
            _ => {
                return Err(invalid(
                    "native objective must be explicitly hinge or hinge8",
                ))
            }
        };
        c
    };
    for (key, want) in [
        ("arch", "geometric"),
        ("gate", "hamilton"),
        ("rec_gates", "geometric"),
        ("projection", "spiralcore_fbs"),
        ("head", "spiralcore_product"),
    ] {
        if args.optional(key).as_deref() != Some(want) {
            return Err(invalid(format!(
                "native rung0 requires explicit {key}={want}"
            )));
        }
    }
    if init.is_some() {
        for (key, actual) in [
            ("width", config.width),
            ("heads", config.heads),
            ("layers", config.layers()),
            ("stack_mlp", config.mlp_hidden),
            ("context", config.context),
            ("seed", config.seed as usize),
        ] {
            if args.optional(key).is_some() && args.number(key, actual)? != actual {
                return Err(invalid(format!("{key} differs from init config")));
            }
        }
        if args
            .optional("pattern")
            .is_some_and(|p| p != config.pattern)
        {
            return Err(invalid("pattern differs from init config"));
        }
        let objective = match args.required("objective")?.as_str() {
            "hinge" => Objective::MarginHinge { rivals: 1 },
            "hinge8" => Objective::MarginHinge { rivals: 8 },
            _ => return Err(invalid("native objective hinge|hinge8 required")),
        };
        if config.objective != objective {
            return Err(invalid("objective differs from init config"));
        }
    }
    require_native_training_config(&config, true, Some(QuaternionWeights::SpiralcoreFbs), false)?;
    config.validate()?;
    Ok(config)
}
struct Settings {
    args: Args,
    config: StackConfig,
    train: PathBuf,
    train_mask: PathBuf,
    valid: PathBuf,
    valid_mask: PathBuf,
    tokenizer: PathBuf,
    steps: usize,
    batch: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    eval_windows: usize,
    final_windows: usize,
    checkpoint_every: usize,
    max_seconds: f64,
}
impl Settings {
    fn new(args: Args) -> Result<Self> {
        native_process_options(
            args.number("sample_tokens", 0)?,
            args.number("max_seconds", f64::INFINITY)?,
            std::env::var("UOR_NAN_TRACE").is_ok_and(|v| v == "1"),
        )?;
        let train = PathBuf::from(args.required("train")?);
        let mask = args.optional("train_mask").map(PathBuf::from);
        native_train_reference_options(
            train.to_string_lossy().split(',').count(),
            mask.as_deref(),
            args.number("expected_train_targets", 2425925)?,
        )?;
        let config = config_from_args(&args)?;
        let s = Self {
            train,
            train_mask: mask.ok_or_else(|| invalid("missing train_mask"))?,
            valid: args.required("valid")?.into(),
            valid_mask: args.required("valid_mask")?.into(),
            tokenizer: args.required("tokenizer")?.into(),
            steps: args.number("steps", 7324)?,
            batch: args.number("batch", 16)?,
            lr: args.number("lr", 0.002)?,
            warmup: args.number("warmup", 200)?,
            min_lr: args.number("min_lr", 0.1)?,
            weight_decay: args.number("weight_decay", 0.1)?,
            clip: args.number("clip", 1.0)?,
            eval_every: args.number("eval_every", 250)?,
            eval_windows: args.number("eval_windows", 111)?,
            final_windows: args.number("final_windows", 111)?,
            checkpoint_every: args.number("checkpoint_every", 250)?,
            max_seconds: args.number("max_seconds", f64::INFINITY)?,
            config,
            args,
        };
        if s.steps == 0
            || s.batch == 0
            || s.eval_every == 0
            || s.eval_windows == 0
            || s.final_windows == 0
            || !s.lr.is_finite()
            || s.lr <= 0.0
            || !(0.0..=1.0).contains(&s.min_lr)
            || s.warmup > s.steps
            || !s.weight_decay.is_finite()
            || s.weight_decay < 0.0
            || !s.clip.is_finite()
            || s.clip <= 0.0
        {
            return Err(invalid("invalid native schedule/count/optimizer settings"));
        }
        Ok(s)
    }
}
fn evaluate_native(
    model: &mut StackModel,
    reference: &native_rung0::MaskedTargets,
    valid: &[u32],
    mask: &[u8],
    plan: &native_rung0::PanelPlan,
) -> Result<Value> {
    let evaluator = native_rung0::Evaluator::new(model, reference, valid, mask, plan.clone())?;
    model.scored_in_f32(|m| evaluator.evaluate(m, valid, mask))
}
fn train(s: &Settings, out: &Path) -> Result<()> {
    // All source identities and panel/target metadata are admitted before CUDA model allocation.
    let train_content = native_input_content(&s.train)?;
    if train_content == native_input_content(&s.valid)? {
        return Err(invalid("TRAIN reference must differ from DEV content"));
    }
    let train = read_tokens(&s.train, s.config.vocab_size)?;
    let valid = read_tokens(&s.valid, s.config.vocab_size)?;
    let train_mask = fs::read(&s.train_mask)?;
    let valid_mask = fs::read(&s.valid_mask)?;
    let bos = native_bos_identity(&s.tokenizer)?;
    let reference = native_rung0::MaskedTargets::new(&train, &train_mask, bos)?;
    let binding = native_reference_binding(
        &train_content,
        &native_input_content(&s.train_mask)?,
        &native_input_content(&s.tokenizer)?,
        &reference,
        s.args.number("expected_train_targets", 2425925)?,
        bos,
    )?;
    let plan = |windows| {
        native_rung0::PanelPlan::new(
            &valid,
            &valid_mask,
            s.config.context,
            windows,
            bos,
            s.args.number("expected_dev_targets", 14287)?,
        )
    };
    let dev_plan = plan(s.eval_windows)?;
    let final_plan = plan(s.final_windows)?;
    if train.len() <= s.config.context + 1 {
        return Err(invalid("TRAIN too short for sampled input/target window"));
    }
    let mut settings =
        json!({"config":s.config,"arguments":s.args.0,"native_count_reference":binding});
    // Output/resume paths are transport, not content lineage.
    for key in ["out", "resume"] {
        settings["arguments"]
            .as_object_mut()
            .ok_or_else(|| invalid("arguments object"))?
            .remove(key);
    }
    let init = s.args.optional("init").map(PathBuf::from);
    let init_identity=init.as_ref().map(|p|->Result<Value>{Ok(json!({"config":native_input_content(&p.join("config.json"))?,"model":native_input_content(&p.join("model.safetensors"))?}))}).transpose()?;
    let lineage = json!({"schema":"uor-r4.native-rung0-lineage/2","settings":settings,"native_count_reference":binding,"valid":native_input_content(&s.valid)?,"valid_mask":native_input_content(&s.valid_mask)?,"init":init_identity});
    let device = cuda_device(&s.args)?;
    let (mut model, mut optimizer, mut progress, resumed_from) =
        if let Some(checkpoint) = s.args.optional("resume") {
            let (m, o, p, receipt) = load_checkpoint(Path::new(&checkpoint), &lineage, &device)?;
            let saved: Value = serde_json::from_slice(&fs::read(
                Path::new(&checkpoint)
                    .join("model")
                    .join(NATIVE_COUNT_REFERENCE_FILE),
            )?)?;
            require_bound_native_reference(&saved, &binding)?;
            (m, o, p, Some(receipt))
        } else {
            let mut model = match init {
                Some(ref dir) => StackModel::load(dir, &device)?,
                None => StackModel::new(s.config.clone(), &device)?,
            };
            if model.config != s.config {
                return Err(invalid(
                    "loaded model config differs from requested native config",
                ));
            }
            if model.native_vector_head_spec().is_none() {
                model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
                model.set_native_vector_head(
                    uor_r4_training::native_vector_writer::DEFAULT_KEPT_CELLS,
                )?;
            }
            let optimizer = StackAdamW::new(&model, s.weight_decay, s.clip)?;
            (
                model,
                optimizer,
                Progress {
                    step: 0,
                    rng: Rng(s.config.seed ^ 0x5749_4E44_4F57),
                    curve: Vec::new(),
                    train_seconds: 0.0,
                },
                None,
            )
        };
    require_loaded_runtime_metadata(
        &model.config,
        &s.config,
        optimizer.weight_decay,
        optimizer.clip,
        s.weight_decay,
        s.clip,
    )?;
    require_native_training_config(
        &model.config,
        model.native_vector_head_spec().is_some(),
        Some(model.quaternion_weights()),
        false,
    )?;
    if progress.step > s.steps {
        return Err(invalid("checkpoint beyond requested final step"));
    }
    if progress.step == 0 {
        let dev = evaluate_native(&mut model, &reference, &valid, &valid_mask, &dev_plan)?;
        progress.curve.push(json!({"step":0,"tokens":0,"dev":dev}));
    }
    let started = Instant::now();
    let mut extended = false;
    let mut window_loss = (0.0, 0usize);
    let mut step_seconds = Vec::new();
    let time = s.config.context;
    while progress.step < s.steps {
        let lr = cosine_rate(s.lr, s.warmup, s.min_lr, s.steps, progress.step);
        device.synchronize()?;
        let clock = Instant::now();
        let mut ids = Vec::with_capacity(s.batch * time);
        let mut targets = Vec::with_capacity(s.batch * time);
        for _ in 0..s.batch {
            let _stream_draw = progress.rng.uniform();
            let start = progress.rng.below(train.len() - time - 1);
            ids.extend_from_slice(&train[start..start + time]);
            targets.extend_from_slice(&train[start + 1..start + time + 1]);
        }
        // The TRAIN mask defines the count reference only; the registered training margin remains unmasked.
        let loss = model.loss(&ids, &targets, s.batch, time)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!(
                "nonfinite training loss at step {}",
                progress.step
            )));
        }
        let grads = loss.backward()?;
        let grad_norm = optimizer.update(&model, &grads, lr)?;
        device.synchronize()?;
        let seconds = clock.elapsed().as_secs_f64();
        step_seconds.push(seconds);
        progress.train_seconds += seconds;
        progress.step += 1;
        window_loss.0 += value;
        window_loss.1 += 1;
        if progress.step % s.eval_every == 0 || progress.step == s.steps {
            let dev = evaluate_native(&mut model, &reference, &valid, &valid_mask, &dev_plan)?;
            let tokens = progress.step * s.batch * time;
            let point = json!({"step":progress.step,"tokens":tokens,"lr":lr,"train_loss":window_loss.0/window_loss.1 as f64,"dev":dev,"grad_norm":grad_norm,"train_seconds":progress.train_seconds,"tokens_per_second":tokens as f64/progress.train_seconds});
            eprintln!("{point}");
            progress.curve.push(point);
            window_loss = (0.0, 0);
        }
        if s.checkpoint_every > 0
            && progress.step % s.checkpoint_every == 0
            && progress.step < s.steps
        {
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
        }
        if started.elapsed().as_secs_f64() > s.max_seconds && !extended {
            eprintln!("native healthy run exceeded max_seconds estimate at step {}; extending; renew lease and record extension",progress.step);
            extended = true;
        }
    }
    let final_eval = evaluate_native(&mut model, &reference, &valid, &valid_mask, &final_plan)?;
    model.save(&out.join("model"))?;
    save_native_reference_binding(&out.join("model"), &lineage)?;
    let untrained = progress
        .curve
        .iter()
        .find(|p| p["step"].as_u64() == Some(0))
        .map(|p| p["dev"].clone())
        .ok_or_else(|| invalid("checkpoint lacks same-seed zero-step constant"))?;
    let report = json!({"schema":"uor-r4.geometric-stack-run/1","settings":settings,"native_count_reference":binding,"native_objective":{"training":"mean token_margin + mean address_margin; coefficient1 each","dev_bars":"token-only margins/unviolated; TRAIN masked count reference; DEV oracle diagnostic only","timing_boundary":"synchronized CUDA boundaries around full step including window preparation, forward/backward/update"},"parameters":model.parameter_count(),"completed_steps":progress.step,"stopped_early":false,"estimate_extended":extended,"resumed_from":resumed_from,"target_visits":progress.step*s.batch*time,"train_seconds":progress.train_seconds,"tokens_per_second":(progress.step*s.batch*time) as f64/progress.train_seconds.max(1e-9),"step_seconds":step_seconds,"curve":progress.curve,"untrained_constant":untrained,"final":final_eval,"samples":Value::Null,"inputs":{"train":identity(&s.train)?,"train_mask":identity(&s.train_mask)?,"valid":identity(&s.valid)?,"valid_mask":identity(&s.valid_mask)?,"tokenizer":identity(&s.tokenizer)?},"executable":identity(&std::env::current_exe()?)?,"model_sha256":sha256_file(&out.join("model/model.safetensors"))?});
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
fn evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "model",
            "tokens",
            "out",
            "valid_mask",
            "train_reference",
            "train_mask",
            "expected_train_targets",
            "tokenizer",
            "expected_dev_targets",
            "max_blocks",
            "device",
            "precision",
            "data_parallel",
            "tf32",
        ],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokens_path = PathBuf::from(args.required("tokens")?);
    let mask_path = PathBuf::from(args.required("valid_mask")?);
    let train_path = PathBuf::from(args.required("train_reference")?);
    let train_mask_path = PathBuf::from(args.required("train_mask")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let saved: Value = serde_json::from_slice(&fs::read(model_dir.join("config.json"))?)?;
        if saved.get("native_vector_head").is_none() {
            return Err(invalid(
                "native evaluate refuses artifact without declared native head",
            ));
        }
        let config: StackConfig = serde_json::from_value(saved)?;
        require_native_training_config(
            &config,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            false,
        )?;
        config.validate()?;
        let train_content = native_input_content(&train_path)?;
        if train_content == native_input_content(&tokens_path)? {
            return Err(invalid("DEV cannot substitute for TRAIN count reference"));
        }
        let tokens = read_tokens(&tokens_path, config.vocab_size)?;
        let mask = fs::read(&mask_path)?;
        let train = read_tokens(&train_path, config.vocab_size)?;
        let train_mask = fs::read(&train_mask_path)?;
        let bos = native_bos_identity(&tokenizer_path)?;
        let reference = native_rung0::MaskedTargets::new(&train, &train_mask, bos)?;
        let binding = native_reference_binding(
            &train_content,
            &native_input_content(&train_mask_path)?,
            &native_input_content(&tokenizer_path)?,
            &reference,
            args.number("expected_train_targets", 2425925)?,
            bos,
        )?;
        let saved_binding: Value = serde_json::from_slice(
            &fs::read(model_dir.join(NATIVE_COUNT_REFERENCE_FILE))
                .map_err(|e| invalid(format!("missing native TRAIN binding: {e}")))?,
        )?;
        require_bound_native_reference(&saved_binding, &binding)?;
        let plan = native_rung0::PanelPlan::new(
            &tokens,
            &mask,
            config.context,
            args.number("max_blocks", tokens.len() / config.context)?,
            bos,
            args.number("expected_dev_targets", 14287)?,
        )?;
        let device = cuda_device(&args)?;
        let mut model = StackModel::load(&model_dir, &device)?;
        require_native_training_config(
            &model.config,
            model.native_vector_head_spec().is_some(),
            Some(model.quaternion_weights()),
            false,
        )?;
        let full = evaluate_native(&mut model, &reference, &tokens, &mask, &plan)?;
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(
                &json!({"schema":"uor-r4.native-rung0-evaluation/2","model":identity(&model_dir.join("model.safetensors"))?,"config":model.config,"tokens":identity(&tokens_path)?,"valid_mask":identity(&mask_path)?,"tokenizer":identity(&tokenizer_path)?,"train_reference":identity(&train_path)?,"train_mask":identity(&train_mask_path)?,"native_count_reference":binding,"full":full}),
            )?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}
fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(e) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":e.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}
fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|a| matches!(a.as_str(), "--help" | "-h" | "help"))
    {
        println!("geometric-stack train|evaluate key=value ...\nNative CUDA rung0 only; train requires train,train_mask,valid,valid_mask,tokenizer; evaluate requires separately bound train_reference,train_mask. Historical modes are excluded.");
        return Ok(());
    }
    let (mode, rest) = arguments
        .split_first()
        .ok_or_else(|| invalid("usage: geometric-stack train|evaluate key=value ..."))?;
    match mode.as_str() {
        "train" => {
            let args = Args::parse(rest, TRAIN_ARGS)?;
            let out = PathBuf::from(args.required("out")?);
            let settings = Settings::new(args)?;
            report_output::claim(&out)?;
            finish(&out, train(&settings, &out))
        }
        "evaluate" => evaluate_mode(rest),
        _ => Err(invalid("mode excluded from native rung0 build")),
    }
}

#[cfg(test)]
mod native_rung0_argument_tests {
    use super::*;
    fn native_config() -> Result<StackConfig> {
        let mut config = StackConfig::transformer(256, 4, 6, 768, 384, 1)?;
        config.arch = StackArch::Geometric;
        config.pattern = "rrrrrr".into();
        config.mlp_hidden = 384;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        config.objective = Objective::MarginHinge { rivals: 8 };
        config.validate()?;
        Ok(config)
    }
    #[test]
    fn complete_native_argument_contract_refuses_legacy_modes() -> Result<()> {
        let config = native_config()?;
        require_loaded_runtime_metadata(&config, &config, 0.1, 1.0, 0.1, 1.0)?;
        let mut swapped = config.clone();
        swapped.seed += 1;
        assert!(require_loaded_runtime_metadata(&swapped, &config, 0.1, 1.0, 0.1, 1.0).is_err());
        assert!(require_loaded_runtime_metadata(&config, &config, 0.2, 1.0, 0.1, 1.0).is_err());
        assert!(require_loaded_runtime_metadata(&config, &config, 0.1, 2.0, 0.1, 1.0).is_err());
        require_native_training_config(
            &config,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            false,
        )?;
        assert!(require_native_training_config(
            &config,
            false,
            Some(QuaternionWeights::SpiralcoreFbs),
            false
        )
        .is_err());
        let mut historical = config.clone();
        historical.rec_gates = RecGates::Historical;
        assert!(require_native_training_config(
            &historical,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            false
        )
        .is_err());
        let mut ce = config.clone();
        ce.objective = Objective::Ce;
        assert!(require_native_training_config(
            &ce,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            false
        )
        .is_err());
        let mut read = config.clone();
        read.pattern = "rrarra".into();
        assert!(require_native_training_config(
            &read,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            false
        )
        .is_err());
        // Read layers are admitted only through the declared native read,
        // and the native read only with a read layer.
        require_native_training_config(&read, true, Some(QuaternionWeights::SpiralcoreFbs), true)?;
        assert!(require_native_training_config(
            &config,
            true,
            Some(QuaternionWeights::SpiralcoreFbs),
            true
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn native_process_flags_refuse_sampling_host_diagnostics_and_bad_estimates() -> Result<()> {
        native_process_options(0, f64::INFINITY, false)?;
        assert!(native_process_options(128, f64::INFINITY, false).is_err());
        assert!(native_process_options(0, f64::INFINITY, true).is_err());
        native_process_options(0, 60., false)?;
        assert!(native_process_options(0, f64::NAN, false).is_err());
        Ok(())
    }
    #[test]
    fn native_train_reference_arguments_require_one_masked_stream_and_positive_count() -> Result<()>
    {
        native_train_reference_options(1, Some(Path::new("train-mask.u8")), 2425925)?;
        assert!(native_train_reference_options(1, None, 2425925).is_err());
        assert!(
            native_train_reference_options(2, Some(Path::new("train-mask.u8")), 2425925).is_err()
        );
        assert!(native_train_reference_options(1, Some(Path::new("train-mask.u8")), 0).is_err());
        let missing = Args::parse(
            &["valid_mask=dev-mask.u8".into()],
            &["valid_mask", "train_reference", "train_mask"],
        )?;
        assert!(missing.required("train_reference").is_err());
        assert!(missing.required("train_mask").is_err());
        Ok(())
    }

    #[test]
    fn native_reference_metadata_rejects_invalid_masks_and_count_claims() -> Result<()> {
        let tokens = [0, 9, 1, 8, 2];
        let mask = [0, 0, 1, 0, 1];
        let reference = native_rung0::MaskedTargets::new(&tokens, &mask, 0)?;
        assert_eq!(reference.len(), 2);
        assert_eq!(reference.payload_tokens(), 5);
        assert!(native_rung0::MaskedTargets::new(&tokens, &mask[..4], 0).is_err());
        assert!(native_rung0::MaskedTargets::new(&tokens, &[0, 0, 2, 0, 1], 0).is_err());
        assert!(native_rung0::MaskedTargets::new(&tokens, &[0; 5], 0).is_err());
        let content = json!({"bytes": 10, "sha256": "train-token-pin"});
        let mask_content = json!({"bytes": 5, "sha256": "train-mask-pin"});
        let tokenizer = json!({"bytes": 30, "sha256": "tokenizer-pin"});
        assert!(
            native_reference_binding(&content, &mask_content, &tokenizer, &reference, 3, 0)
                .is_err()
        );
        let binding =
            native_reference_binding(&content, &mask_content, &tokenizer, &reference, 2, 0)?;
        require_bound_native_reference(&binding, &binding)?;
        assert_eq!(binding["source_split"], "train");
        assert_eq!(binding["reference_targets"], 2);
        for (field, changed) in [
            ("source_split", json!("dev")),
            ("frequency_policy", json!("dev-all-tokens")),
            ("train_mask", json!({"bytes": 5, "sha256": "changed-mask"})),
            ("train", json!({"bytes": 10, "sha256": "changed-train"})),
            ("expected_train_targets", json!(3)),
        ] {
            let mut stale = binding.clone();
            stale[field] = changed;
            assert!(require_bound_native_reference(&stale, &binding).is_err());
        }
        assert!(require_bound_native_reference(&Value::Null, &binding).is_err());
        Ok(())
    }

    #[test]
    fn native_metric_record_never_mislabels_raw_margins_as_likelihood() {
        let record=Evaluation::Native(json!({"token_margin":1.0,"address_margin":1.0,"total_margin":2.0,"top1":0.0,"recall_at_8":1.0})).record();
        assert_eq!(record["total_margin"], 2.0);
        assert!(record.get("nll").is_none());
        assert!(record.get("bits_per_byte").is_none());
    }
}
