//! Reply-level memory evaluation (Lab 1; #973 comment 5884046794): greedy
//! replies to a request panel from saved AERM-format checkpoints, with the
//! prime-route store in the loop (`Store`) and without it (`Silent`), scored
//! on the panel's ten memory requests against the frozen answer key
//! (`uor_r4_training::stack_memory_replies`).
//!
//! ```text
//! memory-replies out=NEW_REPORT_ROOT checkpoints=DIR[,DIR...] tokenizer=TOKENIZER.json \
//!   requests=REQUESTS.json [max_new_tokens=32]
//! ```
//!
//! The report root is claimed before anything is loaded and sealed at the
//! end. Evaluation only. Set RAYON_NUM_THREADS to bound the threads.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::stack_aerm::AermModel;
use uor_r4_training::stack_dialogue::load_requests;
use uor_r4_training::stack_memory_replies::{memory_reply_panel, score_memory, MemoryArm};
use uor_r4_training::stack_prime_route::PrimeRegistry;
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

fn run(args: &Args, out: &Path) -> Result<()> {
    let checkpoints: Vec<PathBuf> = args
        .required("checkpoints")?
        .split(',')
        .map(|item| PathBuf::from(item.trim()))
        .collect();
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let requests_path = PathBuf::from(args.required("requests")?);
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let started = Instant::now();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let requests = load_requests(&requests_path)?;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let device = Device::Cpu;
    let mut results = Vec::new();
    for checkpoint in &checkpoints {
        let identity = checkpoint_identity(checkpoint)?;
        let model = AermModel::load(checkpoint, &device)?;
        if !model.has_memory() {
            return Err(invalid(
                "memory-replies needs a checkpoint with a memory branch",
            ));
        }
        let registry = PrimeRegistry::new(model.stack.config.vocab_size)?;
        let mut arms = serde_json::Map::new();
        for arm in [MemoryArm::Store, MemoryArm::Silent] {
            let clock = Instant::now();
            let panel = memory_reply_panel(
                &model,
                &encoder,
                &protocol,
                &requests,
                max_new_tokens,
                &decode,
                &registry,
                arm,
            )?;
            let score = score_memory(&panel)?;
            println!(
                "{} {arm:?}: memory {}/{}",
                checkpoint.display(),
                score["correct"],
                score["of"]
            );
            arms.insert(
                format!("{arm:?}"),
                json!({"memory": score, "panel": panel, "seconds": clock.elapsed().as_secs_f64()}),
            );
        }
        results.push(json!({
            "checkpoint": checkpoint.display().to_string(),
            "files_sha256": identity,
            "arms": arms,
        }));
    }
    let report = json!({
        "schema": "uor-r4.memory-replies/1",
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "requests_sha256": sha256_file(&requests_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "max_new_tokens": max_new_tokens,
        "answer_key": "stack_memory_replies::PANEL_MEMORY_ANSWERS (#973 comment 5884046794)",
        "checkpoints": results,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("memory_replies.json"),
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
            "tokenizer",
            "requests",
            "max_new_tokens",
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
