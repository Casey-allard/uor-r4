//! Build the deterministic M2 exclusion input for `m-world-recall-corpus`.
//!
//! `m2-sft-exclusions --plan PLAN --labels LABELS --out FILE` reads every plan
//! statement and query plus every frozen accepted answer, sorts and deduplicates
//! the strings, and writes `[{"user_turns": [string]}, ...]`.

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("m2-sft-exclusions: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let flag = |name: &str| {
        args.get(name)
            .cloned()
            .ok_or(format!("--{name} is required"))
    };
    let plan_path = PathBuf::from(flag("plan")?);
    let labels_path = PathBuf::from(flag("labels")?);
    let out = PathBuf::from(flag("out")?);
    let plan: Value = read_json(&plan_path)?;
    let labels: Value = read_json(&labels_path)?;
    let strings = exclusions(&plan, &labels)?;
    if strings.is_empty() {
        return Err("the M2 plan and labels hold no exclusion strings".to_owned());
    }
    let rows: Vec<Value> = strings
        .into_iter()
        .map(|text| json!({"user_turns": [text]}))
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&rows).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn exclusions(plan: &Value, labels: &Value) -> Result<BTreeSet<String>> {
    let histories = plan
        .get("histories")
        .and_then(Value::as_array)
        .ok_or("plan.histories is not an array")?;
    let cases = labels
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("labels.cases is not an array")?;
    let mut strings = BTreeSet::new();
    for history in histories {
        for key in ["turns", "queries"] {
            let rows = history
                .get(key)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("plan history has no {key} array"))?;
            for row in rows {
                let text = row
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("plan {key} row has no text string"))?;
                if !text.trim().is_empty() {
                    strings.insert(text.to_owned());
                }
            }
        }
    }
    for case in cases {
        let accepted = case
            .get("answers")
            .and_then(|answers| answers.get("accepted"))
            .and_then(Value::as_array)
            .ok_or("label case has no answers.accepted array")?;
        for answer in accepted {
            let text = answer
                .as_str()
                .ok_or("an accepted answer is not a string")?;
            if !text.trim().is_empty() {
                strings.insert(text.to_owned());
            }
        }
    }
    Ok(strings)
}

fn parse_args(
    args: impl Iterator<Item = String>,
) -> Result<std::collections::BTreeMap<String, String>> {
    const FLAGS: [&str; 3] = ["plan", "labels", "out"];
    let mut values = std::collections::BTreeMap::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        let name = arg
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or(format!("unknown argument {arg}"))?;
        let value = args.next().ok_or(format!("{arg} needs a value"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("{arg} given twice"));
        }
    }
    Ok(values)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_and_sorts_every_plan_and_label_string_once() {
        let plan = json!({
            "histories": [{
                "turns": [{"text": "B"}, {"text": "A"}],
                "queries": [{"text": "Question"}, {"text": "B"}]
            }]
        });
        let labels = json!({
            "cases": [{
                "answers": {"accepted": ["Answer", "A"]}
            }]
        });
        let got: Vec<String> = exclusions(&plan, &labels).unwrap().into_iter().collect();
        assert_eq!(got, ["A", "Answer", "B", "Question"]);
    }

    #[test]
    fn malformed_shapes_are_refused() {
        assert!(exclusions(&json!({}), &json!({"cases": []})).is_err());
        assert!(exclusions(
            &json!({"histories": [{"turns": [], "queries": []}]}),
            &json!({})
        )
        .is_err());
    }
}
