use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_suffix_fallback_rows, derive_reply_boundary_memory, derive_suffix_content_memory,
    evaluate_suffix_fallback_replies, shuffle_suffix_memory, suffix_fallback_changes,
    validate_corpus_manifest, CopyKeyReplyMetric, CpuCorpusIdentity, CpuMaskedCorpus,
    MemoryWeightSample, SuffixFallbackArm, TimedSuffixFallbackReading,
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
        eprintln!("cpu-copy-suffix-fallback: {error}");
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

    let suffix = derive_suffix_content_memory(&train.samples).map_err(|error| error.to_string())?;
    let suffix_census = suffix.census();
    if suffix_census.bos_transitions != 0 {
        return Err("content-only suffix table contains BOS targets".to_owned());
    }
    let (shuffled, changed_targets) =
        shuffle_suffix_memory(&suffix, config.seed).map_err(|error| error.to_string())?;
    let boundary =
        derive_reply_boundary_memory(&train.samples).map_err(|error| error.to_string())?;

    let mut arms = BTreeMap::new();
    for arm in SuffixFallbackArm::EVALUATION_ARMS {
        let table = match arm {
            SuffixFallbackArm::LocalThenContent32
            | SuffixFallbackArm::LocalThenContentDepth1
            | SuffixFallbackArm::Content32Only => Some(&suffix),
            SuffixFallbackArm::LocalThenShuffledContent32 => Some(&shuffled),
            SuffixFallbackArm::LocalLongest32Boundary
            | SuffixFallbackArm::WriterBoundary
            | SuffixFallbackArm::ReferenceLengthOracle => None,
        };
        let reading = evaluate_suffix_fallback_replies(
            &artifact,
            &panel.samples,
            config.cap,
            arm,
            table,
            &boundary,
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

    let treatment = &arms[SuffixFallbackArm::LocalThenContent32.name()];
    let depth_one = &arms[SuffixFallbackArm::LocalThenContentDepth1.name()];
    let shuffled_arm = &arms[SuffixFallbackArm::LocalThenShuffledContent32.name()];
    let content_only = &arms[SuffixFallbackArm::Content32Only.name()];
    let local = &arms[SuffixFallbackArm::LocalLongest32Boundary.name()];
    let writer = &arms[SuffixFallbackArm::WriterBoundary.name()];
    let oracle = &arms[SuffixFallbackArm::ReferenceLengthOracle.name()];

    let changes_from_local = suffix_fallback_changes(treatment, local);
    let changes_from_shuffled = suffix_fallback_changes(treatment, shuffled_arm);
    let comparisons = compare_arms(
        treatment,
        local,
        depth_one,
        shuffled_arm,
        content_only,
        writer,
    )?;
    let controls = [
        "local_longest_32_boundary",
        "local_then_content_depth_1",
        "local_then_shuffled_content_32",
        "content_32_only",
        "writer_boundary",
    ];
    let body_pass = controls.iter().all(|arm| {
        comparisons[*arm]["correct_body_positions"]["passes"]
            .as_bool()
            .unwrap_or(false)
    });
    let exact_pass = comparisons["local_longest_32_boundary"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false);
    let control_non_vacuous = changed_targets > 0
        && changes_from_local.changed_steps > 0
        && changes_from_shuffled.changed_steps > 0
        && treatment.reading.suffix_decisions >= 100;
    let premature_ok = treatment.reading.premature_bos <= 4;
    let exact_power = oracle.reading.exact_reference >= 10;
    let outcome = if !control_non_vacuous {
        "CONTROL_FAULT"
    } else if !body_pass || !premature_ok {
        "REJECT"
    } else if !exact_power {
        "BODY_POSITIVE_INSUFFICIENT_COMPLETION"
    } else if exact_pass {
        "KEEP"
    } else {
        "BODY_POSITIVE_EXACT_REJECT"
    };

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-suffix-fallback/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "confident content-suffix fallback with local-copy priority; no language, fresh-wording or serving claim",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "maximum_suffix_depth": uor_r4_cpu_train::SUFFIX_MEMORY_MAX_DEPTH,
            "minimum_observations": uor_r4_cpu_train::SUFFIX_FALLBACK_MIN_OBSERVATIONS,
            "confidence": "strict_majority",
            "local_priority": "longest_32_to_1",
            "arms": SuffixFallbackArm::EVALUATION_ARMS
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>(),
        },
        "examples": {
            "train": train.samples.len(),
            "panel": panel.samples.len(),
            "reachable": treatment.reading.reference_reachable,
        },
        "table": {
            "content": suffix_census,
            "shuffled": shuffled.census(),
            "changed_targets": changed_targets,
        },
        "changes": {
            "from_local": changes_from_local,
            "from_shuffled": changes_from_shuffled,
        },
        "comparisons": comparisons,
        "bar": {
            "body_comparisons": body_pass,
            "exact_oracle": oracle.reading.exact_reference,
            "exact_power": exact_power,
            "exact_comparison_vs_local": exact_pass,
            "premature_bos_at_most_4": premature_ok,
            "control_non_vacuous": control_non_vacuous,
            "changed_targets": changed_targets,
            "suffix_decisions": treatment.reading.suffix_decisions,
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
    candidate: &TimedSuffixFallbackReading,
    local: &TimedSuffixFallbackReading,
    depth_one: &TimedSuffixFallbackReading,
    shuffled: &TimedSuffixFallbackReading,
    content_only: &TimedSuffixFallbackReading,
    writer: &TimedSuffixFallbackReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("local_longest_32_boundary", local),
        ("local_then_content_depth_1", depth_one),
        ("local_then_shuffled_content_32", shuffled),
        ("content_32_only", content_only),
        ("writer_boundary", writer),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_suffix_fallback_rows(&candidate.rows, &control.rows, metric)
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
    let mut seed = 196u64;
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
    "usage: cpu-copy-suffix-fallback TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
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
