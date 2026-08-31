use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use uor_r4_api::{BundleCapability, UorMatmulProvenance};
use uor_r4_core::local_geometric_generation::{
    LocalGenerationControl, LocalGenerationStopReason, LocalGeometricGenerator,
};
use uor_r4_core::source_free_table::{
    d3_is_held_out, ContinuationStop, GeometricFirstDivergence, MultiscaleCountRadiusR4V1,
    SourceDocument, SourceFreeTable,
};
use uor_r4_core::transformerless::hf_bpe::{resolve_source_tokenizer, TokenizerAdapterKey};
use uor_r4_graph_cli as transformerless_command;
use uor_r4_wasm_router::chat::{ChatAnswer, ChatEngine, ChatError, DEFAULT_SAMPLE_SEED};
use uor_r4_wasm_router::model::{
    default_model_reference, download_source, evaluate_live_quality, model_store_root,
    ModelCapability, ModelError, ModelManifest, ModelStore, QualityAttestation, SourceDownload,
};
use uor_r4_wasm_router::release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME;
use uor_r4_wasm_router::release_bundle_packager::{self, PackageInputs};
use uor_r4_wasm_router::server::{self, ServerConfig};
use uor_r4_wasm_router::tless_uor;

/// UOR-R4 geometric intelligence research tools.
#[derive(Parser, Debug)]
#[command(
    name = "r4",
    version,
    about,
    long_about = "Run the working R4/Spin causal-attention generator, exercise the fail-closed source-span-pointer research surface, or inspect the preserved geometric research tools.\n\n`generate` uses the coherent #1017 reference. `answer` accepts a #954 pointer artifact over independently encoded #1017 R4/Spin states and serves only an exact source sentence or a typed non-answer; the frozen C1-SB1 run failed its development gate and emitted no qualified final head. Both paths remain source-backed floating-point/softmax references, not the final source-free transformerless runtime. Preserved compiler and certification commands remain available through `r4 research-tools`."
)]
struct Cli {
    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Host interface to bind.
    #[arg(long, env = "UOR_R4_HOST", default_value = "127.0.0.1", global = true)]
    host: String,

    /// Port to listen on.
    #[arg(long, env = "UOR_R4_PORT", default_value_t = 8000, global = true)]
    port: u16,

    /// Router manifold cache file.
    #[arg(
        long,
        env = "UOR_R4_MANIFOLD_CACHE",
        default_value = "manifold_cache_rust.json",
        global = true
    )]
    manifold_cache: String,

    /// Legacy transformerless artifact container.
    #[arg(
        long,
        env = "TLESS_ARTIFACTS",
        default_value = "/tmp/tless_artifacts.bin",
        global = true
    )]
    tless_artifacts: String,

    /// Legacy transformerless graded store.
    #[arg(
        long,
        env = "TLESS_STORE",
        default_value = "/tmp/tless_store.bin",
        global = true
    )]
    tless_store: String,

    /// Legacy llama2.c tokenizer.
    #[arg(
        long,
        env = "TLESS_TOKENIZER",
        default_value = "/tmp/ref/tokenizer.bin",
        global = true
    )]
    tless_tokenizer: String,

    /// Validated scored R4G1 graph. Defaults to graph/score.r4g1 beside the
    /// configured transformerless artifact when present.
    #[arg(long, env = "R4G1_ARTIFACT", global = true)]
    r4g1_artifact: Option<String>,

    /// Observation-corpus metadata used by the dashboard R4G1 compiler.
    #[arg(long, env = "TLESS_CORPUS_META", global = true)]
    tless_corpus_meta: Option<String>,

    /// Observation-corpus records used by the dashboard R4G1 compiler.
    #[arg(long, env = "TLESS_CORPUS_RECS", global = true)]
    tless_corpus_recs: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Launch the recommended geometric router research dashboard.
    Demo,
    /// Inspect one prompt's geometric route without compiling or loading a model.
    Route(RouteArgs),
    /// Run the bounded #953 provider-free decoded geometric generation witness.
    BoundedGeometricGenerate(BoundedGeometricGenerateArgs),
    /// Build and measure the #989 source-free lexical table baseline.
    SourceFreeTable(SourceFreeTableArgs),
    /// Generate bounded text through all-layer R4/Spin causal softmax.
    R4SoftmaxGenerate(R4SoftmaxGenerateArgs),
    /// Generate from the local learned R4/Spin model through causal softmax.
    #[command(visible_alias = "generate")]
    R4SoftmaxLocalGenerate(R4SoftmaxLocalGenerateArgs),
    /// Exercise a #954 source-span pointer; no qualified final head exists yet.
    Answer(GroundedAnswerArgs),
    /// Qualify one frozen #1014, #1017, or #1019 local causal-softmax campaign.
    R4SoftmaxLocalQualify(R4SoftmaxLocalQualifyArgs),
    /// Compile and score the first source-free student of R4/Spin softmax traces.
    R4SoftmaxTraceStudent(R4SoftmaxTraceStudentArgs),
    /// Compile the #1011 recurrent state artifact from frozen construction inputs only.
    R4SoftmaxTraceStateCompile(R4SoftmaxTraceStateCompileArgs),
    /// Seal all #1011 state arms from causal token inputs only.
    R4SoftmaxTraceStateSeal(R4SoftmaxTraceStateSealArgs),
    /// Reveal the frozen #1010 judge against an immutable #1011 state seal.
    R4SoftmaxTraceStateReveal(R4SoftmaxTraceStateRevealArgs),
    /// Localize construction-trace signal loss across the four frozen #1012 boundaries.
    R4SoftmaxTraceObservability(R4SoftmaxTraceObservabilityArgs),
    /// Run the fixed S0 lexical/route-state round-trip witness without loading a model.
    LexicalIngestionWitness,
    /// Run the frozen A1.0 ordered-state/value-reachability gate without a scorer.
    RecursiveAttentionA1Probe,
    /// Run the frozen A1R associative ordered-summary and candidate-relative probe.
    AssociativeOrderedSummaryA1rProbe,
    /// Run the frozen A1P candidate-relative identifiability hard gate.
    CandidateRelativeIdentifiabilityA1pProbe,
    /// List preserved compiler, serving, and certification commands.
    ResearchTools,
    /// Run the full artifact-discovering historical HTTP server.
    Serve(ServeArgs),
    /// Ask one question using the local transformerless library directly.
    #[command(hide = true)]
    Ask(AskArgs),
    /// Start an interactive, stateful local chat.
    #[command(hide = true)]
    Chat(ChatArgs),
    /// Connect to a local r4 server endpoint as a remote interactive client.
    #[command(hide = true)]
    Client(ClientArgs),
    /// View or export UOR Q&A audit traces and geometry metrics.
    #[command(hide = true)]
    Audit(AuditArgs),
    /// Compile a recorded corpus or a local/pinned Hugging Face model into an R⁴ bundle.
    #[command(hide = true)]
    Compile(CompileArgs),
    /// Download pinned open weights for offline compilation.
    #[command(hide = true)]
    Download(DownloadArgs),
    /// Import an evaluated compiled bundle into the UOR CID store.
    #[command(hide = true)]
    Import(ImportArgs),
    /// #655-D2: package a compiled R4G1 bundle's release-bundle.json
    /// sidecar, giving `release_bundle_loader::verify_release_bundle_sidecar`
    /// (#655-C1c) a real manifest to verify.
    #[command(hide = true)]
    PackageReleaseBundle(PackageReleaseBundleArgs),
    /// #741: explicitly fetch a published release's packaged bundle from
    /// its GitHub Release and install it — only after every component
    /// digest matches the release's attested `release-bundle.json`
    /// manifest. Never runs implicitly; a digest mismatch, an unattested
    /// archive entry, or an existing install refuses with nothing
    /// written.
    #[command(hide = true)]
    InstallRelease(InstallReleaseArgs),
    /// Evaluate an HF-compiled bundle and emit an instruction-quality report.
    #[command(hide = true)]
    EvaluateReport(EvaluateReportArgs),
    /// Run the bounded #950 local source-control and one-layer R4 spike.
    #[command(hide = true)]
    GeometricDecoderSpike(GeometricDecoderSpikeArgs),
    /// Fit and qualify the bounded #951 one-layer mixer and memory adapter.
    #[command(hide = true)]
    GeometricMixerQualification(GeometricMixerQualificationArgs),
    /// Print legacy proof-workflow prerequisites.
    #[command(hide = true)]
    Setup,
    /// Generate the legacy resumable teacher corpus.
    #[command(hide = true)]
    Gen {
        #[arg(default_value_t = 300)]
        seconds: u64,
        #[arg(default_value_t = 150_000)]
        target: usize,
    },
    /// Build the legacy graded store.
    #[command(hide = true)]
    Store,
    /// Run the transformerless certificate workflow.
    #[command(hide = true)]
    Certify,
    /// Run the teacher-free, CID-bound normative deployed-quality evaluator.
    /// Sample mode is the mandatory cheap instrument; full mode is explicit
    /// and is used only after the sample's predeclared gates can still pass.
    #[command(hide = true)]
    DeployedQuality(DeployedQualityArgs),
    /// Run the measured local comparison.
    #[command(hide = true)]
    Compare,
    /// Print the recorded comparison certificate.
    #[command(hide = true)]
    CompareReport,
    /// Run the transformerless scenario suite.
    #[command(hide = true)]
    Scenarios,
    /// Print the legacy teacher checkpoint κ.
    #[command(hide = true)]
    TeacherKappa,
    /// Forward a command to the legacy transformerless toolset, e.g.
    /// `r4 transformerless convert-r4g1 --artifacts <TLA> --store <TLS1>
    /// [--calibration <hamming_calibration.json>] --out <R4G1>`.
    #[command(hide = true)]
    Transformerless {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// A-mode graph serving commands, e.g. `r4 graph infill --artifact
    /// graph/score.r4g1 --skeleton 12,_,_,_,99,_,_,_,7`.
    #[command(hide = true)]
    Graph {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run the new R4G1 multiresolution graph compiler pipeline.
    #[command(hide = true)]
    GraphCompile {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = false, num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Sample observation records using a teacher oracle.
    #[command(hide = true)]
    GraphObserve {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = false, num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Args, Debug)]
struct RouteArgs {
    /// Stable identity scope for the route.
    #[arg(long, default_value = "research-demo")]
    identity: String,
    /// Optional UTF-8 text file to add to the small built-in demonstration corpus.
    #[arg(long, value_name = "FILE")]
    corpus: Option<PathBuf>,
    /// Emit the complete route summary as JSON.
    #[arg(long)]
    json: bool,
    /// Prompt to route. Multiple unquoted words are accepted.
    #[arg(required = true, num_args = 1..)]
    prompt: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum BoundedGeometricControl {
    FullPath,
    StateDisabled,
}

impl BoundedGeometricControl {
    const fn generation_control(self) -> LocalGenerationControl {
        match self {
            Self::FullPath => LocalGenerationControl::FullPath,
            Self::StateDisabled => LocalGenerationControl::StateDisabled,
        }
    }
}

#[derive(Args, Debug)]
struct BoundedGeometricGenerateArgs {
    /// Canonical S0 artifact whose embedded input reconstructs its full codec registry.
    #[arg(long, value_name = "FILE")]
    artifact: PathBuf,

    /// Exact UTF-8 prompt bytes. Trailing whitespace is rejected fail closed.
    #[arg(long, value_name = "PROMPT")]
    prompt: String,

    /// Hard maximum number of emitted continuation lexical units.
    #[arg(long, value_name = "UNITS")]
    continuation_cap: usize,

    /// #969 path arm; #953 exposes exactly these two matched arms.
    #[arg(long, value_enum, default_value = "full-path")]
    control: BoundedGeometricControl,

    /// Emit the generator's deterministic canonical JSON report.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct SourceFreeTableArgs {
    /// Pinned JSONL corpus: one object per line with `id` and UTF-8 `text`.
    #[arg(long, value_name = "ARTICLES_JSONL")]
    corpus: PathBuf,

    /// Exact prompt bytes to continue through the fitted lexical tables.
    #[arg(long, value_name = "PROMPT")]
    prompt: String,

    /// Hard maximum number of emitted continuation lexical units.
    #[arg(long, value_name = "UNITS")]
    continuation_cap: usize,

    /// Optional destination for the deterministic packed table artifact.
    #[arg(long, value_name = "FILE")]
    artifact_out: Option<PathBuf>,

    /// Enable the frozen #953 MultiscaleCountRadiusR4V1 matched comparison.
    #[arg(long)]
    geometric_intervention: bool,

    /// Optional destination for the deterministic overlay bound to the table.
    #[arg(long, value_name = "FILE", requires = "geometric_intervention")]
    geometry_overlay_out: Option<PathBuf>,

    /// Emit the deterministic measurement report as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct AskArgs {
    /// CID manifest name/CID, or a locally compiled bundle name.
    #[arg(long, env = "TLESS_MODEL")]
    model: Option<String>,
    /// Override the pinned default sampling seed (#655 decode-default
    /// decision, 2026-08-19: seeded weighted sampling IS the default on
    /// both generation paths — issue #762 lever 2 on the plain path, the
    /// same weighting over the R4G1 candidate walk per #785-C2). Absent,
    /// the pinned `chat::DEFAULT_SAMPLE_SEED` keeps default runs
    /// reproducible. Reproducible from the seed either way.
    #[arg(long, value_name = "SEED", conflicts_with = "greedy")]
    sample: Option<u32>,
    /// Opt into the deterministic decode (greedy beam on the R4G1 path,
    /// greedy argmax on the plain path) instead of the default seeded
    /// sampling.
    #[arg(long, conflicts_with = "sample")]
    greedy: bool,
    /// Explicitly permit a pre-schema-2 local bundle for research. Such a
    /// session prints a typed warning and is never production admission.
    #[arg(long)]
    research: bool,
    /// Question to ask. Multiple unquoted words are accepted.
    #[arg(required = true, num_args = 1..)]
    question: Vec<String>,
}

#[derive(Args, Debug)]
struct ChatArgs {
    /// CID manifest name/CID, or a locally compiled bundle name.
    #[arg(long, env = "TLESS_MODEL")]
    model: Option<String>,
    /// Remote HTTP server URL (e.g. http://127.0.0.1:8000/v1) for client mode.
    #[arg(long)]
    remote: Option<String>,
    /// Override the pinned default sampling seed (#655: seeded weighted
    /// sampling IS the default on both generation paths — #762 lever 2
    /// plain, #785-C2 R4G1); the seed advances turn to turn so the whole
    /// session is reproducible from it. Has no effect in `--remote`
    /// client mode.
    #[arg(long, value_name = "SEED", conflicts_with = "greedy")]
    sample: Option<u32>,
    /// Opt into the deterministic decode instead of the default seeded
    /// sampling. Has no effect in `--remote` client mode.
    #[arg(long, conflicts_with = "sample")]
    greedy: bool,
    /// Explicitly permit a pre-schema-2 local bundle for research. Such a
    /// session prints a typed warning and is never production admission.
    #[arg(long, conflicts_with = "remote")]
    research: bool,
}

#[derive(Args, Debug)]
struct ClientArgs {
    /// Remote HTTP server URL [default: http://127.0.0.1:8000/v1].
    #[arg(long, default_value = "http://127.0.0.1:8000/v1")]
    remote: String,
    /// Model name to send in chat completions payload.
    #[arg(long, default_value = "r4")]
    model: String,
}

#[derive(Args, Debug)]
struct CompileArgs {
    /// Existing local Hugging Face model directory.
    #[arg(long, conflicts_with = "model")]
    source: Option<PathBuf>,
    /// Hugging Face owner/repository to download and compile.
    #[arg(long, conflicts_with = "source")]
    model: Option<String>,
    /// Completed corpus metadata for a transformer-free recorded compile.
    #[arg(long, requires = "corpus_recs", requires = "vocab_size")]
    corpus_meta: Option<PathBuf>,
    /// Completed corpus records for a transformer-free recorded compile.
    #[arg(long, requires = "corpus_meta", requires = "vocab_size")]
    corpus_recs: Option<PathBuf>,
    /// Vocabulary width for a transformer-free recorded compile.
    #[arg(long, requires = "corpus_meta", requires = "corpus_recs")]
    vocab_size: Option<usize>,
    /// Registered source-tokenizer family. Must be supplied atomically with
    /// `--tokenizer-version`; recorded-corpus compiles already carry token ids
    /// and therefore do not accept a source-tokenizer selection.
    #[arg(
        long,
        requires = "tokenizer_version",
        conflicts_with_all = ["corpus_meta", "corpus_recs", "vocab_size"]
    )]
    tokenizer_family: Option<String>,
    /// Registered source-tokenizer adapter version. Must be supplied
    /// atomically with `--tokenizer-family`.
    #[arg(
        long,
        requires = "tokenizer_family",
        conflicts_with_all = ["corpus_meta", "corpus_recs", "vocab_size"]
    )]
    tokenizer_version: Option<u32>,
    /// Immutable 40-character Hugging Face commit SHA.
    #[arg(long, requires = "model")]
    revision: Option<String>,
    /// Compiled bundle directory [default: .uor-models/compiled/<name>].
    #[arg(long)]
    output: Option<PathBuf>,
    /// Maximum teacher-generation time for this resumable invocation.
    #[arg(long, default_value_t = 300)]
    seconds: u64,
    /// Teacher-token goal.
    #[arg(long, default_value_t = 20_000)]
    target: usize,
    /// Teacher context allocation and story length.
    #[arg(long, default_value_t = 128)]
    sequence_length: usize,
    /// Enable the current experimental Llama teacher attention variant (#704
    /// operator `experimental-r4-source-attention/2`: certified-exact Q·K over
    /// the leading 4-wide domain and certified-exact value aggregation, with
    /// the standard max-subtracted softmax) during compilation. GPT-2 ignores
    /// this legacy switch and uses its architecture-owned
    /// `learned-absolute-source-attention/2` record.
    #[arg(long, default_value_t = false)]
    r4_attention: bool,
    /// Deprecated compatibility flag; no longer selects a different Llama matmul owner.
    #[arg(long, default_value_t = false)]
    exact_scalar: bool,
    /// Use the portable libm teacher path and scalar reductions for certificate-bearing builds.
    #[arg(long, default_value_t = false)]
    canonical_deterministic: bool,
}

#[derive(Args, Debug)]
struct AuditArgs {
    /// Path to the session audit log file [default: .uor-models/audit_log.json].
    #[arg(long, default_value = ".uor-models/audit_log.json")]
    log_file: PathBuf,
}

#[derive(Args, Debug)]
struct DownloadArgs {
    #[arg(long)]
    repository: String,
    #[arg(long)]
    revision: String,
    #[arg(long)]
    name: String,
    /// Download destination [default: .uor-models/sources/<name>].
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Capability {
    Continuation,
    InstructionChat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DeployedQualityMode {
    Sample,
    Full,
}

impl DeployedQualityMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Sample => "sample",
            Self::Full => "full",
        }
    }
}

#[derive(Args, Debug)]
struct DeployedQualityArgs {
    /// Exact staging bundle to evaluate. The command never scans or
    /// substitutes another bundle.
    #[arg(long)]
    bundle: PathBuf,
    /// Full source revision that emitted the graph/configuration under test.
    #[arg(long)]
    compiler_revision: String,
    /// Explicit evaluation extent. Only full can authorize production.
    #[arg(
        long,
        env = "R4_DEPLOYED_QUALITY_MODE",
        value_enum,
        default_value = "sample"
    )]
    mode: DeployedQualityMode,
    /// Exact nested, story-distributed sample size. Selection is label-free
    /// and shared by evaluation, report binding, and witness production.
    #[arg(
        long,
        env = "R4_DEPLOYED_QUALITY_POSITIONS",
        default_value_t = uor_r4_api::serving_eval::SAMPLE_TARGET
    )]
    positions: usize,
    /// Dedicated deterministic evaluation workers [default: all available].
    #[arg(long, env = "R4_DEPLOYED_QUALITY_WORKERS")]
    workers: Option<usize>,
    /// Readiness-probe wall-clock ceiling.
    #[arg(long, env = "R4_CERTIFY_R4G1_BUDGET_SECS", default_value_t = 120)]
    probe_budget_secs: u64,
    /// Evaluation wall-clock ceiling. Partial rows are discarded on expiry.
    #[arg(long, env = "R4_CERTIFY_R4G1_EVAL_BUDGET_SECS", default_value_t = 3600)]
    eval_budget_secs: u64,
    /// Replayable cross-surface artifact. Defaults to
    /// <bundle>/graph/cross_surface_parity.json.
    #[arg(long)]
    cross_surface_evidence: Option<PathBuf>,
}

