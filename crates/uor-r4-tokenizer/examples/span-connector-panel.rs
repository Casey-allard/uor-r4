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
const EXTENDS_TRAIN: [&str; 3] = ["found", "found only", "found and kept"];
const FRESH_ONLY: [&str; 3] = ["got", "collects", "now collects"];

#[derive(Clone, Copy)]
enum Suite {
    Train,
    Held,
    Shared,
    ShortPrefix,
    Unseen,
}

impl Suite {
    fn split(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Held => "held",
            _ => "fresh",
        }
    }

    fn is_fresh(self) -> bool {
        matches!(self, Self::Shared | Self::ShortPrefix | Self::Unseen)
    }
}

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
    suite: Suite,
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
    let train = matches!(suite, Suite::Train);
    let fresh = suite.is_fresh();
    let split = suite.split();
    let names = match suite {
        Suite::Train => [["Tim", "Sue"], ["Nala", "Navi"]],
        Suite::Held => [["Joe", "Leo"], ["Rana", "Rani"]],
        _ => [["Jim", "Jen"], ["Mira", "Miri"]],
    }[family];
    let value_set = family ^ order;
    let values = match suite {
        Suite::Train => [["gray", "blue"], ["small cup", "small hat"]],
        Suite::Held => [["yellow", "pink"], ["round stone", "round coin"]],
        _ => [["white", "gold"], ["old lamp", "old ring"]],
    }[value_set];
    let (set, connectors) = match suite {
        Suite::Train if assignment != 0 => ("split_specific", TRAIN_ONLY),
        Suite::Held if assignment != 0 => ("split_specific", HELD_ONLY),
        Suite::ShortPrefix => ("short_prefix", EXTENDS_TRAIN),
        Suite::Unseen => ("unseen", FRESH_ONLY),
        _ => ("shared", COMMON),
    };
    let verb = |who: usize| connectors[lengths[who] - 1];
    let fact = |who: usize| format!("{} {} {}.", names[who], verb(who), values[who ^ assignment]);
    let facts = format!("{} {}", fact(order), fact(1 - order));
    let name = names[asked];
    let other = names[1 - asked];
    let question = match (suite, position) {
        (Suite::Train, "early") => format!("For {name}, which item was listed?"),
        (Suite::Train, _) => format!("Which item is listed for {name}?"),
        (Suite::Held, "early") => format!("About {name}, which item was listed?"),
        (Suite::Held, _) => format!("What is the item that {name} has?"),
        (_, "early") => format!("About {name}, which item did the note list?"),
        (_, _) => format!("Which item did the note list for {name}?"),
    };
    let neg = if fresh {
        format!("Skip {other}.")
    } else if train {
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
    let pair = if fresh { format!("{pair}-{set}") } else { pair };
    let mut factors = json!({"lexical_set":family,"assignment":assignment,"fact_order":order,"value_set":value_set,
        "connector_set":set,"connector_0_len":lengths[0],
        "connector_1_len":lengths[1],"asked_connector_len":lengths[asked],"other_connector_len":lengths[1-asked],
        "requested_position":position,"exclusion":exclusion,"tail":tail,"asked_role":asked});
    if fresh {
        factors["asked_connector_surface"] = json!(verb(asked));
        let connector = &prefix[sites.fact.end + 1..src.start];
        let longest_prefix = COMMON
            .iter()
            .chain(&TRAIN_ONLY)
            .map(|verb| tokens(t, verb))
            .filter(|ids| connector.starts_with(ids))
            .map(|ids| ids.len())
            .max()
            .unwrap_or(0);
        factors["longest_train_prefix"] = json!(longest_prefix);
        factors["exact_train_connector"] = json!(longest_prefix == connector.len());
    }
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
    suites: &[Suite],
    census: &mut BTreeSet<Vec<u32>>,
) -> Result<()> {
    let mut grades = Vec::new();
    let mut inputs = Vec::new();
    let split = suites.first().ok_or("empty connector panel")?.split();
    for &suite in suites {
        let exclusions: &[&str] = if suite.is_fresh() {
            &["none", "before"]
        } else {
            &["none", "before", "after"]
        };
        let tails: &[&str] = if suite.is_fresh() {
            &["none"]
        } else {
            &["none", "long"]
        };
        if suite.split() != split {
            return Err("inconsistent panel split".into());
        }
        for family in 0..2 {
            for assignment in 0..2 {
                for order in 0..2 {
                    for l0 in 1..=3 {
                        for l1 in 1..=3 {
                            for position in ["early", "late"] {
                                for &exclusion in exclusions {
                                    for &tail in tails {
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
                                            let (labels, blind, grade) =
                                                row(t, encoder, suite, cell)?;
                                            if !census.insert(serde_json::from_value(
                                                blind["prefix"].clone(),
                                            )?) {
                                                return Err("duplicate prefix across panels".into());
                                            }
                                            inputs.push(if matches!(suite, Suite::Train) {
                                                labels
                                            } else {
                                                blind
                                            });
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
    }
    if grades.len() != 1728 {
        return Err("factor count mismatch".into());
    }
    let inputs_name = if split == "train" {
        "train.json".to_string()
    } else {
        format!("{split}-blind.json")
    };
    write_rows(root, &inputs_name, &inputs)?;
    write_rows(root, &format!("{split}-grade.json"), &grades)
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
        return Err(
            "usage: span-connector-panel TOKENIZER PREVIOUS_PANEL_DIR|--fresh NEW_OUT".into(),
        );
    };
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tok)?)
        .ok_or("invalid public tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&t)?;
    let encoder = protocol.bind(&t)?;
    let root = Path::new(out);
    fs::create_dir(root)?;
    let mut census = BTreeSet::new();
    if previous == "--fresh" {
        return generated(
            root,
            &t,
            &encoder,
            &[Suite::Shared, Suite::ShortPrefix, Suite::Unseen],
            &mut census,
        );
    }
    generated(root, &t, &encoder, &[Suite::Train], &mut census)?;
    generated(root, &t, &encoder, &[Suite::Held], &mut census)?;
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
