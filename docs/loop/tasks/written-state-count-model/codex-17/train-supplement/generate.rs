// Static TRAIN custody tool. It binds tokenizer bytes without loading D11 or a predictor.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];
const HEAD: &str = "<|bos|>User: ";
const TAIL: &str = "\nAssistant:";
const HASHES: [&str; 2] = [
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

fn question(family: &str) -> Result<[&'static str; 2]> {
    Ok(match family {
        "sale" => [
            "What is the complete goods description on the line for ",
            "?",
        ],
        "wash" => ["Give me the exact object wording beside ", "'s name."],
        "organize" => ["Which full description appears in the entry for ", "?"],
        "shine" => ["Read back the recorded item for ", " in full."],
        "pricing" => ["Which complete object phrase appears for ", "?"],
        "trim" => ["Copy the precise item description given for ", "."],
        "stack" => [
            "According to this line, what exact object is listed for ",
            "?",
        ],
        "repair" => [
            "I need the full wording of the object in the note about ",
            ".",
        ],
        _ => return Err("unknown authored wording family".into()),
    })
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [seed, tok, out] = args.as_slice() else {
        return Err("usage: generate SEED TOKENIZER NEW_ROOT".into());
    };
    let idx = match seed.as_str() {
        "1" => 0,
        "2" => 1,
        _ => return Err("seed must be 1 or 2".into()),
    };
    let bytes = fs::read(tok)?;
    if sha(&bytes) != HASHES[idx] {
        return Err("stale frozen tokenizer".into());
    }
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&t)?;
    let enc = protocol.bind(&t)?;
    let mut checked = BTreeSet::new();
    let (mut blind, mut grades, mut candidates, mut public, mut bounds) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (index, line) in include_str!("authored.tsv").lines().enumerate() {
        let id = format!("supplement-{index:02}");
        let cols: Vec<_> = line.split('\t').collect();
        let [family, person, verb, literal, depth, width] = cols.as_slice() else {
            return Err(format!("wrong schema {id}").into());
        };
        let depth: usize = depth.parse()?;
        let width: usize = width.parse()?;
        let [pre, suf] = question(family)?;
        let fact = format!("{person} {verb} {literal}.");
        let q = format!("{pre}{person}{suf}");
        let text = format!("{fact} {q}");
        let m = Message {
            role: "user",
            content: &text,
        };
        let ids = enc.encode_assistant_prefix(&[m.clone()]).tokens;
        if !checked.insert(ids.clone())
            || ids.first() != Some(&0)
            || ids.len() + 16 > 192
            || t.decode(&ids) != format!("{HEAD}{text}{TAIL}")
        {
            return Err(format!("ordinary framing failed on {id}").into());
        }
        let mut offsets = vec![0];
        for &id in &ids {
            offsets.push(offsets[offsets.len() - 1] + t.decode_bytes(&[id]).len());
        }
        let classes: Vec<_> = ids.iter().map(|&id| class(&t, id)).collect();
        let locate = |at, s| span(&t, &ids, &offsets, at, s);
        let len = HEAD.len();
        let name = locate(len, person)?;
        let connector = locate(len + person.len() + 1, verb)?;
        let value = locate(len + person.len() + verb.len() + 2, literal)?;
        let qbyte = len + fact.len() + 1;
        let question_span = locate(qbyte, &q)?;
        let requested = locate(qbyte + pre.len(), person)?;
        if !complete(&classes, name)
            || !complete(&classes, requested)
            || ids[name[0]..name[1]] != ids[requested[0]..requested[1]]
            || ![16, 28].contains(&ids[name[0] - 1])
            || connector[0] != name[1]
            || connector[1] != value[0]
            || ids.get(value[1]) != Some(&16)
            || value[0] - name[1] != depth
            || value[1] - value[0] > 7
        {
            return Err(format!(
                "unaligned authored assertion {id}: {text} ({name:?}/{connector:?}/{value:?})"
            )
            .into());
        }
        let mut legal = Vec::new();
        let fence = value[1];
        for d in 1..=3 {
            let start = name[1] + d;
            if start >= fence || classes[start] != 1 || fence - start > 7 {
                continue;
            }
            let end = (start + 1..fence)
                .find(|&j| classes[j] != 2)
                .unwrap_or(fence);
            let word = &ids[start..end];
            let shape = json!([d, word.len(), classes[name[1]..end]]);
            legal.push(json!({"offset":d,"start":start,"word_span":[start,end],"first_word":t.decode(word),
                "first_word_ids":word,"entire_copy":t.decode(&ids[start..fence]),"copy_ids_through_eos":expected_slice(&ids,start,fence),"shape":shape}));
        }
        if !legal.iter().any(|r| {
            r["start"] == value[0] && r["first_word_ids"].as_array().map(Vec::len) == Some(width)
        }) {
            return Err(format!(
                "authored width or source outside a legal word at {id}: {text}; {legal:?}"
            )
            .into());
        }
        let full = enc
            .encode_document(&[
                m,
                Message {
                    role: "assistant",
                    content: literal,
                },
            ])
            .tokens;
        let expected = full
            .strip_prefix(ids.as_slice())
            .ok_or("assistant changed original prefix")?;
        if expected != expected_slice(&ids, value[0], fence) {
            return Err(format!("non-verbatim literal {id}").into());
        }
        let support = json!({"name":person,"name_span":name,"verb_phrase":verb,"connector_span":connector,"value":literal,"value_span":value,"delimiter":[fence,16],"legal":legal});
        let shape_facts = vec![json!({"name_span":name,"delimiter":[fence,16],
            "starts":legal.iter().map(|r|json!({"offset":r["offset"],"start":r["start"],"span":r["word_span"]})).collect::<Vec<_>>()})];
        let digest = sha(&ids.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>());
        grades.push(json!({"id":id,"family":family,"pair":id,"direction":0,"stratum":"supported","expected":expected,
            "requested":requested,"requested_name":person,"excluded":null,"question_mentions":[[person,&requested]],
            "gold_start":value[0],"gold_offset":depth,"gold_assertion_index":0,"facts":[support],"answer_text":literal,
            "question_bounds":question_span,"prefix_sha256":digest}));
        candidates.push(json!({"id":id,"stratum":"supported","gold_candidate_present":true,"gold_start":value[0],
            "all_requested_facts":[0],"facts":[support],"nonlexical_observation":{
                 "token_classes":classes,"same_name_fact_indices":[0],"facts":shape_facts}}));
        bounds.push(json!({"start":question_span[0],"end":question_span[1]}));
        public.push(json!({"index":index,"text":text}));
        blind.push(json!({"text":text,"prefix":ids}));
    }
    fs::create_dir(out)?;
    let out = Path::new(out);
    for (name, rows) in [
        ("blind", &blind),
        ("grades", &grades),
        ("candidates", &candidates),
        ("bounds", &bounds),
        ("public-text", &public),
    ] {
        save(&out.join(format!("{name}.json")), rows)?;
    }
    println!(
        "{} independently labelled supplement TRAIN rows, {} raw candidate slots",
        blind.len(),
        candidates
            .iter()
            .map(|r| r["facts"][0]["legal"].as_array().map_or(0, Vec::len))
            .sum::<usize>()
    );
    Ok(())
}
