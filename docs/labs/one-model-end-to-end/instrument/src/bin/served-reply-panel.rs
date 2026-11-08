//! Whole-reply scorer for a served `UORLUT01` artifact on panel m2-fresh256.
//!
//! Each case renders as one user turn (`one-user-turn/v1`): the `Context`
//! segments' decoded text in event order, then the question, joined by single
//! spaces, encoded with the literal-role dialogue protocol v2 up to the open
//! assistant marker. `Source` segments are skipped. Generation is greedy on the
//! artifact's nearest head through `CpuFbsForward`. A row is complete when it
//! stopped on EOS, its reply decodes as UTF-8 and the frozen answer set accepts
//! the reply text without the v2 leading space.
//!
//! The default `one-user-turn/v1` rendering is unchanged. `--rendering
//! per-turn/v1` keeps each `Context` as its own user turn, and
//! `--teacher-forced 1` ranks each reference reply token, including EOS, with
//! the same integer nearest read that generation uses. `--paired-first 1` (with
//! `--teacher-forced 1`) also ranks, at the first reply position, the first
//! reference token of the other case in the same pair beside the asked one's.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.served-reply-panel/1";
const RENDERING: &str = "one-user-turn/v1";
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
        eprintln!("served-reply-panel: {error}");
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
    #[allow(dead_code)]
    membership_only: bool,
    protocol: String,
    #[allow(dead_code)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Stop {
    Eos,
    Bos,
    Cap,
    Context,
}

#[derive(Serialize)]
struct Row {
    id: String,
    stratum: String,
    prompt_text: String,
    prompt_tokens: usize,
    reply_ids: Vec<u32>,
    reply_text: String,
    stop: Stop,
    complete: bool,
}

#[derive(Serialize)]
struct TeacherForcedRow {
    id: String,
    stratum: String,
    answer: String,
    reference_ids: Vec<u32>,
    target_ranks: Vec<usize>,
    raw_target_ranks: Vec<usize>,
    raw_nearest_tokens: Vec<u32>,
    eos_rank: Option<usize>,
    raw_eos_rank: Option<usize>,
    point_key_overrides: usize,
    chosen_target_mismatches: usize,
    candidate_count: usize,
}

/// One case's first reply position: the rank of its own first reference token
/// and of the first reference token of the other case in its pair.
#[derive(Serialize)]
struct PairedFirstRow {
    id: String,
    other_id: String,
    stratum: String,
    answer: String,
    other_answer: String,
    asked_token: u32,
    other_token: u32,
    same_first_token: bool,
    asked_rank: usize,
    other_rank: usize,
    raw_asked_rank: usize,
    raw_other_rank: usize,
}

#[derive(Default, Serialize)]
struct PairedFirstTally {
    rows: usize,
    same_first_token: usize,
    distinct_first_token: usize,
    asked_better: usize,
    other_better: usize,
    asked_top1: usize,
    other_top1: usize,
}

#[derive(Default, Serialize)]
struct RankTally {
    targets: usize,
    top1: usize,
    top8: usize,
    top32: usize,
    eos_targets: usize,
    eos_top1: usize,
    eos_top8: usize,
    eos_top32: usize,
    rank_min: Option<usize>,
    rank_p50: Option<usize>,
    rank_p90: Option<usize>,
    rank_p95: Option<usize>,
    rank_max: Option<usize>,
    eos_rank_min: Option<usize>,
    eos_rank_p50: Option<usize>,
    eos_rank_max: Option<usize>,
}

#[derive(Debug)]
struct NearestRank {
    target_token: u32,
    chosen_token: u32,
    rank: usize,
    raw_rank: usize,
    raw_nearest_token: u32,
    point_key_override: bool,
    candidate_count: usize,
}

