//! Freeze factorial role selection data. No artifact is loaded and no model runs.
//! Arguments: public tokenizer.json, original panel.json, new output directory.
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::{env, fs, path::Path};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize)]
struct Labels<'a> {
    prefix: &'a [u32],
    query: usize,
    query_other: Option<usize>,
    fact: usize,
    other_fact: usize,
    source: usize,
    other_source: usize,
}

#[derive(Serialize)]
struct Query<'a> {
    prefix: &'a [u32],
}

struct Family {
    names: [&'static str; 2],
    values: [&'static str; 2],
}

// The encode/decode proof below checks that these spans correspond to whole
// occurrences, even where one name or value occupies several model IDs.
fn occurrences(prefix: &[u32], surface: &str, tokenizer: &ByteBpeTokenizer) -> Result<Vec<usize>> {
    let ids = tokenizer.encode(&format!(" {surface}"));
    if ids.is_empty() {
        return Err("empty tokenized surface".into());
    }
    Ok(prefix
        .windows(ids.len())
        .enumerate()
        .filter_map(|(i, w)| (w == ids).then_some(i))
        .collect())
}

fn write_rows<T: Serialize>(root: &Path, name: &str, rows: &[T]) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"[\n");
    for (index, row) in rows.iter().enumerate() {
        if index != 0 {
            out.extend_from_slice(b",\n");
        }
        serde_json::to_writer(&mut out, row)?;
    }
    out.extend_from_slice(b"\n]\n");
    fs::write(root.join(name), out)?;
    Ok(())
}

fn one_span(prefix: &[u32], text: &str, tokenizer: &ByteBpeTokenizer) -> Result<(usize, usize)> {
    let hits = occurrences(prefix, text, tokenizer)?;
    let [start] = hits.as_slice() else {
        return Err(format!("ambiguous value {text}").into());
    };
    let end = start + tokenizer.encode(&format!(" {text}")).len();
    Ok((*start, end))
}

