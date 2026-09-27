//! Bounded offline observation of an existing learned dialogue artifact.
//! No training, integer export, fresh-panel evaluation or chat qualification.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, error::Error, fs, io, path::PathBuf, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    dialogue_artifact::LegacyDialogueArtifact, joint_evaluation::JointGenerationStop,
    joint_model::ReadMode, sha256_file,
};

const MAX_NEW_TOKENS: usize = 32;
const CONTEXT: usize = 256;
const PREFIX_TOLERANCE: f64 = 1e-5;
const PANEL_SHA256: &str = "10dfc0e7c529feecfa61e393b22a404ffee561339281abf5ea03f53964c6cfd2";

// Unknown fields, including the sealed fresh array and expected answers, are
// skipped by serde. Only already-open requests enter this observation.
#[derive(Deserialize)]
struct DevelopmentPanel {
    development: Vec<Request>,
}
#[derive(Deserialize, Serialize)]
struct Request {
    id: String,
    category: String,
    user_turns: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 5 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "usage: dialogue-artifact-replay FIT_ROOT CORPUS_MANIFEST TOKENIZER PANEL NEW_REPORT_ROOT").into());
    }
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    if source.len() != 40 || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "build with full UOR_BUILD_SOURCE_COMMIT",
        )
        .into());
    }
    let report = &args[4];
    report_output::claim(report)?;
    let result = observe(&args, source);
    if let Err(error) = &result {
        write_json(
            report.join("failed-attempt.json"),
            &json!({
                "status": "EXECUTION_FAILED", "error": error.to_string(),
                "replay_source_commit": source,
                "scope": "Incomplete observation; this is not a model-quality verdict"
            }),
        )?;
    }
    report_output::seal(report)?;
    report_output::verify(report)?;
    result?;
    println!(
        "{}",
        json!({"report_root": report, "status": "OBSERVATION_COMPLETE"})
    );
    Ok(())
}

