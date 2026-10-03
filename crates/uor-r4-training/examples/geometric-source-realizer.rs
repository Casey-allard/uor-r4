//! Native selected-source Copy/Period/Stop realizer on exposed development cases.
//! Parent checkpoint is loaded for identity only; no parent scoring or responses.
//! This is a selected-record component, not a complete language-model qualification.
use candle_core::{Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
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
    if indices.len() != 8 {
        return Err(invalid("complete B8 required"));
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
            let scaled = (&out.loss * (1f64 / (8. * e.target.len() as f64)))?;
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
            trace_rows.push(json!({"step":step,"target_label_only":target,"teacherforced_prefix_ids":&e.target[..step],"nll":loss,"native_target_probability":probability,"native_loss_equal":true,"trace":out.trace}));
        }
        let episode_mean = sum / e.target.len() as f64;
        mean += episode_mean / 8.;
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
        report: json!({"episode_indices":indices,"episodes":8,"tokens":tokens,"objective":"mean8episodes(mean CE token+EOS)","mean_episode_nll":mean,"gradient_global_norm":square.sqrt(),"gradient_family_l1":families,"gradient_parameters":gradient_rows,"elapsed_seconds":begun.elapsed().as_secs_f64(),"rows":rows}),
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
            traces.push(
                json!({"step":step,"own_prefix_ids":prefix,"chosen_token_id":chosen,"trace":trace}),
            );
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
        complete += usize::from(accepted);
        rows.push(json!({"id":e.id,"source_record":e.record,"source_commit":e.commit,"original_source_ids":e.tokens,"source_text":e.source_text,"source_view":e.view,"query_ids":e.query,"generated_ids":prefix,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(tok.decode_bytes(&prefix)),"raw_decoded_text_lossy":decoded,"raw_utf8_valid":String::from_utf8(tok.decode_bytes(&prefix)).is_ok(),"text_policy":"strip only protocol2 single leading content separator","eos":ended,"stop":if ended{"eos"}else{"max64"},"accepted_complete_answer":accepted,"source_mention_diagnostic_only":text.contains(&e.source_text),"tokens":traces}));
        write(
            &a.out.join(format!("{stage}-progress.json")),
            &json!({"completed_cases":rows.len(),"rows":rows,"elapsed_seconds":begun.elapsed().as_secs_f64()}),
        )?;
    }
    Ok(
        json!({"scope":"20 exposed development own-prefix native selected-source generation; no parent scores; not heldout or completechat","cases":rows.len(),"complete_answers":complete,"eos_count":eos_count,"elapsed_seconds":begun.elapsed().as_secs_f64(),"rows":rows}),
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
    if !matches!(a.mode.as_str(), "construction" | "fit")
        || a.maximum_seconds == 0
        || (a.mode == "construction" && a.maximum_seconds > 300)
        || (a.mode == "fit" && a.maximum_seconds > 1200)
    {
        return Err(invalid(
            "construction wall limit1..300; fit limit1..1200 with fixed64updates",
        ));
    }
    let admitted = admission(&a)?;
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = run(&a, admitted.as_ref());
    if let Err(e) = &result {
        write(&a.out.join("error.json"), &json!({"error":e.to_string()}))?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
