#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use uor_r4_integer::{
    content_selector::Choice,
    discrete_d11::{
        contextual_source::{SourceArm, SourceBank, SourceCandidate, SourceOutcome, SourceScore},
        ExactCopyState, IntegerDiscreteModel, PairedCopyDecision, ROLE_COPY_STOPS,
    },
    paired_role::RoleSpan,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type GpuScores = BTreeMap<(u32, usize, u32), SourceScore>;
type GpuPicks = BTreeMap<(u32, usize), (String, Option<u32>)>;
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const EOS: u32 = 1;
const MAX_TOKENS: usize = 16;

#[derive(Deserialize)]
struct Key {
    row: u32,
    reading: String,
    gold: Option<u32>,
    onsets: [Option<u32>; 2],
    fact_and_query: Option<[[usize; 2]; 2]>,
    fence: Option<u32>,
    candidates: BTreeMap<u32, bool>,
    original_ids: Vec<u32>,
    plausible_writer_ids: BTreeMap<u32, Vec<u32>>,
    expected_reply: String,
}

#[derive(Deserialize)]
struct Client {
    artifact_sha256: String,
    tokenizer_sha256: String,
    model_schema: String,
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
    paired: ClientChoice,
    read_enabled: bool,
    text: String,
    stop: String,
}

#[derive(Deserialize)]
struct ClientChoice {
    choice: Choice,
    role_words: Option<[u32; 8]>,
    supporting_facts: u32,
    complete_values: u32,
    values_disagree: bool,
}

#[derive(Clone, Serialize)]
struct Replay {
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
}

fn optional<T: std::str::FromStr>(field: &str) -> Result<Option<T>>
where
    T::Err: Error + 'static,
{
    if field == "NA" {
        Ok(None)
    } else {
        Ok(Some(field.parse()?))
    }
}

fn gpu_scores(path: &Path) -> Result<GpuScores> {
    let mut trace = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let [row, source, arm, eligible, positive, negative, margin] = fields.as_slice() else {
            return Err(format!("invalid GPU trace row {line:?}").into());
        };
        let source = source.parse()?;
        let eligible = match *eligible {
            "0" => false,
            "1" => true,
            _ => return Err("invalid GPU eligibility flag".into()),
        };
        let score = SourceScore {
            source,
            eligible,
            nearest_positive: optional(positive)?,
            nearest_negative: optional(negative)?,
            margin: optional(margin)?,
        };
        if trace
            .insert((row.parse()?, arm.parse()?, source), score)
            .is_some()
        {
            return Err("duplicate GPU score".into());
        }
    }
    Ok(trace)
}

fn gpu_picks(path: &Path) -> Result<GpuPicks> {
    let mut picks = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let [row, arm, reason, source] = fields.as_slice() else {
            return Err(format!("invalid GPU result row {line:?}").into());
        };
        if picks
            .insert(
                (row.parse()?, arm.parse()?),
                ((*reason).into(), optional(source)?),
            )
            .is_some()
        {
            return Err("duplicate GPU result".into());
        }
    }
    Ok(picks)
}

fn replay(model: &IntegerDiscreteModel, original: &[u32], source: Option<u32>) -> Result<Replay> {
    let context_limit = model.shape().context as usize;
    let mut context = original.to_vec();
    let mut cursor = ExactCopyState::new(source, original.len().try_into()?);
    let mut trace = Replay {
        generated_ids: Vec::new(),
        copy_step_words: Vec::new(),
    };
    for _ in 0..MAX_TOKENS {
        let trial = cursor
            .write(original, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, EOS);
        let step = if trial.copied != 0 || trial.completed_span() {
            trial
        } else {
            let prediction = model.evaluate_prefix(&context)?.prediction;
            cursor
                .write(original, prediction, ROLE_COPY_STOPS)?
                .finish_with_eos(true, EOS)
        };
        cursor = step.state;
        trace.generated_ids.push(step.emitted);
        trace.copy_step_words.push(step.words());
        if step.emitted == EOS {
            break;
        }
        if context.len() == context_limit {
            context.remove(0);
        }
        context.push(step.emitted);
    }
    Ok(trace)
}

