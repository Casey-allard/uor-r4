//! Typed compile API: one call from a verified local teacher source to a
//! scored deployable R4G1 graph, orchestrating the three existing
//! compiler stages in-process:
//!
//! 1. teacher bundle (`uor-r4-graph-cli::compile_hugging_face_with_progress`)
//!    — tokenizer export, resumable teacher corpus, table-native
//!    artifact, graded store, calibration, manifest;
//! 2. multiresolution cover (`uor-r4-graph-compiler::compile`) —
//!    `compiled.r4g1` + `compile_report.json`;
//! 3. scoring (`uor-r4-graph-cli::score_command`) — `score.r4g1` +
//!    `score_report.json`.
//!
//! COMPAT SHIM: the stage entry points predate this crate and take CLI
//! flag strings. Building those flags from the typed request is the
//! sanctioned translation layer; it lives entirely in the private
//! `stage_*_flags` functions below and goes away when the stages grow
//! typed entry points. No stage stdout is parsed anywhere: the only
//! resumability signal is the structural corpus-completion check in
//! [`corpus_complete`], which mirrors the gate the corpus loader
//! applies (`uor-r4-core`'s `transformerless::compiler::load_corpus_from`).
//!
//! All stage I/O stays inside the caller-supplied work directory; the
//! returned [`CompiledModel`] carries the deployable components as bytes.

use std::fmt;
use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::hf_bpe::{adapter_constructor, resolve_source_tokenizer};
pub use uor_r4_core::transformerless::hf_bpe::{TokenizerAdapter, TokenizerAdapterKey};
use uor_r4_graph_certify::score::{
    DEFAULT_EMISSION_ENTRIES, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B,
    DEFAULT_TRANSITION_OUT_DEGREE, DEFAULT_WITNESS_SAMPLE,
};
use uor_r4_graph_compiler::induction::{
    DEFAULT_DEPTHS, DEFAULT_K0, DEFAULT_MEMORY_BUDGET_MB, DEFAULT_REGIONS_BUDGET,
};
use uor_r4_graph_format::{
    ContractVersion, FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR,
    INFERENCE_OPERATION_CONTRACT_VERSION,
};
use uor_r4_model_source::SourceUnavailable;

/// Which compiler stage a progress event or failure belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Teacher bundle generation (tokenizer, corpus, artifact, store).
    TeacherBundle,
    /// Multiresolution cover induction (`compiled.r4g1`).
    GraphCover,
    /// Transition/emission scoring (`score.r4g1`).
    Scoring,
}

impl Stage {
    /// Stable lowercase label for logs and error messages.
    pub fn label(self) -> &'static str {
        match self {
            Stage::TeacherBundle => "teacher-bundle",
            Stage::GraphCover => "graph-cover",
            Stage::Scoring => "scoring",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// One progress observation. `percent` is stage-local (0–100): the
/// teacher-bundle stage reports its own coarse phases; the cover and
/// scoring stages bracket their single call with 0 and 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressEvent {
    pub stage: Stage,
    pub percent: u8,
    pub label: &'static str,
}

/// The quality basis declared in the score report. Hugging Face-sourced
/// builds default to `RelativeTla`: their teacher-generated distributions
/// are not comparable to the legacy fixture corpus that established the
/// absolute pinned floor (see `validate_quality_report`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualityProfile {
    /// Absolute pinned floor (legacy fixture corpus basis).
    Pinned,
    /// Relative to the TLA baseline of the same teacher (HF builds).
    #[default]
    RelativeTla,
}

impl QualityProfile {
    fn as_flag(self) -> &'static str {
        match self {
            QualityProfile::Pinned => "pinned",
            QualityProfile::RelativeTla => "relative_tla",
        }
    }
}

/// Scoring-stage knobs; defaults mirror the `score` command's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringOptions {
    pub transition_out_degree: usize,
    pub emission_entries: usize,
    pub root_top_b: usize,
    pub exct_top_x: usize,
    pub witness_sample: usize,
}

