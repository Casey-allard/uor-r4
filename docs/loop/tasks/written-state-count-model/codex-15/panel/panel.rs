//! Seals independently authored text and audits frozen tokenizer/feature admission only.
mod design;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use uor_r4_integer::discrete_d11::{
    spiralcore_orbit_roots, spiralcore_signed_axes, IntegerDiscreteModel,
};
use uor_r4_integer::paired_role::{role_candidates, PairedRoleState};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [usize; 2];
const HEAD: &str = "<|bos|>User: ";
const TAIL: &str = "\nAssistant:";
const WIDTH: usize = 12;
const MODEL: [&str; 2] = [
    "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
    "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
];
const TOKENIZER: [&str; 2] = [
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
    "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
];
const BANK: [&str; 2] = [
    "56610eab5073cc4b1c8edc7bbae9a9345cbe12876d47b9aec81f4862630087d0",
    "1c386426e4b5395de9cfa0e4fd4e2d59b2867e55ce1c930bc1eff4ee534f9d44",
];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn id_digest(ids: &[u32]) -> String {
    digest(&ids.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
}

fn pin(path: &str, expected: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    if digest(&bytes) != expected {
        return Err(format!("pinned input changed: {path}").into());
    }
    Ok(bytes)
}

fn write(out: &Path, name: &str, rows: &Value) -> Result<()> {
    let mut file = BufWriter::new(
        File::options()
            .create_new(true)
            .write(true)
            .open(out.join(name))?,
    );
    if let Some(rows) = rows.as_array() {
        writeln!(file, "[")?;
        for (i, row) in rows.iter().enumerate() {
            serde_json::to_writer(&mut file, row)?;
            writeln!(file, "{}", if i + 1 == rows.len() { "" } else { "," })?;
        }
        writeln!(file, "]")?;
    } else {
        serde_json::to_writer_pretty(&mut file, rows)?;
        writeln!(file)?;
    }
    file.flush()?;
    Ok(())
}

struct Inputs<'a> {
    tokenizer: &'a ByteBpeTokenizer,
    encoder: DialogueEncoder<'a>,
    sites: &'a [[u32; 2]],
    roots: &'a [u32],
}

#[derive(Clone)]
struct Fact {
    name: String,
    value: String,
}

struct Topic {
    facts: Vec<Fact>,
    names: [String; 4],
}

fn topic(train: bool, family: usize, i: usize) -> Topic {
    let (names, adjectives, things) = if train {
        (
            &design::TRAIN_NAMES,
            &design::TRAIN_ADJ,
            &design::TRAIN_THING,
        )
    } else {
        (&design::HELD_NAMES, &design::HELD_ADJ, &design::HELD_THING)
    };
    let names = names[i % 4].map(|name| {
        if i & 4 == 0 {
            name.into()
        } else {
            name.to_ascii_lowercase()
        }
    });
    let facts = [0, 1, 2]
        .map(|p| {
            let adjective = adjectives[(family + p * 3) % adjectives.len()];
            let object = things[(i / 4 + p * 3) % things.len()];
            Fact {
                name: names[p].clone(),
                value: format!("{adjective} {object}"),
            }
        })
        .into_iter()
        .collect::<Vec<_>>();
    Topic { facts, names }
}

fn ordered_facts(topic: &Topic, third: bool, reversed: bool) -> Vec<Fact> {
    let mut facts = topic.facts[..if third { 3 } else { 2 }].to_vec();
    if reversed {
        facts.swap(0, 1);
    }
    facts
}

struct Question {
    text: String,
    mentions: Vec<(char, usize, String)>,
}

fn render(wrapper: &str, names: [&str; 3]) -> Result<Question> {
    let mut text = String::new();
    let mut mentions = Vec::new();
    let mut parts = wrapper.split('{');
    text.push_str(parts.next().ok_or("missing template")?);
    for piece in parts {
        let (slot, suffix) = piece.split_once('}').ok_or("unterminated slot")?;
        let (role, name) = match slot {
            "R" => ('R', names[0]),
            "X" => ('X', names[1]),
            "B" => ('B', names[1]),
            "C" => ('C', names[2]),
            _ => return Err("unknown role slot".into()),
        };
        mentions.push((role, text.len(), name.into()));
        text.push_str(name);
        text.push_str(suffix);
    }
    Ok(Question { text, mentions })
}

