use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, error::Error, fs, path::Path};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];
const HEADER: &str = "<|bos|>User: ";
const TOKENIZER_SHA: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const MAX_PREFIX: usize = 192 - 16;
const MAX_COPY: usize = 7;

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn append(text: &mut String, value: &str) -> Span {
    let start = HEADER.len() + text.len();
    text.push(' ');
    text.push_str(value);
    [start, HEADER.len() + text.len()]
}

fn token_span(boundaries: &[usize], [start, end]: Span) -> Result<Span> {
    let at = |byte| {
        boundaries
            .binary_search(&byte)
            .map_err(|_| format!("byte {byte} splits an original token"))
    };
    Ok([at(start)?, at(end)?])
}

fn class(piece: &[u8]) -> u8 {
    match piece {
        [b' ', head, rest @ ..]
            if head.is_ascii_alphabetic() && rest.iter().all(u8::is_ascii_alphabetic) =>
        {
            1
        }
        [head, rest @ ..]
            if head.is_ascii_alphabetic() && rest.iter().all(u8::is_ascii_alphabetic) =>
        {
            2
        }
        _ => 0,
    }
}

fn jsonl(path: &Path, rows: &[Value]) -> Result<()> {
    use std::io::{BufWriter, Write};
    let mut file = BufWriter::new(fs::File::create_new(path)?);
    for row in rows {
        serde_json::to_writer(&mut file, row)?;
        writeln!(file)?;
    }
    file.flush()?;
    Ok(())
}

