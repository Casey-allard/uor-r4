//! Whole-reply scorer for a served `UORLUT01` artifact on panel m2-fresh256.
//!
//! Each case renders as one user turn (`one-user-turn/v1`): the `Context`
//! segments' decoded text in event order, then the question, joined by single
//! spaces, encoded with the literal-role dialogue protocol v2 up to the open
//! assistant marker. `Source` segments are skipped. Generation is greedy on the
//! artifact's nearest head through `CpuFbsForward` (`--engine cpu`, the default) or
//! the integer session (`--engine integer`). A row is complete when it
//! stopped on EOS, its reply decodes as UTF-8 and the frozen answer set accepts
//! the reply text without the v2 leading space.
//!
//! The default `one-user-turn/v1` rendering is unchanged. `--rendering
//! per-turn/v1` keeps each `Context` as its own user turn. `--teacher-forced 1`
//! ranks each reference reply token, including EOS, on the CPU port, the same
//! nearest read CPU generation uses. `--paired-first 1` (with
//! `--teacher-forced 1`) also ranks, at the first reply position, the first
//! reference token of the other case in the same pair beside the asked one's.
//!
//! The CPU port's Hamming reads never read an ask table or a question carry
//! (claude-336), so `--engine cpu` refuses, by name, an artifact whose Hamming
//! image asks or carries. `--engine integer` decodes through
//! `IntegerStackSession::serve_step` (claude-336), the product path, and records
//! `settings.engine`. `--content-readout` attaches an artifact-bound sidecar to
//! the integer model, as `uor-chat` does. The teacher-forced and paired-first
//! ranks stay on the CPU port, so the integer engine refuses them.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead, CpuFbsStep};
use uor_r4_integer::stack::{
    read_hamming_uorlut01, ContentReadout, IntegerStackModel, IntegerStackSession,
};
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
    run_with_args(std::env::args().skip(1))
}

fn run_with_args(args: impl Iterator<Item = String>) -> Result<()> {
    let args = parse_args(args)?;
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
    let engine = engine_switch(&args)?;
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
    let mut model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    if !model.shape().head_nearest() {
        return Err(
            "artifact serves the writer head; this scorer reads the nearest head".to_owned(),
        );
    }
    refuse_engine(
        engine,
        &artifact,
        teacher_forced,
        args.contains_key("content-readout"),
    )?;
    let content_readout_sha256 = match args.get("content-readout") {
        Some(path) => Some(attach_content_readout(
            &mut model,
            &artifact_sha,
            Path::new(path),
        )?),
        None => None,
    };
    // Built once: every row decodes from a clone of this untouched forward, or from a fresh session
    // of the parsed model, so the timed loop holds no artifact parse.
    let fresh_forward = match engine {
        Engine::Cpu => Some(CpuFbsForward::from_artifact(&artifact).map_err(|e| e.to_string())?),
        Engine::Integer => None,
    };
    let context_length = match &fresh_forward {
        Some(forward) => forward.context_length(),
        None => model.shape().context,
    };
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
        let (reply_ids, stop) = match &fresh_forward {
            Some(fresh) => {
                let mut forward = fresh.clone();
                let decode_started = Instant::now();
                let decoded = generate(&mut forward, &prompt, cap)?;
                decode_time += decode_started.elapsed();
                decoded
            }
            None => {
                let mut session = IntegerDecoder::new(&model);
                let decode_started = Instant::now();
                let decoded = generate(&mut session, &prompt, cap)?;
                decode_time += decode_started.elapsed();
                decoded
            }
        };
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
    if engine == Engine::Integer {
        // Recorded only off the default, so a CPU-port report is byte-for-byte what it was.
        report["settings"]["engine"] = "integer".into();
        record_content_readout(&mut report, content_readout_sha256.as_deref());
    }
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
    let mut next = prime_cpu_prompt(forward, prompt)?;
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
    let next = prime_cpu_prompt(forward, prompt)?;
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

/// Which decoder serves the rows: the CPU port (the default) or the integer session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Engine {
    Cpu,
    Integer,
}

