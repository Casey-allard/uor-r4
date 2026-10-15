//! Stage-A derived evaluation only. No proposal, apply, resume or release path.
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "cuda"), allow(dead_code))]
use serde::Serialize;
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};
use uor_r4_discrete::{corpus, training, Error, Result};

// Reuse the report protocol without compiling the integer model runtime.
#[allow(dead_code)]
#[path = "../../../../../crates/uor-r4-integer/src/report_output.rs"]
mod report_output;

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
fn encode<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|e| invalid(e.to_string()))
}
fn number(v: &Value, key: &str) -> Result<u64> {
    v[key]
        .as_u64()
        .ok_or_else(|| invalid(format!("missing unsigned integer {key}")))
}
fn require(ok: bool, reason: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(invalid(reason))
    }
}
#[derive(Clone, Copy)]
struct Quarter {
    cursor: u64,
    sha: &'static str,
    counts: [u64; 5],
}
const QUARTERS: [Quarter; 4] = [
    Quarter {
        cursor: 4101,
        sha: "a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd",
        counts: [96, 1090, 177, 41, 2201],
    },
    Quarter {
        cursor: 8202,
        sha: "dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822",
        counts: [94, 967, 178, 42, 2131],
    },
    Quarter {
        cursor: 12303,
        sha: "102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982",
        counts: [96, 953, 214, 44, 1954],
    },
    Quarter {
        cursor: 16404,
        sha: "4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf",
        counts: [149, 747, 105, 65, 1920],
    },
];
fn quarter_for_hash(hash: &str) -> Result<Quarter> {
    QUARTERS
        .iter()
        .find(|q| q.sha == hash)
        .copied()
        .ok_or_else(|| {
            invalid("checkpoint is not one of the four sheet-pinned seed-1 quarter artifacts")
        })
}
fn read_checkpoint(path: &Path) -> Result<(Quarter, training::DriverCheckpoint)> {
    let bytes = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let quarter = quarter_for_hash(&corpus::sha256(&bytes))?;
    let envelope: Value = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    require(
        envelope["schema"] == "uor-r4.completion-checkpoint/1",
        "wrong completion envelope schema",
    )?;
    let checkpoint: training::DriverCheckpoint =
        serde_json::from_value(envelope["checkpoint"].clone())
            .map_err(|e| invalid(e.to_string()))?;
    require(
        checkpoint.cursor.completed_batches == quarter.cursor,
        "quarter cursor mismatch",
    )?;
    require(
        checkpoint.lineage.config.seed == 1 && checkpoint.lineage.config.spec.kept_cells == 8,
        "Stage A requires the frozen seed-1 K=8 checkpoint",
    )?;
    Ok((quarter, checkpoint))
}
/// Change metadata in a derived copy only. This never rewrites a sealed checkpoint.
fn derive_checkpoint(
    original: &training::DriverCheckpoint,
    kept: u32,
) -> Result<training::DriverCheckpoint> {
    require(
        (8..=16).contains(&kept),
        "Stage A kept cells must be 8..=16",
    )?;
    require(
        corpus::sha256(&encode(&original.model)?) == original.model_sha256,
        "original checkpoint model hash mismatch",
    )?;
    let mut derived = original.clone();
    derived.lineage.config.spec.kept_cells = kept;
    derived.lineage.config.spec.validate()?;
    derived.model["spec"]["kept_cells"] = json!(kept);
    derived.model_sha256 = corpus::sha256(&encode(&derived.model)?);
    // Driver hash is operational provenance, not a model-weight/source rewrite.
    // The original hash and both lineages are retained in every derived record.
    derived.lineage.driver_source_sha256 = training::driver_source_sha256();
    Ok(derived)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Metrics {
    scored_rows: u64,
    top1_correct: u64,
    top8_correct: u64,
    strict_win_rows: u64,
    admitted_strict_win_rows: u64,
    target_admitted_rows: u64,
    admitted_cell_count_sum: u64,
    address_rival_rows: u64,
}
/// Audit stored GPU observations only; no host vector, distance or model reference.
fn summarize(evaluation: &Value, kept: u32) -> Result<(Metrics, Vec<bool>)> {
    let batches = evaluation["batches"]
        .as_array()
        .ok_or_else(|| invalid("missing batches"))?;
    require(
        batches.len() == 224,
        "expected all 224 frozen-panel batches",
    )?;
    let mut m = Metrics {
        scored_rows: 0,
        top1_correct: 0,
        top8_correct: 0,
        strict_win_rows: 0,
        admitted_strict_win_rows: 0,
        target_admitted_rows: 0,
        admitted_cell_count_sum: 0,
        address_rival_rows: 0,
    };
    let mut admitted = Vec::with_capacity(14287);
    for (i, batch) in batches.iter().enumerate() {
        let rows = batch["rows"]
            .as_array()
            .ok_or_else(|| invalid("missing rows"))?;
        require(
            rows.len() == if i == 223 { 15 } else { 64 },
            "frozen-panel batch shape differs",
        )?;
        for row in rows {
            let cells = number(row, "admitted_cells")?;
            let rivals = number(row, "address_rivals")?;
            let target = cells
                .checked_sub(rivals)
                .filter(|x| *x <= 1)
                .ok_or_else(|| invalid("invalid B3 target admission difference"))?;
            let one = number(row, "top1_correct")?;
            let eight = number(row, "top8_correct")?;
            require(
                (1..=u64::from(kept)).contains(&cells) && one <= eight && eight <= target,
                "invalid admitted count or B3 rank nesting",
            )?;
            let strict = number(row, "token_rivals")? > 0 && number(row, "token_hinge")? < 32;
            require(
                !strict || target == 0 || one == 1,
                "admitted strict win is not top1",
            )?;
            m.scored_rows += 1;
            m.top1_correct += one;
            m.top8_correct += eight;
            m.strict_win_rows += u64::from(strict);
            m.admitted_strict_win_rows += u64::from(strict) * target;
            m.target_admitted_rows += target;
            m.admitted_cell_count_sum += cells;
            m.address_rival_rows += u64::from(rivals > 0);
            admitted.push(target == 1);
        }
    }
    for (key, expected) in [
        ("scored_rows", m.scored_rows),
        ("top1_correct", m.top1_correct),
        ("top8_correct", m.top8_correct),
        ("token_strict_win_eligible_rows", m.strict_win_rows),
        ("admitted_cell_count_sum", m.admitted_cell_count_sum),
        ("address_eligible_rows", m.address_rival_rows),
    ] {
        require(
            number(&evaluation["aggregate"], key)? == expected,
            "GPU aggregate and stored rows disagree",
        )?;
    }
    require(m.scored_rows == 14287, "panel cardinality mismatch")?;
    Ok((m, admitted))
}
fn qualify_k8(q: Quarter, metrics: &Metrics) -> Result<()> {
    require([metrics.top1_correct, metrics.top8_correct, metrics.strict_win_rows,
        metrics.admitted_strict_win_rows, metrics.target_admitted_rows] == q.counts,
        "INSTRUMENT_FAULT: K=8 does not exactly reproduce sealed quarter metrics; wider K forbidden")
}
fn write_new(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::File::create_new(path).map_err(|e| invalid(e.to_string()))?;
    file.write_all(&encode(value)?)
        .and_then(|_| file.sync_all())
        .map_err(|e| invalid(e.to_string()))
}
fn admit_output(input: &Path, output: &Path) -> Result<()> {
    let input_parent = fs::canonicalize(input.parent().unwrap_or(Path::new(".")))
        .map_err(|e| invalid(e.to_string()))?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|e| invalid(format!("output parent must exist: {e}")))?;
    require(
        !parent.starts_with(&input_parent),
        "derived output cannot be beneath the checkpoint directory",
    )?;
    for ancestor in parent.ancestors() {
        require(
            !ancestor.join(report_output::MANIFEST_FILE).exists()
                && !ancestor.join(report_output::ATTEMPT_FILE).exists(),
            "derived output cannot be beneath an existing report root",
        )?;
    }
    Ok(())
}
fn instrument_identity() -> Result<Value> {
    let executable = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
    Ok(json!({"schema":"uor-r4.stage-a-instrument/1",
        "instrument_source_sha256":corpus::sha256(include_bytes!("main.rs")),
        "instrument_manifest_sha256":corpus::sha256(include_bytes!("../Cargo.toml")),
        "instrument_lock_sha256":corpus::sha256(include_bytes!("../Cargo.lock")),
        "report_output_source_sha256":corpus::sha256(include_bytes!("../../../../../crates/uor-r4-integer/src/report_output.rs")),
        "driver_source_sha256":training::driver_source_sha256(),
        "executable_sha256":corpus::sha256(&fs::read(executable).map_err(|e|invalid(e.to_string()))?),
        "training":"HELD","model_updates":0}))
}
const ROW_COLUMNS: [&str; 9] = [
    "top8",
    "token_hinge",
    "address_hinge",
    "total_loss_numerator",
    "token_rivals",
    "address_rivals",
    "top1_correct",
    "top8_correct",
    "admitted_cells",
];
/// Lossless column naming once per file, keeping every GPU observation under the
/// sheet's 50 MiB storage budget. No result is recomputed by this encoding.
fn compact_evaluation(mut evaluation: Value) -> Result<Value> {
    let batches = evaluation
        .as_object_mut()
        .and_then(|v| v.remove("batches"))
        .ok_or_else(|| invalid("missing batches"))?;
    let batches = batches
        .as_array()
        .ok_or_else(|| invalid("batches must be array"))?;
    let mut rows = Vec::new();
    let mut lengths = Vec::new();
    let mut aggregates = Vec::new();
    for batch in batches {
        require(
            batch.as_object().is_some_and(|b| {
                b.len() == 2 && b.contains_key("rows") && b.contains_key("aggregate")
            }),
            "unexpected batch schema would lose fields",
        )?;
        let source = batch["rows"]
            .as_array()
            .ok_or_else(|| invalid("missing rows"))?;
        lengths.push(source.len());
        aggregates.push(batch["aggregate"].clone());
        for row in source {
            require(
                row.as_object()
                    .is_some_and(|r| r.len() == ROW_COLUMNS.len()),
                "unexpected row schema would lose fields",
            )?;
            rows.push(
                ROW_COLUMNS
                    .iter()
                    .map(|key| {
                        row.get(*key)
                            .cloned()
                            .ok_or_else(|| invalid("missing observation column"))
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        }
    }
    evaluation["row_columns"] = json!(ROW_COLUMNS);
    evaluation["batch_row_counts"] = json!(lengths);
    evaluation["batch_aggregates"] = json!(aggregates);
    evaluation["rows"] = json!(rows);
    Ok(evaluation)
}
fn monotone(previous: &[bool], current: &[bool]) -> Result<()> {
    require(
        previous.len() == current.len()
            && previous.iter().zip(current).all(|(was, now)| !was || *now),
        "INSTRUMENT_FAULT: target admission is not monotone with K",
    )
}
#[cfg(feature = "cuda")]
fn sweep(input: &Path, output: &Path) -> Result<Value> {
    use uor_r4_discrete::training::PreparedDriver;
    let mut quarters = Vec::new();
    for q in QUARTERS {
        let (actual, original) =
            read_checkpoint(&input.join(format!("checkpoint-{:05}.json", q.cursor)))?;
        require(
            actual.cursor == q.cursor,
            "checkpoint filename/pin mismatch",
        )?;
        let original_driver = PreparedDriver::load(original.lineage.config.clone())?;
        original_driver.validate_checkpoint(&derive_checkpoint(&original, 8)?)?;
        require(
            original_driver.config().spec.blocks == 4
                && original_driver.config().spec.vocabulary == 4096
                && original_driver.config().spec.context == 192
                && original_driver.config().batch_rows == 64,
            "wrong frozen quarter configuration",
        )?;
        quarters.push((q, original));
    }
    require(
        quarters
            .iter()
            .all(|(_, c)| c.lineage == quarters[0].1.lineage),
        "quarter lineage mismatch",
    )?;
    let mut identity = instrument_identity()?;
    identity["compiled"] = uor_r4_discrete::device::compiled_identity();
    let mut summaries = Vec::new();
    let mut previous = vec![vec![false; 14287]; 4];
    let mut reference = None;
    let mut written = 0usize;
    // All four K8 controls finish before the first K9 evaluation.
    for kept in 8..=16 {
        for (i, (quarter, original)) in quarters.iter().enumerate() {
            let derived = derive_checkpoint(original, kept)?;
            let driver = PreparedDriver::load(derived.lineage.config.clone())?;
            let (model, _) = driver.restore(derived.clone(), 0)?;
            let evaluation = driver.evaluate(&model)?;
            let (metrics, admitted) = summarize(&evaluation, kept)?;
            monotone(&previous[i], &admitted)?;
            previous[i] = admitted;
            require(
                evaluation["updates_applied_by_evaluation"] == 0
                    && evaluation["model_epoch"] == original.cursor.accepted_updates,
                "evaluation changed epoch or applied updates",
            )?;
            require(
                serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
                    == derived.model,
                "evaluation changed snapshot bytes",
            )?;
            let reference_matches = reference
                .as_ref()
                .is_none_or(|r| r == &evaluation["count_reference"]);
            if reference.is_none() {
                reference = Some(evaluation["count_reference"].clone());
            }
            let control = if kept == 8 {
                qualify_k8(*quarter, &metrics)
            } else {
                Ok(())
            };
            let record = json!({"schema":"uor-r4.stage-a-derived-evaluation/1","instrument":identity,
                "input_checkpoint_sha256":quarter.sha,"completed_batches":quarter.cursor,
                "original_lineage":original.lineage,"original_model_sha256":original.model_sha256,
                "derived_lineage":derived.lineage,"derived_model_sha256":derived.model_sha256,
                "derivation":"only K in config/snapshot plus disclosed operational driver hash; original model bindings retained",
                "kept_cells":kept,"metrics":metrics,"evaluation":compact_evaluation(evaluation)?,
                "k8_control":if kept==8 {if control.is_ok(){"EXACT"}else{"MISMATCH"}}else{"NOT_APPLICABLE"},
                "snapshot_unchanged":true,"training":"HELD","updates_applied":0});
            written = written
                .checked_add(encode(&record)?.len())
                .ok_or_else(|| invalid("report size overflow"))?;
            require(
                written < 49 * 1024 * 1024,
                "derived records exceed reserved 49 MiB; instrument fault, not finding",
            )?;
            write_new(
                &output.join(format!("evaluation-{:05}-k{kept:02}.json", quarter.cursor)),
                &record,
            )?;
            control?;
            require(
                reference_matches,
                "count reference changed across checkpoint/K",
            )?;
            summaries.push(
                json!({"completed_batches":quarter.cursor,"kept_cells":kept,"metrics":metrics}),
            );
        }
    }
    let first = summaries
        .iter()
        .find(|row| {
            row["completed_batches"] == 16404
                && row["kept_cells"].as_u64().is_some_and(|k| k > 8)
                && row["metrics"]["target_admitted_rows"]
                    .as_u64()
                    .is_some_and(|n| n >= 4894)
        })
        .map(|r| r["kept_cells"].clone());
    Ok(
        json!({"schema":"uor-r4.stage-a-sweep/1","status":"EVALUATED","instrument":identity,
        "k8_qualification":"ALL_FOUR_EXACT","sweep":summaries,"smallest_passing_k":first,
        "g_a":if first.is_some(){"PASS"}else{"FAIL_EVAL_TIME_WIDENING_ONLY"},
        "scope":"K8-trained checkpoints only; K>8-trained lineages remain untested; PM reviews all four trajectories",
        "training":"HELD","updates_applied":0,"derived_record_bytes":written}),
    )
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] || args.is_empty() {
        println!("stage-a-admission identity | check CHECKPOINT | run CHECKPOINT_DIR NEW_REPORT_DIR\nrun requires CUDA build plus Claude exact-source review and the Stage-A instrument packet; never trains. Restore corpus files at their pinned config paths. Output parent must exist outside sealed roots.");
        return Ok(());
    }
    match args.first().map(String::as_str) {
        Some("identity") if args.len() == 1 => {
            let mut identity = instrument_identity()?;
            #[cfg(feature = "cuda")]
            {
                identity["compiled"] = uor_r4_discrete::device::compiled_identity();
            }
            identity["cuda_enabled"] = json!(cfg!(feature = "cuda"));
            identity["cuda_context_created"] = json!(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&identity).map_err(|e| invalid(e.to_string()))?
            );
            Ok(())
        }
        Some("check") if args.len() == 2 => {
            let (q, c) = read_checkpoint(Path::new(&args[1]))?;
            println!(
                "{}",
                json!({"status":"PINNED_CHECKPOINT","checkpoint_sha256":q.sha,"cursor":c.cursor,
                "original_driver_source_sha256":c.lineage.driver_source_sha256,"cuda_context_created":false})
            );
            Ok(())
        }
        Some("run") if args.len() == 3 => {
            #[cfg(not(feature = "cuda"))]
            {
                Err(invalid("CUDA feature required; no host model fallback"))
            }
            #[cfg(feature = "cuda")]
            {
                let path = Path::new(&args[1]);
                let output = Path::new(&args[2]);
                admit_output(&path.join("checkpoint-04101.json"), output)?;
                report_output::claim(output).map_err(|e| invalid(e.to_string()))?;
                let result = sweep(path, output);
                let report = match &result {
                    Ok(value) => value.clone(),
                    Err(e) => {
                        json!({"status":"INSTRUMENT_FAULT","error":e.to_string(),"training":"HELD","updates_applied":0})
                    }
                };
                write_new(&output.join("report.json"), &report)?;
                report_output::seal(output).map_err(|e| invalid(e.to_string()))?;
                result.map(|_| ())
            }
        }
        _ => Err(invalid(
            "use identity | check CHECKPOINT | run CHECKPOINT_DIR NEW_REPORT_DIR",
        )),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checkpoint() -> training::DriverCheckpoint {
        let plan: Value = serde_json::from_str(include_str!(
            "../../../rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
        ))
        .unwrap();
        let config: training::DriverConfig =
            serde_json::from_value(plan["config"].clone()).unwrap();
        let binding = |source: corpus::CorpusSource| corpus::CorpusBinding {
            source,
            vocabulary: config.spec.vocabulary,
            bos: config.bos.clone(),
            payload_tokens: 0,
            mask_bytes: 0,
            mask_selected_positions: 0,
            eligible_targets: 0,
            documents: 0,
            window_policy: corpus::WINDOW_POLICY.into(),
            identity_policy: corpus::IDENTITY_POLICY.into(),
        };
        let model = json!({"spec":config.spec,"seed":1,"epoch":17,"table":[2,-2,0],"routes":[4,7],"gates":[0,1],
            "source_sha256":"retained","kernel_sha256":"retained-ptx","geometry_sha256":"retained-geometry"});
        training::DriverCheckpoint {
            schema: training::CHECKPOINT_SCHEMA.into(),
            lineage: training::Lineage {
                schema: training::DRIVER_SCHEMA.into(),
                driver_source_sha256: "historical".into(),
                train: binding(config.train.clone()),
                dev: binding(config.dev.clone()),
                config,
            },
            cursor: training::Cursor {
                accepted_updates: 17,
                ..training::Cursor::initial()
            },
            model_sha256: corpus::sha256(&encode(&model).unwrap()),
            model,
        }
    }
    #[test]
    fn derivation_preserves_original_parameters_cursor_and_pins() {
        let original = checkpoint();
        let before = encode(&original).unwrap();
        for kept in 8..=16 {
            let mut derived = derive_checkpoint(&original, kept).unwrap();
            assert_eq!(derived.model["spec"]["kept_cells"], kept);
            assert_eq!(
                derived.model_sha256,
                corpus::sha256(&encode(&derived.model).unwrap())
            );
            derived.lineage.config.spec.kept_cells = 8;
            derived.lineage.driver_source_sha256 = original.lineage.driver_source_sha256.clone();
            derived.model["spec"]["kept_cells"] = json!(8);
            derived.model_sha256 = original.model_sha256.clone();
            assert_eq!(encode(&derived).unwrap(), before);
        }
        assert_eq!(encode(&original).unwrap(), before);
        for kept in [0, 7, 17, u32::MAX] {
            assert!(derive_checkpoint(&original, kept).is_err());
        }
        let mut corrupt = original;
        corrupt.model["table"][0] = json!(99);
        assert!(derive_checkpoint(&corrupt, 8).is_err());
    }
    #[test]
    fn only_exact_registered_checkpoint_hashes_are_accepted() {
        for q in QUARTERS {
            assert_eq!(quarter_for_hash(q.sha).unwrap().cursor, q.cursor);
        }
        assert!(quarter_for_hash(&"0".repeat(64)).is_err());
        assert!(quarter_for_hash(&QUARTERS[0].sha.to_uppercase()).is_err());
    }
    fn observations() -> Value {
        // Synthetic stored-receipt fields only: never a host forward/reference.
        let row = json!({"top8":[0,1,2,3,4,5,6,7],"token_hinge":0,"address_hinge":0,
            "total_loss_numerator":0,"token_rivals":7,"address_rivals":7,
            "top1_correct":1,"top8_correct":1,"admitted_cells":8});
        let batches:Vec<_>=(0..224).map(|i|json!({"rows":vec![row.clone();if i==223 {15}else{64}],"aggregate":{"stored":true}})).collect();
        json!({"batches":batches,"aggregate":{"scored_rows":14287,"top1_correct":14287,
            "top8_correct":14287,"token_strict_win_eligible_rows":14287,"admitted_cell_count_sum":114296,
            "address_eligible_rows":14287}})
    }
    #[test]
    fn receipt_accounting_checks_nesting_bounds_and_zero_rivals() {
        let mut e = observations();
        let (m, _) = summarize(&e, 8).unwrap();
        assert_eq!(m.target_admitted_rows, 14287);
        assert_eq!(m.admitted_strict_win_rows, 14287);
        e["batches"][0]["rows"][0]["token_rivals"] = json!(0);
        e["aggregate"]["token_strict_win_eligible_rows"] = json!(14286);
        assert_eq!(summarize(&e, 8).unwrap().0.strict_win_rows, 14286);
        e["batches"][0]["rows"][0]["address_rivals"] = json!(8);
        assert!(summarize(&e, 8).is_err(), "hit without target admission");
        let e = observations();
        assert!(summarize(&e, 7).is_err(), "bound follows K");
        let mut e = observations();
        e["aggregate"]["scored_rows"] = json!(1);
        assert!(summarize(&e, 8).is_err(), "aggregate mismatch");
    }
    #[test]
    fn each_k8_gate_requires_exact_registered_counts() {
        for q in QUARTERS {
            let mut m = Metrics {
                scored_rows: 14287,
                top1_correct: q.counts[0],
                top8_correct: q.counts[1],
                strict_win_rows: q.counts[2],
                admitted_strict_win_rows: q.counts[3],
                target_admitted_rows: q.counts[4],
                admitted_cell_count_sum: 114296,
                address_rival_rows: 14287,
            };
            qualify_k8(q, &m).unwrap();
            m.target_admitted_rows += 1;
            assert!(qualify_k8(q, &m).is_err());
        }
        assert!(monotone(&[true, false], &[true, true]).is_ok());
        assert!(monotone(&[true], &[false]).is_err());
        assert!(monotone(&[true], &[]).is_err());
    }
    #[test]
    fn compact_receipts_retain_every_column_and_fit_budget() {
        let source = observations();
        let compact = compact_evaluation(source.clone()).unwrap();
        let rows = compact["rows"].as_array().unwrap();
        let mut at = 0;
        for batch in source["batches"].as_array().unwrap() {
            for row in batch["rows"].as_array().unwrap() {
                for (index, name) in ROW_COLUMNS.iter().enumerate() {
                    assert_eq!(&rows[at][index], &row[*name]);
                }
                at += 1;
            }
        }
        assert_eq!(at, 14287);
        assert_eq!(compact["aggregate"], source["aggregate"]);
        assert!(encode(&compact).unwrap().len() * 36 < 49 * 1024 * 1024);
        let mut unknown_batch = source.clone();
        unknown_batch["batches"][0]["new_observation"] = json!(1);
        assert!(compact_evaluation(unknown_batch).is_err());
        let mut unknown = source;
        unknown["batches"][0]["rows"][0]["new_observation"] = json!(1);
        assert!(
            compact_evaluation(unknown).is_err(),
            "new fields cannot silently disappear"
        );
    }
    #[test]
    fn output_refuses_checkpoint_ancestors_and_sealed_report_ancestors() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("source");
        fs::create_dir(&input).unwrap();
        let checkpoint = input.join("checkpoint.json");
        fs::write(&checkpoint, b"fixture").unwrap();
        assert!(admit_output(&checkpoint, &input.join("new")).is_err());
        let safe = tmp.path().join("derived");
        assert!(admit_output(&checkpoint, &safe).is_ok());
        let sealed = tmp.path().join("sealed");
        report_output::claim(&sealed).unwrap();
        let child = sealed.join("nested");
        fs::create_dir(&child).unwrap();
        assert!(admit_output(&checkpoint, &child.join("new")).is_err());
        #[cfg(unix)]
        {
            let alias = tmp.path().join("alias");
            std::os::unix::fs::symlink(&input, &alias).unwrap();
            assert!(admit_output(&checkpoint, &alias.join("new")).is_err());
        }
        assert_eq!(fs::read(checkpoint).unwrap(), b"fixture");
    }
}
