use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    discrete_d11::{
        spiralcore_orbit_roots, spiralcore_signed_axes, ExactCopyState, IntegerDiscreteModel,
        ROLE_COPY_STOPS,
    },
    paired_role::{PairedRoleDecision, RoleSpan},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

fn token_at(boundaries: &[usize], byte: usize) -> Result<usize> {
    boundaries
        .binary_search(&byte)
        .map_err(|_| format!("byte {byte} splits an original token").into())
}

fn writer(ids: &[u32], start: usize, fence: usize) -> Result<Vec<u32>> {
    let mut cursor = ExactCopyState::new(Some(u32::try_from(start)?), u32::try_from(ids.len())?);
    let mut emitted = Vec::new();
    for _ in 0..8 {
        let step = cursor
            .write(ids, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        emitted.push(step.emitted);
        cursor = step.state;
        if step.emitted == 1 {
            if step.completed_span() && emitted[..emitted.len() - 1] == ids[start..fence] {
                return Ok(emitted);
            }
            break;
        }
    }
    Err("forced writer did not preserve exact original IDs through EOS".into())
}

fn key(source: &str) -> Result<BTreeMap<u32, [&str; 3]>> {
    let mut entries = BTreeMap::new();
    for line in source.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let [row, early, late, reading] = fields.as_slice() else {
            return Err("bad key row".into());
        };
        if entries
            .insert(row.parse()?, [*early, *late, *reading])
            .is_some()
        {
            return Err("duplicate key row".into());
        }
    }
    Ok(entries)
}