impl From<Capability> for ModelCapability {
    fn from(value: Capability) -> Self {
        match value {
            Capability::Continuation => Self::Continuation,
            Capability::InstructionChat => Self::InstructionChat,
        }
    }
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    source_model: String,
    #[arg(long, value_enum)]
    capability: Capability,
    #[arg(long)]
    artifacts: PathBuf,
    #[arg(long)]
    store: PathBuf,
    #[arg(long)]
    tokenizer: PathBuf,
    /// Optional compiled R4G1 graph (`compiled.r4g1`). When present, the
    /// live quality gate (#750) probes this path in addition to the plain
    /// TLA path and requires both to pass, since `r4 ask`/`r4 chat` will
    /// transparently prefer this graph over the plain path at serving time
    /// if one exists at the manifest-name-keyed convention path.
    #[arg(long)]
    r4g1: Option<PathBuf>,
    /// Offline held-out evaluation report from `r4 evaluate-report`.
    /// Required for `--capability instruction-chat`: not consulted for
    /// the pass/fail decision (that's `evaluate_live_quality`, run
    /// against the exact bytes below), but retained as CID-addressed
    /// provenance of the offline top-1/agreement/bits measurement.
    #[arg(long)]
    evaluation_report: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct PackageReleaseBundleArgs {
    /// Compiled bundle directory to package (its physical_root, e.g.
    /// .uor-models/compiled/<name>).
    #[arg(long)]
    compiled: PathBuf,
    /// Public model identity this bundle serves.
    #[arg(long, default_value = "r4")]
    model_id: String,
    /// Declared serving capability.
    #[arg(long, value_enum)]
    capability: Capability,
    /// Original Hugging Face source snapshot this bundle was compiled from.
    /// Required for `--capability instruction-chat`: its registered adapter
    /// must exactly equal the bundle's persisted tokenizer_adapter.json.
    /// Continuation packaging may rely on the persisted adapter alone.
    #[arg(long)]
    source: Option<PathBuf>,
    /// Registered source-tokenizer family. Required together with
    /// --tokenizer-version whenever --source is given: this project
    /// never infers tokenizer identity (#718), the same policy `r4
    /// compile`'s own --tokenizer-family/--tokenizer-version pair
    /// enforces.
    #[arg(long, requires = "tokenizer_version")]
    tokenizer_family: Option<String>,
    /// Registered source-tokenizer adapter version. Must be supplied
    /// atomically with --tokenizer-family.
    #[arg(long, requires = "tokenizer_family")]
    tokenizer_version: Option<u32>,
    /// Full source revision of the graph compiler that produced this bundle.
    /// Required external release authority: it is not accepted from the
    /// deployed-quality report being packaged.
    #[arg(long)]
    compiler_revision: String,
    /// uor-matmul git revision the bundle's compile-time arithmetic ran
    /// against [default: the project's standing #655 pin].
    #[arg(long, default_value = "b13c98449948174f590e337c4dc25dfc394a07d0")]
    uor_matmul_rev: String,
    /// uor-matmul operation/codec profile name.
    #[arg(long, default_value = "exact-gemm-float")]
    uor_matmul_operation_profile: String,
    /// SPDX license identifier of the pinned uor-matmul source. NOTE: as
    /// of the standing pin, upstream's own Cargo.toml declares
    /// "Apache-2.0" while its checked-in LICENSE file text is an MIT
    /// license -- this default matches the LICENSE file; override once
    /// that upstream inconsistency is resolved.
    #[arg(long, default_value = "MIT")]
    uor_matmul_license: String,
    /// Free-text provenance pointer (e.g. an issue/PR reference).
    #[arg(long)]
    provenance_note: Option<String>,
    /// Write the manifest JSON here instead of
    /// `<compiled>/release-bundle.json`.
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct InstallReleaseArgs {
    /// Release tag to install (e.g. v0.1).
    #[arg(long)]
    tag: String,
    /// GitHub repository the release lives in.
    #[arg(long, default_value = "UOR-Foundation/uor-r4")]
    repo: String,
    /// Install name under the model store's compiled/ inventory
    /// [default: the release manifest's own model_id].
    #[arg(long)]
    name: Option<String>,
}

#[derive(Args, Debug)]
struct EvaluateReportArgs {
    /// Existing local Hugging Face model directory [default: .uor-models/sources/smollm2-135m-instruct].
    #[arg(long)]
    source: Option<PathBuf>,
    /// Compiled bundle directory [default: .uor-models/compiled/smollm2-135m-instruct].
    #[arg(long)]
    compiled: Option<PathBuf>,
    /// Evaluation report output path [default: <compiled>/instruction-eval.json].
    #[arg(long)]
    report: Option<PathBuf>,
    /// Teacher sequence length used for source-model loading.
    #[arg(long, default_value_t = 128)]
    sequence_length: usize,
    /// Registered source-tokenizer family. Must be supplied atomically with
    /// `--tokenizer-version`.
    #[arg(long, requires = "tokenizer_version")]
    tokenizer_family: Option<String>,
    /// Registered source-tokenizer adapter version. Must be supplied
    /// atomically with `--tokenizer-family`.
    #[arg(long, requires = "tokenizer_family")]
    tokenizer_version: Option<u32>,
}

#[derive(Args, Debug)]
struct GeometricDecoderSpikeArgs {
    /// Exact local Hugging Face source snapshot; no provider/network fallback.
    #[arg(long)]
    source: PathBuf,
    /// Full Hugging Face source commit bound by the snapshot cache tree.
    #[arg(
        long,
        default_value = uor_r4_wasm_router::geometric_decoder::PINNED_SOURCE_REVISION
    )]
    source_revision: String,
    /// Retained control/treatment transcript and operator report.
    #[arg(long, default_value = "docs/geometric_decoder_spike_950_raw.json")]
    output: PathBuf,
    /// Reloadable router state used for the persistence/restart probe.
    #[arg(long, default_value = "/tmp/uor-r4-issue-950-router-state.json")]
    router_state_output: PathBuf,
    /// Identity scope used for the retained user/assistant turns.
    #[arg(long, default_value = "issue-950-smoke")]
    identity: String,
    /// Fixed exact output-row workers for local `uor-matmul` projections.
    #[arg(long, default_value_t = 4)]
    workers: usize,
    /// Reuse only the five controls from an exact retained negative-treatment
    /// report; all source/decode bindings are revalidated before repair.
    #[arg(long)]
    control_report: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct R4SoftmaxGenerateArgs {
    /// Exact local pinned Hugging Face source snapshot; no network fallback.
    #[arg(long, default_value = ".uor-models/sources/smollm2-135m-instruct")]
    source: PathBuf,
    /// Full Hugging Face source commit bound by the snapshot cache tree.
    #[arg(
        long,
        default_value = uor_r4_wasm_router::geometric_decoder::PINNED_SOURCE_REVISION
    )]
    source_revision: String,
    /// Exact user prompt; the pinned source chat template is applied once.
    #[arg(long)]
    prompt: String,
    /// Maximum generated tokens (1..=128); EOS or a short cycle may stop sooner.
    #[arg(
        long,
        default_value_t = uor_r4_wasm_router::r4_softmax_reference_generation::DEFAULT_MAX_NEW_TOKENS
    )]
    max_tokens: usize,
    /// Fixed exact output-row workers for the source decoder projections.
    #[arg(long, default_value_t = 4)]
    workers: usize,
    /// Optional path for the complete source/policy/token/audit JSON report.
    #[arg(long)]
    json_output: Option<PathBuf>,
}

fn parse_r4_softmax_local_max_new_tokens(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be an integer in 1..=128".to_owned())?;
    if (1..=128).contains(&parsed) {
        Ok(parsed)
    } else {
        Err("must be in 1..=128".to_owned())
    }
}

#[derive(Args, Debug)]
struct R4SoftmaxLocalGenerateArgs {
    /// Local learned checkpoint directory; defaults to the working #1017 model.
    #[arg(long, default_value = ".uor-models/research/issue-1017/export")]
    model: PathBuf,
    /// Exact raw prompt; checkpoint BOS is prepended once and no chat template is applied.
    #[arg(long)]
    prompt: String,
    /// Maximum generated tokens (1..=128); EOS or a short cycle may stop sooner.
    #[arg(
        long,
        default_value_t = uor_r4_wasm_router::r4_softmax_local_generation::DEFAULT_MAX_NEW_TOKENS,
        value_parser = parse_r4_softmax_local_max_new_tokens
    )]
    max_new_tokens: usize,
    /// Isolated control: zero post-Wo attention output before residual addition.
    #[arg(long, default_value_t = false)]
    attention_off: bool,
    /// Stable seed for the versioned temperature-0.8/top-k-40 sampler; omit for greedy.
    #[arg(long)]
    seed: Option<u64>,
    /// Optional path for the complete checkpoint/policy/token/audit JSON report.
    #[arg(long)]
    json_output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct GroundedAnswerArgs {
    /// Frozen local #1017 checkpoint directory used to encode pointer states.
    #[arg(long, default_value = ".uor-models/research/issue-1017/export")]
    model: PathBuf,
    /// Research #954 pointer artifact bound to the frozen checkpoint.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-954/source-span-pointer/source-span-pointer.json"
    )]
    head: PathBuf,
    /// Exact regular, non-symlink UTF-8 source file used to ground the answer.
    #[arg(long)]
    source_file: PathBuf,
    /// Exact question in the sole admitted form: `Where is the <subject>?`.
    #[arg(long)]
    question: String,
    /// Optional path for the source binding, pointer scores, and R4 state audit.
    #[arg(long)]
    json_output: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct R4SoftmaxLocalQualifyArgs {
    /// Frozen local Hugging Face export directory.
    #[arg(long)]
    model: PathBuf,
    /// Explicit Python 32-token prefix fixture.
    #[arg(long)]
    python_prefix_logits: PathBuf,
    /// Optional final sealed-reveal manifest; omitted for the pre-campaign smoke export gate.
    #[arg(long)]
    reveal_manifest: Option<PathBuf>,
    /// Fixed exact output-row workers for source projections.
    #[arg(long, default_value_t = 4, value_parser = parse_positive_workers)]
    workers: usize,
    /// Closed qualification campaign; omit for the legacy #1014 two-arm mode.
    #[arg(long, value_enum, conflicts_with = "enabled_only")]
    campaign: Option<R4SoftmaxLocalQualificationCampaignArg>,
    /// Legacy spelling for the frozen #1017 enabled-only continuation.
    #[arg(long, default_value_t = false, conflicts_with = "campaign")]
    enabled_only: bool,
    /// Required durable qualification report path.
    #[arg(long)]
    json_output: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum R4SoftmaxLocalQualificationCampaignArg {
    #[value(name = "issue-1014")]
    Issue1014,
    #[value(name = "issue-1017")]
    Issue1017,
    #[value(name = "issue-1019")]
    Issue1019,
}

impl R4SoftmaxLocalQualificationCampaignArg {
    const fn campaign(
        self,
    ) -> uor_r4_wasm_router::r4_softmax_local_generation::R4SoftmaxLocalQualificationCampaign {
        use uor_r4_wasm_router::r4_softmax_local_generation::R4SoftmaxLocalQualificationCampaign;
        match self {
            Self::Issue1014 => R4SoftmaxLocalQualificationCampaign::Issue1014TwoArm,
            Self::Issue1017 => R4SoftmaxLocalQualificationCampaign::Issue1017EnabledOnly,
            Self::Issue1019 => R4SoftmaxLocalQualificationCampaign::Issue1019EnabledOnly,
        }
    }
}

fn parse_positive_workers(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "must be a positive integer".to_owned())?;
    if parsed == 0 {
        Err("must be greater than zero".to_owned())
    } else {
        Ok(parsed)
    }
}

#[derive(Args, Debug)]
struct R4SoftmaxTraceStudentArgs {
    /// Exact local pinned Hugging Face source snapshot; no network fallback.
    #[arg(long, default_value = ".uor-models/sources/smollm2-135m-instruct")]
    source: PathBuf,
    /// Full Hugging Face source commit bound by the snapshot cache tree.
    #[arg(
        long,
        default_value = uor_r4_wasm_router::geometric_decoder::PINNED_SOURCE_REVISION
    )]
    source_revision: String,
    /// Exact 40-character uor-r4 Git commit that built this evidence command.
    #[arg(long, default_value = "UNBOUND")]
    implementation_revision: String,
    /// Fixed exact output-row workers for the source decoder projections.
    #[arg(long, default_value_t = 8)]
    workers: usize,
    /// Stop after tokenizer-only partition and suffix-reachability arithmetic.
    #[arg(long, conflicts_with = "reveal")]
    preflight_only: bool,
    /// Reveal the held-out document against an already frozen construction artifact.
    #[arg(long)]
    reveal: bool,
    /// Frozen source-free student artifact.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-trace-student-v1.bin"
    )]
    artifact_output: PathBuf,
    /// Construction-only R4 attention teacher trace.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-teacher-trace-v1.bin"
    )]
    trace_output: PathBuf,
    /// Construction-only freeze manifest binding trace and artifact CIDs.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-trace-freeze-v1.json"
    )]
    freeze_output: PathBuf,
    /// Decision-bearing held-out result and replay audit.
    #[arg(long, default_value = "docs/r4_softmax_trace_student_973_raw.json")]
    result_output: PathBuf,
}

#[derive(Args, Debug)]
struct R4SoftmaxTraceStateCompileArgs {
    /// Exact 40-character uor-r4 Git commit that built this evidence command.
    #[arg(long)]
    implementation_revision: String,
    /// Frozen four-document R4/Spin teacher trace bundle from #1010.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-teacher-trace-v1.bin"
    )]
    trace_bundle: PathBuf,
    /// Frozen predecessor construction manifest from #1010.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-trace-freeze-v1.json"
    )]
    predecessor_freeze: PathBuf,
    /// Frozen source-free suffix artifact from #1010.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-trace-student-v1.bin"
    )]
    suffix_artifact: PathBuf,
    /// Compiled recurrent state artifact output.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-student-v1.bin"
    )]
    artifact_output: PathBuf,
    /// Construction-only state freeze output.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-freeze-v1.json"
    )]
    freeze_output: PathBuf,
}

#[derive(Args, Debug)]
struct R4SoftmaxTraceStateSealArgs {
    /// Frozen recurrent state artifact; no source checkpoint is accepted.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-student-v1.bin"
    )]
    artifact: PathBuf,
    /// Phase-1 provenance freeze binding the exact artifact CID and construction inputs.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-freeze-v1.json"
    )]
    freeze: PathBuf,
    /// Frozen held-out causal token inputs containing no labels or teacher rows.
    #[arg(
        long,
        default_value = "docs/r4_softmax_trace_state_causal_input_1011.json"
    )]
    causal_input: PathBuf,
    /// Immutable prediction/state/provenance seal output.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-seal-v1.json"
    )]
    seal_output: PathBuf,
}

#[derive(Args, Debug)]
struct R4SoftmaxTraceStateRevealArgs {
    /// Immutable source-free seal produced before judge reveal.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-1011/r4-softmax-trace-state-seal-v1.json"
    )]
    seal: PathBuf,
    /// Frozen #1010 held-out judge JSON, hard-bound by its result CID.
    #[arg(long, default_value = "docs/r4_softmax_trace_student_973_raw.json")]
    judge: PathBuf,
    /// Decision-bearing #1011 result output.
    #[arg(
        long,
        default_value = "docs/r4_softmax_trace_state_student_1011_raw.json"
    )]
    result_output: PathBuf,
}

#[derive(Args, Debug)]
struct R4SoftmaxTraceObservabilityArgs {
    /// Exact 40-character uor-r4 Git commit that built this evidence command.
    #[arg(long)]
    implementation_revision: String,
    /// Frozen four-document R4/Spin construction trace bundle from #1010.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-teacher-trace-v1.bin"
    )]
    trace_bundle: PathBuf,
    /// Frozen #1010 construction manifest that binds the trace and teacher policy.
    #[arg(
        long,
        default_value = ".uor-models/research/issue-973/r4-softmax-trace-freeze-v1.json"
    )]
    predecessor_freeze: PathBuf,
    /// Construction-only four-fold observability result.
    #[arg(
        long,
        default_value = "docs/r4_softmax_trace_observability_1012_raw.json"
    )]
    result_output: PathBuf,
}

#[derive(Args, Debug)]
struct ServeArgs {
    /// Explicitly expose the local-only native R4/Spin softmax reference route.
    #[arg(long)]
    enable_r4_softmax_reference: bool,
    /// Exact local pinned source snapshot used only by the opt-in reference route.
    #[arg(long, requires = "enable_r4_softmax_reference")]
    r4_softmax_source: Option<PathBuf>,
    /// Operator-owned projection workers (defaults to up to 8, capped by process parallelism).
    #[arg(long, requires = "enable_r4_softmax_reference")]
    r4_softmax_workers: Option<usize>,
}

#[derive(Args, Debug)]
struct GeometricMixerQualificationArgs {
    /// Exact local Hugging Face source snapshot. Required for fitting; ignored
    /// by source-free preflight and review finalization modes.
    #[arg(long)]
    source: Option<PathBuf>,
    /// Full Hugging Face source commit bound by the snapshot cache tree.
    #[arg(
        long,
        default_value = uor_r4_wasm_router::geometric_decoder::PINNED_SOURCE_REVISION
    )]
    source_revision: String,
    /// Execute only the three source-free hard preflight gates.
    #[arg(long, conflicts_with = "finalize_review")]
    preflight_only: bool,
    /// Apply a human review file to an existing machine qualification report.
    #[arg(long, value_name = "REVIEW_JSON", conflicts_with = "preflight_only")]
    finalize_review: Option<PathBuf>,
    /// Passing source-free preflight report consumed before source access.
    #[arg(long, default_value = "/tmp/uor-r4-issue-951-preflight.json")]
    preflight_report: PathBuf,
    /// Source-free checkpoint used only by the checkpoint preflight.
    #[arg(
        long,
        default_value = "/tmp/uor-r4-issue-951-preflight-checkpoint.json"
    )]
    preflight_checkpoint: PathBuf,
    /// Authoritative retained G0 report and student-prefix transcript.
    #[arg(long, default_value = "docs/geometric_decoder_spike_950_raw.json")]
    g0_report: PathBuf,
    /// Compact G1 metrics, controls, and rollout report.
    #[arg(
        long,
        default_value = "docs/geometric_mixer_qualification_951_raw.json"
    )]
    output: PathBuf,
    /// Accepted or retained-negative deterministic mixer checkpoint.
    #[arg(long, default_value = "docs/geometric_mixer_checkpoint_951.json")]
    checkpoint: PathBuf,
    /// Identity scope used for each matched persistent-memory rollout.
    #[arg(long, default_value = "issue-951-qualification")]
    identity: String,
    /// Fixed exact output-row workers for source and mixer projections.
    #[arg(long, default_value_t = 4)]
    workers: usize,
    /// Fixed dataset, initialization, negative-sampling, and fitting seed.
    #[arg(long, default_value_t = uor_r4_wasm_router::geometric_mixer_qualification::DEFAULT_SEED)]
    seed: u64,
    /// Mixer-specific full-batch steps in each of at most three rounds.
    #[arg(
        long,
        default_value_t = uor_r4_wasm_router::geometric_mixer_qualification::MAX_STEPS_PER_ROUND
    )]
    steps_per_round: usize,
}

impl Cli {
    fn server_config(&self) -> ServerConfig {
        ServerConfig {
            host: self.host.clone(),
            port: self.port,
            manifold_cache: self.manifold_cache.clone(),
            tless_artifacts: self.tless_artifacts.clone(),
            tless_store: self.tless_store.clone(),
            tless_tokenizer: self.tless_tokenizer.clone(),
            r4g1_artifact: self.r4g1_artifact.clone(),
            tless_corpus_meta: self.tless_corpus_meta.clone(),
            tless_corpus_recs: self.tless_corpus_recs.clone(),
            geometric_demo: false,
            r4_softmax_reference: None,
        }
    }

    fn configure_tless(&self) {
        tless_uor::configure_tless_paths(tless_uor::TlessPaths {
            artifacts: self.tless_artifacts.clone(),
            store: self.tless_store.clone(),
            tokenizer: self.tless_tokenizer.clone(),
        });
    }
}

fn resolve_r4_softmax_reference_workers(
    requested: Option<usize>,
) -> Result<std::num::NonZeroUsize, String> {
    let available = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1);
    let workers =
        requested.unwrap_or(server::R4_SOFTMAX_REFERENCE_HTTP_DEFAULT_WORKERS.min(available));
    let workers = std::num::NonZeroUsize::new(workers)
        .ok_or_else(|| "--r4-softmax-workers must be greater than zero".to_owned())?;
    if workers.get() > available {
        return Err(format!(
            "--r4-softmax-workers={} exceeds the {available} workers available to this process",
            workers.get()
        ));
    }
    Ok(workers)
}

trait Chat {
    fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError>;
}

impl Chat for ChatEngine {
    fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
        ChatEngine::ask(self, question)
    }
}

#[derive(Debug)]
enum RunError {
    Chat(ChatError),
    Model(ModelError),
    Io(io::Error),
    Command(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chat(error) => error.fmt(formatter),
            Self::Model(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::Command(error) => formatter.write_str(error),
        }
    }
}

impl From<ChatError> for RunError {
    fn from(error: ChatError) -> Self {
        Self::Chat(error)
    }
}

impl From<ModelError> for RunError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

impl From<io::Error> for RunError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// #811: render one turn honestly — served text as before, and a typed
/// D4 abstention as an explicit line instead of empty output or a
/// confidently served guess.
fn rendered_answer(answer: &ChatAnswer) -> String {
    match &answer.abstention {
        None => answer.text.clone(),
        Some(abstention) => format!(
            "[{}: cause={} coverage={} d4={}{}] no answer served — the model's \
             D4 policy declined this prompt as outside its certified context \
             (typed selective-prediction record, RF-30; no confidence value \
             exists in legacy-coverage mode).",
            abstention.outcome,
            abstention.cause,
            abstention.coverage,
            abstention.status,
            if abstention.widened {
                ", widened once"
            } else {
                ""
            }
        ),
    }
}

fn answer_once(
    chat: &mut impl Chat,
    question: &str,
    output: &mut impl Write,
) -> Result<(), RunError> {
    let answer = chat.ask(question)?;
    writeln!(output, "{}", rendered_answer(&answer))?;
    Ok(())
}

fn interactive_chat(
    chat: &mut impl Chat,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), io::Error> {
    writeln!(output, "R⁴ Router — interactive transformerless chat")?;
    writeln!(output, "type 'exit' or Ctrl-D to quit\n")?;
    loop {
        write!(output, "you> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let question = line.trim();
        if matches!(question, "exit" | "quit") {
            break;
        }
        if question.is_empty() {
            continue;
        }
        match chat.ask(question) {
            Ok(answer) => writeln!(output, "r4> {}\n", rendered_answer(&answer))?,
            Err(error) => tracing::error!(%error, "chat response failed"),
        }
    }
    Ok(())
}

/// Build the local chat engine under the #655 decode default
/// (2026-08-19): seeded weighted sampling unless `--greedy` opts out,
/// with `--sample <SEED>` overriding the pinned default seed. The
/// library builder itself stays explicit (`sample_seed` opt-in); the
/// default is applied here, at the product surface, exactly like the
/// server's request mapping.
fn build_chat_engine(
    model: Option<&str>,
    sample: Option<u32>,
    greedy: bool,
    research: bool,
) -> Result<ChatEngine, ChatError> {
    let mut builder =
        ChatEngine::builder().model(model.map_or_else(default_model_reference, ToOwned::to_owned));
    if !greedy {
        builder = builder.sample_seed(sample.unwrap_or(DEFAULT_SAMPLE_SEED));
    }
    if research {
        let (engine, warning) = builder.build_for_research()?;
        eprintln!("warning: {warning}");
        Ok(engine)
    } else {
        builder.build()
    }
}

fn compile(args: &CompileArgs) -> Result<(), RunError> {
    if args.sequence_length == 0 {
        return Err(RunError::Command(
            "--sequence-length must be greater than zero".to_owned(),
        ));
    }
    if let (Some(corpus_meta), Some(corpus_recs), Some(vocab_size)) =
        (&args.corpus_meta, &args.corpus_recs, args.vocab_size)
    {
        if args.source.is_some() || args.model.is_some() || args.revision.is_some() {
            return Err(RunError::Command(
                "--corpus-meta/--corpus-recs cannot be combined with a teacher source".to_owned(),
            ));
        }
        let output = args
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from(".uor-models/compiled/recorded"));
        let values = vec![
            "compile-recorded".to_owned(),
            "--corpus-meta".to_owned(),
            corpus_meta.display().to_string(),
            "--corpus-recs".to_owned(),
            corpus_recs.display().to_string(),
            "--vocab-size".to_owned(),
            vocab_size.to_string(),
            "--out".to_owned(),
            output.display().to_string(),
        ];
        return transformerless_command::run(&values)
            .map_err(|error| RunError::Command(error.to_string()));
    }
    let mut values = Vec::new();
    if let Some(source) = &args.source {
        values.extend(["--source".to_owned(), source.display().to_string()]);
    }
    if let Some(model) = &args.model {
        values.extend(["--model".to_owned(), model.clone()]);
    }
    if let Some(revision) = &args.revision {
        values.extend(["--revision".to_owned(), revision.clone()]);
    }
    match (&args.tokenizer_family, args.tokenizer_version) {
        (Some(family), Some(version)) => values.extend([
            "--tokenizer-family".to_owned(),
            family.clone(),
            "--tokenizer-version".to_owned(),
            version.to_string(),
        ]),
        (None, None) => {}
        _ => {
            return Err(RunError::Command(
                "--tokenizer-family and --tokenizer-version must be supplied together".to_owned(),
            ));
        }
    }
    if let Some(output) = &args.output {
        values.extend(["--output".to_owned(), output.display().to_string()]);
    }
    values.extend(["--seconds".to_owned(), args.seconds.to_string()]);
    values.extend(["--target".to_owned(), args.target.to_string()]);
    values.extend([
        "--sequence-length".to_owned(),
        args.sequence_length.to_string(),
    ]);
    if args.r4_attention {
        values.push("--r4-attention".to_owned());
    }
    if args.exact_scalar {
        std::env::set_var("TLESS_EXACT_SCALAR", "1");
    }
    if args.canonical_deterministic {
        std::env::set_var("TLESS_CANONICAL_DETERMINISTIC", "1");
    }
    transformerless_command::compile_hugging_face(&values)
        .map_err(|error| RunError::Command(error.to_string()))
}

fn download(args: &DownloadArgs) -> Result<(), RunError> {
    let path = download_source(&SourceDownload {
        repository: args.repository.clone(),
        revision: args.revision.clone(),
        name: args.name.clone(),
        output: args.output.clone(),
        license: descriptor_license(&args.name, &args.repository, &args.revision),
    })?;
    println!("{}", path.display());
    Ok(())
}

