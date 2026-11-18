//! Generate tokenizer-only role and connector panels for the span-selector study.
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::error::Error;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{env, fs};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const COMMON: [&str; 3] = ["has", "has kept", "has just kept"];
const TRAIN_ONLY: [&str; 3] = ["found", "once found", "once again found"];
const HELD_ONLY: [&str; 3] = ["took", "chose", "also chose"];

fn write_rows(root: &Path, file: &str, rows: &[Value]) -> Result<()> {
    let mut out = BufWriter::new(fs::File::create_new(root.join(file))?);
    out.write_all(b"[\n")?;
    for (i, row) in rows.iter().enumerate() {
        if i != 0 {
            out.write_all(b",\n")?;
        }
        serde_json::to_writer(&mut out, row)?;
    }
    out.write_all(b"\n]\n")?;
    out.flush()?;
    Ok(())
}

fn tokens(t: &ByteBpeTokenizer, surface: &str) -> Vec<u32> {
    t.encode(&format!(" {surface}"))
}

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize, // inclusive for entities
}

fn occurrences(ids: &[u32], value: &[u32]) -> Result<Vec<Span>> {
    if value.is_empty() {
        return Err("empty tokenized surface".into());
    }
    Ok(ids
        .windows(value.len())
        .enumerate()
        .filter_map(|(i, w)| {
            (w == value).then_some(Span {
                start: i,
                end: i + value.len() - 1,
            })
        })
        .collect())
}

fn value_span(ids: &[u32], t: &ByteBpeTokenizer, value: &str) -> Result<Span> {
    let hits = occurrences(ids, &tokens(t, value))?;
    match hits.as_slice() {
        [site] => Ok(*site),
        _ => Err(format!("nonunique value {value}").into()),
    }
}

struct Sites {
    query: Span,
    fact: Span,
    other_fact: Span,
    excluded: Option<Span>,
}

impl Sites {
    fn locate(ids: &[u32], t: &ByteBpeTokenizer, name: &str, other: &str) -> Result<Self> {
        let requested = occurrences(ids, &tokens(t, name))?;
        let others = occurrences(ids, &tokens(t, other))?;
        let [fact, query] = requested.as_slice() else {
            return Err("requested occurrence count".into());
        };
        let (other_fact, excluded) = match others.as_slice() {
            [f] => (*f, None),
            [f, x] => (*f, Some(*x)),
            _ => return Err("other occurrence count".into()),
        };
        Ok(Self {
            query: *query,
            fact: *fact,
            other_fact,
            excluded,
        })
    }

    fn json(&self) -> Value {
        json!({"query":self.query.end,"fact":self.fact.end,"other_fact":self.other_fact.end,
            "query_other":self.excluded.map(|s|s.end),"query_start":self.query.start,
            "query_end":self.query.end,"fact_start":self.fact.start,"fact_end":self.fact.end,
            "other_fact_start":self.other_fact.start,"other_fact_end":self.other_fact.end,
            "query_other_start":self.excluded.map(|s|s.start),"query_other_end":self.excluded.map(|s|s.end)})
    }
}

#[derive(Clone, Copy)]
struct Cell {
    family: usize,
    assignment: usize,
    order: usize,
    lengths: [usize; 2],
    position: &'static str,
    exclusion: &'static str,
    tail: &'static str,
    asked: usize,
}

