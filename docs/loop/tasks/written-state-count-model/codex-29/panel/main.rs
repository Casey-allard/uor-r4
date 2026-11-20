use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs,
    io::Write,
    path::Path,
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];
const HEADER: &str = "<|bos|>User: ";
const TOKENIZER_SHA: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const STOPS: [u32; 4] = [16, 1, 0, 201];

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn append(text: &mut String, value: &str) -> Span {
    let start = HEADER.len() + text.len();
    text.push(' ');
    text.push_str(value);
    [start, HEADER.len() + text.len()]
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

struct Fact {
    name: Span,
    connector: Span,
    value: Span,
    literal: String,
}

fn fact(text: &mut String, name: &str, connector: &str, value: &str) -> Fact {
    let name = append(text, name);
    let connector = append(text, connector);
    let value_span = append(text, value);
    text.push('.');
    Fact {
        name,
        connector,
        value: value_span,
        literal: value.to_owned(),
    }
}

struct Case {
    row: u32,
    stratum: &'static str,
    family: &'static str,
    connector_arm: Option<usize>,
    object_arm: Option<usize>,
    context_arm: Option<usize>,
    role_factors: Option<[usize; 3]>,
    text: String,
    requested: Span,
    excluded: Option<Span>,
    unrequested: Option<Span>,
    query_start: usize,
    facts: Vec<Fact>,
    selected: Vec<usize>,
}

fn cases() -> Result<(Vec<Case>, Vec<Case>)> {
    let mut held = Vec::new();
    let mut roles = Vec::new();
    for (family_ix, line) in include_str!("cells.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [family, name, short, long, a, b] = fields.as_slice() else {
            return Err("bad matched source row".into());
        };
        for (connector_arm, connector) in [*short, *long].into_iter().enumerate() {
            for (object_arm, value) in [*a, *b].into_iter().enumerate() {
                let mut text = "A worker left this note.".to_owned();
                let assertion = fact(&mut text, name, connector, value);
                let query_start = append(&mut text, "What is the complete item described for")[0];
                let requested = append(&mut text, name);
                text.push('?');
                held.push(Case {
                    row: 29000 + (4 * family_ix + 2 * connector_arm + object_arm) as u32,
                    stratum: "matched",
                    family,
                    connector_arm: Some(connector_arm),
                    object_arm: Some(object_arm),
                    context_arm: None,
                    role_factors: None,
                    text,
                    requested,
                    excluded: None,
                    unrequested: None,
                    query_start,
                    facts: vec![assertion],
                    selected: vec![0],
                });
            }
        }
    }
    for (family_ix, line) in include_str!("boundary.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [family, name, verb, full, particle, short, late_context, early_context] =
            fields.as_slice()
        else {
            return Err("bad contextual source row".into());
        };
        if format!("{particle} {short}") != *full {
            return Err("boundary pair does not preserve the same assertion".into());
        }
        for (context_arm, context) in [*early_context, *late_context].into_iter().enumerate() {
            let mut text = context.to_owned();
            let connector = if context_arm == 0 {
                verb.to_string()
            } else {
                format!("{verb} {particle}")
            };
            let value = if context_arm == 0 { *full } else { *short };
            let assertion = fact(&mut text, name, &connector, value);
            let query_start = append(&mut text, "Read back the exact complete item wording for")[0];
            let requested = append(&mut text, name);
            text.push('.');
            held.push(Case {
                row: 29050 + (2 * family_ix + context_arm) as u32,
                stratum: "boundary_switch",
                family,
                connector_arm: None,
                object_arm: None,
                context_arm: Some(context_arm),
                role_factors: None,
                text,
                requested,
                excluded: None,
                unrequested: None,
                query_start,
                facts: vec![assertion],
                selected: vec![0],
            });
        }
    }
    for (index, line) in include_str!("controls.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [stratum, name, ask, short, a, long, b] = fields.as_slice() else {
            return Err("bad control source row".into());
        };
        let mut text =
            "The register contains two notes about the same inspection, with room for one item per person.".to_owned();
        let facts = vec![
            fact(&mut text, name, short, a),
            fact(&mut text, name, long, b),
        ];
        let query_start = append(
            &mut text,
            "What single complete item wording do both notes give for",
        )[0];
        let requested = append(&mut text, ask);
        text.push('?');
        let selected = match *stratum {
            "agreement" if a == b && name == ask => vec![0, 1],
            "conflict" if a != b && name == ask => vec![0, 1],
            "absent" if name != ask => vec![],
            _ => return Err("invalid semantic control".into()),
        };
        let family = *stratum;
        let stratum = match family {
            "agreement" => "agreement",
            "conflict" => "conflict",
            _ => "absent",
        };
        held.push(Case {
            row: 29080 + index as u32,
            stratum,
            family,
            connector_arm: None,
            object_arm: None,
            context_arm: None,
            role_factors: None,
            text,
            requested,
            excluded: None,
            unrequested: None,
            query_start,
            facts,
            selected,
        });
    }
    for (family_ix, line) in include_str!("role.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let [family, first, second, observer, verb, a, b, c] = fields.as_slice() else {
            return Err("bad role source row".into());
        };
        let people = [*first, *second, *observer];
        let values = [*a, *b, *c];
        for selected in 0..2 {
            for fact_order in 0..2 {
                for query_order in 0..2 {
                    let mut order = vec![fact_order, 1 - fact_order];
                    if family_ix == 0 {
                        order.push(2);
                    }
                    let mut text = "The sheet lists a different object on each line.".to_owned();
                    let facts = order
                        .into_iter()
                        .map(|i| fact(&mut text, people[i], verb, values[i]))
                        .collect();
                    let mut requested = None;
                    let mut excluded = None;
                    let query_start = HEADER.len() + text.len();
                    for polarity in [query_order, 1 - query_order] {
                        if polarity == 0 {
                            append(&mut text, "Give the exact item for");
                            requested = Some(append(&mut text, people[selected]));
                        } else {
                            append(&mut text, "Ignore the line for");
                            excluded = Some(append(&mut text, people[1 - selected]));
                        }
                        text.push('.');
                    }
                    let unrequested = if family_ix == 0 {
                        let span = append(&mut text, observer);
                        append(&mut text, "is mentioned only to identify the third line");
                        text.push('.');
                        Some(span)
                    } else {
                        None
                    };
                    roles.push(Case {
                        row: 29100
                            + (8 * family_ix + 4 * selected + 2 * fact_order + query_order) as u32,
                        stratum: "role",
                        family,
                        connector_arm: None,
                        object_arm: None,
                        context_arm: None,
                        role_factors: Some([selected, fact_order, query_order]),
                        text,
                        requested: requested.ok_or("missing requested role")?,
                        excluded,
                        unrequested,
                        query_start,
                        facts,
                        selected: vec![selected ^ fact_order],
                    });
                }
            }
        }
        let mut text = "There is only one entry on this slip.".to_owned();
        let assertion = fact(&mut text, people[0], verb, values[0]);
        let query_start = append(&mut text, "Which exact item is named for")[0];
        let requested = append(&mut text, people[0]);
        text.push('?');
        roles.push(Case {
            row: 29140 + family_ix as u32,
            stratum: "role_direct",
            family,
            connector_arm: None,
            object_arm: None,
            context_arm: None,
            role_factors: None,
            text,
            requested,
            excluded: None,
            unrequested: None,
            query_start,
            facts: vec![assertion],
            selected: vec![0],
        });
    }
    Ok((held, roles))
}