fn engine_switch(args: &BTreeMap<String, String>) -> Result<Engine> {
    match args.get("engine").map(String::as_str) {
        None | Some("cpu") => Ok(Engine::Cpu),
        Some("integer") => Ok(Engine::Integer),
        Some(other) => Err(format!("--engine wants cpu or integer, not {other}")),
    }
}

fn attach_content_readout(
    model: &mut IntegerStackModel,
    artifact_sha256: &str,
    path: &Path,
) -> Result<String> {
    let (bytes, readout) = ContentReadout::read_sidecar(path, artifact_sha256, model.shape().vocab)
        .map_err(|error| format!("--content-readout: {error}"))?;
    model
        .set_content_readout(readout)
        .map_err(|error| format!("--content-readout: {error}"))?;
    Ok(sha256(&bytes))
}

fn record_content_readout(report: &mut serde_json::Value, sha256: Option<&str>) {
    if let Some(sha256) = sha256 {
        report["inputs"]["content_readout_sha256"] = sha256.into();
        report["settings"]["content_readout"] = "attached".into();
    }
}

/// Refuses what an engine would score silently wrong: the CPU port on an artifact whose Hamming
/// image asks (a carry needs an ask, so this covers the carry), and the integer engine with the
/// teacher-forced ranks, which stay on the CPU port.
fn refuse_engine(
    engine: Engine,
    artifact: &[u8],
    teacher_forced: bool,
    content_readout: bool,
) -> Result<()> {
    match engine {
        Engine::Cpu => {
            let asks = read_hamming_uorlut01(artifact)
                .map_err(|e| format!("artifact: {e}"))?
                .is_some_and(|image| image.reads.iter().any(|read| read.ask.is_some()));
            if asks {
                return Err(
                    "the artifact's Hamming image asks or carries a question, which the \
                            CPU port does not read; score it with --engine integer"
                        .to_owned(),
                );
            }
            if content_readout {
                return Err(
                    "--content-readout is applied by the integer serving path; use \
                     --engine integer"
                        .to_owned(),
                );
            }
        }
        Engine::Integer => {
            if teacher_forced {
                return Err(
                    "--teacher-forced ranks through the CPU port; it is refused under \
                            --engine integer"
                        .to_owned(),
                );
            }
        }
    }
    Ok(())
}

/// One greedy decoder on the nearest head: the CPU port or the integer session.
trait Decoder {
    fn context_length(&self) -> usize;
    fn mark_reply_start(&mut self);
    fn step_served(&mut self, token: u32) -> Result<u32>;
}

impl Decoder for CpuFbsForward {
    fn context_length(&self) -> usize {
        CpuFbsForward::context_length(self)
    }
    fn mark_reply_start(&mut self) {
        CpuFbsForward::mark_reply_start(self);
    }
    fn step_served(&mut self, token: u32) -> Result<u32> {
        self.step(token, CpuFbsHead::Nearest)
            .map_err(|e| e.to_string())
            .map(|step| step.token)
    }
}

/// A fresh integer session: `serve_step` is the product path, serving the Hamming reads, their
/// ask heads and carry, turn memory and an attached content readout.
struct IntegerDecoder<'m> {
    session: IntegerStackSession<'m>,
    context: usize,
}

impl<'m> IntegerDecoder<'m> {
    fn new(model: &'m IntegerStackModel) -> Self {
        Self {
            session: model.session(),
            context: model.shape().context,
        }
    }
}

impl Decoder for IntegerDecoder<'_> {
    fn context_length(&self) -> usize {
        self.context
    }
    fn mark_reply_start(&mut self) {
        self.session.mark_reply_start();
    }
    fn step_served(&mut self, token: u32) -> Result<u32> {
        self.session
            .serve_step(token)
            .map_err(|e| e.to_string())
            .map(|(token, _)| token)
    }
}

