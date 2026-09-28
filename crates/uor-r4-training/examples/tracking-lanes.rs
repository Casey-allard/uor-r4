//! Roadmap T1(b), the B1 hypothesis (`uor_r4_training::stack_tracking`).
//!
//! Stage A (`a5`): train token-conditioned tracking lanes on the A5 word
//! problem, then close each trained lane into a finite group and serve it as an
//! exact table automaton.
//!
//! Stage B (`mixed`): train the geometric stack on text and A5 words together,
//! with and without a tracking-lane side channel, and report development text
//! NLL, the stack's A5 accuracy inside the context and the snapped lanes.
//!
//! ```text
//! tracking-lanes a5 out=NEW_REPORT_ROOT [kinds=quaternion,phase,reflection_pair,frozen] \
//!   [seeds=1,2,3] [lanes=8] [hidden=128] [steps=3000] [batch=64] [lr=0.005] \
//!   [train_length=32] [eval_lengths=32,64,128,256,512,1024,2048,4096] [eval_words=256] \
//!   [fit_words=512]
//! tracking-lanes mixed out=NEW_REPORT_ROOT text=TOKENS.u16 [train_tokens=16777216] \
//!   [dev=DEV.u16] [dev_tokens=1048576] [arms=none,quaternion,reflection_pair,phase] [seeds=1,2,3] [lanes=8] \
//!   [width=128] [heads=4] [pattern=rrar] [mlp=384] [context=128] [steps=1500] [batch=16] \
//!   [lr=0.003] [warmup=100] [weight_decay=0.1] [a5_weight=1] [a5_start=8] [lane_lr=0.03] [dev_windows=256] \
//!   [a5_words=256] [snap_lengths=128,512,4096]
//! ```
//!
//! Every arm with one seed sees the same training data, and every arm is
//! evaluated on the same fresh words and development windows. `mixed` trains on
//! the first `train_tokens` tokens of TOKENS.u16 (little-endian u16 ids below
//! 4096) and evaluates on the first `dev_tokens` of DEV.u16 when `dev=` is given,
//! otherwise on the last `dev_tokens` of TOKENS.u16. The report root is claimed
//! before any model is built and sealed at the end. Set RAYON_NUM_THREADS to
//! bound the threads.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{ReadScore, StackArch, StackConfig};
use uor_r4_training::stack_tracking::{
    a5_stack_accuracy, evaluate, text_nll, train, train_mixed, A5Task, LaneAutomaton, LaneConfig,
    LaneKind, LaneModel, MixedConfig, TrackedStack, TrainConfig, A5_ORDER, TEXT_VOCAB,
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

    fn list<T: std::str::FromStr>(&self, key: &str, default: &str) -> Result<Vec<T>> {
        self.0
            .get(key)
            .map(String::as_str)
            .unwrap_or(default)
            .split(',')
            .map(|item| {
                item.trim()
                    .parse()
                    .map_err(|_| invalid(format!("invalid {key} item {item}")))
            })
            .collect()
    }
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn a5(args: &Args, out: &Path) -> Result<()> {
    let kinds: Vec<LaneKind> = args
        .0
        .get("kinds")
        .map(String::as_str)
        .unwrap_or("quaternion,phase,reflection_pair,frozen")
        .split(',')
        .map(|name| LaneKind::parse(name.trim()))
        .collect::<Result<_>>()?;
    let seeds: Vec<u64> = args.list("seeds", "1,2,3")?;
    let lanes: usize = args.number("lanes", 8)?;
    let hidden: usize = args.number("hidden", 128)?;
    let eval_lengths: Vec<usize> = args.list("eval_lengths", "32,64,128,256,512,1024,2048,4096")?;
    let eval_words: usize = args.number("eval_words", 256)?;
    let fit_words: usize = args.number("fit_words", 512)?;
    let train_config = TrainConfig {
        steps: args.number("steps", 3000)?,
        batch: args.number("batch", 64)?,
        learning_rate: args.number("lr", 0.005)?,
        train_length: args.number("train_length", 32)?,
        data_seed: 0,
    };
    let task = A5Task::standard()?;
    let device = Device::Cpu;
    fs::create_dir(out.join("runs"))?;
    let mut summary = Vec::new();
    println!(
        "{:<16} {:>4} {:>8} {:>9} {:>9} {:>9} {:>6} {:>9} {:>9}",
        "kind", "seed", "train_s", "f@32", "f@512", "f@4096", "order", "snap@512", "snap@4096"
    );
    for &kind in &kinds {
        for &seed in &seeds {
            let config = LaneConfig {
                kind,
                vocab: task.vocab(),
                lanes,
                hidden,
                classes: A5_ORDER,
                seed,
            };
            let model = LaneModel::new(config.clone(), &device)?;
            let run_train = TrainConfig {
                data_seed: 1_000 + seed,
                ..train_config.clone()
            };
            let (records, train_seconds) = train(&model, &task, &run_train)?;
            let started = Instant::now();
            let float = evaluate(&model, &task, &eval_lengths, eval_words, 77)?;
            let float_seconds = started.elapsed().as_secs_f64();

            let mut lanes_report = Vec::new();
            let mut best: Option<LaneAutomaton> = None;
            if kind != LaneKind::Frozen {
                for lane in 0..lanes {
                    match LaneAutomaton::snap(&model, &task, lane, fit_words, 32, 55)? {
                        None => lanes_report.push(json!({"lane": lane, "snapped": false})),
                        Some(automaton) => {
                            lanes_report.push(json!({
                                "lane": lane,
                                "snapped": true,
                                "order": automaton.order,
                                "fit_accuracy": automaton.fit_accuracy,
                                "minimal_order": automaton.minimal_order,
                                "max_merge_distance": automaton.max_merge_distance,
                                "max_trace_deviation": automaton.max_trace_deviation,
                            }));
                            let better = best
                                .as_ref()
                                .is_none_or(|b| automaton.fit_accuracy > b.fit_accuracy);
                            if better {
                                best = Some(automaton);
                            }
                        }
                    }
                }
            }
            let served = best
                .as_ref()
                .map(|automaton| automaton.evaluate(&task, &eval_lengths, eval_words, 77));
            let at = |rows: &[uor_r4_training::stack_tracking::LengthAccuracy], length: usize| {
                rows.iter()
                    .find(|row| row.length == length)
                    .map(|row| row.final_position)
            };
            let fmt =
                |value: Option<f64>| value.map_or_else(|| "-".to_owned(), |v| format!("{v:.3}"));
            println!(
                "{:<16} {:>4} {:>8.1} {:>9} {:>9} {:>9} {:>6} {:>9} {:>9}",
                format!("{kind:?}"),
                seed,
                train_seconds,
                fmt(at(&float, 32)),
                fmt(at(&float, 512)),
                fmt(at(&float, 4096)),
                best.as_ref().map_or_else(
                    || "-".to_owned(),
                    |b| format!("{}/{}", b.order, b.minimal_order)
                ),
                fmt(served.as_deref().and_then(|rows| at(rows, 512))),
                fmt(served.as_deref().and_then(|rows| at(rows, 4096))),
            );
            let run = json!({
                "kind": kind,
                "seed": seed,
                "config": config,
                "train": run_train,
                "train_seconds": train_seconds,
                "train_records": records,
                "float_eval": float,
                "float_eval_seconds": float_seconds,
                "lanes": lanes_report,
                "served": best.as_ref().map(|b| json!({
                    "lane": b.lane,
                    "order": b.order,
                    "fit_accuracy": b.fit_accuracy,
                    "minimal_order": b.minimal_order,
                    "max_merge_distance": b.max_merge_distance,
                    "max_trace_deviation": b.max_trace_deviation,
                    "eval": served,
                })),
            });
            fs::write(
                out.join("runs")
                    .join(format!("{kind:?}-seed{seed}.json").to_lowercase()),
                serde_json::to_vec_pretty(&run)?,
            )?;
            summary.push(run_summary(&run));
        }
    }
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&json!({
            "task": {
                "generators": task.generators,
                "classes": A5_ORDER,
                "chance": 1.0 / A5_ORDER as f64,
            },
            "eval_lengths": eval_lengths,
            "eval_words": eval_words,
            "runs": summary,
        }))?,
    )?;
    Ok(())
}

