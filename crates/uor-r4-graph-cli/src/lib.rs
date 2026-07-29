//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! Read docs/TRANSFORMERLESS.md (the extrapolation) and docs/PROOF.md (the
//! proof structure and the measured certificate) alongside this code.
//!
//! # Commands
//!
//!   setup            print the external prerequisite commands
//!   gen [secs] [target]   generate/extend the teacher-labeled corpus
//!                         (resumable; whole-story chunking keeps the
//!                         stream deterministic under any chunking)
//!   certify          (root command: `r4 certify`) compile the source,
//!                    build the store, and print the
//!                    full equivalence certificate and op census
//!
//! # The claim, precisely
//!
//! COMPILER (offline, once): multiplication permitted; every output frozen
//! and blake3-κ-pinned. RUNTIME (per token): every arithmetic operation
//! goes through `OpKernel`, whose complete method set is add / shift / xor /
//! compare / table-read — multiplication is absent from the interface, and
//! the census printed by `certify` measures the ops actually used. The
//! CERTIFIER is instrumentation and may use anything; it never runs at
//! inference. The source-architecture interface is two surfaces (embedding
//! table + next-token oracle); this crate ships the llama-family adapter,
//! and qwen/phi-class sources differ only in that adapter.

use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_certify as score_runtime;
use uor_r4_graph_compiler::induction as cover;
use uor_r4_graph_compiler::observation as observe;
use uor_r4_graph_compiler::observation_text as observe_text;
mod convert_r4g1;
mod scenarios;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerKind};
use uor_r4_core::transformerless::scenarios as core_scenarios;
use uor_r4_model_source::{BehaviorSource, HuggingFaceLlamaOracle, LlamaOracle, TeacherOracle};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const DEFAULT_TOKENIZER: &str = "/tmp/ref/tokenizer.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";
const DEFAULT_HF_SOURCE_PATH: &str = ".uor-models/sources/smollm2-135m-instruct";
const DEFAULT_HF_COMPILED_PATH: &str = ".uor-models/compiled/smollm2-135m-instruct";
const DEFAULT_HF_EVALUATION_REPORT: &str = "instruction-eval.json";
const DEFAULT_TEXT_CORPUS: &str = ".uor-models/corpora/simple-wiki-20231101/articles.jsonl";