/// SPDX license identifier from the pinned `models/<name>.json`
/// descriptor, forwarded into the #597 source-snapshot manifest — only
/// when the descriptor pins exactly the requested repository and
/// revision. Any miss (no descriptor, malformed JSON, different pin)
/// yields `None`; the snapshot's license file is digested either way.
fn descriptor_license(name: &str, repository: &str, revision: &str) -> Option<String> {
    let bytes = std::fs::read(Path::new("models").join(format!("{name}.json"))).ok()?;
    let descriptor: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if descriptor.get("repository")?.as_str()? != repository
        || descriptor.get("revision")?.as_str()? != revision
    {
        return None;
    }
    descriptor
        .get("license")?
        .as_str()
        .map(|license| license.to_owned())
}

fn import(args: &ImportArgs) -> Result<(), RunError> {
    let model_store = ModelStore::from_env();
    let artifact_bytes = std::fs::read(&args.artifacts)?;
    let store_bytes = std::fs::read(&args.store)?;
    let tokenizer_bytes = std::fs::read(&args.tokenizer)?;
    let r4g1_bytes = args.r4g1.as_ref().map(std::fs::read).transpose()?;
    // Quality is DERIVED, not accepted as operator input (#744): a
    // manually-typed `--grounded-answer-rate`/`--repetition-rate` could
    // (and did) disagree with the actual compiled bytes and with itself
    // across re-imports of the same artifact. Continuation-capability
    // bundles are never chat-gated (`validate_for_chat` below), so there
    // is nothing honest to compute for them; they get a fixed
    // not-evaluated attestation instead of running probes that would
    // never be consulted. When `--r4g1` is supplied, #750 requires that
    // path to independently pass too, since `ask`/`chat` will prefer it
    // over the plain path at serving time.
    let quality = match args.capability {
        Capability::InstructionChat => evaluate_live_quality(
            &artifact_bytes,
            &store_bytes,
            &tokenizer_bytes,
            r4g1_bytes.as_deref(),
        )?,
        Capability::Continuation => QualityAttestation {
            instruction_eval_passed: false,
            grounded_answer_rate: 0.0,
            repetition_rate: 1.0,
            r4g1_grounded_answer_rate: None,
            r4g1_repetition_rate: None,
        },
    };
    let artifacts = model_store.put(&artifact_bytes)?;
    let store = model_store.put(&store_bytes)?;
    let tokenizer = model_store.put(&tokenizer_bytes)?;
    let evaluation_report = args
        .evaluation_report
        .as_ref()
        .map(std::fs::read)
        .transpose()?
        .map(|bytes| model_store.put(&bytes))
        .transpose()?;
    let manifest = ModelManifest {
        schema: 1,
        name: args.name.clone(),
        source_model: args.source_model.clone(),
        capability: args.capability.into(),
        artifacts,
        store,
        tokenizer,
        evaluation_report,
        quality,
    };
    manifest.validate_for_chat().or_else(|error| {
        (manifest.capability == ModelCapability::Continuation)
            .then_some(())
            .ok_or(error)
    })?;
    println!("{}", model_store.write_manifest(&manifest)?);
    Ok(())
}

/// #655-D2: resolve a real `tokenizer_adapter` (for `--capability
/// instruction-chat`, from `--source`) or an empty one (for `--capability
/// continuation`, valid since `ReleaseBundleManifest::validate` only
/// requires a real adapter family for instruction-chat bundles), then
/// call `release_bundle_packager::package_release_bundle` and write its
/// result to `--output` (default `<compiled>/release-bundle.json`).
fn package_release_bundle_command(args: &PackageReleaseBundleArgs) -> Result<(), RunError> {
    let capability = match args.capability {
        Capability::Continuation => BundleCapability::Continuation,
        Capability::InstructionChat => BundleCapability::InstructionChat,
    };
    let admission = release_bundle_packager::verify_bundle_for_production_packaging(
        &args.compiled,
        &args.compiler_revision,
    )
    .map_err(|error| RunError::Command(error.to_string()))?;
    let tokenizer_adapter = match &args.source {
        Some(source) => {
            let key = match (&args.tokenizer_family, args.tokenizer_version) {
                (Some(family), Some(version)) => TokenizerAdapterKey::new(family.clone(), version),
                _ => {
                    return Err(RunError::Command(
                        "--tokenizer-family and --tokenizer-version are required together with \
                         --source (this project never infers tokenizer identity, #718)"
                            .to_owned(),
                    ));
                }
            };
            let resolved = resolve_source_tokenizer(source, Some(&key))
                .map_err(|error| RunError::Command(format!("--source: {}", error.reason)))?;
            let resolved_adapter = resolved.adapter().ok_or_else(|| {
                RunError::Command(format!(
                    "{} resolved to an adapterless tokenizer for {}/{}",
                    source.display(),
                    key.family,
                    key.version
                ))
            })?;
            if resolved_adapter != admission.tokenizer_adapter {
                return Err(RunError::Command(format!(
                    "{} resolves tokenizer adapter {:?}, but the captured bundle binds {:?}",
                    source.display(),
                    resolved_adapter,
                    admission.tokenizer_adapter
                )));
            }
            resolved_adapter
        }
        None if capability == BundleCapability::InstructionChat => {
            return Err(RunError::Command(
                "--source is required for --capability instruction-chat: physical_root alone \
                 cannot supply a real tokenizer_adapter"
                    .to_owned(),
            ));
        }
        None => admission.tokenizer_adapter.clone(),
    };

    let inputs = PackageInputs {
        model_id: args.model_id.clone(),
        capability,
        uor_matmul: UorMatmulProvenance {
            rev: args.uor_matmul_rev.clone(),
            operation_profile: args.uor_matmul_operation_profile.clone(),
            license: args.uor_matmul_license.clone(),
            source_digest: None,
        },
        tokenizer_adapter,
        selector: admission.bindings.selector.clone(),
        compiler: admission.bindings.compiler.clone(),
        provenance_note: args.provenance_note.clone(),
    };

    let manifest = release_bundle_packager::package_verified_release_bundle(
        &args.compiled,
        inputs,
        &args.compiler_revision,
    )
    .map_err(|error| RunError::Command(error.to_string()))?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.compiled.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME));
    let json = serde_json::to_vec_pretty(&manifest).map_err(|error| {
        RunError::Command(format!("serialize release-bundle manifest: {error}"))
    })?;
    std::fs::write(&output_path, json)?;
    println!("{}", output_path.display());
    Ok(())
}

/// #741: the explicit verified fetch. All verification lives in
/// `release_install::install_release`; this handler supplies the real
/// curl fetcher and the model-store root, then reports the installed
/// identity so the user can bind what they fetched to the release tag.
fn install_release_command(args: &InstallReleaseArgs) -> Result<(), RunError> {
    let store_root = uor_r4_wasm_router::model::model_store_root();
    let request = uor_r4_wasm_router::release_install::InstallReleaseRequest {
        repo: args.repo.clone(),
        tag: args.tag.clone(),
        name: args.name.clone(),
    };
    let installed = uor_r4_wasm_router::release_install::install_release(
        &store_root,
        &request,
        &mut uor_r4_wasm_router::release_install::CurlFetcher,
    )
    .map_err(RunError::Command)?;
    println!("installed: {}", installed.destination.display());
    println!("release: {} @ {}", args.repo, args.tag);
    println!(
        "model_id: {} ({:?})",
        installed.manifest.model_id, installed.manifest.capability
    );
    println!("graph: {}", installed.manifest.components.graph);
    println!(
        "signature_artifact: {}",
        installed.manifest.components.signature_artifact
    );
    if let Some(tokenizer) = installed.manifest.components.tokenizer.as_deref() {
        println!("tokenizer: {tokenizer}");
    }
    println!(
        "every component digest verified against the release's attested manifest; \
         the sidecar is installed beside the bundle for serving-time verification"
    );
    Ok(())
}

fn evaluate_report(args: &EvaluateReportArgs) -> Result<(), RunError> {
    if args.sequence_length == 0 {
        return Err(RunError::Command(
            "--sequence-length must be greater than zero".to_owned(),
        ));
    }
    let mut values = Vec::new();
    if let Some(source) = &args.source {
        values.extend(["--source".to_owned(), source.display().to_string()]);
    }
    if let Some(compiled) = &args.compiled {
        values.extend(["--compiled".to_owned(), compiled.display().to_string()]);
    }
    if let Some(report) = &args.report {
        values.extend(["--report".to_owned(), report.display().to_string()]);
    }
    match (&args.tokenizer_family, args.tokenizer_version) {
        (Some(family), Some(version)) => values.extend([
            "--tokenizer-family".to_owned(),
            family.clone(),
            "--tokenizer-version".to_owned(),
            version.to_string(),
        ]),
        (None, None) => {}
        _ => {
            return Err(RunError::Command(
                "--tokenizer-family and --tokenizer-version must be supplied together".to_owned(),
            ));
        }
    }
    values.extend([
        "--sequence-length".to_owned(),
        args.sequence_length.to_string(),
    ]);
    run_core("evaluate-report", &values)
}

fn run_core(name: &str, arguments: &[String]) -> Result<(), RunError> {
    let mut values = vec![name.to_owned()];
    values.extend_from_slice(arguments);
    transformerless_command::run(&values).map_err(|error| RunError::Command(error.to_string()))
}

fn route_demo(args: &RouteArgs) -> Result<(), RunError> {
    let prompt = args.prompt.join(" ");
    let mut router = uor_r4_wasm_router::UorR4Router::new(0.85);
    let indexed_from_file = if let Some(path) = &args.corpus {
        let corpus = std::fs::read_to_string(path).map_err(|error| {
            RunError::Command(format!(
                "read demonstration corpus {}: {error}",
                path.display()
            ))
        })?;
        router.index_corpus(&corpus, &args.identity)
    } else {
        0
    };

    let routing = router.route_query_to_manifold_native(&prompt, &args.identity);
    let resonances = router.get_top_resonances_native(&prompt, &args.identity, 3);
    let routed = &routing.routed;
    let summary = serde_json::json!({
        "surface": "uor-r4-geometric-router-research-demo/1",
        "status": "route-only-research-demo",
        "capability": "geometric routing, indexed retrieval, and route telemetry",
        "not_established": [
            "source-free attention",
            "coherent inference",
            "answer correctness",
            "reasoning"
        ],
        "prompt": prompt,
        "identity": args.identity,
        "indexed_sentences_from_file": indexed_from_file,
        "route": {
            "window_index": routed.window_index,
            "scale_x": routed.scale_x,
            "uor_address": routed.uor_address,
            "kappa": routed.metrics.kappa,
            "deficit_angle": routed.metrics.deficit_angle,
            "entropy": routed.metrics.lambda_entropy,
            "hopf_sector": routed.hopf.sector_id,
            "hopf_chi": routed.hopf.chi,
            "transported_alpha": routed.hopf.transported_alpha,
        },
        "candidate_routes": routing.all_routes.iter().map(|candidate| serde_json::json!({
            "window_index": candidate.window_index,
            "scale_x": candidate.scale_x,
            "routing_score": candidate.routing_score,
            "kappa": candidate.kappa,
            "deficit_angle": candidate.deficit_angle,
        })).collect::<Vec<_>>(),
        "nearest_indexed_routes": resonances,
    });

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summary)
                .map_err(|error| RunError::Command(error.to_string()))?
        );
    } else {
        println!("UOR-R4 geometric router research demo");
        println!("  prompt: {prompt}");
        println!(
            "  route: W{} · scale {:.0} · κ {:.4} · θd {:.4}",
            routed.window_index, routed.scale_x, routed.metrics.kappa, routed.metrics.deficit_angle
        );
        println!(
            "  Hopf: sector {} · χ {:.4} · transported α {:.4}",
            routed.hopf.sector_id, routed.hopf.chi, routed.hopf.transported_alpha
        );
        println!("  UOR address: {}", routed.uor_address);
        if let Some(nearest) = resonances.first() {
            println!("  nearest indexed route: {}", nearest.sentence);
        }
        if indexed_from_file > 0 {
            if let Some(path) = &args.corpus {
                println!("  indexed from {}: {indexed_from_file}", path.display());
            }
        }
        println!(
            "  status: routing/retrieval demonstration only; coherent inference, correctness, and reasoning are not established"
        );
        println!("  use --json for the full candidate-route summary");
    }
    Ok(())
}

fn bounded_geometric_generate(args: &BoundedGeometricGenerateArgs) -> Result<(), RunError> {
    let artifact_bytes = std::fs::read(&args.artifact).map_err(|error| {
        RunError::Command(format!(
            "read canonical route artifact {}: {error}",
            args.artifact.display()
        ))
    })?;
    let generator =
        LocalGeometricGenerator::from_canonical_bytes(&artifact_bytes).map_err(|error| {
            RunError::Command(format!(
                "load canonical route artifact {}: {error}",
                args.artifact.display()
            ))
        })?;
    let report = generator
        .generate(
            args.prompt.as_bytes(),
            args.control.generation_control(),
            args.continuation_cap,
        )
        .map_err(|error| RunError::Command(format!("bounded geometric generation: {error}")))?;

    let mut stdout = io::stdout().lock();
    if args.json {
        let bytes = report
            .canonical_bytes()
            .map_err(|error| RunError::Command(format!("serialize #953 report: {error}")))?;
        stdout.write_all(&bytes)?;
        stdout.write_all(b"\n")?;
    } else {
        stdout.write_all(b"continuation:")?;
        stdout.write_all(&report.continuation_bytes)?;
        stdout.write_all(b"\nstop: ")?;
        stdout.write_all(local_generation_stop_label(report.stop_reason).as_bytes())?;
        stdout.write_all(b"\n")?;
    }
    Ok(())
}

const SOURCE_FREE_TABLE_REPORT_SCHEMA: &str = "uor-r4-source-free-table-report/1";
const SOURCE_FREE_TABLE_POSITIVE: &str = "NUMERIC_BASELINE_GATE_MET_PENDING_REPLAY_BINDING";
const SOURCE_FREE_TABLE_NEGATIVE: &str = "REPAIR_LEXICAL_REPRESENTATION_OR_COUNT_OBJECTIVE";
const SOURCE_FREE_GEOMETRIC_REPORT_SCHEMA: &str =
    "uor-r4-source-free-table-geometric-intervention-report/1";
const SOURCE_FREE_GEOMETRIC_MECHANISM: &str = "MultiscaleCountRadiusR4V1";
const SOURCE_FREE_GEOMETRIC_PENDING_REPLAY: &str = "PROCEED_TO_A1Q_H_PENDING_BYTE_IDENTICAL_REPLAY";
const SOURCE_FREE_GEOMETRIC_NEGATIVE: &str = "RETAIN_TABLE_BASELINE_GEOMETRY_NO_INCREMENT";
const SOURCE_FREE_FROZEN_CORPUS_CID: &str =
    "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const SOURCE_FREE_FROZEN_ARTIFACT_CID: &str =
    "blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297";
const SOURCE_FREE_FROZEN_DOCUMENTS: usize = 3_000;
const SOURCE_FREE_FROZEN_CONSTRUCTION_DOCUMENTS: usize = 2_404;
const SOURCE_FREE_FROZEN_HELD_OUT_DOCUMENTS: usize = 596;
const SOURCE_FREE_FROZEN_KNOWN_TARGETS: u64 = 446_342;
const SOURCE_FREE_FROZEN_BASELINE_CORRECT: u64 = 99_362;
const SOURCE_FREE_FROZEN_PROMPT: &str = "The United States";
const SOURCE_FREE_FROZEN_CONTINUATION_CAP: usize = 16;