fn datasets(root: &Path, train: bool, tokenizer: &ByteBpeTokenizer) -> Result<()> {
    let (split, verb, families) = if train {
        (
            "train",
            "picked",
            [
                Family {
                    names: ["Tom", "Sam"],
                    values: ["black", "white"],
                },
                Family {
                    names: ["Amy", "Max"],
                    values: ["gold coin", "gold key"],
                },
            ],
        )
    } else {
        (
            "held",
            "took",
            [
                Family {
                    names: ["Kim", "Mia"],
                    values: ["green", "brown"],
                },
                Family {
                    names: ["Ada", "Noah"],
                    values: ["silver bell", "silver leaf"],
                },
            ],
        )
    };
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let mut grades = Vec::new();
    let mut train_rows = Vec::new();
    let mut blind = Vec::new();
    let mut distinct = BTreeSet::new();
    for (family_index, family) in families.iter().enumerate() {
        for assignment in 0..2 {
            for order in 0..2 {
                let fact = |who: usize| {
                    format!(
                        "{} {verb} {}.",
                        family.names[who],
                        family.values[who ^ assignment]
                    )
                };
                let facts = format!("{} {}", fact(order), fact(1 - order));
                for position in ["early", "late"] {
                    for exclusion in ["none", "before", "after"] {
                        for tail in ["none", "short", "long"] {
                            let pair=format!("{split}-{family_index}-{assignment}-{order}-{position}-{exclusion}-{tail}");
                            for asked in 0..2 {
                                let name = family.names[asked];
                                let other = family.names[1 - asked];
                                let wording = (family_index ^ assignment ^ order) & 1;
                                let excluded = match (train, wording) {
                                    (true, 0) => format!("Ignore {other}."),
                                    (true, _) => format!("Leave {other} out."),
                                    (false, 0) => format!("Set {other} aside."),
                                    (false, _) => format!("Skip {other}."),
                                };
                                let question = match (train, position) {
                                    (true, "early") => format!("For {name}, what was picked?"),
                                    (true, _) => format!("Which item was picked by {name}?"),
                                    (false, "early") => {
                                        format!("About {name}, which item was taken?")
                                    }
                                    (false, _) => format!("What was the item that {name} took?"),
                                };
                                let trailer = match (train, tail) {
                                    (_, "none") => "",
                                    (true, "short") => " Answer.",
                                    (true, _) => " Give only the requested item in your answer.",
                                    (false, "short") => " Respond.",
                                    (false, _) => " Write just the name of the item here.",
                                };
                                let before = if exclusion == "before" {
                                    format!("{excluded} ")
                                } else {
                                    String::new()
                                };
                                let after = if exclusion == "after" {
                                    format!(" {excluded}")
                                } else {
                                    String::new()
                                };
                                let text = format!("{facts} {before}{question}{after}{trailer}");
                                let prefix = encoder
                                    .encode_assistant_prefix(&[Message {
                                        role: "user",
                                        content: &text,
                                    }])
                                    .tokens;
                                let rendered = format!("<|bos|>User: {text}\nAssistant:");
                                if tokenizer.decode(&prefix) != rendered
                                    || prefix.len() > 192
                                    || prefix.contains(&2)
                                    || !distinct.insert(prefix.clone())
                                {
                                    return Err(format!(
                                        "prefix domain/identity failure in {pair} {name}"
                                    )
                                    .into());
                                }
                                let entity_tokens = |s: &str| tokenizer.encode(&format!(" {s}"));
                                let mentioned = occurrences(&prefix, name, tokenizer)?;
                                let others = occurrences(&prefix, other, tokenizer)?;
                                if mentioned.len() != 2
                                    || others.len() != 1 + usize::from(exclusion != "none")
                                {
                                    return Err(
                                        format!("entity multiplicity failure in {pair}").into()
                                    );
                                }
                                let end_id =
                                    |start: usize, who: &str| start + entity_tokens(who).len() - 1;
                                let query = end_id(mentioned[1], name);
                                let other_fact = end_id(others[0], other);
                                let query_other = others.get(1).map(|&i| end_id(i, other));
                                let fact_anchor = end_id(mentioned[0], name);
                                let answer = family.values[asked ^ assignment];
                                let distractor = family.values[(1 - asked) ^ assignment];
                                let (source, end) = one_span(&prefix, answer, tokenizer)?;
                                let (other_source, other_end) =
                                    one_span(&prefix, distractor, tokenizer)?;
                                let expected: Vec<_> =
                                    prefix[source..end].iter().copied().chain([1]).collect();
                                let completed = encoder
                                    .encode_document(&[
                                        Message {
                                            role: "user",
                                            content: &text,
                                        },
                                        Message {
                                            role: "assistant",
                                            content: answer,
                                        },
                                    ])
                                    .tokens;
                                if completed != [prefix.as_slice(), expected.as_slice()].concat()
                                    || prefix[query] != prefix[fact_anchor]
                                    || query_other.is_some_and(|q| prefix[q] != prefix[other_fact])
                                {
                                    return Err(
                                        format!("target alignment failure in {pair}").into()
                                    );
                                }
                                let q_tokens = entity_tokens(&question);
                                let q0 = prefix
                                    .windows(q_tokens.len())
                                    .position(|w| w == q_tokens)
                                    .ok_or("cannot locate complete question")?;
                                if !(q0..q0 + q_tokens.len()).contains(&query) {
                                    return Err("query anchor outside question".into());
                                }
                                let labels = Labels {
                                    prefix: &prefix,
                                    query,
                                    query_other,
                                    fact: fact_anchor,
                                    other_fact,
                                    source,
                                    other_source,
                                };
                                if train {
                                    train_rows.push(serde_json::to_value(&labels)?);
                                }
                                blind.push(serde_json::to_value(Query { prefix: &prefix })?);
                                grades.push(json!({"id":format!("{pair}-ask-{name}"),"pair":pair,"role":name,"prefix":prefix,
          "asked_start":source,"asked_end":end,"other_start":other_source,"other_end":other_end,"expected":expected,
          "factors":{"lexical_set":family_index,"assignment":assignment,"fact_order":order,"asked_role":asked,
            "requested_position":position,"exclusion":exclusion,"exclusion_wording":if exclusion=="none" {None}else{Some(wording)},
            "tail":tail},"token_positions":{"query":query,"query_other":query_other,"query_within_question":query-q0,
            "query_inverse_offset":prefix.len()-1-query,"fact":fact_anchor,"other_fact":other_fact,"trailing_token_count":tokenizer.encode(trailer).len()},
          "prompt":text,"answer_text":answer,"other_text":distractor}));
                            }
                        }
                    }
                }
            }
        }
    }
    if grades.len() != 288 {
        return Err("factor count mismatch".into());
    }
    write_rows(root, &format!("{split}-grade.json"), &grades)?;
    if train {
        write_rows(root, "train.json", &train_rows)?;
    } else {
        write_rows(root, "held-blind.json", &blind)?;
    }
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [tokenizer, original, out] = args.as_slice() else {
        return Err("usage: content-selector-panel TOKENIZER ORIGINAL NEW_OUTPUT_DIR".into());
    };
    let raw = fs::read(tokenizer)?;
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&raw).ok_or("invalid public BPE")?;
    let out = Path::new(out);
    fs::create_dir(out)?;
    datasets(out, true, &t)?;
    datasets(out, false, &t)?;
    let original: Vec<Value> = serde_json::from_slice(&fs::read(original)?)?;
    if original.len() != 16 {
        return Err("original suite size changed".into());
    }
    let mut grade = Vec::new();
    let mut blind = Vec::new();
    for mut v in original {
        let prefix: Vec<u32> = serde_json::from_value(v["prefix"].clone())?;
        let ids: Vec<u32> = serde_json::from_value(v["expected"].clone())?;
        let a = v["asked_start"].as_u64().ok_or("bad source")? as usize;
        let b = v["asked_end"].as_u64().ok_or("bad end")? as usize;
        let answer = prefix
            .get(a..b)
            .filter(|s| !s.is_empty())
            .ok_or("bad original value span")?;
        let target = ids
            .strip_suffix(&[1])
            .ok_or("original target must terminate in EOS")?;
        if answer != target {
            return Err("original target boundary mismatch".into());
        }
        blind.push(json!({"prefix":prefix}));
        v["factors"] = json!({"split":"original"});
        grade.push(v);
    }
    write_rows(out, "original-grade.json", &grade)?;
    write_rows(out, "original-blind.json", &blind)?;
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::FAILURE
        }
    }
}