fn run_summary(run: &Value) -> Value {
    json!({
        "kind": run["kind"],
        "seed": run["seed"],
        "train_seconds": run["train_seconds"],
        "float_eval": run["float_eval"],
        "served": run["served"],
    })
}

/// `count` little-endian u16 ids starting at token `start` of `path`.
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

fn mixed(args: &Args, out: &Path) -> Result<()> {
    let text_path = PathBuf::from(args.required("text")?);
    let train_tokens: usize = args.number("train_tokens", 16_777_216)?;
    let dev_tokens: usize = args.number("dev_tokens", 1_048_576)?;
    let file_tokens = fs::metadata(&text_path)?.len() / 2;
    let dev_path = args.0.get("dev").map(PathBuf::from);
    let dev_file_tokens = match &dev_path {
        Some(path) => fs::metadata(path)?.len() / 2,
        None => file_tokens,
    };
    let separate_dev = dev_path.is_some();
    let dev_start = if separate_dev {
        0
    } else {
        file_tokens.saturating_sub(dev_tokens as u64)
    };
    if dev_tokens as u64 > dev_file_tokens
        || (!separate_dev && (train_tokens + dev_tokens) as u64 > file_tokens)
        || (separate_dev && train_tokens as u64 > file_tokens)
    {
        return Err(invalid("train_tokens + dev_tokens exceed the token file"));
    }
    let arms: Vec<Option<LaneKind>> = args
        .0
        .get("arms")
        .map(String::as_str)
        .unwrap_or("none,quaternion,reflection_pair,phase")
        .split(',')
        .map(|name| match name.trim() {
            "none" => Ok(None),
            other => LaneKind::parse(other).map(Some),
        })
        .collect::<Result<_>>()?;
    let seeds: Vec<u64> = args.list("seeds", "1,2,3")?;
    let lanes: usize = args.number("lanes", 8)?;
    let context: usize = args.number("context", 128)?;
    let snap_lengths: Vec<usize> = args.list("snap_lengths", "128,512,4096")?;
    let dev_windows: usize = args.number("dev_windows", 256)?;
    let a5_words: usize = args.number("a5_words", 256)?;
    let task = A5Task::standard()?;
    let stack_template = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: TEXT_VOCAB + task.vocab(),
        width: args.number("width", 128)?,
        heads: args.number("heads", 4)?,
        mlp_hidden: args.number("mlp", 384)?,
        context,
        pattern: args
            .0
            .get("pattern")
            .cloned()
            .unwrap_or_else(|| "rrar".into()),
        read: ReadScore::Dot,
        rotation: true,
        seed: 0,
    };
    stack_template.validate()?;
    let mixed_template = MixedConfig {
        steps: args.number("steps", 1500)?,
        batch: args.number("batch", 16)?,
        context,
        learning_rate: args.number("lr", 0.003)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        a5_weight: args.number("a5_weight", 1.0)?,
        a5_start_length: args.number("a5_start", 8)?,
        lane_learning_rate: args.number("lane_lr", 0.03)?,
        data_seed: 0,
    };

    let started = Instant::now();
    let text_sha256 = sha256_file(&text_path)?;
    let train = read_u16_range(&text_path, 0, train_tokens)?;
    let dev_sha256 = match &dev_path {
        Some(path) => sha256_file(path)?,
        None => text_sha256.clone(),
    };
    let dev = read_u16_range(
        dev_path.as_deref().unwrap_or(&text_path),
        dev_start,
        dev_tokens,
    )?;
    if train.iter().chain(&dev).any(|&t| t as usize >= TEXT_VOCAB) {
        return Err(invalid("token file has ids at or above 4096"));
    }
    let data_seconds = started.elapsed().as_secs_f64();
    let device = Device::Cpu;
    fs::create_dir(out.join("runs"))?;
    println!(
        "{:<16} {:>4} {:>8} {:>9} {:>8} {:>8} {:>8} {:>6} {:>9} {:>9}",
        "arm",
        "seed",
        "train_s",
        "text_nll",
        "a5@8",
        "a5@64",
        "a5@last",
        "order",
        "snap@512",
        "snap@4096"
    );
    let mut summary = Vec::new();
    for &arm in &arms {
        for &seed in &seeds {
            let config = StackConfig {
                seed,
                ..stack_template.clone()
            };
            let model = TrackedStack::new(
                config.clone(),
                arm.map(|kind| (kind, lanes)),
                &task,
                seed,
                &device,
            )?;
            let run_config = MixedConfig {
                data_seed: 2_000 + seed,
                ..mixed_template.clone()
            };
            let (records, train_seconds) = train_mixed(&model, &task, &train, &run_config)?;
            let started = Instant::now();
            let nll = text_nll(&model, &dev, context, dev_windows, 16)?;
            let a5 = a5_stack_accuracy(&model, &task, context, a5_words, 16, 99)?;
            let eval_seconds = started.elapsed().as_secs_f64();
            let mut lanes_report = Vec::new();
            let mut best: Option<LaneAutomaton> = None;
            if arm.is_some_and(|kind| kind != LaneKind::Frozen) {
                for lane in 0..lanes {
                    let generators = model.lane_generators(&task, lane)?;
                    match LaneAutomaton::from_generators(&generators, lane, &task, 512, 32, 55)? {
                        None => lanes_report.push(json!({"lane": lane, "snapped": false})),
                        Some(automaton) => {
                            lanes_report.push(json!({
                                "lane": lane,
                                "snapped": true,
                                "order": automaton.order,
                                "fit_accuracy": automaton.fit_accuracy,
                                "minimal_order": automaton.minimal_order,
                                "max_merge_distance": automaton.max_merge_distance,
                                "max_trace_deviation": automaton.max_trace_deviation,
                            }));
                            if best
                                .as_ref()
                                .is_none_or(|b| automaton.fit_accuracy > b.fit_accuracy)
                            {
                                best = Some(automaton);
                            }
                        }
                    }
                }
            }
            let served = best
                .as_ref()
                .map(|automaton| automaton.evaluate(&task, &snap_lengths, a5_words, 77));
            let at_position = |p: usize| {
                a5.by_position
                    .iter()
                    .find(|(q, _)| *q == p)
                    .map_or_else(|| "-".to_owned(), |(_, v)| format!("{v:.3}"))
            };
            let snap_at = |length: usize| {
                served
                    .as_ref()
                    .and_then(|rows| rows.iter().find(|r| r.length == length))
                    .map_or_else(|| "-".to_owned(), |r| format!("{:.3}", r.final_position))
            };
            let arm_name = arm.map_or_else(|| "none".to_owned(), |k| format!("{k:?}"));
            println!(
                "{:<16} {:>4} {:>8.1} {:>9.4} {:>8} {:>8} {:>8} {:>6} {:>9} {:>9}",
                arm_name,
                seed,
                train_seconds,
                nll,
                at_position(8),
                at_position(64),
                at_position(context),
                best.as_ref().map_or_else(
                    || "-".to_owned(),
                    |b| format!("{}/{}", b.order, b.minimal_order)
                ),
                snap_at(512),
                snap_at(4096),
            );
            let (stack_parameters, side_parameters) = model.parameter_counts();
            let run = json!({
                "arm": arm_name,
                "seed": seed,
                "stack": config,
                "train": run_config,
                "stack_parameters": stack_parameters,
                "side_parameters": side_parameters,
                "train_seconds": train_seconds,
                "eval_seconds": eval_seconds,
                "records": records,
                "dev_text_nll": nll,
                "a5_stack": a5,
                "lanes": lanes_report,
                "served": best.as_ref().map(|b| json!({
                    "lane": b.lane,
                    "order": b.order,
                    "fit_accuracy": b.fit_accuracy,
                    "minimal_order": b.minimal_order,
                    "max_trace_deviation": b.max_trace_deviation,
                    "eval": served,
                })),
            });
            fs::write(
                out.join("runs")
                    .join(format!("{arm_name}-seed{seed}.json").to_lowercase()),
                serde_json::to_vec_pretty(&run)?,
            )?;
            summary.push(json!({
                "arm": run["arm"],
                "seed": seed,
                "dev_text_nll": nll,
                "a5_all_positions": a5.all_positions,
                "a5_by_position": a5.by_position,
                "served": run["served"],
                "train_seconds": train_seconds,
            }));
        }
    }
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&json!({
            "text": {
                "path": text_path.display().to_string(),
                "sha256": text_sha256,
                "file_tokens": file_tokens,
                "train_tokens": train_tokens,
                "dev_tokens": dev_tokens,
                "dev_path": dev_path.as_ref().map(|p| p.display().to_string()),
                "dev_sha256": dev_sha256,
                "dev_region_start": dev_start,
            },
            "task": {"generators": task.generators, "classes": A5_ORDER},
            "data_seconds": data_seconds,
            "runs": summary,
        }))?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((mode, rest)) = arguments.split_first() else {
        return Err(invalid(
            "usage: tracking-lanes a5|mixed out=NEW_REPORT_ROOT key=value ...",
        ));
    };
    match mode.as_str() {
        "a5" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "kinds",
                    "seeds",
                    "lanes",
                    "hidden",
                    "steps",
                    "batch",
                    "lr",
                    "train_length",
                    "eval_lengths",
                    "eval_words",
                    "fit_words",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = a5(&args, &out);
            finish(&out, result)
        }
        "mixed" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "text",
                    "train_tokens",
                    "dev",
                    "dev_tokens",
                    "arms",
                    "seeds",
                    "lanes",
                    "width",
                    "heads",
                    "pattern",
                    "mlp",
                    "context",
                    "steps",
                    "batch",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "a5_weight",
                    "a5_start",
                    "lane_lr",
                    "dev_windows",
                    "a5_words",
                    "snap_lengths",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = mixed(&args, &out);
            finish(&out, result)
        }
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}
