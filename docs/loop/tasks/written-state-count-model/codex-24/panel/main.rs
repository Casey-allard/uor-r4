mod prose;

use prose::{Panel, Role, PANELS, ROLE_PAIRS};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_integer::{
    discrete_d11::{spiralcore_orbit_roots, spiralcore_signed_axes, IntegerDiscreteModel},
    ordered_role::{RoleFraming, RoleMode},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PINS: [[&str; 2]; 2] = [
    [
        "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
        "b78c4ddc9dfca0b0c0f7f6fcab84ad6c09bdcb3c9c404a817dda7fa2548b3c33",
    ],
    [
        "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
        "4d6427dd27fd2fde428f647d25ca3fa8c0446c9720a4f262a3cdee51ffbbaa9d",
    ],
];

fn pinned(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let content = fs::read(path)?;
    if format!("{:x}", Sha256::digest(&content)) != expected {
        return Err(format!("asset digest mismatch for {}", path.display()).into());
    }
    Ok(content)
}

fn punctuation(tokenizer: &ByteBpeTokenizer, text: &str) -> Result<u32> {
    match tokenizer.encode(text).as_slice() {
        [id] => Ok(*id),
        _ => Err(format!("punctuation is not one token: {text}").into()),
    }
}

fn write_coordinates(mut out: impl Write, coordinates: &[i16]) -> Result<()> {
    for (column, coordinate) in coordinates.iter().enumerate() {
        if column > 0 {
            write!(out, " ")?;
        }
        write!(out, "{coordinate}")?;
    }
    writeln!(out)?;
    Ok(())
}

fn seal(
    panel: &Panel,
    model: &IntegerDiscreteModel,
    tokenizer: &ByteBpeTokenizer,
    out: &Path,
) -> Result<serde_json::Value> {
    let split = panel.split.name();
    let file = |name| -> Result<_> {
        Ok(BufWriter::new(File::create(
            out.join(format!("{split}.{name}")),
        )?))
    };
    let mut identity = file("identity.txt")?;
    let mut positional = file("positional.txt")?;
    let mut labels = file("labels.txt")?;
    let mut rows = file("rows.jsonl")?;
    let mut prompts = file("prompts.jsonl")?;
    let mut slots = file("slots.txt")?;
    let mut candidates = file("candidates.txt")?;
    let mut gold = file("writer-gold.jsonl")?;
    let sites = model.ordered_role_sites()?;
    let roots: Vec<_> = sites.packed_roots().collect();
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let framing = RoleFraming::from_tokenizer(tokenizer)?;
    let dot = punctuation(tokenizer, ".")?;
    let separator = punctuation(tokenizer, ";")?;
    let mut unique = BTreeSet::new();
    let mut role_counts = [0; 3];
    let mut support_role_counts = [[0; 3]; 2];
    let mut requested_set_sizes = [0; 3];
    let mut family_counts = BTreeMap::new();
    let mut max_abs = 0;
    let mut ambiguous_prompts = 0;
    for (group, name_pairs) in panel.names.iter().enumerate() {
        for form in 0..panel.forms {
            let include_ambiguous = panel.split.name() == "held" && form == 0;
            let roles = ROLE_PAIRS
                .iter()
                .copied()
                .chain(include_ambiguous.then_some([Role::Unspecified; 2]));
            for (arm, role_pair) in roles.enumerate() {
                let ambiguous = arm == ROLE_PAIRS.len();
                for name_order in 0..2 {
                    let names = &name_pairs[name_order];
                    for clause_order in 0..2 {
                        for fact_order in 0..2 {
                            let supported = panel.support(group);
                            let text = panel.text(
                                names,
                                group,
                                form,
                                role_pair,
                                ambiguous,
                                clause_order,
                                fact_order,
                            );
                            if !unique.insert(text.clone()) {
                                return Err(format!("duplicate {split} prompt: {text}").into());
                            }
                            let ids = encoder
                                .encode_assistant_prefix(&[Message {
                                    role: "user",
                                    content: &text,
                                }])
                                .tokens;
                            if tokenizer.decode(&ids) != format!("<|bos|>User: {text}\nAssistant:")
                            {
                                return Err("dialogue tape did not reproduce its text".into());
                            }
                            if ids.len() > 192 {
                                return Err("tape exceeds device packet limit".into());
                            }
                            let content = framing.content(&ids)?;
                            let full = sites.prompt(&ids, content.clone())?;
                            let terminal = content.end - 1;
                            if ids[terminal] != dot {
                                return Err("missing terminal punctuation".into());
                            }
                            let start = (content.start..terminal)
                                .rfind(|&at| ids[at] == dot)
                                .ok_or("missing query boundary")?
                                + 1;
                            let fences: Vec<_> = (start..terminal)
                                .filter(|&at| ids[at] == separator)
                                .collect();
                            let [fence] = fences.as_slice() else {
                                return Err("expected one clause separator".into());
                            };
                            let fence = *fence;
                            let clauses = [start..fence, fence + 1..terminal];
                            let base = format!("{split}-{group}-{form}");
                            let prompt_id =
                                format!("{base}-{arm}-{name_order}-{clause_order}-{fact_order}");
                            let controls = json!({
                                "family": if ambiguous { "ambiguous" } else { panel.family(form) },
                                "ambiguous": ambiguous,
                                "form": form,
                                "variant": arm,
                                "name_variant": name_order,
                                "clause_order": clause_order,
                                "fact_order": fact_order,
                                "polarity_pair": (arm < 2).then(|| format!("{base}-p-{name_order}-{clause_order}-{fact_order}")),
                                "clause_order_pair": format!("{base}-c-{arm}-{name_order}-{fact_order}"),
                                "fact_order_pair": format!("{base}-f-{arm}-{name_order}-{clause_order}"),
                                "name_variant_pair": format!("{base}-n-{arm}-{clause_order}-{fact_order}"),
                            });
                            let mut metadata = Vec::new();
                            let mut discovered = BTreeSet::new();
                            let mut requested = Vec::new();
                            for span in full
                                .candidates()
                                .filter(|span| span.start >= start && span.end <= terminal)
                            {
                                let word = tokenizer.decode(&ids[span.start..span.end]);
                                let Some(person) = word.strip_prefix(' ') else {
                                    continue;
                                };
                                let mut bytes = person.bytes();
                                if !bytes.next().is_some_and(|byte| byte.is_ascii_uppercase())
                                    || !bytes.all(|byte| byte.is_ascii_lowercase())
                                {
                                    continue;
                                }
                                let (clause_index, clause) = clauses
                                    .iter()
                                    .enumerate()
                                    .find(|(_, clause)| {
                                        clause.contains(&span.start) && span.end <= clause.end
                                    })
                                    .ok_or("discovered name straddles clause")?;
                                let assertion_span = full
                                    .candidates()
                                    .find(|head| {
                                        head.end < start
                                            && head.start > content.start
                                            && ids[head.start - 1] == dot
                                            && ids[head.start..head.end]
                                                == ids[span.start..span.end]
                                    })
                                    .map(|head| [head.start, head.end]);

                                // Author truth is consulted only after discovery and both projections.
                                let identity_feature =
                                    full.feature(RoleMode::Identity, span, clause.clone())?;
                                let positional_feature =
                                    full.feature(RoleMode::Positional, span, clause.clone())?;
                                let feature_positions =
                                    full.feature_positions(span, clause.clone())?;
                                let slot_roots = feature_positions.map(|position| {
                                    position.map_or(-1, |at| roots[ids[at] as usize] as i32)
                                });
                                max_abs = max_abs.max(
                                    identity_feature
                                        .iter()
                                        .chain(&positional_feature)
                                        .map(|coordinate| coordinate.abs())
                                        .max()
                                        .unwrap_or(0),
                                );
                                write_coordinates(&mut identity, &identity_feature)?;
                                write_coordinates(&mut positional, &positional_feature)?;
                                for (slot, root) in slot_roots.iter().enumerate() {
                                    if slot > 0 {
                                        write!(slots, " ")?;
                                    }
                                    write!(slots, "{root}")?;
                                }
                                writeln!(slots)?;
                                write!(
                                    candidates,
                                    "{} {} {} {}",
                                    span.start, span.end, clause.start, clause.end
                                )?;
                                for id in &ids {
                                    write!(candidates, " {id}")?;
                                }
                                writeln!(candidates)?;
                                let name_index =
                                    names.iter().position(|&name| name == person).ok_or_else(
                                        || format!("unexpected mention in {prompt_id}: {person}"),
                                    )?;
                                if !discovered.insert(name_index)
                                    || assertion_span.is_some() != supported[name_index]
                                {
                                    return Err(format!(
                                        "mention/source mismatch in {prompt_id}: {person}"
                                    )
                                    .into());
                                }
                                let role = role_pair[name_index];
                                if role == Role::Requested {
                                    requested.push(name_index);
                                }
                                role_counts[role as usize] += 1;
                                support_role_counts[usize::from(assertion_span.is_some())]
                                    [role as usize] += 1;
                                writeln!(labels, "{}", role as u8)?;
                                let mut row = controls.clone();
                                row["prompt_id"] = json!(prompt_id);
                                row["person"] = json!(person);
                                row["span"] = json!([span.start, span.end]);
                                row["clause"] = json!([clause.start, clause.end]);
                                row["clause_index"] = json!(clause_index);
                                row["assertion_supported"] = json!(assertion_span.is_some());
                                row["assertion_span"] = json!(assertion_span);
                                writeln!(rows, "{row}")?;
                                metadata.push(person.to_owned());
                            }
                            if discovered.len() != names.len() {
                                return Err(format!("missing mention in {prompt_id}: {text}; discovered {metadata:?}").into());
                            }
                            requested_set_sizes[requested.len()] += 1;
                            let answer = match requested.as_slice() {
                                [i] if supported[*i] => Some((names[*i], panel.items()[*i])),
                                _ => None,
                            };
                            writeln!(
                                gold,
                                "{}",
                                json!({
                                    "prompt_id": prompt_id,
                                    "requested_person": answer.map(|(person, _)| person),
                                    "answer_literal": answer.map(|(_, literal)| literal),
                                })
                            )?;
                            if ambiguous {
                                ambiguous_prompts += 1;
                            }
                            *family_counts
                                .entry(if ambiguous {
                                    "ambiguous"
                                } else {
                                    panel.family(form)
                                })
                                .or_insert(0) += 1;
                            writeln!(
                                prompts,
                                "{}",
                                json!({
                                    "prompt_id": prompt_id, "text": text, "prefix": ids,
                                    "clauses": clauses.map(|clause| [clause.start, clause.end]),
                                })
                            )?;
                        }
                    }
                }
            }
        }
    }
    for writer in [
        &mut identity,
        &mut positional,
        &mut labels,
        &mut rows,
        &mut prompts,
        &mut slots,
        &mut candidates,
        &mut gold,
    ] {
        writer.flush()?;
    }
    Ok(json!({
        "prompts": unique.len(), "candidates": role_counts.iter().sum::<usize>(),
        "role_counts": role_counts, "ambiguous_prompts": ambiguous_prompts, "support_role_counts": support_role_counts,
        "requested_set_sizes": requested_set_sizes, "family_counts": family_counts,
        "max_abs_coordinate": max_abs
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [seed, assets, out] = args.as_slice() else {
        return Err("usage: seal-clause-role-panel {1|2} ASSETS NEW_OUTPUT_DIRECTORY".into());
    };
    let seed: usize = seed.parse()?;
    let [tokenizer_pin, model_pin] = *PINS.get(seed.wrapping_sub(1)).ok_or("invalid seed")?;
    let assets = Path::new(assets);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&pinned(
        &assets.join(format!("tokenizer-seed{seed}.json")),
        tokenizer_pin,
    )?)
    .ok_or("invalid pinned tokenizer")?;
    let model_path = assets.join(format!("seed{seed}-paired.d11"));
    pinned(&model_path, model_pin)?;
    let model = IntegerDiscreteModel::load(&model_path)?;
    let out = Path::new(out);
    fs::create_dir(out)?;
    let sites = model.ordered_role_sites()?;
    let mut tables = BufWriter::new(File::create(out.join("tables.bin"))?);
    for (root, class) in sites.packed_roots().zip(sites.word_classes()) {
        tables.write_all(&root.to_le_bytes())?;
        tables.write_all(&u32::from(class).to_le_bytes())?;
    }
    tables.write_all(
        &spiralcore_orbit_roots()
            .map(|root| root.map(|value| value as u8))
            .concat(),
    )?;
    tables.write_all(spiralcore_signed_axes())?;
    tables.flush()?;
    let train_names: BTreeSet<_> = PANELS[0].names.iter().flatten().flatten().collect();
    if PANELS[1]
        .names
        .iter()
        .flatten()
        .flatten()
        .any(|name| train_names.contains(name))
    {
        return Err("TRAIN and held names overlap".into());
    }
    let mut census =
        json!({"seed": seed, "tokenizer_sha256": tokenizer_pin, "d11_sha256": model_pin});
    for panel in PANELS {
        census[panel.split.name()] = seal(&panel, &model, &tokenizer, out)?;
    }
    fs::write(
        out.join("census.json"),
        format!("{}\n", serde_json::to_string_pretty(&census)?),
    )?;
    Ok(())
}
