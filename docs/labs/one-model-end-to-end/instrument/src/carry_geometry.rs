//! Deterministic CPU diagnosis of TRAIN question distances and the official first-reply carry.
//!
//! This module is an instrument. It trains nothing, changes no artifact, moves no bar and makes
//! no capability claim. The TRAIN census counts question-to-answer distances in the published
//! d3mwcap6 stream. The first-reply trace replays the carried artifact and its exact `-C` control
//! through `CpuFbsForward`, recording the Hamming choices and the carry's exact value-row terms.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead, CpuHammingHeadKind, CpuHammingReadTrace};
use uor_r4_integer::stack::{
    drop_hamming_carry_uorlut01, read_hamming_uorlut01, IntegerStackModel,
};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const CENSUS_SCHEMA: &str = "uor-r4.real-carry-train-census/1";
const TRACE_SCHEMA: &str = "uor-r4.real-carry-first-reply/1";
const TRAIN_SCHEMA: &str = "uor-r4-scale-probe-mix/v1";
const PANEL_SCHEMA: &str = "uor-r4.m2-fresh256-panel-manifest/1";
const BOS: u32 = 0;
const EOS: u32 = 1;
const UORT_HEADER: usize = 64;
const CONTEXT: usize = 192;
const WINDOW_ROWS: [usize; 14] = [0, 1, 2, 4, 8, 16, 24, 32, 64, 128, 191, 192, 512, 1_024];
const RECEIPT_SCHEMA: &str = "uor-r4.real-carry-geometry-receipt/1";
const USAGE: &str = "usage:\n  real-carry-geometry train-census --tokens FILE --mask FILE --manifest FILE --manifest-sha256 HEX --tokenizer FILE --tokenizer-sha256 HEX --carry-rows N [--skip-documents N --limit-documents N] --out NEW_FILE\n  real-carry-geometry first-reply --carried FILE --carried-sha256 HEX --minus-c FILE --minus-c-sha256 HEX --carried-report FILE --minus-c-report FILE --carried-rank FILE --minus-c-rank FILE --inputs FILE --labels FILE --plan FILE --panel-manifest FILE --panel-manifest-sha256 HEX --tokenizer FILE --expected-carried-first-lines N --expected-minus-c-first-lines N [--limit N] --out NEW_FILE\n  real-carry-geometry receipt --train-census FILE [--train-census-repeat FILE] --s1 FILE [--s1-repeat FILE] --s2 FILE [--s2-repeat FILE] --out NEW_FILE";

type Result<T> = std::result::Result<T, String>;

pub fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let args = args.collect::<Vec<_>>();
    let Some((mode, rest)) = args.split_first() else {
        return Err(USAGE.to_owned());
    };
    let flags = parse_flags(rest)?;
    match mode.as_str() {
        "train-census" => train_census(&flags),
        "first-reply" => first_reply(&flags),
        "receipt" => receipt(&flags),
        other => Err(format!("unknown mode {other:?}\n{USAGE}")),
    }
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(format!("each flag needs one value\n{USAGE}"));
    }
    let mut values = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let name = pair[0]
            .strip_prefix("--")
            .ok_or_else(|| format!("flag {:?} does not start with --\n{USAGE}", pair[0]))?;
        if values.insert(name.to_owned(), pair[1].clone()).is_some() {
            return Err(format!("--{name} given twice"));
        }
    }
    Ok(values)
}

fn flag<'a>(flags: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("--{name} is required\n{USAGE}"))
}

fn optional_usize(flags: &BTreeMap<String, String>, name: &str) -> Result<Option<usize>> {
    flags
        .get(name)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| format!("--{name} is not a count"))
        })
        .transpose()
}

fn sha256_file(path: &str) -> Result<String> {
    let bytes = fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn executable_sha256() -> Result<String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("current executable: {error}"))?;
    let path = executable
        .to_str()
        .ok_or("current executable path is not UTF-8")?;
    sha256_file(path)
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn parse_json(path: &str, label: &str) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).map_err(|error| format!("{path}: {error}"))?)
        .map_err(|error| format!("{label} {path}: {error}"))
}

fn write_new(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn invocation() -> Result<Value> {
    let cwd = std::env::current_dir().map_err(|error| format!("current directory: {error}"))?;
    let mut argv = std::env::args().collect::<Vec<_>>();
    let mut at = 0usize;
    while at + 1 < argv.len() {
        if argv[at] == "--out" {
            argv[at + 1] = "<out>".to_owned();
        }
        at += 1;
    }
    Ok(json!({
        "argv": argv,
        "cwd": cwd,
    }))
}

#[derive(Clone, Debug)]
struct Uort {
    tokens: Vec<u32>,
    vocabulary: usize,
    source_sha256: String,
}

fn read_uort(path: &str) -> Result<Uort> {
    let bytes = fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    if bytes.len() < UORT_HEADER || &bytes[..4] != b"UORT" {
        return Err(format!("{path}: not a canonical UORT token file"));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| "UORT version")?);
    let count = u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "UORT count")?);
    let vocabulary =
        u32::from_le_bytes(bytes[16..20].try_into().map_err(|_| "UORT vocabulary")?) as usize;
    let expected = count
        .checked_mul(2)
        .and_then(|payload| payload.checked_add(UORT_HEADER as u64))
        .ok_or("UORT length overflows")?;
    if version != 1
        || expected != bytes.len() as u64
        || bytes[20..UORT_HEADER].iter().any(|&byte| byte != 0)
    {
        return Err(format!("{path}: UORT header or physical length differs"));
    }
    let tokens = bytes[UORT_HEADER..]
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect::<Vec<_>>();
    if let Some(&token) = tokens.iter().find(|&&token| token as usize >= vocabulary) {
        return Err(format!(
            "{path}: token {token} is outside vocabulary {vocabulary}"
        ));
    }
    Ok(Uort {
        tokens,
        vocabulary,
        source_sha256: sha256(&bytes),
    })
}

fn tokenizer(path: &str) -> Result<(ByteBpeTokenizer, String)> {
    let bytes = fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| format!("{path}: unsupported byte-level BPE tokenizer"))?;
    Ok((tokenizer, sha256(&bytes)))
}

#[derive(Clone, Debug)]
struct MarkerIds {
    user: Vec<u32>,
    assistant: Vec<u32>,
    newline: Vec<u32>,
    system: Vec<u32>,
}

impl MarkerIds {
    fn from_tokenizer(tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        let marker = |text| {
            let ids = tokenizer.encode(text);
            if ids.is_empty() {
                Err(format!("tokenizer encoded {text:?} to no tokens"))
            } else {
                Ok(ids)
            }
        };
        Ok(Self {
            user: marker("User:")?,
            assistant: marker("Assistant:")?,
            newline: marker("\n")?,
            system: marker("System:")?,
        })
    }
}

