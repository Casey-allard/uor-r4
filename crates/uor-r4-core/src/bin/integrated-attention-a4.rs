//! Matched parent-continuation driver for candidate-or-NoRead and evidence language learning.
//! Offline likelihood and fitting use floating point. Session decisions do not.
#![recursion_limit = "256"]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::{
    encoder::CodeEncoder,
    features,
    geometry::AlgebraKind,
    output::Action,
    read_action::ReadActionReadCounts,
    training::{
        evaluate_episode, token_nll_offline, A4FitConfig, FitConfig, FitMetrics, JointTrainer,
        TrainingEpisode,
    },
    AccessCounts, ArtifactBinding, IntegratedModel, Session, SessionSnapshot,
};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

#[path = "support/integrated_attention_a2_data.rs"]
mod integrated_attention_a2_data;
#[path = "support/integrated_attention_a3_routes.rs"]
mod integrated_attention_a3_routes;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const MAX_NEW: usize = 32;
const PARENT_SOURCE: &str = "626421e739e9aa4572496d3d9287ace7034e7483";
const C120_PARENT_SHA: &str = "dea0356528763649b8d8212212ac718f33d71e4d64631063b81e78d7728f3faf";
const GEOMETRIC_PARENT_SHA: &str =
    "53675902636cdbbd5d59bfffc93a9a1a77953bb490ec9018d38778d0ad11c1a6";
const CREDIT_PREFIX: &str = "A4 integrated candidate-or-NoRead with balanced answer head; ";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FitMode {
    A4,
    Control,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FitProvenance {
    mode: FitMode,
    parent_artifact_sha256: String,
    parent_payload_blake3: String,
    parent_source_commit: String,
    parent_model_version: u16,
    parent_training_updates: u64,
    parent_coarse_address_sha256: String,
    fit_configuration: FitConfig,
    a4_configuration: A4FitConfig,
    requested_epochs: usize,
    new_update_limit: u64,
    joint_seconds_limit: u64,
}

fn parent_sha(kind: AlgebraKind) -> &'static str {
    match kind {
        AlgebraKind::Cyclic120 => C120_PARENT_SHA,
        AlgebraKind::BinaryIcosahedral => GEOMETRIC_PARENT_SHA,
    }
}

fn bank_lanes(model: &CodeEncoder) -> Vec<Vec<i8>> {
    let lanes = usize::from(model.lanes);
    let mut out = vec![Vec::new(); lanes];
    for (row, coefficients) in model.address_coefficients().chunks_exact(128).enumerate() {
        out[row % lanes].extend_from_slice(coefficients);
    }
    out
}

fn bank_digests(model: &CodeEncoder) -> Value {
    json!(bank_lanes(model)
        .iter()
        .map(|lane| sha(&lane.iter().map(|&code| code as u8).collect::<Vec<_>>()))
        .collect::<Vec<_>>())
}

fn coarse_digest(model: &CodeEncoder) -> AnyResult<String> {
    let lanes = bank_lanes(model);
    let coarse = lanes.first().ok_or("missing coarse address bank")?;
    Ok(sha(&coarse
        .iter()
        .map(|&code| code as u8)
        .collect::<Vec<_>>()))
}

fn changed_per_lane(before: &CodeEncoder, after: &CodeEncoder) -> AnyResult<Vec<usize>> {
    let old = bank_lanes(before);
    let new = bank_lanes(after);
    if old.len() != new.len() || old.iter().zip(&new).any(|(a, b)| a.len() != b.len()) {
        return Err("address bank shape changed".into());
    }
    Ok(old
        .iter()
        .zip(&new)
        .map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count())
        .collect())
}

fn route_rows(cases: &[integrated_attention_a3_routes::RouteCase]) -> Vec<Value> {
    cases
        .iter()
        .map(|case| {
            json!({
                "episode":case.episode_name,
                "split":case.split,
                "entity":case.entity,
                "style":case.style,
                "polarity":case.polarity,
                "pair_id":case.pair_id,
                "source_id":case.source_id,
                "prompt_len":case.prompt_len,
                "target_first":case.target_first,
                "committed_keys":case.committed.len(),
                "explicit_negative_indices":case.negative_indices,
                "positive_committed_index":case.positive_committed_index,
                "frozen_prefix_blake3":case.frozen_prefix_blake3,
                "frozen_address_context_blake3":case.frozen_address_context_blake3,
                "source_manifest_sha256":case.source_manifest_sha256,
                "hard":case.hard,
            })
        })
        .collect()
}

