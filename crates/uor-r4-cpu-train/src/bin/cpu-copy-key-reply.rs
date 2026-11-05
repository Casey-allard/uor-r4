use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_copy_key_reply_rows, evaluate_copy_key_replies, validate_corpus_manifest,
    CopyKeyReplyArm, CopyKeyReplyMetric, CpuMaskedCorpus, MemoryWeightSample,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-copy-key-reply: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut values = std::env::args().skip(1);
    let tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let artifact = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut cap = 32usize;
    let mut seed = 153u64;
    let mut allow_underpowered = false;
    while let Some(flag) = values.next() {
        if flag == "--allow-underpowered" {
            allow_underpowered = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))?;
        match flag.as_str() {
            "--cap" => {
                cap = value
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
    if !(1..=64).contains(&cap) {
        return Err("copy-key reply cap must be in 1..=64".to_owned());
    }

    let token_bytes = read(&tokens)?;
    let mask_bytes = read(&mask)?;
    let manifest_bytes = read(&manifest)?;
    let artifact_bytes = read(&artifact)?;
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask_bytes,
        identity.vocab,
        identity.max_tokens,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let examples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let samples = examples
        .iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt.clone(),
            reply: sample.reply.clone(),
        })
        .collect::<Vec<_>>();

    let mut arms = BTreeMap::new();
    for arm in CopyKeyReplyArm::EVALUATION_ARMS {
        let reading = evaluate_copy_key_replies(&artifact_bytes, &samples, cap, arm, seed)
            .map_err(|error| error.to_string())?;
        if !allow_underpowered && reading.reading.reference_reachable < 200 {
            return Err(format!(
                "{} has {} reachable complete-reply examples, below the 200-row floor",
                arm.name(),
                reading.reading.reference_reachable
            ));
        }
        arms.insert(arm.name().to_owned(), reading);
    }

    let candidate = arms
        .get(CopyKeyReplyArm::GeometricCopy32.name())
        .ok_or_else(|| "geometric-copy reading is missing".to_owned())?;
    let longest = arms
        .get(CopyKeyReplyArm::Longest32To1.name())
        .ok_or_else(|| "longest-32 reading is missing".to_owned())?;
    let longest_4 = arms
        .get(CopyKeyReplyArm::Longest4To1.name())
        .ok_or_else(|| "longest-4 reading is missing".to_owned())?;
    let random = arms
        .get(CopyKeyReplyArm::RandomAvailableDepth32.name())
        .ok_or_else(|| "random-depth-32 reading is missing".to_owned())?;
    let random_4 = arms
        .get(CopyKeyReplyArm::RandomAvailableDepth.name())
        .ok_or_else(|| "random-depth-4 reading is missing".to_owned())?;
    let mut comparisons = BTreeMap::new();
    for control in [
        CopyKeyReplyArm::Longest32To1,
        CopyKeyReplyArm::Longest4To1,
        CopyKeyReplyArm::K2BackoffK1,
        CopyKeyReplyArm::FixedDepth1,
        CopyKeyReplyArm::RandomAvailableDepth32,
        CopyKeyReplyArm::Writer,
    ] {
        let control_reading = arms
            .get(control.name())
            .ok_or_else(|| format!("{} reading is missing", control.name()))?;
        let mut by_metric = BTreeMap::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let comparison =
                compare_copy_key_reply_rows(&candidate.rows, &control_reading.rows, metric)
                    .map_err(|error| error.to_string())?;
            by_metric.insert(metric_name(metric).to_owned(), comparison);
        }
        comparisons.insert(control.name().to_owned(), by_metric);
    }

    let random_changes_from_longest_32 = random_changes(longest, random);
    let geometric_changes_from_longest_32 = random_changes(candidate, longest);
    let random_changes = random_changes(longest_4, random_4);
    let input_sha256 = uor_r4_discrete::corpus::sha256(&artifact_bytes);
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-key-reply/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded complete-reply writer override; no language or serving claim",
        "inputs": {
            "tokens": tokens,
            "tokens_sha256": identity.tokens_sha256,
            "mask": mask,
            "mask_sha256": identity.mask_sha256,
            "manifest": manifest,
            "manifest_split": identity.split,
            "artifact": artifact,
            "artifact_sha256": input_sha256,
        },
        "settings": {
            "cap": cap,
            "seed": seed,
            "maximum_copy_depth": uor_r4_cpu_train::COPY_KEY_REPLY_MAX_DEPTH,
            "arms": CopyKeyReplyArm::EVALUATION_ARMS.iter().map(|arm| arm.name()).collect::<Vec<_>>(),
        },
        "examples": {
            "documents": corpus.document_count(),
            "total": examples.len(),
            "evaluated": samples.len(),
            "reachable": longest.reading.reference_reachable,
        },
        "comparisons": comparisons,
        "random_nonvacuity": random_changes,
        "random_nonvacuity_from_longest_32": random_changes_from_longest_32,
        "geometric_changes_from_longest_32": geometric_changes_from_longest_32,
        "arms": arms,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn random_changes(
    longest: &uor_r4_cpu_train::TimedCopyKeyReplyReading,
    random: &uor_r4_cpu_train::TimedCopyKeyReplyReading,
) -> serde_json::Value {
    let reachable_rows = longest
        .rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let longest_steps = longest
        .steps
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let random_steps = random
        .steps
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let changed = longest_steps
        .iter()
        .filter_map(|(key, token)| {
            if !reachable_rows.contains(&key.0) {
                return None;
            }
            random_steps
                .get(key)
                .filter(|random_token| *random_token != token)
                .map(|_| *key)
        })
        .collect::<Vec<_>>();
    let changed_rows = changed
        .iter()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>()
        .len();
    let changed_steps = changed.len();
    let first = changed.first();
    serde_json::json!({
        "changed_steps": changed_steps,
        "changed_rows": changed_rows,
        "first_changed_step": first.map(|(row, step)| serde_json::json!({
            "row": row,
            "generated_step": step,
        })),
        "non_vacuous": changed_steps > 0,
    })
}

fn metric_name(metric: CopyKeyReplyMetric) -> &'static str {
    match metric {
        CopyKeyReplyMetric::ExactReference => "exact_reference",
        CopyKeyReplyMetric::NonPrematureBos => "non_premature_bos",
        CopyKeyReplyMetric::OpeningCorrect => "opening_correct",
        CopyKeyReplyMetric::CorrectBodyPositions => "correct_body_positions",
    }
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-copy-key-reply TOKENS MASK MANIFEST ARTIFACT [--cap N] [--seed N] [--allow-underpowered]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_every_required_input() {
        let usage = usage();
        for word in ["TOKENS", "MASK", "MANIFEST", "ARTIFACT", "--cap", "--seed"] {
            assert!(usage.contains(word), "{usage}");
        }
    }

    #[test]
    fn metric_names_are_stable() {
        assert_eq!(
            metric_name(CopyKeyReplyMetric::ExactReference),
            "exact_reference"
        );
        assert_eq!(
            metric_name(CopyKeyReplyMetric::CorrectBodyPositions),
            "correct_body_positions"
        );
    }
}
