//! Separately released, bounded Relation/Flock learning. This module handles
//! admission, storage and receipt metadata; all model work remains in CUDA.
//! Owner records are operational safeguards, not cryptographic authentication.
//! No lab may create a release record or inherit the earlier pilot delegation.
use super::*;
use crate::ReadInitialization;
#[path = "attention_evidence.rs"]
pub mod evidence;
#[path = "attention_preflight.rs"]
pub mod preflight;
#[path = "attention_transfer.rs"]
pub mod transfer;
#[cfg(any(feature = "cuda", test))]
use crate::{metadata::BinderPlan, training::CoordinateKind};

pub const ATTENTION_BATCHES: u64 = 16_404;
pub const ATTENTION_BASE_SOURCE: &str = "ef9d30999700be11e103f96590e9e3b83b6807c7";
/// Canonical Unit2b and paired-fresh clarification through PR2583/mainc7b2b1cb.
/// Scope and the selected
/// arm/seed remain independently pinned by the review payload and owner plan.
pub const ATTENTION_SHEET_SHA256: &str =
    "844814009ecc7368c0a3ecabf62434bae7d72850363e19f67c0126810fee7ffc";
const PLAN_SCHEMA: &str = "uor-r4.attention-learning-owner-plan/2";
#[cfg(feature = "cuda")]
const ADMISSION_SCHEMA: &str = "uor-r4.attention-learning-owner-admission/2";
#[cfg(any(feature = "cuda", test))]
const OWNER_DECISION: &str = "RELEASE_D4_FOR_THIS_ATTENTION_LEARNING_ARM_SEED_ONLY";
#[cfg(feature = "cuda")]
const PROGRESS_SCHEMA: &str = "uor-r4.attention-learning-checkpoint/2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionCostContract {
    pub max_batches: u64,
    pub checkpoint_every_batches: u64,
    pub baseline_dev_evaluations: u32,
    pub q1_dev_evaluations: u32,
    pub final_dev_evaluations: u32,
    pub relation_off_dev_evaluations: u32,
    pub final_transfer_evaluations: u32,
    pub relation_off_transfer_evaluations: u32,
}
impl AttentionCostContract {
    fn expected(config: &DriverConfig) -> Self {
        let ablation = matches!(config.read_spec, ReadSpec::Relation { .. })
            && config.relation_learning_policy == training::RelationLearningPolicy::LearnV1;
        Self {
            max_batches: ATTENTION_BATCHES,
            checkpoint_every_batches: 64,
            baseline_dev_evaluations: 1,
            q1_dev_evaluations: 1,
            final_dev_evaluations: 1,
            relation_off_dev_evaluations: u32::from(ablation),
            final_transfer_evaluations: u32::from(matches!(
                config.read_spec,
                ReadSpec::Relation { .. }
            )),
            relation_off_transfer_evaluations: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionPlan {
    pub schema: String,
    pub status: String,
    pub scope: String,
    pub execution: ExecutionIdentity,
    pub config: DriverConfig,
    pub sheet: FilePin,
    pub independent_review: FilePin,
    /// Prospective evaluation-only source with at least 2000 eligible rows; its
    /// labels cannot choose a checkpoint. The PM packet must separately establish
    /// M4's >=500 successor rows and held-out provenance before execution.
    pub transfer: corpus::CorpusSource,
    pub run_state_directory: PathBuf,
    pub cost_contract: AttentionCostContract,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionAdmission {
    pub schema: String,
    pub plan: AttentionPlan,
    pub owner_release: FilePin,
}
#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttentionOwner {
    owner: String,
    decision: String,
    scope: String,
    plan_sha256: String,
    receipt_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttentionReview {
    reviewer: String,
    verdict: String,
    scope: String,
    execution: ExecutionIdentity,
    reviewed_plan_payload_sha256: String,
    receipt_url: String,
}
/// The review pins every plan field except its own FilePin, avoiding a hash cycle.
pub fn review_payload_sha256(plan: &AttentionPlan) -> Result<String> {
    let mut value = serde_json::to_value(plan).map_err(|e| invalid(e.to_string()))?;
    value
        .as_object_mut()
        .ok_or_else(|| invalid("attention plan is not an object"))?
        .remove("independent_review");
    Ok(corpus::sha256(&encode(&value)?))
}

pub fn scope(config: &DriverConfig) -> Result<String> {
    let arm = match config.read_spec {
        ReadSpec::Flock { kept: 8 } => "flock",
        ReadSpec::Relation { kept: 8, sweeps: 1 } => {
            if config.relation_learning_policy == training::RelationLearningPolicy::FrozenZeroV1 {
                "relation-frozen-zero"
            } else {
                "relation"
            }
        }
        _ => return Err(invalid("attention admits only Flock8 or Relation8/sweep1")),
    };
    if !matches!(config.seed, 1 | 2) {
        return Err(invalid(
            "attention requires a separate seed-1 or seed-2 release",
        ));
    }
    // A pinned initial codebook is a distinct lineage, never the seeded-init scope.
    let lineage = if config.codebook_initialization.is_seeded_device() {
        ""
    } else {
        "injective-"
    };
    Ok(format!(
        "m1-attention-learning-{lineage}{arm}-k8-seed{}-batches0..16404",
        config.seed
    ))
}
pub fn validate_config(config: &DriverConfig) -> Result<()> {
    config.validate()?;
    validate_registered_corpus(config)?;
    scope(config)?;
    if config.spec.kept_cells != 8
        || config.objective_policy != crate::ObjectivePolicy::NearestAddressMarginV1
        || config.schedule != training::SchedulePolicy::InterleavedReadEvery8V1
        || config.read_initialization != ReadInitialization::ActiveLeafV1
    {
        return Err(invalid("attention requires K8, nearest-address objective, active-leaf initialization and interleaved read-every-8 schedule"));
    }
    Ok(())
}
fn validate_plan_header(plan: &AttentionPlan) -> Result<()> {
    validate_config(&plan.config)?;
    validate_transfer_source(plan)?;
    let e = &plan.execution;
    for hash in [
        &e.source_sha256,
        &e.kernel_sha256,
        &e.geometry_sha256,
        &e.executable_sha256,
        &e.driver_source_sha256,
        &e.config_sha256,
        &e.lineage_sha256,
        &e.sheet_sha256,
    ] {
        corpus::validate_sha256(hash)?;
    }
    if plan.schema != PLAN_SCHEMA
        || plan.status != "HELD_OWNER_ACTION_REQUIRED"
        || plan.scope != scope(&plan.config)?
        || plan.cost_contract != AttentionCostContract::expected(&plan.config)
        || e.base_source != ATTENTION_BASE_SOURCE
        || e.reviewed_source_commit.len() != 40
        || !e
            .reviewed_source_commit
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || e.driver_source_sha256 != training::driver_source_sha256()
        || e.config_sha256 != corpus::sha256(&encode(&plan.config)?)
        || e.sheet_sha256 != plan.sheet.sha256
        || plan.sheet.sha256 != ATTENTION_SHEET_SHA256
        || !plan.run_state_directory.is_absolute()
    {
        return Err(invalid(
            "attention plan schema/scope/source/config/cost contract mismatch",
        ));
    }
    Ok(())
}
fn validate_transfer_source(plan: &AttentionPlan) -> Result<()> {
    let transfer = &plan.transfer;
    corpus::validate_sha256(&transfer.tokens.sha256)?;
    corpus::validate_sha256(&transfer.mask.sha256)?;
    if transfer.split != corpus::Split::Dev
        || transfer.expected_eligible_targets < 2000
        || transfer.expected_eligible_targets > transfer.expected_payload_tokens
        || [&plan.config.train, &plan.config.dev]
            .into_iter()
            .any(|source| {
                transfer.tokens.sha256 == source.tokens.sha256
                    || transfer.mask.sha256 == source.mask.sha256
            })
    {
        return Err(invalid("attention transfer must be separately pinned evaluation content and mask with at least 2000 eligible rows"));
    }
    Ok(())
}
fn validate_review(plan: &AttentionPlan, review: &AttentionReview) -> Result<()> {
    if !matches!(
        review.reviewer.as_str(),
        "AGY" | "Antigravity" | "antigravity" | "Claude" | "claude" | "DeepSeek" | "deepseek"
    ) || review.verdict != "APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF"
        || review.scope != plan.scope
        || review.execution != plan.execution
        || review.reviewed_plan_payload_sha256 != review_payload_sha256(plan)?
        || !valid_receipt_url(&review.receipt_url)
    {
        return Err(invalid(
            "attention requires an independent non-author exact-source scoped review",
        ));
    }
    Ok(())
}
fn validate_plan(plan: &AttentionPlan) -> Result<()> {
    validate_plan_header(plan)?;
    let bytes = plan.sheet.read()?;
    let sheet = std::str::from_utf8(&bytes).map_err(|e| invalid(e.to_string()))?;
    if !sheet
        .lines()
        .any(|line| line.starts_with("TEST FITNESS: FIT"))
        || !sheet.starts_with("# Pre-registration — M1 learned-attention link")
    {
        return Err(invalid(
            "attention requires the canonical M1 learned-attention FIT sheet",
        ));
    }
    let review: AttentionReview = serde_json::from_slice(&plan.independent_review.read()?)
        .map_err(|e| invalid(e.to_string()))?;
    validate_review(plan, &review)
}
fn validate_execution(
    plan: &AttentionPlan,
    driver: &PreparedDriver,
    executable: &str,
) -> Result<()> {
    if plan.config != *driver.config()
        || plan.execution.lineage_sha256 != driver.lineage_sha256()?
        || plan.execution.executable_sha256 != executable
        || driver.schedule_cycle() != ATTENTION_BATCHES as u32
    {
        return Err(invalid(
            "attention executable/lineage/schedule identity mismatch",
        ));
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn validate_compiled(plan: &AttentionPlan, compiled: &Value) -> Result<()> {
    for (key, expected) in [
        ("source_sha256", &plan.execution.source_sha256),
        ("kernel_sha256", &plan.execution.kernel_sha256),
        ("geometry_sha256", &plan.execution.geometry_sha256),
    ] {
        if compiled[key].as_str() != Some(expected.as_str()) {
            return Err(invalid(format!("attention compiled {key} mismatch")));
        }
    }
    Ok(())
}
fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
    Ok(corpus::sha256(
        &fs::read(path).map_err(|e| invalid(e.to_string()))?,
    ))
}
/// Reads pinned metadata/data and the executable; creates no CUDA context or release.
pub fn check_plan(plan_path: &Path) -> Result<Value> {
    let plan: AttentionPlan = read_json(plan_path)?;
    validate_plan(&plan)?;
    let driver = PreparedDriver::load(plan.config.clone())?;
    validate_execution(&plan, &driver, &executable_sha256()?)?;
    #[cfg(feature = "cuda")]
    validate_compiled(&plan, &crate::device::compiled_identity())?;
    Ok(json!({"status":"PLAN_METADATA_VALID","scope":plan.scope,
        "plan_sha256":corpus::sha256(&encode(&plan)?),"cost_contract":plan.cost_contract,
        "compiled_identity_checked":cfg!(feature="cuda"),"cuda_context_created":false,
        "transfer_source_seal":plan.transfer,"fresh_content_read":false,
        "owner_release":"STILL_REQUIRED","training":"HELD",
        "evaluation_scope":"baseline/Q1/final DEV; Relation ablation DEV; separate paired fresh command only after sealed DEV H_geom decision",
        "learned_attention_result":"NOT_YET_SCORED"}))
}
#[cfg(any(feature = "cuda", test))]
fn validate_owner(plan: &AttentionPlan, pin: &FilePin, owner: &AttentionOwner) -> Result<()> {
    if owner.owner != "Casey"
        || owner.decision != OWNER_DECISION
        || owner.scope != plan.scope
        || owner.plan_sha256 != corpus::sha256(&encode(plan)?)
        || owner.receipt_url != format!("owner-terminal:{}", pin.path.display())
    {
        return Err(Error::TrainingHeld);
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn require_owner_terminal(lab_present: bool, stdin_tty: bool, stdout_tty: bool) -> Result<()> {
    if lab_present || !stdin_tty || !stdout_tty {
        return Err(invalid("attention owner release is Casey-personal at a live terminal; every UOR_LAB value and all pilot delegations are refused"));
    }
    Ok(())
}
/// Casey invokes this himself. Implementing this entry point grants no agent permission to invoke it.
#[cfg(not(feature = "cuda"))]
pub fn owner_release(_plan_path: &Path, _output: &Path) -> Result<()> {
    Err(invalid("attention owner release requires the exact CUDA executable identity; no release was written"))
}
#[cfg(feature = "cuda")]
pub fn owner_release(plan_path: &Path, output: &Path) -> Result<()> {
    use std::io::IsTerminal;
    require_owner_terminal(
        std::env::var_os("UOR_LAB").is_some(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    )?;
    let checked = check_plan(plan_path)?;
    let plan: AttentionPlan = read_json(plan_path)?;
    if checked["plan_sha256"] != json!(corpus::sha256(&encode(&plan)?)) {
        return Err(invalid("attention plan changed after metadata preflight"));
    }
    validate_plan(&plan)?;
    require_disjoint_directories(output, &plan.run_state_directory)?;
    if output.exists() || plan.run_state_directory.exists() {
        return Err(invalid(
            "attention owner output and run-state directories must be fresh",
        ));
    }
    let phrase = format!("I am Casey; release D4 for {} only through batch 16404 with the pinned DEV transfer and relation-off evaluations", plan.scope);
    println!("{}\nType exactly: {phrase}\nAll other training scopes, M2 and Stage B remain HELD; Q4 is NOT_EXECUTABLE.",
        serde_json::to_string_pretty(&plan).map_err(|e| invalid(e.to_string()))?);
    std::io::stdout()
        .flush()
        .map_err(|e| invalid(e.to_string()))?;
    let mut typed = String::new();
    std::io::stdin()
        .read_line(&mut typed)
        .map_err(|e| invalid(e.to_string()))?;
    if typed.trim_end_matches(['\r', '\n']) != phrase {
        return Err(Error::TrainingHeld);
    }
    // Revalidate the exact displayed plan after the interactive wait.
    let current: AttentionPlan = read_json(plan_path)?;
    if encode(&current)? != encode(&plan)? {
        return Err(invalid("attention plan changed during owner confirmation"));
    }
    validate_plan(&plan)?;
    fs::create_dir(output).map_err(|e| invalid(e.to_string()))?;
    let output = fs::canonicalize(output).map_err(|e| invalid(e.to_string()))?;
    let path = output.join("owner.json");
    let owner = AttentionOwner {
        owner: "Casey".into(),
        decision: OWNER_DECISION.into(),
        scope: plan.scope.clone(),
        plan_sha256: corpus::sha256(&encode(&plan)?),
        receipt_url: format!("owner-terminal:{}", path.display()),
    };
    write_new(&path, &owner)?;
    let admission = AttentionAdmission {
        schema: ADMISSION_SCHEMA.into(),
        plan,
        owner_release: FilePin {
            path,
            sha256: corpus::sha256(&encode(&owner)?),
        },
    };
    write_new(&output.join("admission.json"), &admission)?;
    sync_directory(&output)
}
#[cfg(feature = "cuda")]
fn release(
    driver: &PreparedDriver,
    admission: &AttentionAdmission,
    compiled: &Value,
    executable: &str,
) -> Result<ValidatedRelease> {
    validate_plan(&admission.plan)?;
    if admission.schema != ADMISSION_SCHEMA {
        return Err(Error::TrainingHeld);
    }
    validate_execution(&admission.plan, driver, executable)?;
    validate_compiled(&admission.plan, compiled)?;
    let owner: AttentionOwner = serde_json::from_slice(&admission.owner_release.read()?)
        .map_err(|e| invalid(e.to_string()))?;
    validate_owner(&admission.plan, &admission.owner_release, &owner)?;
    Ok(ValidatedRelease {
        manifest: ReleaseManifest {
            schema: ADMISSION_SCHEMA.into(),
            scope: admission.plan.scope.clone(),
            execution: admission.plan.execution.clone(),
            sheet: admission.plan.sheet.clone(),
            claude_review: admission.plan.independent_review.clone(),
            owner_release: admission.owner_release.clone(),
            run_state_directory: admission.plan.run_state_directory.clone(),
        },
        manifest_sha256: corpus::sha256(&encode(admission)?),
        max_batches: ATTENTION_BATCHES,
    })
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Counters {
    vector: KindCounters,
    route: KindCounters,
    gate_combine: KindCounters,
    gate_relation: KindCounters,
    refused_relation_proposals: u64,
}
#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Vector,
    GateCombine,
    GateRelation,
}
#[cfg(any(feature = "cuda", test))]
fn gate_inventory(config: &DriverConfig) -> Result<(u32, u32)> {
    let stages = BinderPlan::new(config.spec.context as usize)?.stages() as u32;
    Ok((
        config.spec.blocks * stages,
        config.read_spec.gate_count(config.spec.blocks, stages)?,
    ))
}
#[cfg(any(feature = "cuda", test))]
fn family(config: &DriverConfig, coordinate: training::Coordinate) -> Result<Family> {
    let (combine, total) = gate_inventory(config)?;
    match coordinate.kind {
        CoordinateKind::VectorBlock
            if coordinate.index < config.spec.vocabulary * config.spec.blocks =>
        {
            Ok(Family::Vector)
        }
        CoordinateKind::Gate if coordinate.index < combine => Ok(Family::GateCombine),
        CoordinateKind::Gate
            if matches!(config.read_spec, ReadSpec::Relation { .. })
                && coordinate.index < total =>
        {
            Ok(Family::GateRelation)
        }
        _ => Err(invalid(
            "attention coordinate is outside the admitted vector/combine/relation family",
        )),
    }
}
#[cfg(any(feature = "cuda", test))]
impl Counters {
    fn validate(&self, config: &DriverConfig, completed: u64, accepted: u64) -> Result<()> {
        if completed > ATTENTION_BATCHES {
            return Err(invalid("attention completed batches exceed release"));
        }
        let (combine, total) = gate_inventory(config)?;
        let reads = completed / 8;
        let combines = (reads / u64::from(total)) * u64::from(combine)
            + (reads % u64::from(total)).min(u64::from(combine));
        let expected = [completed - reads, 0, combines, reads - combines];
        let refused =
            if config.relation_learning_policy == training::RelationLearningPolicy::FrozenZeroV1 {
                reads - combines
            } else {
                0
            };
        if self.refused_relation_proposals != refused
            || (refused != 0 && self.gate_relation.accepted_updates != 0)
        {
            return Err(invalid(
                "attention frozen-zero refusal counts or relation applies differ",
            ));
        }
        let mut sum = 0;
        for (counter, attempts) in [
            &self.vector,
            &self.route,
            &self.gate_combine,
            &self.gate_relation,
        ]
        .into_iter()
        .zip(expected)
        {
            if counter.attempted != attempts
                || counter.accepted_updates > counter.any_candidate_cost_differs_from_identity
                || counter.any_candidate_cost_differs_from_identity > counter.attempted
            {
                return Err(invalid("attention family counters disagree with absolute schedule or sensitivity ordering"));
            }
            // Every term is now bounded by the 16404-batch release.
            sum += counter.accepted_updates;
        }
        if sum != accepted {
            return Err(invalid(
                "attention accepted counter sum differs from model cursor",
            ));
        }
        Ok(())
    }
    #[cfg(feature = "cuda")]
    fn record(&mut self, family: Family, receipt: &training::StepReceipt) {
        let counter = match family {
            Family::Vector => &mut self.vector,
            Family::GateCombine => &mut self.gate_combine,
            Family::GateRelation => &mut self.gate_relation,
        };
        // Preflight proved all counts <= dose before the sole transition call.
        counter.attempted += 1;
        counter.any_candidate_cost_differs_from_identity +=
            u64::from(receipt.any_candidate_cost_differs_from_identity);
        counter.accepted_updates += u64::from(receipt.applied);
        self.refused_relation_proposals += u64::from(receipt.apply_refused_by_policy);
    }
}
#[cfg(feature = "cuda")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttentionCheckpoint {
    schema: String,
    admission_sha256: String,
    checkpoint: training::DriverCheckpoint,
    model_storage_sha256: String,
    model_snapshot_sha256: String,
    baseline_evaluation: Value,
    baseline_snapshot_metadata: Value,
    counters: Counters,
    q1: Option<FilePin>,
}
#[cfg(feature = "cuda")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Q1Record {
    admission_sha256: String,
    baseline_evaluation_sha256: String,
    checkpoint: training::DriverCheckpoint,
    model_storage_sha256: String,
    model_snapshot_sha256: String,
    counters: Counters,
    evaluation: FilePin,
    init_b3: evidence::B3Summary,
    q1_b3: evidence::B3Summary,
    decision: evidence::Q1Decision,
}
fn dev_row_identities(config: &DriverConfig) -> Result<Vec<(u64, u32, bool)>> {
    // Corpus/window partition metadata only. No scoring or forward computation.
    let source = corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
    let mut rows = Vec::with_capacity(source.eligible_len());
    let mut first = 0;
    while first < source.eligible_len() as u64 {
        let batch = source.batch(first, config.spec.context, config.batch_rows)?;
        for ((window, &position), &target) in batch
            .windows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
        {
            rows.push((
                position,
                target,
                evidence::is_copy_successor(window, target),
            ));
        }
        first = batch.next_target;
    }
    if rows.len() != 14_287 || rows.iter().filter(|row| row.2).count() != 3_847 {
        return Err(invalid(
            "attention DEV partition must reproduce 14287/3847 registered rows",
        ));
    }
    Ok(rows)
}
#[cfg(feature = "cuda")]
fn validate_q1(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &AttentionCheckpoint,
) -> Result<Option<Q1Record>> {
    if p.checkpoint.cursor.completed_batches < 4096 {
        if p.q1.is_some() {
            return Err(invalid("attention Q1 evidence precedes checkpoint4096"));
        }
        return Ok(None);
    }
    let pin =
        p.q1.as_ref()
            .ok_or_else(|| invalid("attention progress cannot bypass Q1"))?;
    let expected_path = release.manifest.run_state_directory.join("q1.json");
    if pin.path != fs::canonicalize(expected_path).map_err(|e| invalid(e.to_string()))? {
        return Err(invalid(
            "attention Q1 must use the canonical durable record",
        ));
    }
    let q: Q1Record = serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
    driver.validate_checkpoint(&q.checkpoint)?;
    let hashes = checkpoint_hash_domains(&q.checkpoint)?;
    verify_snapshot_hash_fields(
        &json!({"model_storage_sha256":q.model_storage_sha256,"model_snapshot_sha256":q.model_snapshot_sha256}),
        &hashes.model_storage_sha256,
        &hashes.model_snapshot_sha256,
    )?;
    q.counters
        .validate(driver.config(), 4096, q.checkpoint.cursor.accepted_updates)?;
    if q.checkpoint.cursor.completed_batches != 4096
        || q.admission_sha256 != release.manifest_sha256
        || q.baseline_evaluation_sha256 != corpus::sha256(&encode(&p.baseline_evaluation)?)
        || q.counters.vector.attempted != 3584
        || (matches!(driver.config().read_spec, ReadSpec::Relation { .. })
            && (q.counters.gate_combine.attempted != 237
                || q.counters.gate_relation.attempted != 275))
        || q.init_b3.full.rows != 14_287
        || q.init_b3.psucc.rows != 3_847
        || q.evaluation.path
            != fs::canonicalize(
                release
                    .manifest
                    .run_state_directory
                    .join("q1-evaluation.json"),
            )
            .map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "attention Q1 admission/baseline/prefix identity mismatch",
        ));
    }
    let observed: Value =
        serde_json::from_slice(&q.evaluation.read()?).map_err(|e| invalid(e.to_string()))?;
    if observed["schema"] != json!("uor-r4.attention-learning-evaluation/1")
        || observed["admission_sha256"] != json!(release.manifest_sha256)
        || observed["driver_checkpoint_sha256"] != json!(corpus::sha256(&encode(&q.checkpoint)?))
        || observed["transfer"] != Value::Null
    {
        return Err(invalid("attention Q1 evaluation envelope differs"));
    }
    verify_checkpoint_hash_fields(&observed, &q.checkpoint)?;
    verify_evaluation(&observed["evaluation"], &q.checkpoint)?;
    let rows = dev_row_identities(driver.config())?;
    if evidence::summarize(&p.baseline_evaluation, &rows)? != q.init_b3
        || evidence::summarize(&observed["evaluation"], &rows)? != q.q1_b3
        || evidence::q1_decision(
            &q.init_b3,
            &q.q1_b3,
            q.counters
                .gate_combine
                .any_candidate_cost_differs_from_identity,
            q.counters
                .gate_relation
                .any_candidate_cost_differs_from_identity,
        )? != q.decision
        || (p.checkpoint.cursor.completed_batches == 4096
            && (encode(&p.checkpoint)? != encode(&q.checkpoint)? || p.counters != q.counters))
        || (p.checkpoint.cursor.completed_batches > 4096 && q.decision.diagnostic_stop)
    {
        return Err(invalid(
            "attention Q1 rows/decision/current progress disagree",
        ));
    }
    Ok(Some(q))
}
#[cfg(feature = "cuda")]
fn capture_q1(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &mut AttentionCheckpoint,
    model: &crate::device::Device,
) -> Result<()> {
    let rows = dev_row_identities(driver.config())?;
    let init_b3 = evidence::summarize(&p.baseline_evaluation, &rows)?;
    let evaluation = evaluate_once(
        driver,
        release,
        "q1-evaluation.json",
        &p.checkpoint,
        model,
        None,
    )?;
    let value: Value =
        serde_json::from_slice(&evaluation.read()?).map_err(|e| invalid(e.to_string()))?;
    let q1_b3 = evidence::summarize(&value["evaluation"], &rows)?;
    let decision = evidence::q1_decision(
        &init_b3,
        &q1_b3,
        p.counters
            .gate_combine
            .any_candidate_cost_differs_from_identity,
        p.counters
            .gate_relation
            .any_candidate_cost_differs_from_identity,
    )?;
    let hashes = checkpoint_hash_domains(&p.checkpoint)?;
    let record = Q1Record {
        admission_sha256: release.manifest_sha256.clone(),
        baseline_evaluation_sha256: corpus::sha256(&encode(&p.baseline_evaluation)?),
        checkpoint: p.checkpoint.clone(),
        model_storage_sha256: hashes.model_storage_sha256,
        model_snapshot_sha256: hashes.model_snapshot_sha256,
        counters: p.counters.clone(),
        evaluation,
        init_b3,
        q1_b3,
        decision,
    };
    let path = release.manifest.run_state_directory.join("q1.json");
    if !path.exists() {
        write_new(&path, &record)?;
        sync_directory(&release.manifest.run_state_directory)?;
    }
    let bytes = fs::read(&path).map_err(|e| invalid(e.to_string()))?;
    if bytes != encode(&record)? {
        return Err(invalid("attention existing Q1 record differs"));
    }
    p.q1 = Some(FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&bytes),
    });
    validate_q1(driver, release, p)?;
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn checkpoint_boundary(n: u64) -> Result<()> {
    if n > ATTENTION_BATCHES || (!n.is_multiple_of(64) && n != ATTENTION_BATCHES) {
        return Err(invalid(
            "attention resume cursor is not a bounded durable checkpoint",
        ));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn validate_initial_storage(config: &DriverConfig, model: &Value) -> Result<()> {
    let (combine, total) = gate_inventory(config)?;
    let stages = BinderPlan::new(config.spec.context as usize)?.stages();
    let gates = model["gates"]
        .as_array()
        .ok_or_else(|| invalid("initial checkpoint lacks gates"))?;
    if model["epoch"] != json!(0)
        || model["read_initialization"] != json!(ReadInitialization::ActiveLeafV1)
        || gates.len() != total as usize
        || gates.iter().enumerate().any(|(index, word)| {
            word.as_i64()
                != Some(i64::from(
                    index < combine as usize && index.is_multiple_of(stages),
                ))
        })
    {
        return Err(invalid(
            "attention initial snapshot must preserve the declared active-leaf literal storage",
        ));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn validate_progress(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &AttentionCheckpoint,
) -> Result<()> {
    driver.validate_checkpoint(&p.checkpoint)?;
    let hashes = checkpoint_hash_domains(&p.checkpoint)?;
    verify_snapshot_hash_fields(
        &json!({"model_storage_sha256":p.model_storage_sha256,"model_snapshot_sha256":p.model_snapshot_sha256}),
        &hashes.model_storage_sha256,
        &hashes.model_snapshot_sha256,
    )?;
    let c = p.checkpoint.cursor;
    checkpoint_boundary(c.completed_batches)?;
    p.counters
        .validate(driver.config(), c.completed_batches, c.accepted_updates)?;
    if p.schema != PROGRESS_SCHEMA
        || p.admission_sha256 != release.manifest_sha256
        || p.baseline_evaluation["model_epoch"] != json!(0)
        || p.baseline_evaluation["evaluation_lineage_sha256"] != json!(release.lineage_sha256())
        || p.baseline_evaluation["evaluation_mutates_model"] != json!(false)
        || p.baseline_evaluation["updates_applied_by_evaluation"] != json!(0)
    {
        return Err(invalid(
            "attention progress admission or baseline identity mismatch",
        ));
    }
    for (key, expected) in [
        ("source_sha256", &release.manifest.execution.source_sha256),
        ("kernel_sha256", &release.manifest.execution.kernel_sha256),
        (
            "geometry_sha256",
            &release.manifest.execution.geometry_sha256,
        ),
    ] {
        if p.checkpoint.model[key].as_str() != Some(expected.as_str()) {
            return Err(invalid(format!(
                "attention checkpoint {key} differs from admission"
            )));
        }
    }
    if c.completed_batches == 0 {
        validate_initial_storage(driver.config(), &p.checkpoint.model)?;
    }
    validate_q1(driver, release, p)?;
    Ok(())
}
/// Validate named metadata domains. Old attention wrappers without the new pair
/// are refused; this does not migrate or alter the historical DriverCheckpoint.
#[cfg(any(feature = "cuda", test))]
fn verify_snapshot_hash_fields(record: &Value, storage: &str, snapshot: &str) -> Result<()> {
    corpus::validate_sha256(storage)?;
    corpus::validate_sha256(snapshot)?;
    if record["model_storage_sha256"].as_str() != Some(storage)
        || record["model_snapshot_sha256"].as_str() != Some(snapshot)
    {
        return Err(invalid(
            "snapshot storage/typed hash domains are missing or differ",
        ));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn checkpoint_hash_domains(
    checkpoint: &training::DriverCheckpoint,
) -> Result<crate::device::SnapshotHashDomains> {
    let hashes = crate::device::stored_snapshot_hash_domains(&checkpoint.model)?;
    if hashes.model_storage_sha256 != checkpoint.model_sha256 {
        return Err(invalid("DriverCheckpoint legacy storage hash differs"));
    }
    Ok(hashes)
}
#[cfg(feature = "cuda")]
fn verify_checkpoint_hash_fields(
    record: &Value,
    checkpoint: &training::DriverCheckpoint,
) -> Result<()> {
    let hashes = checkpoint_hash_domains(checkpoint)?;
    verify_snapshot_hash_fields(
        record,
        &hashes.model_storage_sha256,
        &hashes.model_snapshot_sha256,
    )
}
#[cfg(feature = "cuda")]
fn verify_evaluation(evaluation: &Value, checkpoint: &training::DriverCheckpoint) -> Result<()> {
    verify_checkpoint_hash_fields(evaluation, checkpoint)?;
    if evaluation["model_epoch"] != json!(checkpoint.cursor.accepted_updates)
        || evaluation["evaluation_lineage_sha256"]
            != json!(corpus::sha256(&encode(&checkpoint.lineage)?))
        || evaluation["evaluation_mutates_model"] != json!(false)
        || evaluation["updates_applied_by_evaluation"] != json!(0)
    {
        return Err(invalid(
            "attention evaluation is not bound to the exact immutable snapshot",
        ));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn save(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    out: &Path,
    p: &AttentionCheckpoint,
) -> Result<FilePin> {
    validate_progress(driver, release, p)?;
    let path = out.join(format!(
        "checkpoint-{:05}.json",
        p.checkpoint.cursor.completed_batches
    ));
    write_new(&path, p)?;
    sync_directory(out)?;
    let pin = FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&encode(p)?),
    };
    let state = &release.manifest.run_state_directory;
    if p.checkpoint.cursor.completed_batches == 0 && !state.join("initial.json").exists() {
        write_new(&state.join("initial.json"), &pin)?;
        sync_directory(state)?;
    }
    // Only the latest pointer is replaceable. Interrupted pending files survive.
    let pending = state.join(format!(
        "latest-{}-{}.pending",
        std::process::id(),
        unix_ns()?
    ));
    write_new(&pending, &pin)?;
    fs::rename(pending, state.join("latest.json")).map_err(|e| invalid(e.to_string()))?;
    sync_directory(state)?;
    Ok(pin)
}
#[cfg(feature = "cuda")]
fn read_progress(pin: &FilePin) -> Result<AttentionCheckpoint> {
    serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))
}
#[cfg(feature = "cuda")]
fn evaluate_once(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    name: &str,
    checkpoint: &training::DriverCheckpoint,
    model: &crate::device::Device,
    transfer: Option<&corpus::PreparedCorpus>,
) -> Result<FilePin> {
    let path = release.manifest.run_state_directory.join(name);
    let hashes = checkpoint_hash_domains(checkpoint)?;
    if !path.exists() {
        let evaluation = match transfer {
            Some(source) => driver.evaluate_transfer_checkpoint(model, source, checkpoint)?,
            None => driver.evaluate_checkpoint(model, checkpoint)?,
        };
        verify_evaluation(&evaluation, checkpoint)?;
        write_new(
            &path,
            &json!({"schema":"uor-r4.attention-learning-evaluation/1",
            "admission_sha256":release.manifest_sha256,
            "transfer":transfer.map(|source| source.binding()),
            "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
            "driver_checkpoint_sha256":corpus::sha256(&encode(checkpoint)?),"evaluation":evaluation}),
        )?;
        sync_directory(&release.manifest.run_state_directory)?;
    }
    let bytes = fs::read(&path).map_err(|e| invalid(e.to_string()))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    if value["schema"] != json!("uor-r4.attention-learning-evaluation/1")
        || value["admission_sha256"] != json!(release.manifest_sha256)
        || value["transfer"] != json!(transfer.map(|source| source.binding()))
        || value["driver_checkpoint_sha256"] != json!(corpus::sha256(&encode(checkpoint)?))
    {
        return Err(invalid(
            "attention durable evaluation admission/checkpoint differs",
        ));
    }
    verify_checkpoint_hash_fields(&value, checkpoint)?;
    verify_evaluation(&value["evaluation"], checkpoint)?;
    if let Some(source) = transfer {
        if value["evaluation"]["transfer_corpus"] != json!(source.binding())
            || value["evaluation"]["transfer_is_separate_evaluation_content"] != json!(true)
        {
            return Err(invalid(
                "attention transfer evaluation source binding differs",
            ));
        }
    } else if value["evaluation"].get("transfer_corpus").is_some() {
        return Err(invalid(
            "attention DEV evaluation cannot substitute a transfer panel",
        ));
    }
    Ok(FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&bytes),
    })
}
#[cfg(feature = "cuda")]
fn relation_off(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &AttentionCheckpoint,
) -> Result<Value> {
    if matches!(driver.config().read_spec, ReadSpec::Flock { .. }) {
        return Ok(json!({"status":"NOT_APPLICABLE_FLOCK_RELATION_ALREADY_OFF"}));
    }
    if driver.config().relation_learning_policy == training::RelationLearningPolicy::FrozenZeroV1 {
        return Ok(json!({"status":"NOT_APPLICABLE_MATCHED_CONTROL_RELATION_PINNED_ZERO"}));
    }
    let mut snapshot: crate::device::Snapshot =
        serde_json::from_value(p.checkpoint.model.clone()).map_err(|e| invalid(e.to_string()))?;
    let (combine, total) = gate_inventory(driver.config())?;
    if snapshot.gates.len() != total as usize {
        return Err(invalid("relation-off snapshot gate inventory differs"));
    }
    // Storage-only ablation. No host forward, scoring, routing or apply occurs.
    snapshot.gates[combine as usize..].fill(0);
    let mut ablated = p.checkpoint.clone();
    ablated.model = serde_json::to_value(&snapshot).map_err(|e| invalid(e.to_string()))?;
    ablated.model_sha256 = corpus::sha256(&encode(&ablated.model)?);
    driver.validate_checkpoint(&ablated)?;
    let parent_hashes = checkpoint_hash_domains(&p.checkpoint)?;
    let ablated_hashes = checkpoint_hash_domains(&ablated)?;
    let state = &release.manifest.run_state_directory;
    let path = state.join("relation-off-checkpoint.json");
    let record = json!({"schema":"uor-r4.attention-relation-off/1","admission_sha256":release.manifest_sha256,
        "parent_driver_checkpoint_sha256":corpus::sha256(&encode(&p.checkpoint)?),
        "parent_model_sha256":p.checkpoint.model_sha256,"modified_storage":"only relation gate trits set to zero",
        "parent_model_storage_sha256":parent_hashes.model_storage_sha256,"parent_model_snapshot_sha256":parent_hashes.model_snapshot_sha256,
        "model_storage_sha256":ablated_hashes.model_storage_sha256,"model_snapshot_sha256":ablated_hashes.model_snapshot_sha256,
        "checkpoint":ablated});
    if !path.exists() {
        write_new(&path, &record)?;
        sync_directory(state)?;
    }
    let persisted: Value = read_json(&path)?;
    if persisted != record {
        return Err(invalid("relation-off durable checkpoint changed"));
    }
    let checkpoint_pin = FilePin {
        path: fs::canonicalize(&path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&encode(&record)?),
    };
    let model = crate::device::Device::from_snapshot(snapshot, 0)?;
    let evaluation = evaluate_once(
        driver,
        release,
        "relation-off-evaluation.json",
        &ablated,
        &model,
        None,
    )?;
    let evaluated: Value =
        serde_json::from_slice(&evaluation.read()?).map_err(|e| invalid(e.to_string()))?;
    let b3 = evidence::summarize(
        &evaluated["evaluation"],
        &dev_row_identities(driver.config())?,
    )?;
    Ok(
        json!({"status":"OBSERVED_PENDING_REGISTERED_DISCRIMINATOR","checkpoint":checkpoint_pin,"b3":b3,
        "evaluation":evaluation,"transfer_evaluations":0,"parent_model_sha256":p.checkpoint.model_sha256,
        "parent_model_storage_sha256":parent_hashes.model_storage_sha256,"parent_model_snapshot_sha256":parent_hashes.model_snapshot_sha256,
        "model_storage_sha256":ablated_hashes.model_storage_sha256,"model_snapshot_sha256":ablated_hashes.model_snapshot_sha256,
        "ablated_model_sha256":ablated.model_sha256,"learned_attention_result":"NOT_YET_SCORED"}),
    )
}
/// One separately released arm/seed only. Missing admission never issues a capability.
#[cfg(feature = "cuda")]
pub fn run(
    config: DriverConfig,
    admission: AttentionAdmission,
    out: &Path,
    resume: bool,
) -> Result<()> {
    let started = std::time::Instant::now();
    validate_config(&config)?;
    let driver = PreparedDriver::load(config)?;
    let release = release(
        &driver,
        &admission,
        &crate::device::compiled_identity(),
        &executable_sha256()?,
    )?;
    require_disjoint_directories(out, &release.manifest.run_state_directory)?;
    fs::create_dir(out).map_err(|e| invalid(format!("exclusive attention output: {e}")))?;
    let _lock = lock_run(&release, resume)?;
    write_new(&out.join("admission.json"), &admission)?;
    let result = (|| -> Result<()> {
        let state = &release.manifest.run_state_directory;
        let (mut model, mut progress) = if resume {
            let latest: FilePin = read_json(&state.join("latest.json"))?;
            let p = read_progress(&latest)?;
            let initial_pin: FilePin = read_json(&state.join("initial.json"))?;
            let initial = read_progress(&initial_pin)?;
            // All admission, cursor, counters and initial baseline checks precede CUDA.
            validate_progress(&driver, &release, &p)?;
            validate_progress(&driver, &release, &initial)?;
            if initial.checkpoint.cursor != training::Cursor::initial()
                || initial.counters != Counters::default()
                || initial.baseline_snapshot_metadata
                    != training::snapshot_metadata(&initial.checkpoint.model)?
                || p.baseline_evaluation != initial.baseline_evaluation
                || p.baseline_snapshot_metadata != initial.baseline_snapshot_metadata
            {
                return Err(invalid(
                    "attention resume lost the immutable checkpoint0 baseline",
                ));
            }
            verify_evaluation(&initial.baseline_evaluation, &initial.checkpoint)?;
            if validate_q1(&driver, &release, &p)?.is_some_and(|q| q.decision.diagnostic_stop) {
                return Err(invalid(
                    "Q1 stopped for diagnosis; resume cannot bypass registered stop",
                ));
            }
            write_new(&out.join("resume-parent.json"), &latest)?;
            let (model, _) = driver.restore(p.checkpoint.clone(), 0)?;
            (model, p)
        } else {
            let model = driver.new_model(0)?;
            let checkpoint = driver.checkpoint(&model, training::Cursor::initial())?;
            validate_initial_storage(driver.config(), &checkpoint.model)?;
            let baseline_snapshot_metadata = training::snapshot_metadata(&checkpoint.model)?;
            let baseline_evaluation = driver.evaluate_checkpoint(&model, &checkpoint)?;
            verify_evaluation(&baseline_evaluation, &checkpoint)?;
            let hashes = checkpoint_hash_domains(&checkpoint)?;
            (
                model,
                AttentionCheckpoint {
                    schema: PROGRESS_SCHEMA.into(),
                    admission_sha256: release.manifest_sha256.clone(),
                    model_storage_sha256: hashes.model_storage_sha256,
                    model_snapshot_sha256: hashes.model_snapshot_sha256,
                    checkpoint,
                    baseline_evaluation,
                    baseline_snapshot_metadata,
                    counters: Counters::default(),
                    q1: None,
                },
            )
        };
        // Checkpoint0 and its complete baseline are durable before capability issuance.
        let mut final_pin = save(&driver, &release, out, &progress)?;
        let capability = training::issue_corpus_update_admission(&driver, &release)?;
        let mut receipts = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("steps.jsonl"))
            .map_err(|e| invalid(e.to_string()))?;
        while progress.checkpoint.cursor.completed_batches < ATTENTION_BATCHES {
            let before = progress.checkpoint.cursor;
            driver.validate_cursor(&before)?;
            progress.counters.validate(
                driver.config(),
                before.completed_batches,
                before.accepted_updates,
            )?;
            let coordinate = driver.coordinate_at(before.completed_batches)?;
            let expected_family = family(driver.config(), coordinate)?;
            // Family and all counters are admitted BEFORE the existing sole apply path.
            let receipt = driver.transition(&mut model, &before, &capability)?;
            if receipt.coordinate != coordinate
                || receipt.cursor_before != before
                || receipt.cursor_after != driver.advance(&before, receipt.applied)?
                || receipt.model_epoch != receipt.cursor_after.accepted_updates
                || receipt.lineage_sha256 != release.lineage_sha256()
                || (receipt.applied && !receipt.any_candidate_cost_differs_from_identity)
                || receipt.apply_refused_by_policy
                    != (driver.config().relation_learning_policy
                        == training::RelationLearningPolicy::FrozenZeroV1
                        && expected_family == Family::GateRelation)
                || (receipt.apply_refused_by_policy && receipt.applied)
            {
                return Err(invalid(
                    "attention transition receipt violates preflight identity",
                ));
            }
            receipts
                .write_all(&encode(&receipt)?)
                .and_then(|_| receipts.write_all(b"\n"))
                .and_then(|_| receipts.sync_all())
                .map_err(|e| invalid(e.to_string()))?;
            progress.counters.record(expected_family, &receipt);
            progress.checkpoint.cursor = receipt.cursor_after;
            let completed = receipt.cursor_after.completed_batches;
            if completed.is_multiple_of(64) || completed == ATTENTION_BATCHES {
                progress.checkpoint = driver.checkpoint(&model, receipt.cursor_after)?;
                let hashes = checkpoint_hash_domains(&progress.checkpoint)?;
                progress.model_storage_sha256 = hashes.model_storage_sha256;
                progress.model_snapshot_sha256 = hashes.model_snapshot_sha256;
                if completed == 4096 {
                    capture_q1(&driver, &release, &mut progress, &model)?;
                }
                final_pin = save(&driver, &release, out, &progress)?;
                if completed == 4096
                    && validate_q1(&driver, &release, &progress)?
                        .is_some_and(|q| q.decision.diagnostic_stop)
                {
                    write_new(
                        &out.join("report.json"),
                        &json!({"status":"Q1_STOP_FOR_DIAGNOSIS",
                        "mechanism_negative":false,"learned_attention_result":"NOT_YET_SCORED",
                        "admission_sha256":release.manifest_sha256,"checkpoint":final_pin,"q1":progress.q1,
                        "model_storage_sha256":progress.model_storage_sha256,"model_snapshot_sha256":progress.model_snapshot_sha256,
                        "completed_batches":4096,"fresh_transfer_evaluations":0}),
                    )?;
                    return sync_directory(out);
                }
            }
        }
        let final_evaluation = evaluate_once(
            &driver,
            &release,
            "final-evaluation.json",
            &progress.checkpoint,
            &model,
            None,
        )?;
        let final_value: Value = serde_json::from_slice(&final_evaluation.read()?)
            .map_err(|e| invalid(e.to_string()))?;
        let final_b3 = evidence::summarize(
            &final_value["evaluation"],
            &dev_row_identities(driver.config())?,
        )?;
        let ablation = relation_off(&driver, &release, &progress)?;
        write_new(
            &out.join("report.json"),
            &json!({"schema":"uor-r4.attention-learning-report/1",
            "status":"COMPLETE_PENDING_REGISTERED_DISCRIMINATOR","learned_attention_result":"NOT_YET_SCORED",
            "scope":admission.plan.scope,"admission_sha256":release.manifest_sha256,
            "cost_contract":admission.plan.cost_contract,"completed_batches":ATTENTION_BATCHES,
            "eligible_train_target_exposures":ATTENTION_BATCHES*64,"sensitivity":progress.counters,
            "baseline_evaluation":progress.baseline_evaluation,"baseline_snapshot_metadata":progress.baseline_snapshot_metadata,
            "final_checkpoint":final_pin,"final_evaluation":final_evaluation,"final_b3":final_b3,"relation_off":ablation,
            "model_storage_sha256":progress.model_storage_sha256,"model_snapshot_sha256":progress.model_snapshot_sha256,
            "attempt_elapsed_ns":started.elapsed().as_nanos().to_string(),
            "fresh_transfer":{"status":"SEALED_PENDING_PAIRED_DEV_DECISION","source":admission.plan.transfer,"evaluations":0},
            "q1":progress.q1,
            "other_arm_seed":"SEPARATE_OWNER_RELEASE_REQUIRED",
            "training_other_scopes":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}),
        )?;
        sync_directory(out)
    })();
    if let Err(error) = &result {
        // Preserve the original failure even if writing its diagnostic receipt also fails.
        let _ = write_new(
            &out.join("failure.json"),
            &json!({"status":"INSTRUMENT_OR_EXECUTION_FAILURE",
            "error":error.to_string(),"mechanism_negative":false,"learned_attention_result":"NOT_YET_SCORED",
            "attempt_elapsed_ns":started.elapsed().as_nanos().to_string(),
            "resume":"latest durable 64-batch checkpoint; unpersisted segment replay is explicit, retained steps remain evidence",
            "training_other_scopes":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_hash_pair_requires_named_domains_without_legacy_fallback() -> Result<()> {
        let storage = "a".repeat(64);
        let snapshot = "b".repeat(64);
        let pair = json!({"model_storage_sha256":storage,"model_snapshot_sha256":snapshot});
        verify_snapshot_hash_fields(&pair, &storage, &snapshot)?;
        for invalid in [
            json!({"model_snapshot_sha256":snapshot}),
            json!({"model_sha256":storage,"model_snapshot_sha256":snapshot}),
            json!({"model_storage_sha256":snapshot,"model_snapshot_sha256":storage}),
            json!({"model_storage_sha256":storage,"model_snapshot_sha256":null}),
            json!({"model_storage_sha256":storage,"model_snapshot_sha256":[]}),
        ] {
            assert!(verify_snapshot_hash_fields(&invalid, &storage, &snapshot).is_err());
        }
        assert!(verify_snapshot_hash_fields(&pair, "bad", &snapshot).is_err());
        Ok(())
    }

    #[test]
    fn hash_pair_validation_keeps_file_and_checkpoint_domains_separate() -> Result<()> {
        // A metadata-only encoding example: declared struct order and sorted
        // Value order differ, just as the production typed Snapshot fixture
        // (CUDA-feature source test) checks with its real serializer.
        #[derive(Serialize)]
        struct StoredMetadata {
            z: u32,
            a: u32,
        }
        let typed = StoredMetadata { z: 7, a: 1 };
        let stored = serde_json::to_value(&typed).map_err(|e| invalid(e.to_string()))?;
        let storage = corpus::sha256(&encode(&stored)?);
        let snapshot = corpus::sha256(&encode(&typed)?);
        assert_ne!(storage, snapshot);
        let raw_file = corpus::sha256(
            &serde_json::to_vec_pretty(&stored).map_err(|e| invalid(e.to_string()))?,
        );
        let checkpoint = corpus::sha256(&encode(&json!({"model":stored,"model_sha256":storage}))?);
        let mut record = json!({"model_storage_sha256":storage,"model_snapshot_sha256":snapshot,
            "driver_checkpoint_sha256":checkpoint,"checkpoint_file":{"sha256":raw_file}});
        verify_snapshot_hash_fields(&record, &storage, &snapshot)?;
        record["model_snapshot_sha256"] = json!(checkpoint);
        assert!(verify_snapshot_hash_fields(&record, &storage, &snapshot).is_err());
        record["model_snapshot_sha256"] = json!(raw_file);
        assert!(verify_snapshot_hash_fields(&record, &storage, &snapshot).is_err());
        Ok(())
    }

    // Configuration/admission/counter metadata only. No corpus, model or CUDA is constructed.
    fn config(read_spec: ReadSpec) -> DriverConfig {
        let pin = |path: &str, hash: &str| FilePin {
            path: PathBuf::from(path),
            sha256: hash.into(),
        };
        DriverConfig {
            schema: training::DRIVER_SCHEMA.into(),
            spec: Spec {
                vocabulary: 4096,
                blocks: 4,
                context: 192,
                kept_cells: 8,
            },
            read_spec,
            objective_policy: crate::ObjectivePolicy::NearestAddressMarginV1,
            read_initialization: ReadInitialization::ActiveLeafV1,
            relation_learning_policy: training::RelationLearningPolicy::LearnV1,
            counterfactual_path: training::CounterfactualPath::LanesV1,
            codebook_initialization: training::CodebookInitialization::SeededDeviceV1,
            seed: 1,
            batch_rows: 64,
            train: corpus::CorpusSource {
                split: corpus::Split::Train,
                tokens: pin(
                    "/metadata/train",
                    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8",
                ),
                mask: pin(
                    "/metadata/train-mask",
                    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3",
                ),
                expected_payload_tokens: 7_512_797,
                expected_eligible_targets: 2_425_925,
            },
            dev: corpus::CorpusSource {
                split: corpus::Split::Dev,
                tokens: pin(
                    "/metadata/dev",
                    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b",
                ),
                mask: pin(
                    "/metadata/dev-mask",
                    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078",
                ),
                expected_payload_tokens: 42_856,
                expected_eligible_targets: 14_287,
            },
            tokenizer: pin(
                "/metadata/tokenizer",
                "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89",
            ),
            bos: corpus::BosIdentity {
                token: "<|bos|>".into(),
                id: 0,
            },
            schedule: training::SchedulePolicy::InterleavedReadEvery8V1,
        }
    }
    fn plan() -> Result<AttentionPlan> {
        let config = config(ReadSpec::Relation { kept: 8, sweeps: 1 });
        let pin = |name: &str, digit: char| FilePin {
            path: PathBuf::from(format!("/metadata/{name}")),
            sha256: digit.to_string().repeat(64),
        };
        Ok(AttentionPlan {
            schema: PLAN_SCHEMA.into(),
            status: "HELD_OWNER_ACTION_REQUIRED".into(),
            scope: scope(&config)?,
            execution: ExecutionIdentity {
                base_source: ATTENTION_BASE_SOURCE.into(),
                reviewed_source_commit: "1".repeat(40),
                source_sha256: "0".repeat(64),
                kernel_sha256: "1".repeat(64),
                geometry_sha256: "2".repeat(64),
                executable_sha256: "3".repeat(64),
                driver_source_sha256: training::driver_source_sha256(),
                config_sha256: corpus::sha256(&encode(&config)?),
                lineage_sha256: "4".repeat(64),
                sheet_sha256: ATTENTION_SHEET_SHA256.into(),
            },
            sheet: FilePin {
                path: PathBuf::from("/metadata/m1-learned-attention-link-2026-10-17.md"),
                sha256: ATTENTION_SHEET_SHA256.into(),
            },
            independent_review: pin("review", '6'),
            transfer: corpus::CorpusSource {
                split: corpus::Split::Dev,
                tokens: pin("transfer", '8'),
                mask: pin("transfer-mask", '9'),
                expected_payload_tokens: 2001,
                expected_eligible_targets: 2000,
            },
            run_state_directory: PathBuf::from("/metadata/state"),
            cost_contract: AttentionCostContract::expected(&config),
            config,
        })
    }
    #[test]
    fn attention_requires_its_own_scope_initialization_and_pins() -> Result<()> {
        let p = plan()?;
        validate_plan_header(&p)?;
        let mut unrelated_sheet = p.clone();
        unrelated_sheet.sheet.sha256 = "5".repeat(64);
        unrelated_sheet.execution.sheet_sha256 = unrelated_sheet.sheet.sha256.clone();
        assert!(validate_plan_header(&unrelated_sheet).is_err());
        assert!(validate_pilot_config(&p.config).is_err());
        for old_scope in [PILOT_SCOPE, "m1-a2-completion-seed1-checkpoint1640..16404"] {
            let mut wrong = p.clone();
            wrong.scope = old_scope.into();
            assert!(validate_plan_header(&wrong).is_err());
        }
        for old_schema in [
            "uor-r4.pilot-owner-release-plan/1",
            "uor-r4.completion-owner-release-plan/1",
        ] {
            let mut wrong = p.clone();
            wrong.schema = old_schema.into();
            assert!(validate_plan_header(&wrong).is_err());
        }
        let mut wrong = p.config.clone();
        wrong.read_initialization = ReadInitialization::ZeroGatesV1;
        assert!(validate_config(&wrong).is_err());
        wrong = p.config.clone();
        wrong.schedule = training::SchedulePolicy::CanonicalVectorThenReadV1;
        assert!(validate_config(&wrong).is_err());
        wrong = p.config.clone();
        wrong.seed = 3;
        assert!(validate_config(&wrong).is_err());
        wrong = p.config.clone();
        wrong.train.mask.sha256 = "a".repeat(64);
        assert!(validate_config(&wrong).is_err());
        let flock = config(ReadSpec::Flock { kept: 8 });
        validate_config(&flock)?;
        assert_ne!(scope(&flock)?, p.scope);
        Ok(())
    }
    #[test]
    fn review_and_owner_bind_plan_and_refuse_inherited_authority() -> Result<()> {
        let p = plan()?;
        let review = AttentionReview {
            reviewer: "AGY".into(),
            verdict: "APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF".into(),
            scope: p.scope.clone(),
            execution: p.execution.clone(),
            reviewed_plan_payload_sha256: review_payload_sha256(&p)?,
            receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-1"
                .into(),
        };
        validate_review(&p, &review)?;
        let mut author = review.clone();
        author.reviewer = "Codex".into();
        assert!(validate_review(&p, &author).is_err());
        let mut altered = p.clone();
        altered.transfer.tokens.sha256 = "b".repeat(64);
        assert!(validate_review(&altered, &review).is_err());
        altered = p.clone();
        altered.run_state_directory = PathBuf::from("/metadata/other-state");
        assert!(validate_review(&altered, &review).is_err());
        let pin = FilePin {
            path: PathBuf::from("/metadata/owner.json"),
            sha256: "0".repeat(64),
        };
        // Unwritten validation fixture: no owner-release file is produced by this test.
        let owner = AttentionOwner {
            owner: "Casey".into(),
            decision: OWNER_DECISION.into(),
            scope: p.scope.clone(),
            plan_sha256: corpus::sha256(&encode(&p)?),
            receipt_url: "owner-terminal:/metadata/owner.json".into(),
        };
        validate_owner(&p, &pin, &owner)?;
        for actor in ["Claude", "Codex", "AGY", "Casey via Claude"] {
            let mut wrong = owner.clone();
            wrong.owner = actor.into();
            assert!(validate_owner(&p, &pin, &wrong).is_err());
        }
        for old_decision in [
            "RELEASE_D4_FOR_A2_PILOT_ONLY",
            "RELEASE_D4_FOR_THIS_COMPLETION_SEED_ONLY",
        ] {
            let mut wrong = owner.clone();
            wrong.decision = old_decision.into();
            assert!(validate_owner(&p, &pin, &wrong).is_err());
        }
        assert!(validate_owner(&altered, &pin, &owner).is_err());
        require_owner_terminal(false, true, true)?;
        for flags in [
            (true, true, true),
            (false, false, true),
            (false, true, false),
        ] {
            assert!(require_owner_terminal(flags.0, flags.1, flags.2).is_err());
        }
        Ok(())
    }
    #[test]
    fn transfer_and_evaluation_cost_are_prospectively_fixed() -> Result<()> {
        let p = plan()?;
        assert_eq!(p.cost_contract.q1_dev_evaluations, 1);
        assert_eq!(p.cost_contract.final_transfer_evaluations, 1);
        assert_eq!(p.cost_contract.relation_off_transfer_evaluations, 0);
        let mut control = p.config.clone();
        control.relation_learning_policy = training::RelationLearningPolicy::FrozenZeroV1;
        assert_eq!(
            AttentionCostContract::expected(&control).final_transfer_evaluations,
            1
        );
        let mut wrong = p.clone();
        wrong.cost_contract.q1_dev_evaluations = 0;
        assert!(validate_plan_header(&wrong).is_err());
        wrong = p.clone();
        wrong.transfer.tokens.sha256 = p.config.dev.tokens.sha256.clone();
        assert!(validate_transfer_source(&wrong).is_err());
        wrong = p.clone();
        wrong.transfer.mask.sha256 = p.config.train.mask.sha256.clone();
        assert!(validate_transfer_source(&wrong).is_err());
        wrong = p.clone();
        wrong.transfer.expected_eligible_targets = 1999;
        assert!(validate_transfer_source(&wrong).is_err());
        wrong = p.clone();
        wrong.transfer.split = corpus::Split::Train;
        assert!(validate_transfer_source(&wrong).is_err());
        wrong = p.clone();
        wrong.cost_contract.relation_off_transfer_evaluations = 1;
        assert!(validate_plan_header(&wrong).is_err());
        wrong = p;
        wrong.cost_contract.max_batches = ATTENTION_BATCHES + 1;
        assert!(validate_plan_header(&wrong).is_err());
        Ok(())
    }
    #[test]
    fn pinned_codebook_lineage_has_its_own_scope() -> Result<()> {
        let seeded = config(ReadSpec::Relation { kept: 8, sweeps: 1 });
        // The seeded (Dose 1) scope string is unchanged by this field.
        assert_eq!(
            scope(&seeded)?,
            "m1-attention-learning-relation-k8-seed1-batches0..16404"
        );
        let mut pinned = seeded.clone();
        pinned.codebook_initialization = training::CodebookInitialization::PinnedTableV1 {
            table: FilePin {
                path: PathBuf::from("/codebook.json"),
                sha256: "a".repeat(64),
            },
        };
        assert_eq!(
            scope(&pinned)?,
            "m1-attention-learning-injective-relation-k8-seed1-batches0..16404"
        );
        let mut control = pinned.clone();
        control.relation_learning_policy = training::RelationLearningPolicy::FrozenZeroV1;
        assert_eq!(
            scope(&control)?,
            "m1-attention-learning-injective-relation-frozen-zero-k8-seed1-batches0..16404"
        );
        Ok(())
    }
    #[test]
    fn matched_frozen_zero_control_keeps_inventory_and_refuses_relation_slots() -> Result<()> {
        let treatment = config(ReadSpec::Relation { kept: 8, sweeps: 1 });
        let mut control = treatment.clone();
        control.relation_learning_policy = training::RelationLearningPolicy::FrozenZeroV1;
        validate_config(&control)?;
        assert_eq!(gate_inventory(&treatment)?, gate_inventory(&control)?);
        assert_ne!(scope(&treatment)?, scope(&control)?);
        assert_eq!(
            AttentionCostContract::expected(&control).relation_off_dev_evaluations,
            0
        );
        assert_eq!(
            AttentionCostContract::expected(&control).relation_off_transfer_evaluations,
            0
        );
        let mut matched = counts(14354, 920, 1130);
        assert!(matched.validate(&control, ATTENTION_BATCHES, 0).is_err());
        matched.refused_relation_proposals = 1130;
        matched.validate(&control, ATTENTION_BATCHES, 0)?;
        assert!(matched.validate(&treatment, ATTENTION_BATCHES, 0).is_err());
        matched
            .gate_relation
            .any_candidate_cost_differs_from_identity = 1;
        matched.gate_relation.accepted_updates = 1;
        assert!(matched.validate(&control, ATTENTION_BATCHES, 1).is_err());
        let mut wrong = control;
        wrong.read_spec = ReadSpec::Flock { kept: 8 };
        assert!(validate_config(&wrong).is_err());
        Ok(())
    }
    fn counts(vector: u64, combine: u64, relation: u64) -> Counters {
        Counters {
            vector: KindCounters {
                attempted: vector,
                ..KindCounters::default()
            },
            gate_combine: KindCounters {
                attempted: combine,
                ..KindCounters::default()
            },
            gate_relation: KindCounters {
                attempted: relation,
                ..KindCounters::default()
            },
            ..Counters::default()
        }
    }
    #[test]
    fn absolute_schedule_counter_bounds_refuse_corruption_before_apply() -> Result<()> {
        let relation = config(ReadSpec::Relation { kept: 8, sweeps: 1 });
        for (n, v, c, r) in [
            (0, 0, 0, 0),
            (7, 7, 0, 0),
            (8, 7, 1, 0),
            (160, 140, 20, 0),
            (168, 147, 20, 1),
            (4096, 3584, 237, 275),
            (16404, 14354, 920, 1130),
        ] {
            counts(v, c, r).validate(&relation, n, 0)?;
        }
        counts(3584, 512, 0).validate(&config(ReadSpec::Flock { kept: 8 }), 4096, 0)?;
        counts(14354, 2050, 0).validate(&config(ReadSpec::Flock { kept: 8 }), 16404, 0)?;
        let valid = counts(14354, 920, 1130);
        for family in 0..4 {
            for field in 0..3 {
                let mut corrupt = valid.clone();
                let counter = match family {
                    0 => &mut corrupt.vector,
                    1 => &mut corrupt.route,
                    2 => &mut corrupt.gate_combine,
                    _ => &mut corrupt.gate_relation,
                };
                match field {
                    0 => counter.attempted = u64::MAX,
                    1 => counter.any_candidate_cost_differs_from_identity = u64::MAX,
                    _ => counter.accepted_updates = u64::MAX,
                }
                assert!(corrupt.validate(&relation, 16404, 0).is_err());
            }
        }
        assert!(valid.validate(&relation, u64::MAX, 0).is_err());
        assert!(valid.validate(&relation, 16404, 1).is_err());
        let mut wrong = counts(7, 1, 0);
        wrong.gate_combine.accepted_updates = 1;
        assert!(wrong.validate(&relation, 8, 1).is_err());
        Ok(())
    }
    #[test]
    fn family_and_resume_checkpoint_bounds_are_fail_closed() -> Result<()> {
        let relation = config(ReadSpec::Relation { kept: 8, sweeps: 1 });
        let gate = |index| training::Coordinate {
            kind: CoordinateKind::Gate,
            index,
        };
        assert_eq!(family(&relation, gate(19))?, Family::GateCombine);
        assert_eq!(family(&relation, gate(20))?, Family::GateRelation);
        assert_eq!(family(&relation, gate(44))?, Family::GateRelation);
        assert!(family(&relation, gate(45)).is_err());
        assert!(family(&config(ReadSpec::Flock { kept: 8 }), gate(20)).is_err());
        assert!(family(
            &relation,
            training::Coordinate {
                kind: CoordinateKind::Route,
                index: 0
            }
        )
        .is_err());
        assert!(family(
            &relation,
            training::Coordinate {
                kind: CoordinateKind::VectorBlock,
                index: 16384
            }
        )
        .is_err());
        for n in [0, 64, 16384, 16404] {
            checkpoint_boundary(n)?;
        }
        for n in [1, 63, 65, 16403, 16405, u64::MAX] {
            assert!(checkpoint_boundary(n).is_err());
        }
        Ok(())
    }
    #[test]
    fn compiled_identity_requires_all_three_exact_pins() -> Result<()> {
        let p = plan()?;
        let identity = json!({"source_sha256":p.execution.source_sha256,
            "kernel_sha256":p.execution.kernel_sha256,"geometry_sha256":p.execution.geometry_sha256});
        validate_compiled(&p, &identity)?;
        for key in ["source_sha256", "kernel_sha256", "geometry_sha256"] {
            let mut wrong = identity.clone();
            wrong[key] = json!("f".repeat(64));
            assert!(validate_compiled(&p, &wrong).is_err());
        }
        Ok(())
    }
}
