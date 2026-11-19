use super::{ids, read, save, valid, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs};
use uor_r4_tokenizer::ByteBpeTokenizer;

const ROWS: usize = 42;
const POSITIVES: usize = 36;
const MAX_TOKENS: usize = 16;
const CONTEXT: u32 = 192;

fn array<'a>(value: &'a Value, label: &str) -> Result<&'a [Value]> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{label} is not an array").into())
}

fn field<'a>(row: &'a Value, name: &str) -> Result<&'a Value> {
    row.get(name)
        .ok_or_else(|| format!("missing {name}").into())
}

fn string<'a>(row: &'a Value, name: &str) -> Result<&'a str> {
    field(row, name)?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("invalid {name}").into())
}

fn hash(path: &str) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn legal_ids(tokens: &[u32], tokenizer: &ByteBpeTokenizer) -> bool {
    let unknown = tokenizer.token_id("<|unk|>");
    tokens
        .iter()
        .all(|&id| id != 0 && Some(id) != unknown && tokenizer.token_byte_len(id).is_some())
}

fn first_wrong(actual: &[u32], expected: &[u32]) -> Option<usize> {
    actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
        .or_else(|| (actual.len() != expected.len()).then_some(actual.len().min(expected.len())))
}

struct Literal<'a> {
    answer: &'a str,
    ids: Vec<u32>,
}

impl<'a> Literal<'a> {
    fn from_label(label: &'a Value, tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        let answer = string(label, "answer_text")?;
        if answer.trim() != answer || answer.chars().any(char::is_control) {
            return Err("literal includes extra framing or control characters".into());
        }
        let mut canonical = tokenizer.encode(&format!(" {answer}"));
        canonical.push(1);
        let expected = ids(field(label, "expected")?)?;
        if expected.len() < 2
            || expected.len() > MAX_TOKENS
            || !legal_ids(&expected, tokenizer)
            || !valid(&expected, &canonical, answer, tokenizer)
        {
            return Err("literal label fails independent bytes, IDs or EOS".into());
        }
        Ok(Self {
            answer,
            ids: expected,
        })
    }

    fn exact(&self, actual: &[u32], tokenizer: &ByteBpeTokenizer) -> bool {
        legal_ids(actual, tokenizer) && valid(actual, &self.ids, self.answer, tokenizer)
    }

    fn matches_fact(&self, fact: &Value) -> Result<bool> {
        Ok(field(fact, "value")? == self.answer
            && array(field(fact, "legal")?, "legal candidates")?
                .iter()
                .any(|candidate| {
                    candidate["start"] == fact["value_span"][0]
                        && candidate["copy_ids_through_eos"] == json!(self.ids)
                }))
    }
}

struct Labels<'a> {
    status: &'static str,
    original: Option<Literal<'a>>,
    active: Option<Literal<'a>>,
}