#[derive(Default, Serialize)]
struct Tally {
    rows: usize,
    complete: usize,
    pairs: usize,
    pairs_both_complete_distinct: usize,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let flag = |name: &str| {
        args.get(name)
            .cloned()
            .ok_or(format!("--{name} is required"))
    };
    let out = PathBuf::from(flag("out")?);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    let cap: usize = match args.get("reply-tokens") {
        Some(value) => value
            .parse()
            .map_err(|_| "--reply-tokens is not a count".to_owned())?,
        None => 64,
    };
    if cap == 0 {
        return Err("--reply-tokens must be positive".to_owned());
    }
    let rendering = args
        .get("rendering")
        .map(String::as_str)
        .unwrap_or(RENDERING);
    if rendering != RENDERING && rendering != PER_TURN_RENDERING {
        return Err(format!(
            "--rendering wants {RENDERING} or {PER_TURN_RENDERING}"
        ));
    }
    let (teacher_forced, paired_first) = rank_switches(&args)?;
    let read = |name: &str| -> Result<(Vec<u8>, String)> {
        let path = PathBuf::from(flag(name)?);
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let sha = sha256(&bytes);
        Ok((bytes, sha))
    };
    let (artifact, artifact_sha) = read("artifact")?;
    let (tokenizer_bytes, tokenizer_sha) = read("tokenizer")?;
    let (inputs_bytes, inputs_sha) = read("inputs")?;
    let (labels_bytes, labels_sha) = read("labels")?;
    let (plan_bytes, plan_sha) = read("plan")?;
    let (manifest_bytes, manifest_sha) = read("manifest")?;
    let manifest: Manifest = parse_json("manifest", &manifest_bytes)?;
    for (file, actual) in [
        ("inputs.json", &inputs_sha),
        ("labels.json", &labels_sha),
        ("plan.json", &plan_sha),
    ] {
        let pinned = manifest.files.get(file).map(|entry| &entry.sha256);
        if pinned != Some(actual) {
            return Err(format!(
                "manifest files.{file}.sha256 differs from the file read"
            ));
        }
    }
    if manifest.tokenizer_sha256 != tokenizer_sha {
        return Err("manifest tokenizer_sha256 differs from --tokenizer".to_owned());
    }
    let expected: BTreeMap<String, usize> =
        STRATA.iter().map(|(s, n)| (s.to_string(), *n)).collect();
    if manifest.strata_rows != expected {
        return Err(format!(
            "manifest strata_rows {:?} differ from {expected:?}",
            manifest.strata_rows
        ));
    }
    let inputs: Inputs = parse_json("inputs", &inputs_bytes)?;
    let labels: Labels = parse_json("labels", &labels_bytes)?;
    let plan: Plan = parse_json("plan", &plan_bytes)?;
    if labels.protocol != SCHEMA_V2 {
        return Err(format!(
            "labels protocol {} is not {SCHEMA_V2}",
            labels.protocol
        ));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    if !model.shape().head_nearest() {
        return Err(
            "artifact serves the writer head; this scorer reads the nearest head".to_owned(),
        );
    }
    // Built once: every row decodes from a clone of this untouched forward, so the timed loop
    // holds no artifact parse.
    let fresh_forward = CpuFbsForward::from_artifact(&artifact).map_err(|e| e.to_string())?;
    let context_length = fresh_forward.context_length();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer JSON is not a supported byte-level BPE tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let strata: BTreeMap<String, String> = plan
        .histories
        .iter()
        .map(|h| {
            (
                format!("fresh-{}", h.id),
                h.stratum.split('/').next().unwrap_or("").to_owned(),
            )
        })
        .collect();
    let by_id: BTreeMap<&str, &InputCase> =
        inputs.cases.iter().map(|c| (c.id.as_str(), c)).collect();
    if by_id.len() != labels.cases.len() || inputs.cases.len() != labels.cases.len() {
        return Err("inputs and labels do not hold the same cases".to_owned());
    }

    let started = Instant::now();
    // Time inside `generate` only (prompt priming plus reply steps), summed exactly and rounded once.
    let mut decode_time = Duration::ZERO;
    let mut rows = Vec::with_capacity(labels.cases.len());
    let mut pair_rows = Vec::with_capacity(labels.cases.len());
    for label in &labels.cases {
        let case = by_id
            .get(label.id.as_str())
            .ok_or(format!("{}: no input case", label.id))?;
        let stratum = strata
            .get(&label.pair_id)
            .ok_or(format!("{}: no plan history", label.pair_id))?;
        let (prompt_text, prompt) = render(&tokenizer, &encoder, case, rendering)?;
        let mut forward = fresh_forward.clone();
        let decode_started = Instant::now();
        let (reply_ids, stop) = generate(&mut forward, &prompt, cap)?;
        decode_time += decode_started.elapsed();
        let bytes = tokenizer.decode_bytes(&reply_ids);
        let (reply_text, complete) = score(stop, &bytes, &label.answers, &protocol);
        pair_rows.push((
            label.pair_id.clone(),
            stratum.clone(),
            complete,
            protocol.reply_text(&reply_text).to_owned(),
        ));
        rows.push(Row {
            id: label.id.clone(),
            stratum: stratum.clone(),
            prompt_text,
            prompt_tokens: prompt.len(),
            reply_ids,
            reply_text,
            stop,
            complete,
        });
    }
    let wall_milliseconds = started.elapsed().as_millis().max(1);
    let decode_milliseconds = decode_time.as_millis().max(1);
    let partners = if paired_first {
        pair_partners(&labels.cases)?
    } else {
        BTreeMap::new()
    };
    let mut paired_rows = Vec::new();
    // Paired-first time is kept out of the teacher-forced wall time, which stays as before.
    let mut paired_time = Duration::ZERO;
    let (teacher_rows, teacher_wall_milliseconds) = if teacher_forced {
        let started = Instant::now();
        let mut teacher_rows = Vec::with_capacity(labels.cases.len());
        for label in &labels.cases {
            let case = by_id
                .get(label.id.as_str())
                .ok_or(format!("{}: no input case", label.id))?;
            let stratum = strata
                .get(&label.pair_id)
                .ok_or(format!("{}: no plan history", label.pair_id))?;
            let teacher = teacher_forced_row(
                &tokenizer, &encoder, &artifact, case, label, stratum, rendering,
            )?;
            if paired_first {
                let paired_started = Instant::now();
                let other = partners
                    .get(label.id.as_str())
                    .ok_or(format!("{}: no pair partner", label.id))?;
                paired_rows.push(paired_first_row(
                    &tokenizer, &encoder, &artifact, case, label, other, &teacher, rendering,
                )?);
                paired_time += paired_started.elapsed();
            }
            teacher_rows.push(teacher);
        }
        (
            teacher_rows,
            started
                .elapsed()
                .saturating_sub(paired_time)
                .as_millis()
                .clamp(1, u128::from(u64::MAX)) as u64,
        )
    } else {
        (Vec::new(), 0)
    };

    let mut per_stratum: BTreeMap<String, Tally> = BTreeMap::new();
    for row in &rows {
        let tally = per_stratum.entry(row.stratum.clone()).or_default();
        tally.rows += 1;
        tally.complete += usize::from(row.complete);
    }
    for (stratum, pairs, distinct) in pair_metric(&pair_rows)? {
        let tally = per_stratum.entry(stratum).or_default();
        tally.pairs += pairs;
        tally.pairs_both_complete_distinct += distinct;
    }
    let observed: BTreeMap<String, usize> = per_stratum
        .iter()
        .map(|(s, t)| (s.clone(), t.rows))
        .collect();
    if observed != expected {
        return Err(format!(
            "scored strata rows {observed:?} differ from {expected:?}"
        ));
    }
    let stops = |kind: Stop| rows.iter().filter(|row| row.stop == kind).count();
    let reply_tokens: usize = rows.iter().map(|row| row.reply_ids.len()).sum();
    let mut report = serde_json::json!({
        "schema": SCHEMA,
        "inputs": {
            "artifact_sha256": artifact_sha,
            "tokenizer_sha256": tokenizer_sha,
            "inputs_sha256": inputs_sha,
            "labels_sha256": labels_sha,
            "plan_sha256": plan_sha,
            "manifest_sha256": manifest_sha,
        },
        "settings": {"reply_tokens": cap, "head": "nearest", "decoding": "greedy",
                     "context_length": context_length, "eos_id": EOS, "bos_id": BOS},
        "rendering": {"name": rendering, "protocol": SCHEMA_V2, "ending": "assistant-prefix"},
        "totals": {
            "rows": rows.len(),
            "complete": rows.iter().filter(|row| row.complete).count(),
            "eos_stops": stops(Stop::Eos),
            "bos_stops": stops(Stop::Bos),
            "cap_stops": stops(Stop::Cap),
            "context_stops": stops(Stop::Context),
            "pairs": per_stratum.values().map(|t| t.pairs).sum::<usize>(),
            "pairs_both_complete_distinct":
                per_stratum.values().map(|t| t.pairs_both_complete_distinct).sum::<usize>(),
        },
        "per_stratum": per_stratum,
        "reply_tokens": reply_tokens,
        "wall_milliseconds": wall_milliseconds,
        "tokens_per_second": reply_tokens as u128 * 1000 / wall_milliseconds,
        "decode_milliseconds": decode_milliseconds,
        "decode_tokens_per_second": reply_tokens as u128 * 1000 / decode_milliseconds,
        "rows": rows,
    });
    if teacher_forced {
        let mut teacher_report = teacher_forced_report(teacher_rows)?;
        teacher_report["wall_milliseconds"] = teacher_wall_milliseconds.into();
        report["teacher_forced"] = teacher_report;
    }
    if paired_first {
        let mut paired_report = paired_first_report(paired_rows)?;
        paired_report["wall_milliseconds"] =
            (paired_time.as_millis().clamp(1, u128::from(u64::MAX)) as u64).into();
        report["paired_first"] = paired_report;
    }
    let mut text = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    text.push(b'\n');
    write_new(&out, &text)?;
    println!(
        "{}",
        serde_json::json!({"schema": SCHEMA, "totals": report["totals"],
        "out": out, "sha256": sha256(&text)})
    );
    Ok(())
}

/// `one-user-turn/v1`: `Context` texts in event order, then the question,
/// joined by single spaces. `per-turn/v1`: one user message per `Context`,
/// then the question. Both end at the open `Assistant:` marker.
fn render(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    case: &InputCase,
    rendering: &str,
) -> Result<(String, Vec<u32>)> {
    let mut contexts: Vec<&Segment> = case
        .segments
        .iter()
        .filter(|s| s.kind == "Context")
        .collect();
    contexts.sort_by_key(|segment| segment.event);
    let mut parts: Vec<String> = contexts
        .iter()
        .map(|segment| tokenizer.decode(&segment.token_ids).trim().to_owned())
        .collect();
    parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
    let (content, ids) = match rendering {
        RENDERING => {
            let content = parts.join(" ");
            let ids = encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &content,
                }])
                .tokens;
            (content, ids)
        }
        PER_TURN_RENDERING => {
            if parts.iter().any(|part| part.is_empty()) {
                return Err(format!("{}: an empty turn cannot be rendered", case.id));
            }
            let messages: Vec<Message<'_>> = parts
                .iter()
                .map(|content| Message {
                    role: "user",
                    content,
                })
                .collect();
            let content = parts.join("\n");
            let ids = encoder.encode_assistant_prefix(&messages).tokens;
            (content, ids)
        }
        _ => return Err("unknown rendering".to_owned()),
    };
    Ok((content, ids))
}

