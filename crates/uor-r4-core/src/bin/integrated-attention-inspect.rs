//! Read-only diagnosis of the loaded integrated attention reader on two exposed
//! correction episodes. Fit-only source pointers are used solely for this report.
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::integrated_attention::{
    features,
    training::{token_nll_offline, TrainingEpisode},
    IntegratedModel, Session,
};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

#[path = "support/integrated_attention_data.rs"]
mod integrated_attention_data;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";

struct Args {
    repo: PathBuf,
    tokenizer: PathBuf,
    model: PathBuf,
    out: PathBuf,
}

fn args() -> AnyResult<Args> {
    let values: Vec<String> = std::env::args().skip(1).collect();
    if values.len() != 4 {
        return Err("usage: integrated-attention-inspect REPO TOKENIZER MODEL OUT".into());
    }
    Ok(Args {
        repo: values[0].clone().into(),
        tokenizer: values[1].clone().into(),
        model: values[2].clone().into(),
        out: values[3].clone().into(),
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

fn common_prefix(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

#[derive(Clone, Copy)]
struct Seen {
    event_id: u64,
    record_id: Option<u64>,
    key: [u8; 16],
}

#[derive(Default)]
struct Totals {
    positions: usize,
    source_written: usize,
    source_admitted: usize,
    source_ranked: usize,
    source_selected: usize,
    gate_open: usize,
    incomplete_searches: usize,
    candidate_count: usize,
    distinct_candidate_codes: usize,
    source_code_collisions: usize,
    no_read_nll_nats: f64,
    actual_read_nll_nats: f64,
    forced_source_nll_nats: f64,
}

impl Totals {
    fn value(&self, scope: &str) -> Value {
        json!({
            "scope":scope,
            "positions":self.positions,
            "source_written":self.source_written,
            "source_admitted":self.source_admitted,
            "source_ranked_before_gate":self.source_ranked,
            "source_selected_after_gate":self.source_selected,
            "gate_open":self.gate_open,
            "incomplete_searches":self.incomplete_searches,
            "candidate_count_sum":self.candidate_count,
            "distinct_candidate_codes_sum":self.distinct_candidate_codes,
            "source_code_collision_count_sum":self.source_code_collisions,
            "no_read_nll_nats":self.no_read_nll_nats,
            "actual_read_nll_nats":self.actual_read_nll_nats,
            "forced_annotated_source_nll_nats":self.forced_source_nll_nats,
            "forced_minus_actual_nll_nats":self.forced_source_nll_nats-self.actual_read_nll_nats,
        })
    }

    fn add(&mut self, other: &Self) {
        self.positions += other.positions;
        self.source_written += other.source_written;
        self.source_admitted += other.source_admitted;
        self.source_ranked += other.source_ranked;
        self.source_selected += other.source_selected;
        self.gate_open += other.gate_open;
        self.incomplete_searches += other.incomplete_searches;
        self.candidate_count += other.candidate_count;
        self.distinct_candidate_codes += other.distinct_candidate_codes;
        self.source_code_collisions += other.source_code_collisions;
        self.no_read_nll_nats += other.no_read_nll_nats;
        self.actual_read_nll_nats += other.actual_read_nll_nats;
        self.forced_source_nll_nats += other.forced_source_nll_nats;
    }
}

fn inspect_episode(
    model: &IntegratedModel,
    tokenizer: &HfBpeTokenizer,
    episode: &TrainingEpisode,
) -> AnyResult<(Vec<Value>, Totals)> {
    episode.validate(&model.config)?;
    let prompt_len = episode
        .prompt_len
        .ok_or("correction episode lacks prompt")?;
    let lanes = model.config.lanes;
    let mut session = Session::new(model.runtime(), episode.source_id)?;
    let mut seen = Vec::<Seen>::with_capacity(episode.tokens.len());
    let mut rows = Vec::with_capacity(episode.tokens.len() - prompt_len);
    let mut totals = Totals::default();
    for (position, &target) in episode.tokens.iter().enumerate() {
        let read = session.read(model.runtime(), true)?;
        if position >= prompt_len {
            let source_position = episode.source_targets[position]
                .ok_or("correction answer has no annotated source")?;
            let source = *seen
                .get(source_position)
                .ok_or("annotated source was not observed causally")?;
            let source_token = episode.tokens[source_position];
            let source_code = &source.key[..lanes];
            let candidates = &read.candidates[..read.candidate_count];
            let admitted = source.record_id.and_then(|id| {
                candidates
                    .iter()
                    .position(|candidate| candidate.record_id == id)
            });
            let ranked = read.ranked.map(|index| read.candidates[index]);
            let selected = read.selected.map(|index| read.candidates[index]);
            let distinct_codes: HashSet<Vec<u8>> = candidates
                .iter()
                .map(|candidate| candidate.key[..lanes].to_vec())
                .collect();
            let code_collisions = candidates
                .iter()
                .filter(|candidate| candidate.key[..lanes] == *source_code)
                .count();
            let energies: Vec<i32> = candidates
                .iter()
                .map(|candidate| candidate.energy)
                .collect();
            let min_energy = energies.iter().min().copied();
            let max_energy = energies.iter().max().copied();
            let min_energy_ties = min_energy
                .map(|minimum| energies.iter().filter(|&&energy| energy == minimum).count())
                .unwrap_or(0);
            let no_read = token_nll_offline(
                &model.output,
                features(&model.config, session.last_token, &session.state, None).as_slice(),
                None,
                target,
            )?;
            let actual = token_nll_offline(
                &model.output,
                features(
                    &model.config,
                    session.last_token,
                    &session.state,
                    read.selected_token(),
                )
                .as_slice(),
                read.selected_token(),
                target,
            )?;
            // Counterfactual one-step score only. The annotated source is never
            // passed to Session::read or Session::observe.
            let forced = token_nll_offline(
                &model.output,
                features(
                    &model.config,
                    session.last_token,
                    &session.state,
                    Some(source_token),
                )
                .as_slice(),
                Some(source_token),
                target,
            )?;
            rows.push(json!({
                "episode":episode.name,
                "position":position,
                "answer_position":position-prompt_len,
                "target_token":target,
                "target_token_text":String::from_utf8_lossy(&episode.token_bytes[position]),
                "target_token_bytes_hex":hex::encode(&episode.token_bytes[position]),
                "annotated_source_position":source_position,
                "annotated_source_event_id":source.event_id,
                "annotated_source_record_id":source.record_id,
                "annotated_source_token":source_token,
                "annotated_source_token_text":String::from_utf8_lossy(&episode.token_bytes[source_position]),
                "annotated_source_token_bytes_hex":hex::encode(&episode.token_bytes[source_position]),
                "annotated_source_code":source_code,
                "query_code":&read.query[..lanes],
                "query_source_prefix_agreement":common_prefix(&read.query[..lanes],source_code),
                "source_admitted":admitted.is_some(),
                "source_energy_if_admitted":admitted.map(|index|candidates[index].energy),
                "ranked_before_gate":ranked.map(|candidate|json!({
                    "record_id":candidate.record_id,
                    "source_event_id":candidate.source_event_id,
                    "token":candidate.token,
                    "token_text":String::from_utf8_lossy(&tokenizer.decode_bytes(&[u32::from(candidate.token)])),
                    "code":&candidate.key[..lanes],
                    "energy":candidate.energy,
                    "query_prefix_agreement":common_prefix(&read.query[..lanes],&candidate.key[..lanes]),
                })),
                "ranked_is_annotated_source":ranked.is_some_and(|candidate|Some(candidate.record_id)==source.record_id),
                "gate_enabled":read.gate_enabled,
                "selected_after_gate":selected.map(|candidate|json!({
                    "record_id":candidate.record_id,
                    "source_event_id":candidate.source_event_id,
                    "token":candidate.token,
                    "code":&candidate.key[..lanes],
                    "energy":candidate.energy,
                })),
                "selected_is_annotated_source":selected.is_some_and(|candidate|Some(candidate.record_id)==source.record_id),
                "candidate_count":read.candidate_count,
                "distinct_complete_candidate_codes":distinct_codes.len(),
                "candidate_source_code_collisions":code_collisions,
                "candidate_energy_min":min_energy,
                "candidate_energy_max":max_energy,
                "candidate_min_energy_ties":min_energy_ties,
                "search_incomplete":read.search_incomplete,
                "read_access":read.access,
                "no_read_nll_nats":no_read,
                "actual_read_nll_nats":actual,
                "forced_annotated_source_nll_nats":forced,
            }));
            totals.positions += 1;
            totals.source_written += usize::from(source.record_id.is_some());
            totals.source_admitted += usize::from(admitted.is_some());
            totals.source_ranked += usize::from(
                ranked.is_some_and(|candidate| Some(candidate.record_id) == source.record_id),
            );
            totals.source_selected += usize::from(
                selected.is_some_and(|candidate| Some(candidate.record_id) == source.record_id),
            );
            totals.gate_open += usize::from(read.gate_enabled);
            totals.incomplete_searches += usize::from(read.search_incomplete);
            totals.candidate_count += read.candidate_count;
            totals.distinct_candidate_codes += distinct_codes.len();
            totals.source_code_collisions += code_collisions;
            totals.no_read_nll_nats += no_read;
            totals.actual_read_nll_nats += actual;
            totals.forced_source_nll_nats += forced;
        }
        let observed = session.observe(
            model.runtime(),
            u32::from(target),
            &episode.token_bytes[position],
            read.selected_token(),
        )?;
        seen.push(Seen {
            event_id: observed.event_id,
            record_id: observed.record_id,
            key: observed.key,
        });
    }
    Ok((rows, totals))
}

fn execute(a: &Args) -> AnyResult<()> {
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    let tokenizer_sha = sha(&tokenizer_bytes);
    if tokenizer_sha != TOKENIZER_SHA {
        return Err("pinned tokenizer digest mismatch".into());
    }
    let tokenizer =
        HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let data = integrated_attention_data::load_development_data(&a.repo, &tokenizer)?;
    let data_hash = blake3::hash(&serde_json::to_vec(&data.manifest)?)
        .to_hex()
        .to_string();
    let artifact_bytes = fs::read(&a.model)?;
    let model = IntegratedModel::from_bytes(&artifact_bytes)?;
    if model.binding.tokenizer_sha256 != tokenizer_sha
        || model.binding.training_data_blake3 != data_hash
    {
        return Err("loaded model tokenizer/data binding mismatch".into());
    }
    let executable = std::env::current_exe()?;
    let evaluator_source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if evaluator_source_commit == "UNBOUND" {
        return Err("build with the evaluator source commit bound".into());
    }
    let mut rows = Vec::new();
    let mut episode_summaries = Vec::new();
    let mut total = Totals::default();
    for name in ["orion-allowed", "larch-denied"] {
        let episode = data
            .dev
            .iter()
            .find(|episode| episode.name == format!("dev:correction:{name}"))
            .ok_or("missing expected development correction episode")?;
        let (episode_rows, episode_total) = inspect_episode(&model, &tokenizer, episode)?;
        rows.extend(episode_rows);
        episode_summaries.push(episode_total.value(&episode.name));
        total.add(&episode_total);
    }
    let result = json!({
        "schema":"uor-r4.integrated-attention-inspection/1",
        "scope":"read-only loaded-model teacher-forced diagnosis of two exposed development corrections; one-step forced source uses fit-only annotation offline and never enters the session; no generation or capability claim",
        "artifact_path":a.model,
        "artifact_sha256":sha(&artifact_bytes),
        "artifact_payload_blake3":hex::encode(model.digest()),
        "artifact_training_source_commit":model.binding.source_commit,
        "artifact_training_updates":model.binding.training_updates,
        "evaluator_source_commit":evaluator_source_commit,
        "executable_sha256":sha(&fs::read(executable)?),
        "tokenizer_sha256":tokenizer_sha,
        "development_manifest_blake3":data_hash,
        "algebra":format!("{:?}",model.algebra.kind()),
        "episodes":episode_summaries,
        "total":total.value("both development corrections"),
        "rows":rows.len(),
    });
    write_json(&a.out.join("rows.json"), &rows)?;
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
