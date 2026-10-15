use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedirectEvidence {
    pub packet: FilePin,
    pub qualification: FilePin,
    pub pm_review: FilePin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Qualification {
    schema: String,
    scope: String,
    execution: ExecutionIdentity,
    packet_sha256: String,
    controls: Vec<ControlReceipt>,
    cuda_build: FilePin,
    cuda_build_exit: i32,
    cuda_clippy: FilePin,
    cuda_clippy_exit: i32,
    timing: FilePin,
    initial_checkpoint: FilePin,
    initial_evaluation: FilePin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlReceipt {
    id: String,
    status: String,
    receipt: FilePin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QualificationReview {
    reviewer: String,
    verdict: String,
    scope: String,
    execution: ExecutionIdentity,
    qualification_sha256: String,
    packet_sha256: String,
    exact_source_review: String,
    receipt_url: String,
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(format!("K16 redirect: {message}")))
    }
}

fn read_pin<T: serde::de::DeserializeOwned>(pin: &FilePin) -> Result<T> {
    serde_json::from_slice(&pin.read()?).map_err(|error| invalid(error.to_string()))
}

fn nonempty(pin: &FilePin) -> Result<()> {
    require(!pin.read()?.is_empty(), "empty evidence file")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Result<DriverConfig> {
        serde_json::from_str(include_str!(
            "../../../docs/labs/admission-redirect-packet-2026-10-15/configs/k16-R-seed1.json"
        ))
        .map_err(|error| invalid(error.to_string()))
    }

    #[test]
    fn four_redirect_scopes_preserve_pilot_and_reject_unregistered_data() -> Result<()> {
        let mut config = config()?;
        let mut scopes = std::collections::BTreeSet::new();
        for seed in [1, 2] {
            for policy in [
                crate::ObjectivePolicy::NearestAddressMarginV1,
                crate::ObjectivePolicy::OccupiedCellRankHingeV1,
            ] {
                config.seed = seed;
                config.objective_policy = policy;
                validate_config(&config)?;
                assert!(!imports_pilot(&config));
                assert!(validate_pilot_config(&config).is_err());
                assert!(scopes.insert(scope(&config)?));
                assert_ne!(scope(&config)?, super::super::scope(seed)?);
            }
        }
        for mutation in 0..9 {
            let mut wrong = config.clone();
            match mutation {
                0 => wrong.seed = 3,
                1 => wrong.spec.kept_cells = 8,
                2 => wrong.batch_rows = 32,
                3 => wrong.train.mask.sha256 = "a".repeat(64),
                4 => wrong.dev.expected_eligible_targets -= 1,
                5 => wrong.read_spec = ReadSpec::Relation { kept: 8, sweeps: 1 },
                6 => wrong.bos.id = 1,
                7 => wrong.spec.context = 128,
                _ => wrong.tokenizer.sha256 = "b".repeat(64),
            }
            assert!(scope(&wrong).is_err(), "mutation {mutation}");
        }
        Ok(())
    }

    fn fixture(directory: &Path) -> Result<(CompletionPlan, Qualification)> {
        let (mut checkpoint, mut plan, _, _) =
            baseline_authentication::metadata_controls::fixture(false)?;
        plan.config = config()?;
        plan.schema = "uor-r4.redirect-owner-release-plan/1".into();
        plan.scope = scope(&plan.config)?;
        plan.execution.base_source = "2fa663b54".into();
        plan.execution.driver_source_sha256 = training::driver_source_sha256();
        plan.execution.config_sha256 = corpus::sha256(&encode(&plan.config)?);
        plan.run_state_directory = directory.join("state");
        checkpoint.checkpoint.lineage.config = plan.config.clone();
        checkpoint.checkpoint.model["seed"] = json!(1);
        checkpoint.checkpoint.model["spec"] = json!(plan.config.spec);
        checkpoint.checkpoint.model["objective_policy"] = json!(plan.config.objective_policy);
        checkpoint.checkpoint.model_sha256 = corpus::sha256(&encode(&checkpoint.checkpoint.model)?);
        plan.execution.lineage_sha256 = corpus::sha256(&encode(&checkpoint.checkpoint.lineage)?);
        let evaluation = &mut checkpoint.baseline_evaluation;
        evaluation["evaluation_lineage"] = json!(checkpoint.checkpoint.lineage);
        evaluation["evaluation_lineage_sha256"] = json!(plan.execution.lineage_sha256);
        evaluation["objective_policy"] = json!(plan.config.objective_policy);
        evaluation["model_bindings"]["seed"] = json!(1);
        evaluation["model_bindings"]["spec"] = json!(plan.config.spec);
        evaluation["model_bindings"]["objective_policy"] = json!(plan.config.objective_policy);
        let pin = |name: &str, bytes: Vec<u8>| -> Result<FilePin> {
            let path = directory.join("evidence").join(name);
            fs::write(&path, &bytes).map_err(|error| invalid(error.to_string()))?;
            Ok(FilePin {
                path,
                sha256: corpus::sha256(&bytes),
            })
        };
        plan.sheet = pin(
            "sheet",
            b"discrete-rung0-admission-redirect\nTEST FITNESS: FIT\nMETADATA TEST ONLY".to_vec(),
        )?;
        plan.execution.sheet_sha256 = plan.sheet.sha256.clone();
        let packet = pin(
            "packet",
            b"METADATA TEST ONLY, not an executed packet".to_vec(),
        )?;
        let transcript = pin("transcript", b"SYNTHETIC, not a CUDA execution".to_vec())?;
        let qualification = Qualification {
            schema: "uor-r4.redirect-qualification/1".into(),
            scope: plan.scope.clone(),
            execution: plan.execution.clone(),
            packet_sha256: packet.sha256.clone(),
            controls: (1..=6)
                .map(|index| ControlReceipt {
                    id: format!("C-R{index}"),
                    status: "PASS".into(),
                    receipt: transcript.clone(),
                })
                .collect(),
            cuda_build: transcript.clone(),
            cuda_build_exit: 0,
            cuda_clippy: transcript.clone(),
            cuda_clippy_exit: 0,
            timing: transcript,
            initial_checkpoint: pin("checkpoint", encode(&checkpoint.checkpoint)?)?,
            initial_evaluation: pin("evaluation", encode(evaluation)?)?,
        };
        let qualification_pin = pin("qualification", encode(&qualification)?)?;
        let review = QualificationReview {
            reviewer: "Kimi".into(),
            verdict: "APPROVE_K16_REDIRECT_QUALIFIED_ADMISSION".into(),
            scope: plan.scope.clone(),
            execution: plan.execution.clone(),
            qualification_sha256: qualification_pin.sha256.clone(),
            packet_sha256: packet.sha256.clone(),
            exact_source_review:
                "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-test".into(),
            receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-test"
                .into(),
        };
        let review_pin = pin("review", encode(&review)?)?;
        plan.claude_review = review_pin.clone();
        plan.redirect = Some(RedirectEvidence {
            packet,
            qualification: qualification_pin,
            pm_review: review_pin,
        });
        Ok((plan, qualification))
    }

    #[test]
    fn qualification_metadata_rejects_scope_parent_missing_evidence_and_failed_gates() -> Result<()>
    {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../local")
            .join(format!(
                "redirect-admission-test-{}-{}",
                std::process::id(),
                unix_ns()?
            ));
        fs::create_dir_all(directory.join("evidence"))
            .map_err(|error| invalid(error.to_string()))?;
        let (plan, qualification) = fixture(&directory)?;
        validate_plan(&plan)?;
        assert_eq!(check_plan(&plan)?["training"], json!("HELD"));
        let initial: training::DriverCheckpoint = read_pin(&qualification.initial_checkpoint)?;
        let evaluation: Value = read_pin(&qualification.initial_evaluation)?;
        let mut low_ceiling = evaluation.clone();
        for batch in low_ceiling["batches"]
            .as_array_mut()
            .ok_or_else(|| invalid("test batches"))?
        {
            let rows = batch["rows"]
                .as_array_mut()
                .ok_or_else(|| invalid("test rows"))?;
            let count = rows.len();
            for row in rows {
                row["address_rivals"] = json!(1);
            }
            batch["aggregate"]["address_rival_count_sum"] = json!(count);
            batch["aggregate"]["address_eligible_rows"] = json!(count);
            batch["aggregate"]["address_unviolated_eligible_rows"] = json!(count);
        }
        for key in [
            "address_rival_count_sum",
            "address_eligible_rows",
            "address_unviolated_eligible_rows",
        ] {
            low_ceiling["aggregate"][key] = json!(14_287);
        }
        assert_eq!(baseline_authentication::receipt_counts(&low_ceiling)?.1, 0);
        assert!(validate_initial(&plan, &initial, &low_ceiling).is_err());
        let mut wrong = evaluation.clone();
        wrong["updates_applied_by_evaluation"] = json!(1);
        assert!(validate_initial(&plan, &initial, &wrong).is_err());
        let mut wrong = initial.clone();
        wrong.cursor.completed_batches = 1;
        assert!(validate_initial(&plan, &wrong, &evaluation).is_err());
        let mut wrong = evaluation.clone();
        wrong["model_bindings"]["objective_policy"] = json!("nearest_address_margin_v1");
        assert!(validate_initial(&plan, &initial, &wrong).is_err());
        for mutation in 0..7 {
            let mut wrong = plan.clone();
            match mutation {
                0 => wrong.config.seed = 2,
                1 => wrong.config.objective_policy = crate::ObjectivePolicy::NearestAddressMarginV1,
                2 => wrong.schema = "uor-r4.completion-owner-release-plan/1".into(),
                3 => wrong.pilot_parent = Some(qualification.initial_checkpoint.clone()),
                4 => wrong.redirect = None,
                5 => wrong.run_state_directory = directory.join("evidence"),
                _ => wrong.execution.executable_sha256 = "f".repeat(64),
            }
            assert!(validate_plan(&wrong).is_err(), "mutation {mutation}");
        }
        for mutation in 0..5 {
            let mut wrong = qualification.clone();
            match mutation {
                0 => wrong.cuda_build_exit = 1,
                1 => wrong.cuda_clippy_exit = 1,
                2 => wrong.controls[0].status = "NOT_RUN".into(),
                3 => wrong.controls[0].id = "C-R2".into(),
                _ => wrong.execution.config_sha256 = "f".repeat(64),
            }
            let evidence = plan
                .redirect
                .as_ref()
                .ok_or_else(|| invalid("test evidence"))?;
            let mut wrong_plan = plan.clone();
            let bytes = encode(&wrong)?;
            fs::write(&evidence.qualification.path, &bytes)
                .map_err(|error| invalid(error.to_string()))?;
            wrong_plan
                .redirect
                .as_mut()
                .ok_or_else(|| invalid("test evidence"))?
                .qualification
                .sha256 = corpus::sha256(&bytes);
            assert!(
                super::qualification(&wrong_plan).is_err(),
                "gate mutation {mutation}"
            );
        }
        fs::remove_dir_all(directory).map_err(|error| invalid(error.to_string()))?;
        Ok(())
    }

    #[test]
    fn owner_transport_is_unchanged_and_cannot_reuse_other_arm_or_k8_release() -> Result<()> {
        let (_, mut plan, _, _) = baseline_authentication::metadata_controls::fixture(false)?;
        plan.config = config()?;
        plan.scope = scope(&plan.config)?;
        let pin = FilePin {
            path: "/no-owner-file-created-k16-test".into(),
            sha256: "a".repeat(64),
        };
        let mut owner = CompletionOwner {
            owner: "Casey".into(),
            decision: owner_decision(&plan).into(),
            scope: plan.scope.clone(),
            plan_sha256: corpus::sha256(&encode(&plan)?),
            receipt_url: format!("owner-terminal:{}", pin.path.display()),
        };
        validate_owner(&plan, &pin, &owner)?;
        owner.decision = "RELEASE_D4_FOR_THIS_COMPLETION_SEED_ONLY".into();
        assert!(validate_owner(&plan, &pin, &owner).is_err());
        owner.decision = owner_decision(&plan).into();
        plan.config.objective_policy = crate::ObjectivePolicy::NearestAddressMarginV1;
        assert!(validate_owner(&plan, &pin, &owner).is_err());
        assert_eq!(admission_schema(&plan), "uor-r4.redirect-owner-admission/1");
        assert!(!pin.path.exists());
        Ok(())
    }
}

pub(super) fn scope(config: &DriverConfig) -> Result<String> {
    validate_redirect_config(config)?;
    let arm = match config.objective_policy {
        crate::ObjectivePolicy::NearestAddressMarginV1 => "a",
        crate::ObjectivePolicy::OccupiedCellRankHingeV1 => "r",
    };
    Ok(format!(
        "m1-admission-redirect-k16-{arm}-seed{}-checkpoint0..16404",
        config.seed
    ))
}

fn qualification(plan: &CompletionPlan) -> Result<Qualification> {
    let evidence = plan
        .redirect
        .as_ref()
        .ok_or_else(|| invalid("K16 qualification missing"))?;
    let qualification: Qualification = read_pin(&evidence.qualification)?;
    require(
        qualification.schema == "uor-r4.redirect-qualification/1"
            && qualification.scope == plan.scope
            && qualification.execution == plan.execution
            && qualification.packet_sha256 == evidence.packet.sha256,
        "qualification scope/build/packet mismatch",
    )?;
    require(
        qualification.cuda_build_exit == 0 && qualification.cuda_clippy_exit == 0,
        "CUDA build/clippy preflight failed",
    )?;
    for pin in [
        &evidence.packet,
        &qualification.cuda_build,
        &qualification.cuda_clippy,
        &qualification.timing,
    ] {
        nonempty(pin)?;
    }
    require(
        qualification.controls.len() == 6,
        "six GPU control receipts required",
    )?;
    for (index, control) in qualification.controls.iter().enumerate() {
        require(
            control.id == format!("C-R{}", index + 1) && control.status == "PASS",
            "GPU control missing, duplicated or failed",
        )?;
        nonempty(&control.receipt)?;
    }
    let initial: training::DriverCheckpoint = read_pin(&qualification.initial_checkpoint)?;
    let evaluation: Value = read_pin(&qualification.initial_evaluation)?;
    validate_initial(plan, &initial, &evaluation)?;
    Ok(qualification)
}

fn validate_initial(
    plan: &CompletionPlan,
    initial: &training::DriverCheckpoint,
    evaluation: &Value,
) -> Result<()> {
    require(
        initial.schema == training::CHECKPOINT_SCHEMA
            && initial.cursor == training::Cursor::initial()
            && initial.lineage.config == plan.config
            && initial.lineage.driver_source_sha256 == plan.execution.driver_source_sha256
            && corpus::sha256(&encode(&initial.lineage)?) == plan.execution.lineage_sha256
            && corpus::sha256(&encode(&initial.model)?) == initial.model_sha256,
        "qualified checkpoint must be exact checkpoint zero of this lineage",
    )?;
    require(
        initial.model["epoch"] == json!(0)
            && initial.model["schema"] == json!(crate::SCHEMA)
            && initial.model["seed"] == json!(plan.config.seed)
            && initial.model["spec"] == json!(plan.config.spec)
            && initial.model["read_spec"] == json!(plan.config.read_spec),
        "initial snapshot schema/seed/spec/read/epoch mismatch",
    )?;
    let objective: crate::ObjectivePolicy = initial
        .model
        .get("objective_policy")
        .map(|value| {
            serde_json::from_value(value.clone()).map_err(|error| invalid(error.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    require(
        objective == plan.config.objective_policy,
        "snapshot objective mismatch",
    )?;
    for (key, expected) in [
        ("source_sha256", &plan.execution.source_sha256),
        ("kernel_sha256", &plan.execution.kernel_sha256),
        ("geometry_sha256", &plan.execution.geometry_sha256),
    ] {
        require(
            initial.model[key] == json!(expected),
            "initial build mismatch",
        )?;
    }
    require(
        evaluation["schema"] == json!(training::DRIVER_SCHEMA)
            && evaluation["evaluation_lineage"] == json!(initial.lineage)
            && evaluation["evaluation_lineage_sha256"] == json!(plan.execution.lineage_sha256)
            && evaluation["model_epoch"] == json!(0)
            && evaluation["accepted_updates_in_model"] == json!(0)
            && evaluation["updates_applied_by_evaluation"] == json!(0)
            && evaluation["evaluation_mutates_model"] == json!(false)
            && evaluation["objective_policy"] == json!(objective)
            && evaluation["model_bindings"]["objective_policy"] == json!(objective),
        "G-Pre must be zero-update evaluation of this configuration",
    )?;
    for key in [
        "schema",
        "spec",
        "read_spec",
        "seed",
        "source_sha256",
        "kernel_sha256",
        "geometry_sha256",
        "operator_kappa",
        "composition_kappa",
    ] {
        require(
            initial.model.get(key).is_some()
                && evaluation["model_bindings"][key] == initial.model[key],
            "G-Pre model binding mismatch",
        )?;
    }
    let (_, admitted) = baseline_authentication::receipt_counts(evaluation)?;
    require(
        admitted >= 4_894,
        "G-Pre below 4894: instrument/release blocker, not mechanism negative",
    )
}

pub(super) fn validate_plan(plan: &CompletionPlan) -> Result<()> {
    require(
        plan.schema == "uor-r4.redirect-owner-release-plan/1"
            && plan.status == "HELD_OWNER_ACTION_REQUIRED"
            && plan.scope == scope(&plan.config)?
            && plan.pilot_parent.is_none()
            && plan.run_state_directory.is_absolute(),
        "plan schema/scope/fresh initialization/state directory mismatch",
    )?;
    let execution = &plan.execution;
    for digest in [
        &execution.source_sha256,
        &execution.kernel_sha256,
        &execution.geometry_sha256,
        &execution.executable_sha256,
        &execution.driver_source_sha256,
        &execution.config_sha256,
        &execution.lineage_sha256,
        &execution.sheet_sha256,
    ] {
        corpus::validate_sha256(digest)?;
    }
    require(
        execution.base_source == "2fa663b54"
            && execution.reviewed_source_commit.len() == 40
            && execution
                .reviewed_source_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            && execution.driver_source_sha256 == training::driver_source_sha256()
            && execution.config_sha256 == corpus::sha256(&encode(&plan.config)?)
            && execution.sheet_sha256 == plan.sheet.sha256,
        "source/config/sheet identity mismatch",
    )?;
    let sheet = plan.sheet.read()?;
    let sheet = std::str::from_utf8(&sheet).map_err(|error| invalid(error.to_string()))?;
    require(
        sheet
            .lines()
            .any(|line| line.starts_with("TEST FITNESS: FIT"))
            && sheet.contains("discrete-rung0-admission-redirect"),
        "redirect FIT sheet missing",
    )?;
    let evidence = plan
        .redirect
        .as_ref()
        .ok_or_else(|| invalid("redirect evidence required"))?;
    require(
        plan.claude_review == evidence.pm_review,
        "review pin must be the scoped PM qualification acceptance",
    )?;
    let review: QualificationReview = read_pin(&evidence.pm_review)?;
    require(
        review.reviewer == "Kimi"
            && review.verdict == "APPROVE_K16_REDIRECT_QUALIFIED_ADMISSION"
            && review.scope == plan.scope
            && review.execution == plan.execution
            && review.qualification_sha256 == evidence.qualification.sha256
            && review.packet_sha256 == evidence.packet.sha256
            && valid_receipt_url(&review.receipt_url)
            && valid_receipt_url(&review.exact_source_review),
        "PM must accept exact-source review, completed packet and qualification for this arm/seed",
    )?;
    let qualification = qualification(plan)?;
    for pin in [
        &evidence.packet,
        &evidence.qualification,
        &evidence.pm_review,
        &plan.sheet,
        &qualification.initial_checkpoint,
        &qualification.initial_evaluation,
        &qualification.cuda_build,
        &qualification.cuda_clippy,
        &qualification.timing,
    ]
    .into_iter()
    .chain(
        qualification
            .controls
            .iter()
            .map(|control| &control.receipt),
    ) {
        let directory = pin
            .path
            .parent()
            .ok_or_else(|| invalid("evidence directory missing"))?;
        require_disjoint_directories(&plan.run_state_directory, directory)?;
    }
    Ok(())
}

#[cfg(feature = "cuda")]
pub(super) fn require_initial(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    plan: &CompletionPlan,
    current: &CompletionCheckpoint,
) -> Result<()> {
    let qualification = qualification(plan)?;
    let qualified: training::DriverCheckpoint = read_pin(&qualification.initial_checkpoint)?;
    let evaluation: Value = read_pin(&qualification.initial_evaluation)?;
    let initial_pin: FilePin =
        read_json(&release.manifest.run_state_directory.join("initial.json"))?;
    let initial: CompletionCheckpoint = read_pin(&initial_pin)?;
    validate_progress(driver, release, &initial)?;
    require(
        encode(&initial.checkpoint)? == encode(&qualified)?
            && initial.baseline_evaluation == evaluation
            && current.baseline_evaluation == evaluation,
        "runtime initialization/evaluation differs from qualified checkpoint zero",
    )
}