fn check_labels<'a>(
    original: &'a Value,
    active: &'a Value,
    reading: &Value,
    index: usize,
    mutated: bool,
    tokenizer: &ByteBpeTokenizer,
) -> Result<Labels<'a>> {
    let old = original.as_object().ok_or("invalid original label")?;
    let new = active.as_object().ok_or("invalid active label")?;
    if old.len() != new.len()
        || old.iter().any(|(key, val)| {
            let allow = mutated && matches!(key.as_str(), "answer_text" | "expected");
            new.get(key).is_none_or(|other| !allow && val != other)
        })
    {
        return Err("label file changed an original row binding".into());
    }
    let (status, support_count) = match string(original, "stratum")? {
        "supported" => ("supported", 1),
        "unmentioned" => ("absent", 0),
        "conflict" => ("conflict", 2),
        "unspecified" => ("unspecified", 0),
        _ => return Err("unknown original stratum".into()),
    };
    let facts = array(field(original, "facts")?, "facts")?;
    let requested = field(original, "requested_name")?;
    let supporting = facts
        .iter()
        .enumerate()
        .filter_map(|(j, fact)| (fact["name"] == *requested).then_some(j))
        .collect::<Vec<_>>();
    if field(reading, "index")? != index
        || field(reading, "status")? != status
        || field(reading, "person")? != requested
        || string(reading, "why").is_err()
        || requested.is_null() != (status == "unspecified")
        || supporting.len() != support_count
    {
        return Err("independent reading disagrees with scenario identity or support".into());
    }
    if status != "supported" {
        if [
            field(original, "expected")?,
            field(original, "answer_text")?,
            field(active, "expected")?,
            field(active, "answer_text")?,
            field(reading, "answer")?,
        ]
        .iter()
        .any(|v| !v.is_null())
        {
            return Err("unsupported row carries a literal label".into());
        }
        if status == "conflict"
            && string(&facts[supporting[0]], "value")? == string(&facts[supporting[1]], "value")?
        {
            return Err("conflict row repeats the same value".into());
        }
        return Ok(Labels {
            status,
            original: None,
            active: None,
        });
    }
    let old = Literal::from_label(original, tokenizer)?;
    let new = Literal::from_label(active, tokenizer)?;
    let gold = supporting[0];
    if field(reading, "answer")? != old.answer
        || field(original, "gold_assertion_index")? != gold
        || !old.matches_fact(&facts[gold])?
    {
        return Err("independent noun phrase disagrees with the original fact".into());
    }
    if mutated {
        let other: Vec<_> = facts
            .iter()
            .filter(|fact| fact["name"] != *requested)
            .collect();
        if other.len() != 1
            || new.ids == old.ids
            || new.answer == old.answer
            || !new.matches_fact(other[0])?
        {
            return Err("mutated label is not the other entity's distinct recorded literal".into());
        }
    }
    Ok(Labels {
        status,
        original: Some(old),
        active: Some(new),
    })
}

fn span(value: &Value) -> Result<Option<[u32; 2]>> {
    let range: Option<[u32; 2]> = serde_json::from_value(value.clone())?;
    if range.is_some_and(|[start, end]| start >= end || end > CONTEXT) {
        return Err("invalid role span".into());
    }
    Ok(range)
}

fn public_yes_no(bytes: &[u8], eos: bool) -> Option<&'static str> {
    if !eos {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?.trim();
    let split = text.find(|ch: char| ch.is_whitespace() || ".,!?;:".contains(ch));
    let word = split.map_or(text, |at| &text[..at]);
    if word.eq_ignore_ascii_case("no") {
        Some("no")
    } else if word.eq_ignore_ascii_case("yes") {
        Some("yes")
    } else {
        None
    }
}

