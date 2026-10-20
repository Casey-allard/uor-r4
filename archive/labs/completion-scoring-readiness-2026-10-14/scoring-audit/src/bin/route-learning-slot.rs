//! Final-quarter receipt metadata only. No model dependency or model execution.
//! Consume a successful score-quarter.sh audit; do not replace its row scoring.
#![forbid(unsafe_code)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    io::{Read, Write},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PLAN: &str = include_str!("../../../../rung0-completion-release-2026-10-14/SEED1_PLAN.json");
const ADMISSION: &str = "2063239bd9c65f6ccdd1fcdec21c1e6898e076dbd6cc9d171f6de9f582e51203";
const BASELINE: &str = "e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3";
const REMOTE: &str = "/workspace/uor-r4/rung0-completion-20261014";
const EVALUATION: &str = "seed1-run-state/evaluation-16404.json";
const CHECKPOINT: &str = "seed1-attempt-1/checkpoint-16404.json";
// Receipt parser bounds, not model/test acceptance bars. Checked before allocation.
const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn count(v: &Value, key: &str) -> Result<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing/non-integer counter: {key}").into())
}
fn read(path: &Path) -> Result<(Value, String)> {
    let file = fs::File::open(path)?;
    require(
        file.metadata()?.len() <= MAX_FILE_BYTES,
        "receipt exceeds 128 MiB input limit",
    )?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    require(
        bytes.len() as u64 <= MAX_FILE_BYTES,
        "receipt grew beyond input limit",
    )?;
    Ok((serde_json::from_slice(&bytes)?, hash(&bytes)))
}
fn counters(v: &Value, expected: u64) -> Result<(u64, u64, u64)> {
    let a = count(v, "attempted")?;
    let d = count(v, "any_candidate_cost_differs_from_identity")?;
    let u = count(v, "accepted_updates")?;
    require(
        a == expected && u <= d && d <= a,
        "counter schedule/order mismatch",
    )?;
    require(
        v.as_object().is_some_and(|o| o.len() == 3),
        "unexpected counter fields",
    )?;
    Ok((a, d, u))
}
fn analyze(
    audit: &Value,
    eval: &Value,
    checkpoint: &Value,
    pin: &Value,
    eval_sha: &str,
    checkpoint_sha: &str,
    plan: &Value,
) -> Result<Value> {
    require(
        audit["schema"] == "uor-r4.completion-scoring-audit/1"
            && audit["observation"] == "SINGLE_SEED_FINAL_RECEIPT"
            && audit["completed_batches"] == 16404
            && audit["seed"] == 1
            && audit["authorization"] == "none"
            && audit["authorizes_model_compute"] == false,
        "requires successful seed1 final-quarter audit, not an interim/pilot receipt",
    )?;
    require(
        audit["inputs"]["receipt_sha256"] == eval_sha
            && audit["inputs"]["referenced_checkpoint_sha256"] == checkpoint_sha
            && audit["inputs"]["plan_sha256"] == hash(PLAN.as_bytes())
            && audit["inputs"]["baseline_document_sha256"] == BASELINE,
        "audit input pins mismatch",
    )?;
    require(
        pin["path"] == format!("{REMOTE}/{EVALUATION}") && pin["sha256"] == eval_sha,
        "sealed evaluation FilePin mismatch",
    )?;
    require(
        eval["schema"] == "uor-r4.completion-evaluation/1"
            && eval["completed_batches"] == 16404
            && eval["admission_sha256"] == ADMISSION
            && eval["checkpoint"]["path"] == format!("{REMOTE}/{CHECKPOINT}")
            && eval["checkpoint"]["sha256"] == checkpoint_sha,
        "final evaluation/checkpoint/admission mismatch",
    )?;
    let cursor = &eval["cursor"];
    require(
        cursor["completed_batches"] == 16404
            && cursor["next_coordinate"] == 0
            && cursor["next_target"] == 1049856
            && cursor["corpus_pass"] == 0,
        "final cursor mismatch (16403 is a batch index)",
    )?;
    let e = &eval["evaluation"];
    let model = &checkpoint["checkpoint"]["model"];
    require(
        plan["config"]["seed"] == 1 && plan["config"]["read_spec"] == json!({"kind":"route"}),
        "only frozen seed1 Route plan supported",
    )?;
    for key in ["seed", "spec", "read_spec"] {
        require(
            e["model_bindings"][key] == plan["config"][key] && model[key] == plan["config"][key],
            "model/config binding mismatch",
        )?;
    }
    require(
        e["evaluation_lineage_sha256"] == plan["execution"]["lineage_sha256"]
            && e["evaluation_mutates_model"] == false
            && e["updates_applied_by_evaluation"] == 0,
        "evaluation lineage/mutation mismatch",
    )?;
    require(
        checkpoint["schema"] == "uor-r4.completion-checkpoint/1"
            && checkpoint["admission_sha256"] == ADMISSION
            && checkpoint["checkpoint"]["cursor"] == *cursor,
        "checkpoint wrapper/cursor mismatch",
    )?;
    let sensitivity = &eval["sensitivity"];
    require(
        sensitivity.as_object().is_some_and(|o| {
            o.len() == 2 && o.contains_key("vector_block") && o.contains_key("route")
        }),
        "unexpected per-kind inventory: frozen completion admits VectorBlock and Route only",
    )?;
    require(
        audit["sensitivity"] == *sensitivity
            && checkpoint["vector"] == sensitivity["vector_block"]
            && checkpoint["route"] == sensitivity["route"],
        "per-kind counters disagree across receipts",
    )?;
    let vector = counters(&sensitivity["vector_block"], 16384)?;
    let route = counters(&sensitivity["route"], 20)?;
    let accepted = vector
        .2
        .checked_add(route.2)
        .ok_or("accepted counter overflow")?;
    require(
        count(cursor, "accepted_updates")? == accepted
            && count(e, "model_epoch")? == accepted
            && count(e, "accepted_updates_in_model")? == accepted
            && count(model, "epoch")? == accepted,
        "accepted totals/epochs mismatch",
    )?;
    Ok(json!({
        "schema":"uor-r4.route-learning-slot/1",
        "observation":"SINGLE_SEED_FINAL_RECEIPT",
        "seed":1, "completed_batches":16404,
        "schedule": {"name":"canonical_vector_then_read_v1", "batch_index_start_inclusive":16384,
            "batch_index_end_exclusive":16404, "expected_route_coordinate_indices":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19],
            "coordinate_indices_evidence":"derived from frozen schedule; per-step receipts not consumed"},
        "per_kind": {
            "vector_block":{"evidence":"observed cumulative counters, including inherited pilot", "counters":sensitivity["vector_block"]},
            "route":{"evidence":"observed cumulative counters; all Route attempts belong to final 20 batches", "counters":sensitivity["route"],
                "attempted_any":route.0 > 0, "cost_differed_any":route.1 > 0, "accepted_any":route.2 > 0},
            "gate":{"evidence":"excluded by frozen Route plan and completion executor; no serialized Gate counter", "admitted":false,
                "attempted_any":false,"accepted_any":false,"observed_counters":null}
        },
        "read_parameter_observation": if route.2 > 0 { "ROUTE_UPDATE_ACCEPTED" } else if route.1 > 0 {
            "ROUTE_COST_CHANGED_NO_UPDATE_ACCEPTED" } else { "ROUTE_ATTEMPTED_NO_CANDIDATE_COST_DIFFERENCE" },
        "learning_slot_evidence":"final Route counter evidence available; no schedule or mechanism adoption implied",
        "limitations":["Requires successful audited scorer output from the runbook; this tool cross-checks bindings and counters, not every quality row again",
            "File hashes bind receipts, not signatures or owner authorization",
            "No per-coordinate accepted index, before/after parameter value or step-level loss attribution: steps.jsonl not consumed",
            "No isolated Route quality delta: last earlier scored quarter also precedes remaining VectorBlock proposals",
            "Gate was not tested; Route result cannot establish Gate or Relation efficacy",
            "One seed, no fresh transfer or two-seed verdict; learning schedule remains unchanged"],
        "authorization":"none", "authorizes_model_compute":false, "seed2":"HELD", "m2":"HELD"
    }))
}
fn run(dir: &Path, output: &Path) -> Result<()> {
    let (audit, audit_sha) = read(&dir.join("audit.json"))?;
    let (eval, eval_sha) = read(&dir.join(EVALUATION))?;
    let (checkpoint, checkpoint_sha) = read(&dir.join(CHECKPOINT))?;
    let (pin, pin_sha) = read(&dir.join("seed1-run-state/evaluation-16404.pin.json"))?;
    let plan: Value = serde_json::from_str(PLAN)?;
    let mut result = analyze(
        &audit,
        &eval,
        &checkpoint,
        &pin,
        &eval_sha,
        &checkpoint_sha,
        &plan,
    )?;
    result["inputs"] = json!({"audit_sha256":audit_sha,"evaluation_sha256":eval_sha,
        "checkpoint_sha256":checkpoint_sha,"evaluation_pin_sha256":pin_sha,"plan_sha256":hash(PLAN.as_bytes())});
    let bytes = serde_json::to_vec_pretty(&result)?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    out.write_all(&bytes)?;
    out.write_all(b"\n")?;
    println!(
        "Route learning-slot receipt validation PASS; cursor 16404; output {}",
        output.display()
    );
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    require(
        args.len() == 3,
        "usage: route-learning-slot SEALED_Q4_AUDIT_DIRECTORY NEW_OUTPUT_JSON",
    )?;
    run(Path::new(&args[1]), Path::new(&args[2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Synthetic metadata only: deliberately not a model or final-run measurement.
    fn fixture(differs: u64, accepted: u64) -> (Value, Value, Value, Value, Value) {
        let plan: Value = serde_json::from_str(PLAN).unwrap();
        let vector = json!({"attempted":16384,"any_candidate_cost_differs_from_identity":4000,"accepted_updates":400});
        let route = json!({"attempted":20,"any_candidate_cost_differs_from_identity":differs,"accepted_updates":accepted});
        let cursor = json!({"completed_batches":16404,"next_coordinate":0,"next_target":1049856,"corpus_pass":0,"accepted_updates":400+accepted});
        let binding = json!({"seed":1,"spec":plan["config"]["spec"],"read_spec":{"kind":"route"},"epoch":400+accepted});
        let sensitivity = json!({"vector_block":vector,"route":route});
        let eval = json!({"schema":"uor-r4.completion-evaluation/1","completed_batches":16404,"admission_sha256":ADMISSION,
            "checkpoint":{"path":format!("{REMOTE}/{CHECKPOINT}"),"sha256":"checkpoint"},"cursor":cursor,"sensitivity":sensitivity,
            "evaluation":{"model_bindings":binding,"evaluation_lineage_sha256":plan["execution"]["lineage_sha256"],
                "evaluation_mutates_model":false,"updates_applied_by_evaluation":0,"model_epoch":400+accepted,"accepted_updates_in_model":400+accepted}});
        let checkpoint = json!({"schema":"uor-r4.completion-checkpoint/1","admission_sha256":ADMISSION,"vector":vector,"route":route,
            "checkpoint":{"cursor":cursor,"model":binding}});
        let audit = json!({"schema":"uor-r4.completion-scoring-audit/1","observation":"SINGLE_SEED_FINAL_RECEIPT","completed_batches":16404,
            "seed":1,"authorization":"none","authorizes_model_compute":false,"sensitivity":sensitivity,
            "inputs":{"receipt_sha256":"eval","referenced_checkpoint_sha256":"checkpoint","plan_sha256":hash(PLAN.as_bytes()),"baseline_document_sha256":BASELINE}});
        let pin = json!({"path":format!("{REMOTE}/{EVALUATION}"),"sha256":"eval"});
        (audit, eval, checkpoint, pin, plan)
    }
    fn check(f: &(Value, Value, Value, Value, Value)) -> Result<Value> {
        analyze(&f.0, &f.1, &f.2, &f.3, "eval", "checkpoint", &f.4)
    }
    #[test]
    fn three_distinct_route_observations_and_gate_not_tested() {
        for (d, u, expected) in [
            (0, 0, "ROUTE_ATTEMPTED_NO_CANDIDATE_COST_DIFFERENCE"),
            (4, 0, "ROUTE_COST_CHANGED_NO_UPDATE_ACCEPTED"),
            (4, 2, "ROUTE_UPDATE_ACCEPTED"),
        ] {
            let r = check(&fixture(d, u)).unwrap();
            assert_eq!(r["read_parameter_observation"], expected);
            assert_eq!(r["per_kind"]["route"]["accepted_any"], u > 0);
            assert!(r["per_kind"]["gate"]["observed_counters"].is_null());
            assert_eq!(r["authorizes_model_compute"], false);
        }
    }
    #[test]
    fn refusals_finality_pins_inventory_epochs_and_lineage() {
        for (which, path, value) in [
            (0, "/completed_batches", json!(12303)),
            (0, "/seed", json!(2)),
            (0, "/inputs/receipt_sha256", json!("wrong")),
            (0, "/inputs/plan_sha256", json!("wrong")),
            (0, "/inputs/baseline_document_sha256", json!("wrong")),
            (1, "/completed_batches", json!(16403)),
            (1, "/cursor/next_coordinate", json!(16404)),
            (1, "/admission_sha256", json!("wrong")),
            (1, "/checkpoint/path", json!("other-attempt")),
            (
                1,
                "/evaluation/model_bindings/read_spec/kind",
                json!("flock"),
            ),
            (1, "/evaluation/model_epoch", json!(402)),
            (1, "/evaluation/evaluation_lineage_sha256", json!("wrong")),
            (2, "/route/attempted", json!(19)),
            (2, "/checkpoint/cursor/accepted_updates", json!(999)),
            (3, "/sha256", json!("wrong")),
            (3, "/path", json!("other-pin")),
        ] {
            let mut f = fixture(1, 1);
            let doc = match which {
                0 => &mut f.0,
                1 => &mut f.1,
                2 => &mut f.2,
                _ => &mut f.3,
            };
            *doc.pointer_mut(path).unwrap() = value;
            assert!(check(&f).is_err(), "accepted {which}:{path}");
        }
        let mut f = fixture(1, 1);
        f.1["sensitivity"]["gate"] = json!({});
        assert!(check(&f).is_err());
    }
    #[test]
    fn impossible_or_missing_counters_refused() {
        for v in [
            json!({"attempted":20,"any_candidate_cost_differs_from_identity":21,"accepted_updates":1}),
            json!({"attempted":20,"any_candidate_cost_differs_from_identity":1,"accepted_updates":2}),
            json!({"attempted":20,"any_candidate_cost_differs_from_identity":1}),
            json!({"attempted":-1,"any_candidate_cost_differs_from_identity":0,"accepted_updates":0}),
            json!({"attempted":"20","any_candidate_cost_differs_from_identity":0,"accepted_updates":0}),
        ] {
            assert!(counters(&v, 20).is_err());
        }
    }
    #[test]
    fn filesystem_round_trip_tamper_and_no_clobber() {
        let dir = std::env::temp_dir().join(format!(
            "route-slot-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("seed1-run-state")).unwrap();
        fs::create_dir_all(dir.join("seed1-attempt-1")).unwrap();
        let (mut audit, mut eval, checkpoint, mut pin, _) = fixture(3, 1);
        let cb = serde_json::to_vec(&checkpoint).unwrap();
        let cs = hash(&cb);
        eval["checkpoint"]["sha256"] = json!(cs);
        let eb = serde_json::to_vec(&eval).unwrap();
        let es = hash(&eb);
        audit["inputs"]["receipt_sha256"] = json!(es);
        audit["inputs"]["referenced_checkpoint_sha256"] = json!(cs);
        pin["sha256"] = json!(es);
        fs::write(dir.join(EVALUATION), &eb).unwrap();
        fs::write(dir.join(CHECKPOINT), &cb).unwrap();
        fs::write(dir.join("audit.json"), serde_json::to_vec(&audit).unwrap()).unwrap();
        fs::write(
            dir.join("seed1-run-state/evaluation-16404.pin.json"),
            serde_json::to_vec(&pin).unwrap(),
        )
        .unwrap();
        let output = dir.join("result.json");
        run(&dir, &output).unwrap();
        let original = fs::read(&output).unwrap();
        assert!(run(&dir, &output).is_err());
        assert_eq!(fs::read(&output).unwrap(), original);
        fs::write(dir.join(EVALUATION), [eb, b" ".to_vec()].concat()).unwrap();
        assert!(run(&dir, &dir.join("bad.json")).is_err());
        assert!(!dir.join("bad.json").exists());
        let oversized = dir.join("oversized");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        assert!(read(&oversized).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
