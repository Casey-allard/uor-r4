//! Attribution diagnostic for current-state replies on `m2-fresh256`.
//!
//! Each panel row is generated without a forced prefix and after forcing the
//! correct first token, the complete correct value, the paired other value and
//! a deterministic shuffled value. The forced prefixes use the exact CPU served
//! forward and the normal continuation/stop contract.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.cpu-forced-copy-panel/1";
const ONE_USER_RENDERING: &str = "one-user-turn/v1";
const PER_TURN_RENDERING: &str = "per-turn/v1";
const BOS: u32 = 0;
const EOS: u32 = 1;
const STRATA: [(&str, usize); 5] = [
    ("length2", 64),
    ("length4", 64),
    ("length8", 64),
    ("reassert", 32),
    ("update", 32),
];

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-forced-copy-panel: {error}");
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Inputs {
    cases: Vec<InputCase>,
}

#[derive(Deserialize)]
struct InputCase {
    id: String,
    query_ids: Vec<u32>,
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
struct Segment {
    kind: String,
    event: u64,
    #[serde(default)]
    token_ids: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    cases: Vec<LabelCase>,
    membership_only: bool,
    protocol: String,
    schema: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelCase {
    answers: FrozenAnswers,
    id: String,
    pair_id: String,
}

#[derive(Deserialize)]
struct Plan {
    histories: Vec<History>,
}

#[derive(Deserialize)]
struct History {
    id: String,
    stratum: String,
}

#[derive(Deserialize)]
struct Manifest {
    files: BTreeMap<String, ManifestFile>,
    tokenizer_sha256: String,
    strata_rows: BTreeMap<String, usize>,
}

#[derive(Deserialize)]
struct ManifestFile {
    sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Arm {
    Free,
    FirstTarget,
    CorrectValue,
    OtherRelation,
    ShuffledValue,
}

const ARMS: [Arm; 5] = [
    Arm::Free,
    Arm::FirstTarget,
    Arm::CorrectValue,
    Arm::OtherRelation,
    Arm::ShuffledValue,
];

impl Arm {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::FirstTarget => "first-target",
            Self::CorrectValue => "correct-value",
            Self::OtherRelation => "other-relation",
            Self::ShuffledValue => "shuffled-value",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    Eos,
    Bos,
    Cap,
    Context,
}

struct Generated {
    tokens: Vec<u32>,
    stop: Stop,
}

struct ArmScoring<'a> {
    target_token: u32,
    target_words: &'a [String],
    other_words: &'a [String],
    tokenizer: &'a ByteBpeTokenizer,
    protocol: &'a DialogueProtocol,
}

#[derive(Clone)]
struct Reference {
    answer: String,
    ids: Vec<u32>,
}

#[derive(Clone)]
struct PreparedCase {
    id: String,
    pair_id: String,
    stratum: String,
    prompt: Vec<u32>,
    target_answers: FrozenAnswers,
    target: Reference,
    other: Reference,
    shuffled: Reference,
    target_value_in_prompt: bool,
}

#[derive(Clone, Serialize)]
struct ArmRow {
    id: String,
    pair_id: String,
    stratum: String,
    arm: Arm,
    prefix_len: usize,
    target_token: u32,
    first_token: Option<u32>,
    reply_ids: Vec<u32>,
    reply_text: String,
    stop: String,
    complete: bool,
    first_line_asked_value: bool,
    first_line_other_value: bool,
    target_value_in_prompt: bool,
    forced_prefix_retained: bool,
    prefix_retained_without_eos: bool,
}

#[derive(Clone, Default, Serialize)]
struct Tally {
    rows: usize,
    complete: usize,
    eos_stops: usize,
    bos_stops: usize,
    cap_stops: usize,
    context_stops: usize,
    first_token_target: usize,
    first_line_asked_value: usize,
    first_line_other_value: usize,
    first_line_neither_value: usize,
    target_value_in_prompt: usize,
    forced_prefix_retained: usize,
    prefix_retained_without_eos: usize,
    pairs: usize,
    pairs_both_complete_distinct: usize,
}

#[derive(Serialize)]
struct ArmReport {
    totals: Tally,
    per_stratum: BTreeMap<String, Tally>,
    rows: Vec<ArmRow>,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let flag = |name: &str| {
        args.get(name)
            .cloned()
            .ok_or_else(|| format!("--{name} is required"))
    };
    let out = PathBuf::from(flag("out")?);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    let cap: usize = args.get("reply-tokens").map_or(Ok(64), |value| {
        value
            .parse()
            .map_err(|_| "--reply-tokens is not a count".to_owned())
    })?;
    if cap == 0 || cap > 64 {
        return Err("--reply-tokens must be in 1..=64".to_owned());
    }
    let rendering = args
        .get("rendering")
        .map(String::as_str)
        .unwrap_or(ONE_USER_RENDERING);
    if !matches!(rendering, ONE_USER_RENDERING | PER_TURN_RENDERING) {
        return Err(format!(
            "--rendering wants {ONE_USER_RENDERING} or {PER_TURN_RENDERING}"
        ));
    }
    let random_seed: u64 = args.get("random-seed").map_or(Ok(1), |value| {
        value
            .parse()
            .map_err(|_| "--random-seed is not an integer".to_owned())
    })?;
    let required_hashes = [
        "expected-artifact-sha256",
        "expected-tokenizer-sha256",
        "expected-inputs-sha256",
        "expected-labels-sha256",
        "expected-plan-sha256",
        "expected-manifest-sha256",
    ];
    for name in required_hashes {
        validate_sha256(&flag(name)?)?;
    }

