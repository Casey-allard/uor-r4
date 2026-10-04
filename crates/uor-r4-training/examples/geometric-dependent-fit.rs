//! Bounded bridge-only Rust fit, independently reloaded native selection and fresh transfer.
//! Labels enter the existing offline final loss only; stage1 occurrence selection is detached.
use candle_core::{Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_core::report_output;
use uor_r4_integer::{
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_read_feedback::{FeedbackInputMode, NativeReadFeedback},
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer},
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{FeedbackBridgeWeights, NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_source_emission_view::SourceEmissionCompiler,
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT_CAP: usize = 256 * 1024 * 1024;
const UPDATES: usize = 64;
const STAGES: [usize; 5] = [0, 16, 32, 48, 64];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    out: PathBuf,
    checkpoint: PathBuf,
    trusted_binding: PathBuf,
    compiled_panel: PathBuf,
    parent_evaluation: PathBuf,
    baseline_probe: PathBuf,
    fixed_value: PathBuf,
    feedback_input: String,
    maximum_seconds: u64,
    admission_report: PathBuf,
    transfer_panel_root: PathBuf,
    transfer_panel_manifest_sha256: String,
}
#[derive(Deserialize)]
struct Panel {
    training64: Vec<Episode>,
}
#[derive(Deserialize)]
struct Episode {
    id: String,
    record: u64,
    commit: u64,
    entity: Vec<u32>,
    relation: u32,
    original_source_ids: Vec<u32>,
    query_ids: Vec<u32>,
    target_ids_labels_only: Vec<u32>,
    source_view: Value,
    accepted: Vec<String>,
    literal: String,
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
            token_ids: &self.original_source_ids,
        }
    }
}
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write_json(root: &Path, name: &str, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let destination = root.join(name);
    let existing = fs::metadata(&destination).map_or(0, |m| m.len() as usize);
    let current = tree_bytes(root)?;
    if current.saturating_sub(existing).saturating_add(bytes.len()) > REPORT_CAP - 1024 * 1024 {
        return Err(invalid("report byte cap reached").into());
    }
    fs::write(destination, bytes)?;
    Ok(())
}
fn checked_args() -> Result<Args> {
    let mut argv = std::env::args().skip(1);
    let config = argv
        .next()
        .ok_or_else(|| invalid("one JSON config path required"))?;
    if argv.next().is_some() {
        return Err(invalid("only one JSON config path accepted").into());
    }
    let a: Args = serde_json::from_slice(&fs::read(config)?)?;
    if !["joint-copy", "role-surface"].contains(&a.feedback_input.as_str())
        || a.maximum_seconds == 0
        || a.maximum_seconds > 900
        || a.transfer_panel_manifest_sha256.len() != 64
        || !a
            .transfer_panel_manifest_sha256
            .bytes()
            .all(|x| x.is_ascii_hexdigit())
    {
        return Err(invalid("admission mode/wall cap differs").into());
    }
    let output = output_support::prospective_output(&a.out)?;
    for input in [
        &a.checkpoint,
        &a.trusted_binding,
        &a.compiled_panel,
        &a.parent_evaluation,
        &a.baseline_probe,
        &a.fixed_value,
        &a.admission_report,
        &a.transfer_panel_root,
    ] {
        let canonical = fs::canonicalize(input)?;
        if output.starts_with(&canonical) {
            return Err(invalid("output beneath input").into());
        }
        for ancestor in canonical.ancestors() {
            if ancestor.is_dir()
                && ancestor.join(report_output::MANIFEST_FILE).exists()
                && output.starts_with(ancestor)
            {
                return Err(invalid("output beneath sealed input ancestor").into());
            }
        }
    }
    Ok(a)
}
fn sealed_parent(path: &Path) -> Result<()> {
    report_output::verify(
        path.parent()
            .ok_or_else(|| invalid("sealed parent absent"))?,
    )?;
    Ok(())
}
fn deadline(start: Instant, a: &Args) -> Result<()> {
    if peak_rss_kib().is_some_and(|rss| rss > 4 * 1024 * 1024) {
        return Err(invalid("fit4GiB RSS cap exceeded").into());
    }
    if start.elapsed().as_secs() >= a.maximum_seconds {
        return Err(invalid("declared admission wall limit reached").into());
    }
    Ok(())
}
fn parameter_receipts(parameters: &BTreeMap<String, Var>) -> Result<Value> {
    let mut receipts = BTreeMap::new();
    for (name, var) in parameters {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        let bytes = values
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect::<Vec<_>>();
        receipts.insert(name, json!({"shape":var.dims(),"elements":values.len(),"f32_le_bits_sha256":sha256_bytes(&bytes)}));
    }
    Ok(serde_json::to_value(receipts)?)
}
fn peak_rss_kib() -> Option<u64> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|s| {
            s.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}
