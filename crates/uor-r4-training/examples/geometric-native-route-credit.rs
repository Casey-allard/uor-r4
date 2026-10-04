//! Zero-update exact native conditional token-mixture credit admission.
//! Exhaustive120 lane interventions, frozen actual stage1/source/value, no optimizer.
//! Labels enter the existing offline final loss only; stage1 occurrence selection is detached.
use candle_core::{Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::{
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_read_feedback::{FeedbackInputMode, NativeReadFeedback},
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer},
};
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
const REPORT_CAP: usize = 64 * 1024 * 1024;
const RSS_CAP_KIB: u64 = 8 * 1024 * 1024;
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
    admission_report: PathBuf,
    feedback_input: String,
    maximum_seconds: u64,
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
    let current = fs::read_dir(root)?.try_fold(0usize, |sum, entry| -> io::Result<usize> {
        let metadata = entry?.metadata()?;
        Ok(sum.saturating_add(metadata.len() as usize))
    })?;
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
    if peak_rss_kib().is_some_and(|rss| rss > RSS_CAP_KIB) {
        return Err(invalid("declared admission RSS limit reached").into());
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
fn run(a: &Args, start: Instant) -> Result<Value> {
    report_output::verify(&a.checkpoint)?;
    sealed_parent(&a.compiled_panel)?;
    sealed_parent(&a.parent_evaluation)?;
    sealed_parent(&a.fixed_value)?;
    sealed_parent(&a.admission_report)?;
    report_output::verify(&a.baseline_probe)?;
    let mut inputs = BTreeMap::new();
    for path in [
        &a.trusted_binding,
        &a.compiled_panel,
        &a.parent_evaluation,
        &a.fixed_value,
        &a.admission_report,
        &a.checkpoint.join("manifest.json"),
        &a.baseline_probe.join("manifest.json"),
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
        return Err(invalid(
            "source-backed parent differs from independently trusted native binding",
        )
        .into());
    }
    let old_admission = read_json(&a.admission_report)?;
    if old_admission["schema"] != "uor-r4.geometric-dependent-learning-admission/1"
        || old_admission["status"] != "completed"
        || old_admission["optimizer_updates"] != 0
        || old_admission["feedback_input"] != a.feedback_input
        || old_admission["cases"] != 64
        || old_admission["target_steps"] != 570
        || old_admission["actual_parent_replays_exact"] != 530
        || old_admission["canonical_identity_feedback_exact"] != 570
        || old_admission["frozen_source_context_readouts"] != true
        || old_admission["input_files_unchanged"] != true
        || old_admission["trusted_native_binding"] != serde_json::to_value(&expected)?
    {
        return Err(invalid("retained zero-update learning admission differs").into());
    }
    let old_inputs = old_admission["input_files_sha256"]
        .as_object()
        .ok_or_else(|| invalid("retained admission hashes absent"))?;
    for path in [
        &a.compiled_panel,
        &a.parent_evaluation,
        &a.fixed_value,
        &a.checkpoint.join("manifest.json"),
    ] {
        let hash = sha256_file(path)?;
        if !old_inputs
            .values()
            .any(|v| v.as_str() == Some(hash.as_str()))
        {
            return Err(
                invalid("retained admission exact source/data input binding differs").into(),
            );
        }
    }
    let prepared = source.prepare(&native)?;
    let frozen = parameter_receipts(&source.parameters())?;
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let panel: Panel = serde_json::from_slice(&fs::read(&a.compiled_panel)?)?;
    let parent = read_json(&a.parent_evaluation)?;
    if parent["optimizer_updates"] != 64
        || parent["generation"]["cases"] != 64
        || panel.training64.len() != 64
    {
        return Err(invalid("selected grid64/all64 development input required").into());
    }
    let fixture_root = a
        .fixed_value
        .parent()
        .ok_or_else(|| invalid("value envelope absent"))?;
    let fixture_report = read_json(&fixture_root.join("report.json"))?;
    if fixture_report["mode"] != "bridge"
        || fixture_report["optimizer_updates"] != 0
        || fixture_report["trusted_binding_sha256"] != sha256_file(&a.trusted_binding)?
    {
        return Err(invalid("fixed producer parent/fixture binding differs").into());
    }
    let value = fs::read(&a.fixed_value)?;
    if old_admission["fixed_producer_sha256"] != sha256_bytes(&value)
        || read_json(
            &a.admission_report
                .parent()
                .ok_or_else(|| invalid("admission parent absent"))?
                .join("frozen-source-parameters.json"),
        )? != frozen
    {
        return Err(invalid("retained frozen producer/context/readouts differ").into());
    }
    let count = NativeReadFeedback::bridge_coefficient_count(integer.context_config())?;
    let zero = vec![0u8; count.div_ceil(2)];
    let identity_feedback =
        NativeReadFeedback::compile(expected.clone(), integer.context_config(), &value, &zero)?;
    let bridge = FeedbackBridgeWeights::from_native(&identity_feedback)?;
    bridge.project_shadow_range()?;
    let update = bridge.compile()?;
    if update.bridge_packed().iter().any(|v| *v != 0) {
        return Err(invalid("admission bridge must be zero identity").into());
    }
    fs::write(a.out.join("fixed-value-q4.bin"), update.value_packed())?;
    fs::write(a.out.join("bridge-q4.bin"), update.bridge_packed())?;
    write_json(
        &a.out,
        "feedback-metadata.json",
        &serde_json::to_value(update.metadata())?,
    )?;
    let mode = if a.feedback_input == "joint-copy" {
        FeedbackInputMode::JointCopy
    } else {
        FeedbackInputMode::RoleSurface
    };
    let baseline_report = read_json(&a.baseline_probe.join("report.json"))?;
    if baseline_report["mode"] != "baseline"
        || baseline_report["generation_token_cap"] != 64
        || baseline_report["trusted_binding_sha256"] != sha256_file(&a.trusted_binding)?
    {
        return Err(invalid("full-prefix baseline binding differs").into());
    }
    let baseline_rows = baseline_report["rows"]
        .as_array()
        .ok_or_else(|| invalid("baseline rows absent"))?;
    if baseline_rows.len() != 64 {
        return Err(invalid("baseline64 required").into());
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut gradient_sum: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let parameters = bridge.parameters();
    let bridge_before = parameter_receipts(&parameters)?;
    write_json(&a.out, "frozen-source-parameters.json", &frozen)?;
    write_json(&a.out, "bridge-shadow-parameters.json", &bridge_before)?;
    let mut actual_replays = 0usize;
    let mut total_steps = 0usize;
    let mut mean_ce = 0f64;
    let mut backward_seconds = 0f64;
    let mut counterfactual_seconds = 0f64;
    let mut counterfactual_actions = 0usize;
    let mut zero_support_actions = 0usize;
    let mut useful_actions = 0usize;
    let mut lanes_with_useful_actions = 0usize;
    let mut maximum_utility_gain = 0f64;
    for (index, episode) in panel.training64.iter().enumerate() {
        deadline(start, a)?;
        if !seen.insert(&episode.id)
            || episode.id.is_empty()
            || episode.target_ids_labels_only.is_empty()
            || episode.target_ids_labels_only.len() > 64
            || episode.target_ids_labels_only.last().copied()
                != Some(integer.binding().eos_token_id())
            || episode.query_ids.is_empty()
            || episode.original_source_ids.is_empty()
            || [
                &episode.query_ids,
                &episode.original_source_ids,
                &episode.target_ids_labels_only,
            ]
            .iter()
            .any(|ids| ids.iter().any(|id| *id >= 4096))
            || episode.target_ids_labels_only[..episode.target_ids_labels_only.len() - 1]
                .contains(&integer.binding().eos_token_id())
        {
            return Err(invalid("case identity/complete label admission differs").into());
        }
        let view = compiler.compile(&episode.original_source_ids)?;
        let parent_row = &parent["generation"]["rows"][index];
        if parent_row["id"] != episode.id
            || parent_row["original_source_ids"] != json!(episode.original_source_ids)
            || parent_row["query_ids"] != json!(episode.query_ids)
            || parent_row["source_record"] != episode.record
            || parent_row["source_commit"] != episode.commit
        {
            return Err(invalid(
                "frozen panel differs from grid64 parent generation source/query identity",
            )
            .into());
        }
        if serde_json::to_value(&view)? != episode.source_view
            || view.emitted_token_ids().len() + episode.query_ids.len() + 64 > 128
        {
            return Err(invalid("frozen source-only view/context window differs").into());
        }
        let saved = &baseline_rows[index];
        let relative = saved["path"]
            .as_str()
            .ok_or_else(|| invalid("baseline row path absent"))?;
        if Path::new(relative)
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err(invalid("baseline path traversal").into());
        }
        let path = a.baseline_probe.join(relative);
        if saved["id"] != episode.id || saved["sha256"] != sha256_file(&path)? {
            return Err(invalid("baseline row identity/hash differs").into());
        }
        let replay = read_json(&path)?;
        let mut reached = Vec::new();
        for (position, step) in replay["steps"]
            .as_array()
            .ok_or_else(|| invalid("actual baseline steps absent"))?
            .iter()
            .enumerate()
        {
            deadline(start, a)?;
            let prefix: Vec<u32> = serde_json::from_value(step["actual_prefix_ids"].clone())?;
            if prefix != reached
                || step["step"] != position
                || reached.last().copied() == Some(integer.binding().eos_token_id())
            {
                return Err(
                    invalid("saved replay is not an actual causal own-prefix sequence").into(),
                );
            }
            let independent = integer.read(episode.frame(), &view, &episode.query_ids, &prefix)?;
            let actual = native.read(episode.frame(), &view, &episode.query_ids, &prefix)?;
            if serde_json::to_value(&independent)? != step["trace"]
                || serde_json::to_value(&actual)? != step["trace"]
            {
                return Err(invalid("actual own-prefix parent full trace differs").into());
            }
            if step["chosen_token_id"] != actual.actions.chosen_token_id {
                return Err(invalid("saved chosen token differs from actual native action").into());
            }
            let identity_native = native.read_dependent(
                episode.frame(),
                &view,
                &episode.query_ids,
                &prefix,
                &update,
                mode,
            )?;
            if identity_native.stage1 != actual
                || identity_native.stage2 != actual
                || identity_native.feedback.before != identity_native.feedback.after
            {
                return Err(invalid("actual own-prefix zero-bridge full trace differs").into());
            }
            reached.push(actual.actions.chosen_token_id);
            actual_replays += 1;
        }
        if replay["generated_ids_including_eos"] != json!(reached) {
            return Err(invalid("saved full generation sequence differs").into());
        }
        let canonical = &parent["canonical"]["rows"][index];
        if canonical["id"] != episode.id {
            return Err(invalid("canonical parent case ordering differs").into());
        }
        let mut losses = Vec::new();
        let mut token_receipts = Vec::new();
        let row_start = Instant::now();
        for (step, target) in episode.target_ids_labels_only.iter().copied().enumerate() {
            deadline(start, a)?;
            let prefix = &episode.target_ids_labels_only[..step];
            let measured = match prepared.loss_native_route_with_native(
                episode.frame(),
                &view,
                &episode.query_ids,
                prefix,
                target,
                &bridge,
                &update,
                mode,
            ) {
                Ok(measured) => measured,
                Err(error) => {
                    let replay: Result<Value> = match native.read_dependent(
                        episode.frame(),
                        &view,
                        &episode.query_ids,
                        prefix,
                        &update,
                        mode,
                    ) {
                        Ok(trace) => Ok(serde_json::to_value(trace)?),
                        Err(replay_error) => Err(replay_error.into()),
                    };
                    // Error-only factual read: no huge counterfactual dump or altered input.
                    let witness = match replay {
                        Ok(trace) => json!({"trace":trace}),
                        Err(replay_error) => json!({"replay_error":replay_error.to_string()}),
                    };
                    write_json(
                        &a.out,
                        "failed-native-route-step.json",
                        &json!({"id":episode.id,"step":step,"original_source_ids":episode.original_source_ids,"query_ids":episode.query_ids,"actual_prefix_ids":prefix,"target_label_only":target,"error":error.to_string(),"factual_error_only_replay":witness,"completed_tokens_in_row":token_receipts,"optimizer_updates":0,"bridge_shadow_receipt":bridge_before,"frozen_source_receipt":frozen}),
                    )?;
                    return Err(error.into());
                }
            };
            let trace = &measured.trace;
            let saved = &canonical["tokens"][step];
            if saved["target_label_only"] != target
                || saved["teacherforced_prefix_ids"] != json!(prefix)
                || serde_json::to_value(&trace.stage1.actions)? != saved["actions"]
                || serde_json::to_value(&trace.stage1)? != serde_json::to_value(&trace.stage2)?
                || trace.feedback.before != trace.feedback.after
                || trace.stage2_controller_snapshot != trace.feedback.after
            {
                return Err(invalid("canonical baseline/native identity feedback differs").into());
            }
            let ce = -measured.target_probability.ln();
            if !ce.is_finite() {
                return Err(invalid("nonfinite native full-answer CE").into());
            }
            if measured.loss.to_scalar::<f32>()?.to_bits()
                != (-(measured.target_probability as f32).ln()).to_bits()
            {
                // Independent ordinary loss uses Candle's own scalar log implementation.
                let native_loss = Tensor::new(
                    measured.target_probability as f32,
                    &candle_core::Device::Cpu,
                )?
                .log()?
                .neg()?;
                if measured.loss.to_scalar::<f32>()?.to_bits()
                    != native_loss.to_scalar::<f32>()?.to_bits()
                {
                    return Err(invalid("native route hard loss forward differs").into());
                }
            }
            if measured.counterfactual_actions != 960 || measured.utilities.len() != 8 {
                return Err(invalid("all120/eight-lane utilities required").into());
            }
            counterfactual_seconds += measured.counterfactual_seconds;
            counterfactual_actions += measured.counterfactual_actions;
            for utility in &measured.utilities {
                zero_support_actions += utility.zero_support_actions;
                useful_actions += utility.useful_actions;
                lanes_with_useful_actions += usize::from(utility.useful_actions > 0);
                maximum_utility_gain = maximum_utility_gain
                    .max(utility.best_target_probability - measured.target_probability);
                let alignment = utility
                    .policy_adjoint
                    .iter()
                    .zip(&utility.target_probabilities)
                    .map(|(g, u)| g * u)
                    .sum::<f64>();
                if !alignment.is_finite() || alignment > 1e-6 {
                    return Err(invalid(
                        "native probability policy credit has wrong utility direction",
                    )
                    .into());
                }
            }
            token_receipts.push(json!({"native_action_utilities":measured.utilities,"counterfactual_seconds":measured.counterfactual_seconds,"counterfactual_actions":measured.counterfactual_actions,"hard_forward_loss_exact":true,"step":step,"prefix_ids":prefix,"target_label_only":target,"native_ce":ce,"target_probability":measured.target_probability,"stage1_actions":trace.stage1.actions,"stage2_actions":trace.stage2.actions,"feedback":trace.feedback,"stage2_controller_snapshot":trace.stage2_controller_snapshot,"stage2_context_replay_policy":trace.stage2_context_replay_policy,"identity_feedback_exact":true}));
            losses.push(measured.loss);
        }
        let loss = Tensor::stack(&losses, 0)?.mean_all()?;
        let loss_scalar = loss.to_scalar::<f32>()?;
        if !loss_scalar.is_finite() {
            return Err(invalid("nonfinite offline marginal loss").into());
        }
        let backward_start = Instant::now();
        let gradients = loss.backward()?;
        backward_seconds += backward_start.elapsed().as_secs_f64();
        let mut gradient_receipts = BTreeMap::new();
        for (name, var) in &parameters {
            let gradient = gradients
                .get(var)
                .ok_or_else(|| invalid(format!("disconnected bridge family {name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if gradient.iter().any(|v| !v.is_finite()) {
                return Err(invalid("nonfinite bridge gradient").into());
            }
            let sum = gradient_sum
                .entry(name.clone())
                .or_insert_with(|| vec![0.; gradient.len()]);
            if sum.len() != gradient.len() {
                return Err(invalid("bridge gradient shape changed").into());
            }
            for (sum, g) in sum.iter_mut().zip(&gradient) {
                *sum += f64::from(*g) / 64.;
            }
            gradient_receipts.insert(name.clone(),json!({"elements":gradient.len(),"nonzero":gradient.iter().filter(|v|**v!=0.).count(),"l1":gradient.iter().map(|g|f64::from(g.abs())).sum::<f64>(),"l2":gradient.iter().map(|g|f64::from(*g).powi(2)).sum::<f64>().sqrt(),"finite":true}));
        }
        let row_ce = token_receipts
            .iter()
            .map(|v| v["native_ce"].as_f64().unwrap_or(f64::NAN))
            .sum::<f64>()
            / losses.len() as f64;
        mean_ce += row_ce / 64.;
        total_steps += losses.len();
        let receipt = json!({"id":episode.id,"target_steps":losses.len(),"native_mean_token_ce":row_ce,"offline_loss_f32":loss_scalar,"ordinary_episode_gradient":gradient_receipts,"tokens":token_receipts,"elapsed_seconds":row_start.elapsed().as_secs_f64()});
        let name = format!("admission-row-{index:04}.json");
        write_json(&a.out, &name, &receipt)?;
        rows.push(json!({"id":episode.id,"path":name,"sha256":sha256_file(&a.out.join(&name))?,"native_mean_token_ce":row_ce}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"completed_episodes":index+1,"target_steps":total_steps,"actual_parent_replays":actual_replays,"optimizer_updates":0,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
    }
    if total_steps != 570 || actual_replays != 530 || counterfactual_actions != 547200 {
        return Err(invalid("complete canonical/actual-prefix/all-action coverage differs").into());
    }
    let mut aggregate = BTreeMap::new();
    for (name, g) in &gradient_sum {
        let bytes = g
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect::<Vec<_>>();
        aggregate.insert(name.clone(),json!({"elements":g.len(),"finite":g.iter().all(|v|v.is_finite()),"nonzero":g.iter().filter(|v|**v!=0.).count(),"l1":g.iter().map(|v|v.abs()).sum::<f64>(),"l2":g.iter().map(|v|v*v).sum::<f64>().sqrt(),"f64_le_bits_sha256":sha256_bytes(&bytes)}));
    }
    if frozen != parameter_receipts(&source.parameters())? {
        return Err(invalid("frozen source/readouts/context changed").into());
    }
    if bridge_before != parameter_receipts(&bridge.parameters())? {
        return Err(invalid("zero-update bridge shadows changed").into());
    }
    for (path, hash) in &inputs {
        if sha256_file(Path::new(path))? != *hash {
            return Err(invalid("input file changed").into());
        }
    }
    report_output::verify(&a.checkpoint)?;
    let final_update = bridge.compile()?;
    if final_update.bridge_packed() != update.bridge_packed()
        || final_update.value_packed() != value
    {
        return Err(invalid("zero-update feedback payload changed").into());
    }
    write_json(&a.out, "frozen-source-parameters.json", &frozen)?;
    Ok(
        json!({"schema":"uor-r4.geometric-native-route-credit-admission/1","status":"completed","optimizer_updates":0,"feedback_input":a.feedback_input,"cases":64,"target_steps":total_steps,"native_equal_episode_ce":mean_ce,"objective":"hard current native marginal token CE forward; conditional lane latent-policy marginal native-token probability adjoint, mean8 lanes/complete answer+EOS tokens/64episodes","surrogate":uor_r4_training::geometric_read_feedback::ROUTE_SURROGATE,"counterfactual_actions":counterfactual_actions,"counterfactual_seconds":counterfactual_seconds,"utility_zero_support_actions":zero_support_actions,"utility_useful_actions":useful_actions,"utility_lanes_with_improvement":lanes_with_useful_actions,"maximum_native_target_probability_gain":maximum_utility_gain,"action_count_per_lane":120,"global_lanes":8,"cache":"none; one shared preparation per prefix; full120 action traces released after each prefix","runtime_unchanged":true,"gradient_accumulation":"per-episode mean-token loss backward; detached f64 summed episode gradients divided by64, no optimizer; graph lifetime one episode","aggregate_gradient":aggregate,"rows":rows,"actual_parent_replays_exact":actual_replays,"actual_ownprefix_identity_feedback_exact":actual_replays,"canonical_identity_feedback_exact":total_steps,"frozen_source_context_readouts":true,"stage1_occurrence":"actual detached conditional highest joint Copy; no target offset or forced final action","fixed_producer_sha256":sha256_bytes(&value),"input_files_sha256":inputs,"input_files_unchanged":true,"trusted_native_binding":expected,"feedback_stats":update.stats(),"backward_seconds":backward_seconds,"peak_rss_kib_linux":peak_rss_kib(),"elapsed_seconds":start.elapsed().as_secs_f64(),"heldout_predictions":0,"scope":"zero-update exact native utility/gradient admission; no fitted route, hard-action descent guarantee, transfer/chat/energy qualification"}),
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
        Err(error) => fs::write(
            a.out.join("failure.json"),
            serde_json::to_vec(
                &json!({"schema":"uor-r4.geometric-native-route-credit-admission/1","status":"failed_or_stopped","optimizer_updates":0,"error":error.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
            )?,
        )?,
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    work.map(|_| ())
}
