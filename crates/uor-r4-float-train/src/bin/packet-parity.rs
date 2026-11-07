//! Compare float-checkpoint and served-integer decisions on the same rendered
//! `m2-fresh256` packet prompts.
//!
//! This reads one checkpoint and one `UORLUT01` artifact made from it. It
//! teacher-forces the same shortest accepted answer plus EOS through both
//! readers and reports every chosen-token, target-rank and EOS-rank
//! disagreement. It is a GPU instrument; the binary's `cuda-parity` feature
//! carries the float trainer.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use candle_core::Device;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_float_train::in_space_long::{FloatTokenRank, PacketCheckpoint};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.packet-parity/1";
const ONE_USER: &str = "one-user-turn/v1";
const PER_TURN: &str = "per-turn/v1";
const EOS: u32 = 1;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("packet-parity: {error}");
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
    #[allow(dead_code)]
    pair_id: String,
}

#[derive(Deserialize)]
struct Manifest {
    files: BTreeMap<String, ManifestFile>,
    tokenizer_sha256: String,
}

#[derive(Deserialize)]
struct ManifestFile {
    sha256: String,
}

#[derive(Debug, Serialize)]
struct RowParity {
    id: String,
    prompt_tokens: usize,
    reference_targets: usize,
    eos_target_index: usize,
    chosen_token_mismatches: usize,
    rank_mismatches: usize,
    float_top1: usize,
    integer_top1: usize,
    float_eos_rank: usize,
    integer_eos_rank: usize,
    max_rank_delta: usize,
}

#[derive(Debug)]
struct IntegerTokenRank {
    target_token: u32,
    chosen_token: u32,
    rank: usize,
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
    let rendering = args
        .get("rendering")
        .map(String::as_str)
        .unwrap_or(PER_TURN);
    if rendering != ONE_USER && rendering != PER_TURN {
        return Err("--rendering must be one-user-turn/v1 or per-turn/v1".to_owned());
    }
    let read = |name: &str| -> Result<(Vec<u8>, String)> {
        let path = PathBuf::from(flag(name)?);
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let sha = sha256(&bytes);
        Ok((bytes, sha))
    };
    let (tokenizer_bytes, tokenizer_sha) = read("tokenizer")?;
    let (inputs_bytes, inputs_sha) = read("inputs")?;
    let (labels_bytes, labels_sha) = read("labels")?;
    let (_, plan_sha) = read("plan")?;
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

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "tokenizer is not a byte-level BPE tokenizer".to_owned())?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let inputs: Inputs = parse_json("inputs", &inputs_bytes)?;
    let labels: Labels = parse_json("labels", &labels_bytes)?;
    if labels.protocol != SCHEMA_V2 {
        return Err(format!(
            "labels protocol {} is not {SCHEMA_V2}",
            labels.protocol
        ));
    }
    let by_id: BTreeMap<&str, &InputCase> = inputs
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect();
    if by_id.len() != inputs.cases.len() {
        return Err("inputs repeat a case id".to_owned());
    }
    let referenced: BTreeSet<&str> = labels.cases.iter().map(|case| case.id.as_str()).collect();
    if referenced.len() != labels.cases.len() {
        return Err("labels repeat a case id".to_owned());
    }
    if referenced.len() != by_id.len() {
        return Err("inputs and labels name different case sets".to_owned());
    }

    let checkpoint = PathBuf::from(flag("checkpoint")?);
    let file_sha = |name: &str| -> Result<String> {
        let path = checkpoint.join(name);
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(sha256(&bytes))
    };
    let mut checkpoint_hashes = serde_json::json!({
        "model.safetensors": file_sha("model.safetensors")?,
        "config.json": file_sha("config.json")?,
    });
    if checkpoint.join("hamming_read.safetensors").is_file() {
        checkpoint_hashes["hamming_read.safetensors"] =
            serde_json::Value::String(file_sha("hamming_read.safetensors")?);
    }
    let run_config = find_run_config(&checkpoint)?;
    let run_config_bytes =
        fs::read(&run_config).map_err(|error| format!("{}: {error}", run_config.display()))?;
    let run_config_sha = sha256(&run_config_bytes);
    let artifact = fs::read(flag("artifact")?).map_err(|error| format!("artifact: {error}"))?;
    let artifact_sha = sha256(&artifact);
    let device = Device::new_cuda(0).map_err(|error| format!("CUDA device: {error}"))?;
    let float = PacketCheckpoint::load(&checkpoint, &device).map_err(|e| e.to_string())?;

    let mut rows = Vec::with_capacity(labels.cases.len());
    let mut totals = Totals::default();
    for label in &labels.cases {
        let case = by_id
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no input case", label.id))?;
        let (_, prompt) = render(&tokenizer, &encoder, case, rendering)?;
        let (_, reference) = reference_ids(&encoder, &label.answers)?;
        let mut integer = CpuFbsForward::from_artifact(&artifact)
            .map_err(|error| format!("{}: {error}", label.id))?;
        let integer_ranks = teacher_forced_nearest_ranks(&mut integer, &prompt, &reference)
            .map_err(|error| format!("{}: {error}", label.id))?;
        let float_ranks = float
            .teacher_forced_ranks(&prompt, &reference)
            .map_err(|error| format!("{}: {error}", label.id))?;
        let row = compare_row(&label.id, &prompt, &reference, &float_ranks, &integer_ranks)?;
        totals.add(&row);
        rows.push(row);
    }

    let report = serde_json::json!({
        "schema": SCHEMA,
        "inputs": {
            "checkpoint_files": checkpoint_hashes,
            "source_run_config_sha256": run_config_sha,
            "artifact_sha256": artifact_sha,
            "tokenizer_sha256": tokenizer_sha,
            "inputs_sha256": inputs_sha,
            "labels_sha256": labels_sha,
            "plan_sha256": plan_sha,
            "manifest_sha256": manifest_sha,
        },
        "settings": {
            "rendering": rendering,
            "comparison": "teacher-forced nearest decisions",
            "tie_break": "larger score, then lower token id",
            "eos_id": EOS,
        },
        "totals": totals,
        "rows": rows,
    });
    let mut text = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    text.push(b'\n');
    write_new(&out, &text)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "totals": report["totals"],
            "out": out,
            "sha256": sha256(&text),
        })
    );
    Ok(())
}

