//! Tokenizes independently authored labels without a D11 model or selector.
mod author;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
mod seal;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];
const HEAD: &str = "<|bos|>User: ";
const TAIL: &str = "\nAssistant:";
const PINS: [&str; 2] = [
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
    "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
];

fn sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn save(path: &Path, rows: &[Value]) -> Result<()> {
    let mut handle = BufWriter::new(File::options().create_new(true).write(true).open(path)?);
    serde_json::to_writer_pretty(&mut handle, rows)?;
    writeln!(handle)?;
    handle.flush()?;
    Ok(())
}

fn span(
    t: &ByteBpeTokenizer,
    ids: &[u32],
    offsets: &[usize],
    byte: usize,
    surface: &str,
) -> Result<Span> {
    let start = offsets
        .binary_search(&byte.checked_sub(1).ok_or("missing separator")?)
        .map_err(|_| format!("unaligned start for {surface} at {byte}"))?;
    let end = offsets
        .binary_search(&(byte + surface.len()))
        .map_err(|_| format!("unaligned end for {surface} at {byte}"))?;
    if start >= end || t.decode_bytes(&ids[start..end]) != format!(" {surface}").as_bytes() {
        return Err(format!("verbatim span mismatch for {surface}").into());
    }
    Ok([start, end])
}

fn class(tokenizer: &ByteBpeTokenizer, id: u32) -> u8 {
    let bytes = tokenizer.decode_bytes(&[id]);
    if bytes.is_empty() {
        return 0;
    }
    if bytes[0] == b' ' && bytes.len() > 1 && bytes[1..].iter().all(u8::is_ascii_alphabetic) {
        1
    } else if bytes.iter().all(u8::is_ascii_alphabetic) {
        2
    } else {
        0
    }
}

fn complete(classes: &[u8], [a, b]: Span) -> bool {
    a < b
        && classes[a] == 1
        && classes[a + 1..b].iter().all(|&x| x == 2)
        && classes.get(b) != Some(&2)
}

fn expected_slice(ids: &[u32], a: usize, b: usize) -> Vec<u32> {
    [&ids[a..b], &[1]].concat()
}