fn verify_new_surfaces(cells: &str, controls: &str) -> Result<()> {
    let mut known = BTreeSet::new();
    for rows in [
        include_str!("train.tsv"),
        include_str!("held.tsv"),
        include_str!("controls.tsv"),
    ] {
        for line in rows.lines() {
            known.extend(line.split('\t').skip(1).map(str::to_ascii_lowercase));
        }
    }
    for line in cells.lines().chain(controls.lines()) {
        for term in line.split('\t').skip(1) {
            if known.contains(&term.to_ascii_lowercase()) {
                return Err(format!(
                    "external held surface duplicates a prior authored field: {term}"
                )
                .into());
            }
        }
    }
    if cells.trim().is_empty() {
        return Err("external held cell roster is empty".into());
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [split, tokenizer_path, out, extra @ ..] = args.as_slice() else {
        return Err("usage: d28-onset-panel train|held TOKENIZER NEW_OUTPUT_DIRECTORY [EXTERNAL_CELLS ROW_BASE [EXTERNAL_CONTROLS]]".into());
    };
    let (rows, row_base, controls, control_base) = match (split.as_str(), extra) {
        ("train", []) => (
            include_str!("train.tsv").to_owned(),
            28000,
            String::new(),
            0,
        ),
        ("held", []) => (
            include_str!("held.tsv").to_owned(),
            28100,
            include_str!("controls.tsv").to_owned(),
            28140,
        ),
        ("held", [cells, base, opt_control @ ..]) if opt_control.len() <= 1 => {
            let cells = fs::read_to_string(cells)?;
            let controls = opt_control
                .first()
                .map(fs::read_to_string)
                .transpose()?
                .unwrap_or_default();
            let base: usize = base.parse()?;
            if base < 28200
                || base > 29900
                || cells.lines().count() > 20
                || controls.lines().count() > 20
            {
                return Err(
                    "external held roster overlaps earlier row IDs or exceeds bounds".into(),
                );
            }
            verify_new_surfaces(&cells, &controls)?;
            (cells, base, controls, base + 80)
        }
        _ => return Err("invalid split or external held argument count".into()),
    };
    let frame = match (split.as_str(), extra.is_empty()) {
        ("train", _) => (
            "Read the next entry carefully.",
            "Repeat the complete item recorded for",
            ".",
        ),
        ("held", true) => (
            "One entry in the notebook reads as follows.",
            "What exact object wording appears beside",
            " in that entry?",
        ),
        ("held", false) => (
            "Here is a note from the ledger.",
            "According to the note, what is the complete item named for",
            "?",
        ),
        _ => return Err("invalid split".into()),
    };
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    if sha(&tokenizer_bytes) != TOKENIZER_SHA {
        return Err("tokenizer SHA differs from seed-1 D11".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut unique = BTreeSet::new();
    let (mut blind, mut public, mut grades, mut framing, mut persons, mut labeled) = (
        Vec::new(),
        String::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for (family, line) in rows.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [domain, person, short, long, first, second] = fields.as_slice() else {
            return Err("invalid authored row".into());
        };
        for (length, connector) in [*short, *long].into_iter().enumerate() {
            for (item, value) in [*first, *second].into_iter().enumerate() {
                let row = row_base + family * 4 + length * 2 + item;
                let mut text = frame.0.to_owned();
                let name = append(&mut text, person);
                let connector_span = append(&mut text, connector);
                let object = append(&mut text, value);
                text.push('.');
                let query = append(&mut text, frame.1);
                let requested = append(&mut text, person);
                text.push_str(frame.2);
                let user = Message {
                    role: "user",
                    content: &text,
                };
                let ids = encoder.encode_assistant_prefix(&[user.clone()]).tokens;
                if ids.first() != Some(&0)
                    || ids.len() > MAX_PREFIX
                    || !unique.insert(ids.clone())
                    || tokenizer.decode(&ids) != format!("{HEADER}{text}\nAssistant:")
                {
                    return Err(format!("row {row} ordinary framing mismatch").into());
                }
                let mut boundaries = vec![0];
                let mut classes = Vec::with_capacity(ids.len());
                for &id in &ids {
                    let piece = tokenizer.decode_bytes(&[id]);
                    boundaries.push(boundaries[boundaries.len() - 1] + piece.len());
                    classes.push(class(&piece));
                }
                let at = |s| token_span(&boundaries, s);
                let name = at(name)?;
                let connected = at(connector_span)?;
                let object = at(object)?;
                let requested = at(requested)?;
                let query = at([query[0], HEADER.len() + text.len()])?;
                let content = at([HEADER.len() - 1, HEADER.len() + text.len()])?;
                let depth = object[0] - name[1];
                let fence = object[1];
                if name[1] != connected[0]
                    || connected[1] != object[0]
                    || ids.get(fence) != Some(&16)
                    || ids[name[0]..name[1]] != ids[requested[0]..requested[1]]
                    || fence - object[0] > MAX_COPY
                    || (length == 0 && depth > 3)
                    || (length == 1 && depth <= 3)
                {
                    return Err(format!("row {row} invalid onset/crossing: {text}; {name:?} {connected:?} {object:?}").into());
                }
                let expected = [&ids[object[0]..fence], &[1]].concat();
                let document = encoder
                    .encode_document(&[
                        user,
                        Message {
                            role: "assistant",
                            content: value,
                        },
                    ])
                    .tokens;
                if document.strip_prefix(ids.as_slice()) != Some(expected.as_slice())
                    || tokenizer.decode_bytes(&ids[object[0]..fence])
                        != format!(" {value}").as_bytes()
                {
                    return Err(format!(
                        "row {row} original source cannot replay exact answer+EOS"
                    )
                    .into());
                }
                let mut candidates = Vec::new();
                for start in name[1] + 1..fence {
                    if classes[start] != 1 || fence - start > MAX_COPY {
                        continue;
                    }
                    let word_end = (start + 1..fence)
                        .find(|&end| classes[end] != 2)
                        .unwrap_or(fence);
                    let offset = start - name[1];
                    let profile_start = start - offset.min(3);
                    let profile = &classes[profile_start..word_end];
                    let candidate = json!({
                        "source": start, "word_end": word_end, "fence": fence,
                        "offset": offset, "legacy": offset <= 3,
                        "descriptor_depth": offset.min(3), "word_width": word_end - start,
                        "descriptor_classes": profile, "descriptor_ids": &ids[profile_start..word_end],
                        "copy_ids_through_eos": ([&ids[start..fence], &[1]].concat()),
                        "positive": start == object[0],
                    });
                    if split == "train" {
                        labeled.push(json!({"row":row, "candidate":candidate}));
                    }
                    candidates.push(candidate);
                }
                let wrong = candidates
                    .iter()
                    .find(|c| c["positive"] == false)
                    .ok_or_else(|| format!("row {row} lacks a competing source"))?;
                if candidates.iter().filter(|c| c["positive"] == true).count() != 1 {
                    return Err(format!("row {row} lacks unique reachable gold").into());
                }
                let prefix_sha = sha(&ids
                    .iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>());
                public.push_str(&format!("{row}\t{text}\n"));
                blind.push(json!({"row":row, "ids":ids}));
                framing.push(json!({"row":row, "content":content, "query":query}));
                persons.push(json!({"row":row, "requested":requested, "excluded":null}));
                grades.push(json!({
                    "row":row, "stratum":"matched", "family":domain, "connector_arm":length, "object_arm":item,
                    "requested":requested, "assertion_head":name, "fact_end":name[1]-1,
                    "query_start":requested[0], "connector":connected, "value":object,
                    "gold_source":object[0], "wrong_source":wrong["source"], "fence":fence,
                    "expected_value":value, "expected_tape":expected,
                    "wrong_tape":wrong["copy_ids_through_eos"], "prefix_sha256":prefix_sha,
                    "assertions":[{"person":name, "relation":connected, "value":object}],
                    "selected":[0], "selected_value_spans":[object],
                    "candidates":candidates,
                }));
            }
        }
    }
    if split == "held" {
        for (index, line) in controls.lines().enumerate() {
            let fields: Vec<_> = line.split('\t').collect();
            let [stratum, person, ask, a, av, b, bv] = fields.as_slice() else {
                return Err("invalid authored control".into());
            };
            let row = control_base + index;
            let (intro, query_text) = if extra.is_empty() {
                (
                    "The inventory contains these notes.",
                    "What exact item wording is listed for",
                )
            } else {
                (
                    "The ledger should list one item per person.",
                    "What single complete item wording do the entries agree on for",
                )
            };
            let mut text = intro.to_owned();
            let mut spans = Vec::new();
            for (connector, value) in [(*a, *av), (*b, *bv)] {
                spans.push([
                    append(&mut text, person),
                    append(&mut text, connector),
                    append(&mut text, value),
                ]);
                text.push('.');
            }
            let query_start = append(&mut text, query_text)[0];
            let requested = append(&mut text, ask);
            text.push('?');
            let user = Message {
                role: "user",
                content: &text,
            };
            let ids = encoder.encode_assistant_prefix(&[user.clone()]).tokens;
            if ids.first() != Some(&0)
                || ids.len() > MAX_PREFIX
                || !unique.insert(ids.clone())
                || tokenizer.decode(&ids) != format!("{HEADER}{text}\nAssistant:")
            {
                return Err(format!("control {row} ordinary framing mismatch").into());
            }
            let mut boundaries = vec![0];
            for &id in &ids {
                boundaries
                    .push(boundaries[boundaries.len() - 1] + tokenizer.decode_bytes(&[id]).len());
            }
            let at = |span| token_span(&boundaries, span);
            let requested = at(requested)?;
            let query = at([query_start, HEADER.len() + text.len()])?;
            let content = at([HEADER.len() - 1, HEADER.len() + text.len()])?;
            let facts = spans
                .into_iter()
                .map(|[name, connector, value]| {
                    let value = at(value)?;
                    if ids.get(value[1]) != Some(&16)
                        || value[1] - value[0] > MAX_COPY
                        || class(&tokenizer.decode_bytes(&[ids[value[0]]])) != 1
                    {
                        return Err(format!("control {row} has unreachable value").into());
                    }
                    Ok(json!({"person":at(name)?, "relation":at(connector)?, "value":value}))
                })
                .collect::<Result<Vec<_>>>()?;
            let selected = if *stratum == "absent" {
                vec![]
            } else {
                vec![0, 1]
            };
            let values = selected
                .iter()
                .map(|&i| {
                    let span = &facts[i]["value"];
                    Ok([
                        span[0].as_u64().ok_or("missing start")? as usize,
                        span[1].as_u64().ok_or("missing end")? as usize,
                    ])
                })
                .collect::<Result<Vec<Span>>>()?;
            let expected = if *stratum == "agreement" {
                let [first, second] = values.as_slice() else {
                    return Err("agreement requires two sources".into());
                };
                let tape = [&ids[first[0]..first[1]], &[1]].concat();
                let document = encoder
                    .encode_document(&[
                        user,
                        Message {
                            role: "assistant",
                            content: av,
                        },
                    ])
                    .tokens;
                if ids[first[0]..first[1]] != ids[second[0]..second[1]]
                    || document.strip_prefix(ids.as_slice()) != Some(tape.as_slice())
                {
                    return Err(format!("control {row} lacks verbatim agreement").into());
                }
                Some(tape)
            } else {
                None
            };
            if *stratum == "conflict" && av == bv || *stratum == "absent" && person == ask {
                return Err("invalid control semantics".into());
            }
            let prefix_sha = sha(&ids
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect::<Vec<_>>());
            public.push_str(&format!("{row}\t{text}\n"));
            blind.push(json!({"row":row, "ids":ids}));
            framing.push(json!({"row":row, "content":content, "query":query}));
            persons.push(json!({"row":row, "requested":requested, "excluded":null}));
            grades.push(json!({
                "row":row, "stratum":stratum, "requested":requested, "assertions":facts,
                "selected":selected, "selected_value_spans":values, "expected_tape":expected,
                "expected_value":expected.as_ref().map(|_| av), "prefix_sha256":prefix_sha,
            }));
        }
    }
    let out = Path::new(out);
    fs::create_dir(out)?;
    fs::write(out.join(format!("{split}.blind.tsv")), public)?;
    for (kind, records) in [
        ("blind", &blind),
        ("framing", &framing),
        ("person", &persons),
        ("gold", &grades),
    ] {
        jsonl(&out.join(format!("{split}.{kind}.jsonl")), records)?;
    }
    if split == "train" {
        jsonl(&out.join("train.exemplars.jsonl"), &labeled)?;
    }
    println!(
        "{split}: {} prompts, {} supervised TRAIN candidate records",
        blind.len(),
        labeled.len()
    );
    Ok(())
}