const PARENT: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const VIEWS: [&str; 3] = ["post", "relation", "recurrence"];
const SCORED_SPLITS: [&str; 3] = ["held", "exposed", "donor"];
const SCORE_ARMS: [SourceArm; 4] = [
    SourceArm::NarrowLocal,
    SourceArm::WideLocal,
    SourceArm::NarrowContextual,
    SourceArm::WideContextual,
];

#[derive(PartialEq, Eq)]
struct Features {
    address: u32,
    narrow: bool,
    wide: bool,
    local: Vec<i16>,
    context: Vec<i32>,
}

fn features(path: &Path) -> Result<BTreeMap<u32, Vec<Features>>> {
    let input = fs::read_to_string(path)?;
    let mut lines = input.lines();
    if lines.next() != Some("D25ROWS1") {
        return Err("wrong feature ABI".into());
    }
    let mut rows: BTreeMap<u32, Vec<Features>> = BTreeMap::new();
    for line in lines {
        let f: Vec<_> = line.split_ascii_whitespace().collect();
        if f.len() != 69 || f[4] != "0" {
            return Err("malformed blind feature row".into());
        }
        rows.entry(f[0].parse()?).or_default().push(Features {
            address: f[1].parse()?,
            narrow: f[2] == "1",
            wide: f[3] == "1",
            local: f[5..37]
                .iter()
                .map(|s| s.parse())
                .collect::<std::result::Result<_, _>>()?,
            context: f[37..]
                .iter()
                .map(|s| s.parse())
                .collect::<std::result::Result<_, _>>()?,
        });
    }
    Ok(rows)
}

fn check_donors(
    held: &BTreeMap<u32, Vec<Features>>,
    swapped: &BTreeMap<u32, Vec<Features>>,
    donors: &BTreeMap<u32, u32>,
) -> Result<()> {
    if held.keys().ne(swapped.keys()) || donors.keys().any(|row| !held.contains_key(row)) {
        return Err("donor coverage differs".into());
    }
    for (&row, native) in held {
        let donor = donors.get(&row).copied().unwrap_or(row);
        let donor_context = &held.get(&donor).ok_or("missing donor row")?[0].context;
        let modified = &swapped[&row];
        if native.len() != modified.len()
            || native.iter().zip(modified).any(|(a, b)| {
                a.address != b.address
                    || a.narrow != b.narrow
                    || a.wide != b.wide
                    || a.local != b.local
                    || &b.context != donor_context
            })
        {
            return Err(format!("donor altered non-context features for row {row}").into());
        }
    }
    Ok(())
}

fn checked_client(
    path: &Path,
    model: &IntegerDiscreteModel,
    decision: &PairedCopyDecision,
    key: &Key,
    tokenizer: &ByteBpeTokenizer,
    native: &Replay,
    enabled: bool,
) -> Result<()> {
    let record: Client = serde_json::from_slice(&fs::read(path)?)?;
    let tape = native.generated_ids.as_slice();
    let visible = tape.strip_suffix(&[EOS]);
    if record.artifact_sha256 != model.artifact_sha256()
        || record.model_schema != model.artifact_schema()
        || record.tokenizer_sha256 != TOKENIZER
        || record.prompt_ids != key.original_ids
        || record.read_enabled != enabled
        || record.paired.role_words != decision.role_words
        || record.paired.choice != decision.choice
        || record.paired.supporting_facts != decision.supporting_facts
        || record.paired.complete_values != decision.complete_values
        || record.paired.values_disagree != decision.values_disagree
        || record.generated_ids != tape
        || record.copy_step_words != native.copy_step_words
        || record.stop
            != if visible.is_some() {
                "eos"
            } else {
                "max_tokens"
            }
        || record.text != tokenizer.decode(visible.unwrap_or(tape))
    {
        return Err(format!("client tape, role, source or cursor differs in {path:?}").into());
    }
    Ok(())
}

