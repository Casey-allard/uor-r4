#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_integer::discrete_d11::{ExactCopyState, ROLE_COPY_STOPS};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];
const HEAD: &str = "<|bos|>User: ";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

fn append(text: &mut String, word: &str) -> Span {
    let start = HEAD.len() + text.len();
    text.push(' ');
    text.push_str(word);
    [start, HEAD.len() + text.len()]
}

struct RoleRow {
    row: u32,
    family: &'static str,
    stratum: &'static str,
    text: String,
    query: Span,
    endpoint_bytes: [Option<Span>; 3],
    head_bytes: Vec<Span>,
    value_bytes: Vec<Span>,
    response: &'static str,
    selected: Option<usize>,
    factors: Option<[usize; 3]>,
}

fn role_rows() -> Result<Vec<RoleRow>> {
    let mut rows = Vec::new();
    for (family_id, line) in include_str!("role.tsv").lines().enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [family, first, second, third, verb, a, b, c, request, exclude] = fields.as_slice()
        else {
            return Err("malformed role-family fixture".into());
        };
        let (people, values) = ([*first, *second, *third], [*a, *b, *c]);
        for variant in 0..11 {
            let direct = variant == 8;
            let unspecified = variant == 9;
            let incidental_only = variant == 10;
            let who = if variant < 8 { variant / 4 } else { 0 };
            let fact_order = variant / 2 % 2;
            let query_order = variant % 2;
            let mut text = "The ledger lists each person's item on a separate line.".to_owned();
            let order = if direct {
                vec![who]
            } else {
                vec![fact_order, 1 - fact_order, 2]
            };
            let mut head_bytes = Vec::new();
            let mut value_bytes = Vec::new();
            for &index in &order {
                head_bytes.push(append(&mut text, people[index]));
                append(&mut text, verb);
                value_bytes.push(append(&mut text, values[index]));
                text.push('.');
            }
            let query_start = HEAD.len() + text.len();
            let mut endpoint_bytes = [None; 3];
            let selected = if unspecified || incidental_only {
                if incidental_only {
                    append(&mut text, "The clerk telephoned");
                    endpoint_bytes[2] = Some(append(&mut text, people[2]));
                    append(&mut text, "to check the ledger's publication date");
                    text.push('.');
                }
                append(&mut text, "Which person's item does the ledger identify for this request? No person was specified. Please say I don't know");
                text.push('.');
                None
            } else {
                let polarities = if direct {
                    vec![0]
                } else {
                    vec![query_order, 1 - query_order]
                };
                for polarity in polarities {
                    append(&mut text, if polarity == 0 { request } else { exclude });
                    endpoint_bytes[polarity] = Some(append(&mut text, people[who ^ polarity]));
                    text.push(if polarity == 0 && family_id == 1 {
                        '?'
                    } else {
                        '.'
                    });
                }
                if !direct {
                    append(&mut text, "The clerk telephoned");
                    endpoint_bytes[2] = Some(append(&mut text, people[2]));
                    append(&mut text, "to check the ledger's publication date");
                    text.push('.');
                }
                Some(order.iter().position(|&i| i == who).ok_or("missing fact")?)
            };
            let stratum = if direct {
                "direct"
            } else if unspecified {
                "unspecified"
            } else if incidental_only {
                "incidental_only"
            } else {
                "crossed"
            };
            rows.push(RoleRow {
                row: 30100 + (16 * family_id + variant) as u32,
                family,
                stratum,
                query: [query_start, HEAD.len() + text.len()],
                text,
                endpoint_bytes,
                head_bytes,
                value_bytes,
                response: selected.map_or("I don't know.", |_| values[who]),
                selected,
                factors: (variant < 8).then_some([who, fact_order, query_order]),
            });
        }
    }
    Ok(rows)
}

