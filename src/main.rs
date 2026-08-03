use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use uor_r4_graph_cli as transformerless_command;
use uor_r4_wasm_router::chat::{ChatAnswer, ChatEngine, ChatError};
use uor_r4_wasm_router::model::{
    default_model_reference, download_source, ModelCapability, ModelError, ModelManifest,
    ModelStore, QualityAttestation, SourceDownload,
};
use uor_r4_wasm_router::server::{self, ServerConfig};
use uor_r4_wasm_router::tless_uor;

/// R⁴ local AI: compile, manage, ask, chat, benchmark, or serve.
#[derive(Parser, Debug)]
#[command(name = "r4", version, about, long_about = None)]
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
    /// Run the HTTP server (the default).
    Serve,
    /// Ask one question using the local transformerless library directly.
    Ask(AskArgs),
    /// Start an interactive, stateful local chat.
    Chat(ChatArgs),
    /// Connect to a local uor-r4 server endpoint as a remote interactive client.
    Client(ClientArgs),
    /// View or export UOR Q&A audit traces and geometry metrics.
    Audit(AuditArgs),
    /// Compile a recorded corpus or a local/pinned Hugging Face model into an R⁴ bundle.
    Compile(CompileArgs),
    /// Download pinned open weights for offline compilation.
    Download(DownloadArgs),
    /// Import an evaluated compiled bundle into the UOR CID store.
    Import(ImportArgs),
    /// Evaluate an HF-compiled bundle and emit an instruction-quality report.
    EvaluateReport(EvaluateReportArgs),
    /// Print legacy proof-workflow prerequisites.
    Setup,
    /// Generate the legacy resumable teacher corpus.
    Gen {
        #[arg(default_value_t = 300)]
        seconds: u64,
        #[arg(default_value_t = 150_000)]
        target: usize,
    },
    /// Build the legacy graded store.
    Store,
    /// Run the transformerless certificate workflow.
    Certify,
    /// Run the measured local comparison.
    Compare,
    /// Print the recorded comparison certificate.
    CompareReport,
    /// Run the transformerless scenario suite.
    Scenarios,
    /// Print the legacy teacher checkpoint κ.
    TeacherKappa,
    /// Forward a command to the legacy transformerless toolset, e.g.
    /// `r4 transformerless convert-r4g1 --artifacts <TLA> --store <TLS1>
    /// [--calibration <hamming_calibration.json>] --out <R4G1>`.
    Transformerless {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run the new R4G1 multiresolution graph compiler pipeline.
    GraphCompile {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = false, num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Sample observation records using a teacher oracle.
    GraphObserve {
        /// Subcommand and arguments forwarded verbatim.
        #[arg(required = false, num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Args, Debug)]
struct AskArgs {
    /// CID manifest name/CID, or a locally compiled bundle name.
    #[arg(long, env = "TLESS_MODEL")]
    model: Option<String>,
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
}

#[derive(Args, Debug)]
struct ClientArgs {
    /// Remote HTTP server URL [default: http://127.0.0.1:8000/v1].
    #[arg(long, default_value = "http://127.0.0.1:8000/v1")]
    remote: String,
    /// Model name to send in chat completions payload.
    #[arg(long, default_value = "uor-r4")]
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
    /// Enable experimental R4 Spin(4) softmax-free attention during compilation.
    #[arg(long, default_value_t = false)]
    r4_attention: bool,
    /// Force exact scalar CPU math instead of Apple Accelerate / SIMD hardware matrix acceleration.
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
    #[arg(long)]
    evaluation_report: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    instruction_eval_passed: bool,
    #[arg(long, default_value_t = 0.0)]
    grounded_answer_rate: f32,
    #[arg(long, default_value_t = 1.0)]
    repetition_rate: f32,
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

fn answer_once(
    chat: &mut impl Chat,
    question: &str,
    output: &mut impl Write,
) -> Result<(), RunError> {
    writeln!(output, "{}", chat.ask(question)?.text)?;
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
            Ok(answer) => writeln!(output, "r4> {}\n", answer.text)?,
            Err(error) => tracing::error!(%error, "chat response failed"),
        }
    }
    Ok(())
}

fn build_chat_engine(model: Option<&str>) -> Result<ChatEngine, ChatError> {
    ChatEngine::builder()
        .model(model.map_or_else(default_model_reference, ToOwned::to_owned))
        .build()
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
        return transformerless_command::run(&values).map_err(RunError::Command);
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
    transformerless_command::compile_hugging_face(&values).map_err(RunError::Command)
}

fn download(args: &DownloadArgs) -> Result<(), RunError> {
    let path = download_source(&SourceDownload {
        repository: args.repository.clone(),
        revision: args.revision.clone(),
        name: args.name.clone(),
        output: args.output.clone(),
    })?;
    println!("{}", path.display());
    Ok(())
}

fn import(args: &ImportArgs) -> Result<(), RunError> {
    let model_store = ModelStore::from_env();
    let artifacts = model_store.put(&std::fs::read(&args.artifacts)?)?;
    let store = model_store.put(&std::fs::read(&args.store)?)?;
    let tokenizer = model_store.put(&std::fs::read(&args.tokenizer)?)?;
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
        quality: QualityAttestation {
            instruction_eval_passed: args.instruction_eval_passed,
            grounded_answer_rate: args.grounded_answer_rate,
            repetition_rate: args.repetition_rate,
        },
    };
    manifest.validate_for_chat().or_else(|error| {
        (manifest.capability == ModelCapability::Continuation)
            .then_some(())
            .ok_or(error)
    })?;
    println!("{}", model_store.write_manifest(&manifest)?);
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
    values.extend([
        "--sequence-length".to_owned(),
        args.sequence_length.to_string(),
    ]);
    run_core("evaluate-report", &values)
}

fn run_core(name: &str, arguments: &[String]) -> Result<(), RunError> {
    let mut values = vec![name.to_owned()];
    values.extend_from_slice(arguments);
    transformerless_command::run(&values).map_err(RunError::Command)
}

/// Default location of the reference teacher checkpoint used by `certify`/`compare`.
const DEFAULT_REFERENCE_CHECKPOINT: &str = "/tmp/ref/out/model.bin";

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
        Some(Command::Ask(args)) => {
            let mut chat = build_chat_engine(args.model.as_deref())?;
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
                    args.model.as_deref().unwrap_or("uor-r4"),
                    &mut io::stdin().lock(),
                    &mut io::stdout().lock(),
                )?;
                Ok(())
            } else {
                let mut chat = build_chat_engine(args.model.as_deref())?;
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
        Some(Command::EvaluateReport(args)) => evaluate_report(args),
        Some(Command::Setup) => run_core("setup", &[]),
        Some(Command::Gen { seconds, target }) => {
            run_core("gen", &[seconds.to_string(), target.to_string()])
        }
        Some(Command::Store) => run_core("store", &[]),
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
            transformerless_command::run(args).map_err(RunError::Command)
        }
        Some(Command::GraphCompile { args }) => {
            uor_r4_graph_compiler::compile(args).map_err(RunError::Command)
        }
        Some(Command::GraphObserve { args }) => {
            uor_r4_graph_compiler::observe(args).map_err(RunError::Command)
        }
        Some(Command::Audit(args)) => audit_command(&args.log_file),
        Some(Command::Serve) | None => {
            server::run_server(Arc::new(cli.server_config()));
            Ok(())
        }
    }
}