#[derive(Debug, Deserialize)]
struct SourceFreeCorpusArticle {
    id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct SourceFreeCorpusManifest {
    article_count: usize,
    corpus_cid: String,
}

#[derive(Debug, Serialize)]
struct SourceFreeClosureCounters {
    teacher_calls: u64,
    provider_calls: u64,
    source_weight_reads: u64,
    geometry_operations: u64,
}

#[derive(Debug, Serialize)]
struct SourceFreeCriteria {
    pinned_three_thousand_documents: bool,
    at_least_one_hundred_thousand_known_targets: bool,
    nonzero_context_coverage: bool,
    at_least_five_percentage_point_uplift: bool,
    continuation_at_least_four_units: bool,
    continuation_has_no_period_one_or_two_cycle: bool,
    continuation_is_utf8: bool,
    external_byte_identical_replay_required: bool,
}

#[derive(Debug, Serialize)]
struct SourceFreeContinuationReport {
    prompt: String,
    continuation_cap: usize,
    emitted_units: usize,
    decoded_text: String,
    stop: &'static str,
}

#[derive(Debug, Serialize)]
struct SourceFreeTableReport {
    schema: &'static str,
    decision: &'static str,
    capability: &'static str,
    nonclaims: &'static str,
    corpus_cid: String,
    manifest_cid: String,
    document_count: usize,
    construction_documents: usize,
    held_out_documents: usize,
    unique_document_ids: usize,
    lexical_routes: usize,
    artifact_bytes: usize,
    artifact_cid: String,
    held_out_encoded_positions: u64,
    held_out_known_target_positions: u64,
    table_correct: u64,
    unigram_correct: u64,
    table_top1_percent: String,
    unigram_top1_percent: String,
    uplift_percentage_points: String,
    trigram_choices: u64,
    bigram_choices: u64,
    unigram_choices: u64,
    changed_choices: u64,
    changed_choice_correct: u64,
    continuation: SourceFreeContinuationReport,
    source_closure: SourceFreeClosureCounters,
    criteria: SourceFreeCriteria,
}

#[derive(Debug, Serialize)]
struct SourceFreeTableEnvelope {
    report_payload_cid: String,
    report: SourceFreeTableReport,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricArtifactReport {
    bytes: usize,
    cid: String,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricOverlayStatsReport {
    eligible_bigram_rows: u64,
    eligible_trigram_rows: u64,
    geometry_changed_bigram_rows: u64,
    geometry_changed_trigram_rows: u64,
    geometry_changed_rows: u64,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricContinuationReport {
    prompt: String,
    continuation_cap: usize,
    emitted_units: usize,
    tokens: Vec<u32>,
    decoded_text: String,
    stop: &'static str,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricClosureCounters {
    teacher_calls: u64,
    provider_calls: u64,
    source_weight_reads: u64,
    held_out_fit_reads: u64,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricCriteria {
    frozen_corpus_cid: bool,
    frozen_document_partition: bool,
    frozen_known_target_positions: bool,
    frozen_baseline_correct: bool,
    frozen_table_artifact_cid: bool,
    frozen_prompt_and_cap: bool,
    overlay_round_trip_byte_identical: bool,
    overlay_table_binding_matches_base: bool,
    baseline_continuation_matches_unchanged_table: bool,
    geometric_correct_strictly_greater_than_frozen_baseline: bool,
    changed_choice_geometric_advantage: bool,
    nonzero_reachable_ties: bool,
    teacher_forced_support_identical: bool,
    teacher_forced_work_identical: bool,
    fixed_prompt_first_divergence: bool,
    first_divergence_within_continuation_cap: bool,
    pre_divergence_frame_identical: bool,
    pre_divergence_work_identical: bool,
    geometric_output_distinct: bool,
    geometric_continuation_at_least_four_units: bool,
    geometric_continuation_has_no_period_one_or_two_cycle: bool,
    geometric_continuation_is_utf8: bool,
    source_closure_counters_zero: bool,
    external_byte_identical_replay_required: bool,
}

impl SourceFreeGeometricCriteria {
    fn passes_before_replay(&self) -> bool {
        self.frozen_corpus_cid
            && self.frozen_document_partition
            && self.frozen_known_target_positions
            && self.frozen_baseline_correct
            && self.frozen_table_artifact_cid
            && self.frozen_prompt_and_cap
            && self.overlay_round_trip_byte_identical
            && self.overlay_table_binding_matches_base
            && self.baseline_continuation_matches_unchanged_table
            && self.geometric_correct_strictly_greater_than_frozen_baseline
            && self.changed_choice_geometric_advantage
            && self.nonzero_reachable_ties
            && self.teacher_forced_support_identical
            && self.teacher_forced_work_identical
            && self.fixed_prompt_first_divergence
            && self.first_divergence_within_continuation_cap
            && self.pre_divergence_frame_identical
            && self.pre_divergence_work_identical
            && self.geometric_output_distinct
            && self.geometric_continuation_at_least_four_units
            && self.geometric_continuation_has_no_period_one_or_two_cycle
            && self.geometric_continuation_is_utf8
            && self.source_closure_counters_zero
    }
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricReport {
    schema: &'static str,
    decision: &'static str,
    mechanism: &'static str,
    capability: &'static str,
    nonclaims: &'static str,
    corpus_cid: String,
    manifest_cid: String,
    document_count: usize,
    construction_documents: usize,
    held_out_documents: usize,
    unique_document_ids: usize,
    lexical_routes: usize,
    base_artifact: SourceFreeGeometricArtifactReport,
    overlay_artifact: SourceFreeGeometricArtifactReport,
    overlay_table_artifact_cid: String,
    overlay_stats: SourceFreeGeometricOverlayStatsReport,
    held_out_encoded_positions: u64,
    held_out_known_target_positions: u64,
    baseline_correct: u64,
    geometric_correct: u64,
    baseline_top1_percent: String,
    geometric_top1_percent: String,
    geometric_delta_percentage_points: String,
    reachable_tie_positions: u64,
    changed_choices: u64,
    geometric_changed_correct: u64,
    baseline_changed_correct: u64,
    support_mismatches: u64,
    work_mismatches: u64,
    trigram_choices: u64,
    bigram_choices: u64,
    unigram_choices: u64,
    baseline_continuation: SourceFreeGeometricContinuationReport,
    geometric_continuation: SourceFreeGeometricContinuationReport,
    first_divergence: Option<GeometricFirstDivergence>,
    source_closure: SourceFreeGeometricClosureCounters,
    criteria: SourceFreeGeometricCriteria,
}

#[derive(Debug, Serialize)]
struct SourceFreeGeometricEnvelope {
    report_payload_cid: String,
    report: SourceFreeGeometricReport,
}

fn source_free_table(args: &SourceFreeTableArgs) -> Result<(), RunError> {
    let corpus_bytes = std::fs::read(&args.corpus).map_err(|error| {
        RunError::Command(format!(
            "read source-free corpus {}: {error}",
            args.corpus.display()
        ))
    })?;
    let corpus_cid = format!("blake3:{}", blake3::hash(&corpus_bytes).to_hex());
    let manifest_path = args
        .corpus
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path).map_err(|error| {
        RunError::Command(format!(
            "read source-free corpus manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    let manifest: SourceFreeCorpusManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|error| {
            RunError::Command(format!(
                "parse source-free corpus manifest {}: {error}",
                manifest_path.display()
            ))
        })?;
    if manifest.corpus_cid != corpus_cid {
        return Err(RunError::Command(format!(
            "corpus CID mismatch: manifest {} != actual {corpus_cid}",
            manifest.corpus_cid
        )));
    }

    let mut documents = Vec::new();
    let mut unique_ids = BTreeSet::new();
    let jsonl_line_count = corpus_bytes.split(|byte| *byte == b'\n').count();
    for (line_index, line) in corpus_bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.is_empty() && line_index + 1 == jsonl_line_count {
            continue;
        }
        if line.is_empty() {
            return Err(RunError::Command(format!(
                "empty JSONL row at line {}",
                line_index + 1
            )));
        }
        let article: SourceFreeCorpusArticle = serde_json::from_slice(line).map_err(|error| {
            RunError::Command(format!(
                "parse source-free corpus line {}: {error}",
                line_index + 1
            ))
        })?;
        if !unique_ids.insert(article.id.clone()) {
            return Err(RunError::Command(format!(
                "duplicate source-free corpus article id {}",
                article.id
            )));
        }
        documents.push(SourceDocument::new(article.id, article.text.into_bytes()));
    }
    if documents.len() != manifest.article_count {
        return Err(RunError::Command(format!(
            "manifest article_count {} != parsed document count {}",
            manifest.article_count,
            documents.len()
        )));
    }
    documents.sort_by(|left, right| left.id.cmp(&right.id));
    let (held_out, construction): (Vec<_>, Vec<_>) = documents
        .into_iter()
        .partition(|document| d3_is_held_out(&document.id));

    let table = SourceFreeTable::compile(&construction)
        .map_err(|error| RunError::Command(format!("compile source-free table: {error}")))?;
    if args.geometric_intervention {
        return source_free_table_geometric_intervention(
            args,
            corpus_cid,
            &manifest_bytes,
            manifest.article_count,
            construction.len(),
            &held_out,
            unique_ids.len(),
            table,
        );
    }
    let evaluation = table
        .evaluate_held_out(&held_out)
        .map_err(|error| RunError::Command(format!("evaluate source-free table: {error}")))?;
    let continuation = table
        .continue_text(args.prompt.as_bytes(), args.continuation_cap)
        .map_err(|error| RunError::Command(format!("continue source-free table: {error}")))?;
    let continuation_utf8 = String::from_utf8(continuation.decoded.clone()).map_err(|error| {
        RunError::Command(format!("source-free continuation is not UTF-8: {error}"))
    })?;
    let artifact_bytes = table.to_bytes();
    let artifact_cid = format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex());
    if let Some(path) = &args.artifact_out {
        std::fs::write(path, &artifact_bytes).map_err(|error| {
            RunError::Command(format!(
                "write source-free table artifact {}: {error}",
                path.display()
            ))
        })?;
    }

    let no_short_cycle = !matches!(
        continuation.stop,
        ContinuationStop::PeriodOneCycle | ContinuationStop::PeriodTwoCycle
    );
    let context_choices = evaluation
        .trigram_choices
        .checked_add(evaluation.bigram_choices)
        .ok_or_else(|| RunError::Command("context choice count overflow".to_owned()))?;
    let five_point_uplift = u128::from(evaluation.table_correct) * 20
        >= u128::from(evaluation.unigram_correct) * 20
            + u128::from(evaluation.known_target_positions);
    let criteria = SourceFreeCriteria {
        pinned_three_thousand_documents: manifest.article_count == 3_000
            && construction.len() + held_out.len() == 3_000,
        at_least_one_hundred_thousand_known_targets: evaluation.known_target_positions >= 100_000,
        nonzero_context_coverage: context_choices > 0,
        at_least_five_percentage_point_uplift: five_point_uplift,
        continuation_at_least_four_units: continuation.tokens.len() >= 4,
        continuation_has_no_period_one_or_two_cycle: no_short_cycle,
        continuation_is_utf8: true,
        external_byte_identical_replay_required: true,
    };
    let numeric_gate_met = criteria.pinned_three_thousand_documents
        && criteria.at_least_one_hundred_thousand_known_targets
        && criteria.nonzero_context_coverage
        && criteria.at_least_five_percentage_point_uplift
        && criteria.continuation_at_least_four_units
        && criteria.continuation_has_no_period_one_or_two_cycle;
    let report = SourceFreeTableReport {
        schema: SOURCE_FREE_TABLE_REPORT_SCHEMA,
        decision: if numeric_gate_met {
            SOURCE_FREE_TABLE_POSITIVE
        } else {
            SOURCE_FREE_TABLE_NEGATIVE
        },
        capability: "construction-trained source-free lexical table prediction and bounded decoding",
        nonclaims: "does not establish semantics, attention, geometry, correctness, reasoning, chat quality, multiplication-free serving, performance superiority, or release readiness",
        corpus_cid,
        manifest_cid: format!("blake3:{}", blake3::hash(&manifest_bytes).to_hex()),
        document_count: construction.len() + held_out.len(),
        construction_documents: construction.len(),
        held_out_documents: held_out.len(),
        unique_document_ids: unique_ids.len(),
        lexical_routes: table.lexical_piece_count(),
        artifact_bytes: artifact_bytes.len(),
        artifact_cid,
        held_out_encoded_positions: evaluation.positions,
        held_out_known_target_positions: evaluation.known_target_positions,
        table_correct: evaluation.table_correct,
        unigram_correct: evaluation.unigram_correct,
        table_top1_percent: fixed_percent(
            evaluation.table_correct,
            evaluation.known_target_positions,
        ),
        unigram_top1_percent: fixed_percent(
            evaluation.unigram_correct,
            evaluation.known_target_positions,
        ),
        uplift_percentage_points: fixed_percentage_point_difference(
            evaluation.table_correct,
            evaluation.unigram_correct,
            evaluation.known_target_positions,
        ),
        trigram_choices: evaluation.trigram_choices,
        bigram_choices: evaluation.bigram_choices,
        unigram_choices: evaluation.unigram_choices,
        changed_choices: evaluation.changed_choices,
        changed_choice_correct: evaluation.changed_choice_correct,
        continuation: SourceFreeContinuationReport {
            prompt: args.prompt.clone(),
            continuation_cap: args.continuation_cap,
            emitted_units: continuation.tokens.len(),
            decoded_text: continuation_utf8,
            stop: source_free_continuation_stop(continuation.stop),
        },
        source_closure: SourceFreeClosureCounters {
            teacher_calls: 0,
            provider_calls: 0,
            source_weight_reads: 0,
            geometry_operations: 0,
        },
        criteria,
    };
    let report_payload = serde_json::to_vec(&report)
        .map_err(|error| RunError::Command(format!("serialize source-free report: {error}")))?;
    let envelope = SourceFreeTableEnvelope {
        report_payload_cid: format!("blake3:{}", blake3::hash(&report_payload).to_hex()),
        report,
    };

    if args.json {
        let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
            RunError::Command(format!("serialize source-free report envelope: {error}"))
        })?;
        bytes.push(b'\n');
        io::stdout().lock().write_all(&bytes)?;
    } else {
        println!("source-free lexical table baseline");
        println!("  decision: {}", envelope.report.decision);
        println!(
            "  held-out top-1: {}% table vs {}% unigram ({} pp)",
            envelope.report.table_top1_percent,
            envelope.report.unigram_top1_percent,
            envelope.report.uplift_percentage_points
        );
        println!(
            "  continuation: {}",
            envelope.report.continuation.decoded_text
        );
        println!("  artifact: {}", envelope.report.artifact_cid);
        println!("  report: {}", envelope.report_payload_cid);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn source_free_table_geometric_intervention(
    args: &SourceFreeTableArgs,
    corpus_cid: String,
    manifest_bytes: &[u8],
    document_count: usize,
    construction_documents: usize,
    held_out: &[SourceDocument],
    unique_document_ids: usize,
    compiled_table: SourceFreeTable,
) -> Result<(), RunError> {
    if args.artifact_out.as_ref().is_some_and(|base| {
        args.geometry_overlay_out
            .as_ref()
            .is_some_and(|overlay| overlay == base)
    }) {
        return Err(RunError::Command(
            "source-free table and geometric overlay outputs must be different files".to_owned(),
        ));
    }

    let base_artifact_bytes = compiled_table.to_bytes();
    let base_artifact_cid = format!("blake3:{}", blake3::hash(&base_artifact_bytes).to_hex());
    let table = SourceFreeTable::from_bytes(&base_artifact_bytes)
        .map_err(|error| RunError::Command(format!("reload source-free table: {error}")))?;
    if table.artifact_cid() != base_artifact_cid {
        return Err(RunError::Command(
            "reloaded source-free table CID does not reproduce".to_owned(),
        ));
    }

    let compiled_overlay = MultiscaleCountRadiusR4V1::compile(&table).map_err(|error| {
        RunError::Command(format!(
            "compile {SOURCE_FREE_GEOMETRIC_MECHANISM} overlay: {error}"
        ))
    })?;
    let overlay_artifact_bytes = compiled_overlay.to_bytes();
    let overlay_artifact_cid = format!("blake3:{}", blake3::hash(&overlay_artifact_bytes).to_hex());
    let overlay = MultiscaleCountRadiusR4V1::from_bytes(&table, &overlay_artifact_bytes).map_err(
        |error| {
            RunError::Command(format!(
                "reload {SOURCE_FREE_GEOMETRIC_MECHANISM} overlay: {error}"
            ))
        },
    )?;
    let overlay_round_trip_byte_identical = overlay.to_bytes() == overlay_artifact_bytes
        && overlay.artifact_cid() == overlay_artifact_cid;
    let overlay_table_artifact_cid = overlay.table_artifact_cid();
    let overlay_table_binding_matches_base = overlay_table_artifact_cid == base_artifact_cid;
    let overlay_stats = overlay.stats();

    if let Some(path) = &args.artifact_out {
        std::fs::write(path, &base_artifact_bytes).map_err(|error| {
            RunError::Command(format!(
                "write source-free table artifact {}: {error}",
                path.display()
            ))
        })?;
    }
    if let Some(path) = &args.geometry_overlay_out {
        std::fs::write(path, &overlay_artifact_bytes).map_err(|error| {
            RunError::Command(format!(
                "write {SOURCE_FREE_GEOMETRIC_MECHANISM} overlay {}: {error}",
                path.display()
            ))
        })?;
    }

    let evaluation = table
        .evaluate_held_out_multiscale_count_radius(&overlay, held_out)
        .map_err(|error| {
            RunError::Command(format!(
                "evaluate {SOURCE_FREE_GEOMETRIC_MECHANISM} intervention: {error}"
            ))
        })?;
    let continuation = table
        .continue_text_multiscale_count_radius(
            &overlay,
            args.prompt.as_bytes(),
            args.continuation_cap,
        )
        .map_err(|error| {
            RunError::Command(format!(
                "continue {SOURCE_FREE_GEOMETRIC_MECHANISM} intervention: {error}"
            ))
        })?;
    let unchanged_continuation = table
        .continue_text(args.prompt.as_bytes(), args.continuation_cap)
        .map_err(|error| RunError::Command(format!("continue source-free table: {error}")))?;

    let baseline_continuation_matches_unchanged_table =
        continuation.baseline == unchanged_continuation;
    let baseline_continuation_utf8 = String::from_utf8(continuation.baseline.decoded.clone())
        .map_err(|error| {
            RunError::Command(format!(
                "source-free baseline continuation is not UTF-8: {error}"
            ))
        })?;
    let geometric_continuation_utf8 = String::from_utf8(continuation.geometric.decoded.clone())
        .map_err(|error| {
            RunError::Command(format!(
                "source-free geometric continuation is not UTF-8: {error}"
            ))
        })?;
    let first_divergence = continuation.first_divergence.as_ref();
    let source_closure = SourceFreeGeometricClosureCounters {
        teacher_calls: evaluation.teacher_calls,
        provider_calls: evaluation.provider_calls,
        source_weight_reads: evaluation.source_weight_reads,
        held_out_fit_reads: 0,
    };
    let source_closure_counters_zero = source_closure.teacher_calls == 0
        && source_closure.provider_calls == 0
        && source_closure.source_weight_reads == 0
        && source_closure.held_out_fit_reads == 0;
    let geometric_no_short_cycle = !matches!(
        continuation.geometric.stop,
        ContinuationStop::PeriodOneCycle | ContinuationStop::PeriodTwoCycle
    );
    let criteria = SourceFreeGeometricCriteria {
        frozen_corpus_cid: corpus_cid == SOURCE_FREE_FROZEN_CORPUS_CID,
        frozen_document_partition: document_count == SOURCE_FREE_FROZEN_DOCUMENTS
            && construction_documents == SOURCE_FREE_FROZEN_CONSTRUCTION_DOCUMENTS
            && held_out.len() == SOURCE_FREE_FROZEN_HELD_OUT_DOCUMENTS,
        frozen_known_target_positions: evaluation.known_target_positions
            == SOURCE_FREE_FROZEN_KNOWN_TARGETS,
        frozen_baseline_correct: evaluation.baseline_correct == SOURCE_FREE_FROZEN_BASELINE_CORRECT,
        frozen_table_artifact_cid: base_artifact_cid == SOURCE_FREE_FROZEN_ARTIFACT_CID,
        frozen_prompt_and_cap: args.prompt == SOURCE_FREE_FROZEN_PROMPT
            && args.continuation_cap == SOURCE_FREE_FROZEN_CONTINUATION_CAP,
        overlay_round_trip_byte_identical,
        overlay_table_binding_matches_base,
        baseline_continuation_matches_unchanged_table,
        geometric_correct_strictly_greater_than_frozen_baseline: evaluation.geometric_correct
            > SOURCE_FREE_FROZEN_BASELINE_CORRECT,
        changed_choice_geometric_advantage: evaluation.geometric_changed_correct
            > evaluation.baseline_changed_correct,
        nonzero_reachable_ties: evaluation.reachable_tie_positions > 0,
        teacher_forced_support_identical: evaluation.support_mismatches == 0,
        teacher_forced_work_identical: evaluation.work_mismatches == 0,
        fixed_prompt_first_divergence: first_divergence.is_some(),
        first_divergence_within_continuation_cap: first_divergence
            .is_some_and(|witness| witness.unit_index < args.continuation_cap),
        pre_divergence_frame_identical: first_divergence
            .is_some_and(|witness| witness.support_matched),
        pre_divergence_work_identical: first_divergence.is_some_and(|witness| witness.work_matched),
        geometric_output_distinct: continuation.geometric.decoded != continuation.baseline.decoded,
        geometric_continuation_at_least_four_units: continuation.geometric.tokens.len() >= 4,
        geometric_continuation_has_no_period_one_or_two_cycle: geometric_no_short_cycle,
        geometric_continuation_is_utf8: true,
        source_closure_counters_zero,
        external_byte_identical_replay_required: true,
    };
    let decision = if criteria.passes_before_replay() {
        SOURCE_FREE_GEOMETRIC_PENDING_REPLAY
    } else {
        SOURCE_FREE_GEOMETRIC_NEGATIVE
    };

    let report = SourceFreeGeometricReport {
        schema: SOURCE_FREE_GEOMETRIC_REPORT_SCHEMA,
        decision,
        mechanism: SOURCE_FREE_GEOMETRIC_MECHANISM,
        capability: "construction-only fixed-point R4 radius tie intervention over the unchanged source-free lexical table",
        nonclaims: "does not establish attention, semantics, correctness, reasoning, chat quality, performance superiority, formal closure, or release readiness",
        corpus_cid,
        manifest_cid: format!("blake3:{}", blake3::hash(manifest_bytes).to_hex()),
        document_count,
        construction_documents,
        held_out_documents: held_out.len(),
        unique_document_ids,
        lexical_routes: table.lexical_piece_count(),
        base_artifact: SourceFreeGeometricArtifactReport {
            bytes: base_artifact_bytes.len(),
            cid: base_artifact_cid,
        },
        overlay_artifact: SourceFreeGeometricArtifactReport {
            bytes: overlay_artifact_bytes.len(),
            cid: overlay_artifact_cid,
        },
        overlay_table_artifact_cid,
        overlay_stats: SourceFreeGeometricOverlayStatsReport {
            eligible_bigram_rows: overlay_stats.eligible_bigram_rows,
            eligible_trigram_rows: overlay_stats.eligible_trigram_rows,
            geometry_changed_bigram_rows: overlay_stats.geometry_changed_bigram_rows,
            geometry_changed_trigram_rows: overlay_stats.geometry_changed_trigram_rows,
            geometry_changed_rows: overlay_stats.geometry_changed_rows,
        },
        held_out_encoded_positions: evaluation.positions,
        held_out_known_target_positions: evaluation.known_target_positions,
        baseline_correct: evaluation.baseline_correct,
        geometric_correct: evaluation.geometric_correct,
        baseline_top1_percent: fixed_percent(
            evaluation.baseline_correct,
            evaluation.known_target_positions,
        ),
        geometric_top1_percent: fixed_percent(
            evaluation.geometric_correct,
            evaluation.known_target_positions,
        ),
        geometric_delta_percentage_points: fixed_percentage_point_difference(
            evaluation.geometric_correct,
            evaluation.baseline_correct,
            evaluation.known_target_positions,
        ),
        reachable_tie_positions: evaluation.reachable_tie_positions,
        changed_choices: evaluation.changed_choices,
        geometric_changed_correct: evaluation.geometric_changed_correct,
        baseline_changed_correct: evaluation.baseline_changed_correct,
        support_mismatches: evaluation.support_mismatches,
        work_mismatches: evaluation.work_mismatches,
        trigram_choices: evaluation.trigram_choices,
        bigram_choices: evaluation.bigram_choices,
        unigram_choices: evaluation.unigram_choices,
        baseline_continuation: SourceFreeGeometricContinuationReport {
            prompt: args.prompt.clone(),
            continuation_cap: args.continuation_cap,
            emitted_units: continuation.baseline.tokens.len(),
            tokens: continuation.baseline.tokens.clone(),
            decoded_text: baseline_continuation_utf8,
            stop: source_free_continuation_stop(continuation.baseline.stop),
        },
        geometric_continuation: SourceFreeGeometricContinuationReport {
            prompt: args.prompt.clone(),
            continuation_cap: args.continuation_cap,
            emitted_units: continuation.geometric.tokens.len(),
            tokens: continuation.geometric.tokens.clone(),
            decoded_text: geometric_continuation_utf8,
            stop: source_free_continuation_stop(continuation.geometric.stop),
        },
        first_divergence: continuation.first_divergence,
        source_closure,
        criteria,
    };
    let report_payload = serde_json::to_vec(&report).map_err(|error| {
        RunError::Command(format!(
            "serialize {SOURCE_FREE_GEOMETRIC_MECHANISM} report: {error}"
        ))
    })?;
    let envelope = SourceFreeGeometricEnvelope {
        report_payload_cid: format!("blake3:{}", blake3::hash(&report_payload).to_hex()),
        report,
    };

    if args.json {
        let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
            RunError::Command(format!(
                "serialize {SOURCE_FREE_GEOMETRIC_MECHANISM} report envelope: {error}"
            ))
        })?;
        bytes.push(b'\n');
        io::stdout().lock().write_all(&bytes)?;
    } else {
        println!("source-free geometric tie intervention");
        println!("  decision: {}", envelope.report.decision);
        println!(
            "  held-out top-1: {}% geometric vs {}% baseline ({} pp)",
            envelope.report.geometric_top1_percent,
            envelope.report.baseline_top1_percent,
            envelope.report.geometric_delta_percentage_points
        );
        println!(
            "  baseline continuation: {}",
            envelope.report.baseline_continuation.decoded_text
        );
        println!(
            "  geometric continuation: {}",
            envelope.report.geometric_continuation.decoded_text
        );
        println!("  base artifact: {}", envelope.report.base_artifact.cid);
        println!(
            "  geometric overlay: {}",
            envelope.report.overlay_artifact.cid
        );
        println!("  report: {}", envelope.report_payload_cid);
    }
    Ok(())
}

fn fixed_percent(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "0.000000".to_owned();
    }
    let scaled = u128::from(numerator) * 100_000_000 / u128::from(denominator);
    format!("{}.{:06}", scaled / 1_000_000, scaled % 1_000_000)
}

fn fixed_percentage_point_difference(table: u64, unigram: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "0.000000".to_owned();
    }
    let difference = i128::from(table) - i128::from(unigram);
    let scaled = difference * 100_000_000 / i128::from(denominator);
    let sign = if scaled < 0 { "-" } else { "" };
    let magnitude = scaled.unsigned_abs();
    format!(
        "{sign}{}.{:06}",
        magnitude / 1_000_000,
        magnitude % 1_000_000
    )
}

fn source_free_continuation_stop(stop: ContinuationStop) -> &'static str {
    match stop {
        ContinuationStop::EndOfDocument => "end_of_document",
        ContinuationStop::PeriodOneCycle => "period_one_cycle",
        ContinuationStop::PeriodTwoCycle => "period_two_cycle",
        ContinuationStop::Bound => "continuation_cap",
    }
}

fn local_generation_stop_label(reason: LocalGenerationStopReason) -> String {
    match reason {
        LocalGenerationStopReason::Abstained { tie } => format!("abstained(tie={tie})"),
        LocalGenerationStopReason::TerminalPunctuation => "terminal_punctuation".to_owned(),
        LocalGenerationStopReason::ShortCycle { period } => {
            format!("short_cycle(period={period})")
        }
        LocalGenerationStopReason::ContinuationCap => "continuation_cap".to_owned(),
    }
}

fn print_research_tools() {
    println!("Active capability-first command:");
    println!("  source-free-table");
    println!();
    println!("Preserved research commands (not required for the table baseline):");
    println!("  bounded-geometric-generate");
    println!("  ask, chat, client, audit");
    println!("  compile, download, import, install-release, package-release-bundle");
    println!("  transformerless, graph, graph-compile, graph-observe");
    println!("  geometric-decoder-spike, geometric-mixer-qualification");
    println!("  deployed-quality, certify, compare, compare-report, scenarios");
    println!();
    println!("Run `r4 <command> --help` for an individual preserved command.");
    println!("These paths are retained research infrastructure, not the active route-native intelligence sequence.");
}

/// Default location of the reference teacher checkpoint used by `certify`/`compare`.
const DEFAULT_REFERENCE_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const DEFAULT_R4_SOFTMAX_LOCAL_MODEL: &str = ".uor-models/research/issue-1017/export";
const DEFAULT_GROUNDED_ANSWER_MODEL: &str = ".uor-models/research/issue-1017/export";
const DEFAULT_GROUNDED_ANSWER_HEAD: &str =
    ".uor-models/research/issue-954/source-span-pointer/source-span-pointer.json";

fn resolve_r4_softmax_local_model(configured: &Path) -> Result<PathBuf, RunError> {
    let model = if configured == Path::new(DEFAULT_R4_SOFTMAX_LOCAL_MODEL) {
        model_store_root().join("research/issue-1017/export")
    } else {
        configured.to_path_buf()
    };
    if !model.join("model.safetensors").is_file() {
        return Err(RunError::Command(format!(
            "no local #1017 model found at {}; set UOR_MODEL_STORE or pass --model",
            model.display()
        )));
    }
    Ok(model)
}

fn resolve_grounded_answer_model(configured: &Path) -> Result<PathBuf, RunError> {
    let model = if configured == Path::new(DEFAULT_GROUNDED_ANSWER_MODEL) {
        model_store_root().join("research/issue-1017/export")
    } else {
        configured.to_path_buf()
    };
    if !model.join("model.safetensors").is_file() {
        return Err(RunError::Command(format!(
            "no frozen #1017 model found at {}; set UOR_MODEL_STORE or pass --model",
            model.display()
        )));
    }
    Ok(model)
}

fn resolve_grounded_answer_head(configured: &Path) -> Result<PathBuf, RunError> {
    let head = if configured == Path::new(DEFAULT_GROUNDED_ANSWER_HEAD) {
        model_store_root().join("research/issue-954/source-span-pointer/source-span-pointer.json")
    } else {
        configured.to_path_buf()
    };
    if !head.is_file() {
        return Err(RunError::Command(format!(
            "no #954 source-span pointer artifact found at {}; the bounded run emitted no qualified final head, so pass --head only for an explicit research artifact",
            head.display()
        )));
    }
    Ok(head)
}

/// Resolve the reference teacher checkpoint path, honoring the `TLESS_CHECKPOINT`
/// override, and confirm it exists before handing it to `LlamaOracle::load`
/// (which otherwise panics on a missing file).
fn reference_checkpoint_path() -> Result<String, RunError> {
    let path = std::env::var("TLESS_CHECKPOINT")
        .unwrap_or_else(|_| DEFAULT_REFERENCE_CHECKPOINT.to_owned());
    if !std::path::Path::new(&path).exists() {
        return Err(RunError::Command(format!(
            "reference checkpoint not found at {path} (set TLESS_CHECKPOINT to override)"
        )));
    }
    Ok(path)
}

