use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];

const SELECTED_SHA256: &str = "c15316b545e774872fcfe47e9cc556a58515ca59c55db14573f5a03da170e161";
const TOKENIZER_SHA256: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const MODEL_SHA256: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";

#[derive(Deserialize)]
struct Selected {
    family: String,
    prompt_id: String,
    span: Span,
    person: String,
    text: String,
    answer: String,
}

#[derive(Deserialize)]
struct Grade {
    rows: Vec<GradedPrompt>,
}

#[derive(Deserialize)]
struct GradedPrompt {
    prompt_id: String,
    family: String,
    mentions: Vec<Mention>,
}

#[derive(Deserialize)]
struct Mention {
    person: String,
    span: Span,
    assertion_span: Option<Span>,
    truth: u8,
    identity: i8,
    positional: i8,
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn read_pinned(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    if format!("{:x}", Sha256::digest(&bytes)) != expected {
        return Err(format!("input digest mismatch for {}", path.display()).into());
    }
    Ok(bytes)
}

fn prepare(tokenizer: &ByteBpeTokenizer, selected: &[Selected], input: &Path) -> Result<Value> {
    let grade: Grade = read_json(&input.join("seed1-grade.json"))?;
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let eos = tokenizer.encode("<|eos|>");
    if eos != [1] {
        return Err("unexpected EOS encoding".into());
    }
    let mut cases = Vec::new();
    for case in selected {
        let rows: Vec<_> = grade
            .rows
            .iter()
            .filter(|row| row.prompt_id == case.prompt_id)
            .collect();
        let [row] = rows.as_slice() else {
            return Err(format!("missing or duplicate frozen case {}", case.prompt_id).into());
        };
        if row.family != case.family || row.mentions.len() != 2 {
            return Err("wrong family or mention count for frozen case".into());
        }
        let gold: Vec<_> = row
            .mentions
            .iter()
            .filter(|mention| mention.truth == 1)
            .collect();
        let [requested] = gold.as_slice() else {
            return Err("frozen case does not have a single requested person".into());
        };
        if requested.person != case.person || requested.span != case.span {
            return Err("sealed requested decision differs from frozen case".into());
        }
        let [head_start, head_end] = requested
            .assertion_span
            .ok_or("gold person has no assertion")?;
        let prefix = encoder
            .encode_assistant_prefix(&[Message {
                role: "user",
                content: &case.text,
            }])
            .tokens;
        if tokenizer.decode(&prefix) != format!("<|bos|>User: {}\nAssistant:", case.text) {
            return Err("independent framed tokenization did not round-trip".into());
        }
        let client: Value = read_json(&input.join(format!("{}.json", case.family)))?;
        if client["protocol"] != "uor-r4.literal-role-dialogue/2"
            || client["read_enabled"] != true
            || client["tokenizer_sha256"] != TOKENIZER_SHA256
            || client["artifact_sha256"] != MODEL_SHA256
            || client["prompt_ids"] != json!(prefix)
        {
            return Err("ordinary baseline binding differs from frozen inputs".into());
        }
        let name = tokenizer.encode(&format!(" {}", case.person));
        let link = tokenizer.encode(" owns");
        let literal = tokenizer.encode(&format!(" {}", case.answer));
        let full_fact = tokenizer.encode(&format!(" {} owns {}.", case.person, case.answer));
        let periods = tokenizer.encode(".");
        let [period] = periods.as_slice() else {
            return Err("expected a single-token statement stop".into());
        };
        let onset = head_end + link.len();
        let fact_end = onset + literal.len();
        let [query_start, query_end] = requested.span;
        if head_start == 0
            || fact_end >= query_start
            || literal.is_empty()
            || prefix.get(head_start - 1) != Some(period)
            || prefix.get(head_start..head_end) != Some(name.as_slice())
            || prefix.get(query_start..query_end) != Some(name.as_slice())
            || prefix.get(head_start..=fact_end) != Some(full_fact.as_slice())
            || prefix.get(head_end..onset) != Some(link.as_slice())
            || prefix.get(onset..fact_end) != Some(literal.as_slice())
            || prefix.get(fact_end) != Some(period)
        {
            return Err(
                "could not independently bind complete source text at the assertion head".into(),
            );
        }
        let gold_ids: Vec<_> = literal.into_iter().chain(eos.iter().copied()).collect();
        let positives = |project: fn(&Mention) -> i8| {
            row.mentions
                .iter()
                .filter(|mention| project(mention) == 1)
                .map(|mention| mention.span)
                .collect::<Vec<_>>()
        };
        cases.push(json!({
            "prompt_id": case.prompt_id, "prefix": prefix, "ordinary_client": client,
            "identity": positives(|mention| mention.identity),
            "positional": positives(|mention| mention.positional),
            "gold_person": requested.span, "gold_ids": gold_ids, "gold_onset": onset,
        }));
    }
    Ok(json!(cases))
}

fn summarize(tokenizer: &ByteBpeTokenizer, selected: &[Selected], trace: &Path) -> Result<Value> {
    let raw: Vec<Value> = read_json(trace)?;
    if raw.len() != selected.len() {
        return Err("trace count differs from frozen prompt count".into());
    }
    let mut rows = Vec::new();
    let mut counts: BTreeMap<&str, [usize; 2]> = BTreeMap::new();
    for (case, trace) in selected.iter().zip(raw) {
        if trace["prompt_id"] != case.prompt_id || trace["gold_person"] != json!(case.span) {
            return Err("trace identity differs from frozen case".into());
        }
        let gold: Vec<u32> = serde_json::from_value(trace["gold_ids"].clone())?;
        let mut expected = tokenizer.encode(&format!(" {}", case.answer));
        expected.push(1);
        if gold != expected {
            return Err("writer gold IDs differ from independent encoding".into());
        }
        let mut arms = BTreeMap::new();
        for name in [
            "reference",
            "identity",
            "positional",
            "gold_person",
            "gold_onset",
            "disabled",
        ] {
            let arm = &trace["arms"][name];
            let emitted: Vec<u32> = serde_json::from_value(arm["ids"].clone())?;
            let eos = emitted.last() == Some(&1);
            let exact = emitted == gold;
            if arm["eos"] != eos || (name == "gold_onset" && !exact) {
                return Err("EOS or forced onset validation failed".into());
            }
            let first_error = (0..emitted.len().max(gold.len()))
                .find(|&i| emitted.get(i) != gold.get(i))
                .map_or(
                    Value::Null,
                    |i| json!({"offset":i,"expected":gold.get(i),"emitted":emitted.get(i)}),
                );
            if arm.get("first_error") != Some(&first_error) {
                return Err("first-error validation failed".into());
            }
            let text_ids = if eos {
                &emitted[..emitted.len() - 1]
            } else {
                &emitted
            };
            let count = counts.entry(name).or_default();
            count[0] += usize::from(exact);
            count[1] += usize::from(eos);
            arms.insert(
                name,
                json!({
                    "source": arm["source"], "ids": emitted, "text": tokenizer.decode(text_ids),
                    "requested_spans": arm["requested_spans"], "exact": exact,
                    "eos": eos, "first_error": first_error,
                }),
            );
        }
        rows.push(json!({
            "family": case.family, "prompt_id": case.prompt_id,
            "gold_person": trace["gold_person"], "gold_onset": trace["gold_onset"],
            "gold_ids": gold, "arms": arms,
        }));
    }
    let counts: BTreeMap<_, _> = counts
        .into_iter()
        .map(|(name, [exact, eos])| (name, json!({"exact":exact,"eos":eos})))
        .collect();
    Ok(
        json!({"model_sha256": MODEL_SHA256, "tokenizer_sha256": TOKENIZER_SHA256,
        "selected_sha256": SELECTED_SHA256, "rows": rows, "counts": counts}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [mode, tokenizer, input, trailing @ ..] = args.as_slice() else {
        return Err("usage: bind-clause-writer-cases prepare TOKENIZER WRITER_INPUT | summarize TOKENIZER WRITER_INPUT TRACE".into());
    };
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&read_pinned(
        Path::new(tokenizer),
        TOKENIZER_SHA256,
    )?)
    .ok_or("invalid pinned tokenizer")?;
    let input = Path::new(input);
    let selected: Vec<Selected> =
        serde_json::from_slice(&read_pinned(&input.join("selected.json"), SELECTED_SHA256)?)?;
    let value = match (mode.as_str(), trailing) {
        ("prepare", []) => prepare(&tokenizer, &selected, input)?,
        ("summarize", [trace]) => summarize(&tokenizer, &selected, Path::new(trace))?,
        _ => return Err("invalid writer adapter invocation".into()),
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
