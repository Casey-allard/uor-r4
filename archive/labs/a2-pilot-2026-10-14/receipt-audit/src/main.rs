#![forbid(unsafe_code)]
// Metadata-only audit of completed, recorded GPU work. No model execution.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};
type AuditResult<T> = Result<T, Box<dyn Error>>;
// Exact metadata field order from the qualified device.rs Snapshot and lib.rs
// Spec. The controller hashes sorted Value JSON; evaluate hashes this struct.
// No geometry, state transition or model scoring is executed by this adapter.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SpecWire {
    vocabulary: u32,
    blocks: u32,
    context: u32,
    kept_cells: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ReadWire {
    Route {},
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire {
    schema: String,
    spec: SpecWire,
    read_spec: ReadWire,
    seed: u32,
    epoch: u64,
    table: Vec<i32>,
    routes: Vec<u32>,
    gates: Vec<i32>,
    operator_kappa: String,
    composition_kappa: String,
    kernel_sha256: String,
    source_sha256: String,
    geometry_sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn verify_snapshot(checkpoint: &Value, evaluation: &Value) -> AuditResult<()> {
    require(
        checkpoint["model_sha256"] == digest(&serde_json::to_vec(&checkpoint["model"])?),
        "checkpoint Value digest mismatch",
    )?;
    let wire: SnapshotWire = serde_json::from_value(checkpoint["model"].clone())?;
    require(
        evaluation["model_snapshot_sha256"] == digest(&serde_json::to_vec(&wire)?),
        "evaluated typed snapshot digest mismatch",
    )
}
fn require(condition: bool, message: &str) -> AuditResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn number(v: &Value) -> AuditResult<u64> {
    v.as_u64()
        .ok_or_else(|| "expected exact unsigned integer".into())
}
fn boolean(v: &Value) -> AuditResult<bool> {
    v.as_bool().ok_or_else(|| "expected boolean".into())
}
fn read(root: &Path, name: &str) -> AuditResult<Value> {
    Ok(serde_json::from_slice(&fs::read(root.join(name))?)?)
}
fn main() -> AuditResult<()> {
    let root = std::env::args()
        .nth(1)
        .ok_or("expected completed attempt directory")?;
    let root = Path::new(&root);
    if root.is_file() {
        let checkpoint: Value = serde_json::from_slice(&fs::read(root)?)?;
        require(
            number(&checkpoint["checkpoint"]["cursor"]["completed_batches"])? == 0,
            "baseline-only audit needs checkpoint0",
        )?;
        verify_snapshot(
            &checkpoint["checkpoint"],
            &checkpoint["baseline_evaluation"],
        )?;
        println!("{{\"status\":\"PASS\",\"scope\":\"checkpoint-zero metadata serialization binding only\"}}");
        return Ok(());
    }
    let report = read(root, "report.json")?;
    let initial = read(root, "checkpoint-0000.json")?;
    let end = read(root, "checkpoint-1640.json")?;
    require(
        !root.join("failure.json").exists(),
        "failure receipt present",
    )?;
    require(
        report["status"] == "COMPLETE_DIAGNOSTIC_ONLY",
        "not complete",
    )?;
    require(report["attempt_resume"] == false, "unexpected resume")?;
    require(number(&report["seed"])? == 1, "seed mismatch")?;
    require(
        report["bars"] == "NOT_DECIDED_PILOT" && report["guard"] == "NOT_DECIDED_PILOT",
        "pilot scope mismatch",
    )?;
    require(
        report["mechanism_verdict"] == "NOT_ASSIGNED" && report["m2"] == "HELD",
        "verdict/hold mismatch",
    )?;
    require(
        report["fresh_transfer"] == "UNAVAILABLE_NOT_RUN",
        "unexpected transfer claim",
    )?;
    let lineage = report["baseline_evaluation"]["evaluation_lineage_sha256"]
        .as_str()
        .ok_or("missing lineage")?;
    require(
        lineage == "6db5ecf5d6e1b334b6f788a28b741b7ff2e012e7bf451e5f89ed29f89ef82a12",
        "unexpected lineage",
    )?;
    let mut cursor = initial["checkpoint"]["cursor"].clone();
    let (mut count, mut accepted, mut sensitive, mut exposures) = (0u64, 0u64, 0u64, 0u64);
    let mut last_target = None;
    for line in fs::read_to_string(root.join("steps.jsonl"))?.lines() {
        let step: Value = serde_json::from_str(line)?;
        require(step["lineage_sha256"] == lineage, "step lineage mismatch")?;
        require(step["cursor_before"] == cursor, "cursor discontinuity")?;
        require(
            step["coordinate"]["kind"] == "vector_block"
                && number(&step["coordinate"]["index"])? == count,
            "coordinate mismatch",
        )?;
        let applied = boolean(&step["applied"])?;
        let differs = boolean(&step["any_candidate_cost_differs_from_identity"])?;
        let before = number(&step["loss_before"])?;
        let after = number(&step["loss_after"])?;
        require(
            if applied {
                after < before && differs
            } else {
                after == before
            },
            "recorded move violates strict improvement/identity",
        )?;
        require(number(&step["candidate"])? < 16, "candidate out of range")?;
        let targets = step["target_positions"]
            .as_array()
            .ok_or("missing targets")?;
        require(targets.len() == 64, "wrong batch size")?;
        for target in targets {
            let target = number(target)?;
            require(
                last_target.is_none_or(|previous| target > previous),
                "repeated/out-of-order TRAIN target",
            )?;
            last_target = Some(target);
            exposures += 1;
        }
        count += 1;
        accepted += u64::from(applied);
        sensitive += u64::from(differs);
        cursor = step["cursor_after"].clone();
        require(
            number(&cursor["completed_batches"])? == count
                && number(&cursor["next_coordinate"])? == count,
            "dose/cursor mismatch",
        )?;
        require(
            number(&cursor["next_target"])? == exposures && number(&cursor["corpus_pass"])? == 0,
            "target cursor mismatch",
        )?;
        require(
            number(&cursor["accepted_updates"])? == accepted
                && number(&step["model_epoch"])? == accepted,
            "accepted/epoch mismatch",
        )?;
    }
    require(count == 1640 && exposures == 104960, "incomplete dose")?;
    require(
        cursor == end["checkpoint"]["cursor"],
        "final cursor mismatch",
    )?;
    require(
        number(&report["completed_batches"])? == count
            && number(&report["eligible_train_target_exposures"])? == exposures
            && number(&report["nominal_candidate_slots"])? == count * 16,
        "report dose mismatch",
    )?;
    require(
        report["sensitivity"]["vector_block"]
            == json!({"attempted":count,"accepted_updates":accepted,"any_candidate_cost_differs_from_identity":sensitive}),
        "sensitivity counters mismatch",
    )?;
    require(
        end["vector"] == report["sensitivity"]["vector_block"],
        "checkpoint counters mismatch",
    )?;
    require(
        number(&report["sensitivity"]["route"]["attempted"])? == 0,
        "unexpected route work",
    )?;
    require(
        initial["baseline_evaluation"] == end["baseline_evaluation"]
            && initial["baseline_evaluation"] == report["baseline_evaluation"],
        "baseline was changed",
    )?;
    require(
        report["baseline_evaluation"]["count_reference"]
            == report["final_evaluation"]["count_reference"],
        "reference was changed",
    )?;
    require(
        report["baseline_evaluation"]["pilot_slice_bias"]
            == report["final_evaluation"]["pilot_slice_bias"],
        "P1 was changed",
    )?;
    require(
        report["P2"]["checkpoint_0"] == initial["baseline_snapshot_metadata"],
        "P2 baseline mismatch",
    )?;
    verify_snapshot(&initial["checkpoint"], &report["baseline_evaluation"])?;
    verify_snapshot(&end["checkpoint"], &report["final_evaluation"])?;
    for key in [
        "routes",
        "gates",
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
            initial["checkpoint"]["model"][key] == end["checkpoint"]["model"][key],
            "unexpected change outside admitted vector updates",
        )?;
    }
    for key in ["baseline_evaluation", "final_evaluation"] {
        require(
            number(&report[key]["aggregate"]["scored_rows"])? == 14287,
            "DEV denominator mismatch",
        )?;
        require(
            report[key]["evaluation_lineage_sha256"] == lineage,
            "evaluation lineage mismatch",
        )?;
        require(
            report[key]["evaluation_mutates_model"] == false
                && number(&report[key]["updates_applied_by_evaluation"])? == 0,
            "evaluation mutated model",
        )?;
    }
    require(
        number(&report["final_evaluation"]["model_epoch"])? == accepted,
        "final epoch mismatch",
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"status":"PASS","scope":"recorded GPU receipt consistency only; no model execution","completed_batches":count,"accepted_updates":accepted,"sensitive_steps":sensitive,"masked_target_exposures":exposures,"dev_scored_rows":14287,"baseline_unchanged":true,"count_reference_unchanged":true,"final_evaluation_matches_saved_checkpoint":true})
        )?
    );
    Ok(())
}
