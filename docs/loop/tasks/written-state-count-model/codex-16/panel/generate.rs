//! Framing and gold grade authoring only. No D11 assets or model calls.
mod author;

use author::{
    Fact, ABSENT, ABSENT_ASK, CONFLICT_ALTERNATIVES, CONFLICT_ASK, FACTORIAL, STRUCTURED,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];
const HEAD: &str = "<|bos|>User: ";
const TAIL: &str = "\nAssistant:";
const TOKENIZERS: [&str; 2] = [
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
    "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
];

struct Case {
    id: String,
    family: &'static str,
    pair: String,
    direction: usize,
    stratum: &'static str,
    form: &'static str,
    facts: Vec<Fact>,
    person: &'static str,
    ask: [&'static str; 2],
}

fn cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (f, family) in FACTORIAL.iter().enumerate() {
        for form in 0..2 {
            for swap in 0..2 {
                let mut facts = [0, 1].map(|i| {
                    let (literal, literal_head) = family.items[i ^ swap];
                    Fact {
                        person: family.people[i],
                        verb: family.verbs[form],
                        literal,
                        literal_head,
                    }
                });
                if (f + swap) % 2 != 0 {
                    facts.reverse();
                }
                for direction in 0..2 {
                    let pair = format!("{}-{form}-{swap}", family.key);
                    rows.push(Case {
                        id: format!("{pair}-d{direction}"),
                        pair,
                        family: family.key,
                        direction,
                        stratum: "factorial",
                        form: if form == 0 { "short" } else { "long" },
                        facts: facts.to_vec(),
                        person: family.people[direction],
                        ask: family.query,
                    });
                }
            }
        }
    }
    for (i, pair) in STRUCTURED.iter().enumerate() {
        let facts = vec![pair.first, pair.second];
        for (direction, fact) in facts.iter().enumerate() {
            rows.push(Case {
                id: format!("{}-d{direction}", pair.key),
                pair: pair.key.into(),
                family: pair.key,
                direction,
                stratum: "structured-positive",
                form: if fact.verb.contains(' ') {
                    "long"
                } else {
                    "short"
                },
                facts: facts.clone(),
                person: fact.person,
                ask: pair.ask,
            });
        }
        rows.push(Case {
            id: format!("{}-absent", pair.key),
            pair: pair.key.into(),
            family: pair.key,
            direction: 0,
            stratum: "absent",
            form: "unknown",
            facts: facts.clone(),
            person: ABSENT[i],
            ask: ABSENT_ASK,
        });
        let (literal, literal_head) = CONFLICT_ALTERNATIVES[i];
        let mut conflicting = facts;
        conflicting.push(Fact {
            literal,
            literal_head,
            ..pair.first
        });
        rows.push(Case {
            id: format!("{}-conflict", pair.key),
            pair: pair.key.into(),
            family: pair.key,
            direction: 0,
            stratum: "conflict",
            form: "ambiguous",
            facts: conflicting,
            person: pair.first.person,
            ask: CONFLICT_ASK,
        });
    }
    rows
}

fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn expect_span(
    t: &ByteBpeTokenizer,
    ids: &[u32],
    offsets: &[usize],
    start: usize,
    s: &str,
) -> Result<Span> {
    let left = start.checked_sub(1).ok_or("span requires leading blank")?;
    let a = offsets
        .binary_search(&left)
        .map_err(|_| format!("unaligned start for {s} at {start}"))?;
    let b = offsets
        .binary_search(&(start + s.len()))
        .map_err(|_| format!("unaligned end for {s} at {start}"))?;
    if t.decode_bytes(&ids[a..b]) != format!(" {s}").as_bytes() || a >= b {
        return Err(format!("unfaithful token slice for {s}").into());
    }
    Ok([a, b])
}

fn whole(t: &ByteBpeTokenizer, ids: &[u32], span: Span) -> bool {
    let first = t.decode_bytes(&ids[span[0]..span[0] + 1]);
    first.first() == Some(&b' ')
        && first[1..].iter().all(u8::is_ascii_alphabetic)
        && first.len() > 1
        && ids[span[0] + 1..span[1]].iter().all(|id| {
            let piece = t.decode_bytes(&[*id]);
            !piece.is_empty() && piece.iter().all(u8::is_ascii_alphabetic)
        })
        && ids.get(span[1]).is_none_or(|next| {
            let piece = t.decode_bytes(&[*next]);
            piece.is_empty() || !piece.iter().all(u8::is_ascii_alphabetic)
        })
}

