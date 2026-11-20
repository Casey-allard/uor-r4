#![forbid(unsafe_code)]

mod common;

use common::{sha, write_json, write_manifest, Input, Result, Span, TOKENIZER};
use serde::Serialize;
use std::{collections::BTreeSet, env, fs, io::Write, path::Path};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

const HEAD: &str = "<|bos|>User: ";
const MAX_IDS: usize = 192;
const EOS: u32 = 1;

#[derive(Debug)]
struct Row {
    row: u32,
    group: &'static str,
    stratum: &'static str,
    split: &'static str,
    source: String,
    query_region: Span,
    query: Span,
    fact: Option<Span>,
    value: Option<Span>,
    excluded: Option<Span>,
    factors: Option<[usize; 4]>,
}

fn append(out: &mut String, segment: &str) -> Span {
    let start = out.len();
    out.push(' ');
    out.push_str(segment);
    [start, out.len()]
}

fn fact(out: &mut String, name: &str, verb: &str, modifier: &str, value: &str) -> (Span, Span) {
    let name = append(out, name);
    append(out, verb);
    if !modifier.is_empty() {
        append(out, modifier);
    }
    let value = append(out, value);
    out.push('.');
    (name, value)
}

fn query(out: &mut String, template: &str, name: &str) -> Span {
    append(out, template);
    let mention = append(out, name);
    out.push('.');
    mention
}

fn rows(short: bool) -> Result<Vec<Row>> {
    let mut result = Vec::new();
    let mut train_id = 35_000;
    let mut held_id = if short { 46_000 } else { 36_000 };
    let mut train_entities = BTreeSet::new();
    let mut held_entities = BTreeSet::new();
    let mut train_values = BTreeSet::new();
    let mut held_values = BTreeSet::new();
    let families = include_str!("families.tsv").lines();
    let family_rows: Vec<_> = if short {
        families
            .filter(|line| line.starts_with("train\t"))
            .chain(include_str!("held_short.tsv").lines())
            .collect()
    } else {
        families.collect()
    };
    for line in family_rows {
        let fields: Vec<_> = line.split('\t').collect();
        let [split, group, n1, n2, verb, v1, v2, introduction, request, exclude] =
            fields.as_slice()
        else {
            return Err("wrong family columns".into());
        };
        let (id, names, values) = match *split {
            "train" => (&mut train_id, &mut train_entities, &mut train_values),
            "held" => (&mut held_id, &mut held_entities, &mut held_values),
            _ => return Err("wrong family split".into()),
        };
        for name in [n1, n2] {
            if !names.insert(*name) {
                return Err(format!("entity {name} reused").into());
            }
        }
        for value in [v1, v2] {
            if !values.insert(*value) {
                return Err(format!("value {value} reused").into());
            }
        }
        let names = [n1, n2];
        let values = [v1, v2];
        for (connector, modifier) in ["", "only", "also"].into_iter().enumerate() {
            for who in 0..2 {
                for fact_order in 0..2 {
                    for query_order in 0..2 {
                        let mut source = introduction.to_string();
                        let first = fact_order;
                        let second = 1 - first;
                        let a = fact(&mut source, names[first], verb, modifier, values[first]);
                        let b = fact(&mut source, names[second], verb, modifier, values[second]);
                        let (head, value) = if who == first { a } else { b };
                        let query_start = source.len();
                        let (target, excluded) = if query_order == 0 {
                            let target = query(&mut source, request, names[who]);
                            let excluded = query(&mut source, exclude, names[1 - who]);
                            (target, excluded)
                        } else {
                            let excluded = query(&mut source, exclude, names[1 - who]);
                            let target = query(&mut source, request, names[who]);
                            (target, excluded)
                        };
                        let query_region = [query_start, source.len()];
                        result.push(Row {
                            row: *id,
                            group,
                            stratum: if modifier.is_empty() {
                                "bare"
                            } else {
                                "past_connector"
                            },
                            split,
                            source,
                            query_region,
                            query: target,
                            fact: Some(head),
                            value: Some(value),
                            excluded: Some(excluded),
                            factors: Some([who, fact_order, query_order, connector]),
                        });
                        *id += 1;
                    }
                }
            }
        }
        if *split == "held" {
            let introduction = introduction.to_string();
            let mut source = introduction.clone();
            fact(&mut source, n2, verb, "", v2);
            let query_start = source.len();
            let target = query(&mut source, request, n1);
            let excluded = query(&mut source, exclude, n2);
            let query_region = [query_start, source.len()];
            result.push(Row {
                row: *id,
                group,
                stratum: "missing",
                split,
                source,
                query_region,
                query: target,
                fact: None,
                value: None,
                excluded: Some(excluded),
                factors: None,
            });
            *id += 1;
            let mut source = introduction;
            fact(&mut source, n1, verb, "", v1);
            fact(&mut source, n1, verb, "", v2);
            let query_start = source.len();
            let target = query(&mut source, request, n1);
            let query_region = [query_start, source.len()];
            result.push(Row {
                row: *id,
                group,
                stratum: "conflicting",
                split,
                source,
                query_region,
                query: target,
                fact: None,
                value: None,
                excluded: None,
                factors: None,
            });
            *id += 1;
        }
    }
    for line in include_str!("causal.tsv").lines().filter(|_| !short) {
        let fields: Vec<_> = line.split('\t').collect();
        let [group, name, verb, particle, noun, context_a, context_b, context_missing, request] =
            fields.as_slice()
        else {
            return Err("wrong causal columns".into());
        };
        if !held_entities.insert(*name) || !held_values.insert(*noun) {
            return Err(format!("causal group {group} reuses held material").into());
        }
        for (i, context) in [context_a, context_b, context_missing].iter().enumerate() {
            let mut source = context.to_string();
            let (head, short_value) = fact(&mut source, name, verb, particle, noun);
            let full_value = [short_value[0] - particle.len() - 1, short_value[1]];
            let query_start = source.len();
            let target = query(&mut source, request, name);
            let query_region = [query_start, source.len()];
            let (stratum, value) = match i {
                0 => ("causal_early", Some(full_value)),
                1 => ("causal_late", Some(short_value)),
                _ => ("ambiguous", None),
            };
            result.push(Row {
                row: held_id,
                group,
                stratum,
                split: "held",
                source,
                query_region,
                query: target,
                fact: Some(head),
                value,
                excluded: None,
                factors: None,
            });
            held_id += 1;
        }
    }
    if !train_entities.is_disjoint(&held_entities) || !train_values.is_disjoint(&held_values) {
        return Err("train/held split material leaks".into());
    }
    let prompts: BTreeSet<_> = result.iter().map(|r| &r.source).collect();
    if prompts.len() != result.len() {
        return Err("duplicate panel text".into());
    }
    Ok(result)
}