fn row(
    t: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    train: bool,
    cell: Cell,
) -> Result<(Value, Value, Value)> {
    let Cell {
        family,
        assignment,
        order,
        lengths,
        position,
        exclusion,
        tail,
        asked,
    } = cell;
    let split = if train { "train" } else { "held" };
    let names = if train {
        [["Tim", "Sue"], ["Nala", "Navi"]]
    } else {
        [["Joe", "Leo"], ["Rana", "Rani"]]
    }[family];
    let value_set = family ^ order;
    let values = if train {
        [["gray", "blue"], ["small cup", "small hat"]]
    } else {
        [["yellow", "pink"], ["round stone", "round coin"]]
    }[value_set];
    let connectors = if assignment == 0 {
        COMMON
    } else if train {
        TRAIN_ONLY
    } else {
        HELD_ONLY
    };
    let verb = |who: usize| connectors[lengths[who] - 1];
    let fact = |who: usize| format!("{} {} {}.", names[who], verb(who), values[who ^ assignment]);
    let facts = format!("{} {}", fact(order), fact(1 - order));
    let name = names[asked];
    let other = names[1 - asked];
    let question = match (train, position) {
        (true, "early") => format!("For {name}, which item was listed?"),
        (true, _) => format!("Which item is listed for {name}?"),
        (false, "early") => format!("About {name}, which item was listed?"),
        (false, _) => format!("What is the item that {name} has?"),
    };
    let neg = if train {
        format!("Ignore {other}.")
    } else {
        format!("Set {other} aside.")
    };
    let inquiry = match exclusion {
        "before" => format!("{neg} {question}"),
        "after" => format!("{question} {neg}"),
        _ => question.clone(),
    };
    let suffix = match (train, tail) {
        (_, "none") => "",
        (true, _) => " Give only the requested item.",
        (false, _) => " Write just the name of the item here.",
    };
    let text = format!("{facts} {inquiry}{suffix}");
    let prefix = encoder
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: &text,
        }])
        .tokens;
    if t.decode(&prefix) != format!("<|bos|>User: {text}\nAssistant:")
        || prefix.len() + 16 > 192
        || prefix.contains(&2)
    {
        return Err("invalid rendered prefix".into());
    }
    let sites = Sites::locate(&prefix, t, name, other)?;
    let value = values[asked ^ assignment];
    let other_value = values[(1 - asked) ^ assignment];
    let src = value_span(&prefix, t, value)?;
    let distract = value_span(&prefix, t, other_value)?;
    let expected: Vec<_> = prefix[src.start..=src.end]
        .iter()
        .copied()
        .chain([1])
        .collect();
    let full = encoder
        .encode_document(&[
            Message {
                role: "user",
                content: &text,
            },
            Message {
                role: "assistant",
                content: value,
            },
        ])
        .tokens;
    let check_connector = |fact: Span, source: usize, who| -> Result<()> {
        let gap = prefix
            .get(fact.end + 1..source)
            .ok_or("unordered connector")?;
        if gap != tokens(t, verb(who)) || gap.len() != lengths[who] {
            return Err(format!("connector IDs {gap:?} vs {} in {text}", lengths[who]).into());
        }
        Ok(())
    };
    check_connector(sites.fact, src.start, asked)?;
    check_connector(sites.other_fact, distract.start, 1 - asked)?;
    if full != [prefix.as_slice(), expected.as_slice()].concat()
        || sites.excluded.is_some() != (exclusion != "none")
        || sites
            .excluded
            .is_some_and(|s| (s.end < sites.query.start) != (exclusion == "before"))
        || (sites.fact.start < sites.other_fact.start) != (asked == order)
        || (sites.query.start == sites.query.end) != (family == 0)
    {
        return Err("role, order, annotation or target mismatch".into());
    }
    let pair = format!(
        "{split}-f{family}-a{assignment}-o{order}-c{}{}-{position}-{exclusion}-{tail}",
        lengths[0], lengths[1]
    );
    let factors = json!({"lexical_set":family,"assignment":assignment,"fact_order":order,"value_set":value_set,
        "connector_set":if assignment==0{"shared"}else{"split_specific"},"connector_0_len":lengths[0],
        "connector_1_len":lengths[1],"asked_connector_len":lengths[asked],"other_connector_len":lengths[1-asked],
        "requested_position":position,"exclusion":exclusion,"tail":tail,"asked_role":asked});
    let labels = json!({"prefix":prefix,"query_start":sites.query.start,"query_end":sites.query.end,
        "fact_start":sites.fact.start,"fact_end":sites.fact.end,"source":src.start});
    let grade = json!({"id":format!("{pair}-{name}"),"pair":pair,"role":name,"expected":expected,
        "prefix":prefix,"asked_start":src.start,"asked_end":src.end+1,"other_start":distract.start,"other_end":distract.end+1,
        "token_positions":sites.json(),"factors":factors,"prompt":text,"answer_text":value,"other_text":other_value});
    Ok((labels, json!({"prefix":prefix}), grade))
}