fn fresh() -> Result<()> {
    let old: BTreeSet<_> = [
        include_str!("../../codex-28/panel/train.tsv"),
        include_str!("../../codex-28/panel/held.tsv"),
        include_str!("../../codex-28/panel/controls.tsv"),
        include_str!("../../codex-28/held/cells.tsv"),
        include_str!("../../codex-28/held/controls.tsv"),
    ]
    .into_iter()
    .flat_map(|table| table.lines().flat_map(|line| line.split('\t').skip(1)))
    .map(str::to_ascii_lowercase)
    .collect();
    for table in [
        include_str!("cells.tsv"),
        include_str!("boundary.tsv"),
        include_str!("controls.tsv"),
        include_str!("role.tsv"),
    ] {
        for line in table.lines() {
            for term in line.split('\t').skip(1) {
                if old.contains(&term.to_ascii_lowercase()) {
                    return Err(format!("D29 surface duplicates D28: {term}").into());
                }
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct Panel {
    public: String,
    blind: Vec<Value>,
    framing: Vec<Value>,
    person: Vec<Value>,
    gold: Vec<Value>,
    boundary: BTreeMap<&'static str, (Vec<u32>, usize)>,
}

fn rows(cases: Vec<Case>, tokenizer: &ByteBpeTokenizer) -> Result<Panel> {
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let mut panel = Panel::default();
    let mut unique = BTreeSet::new();
    for case in cases {
        let row = case.row;
        let user = Message {
            role: "user",
            content: &case.text,
        };
        let ids = encoder.encode_assistant_prefix(&[user]).tokens;
        if ids.first() != Some(&0)
            || ids.len() > 176
            || !unique.insert(ids.clone())
            || tokenizer.decode(&ids) != format!("{HEADER}{}\nAssistant:", case.text)
        {
            return Err(format!("row {row} framing differs").into());
        }
        let mut boundaries = vec![0];
        let mut classes = Vec::new();
        for &id in &ids {
            let word = tokenizer.decode_bytes(&[id]);
            boundaries
                .push(boundaries.last().copied().ok_or("missing token boundary")? + word.len());
            classes.push(class(&word));
        }
        let at = |[a, b]: Span| -> Result<Span> {
            let index = |x| {
                boundaries
                    .binary_search(&x)
                    .map_err(|_| format!("row {row} byte {x} splits a token: {}", case.text))
            };
            Ok([index(a)?, index(b)?])
        };
        let requested = at(case.requested)?;
        let excluded = case.excluded.map(at).transpose()?;
        let unrequested = case.unrequested.map(at).transpose()?;
        let content = at([HEADER.len() - 1, HEADER.len() + case.text.len()])?;
        let query = at([case.query_start, HEADER.len() + case.text.len()])?;
        let mut fact_spans = Vec::new();
        for ft in &case.facts {
            let head = at(ft.name)?;
            let connector = at(ft.connector)?;
            let value = at(ft.value)?;
            let fence = (head[1]..query[0]).find(|&i| STOPS.contains(&ids[i]));
            if head[1] != connector[0]
                || connector[1] != value[0]
                || fence != Some(value[1])
                || ids.get(value[1]) != Some(&16)
                || value[1] - value[0] > 7
                || classes[value[0]] != 1
            {
                return Err(format!("row {row} assertion or value boundary invalid: {} {head:?} {connector:?} {value:?}",case.text).into());
            }
            fact_spans.push((head, connector, value));
        }
        let mut selected_value_spans = Vec::new();
        for &index in &case.selected {
            let (head, _, value) = fact_spans[index];
            if ids[head[0]..head[1]] != ids[requested[0]..requested[1]] {
                return Err(format!("row {row} selected person changed token IDs").into());
            }
            selected_value_spans.push(value);
        }
        let values_agree = selected_value_spans.first().is_some_and(|&[a, b]| {
            selected_value_spans
                .iter()
                .all(|&[c, d]| ids[a..b] == ids[c..d])
        });
        if case.stratum == "conflict" && values_agree
            || case.stratum == "agreement" && !values_agree
        {
            return Err(format!("row {row} multiple assertion control changed relation").into());
        }
        let mut expected = None;
        let mut expected_value = None;
        if values_agree {
            let [a, b] = selected_value_spans[0];
            let answer = &case.facts[case.selected[0]].literal;
            let tape = [&ids[a..b], &[1]].concat();
            let doc = encoder
                .encode_document(&[
                    user,
                    Message {
                        role: "assistant",
                        content: answer,
                    },
                ])
                .tokens;
            if doc.strip_prefix(ids.as_slice()) != Some(tape.as_slice()) {
                return Err(format!("row {row} literal writer oracle differs").into());
            }
            expected = Some(tape);
            expected_value = Some(answer);
        }
        let mut candidates = Vec::new();
        let mut source = None;
        let mut rival = None;
        let mut selected_head = None;
        let mut selected_connector = None;
        let mut selected_fence = None;
        let mut paired_local = None;
        if case.selected.len() == 1 {
            let (head, connector, [gold, fence]) = fact_spans[case.selected[0]];
            let offset = gold - head[1];
            if case
                .connector_arm
                .is_some_and(|arm| (offset > 3) != (arm == 1))
            {
                return Err(format!(
                    "row {row} wrong actual connector class at offset {offset}: {}",
                    case.text
                )
                .into());
            }
            for start in head[1] + 1..fence {
                if classes[start] != 1 || fence - start > 7 {
                    continue;
                }
                let end = (start + 1..fence)
                    .find(|&i| classes[i] != 2)
                    .unwrap_or(fence);
                let depth = (start - head[1]).min(3);
                candidates.push(json!({"source":start,"word_end":end,"fence":fence,
                    "offset":start-head[1],"legacy":start-head[1]<=3,"descriptor_depth":depth,
                    "word_width":end-start,"descriptor_ids":&ids[start-depth..end],
                    "descriptor_classes":&classes[start-depth..end],
                    "copy_ids_through_eos":([&ids[start..fence],&[1]].concat()),"positive":start==gold}));
            }
            if !candidates.iter().any(|c| c["source"] == gold) {
                return Err(
                    format!("row {row} gold is not a legal whole word after connector").into(),
                );
            }
            let context_witness = case.context_arm.map(|arm| {
                let first = &ids[head[0]..];
                (first.to_vec(), gold - head[0], arm)
            });
            if let Some((local, relative_gold, arm)) = context_witness {
                let relative_rival = if arm == 0 {
                    candidates
                        .iter()
                        .find(|c| c["source"].as_u64().is_some_and(|s| s as usize > gold))
                        .map(|c| c["source"].as_u64().unwrap_or_default() as usize - head[0])
                } else {
                    panel
                        .boundary
                        .get(case.family)
                        .map(|(_, old_gold)| *old_gold)
                };
                if arm == 0 {
                    panel
                        .boundary
                        .insert(case.family, (local.clone(), relative_gold));
                } else if !panel
                    .boundary
                    .get(case.family)
                    .is_some_and(|(other, t)| *other == local && *t < relative_gold)
                {
                    return Err(
                        format!("row {row} contextual pair local IDs or order differ").into(),
                    );
                }
                rival = relative_rival.map(|i| i + head[0]);
                paired_local =
                    Some(json!({"start":head[0],"ids":local,"gold_offset":relative_gold}));
            } else {
                rival = candidates
                    .iter()
                    .filter_map(|c| c["source"].as_u64())
                    .map(|i| i as usize)
                    .rfind(|&i| i < gold)
                    .or_else(|| {
                        candidates
                            .iter()
                            .filter_map(|c| c["source"].as_u64())
                            .map(|i| i as usize)
                            .find(|&i| i > gold)
                    });
            }
            if rival.is_none()
                || case.connector_arm == Some(1) && !rival.is_some_and(|r| r < gold)
                || !rival.is_some_and(|r| candidates.iter().any(|c| c["source"] == r && r != gold))
            {
                return Err(
                    format!("row {row} missing legal competing onset from actual tape").into(),
                );
            }
            source = Some(gold);
            selected_head = Some(head);
            selected_connector = Some(connector);
            selected_fence = Some(fence);
        }
        let wrong_tape = rival
            .zip(selected_fence)
            .map(|(start, end)| [&ids[start..end], &[1]].concat());
        let assertions:Vec<_>=fact_spans.into_iter().map(|(person,relation,value)|
            json!({"person":person,"relation":relation,"value":value})).collect();
        let bytes: Vec<_> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
        panel.public.push_str(&format!("{row}\t{}\n", case.text));
        panel.blind.push(json!({"row":row,"ids":ids}));
        panel
            .framing
            .push(json!({"row":row,"content":content,"query":query}));
        panel
            .person
            .push(json!({"row":row,"requested":requested,"excluded":excluded}));
        panel.gold.push(json!({"row":row,"stratum":case.stratum,"family":case.family,
            "connector_arm":case.connector_arm,"object_arm":case.object_arm,"context_arm":case.context_arm,
            "role_factors":case.role_factors,"matched_pair":case.context_arm.map(|_|case.family),"local_pair_tape":paired_local,
            "requested":requested,"role_targets":{"requested":requested,"excluded":excluded,"unrequested":unrequested},
            "assertion_head":selected_head,"fact_end":selected_head.map(|h|h[1]-1),"query_start":requested[0],
            "connector":selected_connector,"gold_source":source,"wrong_source":rival,"fence":selected_fence,
            "expected_value":expected_value,"expected_tape":expected,"wrong_tape":wrong_tape,"prefix_sha256":sha(&bytes),
            "assertions":assertions,"selected":case.selected,"selected_value_spans":selected_value_spans,"candidates":candidates,
        }));
    }
    Ok(panel)
}

fn write(path: &Path, prefix: &str, panel: Panel) -> Result<()> {
    fs::write(path.join(format!("{prefix}.blind.tsv")), panel.public)?;
    for (name, records) in [
        ("blind", panel.blind),
        ("framing", panel.framing),
        ("person", panel.person),
        ("gold", panel.gold),
    ] {
        let mut writer = std::io::BufWriter::new(fs::File::create_new(
            path.join(format!("{prefix}.{name}.jsonl")),
        )?);
        for record in records {
            serde_json::to_writer(&mut writer, &record)?;
            writeln!(writer)?;
        }
        writer.flush()?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [tokenizer, out] = args.as_slice() else {
        return Err("usage: d29-rival-panel TOKENIZER NEW_OUTPUT".into());
    };
    fresh()?;
    let data = fs::read(tokenizer)?;
    if sha(&data) != TOKENIZER_SHA {
        return Err("tokenizer pin differs".into());
    }
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&data).ok_or("invalid tokenizer")?;
    let (held, roles) = cases()?;
    let panels = [("held", rows(held, &t)?), ("role", rows(roles, &t)?)];
    let out = Path::new(out);
    fs::create_dir(out)?;
    for (prefix, panel) in panels {
        write(out, prefix, panel)?;
    }
    Ok(())
}
