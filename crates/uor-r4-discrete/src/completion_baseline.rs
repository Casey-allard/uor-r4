//! Fresh baseline provenance and stored-receipt consistency only. No model arithmetic.
//! Authentication binds artifacts; it is not proof that GPU execution occurred.
use super::*;
use std::collections::BTreeMap;

const REFERENCE_SHA256: &str = "b92da0bf1ccda1c9de56a33de41d4698dc214bd71c588a6938a87443b9aca0b9";
const PANEL_ROWS: u64 = 14_287;
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(invalid(format!("seed2 baseline: {message}")))
    }
}
fn number(v: &Value, key: &str) -> Result<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("seed2 baseline missing integer {key}")))
}
fn digest(v: &impl Serialize) -> Result<String> {
    Ok(corpus::sha256(&encode(v)?))
}
fn sum_into(sums: &mut BTreeMap<&'static str, u64>, key: &'static str, value: u64) -> Result<()> {
    let entry = sums.entry(key).or_default();
    *entry = entry
        .checked_add(value)
        .ok_or_else(|| invalid("baseline receipt sum overflow"))?;
    Ok(())
}
fn aggregate(sums: &BTreeMap<&str, u64>, actual: &Value) -> Result<()> {
    require(
        number(actual, "loss_denominator_per_row")? == 256,
        "loss denominator",
    )?;
    for (key, value) in sums {
        require(number(actual, key)? == *value, "row/aggregate mismatch")?;
    }
    Ok(())
}
/// Recounts already measured row observations; never evaluates a model or predictor.
pub(super) fn receipt_counts(e: &Value) -> Result<(u64, u64)> {
    let batches = e["batches"]
        .as_array()
        .ok_or_else(|| invalid("baseline batches missing"))?;
    require(batches.len() == 224, "DEV batch inventory")?;
    let mut total = BTreeMap::new();
    let mut admitted = 0u64;
    for (index, batch) in batches.iter().enumerate() {
        let rows = batch["rows"]
            .as_array()
            .ok_or_else(|| invalid("baseline rows missing"))?;
        require(
            rows.len() == if index == 223 { 15 } else { 64 },
            "DEV batch shape",
        )?;
        let mut sums = BTreeMap::new();
        for row in rows {
            let t = number(row, "token_hinge")?;
            let a = number(row, "address_hinge")?;
            let loss = number(row, "total_loss_numerator")?;
            let tr = number(row, "token_rivals")?;
            let ar = number(row, "address_rivals")?;
            let cells = number(row, "admitted_cells")?;
            let h1 = number(row, "top1_correct")?;
            let h8 = number(row, "top8_correct")?;
            let target_admitted = cells
                .checked_sub(ar)
                .ok_or_else(|| invalid("baseline address rival underflow"))?;
            require(
                (1..=16).contains(&cells) && target_admitted <= 1 && tr <= 4095,
                "admission bounds",
            )?;
            require(h1 <= h8 && h8 <= target_admitted, "rank/admission nesting")?;
            require(
                (8..=4096).contains(&(tr + target_admitted)),
                "admitted token inventory",
            )?;
            let strict = tr > 0 && t < 32;
            require(
                !(strict && target_admitted == 1) || h1 == 1,
                "admitted strict win must be top1",
            )?;
            let weighted = t
                .checked_mul(8)
                .and_then(|x| a.checked_mul(32).and_then(|y| x.checked_add(y)))
                .ok_or_else(|| invalid("baseline loss overflow"))?;
            require(loss == weighted, "weighted row loss")?;
            let top = row["top8"]
                .as_array()
                .ok_or_else(|| invalid("baseline ranked IDs missing"))?;
            let mut seen = std::collections::BTreeSet::new();
            require(
                top.len() == 8
                    && top
                        .iter()
                        .all(|v| v.as_u64().is_some_and(|id| id < 4096 && seen.insert(id))),
                "ranked identities",
            )?;
            admitted = admitted
                .checked_add(target_admitted)
                .ok_or_else(|| invalid("baseline admission overflow"))?;
            for (key, value) in [
                ("scored_rows", 1),
                ("top1_correct", h1),
                ("top8_correct", h8),
                ("token_hinge_sum", t),
                ("address_hinge_sum", a),
                ("total_loss_numerator_sum", loss),
                ("token_eligible_rows", u64::from(tr > 0)),
                ("address_eligible_rows", u64::from(ar > 0)),
                ("token_rival_count_sum", tr),
                ("address_rival_count_sum", ar),
                ("admitted_cell_count_sum", cells),
                ("eligible_any_rows", u64::from(tr > 0 || ar > 0)),
                (
                    "eligible_unviolated_rows",
                    u64::from(loss == 0 && (tr > 0 || ar > 0)),
                ),
                (
                    "token_unviolated_eligible_rows",
                    u64::from(tr > 0 && t == 0),
                ),
                (
                    "address_unviolated_eligible_rows",
                    u64::from(ar > 0 && a == 0),
                ),
                ("token_strict_win_eligible_rows", u64::from(strict)),
            ] {
                sum_into(&mut sums, key, value)?;
                sum_into(&mut total, key, value)?;
            }
        }
        aggregate(&sums, &batch["aggregate"])?;
    }
    aggregate(&total, &e["aggregate"])?;
    require(
        total.get("scored_rows") == Some(&PANEL_ROWS),
        "panel row total",
    )?;
    Ok((
        *total
            .get("token_strict_win_eligible_rows")
            .ok_or_else(|| invalid("baseline strict sum missing"))?,
        admitted,
    ))
}

