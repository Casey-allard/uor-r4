pub mod behavioral_probes;
pub mod future_state_planner;
pub mod graph;
pub mod induction;
pub mod lower_semantic_regions;
pub mod monograph;
pub mod observation;
pub mod observation_text;
pub mod pack;
pub mod patch_induction;
pub mod perturbation;
pub mod quantum_cover;
pub mod rate_distortion_compression;
pub mod reference_compiler_ir;
pub mod residual;
pub mod routing;
pub mod semantic_emission_decoupling;
pub mod semantic_state;

use std::path::PathBuf;
use uor_r4_core::transformerless::compiler;

pub struct GraphCompileOptions {
    pub corpus_meta: PathBuf,
    pub corpus_recs: PathBuf,
    pub artifacts: PathBuf,
    pub depths: usize,
    pub k0: usize,
    pub regions_budget: usize,
    pub memory_budget_mb: u64,
    pub output: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_options(args: &[String]) -> Result<GraphCompileOptions, String> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = GraphCompileOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        depths: induction::DEFAULT_DEPTHS,
        k0: induction::DEFAULT_K0,
        regions_budget: induction::DEFAULT_REGIONS_BUDGET,
        memory_budget_mb: induction::DEFAULT_MEMORY_BUDGET_MB,
        output: PathBuf::from("r4g1_output"),
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
            }
            "--out" => options.output = PathBuf::from(value),
            _ => return Err(format!("unknown graph-compile option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Run the full multiresolution graph compilation pipeline (Option 1).
#[cfg(not(target_arch = "wasm32"))]
pub fn compile(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make graph compilation much slower; use `cargo run --release -- graph-compile ...`"
    );
    let options = parse_options(args)?;
    let corpus_meta = options
        .corpus_meta
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let corpus_recs = options
        .corpus_recs
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            options.corpus_meta.display(),
            options.corpus_recs.display()
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
    let induced = induction::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)?;
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
        .map_err(|_| "vocabulary exceeds u32 token ids".to_owned())?;
    let (artifact_bytes, info) = induction::emit_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &induced.cover,
        &edges,
        &prior,
        &train,
    )?;
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

    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let artifact_path = options.output.join("compiled.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| format!("{}: {error}", artifact_path.display()))?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    let report_path = options.output.join("compile_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;

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

pub fn parse_observe_options(args: &[String]) -> Result<ObserveOptions, String> {
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
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--source" => options.source = Some(std::path::PathBuf::from(value)),
            "--checkpoint" => options.checkpoint = Some(std::path::PathBuf::from(value)),
            "--out" => options.output = std::path::PathBuf::from(value),
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
            }
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
            }
            _ => return Err(format!("unknown observe option: {flag}")),
        }
        index += 2;
    }
    if options.source.is_none() && options.checkpoint.is_none() {
        return Err("must provide either --source or --checkpoint".to_owned());
    }
    Ok(options)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn observe(args: &[String]) -> Result<(), String> {
    let options = parse_observe_options(args)?;

    let mut oracle: Box<dyn uor_r4_model_source::TeacherOracle> =
        if let Some(ref ckpt) = options.checkpoint {
            let o = uor_r4_model_source::LlamaOracle::load(ckpt.to_str().unwrap());
            Box::new(o)
        } else {
            let o = uor_r4_model_source::HuggingFaceLlamaOracle::load_with_sequence_length(
                options.source.as_ref().unwrap(),
                options.sequence_length,
            )
            .map_err(|e| format!("failed to load HF model: {e}"))?;
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