fn prepare(
    model_path: &str,
    model_hash: &str,
    tokenizer_path: &str,
    out: &Path,
    fixtures: [&str; 3],
    key_source: &str,
    donors: &BTreeMap<u32, u32>,
) -> Result<()> {
    let model = IntegerDiscreteModel::load(Path::new(model_path))?;
    let tokenizer_data = fs::read(tokenizer_path)?;
    if model.artifact_sha256() != model_hash
        || format!("{:x}", Sha256::digest(&tokenizer_data)) != TOKENIZER
        || model.tokenizer_sha256() != Some(TOKENIZER)
    {
        return Err("model or tokenizer pin mismatch".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_data).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut key = key(key_source)?;
    let mut unique_prompts = BTreeSet::new();
    for (split, fixture) in ["train", "held", "exposed"].into_iter().zip(fixtures) {
        let mut packet = b"D25ROWS1\n".to_vec();
        let mut ordinary = Vec::new();
        let mut grading = Vec::new();
        let mut context_input = Vec::new();
        let mut context_expected = Vec::new();
        let mut local_input = Vec::new();
        let mut local_expected = Vec::new();
        let mut row_context = BTreeMap::new();
        let mut previous_row = 0;
        for line in fixture.lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let [row, context, sentence, tail, question] = fields.as_slice() else {
                return Err("bad panel row".into());
            };
            let row: u32 = row.parse()?;
            if row <= previous_row {
                return Err("panel rows must be strictly ascending".into());
            }
            previous_row = row;
            let [early, late, reading] = key.remove(&row).ok_or("missing panel key row")?;
            let mut text = context.to_string();
            let target_byte = "<|bos|>User: ".len() + text.len();
            for part in [sentence, tail, question] {
                if !part.is_empty() {
                    text.push(' ');
                    text.push_str(part);
                }
            }
            if !unique_prompts.insert(text.clone()) {
                return Err(format!("row {row} duplicates a complete prompt").into());
            }
            let ids = encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &text,
                }])
                .tokens;
            let decoded = tokenizer.decode(&ids);
            if decoded != format!("<|bos|>User: {text}\nAssistant:") {
                return Err(format!("row {row} protocol-2 round trip changed the prompt").into());
            }
            let mut byte = 0;
            let mut boundaries = vec![byte];
            for &id in &ids {
                byte += tokenizer.decode_bytes(&[id]).len();
                boundaries.push(byte);
            }
            let mut candidates = Vec::new();
            let mut exact = [None; 2];
            let mut fact_and_query = None;
            let mut fence = None;
            let mut writer_witnesses = BTreeMap::new();
            let mut legacy = None;
            // The supplied-assertion diagnostic receives no anchor when the
            // requested assertion is missing, even when another one is present.
            if !matches!(reading, "missing" | "irrelevant") {
                let name = sentence
                    .split_whitespace()
                    .next()
                    .ok_or("missing fact name")?;
                let fact = RoleSpan {
                    start: token_at(&boundaries, target_byte)?,
                    end: token_at(&boundaries, target_byte + 1 + name.len())?,
                };
                let query_byte = decoded
                    .rfind(&format!(" {name}"))
                    .ok_or("missing query name")?;
                let query = RoleSpan {
                    start: token_at(&boundaries, query_byte)?,
                    end: token_at(&boundaries, query_byte + 1 + name.len())?,
                };
                let end = token_at(&boundaries, target_byte + sentence.len())?;
                for (slot, literal) in [early, late].into_iter().enumerate() {
                    let offset = sentence
                        .rfind(&format!(" {literal}."))
                        .ok_or("literal is not an original sentence suffix")?;
                    let start = token_at(&boundaries, target_byte + offset + 1)?;
                    if tokenizer.decode(&ids[start..end]) != format!(" {literal}") {
                        return Err(format!("row {row} onsets decode differently").into());
                    }
                    writer_witnesses.insert(start, writer(&ids, start, end)?);
                    exact[slot] = Some(start);
                }
                candidates = model
                    .contextual_source_candidates(&ids, fact, query)
                    .map_err(|error| {
                        format!(
                            "row {row}, fact {fact:?}, query {query:?}, schema {}: {error}",
                            model.artifact_schema()
                        )
                    })?;
                if !exact.iter().all(|&start| {
                    candidates
                        .iter()
                        .any(|candidate| Some(candidate.span.start) == start)
                }) {
                    return Err(format!("row {row} lacks either plausible whole-word start").into());
                }
                fact_and_query = Some([[fact.start, fact.end], [query.start, query.end]]);
                fence = Some(end);
                let roles = PairedRoleDecision {
                    requested: None,
                    excluded: None,
                    requested_score: 0,
                    excluded_score: 0,
                    requested_runner_up: 0,
                    excluded_runner_up: 0,
                };
                let (legacy_decision, legacy_facts) =
                    model.paired_copy_witness(&ids, roles, Some(query))?;
                legacy = Some(json!({
                    "binder": legacy_decision,
                    "facts": legacy_facts,
                }));
                let context_coordinates = candidates
                    .first()
                    .ok_or("missing context for nonempty candidates")?
                    .contextual;
                if candidates
                    .iter()
                    .any(|candidate| candidate.contextual != context_coordinates)
                {
                    return Err(format!("row {row} has more than one assertion context").into());
                }
                write!(context_input, "{row}")?;
                write_fields(&mut context_input, &ids[..fact.start])?;
                write!(context_expected, "{row}")?;
                write_fields(&mut context_expected, &context_coordinates)?;
                row_context.insert(row, context_coordinates);
            }
            let gold = match reading {
                "early" => exact[0],
                "late" => exact[1],
                "ambiguous" | "missing" | "irrelevant" => None,
                _ => return Err("bad reading code".into()),
            };
            if split == "train" && gold.is_none() {
                return Err("TRAIN row has no gold onset".into());
            }
            if candidates.is_empty() {
                write!(packet, "{row} 0 0 0 0")?;
                for _ in 0..64 {
                    write!(packet, " 0")?;
                }
                writeln!(packet)?;
            }
            candidates.sort_by_key(|candidate| candidate.span.start);
            let mut masks = BTreeMap::new();
            for candidate in candidates {
                let address = candidate.span.start;
                let label = if split != "train" {
                    0
                } else if gold == Some(address) {
                    1
                } else {
                    -1
                };
                if candidate.completed_source_range
                    != [
                        u32::try_from(address)?,
                        u32::try_from(fence.ok_or("missing candidate fence")?)?,
                    ]
                {
                    return Err(
                        format!("row {row} candidate carries a different writer range").into(),
                    );
                }
                if masks.insert(address, candidate.narrow).is_some() {
                    return Err(format!("row {row} repeats a candidate address").into());
                }
                let [[fact_start, _], _] = fact_and_query.ok_or("candidate missing assertion")?;
                let end = fence.ok_or("candidate missing fence")?;
                write!(
                    local_input,
                    "{address} {} {fact_start} {end}",
                    candidate.span.end
                )?;
                write_fields(&mut local_input, &ids)?;
                let (first, rest) = candidate
                    .local
                    .split_first()
                    .ok_or("missing local coordinates")?;
                write!(local_expected, "{first}")?;
                write_fields(&mut local_expected, rest)?;
                writeln_packet(
                    &mut packet,
                    row,
                    address,
                    candidate.narrow,
                    label,
                    &candidate.local,
                    &candidate.contextual,
                )?;
            }
            grading.push(json!({
                "row": row, "reading": reading, "gold": gold, "onsets": exact, "fact_and_query": fact_and_query,
                "fence": fence, "candidates": masks, "original_ids": ids, "plausible_writer_ids": writer_witnesses,
                "legacy": legacy,
                "expected_reply": match reading { "early" => early, "late" => late, _ => "I don't know." },
            }));
            ordinary.push(json!({"row": row, "messages": [{"role": "user", "content": text}]}));
        }
        if split == "held" {
            for (&row, partner) in donors {
                if row == *partner
                    || donors.get(partner) != Some(&row)
                    || !row_context.contains_key(&row)
                {
                    return Err(format!("invalid held context swap {row}/{partner}").into());
                }
            }
            let mut swapped = b"D25ROWS1\n".to_vec();
            for line in std::str::from_utf8(&packet)?.lines().skip(1) {
                let fields: Vec<_> = line.split(' ').collect();
                let row: u32 = fields[0].parse()?;
                if let Some(partner) = donors.get(&row) {
                    if fields.len() != 69 {
                        return Err("malformed candidate row packet".into());
                    }
                    write!(swapped, "{}", fields[..37].join(" "))?;
                    write_fields(
                        &mut swapped,
                        row_context.get(partner).ok_or("missing swap partner")?,
                    )?;
                } else {
                    writeln!(swapped, "{line}")?;
                }
            }
            fs::write(out.join("held-swapped-context.rows"), swapped)?;
        }
        fs::write(out.join(format!("{split}.rows")), packet)?;
        for (kind, bytes) in [
            ("context-input", context_input),
            ("context-expected", context_expected),
            ("local-input", local_input),
            ("local-expected", local_expected),
        ] {
            fs::write(out.join(format!("{split}.{kind}.txt")), bytes)?;
        }
        fs::write(
            out.join(format!("{split}.grade.json")),
            serde_json::to_vec_pretty(&grading)?,
        )?;
        fs::write(
            out.join(format!("{split}.ordinary.json")),
            serde_json::to_vec_pretty(&ordinary)?,
        )?;
    }
    if !key.is_empty() {
        return Err("unused panel key rows".into());
    }
    write_local_tables(&model, &out.join("tables.bin"))?;
    Ok(())
}