fn check_route_contexts(
    before: &[integrated_attention_a3_routes::RouteCase],
    after: &[integrated_attention_a3_routes::RouteCase],
) -> AnyResult<()> {
    if before.len() != after.len()
        || before.iter().zip(after).any(|(a, b)| {
            a.episode_name != b.episode_name
                || a.source_id != b.source_id
                || a.frozen_prefix_blake3 != b.frozen_prefix_blake3
                || a.frozen_address_context_blake3 != b.frozen_address_context_blake3
        })
    {
        return Err("A4 route cases changed causal raw context or source identity".into());
    }
    Ok(())
}

fn route_summary(cases: &[integrated_attention_a3_routes::RouteCase]) -> Value {
    let authored: Vec<_> = cases
        .iter()
        .filter(|case| case.style.is_some())
        .cloned()
        .collect();
    let inherited: Vec<_> = cases
        .iter()
        .filter(|case| case.style.is_none())
        .cloned()
        .collect();
    json!({
        "authored":integrated_attention_a3_routes::summarize(&authored),
        "inherited_exposed":integrated_attention_a3_routes::summarize(&inherited),
    })
}

struct Args {
    command: String,
    repo: PathBuf,
    tokenizer: PathBuf,
    out: PathBuf,
    mode: Option<FitMode>,
    parent: Option<PathBuf>,
    model: Option<PathBuf>,
    epochs: usize,
    max_updates: u64,
    max_seconds: u64,
}