impl Default for ScoringOptions {
    fn default() -> Self {
        Self {
            transition_out_degree: DEFAULT_TRANSITION_OUT_DEGREE,
            emission_entries: DEFAULT_EMISSION_ENTRIES,
            root_top_b: DEFAULT_ROOT_TOP_B,
            exct_top_x: DEFAULT_EXCT_TOP_X,
            witness_sample: DEFAULT_WITNESS_SAMPLE,
        }
    }
}

/// Knobs for all three stages; defaults mirror the stage commands' own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileOptions {
    /// Teacher-corpus generation budget in wall-clock seconds per run.
    pub seconds: u64,
    /// Teacher-corpus target size (records) before the bundle compiles.
    pub target: usize,
    /// Oracle sequence length.
    pub sequence_length: usize,
    /// Enable the R4 attention variant in the teacher oracle.
    pub r4_attention: bool,
    /// Cover depth count.
    pub depths: usize,
    /// Cover branching factor.
    pub k0: usize,
    /// Cover region budget.
    pub regions_budget: usize,
    /// Cover induction memory budget (MiB).
    pub memory_budget_mb: u64,
    /// Scoring-stage knobs.
    pub scoring: ScoringOptions,
    /// Quality basis declared in the score report.
    pub quality_profile: QualityProfile,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            seconds: 300,
            target: 20_000,
            sequence_length: 128,
            r4_attention: false,
            depths: DEFAULT_DEPTHS,
            k0: DEFAULT_K0,
            regions_budget: DEFAULT_REGIONS_BUDGET,
            memory_budget_mb: DEFAULT_MEMORY_BUDGET_MB,
            scoring: ScoringOptions::default(),
            quality_profile: QualityProfile::default(),
        }
    }
}

/// A typed compile invocation.
///
/// Since issue #718, `tokenizer_adapter` is required rather than inferred.
/// This is an intentional source-level API break: existing callers must name
/// the exact family/version pair so adding a second tokenizer definition to a
/// source snapshot can never silently change compilation semantics.
#[derive(Debug, Clone)]
pub struct CompileRequest {
    /// Verified local Hugging Face-style source directory (must contain
    /// `config.json`, the selected tokenizer definition, and at least one
    /// `*.safetensors`).
    pub source_dir: PathBuf,
    /// Private resumable workspace: the corpus, checkpoints, and
    /// intermediate artifacts live here. Re-running [`compile`] with the
    /// same request resumes an incomplete corpus.
    pub work_dir: PathBuf,
    /// Exact registered source-tokenizer adapter to use. Compilation never
    /// infers a preference when a snapshot presents multiple tokenizer
    /// definitions: callers select the family and version as one typed pair.
    pub tokenizer_adapter: TokenizerAdapterKey,
    /// Stage knobs.
    pub options: CompileOptions,
    /// Root κ of the #597 source-snapshot manifest
    /// (`source_manifest.json`) describing `source_dir`, when the caller
    /// has computed it (the root crate's `model::source_manifest_kappa`).
    /// Passed through to the cover stage so `compile_report.json` binds
    /// the snapshot identity; `None` leaves legacy reports unchanged.
    /// This crate never computes the κ itself (no uor-addr dependency).
    pub source_manifest_kappa: Option<String>,
}

/// How to resume an incomplete compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeHint {
    /// The work directory holding the partial corpus.
    pub work_dir: PathBuf,
    /// Human-readable resume instruction.
    pub detail: String,
}

/// The deployable output of a completed compile: all components as
/// bytes, with provenance.
#[derive(Debug, Clone)]
pub struct CompiledModel {
    /// Scored deployable R4G1 graph (`score.r4g1` bytes).
    pub graph: Vec<u8>,
    /// Teacher artifact bytes the engine derives input signatures from.
    pub signature_artifact: Vec<u8>,
    /// Bundle tokenizer bytes (binary tokenizer.bin format).
    pub tokenizer: Option<Vec<u8>>,
    /// `score_report.json` bytes.
    pub score_report: Vec<u8>,
    /// `compile_report.json` bytes.
    pub compile_report: Vec<u8>,
    /// Options + format/contract versions + component digests.
    pub provenance: CompileProvenance,
}