fn span(
    t: &ByteBpeTokenizer,
    ids: &[u32],
    offsets: &[usize],
    surface: &str,
    start: usize,
) -> Result<Span> {
    let begin = offsets
        .binary_search(&start.checked_sub(1).ok_or("missing leading space")?)
        .map_err(|_| format!("unaligned start for {surface} at {start}"))?;
    let end = offsets
        .binary_search(&(start + surface.len()))
        .map_err(|_| format!("unaligned end for {surface} at {start}"))?;
    if begin >= end || t.decode_bytes(&ids[begin..end]) != format!(" {surface}").as_bytes() {
        return Err(format!("verbatim span failed for {surface} at {start}").into());
    }
    Ok([begin, end])
}

struct Features {
    span: Span,
    ids: Vec<u32>,
    sorted: Vec<u32>,
    identity: [i16; 32],
    action: [i16; 32],
    slots: Vec<(usize, usize, u32)>,
}

fn features(roots: &[u32], ids: &[u32], span: Span, bounds: Span, structural: Span) -> Features {
    let mut result = Features {
        span,
        ids: ids[span[0]..span[1]].to_vec(),
        sorted: Vec::new(),
        identity: [0; 32],
        action: [0; 32],
        slots: Vec::new(),
    };
    let axes = spiralcore_signed_axes();
    let orbit = spiralcore_orbit_roots();
    for distance in 0..WIDTH {
        for (position, state) in [
            (span[0].checked_sub(1 + distance), 1 + distance),
            (span[1].checked_add(distance), 33 + distance),
        ] {
            let Some(position) = position else { continue };
            if !(bounds[0]..bounds[1]).contains(&position)
                || !(structural[0]..structural[1]).contains(&position)
            {
                continue;
            }
            let id = ids[position];
            result.sorted.push(id);
            result.slots.push((position, state, id));
            for block in 0..4 {
                let root = orbit[((roots[id as usize] >> (4 * block)) & 15) as usize];
                for axis in 0..8 {
                    let signed = axes[8 * state + axis];
                    let component = root[usize::from(signed & 7)] as i16;
                    let offset = 8 * block + axis;
                    result.identity[offset] += i16::from(root[axis]);
                    result.action[offset] += if signed & 8 == 0 {
                        component
                    } else {
                        -component
                    };
                }
            }
        }
    }
    result.sorted.sort_unstable();
    result
}

struct Row {
    ids: Vec<u32>,
    text: String,
    bounds: Span,
    grade: Value,
    clipped: Vec<Features>,
    matched: Vec<Option<Span>>,
    census: Value,
}

struct Metadata {
    id: String,
    family: String,
    pair: String,
    direction: usize,
    stratum: &'static str,
    wrapper: String,
    counterpart: Option<String>,
    accept: Option<String>,
}