fn write(path: &Path, rows: &[Value]) -> Result<()> {
    let mut file = BufWriter::new(File::options().create_new(true).write(true).open(path)?);
    serde_json::to_writer_pretty(&mut file, rows)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [seed, tokenizer, out] = args.as_slice() else {
        return Err("generate SEED TOKENIZER OUT_DIR".into());
    };
    let seed_index = match seed.as_str() {
        "1" => 0,
        "2" => 1,
        _ => return Err("unknown frozen seed".into()),
    };
    let bytes = fs::read(tokenizer)?;
    if digest(&bytes) != TOKENIZERS[seed_index] {
        return Err("pinned tokenizer differs".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut seen = BTreeSet::new();
    let mut blind = Vec::new();
    let mut grades = Vec::new();
    let mut census = Vec::new();
    for c in cases() {
        let mut text = String::new();
        let mut authored_offsets = Vec::new();
        for fact in &c.facts {
            if !text.is_empty() {
                text.push(' ');
            }
            let name_byte = HEAD.len() + text.len();
            text.push_str(fact.person);
            text.push(' ');
            let verb_byte = HEAD.len() + text.len();
            text.push_str(fact.verb);
            text.push(' ');
            let value_byte = HEAD.len() + text.len();
            text.push_str(fact.literal);
            text.push('.');
            authored_offsets.push([name_byte, verb_byte, value_byte]);
        }
        text.push(' ');
        let bound_byte = HEAD.len() + text.len();
        let question = format!("{}{}{}", c.ask[0], c.person, c.ask[1]);
        text.push_str(&question);
        let message = Message {
            role: "user",
            content: &text,
        };
        let prefix = encoder.encode_assistant_prefix(&[message.clone()]).tokens;
        if !seen.insert(prefix.clone())
            || prefix.first() != Some(&0)
            || prefix[1..].contains(&0)
            || prefix[1..].contains(&1)
            || prefix[1..].contains(&2)
            || prefix.len() + 16 > 192
            || tokenizer.decode(&prefix) != format!("{HEAD}{text}{TAIL}")
        {
            return Err(format!("invalid ordinary prompt {}", c.id).into());
        }
        let mut offsets = vec![0];
        for &id in &prefix {
            offsets.push(offsets[offsets.len() - 1] + tokenizer.decode_bytes(&[id]).len());
        }
        let pos = |at, s| expect_span(&tokenizer, &prefix, &offsets, at, s);
        let requested = pos(bound_byte + c.ask[0].len(), c.person)?;
        let question_range = pos(bound_byte, &question)?;
        if !whole(&tokenizer, &prefix, requested)
            || requested[0] < question_range[0]
            || requested[1] > question_range[1]
        {
            return Err(format!("request is not a complete query word in {}", c.id).into());
        }
        let mut facts = Vec::new();
        let mut actual = Vec::new();
        for (i, (fact, [at, verb, val])) in c.facts.iter().zip(authored_offsets).enumerate() {
            let name = pos(at, fact.person)?;
            let bridge = pos(verb, fact.verb)?;
            let value = pos(val, fact.literal)?;
            let mut value_words = fact.literal.split_ascii_whitespace();
            let first_word = value_words.next().ok_or("empty literal")?;
            let first_span = pos(val, first_word)?;
            let (rival_word, rival) = if let Some((before, tail)) = fact.verb.rsplit_once(' ') {
                (tail, pos(verb + before.len() + 1, tail)?)
            } else {
                let before = fact
                    .literal
                    .strip_suffix(fact.literal_head)
                    .ok_or("literal head missing")?;
                (
                    fact.literal_head,
                    pos(val + before.len(), fact.literal_head)?,
                )
            };
            let stop = prefix.get(value[1]).ok_or("missing assertion terminal")?;
            if !whole(&tokenizer, &prefix, name)
                || !whole(&tokenizer, &prefix, rival)
                || !whole(&tokenizer, &prefix, first_span)
                || rival[0] == value[0]
                || tokenizer.decode_bytes(&[*stop]) != b"."
                || bridge[0] != name[1]
                || bridge[1] != value[0]
                || value[1] - value[0] > 7
                || (fact.person == c.person
                    && prefix[name[0]..name[1]] != prefix[requested[0]..requested[1]])
            {
                return Err(format!("misaligned whole-word assertion in {} ({text})", c.id).into());
            }
            let verb_len = bridge[1] - bridge[0];
            let range = if (1..=3).contains(&verb_len) {
                "inside-legacy-1-to-3-connector-id-window"
            } else {
                "outside-legacy-connector-window"
            };
            if c.stratum == "factorial"
                && ((c.form == "short" && verb_len != 1)
                    || (c.form == "long" && !(2..=3).contains(&verb_len)))
            {
                return Err(format!(
                    "factorial connector token width differs in {}: {verb_len}",
                    c.id
                )
                .into());
            }
            let explanation = if fact.verb.contains(' ') {
                format!("“{rival_word}” completes the action “{}”. The object reported is the entire phrase “{}”.",fact.verb,fact.literal)
            } else {
                format!("“{}” is the full action. “{rival_word}” alone drops the modifier in the recorded object “{}”.",fact.verb,fact.literal)
            };
            let mut copy = prefix[value[0]..value[1]].to_vec();
            copy.push(1);
            let mut alternate = prefix[rival[0]..value[1]].to_vec();
            alternate.push(1);
            facts.push(json!({"name":fact.person,"name_span":name,"verb_phrase":fact.verb,"connector_span":bridge,
                 "connector_ids":&prefix[bridge[0]..bridge[1]],"connector_range":range,
                 "value":fact.literal,"value_span":value,"value_first_word_span":first_span,
                 "value_first_word_ids":&prefix[first_span[0]..first_span[1]],"value_head":fact.literal_head,
                 "rival_word":rival_word,"rival_span":rival,"rival_ids":&prefix[rival[0]..rival[1]],
                 "literal_ids_through_eos":copy,"rival_ids_through_eos":alternate,
                 "boundary_explanation":explanation,"delimiter":[value[1],*stop]}));
            if fact.person == c.person {
                actual.push((i, fact, value, rival));
            }
        }
        let (expected, gold, rival, answer) = match (c.stratum, actual.as_slice()) {
            ("absent", []) => (None, None, None, None),
            ("conflict", [(_, a, _, _), (_, b, _, _)]) if a.literal != b.literal => {
                (None, None, None, None)
            }
            ("factorial" | "structured-positive", [(i, fact, [start, end], other)]) => {
                let full = encoder
                    .encode_document(&[
                        message,
                        Message {
                            role: "assistant",
                            content: fact.literal,
                        },
                    ])
                    .tokens;
                let replay = full
                    .strip_prefix(prefix.as_slice())
                    .ok_or("encoded assistant document changed prefix")?;
                if replay != [&prefix[*start..*end], &[1]].concat() || replay.len() < 2 {
                    return Err(
                        format!("literal is not exact original IDs then EOS in {}", c.id).into(),
                    );
                }
                (
                    Some(replay.to_vec()),
                    Some((i, start, end)),
                    Some(other),
                    Some(fact.literal),
                )
            }
            _ => {
                return Err(format!("fact support disagrees with authored stratum {}", c.id).into())
            }
        };
        let refusal=match c.stratum {
            "absent"=>Some("These statements give no item for the explicitly asked person."),
            "conflict"=>Some("The question explicitly identifies incompatible entries for the same person and requires uncertainty."),
            _=>None,
        };
        let positive_fact = gold.map(|(i, _, _)| facts[*i].clone());
        census.push(
            json!({"id":c.id,"form":c.form,"stratum":c.stratum,"question":question,
           "reported_connector":positive_fact.as_ref().map(|r|r["connector_ids"].clone()),
           "leading_value_word":positive_fact.as_ref().map(|r|r["value_first_word_ids"].clone()),
           "connector_range":positive_fact.as_ref().map(|r|r["connector_range"].clone()),
           "prior_position":gold.map(|(i,_,_)|i),"tape_length":prefix.len()}),
        );
        grades.push(json!({"id":c.id,"family":c.family,"pair":c.pair,"direction":c.direction,"form":c.form,
            "stratum":match c.stratum {"factorial"|"structured-positive"=>"supported","absent"=>"unmentioned",other=>other}, "panel_stratum":c.stratum, "requested":requested,"requested_name":c.person,"excluded":null,
            "question_mentions":[[c.person,requested]],
            "expected":expected,"answer_text":answer,"gold_assertion_index":gold.map(|(i,_,_)|i),
            "gold_start":gold.map(|(_,start,_)|start),"gold_value_span":gold.map(|(_,start,end)|[*start,*end]),
            "rival_span":rival,"rival_start":rival.map(|s|s[0]),"question_bounds":question_range,
            "supporting_facts":actual.iter().map(|(i,_,_,_)|i).collect::<Vec<_>>(),
            "facts":facts,"prose_acceptable":refusal,
            "prefix_sha256":digest(&prefix.iter().flat_map(|id|id.to_le_bytes()).collect::<Vec<_>>())}));
        blind.push(json!({"prefix":prefix,"text":text}));
    }
    let out = Path::new(out);
    fs::create_dir(out)?;
    write(&out.join("held-blind.json"), &blind)?;
    write(&out.join("held-grades.json"), &grades)?;
    write(&out.join("held-census.json"), &census)?;
    println!("seed {seed} {} independently framed prompts", blind.len());
    Ok(())
}
