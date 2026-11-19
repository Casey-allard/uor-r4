//! Resolve an independently read person inside the final ordinary clause.
use super::{sha, Result};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn seal(raw: &Path, people: &Path, out: &Path) -> Result<()> {
    let source = fs::read(raw)?;
    let prompts: Vec<Value> = serde_json::from_slice(&source)?;
    let readings: Vec<Value> = serde_json::from_slice(&fs::read(people)?)?;
    if prompts.len() != 70 || readings.len() != prompts.len() {
        return Err("missing or excess read ordinals".into());
    }
    let mut rows = Vec::with_capacity(prompts.len());
    for (index, (prompt, reading)) in prompts.iter().zip(readings.iter()).enumerate() {
        if prompt["index"] != index
            || reading["index"] != index
            || reading.as_object().map(|object| object.len()) != Some(2)
            || reading.get("person").is_none()
        {
            return Err(format!("row {index}: expected only ordinal and person").into());
        }
        let text = prompt["text"].as_str().ok_or("missing raw prompt")?;
        let requested = match &reading["person"] {
            Value::Null => Value::Null,
            Value::String(person)
                if !person.is_empty() && person.bytes().all(|b| b.is_ascii_alphabetic()) =>
            {
                let (facts, question) = text.rsplit_once(". ").ok_or("missing question")?;
                let base = facts.len() + 2;
                let bytes = question.as_bytes();
                let spans: Vec<_> = question
                    .match_indices(person)
                    .filter_map(|(start, word)| {
                        let end = start + word.len();
                        let before = start
                            .checked_sub(1)
                            .and_then(|position| bytes.get(position));
                        let after = bytes.get(end);
                        (!before.is_some_and(u8::is_ascii_alphanumeric)
                            && !after.is_some_and(u8::is_ascii_alphanumeric))
                        .then_some([base + start, base + end])
                    })
                    .collect();
                let [span] = spans.as_slice() else {
                    return Err(
                        format!("row {index}: person has no unique question occurrence").into(),
                    );
                };
                json!({"name":person,"bytes":span})
            }
            _ => return Err(format!("row {index}: invalid person").into()),
        };
        rows.push(json!({"index":index,"text_sha256":sha(text.as_bytes()),"requested":requested}));
    }
    let mut file = OpenOptions::new().create_new(true).write(true).open(out)?;
    serde_json::to_writer_pretty(
        &mut file,
        &json!({
            "schema":"uor-r4.sealed-requested-names/1",
            "raw_sha256":sha(&source),
            "rows":rows,
        }),
    )?;
    writeln!(file)?;
    Ok(())
}
