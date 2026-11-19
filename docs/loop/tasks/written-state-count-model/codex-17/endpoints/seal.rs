// Requested-person custody only. Deliberately has no model or outcome interface.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load(path: &Path) -> Result<Vec<Value>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn save(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

fn question_start(text: &str) -> Result<usize> {
    let (prior, last) = text
        .rsplit_once(". ")
        .ok_or("no history/question separator")?;
    if last == "Can its exact object be determined?" {
        let (facts, query_intro) = prior
            .rsplit_once(". ")
            .ok_or("incomplete conflict question")?;
        if !query_intro.starts_with("The repeated entries for ") {
            return Err("unexpected preface in two-sentence question".into());
        }
        return Ok(facts.len() + 2);
    }
    Ok(prior.len() + 2)
}

fn complete(text: &str, start: usize, end: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
        && text[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
}

fn batch(input: &Path, out: &Path, stem: &str, groups: &[(&str, usize)]) -> Result<()> {
    let mut text_rows = Vec::new();
    let mut sealed_rows = Vec::new();
    let mut audit = Vec::new();
    for &(group, count) in groups {
        let raw = load(&input.join(format!("{group}-raw.json")))?;
        let a = load(&input.join(format!("{group}-person-a.json")))?;
        let b = load(&input.join(format!("{group}-person-b.json")))?;
        if raw.len() != count || a.len() != count || b.len() != count || a != b {
            return Err(format!("{group} census or independent person agreement failed").into());
        }
        for (i, (line, person)) in raw.into_iter().zip(a).enumerate() {
            let text = line["text"].as_str().ok_or("missing unmarked text")?;
            if line["index"] != i
                || person["index"] != i
                || line.as_object().map(|m| m.len()) != Some(2)
                || person.as_object().map(|m| m.len()) != Some(2)
            {
                return Err(format!("{group}:{i} ordinal or field envelope failed").into());
            }
            let start = question_start(text)?;
            let name = &person["person"];
            let requested = if let Some(name) = name.as_str() {
                if name.is_empty() || name.trim() != name {
                    return Err(format!("{group}:{i} malformed agreed name").into());
                }
                let matches: Vec<_> = text[start..]
                    .match_indices(name)
                    .filter_map(|(at, value)| {
                        let left = start + at;
                        let right = left + value.len();
                        complete(text, left, right).then_some([left, right])
                    })
                    .collect();
                let [bytes] = matches.as_slice() else {
                    return Err(format!(
                        "{group}:{i} expected one whole agreed name in question, got {matches:?}"
                    )
                    .into());
                };
                json!({"name":name,"bytes":bytes})
            } else if name.is_null() && person.get("person").is_some() {
                Value::Null
            } else {
                return Err(format!("{group}:{i} unsupported person type").into());
            };
            let index = text_rows.len();
            text_rows.push(json!({"index":index,"text":text}));
            sealed_rows.push(
                json!({"index":index,"text_sha256":hash(text.as_bytes()),"requested":requested}),
            );
            audit.push(
                json!({"index":index,"original_group":group,"original_index":i,
                 "question_bytes":[start,text.len()],"requested_present":!requested.is_null()}),
            );
        }
    }
    let raw = out.join(format!("{stem}-raw.json"));
    save(&raw, &json!(text_rows))?;
    save(
        &out.join(format!("{stem}-names.json")),
        &json!({
      "schema":"uor-r4.sealed-requested-names/1",
      "raw_sha256":hash(&fs::read(raw)?),"rows":sealed_rows}),
    )?;
    save(
        &out.join(format!("{stem}-ordinal-audit.json")),
        &json!(audit),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [input, out] = args.as_slice() else {
        return Err("seal INPUTS NEW_OUTPUT".into());
    };
    let out = Path::new(out);
    fs::create_dir(out)?;
    batch(
        Path::new(input),
        out,
        "fit",
        &[("base-fit", 42), ("supplement", 52)],
    )?;
    batch(Path::new(input), out, "held", &[("held", 42)])?;
    Ok(())
}