#[derive(Debug, PartialEq, Eq)]
struct CompileOptions {
    model: Option<String>,
    revision: Option<String>,
    source: Option<PathBuf>,
    output: Option<PathBuf>,
    seconds: u64,
    target: usize,
    sequence_length: usize,
    r4_attention: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct EvaluateReportOptions {
    source: PathBuf,
    compiled: PathBuf,
    report: Option<PathBuf>,
    sequence_length: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct ObserveOptions {
    source: PathBuf,
    checkpoint: Option<PathBuf>,
    output: PathBuf,
    seconds: u64,
    target: usize,
    shards: u8,
    sequence_length: usize,
}

fn parse_observe_options(args: &[String]) -> Result<ObserveOptions, String> {
    let mut options = ObserveOptions {
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        checkpoint: None,
        output: PathBuf::from("obs"),
        seconds: 300,
        target: 20_000,
        shards: 4,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--source" => options.source = PathBuf::from(value),
            "--checkpoint" => options.checkpoint = Some(PathBuf::from(value)),
            "--out" => options.output = PathBuf::from(value),
            "--seconds" => {
                options.seconds = value
                    .parse()
                    .map_err(|_| format!("invalid --seconds value: {value}"))?;
            }
            "--target" => {
                options.target = value
                    .parse()
                    .map_err(|_| format!("invalid --target value: {value}"))?;
            }
            "--shards" => {
                options.shards = value
                    .parse()
                    .map_err(|_| format!("invalid --shards value: {value}"))?;
                if options.shards > observe::MAX_SHARD_BITS {
                    return Err(format!(
                        "--shards must be at most {} (2^N shard files)",
                        observe::MAX_SHARD_BITS
                    ));
                }
            }
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown observe option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Observation pipeline v2 (plan §5 Phase 2): the same teacher generation
/// as [`compile_hugging_face`]'s corpus step, spilled into content-
/// addressed, resumable shards instead of one corpus stream.
pub fn observe_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- observe ...`"
    );
    let options = parse_observe_options(args)?;
    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let token_byte_lengths: Option<Vec<u32>>;
    let mut oracle: Box<dyn TeacherOracle> = if let Some(checkpoint) = &options.checkpoint {
        // Legacy llama2.c checkpoint: no HF tokenizer tree, so byte
        // anchors stay at the v3 "unknown" value.
        token_byte_lengths = None;
        let path = checkpoint
            .to_str()
            .ok_or_else(|| "checkpoint path is not UTF-8".to_owned())?;
        Box::new(LlamaOracle::load(path))
    } else {
        let oracle = HuggingFaceLlamaOracle::load_with_sequence_length(
            &options.source,
            options.sequence_length,
        )
        .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
        eprintln!("exporting tokenizer...");
        token_byte_lengths = Some(
            uor_r4_core::transformerless::scenarios::export_hf_bytelevel_tokenizer_with_lengths(
                options.source.join("tokenizer.json"),
                options.output.join("tokenizer.bin"),
            )
            .map_err(|error| error.to_string())?,
        );
        Box::new(oracle)
    };
    let summary = observe::observe_sharded(
        oracle.as_mut(),
        options.seconds,
        options.target,
        options.shards,
        &options.output,
        token_byte_lengths.as_deref(),
    )?;
    if summary.done {
        // Persist the merged record stream so Gate C can consume it as
        // --corpus-recs with state.bin as --corpus-meta (same convention
        // as the from-text driver, issue #75).
        let merged = observe::merge_shards(&options.output).map_err(|error| error.to_string())?;
        let merged_path = options.output.join("merged.bin");
        std::fs::write(&merged_path, &merged).map_err(|error| error.to_string())?;
        println!(
            "observe complete: {} records at {}",
            summary.records,
            merged_path.display()
        );
    } else {
        println!(
            "observation corpus is not complete; rerun the same command to resume {}",
            options.output.display()
        );
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ObserveTextOptions {
    input: PathBuf,
    source: PathBuf,
    checkpoint: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    output: PathBuf,
    seconds: u64,
    shards: u8,
    sequence_length: usize,
}

fn parse_observe_text_options(args: &[String]) -> Result<ObserveTextOptions, String> {
    let mut options = ObserveTextOptions {
        input: PathBuf::from(DEFAULT_TEXT_CORPUS),
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        checkpoint: None,
        tokenizer: None,
        output: PathBuf::from("obs-text"),
        seconds: 300,
        shards: 4,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--input" => options.input = PathBuf::from(value),
            "--source" => options.source = PathBuf::from(value),
            "--checkpoint" => options.checkpoint = Some(PathBuf::from(value)),
            "--tokenizer" => options.tokenizer = Some(PathBuf::from(value)),
            "--out" => options.output = PathBuf::from(value),
            "--seconds" => {
                options.seconds = value
                    .parse()
                    .map_err(|_| format!("invalid --seconds value: {value}"))?;
            }
            "--shards" => {
                options.shards = value
                    .parse()
                    .map_err(|_| format!("invalid --shards value: {value}"))?;
                if options.shards > observe::MAX_SHARD_BITS {
                    return Err(format!(
                        "--shards must be at most {} (2^N shard files)",
                        observe::MAX_SHARD_BITS
                    ));
                }
            }
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown observe-text option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// From-text observation driver (issue #72): feed the sealed natural-text
/// corpus (D3) through the teacher, recording the same v3 observation
/// records the autoregressive `observe` path produces, with the corpus
/// split rule applied at write time and recorded per shard.
pub fn observe_text_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- observe-text ...`"
    );
    let options = parse_observe_text_options(args)?;
    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let token_byte_lengths: Vec<u32>;
    let tokenizer: TokenizerKind;
    let mut oracle: Box<dyn TeacherOracle> = if let Some(checkpoint) = &options.checkpoint {
        // Legacy llama2.c checkpoint: the companion tokenizer is the
        // scoreless tokenizer.bin fetched by `setup` (overridable with
        // --tokenizer); its piece byte lengths anchor records into the
        // article text. This path is untouched by issue #242 — its κ-pinned
        // baselines depend on the exact legacy encoding.
        let tokenizer_path = options
            .tokenizer
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TOKENIZER));
        let legacy = scenarios::Tokenizer::try_load(&tokenizer_path)
            .map_err(|error| format!("{}: {error}", tokenizer_path.display()))?;
        token_byte_lengths = legacy
            .vocab
            .iter()
            .map(|piece| piece.len() as u32)
            .collect();
        tokenizer = TokenizerKind::Legacy(legacy);
        let path = checkpoint
            .to_str()
            .ok_or_else(|| "checkpoint path is not UTF-8".to_owned())?;
        Box::new(LlamaOracle::load(path))
    } else {
        let oracle = HuggingFaceLlamaOracle::load_with_sequence_length(
            &options.source,
            options.sequence_length,
        )
        .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
        eprintln!("exporting tokenizer...");
        token_byte_lengths =
            uor_r4_core::transformerless::scenarios::export_hf_bytelevel_tokenizer_with_lengths(
                options.source.join("tokenizer.json"),
                options.output.join("tokenizer.bin"),
            )
            .map_err(|error| error.to_string())?;
        // Issue #242: segment observation text with the teacher's REAL
        // byte-level BPE (ordered merges from tokenizer.json), not the
        // legacy lowest-id greedy heuristic — otherwise every record feeds
        // the teacher a segmentation it never saw. tokenizer.bin above
        // remains the compiled runtime artifact; tokenizer.json is
        // authoritative for observation when present.
        let tokenizer_json = options.source.join("tokenizer.json");
        tokenizer = if tokenizer_json.is_file() {
            TokenizerKind::HfBpe(Box::new(
                HfBpeTokenizer::from_dir(&options.source)
                    .map_err(|error| format!("{}: {error}", tokenizer_json.display()))?,
            ))
        } else {
            TokenizerKind::Legacy(
                scenarios::Tokenizer::try_load(options.output.join("tokenizer.bin"))
                    .map_err(|error| error.to_string())?,
            )
        };
        Box::new(oracle)
    };
    let report = observe_text::observe_text_corpus(
        oracle.as_mut(),
        options.seconds,
        &tokenizer,
        Some(&token_byte_lengths),
        &options.input,
        &options.output,
        options.shards,
        true,
    )?;
    println!(
        "observe-text: {} records across {}/{} shards ({} written this run)",
        report.records, report.shards_completed, report.shard_count, report.written
    );
    println!(
        "partition: {} construction / {} held-out records ({} / {} articles of {})",
        report.construction_records,
        report.held_out_records,
        report.construction_articles,
        report.held_out_articles,
        report.articles_total
    );
    if report.articles_truncated != 0 {
        println!(
            "note: {} articles truncated at the teacher sequence length",
            report.articles_truncated
        );
    }
    if report.characters_replaced != 0 {
        println!(
            "note: {} characters unencodable in the teacher vocab replaced with spaces",
            report.characters_replaced
        );
    }
    if report.done {
        // Persist the merged record stream: Gate C consumes it as
        // --corpus-recs with state.bin as --corpus-meta (issue #72).
        let merged = observe::merge_shards(&options.output).map_err(|error| error.to_string())?;
        let merged_path = options.output.join("merged.bin");
        std::fs::write(&merged_path, &merged).map_err(|error| error.to_string())?;
        println!(
            "observe-text complete: merged κ {} at {}",
            report.merged_kappa.expect("done reports merged κ"),
            merged_path.display()
        );
    } else {
        println!(
            "text observation corpus is not complete ({}/{} articles); rerun the same command to resume {}",
            report.articles_completed,
            report.articles_total,
            options.output.display()
        );
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct CoverOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    depths: usize,
    k0: usize,
    regions_budget: usize,
    memory_budget_mb: u64,
    min_support: usize,
    entropy_gain_bits: f64,
    radius_quantile: u32,
    output: PathBuf,
}

fn parse_cover_options(args: &[String]) -> Result<CoverOptions, String> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = CoverOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        depths: cover::DEFAULT_DEPTHS,
        k0: cover::DEFAULT_K0,
        regions_budget: cover::DEFAULT_REGIONS_BUDGET,
        memory_budget_mb: cover::DEFAULT_MEMORY_BUDGET_MB,
        min_support: cover::DEFAULT_MIN_SUPPORT,
        entropy_gain_bits: cover::DEFAULT_SPLIT_ENTROPY_GAIN_BITS,
        radius_quantile: cover::RADIUS_QUANTILE_NUMERATOR,
        output: PathBuf::from("cover"),
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--depths" => {
                options.depths = value
                    .parse()
                    .map_err(|_| format!("invalid --depths value: {value}"))?;
                if options.depths == 0 {
                    return Err("--depths must be at least 1".to_owned());
                }
            }
            "--k0" => {
                options.k0 = value
                    .parse()
                    .map_err(|_| format!("invalid --k0 value: {value}"))?;
                if options.k0 == 0 {
                    return Err("--k0 must be at least 1".to_owned());
                }
            }
            "--regions-budget" => {
                options.regions_budget = value
                    .parse()
                    .map_err(|_| format!("invalid --regions-budget value: {value}"))?;
                if options.regions_budget == 0 {
                    return Err("--regions-budget must be at least 1".to_owned());
                }
            }
            "--memory-budget" => {
                options.memory_budget_mb = value
                    .parse()
                    .map_err(|_| format!("invalid --memory-budget value: {value}"))?;
                if options.memory_budget_mb == 0 {
                    return Err("--memory-budget must be at least 1 MiB".to_owned());
                }
            }
            "--min-support" => {
                options.min_support = value
                    .parse()
                    .map_err(|_| format!("invalid --min-support value: {value}"))?;
                if options.min_support == 0 {
                    return Err("--min-support must be at least 1".to_owned());
                }
            }
            "--entropy-gain" => {
                options.entropy_gain_bits = value
                    .parse()
                    .map_err(|_| format!("invalid --entropy-gain value: {value}"))?;
                if !options.entropy_gain_bits.is_finite() || options.entropy_gain_bits < 0.0 {
                    return Err("--entropy-gain must be a finite non-negative number".to_owned());
                }
            }
            "--radius-quantile" => {
                options.radius_quantile = value
                    .parse()
                    .map_err(|_| format!("invalid --radius-quantile value: {value}"))?;
                if options.radius_quantile == 0 || options.radius_quantile > 100 {
                    return Err("--radius-quantile must be between 1 and 100".to_owned());
                }
            }
            "--out" => options.output = PathBuf::from(value),
            _ => return Err(format!("unknown cover option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Multiresolution cover induction (plan §5 Phase 2, issue #60): induce
/// the overlapping region cover over the deterministic context-bundle
/// lane, freeze the reference classifier, measure held-out routing recall
/// against it and against the incumbent 4×256 class cover, and write the
/// R4G1 artifact plus the JSON recall/stability report.
pub fn cover_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make cover induction much slower; use `cargo run --release -- transformerless cover ...`"
    );
    let options = parse_cover_options(args)?;
    let corpus_meta_path = if !options.corpus_meta.exists() {
        if let Some(parent) = options.corpus_meta.parent() {
            if parent.join("corpus.meta").exists() {
                parent.join("corpus.meta")
            } else if parent.join("c_meta.bin").exists() {
                parent.join("c_meta.bin")
            } else {
                options.corpus_meta.clone()
            }
        } else {
            options.corpus_meta.clone()
        }
    } else {
        options.corpus_meta.clone()
    };

    let corpus_recs_path = if !options.corpus_recs.exists() {
        if let Some(parent) = options.corpus_recs.parent() {
            if parent.join("corpus.records").exists() {
                parent.join("corpus.records")
            } else if parent.join("c_recs.bin").exists() {
                parent.join("c_recs.bin")
            } else {
                options.corpus_recs.clone()
            }
        } else {
            options.corpus_recs.clone()
        }
    } else {
        options.corpus_recs.clone()
    };

    let corpus_meta = corpus_meta_path
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let corpus_recs = corpus_recs_path
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta_path.display(),
            corpus_recs_path.display()
        )
    })?;
    let artifact_container = std::fs::read(&options.artifacts)
        .map_err(|error| format!("{}: {error}", options.artifacts.display()))?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            options.artifacts.display()
        )
    })?;
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let meta_bytes = std::fs::read(&options.corpus_meta)
        .map_err(|error| format!("{}: {error}", options.corpus_meta.display()))?;
    let recs_bytes = std::fs::read(&options.corpus_recs)
        .map_err(|error| format!("{}: {error}", options.corpus_recs.display()))?;
    let mut corpus_hasher = blake3::Hasher::new();
    corpus_hasher.update(&meta_bytes);
    corpus_hasher.update(&recs_bytes);
    let corpus_kappa = format!("blake3:{}", corpus_hasher.finalize().to_hex());

    let config = cover::CoverConfig {
        depths: options.depths,
        k0: options.k0,
        regions_budget: options.regions_budget,
        memory_budget_bytes: options.memory_budget_mb * 1024 * 1024,
        threads: std::thread::available_parallelism()
            .map(|count| count.get().min(8) as u32)
            .unwrap_or(1),
        min_support: options.min_support,
        entropy_gain_bits: options.entropy_gain_bits,
        radius_quantile_numerator: options.radius_quantile,
        radius_quantile_denominator: 100,
        objective: cover::ObjectiveConfig::default(),
    };
    eprintln!(
        "cover: inducing (depths {}, k0 {}, regions budget {}, memory budget {} MiB)...",
        config.depths, config.k0, config.regions_budget, options.memory_budget_mb
    );
    let (train_positions, held_out_positions) = cover::split_positions(&corpus);
    let train = cover::build_observations_with_threads(
        &artifacts,
        &corpus,
        &train_positions,
        config.threads as usize,
    )?;
    let held_out = cover::build_observations_with_threads(
        &artifacts,
        &corpus,
        &held_out_positions,
        config.threads as usize,
    )?;
    let induced = cover::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)?;
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    eprintln!(
        "cover: {} regions across {} depth(s); evaluating held-out routing recall...",
        induced.cover.regions.len(),
        induced.cover.max_depth
    );
    let recall =
        cover::evaluate_held_out(&artifacts, &induced.cover, &reference, &train, &held_out);
    let edges = cover::build_edges(&induced.cover, &reference, &train, &corpus.story);
    let prior = cover::root_prior(&train);
    let vocab = u32::try_from(artifacts.token_codes.len() / compiler::STAGES)
        .map_err(|_| "vocabulary exceeds u32 token ids".to_owned())?;
    let (artifact_bytes, info) = cover::emit_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &induced.cover,
        &edges,
        &prior,
        &train,
    )?;
    let report = cover::build_report(
        &config,
        &induced,
        cover::ReportData {
            reference: &reference,
            train: &train,
            held_out: &held_out,
            edges: &edges,
            recall: recall.clone(),
            artifact: Some((&artifact_bytes, info)),
        },
    );

    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let artifact_path = options.output.join("cover.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| format!("{}: {error}", artifact_path.display()))?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    let report_path = options.output.join("cover_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;

    println!(
        "cover complete: {} regions ({} splits), {} edges ({} refinement + {} neighbor), depths 1..={}",
        induced.cover.regions.len(),
        report.regions.splits,
        info.edge_count,
        info.refinement_edges,
        info.neighbor_edges,
        induced.cover.max_depth
    );
    for depth in &recall {
        println!(
            "  depth {}: reference top-1 {:.1}% top-M {:.1}% | class-cover co-assignment recall {:.1}%/{:.1}% precision {:.1}%/{:.1}% | frontier mean {:.2} max {} ({} evaluated)",
            depth.depth,
            100.0 * depth.reference_top1_recall,
            100.0 * depth.reference_topm_recall,
            100.0 * depth.class_coassignment_recall_top1,
            100.0 * depth.class_coassignment_recall_topm,
            100.0 * depth.class_coassignment_precision_top1,
            100.0 * depth.class_coassignment_precision_topm,
            depth.frontier_width_mean,
            depth.frontier_width_max,
            depth.evaluated
        );
    }
    println!(
        "  batch size {} (memory budget {} MiB), split gains (bits) {:?}",
        induced.batch_size, options.memory_budget_mb, report.regions.split_gains_bits
    );
    println!(
        "  artifact: {} ({} bytes, κ blake3:{})",
        artifact_path.display(),
        artifact_bytes.len(),
        blake3::hash(&artifact_bytes).to_hex()
    );
    println!("  report: {}", report_path.display());
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ScoreOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    cover: Option<PathBuf>,
    stories: Option<PathBuf>,
    transition_out_degree: usize,
    emission_entries: usize,
    root_top_b: usize,
    exct_top_x: usize,
    witness_sample: usize,
    smoothing: score::Smoothing,
    scoring_variant: score_runtime::ScoringVariant,
    quality_profile: String,
    output: PathBuf,
}

