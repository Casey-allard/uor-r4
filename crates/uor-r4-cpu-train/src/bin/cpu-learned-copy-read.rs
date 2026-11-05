use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    compare_copy_key_reply_rows, content_mapping_sha256, derive_copy_candidate_rankers,
    derive_learned_copy_readout, evaluate_copy_key_replies, evaluate_learned_copy_replies,
    evaluate_ranked_copy_replies, shuffle_content_targets, validate_corpus_manifest,
    CopyCandidateRanker, CopyKeyReplyArm, CopyKeyReplyMetric, CopyKeyReplyRow, CopyKeyReplyStep,
    CpuCorpusIdentity, CpuMaskedCorpus, LearnedCopyArm, LearnedCopyStep, MemoryWeightSample,
    RankedCopyStep,
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
        return Err("learned-copy reply cap must be in 1..=64".to_owned());
    }

    let artifact_bytes = read(&artifact)?;
    let (train_samples, train_identity, train_documents) =
        load_samples(&train_tokens, &train_mask, &train_manifest)?;
    let (test_samples, test_identity, test_documents) =
        load_samples(&test_tokens, &test_mask, &test_manifest)?;
    if train_identity.tokenizer_sha256 != test_identity.tokenizer_sha256
        || train_identity.vocab != test_identity.vocab
        || train_identity.bos != test_identity.bos
    {
        return Err("learned-copy train and test corpora differ".to_owned());
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
    let ranker_training = derive_copy_candidate_rankers(&artifact_bytes, &train_samples, cap, seed)
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
        ] {
            assert!(usage.contains(word), "{usage}");
        }
    }
}