fn row(input: &Inputs, facts: &[Fact], question: Question, meta: Metadata) -> Result<Row> {
    let mut text = String::new();
    let mut positions = Vec::new();
    for fact in facts {
        if !text.is_empty() {
            text.push(' ');
        }
        let name_byte = HEAD.len() + text.len();
        text.push_str(&fact.name);
        text.push_str(" kept ");
        let value_byte = HEAD.len() + text.len();
        text.push_str(&fact.value);
        text.push('.');
        positions.push((name_byte, value_byte));
    }
    if !text.is_empty() {
        text.push(' ');
    }
    let question_byte = text.len() + HEAD.len();
    text.push_str(&question.text);
    let message = Message {
        role: "user",
        content: &text,
    };
    let ids = input
        .encoder
        .encode_assistant_prefix(&[message.clone()])
        .tokens;
    if ids.len() + 16 > 192 || input.tokenizer.decode(&ids) != format!("{HEAD}{text}{TAIL}") {
        return Err(format!("unfaithful protocol-2 tape {}", meta.id).into());
    }
    let mut offsets = vec![0];
    for &id in &ids {
        offsets.push(offsets[offsets.len() - 1] + input.tokenizer.decode_bytes(&[id]).len());
    }
    let bounds = span(
        input.tokenizer,
        &ids,
        &offsets,
        &question.text,
        question_byte,
    )?;
    let content = span(input.tokenizer, &ids, &offsets, &text, HEAD.len())?;
    let spans = role_candidates(input.sites, &ids)?
        .map(|(s, _)| [s.start, s.end])
        .collect::<Vec<_>>();

    let mut requested = None;
    let mut requested_name = None;
    let mut excluded = None;
    let mut mentioned = Vec::new();
    for (role, at, name) in question.mentions {
        let endpoints = span(input.tokenizer, &ids, &offsets, &name, question_byte + at)?;
        if endpoints[0] < bounds[0] || endpoints[1] > bounds[1] {
            return Err(format!("question excludes label {}", meta.id).into());
        }
        match role {
            'R' => {
                requested = Some(endpoints);
                requested_name = Some(name.clone());
            }
            'X' => excluded = Some(endpoints),
            _ => {}
        }
        mentioned.push((name, endpoints));
    }
    let labels_admitted = [requested, excluded]
        .into_iter()
        .flatten()
        .all(|s| spans.contains(&s));
    if meta.id.starts_with("train-") && !labels_admitted {
        return Err(format!(
            "TRAIN label failed full-prefix admission {} ({text})",
            meta.id
        )
        .into());
    }

    let mut annotated_facts = Vec::new();
    let mut support = Vec::new();
    let mut matched = Vec::new();
    for (fact, (name_byte, value_byte)) in facts.iter().zip(positions) {
        let name = span(input.tokenizer, &ids, &offsets, &fact.name, name_byte)?;
        let value = span(input.tokenizer, &ids, &offsets, &fact.value, value_byte)?;
        if requested_name.as_ref() == Some(&fact.name) {
            support.push((fact.value.as_str(), value));
        }
        // Supported and matched fixtures mention each factual identity at most once
        // in the question. Resolve via exact whole-word IDs plus occurrence ordinal 0.
        let hits = spans
            .iter()
            .copied()
            .filter(|candidate| {
                candidate[0] >= bounds[0]
                    && candidate[1] <= bounds[1]
                    && ids[candidate[0]..candidate[1]] == ids[name[0]..name[1]]
            })
            .collect::<Vec<_>>();
        if meta.stratum == "supported" && hits.len() > 1 {
            return Err(format!("ambiguous name ordinal in {}", meta.id).into());
        }
        matched.push(hits.first().copied());
        annotated_facts.push(json!({
            "name": fact.name,
            "value": fact.value,
            "name_span": name,
            "value_span": value,
            "name_ids": ids[name[0]..name[1]],
            "question_ordinal": if hits.is_empty() { None } else { Some(0) },
        }));
    }
    let mut expected = None;
    let mut answer_text = None;
    if matches!(meta.stratum, "supported" | "surface-reorder") {
        let [(answer, [start, end])] = support.as_slice() else {
            return Err(format!("supported fixture missing a unique literal {}", meta.id).into());
        };
        let full = input
            .encoder
            .encode_document(&[
                message,
                Message {
                    role: "assistant",
                    content: answer,
                },
            ])
            .tokens;
        let suffix = full
            .strip_prefix(ids.as_slice())
            .ok_or("unaligned assistant continuation")?;
        if suffix != [&ids[*start..*end], &[1]].concat() || end - start > 7 {
            return Err(format!("unfaithful exact value {} ({answer})", meta.id).into());
        }
        expected = Some(suffix.to_vec());
        answer_text = Some(*answer);
    }
    let mut clipped = Vec::new();
    let mut census = Vec::new();
    for candidate in spans {
        let admitted = bounds[0] <= candidate[0] && candidate[1] <= bounds[1];
        if admitted {
            let feature = features(input.roots, &ids, candidate, bounds, content);
            census.push(json!({
                "span": candidate,
                "word_ids": feature.ids,
                "admitted": true,
                "clipped_ids": feature.sorted,
                "slots": feature.slots,
                "identity": feature.identity,
                "positional": feature.action,
                "identity_sha256": digest(&feature.identity.map(i16::to_le_bytes).concat()),
                "positional_sha256": digest(&feature.action.map(i16::to_le_bytes).concat()),
            }));
            clipped.push(feature);
        } else {
            census.push(json!({"span": candidate, "word_ids": ids[candidate[0]..candidate[1]], "admitted": false }));
        }
    }
    let grade = json!({
        "id": meta.id, "family": meta.family, "pair": meta.pair, "direction": meta.direction,
        "stratum": meta.stratum, "delexicalized": meta.wrapper, "question": question.text,
        "counterpart": meta.counterpart, "requested": requested, "excluded": excluded,
        "requested_name": requested_name, "exact_labels_admitted": labels_admitted,
        "answer_text": answer_text, "expected": expected, "prose_acceptable": meta.accept,
        "facts": annotated_facts, "question_mentions": mentioned,
        "prefix_sha256": id_digest(&ids),
    });
    Ok(Row {
        ids,
        text,
        bounds,
        grade,
        clipped,
        matched,
        census: json!(census),
    })
}

