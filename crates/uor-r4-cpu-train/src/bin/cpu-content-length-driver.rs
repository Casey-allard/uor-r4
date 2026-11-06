use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_suffix_arbitration_rows, derive_full_sequence_suffix_content_memory,
    derive_prompt_retrieval_memory, derive_reply_boundary_memory, evaluate_prompt_retrieval,
    evaluate_suffix_arbitration_replies, evaluate_suffix_arbitration_replies_to_length,
    normalized_reference, shuffle_suffix_memory, suffix_arbitration_changes,
    validate_corpus_manifest, CopyKeyReplyMetric, CpuCorpusIdentity, CpuMaskedCorpus,
    MemoryWeightSample, PromptRetrievalArm, SuffixArbitrationArm, TimedSuffixArbitrationReading,
};

struct Panel {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    manifest_sha256: String,
    identity: CpuCorpusIdentity,
    samples: Vec<MemoryWeightSample>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-content-length-driver: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let train = load_panel(&config.train)?;
    let panel = load_panel(&config.panel)?;
    let artifact = read(&config.artifact)?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    if artifact_sha256 != config.expected_artifact_sha256 {
        return Err(format!(
            "artifact sha256 {artifact_sha256} != expected {}",
            config.expected_artifact_sha256
        ));
    }

    let suffix = derive_full_sequence_suffix_content_memory(&train.samples)
        .map_err(|error| error.to_string())?;
    let suffix_census = suffix.census();
    if suffix_census.bos_transitions != 0 {
        return Err("full-sequence suffix table contains BOS targets".to_owned());
    }
    let (shuffled, changed_targets) =
        shuffle_suffix_memory(&suffix, config.seed).map_err(|error| error.to_string())?;
    let boundary =
        derive_reply_boundary_memory(&train.samples).map_err(|error| error.to_string())?;

    let retrieval = derive_prompt_retrieval_memory(&artifact, &train.samples, config.cap)
        .map_err(|error| error.to_string())?;
    let full_retrieval = evaluate_prompt_retrieval(
        &artifact,
        &retrieval,
        &retrieval,
        &panel.samples,
        config.cap,
        PromptRetrievalArm::UnigramJaccardOrdinalTie,
    )
    .map_err(|error| error.to_string())?;
    let lengths = full_retrieval
        .steps
        .iter()
        .map(|step| normalized_reference(&train.samples[step.selected_ordinal].reply).len())
        .collect::<Vec<_>>();
    if lengths
        .iter()
        .any(|&length| length == 0 || length > config.cap)
    {
        return Err("retrieved reply length is outside 1..=cap".to_owned());
    }

