//! D2/G v1 (whole-project synthesis §3 M1, §4 D2): architectural exact
//! relational memory in the recurrence-primary stack against the same stack
//! with an equal-size MLP and no store (`uor_r4_training::stack_aerm`).
//!
//! G v1 replaces D2's trigger-gated read with the always-on, address-driven
//! dual-register read (`stack_aerm`'s G v1 read policy). Everything else —
//! data, schedule, seeds, evaluation and the control — is D2's.
//!
//! ```text
//! aerm-probe run out=NEW_REPORT_ROOT text=TRAIN.u16 dev=DEV.u16 tokenizer=TOKENIZER.json \
//!   [arms=aerm,control] [seeds=1] [train_tokens=16777216] [dev_tokens=249000] [width=128] \
//!   [heads=4] [mlp=384] [pattern=rrar] [context=256] [split=2] [steps=1500] [batch=16] \
//!   [lr=0.003] [warmup=100] [weight_decay=0.1] [clip=1] [tag_weight=1] [trigger_weight=1] \
//!   [trigger_positive=5] [dev_windows=512] [eval_episodes=512] [free_episodes=32] [verify=256]
//! aerm-probe summarize out=NEW_REPORT_ROOT roots=ROOT,ROOT,... [gates=g1|d2]
//! ```
//!
//! `run` claims its report root before building anything, verifies `verify`
//! generated episodes token for token against the shared literal-role
//! protocol encoder, trains each arm with each seed on identical data, and
//! evaluates development text NLL, fresh relation dialogues (training
//! templates and names), held-out-template dialogues and free-running answers.
//! `summarize` applies the pre-registered gate set to sealed run roots:
//! `gates=g1` is Lab 1's five-part G v1 acceptance (held-out Updated ≥ 0.90 in
//! every seed, an equal-parameter control, text NLL within 0.05, the failure
//! trace reported, sealed roots); `gates=d2` reproduces D2's original gates.
//! Set RAYON_NUM_THREADS to bound the threads.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::{ReadScore, StackArch, StackConfig};
use uor_r4_training::stack_aerm::{
    evaluate_dialogues, free_running, text_nll, train_aerm, AermConfig, AermModel, QueryClass,
    RelationWorld,
};
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

/// Pre-registered D2 gates (synthesis §4) and the G v1 acceptance gates
/// (Lab 1's 17:33 UTC assignment).
const GATE_ACCURACY: f64 = 0.90;
const GATE_MARGIN: f64 = 0.30;
const GATE_TEXT: f64 = 0.05;
/// The equal-parameter control's total may differ from the memory arm's by at
/// most this fraction (the MLP widening rounds to whole units).
const GATE_PARAMETER_TOLERANCE: f64 = 0.0005;
const DIALOGUE_SEED: u64 = 9_001;
const HELD_SEED: u64 = 9_002;
const FREE_SEED: u64 = 9_003;
const VERIFY_SEED: u64 = 9_004;

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

