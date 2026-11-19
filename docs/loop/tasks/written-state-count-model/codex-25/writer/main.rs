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
const PARENT: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const ARMS: [SourceArm; 4] = [
    SourceArm::NarrowLocal,
    SourceArm::WideLocal,
    SourceArm::NarrowContextual,
    SourceArm::WideContextual,
];
const NAMES: [&str; 5] = ["ref", "nl", "wl", "nc", "wc"];
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

#[derive(Serialize)]
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

fn check_gpu(
    model: &IntegerDiscreteModel,
    key: &Key,
    arm: usize,
    bank: &SourceBank,
    gpu_scores: &mut GpuScores,
    gpu_picks: &mut GpuPicks,
) -> Result<(Option<u32>, Value)> {
    let (outcome, picked, traces) = match key.fact_and_query {
        Some([[fs, fe], [qs, qe]]) => {
            let (selection, traces) = model.contextual_source_trace(
                &key.original_ids,
                RoleSpan { start: fs, end: fe },
                RoleSpan { start: qs, end: qe },
            )?;
            (selection.outcome, selection.source, traces)
        }
        None => (SourceOutcome::Empty, None, Vec::new()),
    };
    let reason = match outcome {
        SourceOutcome::Pick => "pick",
        SourceOutcome::Empty => "empty",
        SourceOutcome::Nonpositive => "nonpositive",
        SourceOutcome::Tie => "tie",
    };
    if gpu_picks.remove(&(key.row, arm)) != Some((reason.to_owned(), picked)) {
        return Err(format!("GPU pick diverges at row {} arm {arm}", key.row).into());
    }
    let masks: BTreeMap<_, _> = traces.iter().map(|(c, s)| (s.source, c.narrow)).collect();
    if masks != key.candidates || masks.len() != traces.len() {
        return Err(format!("native candidate set diverges at row {} arm {arm}", key.row).into());
    }
    for (candidate, score) in &traces {
        if gpu_scores.remove(&(key.row, arm, score.source)) != Some(*score)
            || Some(candidate.completed_source_range[1]) != key.fence
            || candidate.completed_source_range[0] != score.source
        {
            return Err(format!(
                "GPU or writer range diverges at row {} arm {arm} source {}",
                key.row, score.source
            )
            .into());
        }
    }
    if traces.is_empty() {
        // The device represents an empty group with a masked all-zero row.
        // It is never a native candidate or a writable original position.
        let sentinel = SourceCandidate {
            span: RoleSpan { start: 0, end: 0 },
            narrow: false,
            local: [0; 32],
            contextual: [0; 32],
            completed_source_range: [0; 2],
        };
        let score = SourceScore {
            eligible: false,
            ..bank.score(ARMS[arm], &sentinel)
        };
        if gpu_scores.remove(&(key.row, arm, 0)) != Some(score) {
            return Err(format!("GPU empty sentinel diverges at row {} arm {arm}", key.row).into());
        }
    }
    Ok((picked, json!({ "reason": reason, "candidates": traces })))
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

fn outcome(key: &Key, tokenizer: &ByteBpeTokenizer, source: Option<u32>, replay: &Replay) -> Value {
    let tape = replay.generated_ids.as_slice();
    let visible = tape.strip_suffix(&[EOS]).unwrap_or(tape);
    let text = tokenizer.decode(visible);
    let expected = key
        .gold
        .and_then(|gold| key.plausible_writer_ids.get(&gold));
    let first = expected.and_then(|ids| {
        (0..ids.len().max(tape.len()))
            .find(|&i| ids.get(i) != tape.get(i))
            .map(|i| json!({"offset": i, "expected_id": ids.get(i), "generated_id": tape.get(i)}))
    });
    json!({
        "source": source,
        "gold_source": key.gold,
        "exact_source": key.gold.map(|gold| source == Some(gold)),
        "exact_through_eos": expected.map(|ids| ids == tape),
        "first_mismatch": first,
        "false_copy": key.gold.is_none() && source.is_some(),
        "literal_refusal": key.gold.is_none() && tape.ends_with(&[EOS])
            && text.trim() == key.expected_reply,
        "text": text,
        "trace": replay,
    })
}

fn checked_client(
    captured: &Path,
    model: &IntegerDiscreteModel,
    decision: &PairedCopyDecision,
    key: &Key,
    tokenizer: &ByteBpeTokenizer,
    replay: &Replay,
) -> Result<Value> {
    let record: Client = serde_json::from_slice(&fs::read(captured)?)?;
    let tape = replay.generated_ids.as_slice();
    let visible = tape.strip_suffix(&[EOS]);
    if record.artifact_sha256 != model.artifact_sha256()
        || record.model_schema != model.artifact_schema()
        || record.tokenizer_sha256 != TOKENIZER
        || record.prompt_ids != key.original_ids
        || !record.read_enabled
        || record.paired.role_words != decision.role_words
        || record.paired.choice != decision.choice
        || record.paired.supporting_facts != decision.supporting_facts
        || record.paired.complete_values != decision.complete_values
        || record.paired.values_disagree != decision.values_disagree
        || record.generated_ids != tape
        || record.copy_step_words != replay.copy_step_words
        || record.stop
            != if visible.is_some() {
                "eos"
            } else {
                "max_tokens"
            }
        || record.text != tokenizer.decode(visible.unwrap_or(tape))
    {
        return Err(format!("client tape, role, source or cursor differs in {captured:?}").into());
    }
    Ok(outcome(key, tokenizer, decision.choice.source, replay))
}

fn model_row(
    model: &IntegerDiscreteModel,
    key: &Key,
    ordinary: &Path,
    tokenizer: &ByteBpeTokenizer,
    role_words: Option<[u32; 8]>,
    native_replay: &mut impl FnMut(Option<u32>) -> Result<Replay>,
    gpu: Option<(Option<u32>, Value)>,
) -> Result<Value> {
    let ids = &key.original_ids;
    let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
    let (decision, witnesses) = model.paired_copy_witness(ids, roles, None)?;
    if decision.role_words != role_words || Some(roles.words()) != role_words {
        return Err(format!("D21 roles changed on row {}", key.row).into());
    }
    let ordinary = checked_client(
        ordinary,
        model,
        &decision,
        key,
        tokenizer,
        &native_replay(decision.choice.source)?,
    )?;
    let (all_facts, isolated) = if let Some([[fs, fe], [qs, qe]]) = key.fact_and_query {
        let (provided, facts) =
            model.paired_copy_witness(ids, roles, Some(RoleSpan { start: qs, end: qe }))?;
        let intended = facts
            .iter()
            .find(|witness| witness.fact_span == [fs, fe])
            .ok_or("supplied fact is not a bound assertion")?;
        let source = intended.assertion.source;
        if gpu.as_ref().is_some_and(|(pick, _)| *pick != source) {
            return Err(format!(
                "picked source diverges from supplied-fact binder on row {}",
                key.row
            )
            .into());
        }
        (
            json!({
                "decision": provided, "witnesses": facts,
                "writer": outcome(key, tokenizer, provided.choice.source, &native_replay(provided.choice.source)?),
            }),
            json!({
                "witness": intended, "gpu": gpu.as_ref().map(|(_, trace)| trace),
                "writer": outcome(key, tokenizer, source, &native_replay(source)?),
            }),
        )
    } else {
        (
            Value::Null,
            json!({
                "no_supplied_anchor": true,
                "gpu": gpu.as_ref().map(|(_, trace)| trace),
                "writer": outcome(key, tokenizer, None, &native_replay(None)?),
            }),
        )
    };
    Ok(json!({
        "decision": decision,
        "ordinary_assertions": witnesses,
        "ordinary": ordinary,
        "supplied_query": all_facts,
        "supplied_fact_and_query": isolated,
    }))
}

fn load_models(paths: &[PathBuf], bank_digest: &str) -> Result<Vec<IntegerDiscreteModel>> {
    let mut models = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        let model = IntegerDiscreteModel::load(path)?;
        if model.tokenizer_sha256() != Some(TOKENIZER) {
            return Err("artifact tokenizer pin mismatch".into());
        }
        if index == 0 {
            if model.artifact_sha256() != PARENT || model.contextual_source_arm().is_some() {
                return Err("wrong frozen D21 parent".into());
            }
        } else {
            let binding = model
                .contextual_source_bindings()
                .ok_or("missing source binding")?;
            if model.contextual_source_arm() != Some(ARMS[index - 1])
                || binding.parent_sha256 != PARENT
                || binding.bank_sha256 != bank_digest
            {
                return Err(
                    "carrier bank, arm or D21 parent differs from the sealed inputs".into(),
                );
            }
        }
        models.push(model);
    }
    let mut bindings = models
        .iter()
        .skip(1)
        .map(IntegerDiscreteModel::contextual_source_bindings);
    let first = bindings.next();
    if first.is_none() || bindings.any(|binding| Some(binding) != first) {
        return Err("source carrier TRAIN bindings disagree across arms".into());
    }
    Ok(models)
}

