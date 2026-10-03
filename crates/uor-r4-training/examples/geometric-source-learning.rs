//! Exposed-development B8 selected-source learning, not native-chat qualification.
//! The floating parent is frozen. Only the geometric occurrence consumer learns.
//! Retained predicted source frames are inputs; accepted complete answers are labels.
use candle_core::{Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_occurrence_read::{
    FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_occurrence_consumer::{
        mix_scores, mix_source_view_scores, ConsumerIdentity, ConsumerWeights,
        NativeConsumerArtifact,
    },
    geometric_source_emission_view::{SourceEmissionCompiler, SourceEmissionView},
    geometric_stack::StackModel,
    sha256_file,
    stack_checkpoint::{load_checkpoint, sealed_manifest_sha256},
    stack_grounded_session::{CompiledAction, MemoryEffect, TurnOutcome},
    stack_store::StoreRead,
    Result, TrainingError,
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: String,
    out: PathBuf,
    retained_report: PathBuf,
    tokenizer: PathBuf,
    checkpoint: PathBuf,
    initial_construction: PathBuf,
    maximum_updates: usize,
    maximum_seconds: u64,
    #[serde(default)]
    source_emission_view: bool,
}
#[derive(Deserialize)]
struct Report {
    schema: String,
    tokenizer_sha256: String,
    checkpoint_manifest_sha256: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    case: Labels,
    read: bool,
    outcomes: Vec<TurnOutcome>,
}
#[derive(Deserialize)]
struct Labels {
    id: String,
    answers: Answers,
}
#[derive(Deserialize)]
struct Answers {
    accepted: Vec<String>,
}
struct Episode {
    id: String,
    query: Vec<u32>,
    parent_input: Vec<u32>,
    tokens: Vec<u32>,
    record: u64,
    commit: u64,
    relation: u32,
    entity: Vec<u32>,
    target: Vec<u32>,
    accepted: Vec<String>,
    selected_text: String,
    source_view: Option<SourceEmissionView>,
}
impl Episode {
    fn scoring_source_len(&self) -> usize {
        self.source_view
            .as_ref()
            .map(|v| v.emitted_token_ids().len())
            .unwrap_or(self.tokens.len())
    }
    fn frame(&self) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: self.record,
                commit: self.commit,
            },
            metadata: FrameMetadata {
                scope: b"m-world-v2",
                entity: &self.entity,
                relation: self.relation,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: &self.tokens,
        }
    }
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn write(p: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn hash(scores: &[f32]) -> String {
    let mut h = Sha256::new();
    for x in scores {
        h.update(x.to_bits().to_le_bytes());
    }
    hex::encode(h.finalize())
}
struct DriverLoss {
    loss: Tensor,
    trace: Value,
    mixed_scores: Vec<f32>,
}
struct DriverRead {
    trace: Value,
    mixed_scores: Option<Vec<f32>>,
}
fn compile(
    weights: &ConsumerWeights,
    identity: &ConsumerIdentity,
    view: bool,
) -> Result<NativeConsumerArtifact> {
    if view {
        weights.compile_source_view(identity.clone())
    } else {
        weights.compile(identity.clone())
    }
}
fn read(
    native: &NativeConsumerArtifact,
    episode: &Episode,
    prefix: &[u32],
    parent: Option<&[f32]>,
) -> Result<DriverRead> {
    match &episode.source_view {
        Some(view) => {
            let trace = native.read_source_view(episode.frame(), view, &episode.query, prefix)?;
            let mixed_scores = parent
                .map(|p| mix_source_view_scores(p, &trace, true))
                .transpose()?;
            Ok(DriverRead {
                trace: serde_json::to_value(trace)?,
                mixed_scores,
            })
        }
        None => {
            let trace = native.read(episode.frame(), &episode.query, prefix)?;
            let mixed_scores = parent.map(|p| mix_scores(p, &trace, true)).transpose()?;
            Ok(DriverRead {
                trace: serde_json::to_value(trace)?,
                mixed_scores,
            })
        }
    }
}
fn deadline(start: Instant, seconds: u64) -> Result<()> {
    if start.elapsed().as_secs() >= seconds {
        Err(invalid("declared wall limit reached; attempt retained"))
    } else {
        Ok(())
    }
}
fn prepare(
    report: Report,
    tok: &ByteBpeTokenizer,
    eos: u32,
    compiler: Option<&SourceEmissionCompiler>,
) -> Result<Vec<Episode>> {
    let mut episodes = Vec::new();
    for row in report.rows.into_iter().filter(|x| x.read) {
        let turn = row
            .outcomes
            .last()
            .ok_or_else(|| invalid("empty retained outcome"))?;
        let found = match &turn.memory {
            MemoryEffect::Read {
                read: StoreRead::Found(f),
            } => f,
            _ => return Err(invalid("last predicted source is not Found")),
        };
        if found.conflict || !turn.controls.read || !turn.controls.write {
            return Err(invalid("conflict/intervention not admitted"));
        }
        let relation = match turn.action {
            CompiledAction::QueryCurrent { relation } => relation,
            _ => return Err(invalid("only predicted current query admitted")),
        };
        // Source-only protocol view is constructed before inspecting answer labels.
        let source_view = match compiler {
            Some(c) => Some(c.compile(&found.tokens)?),
            None => None,
        };
        if let Some(view) = &source_view {
            let original_bytes = tok.decode_bytes(&found.tokens);
            let mut expected_rendered = vec![b' '];
            expected_rendered.extend_from_slice(&original_bytes);
            if view.original_token_ids() != found.tokens.as_slice()
                || view.original_bytes() != original_bytes.as_slice()
                || view.rendered_bytes() != expected_rendered.as_slice()
                || view.emitted_token_ids().is_empty()
                || view.emitted_token_ids().iter().any(|x| *x as usize >= 4096)
            {
                return Err(invalid("source-only emission view changed original identity or protocol separator bytes"));
            }
        }
        let emitted_len = source_view
            .as_ref()
            .map(|v| v.emitted_token_ids().len())
            .unwrap_or(found.tokens.len());
        let query = tok.encode(&turn.source);
        let answer = row
            .case
            .answers
            .accepted
            .first()
            .ok_or_else(|| invalid("missing frozen complete answer labels"))?;
        let mut target = tok.encode(&format!(" {answer}"));
        target.push(eos);
        if query.is_empty()
            || found.tokens.is_empty()
            || target.len() > 64
            || query.len() + found.tokens.len() + 64 > 128
            || query.len() + emitted_len + 64 > 128
            || turn.emitter_input_ids.len() + 64 > 384
        {
            return Err(invalid(
                "whole response/evaluation admission exceeds consumer128/parent384",
            ));
        }
        for ids in [&query, &target, &found.tokens, &turn.emitter_input_ids] {
            if ids.iter().any(|x| *x as usize >= 4096) {
                return Err(invalid("token outside V4096"));
            }
        }
        episodes.push(Episode {
            id: row.case.id,
            query,
            parent_input: turn.emitter_input_ids.clone(),
            tokens: found.tokens.clone(),
            record: found.record,
            commit: found.commit,
            relation,
            entity: tok.encode("user"),
            target,
            accepted: row.case.answers.accepted,
            selected_text: tok.decode(&found.tokens),
            source_view,
        });
    }
    if episodes.len() != 20 {
        return Err(invalid(
            "exact retained 20 exposed development cases required",
        ));
    }
    Ok(episodes)
}
fn load(
    a: &Args,
) -> Result<(
    uor_r4_training::stack_checkpoint::StackCheckpoint,
    ByteBpeTokenizer,
    Vec<Episode>,
    ConsumerWeights,
    ConsumerIdentity,
)> {
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained report envelope missing"))?,
    )?;
    report_output::verify(&a.initial_construction)?;
    let d: Report = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if d.schema != "uor-r4.source-binding-diagnostic/1"
        || d.tokenizer_sha256 != sha256_file(&a.tokenizer)?
        || d.checkpoint_manifest_sha256
            != sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "retained source/tokenizer/checkpoint binding differs",
        ));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("unreadable tokenizer"))?;
    let parent =
        load_checkpoint(&a.checkpoint, &Device::Cpu).map_err(|e| invalid(e.to_string()))?;
    parent
        .identity()
        .check_tokenizer(&bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if parent.identity().protocol.schema != uor_r4_tokenizer::dialogue::SCHEMA_V2 {
        return Err(invalid("literal-role dialogue protocol2 required"));
    }
    if parent.model.config.vocab_size != 4096
        || parent.model.config.context != 384
        || tok.vocab_size() != 4096
    {
        return Err(invalid("actual V4096 context384 parent required"));
    }
    let source_view_compiler = if a.source_emission_view {
        // Explicitly bind the known literal-role /2 separator policy.
        if !parent.identity().protocol.spaced_content()
            || parent.identity().protocol.assistant_marker() != "Assistant:"
        {
            return Err(invalid(
                "source emission view requires literal-role protocol2 content separator",
            ));
        }
        Some(SourceEmissionCompiler::new(&bytes)?)
    } else {
        None
    };
    let episodes = prepare(
        d,
        &tok,
        parent.identity().protocol.eos_id,
        source_view_compiler.as_ref(),
    )?;
    let initial: Value =
        serde_json::from_slice(&fs::read(a.initial_construction.join("report.json"))?)?;
    if initial["optimizer_updates"] != 0
        || initial["retained_report_sha256"] != sha256_file(&a.retained_report)?
        || initial["parent_model_sha256"] != parent.record.model_sha256
    {
        return Err(invalid("initial sealed construction identity differs"));
    }
    let weights = ConsumerWeights::load_source(&a.initial_construction.join("consumer-source"))?;
    let identity = ConsumerIdentity::new(&bytes, &a.checkpoint)?;
    let initial_raw = NativeConsumerArtifact::load(
        &a.initial_construction.join("consumer-native"),
        &weights,
        &identity,
    )?;
    if a.source_emission_view {
        let ep = episodes
            .first()
            .ok_or_else(|| invalid("empty initial representation witness"))?;
        let view = ep
            .source_view
            .as_ref()
            .ok_or_else(|| invalid("missing initial representation witness"))?;
        let rejection = match initial_raw.read_source_view(ep.frame(), view, &ep.query, &[]) {
            Err(e) => e.to_string(),
            Ok(_) => {
                return Err(invalid(
                    "retained initial raw /1 artifact admitted source-view read",
                ))
            }
        };
        write(
            &a.out.join("initial-representation-guards.json"),
            &json!({
                "retained_initial_raw_artifact_rejects_source_view_read":true,
                "initial_native_metadata_sha256":sha256_file(&a.initial_construction.join("consumer-native/metadata.json"))?,
                "error":rejection
            }),
        )?;
    }
    Ok((parent, tok, episodes, weights, identity))
}
type ParentCache = BTreeMap<usize, Vec<Vec<f32>>>;
fn cache_episode(
    index: usize,
    ep: &Episode,
    parent: &StackModel,
    cache: &mut ParentCache,
    start: Instant,
    a: &Args,
) -> Result<()> {
    if cache.contains_key(&index) {
        return Ok(());
    }
    let mut scores = Vec::new();
    for step in 0..ep.target.len() {
        deadline(start, a.maximum_seconds)?;
        let mut ids = ep.parent_input.clone();
        ids.extend_from_slice(&ep.target[..step]);
        let row = parent.next_scores(&ids)?;
        if row.len() != 4096 || row.iter().any(|x| !x.is_finite()) {
            return Err(invalid(
                "invalid frozen full pointer-aware parent distribution",
            ));
        }
        scores.push(row);
    }
    cache.insert(index, scores);
    Ok(())
}
struct Batch {
    gradients: BTreeMap<String, Tensor>,
    report: Value,
}
fn batch(
    indices: &[usize],
    episodes: &[Episode],
    weights: &ConsumerWeights,
    native: &NativeConsumerArtifact,
    cache: &ParentCache,
    start: Instant,
    a: &Args,
) -> Result<Batch> {
    if indices.len() != 8 {
        return Err(invalid("complete eight-episode batch required"));
    }
    let begun = Instant::now();
    let params = weights.parameters();
    let prepared = weights.prepare(native)?;
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut episode_rows = Vec::new();
    let mut mean = 0f64;
    let mut tokens = 0usize;
    for &index in indices {
        let ep = episodes
            .get(index)
            .ok_or_else(|| invalid("episode index invalid"))?;
        let parents = cache
            .get(&index)
            .ok_or_else(|| invalid("missing frozen parent cache"))?;
        let mut loss_sum = 0f64;
        let mut token_rows = Vec::new();
        for (step, &target) in ep.target.iter().enumerate() {
            deadline(start, a.maximum_seconds)?;
            let parent = parents
                .get(step)
                .ok_or_else(|| invalid("incomplete cached episode"))?;
            let out = match &ep.source_view {
                Some(view) => {
                    let out = prepared.loss_source_view(
                        ep.frame(),
                        view,
                        &ep.query,
                        &ep.target[..step],
                        parent,
                        target,
                    )?;
                    DriverLoss {
                        loss: out.loss,
                        trace: serde_json::to_value(out.trace)?,
                        mixed_scores: out.mixed_scores,
                    }
                }
                None => {
                    let out =
                        prepared.loss(ep.frame(), &ep.query, &ep.target[..step], parent, target)?;
                    DriverLoss {
                        loss: out.loss,
                        trace: serde_json::to_value(out.trace)?,
                        mixed_scores: out.mixed_scores,
                    }
                }
            };
            let value = out.loss.to_scalar::<f32>()?;
            let expected = -*out
                .mixed_scores
                .get(target as usize)
                .ok_or_else(|| invalid("target outside mixed vocabulary"))?;
            if !value.is_finite()
                || !expected.is_finite()
                || (value - expected).abs() > 1e-4 + 1e-5 * expected.abs()
            {
                return Err(invalid("loss differs from actual native mixed target NLL"));
            }
            loss_sum += f64::from(value);
            tokens += 1;
            let scaled = (&out.loss * (1f64 / (ep.target.len() as f64 * 8.)))?;
            let store = scaled.backward()?;
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let detached = g.detach();
                    if let Some(prior) = gradients.remove(name) {
                        gradients.insert(name.clone(), (&prior + &detached)?.detach());
                    } else {
                        gradients.insert(name.clone(), detached);
                    }
                }
            }
            token_rows.push(json!({"step":step,"target_label_only":target,"target_present_original_source_diagnostic":ep.tokens.contains(&target),"target_present_emission_view_diagnostic":ep.source_view.as_ref().map(|v|v.emitted_token_ids().contains(&target)),"teacher_forced_prefix_ids":&ep.target[..step],"parent_scores_sha256":hash(parent),"native_trace":out.trace,"mixed_scores_sha256":hash(&out.mixed_scores),"nll":value,"loss_native_equal":true}));
        }
        let episode_mean = loss_sum / ep.target.len() as f64;
        mean += episode_mean / 8.;
        episode_rows.push(json!({"id":ep.id,"source_record":ep.record,"source_commit":ep.commit,"source_tokens":ep.tokens,"source_emission_view":ep.source_view,"query_ids":ep.query,"target_ids_labels_only":ep.target,"target_count":ep.target.len(),"mean_token_nll":episode_mean,"tokens":token_rows}));
    }
    let mut sumsq = 0f64;
    let mut gradient_rows = Vec::new();
    for (name, g) in &gradients {
        let values = g.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid(format!("nonfinite accumulated gradient {name}")));
        }
        let square: f64 = values.iter().map(|x| f64::from(*x).powi(2)).sum();
        sumsq += square;
        gradient_rows.push(json!({"name":name,"l1":values.iter().map(|x|f64::from(x.abs())).sum::<f64>(),"squared_norm":square,"nonzero":values.iter().filter(|x|**x!=0.).count()}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episode_indices":indices,"batch_episodes":8,"token_count":tokens,"objective":"mean over 8 episodes of mean token CE including EOS","mean_episode_nll":mean,"gradient_global_norm":sumsq.sqrt(),"gradient_parameters":gradient_rows,"elapsed_seconds":begun.elapsed().as_secs_f64(),"episodes":episode_rows}),
    })
}
fn apply(
    weights: &ConsumerWeights,
    optimizer: &mut AdamW,
    gradients: BTreeMap<String, Tensor>,
) -> Result<f64> {
    let params = weights.parameters();
    let mut square = 0f64;
    for g in gradients.values() {
        for x in g.flatten_all()?.to_vec1::<f32>()? {
            square += f64::from(x).powi(2);
        }
    }
    let norm = square.sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite clip norm"));
    }
    let factor = if norm > 1. { 1. / norm } else { 1. };
    // A constant's backward store contains only its scalar seed; accumulated parameter gradients
    // are inserted. No token graph/intermediate store survives this update.
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    for (name, g) in gradients {
        let var = params
            .get(&name)
            .ok_or_else(|| invalid("gradient source variable missing"))?;
        store.insert(var.as_tensor(), (&g * factor)?.detach());
    }
    optimizer.step(&store)?;
    weights.project_quarter_range()?;
    Ok(factor)
}
fn evaluate(
    stage: &str,
    episodes: &[Episode],
    parent: &StackModel,
    tok: &ByteBpeTokenizer,
    protocol: &uor_r4_tokenizer::dialogue::DialogueProtocol,
    native: &NativeConsumerArtifact,
    start: Instant,
    a: &Args,
) -> Result<Value> {
    let begun = Instant::now();
    let mut rows = Vec::new();
    let mut complete = 0usize;
    for ep in episodes {
        let mut generated = Vec::<u32>::new();
        let mut traces = Vec::new();
        let mut ended = false;
        for step in 0..64 {
            deadline(start, a.maximum_seconds)?;
            let mut ids = ep.parent_input.clone();
            ids.extend_from_slice(&generated);
            if ids.len() > 384 || ep.scoring_source_len() + ep.query.len() + generated.len() > 128 {
                return Err(invalid("own generated prefix exceeded declared admission"));
            }
            let parent_scores = parent.next_scores(&ids)?;
            let observed = read(native, ep, &generated, Some(&parent_scores))?;
            let trace = observed.trace;
            let mixed = observed
                .mixed_scores
                .ok_or_else(|| invalid("missing generated-prefix score mixture"))?;
            let mut chosen = 0usize;
            let mut best = f32::NEG_INFINITY;
            for (id, &score) in mixed.iter().enumerate() {
                if score.is_nan() || score == f32::INFINITY {
                    return Err(invalid("invalid greedy mixed score"));
                }
                if score > best {
                    chosen = id;
                    best = score;
                }
            }
            if !best.is_finite() {
                return Err(invalid(
                    "greedy distribution has no positive-probability token",
                ));
            }
            let token = u32::try_from(chosen).map_err(|_| invalid("chosen id exceeds u32"))?;
            traces.push(json!({"step":step,"own_prefix_ids":generated,"parent_input_ids":ids,"parent_scores_sha256":hash(&parent_scores),"mixed_scores_sha256":hash(&mixed),"chosen_token":token,"chosen_log_probability":best,"native_trace":trace}));
            if token == protocol.eos_id {
                ended = true;
                break;
            }
            generated.push(token);
        }
        let decoded = tok.decode(&generated);
        let reply = protocol.reply_text(&decoded);
        let accepted = ended && ep.accepted.iter().any(|x| x == reply);
        if accepted {
            complete += 1;
        }
        rows.push(json!({"id":ep.id,"source_record":ep.record,"source_commit":ep.commit,"source_tokens":ep.tokens,"source_emission_view":ep.source_view,"source_text":ep.selected_text,"query_ids":ep.query,"reply_ids":generated,"reply_text":reply,"eos":ended,"stop":if ended{"eos"}else{"max64"},"accepted_complete_answer":accepted,"selected_text_mention_diagnostic_only":reply.contains(&ep.selected_text),"accepted_labels_evaluation_only":ep.accepted,"tokens":traces}));
        write(
            &a.out.join(format!("{stage}-progress.json")),
            &json!({"completed_cases":rows.len(),"rows":rows,"elapsed_seconds":begun.elapsed().as_secs_f64()}),
        )?;
    }
    Ok(
        json!({"scope":"all20exposed development, greedy ownprefix, exact finite complete answer membership plus EOS; not heldout","complete_answers":complete,"cases":rows.len(),"elapsed_seconds":begun.elapsed().as_secs_f64(),"rows":rows}),
    )
}
fn save_checkpoint_with_reader(
    path: &Path,
    weights: &ConsumerWeights,
    identity: &ConsumerIdentity,
    episodes: &[Episode],
    cache: &ParentCache,
    updates: usize,
    status: &str,
    source_emission_view: bool,
) -> Result<(Value, NativeConsumerArtifact)> {
    report_output::claim(path)?;
    weights.save_source(&path.join("consumer-source"))?;
    let native = compile(weights, identity, source_emission_view)?;
    native.save(&path.join("consumer-native"))?;
    let loaded_source = ConsumerWeights::load_source(&path.join("consumer-source"))?;
    let loaded =
        NativeConsumerArtifact::load(&path.join("consumer-native"), &loaded_source, identity)?;
    let representation_guards = if source_emission_view {
        let episode = episodes
            .first()
            .ok_or_else(|| invalid("empty source-view checkpoint witness"))?;
        let view = episode
            .source_view
            .as_ref()
            .ok_or_else(|| invalid("missing source-view checkpoint witness"))?;
        let raw_rejection = match loaded.read(episode.frame(), &episode.query, &[]) {
            Err(e) => e.to_string(),
            Ok(_) => return Err(invalid("loaded /2 artifact admitted raw /1 read")),
        };
        let raw_artifact = loaded_source.compile(identity.clone())?;
        let view_rejection =
            match raw_artifact.read_source_view(episode.frame(), view, &episode.query, &[]) {
                Err(e) => e.to_string(),
                Ok(_) => return Err(invalid("raw /1 artifact admitted source-view /2 read")),
            };
        // Negative payloads live only beneath this new, unsealed checkpoint.
        // Never alter the retained construction or the accepted exported payload.
        let negative_root = path.join("source-view-policy-negative");
        report_output::claim(&negative_root)?;
        let negative_native = negative_root.join("native");
        loaded.save(&negative_native)?;
        let metadata_path = negative_native.join("metadata.json");
        let original_metadata_sha256 = sha256_file(&metadata_path)?;
        let mut metadata: Value = serde_json::from_slice(&fs::read(&metadata_path)?)?;
        metadata["source_view_policy"] = json!("unknown-source-emission-policy-negative/1");
        write(&metadata_path, &metadata)?;
        let policy_rejection =
            match NativeConsumerArtifact::load(&negative_native, &loaded_source, identity) {
                Err(e) => e.to_string(),
                Ok(_) => return Err(invalid("altered /2 source-view policy was admitted")),
            };
        let guards = json!({"loaded_source_view_rejects_raw_read":true,"raw_source_rejects_view_read":true,"corrupt_source_view_policy_rejected":true,"raw_read_error":raw_rejection,"view_read_error":view_rejection,"policy_error":policy_rejection,"original_metadata_sha256":original_metadata_sha256,"corrupt_metadata_sha256":sha256_file(&metadata_path)?});
        write(&negative_root.join("representation-guards.json"), &guards)?;
        report_output::seal(&negative_root)?;
        report_output::verify(&negative_root)?;
        Some(guards)
    } else {
        None
    };
    let mut traces = Vec::new();
    for (index, ep) in episodes.iter().enumerate() {
        let parent_scores = cache.get(&index).and_then(|x| x.first()).map(Vec::as_slice);
        let original = read(&native, ep, &[], parent_scores)?;
        let reloaded = read(&loaded, ep, &[], parent_scores)?;
        if original.trace != reloaded.trace {
            return Err(invalid(
                "checkpoint independent source/native replay mismatch",
            ));
        }
        let score_equal = if let (Some(original_scores), Some(reloaded_scores)) =
            (&original.mixed_scores, &reloaded.mixed_scores)
        {
            if hash(original_scores) != hash(reloaded_scores) {
                return Err(invalid("checkpoint hard-score replay mismatch"));
            }
            Some(true)
        } else {
            None
        };
        traces.push(json!({"id":ep.id,"trace_equal":true,"hard_scores_equal_when_parent_cache_available":score_equal,"trace":original.trace,"source_emission_view":ep.source_view}));
    }
    let receipt = json!({"optimizer_updates":updates,"status":status,"independent_source_native_reload_equal":true,"scope":"teacherforced-prefix0 native trace replay, not fresh-process chat","native_stats":native.stats(),"source_emission_view":source_emission_view,"artifact_schema_policy":if source_emission_view {"source-emission-view /2"}else{"original-occurrence /1"},"representation_guards":representation_guards,"traces":traces});
    write(&path.join("checkpoint.json"), &receipt)?;
    report_output::seal(path)?;
    report_output::verify(path)?;
    Ok((receipt, loaded))
}
fn save_checkpoint(
    path: &Path,
    weights: &ConsumerWeights,
    identity: &ConsumerIdentity,
    episodes: &[Episode],
    cache: &ParentCache,
    updates: usize,
    status: &str,
    source_emission_view: bool,
) -> Result<Value> {
    save_checkpoint_with_reader(
        path,
        weights,
        identity,
        episodes,
        cache,
        updates,
        status,
        source_emission_view,
    )
    .map(|(receipt, _reader)| receipt)
}
fn run(a: &Args) -> Result<()> {
    let start = Instant::now();
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| invalid("full build-bound UOR_BUILD_SOURCE_COMMIT required"))?;
    let (parent, tok, episodes, weights, identity) = load(a)?;
    let initial_native = compile(&weights, &identity, a.source_emission_view)?;
    let mut cache = ParentCache::new();
    let mut batch_reports = Vec::<Value>::new();
    let mut updates = 0usize;
    write(
        &a.out.join("binding.json"),
        &json!({"source_commit":source_commit,"retained_report_sha256":sha256_file(&a.retained_report)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"parent_checkpoint_manifest_sha256":sealed_manifest_sha256(&a.checkpoint).map_err(|e|invalid(e.to_string()))?,"parent_model_sha256":parent.record.model_sha256,"initial_construction_manifest_sha256":sha256_file(&a.initial_construction.join("manifest.json"))?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"protocol_identity":parent.identity().protocol.identity().map_err(|e|invalid(e.to_string()))?,"mode":a.mode,"source_emission_view":a.source_emission_view,"view_scope":"source-only protocol2 separator32 rendered candidates retain original source IDs and byte-occurrence provenance; no label-dependent adaptation","maximum_updates":a.maximum_updates,"maximum_seconds":a.maximum_seconds,"optimizer":{"name":"AdamW","lr":0.003,"beta1":0.9,"beta2":0.999,"epsilon":1e-8,"weight_decay":0.,"global_gradient_clip":1.,"projection":"quarter-code shadow range after each B8 update"},"objective":"mean8episodes(mean token CE including EOS)","control_scope":"frozen floating pointer-aware parent; no trained ordinary control or transformer superiority gate","evaluation_scope":"20 exposed development cases, no heldout"}),
    )?;
    let mut saved_final_checkpoint: Option<Value> = None;
    let mut final_checkpoint_started = false;
    let work = (|| -> Result<()> {
        if a.mode == "cost" {
            let mut sorted: Vec<usize> = (0..episodes.len()).collect();
            sorted.sort_by(|&x, &y| {
                episodes[x]
                    .target
                    .len()
                    .cmp(&episodes[y].target.len())
                    .then_with(|| episodes[x].id.cmp(&episodes[y].id))
            });
            let short = sorted[..8].to_vec();
            let long = sorted[sorted.len() - 8..].to_vec();
            for (label, indices) in [("shortest8", short), ("longest8", long)] {
                let cache_start = Instant::now();
                for &index in &indices {
                    cache_episode(index, &episodes[index], &parent.model, &mut cache, start, a)?;
                }
                let cache_seconds = cache_start.elapsed().as_secs_f64();
                let measured = batch(
                    &indices,
                    &episodes,
                    &weights,
                    &initial_native,
                    &cache,
                    start,
                    a,
                )?;
                batch_reports.push(json!({"kind":label,"parent_cache_seconds":cache_seconds,"optimizer_updates":0,"batch":measured.report}));
                write(
                    &a.out.join("progress.json"),
                    &json!({"mode":"cost","completed_batches":batch_reports.len(),"optimizer_updates":0,"batches":batch_reports,"wall_seconds":start.elapsed().as_secs_f64()}),
                )?;
            }
        } else {
            let initial = evaluate(
                "initial",
                &episodes,
                &parent.model,
                &tok,
                &parent.identity().protocol,
                &initial_native,
                start,
                a,
            )?;
            write(&a.out.join("initial-evaluation.json"), &initial)?;
            let cache_start = Instant::now();
            for (index, ep) in episodes.iter().enumerate() {
                cache_episode(index, ep, &parent.model, &mut cache, start, a)?;
            }
            write(
                &a.out.join("parent-cache.json"),
                &json!({"seconds":cache_start.elapsed().as_secs_f64(),"episodes":episodes.iter().enumerate().map(|(i,e)|json!({"id":e.id,"teacherforced_target_ids":e.target,"scores_sha256":cache.get(&i).map(|rows|rows.iter().map(|x|hash(x)).collect::<Vec<_>>())})).collect::<Vec<_>>() }),
            )?;
            let mut optimizer = AdamW::new(
                weights.parameters().into_values().collect(),
                ParamsAdamW {
                    lr: 0.003,
                    beta1: 0.9,
                    beta2: 0.999,
                    eps: 1e-8,
                    weight_decay: 0.,
                },
            )?;
            for step in 0..a.maximum_updates {
                deadline(start, a.maximum_seconds)?;
                let indices: Vec<usize> = (0..8)
                    .map(|offset| (step * 8 + offset) % episodes.len())
                    .collect();
                let native = compile(&weights, &identity, a.source_emission_view)?;
                let measured = batch(&indices, &episodes, &weights, &native, &cache, start, a)?;
                let clip_factor = apply(&weights, &mut optimizer, measured.gradients)?;
                updates += 1;
                batch_reports.push(json!({"optimizer_update":updates,"clip_factor":clip_factor,"batch":measured.report}));
                write(
                    &a.out.join("progress.json"),
                    &json!({"mode":"fit","optimizer_updates":updates,"declared_updates":a.maximum_updates,"batches":batch_reports,"wall_seconds":start.elapsed().as_secs_f64()}),
                )?;
                if updates % 4 == 0 {
                    save_checkpoint(
                        &a.out.join(format!("checkpoint-{updates:04}")),
                        &weights,
                        &identity,
                        &episodes,
                        &cache,
                        updates,
                        "intermediate",
                        a.source_emission_view,
                    )?;
                }
            }
            final_checkpoint_started = true;
            let (receipt, final_native) = save_checkpoint_with_reader(
                &a.out.join("final-checkpoint"),
                &weights,
                &identity,
                &episodes,
                &cache,
                updates,
                "fit_updates_completed_before_final_eval",
                a.source_emission_view,
            )?;
            saved_final_checkpoint = Some(receipt);
            let mut final_eval = evaluate(
                "final",
                &episodes,
                &parent.model,
                &tok,
                &parent.identity().protocol,
                &final_native,
                start,
                a,
            )?;
            final_eval["consumer_loaded_from_disk"] = json!(true);
            final_eval["consumer_artifact_metadata_sha256"] = json!(sha256_file(
                &a.out.join("final-checkpoint/consumer-native/metadata.json"),
            )?);
            final_eval["consumer_source_root"] =
                json!(a.out.join("final-checkpoint/consumer-source"));
            write(&a.out.join("final-evaluation.json"), &final_eval)?;
        }
        Ok(())
    })();
    let fallback_checkpoint_status = if work.is_ok() {
        "cost_completed"
    } else {
        "stopped_or_error"
    };
    let final_checkpoint = match saved_final_checkpoint {
        Some(receipt) => Ok(receipt),
        None if final_checkpoint_started => Err(invalid(
            "final checkpoint attempt failed; partial directory retained without a duplicate claim",
        )),
        None => save_checkpoint(
            &a.out.join("final-checkpoint"),
            &weights,
            &identity,
            &episodes,
            &cache,
            updates,
            fallback_checkpoint_status,
            a.source_emission_view,
        ),
    };
    if let Err(e) = &final_checkpoint {
        write(
            &a.out.join("checkpoint-error.json"),
            &json!({"error":e.to_string(),"optimizer_updates":updates}),
        )?;
    }
    let mut hard_payload_changes = Vec::new();
    let mut any_hard_change = false;
    if final_checkpoint.is_ok() {
        for name in ["context-q4.bin", "potential-q4.bin", "no-read-q4.bin"] {
            let initial_sha =
                sha256_file(&a.initial_construction.join("consumer-native").join(name))?;
            let final_sha =
                sha256_file(&a.out.join("final-checkpoint/consumer-native").join(name))?;
            let changed = initial_sha != final_sha;
            any_hard_change |= changed;
            hard_payload_changes.push(json!({"file":name,"initial_sha256":initial_sha,"final_sha256":final_sha,"changed":changed}));
        }
    }
    let hard_change_available = final_checkpoint.is_ok();
    let cost_hard_invariant = a.mode != "cost" || (hard_change_available && !any_hard_change);
    let status = if work.is_ok() && final_checkpoint.is_ok() && cost_hard_invariant {
        "completed"
    } else {
        "stopped_or_error"
    };
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-source-learning/1","mode":a.mode,"source_emission_view":a.source_emission_view,"view_scope":"source-only protocol2 separator32 rendered candidates retain original source IDs and byte-occurrence provenance; no label-dependent adaptation","status":status,"optimizer_updates":updates,"declared_updates":a.maximum_updates,"scope":"exposed-development frozen floating parent and learned geometric consumer; not native complete chat or heldout quality","batches":batch_reports,"wall_seconds":start.elapsed().as_secs_f64(),"source_commit":source_commit,"final_checkpoint":final_checkpoint.as_ref().ok(),"hard_payload_change_available":hard_change_available,"hard_payload_changes":hard_payload_changes,"any_hard_native_payload_changed":if hard_change_available {Some(any_hard_change)}else{None},"shadow_only_fit":if hard_change_available {Some(updates>0&&!any_hard_change)}else{None},"hard_change_scope":"packed q4 coefficient changes; not evidence of changed trace, quality or generalization","work_error":work.as_ref().err().map(|x|x.to_string()),"final_checkpoint_error":final_checkpoint.as_ref().err().map(|x|x.to_string()),"cost_hard_unchanged_invariant":if a.mode=="cost" {Some(cost_hard_invariant)}else{None}}),
    )?;
    work?;
    final_checkpoint?;
    if a.mode == "cost" && any_hard_change {
        return Err(invalid("zero-update cost changed packed native payload"));
    }
    Ok(())
}
fn main() -> Result<()> {
    let mut argv = std::env::args().skip(1);
    let file = argv
        .next()
        .ok_or_else(|| invalid("usage: geometric-source-learning ARGS.json"))?;
    if argv.next().is_some() {
        return Err(invalid("one JSON argument expected"));
    }
    let a: Args = serde_json::from_slice(&fs::read(file)?)?;
    if !matches!(a.mode.as_str(), "cost" | "fit")
        || a.maximum_seconds == 0
        || (a.mode == "cost" && a.maximum_updates != 0)
        || (a.mode == "fit" && a.maximum_updates == 0)
    {
        return Err(invalid(
            "cost requires exactly0 updates; fit requires declaredpositive updates andwalllimit",
        ));
    }
    if a.mode == "fit" && a.source_emission_view {
        return Err(invalid("source-emission-view fit is not admitted: the frozen-parent boundary audit retains unsupported period targets at all 20 canonical labeled prefixes; cost validation only until the emission interface is repaired"));
    }
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = run(&a);
    if let Err(e) = &result {
        write(&a.out.join("error.json"), &json!({"error":e.to_string()}))?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