/// `--teacher-forced` and `--paired-first`; the second needs the first.
fn rank_switches(args: &BTreeMap<String, String>) -> Result<(bool, bool)> {
    let teacher_forced = parse_switch("teacher-forced", args.get("teacher-forced"))?;
    let paired_first = parse_switch("paired-first", args.get("paired-first"))?;
    if paired_first && !teacher_forced {
        return Err("--paired-first 1 needs --teacher-forced 1".to_owned());
    }
    Ok((teacher_forced, paired_first))
}

fn parse_switch(name: &str, value: Option<&String>) -> Result<bool> {
    match value.map(String::as_str) {
        None | Some("0") => Ok(false),
        Some("1") => Ok(true),
        Some(_) => Err(format!("--{name} wants 0 or 1")),
    }
}

fn reference_ids(
    encoder: &DialogueEncoder<'_>,
    answers: &FrozenAnswers,
) -> Result<(String, Vec<u32>)> {
    let answer = answers
        .accepted
        .iter()
        .min_by_key(|answer| answer.len())
        .ok_or("frozen answer set is empty")?;
    let encoded = encoder.encode_assistant_suffix(
        &[Message {
            role: "assistant",
            content: answer,
        }],
        true,
    );
    let ids: Vec<u32> = encoded
        .tokens
        .iter()
        .zip(&encoded.response_mask)
        .filter_map(|(&token, &mask)| (mask == 1).then_some(token))
        .collect();
    if ids.last().copied() != Some(EOS) {
        return Err(format!("reference answer {answer:?} does not end in EOS"));
    }
    Ok((answer.clone(), ids))
}

fn nearest_rank(
    query: &[i32],
    table: &[i16],
    width: usize,
    target_token: u32,
    served_token: u32,
) -> Result<NearestRank> {
    if width == 0 || !table.len().is_multiple_of(width) {
        return Err("nearest-rank table shape differs".to_owned());
    }
    let vocabulary = table.len() / width;
    if query.len() != width
        || target_token as usize >= vocabulary
        || served_token as usize >= vocabulary
    {
        return Err("nearest-rank query or target differs".to_owned());
    }
    let score = |token: usize| {
        query
            .iter()
            .zip(&table[token * width..(token + 1) * width])
            .fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
            })
            .wrapping_neg()
    };
    let target = target_token as usize;
    let target_score = score(target);
    let mut raw_rank = 1;
    let mut raw_nearest = 0u32;
    let mut raw_nearest_score = i64::MIN;
    for token in 0..vocabulary {
        let candidate = score(token);
        if candidate > raw_nearest_score
            || (candidate == raw_nearest_score && token < raw_nearest as usize)
        {
            raw_nearest_score = candidate;
            raw_nearest = token as u32;
        }
        if token != target
            && (candidate > target_score || (candidate == target_score && token < target))
        {
            raw_rank += 1;
        }
    }
    let point_key_override = served_token != raw_nearest;
    let rank = if !point_key_override {
        raw_rank
    } else if target_token == served_token {
        1
    } else {
        2 + (0..vocabulary)
            .filter(|&token| token != target && token as u32 != served_token)
            .filter(|&token| {
                let candidate = score(token);
                candidate > target_score || (candidate == target_score && token < target)
            })
            .count()
    };
    Ok(NearestRank {
        target_token,
        chosen_token: served_token,
        rank,
        raw_rank,
        raw_nearest_token: raw_nearest,
        point_key_override,
        candidate_count: vocabulary,
    })
}

