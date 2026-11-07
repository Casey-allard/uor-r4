//! Check a prepared token/mask store against all M2 plan strings and accepted answers.
//!
//! `corpus-panel-overlap` decodes each BOS-delimited document and each assistant
//! response run with the pinned tokenizer. It refuses exact plan/query strings in
//! documents, exact accepted answers in assistant replies, or any shared word
//! 6-gram anywhere. The report records both kinds of overlap and example rows.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.corpus-panel-overlap/1";
const NGRAM: usize = 6;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("corpus-panel-overlap: {error}");
        std::process::exit(1);
    }
}

#[derive(Default, Serialize)]
struct Counts {
    documents: usize,
    assistant_replies: usize,
    exact_panel_in_document: usize,
    exact_accepted_assistant_reply: usize,
    documents_with_panel_6gram: usize,
    assistant_replies_with_panel_6gram: usize,
    examples: Vec<BTreeMap<String, String>>,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let flag = |name: &str| {
        args.get(name)
            .cloned()
            .ok_or(format!("--{name} is required"))
    };
    let tokens_path = PathBuf::from(flag("tokens")?);
    let mask_path = PathBuf::from(flag("mask")?);
    let tokenizer_path = PathBuf::from(flag("tokenizer")?);
    let plan_path = PathBuf::from(flag("plan")?);
    let labels_path = PathBuf::from(flag("labels")?);
    let out = PathBuf::from(flag("out")?);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    let tokens_bytes = fs::read(&tokens_path).map_err(|e| e.to_string())?;
    let mask_bytes = fs::read(&mask_path).map_err(|e| e.to_string())?;
    let tokenizer_bytes = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer JSON is not a supported byte-level BPE tokenizer")?;
    let reader = MmapCorpusReader::open(&tokens_path).map_err(|e| e.to_string())?;
    let ids: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
    if ids.len() != mask_bytes.len() {
        return Err(format!(
            "{} tokens against {} mask bytes",
            ids.len(),
            mask_bytes.len()
        ));
    }
    let panel = panel_strings(&read_json(&plan_path)?, &read_json(&labels_path)?)?;
    let panel_words: Vec<Vec<String>> = panel.prompts.iter().map(|s| words(s)).collect();
    let accepted_words: Vec<Vec<String>> = panel.answers.iter().map(|s| words(s)).collect();
    let panel_grams: BTreeSet<String> = panel
        .prompts
        .iter()
        .chain(&panel.answers)
        .flat_map(|s| grams(s))
        .collect();
    let mut counts = Counts::default();
    let mut start = 0usize;
    for end in (0..=ids.len()).filter(|&i| i == ids.len() || ids[i] == 0) {
        if end <= start {
            start = end + 1;
            continue;
        }
        let text = tokenizer.decode(&ids[start..end]);
        let doc_words = words(&text);
        counts.documents += 1;
        let exact = panel_words
            .iter()
            .chain(&accepted_words)
            .filter(|needle| contains_sequence(&doc_words, needle))
            .count();
        counts.exact_panel_in_document += exact;
        let doc_grams: BTreeSet<String> = grams_from(&doc_words);
        if doc_grams.intersection(&panel_grams).next().is_some() {
            counts.documents_with_panel_6gram += 1;
            example(&mut counts, "document", &text);
        }
        let mut reply_start = None;
        for offset in start..=end {
            let inside = offset < end && mask_bytes[offset] == 1;
            match (inside, reply_start) {
                (true, None) => reply_start = Some(offset),
                (false, Some(from)) => {
                    counts.assistant_replies += 1;
                    let reply = tokenizer.decode(&ids[from..offset]);
                    let reply_words = words(&reply);
                    if accepted_words
                        .iter()
                        .any(|answer| normalized_words(answer) == normalized_words(&reply_words))
                    {
                        counts.exact_accepted_assistant_reply += 1;
                        example(&mut counts, "accepted-reply", &reply);
                    }
                    if grams_from(&reply_words)
                        .intersection(&panel_grams)
                        .next()
                        .is_some()
                    {
                        counts.assistant_replies_with_panel_6gram += 1;
                        example(&mut counts, "reply-6gram", &reply);
                    }
                    reply_start = None;
                }
                _ => {}
            }
        }
        if exact > 0 {
            example(&mut counts, "exact-panel", &text);
        }
        start = end + 1;
    }
    let pass = counts.exact_panel_in_document == 0
        && counts.exact_accepted_assistant_reply == 0
        && counts.documents_with_panel_6gram == 0
        && counts.assistant_replies_with_panel_6gram == 0;
    let report = serde_json::json!({
        "schema": SCHEMA,
        "tokens_sha256": sha256(&tokens_bytes),
        "mask_sha256": sha256(&mask_bytes),
        "tokenizer_sha256": sha256(&tokenizer_bytes),
        "plan": file_identity(&plan_path)?,
        "labels": file_identity(&labels_path)?,
        "panel_strings": panel.prompts.len() + panel.answers.len(),
        "panel_prompts": panel.prompts.len(),
        "panel_accepted_answers": panel.answers.len(),
        "counts": counts,
        "pass": pass,
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "pass": pass,
            "out": out,
            "sha256": sha256(&bytes),
            "counts": report["counts"],
        })
    );
    if !pass {
        return Err("panel overlap found; the store is not admissible".to_owned());
    }
    Ok(())
}