    let candidate = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let boundary_arm = evaluate_suffix_arbitration_replies(
        &artifact,
        &panel.samples,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_arm = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequenceShuffled,
        Some(&shuffled),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let shallow = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::ShallowLocalGlobal4,
        Some(&suffix),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let local = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::LocalLongest32Boundary,
        None,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let writer = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::WriterBoundary,
        None,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let oracle = evaluate_suffix_arbitration_replies(
        &artifact,
        &panel.samples,
        config.cap,
        SuffixArbitrationArm::ReferenceLengthOracle,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;

    let comparisons = compare_arms(
        &candidate,
        &boundary_arm,
        &shuffled_arm,
        &shallow,
        &local,
        &writer,
    )?;
    let changes_from_local = suffix_arbitration_changes(&candidate, &local);
    let changes_from_shuffled = suffix_arbitration_changes(&candidate, &shuffled_arm);
    let reachable = candidate.reading.reference_reachable;
    let reachability_consistent = [
        boundary_arm.reading.reference_reachable,
        shuffled_arm.reading.reference_reachable,
        shallow.reading.reference_reachable,
        local.reading.reference_reachable,
        writer.reading.reference_reachable,
        full_retrieval.reading.reference_reachable,
        oracle.reading.reference_reachable,
    ]
    .into_iter()
    .all(|arm_reachable| arm_reachable == reachable);
    let emitted = candidate.reading.bos_complete + candidate.reading.length_complete;
    let all_emitted = emitted == reachable;
    let exact_local = comparisons["local_longest_32_length"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false);
    let exact_shuffled = comparisons["shuffled_suffix_length"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false);
    let best_body_control = boundary_arm
        .reading
        .correct_body_positions
        .max(local.reading.correct_body_positions);
    let body_pass = candidate.reading.correct_body_positions >= best_body_control;
    let active = candidate.reading.global_decisions >= 100
        && changes_from_local.changed_steps > 0
        && changes_from_shuffled.changed_steps > 0
        && changed_targets > 0;
    let output = if reachable < 200 || candidate.reading.global_decisions < 100 {
        "INSUFFICIENT_POWER"
    } else if !reachability_consistent
        || !active
        || !all_emitted
        || candidate.reading.premature_bos != 0
    {
        "CONTROL_FAULT"
    } else if exact_local && exact_shuffled && body_pass {
        "KEEP"
    } else {
        "REJECT"
    };

    let readings = serde_json::json!({
        "full_sequence_stored_length": candidate.reading,
        "full_sequence_boundary": boundary_arm.reading,
        "shuffled_suffix_stored_length": shuffled_arm.reading,
        "shallow_global_stored_length": shallow.reading,
        "local_longest_stored_length": local.reading,
        "writer_stored_length": writer.reading,
        "full_retrieval": full_retrieval.reading,
        "reference_length_oracle": oracle.reading,
    });
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-content-length/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "full-sequence suffix content with reference-independent stored-length completion on a public CPU development panel",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "length_source": "nearest TRAIN prompt's stored reply",
            "retrieval": "raw unigram Jaccard, lower ordinal on ties",
            "suffix_depth": uor_r4_cpu_train::SUFFIX_MEMORY_MAX_DEPTH,
            "minimum_observations": uor_r4_cpu_train::SUFFIX_ARBITRATION_MIN_OBSERVATIONS,
            "shuffle_changed_targets": changed_targets,
        },
        "table": {
            "content": suffix_census,
            "shuffled": shuffled.census(),
        },
        "length_census": {
            "minimum": lengths.iter().copied().min().unwrap_or(0),
            "maximum": lengths.iter().copied().max().unwrap_or(0),
            "mean_x1000": lengths.iter().sum::<usize>() * 1000 / lengths.len().max(1),
        },
        "changes": {
            "from_local": changes_from_local,
            "from_shuffled": changes_from_shuffled,
        },
        "comparisons": comparisons,
        "readings": readings,
        "arm_hashes": {
            "full_sequence_stored_length": {
                "reply": candidate.reply_sha256,
                "decision": candidate.decision_sha256,
            },
            "full_sequence_boundary": {
                "reply": boundary_arm.reply_sha256,
                "decision": boundary_arm.decision_sha256,
            },
            "shuffled_suffix_stored_length": {
                "reply": shuffled_arm.reply_sha256,
                "decision": shuffled_arm.decision_sha256,
            },
        },
        "bar": {
            "reachable": reachable,
            "emitted": emitted,
            "premature_bos": candidate.reading.premature_bos,
            "global_decisions": candidate.reading.global_decisions,
            "reachability_consistent": reachability_consistent,
            "exact_vs_local": exact_local,
            "exact_vs_shuffled": exact_shuffled,
            "body_candidate": candidate.reading.correct_body_positions,
            "body_best_boundary_or_local": best_body_control,
            "body_retained": body_pass,
            "control_active": active,
            "outcome": output,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn compare_arms(
    candidate: &TimedSuffixArbitrationReading,
    boundary: &TimedSuffixArbitrationReading,
    shuffled: &TimedSuffixArbitrationReading,
    shallow: &TimedSuffixArbitrationReading,
    local: &TimedSuffixArbitrationReading,
    writer: &TimedSuffixArbitrationReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("full_sequence_boundary", boundary),
        ("shuffled_suffix_length", shuffled),
        ("shallow_global_length", shallow),
        ("local_longest_32_length", local),
        ("writer_length", writer),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_suffix_arbitration_rows(&candidate.rows, &control.rows, metric)
                .map_err(|error| error.to_string())?;
            let name = match metric {
                CopyKeyReplyMetric::ExactReference => "exact_reference",
                CopyKeyReplyMetric::CorrectBodyPositions => "correct_body_positions",
                _ => unreachable!(),
            };
            metrics.insert(
                name.to_owned(),
                serde_json::to_value(reading).map_err(|error| error.to_string())?,
            );
        }
        comparisons.insert(name.to_owned(), serde_json::Value::Object(metrics));
    }
    Ok(serde_json::Value::Object(comparisons))
}

fn panel_input(panel: &Panel) -> serde_json::Value {
    serde_json::json!({
        "tokens": panel.tokens,
        "tokens_sha256": panel.identity.tokens_sha256,
        "mask": panel.mask,
        "mask_sha256": panel.identity.mask_sha256,
        "manifest": panel.manifest,
        "manifest_sha256": panel.manifest_sha256,
        "manifest_split": panel.identity.split,
    })
}

fn load_panel(paths: &PanelPaths) -> Result<Panel, String> {
    let token_bytes = read(&paths.tokens)?;
    let mask_bytes = read(&paths.mask)?;
    let manifest_bytes = read(&paths.manifest)?;
    let manifest_sha256 = uor_r4_discrete::corpus::sha256(&manifest_bytes);
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    if identity.tokens_sha256 != paths.expected_tokens_sha256
        || identity.mask_sha256 != paths.expected_mask_sha256
        || manifest_sha256 != paths.expected_manifest_sha256
    {
        return Err(format!(
            "{}: panel identity differs",
            paths.tokens.display()
        ));
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
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt,
            reply: sample.reply,
        })
        .collect();
    Ok(Panel {
        tokens: paths.tokens.clone(),
        mask: paths.mask.clone(),
        manifest: paths.manifest.clone(),
        manifest_sha256,
        identity,
        samples,
    })
}