fn teacher_forced_nearest_ranks(
    forward: &mut CpuFbsForward,
    prompt: &[u32],
    targets: &[u32],
) -> Result<Vec<NearestRank>> {
    if prompt.is_empty() {
        return Err("teacher-forced ranking needs a nonempty prompt".to_owned());
    }
    if prompt.len().saturating_add(targets.len().saturating_sub(1)) > forward.context_length() {
        return Err("teacher-forced sequence passes the context".to_owned());
    }
    forward.reset();
    let mut next = prime_prompt(forward, prompt)?;
    let mut rows = Vec::with_capacity(targets.len());
    for (index, &target) in targets.iter().enumerate() {
        rows.push(nearest_rank(
            &next.query_q12,
            forward.shared_table(),
            forward.width(),
            target,
            next.token,
        )?);
        if index + 1 < targets.len() {
            next = forward
                .step(target, CpuFbsHead::Nearest)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(rows)
}

fn teacher_forced_row(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    artifact: &[u8],
    case: &InputCase,
    label: &LabelCase,
    stratum: &str,
    rendering: &str,
) -> Result<TeacherForcedRow> {
    let (_, prompt) = render(tokenizer, encoder, case, rendering)?;
    let (answer, reference_ids) = reference_ids(encoder, &label.answers)?;
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|e| e.to_string())?;
    if prompt.is_empty() {
        return Err(format!("{}: empty prompt", case.id));
    }
    let target_inputs = reference_ids.len().saturating_sub(1);
    if prompt.len() + target_inputs > forward.context_length() {
        return Err(format!(
            "{}: prompt {} plus reference {} passes context {}",
            case.id,
            prompt.len(),
            reference_ids.len(),
            forward.context_length()
        ));
    }
    let diagnostics = teacher_forced_nearest_ranks(&mut forward, &prompt, &reference_ids)
        .map_err(|e| e.to_string())?;
    if diagnostics.len() != reference_ids.len() {
        return Err(format!(
            "{}: {} reference targets yielded {} ranks",
            case.id,
            reference_ids.len(),
            diagnostics.len()
        ));
    }
    let mut target_ranks = Vec::with_capacity(reference_ids.len());
    let mut raw_target_ranks = Vec::with_capacity(reference_ids.len());
    let mut raw_nearest_tokens = Vec::with_capacity(reference_ids.len());
    let mut eos_rank = None;
    let mut raw_eos_rank = None;
    let mut point_key_overrides = 0;
    let mut chosen_target_mismatches = 0;
    let mut candidate_count = 0;
    for (target, diagnostic) in reference_ids.iter().zip(&diagnostics) {
        if diagnostic.target_token != *target {
            return Err(format!("{}: teacher-forced target misalignment", case.id));
        }
        candidate_count = diagnostic.candidate_count;
        target_ranks.push(diagnostic.rank);
        raw_target_ranks.push(diagnostic.raw_rank);
        raw_nearest_tokens.push(diagnostic.raw_nearest_token);
        point_key_overrides += usize::from(diagnostic.point_key_override);
        chosen_target_mismatches += usize::from(diagnostic.chosen_token != *target);
        if *target == EOS {
            eos_rank = Some(diagnostic.rank);
            raw_eos_rank = Some(diagnostic.raw_rank);
        }
    }
    Ok(TeacherForcedRow {
        id: label.id.clone(),
        stratum: stratum.to_owned(),
        answer,
        reference_ids,
        target_ranks,
        raw_target_ranks,
        raw_nearest_tokens,
        eos_rank,
        raw_eos_rank,
        point_key_overrides,
        chosen_target_mismatches,
        candidate_count,
    })
}

fn teacher_forced_report(rows: Vec<TeacherForcedRow>) -> Result<serde_json::Value> {
    if rows
        .iter()
        .any(|row| row.reference_ids.len() != row.target_ranks.len())
    {
        return Err("teacher-forced target count differs from its rank count".to_owned());
    }
    let targets: Vec<usize> = rows
        .iter()
        .flat_map(|row| row.target_ranks.iter().copied())
        .collect();
    let eos: Vec<usize> = rows.iter().filter_map(|row| row.eos_rank).collect();
    let first_targets: Vec<usize> = rows
        .iter()
        .filter_map(|row| row.target_ranks.first().copied())
        .collect();
    if first_targets.len() != rows.len() {
        return Err("teacher-forced rows do not each hold a first target rank".to_owned());
    }
    let raw_targets: Vec<usize> = rows
        .iter()
        .flat_map(|row| row.raw_target_ranks.iter().copied())
        .collect();
    let raw_eos: Vec<usize> = rows.iter().filter_map(|row| row.raw_eos_rank).collect();
    if eos.len() != rows.len() {
        return Err("teacher-forced rows do not each hold one EOS rank".to_owned());
    }
    let candidates: Vec<usize> = rows.iter().map(|row| row.candidate_count).collect();
    if candidates.windows(2).any(|pair| pair[0] != pair[1]) {
        return Err("teacher-forced rows disagree on candidate count".to_owned());
    }
    let mut tally = rank_tally(&targets, &eos);
    tally.eos_targets = eos.len();
    let first_target = rank_tally(&first_targets, &[]);
    let raw_tally = rank_tally(&raw_targets, &raw_eos);
    let per_stratum = {
        let mut by_stratum: BTreeMap<String, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
        for row in &rows {
            let entry = by_stratum.entry(row.stratum.clone()).or_default();
            entry.0.extend_from_slice(&row.target_ranks);
            if let Some(rank) = row.eos_rank {
                entry.1.push(rank);
            }
        }
        by_stratum
            .into_iter()
            .map(|(stratum, (targets, eos))| (stratum, rank_tally(&targets, &eos)))
            .collect::<BTreeMap<_, _>>()
    };
    Ok(serde_json::json!({
        "method": "teacher-forced nearest reference-token rank",
        "reference": "shortest frozen accepted answer",
        "candidate_count": *candidates.first().unwrap_or(&0),
        "totals": tally,
        "first_target": first_target,
        "raw_totals": raw_tally,
        "point_key_overrides": rows.iter().map(|row| row.point_key_overrides).sum::<usize>(),
        "chosen_target_mismatches": rows.iter().map(|row| row.chosen_target_mismatches).sum::<usize>(),
        "raw_nearest_target_matches": rows.iter().map(|row| row.reference_ids.iter().zip(&row.raw_nearest_tokens).filter(|(target, nearest)| target == nearest).count()).sum::<usize>(),
        "per_stratum": per_stratum,
        "rows": rows,
    }))
}

/// Each case's partner: the other case with its `pair_id`. Every pair holds
/// exactly two cases.
fn pair_partners(cases: &[LabelCase]) -> Result<BTreeMap<&str, &LabelCase>> {
    let mut by_pair: BTreeMap<&str, Vec<&LabelCase>> = BTreeMap::new();
    for case in cases {
        by_pair.entry(case.pair_id.as_str()).or_default().push(case);
    }
    let mut partners = BTreeMap::new();
    for (pair, members) in by_pair {
        let [left, right] = members.as_slice() else {
            return Err(format!("pair {pair} holds {} cases, not 2", members.len()));
        };
        if left.id == right.id {
            return Err(format!("pair {pair} holds {} twice", left.id));
        }
        partners.insert(left.id.as_str(), *right);
        partners.insert(right.id.as_str(), *left);
    }
    Ok(partners)
}

/// Every target ranked against the same first reply position: the state after
/// the prompt, before any reply token is fed.
fn first_position_ranks(
    forward: &mut CpuFbsForward,
    prompt: &[u32],
    targets: &[u32],
) -> Result<Vec<NearestRank>> {
    if prompt.is_empty() {
        return Err("first-position ranking needs a nonempty prompt".to_owned());
    }
    forward.reset();
    let next = prime_prompt(forward, prompt)?;
    targets
        .iter()
        .map(|&target| {
            nearest_rank(
                &next.query_q12,
                forward.shared_table(),
                forward.width(),
                target,
                next.token,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn paired_first_row(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    artifact: &[u8],
    case: &InputCase,
    label: &LabelCase,
    other: &LabelCase,
    teacher: &TeacherForcedRow,
    rendering: &str,
) -> Result<PairedFirstRow> {
    let (_, prompt) = render(tokenizer, encoder, case, rendering)?;
    let (other_answer, other_ids) = reference_ids(encoder, &other.answers)?;
    let asked_token = *teacher
        .reference_ids
        .first()
        .ok_or(format!("{}: empty reference", label.id))?;
    let other_token = *other_ids
        .first()
        .ok_or(format!("{}: empty reference", other.id))?;
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|e| e.to_string())?;
    let ranks = first_position_ranks(&mut forward, &prompt, &[asked_token, other_token])?;
    let [asked, other_rank] = ranks.as_slice() else {
        return Err(format!("{}: first-position ranks differ", label.id));
    };
    if Some(&asked.rank) != teacher.target_ranks.first() {
        return Err(format!(
            "{}: first-position rank {} differs from the teacher-forced {:?}",
            label.id,
            asked.rank,
            teacher.target_ranks.first()
        ));
    }
    Ok(PairedFirstRow {
        id: label.id.clone(),
        other_id: other.id.clone(),
        stratum: teacher.stratum.clone(),
        answer: teacher.answer.clone(),
        other_answer,
        asked_token,
        other_token,
        same_first_token: asked_token == other_token,
        asked_rank: asked.rank,
        other_rank: other_rank.rank,
        raw_asked_rank: asked.raw_rank,
        raw_other_rank: other_rank.raw_rank,
    })
}

fn paired_first_tally<'a>(rows: impl Iterator<Item = &'a PairedFirstRow>) -> PairedFirstTally {
    let mut tally = PairedFirstTally::default();
    for row in rows {
        tally.rows += 1;
        if row.same_first_token {
            tally.same_first_token += 1;
            continue;
        }
        tally.distinct_first_token += 1;
        tally.asked_better += usize::from(row.asked_rank < row.other_rank);
        tally.other_better += usize::from(row.other_rank < row.asked_rank);
        tally.asked_top1 += usize::from(row.asked_rank == 1);
        tally.other_top1 += usize::from(row.other_rank == 1);
    }
    tally
}

fn paired_first_report(rows: Vec<PairedFirstRow>) -> Result<serde_json::Value> {
    if rows
        .iter()
        .any(|row| !row.same_first_token && row.asked_rank == row.other_rank)
    {
        return Err("two distinct first tokens share one rank".to_owned());
    }
    let mut by_stratum: BTreeMap<String, Vec<&PairedFirstRow>> = BTreeMap::new();
    for row in &rows {
        by_stratum.entry(row.stratum.clone()).or_default().push(row);
    }
    let per_stratum: BTreeMap<String, PairedFirstTally> = by_stratum
        .into_iter()
        .map(|(stratum, rows)| (stratum, paired_first_tally(rows.into_iter())))
        .collect();
    Ok(serde_json::json!({
        "method": "first-reply-position nearest rank of the asked and the other value's first reference token",
        "reference": "shortest frozen accepted answer of each case",
        "totals": paired_first_tally(rows.iter()),
        "per_stratum": per_stratum,
        "rows": rows,
    }))
}

fn rank_tally(targets: &[usize], eos: &[usize]) -> RankTally {
    let ranks = |values: &[usize]| {
        let mut values = values.to_vec();
        values.sort_unstable();
        values
    };
    let targets_sorted = ranks(targets);
    let eos_sorted = ranks(eos);
    let percentile = |values: &[usize], percent: usize| {
        if values.is_empty() {
            None
        } else {
            let index = (values.len() * percent).div_ceil(100).saturating_sub(1);
            values.get(index).copied()
        }
    };
    let count_at_most =
        |values: &[usize], limit: usize| values.iter().take_while(|&&rank| rank <= limit).count();
    RankTally {
        targets: targets_sorted.len(),
        top1: count_at_most(&targets_sorted, 1),
        top8: count_at_most(&targets_sorted, 8),
        top32: count_at_most(&targets_sorted, 32),
        eos_targets: eos_sorted.len(),
        eos_top1: count_at_most(&eos_sorted, 1),
        eos_top8: count_at_most(&eos_sorted, 8),
        eos_top32: count_at_most(&eos_sorted, 32),
        rank_min: targets_sorted.first().copied(),
        rank_p50: percentile(&targets_sorted, 50),
        rank_p90: percentile(&targets_sorted, 90),
        rank_p95: percentile(&targets_sorted, 95),
        rank_max: targets_sorted.last().copied(),
        eos_rank_min: eos_sorted.first().copied(),
        eos_rank_p50: percentile(&eos_sorted, 50),
        eos_rank_max: eos_sorted.last().copied(),
    }
}

/// The stop rule for a newly generated `token` with `kept` reply tokens
/// already kept: EOS and BOS stop unkept; otherwise the token is kept and the
/// reply stops at the cap, then at the context.
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

fn generate(forward: &mut CpuFbsForward, prompt: &[u32], cap: usize) -> Result<(Vec<u32>, Stop)> {
    let context = forward.context_length();
    if prompt.is_empty() {
        return Err("empty prompt".to_owned());
    }
    if prompt.len() > context {
        return Ok((Vec::new(), Stop::Context));
    }
    let mut next = prime_prompt(forward, prompt)?.token;
    let mut reply = Vec::new();
    loop {
        let stop = stop_for(next, reply.len(), prompt.len(), cap, context);
        if !matches!(stop, Some(Stop::Eos | Stop::Bos)) {
            reply.push(next);
        }
        if let Some(stop) = stop {
            return Ok((reply, stop));
        }
        next = forward
            .step(next, CpuFbsHead::Nearest)
            .map_err(|e| e.to_string())?
            .token;
    }
}

fn prime_prompt(
    forward: &mut CpuFbsForward,
    prompt: &[u32],
) -> Result<uor_r4_cpu_train::CpuFbsStep> {
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        step = Some(
            forward
                .step(token, CpuFbsHead::Nearest)
                .map_err(|e| e.to_string())?,
        );
    }
    step.ok_or_else(|| "empty prompt".to_owned())
}

/// The reply text (lossy when not UTF-8) and whether the row is complete.
fn score(
    stop: Stop,
    bytes: &[u8],
    answers: &FrozenAnswers,
    protocol: &DialogueProtocol,
) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(text) => {
            let complete = stop == Stop::Eos && answers.accepts(protocol.reply_text(text));
            (text.to_owned(), complete)
        }
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), false),
    }
}