/// The result of one [`compile`] call: either the scored graph is ready,
/// or the teacher corpus is still incomplete and the same request should
/// be re-run (the corpus checkpoint in the work directory resumes). The
/// complete model is boxed to keep the enum small.
#[derive(Debug, Clone)]
pub enum CompileOutcome {
    Complete(Box<CompiledModel>),
    Incomplete { resume_hint: ResumeHint },
}

/// blake3 digests (`blake3:<hex>`) of the returned components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDigests {
    pub graph: String,
    pub signature_artifact: String,
    pub tokenizer: Option<String>,
    pub score_report: String,
    pub compile_report: String,
}

/// What a completed compile ran with and produced.
#[derive(Debug, Clone)]
pub struct CompileProvenance {
    /// The exact options the stages ran with.
    pub options: CompileOptions,
    /// Complete validated tokenizer-adapter identity used by stage A,
    /// including the raw-definition CID, declared policy, and adapter digest.
    pub tokenizer_adapter: TokenizerAdapter,
    /// R4G1 format version of the emitted graph.
    pub format_version: (u8, u8),
    /// Normative inference operation contract version.
    pub contract_version: ContractVersion,
    /// Component digests.
    pub digests: ComponentDigests,
}

/// Orchestrate the three compiler stages for `request`. Resumable: an
/// incomplete teacher corpus returns
/// [`CompileOutcome::Incomplete`] and re-calling with the same request
/// continues generation from the checkpoint in the work directory.
pub fn compile(
    request: &CompileRequest,
    progress: &mut dyn FnMut(ProgressEvent),
) -> Result<CompileOutcome, SourceUnavailable> {
    let preflight_tokenizer_adapter =
        validate_source(&request.source_dir, &request.tokenizer_adapter)?;
    let work = &request.work_dir;
    std::fs::create_dir_all(work).map_err(|source| {
        SourceUnavailable::new(format!("{} stage I/O: {source}", Stage::TeacherBundle))
    })?;
    let meta = work.join("corpus.meta");
    let recs = work.join("corpus.records");
    let artifacts = work.join("tless_artifacts.bin");
    for path in [&meta, &recs, &artifacts] {
        utf8(path, Stage::TeacherBundle)?;
    }

    uor_r4_graph_cli::compile_hugging_face_with_progress(
        &stage_a_flags(request),
        |percent, label| {
            progress(ProgressEvent {
                stage: Stage::TeacherBundle,
                percent,
                label,
            });
        },
    )
    .map_err(|message| {
        SourceUnavailable::new(format!("{} stage: {message}", Stage::TeacherBundle))
    })?;
    let tokenizer_adapter = read_stage_a_tokenizer_adapter(work, &preflight_tokenizer_adapter)?;
    let attention_operator = uor_r4_graph_cli::compiled_attention_operator(work)?;

    if !corpus_complete(&meta)? {
        return Ok(CompileOutcome::Incomplete {
            resume_hint: ResumeHint {
                work_dir: work.clone(),
                detail: "teacher corpus incomplete; call `compile` again with the same request \
                         to resume from the work-directory checkpoint"
                    .to_owned(),
            },
        });
    }

    let cover_out = work.join("graph");
    progress(ProgressEvent {
        stage: Stage::GraphCover,
        percent: 0,
        label: "Inducing multiresolution cover...",
    });
    uor_r4_graph_compiler::compile(&stage_b_flags(request, &cover_out, &attention_operator)?)
        .map_err(|error| SourceUnavailable::new(format!("{} stage: {error}", Stage::GraphCover)))?;
    progress(ProgressEvent {
        stage: Stage::GraphCover,
        percent: 100,
        label: "Cover compiled.",
    });

    let scored_out = work.join("scored");
    progress(ProgressEvent {
        stage: Stage::Scoring,
        percent: 0,
        label: "Compiling transitions and emission residuals...",
    });
    uor_r4_graph_cli::score_command(&stage_c_flags(request, &cover_out, &scored_out)).map_err(
        |message| SourceUnavailable::new(format!("{} stage: {message}", Stage::Scoring)),
    )?;
    progress(ProgressEvent {
        stage: Stage::Scoring,
        percent: 100,
        label: "Scored graph ready.",
    });

    let graph = read_stage_file(Stage::Scoring, &scored_out.join("score.r4g1"))?;
    let signature_artifact = read_stage_file(Stage::TeacherBundle, &artifacts)?;
    let tokenizer = read_optional_stage_file(Stage::TeacherBundle, &work.join("tokenizer.bin"))?;
    let score_report = read_stage_file(Stage::Scoring, &scored_out.join("score_report.json"))?;
    let compile_report =
        read_stage_file(Stage::GraphCover, &cover_out.join("compile_report.json"))?;

    let digests = ComponentDigests {
        graph: digest(&graph),
        signature_artifact: digest(&signature_artifact),
        tokenizer: tokenizer.as_deref().map(digest),
        score_report: digest(&score_report),
        compile_report: digest(&compile_report),
    };
    let provenance = CompileProvenance {
        options: request.options,
        tokenizer_adapter,
        format_version: (FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR),
        contract_version: INFERENCE_OPERATION_CONTRACT_VERSION,
        digests,
    };
    Ok(CompileOutcome::Complete(Box::new(CompiledModel {
        graph,
        signature_artifact,
        tokenizer,
        score_report,
        compile_report,
        provenance,
    })))
}

