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
pub mod route_fit;
pub mod routing;
pub mod semantic_emission_decoupling;
pub mod semantic_state;
pub mod stage_dag;
pub mod trace_profile;

use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::hf_bpe::TokenizerAdapterKey;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::hf_bpe::{TokenizerAdapter, resolve_source_tokenizer};
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_core::transformerless::scenarios::{
    self as runtime_tokenizer, RuntimeTokenizerDecodeTable,
};
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;

pub struct GraphCompileOptions {
    pub corpus_meta: PathBuf,
    pub corpus_recs: PathBuf,
    pub artifacts: PathBuf,
    /// Exact deployed `tokenizer.bin` whose BLAKE3 CID is bound into HEAD.
    /// Absence retains the legacy zero-CID artifact contract.
    pub tokenizer: Option<PathBuf>,
    pub depths: usize,
    pub k0: usize,
    pub regions_budget: usize,
    pub memory_budget_mb: u64,
    pub jobs: Option<usize>,
    pub output: PathBuf,
    /// Root κ of the #597 source-snapshot manifest of the teacher source
    /// (`--source-manifest-kappa`), carried verbatim into the compile
    /// report. This crate never computes the κ itself (no uor-addr
    /// dependency); callers that hold the snapshot pass the string.
    pub source_manifest_kappa: Option<String>,
    /// #600 typed geometry-projection record of the teacher source
    /// (`--geometry-projection`, a JSON serialization of
    /// [`uor_r4_model_source::geometry::GeometryProjection`]), carried
    /// into the compile report. This crate never derives the record
    /// itself; the pipeline that held the oracle passes it.
    pub geometry: Option<uor_r4_model_source::geometry::GeometryProjection>,
    /// #602 typed attention-operator record of the teacher source
    /// (`--attention-operator`, a JSON serialization of
    /// [`uor_r4_model_source::attention::AttentionOperatorSpec`]),
    /// carried into the compile report. This crate never derives the
    /// record itself; the pipeline that held the oracle (or its
    /// `r4_attention` switch) passes it.
    pub attention_operator: Option<uor_r4_model_source::attention::AttentionOperatorSpec>,
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
        tokenizer: None,
        depths: induction::DEFAULT_DEPTHS,
        k0: induction::DEFAULT_K0,
        regions_budget: induction::DEFAULT_REGIONS_BUDGET,
        memory_budget_mb: induction::DEFAULT_MEMORY_BUDGET_MB,
        jobs: None,
        output: PathBuf::from("r4g1_output"),
        source_manifest_kappa: None,
        geometry: None,
        attention_operator: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args.get(index + 1)?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--tokenizer" => options.tokenizer = Some(PathBuf::from(value)),
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
            "--source-manifest-kappa" => {
                options.source_manifest_kappa = Some(value.clone());
            }
            "--geometry-projection" => {
                options.geometry = Some(serde_json::from_str(value).ok()?);
            }
            "--attention-operator" => {
                options.attention_operator = Some(serde_json::from_str(value).ok()?);
            }
            _ => return None,
        }
        index += 2;
    }
    Some(options)
}

