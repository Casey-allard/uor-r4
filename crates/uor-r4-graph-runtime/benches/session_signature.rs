//! Held-out session-signature A/B harness for issue #247.
//!
//! The graph runtime deliberately keeps the session signature out of ROUT
//! fallback. This benchmark measures the shipped bias-only lane against the
//! same held-out context without a session signature, using the corpus's
//! story prefix to provide a deterministic session history. It is an
//! empirical harness, not a quality gate: the command reports the observed
//! deltas and never changes artifact or routing semantics.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_compiler::{
    induction, observation,
    probability_calibration::{
        EntropyBucket, entropy_bucket, entropy_quartiles, sampled_teacher_bits_per_token,
    },
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;
use uor_r4_router::session_signature_from_tokens;

const DEFAULT_ROOT: &str = ".uor-models/compiled/smollm2-135m-instruct";
const DEFAULT_SAMPLE: usize = 256;
const WINDOW: usize = compiler::WINDOW;

fn load_probability_metadata(
    probability_dir: Option<&Path>,
    expected_records: usize,
) -> Result<Option<Vec<observation::ProbabilityMetadata>>, String> {
    let Some(dir) = probability_dir else {
        return Ok(None);
    };
    let metadata = observation::merge_probability_metadata(dir).map_err(|error| {
        format!(
            "cannot read probability metadata {}: {error}",
            dir.display()
        )
    })?;
    if metadata.len() != expected_records {
        return Err(format!(
            "probability metadata has {} rows but corpus has {expected_records} records",
            metadata.len()
        ));
    }
    Ok(Some(metadata))
}

fn resolve(path: &Path) -> Result<(PathBuf, Vec<u8>), String> {
    let candidates = if path.is_absolute() {
        vec![path.to_owned()]
    } else {
        vec![path.to_owned(), Path::new("../..").join(path)]
    };
    for candidate in candidates {
        if let Ok(bytes) = fs::read(&candidate) {
            return Ok((candidate, bytes));
        }
    }
    Err(format!("cannot read {}", path.display()))
}

fn story_history(corpus: &compiler::Corpus, position: usize) -> Vec<u32> {
    let mut start = position;
    while start > 0 && corpus.story[start - 1] == corpus.story[position] {
        start -= 1;
    }
    (start..=position)
        .map(|index| corpus.input[index])
        .collect()
}

fn context_window(corpus: &compiler::Corpus, position: usize) -> Vec<u32> {
    let history = story_history(corpus, position);
    let start = history.len().saturating_sub(WINDOW);
    history[start..].to_vec()
}