fn write_local_tables(model: &IntegerDiscreteModel, path: &Path) -> Result<()> {
    let classes = model.frozen_role_replay()?.assertion_classes;
    let roots: Vec<_> = model.ordered_role_sites()?.packed_roots().collect();
    if classes.len() != roots.len() {
        return Err("D25 assertion class count differs from the root count".into());
    }
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    for (root, class) in roots.into_iter().zip(classes) {
        out.write_all(&root.to_le_bytes())?;
        out.write_all(&class.to_le_bytes())?;
    }
    out.write_all(
        &spiralcore_orbit_roots()
            .map(|root| root.map(|value| value as u8))
            .concat(),
    )?;
    out.write_all(spiralcore_signed_axes())?;
    Ok(())
}

fn write_fields(out: &mut Vec<u8>, fields: &[impl std::fmt::Display]) -> Result<()> {
    for field in fields {
        write!(out, " {field}")?;
    }
    writeln!(out)?;
    Ok(())
}

fn writeln_packet(
    packet: &mut Vec<u8>,
    row: u32,
    address: usize,
    narrow: bool,
    label: i8,
    local: &[i16; 32],
    contextual: &[i32; 32],
) -> Result<()> {
    write!(packet, "{row} {address} {} 1 {label}", u8::from(narrow))?;
    for value in local {
        write!(packet, " {value}")?;
    }
    for value in contextual {
        write!(packet, " {value}")?;
    }
    writeln!(packet)?;
    Ok(())
}

