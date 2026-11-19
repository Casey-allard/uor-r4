//! Same-fact query swaps with independently decoded endpoint and value keys.
use super::support::token_span;
use super::{
    json, write_rows, BTreeSet, ByteBpeTokenizer, DialogueEncoder, Message, Path, Result, Value,
};

const USER: &str = "<|bos|>User: ";

struct Variant {
    family: &'static str,
    wording: &'static str,
    roles: &'static [u8],
}

// Marked slots are renderer metadata. No marker or annotation is in a blind tape.
const TRAIN: &[Variant] = &[
    Variant {
        family: "for-rather",
        wording: "For @ rather than @, give the item.",
        roles: &[1, 2],
    },
    Variant {
        family: "excluding-before",
        wording: "Excluding @, what did @ keep?",
        roles: &[2, 1],
    },
    Variant {
        family: "ignoring-by",
        wording: "Ignoring @, which item was kept by @?",
        roles: &[2, 1],
    },
    Variant {
        family: "regarding",
        wording: "Regarding @, what did the line name?",
        roles: &[1],
    },
    Variant {
        family: "belongs-to",
        wording: "Which item belongs to @?",
        roles: &[1],
    },
    Variant {
        family: "for-ignore",
        wording: "Find the item for @. Ignore @, please.",
        roles: &[1, 2],
    },
    Variant {
        family: "beside",
        wording: "For @, beside the mention of @, name the item.",
        roles: &[1, 0],
    },
    Variant {
        family: "between",
        wording: "Between @ and @, what item did the unnamed person keep?",
        roles: &[0, 0],
    },
    Variant {
        family: "either",
        wording: "For either @ or @, tell me their item.",
        roles: &[0, 0],
    },
    Variant {
        family: "ordinary",
        wording: "Explain why Cedar and Oak are different trees.",
        roles: &[],
    },
];
const HELD: &[Variant] = &[
    Variant {
        family: "excluding-after",
        wording: "Tell me the thing for @, excluding @.",
        roles: &[1, 2],
    },
    Variant {
        family: "ignoring-after",
        wording: "What did @ keep, ignoring @?",
        roles: &[1, 2],
    },
    Variant {
        family: "regarding-tail",
        wording: "Regarding @, name the object, even if Cedar saw @.",
        roles: &[1, 0],
    },
    Variant {
        family: "belongs-after",
        wording: "What belongs to @, rather than @, according to Harbor?",
        roles: &[1, 2],
    },
    Variant {
        family: "unspecified-after",
        wording: "Of @ and @, what did the unspecified person hold?",
        roles: &[0, 0],
    },
    Variant {
        family: "ordinary-after",
        wording: "When might Harbor and Summit agree about a blue sky?",
        roles: &[],
    },
];
const FRESH: &[Variant] = &[
    Variant {
        family: "set-aside",
        wording: "Set aside @, and describe the possession of @.",
        roles: &[2, 1],
    },
    Variant {
        family: "skip-serve",
        wording: "Skip @. Report what @ was carrying for Cedar.",
        roles: &[2, 1],
    },
    Variant {
        family: "except-before",
        wording: "Except for @, which object was held by @?",
        roles: &[2, 1],
    },
    Variant {
        family: "for-conversation",
        wording: "Give me the record for @ even though @ had a long conversation with Summit.",
        roles: &[1, 0],
    },
    Variant {
        family: "someone-unknown",
        wording: "Which object did someone unmentioned hold, with @ and @ standing nearby?",
        roles: &[0, 0],
    },
    Variant {
        family: "ordinary-fresh",
        wording: "Can Cedar and Summit be used as names for colors?",
        roles: &[],
    },
];

fn render(variant: &Variant, people: [&str; 2]) -> Result<(String, Vec<(u8, usize, usize)>)> {
    let mut segments = variant.wording.split('@');
    let mut question = segments.next().ok_or("missing wording")?.to_owned();
    let mut sites = Vec::new();
    for (i, suffix) in segments.enumerate() {
        let role = *variant.roles.get(i).ok_or("extra query slot")?;
        let at = question.len();
        question.push_str(people[i]);
        sites.push((role, at, i));
        question.push_str(suffix);
    }
    if sites.len() != variant.roles.len() {
        return Err("missing query slot".into());
    }
    Ok((question, sites))
}