pub(super) fn authenticate(
    p: &CompletionCheckpoint,
    plan: &CompletionPlan,
    initial_pin: &FilePin,
    admission_sha256: &str,
    typed_snapshot_sha256: &str,
) -> Result<Value> {
    authenticate_with_reference(
        p,
        plan,
        initial_pin,
        admission_sha256,
        typed_snapshot_sha256,
        REFERENCE_SHA256,
    )
}
fn authenticate_with_reference(
    p: &CompletionCheckpoint,
    plan: &CompletionPlan,
    initial_pin: &FilePin,
    admission_sha256: &str,
    typed_snapshot_sha256: &str,
    reference_sha256: &str,
) -> Result<Value> {
    let c = &p.checkpoint;
    let e = &p.baseline_evaluation;
    require(
        (plan.config.seed == 2 || plan.config.spec.kept_cells == 16)
            && plan.pilot_parent.is_none()
            && p.pilot_parent.is_none(),
        "fresh seed 2 or registered K16 redirect only",
    )?;
    require(
        p.schema == "uor-r4.completion-checkpoint/1" && c.schema == training::CHECKPOINT_SCHEMA,
        "checkpoint schema",
    )?;
    require(
        c.cursor == training::initial_cursor_for(&plan.config)
            && p.vector == KindCounters::default()
            && p.route == KindCounters::default(),
        "checkpoint zero/counters",
    )?;
    require(
        p.admission_sha256 == admission_sha256 && initial_pin.sha256 == digest(p)?,
        "initial/admission digest",
    )?;
    require(
        c.lineage.schema == training::DRIVER_SCHEMA
            && c.lineage.config == plan.config
            && c.lineage.driver_source_sha256 == plan.execution.driver_source_sha256,
        "lineage/configuration",
    )?;
    require(
        digest(&c.lineage)? == plan.execution.lineage_sha256
            && digest(&plan.config)? == plan.execution.config_sha256,
        "lineage/configuration digest",
    )?;
    require(
        digest(&c.model)? == c.model_sha256
            && c.model["schema"] == json!(crate::SCHEMA)
            && c.model["epoch"] == json!(0),
        "checkpoint model storage",
    )?;
    require(
        e["schema"] == json!(training::DRIVER_SCHEMA)
            && e["scored_set_policy"] == json!(corpus::WINDOW_POLICY),
        "evaluation schema/policy",
    )?;
    require(
        e["evaluation_lineage"]
            == serde_json::to_value(&c.lineage).map_err(|x| invalid(x.to_string()))?
            && e["evaluation_lineage_sha256"] == json!(plan.execution.lineage_sha256),
        "evaluation lineage",
    )?;
    require(
        e["model_epoch"] == json!(0)
            && e["accepted_updates_in_model"] == json!(0)
            && e["updates_applied_by_evaluation"] == json!(0)
            && e["evaluation_mutates_model"] == json!(false),
        "evaluation must precede apply",
    )?;
    require(
        e["model_snapshot_sha256"] == json!(typed_snapshot_sha256),
        "typed snapshot digest",
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
            e["model_bindings"].get(key).is_some() && e["model_bindings"][key] == c.model[key],
            "model bindings",
        )?;
    }
    for (key, expected) in [
        ("source_sha256", &plan.execution.source_sha256),
        ("kernel_sha256", &plan.execution.kernel_sha256),
        ("geometry_sha256", &plan.execution.geometry_sha256),
    ] {
        require(
            c.model[key] == json!(expected),
            "qualified numerical source binding",
        )?;
    }
    require(
        c.model["seed"] == json!(plan.config.seed)
            && c.model["spec"] == json!(plan.config.spec)
            && c.model["read_spec"] == json!(plan.config.read_spec),
        "seed/spec/read binding",
    )?;
    require(
        training::snapshot_metadata(&c.model)? == p.baseline_snapshot_metadata,
        "snapshot metadata",
    )?;
    require(
        digest(&e["count_reference"])? == reference_sha256,
        "frozen full masked-TRAIN reference/DEV diagnostic digest",
    )?;
    let (u, b3) = receipt_counts(e)?;
    Ok(
        json!({"schema":if plan.config.spec.kept_cells == 16 {"uor-r4.redirect-baseline-authentication/1"} else {"uor-r4.completion-seed2-baseline-authentication/1"},
        "status":"AUTHENTICATED_METADATA_BEFORE_APPLY", "seed":plan.config.seed,
        "scope":plan.scope,"plan_sha256":digest(plan)?,"admission_sha256":admission_sha256,
        "initial_checkpoint":initial_pin,"driver_checkpoint_sha256":digest(c)?,
        "lineage_sha256":plan.execution.lineage_sha256,"baseline_evaluation_sha256":digest(e)?,
        "model_storage_sha256":c.model_sha256,"typed_snapshot_sha256":typed_snapshot_sha256,
        "count_reference_sha256":reference_sha256,"reference_origin":"authenticated frozen pilot reference; identical TRAIN/DEV inputs; pilot u never reused",
        "u":{"numerator":u,"denominator":PANEL_ROWS,"definition":"token_rivals > 0 && token_hinge < 32; admission not required","instrument_invalid":100*u>95*PANEL_ROWS},
        "target_admitted_rows":b3,"evidence_scope":"artifact provenance and stored receipt consistency; not proof of GPU execution", "release_effect":"NONE"}),
    )
}