fn outcome(
    key: &Key,
    tokenizer: &ByteBpeTokenizer,
    source: Option<u32>,
    replay: &Replay,
) -> Result<Value> {
    let tape = replay.generated_ids.as_slice();
    let visible = tape.strip_suffix(&[EOS]);
    let text = tokenizer.decode(visible.unwrap_or(tape));
    let expected = if let Some(gold) = key.gold {
        key.plausible_writer_ids
            .get(&gold)
            .ok_or("missing gold tape")?
            .clone()
    } else {
        let mut refused = tokenizer.encode(&format!(" {}", key.expected_reply.trim()));
        refused.push(EOS);
        refused
    };
    let first = (0..expected.len().max(tape.len()))
        .find(|&i| expected.get(i) != tape.get(i))
        .map(|i| json!({"offset": i, "expected_id": expected.get(i), "generated_id": tape.get(i)}));
    Ok(json!({
        "source": source,
        "gold_source": key.gold,
        "exact_gold_source": source == key.gold && key.gold.is_some(),
        "source_absent": source.is_none(),
        "expected_text": key.expected_reply,
        "expected_ids": expected,
        "exact_through_eos": expected == tape && visible.is_some(),
        "first_mismatch": first,
        "false_copy": key.gold.is_none() && source.is_some(),
        "literal_refusal": key.gold.is_none() && text.trim() == key.expected_reply && visible.is_some(),
        "text": text,
        "trace": replay,
    }))
}

fn check_device_row(
    key: &Key,
    scores: &mut GpuScores,
    picks: &mut GpuPicks,
    arm: usize,
    features: &[Features],
    bank: &SourceBank,
) -> Result<(String, Option<u32>, Vec<SourceScore>)> {
    let mut row_scores = Vec::new();
    let addresses: Vec<_> = if key.candidates.is_empty() {
        vec![0]
    } else {
        key.candidates.keys().copied().collect()
    };
    if addresses.len() != features.len() {
        return Err(format!("wrong feature count for row {}", key.row).into());
    }
    for (address, f) in addresses.into_iter().zip(features) {
        let s = scores
            .remove(&(key.row, arm, address))
            .ok_or("absent trace score")?;
        let allowed = key
            .candidates
            .get(&address)
            .is_some_and(|&narrow| arm % 2 == 1 || narrow);
        // The scorer reads only the start, narrow mask and coordinates.
        let candidate = SourceCandidate {
            span: RoleSpan {
                start: address as usize,
                end: address as usize + 1,
            },
            narrow: f.narrow,
            local: f.local.as_slice().try_into()?,
            contextual: f.context.as_slice().try_into()?,
            completed_source_range: [address, address + 1],
        };
        let mut native = bank.score(SCORE_ARMS[arm].clone(), &candidate);
        native.eligible &= f.wide;
        if s.eligible != allowed || f.address != address || native != s {
            return Err(format!(
                "invalid trace margin or eligibility for {} arm {arm} address {address}",
                key.row
            )
            .into());
        }
        row_scores.push(s);
    }
    let mut eligible = row_scores.iter().filter(|s| s.eligible).peekable();
    let (reason, expected) = if eligible.peek().is_none() {
        ("empty", None)
    } else {
        let mut best = 0;
        let mut best_sources = Vec::new();
        for score in eligible {
            let margin = score.margin.ok_or("missing distance from two-class bank")?;
            if margin > best {
                best = margin;
                best_sources.clear();
            }
            if margin == best && best > 0 {
                best_sources.push(score.source);
            }
        }
        match best_sources.as_slice() {
            [] => ("nonpositive", None),
            [source] => ("pick", Some(*source)),
            _ => ("tie", None),
        }
    };
    let (observed, source) = picks
        .remove(&(key.row, arm))
        .ok_or("absent device result")?;
    if observed != reason || source != expected {
        return Err(format!("device result differs from trace on {} arm {arm}", key.row).into());
    }
    Ok((observed, source, row_scores))
}