/// Checks generated episodes token for token against the shared protocol.
fn verify_protocol(
    world: &RelationWorld,
    tokenizer: &ByteBpeTokenizer,
    count: usize,
    context: usize,
) -> Result<Value> {
    let protocol = DialogueProtocol::literal_roles_v1(tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    if protocol.bos_id != world.bos || protocol.eos_id != world.eos {
        return Err(invalid("the world's BOS/EOS differ from the protocol's"));
    }
    let mut rng = Rng::new(VERIFY_SEED);
    let mut tokens = 0usize;
    for index in 0..count {
        let episode = world.episode(&mut rng, index % 2 == 1, context + 1)?;
        let mut messages = Vec::new();
        for (user, reply) in &episode.messages {
            messages.push(Message {
                role: "user",
                content: user,
            });
            messages.push(Message {
                role: "assistant",
                content: reply,
            });
        }
        let encoded = encoder.encode_document(&messages);
        if encoded.tokens != episode.tokens || encoded.response_mask != episode.response_mask {
            return Err(invalid(format!(
                "episode {index} differs from the protocol encoding"
            )));
        }
        tokens += episode.tokens.len();
    }
    Ok(json!({
        "episodes": count,
        "tokens": tokens,
        "identical_tokens_and_response_masks": true,
        "protocol": protocol,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
    }))
}

fn run(args: &Args, out: &Path) -> Result<()> {
    let text_path = PathBuf::from(args.required("text")?);
    let dev_path = PathBuf::from(args.required("dev")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let train_tokens: usize = args.number("train_tokens", 16_777_216)?;
    let dev_tokens: usize = args.number("dev_tokens", 249_000)?;
    let arms: Vec<String> = args.list("arms", "aerm,control")?;
    let seeds: Vec<u64> = args.list("seeds", "1")?;
    let context: usize = args.number("context", 256)?;
    let split: usize = args.number("split", 2)?;
    let dev_windows: usize = args.number("dev_windows", 512)?;
    let eval_episodes: usize = args.number("eval_episodes", 512)?;
    let free_episodes: usize = args.number("free_episodes", 32)?;
    let verify: usize = args.number("verify", 256)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::new(&encode, protocol.bos_id, protocol.eos_id)?;
    let template = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 4096,
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
        memory: None,
    };
    template.validate()?;
    let train_template = AermConfig {
        steps: args.number("steps", 1500)?,
        batch: args.number("batch", 16)?,
        context,
        learning_rate: args.number("lr", 0.003)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        tag_weight: args.number("tag_weight", 1.0)?,
        trigger_weight: args.number("trigger_weight", 1.0)?,
        trigger_positive_weight: args.number("trigger_positive", 5.0)?,
        data_seed: 0,
    };
    let started = Instant::now();
    let protocol_check = verify_protocol(&world, &tokenizer, verify, context)?;
    let text_sha256 = sha256_file(&text_path)?;
    let dev_sha256 = sha256_file(&dev_path)?;
    let tokenizer_sha256 = sha256_file(&tokenizer_path)?;
    let train = read_u16_range(&text_path, 0, train_tokens)?;
    let dev = read_u16_range(&dev_path, 0, dev_tokens)?;
    if train.iter().chain(&dev).any(|&t| t >= 4096) {
        return Err(invalid("token files have ids at or above 4096"));
    }
    let device = Device::Cpu;
    // The control widens its MLP by the memory branch's parameter count.
    let probe = AermModel::new(template.clone(), split, true, 0, &device)?;
    let (_, _, memory_parameters) = probe.parameter_counts();
    let unit = 3 * template.width * template.layers();
    let extra_mlp = (memory_parameters + unit / 2) / unit;
    drop(probe);
    fs::create_dir(out.join("runs"))?;
    println!(
        "{:<8} {:>4} {:>8} {:>9} {:>8} {:>8} {:>8} {:>8}",
        "arm", "seed", "train_s", "text_nll", "updated", "first", "held_upd", "free"
    );
    for &seed in &seeds {
        for arm in &arms {
            let memory = match arm.as_str() {
                "aerm" => true,
                "control" => false,
                other => return Err(invalid(format!("arm is aerm or control, got {other}"))),
            };
            let config = StackConfig {
                seed,
                mlp_hidden: template.mlp_hidden + if memory { 0 } else { extra_mlp },
                ..template.clone()
            };
            let model = AermModel::new(config.clone(), split, memory, seed, &device)?;
            let train_config = AermConfig {
                data_seed: 5_000 + seed,
                ..train_template.clone()
            };
            let (records, train_seconds) = train_aerm(&model, &world, &train, &train_config)?;
            let eval_started = Instant::now();
            let (nll, triggers_per_thousand) =
                text_nll(&model, &dev, context, dev_windows, 16, world.eos)?;
            let dialogues = evaluate_dialogues(
                &model,
                &world,
                eval_episodes,
                DIALOGUE_SEED,
                false,
                context,
                16,
            )?;
            let held =
                evaluate_dialogues(&model, &world, eval_episodes, HELD_SEED, true, context, 16)?;
            let free = free_running(&model, &world, free_episodes, FREE_SEED, false, context, 16)?;
            let eval_seconds = eval_started.elapsed().as_secs_f64();
            let free_exact = free.iter().filter(|(gold, got)| gold == got).count();
            let free_rows: Vec<Value> = free
                .iter()
                .map(|(gold, got)| {
                    json!({
                        "gold": tokenizer.decode(gold),
                        "generated": tokenizer.decode(got),
                        "exact": gold == got,
                    })
                })
                .collect();
            let fmt = |v: Option<f64>| v.map_or("-".to_owned(), |v| format!("{v:.3}"));
            println!(
                "{:<8} {:>4} {:>8.1} {:>9.4} {:>8} {:>8} {:>8} {:>5}/{}",
                arm,
                seed,
                train_seconds,
                nll,
                fmt(dialogues.accuracy(QueryClass::Updated)),
                fmt(dialogues.accuracy(QueryClass::First)),
                fmt(held.accuracy(QueryClass::Updated)),
                free_exact,
                free.len()
            );
            let (stack_parameters, head_parameters, memory_branch) = model.parameter_counts();
            let run = json!({
                "arm": arm,
                "seed": seed,
                "read_policy": "always_on_address_driven",
                "stack": config,
                "split": split,
                "train": train_config,
                "parameters": {
                    "stack": stack_parameters,
                    "heads": head_parameters,
                    "memory_branch": memory_branch,
                    "total": stack_parameters + head_parameters + memory_branch,
                    "control_extra_mlp_units": if memory { 0 } else { extra_mlp },
                },
                "train_seconds": train_seconds,
                "eval_seconds": eval_seconds,
                "records": records,
                "text": {"dev_nll": nll, "dev_windows": dev_windows, "context": context,
                         "non_none_triggers_per_thousand_tokens": triggers_per_thousand},
                "dialogues": dialogues,
                "held_out": held,
                "free_running": {"episodes": free.len(), "exact": free_exact, "rows": free_rows},
            });
            fs::write(
                out.join("runs").join(format!("{arm}-s{seed}.json")),
                serde_json::to_vec_pretty(&run)?,
            )?;
        }
    }
    let summary = json!({
        "schema": "uor-r4.g1-aerm-probe/1",
        "read_policy": "always_on_address_driven",
        "arms": arms,
        "seeds": seeds,
        "stack_template": template,
        "train_template": train_template,
        "split": split,
        "control_extra_mlp_units": extra_mlp,
        "memory_branch_parameters": memory_parameters,
        "protocol_check": protocol_check,
        "vocabulary": world.vocabulary(),
        "data": {
            "text": text_path, "text_sha256": text_sha256, "train_tokens": train_tokens,
            "dev": dev_path, "dev_sha256": dev_sha256, "dev_tokens": dev_tokens,
            "tokenizer": tokenizer_path, "tokenizer_sha256": tokenizer_sha256,
        },
        "evaluation_seeds": {"dialogues": DIALOGUE_SEED, "held_out": HELD_SEED, "free_running": FREE_SEED, "protocol": VERIFY_SEED},
        "eval_episodes": eval_episodes,
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(())
}

fn accuracy(run: &Value, set: &str, class: &str) -> Option<f64> {
    let score = &run[set]["classes"][class];
    let queries = score["queries"].as_f64()?;
    (queries > 0.0).then(|| score["correct"].as_f64().unwrap_or(0.0) / queries)
}

fn summarize(args: &Args, out: &Path) -> Result<()> {
    let roots: Vec<PathBuf> = args.list("roots", "")?;
    let mut runs: BTreeMap<(String, u64), Value> = BTreeMap::new();
    let mut identities = Vec::new();
    for root in &roots {
        report_output::verify(root)?;
        identities.push(
            json!({"root": root, "manifest_sha256": sha256_file(&root.join("manifest.json"))?}),
        );
        for entry in fs::read_dir(root.join("runs"))? {
            let path = entry?.path();
            let run: Value = serde_json::from_slice(&fs::read(&path)?)?;
            let arm = run["arm"].as_str().unwrap_or_default().to_owned();
            let seed = run["seed"].as_u64().unwrap_or_default();
            if runs.insert((arm.clone(), seed), run).is_some() {
                return Err(invalid(format!("duplicate run {arm} seed {seed}")));
            }
        }
    }
    let seeds: Vec<u64> = runs
        .keys()
        .map(|(_, s)| *s)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let gates = args.0.get("gates").map(String::as_str).unwrap_or("g1");
    if !matches!(gates, "g1" | "d2") {
        return Err(invalid(format!("gates is g1 or d2, got {gates}")));
    }
    let mut rows = Vec::new();
    let mut all_pass = !seeds.is_empty();
    let summary = if gates == "g1" {
        for seed in &seeds {
            let aerm = runs.get(&("aerm".to_owned(), *seed));
            let control = runs.get(&("control".to_owned(), *seed));
            let (Some(aerm), Some(control)) = (aerm, control) else {
                all_pass = false;
                rows.push(json!({"seed": seed, "complete": false}));
                continue;
            };
            let held = accuracy(aerm, "held_out", "Updated");
            let fresh = accuracy(aerm, "dialogues", "Updated");
            let control_fresh = accuracy(control, "dialogues", "Updated");
            let aerm_total = aerm["parameters"]["total"].as_f64();
            let control_total = control["parameters"]["total"].as_f64();
            let text_delta = aerm["text"]["dev_nll"]
                .as_f64()
                .zip(control["text"]["dev_nll"].as_f64())
                .map(|(a, c)| a - c);
            let held_pass = held.is_some_and(|h| h >= GATE_ACCURACY);
            let text_pass = text_delta.is_some_and(|d| d <= GATE_TEXT);
            let parameters_pass = aerm_total.zip(control_total).is_some_and(|(a, c)| {
                (a - c).abs() / a.max(c).max(1.0) <= GATE_PARAMETER_TOLERANCE
            });
            let pass = held_pass && text_pass && parameters_pass;
            all_pass &= pass;
            rows.push(json!({
                "seed": seed, "complete": true,
                "held_out_updated": held,
                "held_out_first": accuracy(aerm, "held_out", "First"),
                "control_held_out_updated": accuracy(control, "held_out", "Updated"),
                "in_distribution_updated": fresh, "control_updated": control_fresh,
                "margin": fresh.zip(control_fresh).map(|(a, c)| a - c),
                "aerm_total_parameters": aerm_total, "control_total_parameters": control_total,
                "aerm_text_nll": aerm["text"]["dev_nll"], "control_text_nll": control["text"]["dev_nll"],
                "text_delta": text_delta,
                "trace": aerm["held_out"]["trace"], "read_events": aerm["held_out"]["read_events"],
                "held_out_pass": held_pass, "text_pass": text_pass,
                "parameters_pass": parameters_pass, "pass": pass,
            }));
        }
        json!({
            "schema": "uor-r4.g1-aerm-gate-summary/1",
            "gates": {
                "held_out_updated_accuracy_at_least": GATE_ACCURACY,
                "text_nll_delta_at_most": GATE_TEXT,
                "parameter_relative_tolerance": GATE_PARAMETER_TOLERANCE,
                "every_seed": true,
            },
            "roots": identities,
            "seeds": rows,
            "outcome": if all_pass { "PASS" } else { "FAIL" },
        })
    } else {
        for seed in &seeds {
            let aerm = runs.get(&("aerm".to_owned(), *seed));
            let control = runs.get(&("control".to_owned(), *seed));
            let (Some(aerm), Some(control)) = (aerm, control) else {
                all_pass = false;
                rows.push(json!({"seed": seed, "complete": false}));
                continue;
            };
            let a = accuracy(aerm, "dialogues", "Updated");
            let c = accuracy(control, "dialogues", "Updated");
            let text_delta = aerm["text"]["dev_nll"]
                .as_f64()
                .zip(control["text"]["dev_nll"].as_f64())
                .map(|(a, c)| a - c);
            let accuracy_pass = a.is_some_and(|a| a >= GATE_ACCURACY);
            let margin = a.zip(c).map(|(a, c)| a - c);
            let margin_pass = margin.is_some_and(|m| m >= GATE_MARGIN);
            let text_pass = text_delta.is_some_and(|d| d <= GATE_TEXT);
            let pass = accuracy_pass && margin_pass && text_pass;
            all_pass &= pass;
            rows.push(json!({
                "seed": seed, "complete": true,
                "aerm_updated": a, "control_updated": c, "margin": margin,
                "aerm_text_nll": aerm["text"]["dev_nll"], "control_text_nll": control["text"]["dev_nll"],
                "text_delta": text_delta,
                "accuracy_pass": accuracy_pass, "margin_pass": margin_pass, "text_pass": text_pass, "pass": pass,
            }));
        }
        json!({
            "schema": "uor-r4.d2-aerm-probe-summary/1",
            "gates": {"aerm_updated_accuracy_at_least": GATE_ACCURACY, "margin_over_control_at_least": GATE_MARGIN,
                      "text_nll_delta_at_most": GATE_TEXT, "every_seed": true},
            "roots": identities,
            "seeds": rows,
            "outcome": if all_pass { "PASS" } else { "FAIL" },
        })
    };
    println!("{}", serde_json::to_string_pretty(&summary)?);
    fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((mode, rest)) = arguments.split_first() else {
        return Err(invalid(
            "usage: aerm-probe run|summarize out=NEW_REPORT_ROOT key=value ...",
        ));
    };
    match mode.as_str() {
        "run" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "text",
                    "dev",
                    "tokenizer",
                    "train_tokens",
                    "dev_tokens",
                    "arms",
                    "seeds",
                    "width",
                    "heads",
                    "mlp",
                    "pattern",
                    "context",
                    "split",
                    "steps",
                    "batch",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "clip",
                    "tag_weight",
                    "trigger_weight",
                    "trigger_positive",
                    "dev_windows",
                    "eval_episodes",
                    "free_episodes",
                    "verify",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = run(&args, &out);
            finish(&out, result)
        }
        "summarize" => {
            let args = Args::parse(rest, &["out", "roots", "gates"])?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = summarize(&args, &out);
            finish(&out, result)
        }
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}