#[derive(Default)]
struct Split {
    rows: Vec<Row>,
    matches: Vec<Value>,
    supported: BTreeSet<String>,
}

fn meta(
    family: usize,
    i: usize,
    realization: usize,
    direction: usize,
    train: bool,
    wrapper: &str,
) -> Metadata {
    let split = if train { "train" } else { "held" };
    let family = format!("{split}-{family}");
    let pair = format!("{family}-{i:02}-w{realization}");
    Metadata {
        id: format!("{pair}-d{direction}"),
        family,
        pair,
        direction,
        stratum: "supported",
        wrapper: wrapper.into(),
        counterpart: None,
        accept: None,
    }
}

fn compare_pair(left: &Row, right: &Row, row_left: usize) -> Value {
    let mut reasons = BTreeSet::new();
    let mut changes = Vec::new();
    let [mut left_spans, mut right_spans] = [Vec::new(), Vec::new()];
    for (&a, &b) in left.matched[..2].iter().zip(&right.matched[..2]) {
        left_spans.push(a);
        right_spans.push(b);
        let (Some(a), Some(b)) = (a, b) else {
            reasons.insert("mapped_candidate_missing");
            continue;
        };
        let left_feature = left.clipped.iter().find(|feature| feature.span == a);
        let right_feature = right.clipped.iter().find(|feature| feature.span == b);
        let (Some(l), Some(r)) = (left_feature, right_feature) else {
            reasons.insert("mapped_candidate_not_admitted");
            continue;
        };
        if l.ids != r.ids {
            reasons.insert("mapped_name_ids_changed");
        }
        if l.sorted != r.sorted {
            reasons.insert("clipped_token_multiset_changed");
        }
        if l.identity != r.identity {
            reasons.insert("identity_feature_changed");
        }
        if l.action == r.action {
            reasons.insert("positional_feature_collision");
        }
        changes.push(json!({
            "name_ids":l.ids, "ordinal":0,
            "left":[a[0] - left.bounds[0], left.bounds[1] - a[1]],
            "right":[b[0] - right.bounds[0], right.bounds[1] - b[1]],
            "identity_equal":l.identity == r.identity,
            "positional_equal":l.action == r.action,
            "token_multiset_equal":l.sorted == r.sorted,
        }));
    }
    if left.grade["requested"] == Value::Null
        || right.grade["requested"] == Value::Null
        || !left_spans
            .iter()
            .any(|s| json!(s) == left.grade["requested"])
        || !right_spans
            .iter()
            .any(|s| json!(s) == right.grade["requested"])
        || left.grade["requested_name"] == right.grade["requested_name"]
    {
        reasons.insert("requested_reversal_missing");
    }
    if left.grade["exact_labels_admitted"] != true || right.grade["exact_labels_admitted"] != true {
        reasons.insert("gold_label_not_admitted");
    }
    json!({
        "family":left.grade["family"], "pair":left.grade["pair"],
        "row_left":row_left, "row_right":row_left+1, "left":left_spans, "right":right_spans,
        "eligible":reasons.is_empty(), "reasons":reasons, "names":changes,
    })
}