#[derive(Serialize)]
struct Key<'a> {
    row: u32,
    group: &'a str,
    split: &'a str,
    stratum: &'a str,
    source: &'a str,
    query_region: Span,
    query: Span,
    fact: Option<Span>,
    excluded: Option<Span>,
    gold_source: Option<Span>,
    gold_bytes: Option<Span>,
    query_region_bytes: Span,
    query_bytes: Span,
    fact_bytes: Option<Span>,
    excluded_bytes: Option<Span>,
    original_ids: Vec<u32>,
    expected_tape: Option<Vec<u32>>,
    expected_text: Option<&'a str>,
    factors: Option<[usize; 4]>,
}

fn jsonl(file: &mut fs::File, value: &Input) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn freeze(tokenizer_path: &Path, output: &Path, short: bool) -> Result<()> {
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    if sha(&tokenizer_bytes) != TOKENIZER {
        return Err("D26-bound tokenizer SHA-256 mismatch".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let rows = rows(short)?;
    fs::create_dir(output)?;
    let fit = output.join("train");
    let blind = output.join("held");
    let offline = output.join("offline");
    for dir in [&fit, &blind, &offline] {
        fs::create_dir(dir)?;
    }
    let mut train = fs::File::create_new(fit.join("supplied.jsonl"))?;
    let mut labels = Vec::new();
    let mut ordinary = fs::File::create_new(blind.join("ordinary.jsonl"))?;
    let mut supplied = fs::File::create_new(blind.join("supplied.jsonl"))?;
    let mut keys = Vec::new();
    for row in &rows {
        let user = Message {
            role: "user",
            content: &row.source,
        };
        let ids = encoder.encode_assistant_prefix(&[user]).tokens;
        if ids.len() > MAX_IDS
            || tokenizer.decode(&ids) != format!("{HEAD}{}\nAssistant:", row.source)
        {
            return Err(format!("row {} failed original protocol-2 round trip", row.row).into());
        }
        let mut edges = vec![0];
        for &id in &ids {
            edges.push(
                edges.last().copied().ok_or("missing initial edge")?
                    + tokenizer.decode_bytes(&[id]).len(),
            );
        }
        let span = |[a, b]: Span| -> Result<Span> {
            let token = |raw_byte| {
                let framed = HEAD.len() + raw_byte;
                edges.binary_search(&framed).map_err(|_| {
                    format!(
                        "row {} source byte {raw_byte} splits an original token in {}",
                        row.row, row.source
                    )
                })
            };
            Ok([token(a)?, token(b)?])
        };
        let query = span(row.query)?;
        let query_region = span(row.query_region)?;
        let fact = row.fact.map(span).transpose()?;
        let excluded = row.excluded.map(span).transpose()?;
        let value = row.value.map(span).transpose()?;
        if let Some([a, b]) = fact {
            if ids[a..b] != ids[query[0]..query[1]] {
                return Err(format!("row {} has unequal query/fact IDs", row.row).into());
            }
        }
        let expected_text = row.value.map(|[a, b]| row.source[a..b].trim_start());
        let expected_tape = match (value, expected_text) {
            (Some([a, b]), Some(text)) => {
                let document = encoder
                    .encode_document(&[
                        user,
                        Message {
                            role: "assistant",
                            content: text,
                        },
                    ])
                    .tokens;
                let tape = document
                    .strip_prefix(ids.as_slice())
                    .ok_or("assistant changed prefix")?;
                if ids.get(b) != Some(&16) || tape != [&ids[a..b], &[EOS]].concat() {
                    return Err(format!("row {} lacks an exact copy-to-EOS path", row.row).into());
                }
                if short && row.split == "held" && tape.len() > 8 {
                    return Err(format!("row {} exceeds the 7-token writer limit", row.row).into());
                }
                Some(tape.to_vec())
            }
            (None, None) => None,
            _ => return Err("value byte/token mismatch".into()),
        };
        let blind_row = Input {
            row: row.row,
            source: row.source.clone(),
            oracle: None,
        };
        let supplied_row = Input {
            oracle: fact.map(|fact| [query[0], query[1], fact[0], fact[1]]),
            ..blind_row.clone()
        };
        if row.split == "train" {
            jsonl(&mut train, &supplied_row)?;
            labels.push((row.row, value.ok_or("TRAIN row lacks answer")?));
        } else {
            jsonl(&mut ordinary, &blind_row)?;
            jsonl(&mut supplied, &supplied_row)?;
            keys.push(Key {
                row: row.row,
                group: row.group,
                split: row.split,
                stratum: row.stratum,
                source: &row.source,
                query_region,
                query,
                fact,
                excluded,
                gold_source: value,
                gold_bytes: row.value,
                query_region_bytes: row.query_region,
                query_bytes: row.query,
                fact_bytes: row.fact,
                excluded_bytes: row.excluded,
                original_ids: ids,
                expected_tape,
                expected_text,
                factors: row.factors,
            });
        }
    }
    write_json(&fit.join("labels.json"), &labels)?;
    write_json(&offline.join("key.json"), &keys)?;
    write_manifest(&fit, "manifest.sha256", &["labels.json", "supplied.jsonl"])?;
    write_manifest(
        &blind,
        "manifest.sha256",
        &["ordinary.jsonl", "supplied.jsonl"],
    )?;
    write_manifest(&offline, "manifest.sha256", &["key.json"])?;
    write_manifest(
        output,
        "manifest.sha256",
        &[
            "train/manifest.sha256",
            "held/manifest.sha256",
            "offline/manifest.sha256",
        ],
    )?;
    println!(
        "FROZEN {} TRAIN, {} held, original protocol-2 tapes checked",
        labels.len(),
        keys.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [tokenizer, output] => freeze(Path::new(tokenizer), Path::new(output), false),
        [option, tokenizer, output] if option == "--short" => {
            freeze(Path::new(tokenizer), Path::new(output), true)
        }
        _ => Err("usage: d35-panel-freeze [--short] TOKENIZER NEW_DIR".into()),
    }
}
