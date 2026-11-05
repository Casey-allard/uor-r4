use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    calibrate_expanded_candidate_union, compare_anchored_candidate_rows,
    compare_copy_key_reply_rows, compare_geometric_candidate_rows, content_mapping_sha256,
    derive_anchored_candidate_rankers, derive_copy_candidate_rankers,
    derive_geometric_candidate_rankers, derive_learned_copy_readout,
    evaluate_anchored_candidate_replies, evaluate_copy_key_replies,
    evaluate_geometric_candidate_replies, evaluate_learned_copy_replies,
    evaluate_ranked_copy_replies, shuffle_content_targets, validate_corpus_manifest,
    AnchoredCandidateArm, CopyCandidateRanker, CopyKeyReplyArm, CopyKeyReplyMetric,
    CopyKeyReplyRow, CopyKeyReplyStep, CopyRankerTrajectory, CpuCorpusIdentity, CpuMaskedCorpus,
    GeometricCandidateArm, GeometricCandidateStep, LearnedCopyArm, LearnedCopyStep,
    MemoryWeightSample, RankedCopyStep,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-learned-copy-read: {error}");
        std::process::exit(1);
    }
}

struct Readouts<'a> {
    learned: &'a uor_r4_cpu_train::ContentReadout,
    shuffled: &'a uor_r4_cpu_train::ContentReadout,
}

struct Rankers<'a> {
    trained: &'a CopyCandidateRanker,
    shuffled: &'a CopyCandidateRanker,
    depth_only: &'a CopyCandidateRanker,
}

struct PanelRun<'a> {
    artifact: &'a [u8],
    samples: &'a [MemoryWeightSample],
    cap: usize,
    seed: u64,
    readouts: Readouts<'a>,
    rankers: Rankers<'a>,
    documents: usize,
    allow_underpowered: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RankerCandidates {
    ExactCopy,
    GeometricUnion,
    AnchoredExpanded,
}

