//! Trace which prompt positions each served Hamming head keeps when predicting
//! the first reply token on `m2-fresh256`.
//!
//! This is a read-only CPU diagnostic over sealed integer artifacts. It does not
//! train, mutate an artifact or claim that keeping a value position means the
//! model selected that value.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead, CpuHammingHeadKind, CpuHammingReadTrace};
use uor_r4_integer::stack::{
    read_hamming_uorlut01, HammingStepRead, IntegerStackModel, NativeStep,
};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.cpu-first-reply-hamming-trace/2";
const ONE_USER_RENDERING: &str = "one-user-turn/v1";
const PER_TURN_RENDERING: &str = "per-turn/v1";
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
        eprintln!("cpu-first-reply-hamming-trace: {error}");
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

#[derive(Clone)]
struct Reference {
    answer: String,
    content_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct ValueSpan {
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct PreparedCase {
    id: String,
    pair_id: String,
    stratum: String,
    prompt: Vec<u32>,
    target: Reference,
    other: Reference,
    target_spans: Vec<ValueSpan>,
    other_spans: Vec<ValueSpan>,
}

#[derive(Serialize)]
struct HeadTrace {
    head: usize,
    kind: &'static str,
    margin_q16: i32,
    kept: Vec<PositionTrace>,
    asked_span: bool,
    other_span: bool,
    asked_first_token: bool,
    other_first_token: bool,
    asked_first_token_active: bool,
    other_first_token_active: bool,
    asked_active_span_rows: bool,
    other_active_span_rows: bool,
    asked_weight_q31: u64,
    other_weight_q31: u64,
}

#[derive(Serialize)]
struct PositionTrace {
    position: u32,
    distance: u32,
    weight_q31: u64,
}

#[derive(Serialize)]
struct ReadTrace {
    depth: usize,
    heads: Vec<HeadTrace>,
}

#[derive(Serialize)]
struct RowTrace {
    id: String,
    pair_id: String,
    stratum: String,
    prompt_len: usize,
    chosen_token: u32,
    target_first_token: u32,
    other_first_token: u32,
    target_spans: Vec<ValueSpan>,
    other_spans: Vec<ValueSpan>,
    reads: Vec<ReadTrace>,
}

#[derive(Clone, Default)]
struct HeadTally {
    rows: usize,
    empty_kept: usize,
    neither_span: usize,
    asked_only: usize,
    other_only: usize,
    both_spans: usize,
    asked_first_token: usize,
    other_first_token: usize,
    asked_first_token_active: usize,
    other_first_token_active: usize,
    margin_q16: i32,
    distance_sum: u64,
    distance_min: Option<u32>,
    distance_max: u32,
    kept_positions: u64,
    active_positions: u64,
    asked_active_span_rows: usize,
    other_active_span_rows: usize,
    both_active_span_rows: usize,
    asked_weight_q31: u64,
    other_weight_q31: u64,
}

#[derive(Default)]
struct ChosenTally {
    rows: usize,
    target_first_token: usize,
    other_first_token: usize,
    neither: usize,
    same_first_token: usize,
}

#[derive(Serialize)]
struct HeadSummary {
    head: usize,
    kind: &'static str,
    rows: usize,
    empty_kept: usize,
    neither_span: usize,
    asked_only: usize,
    other_only: usize,
    both_spans: usize,
    asked_first_token: usize,
    other_first_token: usize,
    asked_first_token_active: usize,
    other_first_token_active: usize,
    margin_q16: i32,
    distance_sum: u64,
    distance_min: Option<u32>,
    distance_max: u32,
    kept_positions: u64,
    active_positions: u64,
    asked_active_span_rows: usize,
    other_active_span_rows: usize,
    both_active_span_rows: usize,
    asked_weight_q31: u64,
    other_weight_q31: u64,
}

#[derive(Serialize)]
struct ReadSummary {
    depth: usize,
    heads: Vec<HeadSummary>,
}

#[derive(Serialize)]
struct ChosenSummary {
    rows: usize,
    target_first_token: usize,
    other_first_token: usize,
    neither: usize,
    same_first_token: usize,
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
    let rendering = args
        .get("rendering")
        .map(String::as_str)
        .unwrap_or(ONE_USER_RENDERING);
    if !matches!(rendering, ONE_USER_RENDERING | PER_TURN_RENDERING) {
        return Err(format!(
            "--rendering wants {ONE_USER_RENDERING} or {PER_TURN_RENDERING}"
        ));
    }

    let artifact = read_arg(&args, "artifact")?;
    let tokenizer_bytes = read_arg(&args, "tokenizer")?;
    let inputs_bytes = read_arg(&args, "inputs")?;
    let labels_bytes = read_arg(&args, "labels")?;
    let plan_bytes = read_arg(&args, "plan")?;
    let manifest_bytes = read_arg(&args, "manifest")?;
    for (name, bytes, expected) in [
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
    ] {
        verify_sha256(name, bytes, expected)?;
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
            "artifact serves the writer head; this trace needs the nearest head".to_owned(),
        );
    }
    let hamming = read_hamming_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "artifact carries no Hamming read".to_owned())?;
    let forward = CpuFbsForward::from_artifact(&artifact).map_err(|error| error.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "tokenizer JSON is not a supported byte-level BPE tokenizer".to_owned())?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let prepared = prepare_cases(&inputs, &labels, &plan, &tokenizer, &encoder, rendering)?;