#[derive(Default, Serialize)]
struct Totals {
    rows: usize,
    targets: usize,
    chosen_token_mismatches: usize,
    rank_mismatches: usize,
    float_top1: usize,
    integer_top1: usize,
    eos_rows: usize,
    eos_rank_mismatches: usize,
    max_rank_delta: usize,
}

impl Totals {
    fn add(&mut self, row: &RowParity) {
        self.rows += 1;
        self.targets += row.reference_targets;
        self.chosen_token_mismatches += row.chosen_token_mismatches;
        self.rank_mismatches += row.rank_mismatches;
        self.float_top1 += row.float_top1;
        self.integer_top1 += row.integer_top1;
        self.eos_rows += 1;
        self.eos_rank_mismatches += usize::from(row.float_eos_rank != row.integer_eos_rank);
        self.max_rank_delta = self.max_rank_delta.max(row.max_rank_delta);
    }
}

fn compare_row(
    id: &str,
    prompt: &[u32],
    reference: &[u32],
    float: &[FloatTokenRank],
    integer: &[IntegerTokenRank],
) -> Result<RowParity> {
    if float.len() != reference.len() || integer.len() != reference.len() {
        return Err(format!("{id}: rank counts differ from reference targets"));
    }
    let mut chosen_token_mismatches = 0;
    let mut rank_mismatches = 0;
    let mut float_top1 = 0;
    let mut integer_top1 = 0;
    let mut max_rank_delta = 0;
    let mut float_eos_rank = None;
    let mut integer_eos_rank = None;
    for ((target, f), i) in reference.iter().zip(float).zip(integer) {
        if f.target_token != *target || i.target_token != *target {
            return Err(format!("{id}: target misalignment"));
        }
        chosen_token_mismatches += usize::from(f.chosen_token != i.chosen_token);
        rank_mismatches += usize::from(f.rank != i.rank);
        float_top1 += usize::from(f.rank == 1);
        integer_top1 += usize::from(i.rank == 1);
        max_rank_delta = max_rank_delta.max(f.rank.abs_diff(i.rank));
        if *target == EOS {
            float_eos_rank = Some(f.rank);
            integer_eos_rank = Some(i.rank);
        }
    }
    Ok(RowParity {
        id: id.to_owned(),
        prompt_tokens: prompt.len(),
        reference_targets: reference.len(),
        eos_target_index: reference
            .iter()
            .rposition(|&token| token == EOS)
            .ok_or_else(|| format!("{id}: reference has no EOS"))?,
        chosen_token_mismatches,
        rank_mismatches,
        float_top1,
        integer_top1,
        float_eos_rank: float_eos_rank.ok_or_else(|| format!("{id}: no float EOS rank"))?,
        integer_eos_rank: integer_eos_rank.ok_or_else(|| format!("{id}: no integer EOS rank"))?,
        max_rank_delta,
    })
}