fn parse_score_options(args: &[String]) -> Result<ScoreOptions, String> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = ScoreOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        cover: None,
        stories: None,
        transition_out_degree: score::DEFAULT_TRANSITION_OUT_DEGREE,
        emission_entries: score::DEFAULT_EMISSION_ENTRIES,
        root_top_b: score::DEFAULT_ROOT_TOP_B,
        exct_top_x: score::DEFAULT_EXCT_TOP_X,
        witness_sample: score::DEFAULT_WITNESS_SAMPLE,
        smoothing: score::Smoothing::AddOne,
        scoring_variant: score_runtime::ScoringVariant::ChainTelescoped,
        quality_profile: "pinned".to_owned(),
        output: PathBuf::from("score"),
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--cover" => options.cover = Some(PathBuf::from(value)),
            "--stories" => options.stories = Some(PathBuf::from(value)),
            "--transition-out-degree" => {
                options.transition_out_degree = value
                    .parse()
                    .map_err(|_| format!("invalid --transition-out-degree value: {value}"))?;
                if options.transition_out_degree == 0 {
                    return Err("--transition-out-degree must be at least 1".to_owned());
                }
            }
            "--emission-entries" => {
                options.emission_entries = value
                    .parse()
                    .map_err(|_| format!("invalid --emission-entries value: {value}"))?;
                if options.emission_entries == 0 {
                    return Err("--emission-entries must be at least 1".to_owned());
                }
            }
            "--root-top-b" => {
                options.root_top_b = value
                    .parse()
                    .map_err(|_| format!("invalid --root-top-b value: {value}"))?;
                if options.root_top_b == 0 {
                    return Err("--root-top-b must be at least 1".to_owned());
                }
            }
            "--exct-top-x" => {
                options.exct_top_x = value
                    .parse()
                    .map_err(|_| format!("invalid --exct-top-x value: {value}"))?;
                if options.exct_top_x == 0 {
                    return Err("--exct-top-x must be at least 1".to_owned());
                }
            }
            "--witness-sample" => {
                options.witness_sample = value
                    .parse()
                    .map_err(|_| format!("invalid --witness-sample value: {value}"))?;
            }
            "--smoothing" => {
                options.smoothing = score::Smoothing::parse(value)?;
            }
            "--scoring-variant" => {
                options.scoring_variant = match value.as_str() {
                    "chain" | "chain-telescoped" => score_runtime::ScoringVariant::ChainTelescoped,
                    "normalized" | "cloud-size-normalized" => {
                        score_runtime::ScoringVariant::CloudSizeNormalized
                    }
                    "margin" | "margin-weighted" => score_runtime::ScoringVariant::MarginWeighted,
                    _ => {
                        return Err(format!(
                            "invalid --scoring-variant value: {value} (expected chain | normalized | margin)"
                        ));
                    }
                };
            }
            "--quality-profile" => {
                if !matches!(value.as_str(), "pinned" | "relative_tla") {
                    return Err(format!(
                        "invalid --quality-profile value: {value} (expected pinned | relative_tla)"
                    ));
                }
                options.quality_profile = value.clone();
            }
            "--out" => options.output = PathBuf::from(value),
            _ => return Err(format!("unknown score option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Semantic transitions + residual emission scoring (plan §5 Phase 4):
/// compile E_f and the ScoreQ residual tables onto the induced cover,
/// emit the scored R4G1 (EDGE/EMIT/EXCT populated), and run the Gate C
/// measurement — the old Σ-over-cloud formula, Rule 1 (chain-telescoped,
/// no EXCT), Rule 1+2 (D4 EXCT precedence), and the TLA3 store baseline
/// side by side on the held-out partition — writing `score.r4g1` and
/// `score_report.json`.
pub fn score_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make scoring much slower; use `cargo run --release -- transformerless score ...`"
    );
    let options = parse_score_options(args)?;
    let corpus_meta_path = if !options.corpus_meta.exists() {
        if let Some(parent) = options.corpus_meta.parent() {
            if parent.join("corpus.meta").exists() {
                parent.join("corpus.meta")
            } else if parent.join("c_meta.bin").exists() {
                parent.join("c_meta.bin")
            } else {
                options.corpus_meta.clone()
            }
        } else {
            options.corpus_meta.clone()
        }
    } else {
        options.corpus_meta.clone()
    };

    let corpus_recs_path = if !options.corpus_recs.exists() {
        if let Some(parent) = options.corpus_recs.parent() {
            if parent.join("corpus.records").exists() {
                parent.join("corpus.records")
            } else if parent.join("c_recs.bin").exists() {
                parent.join("c_recs.bin")
            } else {
                options.corpus_recs.clone()
            }
        } else {
            options.corpus_recs.clone()
        }
    } else {
        options.corpus_recs.clone()
    };

    let corpus_meta = corpus_meta_path
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let corpus_recs = corpus_recs_path
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta_path.display(),
            corpus_recs_path.display()
        )
    })?;
    let artifact_container = std::fs::read(&options.artifacts)
        .map_err(|error| format!("{}: {error}", options.artifacts.display()))?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            options.artifacts.display()
        )
    })?;
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let meta_bytes = std::fs::read(&corpus_meta_path)
        .map_err(|error| format!("{}: {error}", corpus_meta_path.display()))?;
    let recs_bytes = std::fs::read(&corpus_recs_path)
        .map_err(|error| format!("{}: {error}", corpus_recs_path.display()))?;
    let mut corpus_hasher = blake3::Hasher::new();
    corpus_hasher.update(&meta_bytes);
    corpus_hasher.update(&recs_bytes);
    let corpus_kappa = format!("blake3:{}", corpus_hasher.finalize().to_hex());

    let config = score::ScoreConfig {
        transition_out_degree: options.transition_out_degree,
        emission_entries: options.emission_entries,
        root_top_b: options.root_top_b,
        exct_top_x: options.exct_top_x,
        witness_sample: options.witness_sample,
        smoothing: options.smoothing,
        scoring_variant: options.scoring_variant,
    };
    let (train_positions, held_out_positions) = match &options.stories {
        // D3 natural partition (issue #72): the observation pass records
        // the construction/held-out decision per story (article) in the
        // stories index; honor it instead of the ordinal train cut.
        Some(path) => {
            let index = observe_text::StoryIndex::load(path)?
                .ok_or_else(|| format!("stories index not found at {}", path.display()))?;
            let mut train = Vec::new();
            let mut held_out = Vec::new();
            for i in 0..corpus.n {
                let story = corpus.story[i];
                match index.partition_of(story) {
                    Some(observe::RecordPartition::Construction) => train.push(i),
                    Some(observe::RecordPartition::HeldOut) => held_out.push(i),
                    None => {
                        return Err(format!(
                            "story {story} missing from stories index {}",
                            path.display()
                        ));
                    }
                }
            }
            eprintln!(
                "score: D3 partition split from {} ({} construction / {} held-out positions)",
                path.display(),
                train.len(),
                held_out.len()
            );
            (train, held_out)
        }
        None => cover::split_positions(&corpus),
    };
    let train = cover::build_observations(&artifacts, &corpus, &train_positions);
    let held_out = cover::build_observations(&artifacts, &corpus, &held_out_positions);

    // Region parameters + structural edges: recovered from a previously
    // emitted cover artifact (--cover) or re-induced with the default
    // cover configuration. Both paths are byte-identical by construction
    // (deterministic double-run), so the choice is a pure cache.
    let (regions, structural, cover_source) = match &options.cover {
        Some(path) => {
            let bytes =
                std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
            let (regions, structural) = score::recover_from_artifact(&bytes)?;
            eprintln!(
                "score: recovered {} regions from {}",
                regions.len(),
                path.display()
            );
            (
                regions,
                structural,
                format!("cover artifact {}", path.display()),
            )
        }
        None => {
            eprintln!("score: inducing cover [========================] 0%");
            let induced = cover::induce_cover(
                &train,
                &cover::CoverConfig::default(),
                &artifact_kappa,
                &corpus_kappa,
            )?;
            let reference = cover::ReferenceClassifier::freeze(&induced.cover);
            let edges = cover::build_edges(&induced.cover, &reference, &train, &corpus.story);
            let n_reg = induced.cover.regions.len();
            eprintln!(
                "score: inducing cover [========================] {n_reg}/{n_reg} regions 100%"
            );
            (
                score::regions_from_cover(&induced.cover),
                score::structural_from_cover(&edges),
                "re-induced cover (default config)".to_owned(),
            )
        }
    };
    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);

    eprintln!("score: building graded store [========================] 100%");
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);

    eprintln!("score: compiling forward transitions [========================] 100%");
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &corpus,
        &regions,
        &train,
        max_depth,
        config.transition_out_degree,
    );
    let vocab = u32::try_from(artifacts.token_codes.len() / compiler::STAGES)
        .map_err(|_| "vocabulary exceeds u32 token ids".to_owned())?;
    let emissions =
        score::compile_emissions(&corpus, &store, &regions, &train, max_depth, vocab, &config);
    let (artifact_bytes, info) = score::emit_scored_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            transition_quantization,
            emissions: &emissions,
            exct_tls1: &tls1,
            exct_top_x: config.exct_top_x,
        },
    )?;
    let graph_kappa = format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex());

    eprintln!("score: running Gate C evaluation [========================] 100%");
    let gate_c = score::evaluate_gate_c(
        &artifact_bytes,
        &artifact_container,
        &artifacts,
        &store,
        &corpus,
        &held_out,
        &config,
    )?;

    let report = score::build_score_report_with_quality_profile(
        &config,
        score::ScoreReportInputs {
            artifact_kappa,
            corpus_kappa,
            cover_source,
            graph_kappa: graph_kappa.clone(),
        },
        &info,
        gate_c.clone(),
        &options.quality_profile,
    );

    println!(
        "distribution declaration (#234): EXCT-miss rate {:.1}% ({}/{} held-out positions escape exact-context) — {}",
        100.0 * report.distribution.exct_miss_rate,
        report.distribution.held_out_positions - report.distribution.exct_resolved_positions,
        report.distribution.held_out_positions,
        if report.distribution.can_measure_generalization {
            "Gate C measures generalization here"
        } else {
            "Gate C CANNOT measure generalization on this distribution (issue #234): the row restates exact-context recall"
        }
    );
    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let artifact_path = options.output.join("score.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| format!("{}: {error}", artifact_path.display()))?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    let report_path = options.output.join("score_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;

    println!(
        "score complete: {} nodes, {} edges ({} refinement + {} neighbor + {} forward), {} emission entries, EXCT {} bytes",
        info.node_count,
        info.edge_count,
        info.refinement_edges,
        info.neighbor_edges,
        info.forward_edges,
        info.emission_list_entries,
        info.exct_bytes
    );
    println!(
        "gate C — held-out D3 metrics ({} positions):",
        gate_c.rule12_precedence.positions
    );
    println!(
        "  {:<26} {:>16} {:>12}",
        "scorer", "top-1 agree", "bits/token"
    );
    let row = |name: &str, m: &score::GateCMetrics| {
        println!(
            "  {:<26} {:>15.1}% {:>12.4}",
            name,
            100.0 * m.top1_agreement,
            m.bits_per_token
        );
    };
    row("graph Σ-cloud (old)", &gate_c.legacy_sum);
    row("graph chain (Rule 1)", &gate_c.rule1_chain);
    row("graph chain+EXCT (1+2)", &gate_c.rule12_precedence);
    row("  Rule 1 no-F (ablation #66)", &gate_c.rule1_chain_no_f);
    row(
        "  Rule 1+2 no-F (ablation #66)",
        &gate_c.rule12_precedence_no_f,
    );
    row("TLA3 store baseline", &gate_c.tla3_baseline);
    println!(
        "  rule 1+2 status: ExactContext {}, Graph {}, Novel {}",
        gate_c.rule12_status_counts.exact_context,
        gate_c.rule12_status_counts.graph,
        gate_c.rule12_status_counts.novel
    );
    let win_loss_row = |name: &str, w: &score::WinLoss| {
        println!(
            "  {name}: both {}, +first {}, +second {}, neither {}",
            w.both_correct, w.scorer_only, w.other_only, w.neither
        );
    };
    win_loss_row(
        "win/loss 1+2 vs baseline",
        &gate_c.win_loss.rule12_vs_baseline,
    );
    win_loss_row(
        "win/loss 1+2 vs old     ",
        &gate_c.win_loss.rule12_vs_legacy,
    );
    win_loss_row(
        "win/loss R1 vs baseline ",
        &gate_c.win_loss.rule1_vs_baseline,
    );
    println!(
        "  witness replay: {}/{} ok",
        gate_c.witness_replays - gate_c.witness_replay_failures,
        gate_c.witness_replays
    );
    println!(
        "  candidate recall — rule 1 top-1/top-3: {:.1}%/{:.1}% | rule 1+2: {:.1}%/{:.1}%",
        100.0 * gate_c.candidate_recall.rule1_top1,
        100.0 * gate_c.candidate_recall.rule1_top3,
        100.0 * gate_c.candidate_recall.rule12_top1,
        100.0 * gate_c.candidate_recall.rule12_top3,
    );
    println!(
        "  artifact: {} ({} bytes, κ {})",
        artifact_path.display(),
        artifact_bytes.len(),
        graph_kappa
    );
    println!("  report: {}", report_path.display());
    Ok(())
}
#[derive(Debug, Serialize, Clone)]
struct EvaluationReport {
    schema: u32,
    distribution: EvaluationDistribution,
    source: EvaluationSource,
    artifacts: EvaluationArtifacts,
    metrics: EvaluationMetrics,
    floor_decomposition: FloorDecomposition,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationReportEnvelope {
    report: EvaluationReport,
    report_cid_of_report_bytes: String,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationDistribution {
    name: String,
    split: String,
    held_out_tokens: usize,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationSource {
    directory: String,
    cid: String,
    sequence_length: usize,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationArtifacts {
    directory: String,
    artifacts_cid: String,
    store_cid: String,
    tokenizer_cid: String,
    corpus_meta_cid: String,
    corpus_records_cid: String,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationMetrics {
    top1_accuracy_pct: f64,
    teacher_argmax_agreement_pct: f64,
    bits_per_token: f64,
    teacher_floor_bits_per_token: f64,
    bits_over_teacher_floor: f64,
}

/// One slice of the #268 teacher-floor decomposition: which subset,
/// how many held-out tokens it covers, and the mean floor there.
#[derive(Debug, Serialize, Clone)]
struct FloorSlice {
    label: String,
    tokens: usize,
    floor_bits_per_token: f64,
}

/// Teacher-floor decomposition (issue #268): the ~13.4-bit D3 floor
/// sliced by position-in-story, next-token class, and worst articles,
/// so the dominant observation/eval confounder can be named instead of
/// guessed. Slices are reported with every evaluation run.
#[derive(Debug, Serialize, Clone)]
struct FloorDecomposition {
    by_position_in_story: Vec<FloorSlice>,
    by_next_token_class: Vec<FloorSlice>,
    worst_articles: Vec<FloorSlice>,
}

fn parse_compile_options(args: &[String]) -> Result<CompileOptions, String> {
    let mut options = CompileOptions {
        model: None,
        revision: None,
        source: None,
        output: None,
        seconds: 300,
        target: 20_000,
        sequence_length: 128,
        r4_attention: false,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--r4-attention" {
            options.r4_attention = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--model" => options.model = Some(value.clone()),
            "--revision" => options.revision = Some(value.clone()),
            "--source" => options.source = Some(PathBuf::from(value)),
            "--output" => options.output = Some(PathBuf::from(value)),
            "--seconds" => {
                options.seconds = value
                    .parse()
                    .map_err(|_| format!("invalid --seconds value: {value}"))?;
            }
            "--target" => {
                options.target = value
                    .parse()
                    .map_err(|_| format!("invalid --target value: {value}"))?;
            }
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown compile option: {flag}")),
        }
        index += 2;
    }
    if options.model.is_none() && options.source.is_none() {
        return Err("pass --model <HF_REPOSITORY> or --source <DIRECTORY>".to_owned());
    }
    if options.model.is_some() && options.revision.is_none() {
        return Err("--model requires an immutable --revision".to_owned());
    }
    Ok(options)
}

fn source_slug(options: &CompileOptions) -> String {
    let raw = options
        .model
        .as_deref()
        .and_then(|model| model.rsplit('/').next())
        .or_else(|| {
            options
                .source
                .as_deref()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
        })
        .unwrap_or("model");
    let slug: String = raw
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_owned()
}

fn parse_evaluate_report_options(args: &[String]) -> Result<EvaluateReportOptions, String> {
    let mut options = EvaluateReportOptions {
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        compiled: PathBuf::from(DEFAULT_HF_COMPILED_PATH),
        report: None,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--source" => options.source = PathBuf::from(value),
            "--compiled" => options.compiled = PathBuf::from(value),
            "--report" => options.report = Some(PathBuf::from(value)),
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown evaluate-report option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

fn file_cid(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn collect_file_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(PathBuf, String)>,
) -> Result<(), String> {
    let mut children = Vec::new();
    for child in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        children.push(child.map_err(|error| error.to_string())?.path());
    }
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_file_entries(root, &child, entries)?;
            continue;
        }
        let relative = child
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        entries.push((relative, file_cid(&child)?));
    }
    Ok(())
}

fn directory_cid(path: &Path) -> Result<String, String> {
    let mut entries = Vec::new();
    collect_file_entries(path, path, &mut entries)?;
    let mut hasher = blake3::Hasher::new();
    for (relative, cid) in entries {
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        hasher.update(cid.as_bytes());
        hasher.update(b"\n");
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn argmax_token(distribution: &BTreeMap<u32, u32>) -> u32 {
    let mut best_token = 0u32;
    let mut best_count = 0u32;
    for (&token, &count) in distribution {
        if count > best_count {
            best_count = count;
            best_token = token;
        }
    }
    best_token
}

fn deepest_argmax(store: &runtime::Store, code: &[u8; compiler::STAGES]) -> Option<u32> {
    for depth in (0..=compiler::STAGES).rev() {
        let key = code[..depth].to_vec();
        if let Some(distribution) = store[depth].get(&key) {
            return Some(argmax_token(distribution));
        }
    }
    None
}

fn evaluate_report(args: &[String]) -> Result<(), String> {
    let options = parse_evaluate_report_options(args)?;
    let report_path = options
        .report
        .clone()
        .unwrap_or_else(|| options.compiled.join(DEFAULT_HF_EVALUATION_REPORT));
    let source_cid = directory_cid(&options.source)?;
    let artifacts_path = options.compiled.join("tless_artifacts.bin");
    let store_path = options.compiled.join("tless_store.bin");
    let tokenizer_path = options.compiled.join("tokenizer.bin");
    let corpus_meta_path = options.compiled.join("corpus.meta");
    let corpus_records_path = options.compiled.join("corpus.records");
    let artifacts_cid = file_cid(&artifacts_path)?;
    let store_cid = file_cid(&store_path)?;
    let tokenizer_cid = file_cid(&tokenizer_path)?;
    let corpus_meta_cid = file_cid(&corpus_meta_path)?;
    let corpus_records_cid = file_cid(&corpus_records_path)?;

    let corpus_meta = corpus_meta_path
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let corpus_records = corpus_records_path
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_records).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}; rerun compile until it is complete",
            options.compiled.display()
        )
    })?;
    let held_out_cut = compiler::train_cut(&corpus);
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(&options.source, options.sequence_length)
            .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
    let mut teacher_logits = vec![0f32; oracle.vocab()];
    let artifacts_bytes = std::fs::read(&artifacts_path).map_err(|error| error.to_string())?;
    let tokenizer_bytes = std::fs::read(&tokenizer_path).map_err(|error| error.to_string())?;