    let artifact =
        std::fs::read(PathBuf::from(flag("artifact")?)).map_err(|error| error.to_string())?;
    let tokenizer_bytes =
        std::fs::read(PathBuf::from(flag("tokenizer")?)).map_err(|error| error.to_string())?;
    let inputs_bytes =
        std::fs::read(PathBuf::from(flag("inputs")?)).map_err(|error| error.to_string())?;
    let labels_bytes =
        std::fs::read(PathBuf::from(flag("labels")?)).map_err(|error| error.to_string())?;
    let plan_bytes =
        std::fs::read(PathBuf::from(flag("plan")?)).map_err(|error| error.to_string())?;
    let manifest_bytes =
        std::fs::read(PathBuf::from(flag("manifest")?)).map_err(|error| error.to_string())?;
    let identities = [
        (
            "artifact",
            &artifact,
            args["expected-artifact-sha256"].as_str(),
        ),
        (
            "tokenizer",
            &tokenizer_bytes,
            args["expected-tokenizer-sha256"].as_str(),
        ),
        (
            "inputs",
            &inputs_bytes,
            args["expected-inputs-sha256"].as_str(),
        ),
        (
            "labels",
            &labels_bytes,
            args["expected-labels-sha256"].as_str(),
        ),
        ("plan", &plan_bytes, args["expected-plan-sha256"].as_str()),
        (
            "manifest",
            &manifest_bytes,
            args["expected-manifest-sha256"].as_str(),
        ),
    ];
    for (name, bytes, expected) in identities {
        verify_sha256_identity(name, bytes, expected)?;
    }

    let manifest: Manifest = parse_json("manifest", &manifest_bytes)?;
    verify_manifest_files(
        &manifest,
        &[
            ("inputs.json", &inputs_bytes),
            ("labels.json", &labels_bytes),
            ("plan.json", &plan_bytes),
        ],
    )?;
    if manifest.tokenizer_sha256 != args["expected-tokenizer-sha256"] {
        return Err("panel manifest tokenizer_sha256 differs".to_owned());
    }
    let expected_strata: BTreeMap<String, usize> = STRATA
        .iter()
        .map(|(name, rows)| ((*name).to_owned(), *rows))
        .collect();
    if manifest.strata_rows != expected_strata {
        return Err("panel manifest strata_rows differs".to_owned());
    }

    let inputs: Inputs = parse_json("inputs", &inputs_bytes)?;
    let labels: Labels = parse_json("labels", &labels_bytes)?;
    let plan: Plan = parse_json("plan", &plan_bytes)?;
    if labels.protocol != SCHEMA_V2
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || !labels.membership_only
    {
        return Err("panel label contract differs".to_owned());
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    if !model.shape().head_nearest() {
        return Err(
            "artifact serves the writer head; this probe needs the nearest head".to_owned(),
        );
    }
    let base_forward =
        CpuFbsForward::from_artifact(&artifact).map_err(|error| error.to_string())?;
    let context = base_forward.context_length();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "tokenizer JSON is not a supported byte-level BPE tokenizer".to_owned())?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;

