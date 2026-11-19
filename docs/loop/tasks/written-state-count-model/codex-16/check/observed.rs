use super::{ids, read, save, valid, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs};
use uor_r4_tokenizer::ByteBpeTokenizer;

const ROWS: usize = 32;
const POSITIVES: usize = 24;
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
            && ids(field(fact, "literal_ids_through_eos")?)? == self.ids)
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
        _ => return Err("unknown original stratum".into()),
    };
    let facts = array(field(original, "facts")?, "facts")?;
    let requested = string(original, "requested_name")?;
    let supporting = facts
        .iter()
        .enumerate()
        .filter_map(|(j, fact)| (fact["name"] == requested).then_some(j))
        .collect::<Vec<_>>();
    if field(reading, "index")? != index
        || field(reading, "status")? != status
        || field(reading, "requested_name")? != requested
        || field(reading, "support_count")? != support_count
        || supporting.len() != support_count
        || field(original, "supporting_facts")? != &json!(supporting)
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
            .filter(|fact| fact["name"] != requested)
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
    if source.is_some_and(|start| start >= CONTEXT)
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
        return Err("observed TOKENIZER GRADES INDEPENDENT_READINGS ORIGINAL_GRADES_OR_MINUS OBSERVED_WRITER_GRADE_JSON OUT".into());
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
            != 4
        || labels
            .iter()
            .filter(|label| label.status == "conflict")
            .count()
            != 4
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
            "schema":"uor-r4.fresh-validity-observed-grader/1",
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

    fn tokenizer() -> ByteBpeTokenizer {
        // Surfaces and sparse IDs for the published seed-1 emission fixture.
        // Added tokens keep this finite test independent of asset downloads.
        let pieces = [
            (0, "<|bos|>"),
            (1, "<|eos|>"),
            (2, "<|unk|>"),
            (14, ","),
            (16, "."),
            (302, "ot"),
            (311, " it"),
            (340, " I"),
            (383, " that"),
            (1155, " G"),
            (1203, " remember"),
            (1919, "'ll"),
            (2167, " whistle"),
            (2211, "let"),
            (2346, " scar"),
            (3300, " violet pencil"),
            (3301, " No"),
            (3302, " Yes"),
        ];
        let added: Vec<_> = pieces
            .iter()
            .map(|&(id, content)| json!({"id":id,"content":content}))
            .collect();
        let spec = json!({
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2},"merges":[]},
            "added_tokens":added
        });
        ByteBpeTokenizer::from_tokenizer_json_bytes(&serde_json::to_vec(&spec).unwrap()).unwrap()
    }

    fn example() -> Value {
        let archive: Value =
            serde_json::from_str(include_str!("../../codex-15/receipts/seed1.json")).unwrap();
        assert_eq!(archive["examples"][0]["id"], "held-0-04-w0-d0");
        archive["examples"][0].clone()
    }

    fn writer_row(actual: &[u32], expected: Option<&[u32]>, text: Value) -> Value {
        let wrong = expected.and_then(|w| first_wrong(actual, w));
        json!({
            "generated": actual, "expected": expected, "terminal_eos": actual.last() == Some(&1),
            "exact": expected.map(|_| wrong.is_none()), "first_wrong": wrong,
            "expected_wrong_id": wrong.and_then(|i| expected?.get(i)),
            "actual_wrong_id": wrong.and_then(|i| actual.get(i)),
            "source": null, "requested": null, "gold_requested": null, "endpoint_exact": true,
            "text": text, "acceptable_prose": null
        })
    }

    fn labels(expected: &[u32], active: &[u32]) -> (Value, Value, Value) {
        let source = json!({
            "id": "example", "expected": expected, "answer_text": "scarlet whistle",
            "stratum": "supported", "requested_name": "mary", "requested": null,
            "prose_acceptable": null, "gold_assertion_index": 0, "supporting_facts": [0],
            "facts": [
                {"name":"mary","value":"scarlet whistle","literal_ids_through_eos":expected},
                {"name":"john","value":"violet pencil","literal_ids_through_eos":active}
            ]
        });
        let mut mutated = source.clone();
        mutated["expected"] = json!(active);
        mutated["answer_text"] = json!("violet pencil");
        let read = json!({"index":0,"status":"supported","requested_name":"mary",
            "support_count":1,"answer":"scarlet whistle"});
        (source, mutated, read)
    }

    #[test]
    fn genuine_emission_does_not_follow_altered_label_or_eos() {
        let tokenizer = tokenizer();
        let example = example();
        let actual = ids(&example["forced_literal"]["generated"]).unwrap();
        let alternative = [3300, 1];
        let (source, mutated, reading) = labels(&actual, &alternative);
        let truth = check_labels(&source, &source, &reading, 0, false, &tokenizer).unwrap();
        let old_grade = writer_row(&actual, Some(&actual), Value::Null);
        let ordinary =
            inspect(0, &source, &reading, &truth, &old_grade, false, &tokenizer).unwrap();
        assert_eq!(ordinary["original_exact"], true);
        assert_eq!(ordinary["active_exact"], true);
        assert_eq!(ordinary["decoded_text"], " scarlet whistle");

        let changed = check_labels(&source, &mutated, &reading, 0, true, &tokenizer).unwrap();
        let new_grade = writer_row(&actual, Some(&alternative), Value::Null);
        let rescored = inspect(
            0, &mutated, &reading, &changed, &new_grade, false, &tokenizer,
        )
        .unwrap();
        assert_eq!(rescored["generated"], ordinary["generated"]);
        assert_eq!(rescored["original_exact"], true);
        assert_eq!(rescored["active_exact"], false);
        assert_eq!(rescored["active_first_wrong"], 0);
        assert!(inspect(0, &mutated, &reading, &changed, &old_grade, false, &tokenizer).is_err());

        let missing_eos = &actual[..actual.len() - 1];
        let bad_end = writer_row(missing_eos, Some(&actual), Value::Null);
        let stopped = inspect(0, &source, &reading, &truth, &bad_end, false, &tokenizer).unwrap();
        assert_eq!(stopped["original_exact"], false);
        assert_eq!(stopped["original_first_wrong"], missing_eos.len());
        assert_eq!(stopped["active_expected_wrong_id"], 1);
        for replacement in [0, 2] {
            let mut malformed = actual.clone();
            malformed[0] = replacement;
            let row = writer_row(&malformed, Some(&actual), Value::Null);
            let report = inspect(0, &source, &reading, &truth, &row, false, &tokenizer).unwrap();
            assert_eq!(report["original_exact"], false);
            assert_eq!(report["body_has_special"], true);
        }
        let row = writer_row(&[4000, 1], Some(&actual), Value::Null);
        assert!(inspect(0, &source, &reading, &truth, &row, false, &tokenizer).is_err());
        let mut interior_eos = actual.clone();
        interior_eos.insert(1, 1);
        let row = writer_row(&interior_eos, Some(&actual), Value::Null);
        assert!(inspect(0, &source, &reading, &truth, &row, false, &tokenizer).is_err());
    }

    #[test]
    fn genuine_public_fallback_and_negative_decisions() {
        let tokenizer = tokenizer();
        let example = example();
        let actual = ids(&example["client"]["generated"]).unwrap();
        let label = json!({"id":"negative","requested":null,"facts":[],"prose_acceptable":null});
        let reading = json!({"why":"The question names a person without a recorded item."});
        let labels = Labels {
            status: "absent",
            original: None,
            active: None,
        };
        let mut row = writer_row(&actual, None, example["client"]["text"].clone());
        let fallback = inspect(0, &label, &reading, &labels, &row, true, &tokenizer).unwrap();
        assert_eq!(fallback["negative"]["classification"], "fallback");
        assert_eq!(fallback["decoded_text"], " Got it, I'll remember that.");
        row["source"] = json!(4);
        let copy = inspect(0, &label, &reading, &labels, &row, true, &tokenizer).unwrap();
        assert_eq!(copy["negative"]["classification"], "copy");
        let row = writer_row(&[3301, 16, 1], None, json!(" No."));
        let no = inspect(0, &label, &reading, &labels, &row, true, &tokenizer).unwrap();
        assert_eq!(no["negative"]["classification"], "no");
        assert_eq!(no["negative"]["normalized_yes_no"], "no");
        let row = writer_row(&[3302, 1], None, json!(" Yes"));
        let yes = inspect(0, &label, &reading, &labels, &row, true, &tokenizer).unwrap();
        assert_eq!(yes["negative"]["classification"], "fallback");
        assert_eq!(yes["negative"]["normalized_yes_no"], "yes");
    }
}
