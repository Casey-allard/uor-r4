//! Native selected-source Copy/Period/Stop realizer on exposed development cases.
//! Parent checkpoint is loaded for identity only; no parent scoring or responses.
//! This is a selected-record component, not a complete language-model qualification.
use candle_core::{Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_occurrence_read::{
    FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity,
};
use uor_r4_integer::h4_tables::{H4Code, HistoricalH4Tables};
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{
            NativeSourceRealizer, ObservedCode, RealizerTrace, SerializableContextReplay,
            SourceRealizerWeights,
        },
        ConsumerIdentity, ConsumerWeights, NativeConsumerArtifact,
    },
    geometric_source_emission_view::{SourceEmissionCompiler, SourceEmissionView},
    sha256_file,
    stack_checkpoint::{load_checkpoint, sealed_manifest_sha256},
    stack_grounded_session::{CompiledAction, MemoryEffect, TurnOutcome},
    stack_store::StoreRead,
    Result, TrainingError,
};
const UPDATES: usize = 64;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: String,
    out: PathBuf,
    retained_report: PathBuf,
    tokenizer: PathBuf,
    checkpoint: PathBuf,
    initial_construction: PathBuf,
    period_seed: u64,
    maximum_seconds: u64,
    fit_admission: Option<PathBuf>,
    #[serde(default)]
    audit_checkpoint: Option<PathBuf>,
    #[serde(default)]
    expected_generation: Option<PathBuf>,
    #[serde(default)]
    audit_report: Option<PathBuf>,
    #[serde(default)]
    transfer_checkpoint: Option<PathBuf>,
    #[serde(default)]
    observation_checkpoint: Option<PathBuf>,
    #[serde(default)]
    consumed_checkpoint: Option<PathBuf>,
    #[serde(default)]
    geometry_checkpoint: Option<PathBuf>,
    #[serde(default)]
    transfer_admission: Option<PathBuf>,
    #[serde(default)]
    query_diagnostic_admission: Option<PathBuf>,
}
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct OptimizerIdentity {
    updates: usize,
    lr: f64,
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    weight_decay: f64,
    gradient_clip: f64,
}
fn optimizer_identity() -> OptimizerIdentity {
    OptimizerIdentity {
        updates: UPDATES,
        lr: 0.003,
        beta1: 0.9,
        beta2: 0.999,
        epsilon: 1e-8,
        weight_decay: 0.,
        gradient_clip: 1.,
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Admission {
    construction_root: PathBuf,
    construction_manifest_sha256: String,
    observed_b8_wall_seconds: f64,
    projected_fit_wall_seconds: f64,
    source_commit: String,
    optimizer: OptimizerIdentity,
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
#[derive(Clone)]
struct Episode {
    id: String,
    query: Vec<u32>,
    tokens: Vec<u32>,
    record: u64,
    commit: u64,
    relation: u32,
    entity: Vec<u32>,
    target: Vec<u32>,
    accepted: Vec<String>,
    source_text: String,
    view: SourceEmissionView,
}
impl Episode {
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
fn deadline(start: Instant, a: &Args) -> Result<()> {
    if start.elapsed().as_secs() >= a.maximum_seconds {
        Err(invalid(
            "declared realizer wall limit reached; checkpoint/progress retained",
        ))
    } else {
        Ok(())
    }
}
fn source_commit() -> Result<&'static str> {
    option_env!("UOR_BUILD_SOURCE_COMMIT")
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| invalid("full build-bound source commit required"))
}
fn admission(a: &Args) -> Result<Option<Admission>> {
    if matches!(
        a.mode.as_str(),
        "readout-geometry-coadapt"
            | "readout-geometry-transfer"
            | "readout-geometry-query-diagnostic"
    ) {
        return Ok(None);
    }
    if a.mode != "fit" {
        if a.fit_admission.is_some() {
            return Err(invalid("construction does not consume fit admission"));
        }
        return Ok(None);
    }
    let p = a
        .fit_admission
        .as_ref()
        .ok_or_else(|| invalid("fit requires measured construction cost admission"))?;
    let v: Admission = serde_json::from_slice(&fs::read(p)?)?;
    report_output::verify(&v.construction_root)?;
    if sha256_file(&v.construction_root.join("manifest.json"))? != v.construction_manifest_sha256
        || v.optimizer != optimizer_identity()
        || v.source_commit != source_commit()?
        || !v.observed_b8_wall_seconds.is_finite()
        || v.observed_b8_wall_seconds <= 0.
        || !v.projected_fit_wall_seconds.is_finite()
        || v.projected_fit_wall_seconds <= 0.
        || v.projected_fit_wall_seconds > a.maximum_seconds as f64
        || v.projected_fit_wall_seconds > 1200.
    {
        return Err(invalid(
            "construction cost/source/optimizer admission differs",
        ));
    }
    let r: Value = serde_json::from_slice(&fs::read(v.construction_root.join("report.json"))?)?;
    if r["schema"] != "uor-r4.geometric-source-realizer/1"
        || r["mode"] != "construction"
        || r["status"] != "completed"
        || r["optimizer_updates"] != 0
        || r["source_commit"] != v.source_commit
        || r["retained_report_sha256"] != sha256_file(&a.retained_report)?
        || r["tokenizer_sha256"] != sha256_file(&a.tokenizer)?
        || r["checkpoint_manifest_sha256"]
            != sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
        || r["initial_construction_manifest_sha256"]
            != sha256_file(&a.initial_construction.join("manifest.json"))?
        || r["period_seed"] != a.period_seed
    {
        return Err(invalid(
            "construction report admission/input binding differs",
        ));
    }
    if r["all_active_gradient_families_present"] != true
        || r["hard_inventory_equal"] != true
        || r["construction_payload_unchanged"] != true
    {
        return Err(invalid(
            "construction gradient/reload/payload invariants are not admitted",
        ));
    }
    let batches = r["batches"]
        .as_array()
        .filter(|b| b.len() == 2)
        .ok_or_else(|| invalid("two complete construction B8 batches required"))?;
    let observed = batches.iter().try_fold(0f64, |largest, b| -> Result<f64> {
        if b["batch"]["episodes"] != 8 {
            return Err(invalid("construction batch is not B8"));
        }
        let seconds = b["batch"]["elapsed_seconds"]
            .as_f64()
            .filter(|x| x.is_finite() && *x > 0.)
            .ok_or_else(|| invalid("construction B8 time missing"))?;
        Ok(largest.max(seconds))
    })?;
    if (observed - v.observed_b8_wall_seconds).abs() > 1e-6 * observed.max(1.)
        || v.projected_fit_wall_seconds < observed * UPDATES as f64
    {
        return Err(invalid(
            "fit projection does not bind actual complete B8 cost",
        ));
    }
    Ok(Some(v))
}
fn prepare(
    d: Report,
    tok: &ByteBpeTokenizer,
    eos: u32,
    compiler: &SourceEmissionCompiler,
) -> Result<Vec<Episode>> {
    let mut out = Vec::new();
    for row in d.rows.into_iter().filter(|x| x.read) {
        let turn = row
            .outcomes
            .last()
            .ok_or_else(|| invalid("empty retained outcome"))?;
        let f = match &turn.memory {
            MemoryEffect::Read {
                read: StoreRead::Found(f),
            } => f,
            _ => return Err(invalid("predicted Found source required")),
        };
        if f.conflict || !turn.controls.read || !turn.controls.write {
            return Err(invalid("conflicted/intervened source not admitted"));
        }
        let relation = match turn.action {
            CompiledAction::QueryCurrent { relation } => relation,
            _ => return Err(invalid("predicted current query required")),
        };
        // Source-only adapter receives no answer labels or selected-value oracle.
        let view = compiler.compile(&f.tokens)?;
        let query = tok.encode(&turn.source);
        if view.original_token_ids() != f.tokens.as_slice() {
            return Err(invalid("source identity changed by view"));
        }
        let accepted = row.case.answers.accepted;
        let answer = accepted
            .first()
            .ok_or_else(|| invalid("complete response labels missing"))?;
        let mut target = tok.encode(&format!(" {answer}"));
        target.push(eos);
        if target.len() > 64
            || query.is_empty()
            || f.tokens.is_empty()
            || f.tokens.len() + query.len() + 64 > 128
            || view.emitted_token_ids().len() + query.len() + 64 > 128
        {
            return Err(invalid(
                "whole native sequence exceeds admission128/response64",
            ));
        }
        if [&query, &target, &f.tokens]
            .iter()
            .any(|ids| ids.iter().any(|id| *id as usize >= 4096))
        {
            return Err(invalid("token outside V4096"));
        }
        out.push(Episode {
            id: row.case.id,
            query,
            tokens: f.tokens.clone(),
            record: f.record,
            commit: f.commit,
            relation,
            entity: tok.encode("user"),
            target,
            accepted,
            source_text: tok.decode(&f.tokens),
            view,
        });
    }
    if out.len() != 20 {
        return Err(invalid("all20exposed development cases required"));
    }
    Ok(out)
}
fn load(
    a: &Args,
    admitted: Option<&Admission>,
) -> Result<(
    SourceRealizerWeights,
    ConsumerIdentity,
    ByteBpeTokenizer,
    Vec<Episode>,
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
            "retained report tokenizer/checkpoint identity differs",
        ));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("unreadable tokenizer"))?;
    // Offline admission only: no scores, hidden states, or responses from this parent.
    let parent =
        load_checkpoint(&a.checkpoint, &Device::Cpu).map_err(|e| invalid(e.to_string()))?;
    parent
        .identity()
        .check_tokenizer(&bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if parent.identity().protocol.schema != SCHEMA_V2
        || tok.vocab_size() != 4096
        || parent.model.config.context != 384
        || parent.model.config.vocab_size != 4096
    {
        return Err(invalid(
            "protocol2 V4096/context384 identity parent required",
        ));
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let episodes = prepare(d, &tok, parent.identity().protocol.eos_id, &compiler)?;
    let identity = ConsumerIdentity::new(&bytes, &a.checkpoint)?;
    drop(parent);
    let initial: Value =
        serde_json::from_slice(&fs::read(a.initial_construction.join("report.json"))?)?;
    if initial["optimizer_updates"] != 0
        || initial["retained_report_sha256"] != sha256_file(&a.retained_report)?
        || initial["parent_model_sha256"] != identity.parent_model_sha256
    {
        return Err(invalid("initial consumer construction identity differs"));
    }
    let consumer = ConsumerWeights::load_source(&a.initial_construction.join("consumer-source"))?;
    NativeConsumerArtifact::load(
        &a.initial_construction.join("consumer-native"),
        &consumer,
        &identity,
    )?;
    let weights = match admitted {
        Some(v) => {
            let frozen = SourceRealizerWeights::load_source(
                &v.construction_root.join("final-checkpoint/realizer-source"),
                &bytes,
            )?;
            NativeSourceRealizer::load(
                &v.construction_root.join("final-checkpoint/realizer-native"),
                &frozen,
                &identity,
            )?;
            frozen
        }
        None => SourceRealizerWeights::new(consumer, &bytes, a.period_seed)?,
    };
    Ok((weights, identity, tok, episodes))
}
struct Batch {
    gradients: BTreeMap<String, Tensor>,
    report: Value,
}
fn batch(
    indices: &[usize],
    episodes: &[Episode],
    weights: &SourceRealizerWeights,
    native: &NativeSourceRealizer,
    start: Instant,
    a: &Args,
) -> Result<Batch> {
    if !readout_batch_admitted(a.mode.as_str(), indices, episodes.len()) {
        return Err(invalid("complete fixed batch required"));
    }
    let begun = Instant::now();
    let prepared = weights.prepare(native)?;
    let params = weights.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut mean = 0f64;
    let mut tokens = 0usize;
    for &index in indices {
        let e = episodes
            .get(index)
            .ok_or_else(|| invalid("episode index outside fixed panel"))?;
        let mut sum = 0f64;
        let mut trace_rows = Vec::new();
        for (step, &target) in e.target.iter().enumerate() {
            deadline(start, a)?;
            let out = prepared.loss(e.frame(), &e.view, &e.query, &e.target[..step], target)?;
            let loss = out.loss.to_scalar::<f32>()?;
            let mass = out
                .trace
                .actions
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .map(|m| m.weight_q31)
                .unwrap_or(0);
            let total = out.trace.actions.total_weight_q31;
            if total == 0 || mass == 0 {
                return Err(invalid("labeled native action target has zero mass"));
            }
            let probability = mass as f64 / total as f64;
            let expected = -probability.ln();
            if !loss.is_finite()
                || out.target_probability != probability
                || (f64::from(loss) - expected).abs() > 1e-4 + 1e-5 * expected.abs()
            {
                return Err(invalid(
                    "loss does not bind actual native action target mass",
                ));
            }
            sum += f64::from(loss);
            tokens += 1;
            let scaled = (&out.loss * equal_episode_token_scale(indices.len(), e.target.len())?)?;
            let store = scaled.backward()?;
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let detached = g.detach();
                    if let Some(previous) = gradients.remove(name) {
                        gradients.insert(name.clone(), (&previous + &detached)?.detach());
                    } else {
                        gradients.insert(name.clone(), detached);
                    }
                }
            }
            trace_rows.push(if matches!(a.mode.as_str(), "composition-fit" | "context-fit" | "context-direction" | "readout-geometry-coadapt") {
                json!({"step":step,"target_label_only":target,"teacherforced_prefix_ids":&e.target[..step],"nll":loss,"native_target_probability":probability,"native_loss_equal":true,"actions":out.trace.actions})
            } else { json!({"step":step,"target_label_only":target,"teacherforced_prefix_ids":&e.target[..step],"nll":loss,"native_target_probability":probability,"native_loss_equal":true,"trace":out.trace}) });
        }
        let episode_mean = sum / e.target.len() as f64;
        mean += episode_mean / indices.len() as f64;
        rows.push(json!({"id":e.id,"original_source_ids":e.tokens,"source_record":e.record,"source_commit":e.commit,"source_view":e.view,"query_ids":e.query,"target_ids_labels_only":e.target,"mean_token_nll":episode_mean,"tokens":trace_rows}));
    }
    let mut square = 0f64;
    let mut families = BTreeMap::<String, f64>::new();
    let mut gradient_rows = Vec::new();
    for (name, g) in &gradients {
        let values = g.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid(format!("nonfinite gradient {name}")));
        }
        let l1: f64 = values.iter().map(|x| f64::from(x.abs())).sum();
        let sq: f64 = values.iter().map(|x| f64::from(*x).powi(2)).sum();
        square += sq;
        let family = name.split('.').take(2).collect::<Vec<_>>().join(".");
        *families.entry(family).or_default() += l1;
        gradient_rows.push(json!({"name":name,"l1":l1,"nonzero":values.iter().filter(|x|**x!=0.).count(),"elements":values.len()}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episode_indices":indices,"episodes":indices.len(),"tokens":tokens,"objective":"mean batch episodes(mean CE token+EOS)","mean_episode_nll":mean,"gradient_global_norm":square.sqrt(),"gradient_family_l1":families,"gradient_parameters":gradient_rows,"elapsed_seconds":begun.elapsed().as_secs_f64(),"rows":rows}),
    })
}
fn apply(
    weights: &SourceRealizerWeights,
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
        return Err(invalid("clip norm nonfinite"));
    }
    let factor = if norm > 1. { 1. / norm } else { 1. };
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    for (name, g) in gradients {
        let var = params
            .get(&name)
            .ok_or_else(|| invalid("gradient parameter absent"))?;
        store.insert(var.as_tensor(), (&g * factor)?.detach());
    }
    optimizer.step(&store)?;
    weights.project_quarter_range()?;
    Ok(factor)
}
fn generate(
    stage: &str,
    native: &NativeSourceRealizer,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    start: Instant,
    a: &Args,
) -> Result<Value> {
    let begun = Instant::now();
    let mut rows = Vec::new();
    let mut complete = 0usize;
    let mut eos_count = 0usize;
    for e in episodes {
        let mut prefix = Vec::<u32>::new();
        let mut traces = Vec::new();
        let mut ended = false;
        for step in 0..64 {
            deadline(start, a)?;
            if e.view.emitted_token_ids().len() + e.query.len() + prefix.len() > 128 {
                return Err(invalid("whole generated sequence exceeds128 admission"));
            }
            let trace = native.read(e.frame(), &e.view, &e.query, &prefix)?;
            let chosen = trace.actions.chosen_token_id;
            if chosen as usize >= 4096 {
                return Err(invalid("chosen action token outsideV4096"));
            }
            traces.push(if (a.mode == "transfer" && stage.starts_with("transfer-")) || ((a.mode == "composition-fit" && stage != "readout-baseline") || matches!(a.mode.as_str(), "context-fit" | "context-transplant" | "context-direction" | "context-frontier-direction" | "context-observation-learn" | "context-observable-cells" | "context-consumed-cells" | "context-later-query-cells" | "readout-geometry-coadapt" | "readout-geometry-transfer")) {
                json!({"step":step,"own_prefix_ids":prefix,"chosen_token_id":chosen,"actions":trace.actions})
            } else {
                json!({"step":step,"own_prefix_ids":prefix,"chosen_token_id":chosen,"trace":trace})
            });
            if chosen == native.binding().eos_token_id() {
                ended = true;
                eos_count += 1;
                break;
            }
            prefix.push(chosen);
        }
        let decoded = tok.decode(&prefix);
        let text = native.binding().protocol().reply_text(&decoded);
        let accepted = ended && e.accepted.iter().any(|v| v == text);
        let mut actual_ids = prefix.clone();
        if ended {
            actual_ids.push(native.binding().eos_token_id());
        }
        let first_divergence = if matches!(
            a.mode.as_str(),
            "transfer"
                | "composition-fit"
                | "context-fit"
                | "context-transplant"
                | "context-direction"
                | "context-frontier-direction"
                | "context-observation-learn"
                | "context-observable-cells"
                | "context-consumed-cells"
                | "context-later-query-cells"
                | "readout-geometry-coadapt"
                | "readout-geometry-transfer"
        ) {
            (0..actual_ids.len().max(e.target.len()))
                .find(|i| actual_ids.get(*i) != e.target.get(*i))
        } else {
            None
        };
        complete += usize::from(accepted);
        rows.push(json!({"id":e.id,"source_record":e.record,"source_commit":e.commit,"original_source_ids":e.tokens,"source_text":e.source_text,"source_view":e.view,"query_ids":e.query,"generated_ids":prefix,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(tok.decode_bytes(&prefix)),"raw_decoded_text_lossy":decoded,"raw_utf8_valid":String::from_utf8(tok.decode_bytes(&prefix)).is_ok(),"text_policy":"strip only protocol2 single leading content separator","eos":ended,"stop":if ended{"eos"}else{"max64"},"accepted_complete_answer":accepted,"source_mention_diagnostic_only":text.contains(&e.source_text),"tokens":traces}));
        if (a.mode == "transfer" && stage.starts_with("transfer-"))
            || ((a.mode == "composition-fit" && stage != "readout-baseline")
                || matches!(
                    a.mode.as_str(),
                    "context-fit"
                        | "context-transplant"
                        | "context-direction"
                        | "context-frontier-direction"
                        | "context-observation-learn"
                        | "context-observable-cells"
                        | "context-consumed-cells"
                        | "context-later-query-cells"
                        | "readout-geometry-coadapt"
                        | "readout-geometry-transfer"
                ))
        {
            if let Some(row) = rows.last_mut() {
                row["first_divergence_from_frozen_target"] = json!(first_divergence);
            }
        }
        write(
            &a.out.join(format!("{stage}-progress.json")),
            &if matches!(
                a.mode.as_str(),
                "context-observation-learn"
                    | "context-observable-cells"
                    | "context-consumed-cells"
                    | "context-later-query-cells"
                    | "readout-geometry-coadapt"
                    | "readout-geometry-transfer"
            ) {
                json!({"completed_cases":rows.len(),"complete_answers":complete,"eos_count":eos_count,"raw_rows_retained_in_enclosing_attempt":true,"elapsed_seconds":begun.elapsed().as_secs_f64()})
            } else {
                json!({"completed_cases":rows.len(),"rows":rows,"elapsed_seconds":begun.elapsed().as_secs_f64()})
            },
        )?;
    }
    Ok(
        json!({"scope":"own-prefix native selected-source generation; see enclosing report for panel identity; no parent scores or completechat qualification","cases":rows.len(),"complete_answers":complete,"eos_count":eos_count,"elapsed_seconds":begun.elapsed().as_secs_f64(),"rows":rows}),
    )
}
fn checkpoint(
    path: &Path,
    weights: &SourceRealizerWeights,
    identity: &ConsumerIdentity,
    tokenizer: &[u8],
    episodes: &[Episode],
    updates: usize,
    status: &str,
) -> Result<(Value, NativeSourceRealizer)> {
    report_output::claim(path)?;
    weights.save_source(&path.join("realizer-source"))?;
    let native = weights.compile(identity.clone())?;
    native.save(&path.join("realizer-native"))?;
    let source = SourceRealizerWeights::load_source(&path.join("realizer-source"), tokenizer)?;
    let loaded = NativeSourceRealizer::load(&path.join("realizer-native"), &source, identity)?;
    let mut rows = Vec::new();
    for e in episodes {
        let before = native.read(e.frame(), &e.view, &e.query, &[])?;
        let after = loaded.read(e.frame(), &e.view, &e.query, &[])?;
        if serde_json::to_value(&before)? != serde_json::to_value(&after)? {
            return Err(invalid("independent native realizer reload differs"));
        }
        rows.push(json!({"id":e.id,"trace_equal":true,"trace":after}));
    }
    let receipt = json!({"optimizer_updates":updates,"status":status,"reload_equal":true,"binding":loaded.binding(),"native_stats":loaded.stats(),"traces":rows});
    write(&path.join("checkpoint.json"), &receipt)?;
    report_output::seal(path)?;
    report_output::verify(path)?;
    Ok((receipt, loaded))
}
fn bin_files(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if entry.file_type()?.is_dir() {
                visit(root, &p, out)?;
            } else if p.extension().and_then(|x| x.to_str()) == Some("bin") {
                let relative = p
                    .strip_prefix(root)
                    .map_err(|e| invalid(e.to_string()))?
                    .to_string_lossy()
                    .into_owned();
                out.insert(relative, sha256_file(&p)?);
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out)?;
    Ok(out)
}
fn run(a: &Args, admitted: Option<&Admission>) -> Result<()> {
    let start = Instant::now();
    let source = source_commit()?;
    let (weights, identity, tok, episodes) = load(a, admitted)?;
    let bytes = fs::read(&a.tokenizer)?;
    let initial_native = weights.compile(identity.clone())?;
    initial_native.save(&a.out.join("initial-native"))?;
    let initial_bins = bin_files(&a.out.join("initial-native"))?;
    let mut updates = 0usize;
    let mut batches = Vec::<Value>::new();
    let mut saved_final = None::<Value>;
    let mut final_started = false;
    let mut gradient_family_l1 = BTreeMap::<String, f64>::new();
    write(
        &a.out.join("binding.json"),
        &json!({"source_commit":source,"retained_report_sha256":sha256_file(&a.retained_report)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"checkpoint_manifest_sha256":identity.parent_checkpoint_manifest_sha256,"parent_model_sha256_identity_only":identity.parent_model_sha256,"initial_construction_manifest_sha256":sha256_file(&a.initial_construction.join("manifest.json"))?,"period_seed":a.period_seed,"binding":weights.binding(),"optimizer":optimizer_identity(),"runtime_scope":"native selected-source Copy/Period/Stop, no parent scores/float normalization; offline loader identity is not whole-path serving qualification","executable_sha256":sha256_file(&std::env::current_exe()?)?}),
    )?;
    let work = (|| -> Result<()> {
        if a.mode == "construction" {
            let mut order: Vec<usize> = (0..episodes.len()).collect();
            order.sort_by(|&x, &y| {
                episodes[x]
                    .target
                    .len()
                    .cmp(&episodes[y].target.len())
                    .then_with(|| episodes[x].id.cmp(&episodes[y].id))
            });
            for (label, indices) in [
                ("shortest8", order[..8].to_vec()),
                ("longest8", order[order.len() - 8..].to_vec()),
            ] {
                let measured = batch(&indices, &episodes, &weights, &initial_native, start, a)?;
                if let Some(families) = measured.report["gradient_family_l1"].as_object() {
                    for (name, value) in families {
                        let v = value
                            .as_f64()
                            .ok_or_else(|| invalid("invalid gradient family report"))?;
                        *gradient_family_l1.entry(name.clone()).or_default() += v;
                    }
                }
                batches.push(json!({"kind":label,"optimizer_updates":0,"batch":measured.report}));
                write(
                    &a.out.join("progress.json"),
                    &json!({"mode":a.mode,"optimizer_updates":0,"batches":batches,"gradient_family_l1":gradient_family_l1,"wall_seconds":start.elapsed().as_secs_f64()}),
                )?;
            }
        } else {
            let initial = generate("initial", &initial_native, &episodes, &tok, start, a)?;
            write(&a.out.join("initial-generation.json"), &initial)?;
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
            for step in 0..UPDATES {
                deadline(start, a)?;
                let indices: Vec<usize> = (0..8).map(|i| (step * 8 + i) % episodes.len()).collect();
                let native = weights.compile(identity.clone())?;
                let measured = batch(&indices, &episodes, &weights, &native, start, a)?;
                if let Some(families) = measured.report["gradient_family_l1"].as_object() {
                    for (name, value) in families {
                        *gradient_family_l1.entry(name.clone()).or_default() += value
                            .as_f64()
                            .ok_or_else(|| invalid("invalid gradient family report"))?;
                    }
                }
                let clip = apply(&weights, &mut optimizer, measured.gradients)?;
                updates += 1;
                batches.push(
                    json!({"optimizer_update":updates,"clip_factor":clip,"batch":measured.report}),
                );
                write(
                    &a.out.join("progress.json"),
                    &json!({"mode":a.mode,"optimizer_updates":updates,"declared_updates":UPDATES,"batches":batches,"gradient_family_l1":gradient_family_l1,"wall_seconds":start.elapsed().as_secs_f64()}),
                )?;
                if updates % 16 == 0 {
                    checkpoint(
                        &a.out.join(format!("checkpoint-{updates:04}")),
                        &weights,
                        &identity,
                        &bytes,
                        &episodes,
                        updates,
                        "intermediate",
                    )?;
                }
            }
        }
        final_started = true;
        let (receipt, loaded) = checkpoint(
            &a.out.join("final-checkpoint"),
            &weights,
            &identity,
            &bytes,
            &episodes,
            updates,
            "updates_completed_before_generation",
        )?;
        saved_final = Some(receipt);
        let mut generation = generate("final", &loaded, &episodes, &tok, start, a)?;
        generation["native_loaded_from_disk"] = json!(true);
        generation["native_metadata_sha256"] = json!(sha256_file(
            &a.out.join("final-checkpoint/realizer-native/metadata.json")
        )?);
        write(&a.out.join("final-generation.json"), &generation)?;
        Ok(())
    })();
    let final_checkpoint = match saved_final {
        Some(receipt) => Ok(receipt),
        None if final_started => Err(invalid(
            "final checkpoint failed; partial directory retained without duplicate claim",
        )),
        None => checkpoint(
            &a.out.join("final-checkpoint"),
            &weights,
            &identity,
            &bytes,
            &episodes,
            updates,
            "stopped_or_error",
        )
        .map(|(receipt, _)| receipt),
    };
    let mut payload_changes = Vec::new();
    let mut changed = false;
    let mut inventory_equal = false;
    if final_checkpoint.is_ok() {
        let final_bins = bin_files(&a.out.join("final-checkpoint/realizer-native"))?;
        inventory_equal = initial_bins.keys().eq(final_bins.keys());
        for (file, sha) in &initial_bins {
            let final_sha = final_bins.get(file);
            let difference = final_sha != Some(sha);
            let coefficient = file != "consumer/exp-q31.bin";
            if coefficient && difference {
                changed = true;
            }
            payload_changes.push(json!({"file":file,"initial_sha256":sha,"final_sha256":final_sha,"changed":difference,"coefficient_payload":coefficient}));
        }
    }
    let cost_unchanged = a.mode != "construction" || (inventory_equal && !changed);
    let status = if work.is_ok() && final_checkpoint.is_ok() && inventory_equal && cost_unchanged {
        "completed"
    } else {
        "stopped_or_error"
    };
    let gradient_families_present = ["consumer.context", "consumer.potential", "consumer.no_read"]
        .iter()
        .all(|k| gradient_family_l1.get(*k).is_some_and(|v| *v > 0.))
        && gradient_family_l1
            .iter()
            .any(|(k, v)| k.starts_with("period.") && *v > 0.);
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-source-realizer/1","mode":a.mode,"status":status,"optimizer_updates":updates,"declared_updates":if a.mode=="fit"{UPDATES}else{0},"source_commit":source,"retained_report_sha256":sha256_file(&a.retained_report)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"checkpoint_manifest_sha256":identity.parent_checkpoint_manifest_sha256,"initial_construction_manifest_sha256":sha256_file(&a.initial_construction.join("manifest.json"))?,"period_seed":a.period_seed,"scope":"20 exposed development selected-source Copy/Period/Stop native component; no parent model scores in loss or generation; no heldout or complete-chat claim","no_parent_scores":true,"optimizer":optimizer_identity(),"batches":batches,"gradient_family_l1":gradient_family_l1,"all_active_gradient_families_present":gradient_families_present,"hard_payload_changes":payload_changes,"hard_inventory_equal":inventory_equal,"any_coefficient_payload_changed":if final_checkpoint.is_ok(){Some(changed)}else{None},"shadow_only_fit":if final_checkpoint.is_ok(){Some(updates>0&&!changed)}else{None},"construction_payload_unchanged":if a.mode=="construction"{Some(cost_unchanged)}else{None},"final_checkpoint":final_checkpoint.as_ref().ok(),"work_error":work.as_ref().err().map(|e|e.to_string()),"final_checkpoint_error":final_checkpoint.as_ref().err().map(|e|e.to_string()),"wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    work?;
    final_checkpoint?;
    if !inventory_equal || !cost_unchanged {
        return Err(invalid(
            "realizer native inventory or zero-update payload invariant failed",
        ));
    }
    Ok(())
}
fn main() -> Result<()> {
    let mut argv = std::env::args().skip(1);
    let p = argv
        .next()
        .ok_or_else(|| invalid("usage: geometric-source-realizer ARGS.json"))?;
    if argv.next().is_some() {
        return Err(invalid("one JSON argument required"));
    }
    let a: Args = serde_json::from_slice(&fs::read(p)?)?;
    if !matches!(
        a.mode.as_str(),
        "construction"
            | "fit"
            | "audit"
            | "direction"
            | "readout-fit"
            | "transfer"
            | "composition-fit"
            | "context-fit"
            | "context-transplant"
            | "context-direction"
            | "context-frontier-direction"
            | "context-observation-learn"
            | "context-observable-cells"
            | "context-consumed-cells"
            | "context-later-query-cells"
            | "readout-geometry-coadapt"
            | "readout-geometry-transfer"
            | "readout-geometry-query-diagnostic"
    ) || a.maximum_seconds == 0
        || (a.mode == "construction" && a.maximum_seconds > 300)
        || (a.mode == "fit" && a.maximum_seconds > 1200)
        || (a.mode == "audit"
            && (a.maximum_seconds > 300
                || a.audit_checkpoint.is_none()
                || a.fit_admission.is_some()))
        || (a.mode == "direction"
            && (a.maximum_seconds > 900
                || a.audit_checkpoint.is_none()
                || a.audit_report.is_none()
                || a.fit_admission.is_some()))
        || (a.mode == "readout-fit"
            && (a.maximum_seconds > 1200
                || a.audit_checkpoint.is_none()
                || a.expected_generation.is_none()
                || a.audit_report.is_some()
                || a.fit_admission.is_some()))
        || (a.mode == "transfer"
            && (a.maximum_seconds > 900
                || a.audit_checkpoint.is_none()
                || a.expected_generation.is_none()
                || a.transfer_checkpoint.is_none()
                || a.audit_report.is_some()
                || a.fit_admission.is_some()))
        || (matches!(
            a.mode.as_str(),
            "context-direction"
                | "context-frontier-direction"
                | "context-observation-learn"
                | "context-observable-cells"
                | "context-consumed-cells"
                | "context-later-query-cells"
        ) && (a.maximum_seconds > 900
            || a.audit_checkpoint.is_none()
            || a.transfer_checkpoint.is_none()
            || a.expected_generation.is_some()
            || a.audit_report.is_some()
            || a.fit_admission.is_some()))
        || (a.mode == "context-transplant"
            && (a.maximum_seconds > 300
                || a.audit_checkpoint.is_none()
                || a.transfer_checkpoint.is_none()
                || a.expected_generation.is_some()
                || a.audit_report.is_some()
                || a.fit_admission.is_some()))
        || (matches!(a.mode.as_str(), "composition-fit" | "context-fit")
            && (a.maximum_seconds > 1200
                || a.audit_checkpoint.is_none()
                || a.expected_generation.is_none()
                || a.transfer_checkpoint.is_none()
                || a.audit_report.is_some()
                || a.fit_admission.is_some()))
        || (!matches!(
            a.mode.as_str(),
            "transfer"
                | "composition-fit"
                | "context-fit"
                | "context-transplant"
                | "context-direction"
                | "context-frontier-direction"
                | "context-observation-learn"
                | "context-observable-cells"
                | "context-consumed-cells"
                | "context-later-query-cells"
        ) && a.transfer_checkpoint.is_some())
        || (!matches!(
            a.mode.as_str(),
            "audit"
                | "direction"
                | "readout-fit"
                | "transfer"
                | "composition-fit"
                | "context-fit"
                | "context-transplant"
                | "context-direction"
                | "context-frontier-direction"
                | "context-observation-learn"
                | "context-observable-cells"
                | "context-consumed-cells"
                | "context-later-query-cells"
        ) && (a.audit_checkpoint.is_some()
            || a.expected_generation.is_some()
            || a.audit_report.is_some()))
    {
        return Err(invalid(
            "construction/audit/context-transplant limit 1..300; direction/transfer/context-direction limit 1..900; fit/readout-fit/composition-fit/context-fit limit 1..1200 with fixed64 updates",
        ));
    }
    if matches!(
        a.mode.as_str(),
        "context-observable-cells" | "context-consumed-cells" | "context-later-query-cells"
    ) != a.observation_checkpoint.is_some()
    {
        return Err(invalid(
            "observable-cell mode requires sole observation_checkpoint field",
        ));
    }
    if (a.mode == "readout-geometry-coadapt") != a.geometry_checkpoint.is_some() {
        return Err(invalid(
            "geometry coadapt requires sole geometry_checkpoint field",
        ));
    }
    if a.mode == "readout-geometry-coadapt"
        && (a.maximum_seconds > 1800
            || a.fit_admission.is_none()
            || a.audit_checkpoint.is_some()
            || a.transfer_checkpoint.is_some()
            || a.expected_generation.is_some()
            || a.audit_report.is_some())
    {
        return Err(invalid(
            "geometry coadapt needs admission and max1800; old recursive parent fields excluded",
        ));
    }
    if (a.mode == "context-later-query-cells") != a.consumed_checkpoint.is_some() {
        return Err(invalid(
            "later-query mode requires sole consumed_checkpoint field",
        ));
    }
    if matches!(
        a.mode.as_str(),
        "readout-geometry-transfer" | "readout-geometry-query-diagnostic"
    ) != a.transfer_admission.is_some()
    {
        return Err(invalid(
            "geometry transfer requires its separate transfer_admission",
        ));
    }
    if matches!(
        a.mode.as_str(),
        "readout-geometry-transfer" | "readout-geometry-query-diagnostic"
    ) && (a.maximum_seconds > 900 || a.fit_admission.is_some())
    {
        return Err(invalid(
            "zero-update geometry transfer maximum900 and no optimizer admission",
        ));
    }
    if (a.mode == "readout-geometry-query-diagnostic") != a.query_diagnostic_admission.is_some()
        || (a.mode == "readout-geometry-query-diagnostic" && a.maximum_seconds > 300)
    {
        return Err(invalid(
            "query diagnostic requires sole query_diagnostic_admission and maximum300",
        ));
    }
    let admitted = admission(&a)?;
    if matches!(
        a.mode.as_str(),
        "audit"
            | "direction"
            | "readout-fit"
            | "transfer"
            | "composition-fit"
            | "context-fit"
            | "context-transplant"
            | "context-direction"
            | "context-frontier-direction"
            | "context-observation-learn"
            | "context-observable-cells"
            | "context-consumed-cells"
            | "context-later-query-cells"
    ) {
        audit_output_location(&a)?;
    }
    if a.mode == "readout-geometry-coadapt" {
        geometry_output_location(&a)?;
    }
    if matches!(
        a.mode.as_str(),
        "readout-geometry-transfer" | "readout-geometry-query-diagnostic"
    ) {
        frozen_transfer_output_location(&a)?;
        if a.mode == "readout-geometry-query-diagnostic" {
            query_diagnostic_output_location(&a)?;
        }
    }
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = if a.mode == "readout-geometry-query-diagnostic" {
        frozen_geometry_query_diagnostic(&a)
    } else if a.mode == "readout-geometry-transfer" {
        frozen_geometry_transfer(&a)
    } else if a.mode == "readout-geometry-coadapt" {
        geometry_readout_coadapt(&a)
    } else if matches!(
        a.mode.as_str(),
        "context-observation-learn"
            | "context-observable-cells"
            | "context-consumed-cells"
            | "context-later-query-cells"
    ) {
        context_observation_learn(&a)
    } else if matches!(
        a.mode.as_str(),
        "context-direction"
            | "context-frontier-direction"
            | "context-observation-learn"
            | "context-observable-cells"
            | "context-consumed-cells"
            | "context-later-query-cells"
    ) {
        context_direction(&a)
    } else if a.mode == "context-transplant" {
        context_transplant(&a)
    } else if a.mode == "transfer" {
        transfer(&a)
    } else if matches!(
        a.mode.as_str(),
        "readout-fit" | "composition-fit" | "context-fit"
    ) {
        readout_fit(&a)
    } else if a.mode == "direction" {
        direction(&a)
    } else if a.mode == "audit" {
        audit(&a)
    } else {
        run(&a, admitted.as_ref())
    };
    if let Err(e) = &result {
        write(&a.out.join("error.json"), &json!({"error":e.to_string()}))?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
#[derive(Deserialize)]
struct SavedRealizerIdentity {
    identity: ConsumerIdentity,
}
fn aggregate_rank(masses: &[(u32, u64)], target: u32) -> Result<Value> {
    let mut aggregate = BTreeMap::<u32, u64>::new();
    for &(token, mass) in masses {
        let old = aggregate.entry(token).or_default();
        *old = old
            .checked_add(mass)
            .ok_or_else(|| invalid("aggregate action mass overflow"))?;
    }
    let total = aggregate.values().try_fold(0u64, |sum, x| {
        sum.checked_add(*x)
            .ok_or_else(|| invalid("action total overflow"))
    })?;
    if total == 0 {
        return Err(invalid("empty action distribution"));
    }
    let target_mass = aggregate.get(&target).copied().unwrap_or(0);
    let higher = aggregate.values().filter(|m| **m > target_mass).count();
    let tie_before = aggregate
        .iter()
        .filter(|(id, mass)| **mass == target_mass && **id < target)
        .count();
    let mut best = None::<(u32, u64)>;
    for (&token, &mass) in &aggregate {
        if best.map_or(true, |(_, old)| mass > old) {
            best = Some((token, mass));
        }
    }
    let (top, top_mass) = best.ok_or_else(|| invalid("empty aggregated action candidates"))?;
    Ok(
        json!({"target_token_id":target,"target_mass_q31":target_mass,"total_mass_q31":total,"target_probability_diagnostic":target_mass as f64/total as f64,"target_rank_strict":higher+1,"target_rank_native_greedy":higher+tie_before+1,"equal_mass_smaller_token_count":tie_before,"strictly_higher_token_count":higher,"top_token_id":top,"top_mass_q31":top_mass,"target_supported":target_mass>0,"rank_scope":"aggregated token mass including action aliases; equal-mass ties receive equal strict rank"}),
    )
}
fn audit_output_location(a: &Args) -> Result<()> {
    // Existing output parent keeps normalization simple and prevents a failed
    // attempt from adding files beneath any immutable input envelope.
    let parent = a
        .out
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let output = fs::canonicalize(parent)?.join(
        a.out
            .file_name()
            .ok_or_else(|| invalid("output leaf missing"))?,
    );
    let path = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("audit checkpoint missing"))?;
    let roots = [
        path.parent().ok_or_else(|| invalid("fit root missing"))?,
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope missing"))?,
        a.checkpoint.as_path(),
    ];
    for root in roots {
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid("audit output must be outside sealed input roots"));
        }
    }
    if let Some(expected) = &a.expected_generation {
        if output.starts_with(fs::canonicalize(
            expected
                .parent()
                .ok_or_else(|| invalid("expected envelope missing"))?,
        )?) {
            return Err(invalid("audit output beneath expected envelope"));
        }
    }
    if let Some(report) = &a.audit_report {
        if output.starts_with(fs::canonicalize(
            report
                .parent()
                .ok_or_else(|| invalid("audit report envelope missing"))?,
        )?) {
            return Err(invalid("output beneath saved audit envelope"));
        }
    }
    if let Some(candidate) = &a.consumed_checkpoint {
        let root = candidate
            .parent()
            .ok_or_else(|| invalid("consumed continuation envelope absent"))?;
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid(
                "later-query output beneath consumed parent envelope",
            ));
        }
    }
    if let Some(candidate) = &a.observation_checkpoint {
        let root = candidate
            .parent()
            .ok_or_else(|| invalid("observation envelope absent"))?;
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid(
                "observable-cell output beneath retained parent envelope",
            ));
        }
    }
    if let Some(candidate) = &a.transfer_checkpoint {
        let root = candidate
            .parent()
            .ok_or_else(|| invalid("candidate envelope missing"))?;
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid("transfer output beneath candidate envelope"));
        }
    }
    Ok(())
}
fn scorer_features(trace: &RealizerTrace, geometry: &HistoricalH4Tables) -> Result<Value> {
    let replay = &trace.period_context;
    let m = replay
        .heads
        .checked_mul(replay.lanes_per_head)
        .ok_or_else(|| invalid("lane overflow"))?;
    let n = trace.source.emission_view.emitted_token_ids().len();
    let final_codes = replay
        .codes
        .get(
            replay
                .codes
                .len()
                .checked_sub(m)
                .ok_or_else(|| invalid("final codes absent"))?..,
        )
        .ok_or_else(|| invalid("final code range"))?;
    if final_codes.len() != m
        || n.checked_mul(m)
            .map_or(true, |end| end > replay.codes.len())
    {
        return Err(invalid("source feature replay shape"));
    }
    let mut copy = Vec::new();
    for occurrence in 0..n {
        let mut lanes = Vec::new();
        for (lane, q) in final_codes.iter().enumerate() {
            let k = &replay.codes[occurrence * m + lane];
            let present = q.present && k.present;
            let relative = if present {
                Some(
                    geometry
                        .relative(
                            H4Code::try_from(q.root).map_err(|e| invalid(e.to_string()))?,
                            H4Code::try_from(k.root).map_err(|e| invalid(e.to_string()))?,
                        )
                        .index(),
                )
            } else {
                None
            };
            lanes.push(json!({"context_presence":2*u8::from(q.present)+u8::from(k.present),"relative_h4":relative,"radius_index":if present {Some(32*usize::from(q.radius_bin)+usize::from(k.radius_bin))} else {None}}));
        }
        copy.push(lanes);
    }
    let last_token = replay
        .tokens
        .last()
        .ok_or_else(|| invalid("final token absent"))?;
    let last_latent = replay
        .states
        .last()
        .ok_or_else(|| invalid("final latent absent"))?;
    let categories = final_codes
        .iter()
        .map(|x| if x.present { x.radius_bin + 1 } else { 0 })
        .collect::<Vec<_>>();
    Ok(
        json!({"ordered_copy_tokens":trace.source.emission_view.emitted_token_ids(),"copy_content_presence_constant":0,"copy_age_constant":0,"copy_features":copy,"period_stop_last_token":last_token,"period_stop_all_latent_roots":last_latent,"period_stop_all_categories":categories,"period_stop_held":null}),
    )
}
fn add_signature(
    groups: &mut BTreeMap<String, Vec<Value>>,
    signature: &Value,
    witness: Value,
) -> Result<()> {
    groups
        .entry(serde_json::to_string(signature)?)
        .or_default()
        .push(witness);
    Ok(())
}
fn conflicts(groups: BTreeMap<String, Vec<Value>>) -> Vec<Value> {
    groups
        .into_iter()
        .filter_map(|(signature, rows)| {
            let first = rows.first()?;
            rows.iter()
                .any(|row| row["target"] != first["target"])
                .then(|| json!({"signature_json":signature,"rows":rows}))
        })
        .collect()
}
struct LoadedFinal {
    source: SourceRealizerWeights,
    native: NativeSourceRealizer,
    identity: ConsumerIdentity,
    tok: ByteBpeTokenizer,
    episodes: Vec<Episode>,
    fit: Value,
    retained_sha: String,
    before: BTreeMap<String, String>,
}
fn load_final(a: &Args) -> Result<LoadedFinal> {
    let path = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("audit checkpoint missing"))?;
    if path.file_name().and_then(|x| x.to_str()) != Some("final-checkpoint") {
        return Err(invalid("audit requires the named final-checkpoint"));
    }
    report_output::verify(path)?;
    let fit_root = path
        .parent()
        .ok_or_else(|| invalid("fit input root missing"))?;
    report_output::verify(fit_root)?;
    let fit: Value = serde_json::from_slice(&fs::read(fit_root.join("report.json"))?)?;
    let receipt: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    if fit["schema"] != "uor-r4.geometric-source-realizer/1"
        || fit["mode"] != "fit"
        || fit["status"] != "completed"
        || fit["optimizer_updates"] != 64
        || receipt["optimizer_updates"] != 64
    {
        return Err(invalid(
            "audit requires completed final64 fit checkpoint, not pre-update batch traces",
        ));
    }
    let saved: SavedRealizerIdentity =
        serde_json::from_slice(&fs::read(path.join("realizer-native/metadata.json"))?)?;
    let identity = saved.identity;
    if sha256_file(&a.tokenizer)? != identity.tokenizer_sha256
        || sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
            != identity.parent_checkpoint_manifest_sha256
        || sha256_file(&a.checkpoint.join("model.safetensors"))? != identity.parent_model_sha256
        || sha256_file(&a.checkpoint.join("config.json"))? != identity.parent_config_sha256
    {
        return Err(invalid("saved realizer parent/tokenizer identity differs"));
    }
    let retained_sha = sha256_file(&a.retained_report)?;
    if fit["retained_report_sha256"] != retained_sha
        || fit["tokenizer_sha256"] != identity.tokenizer_sha256
        || fit["checkpoint_manifest_sha256"] != identity.parent_checkpoint_manifest_sha256
    {
        return Err(invalid("fit report input binding differs"));
    }
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope missing"))?,
    )?;
    let retained: Report = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if retained.schema != "uor-r4.source-binding-diagnostic/1"
        || retained.tokenizer_sha256 != identity.tokenizer_sha256
        || retained.checkpoint_manifest_sha256 != identity.parent_checkpoint_manifest_sha256
    {
        return Err(invalid("retained frame binding differs"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("audit tokenizer unreadable"))?;
    let before = bin_files(&path.join("realizer-native"))?;
    let source = SourceRealizerWeights::load_source(&path.join("realizer-source"), &bytes)?;
    let native = NativeSourceRealizer::load(&path.join("realizer-native"), &source, &identity)?;
    if native.binding().protocol().schema != SCHEMA_V2
        || native.binding().tokenizer_sha256() != identity.tokenizer_sha256
        || native.binding().vocab_size() != 4096
    {
        return Err(invalid("audit source/action binding differs"));
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let episodes = prepare(retained, &tok, native.binding().eos_token_id(), &compiler)?;
    Ok(LoadedFinal {
        source,
        native,
        identity,
        tok,
        episodes,
        fit,
        retained_sha,
        before,
    })
}
fn audit(a: &Args) -> Result<()> {
    let start = Instant::now();
    let LoadedFinal {
        source: _source,
        native,
        identity,
        tok,
        episodes,
        fit,
        retained_sha,
        before,
    } = load_final(a)?;
    let path = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("audit checkpoint missing"))?;
    let fit_root = path.parent().ok_or_else(|| invalid("fit root missing"))?;
    let mut rows = Vec::new();
    let mut targets = 0usize;
    let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
        "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
    ))
    .map_err(|e| invalid(e.to_string()))?;
    let mut feature_groups = BTreeMap::new();
    let mut q24_groups = BTreeMap::new();
    let mut q31_groups = BTreeMap::new();
    let mut aliases = Vec::new();
    let mut compact = Vec::new();
    for e in &episodes {
        let mut token_rows = Vec::new();
        for (step, &target) in e.target.iter().enumerate() {
            deadline(start, a)?;
            let trace = native.read(e.frame(), &e.view, &e.query, &e.target[..step])?;
            let masses = trace
                .actions
                .token_masses
                .iter()
                .map(|m| (m.token_id, m.weight_q31))
                .collect::<Vec<_>>();
            let rank = aggregate_rank(&masses, target)?;
            if rank["total_mass_q31"] != trace.actions.total_weight_q31 {
                return Err(invalid(
                    "aggregate integer target diagnostic denominator differs",
                ));
            }
            if rank["top_token_id"] != trace.actions.chosen_token_id {
                return Err(invalid("target diagnostic greedy tie law differs"));
            }
            let features = scorer_features(&trace, &geometry)?;
            let witness = json!({"id":e.id,"step":step,"target":target});
            add_signature(&mut feature_groups, &features, witness.clone())?;
            let q24 = json!(trace
                .actions
                .actions
                .iter()
                .map(|x| (x.token_id, x.score_q24))
                .collect::<Vec<_>>());
            let q31 = json!(trace
                .actions
                .token_masses
                .iter()
                .map(|x| (x.token_id, x.weight_q31))
                .collect::<Vec<_>>());
            add_signature(&mut q24_groups, &q24, witness.clone())?;
            add_signature(&mut q31_groups, &q31, witness)?;
            let alias_rows = trace
                .actions
                .token_masses
                .iter()
                .filter(|x| x.action_offsets.len() > 1)
                .collect::<Vec<_>>();
            if !alias_rows.is_empty() {
                aliases.push(json!({"id":e.id,"step":step,"token_aliases":alias_rows}));
            }
            compact.push(json!({"id":e.id,"step":step,"target":target,"chosen":trace.actions.chosen_token_id,"rank":rank["target_rank_native_greedy"],"p":rank["target_probability_diagnostic"]}));
            token_rows.push(json!({"step":step,"target_label_only":target,"teacherforced_canonical_prefix_ids":&e.target[..step],"chosen_token_id":trace.actions.chosen_token_id,"target_rank_probability":rank,"scorer_features":features,"trace":trace}));
            targets += 1;
        }
        rows.push(json!({"id":e.id,"original_source_ids":e.tokens,"source_record":e.record,"source_commit":e.commit,"source_view":e.view,"query_ids":e.query,"target_ids_labels_only":e.target,"tokens":token_rows}));
        write(
            &a.out.join("canonical-progress.json"),
            &json!({"rows":rows,"completed_cases":rows.len(),"target_steps":targets,"optimizer_updates":0}),
        )?;
    }
    let generation = generate("audit-ownprefix", &native, &episodes, &tok, start, a)?;
    write(&a.out.join("audit-ownprefix-generation.json"), &generation)?;
    let native_metadata_sha = sha256_file(&path.join("realizer-native/metadata.json"))?;
    let expected_equal = if let Some(expected_path) = &a.expected_generation {
        report_output::verify(
            expected_path
                .parent()
                .ok_or_else(|| invalid("expected generation envelope missing"))?,
        )?;
        let expected: Value = serde_json::from_slice(&fs::read(expected_path)?)?;
        if expected["native_loaded_from_disk"] != true
            || expected["native_metadata_sha256"] != native_metadata_sha
        {
            return Err(invalid(
                "expected generation belongs to a different/pre-update artifact",
            ));
        }
        let previous = expected["rows"]
            .as_array()
            .ok_or_else(|| invalid("expected generation rows missing"))?;
        let actual = generation["rows"]
            .as_array()
            .ok_or_else(|| invalid("audit generation rows missing"))?;
        if previous.len() != actual.len() {
            return Err(invalid("own-prefix reload case count mismatch"));
        }
        for (old, new) in previous.iter().zip(actual) {
            if old != new {
                return Err(invalid(format!(
                    "saved final own-prefix complete row differs for {}",
                    new["id"]
                )));
            }
            for key in [
                "id",
                "source_record",
                "source_commit",
                "original_source_ids",
                "source_view",
                "query_ids",
                "generated_ids",
                "eos",
                "stop",
                "raw_decoded_bytes_hex",
                "reply_text",
                "tokens",
            ] {
                if old.get(key).is_none() || new.get(key).is_none() || old[key] != new[key] {
                    return Err(invalid(format!(
                        "saved final own-prefix replay differs for {} field {key}",
                        new["id"]
                    )));
                }
            }
        }
        Some(true)
    } else {
        None
    };
    let after = bin_files(&path.join("realizer-native"))?;
    if before != after {
        return Err(invalid("audit changed saved native packed payloads"));
    }
    report_output::verify(path)?;
    report_output::verify(fit_root)?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-source-realizer-audit/1","mode":"audit","status":"completed","optimizer_updates":0,"loaded_artifact_optimizer_updates":64,"source_commit":source_commit()?,"fit_source_commit":fit["source_commit"],"executable_sha256":sha256_file(&std::env::current_exe()?)?,"checkpoint_manifest_sha256":sha256_file(&path.join("manifest.json"))?,"fit_manifest_sha256":sha256_file(&fit_root.join("manifest.json"))?,"retained_report_sha256":retained_sha,"tokenizer_sha256":identity.tokenizer_sha256,"saved_consumer_identity":identity,"native_metadata_sha256":native_metadata_sha,"packed_files_before":before,"packed_files_after":after,"packed_files_unchanged":true,"parent_model_loaded_or_scored":false,"explicit_compile_reexport_or_optimizer":false,"loader_scope":"source/native admission may reconstruct validation tables internally; numeric generation executes saved packed payloads","scope":"all20exposed canonical teacherforced prefix diagnostic plus ownprefix final64 native artifact replay; not pre-update64 training traces, heldout, or completechat","workload_host":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"metal_tests_run":false},"target_steps":targets,"compact_target_summary":compact,"full_feature_conflicts":conflicts(feature_groups),"equal_q24_conflicting_targets":conflicts(q24_groups),"equal_q31_conflicting_targets":conflicts(q31_groups),"token_alias_rows":aliases,"signature_scope":"current scorer features only; record/commit/relation/offset/prefix length are provenance, not scoring features; canonical labels only; equal quantized scores do not prove structural collisions","canonical_rows":rows,"ownprefix_generation":generation,"expected_final_generation_exact_replay":expected_equal,"wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}
#[cfg(test)]
mod audit_tests {
    use super::*;
    #[test]
    fn alias_mass_controls_token_rank() -> Result<()> {
        let r = aggregate_rank(&[(16, 3), (16, 5), (7, 7), (1, 2)], 16)?;
        assert_eq!(r["target_mass_q31"], 8);
        assert_eq!(r["total_mass_q31"], 17);
        assert_eq!(r["target_rank_strict"], 1);
        assert_eq!(r["top_token_id"], 16);
        Ok(())
    }
    #[test]
    fn absent_target_has_zero_probability() -> Result<()> {
        let r = aggregate_rank(&[(5, 4), (1, 4)], 9)?;
        assert_eq!(r["target_mass_q31"], 0);
        assert_eq!(r["target_probability_diagnostic"], 0.);
        assert_eq!(r["target_rank_strict"], 3);
        Ok(())
    }
    #[test]
    fn native_greedy_ties_use_smallest_token_id() -> Result<()> {
        let r = aggregate_rank(&[(5, 4), (1, 4)], 5)?;
        assert_eq!(r["top_token_id"], 1);
        assert_eq!(r["target_rank_strict"], 1);
        assert_eq!(r["target_rank_native_greedy"], 2);
        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct DirectionCoordinate {
    name: String,
    index: usize,
    gradient: f64,
    original_shadow: f32,
    original_q: i8,
    calibration: bool,
    eligibility: Vec<Value>,
}
fn select_direction(mut coordinates: Vec<DirectionCoordinate>) -> Vec<DirectionCoordinate> {
    coordinates.sort_by(|a, b| {
        b.gradient
            .abs()
            .total_cmp(&a.gradient.abs())
            .then(a.name.cmp(&b.name))
            .then(a.index.cmp(&b.index))
    });
    coordinates.truncate(4);
    coordinates
}
fn quantum_change(before: &[u8], after: &[u8], expected: usize, delta: i8) -> Result<Value> {
    if before.len() != after.len() {
        return Err(invalid("packed potential length changed"));
    }
    let mut changes = Vec::new();
    for (index, (&a, &b)) in before.iter().zip(after).enumerate() {
        for half in 0..2 {
            let decode = |x: u8| {
                let n = ((x >> (half * 4)) & 15) as i8;
                if n >= 8 {
                    n - 16
                } else {
                    n
                }
            };
            let old = decode(a);
            let new = decode(b);
            if old != new {
                changes.push((index * 2 + half, old, new));
            }
        }
    }
    if changes.len() != 1
        || changes[0].0 != expected
        || changes[0].2 - changes[0].1 != delta
        || !(-7..=7).contains(&changes[0].2)
    {
        return Err(invalid(
            "candidate must change exactly selected legal quarter quantum",
        ));
    }
    Ok(
        json!({"packed_coefficient_index":expected,"old_q":changes[0].1,"new_q":changes[0].2,"exact_one_quantum":true}),
    )
}
fn fixed_feature(
    features: &Value,
    name: &str,
    index: usize,
    occurrence: usize,
    lanes: usize,
) -> Result<f64> {
    let family = name
        .strip_prefix("consumer.potential.")
        .ok_or_else(|| invalid("non-potential coordinate"))?;
    let width = match family {
        "content_unary" | "context_unary" | "content_presence" | "context_presence" => 4,
        "pair" => 16,
        "content_radius" | "context_radius" => 1024,
        _ => return Err(invalid("unknown potential family")),
    };
    let lane = index / width;
    let local = index % width;
    if lane >= lanes {
        return Err(invalid("feature coordinate lane out of bounds"));
    }
    let row = &features["copy_features"][occurrence][lane];
    Ok(match family {
        "content_presence" => f64::from(u8::from(local == 0)),
        "context_presence" => f64::from(u8::from(
            row["context_presence"].as_u64() == Some(local as u64),
        )),
        "context_radius" => f64::from(u8::from(row["radius_index"].as_u64() == Some(local as u64))),
        "context_unary" => match row["relative_h4"].as_u64() {
            Some(root) => {
                f64::from(
                    uor_r4_integer::geometric_potential_q4::canonical_basis_q25()[root as usize]
                        [local],
                ) / 33_554_432.
            }
            None => 0.,
        },
        _ => 0.,
    })
}
fn copy_margin(trace: &RealizerTrace, n: usize, target: u32) -> Result<Value> {
    let copy = &trace.actions.actions[..n];
    let mut masses = BTreeMap::<u32, u64>::new();
    for action in copy {
        let mass = masses.entry(action.token_id).or_default();
        *mass = mass
            .checked_add(action.weight_q31)
            .ok_or_else(|| invalid("Copy alias mass overflow"))?;
    }
    let mut best = None;
    for (&token, &mass) in &masses {
        if best.map_or(true, |(_, old)| mass > old) {
            best = Some((token, mass));
        }
    }
    let target_score = copy
        .iter()
        .filter(|x| x.token_id == target)
        .map(|x| x.score_q24)
        .max();
    let other_score = copy
        .iter()
        .filter(|x| x.token_id != target)
        .map(|x| x.score_q24)
        .max();
    let margin = match (target_score, other_score) {
        (Some(t), Some(o)) => Some(
            t.checked_sub(o)
                .ok_or_else(|| invalid("Copy Q24margin overflow"))?,
        ),
        _ => None,
    };
    let target_mass = masses.get(&target).copied().unwrap_or(0);
    let rank = if masses.contains_key(&target) {
        Some(
            1 + masses
                .iter()
                .filter(|(token, mass)| {
                    **mass > target_mass || (**mass == target_mass && **token < target)
                })
                .count(),
        )
    } else {
        None
    };
    Ok(
        json!({"best_copy_token_id":best.map(|x|x.0),"target_copy_rank_native_ties":rank,"target_copy_max_q24":target_score,"best_other_copy_max_q24":other_score,"target_minus_best_other_copy_q24":margin,"copy_token_masses":masses,"scope":"raw summed-Q24 max target occurrence minus max non-target Copy occurrence; null for missing target/otherCopy; alias token rank uses Copy-only aggregated action masses"}),
    )
}
fn canonical_measure(
    native: &NativeSourceRealizer,
    episodes: &[Episode],
    start: Instant,
    a: &Args,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut objective = 0.;
    let mut steps = 0;
    for e in episodes {
        let mut tokens = Vec::new();
        let mut sum = 0.;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(start, a)?;
            let trace = native.read(e.frame(), &e.view, &e.query, &e.target[..step])?;
            let masses = trace
                .actions
                .token_masses
                .iter()
                .map(|x| (x.token_id, x.weight_q31))
                .collect::<Vec<_>>();
            let diagnostic = aggregate_rank(&masses, target)?;
            let target_mass = diagnostic["target_mass_q31"]
                .as_u64()
                .ok_or_else(|| invalid("target mass absent"))?;
            let other = trace
                .actions
                .token_masses
                .iter()
                .filter(|x| x.token_id != target)
                .map(|x| x.weight_q31)
                .max()
                .unwrap_or(0);
            let margin = i128::from(target_mass) - i128::from(other);
            let margin =
                i64::try_from(margin).map_err(|_| invalid("mass margin exceeds report range"))?;
            let p = diagnostic["target_probability_diagnostic"]
                .as_f64()
                .ok_or_else(|| invalid("target probability absent"))?;
            if p <= 0. {
                return Err(invalid("canonical target has zero native mass"));
            }
            let ce = -p.ln();
            sum += ce;
            steps += 1;
            let copy = copy_margin(&trace, e.view.emitted_token_ids().len(), target)?;
            tokens.push(json!({"step":step,"target":target,"native_ce":ce,"target_mass_margin_q31":margin,"copy":copy,"target_rank_probability":diagnostic,"trace":trace}));
        }
        objective += sum / e.target.len() as f64 / episodes.len() as f64;
        rows.push(json!({"id":e.id,"tokens":tokens}));
    }
    Ok(json!({"mean_episode_ce":objective,"target_steps":steps,"rows":rows}))
}
// Retained source weights initialize this continuation; optimizer moments were
// not saved by the original fit, so AdamW starts fresh on these readouts only.
fn readout_parameter(name: &str) -> bool {
    name.starts_with("consumer.potential.")
        || name.starts_with("consumer.no_read.")
        || name.starts_with("period.")
}
fn readout_gradients(gradients: BTreeMap<String, Tensor>) -> Result<BTreeMap<String, Tensor>> {
    if gradients
        .keys()
        .any(|n| !readout_parameter(n) && !n.starts_with("consumer.context."))
    {
        return Err(invalid("unexpected readout continuation gradient family"));
    }
    Ok(gradients
        .into_iter()
        .filter(|(n, _)| readout_parameter(n))
        .collect())
}
fn context_gradients(gradients: BTreeMap<String, Tensor>) -> Result<BTreeMap<String, Tensor>> {
    if gradients
        .keys()
        .any(|n| !readout_parameter(n) && !n.starts_with("consumer.context."))
    {
        return Err(invalid("unexpected context adaptation gradient family"));
    }
    Ok(gradients)
}
fn source_files(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if entry.file_type()?.is_dir() {
                visit(root, &p, out)?;
            } else {
                let relative = p
                    .strip_prefix(root)
                    .map_err(|e| invalid(e.to_string()))?
                    .to_string_lossy()
                    .into_owned();
                out.insert(relative, sha256_file(&p)?);
            }
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result)?;
    Ok(result)
}
fn changed_file_bytes(before: &Path, after: &Path) -> Result<usize> {
    let before = fs::read(before)?;
    let after = fs::read(after)?;
    if before.len() != after.len() {
        return Err(invalid("native context dimensions changed"));
    }
    Ok(before.iter().zip(&after).filter(|(a, b)| a != b).count())
}
fn context_bit_changes(
    before: &BTreeMap<String, Vec<u32>>,
    after: &BTreeMap<String, Vec<u32>>,
) -> Result<BTreeMap<String, usize>> {
    if before.is_empty() || before.keys().ne(after.keys()) {
        return Err(invalid("context source inventory differs"));
    }
    before
        .iter()
        .map(|(name, bits)| {
            let current = after
                .get(name)
                .ok_or_else(|| invalid("context source family absent"))?;
            if bits.len() != current.len() {
                return Err(invalid("context source dimension differs"));
            }
            Ok((
                name.clone(),
                bits.iter().zip(current).filter(|(a, b)| a != b).count(),
            ))
        })
        .collect()
}
fn context_shadow(weights: &SourceRealizerWeights) -> Result<BTreeMap<String, Vec<u32>>> {
    weights
        .parameters()
        .into_iter()
        .filter(|(n, _)| n.starts_with("consumer.context."))
        .map(|(n, v)| {
            Ok((
                n,
                v.flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(f32::to_bits)
                    .collect(),
            ))
        })
        .collect()
}
fn verify_context_shadow(
    weights: &SourceRealizerWeights,
    before: &BTreeMap<String, Vec<u32>>,
) -> Result<()> {
    if before.is_empty() || context_shadow(weights)? != *before {
        return Err(invalid("frozen context source bits changed"));
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometryReadoutAdmission {
    source_commit: String,
    geometry_run_source_commit: String,
    geometry_run_manifest_sha256: String,
    geometry_checkpoint_manifest_sha256: String,
    geometry_result_sha256: String,
    round: usize,
    candidate: usize,
    optimizer: OptimizerIdentity,
    maximum_seconds: u64,
    starting_role: String,
}
fn readout_batch_admitted(mode: &str, indices: &[usize], episodes: usize) -> bool {
    if mode == "readout-geometry-coadapt" {
        episodes == 28 && indices == (0..28).collect::<Vec<_>>()
    } else {
        indices.len() == 8
    }
}
fn equal_episode_token_scale(episodes: usize, tokens: usize) -> Result<f64> {
    if episodes == 0 || tokens == 0 {
        return Err(invalid("empty episode normalization"));
    }
    let count = episodes
        .checked_mul(tokens)
        .ok_or_else(|| invalid("episode normalization overflow"))?;
    Ok(1. / count as f64)
}
fn geometry_parent_header(
    admission: &GeometryReadoutAdmission,
    report: &Value,
    result: &Value,
    receipt: &Value,
    identity: &ConsumerIdentity,
    retained_sha: &str,
) -> Result<()> {
    if admission.source_commit != source_commit()?
        || admission.optimizer != optimizer_identity()
        || admission.maximum_seconds == 0
        || admission.maximum_seconds > 1800
        || admission.starting_role != "retained_main_candidate_new_research_parent"
        || report["schema"] != "uor-r4.geometric-later-query-cells/1"
        || report["mode"] != "context-later-query-cells"
        || report["status"] != "completed"
        || report["source_commit"] != admission.geometry_run_source_commit
        || report["saved_identity"] != serde_json::to_value(identity)?
        || report["retained_report_sha256"] != retained_sha
        || report["transitions_readouts_tables_fixed"] != true
        || report["no_adopted_model"] != true
        || result["ablation_only"] != false
        || result["readouts_tables_transitions_fixed"] != true
        || result["all_other_source_bits_fixed"] != true
        || result["checkpoint"] != *receipt
        || receipt["optimizer_updates"] != 0
    {
        return Err(invalid("geometry readout parent/admission binding differs"));
    }
    Ok(())
}
fn geometry_run_root(path: &Path) -> Result<&Path> {
    path.parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| invalid("geometry candidate run root absent"))
}
fn geometry_output_location(a: &Args) -> Result<()> {
    let path = a
        .geometry_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("geometry checkpoint absent"))?;
    let parent = a
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let output = fs::canonicalize(parent)?.join(
        a.out
            .file_name()
            .ok_or_else(|| invalid("output leaf absent"))?,
    );
    for root in [
        geometry_run_root(path)?,
        a.checkpoint.as_path(),
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope absent"))?,
    ] {
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid("geometry output beneath sealed input"));
        }
    }
    Ok(())
}
struct FrozenGeometryStart {
    source: SourceRealizerWeights,
    native: NativeSourceRealizer,
    identity: ConsumerIdentity,
    tok: ByteBpeTokenizer,
    original: Vec<Episode>,
    construction: Vec<Episode>,
    expected: Value,
    bins: BTreeMap<String, String>,
    binding: Value,
}
fn load_frozen_geometry(a: &Args) -> Result<FrozenGeometryStart> {
    let path = a
        .geometry_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("geometry candidate checkpoint absent"))?;
    let admission: GeometryReadoutAdmission = serde_json::from_slice(&fs::read(
        a.fit_admission
            .as_ref()
            .ok_or_else(|| invalid("geometry readout admission absent"))?,
    )?)?;
    let root = geometry_run_root(path)?;
    let expected_path = root.join(format!(
        "round-{:02}/candidate-{:02}/checkpoint",
        admission.round, admission.candidate
    ));
    if fs::canonicalize(path)? != fs::canonicalize(&expected_path)?
        || admission.maximum_seconds != a.maximum_seconds
    {
        return Err(invalid(
            "declared geometry starting candidate path/budget differs",
        ));
    }
    report_output::verify(root)?;
    report_output::verify(path)?;
    let candidate_root = path
        .parent()
        .ok_or_else(|| invalid("candidate envelope absent"))?;
    let result_path = candidate_root.join("result.json");
    if sha256_file(&root.join("manifest.json"))? != admission.geometry_run_manifest_sha256
        || sha256_file(&path.join("manifest.json"))?
            != admission.geometry_checkpoint_manifest_sha256
        || sha256_file(&result_path)? != admission.geometry_result_sha256
    {
        return Err(invalid(
            "geometry exact source/result/checkpoint hashes differ",
        ));
    }
    let report: Value = serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    let receipt: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    let saved: SavedRealizerIdentity =
        serde_json::from_slice(&fs::read(path.join("realizer-native/metadata.json"))?)?;
    let identity = saved.identity;
    let retained_sha = sha256_file(&a.retained_report)?;
    geometry_parent_header(
        &admission,
        &report,
        &result,
        &receipt,
        &identity,
        &retained_sha,
    )?;
    let round_root = root.join(format!("round-{:02}", admission.round));
    report_output::verify(&round_root)?;
    report_output::verify(candidate_root)?;
    let round_entry = report["rounds"]
        .as_array()
        .and_then(|v| v.get(admission.round))
        .ok_or_else(|| invalid("declared geometry round absent"))?;
    if round_entry["round"] != admission.round
        || round_entry["report_sha256"] != sha256_file(&round_root.join("report.json"))?
    {
        return Err(invalid("geometry round report binding differs"));
    }
    let round_report: Value = serde_json::from_slice(&fs::read(round_root.join("report.json"))?)?;
    let main_count = round_report["main_proposal_count"]
        .as_u64()
        .ok_or_else(|| invalid("geometry main candidate count absent"))?
        as usize;
    let summary = round_report["proposals"]
        .as_array()
        .and_then(|v| v.get(admission.candidate))
        .ok_or_else(|| invalid("declared geometry main candidate absent"))?;
    if admission.candidate >= main_count
        || summary["index"] != admission.candidate
        || summary["ablation_only"] != false
        || summary["result_sha256"] != admission.geometry_result_sha256
        || summary["round_frozen_ce"] != result["frontier"]["mean_episode_ce"]
    {
        return Err(invalid(
            "geometry candidate is not source-bound prospective main result",
        ));
    }
    if sha256_file(&a.tokenizer)? != identity.tokenizer_sha256
        || sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
            != identity.parent_checkpoint_manifest_sha256
        || sha256_file(&a.checkpoint.join("model.safetensors"))? != identity.parent_model_sha256
        || sha256_file(&a.checkpoint.join("config.json"))? != identity.parent_config_sha256
    {
        return Err(invalid(
            "geometry original identity/tokenizer/config differs",
        ));
    }
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope absent"))?,
    )?;
    let retained: Report = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if retained.schema != "uor-r4.source-binding-diagnostic/1"
        || retained.tokenizer_sha256 != identity.tokenizer_sha256
        || retained.checkpoint_manifest_sha256 != identity.parent_checkpoint_manifest_sha256
    {
        return Err(invalid("geometry retained diagnostic identity differs"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("geometry tokenizer unreadable"))?;
    let source = SourceRealizerWeights::load_source(&path.join("realizer-source"), &bytes)?;
    let native = NativeSourceRealizer::load(&path.join("realizer-native"), &source, &identity)?;
    let bins = bin_files(&path.join("realizer-native"))?;
    if bins
        != serde_json::from_value::<BTreeMap<String, String>>(
            result["native_payload_sha256"].clone(),
        )?
        || native.binding().protocol().schema != SCHEMA_V2
        || native.binding().vocab_size() != 4096
    {
        return Err(invalid(
            "geometry native payload/protocol/vocabulary differs",
        ));
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let original = prepare(retained, &tok, native.binding().eos_token_id(), &compiler)?;
    let construction = composition_panel(
        original
            .first()
            .ok_or_else(|| invalid("geometry original cases absent"))?,
        &tok,
        &compiler,
        native.binding().eos_token_id(),
    )?;
    let panel: Value = serde_json::from_slice(&fs::read(root.join("frozen-panel.json"))?)?;
    if original.len() != 20
        || construction.len() != 8
        || panel
            != json!({"original":episode_labels(&original),"construction":episode_labels(&construction)})
    {
        return Err(invalid("geometry frozen28 panel differs"));
    }
    let binding = json!({"admission":admission,"geometry_run_report_sha256":sha256_file(&root.join("report.json"))?,"geometry_round_report_sha256":round_entry["report_sha256"],"geometry_result_sha256":sha256_file(&result_path)?,"checkpoint_manifest_sha256":sha256_file(&path.join("manifest.json"))?,"original_identity":identity,"retained_report_sha256":retained_sha,"runtime_parent_scores_used":false,"transitive_old_parents":"hash receipts preserved; not opened by this loader","new_research_parent_not_default_or_retrospective_winner":true});
    Ok(FrozenGeometryStart {
        source,
        native,
        identity,
        tok,
        original,
        construction,
        expected: result,
        bins,
        binding,
    })
}
fn frozen_readout_source_bits_fixed(
    before: &BTreeMap<String, Vec<u32>>,
    after: &BTreeMap<String, Vec<u32>>,
) -> bool {
    !before.is_empty()
        && before.keys().eq(after.keys())
        && before
            .iter()
            .all(|(name, bits)| readout_parameter(name) || after.get(name) == Some(bits))
}
fn readout_native_payloads_fixed(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> bool {
    before.keys().eq(after.keys())
        && before.iter().all(|(name, sha)| {
            [
                "consumer/potential-q4.bin",
                "consumer/no-read-q4.bin",
                "period-q4.bin",
            ]
            .contains(&name.as_str())
                || after.get(name) == Some(sha)
        })
}
fn best_finite_readout_checkpoint(scores: &[Option<f64>]) -> Option<usize> {
    let mut best = None;
    for (index, score) in scores.iter().enumerate() {
        if let Some(score) = score.filter(|v| v.is_finite()) {
            if best.is_none_or(|(_, previous)| score < previous) {
                best = Some((index, score));
            }
        }
    }
    best.map(|(index, _)| index)
}
fn full28_native_ce(original: &Value, construction: &Value) -> Result<Option<f64>> {
    if !original["rows"]
        .as_array()
        .is_some_and(|rows| rows.len() == 20)
        || !construction["rows"]
            .as_array()
            .is_some_and(|rows| rows.len() == 8)
    {
        return Err(invalid("full28 native loss panel sizes differ"));
    }
    Ok(original["mean_episode_ce"]
        .as_f64()
        .zip(construction["mean_episode_ce"].as_f64())
        .map(|(a, b)| (20. * a + 8. * b) / 28.)
        .filter(|v| v.is_finite()))
}
fn geometry_readout_coadapt(a: &Args) -> Result<()> {
    let start = Instant::now();
    let FrozenGeometryStart {
        source: weights,
        native,
        identity,
        tok,
        original,
        construction,
        expected,
        bins: before,
        binding,
    } = load_frozen_geometry(a)?;
    let input = a
        .geometry_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("geometry checkpoint absent"))?;
    let inputs = source_files(input)?;
    let run_root = geometry_run_root(input)?;
    let run_manifest = sha256_file(&run_root.join("manifest.json"))?;
    let frozen = context_shadow(&weights)?;
    let initial_bits = parameter_bits(&weights.parameters())?;
    let bytes = fs::read(&a.tokenizer)?;
    let initial_original = generate(
        "geometry-readout-baseline-original",
        &native,
        &original,
        &tok,
        start,
        a,
    )?;
    let initial_construction = generate(
        "geometry-readout-baseline-construction",
        &native,
        &construction,
        &tok,
        start,
        a,
    )?;
    if initial_original["rows"] != expected["original_generation"]["rows"]
        || initial_construction["rows"] != expected["construction_generation"]["rows"]
    {
        return Err(invalid(
            "geometry starting candidate actual28 replay differs",
        ));
    }
    let mut training = original.clone();
    training.extend(construction.iter().cloned());
    write(
        &a.out.join("frozen-panel.json"),
        &json!({"original":episode_labels(&original),"construction":episode_labels(&construction),"training_order":training.iter().map(|e|e.id.clone()).collect::<Vec<_>>(),"all28_each_update":true}),
    )?;
    let indices = (0..28).collect::<Vec<_>>();
    let mut optimizer = AdamW::new(
        weights
            .parameters()
            .into_iter()
            .filter(|(name, _)| readout_parameter(name))
            .map(|(_, var)| var)
            .collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let mut checkpoints = Vec::<Value>::new();
    let mut batch_receipts = Vec::new();
    let mut updates = 0usize;
    let mut measured_admission = None;
    let mut export = |step: usize| -> Result<()> {
        deadline(start, a)?;
        verify_context_shadow(&weights, &frozen)?;
        let path = a.out.join(format!("checkpoint-{step:04}"));
        let (receipt, loaded) = checkpoint(
            &path,
            &weights,
            &identity,
            &bytes,
            &original,
            step,
            "new_geometry_readout_coadapt_candidate_unadopted",
        )?;
        let payload = bin_files(&path.join("realizer-native"))?;
        if !readout_native_payloads_fixed(&before, &payload) || (step == 0 && payload != before) {
            return Err(invalid(
                "readout export changed frozen geometry/context/exp or zero-stage payload",
            ));
        }
        let now_bits = parameter_bits(&weights.parameters())?;
        if !frozen_readout_source_bits_fixed(&initial_bits, &now_bits) {
            return Err(invalid("readout export changed frozen source bits"));
        }
        let original_ce = context_direction_measure(&loaded, &original, start, a)?;
        let construction_ce = context_direction_measure(&loaded, &construction, start, a)?;
        let all28 = full28_native_ce(&original_ce, &construction_ce)?;
        let original_generation = generate(
            &format!("geometry-readout-{step:04}-original"),
            &loaded,
            &original,
            &tok,
            start,
            a,
        )?;
        let construction_generation = generate(
            &format!("geometry-readout-{step:04}-construction"),
            &loaded,
            &construction,
            &tok,
            start,
            a,
        )?;
        if step == 0
            && (original_generation["rows"] != initial_original["rows"]
                || construction_generation["rows"] != initial_construction["rows"])
        {
            return Err(invalid(
                "zero-update independently exported28 replay differs",
            ));
        }
        let comparisons=checkpoints.iter().map(|previous|Ok(json!({"previous_updates":previous["optimizer_updates"],"original":generation_comparison(&previous["original_generation"],&original_generation)?,"construction":generation_comparison(&previous["construction_generation"],&construction_generation)?,"original_canonical":context_direction_comparison(&previous["original_canonical"],&original_ce)?,"construction_canonical":context_direction_comparison(&previous["construction_canonical"],&construction_ce)?}))).collect::<Result<Vec<_>>>()?;
        let evaluation = json!({"optimizer_updates":step,"checkpoint":receipt,"native_payload_sha256":payload,"context_source_bits_unchanged":true,"context_and_geometry_native_payloads_unchanged":true,"all28_equal_episode_ce":all28,"original_canonical":original_ce,"construction_canonical":construction_ce,"original_generation":original_generation,"construction_generation":construction_generation,"comparisons_against_each_prior_export":comparisons});
        write(
            &a.out.join(format!("evaluation-{step:04}.json")),
            &evaluation,
        )?;
        checkpoints.push(evaluation);
        Ok(())
    };
    let work = (|| -> Result<()> {
        export(0)?;
        let baseline_seconds = start.elapsed().as_secs_f64();
        for step in 0..UPDATES {
            deadline(start, a)?;
            verify_context_shadow(&weights, &frozen)?;
            let current = weights.compile(identity.clone())?;
            let measured = batch(&indices, &training, &weights, &current, start, a)?;
            let gradients = readout_gradients(measured.gradients)?;
            if step == 0 {
                let seconds = measured.report["elapsed_seconds"]
                    .as_f64()
                    .filter(|v| v.is_finite() && *v > 0.)
                    .ok_or_else(|| invalid("first full28 timing invalid"))?;
                let family_names = ["consumer.potential", "consumer.no_read", "period"];
                let gradient_valid = family_names.iter().all(|name| {
                    gradients
                        .iter()
                        .filter(|(n, _)| n.starts_with(name))
                        .any(|(_, g)| {
                            g.flatten_all()
                                .and_then(|g| g.to_vec1::<f32>())
                                .is_ok_and(|values| {
                                    values.iter().all(|v| v.is_finite())
                                        && values.iter().any(|v| *v != 0.)
                                })
                        })
                });
                let reserve = 4. * baseline_seconds + 30.;
                let projected = UPDATES as f64 * seconds * 1.25 + reserve;
                let remaining = a.maximum_seconds as f64 - start.elapsed().as_secs_f64();
                measured_admission = Some(
                    json!({"first_full28_seconds":seconds,"gradient_families_nonzero_finite":gradient_valid,"baseline_load_replay_export_seconds":baseline_seconds,"fit_safety_factor":1.25,"evaluation_stop_reserve_seconds":reserve,"projected_remaining_seconds":projected,"remaining_declared_seconds":remaining,"admitted":gradient_valid && projected<=remaining,"before_first_optimizer_update":true}),
                );
                write(&a.out.join("measured-admission.json"), &measured_admission)?;
                if !gradient_valid || projected > remaining {
                    return Err(invalid(
                        "full28 gradient/runtime admission failed before first update",
                    ));
                }
            }
            let clip = apply(&weights, &mut optimizer, gradients)?;
            verify_context_shadow(&weights, &frozen)?;
            updates = step + 1;
            let report = json!({"optimizer_update":updates,"readout_gradient_clip_factor":clip,"gradient_filter_before_clipping":true,"all28_each_update":true,"context_source_bits_unchanged":true,"batch":measured.report});
            let name = format!("batch-{updates:04}.json");
            write(&a.out.join(&name), &report)?;
            batch_receipts.push(
                json!({"update":updates,"path":name,"sha256":sha256_file(&a.out.join(&name))?}),
            );
            write(
                &a.out.join("progress.json"),
                &json!({"optimizer_updates":updates,"declared_updates":UPDATES,"batch_receipts":batch_receipts,"elapsed_seconds":start.elapsed().as_secs_f64()}),
            )?;
            if updates % 16 == 0 {
                export(updates)?;
            }
        }
        Ok(())
    })();
    drop(export);
    let scores = checkpoints
        .iter()
        .map(|stage| stage["all28_equal_episode_ce"].as_f64())
        .collect::<Vec<_>>();
    let selected = best_finite_readout_checkpoint(&scores);
    if source_files(input)? != inputs
        || sha256_file(&run_root.join("manifest.json"))? != run_manifest
    {
        return Err(invalid("geometry parent inputs changed"));
    }
    report_output::verify(input)?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-frozen-context-readout/1","mode":a.mode,"status":if work.is_ok(){"completed"}else{"stopped_or_error"},"source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"starting_parent_binding":binding,"saved_identity":identity,"optimizer":optimizer_identity(),"optimizer_moments":"fresh AdamW","optimizer_updates":updates,"declared_updates":UPDATES,"objective":"static mean28episodes(mean token+EOS marginal CE); every28 episodes each update; existing ordinary alias-aware loss","training_case_visits":training.iter().map(|e|json!({"id":e.id,"visits":updates})).collect::<Vec<_>>(),"updated_families":["consumer.potential.*","consumer.no_read.*","period.*"],"frozen_families":["consumer.context.*"],"measured_admission":measured_admission,"checkpoints":checkpoints,"batch_receipts":batch_receipts,"selected_checkpoint_index":selected,"selection":"lowest finite native all28 equal-episode CE including unchanged checkpoint0; strict decrease, earlier checkpoint ties; no completion or transformer veto","context_source_bits_unchanged":context_shadow(&weights)?==frozen,"input_files_sha256":inputs,"input_files_unchanged":true,"initial_actual28_replay_equal":true,"no_adopted_model":true,"geometry_role":"retained new research parent; no prior winner rewrite","scope":"28 exposed development episodes on frozen newly learned geometry; no heldout/general-chat/geometry-advantage/energy qualification","work_error":work.as_ref().err().map(|e|e.to_string()),"wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    work
}

fn readout_fit(a: &Args) -> Result<()> {
    let start = Instant::now();
    let context_fit = a.mode == "context-fit";
    let composition = matches!(a.mode.as_str(), "composition-fit" | "context-fit");
    let LoadedFinal {
        source: mut weights,
        mut native,
        identity,
        tok,
        episodes,
        mut fit,
        retained_sha,
        mut before,
    } = load_final(a)?;
    let original_input = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("original checkpoint missing"))?;
    let mut composition_expected = None;
    let mut context_parent_evaluation = None;
    let mut prior_coadapt = None;
    if composition {
        let loaded = if context_fit {
            load_context_parent(a, &identity, &retained_sha, &before)?
        } else {
            load_continuation(a, &identity, &retained_sha, &before)?
        };
        weights = loaded.source;
        native = loaded.native;
        fit = loaded.fit;
        before = loaded.bins;
        composition_expected = Some(loaded.evaluation["ownprefix"].clone());
        if context_fit {
            context_parent_evaluation = Some(loaded.evaluation.clone());
            let root = a
                .transfer_checkpoint
                .as_ref()
                .and_then(|p| p.parent())
                .ok_or_else(|| invalid("context parent root missing"))?;
            let parent_args: Args = serde_json::from_slice(&fs::read(root.join("args.json"))?)?;
            let coadapt_root = parent_args
                .transfer_checkpoint
                .as_ref()
                .and_then(|p| p.parent())
                .ok_or_else(|| invalid("coadapt root missing"))?;
            prior_coadapt = Some((
                serde_json::from_slice::<Value>(&fs::read(
                    coadapt_root.join("evaluation-0032.json"),
                )?)?,
                serde_json::from_slice::<Value>(&fs::read(
                    coadapt_root.join("evaluation-0064.json"),
                )?)?,
            ));
        }
    }
    let input = if composition {
        a.transfer_checkpoint
            .as_ref()
            .ok_or_else(|| invalid("composition checkpoint missing"))?
    } else {
        original_input
    };
    let input_source_files = source_files(&input.join("realizer-source"))?;
    let input_native_files = source_files(&input.join("realizer-native"))?;
    let expected_path = a
        .expected_generation
        .as_ref()
        .ok_or_else(|| invalid("continuation generation missing"))?;
    report_output::verify(
        expected_path
            .parent()
            .ok_or_else(|| invalid("expected envelope missing"))?,
    )?;
    let saved_original: Value = serde_json::from_slice(&fs::read(expected_path)?)?;
    if composition
        && (saved_original["native_loaded_from_disk"] != true
            || saved_original["native_metadata_sha256"]
                != sha256_file(&original_input.join("realizer-native/metadata.json"))?)
    {
        return Err(invalid("original saved generation binding differs"));
    }
    let expected: Value = if let Some(value) = composition_expected {
        value
    } else {
        saved_original.clone()
    };
    let initial_generation = generate("readout-baseline", &native, &episodes, &tok, start, a)?;
    if expected["native_loaded_from_disk"] != true
        || expected["native_metadata_sha256"]
            != sha256_file(&input.join("realizer-native/metadata.json"))?
        || expected["rows"] != initial_generation["rows"]
    {
        return Err(invalid(
            "readout continuation baseline own-prefix replay differs",
        ));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let template = episodes
        .first()
        .ok_or_else(|| invalid("original cases missing"))?;
    let construction = if composition {
        composition_panel(template, &tok, &compiler, native.binding().eos_token_id())?
    } else {
        Vec::new()
    };
    let (transfer_cases, transfer_labels) = if composition {
        transfer_panel(
            template,
            &episodes,
            &tok,
            &compiler,
            native.binding().eos_token_id(),
        )?
    } else {
        (Vec::new(), Vec::new())
    };
    if composition
        && construction.iter().any(|e| {
            transfer_cases
                .iter()
                .any(|t| t.source_text == e.source_text)
        })
    {
        return Err(invalid(
            "composition construction overlaps development transfer literals",
        ));
    }
    // Start the cyclic schedule with the longer construction B8 so its actual
    // cost, rather than the short original cases, admits the remaining fit.
    let mut training = construction.clone();
    training.extend(episodes.iter().cloned());
    if composition {
        let root = a.out.join("construction-panel");
        report_output::claim(&root)?;
        write(
            &root.join("panel.json"),
            &json!({"schema":"uor-r4.geometric-composition-construction/1","original_cases":episode_labels(&episodes),"construction_cases":episode_labels(&construction),"development_transfer":transfer_labels,"training_cases":training.len(),"training_order_ids":training.iter().map(|e|e.id.clone()).collect::<Vec<_>>(),"answer_policy":"FrozenAnswers Current literal plus period; membership only","development_transfer_not_fresh_holdout":true,"transfer_targets_excluded_from_training":true}),
        )?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
    }
    if context_fit {
        let root = input
            .parent()
            .ok_or_else(|| invalid("context parent envelope missing"))?;
        report_output::verify(&root.join("construction-panel"))?;
        let saved: Value =
            serde_json::from_slice(&fs::read(root.join("construction-panel/panel.json"))?)?;
        let current: Value =
            serde_json::from_slice(&fs::read(a.out.join("construction-panel/panel.json"))?)?;
        if saved != current {
            return Err(invalid("context adaptation frozen panels differ"));
        }
    }
    let frozen = context_shadow(&weights)?;
    verify_context_shadow(&weights, &frozen)?;
    let context_sha = before
        .get("consumer/context-q4.bin")
        .ok_or_else(|| invalid("native context payload absent"))?;
    let bytes = fs::read(&a.tokenizer)?;
    let mut optimizer = AdamW::new(
        weights
            .parameters()
            .into_iter()
            .filter(|(n, _)| context_fit || readout_parameter(n))
            .map(|(_, v)| v)
            .collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let mut updates = 0usize;
    let mut batches = Vec::new();
    let mut checkpoints = Vec::new();
    let mut measured_admission = None::<Value>;
    let mut previous_evaluation = None::<Value>;
    let mut first_evaluation = None::<Value>;
    let mut export = |updates: usize| -> Result<()> {
        deadline(start, a)?;
        if !context_fit {
            verify_context_shadow(&weights, &frozen)?;
        }
        let path = a.out.join(format!("checkpoint-{updates:04}"));
        let (receipt, loaded) = checkpoint(
            &path,
            &weights,
            &identity,
            &bytes,
            &episodes,
            updates,
            "readout_continuation_unadopted",
        )?;
        let bins = bin_files(&path.join("realizer-native"))?;
        if before.keys().ne(bins.keys())
            || (updates == 0 && bins != before)
            || (!context_fit && bins.get("consumer/context-q4.bin") != Some(context_sha))
            || bins.get("consumer/exp-q31.bin") != before.get("consumer/exp-q31.bin")
        {
            return Err(invalid(
                "continuation native context/table inventory changed",
            ));
        }
        let canonical = if composition {
            json!(null)
        } else {
            canonical_measure(&loaded, &episodes, start, a)?
        };
        if !composition && canonical["target_steps"] != 114 {
            return Err(invalid(
                "continuation requires exact114 canonical positions",
            ));
        }
        let mut generation = if updates == 0 {
            initial_generation.clone()
        } else {
            generate(
                &format!("readout-{updates:04}"),
                &loaded,
                &episodes,
                &tok,
                start,
                a,
            )?
        };
        generation["native_loaded_from_disk"] = json!(true);
        generation["native_metadata_sha256"] =
            json!(sha256_file(&path.join("realizer-native/metadata.json"))?);
        let construction_generation = if composition {
            generate(
                &format!("composition-construction-{updates:04}"),
                &loaded,
                &construction,
                &tok,
                start,
                a,
            )?
        } else {
            json!(null)
        };
        let transfer_generation = if composition {
            generate(
                &format!("composition-transfer-{updates:04}"),
                &loaded,
                &transfer_cases,
                &tok,
                start,
                a,
            )?
        } else {
            json!(null)
        };
        if updates == 0 {
            if let Some(parent) = &context_parent_evaluation {
                if parent["ownprefix"]["rows"] != generation["rows"]
                    || parent["construction_ownprefix"]["rows"] != construction_generation["rows"]
                    || parent["development_transfer_ownprefix"]["rows"]
                        != transfer_generation["rows"]
                {
                    return Err(invalid("context adaptation all44 saved parent rows differ"));
                }
            }
        }
        let comparisons = if composition {
            let now = json!({"original":generation,"construction":construction_generation,"transfer":transfer_generation});
            let base = first_evaluation.as_ref().unwrap_or(&now);
            let prev = previous_evaluation.as_ref().unwrap_or(&now);
            let mut comparison = BTreeMap::new();
            for category in ["original", "construction", "transfer"] {
                comparison.insert(category,json!({"versus_start":generation_comparison(&base[category],&now[category])?,"versus_previous":generation_comparison(&prev[category],&now[category])?}));
            }
            if first_evaluation.is_none() {
                first_evaluation = Some(now.clone());
            }
            previous_evaluation = Some(now);
            json!(comparison)
        } else {
            json!(null)
        };
        let evaluation = json!({"optimizer_updates":updates,"checkpoint":receipt,"canonical":canonical,"ownprefix":generation,"construction_ownprefix":construction_generation,"development_transfer_ownprefix":transfer_generation,"row_comparisons":comparisons,"original_parent_comparison":if composition{Some(generation_comparison(&saved_original,&generation)?)}else{None},"context_source_bits_unchanged":context_shadow(&weights)? == frozen,"context_source_bit_changes":context_bit_changes(&frozen,&context_shadow(&weights)?)?,"context_native_payload_unchanged":bins.get("consumer/context-q4.bin") == Some(context_sha),"context_native_changed_bytes":changed_file_bytes(&input.join("realizer-native/consumer/context-q4.bin"),&path.join("realizer-native/consumer/context-q4.bin"))?,"prior_coadapt_original_comparisons":if let Some((p32,p64))=&prior_coadapt{Some(json!({"checkpoint0032":generation_comparison(&p32["ownprefix"],&generation)?,"checkpoint0064":generation_comparison(&p64["ownprefix"],&generation)?}))}else{None},"hard_payload_sha256":bins});
        write(
            &a.out.join(format!("evaluation-{updates:04}.json")),
            &evaluation,
        )?;
        checkpoints.push(evaluation);
        Ok(())
    };
    let work = (|| -> Result<()> {
        export(0)?;
        let baseline_stage_seconds = start.elapsed().as_secs_f64();
        for step in 0..UPDATES {
            deadline(start, a)?;
            let indices = (0..8)
                .map(|i| (step * 8 + i) % training.len())
                .collect::<Vec<_>>();
            let current = weights.compile(identity.clone())?;
            let measured = batch(&indices, &training, &weights, &current, start, a)?;
            if step == 0 {
                if context_fit
                    && !measured.report["gradient_family_l1"]["consumer.context"]
                        .as_f64()
                        .is_some_and(|v| v.is_finite() && v > 0.)
                {
                    return Err(invalid(
                        "first construction B8 has no nonzero context gradient",
                    ));
                }
                let first_b8_seconds = measured.report["elapsed_seconds"]
                    .as_f64()
                    .ok_or_else(|| invalid("first B8 elapsed absent"))?;
                // Four remaining evaluations plus an export/report stop margin.
                // Baseline stage includes load/replay and is a conservative
                // measured reserve, not a second construction campaign.
                let evaluation_reserve_seconds = 4. * baseline_stage_seconds + 30.;
                let projected_remaining_seconds =
                    UPDATES as f64 * first_b8_seconds + evaluation_reserve_seconds;
                let remaining_seconds = a.maximum_seconds as f64 - start.elapsed().as_secs_f64();
                measured_admission = Some(
                    json!({"first_b8_seconds":first_b8_seconds,"baseline_stage_seconds":baseline_stage_seconds,"evaluation_and_stop_reserve_seconds":evaluation_reserve_seconds,"projected_remaining_seconds":projected_remaining_seconds,"remaining_declared_seconds":remaining_seconds,"admitted":projected_remaining_seconds <= remaining_seconds}),
                );
                write(&a.out.join("measured-admission.json"), &measured_admission)?;
                if !first_b8_seconds.is_finite()
                    || first_b8_seconds <= 0.
                    || projected_remaining_seconds > remaining_seconds
                {
                    return Err(invalid("measured first B8 continuation projection exceeds remaining wall allowance"));
                }
            }
            // Readout-only modes filter frozen gradients before clipping. Context-fit
            // deliberately clips the complete existing context+readout gradient.
            let gradients = if context_fit {
                context_gradients(measured.gradients)?
            } else {
                readout_gradients(measured.gradients)?
            };
            let clip = apply(&weights, &mut optimizer, gradients)?;
            if !context_fit {
                verify_context_shadow(&weights, &frozen)?;
            }
            updates += 1;
            batches.push(json!({"optimizer_update":updates,"readout_only_clip_factor":if context_fit{None}else{Some(clip)},"all_family_clip_factor":if context_fit{Some(clip)}else{None},"unfiltered_batch_diagnostic":measured.report}));
            write(
                &a.out.join("progress.json"),
                &json!({"mode":a.mode,"optimizer_updates":updates,"declared_updates":UPDATES,"batches":batches,"wall_seconds":start.elapsed().as_secs_f64()}),
            )?;
            if updates % 16 == 0 {
                export(updates)?;
            }
        }
        Ok(())
    })();
    write(
        &a.out.join("report.json"),
        &json!({
            "schema":if context_fit{"uor-r4.geometric-context-adaptation/1"}else if composition{"uor-r4.geometric-composition-continuation/1"}else{"uor-r4.geometric-readout-continuation/1"},"mode":a.mode,"status":if work.is_ok(){"completed"}else{"stopped_or_error"},
            "source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"fit_source_commit":fit["source_commit"],
            "initial_checkpoint_manifest_sha256":sha256_file(&input.join("manifest.json"))?,"retained_report_sha256":retained_sha,"saved_identity":identity,
            "optimizer_updates":updates,"declared_updates":UPDATES,"measured_admission":measured_admission,"optimizer":optimizer_identity(),"optimizer_moments":"fresh AdamW state; original moments not saved",
            "updated_families":if context_fit{vec!["consumer.context.*","consumer.potential.*","consumer.no_read.*","period.*"]}else{vec!["consumer.potential.*","consumer.no_read.*","period.*"]},"frozen_families":if context_fit{vec![]}else{vec!["consumer.context.*"]},
            "gradient_filter_before_clipping":!context_fit,"context_gradient_rule":if context_fit{Some("existing declared context straight-through estimator; biased adjoint, not exact derivative of hard integer transitions")}else{None},"clipping_scope":if context_fit{"all existing context and readout gradient families"}else{"readout families only"},"objective":"unchanged mean8episodes(mean joint answer token+EOS CE)","baseline_saved_generation_exact":true,
            "checkpoints":checkpoints,"batches":batches,"context_source_bits_unchanged":context_shadow(&weights)? == frozen,
            "input_native_payload_unchanged":bin_files(&input.join("realizer-native"))? == before,"input_source_files_unchanged":source_files(&input.join("realizer-source"))? == input_source_files,"input_source_files_sha256":input_source_files,"input_all_native_files_unchanged":source_files(&input.join("realizer-native"))? == input_native_files,"baseline_all44_saved_rows_exact":if context_fit{Some(true)}else{None},"no_adopted_model":true,
            "training_cases":training.len(),"sampling":"fixed cyclic B8; equal episode loss within batch, per-episode update visits differ by at most one",
            "training_case_visits":training.iter().enumerate().map(|(i,e)|json!({"id":e.id,"visits":(0..updates*8).filter(|j|j%training.len()==i).count()})).collect::<Vec<_>>(),"construction_cases":construction.len(),"development_transfer_cases":transfer_cases.len(),"development_transfer_not_fresh_holdout":composition,
            "construction_manifest_sha256":if composition{Some(sha256_file(&a.out.join("construction-panel/manifest.json"))?)}else{None},
            "scope":if context_fit{"64-update existing context+readout adaptation;20 preservation+8 fixed construction training,16 previously examined development transfer; no heldout/geometric-advantage/chat qualification"}else if composition{"64-update readout-only composition continuation;20 preservation+8 fixed construction training,16 previously examined development transfer; no heldout/geometric-advantage/chat qualification"}else{"64-update existing-readout continuation on20 exposed development cases; five numerical source/query groups; no heldout, geometric-advantage or complete-chat qualification"},
            "work_error":work.as_ref().err().map(|e|e.to_string()),"wall_seconds":start.elapsed().as_secs_f64()
        }),
    )?;
    work
}
// Fixed, zero-update transfer panel. Labels are never passed to native.read.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenTransferAdmission {
    source_commit: String,
    frozen_fit_root: PathBuf,
    frozen_fit_manifest_sha256: String,
    frozen_fit_report_sha256: String,
    frozen_fit_source_commit: String,
    composition_root: PathBuf,
    composition_manifest_sha256: String,
    composition_report_sha256: String,
    composition_source_commit: String,
    checkpoint_manifest_sha256: BTreeMap<String, String>,
    panel_spec: PathBuf,
    panel_spec_sha256: String,
}
fn frozen_transfer_admission(a: &Args) -> Result<FrozenTransferAdmission> {
    serde_json::from_slice(&fs::read(
        a.transfer_admission
            .as_ref()
            .ok_or_else(|| invalid("transfer admission absent"))?,
    )?)
    .map_err(Into::into)
}
fn frozen_transfer_output_location(a: &Args) -> Result<()> {
    let admission = frozen_transfer_admission(a)?;
    let parent = a
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let output = fs::canonicalize(parent)?.join(
        a.out
            .file_name()
            .ok_or_else(|| invalid("transfer output leaf absent"))?,
    );
    for root in [
        admission.frozen_fit_root.as_path(),
        admission.composition_root.as_path(),
        a.checkpoint.as_path(),
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained root absent"))?,
    ] {
        if output.starts_with(fs::canonicalize(root)?) {
            return Err(invalid("transfer output beneath sealed input"));
        }
    }
    Ok(())
}
const FRESH_TRANSFER_PAIRS: [(&str, &str, &str); 4] = [
    ("replacement", "singer Louston", "dancer Brimfold"),
    ("order", "singer dancer Louston", "Louston dancer singer"),
    ("repetition", "singer dancer dancer", "singer singer dancer"),
    ("fresh-lexical", "Mervalon Quistara", "Mervalon Neskorin"),
];
fn fresh_transfer_spec_valid(spec: &Value) -> bool {
    spec["schema"] == "uor-r4.prospective-fresh-source-transfer/1"
        && spec["predictions"] == "NOT_RUN"
        && spec["optimizer_updates"] == 0
        && spec["context_tokens"] == 128
        && spec["response_tokens"] == 64
        && spec["pairs"].as_array().is_some_and(|pairs| {
            pairs.len() == 4
                && pairs
                    .iter()
                    .zip(FRESH_TRANSFER_PAIRS)
                    .all(|(pair, (id, left, right))| {
                        pair["id"] == id && pair["left"] == left && pair["right"] == right
                    })
        })
}
fn fresh_literals_disjoint(training: &[String], old: &[String]) -> bool {
    FRESH_TRANSFER_PAIRS.iter().all(|(_, left, right)| {
        [left, right]
            .into_iter()
            .all(|literal| !training.iter().chain(old).any(|seen| seen == literal))
    })
}
fn transfer_pair_inputs_fixed(left: &Episode, right: &Episode) -> bool {
    left.query == right.query
        && left.record == right.record
        && left.commit == right.commit
        && left.relation == right.relation
        && left.entity == right.entity
        && left.source_text != right.source_text
}
fn transfer_training_id_sets(training: &[Episode], eos: u32) -> (BTreeSet<u32>, BTreeSet<u32>) {
    let source = training
        .iter()
        .flat_map(|e| e.view.emitted_token_ids().iter().copied())
        .collect();
    let context = training
        .iter()
        .flat_map(|e| {
            e.view
                .emitted_token_ids()
                .iter()
                .chain(&e.query)
                .chain(e.target.iter().take(e.target.len().saturating_sub(1)))
        })
        .copied()
        .filter(|id| *id != eos)
        .collect();
    (source, context)
}
// This packet names exactly the inputs passed to native read, excluding all
// answer labels and their lengths. It is also preserved for inference audits.
fn transfer_source_packet(e: &Episode, prefix: &[u32]) -> Value {
    json!({"record":e.record,"commit":e.commit,"scope":"m-world-v2","entity":e.entity,"relation":e.relation,"view":0,"status":"Found","original_source_ids":e.tokens,"source_view":e.view,"query_ids":e.query,"actual_prefix_ids":prefix})
}
fn transfer_stage_receipt_bound(stage: &Value, evaluation: &Value, receipt: &Value) -> bool {
    stage == evaluation && stage["checkpoint"] == *receipt
}
fn fresh_transfer_panel(
    template: &Episode,
    training: &[Episode],
    old: &[Episode],
    tok: &ByteBpeTokenizer,
    compiler: &SourceEmissionCompiler,
    eos: u32,
) -> Result<(Vec<Episode>, Vec<Value>)> {
    if training.len() != 28
        || old.len() != 16
        || !fresh_literals_disjoint(
            &training
                .iter()
                .map(|e| e.source_text.clone())
                .collect::<Vec<_>>(),
            &old.iter()
                .map(|e| e.source_text.clone())
                .collect::<Vec<_>>(),
        )
    {
        return Err(invalid("fresh8 literals overlap exposed28 or previous16"));
    }
    let (source_ids, context_ids) = transfer_training_id_sets(training, eos);
    let mut episodes = Vec::new();
    let mut labels = Vec::new();
    for (pair, left, right) in FRESH_TRANSFER_PAIRS {
        let left_view = compiler.compile(&tok.encode(left))?;
        let right_view = compiler.compile(&tok.encode(right))?;
        let common = left_view
            .emitted_token_ids()
            .iter()
            .zip(right_view.emitted_token_ids())
            .take_while(|(a, b)| a == b)
            .count();
        let shared_divergence = shared_prefix_diverges(
            left_view.emitted_token_ids(),
            right_view.emitted_token_ids(),
        );
        for (side, literal) in [("left", left), ("right", right)] {
            let id = format!("fresh-{pair}-{side}");
            let answers = uor_r4_core::answer_oracle::FrozenAnswers {
                intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
                accepted: vec![format!("{literal}.")],
            };
            answers.validate().map_err(|e| invalid(e.to_string()))?;
            let tokens = tok.encode(literal);
            let view = compiler.compile(&tokens)?;
            let mut target = tok.encode(&format!(" {literal}."));
            target.push(eos);
            if tokens.is_empty()
                || view.original_bytes() != literal.as_bytes()
                || target.len() > 64
                || view.emitted_token_ids().len() + template.query.len() + 64 > 128
                || tokens.iter().chain(&target).any(|id| *id >= 4096)
            {
                return Err(invalid(
                    "fresh transfer byte/token/sequence admission differs",
                ));
            }
            let source_unseen = view
                .emitted_token_ids()
                .iter()
                .filter(|id| !source_ids.contains(id))
                .copied()
                .collect::<BTreeSet<_>>();
            let context_unseen = view
                .emitted_token_ids()
                .iter()
                .filter(|id| !context_ids.contains(id))
                .copied()
                .collect::<BTreeSet<_>>();
            labels.push(json!({"id":id,"pair_id":pair,"side":side,"literal":literal,"answers":answers,"target_ids_labels_only":target,"original_source_ids":tokens,"source_view":view,"query_ids":template.query,"record":template.record,"commit":template.commit,"relation":template.relation,"entity":template.entity,"scope":"m-world-v2","view":0,"initial_prefix_ids":[],"paired_emitted_common_prefix_length":common,"paired_emitted_shared_prefix_then_divergence":shared_divergence,"shared_prefix_property":"measured; absent property does not replace frozen literals","unseen_training28_source_ids":source_unseen,"unseen_training28_context_ids_excluding_eos":context_unseen,"literal_disjoint_all28_and_previous16":true}));
            episodes.push(Episode {
                id,
                query: template.query.clone(),
                tokens,
                record: template.record,
                commit: template.commit,
                relation: template.relation,
                entity: template.entity.clone(),
                target,
                accepted: answers.accepted,
                source_text: literal.into(),
                view,
            });
        }
    }
    if !episodes
        .chunks_exact(2)
        .all(|pair| transfer_pair_inputs_fixed(&pair[0], &pair[1]))
    {
        return Err(invalid("fresh pair typed inputs differ"));
    }
    Ok((episodes, labels))
}
fn frozen_transfer_fit_selection(report: &Value) -> Result<usize> {
    let stages = report["checkpoints"]
        .as_array()
        .ok_or_else(|| invalid("fit checkpoints absent"))?;
    if report["schema"] != "uor-r4.geometric-frozen-context-readout/1"
        || report["mode"] != "readout-geometry-coadapt"
        || report["status"] != "completed"
        || report["optimizer_updates"] != 64
        || report["declared_updates"] != 64
        || report["context_source_bits_unchanged"] != true
        || report["initial_actual28_replay_equal"] != true
        || report["input_files_unchanged"] != true
        || report["no_adopted_model"] != true
        || stages.len() != 5
    {
        return Err(invalid("completed frozen geometry fit admission differs"));
    }
    let mut scores = Vec::new();
    for (stage, step) in stages.iter().zip([0, 16, 32, 48, 64]) {
        if stage["optimizer_updates"] != step
            || stage["context_source_bits_unchanged"] != true
            || stage["context_and_geometry_native_payloads_unchanged"] != true
            || full28_native_ce(
                &stage["original_canonical"],
                &stage["construction_canonical"],
            )? != stage["all28_equal_episode_ce"].as_f64()
        {
            return Err(invalid("frozen fit stage or all28 criterion differs"));
        }
        scores.push(stage["all28_equal_episode_ce"].as_f64());
    }
    let best = best_finite_readout_checkpoint(&scores)
        .ok_or_else(|| invalid("finite fit selection absent"))?;
    if report["selected_checkpoint_index"] != best || stages[best]["optimizer_updates"] != 64 {
        return Err(invalid(
            "predeclared criterion-selected0064 differs; no winner substitution",
        ));
    }
    Ok(best)
}
fn frozen_transfer_failure(
    native: &NativeSourceRealizer,
    episode: &Episode,
    row: &Value,
) -> Result<Value> {
    let Some(step) = row["first_divergence_from_frozen_target"]
        .as_u64()
        .map(|v| v as usize)
    else {
        return Ok(Value::Null);
    };
    let Some(&target) = episode.target.get(step) else {
        return Ok(
            json!({"step":step,"target_label_only":null,"scope":"generated sequence longer than canonical labels"}),
        );
    };
    let generated = row["generated_ids"]
        .as_array()
        .ok_or_else(|| invalid("generated IDs absent"))?;
    let prefix = generated
        .iter()
        .take(step)
        .map(|v| {
            v.as_u64()
                .and_then(|x| u32::try_from(x).ok())
                .ok_or_else(|| invalid("generated ID invalid"))
        })
        .collect::<Result<Vec<_>>>()?;
    // Native read receives ONLY actual source frame/view, query and generated prefix.
    let trace = native.read(episode.frame(), &episode.view, &episode.query, &prefix)?;
    let masses = trace
        .actions
        .token_masses
        .iter()
        .map(|x| (x.token_id, x.weight_q31))
        .collect::<Vec<_>>();
    let rank = aggregate_rank(&masses, target)?;
    let target_mass = rank["target_mass_q31"]
        .as_u64()
        .ok_or_else(|| invalid("failure target mass absent"))?;
    let rival = masses
        .iter()
        .filter(|(id, _)| *id != target)
        .map(|(_, mass)| *mass)
        .max()
        .unwrap_or(0);
    let margin = i64::try_from(i128::from(target_mass) - i128::from(rival))
        .map_err(|_| invalid("failure mass margin overflow"))?;
    Ok(
        json!({"step":step,"actual_prefix_ids":prefix,"target_label_only":target,"chosen_token_id":trace.actions.chosen_token_id,"target_rank_probability":rank,"target_minus_largest_other_token_mass_q31":margin,"copy":copy_margin(&trace,episode.view.emitted_token_ids().len(),target)?,"native_actions":trace.actions,"inference_inputs":transfer_source_packet(episode,&prefix),"scope":"target only evaluated after native read; marginal aliases retained"}),
    )
}
fn frozen_transfer_pairs(labels: &[Value], generation: &Value) -> Result<Vec<Value>> {
    let rows = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("transfer rows absent"))?;
    if labels.len() != 8 || rows.len() != 8 {
        return Err(invalid("fresh pair count differs"));
    }
    Ok(labels.chunks_exact(2).zip(rows.chunks_exact(2)).map(|(label,row)|json!({"pair_id":label[0]["pair_id"],"left":row[0]["id"],"right":row[1]["id"],"both_exact_source_answers_with_eos":row.iter().all(|r|r["accepted_complete_answer"] == true && r["eos"] == true),"outputs_changed":row[0]["generated_ids"] != row[1]["generated_ids"]})).collect())
}
struct FrozenTransferInputs {
    admission: FrozenTransferAdmission,
    spec: Value,
    fit: Value,
    composition: Value,
    selected: usize,
    identity: ConsumerIdentity,
    retained_sha: String,
    bytes: Vec<u8>,
    tok: ByteBpeTokenizer,
    compiler: SourceEmissionCompiler,
    retained: Report,
}
fn load_frozen_transfer_inputs(a: &Args) -> Result<FrozenTransferInputs> {
    let admission = frozen_transfer_admission(a)?;
    if admission.source_commit != source_commit()?
        || admission.panel_spec_sha256
            != "b8cc71ac1f8264493f05dc168b2f26188f7532e975a7e3abec5b245320bbdf2d"
        || sha256_file(&admission.panel_spec)? != admission.panel_spec_sha256
    {
        return Err(invalid("fresh panel or executed source admission differs"));
    }
    let spec: Value = serde_json::from_slice(&fs::read(&admission.panel_spec)?)?;
    if !fresh_transfer_spec_valid(&spec) {
        return Err(invalid("prospective exact8 literal spec differs"));
    }
    let roots = [
        (
            &admission.frozen_fit_root,
            &admission.frozen_fit_manifest_sha256,
            &admission.frozen_fit_report_sha256,
            &admission.frozen_fit_source_commit,
        ),
        (
            &admission.composition_root,
            &admission.composition_manifest_sha256,
            &admission.composition_report_sha256,
            &admission.composition_source_commit,
        ),
    ];
    for (root, manifest, report, _) in &roots {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != **manifest
            || sha256_file(&root.join("report.json"))? != **report
        {
            return Err(invalid("portable fit envelope hashes differ"));
        }
    }
    let fit: Value =
        serde_json::from_slice(&fs::read(admission.frozen_fit_root.join("report.json"))?)?;
    let composition: Value =
        serde_json::from_slice(&fs::read(admission.composition_root.join("report.json"))?)?;
    let selected = frozen_transfer_fit_selection(&fit)?;
    if fit["source_commit"] != admission.frozen_fit_source_commit
        || composition["source_commit"] != admission.composition_source_commit
        || composition["schema"] != "uor-r4.geometric-composition-continuation/1"
        || composition["status"] != "completed"
        || composition["optimizer_updates"] != 64
        || composition["context_source_bits_unchanged"] != true
        || fit["saved_identity"] != composition["saved_identity"]
    {
        return Err(invalid("original/current comparator fit identity differs"));
    }
    let identity: ConsumerIdentity = serde_json::from_value(fit["saved_identity"].clone())?;
    if sha256_file(&a.tokenizer)? != identity.tokenizer_sha256
        || sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
            != identity.parent_checkpoint_manifest_sha256
        || sha256_file(&a.checkpoint.join("model.safetensors"))? != identity.parent_model_sha256
        || sha256_file(&a.checkpoint.join("config.json"))? != identity.parent_config_sha256
    {
        return Err(invalid(
            "transfer original stack/tokenizer identity differs",
        ));
    }
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope absent"))?,
    )?;
    let retained_sha = sha256_file(&a.retained_report)?;
    if fit["starting_parent_binding"]["retained_report_sha256"] != retained_sha
        || composition["retained_report_sha256"] != retained_sha
    {
        return Err(invalid("transfer retained diagnostic binding differs"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("transfer tokenizer invalid"))?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let retained: Report = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if retained.schema != "uor-r4.source-binding-diagnostic/1"
        || retained.tokenizer_sha256 != identity.tokenizer_sha256
        || retained.checkpoint_manifest_sha256 != identity.parent_checkpoint_manifest_sha256
    {
        return Err(invalid("retained transfer identity differs"));
    }
    Ok(FrozenTransferInputs {
        admission,
        spec,
        fit,
        composition,
        selected,
        identity,
        retained_sha,
        bytes,
        tok,
        compiler,
        retained,
    })
}
type FrozenTransferModel = (
    &'static str,
    PathBuf,
    Value,
    bool,
    NativeSourceRealizer,
    BTreeMap<String, String>,
);
fn load_transfer_comparator(
    name: &'static str,
    path: PathBuf,
    stage: Value,
    old: bool,
    admission: &FrozenTransferAdmission,
    identity: &ConsumerIdentity,
    bytes: &[u8],
) -> Result<FrozenTransferModel> {
    report_output::verify(&path)?;
    let receipt: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    let evaluation: Value = serde_json::from_slice(&fs::read(
        path.parent()
            .ok_or_else(|| invalid("stage parent absent"))?
            .join(format!(
                "evaluation-{:04}.json",
                if old {
                    32
                } else if name == "frozen0000" {
                    0
                } else {
                    64
                }
            )),
    )?)?;
    if admission.checkpoint_manifest_sha256.get(name)
        != Some(&sha256_file(&path.join("manifest.json"))?)
        || !transfer_stage_receipt_bound(&stage, &evaluation, &receipt)
    {
        return Err(invalid(
            "comparator checkpoint/stage receipt binding differs",
        ));
    }
    let source = SourceRealizerWeights::load_source(&path.join("realizer-source"), &bytes)?;
    let native = NativeSourceRealizer::load(&path.join("realizer-native"), &source, &identity)?;
    let bins = bin_files(&path.join("realizer-native"))?;
    let expected_bins = if old {
        &stage["hard_payload_sha256"]
    } else {
        &stage["native_payload_sha256"]
    };
    if bins != serde_json::from_value::<BTreeMap<String, String>>(expected_bins.clone())?
        || native.binding().protocol().schema != SCHEMA_V2
        || native.binding().vocab_size() != 4096
    {
        return Err(invalid("comparator source/native payload binding differs"));
    }
    let inventory = source_files(&path)?;
    Ok((name, path, stage, old, native, inventory))
}
fn frozen_geometry_transfer(a: &Args) -> Result<()> {
    let start = Instant::now();
    let FrozenTransferInputs {
        admission,
        spec,
        fit,
        composition,
        selected,
        identity,
        retained_sha,
        bytes,
        tok,
        compiler,
        retained,
    } = load_frozen_transfer_inputs(a)?;
    let roots = [
        (
            &admission.frozen_fit_root,
            &admission.frozen_fit_manifest_sha256,
            &admission.frozen_fit_report_sha256,
            &admission.frozen_fit_source_commit,
        ),
        (
            &admission.composition_root,
            &admission.composition_manifest_sha256,
            &admission.composition_report_sha256,
            &admission.composition_source_commit,
        ),
    ];
    let specs = [
        (
            "original0032",
            admission.composition_root.join("checkpoint-0032"),
            composition["checkpoints"]
                .as_array()
                .and_then(|st| st.iter().find(|v| v["optimizer_updates"] == 32))
                .ok_or_else(|| invalid("composition32 stage absent"))?
                .clone(),
            true,
        ),
        (
            "frozen0000",
            admission.frozen_fit_root.join("checkpoint-0000"),
            fit["checkpoints"][0].clone(),
            false,
        ),
        (
            "selected0064",
            admission.frozen_fit_root.join("checkpoint-0064"),
            fit["checkpoints"][selected].clone(),
            false,
        ),
    ];
    if admission.checkpoint_manifest_sha256.len() != 3 {
        return Err(invalid("three comparator manifests required"));
    }
    let mut models = Vec::new();
    for (name, path, stage, old) in specs {
        models.push(load_transfer_comparator(
            name, path, stage, old, &admission, &identity, &bytes,
        )?);
    }
    let eos = models[0].4.binding().eos_token_id();
    if models.iter().any(|m| m.4.binding().eos_token_id() != eos) {
        return Err(invalid("comparator EOS differs"));
    }
    let original = prepare(retained, &tok, eos, &compiler)?;
    let template = original
        .first()
        .ok_or_else(|| invalid("transfer template absent"))?;
    let construction = composition_panel(template, &tok, &compiler, eos)?;
    let mut training = original.clone();
    training.extend(construction.iter().cloned());
    let training_panel: Value = serde_json::from_slice(&fs::read(
        admission.frozen_fit_root.join("frozen-panel.json"),
    )?)?;
    if training_panel
        != json!({"original":episode_labels(&original),"construction":episode_labels(&construction),"training_order":training.iter().map(|e|e.id.clone()).collect::<Vec<_>>(),"all28_each_update":true})
    {
        return Err(invalid(
            "actual frozen training28/source-view/tokenizer panel differs",
        ));
    }
    let (old_panel, _) = transfer_panel(template, &original, &tok, &compiler, eos)?;
    let (fresh, labels) =
        fresh_transfer_panel(template, &training, &old_panel, &tok, &compiler, eos)?;
    let panel_root = a.out.join("panel");
    report_output::claim(&panel_root)?;
    write(&panel_root.join("panel-spec.json"), &spec)?;
    write(
        &panel_root.join("frozen-panel.json"),
        &json!({"schema":"uor-r4.frozen-geometric-fresh-transfer-panel/1","cases":labels,"training28":episode_labels(&training),"previous16":episode_labels(&old_panel),"spec_sha256":admission.panel_spec_sha256,"source_coverage_reference":"all28 emitted source IDs","context_coverage_reference":"all28 source/query/canonical-prefix IDs excluding EOS","predictions":"NOT_RUN","optimizer_updates":0,"targets":"FrozenAnswers labels; never native read inputs"}),
    )?;
    report_output::seal(&panel_root)?;
    report_output::verify(&panel_root)?;
    let mut replays = Vec::new();
    // All comparator replay admissions finish BEFORE any fresh/old transfer prediction.
    for (name, _, stage, old, native, _) in &models {
        let original_replay = generate(
            &format!("{name}-replay-original"),
            native,
            &original,
            &tok,
            start,
            a,
        )?;
        let construction_replay = generate(
            &format!("{name}-replay-construction"),
            native,
            &construction,
            &tok,
            start,
            a,
        )?;
        let expected_original = if *old {
            &stage["ownprefix"]
        } else {
            &stage["original_generation"]
        };
        let expected_construction = if *old {
            &stage["construction_ownprefix"]
        } else {
            &stage["construction_generation"]
        };
        if original_replay["rows"] != expected_original["rows"]
            || construction_replay["rows"] != expected_construction["rows"]
        {
            return Err(invalid("comparator independent actual28 replay differs"));
        }
        replays.push(json!({"model":name,"original":original_replay,"construction":construction_replay,"saved28_exact":true}));
    }
    let mut results = Vec::new();
    for (name, path, _, _, native, _) in &models {
        let development = generate(&format!("{name}-old16"), native, &old_panel, &tok, start, a)?;
        let fresh_generation = generate(&format!("{name}-fresh8"), native, &fresh, &tok, start, a)?;
        let rows = fresh_generation["rows"]
            .as_array()
            .ok_or_else(|| invalid("fresh rows absent"))?;
        let failures=fresh.iter().zip(rows).map(|(episode,row)|Ok(json!({"id":episode.id,"first_failure":frozen_transfer_failure(native,episode,row)?}))).collect::<Result<Vec<_>>>()?;
        results.push(json!({"model":name,"checkpoint":path,"checkpoint_manifest_sha256":sha256_file(&path.join("manifest.json"))?,"old16_open_development":development,"fresh8":fresh_generation,"pairs":frozen_transfer_pairs(&labels,&fresh_generation)?,"first_failure_diagnostics":failures}));
    }
    let mut comparisons = Vec::new();
    for left in 0..results.len() {
        for right in left + 1..results.len() {
            comparisons.push(json!({"left":results[left]["model"],"right":results[right]["model"],"old16":generation_comparison(&results[left]["old16_open_development"],&results[right]["old16_open_development"])?,"fresh8":generation_comparison(&results[left]["fresh8"],&results[right]["fresh8"])?}));
        }
    }
    for (_, path, _, _, _, inventory) in &models {
        if source_files(path)? != *inventory {
            return Err(invalid("transfer comparator changed"));
        }
        report_output::verify(path)?;
    }
    for (root, manifest, _, _) in roots {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *manifest {
            return Err(invalid("transfer input envelope changed"));
        }
    }
    if sha256_file(&admission.panel_spec)? != admission.panel_spec_sha256
        || sha256_file(&a.retained_report)? != retained_sha
        || sha256_file(&a.tokenizer)? != identity.tokenizer_sha256
    {
        return Err(invalid("transfer panel/retained/tokenizer input changed"));
    }
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained root absent"))?,
    )?;
    if sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
        != identity.parent_checkpoint_manifest_sha256
        || sha256_file(&a.checkpoint.join("model.safetensors"))? != identity.parent_model_sha256
        || sha256_file(&a.checkpoint.join("config.json"))? != identity.parent_config_sha256
    {
        return Err(invalid("transfer original identity input changed"));
    }
    report_output::verify(&panel_root)?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.frozen-geometric-readout-transfer/1","mode":a.mode,"status":"completed","source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"admission":admission,"saved_identity":identity,"optimizer_updates":0,"model_count":3,"fresh_cases_per_model":8,"old_development_cases_per_model":16,"replayed_cases_per_model":28,"selected_fit_checkpoint_index_recomputed":selected,"panel_manifest_sha256":sha256_file(&panel_root.join("manifest.json"))?,"replays":replays,"results":results,"comparisons":comparisons,"all_inputs_unchanged":true,"source_only_within_pair_intervention":true,"no_adopted_model":true,"scope":"authored selected-source component transfer; old16 open development; no session/distractor/NoRead/generalchat/geometryadvantage/energy qualification","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryDiagnosticAdmission {
    source_commit: String,
    transfer_root: PathBuf,
    transfer_manifest_sha256: String,
    transfer_report_sha256: String,
    transfer_source_commit: String,
}
fn query_diagnostic_admission(a: &Args) -> Result<QueryDiagnosticAdmission> {
    serde_json::from_slice(&fs::read(
        a.query_diagnostic_admission
            .as_ref()
            .ok_or_else(|| invalid("query diagnostic admission absent"))?,
    )?)
    .map_err(Into::into)
}
fn query_diagnostic_output_location(a: &Args) -> Result<()> {
    let admission = query_diagnostic_admission(a)?;
    let parent = a
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let output = fs::canonicalize(parent)?.join(
        a.out
            .file_name()
            .ok_or_else(|| invalid("query output leaf absent"))?,
    );
    if output.starts_with(fs::canonicalize(&admission.transfer_root)?) {
        return Err(invalid("query diagnostic beneath sealed transfer input"));
    }
    Ok(())
}
fn query_transfer_binding(
    report: &Value,
    admission: &FrozenTransferAdmission,
    identity: &ConsumerIdentity,
    transfer_source: &str,
) -> Result<()> {
    if report["schema"] != "uor-r4.frozen-geometric-readout-transfer/1"
        || report["mode"] != "readout-geometry-transfer"
        || report["status"] != "completed"
        || report["optimizer_updates"] != 0
        || report["all_inputs_unchanged"] != true
        || report["model_count"] != 3
        || report["fresh_cases_per_model"] != 8
        || report["replayed_cases_per_model"] != 28
        || report["selected_fit_checkpoint_index_recomputed"] != 4
        || report["saved_identity"] != serde_json::to_value(identity)?
        || report["source_commit"] != transfer_source
        || report["admission"]["source_commit"] != transfer_source
    {
        return Err(invalid("sealed transfer diagnostic admission differs"));
    }
    let current = serde_json::to_value(admission)?;
    // Input roots may be portable copies; scientific SHA identities must match.
    for key in [
        "frozen_fit_manifest_sha256",
        "frozen_fit_report_sha256",
        "frozen_fit_source_commit",
        "composition_manifest_sha256",
        "composition_report_sha256",
        "composition_source_commit",
        "checkpoint_manifest_sha256",
        "panel_spec_sha256",
    ] {
        if report["admission"][key] != current[key] {
            return Err(invalid(
                "query diagnostic transfer scientific binding differs",
            ));
        }
    }
    Ok(())
}
fn query_pair_replay_equal(
    left: &RealizerTrace,
    right: &RealizerTrace,
    shared: usize,
) -> Result<()> {
    let a = &left.period_context;
    let b = &right.period_context;
    if left.source.emission_view.emitted_token_ids().get(..shared)
        != right.source.emission_view.emitted_token_ids().get(..shared)
    {
        return Err(invalid("shared source emitted IDs differ"));
    }
    query_shared_context_equal(a, b, shared)
}
fn query_shared_context_equal(
    a: &SerializableContextReplay,
    b: &SerializableContextReplay,
    shared: usize,
) -> Result<()> {
    let lanes = a
        .heads
        .checked_mul(a.lanes_per_head)
        .ok_or_else(|| invalid("query diagnostic lane overflow"))?;
    let flat = shared
        .checked_mul(lanes)
        .ok_or_else(|| invalid("query diagnostic prefix overflow"))?;
    if a.heads != b.heads
        || a.lanes_per_head != b.lanes_per_head
        || shared == 0
        || a.tokens.get(..shared) != b.tokens.get(..shared)
        || a.tokens.get(..shared).is_none()
        || a.states.get(..shared).is_none()
        || a.actions.get(..shared).is_none()
        || a.raw_roots.get(..flat).is_none()
        || a.categories.get(..flat).is_none()
        || a.codes.get(..flat).is_none()
        || a.states.get(..shared) != b.states.get(..shared)
        || a.actions.get(..shared) != b.actions.get(..shared)
        || a.raw_roots.get(..flat) != b.raw_roots.get(..flat)
        || a.categories.get(..flat) != b.categories.get(..flat)
        || a.codes.get(..flat) != b.codes.get(..flat)
    {
        return Err(invalid(
            "causal shared source keys/states/actions/observations differ",
        ));
    }
    Ok(())
}
fn query_trajectory(trace: &RealizerTrace, episode: &Episode) -> Result<Vec<Value>> {
    query_trajectory_context(&trace.period_context, episode)
}
fn query_trajectory_context(
    c: &SerializableContextReplay,
    episode: &Episode,
) -> Result<Vec<Value>> {
    let lanes = c.heads * c.lanes_per_head;
    let source = episode.view.emitted_token_ids().len();
    if c.tokens.len() != source + episode.query.len() || c.tokens[source..] != episode.query {
        return Err(invalid(
            "query diagnostic empty-prefix/query alignment differs",
        ));
    }
    (0..episode.query.len()).map(|offset| {
        let t=source+offset;let start=t*lanes;let end=start+lanes;
        Ok(json!({"query_offset":offset,"absolute_context_time":t,"token_id":c.tokens[t],"states":c.states.get(t).ok_or_else(||invalid("query state absent"))?,"actions":c.actions.get(t).ok_or_else(||invalid("query action absent"))?,"raw_roots":c.raw_roots.get(start..end).ok_or_else(||invalid("query raw roots absent"))?,"categories":c.categories.get(start..end).ok_or_else(||invalid("query categories absent"))?,"codes":c.codes.get(start..end).ok_or_else(||invalid("query codes absent"))?}))
    }).collect()
}
struct QueryDiagnosticTables {
    potential: uor_r4_integer::geometric_potential_q4::NativePotentialQ4,
    stop: uor_r4_integer::geometric_no_read::NativeGeometricNoRead,
    period: uor_r4_integer::geometric_no_read::NativeGeometricNoRead,
}
fn load_query_tables(path: &Path, trace: &RealizerTrace) -> Result<QueryDiagnosticTables> {
    use uor_r4_integer::{
        geometric_no_read::{NativeGeometricNoRead, NoReadConfig},
        geometric_potential_q4::{NativePotentialQ4, PotentialQ4Config},
    };
    let c = &trace.period_context;
    let config = NoReadConfig {
        vocabulary: 4096,
        heads: c.heads,
        latent_lanes_per_head: c.lanes_per_head,
    };
    Ok(QueryDiagnosticTables {
        potential: NativePotentialQ4::new(
            PotentialQ4Config {
                heads: c.heads,
                lanes_per_head: c.lanes_per_head,
            },
            &fs::read(path.join("realizer-native/consumer/potential-q4.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?,
        stop: NativeGeometricNoRead::new(
            config,
            &fs::read(path.join("realizer-native/consumer/no-read-q4.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?,
        period: NativeGeometricNoRead::new(
            config,
            &fs::read(path.join("realizer-native/period-q4.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?,
    })
}
fn query_controller_contributions(
    table: &uor_r4_integer::geometric_no_read::NativeGeometricNoRead,
    trace: &RealizerTrace,
    expected: &[i64],
) -> Result<Value> {
    let roots = trace
        .period_context
        .states
        .last()
        .ok_or_else(|| invalid("controller final roots absent"))?;
    let last = *trace
        .period_context
        .tokens
        .last()
        .ok_or_else(|| invalid("controller token absent"))? as usize;
    let lanes = table.config().lanes();
    let codes = trace
        .period_context
        .codes
        .get(
            trace
                .period_context
                .codes
                .len()
                .checked_sub(lanes)
                .ok_or_else(|| invalid("controller codes absent"))?..,
        )
        .ok_or_else(|| invalid("controller code slice absent"))?;
    query_controller_parts(table, last, roots, codes, expected)
}
fn query_controller_parts(
    table: &uor_r4_integer::geometric_no_read::NativeGeometricNoRead,
    last: usize,
    roots: &[u8],
    codes: &[ObservedCode],
    expected: &[i64],
) -> Result<Value> {
    let c = table.config();
    let lanes = c.lanes();
    let expanded = table.expanded_q24();
    let stride = 1 + c.vocabulary + lanes * (128 + 64 + 128 + 1);
    if roots.len() != lanes
        || codes.len() != lanes
        || expected.len() != c.heads
        || last >= c.vocabulary
    {
        return Err(invalid("controller contribution dimensions differ"));
    }
    let coefficients = uor_r4_integer::geometric_potential_q4::unpack_coefficients(
        c.coefficient_count(),
        table.packed_coefficients(),
    )
    .map_err(|e| invalid(e.to_string()))?;
    let mut heads = Vec::new();
    for (head, &expected_score) in expected.iter().enumerate() {
        let base = head * stride;
        let coefficient_base = head * c.coefficients_per_head();
        let bias = expanded[base];
        let token_index = base + 1 + last;
        let token = expanded[token_index];
        let mut sum = i64::from(bias) + i64::from(token);
        let mut parts = Vec::new();
        for lane in 0..lanes {
            let root = usize::from(roots[lane]);
            let category = if codes[lane].present {
                usize::from(codes[lane].radius_bin) + 1
            } else {
                0
            };
            if root >= 120 || category >= 33 {
                return Err(invalid("controller root/category bounds"));
            }
            let root_index = base + 1 + c.vocabulary + lane * 128 + root;
            let category_index = base + 1 + c.vocabulary + lanes * 128 + lane * 64 + category;
            let root_coefficients = (0..4)
                .map(|axis| coefficient_base + 1 + c.vocabulary + lane * 4 + axis)
                .collect::<Vec<_>>();
            let category_coefficient =
                coefficient_base + 1 + c.vocabulary + lanes * 4 + lane * 33 + category;
            sum += i64::from(expanded[root_index]) + i64::from(expanded[category_index]);
            parts.push(json!({"global_latent_lane":lane,"root":root,"category":category,"latent_table_index":root_index,"latent_q24":expanded[root_index],"latent_basis_coefficient_indices":root_coefficients,"latent_basis_q":root_coefficients.iter().map(|i|coefficients[*i]).collect::<Vec<_>>(),"category_table_index":category_index,"category_q24":expanded[category_index],"category_coefficient_index":category_coefficient,"category_coefficient_q":coefficients[category_coefficient]}));
        }
        if sum != expected_score {
            return Err(invalid(
                "controller decomposition differs from native score",
            ));
        }
        heads.push(json!({"head":head,"bias_q24":bias,"bias_coefficient_index":coefficient_base,"bias_q":coefficients[coefficient_base],"last_token_id":last,"token_q24":token,"token_table_index":token_index,"token_coefficient_index":coefficient_base+1+last,"token_q":coefficients[coefficient_base+1+last],"all_latent_lanes":parts,"held_valid_active":false,"sum_q24":sum,"native_score_q24":expected_score,"exact":true}));
    }
    Ok(
        json!({"heads":heads,"summed_heads_q24":expected.iter().sum::<i64>(),"table_identity":"expanded native q4 codec; indices are expanded entries, coefficient indices name distinct learned inputs"}),
    )
}
fn query_copy_contributions(
    table: &uor_r4_integer::geometric_potential_q4::NativePotentialQ4,
    trace: &RealizerTrace,
    features: &Value,
) -> Result<Value> {
    use uor_r4_integer::geometric_potential::{
        CONTENT_PRESENCE_OFFSET, CONTEXT_PRESENCE_OFFSET, CONTEXT_RADIUS_OFFSET,
        CONTEXT_UNARY_OFFSET, ENTRIES_PER_LANE,
    };
    let config = table.config();
    let lanes = config.heads * config.lanes_per_head;
    let values = table.expanded_q24();
    let counts = [4, 4, 16, 1024, 1024, 4, 4];
    let mut offsets = [0; 7];
    let mut total = 0;
    for (at, count) in offsets.iter_mut().zip(counts) {
        *at = total;
        total += lanes * count;
    }
    let coefficients = uor_r4_integer::geometric_potential_q4::unpack_coefficients(
        total,
        table.packed_coefficients(),
    )
    .map_err(|e| invalid(e.to_string()))?;
    let mut occurrences = Vec::new();
    for (occurrence, copy) in features["copy_features"]
        .as_array()
        .ok_or_else(|| invalid("copy features absent"))?
        .iter()
        .enumerate()
    {
        let mut heads = Vec::new();
        let mut action_sum = 0i64;
        for head in 0..config.heads {
            let mut sum = 0i64;
            let mut lane_parts = Vec::new();
            for local in 0..config.lanes_per_head {
                let lane = head * config.lanes_per_head + local;
                let base = lane * ENTRIES_PER_LANE;
                let feature = &copy[lane];
                let presence = feature["context_presence"]
                    .as_u64()
                    .ok_or_else(|| invalid("copy presence absent"))?
                    as usize;
                if presence >= 4 {
                    return Err(invalid("copy presence bounds"));
                }
                let mut terms = vec![
                    ("content_presence", 5, 0, CONTENT_PRESENCE_OFFSET),
                    (
                        "context_presence",
                        6,
                        presence,
                        CONTEXT_PRESENCE_OFFSET + presence,
                    ),
                ];
                if let Some(root) = feature["relative_h4"].as_u64() {
                    let radius = feature["radius_index"]
                        .as_u64()
                        .ok_or_else(|| invalid("active radius absent"))?
                        as usize;
                    if root >= 120 || radius >= 1024 {
                        return Err(invalid("copy root/radius bounds"));
                    }
                    terms.push((
                        "context_unary",
                        1,
                        root as usize,
                        CONTEXT_UNARY_OFFSET + root as usize,
                    ));
                    terms.push(("context_radius", 4, radius, CONTEXT_RADIUS_OFFSET + radius));
                }
                let mut parts = Vec::new();
                for (family, family_index, local_index, entry) in terms {
                    let table_index = base + entry;
                    let q24 = values[table_index];
                    sum += i64::from(q24);
                    let coefficient_indices = if family_index == 1 {
                        (0..4)
                            .map(|axis| offsets[family_index] + lane * 4 + axis)
                            .collect::<Vec<_>>()
                    } else {
                        vec![offsets[family_index] + lane * counts[family_index] + local_index]
                    };
                    parts.push(json!({"family":family,"expanded_table_index":table_index,"family_entry_index":local_index,"value_q24":q24,"active":true,"learned_coefficient_indices":coefficient_indices,"coefficient_q":coefficient_indices.iter().map(|i|coefficients[*i]).collect::<Vec<_>>() }));
                }
                for family in [
                    "content_unary",
                    "pair",
                    "content_radius",
                    "context_unary",
                    "context_radius",
                ] {
                    if !parts.iter().any(|part| part["family"] == family) {
                        parts.push(json!({"family":family,"active":false,"expanded_table_index":null,"family_entry_index":null,"value_q24":0,"learned_coefficient_indices":[],"coefficient_q":[],"reason":"masked by absent content lanes or an absent context counterpart; no table entry read"}));
                    }
                }
                lane_parts.push(json!({"global_lane":lane,"local_lane":local,"feature":feature,"terms":parts,"content_unary_radius_pair_active":false}));
            }
            let expected = *trace
                .source
                .view_kernel_trace
                .heads
                .get(head)
                .and_then(|h| h.scores_q24.get(occurrence))
                .ok_or_else(|| invalid("native copy score absent"))?;
            if sum != expected {
                return Err(invalid("Copy decomposition differs from native score"));
            }
            action_sum += sum;
            heads.push(json!({"head":head,"lanes":lane_parts,"sum_q24":sum,"native_score_q24":expected,"exact":true}));
        }
        if trace.actions.actions.get(occurrence).map(|a| a.score_q24) != Some(action_sum) {
            return Err(invalid("Copy summed heads differ from joint action score"));
        }
        occurrences.push(json!({"emission_view_offset":occurrence,"token_id":trace.source.emission_view.emitted_token_ids()[occurrence],"heads":heads,"summed_heads_q24":action_sum}));
    }
    Ok(
        json!({"occurrences":occurrences,"alias_reduction":"unchanged joint native token alias aggregation; score contributions do not choose a unique occurrence"}),
    )
}
fn frozen_geometry_query_diagnostic(a: &Args) -> Result<()> {
    let start = Instant::now();
    let query_admission = query_diagnostic_admission(a)?;
    if query_admission.source_commit != source_commit()? {
        return Err(invalid("query diagnostic executed source differs"));
    }
    report_output::verify(&query_admission.transfer_root)?;
    if sha256_file(&query_admission.transfer_root.join("manifest.json"))?
        != query_admission.transfer_manifest_sha256
        || sha256_file(&query_admission.transfer_root.join("report.json"))?
            != query_admission.transfer_report_sha256
    {
        return Err(invalid("query sealed transfer report binding differs"));
    }
    let transfer: Value = serde_json::from_slice(&fs::read(
        query_admission.transfer_root.join("report.json"),
    )?)?;
    let FrozenTransferInputs {
        admission,
        fit,
        composition: _,
        selected,
        identity,
        retained_sha,
        bytes,
        tok,
        compiler,
        retained,
        spec: _,
    } = load_frozen_transfer_inputs(a)?;
    query_transfer_binding(
        &transfer,
        &admission,
        &identity,
        &query_admission.transfer_source_commit,
    )?;
    let (_, path, stage, _, native, inventory) = load_transfer_comparator(
        "selected0064",
        admission.frozen_fit_root.join("checkpoint-0064"),
        fit["checkpoints"][selected].clone(),
        false,
        &admission,
        &identity,
        &bytes,
    )?;
    let original = prepare(retained, &tok, native.binding().eos_token_id(), &compiler)?;
    let template = original
        .first()
        .ok_or_else(|| invalid("query template absent"))?;
    let construction =
        composition_panel(template, &tok, &compiler, native.binding().eos_token_id())?;
    let mut training = original.clone();
    training.extend(construction.iter().cloned());
    let (old_panel, _) = transfer_panel(
        template,
        &original,
        &tok,
        &compiler,
        native.binding().eos_token_id(),
    )?;
    let (fresh, labels) = fresh_transfer_panel(
        template,
        &training,
        &old_panel,
        &tok,
        &compiler,
        native.binding().eos_token_id(),
    )?;
    let panel: Value = serde_json::from_slice(&fs::read(
        query_admission
            .transfer_root
            .join("panel/frozen-panel.json"),
    )?)?;
    if panel["training28"] != json!(episode_labels(&training))
        || panel["cases"] != json!(labels)
        || transfer["panel_manifest_sha256"]
            != sha256_file(&query_admission.transfer_root.join("panel/manifest.json"))?
    {
        return Err(invalid(
            "query exact saved source-view/panel admission differs",
        ));
    }
    let specs = [
        ("construction-00", 4usize),
        ("fresh-repetition-left", 4),
        ("construction-05", 6),
        ("fresh-order-right", 6),
    ];
    let probes = specs
        .iter()
        .map(|(id, _)| {
            training
                .iter()
                .chain(&fresh)
                .find(|e| e.id == *id)
                .cloned()
                .ok_or_else(|| invalid("frozen query probe absent"))
        })
        .collect::<Result<Vec<_>>>()?;
    let expected_emitted = [
        vec![1130, 284, 1357, 284, 1130, 284],
        vec![1130, 284, 1357, 284, 1357, 284],
        vec![363, 277, 353, 292, 1357, 284],
        vec![363, 277, 353, 292, 1357, 284, 1130, 284],
    ];
    let expected_original = [
        vec![85, 300, 284, 1357, 284, 1130, 284],
        vec![85, 300, 284, 1357, 284, 1357, 284],
        vec![46, 277, 353, 292, 1357, 284],
        vec![46, 277, 353, 292, 1357, 284, 1130, 284],
    ];
    for ((episode, expected), original_ids) in
        probes.iter().zip(&expected_emitted).zip(&expected_original)
    {
        if episode.view.emitted_token_ids() != expected
            || episode.tokens != *original_ids
            || episode.query != [52, 970, 445, 519, 579, 627, 1541, 435, 16]
            || episode.record != 2
            || episode.commit != 2
            || episode.relation != 7
            || episode.entity != [644, 284]
        {
            return Err(invalid(
                "exact frozen four probe source/query/frame differs",
            ));
        }
    }
    let inputs = a.out.join("inputs");
    report_output::claim(&inputs)?;
    write(
        &inputs.join("probes.json"),
        &json!({"probes":episode_labels(&probes),"source_packets":probes.iter().map(|e|transfer_source_packet(e,&[])).collect::<Vec<_>>(),"actual_prefix_ids":[],"shared_source_keys":[4,6],"selected_checkpoint_manifest_sha256":sha256_file(&path.join("manifest.json"))?,"predictions":"NOT_RUN","labels":"diagnostics only after native reads"}),
    )?;
    report_output::seal(&inputs)?;
    report_output::verify(&inputs)?;
    let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
        "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
    ))
    .map_err(|e| invalid(e.to_string()))?;
    let selected_transfer = transfer["results"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["model"] == "selected0064"))
        .ok_or_else(|| invalid("selected transfer result absent"))?;
    let mut traces = Vec::new();
    let mut rows = Vec::new();
    let mut tables = None;
    for episode in &probes {
        deadline(start, a)?;
        let trace = native.read(episode.frame(), &episode.view, &episode.query, &[])?;
        let saved = if episode.id.starts_with("construction-") {
            stage["construction_canonical"]["rows"]
                .as_array()
                .and_then(|rows| rows.iter().find(|r| r["id"] == episode.id))
                .map(|r| r["tokens"][0]["actions"].clone())
        } else {
            selected_transfer["first_failure_diagnostics"]
                .as_array()
                .and_then(|rows| rows.iter().find(|r| r["id"] == episode.id))
                .map(|r| r["first_failure"]["native_actions"].clone())
        }
        .ok_or_else(|| invalid("saved probe action object absent"))?;
        if serde_json::to_value(&trace.actions)? != saved {
            return Err(invalid(
                "query full action replay differs from saved fit/transfer",
            ));
        }
        if tables.is_none() {
            tables = Some(load_query_tables(&path, &trace)?);
        }
        let table = tables
            .as_ref()
            .ok_or_else(|| invalid("query tables absent"))?;
        let features = scorer_features(&trace, &geometry)?;
        let target = *episode
            .target
            .first()
            .ok_or_else(|| invalid("diagnostic target label absent"))?;
        let masses = trace
            .actions
            .token_masses
            .iter()
            .map(|m| (m.token_id, m.weight_q31))
            .collect::<Vec<_>>();
        let stop = trace
            .source
            .view_kernel_trace
            .heads
            .iter()
            .map(|h| h.no_read_q24)
            .collect::<Vec<_>>();
        rows.push(json!({"id":episode.id,"inference_inputs":transfer_source_packet(episode,&[]),"target_label_only":target,"target_rank_probability":aggregate_rank(&masses,target)?,"full_action_object_saved_replay_equal":true,"trace":trace,"scorer_features":features,"consumed_source_context_indices":(0..episode.view.emitted_token_ids().len()).collect::<Vec<_>>(),"consumed_final_query_context_index":trace.period_context.tokens.len()-1,"controller_consumption":"last token and all final latent roots/categories; held=None","copy_contributions":query_copy_contributions(&table.potential,&trace,&features)?,"stop_contributions":query_controller_contributions(&table.stop,&trace,&stop)?,"period_contributions":query_controller_contributions(&table.period,&trace,&trace.period_q24)?,"query_trajectory":query_trajectory(&trace,episode)?}));
        traces.push(trace);
    }
    let mut pairs = Vec::new();
    for (index, shared) in [(0, 4), (2, 6)] {
        if !transfer_pair_inputs_fixed(&probes[index], &probes[index + 1]) {
            return Err(invalid("query paired typed inputs differ"));
        }
        query_pair_replay_equal(&traces[index], &traces[index + 1], shared)?;
        pairs.push(json!({"fitted":probes[index].id,"transfer":probes[index+1].id,"shared_source_keys":shared,"shared_source_states_actions_rawroots_categories_codes_exact":true,"query_alignment":"relative query offset, not absolute sequence time","fitted_query_trajectory":rows[index]["query_trajectory"],"transfer_query_trajectory":rows[index+1]["query_trajectory"],"fitted_actions":traces[index].actions,"transfer_actions":traces[index+1].actions}));
    }
    if source_files(&path)? != inventory
        || sha256_file(&a.retained_report)? != retained_sha
        || sha256_file(&a.tokenizer)? != identity.tokenizer_sha256
    {
        return Err(invalid("query diagnostic input changed"));
    }
    report_output::verify(&path)?;
    report_output::verify(&admission.frozen_fit_root)?;
    report_output::verify(&admission.composition_root)?;
    report_output::verify(&query_admission.transfer_root)?;
    report_output::verify(&inputs)?;
    if sha256_file(&query_admission.transfer_root.join("manifest.json"))?
        != query_admission.transfer_manifest_sha256
    {
        return Err(invalid("query transfer envelope changed"));
    }
    if sha256_file(&admission.frozen_fit_root.join("manifest.json"))?
        != admission.frozen_fit_manifest_sha256
        || sha256_file(&admission.composition_root.join("manifest.json"))?
            != admission.composition_manifest_sha256
        || sha256_file(&admission.panel_spec)? != admission.panel_spec_sha256
        || sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
            != identity.parent_checkpoint_manifest_sha256
        || sha256_file(&a.checkpoint.join("model.safetensors"))? != identity.parent_model_sha256
        || sha256_file(&a.checkpoint.join("config.json"))? != identity.parent_config_sha256
    {
        return Err(invalid("query admitted authority inputs changed"));
    }
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained root absent"))?,
    )?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-query-diagnostic/1","mode":a.mode,"status":"completed","source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"query_admission":query_admission,"transfer_admission":admission,"saved_identity":identity,"optimizer_updates":0,"native_read_count":4,"actual_prefix_ids":[],"inputs_manifest_sha256":sha256_file(&inputs.join("manifest.json"))?,"rows":rows,"pairs":pairs,"all_input_files_unchanged":true,"native_source_and_context_unchanged":true,"no_adopted_model":true,"scope":"four frozen empty-prefix reads; observed source-conditioned query/controller differences, not a counterfactual role-reset effect, linguistic invariance proof or new language result","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}

fn transfer_panel(
    template: &Episode,
    originals: &[Episode],
    tok: &ByteBpeTokenizer,
    compiler: &SourceEmissionCompiler,
    eos: u32,
) -> Result<(Vec<Episode>, Vec<Value>)> {
    let familiar: BTreeSet<u32> = originals
        .iter()
        .flat_map(|e| e.view.emitted_token_ids().iter().copied())
        .collect();
    let trained_context_ids: BTreeSet<u32> = originals
        .iter()
        .flat_map(|e| {
            e.view
                .emitted_token_ids()
                .iter()
                .chain(&e.query)
                .chain(e.target.iter().take(e.target.len().saturating_sub(1)))
                .copied()
        })
        .collect();
    // Explicit paired interventions; no selection based on candidate predictions.
    let specs = [
        ("replacement", "occupation", "singer", "dancer"),
        ("replacement", "place", "Brimfold", "Louston"),
        ("composition", "order", "singer dancer", "dancer singer"),
        ("composition", "repeat", "singer singer", "dancer dancer"),
        (
            "composition",
            "shared_prefix",
            "singer dancer",
            "singer singer",
        ),
        (
            "novel_literal",
            "concatenation",
            "singersinger",
            "dancerdancer",
        ),
        ("novel_literal", "unseen_literal", "Vexalnorp", "Quendazith"),
        (
            "novel_literal",
            "shared_prefix",
            "Vexalnorp singer",
            "Vexalnorp dancer",
        ),
    ];
    let mut episodes = Vec::new();
    let mut labels = Vec::new();
    for (category, pair, left, right) in specs {
        let pair_id = format!("{category}-{pair}");
        let left_ids = compiler.compile(&tok.encode(left))?;
        let right_ids = compiler.compile(&tok.encode(right))?;
        let common_prefix = left_ids
            .emitted_token_ids()
            .iter()
            .zip(right_ids.emitted_token_ids())
            .take_while(|(a, b)| a == b)
            .count();
        for (side, literal) in [("left", left), ("right", right)] {
            let id = format!("{pair_id}-{side}");
            let answers = uor_r4_core::answer_oracle::FrozenAnswers {
                intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
                accepted: vec![format!("{literal}.")],
            };
            answers.validate().map_err(|e| invalid(e.to_string()))?;
            let tokens = tok.encode(literal);
            let view = compiler.compile(&tokens)?;
            if view.original_bytes() != literal.as_bytes() {
                return Err(invalid("literal source byte identity differs"));
            }
            let mut target = tok.encode(&format!(" {literal}."));
            target.push(eos);
            let unfamiliar_ids: BTreeSet<_> = view
                .emitted_token_ids()
                .iter()
                .copied()
                .filter(|id| !familiar.contains(id))
                .collect();
            if target.len() > 64
                || view.emitted_token_ids().len() + template.query.len() + 64 > 128
                || tokens.iter().chain(&target).any(|id| *id >= 4096)
            {
                return Err(invalid(
                    "transfer literal exceeds sequence128/response64/V4096",
                ));
            }
            let never_context_ids: BTreeSet<_> = view
                .emitted_token_ids()
                .iter()
                .copied()
                .filter(|id| !trained_context_ids.contains(id))
                .collect();
            labels.push(json!({"id":id,"category":category,"pair_id":pair_id,"side":side,
                "subtype":pair,"literal":literal,"answers":answers,
                "target_ids_labels_only":target,"original_source_ids":tokens,
                "source_view":view,"query_ids":template.query,
                "source_record":template.record,"source_commit":template.commit,
                "scope":"m-world-v2","entity":template.entity,"relation":template.relation,"view":0,
                "initial_prefix_ids":[],"maximum_response_tokens":64,
                "paired_emitted_common_prefix_length":common_prefix,
                "emitted_token_count":view.emitted_token_ids().len(),
                "token_support":if unfamiliar_ids.is_empty(){"all_familiar_emitted_ids"}else{"contains_unseen_emitted_ids"},
                "unfamiliar_emitted_ids":unfamiliar_ids,"never_seen_training_context_ids":never_context_ids,
                "training_context_token_support":if never_context_ids.is_empty(){"all_seen_training_context_ids"}else{"contains_never_seen_training_context_ids"},"distractor_input":"absent from selected-source component interface"}));
            episodes.push(Episode {
                id,
                query: template.query.clone(),
                tokens,
                record: template.record,
                commit: template.commit,
                relation: template.relation,
                entity: template.entity.clone(),
                target,
                accepted: answers.accepted,
                source_text: literal.into(),
                view,
            });
        }
    }
    if !labels
        .iter()
        .any(|x| x["token_support"] == "all_familiar_emitted_ids")
        || !labels
            .iter()
            .any(|x| x["token_support"] == "contains_unseen_emitted_ids")
    {
        return Err(invalid(
            "fixed panel must contain familiar and source-unseen token buckets before inference",
        ));
    }
    for (pair, source_pair) in labels.chunks_exact(2).zip(episodes.chunks_exact(2)) {
        if source_pair[0].query != source_pair[1].query
            || source_pair[0].record != source_pair[1].record
            || source_pair[0].commit != source_pair[1].commit
            || source_pair[0].relation != source_pair[1].relation
            || source_pair[0].entity != source_pair[1].entity
        {
            return Err(invalid("paired transfer metadata differs"));
        }
        if pair[0]["subtype"] == "shared_prefix" {
            let left = source_pair[0].view.emitted_token_ids();
            let right = source_pair[1].view.emitted_token_ids();
            if !shared_prefix_diverges(left, right) {
                return Err(invalid(
                    "fixed shared-prefix pair must share an emitted ID then diverge",
                ));
            }
        }
    }
    Ok((episodes, labels))
}
fn composition_panel(
    template: &Episode,
    tok: &ByteBpeTokenizer,
    compiler: &SourceEmissionCompiler,
    eos: u32,
) -> Result<Vec<Episode>> {
    let literals = [
        "singer dancer singer",
        "dancer singer dancer",
        "singer Brimfold",
        "dancer Louston",
        "Brimfold singer",
        "Louston dancer",
        "singer singer singer",
        "dancer dancer dancer",
    ];
    literals
        .iter()
        .enumerate()
        .map(|(index, literal)| {
            let answers = uor_r4_core::answer_oracle::FrozenAnswers {
                intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
                accepted: vec![format!("{literal}.")],
            };
            answers.validate().map_err(|e| invalid(e.to_string()))?;
            let tokens = tok.encode(literal);
            let view = compiler.compile(&tokens)?;
            let mut target = tok.encode(&format!(" {literal}."));
            target.push(eos);
            if view.original_bytes() != literal.as_bytes()
                || target.len() > 64
                || view.emitted_token_ids().len() + template.query.len() + 64 > 128
                || tokens.iter().chain(&target).any(|id| *id >= 4096)
            {
                return Err(invalid("composition literal sequence/identity invalid"));
            }
            Ok(Episode {
                id: format!("construction-{index:02}"),
                query: template.query.clone(),
                tokens,
                record: template.record,
                commit: template.commit,
                relation: template.relation,
                entity: template.entity.clone(),
                target,
                accepted: answers.accepted,
                source_text: (*literal).into(),
                view,
            })
        })
        .collect()
}
fn episode_labels(episodes: &[Episode]) -> Vec<Value> {
    episodes.iter().map(|e|json!({"id":e.id,"typed_intent":"current","accepted":e.accepted,
        "literal":e.source_text,"query_ids":e.query,"original_source_ids":e.tokens,"source_view":e.view,
        "target_ids_labels_only":e.target,"record":e.record,"commit":e.commit,"relation":e.relation,"entity":e.entity})).collect()
}
fn shared_prefix_diverges(left: &[u32], right: &[u32]) -> bool {
    let prefix = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    prefix > 0 && prefix < left.len().min(right.len()) && left[prefix] != right[prefix]
}
fn generation_comparison(parent: &Value, candidate: &Value) -> Result<Vec<Value>> {
    let old = parent["rows"]
        .as_array()
        .ok_or_else(|| invalid("parent generation rows absent"))?;
    let new = candidate["rows"]
        .as_array()
        .ok_or_else(|| invalid("candidate generation rows absent"))?;
    if old.len() != new.len() {
        return Err(invalid("generation comparison length differs"));
    }
    old.iter().zip(new).map(|(a,b)| {
        if a["id"] != b["id"] || a["original_source_ids"] != b["original_source_ids"] || a["query_ids"] != b["query_ids"] {
            return Err(invalid("generation comparison inputs differ"));
        }
        let before=a["accepted_complete_answer"]==true;
        let after=b["accepted_complete_answer"]==true;
        Ok(json!({"id":a["id"],"parent_complete":before,"candidate_complete":after,
            "gain":!before&&after,"loss":before&&!after,"parent_eos":a["eos"],"candidate_eos":b["eos"],
            "parent_generated_ids":a["generated_ids"],"candidate_generated_ids":b["generated_ids"]}))
    }).collect()
}
struct LoadedContinuation {
    source: SourceRealizerWeights,
    native: NativeSourceRealizer,
    fit: Value,
    evaluation: Value,
    bins: BTreeMap<String, String>,
}
fn parameter_bits(
    parameters: &BTreeMap<String, candle_core::Var>,
) -> Result<BTreeMap<String, Vec<u32>>> {
    parameters
        .iter()
        .map(|(name, var)| {
            Ok((
                name.clone(),
                var.flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(f32::to_bits)
                    .collect(),
            ))
        })
        .collect()
}
fn transplant_family(
    destination: &BTreeMap<String, candle_core::Var>,
    donor: &BTreeMap<String, candle_core::Var>,
    copy_context: bool,
) -> Result<()> {
    if destination.is_empty() || destination.keys().ne(donor.keys()) {
        return Err(invalid("transplant parameter inventory differs"));
    }
    // Validate every family before changing any destination variable.
    for (name, var) in destination {
        if (!name.starts_with("consumer.context.") && !readout_parameter(name))
            || donor.get(name).is_none_or(|v| v.shape() != var.shape())
        {
            return Err(invalid("transplant family or dimensions differ"));
        }
    }
    for (name, var) in destination {
        if name.starts_with("consumer.context.") == copy_context {
            var.set(
                donor
                    .get(name)
                    .ok_or_else(|| invalid("transplant donor absent"))?
                    .as_tensor(),
            )?;
        }
    }
    Ok(())
}
fn saved_failure_prefix(row: &Value, episode: &Episode) -> Result<(usize, Vec<u32>, u32)> {
    saved_failure_prefix_ids(row, &episode.id, &episode.target)
}
fn saved_failure_prefix_ids(
    row: &Value,
    id: &str,
    targets: &[u32],
) -> Result<(usize, Vec<u32>, u32)> {
    if row["id"] != id || row["accepted_complete_answer"] == true {
        return Err(invalid("fixed failure row identity differs"));
    }
    let tokens = row["tokens"]
        .as_array()
        .ok_or_else(|| invalid("fixed failure tokens absent"))?;
    let mut preceding = Vec::new();
    for (step, token) in tokens.iter().enumerate() {
        let actual_prefix: Vec<u32> = serde_json::from_value(token["own_prefix_ids"].clone())?;
        if actual_prefix != preceding {
            return Err(invalid("saved failure prefix continuity differs"));
        }
        let chosen = u32::try_from(
            token["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| invalid("fixed failure chosen token absent"))?,
        )
        .map_err(|_| invalid("fixed failure token exceeds u32"))?;
        let target = *targets
            .get(step)
            .ok_or_else(|| invalid("fixed failure has no frozen target at step"))?;
        if chosen != target {
            let prefix: Vec<u32> = serde_json::from_value(token["own_prefix_ids"].clone())?;
            if prefix.len() != step || prefix != targets[..step] {
                return Err(invalid(
                    "fixed first divergence prefix differs from preceding frozen targets",
                ));
            }
            return Ok((step, prefix, target));
        }
        preceding.push(chosen);
    }
    Err(invalid(
        "fixed final first original has no divergent chosen token",
    ))
}
fn fixed_failure_trace(
    cell: &str,
    native: &NativeSourceRealizer,
    episode: &Episode,
    step: usize,
    prefix: &[u32],
    target: u32,
) -> Result<Value> {
    // The frozen target is a report label only; read receives the same saved
    // actual final64 prefix for each of the four factorization cells.
    let trace = native.read(episode.frame(), &episode.view, &episode.query, prefix)?;
    let target_mass = trace
        .actions
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .map(|m| m.weight_q31)
        .unwrap_or(0);
    let other_mass = trace
        .actions
        .token_masses
        .iter()
        .filter(|m| m.token_id != target)
        .map(|m| m.weight_q31)
        .max()
        .unwrap_or(0);
    let margin = i64::try_from(i128::from(target_mass) - i128::from(other_mass))
        .map_err(|_| invalid("fixed failure action margin exceeds i64"))?;
    Ok(
        json!({"cell":cell,"id":episode.id,"step":step,"prefix_ids":prefix,"prefix_policy":"identical saved final64 own-prefix at first divergence for all cells","target_label_only":target,"winner_token_id":trace.actions.chosen_token_id,"target_mass_q31":target_mass,"best_other_mass_q31":other_mass,"target_minus_best_other_mass_q31":margin,"trace":trace}),
    )
}
fn context_transplant(a: &Args) -> Result<()> {
    let start = Instant::now();
    let LoadedFinal {
        identity,
        tok,
        episodes,
        retained_sha,
        before,
        ..
    } = load_final(a)?;
    let new_path = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("transplant final checkpoint missing"))?;
    if new_path.file_name().and_then(|x| x.to_str()) != Some("checkpoint-0064") {
        return Err(invalid("transplant requires context-fit final64"));
    }
    let root = new_path
        .parent()
        .ok_or_else(|| invalid("context-fit parent envelope missing"))?;
    report_output::verify(root)?;
    report_output::verify(new_path)?;
    let saved_args: Args = serde_json::from_slice(&fs::read(root.join("args.json"))?)?;
    if saved_args.mode != "context-fit"
        || saved_args.period_seed != a.period_seed
        || sha256_file(&saved_args.tokenizer)? != sha256_file(&a.tokenizer)?
        || sha256_file(&saved_args.retained_report)? != retained_sha
        || sealed_manifest_sha256(&saved_args.checkpoint).map_err(|e| invalid(e.to_string()))?
            != identity.parent_checkpoint_manifest_sha256
        || sha256_file(
            &saved_args
                .audit_checkpoint
                .as_ref()
                .ok_or_else(|| invalid("saved original checkpoint missing"))?
                .join("manifest.json"),
        )? != sha256_file(
            &a.audit_checkpoint
                .as_ref()
                .ok_or_else(|| invalid("original checkpoint missing"))?
                .join("manifest.json"),
        )?
    {
        return Err(invalid("transplant context-fit input identities differ"));
    }
    let old = load_context_parent(&saved_args, &identity, &retained_sha, &before)?;
    let old_path = saved_args
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("old context checkpoint missing"))?;
    let fit: Value = serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
    let initial: Value = serde_json::from_slice(&fs::read(root.join("evaluation-0000.json"))?)?;
    let final_evaluation: Value =
        serde_json::from_slice(&fs::read(root.join("evaluation-0064.json"))?)?;
    let receipt: Value = serde_json::from_slice(&fs::read(new_path.join("checkpoint.json"))?)?;
    report_output::verify(&root.join("construction-panel"))?;
    if fit["schema"] != "uor-r4.geometric-context-adaptation/1"
        || fit["mode"] != "context-fit"
        || fit["status"] != "completed"
        || fit["optimizer_updates"] != 64
        || fit["retained_report_sha256"] != retained_sha
        || fit["saved_identity"] != serde_json::to_value(&identity)?
        || fit["initial_checkpoint_manifest_sha256"]
            != sha256_file(&old_path.join("manifest.json"))?
        || fit["fit_source_commit"] != old.fit["source_commit"]
        || !fit["source_commit"]
            .as_str()
            .is_some_and(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        || fit["construction_manifest_sha256"]
            != sha256_file(&root.join("construction-panel/manifest.json"))?
        || !fit["checkpoints"]
            .as_array()
            .is_some_and(|v| v.contains(&initial) && v.contains(&final_evaluation))
        || initial["optimizer_updates"] != 0
        || final_evaluation["optimizer_updates"] != 64
        || receipt["optimizer_updates"] != 64
        || receipt != final_evaluation["checkpoint"]
        || final_evaluation["ownprefix"]["native_metadata_sha256"]
            != sha256_file(&new_path.join("realizer-native/metadata.json"))?
    {
        return Err(invalid(
            "transplant context-fit report/checkpoint binding differs",
        ));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let new_source = SourceRealizerWeights::load_source(&new_path.join("realizer-source"), &bytes)?;
    let new_native =
        NativeSourceRealizer::load(&new_path.join("realizer-native"), &new_source, &identity)?;
    let new_bins = bin_files(&new_path.join("realizer-native"))?;
    if new_bins
        != serde_json::from_value::<BTreeMap<String, String>>(
            final_evaluation["hard_payload_sha256"].clone(),
        )?
        || old.bins
            != serde_json::from_value::<BTreeMap<String, String>>(
                initial["hard_payload_sha256"].clone(),
            )?
        || old.bins.keys().ne(new_bins.keys())
        || old.bins.get("consumer/exp-q31.bin") != new_bins.get("consumer/exp-q31.bin")
    {
        return Err(invalid("transplant parent payload identities differ"));
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let first = episodes
        .first()
        .ok_or_else(|| invalid("original episodes missing"))?;
    let construction =
        composition_panel(first, &tok, &compiler, old.native.binding().eos_token_id())?;
    let (_, transfer_labels) = transfer_panel(
        first,
        &episodes,
        &tok,
        &compiler,
        old.native.binding().eos_token_id(),
    )?;
    let mut training = construction.clone();
    training.extend(episodes.iter().cloned());
    let labels = json!({"schema":"uor-r4.geometric-composition-construction/1","original_cases":episode_labels(&episodes),"construction_cases":episode_labels(&construction),"development_transfer":transfer_labels,"training_cases":training.len(),"training_order_ids":training.iter().map(|e|e.id.clone()).collect::<Vec<_>>(),"answer_policy":"FrozenAnswers Current literal plus period; membership only","development_transfer_not_fresh_holdout":true,"transfer_targets_excluded_from_training":true});
    if labels
        != serde_json::from_slice::<Value>(&fs::read(root.join("construction-panel/panel.json"))?)?
    {
        return Err(invalid(
            "transplant panel differs from frozen context-fit panel",
        ));
    }
    let panel = a.out.join("frozen-panel");
    report_output::claim(&panel)?;
    write(&panel.join("panel.json"), &labels)?;
    report_output::seal(&panel)?;
    report_output::verify(&panel)?;
    let mut input_files = BTreeMap::new();
    for (name, path) in [("old", old_path), ("new", new_path)] {
        input_files.insert(name, source_files(path)?);
    }
    let old_bits = parameter_bits(&old.source.parameters())?;
    let new_bits = parameter_bits(&new_source.parameters())?;
    if old_bits.keys().ne(new_bits.keys()) {
        return Err(invalid("transplant source family inventories differ"));
    }
    let mut results = Vec::new();
    let mut fixed_traces = Vec::new();
    let mut fixed_failure_traces = Vec::new();
    let final_first = final_evaluation["ownprefix"]["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .ok_or_else(|| invalid("final first original row absent"))?;
    let (failure_step, failure_prefix, failure_target) = saved_failure_prefix(final_first, first)?;
    for (name, native, saved) in [
        ("old-context-old-readouts", &old.native, &initial),
        ("new-context-new-readouts", &new_native, &final_evaluation),
    ] {
        deadline(start, a)?;
        let original = generate(
            &format!("{name}-original"),
            native,
            &episodes,
            &tok,
            start,
            a,
        )?;
        let construction_generation = generate(
            &format!("{name}-construction"),
            native,
            &construction,
            &tok,
            start,
            a,
        )?;
        if original["rows"] != saved["ownprefix"]["rows"]
            || construction_generation["rows"] != saved["construction_ownprefix"]["rows"]
        {
            return Err(invalid(
                "transplant saved original20/construction8 parent replay differs",
            ));
        }
        fixed_traces.push(json!({"cell":name,"id":first.id,"prefix_ids":Vec::<u32>::new(),"trace":native.read(first.frame(),&first.view,&first.query,&[])?}));
        fixed_failure_traces.push(fixed_failure_trace(
            name,
            &native,
            first,
            failure_step,
            &failure_prefix,
            failure_target,
        )?);
        results.push(json!({"cell":name,"saved_rows_exact":true,"original":original,"construction":construction_generation}));
    }
    for (name, copy_context) in [
        ("old-context-new-readouts", false),
        ("new-context-old-readouts", true),
    ] {
        deadline(start, a)?;
        let mixed = SourceRealizerWeights::load_source(&old_path.join("realizer-source"), &bytes)?;
        transplant_family(&mixed.parameters(), &new_source.parameters(), copy_context)?;
        let expected_bits: BTreeMap<_, _> = old_bits
            .iter()
            .map(|(name, bits)| {
                (
                    name.clone(),
                    if name.starts_with("consumer.context.") == copy_context {
                        new_bits.get(name).cloned().unwrap_or_default()
                    } else {
                        bits.clone()
                    },
                )
            })
            .collect();
        if parameter_bits(&mixed.parameters())? != expected_bits {
            return Err(invalid("transplanted source family exact bits differ"));
        }
        let path = a.out.join(name);
        let (receipt, native) = checkpoint(
            &path,
            &mixed,
            &identity,
            &bytes,
            &episodes,
            0,
            "zero_update_context_readout_transplant_unadopted",
        )?;
        let bins = bin_files(&path.join("realizer-native"))?;
        let expected_bins: BTreeMap<_, _> = old
            .bins
            .iter()
            .map(|(file, sha)| {
                (
                    file.clone(),
                    if (file == "consumer/context-q4.bin") == copy_context {
                        new_bins.get(file).cloned().unwrap_or_default()
                    } else {
                        sha.clone()
                    },
                )
            })
            .collect();
        if bins != expected_bins {
            return Err(invalid("transplanted native payload mosaic differs"));
        }
        let original = generate(
            &format!("{name}-original"),
            &native,
            &episodes,
            &tok,
            start,
            a,
        )?;
        let construction_generation = generate(
            &format!("{name}-construction"),
            &native,
            &construction,
            &tok,
            start,
            a,
        )?;
        fixed_traces.push(json!({"cell":name,"id":first.id,"prefix_ids":Vec::<u32>::new(),"trace":native.read(first.frame(),&first.view,&first.query,&[])?}));
        fixed_failure_traces.push(fixed_failure_trace(
            name,
            &native,
            first,
            failure_step,
            &failure_prefix,
            failure_target,
        )?);
        results.push(json!({"cell":name,"checkpoint":receipt,"source_family_bits_exact":true,"native_payload_mosaic_exact":true,"native_payload_sha256":bins,"original":original,"construction":construction_generation,
            "original_comparisons":{"versus_start":generation_comparison(&initial["ownprefix"],&original)?,"versus_final64":generation_comparison(&final_evaluation["ownprefix"],&original)?},
            "construction_comparisons":{"versus_start":generation_comparison(&initial["construction_ownprefix"],&construction_generation)?,"versus_final64":generation_comparison(&final_evaluation["construction_ownprefix"],&construction_generation)?}}));
        write(&a.out.join("transplant-progress.json"), &results)?;
    }
    for (name, path) in [("old", old_path), ("new", new_path)] {
        if input_files.get(name) != Some(&source_files(path)?) {
            return Err(invalid("transplant input file set mutated"));
        }
        report_output::verify(path)?;
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-context-transplant/1","mode":a.mode,"status":"completed","source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"optimizer_updates":0,"native_hybrid_exports":2,"saved_identity":identity,"retained_report_sha256":retained_sha,"context_fit_source_commit":fit["source_commit"],"context_fit_manifest_sha256":sha256_file(&root.join("manifest.json"))?,"old_checkpoint_manifest_sha256":sha256_file(&old_path.join("manifest.json"))?,"new_checkpoint_manifest_sha256":sha256_file(&new_path.join("manifest.json"))?,"frozen_panel_manifest_sha256":sha256_file(&panel.join("manifest.json"))?,"input_files_sha256":input_files,"input_files_unchanged":true,"results":results,"fixed_first_original_empty_prefix_traces":fixed_traces,"fixed_first_original_final64_divergence_prefix_traces":fixed_failure_traces,"transfer_generation_run":false,"no_adopted_model":true,"scope":"zero-update2x2 component transplant diagnosis on20 exposed original+8 construction replies; mathematical factorization intervention, no heldout/geometric-advantage/chat qualification","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}
fn load_context_parent(
    a: &Args,
    identity: &ConsumerIdentity,
    retained_sha: &str,
    before: &BTreeMap<String, String>,
) -> Result<LoadedContinuation> {
    let path = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("context parent checkpoint missing"))?;
    if path.file_name().and_then(|x| x.to_str()) != Some("checkpoint-0032") {
        return Err(invalid("context adaptation requires fixed checkpoint-0032"));
    }
    let root = path
        .parent()
        .ok_or_else(|| invalid("context parent envelope missing"))?;
    report_output::verify(root)?;
    report_output::verify(path)?;
    let parent_args: Args = serde_json::from_slice(&fs::read(root.join("args.json"))?)?;
    let coadapt = load_continuation(&parent_args, identity, retained_sha, before)?;
    let fit: Value = serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
    let evaluation: Value = serde_json::from_slice(&fs::read(root.join("evaluation-0032.json"))?)?;
    let receipt: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    let coadapt_path = parent_args
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("coadapt checkpoint missing"))?;
    if fit["schema"] != "uor-r4.geometric-composition-continuation/1"
        || fit["mode"] != "composition-fit"
        || fit["status"] != "completed"
        || fit["optimizer_updates"] != 64
        || fit["initial_checkpoint_manifest_sha256"]
            != sha256_file(&coadapt_path.join("manifest.json"))?
        || fit["retained_report_sha256"] != retained_sha
        || fit["saved_identity"] != serde_json::to_value(identity)?
        || !fit["source_commit"]
            .as_str()
            .is_some_and(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        || fit["fit_source_commit"] != coadapt.fit["source_commit"]
        || fit["construction_manifest_sha256"]
            != sha256_file(&root.join("construction-panel/manifest.json"))?
        || !fit["checkpoints"]
            .as_array()
            .is_some_and(|stages| stages.iter().any(|v| v == &evaluation))
        || evaluation["ownprefix"]["native_metadata_sha256"]
            != sha256_file(&path.join("realizer-native/metadata.json"))?
        || sha256_file(&parent_args.tokenizer)? != sha256_file(&a.tokenizer)?
        || sha256_file(&parent_args.retained_report)? != retained_sha
        || parent_args.period_seed != a.period_seed
        || sha256_file(
            &parent_args
                .audit_checkpoint
                .as_ref()
                .ok_or_else(|| invalid("composition original checkpoint missing"))?
                .join("manifest.json"),
        )? != sha256_file(
            &a.audit_checkpoint
                .as_ref()
                .ok_or_else(|| invalid("current original checkpoint missing"))?
                .join("manifest.json"),
        )?
        || sealed_manifest_sha256(&parent_args.checkpoint).map_err(|e| invalid(e.to_string()))?
            != sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
        || receipt["optimizer_updates"] != 32
        || evaluation["optimizer_updates"] != 32
        || receipt != evaluation["checkpoint"]
        || evaluation["ownprefix"]["complete_answers"] != 20
        || evaluation["ownprefix"]["eos_count"] != 20
    {
        return Err(invalid("context parent identity/checkpoint/result differs"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let source = SourceRealizerWeights::load_source(&path.join("realizer-source"), &bytes)?;
    let native = NativeSourceRealizer::load(&path.join("realizer-native"), &source, identity)?;
    let bins = bin_files(&path.join("realizer-native"))?;
    if bins
        != serde_json::from_value::<BTreeMap<String, String>>(
            evaluation["hard_payload_sha256"].clone(),
        )?
        || before.keys().ne(bins.keys())
        || ["consumer/context-q4.bin", "consumer/exp-q31.bin"]
            .iter()
            .any(|n| before.get(*n) != bins.get(*n))
    {
        return Err(invalid("context parent payload binding differs"));
    }
    Ok(LoadedContinuation {
        source,
        native,
        fit,
        evaluation,
        bins,
    })
}
fn load_continuation(
    a: &Args,
    identity: &ConsumerIdentity,
    retained_sha: &str,
    before: &BTreeMap<String, String>,
) -> Result<LoadedContinuation> {
    let original = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("original checkpoint missing"))?;
    let candidate_path = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("continuation checkpoint missing"))?;
    if candidate_path.file_name().and_then(|x| x.to_str()) != Some("checkpoint-0064") {
        return Err(invalid("transfer requires fixed checkpoint-0064"));
    }
    let candidate_root = candidate_path
        .parent()
        .ok_or_else(|| invalid("candidate envelope missing"))?;
    report_output::verify(candidate_root)?;
    report_output::verify(candidate_path)?;
    let fit: Value = serde_json::from_slice(&fs::read(candidate_root.join("report.json"))?)?;
    let evaluation: Value =
        serde_json::from_slice(&fs::read(candidate_root.join("evaluation-0064.json"))?)?;
    let receipt: Value =
        serde_json::from_slice(&fs::read(candidate_path.join("checkpoint.json"))?)?;
    if fit["schema"] != "uor-r4.geometric-readout-continuation/1"
        || fit["mode"] != "readout-fit"
        || fit["status"] != "completed"
        || fit["optimizer_updates"] != 64
        || fit["initial_checkpoint_manifest_sha256"]
            != sha256_file(&original.join("manifest.json"))?
        || fit["retained_report_sha256"] != retained_sha
        || fit["saved_identity"] != serde_json::to_value(identity)?
        || receipt["optimizer_updates"] != 64
        || evaluation["optimizer_updates"] != 64
    {
        return Err(invalid("candidate continuation identity/parent differs"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let candidate_source =
        SourceRealizerWeights::load_source(&candidate_path.join("realizer-source"), &bytes)?;
    let candidate = NativeSourceRealizer::load(
        &candidate_path.join("realizer-native"),
        &candidate_source,
        identity,
    )?;
    let candidate_bins = bin_files(&candidate_path.join("realizer-native"))?;
    if before.keys().ne(candidate_bins.keys())
        || ["consumer/context-q4.bin", "consumer/exp-q31.bin"]
            .iter()
            .any(|name| {
                before.get(*name).is_none() || before.get(*name) != candidate_bins.get(*name)
            })
    {
        return Err(invalid("transfer candidate context differs from parent"));
    }
    Ok(LoadedContinuation {
        source: candidate_source,
        native: candidate,
        fit,
        evaluation,
        bins: candidate_bins,
    })
}
fn transfer(a: &Args) -> Result<()> {
    let start = Instant::now();
    let LoadedFinal {
        native: parent,
        identity,
        tok,
        episodes,
        retained_sha,
        before,
        ..
    } = load_final(a)?;
    let original = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("original checkpoint missing"))?;
    let candidate_path = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("transfer checkpoint missing"))?;
    let LoadedContinuation {
        native: candidate,
        evaluation,
        bins: candidate_bins,
        ..
    } = load_continuation(a, &identity, &retained_sha, &before)?;
    let candidate_root = candidate_path
        .parent()
        .ok_or_else(|| invalid("candidate envelope missing"))?;
    let bytes = fs::read(&a.tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let template = episodes
        .first()
        .ok_or_else(|| invalid("original cases missing"))?;
    let (panel, labels) = transfer_panel(
        template,
        &episodes,
        &tok,
        &compiler,
        parent.binding().eos_token_id(),
    )?;
    let panel_path = a.out.join("frozen-panel.json");
    // Freeze all answers, categories and inputs before any transfer predictions.
    write(
        &panel_path,
        &json!({"schema":"uor-r4.geometric-transfer-panel/1","cases":labels,
        "authoring":"fixed literal intent FrozenAnswers Current; membership only","design_selected_before_predictions":true,
        "familiarity_reference":"union of original20 emitted source IDs","evaluation_status":"open development transfer; authored panel, not final heldout"}),
    )?;
    let expected_path = a
        .expected_generation
        .as_ref()
        .ok_or_else(|| invalid("original saved generation missing"))?;
    report_output::verify(
        expected_path
            .parent()
            .ok_or_else(|| invalid("expected envelope missing"))?,
    )?;
    let expected: Value = serde_json::from_slice(&fs::read(expected_path)?)?;
    let original_baseline = generate("original-baseline", &parent, &episodes, &tok, start, a)?;
    let candidate_baseline = generate("candidate-baseline", &candidate, &episodes, &tok, start, a)?;
    if expected["native_loaded_from_disk"] != true
        || expected["native_metadata_sha256"]
            != sha256_file(&original.join("realizer-native/metadata.json"))?
        || expected["rows"] != original_baseline["rows"]
        || evaluation["ownprefix"]["native_loaded_from_disk"] != true
        || evaluation["ownprefix"]["rows"] != candidate_baseline["rows"]
        || evaluation["ownprefix"]["native_metadata_sha256"]
            != sha256_file(&candidate_path.join("realizer-native/metadata.json"))?
    {
        return Err(invalid(
            "fixed artifact original/candidate baseline replay differs",
        ));
    }
    let parent_generation = generate("transfer-parent", &parent, &panel, &tok, start, a)?;
    let candidate_generation = generate("transfer-candidate", &candidate, &panel, &tok, start, a)?;
    let mut pairs = Vec::new();
    let mut categories = BTreeMap::<String, Value>::new();
    let actual = candidate_generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("candidate rows absent"))?;
    for (metadata, rows) in labels.chunks_exact(2).zip(actual.chunks_exact(2)) {
        let follows = rows
            .iter()
            .all(|row| row["accepted_complete_answer"] == true);
        pairs.push(json!({"pair_id":metadata[0]["pair_id"],"category":metadata[0]["category"],
            "both_exact_source_answers_with_eos":follows,"outputs_changed":rows[0]["generated_ids"]!=rows[1]["generated_ids"],
            "left":rows[0]["id"],"right":rows[1]["id"]}));
    }
    for (label, row) in labels.iter().zip(actual) {
        for key in [label["category"].as_str(), label["token_support"].as_str()]
            .into_iter()
            .flatten()
        {
            let count = categories
                .entry(key.to_owned())
                .or_insert(json!({"cases":0,"complete":0,"eos":0}));
            for (field, increment) in [
                ("cases", 1),
                (
                    "complete",
                    u64::from(row["accepted_complete_answer"] == true),
                ),
                ("eos", u64::from(row["eos"] == true)),
            ] {
                count[field] = json!(
                    count[field]
                        .as_u64()
                        .ok_or_else(|| invalid("summary count invalid"))?
                        + increment
                );
            }
        }
    }
    if before != bin_files(&original.join("realizer-native"))?
        || candidate_bins != bin_files(&candidate_path.join("realizer-native"))?
    {
        return Err(invalid("transfer changed immutable native inputs"));
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-transfer/1","mode":"transfer","status":"completed",
        "source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"optimizer_updates":0,
        "saved_identity":identity,"retained_report_sha256":retained_sha,"frozen_panel_sha256":sha256_file(&panel_path)?,
        "parent_checkpoint_manifest_sha256":sha256_file(&original.join("manifest.json"))?,
        "candidate_checkpoint_manifest_sha256":sha256_file(&candidate_path.join("manifest.json"))?,
        "candidate_fit_report_sha256":sha256_file(&candidate_root.join("report.json"))?,
        "baseline_original_saved_rows_exact":true,"baseline_candidate_saved_rows_exact":true,"context_native_payload_equal":true,
        "baseline_comparison":generation_comparison(&original_baseline,&candidate_baseline)?,
        "original_baseline":original_baseline,"candidate_baseline":candidate_baseline,
        "transfer_comparison":generation_comparison(&parent_generation,&candidate_generation)?,
        "parent_generation":parent_generation,"candidate_generation":candidate_generation,"paired_source_following":pairs,"candidate_summaries":categories,
        "input_native_payloads_unchanged":true,"no_adopted_model":true,
        "scope":"fixed zero-update selected-source consumption transfer; no store/compiler selection or distractor input, no general prose/geometry advantage/chat qualification",
        "wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}
#[derive(Serialize)]
struct FrontierPosition {
    id: String,
    step: usize,
    prefix_ids: Vec<u32>,
    target_label_only: u32,
}
fn frontier_positions(episodes: &[Episode], generation: &Value) -> Result<Vec<FrontierPosition>> {
    let rows = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("frontier generation rows absent"))?;
    if episodes.len() != 8 || rows.len() != 8 {
        return Err(invalid("frontier requires all8 fixed construction rows"));
    }
    episodes
        .iter()
        .zip(rows)
        .map(|(episode, row)| {
            let (step, prefix_ids, target_label_only) = saved_failure_prefix(row, episode)?;
            Ok(FrontierPosition {
                id: episode.id.clone(),
                step,
                prefix_ids,
                target_label_only,
            })
        })
        .collect()
}
fn frontier_batch(
    positions: &[FrontierPosition],
    episodes: &[Episode],
    weights: &SourceRealizerWeights,
    native: &NativeSourceRealizer,
    start: Instant,
    a: &Args,
) -> Result<Batch> {
    let begun = Instant::now();
    let params = weights.parameters();
    let prepared = weights.prepare(native)?;
    let mut gradients = BTreeMap::new();
    let mut rows = Vec::new();
    let mut mean = 0f64;
    if positions.len() != 8 || episodes.len() != 8 {
        return Err(invalid("frontier loss requires B8"));
    }
    for (position, episode) in positions.iter().zip(episodes) {
        deadline(start, a)?;
        if position.id != episode.id {
            return Err(invalid("frontier loss row identity differs"));
        }
        let out = prepared.loss(
            episode.frame(),
            &episode.view,
            &episode.query,
            &position.prefix_ids,
            position.target_label_only,
        )?;
        let loss = out.loss.to_scalar::<f32>()?;
        let mass = out
            .trace
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == position.target_label_only)
            .map(|m| m.weight_q31)
            .unwrap_or(0);
        let total = out.trace.actions.total_weight_q31;
        if total == 0 {
            return Err(invalid("frontier native distribution empty"));
        }
        let probability = mass as f64 / total as f64;
        let expected = -probability.ln();
        if probability <= 0.
            || out.target_probability != probability
            || !loss.is_finite()
            || (f64::from(loss) - expected).abs() > 1e-4 + 1e-5 * expected.abs()
        {
            return Err(invalid("frontier loss differs from native target mass"));
        }
        let store = (&out.loss * (1f64 / 8.))?.backward()?;
        for (name, var) in &params {
            if let Some(g) = store.get(var.as_tensor()) {
                let detached = g.detach();
                if let Some(old) = gradients.remove(name) {
                    gradients.insert(name.clone(), (&old + &detached)?.detach());
                } else {
                    gradients.insert(name.clone(), detached);
                }
            }
        }
        mean += f64::from(loss) / 8.;
        rows.push(json!({"id":episode.id,"step":position.step,"own_prefix_ids":position.prefix_ids,"target_label_only":position.target_label_only,"nll":loss,"native_target_probability":probability,"native_loss_equal":true,"actions":out.trace.actions}));
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":8,"tokens":8,"objective":"mean8 frozen ownprefix next-token CE; one position per construction episode","mean_episode_nll":mean,"rows":rows,"elapsed_seconds":begun.elapsed().as_secs_f64()}),
    })
}
fn frontier_measure(
    positions: &[FrontierPosition],
    episodes: &[Episode],
    native: &NativeSourceRealizer,
    start: Instant,
    a: &Args,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut mean = 0f64;
    let mut zeros = 0usize;
    for (position, episode) in positions.iter().zip(episodes) {
        deadline(start, a)?;
        let mut row = fixed_failure_trace(
            "frozen-construction-frontier",
            native,
            episode,
            position.step,
            &position.prefix_ids,
            position.target_label_only,
        )?;
        let mass = row["target_mass_q31"]
            .as_u64()
            .ok_or_else(|| invalid("frontier mass absent"))?;
        let total = row["trace"]["actions"]["total_weight_q31"]
            .as_u64()
            .ok_or_else(|| invalid("frontier total absent"))?;
        if total == 0 {
            return Err(invalid("frontier distribution empty"));
        }
        let probability = mass as f64 / total as f64;
        let ce = if mass == 0 {
            zeros += 1;
            None
        } else {
            Some(-probability.ln())
        };
        if let Some(ce) = ce {
            mean += ce / positions.len() as f64;
        }
        row["native_probability"] = json!(probability);
        row["native_ce"] = json!(ce);
        row["zero_target_mass"] = json!(mass == 0);
        row["prefix_policy"] = json!(
            "same frozen round positions for incumbent and every candidate; labels offline only"
        );
        rows.push(row);
    }
    Ok(
        json!({"positions":positions.len(),"mean_episode_ce":if zeros==0{Some(mean)}else{None},"finite_contribution_to_mean_episode_ce":mean,"zero_target_mass_positions":zeros,"rows":rows}),
    )
}
fn frontier_coordinate_class(name: &str) -> Option<&'static str> {
    if !name.starts_with("consumer.context.") {
        None
    } else if name.ends_with("_root") {
        Some("root")
    } else if name.ends_with("_category") {
        Some("category")
    } else {
        None
    }
}
fn select_frontier_direction(
    mut coordinates: Vec<DirectionCoordinate>,
) -> Vec<DirectionCoordinate> {
    coordinates.retain(|c| {
        c.gradient.is_finite()
            && c.gradient != 0.
            && (-6..=6).contains(&c.original_q)
            && frontier_coordinate_class(&c.name).is_some()
    });
    coordinates.sort_by(|a, b| {
        b.gradient
            .abs()
            .total_cmp(&a.gradient.abs())
            .then(a.name.cmp(&b.name))
            .then(a.index.cmp(&b.index))
    });
    ["root", "category"]
        .into_iter()
        .filter_map(|class| {
            coordinates
                .iter()
                .find(|c| frontier_coordinate_class(&c.name) == Some(class))
                .cloned()
        })
        .collect()
}
fn context_coordinate_class(name: &str) -> Option<&'static str> {
    let family = name.strip_prefix("consumer.context.")?;
    if family.ends_with("_transition") {
        Some("transition")
    } else if family.ends_with("_root") || family.ends_with("_category") {
        Some("observation")
    } else {
        None
    }
}
fn select_context_direction(mut coordinates: Vec<DirectionCoordinate>) -> Vec<DirectionCoordinate> {
    coordinates.retain(|c| {
        c.gradient.is_finite()
            && c.gradient != 0.
            && (-6..=6).contains(&c.original_q)
            && context_coordinate_class(&c.name).is_some()
    });
    coordinates.sort_by(|a, b| {
        b.gradient
            .abs()
            .total_cmp(&a.gradient.abs())
            .then(a.name.cmp(&b.name))
            .then(a.index.cmp(&b.index))
    });
    ["transition", "observation"]
        .into_iter()
        .filter_map(|class| {
            coordinates
                .iter()
                .find(|c| context_coordinate_class(&c.name) == Some(class))
                .cloned()
        })
        .collect()
}
fn context_direction_measure(
    native: &NativeSourceRealizer,
    episodes: &[Episode],
    start: Instant,
    a: &Args,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut objective = 0f64;
    let mut zeros = 0usize;
    let mut count = 0usize;
    for episode in episodes {
        let mut tokens = Vec::new();
        let mut total = 0f64;
        let mut episode_zeros = 0usize;
        for (step, &target) in episode.target.iter().enumerate() {
            deadline(start, a)?;
            let trace = native.read(
                episode.frame(),
                &episode.view,
                &episode.query,
                &episode.target[..step],
            )?;
            let masses = trace
                .actions
                .token_masses
                .iter()
                .map(|m| (m.token_id, m.weight_q31))
                .collect::<Vec<_>>();
            let rank = aggregate_rank(&masses, target)?;
            let mass = rank["target_mass_q31"]
                .as_u64()
                .ok_or_else(|| invalid("context direction target mass absent"))?;
            let probability = rank["target_probability_diagnostic"]
                .as_f64()
                .ok_or_else(|| invalid("context direction probability absent"))?;
            let ce = if mass == 0 {
                episode_zeros += 1;
                None
            } else {
                Some(-probability.ln())
            };
            if let Some(value) = ce {
                total += value;
            }
            let other = trace
                .actions
                .token_masses
                .iter()
                .filter(|m| m.token_id != target)
                .map(|m| m.weight_q31)
                .max()
                .unwrap_or(0);
            let margin = i64::try_from(i128::from(mass) - i128::from(other))
                .map_err(|_| invalid("context direction mass margin overflow"))?;
            tokens.push(json!({"step":step,"target_label_only":target,"teacherforced_prefix_ids":&episode.target[..step],"native_ce":ce,"zero_target_mass":mass==0,"native_target_mass_q31":mass,"target_minus_best_other_mass_q31":margin,"target_rank_probability":rank,"actions":trace.actions,"period_q24":trace.period_q24,"context":trace.period_context}));
            count += 1;
        }
        objective += total / episode.target.len() as f64 / episodes.len() as f64;
        zeros += episode_zeros;
        rows.push(json!({"id":episode.id,"target_steps":episode.target.len(),"zero_target_mass_positions":episode_zeros,"mean_token_ce":if episode_zeros==0{Some(total/episode.target.len() as f64)}else{None},"tokens":tokens}));
    }
    Ok(
        json!({"mean_episode_ce":if zeros==0{Some(objective)}else{None},"finite_contribution_to_mean_episode_ce":objective,"zero_target_mass_positions":zeros,"zero_mass_policy":"nullCE denotes infinite native NLL; no probability floor","target_steps":count,"rows":rows}),
    )
}
fn context_trajectory_difference(before: &Value, after: &Value) -> Result<Value> {
    if before["tokens"] != after["tokens"]
        || before["heads"] != after["heads"]
        || before["lanes_per_head"] != after["lanes_per_head"]
    {
        return Err(invalid(
            "context direction trajectory inputs/dimensions differ",
        ));
    }
    let width = before["heads"]
        .as_u64()
        .and_then(|h| before["lanes_per_head"].as_u64().map(|l| h * l))
        .ok_or_else(|| invalid("context direction trajectory width absent"))?
        as usize;
    if width == 0 {
        return Err(invalid("context direction zero trajectory width"));
    }
    let time = before["tokens"]
        .as_array()
        .ok_or_else(|| invalid("context direction tokens absent"))?
        .len();
    let mut first = None;
    let mut changed = BTreeMap::new();
    for field in ["states", "actions", "raw_roots", "categories", "codes"] {
        let old = before[field]
            .as_array()
            .ok_or_else(|| invalid(format!("context trajectory {field} absent")))?;
        let new = after[field]
            .as_array()
            .ok_or_else(|| invalid(format!("context trajectory {field} absent")))?;
        if old.len() != new.len() {
            return Err(invalid("context direction trajectory shape differs"));
        }
        let per_position = if matches!(field, "states" | "actions") {
            1
        } else {
            width
        };
        if old.len() != time * per_position {
            return Err(invalid("context direction trajectory field length differs"));
        }
        let mut n = 0usize;
        for (index, (a, b)) in old.iter().zip(new).enumerate() {
            if a != b {
                n += 1;
                let position = index / per_position;
                first = Some(first.map_or(position, |f: usize| f.min(position)));
            }
        }
        changed.insert(field, n);
    }
    Ok(
        json!({"first_changed_sequence_position":first,"first_changed_input_token":first.and_then(|p|before["tokens"].get(p)).cloned(),"changed_entries":changed,"trajectory_unchanged":first.is_none()}),
    )
}
fn context_direction_comparison(before: &Value, after: &Value) -> Result<Value> {
    let old = before["rows"]
        .as_array()
        .ok_or_else(|| invalid("context direction baseline rows absent"))?;
    let new = after["rows"]
        .as_array()
        .ok_or_else(|| invalid("context direction candidate rows absent"))?;
    if old.len() != new.len() {
        return Err(invalid("context direction canonical case count differs"));
    }
    let mut rows = Vec::new();
    for (a, b) in old.iter().zip(new) {
        if a["id"] != b["id"] {
            return Err(invalid("context direction canonical case identity differs"));
        }
        let x = a["tokens"]
            .as_array()
            .ok_or_else(|| invalid("context direction baseline tokens absent"))?;
        let y = b["tokens"]
            .as_array()
            .ok_or_else(|| invalid("context direction candidate tokens absent"))?;
        if x.len() != y.len() {
            return Err(invalid("context direction canonical token count differs"));
        }
        let mut positions = Vec::new();
        for (old, new) in x.iter().zip(y) {
            if old["step"] != new["step"]
                || old["target_label_only"] != new["target_label_only"]
                || old["teacherforced_prefix_ids"] != new["teacherforced_prefix_ids"]
            {
                return Err(invalid("context direction target/prefix identity differs"));
            }
            positions.push(json!({"step":old["step"],"target_label_only":old["target_label_only"],"delta_native_ce":old["native_ce"].as_f64().zip(new["native_ce"].as_f64()).map(|(a,b)|b-a),"parent_zero_target_mass":old["zero_target_mass"],"candidate_zero_target_mass":new["zero_target_mass"],"parent_target_mass_q31":old["native_target_mass_q31"],"candidate_target_mass_q31":new["native_target_mass_q31"],"parent_margin_q31":old["target_minus_best_other_mass_q31"],"candidate_margin_q31":new["target_minus_best_other_mass_q31"],"parent_winner":old["actions"]["chosen_token_id"],"candidate_winner":new["actions"]["chosen_token_id"],"actions_unchanged":old["actions"]==new["actions"],"context_difference":context_trajectory_difference(&old["context"],&new["context"])?}));
        }
        rows.push(json!({"id":a["id"],"positions":positions}));
    }
    Ok(
        json!({"delta_mean_episode_ce":before["mean_episode_ce"].as_f64().zip(after["mean_episode_ce"].as_f64()).map(|(a,b)|b-a),"rows":rows}),
    )
}

// An accepted alias is anchored on its actual trajectory, never the canonical
// label sequence. The target EOS is used only by offline credit/measurement.
fn completed_eos_prefix(row: &Value, id: &str, eos: u32) -> Result<Vec<u32>> {
    if row["id"] != id || row["accepted_complete_answer"] != true || row["eos"] != true {
        return Err(invalid("completion anchor identity/acceptance/EOS differs"));
    }
    let generated: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
    let tokens = row["tokens"]
        .as_array()
        .ok_or_else(|| invalid("completion traces absent"))?;
    if tokens.len() != generated.len() + 1 || generated.contains(&eos) {
        return Err(invalid("completion trace length or embedded EOS differs"));
    }
    for (step, token) in tokens.iter().enumerate() {
        let prefix: Vec<u32> = serde_json::from_value(token["own_prefix_ids"].clone())?;
        let expected = if step == generated.len() {
            eos
        } else {
            generated[step]
        };
        if prefix != generated[..step]
            || token["chosen_token_id"] != expected
            || token["step"] != step
        {
            return Err(invalid("completion actual prefix/EOS continuity differs"));
        }
    }
    Ok(generated)
}
fn learning_frontiers(
    episodes: &[Episode],
    generation: &Value,
    tok: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
) -> Result<Vec<FrontierPosition>> {
    let rows = generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("learner rows absent"))?;
    if episodes.len() != 8 || rows.len() != 8 {
        return Err(invalid("learner requires B8"));
    }
    episodes
        .iter()
        .zip(rows)
        .map(|(episode, row)| {
            let (step, prefix_ids, target_label_only) = if row["accepted_complete_answer"] == true {
                let prefix =
                    completed_eos_prefix(row, &episode.id, native.binding().eos_token_id())?;
                let decoded = String::from_utf8(tok.decode_bytes(&prefix))
                    .map_err(|e| invalid(format!("completion UTF8: {e}")))?;
                let text = native.binding().protocol().reply_text(&decoded);
                if row["reply_text"] != text
                    || !episode.accepted.iter().any(|answer| answer == text)
                {
                    return Err(invalid(
                        "completion anchor differs from frozen answer membership",
                    ));
                }
                (prefix.len(), prefix, native.binding().eos_token_id())
            } else {
                saved_failure_prefix(row, episode)?
            };
            Ok(FrontierPosition {
                id: episode.id.clone(),
                step,
                prefix_ids,
                target_label_only,
            })
        })
        .collect()
}
// Strict native decrease; equal scores retain the earlier deterministic proposal.
fn best_native_proposal(baseline: f64, scores: &[Option<f64>]) -> Option<usize> {
    let mut best = baseline;
    let mut selected = None;
    for (index, score) in scores.iter().enumerate() {
        if let Some(score) = score.filter(|v| v.is_finite()) {
            if score < best {
                best = score;
                selected = Some(index);
            }
        }
    }
    selected
}
fn packed_byte_changes(before: &[u8], after: &[u8]) -> Result<Vec<Value>> {
    if before.len() != after.len() {
        return Err(invalid("cumulative packed inventory length differs"));
    }
    Ok(before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(index, (a, b))| json!({"byte_index":index,"parent_byte":a,"candidate_byte":b}))
        .collect())
}
#[derive(Clone, Serialize)]
struct CellEdit {
    coordinate: DirectionCoordinate,
    delta_q: i8,
}
#[derive(Clone, Serialize)]
struct CellProposal {
    edits: Vec<CellEdit>,
    witness: Value,
    predicted_gain: f64,
}
fn cell_winner(scores: &[i64]) -> Result<usize> {
    let mut best = None;
    for (index, &score) in scores.iter().enumerate() {
        if best.is_none_or(|(_, value)| score > value) {
            best = Some((index, score));
        }
    }
    best.map(|(index, _)| index)
        .ok_or_else(|| invalid("cell score vector empty"))
}
// Minimal signed legal source change that crosses this event's entire native
// argmax cell. Enumeration is offline arithmetic, not model evaluations.
fn cell_crossing(
    scores: &[i64],
    class: usize,
    q: i8,
    direction: i8,
) -> Result<Option<(i8, usize)>> {
    if !(-7..=7).contains(&q) || ![-1, 1].contains(&direction) || class >= scores.len() {
        return Err(invalid("cell coordinate invalid"));
    }
    let old = cell_winner(scores)?;
    for distance in 1..=14i8 {
        let delta = direction * distance;
        if !(-7..=7).contains(&(i16::from(q) + i16::from(delta))) {
            break;
        }
        let mut changed = scores.to_vec();
        changed[class] += i64::from(delta) << 22;
        let new = cell_winner(&changed)?;
        if new != old {
            return Ok(Some((delta, new)));
        }
    }
    Ok(None)
}
fn cell_key(p: &CellProposal) -> String {
    p.edits
        .iter()
        .map(|e| format!("{}:{}:{}", e.coordinate.name, e.coordinate.index, e.delta_q))
        .collect::<Vec<_>>()
        .join(";")
}
fn verify_cell_packed_edits(
    before: &[u8],
    after: &[u8],
    context: &BTreeMap<String, Vec<f32>>,
    edits: &[CellEdit],
) -> Result<Value> {
    let mut expected = before.to_vec();
    let mut seen = BTreeSet::new();
    for edit in edits {
        let family = edit
            .coordinate
            .name
            .strip_prefix("consumer.context.")
            .ok_or_else(|| invalid("cell family absent"))?;
        if !["token_root", "token_category"].contains(&family) {
            return Err(invalid("cell changed non-token observation"));
        }
        let mut offset = 0;
        for name in uor_r4_integer::geometric_context_q4::FAMILY_NAMES {
            if name == family {
                break;
            }
            offset += context
                .get(&format!("consumer.context.{name}"))
                .map_or(0, Vec::len);
        }
        let index = offset + edit.coordinate.index;
        if !seen.insert(index) {
            return Err(invalid("cell duplicate edit"));
        }
        let byte = expected
            .get_mut(index / 2)
            .ok_or_else(|| invalid("cell packed index absent"))?;
        let shift = (index % 2) * 4;
        let old = ((*byte >> shift) & 15) as i8;
        let old = if old >= 8 { old - 16 } else { old };
        if old != edit.coordinate.original_q {
            return Err(invalid("cell packed parent mismatch"));
        }
        let q = old + edit.delta_q;
        if !(-7..=7).contains(&q) {
            return Err(invalid("cell packed bound"));
        }
        *byte = (*byte & !(15 << shift)) | (((q as u8) & 15) << shift);
    }
    if expected != after {
        return Err(invalid("cell packed exact edits differ"));
    }
    Ok(
        json!({"exact_edits":edits,"all_other_nibbles_fixed":true,"l1_quanta":edits.iter().map(|e|i16::from(e.delta_q).abs()).sum::<i16>()}),
    )
}
fn cell_order(a: &CellProposal, b: &CellProposal) -> std::cmp::Ordering {
    let cost = |p: &CellProposal| {
        p.edits
            .iter()
            .map(|e| f64::from(e.delta_q.abs()))
            .sum::<f64>()
    };
    (b.predicted_gain / cost(b))
        .total_cmp(&(a.predicted_gain / cost(a)))
        .then_with(|| {
            a.edits
                .iter()
                .map(|e| (&e.coordinate.name, e.coordinate.index, e.delta_q))
                .cmp(
                    b.edits
                        .iter()
                        .map(|e| (&e.coordinate.name, e.coordinate.index, e.delta_q)),
                )
        })
        .then(a.witness.to_string().cmp(&b.witness.to_string()))
}
fn retain_cell(pool: &mut Vec<CellProposal>, proposal: CellProposal) {
    let key = cell_key(&proposal);
    if let Some(index) = pool.iter().position(|old| cell_key(old) == key) {
        if cell_order(&proposal, &pool[index]).is_lt() {
            pool[index] = proposal;
        }
    } else {
        pool.push(proposal);
    }
    pool.sort_by(cell_order);
    pool.truncate(4);
}
// Exact observation footprint of NativeOccurrenceReader::read: Copy uses the
// derived-source address at j<count and final address at total-1. Intermediate
// query/prefix outputs do not feed recurrence and are not scored directly.
fn consumed_context_roles(
    source: &[u32],
    query: &[u32],
    prefix: &[u32],
    actual: &[u32],
) -> Result<Vec<Option<&'static str>>> {
    if query.is_empty() {
        return Err(invalid("consumed footprint query empty"));
    }
    let expected = source
        .iter()
        .chain(query)
        .chain(prefix)
        .copied()
        .collect::<Vec<_>>();
    if expected != actual || actual.is_empty() {
        return Err(invalid("consumed footprint token order differs"));
    }
    Ok((0..actual.len())
        .map(|time| {
            if time < source.len() {
                Some("source_key")
            } else if time + 1 == actual.len() {
                Some("final_query_address")
            } else {
                None
            }
        })
        .collect())
}
fn consumed_replay_roles(
    trace: &RealizerTrace,
    query: &[u32],
    prefix: &[u32],
) -> Result<Vec<Option<&'static str>>> {
    let replay = &trace.period_context;
    let source = trace.source.emission_view.emitted_token_ids();
    let roles = consumed_context_roles(source, query, prefix, &replay.tokens)?;
    let kernel = &trace.source.view_kernel_trace;
    let lanes = replay
        .heads
        .checked_mul(replay.lanes_per_head)
        .ok_or_else(|| invalid("consumed footprint lane overflow"))?;
    if kernel.sequence_tokens != replay.tokens.len()
        || kernel.occurrences.len() != source.len()
        || kernel.heads.len() != replay.heads
        || replay.codes.len() != replay.tokens.len() * lanes
        || kernel
            .heads
            .iter()
            .any(|h| h.scores_q24.len() != source.len() || h.weights_q31.len() != source.len())
    {
        return Err(invalid("consumed footprint native dimensions differ"));
    }
    for (offset, (&token, occurrence)) in source.iter().zip(&kernel.occurrences).enumerate() {
        if occurrence.token_offset as usize != offset
            || occurrence.token_id != token
            || occurrence.record != trace.source.record
            || occurrence.commit != trace.source.commit
        {
            return Err(invalid("consumed footprint exact occurrence differs"));
        }
    }
    Ok(roles)
}
fn consumed_cell_selection(
    singles: &[CellProposal],
    pairs: &[CellProposal],
) -> Result<Vec<CellProposal>> {
    let mut selected = singles.iter().take(4).cloned().collect::<Vec<_>>();
    let selected_pairs = pairs.iter().take(2).cloned().collect::<Vec<_>>();
    selected.extend(selected_pairs.iter().cloned());
    for pair in selected_pairs {
        if pair.edits.len() != 2 || !pair.edits[1].coordinate.name.ends_with("token_category") {
            return Err(invalid("consumed matched category part absent"));
        }
        let edit = pair.edits[1].clone();
        let w = &pair.witness;
        selected.push(CellProposal{
            predicted_gain:-edit.coordinate.gradient*f64::from(edit.delta_q)*0.25,
            edits:vec![edit],
            witness:json!({"event":w["event"],"id":w["id"],"context_time":w["context_time"],"lane":w["lane"],"old_root":w["old_root"],"old_category":w["old_category"],"family":"token_category","old_winner":w["old_category"],"new_winner":w["presence"]["new_winner"],"class":w["presence"]["class"],"prospective_matched_category_part":true,"paired_edits":pair.edits,"component_credit_may_be_adverse":true}),
        });
    }
    let mut seen = BTreeSet::new();
    selected.retain(|p| seen.insert(cell_key(p)));
    if selected.len() > 8 {
        return Err(invalid("consumed proposal bound exceeded"));
    }
    Ok(selected)
}

fn later_query_cell_selection(
    globals: &[CellProposal],
    later: &[CellProposal],
    pairs: &[CellProposal],
) -> Result<Vec<CellProposal>> {
    let seed = if let Some(reserved) = later.first() {
        if reserved.edits.len() != 1
            || reserved.predicted_gain <= 0.
            || reserved.witness["reserved_later_query"] != true
            || !reserved.witness["actual_nonempty_prefix_ids"]
                .as_array()
                .is_some_and(|prefix| !prefix.is_empty())
        {
            return Err(invalid("reserved later-query singleton invalid"));
        }
        let mut seed = globals.iter().take(3).cloned().collect::<Vec<_>>();
        if let Some(existing) = seed.iter_mut().find(|p| cell_key(p) == cell_key(reserved)) {
            existing.witness = reserved.witness.clone();
        } else {
            seed.push(reserved.clone());
        }
        seed
    } else {
        globals.iter().take(4).cloned().collect()
    };
    consumed_cell_selection(&seed, pairs)
}
fn only_token_observation_bits_changed(
    before: &BTreeMap<String, Vec<u32>>,
    after: &BTreeMap<String, Vec<u32>>,
) -> bool {
    before.keys().eq(after.keys())
        && before.iter().all(|(name, bits)| {
            after.get(name).is_some_and(|candidate| {
                bits.len() == candidate.len()
                    && (matches!(
                        name.as_str(),
                        "consumer.context.token_root" | "consumer.context.token_category"
                    ) || bits == candidate)
            })
        })
}
fn consumed_parent_header(
    parent: &Value,
    identity: &ConsumerIdentity,
    retained_sha: &str,
    observation_manifest: &str,
    receipt: &Value,
) -> Result<()> {
    if parent["schema"] != "uor-r4.geometric-consumed-cells/1"
        || parent["mode"] != "context-consumed-cells"
        || parent["status"] != "completed"
        || parent["stop_reason"] != "accepted_candidate_limit"
        || parent["maximum_accepted_candidates"] != 3
        || parent["accepted_candidates"] != 3
        || parent["maximum_native_candidates"] != 24
        || !parent["native_candidate_evaluations"]
            .as_u64()
            .is_some_and(|n| (3..=24).contains(&n))
        || parent["optimizer_updates"] != 0
        || parent["parent_checkpoint_manifest_sha256"] != observation_manifest
        || parent["saved_identity"] != serde_json::to_value(identity)?
        || parent["retained_report_sha256"] != retained_sha
        || parent["final_checkpoint"] != *receipt
        || parent["final_independent_reload_full_replies_equal"] != true
        || parent["final_accepted_candidate_replies_equal"] != true
        || parent["transitions_readouts_tables_fixed"] != true
        || parent["no_adopted_model"] != true
        || !parent["source_commit"]
            .as_str()
            .is_some_and(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(invalid("accepted consumed-run header/lineage differs"));
    }
    Ok(())
}
fn checkpoint_model_files(path: &Path) -> Result<BTreeMap<String, String>> {
    Ok(source_files(path)?
        .into_iter()
        .filter(|(name, _)| {
            name.starts_with("realizer-source/") || name.starts_with("realizer-native/")
        })
        .collect())
}
fn consumed_accepted_round(report: &Value, entry: &Value) -> Result<(usize, usize)> {
    let selected = report["selected_proposal"]
        .as_u64()
        .ok_or_else(|| invalid("consumed round has no accepted selection"))?
        as usize;
    let summaries = report["proposals"]
        .as_array()
        .ok_or_else(|| invalid("consumed round candidate summaries absent"))?;
    let count = report["main_proposal_count"]
        .as_u64()
        .ok_or_else(|| invalid("consumed main quota absent"))? as usize;
    if count > 8
        || count != summaries.len()
        || report["ablation_count"] != 0
        || selected >= count
        || entry["selected_proposal"] != selected
        || entry["candidate_evaluations"] != count
    {
        return Err(invalid(
            "consumed selected candidate admission/count differs",
        ));
    }
    let baseline = report["baseline_frontier"]["mean_episode_ce"]
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid("consumed baseline CE nonfinite"))?;
    let scores = summaries
        .iter()
        .map(|v| v["round_frozen_ce"].as_f64())
        .collect::<Vec<_>>();
    if best_native_proposal(baseline, &scores) != Some(selected) {
        return Err(invalid("consumed selected candidate not native winner"));
    }
    Ok((selected, count))
}
fn admit_consumed_continuation(
    path: &Path,
    observation: &Path,
    identity: &ConsumerIdentity,
    retained_sha: &str,
    frozen_bins: &BTreeMap<String, String>,
) -> Result<(Value, Value)> {
    if path.file_name().and_then(|x| x.to_str()) != Some("final-checkpoint") {
        return Err(invalid(
            "consumed continuation must be run final checkpoint",
        ));
    }
    let envelope = path
        .parent()
        .ok_or_else(|| invalid("consumed continuation envelope absent"))?;
    report_output::verify(envelope)?;
    report_output::verify(path)?;
    let parent: Value = serde_json::from_slice(&fs::read(envelope.join("report.json"))?)?;
    let receipt: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    let observation_manifest = sha256_file(&observation.join("manifest.json"))?;
    consumed_parent_header(
        &parent,
        identity,
        retained_sha,
        &observation_manifest,
        &receipt,
    )?;
    if source_files(observation)?
        != serde_json::from_value::<BTreeMap<String, String>>(parent["input_files_sha256"].clone())?
    {
        return Err(invalid(
            "consumed continuation observation ancestor inventory differs",
        ));
    }
    let final_bins = bin_files(&path.join("realizer-native"))?;
    if final_bins
        != serde_json::from_value::<BTreeMap<String, String>>(
            parent["final_native_payload_sha256"].clone(),
        )?
        || frozen_bins.keys().ne(final_bins.keys())
        || frozen_bins.iter().any(|(name, sha)| {
            name != "consumer/context-q4.bin" && final_bins.get(name) != Some(sha)
        })
    {
        return Err(invalid(
            "consumed continuation frozen payload/final hashes differ",
        ));
    }
    let rounds = parent["rounds"]
        .as_array()
        .filter(|r| r.len() == 3)
        .ok_or_else(|| invalid("consumed accepted rounds absent"))?;
    let observation_report: Value = serde_json::from_slice(&fs::read(
        observation
            .parent()
            .ok_or_else(|| invalid("observation envelope absent"))?
            .join("report.json"),
    )?)?;
    let mut original_rows = observation_report["final_original_generation"]["rows"].clone();
    let mut construction_rows = observation_report["final_construction_generation"]["rows"].clone();
    let mut selected_chain = Vec::new();
    let mut evaluations = 0usize;
    let mut last_checkpoint = None;
    for (round, entry) in rounds.iter().enumerate() {
        let round_root = envelope.join(format!("round-{round:02}"));
        report_output::verify(&round_root)?;
        if entry["round"] != round
            || entry["report_path"] != format!("round-{round:02}/report.json")
            || entry["report_sha256"] != sha256_file(&round_root.join("report.json"))?
        {
            return Err(invalid("consumed round source binding differs"));
        }
        let report: Value = serde_json::from_slice(&fs::read(round_root.join("report.json"))?)?;
        let (selected, count) = consumed_accepted_round(&report, entry)?;
        let summaries = report["proposals"]
            .as_array()
            .ok_or_else(|| invalid("consumed round summaries absent"))?;
        if report["incumbent_original_generation"]["rows"] != original_rows
            || report["incumbent_construction_generation"]["rows"] != construction_rows
        {
            return Err(invalid("consumed incumbent reply chain differs"));
        }
        let summary = &summaries[selected];
        let candidate_root = round_root.join(format!("candidate-{selected:02}"));
        report_output::verify(&candidate_root)?;
        if summary["index"] != selected
            || summary["result_path"] != format!("candidate-{selected:02}/result.json")
            || summary["result_sha256"] != sha256_file(&candidate_root.join("result.json"))?
        {
            return Err(invalid("consumed selected result binding differs"));
        }
        let result: Value = serde_json::from_slice(&fs::read(candidate_root.join("result.json"))?)?;
        let candidate_checkpoint = candidate_root.join("checkpoint");
        report_output::verify(&candidate_checkpoint)?;
        let checkpoint_receipt: Value =
            serde_json::from_slice(&fs::read(candidate_checkpoint.join("checkpoint.json"))?)?;
        if result["checkpoint"] != checkpoint_receipt
            || result["ablation_only"] != false
            || result["all_other_source_bits_fixed"] != true
            || result["readouts_tables_transitions_fixed"] != true
            || result["frontier"]["mean_episode_ce"] != summary["round_frozen_ce"]
            || bin_files(&candidate_checkpoint.join("realizer-native"))?
                != serde_json::from_value::<BTreeMap<String, String>>(
                    result["native_payload_sha256"].clone(),
                )?
        {
            return Err(invalid("consumed selected checkpoint/decision differs"));
        }
        original_rows = result["original_generation"]["rows"].clone();
        construction_rows = result["construction_generation"]["rows"].clone();
        selected_chain.push(json!({"round":round,"selected_candidate":selected,"round_report_sha256":entry["report_sha256"],"result_sha256":summary["result_sha256"],"checkpoint_manifest_sha256":sha256_file(&candidate_checkpoint.join("manifest.json"))?}));
        last_checkpoint = Some(candidate_checkpoint);
        evaluations += count;
    }
    if parent["native_candidate_evaluations"] != evaluations
        || parent["final_original_generation"]["rows"] != original_rows
        || parent["final_construction_generation"]["rows"] != construction_rows
        || checkpoint_model_files(path)?
            != checkpoint_model_files(
                &last_checkpoint
                    .ok_or_else(|| invalid("consumed last accepted checkpoint absent"))?,
            )?
    {
        return Err(invalid(
            "consumed final does not equal last accepted model/replies",
        ));
    }
    let binding = json!({"schema":"uor-r4.accepted-consumed-continuation/1","consumed_source_commit":parent["source_commit"],"consumed_report_sha256":sha256_file(&envelope.join("report.json"))?,"consumed_checkpoint_manifest_sha256":sha256_file(&path.join("manifest.json"))?,"observation_manifest_sha256":observation_manifest,"selected_chain":selected_chain,"final_equals_last_accepted_model_and_saved_replies":true,"reload_actual28_replies":"verified by first continuation round before learning","candidate_only_no_default_adoption":true});
    Ok((parent, binding))
}

fn observable_cell_proposals(
    native: &NativeSourceRealizer,
    input: &Path,
    positions: &[FrontierPosition],
    episodes: &[Episode],
    context: &BTreeMap<String, Vec<f32>>,
    gradients: &BTreeMap<String, Vec<f32>>,
    root: &Path,
    consumed: bool,
    later: bool,
) -> Result<Vec<CellProposal>> {
    use uor_r4_integer::geometric_context_q4::{ContextQ4Config, NativeContextQ4};
    let metadata: Value = serde_json::from_slice(&fs::read(
        input.join("realizer-native/consumer/metadata.json"),
    )?)?;
    let config: ContextQ4Config = serde_json::from_value(metadata["context"].clone())?;
    let codec = NativeContextQ4::new(
        config,
        &fs::read(input.join("realizer-native/consumer/context-q4.bin"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let tables = codec.table_slices();
    let lanes = config.heads * config.lanes_per_head;
    let mut events = Vec::new();
    let mut singles = Vec::new();
    let mut pairs = Vec::new();
    let mut single_witnesses = 0usize;
    let mut later_query_singles = Vec::new();
    let mut later_query_witnesses = 0usize;
    let mut pair_witnesses = 0usize;
    let coordinate = |family: &str,
                      token: usize,
                      lane: usize,
                      class: usize,
                      classes: usize|
     -> Result<DirectionCoordinate> {
        let name = format!("consumer.context.{family}");
        let index = (token * lanes + lane) * classes + class;
        let shadow = *context
            .get(&name)
            .and_then(|v| v.get(index))
            .ok_or_else(|| invalid("cell source coordinate absent"))?;
        let g = *gradients
            .get(&name)
            .and_then(|v| v.get(index))
            .ok_or_else(|| invalid("cell gradient coordinate absent"))?;
        if !shadow.is_finite() || !g.is_finite() {
            return Err(invalid("cell source/gradient nonfinite"));
        }
        Ok(DirectionCoordinate {
            name,
            index,
            gradient: f64::from(g),
            original_shadow: shadow,
            original_q: (shadow * 4.).round() as i8,
            calibration: false,
            eligibility: vec![
                json!({"token":token,"lane":lane,"class":class,"ranking":"signed coefficient directional surrogate; not desired class probability"}),
            ],
        })
    };
    for (position, episode) in positions.iter().zip(episodes) {
        let trace = native.read(
            episode.frame(),
            &episode.view,
            &episode.query,
            &position.prefix_ids,
        )?;
        let replay = &trace.period_context;
        let consumed_roles = if consumed {
            Some(consumed_replay_roles(
                &trace,
                &episode.query,
                &position.prefix_ids,
            )?)
        } else {
            None
        };
        for (time, &token) in replay.tokens.iter().enumerate() {
            if consumed_roles
                .as_ref()
                .is_some_and(|roles| roles[time].is_none())
            {
                continue;
            }
            for lane in 0..lanes {
                let own = usize::from(replay.states[time][lane]);
                let head = lane / config.lanes_per_head;
                let neighbor = head * config.lanes_per_head
                    + (lane % config.lanes_per_head + 1) % config.lanes_per_head;
                let other = usize::from(replay.states[time][neighbor]);
                let scores = |token_table: &[i32],
                              self_table: &[i32],
                              neighbor_table: Option<&[i32]>,
                              classes: usize,
                              stride: usize| {
                    (0..classes)
                        .map(|class| {
                            i64::from(token_table[(token as usize * lanes + lane) * stride + class])
                                + i64::from(self_table[(lane * 128 + own) * stride + class])
                                + neighbor_table.map_or(0, |table| {
                                    i64::from(table[(lane * 128 + other) * stride + class])
                                })
                        })
                        .collect::<Vec<_>>()
                };
                let roots = scores(
                    tables.token_root,
                    tables.self_root,
                    tables.neighbor_root,
                    120,
                    128,
                );
                let categories = scores(
                    tables.token_category,
                    tables.self_category,
                    tables.neighbor_category,
                    33,
                    64,
                );
                let root_winner = cell_winner(&roots)?;
                let category_winner = cell_winner(&categories)?;
                let observed = time * lanes + lane;
                if root_winner != usize::from(replay.raw_roots[observed])
                    || category_winner != usize::from(replay.categories[observed])
                {
                    return Err(invalid("cell reconstructed native winner differs"));
                }
                let event = events.len();
                events.push(json!({"event":event,"id":position.id,"frontier_step":position.step,"context_time":time,"token_id":token,"lane":lane,"own_new_state":own,"neighbor_lane":neighbor,"neighbor_new_state":other,"root_scores_q24":roots,"category_scores_q24":categories,"root_winner":root_winner,"category_winner":category_winner,"native_code":replay.codes[observed],"consumed_role":consumed_roles.as_ref().and_then(|roles|roles[time]),"copy_pair_presence_at_witness":if consumed{Some(json!({"final_query_lane_present":replay.codes[(replay.tokens.len()-1)*lanes+lane].present,"source_key_lane_present":if time<episode.view.emitted_token_ids().len(){Some(replay.codes[observed].present)}else{None},"source_present_counterparts":(0..episode.view.emitted_token_ids().len()).filter(|source_time|replay.codes[source_time*lanes+lane].present).count(),"root_radius_require_query_and_key_presence":true,"final_category_also_consumed_by_period_stop":time+1==replay.tokens.len()}))}else{None},"tie":"earliest index"}));
                let mut hidden = Vec::new();
                let mut presence = Vec::new();
                for (family, values, classes) in [
                    ("token_root", &roots, 120),
                    ("token_category", &categories, 33),
                ] {
                    for class in 0..classes {
                        let c = coordinate(family, token as usize, lane, class, classes)?;
                        if c.gradient == 0. {
                            continue;
                        }
                        let direction = if c.gradient < 0. { 1 } else { -1 };
                        let Some((delta, new)) =
                            cell_crossing(values, class, c.original_q, direction)?
                        else {
                            continue;
                        };
                        let gain = -c.gradient * f64::from(delta) * 0.25;
                        let edit = CellEdit {
                            coordinate: c,
                            delta_q: delta,
                        };
                        let witness = json!({"event":event,"id":position.id,"context_time":time,"lane":lane,"old_root":root_winner,"old_category":category_winner,"family":family,"old_winner":cell_winner(values)?,"new_winner":new,"class":class,"minimal_legal_directional_crossing":true});
                        if family == "token_root" && category_winner == 0 {
                            hidden.push((edit.clone(), witness.clone(), gain));
                        } else {
                            single_witnesses += 1;
                            if later
                                && !position.prefix_ids.is_empty()
                                && consumed_roles.as_ref().and_then(|roles| roles[time])
                                    == Some("final_query_address")
                            {
                                later_query_witnesses += 1;
                                let mut later_witness = witness.clone();
                                later_witness["reserved_later_query"] = json!(true);
                                later_witness["actual_nonempty_prefix_ids"] =
                                    json!(position.prefix_ids);
                                retain_cell(
                                    &mut later_query_singles,
                                    CellProposal {
                                        edits: vec![edit.clone()],
                                        witness: later_witness,
                                        predicted_gain: gain,
                                    },
                                );
                            }
                            retain_cell(
                                &mut singles,
                                CellProposal {
                                    edits: vec![edit.clone()],
                                    witness: witness.clone(),
                                    predicted_gain: gain,
                                },
                            );
                        }
                        if family == "token_category" && category_winner == 0 && new != 0 {
                            presence.push((edit, witness, gain));
                        }
                    }
                }
                // Presence may require a direction with adverse individual credit;
                // permit it only in a same-event pair with positive total credit.
                if category_winner == 0 {
                    for class in 1..33 {
                        let c = coordinate("token_category", token as usize, lane, class, 33)?;
                        if let Some((delta, new)) =
                            cell_crossing(&categories, class, c.original_q, 1)?
                        {
                            if new != 0 {
                                let gain = -c.gradient * f64::from(delta) * 0.25;
                                presence.push((CellEdit{coordinate:c,delta_q:delta},json!({"event":event,"family":"token_category","old_winner":0,"new_winner":new,"class":class,"minimal_legal_presence_crossing":true}),gain));
                            }
                        }
                    }
                }
                for (edit, witness, gain) in hidden {
                    for (p, pw, pg) in &presence {
                        if gain + pg > 0. {
                            pair_witnesses += 1;
                            retain_cell(
                                &mut pairs,
                                CellProposal {
                                    edits: vec![edit.clone(), p.clone()],
                                    witness: json!({"event":event,"id":position.id,"context_time":time,"lane":lane,"old_root":root_winner,"old_category":category_winner,"root":witness,"presence":pw,"same_token_lane_event":true,"old_category":0}),
                                    predicted_gain: gain + pg,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    let selected = if later {
        later_query_cell_selection(&singles, &later_query_singles, &pairs)?
    } else if consumed {
        consumed_cell_selection(&singles, &pairs)?
    } else {
        singles
            .iter()
            .take(4)
            .chain(pairs.iter().take(4))
            .cloned()
            .collect::<Vec<_>>()
    };
    write(
        &root.join("observable-cell-events.json"),
        &json!({"schema":"uor-r4.observable-cell-events/1","events":events,"native_winner_reconstruction_equal":true,"states":"completed NEW state; same-head next-neighbor; transitions fixed","scores":"authoritative regenerated native Q24 table token+self+neighbor i64 sums"}),
    )?;
    write(
        &root.join("observable-cell-selection.json"),
        &json!({"single_ranked_top4":singles,"later_query_ranked_top4":later_query_singles,"later_query_crossing_witnesses_searched":later_query_witnesses,"later_query_slot_enabled":later,"later_query_fallback":if later && later_query_singles.is_empty(){Some("global fourth singleton; no eligible later-query edge before outcomes")}else{None},"pair_ranked_top4":pairs,"single_crossing_witnesses_searched":single_witnesses,"pair_crossing_witnesses_searched":pair_witnesses,"unselected_offline_edges":"reconstructible from exactscoreevents/source/credit; not model-evaluated","selected":selected,"maximum_selected":8,"consumer_aware":consumed,"current_native_parent_manifest_sha256":sha256_file(&input.join("manifest.json"))?,"rank":if later{"top3global visible singles+best actualnonempty-prefix finalquery singleton; if noeligibleedge globalfourth fallback; top2pairs+bothparts; dedup preserves reserved witness and no refills"}else if consumed{"rank4visible singles+2positive-totalcredit pairs; prospectively admit each selected pair category part regardless partcredit; dedup no quota refill"}else{"positive signed coefficient credit gain per L1 quantum; deterministic edit/witness ties; max4 singles+4 same-event absence pairs"},"frozen_before_candidate_loss":true,"token_only":true,"basis_family_search":"NOT_RUN"}),
    )?;
    Ok(selected)
}

fn verify_cell_witness(
    proposal: &CellProposal,
    native: &NativeSourceRealizer,
    positions: &[FrontierPosition],
    episodes: &[Episode],
    consumed: bool,
    incumbent: Option<&NativeSourceRealizer>,
) -> Result<Value> {
    let w = &proposal.witness;
    let i = positions
        .iter()
        .position(|p| w["id"] == p.id)
        .ok_or_else(|| invalid("cell witness identity absent"))?;
    let time = w["context_time"]
        .as_u64()
        .ok_or_else(|| invalid("cell witness time absent"))? as usize;
    let lane = w["lane"]
        .as_u64()
        .ok_or_else(|| invalid("cell witness lane absent"))? as usize;
    let trace = native.read(
        episodes[i].frame(),
        &episodes[i].view,
        &episodes[i].query,
        &positions[i].prefix_ids,
    )?;
    let replay = &trace.period_context;
    if w["reserved_later_query"] == true {
        if positions[i].prefix_ids.is_empty()
            || w["actual_nonempty_prefix_ids"] != json!(positions[i].prefix_ids)
            || time + 1 != replay.tokens.len()
        {
            return Err(invalid(
                "reserved later-query witness no longer matches reached prefix",
            ));
        }
    }
    let role = if consumed {
        consumed_replay_roles(&trace, &episodes[i].query, &positions[i].prefix_ids)?
            .get(time)
            .copied()
            .flatten()
            .ok_or_else(|| invalid("proposal witness not consumed"))?
    } else {
        "unrestricted_observation"
    };
    let at = time * (replay.heads * replay.lanes_per_head) + lane;
    let oldroot = w["old_root"]
        .as_u64()
        .ok_or_else(|| invalid("cell old root absent"))? as u8;
    let oldcategory = w["old_category"]
        .as_u64()
        .ok_or_else(|| invalid("cell old category absent"))? as u8;
    let (root, category) = if proposal.edits.len() == 2 {
        (
            w["root"]["new_winner"].as_u64(),
            w["presence"]["new_winner"].as_u64(),
        )
    } else if w["family"] == "token_root" {
        (w["new_winner"].as_u64(), Some(u64::from(oldcategory)))
    } else {
        (Some(u64::from(oldroot)), w["new_winner"].as_u64())
    };
    let root = root.ok_or_else(|| invalid("cell predicted root absent"))? as u8;
    let category = category.ok_or_else(|| invalid("cell predicted category absent"))? as u8;
    let code = replay
        .codes
        .get(at)
        .ok_or_else(|| invalid("cell observed code absent"))?;
    let expected = if category == 0 {
        json!({"root":1,"radius_bin":0,"present":false})
    } else {
        json!({"root":root,"radius_bin":category-1,"present":true})
    };
    let old = if oldcategory == 0 {
        json!({"root":1,"radius_bin":0,"present":false})
    } else {
        json!({"root":oldroot,"radius_bin":oldcategory-1,"present":true})
    };
    if replay.raw_roots[at] != root
        || replay.categories[at] != category
        || serde_json::to_value(code)? != expected
        || expected == old
    {
        return Err(invalid(
            "cell native prediction/witness differs or invisible",
        ));
    }
    let consumed_score_effect = if let Some(incumbent) = incumbent {
        let before = incumbent.read(
            episodes[i].frame(),
            &episodes[i].view,
            &episodes[i].query,
            &positions[i].prefix_ids,
        )?;
        consumed_replay_roles(&before, &episodes[i].query, &positions[i].prefix_ids)?;
        let changed_heads=before.source.view_kernel_trace.heads.iter().zip(&trace.source.view_kernel_trace.heads).enumerate().map(|(head,(a,b))|json!({"head":head,"before_copy_q24":a.scores_q24,"after_copy_q24":b.scores_q24,"before_stop_q24":a.no_read_q24,"after_stop_q24":b.no_read_q24,"before_period_q24":before.period_q24[head],"after_period_q24":trace.period_q24[head]})).collect::<Vec<_>>();
        Some(
            json!({"native_head_scores":changed_heads,"copy_or_stop_head_traces_changed":before.source.view_kernel_trace.heads!=trace.source.view_kernel_trace.heads,"copy_or_stop_q24_changed":before.source.view_kernel_trace.heads.iter().zip(&trace.source.view_kernel_trace.heads).any(|(a,b)|a.scores_q24!=b.scores_q24 || a.no_read_q24!=b.no_read_q24),"period_scores_changed":before.period_q24!=trace.period_q24,"actions_changed":before.actions!=trace.actions,"source_observations_changed":before.period_context.codes[..episodes[i].view.emitted_token_ids().len()*replay.heads*replay.lanes_per_head]!=replay.codes[..episodes[i].view.emitted_token_ids().len()*replay.heads*replay.lanes_per_head],"measurement_only_no_extra_acceptance_gate":true}),
        )
    } else {
        None
    };
    Ok(
        json!({"id":positions[i].id,"context_time":time,"lane":lane,"old_code":old,"predicted_code":expected,"actual_code":code,"actual_raw_root":root,"actual_category":category,"equal":true,"consumed_role":role,"consumed_score_effect":consumed_score_effect,"scope":"witness-local minimal directional crossing; repeated token effects retained in full native traces"}),
    )
}
fn observation_learning_limits(consumed: bool, cells: bool) -> (usize, usize) {
    if consumed {
        (3, 24)
    } else if cells {
        (1, 10)
    } else {
        (8, 32)
    }
}
fn observation_incumbent_checkpoint(
    input: &Path,
    out: &Path,
    round: usize,
    rounds: &[Value],
) -> Result<PathBuf> {
    if rounds.len() != round {
        return Err(invalid("incumbent round history differs"));
    }
    if round == 0 {
        return Ok(input.to_path_buf());
    }
    let winner = rounds
        .last()
        .and_then(|previous| previous["selected_proposal"].as_u64())
        .ok_or_else(|| invalid("next round has no accepted incumbent"))?;
    Ok(out.join(format!(
        "round-{:02}/candidate-{winner:02}/checkpoint",
        round - 1
    )))
}
fn context_observation_learn(a: &Args) -> Result<()> {
    let start = Instant::now();
    let later = a.mode == "context-later-query-cells";
    let consumed = matches!(
        a.mode.as_str(),
        "context-consumed-cells" | "context-later-query-cells"
    );
    let cells = matches!(
        a.mode.as_str(),
        "context-observable-cells" | "context-consumed-cells" | "context-later-query-cells"
    );
    let LoadedFinal {
        identity,
        tok,
        episodes,
        retained_sha,
        before,
        ..
    } = load_final(a)?;
    let LoadedContinuation {
        mut source,
        mut native,
        fit,
        mut evaluation,
        mut bins,
    } = load_context_parent(a, &identity, &retained_sha, &before)?;
    let input = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("learner old32 absent"))?;
    let original_input = input;
    let mut cell_parent_inventory = None;
    let cell_input;
    let input = if cells {
        cell_input = a
            .observation_checkpoint
            .as_ref()
            .ok_or_else(|| invalid("observable-cell parent absent"))?;
        let envelope = cell_input
            .parent()
            .ok_or_else(|| invalid("observable-cell envelope absent"))?;
        report_output::verify(envelope)?;
        report_output::verify(cell_input)?;
        let parent: Value = serde_json::from_slice(&fs::read(envelope.join("report.json"))?)?;
        let receipt: Value =
            serde_json::from_slice(&fs::read(cell_input.join("checkpoint.json"))?)?;
        if parent["schema"] != "uor-r4.geometric-observation-learning/1"
            || parent["status"] != "completed"
            || parent["accepted_quanta"] != 2
            || parent["native_candidate_evaluations"] != 12
            || parent["parent_checkpoint_manifest_sha256"]
                != sha256_file(&original_input.join("manifest.json"))?
            || parent["saved_identity"] != serde_json::to_value(&identity)?
            || parent["retained_report_sha256"] != retained_sha
            || parent["final_checkpoint"] != receipt
        {
            return Err(invalid("observable-cell parent lineage differs"));
        }
        let bytes = fs::read(&a.tokenizer)?;
        source = SourceRealizerWeights::load_source(&cell_input.join("realizer-source"), &bytes)?;
        native =
            NativeSourceRealizer::load(&cell_input.join("realizer-native"), &source, &identity)?;
        let old_bins = bins;
        bins = bin_files(&cell_input.join("realizer-native"))?;
        if old_bins.keys().ne(bins.keys())
            || old_bins
                .iter()
                .any(|(name, sha)| name != "consumer/context-q4.bin" && bins.get(name) != Some(sha))
        {
            return Err(invalid("observable-cell frozen payload differs from0032"));
        }
        evaluation = json!({"ownprefix":parent["final_original_generation"],"construction_ownprefix":parent["final_construction_generation"]});
        cell_parent_inventory = Some(source_files(original_input)?);
        cell_input
    } else {
        input
    };
    let observation_input = input;
    let mut observation_parent_inventory = None;
    let consumed_parent;
    let mut continuation_binding = None;
    let input = if later {
        consumed_parent = a
            .consumed_checkpoint
            .as_ref()
            .ok_or_else(|| invalid("accepted consumed continuation absent"))?;
        let (parent, binding) = admit_consumed_continuation(
            consumed_parent,
            observation_input,
            &identity,
            &retained_sha,
            &bins,
        )?;
        observation_parent_inventory = Some(source_files(observation_input)?);
        let old_bits = parameter_bits(&source.parameters())?;
        let bytes = fs::read(&a.tokenizer)?;
        source =
            SourceRealizerWeights::load_source(&consumed_parent.join("realizer-source"), &bytes)?;
        if !only_token_observation_bits_changed(&old_bits, &parameter_bits(&source.parameters())?) {
            return Err(invalid(
                "consumed continuation changed frozen source parameters",
            ));
        }
        native = NativeSourceRealizer::load(
            &consumed_parent.join("realizer-native"),
            &source,
            &identity,
        )?;
        bins = bin_files(&consumed_parent.join("realizer-native"))?;
        evaluation = json!({"ownprefix":parent["final_original_generation"],"construction_ownprefix":parent["final_construction_generation"]});
        continuation_binding = Some(binding);
        consumed_parent
    } else {
        input
    };
    let input_files = source_files(input)?;
    let parent_context = fs::read(input.join("realizer-native/consumer/context-q4.bin"))?;
    let initial_bits = parameter_bits(&source.parameters())?;
    let bytes = fs::read(&a.tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let template = episodes
        .first()
        .ok_or_else(|| invalid("original episodes absent"))?;
    let construction =
        composition_panel(template, &tok, &compiler, native.binding().eos_token_id())?;
    let saved: Value = serde_json::from_slice(&fs::read(
        original_input
            .parent()
            .ok_or_else(|| invalid("composition envelope absent"))?
            .join("construction-panel/panel.json"),
    )?)?;
    if saved["original_cases"] != serde_json::to_value(episode_labels(&episodes))?
        || saved["construction_cases"] != serde_json::to_value(episode_labels(&construction))?
    {
        return Err(invalid("learner fixed panels differ"));
    }
    if later {
        let envelope = input
            .parent()
            .ok_or_else(|| invalid("consumed envelope absent"))?;
        let parent_panel: Value =
            serde_json::from_slice(&fs::read(envelope.join("frozen-panel.json"))?)?;
        if parent_panel
            != json!({"original":episode_labels(&episodes),"construction":episode_labels(&construction)})
        {
            return Err(invalid("consumed continuation panels differ"));
        }
    }
    write(
        &a.out.join("frozen-panel.json"),
        &json!({"original":episode_labels(&episodes),"construction":episode_labels(&construction)}),
    )?;
    let mut accepted = 0usize;
    let mut total_accepted_l1_quanta = 0usize;
    let mut candidate_count = 0usize;
    let mut rounds: Vec<Value> = Vec::new();
    let mut last_accepted_replies: Option<(Value, Value)> = None;
    let mut stop = if consumed {
        "accepted_candidate_limit"
    } else {
        "accepted_quantum_limit"
    };
    let (round_limit, candidate_limit) = observation_learning_limits(consumed, cells);
    for round in 0..round_limit {
        deadline(start, a)?;
        let root = a.out.join(format!("round-{round:02}"));
        report_output::claim(&root)?;
        let old_original = generate(
            &format!("learn-{round:02}-incumbent-original"),
            &native,
            &episodes,
            &tok,
            start,
            a,
        )?;
        let old_construction = generate(
            &format!("learn-{round:02}-incumbent-construction"),
            &native,
            &construction,
            &tok,
            start,
            a,
        )?;
        if round == 0
            && (old_original["rows"] != evaluation["ownprefix"]["rows"]
                || old_construction["rows"] != evaluation["construction_ownprefix"]["rows"])
        {
            return Err(invalid("learner saved28 initial rows differ"));
        }
        if old_construction["complete_answers"] == 8 {
            write(
                &root.join("report.json"),
                &json!({"round":round,"accepted_before_round":accepted,
                "stop_reason":"construction_complete","incumbent_original_generation":old_original,
                "incumbent_construction_generation":old_construction,
                "incumbent_original_canonical":context_direction_measure(&native,&episodes,start,a)?,
                "incumbent_construction_canonical":context_direction_measure(&native,&construction,start,a)?,
                "proposals":[],"selected_proposal":null}),
            )?;
            report_output::seal(&root)?;
            report_output::verify(&root)?;
            rounds.push(
                json!({"round":round,"report_path":format!("round-{round:02}/report.json"),
                "report_sha256":sha256_file(&root.join("report.json"))?,"selected_proposal":null,
                "candidate_evaluations":0,"stop_reason":"construction_complete"}),
            );
            stop = "construction_complete";
            break;
        }
        let positions = learning_frontiers(&construction, &old_construction, &tok, &native)?;
        write(
            &root.join("frozen-frontier.json"),
            &json!({"positions":positions,"completed_case_policy":"actual accepted pre-EOS prefix, validated membership/continuity; EOS label offline only","candidate_selection_before_predictions":true}),
        )?;
        let baseline = frontier_measure(&positions, &construction, &native, start, a)?;
        let baseline_ce = baseline["mean_episode_ce"]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("learner baseline frontier CE nonfinite"))?;
        let old_original_ce = context_direction_measure(&native, &episodes, start, a)?;
        let old_construction_ce = context_direction_measure(&native, &construction, start, a)?;
        let params = source.parameters();
        let all_bits = parameter_bits(&params)?;
        let context = params
            .iter()
            .filter(|(name, _)| name.starts_with("consumer.context."))
            .map(|(name, var)| Ok((name.clone(), var.flatten_all()?.to_vec1::<f32>()?)))
            .collect::<Result<BTreeMap<String, Vec<f32>>>>()?;
        let measured = frontier_batch(&positions, &construction, &source, &native, start, a)?;
        let mut gradients = BTreeMap::new();
        let mut eligible = Vec::new();
        for (name, values) in &context {
            let g = measured
                .gradients
                .get(name)
                .map(|t| t.flatten_all()?.to_vec1::<f32>())
                .transpose()?
                .unwrap_or_else(|| vec![0.; values.len()]);
            if g.len() != values.len() {
                return Err(invalid("learner gradient shape differs"));
            }
            for (index, (&gradient, &shadow)) in g.iter().zip(values).enumerate() {
                if !gradient.is_finite() || !shadow.is_finite() {
                    return Err(invalid("learner nonfinite source/credit"));
                }
                if gradient != 0. {
                    eligible.push(DirectionCoordinate{name:name.clone(),index,
                    gradient:f64::from(gradient),original_shadow:shadow,original_q:(shadow*4.).round() as i8,
                    calibration:false,eligibility:vec![json!({"objective":"round-frozen actual first error/EOS anchor mean8 CE","coordinate_class":frontier_coordinate_class(name)})]});
                }
            }
            gradients.insert(name.clone(), g);
        }
        let selected = select_frontier_direction(eligible);
        let current_checkpoint = observation_incumbent_checkpoint(input, &a.out, round, &rounds)?;
        let cell_proposals = if cells {
            Some(observable_cell_proposals(
                &native,
                &current_checkpoint,
                &positions,
                &construction,
                &context,
                &gradients,
                &root,
                consumed,
                later,
            )?)
        } else {
            None
        };
        write(
            &root.join("selection.json"),
            &json!({"selected":selected,"gradient_values":gradients,"credit":measured.report,"selector":if cells{"actual prospective proposals bound in observable-cell-selection.json; strongest credit coordinates below are diagnostic only"}else{"strongest finite nonzero interior q[-6,6] per root/category, ties name/index, both signs; no replacement"},"maximum_candidates":if cells{8}else{4},"biased_credit_not_discrete_derivative":true}),
        )?;
        let packed_before =
            fs::read(current_checkpoint.join("realizer-native/consumer/context-q4.bin"))?;
        let mut proposals = Vec::new();
        let mut declared = cell_proposals.clone().unwrap_or_else(|| {
            selected
                .iter()
                .flat_map(|c| {
                    [-1i8, 1].into_iter().map(|delta| CellProposal {
                        edits: vec![CellEdit {
                            coordinate: c.clone(),
                            delta_q: delta,
                        }],
                        witness: Value::Null,
                        predicted_gain: -c.gradient * f64::from(delta) * 0.25,
                    })
                })
                .collect()
        });
        let main_proposal_count = declared.len();
        let mut proposal_index = 0usize;
        while proposal_index < declared.len() {
            let proposal = &declared[proposal_index];
            let c = &proposal.edits[0].coordinate;
            let delta = proposal.edits[0].delta_q;
            deadline(start, a)?;
            if candidate_count >= candidate_limit {
                return Err(invalid("learner candidate cap exceeded"));
            }
            let q = c.original_q + delta;
            let var = params
                .get(&c.name)
                .ok_or_else(|| invalid("learner Var absent"))?;
            let original = context
                .get(&c.name)
                .ok_or_else(|| invalid("learner shadow absent"))?;
            let mut changed = original.clone();
            *changed
                .get_mut(c.index)
                .ok_or_else(|| invalid("learner index absent"))? = f32::from(q) * 0.25;
            let candidate_root = root.join(format!("candidate-{:02}", proposals.len()));
            report_output::claim(&candidate_root)?;
            var.set(&Tensor::from_vec(changed, var.shape(), &Device::Cpu)?)?;
            for edit in proposal.edits.iter().skip(1) {
                let v = params
                    .get(&edit.coordinate.name)
                    .ok_or_else(|| invalid("cell paired Var absent"))?;
                let mut values = context
                    .get(&edit.coordinate.name)
                    .ok_or_else(|| invalid("cell paired source absent"))?
                    .clone();
                values[edit.coordinate.index] =
                    f32::from(edit.coordinate.original_q + edit.delta_q) * 0.25;
                v.set(&Tensor::from_vec(values, v.shape(), &Device::Cpu)?)?;
            }
            let attempt = (|| -> Result<Value> {
                let mut expected = all_bits.clone();
                *expected
                    .get_mut(&c.name)
                    .and_then(|v| v.get_mut(c.index))
                    .ok_or_else(|| invalid("learner expected bits absent"))? =
                    (f32::from(q) * 0.25).to_bits();
                for edit in proposal.edits.iter().skip(1) {
                    *expected
                        .get_mut(&edit.coordinate.name)
                        .and_then(|v| v.get_mut(edit.coordinate.index))
                        .ok_or_else(|| invalid("cell paired expected absent"))? =
                        (f32::from(edit.coordinate.original_q + edit.delta_q) * 0.25).to_bits();
                }
                if parameter_bits(&params)? != expected {
                    return Err(invalid("learner other source bits changed"));
                }
                let (receipt, loaded) = checkpoint(
                    &candidate_root.join("checkpoint"),
                    &source,
                    &identity,
                    &bytes,
                    &episodes,
                    0,
                    "native_screened_observation_proposal",
                )?;
                let packed = bin_files(&candidate_root.join("checkpoint/realizer-native"))?;
                if bins.keys().ne(packed.keys())
                    || bins.iter().any(|(name, sha)| {
                        name != "consumer/context-q4.bin" && packed.get(name) != Some(sha)
                    })
                {
                    return Err(invalid("learner readout/table/inventory changed"));
                }
                let family = c
                    .name
                    .strip_prefix("consumer.context.")
                    .ok_or_else(|| invalid("learner family prefix absent"))?;
                let mut offset = 0usize;
                let mut found = false;
                for name in uor_r4_integer::geometric_context_q4::FAMILY_NAMES {
                    if name == family {
                        found = true;
                        break;
                    }
                    offset += context
                        .get(&format!("consumer.context.{name}"))
                        .map_or(0, Vec::len);
                }
                if !found {
                    return Err(invalid("learner packed family absent"));
                }
                let candidate_packed = fs::read(
                    candidate_root.join("checkpoint/realizer-native/consumer/context-q4.bin"),
                )?;
                let quantum = if cells {
                    verify_cell_packed_edits(
                        &packed_before,
                        &candidate_packed,
                        &context,
                        &proposal.edits,
                    )?
                } else {
                    quantum_change(&packed_before, &candidate_packed, offset + c.index, delta)?
                };
                let witness = if cells && proposal_index < main_proposal_count {
                    Some(verify_cell_witness(
                        proposal,
                        &loaded,
                        &positions,
                        &construction,
                        consumed,
                        if consumed { Some(&native) } else { None },
                    )?)
                } else {
                    None
                };
                let frontier = frontier_measure(&positions, &construction, &loaded, start, a)?;
                let original_ce = context_direction_measure(&loaded, &episodes, start, a)?;
                let construction_ce = context_direction_measure(&loaded, &construction, start, a)?;
                let original_generation = generate(
                    &format!("learn-{round:02}-candidate-{:02}-original", proposals.len()),
                    &loaded,
                    &episodes,
                    &tok,
                    start,
                    a,
                )?;
                let construction_generation = generate(
                    &format!(
                        "learn-{round:02}-candidate-{:02}-construction",
                        proposals.len()
                    ),
                    &loaded,
                    &construction,
                    &tok,
                    start,
                    a,
                )?;
                let cumulative = packed_byte_changes(
                    &parent_context,
                    &fs::read(
                        candidate_root.join("checkpoint/realizer-native/consumer/context-q4.bin"),
                    )?,
                )?;
                let result = json!({"cumulative_packed_byte_changes_from_parent":cumulative,"coordinate":c,"delta_q":delta,"observable_cell_proposal":if cells{Some(proposal)}else{None},"observable_witness":witness,"ablation_only":cells && proposal_index>=main_proposal_count,"checkpoint":receipt,"quantum":quantum,"native_payload_sha256":packed,"all_other_source_bits_fixed":true,"readouts_tables_transitions_fixed":true,"frontier":frontier,"delta_round_frozen_ce":frontier["mean_episode_ce"].as_f64().map(|v|v-baseline_ce),"original_canonical":original_ce,"construction_canonical":construction_ce,"original_canonical_comparison":context_direction_comparison(&old_original_ce,&original_ce)?,"construction_canonical_comparison":context_direction_comparison(&old_construction_ce,&construction_ce)?,"original_generation":original_generation,"construction_generation":construction_generation,"original_reply_comparison":generation_comparison(&old_original,&original_generation)?,"construction_reply_comparison":generation_comparison(&old_construction,&construction_generation)?});
                write(&candidate_root.join("result.json"), &result)?;
                Ok(result)
            })();
            var.set(&Tensor::from_vec(
                original.clone(),
                var.shape(),
                &Device::Cpu,
            )?)?;
            for edit in proposal.edits.iter().skip(1) {
                let v = params
                    .get(&edit.coordinate.name)
                    .ok_or_else(|| invalid("cell restore Var absent"))?;
                v.set(&Tensor::from_vec(
                    context[&edit.coordinate.name].clone(),
                    v.shape(),
                    &Device::Cpu,
                )?)?;
            }
            if parameter_bits(&params)? != all_bits {
                return Err(invalid("learner shadow restore differs"));
            }
            if let Err(error) = &attempt {
                write(
                    &candidate_root.join("error.json"),
                    &json!({"error":error.to_string(),"source_restored":true}),
                )?;
            }
            report_output::seal(&candidate_root)?;
            report_output::verify(&candidate_root)?;
            proposals.push(attempt?);
            candidate_count += 1;
            write(
                &root.join("proposal-progress.json"),
                &json!({"completed_candidates":candidate_count,"round_completed_candidates":proposals.len(),"raw_results":"candidate-NN/result.json"}),
            )?;
            proposal_index += 1;
            if cells && !consumed && proposal_index == main_proposal_count {
                let scores = proposals
                    .iter()
                    .map(|v| v["frontier"]["mean_episode_ce"].as_f64())
                    .collect::<Vec<_>>();
                if let Some(best) = best_native_proposal(baseline_ce, &scores) {
                    if declared[best].edits.len() == 2 {
                        let pair = declared[best].clone();
                        for edit in pair.edits {
                            declared.push(CellProposal{predicted_gain:-edit.coordinate.gradient*f64::from(edit.delta_q)*0.25,edits:vec![edit],witness:json!({"ablation_of_candidate":best,"paired_witness":pair.witness})});
                        }
                    }
                }
            }
        }
        let scores = proposals
            .iter()
            .take(main_proposal_count)
            .map(|v| v["frontier"]["mean_episode_ce"].as_f64())
            .collect::<Vec<_>>();
        let best = best_native_proposal(baseline_ce, &scores);
        let proposal_summaries = proposals.iter().enumerate().map(|(index,result)| {
            Ok(json!({"index":index,"result_path":format!("candidate-{index:02}/result.json"),
                "result_sha256":sha256_file(&root.join(format!("candidate-{index:02}/result.json")))?,
                "coordinate":result["coordinate"],"delta_q":result["delta_q"],
                "ablation_only":result["ablation_only"],"observable_cell_proposal":result["observable_cell_proposal"],"round_frozen_ce":result["frontier"]["mean_episode_ce"],
                "delta_round_frozen_ce":result["delta_round_frozen_ce"],
                "original_complete":result["original_generation"]["complete_answers"],
                "construction_complete":result["construction_generation"]["complete_answers"]}))
        }).collect::<Result<Vec<Value>>>()?;
        let report = json!({"round":round,"accepted_before_round":accepted,"frontier_sha256":sha256_file(&root.join("frozen-frontier.json"))?,"positions":positions,"baseline_frontier":baseline,"incumbent_original_generation":old_original,"incumbent_construction_generation":old_construction,"incumbent_original_canonical":old_original_ce,"incumbent_construction_canonical":old_construction_ce,"proposals":proposal_summaries,"selected_proposal":best,"main_proposal_count":main_proposal_count,"ablation_count":proposals.len()-main_proposal_count,"acceptance":"strict native CE decrease on same frozen round positions; deterministic first proposal ties; preservation diagnostic only","loss_scope":"iteration-local, not a comparable cross-round curve"});
        write(&root.join("report.json"), &report)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        rounds.push(json!({"round":round,"report_path":format!("round-{round:02}/report.json"),"report_sha256":sha256_file(&root.join("report.json"))?,"selected_proposal":best,"baseline_round_frozen_ce":baseline_ce,"accepted_round_frozen_ce":best.and_then(|index|scores[index]),"candidate_evaluations":proposals.len()}));
        write(&a.out.join("learning-progress.json"), &rounds)?;
        if let Some(best) = best {
            last_accepted_replies = Some((
                proposals[best]["original_generation"].clone(),
                proposals[best]["construction_generation"].clone(),
            ));
            let path = root.join(format!("candidate-{best:02}/checkpoint"));
            source = SourceRealizerWeights::load_source(&path.join("realizer-source"), &bytes)?;
            native = NativeSourceRealizer::load(&path.join("realizer-native"), &source, &identity)?;
            accepted += 1;
            total_accepted_l1_quanta += declared[best]
                .edits
                .iter()
                .map(|e| usize::from(e.delta_q.unsigned_abs()))
                .sum::<usize>();
            if cells && !consumed {
                stop = "single_observable_cell_round_completed";
            }
        } else {
            stop = "no_native_improving_declared_proposal";
            break;
        }
    }
    let final_bits = parameter_bits(&source.parameters())?;
    if initial_bits.keys().ne(final_bits.keys())
        || initial_bits.iter().any(|(name, bits)| {
            frontier_coordinate_class(name).is_none() && final_bits.get(name) != Some(bits)
        })
    {
        return Err(invalid("learner changed transition/readout source"));
    }
    let (receipt, loaded) = checkpoint(
        &a.out.join("final-checkpoint"),
        &source,
        &identity,
        &bytes,
        &episodes,
        0,
        "bounded_observation_learning_candidate_unadopted",
    )?;
    let final_bins = bin_files(&a.out.join("final-checkpoint/realizer-native"))?;
    if bins.keys().ne(final_bins.keys())
        || bins.iter().any(|(name, sha)| {
            name != "consumer/context-q4.bin" && final_bins.get(name) != Some(sha)
        })
    {
        return Err(invalid("learner final frozen payload mismatch"));
    }
    let final_packed = fs::read(
        a.out
            .join("final-checkpoint/realizer-native/consumer/context-q4.bin"),
    )?;
    let signed = |n: u8| {
        if n >= 8 {
            i16::from(n) - 16
        } else {
            i16::from(n)
        }
    };
    let accepted_l1_quanta = parent_context
        .iter()
        .zip(&final_packed)
        .map(|(&a, &b)| {
            (signed(a & 15) - signed(b & 15)).abs() + (signed(a >> 4) - signed(b >> 4)).abs()
        })
        .sum::<i16>();
    let final_original = generate("learn-final-original", &loaded, &episodes, &tok, start, a)?;
    let final_construction = generate(
        "learn-final-construction",
        &loaded,
        &construction,
        &tok,
        start,
        a,
    )?;
    let replay_original = generate(
        "learn-final-replay-original",
        &native,
        &episodes,
        &tok,
        start,
        a,
    )?;
    let replay_construction = generate(
        "learn-final-replay-construction",
        &native,
        &construction,
        &tok,
        start,
        a,
    )?;
    if final_original["rows"] != replay_original["rows"]
        || final_construction["rows"] != replay_construction["rows"]
    {
        return Err(invalid("learner final reload full replies differ"));
    }
    if let Some((original, construction)) = &last_accepted_replies {
        if original["rows"] != final_original["rows"]
            || construction["rows"] != final_construction["rows"]
        {
            return Err(invalid(
                "final reload differs from final accepted candidate replies",
            ));
        }
    }
    if source_files(input)? != input_files {
        return Err(invalid("learner parent inputs changed"));
    }
    report_output::verify(input)?;
    if let Some(inventory) = observation_parent_inventory {
        if source_files(observation_input)? != inventory {
            return Err(invalid("observation ancestor inputs changed"));
        }
        report_output::verify(observation_input)?;
    }
    if let Some(inventory) = cell_parent_inventory {
        if source_files(original_input)? != inventory {
            return Err(invalid("original0032 inputs changed"));
        }
        report_output::verify(original_input)?;
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":if later{"uor-r4.geometric-later-query-cells/1"}else if consumed{"uor-r4.geometric-consumed-cells/1"}else if cells{"uor-r4.geometric-observable-cells/1"}else{"uor-r4.geometric-observation-learning/1"},"mode":a.mode,"status":"completed","source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"saved_identity":identity,"fit_source_commit":fit["source_commit"],"retained_report_sha256":retained_sha,"parent_checkpoint_manifest_sha256":sha256_file(&input.join("manifest.json"))?,"observation_base_checkpoint_manifest_sha256":sha256_file(&observation_input.join("manifest.json"))?,"continuation_binding":continuation_binding,"input_files_sha256":input_files,"input_files_unchanged":true,"saved_parent28_rows_exact":true,"optimizer_updates":0,"accepted_candidates":accepted,"accepted_l1_quanta":accepted_l1_quanta,"total_accepted_edit_l1_quanta":total_accepted_l1_quanta,"accepted_l1_scope":"net packed difference from starting parent; total edit L1 charged separately","original0032_manifest_sha256":sha256_file(&original_input.join("manifest.json"))?,"accepted_quanta":if cells{Value::Null}else{json!(accepted)},"native_candidate_evaluations":candidate_count,"maximum_accepted_candidates":round_limit,"maximum_accepted_quanta":if cells{Value::Null}else{json!(8)},"maximum_native_candidates":candidate_limit,"stop_reason":stop,"rounds":rounds,"final_checkpoint":receipt,"final_native_payload_sha256":final_bins,"final_original_generation":final_original,"final_construction_generation":final_construction,"final_original_canonical":context_direction_measure(&loaded,&episodes,start,a)?,"final_construction_canonical":context_direction_measure(&loaded,&construction,start,a)?,"final_independent_reload_full_replies_equal":true,"final_accepted_candidate_replies_equal":last_accepted_replies.is_some(),"transitions_readouts_tables_fixed":true,"no_adopted_model":true,"scope":"bounded root/category learning on8 exposed construction plus20 preservation development cases; no heldout/general-chat/geometry-advantage/energy qualification","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}
fn context_direction(a: &Args) -> Result<()> {
    let frontier = a.mode == "context-frontier-direction";
    let start = Instant::now();
    let LoadedFinal {
        identity,
        tok,
        episodes,
        retained_sha,
        before,
        ..
    } = load_final(a)?;
    let LoadedContinuation {
        source,
        native,
        fit,
        evaluation,
        bins,
    } = load_context_parent(a, &identity, &retained_sha, &before)?;
    let input = a
        .transfer_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("context direction old32 missing"))?;
    let input_files = source_files(input)?;
    let bytes = fs::read(&a.tokenizer)?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let first = episodes
        .iter()
        .find(|e| e.id == "mw2-0246-s0-d0")
        .ok_or_else(|| invalid("fixed singer original case absent"))?;
    let fixed_prefix = vec![1130u32, 284, 16];
    if first.target.get(..3) != Some(fixed_prefix.as_slice()) {
        return Err(invalid("fixed singer prefix differs from artifact targets"));
    }
    let fixed_target = *first
        .target
        .get(3)
        .ok_or_else(|| invalid("fixed singer EOS label absent"))?;
    let template = episodes
        .first()
        .ok_or_else(|| invalid("context direction original cases absent"))?;
    let construction =
        composition_panel(template, &tok, &compiler, native.binding().eos_token_id())?;
    let saved_panel: Value = serde_json::from_slice(&fs::read(
        input
            .parent()
            .ok_or_else(|| invalid("composition envelope absent"))?
            .join("construction-panel/panel.json"),
    )?)?;
    if saved_panel["original_cases"] != serde_json::to_value(episode_labels(&episodes))?
        || saved_panel["construction_cases"] != serde_json::to_value(episode_labels(&construction))?
    {
        return Err(invalid("context direction frozen panels differ"));
    }
    write(
        &a.out.join("frozen-panel.json"),
        &json!({"original":episode_labels(&episodes),"construction":episode_labels(&construction),"fixed_singer_id":first.id,"fixed_prefix_ids":fixed_prefix,"fixed_target_label_only":fixed_target,"scope":"no transfer predictions"}),
    )?;
    let original_generation = generate(
        "context-direction-parent-original",
        &native,
        &episodes,
        &tok,
        start,
        a,
    )?;
    let construction_generation = generate(
        "context-direction-parent-construction",
        &native,
        &construction,
        &tok,
        start,
        a,
    )?;
    if original_generation["rows"] != evaluation["ownprefix"]["rows"]
        || construction_generation["rows"] != evaluation["construction_ownprefix"]["rows"]
    {
        return Err(invalid("context direction saved28 parent rows differ"));
    }
    let frontiers = if frontier {
        frontier_positions(&construction, &construction_generation)?
    } else {
        Vec::new()
    };
    let baseline_frontier = if frontier {
        Some(frontier_measure(
            &frontiers,
            &construction,
            &native,
            start,
            a,
        )?)
    } else {
        None
    };
    if frontier {
        write(
            &a.out.join("frozen-frontier.json"),
            &json!({"positions":frontiers,"derived_from_saved_exact_baseline":true,"selection_before_candidate_predictions":true}),
        )?;
    }
    let baseline_original = context_direction_measure(&native, &episodes, start, a)?;
    let baseline_construction = context_direction_measure(&native, &construction, start, a)?;
    let fixed_baseline = fixed_failure_trace(
        "old32-baseline",
        &native,
        first,
        3,
        &fixed_prefix,
        fixed_target,
    )?;
    let params = source.parameters();
    let all_bits = parameter_bits(&params)?;
    let context = params
        .iter()
        .filter(|(n, _)| n.starts_with("consumer.context."))
        .map(|(n, v)| Ok((n.clone(), v.flatten_all()?.to_vec1::<f32>()?)))
        .collect::<Result<BTreeMap<String, Vec<f32>>>>()?;
    // Original mode reuses the first full-answer construction B8 objective.
    // Frontier mode averages one saved actual first-error loss per episode.
    // Neither mode registers or changes a parameter through an optimizer.
    let measured = if frontier {
        frontier_batch(&frontiers, &construction, &source, &native, start, a)?
    } else {
        batch(
            &(0..8).collect::<Vec<_>>(),
            &construction,
            &source,
            &native,
            start,
            a,
        )?
    };
    let mut gradients = BTreeMap::new();
    let mut eligible = Vec::new();
    for (name, values) in &context {
        let g = if let Some(t) = measured.gradients.get(name) {
            t.flatten_all()?.to_vec1::<f32>()?
        } else {
            vec![0.; values.len()]
        };
        if g.len() != values.len() {
            return Err(invalid("context direction gradient dimensions differ"));
        }
        for (index, (&gradient, &shadow)) in g.iter().zip(values).enumerate() {
            if !gradient.is_finite() || !shadow.is_finite() {
                return Err(invalid("context direction nonfinite source/credit"));
            }
            if gradient != 0. {
                eligible.push(DirectionCoordinate{name:name.clone(),index,gradient:f64::from(gradient),original_shadow:shadow,original_q:(shadow*4.).round() as i8,calibration:false,eligibility:vec![json!({"objective":if frontier{"fixed8 construction first actual divergence equal-episode token CE"}else{"fixed8 construction equal-episode token/EOS CE"},"coordinate_class":if frontier{frontier_coordinate_class(name)}else{context_coordinate_class(name)}})]});
            }
        }
        gradients.insert(name.clone(), g);
    }
    let selected = if frontier {
        select_frontier_direction(eligible)
    } else {
        select_context_direction(eligible)
    };
    write(
        &a.out.join("selection.json"),
        &json!({"selected":selected,"selector":if frontier{"strongest absolute nonzero finite first-actual-divergence B8 gradient per root and category class; q[-6,6]; ties name/index; max2, no replacement after candidate outcomes"}else{"strongest absolute nonzero finite constructionB8 gradient per transition and root/category observation class; q[-6,6]; ties name/index; max2, no replacement after candidate outcomes"},"gradient_values":gradients,"construction_b8":measured.report,"credit_scope":"existing biased context native-conditioned STE; neither exact discrete derivative nor finite quantum magnitude guarantee","maximum_native_candidates":4,"selected_before_candidate_predictions":true}),
    )?;
    let packed_before = fs::read(input.join("realizer-native/consumer/context-q4.bin"))?;
    let mut results = Vec::new();
    for c in &selected {
        for delta in [-1i8, 1] {
            deadline(start, a)?;
            let q = c.original_q + delta;
            if !(-7..=7).contains(&q) {
                return Err(invalid(
                    "preregistered context coordinate direction illegal",
                ));
            }
            let var = params
                .get(&c.name)
                .ok_or_else(|| invalid("context direction Var absent"))?;
            let saved = context
                .get(&c.name)
                .ok_or_else(|| invalid("context direction source absent"))?;
            let mut changed = saved.clone();
            let destination = changed
                .get_mut(c.index)
                .ok_or_else(|| invalid("context direction coordinate outside source"))?;
            *destination = f32::from(q) * 0.25;
            let root = a.out.join(format!("candidate-{:02}", results.len()));
            report_output::claim(&root)?;
            var.set(&Tensor::from_vec(changed, var.shape(), &Device::Cpu)?)?;
            let attempt = (|| -> Result<Value> {
                let mut expected_bits = all_bits.clone();
                *expected_bits
                    .get_mut(&c.name)
                    .and_then(|v| v.get_mut(c.index))
                    .ok_or_else(|| invalid("expected context source coordinate absent"))? =
                    (f32::from(q) * 0.25).to_bits();
                if parameter_bits(&params)? != expected_bits {
                    return Err(invalid(
                        "context candidate altered other source coefficients",
                    ));
                }
                let (receipt, loaded) = checkpoint(
                    &root.join("checkpoint"),
                    &source,
                    &identity,
                    &bytes,
                    &episodes,
                    0,
                    "one_quarter_context_direction_unadopted",
                )?;
                let packed = bin_files(&root.join("checkpoint/realizer-native"))?;
                if bins.keys().ne(packed.keys())
                    || bins.iter().any(|(name, sha)| {
                        name != "consumer/context-q4.bin" && packed.get(name) != Some(sha)
                    })
                {
                    return Err(invalid(
                        "context candidate changed readout/table payload or inventory",
                    ));
                }
                let family = c
                    .name
                    .strip_prefix("consumer.context.")
                    .ok_or_else(|| invalid("context coordinate prefix differs"))?;
                let mut offset = 0usize;
                let mut found = false;
                for name in uor_r4_integer::geometric_context_q4::FAMILY_NAMES {
                    if name == family {
                        found = true;
                        break;
                    }
                    offset += context
                        .get(&format!("consumer.context.{name}"))
                        .map_or(0, Vec::len);
                }
                if !found {
                    return Err(invalid("context packed family unknown"));
                }
                let quantum = quantum_change(
                    &packed_before,
                    &fs::read(root.join("checkpoint/realizer-native/consumer/context-q4.bin"))?,
                    offset + c.index,
                    delta,
                )?;
                let original = context_direction_measure(&loaded, &episodes, start, a)?;
                let construct = context_direction_measure(&loaded, &construction, start, a)?;
                let original_reply = generate(
                    &format!("context-direction-{:02}-original", results.len()),
                    &loaded,
                    &episodes,
                    &tok,
                    start,
                    a,
                )?;
                let construct_reply = generate(
                    &format!("context-direction-{:02}-construction", results.len()),
                    &loaded,
                    &construction,
                    &tok,
                    start,
                    a,
                )?;
                let shadow_delta = f64::from(q) * 0.25 - f64::from(c.original_shadow);
                let mut value = json!({"coordinate":c,"delta_q":delta,"status":"completed","checkpoint":receipt,"quantum":quantum,"all_other_source_bits_fixed":true,"readout_and_table_payloads_fixed":true,"native_payload_sha256":packed,"shadow_delta_nat":shadow_delta,"biased_predicted_delta_ce_shadow":c.gradient*shadow_delta,"biased_predicted_delta_ce_quarter_grid":c.gradient*f64::from(delta)*0.25,"original_canonical":original,"construction_canonical":construct,"original_canonical_comparison":context_direction_comparison(&baseline_original,&original)?,"construction_canonical_comparison":context_direction_comparison(&baseline_construction,&construct)?,"original_generation":original_reply,"construction_generation":construct_reply,"original_reply_comparison":generation_comparison(&original_generation,&original_reply)?,"construction_reply_comparison":generation_comparison(&construction_generation,&construct_reply)?,"fixed_singer_prefix3_trace":fixed_failure_trace("candidate",&loaded,first,3,&fixed_prefix,fixed_target)?});
                if frontier {
                    value["frontier"] =
                        frontier_measure(&frontiers, &construction, &loaded, start, a)?;
                    value["frontier_delta_mean_episode_ce"] = json!(baseline_frontier
                        .as_ref()
                        .and_then(|v| v["mean_episode_ce"].as_f64())
                        .zip(value["frontier"]["mean_episode_ce"].as_f64())
                        .map(|(b, c)| c - b));
                }
                write(&root.join("result.json"), &value)?;
                Ok(value)
            })();
            // Restore before propagating evaluation/serialization failure.
            var.set(&Tensor::from_vec(saved.clone(), var.shape(), &Device::Cpu)?)?;
            if parameter_bits(&params)? != all_bits {
                return Err(invalid(
                    "context direction original source bits not restored",
                ));
            }
            if let Err(error) = &attempt {
                write(
                    &root.join("error.json"),
                    &json!({"coordinate":c,"delta_q":delta,"error":error.to_string(),"all_source_bits_restored":true}),
                )?;
            }
            report_output::seal(&root)?;
            report_output::verify(&root)?;
            results.push(attempt?);
            write(&a.out.join("direction-progress.json"), &results)?;
        }
    }
    if source_files(input)? != input_files || parameter_bits(&params)? != all_bits {
        return Err(invalid(
            "context direction immutable input/source restoration failed",
        ));
    }
    report_output::verify(input)?;
    if frontier {
        write(
            &a.out.join("frontier-summary.json"),
            &json!({"schema":"uor-r4.geometric-context-frontier-direction/1","frozen_frontier_sha256":sha256_file(&a.out.join("frozen-frontier.json"))?,"frontier_positions":frontiers,"baseline":baseline_frontier,"objective":"mean8 first actual ownprefix divergence token CE","candidate_frontiers":results.iter().map(|v|json!({"coordinate":v["coordinate"],"delta_q":v["delta_q"],"frontier":v["frontier"],"delta_mean_episode_ce":v["frontier_delta_mean_episode_ce"]})).collect::<Vec<_>>()}),
        )?;
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-context-direction/1","mode":a.mode,"status":"completed","source_commit":source_commit()?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"saved_identity":identity,"fit_source_commit":fit["source_commit"],"retained_report_sha256":retained_sha,"parent_checkpoint_manifest_sha256":sha256_file(&input.join("manifest.json"))?,"frozen_panel_sha256":sha256_file(&a.out.join("frozen-panel.json"))?,"selection_sha256":sha256_file(&a.out.join("selection.json"))?,"optimizer_updates":0,"selected":selected,"results":results,"baseline_original_canonical":baseline_original,"baseline_construction_canonical":baseline_construction,"baseline_original_generation":original_generation,"baseline_construction_generation":construction_generation,"fixed_singer_prefix3_baseline_trace":fixed_baseline,"saved_parent28_rows_exact":true,"input_files_unchanged":true,"input_files_sha256":input_files,"all_source_bits_restored":true,"no_adopted_model":true,"transfer_predictions_run":false,"scope":"max2 existing context coordinate both-sign quarter quantum diagnosis with frozen readouts on8 construction and20 preservation development cases; no fit or heldout/geometry-advantage/chat qualification","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}
fn direction(a: &Args) -> Result<()> {
    let start = Instant::now();
    let LoadedFinal {
        source,
        native,
        identity,
        tok,
        episodes,
        fit,
        retained_sha,
        before,
    } = load_final(a)?;
    let input = a
        .audit_checkpoint
        .as_ref()
        .ok_or_else(|| invalid("checkpoint missing"))?;
    let audit_path = a
        .audit_report
        .as_ref()
        .ok_or_else(|| invalid("direction needs saved audit report"))?;
    report_output::verify(
        audit_path
            .parent()
            .ok_or_else(|| invalid("audit envelope missing"))?,
    )?;
    let old: Value = serde_json::from_slice(&fs::read(audit_path)?)?;
    if old["schema"] != "uor-r4.geometric-source-realizer-audit/1"
        || old["status"] != "completed"
        || old["target_steps"] != 114
        || old["checkpoint_manifest_sha256"] != sha256_file(&input.join("manifest.json"))?
        || old["retained_report_sha256"] != retained_sha
    {
        return Err(invalid("saved final-prefix audit binding differs"));
    }
    let baseline = canonical_measure(&native, &episodes, start, a)?;
    if baseline["target_steps"] != 114 {
        return Err(invalid("direction requires exact114 canonical positions"));
    }
    let baseline_rows = baseline["rows"]
        .as_array()
        .ok_or_else(|| invalid("baseline rows absent"))?;
    let old_rows = old["canonical_rows"]
        .as_array()
        .ok_or_else(|| invalid("audit canonical rows absent"))?;
    if baseline_rows.len() != old_rows.len() {
        return Err(invalid("audit case count differs"));
    }
    for (actual, previous) in baseline_rows.iter().zip(old_rows) {
        if actual["id"] != previous["id"] {
            return Err(invalid("audit case identity differs"));
        }
        let x = actual["tokens"]
            .as_array()
            .ok_or_else(|| invalid("baseline tokens absent"))?;
        let y = previous["tokens"]
            .as_array()
            .ok_or_else(|| invalid("audit tokens absent"))?;
        if x.len() != y.len() {
            return Err(invalid("audit target count differs"));
        }
        for (x, y) in x.iter().zip(y) {
            if x["trace"] != y["trace"]
                || x["target_rank_probability"] != y["target_rank_probability"]
                || x["target"] != y["target_label_only"]
            {
                return Err(invalid(
                    "baseline does not reproduce saved audit native masses/ranks",
                ));
            }
        }
    }
    let generation = generate("direction-baseline", &native, &episodes, &tok, start, a)?;
    let replay = if let Some(p) = &a.expected_generation {
        report_output::verify(
            p.parent()
                .ok_or_else(|| invalid("expected envelope missing"))?,
        )?;
        let expected: Value = serde_json::from_slice(&fs::read(p)?)?;
        if expected["native_loaded_from_disk"] != true
            || expected["native_metadata_sha256"]
                != sha256_file(&input.join("realizer-native/metadata.json"))?
            || expected["rows"] != generation["rows"]
        {
            return Err(invalid("direction baseline own-prefix replay differs"));
        }
        Some(true)
    } else {
        None
    };
    write(
        &a.out.join("baseline.json"),
        &json!({"canonical":baseline,"ownprefix":generation,"saved_audit_exact":true,"saved_generation_exact":replay}),
    )?;
    let params = source.parameters();
    let mut shadows = BTreeMap::new();
    let mut gradients = BTreeMap::<String, Vec<f64>>::new();
    for (name, var) in params
        .iter()
        .filter(|(n, _)| n.starts_with("consumer.potential."))
    {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        gradients.insert(name.clone(), vec![0.; values.len()]);
        shadows.insert(name.clone(), values);
    }
    if shadows.len() != 7 {
        return Err(invalid("seven potential families required"));
    }
    // The prepared graph is dropped before any source Var is changed.
    {
        let prepared = source.prepare(&native)?;
        for e in &episodes {
            for (step, &target) in e.target.iter().enumerate() {
                deadline(start, a)?;
                let out = prepared.loss(e.frame(), &e.view, &e.query, &e.target[..step], target)?;
                let scaled = (&out.loss * (1. / (episodes.len() * e.target.len()) as f64))?;
                let store = scaled.backward()?;
                for (name, gradient) in &mut gradients {
                    let var = params
                        .get(name)
                        .ok_or_else(|| invalid("gradient Var absent"))?;
                    if let Some(g) = store.get(var.as_tensor()) {
                        for (dst, x) in gradient.iter_mut().zip(g.flatten_all()?.to_vec1::<f32>()?)
                        {
                            if !x.is_finite() {
                                return Err(invalid("nonfinite potential gradient"));
                            }
                            *dst += f64::from(x);
                        }
                    }
                }
            }
        }
    }
    let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
        "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
    ))
    .map_err(|e| invalid(e.to_string()))?;
    let mut eligible = BTreeMap::<(String, usize), Vec<Value>>::new();
    let mut eligibility_copy_failures = 0usize;
    for (e, row) in episodes.iter().zip(baseline_rows) {
        for token in row["tokens"]
            .as_array()
            .ok_or_else(|| invalid("token rows absent"))?
        {
            let winner = token["copy"]["best_copy_token_id"]
                .as_u64()
                .ok_or_else(|| invalid("winner absent"))? as u32;
            let target = token["target"]
                .as_u64()
                .ok_or_else(|| invalid("target absent"))? as u32;
            if winner == target {
                continue;
            }
            let ids = e.view.emitted_token_ids();
            let ts = ids
                .iter()
                .enumerate()
                .filter(|(_, id)| **id == target)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let ws = ids
                .iter()
                .enumerate()
                .filter(|(_, id)| **id == winner)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            if ts.is_empty() || ws.is_empty() {
                continue;
            }
            eligibility_copy_failures += 1;
            let step = token["step"]
                .as_u64()
                .ok_or_else(|| invalid("step absent"))? as usize;
            let typed = native.read(e.frame(), &e.view, &e.query, &e.target[..step])?;
            let features = scorer_features(&typed, &geometry)?;
            let lanes = typed.period_context.heads * typed.period_context.lanes_per_head;
            for (name, values) in &shadows {
                for index in 0..values.len() {
                    let mut distinguishes = false;
                    for &t in &ts {
                        for &w in &ws {
                            if fixed_feature(&features, name, index, t, lanes)?
                                != fixed_feature(&features, name, index, w, lanes)?
                            {
                                distinguishes = true;
                            }
                        }
                    }
                    if distinguishes {
                        eligible.entry((name.clone(),index)).or_default().push(json!({"id":e.id,"step":step,"target_occurrences":ts,"winning_copy_occurrences":ws}));
                    }
                }
            }
        }
    }
    let mut candidates = Vec::new();
    for ((name, index), witness) in eligible {
        let shadow = shadows[&name][index];
        let gradient = gradients[&name][index];
        if gradient != 0. {
            candidates.push(DirectionCoordinate {
                name,
                index,
                gradient,
                original_shadow: shadow,
                original_q: (shadow * 4.).round() as i8,
                calibration: false,
                eligibility: witness,
            });
        }
    }
    let mut selected = select_direction(candidates);
    let name = "consumer.potential.content_presence";
    let values = shadows
        .get(name)
        .ok_or_else(|| invalid("calibration family absent"))?;
    let calibration = values
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 4 == 0)
        .map(|(index, &shadow)| DirectionCoordinate {
            name: name.into(),
            index,
            gradient: gradients[name][index],
            original_shadow: shadow,
            original_q: (shadow * 4.).round() as i8,
            calibration: true,
            eligibility: Vec::new(),
        })
        .max_by(|a, b| {
            a.gradient
                .abs()
                .total_cmp(&b.gradient.abs())
                .then(b.index.cmp(&a.index))
        });
    if let Some(c) = calibration {
        selected.push(c);
    }
    let mut analytic = Vec::new();
    for c in &selected {
        let mut calculated = 0.;
        for (e, row) in episodes.iter().zip(baseline_rows) {
            for token in row["tokens"]
                .as_array()
                .ok_or_else(|| invalid("baseline tokens missing"))?
            {
                let step = token["step"]
                    .as_u64()
                    .ok_or_else(|| invalid("step absent"))? as usize;
                let trace = native.read(e.frame(), &e.view, &e.query, &e.target[..step])?;
                let features = scorer_features(&trace, &geometry)?;
                let logits = trace
                    .actions
                    .actions
                    .iter()
                    .map(|x| (x.score_q24 as f64 / 16_777_216.) as f32)
                    .collect::<Vec<_>>();
                let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let exp = logits.iter().map(|x| (*x - max).exp()).collect::<Vec<_>>();
                let z: f32 = exp.iter().sum();
                let pi = exp.iter().map(|x| *x / z).collect::<Vec<_>>();
                let target = e.target[step];
                let target_pi: f32 = trace
                    .actions
                    .actions
                    .iter()
                    .zip(&pi)
                    .filter(|(action, _)| action.token_id == target)
                    .map(|(_, p)| *p)
                    .sum();
                let p = token["target_rank_probability"]["target_probability_diagnostic"]
                    .as_f64()
                    .ok_or_else(|| invalid("native target probability absent"))?
                    as f32;
                let lanes = trace.period_context.heads * trace.period_context.lanes_per_head;
                for (j, action) in trace
                    .actions
                    .actions
                    .iter()
                    .enumerate()
                    .take(e.view.emitted_token_ids().len())
                {
                    let ds =
                        pi[j] * (target_pi - if action.token_id == target { 1. } else { 0. }) / p;
                    calculated += f64::from(ds)
                        * fixed_feature(&features, &c.name, c.index, j, lanes)?
                        / (episodes.len() * e.target.len()) as f64;
                }
            }
        }
        let tolerance = 1e-5 + 1e-3 * c.gradient.abs();
        analytic.push(json!({"coordinate":c,"analytic_gradient_nat":calculated,"extracted_gradient_nat":c.gradient,"absolute_difference":(calculated-c.gradient).abs(),"tolerance":tolerance,"matches":(calculated-c.gradient).abs()<=tolerance}));
    }
    write(
        &a.out.join("selection.json"),
        &json!({"selected":selected,"analytic_adjoint":analytic,"gradient_families":gradients,"selector":"top4 |equal-episode CEgradient| among target-vs-winningCopy distinguishing fixed features, deterministic name/index ties; plus one common Copy calibration","maximum_native_candidates":10,"eligibility_copy_failures":eligibility_copy_failures,"adjoint_scope":"biased hard-native-probability/F32-softmax STE; not the native discrete derivative","calibration_scope":"one common Copy-relative-margin control; Period/Stop fixed, joint CE may change"}),
    )?;
    let original_potential = fs::read(input.join("realizer-native/consumer/potential-q4.bin"))?;
    let families = uor_r4_integer::geometric_potential_q4::FAMILY_NAMES;
    let mut results = Vec::new();
    for c in &selected {
        for delta in [-1i8, 1] {
            let q = c.original_q + delta;
            if !(-7..=7).contains(&q) {
                results.push(json!({"coordinate":c,"delta_q":delta,"status":"saturated_skip"}));
                continue;
            }
            deadline(start, a)?;
            let var = params
                .get(&c.name)
                .ok_or_else(|| invalid("selected Var absent"))?;
            let original = shadows
                .get(&c.name)
                .ok_or_else(|| invalid("original shadow absent"))?;
            let mut values = original.clone();
            values[c.index] = f32::from(q) * 0.25;
            let root = a.out.join(format!("candidate-{:02}", results.len()));
            report_output::claim(&root)?;
            var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
            let attempt = (|| -> Result<Value> {
                let candidate = source.compile(identity.clone())?;
                candidate.save(&root.join("native"))?;
                let packed = bin_files(&root.join("native"))?;
                for (name, sha) in &before {
                    if name != "consumer/potential-q4.bin" && packed.get(name) != Some(sha) {
                        return Err(invalid("candidate changed frozen payload"));
                    }
                }
                if !before.keys().eq(packed.keys()) {
                    return Err(invalid("candidate payload inventory differs"));
                }
                let family = c
                    .name
                    .strip_prefix("consumer.potential.")
                    .ok_or_else(|| invalid("family prefix"))?;
                let mut offset = 0;
                for name in families {
                    if name == family {
                        break;
                    }
                    offset += shadows
                        .get(&format!("consumer.potential.{name}"))
                        .ok_or_else(|| invalid("packed family absent"))?
                        .len();
                }
                let quantum = quantum_change(
                    &original_potential,
                    &fs::read(root.join("native/consumer/potential-q4.bin"))?,
                    offset + c.index,
                    delta,
                )?;
                let measured = canonical_measure(&candidate, &episodes, start, a)?;
                let mut regressions = Vec::new();
                let mut differences = Vec::new();
                for (base, changed) in baseline_rows.iter().zip(
                    measured["rows"]
                        .as_array()
                        .ok_or_else(|| invalid("candidate rows absent"))?,
                ) {
                    for (b, n) in base["tokens"]
                        .as_array()
                        .ok_or_else(|| invalid("base tokens absent"))?
                        .iter()
                        .zip(
                            changed["tokens"]
                                .as_array()
                                .ok_or_else(|| invalid("candidate tokens absent"))?,
                        )
                    {
                        if b["trace"]["period_context"] != n["trace"]["period_context"] {
                            return Err(invalid("candidate changed frozen context replay"));
                        }
                        let ba = b["trace"]["actions"]["actions"]
                            .as_array()
                            .ok_or_else(|| invalid("baseline action rows absent"))?;
                        let na = n["trace"]["actions"]["actions"]
                            .as_array()
                            .ok_or_else(|| invalid("candidate action rows absent"))?;
                        if ba.len() != na.len() || ba.len() < 2 {
                            return Err(invalid("candidate action shape differs"));
                        }
                        let copy_n = ba.len() - 2;
                        let mut action_deltas = Vec::new();
                        for (i, (x, y)) in ba.iter().zip(na).enumerate() {
                            let delta = y["score_q24"]
                                .as_i64()
                                .ok_or_else(|| invalid("candidate rawscore absent"))?
                                - x["score_q24"]
                                    .as_i64()
                                    .ok_or_else(|| invalid("baseline rawscore absent"))?;
                            if i >= copy_n && delta != 0 {
                                return Err(invalid("candidate changed Period/Stop rawscore"));
                            }
                            action_deltas.push(delta);
                        }
                        if c.calibration && action_deltas[..copy_n].windows(2).any(|x| x[0] != x[1])
                        {
                            return Err(invalid(
                                "common Copy calibration changed pairwise Copy Q24 differences",
                            ));
                        }
                        let d = n["native_ce"]
                            .as_f64()
                            .ok_or_else(|| invalid("candidate CE absent"))?
                            - b["native_ce"]
                                .as_f64()
                                .ok_or_else(|| invalid("baseline CE absent"))?;
                        let entry = json!({"id":base["id"],"step":b["step"],"delta_ce":d,"baseline_margin_q31":b["target_mass_margin_q31"],"candidate_margin_q31":n["target_mass_margin_q31"],"baseline_copy":b["copy"],"candidate_copy":n["copy"],"action_raw_q24_deltas":action_deltas,"period_stop_raw_scores_unchanged":true,"context_replay_unchanged":true,"calibration_pairwise_copy_differences_unchanged":if c.calibration {Some(true)} else {None},"baseline_rank_probability":b["target_rank_probability"],"candidate_rank_probability":n["target_rank_probability"]});
                        if d > 0. {
                            regressions.push(entry.clone());
                        }
                        differences.push(entry);
                    }
                }
                let d = measured["mean_episode_ce"]
                    .as_f64()
                    .ok_or_else(|| invalid("candidate objective absent"))?
                    - baseline["mean_episode_ce"]
                        .as_f64()
                        .ok_or_else(|| invalid("baseline objective absent"))?;
                let result = json!({"coordinate":c,"delta_q":delta,"delta_nat":f64::from(delta)*0.25,"actual_shadow_delta":f64::from(q)*0.25-f64::from(c.original_shadow),"predicted_native_lattice_delta_ce":c.gradient*f64::from(delta)*0.25,"predicted_actual_shadow_delta_ce":c.gradient*(f64::from(q)*0.25-f64::from(c.original_shadow)),"actual_delta_mean_episode_ce":d,"quantum":quantum,"packed_files":packed,"all_target_differences":differences,"target_ce_regressions":regressions,"canonical":measured,"status":"measured","scope":"independent original-lattice perturbation; no adoption or own-prefix qualification"});
                write(&root.join("report.json"), &result)?;
                Ok(result)
            })();
            // Restoration executes before propagating candidate errors.
            var.set(&Tensor::from_vec(
                original.clone(),
                var.shape(),
                &Device::Cpu,
            )?)?;
            if var
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|x| x.to_bits())
                .ne(original.iter().map(|x| x.to_bits()))
            {
                return Err(invalid("source shadow restoration differs"));
            }
            if let Err(error) = &attempt {
                write(
                    &root.join("error.json"),
                    &json!({"error":error.to_string(),"coordinate":c,"delta_q":delta,"source_shadow_restored":true}),
                )?;
            }
            report_output::seal(&root)?;
            report_output::verify(&root)?;
            results.push(attempt?);
            write(
                &a.out.join("direction-progress.json"),
                &json!({"results":results,"source_shadow_restored":true,"optimizer_updates":0,"wall_seconds":start.elapsed().as_secs_f64()}),
            )?;
        }
    }
    if before != bin_files(&input.join("realizer-native"))? {
        return Err(invalid("direction altered input packed payloads"));
    }
    report_output::verify(input.parent().ok_or_else(|| invalid("fit root missing"))?)?;
    report_output::verify(
        audit_path
            .parent()
            .ok_or_else(|| invalid("audit envelope missing"))?,
    )?;
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("retained envelope missing"))?,
    )?;
    report_output::verify(&a.checkpoint)?;
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-q4-readout-direction/1","status":"completed","mode":"direction","source_commit":source_commit()?,"fit_source_commit":fit["source_commit"],"executable_sha256":sha256_file(&std::env::current_exe()?)?,"saved_identity":identity,"checkpoint_manifest_sha256":sha256_file(&input.join("manifest.json"))?,"audit_report_sha256":sha256_file(audit_path)?,"retained_report_sha256":retained_sha,"optimizer_updates":0,"maximum_selected_coordinates":4,"calibration_coordinates":1,"maximum_native_candidate_compiles":10,"eligibility_copy_failures":eligibility_copy_failures,"selected":selected,"analytic_adjoint":analytic,"baseline":baseline,"saved_generation_exact":replay,"results":results,"packed_input_unchanged":true,"source_shadows_restored":true,"no_adopted_model":true,"scope":"bounded local direction evidence on114 exposed canonical positions; no expressivity, credit-bug, geometry-advantage or chat qualification from a finite perturbation","wall_seconds":start.elapsed().as_secs_f64()}),
    )?;
    Ok(())
}

#[cfg(test)]
mod direction_tests {
    use super::*;
    #[test]
    fn frozen_geometry_updates_use_full_equal_episode_objective() -> Result<()> {
        let indices = (0..28).collect::<Vec<_>>();
        let mut visits = [0usize; 28];
        for _ in 0..UPDATES {
            assert!(readout_batch_admitted(
                "readout-geometry-coadapt",
                &indices,
                28
            ));
            for &index in &indices {
                visits[index] += 1;
            }
        }
        assert_eq!(visits, [64; 28]);
        for tokens in [1, 2, 7, 23] {
            assert!(
                (equal_episode_token_scale(28, tokens)? * tokens as f64 - 1. / 28.).abs() < 1e-15
            );
        }
        let mut duplicate = indices.clone();
        duplicate[27] = 26;
        assert!(!readout_batch_admitted(
            "readout-geometry-coadapt",
            &duplicate,
            28
        ));
        assert!(!readout_batch_admitted(
            "readout-geometry-coadapt",
            &indices[..8],
            28
        ));
        assert!(!readout_batch_admitted(
            "readout-geometry-coadapt",
            &indices,
            29
        ));
        assert!(readout_batch_admitted("readout-coadapt", &indices[..8], 28));
        assert!(equal_episode_token_scale(0, 1).is_err());
        assert!(equal_episode_token_scale(usize::MAX, 2).is_err());
        Ok(())
    }
    #[test]
    fn frozen_geometry_native_payloads_allow_only_existing_three_readouts() {
        let before = [
            "consumer/context-q4.bin",
            "consumer/exp-q31.bin",
            "consumer/potential-q4.bin",
            "consumer/no-read-q4.bin",
            "period-q4.bin",
            "h4.bin",
        ]
        .into_iter()
        .map(|name| (name.to_owned(), "before".to_owned()))
        .collect::<BTreeMap<_, _>>();
        for name in [
            "consumer/potential-q4.bin",
            "consumer/no-read-q4.bin",
            "period-q4.bin",
        ] {
            let mut after = before.clone();
            after.insert(name.into(), "after".into());
            assert!(readout_native_payloads_fixed(&before, &after));
        }
        for name in ["consumer/context-q4.bin", "consumer/exp-q31.bin", "h4.bin"] {
            let mut after = before.clone();
            after.insert(name.into(), "after".into());
            assert!(!readout_native_payloads_fixed(&before, &after));
        }
        let mut extra = before.clone();
        extra.insert("new.bin".into(), "after".into());
        assert!(!readout_native_payloads_fixed(&before, &extra));
    }
    #[test]
    fn frozen_geometry_source_bits_and_gradient_filter_preserve_context() -> Result<()> {
        let params = [
            (
                "consumer.context.token_root".to_owned(),
                candle_core::Var::from_vec(vec![0f32, -0f32], 2, &Device::Cpu)?,
            ),
            (
                "period.weights".to_owned(),
                candle_core::Var::from_vec(vec![1f32, 2f32], 2, &Device::Cpu)?,
            ),
        ];
        let parameter_map = params.iter().cloned().collect::<BTreeMap<_, _>>();
        let before = parameter_bits(&parameter_map)?;
        params[1]
            .1
            .set(&Tensor::from_vec(vec![3f32, 4f32], 2, &Device::Cpu)?)?;
        assert!(frozen_readout_source_bits_fixed(
            &before,
            &parameter_bits(&parameter_map)?
        ));
        params[0]
            .1
            .set(&Tensor::from_vec(vec![0f32, 0f32], 2, &Device::Cpu)?)?;
        assert!(!frozen_readout_source_bits_fixed(
            &before,
            &parameter_bits(&parameter_map)?
        ));
        let gradients = params
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        let filtered = readout_gradients(gradients)?;
        assert_eq!(
            filtered.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["period.weights"]
        );
        assert!(!frozen_readout_source_bits_fixed(
            &BTreeMap::new(),
            &BTreeMap::new()
        ));
        Ok(())
    }
    #[test]
    fn frozen_geometry_ranking_includes_baseline_and_ignores_nonfinite_ce() {
        assert_eq!(
            best_finite_readout_checkpoint(&[Some(0.4), Some(0.5), Some(0.4)]),
            Some(0)
        );
        assert_eq!(
            best_finite_readout_checkpoint(&[Some(0.4), Some(0.3), Some(0.3)]),
            Some(1)
        );
        assert_eq!(
            best_finite_readout_checkpoint(&[None, Some(f64::NAN), Some(0.7)]),
            Some(2)
        );
        assert_eq!(
            best_finite_readout_checkpoint(&[None, Some(f64::INFINITY)]),
            None
        );
    }
    #[test]
    fn frozen_geometry_native_ce_is_episode_weighted_and_requires_both_panels() -> Result<()> {
        let original = json!({"rows":vec![Value::Null;20],"mean_episode_ce":1.});
        let construction = json!({"rows":vec![Value::Null;8],"mean_episode_ce":3.});
        assert_eq!(full28_native_ce(&original, &construction)?, Some(44. / 28.));
        let mut bad = construction.clone();
        bad["rows"] = json!([]);
        assert!(full28_native_ce(&original, &bad).is_err());
        bad = construction;
        bad["mean_episode_ce"] = Value::Null;
        assert_eq!(full28_native_ce(&original, &bad)?, None);
        Ok(())
    }
    #[test]
    fn frozen_geometry_admission_rejects_other_parent_or_mutable_context() -> Result<()> {
        let identity = ConsumerIdentity {
            tokenizer_sha256: "t".into(),
            parent_checkpoint_manifest_sha256: "c".into(),
            parent_model_sha256: "m".into(),
            parent_config_sha256: "f".into(),
        };
        let mut admission = GeometryReadoutAdmission {
            source_commit: source_commit()?.to_owned(),
            geometry_run_source_commit: "geometry-source".into(),
            geometry_run_manifest_sha256: "manifest".into(),
            geometry_checkpoint_manifest_sha256: "checkpoint".into(),
            geometry_result_sha256: "result".into(),
            round: 2,
            candidate: 4,
            optimizer: optimizer_identity(),
            maximum_seconds: 1800,
            starting_role: "retained_main_candidate_new_research_parent".into(),
        };
        let report = json!({"schema":"uor-r4.geometric-later-query-cells/1","mode":"context-later-query-cells","status":"completed","source_commit":"geometry-source","saved_identity":identity,"retained_report_sha256":"retained","transitions_readouts_tables_fixed":true,"no_adopted_model":true});
        let receipt = json!({"optimizer_updates":0});
        let result = json!({"ablation_only":false,"readouts_tables_transitions_fixed":true,"all_other_source_bits_fixed":true,"checkpoint":receipt});
        assert!(geometry_parent_header(
            &admission, &report, &result, &receipt, &identity, "retained"
        )
        .is_ok());
        for field in ["transitions_readouts_tables_fixed", "no_adopted_model"] {
            let mut bad = report.clone();
            bad[field] = json!(false);
            assert!(geometry_parent_header(
                &admission, &bad, &result, &receipt, &identity, "retained"
            )
            .is_err());
        }
        let mut bad = result.clone();
        bad["all_other_source_bits_fixed"] = json!(false);
        assert!(
            geometry_parent_header(&admission, &report, &bad, &receipt, &identity, "retained")
                .is_err()
        );
        let mut bad = identity.clone();
        bad.parent_model_sha256 = "other".into();
        assert!(
            geometry_parent_header(&admission, &report, &result, &receipt, &bad, "retained")
                .is_err()
        );
        admission.starting_role = "old-default".into();
        assert!(geometry_parent_header(
            &admission, &report, &result, &receipt, &identity, "retained"
        )
        .is_err());
        Ok(())
    }
    fn query_test_context(tokens: Vec<u32>, lanes: usize) -> SerializableContextReplay {
        let time = tokens.len();
        SerializableContextReplay {
            tokens,
            heads: 1,
            lanes_per_head: lanes,
            states: vec![vec![1; lanes]; time],
            actions: vec![vec![1; lanes]; time],
            raw_roots: vec![1; time * lanes],
            categories: vec![1; time * lanes],
            codes: vec![
                ObservedCode {
                    root: 1,
                    radius_bin: 0,
                    present: true
                };
                time * lanes
            ],
            coefficient_reads: 0,
        }
    }
    #[test]
    fn query_diagnostic_shared_keys_require_complete_early_state_and_observation() -> Result<()> {
        let left = query_test_context(vec![3, 4, 5, 6], 2);
        let mut right = left.clone();
        right.tokens[3] = 9;
        right.states[3][0] = 4;
        assert!(query_shared_context_equal(&left, &right, 3).is_ok());
        for field in 0..6 {
            let mut bad = right.clone();
            match field {
                0 => bad.tokens[1] = 8,
                1 => bad.states[1][0] = 3,
                2 => bad.actions[1][0] = 3,
                3 => bad.raw_roots[2] = 3,
                4 => bad.categories[2] = 2,
                _ => bad.codes[2].root = 3,
            }
            assert!(query_shared_context_equal(&left, &bad, 3).is_err());
        }
        let mut missing = left.clone();
        missing.codes.clear();
        assert!(query_shared_context_equal(&missing, &missing, 3).is_err());
        assert!(query_shared_context_equal(&left, &right, 5).is_err());
        Ok(())
    }
    #[test]
    fn query_diagnostic_trajectory_offsets_and_empty_prefix_are_explicit() -> Result<()> {
        let e = frozen_transfer_test_episode()?;
        let source = e.view.emitted_token_ids().len();
        let mut tokens = e.view.emitted_token_ids().to_vec();
        tokens.extend_from_slice(&e.query);
        let replay = query_test_context(tokens, 2);
        let aligned = query_trajectory_context(&replay, &e)?;
        assert_eq!(aligned.len(), e.query.len());
        assert_eq!(aligned[0]["query_offset"], 0);
        assert_eq!(aligned[0]["absolute_context_time"], source);
        let mut extra = replay.clone();
        extra.tokens.push(9);
        assert!(query_trajectory_context(&extra, &e).is_err());
        let mut wrong = replay;
        wrong.tokens[source] = 99;
        assert!(query_trajectory_context(&wrong, &e).is_err());
        Ok(())
    }
    #[test]
    fn query_diagnostic_controller_parts_match_public_codec_across_all_head_lanes() -> Result<()> {
        use uor_r4_integer::geometric_no_read::{
            pack_coefficients, NativeGeometricNoRead, NoReadConfig,
        };
        use uor_r4_integer::geometric_potential::AddressLane;
        let config = NoReadConfig {
            vocabulary: 8,
            heads: 2,
            latent_lanes_per_head: 4,
        };
        let q = (0..config.coefficient_count())
            .map(|i| (i % 5) as i8 - 2)
            .collect::<Vec<_>>();
        let packed = pack_coefficients(&q).map_err(|e| invalid(e.to_string()))?;
        let table =
            NativeGeometricNoRead::new(config, &packed).map_err(|e| invalid(e.to_string()))?;
        let roots = (0..8).map(|i| i as u8 + 1).collect::<Vec<_>>();
        let latent = roots
            .iter()
            .map(|r| H4Code::try_from(*r).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let addresses = roots
            .iter()
            .enumerate()
            .map(|(i, r)| AddressLane::new(*r, i as u8, true).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let codes = addresses
            .iter()
            .copied()
            .map(ObservedCode::from)
            .collect::<Vec<_>>();
        let expected = table
            .score(3, &latent, &addresses, None)
            .map_err(|e| invalid(e.to_string()))?;
        let parts = query_controller_parts(&table, 3, &roots, &codes, &expected)?;
        assert_eq!(
            parts["heads"][0]["all_latent_lanes"]
                .as_array()
                .map(Vec::len),
            Some(8)
        );
        assert_eq!(
            parts["heads"][1]["all_latent_lanes"]
                .as_array()
                .map(Vec::len),
            Some(8)
        );
        assert_eq!(parts["heads"][0]["sum_q24"], expected[0]);
        assert_eq!(parts["heads"][1]["sum_q24"], expected[1]);
        let mut wrong = expected;
        wrong[0] += 1;
        assert!(query_controller_parts(&table, 3, &roots, &codes, &wrong).is_err());
        assert!(query_controller_parts(&table, 3, &roots[..4], &codes, &expected).is_err());
        Ok(())
    }
    #[test]
    fn query_diagnostic_binding_allows_portable_paths_but_not_scientific_substitution() -> Result<()>
    {
        let identity = ConsumerIdentity {
            tokenizer_sha256: "t".into(),
            parent_checkpoint_manifest_sha256: "c".into(),
            parent_model_sha256: "m".into(),
            parent_config_sha256: "f".into(),
        };
        let admission = FrozenTransferAdmission {
            source_commit: "new-source".into(),
            frozen_fit_root: "copied-fit".into(),
            frozen_fit_manifest_sha256: "fit-manifest".into(),
            frozen_fit_report_sha256: "fit-report".into(),
            frozen_fit_source_commit: "fit-source".into(),
            composition_root: "copied-composition".into(),
            composition_manifest_sha256: "composition-manifest".into(),
            composition_report_sha256: "composition-report".into(),
            composition_source_commit: "composition-source".into(),
            checkpoint_manifest_sha256: BTreeMap::from([(
                "selected0064".into(),
                "selected-manifest".into(),
            )]),
            panel_spec: "copied-panel".into(),
            panel_spec_sha256: "panel-sha".into(),
        };
        let mut old = serde_json::to_value(&admission)?;
        old["source_commit"] = json!("old-source");
        old["frozen_fit_root"] = json!("old-absolute-fit");
        let report = json!({"schema":"uor-r4.frozen-geometric-readout-transfer/1","mode":"readout-geometry-transfer","status":"completed","optimizer_updates":0,"all_inputs_unchanged":true,"model_count":3,"fresh_cases_per_model":8,"replayed_cases_per_model":28,"selected_fit_checkpoint_index_recomputed":4,"saved_identity":identity,"source_commit":"old-source","admission":old});
        assert!(query_transfer_binding(&report, &admission, &identity, "old-source").is_ok());
        for field in [
            "frozen_fit_manifest_sha256",
            "checkpoint_manifest_sha256",
            "panel_spec_sha256",
        ] {
            let mut wrong = report.clone();
            wrong["admission"][field] = json!("substituted");
            assert!(query_transfer_binding(&wrong, &admission, &identity, "old-source").is_err());
        }
        let mut wrong = report;
        wrong["all_inputs_unchanged"] = json!(false);
        assert!(query_transfer_binding(&wrong, &admission, &identity, "old-source").is_err());
        Ok(())
    }
    fn frozen_transfer_test_episode() -> Result<Episode> {
        let tokenizer = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"a":0,"b":1,"ab":2,"Ġ":3,"Ġa":4,"Ġab":5},"merges":["a b","Ġ ab"]}}"#;
        let compiler = SourceEmissionCompiler::new(tokenizer.as_bytes())?;
        Ok(Episode {
            id: "test".into(),
            query: vec![4],
            tokens: vec![0],
            record: 7,
            commit: 9,
            relation: 2,
            entity: vec![0],
            target: vec![5, 99],
            accepted: vec!["a.".into()],
            source_text: "a".into(),
            view: compiler.compile(&[0])?,
        })
    }
    #[test]
    fn frozen_transfer_source_packet_excludes_labels_and_expected_length() -> Result<()> {
        let episode = frozen_transfer_test_episode()?;
        let packet = transfer_source_packet(&episode, &[3]);
        let mut relabeled = episode.clone();
        relabeled.target = vec![99; 20];
        relabeled.accepted = vec!["other answer".into()];
        assert_eq!(packet, transfer_source_packet(&relabeled, &[3]));
        assert_ne!(packet, transfer_source_packet(&relabeled, &[5]));
        assert_eq!(packet["actual_prefix_ids"], json!([3]));
        assert!(packet.get("target").is_none());
        assert!(packet.get("accepted").is_none());
        Ok(())
    }
    #[test]
    fn frozen_transfer_pair_requires_same_query_and_typed_frame() -> Result<()> {
        let left = frozen_transfer_test_episode()?;
        let mut right = left.clone();
        right.source_text = "b".into();
        assert!(transfer_pair_inputs_fixed(&left, &right));
        for field in 0..5 {
            let mut bad = right.clone();
            match field {
                0 => bad.query.push(1),
                1 => bad.record += 1,
                2 => bad.commit += 1,
                3 => bad.relation += 1,
                _ => bad.entity.push(1),
            }
            assert!(!transfer_pair_inputs_fixed(&left, &bad));
        }
        assert!(!transfer_pair_inputs_fixed(&left, &left));
        Ok(())
    }
    #[test]
    fn frozen_transfer_coverage_distinguishes_query_only_ids_and_excludes_eos() -> Result<()> {
        let mut episode = frozen_transfer_test_episode()?;
        episode.query = vec![42, 99];
        episode.target = vec![43, 99];
        let (source, context) = transfer_training_id_sets(&[episode], 99);
        assert!(!source.contains(&42));
        assert!(context.contains(&42));
        assert!(context.contains(&43));
        assert!(!context.contains(&99));
        assert!(source.is_subset(&context));
        Ok(())
    }
    #[test]
    fn frozen_transfer_literals_and_spec_are_frozen_before_results() {
        assert!(fresh_literals_disjoint(
            &["singer".into()],
            &["Vexalnorp".into()]
        ));
        assert!(!fresh_literals_disjoint(&["Mervalon Quistara".into()], &[]));
        assert!(!fresh_literals_disjoint(&[], &["singer Louston".into()]));
        let spec = json!({"schema":"uor-r4.prospective-fresh-source-transfer/1","predictions":"NOT_RUN","optimizer_updates":0,"context_tokens":128,"response_tokens":64,"pairs":FRESH_TRANSFER_PAIRS.iter().map(|(id,left,right)|json!({"id":id,"left":left,"right":right})).collect::<Vec<_>>()});
        assert!(fresh_transfer_spec_valid(&spec));
        let mut changed = spec.clone();
        changed["pairs"][0]["left"] = json!("replacement-after-prediction");
        assert!(!fresh_transfer_spec_valid(&changed));
        changed = spec;
        changed["optimizer_updates"] = json!(1);
        assert!(!fresh_transfer_spec_valid(&changed));
    }
    #[test]
    fn frozen_transfer_loader_rejects_checkpoint_or_evaluation_substitution() {
        let receipt = json!({"optimizer_updates":64,"binding":{"protocol":"dialogue2"}});
        let stage = json!({"checkpoint":receipt,"optimizer_updates":64,"native_payload_sha256":{"context":"fixed"}});
        assert!(transfer_stage_receipt_bound(&stage, &stage, &receipt));
        let mut different = receipt.clone();
        different["optimizer_updates"] = json!(32);
        assert!(!transfer_stage_receipt_bound(&stage, &stage, &different));
        let mut different = stage.clone();
        different["native_payload_sha256"]["context"] = json!("changed");
        assert!(!transfer_stage_receipt_bound(&stage, &different, &receipt));
    }
    #[test]
    fn frozen_transfer_selection_recomputes_full28_with_baseline_ties() -> Result<()> {
        let stages=[0,16,32,48,64].into_iter().zip([0.5,0.4,0.3,0.2,0.1]).map(|(step,ce)|json!({"optimizer_updates":step,"context_source_bits_unchanged":true,"context_and_geometry_native_payloads_unchanged":true,"original_canonical":{"rows":vec![Value::Null;20],"mean_episode_ce":ce},"construction_canonical":{"rows":vec![Value::Null;8],"mean_episode_ce":ce},"all28_equal_episode_ce":(20.*ce+8.*ce)/28.})).collect::<Vec<_>>();
        let report = json!({"schema":"uor-r4.geometric-frozen-context-readout/1","mode":"readout-geometry-coadapt","status":"completed","optimizer_updates":64,"declared_updates":64,"context_source_bits_unchanged":true,"initial_actual28_replay_equal":true,"input_files_unchanged":true,"no_adopted_model":true,"selected_checkpoint_index":4,"checkpoints":stages});
        assert_eq!(frozen_transfer_fit_selection(&report)?, 4);
        let mut wrong = report.clone();
        wrong["selected_checkpoint_index"] = json!(3);
        assert!(frozen_transfer_fit_selection(&wrong).is_err());
        wrong = report.clone();
        wrong["checkpoints"][4]["all28_equal_episode_ce"] = json!(0.0);
        assert!(frozen_transfer_fit_selection(&wrong).is_err());
        wrong = report;
        wrong["checkpoints"][4] = wrong["checkpoints"][0].clone();
        wrong["checkpoints"][4]["optimizer_updates"] = json!(64);
        assert!(frozen_transfer_fit_selection(&wrong).is_err());
        Ok(())
    }
    fn later_test_single(index: usize, reserved: bool) -> CellProposal {
        CellProposal {
            edits: vec![consumed_test_edit("token_category", index, -1.)],
            predicted_gain: 1. / (index + 1) as f64,
            witness: if reserved {
                json!({"reserved_later_query":true,"actual_nonempty_prefix_ids":[363],"context_time":12,"family":"token_category"})
            } else {
                Value::Null
            },
        }
    }
    #[test]
    fn later_query_quota_preserves_low_global_rank_before_native_outcomes() -> Result<()> {
        let globals = (0..80)
            .map(|index| later_test_single(index, false))
            .collect::<Vec<_>>();
        let selected = later_query_cell_selection(&globals, &[later_test_single(77, true)], &[])?;
        assert_eq!(selected.len(), 4);
        assert!(selected.iter().any(|p| p.edits[0].coordinate.index == 77));
        assert!(!selected.iter().any(|p| p.edits[0].coordinate.index == 3));
        Ok(())
    }
    #[test]
    fn later_query_duplicate_keeps_reached_witness_and_fallback_does_not_refill() -> Result<()> {
        let globals = (0..5)
            .map(|index| later_test_single(index, false))
            .collect::<Vec<_>>();
        let selected = later_query_cell_selection(&globals, &[later_test_single(1, true)], &[])?;
        assert_eq!(selected.len(), 3);
        assert!(selected
            .iter()
            .find(|p| p.edits[0].coordinate.index == 1)
            .is_some_and(|p| p.witness["reserved_later_query"] == true));
        let fallback = later_query_cell_selection(&globals, &[], &[])?;
        assert_eq!(fallback.len(), 4);
        assert!(fallback.iter().any(|p| p.edits[0].coordinate.index == 3));
        let mut malformed = later_test_single(77, true);
        malformed.witness["actual_nonempty_prefix_ids"] = json!([]);
        assert!(later_query_cell_selection(&globals, &[malformed], &[]).is_err());
        Ok(())
    }
    #[test]
    fn consumed_continuation_rejects_wrong_ancestor_and_unaccepted_control_header() -> Result<()> {
        let identity: ConsumerIdentity = serde_json::from_value(
            json!({"tokenizer_sha256":"a".repeat(64),"parent_checkpoint_manifest_sha256":"b".repeat(64),"parent_model_sha256":"c".repeat(64),"parent_config_sha256":"d".repeat(64)}),
        )?;
        let receipt = json!({"optimizer_updates":0});
        let header = json!({"schema":"uor-r4.geometric-consumed-cells/1","mode":"context-consumed-cells","status":"completed","stop_reason":"accepted_candidate_limit","maximum_accepted_candidates":3,"accepted_candidates":3,"maximum_native_candidates":24,"native_candidate_evaluations":22,"optimizer_updates":0,"parent_checkpoint_manifest_sha256":"parent","saved_identity":identity,"retained_report_sha256":"retained","final_checkpoint":receipt,"final_independent_reload_full_replies_equal":true,"final_accepted_candidate_replies_equal":true,"transitions_readouts_tables_fixed":true,"no_adopted_model":true,"source_commit":"a".repeat(40)});
        consumed_parent_header(&header, &identity, "retained", "parent", &receipt)?;
        assert!(
            consumed_parent_header(&header, &identity, "retained", "wrong-parent", &receipt)
                .is_err()
        );
        for (key, value) in [
            ("accepted_candidates", json!(0)),
            ("mode", json!("context-observable-cells")),
            ("final_accepted_candidate_replies_equal", json!(false)),
            ("native_candidate_evaluations", json!(25)),
        ] {
            let mut bad = header.clone();
            bad[key] = value;
            assert!(
                consumed_parent_header(&bad, &identity, "retained", "parent", &receipt).is_err()
            );
        }
        Ok(())
    }
    #[test]
    fn consumed_continuation_freezes_basis_transitions_readouts_and_parameter_inventory() {
        let before = BTreeMap::from([
            ("consumer.context.token_root".to_owned(), vec![1u32]),
            ("consumer.context.self_root".to_owned(), vec![2]),
            ("consumer.context.token_transition".to_owned(), vec![3]),
            ("period.readout".to_owned(), vec![4]),
        ]);
        let mut allowed = before.clone();
        allowed.insert("consumer.context.token_root".to_owned(), vec![5]);
        assert!(only_token_observation_bits_changed(&before, &allowed));
        for name in [
            "consumer.context.self_root",
            "consumer.context.token_transition",
            "period.readout",
        ] {
            let mut bad = allowed.clone();
            bad.insert(name.to_owned(), vec![6]);
            assert!(!only_token_observation_bits_changed(&before, &bad));
        }
        let mut bad = allowed;
        bad.insert("invented".to_owned(), vec![0]);
        assert!(!only_token_observation_bits_changed(&before, &bad));
    }
    #[test]
    fn consumed_continuation_accepts_only_native_selected_main_winner() -> Result<()> {
        let entry = json!({"selected_proposal":1,"candidate_evaluations":2});
        let report = json!({"selected_proposal":1,"main_proposal_count":2,"ablation_count":0,"proposals":[{"round_frozen_ce":1.9},{"round_frozen_ce":1.8}],"baseline_frontier":{"mean_episode_ce":2.}});
        assert_eq!(consumed_accepted_round(&report, &entry)?, (1, 2));
        let mut bad = report.clone();
        bad["selected_proposal"] = json!(0);
        assert!(consumed_accepted_round(
            &bad,
            &json!({"selected_proposal":0,"candidate_evaluations":2})
        )
        .is_err());
        let mut bad = report.clone();
        bad["ablation_count"] = json!(1);
        assert!(consumed_accepted_round(&bad, &entry).is_err());
        let mut bad = report;
        bad["baseline_frontier"]["mean_episode_ce"] = json!(1.8);
        assert!(consumed_accepted_round(&bad, &entry).is_err());
        Ok(())
    }

    #[test]
    fn consumed_footprint_moves_final_query_after_generated_prefix() -> Result<()> {
        assert_eq!(
            consumed_context_roles(&[10, 11], &[20, 21], &[], &[10, 11, 20, 21])?,
            vec![
                Some("source_key"),
                Some("source_key"),
                None,
                Some("final_query_address")
            ]
        );
        assert_eq!(
            consumed_context_roles(&[10, 11], &[20, 21], &[30, 31], &[10, 11, 20, 21, 30, 31])?,
            vec![
                Some("source_key"),
                Some("source_key"),
                None,
                None,
                None,
                Some("final_query_address")
            ]
        );
        Ok(())
    }
    #[test]
    fn consumed_footprint_rejects_original_source_substitution_and_token_reordering() {
        assert!(consumed_context_roles(&[10, 11], &[20], &[], &[10, 12, 20]).is_err());
        assert!(consumed_context_roles(&[10, 11], &[20], &[30], &[10, 11, 30, 20]).is_err());
        assert!(consumed_context_roles(&[10], &[], &[30], &[10, 30]).is_err());
    }
    fn consumed_test_edit(family: &str, index: usize, gradient: f64) -> CellEdit {
        CellEdit {
            coordinate: DirectionCoordinate {
                name: format!("consumer.context.{family}"),
                index,
                gradient,
                original_shadow: 0.,
                original_q: 0,
                calibration: false,
                eligibility: vec![],
            },
            delta_q: 1,
        }
    }
    fn consumed_test_pair(index: usize, category_gradient: f64) -> CellProposal {
        CellProposal {
            edits: vec![
                consumed_test_edit("token_root", index, -4.),
                consumed_test_edit("token_category", index, category_gradient),
            ],
            predicted_gain: 1.,
            witness: json!({"event":index,"id":"x","context_time":3,"lane":0,"old_root":1,"old_category":0,"presence":{"new_winner":2,"class":2}}),
        }
    }
    #[test]
    fn consumed_selection_admits_matched_category_even_when_part_credit_adverse() -> Result<()> {
        let pairs = vec![
            consumed_test_pair(0, 1.),
            consumed_test_pair(1, -1.),
            consumed_test_pair(2, -10.),
        ];
        let selected = consumed_cell_selection(&[], &pairs)?;
        assert_eq!(selected.len(), 4);
        assert_eq!(selected.iter().filter(|p| p.edits.len() == 2).count(), 2);
        assert!(selected.iter().any(|p| p.edits.len() == 1
            && p.predicted_gain < 0.
            && p.witness["prospective_matched_category_part"] == true));
        assert!(selected.iter().all(|p| p.edits[0].coordinate.index != 2));
        Ok(())
    }
    #[test]
    fn consumed_selection_deduplicates_parts_without_adaptive_quota_refill() -> Result<()> {
        let pairs = vec![consumed_test_pair(0, -1.), consumed_test_pair(1, -1.)];
        let mut singles = (0..5)
            .map(|index| CellProposal {
                edits: vec![consumed_test_edit("token_category", index, -1.)],
                predicted_gain: 1.,
                witness: Value::Null,
            })
            .collect::<Vec<_>>();
        // Both matched parts already occupy prospective visible-single slots.
        let selected = consumed_cell_selection(&singles, &pairs)?;
        assert_eq!(selected.len(), 6);
        assert!(!selected
            .iter()
            .any(|p| p.edits.len() == 1 && p.edits[0].coordinate.index == 4));
        singles.reverse();
        assert!(consumed_cell_selection(&singles, &pairs)?.len() <= 8);
        Ok(())
    }
    #[test]
    fn consumed_round_uses_only_accepted_checkpoint_and_preserves_other_mode_limits() -> Result<()>
    {
        assert_eq!(observation_learning_limits(true, true), (3, 24));
        assert_eq!(observation_learning_limits(false, true), (1, 10));
        assert_eq!(observation_learning_limits(false, false), (8, 32));
        let input = Path::new("/retained/final-checkpoint");
        let out = Path::new("/claimed/run");
        assert_eq!(observation_incumbent_checkpoint(input, out, 0, &[])?, input);
        let accepted = vec![json!({"selected_proposal":6})];
        assert_eq!(
            observation_incumbent_checkpoint(input, out, 1, &accepted)?,
            out.join("round-00/candidate-06/checkpoint")
        );
        assert!(observation_incumbent_checkpoint(
            input,
            out,
            1,
            &[json!({"selected_proposal":null})]
        )
        .is_err());
        assert!(observation_incumbent_checkpoint(input, out, 2, &accepted).is_err());
        Ok(())
    }

    #[test]
    fn observable_cell_bounded_reservoir_is_exact_deduplicated_rank() {
        let make = |index: usize, gain: f64, delta: i8, witness: usize| CellProposal {
            edits: vec![CellEdit {
                coordinate: DirectionCoordinate {
                    name: "consumer.context.token_root".to_owned(),
                    index,
                    gradient: -1.,
                    original_shadow: 0.,
                    original_q: 0,
                    calibration: false,
                    eligibility: vec![],
                },
                delta_q: delta,
            }],
            witness: json!({"event":witness}),
            predicted_gain: gain,
        };
        let incoming = vec![
            make(0, 1., 1, 4),
            make(1, 3., 2, 3),
            make(2, 2., 1, 2),
            make(3, 1., 1, 1),
            make(4, 3., 1, 0),
            make(0, 1., 1, 0),
        ];
        let mut bounded = Vec::new();
        for proposal in &incoming {
            retain_cell(&mut bounded, proposal.clone());
        }
        let mut full = incoming;
        full.sort_by(cell_order);
        let mut seen = BTreeSet::new();
        full.retain(|p| seen.insert(cell_key(p)));
        full.truncate(4);
        assert_eq!(
            serde_json::to_value(bounded).ok(),
            serde_json::to_value(full).ok()
        );
    }
    #[test]
    fn observable_cell_first_index_ties_and_all_rivals() -> Result<()> {
        let unit = 1i64 << 22;
        assert_eq!(cell_winner(&[unit, unit, unit])?, 0);
        assert_eq!(cell_crossing(&[0, unit, 2 * unit], 0, 0, 1)?, Some((2, 0)));
        assert_eq!(cell_crossing(&[2 * unit, unit, 0], 2, 0, 1)?, Some((3, 2)));
        Ok(())
    }
    #[test]
    fn observable_cell_demotion_and_legal_saturation() -> Result<()> {
        let unit = 1i64 << 22;
        assert_eq!(cell_crossing(&[unit, 0, 0], 0, 0, -1)?, Some((-2, 1)));
        assert_eq!(cell_crossing(&[0, 100 * unit], 0, 0, 1)?, None);
        assert_eq!(cell_crossing(&[0, unit], 0, 7, 1)?, None);
        assert_eq!(cell_crossing(&[unit, 0], 0, -7, -1)?, None);
        assert!(cell_crossing(&[], 0, 0, 1).is_err());
        Ok(())
    }
    #[test]
    fn observable_cell_quantum_minimum_matches_independent_legal_search() -> Result<()> {
        let unit = 1i64 << 22;
        for scores in [
            vec![0, 0, 0],
            vec![unit, 2 * unit + 1, -unit],
            vec![-unit, unit, unit],
        ] {
            for class in 0..3 {
                for q in -7..=7i8 {
                    for direction in [-1, 1i8] {
                        let old = cell_winner(&scores)?;
                        let expected = (-7..=7i8)
                            .filter(|candidate| (*candidate - q).signum() == direction)
                            .map(|candidate| candidate - q)
                            .collect::<Vec<_>>();
                        let mut expected = expected;
                        expected.sort_by_key(|delta| delta.abs());
                        let brute = expected.into_iter().find_map(|delta| {
                            let mut changed = scores.clone();
                            changed[class] += i64::from(delta) * unit;
                            let winner = changed
                                .iter()
                                .enumerate()
                                .max_by(|(i, a), (j, b)| a.cmp(b).then(j.cmp(i)))
                                .map(|(i, _)| i)?;
                            (winner != old).then_some((delta, winner))
                        });
                        assert_eq!(cell_crossing(&scores, class, q, direction)?, brute);
                    }
                }
            }
        }
        Ok(())
    }
    #[test]
    fn observable_cell_exact_two_family_nibbles_and_reserved_minus8() -> Result<()> {
        let c = |name: &str, index: usize, q: i8| DirectionCoordinate {
            name: format!("consumer.context.{name}"),
            index,
            gradient: -1.,
            original_shadow: f32::from(q) * 0.25,
            original_q: q,
            calibration: false,
            eligibility: vec![],
        };
        let context = BTreeMap::from([
            ("consumer.context.token_root".to_owned(), vec![0., 0.]),
            ("consumer.context.token_category".to_owned(), vec![0., 0.]),
        ]);
        let edits = vec![
            CellEdit {
                coordinate: c("token_root", 1, 0),
                delta_q: 2,
            },
            CellEdit {
                coordinate: c("token_category", 0, 0),
                delta_q: 3,
            },
        ];
        assert!(verify_cell_packed_edits(&[0, 0], &[0x20, 0x03], &context, &edits).is_ok());
        assert!(verify_cell_packed_edits(&[0, 0], &[0x21, 0x03], &context, &edits).is_err());
        let bad = vec![CellEdit {
            coordinate: c("token_root", 0, 0),
            delta_q: -8,
        }];
        assert!(verify_cell_packed_edits(&[0, 0], &[8, 0], &context, &bad).is_err());
        Ok(())
    }

    #[test]
    fn shared_prefix_requires_common_emitted_id_and_later_divergence() {
        assert!(shared_prefix_diverges(&[3, 7, 9], &[3, 7, 10]));
        assert!(!shared_prefix_diverges(&[3, 7], &[3, 7]));
        assert!(!shared_prefix_diverges(&[3, 7], &[3, 7, 9]));
        assert!(!shared_prefix_diverges(&[3, 7], &[4, 7]));
        assert!(!shared_prefix_diverges(&[], &[3]));
    }
    #[test]
    fn transfer_comparison_preserves_row_identity_and_losses() -> Result<()> {
        let row = json!({"id":"x","original_source_ids":[3],"query_ids":[4],"accepted_complete_answer":true,"eos":true,"generated_ids":[3]});
        let mut after = row.clone();
        after["accepted_complete_answer"] = json!(false);
        let compared = generation_comparison(
            &json!({"rows":[row.clone()]}),
            &json!({"rows":[after.clone()]}),
        )?;
        assert_eq!(compared[0]["loss"], true);
        assert_eq!(compared[0]["gain"], false);
        after["original_source_ids"] = json!([8]);
        assert!(generation_comparison(&json!({"rows":[row]}), &json!({"rows":[after]})).is_err());
        Ok(())
    }
    #[test]
    fn completed_anchor_uses_actual_alias_and_rejects_false_trace() -> Result<()> {
        // The accepted alias may use IDs unlike the canonical target. This
        // validator preserves its actual trajectory; caller checks text membership.
        let row = json!({"id":"alias","accepted_complete_answer":true,"eos":true,
            "generated_ids":[8,9],"tokens":[{"step":0,"own_prefix_ids":[],"chosen_token_id":8},
            {"step":1,"own_prefix_ids":[8],"chosen_token_id":9},
            {"step":2,"own_prefix_ids":[8,9],"chosen_token_id":1}]});
        assert_eq!(completed_eos_prefix(&row, "alias", 1)?, vec![8, 9]);
        let mut bad = row.clone();
        bad["tokens"][2]["own_prefix_ids"] = json!([3, 4]);
        assert!(completed_eos_prefix(&bad, "alias", 1).is_err());
        let mut bad = row.clone();
        bad["tokens"][2]["step"] = json!(7);
        assert!(completed_eos_prefix(&bad, "alias", 1).is_err());
        let mut bad = row.clone();
        bad["tokens"][2]["chosen_token_id"] = json!(7);
        assert!(completed_eos_prefix(&bad, "alias", 1).is_err());
        let mut bad = row.clone();
        bad["accepted_complete_answer"] = json!(false);
        assert!(completed_eos_prefix(&bad, "alias", 1).is_err());
        let mut bad = row;
        bad["tokens"] = json!([]);
        assert!(completed_eos_prefix(&bad, "alias", 1).is_err());
        Ok(())
    }
    #[test]
    fn native_screen_selects_best_strict_decrease_with_stable_ties() {
        assert_eq!(
            best_native_proposal(2., &[Some(1.9), Some(1.8), Some(1.8), None]),
            Some(1)
        );
        assert_eq!(
            best_native_proposal(2., &[Some(2.), Some(2.1), Some(f64::NAN), None]),
            None
        );
        assert_eq!(
            best_native_proposal(2., &[Some(f64::NEG_INFINITY), Some(1.9)]),
            Some(1)
        );
    }
    #[test]
    fn frontier_rejects_absence_and_invalid_saved_prefix() -> Result<()> {
        assert!(saved_failure_prefix_ids(&json!({"id":"x","tokens":[]}), "x", &[3, 4]).is_err());
        let row = json!({"id":"x","accepted_complete_answer":false,"tokens":[{"chosen_token_id":3,"own_prefix_ids":[]},{"chosen_token_id":5,"own_prefix_ids":[3]}]});
        assert_eq!(
            saved_failure_prefix_ids(&row, "x", &[3, 4])?,
            (1, vec![3], 4)
        );
        let mut bad = row;
        bad["tokens"][1]["own_prefix_ids"] = json!([7]);
        assert!(saved_failure_prefix_ids(&bad, "x", &[3, 4]).is_err());
        Ok(())
    }
    #[test]
    fn frontier_selection_uses_one_root_and_one_category() {
        let make = |name: &str, g: f64| DirectionCoordinate {
            name: name.into(),
            index: 0,
            gradient: g,
            original_shadow: 0.,
            original_q: 0,
            calibration: false,
            eligibility: Vec::new(),
        };
        let result = select_frontier_direction(vec![
            make("consumer.context.token_transition", 100.),
            make("consumer.context.self_root", 2.),
            make("consumer.context.token_root", 1.),
            make("consumer.context.token_category", -3.),
        ]);
        assert_eq!(
            result.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec![
                "consumer.context.self_root",
                "consumer.context.token_category"
            ]
        );
    }
    #[test]
    fn context_direction_selector_respects_classes_ties_and_two_legal_signs() {
        let make = |name: &str, index: usize, gradient: f64, q: i8| DirectionCoordinate {
            name: name.into(),
            index,
            gradient,
            original_shadow: f32::from(q) * 0.25,
            original_q: q,
            calibration: false,
            eligibility: Vec::new(),
        };
        let selected = select_context_direction(vec![
            make("consumer.context.token_transition", 0, 5., 7),
            make("consumer.context.self_transition", 1, -2., 0),
            make("consumer.context.self_transition", 0, 2., 0),
            make("consumer.context.token_root", 0, 3., 0),
            make("consumer.context.token_category", 0, -3., 0),
            make("consumer.context.self_root", 0, 0., 0),
            make("consumer.potential.context_unary", 0, 100., 0),
        ]);
        assert_eq!(selected.len(), 2);
        assert_eq!(
            (&selected[0].name, selected[0].index),
            (&"consumer.context.self_transition".to_string(), 0)
        );
        assert_eq!(selected[1].name, "consumer.context.token_category");
        assert!(selected.iter().all(|c| [-1, 1]
            .iter()
            .all(|delta| (-7..=7).contains(&(c.original_q + delta)))));
    }
    #[test]
    fn context_direction_trajectory_distinguishes_plateau_from_first_change() -> Result<()> {
        let before = json!({"tokens":[3,7],"heads":1,"lanes_per_head":1,"states":[[1],[2]],"actions":[[1],[2]],"raw_roots":[1,2],"categories":[1,1],"codes":[{"root":1},{"root":2}]});
        assert_eq!(
            context_trajectory_difference(&before, &before)?["trajectory_unchanged"],
            true
        );
        let mut after = before.clone();
        after["states"][1] = json!([4]);
        let diff = context_trajectory_difference(&before, &after)?;
        assert_eq!(diff["first_changed_sequence_position"], 1);
        assert_eq!(diff["first_changed_input_token"], 7);
        assert_eq!(diff["changed_entries"]["states"], 1);
        after["tokens"][0] = json!(5);
        assert!(context_trajectory_difference(&before, &after).is_err());
        Ok(())
    }
    #[test]
    fn transplant_selects_exact_families_without_mutating_donor() -> Result<()> {
        for copy_context in [false, true] {
            let destination = BTreeMap::from([
                (
                    "consumer.context.transition".into(),
                    candle_core::Var::from_vec(vec![-0f32], 1, &Device::Cpu)?,
                ),
                (
                    "consumer.potential.content_unary".into(),
                    candle_core::Var::from_vec(vec![1f32], 1, &Device::Cpu)?,
                ),
                (
                    "consumer.no_read.bias".into(),
                    candle_core::Var::from_vec(vec![2f32], 1, &Device::Cpu)?,
                ),
                (
                    "period.bias".into(),
                    candle_core::Var::from_vec(vec![3f32], 1, &Device::Cpu)?,
                ),
            ]);
            let donor = BTreeMap::from([
                (
                    "consumer.context.transition".into(),
                    candle_core::Var::from_vec(vec![0f32], 1, &Device::Cpu)?,
                ),
                (
                    "consumer.potential.content_unary".into(),
                    candle_core::Var::from_vec(vec![4f32], 1, &Device::Cpu)?,
                ),
                (
                    "consumer.no_read.bias".into(),
                    candle_core::Var::from_vec(vec![5f32], 1, &Device::Cpu)?,
                ),
                (
                    "period.bias".into(),
                    candle_core::Var::from_vec(vec![6f32], 1, &Device::Cpu)?,
                ),
            ]);
            let before = parameter_bits(&destination)?;
            let donor_before = parameter_bits(&donor)?;
            transplant_family(&destination, &donor, copy_context)?;
            let after = parameter_bits(&destination)?;
            for (name, bits) in &after {
                assert_eq!(
                    bits,
                    if name.starts_with("consumer.context.") == copy_context {
                        &donor_before[name]
                    } else {
                        &before[name]
                    }
                );
            }
            assert_eq!(parameter_bits(&donor)?, donor_before);
        }
        let unknown = BTreeMap::from([(
            "unknown.parameter".into(),
            candle_core::Var::from_vec(vec![1f32], 1, &Device::Cpu)?,
        )]);
        assert!(transplant_family(&unknown, &unknown, true).is_err());
        Ok(())
    }
    #[test]
    fn context_adaptation_retains_context_gradients() -> Result<()> {
        let mut gradients = BTreeMap::new();
        gradients.insert(
            "consumer.context.transition".into(),
            Tensor::new(&[2f32], &Device::Cpu)?,
        );
        gradients.insert(
            "consumer.potential.context_unary".into(),
            Tensor::new(&[1f32], &Device::Cpu)?,
        );
        let gradients = context_gradients(gradients)?;
        assert_eq!(gradients.len(), 2);
        assert!(gradients.contains_key("consumer.context.transition"));
        let mut unknown = BTreeMap::new();
        unknown.insert(
            "unknown.parameter".into(),
            Tensor::new(&[1f32], &Device::Cpu)?,
        );
        assert!(context_gradients(unknown).is_err());
        Ok(())
    }
    #[test]
    fn context_bit_changes_preserves_exact_float_bits() -> Result<()> {
        let before = BTreeMap::from([(
            "consumer.context.transition".into(),
            vec![0f32.to_bits(), 1f32.to_bits()],
        )]);
        let after = BTreeMap::from([(
            "consumer.context.transition".into(),
            vec![(-0f32).to_bits(), 1f32.to_bits()],
        )]);
        assert_eq!(
            context_bit_changes(&before, &after)?.get("consumer.context.transition"),
            Some(&1)
        );
        assert!(context_bit_changes(&before, &BTreeMap::new()).is_err());
        Ok(())
    }
    #[test]
    fn frozen_context_gradient_is_removed_before_clip() -> Result<()> {
        let mut gradients = BTreeMap::new();
        gradients.insert(
            "consumer.context.transition".into(),
            Tensor::new(&[1000f32], &Device::Cpu)?,
        );
        gradients.insert(
            "consumer.potential.context_unary".into(),
            Tensor::new(&[0.5f32], &Device::Cpu)?,
        );
        gradients.insert(
            "consumer.no_read.bias".into(),
            Tensor::new(&[0.25f32], &Device::Cpu)?,
        );
        gradients.insert("period.bias".into(), Tensor::new(&[0.25f32], &Device::Cpu)?);
        let selected = readout_gradients(gradients)?;
        assert_eq!(selected.len(), 3);
        let mut norm_square = 0.;
        for (name, tensor) in selected {
            assert!(readout_parameter(&name));
            for v in tensor.flatten_all()?.to_vec1::<f32>()? {
                norm_square += f64::from(v).powi(2);
            }
        }
        assert_eq!(norm_square, 0.375);
        assert!(!readout_parameter("consumer.context.transition"));
        let mut unexpected = BTreeMap::new();
        unexpected.insert(
            "unexpected.bias".into(),
            Tensor::new(&[1f32], &Device::Cpu)?,
        );
        assert!(readout_gradients(unexpected).is_err());
        Ok(())
    }
    #[test]
    fn exact_one_legal_quantum_is_required() -> Result<()> {
        quantum_change(&[0x00], &[0x01], 0, 1)?;
        assert!(quantum_change(&[0x00], &[0x11], 0, 1).is_err());
        assert!(quantum_change(&[0x00], &[0x02], 0, 1).is_err());
        assert!(quantum_change(&[0x07], &[0x08], 0, 1).is_err());
        Ok(())
    }
    #[test]
    fn selection_ties_are_name_then_index() {
        let make = |name: &str, index: usize| DirectionCoordinate {
            name: name.into(),
            index,
            gradient: 1.,
            original_shadow: 0.,
            original_q: 0,
            calibration: false,
            eligibility: Vec::new(),
        };
        let selected = select_direction(vec![
            make("z", 0),
            make("a", 2),
            make("a", 1),
            make("b", 0),
            make("c", 0),
        ]);
        assert_eq!(
            selected
                .iter()
                .map(|x| (x.name.as_str(), x.index))
                .collect::<Vec<_>>(),
            vec![("a", 1), ("a", 2), ("b", 0), ("c", 0)]
        );
    }
    #[test]
    fn quarter_candidate_shadow_restores_exactly() -> Result<()> {
        let original = vec![0.13f32, -0.47];
        let var = candle_core::Var::from_vec(original.clone(), 2, &Device::Cpu)?;
        let mut changed = original.clone();
        changed[0] = 0.5;
        var.set(&Tensor::from_vec(changed, 2, &Device::Cpu)?)?;
        var.set(&Tensor::from_vec(original.clone(), 2, &Device::Cpu)?)?;
        assert_eq!(var.flatten_all()?.to_vec1::<f32>()?, original);
        Ok(())
    }
}
