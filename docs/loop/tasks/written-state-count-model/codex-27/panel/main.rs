use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::{env, error::Error, fs, path::Path};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const PREFIX: &str = "<|bos|>User: ";
const PRELUDE: &str = "The workshop keeps a written inventory.";

#[derive(Clone, Copy)]
struct Fact<'a> {
    person: &'a str,
    cue: &'a str,
    value: &'a str,
}

struct Case<'a> {
    stratum: &'a str,
    person: &'a str,
    relation: &'a str,
    excluded: Option<&'a str>,
    facts: Vec<Fact<'a>>,
    selected: Vec<usize>,
    factors: Option<[usize; 4]>,
}

fn cases() -> Result<Vec<Case<'static>>> {
    let mut rows = Vec::new();
    for (cell, line) in include_str!("cells.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [stratum, person, first, second, ask_first, ask_second, value_first, value_second] =
            fields.as_slice()
        else {
            return Err("invalid crossed cell definition".into());
        };
        for relation in 0..2 {
            for order in 0..2 {
                for exchange in 0..2 {
                    let cues = [*first, *second];
                    let values = [*value_first, *value_second];
                    let facts = (0..2)
                        .map(|position| {
                            let index = position ^ order;
                            Fact {
                                person,
                                cue: cues[index],
                                value: values[index ^ exchange],
                            }
                        })
                        .collect();
                    rows.push(Case {
                        stratum,
                        person,
                        relation: [*ask_first, *ask_second][relation],
                        excluded: None,
                        facts,
                        selected: vec![relation ^ order],
                        factors: Some([cell, relation, order, exchange]),
                    });
                }
            }
        }
    }
    for line in include_str!("controls.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let [stratum, person, relation, assertions, selected] = fields.as_slice() else {
            return Err("invalid control definition".into());
        };
        let facts = assertions
            .split(';')
            .map(|assertion| {
                let fields: Vec<_> = assertion.split('|').collect();
                let [person, cue, value] = fields.as_slice() else {
                    return Err("invalid control assertion".into());
                };
                Ok(Fact { person, cue, value })
            })
            .collect::<Result<Vec<_>>>()?;
        let selected = if *selected == "-" {
            Vec::new()
        } else {
            selected
                .split(',')
                .map(str::parse::<usize>)
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let excluded =
            (*stratum == "exclusion").then_some(if *person == "Nina" { "Owen" } else { "Nina" });
        rows.push(Case {
            stratum,
            person,
            relation,
            excluded,
            facts,
            selected,
            factors: None,
        });
    }
    Ok(rows)
}

fn append(text: &mut String, token: &str) -> Span {
    let start = PREFIX.len() + text.len();
    text.push(' ');
    text.push_str(token);
    [start, PREFIX.len() + text.len()]
}

struct Prompt {
    text: String,
    facts: Vec<[Span; 3]>,
    query: Span,
    requested: Span,
    relation: Span,
    excluded: Option<Span>,
}

fn prompt(case: &Case<'_>, instructed: bool) -> Prompt {
    let mut text = PRELUDE.to_owned();
    let facts = case
        .facts
        .iter()
        .map(|fact| {
            let person = append(&mut text, fact.person);
            let cue = append(&mut text, fact.cue);
            let value = append(&mut text, fact.value);
            text.push('.');
            [person, cue, value]
        })
        .collect();
    let question_start = PREFIX.len() + text.len();
    text.push_str(" Which exact item does the text say");
    let requested = append(&mut text, case.person);
    let relation = append(&mut text, case.relation);
    let excluded = case.excluded.map(|person| {
        text.push_str(" (ignore");
        let span = append(&mut text, person);
        text.push(')');
        span
    });
    text.push('?');
    let query = [question_start, PREFIX.len() + text.len()];
    if instructed {
        text.push_str(" If the item cannot be determined, say \"I don't know.\"");
    }
    Prompt {
        text,
        facts,
        query,
        requested,
        relation,
        excluded,
    }
}

