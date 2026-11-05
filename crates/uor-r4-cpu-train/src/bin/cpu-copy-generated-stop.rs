use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use uor_r4_cpu_train::{
    fit_stop_signature_gate, normalized_reference, pack_stop_signature,
    shuffle_stop_signature_rows, stop_signature_fires, validate_corpus_manifest, CpuFbsForward,
    CpuFbsHead, CpuMaskedCorpus, StopSignature, StopSignatureGate, StopSignatureRow,
    StopSignatureTraining,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
struct Decision {
    generated_step: usize,
    signature: StopSignature,
}

#[derive(Clone, Debug)]
struct Trajectory {
    row: usize,
    selected: Vec<u32>,
    reference: Vec<u32>,
    decisions: Vec<Decision>,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
struct ReplyReading {
    replies: u64,
    exact: u64,
    premature: u64,
    at_reference: u64,
    no_fire: u64,
    generated_tokens: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
struct ReplyRow {
    row: usize,
    fired_step: Option<usize>,
    class: StopClass,
    exact: bool,
    premature: bool,
    at_reference: bool,
    selected_sha256: String,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum StopClass {
    NoFire,
    Premature,
    AtReference,
}

#[derive(Clone, Debug)]
struct PanelTrace {
    trajectories: Vec<Trajectory>,
    all_rows: Vec<StopSignatureRow>,
    exact_rows: Vec<StopSignatureRow>,
}

struct BarInput<'a> {
    replies: usize,
    oracle: u64,
    minimum_exact: u64,
    minimum_replies: usize,
    minimum_oracle: u64,
    training_power: bool,
    true_rows: &'a [ReplyRow],
    shuffled_rows: &'a [ReplyRow],
    all_reference_rows: Option<&'a [ReplyRow]>,
    no_gate_rows: &'a [ReplyRow],
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-copy-generated-stop: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut values = std::env::args().skip(1);
    let artifact_path = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_report = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let dev_report = PathBuf::from(values.next().ok_or_else(usage)?);
    let dev_tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let dev_mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let dev_manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut prototypes = 8usize;
    let mut seed = 157u64;
    let mut exact_body_terminal = false;
    let mut min_train_exact = 0u64;
    let mut min_dev_exact = 0u64;
    let mut min_dev_replies = 0usize;
    let mut min_dev_oracle = 0u64;
    let mut min_distinct_terminal = 0usize;
    while let Some(flag) = values.next() {
        if flag == "--exact-body" {
            exact_body_terminal = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))?;
        match flag.as_str() {
            "--prototypes" => {
                prototypes = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--seed" => {
                seed = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--min-train-exact" => {
                min_train_exact = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--min-dev-exact" => {
                min_dev_exact = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--min-dev-replies" => {
                min_dev_replies = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--min-dev-oracle" => {
                min_dev_oracle = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--min-distinct-terminal" => {
                min_distinct_terminal = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if !(1..=32).contains(&prototypes) {
        return Err("prototypes must be in 1..=32".to_owned());
    }

    let artifact = read(&artifact_path)?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    let train_report_bytes = read(&train_report)?;
    let dev_report_bytes = read(&dev_report)?;
    let train_report_sha256 = uor_r4_discrete::corpus::sha256(&train_report_bytes);
    let dev_report_sha256 = uor_r4_discrete::corpus::sha256(&dev_report_bytes);
    if CpuFbsForward::from_artifact(&artifact)
        .map_err(|error| error.to_string())?
        .context_length()
        < 2
    {
        return Err("artifact context is too short".to_owned());
    }
    let train = load_panel(
        &artifact,
        &artifact_sha256,
        &train_report_bytes,
        &train_tokens,
        &train_mask,
        &train_manifest,
    )?;
    let dev = load_panel(
        &artifact,
        &artifact_sha256,
        &dev_report_bytes,
        &dev_tokens,
        &dev_mask,
        &dev_manifest,
    )?;

    let training_rows = select_training_rows(&train, exact_body_terminal);
    let true_training = fit_stop_signature_gate(training_rows, prototypes)
        .map_err(|error| format!("true gate fit: {error}"))?;
    let shuffled_rows = shuffle_stop_signature_rows(training_rows, seed)
        .map_err(|error| format!("shuffled rows: {error}"))?;
    if shuffled_rows
        .iter()
        .zip(training_rows)
        .all(|(left, right)| left.terminal == right.terminal)
    {
        return Err("shuffled gate is vacuous".to_owned());
    }
    let shuffled_training = fit_stop_signature_gate(&shuffled_rows, prototypes)
        .map_err(|error| format!("shuffled gate fit: {error}"))?;
    let all_reference_training = exact_body_terminal
        .then(|| fit_stop_signature_gate(&train.all_rows, prototypes))
        .transpose()
        .map_err(|error| format!("all-reference gate fit: {error}"))?;

    let train_true = evaluate_gate(&train.trajectories, &true_training.gate)?;
    let train_shuffled = evaluate_gate(&train.trajectories, &shuffled_training.gate)?;
    let train_all_reference = all_reference_training
        .as_ref()
        .map(|training| evaluate_gate(&train.trajectories, &training.gate))
        .transpose()?;
    let train_none = evaluate_gate(&train.trajectories, &no_gate())?;
    let dev_true = evaluate_gate(&dev.trajectories, &true_training.gate)?;
    let dev_shuffled = evaluate_gate(&dev.trajectories, &shuffled_training.gate)?;
    let dev_all_reference = all_reference_training
        .as_ref()
        .map(|training| evaluate_gate(&dev.trajectories, &training.gate))
        .transpose()?;
    let dev_none = evaluate_gate(&dev.trajectories, &no_gate())?;

    let train_oracle = oracle_exact(&train.trajectories);
    let dev_oracle = oracle_exact(&dev.trajectories);
    let census = signature_census(training_rows);
    let distinct_terminal = census["distinct_terminal"]
        .as_u64()
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| "selected terminal census is not a usize".to_owned())?;
    let training_power = distinct_terminal >= min_distinct_terminal;
    let train_bar = panel_bar(BarInput {
        replies: train.trajectories.len(),
        oracle: train_oracle,
        minimum_exact: min_train_exact,
        minimum_replies: 0,
        minimum_oracle: 0,
        training_power,
        true_rows: &train_true.1,
        shuffled_rows: &train_shuffled.1,
        all_reference_rows: train_all_reference
            .as_ref()
            .map(|(_, rows)| rows.as_slice()),
        no_gate_rows: &train_none.1,
    })?;
    let dev_bar = panel_bar(BarInput {
        replies: dev.trajectories.len(),
        oracle: dev_oracle,
        minimum_exact: min_dev_exact,
        minimum_replies: min_dev_replies,
        minimum_oracle: min_dev_oracle,
        training_power,
        true_rows: &dev_true.1,
        shuffled_rows: &dev_shuffled.1,
        all_reference_rows: dev_all_reference.as_ref().map(|(_, rows)| rows.as_slice()),
        no_gate_rows: &dev_none.1,
    })?;

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-generated-stop/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded generated-trajectory copy stop surrogate; no serving or language claim",
        "inputs": {
            "artifact": artifact_path,
            "artifact_sha256": artifact_sha256,
            "train_report_sha256": train_report_sha256,
            "train_tokens": train_tokens,
            "train_mask": train_mask,
            "train_manifest": train_manifest,
            "dev_report_sha256": dev_report_sha256,
            "dev_tokens": dev_tokens,
            "dev_mask": dev_mask,
            "dev_manifest": dev_manifest,
        },
        "settings": {
            "prototypes": prototypes,
            "seed": seed,
            "terminal_label": if exact_body_terminal {
                "exact body + generated_step == normalized reference length"
            } else {
                "generated_step == normalized reference length"
            },
        },
        "training": {
            "true": gate_summary(&true_training),
            "shuffled": gate_summary(&shuffled_training),
            "all_reference": all_reference_training.as_ref().map(gate_summary),
            "selected_census": signature_census(training_rows),
            "requirements": {
                "minimum_exact_train_replies": min_train_exact,
                "minimum_exact_dev_replies": min_dev_exact,
                "minimum_dev_replies": min_dev_replies,
                "minimum_dev_oracle_exact": min_dev_oracle,
                "minimum_distinct_terminal_signatures": min_distinct_terminal,
            },
        },
        "train": {
            "oracle_exact": train_oracle,
            "true": reply_summary(&train_true),
            "shuffled": reply_summary(&train_shuffled),
            "all_reference": train_all_reference.as_ref().map(reply_summary),
            "no_gate": reply_summary(&train_none),
            "bar": train_bar,
        },
        "dev": {
            "oracle_exact": dev_oracle,
            "true": reply_summary(&dev_true),
            "shuffled": reply_summary(&dev_shuffled),
            "all_reference": dev_all_reference.as_ref().map(reply_summary),
            "no_gate": reply_summary(&dev_none),
            "bar": dev_bar,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn load_panel(
    artifact: &[u8],
    artifact_sha256: &str,
    report_bytes: &[u8],
    tokens_path: &Path,
    mask_path: &Path,
    manifest_path: &Path,
) -> Result<PanelTrace> {
    let report: Value =
        serde_json::from_slice(report_bytes).map_err(|error| format!("report: {error}"))?;
    if report.get("schema").and_then(Value::as_str) != Some("uor-r4.cpu-copy-key-reply/1") {
        return Err("source report schema differs".to_owned());
    }
    if report
        .pointer("/inputs/artifact_sha256")
        .and_then(Value::as_str)
        != Some(artifact_sha256)
    {
        return Err("source report artifact hash differs".to_owned());
    }
    let token_bytes = read(tokens_path)?;
    let mask_bytes = read(mask_path)?;
    let manifest_bytes = read(manifest_path)?;
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    if report
        .pointer("/inputs/tokens_sha256")
        .and_then(Value::as_str)
        != Some(identity.tokens_sha256.as_str())
        || report
            .pointer("/inputs/mask_sha256")
            .and_then(Value::as_str)
            != Some(identity.mask_sha256.as_str())
    {
        return Err("source report token or mask hash differs".to_owned());
    }
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask_bytes,
        identity.vocab,
        identity.max_tokens,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let samples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let arm = report
        .pointer("/arms/longest_4_to_1")
        .ok_or_else(|| "longest-first arm missing".to_owned())?;
    let arm_name = arm
        .pointer("/reading/arm")
        .and_then(Value::as_str)
        .ok_or_else(|| "arm name missing".to_owned())?;
    if arm_name != "longest4_to1" {
        return Err("longest-first arm name differs".to_owned());
    }
    let steps = arm
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| "arm steps missing".to_owned())?;
    let generated_tokens = arm
        .pointer("/reading/generated_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(|| "generated-token count missing".to_owned())?;
    if steps.len() as u64 != generated_tokens {
        return Err("step count differs from generated-token count".to_owned());
    }
    let source_rows = arm
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "row readings missing".to_owned())?;
    if source_rows.len() != samples.len() {
        return Err("row count differs from corpus examples".to_owned());
    }
    let mut by_row = vec![Vec::<(usize, u32)>::new(); samples.len()];
    for step in steps {
        let row = number(step, "row", true)?;
        let generated_step = number(step, "generated_step", true)?;
        let token = number(step, "selected_token", false)? as u32;
        by_row
            .get_mut(row)
            .ok_or_else(|| "step row outside samples".to_owned())?
            .push((generated_step, token));
    }

    let mut trajectories = Vec::with_capacity(samples.len());
    let mut all_rows = Vec::new();
    let mut exact_rows = Vec::new();
    for (row, sample) in samples.iter().enumerate() {
        let mut trace = std::mem::take(&mut by_row[row]);
        trace.sort_unstable_by_key(|(step, _)| *step);
        for (expected, (actual, _)) in trace.iter().enumerate() {
            if expected != *actual {
                return Err(format!("row {row} has a non-monotonic trace"));
            }
        }
        let selected = trace
            .into_iter()
            .map(|(_, token)| token)
            .collect::<Vec<_>>();
        let source_hash = source_rows[row]
            .get("reply_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| "row reply hash missing".to_owned())?;
        if token_hash(&selected) != source_hash {
            return Err(format!("row {row} full-trace hash differs"));
        }
        let reference = normalized_reference(&sample.reply).to_vec();
        let decisions = replay_decisions(
            artifact,
            &sample.prompt,
            &selected,
            &reference,
            row,
            &mut all_rows,
            &mut exact_rows,
        )?;
        trajectories.push(Trajectory {
            row,
            selected,
            reference,
            decisions,
        });
    }
    Ok(PanelTrace {
        trajectories,
        all_rows,
        exact_rows,
    })
}

fn replay_decisions(
    artifact: &[u8],
    prompt: &[u32],
    selected: &[u32],
    reference: &[u32],
    row: usize,
    all_rows: &mut Vec<StopSignatureRow>,
    exact_rows: &mut Vec<StopSignatureRow>,
) -> Result<Vec<Decision>> {
    let reference_len = reference.len();
    let body_exact = body_matches_reference(selected, reference);
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| format!("replay forward: {error}"))?;
    let context = forward.context_length();
    if prompt.is_empty() || prompt.len() > context || selected.len() < reference_len {
        return Err(format!("row {row} trace is outside the artifact context"));
    }
    let mut step = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let current = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| format!("row {row} prompt step: {error}"))?;
        if index + 1 == prompt.len() {
            step = Some(current);
        }
    }
    let mut step = step.ok_or_else(|| format!("row {row} prompt is empty"))?;
    let mut decisions = Vec::with_capacity(reference_len + 1);
    for generated_step in 0..=reference_len {
        let terminal = generated_step == reference_len;
        let signature = pack_stop_signature(&step.query_q12)
            .map_err(|error| format!("row {row} signature: {error}"))?;
        decisions.push(Decision {
            generated_step,
            signature,
        });
        let row_reading = StopSignatureRow {
            signature,
            terminal,
            writer_token: step.token,
        };
        all_rows.push(row_reading.clone());
        if body_exact {
            exact_rows.push(row_reading);
        }
        if terminal {
            break;
        }
        let next_token = replay_token(selected, generated_step, reference_len, row)?
            .ok_or_else(|| format!("row {row} missing nonterminal token"))?;
        step = forward
            .step(next_token, CpuFbsHead::Writer)
            .map_err(|error| format!("row {row} generated step: {error}"))?;
    }
    Ok(decisions)
}

fn replay_token(
    selected: &[u32],
    generated_step: usize,
    reference_len: usize,
    row: usize,
) -> Result<Option<u32>> {
    if generated_step == reference_len {
        return Ok(None);
    }
    selected
        .get(generated_step)
        .copied()
        .map(Some)
        .ok_or_else(|| format!("row {row} missing generated token {generated_step}"))
}

fn evaluate_gate(
    trajectories: &[Trajectory],
    gate: &StopSignatureGate,
) -> Result<(ReplyReading, Vec<ReplyRow>)> {
    let mut reading = ReplyReading::default();
    let mut rows = Vec::with_capacity(trajectories.len());
    for trajectory in trajectories {
        let mut fired = None;
        for decision in &trajectory.decisions {
            if stop_signature_fires(gate, &decision.signature).map_err(|error| error.to_string())? {
                fired = Some(decision.generated_step);
                break;
            }
        }
        let exact = fired == Some(trajectory.reference.len())
            && trajectory.selected[..trajectory.reference.len()] == trajectory.reference;
        let premature = fired.is_some_and(|step| step < trajectory.reference.len());
        let at_reference = fired == Some(trajectory.reference.len());
        reading.replies += 1;
        reading.exact += u64::from(exact);
        reading.premature += u64::from(premature);
        reading.at_reference += u64::from(at_reference);
        reading.no_fire += u64::from(fired.is_none());
        reading.generated_tokens += fired.unwrap_or(trajectory.selected.len()) as u64;
        rows.push(ReplyRow {
            row: trajectory.row,
            fired_step: fired,
            class: if premature {
                StopClass::Premature
            } else if at_reference {
                StopClass::AtReference
            } else {
                StopClass::NoFire
            },
            exact,
            premature,
            at_reference,
            selected_sha256: token_hash(&trajectory.selected),
        });
    }
    Ok((reading, rows))
}

fn oracle_exact(trajectories: &[Trajectory]) -> u64 {
    trajectories
        .iter()
        .filter(|trajectory| body_matches_reference(&trajectory.selected, &trajectory.reference))
        .count() as u64
}

fn body_matches_reference(selected: &[u32], reference: &[u32]) -> bool {
    selected.len() >= reference.len() && &selected[..reference.len()] == reference
}

fn select_training_rows(panel: &PanelTrace, exact_body: bool) -> &[StopSignatureRow] {
    if exact_body {
        &panel.exact_rows
    } else {
        &panel.all_rows
    }
}

fn panel_bar(input: BarInput<'_>) -> Result<Value> {
    let true_reading = summarize_rows(input.true_rows);
    let shuffled_reading = summarize_rows(input.shuffled_rows);
    let all_reference_reading = input.all_reference_rows.map(summarize_rows);
    let no_gate_reading = summarize_rows(input.no_gate_rows);
    let floor = input.oracle.div_ceil(2).max(input.minimum_exact);
    let false_stop_limit = input.replies as u64 / 100;
    let shuffled = compare_rows(input.true_rows, input.shuffled_rows)?;
    let all_reference = input
        .all_reference_rows
        .map(|rows| compare_rows(input.true_rows, rows))
        .transpose()?;
    let no_gate = compare_rows(input.true_rows, input.no_gate_rows)?;
    let all_reference_passes = all_reference
        .as_ref()
        .is_none_or(|comparison| comparison["passes"].as_bool() == Some(true));
    Ok(serde_json::json!({
        "oracle_floor": floor,
        "minimum_exact": input.minimum_exact,
        "minimum_replies": input.minimum_replies,
        "minimum_oracle": input.minimum_oracle,
        "training_power": input.training_power,
        "reply_power": input.replies >= input.minimum_replies,
        "oracle_power": input.oracle >= input.minimum_oracle,
        "true_exact": true_reading.exact,
        "true_premature": true_reading.premature,
        "false_stop_limit": false_stop_limit,
        "shuffled_exact": shuffled_reading.exact,
        "all_reference_exact": all_reference_reading.map(|reading| reading.exact),
        "no_gate_exact": no_gate_reading.exact,
        "true_vs_shuffled": shuffled,
        "true_vs_all_reference": all_reference,
        "true_vs_no_gate": no_gate,
        "passes": input.training_power
            && input.replies >= input.minimum_replies
            && input.oracle >= input.minimum_oracle
            && true_reading.exact >= floor
            && true_reading.premature <= false_stop_limit
            && shuffled["passes"].as_bool() == Some(true)
            && all_reference_passes
            && no_gate["passes"].as_bool() == Some(true),
    }))
}

fn compare_rows(candidate: &[ReplyRow], control: &[ReplyRow]) -> Result<Value> {
    if candidate.len() != control.len() {
        return Err("paired stop rows differ in length".to_owned());
    }
    let (mut candidate_only, mut control_only, mut candidate_sum, mut control_sum) =
        (0u64, 0u64, 0u64, 0u64);
    for (candidate, control) in candidate.iter().zip(control) {
        if candidate.row != control.row {
            return Err("paired stop row identity differs".to_owned());
        }
        candidate_sum += u64::from(candidate.exact);
        control_sum += u64::from(control.exact);
        candidate_only += u64::from(candidate.exact && !control.exact);
        control_only += u64::from(control.exact && !candidate.exact);
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    }))
}

fn summarize_rows(rows: &[ReplyRow]) -> ReplyReading {
    ReplyReading {
        replies: rows.len() as u64,
        exact: rows.iter().filter(|row| row.exact).count() as u64,
        premature: rows.iter().filter(|row| row.premature).count() as u64,
        at_reference: rows.iter().filter(|row| row.at_reference).count() as u64,
        no_fire: rows
            .iter()
            .filter(|row| !row.at_reference && !row.premature)
            .count() as u64,
        generated_tokens: 0,
    }
}

fn reply_summary((reading, rows): &(ReplyReading, Vec<ReplyRow>)) -> Value {
    serde_json::json!({ "reading": reading, "rows": rows })
}

fn gate_summary(training: &StopSignatureTraining) -> Value {
    serde_json::json!({
        "terminal_rows": training.terminal_rows,
        "nonterminal_rows": training.nonterminal_rows,
        "terminal_stops": training.terminal_stops,
        "nonterminal_stops": training.nonterminal_stops,
        "nonterminal_stop_limit": training.nonterminal_stop_limit,
        "margin": training.gate.margin,
    })
}

fn signature_census(rows: &[StopSignatureRow]) -> Value {
    let terminal = rows
        .iter()
        .filter(|row| row.terminal)
        .map(|row| row.signature)
        .collect::<BTreeSet<_>>();
    let nonterminal = rows
        .iter()
        .filter(|row| !row.terminal)
        .map(|row| row.signature)
        .collect::<BTreeSet<_>>();
    serde_json::json!({
        "terminal_rows": rows.iter().filter(|row| row.terminal).count(),
        "nonterminal_rows": rows.iter().filter(|row| !row.terminal).count(),
        "distinct_terminal": terminal.len(),
        "distinct_nonterminal": nonterminal.len(),
    })
}

fn no_gate() -> StopSignatureGate {
    StopSignatureGate {
        signature_bits: uor_r4_cpu_train::STOP_SIGNATURE_BITS,
        terminal_prototypes: vec![[u64::MAX; uor_r4_cpu_train::STOP_SIGNATURE_WORDS]],
        nonterminal_prototypes: vec![[0; uor_r4_cpu_train::STOP_SIGNATURE_WORDS]],
        margin: i64::MAX,
    }
}

fn number(value: &Value, key: &str, usize_value: bool) -> Result<usize> {
    let value = value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| format!("{key} is not a usize"))?;
    if !usize_value && value > u32::MAX as usize {
        return Err(format!("{key} exceeds u32"));
    }
    Ok(value)
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn ceil_two_sqrt(value: u64) -> u64 {
    let value = u128::from(value);
    let mut root = 0u128;
    while root * root < 4 * value {
        root += 1;
    }
    root.try_into().unwrap_or(u64::MAX)
}

fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-copy-generated-stop ARTIFACT TRAIN_REPORT TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST DEV_REPORT DEV_TOKENS DEV_MASK DEV_MANIFEST [--prototypes N] [--seed N] [--exact-body] [--min-train-exact N] [--min-dev-exact N] [--min-dev-replies N] [--min-dev-oracle N] [--min-distinct-terminal N]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(step: usize, exact: bool, premature: bool, at_reference: bool) -> ReplyRow {
        ReplyRow {
            row: step,
            fired_step: (premature || at_reference).then_some(step),
            class: if premature {
                StopClass::Premature
            } else if at_reference {
                StopClass::AtReference
            } else {
                StopClass::NoFire
            },
            exact,
            premature,
            at_reference,
            selected_sha256: token_hash(&[step as u32]),
        }
    }

    #[test]
    fn gate_comparison_uses_exact_rows_and_two_sigma() {
        let candidate = vec![
            decision(0, true, false, true),
            decision(1, true, false, true),
        ];
        let control = vec![
            decision(0, false, false, false),
            decision(1, false, false, false),
        ];
        let result = compare_rows(&candidate, &control).unwrap();
        assert_eq!(result["candidate_only"], 2);
        assert_eq!(result["threshold"], 2);
        assert_eq!(result["passes"], true);
    }

    #[test]
    fn no_gate_sentinel_never_fires() {
        let gate = no_gate();
        let signature = [0; uor_r4_cpu_train::STOP_SIGNATURE_WORDS];
        assert!(!stop_signature_fires(&gate, &signature).unwrap());
    }

    #[test]
    fn oracle_requires_all_reference_tokens() {
        assert_eq!(
            oracle_exact(&[Trajectory {
                row: 0,
                selected: vec![1, 2, 3],
                reference: vec![1, 2],
                decisions: Vec::new(),
            }]),
            1
        );
        assert_eq!(
            oracle_exact(&[Trajectory {
                row: 0,
                selected: vec![1, 9, 3],
                reference: vec![1, 2],
                decisions: Vec::new(),
            }]),
            0
        );
    }

    #[test]
    fn exact_body_requires_complete_reference_prefix() {
        assert!(body_matches_reference(&[1, 2, 3], &[1, 2]));
        assert!(!body_matches_reference(&[1, 9, 3], &[1, 2]));
        assert!(!body_matches_reference(&[1], &[1, 2]));
    }

    #[test]
    fn exact_cap_does_not_require_a_trailing_token() {
        assert_eq!(replay_token(&[1, 2], 1, 2, 0).unwrap(), Some(2));
        assert_eq!(replay_token(&[1, 2], 2, 2, 0).unwrap(), None);
        assert!(replay_token(&[1], 1, 2, 0).is_err());
    }

    #[test]
    fn exact_body_selection_uses_only_clean_rows() {
        let row = |terminal| StopSignatureRow {
            signature: [u64::from(terminal); uor_r4_cpu_train::STOP_SIGNATURE_WORDS],
            terminal,
            writer_token: u32::from(terminal),
        };
        let panel = PanelTrace {
            trajectories: Vec::new(),
            all_rows: vec![row(false), row(true), row(false)],
            exact_rows: vec![row(true)],
        };
        assert_eq!(select_training_rows(&panel, false).len(), 3);
        assert_eq!(select_training_rows(&panel, true).len(), 1);
        assert!(select_training_rows(&panel, true)[0].terminal);
    }

    #[test]
    fn signature_census_deduplicates_each_class() {
        let row = |signature_word, terminal| StopSignatureRow {
            signature: [signature_word; uor_r4_cpu_train::STOP_SIGNATURE_WORDS],
            terminal,
            writer_token: 0,
        };
        let census = signature_census(&[
            row(1, true),
            row(1, true),
            row(2, true),
            row(3, false),
            row(3, false),
        ]);
        assert_eq!(census["terminal_rows"], 3);
        assert_eq!(census["nonterminal_rows"], 2);
        assert_eq!(census["distinct_terminal"], 2);
        assert_eq!(census["distinct_nonterminal"], 1);
    }

    #[test]
    fn bar_requires_registered_power_and_floor() {
        let candidate = (0..10)
            .map(|step| decision(step, step < 7, false, step < 7))
            .collect::<Vec<_>>();
        let control = (0..10)
            .map(|step| decision(step, false, false, false))
            .collect::<Vec<_>>();
        let insufficient = panel_bar(BarInput {
            replies: 10,
            oracle: 7,
            minimum_exact: 7,
            minimum_replies: 10,
            minimum_oracle: 7,
            training_power: false,
            true_rows: &candidate,
            shuffled_rows: &control,
            all_reference_rows: Some(&control),
            no_gate_rows: &control,
        })
        .unwrap();
        assert_eq!(insufficient["passes"], false);
        let sufficient = panel_bar(BarInput {
            replies: 10,
            oracle: 7,
            minimum_exact: 7,
            minimum_replies: 10,
            minimum_oracle: 7,
            training_power: true,
            true_rows: &candidate,
            shuffled_rows: &control,
            all_reference_rows: Some(&control),
            no_gate_rows: &control,
        })
        .unwrap();
        assert_eq!(sufficient["passes"], true);
    }
}