fn nearest_rank(
    query: &[i32],
    table: &[i16],
    width: usize,
    target_token: u32,
    served_token: u32,
) -> Result<IntegerTokenRank> {
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
    Ok(IntegerTokenRank {
        target_token,
        chosen_token: served_token,
        rank,
    })
}

fn teacher_forced_nearest_ranks(
    forward: &mut CpuFbsForward,
    prompt: &[u32],
    targets: &[u32],
) -> Result<Vec<IntegerTokenRank>> {
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
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(rows)
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
                .map_err(|error| error.to_string())?,
        );
    }
    step.ok_or_else(|| "empty prompt".to_owned())
}

fn render(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    case: &InputCase,
    rendering: &str,
) -> Result<(String, Vec<u32>)> {
    let mut contexts: Vec<&Segment> = case
        .segments
        .iter()
        .filter(|segment| segment.kind == "Context")
        .collect();
    contexts.sort_by_key(|segment| segment.event);
    let mut parts: Vec<String> = contexts
        .iter()
        .map(|segment| tokenizer.decode(&segment.token_ids).trim().to_owned())
        .collect();
    parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
    let (content, ids) = match rendering {
        ONE_USER => {
            let content = parts.join(" ");
            let ids = encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &content,
                }])
                .tokens;
            (content, ids)
        }
        PER_TURN => {
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

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 9] = [
        "checkpoint",
        "artifact",
        "tokenizer",
        "inputs",
        "labels",
        "plan",
        "manifest",
        "out",
        "rendering",
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

fn find_run_config(checkpoint: &Path) -> Result<PathBuf> {
    let checkpoint = checkpoint
        .canonicalize()
        .map_err(|error| format!("--checkpoint: {error}"))?;
    let mut candidates = vec![checkpoint.join("config.json")];
    if let Some(run) = checkpoint.parent().and_then(Path::parent) {
        candidates.push(run.join("config.json"));
    }
    for path in candidates {
        if !path.is_file() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(
            &fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", path.display()))?;
        if value.get("steps").is_some() {
            return Ok(path);
        }
    }
    Err("--checkpoint requires its source run config beside it or two levels above it".to_owned())
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

    fn float(target: u32, chosen: u32, rank: usize) -> FloatTokenRank {
        FloatTokenRank {
            target_token: target,
            chosen_token: chosen,
            rank,
            candidate_count: 4096,
            target_score: 0.0,
            chosen_score: 1.0,
        }
    }

    fn integer(target: u32, chosen: u32, rank: usize) -> IntegerTokenRank {
        IntegerTokenRank {
            target_token: target,
            rank,
            chosen_token: chosen,
        }
    }

    #[test]
    fn parity_counts_use_exact_served_ranks_and_eos() {
        let row = compare_row(
            "hand",
            &[0, 7],
            &[4, EOS],
            &[float(4, 4, 1), float(EOS, 4, 5)],
            &[integer(4, 4, 1), integer(EOS, 4, 6)],
        )
        .unwrap();
        assert_eq!(row.chosen_token_mismatches, 0);
        assert_eq!(row.rank_mismatches, 1);
        assert_eq!(row.float_top1, 1);
        assert_eq!(row.integer_top1, 1);
        assert_eq!(row.float_eos_rank, 5);
        assert_eq!(row.integer_eos_rank, 6);
        assert_eq!(row.max_rank_delta, 1);
    }

    #[test]
    fn target_misalignment_fails_closed() {
        let error =
            compare_row("bad", &[0], &[4], &[float(4, 4, 1)], &[integer(5, 4, 1)]).unwrap_err();
        assert!(error.contains("target misalignment"));
    }
}