fn parse_map<K: std::str::FromStr + Ord, V: std::str::FromStr>(
    path: &Path,
) -> Result<BTreeMap<K, V>>
where
    K::Err: Error + 'static,
    V::Err: Error + 'static,
{
    let mut result = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let (key, value) = line.split_once('\t').ok_or("invalid two-column TSV")?;
        if result.insert(key.parse()?, value.parse()?).is_some() {
            return Err("duplicate TSV key".into());
        }
    }
    Ok(result)
}

fn grade(
    base: &Path,
    tokenizer: &Path,
    panel: &Path,
    stage: &Path,
    score: &Path,
    ordinary: &Path,
) -> Result<Value> {
    let model = IntegerDiscreteModel::load(base)?;
    let tokenizer = fs::read(tokenizer)?;
    if model.artifact_sha256() != PARENT || format!("{:x}", Sha256::digest(&tokenizer)) != TOKENIZER
    {
        return Err("parent or tokenizer identity mismatch".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer).ok_or("invalid tokenizer")?;
    let mut banks = BTreeMap::new();
    let mut native_banks = BTreeMap::new();
    let mut device_features = BTreeMap::new();
    let mut scores = BTreeMap::new();
    let mut picks = BTreeMap::new();
    let donors: BTreeMap<u32, u32> = parse_map(&panel.join("held.donors.tsv"))?;
    let strata: BTreeMap<u32, String> = parse_map(&panel.join("held.strata.tsv"))?;
    for view in VIEWS {
        let compare = |file: &str| -> Result<Vec<u8>> {
            let first = fs::read(score.join("primary").join(view).join(file))?;
            if fs::read(score.join("cold").join(view).join(file))? != first {
                return Err(format!("cold device bytes differ for {view}/{file}").into());
            }
            Ok(first)
        };
        let bank = compare("bank")?;
        native_banks.insert(view, SourceBank::from_bytes(&bank)?);
        let records = u32::from_le_bytes(bank[8..12].try_into()?);
        banks.insert(view, json!({"records": records, "bytes": bank.len(), "sha256": format!("{:x}", Sha256::digest(&bank))}));
        let held = features(&stage.join(view).join("held.rows"))?;
        let swapped = features(&stage.join(view).join("donor.rows"))?;
        check_donors(&held, &swapped, &donors)?;
        device_features.insert((view, "held"), held);
        device_features.insert((view, "donor"), swapped);
        device_features.insert(
            (view, "exposed"),
            features(&stage.join(view).join("exposed.rows"))?,
        );
        for split in SCORED_SPLITS {
            for ending in ["trace", "results"] {
                compare(&format!("{split}.{ending}"))?;
            }
            let primary = score.join("primary").join(view);
            scores.insert(
                (view, split),
                gpu_scores(&primary.join(format!("{split}.trace")))?,
            );
            picks.insert(
                (view, split),
                gpu_picks(&primary.join(format!("{split}.results")))?,
            );
        }
    }
    let held_features = features(&stage.join("post/held.rows"))?;
    let mut output = Vec::new();
    for (split, matches) in [
        ("held", &["held", "donor"][..]),
        ("exposed", &["exposed"][..]),
    ] {
        let keys: Vec<Key> =
            serde_json::from_slice(&fs::read(panel.join(format!("{split}.grade.json")))?)?;
        for key in keys {
            let ids = &key.original_ids;
            let ref_decision = model
                .paired_copy_decision(ids, true)?
                .ok_or("missing paired reference")?;
            let disabled_decision = model
                .paired_copy_decision(ids, false)?
                .ok_or("missing disabled decision")?;
            if disabled_decision.choice.source.is_some() {
                return Err("disabled selected a source".into());
            }
            let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
            let (observed, assertions) = model.paired_copy_witness(ids, roles, None)?;
            if observed.choice != ref_decision.choice
                || observed.role_words != ref_decision.role_words
                || Some(roles.words()) != ref_decision.role_words
            {
                return Err(format!(
                    "reference roles, witness or source differ on row {}",
                    key.row
                )
                .into());
            }
            let mut cache = BTreeMap::new();
            let mut native = |source| -> Result<Replay> {
                // Replay lazily for addresses selected by the device and each lawful control.
                if !cache.contains_key(&source) {
                    cache.insert(source, replay(&model, ids, source)?);
                }
                Ok(cache[&source].clone())
            };
            let for_client = |arm: &str| ordinary.join(arm).join(format!("{}.json", key.row));
            let disabled = {
                let tape = native(None)?;
                let filename = for_client("disabled");
                checked_client(
                    &filename,
                    &model,
                    &disabled_decision,
                    &key,
                    &tokenizer,
                    &tape,
                    false,
                )?;
                json!({"client_capture_verified": true, "writer": outcome(&key, &tokenizer, None, &tape)?})
            };
            let reference = {
                let tape = native(ref_decision.choice.source)?;
                checked_client(
                    &for_client("ref"),
                    &model,
                    &ref_decision,
                    &key,
                    &tokenizer,
                    &tape,
                    true,
                )?;
                outcome(&key, &tokenizer, ref_decision.choice.source, &tape)?
            };
            let parent_supplied = if let Some([[fs, fe], [qs, qe]]) = key.fact_and_query {
                let (supplied, facts) =
                    model.paired_copy_witness(ids, roles, Some(RoleSpan { start: qs, end: qe }))?;
                let target = facts
                    .iter()
                    .find(|item| item.fact_span == [fs, fe])
                    .ok_or("supplied target not bound")?;
                let source = target.assertion.source;
                json!({"decision": supplied, "facts": facts, "target": target, "writer": outcome(&key, &tokenizer, source, &native(source)?)?})
            } else {
                Value::Null
            };
            let mut forced = Vec::new();
            for source in key.onsets.into_iter().flatten() {
                let tape = native(Some(source))?;
                if key.plausible_writer_ids.get(&source) != Some(&tape.generated_ids) {
                    return Err(format!("forced source {source} differs on row {}", key.row).into());
                }
                forced.push(outcome(&key, &tokenizer, Some(source), &tape)?);
            }

            let mut local = None;
            let mut arms = BTreeMap::new();
            for &family in matches {
                let mut views = BTreeMap::new();
                for view in VIEWS {
                    for arm in 0..4 {
                        let (reason, source, margins) = check_device_row(
                            &key,
                            scores
                                .get_mut(&(view, family))
                                .ok_or("missing view score")?,
                            picks
                                .get_mut(&(view, family))
                                .ok_or("missing view result")?,
                            arm,
                            device_features[&(view, family)]
                                .get(&key.row)
                                .ok_or("missing feature row")?,
                            &native_banks[view],
                        )?;
                        let receipt = json!({"reason": reason, "margins": margins, "writer": outcome(&key, &tokenizer, source, &native(source)?)?});
                        if arm == 1 {
                            match &local {
                                None => local = Some(receipt),
                                Some(original) if original == &receipt => {}
                                _ => {
                                    return Err(format!(
                                        "wide local control changed on row {}",
                                        key.row
                                    )
                                    .into())
                                }
                            }
                        } else if arm == 3 {
                            views.insert(view, receipt);
                        }
                    }
                }
                arms.insert(family, views);
            }
            let donor_alignment = if let Some(&donor) = donors.get(&key.row) {
                let left = &held_features[&key.row];
                let right = &held_features[&donor];
                let aligned = left.len() == right.len()
                    && left.iter().zip(right).all(|(a, b)| {
                        a.address - left[0].address == b.address - right[0].address
                            && a.narrow == b.narrow
                            && a.wide == b.wide
                            && a.local == b.local
                    });
                Some(json!({"donor": donor, "relative_local_identical": aligned}))
            } else {
                None
            };
            output.push(json!({
                "row": key.row, "reading": key.reading, "split": split, "stratum": strata.get(&key.row),
                "fence": key.fence, "supplied_spans": key.fact_and_query, "donor_alignment": donor_alignment,
                "reference_decision": ref_decision, "reference_assertions": assertions,
                "reference": reference, "disabled": disabled, "parent_supplied": parent_supplied,
                "forced": forced, "local": local, "views": arms,
            }));
        }
    }
    if scores.values().any(|rows| !rows.is_empty()) || picks.values().any(|rows| !rows.is_empty()) {
        return Err("device packet contains extra rows".into());
    }
    Ok(
        json!({"bindings": {"parent_sha256": PARENT, "tokenizer_sha256": TOKENIZER, "banks": banks}, "rows": output}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    if args
        .first()
        .is_some_and(|command| command == Path::new("carrier"))
    {
        let [_, model, tokenizer, panel, stage, score, ordinary @ ..] = args.as_slice() else {
            return Err("usage: d26-source-writer carrier MODEL TOKENIZER PANEL SCORE_STAGE SCORE_RUN [ORDINARY_MODEL_DIR]".into());
        };
        if ordinary.len() > 1 {
            return Err("supply at most one ordinary model output directory".into());
        }
        let checked = check_carrier(
            model,
            tokenizer,
            panel,
            stage,
            score,
            ordinary.first().map(PathBuf::as_path),
        )?;
        serde_json::to_writer_pretty(std::io::stdout().lock(), &checked)?;
        println!();
        return Ok(());
    }
    let [base, tokenizer, panel, stage, score, ordinary] = args.as_slice() else {
        return Err(
            "usage: d26-source-writer PARENT TOKENIZER PANEL SCORE_STAGE SCORE_RUN ORDINARY".into(),
        );
    };
    serde_json::to_writer_pretty(
        std::io::stdout().lock(),
        &grade(base, tokenizer, panel, stage, score, ordinary)?,
    )?;
    println!();
    Ok(())
}

fn check_carrier(
    path: &Path,
    tokenizer_path: &Path,
    panel: &Path,
    stage: &Path,
    score: &Path,
    ordinary: Option<&Path>,
) -> Result<Value> {
    let model = IntegerDiscreteModel::load(path)?;
    let bindings = model
        .contextual_source_bindings()
        .ok_or("not a contextual source carrier")?;
    let bank = fs::read(score.join("primary/recurrence/bank"))?;
    let train = fs::read(stage.join("recurrence/train.rows"))?;
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    if model.contextual_source_arm() != Some(SourceArm::WideRecurrence)
        || bindings.parent_sha256 != PARENT
        || bindings.bank_sha256 != format!("{:x}", Sha256::digest(&bank))
        || bindings.train_sha256 != format!("{:x}", Sha256::digest(&train))
        || format!("{:x}", Sha256::digest(&tokenizer_bytes)) != TOKENIZER
    {
        return Err("carrier parent, bank, arm, TRAIN or tokenizer identity differs".into());
    }
    let bank = SourceBank::from_bytes(&bank)?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let mut rows = Vec::new();
    for split in ["held", "exposed"] {
        let keys: Vec<Key> =
            serde_json::from_slice(&fs::read(panel.join(format!("{split}.grade.json")))?)?;
        let features = features(&stage.join(format!("recurrence/{split}.rows")))?;
        let mut scores = gpu_scores(&score.join(format!("primary/recurrence/{split}.trace")))?;
        let mut picks = gpu_picks(&score.join(format!("primary/recurrence/{split}.results")))?;
        for key in keys {
            let ids = &key.original_ids;
            let device_features = &features[&key.row];
            let (reason, source, device) =
                check_device_row(&key, &mut scores, &mut picks, 3, device_features, &bank)?;
            let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
            let ref_decision = model
                .paired_copy_decision(ids, true)?
                .ok_or("missing paired reference")?;
            let (observed, assertions) = model.paired_copy_witness(ids, roles, None)?;
            if observed.choice != ref_decision.choice
                || observed.role_words != ref_decision.role_words
                || Some(roles.words()) != ref_decision.role_words
            {
                return Err(format!(
                    "ordinary witness and loaded roles differ on row {}",
                    key.row
                )
                .into());
            }
            let ordinary_tape = replay(&model, ids, ref_decision.choice.source)?;
            if let Some(directory) = ordinary {
                checked_client(
                    &directory.join(format!("{}.json", key.row)),
                    &model,
                    &ref_decision,
                    &key,
                    &tokenizer,
                    &ordinary_tape,
                    true,
                )?;
            }
            let native_supplied = if let Some([[fs, fe], [qs, qe]]) = key.fact_and_query {
                let fact = RoleSpan { start: fs, end: fe };
                let query = RoleSpan { start: qs, end: qe };
                let (selection, native) = model.contextual_source_trace(ids, fact, query)?;
                let native_reason = match selection.outcome {
                    SourceOutcome::Pick => "pick",
                    SourceOutcome::Empty => "empty",
                    SourceOutcome::Nonpositive => "nonpositive",
                    SourceOutcome::Tie => "tie",
                };
                if native_reason != reason
                    || selection.source != source
                    || native.len() != device_features.len()
                {
                    return Err(format!(
                        "loaded candidate set or source differs at row {}",
                        key.row
                    )
                    .into());
                }
                for (((feature, expected), (candidate, measured)), (&address, &narrow)) in
                    device_features
                        .iter()
                        .zip(&device)
                        .zip(&native)
                        .zip(&key.candidates)
                {
                    if measured != expected
                        || candidate.span.start as u32 != address
                        || candidate.narrow != narrow
                        || candidate.local.as_slice() != feature.local
                        || candidate.contextual.as_slice() != feature.context
                        || candidate.completed_source_range[0] != address
                        || Some(candidate.completed_source_range[1]) != key.fence
                    {
                        return Err(format!(
                            "loaded candidate words differ at row {}, address {address}",
                            key.row
                        )
                        .into());
                    }
                }
                let (supplied, witnesses) = model.paired_copy_witness(ids, roles, Some(query))?;
                let target = witnesses
                    .iter()
                    .find(|w| w.fact_span == [fs, fe])
                    .ok_or("target assertion absent")?;
                if target.assertion.source != source {
                    return Err(
                        format!("loaded target assertion differs at row {}", key.row).into(),
                    );
                }
                json!({
                    "candidate_words_and_distances_verified": native.len(),
                    "selection": selection,
                    "target": target,
                    "supplied_target_writer": outcome(&key, &tokenizer, source, &replay(&model, ids, source)?)?,
                    "all_assertions_decision": supplied,
                    "all_assertions": witnesses,
                    "all_assertions_writer": outcome(&key, &tokenizer, supplied.choice.source, &replay(&model, ids, supplied.choice.source)?)?,
                })
            } else {
                if source.is_some() {
                    return Err(format!(
                        "device selected source without anchor at row {}",
                        key.row
                    )
                    .into());
                }
                Value::Null
            };
            rows.push(json!({
                "row": key.row, "split": split, "reading": key.reading,
                "ordinary_decision": ref_decision,
                "ordinary_assertions": assertions,
                "ordinary": outcome(&key, &tokenizer, ref_decision.choice.source, &ordinary_tape)?,
                "native_supplied": native_supplied,
            }));
        }
        if scores.keys().any(|&(_, arm, _)| arm == 3) || picks.keys().any(|&(_, arm)| arm == 3) {
            return Err(format!("unexpected wide contextual observations in {split}").into());
        }
    }
    Ok(json!({
        "artifact_sha256": model.artifact_sha256(),
        "verified_client_captures": ordinary.is_some(),
        "rows": rows,
    }))
}
