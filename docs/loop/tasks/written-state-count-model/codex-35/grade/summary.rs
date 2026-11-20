#![forbid(unsafe_code)]
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ARMS: [&str; 5] = [
    "baseline",
    "fact_true",
    "fact_reversed",
    "candidate_true",
    "candidate_reversed",
];
const COMPARISONS: [(&str, &str); 5] = [
    ("baseline", "fact_true"),
    ("fact_reversed", "fact_true"),
    ("baseline", "candidate_true"),
    ("candidate_reversed", "candidate_true"),
    ("fact_true", "candidate_true"),
];
fn arm<'a>(r: &'a Value, a: &str) -> &'a Value {
    &r["arms"][a]
}
fn state(a: &Value) -> String {
    if a["exact_eos"] == true {
        "exact_eos".into()
    } else if let Some(pos) = a["first_wrong"]["offset"].as_u64() {
        format!("offset_{pos}")
    } else {
        "unkeyed".into()
    }
}
fn counts(rows: &[&Value], a: &str) -> Value {
    let count = |field: &str| rows.iter().filter(|r| arm(r, a)[field] == true).count();
    let mut failure = BTreeMap::new();
    let mut exclusions = 0;
    let mut exclusion_hits = 0;
    for r in rows {
        *failure.entry(state(arm(r, a))).or_insert(0) += 1;
        if !r["excluded"].is_null() {
            exclusions += 1;
            exclusion_hits += usize::from(arm(r, a)["excluded_correct"] == true);
        }
    }
    json!({"n":rows.len(),"supported":rows.iter().filter(|r|!r["expected"].is_null()).count(),"exact_eos":count("exact_eos"),"correct_source":count("correct_source"),"wrong_source":count("wrong_source"),"withheld":count("withheld"),"unsupported_copy":count("unsupported_copy"),"requested_correct":count("requested_correct"),"explicit_exclusions":exclusions,"explicit_exclusion_correct":exclusion_hits,"assertion_bound":count("assertion_bound"),"eligible_onset":count("eligible_gold"),"eligible_complete":count("eligible_complete"),"first_error":failure})
}
fn compare(rows: &[&Value], from: &str, to: &str) -> Value {
    let mut transitions = BTreeMap::new();
    let mut group_delta = BTreeMap::new();
    let mut exact_gain = Vec::new();
    let mut exact_loss = Vec::new();
    let mut new_wrong = Vec::new();
    let mut new_unsupported = Vec::new();
    let mut recovered_source = Vec::new();
    let mut lost_source = Vec::new();
    for r in rows {
        let (a, b) = (arm(r, from), arm(r, to));
        let row = &r["row"];
        *transitions
            .entry(format!("{} -> {}", state(a), state(b)))
            .or_insert(0) += 1;
        if !r["expected"].is_null() {
            let delta = i32::from(b["exact_eos"] == true) - i32::from(a["exact_eos"] == true);
            *group_delta.entry(r["group"].to_string()).or_insert(0) += delta;
            if delta > 0 {
                exact_gain.push(row)
            } else if delta < 0 {
                exact_loss.push(row)
            }
        }
        if a["correct_source"] != true && b["correct_source"] == true {
            recovered_source.push(row)
        }
        if a["correct_source"] == true && b["correct_source"] != true {
            lost_source.push(row)
        }
        if a["wrong_source"] != true && b["wrong_source"] == true {
            new_wrong.push(row)
        }
        if a["unsupported_copy"] != true && b["unsupported_copy"] == true {
            new_unsupported.push(row)
        }
    }
    json!({"from":from,"to":to,"first_error_transitions":transitions,"exact_recovered_rows":exact_gain,"exact_lost_rows":exact_loss,"new_wrong_rows":new_wrong,"new_unsupported_rows":new_unsupported,"source_recovered_rows":recovered_source,"source_lost_rows":lost_source,"exact_delta_by_group":group_delta})
}
fn main() -> Result<()> {
    let f = std::env::args()
        .nth(1)
        .ok_or("usage: d35-summary GRADED_ROWS")?;
    let data: Value = serde_json::from_reader(fs::File::open(f)?)?;
    let mut modes = BTreeMap::<String, Value>::new();
    let all = data["rows"].as_array().ok_or("missing graded rows")?;
    for mode in ["ordinary", "supplied"] {
        let rows: Vec<_> = all.iter().filter(|r| r["mode"] == mode).collect();
        let mut slices = BTreeMap::<String, Vec<&Value>>::new();
        for bucket in ["target_at_most_7", "target_at_least_8"] {
            slices.entry(bucket.into()).or_default();
        }
        for &row in &rows {
            if let Some(span) = row["gold_source"].as_array() {
                let n = span[1].as_u64().ok_or("end")? - span[0].as_u64().ok_or("start")?;
                let bucket = if n <= 8 {
                    "target_at_most_8"
                } else {
                    "target_over_8"
                };
                slices.entry(bucket.into()).or_default().push(row);
                let writer_bucket = if n <= 7 {
                    "target_at_most_7"
                } else {
                    "target_at_least_8"
                };
                slices.entry(writer_bucket.into()).or_default().push(row);
                if let Some(envelope) = arm(row, "baseline")["writer_envelope"].as_str() {
                    slices
                        .entry(format!("baseline_{envelope}"))
                        .or_default()
                        .push(row);
                }
            }
            slices.entry("all".into()).or_default().push(row);
            slices
                .entry(row["stratum"].as_str().ok_or("stratum")?.into())
                .or_default()
                .push(row);
        }
        let mut breakdown = BTreeMap::new();
        for (name, subset) in slices {
            let scores: BTreeMap<_, _> = ARMS.iter().map(|a| (a, counts(&subset, a))).collect();
            let paired: Vec<_> = COMPARISONS
                .iter()
                .map(|(a, b)| compare(&subset, a, b))
                .collect();
            breakdown.insert(name, json!({"counts":scores,"comparisons":paired}));
        }
        modes.insert(mode.into(),json!({"applicable":rows.len(),"not_applicable":data["n_held"].as_u64().ok_or("n_held")?-rows.len() as u64,"slices":breakdown}));
    }
    println!("{}", serde_json::to_string_pretty(&modes)?);
    Ok(())
}