/// Per stratum: (stratum, pairs, pairs where both rows are complete and
/// their reply texts differ). Every pair must hold exactly two rows.
fn pair_metric(rows: &[(String, String, bool, String)]) -> Result<Vec<(String, usize, usize)>> {
    let mut groups: BTreeMap<&str, Vec<&(String, String, bool, String)>> = BTreeMap::new();
    for row in rows {
        let (pair, _, _, _) = row;
        groups.entry(pair.as_str()).or_default().push(row);
    }
    let mut tallies: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (pair, arms) in groups {
        let [(_, stratum_a, complete_a, text_a), (_, stratum_b, complete_b, text_b)] = arms[..]
        else {
            return Err(format!("pair {pair} does not hold two rows of one stratum"));
        };
        if stratum_a != stratum_b {
            return Err(format!("pair {pair} does not hold two rows of one stratum"));
        }
        let (pairs, distinct) = tallies.entry(stratum_a.clone()).or_default();
        *pairs += 1;
        *distinct += usize::from(*complete_a && *complete_b && text_a != text_b);
    }
    Ok(tallies.into_iter().map(|(s, (p, d))| (s, p, d)).collect())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 11] = [
        "artifact",
        "tokenizer",
        "inputs",
        "labels",
        "plan",
        "manifest",
        "out",
        "reply-tokens",
        "rendering",
        "teacher-forced",
        "paired-first",
    ];
    let mut values = BTreeMap::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        let name = arg
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or(format!("unknown argument {arg}"))?;
        let value = args.next().ok_or(format!("{arg} needs a value"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("{arg} given twice"));
        }
    }
    Ok(values)
}

