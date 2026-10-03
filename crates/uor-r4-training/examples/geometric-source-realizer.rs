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
use uor_r4_integer::h4_tables::{H4Code, HistoricalH4Tables};
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, RealizerTrace, SourceRealizerWeights},
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
    if !matches!(a.mode.as_str(), "construction" | "fit" | "audit")
        || a.maximum_seconds == 0
        || (a.mode == "construction" && a.maximum_seconds > 300)
        || (a.mode == "fit" && a.maximum_seconds > 1200)
        || (a.mode == "audit"
            && (a.maximum_seconds > 300
                || a.audit_checkpoint.is_none()
                || a.fit_admission.is_some()))
        || (a.mode != "audit" && (a.audit_checkpoint.is_some() || a.expected_generation.is_some()))
    {
        return Err(invalid(
            "construction/audit wall limit 1..300; fit limit 1..1200 with fixed 64 updates",
        ));
    }
    let admitted = admission(&a)?;
    if a.mode == "audit" {
        audit_output_location(&a)?;
    }
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = if a.mode == "audit" {
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
fn audit(a: &Args) -> Result<()> {
    let start = Instant::now();
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