struct PanelPaths {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

struct Config {
    train: PanelPaths,
    panel: PanelPaths,
    artifact: PathBuf,
    expected_artifact_sha256: String,
    cap: usize,
    seed: u64,
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut cap = 64usize;
    let mut seed = 223u64;
    let mut expected = BTreeMap::<String, String>::new();
    while let Some(flag) = args.next() {
        if !flag.starts_with("--") {
            positional.push(PathBuf::from(flag));
            continue;
        }
        let value = args
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
            "--expected-train-tokens" => set_expected(&mut expected, "train_tokens", value, &flag)?,
            "--expected-train-mask" => set_expected(&mut expected, "train_mask", value, &flag)?,
            "--expected-train-manifest" => {
                set_expected(&mut expected, "train_manifest", value, &flag)?
            }
            "--expected-panel-tokens" => set_expected(&mut expected, "panel_tokens", value, &flag)?,
            "--expected-panel-mask" => set_expected(&mut expected, "panel_mask", value, &flag)?,
            "--expected-panel-manifest" => {
                set_expected(&mut expected, "panel_manifest", value, &flag)?
            }
            "--expected-artifact" => set_expected(&mut expected, "artifact", value, &flag)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if positional.len() != 7 || !(1..=64).contains(&cap) {
        return Err(usage());
    }
    for key in [
        "train_tokens",
        "train_mask",
        "train_manifest",
        "panel_tokens",
        "panel_mask",
        "panel_manifest",
        "artifact",
    ] {
        let value = expected
            .get(key)
            .ok_or_else(|| format!("--expected-{} is required", key.replace('_', "-")))?;
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("--expected-{key} is not a SHA-256"));
        }
    }
    let panel_paths = |start: usize, prefix: &str| PanelPaths {
        tokens: positional[start].clone(),
        mask: positional[start + 1].clone(),
        manifest: positional[start + 2].clone(),
        expected_tokens_sha256: expected[&format!("{prefix}_tokens")].to_ascii_lowercase(),
        expected_mask_sha256: expected[&format!("{prefix}_mask")].to_ascii_lowercase(),
        expected_manifest_sha256: expected[&format!("{prefix}_manifest")].to_ascii_lowercase(),
    };
    Ok(Config {
        train: panel_paths(0, "train"),
        panel: panel_paths(3, "panel"),
        artifact: positional[6].clone(),
        expected_artifact_sha256: expected["artifact"].to_ascii_lowercase(),
        cap,
        seed,
    })
}

fn set_expected(
    expected: &mut BTreeMap<String, String>,
    key: &str,
    value: String,
    flag: &str,
) -> Result<(), String> {
    if expected.insert(key.to_owned(), value).is_some() {
        return Err(format!("{flag} given more than once"));
    }
    Ok(())
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-content-length-driver TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
     PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT \
     --expected-train-tokens HEX --expected-train-mask HEX \
     --expected-train-manifest HEX --expected-panel-tokens HEX \
     --expected-panel-mask HEX --expected-panel-manifest HEX \
     --expected-artifact HEX [--cap N] [--seed N]"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_every_required_input() {
        let usage = usage();
        for word in [
            "TRAIN_TOKENS",
            "TRAIN_MASK",
            "TRAIN_MANIFEST",
            "PANEL_TOKENS",
            "PANEL_MASK",
            "PANEL_MANIFEST",
            "ARTIFACT",
            "--expected-artifact",
        ] {
            assert!(usage.contains(word), "{usage}");
        }
    }
}