fn args() -> AnyResult<Args> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() == 9 && a[0] == "fit" {
        let mode = match a[5].as_str() {
            "a4" => FitMode::A4,
            "control" => FitMode::Control,
            _ => return Err("fit mode must be a4 or control".into()),
        };
        let epochs: usize = a[6].parse()?;
        let max_updates: u64 = a[7].parse()?;
        let max_seconds: u64 = a[8].parse()?;
        if !(1..=8).contains(&epochs)
            || !(1..=1_500_000).contains(&max_updates)
            || !(1..=900).contains(&max_seconds)
        {
            return Err("fit bounds: epochs 1..8, new updates 1..1500000, seconds 1..900".into());
        }
        return Ok(Args {
            command: a[0].clone(),
            repo: a[1].clone().into(),
            tokenizer: a[2].clone().into(),
            out: a[4].clone().into(),
            mode: Some(mode),
            parent: Some(a[3].clone().into()),
            model: None,
            epochs,
            max_updates,
            max_seconds,
        });
    }
    if a.len() == 5 && a[0] == "replay" {
        return Ok(Args {
            command: a[0].clone(),
            repo: a[1].clone().into(),
            tokenizer: a[2].clone().into(),
            out: a[3].clone().into(),
            mode: None,
            parent: None,
            model: Some(a[4].clone().into()),
            epochs: 0,
            max_updates: 0,
            max_seconds: 0,
        });
    }
    Err("usage: integrated-attention-a4 fit REPO TOKENIZER PARENT_MODEL OUT {a4|control} EPOCHS MAX_UPDATES MAX_SECONDS\n       integrated-attention-a4 replay REPO TOKENIZER OUT MODEL".into())
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> AnyResult<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn component_digest(value: &impl serde::Serialize) -> AnyResult<String> {
    let mut bytes = Vec::new();
    ciborium::into_writer(value, &mut bytes)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn component_digests(model: &IntegratedModel) -> AnyResult<Value> {
    Ok(
        json!({"encoder":component_digest(&model.encoder)?,"energy":component_digest(&model.energy)?,
        "output":component_digest(&model.output)?,"read_gate":component_digest(&model.read_gate)?,
        "write_gate":component_digest(&model.write_gate)?,
        "read_action":component_digest(&model.read_action)?,
        "read_action_parameter_stats":selector_stats(model)?}),
    )
}

fn selector_stats(model: &IntegratedModel) -> AnyResult<Value> {
    Ok(json!(model
        .read_action
        .as_ref()
        .map(|action| action.parameter_stats())
        .transpose()?))
}

fn prefill(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    ids: &[u32],
    source: u64,
    reads: bool,
) -> AnyResult<(Session, AccessCounts)> {
    let mut session = Session::new(model.runtime(), source)?;
    let mut access = AccessCounts::default();
    for &id in ids {
        let read = session.read(model.runtime(), reads)?;
        access.add(read.access);
        let observed = session.observe(
            model.runtime(),
            id,
            &tokenizer.decode_bytes(&[id]),
            read.selected_token(),
        )?;
        access.add(observed.access);
    }
    Ok((session, access))
}

fn decode(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    mut session: Session,
    reads: bool,
) -> AnyResult<Value> {
    let mut tokens = Vec::<u32>::new();
    let mut steps = Vec::new();
    let mut access = AccessCounts::default();
    for step in 0..MAX_NEW {
        let read = session.read(model.runtime(), reads)?;
        let output = session.output(model.runtime(), &read, None)?;
        access.add(read.access);
        access.output_coefficients += u64::from(output.coefficient_reads);
        let selected = read.selected_candidate();
        let token = match output.action {
            Action::Generate(t) => Some(u32::from(t)),
            Action::Copy => Some(u32::from(
                read.selected_token().ok_or("Copy without a source")?,
            )),
            Action::Stop => None,
        };
        steps.push(json!({"step":step,"action":format!("{:?}",output.action),"token":token,
            "candidate_count":read.candidate_count,"search_incomplete":read.search_incomplete,
            "selected_source_event":selected.map(|x|x.source_event_id),
            "selected_record":selected.map(|x|x.record_id),"selected_energy":selected.map(|x|x.energy),
            "selected_action_score":selected.and_then(|x|x.action_score),"no_read_score":read.no_read_score,
            "selected_token":read.selected_token(),"state":&session.state[..model.config.lanes]}));
        let Some(token) = token else {
            break;
        };
        tokens.push(token);
        let event = session.observe(
            model.runtime(),
            token,
            &tokenizer.decode_bytes(&[token]),
            read.selected_token(),
        )?;
        access.add(event.access);
    }
    Ok(
        json!({"text":tokenizer.decode(&tokens),"tokens":tokens,"steps":steps,
        "decoder":"greedy binary branches; not global vocabulary argmax",
        "served_access":access,"served_coefficient_reads":access.learned_coefficient_reads(),
        "raw_events_retained":session.memory.retained_event_count(),
        "generated_token_limit":MAX_NEW}),
    )
}

fn generate(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    name: &str,
    prompt: &str,
    source: u64,
    reads: bool,
) -> AnyResult<Value> {
    let ids = tokenizer.encode(prompt);
    let (session, access) = prefill(model, tokenizer, &ids, source, reads)?;
    let mut snapshot_bytes = Vec::new();
    ciborium::into_writer(&session.snapshot(), &mut snapshot_bytes)?;
    let restored_snapshot: SessionSnapshot = ciborium::from_reader(snapshot_bytes.as_slice())?;
    let restored = Session::restore(model.runtime(), restored_snapshot)?;
    let generated = decode(model, tokenizer, session, reads)?;
    let replay = decode(model, tokenizer, restored, reads)?;
    if generated != replay {
        return Err("loaded session snapshot changed generated behavior".into());
    }
    Ok(
        json!({"name":name,"prompt":prompt,"prompt_tokens":ids.len(),"reads_enabled":reads,
        "prefill_access":access,"snapshot_bytes":snapshot_bytes.len(),"snapshot_generation_identical":true,
        "generation":generated}),
    )
}

fn generation_panel(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    data: &integrated_attention_a2_data::DataSet,
) -> AnyResult<Vec<Value>> {
    let mut reports = Vec::new();
    let mut selected_episodes: Vec<&TrainingEpisode> = data
        .dev
        .iter()
        .filter(|e| {
            e.prompt_len.is_some()
                && e.name.ends_with(":positive")
                && (e.name.contains("style-0") || e.name.contains("style-2"))
        })
        .take(2)
        .collect();
    selected_episodes.extend(
        data.dev
            .iter()
            .filter(|e| {
                e.prompt_len.is_some()
                    && e.name.ends_with(":positive")
                    && (e.name.contains("style-4") || e.name.contains("style-5"))
            })
            .take(2),
    );
    selected_episodes.extend(data.dev.iter().filter(|e| {
        e.name == "dev:correction:orion-allowed" || e.name == "dev:correction:larch-denied"
    }));
    if selected_episodes.len() != 6 {
        return Err("missing authored or inherited generated endpoint".into());
    }
    for episode in selected_episodes {
        let edit = data
            .prompt_edit(episode)?
            .ok_or("missing counterfactual source edit")?;
        for (variant, raw, expected) in [
            (
                "original",
                edit.original.as_str(),
                edit.original_answer.as_str(),
            ),
            (
                "changed-source",
                edit.changed_source.as_str(),
                edit.changed_answer.as_str(),
            ),
        ] {
            for reads in [true, false] {
                let mut row = generate(
                    model,
                    tokenizer,
                    &format!("{}:{variant}", episode.name),
                    raw,
                    episode.source_id,
                    reads,
                )?;
                row["expected_answer"] = json!(expected);
                row["complete_answer_equal"] = json!(row["generation"]["text"]
                    .as_str()
                    .is_some_and(|text| text.trim() == expected.trim()));
                row["changed_byte_range"] = json!(edit.changed_byte_range);
                row["source_values"] = json!([edit.original_value, edit.changed_value]);
                reports.push(row);
            }
        }
    }
    // Natural continuation endpoints are reported separately from grounded curriculum.
    for extension in [".md", ".rs"] {
        let episode = data
            .dev
            .iter()
            .find(|e| e.prompt_len.is_none() && e.name.contains(extension))
            .ok_or("missing natural prose/code continuation")?;
        let ids: Vec<u32> = episode.tokens[..episode.tokens.len().min(96)]
            .iter()
            .map(|&x| u32::from(x))
            .collect();
        for reads in [true, false] {
            reports.push(generate(
                model,
                tokenizer,
                &episode.name,
                &tokenizer.decode(&ids),
                episode.source_id,
                reads,
            )?);
        }
    }
    Ok(reports)
}

// Read-only, one-step head probes on actual teacher-forced prefixes. Source annotations
// never enter Session::read/observe or the generated continuation path.
fn source_diagnosis(model: &IntegratedModel, dev: &[TrainingEpisode]) -> AnyResult<Vec<Value>> {
    let mut rows = Vec::new();
    for episode in dev.iter().filter(|e| e.prompt_len.is_some()) {
        let prompt = episode.prompt_len.ok_or("missing prompt")?;
        let mut session = Session::new(model.runtime(), episode.source_id)?;
        let mut records = std::collections::BTreeMap::new();
        for (position, &target) in episode.tokens.iter().enumerate() {
            let read = session.read(model.runtime(), true)?;
            if position >= prompt {
                if let Some(source_position) = episode.source_targets[position] {
                    let source_event = source_position as u64 + 1;
                    let record_id = records.get(&source_event).copied();
                    let source_token = episode.tokens[source_position];
                    let candidates = &read.candidates[..read.candidate_count];
                    let distinct: std::collections::BTreeSet<Vec<u8>> = candidates
                        .iter()
                        .map(|c| c.key[..model.config.lanes].to_vec())
                        .collect();
                    let score = |value| -> AnyResult<f64> {
                        Ok(token_nll_offline(
                            &model.output,
                            features(&model.config, session.last_token, &session.state, value)
                                .as_slice(),
                            value,
                            target,
                        )?)
                    };
                    // These repeated score calls explain the loaded action decision.
                    // Their diagnostic counters are separate from served Session costs.
                    let mut diagnostic_action_reads = ReadActionReadCounts::default();
                    let no_read_score_recomputed = if let Some(action) = &model.read_action {
                        Some(
                            action.score_none(
                                features(&model.config, session.last_token, &session.state, None)
                                    .as_slice(),
                                &mut diagnostic_action_reads,
                            )?,
                        )
                    } else {
                        None
                    };
                    if no_read_score_recomputed != read.no_read_score {
                        return Err("diagnostic NoRead score disagrees with served decision".into());
                    }
                    let candidate_actions = candidates
                        .iter()
                        .map(|candidate| -> AnyResult<Value> {
                            let components = if let Some(action) = &model.read_action {
                                Some(
                                    action.score_read_components(
                                        features(
                                            &model.config,
                                            session.last_token,
                                            &session.state,
                                            Some(candidate.token),
                                        )
                                        .as_slice(),
                                        &candidate.relative[..model.config.lanes],
                                        &mut diagnostic_action_reads,
                                    )?,
                                )
                            } else {
                                None
                            };
                            if components.map(|value| value.total) != candidate.action_score {
                                return Err(
                                    "diagnostic read score disagrees with served decision".into()
                                );
                            }
                            Ok(json!({
                                "record_id":candidate.record_id,
                                "source_event_id":candidate.source_event_id,
                                "token":candidate.token,
                                "relative_code":&candidate.relative[..model.config.lanes],
                                "legacy_energy":candidate.energy,
                                "action_score":candidate.action_score,
                                "score_components":components,
                            }))
                        })
                        .collect::<AnyResult<Vec<_>>>()?;
                    rows.push(json!({"episode":episode.name,"position":position,"answer_position":position-prompt,
                        "first_answer_decision":position==prompt,
                        "target_token":target,"source_token":source_token,"source_event":source_event,
                        "source_committed":record_id.is_some(),"candidate_count":read.candidate_count,
                        "distinct_candidate_codes":distinct.len(),"source_admitted":record_id.is_some_and(|id|candidates.iter().any(|c|c.record_id==id)),
                        "source_ranked":read.ranked.is_some_and(|i|Some(candidates[i].record_id)==record_id),
                        "source_selected":read.selected.is_some_and(|i|Some(candidates[i].record_id)==record_id),
                        "gate_open":read.gate_enabled,"energy_min":candidates.iter().map(|c|c.energy).min(),
                        "energy_max":candidates.iter().map(|c|c.energy).max(),"query_code":&read.query[..model.config.lanes],
                        "no_read_score":read.no_read_score,
                        "no_read_score_recomputed":no_read_score_recomputed,
                        "offline_component_probe_reads":diagnostic_action_reads,
                        "selected_action_score":read.selected_candidate().and_then(|c|c.action_score),
                        "source_action_score":candidates.iter().find(|c|Some(c.record_id)==record_id).and_then(|c|c.action_score),
                        "candidate_actions":candidate_actions,
                        "legacy_energy_scope":"legacy energy is retained as a diagnostic in A4 and its reads remain included in Session access counts; A4 selection uses action_score; control retains legacy energy plus gate",
                        "source_code":record_id.map(|id|session.memory.record(id).map(|r|r.code().as_slice().to_vec())).transpose()?,
                        "actual_nll":score(read.selected_token())?,"no_read_nll":score(None)?,"forced_source_nll":score(Some(source_token))?}));
                }
            }
            let event = session.observe(
                model.runtime(),
                u32::from(target),
                &episode.token_bytes[position],
                read.selected_token(),
            )?;
            if let (Some(source), Some(record)) = (event.indexed_source_event_id, event.record_id) {
                records.insert(source, record);
            }
        }
    }
    Ok(rows)
}

fn execute(a: &Args) -> AnyResult<()> {
    let started = Instant::now();
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
    write_json(&a.out.join("data.json"), &data.manifest)?;
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if source == "UNBOUND" {
        return Err("build with UOR_BUILD_SOURCE_COMMIT set to the source commit".into());
    }
    let executable = std::env::current_exe()?;
    let executable_sha = sha(&fs::read(&executable)?);
    let fit_config = FitConfig::default();
    let a4_config = A4FitConfig {
        answers_head_repeats: 8,
        natural_selector_stride: 256,
        selector_margin: 2,
        ..A4FitConfig::default()
    };
    let model_path;
    let mut fit_total = FitMetrics::default();
    let mut stop_reason = "replay";
    let mut fit_elapsed_ms = None::<u128>;
    let mut bank_deltas = None::<Value>;
    let mut initial_selector_stats = None::<Value>;
    let mut before_fit_routes = None;
    let mut before_dev_routes = None;
    if let Some(mode) = a.mode {
        let parent_path = a.parent.as_ref().ok_or("fit parent missing")?;
        let parent_bytes = fs::read(parent_path)?;
        let parent_digest = sha(&parent_bytes);
        let mut model = IntegratedModel::from_bytes(&parent_bytes)?;
        if parent_digest != parent_sha(model.algebra.kind())
            || model.binding.source_commit != PARENT_SOURCE
            || model.binding.tokenizer_sha256 != TOKENIZER_SHA
            || model.binding.training_data_blake3 != data_hash
            || model.version != 2
            || !model.encoder.context_addressing
            || model.read_action.is_some()
            || !model
                .binding
                .credit_assignment
                .starts_with("A3 staged hard integer coarse routing;")
        {
            return Err(
                "A4 requires the pinned loaded A3 parent, tokenizer and data lineage".into(),
            );
        }
        let parent_updates = model.binding.training_updates;
        let provenance = FitProvenance {
            mode,
            parent_artifact_sha256: parent_digest,
            parent_payload_blake3: hex::encode(model.digest()),
            parent_source_commit: model.binding.source_commit.clone(),
            parent_model_version: model.version,
            parent_training_updates: parent_updates,
            parent_coarse_address_sha256: coarse_digest(&model.encoder)?,
            fit_configuration: fit_config,
            a4_configuration: a4_config.clone(),
            requested_epochs: a.epochs,
            new_update_limit: a.max_updates,
            joint_seconds_limit: a.max_seconds,
        };
        write_json(&a.out.join("parent-provenance.json"), &provenance)?;
        write_json(
            &a.out.join("parent-component-digests.json"),
            &component_digests(&model)?,
        )?;
        drop(parent_bytes);
        if mode == FitMode::A4 {
            model.enable_read_actions()?;
        }
        initial_selector_stats = Some(selector_stats(&model)?);
        eprintln!(
            "mode={mode:?} algebra={:?} parameters={} bytes; parent updates={parent_updates}; fit episodes={}, dev episodes={}",
            model.algebra.kind(),
            model.parameter_bytes(),
            data.fit.len(),
            data.dev.len()
        );
        write_json(
            &a.out.join("initial-component-digests.json"),
            &component_digests(&model)?,
        )?;
        let initial_encoder = model.encoder.clone();
        let initial_fit = integrated_attention_a3_routes::collect_cases(&model, &data.fit, &data)?;
        let initial_dev = integrated_attention_a3_routes::collect_cases(&model, &data.dev, &data)?;
        write_json(
            &a.out.join("routes-before-fit.json"),
            &route_rows(&initial_fit),
        )?;
        write_json(
            &a.out.join("routes-before-dev.json"),
            &route_rows(&initial_dev),
        )?;
        before_fit_routes = Some(initial_fit);
        before_dev_routes = Some(initial_dev);
        // The provenance is embedded in the artifact for independent-process replay.
        // Parent training updates remain cumulative; all A4 limits count new tokens only.
        model.binding = ArtifactBinding {
            source_commit: source.to_owned(),
            tokenizer_sha256: TOKENIZER_SHA.into(),
            training_data_blake3: data_hash.clone(),
            training_seed: model.binding.training_seed,
            training_updates: parent_updates,
            credit_assignment: format!("{CREDIT_PREFIX}{}", serde_json::to_string(&provenance)?),
        };
        let mut trainer = JointTrainer::from_model(model)?;
        trainer.freeze_context_address_learning(true);
        trainer.configure_a4(a4_config)?;
        let mut progress = fs::File::create_new(a.out.join("fit.jsonl"))?;
        stop_reason = "requested epochs completed";
        let joint_started = Instant::now();
        'epochs: for epoch in 0..a.epochs {
            for (index, episode) in data.fit.iter().enumerate() {
                let new_updates = trainer
                    .binding
                    .training_updates
                    .checked_sub(parent_updates)
                    .ok_or("parent training count decreased")?;
                let prospective = new_updates
                    .checked_add(episode.tokens.len() as u64)
                    .ok_or("new training count overflow")?;
                if prospective > a.max_updates {
                    stop_reason = "prospective new-update cap";
                    break 'epochs;
                }
                if joint_started.elapsed().as_secs() >= a.max_seconds {
                    stop_reason = "wall cap at episode boundary";
                    break 'epochs;
                }
                let metrics = trainer.fit_episode(episode, fit_config)?;
                fit_total.add(&metrics);
                serde_json::to_writer(
                    &mut progress,
                    &json!({"epoch":epoch,"episode":episode.name,
                    "parent_training_updates":parent_updates,
                    "new_training_updates":trainer.binding.training_updates-parent_updates,
                    "cumulative_training_updates":trainer.binding.training_updates,
                    "metrics":metrics,"elapsed_ms":joint_started.elapsed().as_millis()}),
                )?;
                progress.write_all(b"\n")?;
                if index % 8 == 0 || index + 1 == data.fit.len() {
                    eprintln!(
                        "epoch={} episode={}/{} new updates={} cumulative training bpt={:.4}",
                        epoch + 1,
                        index + 1,
                        data.fit.len(),
                        fit_total.positions,
                        fit_total.bits_per_token().unwrap_or(f64::NAN)
                    );
                }
            }
        }
        fit_elapsed_ms = Some(joint_started.elapsed().as_millis());
        let completed_updates = trainer
            .binding
            .training_updates
            .checked_sub(parent_updates)
            .ok_or("parent training count decreased")?;
        if completed_updates == 0 {
            return Err(
                "budget ended before a new training episode completed; no fitted artifact exported"
                    .into(),
            );
        }
        if completed_updates != fit_total.positions || completed_updates > a.max_updates {
            return Err("new training count disagrees with executed metrics or limit".into());
        }
        let model = trainer.export();
        let final_deltas = changed_per_lane(&initial_encoder, &model.encoder)?;
        if final_deltas.first().copied() != Some(0)
            || coarse_digest(&model.encoder)? != provenance.parent_coarse_address_sha256
        {
            return Err(
                "continuation fit changed frozen parent coarse address coefficients".into(),
            );
        }
        bank_deltas = Some(json!({
            "parent_to_joint_changed_per_lane":final_deltas,
            "before_joint":bank_digests(&initial_encoder),
            "after_joint":bank_digests(&model.encoder),
            "parent_coarse_preserved":true,
        }));
        write_json(
            &a.out.join("exported-component-digests.json"),
            &component_digests(&model)?,
        )?;
        model_path = a.out.join("model.ia4");
        let bytes = model.to_bytes()?;
        let mut file = fs::File::create_new(&model_path)?;
        file.write_all(&bytes)?;
        drop(file);
        drop(model);
    } else {
        model_path = a.model.clone().ok_or("replay model missing")?;
    }
    let artifact_bytes = fs::read(&model_path)?;
    let model = IntegratedModel::from_bytes(&artifact_bytes)?;
    let provenance: FitProvenance = serde_json::from_str(
        model
            .binding
            .credit_assignment
            .strip_prefix(CREDIT_PREFIX)
            .ok_or("loaded model is not bound to an A4 continuation")?,
    )?;
    let expected_version = if provenance.mode == FitMode::A4 { 3 } else { 2 };
    let new_updates = model
        .binding
        .training_updates
        .checked_sub(provenance.parent_training_updates)
        .ok_or("loaded training count precedes parent")?;
    if model.binding.tokenizer_sha256 != TOKENIZER_SHA
        || model.binding.training_data_blake3 != data_hash
        || model.binding.source_commit != source
        || model.version != expected_version
        || model.read_action.is_some() != (provenance.mode == FitMode::A4)
        || !model.encoder.context_addressing
        || provenance.parent_artifact_sha256 != parent_sha(model.algebra.kind())
        || provenance.parent_source_commit != PARENT_SOURCE
        || provenance.parent_model_version != 2
        || new_updates == 0
        || new_updates > provenance.new_update_limit
        || coarse_digest(&model.encoder)? != provenance.parent_coarse_address_sha256
    {
        return Err(
            "loaded A4 model source/tokenizer/data/parent/mode/count binding mismatch".into(),
        );
    }
    // Execute actual loaded generation before wider development scoring.
    let first = data
        .dev
        .iter()
        .find(|e| e.prompt_len.is_some())
        .ok_or("missing first loaded generation")?;
    let n = first.prompt_len.ok_or("missing prompt")?;
    let prefix: Vec<u32> = first.tokens[..n].iter().map(|&x| u32::from(x)).collect();
    let smoke = generate(
        &model,
        &tokenizer,
        &first.name,
        &tokenizer.decode(&prefix),
        first.source_id,
        true,
    )?;
    eprintln!("first loaded output: {}", smoke["generation"]["text"]);
    write_json(&a.out.join("first-loaded-generation.json"), &smoke)?;
    write_json(
        &a.out.join("loaded-component-digests.json"),
        &component_digests(&model)?,
    )?;
    write_json(&a.out.join("loaded-provenance.json"), &provenance)?;

    let loaded_fit_routes =
        integrated_attention_a3_routes::collect_cases(&model, &data.fit, &data)?;
    let loaded_dev_routes =
        integrated_attention_a3_routes::collect_cases(&model, &data.dev, &data)?;
    if let Some(before) = &before_fit_routes {
        check_route_contexts(before, &loaded_fit_routes)?;
    }
    if let Some(before) = &before_dev_routes {
        check_route_contexts(before, &loaded_dev_routes)?;
    }
    write_json(
        &a.out.join("routes-loaded-fit.json"),
        &route_rows(&loaded_fit_routes),
    )?;
    write_json(
        &a.out.join("routes-loaded-dev.json"),
        &route_rows(&loaded_dev_routes),
    )?;
    write_json(
        &a.out.join("routes-loaded-summary.json"),
        &json!({
            "fit":route_summary(&loaded_fit_routes),
            "dev":route_summary(&loaded_dev_routes),
            "causal_prefix_and_address_context_hashes_in_each_case":true,
        }),
    )?;

    let mut rows = Vec::new();
    let mut with_read = FitMetrics::default();
    let mut no_read = FitMetrics::default();
    for episode in &data.dev {
        for reads in [true, false] {
            let metrics = evaluate_episode(&model, episode, reads)?;
            if reads {
                with_read.add(&metrics);
            } else {
                no_read.add(&metrics);
            }
            rows.push(json!({"episode":episode.name,"reads_enabled":reads,"bits_per_token":metrics.bits_per_token(),"metrics":metrics}));
        }
    }
    write_json(
        &a.out.join("source-diagnosis.json"),
        &source_diagnosis(&model, &data.dev)?,
    )?;
    let generations = generation_panel(&model, &tokenizer, &data)?;
    write_json(&a.out.join("generations.json"), &generations)?;
    write_json(
        &a.out.join("evaluation.json"),
        &json!({"scope":"same exposed source-separated development; likelihood scoring cost is not decoder cost",
        "with_read":with_read,"without_read":no_read,"with_read_bpt":with_read.bits_per_token(),
        "without_read_bpt":no_read.bits_per_token(),"episodes":rows}),
    )?;
    let result = json!({"schema":"uor-r4.integrated-attention-a4/1","command":a.command,
        "source_commit":source,"executable_sha256":executable_sha,"artifact_path":model_path,
        "artifact_sha256":sha(&artifact_bytes),"artifact_payload_blake3":hex::encode(model.digest()),
        "artifact_bytes":artifact_bytes.len(),"parameter_bytes":model.parameter_bytes(),
        "artifact_model_version":model.version,"mode":provenance.mode,
        "algebra":format!("{:?}",model.algebra.kind()),"binding":model.binding,
        "parent_provenance":provenance,
        "parent_training_updates":provenance.parent_training_updates,
        "new_training_updates":new_updates,"cumulative_training_updates":model.binding.training_updates,
        "fit_configuration":provenance.fit_configuration,"a4_configuration":provenance.a4_configuration,
        "fit_metrics":fit_total,"fit_stop_reason":stop_reason,
        "joint_fit_elapsed_ms":fit_elapsed_ms,"address_bank_delta_by_lane":bank_deltas,
        "initial_read_action_parameter_stats":initial_selector_stats,
        "loaded_read_action_parameter_stats":selector_stats(&model)?,
        "read_action_parameter_stats_scope":"offline all-row inspection of exported integer coefficients; not a serving scan or capability measurement",
        "loaded_coarse_address_sha256":coarse_digest(&model.encoder)?,
        "parent_coarse_preserved":true,
        "loaded_fit_route_summary":route_summary(&loaded_fit_routes),
        "loaded_dev_route_summary":route_summary(&loaded_dev_routes),
        "elapsed_ms":started.elapsed().as_millis(),"development_positions":with_read.positions,
        "training_time_limit_scope":"joint continuation bounded by the requested maximum of 900 seconds, checked before each episode; an in-progress episode may complete; parent updates excluded from new-token cap; preparation and evaluation charged separately in cycle ledger",
        "head_weighting":"one ordinary prefix/body update; eight total answer-head repetitions in both selector and control arms, including forced-source/NoRead auxiliaries and terminal Stop; each category of extra answer updates counted separately from input positions",
        "continuation_training_scope":"both arms preserve the dedicated A3 context-token address bank and use the same balanced head schedule; A4 learns integer candidate-or-NoRead and State, disables direct fine-code/legacy-energy/gate fitting, but State updates shared encoder rows and can change fine codes; control continues legacy direct fine-code, energy and gate fitting",
        "legacy_energy_access_scope":"A4 still evaluates the previous energy table as a diagnostic for each admitted candidate; those coefficient and packed-byte reads are included in Session access counts; candidate action scores determine A4 selection",
        "with_read_bpt":with_read.bits_per_token(),"without_read_bpt":no_read.bits_per_token(),
        "generation_rows":generations.len(),"snapshot_generation_parity":true,
        "measurement_scope":"integer session decisions, selected coefficient/table counters; no physical energy or machine-code audit; exact bounded raw history; one-token records; no semantic binder or multi-token copy qualification; long-session posting reconstruction after eviction is not qualified",
        "quality_status":"exposed development observation; inspect complete generated outputs and matched controls; no full Milestone A acceptance or geometric advantage inferred"});
    write_json(&a.out.join("result.json"), &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
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