fn control_meta(
    split: &str,
    stratum: &'static str,
    i: usize,
    wrapper: &str,
    accept: &str,
) -> Metadata {
    let id = format!("{split}-{stratum}-{i:02}");
    Metadata {
        pair: id.clone(),
        id,
        family: format!("{split}-{stratum}"),
        direction: 0,
        stratum,
        wrapper: wrapper.into(),
        counterpart: None,
        accept: Some(accept.into()),
    }
}

fn generate(input: &Inputs, train: bool) -> Result<Split> {
    let mut out = Split::default();
    let (split, templates, prose) = if train {
        ("train", design::TRAIN.as_slice(), &design::TRAIN_PROSE)
    } else {
        ("held", design::HELD.as_slice(), &design::HELD_PROSE)
    };
    for (family, wrappers) in templates.iter().enumerate() {
        for i in 0..32 {
            let topic = topic(train, family, i);
            let facts = ordered_facts(&topic, !train && family == 3, i & 8 != 0);
            let signature = facts
                .iter()
                .map(|f| format!("{}:{}", f.name, f.value))
                .collect::<BTreeSet<_>>();
            if !out
                .supported
                .insert(signature.into_iter().collect::<Vec<_>>().join("|"))
            {
                return Err(format!("duplicate distinct pair {split} {family} {i}").into());
            }
            for (realization, wrapper) in wrappers.iter().enumerate() {
                if !train && realization != i >> 4 {
                    continue;
                }
                let row_left = out.rows.len();
                for direction in 0..2 {
                    let p = ((i >> 1) ^ (i >> 3) ^ family ^ direction) & 1;
                    let question = render(
                        wrapper,
                        [&topic.names[p], &topic.names[1 - p], &topic.names[2]],
                    )?;
                    out.rows.push(row(
                        input,
                        &facts,
                        question,
                        meta(family, i, realization, direction, train, wrapper),
                    )?);
                }
                if !train {
                    out.matches.push(compare_pair(
                        &out.rows[row_left],
                        &out.rows[row_left + 1],
                        row_left,
                    ));
                }
            }
        }
    }
    for i in 0..32 {
        let family = i % 4;
        let pair_number = (i / 4) * 4 + (i / 4 + family) % 4;
        let topic = topic(train, family, pair_number);
        let facts = ordered_facts(&topic, !train && family == 3, pair_number & 8 != 0);
        let p = ((pair_number >> 1) ^ (pair_number >> 3) ^ family) & 1;
        let names = [
            &topic.names[p][..],
            &topic.names[1 - p][..],
            &topic.names[2][..],
        ];
        let realization = pair_number >> 4;
        let wrapper = templates[family][realization];

        let (query, relevance) = prose[i / 4];
        let standalone = if train {
            [
                "",
                "Answer briefly. ",
                "Explain this. ",
                "Help me understand. ",
            ]
        } else {
            [
                "",
                "One detail puzzles me. ",
                "Could you clarify something? ",
                "Here's my question. ",
            ]
        };
        let wrapper_prose = format!("{}{query}", standalone[i % 4]);
        out.rows.push(row(
            input,
            &[],
            render(&wrapper_prose, names)?,
            control_meta(split, "unbound-prose", i, &wrapper_prose, relevance),
        )?);

        let incidental = if train {
            "{R} was at the desk. Why might a lamp flicker?"
        } else {
            "{R} was on duty. What can make a hinge squeak?"
        };
        let mut no_request = render(incidental, names)?;
        for (role, _, _) in &mut no_request.mentions {
            *role = 'B';
        }
        let mut incidental_meta = control_meta(
            split,
            "incidental-no-request",
            i,
            incidental,
            if train {
                "A loose bulb, poor contact, or power fluctuation can cause flicker."
            } else {
                "Friction or insufficient lubrication at the hinge can cause squeaking."
            },
        );
        incidental_meta.counterpart =
            Some(meta(family, pair_number, realization, 0, train, wrapper).id);
        out.rows
            .push(row(input, &facts, no_request, incidental_meta)?);

        let unknown = if train {
            "Read out {R}'s selection."
        } else {
            "Can you recite {R}'s possession?"
        };
        let mut absent = facts.clone();
        // Known facts stay exactly as in the matched prompt. D is independently named
        // and never included in the assertion list.
        out.rows.push(row(
            input,
            &absent,
            render(unknown, [&topic.names[3], names[1], names[2]])?,
            control_meta(
                split,
                "unmentioned",
                i,
                unknown,
                "The record does not name an item for the requested person.",
            ),
        )?);

        let conflict = if train {
            "Look up {R}'s recorded article."
        } else {
            "Could you recover the entry for {R}?"
        };
        absent.push(Fact {
            name: names[0].into(),
            value: topic.facts[1 - p].value.clone(),
        });
        out.rows.push(row(
            input,
            &absent,
            render(conflict, names)?,
            control_meta(
                split,
                "conflict",
                i,
                conflict,
                "Two different recorded items conflict for the requested person.",
            ),
        )?);

        if !train {
            let reordered = templates[family][1 - realization];
            let mut surface = control_meta(
                split,
                "surface-reorder",
                i,
                reordered,
                "Same literal request after reordering clauses.",
            );
            surface.family = format!("held-{family}");
            let reference = meta(family, pair_number, realization, 0, false, wrapper);
            surface.counterpart = Some(reference.id);
            surface.pair = reference.pair;
            out.rows
                .push(row(input, &facts, render(reordered, names)?, surface)?);
        }
    }
    let unique = out.rows.iter().map(|row| &row.ids).collect::<BTreeSet<_>>();
    if unique.len() != out.rows.len() {
        let mut seen = BTreeMap::new();
        for row in &out.rows {
            if let Some(old) = seen.insert(&row.ids, &row.grade["id"]) {
                return Err(
                    format!("duplicate original IDs: {old} and {}", row.grade["id"]).into(),
                );
            }
        }
    }
    Ok(out)
}