fn output_role(tokenizer: &ByteBpeTokenizer, output: &Path) -> Result<()> {
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let mut public = BufWriter::new(fs::File::create_new(output.join("role.blind.tsv"))?);
    let mut gold = Vec::new();
    for row in role_rows()? {
        let user = Message {
            role: "user",
            content: &row.text,
        };
        let ids = encoder.encode_assistant_prefix(&[user]).tokens;
        if tokenizer.decode(&ids) != format!("{HEAD}{}\nAssistant:", row.text) {
            return Err(format!("row {} failed protocol-2 round trip", row.row).into());
        }
        let mut edges = vec![0];
        let mut offset = 0;
        for &id in &ids {
            offset += tokenizer.decode_bytes(&[id]).len();
            edges.push(offset);
        }
        let span = |[a, b]: Span| -> Result<Span> {
            let token = |byte| {
                edges
                    .binary_search(&byte)
                    .map_err(|_| format!("row {} byte {byte} splits a token", row.row))
            };
            Ok([token(a)?, token(b)?])
        };
        let expected = encoder
            .encode_document(&[
                user,
                Message {
                    role: "assistant",
                    content: row.response,
                },
            ])
            .tokens;
        let expected = expected
            .strip_prefix(ids.as_slice())
            .ok_or("assistant changed prefix")?;
        if expected.last() != Some(&1) {
            return Err("canonical expected answer is missing EOS".into());
        }
        let query = span(row.query)?;
        let [requested, excluded, unrequested] = row
            .endpoint_bytes
            .map(|endpoint| endpoint.map(span).transpose());
        let (requested, excluded, unrequested) = (requested?, excluded?, unrequested?);
        let heads = row
            .head_bytes
            .into_iter()
            .map(span)
            .collect::<Result<Vec<_>>>()?;
        let values = row
            .value_bytes
            .into_iter()
            .map(span)
            .collect::<Result<Vec<_>>>()?;
        let mut expected_source = None;
        if let Some(selected) = row.selected {
            let [head, head_end] = heads[selected];
            let [value, fence] = values[selected];
            let [q, qe] = requested.ok_or("selected row lacks requested span")?;
            if ids[head..head_end] != ids[q..qe]
                || head_end - head < 2
                || ids.get(fence) != Some(&16)
                || expected != [&ids[value..fence], &[1]].concat()
            {
                return Err(
                    format!("row {} does not preserve name or exact value tape", row.row).into(),
                );
            }
            let mut cursor = ExactCopyState::new(Some(value.try_into()?), ids.len().try_into()?);
            for &token in expected {
                let step = cursor
                    .write(&ids, 0, ROLE_COPY_STOPS)?
                    .finish_with_eos(true, 1);
                if step.emitted != token || token == 1 && !step.completed_span() {
                    return Err(format!("row {} fails exact writer control", row.row).into());
                }
                cursor = step.state;
            }
            expected_source = Some(value);
        }
        writeln!(public, "{}\t{}", row.row, row.text)?;
        gold.push(json!({
            "row":row.row, "stratum":row.stratum, "family":row.family,
            "role_factors": row.factors, "original_ids":ids, "query":query,
            "role_targets":{"requested":requested, "excluded":excluded, "unrequested":unrequested},
            "assertion_heads":heads, "value_spans":values,
            "expected_tape":expected, "expected_reply":row.response, "gold_source":expected_source,
        }));
    }
    public.flush()?;
    let mut index = fs::File::create_new(output.join("role.gold.json"))?;
    serde_json::to_writer_pretty(&mut index, &gold)?;
    writeln!(index)?;
    Ok(())
}

