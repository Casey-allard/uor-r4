//! Fully crossed query and evidence interventions with independent token-span keys.
use super::{
    json, write_rows, BTreeSet, ByteBpeTokenizer, DialogueEncoder, Message, Path, Result, Value,
};

fn query(train: bool, form: usize, requested: &str, other: &str) -> String {
    match (train, form) {
        (true, 0) => format!("Which item belongs to {requested}?"),
        (true, 1) => format!("For {requested} rather than {other}, give the item."),
        (true, _) => format!("Ignoring {other}, which item was kept by {requested}?"),
        (false, 0) => format!("Regarding {requested}, what object did the record name?"),
        (false, 1) => format!("Tell me the thing for {requested}, excluding {other}."),
        (false, _) => format!("Set aside {other}, and describe the possession of {requested}."),
    }
}

pub(super) fn token_span(
    t: &ByteBpeTokenizer,
    prefix: &[u32],
    surface: &str,
    byte: usize,
) -> Result<(usize, usize)> {
    let ids = t.encode(&format!(" {surface}"));
    let span = prefix
        .windows(ids.len())
        .enumerate()
        .find(|(i, window)| *window == ids && t.decode(&prefix[..*i]).len() + 1 == byte)
        .ok_or_else(|| format!("unaligned span {surface} at byte {byte}"))?;
    Ok((span.0, span.0 + ids.len()))
}