#[cfg(not(target_arch = "wasm32"))]
fn explicit_tokenizer_cid(path: Option<&std::path::Path>) -> Result<[u8; 32], SourceUnavailable> {
    let Some(path) = path else {
        return Ok([0; 32]);
    };
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{}: explicit --tokenizer input is not a regular file",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    Ok(*blake3::hash(&bytes).as_bytes())
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
    // Resolve the explicitly named tokenizer before any long work or output
    // mutation. A missing, dangling, directory, or other non-regular path is a
    // hard error; only flag absence selects the legacy zero-CID contract.
    let tokenizer_cid = explicit_tokenizer_cid(options.tokenizer.as_deref())?;
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
    let (artifact_bytes, info) = induction::emit_r4g1_with_tokenizer_cid(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &induced.cover,
        &edges,
        &prior,
        &train,
        tokenizer_cid,
    )
    .map_err(|bound| {
        SourceUnavailable::new(format!(
            "a token or count exceeded the i32 R4G1 wire bound: {bound}"
        ))
    })?;
    let mut report = induction::build_report(
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
    // #597: bind the source-snapshot identity into the compile report when
    // the caller passed it (`--source-manifest-kappa`).
    report.source_manifest_kappa = options.source_manifest_kappa.clone();
    // #600: bind the teacher's geometry-projection record when the caller
    // passed it (`--geometry-projection`).
    report.geometry = options.geometry.clone();
    // #602: bind the teacher's attention-operator record when the caller
    // passed it (`--attention-operator`).
    report.attention_operator = options.attention_operator.clone();

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
    /// Exact registered source-tokenizer selection. Both key components are
    /// present or the selection is absent; legacy checkpoint observations do
    /// not accept a registered adapter.
    pub tokenizer_adapter: Option<TokenizerAdapterKey>,
    pub output: std::path::PathBuf,
    pub seconds: u64,
    pub target: usize,
    pub shards: u8,
    pub sequence_length: usize,
    /// Root κ of the #597 source-snapshot manifest of the teacher source
    /// (`--source-manifest-kappa`), recorded in the observation manifest.
    /// Carried as an opaque string; this crate never computes it.
    pub source_manifest_kappa: Option<String>,
    /// #603 teacher-trace profile (`--trace-profile <id>` or
    /// `<id>/<version>`), resolved through the versioned registry
    /// ([`trace_profile::profile_spec`]) at run time. `None` — the
    /// default everywhere — is the minimal profile: exactly today's
    /// observation bytes. Richer profiles are strictly opt-in.
    pub trace_profile: Option<(String, u32)>,
    /// Declared capture layer indices for the richer #603 lanes
    /// (`--trace-layers <csv>`; bounded by the registry).
    pub trace_layers: Vec<u32>,
    /// Declared per-head attention-support cap (`--trace-support <n>`;
    /// bounded by the registry).
    pub trace_support: u32,
}

/// Parse observe CLI arguments. `None` when the arguments do not name a valid
/// option set: a flag with no value, an unparseable numeric value, an unknown
/// flag, or neither `--source` nor `--checkpoint` (R5 — the absence of a
/// product rather than a raised error).
pub fn parse_observe_options(args: &[String]) -> Option<ObserveOptions> {
    let mut tokenizer_family = None;
    let mut tokenizer_version = None;
    let mut options = ObserveOptions {
        source: None,
        checkpoint: None,
        tokenizer_adapter: None,
        output: std::path::PathBuf::from("observe_output"),
        seconds: 300,
        target: 20_000,
        shards: 3,
        sequence_length: 128,
        source_manifest_kappa: None,
        trace_profile: None,
        trace_layers: Vec::new(),
        trace_support: trace_profile::PRIMARY_TOP_K,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args.get(index + 1)?;
        match flag.as_str() {
            "--source" => options.source = Some(std::path::PathBuf::from(value)),
            "--checkpoint" => options.checkpoint = Some(std::path::PathBuf::from(value)),
            "--tokenizer-family" => tokenizer_family = Some(value.clone()),
            "--tokenizer-version" => tokenizer_version = Some(value.parse().ok()?),
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
            "--source-manifest-kappa" => {
                options.source_manifest_kappa = Some(value.clone());
            }
            "--trace-profile" => {
                // `<id>` (registry version 1) or `<id>/<version>`.
                options.trace_profile = Some(match value.split_once('/') {
                    Some((id, version)) => (id.to_owned(), version.parse().ok()?),
                    None => (value.clone(), trace_profile::PROFILE_VERSION),
                });
            }
            "--trace-layers" => {
                options.trace_layers = value
                    .split(',')
                    .map(|index| index.trim().parse().ok())
                    .collect::<Option<Vec<u32>>>()?;
            }
            "--trace-support" => {
                options.trace_support = value.parse().ok()?;
            }
            _ => return None,
        }
        index += 2;
    }
    if options.source.is_none() && options.checkpoint.is_none() {
        return None;
    }
    options.tokenizer_adapter = match (tokenizer_family, tokenizer_version) {
        (Some(family), Some(version)) => Some(TokenizerAdapterKey::new(family, version)),
        (None, None) => None,
        _ => return None,
    };
    if options.checkpoint.is_some() && options.tokenizer_adapter.is_some() {
        return None;
    }
    Some(options)
}

#[cfg(not(target_arch = "wasm32"))]
struct RegisteredObservationTokenizer {
    adapter: TokenizerAdapter,
    runtime_table: RuntimeTokenizerDecodeTable,
}

/// Resolve one registered source model before the teacher or resumable output
/// is touched. The adapter record and runtime table are materialized from this
/// same model, so manifest identity and tokenizer export cannot select
/// different definitions from an ambiguous source tree.
#[cfg(not(target_arch = "wasm32"))]
fn resolve_registered_observation_tokenizer(
    source: &std::path::Path,
    selection: Option<&TokenizerAdapterKey>,
) -> Result<RegisteredObservationTokenizer, SourceUnavailable> {
    let tokenizer = resolve_source_tokenizer(source, selection)?;
    let adapter = tokenizer.adapter().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no adapter identity")
    })?;
    let runtime_table = tokenizer.runtime_decode_table().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no runtime decode table")
    })?;
    Ok(RegisteredObservationTokenizer {
        adapter,
        runtime_table,
    })
}

