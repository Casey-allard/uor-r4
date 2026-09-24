//! Bounded open-development driver for hard page addressing and evidence language learning.
//! Offline likelihood and fitting use floating point. Session decisions do not.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::{
    encoder::{CodeEncoder, EncoderTrainer, HardAddressConfig},
    features,
    geometry::AlgebraKind,
    output::Action,
    training::{
        evaluate_episode, token_nll_offline, FitConfig, FitMetrics, JointTrainer, TrainingEpisode,
    },
    AccessCounts, ArtifactBinding, IntegratedModel, ModelConfig, Session, SessionSnapshot,
};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

#[path = "support/integrated_attention_a2_data.rs"]
mod integrated_attention_a2_data;
#[path = "support/integrated_attention_a3_routes.rs"]
mod integrated_attention_a3_routes;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const SEED: u64 = 0x4131_2026_0924;
const MAX_NEW: usize = 32;
const HARD_ROUTE_SECONDS: u64 = 300;

fn hard_config() -> HardAddressConfig {
    HardAddressConfig {
        margin: 2,
        page_capacity: 64,
        max_pairs: 96,
        max_keys_per_world: 256,
        max_candidate_evaluations: 5_000_000,
        max_proposals: 20_000,
        max_accepted_edits: 4_096,
        max_duration_ms: 240_000,
        max_sweeps: 3,
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
        return Err("A3 route cases changed causal raw context or source identity".into());
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
    kind: Option<AlgebraKind>,
    model: Option<PathBuf>,
    epochs: usize,
    max_updates: u64,
    max_seconds: u64,
}

fn args() -> AnyResult<Args> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() == 8 && a[0] == "fit" {
        let kind = match a[4].as_str() {
            "2i" => AlgebraKind::BinaryIcosahedral,
            "c120" => AlgebraKind::Cyclic120,
            _ => return Err("algebra must be 2i or c120".into()),
        };
        let epochs: usize = a[5].parse()?;
        let max_updates: u64 = a[6].parse()?;
        let max_seconds: u64 = a[7].parse()?;
        if !(1..=32).contains(&epochs)
            || !(1..=3_000_000).contains(&max_updates)
            || !(1..=900).contains(&max_seconds)
        {
            return Err("fit bounds: epochs 1..32, updates 1..3000000, seconds 1..900".into());
        }
        return Ok(Args {
            command: a[0].clone(),
            repo: a[1].clone().into(),
            tokenizer: a[2].clone().into(),
            out: a[3].clone().into(),
            kind: Some(kind),
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
            kind: None,
            model: Some(a[4].clone().into()),
            epochs: 0,
            max_updates: 0,
            max_seconds: 0,
        });
    }
    Err("usage: integrated-attention-a3 fit REPO TOKENIZER OUT {2i|c120} EPOCHS MAX_UPDATES MAX_SECONDS\n       integrated-attention-a3 replay REPO TOKENIZER OUT MODEL".into())
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
        "write_gate":component_digest(&model.write_gate)?}),
    )
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
                    rows.push(json!({"episode":episode.name,"position":position,"answer_position":position-prompt,
                        "target_token":target,"source_token":source_token,"source_event":source_event,
                        "source_committed":record_id.is_some(),"candidate_count":read.candidate_count,
                        "distinct_candidate_codes":distinct.len(),"source_admitted":record_id.is_some_and(|id|candidates.iter().any(|c|c.record_id==id)),
                        "source_ranked":read.ranked.is_some_and(|i|Some(candidates[i].record_id)==record_id),
                        "source_selected":read.selected.is_some_and(|i|Some(candidates[i].record_id)==record_id),
                        "gate_open":read.gate_enabled,"energy_min":candidates.iter().map(|c|c.energy).min(),
                        "energy_max":candidates.iter().map(|c|c.energy).max(),"query_code":&read.query[..model.config.lanes],
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
    let route_config = hard_config();
    let model_path;
    let mut fit_total = FitMetrics::default();
    let mut stop_reason = "replay";
    let mut hard_route_report = None::<Value>;
    let mut hard_route_elapsed_ms = None::<u128>;
    let mut fit_elapsed_ms = None::<u128>;
    let mut hard_route_input_blake3 = None::<String>;
    let mut bank_deltas = None::<Value>;
    if let Some(kind) = a.kind {
        let mut model = IntegratedModel::new(ModelConfig::pilot(4096, 4)?, kind, SEED)?;
        model.enable_context_addressing()?;
        eprintln!(
            "parameters={} bytes; fit episodes={}, dev episodes={}",
            model.parameter_bytes(),
            data.fit.len(),
            data.dev.len()
        );
        write_json(
            &a.out.join("initial-component-digests.json"),
            &component_digests(&model)?,
        )?;
        let initial_encoder = model.encoder.clone();
        let hard_started = Instant::now();
        let before_fit =
            integrated_attention_a3_routes::collect_fit_pairs(&model, &data.fit, &data)?;
        if before_fit.pairs.is_empty() {
            return Err("no causally committed A3 training pairs".into());
        }
        let before_dev = integrated_attention_a3_routes::collect_cases(&model, &data.dev, &data)?;
        let input_blake3 = blake3::hash(&serde_json::to_vec(&before_fit.cases)?)
            .to_hex()
            .to_string();
        hard_route_input_blake3 = Some(input_blake3.clone());
        write_json(
            &a.out.join("routes-before-fit.json"),
            &route_rows(&before_fit.cases),
        )?;
        write_json(
            &a.out.join("routes-before-dev.json"),
            &route_rows(&before_dev),
        )?;
        let mut hard_config = hard_config();
        // Collection and the post-optimization checks share the 300-second hard-routing stage.
        // Reserve at least the measured collection time and five seconds for the latter.
        let collection_ms = hard_started.elapsed().as_millis();
        let reserve_ms = collection_ms.saturating_add(5_000);
        let available_ms = (HARD_ROUTE_SECONDS as u128 * 1_000)
            .saturating_sub(collection_ms.saturating_add(reserve_ms));
        if available_ms == 0 {
            return Err("hard route collection consumed its 300-second stage cap".into());
        }
        hard_config.max_duration_ms = hard_config.max_duration_ms.min(available_ms as u64);
        let mut encoder = EncoderTrainer::from_model(model.encoder.clone())?;
        let hard = encoder.train_hard_address_batch(&before_fit.pairs, hard_config)?;
        model.encoder = encoder.export();
        let after_hard_encoder = model.encoder.clone();
        model.binding = ArtifactBinding {source_commit:source.to_owned(),tokenizer_sha256:TOKENIZER_SHA.into(),
            training_data_blake3:data_hash.clone(),training_seed:SEED,training_updates:0,
            credit_assignment:format!("A3 staged hard integer coarse routing; route_input_blake3={input_blake3}; route_config={}; fit_pairs={}; causal committed-source fit only; freeze coarse address lane during A2 joint fine/state/head/gate fitting; natural recurrence excluded from address supervision; local marginal token CE; fit_config={:?}; epochs={}; max_updates={}",
                serde_json::to_string(&hard_config)?, before_fit.pairs.len(), fit_config, a.epochs, a.max_updates)};
        write_json(
            &a.out.join("hard-route-optimizer.json"),
            &json!({
                "configuration":hard_config,
                "optimizer":hard,
                "input_blake3":input_blake3,
                "fit_pairs":before_fit.pairs.len(),
                "stage_elapsed_ms_at_checkpoint":hard_started.elapsed().as_millis(),
            }),
        )?;
        let checkpoint = model.to_bytes()?;
        let mut file = fs::File::create_new(a.out.join("hard-route-checkpoint.ia2"))?;
        file.write_all(&checkpoint)?;
        drop(file);
        let after_fit = integrated_attention_a3_routes::collect_cases(&model, &data.fit, &data)?;
        let after_dev = integrated_attention_a3_routes::collect_cases(&model, &data.dev, &data)?;
        check_route_contexts(&before_fit.cases, &after_fit)?;
        check_route_contexts(&before_dev, &after_dev)?;
        let hard_ms = hard_started.elapsed().as_millis();
        if hard_ms > HARD_ROUTE_SECONDS as u128 * 1_000 {
            return Err("hard route stage exceeded its 300-second cap".into());
        }
        hard_route_elapsed_ms = Some(hard_ms);
        write_json(
            &a.out.join("routes-after-hard-fit.json"),
            &route_rows(&after_fit),
        )?;
        write_json(
            &a.out.join("routes-after-hard-dev.json"),
            &route_rows(&after_dev),
        )?;
        let route_report = json!({
            "configuration":hard_config,
            "optimizer":hard,
            "fit_pairs":before_fit.pairs.len(),
            "fit_cases":before_fit.cases.len(),
            "dev_cases":before_dev.len(),
            "before_fit":route_summary(&before_fit.cases),
            "before_dev":route_summary(&before_dev),
            "after_hard_fit":route_summary(&after_fit),
            "after_hard_dev":route_summary(&after_dev),
            "skipped_pair_ids":before_fit.skipped_pair_ids,
            "skipped_pair_reasons":before_fit.skipped_pair_reasons,
            "entity_groups":before_fit.entity_groups,
            "input_blake3":input_blake3,
            "collection_ms":collection_ms,
            "stage_elapsed_ms":hard_ms,
            "address_bank_digests_before":bank_digests(&initial_encoder),
            "address_bank_digests_after":bank_digests(&after_hard_encoder),
            "address_bank_changed_per_lane":changed_per_lane(&initial_encoder,&after_hard_encoder)?,
        });
        write_json(&a.out.join("hard-route.json"), &route_report)?;
        hard_route_report = Some(route_report);
        let mut trainer = JointTrainer::from_model(model)?;
        trainer.freeze_context_address_learning(true);
        let mut progress = fs::File::create_new(a.out.join("fit.jsonl"))?;
        stop_reason = "requested epochs completed";
        let joint_started = Instant::now();
        'epochs: for epoch in 0..a.epochs {
            for (index, episode) in data.fit.iter().enumerate() {
                if trainer.binding.training_updates + episode.tokens.len() as u64 > a.max_updates {
                    stop_reason = "prospective update cap";
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
                    "metrics":metrics,"elapsed_ms":joint_started.elapsed().as_millis()}),
                )?;
                progress.write_all(b"\n")?;
                if index % 8 == 0 || index + 1 == data.fit.len() {
                    eprintln!(
                        "epoch={} episode={}/{} updates={} cumulative training bpt={:.4}",
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
        if trainer.binding.training_updates == 0 {
            return Err(
                "budget ended before a training episode completed; no fitted artifact exported"
                    .into(),
            );
        }
        let model = trainer.export();
        let final_deltas = changed_per_lane(&after_hard_encoder, &model.encoder)?;
        if final_deltas.first().copied() != Some(0) {
            return Err("joint fit changed frozen coarse address coefficients".into());
        }
        bank_deltas = Some(json!({
            "hard_to_joint_changed_per_lane":final_deltas,
            "before_hard":bank_digests(&initial_encoder),
            "after_hard":bank_digests(&after_hard_encoder),
            "after_joint":bank_digests(&model.encoder),
        }));
        model_path = a.out.join("model.ia2");
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
    write_json(
        &a.out.join("loaded-component-digests.json"),
        &component_digests(&model)?,
    )?;
    if model.binding.tokenizer_sha256 != TOKENIZER_SHA
        || model.binding.training_data_blake3 != data_hash
        || model.binding.source_commit != source
        || model.version != 2
        || !model
            .binding
            .credit_assignment
            .starts_with("A3 staged hard integer coarse routing;")
    {
        return Err(
            "loaded A3 model source/tokenizer/data/version/training binding mismatch".into(),
        );
    }
    // Execute a loaded generation before any wider development scoring.
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

    let loaded_fit_routes =
        integrated_attention_a3_routes::collect_cases(&model, &data.fit, &data)?;
    let loaded_dev_routes =
        integrated_attention_a3_routes::collect_cases(&model, &data.dev, &data)?;
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
        &json!({"scope":"exposed source-separated development; likelihood scoring cost is not decoder cost",
        "with_read":with_read,"without_read":no_read,"with_read_bpt":with_read.bits_per_token(),
        "without_read_bpt":no_read.bits_per_token(),"episodes":rows}),
    )?;
    let result = json!({"schema":"uor-r4.integrated-attention-a3/1","command":a.command,
        "source_commit":source,"executable_sha256":executable_sha,"artifact_path":model_path,
        "artifact_sha256":sha(&artifact_bytes),"artifact_payload_blake3":hex::encode(model.digest()),
        "artifact_bytes":artifact_bytes.len(),"parameter_bytes":model.parameter_bytes(),
        "algebra":format!("{:?}",model.algebra.kind()),"binding":model.binding,
        "fit_configuration":fit_config,"fit_metrics":fit_total,"fit_stop_reason":stop_reason,
        "hard_route_config_ceiling":route_config,
        "hard_route_report":hard_route_report,
        "hard_route_elapsed_ms":hard_route_elapsed_ms,
        "joint_fit_elapsed_ms":fit_elapsed_ms,
        "hard_route_input_blake3":hard_route_input_blake3,
        "address_bank_delta_by_lane":bank_deltas,
        "loaded_fit_route_summary":route_summary(&loaded_fit_routes),
        "loaded_dev_route_summary":route_summary(&loaded_dev_routes),
        "elapsed_ms":started.elapsed().as_millis(),"development_positions":with_read.positions,
        "training_time_limit_scope":"hard-route optimizer stage bounded at 300 seconds including route collection and checks; joint fit separately bounded at 900 seconds checked before each episode, permits completion of an in-progress episode; evaluation charged separately in cycle ledger",
        "with_read_bpt":with_read.bits_per_token(),"without_read_bpt":no_read.bits_per_token(),
        "generation_rows":generations.len(),"snapshot_generation_parity":true,
        "measurement_scope":"integer session decisions, selected coefficient/table counters; no physical energy or machine-code audit; exact bounded raw history; one-token records; no semantic binder or multi-token copy qualification; long-session posting reconstruction after eviction is not qualified",
        "quality_status":"development observation; inspect complete generated outputs; no full Milestone A acceptance or geometric advantage inferred"});
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