#[derive(Default, Serialize)]
struct Summary {
    supported: usize,
    exact_through_eos: usize,
    exact_source: usize,
    unsupported: usize,
    false_copies: usize,
    literal_refusals: usize,
    no_source: usize,
    first_mismatches: BTreeMap<String, usize>,
}

fn summarize(rows: &[Value], arm: &str, route: &str) -> Summary {
    let mut grade = Summary::default();
    for row in rows {
        let record = &row["arms"][arm][route];
        if record.is_null() {
            continue;
        }
        let result = if route == "ordinary" {
            record
        } else {
            &record["writer"]
        };
        if let Some(exact) = result["exact_through_eos"].as_bool() {
            grade.supported += 1;
            grade.exact_through_eos += usize::from(exact);
            grade.exact_source += usize::from(result["exact_source"] == true);
            if !exact {
                let mismatch = &result["first_mismatch"];
                let key = mismatch.to_string();
                *grade.first_mismatches.entry(key).or_default() += 1;
            }
        } else {
            grade.unsupported += 1;
            grade.false_copies += usize::from(result["false_copy"] == true);
            grade.literal_refusals += usize::from(result["literal_refusal"] == true);
        }
        grade.no_source += usize::from(result["source"].is_null());
    }
    grade
}

fn run(args: &[PathBuf]) -> Result<Value> {
    let [grade, trace, results, bank, tokenizer, ordinary, model_paths @ ..] = args else {
        return Err("usage: d25-contextual-source-writer GRADE GPU_TRACE GPU_RESULTS GPU_BANK TOKENIZER ORDINARY_DIR REF_D11 NL_D11 WL_D11 NC_D11 WC_D11".into());
    };
    if model_paths.len() != NAMES.len() {
        return Err(
            "supply the reference and all four contextual source carriers in arm order".into(),
        );
    }
    let key: Vec<Key> = serde_json::from_slice(&fs::read(grade)?)?;
    let mut gpu_trace = gpu_scores(trace)?;
    let mut gpu_result = gpu_picks(results)?;
    let raw_bank = fs::read(bank)?;
    let bank_digest = format!("{:x}", Sha256::digest(&raw_bank));
    let bank = SourceBank::from_bytes(&raw_bank)?;
    let tokenizer = fs::read(tokenizer)?;
    if format!("{:x}", Sha256::digest(&tokenizer)) != TOKENIZER {
        return Err("tokenizer pin mismatch".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer).ok_or("invalid tokenizer")?;
    let models = load_models(model_paths, &bank_digest)?;
    let parent = &models[0];
    let mut result = Vec::with_capacity(key.len());
    for key in key {
        if key
            .gold
            .is_some_and(|gold| !key.plausible_writer_ids.contains_key(&gold))
        {
            return Err(format!("missing exact gold tape for row {}", key.row).into());
        }
        let ids = &key.original_ids;
        let parent_decision = parent
            .paired_copy_decision(ids, true)?
            .ok_or("missing D21 decision")?;
        if parent
            .paired_copy_decision(ids, false)?
            .and_then(|d| d.choice.source)
            .is_some()
        {
            return Err("disabled parent read selected a source".into());
        }
        let fallback = replay(parent, ids, None)?;
        let mut native_replay = |source: Option<u32>| {
            if source.is_none() {
                return Ok(Replay {
                    generated_ids: fallback.generated_ids.clone(),
                    copy_step_words: fallback.copy_step_words.clone(),
                });
            }
            replay(parent, ids, source)
        };
        let mut controls = Vec::new();
        for onset in key.onsets.into_iter().flatten() {
            let tape = native_replay(Some(onset))?;
            if key.plausible_writer_ids.get(&onset) != Some(&tape.generated_ids) {
                return Err(
                    format!("forced writer diverges on row {}, source {onset}", key.row).into(),
                );
            }
            controls.push(outcome(&key, &tokenizer, Some(onset), &tape));
        }
        let mut arms = BTreeMap::new();
        for (index, (name, model)) in NAMES.into_iter().zip(&models).enumerate() {
            if model
                .paired_copy_decision(ids, false)?
                .and_then(|d| d.choice.source)
                .is_some()
            {
                return Err("disabled carrier read selected a source".into());
            }
            let gpu = if index == 0 {
                None
            } else {
                Some(check_gpu(
                    model,
                    &key,
                    index - 1,
                    &bank,
                    &mut gpu_trace,
                    &mut gpu_result,
                )?)
            };
            let file = ordinary.join(name).join(format!("{}.json", key.row));
            arms.insert(
                name,
                model_row(
                    model,
                    &key,
                    &file,
                    &tokenizer,
                    parent_decision.role_words,
                    &mut native_replay,
                    gpu,
                )?,
            );
        }
        result.push(json!({
            "row": key.row, "reading": key.reading, "original_ids": ids,
            "forced_legal_controls": controls,
            "source_disabled": outcome(&key, &tokenizer, None, &fallback),
            "arms": arms,
        }));
    }
    if !gpu_trace.is_empty() || !gpu_result.is_empty() {
        return Err("extra or unknown GPU observations".into());
    }
    Ok(json!({
        "schema": "uor-r4.d25-writer-grade/1",
        "max_tokens": MAX_TOKENS, "bindings": models[1].contextual_source_bindings(),
        "tokenizer_sha256": TOKENIZER,
        "artifacts": NAMES.into_iter().zip(&models)
            .map(|(name, model)| (name, model.artifact_sha256())).collect::<BTreeMap<_, _>>(),
        "summary": NAMES.into_iter().map(|arm| {
            let by_route = ["ordinary", "supplied_query", "supplied_fact_and_query"]
                .into_iter().map(|route| (route, summarize(&result, arm, route)))
                .collect::<BTreeMap<_, _>>();
            (arm, by_route)
        }).collect::<BTreeMap<_, _>>(),
        "rows": result,
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    println!("{}", serde_json::to_string(&run(&args)?)?);
    Ok(())
}