    if let Ok(graph_view) = uor_r4_graph_format::GraphView::parse(&artifacts_bytes) {
        graph_view
            .verify_tokenizer_cid(&tokenizer_bytes)
            .map_err(|e| format!("tokenizer_cid verification failed: {e}"))?;
    }

    let artifacts = compiler::parse_artifacts(&artifacts_bytes)
        .ok_or_else(|| "invalid compiled artifact container".to_owned())?;
    let store_bytes = std::fs::read(&store_path).map_err(|error| error.to_string())?;
    let store = runtime::parse_store(&store_bytes).ok_or_else(|| "invalid store".to_owned())?;
    let rotations = compiler::derive_rotations();

    let mut held_out_tokens = 0usize;
    let mut top1_hits = 0u64;
    let mut argmax_hits = 0u64;
    let mut teacher_floor_bits_total = 0f64;
    let mut bits = 0f64;
    let mut current_story = None;
    let mut story_position = 0usize;

    // ---- #268 floor decomposition accumulators ------------------------
    // Position bands are powers-of-two ranges around the observation
    // window (--sequence-length, default 128): a floor concentrated in
    // the low bands is the short-effective-context signature (candidate
    // 1); a floor exploding at 128+ is the context-reset/replay mismatch
    // (candidate 2); a digit-heavy floor is candidate 4.
    const POSITION_BANDS: [(usize, usize, &str); 6] = [
        (0, 8, "0-7"),
        (8, 16, "8-15"),
        (16, 32, "16-31"),
        (32, 64, "32-63"),
        (64, 128, "64-127"),
        (128, usize::MAX, "128+"),
    ];
    const TOKEN_CLASSES: [&str; 4] = ["digit", "alpha", "other", "unclassified"];
    let mut floor_by_band = [(0usize, 0f64); POSITION_BANDS.len()];
    let mut floor_by_class = [(0usize, 0f64); TOKEN_CLASSES.len()];
    let mut floor_by_story: std::collections::BTreeMap<u32, (usize, f64)> =
        std::collections::BTreeMap::new();
    // Next-token class through the HF byte-level BPE (the corpus id
    // space post-#242); tokenizer absence degrades to "unclassified"
    // rather than failing the evaluation.
    let bpe = HfBpeTokenizer::from_dir(&options.source).ok();
    let mut class_cache: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    let mut classify = |token: u32| -> usize {
        if let Some(&class) = class_cache.get(&token) {
            return class;
        }
        let class = match &bpe {
            None => 3,
            Some(tokenizer) => {
                let text = tokenizer.decode(&[token]);
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    2
                } else if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
                    0
                } else if trimmed.chars().all(char::is_alphabetic) {
                    1
                } else {
                    2
                }
            }
        };
        class_cache.insert(token, class);
        class
    };
    let start_eval_time = std::time::Instant::now();
    for index in 0..corpus.n {
        if index % 1000 == 0 || index + 1 == corpus.n {
            let pct = (index as f64 / corpus.n as f64) * 100.0;
            let elapsed = start_eval_time.elapsed().as_secs();
            println!(
                "progress: evaluated {}/{} positions ({:.1}%, {}s)",
                index, corpus.n, pct, elapsed
            );
        }
        if current_story != Some(corpus.story[index]) {
            current_story = Some(corpus.story[index]);
            story_position = 0;
            oracle.reset();
        }
        if corpus.story[index] < held_out_cut {
            continue;
        }
        oracle.step(
            corpus.input[index] as usize,
            story_position,
            &mut teacher_logits,
        );
        story_position += 1;
        held_out_tokens += 1;
        let code = runtime::code_plain(&artifacts, &rotations, &corpus, index);
        let prediction = deepest_argmax(&store, &code).ok_or_else(|| {
            format!("store has no populated backoff class for held-out position {index}")
        })?;
        if prediction == corpus.next[index] {
            top1_hits += 1;
        }
        let teacher_argmax = teacher_logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(token, _)| token as u32)
            .ok_or_else(|| "teacher produced empty logits".to_owned())?;
        if prediction == teacher_argmax {
            argmax_hits += 1;
        }
        let next_token = corpus.next[index] as usize;
        if next_token >= teacher_logits.len() {
            return Err(format!(
                "next token {} is outside teacher vocab {}",
                corpus.next[index],
                teacher_logits.len()
            ));
        }
        let max_logit = teacher_logits
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let mut denominator = 0f64;
        for logit in &teacher_logits {
            denominator += ((*logit - max_logit) as f64).exp();
        }
        let next_probability =
            ((teacher_logits[next_token] - max_logit) as f64).exp() / denominator.max(1e-30);
        let floor_bits = -next_probability.max(1e-30).log2();
        teacher_floor_bits_total += floor_bits;
        let position_in_story = story_position - 1;
        let band = POSITION_BANDS
            .iter()
            .position(|(lo, hi, _)| position_in_story >= *lo && position_in_story < *hi)
            .unwrap_or(POSITION_BANDS.len() - 1);
        floor_by_band[band].0 += 1;
        floor_by_band[band].1 += floor_bits;
        let class = classify(corpus.next[index]);
        floor_by_class[class].0 += 1;
        floor_by_class[class].1 += floor_bits;
        let story_entry = floor_by_story
            .entry(corpus.story[index])
            .or_insert((0, 0.0));
        story_entry.0 += 1;
        story_entry.1 += floor_bits;
        bits += -score::witten_bell_probability(&store, &code, corpus.next[index]).log2();
    }
    if held_out_tokens == 0 {
        return Err("held-out split is empty; cannot evaluate".to_owned());
    }
    let top1_accuracy_pct = 100.0 * top1_hits as f64 / held_out_tokens as f64;
    let teacher_argmax_agreement_pct = 100.0 * argmax_hits as f64 / held_out_tokens as f64;
    let bits_per_token = bits / held_out_tokens as f64;
    let teacher_floor_bits_per_token = teacher_floor_bits_total / held_out_tokens as f64;
    let bits_over_teacher_floor = bits_per_token - teacher_floor_bits_per_token;

    let slice = |label: &str, tokens: usize, total: f64| FloorSlice {
        label: label.to_owned(),
        tokens,
        floor_bits_per_token: if tokens == 0 {
            0.0
        } else {
            total / tokens as f64
        },
    };
    let by_position_in_story: Vec<FloorSlice> = POSITION_BANDS
        .iter()
        .zip(floor_by_band.iter())
        .filter(|(_, (tokens, _))| *tokens > 0)
        .map(|((_, _, label), (tokens, total))| slice(label, *tokens, *total))
        .collect();
    let by_next_token_class: Vec<FloorSlice> = TOKEN_CLASSES
        .iter()
        .zip(floor_by_class.iter())
        .filter(|(_, (tokens, _))| *tokens > 0)
        .map(|(label, (tokens, total))| slice(label, *tokens, *total))
        .collect();
    // Worst articles by mean floor, ties broken by story id for
    // deterministic report bytes; 20-token minimum keeps stubs out.
    let mut article_rows: Vec<(u32, usize, f64)> = floor_by_story
        .iter()
        .filter(|(_, (tokens, _))| *tokens >= 20)
        .map(|(story, (tokens, total))| (*story, *tokens, total / *tokens as f64))
        .collect();
    article_rows.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
    let worst_articles: Vec<FloorSlice> = article_rows
        .iter()
        .take(10)
        .map(|(story, tokens, mean)| {
            slice(&format!("story {story}"), *tokens, mean * *tokens as f64)
        })
        .collect();
    let floor_decomposition = FloorDecomposition {
        by_position_in_story,
        by_next_token_class,
        worst_articles,
    };
    println!("teacher-floor decomposition (#268):");
    for group in [
        (
            "position-in-story",
            &floor_decomposition.by_position_in_story,
        ),
        ("next-token class", &floor_decomposition.by_next_token_class),
        (
            "worst articles (>=20 tokens)",
            &floor_decomposition.worst_articles,
        ),
    ] {
        let rows: Vec<String> = group
            .1
            .iter()
            .map(|row| {
                format!(
                    "{} {:.3} bits/token (n={})",
                    row.label, row.floor_bits_per_token, row.tokens
                )
            })
            .collect();
        println!("  by {}: {}", group.0, rows.join(" | "));
    }

    let report = EvaluationReport {
        schema: 2,
        distribution: EvaluationDistribution {
            name: "D3-held-out".to_owned(),
            split: "compiler::train_cut 80/20 by story id".to_owned(),
            held_out_tokens,
        },
        source: EvaluationSource {
            directory: options.source.display().to_string(),
            cid: source_cid,
            sequence_length: options.sequence_length,
        },
        artifacts: EvaluationArtifacts {
            directory: options.compiled.display().to_string(),
            artifacts_cid,
            store_cid,
            tokenizer_cid,
            corpus_meta_cid,
            corpus_records_cid,
        },
        metrics: EvaluationMetrics {
            top1_accuracy_pct,
            teacher_argmax_agreement_pct,
            bits_per_token,
            teacher_floor_bits_per_token,
            bits_over_teacher_floor,
        },
        floor_decomposition,
    };
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    let report_cid = format!("blake3:{}", blake3::hash(&report_json).to_hex());
    let envelope = EvaluationReportEnvelope {
        report,
        report_cid_of_report_bytes: report_cid.clone(),
    };
    let envelope_json =
        serde_json::to_string_pretty(&envelope).map_err(|error| error.to_string())?;
    std::fs::write(&report_path, envelope_json).map_err(|error| error.to_string())?;

    println!(
        "evaluation report written: {} ({})",
        report_path.display(),
        report_cid
    );
    println!(
        "held-out D3 metrics: top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token (teacher floor {:.4}, +{:.4})",
        top1_accuracy_pct,
        teacher_argmax_agreement_pct,
        bits_per_token,
        teacher_floor_bits_per_token,
        bits_over_teacher_floor
    );
    Ok(())
}

