//! Read-only diagnosis of a loaded A2 or A3 artifact's first answer decision.
//! The 4096-token scan below is offline analysis, never a serving decoder.
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::{
    encoder::CodeEncoder,
    features,
    output::{Action, OutputTrace},
    training::{token_nll_offline, TrainingEpisode},
    training_support::GateReadCounts,
    IntegratedModel, Session,
};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

#[path = "support/integrated_attention_a2_data.rs"]
mod integrated_attention_a2_data;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const A2_MODEL_SOURCE: &str = "3b7d2f5811337483c2276ef77e4c9fe27c6a10ba";
const A2_CORRECTED_MODEL_SOURCE: &str = "40542ede834fe223b5575feeeeae1b46677b3801";
const A3_MODEL_SOURCE: &str = "626421e739e9aa4572496d3d9287ace7034e7483";
const COLON_TOKEN: u16 = 42;

struct Args {
    repo: PathBuf,
    tokenizer: PathBuf,
    model: PathBuf,
    out: PathBuf,
}

fn args() -> AnyResult<Args> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: integrated-attention-a2-inspect REPO TOKENIZER MODEL OUT".into());
    }
    Ok(Args {
        repo: args[0].clone().into(),
        tokenizer: args[1].clone().into(),
        model: args[2].clone().into(),
        out: args[3].clone().into(),
    })
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> AnyResult<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn head_probe(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    session: &Session,
    selected: Option<u16>,
    target: u16,
) -> AnyResult<Value> {
    let features = features(&model.config, session.last_token, &session.state, selected);
    let features = features.as_slice();
    let copy_legal = selected.is_some();
    let greedy = model.output.greedy_branch(features, copy_legal)?;
    let generate_42 =
        model
            .output
            .score_action(features, copy_legal, Action::Generate(COLON_TOKEN))?;
    let generate_target =
        model
            .output
            .score_action(features, copy_legal, Action::Generate(target))?;
    let stop = model
        .output
        .score_action(features, copy_legal, Action::Stop)?;
    let copy = if copy_legal {
        Some(model.output.score_action(features, true, Action::Copy)?)
    } else {
        None
    };

    // This deliberately touches every vocabulary path *offline*. Serving
    // remains bounded local-branch decoding and selected-row probability.
    let mut top: Vec<(u16, f64)> = Vec::with_capacity(model.config.vocabulary);
    for token in 0..model.config.vocabulary {
        let token = u16::try_from(token)?;
        top.push((
            token,
            token_nll_offline(&model.output, features, selected, token)?,
        ));
    }
    let surface_probability_sum: f64 = top.iter().map(|(_, nll)| (-nll).exp()).sum();
    top.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    let best = top.first().ok_or("empty vocabulary")?;
    let top_four: Vec<_> = top
        .iter()
        .take(4)
        .map(|(token, nll)| {
            json!({"token":token,"text":tokenizer.decode(&[u32::from(*token)]),
                "surface_nll_nats":nll,"surface_probability":(-nll).exp()})
        })
        .collect();
    let greedy_emitted = match greedy.action {
        Action::Generate(token) => Some(token),
        Action::Copy => selected,
        Action::Stop => None,
    };
    let greedy_surface_nll = greedy_emitted
        .map(|token| token_nll_offline(&model.output, features, selected, token))
        .transpose()?;
    let stop_nll = stop.action_ce_offline();
    let greedy_steps = &greedy.steps[..usize::from(greedy.len)];
    Ok(json!({
        "selected_token":selected,
        "features":features,
        "greedy_action":format!("{:?}",greedy.action),
        "greedy_emitted_token":greedy_emitted,
        "greedy_emitted_text":greedy_emitted.map(|v|tokenizer.decode(&[u32::from(v)])),
        "greedy_trace":greedy_steps,
        "greedy_surface_nll_nats":greedy_surface_nll,
        "greedy_is_surface_map":greedy_emitted == Some(best.0),
        "generate_42_action_nll_nats":generate_42.action_ce_offline(),
        "generate_42_surface_nll_nats":token_nll_offline(&model.output,features,selected,COLON_TOKEN)?,
        "generate_42_trace":&generate_42.steps[..usize::from(generate_42.len)],
        "generate_target_action_nll_nats":generate_target.action_ce_offline(),
        "target_surface_nll_nats":token_nll_offline(&model.output,features,selected,target)?,
        "stop_action_nll_nats":stop_nll,
        "stop_probability":(-stop_nll).exp(),
        "surface_probability_sum":surface_probability_sum,
        "stop_plus_surface_probability_sum":(-stop_nll).exp()+surface_probability_sum,
        "copy_action_nll_nats":copy.as_ref().map(OutputTrace::action_ce_offline),
        "copy_probability":copy.as_ref().map(|v|(-v.action_ce_offline()).exp()),
        "surface_map_token":best.0,
        "surface_map_nll_nats":best.1,
        "surface_map_probability":(-best.1).exp(),
        "stop_exceeds_best_token":stop_nll < best.1,
        "top_surface_tokens":top_four,
        "scan_scope":"offline exhaustive 4096-token diagnostic; not serving computation"
    }))
}