/// A verified local HF-style source: `config.json`, at least one
/// `*.safetensors` weight file, and the exact registered tokenizer definition
/// named by `tokenizer_adapter`. Downloading is out of scope for this crate by
/// design.
fn validate_source(
    source: &Path,
    tokenizer_adapter: &TokenizerAdapterKey,
) -> Result<TokenizerAdapter, SourceUnavailable> {
    let invalid = |message: String| SourceUnavailable::new(format!("invalid source: {message}"));
    let metadata = source
        .metadata()
        .map_err(|error| invalid(format!("{}: {error}", source.display())))?;
    if !metadata.is_dir() {
        return Err(invalid(format!("{} is not a directory", source.display())));
    }
    if !source.join("config.json").is_file() {
        return Err(invalid(format!(
            "{} is missing config.json",
            source.display()
        )));
    }
    let has_weights = std::fs::read_dir(source)
        .map_err(|error| invalid(format!("{}: {error}", source.display())))?
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "safetensors")
        });
    if !has_weights {
        return Err(invalid(format!(
            "{} carries no *.safetensors weights",
            source.display()
        )));
    }
    let tokenizer = resolve_source_tokenizer(source, Some(tokenizer_adapter)).map_err(|error| {
        SourceUnavailable {
            reason: format!("invalid source: {}", error.reason),
            kind: error.kind,
        }
    })?;
    tokenizer.adapter().ok_or_else(|| {
        invalid(format!(
            "{} resolved to an adapterless tokenizer for {}/{}",
            source.display(),
            tokenizer_adapter.family,
            tokenizer_adapter.version
        ))
    })
}

/// Structural corpus-completion check: the corpus metadata trailer's
/// done byte. Mirrors the gate of
/// `uor_r4_core::transformerless::compiler::load_corpus_from` (25-byte
/// meta, `meta[24] == 1`) without loading the corpus; a missing or
/// partial meta means the teacher-bundle stage has more generation to
/// do. This is the ONLY resume signal — no stage stdout is parsed.
fn corpus_complete(meta: &Path) -> Result<bool, SourceUnavailable> {
    match std::fs::read(meta) {
        Ok(bytes) => Ok(bytes.len() == 25 && bytes[24] == 1),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(SourceUnavailable::new(format!(
            "{} stage I/O: {source}",
            Stage::TeacherBundle
        ))),
    }
}

