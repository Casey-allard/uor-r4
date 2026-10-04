//! Exact native winner-crossing singleton root proposals with frozen geometric source/readouts.
//! Conditional utility ranks verified geometric action crossings; full native CE accepts them.
use candle_core::{Device, Tensor, Var};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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
    geometric_read_feedback::{FeedbackInputMode, FeedbackTrace, NativeReadFeedback},
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer},
    geometric_value_q4::canonical_basis_q25,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_occurrence_consumer::{
        source_realizer::{
            FeedbackBridgeWeights, NativeRouteLaneUtility, NativeSourceRealizer,
            SourceRealizerWeights,
        },
        ConsumerIdentity,
    },
    geometric_source_emission_view::SourceEmissionCompiler,
    sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT_CAP: usize = 256 * 1024 * 1024;
const MAX_ACCEPTED_ROUNDS: usize = 4;
const CANDIDATES_PER_ROUND: usize = 16;
const EXPOSED_NATIVE_PANEL: &str =
    "8e040ec65c90097c942827c21095dd920066627a76374dc20505dcddb383b369";
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
    transfer_preparation_seed: u64,
    prior_exposed_panel_manifest_sha256: String,
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
        || a.prior_exposed_panel_manifest_sha256.len() != 64
        || !a
            .prior_exposed_panel_manifest_sha256
            .bytes()
            .all(|x| x.is_ascii_hexdigit())
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
    if peak_rss_kib().is_some_and(|rss| rss > 8 * 1024 * 1024) {
        return Err(invalid("fit8GiB RSS cap exceeded").into());
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
#[derive(Clone, Deserialize, serde::Serialize)]
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
    pool_index: usize,
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
fn validate_seeded_transfer_receipt(
    report: &Value,
    seed: u64,
    prior_exposed_sha256: &str,
) -> Result<()> {
    if report["panel_family"] != "native-route-seeded-transfer-v1"
        || report["finite_pool_version"] != "native-route-transfer-pairs-v1"
        || report["preparation_seed"].as_u64() != Some(seed)
        || report["prior_exposed_panel_manifest_sha256"] != prior_exposed_sha256
        || report["source_literal_disjoint_all_declared_sealed_exposures"] != true
        || report["complete_input_fingerprints_disjoint"] != true
        || report["query_counts"] != json!([16, 16])
        || report["class_query_counts"] != json!([[8, 8], [8, 8]])
        || report["actual_BPE_familiar_source_rows"] != 16
        || !report["exposed_panel_roots"]
            .as_array()
            .is_some_and(|roots| {
                roots
                    .iter()
                    .any(|r| r["manifest_sha256"] == prior_exposed_sha256)
            })
    {
        return Err(invalid("seeded prospective pool/seed/exposed-panel binding differs").into());
    }
    Ok(())
}
/// Labels retain provenance from the sealed seeded pool; this never supplies a
/// serving action, source offset or native target to the reader.
fn validate_transfer_pool_label(
    label: &TransferLabel,
    row: usize,
    pool: &Value,
    selected: &[(usize, usize, usize)],
) -> Result<()> {
    let chosen = selected
        .get(row / 2)
        .ok_or_else(|| invalid("selected pool pair absent"))?;
    let side = if row % 2 == 0 { "left" } else { "right" };
    let pair = pool["pairs"]
        .as_array()
        .and_then(|pairs| pairs.get(label.pool_index))
        .ok_or_else(|| invalid("label pool index is out of bounds"))?;
    if label.pool_index != chosen.0
        || label.side != side
        || pair[side] != label.literal
        || pair["stratum"] != label.stratum
        || label.pair_id != format!("native-route-pair-{:02}", row / 2)
    {
        return Err(
            invalid("label literal/side/stratum differs from selected frozen pool row").into(),
        );
    }
    Ok(())
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
    validate_seeded_transfer_receipt(
        &report,
        a.transfer_preparation_seed,
        &a.prior_exposed_panel_manifest_sha256,
    )?;
    for (file, field) in [
        ("pair-pool.json", "finite_pool_sha256"),
        ("exclusions.json", "exclusion_receipt_sha256"),
        ("training-known-ids.json", "training_known_ids_sha256"),
        ("eligibility.json", "eligibility_receipt_sha256"),
    ] {
        if report[field] != sha256_file(&a.transfer_panel_root.join(file))? {
            return Err(invalid("seeded prospective receipt hash differs").into());
        }
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
    let pool = read_json(&a.transfer_panel_root.join("pair-pool.json"))?;
    let eligibility = read_json(&a.transfer_panel_root.join("eligibility.json"))?;
    if pool["seed"] != a.transfer_preparation_seed
        || pool["version"] != "native-route-transfer-pairs-v1"
        || eligibility["seed"] != a.transfer_preparation_seed
        || eligibility["schema"] != "uor-r4.native-route-transfer-eligibility/1"
        || eligibility["pool_version"] != pool["version"]
    {
        return Err(invalid("frozen pool/eligibility seed/version differs").into());
    }
    let mut selected: Vec<(usize, usize, usize)> =
        serde_json::from_value(eligibility["selected_pool_indices"].clone())?;
    if selected.len() != 16 || selected.iter().map(|s| s.0).collect::<BTreeSet<_>>().len() != 16 {
        return Err(invalid("selected frozen pool count/uniqueness differs").into());
    }
    selected.sort_by_key(|(_, class_index, class)| (*class, *class_index));
    let mut seen = BTreeSet::new();
    let mut literals = BTreeSet::new();
    let mut targets = Vec::new();
    let mut accepted = Vec::new();
    let mut views = Vec::new();
    for (row, (s, l)) in inputs.cases.iter().zip(&labels.cases).enumerate() {
        validate_transfer_pool_label(l, row, &pool, &selected)?;
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
    let diagnostics=serde_json::to_value(labels.cases.iter().map(|l|json!({"id":l.id,"stratum":l.stratum,"pair_id":l.pair_id,"side":l.side,"pool_index":l.pool_index,"literal":l.literal,"query":l.query,"token_support":l.token_support})).collect::<Vec<_>>())?;
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
#[derive(Default)]
struct UtilityStats {
    positions: usize,
    actions: usize,
    zero_support: usize,
    useful_actions: usize,
    improving_lanes: usize,
    mixture_min: Option<f64>,
    mixture_max: f64,
    mixture_sum: f64,
    lanes: usize,
    counterfactual_seconds: f64,
    maximum_native_probability_gain: f64,
}
impl UtilityStats {
    fn observe(
        &mut self,
        utilities: &[NativeRouteLaneUtility],
        factual: f64,
        seconds: f64,
    ) -> Result<()> {
        if utilities.len() != 8
            || !factual.is_finite()
            || factual <= 0.0
            || !seconds.is_finite()
            || seconds < 0.0
        {
            return Err(invalid("native utility summary shape/finite differs").into());
        }
        for (lane, u) in utilities.iter().enumerate() {
            if u.lane != lane
                || u.target_mass_q31.len() != 120
                || u.total_weight_q31.len() != 120
                || u.target_probabilities.len() != 120
                || u.policy_adjoint.len() != 120
                || !u.mixture_probability.is_finite()
                || u.mixture_probability <= 0.0
                || !u.best_target_probability.is_finite()
                || u.best_target_probability > 1.0
            {
                return Err(invalid("native utility summary incomplete/nonfinite").into());
            }
            self.zero_support += u.zero_support_actions;
            self.useful_actions += u.useful_actions;
            self.improving_lanes += usize::from(u.useful_actions > 0);
            self.mixture_min = Some(
                self.mixture_min
                    .map_or(u.mixture_probability, |min| min.min(u.mixture_probability)),
            );
            self.mixture_max = self.mixture_max.max(u.mixture_probability);
            self.mixture_sum += u.mixture_probability;
            self.lanes += 1;
            self.maximum_native_probability_gain = self
                .maximum_native_probability_gain
                .max(u.best_target_probability - factual);
        }
        self.positions += 1;
        self.actions += utilities.len() * 120;
        self.counterfactual_seconds += seconds;
        Ok(())
    }
    fn receipt(&self) -> Value {
        json!({"positions":self.positions,"actions":self.actions,"zero_support_actions":self.zero_support,"useful_actions":self.useful_actions,"improving_lanes":self.improving_lanes,"lane_positions":self.lanes,"mixture_min":self.mixture_min,"mixture_max":self.mixture_max,"mixture_mean":if self.lanes==0{None}else{Some(self.mixture_sum/self.lanes as f64)},"counterfactual_seconds":self.counterfactual_seconds,"maximum_native_probability_gain":self.maximum_native_probability_gain,"utility_cache":false})
    }
}
fn ordered_digest_item(digest: &mut Sha256, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
    Ok(())
}

struct Batch {
    gradient: Vec<f64>,
    canonical: Value,
    actions: Vec<Vec<u8>>,
    crossings: Vec<Crossing>,
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
    let mut utility_stats = UtilityStats::default();
    let q = shadow_quanta(bridge)?;
    let mut crossings = Vec::new();
    let mut coverage = CrossingCoverage::default();
    let mut utility_stream = Sha256::new();
    ordered_digest_item(
        &mut utility_stream,
        &json!({"schema":"uor-r4.native-route-ordered-utilities/1","feedback_metadata":compiled.metadata()}),
    )?;
    for (row, s) in cases.inputs.iter().enumerate() {
        deadline(start, a)?;
        let mut losses = Vec::new();
        let mut tokens = Vec::new();
        let mut sum = 0f64;
        for (step, &target) in cases.targets[row].iter().enumerate() {
            deadline(start, a)?;
            let prefix = &cases.targets[row][..step];
            let (loss, trace, utility_digest) = if backward {
                let measured = prepared.loss_native_route_with_native(
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
                utility_stats.observe(
                    &x.utilities,
                    x.target_probability,
                    x.counterfactual_seconds,
                )?;
                if x.counterfactual_actions != 960 {
                    return Err(invalid("native route fit omitted actions").into());
                }
                discover_crossings(
                    &q,
                    bridge,
                    &x.trace.feedback,
                    &x.utilities,
                    row,
                    step,
                    &s.id,
                    target,
                    &sha256_bytes(&serde_json::to_vec(&x.trace.stage1)?),
                    &mut crossings,
                    &mut coverage,
                )?;
                let record = json!({"id":s.id,"source_frame":{"record":s.record,"commit":s.commit,"scope":s.scope,"entity":s.entity,"relation":s.relation,"view":s.view,"status":s.status},"frame_binding_sha256":x.trace.feedback.before.frame_binding_sha256,"source_ids":s.original_source_ids,"query_ids":s.query_ids,"actual_prefix_ids":prefix,"target_label_only":target,"factual_actions":x.trace.feedback.actions,"factual_final_actions":x.trace.stage2.actions,"utilities":x.utilities});
                let prefix_digest = sha256_bytes(&serde_json::to_vec(&record)?);
                ordered_digest_item(&mut utility_stream, &record)?;
                (Some(x.loss), x.trace, Some(prefix_digest))
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
                    None,
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
            if backward {
                tokens.push(json!({"step":step,"teacherforced_prefix_ids":prefix,"target_label_only":target,"native_ce":ce,"feedback_actions":trace.feedback.actions,"factual_action_trace_sha256":sha256_bytes(&serde_json::to_vec(&trace.stage2.actions)?),"ordered_native_utility_sha256":utility_digest}));
            } else {
                tokens.push(json!({"step":step,"teacherforced_prefix_ids":prefix,"target_label_only":target,"native_ce":ce,"actions":trace.stage2.actions,"feedback_actions":trace.feedback.actions,"selected_occurrence":trace.feedback.selected_occurrence,"producer_packets":trace.feedback.value.packets,"refined_states":trace.stage2_controller_snapshot.states,"query_codes":trace.stage2_controller_snapshot.codes}));
            }
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
    if backward
        && (utility_stats.positions != 570
            || utility_stats.actions != 547200
            || utility_stats.lanes != 4560)
    {
        return Err(invalid("full64/570 exhaustive native utilities incomplete").into());
    }
    Ok(Batch {
        gradient,
        actions,
        crossings,
        canonical: json!({"crossing_search_coverage":coverage,"native_utility_summary":utility_stats.receipt(),"ordered_native_utility_stream_sha256":format!("{:x}",utility_stream.finalize()),"full_utility_arrays_retained":false,"utility_recomputed_current_actions":backward,"cases":64,"mean_equal_episode_ce":mean,"rows":rows,"backward":backward,"feedback_action_positions_changed_from_previous":changed,"elapsed_seconds":begun.elapsed().as_secs_f64()}),
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
        let canonical = factual(a, start, native, &reload, cases)?;
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
            let oldstep = old["accepted_updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior step absent"))?;
            let oldroot = a.out.join(format!("checkpoint-{oldstep:04}"));
            comparisons.push(json!({"prior_accepted_updates":oldstep,"canonical":compare(&read_json(&oldroot.join("canonical.json"))?,&canonical)?,"generation":compare(&read_json(&oldroot.join("generation.json"))?,&generate)?}));
        }
        write_json(
            &a.out,
            &format!("checkpoint-{step:04}/comparisons.json"),
            &json!(comparisons),
        )?;
        let summary = json!({"accepted_updates":step,"native_equal_episode_ce":canonical["mean_equal_episode_ce"],"accepted_complete":generate["accepted_complete"],"eos":generate["eos"],"feedback_metadata_sha256":metadata_sha,"fractional_shadow_sha256":sha256_file(&root.join("bridge-shadow-f32le.bin"))?,"parent_binding":bridge.parent_binding(),"fixed_producer_sha256":sha256_bytes(bridge.value_packed()),"canonical_path":format!("checkpoint-{step:04}/canonical.json"),"generation_path":format!("checkpoint-{step:04}/generation.json"),"comparisons_path":format!("checkpoint-{step:04}/comparisons.json"),"scope":"native CE ranking; generation separate; no adoption"});
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
            &json!({"error":e.to_string(),"accepted_updates":step}),
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
fn validate_route_admission(
    admission: &Value,
    feedback_input: &str,
    expected: &Value,
    producer_sha256: &str,
) -> Result<()> {
    if admission["schema"] != "uor-r4.geometric-native-route-credit-admission/1"
        || admission["status"] != "completed"
        || admission["optimizer_updates"] != 0
        || admission["feedback_input"] != feedback_input
        || admission["cases"] != 64
        || admission["target_steps"] != 570
        || admission["counterfactual_actions"] != 547200
        || admission["action_count_per_lane"] != 120
        || admission["global_lanes"] != 8
        || admission["actual_ownprefix_identity_feedback_exact"] != 530
        || admission["aggregate_gradient"]["bridge.coefficients"]["elements"] != 79680
        || admission["aggregate_gradient"]["bridge.coefficients"]["finite"] != true
        || !admission["aggregate_gradient"]["bridge.coefficients"]["nonzero"]
            .as_u64()
            .is_some_and(|n| n > 0 && n <= 79680)
        || admission["surrogate"] != uor_r4_training::geometric_read_feedback::ROUTE_SURROGATE
        || admission["actual_parent_replays_exact"] != 530
        || admission["canonical_identity_feedback_exact"] != 570
        || admission["frozen_source_context_readouts"] != true
        || admission["input_files_unchanged"] != true
        || admission["trusted_native_binding"] != *expected
        || admission["fixed_producer_sha256"] != producer_sha256
    {
        return Err(invalid("positive source-bound full64 native-route admission required").into());
    }
    Ok(())
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
    let fresh_report = read_json(&a.transfer_panel_root.join("report.json"))?;
    if !fresh_report["exposed_panel_roots"]
        .as_array()
        .is_some_and(|roots| {
            roots
                .iter()
                .any(|r| r["manifest_sha256"] == EXPOSED_NATIVE_PANEL)
        })
    {
        return Err(invalid("previous native-route panel exposure must be excluded").into());
    }
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
        &json!({"manifest_sha256":a.transfer_panel_manifest_sha256,"preparation_seed":a.transfer_preparation_seed,"prior_exposed_panel_manifest_sha256":a.prior_exposed_panel_manifest_sha256,"source_inputs_sha256":sha256_file(&a.transfer_panel_root.join("source-inputs.json"))?,"labels_sha256":sha256_file(&a.transfer_panel_root.join("labels.json"))?,"cases":32,"prepared_before_optimizer_update":true,"predictions":0,"diagnostics":fresh_diagnostics}),
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
    validate_route_admission(
        &admission,
        &a.feedback_input,
        &serde_json::to_value(&expected)?,
        &sha256_bytes(&value),
    )?;
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
    if read_json(
        &a.admission_report
            .parent()
            .ok_or_else(|| invalid("admission parent absent"))?
            .join("frozen-source-parameters.json"),
    )? != frozen
    {
        return Err(invalid("native-route admission frozen parameter receipts differ").into());
    }
    let zero = vec![
        0u8;
        NativeReadFeedback::bridge_coefficient_count(integer.context_config())?
            .div_ceil(2)
    ];
    let feedback =
        NativeReadFeedback::compile(expected.clone(), integer.context_config(), &value, &zero)?;
    let bridge = FeedbackBridgeWeights::from_native(&feedback)?;
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
    let mut incumbent_ce = baseline["mean_equal_episode_ce"]
        .as_f64()
        .ok_or_else(|| invalid("baseline CE absent"))?;
    let mut accepted = 0usize;
    let mut rounds = Vec::new();
    let mut stop_reason = "maximum accepted rounds";
    for round in 0..MAX_ACCEPTED_ROUNDS {
        deadline(start, a)?;
        let begun = Instant::now();
        let round_start_bytes = tree_bytes(&a.out)?;
        let incumbent = bridge.compile()?;
        let measured = batch(
            a,
            start,
            &source,
            &native,
            &bridge,
            &incumbent,
            &cases,
            true,
            &[],
        )?;
        if measured.canonical["mean_equal_episode_ce"].as_f64() != Some(incumbent_ce) {
            return Err(invalid("incumbent credit forward differs from accepted native CE").into());
        }
        write_json(
            &a.out,
            &format!("round-{round:02}-credit.json"),
            &measured.canonical,
        )?;
        let q = shadow_quanta(&bridge)?;
        let proposals = proposals(&q, measured.crossings)?;
        write_json(
            &a.out,
            &format!("round-{round:02}-prospective-proposals.json"),
            &json!({"incumbent_ce":incumbent_ce,"maximum_candidates":CANDIDATES_PER_ROUND,"candidate_count":proposals.len(),"all_positive_native_utility_classes_searched":true,"incumbent_gradient_f64_sha256":sha256_bytes(&measured.gradient.iter().flat_map(|x|x.to_bits().to_le_bytes()).collect::<Vec<_>>()),"root_coordinates":[1,2,3,4,5,6,7,8,9,10,11,12,46,47,48,49],"ranking":"positive conditional probability gain / edit L1 quanta; row,step,lane,action,feature,newq ties","exact_integer_all119_rivals_identity_first":true,"no_bias_category_fallback":true,"frozen_before_candidate_evaluation":true,"proposals":proposals}),
        )?;
        let mut candidate_summaries = Vec::new();
        let mut scores = Vec::new();
        for (index, proposal) in proposals.iter().enumerate() {
            let candidate = FeedbackBridgeWeights::from_native(&incumbent)?;
            set_quanta(&candidate, &proposal.quanta)?;
            let root = a.out.join(format!("round-{round:02}-candidate-{index:02}"));
            report_output::claim(&root)?;
            let result = (|| -> Result<Value> {
                let current = candidate.compile()?;
                // Source-bound proposal receipts retain only the changed bridge;
                // the exact immutable producer is already preserved and bound.
                fs::write(root.join("bridge-q4.bin"), current.bridge_packed())?;
                fs::write(
                    root.join("bridge-shadow-f32le.bin"),
                    proposal
                        .quanta
                        .iter()
                        .flat_map(|q| (f32::from(*q) * 0.25).to_bits().to_le_bytes())
                        .collect::<Vec<_>>(),
                )?;
                write_json(
                    &a.out,
                    &format!("round-{round:02}-candidate-{index:02}/metadata.json"),
                    &serde_json::to_value(current.metadata())?,
                )?;
                let reloaded = NativeReadFeedback::compile(
                    expected.clone(),
                    integer.context_config(),
                    &value,
                    &fs::read(root.join("bridge-q4.bin"))?,
                )?;
                if read_json(&root.join("metadata.json"))?
                    != serde_json::to_value(reloaded.metadata())?
                    || reloaded.metadata() != current.metadata()
                    || reloaded.bridge_packed() != current.bridge_packed()
                    || reloaded.value_packed() != incumbent.value_packed()
                {
                    return Err(invalid("source-bound candidate reload differs").into());
                }
                let restored = FeedbackBridgeWeights::from_native(&reloaded)?;
                if shadow_quanta(&restored)? != proposal.quanta
                    || restored.parent_binding() != bridge.parent_binding()
                {
                    return Err(invalid("candidate raw quarter/parent reload differs").into());
                }
                let witness = verify_witness(a, &native, &reloaded, &restored, &cases, proposal)?;
                write_json(&root, "crossing-witness.json", &witness)?;
                let canonical = factual(a, start, &native, &reloaded, &cases)?;
                write_json(
                    &a.out,
                    &format!("round-{round:02}-candidate-{index:02}/canonical.json"),
                    &canonical,
                )?;
                let old = read_json(
                    &a.out
                        .join(format!("checkpoint-{accepted:04}/canonical.json")),
                )?;
                write_json(
                    &a.out,
                    &format!("round-{round:02}-candidate-{index:02}/comparison-to-incumbent.json"),
                    &compare(&old, &canonical)?,
                )?;
                let summary = json!({"proposal":proposal,"native_equal_episode_ce":canonical["mean_equal_episode_ce"],"zero_support_positions":canonical["zero_support_positions"],"complete_canonical_cases":64,"complete_target_positions":570,"feedback_metadata_sha256":sha256_file(&root.join("metadata.json"))?,"packed_sha256":sha256_file(&root.join("bridge-q4.bin"))?,"shadow_sha256":sha256_file(&root.join("bridge-shadow-f32le.bin"))?,"fixed_producer_sha256":sha256_bytes(&value),"fixed_producer_path":a.fixed_value,"parent_binding":expected,"source_bound_recompile_verified":true,"ownprefix_generation":"NOT_RUN for unaccepted candidate","no_probability_floor":true});
                write_json(
                    &a.out,
                    &format!("round-{round:02}-candidate-{index:02}/summary.json"),
                    &summary,
                )?;
                Ok(summary)
            })();
            if let Err(error) = &result {
                write_json(
                    &root,
                    "failure.json",
                    &json!({"error":error.to_string(),"proposal":proposal}),
                )?;
            }
            report_output::seal(&root)?;
            report_output::verify(&root)?;
            let summary = result?;
            scores.push(summary["native_equal_episode_ce"].as_f64());
            candidate_summaries.push(summary);
        }
        let chosen = strict_best(incumbent_ce, &scores);
        let mut round_report = json!({"round":round,"incumbent_ce":incumbent_ce,"candidate_count":proposals.len(),"candidates":candidate_summaries,"selected_proposal":chosen,"criterion":"strict lowest complete64 equalepisode native alias CE; exact tie retains incumbent, earliest candidate among strictly better ties","source_frozen":true});
        if parameter_receipts(&source.parameters())? != frozen {
            return Err(invalid("candidate evaluation changed frozen source").into());
        }
        if let Some(index) = chosen {
            set_quanta(&bridge, &proposals[index].quanta)?;
            accepted += 1;
            let stage = export(
                a, start, &source, &native, &bridge, &cases, &tok, accepted, &stages,
            )?;
            let candidate_ce = scores[index].ok_or_else(|| invalid("chosen finite CE absent"))?;
            if stage["native_equal_episode_ce"].as_f64() != Some(candidate_ce) {
                return Err(invalid("accepted export fullnative replay differs").into());
            }
            incumbent_ce = candidate_ce;
            stages.push(stage);
            round_report["accepted_updates"] = json!(accepted);
        } else {
            stop_reason = "no strictly improving candidate in frozen finite proposal set";
        }
        round_report["elapsed_seconds"] = json!(begun.elapsed().as_secs_f64());
        write_json(
            &a.out,
            &format!("round-{round:02}-result.json"),
            &round_report,
        )?;
        rounds.push(round_report);
        write_json(
            &a.out,
            "progress.json",
            &json!({"accepted_updates":accepted,"completed_rounds":rounds.len(),"checkpoints":stages,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        if chosen.is_none() {
            break;
        }
        if round == 0 && accepted < MAX_ACCEPTED_ROUNDS {
            let projected = project_remaining_rounds(
                begun.elapsed().as_secs_f64(),
                MAX_ACCEPTED_ROUNDS - 1,
                (4. * baseline_export_seconds + 30.).max(180.),
            )?;
            let remaining = a.maximum_seconds as f64 - start.elapsed().as_secs_f64();
            let retained_bytes = tree_bytes(&a.out)?;
            let projected_bytes = retained_bytes
                .saturating_add(
                    retained_bytes
                        .saturating_sub(round_start_bytes)
                        .saturating_mul(MAX_ACCEPTED_ROUNDS - 1),
                )
                .saturating_add(64 * 1024 * 1024);
            let bytes_admitted = projected_bytes <= REPORT_CAP - 1024 * 1024;
            write_json(
                &a.out,
                "measured-round-admission.json",
                &json!({"first_complete_round_seconds":begun.elapsed().as_secs_f64(),"projected_remaining_seconds":projected,"remaining_declared_seconds":remaining,"before_round2":true,"admitted":projected<=remaining && bytes_admitted,"retained_bytes":retained_bytes,"projected_bytes_including_64MiB_export_transfer_reserve":projected_bytes,"report_cap_bytes":REPORT_CAP}),
            )?;
            if projected > remaining || !bytes_admitted {
                stop_reason = "measured remaining-round resource guard";
                break;
            }
        }
    }
    let best = selected(&stages)?;
    let step = stages[best]["accepted_updates"]
        .as_u64()
        .ok_or_else(|| invalid("selected accepted update absent"))?;
    let chosen_root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::verify(&chosen_root)?;
    let chosen = NativeReadFeedback::load(
        &chosen_root.join("feedback-native"),
        stages[best]["feedback_metadata_sha256"]
            .as_str()
            .ok_or_else(|| invalid("selected SHA absent"))?,
    )?;
    write_json(
        &a.out,
        "selection-before-transfer.json",
        &json!({"selected_accepted_updates":step,"criterion":"lowest actual64 native equalepisode CE includes0; earliest exact ties","fresh_predictions_before_selection":0}),
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
            return Err(invalid("input changed during learning").into());
        }
    }
    if parameter_receipts(&source.parameters())? != frozen {
        return Err(invalid("frozen source changed").into());
    }
    deadline(start, a)?;
    Ok(
        json!({"schema":"uor-r4.geometric-native-route-crossing/1","status":"completed","feedback_input":a.feedback_input,"accepted_updates":accepted,"maximum_accepted_rounds":MAX_ACCEPTED_ROUNDS,"completed_rounds":rounds.len(),"maximum_candidates_per_round":CANDIDATES_PER_ROUND,"stop_reason":stop_reason,"proposal_policy":"minimal legal singleton desired-class root crossings; all positive conditional utilities; exact integer native margins; top16 gain per edit quantum","conditional_native_utility_is_proposal_only":true,"gradient_diagnostic_not_proposal_direction":true,"no_Adam_or_fractional_accumulation":true,"no_utility_cache":true,"native_utilities_recomputed_only_incumbent":true,"candidate_native_objective":"complete64 equalepisode mean completeanswerEOS token alias CE; zero support nonselectable","checkpoints":stages,"selected_accepted_updates":step,"generation_separate_from_selection":true,"fresh_transfer_manifest_sha256":a.transfer_panel_manifest_sha256,"fresh_preparation_seed":a.transfer_preparation_seed,"required_excluded_native_panel":EXPOSED_NATIVE_PANEL,"parent_transfer_complete":parent_transfer["accepted_complete"],"selected_transfer_complete":selected_transfer["accepted_complete"],"frozen_source_context_readouts":true,"fixed_producer_sha256":sha256_bytes(&value),"input_files_sha256":inputs,"input_files_unchanged":true,"trusted_native_binding":expected,"peak_rss_kib_linux":peak_rss_kib(),"elapsed_seconds":start.elapsed().as_secs_f64(),"no_adopted_model":true,"scope":"bounded existing83 bridge native acceptance learning; no new runtime/features; finite proposal negative not family incapacity; no geometricchat/prose/reasoning/energy qualification"}),
    )
}

#[derive(Clone, serde::Serialize)]
struct Crossing {
    row: usize,
    step: usize,
    id: String,
    target_label_only: u32,
    lane: usize,
    requested_action: u8,
    feature: usize,
    coefficient: usize,
    old_q: i8,
    new_q: i8,
    root: u8,
    conditional_probability_gain: f64,
    gain_per_quantum: f64,
    score_before_q24: i64,
    score_after_q24: i64,
    rival_max_q24: i64,
    minimum_winning_score_q24: i64,
    before: FeedbackTrace,
    stage1_sha256: String,
}
#[derive(Default, serde::Serialize)]
struct CrossingCoverage {
    lane_positions: usize,
    positive_class_alternatives: usize,
    active_root_coordinate_searches: usize,
    legal_replacements_examined: usize,
    reachable_coordinate_crossings: usize,
    retained_deduplicated_proposals: usize,
}
#[derive(Clone, serde::Serialize)]
struct Proposal {
    id: String,
    quanta: Vec<i8>,
    changed: Vec<(usize, i8, i8)>,
    witness: Crossing,
}
fn priority(action: usize) -> usize {
    if action == 1 {
        0
    } else if action == 0 {
        1
    } else {
        action
    }
}
fn winner(scores: &[i64]) -> Result<usize> {
    if scores.len() != 120 {
        return Err(invalid("120 exact scores required").into());
    }
    let mut best = 1;
    for action in (0..120).filter(|a| *a != 1) {
        if scores[action] > scores[best] {
            best = action;
        }
    }
    Ok(best)
}
fn winning_threshold(scores: &[i64], desired: usize) -> Result<(i64, i64)> {
    if scores.len() != 120 || desired >= 120 {
        return Err(invalid("exact rival shape differs").into());
    }
    let mut threshold = i64::MIN;
    let mut rival_max = i64::MIN;
    for (action, &score) in scores.iter().enumerate().filter(|(a, _)| *a != desired) {
        rival_max = rival_max.max(score);
        let required = score
            .checked_add(i64::from(priority(action) < priority(desired)))
            .ok_or_else(|| invalid("score threshold overflow"))?;
        threshold = threshold.max(required);
    }
    Ok((threshold, rival_max))
}
fn crossing_order(a: &Crossing, b: &Crossing) -> std::cmp::Ordering {
    b.gain_per_quantum
        .total_cmp(&a.gain_per_quantum)
        .then_with(|| {
            (
                a.row,
                a.step,
                a.lane,
                a.requested_action,
                a.feature,
                a.new_q,
            )
                .cmp(&(
                    b.row,
                    b.step,
                    b.lane,
                    b.requested_action,
                    b.feature,
                    b.new_q,
                ))
        })
}
fn minimal_replacement(
    q: &[i8],
    axis: usize,
    coordinates: &[i32; 4],
    current_score: i64,
    threshold: i64,
) -> Result<Option<(i8, i64, usize)>> {
    if q.len() != 4 || axis >= 4 || q.iter().any(|q| !(-7..=7).contains(q)) {
        return Err(invalid("legal root coefficient shape differs").into());
    }
    let old_factor = FeedbackBridgeWeights::exact_root_factor_q24(q, coordinates)?;
    let mut examined = 0;
    for distance in 1i16..=14 {
        // Ascending signed q is the deterministic tie among equal L1 distances.
        for new in [i16::from(q[axis]) - distance, i16::from(q[axis]) + distance] {
            if !(-7..=7).contains(&new) {
                continue;
            }
            examined += 1;
            let mut replacement = [q[0], q[1], q[2], q[3]];
            replacement[axis] = new as i8;
            let score = current_score - old_factor
                + FeedbackBridgeWeights::exact_root_factor_q24(&replacement, coordinates)?;
            if score >= threshold {
                return Ok(Some((new as i8, score, examined)));
            }
        }
    }
    Ok(None)
}
fn discover_crossings(
    q: &[i8],
    bridge: &FeedbackBridgeWeights,
    trace: &FeedbackTrace,
    utilities: &[NativeRouteLaneUtility],
    row: usize,
    step: usize,
    id: &str,
    target: u32,
    stage1_sha256: &str,
    pool: &mut Vec<Crossing>,
    coverage: &mut CrossingCoverage,
) -> Result<()> {
    let scores = bridge.exact_policy_scores_q24(trace)?;
    if q.len() != 79680 || scores.len() != 8 || utilities.len() != 8 {
        return Err(invalid("crossing parent layout differs").into());
    }
    let basis = canonical_basis_q25();
    for lane in 0..8 {
        coverage.lane_positions += 1;
        let utility = &utilities[lane];
        if utility.lane != lane
            || utility.target_probabilities.len() != 120
            || winner(&scores[lane])? != trace.actions[lane] as usize
            || utility.factual_action != trace.actions[lane]
        {
            return Err(invalid("crossing factual action differs").into());
        }
        let next = lane / 4 * 4 + (lane % 4 + 1) % 4;
        let atoms = &trace.value.packets[lane];
        let factual = utility.target_probabilities[utility.factual_action as usize];
        for desired in 0..120 {
            let gain = utility.target_probabilities[desired] - factual;
            if !gain.is_finite() || gain <= 0. || desired == utility.factual_action as usize {
                continue;
            }
            coverage.positive_class_alternatives += 1;
            let (threshold, rival_max) = winning_threshold(&scores[lane], desired)?;
            for (base, root) in [
                (1, Some(trace.before.states[lane])),
                (5, Some(trace.before.states[next])),
                (
                    9,
                    if atoms[0].state == "PresentNonzero" {
                        Some(atoms[0].root)
                    } else {
                        None
                    },
                ),
                (
                    46,
                    if atoms[1].state == "PresentNonzero" {
                        Some(atoms[1].root)
                    } else {
                        None
                    },
                ),
            ] {
                let Some(root) = root else { continue };
                if root >= 120 {
                    return Err(invalid("root code differs").into());
                }
                let at = (lane * 120 + desired) * 83 + base;
                for axis in 0..4 {
                    coverage.active_root_coordinate_searches += 1;
                    let found = minimal_replacement(
                        &q[at..at + 4],
                        axis,
                        &basis[root as usize],
                        scores[lane][desired],
                        threshold,
                    )?;
                    let Some((new, after, examined)) = found else {
                        coverage.legal_replacements_examined += 14;
                        continue;
                    };
                    coverage.legal_replacements_examined += examined;
                    coverage.reachable_coordinate_crossings += 1;
                    let index = at + axis;
                    let ratio = gain / f64::from((i16::from(new) - i16::from(q[index])).abs());
                    // Compare rank before cloning the complete retained native witness.
                    let key = (row, step, lane, desired as u8, base + axis, new);
                    if pool.len() == CANDIDATES_PER_ROUND
                        && pool.last().is_some_and(|last| {
                            ratio < last.gain_per_quantum
                                || (ratio == last.gain_per_quantum
                                    && key
                                        >= (
                                            last.row,
                                            last.step,
                                            last.lane,
                                            last.requested_action,
                                            last.feature,
                                            last.new_q,
                                        ))
                        })
                    {
                        continue;
                    }
                    let crossing = Crossing {
                        row,
                        step,
                        id: id.into(),
                        target_label_only: target,
                        lane,
                        requested_action: desired as u8,
                        feature: base + axis,
                        coefficient: index,
                        old_q: q[index],
                        new_q: new,
                        root,
                        conditional_probability_gain: gain,
                        gain_per_quantum: ratio,
                        score_before_q24: scores[lane][desired],
                        score_after_q24: after,
                        rival_max_q24: rival_max,
                        minimum_winning_score_q24: threshold,
                        before: trace.clone(),
                        stage1_sha256: stage1_sha256.into(),
                    };
                    if let Some(i) = pool
                        .iter()
                        .position(|x| x.coefficient == index && x.new_q == new)
                    {
                        if crossing_order(&crossing, &pool[i]).is_ge() {
                            continue;
                        }
                        pool.remove(i);
                    }
                    pool.push(crossing);
                    pool.sort_by(crossing_order);
                    pool.truncate(CANDIDATES_PER_ROUND);
                }
            }
        }
    }
    coverage.retained_deduplicated_proposals = pool.len();
    Ok(())
}
fn apply_root_singleton(q: &[i8], index: usize, new: i8) -> Result<Vec<i8>> {
    if q.len() != 79680
        || index >= q.len()
        || q.iter().any(|q| !(-7..=7).contains(q))
        || !(-7..=7).contains(&new)
        || new == q[index]
        || !matches!(index%83,1..=12|46..=49)
    {
        return Err(invalid("legal root-only singleton required").into());
    }
    let mut candidate = q.to_vec();
    candidate[index] = new;
    Ok(candidate)
}
fn proposals(q: &[i8], crossings: Vec<Crossing>) -> Result<Vec<Proposal>> {
    if q.len() != 79680
        || q.iter().any(|q| !(-7..=7).contains(q))
        || crossings.len() > CANDIDATES_PER_ROUND
    {
        return Err(invalid("legal crossing proposal shape differs").into());
    }
    crossings
        .into_iter()
        .enumerate()
        .map(|(i, witness)| {
            if witness.lane >= 8
                || witness.requested_action >= 120
                || witness.feature >= 83
                || witness.coefficient
                    != (witness.lane * 120 + witness.requested_action as usize) * 83
                        + witness.feature
                || !witness.gain_per_quantum.is_finite()
                || witness.gain_per_quantum <= 0.
                || q[witness.coefficient] != witness.old_q
                || !(-7..=7).contains(&witness.new_q)
                || witness.old_q == witness.new_q
                || !matches!(witness.feature,1..=12|46..=49)
            {
                return Err(invalid("root-only singleton binding differs").into());
            }
            let candidate = apply_root_singleton(q, witness.coefficient, witness.new_q)?;
            Ok(Proposal {
                id: format!("root-crossing-{i:02}"),
                quanta: candidate,
                changed: vec![(witness.coefficient, witness.old_q, witness.new_q)],
                witness,
            })
        })
        .collect()
}
fn verify_witness(
    a: &Args,
    native: &NativeSourceRealizer,
    compiled: &NativeReadFeedback,
    bridge: &FeedbackBridgeWeights,
    cases: &Cases,
    proposal: &Proposal,
) -> Result<Value> {
    let w = &proposal.witness;
    let s = &cases.inputs[w.row];
    let prefix = &cases.targets[w.row][..w.step];
    if s.id != w.id || cases.targets[w.row][w.step] != w.target_label_only {
        return Err(invalid("witness row binding differs").into());
    }
    let actual = native.read_dependent(
        s.frame(),
        &cases.views[w.row],
        &s.query_ids,
        prefix,
        compiled,
        mode(a),
    )?;
    if sha256_bytes(&serde_json::to_vec(&actual.stage1)?) != w.stage1_sha256
        || actual.feedback.before != w.before.before
        || actual.feedback.value != w.before.value
        || actual.feedback.selected_occurrence != w.before.selected_occurrence
        || actual.feedback.actions[w.lane] != w.requested_action
        || actual
            .feedback
            .actions
            .iter()
            .enumerate()
            .any(|(l, x)| l != w.lane && *x != w.before.actions[l])
    {
        return Err(invalid("exported native requested singleton crossing not observed").into());
    }
    let scores = bridge.exact_policy_scores_q24(&actual.feedback)?;
    if winner(&scores[w.lane])? != w.requested_action as usize
        || scores[w.lane][w.requested_action as usize] != w.score_after_q24
    {
        return Err(invalid("exact predicted/native crossing score differs").into());
    }
    let (_, rival) = winning_threshold(&scores[w.lane], w.requested_action as usize)?;
    Ok(
        json!({"proposal_id":proposal.id,"source_input":s,"actual_prefix_ids":prefix,
        "target_label_only_after_read":w.target_label_only,"requested_action":w.requested_action,
        "actual_action":actual.feedback.actions[w.lane],"score_margin_before_q24":w.score_before_q24-w.rival_max_q24,
        "score_margin_after_q24":scores[w.lane][w.requested_action as usize]-rival,
        "feature_geometry":w.before,"exact_all120_scores_after":scores[w.lane],
        "native_trace":actual,"crossing_verified_before_full_candidate_evaluation":true}),
    )
}
fn set_quanta(bridge: &FeedbackBridgeWeights, q: &[i8]) -> Result<()> {
    let vars = bridge.parameters();
    let v = vars
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("bridge parameter absent"))?;
    if q.len() != v.elem_count() || q.iter().any(|q| !(-7..=7).contains(q)) {
        return Err(invalid("illegal proposal coefficients").into());
    }
    v.set(&Tensor::from_vec(
        q.iter().map(|q| f32::from(*q) * 0.25).collect::<Vec<_>>(),
        v.shape(),
        &Device::Cpu,
    )?)?;
    Ok(())
}
fn shadow_quanta(bridge: &FeedbackBridgeWeights) -> Result<Vec<i8>> {
    let vars = bridge.parameters();
    let v = vars
        .get("bridge.coefficients")
        .ok_or_else(|| invalid("bridge absent"))?;
    let raw = v.flatten_all()?.to_vec1::<f32>()?;
    if raw
        .iter()
        .any(|x| !x.is_finite() || !(-1.75..=1.75).contains(x) || (*x * 4.).round() != *x * 4.)
    {
        return Err(invalid("incumbent must have exact legal quarter shadows").into());
    }
    Ok(raw.into_iter().map(|x| (x * 4.) as i8).collect())
}
fn strict_best(incumbent: f64, scores: &[Option<f64>]) -> Option<usize> {
    let mut best = incumbent;
    let mut chosen = None;
    for (i, s) in scores.iter().enumerate() {
        if let Some(s) = s.filter(|x| x.is_finite()) {
            if s < best {
                best = s;
                chosen = Some(i);
            }
        }
    }
    chosen
}
fn project_remaining_rounds(first: f64, remaining: usize, reserve: f64) -> Result<f64> {
    if !first.is_finite()
        || first <= 0.
        || remaining > MAX_ACCEPTED_ROUNDS - 1
        || !reserve.is_finite()
        || reserve < 180.
    {
        return Err(invalid("measured fullround projection invalid").into());
    }
    Ok(first * remaining as f64 * 1.25 + reserve)
}
fn complete_panel_mean(episodes: &[Option<f64>]) -> Result<Option<f64>> {
    if episodes.len() != 64 || episodes.iter().flatten().any(|x| !x.is_finite() || *x < 0.) {
        return Err(invalid("complete64 finite episode means required").into());
    }
    if episodes.iter().any(Option::is_none) {
        return Ok(None);
    }
    Ok(Some(episodes.iter().flatten().map(|x| *x / 64.).sum()))
}
fn factual(
    a: &Args,
    start: Instant,
    native: &NativeSourceRealizer,
    feedback: &NativeReadFeedback,
    cases: &Cases,
) -> Result<Value> {
    let begun = Instant::now();
    let mut rows = Vec::new();
    let mut episode_means = Vec::new();
    let mut zeros = Vec::new();
    if cases.inputs.len() != 64 || cases.targets.len() != 64 {
        return Err(invalid("full64 factual cases required").into());
    }
    for (row, s) in cases.inputs.iter().enumerate() {
        let mut tokens = Vec::new();
        let mut sum = 0.;
        let mut finite = true;
        for (step, &target) in cases.targets[row].iter().enumerate() {
            deadline(start, a)?;
            let prefix = &cases.targets[row][..step];
            let t = native.read_dependent(
                s.frame(),
                &cases.views[row],
                &s.query_ids,
                prefix,
                feedback,
                mode(a),
            )?;
            let total = t.stage2.actions.total_weight_q31;
            if total == 0 {
                return Err(invalid("native action denominator zero").into());
            }
            let mass = t
                .stage2
                .actions
                .token_masses
                .iter()
                .filter(|m| m.token_id == target)
                .map(|m| m.weight_q31)
                .sum::<u64>();
            let ce = if mass == 0 {
                finite = false;
                zeros.push(
                    json!({"id":s.id,"step":step,"target_label_only":target,"prefix_ids":prefix}),
                );
                None
            } else {
                let ce = -(mass as f64 / total as f64).ln();
                if !ce.is_finite() {
                    return Err(invalid("native factual CE nonfinite").into());
                }
                sum += ce;
                Some(ce)
            };
            tokens.push(json!({"step":step,"teacherforced_prefix_ids":prefix,"target_label_only":target,"target_mass_q31":mass,"total_weight_q31":total,"native_ce":ce,"zero_support":mass==0,"actions":t.stage2.actions,"feedback_actions":t.feedback.actions,"selected_occurrence":t.feedback.selected_occurrence,"producer_packets":t.feedback.value.packets,"refined_states":t.stage2_controller_snapshot.states,"query_codes":t.stage2_controller_snapshot.codes}));
        }
        let ce = finite.then_some(sum / cases.targets[row].len() as f64);
        episode_means.push(ce);
        rows.push(
            json!({"id":s.id,"native_mean_token_ce":ce,"zero_support":!finite,"tokens":tokens}),
        );
    }
    if rows
        .iter()
        .map(|r| r["tokens"].as_array().map_or(0, Vec::len))
        .sum::<usize>()
        != 570
    {
        return Err(invalid("full570 factual coverage required").into());
    }
    Ok(
        json!({"cases":64,"target_positions":570,"mean_equal_episode_ce":complete_panel_mean(&episode_means)?,"zero_support_positions":zeros,"rows":rows,"backward":false,"native_utilities_evaluated":0,"elapsed_seconds":begun.elapsed().as_secs_f64(),"infinite_NLL_when_any_target_support_zero":true,"probability_floor":false}),
    )
}

fn main() -> Result<()> {
    let a = checked_args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let work = (|| -> Result<Value> {
        let head = option_env!("UOR_BUILD_SOURCE_COMMIT")
            .ok_or_else(|| invalid("source commit absent"))?;
        if head.len() != 40 || !head.bytes().all(|x| x.is_ascii_hexdigit()) {
            return Err(invalid("invalid source commit").into());
        }
        let (exe, lookup) = executable()?;
        let mut report = run(&a, start)?;
        report["source_commit"] = json!(head);
        report["executable_sha256"] = json!(sha256_file(&exe)?);
        report["executable_lookup"] = json!(lookup);
        Ok(report)
    })();
    match &work {
        Ok(r) => write_json(&a.out, "report.json", r)?,
        Err(e) => {
            let progress = read_json(&a.out.join("progress.json")).unwrap_or(Value::Null);
            fs::write(
                a.out.join("failure.json"),
                serde_json::to_vec(
                    &json!({"schema":"uor-r4.geometric-native-route-crossing/1","status":"failed_or_stopped","error":e.to_string(),"last_progress":progress,"elapsed_seconds":start.elapsed().as_secs_f64(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
                )?,
            )?;
        }
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    work.map(|_| ())
}
#[cfg(test)]
mod crossing_tests {
    use super::*;
    #[test]
    fn exact_root_rounding_identity_ties_and_minimal_crossings() -> Result<()> {
        assert_eq!(
            FeedbackBridgeWeights::exact_root_factor_q24(&[1, 0, 0, 0], &[4, 0, 0, 0])?,
            1
        );
        assert_eq!(
            FeedbackBridgeWeights::exact_root_factor_q24(&[-1, 0, 0, 0], &[4, 0, 0, 0])?,
            -1
        );
        assert_eq!(
            FeedbackBridgeWeights::exact_root_factor_q24(&[1, -1, 0, 0], &[4, 4, 0, 0])?,
            0
        );
        let mut scores = vec![0; 120];
        assert_eq!(winner(&scores)?, 1);
        assert_eq!(winning_threshold(&scores, 0)?.0, 1);
        assert_eq!(winning_threshold(&scores, 1)?.0, 0);
        scores[119] = 7;
        assert_eq!(winning_threshold(&scores, 0)?.0, 7);
        assert_eq!(
            minimal_replacement(&[0, 0, 0, 0], 0, &[33554432, 0, 0, 0], 0, 1)?.map(|x| x.0),
            Some(1)
        );
        assert_eq!(
            minimal_replacement(
                &[7, 0, 0, 0],
                0,
                &[33554432, 0, 0, 0],
                7 * 4194304,
                8 * 4194304
            )?,
            None
        );
        assert_eq!(
            minimal_replacement(&[0, 0, 0, 0], 0, &[0, 33554432, 0, 0], 0, 1)?,
            None
        );
        assert!(
            FeedbackBridgeWeights::exact_root_factor_q24(&[-8, 0, 0, 0], &[4, 0, 0, 0]).is_err()
        );
        Ok(())
    }
    #[test]
    fn root_singleton_cannot_edit_bias_categories_or_other_coordinates() -> Result<()> {
        let q = vec![0; 79680];
        for index in [1, 5, 12, 46, 49, 79680 - 83 + 49] {
            let changed = apply_root_singleton(&q, index, -7)?;
            assert_eq!(
                changed
                    .iter()
                    .zip(&q)
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>(),
                vec![index]
            );
        }
        for index in [0, 13, 45, 50, 82, 79680] {
            assert!(apply_root_singleton(&q, index, 1).is_err());
        }
        assert!(apply_root_singleton(&q, 1, -8).is_err());
        assert!(apply_root_singleton(&q, 1, 0).is_err());
        Ok(())
    }
    #[test]
    fn strict_native_selection_and_round_budget_preserve_incumbent() -> Result<()> {
        assert_eq!(strict_best(0.5, &[Some(0.5), Some(f64::NAN), None]), None);
        assert_eq!(
            strict_best(0.5, &[Some(0.4), Some(0.4), Some(0.3)]),
            Some(2)
        );
        assert_eq!(strict_best(0.5, &[Some(0.4), Some(0.4)]), Some(0));
        assert_eq!(project_remaining_rounds(10., 3, 180.)?, 217.5);
        assert!(project_remaining_rounds(10., 4, 180.).is_err());
        Ok(())
    }
    #[test]
    fn complete_equalepisode_objective_preserves_zero_support_rejection() -> Result<()> {
        let episodes = (0..64)
            .map(|i| Some(if i < 32 { 0.2 } else { (0.4 + 0.4 + 0.4) / 3. }))
            .collect::<Vec<_>>();
        assert!(
            (complete_panel_mean(&episodes)?.ok_or_else(|| invalid("finite mean absent"))? - 0.3)
                .abs()
                < 1e-12
        );
        let mut unsupported = episodes;
        unsupported[63] = None;
        assert_eq!(complete_panel_mean(&unsupported)?, None);
        assert!(complete_panel_mean(&unsupported[..63]).is_err());
        unsupported[63] = Some(f64::INFINITY);
        assert!(complete_panel_mean(&unsupported).is_err());
        Ok(())
    }
}
