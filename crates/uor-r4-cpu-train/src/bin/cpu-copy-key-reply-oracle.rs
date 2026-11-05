use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use uor_r4_cpu_train::{
    compare_copy_key_reply_rows, normalized_reference, validate_corpus_manifest, CopyKeyReplyArm,
    CopyKeyReplyMetric, CopyKeyReplyRow, CpuMaskedCorpus,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
struct ArmEvaluation {
    reading: Value,
    source_reading: Value,
    rows: Vec<CopyKeyReplyRow>,
    max_common_prefix: usize,
    common_prefix_sum: u64,
    trailing_mismatch_histogram: BTreeMap<usize, u64>,
    oracle_exact: u64,
    mean_common_prefix: f64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-copy-key-reply-oracle: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut values = std::env::args().skip(1);
    let report_path = PathBuf::from(values.next().ok_or_else(usage)?);
    let tokens_path = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask_path = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest_path = PathBuf::from(values.next().ok_or_else(usage)?);
    if values.next().is_some() {
        return Err(usage());
    }

    let report: Value = serde_json::from_slice(&read(&report_path)?)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;
    if report.get("schema").and_then(Value::as_str) != Some("uor-r4.cpu-copy-key-reply/1") {
        return Err("report schema differs".to_owned());
    }
    let token_bytes = read(&tokens_path)?;
    let mask_bytes = read(&mask_path)?;
    let manifest_bytes = read(&manifest_path)?;
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
        return Err("report token or mask hash differs from the panel".to_owned());
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
    let evaluated = report
        .pointer("/examples/evaluated")
        .and_then(Value::as_u64)
        .ok_or_else(|| "report has no examples.evaluated".to_owned())?;
    if usize::try_from(evaluated).ok() != Some(samples.len()) {
        return Err(format!(
            "report evaluated {evaluated} examples but corpus recovered {}",
            samples.len()
        ));
    }

    let arm_values = report
        .get("arms")
        .and_then(Value::as_object)
        .ok_or_else(|| "report has no arms object".to_owned())?;
    let arm_set: &[CopyKeyReplyArm] = if arm_values.len() == CopyKeyReplyArm::ALL.len() {
        &CopyKeyReplyArm::ALL
    } else if arm_values.len() == CopyKeyReplyArm::EVALUATION_ARMS.len() {
        &CopyKeyReplyArm::EVALUATION_ARMS
    } else {
        return Err("report arm count differs".to_owned());
    };
    let mut arms = BTreeMap::new();
    for arm in arm_set.iter().copied() {
        let value = arm_values
            .get(arm.name())
            .ok_or_else(|| format!("report lacks arm {}", arm.name()))?;
        let expected_name = arm_reading_name(arm.name());
        arms.insert(
            arm.name().to_owned(),
            evaluate_arm(value, &samples, &expected_name)?,
        );
    }

    let longest = arms
        .get("longest_4_to_1")
        .ok_or_else(|| "longest-first arm missing".to_owned())?;
    let mut comparisons = BTreeMap::new();
    for control in [
        CopyKeyReplyArm::K2BackoffK1,
        CopyKeyReplyArm::FixedDepth1,
        CopyKeyReplyArm::RandomAvailableDepth,
        CopyKeyReplyArm::Writer,
    ] {
        let control = arms
            .get(control.name())
            .ok_or_else(|| format!("{} arm missing", control.name()))?;
        let exact = compare_copy_key_reply_rows(
            &longest.rows,
            &control.rows,
            CopyKeyReplyMetric::ExactReference,
        )
        .map_err(|error| error.to_string())?;
        let body = compare_copy_key_reply_rows(
            &longest.rows,
            &control.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?;
        comparisons.insert(
            control.reading["arm"]
                .as_str()
                .unwrap_or(control.reading_name())
                .to_owned(),
            serde_json::json!({
                "exact_reference": exact,
                "correct_body_positions": body,
            }),
        );
    }

    let copy_exact = [
        "longest_4_to_1",
        "k2_backoff_k1",
        "fixed_depth_1",
        "random_available_depth",
    ]
    .iter()
    .filter_map(|name| arms.get(*name).map(|arm| arm.oracle_exact))
    .sum::<u64>();
    let core_controls_pass = ["k2_backoff_k1", "fixed_depth_1", "random_available_depth"]
        .iter()
        .all(|name| {
            comparisons
                .get(&arm_reading_name(name))
                .and_then(|value| value["correct_body_positions"]["passes"].as_bool())
                == Some(true)
        });
    let cell_outcome = if copy_exact > 0 && core_controls_pass {
        "termination_only_blocker"
    } else if copy_exact == 0 {
        "content_remains_insufficient"
    } else {
        "mixed"
    };

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-key-reply-oracle/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "sealed-trace oracle-stop ceiling; no new model decode or language claim",
        "inputs": {
            "report": report_path,
            "tokens": tokens_path,
            "tokens_sha256": identity.tokens_sha256,
            "mask": mask_path,
            "mask_sha256": identity.mask_sha256,
            "manifest": manifest_path,
        },
        "settings": {
            "oracle_stop": "BOS at normalized reference length",
            "rows": samples.len(),
        },
        "cell_outcome": cell_outcome,
        "comparisons": comparisons,
        "arms": arms.iter().map(|(name, arm)| {
            (name.clone(), serde_json::json!({
                "reading": arm.reading,
                "source_reading": arm.source_reading,
                "max_common_prefix": arm.max_common_prefix,
                "common_prefix_sum": arm.common_prefix_sum,
                "trailing_mismatch_histogram": arm.trailing_mismatch_histogram,
                "mean_common_prefix": arm.mean_common_prefix,
            }))
        }).collect::<serde_json::Map<_, _>>(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn evaluate_arm(
    report: &Value,
    samples: &[uor_r4_cpu_train::CpuConversationSample],
    expected_arm_name: &str,
) -> Result<ArmEvaluation> {
    let steps = report
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| "arm has no steps".to_owned())?;
    let source_generated_tokens = report
        .pointer("/reading/generated_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(|| "arm has no source generated-token count".to_owned())?;
    if steps.len() as u64 != source_generated_tokens {
        return Err("step count differs from the source generated-token count".to_owned());
    }
    let source_arm = report
        .pointer("/reading/arm")
        .and_then(Value::as_str)
        .ok_or_else(|| "arm has no source name".to_owned())?;
    if source_arm != expected_arm_name {
        return Err("source arm name differs".to_owned());
    }
    let source_rows = report
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "arm has no row readings".to_owned())?;
    if source_rows.len() != samples.len() {
        return Err("source row count differs from corpus examples".to_owned());
    }
    let mut by_row = vec![Vec::<(usize, u32)>::new(); samples.len()];
    for step in steps {
        let row = step
            .get("row")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "step row differs".to_owned())?;
        let generated_step = step
            .get("generated_step")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "step index differs".to_owned())?;
        let token = step
            .get("selected_token")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| "step token differs".to_owned())?;
        by_row
            .get_mut(row)
            .ok_or_else(|| "step row outside samples".to_owned())?
            .push((generated_step, token));
    }

    let mut rows = Vec::with_capacity(samples.len());
    let mut max_common_prefix = 0;
    let mut common_prefix_sum = 0u64;
    let mut trailing_mismatch_histogram = BTreeMap::new();
    let mut oracle_exact = 0u64;
    let mut opening_correct = 0u64;
    for (row, sample) in samples.iter().enumerate() {
        let mut trace = std::mem::take(&mut by_row[row]);
        trace.sort_unstable_by_key(|(step, _)| *step);
        for (expected, (actual, _)) in trace.iter().enumerate() {
            if expected != *actual {
                return Err(format!("row {row} has a non-monotonic decision trace"));
            }
        }
        let generated = trace
            .into_iter()
            .map(|(_, token)| token)
            .collect::<Vec<_>>();
        let source_reply_sha256 = source_rows[row]
            .get("reply_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("row {row} has no reply hash"))?;
        if token_hash(&generated) != source_reply_sha256 {
            return Err(format!("row {row} full-trace hash differs"));
        }
        let reference = normalized_reference(&sample.reply);
        if generated.len() < reference.len() {
            return Err(format!("row {row} trace is shorter than its reference"));
        }
        let body = &generated[..reference.len()];
        let common_prefix = body
            .iter()
            .zip(reference)
            .take_while(|(generated, reference)| generated == reference)
            .count();
        max_common_prefix = max_common_prefix.max(common_prefix);
        common_prefix_sum += common_prefix as u64;
        let trailing_mismatch = reference.len() - common_prefix;
        if trailing_mismatch <= 3 {
            *trailing_mismatch_histogram
                .entry(trailing_mismatch)
                .or_insert(0) += 1;
        }
        let exact = body == reference;
        oracle_exact += u64::from(exact);
        opening_correct += u64::from(matches!(
            (body.first(), reference.first()),
            (Some(generated), Some(reference)) if generated == reference
        ));
        rows.push(CopyKeyReplyRow {
            row,
            reachable: true,
            exact_reference: exact,
            bos_complete: exact,
            opening_correct: matches!(
                (body.first(), reference.first()),
                (Some(generated), Some(reference)) if generated == reference
            ),
            premature_bos: false,
            non_premature_bos: true,
            generated_tokens: reference.len() as u64,
            reference_tokens: reference.len() as u64,
            correct_body_positions: common_prefix as u64,
            copy_decisions: trace_len(report, row)?,
            fallback_decisions: 0,
            stop: "bos".to_owned(),
            reply_sha256: token_hash(body),
        });
    }
    let reference_tokens = rows.iter().map(|row| row.reference_tokens).sum::<u64>();
    let mean_common_prefix = common_prefix_sum as f64 / rows.len() as f64;
    Ok(ArmEvaluation {
        reading: serde_json::json!({
            "arm": report.pointer("/reading/arm").cloned().unwrap_or(Value::Null),
            "replies": rows.len(),
            "reference_reachable": rows.len(),
            "exact_reference": oracle_exact,
            "bos_complete": oracle_exact,
            "opening_correct": opening_correct,
            "premature_bos": 0,
            "non_premature_bos": rows.len(),
            "generated_tokens": reference_tokens,
            "reference_tokens": reference_tokens,
            "correct_body_positions": common_prefix_sum,
            "oracle_stop": "bos_at_reference_length",
        }),
        source_reading: report
            .get("reading")
            .cloned()
            .ok_or_else(|| "arm has no reading".to_owned())?,
        rows,
        max_common_prefix,
        common_prefix_sum,
        trailing_mismatch_histogram,
        oracle_exact,
        mean_common_prefix,
    })
}

fn trace_len(report: &Value, row: usize) -> Result<u64> {
    let steps = report
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| "arm has no steps".to_owned())?;
    Ok(steps
        .iter()
        .filter(|step| step.get("row").and_then(Value::as_u64) == Some(row as u64))
        .count() as u64)
}