fn run_held(t: &ByteBpeTokenizer, path: &Path) -> Result<()> {
    let protocol = DialogueProtocol::literal_roles_v2(t)?;
    let encoder = protocol.bind(t)?;
    let (mut blind, mut grades, mut bounds, mut candidates, mut public, mut resolved) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let mut seen = BTreeSet::new();
    for c in author::make_cases() {
        let (text, byte_anchors, question_byte) = c.surface(HEAD.len());
        let user = Message {
            role: "user",
            content: &text,
        };
        let ids = encoder.encode_assistant_prefix(&[user.clone()]).tokens;
        if !seen.insert(ids.clone())
            || ids.first() != Some(&0)
            || ids.len() + 16 > 192
            || ids.iter().skip(1).any(|&x| !(3..4096).contains(&x))
            || t.decode(&ids) != format!("{HEAD}{text}{TAIL}")
        {
            return Err(format!("ordinary text/framing failed in {}", c.id).into());
        }
        let mut offsets = vec![0];
        for &id in &ids {
            offsets.push(offsets[offsets.len() - 1] + t.decode_bytes(&[id]).len());
        }
        let classes: Vec<_> = ids.iter().map(|&id| class(t, id)).collect();
        let locate = |byte, s| span(t, &ids, &offsets, byte, s);
        let question = locate(question_byte, &c.question)?;
        let user_bounds = locate(HEAD.len(), &text)?;
        let requested = c
            .name
            .map(|name| locate(question_byte + c.query_name_byte, name))
            .transpose()?;
        let excluded = c
            .excluded_name
            .map(|name| {
                let byte = c.question.find(name).ok_or("missing excluded name")?;
                locate(question_byte + byte, name)
            })
            .transpose()?;
        if requested
            .into_iter()
            .chain(excluded)
            .any(|x| !complete(&classes, x) || x[0] < question[0] || x[1] > question[1])
            || (requested == excluded && requested.is_some())
        {
            return Err(format!("incomplete requested name in {}", c.id).into());
        }
        let mut fact_info = Vec::new();
        let mut any = Vec::new();
        for (i, (fact, [name, verb, item])) in c.facts.iter().zip(byte_anchors).enumerate() {
            let name_span = locate(name, fact.name)?;
            let connector_span = locate(verb, fact.connector)?;
            let value_span = locate(item, fact.item)?;
            let fence = value_span[1];
            let whole_name = complete(&classes, name_span);
            let [start, end] = name_span;
            if !whole_name
                || ![16, 28].contains(
                    ids.get(start.wrapping_sub(1))
                        .ok_or("missing earlier delimiter")?,
                )
                || end != connector_span[0]
                || connector_span[1] != value_span[0]
                || ids.get(fence) != Some(&16)
                || classes[value_span[0]] != 1
                || fence - value_span[0] > 7
            {
                return Err(
                    format!("invalid independently labelled literal in {}: {text}; {name_span:?} {connector_span:?} {value_span:?} class={whole_name} value_class={} preceding={} fence={} ids={ids:?}", c.id, classes[value_span[0]], ids[start - 1], ids[fence]).into(),
                );
            }
            let mut legal = Vec::new();
            for offset in 1..=3 {
                let s = end + offset;
                if s >= fence || classes[s] != 1 || fence - s > 7 {
                    continue;
                }
                let first_word_end = (s + 1..fence).find(|&x| classes[x] != 2).unwrap_or(fence);
                let words = &ids[s..first_word_end];
                let emitted = expected_slice(&ids, s, fence);
                legal.push(
                    json!({"offset":offset,"start":s,"word_span":[s,first_word_end],
                    "first_word":t.decode(words),"first_word_ids":words,
                    "entire_copy":t.decode(&ids[s..fence]),"copy_ids_through_eos":emitted}),
                );
            }
            let width = value_span[0] - end;
            let full = encoder
                .encode_document(&[
                    user.clone(),
                    Message {
                        role: "assistant",
                        content: fact.item,
                    },
                ])
                .tokens;
            if full.strip_prefix(ids.as_slice())
                != Some(expected_slice(&ids, value_span[0], value_span[1]).as_slice())
            {
                return Err(format!("non-verbatim literal for {} / {}", c.id, fact.name).into());
            }
            if c.stratum == "supported"
                && (!legal.iter().any(|r| r["start"] == value_span[0])
                    || legal
                        .iter()
                        .map(|r| r["entire_copy"].as_str())
                        .collect::<BTreeSet<_>>()
                        .len()
                        != legal.len())
            {
                return Err(format!("{} / {}: width {width}, legal {:?}, expected both facts to have a reachable gold and distinct legal replies", c.id,fact.item,
                    legal.iter().map(|r|r["offset"].clone()).collect::<Vec<_>>()).into());
            }
            if Some(fact.name) == c.name {
                let q = requested.ok_or("missing explicit role")?;
                if ids[start..end] != ids[q[0]..q[1]] {
                    return Err(format!("name identity fails in {}", c.id).into());
                }
                any.push((i, fact, value_span, width));
            }
            fact_info.push(
                json!({"name":fact.name,"name_span":name_span,"verb_phrase":fact.connector,
                "connector_span":connector_span,"value":fact.item,"value_span":value_span,
                "delimiter":[fence,ids[fence]],"gold_offset":width,"legal":legal}),
            );
        }
        let (answer, expected, start, assertion_index, gold_offset) =
            match (c.stratum, any.as_slice()) {
                ("supported", [(i, fact, [a, b], width)]) => (
                    Some(fact.item),
                    Some(expected_slice(&ids, *a, *b)),
                    Some(*a),
                    Some(*i),
                    Some(*width),
                ),
                ("conflict", [(_, a, _, _), (_, b, _, _)]) if a.item != b.item => {
                    (None, None, None, None, None)
                }
                ("unmentioned" | "unspecified", []) => (None, None, None, None, None),
                _ => return Err(format!("authored support relation differs on {}", c.id).into()),
            };
        let refusal = match c.stratum {
            "conflict" => Some(
                "The two competing descriptions disagree, so one exact item cannot be identified.",
            ),
            "unmentioned" => Some(
                "The records omit the requested person. A relevant no or uncertainty earns credit.",
            ),
            "unspecified" => Some(
                "The requested person is unspecified. Ask which person or state the ambiguity.",
            ),
            _ => None,
        };
        let prefix_sha = sha(&ids.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>());
        let visible_roles: Vec<_> = c
            .name
            .zip(requested)
            .into_iter()
            .chain(c.excluded_name.zip(excluded))
            .collect();
        // This is the input to the strongest reported lexical-ID-blind
        // control. It includes all token classes and absolute coordinates.
        // It has no semantic connector boundary, answer or token identity.
        let shape = json!({
            "prefix_len":ids.len(), "token_classes":classes,
            "question_bounds":question, "supplied_requested_span":requested,
            "same_name_fact_indices":any.iter().map(|(i,_,_,_)|i).collect::<Vec<_>>(),
            "facts":fact_info.iter().map(|f|json!({
                "name_span":f["name_span"], "delimiter":f["delimiter"],
                "starts":f["legal"].as_array().into_iter().flatten().map(|r|json!({
                    "offset":r["offset"],"start":r["start"],"span":r["word_span"],
                    "completion_len":r["copy_ids_through_eos"].as_array().map(Vec::len),
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>(),
        });
        public.push(json!({"index":blind.len(),"text":text}));
        resolved
            .push(json!({"index":blind.len(),"prefix_sha256":prefix_sha,"requested":requested}));
        blind.push(json!({"prefix":ids,"text":text}));
        bounds.push(json!({"start":question[0],"end":question[1]}));
        let assigned_candidates = assertion_index.map(|i| &fact_info[i]["legal"]);
        let alternatives = assigned_candidates.map(|rows| {
            rows.as_array()
                .into_iter()
                .flatten()
                .filter(|r| r["start"] != json!(start))
                .collect::<Vec<_>>()
        });
        candidates.push(json!({"id":c.id,"stratum":c.stratum,"gold_start":start,"gold_offset":gold_offset,
            "matched_triple":null,"written_value_words":c.written_value_words,"nonlexical_observation":shape,
            "gold_candidate_present":assigned_candidates.map(|m| m.as_array().into_iter().flatten().any(|r|r["start"]==json!(start))),
            "all_requested_facts":any.iter().map(|(i,_,_,_)|i).collect::<Vec<_>>(),
            "facts":fact_info,"pre_inference":true}));
        grades.push(json!({"id":c.id,"family":c.family,"pair":c.pair,"direction":c.direction,
            "form":c.form,"stratum":c.stratum,"requested":requested,"requested_name":c.name,"excluded":excluded,"excluded_name":c.excluded_name,
            "object_swap":c.object_swap,"connector_swap":c.connector_swap,"the":null,"about":null,"reversed":c.reversed,"distractor_first":c.distractor_first,"name_order_reversed":c.name_order_reversed,
            "matched_triple":null,"written_value_words":c.written_value_words,
            "question_mentions":visible_roles,"expected":expected,"answer_text":answer,
            "gold_start":start,"gold_assertion_index":assertion_index,"gold_offset":gold_offset,
            "rival_start":alternatives.as_ref().and_then(|v|v.first()).map(|r|r["start"].clone()),
            "alternative_candidates":alternatives,"facts":fact_info,"prose_acceptable":refusal,
            "question_bounds":question,"user_bounds":user_bounds,"prefix_sha256":prefix_sha}));
    }
    fs::create_dir(path)?;
    for (part, rows) in [
        ("blind", &blind),
        ("grades", &grades),
        ("bounds", &bounds),
        ("candidates", &candidates),
        ("public-text", &public),
        ("resolved", &resolved),
    ] {
        save(&path.join(format!("{part}.json")), rows)?;
    }
    println!("held: {} exact protocol-2 rows", blind.len());
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let [mode, output] = args.as_slice() {
        if mode == "raw" {
            let rows: Vec<_> = author::make_cases()
                .iter()
                .enumerate()
                .map(|(index, case)| json!({"index": index, "text": case.surface(HEAD.len()).0}))
                .collect();
            return save(Path::new(output), &rows);
        }
    }
    if let [mode, raw, people, out] = args.as_slice() {
        if mode == "seal" {
            return seal::seal(Path::new(raw), Path::new(people), Path::new(out));
        }
    }
    let [seed, input, out] = args.as_slice() else {
        return Err("user-text-role-panel SEED TOKENIZER NEW_ROOT".into());
    };
    let index = match seed.as_str() {
        "1" => 0,
        "2" => 1,
        _ => return Err("seed must be 1 or 2".into()),
    };
    let data = fs::read(input)?;
    if sha(&data) != PINS[index] {
        return Err("pinned tokenizer changed".into());
    }
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&data).ok_or("bad tokenizer")?;
    let out = Path::new(out);
    fs::create_dir(out)?;
    run_held(&t, &out.join("held"))
}