pub(super) fn generate(root: &Path, t: &ByteBpeTokenizer, enc: &DialogueEncoder<'_>) -> Result<()> {
    for train in [true, false] {
        let split = if train { "train" } else { "held" };
        let names = if train {
            [
                ["Kim", "Bob", "Moe", "Sam"],
                ["Nala", "Navi", "Nora", "Niva"],
            ]
        } else {
            [
                ["May", "Joe", "Jim", "Roy"],
                ["Zada", "Zadi", "Zora", "Zuri"],
            ]
        };
        let values = if train {
            [["red", "white"], ["round box", "round bag"]]
        } else {
            [["pink", "brown"], ["large cup", "large hat"]]
        };
        let mut seen = BTreeSet::new();
        let mut blind = Vec::new();
        let mut grade = Vec::new();
        for (family, roster) in names.iter().enumerate() {
            for order in 0..2 {
                for assignment in 0..2 {
                    for asked in 0..2 {
                        let peer = 1 - asked;
                        let verb = if train { "found" } else { "held" };
                        let fact = |p: usize, v: usize| {
                            format!("{} {verb} {}.", roster[p], values[family][v])
                        };
                        for kind in [
                            "unique",
                            "removed",
                            "duplicate_agree",
                            "conflict",
                            "excluded",
                            "third",
                        ] {
                            let requested = if kind == "third" {
                                roster[2 + asked]
                            } else {
                                roster[asked]
                            };
                            let target = values[family][asked ^ assignment];
                            let mut facts = vec![
                                (fact(0, assignment), Some((0, assignment)), true),
                                (fact(1, 1 ^ assignment), Some((1, 1 ^ assignment)), true),
                            ];
                            if kind == "removed" {
                                facts[asked] = ("The line is blank.".to_string(), None, false);
                            }
                            if kind == "excluded" {
                                facts[asked].0 = format!("It is false that {}", facts[asked].0);
                                facts[asked].2 = false;
                            }
                            if order == 1 {
                                facts.reverse();
                            }
                            if kind == "duplicate_agree" || kind == "conflict" {
                                let v = if kind == "conflict" {
                                    peer ^ assignment
                                } else {
                                    asked ^ assignment
                                };
                                facts.push((fact(asked, v), Some((asked, v)), true));
                            }
                            let statement = facts
                                .iter()
                                .map(|f| f.0.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");
                            for form in 0..3 {
                                let question = query(train, form, requested, roster[peer]);
                                let prompt = format!("{statement} {question}");
                                let message = Message {
                                    role: "user",
                                    content: &prompt,
                                };
                                let prefix = enc.encode_assistant_prefix(&[message.clone()]).tokens;
                                let literal = format!("<|bos|>User: {prompt}\nAssistant:");
                                if t.decode(&prefix) != literal
                                    || prefix.contains(&2)
                                    || prefix.len() + 16 > 192
                                    || !seen.insert(prefix.clone())
                                {
                                    return Err(
                                        format!("duplicate or invalid prefix {prompt}").into()
                                    );
                                }
                                let qbyte = literal.find(&question).ok_or("missing question")?
                                    + question.find(requested).ok_or("missing requested person")?;
                                let (query_start, query_end) =
                                    token_span(t, &prefix, requested, qbyte)?;
                                let mut pos = "<|bos|>User: ".len();
                                let mut supports = Vec::new();
                                let mut annotations = Vec::new();
                                let mut peer_source = None;
                                for (text, assoc, affirmed) in &facts {
                                    if let Some((p, v)) = assoc {
                                        let person = roster[*p];
                                        let value = values[family][*v];
                                        let (fact_start, fact_end) = token_span(
                                            t,
                                            &prefix,
                                            person,
                                            pos + text.find(person).ok_or("missing person")?,
                                        )?;
                                        let (value_start, value_end) = token_span(
                                            t,
                                            &prefix,
                                            value,
                                            pos + text.find(value).ok_or("missing value")?,
                                        )?;
                                        let row = json!({ "person":person,"value":value,"fact_start":fact_start,"fact_end":fact_end,"value_start":value_start,"value_end":value_end,"affirmed":affirmed });
                                        if person == requested && *affirmed {
                                            supports.push(row.clone());
                                        }
                                        if *p == peer {
                                            peer_source = Some(value_start);
                                        }
                                        annotations.push(row);
                                    }
                                    pos += text.len() + 1;
                                }
                                let answerable = matches!(kind, "unique" | "duplicate_agree");
                                let expected = if answerable {
                                    let full = enc
                                        .encode_document(&[
                                            message,
                                            Message {
                                                role: "assistant",
                                                content: target,
                                            },
                                        ])
                                        .tokens;
                                    let output = full
                                        .strip_prefix(prefix.as_slice())
                                        .ok_or("reference differs from prefix")?;
                                    for source in &supports {
                                        let start =
                                            source["value_start"].as_u64().ok_or("missing start")?
                                                as usize;
                                        let end =
                                            source["value_end"].as_u64().ok_or("missing end")?
                                                as usize;
                                        if output != [&prefix[start..end], &[1]].concat() {
                                            return Err(
                                                "value disagrees with literal response".into()
                                            );
                                        }
                                    }
                                    Some(output.to_vec())
                                } else {
                                    None
                                };
                                let first = answerable.then(|| &supports[0]);
                                let id = format!(
                                    "{split}-{family}-{order}-{assignment}-{asked}-{form}-{kind}"
                                );
                                grade.push(json!({"id":id,"kind":kind,"prompt":prompt,"prefix":prefix,"query_start":query_start,"query_end":query_end,
                                    "asked_start":first.map(|a|&a["value_start"]),"asked_end":first.map(|a|&a["value_end"]),
                                    "other_start":peer_source,"forced_fact_source":first.map(|a|a["value_start"].clone()).or(peer_source.map(Value::from)),
                                    "query_form":form,"family":family,"order":order,"assignment":assignment,"asked_role":asked,
                                    "pair":format!("{split}-{family}-{order}-{assignment}-{form}-{kind}"),
                                    "contrast":format!("{statement} | {form}"),"role":requested,"excluded_role":if form==0 {None} else {Some(roster[peer])},
                                    "supports":supports,"assertions":annotations,"expected":expected,"answer_text":answerable.then_some(target)}));
                                blind.push(json!({"prefix":prefix}));
                            }
                        }
                    }
                }
            }
        }
        write_rows(root, &format!("{split}-blind.json"), &blind)?;
        write_rows(root, &format!("{split}-grade.json"), &grade)?;
    }
    Ok(())
}