    let prepared = prepare_cases(
        &inputs,
        &labels,
        &plan,
        &tokenizer,
        &encoder,
        rendering,
        random_seed,
    )?;
    if prepared.len() != 256 {
        return Err(format!(
            "prepared {} panel rows, expected 256",
            prepared.len()
        ));
    }
    let started = Instant::now();
    let rows = prepared
        .par_iter()
        .map(|case| run_case(&base_forward, case, cap, &tokenizer, &protocol))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let wall = started.elapsed();
    let arms = summarize_arms(&rows, &protocol)?;
    let paired = summarize_pairs(&rows)?;
    let report = serde_json::json!({
        "schema": SCHEMA,
        "inputs": {
            "artifact_sha256": args["expected-artifact-sha256"],
            "tokenizer_sha256": args["expected-tokenizer-sha256"],
            "inputs_sha256": args["expected-inputs-sha256"],
            "labels_sha256": args["expected-labels-sha256"],
            "plan_sha256": args["expected-plan-sha256"],
            "manifest_sha256": args["expected-manifest-sha256"],
        },
        "settings": {
            "rendering": rendering,
            "protocol": SCHEMA_V2,
            "head": "nearest",
            "reply_tokens": cap,
            "random_seed": random_seed,
            "context_length": context,
            "eos_id": EOS,
            "bos_id": BOS,
            "threads": rayon::current_num_threads(),
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
        },
        "arms": arms,
        "paired": paired,
        "wall_milliseconds": wall.as_millis().clamp(1, u128::from(u64::MAX)),
        "decode_milliseconds": wall.as_millis().clamp(1, u128::from(u64::MAX)),
        "peak_rss_kib": peak_rss_kib()?,
        "claim": "CPU development-research forced-prefix attribution; not a capability bar, trained update, useful-chat result or GPU qualification"
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "sha256": uor_r4_discrete::corpus::sha256(&bytes),
            "totals": report["arms"].as_object().map(|arms| {
                arms.iter().map(|(name, value)| (name.clone(), value["totals"].clone())).collect::<BTreeMap<_, _>>()
            }),
        })
    );
    Ok(())
}

fn prepare_cases(
    inputs: &Inputs,
    labels: &Labels,
    plan: &Plan,
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    rendering: &str,
    seed: u64,
) -> Result<Vec<PreparedCase>> {
    if inputs.cases.len() != labels.cases.len() || labels.cases.len() != 256 {
        return Err("inputs and labels must each hold 256 cases".to_owned());
    }
    let by_id = panel_identity_map(inputs, labels)?;
    let strata = plan
        .histories
        .iter()
        .map(|history| {
            (
                format!("fresh-{}", history.id),
                history.stratum.split('/').next().unwrap_or("").to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if strata.len() != 128 {
        return Err("panel plan must hold 128 unique histories".to_owned());
    }
    let mut references = Vec::with_capacity(labels.cases.len());
    for label in &labels.cases {
        label
            .answers
            .validate()
            .map_err(|error| error.to_string())?;
        references.push(reference(encoder, &label.answers)?);
    }
    let pairs = label_pair_map(labels)?;
    let shuffled = distinct_shuffled_indices(&references, seed)?;

    let mut prepared = Vec::with_capacity(labels.cases.len());
    for (index, label) in labels.cases.iter().enumerate() {
        let case = by_id
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no input case", label.id))?;
        let stratum = strata
            .get(&label.pair_id)
            .ok_or_else(|| format!("{}: no plan history", label.pair_id))?
            .clone();
        let (_, prompt) = render(tokenizer, encoder, case, rendering)?;
        let pair = &pairs[label.pair_id.as_str()];
        let other = if pair[0] == index { pair[1] } else { pair[0] };
        let shuffled_index = shuffled[index];
        let prompt_text = tokenizer.decode(&prompt);
        let target_words = value_words(&references[index].answer);
        prepared.push(PreparedCase {
            id: label.id.clone(),
            pair_id: label.pair_id.clone(),
            stratum,
            prompt,
            target_answers: label.answers.clone(),
            target: references[index].clone(),
            other: references[other].clone(),
            shuffled: references[shuffled_index].clone(),
            target_value_in_prompt: contains_words(&words(&prompt_text), &target_words),
        });
    }
    Ok(prepared)
}

fn panel_identity_map<'a>(
    inputs: &'a Inputs,
    labels: &'a Labels,
) -> Result<BTreeMap<&'a str, &'a InputCase>> {
    let by_id = inputs
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let label_ids = labels
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    if by_id.len() != inputs.cases.len() || label_ids.len() != labels.cases.len() {
        return Err("panel case identities are not unique".to_owned());
    }
    Ok(by_id)
}

fn label_pair_map(labels: &Labels) -> Result<BTreeMap<&str, Vec<usize>>> {
    let mut pairs = BTreeMap::<&str, Vec<usize>>::new();
    for (index, label) in labels.cases.iter().enumerate() {
        pairs.entry(label.pair_id.as_str()).or_default().push(index);
    }
    if pairs.len() != 128 || pairs.values().any(|indices| indices.len() != 2) {
        return Err("panel labels must form 128 two-row pairs".to_owned());
    }
    Ok(pairs)
}

fn reference(encoder: &DialogueEncoder<'_>, answers: &FrozenAnswers) -> Result<Reference> {
    let answer = answers
        .accepted
        .iter()
        .min_by_key(|answer| answer.len())
        .ok_or_else(|| "frozen answer set is empty".to_owned())?;
    let encoded = encoder.encode_assistant_suffix(
        &[Message {
            role: "assistant",
            content: answer,
        }],
        true,
    );
    let ids = encoded
        .tokens
        .iter()
        .zip(&encoded.response_mask)
        .filter_map(|(&token, &mask)| (mask == 1).then_some(token))
        .collect::<Vec<_>>();
    if ids.len() < 2 || ids.last().copied() != Some(EOS) || ids[..ids.len() - 1].contains(&EOS) {
        return Err(format!(
            "reference answer {answer:?} has an invalid scored suffix"
        ));
    }
    Ok(Reference {
        answer: answer.clone(),
        ids,
    })
}

fn generate_with_prefix(
    forward: &mut CpuFbsForward,
    prompt: &[u32],
    prefix: &[u32],
    cap: usize,
) -> Result<Generated> {
    let context = forward.context_length();
    if prompt.is_empty() {
        return Err("empty prompt".to_owned());
    }
    if prompt.len() > context {
        return Ok(Generated {
            tokens: Vec::new(),
            stop: Stop::Context,
        });
    }
    if prefix.len() > cap || prompt.len().saturating_add(prefix.len()) >= context {
        return Err("forced prefix passes the reply cap or context".to_owned());
    }
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Nearest)
            .map_err(|error| error.to_string())?;
        if index + 1 == prompt.len() {
            next = Some(step.token);
        }
    }
    let mut next = next.ok_or_else(|| "empty prompt".to_owned())?;
    for &token in prefix {
        next = forward
            .step(token, CpuFbsHead::Nearest)
            .map_err(|error| error.to_string())?
            .token;
    }
    continue_generation(next, prefix.to_vec(), prompt.len(), cap, context, |token| {
        forward
            .step(token, CpuFbsHead::Nearest)
            .map(|step| step.token)
            .map_err(|error| error.to_string())
    })
}

fn continue_generation(
    mut next: u32,
    mut tokens: Vec<u32>,
    prompt: usize,
    cap: usize,
    context: usize,
    mut advance: impl FnMut(u32) -> Result<u32>,
) -> Result<Generated> {
    if tokens.len() >= cap {
        return Ok(Generated {
            tokens,
            stop: Stop::Cap,
        });
    }
    loop {
        let stop = stop_for(next, tokens.len(), prompt, cap, context);
        if !matches!(stop, Some(Stop::Eos | Stop::Bos)) {
            tokens.push(next);
        }
        if let Some(stop) = stop {
            return Ok(Generated { tokens, stop });
        }
        next = advance(next)?;
    }
}

fn stop_for(token: u32, kept: usize, prompt: usize, cap: usize, context: usize) -> Option<Stop> {
    if token == EOS {
        return Some(Stop::Eos);
    }
    if token == BOS {
        return Some(Stop::Bos);
    }
    let kept = kept + 1;
    if kept >= cap {
        Some(Stop::Cap)
    } else if prompt + kept >= context {
        Some(Stop::Context)
    } else {
        None
    }
}

fn run_case(
    base: &CpuFbsForward,
    case: &PreparedCase,
    cap: usize,
    tokenizer: &ByteBpeTokenizer,
    protocol: &DialogueProtocol,
) -> Result<Vec<ArmRow>> {
    let target_token = *case
        .target
        .ids
        .first()
        .ok_or_else(|| "target reference is empty".to_owned())?;
    let prefixes = arm_prefixes(case)?;
    let target_words = value_words(&case.target.answer);
    let other_words = value_words(&case.other.answer);
    let scoring = ArmScoring {
        target_token,
        target_words: &target_words,
        other_words: &other_words,
        tokenizer,
        protocol,
    };
    prefixes
        .into_iter()
        .map(|(arm, prefix)| {
            let mut forward = base.clone();
            let generated = generate_with_prefix(&mut forward, &case.prompt, &prefix, cap)?;
            arm_row(case, arm, &prefix, generated, &scoring)
        })
        .collect()
}

fn arm_prefixes(case: &PreparedCase) -> Result<Vec<(Arm, Vec<u32>)>> {
    let content = |reference: &Reference| -> Result<Vec<u32>> {
        let (last, content) = reference
            .ids
            .split_last()
            .ok_or_else(|| format!("{:?} reference is empty", reference.answer))?;
        if *last != EOS || content.is_empty() || content.contains(&EOS) {
            return Err(format!(
                "{:?} reference does not hold one terminal EOS",
                reference.answer
            ));
        }
        Ok(content.to_vec())
    };
    let target_token = *case
        .target
        .ids
        .first()
        .ok_or_else(|| "target reference is empty".to_owned())?;
    Ok(vec![
        (Arm::Free, Vec::new()),
        (Arm::FirstTarget, vec![target_token]),
        (Arm::CorrectValue, content(&case.target)?),
        (Arm::OtherRelation, content(&case.other)?),
        (Arm::ShuffledValue, content(&case.shuffled)?),
    ])
}

fn arm_row(
    case: &PreparedCase,
    arm: Arm,
    prefix: &[u32],
    generated: Generated,
    scoring: &ArmScoring<'_>,
) -> Result<ArmRow> {
    let stop = generated.stop;
    let reply_bytes = scoring.tokenizer.decode_bytes(&generated.tokens);
    let complete = reply_is_complete(&reply_bytes, stop, &case.target_answers, scoring.protocol);
    let reply_text = String::from_utf8_lossy(&reply_bytes).into_owned();
    let first_line = reply_text.split('\n').next().unwrap_or("");
    let first_line_words = words(first_line);
    let first_line_asked_value = contains_words(&first_line_words, scoring.target_words);
    let first_line_other_value = contains_words(&first_line_words, scoring.other_words);
    let forced_prefix_retained = !prefix.is_empty() && generated.tokens.starts_with(prefix);
    Ok(ArmRow {
        id: case.id.clone(),
        pair_id: case.pair_id.clone(),
        stratum: case.stratum.clone(),
        arm,
        prefix_len: prefix.len(),
        target_token: scoring.target_token,
        first_token: generated.tokens.first().copied(),
        reply_ids: generated.tokens,
        reply_text,
        stop: match stop {
            Stop::Eos => "eos",
            Stop::Bos => "bos",
            Stop::Cap => "cap",
            Stop::Context => "context",
        }
        .to_owned(),
        complete,
        first_line_asked_value,
        first_line_other_value,
        target_value_in_prompt: case.target_value_in_prompt,
        forced_prefix_retained,
        prefix_retained_without_eos: forced_prefix_retained && stop != Stop::Eos,
    })
}

fn reply_is_complete(
    reply_bytes: &[u8],
    stop: Stop,
    answers: &FrozenAnswers,
    protocol: &DialogueProtocol,
) -> bool {
    std::str::from_utf8(reply_bytes)
        .is_ok_and(|text| stop == Stop::Eos && answers.accepts(protocol.reply_text(text)))
}

fn summarize_arms(
    rows: &[ArmRow],
    protocol: &DialogueProtocol,
) -> Result<BTreeMap<String, ArmReport>> {
    let mut output = BTreeMap::new();
    for arm in ARMS {
        let arm_rows = rows
            .iter()
            .filter(|row| row.arm == arm)
            .cloned()
            .collect::<Vec<_>>();
        if arm_rows.len() != 256 {
            return Err(format!(
                "{} arm has {} rows, expected 256",
                arm.as_str(),
                arm_rows.len()
            ));
        }
        let mut per_stratum = BTreeMap::<String, Tally>::new();
        for row in &arm_rows {
            add_tally(per_stratum.entry(row.stratum.clone()).or_default(), row);
        }
        add_pair_tallies(&mut per_stratum, &arm_rows, protocol)?;
        let observed = per_stratum
            .iter()
            .map(|(stratum, tally)| (stratum.clone(), tally.rows))
            .collect::<BTreeMap<_, _>>();
        let expected = STRATA
            .iter()
            .map(|(name, rows)| ((*name).to_owned(), *rows))
            .collect::<BTreeMap<_, _>>();
        if observed != expected {
            return Err(format!("{} arm strata differ", arm.as_str()));
        }
        let mut totals = Tally::default();
        for tally in per_stratum.values() {
            add_tallies(&mut totals, tally);
        }
        output.insert(
            arm.as_str().to_owned(),
            ArmReport {
                totals,
                per_stratum,
                rows: arm_rows,
            },
        );
    }
    Ok(output)
}

fn add_tally(tally: &mut Tally, row: &ArmRow) {
    tally.rows += 1;
    tally.complete += usize::from(row.complete);
    match row.stop.as_str() {
        "eos" => tally.eos_stops += 1,
        "bos" => tally.bos_stops += 1,
        "cap" => tally.cap_stops += 1,
        "context" => tally.context_stops += 1,
        _ => unreachable!("generated stop was validated"),
    }
    tally.first_token_target += usize::from(row.first_token == Some(row.target_token));
    tally.first_line_asked_value += usize::from(row.first_line_asked_value);
    tally.first_line_other_value += usize::from(row.first_line_other_value);
    tally.first_line_neither_value +=
        usize::from(!row.first_line_asked_value && !row.first_line_other_value);
    tally.forced_prefix_retained += usize::from(row.forced_prefix_retained);
    tally.prefix_retained_without_eos += usize::from(row.prefix_retained_without_eos);
    tally.target_value_in_prompt += usize::from(row.target_value_in_prompt);
}

fn add_tallies(total: &mut Tally, part: &Tally) {
    total.rows += part.rows;
    total.complete += part.complete;
    total.eos_stops += part.eos_stops;
    total.bos_stops += part.bos_stops;
    total.cap_stops += part.cap_stops;
    total.context_stops += part.context_stops;
    total.first_token_target += part.first_token_target;
    total.first_line_asked_value += part.first_line_asked_value;
    total.first_line_other_value += part.first_line_other_value;
    total.first_line_neither_value += part.first_line_neither_value;
    total.forced_prefix_retained += part.forced_prefix_retained;
    total.prefix_retained_without_eos += part.prefix_retained_without_eos;
    total.target_value_in_prompt += part.target_value_in_prompt;
    total.pairs += part.pairs;
    total.pairs_both_complete_distinct += part.pairs_both_complete_distinct;
}

fn add_pair_tallies(
    per_stratum: &mut BTreeMap<String, Tally>,
    rows: &[ArmRow],
    protocol: &DialogueProtocol,
) -> Result<()> {
    let mut pairs = BTreeMap::<&str, Vec<&ArmRow>>::new();
    for row in rows {
        pairs.entry(row.pair_id.as_str()).or_default().push(row);
    }
    for (pair, members) in pairs {
        let [a, b] = members.as_slice() else {
            return Err(format!("pair {pair} does not hold two rows"));
        };
        if a.stratum != b.stratum {
            return Err(format!("pair {pair} crosses strata"));
        }
        let tally = per_stratum.entry(a.stratum.clone()).or_default();
        tally.pairs += 1;
        tally.pairs_both_complete_distinct += usize::from(
            a.complete
                && b.complete
                && protocol.reply_text(&a.reply_text) != protocol.reply_text(&b.reply_text),
        );
    }
    Ok(())
}

fn summarize_pairs(rows: &[ArmRow]) -> Result<serde_json::Value> {
    let by_id = rows
        .iter()
        .map(|row| ((row.id.as_str(), row.arm), row.complete))
        .collect::<BTreeMap<_, _>>();
    let comparisons = [
        (Arm::FirstTarget, Arm::Free, true),
        (Arm::CorrectValue, Arm::FirstTarget, true),
        (Arm::CorrectValue, Arm::OtherRelation, false),
        (Arm::CorrectValue, Arm::ShuffledValue, false),
    ];
    let mut overall = BTreeMap::new();
    let mut per_stratum = BTreeMap::<String, BTreeMap<String, serde_json::Value>>::new();
    for (candidate_arm, control_arm, inferential) in comparisons {
        let mut all = Vec::new();
        let mut strata = BTreeMap::<String, Vec<(bool, bool)>>::new();
        for row in rows.iter().filter(|row| row.arm == candidate_arm) {
            let Some(&candidate) = by_id.get(&(row.id.as_str(), candidate_arm)) else {
                return Err(format!("{} is missing {}", row.id, candidate_arm.as_str()));
            };
            let Some(&control) = by_id.get(&(row.id.as_str(), control_arm)) else {
                return Err(format!("{} is missing {}", row.id, control_arm.as_str()));
            };
            all.push((candidate, control));
            strata
                .entry(row.stratum.clone())
                .or_default()
                .push((candidate, control));
        }
        let key = format!("{}-vs-{}", candidate_arm.as_str(), control_arm.as_str());
        overall.insert(key.clone(), paired_binary(&all, inferential)?);
        for (stratum, pairs) in strata {
            per_stratum
                .entry(stratum)
                .or_default()
                .insert(key.clone(), paired_binary(&pairs, inferential)?);
        }
    }
    Ok(serde_json::json!({
        "overall": overall,
        "per_stratum": per_stratum,
    }))
}

fn paired_binary(rows: &[(bool, bool)], inferential: bool) -> Result<serde_json::Value> {
    if rows.is_empty() {
        return Err("paired comparison requires rows".to_owned());
    }
    let candidate_only = rows.iter().filter(|(a, b)| *a && !*b).count();
    let control_only = rows.iter().filter(|(a, b)| !*a && *b).count();
    let discordant = candidate_only + control_only;
    let threshold = std::cmp::max(2, (2.0 * (discordant as f64).sqrt()).ceil() as u64);
    let gain = candidate_only as i64 - control_only as i64;
    Ok(serde_json::json!({
        "rows": rows.len(),
        "candidate_complete": rows.iter().filter(|(a, _)| *a).count(),
        "control_complete": rows.iter().filter(|(_, b)| *b).count(),
        "candidate_only": candidate_only,
        "control_only": control_only,
        "discordant": discordant,
        "gain": gain,
        "threshold": threshold,
        "comparison_kind": if inferential { "matched-treatment" } else { "identity-control" },
        "inferential": inferential,
        "passes": inferential.then_some(gain >= i64::try_from(threshold).unwrap_or(i64::MAX)),
    }))
}

fn render(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    case: &InputCase,
    rendering: &str,
) -> Result<(String, Vec<u32>)> {
    let mut contexts = case
        .segments
        .iter()
        .filter(|segment| segment.kind == "Context")
        .collect::<Vec<_>>();
    contexts.sort_by_key(|segment| segment.event);
    let mut parts = contexts
        .iter()
        .map(|segment| tokenizer.decode(&segment.token_ids).trim().to_owned())
        .collect::<Vec<_>>();
    parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
    match rendering {
        ONE_USER_RENDERING => {
            let content = parts.join(" ");
            Ok((
                content.clone(),
                encoder
                    .encode_assistant_prefix(&[Message {
                        role: "user",
                        content: &content,
                    }])
                    .tokens,
            ))
        }
        PER_TURN_RENDERING => {
            if parts.iter().any(|part| part.is_empty()) {
                return Err(format!("{}: an empty turn cannot be rendered", case.id));
            }
            let messages = parts
                .iter()
                .map(|content| Message {
                    role: "user",
                    content,
                })
                .collect::<Vec<_>>();
            Ok((
                parts.join("\n"),
                encoder.encode_assistant_prefix(&messages).tokens,
            ))
        }
        _ => Err("unknown rendering".to_owned()),
    }
}

fn value_words(answer: &str) -> Vec<String> {
    words(answer.trim().trim_end_matches('.'))
}

fn words(text: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            output.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        output.push(current);
    }
    output
}