fn inspect(
    index: usize,
    label: &Value,
    reading: &Value,
    labels: &Labels<'_>,
    writer: &Value,
    public: bool,
    tokenizer: &ByteBpeTokenizer,
) -> Result<Value> {
    let actual = ids(field(writer, "generated")?)?;
    if actual.is_empty()
        || actual.len() > MAX_TOKENS
        || actual
            .iter()
            .any(|&id| tokenizer.token_byte_len(id).is_none())
    {
        return Err("empty, over-budget, unassigned or out-of-vocabulary emission".into());
    }
    let eos = actual.last() == Some(&1);
    if actual[..actual.len() - 1].contains(&1) || field(writer, "terminal_eos")? != eos {
        return Err("archived trace has inconsistent or interior EOS".into());
    }
    if public && !eos && actual.len() != MAX_TOKENS {
        return Err("public trace stopped before the ordinary EOS or token limit".into());
    }
    let body = actual.strip_suffix(&[1]).unwrap_or(&actual);
    let body_has_special = !legal_ids(body, tokenizer);
    let decoded = tokenizer.decode_bytes(body);
    let visible = String::from_utf8_lossy(&decoded);
    let writer_text = field(writer, "text")?;
    if (public && writer_text.as_str() != Some(visible.as_ref()))
        || (!public && !writer_text.is_null())
    {
        return Err("public text does not match independently decoded bytes".into());
    }
    let nomination = span(field(writer, "requested")?)?;
    let gold_nomination = span(field(writer, "gold_requested")?)?;
    let source: Option<u32> = serde_json::from_value(field(writer, "source")?.clone())?;
    let source_matches = array(field(label, "facts")?, "facts")?.iter().any(|fact| {
        fact["name"] == label["requested_name"] && fact["value_span"][0] == json!(source)
    });
    if field(writer, "source_matches_requested_value")? != source_matches
        || source.is_some_and(|start| start >= CONTEXT)
        || gold_nomination != span(field(label, "requested")?)?
        || field(writer, "endpoint_exact")? != (nomination == gold_nomination)
        || field(writer, "acceptable_prose")? != field(label, "prose_acceptable")?
    {
        return Err("role nomination, source or requested endpoint is misbound".into());
    }
    let original_exact = labels
        .original
        .as_ref()
        .map(|lit| lit.exact(&actual, tokenizer));
    let active_exact = labels
        .active
        .as_ref()
        .map(|lit| lit.exact(&actual, tokenizer));
    let original_first = labels
        .original
        .as_ref()
        .and_then(|lit| first_wrong(&actual, &lit.ids));
    let active_first = labels
        .active
        .as_ref()
        .and_then(|lit| first_wrong(&actual, &lit.ids));
    let active_ids = labels.active.as_ref().map(|lit| &lit.ids);
    let expected_wrong = active_first.and_then(|i| active_ids?.get(i));
    let actual_wrong = active_first.and_then(|i| actual.get(i));
    if field(writer, "expected")? != &json!(active_ids)
        || field(writer, "exact")? != &json!(active_exact)
        || field(writer, "first_wrong")? != &json!(active_first)
        || field(writer, "expected_wrong_id")? != &json!(expected_wrong)
        || field(writer, "actual_wrong_id")? != &json!(actual_wrong)
    {
        return Err("archived writer verdict disagrees with IDs, bytes or EOS".into());
    }
    let negative = if labels.status == "supported" {
        Value::Null
    } else {
        if !public {
            return Err("unsupported row is outside the public comparison".into());
        }
        let decision = public_yes_no(&decoded, eos && !body_has_special);
        let mode = if source.is_some() {
            "copy"
        } else if decision == Some("no") {
            "no"
        } else {
            "fallback"
        };
        let recorded_literal = array(field(label, "facts")?, "facts")?.iter().any(|fact| {
            fact["value"]
                .as_str()
                .is_some_and(|value| value == visible.trim())
        });
        json!({ "classification": mode, "source": source, "role_nomination": nomination,
            "normalized_yes_no": decision, "no_without_copy": mode == "no",
            "matches_recorded_literal": recorded_literal, "expected_prose": label["prose_acceptable"],
            "independent_reason": field(reading, "why")?, "needs_human_semantic_review": true })
    };
    Ok(json!({
        "index": index, "id": label["id"], "semantics": labels.status, "generated": actual,
        "decoded_bytes": decoded, "decoded_text": visible, "terminal_eos": eos,
        "body_has_special": body_has_special, "source": source,
        "role_nomination": nomination, "gold_role_nomination": gold_nomination,
        "independent_answer": labels.original.as_ref().map(|lit| lit.answer),
        "original_expected": labels.original.as_ref().map(|lit| &lit.ids),
        "original_exact": original_exact, "original_first_wrong": original_first,
        "active_answer": labels.active.as_ref().map(|lit| lit.answer),
        "active_expected": active_ids, "active_exact": active_exact, "active_first_wrong": active_first,
        "active_expected_wrong_id": expected_wrong, "active_actual_wrong_id": actual_wrong,
        "negative": negative
    }))
}