/// Fast read-only tokenizer-era check used before loading an expensive
/// teacher. The later setter repeats the full registry/digest/payload audit at
/// the mutation boundary; this check makes already-pinned mismatches fail
/// without either a teacher load or an output write.
#[cfg(not(target_arch = "wasm32"))]
fn preflight_recorded_tokenizer_identity(
    output: &std::path::Path,
    shard_bits: u8,
    requested: Option<&TokenizerAdapter>,
) -> Result<(), SourceUnavailable> {
    let Some(manifest) = observation::ObservationManifest::load(output)? else {
        return Ok(());
    };
    if manifest.shard_bits != shard_bits {
        return Err(SourceUnavailable::new(format!(
            "manifest shard_bits {} does not match requested {shard_bits}",
            manifest.shard_bits
        )));
    }
    match (manifest.tokenizer_adapter.as_ref(), requested) {
        (Some(recorded), Some(requested)) if recorded != requested => {
            Err(SourceUnavailable::new(format!(
                "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested {}/{} (CID {}, digest {}); incompatible resume refused before mutation",
                output.display(),
                recorded.family,
                recorded.version,
                recorded.tokenizer_cid,
                recorded.adapter_digest,
                requested.family,
                requested.version,
                requested.tokenizer_cid,
                requested.adapter_digest,
            )))
        }
        (Some(recorded), None) => Err(SourceUnavailable::new(format!(
            "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested the adapterless legacy tokenizer; incompatible resume refused before mutation",
            output.display(),
            recorded.family,
            recorded.version,
            recorded.tokenizer_cid,
            recorded.adapter_digest,
        ))),
        _ => Ok(()),
    }
}

/// Pin the tokenizer era before any other manifest field, tokenizer export,
/// shard reconciliation, or checkpoint can change. Resume is symmetric:
/// registered output refuses an adapterless legacy request, while
/// [`ObservationShardWriter::set_tokenizer_adapter`] refuses to relabel an
/// adapterless payload and validates the full registered identity.
#[cfg(not(target_arch = "wasm32"))]
fn preflight_observation_tokenizer(
    output: &std::path::Path,
    shard_bits: u8,
    requested: Option<&TokenizerAdapter>,
) -> Result<(), SourceUnavailable> {
    let mut writer = observation::ObservationShardWriter::open(output, shard_bits)?;
    match (writer.manifest().tokenizer_adapter.as_ref(), requested) {
        (Some(recorded), None) => Err(SourceUnavailable::new(format!(
            "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested the adapterless legacy tokenizer; incompatible resume refused before mutation",
            output.display(),
            recorded.family,
            recorded.version,
            recorded.tokenizer_cid,
            recorded.adapter_digest,
        ))),
        (_, Some(adapter)) => writer.set_tokenizer_adapter(adapter),
        (None, None) => Ok(()),
    }
}

