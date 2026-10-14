//! Reads completed GPU receipts only. No model initialization, distances, or forward.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fs, io::Write};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const REPORT_SHA: &str = "e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3";
const LINEAGE: &str = "6db5ecf5d6e1b334b6f788a28b741b7ff2e012e7bf451e5f89ed29f89ef82a12";
type Counts = BTreeMap<String, i64>;

fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn n(v: &Value, key: &str) -> Result<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .filter(|x| *x >= 0)
        .ok_or_else(|| format!("missing/nonnegative integer field: {key}").into())
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array: {key}").into())
}
fn add(c: &mut Counts, key: &str, value: i64) -> Result<()> {
    let slot = c.entry(key.into()).or_default();
    *slot = slot.checked_add(value).ok_or("receipt sum overflow")?;
    Ok(())
}
fn admission(row: &Value) -> Result<i64> {
    let a = n(row, "admitted_cells")? - n(row, "address_rivals")?;
    require(
        (0..=1).contains(&a),
        "invalid target-cell admission difference",
    )?;
    Ok(a)
}
fn row_counts(row: &Value) -> Result<Counts> {
    let t = n(row, "token_hinge")?;
    let a = n(row, "address_hinge")?;
    let total = n(row, "total_loss_numerator")?;
    let tr = n(row, "token_rivals")?;
    require(tr <= 4095, "token rival count exceeds nonself vocabulary")?;
    let ar = n(row, "address_rivals")?;
    let cells = n(row, "admitted_cells")?;
    let h1 = n(row, "top1_correct")?;
    let h8 = n(row, "top8_correct")?;
    require(h1 <= h8 && h8 <= 1, "invalid rank-hit flags")?;
    require(
        (1..=8).contains(&cells) && ar <= cells,
        "invalid admitted cells",
    )?;
    let weighted = t
        .checked_mul(8)
        .and_then(|v| a.checked_mul(32).and_then(|w| v.checked_add(w)))
        .ok_or("stored weighted loss overflow")?;
    require(total == weighted, "row weighted loss mismatch")?;
    require(h8 <= admission(row)?, "hit without target admission")?;
    require(
        tr + admission(row)? >= 8 && tr + admission(row)? <= 4096,
        "invalid admitted token count",
    )?;
    require(
        !(tr > 0 && t < 32 && admission(row)? == 1) || h1 == 1,
        "admitted strict win is not top1",
    )?;
    let top8 = array(row, "top8")?;
    require(top8.len() == 8, "missing top8 identities")?;
    let mut seen = std::collections::BTreeSet::new();
    for id in top8 {
        let id = id.as_u64().ok_or("invalid ranked token ID")?;
        require(
            id < 4096 && seen.insert(id),
            "invalid/duplicate ranked identity",
        )?;
    }
    Ok([
        ("scored_rows", 1),
        ("top1_correct", h1),
        ("top8_correct", h8),
        ("token_hinge_sum", t),
        ("address_hinge_sum", a),
        ("total_loss_numerator_sum", total),
        ("token_eligible_rows", i64::from(tr > 0)),
        ("address_eligible_rows", i64::from(ar > 0)),
        ("token_rival_count_sum", tr),
        ("address_rival_count_sum", ar),
        ("admitted_cell_count_sum", cells),
        ("eligible_any_rows", i64::from(tr > 0 || ar > 0)),
        (
            "eligible_unviolated_rows",
            i64::from(total == 0 && (tr > 0 || ar > 0)),
        ),
        (
            "token_unviolated_eligible_rows",
            i64::from(tr > 0 && t == 0),
        ),
        (
            "address_unviolated_eligible_rows",
            i64::from(ar > 0 && a == 0),
        ),
        (
            "token_strict_win_eligible_rows",
            i64::from(tr > 0 && t < 32),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect())
}
fn verify_aggregate(sums: &Counts, stored: &Value) -> Result<()> {
    require(
        n(stored, "loss_denominator_per_row")? == 256,
        "denominator mismatch",
    )?;
    for (k, v) in sums {
        require(n(stored, k)? == *v, &format!("aggregate mismatch: {k}"))?;
    }
    Ok(())
}
fn rows(eval: &Value) -> Result<Vec<&Value>> {
    let batches = array(eval, "batches")?;
    require(batches.len() == 224, "wrong DEV batch count")?;
    let mut all = Vec::new();
    let mut total = Counts::new();
    for (i, batch) in batches.iter().enumerate() {
        let rows = array(batch, "rows")?;
        require(
            rows.len() == if i == 223 { 15 } else { 64 },
            "wrong DEV batch shape/order",
        )?;
        let mut sums = Counts::new();
        for row in rows {
            for (k, v) in row_counts(row)? {
                add(&mut sums, &k, v)?;
                add(&mut total, &k, v)?;
            }
            all.push(row);
        }
        verify_aggregate(&sums, &batch["aggregate"])?;
    }
    require(all.len() == 14287, "wrong paired row count")?;
    verify_aggregate(&total, &eval["aggregate"])?;
    Ok(all)
}
fn stratum(c: &mut Counts, before: &Value, after: &Value) -> Result<()> {
    for key in [
        "loss_increased_rows",
        "loss_decreased_rows",
        "loss_unchanged_rows",
    ] {
        c.entry(key.into()).or_default();
    }
    add(c, "rows", 1)?;
    for (name, r) in [("baseline", before), ("final", after)] {
        add(c, &format!("{name}_top1_correct"), n(r, "top1_correct")?)?;
        add(c, &format!("{name}_top8_correct"), n(r, "top8_correct")?)?;
    }
    let td = 8 * (n(after, "token_hinge")? - n(before, "token_hinge")?);
    let ad = 32 * (n(after, "address_hinge")? - n(before, "address_hinge")?);
    let d = n(after, "total_loss_numerator")? - n(before, "total_loss_numerator")?;
    require(td + ad == d, "stratum contribution mismatch")?;
    add(c, "weighted_token_delta", td)?;
    add(c, "weighted_address_delta", ad)?;
    add(c, "total_loss_numerator_delta", d)?;
    add(
        c,
        if d > 0 {
            "loss_increased_rows"
        } else if d < 0 {
            "loss_decreased_rows"
        } else {
            "loss_unchanged_rows"
        },
        1,
    )?;
    Ok(())
}

const SEED1: &str = include_str!("../../../rung0-completion-release-2026-10-14/SEED1_PLAN.json");
const SEED2: &str = include_str!("../../../rung0-completion-release-2026-10-14/SEED2_PLAN.json");
const POLICY: &str =
    "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1";
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: &str) -> Result<(Value, String)> {
    let bytes = fs::read(path)?;
    Ok((serde_json::from_slice(&bytes)?, hash(&bytes)))
}
fn sha(v: &Value) -> Result<&str> {
    let s = v.as_str().ok_or("missing SHA")?;
    require(
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid SHA",
    )?;
    Ok(s)
}
fn baseline(doc: &Value) -> Result<&Value> {
    let e = doc.get("baseline_evaluation").unwrap_or(doc);
    require(
        e["schema"] == "uor-r4.discrete-corpus-driver/1",
        "missing original baseline evaluation",
    )?;
    require(
        n(e, "model_epoch")? == 0 && n(e, "accepted_updates_in_model")? == 0,
        "baseline is not checkpoint zero",
    )?;
    Ok(e)
}
fn reference(e: &Value) -> Result<()> {
    let r = &e["count_reference"];
    require(
        r["dev_oracle_diagnostic_only"] == true,
        "DEV oracle must be diagnostic only",
    )?;
    for (key, source, nonzero, top8hits) in [
        ("train_masked_reference", 2425925, 1372, 4060),
        ("dev_response_oracle", 14287, 674, 4134),
    ] {
        let v = &r[key];
        let counts = array(v, "token_counts")?;
        require(counts.len() == 4096, "reference count table length")?;
        let mut sum = 0u64;
        let mut nz = 0;
        for count in counts {
            let count = count.as_u64().ok_or("invalid reference count")?;
            sum = sum.checked_add(count).ok_or("reference overflow")?;
            nz += i64::from(count > 0);
        }
        require(
            sum == source as u64
                && n(v, "source_targets")? == source
                && nz == nonzero
                && n(v, "nonzero_tokens")? == nz,
            "reference count totals",
        )?;
        require(
            n(v, "scored_rows")? == 14287
                && n(v, "top1_correct")? == 1226
                && n(v, "top8_correct")? == top8hits
                && n(v, "zero_count_targets")? == 0,
            "frozen reference measurement mismatch",
        )?;
        let expected = if key == "train_masked_reference" {
            json!([1, 16, 14, 435, 340, 351, 3, 302])
        } else {
            json!([1, 16, 14, 435, 340, 351, 3, 223])
        };
        require(v["top8"] == expected, "frozen reference ranking mismatch")?;
    }
    Ok(())
}
fn validate_evaluation(e: &Value, plan: &Value, migrated: bool) -> Result<()> {
    require(
        e["schema"] == "uor-r4.discrete-corpus-driver/1" && e["scored_set_policy"] == POLICY,
        "wrong evaluation schema/policy",
    )?;
    require(
        e["evaluation_mutates_model"] == false && n(e, "updates_applied_by_evaluation")? == 0,
        "evaluation mutation",
    )?;
    require(
        n(e, "model_epoch")? == n(e, "accepted_updates_in_model")?,
        "evaluation epoch mismatch",
    )?;
    sha(&e["model_snapshot_sha256"])?;
    let execution = &plan["execution"];
    let lineage = &e["evaluation_lineage"];
    let frozen: Value = serde_json::from_str(include_str!(
        "../../../a2-pilot-2026-10-14/report-summary.json"
    ))?;
    let mut expected_lineage = frozen["baseline_evaluation"]["evaluation_lineage"].clone();
    expected_lineage["config"] = plan["config"].clone();
    if migrated {
        expected_lineage["driver_source_sha256"] = execution["driver_source_sha256"].clone();
    }
    require(
        lineage == &expected_lineage,
        "frozen panel lineage metadata differs",
    )?;
    let mut expected_bindings = frozen["baseline_evaluation"]["model_bindings"].clone();
    expected_bindings["seed"] = plan["config"]["seed"].clone();
    require(
        e["model_bindings"] == expected_bindings,
        "full frozen model bindings differ",
    )?;
    require(
        lineage["config"] == plan["config"],
        "evaluation config does not match seed plan",
    )?;
    require(
        lineage["schema"] == "uor-r4.discrete-corpus-driver/1",
        "lineage schema",
    )?;
    if migrated {
        require(
            e["evaluation_lineage_sha256"] == execution["lineage_sha256"]
                && lineage["driver_source_sha256"] == execution["driver_source_sha256"],
            "completion driver/lineage pin mismatch",
        )?;
    } else {
        require(
            e["evaluation_lineage_sha256"] == LINEAGE
                && lineage["driver_source_sha256"]
                    == "236e7fb6e5269e3bd4fed21d391d1ce82f7d9bd31275f1a0d97dd0c4ce9d9806",
            "pilot driver/lineage pin mismatch",
        )?;
    }
    let bindings = &e["model_bindings"];
    for k in ["source_sha256", "kernel_sha256", "geometry_sha256"] {
        require(
            bindings[k] == execution[k],
            &format!("model binding changed: {k}"),
        )?;
    }
    for k in ["seed", "spec", "read_spec"] {
        require(
            bindings[k] == plan["config"][k],
            "model config binding mismatch",
        )?;
    }
    reference(e)?;
    Ok(())
}
fn plan(input: &Value) -> Result<Value> {
    let seed = n(&input["config"], "seed")?;
    let canonical: Value = serde_json::from_str(match seed {
        1 => SEED1,
        2 => SEED2,
        _ => return Err("unsupported seed".into()),
    })?;
    for k in ["schema", "scope", "config", "pilot_parent"] {
        require(
            input[k] == canonical[k],
            &format!("plan differs from canonical seed packet: {k}"),
        )?;
    }
    for k in [
        "base_source",
        "reviewed_source_commit",
        "source_sha256",
        "kernel_sha256",
        "geometry_sha256",
        "driver_source_sha256",
        "config_sha256",
        "lineage_sha256",
    ] {
        require(
            input["execution"][k] == canonical["execution"][k],
            &format!("plan identity differs: {k}"),
        )?;
    }
    for k in [
        "source_sha256",
        "kernel_sha256",
        "geometry_sha256",
        "driver_source_sha256",
        "config_sha256",
        "lineage_sha256",
        "sheet_sha256",
    ] {
        sha(&input["execution"][k])?;
    }
    Ok(canonical)
}
fn completion_wrapper(doc: &Value) -> Result<i64> {
    require(
        doc["schema"] == "uor-r4.completion-evaluation/1",
        "expected actual completion evaluation wrapper",
    )?;
    let count = n(doc, "completed_batches")?;
    require(
        [4101, 8202, 12303, 16404].contains(&count),
        "invalid completed cursor: 16403 is a batch index, not final cursor",
    )?;
    let c = &doc["cursor"];
    require(
        n(c, "completed_batches")? == count
            && n(c, "next_target")? == count * 64
            && n(c, "next_coordinate")? == count % 16404
            && n(c, "corpus_pass")? == 0,
        "completion cursor mismatch",
    )?;
    require(
        n(c, "accepted_updates")? == n(&doc["evaluation"], "model_epoch")?
            && n(c, "accepted_updates")? <= count,
        "completion accepted epoch mismatch",
    )?;
    for k in ["admission_sha256", "driver_checkpoint_sha256"] {
        sha(&doc[k])?;
    }
    sha(&doc["checkpoint"]["sha256"])?;
    require(
        doc["checkpoint"]["path"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "missing checkpoint pin path",
    )?;
    let vector = &doc["sensitivity"]["vector_block"];
    let route = &doc["sensitivity"]["route"];
    for (counter, attempts) in [(vector, count.min(16384)), (route, (count - 16384).max(0))] {
        require(
            n(counter, "attempted")? == attempts
                && n(counter, "accepted_updates")?
                    <= n(counter, "any_candidate_cost_differs_from_identity")?
                && n(counter, "any_candidate_cost_differs_from_identity")? <= attempts,
            "sensitivity counters mismatch",
        )?;
    }
    require(
        n(vector, "accepted_updates")? + n(route, "accepted_updates")? == n(c, "accepted_updates")?,
        "counter accepted sum mismatch",
    )?;
    Ok(count)
}
fn checkpoint_payload(
    doc: &Value,
    checkpoint: &Value,
    original_baseline: &Value,
    plan: &Value,
) -> Result<()> {
    require(
        checkpoint["schema"] == "uor-r4.completion-checkpoint/1",
        "wrong checkpoint wrapper schema",
    )?;
    for key in ["admission_sha256"] {
        require(checkpoint[key] == doc[key], "checkpoint admission mismatch")?;
    }
    require(
        checkpoint["checkpoint"]["cursor"] == doc["cursor"],
        "checkpoint cursor mismatch",
    )?;
    require(
        &checkpoint["baseline_evaluation"] == original_baseline,
        "checkpoint replaced original baseline",
    )?;
    require(
        checkpoint["pilot_parent"] == plan["pilot_parent"],
        "checkpoint pilot parent mismatch",
    )?;
    require(
        checkpoint["vector"] == doc["sensitivity"]["vector_block"]
            && checkpoint["route"] == doc["sensitivity"]["route"],
        "checkpoint sensitivity mismatch",
    )?;
    let inner = &checkpoint["checkpoint"];
    require(
        inner["lineage"] == doc["evaluation"]["evaluation_lineage"],
        "checkpoint lineage mismatch",
    )?;
    require(
        inner["model"]["epoch"] == doc["evaluation"]["model_epoch"],
        "checkpoint model epoch mismatch",
    )?;
    require(
        hash(&serde_json::to_vec(&inner["model"])?) == sha(&inner["model_sha256"])?,
        "checkpoint stored model hash mismatch",
    )?;
    for k in [
        "source_sha256",
        "kernel_sha256",
        "geometry_sha256",
        "operator_kappa",
        "composition_kappa",
        "seed",
        "spec",
        "read_spec",
    ] {
        require(
            inner["model"][k] == doc["evaluation"]["model_bindings"][k],
            "checkpoint model binding mismatch",
        )?;
    }
    let metadata = &doc["snapshot_metadata"];
    require(
        n(metadata, "vocabulary")? == 4096
            && (1..=4096).contains(&n(metadata, "distinct_full_vector_patterns")?)
            && (1..=256).contains(&n(metadata, "occupied_writer_cells")?),
        "missing/invalid checkpoint P2 metadata",
    )?;
    Ok(())
}
fn outcome(base: &Value, end: &Value) -> Result<Value> {
    let u = n(&base["aggregate"], "token_strict_win_eligible_rows")?;
    let current = n(&end["aggregate"], "token_strict_win_eligible_rows")?;
    let min_add = (14287 + 19) / 20;
    let threshold100 = (14287_i64).max((500 * u).min(100 * u + 5 * 14287));
    let threshold = (threshold100 + 99) / 100;
    Ok(
        json!({"rank_bar_minimum_counts":{"top1":1226+min_add,"top8":4060+min_add},
        "rank_bar_comparisons":{"top1":n(&end["aggregate"],"top1_correct")?>=1226+min_add,"top8":n(&end["aggregate"],"top8_correct")?>=4060+min_add},
        "guard":{"baseline_strict_wins":u,"current_strict_wins":current,"minimum_integer_count":threshold,"instrument_invalid":100*u>95*14287,"meets_threshold":current>=threshold,"definition":"token_rivals>0 && token_hinge<32; admission not required"},
        "interpretation":"per-seed receipt comparisons only; no two-seed verdict or transfer claim"}),
    )
}
fn analyze(base: &Value, end: &Value) -> Result<Value> {
    let before = rows(base)?;
    let after = rows(end)?;
    require(
        base["count_reference"] == end["count_reference"],
        "count reference changed",
    )?;
    require(
        base["model_bindings"] == end["model_bindings"],
        "numerical model bindings changed",
    )?;
    let mut l0 = base["evaluation_lineage"].clone();
    let mut l1 = end["evaluation_lineage"].clone();
    for l in [&mut l0, &mut l1] {
        l.as_object_mut()
            .ok_or("missing lineage")?
            .remove("driver_source_sha256");
    }
    require(
        l0 == l1,
        "panel/data lineage changed beyond explicit driver migration",
    )?;
    let mut admission_strata: BTreeMap<String, Counts> = BTreeMap::new();
    let mut hit_strata: BTreeMap<String, Counts> = BTreeMap::new();
    let mut overall = Counts::new();
    let mut metrics: BTreeMap<String, Counts> = BTreeMap::new();
    for (name, rows) in [("baseline", &before), ("current", &after)] {
        let c = metrics.entry(name.into()).or_default();
        for key in [
            "target_admitted_rows",
            "strict_wins",
            "admitted_strict_wins",
            "omitted_strict_wins",
            "top1_correct",
            "top8_correct",
        ] {
            c.insert(key.into(), 0);
        }
        for row in rows {
            let admitted = admission(row)?;
            add(c, "target_admitted_rows", admitted)?;
            add(c, "top1_correct", n(row, "top1_correct")?)?;
            add(c, "top8_correct", n(row, "top8_correct")?)?;
            if n(row, "token_rivals")? > 0 && n(row, "token_hinge")? < 32 {
                add(c, "strict_wins", 1)?;
                add(
                    c,
                    if admitted == 1 {
                        "admitted_strict_wins"
                    } else {
                        "omitted_strict_wins"
                    },
                    1,
                )?;
            }
        }
    }
    for (a, b) in before.iter().zip(after) {
        let name = |x, y| match (x, y) {
            (1, 1) => "retained",
            (1, 0) => "lost",
            (0, 1) => "gained",
            _ => "never",
        };
        stratum(
            admission_strata
                .entry(name(admission(a)?, admission(b)?).into())
                .or_default(),
            a,
            b,
        )?;
        stratum(
            hit_strata
                .entry(name(n(a, "top8_correct")?, n(b, "top8_correct")?).into())
                .or_default(),
            a,
            b,
        )?;
        stratum(&mut overall, a, b)?;
    }
    Ok(
        json!({"metrics":metrics,"admission_transitions":admission_strata,"top8_transitions":hit_strata,"overall":overall,"registered_comparisons":outcome(base,end)?}),
    )
}
fn main() -> Result<()> {
    let started = std::time::Instant::now();
    let args: Vec<String> = std::env::args().collect();
    require((args.len()==4 && args[1]=="pilot") || (args.len()==6 && args[1]=="completion"),"usage: scoring-audit pilot REPORT NEW_OUTPUT | completion EVALUATION ORIGINAL_BASELINE CANONICAL_SEED_PLAN NEW_OUTPUT")?;
    let (doc, input_sha) = read(&args[2])?;
    let mut inputs = json!({"receipt_sha256":input_sha});
    let (baseline_doc, plan_doc, completed, output, mode) = if args[1] == "pilot" {
        require(input_sha == REPORT_SHA, "archived pilot SHA mismatch")?;
        (
            doc.clone(),
            serde_json::from_str::<Value>(SEED1)?,
            1640,
            args[3].as_str(),
            "pilot",
        )
    } else {
        let (baseline_doc, bsha) = read(&args[3])?;
        let (plan_doc, psha) = read(&args[4])?;
        inputs["baseline_document_sha256"] = json!(bsha);
        inputs["plan_sha256"] = json!(psha);
        plan(&plan_doc)?;
        let n = completion_wrapper(&doc)?;
        (baseline_doc, plan_doc, n, args[5].as_str(), "completion")
    };
    let base = baseline(&baseline_doc)?;
    let parsed_ns = started.elapsed().as_nanos();
    let end = if mode == "pilot" {
        &doc["final_evaluation"]
    } else {
        &doc["evaluation"]
    };
    let seed = n(&plan_doc["config"], "seed")?;
    if mode == "completion" {
        if seed == 1 {
            let bsha = inputs["baseline_document_sha256"]
                .as_str()
                .ok_or("missing baseline pin")?;
            require(
                bsha == REPORT_SHA
                    || bsha == "46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440",
                "seed1 original baseline must be exact archived report or checkpoint1640",
            )?;
        }
        let path = std::env::var("SCORING_AUDIT_CHECKPOINT_PATH").unwrap_or_else(|_| {
            doc["checkpoint"]["path"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_default()
        });
        require(!path.is_empty(), "checkpoint path")?;
        let (checkpoint, csha) = read(&path)?;
        require(
            csha == sha(&doc["checkpoint"]["sha256"])?,
            "checkpoint FilePin byte SHA mismatch",
        )?;
        checkpoint_payload(&doc, &checkpoint, base, &plan_doc)?;
        inputs["referenced_checkpoint_sha256"] = json!(csha);
    }
    validate_evaluation(base, &plan_doc, seed == 2)?;
    validate_evaluation(end, &plan_doc, mode == "completion")?;
    let mut result = analyze(base, end)?;
    let validated_ns = started.elapsed().as_nanos() - parsed_ns;
    result["schema"] = json!("uor-r4.completion-scoring-audit/1");
    result["scope"] = json!("stored CUDA receipts only; no model computation or authorization");
    result["inputs"] = inputs;
    result["seed"] = json!(seed);
    result["completed_batches"] = json!(completed);
    result["observation"] = json!(if mode == "pilot" {
        "ARCHIVED_PILOT_ONLY_COMPLETION_NOT_RUN"
    } else if completed == 16404 {
        "SINGLE_SEED_FINAL_RECEIPT"
    } else {
        "INTERMEDIATE_CHECKPOINT_DIAGNOSTIC"
    });
    result["lineage"] = json!({"baseline":base["evaluation_lineage_sha256"],"current":end["evaluation_lineage_sha256"],"driver_migration":base["evaluation_lineage_sha256"]!=end["evaluation_lineage_sha256"],"canonical_plan_execution":plan_doc["execution"]});
    result["limitations"]=json!(["row pairing uses pinned fixed DEV ordinal; row receipts lack explicit target IDs", "batch epoch and binding are not serialized; only overall epoch and completion cursor checked", "completion checkpoint FilePin bytes and metadata are checked; typed inner DriverCheckpoint hash and typed evaluation snapshot hash remain production-validator scope", "owner-release files are not read or authenticated; this is not release admission", "seed2 baseline is checked against the completion checkpoint and fixed panel; independent original checkpoint-zero FilePin authentication remains production-validator scope", "reference tables are checked as stored metadata; no CPU predictor or model forward is run", "two-seed verdict and fresh transfer are not inferred from a single receipt"]);
    result["authorization"] = json!("none");
    result["authorizes_model_compute"] = json!(false);
    result["snapshot_metadata"] = if mode == "pilot" {
        doc["P2"].clone()
    } else {
        doc["snapshot_metadata"].clone()
    };
    result["sensitivity"] = doc["sensitivity"].clone();
    let mut out = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)?;
    out.write_all(&serde_json::to_vec_pretty(&result)?)?;
    out.write_all(b"\n")?;
    eprintln!(
        "receipt phases (ns): parse={parsed_ns} validate={validated_ns} write={}",
        started.elapsed().as_nanos() - parsed_ns - validated_ns
    );
    println!("Receipt audit PASS: seed {seed}, completed cursor {completed}; output {output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pilot() -> Value {
        let path = std::env::var("SCORING_AUDIT_PILOT_REPORT")
            .expect("set SCORING_AUDIT_PILOT_REPORT to archived GPU report for receipt-only tests");
        let (v, h) = read(&path).expect("read immutable receipt");
        assert_eq!(h, REPORT_SHA);
        v
    }
    #[test]
    fn archived_rows_and_reference_validate() {
        let v = pilot();
        for e in [&v["baseline_evaluation"], &v["final_evaluation"]] {
            rows(e).unwrap();
            reference(e).unwrap();
        }
    }
    #[test]
    fn missing_batch_refuses() {
        let mut v = pilot();
        v["final_evaluation"]["batches"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(rows(&v["final_evaluation"]).is_err());
    }
    #[test]
    fn altered_loss_refuses() {
        let mut v = pilot();
        v["final_evaluation"]["batches"][0]["rows"][0]["total_loss_numerator"] = json!(1);
        assert!(rows(&v["final_evaluation"]).is_err());
    }
    #[test]
    fn aggregate_mismatch_refuses() {
        let mut v = pilot();
        v["final_evaluation"]["aggregate"]["top8_correct"] = json!(1170);
        assert!(rows(&v["final_evaluation"]).is_err());
    }
    #[test]
    fn missing_row_metric_refuses() {
        let mut v = pilot();
        v["final_evaluation"]["batches"][0]["rows"][0]
            .as_object_mut()
            .unwrap()
            .remove("address_rivals");
        assert!(rows(&v["final_evaluation"]).is_err());
    }
    #[test]
    fn count_reference_changes_refuse() {
        let mut v = pilot();
        v["final_evaluation"]["count_reference"]["train_masked_reference"]["token_counts"][0] =
            json!(999);
        assert!(reference(&v["final_evaluation"]).is_err());
    }
    #[test]
    fn invalid_completion_cursor_refuses() {
        for cursor in [0, 1640, 16403, 16405] {
            assert!(completion_wrapper(
                &json!({"schema":"uor-r4.completion-evaluation/1","completed_batches":cursor})
            )
            .is_err());
        }
    }
    #[test]
    fn synthetic_metadata_final_cursor_allows_twenty_route_attempts() {
        // Metadata fixture only: no simulated GPU rows or completion result is produced.
        let fixture = json!({"schema":"uor-r4.completion-evaluation/1","completed_batches":16404,
            "cursor":{"completed_batches":16404,"next_target":1049856,"next_coordinate":0,"corpus_pass":0,"accepted_updates":236},
            "evaluation":{"model_epoch":236},"admission_sha256":"0".repeat(64),"driver_checkpoint_sha256":"1".repeat(64),
            "checkpoint":{"path":"synthetic-metadata-fixture-only","sha256":"2".repeat(64)},
            "sensitivity":{"vector_block":{"attempted":16384,"any_candidate_cost_differs_from_identity":236,"accepted_updates":236},"route":{"attempted":20,"any_candidate_cost_differs_from_identity":0,"accepted_updates":0}}});
        assert_eq!(completion_wrapper(&fixture).unwrap(), 16404);
        let mut wrong = fixture;
        wrong["sensitivity"]["route"]["attempted"] = json!(19);
        assert!(completion_wrapper(&wrong).is_err());
    }
    #[test]
    fn lower_cell_occupancy_and_overflow_boundary() {
        let v = pilot();
        let mut row = v["baseline_evaluation"]["batches"][0]["rows"][0].clone();
        let a = admission(&row).unwrap();
        // Mutation of literal receipt metadata, not execution of a model or reference.
        row["admitted_cells"] = json!(7);
        row["address_rivals"] = json!(7 - a);
        assert!(row_counts(&row).is_ok());
        row["token_rivals"] = json!(i64::MAX);
        assert!(row_counts(&row).is_err());
    }
    #[test]
    fn wrong_seed_plan_lineage_refuses() {
        let mut p: Value = serde_json::from_str(SEED1).unwrap();
        p["execution"]["driver_source_sha256"] = json!("0".repeat(64));
        assert!(plan(&p).is_err());
    }
    #[test]
    fn stored_strict_wins_and_integer_bars_match() {
        let v = pilot();
        let a = analyze(&v["baseline_evaluation"], &v["final_evaluation"]).unwrap();
        assert_eq!(a["metrics"]["baseline"]["admitted_strict_wins"], 0);
        assert_eq!(a["metrics"]["current"]["admitted_strict_wins"], 36);
        assert_eq!(
            a["registered_comparisons"]["rank_bar_minimum_counts"],
            json!({"top1":1941,"top8":4775})
        );
        assert_eq!(
            a["registered_comparisons"]["guard"]["minimum_integer_count"],
            525
        );
    }
    #[test]
    fn checkpoint_payload_parser_uses_archived_model_bytes_and_rejects_mutations() {
        // In-memory parser fixture only. Original CUDA model/receipt bytes are reused;
        // no completion measurement or simulated model computation is produced.
        let path = std::env::var("SCORING_AUDIT_PILOT_CHECKPOINT")
            .expect("set SCORING_AUDIT_PILOT_CHECKPOINT to archived checkpoint-1640.json");
        let (mut checkpoint, digest) = read(&path).expect("read archived checkpoint");
        assert_eq!(
            digest,
            "46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440"
        );
        let report = pilot();
        let plan: Value = serde_json::from_str(SEED1).unwrap();
        checkpoint["schema"] = json!("uor-r4.completion-checkpoint/1");
        checkpoint["admission_sha256"] = json!("0".repeat(64));
        checkpoint["pilot_parent"] = plan["pilot_parent"].clone();
        checkpoint["route"] = json!({"attempted":0,"any_candidate_cost_differs_from_identity":0,"accepted_updates":0});
        let doc = json!({"admission_sha256":checkpoint["admission_sha256"],
            "cursor":checkpoint["checkpoint"]["cursor"],
            "evaluation":report["final_evaluation"],
            "sensitivity":{"vector_block":checkpoint["vector"],"route":checkpoint["route"]},
            "snapshot_metadata":report["P2"]["pilot_end"]});
        let baseline = &report["baseline_evaluation"];
        checkpoint_payload(&doc, &checkpoint, baseline, &plan).unwrap();
        for (pointer, replacement) in [
            ("/baseline_evaluation/model_epoch", json!(1)),
            ("/checkpoint/cursor/completed_batches", json!(1639)),
            ("/checkpoint/model_sha256", json!("0".repeat(64))),
            ("/checkpoint/model/epoch", json!(237)),
            ("/vector/accepted_updates", json!(235)),
        ] {
            let mut bad = checkpoint.clone();
            *bad.pointer_mut(pointer).expect("fixture field exists") = replacement;
            assert!(
                checkpoint_payload(&doc, &bad, baseline, &plan).is_err(),
                "accepted {pointer}"
            );
        }
        let mut bad_doc = doc;
        bad_doc["snapshot_metadata"]["occupied_writer_cells"] = json!(257);
        assert!(checkpoint_payload(&bad_doc, &checkpoint, baseline, &plan).is_err());
    }
}