pub(super) fn score(args: &[String]) -> Result<()> {
    let [tokenizer, active, readings, original, writer, out] = args else {
        return Err("TOKENIZER GRADES INDEPENDENT_READINGS ORIGINAL_GRADES_OR_MINUS OBSERVED_WRITER_GRADE_JSON OUT".into());
    };
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tokenizer)?)
        .ok_or("invalid tokenizer")?;
    if tokenizer.token_id("<|bos|>") != Some(0)
        || tokenizer.token_id("<|eos|>") != Some(1)
        || tokenizer.token_id("<|unk|>") != Some(2)
    {
        return Err("incorrect protocol token identity".into());
    }
    let mutated = original != "-";
    let original_path = if mutated { original } else { active };
    let observed = read(writer)?;
    let active_hash = hash(active)?;
    if field(&observed, "schema")? != "uor-r4.ordered-role.grades/1"
        || field(&observed, "grades")? != &active_hash
        || ["selection", "gpu"].iter().any(|key| {
            observed[key]
                .as_str()
                .is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        })
    {
        return Err("writer grade receipt is unbound to the supplied labels or archive".into());
    }
    let original_rows = read(original_path)?;
    let active_rows = read(active)?;
    let independent = read(readings)?;
    let old = array(&original_rows, "original grades")?;
    let new = array(&active_rows, "active grades")?;
    let readings_rows = array(&independent, "independent readings")?;
    let trace = array(field(&observed, "rows")?, "observed archive")?;
    if [old.len(), new.len(), readings_rows.len()] != [ROWS; 3]
        || !matches!(trace.len(), ROWS | POSITIVES)
        || (mutated && trace.len() != POSITIVES)
    {
        return Err("unexpected panel or archive row counts".into());
    }
    let mut ids_seen = BTreeSet::new();
    let mut labels = Vec::with_capacity(ROWS);
    for (index, ((old, new), reading)) in old.iter().zip(new).zip(readings_rows).enumerate() {
        if !ids_seen.insert(string(old, "id")?) {
            return Err(format!("duplicate authored row ID at {index}").into());
        }
        let parsed = check_labels(old, new, reading, index, mutated, &tokenizer)
            .map_err(|error| format!("label row {index}: {error}"))?;
        labels.push(parsed);
    }
    if labels
        .iter()
        .filter(|label| label.status == "supported")
        .count()
        != POSITIVES
        || labels
            .iter()
            .filter(|label| label.status == "absent")
            .count()
            != 2
        || labels
            .iter()
            .filter(|label| label.status == "conflict")
            .count()
            != 2
        || labels
            .iter()
            .filter(|label| label.status == "unspecified")
            .count()
            != 2
    {
        return Err("original panel does not contain the declared cases".into());
    }
    let public = trace.len() == ROWS;
    let mut records = Vec::with_capacity(trace.len());
    let mut seen = BTreeSet::new();
    for row in trace {
        let index = field(row, "index")?
            .as_u64()
            .and_then(|index| usize::try_from(index).ok())
            .filter(|&index| index < ROWS)
            .ok_or("unrecognized archive index")?;
        let label = &new[index];
        if !seen.insert(index)
            || [
                "id",
                "stratum",
                "family",
                "pair",
                "direction",
                "counterpart",
            ]
            .iter()
            .any(|key| row.get(key) != Some(&label[*key]))
            || (!public && labels[index].status != "supported")
        {
            return Err(format!("duplicate or misbound archive row {index}").into());
        }
        records.push(
            inspect(
                index,
                label,
                &readings_rows[index],
                &labels[index],
                row,
                public,
                &tokenizer,
            )
            .map_err(|error| format!("archive row {index}: {error}"))?,
        );
    }
    records.sort_by_key(|row| row["index"].as_u64());
    let mut tape = Sha256::new();
    for row in &records {
        let index = row["index"].as_u64().ok_or("invalid output row")?;
        let emitted = ids(&row["generated"])?;
        tape.update(index.to_le_bytes());
        tape.update((emitted.len() as u64).to_le_bytes());
        for id in emitted {
            tape.update(id.to_le_bytes());
        }
    }
    let count = |key: &str, value: Value| records.iter().filter(|row| row[key] == value).count();
    let negatives = |kind: &str| {
        records
            .iter()
            .filter(|row| row["negative"]["classification"] == kind)
            .count()
    };
    save(
        out,
        &json!({
            "schema":"uor-r4.value-rank-observed-grader/1",
            "mutated_labels":mutated, "public":public,
            "original_grades_sha256":hash(original_path)?, "active_grades_sha256":active_hash,
            "independent_readings_sha256":hash(readings)?, "writer_grade_sha256":hash(writer)?,
            "selection_sha256":observed["selection"], "gpu_sha256":observed["gpu"],
            "generated_id_tape_sha256":format!("{:x}", tape.finalize()),
            "counts":{
                "rows":records.len(), "supported":POSITIVES, "independently_exact":count("original_exact", json!(true)),
                "active_label_exact":count("active_exact", json!(true)),
                "negative_no":negatives("no"), "negative_copy":negatives("copy"),
                "negative_fallback":negatives("fallback")
            },
            "rows":records
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires pinned tokenizer files and decoded v5 fixture custody"]
    fn reread_v5_and_check_finite_mutation() -> Result<()> {
        let panel = std::env::var("UOR_RANK_PANEL_ROOT")?;
        let assets = std::env::var("UOR_RANK_TOKENIZER_DIR")?;
        for seed in [1, 2] {
            let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(format!(
                "{assets}/tokenizer-seed{seed}.json"
            ))?)
            .ok_or("invalid v5 tokenizer")?;
            for split in ["fit", "held"] {
                let grades = read(&format!(
                    "{panel}/value-rank-panel-v5/seed{seed}/{split}/grades.json"
                ))?;
                let grades = array(&grades, "grades")?;
                let mut writer_full = Vec::new();
                for reader in ["a", "b"] {
                    let prefix = if split == "fit" { "v4" } else { "v5" };
                    let readings = read(&format!("{panel}/{prefix}-reader-{reader}-{split}.json"))?;
                    let readings = array(&readings, "readings")?;
                    assert_eq!(grades.len(), ROWS);
                    assert_eq!(readings.len(), ROWS);
                    for (index, (label, reading)) in grades.iter().zip(readings).enumerate() {
                        check_labels(label, label, reading, index, false, &tokenizer).map_err(
                            |error| format!("seed {seed} {split} {reader} {index}: {error}"),
                        )?;
                    }
                }
                let reading = read(&if split == "fit" {
                    format!("{panel}/v4-reader-a-fit.json")
                } else {
                    format!("{panel}/v5-reader-a-held.json")
                })?;
                for (index, label) in grades.iter().enumerate() {
                    let active =
                        check_labels(label, label, &reading[index], index, false, &tokenizer)?;
                    let actual = active
                        .original
                        .as_ref()
                        .map_or(vec![1], |lit| lit.ids.clone());
                    writer_full.push(authored(label, index, &actual, true, &tokenizer));
                }
                let labels_path = format!("{panel}/value-rank-panel-v5/seed{seed}/{split}/grades.json");
                let temp = std::env::temp_dir().join(format!(
                    "uor-rank-grade-{}-{seed}-{split}",
                    std::process::id()
                ));
                fs::create_dir(&temp)?;
                for n in [ROWS, POSITIVES] {
                    let trace = if n == ROWS {
                        writer_full.clone()
                    } else {
                        writer_full
                            .iter()
                            .filter(|row| row["stratum"] == "supported")
                            .map(|row| {
                                let mut row = row.clone();
                                row["text"] = Value::Null;
                                row
                            })
                            .collect()
                    };
                    let writer = temp.join(format!("writer-{n}.json"));
                    let out = temp.join(format!("out-{n}.json"));
                    save(
                        writer.to_str().unwrap(),
                        &json!({
                            "schema":"uor-r4.ordered-role.grades/1", "grades": hash(&labels_path)?,
                            "selection":"a".repeat(64), "gpu":"b".repeat(64), "rows":trace
                        }),
                    )?;
                    let reader = if split == "fit" {
                        format!("{panel}/v4-reader-a-fit.json")
                    } else {
                        format!("{panel}/v5-reader-a-held.json")
                    };
                    score(&[
                        format!(
                            "{assets}/tokenizer-seed{seed}.json"
                        ),
                        labels_path.clone(),
                        reader.into(),
                        "-".into(),
                        writer.to_str().unwrap().into(),
                        out.to_str().unwrap().into(),
                    ])?;
                    let report = read(out.to_str().unwrap())?;
                    assert_eq!(report["counts"]["rows"], n);
                    assert_eq!(report["counts"]["independently_exact"], 36);
                }
                let source = &grades[0];
                let reading = &reading[0];
                let original = check_labels(source, source, reading, 0, false, &tokenizer)?;
                let actual = &original.original.as_ref().unwrap().ids;
                let other = &source["facts"][1];
                let alternative = other["legal"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|candidate| candidate["start"] == other["value_span"][0])
                    .unwrap();
                let mut changed = source.clone();
                changed["answer_text"] = other["value"].clone();
                changed["expected"] = alternative["copy_ids_through_eos"].clone();
                let active = check_labels(source, &changed, reading, 0, true, &tokenizer)?;
                let exact = inspect(
                    0,
                    source,
                    reading,
                    &original,
                    &authored(source, 0, actual, false, &tokenizer),
                    false,
                    &tokenizer,
                )?;
                let wrong = inspect(
                    0,
                    &changed,
                    reading,
                    &active,
                    &authored(&changed, 0, actual, false, &tokenizer),
                    false,
                    &tokenizer,
                )?;
                assert_eq!(exact["generated"], wrong["generated"]);
                assert_eq!(exact["active_exact"], true);
                assert_eq!(wrong["original_exact"], true);
                assert_eq!(wrong["active_exact"], false);
                assert_eq!(wrong["active_first_wrong"], 0);
                let stale = authored(source, 0, actual, false, &tokenizer);
                assert!(inspect(0, &changed, reading, &active, &stale, false, &tokenizer).is_err());
                let mut wrong_public_text = authored(source, 0, actual, true, &tokenizer);
                wrong_public_text["text"] = json!("altered public text");
                assert!(inspect(
                    0,
                    source,
                    reading,
                    &original,
                    &wrong_public_text,
                    true,
                    &tokenizer
                )
                .is_err());
                let no_eos = &actual[..actual.len() - 1];
                let missing = inspect(
                    0,
                    source,
                    reading,
                    &original,
                    &authored(source, 0, no_eos, false, &tokenizer),
                    false,
                    &tokenizer,
                )?;
                assert_eq!(missing["active_exact"], false);
                assert_eq!(missing["active_first_wrong"], no_eos.len());
                assert_eq!(missing["active_expected_wrong_id"], 1);
                assert!(missing["active_actual_wrong_id"].is_null());
            }
        }
        Ok(())
    }

    fn authored(
        label: &Value,
        index: usize,
        actual: &[u32],
        public: bool,
        tokenizer: &ByteBpeTokenizer,
    ) -> Value {
        let want: Option<Vec<u32>> = serde_json::from_value(label["expected"].clone()).unwrap();
        let first = want
            .as_ref()
            .and_then(|w| (0..w.len().max(actual.len())).find(|&i| w.get(i) != actual.get(i)));
        let text = public.then(|| tokenizer.decode(actual.strip_suffix(&[1]).unwrap_or(actual)));
        json!({
            "index":index,"id":label["id"],"family":label["family"],"pair":label["pair"],"direction":label["direction"],
            "stratum":label["stratum"],"counterpart":null,"requested":label["requested"],"gold_requested":label["requested"],
            "endpoint_exact":true,"source":null,"source_matches_requested_value":false,
            "expected":want,"generated":actual,"text":text,"acceptable_prose":label["prose_acceptable"],
            "terminal_eos":actual.last()==Some(&1),"exact":want.as_ref().map(|_|first.is_none()),"first_wrong":first,
            "expected_wrong_id":first.and_then(|k|want.as_ref()?.get(k)),"actual_wrong_id":first.and_then(|k|actual.get(k)),
        })
    }
}
