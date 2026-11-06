use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_prompt_retrieval_rows, derive_prompt_retrieval_memory, evaluate_prompt_retrieval,
    normalized_reference, shuffle_prompt_retrieval_replies, validate_corpus_manifest,
    CopyKeyReplyMetric, CpuCorpusIdentity, CpuMaskedCorpus, MemoryWeightSample, PromptRetrievalArm,
    TimedPromptRetrievalReading,
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
        eprintln!("cpu-prompt-retrieval: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let train = load_panel(&config.train)?;
    let panel = load_panel(&config.panel)?;
    let artifact = fs::read(&config.artifact)
        .map_err(|error| format!("{}: {error}", config.artifact.display()))?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    if artifact_sha256 != config.expected_artifact_sha256 {
        return Err(format!(
            "artifact sha256 {artifact_sha256} != expected {}",
            config.expected_artifact_sha256
        ));
    }

    let memory = derive_prompt_retrieval_memory(&artifact, &train.samples, config.cap)
        .map_err(|e| e.to_string())?;
    let (shuffled, changed_reply_associations) =
        shuffle_prompt_retrieval_replies(&memory, config.seed).map_err(|e| e.to_string())?;
    let memory_census = memory.census();
    if memory_census.unique_unigram_sets < 300
        || memory_census.unique_replies < 250
        || changed_reply_associations < 100
    {
        return Err(format!(
            "prompt retrieval training gate failed: {} unigram sets, {} replies, {} changed associations",
            memory_census.unique_unigram_sets,
            memory_census.unique_replies,
            changed_reply_associations
        ));
    }
    let exact_train_prompts = panel
        .samples
        .iter()
        .filter(|sample| {
            train
                .samples
                .iter()
                .any(|train_sample| train_sample.prompt == sample.prompt)
        })
        .count();
    let panel_prompts_without_exact_train_match = panel.samples.len() - exact_train_prompts;
    if panel_prompts_without_exact_train_match < 200 {
        return Err(format!(
            "prompt retrieval panel has only {panel_prompts_without_exact_train_match} non-exact prompt matches"
        ));
    }

    let mut arms = BTreeMap::new();
    for arm in PromptRetrievalArm::EVALUATION_ARMS {
        let reading = evaluate_prompt_retrieval(
            &artifact,
            &memory,
            &shuffled,
            &panel.samples,
            config.cap,
            arm,
        )
        .map_err(|e| e.to_string())?;
        if reading.reading.reference_reachable < 200 {
            return Err(format!(
                "{} has {} reachable replies, below 200",
                arm.name(),
                reading.reading.reference_reachable
            ));
        }
        arms.insert(arm.name().to_owned(), reading);
    }

    let treatment = &arms[PromptRetrievalArm::UnigramJaccardOrdinalTie.name()];
    let geometry_tie = &arms[PromptRetrievalArm::UnigramJaccardGeometryTie.name()];
    let bigram = &arms[PromptRetrievalArm::BigramJaccardGeometryTie.name()];
    let shuffled_arm = &arms[PromptRetrievalArm::UnigramJaccardShuffledReplies.name()];
    let geometry = &arms[PromptRetrievalArm::QueryGeometryNearest.name()];
    let fixed = &arms[PromptRetrievalArm::FixedOrdinal.name()];
    let oracle = &arms[PromptRetrievalArm::ReferenceMembershipOracle.name()];

    let comparisons = compare_arms(treatment, bigram, shuffled_arm, geometry, fixed)?;
    let exact_comparison_pass = [
        "bigram_jaccard_geometry_tie",
        "unigram_jaccard_shuffled_replies",
        "fixed_ordinal",
    ]
    .iter()
    .all(|name| {
        comparisons[*name]["exact_reference"]["passes"]
            .as_bool()
            .unwrap_or(false)
    });
    let geometry_tie_non_superior = geometry_tie.reading.exact_reference
        <= treatment.reading.exact_reference
        && geometry_tie.reading.correct_body_positions <= treatment.reading.correct_body_positions;
    let required_exact = oracle.reading.exact_reference.div_ceil(8).max(25);
    let exact_floor = treatment.reading.exact_reference >= required_exact;
    let premature_zero = treatment.reading.premature_bos == 0;
    let body_nonfall = treatment.reading.correct_body_positions
        >= bigram.reading.correct_body_positions
        && treatment.reading.correct_body_positions >= geometry.reading.correct_body_positions;
    let emitted_reachable =
        treatment.reading.emitted_replies >= treatment.reading.reference_reachable;
    let control_non_vacuous = changed_reply_associations > 0
        && geometry_tie.reading.geometry_ties > 0
        && treatment.decision_sha256 != bigram.decision_sha256
        && treatment.decision_sha256 != shuffled_arm.decision_sha256
        && treatment.decision_sha256 != geometry_tie.decision_sha256
        && treatment.decision_sha256 != geometry.decision_sha256
        && treatment.decision_sha256 != fixed.decision_sha256;
    let outcome = if !control_non_vacuous || !emitted_reachable {
        "CONTROL_FAULT"
    } else if oracle.reading.exact_reference < 10 {
        "INSUFFICIENT_POWER"
    } else if exact_floor
        && exact_comparison_pass
        && geometry_tie_non_superior
        && premature_zero
        && body_nonfall
    {
        "KEEP"
    } else {
        "REJECT"
    };

    let unique_train_prompts = train
        .samples
        .iter()
        .map(|sample| sample.prompt.as_slice())
        .collect::<BTreeSet<_>>()
        .len();
    let train_replies = train
        .samples
        .iter()
        .map(|sample| normalized_reference(&sample.reply))
        .collect::<BTreeSet<_>>();
    let seen_reply_roots = panel
        .samples
        .iter()
        .filter(|sample| train_replies.contains(normalized_reference(&sample.reply)))
        .count();

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-prompt-retrieval/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "integer token-set Jaccard retrieval with Q12 writer-geometry tie resolution; development evidence only",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "primary_similarity": "set Jaccard over prompt token identities",
            "geometry_tie": "minimum Q12 writer-query L1, then lower train ordinal",
            "arms": PromptRetrievalArm::EVALUATION_ARMS
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>(),
        },
        "examples": {
            "train": train.samples.len(),
            "panel": panel.samples.len(),
            "reachable": treatment.reading.reference_reachable,
            "unique_train_prompts": unique_train_prompts,
            "panel_exact_train_prompts": exact_train_prompts,
            "panel_prompts_without_exact_train_match": panel_prompts_without_exact_train_match,
            "panel_replies_seen_in_train": seen_reply_roots,
        },
        "tables": {
            "memory": memory_census,
            "changed_reply_associations": changed_reply_associations,
        },
        "comparisons": comparisons,
        "bar": {
            "reference_membership_exact": oracle.reading.exact_reference,
            "required_exact": required_exact,
            "exact_floor_pass": exact_floor,
            "exact_comparisons_pass": exact_comparison_pass,
            "geometry_tie_non_superior": geometry_tie_non_superior,
            "premature_bos_zero": premature_zero,
            "body_nonfall": body_nonfall,
            "all_reachable_emitted": emitted_reachable,
            "control_non_vacuous": control_non_vacuous,
            "outcome": outcome,
        },
        "arms": arms,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn compare_arms(
    candidate: &TimedPromptRetrievalReading,
    bigram: &TimedPromptRetrievalReading,
    shuffled: &TimedPromptRetrievalReading,
    geometry: &TimedPromptRetrievalReading,
    fixed: &TimedPromptRetrievalReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("bigram_jaccard_geometry_tie", bigram),
        ("unigram_jaccard_shuffled_replies", shuffled),
        ("query_geometry_nearest", geometry),
        ("fixed_ordinal", fixed),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_prompt_retrieval_rows(&candidate.rows, &control.rows, metric)
                .map_err(|error| error.to_string())?;
            metrics.insert(
                metric_name(metric).to_owned(),
                serde_json::to_value(reading).map_err(|error| error.to_string())?,
            );
        }
        comparisons.insert(name.to_owned(), serde_json::Value::Object(metrics));
    }
    Ok(serde_json::Value::Object(comparisons))
}