fn arm_reading_name(name: &str) -> String {
    match name {
        "longest_32_to_1" => "longest32_to1",
        "longest_4_to_1" => "longest4_to1",
        "k2_backoff_k1" => "k2_backoff_k1",
        "fixed_depth_1" => "fixed_depth1",
        "random_available_depth" => "random_available_depth",
        "random_available_depth_32" => "random_available_depth32",
        "writer" => "writer",
        _ => name,
    }
    .to_owned()
}

impl ArmEvaluation {
    fn reading_name(&self) -> &str {
        self.reading["arm"].as_str().unwrap_or("control")
    }
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-copy-key-reply-oracle REPORT_JSON TOKENS MASK MANIFEST".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_cpu_train::CpuConversationSample;

    fn sample(prompt: Vec<u32>, reply: Vec<u32>) -> CpuConversationSample {
        CpuConversationSample {
            document_ordinal: 0,
            prompt,
            reply,
        }
    }

    fn report(steps: Value, tokens: &[u32]) -> Value {
        serde_json::json!({
            "schema": "uor-r4.cpu-copy-key-reply/1",
            "steps": steps,
            "reading": {
                "arm": "longest4_to1",
                "generated_tokens": tokens.len(),
            },
            "rows": [{"reply_sha256": token_hash(tokens)}],
        })
    }