fn prefill(
    model: &IntegratedModel,
    episode: &TrainingEpisode,
    reads: bool,
) -> AnyResult<(Session, BTreeMap<u64, u64>)> {
    let prompt = episode.prompt_len.ok_or("missing prompt length")?;
    let mut session = Session::new(model.runtime(), episode.source_id)?;
    let mut records = BTreeMap::new();
    for position in 0..prompt {
        let read = session.read(model.runtime(), reads)?;
        let observation = session.observe(
            model.runtime(),
            u32::from(episode.tokens[position]),
            &episode.token_bytes[position],
            read.selected_token(),
        )?;
        if let (Some(source), Some(record)) =
            (observation.indexed_source_event_id, observation.record_id)
        {
            records.insert(source, record);
        }
    }
    Ok((session, records))
}

fn inspect_episode(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    episode: &TrainingEpisode,
) -> AnyResult<Value> {
    let prompt = episode.prompt_len.ok_or("missing prompt length")?;
    let target = *episode.tokens.get(prompt).ok_or("missing first target")?;
    let source_position = episode.source_targets[prompt].ok_or("missing first source")?;
    let source_event = source_position as u64 + 1;
    let source_token = episode.tokens[source_position];
    let (actual_session, records) = prefill(model, episode, true)?;
    let read = actual_session.read(model.runtime(), true)?;
    let source_record_id = records.get(&source_event).copied();
    let source_code = source_record_id
        .map(|id| actual_session.memory.record(id).map(|record| record.code()))
        .transpose()?;
    let source_coarse = source_code.as_ref().map(|code| code.as_slice()[0]);
    let lane = model.encoder.score_lane(read.input, 0)?;
    if lane.best != read.query[0] {
        return Err("query lane score disagrees with served coarse code".into());
    }
    let mut coarse_order: Vec<_> = (0..120usize).collect();
    coarse_order.sort_by(|&a, &b| lane.scores[b].cmp(&lane.scores[a]).then_with(|| a.cmp(&b)));
    let positive_rank = source_coarse.and_then(|code| {
        coarse_order
            .iter()
            .position(|&candidate| candidate == usize::from(code))
            .map(|rank| rank + 1)
    });
    let top_coarse: Vec<_> = coarse_order
        .iter()
        .take(4)
        .map(|&code| json!({"code":code,"score":lane.scores[code]}))
        .collect();
    let candidates = &read.candidates[..read.candidate_count];
    let source_admitted = source_record_id
        .is_some_and(|id| candidates.iter().any(|candidate| candidate.record_id == id));
    let source_ranked = source_record_id.is_some_and(|id| {
        read.ranked
            .and_then(|index| candidates.get(index))
            .is_some_and(|candidate| candidate.record_id == id)
    });
    let source_selected = source_record_id.is_some_and(|id| {
        read.selected
            .and_then(|index| candidates.get(index))
            .is_some_and(|candidate| candidate.record_id == id)
    });
    let source_relative = source_record_id.and_then(|id| {
        candidates
            .iter()
            .find(|candidate| candidate.record_id == id)
            .map(|candidate| &candidate.relative[..model.config.lanes])
    });
    let source_relative_collisions = source_relative.map(|relative| {
        candidates
            .iter()
            .filter(|candidate| {
                Some(candidate.record_id) != source_record_id
                    && &candidate.relative[..model.config.lanes] == relative
            })
            .count()
    });
    let admitted_candidates: Vec<_> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            json!({
                "candidate_index":index,
                "record_id":candidate.record_id,
                "source_event_id":candidate.source_event_id,
                "token":candidate.token,
                "token_text":tokenizer.decode(&[u32::from(candidate.token)]),
                "key":&candidate.key[..model.config.lanes],
                "relative":&candidate.relative[..model.config.lanes],
                "energy":candidate.energy,
                "positive_match":Some(candidate.record_id)==source_record_id,
                "ranked":read.ranked==Some(index),
                "selected":read.selected==Some(index)
            })
        })
        .collect();
    let ranked_candidate = read.ranked.and_then(|index| candidates.get(index));
    let ranked_gate = ranked_candidate
        .map(|candidate| -> AnyResult<Value> {
            let gate_features = features(
                &model.config,
                actual_session.last_token,
                &actual_session.state,
                Some(candidate.token),
            );
            let ids = gate_features.as_slice();
            let mut reads = GateReadCounts::default();
            let score = model.read_gate.score(ids, &mut reads)?;
            let rows: Vec<_> = ids
                .iter()
                .map(|&id| -> AnyResult<Value> {
                    Ok(json!({"feature":id,"weight":model.read_gate.get_weight(id)?}))
                })
                .collect::<AnyResult<_>>()?;
            Ok(json!({
                "score":score,
                "threshold":0,
                "enabled":score>=0,
                "bias":model.read_gate.bias(),
                "features_and_weights":rows,
                "selected_token":candidate.token,
                "selected_record_id":candidate.record_id,
                "coefficient_reads":reads.coefficients
            }))
        })
        .transpose()?;
    let (no_read_session, _) = prefill(model, episode, false)?;
    let no_read_trace = no_read_session.read(model.runtime(), false)?;
    Ok(json!({
        "episode":episode.name,
        "source_id":episode.source_id,
        "prompt_tokens":prompt,
        "target_token":target,
        "target_text":tokenizer.decode(&[u32::from(target)]),
        "source_position":source_position,
        "source_event":source_event,
        "source_token":source_token,
        "source_token_text":tokenizer.decode(&[u32::from(source_token)]),
        "source_record_id":source_record_id,
        "source_coarse_code":source_coarse,
        "query_coarse_code":read.query[0],
        "query_coarse_top_score":lane.best_score,
        "query_coarse_margin":lane.margin,
        "positive_coarse_score":source_coarse.map(|code|lane.scores[usize::from(code)]),
        "positive_coarse_rank":positive_rank,
        "top_four_coarse":top_coarse,
        "candidate_count":read.candidate_count,
        "source_admitted":source_admitted,
        "source_ranked":source_ranked,
        "source_selected":source_selected,
        "source_relative_collision_count":source_relative_collisions,
        "admitted_candidates":admitted_candidates,
        "ranked_gate":ranked_gate,
        "gate_open":read.gate_enabled,
        "actual":head_probe(model,tokenizer,&actual_session,read.selected_token(),target)?,
        "ranked_candidate_head":ranked_candidate
            .map(|candidate|head_probe(model,tokenizer,&actual_session,Some(candidate.token),target))
            .transpose()?,
        "forced_source":head_probe(model,tokenizer,&actual_session,Some(source_token),target)?,
        "no_read":head_probe(model,tokenizer,&no_read_session,no_read_trace.selected_token(),target)?
    }))
}

