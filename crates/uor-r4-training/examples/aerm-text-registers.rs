//! Text NLL of saved AERM probe checkpoints under two register policies
//! (Lab 1; the arm C re-read on #973).
//!
//! `train_aerm` feeds ordinary text with silent registers (every position
//! None), while `text_nll` drives the registers from the model's own tags and
//! triggers. This tool scores both policies on the same windows `text_nll`
//! uses:
//! - `model`: registers from the model's argmax tags and triggers, as
//!   `text_nll` computes them;
//! - `silent`: every register None, as in training. It is also what a gate
//!   that allows memory operations only inside user turns gives on text that
//!   has no user turns.
//!
//! ```text
//! aerm-text-registers out=NEW_REPORT_ROOT checkpoints=DIR[,DIR...] dev=DEV.u16 \
//!   tokenizer=TOKENIZER.json [dev_tokens=249000] [context=256] [dev_windows=512] [batch=16]
//! ```
//!
//! The report root is claimed before anything is loaded and sealed at the
//! end. Evaluation only. Set RAYON_NUM_THREADS to bound the threads.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, D};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::logits_cross_entropy;
use uor_r4_training::stack_aerm::{
    model_registers, AermModel, STATUS_HIT, STATUS_NONE, TRIGGER_NONE, TRIGGER_WRITE,
};
use uor_r4_training::{sha256_file, Result, TrainingError};

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

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

