//! Full-path cost measurement harness for the width-576 dialogue bundle.
//!
//! Fixture-dependent: ignored by default, and fails when run without the local
//! `dialogue-child-bundle-1` fixture or its recorded replay. Run it explicitly:
//!
//! ```text
//! UOR_R4_M1_COST_REPORT_ROOT=/new/attempt/dir UOR_R4_SOURCE_COMMIT=<sha> \
//!   cargo test -p uor-r4-integer --release --test full_path_m1_cost -- --ignored --nocapture
//! ```
//!
//! With `UOR_R4_M1_COST_REPORT_ROOT` set, that directory is claimed exclusively
//! (`report_output::claim`) before the bundle loads, the report is written into
//! it, and the attempt is sealed and verified. Without the variable nothing is
//! written. Measures cold load, tokenizer encode, prompt ingestion, per-step
//! latency over the recorded dialogue replay (exact parity asserted), session
//! save/restore latency with a restore-continuation comparison, and process RSS
//! via `ps`. Bytes touched per token is an analytic count, not a measurement.
//! Latency ceilings are recorded next to the measured values with a computed
//! `meets` flag; they are not asserted. One run is not a qualification.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::model::SlotTarget;
use uor_r4_integer::report_output;
use uor_r4_integer::session::ChatSession;
use uor_r4_integer::{IntegerError, Result};

const BUNDLE_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1";
const RESPONSES_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-observation-1/responses-integer.json";
const REPORT_ROOT_ENV: &str = "UOR_R4_M1_COST_REPORT_ROOT";
const SOURCE_COMMIT_ENV: &str = "UOR_R4_SOURCE_COMMIT";
const REPORT_FILE: &str = "m1-cost.json";

/// Declared alpha ceilings, recorded against the measurement, not asserted.
const CEILING_COLD_LOAD_MS: f64 = 250.0;
const CEILING_STEP_MEAN_MS: f64 = 4.0;
const CEILING_STEP_P90_MS: f64 = 4.0;
/// Greedy decisions compared after a save/restore round trip.
const RESTORE_CONTINUATION_STEPS: usize = 32;

fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}

fn require_fixture(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(invalid(format!(
            "required fixture {} is missing; this ignored test needs the local dialogue-child-bundle-1 fixture",
            path.display()
        )))
    }
}

fn process_rss_mb() -> Option<f64> {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let kib: f64 = std::str::from_utf8(&output.stdout)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(kib / 1024.0)
}

fn sysctl(name: &str) -> Value {
    Command::new("sysctl")
        .args(["-n", name])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| Value::from(text.trim()))
        .unwrap_or_else(|| Value::from("UNAVAILABLE"))
}

fn rss_value(rss: Option<f64>) -> Value {
    rss.map(Value::from)
        .unwrap_or_else(|| Value::from("UNAVAILABLE"))
}

fn greedy_choice(probs: &[u64]) -> u32 {
    let mut best_val = 0u64;
    let mut best_idx = 0u32;
    for (idx, &p) in probs.iter().enumerate() {
        if p > best_val {
            best_val = p;
            best_idx = idx as u32;
        }
    }
    best_idx
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value
        .get(key)
        .ok_or_else(|| invalid(format!("replay fixture lacks '{key}'")))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    field(value, key)?
        .as_array()
        .ok_or_else(|| invalid(format!("replay fixture '{key}' is not an array")))
}

fn token(value: &Value) -> Result<u32> {
    value
        .as_u64()
        .and_then(|id| u32::try_from(id).ok())
        .ok_or_else(|| invalid("replay fixture token is not a u32"))
}

fn percentile(sorted: &[f64], percent: usize) -> f64 {
    sorted[(sorted.len() * percent / 100).min(sorted.len() - 1)]
}

fn ceiling(ceiling: f64, measured: f64) -> Value {
    json!({ "ceiling": ceiling, "measured": measured, "meets": measured <= ceiling })
}