fn run() -> Result<(), String> {
    let mut values = std::env::args().skip(1);
    let train_tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let train_manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let test_tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let test_mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let test_manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let artifact = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut cap = 64usize;
    let mut seed = 153u64;
    let mut ranker_trajectory = CopyRankerTrajectory::Longest32;
    let mut ranker_candidates = RankerCandidates::ExactCopy;
    let mut candidate_cells = 0usize;
    let mut candidate_calibration = None;
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
            "--ranker-trajectory" => {
                ranker_trajectory = match value.as_str() {
                    "longest32" => CopyRankerTrajectory::Longest32,
                    "reference" => CopyRankerTrajectory::ReferencePrefix,
                    _ => {
                        return Err(format!(
                            "--ranker-trajectory expects longest32 or reference: {value}"
                        ))
                    }
                };
            }
            "--ranker-candidates" => {
                ranker_candidates = match value.as_str() {
                    "exact-copy" => RankerCandidates::ExactCopy,
                    "geometric-union" => RankerCandidates::GeometricUnion,
                    "anchored-expanded" => RankerCandidates::AnchoredExpanded,
                    _ => {
                        return Err(format!(
                            "--ranker-candidates expects exact-copy, geometric-union or anchored-expanded: {value}"
                        ))
                    }
                };
            }
            "--candidate-cells" => {
                candidate_cells = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--candidate-calibration" => {
                candidate_calibration = Some(parse_cells(&value)?);
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if !(1..=64).contains(&cap) {
        return Err("learned-copy reply cap must be in 1..=64".to_owned());
    }

    let artifact_bytes = read(&artifact)?;
    let (train_samples, train_identity, train_documents) =
        load_samples(&train_tokens, &train_mask, &train_manifest)?;
    if let Some(cells) = candidate_calibration {
        return run_candidate_calibration(CandidateCalibrationInputs {
            artifact: &artifact_bytes,
            train_samples: &train_samples,
            train_identity: &train_identity,
            train_tokens: &train_tokens,
            train_mask: &train_mask,
            train_manifest: &train_manifest,
            artifact_path: &artifact,
            cap,
            cells: &cells,
        });
    }
    let (test_samples, test_identity, test_documents) =
        load_samples(&test_tokens, &test_mask, &test_manifest)?;
    if train_identity.tokenizer_sha256 != test_identity.tokenizer_sha256
        || train_identity.vocab != test_identity.vocab
        || train_identity.bos != test_identity.bos
    {
        return Err("learned-copy train and test corpora differ".to_owned());
    }
    if ranker_candidates == RankerCandidates::GeometricUnion {
        if ranker_trajectory != CopyRankerTrajectory::ReferencePrefix {
            return Err(
                "geometric-union candidate ranking requires --ranker-trajectory reference"
                    .to_owned(),
            );
        }
        return run_geometric_candidate_union(GeometricCandidateInputs {
            artifact: &artifact_bytes,
            train_samples: &train_samples,
            test_samples: &test_samples,
            train_identity: &train_identity,
            test_identity: &test_identity,
            train_tokens: &train_tokens,
            train_mask: &train_mask,
            train_manifest: &train_manifest,
            test_tokens: &test_tokens,
            test_mask: &test_mask,
            test_manifest: &test_manifest,
            artifact_path: &artifact,
            cap,
            seed,
            allow_underpowered,
        });
    }
    if ranker_candidates == RankerCandidates::AnchoredExpanded {
        if ranker_trajectory != CopyRankerTrajectory::ReferencePrefix {
            return Err(
                "anchored-expanded candidate ranking requires --ranker-trajectory reference"
                    .to_owned(),
            );
        }
        if candidate_cells == 0 {
            return Err("anchored-expanded candidate ranking needs --candidate-cells N".to_owned());
        }
        return run_anchored_candidate_union(AnchoredCandidateInputs {
            artifact: &artifact_bytes,
            train_samples: &train_samples,
            test_samples: &test_samples,
            train_identity: &train_identity,
            test_identity: &test_identity,
            train_tokens: &train_tokens,
            train_mask: &train_mask,
            train_manifest: &train_manifest,
            test_tokens: &test_tokens,
            test_mask: &test_mask,
            test_manifest: &test_manifest,
            artifact_path: &artifact,
            cap,
            seed,
            cells: candidate_cells,
            allow_underpowered,
        });
    }

    let training_started = Instant::now();
    let training = derive_learned_copy_readout(&artifact_bytes, &train_samples, cap)
        .map_err(|error| error.to_string())?;
    let training_elapsed_us = training_started.elapsed().as_micros() as u64;
    let shuffled = shuffle_content_targets(&training.readout.readout, seed)
        .map_err(|error| error.to_string())?;
    let readout_mapping_sha256 =
        content_mapping_sha256(&training.readout.readout).map_err(|error| error.to_string())?;
    let shuffled_mapping_sha256 =
        content_mapping_sha256(&shuffled).map_err(|error| error.to_string())?;
    let ranker_started = Instant::now();
    let ranker_training = derive_copy_candidate_rankers(
        &artifact_bytes,
        &train_samples,
        cap,
        seed,
        ranker_trajectory,
    )
    .map_err(|error| error.to_string())?;
    let ranker_elapsed_us = ranker_started.elapsed().as_micros() as u64;

    let train_report = evaluate_panel(PanelRun {
        artifact: &artifact_bytes,
        samples: &train_samples,
        cap,
        seed,
        readouts: Readouts {
            learned: &training.readout.readout,
            shuffled: &shuffled,
        },
        rankers: Rankers {
            trained: &ranker_training.trained,
            shuffled: &ranker_training.shuffled,
            depth_only: &ranker_training.depth_only,
        },
        documents: train_documents,
        allow_underpowered,
    })?;
    let test_report = evaluate_panel(PanelRun {
        artifact: &artifact_bytes,
        samples: &test_samples,
        cap,
        seed,
        readouts: Readouts {
            learned: &training.readout.readout,
            shuffled: &shuffled,
        },
        rankers: Rankers {
            trained: &ranker_training.trained,
            shuffled: &ranker_training.shuffled,
            depth_only: &ranker_training.depth_only,
        },
        documents: test_documents,
        allow_underpowered,
    })?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact_bytes);
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-learned-copy-read/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded complete-reply candidate arbitration; no language or serving claim",
        "inputs": {
            "artifact": artifact,
            "artifact_sha256": artifact_sha256,
            "train_tokens": train_tokens,
            "train_mask": train_mask,
            "train_manifest": train_manifest,
            "train_tokens_sha256": train_identity.tokens_sha256,
            "train_mask_sha256": train_identity.mask_sha256,
            "train_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(&train_manifest)?),
            "train_split": train_identity.split,
            "test_tokens": test_tokens,
            "test_mask": test_mask,
            "test_manifest": test_manifest,
            "test_tokens_sha256": test_identity.tokens_sha256,
            "test_mask_sha256": test_identity.mask_sha256,
            "test_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(&test_manifest)?),
            "test_split": test_identity.split,
            "tokenizer_sha256": test_identity.tokenizer_sha256,
        },
        "settings": {
            "cap": cap,
            "seed": seed,
            "signature_mode": "modulo_256_code",
            "signature_bits": training.readout.readout.signature_bits,
            "maximum_copy_depth": uor_r4_cpu_train::COPY_KEY_REPLY_MAX_DEPTH,
            "training_row_limit": uor_r4_cpu_train::LEARNED_COPY_MAX_TRAINING_ROWS,
        },
        "training": {
            "generated_rows": training.generated_rows,
            "retained_rows": training.retained_rows,
            "target_values": training.readout.target_values,
            "leave_one_out_top1": training.readout.leave_one_out_top1,
            "leave_one_out_ambiguous": training.readout.leave_one_out_ambiguous,
            "covered_rows": training.readout.covered_rows,
            "max_distance": training.readout.readout.max_distance,
            "mapping_sha256": readout_mapping_sha256,
            "shuffled_mapping_sha256": shuffled_mapping_sha256,
            "elapsed_us": training_elapsed_us,
        },
        "candidate_ranker": {
            "trajectory": ranker_training.trajectory.name(),
            "features": uor_r4_cpu_train::COPY_RANKER_FEATURE_NAMES,
            "epochs": uor_r4_cpu_train::COPY_RANKER_EPOCHS,
            "max_depth": uor_r4_cpu_train::COPY_RANKER_MAX_DEPTH,
            "generated_rows": ranker_training.generated_rows,
            "supervised_groups": ranker_training.supervised_groups,
            "candidate_rows": ranker_training.candidate_rows,
            "target_present_groups": ranker_training.target_present_groups,
            "shuffled_target_changes": ranker_training.shuffled_target_changes,
            "trained_weights": ranker_training.trained.weights,
            "trained_weights_sha256": ranker_training.trained.weights_sha256(),
            "shuffled_weights": ranker_training.shuffled.weights,
            "shuffled_weights_sha256": ranker_training.shuffled.weights_sha256(),
            "depth_only_weights": ranker_training.depth_only.weights,
            "elapsed_us": ranker_elapsed_us,
        },
        "train": train_report,
        "test": test_report,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

struct CandidateCalibrationInputs<'a> {
    artifact: &'a [u8],
    train_samples: &'a [MemoryWeightSample],
    train_identity: &'a CpuCorpusIdentity,
    train_tokens: &'a Path,
    train_mask: &'a Path,
    train_manifest: &'a Path,
    artifact_path: &'a Path,
    cap: usize,
    cells: &'a [usize],
}

