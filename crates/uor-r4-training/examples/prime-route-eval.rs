//! Prime-route addressing arms on saved AERM probe checkpoints (Lab 1;
//! the memory port's address, ROADMAP §2b; the prime router of ADR-0003).
//!
//! Every arm runs the same model on the same episodes and changes only where
//! the store's registers come from (`uor_r4_training::stack_prime_route`):
//! - `GV1`: the probe's typed `(entity, relation)` keys and learned write
//!   trigger;
//! - `ExpertTrigger`: semiprime-expert keys, the learned write trigger;
//! - `ExpertTurnEnd`: semiprime-expert keys, a write at each user turn's end;
//! - `ExpertSieve`: semiprime-expert keys, the learned write trigger, and
//!   sieve clause formation (for reads and writes) over the session's
//!   registered atoms;
//! - `Gold`: gold registers, a perfect parser.
//!
//! ```text
//! prime-route-eval out=NEW_REPORT_ROOT checkpoints=DIR[,DIR...] tokenizer=TOKENIZER.json \
//!   [context=256] [eval_episodes=512] [batch=16]
//! ```
//!
//! The report root is claimed before anything is loaded and sealed at the
//! end. Each checkpoint is scored on the probe's in-distribution dialogues
//! (seed 9001) and held-out-template dialogues (seed 9002), the seeds and
//! batching `aerm-probe` uses, so the `GV1` arm reproduces the probe's
//! per-class counts. Evaluation only. Set RAYON_NUM_THREADS to bound the
//! threads.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::stack_aerm::{AermModel, QueryClass, RelationWorld};
use uor_r4_training::stack_prime_route::{
    arm_accuracy, evaluate_prime_route, PrimeRegistry, RegisterArm,
};
use uor_r4_training::{sha256_file, Result, TrainingError};

/// The probe's evaluation seeds (`aerm-probe`).
const DIALOGUE_SEED: u64 = 9_001;
const HELD_SEED: u64 = 9_002;

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

fn summary(label: &str, evaluation: &uor_r4_training::stack_prime_route::PrimeRouteEvaluation) {
    let classes = [
        QueryClass::First,
        QueryClass::Updated,
        QueryClass::Reasserted,
        QueryClass::Previous,
        QueryClass::PreviousAbsent,
        QueryClass::Absent,
    ];
    println!("{label} (tag accuracy {:.4})", evaluation.tag_accuracy);
    for arm in RegisterArm::ALL {
        let cells: Vec<String> = classes
            .iter()
            .map(|&class| match arm_accuracy(evaluation, arm, class) {
                Some(accuracy) => format!("{class:?} {accuracy:.3}"),
                None => format!("{class:?} -"),
            })
            .collect();
        println!("  {:<14} {}", format!("{arm:?}"), cells.join("  "));
    }
    for (label, causes) in [
        ("ExpertSieve failures", &evaluation.sieve_failure_causes),
        (
            "GV1-right, ExpertSieve-wrong",
            &evaluation.sieve_regression_causes,
        ),
    ] {
        for (class, counts) in causes {
            let cells: Vec<String> = counts.iter().map(|(c, n)| format!("{c} {n}")).collect();
            println!("  {label} [{class}]: {}", cells.join(", "));
        }
    }
}

fn run(args: &Args, out: &Path) -> Result<()> {
    let checkpoints: Vec<PathBuf> = args
        .required("checkpoints")?
        .split(',')
        .map(|item| PathBuf::from(item.trim()))
        .collect();
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let context: usize = args.number("context", 256)?;
    let episodes: usize = args.number("eval_episodes", 512)?;
    let batch: usize = args.number("batch", 16)?;
    let started = Instant::now();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::new(&encode, protocol.bos_id, protocol.eos_id)?;
    let device = Device::Cpu;
    let mut results = Vec::new();
    for checkpoint in &checkpoints {
        let identity = checkpoint_identity(checkpoint)?;
        let model = AermModel::load(checkpoint, &device)?;
        let registry = PrimeRegistry::new(model.stack.config.vocab_size)?;
        let largest_prime = registry.prime((registry.len() - 1) as u32)?;
        let dialogues = evaluate_prime_route(
            &model,
            &world,
            &registry,
            episodes,
            DIALOGUE_SEED,
            false,
            context,
            batch,
        )?;
        let held = evaluate_prime_route(
            &model, &world, &registry, episodes, HELD_SEED, true, context, batch,
        )?;
        summary(
            &format!("{} in-distribution", checkpoint.display()),
            &dialogues,
        );
        summary(
            &format!("{} held-out templates", checkpoint.display()),
            &held,
        );
        results.push(json!({
            "checkpoint": checkpoint.display().to_string(),
            "files_sha256": identity,
            "registry": {"atoms": registry.len(), "largest_prime": largest_prime},
            "dialogues": dialogues,
            "held_out": held,
        }));
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.prime-route-eval/1",
        "executable_sha256": sha256_file(&executable)?,
        "arms": RegisterArm::ALL.iter().map(|arm| format!("{arm:?}")).collect::<Vec<_>>(),
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "context": context,
        "eval_episodes": episodes,
        "batch": batch,
        "seeds": {"dialogues": DIALOGUE_SEED, "held_out": HELD_SEED},
        "checkpoints": results,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("prime_route.json"),
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
            "context",
            "eval_episodes",
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