/// Read-only resume check for the other source identities this command owns.
/// A different explicitly requested value is an incompatible observation era;
/// an omitted optional value preserves a recorded value for legacy callers.
#[cfg(not(target_arch = "wasm32"))]
fn preflight_observation_metadata(
    output: &std::path::Path,
    shard_bits: u8,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&uor_r4_model_source::attention::AttentionOperatorSpec>,
    trace_profile: Option<&trace_profile::TraceProfile>,
) -> Result<(), SourceUnavailable> {
    let Some(recorded) = observation::ObservationManifest::load(output)? else {
        return Ok(());
    };
    if recorded.shard_bits != shard_bits {
        return Err(SourceUnavailable::new(format!(
            "manifest shard_bits {} does not match requested {shard_bits}",
            recorded.shard_bits
        )));
    }
    if let (Some(recorded), Some(requested)) = (
        recorded.source_manifest_kappa.as_deref(),
        source_manifest_kappa,
    ) && recorded != requested
    {
        return Err(SourceUnavailable::new(format!(
            "{} is pinned to source manifest κ {recorded}; requested {requested}; incompatible observation resume refused before mutation",
            output.display()
        )));
    }
    if let (Some(recorded), Some(requested)) = (recorded.geometry.as_ref(), geometry)
        && recorded != requested
    {
        return Err(SourceUnavailable::new(format!(
            "{} is pinned to geometry {}/{} digest {}; requested {}/{} digest {}; incompatible observation resume refused before mutation",
            output.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
            requested.id,
            requested.version,
            requested.declared_digest(),
        )));
    }
    if let (Some(recorded), Some(requested)) =
        (recorded.attention_operator.as_ref(), attention_operator)
        && recorded != requested
    {
        return Err(SourceUnavailable::new(format!(
            "{} is pinned to attention operator {}/{} digest {}; requested {}/{} digest {}; incompatible observation resume refused before mutation",
            output.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
            requested.id,
            requested.version,
            requested.declared_digest(),
        )));
    }
    let state_exists = match std::fs::symlink_metadata(output.join(observation::STATE_FILE)) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(SourceUnavailable::from(error)),
    };
    match (recorded.trace_profile.as_ref(), trace_profile) {
        (None, None) => {}
        (Some(recorded), Some(requested)) if recorded == requested => {}
        (None, Some(requested)) if state_exists || !recorded.completed.is_empty() => {
            return Err(SourceUnavailable::new(format!(
                "{} was captured under the minimal trace profile; profile {}/{} cannot be introduced mid-corpus; incompatible observation resume refused before mutation",
                output.display(),
                requested.id,
                requested.version,
            )));
        }
        (None, Some(_)) => {}
        (Some(recorded), Some(requested)) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to trace profile {}/{}; requested {}/{}; incompatible observation resume refused before mutation",
                output.display(),
                recorded.id,
                recorded.version,
                requested.id,
                requested.version,
            )));
        }
        (Some(recorded), None) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to trace profile {}/{}; pass the same profile to resume; incompatible observation resume refused before mutation",
                output.display(),
                recorded.id,
                recorded.version,
            )));
        }
    }
    Ok(())
}

/// Check all already-recorded identities without writing, then make the
/// tokenizer pin the first mutation of a compatible run.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn preflight_observation_identities(
    output: &std::path::Path,
    shard_bits: u8,
    source_manifest_kappa: Option<&str>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&uor_r4_model_source::attention::AttentionOperatorSpec>,
    trace_profile: Option<&trace_profile::TraceProfile>,
    tokenizer_adapter: Option<&TokenizerAdapter>,
) -> Result<(), SourceUnavailable> {
    preflight_observation_metadata(
        output,
        shard_bits,
        source_manifest_kappa,
        geometry,
        attention_operator,
        trace_profile,
    )?;
    preflight_observation_tokenizer(output, shard_bits, tokenizer_adapter)
}