fn token_span(boundaries: &[usize], [start, end]: Span) -> Result<Span> {
    let at = |byte| {
        boundaries
            .binary_search(&byte)
            .map_err(|_| format!("byte {byte} splits an original token"))
    };
    Ok([at(start)?, at(end)?])
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let [_, tokenizer_path, output] = args.as_slice() else {
        return Err("usage: d27-relation-panel TOKENIZER NEW_OUTPUT_DIRECTORY".into());
    };
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    if format!("{:x}", Sha256::digest(&tokenizer_bytes)) != TOKENIZER {
        return Err("tokenizer SHA-256 differs from the D26 protocol-2 pin".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("bad tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut unique = BTreeSet::new();
    let mut output_rows = Vec::new();
    let cases = cases()?;
    for (index, (case, instructed)) in cases
        .iter()
        .flat_map(|case| [(case, false), (case, true)])
        .enumerate()
    {
        let row = 2700 + (index * 37) % 251;
        let split = if case.stratum == "train" {
            "train"
        } else {
            "held"
        };
        let prompt = prompt(case, instructed);
        if !unique.insert(prompt.text.clone()) {
            return Err(format!("duplicate prompt at row {row}").into());
        }
        let ids = encoder
            .encode_assistant_prefix(&[Message {
                role: "user",
                content: &prompt.text,
            }])
            .tokens;
        let decoded = tokenizer.decode(&ids);
        if decoded != format!("{PREFIX}{}\nAssistant:", prompt.text) {
            return Err(format!("row {row} protocol-2 round trip failed").into());
        }
        let mut boundaries = vec![0];
        for &id in &ids {
            boundaries.push(
                boundaries.last().ok_or("empty boundaries")? + tokenizer.decode_bytes(&[id]).len(),
            );
        }
        let at = |span| token_span(&boundaries, span);
        let content = at([PREFIX.len() - 1, PREFIX.len() + prompt.text.len()])?;
        let query = at(prompt.query)?;
        let requested = at(prompt.requested)?;
        let relation = at(prompt.relation)?;
        let excluded = prompt.excluded.map(at).transpose()?;
        let fact_spans = prompt
            .facts
            .iter()
            .map(|&[person, cue, value]| {
                Ok(json!({
                    "person": at(person)?,
                    "relation": at(cue)?,
                    "value": at(value)?,
                }))
            })
            .collect::<Result<Vec<Value>>>()?;
        let mut accepted_addresses = Vec::new();
        let mut values = BTreeSet::new();
        for &position in &case.selected {
            let fact = case
                .facts
                .get(position)
                .ok_or("invalid gold assertion index")?;
            if fact.person != case.person || Some(fact.person) == case.excluded {
                return Err(format!("row {row} gold assertion violates person filter").into());
            }
            values.insert(fact.value);
            accepted_addresses.push(at(prompt.facts[position][2])?);
        }
        let expected = match values.len() {
            0 | 2.. => None,
            1 => values.first().copied(),
        };
        let expected_tape = if let Some(value) = expected {
            let [start, end] = accepted_addresses[0];
            let mut tape = ids[start..end].to_vec();
            for &[other_start, other_end] in &accepted_addresses {
                if tokenizer.decode(&ids[other_start..other_end]) != format!(" {value}")
                    || ids[other_start..other_end] != tape
                {
                    return Err(
                        format!("row {row} gold value fails exact original-ID agreement").into(),
                    );
                }
            }
            tape.push(1);
            Some(tape)
        } else if instructed {
            let mut tape = tokenizer.encode(" I don't know.");
            tape.push(1);
            Some(tape)
        } else {
            None
        };
        output_rows.push((
            row,
            split,
            prompt.text,
            json!({"row": row, "ids": ids}),
            json!({"row": row, "content": content, "query": query}),
            json!({"row": row, "requested": requested, "excluded": excluded}),
            json!({"row": row, "relation": relation}),
            json!({
                "row": row, "stratum": case.stratum, "factors": case.factors,
                "instructed": instructed,
                "assertions": fact_spans, "selected": case.selected,
                "selected_value_spans": accepted_addresses, "expected_value": expected,
                "expected_tape": expected_tape,
            }),
        ));
    }
    output_rows.sort_by_key(|row| row.0);
    let out = Path::new(output);
    fs::create_dir(out)?;
    for split in ["train", "held"] {
        let mut blind = String::new();
        let mut ids = String::new();
        let mut framing = String::new();
        let mut persons = String::new();
        let mut relations = String::new();
        let mut gold = String::new();
        for (row, row_split, text, input, frame, person, relation, key) in &output_rows {
            if *row_split == split {
                blind.push_str(&format!("{row}\t{text}\n"));
                ids.push_str(&format!("{input}\n"));
                framing.push_str(&format!("{frame}\n"));
                persons.push_str(&format!("{person}\n"));
                relations.push_str(&format!("{relation}\n"));
                gold.push_str(&format!("{key}\n"));
            }
        }
        fs::write(out.join(format!("{split}.blind.tsv")), blind)?;
        fs::write(out.join(format!("{split}.blind.jsonl")), ids)?;
        fs::write(out.join(format!("{split}.framing.jsonl")), framing)?;
        fs::write(out.join(format!("{split}.person.jsonl")), persons)?;
        fs::write(out.join(format!("{split}.relation.jsonl")), relations)?;
        fs::write(out.join(format!("{split}.gold.jsonl")), gold)?;
    }
    Ok(())
}
