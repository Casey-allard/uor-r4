pub mod behavioral_probes;
pub mod dependency_audit;
pub mod executor;
pub mod future_state_planner;
#[cfg(feature = "graph-construction")]
pub mod graph;
pub mod induction;
pub mod jobs_config;
pub mod lower_semantic_regions;
pub mod memory_budget;
pub mod monograph;
pub mod observation;
pub mod observation_shards;
#[cfg(not(target_arch = "wasm32"))]
pub mod observation_text;
pub mod pack;
#[cfg(feature = "patch-induction")]
pub mod patch_induction;
#[cfg(feature = "perturbation")]
pub mod perturbation;
pub mod probability_calibration;
pub mod quantum_cover;
pub mod rate_distortion_compression;
pub mod reference_compiler_ir;
pub mod reproducibility;
pub mod residual;
pub mod routing;
pub mod semantic_emission_decoupling;
pub mod semantic_state;
pub mod stage_dag;

use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::compiler;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;

pub struct GraphCompileOptions {
    pub corpus_meta: PathBuf,
    pub corpus_recs: PathBuf,
    pub artifacts: PathBuf,
    pub depths: usize,
    pub k0: usize,
    pub regions_budget: usize,
    pub memory_budget_mb: u64,
    pub jobs: Option<usize>,
    pub output: PathBuf,
}

/// Parse graph-compile CLI arguments. `None` when the arguments do not name a
/// valid option set: a flag with no value, an unparseable or zero numeric
/// value, or an unknown flag. The requested options are not a product of those
/// arguments (R5 — the absence of a product rather than a raised error).
#[cfg(not(target_arch = "wasm32"))]
pub fn parse_options(args: &[String]) -> Option<GraphCompileOptions> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = GraphCompileOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        depths: induction::DEFAULT_DEPTHS,
        k0: induction::DEFAULT_K0,
        regions_budget: induction::DEFAULT_REGIONS_BUDGET,
        memory_budget_mb: induction::DEFAULT_MEMORY_BUDGET_MB,
        jobs: None,
        output: PathBuf::from("r4g1_output"),
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args.get(index + 1)?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--depths" => {
                options.depths = value.parse().ok()?;
                if options.depths == 0 {
                    return None;
                }
            }
            "--k0" => {
                options.k0 = value.parse().ok()?;
                if options.k0 == 0 {
                    return None;
                }
            }
            "--regions-budget" => {
                options.regions_budget = value.parse().ok()?;
                if options.regions_budget == 0 {
                    return None;
                }
            }
            "--memory-budget" => {
                options.memory_budget_mb = value.parse().ok()?;
            }
            "--jobs" => {
                let j: usize = value.parse().ok()?;
                if j == 0 {
                    return None;
                }
                options.jobs = Some(j);
            }
            "--out" => options.output = PathBuf::from(value),
            _ => return None,
        }
        index += 2;
    }
    Some(options)
}