fn run(cli: &Cli) -> Result<(), RunError> {
    cli.configure_tless();
    match cli.command.as_ref() {
        Some(Command::Demo) | None => {
            let mut config = cli.server_config();
            config.geometric_demo = true;
            println!("UOR-R4 geometric router research demo");
            println!("Open http://{}:{} in a browser.", config.host, config.port);
            println!("This demonstrates routing, retrieval, and geometry telemetry; it is not yet the source-free intelligence engine.");
            server::run_server(Arc::new(config));
            Ok(())
        }
        Some(Command::Route(args)) => route_demo(args),
        Some(Command::BoundedGeometricGenerate(args)) => bounded_geometric_generate(args),
        Some(Command::SourceFreeTable(args)) => source_free_table(args),
        Some(Command::R4SoftmaxGenerate(args)) => {
            let workers = std::num::NonZeroUsize::new(args.workers).ok_or_else(|| {
                RunError::Command("--workers must be greater than zero".to_owned())
            })?;
            let report = uor_r4_wasm_router::r4_softmax_reference_generation::run_r4_softmax_reference_generation(
                &uor_r4_wasm_router::r4_softmax_reference_generation::R4SoftmaxReferenceGeneratorConfig {
                    source: args.source.clone(),
                    source_revision: args.source_revision.clone(),
                    prompt: args.prompt.clone(),
                    max_new_tokens: args.max_tokens,
                    workers,
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            if let Some(path) = &args.json_output {
                uor_r4_wasm_router::r4_softmax_reference_generation::write_json_report(
                    path, &report,
                )
                .map_err(|error| RunError::Command(error.to_string()))?;
            }
            println!("{}", report.transcript.response_text);
            Ok(())
        }
        Some(Command::R4SoftmaxLocalGenerate(args)) => {
            let model = resolve_r4_softmax_local_model(&args.model)?;
            let workers = std::num::NonZeroUsize::new(
                uor_r4_wasm_router::r4_softmax_local_generation::DEFAULT_WORKERS,
            )
            .ok_or_else(|| RunError::Command("local generator worker count is zero".to_owned()))?;
            let report = uor_r4_wasm_router::r4_softmax_local_generation::run_r4_softmax_local_generation(
                &uor_r4_wasm_router::r4_softmax_local_generation::R4SoftmaxLocalGeneratorConfig {
                    model,
                    prompt: args.prompt.clone(),
                    max_new_tokens: args.max_new_tokens,
                    workers,
                    attention_off: args.attention_off,
                    seed: args.seed,
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            if let Some(path) = &args.json_output {
                uor_r4_wasm_router::r4_softmax_local_generation::write_json_report(path, &report)
                    .map_err(|error| RunError::Command(error.to_string()))?;
            }
            println!("{}", report.transcript.response_text);
            Ok(())
        }
        Some(Command::Answer(args)) => {
            let model = resolve_grounded_answer_model(&args.model)?;
            let head = resolve_grounded_answer_head(&args.head)?;
            if let Some(path) = &args.json_output {
                uor_r4_wasm_router::r4_grounded_answer::require_distinct_output_path(
                    &args.source_file,
                    &head,
                    &model,
                    path,
                )
                .map_err(|error| RunError::Command(error.to_string()))?;
            }
            let workers = std::num::NonZeroUsize::new(
                uor_r4_wasm_router::r4_softmax_local_generation::DEFAULT_WORKERS,
            )
            .ok_or_else(|| RunError::Command("local generator worker count is zero".to_owned()))?;
            let report = uor_r4_wasm_router::r4_grounded_answer::run_grounded_answer(
                &uor_r4_wasm_router::r4_grounded_answer::GroundedAnswerConfig {
                    model,
                    head,
                    source_file: args.source_file.clone(),
                    question: args.question.clone(),
                    workers,
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            if let Some(path) = &args.json_output {
                uor_r4_wasm_router::r4_grounded_answer::write_json_report(path, &report)
                    .map_err(|error| RunError::Command(error.to_string()))?;
            }
            match &report.outcome {
                uor_r4_wasm_router::r4_grounded_answer::GroundedAnswerOutcome::Answered {
                    answer,
                    ..
                } => println!("{answer}"),
                uor_r4_wasm_router::r4_grounded_answer::GroundedAnswerOutcome::Contradiction => {
                    println!("CONTRADICTION")
                }
                uor_r4_wasm_router::r4_grounded_answer::GroundedAnswerOutcome::Abstained {
                    reason,
                } => println!("ABSTAIN[{}]", reason.as_str()),
            }
            Ok(())
        }
        Some(Command::R4SoftmaxLocalQualify(args)) => {
            let workers = std::num::NonZeroUsize::new(args.workers).ok_or_else(|| {
                RunError::Command("--workers must be greater than zero".to_owned())
            })?;
            let campaign = if args.enabled_only {
                uor_r4_wasm_router::r4_softmax_local_generation::R4SoftmaxLocalQualificationCampaign::Issue1017EnabledOnly
            } else {
                args.campaign
                    .unwrap_or(R4SoftmaxLocalQualificationCampaignArg::Issue1014)
                    .campaign()
            };
            let report = uor_r4_wasm_router::r4_softmax_local_generation::run_r4_softmax_local_qualification(
                &uor_r4_wasm_router::r4_softmax_local_generation::R4SoftmaxLocalQualificationConfig {
                    model: args.model.clone(),
                    python_prefix_logits: args.python_prefix_logits.clone(),
                    reveal_manifest: args.reveal_manifest.clone(),
                    workers,
                    campaign,
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            uor_r4_wasm_router::r4_softmax_local_generation::write_qualification_json_report(
                &args.json_output,
                &report,
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            let attention_off = report
                .attention_off_prefix_parity
                .as_ref()
                .map(|parity| parity.maximum_absolute_logit_delta.to_string())
                .unwrap_or_else(|| "NOT_RUN".to_owned());
            println!(
                "qualification_passed={} enabled_max_abs={} attention_off_max_abs={} report={}",
                report.qualification_passed,
                report.enabled_prefix_parity.maximum_absolute_logit_delta,
                attention_off,
                args.json_output.display()
            );
            Ok(())
        }
        Some(Command::R4SoftmaxTraceStudent(args)) => {
            let workers = std::num::NonZeroUsize::new(args.workers).ok_or_else(|| {
                RunError::Command("--workers must be greater than zero".to_owned())
            })?;
            let config =
                uor_r4_wasm_router::r4_softmax_trace_experiment::R4SoftmaxTraceExperimentConfig {
                    source: args.source.clone(),
                    source_revision: args.source_revision.clone(),
                    implementation_revision: args.implementation_revision.clone(),
                    workers,
                    artifact_output: args.artifact_output.clone(),
                    trace_output: args.trace_output.clone(),
                    freeze_output: args.freeze_output.clone(),
                    result_output: args.result_output.clone(),
                };
            if args.preflight_only {
                let report = uor_r4_wasm_router::r4_softmax_trace_experiment::preflight(&config)
                    .map_err(|error| RunError::Command(error.to_string()))?;
                let rendered = serde_json::to_string_pretty(&report).map_err(|error| {
                    RunError::Command(format!("serialize R4 softmax trace preflight: {error}"))
                })?;
                println!("{rendered}");
                Ok(())
            } else if args.reveal {
                let result =
                    uor_r4_wasm_router::r4_softmax_trace_experiment::reveal_held_out(&config)
                        .map_err(|error| RunError::Command(error.to_string()))?;
                let rendered = serde_json::to_string_pretty(&result).map_err(|error| {
                    RunError::Command(format!("serialize R4 softmax trace result: {error}"))
                })?;
                println!("{rendered}");
                Ok(())
            } else {
                let freeze =
                    uor_r4_wasm_router::r4_softmax_trace_experiment::compile_construction(&config)
                        .map_err(|error| RunError::Command(error.to_string()))?;
                let rendered = serde_json::to_string_pretty(&freeze).map_err(|error| {
                    RunError::Command(format!("serialize R4 softmax trace freeze: {error}"))
                })?;
                println!("{rendered}");
                Ok(())
            }
        }
        Some(Command::R4SoftmaxTraceStateCompile(args)) => {
            let freeze =
                uor_r4_wasm_router::r4_softmax_trace_state_experiment::compile_construction(
                    &uor_r4_wasm_router::r4_softmax_trace_state_experiment::R4SoftmaxTraceStateCompileConfig {
                        implementation_revision: args.implementation_revision.clone(),
                        trace_bundle: args.trace_bundle.clone(),
                        predecessor_freeze: args.predecessor_freeze.clone(),
                        suffix_artifact: args.suffix_artifact.clone(),
                        artifact_output: args.artifact_output.clone(),
                        freeze_output: args.freeze_output.clone(),
                    },
                )
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&freeze).map_err(|error| {
                RunError::Command(format!("serialize R4 softmax trace state freeze: {error}"))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::R4SoftmaxTraceStateSeal(args)) => {
            let seal = uor_r4_wasm_router::r4_softmax_trace_state_experiment::seal_source_free(
                &uor_r4_wasm_router::r4_softmax_trace_state_experiment::R4SoftmaxTraceStateSealConfig {
                    artifact: args.artifact.clone(),
                    freeze: args.freeze.clone(),
                    causal_input: args.causal_input.clone(),
                    seal_output: args.seal_output.clone(),
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&seal).map_err(|error| {
                RunError::Command(format!("serialize R4 softmax trace state seal: {error}"))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::R4SoftmaxTraceStateReveal(args)) => {
            let result = uor_r4_wasm_router::r4_softmax_trace_state_experiment::reveal(
                &uor_r4_wasm_router::r4_softmax_trace_state_experiment::R4SoftmaxTraceStateRevealConfig {
                    seal: args.seal.clone(),
                    judge: args.judge.clone(),
                    result_output: args.result_output.clone(),
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&result).map_err(|error| {
                RunError::Command(format!("serialize R4 softmax trace state result: {error}"))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::R4SoftmaxTraceObservability(args)) => {
            let result =
                uor_r4_wasm_router::r4_softmax_trace_observability_experiment::run_observability(
                    &uor_r4_wasm_router::r4_softmax_trace_observability_experiment::R4SoftmaxTraceObservabilityConfig {
                        implementation_revision: args.implementation_revision.clone(),
                        trace_bundle: args.trace_bundle.clone(),
                        predecessor_freeze: args.predecessor_freeze.clone(),
                        result_output: args.result_output.clone(),
                    },
                )
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&result).map_err(|error| {
                RunError::Command(format!(
                    "serialize R4 softmax trace observability result: {error}"
                ))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::LexicalIngestionWitness) => {
            let witness = uor_r4_core::canonical_lexical_ingestion::run_authorized_probe()
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&witness).map_err(|error| {
                RunError::Command(format!("serialize lexical-ingestion witness: {error}"))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::RecursiveAttentionA1Probe) => {
            let report = uor_r4_core::recursive_geometric_attention::run_a1_0_ordered_state_probe()
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&report).map_err(|error| {
                RunError::Command(format!(
                    "serialize recursive-attention A1.0 report: {error}"
                ))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::AssociativeOrderedSummaryA1rProbe) => {
            let report = uor_r4_core::recursive_geometric_attention::run_a1r_associative_ordered_summary_probe()
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&report).map_err(|error| {
                RunError::Command(format!(
                    "serialize associative ordered-summary A1R report: {error}"
                ))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::CandidateRelativeIdentifiabilityA1pProbe) => {
            let report = uor_r4_core::recursive_geometric_attention::run_a1p_candidate_relative_identifiability_probe()
                .map_err(|error| RunError::Command(error.to_string()))?;
            let rendered = serde_json::to_string_pretty(&report).map_err(|error| {
                RunError::Command(format!(
                    "serialize candidate-relative A1P identifiability report: {error}"
                ))
            })?;
            println!("{rendered}");
            Ok(())
        }
        Some(Command::ResearchTools) => {
            print_research_tools();
            Ok(())
        }
        Some(Command::Ask(args)) => {
            let mut chat = build_chat_engine(
                args.model.as_deref(),
                args.sample,
                args.greedy,
                args.research,
            )?;
            answer_once(
                &mut chat,
                &args.question.join(" "),
                &mut io::stdout().lock(),
            )
        }
        Some(Command::Chat(args)) => {
            if let Some(remote) = &args.remote {
                uor_r4_wasm_router::chat::remote_interactive_chat(
                    remote,
                    args.model.as_deref().unwrap_or("r4"),
                    &mut io::stdin().lock(),
                    &mut io::stdout().lock(),
                )?;
                Ok(())
            } else {
                let mut chat = build_chat_engine(
                    args.model.as_deref(),
                    args.sample,
                    args.greedy,
                    args.research,
                )?;
                interactive_chat(&mut chat, &mut io::stdin().lock(), &mut io::stdout().lock())?;
                Ok(())
            }
        }
        Some(Command::Client(args)) => {
            uor_r4_wasm_router::chat::remote_interactive_chat(
                &args.remote,
                &args.model,
                &mut io::stdin().lock(),
                &mut io::stdout().lock(),
            )?;
            Ok(())
        }
        Some(Command::Compile(args)) => compile(args),
        Some(Command::Download(args)) => download(args),
        Some(Command::Import(args)) => import(args),
        Some(Command::PackageReleaseBundle(args)) => package_release_bundle_command(args),
        Some(Command::InstallRelease(args)) => install_release_command(args),
        Some(Command::EvaluateReport(args)) => evaluate_report(args),
        Some(Command::GeometricDecoderSpike(args)) => {
            let workers = std::num::NonZeroUsize::new(args.workers).ok_or_else(|| {
                RunError::Command("--workers must be greater than zero".to_owned())
            })?;
            let report = uor_r4_wasm_router::geometric_decoder::run_geometric_spike(
                &uor_r4_wasm_router::geometric_decoder::GeometricSpikeConfig {
                    source: args.source.clone(),
                    source_revision: args.source_revision.clone(),
                    output: args.output.clone(),
                    router_state_output: args.router_state_output.clone(),
                    identity: args.identity.clone(),
                    workers,
                    control_report: args.control_report.clone(),
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            println!(
                "#950 geometric decoder spike: {} (controls={}, treatments={}, changed_logits={}, report={})",
                report.gates.verdict,
                report.control.len(),
                report.treatment.len(),
                report.reachability.changed_logits,
                args.output.display()
            );
            Ok(())
        }
        Some(Command::GeometricMixerQualification(args)) => {
            if let Some(review) = &args.finalize_review {
                let report =
                    uor_r4_wasm_router::geometric_mixer_qualification::finalize_operator_review(
                        &args.output,
                        review,
                    )
                    .map_err(|error| RunError::Command(error.to_string()))?;
                println!(
                    "#951 geometric mixer qualification: {} (reviewed={}, report={})",
                    report.final_verdict.as_deref().unwrap_or("UNAVAILABLE"),
                    report
                        .operator_review
                        .as_ref()
                        .map_or(0, |review| review.reviews.len()),
                    args.output.display()
                );
                return Ok(());
            }
            if args.preflight_only {
                let report = uor_r4_wasm_router::geometric_mixer_qualification::run_preflight_only(
                    args.seed,
                    &args.preflight_report,
                    &args.preflight_checkpoint,
                )
                .map_err(|error| RunError::Command(error.to_string()))?;
                println!(
                    "#951 mixer preflight: {} (overfit_reduction={:.6}, gradient_error={:.8}, report={})",
                    report.verdict,
                    report.tiny_overfit.reduction_fraction,
                    report.gradient_check.absolute_error,
                    args.preflight_report.display()
                );
                return Ok(());
            }
            let source = args.source.clone().ok_or_else(|| {
                RunError::Command(
                    "--source is required for fitting (or use --preflight-only / --finalize-review)"
                        .to_owned(),
                )
            })?;
            if args.steps_per_round == 0
                || args.steps_per_round
                    > uor_r4_wasm_router::geometric_mixer_qualification::MAX_STEPS_PER_ROUND
            {
                return Err(RunError::Command(format!(
                    "--steps-per-round must be in 1..={}",
                    uor_r4_wasm_router::geometric_mixer_qualification::MAX_STEPS_PER_ROUND
                )));
            }
            let workers = std::num::NonZeroUsize::new(args.workers).ok_or_else(|| {
                RunError::Command("--workers must be greater than zero".to_owned())
            })?;
            let report = uor_r4_wasm_router::geometric_mixer_qualification::run_qualification(
                &uor_r4_wasm_router::geometric_mixer_qualification::QualificationConfig {
                    source,
                    source_revision: args.source_revision.clone(),
                    g0_report: args.g0_report.clone(),
                    preflight_report: args.preflight_report.clone(),
                    output: args.output.clone(),
                    checkpoint: args.checkpoint.clone(),
                    identity: args.identity.clone(),
                    workers,
                    seed: args.seed,
                    steps_per_round: args.steps_per_round,
                },
            )
            .map_err(|error| RunError::Command(error.to_string()))?;
            println!(
                "#951 geometric mixer qualification: {} (held_out_advantage={:.6}, rounds={}, report={})",
                report
                    .final_verdict
                    .as_deref()
                    .unwrap_or("PENDING_OPERATOR_REVIEW"),
                report.held_out.relative_real_advantage,
                report.training_rounds.len(),
                args.output.display()
            );
            Ok(())
        }
        Some(Command::Setup) => run_core("setup", &[]),
        Some(Command::Gen { seconds, target }) => {
            run_core("gen", &[seconds.to_string(), target.to_string()])
        }
        Some(Command::Store) => run_core("store", &[]),
        Some(Command::DeployedQuality(args)) => deployed_quality_command(args),
        Some(Command::Certify) => {
            if std::env::var("R4_CERTIFY_C_ONLY").is_ok_and(|value| value != "0") {
                println!(
                    "R4_CERTIFY_C_ONLY set: running only the C serving row; full certificate skipped."
                );
                certify_serving_row();
                return Ok(());
            }
            let checkpoint = reference_checkpoint_path()?;
            let oracle = uor_r4_model_source::LlamaOracle::load(&checkpoint);
            uor_r4_graph_certify::certify::certify(&oracle);
            if std::env::var("R4_CERTIFY_ROWS_ONLY").is_ok_and(|value| value != "0") {
                println!(
                    "R4_CERTIFY_ROWS_ONLY set: C serving row SKIPPED — rows-only run, not a full certificate. No measurement recorded."
                );
            } else {
                certify_serving_row();
            }
            Ok(())
        }
        Some(Command::Compare) => {
            let checkpoint = reference_checkpoint_path()?;
            let mut oracle = uor_r4_model_source::LlamaOracle::load(&checkpoint);
            uor_r4_graph_certify::compare::compare(&mut oracle);
            Ok(())
        }
        Some(Command::CompareReport) => {
            uor_r4_graph_certify::compare::report();
            Ok(())
        }
        Some(Command::Scenarios) => run_core("scenarios", &[]),
        Some(Command::TeacherKappa) => run_core("teacher-kappa", &[]),
        Some(Command::Transformerless { args }) => {
            transformerless_command::run(args).map_err(|error| RunError::Command(error.to_string()))
        }
        Some(Command::Graph { args }) => {
            let mut values = vec!["graph".to_owned()];
            values.extend_from_slice(args);
            transformerless_command::run(&values)
                .map_err(|error| RunError::Command(error.to_string()))
        }
        Some(Command::GraphCompile { args }) => uor_r4_graph_compiler::compile(args)
            .map_err(|error| RunError::Command(error.to_string())),
        Some(Command::GraphObserve { args }) => uor_r4_graph_compiler::observe(args)
            .map_err(|error| RunError::Command(error.to_string())),
        Some(Command::Audit(args)) => audit_command(&args.log_file),
        Some(Command::Serve(args)) => {
            let mut config = cli.server_config();
            if args.enable_r4_softmax_reference {
                let workers = resolve_r4_softmax_reference_workers(args.r4_softmax_workers)
                    .map_err(RunError::Command)?;
                config.r4_softmax_reference = Some(server::R4SoftmaxReferenceHttpConfig {
                    source: args.r4_softmax_source.clone().unwrap_or_else(|| {
                        PathBuf::from(".uor-models/sources/smollm2-135m-instruct")
                    }),
                    workers,
                });
            }
            server::validate_r4_softmax_reference_http_startup(
                &config.host,
                config.r4_softmax_reference.as_ref(),
            )
            .map_err(RunError::Command)?;
            server::run_server(Arc::new(config));
            Ok(())
        }
    }
}

const DEPLOYED_QUALITY_INVOCATION_TERMINAL_SCHEMA: &str =
    "uor-r4-deployed-quality-invocation-terminal/1";
const DEPLOYED_QUALITY_INVOCATION_TERMINAL_PATH: &str =
    "evidence/deployed_quality_invocation_terminal.jsonl";

struct DeployedQualityInvocationContext {
    bundle: String,
    compiler_revision: String,
    mode: &'static str,
    positions: usize,
    workers: Option<usize>,
    probe_budget_secs: u64,
    eval_budget_secs: u64,
    cross_surface_evidence: Option<String>,
}

struct DeployedQualityInvocationTerminal {
    file: std::fs::File,
    path: PathBuf,
    context: DeployedQualityInvocationContext,
    started: std::time::Instant,
    finished: bool,
}

impl DeployedQualityInvocationTerminal {
    fn begin(args: &DeployedQualityArgs) -> Result<Self, RunError> {
        let bundle_metadata = std::fs::symlink_metadata(&args.bundle).map_err(|error| {
            RunError::Command(format!(
                "deployed-quality bundle root {} is unavailable before invocation evidence can start: {error}",
                args.bundle.display()
            ))
        })?;
        if bundle_metadata.file_type().is_symlink() || !bundle_metadata.is_dir() {
            return Err(RunError::Command(format!(
                "deployed-quality bundle root {} must be a real directory, not a file or symlink",
                args.bundle.display()
            )));
        }
        let canonical_bundle = std::fs::canonicalize(&args.bundle).map_err(|error| {
            RunError::Command(format!(
                "canonicalize deployed-quality bundle root {}: {error}",
                args.bundle.display()
            ))
        })?;
        let evidence_dir = args.bundle.join("evidence");
        match std::fs::create_dir(&evidence_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(RunError::Command(format!(
                    "create non-semantic deployed-quality evidence directory {}: {error}",
                    evidence_dir.display()
                )));
            }
        }
        let evidence_metadata = std::fs::symlink_metadata(&evidence_dir).map_err(|error| {
            RunError::Command(format!(
                "inspect deployed-quality evidence directory {}: {error}",
                evidence_dir.display()
            ))
        })?;
        if evidence_metadata.file_type().is_symlink() || !evidence_metadata.is_dir() {
            return Err(RunError::Command(format!(
                "deployed-quality evidence path {} must be a real directory, not a file or symlink",
                evidence_dir.display()
            )));
        }

        let path = args.bundle.join(DEPLOYED_QUALITY_INVOCATION_TERMINAL_PATH);
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .map_err(|error| {
                RunError::Command(format!(
                    "create append-only deployed-quality invocation terminal {}: {error}; use a fresh immutable staging bundle for each invocation",
                    path.display()
                ))
            })?;
        let mut terminal = Self {
            file,
            path,
            context: DeployedQualityInvocationContext {
                bundle: canonical_bundle.display().to_string(),
                compiler_revision: args.compiler_revision.clone(),
                mode: args.mode.as_str(),
                positions: args.positions,
                workers: args.workers,
                probe_budget_secs: args.probe_budget_secs,
                eval_budget_secs: args.eval_budget_secs,
                cross_surface_evidence: args
                    .cross_surface_evidence
                    .as_ref()
                    .map(|path| path.display().to_string()),
            },
            started: std::time::Instant::now(),
            finished: false,
        };
        if let Err(error) = terminal.write_event(0, "started", None, None) {
            terminal.finished = true;
            return Err(RunError::Command(format!(
                "write deployed-quality invocation start terminal {}: {error}",
                terminal.path.display()
            )));
        }
        println!(
            "deployed-quality invocation terminal started: {}",
            terminal.path.display()
        );
        Ok(terminal)
    }

    fn finish(&mut self, outcome: &'static str, reason: Option<&str>) -> io::Result<()> {
        if self.finished {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "deployed-quality invocation terminal already finished",
            ));
        }
        // Set this before the write so an I/O error never causes Drop to make
        // a duplicate, uncertain append attempt.
        self.finished = true;
        self.write_event(1, "terminal", Some(outcome), reason)
    }

    fn write_event(
        &mut self,
        sequence: u8,
        event: &'static str,
        outcome: Option<&str>,
        reason: Option<&str>,
    ) -> io::Result<()> {
        let elapsed_millis = (event == "terminal")
            .then(|| self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        let row = serde_json::json!({
            "schema": DEPLOYED_QUALITY_INVOCATION_TERMINAL_SCHEMA,
            "semantic_admission_input": false,
            "sequence": sequence,
            "event": event,
            "outcome": outcome,
            "reason": reason,
            "elapsed_millis": elapsed_millis,
            "bundle": &self.context.bundle,
            "compiler_revision": &self.context.compiler_revision,
            "mode": self.context.mode,
            "positions": self.context.positions,
            "workers": self.context.workers,
            "worker_source": if self.context.workers.is_some() {
                "explicit"
            } else {
                "available-parallelism"
            },
            "probe_budget_secs": self.context.probe_budget_secs,
            "eval_budget_secs": self.context.eval_budget_secs,
            "cross_surface_evidence": &self.context.cross_surface_evidence,
        });
        serde_json::to_writer(&mut self.file, &row)?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        self.file.sync_all()
    }
}

impl Drop for DeployedQualityInvocationTerminal {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let _ = self.write_event(
            1,
            "terminal",
            Some("interrupted"),
            Some("command unwound before returning a completed or failed outcome"),
        );
    }
}

fn deployed_quality_command(args: &DeployedQualityArgs) -> Result<(), RunError> {
    let mut terminal = DeployedQualityInvocationTerminal::begin(args)?;
    let result = deployed_quality_command_inner(args);
    let (outcome, reason) = match &result {
        Ok(()) => ("completed", None),
        Err(error) => ("failed", Some(error.to_string())),
    };
    let terminal_result = terminal.finish(outcome, reason.as_deref());
    match (result, terminal_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(terminal_error)) => Err(RunError::Command(format!(
            "deployed-quality completed, but its invocation terminal could not be finalized at {}: {terminal_error}",
            terminal.path.display()
        ))),
        (Err(error), Err(terminal_error)) => Err(RunError::Command(format!(
            "{error}; additionally, the invocation terminal could not be finalized at {}: {terminal_error}",
            terminal.path.display()
        ))),
    }
}

fn deployed_quality_command_inner(args: &DeployedQualityArgs) -> Result<(), RunError> {
    use uor_r4_api::serving_eval::{ServingBundle, ServingEvalMode, ServingReportPaths};
    if args.compiler_revision.len() != 40
        || !args
            .compiler_revision
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(RunError::Command(
            "--compiler-revision must be a full 40-character hexadecimal git revision".to_owned(),
        ));
    }
    if args.positions == 0 || args.workers == Some(0) {
        return Err(RunError::Command(
            "--positions and --workers must be positive".to_owned(),
        ));
    }
    let bundle = ServingBundle::discover(&args.bundle).ok_or_else(|| {
        RunError::Command(format!(
            "{} is not the exact compiled serving bundle requested; required graph, artifact, store, and corpus files are unavailable",
            args.bundle.display()
        ))
    })?;
    let workers = args.workers.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
    });
    let cross_surface_path = args.cross_surface_evidence.clone().unwrap_or_else(|| {
        bundle
            .graph
            .parent()
            .unwrap_or(bundle.root.as_path())
            .join("cross_surface_parity.json")
    });
    if args.cross_surface_evidence.is_none() {
        produce_cross_surface_evidence(&bundle, &cross_surface_path)?;
    }
    let cross_surface_evidence = read_regular_evidence(&cross_surface_path)?;

    match args.mode {
        DeployedQualityMode::Sample => {
            let graph_dir = bundle.graph.parent().unwrap_or(bundle.root.as_path());
            let paths = ServingReportPaths {
                progress_jsonl: graph_dir.join("deployed_quality_research_sample_progress.jsonl"),
                terminal_json: graph_dir.join("deployed_quality_research_sample_terminal.json"),
                deployed_quality_json: graph_dir
                    .join("deployed_quality_research_sample_report.json"),
            };
            let witness_path = graph_dir.join("witness_replay_research_sample.json");
            let run = run_deployed_quality_once(
                args,
                &bundle,
                ServingEvalMode::Sample {
                    positions: args.positions,
                },
                workers,
                &cross_surface_path,
                &cross_surface_evidence,
                paths,
                witness_path,
                None,
                "sample",
            )?;
            report_deployed_quality_result("sample", &run)
        }
        DeployedQualityMode::Full => {
            if args.positions != uor_r4_api::serving_eval::SAMPLE_TARGET {
                return Err(RunError::Command(format!(
                    "full mode requires the binding {}-position sample gate; omit --positions or set it exactly",
                    uor_r4_api::serving_eval::SAMPLE_TARGET
                )));
            }
            let graph_dir = bundle.graph.parent().unwrap_or(bundle.root.as_path());
            let binding_sample_positions = binding_sample_target(&bundle)?;
            let sample_paths = ServingReportPaths {
                progress_jsonl: graph_dir.join("deployed_quality_sample_progress.jsonl"),
                terminal_json: graph_dir.join("deployed_quality_sample_terminal.json"),
                deployed_quality_json: graph_dir.join("deployed_quality_sample_report.json"),
            };
            let sample_witness = graph_dir.join("witness_replay_sample.json");
            let sample = run_deployed_quality_once(
                args,
                &bundle,
                ServingEvalMode::Sample {
                    positions: binding_sample_positions,
                },
                workers,
                &cross_surface_path,
                &cross_surface_evidence,
                sample_paths,
                sample_witness,
                None,
                "binding-sample",
            )?;
            report_deployed_quality_result("binding-sample", &sample)?;
            let (sample_kind, sample_decision, sample_row) =
                deployed_quality_sample_decision(&sample, "binding sample")?;
            let sample_generation_cid = sample_row.generation_cid.clone();
            let mut gate_elapsed_millis = sample_row.elapsed_millis;
            let mut gate_sample_n = sample_row.sample_n;
            let mut gate_population_n = sample_row.population_n;
            let mut authorizing_decision = sample_decision.to_owned();

            match deployed_quality_next_phase(
                sample_kind,
                sample_row.sample_n,
                sample_row.population_n,
            ) {
                DeployedQualityNextPhase::Census => {}
                DeployedQualityNextPhase::Stop => {
                    return Err(RunError::Command(format!(
                        "binding sample verdict is {sample_decision}; full census was not launched"
                    )));
                }
                DeployedQualityNextPhase::ExtendedSample { positions } => {
                    let extended_paths = ServingReportPaths {
                        progress_jsonl: graph_dir
                            .join("deployed_quality_extended_sample_progress.jsonl"),
                        terminal_json: graph_dir
                            .join("deployed_quality_extended_sample_terminal.json"),
                        deployed_quality_json: graph_dir
                            .join("deployed_quality_extended_sample_report.json"),
                    };
                    let extended_witness = graph_dir.join("witness_replay_extended_sample.json");
                    let extended = run_deployed_quality_once(
                        args,
                        &bundle,
                        ServingEvalMode::Sample { positions },
                        workers,
                        &cross_surface_path,
                        &cross_surface_evidence,
                        extended_paths,
                        extended_witness,
                        Some(&sample_generation_cid),
                        "extended-binding-sample",
                    )?;
                    report_deployed_quality_result("extended-binding-sample", &extended)?;
                    let (extended_kind, extended_decision, extended_row) =
                        deployed_quality_sample_decision(&extended, "extended binding sample")?;
                    match deployed_quality_next_phase(
                        extended_kind,
                        extended_row.sample_n,
                        extended_row.population_n,
                    ) {
                        DeployedQualityNextPhase::Census => {}
                        DeployedQualityNextPhase::Stop => {
                            return Err(RunError::Command(format!(
                                "extended binding sample verdict is {extended_decision}; full census was not launched"
                            )));
                        }
                        DeployedQualityNextPhase::ExtendedSample { .. } => {
                            return Err(RunError::Command(format!(
                                "extended binding sample selected fewer than {} positions; full census was not launched",
                                uor_r4_api::serving_eval::EXTENDED_SAMPLE_TARGET
                            )));
                        }
                    }
                    if extended_row.generation_cid != sample_generation_cid {
                        return Err(RunError::Command(
                            "extended binding sample changed evidence generation; full census was not launched"
                                .to_owned(),
                        ));
                    }
                    if extended_kind == uor_r4_api::SampleDecisionKind::Inconclusive {
                        let reachability_ceiling_ppm = deployed_quality_reachability_ceiling_ppm(
                            extended_row,
                        )
                        .ok_or_else(|| {
                            RunError::Command(
                                "extended sample reachability ceiling overflowed".to_owned(),
                            )
                        })?;
                        if reachability_ceiling_ppm < uor_r4_api::RF31_MIN_LANE_DELTA_PPM as u128 {
                            return Err(RunError::Command(format!(
                                "extended sample remains inconclusive but its reachability ceiling is only {reachability_ceiling_ppm} ppm; full census was not launched"
                            )));
                        }
                    }
                    gate_elapsed_millis = extended_row.elapsed_millis;
                    gate_sample_n = extended_row.sample_n;
                    gate_population_n = extended_row.population_n;
                    authorizing_decision = extended_decision.to_owned();
                }
            }
            let projected_full_millis =
                projected_census_millis(gate_elapsed_millis, gate_sample_n, gate_population_n)
                    .ok_or_else(|| {
                        RunError::Command("full-census wall-clock projection overflowed".to_owned())
                    })?;
            let contract_ceiling_millis = u128::from(3_600_000_u64);
            let configured_ceiling_millis = u128::from(args.eval_budget_secs)
                .saturating_mul(1_000)
                .min(contract_ceiling_millis);
            if projected_full_millis > configured_ceiling_millis {
                return Err(RunError::Command(format!(
                    "binding funnel is {authorizing_decision}, but the full census projects to {projected_full_millis} ms (> configured/one-hour launch ceiling {configured_ceiling_millis} ms); post a revised arithmetic/run contract before launch"
                )));
            }
            println!(
                "binding funnel authorized the full census: {authorizing_decision}; projected_full_millis={projected_full_millis}"
            );

            let paths = ServingReportPaths::in_bundle(&bundle);
            let witness_path = bundle
                .root
                .join(uor_r4_api::NORMATIVE_WITNESS_REPLAY_BUNDLE_PATH);
            let full = run_deployed_quality_once(
                args,
                &bundle,
                ServingEvalMode::FullCensus,
                workers,
                &cross_surface_path,
                &cross_surface_evidence,
                paths,
                witness_path,
                Some(&sample_generation_cid),
                "full-census",
            )?;
            report_deployed_quality_result("full-census", &full)
        }
    }
}

fn deployed_quality_sample_decision<'a>(
    run: &'a DeployedQualityRun,
    label: &str,
) -> Result<
    (
        uor_r4_api::SampleDecisionKind,
        &'a str,
        &'a uor_r4_api::serving_eval::ServingEvalRow,
    ),
    RunError,
> {
    let report = run.recorded.report.as_ref().ok_or_else(|| {
        RunError::Command(format!(
            "{label} did not emit a report; full census is refused"
        ))
    })?;
    if report.evaluation.mode != uor_r4_api::EvaluationMode::Sample {
        return Err(RunError::Command(format!(
            "{label} did not emit a sample report; full census is refused"
        )));
    }
    let uor_r4_api::QualityVerdict::Estimate { decision } = &report.evaluation.verdict else {
        return Err(RunError::Command(format!(
            "{label} did not emit a typed sample estimate; full census is refused"
        )));
    };
    let kind = report.evaluation.verdict.sample_decision().ok_or_else(|| {
        RunError::Command(format!(
            "{label} emitted an invalid sample decision; full census is refused"
        ))
    })?;
    let row = match &run.recorded.outcome {
        uor_r4_api::serving_eval::ServingEvalOutcome::Row(row) => row.as_ref(),
        uor_r4_api::serving_eval::ServingEvalOutcome::Skipped(_) => {
            return Err(RunError::Command(format!(
                "{label} was skipped; full census is refused"
            )));
        }
    };
    Ok((kind, decision, row))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeployedQualityNextPhase {
    ExtendedSample { positions: usize },
    Census,
    Stop,
}

fn initial_non_census_target(population_positions: usize) -> Option<usize> {
    let max_non_census = population_positions.checked_sub(1)?;
    (max_non_census > 0).then_some(uor_r4_api::serving_eval::SAMPLE_TARGET.min(max_non_census))
}

fn binding_sample_target(
    bundle: &uor_r4_api::serving_eval::ServingBundle,
) -> Result<usize, RunError> {
    let corpus_meta = read_regular_evidence(&bundle.corpus_meta)?;
    let corpus_records = read_regular_evidence(&bundle.corpus_records)?;
    let corpus = uor_r4_core::transformerless::compiler::load_corpus_bytes(
        &corpus_meta,
        &corpus_records,
        None,
    )
    .ok_or_else(|| {
        RunError::Command(
            "corpus.meta/corpus.records are UNAVAILABLE for binding-sample sizing".into(),
        )
    })?;
    let (_, held_out) = uor_r4_core::transformerless::compiler::split_positions(&corpus);
    initial_non_census_target(held_out.len()).ok_or_else(|| {
        RunError::Command(
            "deployed-quality requires at least two held-out positions to run a non-census binding sample"
                .into(),
        )
    })
}

fn deployed_quality_next_phase(
    decision: uor_r4_api::SampleDecisionKind,
    evaluated_positions: usize,
    population_positions: usize,
) -> DeployedQualityNextPhase {
    match decision {
        uor_r4_api::SampleDecisionKind::Proceed => DeployedQualityNextPhase::Census,
        uor_r4_api::SampleDecisionKind::Stop => DeployedQualityNextPhase::Stop,
        uor_r4_api::SampleDecisionKind::Inconclusive
            if evaluated_positions < uor_r4_api::serving_eval::EXTENDED_SAMPLE_TARGET =>
        {
            let positions = uor_r4_api::serving_eval::EXTENDED_SAMPLE_TARGET
                .min(population_positions.saturating_sub(1));
            if positions > evaluated_positions {
                DeployedQualityNextPhase::ExtendedSample { positions }
            } else {
                DeployedQualityNextPhase::Census
            }
        }
        uor_r4_api::SampleDecisionKind::Inconclusive => DeployedQualityNextPhase::Census,
    }
}

fn deployed_quality_reachability_ceiling_ppm(
    row: &uor_r4_api::serving_eval::ServingEvalRow,
) -> Option<u128> {
    (row.sample_n > 0)
        .then(|| u128::from(row.lane_reachable).saturating_mul(1_000_000) / row.sample_n as u128)
}

fn projected_census_millis(
    sample_elapsed_millis: u64,
    sample_n: usize,
    population_n: usize,
) -> Option<u128> {
    if sample_n == 0 || population_n < sample_n {
        return None;
    }
    let sample_n = sample_n as u128;
    u128::from(sample_elapsed_millis.max(1))
        .checked_mul(population_n as u128)?
        .checked_add(sample_n - 1)
        .map(|value| value / sample_n)
}

struct DeployedQualityRun {
    recorded: uor_r4_api::serving_eval::RecordedServingEval,
    paths: uor_r4_api::serving_eval::ServingReportPaths,
}

#[allow(clippy::too_many_arguments)]
fn run_deployed_quality_once(
    args: &DeployedQualityArgs,
    bundle: &uor_r4_api::serving_eval::ServingBundle,
    mode: uor_r4_api::serving_eval::ServingEvalMode,
    workers: usize,
    cross_surface_path: &Path,
    cross_surface_evidence: &[u8],
    paths: uor_r4_api::serving_eval::ServingReportPaths,
    witness_path: PathBuf,
    expected_generation_cid: Option<&str>,
    label: &str,
) -> Result<DeployedQualityRun, RunError> {
    use uor_r4_api::serving_eval::{self, ServingEvalBudgets, ServingReportEvidence};
    use uor_r4_api::{
        produce_normative_witness_replay, NormativeWitnessReplayMaterial,
        NormativeWitnessReplaySpec, DEFAULT_NORMATIVE_WITNESS_SAMPLE,
    };

    let snapshot = serving_eval::ServingBundleSnapshot::capture(bundle)
        .map_err(|error| RunError::Command(format!("deployed-quality UNAVAILABLE: {error}")))?;
    if let Some(expected) = expected_generation_cid {
        snapshot
            .require_generation(expected)
            .map_err(|error| RunError::Command(error.to_string()))?;
    }
    let score_report = snapshot.score_report().ok_or_else(|| {
        RunError::Command(
            "score_report.json is UNAVAILABLE; no quality report can be emitted".into(),
        )
    })?;
    let tokenizer = snapshot.tokenizer().ok_or_else(|| {
        RunError::Command(
            "tokenizer.bin is UNAVAILABLE; normative witness replay cannot be produced".into(),
        )
    })?;
    let corpus = uor_r4_core::transformerless::compiler::load_corpus_bytes(
        snapshot.corpus_meta(),
        snapshot.corpus_records(),
        None,
    )
    .ok_or_else(|| {
        RunError::Command("corpus.meta/corpus.records do not form a completed corpus".into())
    })?;
    let (_, held_out) = uor_r4_graph_compiler::induction::split_positions(&corpus);
    let selection = serving_eval::select_serving_eval_positions(&corpus.story, &held_out, mode)
        .map_err(|error| {
            RunError::Command(format!(
                "canonical deployed-quality position selection: {error}"
            ))
        })?;
    let selected = selection.positions;
    if selected.is_empty() {
        return Err(RunError::Command(
            "held-out evaluation population is empty; evidence is UNAVAILABLE".into(),
        ));
    }
    let evaluated_positions: Vec<u64> = selected
        .iter()
        .map(|&position| {
            u64::try_from(position).map_err(|_| {
                RunError::Command("held-out position does not fit the u64 evidence format".into())
            })
        })
        .collect::<Result<_, _>>()?;
    let witness = produce_normative_witness_replay(NormativeWitnessReplaySpec {
        material: NormativeWitnessReplayMaterial {
            graph: snapshot.graph(),
            signature_artifact: snapshot.signature_artifact(),
            tokenizer,
            score_report: Some(score_report),
            corpus_meta: snapshot.corpus_meta(),
            corpus_records: snapshot.corpus_records(),
        },
        evaluated_positions: &evaluated_positions,
        sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
    })
    .map_err(|error| RunError::Command(format!("normative witness production: {error}")))?;
    let witness_replay_evidence = witness
        .deterministic_json_bytes()
        .map_err(|error| RunError::Command(format!("serialize witness replay: {error}")))?;
    write_atomic_evidence(&witness_path, &witness_replay_evidence)?;

    let evidence = ServingReportEvidence {
        compiler_revision: args.compiler_revision.clone(),
        cross_surface_evidence: cross_surface_evidence.to_vec(),
        witness_replay_evidence,
    };
    let budgets = ServingEvalBudgets {
        probe: Duration::from_secs(args.probe_budget_secs),
        eval: Duration::from_secs(args.eval_budget_secs),
        mode,
        workers,
    };
    println!(
        "deployed-quality contract: phase={label} bundle={} generation_cid={} mode={:?} selected={}/{} workers={} probe_budget={}s eval_budget={}s cross_surface={} witness={}",
        bundle.root.display(),
        snapshot.generation_cid(),
        mode,
        selected.len(),
        held_out.len(),
        workers,
        args.probe_budget_secs,
        args.eval_budget_secs,
        cross_surface_path.display(),
        witness_path.display(),
    );
    let mut progress = |state: serving_eval::ServingProgress| {
        println!(
            "progress run={} phase={} {}/{} workers={} served={} abstained={} declined={} hits(runtime/tla)={}/{} lane(reachable/changed/toward/away)={}/{}/{}/{} controls(absent/shuffled)={}/{} elapsed_ms={} rate_milli={} eta_s={}",
            label,
            state.phase,
            state.processed,
            state.total,
            state.workers,
            state.served,
            state.abstained,
            state.declined,
            state.normative_hits,
            state.tla_hits,
            state.lane_reachable,
            state.lane_changed,
            state.lane_toward,
            state.lane_away,
            state.sections_absent_hits,
            state.label_shuffled_hits,
            state.elapsed_millis,
            state.positions_per_second_milli,
            state
                .eta_seconds
                .map_or_else(|| "unavailable".to_owned(), |eta| eta.to_string()),
        );
    };
    let recorded = serving_eval::evaluate_serving_snapshot_recorded(
        &snapshot,
        budgets,
        &evidence,
        &paths,
        &mut progress,
    )
    .map_err(|error| RunError::Command(format!("deployed-quality UNAVAILABLE: {error}")))?;
    Ok(DeployedQualityRun { recorded, paths })
}

fn report_deployed_quality_result(label: &str, run: &DeployedQualityRun) -> Result<(), RunError> {
    use uor_r4_api::serving_eval::ServingEvalOutcome;

    match (
        &run.recorded.outcome,
        &run.recorded.report,
        &run.recorded.report_cid,
    ) {
        (ServingEvalOutcome::Row(row), Some(report), Some(report_cid)) => {
            println!(
                "deployed-quality complete: phase={label} evaluated={}/{} elapsed_ms={} report={} CID={} verdict={:?}",
                row.sample_n,
                row.population_n,
                row.elapsed_millis,
                run.paths.deployed_quality_json.display(),
                report_cid,
                report.evaluation.verdict,
            );
            println!("progress artifact: {}", run.paths.progress_jsonl.display());
            println!("terminal artifact: {}", run.paths.terminal_json.display());
            Ok(())
        }
        (ServingEvalOutcome::Skipped(skip), _, _) => Err(RunError::Command(format!(
            "deployed-quality SKIPPED with durable terminal evidence: {skip:?}"
        ))),
        _ => Err(RunError::Command(
            "deployed-quality completed without a report; terminal evidence is authoritative"
                .into(),
        )),
    }
}

fn produce_cross_surface_evidence(
    bundle: &uor_r4_api::serving_eval::ServingBundle,
    expected_path: &Path,
) -> Result<(), RunError> {
    use uor_r4_wasm_router::cross_surface_parity::{
        produce_canonical_cross_surface_parity_with_progress, write_canonical_cross_surface_parity,
        CanonicalCrossSurfaceMaterial, CanonicalCrossSurfaceSpec,
    };

    let canonical_path = bundle
        .root
        .join(uor_r4_api::CROSS_SURFACE_PARITY_BUNDLE_PATH);
    if expected_path != canonical_path {
        return Err(RunError::Command(format!(
            "canonical cross-surface path disagreement: expected {}, resolved {}",
            canonical_path.display(),
            expected_path.display()
        )));
    }
    let graph = read_regular_evidence(&bundle.graph)?;
    let signature_artifact = read_regular_evidence(&bundle.teacher)?;
    let tokenizer = bundle
        .tokenizer
        .as_deref()
        .map(read_regular_evidence)
        .transpose()?;
    let score_report_path = bundle.score_report.as_deref().ok_or_else(|| {
        RunError::Command(
            "score_report.json is UNAVAILABLE; cross-surface evidence cannot be produced".into(),
        )
    })?;
    let score_report = read_regular_evidence(score_report_path)?;
    let corpus_meta = read_regular_evidence(&bundle.corpus_meta)?;
    let corpus_records = read_regular_evidence(&bundle.corpus_records)?;
    let corpus = uor_r4_core::transformerless::compiler::load_corpus_bytes(
        &corpus_meta,
        &corpus_records,
        None,
    )
    .ok_or_else(|| {
        RunError::Command(
            "corpus.meta/corpus.records are UNAVAILABLE for cross-surface evidence".into(),
        )
    })?;
    let (_, held_out) = uor_r4_core::transformerless::compiler::split_positions(&corpus);
    let selection = uor_r4_api::serving_eval::select_serving_eval_positions(
        &corpus.story,
        &held_out,
        uor_r4_api::serving_eval::ServingEvalMode::Sample {
            positions: uor_r4_api::serving_eval::SAMPLE_TARGET,
        },
    )
    .map_err(|error| {
        RunError::Command(format!(
            "canonical cross-surface position selection: {error}"
        ))
    })?;
    let positions: Vec<u64> = selection
        .positions
        .into_iter()
        .map(|position| {
            u64::try_from(position).map_err(|_| {
                RunError::Command(
                    "held-out position exceeds the cross-surface evidence format".into(),
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let spec = CanonicalCrossSurfaceSpec {
        material: CanonicalCrossSurfaceMaterial {
            graph: &graph,
            signature_artifact: &signature_artifact,
            tokenizer: tokenizer.as_deref(),
            score_report: Some(&score_report),
            corpus_meta: &corpus_meta,
            corpus_records: &corpus_records,
        },
        evaluated_positions: &positions,
        sample_seed: DEFAULT_SAMPLE_SEED,
    };
    let first_started = std::time::Instant::now();
    println!(
        "cross-surface scan: pass=primary population={} progress_interval=256",
        positions.len()
    );
    let first = produce_canonical_cross_surface_parity_with_progress(spec, |scanned, total| {
        report_cross_surface_progress("primary", first_started, scanned, total);
    })
    .map_err(|error| RunError::Command(format!("cross-surface evidence UNAVAILABLE: {error}")))?;
    let second_started = std::time::Instant::now();
    println!(
        "cross-surface scan: pass=determinism-replay population={} progress_interval=256",
        positions.len()
    );
    let second = produce_canonical_cross_surface_parity_with_progress(spec, |scanned, total| {
        report_cross_surface_progress("determinism-replay", second_started, scanned, total);
    })
    .map_err(|error| {
        RunError::Command(format!(
            "cross-surface determinism reproduction UNAVAILABLE: {error}"
        ))
    })?;
    let first_bytes = first
        .evidence
        .deterministic_json_bytes()
        .map_err(|error| RunError::Command(format!("serialize cross-surface evidence: {error}")))?;
    let second_bytes = second
        .evidence
        .deterministic_json_bytes()
        .map_err(|error| RunError::Command(format!("serialize cross-surface replay: {error}")))?;
    if first.selected_position != second.selected_position
        || first.scanned_positions != second.scanned_positions
        || first_bytes != second_bytes
    {
        return Err(RunError::Command(
            "cross-surface producer is nondeterministic across two identical builds".into(),
        ));
    }
    let path = write_canonical_cross_surface_parity(&bundle.root, &first.evidence)
        .map_err(|error| RunError::Command(format!("write cross-surface evidence: {error}")))?;
    println!(
        "cross-surface evidence: selected_position={} scanned_positions={} checks={} mismatches={} bytes={} CID=blake3:{} path={}",
        first.selected_position,
        first.scanned_positions,
        first.evidence.checks,
        first.evidence.mismatches,
        first_bytes.len(),
        blake3::hash(&first_bytes).to_hex(),
        path.display(),
    );
    let inventory_error = first
        .evidence
        .validate_canonical_production_inventory()
        .err()
        .map(|error| error.to_string());
    let terminal_outcome = if first.evidence.mismatches != 0 {
        "stop-mismatch"
    } else if inventory_error.is_some() {
        "stop-inventory"
    } else {
        "pass"
    };
    let terminal_path = write_cross_surface_terminal(
        &bundle.root,
        first.selected_position,
        first.scanned_positions,
        &first.evidence,
        &first_bytes,
        terminal_outcome,
        inventory_error.as_deref(),
    )?;
    println!("cross-surface terminal: {}", terminal_path.display());
    if first.evidence.mismatches != 0 {
        return Err(RunError::Command(format!(
            "cross-surface STOP: {} candidate/token mismatches across {} mechanically executed checks; deterministic evidence and terminal artifacts were persisted",
            first.evidence.mismatches, first.evidence.checks
        )));
    }
    if let Some(error) = inventory_error {
        return Err(RunError::Command(format!(
            "cross-surface STOP: canonical production inventory did not validate; deterministic evidence and terminal artifacts were persisted: {error}"
        )));
    }
    Ok(())
}

fn write_cross_surface_terminal(
    bundle_root: &Path,
    selected_position: u64,
    scanned_positions: usize,
    evidence: &uor_r4_api::CrossSurfaceParityEvidence,
    evidence_bytes: &[u8],
    outcome: &str,
    reason: Option<&str>,
) -> Result<PathBuf, RunError> {
    let path = bundle_root.join("graph/cross_surface_parity_terminal.json");
    let terminal = serde_json::json!({
        "schema": "uor-r4-cross-surface-terminal/1",
        "outcome": outcome,
        "reason": reason,
        "selected_position": selected_position,
        "scanned_positions": scanned_positions,
        "checks": evidence.checks,
        "mismatches": evidence.mismatches,
        "evidence_cid": format!("blake3:{}", blake3::hash(evidence_bytes).to_hex()),
    });
    let mut bytes = serde_json::to_vec_pretty(&terminal)
        .map_err(|error| RunError::Command(format!("serialize cross-surface terminal: {error}")))?;
    bytes.push(b'\n');
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                return Err(RunError::Command(format!(
                    "cross-surface terminal target {} is not a regular non-symlink file",
                    path.display()
                )));
            }
            let existing = std::fs::read(&path).map_err(|error| {
                RunError::Command(format!(
                    "read cross-surface terminal {}: {error}",
                    path.display()
                ))
            })?;
            if existing == bytes {
                return Ok(path);
            }
            return Err(RunError::Command(format!(
                "cross-surface terminal {} already exists with different bytes; refusing to overwrite another generation",
                path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(RunError::Command(format!(
                "inspect cross-surface terminal {}: {error}",
                path.display()
            )));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            RunError::Command(format!(
                "create cross-surface terminal {}: {error}",
                path.display()
            ))
        })?;
    use std::io::Write as _;
    file.write_all(&bytes).map_err(|error| {
        RunError::Command(format!(
            "write cross-surface terminal {}: {error}",
            path.display()
        ))
    })?;
    file.sync_all().map_err(|error| {
        RunError::Command(format!(
            "sync cross-surface terminal {}: {error}",
            path.display()
        ))
    })?;
    Ok(path)
}

fn report_cross_surface_progress(
    pass: &str,
    started: std::time::Instant,
    scanned: usize,
    total: usize,
) {
    let elapsed_millis = started.elapsed().as_millis().max(1);
    let scanned_u128 = scanned as u128;
    let rate_milli_positions_per_second = scanned_u128.saturating_mul(1_000_000) / elapsed_millis;
    let remaining = total.saturating_sub(scanned) as u128;
    let eta_millis = remaining.saturating_mul(elapsed_millis) / scanned_u128.max(1);
    println!(
        "cross-surface progress: pass={pass} scanned={scanned}/{total} elapsed_ms={elapsed_millis} rate_milli_positions_per_second={rate_milli_positions_per_second} eta_ms={eta_millis}"
    );
}

fn read_regular_evidence(path: &Path) -> Result<Vec<u8>, RunError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        RunError::Command(format!("required evidence {}: {error}", path.display()))
    })?;
    if !metadata.file_type().is_file() {
        return Err(RunError::Command(format!(
            "required evidence {} is not a regular non-symlink file",
            path.display()
        )));
    }
    std::fs::read(path).map_err(RunError::Io)
}

fn write_atomic_evidence(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let parent = path.parent().ok_or_else(|| {
        RunError::Command(format!("evidence path {} has no parent", path.display()))
    })?;
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            RunError::Command(format!("evidence path {} is not UTF-8", path.display()))
        })?;
    let temporary = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let write_result = (|| -> Result<(), RunError> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| {
                RunError::Command(format!(
                    "create temporary evidence {}: {error}",
                    temporary.display()
                ))
            })?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    write_result
}

/// The historical certify-C research diagnostic (issue #280). It measures a
/// row but does not produce the raw parity/witness controls, content-bound
/// report, or sample-gate decision required for production. Use
/// `r4 deployed-quality` for #933 admission evidence. `R4_CERTIFY_C_ONLY=1`
/// invokes only this non-binding diagnostic; it never fails the certify run.
fn certify_serving_row() {
    use uor_r4_api::serving_eval::{
        self, ServingBundle, ServingEvalBudgets, ServingEvalOutcome, ServingEvalSkip,
    };
    let bundles = match std::env::var("R4_CERTIFY_SERVING_BUNDLE") {
        Ok(root) => match ServingBundle::discover(std::path::Path::new(&root)) {
            Some(bundle) => vec![bundle],
            None => {
                println!(
                    "C serving: SKIPPED — R4_CERTIFY_SERVING_BUNDLE={root} is not a compiled serving bundle (needs score.r4g1, tless_artifacts.bin, tless_store.bin, corpus.meta, corpus.records)"
                );
                return;
            }
        },
        Err(_) => ServingBundle::scan(std::path::Path::new(".")),
    };
    if bundles.is_empty() {
        println!(
            "C serving: SKIPPED — no compiled serving bundle under .uor-models/compiled (run `r4 compile`, or set R4_CERTIFY_SERVING_BUNDLE). No measurement recorded."
        );
        return;
    }
    let budgets = ServingEvalBudgets::from_env();
    println!(
        "C research diagnostic (NOT production admission): readiness probe {} positions with accuracy spot-check; mode {:?}; {} workers; wall-clock budgets probe {}s / eval {}s",
        serving_eval::PROBE_POSITIONS,
        budgets.mode,
        budgets.workers,
        budgets.probe.as_secs(),
        budgets.eval.as_secs()
    );
    let mut measured = false;
    for bundle in &bundles {
        if measured {
            println!(
                "C serving: bundle {} not evaluated this run (one bundle per certify; select with R4_CERTIFY_SERVING_BUNDLE)",
                bundle.root.display()
            );
            continue;
        }
        let mut progress = |state: serving_eval::ServingProgress| {
            println!(
                "progress: phase={} {}/{} ({}%) workers={} served={} abstained={} declined={} hits(runtime/tla)={}/{} lane(reachable/changed/toward/away)={}/{}/{}/{} elapsed={:.1}s rate={:.2} pos/s eta={}s",
                state.phase,
                state.processed,
                state.total,
                state
                    .processed
                    .saturating_mul(100)
                    .checked_div(state.total)
                    .unwrap_or(0),
                state.workers,
                state.served,
                state.abstained,
                state.declined,
                state.normative_hits,
                state.tla_hits,
                state.lane_reachable,
                state.lane_changed,
                state.lane_toward,
                state.lane_away,
                state.elapsed_millis as f64 / 1000.0,
                state.positions_per_second_milli as f64 / 1000.0,
                state.eta_seconds.unwrap_or(0),
            );
        };
        match serving_eval::evaluate_serving_bundle(bundle, budgets, &mut progress) {
            Ok(ServingEvalOutcome::Row(row)) => {
                measured = true;
                let pct = |part: u64, whole: u64| {
                    if whole == 0 {
                        0.0
                    } else {
                        100.0 * part as f64 / whole as f64
                    }
                };
                println!(
                    "C research diagnostic (non-binding; R4G1Runtime candidates + token-free D4 policy, {:?}, n={}/{}, workers={}, {:.2}s) bundle {}: served {} ({:.1}%; exact {} ({} ngram), graph {}, novel {}; {} widened) | abstained {} (exact {}, graph {}, novel {}, contradictory {}) | declined {} | teacher agreement runtime/base/TLA {:.2}%/{:.2}%/{:.2}% | paired runtime-vs-TLA both/only/TLA-only/neither {}/{}/{}/{} | lane reachable/changed/toward/away {}/{}/{}/{} | probe: {}/{} served, {} hits",
                    row.mode,
                    row.sample_n,
                    row.population_n,
                    row.workers,
                    row.elapsed_millis as f64 / 1000.0,
                    row.bundle.display(),
                    row.served,
                    pct(row.served, row.sample_n as u64),
                    row.served_by.exact_context,
                    row.served_by.exact_context_ngram,
                    row.served_by.graph,
                    row.served_by.novel,
                    row.served_widened,
                    row.abstained.total(),
                    row.abstained.exact_context,
                    row.abstained.graph,
                    row.abstained.novel,
                    row.abstained.contradictory,
                    row.declined,
                    pct(row.agree_served, row.sample_n as u64),
                    pct(row.base_hits, row.sample_n as u64),
                    pct(row.tla_hits, row.sample_n as u64),
                    row.normative_vs_tla.both,
                    row.normative_vs_tla.normative_only,
                    row.normative_vs_tla.comparator_only,
                    row.normative_vs_tla.neither,
                    row.lane_reachable,
                    row.lane_changed,
                    row.lane_toward,
                    row.lane_away,
                    row.probe_served,
                    row.probe_positions,
                    row.probe_hits
                );
            }
            Ok(ServingEvalOutcome::Skipped(skip)) => {
                measured = true;
                match skip {
                    ServingEvalSkip::ProbeBudgetExceeded { probed, elapsed } => println!(
                        "C serving: SKIPPED — readiness probe exceeded its {}s wall-clock budget after {}/{} positions ({}s elapsed). Full evaluation not run; no measurement recorded.",
                        budgets.probe.as_secs(),
                        probed,
                        serving_eval::PROBE_POSITIONS,
                        elapsed.as_secs()
                    ),
                    ServingEvalSkip::ProbeFunctionalCheckFailed { served, probed } => println!(
                        "C serving: SKIPPED — probe served {served} predictions across {probed} positions with zero accuracy hits (#280 functional spot-check). Full evaluation not run; no measurement recorded.",
                    ),
                    ServingEvalSkip::EvalBudgetExceeded {
                        done,
                        sample_n,
                        elapsed,
                    } => println!(
                        "C serving: SKIPPED — subsampled evaluation (n={sample_n}) exceeded its {}s wall-clock budget after {done}/{sample_n} positions ({}s elapsed; R4_CERTIFY_R4G1_EVAL_BUDGET_SECS overrides). Partial counts discarded; no measurement recorded.",
                        budgets.eval.as_secs(),
                        elapsed.as_secs()
                    ),
                }
            }
            Err(error) => {
                println!(
                    "C serving: bundle {} unusable ({error}); trying next candidate",
                    bundle.root.display()
                );
            }
        }
    }
    if !measured {
        println!(
            "C serving: SKIPPED — no loadable serving bundle produced a measurement. No measurement recorded."
        );
    }
}

fn audit_command(log_file: &PathBuf) -> Result<(), RunError> {
    if !log_file.exists() {
        println!(
            "\x1b[33m[!] No audit log file found at {}\x1b[0m",
            log_file.display()
        );
        println!("    Run the interactive client ('r4 client') and ask questions first.\n");
        return Ok(());
    }
    let content = std::fs::read_to_string(log_file)?;
    let records: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| RunError::Command(format!("Failed to parse audit log: {}", e)))?;

    println!(
        "\n\x1b[1;36m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m"
    );
    println!(
        "\x1b[1;36m│  R⁴ UOR Auditability & Tracing Log Inspector                                │\x1b[0m"
    );
    println!(
        "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
    );
    println!("  Audit Log Path: \x1b[1m{}\x1b[0m", log_file.display());

    if let Some(arr) = records.as_array() {
        println!("  Total Audited Turns: \x1b[32m{}\x1b[0m", arr.len());
        for (idx, item) in arr.iter().enumerate() {
            println!(
                "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
            );
            let q = item.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let a = item.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let audit = item.get(2);

            let short_q = if q.len() > 60 {
                format!("{}...", &q[..60])
            } else {
                q.to_string()
            };
            let short_a = if a.len() > 60 {
                format!("{}...", &a[..60])
            } else {
                a.to_string()
            };
            println!("  [\x1b[1mTurn #{}\x1b[0m] Q: {}", idx + 1, short_q);
            println!("          A: {}", short_a);

            if let Some(aud) = audit.filter(|v| !v.is_null()) {
                let uor_addr = aud["uor_address"].as_str().unwrap_or("N/A");
                let kappa = aud["kappa"].as_f64().unwrap_or(0.0);
                let kappa_pass = aud["kappa_pass"].as_bool().unwrap_or(false);
                let mode = aud["generation_mode"].as_str().unwrap_or("r4g1");
                let lat = aud["total_latency_ms"].as_f64().unwrap_or(0.0);
                let pass_str = if kappa_pass {
                    "\x1b[32m[✓ PASS]\x1b[0m"
                } else {
                    "\x1b[33m[! DRIFT]\x1b[0m"
                };

                println!(
                    "          UOR Address: \x1b[36m{}\x1b[0m | κ: {:.4} {} | mode: \x1b[32m{}\x1b[0m | latency: {:.2}ms",
                    uor_addr, kappa, pass_str, mode, lat
                );
            }
        }
    }
    println!(
        "\x1b[1;36m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n"
    );
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    uor_r4_wasm_router::telemetry::init(cli.verbose);
    if let Err(error) = run(&cli) {
        tracing::error!(%error, "command failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::VecDeque;

    struct FakeChat {
        answers: VecDeque<String>,
        questions: Vec<String>,
    }
    impl FakeChat {
        fn with_answers(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(ToString::to_string).collect(),
                questions: Vec::new(),
            }
        }
    }
    impl Chat for FakeChat {
        fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
            self.questions.push(question.to_owned());
            Ok(ChatAnswer {
                text: self.answers.pop_front().unwrap_or_default(),
                generated_tokens: 1,
                repeated_token_rate: 0.0,
                witness: Default::default(),
                abstention: None,
            })
        }
    }

    /// #839 phase 1 (RF-30): the CLI abstention line carries the typed
    /// outcome, cause, and coverage labels (spec §5 CLI row) alongside the
    /// legacy D4 label, and mints no confidence value in legacy-coverage
    /// mode.
    #[test]
    fn rendered_abstention_carries_the_typed_labels() {
        use uor_r4_wasm_router::chat::ChatAbstention;
        use uor_r4_wasm_router::selective;

        let answer = ChatAnswer {
            text: String::new(),
            generated_tokens: 0,
            repeated_token_rate: 0.0,
            witness: Default::default(),
            abstention: Some(ChatAbstention {
                status: "novel".to_owned(),
                widened: true,
                outcome: selective::STATUS_ABSTENTION,
                cause: selective::CAUSE_DISTRIBUTIONALLY_NOVEL,
                coverage: selective::COVERAGE_DISTRIBUTIONALLY_NOVEL,
            }),
        };
        let line = rendered_answer(&answer);
        assert!(
            line.contains(
                "abstention: cause=distributionally-novel coverage=distributionally-novel \
                 d4=novel, widened once"
            ),
            "the typed record renders every label: {line}"
        );
        assert!(
            !line.contains("confidence="),
            "no confidence value exists in legacy-coverage mode: {line}"
        );
    }

    #[test]
    fn help_definition_is_valid() {
        Cli::command().debug_assert();
        let help = Cli::command().render_long_help().to_string();
        for command in [
            "demo",
            "route",
            "serve",
            "source-free-table",
            "bounded-geometric-generate",
            "r4-softmax-generate",
            "r4-softmax-local-generate",
            "r4-softmax-local-qualify",
            "r4-softmax-trace-student",
            "r4-softmax-trace-state-compile",
            "r4-softmax-trace-state-seal",
            "r4-softmax-trace-state-reveal",
            "r4-softmax-trace-observability",
            "lexical-ingestion-witness",
            "research-tools",
        ] {
            assert!(help.contains(command));
        }
    }

    #[test]
    fn parses_defaults_flags_and_subcommands() {
        let cli = Cli::try_parse_from(["r4"]).unwrap();
        assert_eq!(cli.host, "127.0.0.1");
        assert_eq!(cli.port, 8000);
        assert!(cli.command.is_none());

        let cli =
            Cli::try_parse_from(["r4", "ask", "hello", "world", "--port", "9001", "-vvv"]).unwrap();
        assert_eq!(cli.port, 9001);
        assert_eq!(cli.verbose, 3);
        let Some(Command::Ask(args)) = cli.command else {
            panic!("expected ask")
        };
        assert_eq!(args.question.join(" "), "hello world");
    }

    #[test]
    fn parses_bounded_geometric_generate_command_without_rewriting_prompt_bytes() {
        let cli = Cli::try_parse_from([
            "r4",
            "bounded-geometric-generate",
            "--artifact",
            "/tmp/canonical-route.json",
            "--prompt",
            "active  agile athletes run",
            "--continuation-cap",
            "2",
            "--control",
            "state-disabled",
            "--json",
        ])
        .unwrap();
        let Some(Command::BoundedGeometricGenerate(args)) = cli.command else {
            panic!("expected bounded-geometric-generate")
        };
        assert_eq!(args.artifact, PathBuf::from("/tmp/canonical-route.json"));
        assert_eq!(args.prompt, "active  agile athletes run");
        assert_eq!(args.continuation_cap, 2);
        assert_eq!(args.control, BoundedGeometricControl::StateDisabled);
        assert!(args.json);
    }

    #[test]
    fn parses_source_free_table_command_without_rewriting_prompt_bytes() {
        let cli = Cli::try_parse_from([
            "r4",
            "source-free-table",
            "--corpus",
            "/tmp/articles.jsonl",
            "--prompt",
            "The  United States",
            "--continuation-cap",
            "16",
            "--artifact-out",
            "/tmp/source-free-table.bin",
            "--json",
        ])
        .unwrap();
        let Some(Command::SourceFreeTable(args)) = cli.command else {
            panic!("expected source-free-table")
        };
        assert_eq!(args.corpus, PathBuf::from("/tmp/articles.jsonl"));
        assert_eq!(args.prompt, "The  United States");
        assert_eq!(args.continuation_cap, 16);
        assert_eq!(
            args.artifact_out,
            Some(PathBuf::from("/tmp/source-free-table.bin"))
        );
        assert!(!args.geometric_intervention);
        assert_eq!(args.geometry_overlay_out, None);
        assert!(args.json);

        let cli = Cli::try_parse_from([
            "r4",
            "source-free-table",
            "--corpus",
            "/tmp/articles.jsonl",
            "--prompt",
            "The  United States",
            "--continuation-cap",
            "16",
            "--artifact-out",
            "/tmp/source-free-table.bin",
            "--geometric-intervention",
            "--geometry-overlay-out",
            "/tmp/multiscale-count-radius-r4.bin",
            "--json",
        ])
        .unwrap();
        let Some(Command::SourceFreeTable(args)) = cli.command else {
            panic!("expected source-free-table")
        };
        assert!(args.geometric_intervention);
        assert_eq!(
            args.geometry_overlay_out,
            Some(PathBuf::from("/tmp/multiscale-count-radius-r4.bin"))
        );
        assert!(args.json);
    }

    #[test]
    fn parses_r4_softmax_generate_without_rewriting_prompt_bytes() {
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-generate",
            "--source",
            "/models/smollm2",
            "--prompt",
            "Explain  R4 attention.",
            "--max-tokens",
            "8",
            "--workers",
            "6",
            "--json-output",
            "/tmp/r4-softmax.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxGenerate(args)) = cli.command else {
            panic!("expected r4-softmax-generate")
        };
        assert_eq!(args.source, PathBuf::from("/models/smollm2"));
        assert_eq!(args.prompt, "Explain  R4 attention.");
        assert_eq!(args.max_tokens, 8);
        assert_eq!(args.workers, 6);
        assert_eq!(
            args.json_output,
            Some(PathBuf::from("/tmp/r4-softmax.json"))
        );

        let cli = Cli::try_parse_from(["r4", "r4-softmax-generate", "--prompt", "Use defaults."])
            .unwrap();
        let Some(Command::R4SoftmaxGenerate(args)) = cli.command else {
            panic!("expected r4-softmax-generate")
        };
        assert_eq!(
            args.source,
            PathBuf::from(".uor-models/sources/smollm2-135m-instruct")
        );
        assert_eq!(
            args.max_tokens,
            uor_r4_wasm_router::r4_softmax_reference_generation::DEFAULT_MAX_NEW_TOKENS
        );
        assert_eq!(args.workers, 4);
        assert!(args.json_output.is_none());
    }

    #[test]
    fn parses_r4_softmax_local_generate_strictly_without_rewriting_prompt_bytes() {
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-local-generate",
            "--model",
            "/models/issue-1014",
            "--prompt",
            "Once  upon a time",
            "--max-new-tokens",
            "128",
            "--json-output",
            "/tmp/r4-softmax-local.json",
        ])
        .expect("parse local generator");
        let Some(Command::R4SoftmaxLocalGenerate(args)) = cli.command else {
            panic!("expected r4-softmax-local-generate")
        };
        assert_eq!(args.model, PathBuf::from("/models/issue-1014"));
        assert_eq!(args.prompt, "Once  upon a time");
        assert_eq!(args.max_new_tokens, 128);
        assert!(!args.attention_off);
        assert!(args.seed.is_none());
        assert_eq!(
            args.json_output,
            Some(PathBuf::from("/tmp/r4-softmax-local.json"))
        );

        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-local-generate",
            "--model",
            "/models/issue-1014",
            "--prompt",
            "x",
            "--max-new-tokens",
            "0",
        ])
        .is_err());
        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-local-generate",
            "--source",
            "/models/issue-1014",
            "--prompt",
            "x",
        ])
        .is_err());

        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-local-generate",
            "--model",
            "/models/issue-1014",
            "--prompt",
            "x",
            "--attention-off",
            "--seed",
            "1014",
        ])
        .expect("parse explicit attention-off control");
        let Some(Command::R4SoftmaxLocalGenerate(args)) = cli.command else {
            panic!("expected r4-softmax-local-generate")
        };
        assert!(args.attention_off);
        assert_eq!(args.seed, Some(1014));
    }

    #[test]
    fn parses_r4_softmax_local_qualify_with_explicit_evidence_only() {
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-local-qualify",
            "--model",
            "/research/export",
            "--python-prefix-logits",
            "/research/reveal/python-prefix-logits.json",
            "--reveal-manifest",
            "/research/reveal/reveal-manifest.json",
            "--workers",
            "4",
            "--json-output",
            "/research/reveal/rust-qualification.json",
        ])
        .expect("parse strict local qualifier");
        let Some(Command::R4SoftmaxLocalQualify(args)) = cli.command else {
            panic!("expected r4-softmax-local-qualify")
        };
        assert_eq!(args.model, PathBuf::from("/research/export"));
        assert_eq!(args.workers, 4);
        assert!(args.campaign.is_none());
        assert!(!args.enabled_only);
        assert_eq!(
            args.reveal_manifest,
            Some(PathBuf::from("/research/reveal/reveal-manifest.json"))
        );
        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-local-qualify",
            "--model",
            "/research/export",
            "--python-prefix-logits",
            "/research/prefix.json",
            "--workers",
            "0",
            "--json-output",
            "/research/result.json",
        ])
        .is_err());
        let enabled = Cli::try_parse_from([
            "r4",
            "r4-softmax-local-qualify",
            "--model",
            "/research/export",
            "--python-prefix-logits",
            "/research/dev-prefix.json",
            "--enabled-only",
            "--json-output",
            "/research/enabled-qualification.json",
        ])
        .expect("parse enabled-only continuation qualifier");
        let Some(Command::R4SoftmaxLocalQualify(args)) = enabled.command else {
            panic!("expected r4-softmax-local-qualify")
        };
        assert!(args.campaign.is_none());
        assert!(args.enabled_only);
        assert!(args.reveal_manifest.is_none());

        let capacity = Cli::try_parse_from([
            "r4",
            "r4-softmax-local-qualify",
            "--model",
            "/research/issue-1019/export",
            "--python-prefix-logits",
            "/research/issue-1019/dev-prefix.json",
            "--campaign",
            "issue-1019",
            "--json-output",
            "/research/issue-1019/rust-qualification.json",
        ])
        .expect("parse explicit #1019 capacity qualifier");
        let Some(Command::R4SoftmaxLocalQualify(args)) = capacity.command else {
            panic!("expected r4-softmax-local-qualify")
        };
        assert_eq!(
            args.campaign,
            Some(R4SoftmaxLocalQualificationCampaignArg::Issue1019)
        );
        assert!(!args.enabled_only);
        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-local-qualify",
            "--model",
            "/research/export",
            "--python-prefix-logits",
            "/research/dev-prefix.json",
            "--campaign",
            "issue-1019",
            "--enabled-only",
            "--json-output",
            "/research/result.json",
        ])
        .is_err());
    }

    #[test]
    fn parses_r4_softmax_trace_student_preflight() {
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-student",
            "--source",
            "/models/smollm2",
            "--workers",
            "6",
            "--implementation-revision",
            "0123456789abcdef0123456789abcdef01234567",
            "--preflight-only",
            "--artifact-output",
            "/tmp/student.bin",
            "--trace-output",
            "/tmp/trace.bin",
            "--result-output",
            "/tmp/result.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxTraceStudent(args)) = cli.command else {
            panic!("expected r4-softmax-trace-student")
        };
        assert_eq!(args.source, PathBuf::from("/models/smollm2"));
        assert_eq!(args.workers, 6);
        assert_eq!(
            args.implementation_revision,
            "0123456789abcdef0123456789abcdef01234567"
        );
        assert!(args.preflight_only);
        assert!(!args.reveal);
        assert_eq!(args.artifact_output, PathBuf::from("/tmp/student.bin"));
        assert_eq!(args.trace_output, PathBuf::from("/tmp/trace.bin"));
        assert_eq!(
            args.freeze_output,
            PathBuf::from(".uor-models/research/issue-973/r4-softmax-trace-freeze-v1.json")
        );
        assert_eq!(args.result_output, PathBuf::from("/tmp/result.json"));
    }

    #[test]
    fn parses_three_strict_r4_softmax_trace_state_phases() {
        let revision = "0123456789abcdef0123456789abcdef01234567";
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-compile",
            "--implementation-revision",
            revision,
            "--trace-bundle",
            "/tmp/construction.bin",
            "--predecessor-freeze",
            "/tmp/predecessor.json",
            "--suffix-artifact",
            "/tmp/suffix.bin",
            "--artifact-output",
            "/tmp/state.bin",
            "--freeze-output",
            "/tmp/state-freeze.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxTraceStateCompile(args)) = cli.command else {
            panic!("expected state compile")
        };
        assert_eq!(args.implementation_revision, revision);
        assert_eq!(args.trace_bundle, PathBuf::from("/tmp/construction.bin"));
        assert_eq!(
            args.predecessor_freeze,
            PathBuf::from("/tmp/predecessor.json")
        );
        assert_eq!(args.suffix_artifact, PathBuf::from("/tmp/suffix.bin"));
        assert_eq!(args.artifact_output, PathBuf::from("/tmp/state.bin"));
        assert_eq!(args.freeze_output, PathBuf::from("/tmp/state-freeze.json"));

        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-seal",
            "--artifact",
            "/tmp/state.bin",
            "--freeze",
            "/tmp/state-freeze.json",
            "--causal-input",
            "/tmp/causal.json",
            "--seal-output",
            "/tmp/seal.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxTraceStateSeal(args)) = cli.command else {
            panic!("expected state seal")
        };
        assert_eq!(args.artifact, PathBuf::from("/tmp/state.bin"));
        assert_eq!(args.freeze, PathBuf::from("/tmp/state-freeze.json"));
        assert_eq!(args.causal_input, PathBuf::from("/tmp/causal.json"));
        assert_eq!(args.seal_output, PathBuf::from("/tmp/seal.json"));

        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-reveal",
            "--seal",
            "/tmp/seal.json",
            "--judge",
            "/tmp/judge.json",
            "--result-output",
            "/tmp/result.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxTraceStateReveal(args)) = cli.command else {
            panic!("expected state reveal")
        };
        assert_eq!(args.seal, PathBuf::from("/tmp/seal.json"));
        assert_eq!(args.judge, PathBuf::from("/tmp/judge.json"));
        assert_eq!(args.result_output, PathBuf::from("/tmp/result.json"));

        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-compile",
            "--implementation-revision",
            revision,
            "--source",
            "/models/forbidden",
        ])
        .is_err());
        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-seal",
            "--judge",
            "/tmp/forbidden.json",
        ])
        .is_err());
        assert!(Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-state-reveal",
            "--artifact",
            "/tmp/forbidden.bin",
        ])
        .is_err());
    }

    #[test]
    fn parses_strict_r4_softmax_trace_observability_command() {
        let revision = "0123456789abcdef0123456789abcdef01234567";
        let cli = Cli::try_parse_from([
            "r4",
            "r4-softmax-trace-observability",
            "--implementation-revision",
            revision,
            "--trace-bundle",
            "/tmp/construction.bin",
            "--predecessor-freeze",
            "/tmp/predecessor.json",
            "--result-output",
            "/tmp/result.json",
        ])
        .unwrap();
        let Some(Command::R4SoftmaxTraceObservability(args)) = cli.command else {
            panic!("expected R4 softmax trace observability")
        };
        assert_eq!(args.implementation_revision, revision);
        assert_eq!(args.trace_bundle, PathBuf::from("/tmp/construction.bin"));
        assert_eq!(
            args.predecessor_freeze,
            PathBuf::from("/tmp/predecessor.json")
        );
        assert_eq!(args.result_output, PathBuf::from("/tmp/result.json"));

        for forbidden in ["--source", "--judge", "--fold", "--state-artifact"] {
            assert!(Cli::try_parse_from([
                "r4",
                "r4-softmax-trace-observability",
                "--implementation-revision",
                revision,
                forbidden,
                "/tmp/forbidden",
            ])
            .is_err());
        }
    }

    #[test]
    fn serve_keeps_r4_softmax_reference_disabled_unless_explicitly_enabled() {
        let cli = Cli::try_parse_from(["r4", "serve"]).unwrap();
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve")
        };
        assert!(!args.enable_r4_softmax_reference);
        assert!(args.r4_softmax_source.is_none());
        assert!(args.r4_softmax_workers.is_none());

        let cli = Cli::try_parse_from([
            "r4",
            "serve",
            "--enable-r4-softmax-reference",
            "--r4-softmax-source",
            "/models/smollm2",
            "--r4-softmax-workers",
            "8",
        ])
        .unwrap();
        let Some(Command::Serve(args)) = cli.command else {
            panic!("expected serve")
        };
        assert!(args.enable_r4_softmax_reference);
        assert_eq!(
            args.r4_softmax_source,
            Some(PathBuf::from("/models/smollm2"))
        );
        assert_eq!(args.r4_softmax_workers, Some(8));

        assert!(Cli::try_parse_from(["r4", "serve", "--r4-softmax-workers", "8"]).is_err());
    }

    #[test]
    fn bounded_geometric_generate_labels_every_typed_stop() {
        assert_eq!(
            local_generation_stop_label(LocalGenerationStopReason::Abstained { tie: false }),
            "abstained(tie=false)"
        );
        assert_eq!(
            local_generation_stop_label(LocalGenerationStopReason::Abstained { tie: true }),
            "abstained(tie=true)"
        );
        assert_eq!(
            local_generation_stop_label(LocalGenerationStopReason::TerminalPunctuation),
            "terminal_punctuation"
        );
        assert_eq!(
            local_generation_stop_label(LocalGenerationStopReason::ShortCycle { period: 3 }),
            "short_cycle(period=3)"
        );
        assert_eq!(
            local_generation_stop_label(LocalGenerationStopReason::ContinuationCap),
            "continuation_cap"
        );
    }

    #[test]
    fn parses_compile_command() {
        let cli = Cli::try_parse_from([
            "r4",
            "compile",
            "--source",
            "/models/local",
            "--tokenizer-family",
            "hf-byte-bpe",
            "--tokenizer-version",
            "1",
        ])
        .unwrap();
        let Some(Command::Compile(args)) = cli.command else {
            panic!("expected compile")
        };
        assert_eq!(args.source, Some(PathBuf::from("/models/local")));
        assert_eq!(args.tokenizer_family.as_deref(), Some("hf-byte-bpe"));
        assert_eq!(args.tokenizer_version, Some(1));
        assert_eq!(args.target, 20_000);
        assert_eq!(args.sequence_length, 128);
    }

    #[test]
    fn parses_evaluate_report_command() {
        let cli = Cli::try_parse_from([
            "r4",
            "evaluate-report",
            "--source",
            "/models/source",
            "--compiled",
            "/models/compiled",
            "--report",
            "/tmp/report.json",
            "--sequence-length",
            "256",
            "--tokenizer-family",
            "sentencepiece-unigram",
            "--tokenizer-version",
            "7",
        ])
        .unwrap();
        let Some(Command::EvaluateReport(args)) = cli.command else {
            panic!("expected evaluate-report")
        };
        assert_eq!(args.source, Some(PathBuf::from("/models/source")));
        assert_eq!(args.compiled, Some(PathBuf::from("/models/compiled")));
        assert_eq!(args.report, Some(PathBuf::from("/tmp/report.json")));
        assert_eq!(args.sequence_length, 256);
        assert_eq!(
            args.tokenizer_family.as_deref(),
            Some("sentencepiece-unigram")
        );
        assert_eq!(args.tokenizer_version, Some(7));
    }

    #[test]
    fn parses_geometric_decoder_spike_command() {
        let cli = Cli::try_parse_from([
            "r4",
            "geometric-decoder-spike",
            "--source",
            "/models/source",
            "--control-report",
            "/tmp/control.json",
            "--workers",
            "3",
        ])
        .unwrap();
        let Some(Command::GeometricDecoderSpike(args)) = cli.command else {
            panic!("expected geometric-decoder-spike")
        };
        assert_eq!(args.source, PathBuf::from("/models/source"));
        assert_eq!(
            args.source_revision,
            uor_r4_wasm_router::geometric_decoder::PINNED_SOURCE_REVISION
        );
        assert_eq!(
            args.control_report,
            Some(PathBuf::from("/tmp/control.json"))
        );
        assert_eq!(args.workers, 3);
    }

    #[test]
    fn parses_geometric_mixer_qualification_preflight() {
        let cli = Cli::try_parse_from([
            "r4",
            "geometric-mixer-qualification",
            "--preflight-only",
            "--seed",
            "951",
        ])
        .unwrap();
        let Some(Command::GeometricMixerQualification(args)) = cli.command else {
            panic!("expected geometric-mixer-qualification")
        };
        assert!(args.preflight_only);
        assert!(args.source.is_none());
        assert_eq!(args.seed, 951);
        assert_eq!(args.steps_per_round, 80);
    }

    #[test]
    fn tokenizer_selection_is_an_atomic_non_recorded_pair() {
        for command in ["compile", "evaluate-report"] {
            assert!(Cli::try_parse_from([
                "r4",
                command,
                "--source",
                "/models/source",
                "--tokenizer-family",
                "hf-byte-bpe",
            ])
            .is_err());
            assert!(Cli::try_parse_from([
                "r4",
                command,
                "--source",
                "/models/source",
                "--tokenizer-version",
                "1",
            ])
            .is_err());
        }

        assert!(Cli::try_parse_from([
            "r4",
            "compile",
            "--corpus-meta",
            "/corpus/meta",
            "--corpus-recs",
            "/corpus/records",
            "--vocab-size",
            "32000",
            "--tokenizer-family",
            "hf-byte-bpe",
            "--tokenizer-version",
            "1",
        ])
        .is_err());

        let available = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1);
        assert_eq!(
            super::resolve_r4_softmax_reference_workers(None)
                .expect("bounded default")
                .get(),
            super::server::R4_SOFTMAX_REFERENCE_HTTP_DEFAULT_WORKERS.min(available)
        );
        assert!(super::resolve_r4_softmax_reference_workers(Some(0)).is_err());
        assert!(super::resolve_r4_softmax_reference_workers(Some(available + 1)).is_err());
    }

    #[test]
    fn one_shot_writes_only_the_answer() {
        let mut chat = FakeChat::with_answers(&["Because of Rayleigh scattering."]);
        let mut output = Vec::new();
        answer_once(&mut chat, "Why is the sky blue?", &mut output).unwrap();
        assert_eq!(chat.questions, ["Why is the sky blue?"]);
        assert_eq!(output, b"Because of Rayleigh scattering.\n");
    }

    #[test]
    fn repl_skips_blanks_retains_turns_and_exits() {
        let mut chat = FakeChat::with_answers(&["first", "second"]);
        let mut input = io::Cursor::new("\nhello\nnext\nexit\nignored\n");
        let mut output = Vec::new();
        interactive_chat(&mut chat, &mut input, &mut output).unwrap();
        assert_eq!(chat.questions, ["hello", "next"]);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("r4> first"));
        assert!(output.contains("r4> second"));
        assert!(!output.contains("ignored"));
    }

    #[test]
    fn deployed_quality_projection_is_ceil_scaled_and_fail_closed() {
        assert_eq!(
            super::projected_census_millis(500, 6_000, 72_130),
            Some(6_011)
        );
        assert_eq!(super::projected_census_millis(0, 6_000, 72_130), Some(13));
        assert_eq!(super::projected_census_millis(1, 0, 72_130), None);
        assert_eq!(super::projected_census_millis(1, 7, 6), None);
    }

    #[test]
    fn deployed_quality_funnel_extends_only_an_inconclusive_first_stage() {
        use uor_r4_api::SampleDecisionKind;

        assert_eq!(super::initial_non_census_target(72_130), Some(6_000));
        assert_eq!(super::initial_non_census_target(6_000), Some(5_999));
        assert_eq!(super::initial_non_census_target(2), Some(1));
        assert_eq!(super::initial_non_census_target(1), None);

        assert_eq!(
            super::deployed_quality_next_phase(SampleDecisionKind::Proceed, 6_000, 72_130),
            super::DeployedQualityNextPhase::Census
        );
        assert_eq!(
            super::deployed_quality_next_phase(SampleDecisionKind::Stop, 6_000, 72_130),
            super::DeployedQualityNextPhase::Stop
        );
        assert_eq!(
            super::deployed_quality_next_phase(SampleDecisionKind::Inconclusive, 6_000, 72_130,),
            super::DeployedQualityNextPhase::ExtendedSample { positions: 18_000 }
        );
        assert_eq!(
            super::deployed_quality_next_phase(SampleDecisionKind::Inconclusive, 6_000, 10_000,),
            super::DeployedQualityNextPhase::ExtendedSample { positions: 9_999 },
            "the extension stays non-census for a smaller population"
        );
        assert_eq!(
            super::deployed_quality_next_phase(SampleDecisionKind::Inconclusive, 9_999, 10_000,),
            super::DeployedQualityNextPhase::Census,
            "the maximal non-census extension can authorize only the census"
        );
        assert_eq!(
            super::deployed_quality_next_phase(
                SampleDecisionKind::Inconclusive,
                uor_r4_api::serving_eval::EXTENDED_SAMPLE_TARGET,
                72_130,
            ),
            super::DeployedQualityNextPhase::Census
        );
    }

    #[test]
    fn deployed_quality_preflight_failure_is_durable_and_append_only() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-deployed-quality-preflight-terminal-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create empty staged bundle");
        let args = DeployedQualityArgs {
            bundle: root.clone(),
            compiler_revision: "not-a-revision".to_owned(),
            mode: DeployedQualityMode::Sample,
            positions: uor_r4_api::serving_eval::SAMPLE_TARGET,
            workers: Some(8),
            probe_budget_secs: 120,
            eval_budget_secs: 3_600,
            cross_surface_evidence: None,
        };

        let error = deployed_quality_command(&args).expect_err("invalid revision must fail");
        assert!(error
            .to_string()
            .contains("--compiler-revision must be a full 40-character"));
        let path = root.join(DEPLOYED_QUALITY_INVOCATION_TERMINAL_PATH);
        let first_bytes = std::fs::read(&path).expect("read durable invocation terminal");
        let rows: Vec<serde_json::Value> = String::from_utf8(first_bytes.clone())
            .expect("terminal is UTF-8")
            .lines()
            .map(|line| serde_json::from_str(line).expect("terminal row is JSON"))
            .collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["event"], "started");
        assert_eq!(rows[0]["semantic_admission_input"], false);
        assert_eq!(rows[0]["workers"], 8);
        assert_eq!(rows[1]["event"], "terminal");
        assert_eq!(rows[1]["outcome"], "failed");
        assert!(rows[1]["reason"]
            .as_str()
            .expect("failure reason")
            .contains("--compiler-revision"));
        assert!(rows[1]["elapsed_millis"].is_u64());

        let reuse_error = deployed_quality_command(&args)
            .expect_err("a staged bundle accepts exactly one invocation");
        assert!(reuse_error.to_string().contains("create append-only"));
        assert_eq!(
            std::fs::read(&path).expect("read unchanged invocation terminal"),
            first_bytes
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dropped_deployed_quality_invocation_records_interruption() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-deployed-quality-drop-terminal-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create empty staged bundle");
        let args = DeployedQualityArgs {
            bundle: root.clone(),
            compiler_revision: "1".repeat(40),
            mode: DeployedQualityMode::Full,
            positions: uor_r4_api::serving_eval::SAMPLE_TARGET,
            workers: None,
            probe_budget_secs: 120,
            eval_budget_secs: 3_600,
            cross_surface_evidence: None,
        };
        drop(DeployedQualityInvocationTerminal::begin(&args).expect("start invocation terminal"));

        let rows: Vec<serde_json::Value> =
            std::fs::read_to_string(root.join(DEPLOYED_QUALITY_INVOCATION_TERMINAL_PATH))
                .expect("read interrupted invocation terminal")
                .lines()
                .map(|line| serde_json::from_str(line).expect("terminal row is JSON"))
                .collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["event"], "started");
        assert_eq!(rows[1]["event"], "terminal");
        assert_eq!(rows[1]["outcome"], "interrupted");
        assert_eq!(rows[1]["worker_source"], "available-parallelism");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cross_surface_mismatch_terminal_is_durable_and_fail_closed() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cross-surface-terminal-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("graph")).expect("create terminal fixture root");
        let evidence = uor_r4_api::CrossSurfaceParityEvidence {
            schema: uor_r4_api::CROSS_SURFACE_PARITY_EVIDENCE_SCHEMA.to_owned(),
            graph_cid: format!("blake3:{}", "1".repeat(64)),
            signature_artifact_cid: format!("blake3:{}", "2".repeat(64)),
            tokenizer_cid: None,
            score_report_cid: None,
            checks: 1,
            mismatches: 1,
            records: Vec::new(),
        };
        let evidence_bytes = evidence
            .deterministic_json_bytes()
            .expect("serialize planted mismatch evidence");
        let path = super::write_cross_surface_terminal(
            &root,
            17,
            23,
            &evidence,
            &evidence_bytes,
            "stop-mismatch",
            Some("planted candidate mismatch"),
        )
        .expect("persist mismatch terminal");
        let bytes = std::fs::read(&path).expect("read mismatch terminal");
        let terminal: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse mismatch terminal");
        assert_eq!(terminal["outcome"], "stop-mismatch");
        assert_eq!(terminal["checks"], 1);
        assert_eq!(terminal["mismatches"], 1);
        assert_eq!(
            super::write_cross_surface_terminal(
                &root,
                17,
                23,
                &evidence,
                &evidence_bytes,
                "stop-mismatch",
                Some("planted candidate mismatch"),
            )
            .expect("identical terminal is idempotent"),
            path
        );

        let mut different = evidence;
        different.mismatches = 0;
        assert!(
            super::write_cross_surface_terminal(
                &root,
                17,
                23,
                &different,
                &evidence_bytes,
                "pass",
                None,
            )
            .is_err(),
            "a terminal from another verdict cannot be overwritten"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