fn address_bank_delta(model: &IntegratedModel) -> AnyResult<Value> {
    let mut fresh = CodeEncoder::new(
        model.config.vocabulary,
        model.config.lanes,
        model.encoder.seed,
    )?;
    fresh.enable_context_addressing()?;
    let loaded = serde_json::to_value(&model.encoder)?;
    let initial = serde_json::to_value(&fresh)?;
    let loaded_coefficients = loaded
        .get("address_coefficients")
        .and_then(Value::as_array)
        .ok_or("loaded address bank absent")?;
    let initial_coefficients = initial
        .get("address_coefficients")
        .and_then(Value::as_array)
        .ok_or("fresh address bank absent")?;
    if loaded_coefficients.len() != initial_coefficients.len() {
        return Err("loaded/fresh address bank shape mismatch".into());
    }
    let mut changed = vec![0u64; model.config.lanes];
    for (index, (trained, base)) in loaded_coefficients
        .iter()
        .zip(initial_coefficients)
        .enumerate()
    {
        if trained != base {
            changed[(index / 128) % model.config.lanes] += 1;
        }
    }
    let total_changed: u64 = changed.iter().sum();
    Ok(json!({"total_coefficients":loaded_coefficients.len(),
        "changed_coefficients":total_changed,"changed_by_lane":changed,
        "method":"compare hard i8 address bank against fresh deterministic bank at artifact seed"}))
}