#[test]
#[ignore = "requires the local dialogue-child-bundle-1 fixture; run with --ignored"]
fn test_full_path_m1_cost_dialogue576() -> Result<()> {
    let bundle_path = Path::new(BUNDLE_PATH);
    let responses_path = Path::new(RESPONSES_PATH);
    require_fixture(bundle_path)?;
    require_fixture(responses_path)?;
    let report_root = std::env::var_os(REPORT_ROOT_ENV).map(PathBuf::from);
    if let Some(root) = &report_root {
        report_output::claim(root)?;
    }
    let started_unix_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let load_average_before = sysctl("vm.loadavg");
    let initial_rss = process_rss_mb();

    // 1. Cold bundle load.
    let cold_start = Instant::now();
    let bundle = Bundle::load(bundle_path)?;
    let cold_load_ms = cold_start.elapsed().as_secs_f64() * 1000.0;
    let post_load_rss = process_rss_mb();
    let model = bundle.model();
    assert_eq!(model.config().width, 576);
    assert_eq!(model.config().vocab_size, 4096);

    // 2. Tokenizer encode.
    let tokenizer = bundle.tokenizer();
    let prompts = [
        "Hello!",
        "What is your name and what can you do?",
        "Please remember that the secret code is alpha-7-delta. What was the code?",
        "In geometric language modeling, we replace soft attention matrices and dense MLPs with prime-addressed exact memory, Riemann zeta-zero phase coordinates on the 8-torus, and discrete Hopf holonomy over S3.",
    ];
    let mut encoded_tokens = 0usize;
    let mut encode_nanos = 0u128;
    for prompt in &prompts {
        let start = Instant::now();
        let tokens = tokenizer.encode(prompt);
        encode_nanos += start.elapsed().as_nanos();
        encoded_tokens += tokens.len();
    }
    let encode_us_per_token = encode_nanos as f64 / encoded_tokens as f64 / 1000.0;

    // 3. Prompt ingestion into a fresh session.
    let mut session = model.new_conversational_session();
    let ingest_tokens = tokenizer.encode(prompts[2]);
    let mut ingest_nanos = 0u128;
    for &tok in &ingest_tokens {
        let start = Instant::now();
        model.step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)?;
        ingest_nanos += start.elapsed().as_nanos();
    }
    let ingest_ms_per_token = ingest_nanos as f64 / ingest_tokens.len() as f64 / 1_000_000.0;

    // 4. Per-step latency over the recorded replay, with exact greedy parity.
    let data: Value = serde_json::from_slice(&fs::read(responses_path)?)?;
    let rows = array(&data, "rows")?;
    let mut step_us: Vec<f64> = Vec::with_capacity(3000);
    let mut decisions_verified = 0usize;
    let mut turns_replayed = 0usize;
    let mut peak_rss = post_load_rss;
    let replay_start = Instant::now();
    for (row_idx, row) in rows.iter().enumerate() {
        let turns = array(row, "turns")?;
        let mut session = model.new_conversational_session();
        for (turn_idx, turn) in turns.iter().enumerate() {
            turns_replayed += 1;
            let conversation = field(turn, "native_conversation_turn")?;
            let appended = array(conversation, "appended_token_ids")?;
            let generation = field(field(conversation, "dialogue")?, "generation")?;
            let decisions = array(generation, "decisions")?;
            let mut step = None;
            for id in appended {
                let start = Instant::now();
                step = Some(model.step_conversational(
                    &mut session,
                    token(id)?,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )?);
                step_us.push(start.elapsed().as_nanos() as f64 / 1000.0);
            }
            let mut step = step.ok_or_else(|| invalid("replay turn appends no tokens"))?;
            for (dec_idx, decision) in decisions.iter().enumerate() {
                let expected = token(field(decision, "selected_token")?)?;
                let actual = greedy_choice(&step.probabilities);
                assert_eq!(
                    actual, expected,
                    "replay departure at request {row_idx}, turn {turn_idx}, decision {dec_idx}"
                );
                decisions_verified += 1;
                if dec_idx + 1 < decisions.len() || turn_idx + 1 < turns.len() {
                    let start = Instant::now();
                    step = model.step_conversational(
                        &mut session,
                        actual,
                        SlotTarget::Dialogue,
                        ReadMode::Enabled,
                    )?;
                    step_us.push(start.elapsed().as_nanos() as f64 / 1000.0);
                }
            }
            if let Some(rss) = process_rss_mb() {
                peak_rss = Some(peak_rss.map_or(rss, |peak: f64| peak.max(rss)));
            }
        }
    }
    let replay_seconds = replay_start.elapsed().as_secs_f64();
    assert_eq!(rows.len(), 38);
    assert_eq!(turns_replayed, 58);
    assert_eq!(decisions_verified, 1433);
    assert_eq!(step_us.len(), 2526);

    let mut sorted = step_us.clone();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len() as f64;
    let mean_us = sorted.iter().sum::<f64>() / n;
    let stddev_us = (sorted.iter().map(|&x| (x - mean_us).powi(2)).sum::<f64>() / n).sqrt();
    let step_latency = json!({
        "mean_ms": mean_us / 1000.0,
        "stddev_ms": stddev_us / 1000.0,
        "min_ms": sorted[0] / 1000.0,
        "p50_ms": percentile(&sorted, 50) / 1000.0,
        "p90_ms": percentile(&sorted, 90) / 1000.0,
        "p95_ms": percentile(&sorted, 95) / 1000.0,
        "p99_ms": percentile(&sorted, 99) / 1000.0,
        "max_ms": sorted[sorted.len() - 1] / 1000.0,
        "tok_per_sec": n / replay_seconds,
    });
    let mean_ms = mean_us / 1000.0;
    let p90_ms = percentile(&sorted, 90) / 1000.0;

    // 5. Session save/restore latency and restore continuation.
    let sha = bundle.identity();
    let mut chat = ChatSession::new(&bundle, Some("Persistent system persona."), 12345)?;
    for i in 0..50 {
        chat.ingest_user_turn(&format!("User message {i} with facts to remember."))?;
    }
    let save_start = Instant::now();
    let serialized = chat.to_serialized(Some(sha));
    let save_ms = save_start.elapsed().as_secs_f64() * 1000.0;
    let restore_start = Instant::now();
    let mut restored = ChatSession::from_serialized(&bundle, serialized, sha)?;
    let restore_ms = restore_start.elapsed().as_secs_f64() * 1000.0;
    let serialized_round_trip_equal = serde_json::to_value(restored.to_serialized(Some(sha)))?
        == serde_json::to_value(chat.to_serialized(Some(sha)))?;
    let probe = "What was the first fact?";
    chat.ingest_user_turn(probe)?;
    restored.ingest_user_turn(probe)?;
    let mut first_divergence = None;
    for step_idx in 0..RESTORE_CONTINUATION_STEPS {
        let expected = chat
            .last_step()
            .ok_or_else(|| invalid("original session lacks a step"))?;
        let actual = restored
            .last_step()
            .ok_or_else(|| invalid("restored session lacks a step"))?;
        if expected.probabilities != actual.probabilities {
            first_divergence = Some(step_idx);
            break;
        }
        let next = greedy_choice(&expected.probabilities);
        chat.step_token(next, SlotTarget::Dialogue)?;
        restored.step_token(next, SlotTarget::Dialogue)?;
    }

    // 6. Analytic bytes touched per token (a count, not a measurement).
    let width = 576usize;
    let read_dim = 64usize;
    let vocab_size = 4096usize;
    let memory_slots = 256usize; // 32 persistent + 224 dialogue
    let bytes_touched = vocab_size * width / 2 // 4-bit unembedding
        + read_dim * width / 2 // 4-bit score projection
        + memory_slots * read_dim * 2 // INT16 memory keys
        + memory_slots * width * 4 // INT32 memory values
        + width * 2 // INT16 state
        + 8 * 4 // zeta phases
        + 4 * 4; // Hopf coordinates

    let load_average_after = sysctl("vm.loadavg");
    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let source_commit =
        std::env::var(SOURCE_COMMIT_ENV).unwrap_or_else(|_| "UNRECORDED".to_string());
    println!("full-path cost harness, one run (not a qualification)");
    println!("  cold load            {cold_load_ms:.3} ms");
    println!("  tokenizer encode     {encode_us_per_token:.3} us/token ({encoded_tokens} tokens)");
    println!("  prompt ingestion     {ingest_ms_per_token:.3} ms/token");
    println!(
        "  step mean / p90      {mean_ms:.3} / {p90_ms:.3} ms over {} steps",
        step_us.len()
    );
    println!("  replay parity        {decisions_verified} decisions, 0 departures");
    println!("  save / restore       {save_ms:.3} / {restore_ms:.3} ms");
    println!("  restore continuation first divergence: {first_divergence:?}");
    println!("  load average         {load_average_before} -> {load_average_after}");

    let report = json!({
        "schema": "uor-r4.m1-cost-profile/2",
        "run": {
            "started_unix_s": started_unix_s,
            "harness": "crates/uor-r4-integer/tests/full_path_m1_cost.rs",
            "build_profile": build_profile,
            "source_commit": source_commit,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "cpu": sysctl("machdep.cpu.brand_string"),
            "load_average_before": load_average_before,
            "load_average_after": load_average_after,
            "status": "one run; not a qualification",
        },
        "artifact": {
            "bundle": "dialogue-child-bundle-1",
            "path": bundle_path.display().to_string(),
            "identity": sha,
            "width": width,
            "vocab_size": vocab_size,
            "read_dim": read_dim,
            "context_capacity": model.config().context,
        },
        "metrics": {
            "cold_load_ms": cold_load_ms,
            "tokenizer_encode_us_per_tok": encode_us_per_token,
            "prompt_ingest_ms_per_tok": ingest_ms_per_token,
            "step_latency": step_latency,
            "session_save_ms": save_ms,
            "session_restore_ms": restore_ms,
            "initial_rss_mb": rss_value(initial_rss),
            "post_load_rss_mb": rss_value(post_load_rss),
            "peak_rss_mb": rss_value(peak_rss),
            "bytes_touched_per_token": bytes_touched,
            "bytes_touched_basis": "analytic count in the harness source; not measured",
            "energy_soc_joules_per_token": "UNAVAILABLE",
        },
        "ceilings": {
            "cold_load_ms": ceiling(CEILING_COLD_LOAD_MS, cold_load_ms),
            "step_mean_ms": ceiling(CEILING_STEP_MEAN_MS, mean_ms),
            "step_p90_ms": ceiling(CEILING_STEP_P90_MS, p90_ms),
        },
        "restore": {
            "serialized_round_trip_equal": serialized_round_trip_equal,
            "continuation_steps_compared": RESTORE_CONTINUATION_STEPS,
            "continuation_identical": first_divergence.is_none(),
            "first_divergence_step": first_divergence,
        },
        "parity_gate": {
            "requests": rows.len(),
            "turns": turns_replayed,
            "verified_decisions": decisions_verified,
            "departures": 0,
            "step_calls": step_us.len(),
        },
    });

    match report_root {
        Some(root) => {
            fs::write(root.join(REPORT_FILE), serde_json::to_vec_pretty(&report)?)?;
            report_output::seal(&root)?;
            report_output::verify(&root)?;
            println!("  report sealed at {}", root.display());
        }
        None => println!("  no report written ({REPORT_ROOT_ENV} is unset)"),
    }
    Ok(())
}
