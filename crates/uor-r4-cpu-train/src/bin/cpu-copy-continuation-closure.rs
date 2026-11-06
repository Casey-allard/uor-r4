use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    changed_closure_rows, compare_continuation_closure_rows, continuation_closure_changes,
    derive_continuation_closure, derive_full_sequence_suffix_content_memory,
    derive_reply_boundary_memory, evaluate_continuation_closure_replies,
    shuffle_continuation_closure, validate_corpus_manifest, ContinuationClosureArm,
    CopyKeyReplyMetric, CpuCorpusIdentity, CpuMaskedCorpus, MemoryWeightSample,
    TimedContinuationClosureReading,
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
        eprintln!("cpu-copy-continuation-closure: {error}");
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
    let closure = derive_continuation_closure(&train.samples).map_err(|e| e.to_string())?;
    let (shuffled_closure, changed_continuations) =
        shuffle_continuation_closure(&closure, config.seed).map_err(|e| e.to_string())?;
    let reply_boundary = derive_reply_boundary_memory(&train.samples).map_err(|e| e.to_string())?;

    let mut arms = BTreeMap::new();
    for arm in ContinuationClosureArm::EVALUATION_ARMS {
        let content_table = matches!(
            arm,
            ContinuationClosureArm::SourceClosure
                | ContinuationClosureArm::TokenwiseContent
                | ContinuationClosureArm::SourceClosureShuffled
                | ContinuationClosureArm::ContentNoBoundary
                | ContinuationClosureArm::ContentReferenceOracle
        )
        .then_some(&content);
        let closure_table = match arm {
            ContinuationClosureArm::SourceClosure | ContinuationClosureArm::ClosureOnly => {
                Some(&closure)
            }
            ContinuationClosureArm::SourceClosureShuffled => Some(&shuffled_closure),
            ContinuationClosureArm::TokenwiseContent
            | ContinuationClosureArm::ContentNoBoundary
            | ContinuationClosureArm::LocalReplyBoundary
            | ContinuationClosureArm::WriterBoundary
            | ContinuationClosureArm::ContentReferenceOracle => None,
        };
        let reading = evaluate_continuation_closure_replies(
            &artifact,
            &panel.samples,
            config.cap,
            arm,
            content_table,
            closure_table,
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

    let treatment = &arms[ContinuationClosureArm::SourceClosure.name()];
    let tokenwise = &arms[ContinuationClosureArm::TokenwiseContent.name()];
    let shuffled = &arms[ContinuationClosureArm::SourceClosureShuffled.name()];
    let closure_only = &arms[ContinuationClosureArm::ClosureOnly.name()];
    let no_boundary = &arms[ContinuationClosureArm::ContentNoBoundary.name()];
    let local_reply = &arms[ContinuationClosureArm::LocalReplyBoundary.name()];
    let writer = &arms[ContinuationClosureArm::WriterBoundary.name()];
    let oracle = &arms[ContinuationClosureArm::ContentReferenceOracle.name()];

    let comparisons = compare_arms(
        treatment,
        tokenwise,
        shuffled,
        closure_only,
        no_boundary,
        local_reply,
        writer,
    )?;
    let changes_vs_tokenwise = continuation_closure_changes(treatment, tokenwise);
    let changes_vs_shuffled = continuation_closure_changes(treatment, shuffled);
    let changes_vs_closure_only = continuation_closure_changes(treatment, closure_only);
    let changes_vs_no_boundary = continuation_closure_changes(treatment, no_boundary);
    let changes_vs_local = continuation_closure_changes(treatment, local_reply);
    let closure_output_changes = changed_closure_rows(treatment, &[tokenwise, local_reply]);
    let control_non_vacuous = changed_continuations > 0
        && closure_output_changes >= 100
        && changes_vs_tokenwise > 0
        && changes_vs_shuffled > 0
        && changes_vs_closure_only > 0
        && changes_vs_no_boundary > 0
        && changes_vs_local > 0;
    let required_exact = oracle.reading.exact_reference.div_ceil(2).max(5);
    let exact_pass = comparisons["tokenwise_content"]["exact_reference"]["passes"]
        .as_bool()
        .unwrap_or(false)
        && comparisons["closure_only"]["exact_reference"]["passes"]
            .as_bool()
            .unwrap_or(false)
        && comparisons["source_closure_shuffled"]["exact_reference"]["passes"]
            .as_bool()
            .unwrap_or(false);
    let body_nonfall_vs_controls = treatment.reading.correct_body_positions
        >= no_boundary.reading.correct_body_positions
        && treatment.reading.correct_body_positions >= local_reply.reading.correct_body_positions;
    let body_nonfall_vs_closure_only =
        treatment.reading.correct_body_positions >= closure_only.reading.correct_body_positions;
    let body_nonfall = body_nonfall_vs_controls && body_nonfall_vs_closure_only;
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
        "schema": "uor-r4.cpu-copy-continuation-closure/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "confident whole-reply continuation closure over the fixed full-sequence content selector; development evidence only",
        "inputs": {
            "train": panel_input(&train),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "closure_depth": [
                uor_r4_cpu_train::CONTINUATION_CLOSURE_MIN_DEPTH,
                uor_r4_cpu_train::CONTINUATION_CLOSURE_MAX_DEPTH
            ],
            "content_depth": uor_r4_cpu_train::SUFFIX_MEMORY_MAX_DEPTH,
            "minimum_observations": uor_r4_cpu_train::CONTINUATION_CLOSURE_MIN_OBSERVATIONS,
            "confidence": "strict majority of complete continuation sequences",
            "arms": ContinuationClosureArm::EVALUATION_ARMS
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>(),
        },
        "examples": {
            "train": train.samples.len(),
            "panel": panel.samples.len(),
            "reachable": treatment.reading.reference_reachable,
        },
        "tables": {
            "content": content.census(),
            "closure": closure.census(),
            "shuffled_closure": shuffled_closure.census(),
            "changed_continuations": changed_continuations,
        },
        "changes": {
            "vs_tokenwise": changes_vs_tokenwise,
            "vs_shuffled": changes_vs_shuffled,
            "vs_closure_only": changes_vs_closure_only,
            "vs_no_boundary": changes_vs_no_boundary,
            "vs_local": changes_vs_local,
            "closure_rows_vs_tokenwise_or_local": closure_output_changes,
        },
        "comparisons": comparisons,
        "bar": {
            "content_oracle_exact": oracle.reading.exact_reference,
            "required_exact": required_exact,
            "exact_floor_pass": treatment.reading.exact_reference >= required_exact,
            "exact_comparisons_pass": exact_pass,
            "body_nonfall_vs_controls": body_nonfall_vs_controls,
            "body_nonfall_vs_closure_only": body_nonfall_vs_closure_only,
            "body_nonfall": body_nonfall,
            "premature_bos_at_most_4": premature_ok,
            "closure_decisions": treatment.reading.closure_decisions,
            "closure_changes": treatment.reading.closure_changes,
            "closure_output_changes_vs_tokenwise_or_local": closure_output_changes,
            "closure_output_changes_at_least_100": closure_output_changes >= 100,
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
    candidate: &TimedContinuationClosureReading,
    tokenwise: &TimedContinuationClosureReading,
    shuffled: &TimedContinuationClosureReading,
    closure_only: &TimedContinuationClosureReading,
    no_boundary: &TimedContinuationClosureReading,
    local_reply: &TimedContinuationClosureReading,
    writer: &TimedContinuationClosureReading,
) -> Result<serde_json::Value, String> {
    let mut comparisons = serde_json::Map::new();
    for (name, control) in [
        ("tokenwise_content", tokenwise),
        ("source_closure_shuffled", shuffled),
        ("closure_only", closure_only),
        ("content_no_boundary", no_boundary),
        ("local_reply_boundary", local_reply),
        ("writer_boundary", writer),
    ] {
        let mut metrics = serde_json::Map::new();
        for metric in [
            CopyKeyReplyMetric::ExactReference,
            CopyKeyReplyMetric::NonPrematureBos,
            CopyKeyReplyMetric::OpeningCorrect,
            CopyKeyReplyMetric::CorrectBodyPositions,
        ] {
            let reading = compare_continuation_closure_rows(&candidate.rows, &control.rows, metric)
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
    let mut seed = 202u64;
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
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-copy-continuation-closure TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT --expected-train-tokens HEX --expected-train-mask HEX --expected-train-manifest HEX --expected-panel-tokens HEX --expected-panel-mask HEX --expected-panel-manifest HEX --expected-artifact HEX [--cap N] [--seed N]".to_owned()
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