    #[test]
    fn oracle_body_uses_reference_horizon_and_common_prefix() {
        let samples = vec![sample(vec![1, 2], vec![3, 4, 5])];
        let value = report(
            serde_json::json!([
                {"row": 0, "generated_step": 2, "selected_token": 5},
                {"row": 0, "generated_step": 0, "selected_token": 3},
                {"row": 0, "generated_step": 3, "selected_token": 9},
                {"row": 0, "generated_step": 1, "selected_token": 4}
            ]),
            &[3, 4, 5, 9],
        );
        let evaluated = evaluate_arm(&value, &samples, "longest4_to1").unwrap();
        assert_eq!(evaluated.max_common_prefix, 3);
        assert_eq!(evaluated.common_prefix_sum, 3);
        assert_eq!(evaluated.trailing_mismatch_histogram.get(&0), Some(&1));
        assert!(evaluated.rows[0].exact_reference);
        assert_eq!(evaluated.rows[0].generated_tokens, 3);
        assert_eq!(evaluated.oracle_exact, 1);
        assert_eq!(evaluated.reading["exact_reference"], 1);
        assert_eq!(evaluated.mean_common_prefix, 3.0);
    }

    #[test]
    fn non_monotonic_and_short_traces_are_refused() {
        let samples = vec![sample(vec![1], vec![2, 3])];
        let duplicate = report(
            serde_json::json!([
                {"row": 0, "generated_step": 0, "selected_token": 2},
                {"row": 0, "generated_step": 0, "selected_token": 3}
            ]),
            &[2, 3],
        );
        assert!(evaluate_arm(&duplicate, &samples, "longest4_to1").is_err());
        let short = report(
            serde_json::json!([
                {"row": 0, "generated_step": 0, "selected_token": 2}
            ]),
            &[2],
        );
        assert!(evaluate_arm(&short, &samples, "longest4_to1").is_err());

        let mut removed_step = report(
            serde_json::json!([
                {"row": 0, "generated_step": 0, "selected_token": 2},
                {"row": 0, "generated_step": 1, "selected_token": 3}
            ]),
            &[2, 3],
        );
        removed_step["steps"].as_array_mut().unwrap().pop();
        assert!(evaluate_arm(&removed_step, &samples, "longest4_to1").is_err());

        let mut bad_hash = report(
            serde_json::json!([
                {"row": 0, "generated_step": 0, "selected_token": 2},
                {"row": 0, "generated_step": 1, "selected_token": 3}
            ]),
            &[2, 3],
        );
        bad_hash["rows"][0]["reply_sha256"] = serde_json::json!("deadbeef");
        assert!(evaluate_arm(&bad_hash, &samples, "longest4_to1").is_err());
    }

    #[test]
    fn one_token_tail_is_reported_as_a_near_miss_not_exact() {
        let samples = vec![sample(vec![1], vec![2, 3, 4])];
        let value = report(
            serde_json::json!([
                {"row": 0, "generated_step": 0, "selected_token": 2},
                {"row": 0, "generated_step": 1, "selected_token": 3},
                {"row": 0, "generated_step": 2, "selected_token": 9}
            ]),
            &[2, 3, 9],
        );
        let evaluated = evaluate_arm(&value, &samples, "longest4_to1").unwrap();
        assert_eq!(evaluated.oracle_exact, 0);
        assert_eq!(evaluated.max_common_prefix, 2);
        assert_eq!(evaluated.trailing_mismatch_histogram.get(&1), Some(&1));
    }
}