fn run_candidate_calibration(inputs: CandidateCalibrationInputs<'_>) -> Result<(), String> {
    let started = Instant::now();
    let readings = calibrate_expanded_candidate_union(
        inputs.artifact,
        inputs.train_samples,
        inputs.cap,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let elapsed_us = started.elapsed().as_micros() as u64;
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-expanded-candidate-calibration/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded train-panel candidate eligibility; no language or serving claim",
        "inputs": {
            "artifact": inputs.artifact_path.display().to_string(),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(inputs.artifact),
            "train_tokens": inputs.train_tokens.display().to_string(),
            "train_mask": inputs.train_mask.display().to_string(),
            "train_manifest": inputs.train_manifest.display().to_string(),
            "train_tokens_sha256": inputs.train_identity.tokens_sha256,
            "train_mask_sha256": inputs.train_identity.mask_sha256,
            "train_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(inputs.train_manifest)?),
            "train_split": inputs.train_identity.split,
            "tokenizer_sha256": inputs.train_identity.tokenizer_sha256,
        },
        "settings": {
            "cap": inputs.cap,
            "cells": inputs.cells,
            "fresh_panel_read": false,
            "served_writer_cells": 8,
        },
        "calibration": readings,
        "elapsed_us": elapsed_us,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn parse_cells(value: &str) -> Result<Vec<usize>, String> {
    value
        .split(',')
        .map(|part| {
            part.parse::<usize>()
                .map_err(|_| format!("candidate cell count is not a number: {part}"))
        })
        .collect()
}

struct GeometricCandidateInputs<'a> {
    artifact: &'a [u8],
    train_samples: &'a [MemoryWeightSample],
    test_samples: &'a [MemoryWeightSample],
    train_identity: &'a CpuCorpusIdentity,
    test_identity: &'a CpuCorpusIdentity,
    train_tokens: &'a Path,
    train_mask: &'a Path,
    train_manifest: &'a Path,
    test_tokens: &'a Path,
    test_mask: &'a Path,
    test_manifest: &'a Path,
    artifact_path: &'a Path,
    cap: usize,
    seed: u64,
    allow_underpowered: bool,
}

fn run_geometric_candidate_union(inputs: GeometricCandidateInputs<'_>) -> Result<(), String> {
    let training_started = Instant::now();
    let training = derive_geometric_candidate_rankers(
        inputs.artifact,
        inputs.train_samples,
        inputs.cap,
        inputs.seed,
    )
    .map_err(|error| error.to_string())?;
    let training_elapsed_us = training_started.elapsed().as_micros() as u64;
    let test_started = Instant::now();
    let trained = evaluate_geometric_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        GeometricCandidateArm::Trained,
        &training.trained,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = evaluate_geometric_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        GeometricCandidateArm::Shuffled,
        &training.shuffled,
    )
    .map_err(|error| error.to_string())?;
    let geometry_only = evaluate_geometric_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        GeometricCandidateArm::GeometryOnly,
        &training.geometry_only,
    )
    .map_err(|error| error.to_string())?;
    let longest = evaluate_copy_key_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        CopyKeyReplyArm::Longest32To1,
        inputs.seed,
    )
    .map_err(|error| error.to_string())?;
    let writer = evaluate_copy_key_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        CopyKeyReplyArm::Writer,
        inputs.seed,
    )
    .map_err(|error| error.to_string())?;
    let test_elapsed_us = test_started.elapsed().as_micros() as u64;
    if !inputs.allow_underpowered
        && [
            trained.reading.reference_reachable,
            shuffled.reading.reference_reachable,
            geometry_only.reading.reference_reachable,
            longest.reading.reference_reachable,
            writer.reading.reference_reachable,
        ]
        .into_iter()
        .any(|reachable| reachable < 200)
    {
        return Err("geometric candidate control has fewer than 200 reachable replies".to_owned());
    }
    let comparisons = serde_json::json!({
        "geometry_only": compare_geometric_candidate_rows(
            &trained.rows,
            &geometry_only.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "shuffled": compare_geometric_candidate_rows(
            &trained.rows,
            &shuffled.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "longest_32": compare_copy_key_reply_rows(
            &trained.rows,
            &longest.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "writer": compare_copy_key_reply_rows(
            &trained.rows,
            &writer.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
    });
    let changes = serde_json::json!({
        "geometry_only": geometric_candidate_changes(
            &trained.rows,
            &trained.steps,
            &geometry_only.steps,
        ),
        "shuffled": geometric_candidate_changes(
            &trained.rows,
            &trained.steps,
            &shuffled.steps,
        ),
        "longest_32": copy_control_changes(&trained.rows, &trained.steps, &longest.steps),
        "writer": copy_control_changes(&trained.rows, &trained.steps, &writer.steps),
    });
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(inputs.artifact);
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-geometric-candidate-ranker/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded next-token candidate arbitration; no language or serving claim",
        "inputs": {
            "artifact": inputs.artifact_path.display().to_string(),
            "artifact_sha256": artifact_sha256,
            "train_tokens": inputs.train_tokens.display().to_string(),
            "train_mask": inputs.train_mask.display().to_string(),
            "train_manifest": inputs.train_manifest.display().to_string(),
            "train_tokens_sha256": inputs.train_identity.tokens_sha256,
            "train_mask_sha256": inputs.train_identity.mask_sha256,
            "train_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(inputs.train_manifest)?),
            "train_split": inputs.train_identity.split,
            "test_tokens": inputs.test_tokens.display().to_string(),
            "test_mask": inputs.test_mask.display().to_string(),
            "test_manifest": inputs.test_manifest.display().to_string(),
            "test_tokens_sha256": inputs.test_identity.tokens_sha256,
            "test_mask_sha256": inputs.test_identity.mask_sha256,
            "test_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(inputs.test_manifest)?),
            "test_split": inputs.test_identity.split,
            "tokenizer_sha256": inputs.test_identity.tokenizer_sha256,
        },
        "settings": {
            "cap": inputs.cap,
            "seed": inputs.seed,
            "candidate_set": training.candidate_set,
            "maximum_copy_depth": training.exact_depth,
            "signature_mode": "sign_hamming_and_geometry",
        },
        "training": {
            "trajectory": training.trajectory,
            "supervised_groups": training.supervised_groups,
            "target_present_groups": training.target_present_groups,
            "target_values": training.target_values,
            "generated_rows": training.generated_rows,
            "candidate_rows": training.candidate_rows,
            "shuffled_target_changes": training.shuffled_target_changes,
            "elapsed_us": training_elapsed_us,
        },
        "candidate_ranker": {
            "features": uor_r4_cpu_train::GEOMETRIC_CANDIDATE_FEATURE_NAMES,
            "epochs": uor_r4_cpu_train::GEOMETRIC_CANDIDATE_EPOCHS,
            "writer_candidates": uor_r4_cpu_train::GEOMETRIC_CANDIDATE_TOP_WRITER,
            "exact_depth": uor_r4_cpu_train::COPY_KEY_REPLY_MAX_DEPTH,
            "trained_weights": training.trained.weights,
            "trained_weights_sha256": training.trained.weights_sha256(),
            "shuffled_weights": training.shuffled.weights,
            "shuffled_weights_sha256": training.shuffled.weights_sha256(),
            "depth_only_weights": training.depth_only.weights,
            "depth_only_weights_sha256": training.depth_only.weights_sha256(),
            "geometry_only_weights": training.geometry_only.weights,
        },
        "test": {
            "trained": trained.reading,
            "shuffled": shuffled.reading,
            "geometry_only": geometry_only.reading,
            "longest_32": longest.reading,
            "writer": writer.reading,
            "comparisons": comparisons,
            "changes": changes,
            "signatures": {
                "trained": {
                    "reply_sha256": trained.reply_sha256,
                    "decision_sha256": trained.decision_sha256,
                },
                "shuffled": {
                    "reply_sha256": shuffled.reply_sha256,
                    "decision_sha256": shuffled.decision_sha256,
                },
                "geometry_only": {
                    "reply_sha256": geometry_only.reply_sha256,
                    "decision_sha256": geometry_only.decision_sha256,
                },
                "longest_32": {
                    "reply_sha256": longest.reply_sha256,
                    "decision_sha256": longest.decision_sha256,
                },
                "writer": {
                    "reply_sha256": writer.reply_sha256,
                    "decision_sha256": writer.decision_sha256,
                },
            },
            "elapsed_us": test_elapsed_us,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

struct AnchoredCandidateInputs<'a> {
    artifact: &'a [u8],
    train_samples: &'a [MemoryWeightSample],
    test_samples: &'a [MemoryWeightSample],
    train_identity: &'a CpuCorpusIdentity,
    test_identity: &'a CpuCorpusIdentity,
    train_tokens: &'a Path,
    train_mask: &'a Path,
    train_manifest: &'a Path,
    test_tokens: &'a Path,
    test_mask: &'a Path,
    test_manifest: &'a Path,
    artifact_path: &'a Path,
    cap: usize,
    seed: u64,
    cells: usize,
    allow_underpowered: bool,
}

fn run_anchored_candidate_union(inputs: AnchoredCandidateInputs<'_>) -> Result<(), String> {
    let training_started = Instant::now();
    let training = derive_anchored_candidate_rankers(
        inputs.artifact,
        inputs.train_samples,
        inputs.cap,
        inputs.seed,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let training_elapsed_us = training_started.elapsed().as_micros() as u64;
    let test_started = Instant::now();
    let trained = evaluate_anchored_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        AnchoredCandidateArm::Trained,
        &training.trained,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = evaluate_anchored_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        AnchoredCandidateArm::Shuffled,
        &training.shuffled,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let geometry_only = evaluate_anchored_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        AnchoredCandidateArm::GeometryOnly,
        &training.geometry_only,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let unanchored = evaluate_anchored_candidate_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        AnchoredCandidateArm::Unanchored,
        &training.unanchored,
        inputs.cells,
    )
    .map_err(|error| error.to_string())?;
    let longest = evaluate_copy_key_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        CopyKeyReplyArm::Longest32To1,
        inputs.seed,
    )
    .map_err(|error| error.to_string())?;
    let writer = evaluate_copy_key_replies(
        inputs.artifact,
        inputs.test_samples,
        inputs.cap,
        CopyKeyReplyArm::Writer,
        inputs.seed,
    )
    .map_err(|error| error.to_string())?;
    let test_elapsed_us = test_started.elapsed().as_micros() as u64;
    if !inputs.allow_underpowered
        && [
            trained.reading.reference_reachable,
            shuffled.reading.reference_reachable,
            geometry_only.reading.reference_reachable,
            unanchored.reading.reference_reachable,
            longest.reading.reference_reachable,
            writer.reading.reference_reachable,
        ]
        .into_iter()
        .any(|reachable| reachable < 200)
    {
        return Err("anchored candidate control has fewer than 200 reachable replies".to_owned());
    }
    let comparisons = serde_json::json!({
        "longest_32": compare_anchored_candidate_rows(
            &trained.rows,
            &longest.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "writer": compare_anchored_candidate_rows(
            &trained.rows,
            &writer.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "geometry_only": compare_anchored_candidate_rows(
            &trained.rows,
            &geometry_only.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "shuffled": compare_anchored_candidate_rows(
            &trained.rows,
            &shuffled.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
        "unanchored": compare_anchored_candidate_rows(
            &trained.rows,
            &unanchored.rows,
            CopyKeyReplyMetric::CorrectBodyPositions,
        )
        .map_err(|error| error.to_string())?,
    });
    let changes = serde_json::json!({
        "longest_32": copy_control_changes(&trained.rows, &trained.steps, &longest.steps),
        "writer": copy_control_changes(&trained.rows, &trained.steps, &writer.steps),
        "geometry_only": geometric_candidate_changes(
            &trained.rows,
            &trained.steps,
            &geometry_only.steps,
        ),
        "shuffled": geometric_candidate_changes(
            &trained.rows,
            &trained.steps,
            &shuffled.steps,
        ),
        "unanchored": geometric_candidate_changes(
            &trained.rows,
            &trained.steps,
            &unanchored.steps,
        ),
    });
    let control_changes_vs_longest = serde_json::json!({
        "geometry_only": copy_control_changes(
            &trained.rows,
            &geometry_only.steps,
            &longest.steps,
        ),
        "shuffled": copy_control_changes(
            &trained.rows,
            &shuffled.steps,
            &longest.steps,
        ),
        "unanchored": copy_control_changes(
            &trained.rows,
            &unanchored.steps,
            &longest.steps,
        ),
    });
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-anchored-expanded-candidate-ranker/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "bounded longest-32-anchored next-token candidate correction; no language or serving claim",
        "inputs": {
            "artifact": inputs.artifact_path.display().to_string(),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(inputs.artifact),
            "train_tokens": inputs.train_tokens.display().to_string(),
            "train_mask": inputs.train_mask.display().to_string(),
            "train_manifest": inputs.train_manifest.display().to_string(),
            "train_tokens_sha256": inputs.train_identity.tokens_sha256,
            "train_mask_sha256": inputs.train_identity.mask_sha256,
            "train_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(inputs.train_manifest)?),
            "train_split": inputs.train_identity.split,
            "test_tokens": inputs.test_tokens.display().to_string(),
            "test_mask": inputs.test_mask.display().to_string(),
            "test_manifest": inputs.test_manifest.display().to_string(),
            "test_tokens_sha256": inputs.test_identity.tokens_sha256,
            "test_mask_sha256": inputs.test_identity.mask_sha256,
            "test_manifest_sha256": uor_r4_discrete::corpus::sha256(&read(inputs.test_manifest)?),
            "test_split": inputs.test_identity.split,
            "tokenizer_sha256": inputs.test_identity.tokenizer_sha256,
        },
        "settings": {
            "cap": inputs.cap,
            "seed": inputs.seed,
            "candidate_set": training.candidate_set,
            "cells": training.cells,
            "writer_candidate_limit": training.writer_candidate_limit,
            "maximum_copy_depth": training.exact_depth,
            "baseline": "longest_32",
            "override_rule": "strict_positive_score",
            "served_writer_cells": 8,
        },
        "training": {
            "generated_rows": training.generated_rows,
            "supervised_groups": training.supervised_groups,
            "baseline_correct_groups": training.baseline_correct_groups,
            "baseline_wrong_groups": training.baseline_wrong_groups,
            "writer_candidate_rows": training.writer_candidate_rows,
            "min_writer_candidates_per_group": training.min_writer_candidates_per_group,
            "max_writer_candidates_per_group": training.max_writer_candidates_per_group,
            "target_values": training.target_values,
            "candidate_rows": training.candidate_rows,
            "shuffled_target_changes": training.shuffled_target_changes,
            "elapsed_us": training_elapsed_us,
        },
        "candidate_ranker": {
            "features": uor_r4_cpu_train::ANCHORED_CANDIDATE_FEATURE_NAMES,
            "epochs": uor_r4_cpu_train::ANCHORED_CANDIDATE_EPOCHS,
            "trained_weights": training.trained.weights,
            "trained_weights_sha256": training.trained.weights_sha256(),
            "shuffled_weights": training.shuffled.weights,
            "shuffled_weights_sha256": training.shuffled.weights_sha256(),
            "geometry_only_weights": training.geometry_only.weights,
            "unanchored_weights": training.unanchored.weights,
            "unanchored_weights_sha256": training.unanchored.weights_sha256(),
        },
        "test": {
            "trained": trained.reading,
            "shuffled": shuffled.reading,
            "geometry_only": geometry_only.reading,
            "unanchored": unanchored.reading,
            "longest_32": longest.reading,
            "writer": writer.reading,
            "comparisons": comparisons,
            "changes": changes,
            "control_changes_vs_longest": control_changes_vs_longest,
            "signatures": {
                "trained": {
                    "reply_sha256": trained.reply_sha256,
                    "decision_sha256": trained.decision_sha256,
                },
                "shuffled": {
                    "reply_sha256": shuffled.reply_sha256,
                    "decision_sha256": shuffled.decision_sha256,
                },
                "geometry_only": {
                    "reply_sha256": geometry_only.reply_sha256,
                    "decision_sha256": geometry_only.decision_sha256,
                },
                "unanchored": {
                    "reply_sha256": unanchored.reply_sha256,
                    "decision_sha256": unanchored.decision_sha256,
                },
                "longest_32": {
                    "reply_sha256": longest.reply_sha256,
                    "decision_sha256": longest.decision_sha256,
                },
                "writer": {
                    "reply_sha256": writer.reply_sha256,
                    "decision_sha256": writer.decision_sha256,
                },
            },
            "elapsed_us": test_elapsed_us,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn geometric_candidate_changes(
    rows: &[CopyKeyReplyRow],
    candidate: &[GeometricCandidateStep],
    control: &[GeometricCandidateStep],
) -> serde_json::Value {
    let reachable = reachable_rows(rows);
    let candidate = candidate
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let control = control
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    changed_tokens(&reachable, &candidate, &control)
}

fn copy_control_changes(
    rows: &[CopyKeyReplyRow],
    candidate: &[GeometricCandidateStep],
    control: &[CopyKeyReplyStep],
) -> serde_json::Value {
    let reachable = reachable_rows(rows);
    let candidate = candidate
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let control = control
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    changed_tokens(&reachable, &candidate, &control)
}

fn reachable_rows(rows: &[CopyKeyReplyRow]) -> BTreeSet<usize> {
    rows.iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect()
}

fn changed_tokens(
    reachable: &BTreeSet<usize>,
    candidate: &BTreeMap<(usize, usize), u32>,
    control: &BTreeMap<(usize, usize), u32>,
) -> serde_json::Value {
    let changed = candidate
        .iter()
        .filter_map(|(&(row, step), &token)| {
            if !reachable.contains(&row) {
                return None;
            }
            control
                .get(&(row, step))
                .filter(|&&control| control != token)
                .map(|_| (row, step))
        })
        .collect::<Vec<_>>();
    let changed_rows = changed
        .iter()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>()
        .len();
    serde_json::json!({
        "changed_steps": changed.len(),
        "changed_rows": changed_rows,
        "first_changed_step": changed.first().map(|(row, step)| serde_json::json!({
            "row": row,
            "generated_step": step,
        })),
        "non_vacuous": !changed.is_empty(),
    })
}

fn evaluate_panel(run: PanelRun<'_>) -> Result<serde_json::Value, String> {
    let PanelRun {
        artifact,
        samples,
        cap,
        seed,
        readouts,
        rankers,
        documents,
        allow_underpowered,
    } = run;
    let learned = evaluate_learned_copy_replies(
        artifact,
        samples,
        cap,
        LearnedCopyArm::LearnedRead32,
        readouts.learned,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = evaluate_learned_copy_replies(
        artifact,
        samples,
        cap,
        LearnedCopyArm::ShuffledRead32,
        readouts.shuffled,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let lookahead = evaluate_learned_copy_replies(
        artifact,
        samples,
        cap,
        LearnedCopyArm::LookaheadRead32,
        readouts.learned,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let lookahead_shuffled = evaluate_learned_copy_replies(
        artifact,
        samples,
        cap,
        LearnedCopyArm::ShuffledLookahead32,
        readouts.shuffled,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let ranked = evaluate_ranked_copy_replies(artifact, samples, cap, rankers.trained)
        .map_err(|error| error.to_string())?;
    let ranked_shuffled = evaluate_ranked_copy_replies(artifact, samples, cap, rankers.shuffled)
        .map_err(|error| error.to_string())?;
    let ranked_depth_only =
        evaluate_ranked_copy_replies(artifact, samples, cap, rankers.depth_only)
            .map_err(|error| error.to_string())?;
    if !allow_underpowered && ranked.reading.reference_reachable < 200 {
        return Err(format!(
            "ranked copy has {} reachable complete-reply examples",
            ranked.reading.reference_reachable
        ));
    }
    if !allow_underpowered && lookahead.reading.reference_reachable < 200 {
        return Err(format!(
            "learned read has {} reachable complete-reply examples",
            lookahead.reading.reference_reachable
        ));
    }

    let mut controls = BTreeMap::new();
    for arm in CopyKeyReplyArm::EVALUATION_ARMS {
        let reading = evaluate_copy_key_replies(artifact, samples, cap, arm, seed)
            .map_err(|error| error.to_string())?;
        if !allow_underpowered && reading.reading.reference_reachable < 200 {
            return Err(format!(
                "{} has {} reachable complete-reply examples",
                arm.name(),
                reading.reading.reference_reachable
            ));
        }
        controls.insert(arm.name().to_owned(), reading);
    }
    let longest = controls
        .get(CopyKeyReplyArm::Longest32To1.name())
        .ok_or_else(|| "longest-32 reading is missing".to_owned())?;

    let mut comparisons = BTreeMap::new();
    for (name, rows) in controls
        .iter()
        .map(|(name, reading)| (name.as_str(), reading.rows.as_slice()))
        .chain(std::iter::once((
            LearnedCopyArm::ShuffledRead32.name(),
            shuffled.rows.as_slice(),
        )))
        .chain(std::iter::once((
            LearnedCopyArm::LearnedRead32.name(),
            learned.rows.as_slice(),
        )))
        .chain(std::iter::once((
            LearnedCopyArm::ShuffledLookahead32.name(),
            lookahead_shuffled.rows.as_slice(),
        )))
    {
        let mut by_metric = BTreeMap::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let comparison = compare_copy_key_reply_rows(&lookahead.rows, rows, metric)
                .map_err(|error| error.to_string())?;
            by_metric.insert(metric_name(metric).to_owned(), comparison);
        }
        comparisons.insert(name.to_owned(), by_metric);
    }
    let mut ranker_comparisons = BTreeMap::new();
    for (name, rows) in controls
        .iter()
        .map(|(name, reading)| (name.as_str(), reading.rows.as_slice()))
        .chain(std::iter::once((
            "shuffled_ranker",
            ranked_shuffled.rows.as_slice(),
        )))
        .chain(std::iter::once((
            "depth_only_ranker",
            ranked_depth_only.rows.as_slice(),
        )))
        .chain(std::iter::once(("no_lookahead", learned.rows.as_slice())))
        .chain(std::iter::once(("lookahead", lookahead.rows.as_slice())))
    {
        let mut by_metric = BTreeMap::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let comparison = compare_copy_key_reply_rows(&ranked.rows, rows, metric)
                .map_err(|error| error.to_string())?;
            by_metric.insert(metric_name(metric).to_owned(), comparison);
        }
        ranker_comparisons.insert(name.to_owned(), by_metric);
    }

    let random = controls
        .get(CopyKeyReplyArm::RandomAvailableDepth32.name())
        .ok_or_else(|| "random-depth-32 reading is missing".to_owned())?;
    let longest_4 = controls
        .get(CopyKeyReplyArm::Longest4To1.name())
        .ok_or_else(|| "longest-4 reading is missing".to_owned())?;
    let random_4 = controls
        .get(CopyKeyReplyArm::RandomAvailableDepth.name())
        .ok_or_else(|| "random-depth-4 reading is missing".to_owned())?;
    let changes = serde_json::json!({
        "lookahead_from_longest_32":
            learned_changes(&lookahead.steps, &longest.steps, &longest.rows),
        "lookahead_from_no_lookahead":
            shuffled_changes(&lookahead.steps, &learned.steps, &learned.rows),
        "shuffled_lookahead_from_lookahead":
            shuffled_changes(&lookahead_shuffled.steps, &lookahead.steps, &lookahead.rows),
        "no_lookahead_from_longest_32":
            learned_changes(&learned.steps, &longest.steps, &longest.rows),
        "shuffled_no_lookahead_from_no_lookahead":
            shuffled_changes(&shuffled.steps, &learned.steps, &learned.rows),
        "random_32_from_longest_32": copy_changes(&longest.steps, &random.steps, &longest.rows),
        "random_4_from_longest_4": copy_changes(&longest_4.steps, &random_4.steps, &longest_4.rows),
        "ranker_from_longest_32":
            ranked_changes(&ranked.steps, &longest.steps, &longest.rows),
        "ranker_from_no_lookahead":
            ranked_learned_changes(&ranked.steps, &learned.steps, &learned.rows),
        "ranker_from_shuffled_ranker":
            ranked_vs_ranked_changes(&ranked.steps, &ranked_shuffled.steps, &ranked.rows),
        "ranker_from_depth_only":
            ranked_vs_ranked_changes(&ranked.steps, &ranked_depth_only.steps, &ranked.rows),
    });

    Ok(serde_json::json!({
        "examples": {
            "documents": documents,
            "evaluated": samples.len(),
            "reachable": lookahead.reading.reference_reachable,
        },
        "lookahead": lookahead,
        "lookahead_shuffled": lookahead_shuffled,
        "ranked": ranked,
        "ranked_shuffled": ranked_shuffled,
        "ranked_depth_only": ranked_depth_only,
        "learned": learned,
        "shuffled": shuffled,
        "controls": controls,
        "comparisons": comparisons,
        "ranker_comparisons": ranker_comparisons,
        "changes": changes,
    }))
}

fn load_samples(
    tokens_path: &Path,
    mask_path: &Path,
    manifest_path: &Path,
) -> Result<(Vec<MemoryWeightSample>, CpuCorpusIdentity, usize), String> {
    let tokens = read(tokens_path)?;
    let mask = read(mask_path)?;
    let manifest = read(manifest_path)?;
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    let corpus = CpuMaskedCorpus::from_bytes(
        &tokens,
        &mask,
        identity.vocab,
        identity.max_tokens,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let samples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?
        .iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt.clone(),
            reply: sample.reply.clone(),
        })
        .collect();
    Ok((samples, identity, corpus.document_count()))
}

fn learned_changes(
    learned: &[LearnedCopyStep],
    longest: &[CopyKeyReplyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let learned = learned
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let longest = longest
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &learned, &longest)
}

fn shuffled_changes(
    shuffled: &[LearnedCopyStep],
    learned: &[LearnedCopyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let shuffled = shuffled
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let learned = learned
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &shuffled, &learned)
}

fn copy_changes(
    left: &[CopyKeyReplyStep],
    right: &[CopyKeyReplyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let left = left
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let right = right
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &left, &right)
}

fn ranked_changes(
    ranked: &[RankedCopyStep],
    copy: &[CopyKeyReplyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let ranked = ranked
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let copy = copy
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &ranked, &copy)
}

fn ranked_learned_changes(
    ranked: &[RankedCopyStep],
    learned: &[LearnedCopyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let ranked = ranked
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let learned = learned
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &ranked, &learned)
}

fn ranked_vs_ranked_changes(
    left: &[RankedCopyStep],
    right: &[RankedCopyStep],
    rows: &[CopyKeyReplyRow],
) -> serde_json::Value {
    let reachable = rows
        .iter()
        .filter(|row| row.reachable)
        .map(|row| row.row)
        .collect::<BTreeSet<_>>();
    let left = left
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    let right = right
        .iter()
        .map(|step| ((step.row, step.generated_step), step.selected_token))
        .collect::<BTreeMap<_, _>>();
    change_value(&reachable, &left, &right)
}

fn change_value(
    reachable: &BTreeSet<usize>,
    left: &BTreeMap<(usize, usize), u32>,
    right: &BTreeMap<(usize, usize), u32>,
) -> serde_json::Value {
    let changed = left
        .iter()
        .filter_map(|(key, token)| {
            if !reachable.contains(&key.0) {
                return None;
            }
            right.get(key).filter(|right| *right != token).map(|_| *key)
        })
        .collect::<Vec<_>>();
    let changed_rows = changed
        .iter()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>()
        .len();
    serde_json::json!({
        "changed_steps": changed.len(),
        "changed_rows": changed_rows,
        "first_changed_step": changed.first().map(|(row, step)| serde_json::json!({
            "row": row,
            "generated_step": step,
        })),
        "non_vacuous": !changed.is_empty(),
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
    "usage: cpu-learned-copy-read TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
     TEST_TOKENS TEST_MASK TEST_MANIFEST ARTIFACT [--cap N] [--seed N] \
     [--ranker-trajectory longest32|reference] \
     [--ranker-candidates exact-copy|geometric-union|anchored-expanded] \
     [--candidate-cells N] [--candidate-calibration 8,12,16,24,32] \
     [--allow-underpowered]"
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
            "TEST_TOKENS",
            "TEST_MASK",
            "TEST_MANIFEST",
            "ARTIFACT",
            "--cap",
            "--seed",
            "--ranker-trajectory",
            "--ranker-candidates",
            "--candidate-cells",
            "--candidate-calibration",
        ] {
            assert!(usage.contains(word), "{usage}");
        }
    }

    #[test]
    fn candidate_cell_sweep_parser_is_strict() {
        assert_eq!(parse_cells("8,12,16,24,32").unwrap(), [8, 12, 16, 24, 32]);
        assert!(parse_cells("8,x,16").is_err());
        assert!(parse_cells("").is_err());
    }

    #[test]
    fn control_changes_are_bound_against_longest_not_only_the_trained_arm() {
        let row = CopyKeyReplyRow {
            row: 0,
            reachable: true,
            exact_reference: false,
            bos_complete: false,
            opening_correct: false,
            premature_bos: false,
            non_premature_bos: false,
            generated_tokens: 1,
            reference_tokens: 1,
            correct_body_positions: 0,
            copy_decisions: 0,
            fallback_decisions: 1,
            stop: "cap".to_owned(),
            reply_sha256: "00".repeat(32),
        };
        let step = |token| GeometricCandidateStep {
            row: 0,
            generated_step: 0,
            writer_token: 5,
            longest_token: 7,
            selected_token: token,
            selected_exact_depth: None,
            selected_writer_rank: None,
            selected_score: Some(1),
            candidate_count: 3,
            used_exact_copy: false,
        };
        let unchanged =
            geometric_candidate_changes(std::slice::from_ref(&row), &[step(7)], &[step(7)]);
        assert_eq!(unchanged["non_vacuous"], false);
        let changed = geometric_candidate_changes(&[row], &[step(9)], &[step(7)]);
        assert_eq!(changed["non_vacuous"], true);
    }
}
