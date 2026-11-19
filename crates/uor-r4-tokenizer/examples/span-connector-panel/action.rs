//! Counterfactual action supervision: the same facts appear with four query intentions.
use super::{
    json, value_span, write_rows, BTreeSet, ByteBpeTokenizer, DialogueEncoder, Message, Path,
    Result,
};

pub(super) fn generate(
    root: &Path,
    t: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for train in [true, false] {
        let split = if train { "train" } else { "held" };
        let rosters = if train {
            [["Kim", "Bob", "Sam"], ["Nala", "Navi", "Nora"]]
        } else {
            [["Tom", "Sue", "Ben"], ["Rava", "Ravi", "Rona"]]
        };
        let values = if train {
            [["red", "white"], ["round box", "round bag"]]
        } else {
            [["blue", "gold"], ["small cup", "small hat"]]
        };
        let mut bare = Vec::new();
        let mut grade = Vec::new();
        for (family, names) in rosters.iter().enumerate() {
            for missing in 0..3 {
                let people = [names[(missing + 1) % 3], names[(missing + 2) % 3]];
                for order in 0..2 {
                    for assignment in 0..2 {
                        let words = values[family];
                        let fact = |who: usize| {
                            let verb = if train {
                                ["found", "has kept"][who]
                            } else {
                                ["held", "once held"][who]
                            };
                            format!("{} {} {}.", people[who], verb, words[who ^ assignment])
                        };
                        let facts = format!("{} {}", fact(order), fact(1 - order));
                        for asked in 0..2 {
                            let name = people[asked];
                            let other = people[1 - asked];
                            let query = |person: &str| {
                                if train {
                                    if asked == 0 {
                                        format!("For {person}, which object belongs to them?")
                                    } else {
                                        format!("Whose object is recorded for {person}? Give the object.")
                                    }
                                } else if asked == 0 {
                                    format!("Tell me the item from the note about {person}.")
                                } else {
                                    format!("Regarding {person}, what did the entry name?")
                                }
                            };
                            let ambiguous = if train {
                                format!("One of {name} and {other} is my choice. Which object is theirs?")
                            } else {
                                format!("Between {name} and {other}, what did the unspecified person hold?")
                            };
                            let plain = if train {
                                ["What comes after Monday?", "What is one plus one?"][asked]
                            } else {
                                ["What is two plus two?", "Which planet do we live on?"][asked]
                            };
                            let answer = if train {
                                ["Tuesday", "two"][asked]
                            } else {
                                ["four", "Earth"][asked]
                            };
                            let pair = format!("{split}-{family}-{missing}-{order}-{assignment}");
                            for (kind, text, target) in [
                                ("positive", query(name), Some(words[asked ^ assignment])),
                                ("absent", query(names[missing]), None),
                                ("ambiguous", ambiguous, None),
                                ("prose", plain.to_string(), Some(answer)),
                            ] {
                                let prompt = format!("{facts} {text}");
                                let prefix = encoder
                                    .encode_assistant_prefix(&[Message {
                                        role: "user",
                                        content: &prompt,
                                    }])
                                    .tokens;
                                if !seen.insert(prefix.clone())
                                    || prefix.contains(&2)
                                    || prefix.len() + 16 > 192
                                    || t.decode(&prefix)
                                        != format!("<|bos|>User: {prompt}\nAssistant:")
                                {
                                    return Err(format!("duplicate/invalid {kind} {pair}").into());
                                }
                                let expected = target
                                    .map(|surface| {
                                        let full = encoder
                                            .encode_document(&[
                                                Message {
                                                    role: "user",
                                                    content: &prompt,
                                                },
                                                Message {
                                                    role: "assistant",
                                                    content: surface,
                                                },
                                            ])
                                            .tokens;
                                        full.strip_prefix(prefix.as_slice())
                                            .map(<[u32]>::to_vec)
                                            .ok_or("bad answer prefix")
                                    })
                                    .transpose()?;
                                let original = value_span(&prefix, t, words[asked ^ assignment])?;
                                let peer = value_span(&prefix, t, words[(1 - asked) ^ assignment])?;
                                if kind == "positive"
                                    && expected
                                        != Some(
                                            [&prefix[original.start..=original.end], &[1]].concat(),
                                        )
                                {
                                    return Err("literal value is not expected response".into());
                                }
                                bare.push(json!({"prefix":prefix}));
                                grade.push(json!({"id":format!("{pair}-{asked}-{kind}"),"pair":pair,"role":name,"kind":kind,
                                   "prefix":prefix,"prompt":prompt,"expected":expected,"answer_text":target,
                                   "asked_start":(kind=="positive").then_some(original.start),
                                   "asked_end":(kind=="positive").then_some(original.end+1),"other_start":peer.start,
                                   "forced_fact_source":original.start,"absent_name":names[missing]}));
                            }
                        }
                    }
                }
            }
        }
        // Ordinary questions without prefixed facts probe the unconditioned generate path separately.
        let ordinary = if train {
            [
                ("What is ice made of?", "water"),
                ("What number follows two?", "three"),
            ]
        } else {
            [
                ("What color is fresh snow?", "white"),
                ("How many days are in a week?", "seven"),
            ]
        };
        for (i, (prompt, answer)) in ordinary.into_iter().enumerate() {
            let prefix = encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: prompt,
                }])
                .tokens;
            let full = encoder
                .encode_document(&[
                    Message {
                        role: "user",
                        content: prompt,
                    },
                    Message {
                        role: "assistant",
                        content: answer,
                    },
                ])
                .tokens;
            let expected = full
                .strip_prefix(prefix.as_slice())
                .ok_or("ordinary reference mismatch")?;
            if !seen.insert(prefix.clone()) || prefix.contains(&2) {
                return Err("invalid ordinary prompt".into());
            }
            bare.push(json!({"prefix":prefix}));
            grade.push(json!({"id":format!("{split}-ordinary-{i}"),"pair":null,"role":null,"kind":"prose","prompt":prompt,
                "prefix":prefix,"expected":expected,"answer_text":answer,"asked_start":null,"asked_end":null,"other_start":null,
                "forced_fact_source":null,"absent_name":null}));
        }
        write_rows(root, &format!("action-{split}-blind.json"), &bare)?;
        write_rows(root, &format!("action-{split}-grade.json"), &grade)?;
    }
    Ok(())
}