fn read_u16_range(path: &Path, start: u64, count: usize) -> Result<Vec<u16>> {
    let mut file = fs::File::open(path)?;
    file.seek(SeekFrom::Start(start * 2))?;
    let mut bytes = vec![0u8; count * 2];
    file.read_exact(&mut bytes)?;
    Ok(bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}

/// SHA-256 of every file a checkpoint loads.
fn checkpoint_identity(directory: &Path) -> Result<Value> {
    let mut files = serde_json::Map::new();
    for name in [
        "aerm.json",
        "heads.safetensors",
        "stack/config.json",
        "stack/model.safetensors",
    ] {
        files.insert(name.into(), json!(sha256_file(&directory.join(name))?));
    }
    Ok(Value::Object(files))
}

/// Per-token counts over the scored windows.
#[derive(Default)]
struct Tally {
    tokens: usize,
    model_loss: f64,
    silent_loss: f64,
    triggers: usize,
    writes: usize,
    read_events: usize,
    active: usize,
    hit: usize,
}

impl Tally {
    fn per_thousand(&self, count: usize) -> f64 {
        count as f64 * 1000.0 / self.tokens.max(1) as f64
    }
}

/// `text_nll`'s windows, scored under the model and silent policies.
fn score(
    model: &AermModel,
    dev: &[u16],
    context: usize,
    windows: usize,
    batch: usize,
    eos: u32,
) -> Result<Tally> {
    let span = context + 1;
    if dev.len() < span || windows == 0 || batch == 0 {
        return Err(invalid("development text needs at least one window"));
    }
    let stride = ((dev.len() - span) / windows).max(1);
    let starts: Vec<usize> = (0..windows)
        .map(|w| w * stride)
        .filter(|s| s + span <= dev.len())
        .collect();
    let mut tally = Tally::default();
    for chunk in starts.chunks(batch) {
        let rows = chunk.len() * context;
        let mut ids = Vec::with_capacity(rows);
        let mut targets = Vec::with_capacity(rows);
        for &start in chunk {
            ids.extend(dev[start..start + context].iter().map(|&t| u32::from(t)));
            targets.extend(dev[start + 1..start + span].iter().map(|&t| u32::from(t)));
        }
        let bottom = model.bottom(&ids, chunk.len(), context)?;
        let tags = bottom.tags.detach().argmax(D::Minus1)?.to_vec1::<u32>()?;
        let triggers = bottom
            .triggers
            .detach()
            .argmax(D::Minus1)?
            .to_vec1::<u32>()?;
        tally.triggers += triggers.iter().filter(|&&t| t != TRIGGER_NONE).count();
        tally.writes += triggers.iter().filter(|&&t| t == TRIGGER_WRITE).count();
        let registers = model_registers(&ids, &tags, &triggers, chunk.len(), context, eos)?;
        tally.read_events += registers.read_events;
        tally.active += registers
            .status
            .iter()
            .zip(&registers.status_previous)
            .filter(|(&s, &p)| s != STATUS_NONE || p != STATUS_NONE)
            .count();
        tally.hit += registers
            .status
            .iter()
            .zip(&registers.status_previous)
            .filter(|(&s, &p)| s == STATUS_HIT || p == STATUS_HIT)
            .count();
        let modelled = model
            .top(
                &bottom.hidden,
                &registers.status,
                &registers.value,
                &registers.status_previous,
                &registers.value_previous,
            )?
            .detach();
        let silent_status = vec![STATUS_NONE; rows];
        let silent_value = vec![0u32; rows];
        let silent = model
            .top(
                &bottom.hidden,
                &silent_status,
                &silent_value,
                &silent_status,
                &silent_value,
            )?
            .detach();
        let model_loss = logits_cross_entropy(&modelled, &targets, None)?.to_scalar::<f32>()?;
        let silent_loss = logits_cross_entropy(&silent, &targets, None)?.to_scalar::<f32>()?;
        tally.model_loss += f64::from(model_loss) * rows as f64;
        tally.silent_loss += f64::from(silent_loss) * rows as f64;
        tally.tokens += rows;
    }
    Ok(tally)
}

fn run(args: &Args, out: &Path) -> Result<()> {
    let checkpoints: Vec<PathBuf> = args
        .required("checkpoints")?
        .split(',')
        .map(|item| PathBuf::from(item.trim()))
        .collect();
    let dev_path = PathBuf::from(args.required("dev")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let dev_tokens: usize = args.number("dev_tokens", 249_000)?;
    let context: usize = args.number("context", 256)?;
    let windows: usize = args.number("dev_windows", 512)?;
    let batch: usize = args.number("batch", 16)?;
    let started = Instant::now();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let dev = read_u16_range(&dev_path, 0, dev_tokens)?;
    let device = Device::Cpu;
    let mut results = Vec::new();
    for checkpoint in &checkpoints {
        let identity = checkpoint_identity(checkpoint)?;
        let model = AermModel::load(checkpoint, &device)?;
        let tally = score(&model, &dev, context, windows, batch, protocol.eos_id)?;
        let tokens = tally.tokens.max(1) as f64;
        let model_nll = tally.model_loss / tokens;
        let silent_nll = tally.silent_loss / tokens;
        println!(
            "{}: model {model_nll:.6}  silent {silent_nll:.6}  triggers/1k {:.1}  active/1k {:.1}  hit/1k {:.1}",
            checkpoint.display(),
            tally.per_thousand(tally.triggers),
            tally.per_thousand(tally.active),
            tally.per_thousand(tally.hit),
        );
        results.push(json!({
            "checkpoint": checkpoint.display().to_string(),
            "files_sha256": identity,
            "tokens": tally.tokens,
            "model_registers_nll": model_nll,
            "silent_registers_nll": silent_nll,
            "model_minus_silent": model_nll - silent_nll,
            "per_thousand_tokens": {
                "non_none_triggers": tally.per_thousand(tally.triggers),
                "write_triggers": tally.per_thousand(tally.writes),
                "read_events": tally.per_thousand(tally.read_events),
                "register_active_positions": tally.per_thousand(tally.active),
                "register_hit_positions": tally.per_thousand(tally.hit),
            },
        }));
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.aerm-text-registers/1",
        "executable_sha256": sha256_file(&executable)?,
        "policies": {
            "model": "registers from the model's argmax tags and triggers (text_nll)",
            "silent": "every register None (train_aerm's text batches)",
        },
        "read_events_note": "each address-driven read records two events, current and previous",
        "dev": dev_path.display().to_string(),
        "dev_sha256": sha256_file(&dev_path)?,
        "dev_tokens": dev_tokens,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "eos": protocol.eos_id,
        "context": context,
        "dev_windows": windows,
        "batch": batch,
        "checkpoints": results,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("text_registers.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(
        &arguments,
        &[
            "out",
            "checkpoints",
            "dev",
            "tokenizer",
            "dev_tokens",
            "context",
            "dev_windows",
            "batch",
        ],
    )?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = run(&args, &out);
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