fn grade(key: &str, results: &str) -> Result<Value> {
    let key: Vec<Value> = serde_json::from_slice(&fs::read(key)?)?;
    let by_row = key
        .iter()
        .map(|row| Ok((row["row"].as_u64().ok_or("key missing row ID")?, row)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut seen = BTreeSet::new();
    let mut grades = Vec::new();
    for line in fs::read_to_string(results)?.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let [row, arm, reason, address] = fields.as_slice() else {
            return Err(format!("bad result row {line:?}").into());
        };
        let row: u64 = row.parse()?;
        let arm: u8 = arm.parse()?;
        if arm > 3 || !seen.insert((row, arm)) {
            return Err(format!("invalid or duplicate result {row}/{arm}").into());
        }
        let key = by_row.get(&row).ok_or("result row missing from key")?;
        let address: Option<usize> = if *address == "NA" {
            None
        } else {
            Some(address.parse()?)
        };
        match (reason, address) {
            (&"pick", Some(address)) => {
                let narrow = key["candidates"][address.to_string()]
                    .as_bool()
                    .ok_or("pick is not a wide candidate")?;
                if arm % 2 == 0 && !narrow {
                    return Err("narrow arm picked an address outside its mask".into());
                }
            }
            (&("empty" | "missing" | "nonpositive" | "tie"), None) => {}
            _ => return Err("inconsistent pick or abstention reason".into()),
        }
        let gold = key["gold"].as_u64().map(usize::try_from).transpose()?;
        let emitted = if let Some(start) = address {
            let ids: Vec<u32> = serde_json::from_value(key["original_ids"].clone())?;
            let fence = usize::try_from(key["fence"].as_u64().ok_or("pick missing fence")?)?;
            Some(writer(&ids, start, fence)?)
        } else {
            None
        };
        grades.push(json!({
            "row": row, "arm": arm, "reading": key["reading"], "reason": reason,
            "gold": gold, "selected": address, "forced_writer_ids": emitted,
            "correct": address == gold, "false_write": gold.is_none() && address.is_some(),
        }));
    }
    if seen.len() != by_row.len() * 4 {
        return Err("results omit one or more row/arm combinations".into());
    }
    Ok(json!(grades))
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, model, hash, tokenizer, out] if command == "prepare" => {
            fs::create_dir(out)?;
            let donors = BTreeMap::from_iter(
                [201, 203, 205, 207]
                    .into_iter()
                    .flat_map(|row| [(row, row + 1), (row + 1, row)]),
            );
            prepare(
                model, hash, tokenizer, Path::new(out),
                [include_str!("train.tsv"), include_str!("held.tsv"), include_str!("exposed.tsv")],
                include_str!("key.tsv"), &donors,
            )
        }
        [command, model, hash, tokenizer, fixture_dir, out] if command == "prepare-d26" => {
            let fixture_dir = Path::new(fixture_dir);
            let train = format!(
                "{}{}",
                include_str!("train.tsv"),
                fs::read_to_string(fixture_dir.join("train.tsv"))?
            );
            let held = fs::read_to_string(fixture_dir.join("held.tsv"))?;
            let exposed = concat!(include_str!("held.tsv"), include_str!("exposed.tsv"));
            let key = format!(
                "{}{}",
                include_str!("key.tsv"),
                fs::read_to_string(fixture_dir.join("key.tsv"))?
            );
            let mut donors = BTreeMap::new();
            let donor_source = fs::read_to_string(fixture_dir.join("held.donors.tsv"))?;
            for line in donor_source.lines() {
                let (row, partner) = line.split_once('\t').ok_or("invalid donor line")?;
                if donors.insert(row.parse()?, partner.parse()?).is_some() {
                    return Err("duplicate donor row".into());
                }
            }
            fs::create_dir(out)?;
            prepare(model, hash, tokenizer, Path::new(out), [&train, &held, exposed], &key, &donors)?;
            fs::write(Path::new(out).join("held.donors.tsv"), donor_source)?;
            Ok(())
        }
        [command, key, results] if command == "grade" => {
            println!("{}", serde_json::to_string_pretty(&grade(key, results)?)?);
            Ok(())
        }
        [command, model, hash, out] if command == "tables" => {
            let model = IntegerDiscreteModel::load(Path::new(model))?;
            if model.artifact_sha256() != hash {
                return Err("model pin mismatch".into());
            }
            write_local_tables(&model, Path::new(out))
        }
        _ => Err("usage: d25-contextual-source-panel prepare MODEL MODEL_SHA256 TOKENIZER NEW_OUT | prepare-d26 MODEL MODEL_SHA256 TOKENIZER FIXTURE_DIR NEW_OUT | grade GRADE_KEY RESULTS | tables MODEL MODEL_SHA256 NEW_OUTPUT_FILE".into()),
    }
}