struct PanelStrings {
    prompts: BTreeSet<String>,
    answers: BTreeSet<String>,
}

fn panel_strings(plan: &Value, labels: &Value) -> Result<PanelStrings> {
    let histories = plan
        .get("histories")
        .and_then(Value::as_array)
        .ok_or("plan.histories is not an array")?;
    let cases = labels
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("labels.cases is not an array")?;
    let mut prompts = BTreeSet::new();
    let mut answers = BTreeSet::new();
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
                    prompts.insert(text.to_owned());
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
                answers.insert(text.to_owned());
            }
        }
    }
    Ok(PanelStrings { prompts, answers })
}

fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn grams(text: &str) -> BTreeSet<String> {
    grams_from(&words(text))
}

fn grams_from(words: &[String]) -> BTreeSet<String> {
    words
        .windows(NGRAM)
        .map(|window| window.join(" "))
        .collect()
}

fn contains_sequence(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn normalized_words(words: &[String]) -> String {
    words.join(" ")
}

fn example(counts: &mut Counts, kind: &str, text: &str) {
    if counts.examples.len() >= 40 {
        return;
    }
    let example = text.chars().take(240).collect();
    counts.examples.push(BTreeMap::from([
        ("kind".to_owned(), kind.to_owned()),
        ("text".to_owned(), example),
    ]));
}

fn file_identity(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(serde_json::json!({
        "path": path,
        "bytes": bytes.len(),
        "sha256": sha256(&bytes),
    }))
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 6] = ["tokens", "mask", "tokenizer", "plan", "labels", "out"];
    let mut values = BTreeMap::new();
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

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
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
    fn words_and_six_grams_ignore_case_and_punctuation() {
        assert_eq!(words("One, TWO-three!"), ["one", "two", "three"]);
        assert_eq!(
            grams("a b c d e f g"),
            BTreeSet::from(["a b c d e f".to_owned(), "b c d e f g".to_owned()])
        );
    }

    #[test]
    fn sequence_match_requires_the_whole_contiguous_phrase() {
        let hay = words("prefix exact answer suffix");
        assert!(contains_sequence(&hay, &words("exact answer")));
        assert!(!contains_sequence(&hay, &words("exact suffix")));
        assert!(!contains_sequence(&hay, &[]));
    }

    #[test]
    fn malformed_panel_shapes_are_refused() {
        assert!(panel_strings(&serde_json::json!({}), &serde_json::json!({})).is_err());
        assert!(panel_strings(
            &serde_json::json!({"histories": [{"turns": [], "queries": []}]}),
            &serde_json::json!({"cases": []})
        )
        .is_ok());
    }
}