fn output_source(tokenizer: &ByteBpeTokenizer, candidates: &Path, output: &Path) -> Result<()> {
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let grades: Vec<Value> =
        serde_json::from_slice(&fs::read(candidates.join("held.grade.json"))?)?;
    let prompts: Vec<Value> =
        serde_json::from_slice(&fs::read(candidates.join("held.ordinary.json"))?)?;
    let grade = grades
        .into_iter()
        .map(|v| {
            Ok((
                u32::try_from(v["row"].as_u64().ok_or("invalid grade row")?)?,
                v,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut accepted = BTreeSet::new();
    let donors = fs::read_to_string(candidates.join("held.donors.tsv"))?;
    for line in donors.lines() {
        let (target, donor) = line.split_once('\t').ok_or("invalid donor row")?;
        let (target, donor): (u32, u32) = (target.parse()?, donor.parse()?);
        let left = grade.get(&target).ok_or("target missing grade")?;
        let right = grade.get(&donor).ok_or("donor missing grade")?;
        let offset = |v: &Value, path: &str| -> Result<u64> {
            Ok(v[path].as_u64().ok_or("missing original offset")?)
        };
        let origin = |v: &Value| -> Result<usize> {
            v["fact_and_query"][0][0]
                .as_u64()
                .ok_or("missing fact origin")?
                .try_into()
                .map_err(Into::into)
        };
        let tape = |v: &Value| -> Result<Vec<u32>> {
            Ok(serde_json::from_value(v["original_ids"].clone())?)
        };
        let (li, ri) = (origin(left)?, origin(right)?);
        let (lt, rt) = (tape(left)?, tape(right)?);
        if lt[li..] != rt[ri..]
            || offset(left, "gold")? - li as u64 == offset(right, "gold")? - ri as u64
        {
            return Err(format!(
                "pair {target}/{donor} did not preserve its full local tape and switch the gold"
            )
            .into());
        }
        let left_gold = offset(left, "gold")?.to_string();
        let right_gold = offset(right, "gold")?.to_string();
        if left["plausible_writer_ids"][left_gold] == right["plausible_writer_ids"][right_gold] {
            return Err(
                format!("pair {target}/{donor} failed to produce distinct gold tapes").into(),
            );
        }
        accepted.insert(target);
    }
    let mut public = BufWriter::new(fs::File::create_new(output.join("held.blind.tsv"))?);
    let mut written = BTreeSet::new();
    let mut admissions = Vec::new();
    for prompt in prompts {
        let row = u32::try_from(prompt["row"].as_u64().ok_or("invalid ordinary row")?)?;
        if !grade.contains_key(&row) || !written.insert(row) {
            return Err("ordinary rows and grade rows differ".into());
        }
        let text = prompt["messages"][0]["content"]
            .as_str()
            .ok_or("missing ordinary prompt")?;
        let row_grade = &grade[&row];
        let original: Vec<u32> = serde_json::from_value(row_grade["original_ids"].clone())?;
        let user = Message {
            role: "user",
            content: text,
        };
        if encoder.encode_assistant_prefix(&[user]).tokens != original {
            return Err(format!("row {row} ordinary prompt changed the original tape").into());
        }
        let expected_reply = row_grade["expected_reply"]
            .as_str()
            .ok_or("missing expected reply")?;
        let mut controls = BTreeMap::new();
        let candidates: BTreeMap<u32, bool> =
            serde_json::from_value(row_grade["candidates"].clone())?;
        let forced: BTreeMap<u32, Vec<u32>> =
            serde_json::from_value(row_grade["plausible_writer_ids"].clone())?;
        for (start, through_eos) in forced {
            let content =
                tokenizer.decode(through_eos.strip_suffix(&[1]).ok_or("missing forced EOS")?);
            let document = encoder
                .encode_document(&[
                    user,
                    Message {
                        role: "assistant",
                        content: content.trim_start(),
                    },
                ])
                .tokens;
            if document.strip_prefix(original.as_slice()) != Some(through_eos.as_slice())
                || !candidates.contains_key(&start)
            {
                return Err(format!(
                    "row {row} writer suffix at {start} fails independent tokenization or reach"
                )
                .into());
            }
            if row_grade["gold"].as_u64() == Some(start.into())
                && content.trim_start() != expected_reply
            {
                return Err(
                    format!("row {row} selected wording differs from registered gold").into(),
                );
            }
            controls.insert(
                start,
                json!({"reply":content,"ids_through_eos":through_eos}),
            );
        }
        admissions.push(json!({
            "row":row, "fact_and_query":row_grade["fact_and_query"], "fence":row_grade["fence"],
            "gold_source":row_grade["gold"], "expected_reply":expected_reply,
            "candidates":candidates, "literal_controls":controls,
        }));
        writeln!(public, "{row}\t{text}")?;
    }
    if written.len() != grade.len() || accepted.len() < 12 {
        return Err("need all grade rows and at least four novel plus both exposed pairs".into());
    }
    public.flush()?;
    let mut receipt = fs::File::create_new(output.join("held.admission.json"))?;
    serde_json::to_writer_pretty(&mut receipt, &admissions)?;
    writeln!(receipt)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [tokenizer, candidates, output] = args.as_slice() else {
        return Err("usage: d30-panel TOKENIZER D26_PREPARED_PANEL NEW_OUTPUT".into());
    };
    let bytes = fs::read(tokenizer)?;
    if format!("{:x}", Sha256::digest(&bytes)) != TOKENIZER {
        return Err("tokenizer identity differs".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let output = Path::new(output);
    fs::create_dir(output)?;
    output_role(&tokenizer, output)?;
    output_source(&tokenizer, Path::new(candidates), output)
}