/// The certify C row (issue #280): held-out evaluation of the serving
/// surface — `R4Engine` + `score.r4g1` + the D4 status policy — on the
/// first loadable compiled bundle (`R4_CERTIFY_SERVING_BUNDLE` selects
/// one explicitly). `R4_CERTIFY_C_ONLY=1` invokes this path without loading
/// the reference teacher or running the unrelated full certificate. Prints
/// a measured row or an explicit recorded skip;
/// never fails the certify run. The retired scaffold row's history is
/// recorded on issue #280.
fn certify_serving_row() {
    use uor_r4_api::serving_eval::{
        self, ServingBundle, ServingEvalBudgets, ServingEvalOutcome, ServingEvalSkip,
    };
    let bundles = match std::env::var("R4_CERTIFY_SERVING_BUNDLE") {
        Ok(root) => match ServingBundle::discover(std::path::Path::new(&root)) {
            Some(bundle) => vec![bundle],
            None => {
                println!(
                    "C serving: SKIPPED — R4_CERTIFY_SERVING_BUNDLE={root} is not a compiled serving bundle (needs score.r4g1, tless_artifacts.bin, corpus.meta, corpus.records)"
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
        "C serving readiness probe: {} positions with accuracy spot-check, wall-clock budgets probe {}s / eval {}s (R4_CERTIFY_R4G1_BUDGET_SECS / R4_CERTIFY_R4G1_EVAL_BUDGET_SECS override)",
        serving_eval::PROBE_POSITIONS,
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
        let mut progress = |done: usize, total: usize, secs: u64| {
            println!(
                "progress: C serving eval {done}/{total} ({}%, {secs}s)",
                100 * done / total
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
                    "C serving (R4Engine + score.r4g1 + D4 policy, 80/20 story split, deterministic subsample n={}) bundle {}: served {} ({:.1}%; exact {} ({} ngram), graph {}, novel {}; {} widened) | abstained {} (exact {}, graph {}, novel {}, contradictory {}) | on served: top1 {:.1}% | agreement {:.1}% | overall top1 {:.1}% | probe: {}/{} served, {} hits",
                    row.sample_n,
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
                    pct(row.top1_served, row.served),
                    pct(row.agree_served, row.served),
                    pct(row.top1_served, row.sample_n as u64),
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
        println!(
            "    Run the interactive client ('./uor-r4-cli' or 'r4 client') and ask questions first.\n"
        );
        return Ok(());
    }
    let content = std::fs::read_to_string(log_file)?;
    let records: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| RunError::Command(format!("Failed to parse audit log: {}", e)))?;

    println!("\n\x1b[1;36m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m");
    println!("\x1b[1;36m│  R⁴ UOR Auditability & Tracing Log Inspector                                │\x1b[0m");
    println!("\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m");
    println!("  Audit Log Path: \x1b[1m{}\x1b[0m", log_file.display());

    if let Some(arr) = records.as_array() {
        println!("  Total Audited Turns: \x1b[32m{}\x1b[0m", arr.len());
        for (idx, item) in arr.iter().enumerate() {
            println!("\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m");
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

                println!("          UOR Address: \x1b[36m{}\x1b[0m | κ: {:.4} {} | mode: \x1b[32m{}\x1b[0m | latency: {:.2}ms", uor_addr, kappa, pass_str, mode, lat);
            }
        }
    }
    println!("\x1b[1;36m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n");
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
            })
        }
    }

    #[test]
    fn help_definition_is_valid() {
        Cli::command().debug_assert();
        let help = Cli::command().render_long_help().to_string();
        for command in [
            "serve",
            "ask",
            "chat",
            "compile",
            "download",
            "import",
            "evaluate-report",
            "compare",
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
    fn parses_compile_command() {
        let cli = Cli::try_parse_from(["r4", "compile", "--source", "/models/local"]).unwrap();
        let Some(Command::Compile(args)) = cli.command else {
            panic!("expected compile")
        };
        assert_eq!(args.source, Some(PathBuf::from("/models/local")));
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
        ])
        .unwrap();
        let Some(Command::EvaluateReport(args)) = cli.command else {
            panic!("expected evaluate-report")
        };
        assert_eq!(args.source, Some(PathBuf::from("/models/source")));
        assert_eq!(args.compiled, Some(PathBuf::from("/models/compiled")));
        assert_eq!(args.report, Some(PathBuf::from("/tmp/report.json")));
        assert_eq!(args.sequence_length, 256);
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
}
