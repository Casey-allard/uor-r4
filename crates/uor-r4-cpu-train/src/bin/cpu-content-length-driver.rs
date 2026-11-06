use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_suffix_arbitration_rows, derive_full_sequence_suffix_content_memory,
    derive_prompt_retrieval_memory, derive_reply_boundary_memory, evaluate_prompt_retrieval,
    evaluate_suffix_arbitration_replies, evaluate_suffix_arbitration_replies_to_length,
    evaluate_suffix_arbitration_replies_with_aligned_endpoint,
    evaluate_suffix_arbitration_replies_with_branch_endpoint, normalized_reference,
    shuffle_prompt_retrieval_replies, shuffle_suffix_memory, suffix_arbitration_changes,
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
    let retrieved_replies = full_retrieval
        .steps
        .iter()
        .map(|step| {
            retrieval
                .reply_at(step.selected_ordinal)
                .map(<[u32]>::to_vec)
                .ok_or_else(|| "retrieved reply ordinal is missing".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let lengths = retrieved_replies.iter().map(Vec::len).collect::<Vec<_>>();
    if lengths
        .iter()
        .any(|&length| length == 0 || length > config.cap)
    {
        return Err("retrieved reply length is outside 1..=cap".to_owned());
    }
    let (shuffled_retrieval, changed_retrieval_replies) =
        shuffle_prompt_retrieval_replies(&retrieval, config.seed)
            .map_err(|error| error.to_string())?;
    let shuffled_retrieval_reading = evaluate_prompt_retrieval(
        &artifact,
        &shuffled_retrieval,
        &shuffled_retrieval,
        &panel.samples,
        config.cap,
        PromptRetrievalArm::UnigramJaccardOrdinalTie,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_replies = shuffled_retrieval_reading
        .steps
        .iter()
        .map(|step| {
            shuffled_retrieval
                .reply_at(step.selected_ordinal)
                .map(<[u32]>::to_vec)
                .ok_or_else(|| "shuffled retrieved reply ordinal is missing".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let reference_replies = panel
        .samples
        .iter()
        .map(|sample| normalized_reference(&sample.reply).to_vec())
        .collect::<Vec<_>>();
    let retrieved_branches = panel
        .samples
        .iter()
        .map(|sample| {
            retrieval
                .top_replies(&sample.prompt, config.top_k)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let shuffled_branches = panel
        .samples
        .iter()
        .map(|sample| {
            shuffled_retrieval
                .top_replies(&sample.prompt, config.top_k)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let reference_branches = reference_replies
        .iter()
        .map(|reply| vec![reply.clone()])
        .collect::<Vec<_>>();

    let length_candidate = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let aligned_candidate = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
        &artifact,
        &panel.samples,
        &retrieved_replies,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let branch_candidate = evaluate_suffix_arbitration_replies_with_branch_endpoint(
        &artifact,
        &panel.samples,
        &retrieved_branches,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let candidate = match config.endpoint {
        EndpointMode::Length => &length_candidate,
        EndpointMode::AlignedRetrieved => &aligned_candidate,
        EndpointMode::TopKBranches => &branch_candidate,
    };
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
    let shuffled_suffix_length = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequenceShuffled,
        Some(&shuffled),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_endpoint_guard = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
        &artifact,
        &panel.samples,
        &shuffled_replies,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_branch_guard = evaluate_suffix_arbitration_replies_with_branch_endpoint(
        &artifact,
        &panel.samples,
        &shuffled_branches,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let shallow_length = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::ShallowLocalGlobal4,
        Some(&suffix),
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let local_length = evaluate_suffix_arbitration_replies_to_length(
        &artifact,
        &panel.samples,
        &lengths,
        config.cap,
        SuffixArbitrationArm::LocalLongest32Boundary,
        None,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let local_aligned = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
        &artifact,
        &panel.samples,
        &retrieved_replies,
        config.cap,
        SuffixArbitrationArm::LocalLongest32Boundary,
        None,
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let local_branch = evaluate_suffix_arbitration_replies_with_branch_endpoint(
        &artifact,
        &panel.samples,
        &retrieved_branches,
        config.cap,
        SuffixArbitrationArm::LocalLongest32Boundary,
        None,
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let writer = evaluate_suffix_arbitration_replies(
        &artifact,
        &panel.samples,
        config.cap,
        SuffixArbitrationArm::WriterBoundary,
        None,
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let reference_length_oracle = evaluate_suffix_arbitration_replies(
        &artifact,
        &panel.samples,
        config.cap,
        SuffixArbitrationArm::ReferenceLengthOracle,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let reference_endpoint_oracle = evaluate_suffix_arbitration_replies_with_aligned_endpoint(
        &artifact,
        &panel.samples,
        &reference_replies,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let reference_branch_oracle = evaluate_suffix_arbitration_replies_with_branch_endpoint(
        &artifact,
        &panel.samples,
        &reference_branches,
        config.cap,
        SuffixArbitrationArm::DeeperFullSequence32,
        Some(&suffix),
        &boundary,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let (shuffled_arm, local, oracle, active_changed) = match config.endpoint {
        EndpointMode::Length => (
            &shuffled_suffix_length,
            &local_length,
            &reference_length_oracle,
            changed_targets,
        ),
        EndpointMode::AlignedRetrieved => (
            &shuffled_endpoint_guard,
            &local_aligned,
            &reference_endpoint_oracle,
            changed_retrieval_replies,
        ),
        EndpointMode::TopKBranches => (
            &shuffled_branch_guard,
            &local_branch,
            &reference_branch_oracle,
            changed_retrieval_replies,
        ),
    };

    let comparisons = compare_arms(
        candidate,
        &boundary_arm,
        shuffled_arm,
        &shallow_length,
        local,
        &writer,
    )?;
    let changes_from_local = suffix_arbitration_changes(candidate, local);
    let changes_from_shuffled = suffix_arbitration_changes(candidate, shuffled_arm);
    let changes_from_boundary = suffix_arbitration_changes(candidate, &boundary_arm);
    let complete_rows_changed_from_boundary = complete_outcome_changes(candidate, &boundary_arm);
    let complete_rows_changed_from_shuffled = complete_outcome_changes(candidate, shuffled_arm);
    let reachable = candidate.reading.reference_reachable;
    let reachability_consistent = [
        boundary_arm.reading.reference_reachable,
        shuffled_arm.reading.reference_reachable,
        shallow_length.reading.reference_reachable,
        local.reading.reference_reachable,
        writer.reading.reference_reachable,
        full_retrieval.reading.reference_reachable,
        oracle.reading.reference_reachable,
    ]
    .into_iter()
    .all(|arm_reachable| arm_reachable == reachable);
    let emitted = candidate.reading.bos_complete
        + candidate.reading.length_complete
        + candidate.reading.retrieved_endpoint_complete;
    let all_emitted = emitted == reachable;
    let comparison_control = match config.endpoint {
        EndpointMode::Length => "shuffled_endpoint_control",
        EndpointMode::AlignedRetrieved | EndpointMode::TopKBranches => "shuffled_endpoint_control",
    };
    let exact_shuffled = comparisons[comparison_control]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false);
    let exact_retained = candidate.reading.exact_reference >= boundary_arm.reading.exact_reference;
    let body_retained =
        candidate.reading.correct_body_positions >= boundary_arm.reading.correct_body_positions;
    let changed = match config.endpoint {
        EndpointMode::Length => {
            changes_from_local.changed_steps > 0 && changes_from_shuffled.changed_steps > 0
        }
        EndpointMode::AlignedRetrieved | EndpointMode::TopKBranches => {
            complete_rows_changed_from_boundary > 0 && complete_rows_changed_from_shuffled > 0
        }
    };
    let active = candidate.reading.global_decisions >= 100 && changed && active_changed > 0;
    let endpoint_power = match config.endpoint {
        EndpointMode::Length => true,
        EndpointMode::AlignedRetrieved | EndpointMode::TopKBranches => {
            candidate.reading.retrieved_endpoint_complete >= 20
        }
    };
    let output = if reachable < 200 || candidate.reading.global_decisions < 100 || !endpoint_power {
        "INSUFFICIENT_POWER"
    } else if !reachability_consistent || !active {
        "CONTROL_FAULT"
    } else {
        if all_emitted
            && candidate.reading.premature_bos == 0
            && exact_retained
            && body_retained
            && exact_shuffled
        {
            "KEEP"
        } else {
            "REJECT"
        }
    };

    let readings = serde_json::json!({
        "candidate": candidate.reading,
        "full_sequence_stored_length": length_candidate.reading,
        "aligned_retrieved_endpoint": aligned_candidate.reading,
        "full_sequence_boundary": boundary_arm.reading,
        "shuffled_suffix_stored_length": shuffled_suffix_length.reading,
        "shuffled_retrieval_guard": shuffled_endpoint_guard.reading,
        "shuffled_branch_guard": shuffled_branch_guard.reading,
        "shallow_global_stored_length": shallow_length.reading,
        "local_longest_stored_length": local_length.reading,
        "local_longest_aligned_endpoint": local_aligned.reading,
        "local_longest_branch_endpoint": local_branch.reading,
        "writer_boundary": writer.reading,
        "full_retrieval": full_retrieval.reading,
        "reference_length_oracle": reference_length_oracle.reading,
        "reference_endpoint_oracle": reference_endpoint_oracle.reading,
        "reference_branch_oracle": reference_branch_oracle.reading,
    });
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-content-endpoint/2",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "full-sequence suffix content with a reference-independent aligned retrieved endpoint and boundary fallback on a public CPU development panel",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "endpoint": match config.endpoint {
                EndpointMode::Length => "stored_length",
                EndpointMode::AlignedRetrieved => "aligned_retrieved_with_boundary_fallback",
                EndpointMode::TopKBranches => "top_k_branches_with_boundary_fallback",
            },
            "top_k": config.top_k,
            "retrieval": "raw unigram Jaccard, lower ordinal on ties",
            "suffix_depth": uor_r4_cpu_train::SUFFIX_MEMORY_MAX_DEPTH,
            "minimum_observations": uor_r4_cpu_train::SUFFIX_ARBITRATION_MIN_OBSERVATIONS,
            "shuffle_changed_targets": changed_targets,
            "retrieval_shuffle_changed_replies": changed_retrieval_replies,
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
            "from_boundary": changes_from_boundary,
            "complete_rows_from_shuffled": complete_rows_changed_from_shuffled,
            "complete_rows_from_boundary": complete_rows_changed_from_boundary,
        },
        "comparisons": comparisons,
        "readings": readings,
        "arm_hashes": {
            "candidate": {
                "reply": candidate.reply_sha256,
                "decision": candidate.decision_sha256,
            },
            "full_sequence_boundary": {
                "reply": boundary_arm.reply_sha256,
                "decision": boundary_arm.decision_sha256,
            },
            "endpoint_control": {
                "reply": shuffled_arm.reply_sha256,
                "decision": shuffled_arm.decision_sha256,
            },
        },
        "bar": {
            "reachable": reachable,
            "emitted": emitted,
            "premature_bos": candidate.reading.premature_bos,
            "global_decisions": candidate.reading.global_decisions,
            "endpoint_completions": candidate.reading.retrieved_endpoint_complete,
            "reachability_consistent": reachability_consistent,
            "exact_vs_shuffled": exact_shuffled,
            "body_candidate": candidate.reading.correct_body_positions,
            "boundary_body": boundary_arm.reading.correct_body_positions,
            "body_retained": body_retained,
            "exact_retained": exact_retained,
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

fn complete_outcome_changes(
    candidate: &TimedSuffixArbitrationReading,
    control: &TimedSuffixArbitrationReading,
) -> u64 {
    candidate
        .rows
        .iter()
        .zip(&control.rows)
        .filter(|(candidate, control)| {
            candidate.reply_sha256 != control.reply_sha256 || candidate.stop != control.stop
        })
        .count() as u64
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
        ("shuffled_endpoint_control", shuffled),
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum EndpointMode {
    Length,
    AlignedRetrieved,
    TopKBranches,
}

struct Config {
    train: PanelPaths,
    panel: PanelPaths,
    artifact: PathBuf,
    expected_artifact_sha256: String,
    cap: usize,
    seed: u64,
    top_k: usize,
    endpoint: EndpointMode,
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut cap = 64usize;
    let mut seed = 223u64;
    let mut top_k = 8usize;
    let mut endpoint = EndpointMode::Length;
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
            "--endpoint" => {
                endpoint = match value.as_str() {
                    "length" => EndpointMode::Length,
                    "aligned-retrieved" => EndpointMode::AlignedRetrieved,
                    "top-k-branches" => EndpointMode::TopKBranches,
                    _ => return Err(format!("unknown endpoint {value}\n{}", usage())),
                }
            }
            "--top-k" => {
                top_k = value
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
    if positional.len() != 7 || !(1..=64).contains(&cap) || !(1..=64).contains(&top_k) {
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
        top_k,
        endpoint,
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
     --expected-artifact HEX [--cap N] [--seed N] \
     [--endpoint length|aligned-retrieved|top-k-branches] [--top-k N]"
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
            "--endpoint",
            "--top-k",
        ] {
            assert!(usage.contains(word), "{usage}");
        }
    }
}