fn parse_json<T: for<'de> Deserialize<'de>>(name: &str, bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|error| format!("{name}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
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
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FBS_SYNTH_NON_SQUARE};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot,
        FbsReplyTable, PointKeyImage, MEMORY_PAD,
    };

    /// The pinned tokenizer (`d36d3e87…`) lives outside the repository: name it in
    /// `UOR_PANEL_TOKENIZER`. Without it, the tests that need it say so and pass.
    fn tokenizer() -> Option<ByteBpeTokenizer> {
        let Ok(path) = std::env::var("UOR_PANEL_TOKENIZER") else {
            eprintln!("skipped: set UOR_PANEL_TOKENIZER to the pinned tokenizer.json");
            return None;
        };
        Some(ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path).unwrap()).unwrap())
    }

    fn answers(accepted: &str) -> FrozenAnswers {
        serde_json::from_value(serde_json::json!({"accepted": [accepted], "intent": "current"}))
            .unwrap()
    }

    fn fixture(seed: u64, layers: usize) -> CpuFbsForward {
        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.layers = layers;
        let mut snapshot = synthetic_fbs_trunk_with(seed, spec);
        snapshot.readout_nearest = true;
        CpuFbsForward::from_artifact(&export_fbs_trunk_uorlut01(&snapshot).unwrap()).unwrap()
    }

    fn point_key_fixture(seed: u64) -> CpuFbsForward {
        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.context = 24;
        let mut snapshot = synthetic_fbs_trunk_with(seed, spec);
        snapshot.readout_nearest = true;
        let weight = snapshot.final_norm.clone();
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: 192,
            opening: vec![FbsMemoryEntry {
                key: vec![13, 17],
                value: 42,
            }],
            opening_weight: weight.clone(),
            reply: Some(FbsReplyTable {
                turn: 2,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 1, MEMORY_PAD, 42],
                        value: 77,
                    },
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 2, MEMORY_PAD, 77],
                        value: 99,
                    },
                ],
                weight,
            }),
        });
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let artifact = add_point_key_uorlut01(
            &artifact,
            &PointKeyImage {
                width: snapshot.width,
                points: vec![0; snapshot.width],
                targets: vec![99],
                vote_k: 1,
            },
        )
        .unwrap();
        CpuFbsForward::from_artifact(&artifact).unwrap()
    }

    #[test]
    fn renders_two_contexts_in_event_order_without_source() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let segment = |kind: &str, event, text: &str| Segment {
            kind: kind.to_owned(),
            event,
            token_ids: tokenizer.encode(text),
        };
        let case = InputCase {
            id: "hand".to_owned(),
            query_ids: tokenizer.encode(" Who has the map?"),
            segments: vec![
                segment("Context", 1, " Bo has the map."),
                segment("Source", 0, "Ada keeps"),
                segment("Context", 0, "Ada keeps the key."),
            ],
        };
        let (text, ids) = render(&tokenizer, &encoder, &case, RENDERING).unwrap();
        assert_eq!(text, "Ada keeps the key. Bo has the map. Who has the map?");
        assert_eq!(ids[0], BOS);
        assert_eq!(
            tokenizer.decode(&ids),
            "<|bos|>User: Ada keeps the key. Bo has the map. Who has the map?\nAssistant:"
        );
    }

    #[test]
    fn renders_each_context_as_an_explicit_turn() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let segment = |event, text: &str| Segment {
            kind: "Context".to_owned(),
            event,
            token_ids: tokenizer.encode(text),
        };
        let case = InputCase {
            id: "turns".to_owned(),
            query_ids: tokenizer.encode(" Who has the map?"),
            segments: vec![
                segment(1, " Bo has the map."),
                segment(0, " Ada keeps the key."),
            ],
        };
        let (text, ids) = render(&tokenizer, &encoder, &case, PER_TURN_RENDERING).unwrap();
        assert_eq!(
            text,
            "Ada keeps the key.\nBo has the map.\nWho has the map?"
        );
        let decoded = tokenizer.decode(&ids);
        assert!(decoded.contains("User: Ada keeps the key.\nUser: Bo has the map."));
        assert!(decoded.ends_with("\nAssistant:"));
    }

    #[test]
    fn malformed_turns_are_refused() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let case = InputCase {
            id: "empty-turn".to_owned(),
            query_ids: tokenizer.encode("Who?"),
            segments: vec![Segment {
                kind: "Context".to_owned(),
                event: 0,
                token_ids: Vec::new(),
            }],
        };
        assert!(render(&tokenizer, &encoder, &case, PER_TURN_RENDERING)
            .unwrap_err()
            .contains("empty turn"));
        assert!(render(&tokenizer, &encoder, &case, "unknown").is_err());
    }

    #[test]
    fn default_rendering_preserves_prior_empty_context_behavior() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let case = InputCase {
            id: "empty-context-default".to_owned(),
            query_ids: tokenizer.encode("Who?"),
            segments: vec![Segment {
                kind: "Context".to_owned(),
                event: 0,
                token_ids: Vec::new(),
            }],
        };
        let (text, _) = render(&tokenizer, &encoder, &case, RENDERING).unwrap();
        assert_eq!(text, " Who?");
    }

    #[test]
    fn reference_encoding_keeps_reply_tokens_and_one_terminal_eos() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let accepted = answers("Aster.");
        let (answer, ids) = reference_ids(&encoder, &accepted).unwrap();
        assert_eq!(answer, "Aster.");
        assert_eq!(ids.last().copied(), Some(EOS));
        assert_eq!(ids.iter().filter(|&&token| token == EOS).count(), 1);
        assert!(!ids.contains(&BOS));
    }

    #[test]
    fn rank_tally_reports_tail_and_eos_without_smoothing() {
        let tally = rank_tally(&[1, 2, 8, 9, 32, 100], &[1, 7, 40]);
        assert_eq!(tally.targets, 6);
        assert_eq!(tally.top1, 1);
        assert_eq!(tally.top8, 3);
        assert_eq!(tally.top32, 5);
        assert_eq!(tally.rank_p50, Some(8));
        assert_eq!(tally.rank_max, Some(100));
        assert_eq!(tally.eos_top1, 1);
        assert_eq!(tally.eos_top8, 2);
        assert_eq!(tally.eos_rank_p50, Some(7));
        assert_eq!(tally.eos_rank_max, Some(40));
    }

    #[test]
    fn teacher_forced_report_separates_the_first_target_rank() {
        let row = |id: &str, ranks: Vec<usize>| TeacherForcedRow {
            id: id.to_owned(),
            stratum: "length2".to_owned(),
            answer: "value.".to_owned(),
            reference_ids: vec![10; ranks.len()],
            target_ranks: ranks,
            raw_target_ranks: vec![1],
            raw_nearest_tokens: vec![10],
            eos_rank: Some(2),
            raw_eos_rank: Some(2),
            point_key_overrides: 0,
            chosen_target_mismatches: 0,
            candidate_count: 16,
        };
        let report =
            teacher_forced_report(vec![row("a", vec![9, 1, 1]), row("b", vec![10, 2])]).unwrap();
        assert_eq!(report["first_target"]["targets"], 2);
        assert_eq!(report["first_target"]["top1"], 0);
        assert_eq!(report["first_target"]["top8"], 0);
        assert_eq!(report["first_target"]["rank_p50"], 9);
        assert_eq!(report["totals"]["targets"], 5);
    }

    #[test]
    fn nearest_rank_uses_ties_and_the_served_point_override() {
        let table = [0i16, 0, 0, 0, 1, 1];
        let tied = nearest_rank(&[0, 0], &table, 2, 1, 0).unwrap();
        assert_eq!(tied.chosen_token, 0);
        assert_eq!(tied.rank, 2);

        let overridden = nearest_rank(&[0, 0], &table, 2, 1, 1).unwrap();
        assert_eq!(overridden.chosen_token, 1);
        assert_eq!(overridden.rank, 1);
        assert_eq!(overridden.raw_rank, 2);
        assert_eq!(overridden.raw_nearest_token, 0);
        assert!(overridden.point_key_override);
    }

    #[test]
    fn teacher_forced_ranks_follow_the_artifact_and_context_boundary() {
        let mut cpu = fixture(29, 2);
        let rows = teacher_forced_nearest_ranks(&mut cpu, &[0, 5, 9], &[7, 11, 3]).unwrap();
        assert_eq!(
            rows.iter().map(|row| row.target_token).collect::<Vec<_>>(),
            [7, 11, 3]
        );
        assert!(rows
            .iter()
            .all(|row| (1..=row.candidate_count).contains(&row.rank)));

        let prompt = vec![0; cpu.context_length() - 1];
        let rows = teacher_forced_nearest_ranks(&mut cpu, &prompt, &[2, 3]).unwrap();
        assert_eq!(
            rows.iter().map(|row| row.target_token).collect::<Vec<_>>(),
            [2, 3]
        );
    }

    #[test]
    fn pair_partners_are_symmetric_and_refuse_odd_pairs() {
        let case = |id: &str, pair: &str| LabelCase {
            answers: answers("value."),
            id: id.to_owned(),
            pair_id: pair.to_owned(),
        };
        let cases = vec![
            case("a", "p"),
            case("b", "p"),
            case("c", "q"),
            case("d", "q"),
        ];
        let partners = pair_partners(&cases).unwrap();
        assert_eq!(partners.len(), 4);
        assert_eq!(partners["a"].id, "b");
        assert_eq!(partners["b"].id, "a");
        assert_eq!(partners["c"].id, "d");
        assert_eq!(partners["d"].id, "c");
        assert!(pair_partners(&[case("a", "p")]).is_err());
        assert!(pair_partners(&[case("a", "p"), case("b", "p"), case("c", "p")]).is_err());
        assert!(pair_partners(&[case("a", "p"), case("a", "p")]).is_err());
    }

    #[test]
    fn paired_first_row_ranks_the_partners_first_token_and_checks_the_teacher_rank() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        // A trunk whose vocabulary covers the tokenizer's ids and whose context holds the prompt.
        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.vocab = tokenizer.vocab_size();
        spec.context = 64;
        spec.layers = 2;
        let mut snapshot = synthetic_fbs_trunk_with(31, spec);
        snapshot.readout_nearest = true;
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let case = InputCase {
            id: "a".to_owned(),
            query_ids: tokenizer.encode(" Where does Ada work?"),
            segments: vec![Segment {
                kind: "Context".to_owned(),
                event: 0,
                token_ids: tokenizer.encode("Ada works at the mill. Ada lives by the harbor."),
            }],
        };
        let asked = LabelCase {
            answers: answers("mill."),
            id: "a".to_owned(),
            pair_id: "p".to_owned(),
        };
        let other = LabelCase {
            answers: answers("harbor."),
            id: "b".to_owned(),
            pair_id: "p".to_owned(),
        };
        let (_, asked_ids) = reference_ids(&encoder, &asked.answers).unwrap();
        let (_, other_ids) = reference_ids(&encoder, &other.answers).unwrap();
        assert_ne!(
            asked_ids[0], other_ids[0],
            "the fixture needs distinct first tokens"
        );

        let teacher = teacher_forced_row(
            &tokenizer, &encoder, &artifact, &case, &asked, "length2", RENDERING,
        )
        .unwrap();
        let row = paired_first_row(
            &tokenizer, &encoder, &artifact, &case, &asked, &other, &teacher, RENDERING,
        )
        .unwrap();
        assert_eq!(row.other_id, "b");
        assert_eq!(row.other_answer, "harbor.");
        assert_eq!(row.asked_token, asked_ids[0]);
        assert_eq!(row.other_token, other_ids[0]);
        assert!(!row.same_first_token);
        assert_eq!(row.asked_rank, teacher.target_ranks[0]);
        // The other rank is the partner's first token at this case's first reply position.
        let (_, prompt) = render(&tokenizer, &encoder, &case, RENDERING).unwrap();
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let other_first = teacher_forced_nearest_ranks(&mut forward, &prompt, &other_ids).unwrap();
        // Distinct ranks, so a row that copies the asked rank into the other rank is caught.
        assert_ne!(
            teacher.target_ranks[0], other_first[0].rank,
            "the fixture needs distinct first ranks"
        );
        assert_eq!(row.other_rank, other_first[0].rank);
        assert_eq!(row.raw_other_rank, other_first[0].raw_rank);

        // A teacher row whose first rank disagrees is refused.
        let mut wrong = teacher;
        wrong.target_ranks[0] += 1;
        assert!(paired_first_row(
            &tokenizer, &encoder, &artifact, &case, &asked, &other, &wrong, RENDERING,
        )
        .is_err());
    }

    #[test]
    fn first_position_ranks_match_the_teacher_forced_first_rank() {
        let mut cpu = fixture(29, 2);
        let prompt = [0, 5, 9];
        let first = first_position_ranks(&mut cpu, &prompt, &[7, 11, 3]).unwrap();
        for (target, row) in [7u32, 11, 3].into_iter().zip(&first) {
            let teacher = teacher_forced_nearest_ranks(&mut cpu, &prompt, &[target, 2]).unwrap();
            assert_eq!(row.target_token, target);
            assert_eq!(row.rank, teacher[0].rank);
            assert_eq!(row.raw_rank, teacher[0].raw_rank);
            assert_eq!(row.chosen_token, teacher[0].chosen_token);
        }
        // Distinct tokens hold distinct ranks at one position.
        assert_ne!(first[0].rank, first[1].rank);
        assert_ne!(first[1].rank, first[2].rank);
        // The reset makes a second call independent of the first.
        let again = first_position_ranks(&mut cpu, &prompt, &[7]).unwrap();
        assert_eq!(again[0].rank, first[0].rank);
        assert!(first_position_ranks(&mut cpu, &[], &[7]).is_err());
    }

    #[test]
    fn paired_first_report_counts_distinct_first_token_rows() {
        let row =
            |id: &str, stratum: &str, asked: (u32, usize), other: (u32, usize)| PairedFirstRow {
                id: id.to_owned(),
                other_id: format!("{id}-other"),
                stratum: stratum.to_owned(),
                answer: "a.".to_owned(),
                other_answer: "b.".to_owned(),
                asked_token: asked.0,
                other_token: other.0,
                same_first_token: asked.0 == other.0,
                asked_rank: asked.1,
                other_rank: other.1,
                raw_asked_rank: asked.1,
                raw_other_rank: other.1,
            };
        let report = paired_first_report(vec![
            row("a", "update", (10, 1), (11, 4)),
            row("b", "update", (10, 5), (11, 2)),
            row("c", "length2", (12, 1), (12, 1)),
            row("d", "length2", (13, 2), (14, 1)),
        ])
        .unwrap();
        let totals = &report["totals"];
        assert_eq!(totals["rows"], 4);
        assert_eq!(totals["same_first_token"], 1);
        assert_eq!(totals["distinct_first_token"], 3);
        assert_eq!(totals["asked_better"], 1);
        assert_eq!(totals["other_better"], 2);
        assert_eq!(totals["asked_top1"], 1);
        assert_eq!(totals["other_top1"], 1);
        assert_eq!(report["per_stratum"]["update"]["asked_better"], 1);
        assert_eq!(report["per_stratum"]["length2"]["same_first_token"], 1);
        assert_eq!(report["per_stratum"]["length2"]["other_better"], 1);
        assert!(paired_first_report(vec![row("e", "update", (10, 2), (11, 2))]).is_err());
    }

    #[test]
    fn paired_first_needs_the_teacher_forced_switch() {
        let switches = |given: &[&str]| {
            rank_switches(&parse_args(given.iter().map(|s| s.to_string())).unwrap())
        };
        assert_eq!(switches(&[]), Ok((false, false)));
        assert_eq!(switches(&["--teacher-forced", "1"]), Ok((true, false)));
        assert_eq!(
            switches(&["--teacher-forced", "1", "--paired-first", "1"]),
            Ok((true, true))
        );
        assert_eq!(
            switches(&["--teacher-forced", "1", "--paired-first", "0"]),
            Ok((true, false))
        );
        assert!(switches(&["--paired-first", "1"]).is_err());
        assert!(switches(&["--teacher-forced", "0", "--paired-first", "1"]).is_err());
    }

    #[test]
    fn teacher_forced_ranks_observe_a_real_point_key_override() {
        let mut cpu = point_key_fixture(83);
        let prompt = [0, 7, 11];
        let opening = teacher_forced_nearest_ranks(&mut cpu, &prompt, &[99]).unwrap();
        assert_eq!(opening[0].chosen_token, 99);
        assert_eq!(opening[0].rank, 1);
        assert!(opening[0].point_key_override);

        let raw_target = opening[0].raw_nearest_token;
        let other = teacher_forced_nearest_ranks(&mut cpu, &prompt, &[raw_target]).unwrap();
        assert_eq!(other[0].chosen_token, 99);
        assert_eq!(other[0].rank, 2);
        assert!(other[0].point_key_override);
    }

    #[test]
    fn switches_refuse_values_other_than_zero_or_one() {
        assert_eq!(parse_switch("x", None), Ok(false));
        assert_eq!(parse_switch("x", Some(&"0".to_owned())), Ok(false));
        assert_eq!(parse_switch("x", Some(&"1".to_owned())), Ok(true));
        assert!(parse_switch("x", Some(&"yes".to_owned())).is_err());
    }

    #[test]
    fn stop_rule_order() {
        assert_eq!(stop_for(EOS, 63, 10, 64, 192), Some(Stop::Eos));
        assert_eq!(stop_for(BOS, 63, 10, 64, 192), Some(Stop::Bos));
        assert_eq!(stop_for(7, 63, 190, 64, 192), Some(Stop::Cap));
        assert_eq!(stop_for(7, 3, 188, 64, 192), Some(Stop::Context));
        assert_eq!(stop_for(7, 3, 187, 64, 192), None);
        assert_eq!(stop_for(7, 0, 10, 64, 192), None);
    }

    /// Past EOS and BOS, the cap and context stops are the served path's own rule.
    #[test]
    fn cap_and_context_stops_match_the_served_rule() {
        use uor_r4_integer::stack::{reply_stop, ReplyEnd};
        for prompt in 1..=192 {
            for kept in 0..64 {
                let served = match reply_stop(7, kept, prompt, 64, 192) {
                    Some(ReplyEnd::Length) => Some(Stop::Cap),
                    Some(ReplyEnd::Context) => Some(Stop::Context),
                    Some(ReplyEnd::Bos) => Some(Stop::Bos),
                    None => None,
                };
                assert_eq!(
                    stop_for(7, kept, prompt, 64, 192),
                    served,
                    "prompt {prompt}, kept {kept}"
                );
            }
        }
    }

    #[test]
    fn known_answer_needs_eos_and_strips_leading_space() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let accepted = answers("basalt loggia.");
        assert_eq!(
            score(Stop::Eos, b" basalt loggia.", &accepted, &protocol),
            (" basalt loggia.".to_owned(), true)
        );
        assert!(matches!(
            score(Stop::Cap, b" basalt loggia.", &accepted, &protocol),
            (_, false)
        ));
        assert!(matches!(
            score(Stop::Eos, b"  basalt loggia.", &accepted, &protocol),
            (_, false)
        ));
        assert!(matches!(
            score(Stop::Eos, b" basalt loggia. ", &accepted, &protocol),
            (_, false)
        ));
        assert!(matches!(
            score(
                Stop::Eos,
                b" basalt \xff",
                &answers("basalt \u{fffd}"),
                &protocol
            ),
            (_, false)
        ));
    }

    #[test]
    fn pair_metric_needs_both_complete_and_distinct() {
        let row = |pair: &str, stratum: &str, complete, text: &str| {
            (
                pair.to_owned(),
                stratum.to_owned(),
                complete,
                text.to_owned(),
            )
        };
        let rows = vec![
            row("a", "length2", true, "x"),
            row("a", "length2", true, "y"),
            row("b", "length2", true, "x"),
            row("b", "length2", true, "x"),
            row("c", "update", true, "x"),
            row("c", "update", false, "y"),
        ];
        assert_eq!(
            pair_metric(&rows).unwrap(),
            vec![("length2".to_owned(), 2, 1), ("update".to_owned(), 1, 0)]
        );
        assert!(pair_metric(&rows[..5]).is_err());
    }
}
