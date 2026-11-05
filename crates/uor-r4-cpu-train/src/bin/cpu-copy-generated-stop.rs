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
    rows: Vec<StopSignatureRow>,
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
    while let Some(flag) = values.next() {
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

    let true_training = fit_stop_signature_gate(&train.rows, prototypes)
        .map_err(|error| format!("true gate fit: {error}"))?;
    let shuffled_rows = shuffle_stop_signature_rows(&train.rows, seed)
        .map_err(|error| format!("shuffled rows: {error}"))?;
    if shuffled_rows
        .iter()
        .zip(&train.rows)
        .all(|(left, right)| left.terminal == right.terminal)
    {
        return Err("shuffled gate is vacuous".to_owned());
    }
    let shuffled_training = fit_stop_signature_gate(&shuffled_rows, prototypes)
        .map_err(|error| format!("shuffled gate fit: {error}"))?;

    let train_true = evaluate_gate(&train.trajectories, &true_training.gate)?;
    let train_shuffled = evaluate_gate(&train.trajectories, &shuffled_training.gate)?;
    let train_none = evaluate_gate(&train.trajectories, &no_gate())?;
    let dev_true = evaluate_gate(&dev.trajectories, &true_training.gate)?;
    let dev_shuffled = evaluate_gate(&dev.trajectories, &shuffled_training.gate)?;
    let dev_none = evaluate_gate(&dev.trajectories, &no_gate())?;

    let train_oracle = oracle_exact(&train.trajectories);
    let dev_oracle = oracle_exact(&dev.trajectories);
    let train_bar = panel_bar(
        train.trajectories.len(),
        train_oracle,
        &train_true.1,
        &train_shuffled.1,
        &train_none.1,
    )?;
    let dev_bar = panel_bar(
        dev.trajectories.len(),
        dev_oracle,
        &dev_true.1,
        &dev_shuffled.1,
        &dev_none.1,
    )?;

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
            "terminal_label": "generated_step == normalized reference length",
        },
        "training": {
            "true": gate_summary(&true_training),
            "shuffled": gate_summary(&shuffled_training),
        },
        "train": {
            "oracle_exact": train_oracle,
            "true": reply_summary(&train_true),
            "shuffled": reply_summary(&train_shuffled),
            "no_gate": reply_summary(&train_none),
            "bar": train_bar,
        },
        "dev": {
            "oracle_exact": dev_oracle,
            "true": reply_summary(&dev_true),
            "shuffled": reply_summary(&dev_shuffled),
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
    let mut rows = Vec::new();
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
            reference.len(),
            row,
            &mut rows,
        )?;
        trajectories.push(Trajectory {
            row,
            selected,
            reference,
            decisions,
        });
    }
    Ok(PanelTrace { trajectories, rows })
}

fn replay_decisions(
    artifact: &[u8],
    prompt: &[u32],
    selected: &[u32],
    reference_len: usize,
    row: usize,
    rows: &mut Vec<StopSignatureRow>,
) -> Result<Vec<Decision>> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| format!("replay forward: {error}"))?;
    let context = forward.context_length();
    if prompt.is_empty() || prompt.len() > context || selected.len() <= reference_len {
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
    for (generated_step, &next_token) in selected.iter().take(reference_len + 1).enumerate() {
        let terminal = generated_step == reference_len;
        let signature = pack_stop_signature(&step.query_q12)
            .map_err(|error| format!("row {row} signature: {error}"))?;
        decisions.push(Decision {
            generated_step,
            signature,
        });
        rows.push(StopSignatureRow {
            signature,
            terminal,
            writer_token: step.token,
        });
        if terminal {
            break;
        }
        step = forward
            .step(next_token, CpuFbsHead::Writer)
            .map_err(|error| format!("row {row} generated step: {error}"))?;
    }
    Ok(decisions)
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
        .filter(|trajectory| {
            trajectory.selected[..trajectory.reference.len()] == trajectory.reference
        })
        .count() as u64
}

fn panel_bar(
    replies: usize,
    oracle: u64,
    true_rows: &[ReplyRow],
    shuffled_rows: &[ReplyRow],
    no_gate_rows: &[ReplyRow],
) -> Result<Value> {
    let true_reading = summarize_rows(true_rows);
    let shuffled_reading = summarize_rows(shuffled_rows);
    let no_gate_reading = summarize_rows(no_gate_rows);
    let floor = oracle.div_ceil(2);
    let false_stop_limit = replies as u64 / 100;
    let shuffled = compare_rows(true_rows, shuffled_rows)?;
    let no_gate = compare_rows(true_rows, no_gate_rows)?;
    Ok(serde_json::json!({
        "oracle_floor": floor,
        "true_exact": true_reading.exact,
        "true_premature": true_reading.premature,
        "false_stop_limit": false_stop_limit,
        "shuffled_exact": shuffled_reading.exact,
        "no_gate_exact": no_gate_reading.exact,
        "true_vs_shuffled": shuffled,
        "true_vs_no_gate": no_gate,
        "passes": true_reading.exact >= floor
            && true_reading.premature <= false_stop_limit
            && shuffled["passes"].as_bool() == Some(true)
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
    "usage: cpu-copy-generated-stop ARTIFACT TRAIN_REPORT TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST DEV_REPORT DEV_TOKENS DEV_MASK DEV_MANIFEST [--prototypes N] [--seed N]".to_owned()
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
}