fn utf8(path: &Path, stage: Stage) -> Result<(), SourceUnavailable> {
    path.to_str().map(|_| ()).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{stage} stage: path is not UTF-8: {}",
            path.display()
        ))
    })
}

fn read_stage_file(stage: Stage, path: &Path) -> Result<Vec<u8>, SourceUnavailable> {
    std::fs::read(path)
        .map_err(|source| SourceUnavailable::new(format!("{stage} stage I/O: {source}")))
}

fn read_optional_stage_file(
    stage: Stage,
    path: &Path,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    let io_error =
        |message: String| SourceUnavailable::new(format!("{stage} stage I/O: {message}"));
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(format!("{}: {error}", path.display()))),
    };
    let is_regular = if metadata.file_type().is_symlink() {
        std::fs::metadata(path)
            .map_err(|error| io_error(format!("{}: {error}", path.display())))?
            .is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(io_error(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    std::fs::read(path)
        .map(Some)
        .map_err(|error| io_error(format!("{}: {error}", path.display())))
}

fn is_blake3_cid(value: &str) -> bool {
    value.len() == "blake3:".len() + 64
        && value.starts_with("blake3:")
        && value["blake3:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_tokenizer_adapter_record(adapter: &TokenizerAdapter) -> Result<(), SourceUnavailable> {
    adapter_constructor(&adapter.family, adapter.version)?;
    if !is_blake3_cid(&adapter.tokenizer_cid) {
        return Err(SourceUnavailable::new(format!(
            "tokenizer adapter {}/{} has invalid tokenizer CID {}",
            adapter.family, adapter.version, adapter.tokenizer_cid
        )));
    }
    let declared = adapter.declared_digest();
    if adapter.adapter_digest != declared {
        return Err(SourceUnavailable::new(format!(
            "tokenizer adapter {}/{} declares digest {}, expected {declared}",
            adapter.family, adapter.version, adapter.adapter_digest
        )));
    }
    Ok(())
}

fn read_stage_a_tokenizer_adapter(
    work: &Path,
    preflight: &TokenizerAdapter,
) -> Result<TokenizerAdapter, SourceUnavailable> {
    let path = work.join("tokenizer_adapter.json");
    let stage_error = |message: String| {
        SourceUnavailable::new(format!("{} stage: {message}", Stage::TeacherBundle))
    };
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| stage_error(format!("{}: {error}", path.display())))?;
    if !metadata.file_type().is_file() {
        return Err(stage_error(format!(
            "{} is not a regular tokenizer adapter sidecar",
            path.display()
        )));
    }
    let bytes = std::fs::read(&path)
        .map_err(|error| stage_error(format!("{}: {error}", path.display())))?;
    let recorded: TokenizerAdapter = serde_json::from_slice(&bytes).map_err(|error| {
        stage_error(format!(
            "{} is not a valid tokenizer adapter record: {error}",
            path.display()
        ))
    })?;
    validate_tokenizer_adapter_record(&recorded)
        .map_err(|error| stage_error(format!("{}: {}", path.display(), error.reason)))?;
    if recorded != *preflight {
        return Err(stage_error(format!(
            "{} does not match the preflight source adapter: recorded {}/{} CID {} digest {}; preflight {}/{} CID {} digest {}",
            path.display(),
            recorded.family,
            recorded.version,
            recorded.tokenizer_cid,
            recorded.adapter_digest,
            preflight.family,
            preflight.version,
            preflight.tokenizer_cid,
            preflight.adapter_digest,
        )));
    }
    Ok(recorded)
}

fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

// ------------------------------------------------------------ compat shim
// The stage entry points take CLI flag strings; these builders are the
// sanctioned translation layer (see the module docs). Private, and
// temporary until the stages grow typed entry points.

fn stage_a_flags(request: &CompileRequest) -> Vec<String> {
    let options = &request.options;
    let mut flags = vec![
        "--source".to_owned(),
        request.source_dir.display().to_string(),
        "--output".to_owned(),
        request.work_dir.display().to_string(),
        "--seconds".to_owned(),
        options.seconds.to_string(),
        "--target".to_owned(),
        options.target.to_string(),
        "--sequence-length".to_owned(),
        options.sequence_length.to_string(),
        "--tokenizer-family".to_owned(),
        request.tokenizer_adapter.family.clone(),
        "--tokenizer-version".to_owned(),
        request.tokenizer_adapter.version.to_string(),
    ];
    if options.r4_attention {
        flags.push("--r4-attention".to_owned());
    }
    flags
}

fn stage_b_flags(
    request: &CompileRequest,
    cover_out: &Path,
    operator: &uor_r4_model_source::attention::AttentionOperatorSpec,
) -> Result<Vec<String>, SourceUnavailable> {
    let options = &request.options;
    let work = &request.work_dir;
    let mut flags = vec![
        "--corpus-meta".to_owned(),
        work.join("corpus.meta").display().to_string(),
        "--corpus-recs".to_owned(),
        work.join("corpus.records").display().to_string(),
        "--artifacts".to_owned(),
        work.join("tless_artifacts.bin").display().to_string(),
        "--tokenizer".to_owned(),
        work.join("tokenizer.bin").display().to_string(),
        "--depths".to_owned(),
        options.depths.to_string(),
        "--k0".to_owned(),
        options.k0.to_string(),
        "--regions-budget".to_owned(),
        options.regions_budget.to_string(),
        "--memory-budget".to_owned(),
        options.memory_budget_mb.to_string(),
        "--out".to_owned(),
        cover_out.display().to_string(),
    ];
    // #597: thread the source-snapshot manifest root κ into the cover
    // stage so the emitted compile_report.json binds the snapshot.
    if let Some(kappa) = &request.source_manifest_kappa {
        flags.extend(["--source-manifest-kappa".to_owned(), kappa.clone()]);
    }
    // #602/#704: the registry-validated stage-A binding is authoritative.
    // Reconstructing identity from the request boolean would mislabel an
    // auto-detected GPT-2 teacher as standard attention and could relabel a
    // resumed corpus from another arithmetic era.
    let json = serde_json::to_string(operator).map_err(|error| {
        SourceUnavailable::new(format!("attention-operator binding serialization: {error}"))
    })?;
    flags.extend(["--attention-operator".to_owned(), json]);
    Ok(flags)
}

fn stage_c_flags(request: &CompileRequest, cover_out: &Path, scored_out: &Path) -> Vec<String> {
    let options = &request.options;
    let work = &request.work_dir;
    let scoring = &options.scoring;
    vec![
        "--corpus-meta".to_owned(),
        work.join("corpus.meta").display().to_string(),
        "--corpus-recs".to_owned(),
        work.join("corpus.records").display().to_string(),
        "--artifacts".to_owned(),
        work.join("tless_artifacts.bin").display().to_string(),
        "--tokenizer".to_owned(),
        work.join("tokenizer.bin").display().to_string(),
        // The cover artifact of stage B: byte-identical to re-induction
        // by construction, so this is a pure cache.
        "--cover".to_owned(),
        cover_out.join("compiled.r4g1").display().to_string(),
        "--transition-out-degree".to_owned(),
        scoring.transition_out_degree.to_string(),
        "--emission-entries".to_owned(),
        scoring.emission_entries.to_string(),
        "--root-top-b".to_owned(),
        scoring.root_top_b.to_string(),
        "--exct-top-x".to_owned(),
        scoring.exct_top_x.to_string(),
        "--witness-sample".to_owned(),
        scoring.witness_sample.to_string(),
        "--quality-profile".to_owned(),
        options.quality_profile.as_flag().to_owned(),
        "--out".to_owned(),
        scored_out.display().to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source_manifest_kappa: Option<String>) -> CompileRequest {
        CompileRequest {
            source_dir: PathBuf::from("source"),
            work_dir: PathBuf::from("work"),
            tokenizer_adapter: TokenizerAdapterKey::hf_byte_bpe_v1(),
            options: CompileOptions::default(),
            source_manifest_kappa,
        }
    }

    #[test]
    fn tokenizer_adapter_is_threaded_as_an_atomic_stage_a_pair() {
        let mut request = request(None);
        request.tokenizer_adapter = TokenizerAdapterKey::new("example-family", 41);
        let flags = stage_a_flags(&request);
        assert!(flags
            .windows(2)
            .any(|pair| { pair == ["--tokenizer-family", "example-family"] }));
        assert!(flags
            .windows(2)
            .any(|pair| pair == ["--tokenizer-version", "41"]));
    }

    #[test]
    fn deployed_tokenizer_is_threaded_into_cover_and_score_stages() {
        let request = request(None);
        let expected = request.work_dir.join("tokenizer.bin").display().to_string();
        let operator = uor_r4_model_source::attention::AttentionOperatorSpec::standard();
        for flags in [
            stage_b_flags(&request, Path::new("cover"), &operator).expect("cover flags"),
            stage_c_flags(&request, Path::new("cover"), Path::new("scored")),
        ] {
            assert!(
                flags
                    .windows(2)
                    .any(|pair| pair == ["--tokenizer", expected.as_str()]),
                "stage flags must bind the exact deployed tokenizer: {flags:?}"
            );
        }
    }

    #[test]
    fn source_validation_returns_the_complete_adapter_identity() {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-api-adapter-identity-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("source dir");
        std::fs::write(dir.join("config.json"), b"{}").expect("config");
        std::fs::write(dir.join("model.safetensors"), b"fixture").expect("weights");
        std::fs::write(
            dir.join("tokenizer.json"),
            br#"{
                "model":{"type":"BPE","vocab":{"a":0},"merges":[]},
                "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false}
            }"#,
        )
        .expect("tokenizer");

        let adapter = validate_source(&dir, &TokenizerAdapterKey::hf_byte_bpe_v1())
            .expect("selected source validates");
        assert_eq!(adapter.family, "hf-byte-bpe");
        assert_eq!(adapter.version, 1);
        assert!(adapter.tokenizer_cid.starts_with("blake3:"));
        assert_eq!(adapter.tokenizer_cid.len(), "blake3:".len() + 64);
        assert!(adapter.adapter_digest.starts_with("blake3:"));
        assert_eq!(adapter.adapter_digest, adapter.declared_digest());
        assert_eq!(adapter.policy.normalizer, "none");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn optional_stage_file_distinguishes_absence_from_invalid_entries() {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-api-optional-stage-file-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("work dir");
        assert_eq!(
            read_optional_stage_file(Stage::TeacherBundle, &dir.join("missing.bin"))
                .expect("genuine absence"),
            None
        );
        let invalid = dir.join("tokenizer.bin");
        std::fs::create_dir(&invalid).expect("directory at file path");
        let error = read_optional_stage_file(Stage::TeacherBundle, &invalid)
            .expect_err("non-file is not absence");
        assert!(error.reason.contains("not a regular file"), "{error}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            std::fs::remove_dir(&invalid).expect("remove directory");
            symlink(dir.join("missing-target.bin"), &invalid).expect("dangling symlink");
            let error = read_optional_stage_file(Stage::TeacherBundle, &invalid)
                .expect_err("dangling symlink is not absence");
            assert!(error.reason.contains("tokenizer.bin"), "{error}");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stage_a_adapter_sidecar_is_validated_and_must_match_preflight() {
        let dir =
            std::env::temp_dir().join(format!("uor-r4-api-stage-a-adapter-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("work dir");

        let mut expected = TokenizerAdapter {
            family: TokenizerAdapter::HF_BYTE_BPE_FAMILY.to_owned(),
            version: TokenizerAdapter::HF_BYTE_BPE_VERSION,
            tokenizer_cid: format!("blake3:{}", "1".repeat(64)),
            ..TokenizerAdapter::default()
        };
        expected.adapter_digest = expected.declared_digest();
        let path = dir.join("tokenizer_adapter.json");

        std::fs::write(&path, b"{not json").expect("invalid sidecar");
        let error = read_stage_a_tokenizer_adapter(&dir, &expected)
            .expect_err("malformed sidecar must fail");
        assert!(
            error.reason.contains("not a valid tokenizer adapter"),
            "{error}"
        );

        let mut mismatch = expected.clone();
        mismatch.tokenizer_cid = format!("blake3:{}", "2".repeat(64));
        mismatch.adapter_digest = mismatch.declared_digest();
        std::fs::write(
            &path,
            serde_json::to_vec(&mismatch).expect("serialize mismatch"),
        )
        .expect("mismatching sidecar");
        let error = read_stage_a_tokenizer_adapter(&dir, &expected)
            .expect_err("valid but different stage-A identity must fail");
        assert!(
            error.reason.contains("does not match the preflight"),
            "{error}"
        );

        std::fs::write(
            &path,
            serde_json::to_vec(&expected).expect("serialize expected"),
        )
        .expect("matching sidecar");
        assert_eq!(
            read_stage_a_tokenizer_adapter(&dir, &expected).expect("matching sidecar validates"),
            expected
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    /// #597 plumbing seam: a request carrying the source-snapshot
    /// manifest root κ forwards it to the cover stage (whose report field
    /// it populates); an absent κ adds no flag, so legacy invocations are
    /// byte-identical.
    #[test]
    fn source_manifest_kappa_is_threaded_into_cover_stage_flags() {
        let kappa = format!("blake3:{}", "7".repeat(64));
        let operator = uor_r4_model_source::attention::AttentionOperatorSpec::standard();
        let with = stage_b_flags(&request(Some(kappa.clone())), Path::new("cover"), &operator)
            .expect("flags");
        assert!(with
            .windows(2)
            .any(|pair| pair == ["--source-manifest-kappa", kappa.as_str()]));
        let without = stage_b_flags(&request(None), Path::new("cover"), &operator).expect("flags");
        assert!(!without.iter().any(|flag| flag == "--source-manifest-kappa"));
        assert_eq!(with.len(), without.len() + 2);
    }

    /// #602/#704 plumbing seam: stage B forwards each registry-validated
    /// stage-A record independently of the request switch. Explicit immutable
    /// v1 records must remain their exact identities after the current aliases
    /// advance, and every current v2 family must traverse the same seam.
    #[test]
    fn all_source_attention_records_are_threaded_into_cover_stage_flags() {
        use uor_r4_model_source::attention::AttentionOperatorSpec;
        let flag_value = |flags: &[String]| -> AttentionOperatorSpec {
            let index = flags
                .iter()
                .position(|flag| flag == "--attention-operator")
                .expect("flag present");
            serde_json::from_str(&flags[index + 1]).expect("typed record round-trips")
        };

        for (mut request, operator) in [
            (request(None), AttentionOperatorSpec::standard_v1()),
            (request(None), AttentionOperatorSpec::standard_v2()),
            (request(None), AttentionOperatorSpec::experimental_r4_v1()),
            (request(None), AttentionOperatorSpec::experimental_r4_v2()),
            (request(None), AttentionOperatorSpec::learned_absolute_v1()),
            (request(None), AttentionOperatorSpec::learned_absolute_v2()),
        ] {
            // Deliberately contradict the supplied record where possible:
            // the stage-A binding, not this policy bit, is authoritative.
            request.options.r4_attention = operator == AttentionOperatorSpec::standard();
            let flags = stage_b_flags(&request, Path::new("cover"), &operator).expect("flags");
            assert_eq!(flag_value(&flags), operator);
        }
    }
}