fn metric_name(metric: CopyKeyReplyMetric) -> &'static str {
    match metric {
        CopyKeyReplyMetric::ExactReference => "exact_reference",
        CopyKeyReplyMetric::NonPrematureBos => "non_premature_bos",
        CopyKeyReplyMetric::OpeningCorrect => "opening_correct",
        CopyKeyReplyMetric::CorrectBodyPositions => "correct_body_positions",
    }
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
    if identity.tokens_sha256 != paths.expected_tokens_sha256 {
        return Err(format!("{}: tokens sha256 differs", paths.tokens.display()));
    }
    if identity.mask_sha256 != paths.expected_mask_sha256 {
        return Err(format!("{}: mask sha256 differs", paths.mask.display()));
    }
    if manifest_sha256 != paths.expected_manifest_sha256 {
        return Err(format!(
            "{}: manifest sha256 differs",
            paths.manifest.display()
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

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
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
    let mut seed = 204u64;
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
        return Err(usage().to_owned());
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
        if !expected.contains_key(key) {
            return Err(format!("missing --expected-{key}\n{}", usage()));
        }
    }
    Ok(Config {
        train: PanelPaths {
            tokens: positional[0].clone(),
            mask: positional[1].clone(),
            manifest: positional[2].clone(),
            expected_tokens_sha256: expected["train_tokens"].clone(),
            expected_mask_sha256: expected["train_mask"].clone(),
            expected_manifest_sha256: expected["train_manifest"].clone(),
        },
        panel: PanelPaths {
            tokens: positional[3].clone(),
            mask: positional[4].clone(),
            manifest: positional[5].clone(),
            expected_tokens_sha256: expected["panel_tokens"].clone(),
            expected_mask_sha256: expected["panel_mask"].clone(),
            expected_manifest_sha256: expected["panel_manifest"].clone(),
        },
        artifact: positional[6].clone(),
        expected_artifact_sha256: expected["artifact"].clone(),
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
        return Err(format!("duplicate {flag}"));
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: cpu-prompt-retrieval TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT --cap N --seed N --expected-train-tokens SHA --expected-train-mask SHA --expected-train-manifest SHA --expected-panel-tokens SHA --expected-panel-mask SHA --expected-panel-manifest SHA --expected-artifact SHA"
}