pub(super) fn generate(root: &Path, t: &ByteBpeTokenizer, enc: &DialogueEncoder<'_>) -> Result<()> {
    let mut all = BTreeSet::new();
    for (split, variants, rosters, values, verb) in [
        (
            "train",
            TRAIN,
            [
                ["Kim", "Bob", "Ruma", "Rumi"],
                ["Nala", "Navi", "Moe", "Sam"],
            ],
            [["red", "white"], ["round box", "round bag"]],
            "found",
        ),
        (
            "held",
            HELD,
            [
                ["May", "Joe", "Zura", "Zuri"],
                ["Lava", "Lavi", "Jim", "Roy"],
            ],
            [["pink", "brown"], ["large cup", "large hat"]],
            "held",
        ),
        (
            "fresh",
            FRESH,
            [
                ["Ben", "Eve", "Vela", "Veli"],
                ["Sani", "Sana", "Umber", "Onyx"],
            ],
            [["green", "black"], ["small ring", "small tray"]],
            "kept",
        ),
    ] {
        let mut blind = Vec::new();
        let mut grade = Vec::new();
        for (roster_id, roster) in rosters.iter().enumerate() {
            for lower in [false, true] {
                let roster = roster.map(|s| {
                    if lower {
                        s.to_ascii_lowercase()
                    } else {
                        s.to_owned()
                    }
                });
                for order in 0..2 {
                    for assignment in 0..2 {
                        for evidence in [
                            "unique",
                            "absent",
                            "removed",
                            "negated",
                            "duplicate_agree",
                            "conflict",
                        ] {
                            let fact = |p: usize, v: usize, affirmed: bool| {
                                let negation = if affirmed { "" } else { "It is false that " };
                                (
                                    format!(
                                        "{negation}{} {verb} {}.",
                                        roster[p], values[roster_id][v]
                                    ),
                                    Some((p, v, affirmed)),
                                )
                            };
                            let mut facts = [
                                fact(0, assignment, evidence != "negated"),
                                fact(1, 1 ^ assignment, true),
                            ]
                            .to_vec();
                            if evidence == "removed" {
                                facts[0] = ("The line is blank.".to_owned(), None);
                            }
                            if order == 1 {
                                facts.reverse();
                            }
                            if evidence == "duplicate_agree" {
                                facts.push(fact(0, assignment, true));
                            }
                            if evidence == "conflict" {
                                facts.push(fact(0, 1 ^ assignment, true));
                            }
                            for variant in variants {
                                // Ordinary requests and unknown referents appear with facts too.
                                // The identical absent-history fact text is distinguished by one stable neutral note.
                                let mut statement = facts
                                    .iter()
                                    .map(|f| f.0.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if evidence == "absent" {
                                    statement.push_str(" The record was closed.");
                                }
                                for swap in 0..if variant.roles.is_empty() { 1 } else { 2 } {
                                    let ids = if evidence == "absent" && variant.roles.contains(&1)
                                    {
                                        if variant.roles[0] == 1 {
                                            [2 + swap, 1 - swap]
                                        } else {
                                            [1 - swap, 2 + swap]
                                        }
                                    } else {
                                        [swap, 1 - swap]
                                    };
                                    let people = [roster[ids[0]].as_str(), roster[ids[1]].as_str()];
                                    let (question, sites) = render(variant, people)?;
                                    let prompt = format!("{statement} {question}");
                                    let message = Message {
                                        role: "user",
                                        content: &prompt,
                                    };
                                    let prefix =
                                        enc.encode_assistant_prefix(&[message.clone()]).tokens;
                                    let literal = format!("{USER}{prompt}\nAssistant:");
                                    if t.decode(&prefix) != literal
                                        || prefix.contains(&2)
                                        || prefix.len() + 16 > 192
                                        || !all.insert(prefix.clone())
                                    {
                                        return Err(format!(
                                            "duplicate or invalid prompt {prompt}"
                                        )
                                        .into());
                                    }
                                    let mut role_sites = [None; 3];
                                    let mut requested_person = None;
                                    let mut excluded_person = None;
                                    for (role, at, i) in sites {
                                        let span = token_span(
                                            t,
                                            &prefix,
                                            people[i],
                                            USER.len() + statement.len() + 1 + at,
                                        )?;
                                        if role != 0 {
                                            role_sites[role as usize] = Some(span);
                                        }
                                        if role == 1 {
                                            requested_person = Some(ids[i]);
                                        }
                                        if role == 2 {
                                            excluded_person = Some(ids[i]);
                                        }
                                    }
                                    let mut supports = Vec::new();
                                    let mut assertions = Vec::new();
                                    let mut value_set = BTreeSet::new();
                                    let mut peer_source = None;
                                    let mut offset = USER.len();
                                    for (s, annotation) in &facts {
                                        if let Some((p, v, affirmed)) = annotation {
                                            let person = &roster[*p];
                                            let value = values[roster_id][*v];
                                            let site = |word: &str| {
                                                token_span(
                                                    t,
                                                    &prefix,
                                                    word,
                                                    offset + s.find(word).unwrap_or(s.len()),
                                                )
                                            };
                                            let (fact_start, fact_end) = site(person)?;
                                            let (value_start, value_end) = site(value)?;
                                            let record = json!({"person":person,"value":value,"fact_start":fact_start,"fact_end":fact_end,"value_start":value_start,"value_end":value_end,"affirmed":affirmed});
                                            if Some(*p) == requested_person && *affirmed {
                                                supports.push(record.clone());
                                                value_set.insert(value);
                                            }
                                            if Some(*p) == excluded_person && *affirmed {
                                                peer_source = Some(value_start);
                                            }
                                            assertions.push(record);
                                        }
                                        offset += s.len() + 1;
                                    }
                                    let answer =
                                        (value_set.len() == 1).then(|| *value_set.first().unwrap());
                                    let expected = if let Some(answer) = answer {
                                        let full = enc
                                            .encode_document(&[
                                                message,
                                                Message {
                                                    role: "assistant",
                                                    content: answer,
                                                },
                                            ])
                                            .tokens;
                                        let output = full
                                            .strip_prefix(prefix.as_slice())
                                            .ok_or("reference differs from prefix")?;
                                        for source in &supports {
                                            let start = source["value_start"]
                                                .as_u64()
                                                .ok_or("missing start")?
                                                as usize;
                                            let end =
                                                source["value_end"].as_u64().ok_or("missing end")?
                                                    as usize;
                                            if output != [&prefix[start..end], &[1]].concat() {
                                                return Err(
                                                    "literal value differs from source".into()
                                                );
                                            }
                                        }
                                        Some(output.to_vec())
                                    } else {
                                        None
                                    };
                                    let kind = match (
                                        requested_person.is_some(),
                                        value_set.len(),
                                        supports.len(),
                                        variant.roles.is_empty(),
                                    ) {
                                        (false, _, _, true) => "ordinary",
                                        (false, _, _, false) => "unspecified",
                                        (true, 0, _, _) => evidence,
                                        (true, 1, 1, _) => "unique",
                                        (true, 1, _, _) => "duplicate_agree",
                                        _ => "conflict",
                                    };
                                    let first = answer.map(|_| &supports[0]);
                                    let group = format!("{split}-{roster_id}-{lower}-{order}-{assignment}-{evidence}-{}", variant.family);
                                    let boundary = |role: usize, end: bool| {
                                        role_sites[role].map(|s| if end { s.1 } else { s.0 })
                                    };
                                    let id = format!("{group}-{swap}");
                                    grade.push(json!({"id":id,"pair":group,"kind":kind,"evidence":evidence,"construction":variant.family,"case":if lower {"lower"} else {"title"},"query_start":boundary(1,false),"query_end":boundary(1,true),"excluded_start":boundary(2,false),"excluded_end":boundary(2,true),"prompt":prompt,"prefix":prefix,"supports":supports,"assertions":assertions,"expected":expected,"answer_text":answer,"asked_start":first.map(|s|s["value_start"].clone()),"asked_end":first.map(|s|s["value_end"].clone()),"other_start":peer_source,"forced_fact_source":first.map(|s|s["value_start"].clone()).or(peer_source.map(Value::from))}));
                                    blind.push(json!({"prefix":prefix}));
                                }
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