fn generated(
    root: &Path,
    t: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    train: bool,
    census: &mut BTreeSet<Vec<u32>>,
) -> Result<()> {
    let mut grades = Vec::new();
    let mut inputs = Vec::new();
    for family in 0..2 {
        for assignment in 0..2 {
            for order in 0..2 {
                for l0 in 1..=3 {
                    for l1 in 1..=3 {
                        for position in ["early", "late"] {
                            for exclusion in ["none", "before", "after"] {
                                for tail in ["none", "long"] {
                                    for asked in 0..2 {
                                        let cell = Cell {
                                            family,
                                            assignment,
                                            order,
                                            lengths: [l0, l1],
                                            position,
                                            exclusion,
                                            tail,
                                            asked,
                                        };
                                        let (labels, blind, grade) = row(t, encoder, train, cell)?;
                                        if !census.insert(serde_json::from_value(
                                            blind["prefix"].clone(),
                                        )?) {
                                            return Err("duplicate prefix across panels".into());
                                        }
                                        inputs.push(if train { labels } else { blind });
                                        grades.push(grade);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if grades.len() != 1728 {
        return Err("factor count mismatch".into());
    }
    write_rows(
        root,
        if train {
            "train.json"
        } else {
            "held-blind.json"
        },
        &inputs,
    )?;
    write_rows(
        root,
        if train {
            "train-grade.json"
        } else {
            "held-grade.json"
        },
        &grades,
    )
}

fn historic(
    root: &Path,
    previous: &Path,
    split: &str,
    t: &ByteBpeTokenizer,
    census: &mut BTreeSet<Vec<u32>>,
) -> Result<()> {
    let grade: Vec<Value> =
        serde_json::from_slice(&fs::read(previous.join(format!("{split}-grade.json")))?)?;
    let blind: Vec<Value> =
        serde_json::from_slice(&fs::read(previous.join(format!("{split}-blind.json")))?)?;
    let count = if split == "original" { 16 } else { 288 };
    if grade.len() != count || grade.len() != blind.len() {
        return Err("historical split size mismatch".into());
    }
    let mut out = Vec::new();
    for (r, b) in grade.iter().zip(&blind) {
        let name = r["role"].as_str().ok_or("missing role")?;
        let peer = grade
            .iter()
            .find(|p| p["pair"] == r["pair"] && p["role"] != r["role"])
            .ok_or("unpaired historical row")?;
        let other = peer["role"].as_str().ok_or("missing peer")?;
        let ids: Vec<u32> = serde_json::from_value(r["prefix"].clone())?;
        if b != &json!({"prefix":ids}) || !census.insert(ids.clone()) {
            return Err("historical blind mismatch/overlap".into());
        }
        let sites = Sites::locate(&ids, t, name, other)?;
        let start =
            |key: &str| -> Result<usize> { Ok(r[key].as_u64().ok_or("missing span")?.try_into()?) };
        let s = start("asked_start")?;
        let e = start("asked_end")?;
        let os = start("other_start")?;
        let oe = start("other_end")?;
        let expected: Vec<u32> = serde_json::from_value(r["expected"].clone())?;
        if e <= s
            || oe <= os
            || !(e <= os || oe <= s)
            || e > ids.len()
            || oe > ids.len()
            || expected.strip_suffix(&[1]) != Some(&ids[s..e])
        {
            return Err("historical span mismatch".into());
        }
        let al = s
            .checked_sub(sites.fact.end + 1)
            .ok_or("bad historical relation")?;
        let ol = os
            .checked_sub(sites.other_fact.end + 1)
            .ok_or("bad historical distractor relation")?;
        let required = if split == "original" { 2 } else { 1 };
        if al != required || ol != required {
            return Err("historical connector identity mismatch".into());
        }
        let mut annotated = r.clone();
        let positions = sites.json();
        annotated["token_positions"] = positions;
        let role_index = if split == "original" {
            usize::from(name == "Ben")
        } else {
            start_role(r)?
        };
        for (key, len) in [
            ("asked_connector_len", al),
            ("other_connector_len", ol),
            ("connector_0_len", if role_index == 0 { al } else { ol }),
            ("connector_1_len", if role_index == 1 { al } else { ol }),
        ] {
            annotated["factors"][key] = json!(len);
        }
        out.push(annotated);
    }
    let file = if split == "original" {
        "original"
    } else {
        "prior"
    };
    write_rows(root, &format!("{file}-grade.json"), &out)?;
    write_rows(root, &format!("{file}-blind.json"), &blind)?;
    Ok(())
}

fn start_role(r: &Value) -> Result<usize> {
    Ok(r["factors"]["asked_role"]
        .as_u64()
        .ok_or("missing role index")?
        .try_into()?)
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [tok, previous, out] = args.as_slice() else {
        return Err("usage: span-connector-panel TOKENIZER PREVIOUS_PANEL_DIR NEW_OUT".into());
    };
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tok)?)
        .ok_or("invalid public tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&t)?;
    let encoder = protocol.bind(&t)?;
    let root = Path::new(out);
    fs::create_dir(root)?;
    let mut census = BTreeSet::new();
    generated(root, &t, &encoder, true, &mut census)?;
    generated(root, &t, &encoder, false, &mut census)?;
    for split in ["original", "held"] {
        historic(root, Path::new(previous), split, &t, &mut census)?;
    }
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