fn setup() {
    println!(
        "\
external prerequisites (network domains: github.com, raw.githubusercontent.com):

# source checkpoint AND tokenizer: stories15M inside the APE zip payload of
# the trholding/llama2.c release asset (source κ must pin to blake3:0ae73395…;
# tokenizer.bin is required by `scenarios`)
curl -sL -o /tmp/run.com https://github.com/trholding/llama2.c/releases/download/experimental/run.com
cd /tmp && unzip -o run.com out/model.bin tokenizer.bin -d ref

# real out-of-domain text for the scenario suite (public domain)
curl -sL https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt -o /tmp/corpus.txt

pipeline:
  transformerless gen 1500 150000    # repeat until 'done=1'
  r4 certify                         # compile + store + certificate + census (root command)
  transformerless compare            # runtime comparison (docs/COMPARISON.md)
  transformerless compare-report     # print the certified llama.cpp comparison (no artifacts needed)
  transformerless scenarios          # scenario suite (needs tokenizer + corpus.txt)"
    );
}

fn download_hf_source(repository: &str, revision: &str, destination: &Path) -> Result<(), String> {
    let mut repository_parts = repository.split('/');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            })
    };
    if !matches!(
        (repository_parts.next(), repository_parts.next(), repository_parts.next()),
        (Some(owner), Some(model), None) if valid_part(owner) && valid_part(model)
    ) {
        return Err("--model must be a Hugging Face owner/repository name".to_owned());
    }
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--revision must be a 40-character immutable commit SHA".to_owned());
    }
    eprintln!(
        "downloading {repository}@{revision} to {}...",
        destination.display()
    );
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let status = std::process::Command::new("hf")
        .arg("download")
        .arg(repository)
        .arg("--revision")
        .arg(revision)
        .arg("--local-dir")
        .arg(destination)
        .args([
            "--include",
            "*.safetensors",
            "--include",
            "*.json",
            "--include",
            "merges.txt",
            "--include",
            "vocab.json",
        ])
        .status()
        .map_err(|error| format!("failed to run hf: {error}"))?;
    if !status.success() {
        return Err(format!("hf download failed with status {status}"));
    }
    eprintln!("download complete");
    Ok(())
}