fn alternate_history(history: &[u32]) -> Vec<u32> {
    let prefix_len = history.len().saturating_sub(WINDOW);
    let mut alternate = history.to_vec();
    alternate[..prefix_len].reverse();
    alternate
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1).filter(|arg| arg != "--bench");
    let root = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_ROOT.to_owned()));
    let sample_target = args
        .next()
        .map(|value| value.parse::<usize>().map_err(|_| "invalid sample size"))
        .transpose()?
        .unwrap_or(DEFAULT_SAMPLE);
    let probability_dir = match args.next() {
        None => None,
        Some(flag) if flag == "--probability-dir" => {
            Some(PathBuf::from(args.next().ok_or_else(|| {
                "missing value for --probability-dir".to_owned()
            })?))
        }
        Some(path) => Some(PathBuf::from(path)),
    };
    if sample_target == 0 || args.next().is_some() {
        return Err("usage: session_signature [BUNDLE_ROOT] [SAMPLE] [PROBABILITY_DIR]".to_owned());
    }

    let graph = root.join("graph").join("score.r4g1");
    let artifacts_path = root.join("tless_artifacts.bin");
    let corpus_meta = root.join("corpus.meta");
    let corpus_recs = root.join("corpus.records");
    let (graph_path, graph_bytes) = resolve(&graph)?;
    let (_, artifact_bytes) = resolve(&artifacts_path)?;
    let (meta_path, _) = resolve(&corpus_meta)?;
    let (recs_path, _) = resolve(&corpus_recs)?;

    let runtime = R4G1Runtime::parse(&graph_bytes)
        .map_err(|error| format!("invalid graph {}: {error:?}", graph_path.display()))?;
    let artifacts = compiler::parse_artifacts(&artifact_bytes)
        .ok_or_else(|| format!("invalid teacher artifacts {}", artifacts_path.display()))?;
    let meta = meta_path
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let recs = recs_path
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(meta, recs)
        .ok_or_else(|| "corpus metadata/records do not parse".to_owned())?;
    let (_, held_out) = induction::split_positions(&corpus);
    if held_out.is_empty() {
        return Err("corpus held-out partition is empty".to_owned());
    }

    let stride = (held_out.len() / sample_target).max(1);
    let positions: Vec<usize> = held_out.iter().copied().step_by(stride).collect();
    let probability_metadata = load_probability_metadata(probability_dir.as_deref(), corpus.n)?;
    let entropy_quartiles = entropy_quartiles(&positions, probability_metadata.as_deref());
    let mut entropy_buckets = [EntropyBucket::default(); 4];
    let rotations = compiler::derive_rotations();
    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];

    let mut context_hits = 0u64;
    let mut session_hits = 0u64;
    let mut context_teacher_hits = 0u64;
    let mut session_teacher_hits = 0u64;
    let mut session_token_changes = 0u64;
    let mut alternate_token_changes = 0u64;
    let mut score_changes = 0u64;

    for &position in &positions {
        let context = context_window(&corpus, position);
        let history = story_history(&corpus, position);
        let alternate = alternate_history(&history);
        let bundle = runtime::bundle_window_plain(&artifacts, &rotations, &context);
        let context_signature = runtime::sig_plain(&artifacts, &bundle);
        let session_signature = session_signature_from_tokens(&history);
        let alternate_signature = session_signature_from_tokens(&alternate);

        let (context_token, context_score) = runtime.predict_distribution_with_signature_lanes(
            &context,
            Some(&context_signature),
            None,
            &mut node_scores,
        );
        let (session_token, session_score) = runtime.predict_distribution_with_signature_lanes(
            &context,
            Some(&context_signature),
            Some(&session_signature),
            &mut node_scores,
        );
        let (alternate_token, _) = runtime.predict_distribution_with_signature_lanes(
            &context,
            Some(&context_signature),
            Some(&alternate_signature),
            &mut node_scores,
        );

        context_hits += u64::from(context_token == corpus.next[position]);
        session_hits += u64::from(session_token == corpus.next[position]);
        context_teacher_hits += u64::from(context_token == corpus.t_argmax[position]);
        session_teacher_hits += u64::from(session_token == corpus.t_argmax[position]);
        session_token_changes += u64::from(session_token != context_token);
        alternate_token_changes += u64::from(alternate_token != session_token);
        score_changes += u64::from(session_score != context_score);

        if let (Some(metadata), Some(quartiles)) =
            (probability_metadata.as_ref(), entropy_quartiles.as_ref())
        {
            let bucket = entropy_bucket(metadata[position].entropy_bits, quartiles);
            entropy_buckets[bucket].record(
                metadata[position].entropy_bits,
                context_token,
                session_token,
                corpus.next[position],
                corpus.t_argmax[position],
            );
        }
    }

    let count = positions.len() as f64;
    println!("bundle={}", root.display());
    println!("held_out_positions={}", held_out.len());
    println!("sample_positions={}", positions.len());
    println!(
        "context_top1={}/{} ({:.4})",
        context_hits,
        positions.len(),
        context_hits as f64 / count
    );
    println!(
        "session_top1={}/{} ({:.4})",
        session_hits,
        positions.len(),
        session_hits as f64 / count
    );
    println!(
        "context_teacher_argmax={}/{} ({:.4})",
        context_teacher_hits,
        positions.len(),
        context_teacher_hits as f64 / count
    );
    println!(
        "session_teacher_argmax={}/{} ({:.4})",
        session_teacher_hits,
        positions.len(),
        session_teacher_hits as f64 / count
    );
    println!(
        "session_token_changes={}/{} ({:.4})",
        session_token_changes,
        positions.len(),
        session_token_changes as f64 / count
    );
    println!(
        "alternate_token_changes={}/{} ({:.4})",
        alternate_token_changes,
        positions.len(),
        alternate_token_changes as f64 / count
    );
    println!(
        "session_score_changes={}/{} ({:.4})",
        score_changes,
        positions.len(),
        score_changes as f64 / count
    );
    match (probability_metadata.as_ref(), entropy_quartiles) {
        (Some(metadata), Some(quartiles)) => {
            println!("probability_metadata=available");
            println!(
                "teacher_bits_per_token_all={:.6}",
                observation::message_bits_per_token(metadata).ok_or_else(|| {
                    "probability metadata contains invalid log probabilities".to_owned()
                })?
            );
            println!(
                "teacher_bits_per_token_sample={:.6}",
                sampled_teacher_bits_per_token(&positions, metadata).ok_or_else(|| {
                    "sampled probability metadata contains invalid log probabilities".to_owned()
                })?
            );
            println!(
                "entropy_quartiles_bits={:.6},{:.6},{:.6}",
                quartiles[0], quartiles[1], quartiles[2]
            );
            for (index, bucket) in entropy_buckets.iter().enumerate() {
                println!(
                    "entropy_bucket_{index}=samples:{} mean_entropy_bits:{:.6} context_top1:{}/{} session_top1:{}/{} context_teacher_argmax:{}/{} session_teacher_argmax:{}/{} session_token_changes:{}/{} session_corrections:{}/{} session_regressions:{}/{}",
                    bucket.samples,
                    bucket.mean_entropy_bits(),
                    bucket.context_hits,
                    bucket.samples,
                    bucket.session_hits,
                    bucket.samples,
                    bucket.context_teacher_hits,
                    bucket.samples,
                    bucket.session_teacher_hits,
                    bucket.samples,
                    bucket.session_token_changes,
                    bucket.samples,
                    bucket.session_corrections,
                    bucket.samples,
                    bucket.session_regressions,
                    bucket.samples,
                );
            }
        }
        (None, _) => println!(
            "probability_metadata=unavailable (pass an observation directory containing manifest.json and *.prob sidecars)"
        ),
        (Some(_), None) => unreachable!("non-empty samples produce entropy quartiles"),
    }
    Ok(())
}