/// Only a verified checkpoint-zero invocation may recover an interrupted publication.
fn publish_or_verify(path: &Path, expected: &Value, completed: u64) -> Result<()> {
    let pin_path = path.with_extension("pin.json");
    let bytes = encode(expected)?;
    if path.exists() {
        require(
            fs::read(path).map_err(|e| invalid(e.to_string()))? == bytes,
            "authentication receipt changed",
        )?;
    } else {
        require(
            completed == 0,
            "progressed resume missing baseline authentication",
        )?;
        publish_state_json(path, expected, false)?;
    }
    let expected_pin = FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&bytes),
    };
    if pin_path.exists() {
        let pin: FilePin = read_json(&pin_path)?;
        require(
            pin == expected_pin && pin.read()? == bytes,
            "authentication FilePin mismatch",
        )?;
    } else {
        require(
            completed == 0,
            "progressed resume missing authentication pin",
        )?;
        publish_state_json(&pin_path, &expected_pin, false)?;
    }
    Ok(())
}

#[cfg(feature = "cuda")]
pub(super) fn require_before_apply(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    plan: &CompletionPlan,
    current: &CompletionCheckpoint,
) -> Result<()> {
    if driver.config().seed != 2 && driver.config().spec.kept_cells != 16 {
        return Ok(());
    }
    let state = &release.manifest.run_state_directory;
    let initial_pin: FilePin = read_json(&state.join("initial.json"))?;
    let initial: CompletionCheckpoint =
        serde_json::from_slice(&initial_pin.read()?).map_err(|e| invalid(e.to_string()))?;
    validate_progress(driver, release, &initial)?;
    require(
        current.baseline_evaluation == initial.baseline_evaluation
            && current.baseline_snapshot_metadata == initial.baseline_snapshot_metadata
            && current.pilot_parent == initial.pilot_parent,
        "current checkpoint replaced original baseline",
    )?;
    // Storage decoding/serialization only: no CUDA context, constructor, forward, or arithmetic.
    let snapshot: crate::device::Snapshot =
        serde_json::from_value(initial.checkpoint.model.clone())
            .map_err(|e| invalid(e.to_string()))?;
    let receipt = authenticate(
        &initial,
        plan,
        &initial_pin,
        &release.manifest_sha256,
        &digest(&snapshot)?,
    )?;
    publish_or_verify(
        &state.join("baseline-authentication.json"),
        &receipt,
        current.checkpoint.cursor.completed_batches,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentication_publication_is_immutable_and_resume_fails_closed() -> Result<()> {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../local")
            .join(format!(
                "uor-seed2-auth-{}-{}",
                std::process::id(),
                unix_ns()?
            ));
        fs::create_dir(&directory).map_err(|e| invalid(e.to_string()))?;
        let path = directory.join("baseline-authentication.json");
        let receipt = json!({"fixture":"storage-only"});
        assert!(publish_or_verify(&path, &receipt, 1).is_err());
        publish_or_verify(&path, &receipt, 0)?;
        publish_or_verify(&path, &receipt, 64)?;
        assert!(publish_or_verify(&path, &json!({"fixture":"changed"}), 64).is_err());
        fs::remove_file(path.with_extension("pin.json")).map_err(|e| invalid(e.to_string()))?;
        assert!(publish_or_verify(&path, &receipt, 64).is_err());
        publish_or_verify(&path, &receipt, 0)?;
        fs::remove_dir_all(directory).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod metadata_controls {
    use super::*;
    pub(crate) fn fixture(
        strict: bool,
    ) -> Result<(CompletionCheckpoint, CompletionPlan, FilePin, String)> {
        // Synthetic stored observations and storage bytes, not a CPU reference or model.
        let mut plan: CompletionPlan = serde_json::from_str(include_str!(
            "../../../docs/labs/rung0-completion-release-2026-10-14/SEED2_PLAN.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let summary: Value = serde_json::from_str(include_str!(
            "../../../docs/labs/a2-pilot-2026-10-14/report-summary.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let mut lineage: training::Lineage =
            serde_json::from_value(summary["baseline_evaluation"]["evaluation_lineage"].clone())
                .map_err(|e| invalid(e.to_string()))?;
        lineage.config = plan.config.clone();
        lineage.driver_source_sha256 = training::driver_source_sha256();
        plan.execution.driver_source_sha256 = lineage.driver_source_sha256.clone();
        plan.execution.lineage_sha256 = digest(&lineage)?;
        let model = json!({"schema":crate::SCHEMA,"epoch":0,"seed":2,"spec":plan.config.spec,"read_spec":plan.config.read_spec,
            "table":vec![0;4096*32],"routes":[],"gates":[],"operator_kappa":"metadata-fixture","composition_kappa":"metadata-fixture",
            "source_sha256":plan.execution.source_sha256,"kernel_sha256":plan.execution.kernel_sha256,"geometry_sha256":plan.execution.geometry_sha256});
        let t = if strict { 0 } else { 32 };
        let hit = u64::from(strict);
        let row = json!({"token_hinge":t,"address_hinge":0,"total_loss_numerator":8*t,"token_rivals":8,"address_rivals":0,"admitted_cells":1,"top1_correct":hit,"top8_correct":hit,"top8":[0,1,2,3,4,5,6,7]});
        let sums = |n: u64| json!({"scored_rows":n,"top1_correct":n*hit,"top8_correct":n*hit,"token_hinge_sum":n*t,"address_hinge_sum":0,"total_loss_numerator_sum":8*n*t,"token_eligible_rows":n,"address_eligible_rows":0,"token_rival_count_sum":8*n,"address_rival_count_sum":0,"admitted_cell_count_sum":n,"eligible_any_rows":n,"eligible_unviolated_rows":n*hit,"token_unviolated_eligible_rows":n*hit,"address_unviolated_eligible_rows":0,"token_strict_win_eligible_rows":n*hit,"loss_denominator_per_row":256});
        let batches: Vec<Value> = (0..224)
            .map(|i| {
                let n = if i == 223 { 15 } else { 64 };
                json!({"rows":vec![row.clone();n],"aggregate":sums(n as u64)})
            })
            .collect();
        let reference = json!({"fixture":"not a measured count predictor","token_counts":[1,2,3]});
        let reference_sha = digest(&reference)?;
        let mut bindings = serde_json::Map::new();
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
            bindings.insert(key.into(), model[key].clone());
        }
        let baseline = json!({"schema":training::DRIVER_SCHEMA,"scored_set_policy":corpus::WINDOW_POLICY,"model_epoch":0,"accepted_updates_in_model":0,"updates_applied_by_evaluation":0,"evaluation_mutates_model":false,
            "evaluation_lineage":lineage,"evaluation_lineage_sha256":plan.execution.lineage_sha256,"model_bindings":bindings,"model_snapshot_sha256":"b".repeat(64),"count_reference":reference,"batches":batches,"aggregate":sums(PANEL_ROWS)});
        let p = CompletionCheckpoint {
            schema: "uor-r4.completion-checkpoint/1".into(),
            admission_sha256: "a".repeat(64),
            pilot_parent: None,
            baseline_snapshot_metadata: training::snapshot_metadata(&model)?,
            baseline_evaluation: baseline,
            checkpoint: training::DriverCheckpoint {
                schema: training::CHECKPOINT_SCHEMA.into(),
                lineage,
                cursor: training::initial_cursor_for(&plan.config),
                model_sha256: digest(&model)?,
                model,
            },
            vector: KindCounters::default(),
            route: KindCounters::default(),
        };
        let pin = FilePin {
            path: "/fixture/checkpoint-00000.json".into(),
            sha256: digest(&p)?,
        };
        Ok((p, plan, pin, reference_sha))
    }
    fn check(
        p: &CompletionCheckpoint,
        plan: &CompletionPlan,
        pin: &FilePin,
        reference: &str,
    ) -> Result<Value> {
        authenticate_with_reference(p, plan, pin, &"a".repeat(64), &"b".repeat(64), reference)
    }
    #[test]
    fn zero_baseline_binds_model_rows_reference_and_seed_without_execution() -> Result<()> {
        let (p, plan, pin, reference) = fixture(false)?;
        let receipt = check(&p, &plan, &pin, &reference)?;
        assert_eq!(receipt["u"]["numerator"], json!(0));
        assert_eq!(receipt["u"]["denominator"], json!(14287));
        assert_eq!(receipt["u"]["instrument_invalid"], json!(false));
        assert!(authenticate(&p, &plan, &pin, &"a".repeat(64), &"b".repeat(64)).is_err());
        for mutation in 0..10 {
            let mut wrong: CompletionCheckpoint =
                serde_json::from_slice(&encode(&p)?).map_err(|e| invalid(e.to_string()))?;
            match mutation {
                0 => wrong.checkpoint.cursor.next_coordinate = 1,
                1 => wrong.checkpoint.model["epoch"] = json!(1),
                2 => wrong.baseline_evaluation["model_bindings"]["seed"] = json!(1),
                3 => wrong.baseline_evaluation["model_snapshot_sha256"] = json!("c".repeat(64)),
                4 => {
                    wrong.baseline_evaluation["count_reference"]["token_counts"] = json!([2, 1, 3])
                }
                5 => {
                    wrong.baseline_evaluation["aggregate"]["token_strict_win_eligible_rows"] =
                        json!(1)
                }
                6 => wrong.baseline_evaluation["batches"][0]["rows"][0]["token_hinge"] = json!(31),
                7 => wrong.baseline_evaluation["batches"]
                    .as_array_mut()
                    .ok_or_else(|| invalid("fixture batches"))?
                    .truncate(223),
                8 => wrong.route.attempted = 1,
                _ => wrong.baseline_evaluation["evaluation_mutates_model"] = json!(true),
            }
            let wrong_pin = FilePin {
                path: pin.path.clone(),
                sha256: digest(&wrong)?,
            };
            assert!(
                check(&wrong, &plan, &wrong_pin, &reference).is_err(),
                "mutation {mutation}"
            );
        }
        Ok(())
    }
    #[test]
    fn high_u_is_authenticated_but_instrument_invalid_not_a_model_negative() -> Result<()> {
        let (p, plan, pin, reference) = fixture(true)?;
        let receipt = check(&p, &plan, &pin, &reference)?;
        assert_eq!(receipt["u"]["numerator"], json!(14287));
        assert_eq!(receipt["u"]["instrument_invalid"], json!(true));
        Ok(())
    }
    #[test]
    fn stored_k16_baseline_rows_are_admitted_without_model_execution() -> Result<()> {
        let (checkpoint, _, _, _) = fixture(false)?;
        let mut evaluation = checkpoint.baseline_evaluation;
        let update_aggregate = |aggregate: &mut Value, rows: u64| {
            aggregate["token_rival_count_sum"] = json!(31 * rows);
            aggregate["address_rival_count_sum"] = json!(15 * rows);
            aggregate["admitted_cell_count_sum"] = json!(16 * rows);
            aggregate["address_eligible_rows"] = json!(rows);
            aggregate["address_unviolated_eligible_rows"] = json!(rows);
        };
        for batch in evaluation["batches"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture batches"))?
        {
            let rows = batch["rows"]
                .as_array_mut()
                .ok_or_else(|| invalid("fixture rows"))?;
            for row in rows.iter_mut() {
                row["token_rivals"] = json!(31);
                row["address_rivals"] = json!(15);
                row["admitted_cells"] = json!(16);
            }
            let count = rows.len() as u64;
            update_aggregate(&mut batch["aggregate"], count);
        }
        update_aggregate(&mut evaluation["aggregate"], PANEL_ROWS);
        assert!(receipt_counts(&evaluation).is_ok());
        evaluation["batches"][0]["rows"][0]["admitted_cells"] = json!(17);
        assert!(receipt_counts(&evaluation).is_err());
        Ok(())
    }
    #[test]
    #[ignore = "requires UOR_BASELINE_PILOT_REPORT pointing at the authenticated retained artifact"]
    fn reference_pin_matches_authenticated_pilot_when_supplied() -> Result<()> {
        let path = std::env::var_os("UOR_BASELINE_PILOT_REPORT")
            .ok_or_else(|| invalid("UOR_BASELINE_PILOT_REPORT required"))?;
        let bytes = fs::read(path).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(
            corpus::sha256(&bytes),
            "e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3"
        );
        let pilot: Value = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(
            digest(&pilot["baseline_evaluation"]["count_reference"])?,
            REFERENCE_SHA256
        );
        let (u, _) = receipt_counts(&pilot["baseline_evaluation"])?;
        assert_eq!(u, 105);
        Ok(())
    }
    #[test]
    fn typed_and_value_serialization_are_distinct_digest_domains() -> Result<()> {
        // Storage-only representative field order. The real CUDA-gated Snapshot
        // deserialization seam still requires source/build qualification.
        #[derive(Serialize, Deserialize)]
        struct Stored {
            schema: String,
            epoch: u64,
        }
        let original = Stored {
            schema: "metadata-fixture".into(),
            epoch: 0,
        };
        let value = serde_json::to_value(&original).map_err(|e| invalid(e.to_string()))?;
        assert_ne!(digest(&original)?, digest(&value)?);
        let restored: Stored = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(digest(&original)?, digest(&restored)?);
        Ok(())
    }
}