/// Run the full multiresolution graph compilation pipeline (Option 1).
#[cfg(not(target_arch = "wasm32"))]
pub fn compile(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make graph compilation much slower; use `cargo run --release -- graph-compile ...`"
    );
    let options = parse_options(args).ok_or_else(|| {
        SourceUnavailable::new("could not parse graph-compile options from the provided arguments")
    })?;
    let env_jobs = std::env::var("R4_COMPILER_THREADS").ok();
    let jobs_config = jobs_config::CompilerJobsConfig::resolve(options.jobs, env_jobs.as_deref())
        .ok_or_else(|| {
        SourceUnavailable::new(
            "invalid worker thread count (--jobs / R4_COMPILER_THREADS must be a positive integer)",
        )
    })?;
    let _pool = jobs_config.build_dedicated_thread_pool();
    eprintln!(
        "graph-compiler: initialized dedicated thread pool ({} workers, source: {:?})",
        jobs_config.jobs, jobs_config.source
    );
    let corpus_meta = options
        .corpus_meta
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let corpus_recs = options
        .corpus_recs
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    // #450: announce the resolved containers before the long work.
    let artifact_container = std::fs::read(&options.artifacts).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.artifacts.display()))
    })?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            options.artifacts.display()
        ))
    })?;
    let artifact_kappa = reproducibility::container_kappa(&artifact_container);
    reproducibility::announce_teacher_container(&options.artifacts, &artifact_kappa);
    let meta_bytes = std::fs::read(&options.corpus_meta).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.corpus_meta.display()))
    })?;
    let recs_bytes = std::fs::read(&options.corpus_recs).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.corpus_recs.display()))
    })?;
    let corpus_kappa = reproducibility::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    reproducibility::announce_corpus(&options.corpus_meta, &options.corpus_recs, &corpus_kappa);
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            options.corpus_meta.display(),
            options.corpus_recs.display()
        ))
    })?;

    let config = induction::CoverConfig {
        depths: options.depths,
        k0: options.k0,
        regions_budget: options.regions_budget,
        memory_budget_bytes: options.memory_budget_mb * 1024 * 1024,
        ..induction::CoverConfig::default()
    };
    eprintln!(
        "graph-compiler: inducing (depths {}, k0 {}, regions budget {}, memory budget {} MiB)...",
        config.depths, config.k0, config.regions_budget, options.memory_budget_mb
    );
    let (train_positions, held_out_positions) = induction::split_positions(&corpus);
    let train = induction::build_observations(&artifacts, &corpus, &train_positions);
    let held_out = induction::build_observations(&artifacts, &corpus, &held_out_positions);
    let induced = induction::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)
        .ok_or_else(|| {
            SourceUnavailable::new("cover induction needs at least one train observation")
        })?;
    let reference = induction::ReferenceClassifier::freeze(&induced.cover);
    eprintln!(
        "graph-compiler: {} regions across {} depth(s); evaluating held-out routing recall...",
        induced.cover.regions.len(),
        induced.cover.max_depth
    );
    let recall =
        induction::evaluate_held_out(&artifacts, &induced.cover, &reference, &train, &held_out);
    let edges = induction::build_edges(&induced.cover, &reference, &train, &corpus.story);
    let prior = induction::root_prior(&train);
    let vocab = u32::try_from(artifacts.token_codes.len() / compiler::STAGES)
        .map_err(|_| SourceUnavailable::new("vocabulary exceeds u32 token ids"))?;
    let (artifact_bytes, info) = induction::emit_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &induced.cover,
        &edges,
        &prior,
        &train,
    )
    .map_err(|bound| {
        SourceUnavailable::new(format!(
            "a token or count exceeded the i32 R4G1 wire bound: {bound}"
        ))
    })?;
    let report = induction::build_report(
        &config,
        &induced,
        induction::ReportData {
            reference: &reference,
            train: &train,
            held_out: &held_out,
            edges: &edges,
            recall: recall.clone(),
            artifact: Some((&artifact_bytes, info)),
        },
    );

    std::fs::create_dir_all(&options.output)?;
    let artifact_path = options.output.join("compiled.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", artifact_path.display())))?;
    let report_json = serde_json::to_string_pretty(&report)?;
    let report_path = options.output.join("compile_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", report_path.display())))?;

    println!(
        "graph-compiler complete: {} regions ({} splits), {} edges ({} refinement + {} neighbor), depths 1..={}",
        induced.cover.regions.len(),
        report.regions.splits,
        info.edge_count,
        info.refinement_edges,
        info.neighbor_edges,
        induced.cover.max_depth
    );

    Ok(())
}

pub struct ObserveOptions {
    pub source: Option<std::path::PathBuf>,
    pub checkpoint: Option<std::path::PathBuf>,
    pub output: std::path::PathBuf,
    pub seconds: u64,
    pub target: usize,
    pub shards: u8,
    pub sequence_length: usize,
}

/// Parse observe CLI arguments. `None` when the arguments do not name a valid
/// option set: a flag with no value, an unparseable numeric value, an unknown
/// flag, or neither `--source` nor `--checkpoint` (R5 — the absence of a
/// product rather than a raised error).
pub fn parse_observe_options(args: &[String]) -> Option<ObserveOptions> {
    let mut options = ObserveOptions {
        source: None,
        checkpoint: None,
        output: std::path::PathBuf::from("observe_output"),
        seconds: 300,
        target: 20_000,
        shards: 3,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args.get(index + 1)?;
        match flag.as_str() {
            "--source" => options.source = Some(std::path::PathBuf::from(value)),
            "--checkpoint" => options.checkpoint = Some(std::path::PathBuf::from(value)),
            "--out" => options.output = std::path::PathBuf::from(value),
            "--seconds" => {
                options.seconds = value.parse().ok()?;
            }
            "--target" => {
                options.target = value.parse().ok()?;
            }
            "--shards" => {
                options.shards = value.parse().ok()?;
            }
            "--sequence-length" => {
                options.sequence_length = value.parse().ok()?;
            }
            _ => return None,
        }
        index += 2;
    }
    if options.source.is_none() && options.checkpoint.is_none() {
        return None;
    }
    Some(options)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn observe(args: &[String]) -> Result<(), SourceUnavailable> {
    let options = parse_observe_options(args).ok_or_else(|| {
        SourceUnavailable::new("could not parse observe options from the provided arguments")
    })?;

    let mut oracle: Box<dyn uor_r4_model_source::TeacherOracle> =
        if let Some(ref ckpt) = options.checkpoint {
            let o = uor_r4_model_source::LlamaOracle::load(ckpt.to_str().unwrap());
            Box::new(o)
        } else {
            let o = uor_r4_model_source::HuggingFaceLlamaOracle::load_with_sequence_length(
                options.source.as_ref().unwrap(),
                options.sequence_length,
            )?;
            Box::new(o)
        };

    observation::observe_sharded(
        &mut *oracle,
        options.seconds,
        options.target,
        options.shards,
        &options.output,
        None,
    )?;
    Ok(())
}
