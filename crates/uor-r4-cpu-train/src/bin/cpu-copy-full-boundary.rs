use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_full_boundary_rows, derive_full_sequence_boundary,
    derive_full_sequence_suffix_content_memory, derive_reply_boundary_memory,
    evaluate_full_boundary_replies, full_boundary_stop_changes, shuffle_full_boundary,
    validate_corpus_manifest, CopyKeyReplyMetric, CpuCorpusIdentity, CpuMaskedCorpus,
    FullBoundaryArm, MemoryWeightSample, TimedFullBoundaryReading,
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
        eprintln!("cpu-copy-full-boundary: {error}");
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

    let content =
        derive_full_sequence_suffix_content_memory(&train.samples).map_err(|e| e.to_string())?;
    let full_boundary = derive_full_sequence_boundary(&train.samples).map_err(|e| e.to_string())?;
    let (shuffled_boundary, changed_boundary_terminals) =
        shuffle_full_boundary(&full_boundary, config.seed).map_err(|e| e.to_string())?;
    let reply_boundary = derive_reply_boundary_memory(&train.samples).map_err(|e| e.to_string())?;

    let mut arms = BTreeMap::new();
    for arm in FullBoundaryArm::EVALUATION_ARMS {
        let content_table = match arm {
            FullBoundaryArm::ContentFullBoundary
            | FullBoundaryArm::ContentReplyBoundary
            | FullBoundaryArm::ContentShuffledFullBoundary
            | FullBoundaryArm::ContentNoBoundary
            | FullBoundaryArm::ContentReferenceOracle => Some(&content),
            FullBoundaryArm::LocalFullBoundary
            | FullBoundaryArm::LocalReplyBoundary
            | FullBoundaryArm::WriterFullBoundary => None,
        };
        let full_table = match arm {
            FullBoundaryArm::ContentFullBoundary
            | FullBoundaryArm::LocalFullBoundary
            | FullBoundaryArm::WriterFullBoundary => Some(&full_boundary),
            FullBoundaryArm::ContentShuffledFullBoundary => Some(&shuffled_boundary),
            FullBoundaryArm::ContentReplyBoundary
            | FullBoundaryArm::ContentNoBoundary
            | FullBoundaryArm::LocalReplyBoundary
            | FullBoundaryArm::ContentReferenceOracle => None,
        };
        let reading = evaluate_full_boundary_replies(
            &artifact,
            &panel.samples,
            config.cap,
            arm,
            content_table,
            full_table,
            &reply_boundary,
            config.seed,
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

    let treatment = &arms[FullBoundaryArm::ContentFullBoundary.name()];
    let reply = &arms[FullBoundaryArm::ContentReplyBoundary.name()];
    let shuffled = &arms[FullBoundaryArm::ContentShuffledFullBoundary.name()];
    let no_boundary = &arms[FullBoundaryArm::ContentNoBoundary.name()];
    let local_full = &arms[FullBoundaryArm::LocalFullBoundary.name()];
    let local_reply = &arms[FullBoundaryArm::LocalReplyBoundary.name()];
    let writer = &arms[FullBoundaryArm::WriterFullBoundary.name()];
    let oracle = &arms[FullBoundaryArm::ContentReferenceOracle.name()];

    let comparisons = compare_arms(
        treatment,
        reply,
        shuffled,
        no_boundary,
        local_full,
        local_reply,
        writer,
    )?;
    let stop_changes_vs_reply = full_boundary_stop_changes(treatment, reply);
    let stop_changes_vs_shuffled = full_boundary_stop_changes(treatment, shuffled);
    let control_non_vacuous =
        changed_boundary_terminals > 0 && stop_changes_vs_reply > 0 && stop_changes_vs_shuffled > 0;
    let required_exact = oracle.reading.exact_reference.div_ceil(2).max(5);
    let exact_pass = comparisons["content_reply_boundary"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false)
        && comparisons["content_shuffled_full_boundary"]["exact_reference"]["passes"]
            .as_bool()
            .unwrap_or(false);
    let body_nonfall = treatment.reading.correct_body_positions
        >= no_boundary.reading.correct_body_positions
        && treatment.reading.correct_body_positions >= local_reply.reading.correct_body_positions;
    let premature_ok = treatment.reading.premature_bos <= 4;
    let exact_power = oracle.reading.exact_reference >= 10;
    let outcome = if !control_non_vacuous {
        "CONTROL_FAULT"
    } else if !exact_power {
        "INSUFFICIENT_POWER"
    } else if premature_ok
        && treatment.reading.exact_reference >= required_exact
        && exact_pass
        && body_nonfall
    {
        "KEEP"
    } else {
        "REJECT"
    };

    let output = serde_json::json!({
        "schema": "uor-r4.cpu-copy-full-boundary/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "full prompt-plus-reply boundary memory with fixed full-sequence content arbitration; development evidence only",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "boundary_depth": [uor_r4_cpu_train::FULL_BOUNDARY_MIN_DEPTH, uor_r4_cpu_train::FULL_BOUNDARY_MAX_DEPTH],
            "content_depth": uor_r4_cpu_train::SUFFIX_MEMORY_MAX_DEPTH,
            "minimum_observations": uor_r4_cpu_train::FULL_BOUNDARY_MIN_OBSERVATIONS,
            "arms": FullBoundaryArm::EVALUATION_ARMS.iter().map(|arm| arm.name()).collect::<Vec<_>>(),
        },
        "examples": {
            "train": train.samples.len(),
            "panel": panel.samples.len(),
            "reachable": treatment.reading.reference_reachable,
        },
        "tables": {
            "content": content.census(),
            "full_boundary": full_boundary.census(),
            "shuffled_boundary": shuffled_boundary.census(),
            "changed_boundary_terminals": changed_boundary_terminals,
        },
        "changes": {
            "stop_vs_reply": stop_changes_vs_reply,
            "stop_vs_shuffled": stop_changes_vs_shuffled,
        },
        "comparisons": comparisons,
        "bar": {
            "content_oracle_exact": oracle.reading.exact_reference,
            "required_exact": required_exact,
            "exact_floor_pass": treatment.reading.exact_reference >= required_exact,
            "exact_comparisons_pass": exact_pass,
            "body_nonfall": body_nonfall,
            "premature_bos_at_most_4": premature_ok,
            "control_non_vacuous": control_non_vacuous,
            "outcome": outcome,
        },
        "arms": arms,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn compare_arms(
    candidate: &TimedFullBoundaryReading,
    reply: &TimedFullBoundaryReading,
    shuffled: &TimedFullBoundaryReading,
    no_boundary: &TimedFullBoundaryReading,
    local_full: &TimedFullBoundaryReading,
    local_reply: &TimedFullBoundaryReading,
    writer: &TimedFullBoundaryReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("content_reply_boundary", reply),
        ("content_shuffled_full_boundary", shuffled),
        ("content_no_boundary", no_boundary),
        ("local_full_boundary", local_full),
        ("local_reply_boundary", local_reply),
        ("writer_full_boundary", writer),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_full_boundary_rows(&candidate.rows, &control.rows, metric)
                .map_err(|e| e.to_string())?;
            metrics.insert(
                metric_name(metric).to_owned(),
                serde_json::to_value(reading).map_err(|e| e.to_string())?,
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
        .map_err(|e| e.to_string())?;
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
    .map_err(|e| e.to_string())?;
    let samples = corpus
        .conversation_examples()
        .map_err(|e| e.to_string())?
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
    let mut seed = 200u64;
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
    let paths = |start: usize, prefix: &str| PanelPaths {
        tokens: positional[start].clone(),
        mask: positional[start + 1].clone(),
        manifest: positional[start + 2].clone(),
        expected_tokens_sha256: expected[&format!("{prefix}_tokens")].to_ascii_lowercase(),
        expected_mask_sha256: expected[&format!("{prefix}_mask")].to_ascii_lowercase(),
        expected_manifest_sha256: expected[&format!("{prefix}_manifest")].to_ascii_lowercase(),
    };
    Ok(Config {
        train: paths(0, "train"),
        panel: paths(3, "panel"),
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
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn usage() -> String {
    "usage: cpu-copy-full-boundary TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT --expected-train-tokens HEX --expected-train-mask HEX --expected-train-manifest HEX --expected-panel-tokens HEX --expected-panel-mask HEX --expected-panel-manifest HEX --expected-artifact HEX [--cap N] [--seed N]".to_owned()
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