fn execute(a: &Args) -> AnyResult<()> {
    let executable = std::env::current_exe()?;
    let inspector_source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if inspector_source == "UNBOUND" {
        return Err("inspector build source is unbound".into());
    }
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    if sha(&tokenizer_bytes) != TOKENIZER_SHA {
        return Err("pinned tokenizer digest mismatch".into());
    }
    let tokenizer =
        HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let data = integrated_attention_a2_data::load_development_data(&a.repo, &tokenizer)?;
    let data_hash = blake3::hash(&serde_json::to_vec(&data.manifest)?)
        .to_hex()
        .to_string();
    let artifact_bytes = fs::read(&a.model)?;
    let model = IntegratedModel::from_bytes(&artifact_bytes)?;
    if model.version != 2
        || model.config.vocabulary != 4096
        || (model.binding.source_commit != A2_MODEL_SOURCE
            && model.binding.source_commit != A2_CORRECTED_MODEL_SOURCE
            && model.binding.source_commit != A3_MODEL_SOURCE
            && model.binding.source_commit != inspector_source)
        || model.binding.tokenizer_sha256 != TOKENIZER_SHA
        || model.binding.training_data_blake3 != data_hash
    {
        return Err("loaded A2/A3 artifact source/tokenizer/data/version binding mismatch".into());
    }
    let bank = address_bank_delta(&model)?;
    let correction_episodes: Vec<_> = data.dev.iter().filter(|e| e.prompt_len.is_some()).collect();
    if correction_episodes.len() != 26 {
        return Err("expected 24 new and 2 inherited correction episodes".into());
    }
    let mut rows = Vec::with_capacity(correction_episodes.len());
    for episode in correction_episodes {
        rows.push(inspect_episode(&model, &tokenizer, episode)?);
    }
    let result = json!({
        "schema":"uor-r4.integrated-attention-a2-a3-inspect/2",
        "scope":"read-only loaded A2/A3 first-answer diagnostics on 24 new and 2 inherited exposed development prompts",
        "model_path":a.model,
        "model_sha256":sha(&artifact_bytes),
        "model_payload_blake3":hex::encode(model.digest()),
        "model_source_commit":model.binding.source_commit,
        "inspector_build_source_commit":inspector_source,
        "inspector_executable_sha256":sha(&fs::read(executable)?),
        "tokenizer_sha256":TOKENIZER_SHA,
        "training_data_blake3":data_hash,
        "address_bank_change":bank,
        "episodes":rows,
        "interpretation_limit":"full vocabulary scans and forced sources are offline diagnostics; only actual read and greedy path are served decisions"
    });
    write_json(&a.out.join("result.json"), &result)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema":"uor-r4.integrated-attention-a2-a3-inspect/2",
            "model_sha256":sha(&artifact_bytes),"episodes":26,
            "address_bank_change":result["address_bank_change"]
        }))?
    );
    Ok(())
}

fn run() -> AnyResult<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let result = execute(&a);
    if let Err(error) = &result {
        write_json(
            &a.out.join("failure.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