fn observe(args: &[PathBuf], source: &str) -> Result<(), Box<dyn Error>> {
    let clock = Instant::now();
    let report = &args[4];
    if sha256_file(&args[3])? != PANEL_SHA256 {
        return Err(io::Error::other("development source panel identity differs").into());
    }
    let panel: DevelopmentPanel = serde_json::from_slice(&fs::read(&args[3])?)?;
    let unique: BTreeSet<_> = panel.development.iter().map(|row| &row.id).collect();
    if panel.development.len() != 38
        || unique.len() != 38
        || panel
            .development
            .iter()
            .map(|row| row.user_turns.len())
            .sum::<usize>()
            != 58
        || panel.development.iter().any(|row| {
            !row.id.starts_with("dev-")
                || ![1, 3].contains(&row.user_turns.len())
                || row.user_turns.iter().any(|s| s.trim().is_empty())
        })
    {
        return Err(io::Error::other("fixed development request inventory differs").into());
    }
    write_json(
        report.join("development-requests.json"),
        &serde_json::to_value(&panel.development)?,
    )?;
    let artifact = LegacyDialogueArtifact::load(&args[0], &args[1], &args[2], &Device::Cpu)?;
    let protocol = artifact.protocol();
    let encoder = protocol.bind(artifact.tokenizer())?;
    let first = panel
        .development
        .first()
        .ok_or_else(|| io::Error::other("empty development panel"))?;
    let mut parity_prefix = vec![protocol.bos_id];
    parity_prefix.extend(
        encoder
            .encode_user_prefix(&first.user_turns[0], false)
            .tokens,
    );
    let parity = artifact.compare_prefix(&parity_prefix, ReadMode::Enabled)?;
    write_json(
        report.join("prefix-comparison.json"),
        &serde_json::to_value(&parity)?,
    )?;
    if parity.max_absolute_probability_delta > PREFIX_TOLERANCE {
        return Err(io::Error::other(
            "loaded full-forward/incremental probability comparison differs",
        )
        .into());
    }
    let mut rows = Vec::new();
    let mut total_steps = 0usize;
    let mut total_selections = 0usize;
    let mut eos_stops = 0usize;
    let mut cycle_stops = 0usize;
    let mut cap_stops = 0usize;
    let mut row_stream = fs::File::create_new(report.join("responses.jsonl"))?;
    for request in panel.development {
        let mut history = vec![protocol.bos_id];
        let mut turns = Vec::new();
        for (index, user) in request.user_turns.iter().enumerate() {
            let suffix = encoder.encode_user_prefix(user, index != 0);
            if suffix.emitted_turns != 1 || suffix.special_token_occurrences != 0 {
                return Err(io::Error::other(
                    "request protocol cannot represent one ordinary user turn",
                )
                .into());
            }
            history.extend_from_slice(&suffix.tokens);
            if history.len() + MAX_NEW_TOKENS > CONTEXT {
                return Err(io::Error::other(
                    "exact dialogue history exceeds fixed context; no truncation",
                )
                .into());
            }
            let generation =
                artifact.generate_tokens(&history, ReadMode::Enabled, MAX_NEW_TOKENS)?;
            if generation.prompt_token_ids != history
                || generation.incremental_step_calls
                    != history.len() + generation.generated_token_ids.len() - 1
            {
                return Err(
                    io::Error::other("exact generation input or step accounting differs").into(),
                );
            }
            total_steps += generation.incremental_step_calls;
            total_selections += generation.generated_token_ids.len();
            let model_eos = matches!(generation.stop, JointGenerationStop::Eos);
            match generation.stop {
                JointGenerationStop::Eos => eos_stops += 1,
                JointGenerationStop::ShortCycle { .. } => cycle_stops += 1,
                JointGenerationStop::MaximumNewTokens => cap_stops += 1,
                JointGenerationStop::FirstSentenceBoundary => {
                    return Err(io::Error::other("unexpected stop policy").into())
                }
            }
            history.extend_from_slice(&generation.generated_token_ids);
            // Preserve every generated ID. A caller-supplied closure is
            // separate from model EOS and is added only before another request.
            let caller_eos = !model_eos && index + 1 < request.user_turns.len();
            if caller_eos {
                history.push(protocol.eos_id);
            }
            turns.push(json!({
                "turn": index + 1, "user": user,
                "appended_user_prefix_ids": suffix.tokens,
                "generation": generation,
                "model_eos": model_eos,
                "caller_eos_inserted_before_next_request": caller_eos,
                "retained_history_ids": history,
                "session_policy": "fresh session replays exact full prefix on each turn; no latency claim"
            }));
        }
        let row = json!({"id": request.id, "category": request.category, "turns": turns});
        serde_json::to_writer(&mut row_stream, &row)?;
        std::io::Write::write_all(&mut row_stream, b"\n")?;
        std::io::Write::flush(&mut row_stream)?;
        rows.push(row);
    }
    write_json(
        report.join("result.json"),
        &json!({
            "schema": "uor-r4.legacy-dialogue-output-observation/1",
            "status": "OBSERVATION_COMPLETE", "replay_source_commit": source,
            "executable_sha256": sha256_file(&std::env::current_exe()?)?,
            "source_sha256": {
                "dialogue_artifact.rs": hex::encode(Sha256::digest(include_bytes!("../src/dialogue_artifact.rs"))),
                "joint_model.rs": hex::encode(Sha256::digest(include_bytes!("../src/joint_model.rs"))),
                "joint_evaluation.rs": hex::encode(Sha256::digest(include_bytes!("../src/joint_evaluation.rs"))),
                "dialogue.rs": hex::encode(Sha256::digest(include_bytes!("../../uor-r4-tokenizer/src/dialogue.rs"))),
                "example.rs": hex::encode(Sha256::digest(include_bytes!("dialogue-artifact-replay.rs")))
            },
            "artifact": artifact.provenance(), "protocol": protocol,
            "protocol_identity": protocol.identity()?,
            "development_panel_sha256": PANEL_SHA256,
            "requests": 38, "responses": 58, "rows": rows,
            "max_new_tokens_per_response": MAX_NEW_TOKENS, "context": CONTEXT,
            "greedy": true, "fresh_heldout": false,
            "prefix_probability_comparison": parity, "prefix_tolerance": PREFIX_TOLERANCE,
            "generation_incremental_step_calls": total_steps,
            "generated_selections": total_selections,
            "stop_counts": {"model_eos": eos_stops, "short_cycle": cycle_stops, "token_cap": cap_stops},
            "elapsed_seconds": clock.elapsed().as_secs_f64(),
            "scope": "Offline continuous native artifact output under a new replay binary. No historical-backend bitwise reproduction, integer serving, latency/energy, official panel or general-chat qualification. No new training."
        }),
    )?;
    Ok(())
}

fn write_json(path: PathBuf, value: &Value) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    Ok(())
}