fn find_last(haystack: &[u32], needle: &[u32]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn decode(tokenizer: &ByteBpeTokenizer, tokens: &[u32], label: &str) -> Result<String> {
    String::from_utf8(tokenizer.decode_bytes(tokens))
        .map_err(|_| format!("{label}: decoded tokens are not UTF-8"))
}

#[derive(Clone, Debug)]
struct RootSpan {
    start: usize,
    end: usize,
    name: String,
    component: String,
}

fn root_spans(manifest: &Value) -> Result<Vec<RootSpan>> {
    let roots = manifest["roots"]
        .as_array()
        .ok_or("TRAIN manifest has no roots list")?;
    let mut start = 0usize;
    let mut spans = Vec::with_capacity(roots.len());
    for root in roots {
        let name = root["name"]
            .as_str()
            .ok_or("a TRAIN root has no name")?
            .to_owned();
        let count = root["tokens_taken"]
            .as_u64()
            .ok_or_else(|| format!("TRAIN root {name} has no tokens_taken"))?
            as usize;
        let end = start
            .checked_add(count)
            .ok_or("TRAIN root span overflows")?;
        let component = if name.starts_with("cap") {
            "mw-balp-capped-copy".to_owned()
        } else if name == "oasst2" {
            "d3-oasst2".to_owned()
        } else if name.starts_with("wiki-") {
            "d3-wikipedia".to_owned()
        } else {
            name.clone()
        };
        spans.push(RootSpan {
            start,
            end,
            name,
            component,
        });
        start = end;
    }
    Ok(spans)
}

fn component_at(spans: &[RootSpan], position: usize) -> Result<(&str, &str)> {
    spans
        .iter()
        .find(|span| span.start <= position && position < span.end)
        .map(|span| (span.name.as_str(), span.component.as_str()))
        .ok_or_else(|| format!("position {position} is outside every manifest root"))
}

#[derive(Clone, Debug)]
struct QuestionPair {
    document: usize,
    source: String,
    component: String,
    question_text: String,
    reply_text: String,
    question_start: usize,
    question_end: usize,
    answer_target: usize,
    scored_input: usize,
    first_answer_token: u32,
    first_answer_piece: String,
    value: Option<ValuePair>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct QuestionExclusions {
    assistant_reply_runs: usize,
    missing_assistant_marker: usize,
    assistant_marker_not_terminal: usize,
    missing_user_marker: usize,
    user_turn_not_a_question: usize,
    empty_reply: usize,
}

#[derive(Clone, Debug)]
struct ValuePair {
    row: usize,
    context_start: usize,
    context_end: usize,
    tokens: Vec<u32>,
    text: String,
}

#[derive(Clone, Debug, Serialize)]
struct DistanceStats {
    rows: usize,
    min: Option<usize>,
    p50: Option<usize>,
    p90: Option<usize>,
    p95: Option<usize>,
    p99: Option<usize>,
    max: Option<usize>,
    mean_x1000: Option<u128>,
    within_32: usize,
    within_64: usize,
    within_192: usize,
    histogram: BTreeMap<String, usize>,
}

fn distance_stats(values: impl Iterator<Item = usize>) -> DistanceStats {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    let quantile = |p: usize| {
        if values.is_empty() {
            None
        } else {
            Some(values[(values.len() - 1) * p / 100])
        }
    };
    let mut histogram = BTreeMap::new();
    for value in &values {
        let bucket = WINDOW_ROWS
            .iter()
            .copied()
            .find(|&limit| *value <= limit)
            .map(|limit| format!("<={limit}"))
            .unwrap_or_else(|| ">1024".to_owned());
        *histogram.entry(bucket).or_insert(0) += 1;
    }
    DistanceStats {
        rows: values.len(),
        min: values.first().copied(),
        p50: quantile(50),
        p90: quantile(90),
        p95: quantile(95),
        p99: quantile(99),
        max: values.last().copied(),
        mean_x1000: (!values.is_empty()).then(|| {
            values
                .iter()
                .map(|&value| value as u128 * 1_000)
                .sum::<u128>()
                / values.len() as u128
        }),
        within_32: values.iter().filter(|&&value| value <= 32).count(),
        within_64: values.iter().filter(|&&value| value <= 64).count(),
        within_192: values.iter().filter(|&&value| value <= 191).count(),
        histogram,
    }
}

fn infer_value(context: &[(usize, u32)], question: &[u32], reply: &[u32]) -> Option<ValuePair> {
    if context.is_empty() || question.is_empty() || reply.is_empty() {
        return None;
    }
    let context_start = context.len().saturating_sub(CONTEXT);
    let context = &context[context_start..];
    let mut context_last = BTreeMap::<Vec<u32>, (usize, usize)>::new();
    for width in [3usize, 2] {
        context_last.clear();
        for at in 0..context.len().saturating_sub(width - 1) {
            let span = context[at..at + width]
                .iter()
                .map(|(_, token)| *token)
                .collect::<Vec<_>>();
            let (absolute_start, _) = context[at];
            context_last.insert(span, (at, absolute_start));
        }
        let question_spans = (0..question.len().saturating_sub(width - 1))
            .map(|at| &question[at..at + width])
            .collect::<BTreeSet<_>>();
        for row in 0..reply.len().saturating_sub(width - 1) {
            let span = &reply[row..row + width];
            if question_spans.contains(span) {
                continue;
            }
            if let Some(&(context_at, absolute_start)) = context_last.get(span) {
                let (absolute_end, _) = context[context_at + width - 1];
                return Some(ValuePair {
                    row,
                    context_start: absolute_start,
                    context_end: absolute_end,
                    tokens: span.to_vec(),
                    text: String::new(),
                });
            }
        }
    }
    None
}

fn content_tokens(
    tokens: &[u32],
    mask: &[u8],
    start: usize,
    end: usize,
    markers: &MarkerIds,
) -> Vec<(usize, u32)> {
    let mut content = Vec::new();
    let mut at = start;
    while at < end {
        let mut marker_matched = false;
        for marker in [&markers.user, &markers.assistant, &markers.system] {
            if at + marker.len() <= end && &tokens[at..at + marker.len()] == marker.as_slice() {
                if marker == &markers.user && !content.is_empty() {
                    content.push((at, u32::MAX));
                }
                at += marker.len();
                marker_matched = true;
                break;
            }
        }
        if marker_matched {
            continue;
        }
        if mask[at] == 0 && !markers.newline.contains(&tokens[at]) && tokens[at] != BOS {
            content.push((at, tokens[at]));
        }
        at += 1;
    }
    content
}

#[allow(clippy::too_many_arguments)]
fn question_pairs(
    tokenizer: &ByteBpeTokenizer,
    tokens: &[u32],
    mask: &[u8],
    markers: &MarkerIds,
    spans: &[RootSpan],
    document: usize,
    document_start: usize,
    document_end: usize,
    exclusions: &mut QuestionExclusions,
) -> Result<Vec<QuestionPair>> {
    let mut pairs = Vec::new();
    let mut at = document_start;
    let mut unmasked_start = document_start;
    while at < document_end {
        if mask[at] == 0 {
            at += 1;
            continue;
        }
        let answer_target = at;
        while at < document_end && mask[at] == 1 {
            at += 1;
        }
        exclusions.assistant_reply_runs += 1;
        let run = &tokens[unmasked_start..answer_target];
        let Some(assistant_at) = find_last(run, &markers.assistant) else {
            exclusions.missing_assistant_marker += 1;
            unmasked_start = at;
            continue;
        };
        if assistant_at + markers.assistant.len() != run.len() {
            exclusions.assistant_marker_not_terminal += 1;
            unmasked_start = at;
            continue;
        }
        let Some(user_at) = find_last(&run[..assistant_at], &markers.user) else {
            exclusions.missing_user_marker += 1;
            unmasked_start = at;
            continue;
        };
        let mut question_start = unmasked_start + user_at + markers.user.len();
        let mut question_end = unmasked_start + assistant_at;
        if question_end > question_start
            && markers
                .newline
                .contains(&tokens[question_end.saturating_sub(1)])
        {
            question_end -= 1;
        }
        while question_start < question_end
            && decode(
                tokenizer,
                &tokens[question_start..question_start + 1],
                "question whitespace",
            )?
            .trim()
            .is_empty()
        {
            question_start += 1;
        }
        let question_text = decode(tokenizer, &tokens[question_start..question_end], "question")?;
        if !question_text.trim().ends_with('?') {
            exclusions.user_turn_not_a_question += 1;
            unmasked_start = at;
            continue;
        }
        let mut reply_end = answer_target;
        while reply_end < document_end && mask[reply_end] == 1 {
            if tokens[reply_end] == EOS {
                break;
            }
            reply_end += 1;
        }
        if reply_end == answer_target {
            exclusions.empty_reply += 1;
            unmasked_start = at;
            continue;
        }
        let reply = &tokens[answer_target..reply_end];
        let reply_text = decode(tokenizer, reply, "reply")?;
        let context = content_tokens(tokens, mask, document_start, question_start, markers);
        let value = match infer_value(&context, &tokens[question_start..question_end], reply) {
            Some(mut found) => {
                found.row += answer_target;
                found.text =
                    String::from_utf8_lossy(&tokenizer.decode_bytes(&found.tokens)).into_owned();
                Some(found)
            }
            None => None,
        };
        let (source, component) = component_at(spans, answer_target)?;
        let first_answer_piece = decode(
            tokenizer,
            &tokens[answer_target..answer_target + 1],
            "first answer token",
        )?;
        pairs.push(QuestionPair {
            document,
            source: source.to_owned(),
            component: component.to_owned(),
            question_text,
            reply_text,
            question_start,
            question_end,
            answer_target,
            scored_input: answer_target - 1,
            first_answer_token: tokens[answer_target],
            first_answer_piece,
            value,
        });
        unmasked_start = at;
    }
    Ok(pairs)
}

#[derive(Clone, Debug, Default, Serialize)]
struct ComponentStats {
    questions: usize,
    first_scored_within_32: usize,
    first_scored_within_192: usize,
    heuristic_spans_identified: usize,
    heuristic_span_within_32: usize,
    heuristic_span_within_192: usize,
}

fn census_controls(pairs: &[QuestionPair], carry_rows: usize) -> Value {
    let mut shifted_rows = 0usize;
    let mut shifted_within = 0usize;
    let mut shifted_document_start = 0usize;
    for pair in pairs {
        match pair.question_end.checked_sub(carry_rows) {
            Some(shifted_question) => {
                shifted_rows += 1;
                shifted_within +=
                    usize::from(pair.scored_input.saturating_sub(shifted_question) <= carry_rows);
            }
            None => shifted_document_start += 1,
        }
    }

    let mut documents = BTreeMap::<usize, Vec<&QuestionPair>>::new();
    for pair in pairs {
        documents.entry(pair.document).or_default().push(pair);
    }
    let mut previous_rows = 0usize;
    let mut previous_within = 0usize;
    let mut nonprior_pairs = 0usize;
    for document in documents.values_mut() {
        document.sort_by_key(|pair| pair.question_start);
        for current in document.iter().skip(1) {
            let previous = document
                .iter()
                .rev()
                .find(|candidate| candidate.question_start < current.question_start);
            match previous {
                Some(previous) if previous.question_end < current.scored_input => {
                    previous_rows += 1;
                    previous_within += usize::from(
                        current.scored_input.saturating_sub(previous.question_end) <= carry_rows,
                    );
                }
                _ => nonprior_pairs += 1,
            }
        }
    }

    json!({
        "question_end_offset_32_earlier": {
            "rows": shifted_rows,
            "within_carry": shifted_within,
            "clamped_to_document_start": shifted_document_start,
            "interpretation": "moves the same question earlier by the carry reach; this is an offset perturbation, not the primary result",
        },
        "previous_question_answer_pairs_in_same_document": {
            "rows": previous_rows,
            "within_carry": previous_within,
            "excluded_no_prior_question_or_nonprior": nonprior_pairs,
            "excluded_single_question_documents": documents.values().filter(|rows| rows.len() < 2).count(),
            "interpretation": "only a strictly earlier question may be paired with a scored reply; future questions are never counted",
        },
    })
}

fn train_census(flags: &BTreeMap<String, String>) -> Result<()> {
    let tokens_path = flag(flags, "tokens")?;
    let mask_path = flag(flags, "mask")?;
    let manifest_path = flag(flags, "manifest")?;
    let tokenizer_path = flag(flags, "tokenizer")?;
    let expected_manifest = flag(flags, "manifest-sha256")?;
    let expected_tokenizer = flag(flags, "tokenizer-sha256")?;
    let carry_rows = optional_usize(flags, "carry-rows")?.ok_or("--carry-rows is required")?;
    if carry_rows == 0 {
        return Err("--carry-rows must be positive".to_owned());
    }
    let out = PathBuf::from(flag(flags, "out")?);

    let train = read_uort(tokens_path)?;
    let mask = fs::read(mask_path).map_err(|error| format!("{mask_path}: {error}"))?;
    if mask.len() != train.tokens.len() {
        return Err(format!(
            "{} tokens but {} mask bytes",
            train.tokens.len(),
            mask.len()
        ));
    }
    let manifest_sha256 = sha256_file(manifest_path)?;
    if expected_manifest != manifest_sha256 {
        return Err(format!(
            "TRAIN manifest hash {expected_manifest} differs from {manifest_sha256}"
        ));
    }
    let manifest = parse_json(manifest_path, "TRAIN manifest")?;
    if manifest["schema"].as_str() != Some(TRAIN_SCHEMA) {
        return Err(format!("TRAIN manifest schema is not {TRAIN_SCHEMA}"));
    }
    let declared_tokens = manifest["tokens_sha256"]
        .as_str()
        .ok_or("TRAIN manifest has no tokens_sha256")?;
    let declared_mask = manifest["mask_sha256"]
        .as_str()
        .ok_or("TRAIN manifest has no mask_sha256")?;
    if declared_tokens != train.source_sha256 {
        return Err(format!(
            "TRAIN token hash {declared_tokens} differs from {}",
            train.source_sha256
        ));
    }
    let mask_sha256 = sha256(&mask);
    if declared_mask != mask_sha256 {
        return Err(format!(
            "TRAIN mask hash {declared_mask} differs from {mask_sha256}"
        ));
    }
    let (tokenizer, tokenizer_sha256) = tokenizer(tokenizer_path)?;
    if expected_tokenizer != tokenizer_sha256 {
        return Err(format!(
            "TRAIN tokenizer hash {expected_tokenizer} differs from {tokenizer_sha256}"
        ));
    }
    if train.vocabulary > 65_536 {
        return Err("TRAIN UORT vocabulary exceeds u16".to_owned());
    }
    let markers = MarkerIds::from_tokenizer(&tokenizer)?;
    let spans = root_spans(&manifest)?;
    if spans
        .last()
        .is_none_or(|span| span.end != train.tokens.len())
    {
        return Err(format!(
            "TRAIN manifest roots end at {}, tokens end at {}",
            spans.last().map_or(0, |span| span.end),
            train.tokens.len()
        ));
    }

    let starts = train
        .tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| **token == BOS)
        .map(|(at, _)| at)
        .collect::<Vec<_>>();
    let mut pairs = Vec::new();
    let mut exclusions = QuestionExclusions::default();
    let mut documents_scanned = 0usize;
    let document_skip = optional_usize(flags, "skip-documents")?.unwrap_or(0);
    let document_limit = optional_usize(flags, "limit-documents")?.unwrap_or(starts.len());
    for (document, &start) in starts
        .iter()
        .enumerate()
        .skip(document_skip)
        .take(document_limit)
    {
        documents_scanned += 1;
        let end = starts
            .get(document + 1)
            .copied()
            .unwrap_or(train.tokens.len());
        pairs.extend(question_pairs(
            &tokenizer,
            &train.tokens,
            &mask,
            &markers,
            &spans,
            document,
            start,
            end,
            &mut exclusions,
        )?);
    }

    let first_scored = pairs
        .iter()
        .map(|pair| pair.scored_input.saturating_sub(pair.question_end))
        .collect::<Vec<_>>();
    let first_target = pairs
        .iter()
        .map(|pair| pair.answer_target.saturating_sub(pair.question_end))
        .collect::<Vec<_>>();
    let start_to_scored = pairs
        .iter()
        .map(|pair| pair.scored_input.saturating_sub(pair.question_start))
        .collect::<Vec<_>>();
    let values = pairs
        .iter()
        .filter_map(|pair| {
            pair.value.as_ref().map(|value| {
                (
                    value.row.saturating_sub(pair.question_end),
                    value.row.saturating_sub(value.context_end),
                    value.row.saturating_sub(value.context_start),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut components = BTreeMap::<String, ComponentStats>::new();
    let mut sources = BTreeMap::<String, ComponentStats>::new();
    let mut first_tokens = BTreeMap::<String, usize>::new();
    let mut value_span_lengths = BTreeMap::<usize, usize>::new();
    for pair in &pairs {
        let scored_distance = pair.scored_input.saturating_sub(pair.question_end);
        let stats = components.entry(pair.component.clone()).or_default();
        stats.questions += 1;
        stats.first_scored_within_32 += usize::from(scored_distance <= carry_rows);
        stats.first_scored_within_192 += usize::from(scored_distance < CONTEXT);
        if let Some(value) = &pair.value {
            let value_distance = value.row.saturating_sub(pair.question_end);
            stats.heuristic_spans_identified += 1;
            stats.heuristic_span_within_32 += usize::from(value_distance <= carry_rows);
            stats.heuristic_span_within_192 += usize::from(value_distance < CONTEXT);
            *value_span_lengths.entry(value.tokens.len()).or_insert(0) += 1;
        }
        let source_stats = sources.entry(pair.source.clone()).or_default();
        source_stats.questions += 1;
        source_stats.first_scored_within_32 += usize::from(scored_distance <= carry_rows);
        source_stats.first_scored_within_192 += usize::from(scored_distance < CONTEXT);
        if let Some(value) = &pair.value {
            let value_distance = value.row.saturating_sub(pair.question_end);
            source_stats.heuristic_spans_identified += 1;
            source_stats.heuristic_span_within_32 += usize::from(value_distance <= carry_rows);
            source_stats.heuristic_span_within_192 += usize::from(value_distance < CONTEXT);
        }
        *first_tokens
            .entry(format!(
                "{} {:?}",
                pair.first_answer_token, pair.first_answer_piece
            ))
            .or_insert(0) += 1;
    }

    let report = json!({
        "schema": CENSUS_SCHEMA,
        "verdict": "instrument-distance-census",
        "claim": "none; TRAIN geometry only",
        "command": invocation()?,
        "inputs": {
            "tokens": tokens_path,
            "tokens_sha256": train.source_sha256,
            "mask": mask_path,
            "mask_sha256": mask_sha256,
            "manifest": manifest_path,
            "manifest_sha256": manifest_sha256,
            "tokenizer": tokenizer_path,
            "tokenizer_sha256": tokenizer_sha256,
            "documents": starts.len(),
            "documents_scanned": documents_scanned,
            "tokens": train.tokens.len(),
            "vocabulary": train.vocabulary,
        },
        "method": {
            "question": "the last User turn before an Assistant reply whose content starts with response mask 1 and whose decoded user content ends in ?",
            "question_row": "the last user-content token before the separator and Assistant marker",
            "scored_value_row": "the last Assistant-marker token, which is the input row whose next-token target is the first masked reply token",
            "first_answer_row": "the first masked reply token",
            "repeated_context_span": "the earliest two- or three-token reply span seen in the preceding 192 context tokens and absent from the question; one-token spans are excluded. This is a nonsemantic control, not an identified answer value",
            "window_coverage": "a distance below 192 can be placed in one training window; carry reach is inclusive",
        },
        "carry_rows": carry_rows,
        "questions": pairs.len(),
        "exclusions": exclusions,
        "first_scored_input": distance_stats(first_scored.iter().copied()),
        "first_answer_target": distance_stats(first_target.iter().copied()),
        "question_start_to_scored_input": distance_stats(start_to_scored.iter().copied()),
        "heuristic_repeated_context_span_row": {
            "status": "not-semantic",
            "interpretation": "Decoded examples are required before use. This heuristic can select a key or template phrase instead of the answer value, so it supports no value-reach or capability conclusion.",
            "identified": values.len(),
            "excluded": pairs.len().saturating_sub(values.len()),
            "question_end_to_repeated_span_row": distance_stats(values.iter().map(|(question, _, _)| *question)),
            "context_span_end_to_repeated_span_row": distance_stats(values.iter().map(|(_, copy, _)| *copy)),
            "context_span_start_to_repeated_span_row": distance_stats(values.iter().map(|(_, _, span)| *span)),
        },
        "components": components,
        "sources": sources,
        "first_answer_tokens": first_tokens,
        "heuristic_repeated_span_lengths": value_span_lengths,
        "heuristic_repeated_span_examples": {
            "mw-balp-capped-copy": pairs.iter().filter(|pair| pair.component == "mw-balp-capped-copy").filter_map(|pair| pair.value.as_ref().map(|value| json!({
                "question": pair.question_text,
                "reply": pair.reply_text,
                "value": value.text,
                "question_end_to_value_row": value.row.saturating_sub(pair.question_end),
            }))).take(20).collect::<Vec<_>>(),
            "d3-oasst2-control": pairs.iter().filter(|pair| pair.component == "d3-oasst2").filter_map(|pair| pair.value.as_ref().map(|value| json!({
                "question": pair.question_text,
                "reply": pair.reply_text,
                "value": value.text,
                "question_end_to_value_row": value.row.saturating_sub(pair.question_end),
            }))).take(5).collect::<Vec<_>>(),
        },
        "controls": census_controls(&pairs, carry_rows),
    });
    write_new(&out, &report)
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
#[serde(deny_unknown_fields)]
struct FrozenAnswers {
    accepted: Vec<String>,
    #[allow(dead_code)]
    intent: String,
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
struct ReplyReport {
    schema: String,
    inputs: ReplyReportInputs,
    rows: Vec<ReplyReportRow>,
}

#[derive(Deserialize)]
struct ReplyReportInputs {
    artifact_sha256: String,
}

#[derive(Deserialize)]
struct ReplyReportRow {
    id: String,
    reply_text: String,
}

#[derive(Clone, Debug, Serialize)]
struct NearestRank {
    target_token: u32,
    chosen_token: u32,
    rank: usize,
    raw_rank: usize,
    raw_nearest_token: u32,
    point_key_override: bool,
    candidate_count: usize,
}

#[derive(Clone, Debug, Serialize)]
struct PromptProbe {
    chosen_token: u32,
    asked: NearestRank,
    other: NearestRank,
    asked_top1: bool,
    other_top1: bool,
    point_key_overrides: usize,
    query_q12: Vec<i32>,
}

#[derive(Clone, Debug, Serialize)]
struct PromptMembership {
    row: usize,
    token: u32,
    last_occurrence: Option<usize>,
    rows_back: Option<usize>,
    inside_carry: bool,
}

#[derive(Clone, Debug, Serialize)]
struct CarryTerm {
    lag: usize,
    row: usize,
    token: u32,
    gain_q16: i32,
    base_ask_l1: u128,
    term_l1: u128,
}

fn render_case(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    case: &InputCase,
) -> Result<Vec<u32>> {
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
    let content = parts.join(" ");
    Ok(encoder
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: &content,
        }])
        .tokens)
}

fn reference_ids(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    answers: &FrozenAnswers,
) -> Result<Vec<u32>> {
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
    let ids = encoded
        .tokens
        .iter()
        .zip(encoded.response_mask)
        .filter_map(|(&token, mask)| (mask == 1).then_some(token))
        .collect::<Vec<_>>();
    if ids.last().copied() != Some(EOS) {
        return Err(format!("reference answer {answer:?} does not end in EOS"));
    }
    if tokenizer.decode_bytes(&ids).is_empty() {
        return Err("reference answer encodes to no bytes".to_owned());
    }
    Ok(ids)
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

fn last_occurrence(tokens: &[u32], target: u32) -> Option<usize> {
    tokens.iter().rposition(|&token| token == target)
}

fn membership(tokens: &[u32], token: u32, carry_rows: usize) -> PromptMembership {
    let row = tokens.len().saturating_sub(1);
    let last_occurrence = last_occurrence(tokens, token);
    let rows_back = last_occurrence.map(|at| row.saturating_sub(at));
    PromptMembership {
        row,
        token,
        last_occurrence,
        rows_back,
        inside_carry: rows_back.is_some_and(|back| (1..=carry_rows).contains(&back)),
    }
}

fn carry_terms(
    tokens: &[u32],
    table: &[i16],
    ask: &[i16],
    reach: usize,
    gains: &[i32],
    width: usize,
) -> Vec<CarryTerm> {
    let row = tokens.len().saturating_sub(1);
    let terms = (1..=reach.min(row))
        .map(|lag| {
            let at = row - lag;
            let token = tokens[at];
            let gain = gains[token as usize];
            let base_ask_l1 = (0..width)
                .map(|coordinate| {
                    let index = token as usize * width + coordinate;
                    u128::from(
                        i64::from(table[index]).unsigned_abs()
                            + i64::from(ask[index]).unsigned_abs(),
                    )
                })
                .sum::<u128>();
            CarryTerm {
                lag,
                row: at,
                token,
                gain_q16: gain,
                base_ask_l1,
                term_l1: u128::from(gain.unsigned_abs()) * base_ask_l1,
            }
        })
        .collect::<Vec<_>>();
    terms.into_iter().rev().collect()
}

fn query_l1(left: &[i32], right: &[i32]) -> Result<u128> {
    if left.len() != right.len() {
        return Err("query lengths differ".to_owned());
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(&left, &right)| u128::from(left.abs_diff(right)))
        .sum())
}

fn reply_map<'a>(
    report: &'a ReplyReport,
    name: &str,
    expected_rows: usize,
) -> Result<BTreeMap<&'a str, &'a str>> {
    let rows = report
        .rows
        .iter()
        .map(|row| (row.id.as_str(), row.reply_text.as_str()))
        .collect::<BTreeMap<_, _>>();
    if rows.len() != report.rows.len() {
        return Err(format!("{name} repeats a case id"));
    }
    if rows.len() != expected_rows {
        return Err(format!("{name} does not hold every label case"));
    }
    Ok(rows)
}

fn first_line_matches(text: &str, answers: &FrozenAnswers) -> bool {
    let line = text.lines().next().unwrap_or(text);
    answers.accepted.iter().any(|answer| {
        let needle = answer.trim_end_matches(['.', '!', '?']);
        !needle.is_empty()
            && line.match_indices(needle).any(|(start, _)| {
                let before = line[..start].chars().next_back();
                let end = start + needle.len();
                let after = line[end..].chars().next();
                let boundary = |character: Option<char>| {
                    character
                        .is_none_or(|character| !character.is_alphanumeric() && character != '_')
                };
                boundary(before) && boundary(after)
            })
    })
}

fn first_ranks(
    report: &Value,
    artifact_sha256: &str,
    name: &str,
    expected_rows: usize,
) -> Result<BTreeMap<String, (usize, usize)>> {
    if report["inputs"]["artifact_sha256"].as_str() != Some(artifact_sha256) {
        return Err(format!("{name} names another artifact"));
    }
    let rows = report["paired_first"]["rows"]
        .as_array()
        .ok_or_else(|| format!("{name} has no paired-first rows"))?;
    if rows.len() != expected_rows {
        return Err(format!(
            "{name} has {} paired-first rows, not {expected_rows}",
            rows.len()
        ));
    }
    let mut ranks = BTreeMap::new();
    for row in rows {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| format!("{name} has a paired-first row without an id"))?;
        let asked = row["asked_rank"]
            .as_u64()
            .ok_or_else(|| format!("{name}/{id} has no asked rank"))? as usize;
        let other = row["other_rank"]
            .as_u64()
            .ok_or_else(|| format!("{name}/{id} has no other rank"))? as usize;
        if ranks.insert(id.to_owned(), (asked, other)).is_some() {
            return Err(format!("{name} repeats case {id}"));
        }
    }
    Ok(ranks)
}

fn kept_positions(read: &CpuHammingReadTrace) -> BTreeMap<usize, Vec<(u32, u32, u64)>> {
    read.heads
        .iter()
        .filter(|head| head.kind == CpuHammingHeadKind::Type)
        .map(|head| {
            (
                head.head,
                head.kept
                    .iter()
                    .map(|position| (position.position, position.distance, position.weight_q31))
                    .collect(),
            )
        })
        .collect()
}

#[derive(Clone, Debug, Serialize)]
struct CaseReading {
    id: String,
    pair_id: String,
    stratum: String,
    prompt_tokens: usize,
    asked_token: u32,
    other_token: u32,
    same_first_token: bool,
    carried_asked_first_line: bool,
    carried_other_first_line: bool,
    minus_c_asked_first_line: bool,
    minus_c_other_first_line: bool,
    carried: PromptProbe,
    minus_c: PromptProbe,
    chosen_token_changed: bool,
    asked_rank_changed: bool,
    other_rank_changed: bool,
    chosen_or_rank_changed: bool,
    query_changed_coordinates: usize,
    query_l1: u128,
    type_head_kept_changed: Vec<usize>,
    asked_membership: PromptMembership,
    other_membership: PromptMembership,
    carry_terms: Vec<CarryTerm>,
    carried_trace: Vec<CpuHammingReadTrace>,
    minus_c_trace: Vec<CpuHammingReadTrace>,
}

fn probe(
    template: &CpuFbsForward,
    prompt: &[u32],
    asked_token: u32,
    other_token: u32,
) -> Result<(PromptProbe, Vec<CpuHammingReadTrace>, CpuHammingReadTrace)> {
    let mut forward = template.clone();
    forward.reset();
    let mut chosen = None;
    for (index, &token) in prompt.iter().enumerate() {
        let last = index + 1 == prompt.len();
        if last {
            forward.mark_reply_start();
            forward
                .set_hamming_trace(true)
                .map_err(|error| error.to_string())?;
        }
        let step = forward
            .step(token, CpuFbsHead::Nearest)
            .map_err(|error| error.to_string())?;
        if last {
            chosen = Some(step);
        }
    }
    let step = chosen.ok_or("empty prompt")?;
    let trace = forward
        .take_hamming_trace()
        .ok_or("Hamming trace was not recorded")?;
    let readout = trace
        .iter()
        .max_by_key(|read| read.depth)
        .cloned()
        .ok_or("no Hamming read was traced")?;
    let asked = nearest_rank(
        &step.query_q12,
        forward.shared_table(),
        forward.width(),
        asked_token,
        step.token,
    )?;
    let other = nearest_rank(
        &step.query_q12,
        forward.shared_table(),
        forward.width(),
        other_token,
        step.token,
    )?;
    let point_key_overrides =
        usize::from(asked.point_key_override) + usize::from(other.point_key_override);
    Ok((
        PromptProbe {
            chosen_token: step.token,
            asked_top1: asked.rank == 1,
            other_top1: other.rank == 1,
            asked,
            other,
            point_key_overrides,
            query_q12: step.query_q12,
        },
        trace,
        readout,
    ))
}

fn first_reply(flags: &BTreeMap<String, String>) -> Result<()> {
    let carried_path = flag(flags, "carried")?;
    let minus_c_path = flag(flags, "minus-c")?;
    let inputs_path = flag(flags, "inputs")?;
    let labels_path = flag(flags, "labels")?;
    let plan_path = flag(flags, "plan")?;
    let carried_report_path = flag(flags, "carried-report")?;
    let minus_c_report_path = flag(flags, "minus-c-report")?;
    let carried_rank_path = flag(flags, "carried-rank")?;
    let minus_c_rank_path = flag(flags, "minus-c-rank")?;
    let panel_manifest_path = flag(flags, "panel-manifest")?;
    let expected_panel_manifest = flag(flags, "panel-manifest-sha256")?;
    let tokenizer_path = flag(flags, "tokenizer")?;
    let out = PathBuf::from(flag(flags, "out")?);
    let limit = optional_usize(flags, "limit")?;
    let expected_carried_first_lines = optional_usize(flags, "expected-carried-first-lines")?
        .ok_or("--expected-carried-first-lines is required")?;
    let expected_minus_c_first_lines = optional_usize(flags, "expected-minus-c-first-lines")?
        .ok_or("--expected-minus-c-first-lines is required")?;

    let carried = fs::read(carried_path).map_err(|error| format!("{carried_path}: {error}"))?;
    let minus_c = fs::read(minus_c_path).map_err(|error| format!("{minus_c_path}: {error}"))?;
    let carried_sha256 = sha256(&carried);
    let minus_c_sha256 = sha256(&minus_c);
    let expected_carried = flag(flags, "carried-sha256")?;
    if expected_carried != carried_sha256 {
        return Err(format!(
            "carried artifact hash {expected_carried} differs from {carried_sha256}"
        ));
    }
    let expected_minus_c = flag(flags, "minus-c-sha256")?;
    if expected_minus_c != minus_c_sha256 {
        return Err(format!(
            "minus-C artifact hash {expected_minus_c} differs from {minus_c_sha256}"
        ));
    }
    let reproduced =
        drop_hamming_carry_uorlut01(&carried).map_err(|error| format!("carry cut: {error}"))?;
    if reproduced != minus_c {
        return Err("the supplied minus-C artifact is not the exact carry cut".to_owned());
    }
    let carried_report: ReplyReport = serde_json::from_slice(
        &fs::read(carried_report_path)
            .map_err(|error| format!("{carried_report_path}: {error}"))?,
    )
    .map_err(|error| format!("{carried_report_path}: {error}"))?;
    let minus_c_report: ReplyReport = serde_json::from_slice(
        &fs::read(minus_c_report_path)
            .map_err(|error| format!("{minus_c_report_path}: {error}"))?,
    )
    .map_err(|error| format!("{minus_c_report_path}: {error}"))?;
    if carried_report.schema != "uor-r4.served-reply-panel/1"
        || minus_c_report.schema != "uor-r4.served-reply-panel/1"
    {
        return Err("a reply report is not a served-reply-panel report".to_owned());
    }
    if carried_report.inputs.artifact_sha256 != carried_sha256 {
        return Err("the carried reply report names another artifact".to_owned());
    }
    if minus_c_report.inputs.artifact_sha256 != minus_c_sha256 {
        return Err("the minus-C reply report names another artifact".to_owned());
    }
    let model = IntegerStackModel::parse(&carried).map_err(|error| format!("artifact: {error}"))?;
    if !model.shape().head_nearest() {
        return Err("first-reply trace needs the nearest serving head".to_owned());
    }
    let hamming = read_hamming_uorlut01(&carried)
        .map_err(|error| format!("artifact Hamming reads: {error}"))?
        .ok_or("carried artifact has no Hamming reads")?;
    let readout = hamming
        .reads
        .iter()
        .max_by_key(|read| read.depth)
        .ok_or("carried artifact has no readout read")?;
    let ask = readout.ask.as_ref().ok_or("readout has no ask table")?;
    let (reach, gains) = ask.carry.as_ref().ok_or("readout has no question carry")?;

    let panel_manifest_sha256 = sha256_file(panel_manifest_path)?;
    if expected_panel_manifest != panel_manifest_sha256 {
        return Err(format!(
            "panel manifest hash {expected_panel_manifest} differs from {panel_manifest_sha256}"
        ));
    }
    let manifest = parse_json(panel_manifest_path, "panel manifest")?;
    if manifest["schema"].as_str() != Some(PANEL_SCHEMA) {
        return Err(format!("panel manifest schema is not {PANEL_SCHEMA}"));
    }
    for (name, path) in [
        ("inputs.json", inputs_path),
        ("labels.json", labels_path),
        ("plan.json", plan_path),
    ] {
        let expected = manifest["files"][name]["sha256"]
            .as_str()
            .ok_or_else(|| format!("panel manifest has no hash for {name}"))?;
        let actual = sha256_file(path)?;
        if expected != actual {
            return Err(format!("{name} hash {expected} differs from {actual}"));
        }
    }
    let (tokenizer, tokenizer_sha256) = tokenizer(tokenizer_path)?;
    let declared_tokenizer = manifest["tokenizer_sha256"]
        .as_str()
        .ok_or("panel manifest has no tokenizer hash")?;
    if declared_tokenizer != tokenizer_sha256 {
        return Err(format!(
            "panel tokenizer hash {declared_tokenizer} differs from {tokenizer_sha256}"
        ));
    }
    let inputs: Inputs = serde_json::from_slice(
        &fs::read(inputs_path).map_err(|error| format!("{inputs_path}: {error}"))?,
    )
    .map_err(|error| format!("{inputs_path}: {error}"))?;
    let labels: Labels = serde_json::from_slice(
        &fs::read(labels_path).map_err(|error| format!("{labels_path}: {error}"))?,
    )
    .map_err(|error| format!("{labels_path}: {error}"))?;
    let plan: Plan = serde_json::from_slice(
        &fs::read(plan_path).map_err(|error| format!("{plan_path}: {error}"))?,
    )
    .map_err(|error| format!("{plan_path}: {error}"))?;
    if labels.protocol != SCHEMA_V2 {
        return Err(format!(
            "labels protocol {} is not {SCHEMA_V2}",
            labels.protocol
        ));
    }
    let protocol =
        DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|error| error.to_string())?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|error| error.to_string())?;
    let strata = plan
        .histories
        .iter()
        .map(|history| (format!("fresh-{}", history.id), history.stratum.clone()))
        .collect::<BTreeMap<_, _>>();
    let by_id = inputs
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    if by_id.len() != labels.cases.len() || inputs.cases.len() != labels.cases.len() {
        return Err("inputs and labels do not hold the same cases".to_owned());
    }
    let carried_replies = reply_map(&carried_report, "carried report", labels.cases.len())?;
    let minus_c_replies = reply_map(&minus_c_report, "minus-C report", labels.cases.len())?;
    let carried_ranks = first_ranks(
        &parse_json(carried_rank_path, "carried rank report")?,
        &carried_sha256,
        "carried rank report",
        labels.cases.len(),
    )?;
    let minus_c_ranks = first_ranks(
        &parse_json(minus_c_rank_path, "minus-C rank report")?,
        &minus_c_sha256,
        "minus-C rank report",
        labels.cases.len(),
    )?;
    let mut partners = BTreeMap::new();
    let mut by_pair = BTreeMap::<&str, Vec<&LabelCase>>::new();
    for label in &labels.cases {
        by_pair
            .entry(label.pair_id.as_str())
            .or_default()
            .push(label);
    }
    for (pair, labels) in by_pair {
        let [left, right] = labels.as_slice() else {
            return Err(format!("pair {pair} does not hold two cases"));
        };
        partners.insert(left.id.as_str(), *right);
        partners.insert(right.id.as_str(), *left);
    }

    let carried_forward =
        CpuFbsForward::from_artifact(&carried).map_err(|error| error.to_string())?;
    let minus_c_forward =
        CpuFbsForward::from_artifact(&minus_c).map_err(|error| error.to_string())?;
    if carried_forward.shared_table() != minus_c_forward.shared_table() {
        return Err("carried and minus-C writer tables differ".to_owned());
    }
    let width = carried_forward.width();
    let mut cases = Vec::new();
    for label in labels.cases.iter().take(limit.unwrap_or(usize::MAX)) {
        let case = by_id
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no input case", label.id))?;
        let other = partners
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no pair partner", label.id))?;
        let prompt = render_case(&tokenizer, &encoder, case)?;
        let asked_ids = reference_ids(&tokenizer, &encoder, &label.answers)?;
        let other_ids = reference_ids(&tokenizer, &encoder, &other.answers)?;
        let asked_token = *asked_ids.first().ok_or("asked reference is empty")?;
        let other_token = *other_ids.first().ok_or("other reference is empty")?;
        let (carried_probe, carried_trace, carried_readout) =
            probe(&carried_forward, &prompt, asked_token, other_token)?;
        let (minus_probe, minus_trace, minus_readout) =
            probe(&minus_c_forward, &prompt, asked_token, other_token)?;
        let expected_carried = carried_ranks
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no carried rank", label.id))?;
        let expected_minus_c = minus_c_ranks
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no minus-C rank", label.id))?;
        let (expected_carried_asked, expected_carried_other) = *expected_carried;
        if (carried_probe.asked.rank, carried_probe.other.rank)
            != (expected_carried_asked, expected_carried_other)
        {
            return Err(format!(
                "{}: carried ranks ({}, {}) differ from the official ({}, {})",
                label.id,
                carried_probe.asked.rank,
                carried_probe.other.rank,
                expected_carried_asked,
                expected_carried_other
            ));
        }
        let (expected_minus_c_asked, expected_minus_c_other) = *expected_minus_c;
        if (minus_probe.asked.rank, minus_probe.other.rank)
            != (expected_minus_c_asked, expected_minus_c_other)
        {
            return Err(format!(
                "{}: minus-C ranks ({}, {}) differ from the official ({}, {})",
                label.id,
                minus_probe.asked.rank,
                minus_probe.other.rank,
                expected_minus_c_asked,
                expected_minus_c_other
            ));
        }
        let carried_text = carried_replies
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no carried reply", label.id))?;
        let minus_c_text = minus_c_replies
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no minus-C reply", label.id))?;
        let carried_asked_first_line = first_line_matches(carried_text, &label.answers);
        let carried_other_first_line = first_line_matches(carried_text, &other.answers);
        let minus_c_asked_first_line = first_line_matches(minus_c_text, &label.answers);
        let minus_c_other_first_line = first_line_matches(minus_c_text, &other.answers);
        let query_changed_coordinates = carried_probe
            .query_q12
            .iter()
            .zip(&minus_probe.query_q12)
            .filter(|(left, right)| left != right)
            .count();
        let carried_kept = kept_positions(&carried_readout);
        let minus_kept = kept_positions(&minus_readout);
        let type_head_kept_changed = carried_kept
            .iter()
            .filter(|(head, positions)| minus_kept.get(head) != Some(positions))
            .map(|(&head, _)| head)
            .collect::<Vec<_>>();
        cases.push(CaseReading {
            id: label.id.clone(),
            pair_id: label.pair_id.clone(),
            stratum: strata
                .get(&label.pair_id)
                .cloned()
                .ok_or_else(|| format!("{}: no plan history", label.pair_id))?,
            prompt_tokens: prompt.len(),
            asked_token,
            other_token,
            same_first_token: asked_token == other_token,
            carried_asked_first_line,
            carried_other_first_line,
            minus_c_asked_first_line,
            minus_c_other_first_line,
            chosen_token_changed: carried_probe.chosen_token != minus_probe.chosen_token,
            asked_rank_changed: carried_probe.asked.rank != minus_probe.asked.rank,
            other_rank_changed: carried_probe.other.rank != minus_probe.other.rank,
            chosen_or_rank_changed: carried_probe.chosen_token != minus_probe.chosen_token
                || carried_probe.asked.rank != minus_probe.asked.rank
                || carried_probe.other.rank != minus_probe.other.rank,
            query_changed_coordinates,
            query_l1: query_l1(&carried_probe.query_q12, &minus_probe.query_q12)?,
            type_head_kept_changed,
            asked_membership: membership(&prompt, asked_token, *reach),
            other_membership: membership(&prompt, other_token, *reach),
            carry_terms: carry_terms(
                &prompt,
                carried_forward.shared_table(),
                &ask.table,
                *reach,
                gains,
                width,
            ),
            carried_trace,
            minus_c_trace: minus_trace,
            carried: carried_probe,
            minus_c: minus_probe,
        });
    }

    let success = |case: &CaseReading| case.carried_asked_first_line;
    let failures = cases.iter().filter(|case| !success(case)).count();
    let successes = cases.len() - failures;
    let minus_c_asked_first_line = cases
        .iter()
        .filter(|case| case.minus_c_asked_first_line)
        .count();
    if successes != expected_carried_first_lines {
        return Err(format!(
            "carried first-line count {successes} differs from expected {expected_carried_first_lines}"
        ));
    }
    if minus_c_asked_first_line != expected_minus_c_first_lines {
        return Err(format!(
            "minus-C first-line count {minus_c_asked_first_line} differs from expected {expected_minus_c_first_lines}"
        ));
    }
    let prompt_contains_asked = cases
        .iter()
        .filter(|case| case.asked_membership.last_occurrence.is_some())
        .count();
    let prompt_contains_other = cases
        .iter()
        .filter(|case| case.other_membership.last_occurrence.is_some())
        .count();
    let mut failed_rank_delta = Vec::new();
    let mut success_rank_delta = Vec::new();
    for case in &cases {
        let delta = case.carried.asked.rank as i64 - case.minus_c.asked.rank as i64;
        if case.carried_asked_first_line {
            success_rank_delta.push(delta);
        } else {
            failed_rank_delta.push(delta);
        }
    }
    let sign_tally = |values: &[i64]| {
        json!({
            "rows": values.len(),
            "improved": values.iter().filter(|&&value| value < 0).count(),
            "unchanged": values.iter().filter(|&&value| value == 0).count(),
            "worse": values.iter().filter(|&&value| value > 0).count(),
            "sum": values.iter().sum::<i64>(),
        })
    };
    let asked_ranks = cases
        .iter()
        .map(|case| case.carried.asked.rank)
        .collect::<Vec<_>>();
    let minus_asked_ranks = cases
        .iter()
        .map(|case| case.minus_c.asked.rank)
        .collect::<Vec<_>>();
    let query_changed_delta = cases
        .iter()
        .filter(|case| case.query_changed_coordinates > 0)
        .map(|case| case.carried.asked.rank as i64 - case.minus_c.asked.rank as i64)
        .collect::<Vec<_>>();
    let query_unchanged_delta = cases
        .iter()
        .filter(|case| case.query_changed_coordinates == 0)
        .map(|case| case.carried.asked.rank as i64 - case.minus_c.asked.rank as i64)
        .collect::<Vec<_>>();
    let type_heads_changed = cases
        .iter()
        .map(|case| case.type_head_kept_changed.len())
        .sum::<usize>();
    let report = json!({
        "schema": TRACE_SCHEMA,
        "verdict": "instrument-first-reply-trace",
        "claim": "none; explains this configuration only and moves no bar",
        "command": invocation()?,
        "inputs": {
            "carried": carried_path,
            "carried_sha256": carried_sha256,
            "carried_report": carried_report_path,
            "carried_report_sha256": sha256_file(carried_report_path)?,
            "minus_c": minus_c_path,
            "minus_c_sha256": minus_c_sha256,
            "minus_c_report": minus_c_report_path,
            "minus_c_report_sha256": sha256_file(minus_c_report_path)?,
            "carried_rank": carried_rank_path,
            "carried_rank_sha256": sha256_file(carried_rank_path)?,
            "minus_c_rank": minus_c_rank_path,
            "minus_c_rank_sha256": sha256_file(minus_c_rank_path)?,
            "inputs_sha256": sha256_file(inputs_path)?,
            "labels_sha256": sha256_file(labels_path)?,
            "plan_sha256": sha256_file(plan_path)?,
            "panel_manifest": panel_manifest_path,
            "panel_manifest_sha256": panel_manifest_sha256,
            "tokenizer": tokenizer_path,
            "tokenizer_sha256": tokenizer_sha256,
        },
        "artifact": {
            "ask_table_entries": ask.table.len(),
            "carry_rows": *reach,
            "carry_gains": gains.len(),
            "readout_depth": readout.depth,
            "vocabulary": model.shape().vocab,
            "width": width,
        },
        "checks": {
            "minus_c_reproduced_byte_equal": true,
            "carried_writer_table_equals_minus_c": true,
            "panel_hashes_verified": true,
            "official_first_rank_parity": cases.len(),
            "expected_carried_first_lines": expected_carried_first_lines,
            "expected_minus_c_first_lines": expected_minus_c_first_lines,
            "official_first_line_counts_match": true,
            "rows": cases.len(),
        },
        "totals": {
            "carried_asked_top1": cases.iter().filter(|case| case.carried.asked_top1).count(),
            "carried_asked_not_top1": cases.iter().filter(|case| !case.carried.asked_top1).count(),
            "carried_asked_first_line": successes,
            "carried_asked_not_first_line": failures,
            "carried_other_first_line": cases.iter().filter(|case| case.carried_other_first_line).count(),
            "minus_c_asked_first_line": minus_c_asked_first_line,
            "minus_c_other_first_line": cases.iter().filter(|case| case.minus_c_other_first_line).count(),
            "minus_c_asked_top1": cases.iter().filter(|case| case.minus_c.asked_top1).count(),
            "carried_other_top1": cases.iter().filter(|case| case.carried.other_top1).count(),
            "minus_c_other_top1": cases.iter().filter(|case| case.minus_c.other_top1).count(),
            "chosen_token_changed": cases.iter().filter(|case| case.chosen_token_changed).count(),
            "asked_rank_changed": cases.iter().filter(|case| case.asked_rank_changed).count(),
            "other_rank_changed": cases.iter().filter(|case| case.other_rank_changed).count(),
            "chosen_or_rank_changed": cases.iter().filter(|case| case.chosen_or_rank_changed).count(),
            "query_changed": cases.iter().filter(|case| case.query_changed_coordinates > 0).count(),
            "type_head_kept_changed": cases.iter().filter(|case| !case.type_head_kept_changed.is_empty()).count(),
            "type_heads_changed": type_heads_changed,
            "carried_beats_other": cases.iter().filter(|case| !case.same_first_token && case.carried.asked.rank < case.carried.other.rank).count(),
            "minus_c_beats_other": cases.iter().filter(|case| !case.same_first_token && case.minus_c.asked.rank < case.minus_c.other.rank).count(),
            "asked_token_in_prompt": prompt_contains_asked,
            "other_token_in_prompt": prompt_contains_other,
            "asked_token_inside_carry": cases.iter().filter(|case| case.asked_membership.inside_carry).count(),
            "other_token_inside_carry": cases.iter().filter(|case| case.other_membership.inside_carry).count(),
        },
        "asked_rank_delta_carried_minus_minus_c": {
            "successes": sign_tally(&success_rank_delta),
            "failures": sign_tally(&failed_rank_delta),
            "query_changed": sign_tally(&query_changed_delta),
            "query_unchanged": sign_tally(&query_unchanged_delta),
        },
        "asked_rank": {
            "carried": distance_stats(asked_ranks.into_iter()),
            "minus_c": distance_stats(minus_asked_ranks.into_iter()),
        },
        "controls": {
            "exact_minus_c": "the cut reproduces byte for byte and replaces the carried query",
            "no_reorder": "case order, token order and head order are the panel's",
            "interpretation": "a positive carry result requires a first-token or asked-rank movement not reproduced by minus-C",
        },
        "cases": cases,
    });
    write_new(&out, &report)
}

fn require_schema(value: &Value, schema: &str, name: &str) -> Result<()> {
    if value["schema"].as_str() != Some(schema) {
        return Err(format!("{name} is not a {schema} receipt"));
    }
    Ok(())
}

fn repeat_identity(primary: &str, repeat: Option<&String>, name: &str) -> Result<Value> {
    let primary_sha256 = sha256_file(primary)?;
    match repeat {
        Some(repeat) => {
            let repeat_sha256 = sha256_file(repeat)?;
            if primary_sha256 != repeat_sha256 {
                return Err(format!(
                    "{name} repeat {repeat_sha256} differs from {primary_sha256}"
                ));
            }
            Ok(json!({
                "receipt_sha256": primary_sha256,
                "repeat_sha256": repeat_sha256,
                "byte_identical": true,
            }))
        }
        None => Ok(json!({
            "receipt_sha256": primary_sha256,
            "repeat_sha256": Value::Null,
            "byte_identical": Value::Null,
        })),
    }
}

fn compact_trace(value: &Value) -> Result<Value> {
    require_schema(value, TRACE_SCHEMA, "first-reply receipt")?;
    let successes = value["cases"]
        .as_array()
        .ok_or("first-reply receipt has no cases")?
        .iter()
        .filter(|case| case["carried_asked_first_line"].as_bool() == Some(true))
        .map(|case| {
            json!({
                "id": case["id"],
                "pair_id": case["pair_id"],
                "stratum": case["stratum"],
                "asked_token": case["asked_token"],
                "carried_asked_rank": case["carried"]["asked"]["rank"],
                "minus_c_asked_rank": case["minus_c"]["asked"]["rank"],
                "query_changed_coordinates": case["query_changed_coordinates"],
                "type_head_kept_changed": case["type_head_kept_changed"],
                "asked_membership": case["asked_membership"],
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "command": value["command"],
        "inputs": value["inputs"],
        "artifact": value["artifact"],
        "checks": value["checks"],
        "totals": value["totals"],
        "asked_rank": value["asked_rank"],
        "asked_rank_delta_carried_minus_minus_c": value["asked_rank_delta_carried_minus_minus_c"],
        "controls": value["controls"],
        "first_line_successes": successes,
    }))
}

fn compact_census(value: &Value) -> Result<Value> {
    require_schema(value, CENSUS_SCHEMA, "TRAIN census receipt")?;
    let top_tokens = value["first_answer_tokens"]
        .as_object()
        .ok_or("TRAIN census has no first-answer token map")?
        .iter()
        .map(|(token, count)| (token.clone(), count.as_u64().unwrap_or(0)))
        .collect::<Vec<_>>();
    let mut top_tokens = top_tokens;
    top_tokens.sort_by_key(|(token, count)| (std::cmp::Reverse(*count), token.clone()));
    Ok(json!({
        "command": value["command"],
        "inputs": value["inputs"],
        "method": value["method"],
        "carry_rows": value["carry_rows"],
        "questions": value["questions"],
        "exclusions": value["exclusions"],
        "first_scored_input": value["first_scored_input"],
        "first_answer_target": value["first_answer_target"],
        "question_start_to_scored_input": value["question_start_to_scored_input"],
        "heuristic_repeated_context_span_row": value["heuristic_repeated_context_span_row"],
        "heuristic_repeated_span_lengths": value["heuristic_repeated_span_lengths"],
        "heuristic_repeated_span_examples": value["heuristic_repeated_span_examples"],
        "components": value["components"],
        "sources": value["sources"],
        "controls": value["controls"],
        "first_answer_tokens_top_15": top_tokens.into_iter().take(15).collect::<Vec<_>>(),
    }))
}

fn receipt(flags: &BTreeMap<String, String>) -> Result<()> {
    let census_path = flag(flags, "train-census")?;
    let s1_path = flag(flags, "s1")?;
    let s2_path = flag(flags, "s2")?;
    let out = PathBuf::from(flag(flags, "out")?);
    let census = parse_json(census_path, "TRAIN census")?;
    let s1 = parse_json(s1_path, "seed-1 first-reply trace")?;
    let s2 = parse_json(s2_path, "seed-2 first-reply trace")?;
    let report = json!({
        "schema": RECEIPT_SCHEMA,
        "verdict": "instrument-diagnosis",
        "claim": "none; explains the registered configuration only and moves no bar",
        "command": invocation()?,
        "software": {
            "package": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "carry_geometry_source_sha256": sha256(include_bytes!("carry_geometry.rs")),
            "cargo_lock_sha256": sha256(include_bytes!("../Cargo.lock")),
            "executable_sha256": executable_sha256()?,
        },
        "repeats": {
            "train_census": repeat_identity(census_path, flags.get("train-census-repeat"), "TRAIN census")?,
            "seed_1": repeat_identity(s1_path, flags.get("s1-repeat"), "seed-1 trace")?,
            "seed_2": repeat_identity(s2_path, flags.get("s2-repeat"), "seed-2 trace")?,
        },
        "train_census": compact_census(&census)?,
        "first_reply": {
            "seed_1": compact_trace(&s1)?,
            "seed_2": compact_trace(&s2)?,
        },
    });
    write_new(&out, &report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question_pair(
        document: usize,
        question_start: usize,
        question_end: usize,
        scored_input: usize,
    ) -> QuestionPair {
        QuestionPair {
            document,
            source: "test".to_owned(),
            component: "test".to_owned(),
            question_text: "question?".to_owned(),
            reply_text: "answer".to_owned(),
            question_start,
            question_end,
            answer_target: scored_input + 1,
            scored_input,
            first_answer_token: 2,
            first_answer_piece: " answer".to_owned(),
            value: None,
        }
    }

    #[test]
    fn distance_stats_are_inclusive_at_the_carry_boundary() {
        let stats = distance_stats([0, 1, 32, 33, 64, 192].into_iter());
        assert_eq!(stats.rows, 6);
        assert_eq!(stats.within_32, 3);
        assert_eq!(stats.within_64, 5);
        assert_eq!(stats.within_192, 5);
        assert_eq!(stats.min, Some(0));
        assert_eq!(stats.max, Some(192));
    }

    #[test]
    fn value_inference_prefers_a_context_span_absent_from_the_question() {
        let context = [(7, 10), (8, 11), (9, 12), (10, 20), (11, 21)];
        let question = [30, 31, 32, 33];
        let reply = [30, 12, 20, 21];
        let value = infer_value(&context, &question, &reply).unwrap();
        assert_eq!(value.row, 1);
        assert_eq!(value.tokens, vec![12, 20, 21]);
    }

    #[test]
    fn a_question_span_is_not_mistaken_for_the_value() {
        let context = [
            (7, 30),
            (8, 31),
            (9, 32),
            (10, u32::MAX),
            (11, 40),
            (12, 41),
        ];
        let question = [30, 31, 32];
        let reply = [30, 31, 32, 40, 41];
        let value = infer_value(&context, &question, &reply).unwrap();
        assert_eq!(value.row, 3);
        assert_eq!(value.tokens, vec![40, 41]);
    }

    #[test]
    fn output_is_refused_if_it_already_exists() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-carry-geometry-existing-{}",
            std::process::id()
        ));
        fs::write(&path, b"occupied").unwrap();
        let error = write_new(&path, &json!({"ok": true})).unwrap_err();
        assert!(error.contains("File exists"), "{error}");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn first_line_match_uses_the_accepted_literal_without_requiring_its_period() {
        let answers = FrozenAnswers {
            accepted: vec!["basalt loggia.".to_owned()],
            intent: "current".to_owned(),
        };
        assert!(first_line_matches(" basalt loggia is the job.", &answers));
        assert!(!first_line_matches(
            " The job is unknown.\nbasalt loggia.",
            &answers
        ));
        let number = FrozenAnswers {
            accepted: vec!["71.".to_owned()],
            intent: "current".to_owned(),
        };
        assert!(first_line_matches(" It is 71.", &number));
        assert!(!first_line_matches(" It is 710.", &number));
    }

    #[test]
    fn controls_pair_only_prior_questions_and_never_treat_future_rows_as_reached() {
        let pairs = [
            question_pair(0, 5, 10, 20),
            question_pair(0, 30, 35, 40),
            question_pair(1, 100, 105, 115),
        ];
        let controls = census_controls(&pairs, 32);
        assert_eq!(
            controls["previous_question_answer_pairs_in_same_document"]["rows"],
            1
        );
        assert_eq!(
            controls["previous_question_answer_pairs_in_same_document"]["within_carry"],
            1
        );
        assert_eq!(
            controls["previous_question_answer_pairs_in_same_document"]
                ["excluded_single_question_documents"],
            1
        );
        assert_eq!(controls["question_end_offset_32_earlier"]["rows"], 2);
        assert_eq!(
            controls["question_end_offset_32_earlier"]["clamped_to_document_start"],
            1
        );
    }

    #[test]
    fn receipt_repeat_mismatch_is_refused() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-carry-geometry-repeat-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let first = root.join("first.json");
        let second = root.join("second.json");
        fs::write(&first, b"one").unwrap();
        fs::write(&second, b"two").unwrap();
        let error = repeat_identity(
            first.to_str().unwrap(),
            Some(&second.to_str().unwrap().to_owned()),
            "test",
        )
        .unwrap_err();
        assert!(error.contains("differs"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "needs the pinned tokenizer and official panel inputs"]
    fn official_render_matches_every_report_prompt_length() {
        let tokenizer_path =
            std::env::var("UOR_R4_PINNED_TOKENIZER").expect("UOR_R4_PINNED_TOKENIZER");
        let panel = std::env::var("UOR_R4_CARRY_PANEL").expect("UOR_R4_CARRY_PANEL");
        let report_path = std::env::var("UOR_R4_CARRY_REPORT").expect("UOR_R4_CARRY_REPORT");
        let (tokenizer, _) = tokenizer(&tokenizer_path).unwrap();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let inputs: Inputs =
            serde_json::from_slice(&fs::read(Path::new(&panel).join("inputs.json")).unwrap())
                .unwrap();
        let report = parse_json(&report_path, "report").unwrap();
        let rows = report["rows"].as_array().unwrap();
        let expected = rows
            .iter()
            .map(|row| {
                (
                    row["id"].as_str().unwrap(),
                    row["prompt_tokens"].as_u64().unwrap() as usize,
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(inputs.cases.len(), expected.len());
        for case in &inputs.cases {
            let prompt = render_case(&tokenizer, &encoder, case).unwrap();
            assert_eq!(
                prompt.len(),
                expected[case.id.as_str()],
                "{} prompt length",
                case.id
            );
        }
    }
}