fn contains_words(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn deterministic_offset(seed: u64, row: usize, step: usize) -> usize {
    let mut value = seed
        .wrapping_add((row as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(step as u64);
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    (value.wrapping_mul(0x2545_F491_4F6C_DD1D) as usize) & 0xff
}

fn distinct_shuffled_indices(references: &[Reference], seed: u64) -> Result<Vec<usize>> {
    let mut shuffled = Vec::with_capacity(references.len());
    for (index, reference) in references.iter().enumerate() {
        let chosen = (1..=references.len())
            .map(|step| {
                (index + deterministic_offset(seed, index, 0) % references.len() + step)
                    % references.len()
            })
            .find(|&candidate| {
                candidate != index && references[candidate].answer != reference.answer
            })
            .ok_or_else(|| {
                format!(
                    "could not choose a distinct shuffled value for row {}",
                    index + 1
                )
            })?;
        shuffled.push(chosen);
    }
    Ok(shuffled)
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 16] = [
        "artifact",
        "tokenizer",
        "inputs",
        "labels",
        "plan",
        "manifest",
        "out",
        "expected-artifact-sha256",
        "expected-tokenizer-sha256",
        "expected-inputs-sha256",
        "expected-labels-sha256",
        "expected-plan-sha256",
        "expected-manifest-sha256",
        "reply-tokens",
        "rendering",
        "random-seed",
    ];
    let mut parsed = BTreeMap::new();
    let mut args = args;
    while let Some(argument) = args.next() {
        let name = argument
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or_else(|| format!("unknown argument {argument}"))?;
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        if parsed.insert(name.to_owned(), value).is_some() {
            return Err(format!("{argument} given twice"));
        }
    }
    Ok(parsed)
}

fn parse_json<T: for<'de> Deserialize<'de>>(name: &str, bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|error| format!("{name}: {error}"))
}

fn verify_sha256_identity(name: &str, bytes: &[u8], expected: &str) -> Result<()> {
    let actual = uor_r4_discrete::corpus::sha256(bytes);
    if actual != expected {
        return Err(format!("{name} SHA-256 differs: got {actual}"));
    }
    Ok(())
}

fn verify_manifest_files(manifest: &Manifest, files: &[(&str, &[u8])]) -> Result<()> {
    for &(name, bytes) in files {
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        if manifest.files.get(name).map(|entry| entry.sha256.as_str()) != Some(actual.as_str()) {
            return Err(format!("panel manifest files.{name}.sha256 differs"));
        }
    }
    Ok(())
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("not a SHA-256: {value}"));
    }
    Ok(())
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn peak_rss_kib() -> Result<u64> {
    let text = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("peak RSS unavailable: {error}"))?;
    let peak = text
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .ok_or_else(|| "peak RSS measurement is unavailable".to_owned())?;
    if peak == 0 {
        return Err("peak RSS measurement is zero".to_owned());
    }
    Ok(peak)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn byte_level_tokenizer() -> ByteBpeTokenizer {
        let direct = (b'!'..=b'~')
            .chain(0xA1..=0xAC)
            .chain(0xAE..=0xFF)
            .collect::<BTreeSet<_>>();
        let mut codepoints = Vec::new();
        for byte in 0u16..=255 {
            codepoints.push(u32::from(byte as u8));
        }
        let mut extra = 256u32;
        for byte in 0u16..=255 {
            let byte = byte as u8;
            if !direct.contains(&byte) {
                codepoints.push(extra);
                extra += 1;
            }
        }
        let mut vocab = BTreeMap::<String, u32>::from([
            ("<|bos|>".to_owned(), 0),
            ("<|eos|>".to_owned(), 1),
            ("<|unk|>".to_owned(), 2),
        ]);
        for (index, codepoint) in codepoints.into_iter().enumerate() {
            vocab.insert(
                char::from_u32(codepoint).expect("byte mapping").to_string(),
                3 + index as u32,
            );
        }
        let added = (0..3)
            .map(|id| {
                let content = ["<|bos|>", "<|eos|>", "<|unk|>"][id as usize];
                serde_json::json!({
                    "id": id,
                    "content": content,
                    "single_word": false,
                    "lstrip": false,
                    "rstrip": false,
                    "normalized": false,
                    "special": true
                })
            })
            .collect::<Vec<_>>();
        let json = serde_json::json!({
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": added,
            "normalizer": null,
            "pre_tokenizer": {
                "type": "ByteLevel",
                "add_prefix_space": false,
                "trim_offsets": true,
                "use_regex": true
            },
            "post_processor": null,
            "decoder": {
                "type": "ByteLevel",
                "add_prefix_space": true,
                "trim_offsets": true,
                "use_regex": true
            },
            "model": {
                "type": "BPE",
                "dropout": null,
                "unk_token": "<|unk|>",
                "continuing_subword_prefix": null,
                "end_of_word_suffix": null,
                "fuse_unk": false,
                "byte_fallback": false,
                "ignore_merges": false,
                "vocab": vocab,
                "merges": []
            }
        });
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            &serde_json::to_vec(&json).expect("serialize tokenizer"),
        )
        .expect("byte tokenizer")
    }

    fn test_reference(answer: &str) -> Reference {
        Reference {
            answer: answer.to_owned(),
            ids: vec![3, 4, EOS],
        }
    }

    fn test_reference_with_ids(answer: &str, ids: Vec<u32>) -> Reference {
        Reference {
            answer: answer.to_owned(),
            ids,
        }
    }

    #[test]
    fn renderer_matches_both_registered_panel_forms() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let encoder = protocol.bind(&tokenizer).expect("encoder");
        let case = InputCase {
            id: "case".to_owned(),
            query_ids: tokenizer.encode("Where?"),
            segments: vec![
                Segment {
                    kind: "Context".to_owned(),
                    event: 1,
                    token_ids: tokenizer.encode("second"),
                },
                Segment {
                    kind: "Source".to_owned(),
                    event: 0,
                    token_ids: tokenizer.encode("ignored"),
                },
                Segment {
                    kind: "Context".to_owned(),
                    event: 0,
                    token_ids: tokenizer.encode("first"),
                },
            ],
        };
        let (one_user, one_user_ids) =
            render(&tokenizer, &encoder, &case, ONE_USER_RENDERING).unwrap();
        let (per_turn, per_turn_ids) =
            render(&tokenizer, &encoder, &case, PER_TURN_RENDERING).unwrap();
        assert_eq!(one_user, "first second Where?");
        assert_eq!(per_turn, "first\nsecond\nWhere?");
        assert_eq!(
            one_user_ids,
            encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: "first second Where?",
                }])
                .tokens
        );
        assert_eq!(
            per_turn_ids,
            encoder
                .encode_assistant_prefix(&[
                    Message {
                        role: "user",
                        content: "first",
                    },
                    Message {
                        role: "user",
                        content: "second",
                    },
                    Message {
                        role: "user",
                        content: "Where?",
                    },
                ])
                .tokens
        );
    }

    #[test]
    fn reference_suffix_keeps_one_terminal_eos() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let encoder = protocol.bind(&tokenizer).expect("encoder");
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec!["Aster.".to_owned(), " Aster.\n".to_owned()],
        };
        let reference = reference(&encoder, &answers).unwrap();
        assert_eq!(reference.answer, "Aster.");
        assert_eq!(reference.ids.last().copied(), Some(EOS));
        assert_eq!(
            reference.ids.iter().filter(|&&token| token == EOS).count(),
            1
        );
    }

    #[test]
    fn shuffled_value_is_deterministic_and_role_distinct() {
        let references = vec![
            test_reference("Aster."),
            test_reference("Aster."),
            test_reference("Basalt."),
            test_reference("Citrine."),
        ];
        let first = distinct_shuffled_indices(&references, 17).unwrap();
        let second = distinct_shuffled_indices(&references, 17).unwrap();
        assert_eq!(first, second);
        for (index, control) in first.into_iter().enumerate() {
            assert_ne!(index, control);
            assert_ne!(references[index].answer, references[control].answer);
        }
    }

    #[test]
    fn arm_prefixes_use_exact_role_values_without_eos() {
        let case = PreparedCase {
            id: "case".to_owned(),
            pair_id: "pair".to_owned(),
            stratum: "length2".to_owned(),
            prompt: vec![10, 11],
            target_answers: FrozenAnswers {
                intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
                accepted: vec![" target.".to_owned()],
            },
            target: test_reference_with_ids(" target.", vec![3, 4, EOS]),
            other: test_reference_with_ids(" other.", vec![5, 6, EOS]),
            shuffled: test_reference_with_ids(" shuffled.", vec![7, EOS]),
            target_value_in_prompt: true,
        };
        let prefixes = arm_prefixes(&case).unwrap();
        assert_eq!(
            prefixes,
            vec![
                (Arm::Free, Vec::new()),
                (Arm::FirstTarget, vec![3]),
                (Arm::CorrectValue, vec![3, 4]),
                (Arm::OtherRelation, vec![5, 6]),
                (Arm::ShuffledValue, vec![7]),
            ]
        );
    }

    #[test]
    fn forced_prefix_continues_through_one_complete_forward() {
        let mut snapshot = synthetic_fbs_trunk_with(
            104,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let prefix = vec![7, 8];
        let generated = generate_with_prefix(&mut forward, &[40, 41, 42, 43], &prefix, 8).unwrap();
        assert!(generated.tokens.starts_with(&prefix));
        assert!(matches!(generated.stop, Stop::Eos | Stop::Bos | Stop::Cap));
    }

    #[test]
    fn correct_prefix_continuation_stops_on_eos_without_keeping_it() {
        let generated = continue_generation(EOS, vec![7, 8], 10, 8, 24, |_| {
            Err("EOS was stepped".to_owned())
        })
        .unwrap();
        assert_eq!(generated.tokens, vec![7, 8]);
        assert_eq!(generated.stop, Stop::Eos);

        let generated = continue_generation(9, vec![7, 8], 10, 8, 24, |_| Ok(EOS)).unwrap();
        assert_eq!(generated.tokens, vec![7, 8, 9]);
        assert_eq!(generated.stop, Stop::Eos);
    }

    #[test]
    fn exact_completion_requires_eos_and_the_complete_accepted_answer() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec!["Basalt loggia.".to_owned()],
        };
        assert!(reply_is_complete(
            b" Basalt loggia.",
            Stop::Eos,
            &answers,
            &protocol
        ));
        assert!(!reply_is_complete(
            b" Basalt loggia.",
            Stop::Cap,
            &answers,
            &protocol
        ));
        assert!(!reply_is_complete(
            b" Basalt loggia. Extra",
            Stop::Eos,
            &answers,
            &protocol
        ));
        assert!(!reply_is_complete(&[0xff], Stop::Eos, &answers, &protocol));
    }

    #[test]
    fn metadata_hash_and_manifest_mismatches_fail_closed() {
        let bytes = b"panel";
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        verify_sha256_identity("panel", bytes, &actual).unwrap();
        assert!(verify_sha256_identity("panel", bytes, &"0".repeat(64)).is_err());

        let manifest = Manifest {
            files: BTreeMap::from([(
                "inputs.json".to_owned(),
                ManifestFile {
                    sha256: actual.clone(),
                },
            )]),
            tokenizer_sha256: String::new(),
            strata_rows: BTreeMap::new(),
        };
        verify_manifest_files(&manifest, &[("inputs.json", bytes)]).unwrap();
        assert!(verify_manifest_files(&manifest, &[("labels.json", bytes)]).is_err());
        assert!(verify_manifest_files(&manifest, &[("inputs.json", b"other")]).is_err());
    }

    #[test]
    fn duplicate_cases_and_malformed_pairs_fail_closed() {
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec!["basalt loggia.".to_owned()],
        };
        let inputs = Inputs {
            cases: vec![
                InputCase {
                    id: "same".to_owned(),
                    query_ids: vec![3],
                    segments: Vec::new(),
                },
                InputCase {
                    id: "same".to_owned(),
                    query_ids: vec![4],
                    segments: Vec::new(),
                },
            ],
        };
        let labels = Labels {
            cases: vec![
                LabelCase {
                    answers: answers.clone(),
                    id: "one".to_owned(),
                    pair_id: "pair-0".to_owned(),
                },
                LabelCase {
                    answers: answers.clone(),
                    id: "two".to_owned(),
                    pair_id: "pair-1".to_owned(),
                },
            ],
            membership_only: true,
            protocol: SCHEMA_V2.to_owned(),
            schema: "uor-r4.native-source-bank-labels/1".to_owned(),
        };
        assert!(panel_identity_map(&inputs, &labels).is_err());

        let labels = Labels {
            cases: (0..256)
                .map(|index| LabelCase {
                    answers: answers.clone(),
                    id: format!("case-{index}"),
                    pair_id: format!("pair-{}", index / 2),
                })
                .collect(),
            membership_only: true,
            protocol: SCHEMA_V2.to_owned(),
            schema: "uor-r4.native-source-bank-labels/1".to_owned(),
        };
        assert_eq!(label_pair_map(&labels).unwrap().len(), 128);

        let mut malformed = labels;
        malformed.cases[0].pair_id = "orphan".to_owned();
        assert!(label_pair_map(&malformed).is_err());
    }

    #[test]
    fn value_words_ignore_terminal_punctuation_only() {
        assert_eq!(
            value_words(" basalt loggia."),
            vec!["basalt".to_owned(), "loggia".to_owned()]
        );
        assert!(contains_words(
            &words("The job is basalt loggia."),
            &value_words("basalt loggia.")
        ));
        assert!(!contains_words(
            &words("The job is basalt."),
            &value_words("basalt loggia.")
        ));
    }

    #[test]
    fn paired_binary_uses_the_registered_discordant_margin() {
        let report = paired_binary(&[(true, false), (true, false), (false, true)], true).unwrap();
        assert_eq!(report["candidate_only"], 2);
        assert_eq!(report["control_only"], 1);
        assert_eq!(report["gain"], 1);
        assert_eq!(report["threshold"], 4);
        assert_eq!(report["inferential"], true);
        assert_eq!(report["comparison_kind"], "matched-treatment");
        assert_eq!(report["passes"], false);

        let control = paired_binary(&[(true, false), (false, false)], false).unwrap();
        assert_eq!(control["inferential"], false);
        assert_eq!(control["comparison_kind"], "identity-control");
        assert!(control["passes"].is_null());
    }

    #[test]
    fn panel_stop_rule_uses_token_one_for_eos() {
        assert_eq!(stop_for(EOS, 3, 10, 16, 64), Some(Stop::Eos));
        assert_eq!(stop_for(BOS, 3, 10, 16, 64), Some(Stop::Bos));
        assert_eq!(stop_for(7, 15, 10, 16, 64), Some(Stop::Cap));
        assert_eq!(stop_for(7, 3, 61, 16, 64), Some(Stop::Context));
    }

    #[test]
    fn panel_label_metadata_fails_closed() {
        let invalid = format!(
            r#"{{"cases":[],"membership_only":true,"protocol":"{SCHEMA_V2}","schema":"uor-r4.native-source-bank-labels/1","extra":1}}"#
        );
        assert!(serde_json::from_str::<Labels>(&invalid).is_err());
        assert!(validate_sha256("not-a-hash").is_err());
    }
}