fn generate<D: Decoder>(forward: &mut D, prompt: &[u32], cap: usize) -> Result<(Vec<u32>, Stop)> {
    let context = forward.context_length();
    if prompt.is_empty() {
        return Err("empty prompt".to_owned());
    }
    if prompt.len() > context {
        return Ok((Vec::new(), Stop::Context));
    }
    let mut next = prime_prompt(forward, prompt)?;
    let mut reply = Vec::new();
    loop {
        let stop = stop_for(next, reply.len(), prompt.len(), cap, context);
        if !matches!(stop, Some(Stop::Eos | Stop::Bos)) {
            reply.push(next);
        }
        if let Some(stop) = stop {
            return Ok((reply, stop));
        }
        next = forward.step_served(next)?;
    }
}

fn prime_prompt<D: Decoder>(forward: &mut D, prompt: &[u32]) -> Result<u32> {
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        step = Some(forward.step_served(token)?);
    }
    step.ok_or_else(|| "empty prompt".to_owned())
}

/// Prime the CPU port when a caller also needs its Q12 query for ranking.
fn prime_cpu_prompt(forward: &mut CpuFbsForward, prompt: &[u32]) -> Result<CpuFbsStep> {
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            CpuFbsForward::mark_reply_start(forward);
        }
        step = Some(
            CpuFbsForward::step(forward, token, CpuFbsHead::Nearest).map_err(|e| e.to_string())?,
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
    const FLAGS: [&str; 13] = [
        "engine",
        "content-readout",
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use uor_r4_integer::stack::synthetic::{
        synthetic_fbs_trunk_with, synthetic_hamming, FBS_SYNTH_NON_SQUARE,
    };
    use uor_r4_integer::stack::{
        add_hamming_ask_uorlut01, add_point_key_uorlut01, export_fbs_trunk_uorlut01, AskImage,
        ContentExample, ContentReadout, ContentSignatureMode, FbsMemoryEntry, FbsMemorySnapshot,
        FbsReplyTable, PointKeyImage, CONTENT_CODE_SIGNATURE_BITS, MEMORY_PAD,
        STACK_CONTENT_READOUT_SCHEMA,
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

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "served-reply-panel-{name}-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn byte_level_alphabet() -> [char; 256] {
        let mut table = ['\0'; 256];
        let mut assigned = [false; 256];
        for byte in (b'!'..=b'~').chain(0xA1..=0xAC).chain(0xAE..=0xFF) {
            table[byte as usize] = char::from_u32(u32::from(byte)).unwrap();
            assigned[byte as usize] = true;
        }
        let mut extra = 0u32;
        for byte in 0usize..256 {
            if !assigned[byte] {
                table[byte] = char::from_u32(256 + extra).unwrap();
                extra += 1;
            }
        }
        table
    }

    fn run_tokenizer() -> (Vec<u8>, ByteBpeTokenizer) {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), serde_json::json!(id));
        }
        for (byte, ch) in byte_level_alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), serde_json::json!(byte + 3));
        }
        let bytes = serde_json::to_vec(&serde_json::json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ],
            "model": {"type": "BPE", "vocab": vocab, "merges": []}
        }))
        .unwrap();
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).unwrap();
        (bytes, tokenizer)
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
        CpuFbsForward::from_artifact(&point_key_artifact(seed)).unwrap()
    }

    fn point_key_artifact(seed: u64) -> Vec<u8> {
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
        add_point_key_uorlut01(
            &artifact,
            &PointKeyImage {
                width: snapshot.width,
                points: vec![0; snapshot.width],
                targets: vec![99],
                vote_k: 1,
            },
        )
        .unwrap()
    }

    /// claude-338: a nearest-head trunk with Hamming reads at depths 1 and 3 (the readout), as
    /// exported without ask heads, and the same artifact with them, the readout's carrying 3 rows.
    fn ask_fixture() -> (Vec<u8>, Vec<u8>) {
        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.context = 40;
        let mut snapshot = synthetic_fbs_trunk_with(20261109, spec);
        snapshot.readout_nearest = true;
        let (vocab, width, layers) = (snapshot.vocab, snapshot.width, snapshot.layers.len());
        let mut read = synthetic_hamming(width, layers, 4, 338);
        let depths = std::mem::take(&mut read.depths);
        read.reads = depths
            .iter()
            .zip(std::mem::take(&mut read.reads))
            .filter(|(depth, _)| [1, 3].contains(*depth))
            .map(|(_, r)| r)
            .collect();
        read.depths = vec![1, 3];
        snapshot.hamming = Some(read);
        let plain = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let mut state = 338u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state >> 33
        };
        let mut ask = |carry: Option<usize>| {
            let table = (0..vocab * width)
                .map(|_| ((next() % 8192) as i32 - 4096) as i16)
                .collect();
            let gains = carry.map(|h| {
                (
                    h,
                    (0..vocab)
                        .map(|_| (next() % 131_072) as i32 - 65_536)
                        .collect(),
                )
            });
            Some(AskImage {
                table,
                carry: gains,
            })
        };
        let asks = vec![ask(None), ask(Some(3))];
        let asking = add_hamming_ask_uorlut01(&plain, &asks).unwrap();
        (plain, asking)
    }

    /// Prompts that start each row at BOS, reach the point-key memory's opening `[13, 17]` and its
    /// override after `[7, 11]`, and one past every fixture's context.
    fn engine_prompts() -> Vec<Vec<u32>> {
        vec![
            vec![BOS, 7, 11],
            vec![BOS, 13, 17, 7, 11],
            vec![BOS, 5, 9, 13, 17, 2, 7, 11],
            (0..12).map(|i| (i * 37 + 3) % 500).collect(),
            (0..41).map(|i| (i * 11 + 2) % 500).collect(),
        ]
    }

    fn decode_all<D: Decoder>(fresh: impl Fn() -> D) -> Vec<(Vec<u32>, Stop)> {
        engine_prompts()
            .iter()
            .map(|prompt| generate(&mut fresh(), prompt, 12).unwrap())
            .collect()
    }

    #[test]
    fn each_engine_refuses_what_it_would_score_silently_wrong() {
        let (plain, asking) = ask_fixture();
        assert_eq!(refuse_engine(Engine::Cpu, &plain, true, false), Ok(()));
        let refused = refuse_engine(Engine::Cpu, &asking, false, false).unwrap_err();
        assert!(refused.contains("--engine integer"), "{refused}");
        assert_eq!(
            refuse_engine(Engine::Integer, &asking, false, false),
            Ok(())
        );
        let refused = refuse_engine(Engine::Integer, &plain, true, false).unwrap_err();
        assert!(refused.contains("--teacher-forced"), "{refused}");
        let refused = refuse_engine(Engine::Cpu, &plain, false, true).unwrap_err();
        assert!(refused.contains("--engine integer"), "{refused}");
        assert_eq!(refuse_engine(Engine::Integer, &plain, false, true), Ok(()));
        let args = |v: &str| BTreeMap::from([("engine".to_owned(), v.to_owned())]);
        assert_eq!(engine_switch(&BTreeMap::new()), Ok(Engine::Cpu));
        assert_eq!(engine_switch(&args("cpu")), Ok(Engine::Cpu));
        assert_eq!(engine_switch(&args("integer")), Ok(Engine::Integer));
        assert!(engine_switch(&args("float")).is_err());
        let given = parse_args(["--engine", "integer"].map(String::from).into_iter()).unwrap();
        assert_eq!(engine_switch(&given), Ok(Engine::Integer));
        let given = parse_args(
            ["--engine", "integer", "--content-readout", "readout.json"]
                .map(String::from)
                .into_iter(),
        )
        .unwrap();
        assert_eq!(
            given.get("content-readout").map(String::as_str),
            Some("readout.json")
        );
    }

    /// Without an ask table the engines are one decoder: the same replies and stops through the
    /// Hamming reads, the point-key memory and its override, and a prompt past the context.
    #[test]
    fn the_integer_session_decodes_as_the_cpu_port_without_an_ask() {
        let (plain, _) = ask_fixture();
        for (name, artifact) in [("hamming", plain), ("point key", point_key_artifact(83))] {
            let cpu = CpuFbsForward::from_artifact(&artifact).unwrap();
            let model = IntegerStackModel::parse(&artifact).unwrap();
            let want = decode_all(|| cpu.clone());
            assert!(
                want.iter().any(|(_, stop)| *stop == Stop::Context),
                "{name}"
            );
            assert_eq!(decode_all(|| IntegerDecoder::new(&model)), want, "{name}");
        }
    }

    /// The fault the CPU-port refusal guards: the port decodes the asking artifact exactly as the
    /// one without ask heads, while the integer session reads them and decodes it differently.
    #[test]
    fn only_the_integer_session_reads_the_ask_heads() {
        let (plain, asking) = ask_fixture();
        let cpu = |bytes: &[u8]| {
            let forward = CpuFbsForward::from_artifact(bytes).unwrap();
            decode_all(|| forward.clone())
        };
        assert_eq!(cpu(&asking), cpu(&plain));
        let (plain_model, asking_model) = (
            IntegerStackModel::parse(&plain).unwrap(),
            IntegerStackModel::parse(&asking).unwrap(),
        );
        assert_eq!(
            decode_all(|| IntegerDecoder::new(&plain_model)),
            cpu(&plain)
        );
        assert_ne!(
            decode_all(|| IntegerDecoder::new(&asking_model)),
            decode_all(|| IntegerDecoder::new(&plain_model))
        );
    }

    /// `uor-chat` applies an attached content readout only through `serve_step`. The panel's integer
    /// engine must do the same, or it scores the native pick instead of the product pick.
    #[test]
    fn the_integer_engine_applies_the_product_content_readout() {
        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.d = 256;
        spec.context = 24;
        let mut snapshot = synthetic_fbs_trunk_with(411, spec);
        snapshot.readout_nearest = true;
        let plain = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let prompt = [BOS, 7, 11];
        let probe = IntegerStackModel::parse(&plain).unwrap();
        let mut session = probe.session();
        let mut native = None;
        for (index, &token) in prompt.iter().enumerate() {
            if index + 1 == prompt.len() {
                session.mark_reply_start();
            }
            native = Some(session.step_native(token).unwrap().token);
        }
        let native = native.unwrap();
        let query = session.last_query_q12().unwrap().to_vec();
        let vocab = probe.shape().vocab as u32;
        let mut target = (native + 1) % vocab;
        while target == BOS || target == EOS || target == native {
            target = (target + 1) % vocab;
        }
        let readout = ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows: vec![ContentExample {
                signature: ContentReadout::signature(&query).unwrap(),
                target,
            }],
            max_distance: 0,
        };
        let sidecar = serde_json::json!({
            "schema": STACK_CONTENT_READOUT_SCHEMA,
            "artifact_sha256": sha256(&plain),
            "readout": readout,
        });
        let path = std::env::temp_dir().join(format!(
            "served-reply-panel-content-readout-{}.json",
            std::process::id()
        ));
        let bytes = serde_json::to_vec(&sidecar).unwrap();
        fs::write(&path, &bytes).unwrap();
        let wrong_artifact = attach_content_readout(
            &mut IntegerStackModel::parse(&plain).unwrap(),
            &"0".repeat(64),
            &path,
        )
        .unwrap_err();
        assert!(
            wrong_artifact.contains("sidecar binds artifact"),
            "{wrong_artifact}"
        );

        let out_of_vocab_path = path.with_extension("out-of-vocab.json");
        let mut out_of_vocab = readout.clone();
        out_of_vocab.rows[0].target = vocab;
        let out_of_vocab_bytes = serde_json::to_vec(&serde_json::json!({
            "schema": STACK_CONTENT_READOUT_SCHEMA,
            "artifact_sha256": sha256(&plain),
            "readout": out_of_vocab,
        }))
        .unwrap();
        fs::write(&out_of_vocab_path, &out_of_vocab_bytes).unwrap();
        let wrong_vocab = attach_content_readout(
            &mut IntegerStackModel::parse(&plain).unwrap(),
            &sha256(&plain),
            &out_of_vocab_path,
        )
        .unwrap_err();
        assert!(wrong_vocab.contains("outside vocabulary"), "{wrong_vocab}");
        fs::remove_file(&out_of_vocab_path).unwrap();

        let mut candidate = IntegerStackModel::parse(&plain).unwrap();
        let sidecar_sha256 =
            attach_content_readout(&mut candidate, &sha256(&plain), &path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(sidecar_sha256, sha256(&bytes));
        let decoded = generate(&mut IntegerDecoder::new(&candidate), &prompt, 1).unwrap();
        assert_eq!(decoded.0, vec![target]);
        assert_ne!(target, native);
    }

    #[test]
    fn content_readout_identity_is_recorded_only_when_attached() {
        let mut report = serde_json::json!({
            "inputs": {},
            "settings": {"engine": "integer"},
        });
        record_content_readout(&mut report, None);
        assert!(report["inputs"].get("content_readout_sha256").is_none());
        assert!(report["settings"].get("content_readout").is_none());

        record_content_readout(&mut report, Some("sidecar-sha256"));
        assert_eq!(report["inputs"]["content_readout_sha256"], "sidecar-sha256");
        assert_eq!(report["settings"]["content_readout"], "attached");
    }

    #[test]
    fn the_complete_run_path_attaches_and_reports_the_content_readout() {
        let temp = TempDir::new("content-readout-run");
        let (tokenizer_bytes, tokenizer) = run_tokenizer();
        let query = tokenizer.encode("Hi");

        let mut spec = FBS_SYNTH_NON_SQUARE;
        spec.d = 256;
        spec.context = 64;
        let mut snapshot = synthetic_fbs_trunk_with(431, spec);
        snapshot.readout_nearest = true;
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let artifact_sha256 = sha256(&artifact);
        let tokenizer_sha256 = sha256(&tokenizer_bytes);

        let mut cases = Vec::new();
        let mut labels = Vec::new();
        let mut histories = Vec::new();
        let mut pair = 0usize;
        for (stratum, pairs) in [
            ("length2", 32),
            ("length4", 32),
            ("length8", 32),
            ("reassert", 16),
            ("update", 16),
        ] {
            for _ in 0..pairs {
                let pair_id = format!("fresh-p{pair}");
                histories.push(serde_json::json!({
                    "id": format!("p{pair}"),
                    "stratum": format!("{stratum}/pair"),
                }));
                for arm in ["a", "b"] {
                    let id = format!("case-{pair}-{arm}");
                    cases.push(serde_json::json!({
                        "id": id.clone(),
                        "query_ids": query.clone(),
                        "segments": [],
                    }));
                    labels.push(serde_json::json!({
                        "answers": {"accepted": ["unused"], "intent": "current"},
                        "id": id,
                        "pair_id": pair_id.clone(),
                    }));
                }
                pair += 1;
            }
        }
        let inputs_bytes = serde_json::to_vec(&serde_json::json!({"cases": cases})).unwrap();
        let labels_bytes = serde_json::to_vec(&serde_json::json!({
            "cases": labels,
            "membership_only": false,
            "protocol": SCHEMA_V2,
            "schema": "test",
        }))
        .unwrap();
        let plan_bytes = serde_json::to_vec(&serde_json::json!({"histories": histories})).unwrap();
        let manifest_bytes = serde_json::to_vec(&serde_json::json!({
            "files": {
                "inputs.json": {"sha256": sha256(&inputs_bytes)},
                "labels.json": {"sha256": sha256(&labels_bytes)},
                "plan.json": {"sha256": sha256(&plan_bytes)},
            },
            "tokenizer_sha256": tokenizer_sha256,
            "strata_rows": {
                "length2": 64,
                "length4": 64,
                "length8": 64,
                "reassert": 32,
                "update": 32,
            },
        }))
        .unwrap();

        let artifact_path = temp.0.join("model.uorlut");
        let tokenizer_path = temp.0.join("tokenizer.json");
        let inputs_path = temp.0.join("inputs.json");
        let labels_path = temp.0.join("labels.json");
        let plan_path = temp.0.join("plan.json");
        let manifest_path = temp.0.join("manifest.json");
        for (path, bytes) in [
            (&artifact_path, artifact.as_slice()),
            (&tokenizer_path, tokenizer_bytes.as_slice()),
            (&inputs_path, inputs_bytes.as_slice()),
            (&labels_path, labels_bytes.as_slice()),
            (&plan_path, plan_bytes.as_slice()),
            (&manifest_path, manifest_bytes.as_slice()),
        ] {
            fs::write(path, bytes).unwrap();
        }

        let write_sidecar = |path: &Path, target: u32, artifact_sha256: &str| {
            let readout = ContentReadout {
                mode: ContentSignatureMode::Code,
                signature_bits: CONTENT_CODE_SIGNATURE_BITS,
                includes_reference_end: false,
                rows: vec![ContentExample {
                    signature: vec![0; CONTENT_CODE_SIGNATURE_BITS / 64],
                    target,
                }],
                max_distance: CONTENT_CODE_SIGNATURE_BITS as u32,
            };
            let bytes = serde_json::to_vec(&serde_json::json!({
                "schema": STACK_CONTENT_READOUT_SCHEMA,
                "artifact_sha256": artifact_sha256,
                "readout": readout,
            }))
            .unwrap();
            fs::write(path, &bytes).unwrap();
            sha256(&bytes)
        };
        let readout_target = 42;
        let sidecar_path = temp.0.join("readout.json");
        let sidecar_sha256 = write_sidecar(&sidecar_path, readout_target, &artifact_sha256);
        let args = |engine: &str, out: &Path, sidecar: &Path| {
            [
                "--engine",
                engine,
                "--content-readout",
                sidecar.to_str().unwrap(),
                "--artifact",
                artifact_path.to_str().unwrap(),
                "--tokenizer",
                tokenizer_path.to_str().unwrap(),
                "--inputs",
                inputs_path.to_str().unwrap(),
                "--labels",
                labels_path.to_str().unwrap(),
                "--plan",
                plan_path.to_str().unwrap(),
                "--manifest",
                manifest_path.to_str().unwrap(),
                "--reply-tokens",
                "1",
                "--out",
                out.to_str().unwrap(),
            ]
            .map(String::from)
            .into_iter()
        };

        let out = temp.0.join("report.json");
        run_with_args(args("integer", &out, &sidecar_path)).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
        assert_eq!(report["inputs"]["content_readout_sha256"], sidecar_sha256);
        assert_eq!(report["settings"]["content_readout"], "attached");
        let rows = report["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 256);
        assert!(
            rows.iter()
                .all(|row| row["reply_ids"] == serde_json::json!([readout_target])),
            "each first served token must be the attached readout target"
        );

        let wrong_artifact_sidecar = temp.0.join("wrong-artifact.json");
        write_sidecar(&wrong_artifact_sidecar, readout_target, &"0".repeat(64));
        let wrong_artifact_out = temp.0.join("wrong-artifact-report.json");
        let error = run_with_args(args(
            "integer",
            &wrong_artifact_out,
            &wrong_artifact_sidecar,
        ))
        .unwrap_err();
        assert!(error.contains("sidecar binds artifact"), "{error}");
        assert!(!wrong_artifact_out.exists());

        let wrong_vocab_sidecar = temp.0.join("wrong-vocab.json");
        write_sidecar(
            &wrong_vocab_sidecar,
            snapshot.vocab as u32,
            &artifact_sha256,
        );
        let wrong_vocab_out = temp.0.join("wrong-vocab-report.json");
        let error =
            run_with_args(args("integer", &wrong_vocab_out, &wrong_vocab_sidecar)).unwrap_err();
        assert!(error.contains("outside vocabulary"), "{error}");
        assert!(!wrong_vocab_out.exists());

        let cpu_out = temp.0.join("cpu-report.json");
        let error = run_with_args(args("cpu", &cpu_out, &sidecar_path)).unwrap_err();
        assert!(error.contains("--content-readout is applied"), "{error}");
        assert!(error.contains("--engine integer"), "{error}");
        assert!(!cpu_out.exists());
    }

    /// The panel marks the reply start before the prompt's last token, as `StackChat` does. On the
    /// point-key fixture that mark makes the memory's opening override select token 99.
    #[test]
    fn the_integer_engine_uses_turn_memory_reply_start() {
        let model = IntegerStackModel::parse(&point_key_artifact(83)).unwrap();
        let decoded = generate(&mut IntegerDecoder::new(&model), &[BOS, 7, 11], 1).unwrap();
        assert_eq!(decoded.0, vec![99]);
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