fn split_output(out: &Path, name: &str, split: &Split) -> Result<()> {
    for (part, data) in [
        (
            "blind",
            json!(split
                .rows
                .iter()
                .map(|r| json!({"prefix":r.ids, "text":r.text}))
                .collect::<Vec<_>>()),
        ),
        (
            "bounds",
            json!(split
                .rows
                .iter()
                .map(|r| json!({"start":r.bounds[0], "end":r.bounds[1]}))
                .collect::<Vec<_>>()),
        ),
        (
            "grades",
            json!(split.rows.iter().map(|r| &r.grade).collect::<Vec<_>>()),
        ),
        (
            "census",
            json!(split.rows.iter().map(|r| &r.census).collect::<Vec<_>>()),
        ),
    ] {
        write(out, &format!("{name}-{part}.json"), &data)?;
    }
    if name == "held" {
        write(out, "held-match.json", &json!(split.matches))?;
    }
    Ok(())
}

fn intersection(a: &BTreeSet<String>, b: &BTreeSet<String>) -> Vec<String> {
    a.intersection(b).cloned().collect()
}

fn overlap(split: &Split) -> [BTreeSet<String>; 6] {
    let mut sets: [BTreeSet<String>; 6] = std::array::from_fn(|_| BTreeSet::new());
    for row in &split.rows {
        let label = &row.grade;
        for (index, key) in [(0, "delexicalized"), (1, "question"), (2, "prefix_sha256")] {
            sets[index].insert(label[key].as_str().unwrap_or_default().to_ascii_lowercase());
        }
        for fact in label["facts"].as_array().into_iter().flatten() {
            for (index, key) in [(3, "name"), (4, "value")] {
                sets[index].insert(fact[key].as_str().unwrap_or_default().to_ascii_lowercase());
            }
        }
        for entry in label["question_mentions"].as_array().into_iter().flatten() {
            sets[3].insert(entry[0].as_str().unwrap_or_default().to_ascii_lowercase());
        }
        for id in &row.ids[row.bounds[0]..row.bounds[1]] {
            sets[5].insert(id.to_string());
        }
    }
    sets
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [seed, model, tokenizer, bank, out] = args.as_slice() else {
        return Err("usage: ordered-role-panel SEED MODEL TOKENIZER BANK OUT".into());
    };
    let seed = match seed.as_str() {
        "1" => 0,
        "2" => 1,
        _ => return Err("seed must be 1 or 2".into()),
    };
    let model = IntegerDiscreteModel::from_bytes(&pin(model, MODEL[seed])?)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&pin(tokenizer, TOKENIZER[seed])?)
        .ok_or("invalid pinned tokenizer")?;
    let bank = PairedRoleState::from_bytes(&pin(bank, BANK[seed])?)?;
    if format!(
        "{:x}",
        sha2::digest::Output::<Sha256>::from(bank.bindings()[0])
    ) != MODEL[seed]
        || format!(
            "{:x}",
            sha2::digest::Output::<Sha256>::from(bank.bindings()[1])
        ) != TOKENIZER[seed]
    {
        return Err("role bank did not bind pinned D11 and tokenizer".into());
    }
    if spiralcore_signed_axes()[..8] != [0, 1, 2, 3, 4, 5, 6, 7] {
        return Err("D11 state zero is not identity".into());
    }
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let roots = model.packed_codebook_roots();
    let input = Inputs {
        tokenizer: &tokenizer,
        encoder: protocol.bind(&tokenizer)?,
        sites: bank.sites(),
        roots: &roots,
    };
    let train = generate(&input, true)?;
    let held = generate(&input, false)?;
    let out = Path::new(out);
    fs::create_dir(out)?;
    split_output(out, "train", &train)?;
    split_output(out, "held", &held)?;
    let train_sets = overlap(&train);
    let held_sets = overlap(&held);
    let common = train_sets
        .iter()
        .zip(&held_sets)
        .map(|(a, b)| intersection(a, b))
        .collect::<Vec<_>>();
    let counts = (0..4)
        .map(|family| {
            held.matches
                .iter()
                .filter(|p| p["family"] == format!("held-{family}") && p["eligible"] == true)
                .count()
        })
        .collect::<Vec<_>>();
    let summary = json!({
        "seed":seed+1, "model":MODEL[seed], "tokenizer":TOKENIZER[seed], "role_bank":BANK[seed],
        "train_total":train.rows.len(), "held_total":held.rows.len(), "held_match_per_family":counts,
        "train_held_overlap":{
            "delexicalized":common[0], "questions":common[1], "prefix_sha256":common[2],
            "complete_names_and_case_aliases":common[3], "literal_values":common[4], "question_token_ids":common[5],
            "shared_question_pieces":common[5].iter().map(|id| {
                id.parse::<u32>().map(|id| tokenizer.decode(&[id]))
            }).collect::<std::result::Result<Vec<_>,_>>()?,
        },
    });
    write(out, "summary.json", &summary)?;
    println!("{}", serde_json::to_string(&summary)?);
    if counts.iter().any(|count| *count < 16) || common[..5].iter().any(|set| !set.is_empty()) {
        return Err("pre-inference match or overlap gate failed; attempted panel retained".into());
    }
    Ok(())
}
