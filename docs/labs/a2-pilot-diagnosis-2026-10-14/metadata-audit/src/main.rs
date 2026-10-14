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
    let ar = n(row, "address_rivals")?;
    let cells = n(row, "admitted_cells")?;
    let h1 = n(row, "top1_correct")?;
    let h8 = n(row, "top8_correct")?;
    require(h1 <= h8 && h8 <= 1, "invalid rank-hit flags")?;
    require(cells == 8 && ar <= cells, "invalid admitted cells")?;
    let weighted = t
        .checked_mul(8)
        .and_then(|v| a.checked_mul(32).and_then(|w| v.checked_add(w)))
        .ok_or("stored weighted loss overflow")?;
    require(total == weighted, "row weighted loss mismatch")?;
    require(h8 <= admission(row)?, "hit without target admission")?;
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
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    require(
        args.len() == 4,
        "usage: metadata-audit REPORT STEPS EXCLUSIVE_OUTPUT",
    )?;
    let bytes = fs::read(&args[1])?;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    require(sha == REPORT_SHA, "report SHA mismatch")?;
    let report: Value = serde_json::from_slice(&bytes)?;
    let base = &report["baseline_evaluation"];
    let end = &report["final_evaluation"];
    for k in [
        "evaluation_lineage",
        "evaluation_lineage_sha256",
        "scored_set_policy",
        "model_bindings",
        "count_reference",
    ] {
        require(
            base.get(k).is_some() && base.get(k) == end.get(k),
            &format!("evaluation identity mismatch: {k}"),
        )?;
    }
    require(
        base["evaluation_lineage_sha256"] == LINEAGE,
        "unexpected lineage",
    )?;
    require(base["scored_set_policy"]=="response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1","unexpected evaluation order")?;
    for e in [base, end] {
        require(
            e["evaluation_mutates_model"] == false && n(e, "updates_applied_by_evaluation")? == 0,
            "evaluation mutated model",
        )?;
    }
    let before = rows(base)?;
    let after = rows(end)?;
    let mut strict_wins_by_admission: BTreeMap<String, Counts> = BTreeMap::new();
    for (name, rows) in [("baseline", &before), ("final", &after)] {
        let c = strict_wins_by_admission.entry(name.into()).or_default();
        for key in [
            "strict_wins",
            "target_admitted",
            "target_omitted",
            "top1_correct",
            "top8_correct",
        ] {
            c.entry(key.into()).or_default();
        }
        for row in rows {
            if n(row, "token_rivals")? > 0 && n(row, "token_hinge")? < 32 {
                let admitted = admission(row)?;
                require(
                    admitted == 0 || n(row, "top1_correct")? == 1,
                    "admitted strict token win without top1 hit",
                )?;
                add(c, "strict_wins", 1)?;
                add(c, "target_admitted", admitted)?;
                add(c, "target_omitted", 1 - admitted)?;
                add(c, "top1_correct", n(row, "top1_correct")?)?;
                add(c, "top8_correct", n(row, "top8_correct")?)?;
            }
        }
    }
    let mut admission_strata: BTreeMap<String, Counts> = BTreeMap::new();
    let mut hit_strata: BTreeMap<String, Counts> = BTreeMap::new();
    let mut total = Counts::new();
    for (a, b) in before.iter().zip(&after) {
        let key = match (admission(a)?, admission(b)?) {
            (1, 1) => "retained",
            (1, 0) => "lost",
            (0, 1) => "gained",
            _ => "never",
        };
        stratum(admission_strata.entry(key.into()).or_default(), a, b)?;
        let key = match (n(a, "top8_correct")?, n(b, "top8_correct")?) {
            (1, 1) => "retained",
            (1, 0) => "lost",
            (0, 1) => "gained",
            _ => "never",
        };
        stratum(hit_strata.entry(key.into()).or_default(), a, b)?;
        stratum(&mut total, a, b)?;
    }
    let step_bytes = fs::read(&args[2])?;
    require(
        format!("{:x}", Sha256::digest(&step_bytes))
            == "76dda96a16aec709f4d1dc4d12d412307e5c79174d9aa8b9be5444aaef443efc",
        "steps SHA mismatch",
    )?;
    let counts = array(
        &base["count_reference"]["train_masked_reference"],
        "token_counts",
    )?;
    require(counts.len() == 4096, "wrong TRAIN count table size")?;
    let mut cohorts: BTreeMap<String, Counts> = BTreeMap::new();
    let mut accepted = 0_i64;
    let mut step_count = 0;
    for (index, line) in std::str::from_utf8(&step_bytes)?.lines().enumerate() {
        let s: Value = serde_json::from_str(line)?;
        require(
            s["lineage_sha256"] == LINEAGE && s["coordinate"]["kind"] == "vector_block",
            "step scope mismatch",
        )?;
        require(
            n(&s["coordinate"], "index")? == index as i64,
            "step coordinate order mismatch",
        )?;
        require(
            n(&s["cursor_before"], "completed_batches")? == index as i64
                && n(&s["cursor_after"], "completed_batches")? == index as i64 + 1,
            "step cursor mismatch",
        )?;
        require(index < 1640, "excess steps")?;
        let applied = s["applied"].as_bool().ok_or("missing applied flag")?;
        require(
            if applied {
                n(&s, "loss_after")? < n(&s, "loss_before")?
            } else {
                n(&s, "loss_after")? == n(&s, "loss_before")?
            },
            "invalid accepted loss",
        )?;
        accepted += i64::from(applied);
        require(
            n(&s["cursor_after"], "accepted_updates")? == accepted,
            "accepted cursor mismatch",
        )?;
        let count = counts[index / 4].as_u64().ok_or("invalid TRAIN count")?;
        let c = cohorts
            .entry(
                if count == 0 {
                    "zero_masked_train_target_count"
                } else {
                    "nonzero_masked_train_target_count"
                }
                .into(),
            )
            .or_default();
        add(c, "attempted", 1)?;
        add(c, "accepted", i64::from(applied))?;
        add(
            c,
            "sensitive",
            i64::from(
                s["any_candidate_cost_differs_from_identity"]
                    .as_bool()
                    .ok_or("missing sensitivity")?,
            ),
        )?;
        step_count += 1;
    }
    require(
        step_count == 1640 && accepted == 236,
        "step count/acceptance mismatch",
    )?;
    let output = json!({
        "schema":"uor-r4.pilot-diagnosis-metadata/1",
        "scope":"arithmetic over stored GPU receipts; no model computation",
        "report_sha256":sha,"steps_sha256":format!("{:x}",Sha256::digest(&step_bytes)),
        "evaluation_lineage_sha256":LINEAGE,
        "pairing":"ordinal within identical frozen DEV order; 223 batches of 64 plus 15; source training.rs evaluate iterates first=0 then batch.next_target; no explicit target IDs in row receipts",
        "verification":"all row/batch/evaluation sums and weighted numerator identities matched",
        "paired_rows":14287,"admission_transitions":admission_strata,"top8_transitions":hit_strata,
        "overall":total,"coordinate_count_cohorts":cohorts,
        "strict_wins_by_admission":strict_wins_by_admission,
        "strict_win_definition":"stored GPU predicate token_rivals > 0 && token_hinge < width (32); target admission is separate, unchanged guard semantics",
        "limitations":["stored rows have no explicit target ID; pairing relies on pinned source and identical lineage/order", "transition associations do not isolate causal effects of individual updates", "zero target count does not mean zero context/rival influence"],
        "training":"HELD"
    });
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    file.write_all(&serde_json::to_vec_pretty(&output)?)?;
    file.write_all(b"\n")?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