    let copy_heads = hamming.record.heads / 2;
    let expected_trace_shape = hamming
        .reads
        .iter()
        .flat_map(|read| (0..hamming.record.heads).map(move |head| (read.depth, head)))
        .collect::<Vec<_>>();
    let threads = rayon::current_num_threads();
    let started = Instant::now();
    let rows = prepared
        .par_iter()
        .map(|case| trace_case(&model, &forward, case, copy_heads, &expected_trace_shape))
        .collect::<Result<Vec<_>>>()?;
    let wall = started.elapsed();

    let mut chosen = ChosenTally::default();
    let mut strata_chosen = BTreeMap::<String, ChosenTally>::new();
    let mut overall_tallies = BTreeMap::<(usize, usize, &'static str), HeadTally>::new();
    let mut stratum_tallies =
        BTreeMap::<String, BTreeMap<(usize, usize, &'static str), HeadTally>>::new();
    for row in &rows {
        tally_chosen(
            &mut chosen,
            row.chosen_token,
            row.target_first_token,
            row.other_first_token,
        );
        tally_chosen(
            strata_chosen.entry(row.stratum.clone()).or_default(),
            row.chosen_token,
            row.target_first_token,
            row.other_first_token,
        );
        for read in &row.reads {
            for head in &read.heads {
                let tally = overall_tallies
                    .entry((read.depth, head.head, head.kind))
                    .or_default();
                tally_head(tally, head);
                let tally = stratum_tallies
                    .entry(row.stratum.clone())
                    .or_default()
                    .entry((read.depth, head.head, head.kind))
                    .or_default();
                tally_head(tally, head);
            }
        }
    }
    let expected = STRATA
        .iter()
        .map(|(name, rows)| ((*name).to_owned(), *rows))
        .collect::<BTreeMap<_, _>>();
    let observed = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.stratum.clone()).or_insert(0usize) += 1;
        counts
    });
    if observed != expected {
        return Err("trace strata differ from the panel".to_owned());
    }

    let summary = summarize(&overall_tallies, &chosen);
    let per_stratum = stratum_tallies
        .iter()
        .map(|(stratum, tallies)| (stratum.clone(), summarize(tallies, &strata_chosen[stratum])))
        .collect::<BTreeMap<_, _>>();
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
            "reply_position": "after the final prompt token",
            "value_spans": "exact consecutive token spans in the rendered prompt",
            "distance": "candidate sign-code XOR popcount in bits",
            "margin": "artifact head margin in Q16 bits",
            "position_weight": "exact Q31 margin weight used by the served head",
            "read_count": hamming.reads.len(),
            "head_count_per_read": hamming.record.heads,
            "threads": threads,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
        },
        "summary": summary,
        "per_stratum": per_stratum,
        "rows": rows,
        "wall_milliseconds": wall.as_millis().clamp(1, u128::from(u64::MAX)),
        "peak_rss_kib": peak_rss_kib()?,
        "claim": "read-only first-reply Hamming position, distance and margin-weight attribution; not a causal, capability or serving result"
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
            "summary": report["summary"],
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
) -> Result<Vec<PreparedCase>> {
    if inputs.cases.len() != 256 || labels.cases.len() != 256 {
        return Err("inputs and labels must each hold 256 cases".to_owned());
    }
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
    let mut pairs = BTreeMap::<&str, Vec<usize>>::new();
    for (index, label) in labels.cases.iter().enumerate() {
        pairs.entry(label.pair_id.as_str()).or_default().push(index);
    }
    if pairs.len() != 128 || pairs.values().any(|indices| indices.len() != 2) {
        return Err("panel labels must form 128 two-row pairs".to_owned());
    }
    let references = labels
        .cases
        .iter()
        .map(|label| {
            label
                .answers
                .validate()
                .map_err(|error| error.to_string())?;
            reference(encoder, &label.answers)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut prepared = Vec::with_capacity(256);
    for (index, label) in labels.cases.iter().enumerate() {
        let case = by_id
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no input case", label.id))?;
        let stratum = strata
            .get(&label.pair_id)
            .ok_or_else(|| format!("{}: no plan history", label.pair_id))?
            .clone();
        let (_, prompt) = render(case, tokenizer, encoder, rendering)?;
        let pair = &pairs[label.pair_id.as_str()];
        let other_index = if pair[0] == index { pair[1] } else { pair[0] };
        let target = references[index].clone();
        let other = references[other_index].clone();
        let target_spans = locate_spans(&prompt, &target.content_ids);
        let other_spans = locate_spans(&prompt, &other.content_ids);
        if target_spans.is_empty() {
            return Err(format!(
                "{}: asked value {:?} has no token span in the rendered prompt",
                label.id, target.answer
            ));
        }
        if other_spans.is_empty() {
            return Err(format!(
                "{}: other value {:?} has no token span in the rendered prompt",
                label.id, other.answer
            ));
        }
        prepared.push(PreparedCase {
            id: label.id.clone(),
            pair_id: label.pair_id.clone(),
            stratum,
            prompt,
            target,
            other,
            target_spans,
            other_spans,
        });
    }
    Ok(prepared)
}

fn render(
    case: &InputCase,
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
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
    let (content, ids) = match rendering {
        ONE_USER_RENDERING => {
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
            let messages = parts
                .iter()
                .map(|content| Message {
                    role: "user",
                    content,
                })
                .collect::<Vec<_>>();
            (
                parts.join("\n"),
                encoder.encode_assistant_prefix(&messages).tokens,
            )
        }
        _ => return Err("unsupported rendering".to_owned()),
    };
    Ok((content, ids))
}

fn reference(encoder: &DialogueEncoder<'_>, answers: &FrozenAnswers) -> Result<Reference> {
    let shortest_len = answers
        .accepted
        .iter()
        .map(String::len)
        .min()
        .ok_or("frozen answer set is empty")?;
    let shortest = answers
        .accepted
        .iter()
        .filter(|answer| answer.len() == shortest_len)
        .collect::<Vec<_>>();
    let answer = match shortest.as_slice() {
        [answer] => *answer,
        _ => {
            return Err(format!(
                "the shortest accepted answer is ambiguous among {} forms",
                shortest.len()
            ));
        }
    };
    let encoded = encoder.encode_assistant_suffix(
        &[Message {
            role: "assistant",
            content: answer.as_str(),
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
        content_ids: ids[..ids.len() - 1].to_vec(),
    })
}

fn locate_spans(haystack: &[u32], needle: &[u32]) -> Vec<ValueSpan> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return Vec::new();
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(start, candidate)| {
            (candidate == needle).then_some(ValueSpan {
                start,
                end: start + needle.len(),
            })
        })
        .collect()
}

fn in_spans(position: u32, spans: &[ValueSpan]) -> bool {
    spans
        .iter()
        .any(|span| (span.start..span.end).contains(&(position as usize)))
}

fn kept_in_spans(kept: &[PositionTrace], spans: &[ValueSpan]) -> bool {
    kept.iter()
        .any(|position| in_spans(position.position, spans))
}

fn active_in_spans(kept: &[PositionTrace], spans: &[ValueSpan]) -> bool {
    kept.iter()
        .any(|position| position.weight_q31 != 0 && in_spans(position.position, spans))
}

fn kept_at_span_starts(kept: &[PositionTrace], spans: &[ValueSpan]) -> bool {
    spans.iter().any(|span| {
        kept.iter()
            .any(|position| position.position as usize == span.start)
    })
}

fn active_at_span_starts(kept: &[PositionTrace], spans: &[ValueSpan]) -> bool {
    spans.iter().any(|span| {
        kept.iter()
            .any(|position| position.position as usize == span.start && position.weight_q31 != 0)
    })
}

fn weight_in_spans(kept: &[PositionTrace], spans: &[ValueSpan]) -> u64 {
    kept.iter()
        .filter(|position| in_spans(position.position, spans))
        .map(|position| position.weight_q31)
        .sum()
}

fn trace_first_reply(
    model: &IntegerStackModel,
    prompt: &[u32],
) -> Result<(NativeStep, Vec<HammingStepRead>)> {
    let (last, prefix) = prompt
        .split_last()
        .ok_or_else(|| "first-reply trace needs a nonempty prompt".to_owned())?;
    let mut session = model.session();
    for &token in prefix {
        session
            .step_native(token)
            .map_err(|error| error.to_string())?;
    }
    session.mark_reply_start();
    session.set_hamming_trace(true);
    let decision = session
        .step_native(*last)
        .map_err(|error| error.to_string())?;
    let reads = session
        .take_hamming_trace()
        .ok_or_else(|| "Hamming trace did not turn on".to_owned())?;
    if reads.is_empty() {
        return Err("the first-reply step produced no Hamming trace".to_owned());
    }
    Ok((decision, reads))
}

fn trace_first_reply_weights(
    forward: &CpuFbsForward,
    prompt: &[u32],
) -> Result<(u32, Vec<CpuHammingReadTrace>)> {
    let (last, prefix) = prompt
        .split_last()
        .ok_or_else(|| "first-reply trace needs a nonempty prompt".to_owned())?;
    let mut forward = forward.clone();
    for &token in prefix {
        forward
            .step(token, CpuFbsHead::Nearest)
            .map_err(|error| error.to_string())?;
    }
    forward.mark_reply_start();
    forward
        .set_hamming_trace(true)
        .map_err(|error| error.to_string())?;
    let decision = forward
        .step(*last, CpuFbsHead::Nearest)
        .map_err(|error| error.to_string())?;
    let reads = forward
        .take_hamming_trace()
        .ok_or_else(|| "CPU Hamming trace did not turn on".to_owned())?;
    if reads.is_empty() {
        return Err("the first-reply step produced no CPU Hamming trace".to_owned());
    }
    Ok((decision.token, reads))
}

fn validate_cpu_trace_alignment(
    reads: &[HammingStepRead],
    cpu_reads: &[CpuHammingReadTrace],
    copy_heads: usize,
) -> Result<()> {
    if cpu_reads.len() != reads.len() {
        return Err("CPU and integer Hamming read counts differ".to_owned());
    }
    for (integer, cpu) in reads.iter().zip(cpu_reads) {
        if integer.depth != cpu.depth || integer.heads.len() != cpu.heads.len() {
            return Err(format!(
                "CPU and integer Hamming trace shape differs at depth {}",
                integer.depth
            ));
        }
        for (head, (integer_head, cpu_head)) in integer.heads.iter().zip(&cpu.heads).enumerate() {
            let expected_kind = if head < copy_heads {
                CpuHammingHeadKind::Copy
            } else {
                CpuHammingHeadKind::Type
            };
            let positions = cpu_head
                .kept
                .iter()
                .map(|position| position.position)
                .collect::<Vec<_>>();
            if cpu_head.head != head
                || cpu_head.kind != expected_kind
                || integer_head.query != cpu_head.query
                || integer_head.key != cpu_head.key
                || integer_head.kept != positions
            {
                return Err(format!(
                    "CPU and integer Hamming choices differ at depth {} head {}",
                    integer.depth, head
                ));
            }
        }
    }
    Ok(())
}

fn trace_case(
    model: &IntegerStackModel,
    forward: &CpuFbsForward,
    case: &PreparedCase,
    copy_heads: usize,
    expected_shape: &[(usize, usize)],
) -> Result<RowTrace> {
    let (decision, reads) = trace_first_reply(model, &case.prompt)?;
    let (cpu_decision, cpu_reads) = trace_first_reply_weights(forward, &case.prompt)?;
    if decision.token != cpu_decision {
        return Err(format!("{}: CPU and integer first tokens differ", case.id));
    }
    let current_shape = reads
        .iter()
        .flat_map(|read| (0..read.heads.len()).map(move |head| (read.depth, head)))
        .collect::<Vec<_>>();
    if current_shape != expected_shape {
        return Err(format!(
            "{}: Hamming trace shape differs from the artifact manifest",
            case.id
        ));
    }
    validate_cpu_trace_alignment(&reads, &cpu_reads, copy_heads)
        .map_err(|error| format!("{}: {error}", case.id))?;
    let target_first = *case
        .target
        .content_ids
        .first()
        .ok_or_else(|| format!("{}: target has no first token", case.id))?;
    let other_first = *case
        .other
        .content_ids
        .first()
        .ok_or_else(|| format!("{}: other value has no first token", case.id))?;
    let read_rows = reads
        .iter()
        .zip(&cpu_reads)
        .map(|(read, cpu_read)| {
            let heads = read
                .heads
                .iter()
                .zip(&cpu_read.heads)
                .enumerate()
                .map(|(head, (trace, cpu_trace))| -> Result<HeadTrace> {
                    let kind = if head < copy_heads { "copy" } else { "type" };
                    let kept = trace
                        .kept
                        .iter()
                        .map(|&position| {
                            cpu_trace
                                .kept
                                .iter()
                                .find(|metric| metric.position == position)
                                .map(|metric| PositionTrace {
                                    position,
                                    distance: metric.distance,
                                    weight_q31: metric.weight_q31,
                                })
                                .ok_or_else(|| {
                                    format!("{}: no weight metric for position {position}", case.id)
                                })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let asked_span = kept_in_spans(&kept, &case.target_spans);
                    let other_span = kept_in_spans(&kept, &case.other_spans);
                    let asked_active_span_rows = active_in_spans(&kept, &case.target_spans);
                    let other_active_span_rows = active_in_spans(&kept, &case.other_spans);
                    Ok(HeadTrace {
                        head,
                        kind,
                        margin_q16: cpu_trace.margin_q16,
                        asked_span,
                        other_span,
                        asked_first_token: kept_at_span_starts(&kept, &case.target_spans),
                        other_first_token: kept_at_span_starts(&kept, &case.other_spans),
                        asked_first_token_active: active_at_span_starts(&kept, &case.target_spans),
                        other_first_token_active: active_at_span_starts(&kept, &case.other_spans),
                        asked_active_span_rows,
                        other_active_span_rows,
                        asked_weight_q31: weight_in_spans(&kept, &case.target_spans),
                        other_weight_q31: weight_in_spans(&kept, &case.other_spans),
                        kept,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ReadTrace {
                depth: read.depth,
                heads,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(RowTrace {
        id: case.id.clone(),
        pair_id: case.pair_id.clone(),
        stratum: case.stratum.clone(),
        prompt_len: case.prompt.len(),
        chosen_token: decision.token,
        target_first_token: target_first,
        other_first_token: other_first,
        target_spans: case.target_spans.clone(),
        other_spans: case.other_spans.clone(),
        reads: read_rows,
    })
}

fn tally_head(tally: &mut HeadTally, head: &HeadTrace) {
    tally.rows += 1;
    tally.kept_positions += head.kept.len() as u64;
    tally.active_positions += head
        .kept
        .iter()
        .filter(|position| position.weight_q31 != 0)
        .count() as u64;
    tally.empty_kept += usize::from(head.kept.is_empty());
    tally.asked_first_token += usize::from(head.asked_first_token);
    tally.other_first_token += usize::from(head.other_first_token);
    tally.asked_first_token_active += usize::from(head.asked_first_token_active);
    tally.other_first_token_active += usize::from(head.other_first_token_active);
    tally.margin_q16 = head.margin_q16;
    tally.distance_sum = tally.distance_sum.saturating_add(
        head.kept
            .iter()
            .map(|position| u64::from(position.distance))
            .sum::<u64>(),
    );
    for position in &head.kept {
        tally.distance_min = Some(
            tally
                .distance_min
                .map_or(position.distance, |minimum| minimum.min(position.distance)),
        );
        tally.distance_max = tally.distance_max.max(position.distance);
    }
    tally.asked_active_span_rows += usize::from(head.asked_active_span_rows);
    tally.other_active_span_rows += usize::from(head.other_active_span_rows);
    tally.both_active_span_rows +=
        usize::from(head.asked_active_span_rows && head.other_active_span_rows);
    tally.asked_weight_q31 = tally.asked_weight_q31.saturating_add(head.asked_weight_q31);
    tally.other_weight_q31 = tally.other_weight_q31.saturating_add(head.other_weight_q31);
    match (head.asked_span, head.other_span) {
        (false, false) => tally.neither_span += 1,
        (true, false) => tally.asked_only += 1,
        (false, true) => tally.other_only += 1,
        (true, true) => tally.both_spans += 1,
    }
}

fn tally_chosen(tally: &mut ChosenTally, chosen: u32, target_first: u32, other_first: u32) {
    tally.rows += 1;
    tally.target_first_token += usize::from(chosen == target_first);
    tally.other_first_token += usize::from(chosen == other_first);
    tally.neither += usize::from(chosen != target_first && chosen != other_first);
    tally.same_first_token += usize::from(target_first == other_first);
}

fn summarize(
    tallies: &BTreeMap<(usize, usize, &'static str), HeadTally>,
    chosen: &ChosenTally,
) -> serde_json::Value {
    let mut reads = BTreeMap::<usize, Vec<HeadSummary>>::new();
    for (&(depth, head, kind), tally) in tallies {
        reads.entry(depth).or_default().push(HeadSummary {
            head,
            kind,
            rows: tally.rows,
            empty_kept: tally.empty_kept,
            neither_span: tally.neither_span,
            asked_only: tally.asked_only,
            other_only: tally.other_only,
            both_spans: tally.both_spans,
            asked_first_token: tally.asked_first_token,
            other_first_token: tally.other_first_token,
            asked_first_token_active: tally.asked_first_token_active,
            other_first_token_active: tally.other_first_token_active,
            margin_q16: tally.margin_q16,
            distance_sum: tally.distance_sum,
            distance_min: tally.distance_min,
            distance_max: tally.distance_max,
            kept_positions: tally.kept_positions,
            active_positions: tally.active_positions,
            asked_active_span_rows: tally.asked_active_span_rows,
            other_active_span_rows: tally.other_active_span_rows,
            both_active_span_rows: tally.both_active_span_rows,
            asked_weight_q31: tally.asked_weight_q31,
            other_weight_q31: tally.other_weight_q31,
        });
    }
    let reads = reads
        .into_iter()
        .map(|(depth, mut heads)| {
            heads.sort_by_key(|head| head.head);
            ReadSummary { depth, heads }
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "chosen": ChosenSummary {
            rows: chosen.rows,
            target_first_token: chosen.target_first_token,
            other_first_token: chosen.other_first_token,
            neither: chosen.neither,
            same_first_token: chosen.same_first_token,
        },
        "reads": reads,
    })
}

fn read_arg(args: &BTreeMap<String, String>, name: &str) -> Result<Vec<u8>> {
    let path = args
        .get(name)
        .ok_or_else(|| format!("--{name} is required"))?;
    std::fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 14] = [
        "artifact",
        "tokenizer",
        "inputs",
        "labels",
        "plan",
        "manifest",
        "out",
        "rendering",
        "expected-artifact-sha256",
        "expected-tokenizer-sha256",
        "expected-inputs-sha256",
        "expected-labels-sha256",
        "expected-plan-sha256",
        "expected-manifest-sha256",
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
    for name in [
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
    ] {
        if !parsed.contains_key(name) {
            return Err(format!("--{name} is required"));
        }
    }
    Ok(parsed)
}

fn parse_json<T: for<'de> Deserialize<'de>>(name: &str, bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|error| format!("{name}: {error}"))
}

fn verify_sha256(name: &str, bytes: &[u8], expected: &str) -> Result<()> {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name}: expected value is not a SHA-256"));
    }
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
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{copy_weight_q31, export_fbs_trunk_uorlut01};

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

    #[test]
    fn renderer_matches_the_protocol_prompt_tokens() {
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
            render(&case, &tokenizer, &encoder, ONE_USER_RENDERING).unwrap();
        let (per_turn, per_turn_ids) =
            render(&case, &tokenizer, &encoder, PER_TURN_RENDERING).unwrap();
        assert_eq!(one_user, "first second Where?");
        assert_eq!(per_turn, "first\nsecond\nWhere?");
        assert_eq!(
            tokenizer.decode(&one_user_ids),
            "<|bos|>User: first second Where?\nAssistant:"
        );
        assert_eq!(
            tokenizer.decode(&per_turn_ids),
            "<|bos|>User: first\nUser: second\nUser: Where?\nAssistant:"
        );
        assert_eq!(
            one_user_ids,
            encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &one_user,
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
    fn value_span_search_finds_repeats_and_reports_absence() {
        assert_eq!(
            locate_spans(&[1, 2, 3, 2, 3, 4], &[2, 3]),
            vec![
                ValueSpan { start: 1, end: 3 },
                ValueSpan { start: 3, end: 5 }
            ]
        );
        assert!(locate_spans(&[1, 2, 3], &[4]).is_empty());
        assert!(locate_spans(&[1], &[]).is_empty());
    }

    #[test]
    fn head_tally_separates_asked_other_both_and_neither() {
        let asked = ValueSpan { start: 4, end: 6 };
        let other = ValueSpan { start: 8, end: 10 };
        let mut tally = HeadTally::default();
        let head = |kept: Vec<(u32, u64)>| HeadTrace {
            head: 0,
            kind: "copy",
            margin_q16: 2 << 16,
            asked_span: kept
                .iter()
                .any(|&(position, _)| in_spans(position, &[asked])),
            other_span: kept
                .iter()
                .any(|&(position, _)| in_spans(position, &[other])),
            asked_first_token: kept.iter().any(|&(position, _)| position == 4),
            other_first_token: kept.iter().any(|&(position, _)| position == 8),
            asked_first_token_active: kept
                .iter()
                .any(|&(position, weight)| position == 4 && weight != 0),
            other_first_token_active: kept
                .iter()
                .any(|&(position, weight)| position == 8 && weight != 0),
            asked_active_span_rows: kept
                .iter()
                .any(|&(position, weight)| weight != 0 && in_spans(position, &[asked])),
            other_active_span_rows: kept
                .iter()
                .any(|&(position, weight)| weight != 0 && in_spans(position, &[other])),
            asked_weight_q31: kept
                .iter()
                .filter(|&&(position, _)| in_spans(position, &[asked]))
                .map(|&(_, weight)| weight)
                .sum(),
            other_weight_q31: kept
                .iter()
                .filter(|&&(position, _)| in_spans(position, &[other]))
                .map(|&(_, weight)| weight)
                .sum(),
            kept: kept
                .into_iter()
                .map(|(position, weight_q31)| PositionTrace {
                    position,
                    distance: 0,
                    weight_q31,
                })
                .collect(),
        };
        tally_head(&mut tally, &head(vec![(2, 0)]));
        tally_head(&mut tally, &head(vec![(4, 4), (8, 0), (9, 3)]));
        tally_head(&mut tally, &head(vec![(5, 2)]));
        tally_head(&mut tally, &head(vec![(9, 0)]));
        tally_head(&mut tally, &head(Vec::new()));
        assert_eq!(tally.rows, 5);
        assert_eq!(tally.empty_kept, 1);
        assert_eq!(tally.neither_span, 2);
        assert_eq!(tally.asked_only, 1);
        assert_eq!(tally.other_only, 1);
        assert_eq!(tally.both_spans, 1);
        assert_eq!(tally.asked_first_token, 1);
        assert_eq!(tally.other_first_token, 1);
        assert_eq!(tally.asked_first_token_active, 1);
        assert_eq!(tally.other_first_token_active, 0);
        assert_eq!(tally.active_positions, 3);
        assert_eq!(tally.asked_active_span_rows, 2);
        assert_eq!(tally.other_active_span_rows, 1);
        assert_eq!(tally.both_active_span_rows, 1);
        assert_eq!(tally.asked_weight_q31, 6);
        assert_eq!(tally.other_weight_q31, 3);
        assert_eq!(tally.margin_q16, 2 << 16);
        assert_eq!(tally.distance_sum, 0);
        assert_eq!(tally.distance_min, Some(0));
        assert_eq!(tally.distance_max, 0);
        let kept = [
            PositionTrace {
                position: 3,
                distance: 1,
                weight_q31: 0,
            },
            PositionTrace {
                position: 5,
                distance: 0,
                weight_q31: 2,
            },
        ];
        assert!(kept_in_spans(&kept, &[asked]));
        assert!(active_in_spans(&kept, &[asked]));
        assert_eq!(weight_in_spans(&kept, &[asked]), 2);
        assert!(kept_at_span_starts(
            &[
                PositionTrace {
                    position: 4,
                    distance: 1,
                    weight_q31: 1,
                },
                PositionTrace {
                    position: 8,
                    distance: 1,
                    weight_q31: 1,
                },
            ],
            &[asked, other]
        ));
    }

    #[test]
    fn first_reply_trace_contains_only_the_decision_step() {
        let mut snapshot = synthetic_fbs_trunk_with(
            242,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 2,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        snapshot.hamming = Some(synthetic_hamming(snapshot.width, 2, 4, 242));
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&artifact).unwrap();
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let prompt = [3, 4, 5, 6, 7, 8];
        let (decision, reads) = trace_first_reply(&model, &prompt).unwrap();
        let (cpu_decision, cpu_reads) = trace_first_reply_weights(&forward, &prompt).unwrap();
        assert_eq!(decision.token, cpu_decision);
        assert_eq!(reads.len(), 2);
        assert_eq!(cpu_reads.len(), 2);
        assert_eq!(
            reads.iter().map(|read| read.depth).collect::<Vec<_>>(),
            [1, 2]
        );
        assert!(reads.iter().all(|read| read.heads.len() == 4));
        assert!(reads.iter().flat_map(|read| &read.heads).all(|head| {
            head.kept
                .iter()
                .all(|&position| position < prompt.len() as u32)
        }));
        assert!(reads.iter().zip(&cpu_reads).all(|(integer, cpu)| {
            integer.heads.iter().zip(&cpu.heads).all(|(integer, cpu)| {
                integer.kept
                    == cpu
                        .kept
                        .iter()
                        .map(|position| position.position)
                        .collect::<Vec<_>>()
                    && cpu.kept.iter().all(|position| {
                        position.weight_q31 == copy_weight_q31(cpu.margin_q16, position.distance)
                    })
            })
        }));
    }

    #[test]
    fn metadata_hash_and_manifest_mismatches_fail_closed() {
        let bytes = b"panel";
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        verify_sha256("panel", bytes, &actual).unwrap();
        assert!(verify_sha256("panel", bytes, &"0".repeat(64)).is_err());

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
    fn reference_excludes_eos_and_missing_args_are_refused() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let encoder = protocol.bind(&tokenizer).expect("encoder");
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec!["Aster.".to_owned()],
        };
        let reference = reference(&encoder, &answers).unwrap();
        assert!(!reference.content_ids.is_empty());
        assert!(!reference.content_ids.contains(&EOS));
        assert!(parse_args(std::iter::empty()).is_err());
    }

    #[test]
    fn equal_shortest_answers_are_refused_as_ambiguous() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("protocol");
        let encoder = protocol.bind(&tokenizer).expect("encoder");
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec![" Aster.\n".to_owned(), " Beryl.\n".to_owned()],
        };
        let error = match reference(&encoder, &answers) {
            Err(error) => error,
            Ok(_) => panic!("equal-shortest answers should be ambiguous"),
        };
        assert!(error.contains("ambiguous"), "{error}");
    }

    #[test]
    fn cpu_trace_alignment_fails_closed_and_schema_is_v2() {
        use uor_r4_cpu_train::{CpuHammingHeadTrace, CpuHammingPositionTrace};
        use uor_r4_integer::stack::HammingStepHead;

        assert_eq!(SCHEMA, "uor-r4.cpu-first-reply-hamming-trace/2");
        let position = |position| CpuHammingPositionTrace {
            position,
            distance: 3,
            weight_q31: 7,
        };
        let cpu_head = |head, kind, query: u64, key: u64, kept: Vec<u32>| CpuHammingHeadTrace {
            head,
            kind,
            query,
            key,
            margin_q16: 4 << 16,
            kept: kept.into_iter().map(position).collect(),
        };
        let integer = vec![HammingStepRead {
            depth: 2,
            heads: vec![
                HammingStepHead {
                    query: 11,
                    key: 12,
                    kept: vec![1, 2],
                },
                HammingStepHead {
                    query: 13,
                    key: 14,
                    kept: vec![3],
                },
            ],
        }];
        let matching = vec![CpuHammingReadTrace {
            depth: 2,
            heads: vec![
                cpu_head(0, CpuHammingHeadKind::Copy, 11, 12, vec![1, 2]),
                cpu_head(1, CpuHammingHeadKind::Type, 13, 14, vec![3]),
            ],
        }];
        validate_cpu_trace_alignment(&integer, &matching, 1).unwrap();

        let error = validate_cpu_trace_alignment(&integer, &[], 1).unwrap_err();
        assert!(error.contains("read counts"), "{error}");

        let mut wrong_depth = matching.clone();
        wrong_depth[0].depth = 3;
        let error = validate_cpu_trace_alignment(&integer, &wrong_depth, 1).unwrap_err();
        assert!(error.contains("shape"), "{error}");

        let mut wrong_head_count = matching.clone();
        wrong_head_count[0].heads.pop();
        let error = validate_cpu_trace_alignment(&integer, &wrong_head_count, 1).unwrap_err();
        assert!(error.contains("shape"), "{error}");

        let mut wrong_kept = matching;
        wrong_kept[0].heads[0].kept[1].position = 4;
        let error = validate_cpu_trace_alignment(&integer, &wrong_kept, 1).unwrap_err();
        assert!(error.contains("choices"), "{error}");

        let mut wrong_query = vec![CpuHammingReadTrace {
            depth: 2,
            heads: vec![
                cpu_head(0, CpuHammingHeadKind::Copy, 99, 12, vec![1, 2]),
                cpu_head(1, CpuHammingHeadKind::Type, 13, 14, vec![3]),
            ],
        }];
        let error = validate_cpu_trace_alignment(&integer, &wrong_query, 1).unwrap_err();
        assert!(error.contains("choices"), "{error}");

        wrong_query[0].heads[0].query = 11;
        wrong_query[0].heads[0].key = 99;
        let error = validate_cpu_trace_alignment(&integer, &wrong_query, 1).unwrap_err();
        assert!(error.contains("choices"), "{error}");
    }
}