#[cfg(not(target_arch = "wasm32"))]
fn export_registered_observation_tokenizer(
    resolved: &RegisteredObservationTokenizer,
    output: &std::path::Path,
) -> Result<Option<Vec<u32>>, SourceUnavailable> {
    runtime_tokenizer::export_runtime_tokenizer_table(
        &resolved.runtime_table,
        output.join("tokenizer.bin"),
    )
    .map(|export| export.source_byte_lengths)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn observe(args: &[String]) -> Result<(), SourceUnavailable> {
    let options = parse_observe_options(args).ok_or_else(|| {
        SourceUnavailable::new("could not parse observe options from the provided arguments")
    })?;
    let requested_trace_profile = options
        .trace_profile
        .as_ref()
        .map(|(id, version)| {
            trace_profile::profile_spec(
                id,
                *version,
                &trace_profile::TraceCaptureBounds {
                    layer_indices: options.trace_layers.clone(),
                    support_size: options.trace_support,
                },
            )
        })
        .transpose()?
        .filter(|profile| !profile.is_minimal());

    // Resolve the source definition without touching resumable output.
    // Ambiguity and unknown registry versions therefore leave every
    // pre-existing byte unchanged.
    let registered_tokenizer = if options.checkpoint.is_some() {
        None
    } else {
        let source = options.source.as_deref().ok_or_else(|| {
            SourceUnavailable::new("a source directory is required without --checkpoint")
        })?;
        Some(resolve_registered_observation_tokenizer(
            source,
            options.tokenizer_adapter.as_ref(),
        )?)
    };
    preflight_recorded_tokenizer_identity(
        &options.output,
        options.shards,
        registered_tokenizer
            .as_ref()
            .map(|resolved| &resolved.adapter),
    )?;
    let mut oracle: Box<dyn uor_r4_model_source::TeacherOracle> =
        if let Some(ref ckpt) = options.checkpoint {
            let path = ckpt
                .to_str()
                .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
            Box::new(uor_r4_model_source::LlamaOracle::load(path))
        } else {
            let source = options.source.as_deref().ok_or_else(|| {
                SourceUnavailable::new("a source directory is required without --checkpoint")
            })?;
            Box::new(uor_r4_model_source::Teacher::load_with_sequence_length(
                source,
                options.sequence_length,
            )?)
        };

    // #597: record the source-snapshot identity — and, since #600/#602,
    // the typed geometry-projection and attention-operator records the
    // oracle itself declares — in the observation manifest before the
    // sharded run opens it (idempotent, atomic; the run's own writer
    // loads and preserves the stored fields).
    let geometry = oracle.geometry_projection();
    let attention_operator = oracle.attention_operator_spec();
    preflight_observation_identities(
        &options.output,
        options.shards,
        options.source_manifest_kappa.as_deref(),
        geometry.as_ref(),
        attention_operator.as_ref(),
        requested_trace_profile.as_ref(),
        registered_tokenizer
            .as_ref()
            .map(|resolved| &resolved.adapter),
    )?;
    if options.source_manifest_kappa.is_some() || geometry.is_some() || attention_operator.is_some()
    {
        let mut writer =
            observation::ObservationShardWriter::open(&options.output, options.shards)?;
        if let Some(kappa) = &options.source_manifest_kappa {
            writer.set_source_manifest_kappa(kappa)?;
        }
        if let Some(geometry) = &geometry {
            writer.set_geometry(geometry)?;
        }
        if let Some(operator) = &attention_operator {
            writer.set_attention_operator(operator)?;
        }
    }

    // Export only after every recorded source/geometry/attention/tokenizer
    // identity has passed and its setter has completed. The same resolved
    // model supplies the deployed decode table and source-anchor declaration.
    // Byte BPE carries its historical lengths; SentencePiece deliberately
    // returns None because normalization and unknown collapse cannot define
    // original-input offsets.
    let token_byte_lengths = registered_tokenizer
        .as_ref()
        .map(|resolved| export_registered_observation_tokenizer(resolved, &options.output))
        .transpose()?
        .flatten();

    // #603: resolve the opt-in teacher-trace profile through the
    // versioned registry — an unknown (profile, version) or unbounded
    // declaration is refused by name, never guessed. No flag means the
    // minimal profile: exactly today's observation bytes.
    match &requested_trace_profile {
        None => {
            observation::observe_sharded(
                &mut *oracle,
                options.seconds,
                options.target,
                options.shards,
                &options.output,
                token_byte_lengths.as_deref(),
            )?;
        }
        Some(profile) => {
            observation::observe_sharded_traced(
                &mut *oracle,
                options.seconds,
                options.target,
                options.shards,
                &options.output,
                token_byte_lengths.as_deref(),
                profile,
            )?;
        }
    }
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tokenizer_observe_tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use uor_r4_core::transformerless::scenarios::Tokenizer;

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn unique_path(label: &str) -> PathBuf {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "uor-r4-graph-observe-tokenizer-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    fn write_bpe_source(label: &str, marker: &str) -> PathBuf {
        let source = unique_path(label);
        std::fs::create_dir_all(&source).expect("create BPE source fixture");
        let json = format!(
            r#"{{
                "fixture_marker":"{marker}",
                "pre_tokenizer":{{"type":"ByteLevel","add_prefix_space":false}},
                "model":{{
                    "type":"BPE",
                    "vocab":{{"a":0,"b":1,"ab":2}},
                    "merges":["a b"]
                }}
            }}"#
        );
        std::fs::write(source.join("tokenizer.json"), json).expect("write BPE definition");
        source
    }

    fn directory_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn visit(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut entries = std::fs::read_dir(directory)
                .expect("read fixture directory")
                .map(|entry| entry.expect("read fixture entry").path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    visit(root, &path, files);
                } else {
                    files.push((
                        path.strip_prefix(root)
                            .expect("relative fixture path")
                            .to_owned(),
                        std::fs::read(&path).expect("read fixture file"),
                    ));
                }
            }
        }

        let mut files = Vec::new();
        if root.is_dir() {
            visit(root, root, &mut files);
        }
        files
    }

    fn synthetic_spiece_model() -> Vec<u8> {
        fn varint(out: &mut Vec<u8>, mut value: u64) {
            loop {
                let mut byte = (value & 0x7f) as u8;
                value >>= 7;
                if value != 0 {
                    byte |= 0x80;
                }
                out.push(byte);
                if value == 0 {
                    break;
                }
            }
        }
        fn tag(out: &mut Vec<u8>, field: u64, wire: u8) {
            varint(out, (field << 3) | u64::from(wire));
        }
        fn len_delim(out: &mut Vec<u8>, field: u64, payload: &[u8]) {
            tag(out, field, 2);
            varint(out, payload.len() as u64);
            out.extend_from_slice(payload);
        }

        // Surface, score, ModelProto SentencePiece type. The non-vacuous
        // fixture includes UNKNOWN, CONTROL, and UNUSED pieces alongside
        // ordinary pieces; USER_DEFINED is deliberately excluded because /1
        // refuses its pre-normalization atomic-matching semantics.
        let pieces = [
            ("<unk>", 0.0f32, 2u64),
            ("\u{2581}", -3.0, 1),
            ("\u{2581}a", -1.0, 1),
            ("a", -2.0, 1),
            ("b", -2.0, 1),
            ("<s>", 0.0, 3),
            ("<unused>", 0.0, 5),
        ];
        let mut proto = Vec::new();
        for (surface, score, kind) in pieces {
            let mut piece = Vec::new();
            len_delim(&mut piece, 1, surface.as_bytes());
            tag(&mut piece, 2, 5);
            piece.extend_from_slice(&score.to_bits().to_le_bytes());
            tag(&mut piece, 3, 0);
            varint(&mut piece, kind);
            len_delim(&mut proto, 1, &piece);
        }
        let mut trainer = Vec::new();
        tag(&mut trainer, 3, 0);
        varint(&mut trainer, 1); // UNIGRAM
        tag(&mut trainer, 40, 0);
        varint(&mut trainer, 0); // unk_id
        len_delim(&mut proto, 2, &trainer);

        // Minimal identity precompiled charsmap: zero trie units plus a NUL
        // replacement byte. Default whitespace/dummy-prefix policy remains on.
        let mut normalizer = Vec::new();
        len_delim(&mut normalizer, 1, b"identity");
        len_delim(&mut normalizer, 2, &[0, 0, 0, 0, 0]);
        len_delim(&mut proto, 3, &normalizer);
        proto
    }

    #[test]
    fn graph_compile_tokenizer_flag_hashes_exact_regular_bytes_and_fails_closed() {
        let tokenizer = unique_path("graph-compile-tokenizer.bin");
        std::fs::write(&tokenizer, b"exact runtime tokenizer bytes").expect("write tokenizer");
        let options = parse_options(
            &["--tokenizer", tokenizer.to_str().expect("UTF-8 path")].map(str::to_owned),
        )
        .expect("tokenizer flag parses");
        assert_eq!(options.tokenizer.as_deref(), Some(tokenizer.as_path()));
        assert_eq!(
            explicit_tokenizer_cid(options.tokenizer.as_deref()).expect("hash tokenizer"),
            *blake3::hash(b"exact runtime tokenizer bytes").as_bytes()
        );
        assert_eq!(
            explicit_tokenizer_cid(None).expect("legacy absence"),
            [0; 32]
        );

        let missing = unique_path("missing-graph-compile-tokenizer.bin");
        let missing_error =
            explicit_tokenizer_cid(Some(&missing)).expect_err("missing is an error");
        assert!(
            missing_error
                .reason
                .contains(&missing.display().to_string())
        );

        let directory = unique_path("graph-compile-tokenizer-directory");
        std::fs::create_dir_all(&directory).expect("create directory");
        let directory_error =
            explicit_tokenizer_cid(Some(&directory)).expect_err("directory is an error");
        assert!(directory_error.reason.contains("not a regular file"));

        let _ = std::fs::remove_file(tokenizer);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn observe_parser_requires_an_atomic_registered_key_and_rejects_legacy_mix() {
        let valid = parse_observe_options(
            &[
                "--source",
                "/tmp/source",
                "--tokenizer-family",
                "sentencepiece-unigram",
                "--tokenizer-version",
                "1",
            ]
            .map(str::to_owned),
        )
        .expect("paired selection parses");
        assert_eq!(
            valid.tokenizer_adapter,
            Some(TokenizerAdapterKey::sentencepiece_unigram_v1())
        );
        assert!(
            parse_observe_options(
                &[
                    "--source",
                    "/tmp/source",
                    "--tokenizer-family",
                    "hf-byte-bpe"
                ]
                .map(str::to_owned)
            )
            .is_none()
        );
        assert!(
            parse_observe_options(
                &["--source", "/tmp/source", "--tokenizer-version", "1"].map(str::to_owned)
            )
            .is_none()
        );
        assert!(
            parse_observe_options(
                &[
                    "--checkpoint",
                    "/tmp/checkpoint",
                    "--tokenizer-family",
                    "hf-byte-bpe",
                    "--tokenizer-version",
                    "1",
                ]
                .map(str::to_owned)
            )
            .is_none()
        );
    }

    #[test]
    fn graph_observe_refuses_ambiguous_and_unknown_sources_before_output() {
        let source = write_bpe_source("ambiguous-source", "ambiguous");
        std::fs::write(source.join("spiece.model"), b"selected model is malformed")
            .expect("write second definition");
        let ambiguous_output = unique_path("ambiguous-output");
        let ambiguous = observe(
            &[
                "--source",
                source.to_str().unwrap(),
                "--out",
                ambiguous_output.to_str().unwrap(),
            ]
            .map(str::to_owned),
        )
        .expect_err("ambiguous source must fail closed");
        assert!(
            ambiguous
                .reason
                .contains("both tokenizer.json and spiece.model")
        );
        assert!(!ambiguous_output.exists());

        let unknown_source = write_bpe_source("unknown-source", "unknown");
        let unknown_output = unique_path("unknown-output");
        let unknown = observe(
            &[
                "--source",
                unknown_source.to_str().unwrap(),
                "--out",
                unknown_output.to_str().unwrap(),
                "--tokenizer-family",
                "hf-byte-bpe",
                "--tokenizer-version",
                "99",
            ]
            .map(str::to_owned),
        )
        .expect_err("unknown adapter version must fail by name");
        assert!(unknown.reason.contains("hf-byte-bpe/99"), "{unknown}");
        assert!(!unknown_output.exists());

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(unknown_source);
    }

    #[test]
    fn graph_observe_registered_and_adapterless_resume_mismatches_are_atomic() {
        let first_source = write_bpe_source("resume-first", "first");
        let second_source = write_bpe_source("resume-second", "second");
        let first = resolve_registered_observation_tokenizer(
            &first_source,
            Some(&TokenizerAdapterKey::hf_byte_bpe_v1()),
        )
        .expect("first source resolves");
        let output = unique_path("resume-output");
        preflight_observation_tokenizer(&output, 2, Some(&first.adapter))
            .expect("first adapter pin");
        {
            let mut writer =
                observation::ObservationShardWriter::open(&output, 2).expect("open pinned writer");
            writer
                .set_source_manifest_kappa("blake3:original")
                .expect("write unrelated provenance");
        }
        std::fs::write(output.join("sentinel.bin"), b"preserve me")
            .expect("write preservation sentinel");
        let before = directory_bytes(&output);

        let registered_error = observe(
            &[
                "--source",
                second_source.to_str().unwrap(),
                "--out",
                output.to_str().unwrap(),
                "--shards",
                "2",
                "--tokenizer-family",
                "hf-byte-bpe",
                "--tokenizer-version",
                "1",
                "--source-manifest-kappa",
                "blake3:replacement",
            ]
            .map(str::to_owned),
        )
        .expect_err("different registered identity must fail before teacher load");
        assert!(
            registered_error.reason.contains("incompatible resume"),
            "{registered_error}"
        );
        assert_eq!(directory_bytes(&output), before);

        let legacy_error = observe(
            &[
                "--checkpoint",
                "/path/that/must/not/be-loaded.bin",
                "--out",
                output.to_str().unwrap(),
                "--shards",
                "2",
                "--source-manifest-kappa",
                "blake3:replacement",
            ]
            .map(str::to_owned),
        )
        .expect_err("adapterless request must not resume registered output");
        assert!(legacy_error.reason.contains("adapterless legacy tokenizer"));
        assert_eq!(directory_bytes(&output), before);

        let legacy_output = unique_path("adapterless-payload");
        drop(
            observation::ObservationShardWriter::open(&legacy_output, 2)
                .expect("create adapterless writer"),
        );
        std::fs::write(
            legacy_output.join(observation::STATE_FILE),
            b"legacy checkpoint",
        )
        .expect("write tokenizer-era state");
        let legacy_before = directory_bytes(&legacy_output);
        let relabel_error = preflight_observation_identities(
            &legacy_output,
            2,
            None,
            None,
            None,
            None,
            Some(&first.adapter),
        )
        .expect_err("registered request cannot relabel adapterless payload");
        assert!(
            relabel_error
                .reason
                .contains("no recorded tokenizer adapter")
        );
        assert_eq!(directory_bytes(&legacy_output), legacy_before);

        let _ = std::fs::remove_dir_all(first_source);
        let _ = std::fs::remove_dir_all(second_source);
        let _ = std::fs::remove_dir_all(output);
        let _ = std::fs::remove_dir_all(legacy_output);
    }

    #[test]
    fn graph_observe_metadata_or_trace_mismatch_cannot_overwrite_tokenizer_export() {
        let source = write_bpe_source("metadata-ordering-source", "ordering");
        let resolved = resolve_registered_observation_tokenizer(
            &source,
            Some(&TokenizerAdapterKey::hf_byte_bpe_v1()),
        )
        .expect("source resolves");
        let output = unique_path("metadata-ordering-output");
        let recorded_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(576, 288);
        let requested_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(768, 288);
        let attention = uor_r4_model_source::attention::AttentionOperatorSpec::standard();
        let trace = trace_profile::profile_spec(
            trace_profile::LAYER_PROFILE,
            trace_profile::PROFILE_VERSION,
            &trace_profile::TraceCaptureBounds {
                layer_indices: vec![0],
                support_size: trace_profile::PRIMARY_TOP_K,
            },
        )
        .expect("registered trace profile");
        {
            let mut writer = observation::ObservationShardWriter::open(&output, 2)
                .expect("create observation writer");
            writer
                .set_tokenizer_adapter(&resolved.adapter)
                .expect("pin tokenizer");
            writer
                .set_geometry(&recorded_geometry)
                .expect("pin geometry");
            writer
                .set_attention_operator(&attention)
                .expect("pin attention");
            writer.set_trace_profile(&trace).expect("pin trace profile");
        }
        std::fs::write(output.join("tokenizer.bin"), b"original tokenizer bytes")
            .expect("write existing tokenizer export");
        let before = directory_bytes(&output);

        let error = preflight_observation_identities(
            &output,
            2,
            None,
            Some(&requested_geometry),
            Some(&attention),
            Some(&trace),
            Some(&resolved.adapter),
        )
        .expect_err("geometry mismatch must precede tokenizer publication");
        assert!(
            error.reason.contains("geometry")
                && error.reason.contains("incompatible observation resume")
                && error.reason.contains("before mutation"),
            "{error}"
        );
        assert_eq!(directory_bytes(&output), before);

        let trace_error = preflight_observation_identities(
            &output,
            2,
            None,
            Some(&recorded_geometry),
            Some(&attention),
            None,
            Some(&resolved.adapter),
        )
        .expect_err("trace mismatch must precede tokenizer publication");
        assert!(
            trace_error.reason.contains("trace profile")
                && trace_error
                    .reason
                    .contains("incompatible observation resume")
                && trace_error.reason.contains("before mutation"),
            "{trace_error}"
        );
        assert_eq!(directory_bytes(&output), before);

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn graph_observe_sentencepiece_export_is_decode_only_without_source_anchors() {
        let source = unique_path("spiece-source");
        std::fs::create_dir_all(&source).expect("create SentencePiece source");
        let model = synthetic_spiece_model();
        std::fs::write(source.join("spiece.model"), &model).expect("write spiece.model");

        let resolved = resolve_registered_observation_tokenizer(
            &source,
            Some(&TokenizerAdapterKey::sentencepiece_unigram_v1()),
        )
        .expect("synthetic SentencePiece source resolves");
        assert_eq!(resolved.adapter.family, "sentencepiece-unigram");
        assert_eq!(resolved.adapter.version, 1);
        assert_eq!(
            resolved.adapter.tokenizer_cid,
            format!("blake3:{}", blake3::hash(&model).to_hex())
        );
        assert_eq!(resolved.runtime_table.source_byte_lengths, None);

        let output = unique_path("spiece-output");
        preflight_observation_tokenizer(&output, 2, Some(&resolved.adapter))
            .expect("pin SentencePiece adapter");
        let anchors = export_registered_observation_tokenizer(&resolved, &output)
            .expect("export tagged tokenizer");
        assert_eq!(
            anchors, None,
            "SentencePiece must not fabricate source anchors"
        );
        let bytes = std::fs::read(output.join("tokenizer.bin")).expect("read tokenizer export");
        let runtime = Tokenizer::from_bytes(&bytes).expect("parse tagged runtime tokenizer");
        assert!(runtime.is_decode_only());
        assert_eq!(runtime.adapter_key(), Some(("sentencepiece-unigram", 1)));
        let manifest = observation::ObservationManifest::load(&output)
            .expect("load manifest")
            .expect("manifest exists");
        assert_eq!(manifest.tokenizer_adapter.as_ref(), Some(&resolved.adapter));

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(output);
    }
}