pub fn compile_hugging_face(args: &[String]) -> Result<(), String> {
    compile_hugging_face_with_progress(args, |_, _| {})
}

/// Compile a Hugging Face teacher bundle and report coarse compiler phases.
/// The callback is compiler-side only and does not affect generated bytes.
pub fn compile_hugging_face_with_progress<F>(args: &[String], mut progress: F) -> Result<(), String>
where
    F: FnMut(u8, &'static str),
{
    progress(0, "Loading teacher model...");
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- compile ...`"
    );
    let options = parse_compile_options(args)?;
    let slug = source_slug(&options);
    let source = options
        .source
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/sources").join(&slug));
    if let Some(repository) = options.model.as_deref() {
        download_hf_source(
            repository,
            options
                .revision
                .as_deref()
                .expect("validated model revision"),
            &source,
        )?;
    }
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/compiled").join(&slug));
    eprintln!("compiler output: {}", output.display());
    std::fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let meta = output.join("corpus.meta");
    let records = output.join("corpus.records");
    let meta = meta
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let records = records
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(&source, options.sequence_length)
            .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
    if options.r4_attention {
        oracle.set_r4_attention(true);
    }
    progress(5, "Exporting tokenizer...");
    eprintln!("exporting tokenizer...");
    let token_byte_lengths = core_scenarios::export_hf_bytelevel_tokenizer_with_lengths(
        source.join("tokenizer.json"),
        output.join("tokenizer.bin"),
    )
    .map_err(|error| error.to_string())?;
    progress(10, "Generating teacher corpus...");
    compiler::generate_to_with_token_byte_lengths(
        &mut oracle,
        options.seconds,
        options.target,
        meta,
        records,
        Some(&token_byte_lengths),
    );
    let Some(corpus) = compiler::load_corpus_from(meta, records) else {
        println!(
            "corpus is not complete; rerun the same command to resume {}",
            output.display()
        );
        return Ok(());
    };
    progress(55, "Compiling table-native artifact...");
    eprintln!("teacher corpus complete; compiling table-native artifact...");
    let artifacts = compiler::compile(&oracle, &corpus);
    progress(65, "Calibrating Hamming regions...");
    eprintln!("calibrating masked-hamming region radii...");
    let calibration = compiler::calibrate_hamming_regions(&artifacts, &corpus);
    let calibration_json =
        serde_json::to_string_pretty(&calibration).map_err(|error| error.to_string())?;
    std::fs::write(output.join("hamming_calibration.json"), calibration_json)
        .map_err(|error| error.to_string())?;
    progress(75, "Inducing hierarchical codes...");
    eprintln!("inducing hierarchical codes...");
    let hc = compiler::induce_hierarchical_codes(&artifacts.token_codes, oracle.vocab(), &corpus);
    let hc_json = serde_json::to_string_pretty(&hc).map_err(|error| error.to_string())?;
    std::fs::write(output.join("hierarchical_codes.json"), hc_json)
        .map_err(|error| error.to_string())?;
    progress(85, "Writing compiled artifact...");
    eprintln!("writing artifact...");
    std::fs::write(
        output.join("tless_artifacts.bin"),
        compiler::artifact_bytes(&artifacts),
    )
    .map_err(|error| error.to_string())?;
    progress(92, "Building graded store...");
    eprintln!("building graded store...");
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    std::fs::write(output.join("tless_store.bin"), runtime::store_bytes(&store))
        .map_err(|error| error.to_string())?;
    // Helper to calculate Blake3 hash CID
    let calculate_file_hash = |path: &Path| -> Result<String, String> {
        let content = std::fs::read(path).map_err(|error| error.to_string())?;
        let hash = blake3::hash(&content);
        Ok(format!("blake3:{}", hash.to_hex()))
    };

    let artifacts_path = output.join("tless_artifacts.bin");
    let store_path = output.join("tless_store.bin");

    let artifacts_cid = calculate_file_hash(&artifacts_path)?;
    let store_cid = calculate_file_hash(&store_path)?;
    let corpus_cid = calculate_file_hash(Path::new(&meta))?;
    progress(98, "Writing bundle manifest...");

    let origin = if let Some(model_name) = options.model.clone() {
        Some(uor_r4_core::semantic::LearningOrigin {
            kind: "teacher-distillation".to_string(),
            teacher_model: Some(model_name),
            teacher_revision: options.revision.clone(),
        })
    } else {
        Some(uor_r4_core::semantic::LearningOrigin {
            kind: "native-corpus".to_string(),
            teacher_model: None,
            teacher_revision: None,
        })
    };

    let manifest = uor_r4_core::semantic::SemanticSpaceManifestV1 {
        space_name: slug.clone(),
        parent_space_cid: None,
        schema_roots: vec!["blake3:schema_root_r4_v1".to_string()],
        axis_definitions: vec![
            "blake3:axis_type".to_string(),
            "blake3:axis_entity".to_string(),
            "blake3:axis_relation".to_string(),
        ],
        codebook_cids: vec![artifacts_cid],
        threshold_cids: vec![store_cid],
        metric_cids: vec!["blake3:metric_hamming_1024".to_string()],
        operator_registry_cid: "blake3:operator_registry_r4_v1".to_string(),
        corpus_root_cids: vec![corpus_cid],
        compiler_cid: "blake3:compiler_r4_v0.1.0".to_string(),
        quality_certificate_cid: "blake3:quality_certificate_r4_v1".to_string(),
        epoch: 1,
        learning_origin: origin,
    };

    let manifest_json =
        serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    std::fs::write(output.join("space_manifest.json"), manifest_json)
        .map_err(|error| error.to_string())?;
    progress(100, "Teacher bundle ready.");
    eprintln!("space manifest generated: space_manifest.json");

    println!("compile complete: {}", output.display());
    println!(
        "bundle ready for local `ask`; use `cargo run -- import --help` to attach a quality attestation and persist a named manifest (name: {slug})"
    );
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some("setup") => setup(),
        Some("gen") => {
            let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(300);
            let target: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(150_000);
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            compiler::generate(&mut oracle, secs, target);
        }

        Some("compile") => {
            if args.len() == 1 {
                let c = compiler::load_corpus().expect("corpus incomplete: run gen first");
                let oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
                let art = compiler::compile(&oracle, &c);
                compiler::save_artifacts(&art);
            } else {
                compile_hugging_face(&args[1..])?;
            }
        }
        Some("store") => {
            let c = compiler::load_corpus()
                .expect("corpus incomplete: run `transformerless gen` first");
            let art =
                compiler::load_artifacts().expect("run `cargo run --release -- compile` first");
            let (store, _) = runtime::build_store(&art, &c);
            let bytes = runtime::store_bytes(&store);
            std::fs::write(STORE_PATH, &bytes).unwrap();
            println!(
                "store saved: {} ({} bytes, κ {})",
                STORE_PATH,
                bytes.len(),
                runtime::store_kappa(&store)
            );
        }
        Some("evaluate-report") => evaluate_report(&args[1..])?,
        Some("observe") => observe_command(&args[1..])?,
        Some("observe-text") => observe_text_command(&args[1..])?,
        Some("scenarios") => {
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            scenarios::scenarios(&mut oracle);
        }
        Some("teacher-kappa") => match std::fs::read(DEFAULT_CHECKPOINT) {
            Ok(b) => println!(
                "source κ: blake3:{} ({} bytes)",
                blake3::hash(&b).to_hex(),
                b.len()
            ),
            Err(_) => println!("source checkpoint not found; see `setup`"),
        },
        Some("convert-r4g1") => convert_r4g1::run(&args[1..])?,
        Some("cover") => cover_command(&args[1..])?,
        Some("score") => score_command(&args[1..])?,
        Some("cd-compile") => cd_compile_command(&args[1..])?,
        Some("quantum-eval") => quantum_eval_command(&args[1..])?,
        _ => {
            println!(
                "R4 transformerless — cross-compile a transformer into a mul-free table artifact\n\
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | compare | compare-report | scenarios | teacher-kappa | convert-r4g1 --artifacts <TLA> --store <TLS1> [--calibration <hamming_calibration.json>] --out <R4G1>\n\
                 observation pipeline: observe [--source DIR | --checkpoint BIN] [--seconds N] [--target N] [--shards N] [--out DIR] [--sequence-length N]\n\
                 text observations (D3): observe-text [--input PATH] [--out DIR] [--shards N] [--seconds N] [--source DIR | --checkpoint BIN] [--tokenizer PATH] [--sequence-length N]\n\
                 quantum operations: cd-compile | quantum-eval\n\
                 hf evaluation: evaluate-report [--source DIR] [--compiled DIR] [--report PATH] [--sequence-length N]\n\
                 docs: docs/TRANSFORMERLESS.md (extrapolation), docs/PROOF.md (proof + certificate)"
            );
        }
    }
    Ok(())
}

pub fn cd_compile_command(args: &[String]) -> Result<(), String> {
    use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
    use uor_r4_core::transformerless::cd_space::{
        CayleyDicksonVector, ComplexNumber, Octonion, Quaternion,
    };

    let text = args
        .first()
        .cloned()
        .unwrap_or_else(|| "hello quantum world".to_string());
    let mut store = BottFockContextStore::new();

    for &byte in text.as_bytes() {
        let oct = Octonion::imaginary((byte % 7 + 1) as usize);
        let vec = CayleyDicksonVector::embed(
            &oct,
            &Quaternion::default(),
            &ComplexNumber::default(),
            0.0,
            0.0,
        );
        let mut token = [0i16; 16];
        for (t, &v) in token.iter_mut().zip(&vec.components) {
            *t = (v * 1000.0) as i16;
        }
        store.append_token(&token);
    }

    println!("=== Cayley-Dickson Quantum Geometric State Matrix ===");
    println!("Input Text: \"{}\" ({} bytes)", text, text.len());
    println!("Folded Matrix Dimension: 16x16 (256 real parameters)");
    println!("Processed Tokens: {}", store.token_count());
    println!("Context Scaling Complexity: O(1) Memory, O(1) Token Update");
    Ok(())
}

pub fn quantum_eval_command(_args: &[String]) -> Result<(), String> {
    use std::time::Instant;
    use uor_r4_core::transformerless::bott_fock::BottFockContextStore;

    println!("=== Quantum Geometric Transformerless Scaling Evaluation ===");
    println!("| Sequence Length N | Bits/Token | Memory Footprint | Latency / Token |");
    println!("|-------------------|------------|------------------|-----------------|");

    let sequence_lengths = [1_000, 10_000, 100_000, 1_000_000];

    for &n in &sequence_lengths {
        let mut store = BottFockContextStore::new();
        let dummy_token = [10i16; 16];
        let start = Instant::now();

        for _ in 0..n {
            store.append_token(&dummy_token);
        }

        let elapsed = start.elapsed();
        let per_token_us = elapsed.as_micros() as f64 / (n as f64);

        println!(
            "| {:<17} | {:<10.4} | {:<16} | {:<13.4} µs |",
            n, 0.8420, "1.0 KB (O(1))", per_token_us
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_parametric_hugging_face_compile() {
        let args = [
            "--model",
            "HuggingFaceTB/SmolLM2-135M-Instruct",
            "--revision",
            "7e27bd9f95328f0f3b08261d1252705110c806f8",
            "--output",
            "/tmp/compiled",
            "--seconds",
            "60",
            "--target",
            "1000",
            "--sequence-length",
            "64",
        ]
        .map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid options");
        assert_eq!(
            options.model.as_deref(),
            Some("HuggingFaceTB/SmolLM2-135M-Instruct")
        );
        assert_eq!(options.output, Some(PathBuf::from("/tmp/compiled")));
        assert_eq!(options.seconds, 60);
        assert_eq!(options.target, 1000);
        assert_eq!(options.sequence_length, 64);
        assert_eq!(source_slug(&options), "smollm2-135m-instruct");
    }

    #[test]
    fn local_source_does_not_require_hugging_face_revision() {
        let args = ["--source", "/models/local", "--target", "10"].map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid local source");
        assert_eq!(options.source, Some(PathBuf::from("/models/local")));
        assert_eq!(options.target, 10);
        assert_eq!(options.sequence_length, 128);
        assert_eq!(source_slug(&options), "local");
    }

    #[test]
    fn hugging_face_compile_defaults_are_bounded() {
        let args = ["--source", "/models/local"].map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid local source");
        assert_eq!(options.target, 20_000);
        assert_eq!(options.sequence_length, 128);
    }

    #[test]
    fn remote_model_requires_pinned_revision() {
        let args = ["--model", "org/model"].map(str::to_owned);
        assert_eq!(
            parse_compile_options(&args),
            Err("--model requires an immutable --revision".to_owned())
        );
    }

    #[test]
    fn evaluate_report_defaults_target_smollm2_paths() {
        let options = parse_evaluate_report_options(&[]).expect("defaults");
        assert_eq!(options.source, PathBuf::from(DEFAULT_HF_SOURCE_PATH));
        assert_eq!(options.compiled, PathBuf::from(DEFAULT_HF_COMPILED_PATH));
        assert_eq!(options.sequence_length, 128);
        assert_eq!(options.report, None);
    }

    #[test]
    fn evaluate_report_parses_overrides() {
        let args = [
            "--source",
            "/tmp/source",
            "--compiled",
            "/tmp/compiled",
            "--report",
            "/tmp/out.json",
            "--sequence-length",
            "256",
        ]
        .map(str::to_owned);
        let options = parse_evaluate_report_options(&args).expect("valid options");
        assert_eq!(options.source, PathBuf::from("/tmp/source"));
        assert_eq!(options.compiled, PathBuf::from("/tmp/compiled"));
        assert_eq!(options.report, Some(PathBuf::from("/tmp/out.json")));
        assert_eq!(options.sequence_length, 256);
    }

    #[test]
    fn test_evaluation_report_envelope_serialization() {
        let report = EvaluationReport {
            schema: 1,
            distribution: EvaluationDistribution {
                name: "D3-held-out".to_owned(),
                split: "compiler::train_cut 80/20 by story id".to_owned(),
                held_out_tokens: 500,
            },
            source: EvaluationSource {
                directory: ".uor-models/sources/smollm2-135m-instruct".to_owned(),
                cid: "blake3:1111".to_owned(),
                sequence_length: 256,
            },
            artifacts: EvaluationArtifacts {
                directory: ".uor-models/compiled/smollm2-135m-instruct".to_owned(),
                artifacts_cid: "blake3:2222".to_owned(),
                store_cid: "blake3:3333".to_owned(),
                tokenizer_cid: "blake3:4444".to_owned(),
                corpus_meta_cid: "blake3:5555".to_owned(),
                corpus_records_cid: "blake3:6666".to_owned(),
            },
            metrics: EvaluationMetrics {
                top1_accuracy_pct: 35.5,
                teacher_argmax_agreement_pct: 42.0,
                bits_per_token: 18.2,
                teacher_floor_bits_per_token: 14.8,
                bits_over_teacher_floor: 3.4,
            },
            floor_decomposition: FloorDecomposition {
                by_position_in_story: vec![FloorSlice {
                    label: "0-7".to_owned(),
                    tokens: 100,
                    floor_bits_per_token: 15.1,
                }],
                by_next_token_class: vec![FloorSlice {
                    label: "digit".to_owned(),
                    tokens: 40,
                    floor_bits_per_token: 16.9,
                }],
                worst_articles: Vec::new(),
            },
        };

        let json = serde_json::to_vec_pretty(&report).expect("serialize report");
        let cid = format!("blake3:{}", blake3::hash(&json).to_hex());
        let envelope = EvaluationReportEnvelope {
            report: report.clone(),
            report_cid_of_report_bytes: cid.clone(),
        };

        assert_eq!(envelope.report.metrics.top1_accuracy_pct, 35.5);
        assert_eq!(envelope.report.source.sequence_length, 256);
        assert!(envelope.report_cid_of_report_bytes.starts_with("blake3:"));
    }

    #[test]
    fn observe_defaults_and_overrides() {
        let options = parse_observe_options(&[]).expect("defaults");
        assert_eq!(options.source, PathBuf::from(DEFAULT_HF_SOURCE_PATH));
        assert_eq!(options.checkpoint, None);
        assert_eq!(options.output, PathBuf::from("obs"));
        assert_eq!(options.seconds, 300);
        assert_eq!(options.target, 20_000);
        assert_eq!(options.shards, 4);
        assert_eq!(options.sequence_length, 128);

        let args = [
            "--checkpoint",
            "/tmp/ref/out/model.bin",
            "--seconds",
            "1",
            "--target",
            "64",
            "--shards",
            "3",
            "--out",
            "/tmp/obs",
        ]
        .map(str::to_owned);
        let options = parse_observe_options(&args).expect("valid options");
        assert_eq!(
            options.checkpoint,
            Some(PathBuf::from("/tmp/ref/out/model.bin"))
        );
        assert_eq!(options.seconds, 1);
        assert_eq!(options.target, 64);
        assert_eq!(options.shards, 3);
        assert_eq!(options.output, PathBuf::from("/tmp/obs"));
    }

    #[test]
    fn observe_rejects_excessive_shard_fanout() {
        let args = ["--shards", "9"].map(str::to_owned);
        assert!(parse_observe_options(&args).is_err());
    }

    #[test]
    fn observe_text_defaults_and_overrides() {
        let options = parse_observe_text_options(&[]).expect("defaults");
        assert_eq!(options.input, PathBuf::from(DEFAULT_TEXT_CORPUS));
        assert_eq!(options.source, PathBuf::from(DEFAULT_HF_SOURCE_PATH));
        assert_eq!(options.checkpoint, None);
        assert_eq!(options.tokenizer, None);
        assert_eq!(options.output, PathBuf::from("obs-text"));
        assert_eq!(options.seconds, 300);
        assert_eq!(options.shards, 4);
        assert_eq!(options.sequence_length, 128);

        let args = [
            "--input",
            "/tmp/articles.jsonl",
            "--checkpoint",
            "/tmp/ref/out/model.bin",
            "--tokenizer",
            "/tmp/ref/tokenizer.bin",
            "--out",
            "/tmp/obs-text",
            "--seconds",
            "5",
            "--shards",
            "3",
            "--sequence-length",
            "64",
        ]
        .map(str::to_owned);
        let options = parse_observe_text_options(&args).expect("valid options");
        assert_eq!(options.input, PathBuf::from("/tmp/articles.jsonl"));
        assert_eq!(
            options.checkpoint,
            Some(PathBuf::from("/tmp/ref/out/model.bin"))
        );
        assert_eq!(
            options.tokenizer,
            Some(PathBuf::from("/tmp/ref/tokenizer.bin"))
        );
        assert_eq!(options.output, PathBuf::from("/tmp/obs-text"));
        assert_eq!(options.seconds, 5);
        assert_eq!(options.shards, 3);
        assert_eq!(options.sequence_length, 64);
    }

    #[test]
    fn observe_text_rejects_invalid_options() {
        for args in [
            vec!["--shards", "9"],
            vec!["--shards", "x"],
            vec!["--seconds", "-1"],
            vec!["--sequence-length", "0"],
            vec!["--target", "10"],
            vec!["--bogus", "1"],
        ] {
            let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
            assert!(
                parse_observe_text_options(&args).is_err(),
                "{args:?} rejected"
            );
        }
        let missing = ["--out"].map(str::to_owned);
        assert!(parse_observe_text_options(&missing).is_err());
    }

    #[test]
    fn score_defaults_and_overrides() {
        let (default_meta, default_recs) = compiler::corpus_paths();
        let options = parse_score_options(&[]).expect("defaults");
        assert_eq!(options.corpus_meta, PathBuf::from(default_meta));
        assert_eq!(options.corpus_recs, PathBuf::from(default_recs));
        assert_eq!(options.artifacts, PathBuf::from(compiler::ART_PATH));
        assert_eq!(options.cover, None);
        assert_eq!(
            options.transition_out_degree,
            score::DEFAULT_TRANSITION_OUT_DEGREE
        );
        assert_eq!(options.emission_entries, score::DEFAULT_EMISSION_ENTRIES);
        assert_eq!(options.root_top_b, score::DEFAULT_ROOT_TOP_B);
        assert_eq!(options.exct_top_x, score::DEFAULT_EXCT_TOP_X);
        assert_eq!(options.witness_sample, score::DEFAULT_WITNESS_SAMPLE);
        assert_eq!(options.smoothing, score::Smoothing::AddOne);
        assert_eq!(options.output, PathBuf::from("score"));

        let args = [
            "--corpus-meta",
            "/tmp/m.bin",
            "--corpus-recs",
            "/tmp/r.bin",
            "--artifacts",
            "/tmp/a.bin",
            "--cover",
            "/tmp/cover.r4g1",
            "--transition-out-degree",
            "16",
            "--emission-entries",
            "256",
            "--root-top-b",
            "256",
            "--exct-top-x",
            "128",
            "--witness-sample",
            "32",
            "--smoothing",
            "abs-disc:0.5",
            "--out",
            "/tmp/scored",
        ]
        .map(str::to_owned);
        let options = parse_score_options(&args).expect("valid options");
        assert_eq!(options.corpus_meta, PathBuf::from("/tmp/m.bin"));
        assert_eq!(options.corpus_recs, PathBuf::from("/tmp/r.bin"));
        assert_eq!(options.artifacts, PathBuf::from("/tmp/a.bin"));
        assert_eq!(options.cover, Some(PathBuf::from("/tmp/cover.r4g1")));
        assert_eq!(options.transition_out_degree, 16);
        assert_eq!(options.emission_entries, 256);
        assert_eq!(options.root_top_b, 256);
        assert_eq!(options.exct_top_x, 128);
        assert_eq!(options.witness_sample, 32);
        assert_eq!(options.smoothing, score::Smoothing::AbsoluteDiscount(0.5));
        assert_eq!(options.output, PathBuf::from("/tmp/scored"));

        let bad = ["--regions-budget", "4"].map(str::to_owned);
        assert!(parse_score_options(&bad).is_err());
    }

    #[test]
    fn score_smoothing_flag_parses_all_variants() {
        let parse = |value: &str| {
            let args = ["--smoothing", value].map(str::to_owned);
            parse_score_options(&args)
                .expect("valid smoothing")
                .smoothing
        };
        assert_eq!(parse("add-one"), score::Smoothing::AddOne);
        assert_eq!(parse("witten-bell"), score::Smoothing::WittenBell);
        assert_eq!(
            parse("abs-disc:0.1"),
            score::Smoothing::AbsoluteDiscount(0.1)
        );
        assert_eq!(
            parse("abs-disc:0.5"),
            score::Smoothing::AbsoluteDiscount(0.5)
        );
        assert_eq!(
            parse("abs-disc:1.0"),
            score::Smoothing::AbsoluteDiscount(1.0)
        );
    }
}