fn executable() -> Result<(PathBuf, &'static str)> {
    match std::env::current_exe() {
        Ok(path) => Ok((path, "current_exe")),
        Err(error) => {
            let path = std::env::args().next().map(PathBuf::from).ok_or(error)?;
            if !path.is_absolute() {
                return Err(invalid("current_exe unavailable; argv0 must be absolute").into());
            }
            Ok((path, "absolute_argv0_fallback"))
        }
    }
}
fn tree_bytes(root: &Path) -> Result<usize> {
    let mut sum = 0usize;
    for e in fs::read_dir(root)? {
        let e = e?;
        let t = e.file_type()?;
        if t.is_symlink() {
            return Err(invalid("report symlink").into());
        }
        sum = sum
            .checked_add(if t.is_dir() {
                tree_bytes(&e.path())?
            } else {
                e.metadata()?.len() as usize
            })
            .ok_or_else(|| invalid("report size overflow"))?;
    }
    Ok(sum)
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    id: String,
    record: u64,
    commit: u64,
    scope: String,
    entity: Vec<u32>,
    relation: u32,
    view: u32,
    status: String,
    original_source_ids: Vec<u32>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
impl SourceInput {
    fn frame(&self) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: self.record,
                commit: self.commit,
            },
            metadata: FrameMetadata {
                scope: self.scope.as_bytes(),
                entity: &self.entity,
                relation: self.relation,
                view: self.view,
                status: FrameStatus::Found,
            },
            token_ids: &self.original_source_ids,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferInputs {
    schema: String,
    cases: Vec<SourceInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferLabels {
    schema: String,
    cases: Vec<TransferLabel>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferLabel {
    id: String,
    stratum: String,
    pair_id: String,
    side: String,
    literal: String,
    query: String,
    answers: FrozenAnswers,
    target_ids_labels_only: Vec<u32>,
    source_view: Value,
    token_support: Value,
}
struct Cases {
    inputs: Vec<SourceInput>,
    targets: Vec<Vec<u32>>,
    accepted: Vec<Vec<String>>,
    views: Vec<uor_r4_training::geometric_source_emission_view::SourceEmissionView>,
}
fn mode(a: &Args) -> FeedbackInputMode {
    if a.feedback_input == "joint-copy" {
        FeedbackInputMode::JointCopy
    } else {
        FeedbackInputMode::RoleSurface
    }
}
fn training_cases(
    panel: &Panel,
    compiler: &SourceEmissionCompiler,
    native: &NativeSourceRealizer,
) -> Result<Cases> {
    let mut inputs = Vec::new();
    let mut targets = Vec::new();
    let mut accepted = Vec::new();
    let mut views = Vec::new();
    let mut seen = BTreeSet::new();
    if panel.training64.len() != 64 {
        return Err(invalid("full64 training required").into());
    }
    for e in &panel.training64 {
        let view = compiler.compile(&e.original_source_ids)?;
        if !seen.insert(&e.id)
            || e.id.is_empty()
            || e.target_ids_labels_only.is_empty()
            || e.target_ids_labels_only.len() > 64
            || e.target_ids_labels_only.last() != Some(&native.binding().eos_token_id())
            || e.target_ids_labels_only[..e.target_ids_labels_only.len() - 1]
                .contains(&native.binding().eos_token_id())
            || e.query_ids.is_empty()
            || e.accepted.is_empty()
            || view.emitted_token_ids().len() + e.query_ids.len() + 64 > 128
            || serde_json::to_value(&view)? != e.source_view
        {
            return Err(invalid("development source/labels/window admission differs").into());
        }
        inputs.push(SourceInput {
            id: e.id.clone(),
            record: e.record,
            commit: e.commit,
            scope: "m-world-v2".into(),
            entity: e.entity.clone(),
            relation: e.relation,
            view: 0,
            status: "Found".into(),
            original_source_ids: e.original_source_ids.clone(),
            query_ids: e.query_ids.clone(),
            actual_prefix_ids: vec![],
        });
        targets.push(e.target_ids_labels_only.clone());
        accepted.push(e.accepted.clone());
        views.push(view);
    }
    Ok(Cases {
        inputs,
        targets,
        accepted,
        views,
    })
}
fn transfer_cases(
    a: &Args,
    compiler: &SourceEmissionCompiler,
    tokenizer: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
) -> Result<(Cases, Value)> {
    report_output::verify(&a.transfer_panel_root)?;
    if sha256_file(&a.transfer_panel_root.join(report_output::MANIFEST_FILE))?
        != a.transfer_panel_manifest_sha256
    {
        return Err(invalid("prospective transfer manifest differs").into());
    }
    let report = read_json(&a.transfer_panel_root.join("report.json"))?;
    if report["schema"] != "uor-r4.geometric-dependent-fit-panel/1"
        || report["status"] != "completed"
        || report["predictions"] != "NOT_RUN"
        || report["optimizer_updates"] != 0
        || report["cases"] != 32
        || report["source_literal_disjoint_training64_old32_old8_old16"] != true
        || report["full_source_query_pairs_disjoint"] != true
        || report["same_frame_within_every_pair"] != true
        || report["checkpoint_manifest_sha256"]
            != sha256_file(&a.checkpoint.join(report_output::MANIFEST_FILE))?
        || report["trusted_binding_sha256"] != sha256_file(&a.trusted_binding)?
        || report["training_and_old32_panel_sha256"] != sha256_file(&a.compiled_panel)?
        || report["source_inputs_sha256"]
            != sha256_file(&a.transfer_panel_root.join("source-inputs.json"))?
        || report["labels_sha256"] != sha256_file(&a.transfer_panel_root.join("labels.json"))?
    {
        return Err(invalid("prospective preparation report binding differs").into());
    }
    let inputs: TransferInputs =
        serde_json::from_slice(&fs::read(a.transfer_panel_root.join("source-inputs.json"))?)?;
    let labels: TransferLabels =
        serde_json::from_slice(&fs::read(a.transfer_panel_root.join("labels.json"))?)?;
    if inputs.schema != "uor-r4.geometric-dependent-fit-source-inputs/1"
        || labels.schema != "uor-r4.geometric-dependent-fit-labels/1"
        || inputs.cases.len() != 32
        || labels.cases.len() != 32
    {
        return Err(invalid("frozen transfer32 schemas/count differ").into());
    }
    let mut seen = BTreeSet::new();
    let mut literals = BTreeSet::new();
    let mut targets = Vec::new();
    let mut accepted = Vec::new();
    let mut views = Vec::new();
    for (s, l) in inputs.cases.iter().zip(&labels.cases) {
        l.answers.validate()?;
        let view = compiler.compile(&s.original_source_ids)?;
        if s.id != l.id
            || !seen.insert(&s.id)
            || !literals.insert(&l.literal)
            || s.scope != "m-world-v2"
            || s.status != "Found"
            || s.view != 0
            || !s.actual_prefix_ids.is_empty()
            || s.query_ids.is_empty()
            || s.original_source_ids.is_empty()
            || tokenizer.decode(&s.original_source_ids) != l.literal
            || tokenizer.encode(&l.query) != s.query_ids
            || serde_json::to_value(&view)? != l.source_view
            || view.emitted_token_ids().len() + s.query_ids.len() + 64 > 128
            || l.target_ids_labels_only.is_empty()
            || l.target_ids_labels_only.len() > 64
            || l.target_ids_labels_only.last() != Some(&native.binding().eos_token_id())
        {
            return Err(
                invalid("transfer literal/query/frame/view/label admission differs").into(),
            );
        }
        targets.push(l.target_ids_labels_only.clone());
        accepted.push(l.answers.accepted.clone());
        views.push(view);
    }
    let diagnostics=serde_json::to_value(labels.cases.iter().map(|l|json!({"id":l.id,"stratum":l.stratum,"pair_id":l.pair_id,"side":l.side,"literal":l.literal,"query":l.query,"token_support":l.token_support})).collect::<Vec<_>>())?;
    Ok((
        Cases {
            inputs: inputs.cases,
            targets,
            accepted,
            views,
        },
        diagnostics,
    ))
}
fn quantum(bridge: &FeedbackBridgeWeights, previous: &[u8]) -> Result<Value> {
    let params = bridge.parameters();
    let var = params
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("bridge parameter absent"))?;
    let values = var.flatten_all()?.to_vec1::<f32>()?;
    let packed = bridge.packed_coefficients()?;
    let mut changed = 0usize;
    let mut nonzero = 0usize;
    for (i, &x) in values.iter().enumerate() {
        let q = (x * 4.).round() as i8;
        nonzero += usize::from(q != 0);
        let byte = previous[i / 2];
        let nib = if i % 2 == 0 { byte & 15 } else { byte >> 4 };
        let prior = if nib >= 8 { nib as i8 - 16 } else { nib as i8 };
        changed += usize::from(q != prior);
    }
    Ok(
        json!({"independent_coefficients":values.len(),"shadow_max_absolute":values.iter().map(|x|x.abs()).fold(0f32,f32::max),"packed_nonzero_coefficients":nonzero,"packed_changed_coefficients_from_previous":changed,"packed_sha256":sha256_bytes(&packed),"shadow_bits":parameter_receipts(&params)?}),
    )
}
fn native_ce(
    actions: &uor_r4_integer::geometric_source_actions::ActionTrace,
    target: u32,
) -> Result<f64> {
    let mass = actions
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .map_or(0, |m| m.weight_q31);
    if mass == 0 || actions.total_weight_q31 == 0 {
        return Err(invalid("native target mass0; no floor").into());
    }
    Ok(-(mass as f64 / actions.total_weight_q31 as f64).ln())
}
struct Batch {
    gradient: Vec<f64>,
    canonical: Value,
    actions: Vec<Vec<u8>>,
}
fn save_failed_state(
    a: &Args,
    bridge: &FeedbackBridgeWeights,
    native: &NativeReadFeedback,
) -> Result<()> {
    let root = a.out.join("failed-current-state");
    report_output::claim(&root)?;
    let vars = bridge.parameters();
    let var = vars
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("failed bridge parameter absent"))?;
    let values = var.flatten_all()?.to_vec1::<f32>()?;
    fs::write(
        root.join("bridge-shadow-f32le.bin"),
        values
            .iter()
            .flat_map(|x| x.to_bits().to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    native.save(&root.join("feedback-native"))?;
    write_json(
        &root,
        "receipt.json",
        &json!({"parent_binding":bridge.parent_binding(),"bridge_shadow_shape":var.dims(),"bridge_shadow_sha256":sha256_file(&root.join("bridge-shadow-f32le.bin"))?,"feedback_metadata_sha256":sha256_file(&root.join("feedback-native/metadata.json"))?,"fixed_producer_sha256":sha256_bytes(bridge.value_packed()),"scope":"current unique error candidate preserved; no adoption"}),
    )?;
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    Ok(())
}
fn batch(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    native: &NativeSourceRealizer,
    bridge: &FeedbackBridgeWeights,
    compiled: &NativeReadFeedback,
    cases: &Cases,
    backward: bool,
    previous: &[Vec<u8>],
) -> Result<Batch> {
    let begun = Instant::now();
    let prepared = source.prepare(native)?;
    let parameters = bridge.parameters();
    if parameters.len() != 1 {
        return Err(invalid("bridge-only parameter set differs").into());
    }
    let var = parameters
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("bridge absent"))?;
    let mut gradient = vec![0f64; var.elem_count()];
    let mut rows = Vec::new();
    let mut mean = 0f64;
    let mut actions = Vec::new();
    let mut changed = 0usize;
    for (row, s) in cases.inputs.iter().enumerate() {
        deadline(start, a)?;
        let mut losses = Vec::new();
        let mut tokens = Vec::new();
        let mut sum = 0f64;
        for (step, &target) in cases.targets[row].iter().enumerate() {
            deadline(start, a)?;
            let prefix = &cases.targets[row][..step];
            let (loss, trace) = if backward {
                let measured = prepared.loss_dependent_with_native(
                    s.frame(),
                    &cases.views[row],
                    &s.query_ids,
                    prefix,
                    target,
                    bridge,
                    compiled,
                    mode(a),
                );
                let x = match measured {
                    Ok(x) => x,
                    Err(error) => {
                        save_failed_state(a, bridge, compiled)?;
                        let replay = native.read_dependent(
                            s.frame(),
                            &cases.views[row],
                            &s.query_ids,
                            prefix,
                            compiled,
                            mode(a),
                        );
                        let trace = match &replay {
                            Ok(t) => serde_json::to_value(t)?,
                            Err(e) => json!({"native_error":e.to_string()}),
                        };
                        write_json(
                            &a.out,
                            "failed-canonical-step.json",
                            &json!({"id":s.id,"step":step,"source_ids":s.original_source_ids,"query_ids":s.query_ids,"canonical_prefix_ids":prefix,"target_label_only":target,"loss_error":error.to_string(),"error_only_native_reread":true,"native_trace":trace,"completed_rows":rows,"completed_tokens_in_row":tokens,"feedback_metadata":compiled.metadata()}),
                        )?;
                        return Err(error.into());
                    }
                };
                (Some(x.loss), x.trace)
            } else {
                (
                    None,
                    native.read_dependent(
                        s.frame(),
                        &cases.views[row],
                        &s.query_ids,
                        prefix,
                        compiled,
                        mode(a),
                    )?,
                )
            };
            if compiled.bridge_packed().iter().all(|x| *x == 0)
                && (trace.stage1 != trace.stage2 || trace.feedback.before != trace.feedback.after)
            {
                return Err(invalid("zero bridge native identity differs").into());
            }
            let ce = match native_ce(&trace.stage2.actions, target) {
                Ok(ce) => ce,
                Err(error) => {
                    save_failed_state(a, bridge, compiled)?;
                    write_json(
                        &a.out,
                        "failed-canonical-step.json",
                        &json!({"id":s.id,"step":step,"source_ids":s.original_source_ids,"query_ids":s.query_ids,"canonical_prefix_ids":prefix,"target_label_only":target,"error":error.to_string(),"native_trace":trace,"completed_rows":rows,"completed_tokens_in_row":tokens,"feedback_metadata":compiled.metadata()}),
                    )?;
                    return Err(error);
                }
            };
            sum += ce;
            if let Some(loss) = loss {
                losses.push(loss);
            }
            if previous
                .get(actions.len())
                .is_some_and(|old| old != &trace.feedback.actions)
            {
                changed += 1;
            }
            actions.push(trace.feedback.actions.clone());
            tokens.push(json!({"step":step,"teacherforced_prefix_ids":prefix,"target_label_only":target,"native_ce":ce,"actions":trace.stage2.actions,"feedback_actions":trace.feedback.actions,"selected_occurrence":trace.feedback.selected_occurrence,"producer_packets":trace.feedback.value.packets,"refined_states":trace.stage2_controller_snapshot.states,"query_codes":trace.stage2_controller_snapshot.codes}));
        }
        if backward {
            let loss = Tensor::stack(&losses, 0)?.mean_all()?;
            let grads = loss.backward()?;
            let g = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid("disconnected bridge gradient"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if g.len() != gradient.len() || g.iter().any(|x| !x.is_finite()) {
                return Err(invalid("bridge gradient shape/finite differs").into());
            }
            accumulate_episode(&mut gradient, &g, 64)?;
        }
        let ce = sum / cases.targets[row].len() as f64;
        mean += ce / 64.;
        rows.push(json!({"id":s.id,"native_mean_token_ce":ce,"tokens":tokens}));
    }
    Ok(Batch {
        gradient,
        actions,
        canonical: json!({"cases":64,"mean_equal_episode_ce":mean,"rows":rows,"backward":backward,"feedback_action_positions_changed_from_previous":changed,"elapsed_seconds":begun.elapsed().as_secs_f64()}),
    })
}
fn accumulate_episode(sum: &mut [f64], gradient: &[f32], episodes: usize) -> Result<()> {
    if episodes == 0 || sum.len() != gradient.len() || gradient.iter().any(|x| !x.is_finite()) {
        return Err(invalid("episode gradient admission differs").into());
    }
    for (s, g) in sum.iter_mut().zip(gradient) {
        *s += f64::from(*g) / episodes as f64;
    }
    Ok(())
}
fn apply(bridge: &FeedbackBridgeWeights, optimizer: &mut AdamW, g: &[f64]) -> Result<Value> {
    let norm = g.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !norm.is_finite() {
        return Err(invalid("nonfinite bridge global gradient norm").into());
    }
    let factor = if norm > 1. { 1. / norm } else { 1. };
    let params = bridge.parameters();
    let var = params
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("bridge absent"))?;
    let v = g.iter().map(|x| (x * factor) as f32).collect::<Vec<_>>();
    if v.iter().any(|x| !x.is_finite()) {
        return Err(invalid("F32 optimizer gradient overflow").into());
    }
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    store.insert(
        var.as_tensor(),
        Tensor::from_vec(v, var.shape(), &Device::Cpu)?,
    );
    optimizer.step(&store)?;
    bridge.project_shadow_range()?;
    Ok(
        json!({"global_norm":norm,"clip_factor":factor,"finite":true,"nonzero":g.iter().filter(|x|**x!=0.).count(),"elements":g.len()}),
    )
}
fn generation(
    a: &Args,
    start: Instant,
    native: &NativeSourceRealizer,
    feedback: Option<&NativeReadFeedback>,
    cases: &Cases,
    tokenizer: &ByteBpeTokenizer,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0usize;
    let mut eos = 0usize;
    for (row, s) in cases.inputs.iter().enumerate() {
        let mut ids = Vec::new();
        let mut tokens = Vec::new();
        let mut ended = false;
        for step in 0..64 {
            deadline(start, a)?;
            if cases.views[row].emitted_token_ids().len() + s.query_ids.len() + ids.len() > 128 {
                return Err(invalid("own-prefix window exceeds128").into());
            }
            let (actions, feedback_receipt) = if let Some(f) = feedback {
                let r = native.read_dependent(
                    s.frame(),
                    &cases.views[row],
                    &s.query_ids,
                    &ids,
                    f,
                    mode(a),
                )?;
                (
                    r.stage2.actions,
                    Some(
                        json!({"actions":r.feedback.actions,"selected_occurrence":r.feedback.selected_occurrence,"value_packets":r.feedback.value.packets,"updated_states":r.stage2_controller_snapshot.states}),
                    ),
                )
            } else {
                (
                    native
                        .read(s.frame(), &cases.views[row], &s.query_ids, &ids)?
                        .actions,
                    None,
                )
            };
            let chosen = actions.chosen_token_id;
            tokens.push(json!({"step":step,"actual_prefix_ids":ids,"actions":actions,"feedback":feedback_receipt}));
            ids.push(chosen);
            if chosen == native.binding().eos_token_id() {
                ended = true;
                break;
            }
        }
        // Frozen targets/forms enter only after all native reads for this row.
        let content = if ended {
            &ids[..ids.len() - 1]
        } else {
            &ids[..]
        };
        let text = native
            .binding()
            .protocol()
            .reply_text(&tokenizer.decode(content))
            .to_owned();
        let accepted = ended && cases.accepted[row].iter().any(|x| x == &text);
        complete += usize::from(accepted);
        eos += usize::from(ended);
        let first = (0..ids.len().max(cases.targets[row].len()))
            .find(|&i| ids.get(i) != cases.targets[row].get(i));
        let diagnostic=first.map(|i| {
            let target=cases.targets[row].get(i).copied();let action=tokens.get(i).map(|t|&t["actions"]);
            let mass=action.and_then(|a|a["token_masses"].as_array()).and_then(|a|a.iter().find(|m|m["token_id"].as_u64()==target.map(u64::from))).and_then(|m|m["weight_q31"].as_u64()).unwrap_or(0);
            let total=action.and_then(|a|a["total_weight_q31"].as_u64());
            json!({"step":i,"target_label_only":target,"actual_token_id":ids.get(i),"target_mass_q31":mass,"total_weight_q31":total,"native_actions":action,"scope":"post-read label diagnostic, no target supplied to serving"})
        });
        rows.push(json!({"id":s.id,"record":s.record,"commit":s.commit,"original_source_ids":s.original_source_ids,"query_ids":s.query_ids,"generated_ids_including_eos":ids,"reply_text":text,"eos":ended,"accepted_complete_answer":accepted,"first_failure_diagnostics":diagnostic,"tokens":tokens}));
    }
    Ok(
        json!({"cases":cases.inputs.len(),"accepted_complete":complete,"eos":eos,"maximum_generated_tokens":64,"rows":rows,"canonical_prefixes_used":false}),
    )
}
fn compare(previous: &Value, current: &Value) -> Result<Value> {
    let left = previous["rows"]
        .as_array()
        .ok_or_else(|| invalid("comparison rows absent"))?;
    let right = current["rows"]
        .as_array()
        .ok_or_else(|| invalid("comparison rows absent"))?;
    if left.len() != right.len() {
        return Err(invalid("comparison count differs").into());
    }
    let mut rows = Vec::new();
    for (l, r) in left.iter().zip(right) {
        if l["id"] != r["id"] {
            return Err(invalid("comparison row identity differs").into());
        }
        rows.push(json!({"id":l["id"],"previous_complete":l["accepted_complete_answer"],"current_complete":r["accepted_complete_answer"],"previous_native_ce":l["native_mean_token_ce"],"current_native_ce":r["native_mean_token_ce"],"output_ids_equal":if l.get("generated_ids_including_eos").is_some()&&r.get("generated_ids_including_eos").is_some(){Some(l["generated_ids_including_eos"]==r["generated_ids_including_eos"])}else{None},"previous_reply":l["reply_text"],"current_reply":r["reply_text"]}));
    }
    Ok(json!({"rows":rows}))
}
fn selected(stages: &[Value]) -> Result<usize> {
    let mut best = None;
    for (i, s) in stages.iter().enumerate() {
        let ce = s["native_equal_episode_ce"]
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| invalid("checkpoint native CE nonfinite"))?;
        if best.is_none_or(|j: usize| {
            ce < stages[j]["native_equal_episode_ce"]
                .as_f64()
                .unwrap_or(f64::INFINITY)
        }) {
            best = Some(i);
        }
    }
    best.ok_or_else(|| invalid("no baseline-inclusive checkpoint").into())
}
fn export(
    a: &Args,
    start: Instant,
    source: &SourceRealizerWeights,
    native: &NativeSourceRealizer,
    bridge: &FeedbackBridgeWeights,
    cases: &Cases,
    tokenizer: &ByteBpeTokenizer,
    step: usize,
    prior: &[Value],
) -> Result<Value> {
    deadline(start, a)?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        let params = bridge.parameters();
        let var = params
            .get("bridge.coefficients")
            .ok_or_else(|| invalid("bridge absent"))?;
        let shadow = var.flatten_all()?.to_vec1::<f32>()?;
        let artifact_bytes = shadow.len() * 4
            + bridge.value_packed().len()
            + bridge.packed_coefficients()?.len()
            + 64 * 1024;
        if tree_bytes(&a.out)?.saturating_add(artifact_bytes) > REPORT_CAP - 1024 * 1024 {
            return Err(invalid("checkpoint artifact exceeds report cap").into());
        }
        fs::write(
            root.join("bridge-shadow-f32le.bin"),
            shadow
                .iter()
                .flat_map(|x| x.to_bits().to_le_bytes())
                .collect::<Vec<_>>(),
        )?;
        let f = bridge.compile()?;
        let artifact = root.join("feedback-native");
        f.save(&artifact)?;
        let metadata_sha = sha256_file(&artifact.join("metadata.json"))?;
        let reload = NativeReadFeedback::load(&artifact, &metadata_sha)?;
        if reload.metadata() != f.metadata()
            || reload.value_packed() != bridge.value_packed()
            || reload.bridge_packed() != bridge.packed_coefficients()?
        {
            return Err(invalid("feedback independent export reload differs").into());
        }
        let recreated = FeedbackBridgeWeights::from_native(&reload)?;
        let vars = recreated.parameters();
        let v = vars
            .get("bridge.coefficients")
            .ok_or_else(|| invalid("reloaded bridge absent"))?;
        let raw = fs::read(root.join("bridge-shadow-f32le.bin"))?;
        if raw.len() != v.elem_count() * 4 {
            return Err(invalid("fractional shadow size differs").into());
        }
        let floats = raw
            .chunks_exact(4)
            .map(|x| f32::from_bits(u32::from_le_bytes([x[0], x[1], x[2], x[3]])))
            .collect::<Vec<_>>();
        v.set(&Tensor::from_vec(floats, v.shape(), &Device::Cpu)?)?;
        if recreated.packed_coefficients()? != bridge.packed_coefficients()?
            || parameter_receipts(&recreated.parameters())? != parameter_receipts(&params)?
        {
            return Err(invalid("fractional shadow reload differs").into());
        }
        let canonical = batch(
            a,
            start,
            source,
            native,
            &recreated,
            &reload,
            cases,
            false,
            &[],
        )?
        .canonical;
        let generate = generation(a, start, native, Some(&reload), cases, tokenizer)?;
        write_json(
            &a.out,
            &format!("checkpoint-{step:04}/canonical.json"),
            &canonical,
        )?;
        write_json(
            &a.out,
            &format!("checkpoint-{step:04}/generation.json"),
            &generate,
        )?;
        let mut comparisons = Vec::new();
        for old in prior {
            let oldstep = old["optimizer_updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior step absent"))?;
            let oldroot = a.out.join(format!("checkpoint-{oldstep:04}"));
            comparisons.push(json!({"prior_updates":oldstep,"canonical":compare(&read_json(&oldroot.join("canonical.json"))?,&canonical)?,"generation":compare(&read_json(&oldroot.join("generation.json"))?,&generate)?}));
        }
        write_json(
            &a.out,
            &format!("checkpoint-{step:04}/comparisons.json"),
            &json!(comparisons),
        )?;
        let summary = json!({"optimizer_updates":step,"native_equal_episode_ce":canonical["mean_equal_episode_ce"],"accepted_complete":generate["accepted_complete"],"eos":generate["eos"],"feedback_metadata_sha256":metadata_sha,"fractional_shadow_sha256":sha256_file(&root.join("bridge-shadow-f32le.bin"))?,"parent_binding":bridge.parent_binding(),"fixed_producer_sha256":sha256_bytes(bridge.value_packed()),"canonical_path":format!("checkpoint-{step:04}/canonical.json"),"generation_path":format!("checkpoint-{step:04}/generation.json"),"comparisons_path":format!("checkpoint-{step:04}/comparisons.json"),"scope":"native CE ranking; generation separate; no adoption"});
        write_json(
            &a.out,
            &format!("checkpoint-{step:04}/summary.json"),
            &summary,
        )?;
        Ok(summary)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"optimizer_updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    let mut summary = result?;
    summary["checkpoint_manifest_sha256"] =
        json!(sha256_file(&root.join(report_output::MANIFEST_FILE))?);
    deadline(start, a)?;
    Ok(summary)
}
fn run(a: &Args, start: Instant) -> Result<Value> {
    report_output::verify(&a.checkpoint)?;
    sealed_parent(&a.compiled_panel)?;
    sealed_parent(&a.parent_evaluation)?;
    sealed_parent(&a.fixed_value)?;
    sealed_parent(&a.admission_report)?;
    report_output::verify(&a.baseline_probe)?;
    report_output::verify(&a.transfer_panel_root)?;
    if sha256_file(&a.transfer_panel_root.join(report_output::MANIFEST_FILE))?
        != a.transfer_panel_manifest_sha256
    {
        return Err(invalid("fresh panel was not prospectively bound").into());
    }
    let mut inputs = BTreeMap::new();
    for path in [
        &a.trusted_binding,
        &a.compiled_panel,
        &a.parent_evaluation,
        &a.fixed_value,
        &a.admission_report,
        &a.checkpoint.join(report_output::MANIFEST_FILE),
        &a.baseline_probe.join(report_output::MANIFEST_FILE),
        &a.transfer_panel_root.join(report_output::MANIFEST_FILE),
    ] {
        inputs.insert(path.to_string_lossy().into_owned(), sha256_file(path)?);
    }
    let expected: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.trusted_binding)?)?;
    let native_path = a.checkpoint.join("realizer-native");
    let integer = IntegerRealizer::load_native(&native_path, &expected)?;
    let bytes = fs::read(native_path.join("tokenizer.json"))?;
    let source = SourceRealizerWeights::load_source(&a.checkpoint.join("realizer-source"), &bytes)?;
    let metadata = read_json(&native_path.join("metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(metadata["identity"].clone())?;
    let native = NativeSourceRealizer::load(&native_path, &source, &identity)?;
    if native.artifact_binding()? != expected {
        return Err(invalid("trusted native parent differs").into());
    }
    let frozen = parameter_receipts(&source.parameters())?;
    write_json(&a.out, "frozen-source-parameters.json", &frozen)?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer JSON invalid"))?;
    let panel: Panel = serde_json::from_slice(&fs::read(&a.compiled_panel)?)?;
    let cases = training_cases(&panel, &compiler, &native)?;
    let (fresh, fresh_diagnostics) = transfer_cases(a, &compiler, &tok, &native)?;
    let training_literals = panel
        .training64
        .iter()
        .map(|e| e.literal.as_str())
        .collect::<BTreeSet<_>>();
    for s in &fresh.inputs {
        if training_literals.contains(tok.decode(&s.original_source_ids).as_str()) {
            return Err(invalid("transfer source overlaps development").into());
        }
    }
    write_json(
        &a.out,
        "transfer-prospective-binding.json",
        &json!({"manifest_sha256":a.transfer_panel_manifest_sha256,"source_inputs_sha256":sha256_file(&a.transfer_panel_root.join("source-inputs.json"))?,"labels_sha256":sha256_file(&a.transfer_panel_root.join("labels.json"))?,"cases":32,"prepared_before_optimizer_update":true,"predictions":0,"diagnostics":fresh_diagnostics}),
    )?;
    let value = fs::read(&a.fixed_value)?;
    let fixture_root = a
        .fixed_value
        .parent()
        .ok_or_else(|| invalid("fixed producer parent absent"))?;
    let fixture_report = read_json(&fixture_root.join("report.json"))?;
    if fixture_report["mode"] != "bridge"
        || fixture_report["optimizer_updates"] != 0
        || fixture_report["trusted_binding_sha256"] != sha256_file(&a.trusted_binding)?
    {
        return Err(invalid("fixed producer fixture binding differs").into());
    }
    let admission = read_json(&a.admission_report)?;
    if admission["schema"] != "uor-r4.geometric-dependent-learning-admission/1"
        || admission["status"] != "completed"
        || admission["optimizer_updates"] != 0
        || admission["feedback_input"] != a.feedback_input
        || admission["cases"] != 64
        || admission["target_steps"] != 570
        || admission["actual_parent_replays_exact"] != 530
        || admission["canonical_identity_feedback_exact"] != 570
        || admission["frozen_source_context_readouts"] != true
        || admission["input_files_unchanged"] != true
        || admission["trusted_native_binding"] != serde_json::to_value(&expected)?
        || admission["fixed_producer_sha256"] != sha256_bytes(&value)
    {
        return Err(invalid("positive source-bound full64 admission required").into());
    }
    let admission_inputs = admission["input_files_sha256"]
        .as_object()
        .ok_or_else(|| invalid("admission inputs absent"))?;
    for p in [
        &a.compiled_panel,
        &a.parent_evaluation,
        &a.fixed_value,
        &a.checkpoint.join(report_output::MANIFEST_FILE),
    ] {
        let digest = sha256_file(p)?;
        if !admission_inputs
            .values()
            .any(|v| v.as_str() == Some(digest.as_str()))
        {
            return Err(invalid("admitted source/data hashes differ").into());
        }
    }
    let zero = vec![
        0u8;
        NativeReadFeedback::bridge_coefficient_count(integer.context_config())?
            .div_ceil(2)
    ];
    let feedback =
        NativeReadFeedback::compile(expected.clone(), integer.context_config(), &value, &zero)?;
    let bridge = FeedbackBridgeWeights::from_native(&feedback)?;
    let mut optimizer = AdamW::new(
        bridge.parameters().values().cloned().collect(),
        ParamsAdamW {
            lr: 0.003,
            weight_decay: 0.,
            ..Default::default()
        },
    )?;
    let mut stages = Vec::new();
    stages.push(export(
        a, start, &source, &native, &bridge, &cases, &tok, 0, &stages,
    )?);
    let baseline_export_seconds = start.elapsed().as_secs_f64();
    let parent = read_json(&a.parent_evaluation)?;
    if parent["optimizer_updates"] != 64 || parent["generation"]["cases"] != 64 {
        return Err(invalid("retained grid64 selection differs").into());
    }
    let baseline = read_json(&a.out.join("checkpoint-0000/canonical.json"))?;
    for table in [
        &baseline["rows"],
        &parent["canonical"]["rows"],
        &parent["generation"]["rows"],
    ] {
        if table.as_array().map(Vec::len) != Some(64) {
            return Err(invalid("retained/baseline64 complete rows required").into());
        }
    }
    for (e, (actual, saved)) in panel.training64.iter().zip(
        baseline["rows"]
            .as_array()
            .ok_or_else(|| invalid("baseline canonical rows absent"))?
            .iter()
            .zip(
                parent["canonical"]["rows"]
                    .as_array()
                    .ok_or_else(|| invalid("retained canonical rows absent"))?,
            ),
    ) {
        if actual["id"] != e.id
            || saved["id"] != e.id
            || actual["tokens"].as_array().map(Vec::len) != saved["tokens"].as_array().map(Vec::len)
        {
            return Err(invalid("baseline canonical identity/count differs").into());
        }
        for (l, r) in actual["tokens"]
            .as_array()
            .ok_or_else(|| invalid("baseline tokens absent"))?
            .iter()
            .zip(
                saved["tokens"]
                    .as_array()
                    .ok_or_else(|| invalid("retained tokens absent"))?,
            )
        {
            if l["target_label_only"] != r["target_label_only"]
                || l["teacherforced_prefix_ids"] != r["teacherforced_prefix_ids"]
                || l["actions"] != r["actions"]
            {
                return Err(invalid("native stage0 canonical differs from retained parent").into());
            }
        }
    }
    let baseline_generation = read_json(&a.out.join("checkpoint-0000/generation.json"))?;
    if baseline_generation["rows"].as_array().map(Vec::len) != Some(64) {
        return Err(invalid("baseline generation64 rows required").into());
    }
    for (l, r) in baseline_generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("stage0 rows absent"))?
        .iter()
        .zip(
            parent["generation"]["rows"]
                .as_array()
                .ok_or_else(|| invalid("parent rows absent"))?,
        )
    {
        let mut ids: Vec<u32> = serde_json::from_value(r["generated_ids"].clone())?;
        if r["eos"] == true {
            ids.push(native.binding().eos_token_id());
        }
        if l["id"] != r["id"]
            || l["generated_ids_including_eos"] != json!(ids)
            || l["reply_text"] != r["reply_text"]
        {
            return Err(invalid("stage0 own-prefix differs from retained parent").into());
        }
    }
    let mut previous_packed = feedback.bridge_packed().to_vec();
    let mut previous_actions = Vec::new();
    let mut first_packed = None;
    for step in 1..=UPDATES {
        deadline(start, a)?;
        let current = bridge.compile()?;
        let measured = batch(
            a,
            start,
            &source,
            &native,
            &bridge,
            &current,
            &cases,
            true,
            &previous_actions,
        )?;
        if step == 1 {
            let seconds = measured.canonical["elapsed_seconds"]
                .as_f64()
                .filter(|x| x.is_finite() && *x > 0.)
                .ok_or_else(|| invalid("first fit batch timing absent"))?;
            let reserve = 4. * baseline_export_seconds + 30.;
            let projected = UPDATES as f64 * seconds * 1.25 + reserve;
            let remaining = a.maximum_seconds as f64 - start.elapsed().as_secs_f64();
            write_json(
                &a.out,
                "measured-fit-admission.json",
                &json!({"first_full64_seconds":seconds,"projected_remaining_seconds":projected,"remaining_declared_seconds":remaining,"reserve_seconds":reserve,"admitted":projected<=remaining,"before_optimizer_update1":true}),
            )?;
            if projected > remaining {
                return Err(invalid("actual fit batch exceeds remaining admitted runtime").into());
            }
        }
        let report = apply(&bridge, &mut optimizer, &measured.gradient)?;
        let q = quantum(&bridge, &previous_packed)?;
        if first_packed.is_none() && q["packed_nonzero_coefficients"].as_u64().unwrap_or(0) > 0 {
            first_packed = Some(step);
        }
        previous_packed = bridge.packed_coefficients()?;
        previous_actions = measured.actions;
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("optimizer changed frozen source").into());
        }
        write_json(
            &a.out,
            &format!("batch-{step:04}.json"),
            &json!({"optimizer_update":step,"batch_parameter_step":step-1,"canonical_batch":measured.canonical,"gradient":report,"quantum_after_update":q,"first_packed_nonzero_update":first_packed,"source_frozen":true,"peak_rss_kib_linux":peak_rss_kib()}),
        )?;
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":step,"declared_updates":UPDATES,"elapsed_seconds":start.elapsed().as_secs_f64(),"checkpoints":stages}),
        )?;
        if STAGES.contains(&step) {
            stages.push(export(
                a, start, &source, &native, &bridge, &cases, &tok, step, &stages,
            )?);
        }
    }
    let best = selected(&stages)?;
    let step = stages[best]["optimizer_updates"]
        .as_u64()
        .ok_or_else(|| invalid("selected update absent"))?;
    let chosen_root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::verify(&chosen_root)?;
    let chosen = NativeReadFeedback::load(
        &chosen_root.join("feedback-native"),
        stages[best]["feedback_metadata_sha256"]
            .as_str()
            .ok_or_else(|| invalid("selected metadata SHA absent"))?,
    )?;
    // Selection is now fixed. Only this selected native candidate and unchanged
    // parent see the frozen new transfer inputs, never intermediate checkpoints.
    write_json(
        &a.out,
        "selection-before-transfer.json",
        &json!({"selected_checkpoint_index":best,"selected_updates":step,"criterion":"lowest full64 native equal-episode CE including0; earliest exact tie","checkpoints":stages,"fresh_predictions_before_selection":0}),
    )?;
    let parent_transfer = generation(a, start, &native, None, &fresh, &tok)?;
    let selected_transfer = generation(a, start, &native, Some(&chosen), &fresh, &tok)?;
    write_json(&a.out, "transfer-parent.json", &parent_transfer)?;
    write_json(&a.out, "transfer-selected.json", &selected_transfer)?;
    write_json(
        &a.out,
        "transfer-comparison.json",
        &compare(&parent_transfer, &selected_transfer)?,
    )?;
    for (path, digest) in &inputs {
        if sha256_file(Path::new(path))? != *digest {
            return Err(invalid("input changed during fit").into());
        }
    }
    if parameter_receipts(&source.parameters())? != frozen {
        return Err(invalid("frozen source changed").into());
    }
    deadline(start, a)?;
    Ok(
        json!({"schema":"uor-r4.geometric-dependent-fit/1","status":"completed","feedback_input":a.feedback_input,"optimizer_updates":UPDATES,"declared_updates":UPDATES,"optimizer":{"kind":"AdamW","learning_rate":0.003,"weight_decay":0,"global_gradient_clip":1},"objective":"mean64 episodes(mean complete canonical answer+EOS FINAL native-marginal alias tokenCE)","gradient_accumulation":"per-episode F32 backward; detached F64 sum/64; one clip then F32 GradStore insert; fractional shadows retained","checkpoints":stages,"selected_checkpoint_index":best,"selected_updates":step,"selection_includes_parent_zero":true,"generation_separate_from_selection":true,"fresh_transfer_manifest_sha256":a.transfer_panel_manifest_sha256,"transfer_cases":32,"parent_transfer_complete":parent_transfer["accepted_complete"],"selected_transfer_complete":selected_transfer["accepted_complete"],"first_packed_nonzero_update":first_packed,"frozen_source_context_readouts":true,"fixed_producer_sha256":sha256_bytes(&value),"input_files_sha256":inputs,"input_files_unchanged":true,"trusted_native_binding":expected,"peak_rss_kib_linux":peak_rss_kib(),"elapsed_seconds":start.elapsed().as_secs_f64(),"report_bytes_before_seal":tree_bytes(&a.out)?,"no_adopted_model":true,"scope":"first bounded bridge-only paired fit; no general prose/reasoning/chat or energy qualification"}),
    )
}
fn main() -> Result<()> {
    let a = checked_args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let work = (|| -> Result<Value> {
        let head = option_env!("UOR_BUILD_SOURCE_COMMIT")
            .ok_or_else(|| invalid("build source commit absent"))?;
        if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid("invalid build source commit").into());
        }
        let (exe, lookup) = executable()?;
        let mut report = run(&a, start)?;
        report["source_commit"] = json!(head);
        report["executable_sha256"] = json!(sha256_file(&exe)?);
        report["executable_lookup"] = json!(lookup);
        Ok(report)
    })();
    match &work {
        Ok(report) => write_json(&a.out, "report.json", report)?,
        Err(error) => {
            let progress = read_json(&a.out.join("progress.json")).unwrap_or(Value::Null);
            fs::write(
                a.out.join("failure.json"),
                serde_json::to_vec(
                    &json!({"schema":"uor-r4.geometric-dependent-fit/1","status":"failed_or_stopped","error":error.to_string(),"last_progress":progress,"elapsed_seconds":start.elapsed().as_secs_f64(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
                )?,
            )?;
        }
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    work.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_episode_aggregation_is_f64_mean_and_rejects_partial_gradient() -> Result<()> {
        let mut sum = vec![0f64; 2];
        let g = [0.1f32, -0.3f32];
        for _ in 0..64 {
            accumulate_episode(&mut sum, &g, 64)?;
        }
        assert!((sum[0] - f64::from(g[0])).abs() < 1e-15);
        assert!((sum[1] - f64::from(g[1])).abs() < 1e-15);
        let before = sum.clone();
        assert!(accumulate_episode(&mut sum, &[1.], 64).is_err());
        assert_eq!(sum, before);
        assert!(accumulate_episode(&mut sum, &[f32::NAN, 1.], 64).is_err());
        assert_eq!(sum, before);
        assert!(accumulate_episode(&mut sum, &g, 0).is_err());
        Ok(())
    }
    #[test]
    fn fit_native_selection_includes_zero_and_earliest_tie() -> Result<()> {
        assert_eq!(
            selected(&[
                json!({"native_equal_episode_ce":0.4}),
                json!({"native_equal_episode_ce":0.4}),
                json!({"native_equal_episode_ce":0.5})
            ])?,
            0
        );
        assert_eq!(
            selected(&[
                json!({"native_equal_episode_ce":0.5}),
                json!({"native_equal_episode_ce":0.4}),
                json!({"native_equal_episode_ce":0.4})
            ])?,
            1
        );
        assert!(selected(&[]).is_err());
        assert!(selected(&[json!({"native_equal_episode_ce":null})]).is_err());
        Ok(())
    }
    #[test]
    fn fit_serving_packet_rejects_label_and_view_fields() -> Result<()> {
        let mut packet = json!({"id":"x","record":2,"commit":2,"scope":"m-world-v2","entity":[644,284],"relation":7,"view":0,"status":"Found","original_source_ids":[85],"query_ids":[52],"actual_prefix_ids":[]});
        let _: SourceInput = serde_json::from_value(packet.clone())?;
        packet["target_ids_labels_only"] = json!([85, 1]);
        assert!(serde_json::from_value::<SourceInput>(packet.clone()).is_err());
        packet
            .as_object_mut()
            .ok_or_else(|| invalid("packet object"))?
            .remove("target_ids_labels_only");
        packet["source_view"] = json!({});
        assert!(serde_json::from_value::<SourceInput>(packet).is_err());
        Ok(())
    }
    #[test]
    fn fit_output_guard_rejects_unrelated_sealed_ancestry_without_creating_output() -> Result<()> {
        let root = std::env::temp_dir().join(format!("uor-fit-isolation-{}", std::process::id()));
        report_output::claim(&root)?;
        fs::write(root.join("receipt.json"), b"{}")?;
        report_output::seal(&root)?;
        let output = root.join("nested/new-fit");
        let checked = output_support::prospective_output(&output);
        assert!(checked.is_err());
        assert!(!output.exists());
        report_output::verify(&root)?;
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
