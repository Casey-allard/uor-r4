use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    boundary_stop_changes, compare_boundary_rows, derive_reply_boundary_memory,
    evaluate_boundary_replies, evaluate_copy_key_replies, shuffle_reply_boundary_memory,
    validate_corpus_manifest, CopyKeyReplyArm, CopyKeyReplyMetric, CpuCorpusIdentity,
    CpuMaskedCorpus, MemoryWeightSample, ReplyBoundaryArm, TimedReplyBoundaryReading,
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
        eprintln!("cpu-copy-boundary-reply: {error}");
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

    let boundary =
        derive_reply_boundary_memory(&train.samples).map_err(|error| error.to_string())?;
    let (shuffled, changed_suffix_keys) =
        shuffle_reply_boundary_memory(&boundary, config.seed).map_err(|error| error.to_string())?;
    let mut arms = BTreeMap::new();
    for arm in ReplyBoundaryArm::EVALUATION_ARMS {
        let table = match arm {
            ReplyBoundaryArm::Longest32TrueBoundary => Some(&boundary),
            ReplyBoundaryArm::Longest32ShuffledBoundary => Some(&shuffled),
            ReplyBoundaryArm::Longest32NoBoundary | ReplyBoundaryArm::ReferenceLengthOracle => None,
        };
        let reading = evaluate_boundary_replies(
            &artifact,
            &panel.samples,
            config.cap,
            arm,
            table,
            config.seed,
        )
        .map_err(|error| error.to_string())?;
        if reading.reading.reference_reachable < 200 {
            return Err(format!(
                "{} has {} reachable replies, below 200",
                arm.name(),
                reading.reading.reference_reachable
            ));
        }
        arms.insert(arm.name().to_owned(), reading);
    }

    let no_boundary = &arms[ReplyBoundaryArm::Longest32NoBoundary.name()];
    let true_boundary = &arms[ReplyBoundaryArm::Longest32TrueBoundary.name()];
    let shuffled_boundary = &arms[ReplyBoundaryArm::Longest32ShuffledBoundary.name()];
    let oracle = &arms[ReplyBoundaryArm::ReferenceLengthOracle.name()];
    let existing_no_boundary = evaluate_copy_key_replies(
        &artifact,
        &panel.samples,
        config.cap,
        CopyKeyReplyArm::Longest32To1,
        config.seed,
    )
    .map_err(|error| error.to_string())?;
    let no_boundary_matches_existing = existing_no_boundary.rows == no_boundary.rows;
    if !no_boundary_matches_existing {
        return Err("no-boundary arm differs from the existing Longest32To1 evaluator".to_owned());
    }
    let comparisons = compare_arms(true_boundary, no_boundary, shuffled_boundary)?;
    let stop_changes = boundary_stop_changes(true_boundary, shuffled_boundary);
    let required_exact = oracle.reading.exact_reference.div_ceil(2).max(5);
    let exact_comparisons = comparisons["vs_no_boundary"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false)
        && comparisons["vs_shuffled_boundary"]["exact_reference"]["passes"]
            .as_bool()
            .unwrap_or(false);
    let body_nonfall =
        true_boundary.reading.correct_body_positions >= no_boundary.reading.correct_body_positions;
    let premature_ok = true_boundary.reading.premature_bos <= 4;
    let control_non_vacuous = changed_suffix_keys > 0 && stop_changes > 0;
    let outcome = if oracle.reading.exact_reference < 10 {
        "INSUFFICIENT_POWER"
    } else if !control_non_vacuous {
        "CONTROL_FAULT"
    } else if true_boundary.reading.exact_reference >= required_exact
        && exact_comparisons
        && body_nonfall
        && premature_ok
    {
        "KEEP"
    } else {
        "REJECT"
    };

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-boundary-reply/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "exact-memory reply-boundary screen; no language, fresh-wording or serving claim",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "boundary_depth": [
                uor_r4_cpu_train::REPLY_BOUNDARY_MIN_DEPTH,
                uor_r4_cpu_train::REPLY_BOUNDARY_MAX_DEPTH
            ],
            "arms": ReplyBoundaryArm::EVALUATION_ARMS
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>(),
            "copy_arm": "longest_32_to_1",
        },
        "examples": {
            "train": train.samples.len(),
            "panel": panel.samples.len(),
            "reachable": no_boundary.reading.reference_reachable,
        },
        "boundary": {
            "true": boundary.census(),
            "shuffled": shuffled.census(),
            "changed_suffix_keys": changed_suffix_keys,
            "stop_changes": stop_changes,
        },
        "comparisons": comparisons,
        "bar": {
            "oracle_exact": oracle.reading.exact_reference,
            "required_true_exact": required_exact,
            "true_exact": true_boundary.reading.exact_reference,
            "exact_comparisons": exact_comparisons,
            "body_nonfall": body_nonfall,
            "premature_bos_at_most_4": premature_ok,
            "control_non_vacuous": control_non_vacuous,
            "no_boundary_matches_existing": no_boundary_matches_existing,
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
    candidate: &TimedReplyBoundaryReading,
    no_boundary: &TimedReplyBoundaryReading,
    shuffled: &TimedReplyBoundaryReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("vs_no_boundary", no_boundary),
        ("vs_shuffled_boundary", shuffled),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_boundary_rows(&candidate.rows, &control.rows, metric)
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
        return Err(format!(
            "{}: tokens sha256 {} != expected {}",
            paths.tokens.display(),
            identity.tokens_sha256,
            paths.expected_tokens_sha256
        ));
    }
    if identity.mask_sha256 != paths.expected_mask_sha256 {
        return Err(format!(
            "{}: mask sha256 {} != expected {}",
            paths.mask.display(),
            identity.mask_sha256,
            paths.expected_mask_sha256
        ));
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
    let mut seed = 191u64;
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
            "--expected-train-tokens" => {
                set_expected(&mut expected, "train_tokens", value, &flag)?;
            }
            "--expected-train-mask" => {
                set_expected(&mut expected, "train_mask", value, &flag)?;
            }
            "--expected-train-manifest" => {
                set_expected(&mut expected, "train_manifest", value, &flag)?;
            }
            "--expected-panel-tokens" => {
                set_expected(&mut expected, "panel_tokens", value, &flag)?;
            }
            "--expected-panel-mask" => {
                set_expected(&mut expected, "panel_mask", value, &flag)?;
            }
            "--expected-panel-manifest" => {
                set_expected(&mut expected, "panel_manifest", value, &flag)?;
            }
            "--expected-artifact" => {
                set_expected(&mut expected, "artifact", value, &flag)?;
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if positional.len() != 7 {
        return Err(usage());
    }
    if !(1..=64).contains(&cap) {
        return Err("cap must be in 1..=64".to_owned());
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
        let value = expected.get(key).ok_or_else(|| {
            format!(
                "--expected-{} is required\n{}",
                key.replace('_', "-"),
                usage()
            )
        })?;
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
    "usage: cpu-copy-boundary-reply TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
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
