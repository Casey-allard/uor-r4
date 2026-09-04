//! Native model workflow. Host I/O and fitting are outside the integer kernel.
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use uor_r4_core::native_geometric::{Config, Control, Document, Model, Trainer};

#[derive(Debug, Args)]
pub struct GeometricArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Prepare disjoint open-development JSONL documents in Rust.
    Prepare(PrepareArgs),
    /// Fit native geometric score tables and save a resumable checkpoint.
    Train(TrainArgs),
    /// Learn geometric readout gates on separate construction-fit documents.
    FitReadout(FitReadoutArgs),
    /// Learn a bounded read from prime-addressed retained token values.
    FitMemory(FitMemoryArgs),
    /// Generate from a fitted native artifact with no teacher or source corpus.
    Generate(GenerateArgs),
    /// Compare the full geometry and matched controls on separate documents.
    Evaluate(EvaluateArgs),
    /// Keep a local conversation in the same native model context.
    Chat(ChatArgs),
    /// Open a local workbench using the same native artifact and isolated sessions.
    Serve {
        #[arg(long)]
        model: PathBuf,
        #[arg(long, default_value_t = 8087)]
        port: u16,
        #[arg(long)]
        session_directory: Option<PathBuf>,
    },
    /// Inspect artifact training provenance and configured capacity.
    Inspect {
        #[arg(long)]
        model: PathBuf,
    },
}

#[derive(Debug, Args)]
struct PrepareArgs {
    #[command(flatten)]
    corpus: CorpusArgs,
    #[arg(long)]
    output_directory: PathBuf,
    /// Place every Nth unique document into development; requires at least 2.
    #[arg(long, default_value_t = 5)]
    development_every: usize,
    /// Reserve the document before each development document for readout fitting.
    #[arg(long)]
    readout_split: bool,
}

#[derive(Debug, Args)]
struct FitReadoutArgs {
    #[arg(long)]
    model: PathBuf,
    #[command(flatten)]
    corpus: CorpusArgs,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 4096)]
    max_positions: usize,
    #[arg(long, default_value_t = 8)]
    epochs: usize,
    #[arg(long, default_value_t = 256)]
    max_queries: usize,
    #[arg(long)]
    report: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct FitMemoryArgs {
    #[arg(long)]
    model: PathBuf,
    #[command(flatten)]
    corpus: CorpusArgs,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 8)]
    query_tokens: usize,
    #[arg(long, default_value_t = 4)]
    source_offsets: usize,
    #[arg(long, default_value_t = 4)]
    postings_per_address: usize,
    #[arg(long, default_value_t = 128)]
    candidates: usize,
    #[arg(long, default_value_t = 4096)]
    max_positions: usize,
    #[arg(long, default_value_t = 8)]
    epochs: usize,
    #[arg(long, default_value_t = 65_536)]
    max_features: usize,
    /// Experimental leading-whitespace word equivalence for memory cues.
    #[arg(long)]
    word_cues: bool,
    #[arg(long)]
    report: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct CorpusArgs {
    /// UTF-8 text or JSONL records with id and text fields; may be repeated.
    #[arg(long, required = true)]
    input: Vec<PathBuf>,
    /// Read at most this many input bytes in total; prefix truncation is reported.
    #[arg(long, default_value_t = 16_777_216)]
    max_input_bytes: usize,
    /// Maximum bytes per plain-text training document (UTF-8 boundaries preserved).
    #[arg(long, default_value_t = 8192)]
    document_bytes: usize,
}

#[derive(Debug, Args)]
struct TrainArgs {
    #[command(flatten)]
    corpus: CorpusArgs,
    #[arg(long)]
    model: PathBuf,
    #[arg(long)]
    checkpoint: PathBuf,
    /// Resume this checkpoint; the same ordered construction documents are required.
    #[arg(long)]
    resume: bool,
    #[arg(long, default_value_t = 1)]
    epochs: usize,
    #[arg(long, default_value_t = 128)]
    context: usize,
    #[arg(long, default_value_t = 32)]
    candidates: usize,
    #[arg(long, default_value_t = 4096)]
    lexical_pieces: usize,
    #[arg(long, default_value_t = 65_536)]
    rows: usize,
    #[arg(long, default_value_t = 500_000)]
    associations: usize,
    /// Cumulative elapsed budget across this training checkpoint and its resumes.
    #[arg(long, default_value_t = 1200)]
    max_seconds: u64,
    #[arg(long, default_value_t = 4096)]
    max_rss_mib: u64,
    /// Combined maximum checkpoint and model bytes; replacement is atomic per file.
    #[arg(long, default_value_t = 536_870_912)]
    max_output_bytes: usize,
    #[arg(long, default_value_t = 32)]
    checkpoint_every: usize,
    #[arg(long)]
    report: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ControlArg {
    Full,
    GeometryDisabled,
    ZetaDisabled,
    H4Disabled,
    OrientationDisabled,
    PairedDisabled,
    RadialDisabled,
    HeatmapDisabled,
    MemoryDisabled,
}
impl From<ControlArg> for Control {
    fn from(value: ControlArg) -> Self {
        match value {
            ControlArg::Full => Self::Full,
            ControlArg::GeometryDisabled => Self::GeometryDisabled,
            ControlArg::ZetaDisabled => Self::ZetaDisabled,
            ControlArg::H4Disabled => Self::H4Disabled,
            ControlArg::OrientationDisabled => Self::OrientationDisabled,
            ControlArg::PairedDisabled => Self::PairedDisabled,
            ControlArg::RadialDisabled => Self::RadialDisabled,
            ControlArg::HeatmapDisabled => Self::HeatmapDisabled,
            ControlArg::MemoryDisabled => Self::MemoryDisabled,
        }
    }
}

#[derive(Debug, Args)]
struct GenerateArgs {
    #[arg(long)]
    model: PathBuf,
    #[arg(long)]
    prompt: String,
    #[arg(long, default_value_t = 64)]
    max_tokens: usize,
    #[arg(long, value_enum, default_value = "full")]
    control: ControlArg,
    #[arg(long)]
    json: bool,
}
#[derive(Debug, Args)]
struct EvaluateArgs {
    #[arg(long)]
    model: PathBuf,
    #[command(flatten)]
    corpus: CorpusArgs,
    #[arg(long)]
    report: Option<PathBuf>,
}
#[derive(Debug, Args)]
struct ChatArgs {
    #[arg(long)]
    model: PathBuf,
    #[arg(long, default_value_t = 64)]
    max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointHeader {
    schema: String,
    next_document: usize,
    completed_epochs: usize,
    elapsed_ms: u64,
    documents: Vec<(String, String)>,
}

pub fn run(args: &GeometricArgs) -> Result<(), String> {
    match &args.command {
        Command::Prepare(a) => prepare(a),
        Command::Train(a) => train(a),
        Command::FitReadout(a) => fit_readout(a),
        Command::FitMemory(a) => fit_memory(a),
        Command::Generate(a) => {
            let model = load_model(&a.model)?;
            let result = model
                .generate(&a.prompt, a.max_tokens, a.control.into())
                .map_err(err)?;
            if a.json {
                let mut report = serde_json::to_value(&result).map_err(err)?;
                report["artifact_cid"] = serde_json::json!(model.artifact_cid());
                report["uor_model_address"] = serde_json::json!(model.uor_model_address());
                report["readout_version"] = serde_json::json!(model.readout_version());
                report["memory_read_version"] = serde_json::json!(model.memory_read_version());
                print_json(&report)
            } else {
                println!("{}", result.text);
                Ok(())
            }
        }
        Command::Evaluate(a) => {
            let mut inputs: Vec<&Path> = a.corpus.input.iter().map(PathBuf::as_path).collect();
            inputs.push(&a.model);
            distinct_outputs(
                &inputs,
                &a.report.iter().map(PathBuf::as_path).collect::<Vec<_>>(),
            )?;
            let start = Instant::now();
            let model = load_model(&a.model)?;
            let (documents, source) = read_corpus(&a.corpus)?;
            let mut controls = Vec::new();
            for control in [
                Control::Full,
                Control::GeometryDisabled,
                Control::ZetaDisabled,
                Control::H4Disabled,
                Control::OrientationDisabled,
                Control::PairedDisabled,
                Control::RadialDisabled,
                Control::HeatmapDisabled,
                Control::MemoryDisabled,
            ] {
                let result = model.evaluate(&documents, control).map_err(err)?;
                controls.push(serde_json::json!({"control": control, "result": result}));
            }
            let report = serde_json::json!({"schema":"uor-r4.native-geometric-evaluation/1",
                "artifact_cid":model.artifact_cid(),"uor_model_address":model.uor_model_address(),
                "config":model.config(),"readout_version":model.readout_version(),
                "memory_read_version":model.memory_read_version(),
                "source":source,"controls":controls,"elapsed_ms":start.elapsed().as_millis(),
                "interpretation":"Held-out next-piece prediction; not a conversation or coding qualification."});
            write_report(a.report.as_deref(), &report)
        }
        Command::Chat(a) => chat(a),
        Command::Serve {
            model,
            port,
            session_directory,
        } => crate::native_geometric_service::serve(model, *port, session_directory.as_deref()),
        Command::Inspect { model } => {
            let m = load_model(model)?;
            print_json(
                &serde_json::json!({"config":m.config(),"training":m.training(),
                "artifact_cid":m.artifact_cid(),"uor_model_address":m.uor_model_address(),
                "readout_version":m.readout_version(),"readout_training":m.readout_training(),
                "memory_read_version":m.memory_read_version(),"memory_read_training":m.memory_read_training(),
                "construction":m.construction(),"status":"native geometric development prototype"}),
            )
        }
    }
}

fn prepare(a: &PrepareArgs) -> Result<(), String> {
    if a.development_every < if a.readout_split { 3 } else { 2 } {
        return Err(
            "development-every must leave distinct training, readout and development documents"
                .into(),
        );
    }
    let (documents, source) = read_corpus(&a.corpus)?;
    let mut unique = std::collections::BTreeSet::new();
    let mut train = Vec::new();
    let mut development = Vec::new();
    let mut readout = Vec::new();
    let mut skipped_duplicates = 0;
    for document in documents {
        if !unique.insert(digest(document.text.as_bytes())) {
            skipped_duplicates += 1;
            continue;
        }
        let target = if unique.len() % a.development_every == 0 {
            &mut development
        } else if a.readout_split && unique.len() % a.development_every == a.development_every - 1 {
            &mut readout
        } else {
            &mut train
        };
        target.push(document);
    }
    if train.is_empty() || development.is_empty() {
        return Err("need enough distinct documents for both training and development".into());
    }
    let train_path = a.output_directory.join("train.jsonl");
    let dev_path = a.output_directory.join("development.jsonl");
    let manifest_path = a.output_directory.join("corpus.json");
    let readout_path = a.output_directory.join("readout.jsonl");
    if [&train_path, &dev_path, &manifest_path, &readout_path]
        .iter()
        .any(|p| p.exists())
    {
        return Err("prepared output already exists; choose a new directory".into());
    }
    let encode = |docs: &[Document]| -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        for doc in docs {
            serde_json::to_writer(&mut bytes, doc).map_err(err)?;
            bytes.push(b'\n');
        }
        Ok(bytes)
    };
    let train_bytes = encode(&train)?;
    let dev_bytes = encode(&development)?;
    atomic_write(&train_path, &train_bytes)?;
    atomic_write(&dev_path, &dev_bytes)?;
    let readout_bytes = encode(&readout)?;
    if a.readout_split {
        atomic_write(&readout_path, &readout_bytes)?;
    }
    let manifest = serde_json::json!({"schema":"uor-r4.native-open-development-corpus/1",
        "source":source,"training_documents":train.len(),"development_documents":development.len(),
        "readout_documents":readout.len(),
        "readout_bytes":readout_bytes.len(),"readout_digest":digest(&readout_bytes),
        "training_bytes":train_bytes.len(),"development_bytes":dev_bytes.len(),
        "training_digest":digest(&train_bytes),"development_digest":digest(&dev_bytes),
        "duplicate_documents_removed":skipped_duplicates,
        "development_every":a.development_every,
        "split_scope":"Distinct fixed-byte documents from the declared source prefix. Adjacent chunks may share a story or topic; this is open development, not independent final held-out evidence."});
    write_report(Some(&manifest_path), &manifest)
}

fn fit_readout(a: &FitReadoutArgs) -> Result<(), String> {
    let mut inputs: Vec<_> = a.corpus.input.iter().map(PathBuf::as_path).collect();
    inputs.push(&a.model);
    let mut outputs = vec![a.output.as_path()];
    outputs.extend(a.report.iter().map(PathBuf::as_path));
    distinct_outputs(&inputs, &outputs)?;
    if a.output.exists() {
        return Err("readout output exists; choose a new artifact path".into());
    }
    let start = Instant::now();
    let model = load_model(&a.model)?;
    let (documents, source) = read_corpus(&a.corpus)?;
    let (learned, fit) = model
        .fit_readout(
            &documents,
            uor_r4_core::native_geometric::ReadoutFitConfig {
                max_positions: a.max_positions,
                epochs: a.epochs,
                max_queries: a.max_queries,
            },
        )
        .map_err(err)?;
    let bytes = learned.to_bytes().map_err(err)?;
    atomic_write(&a.output, &bytes)?;
    write_report(
        a.report.as_deref(),
        &serde_json::json!({
        "schema":"uor-r4.native-geometric-readout-run/1","source":source,"fit":fit,
        "baseline_artifact":model.artifact_cid(),"learned_artifact":learned.artifact_cid(),
        "elapsed_ms":elapsed_ms(start),"peak_rss_bytes":peak_rss_bytes()?,"output_bytes":bytes.len(),
        "interpretation":"Readout training metrics, not independent development or alpha evidence."}),
    )
}

fn fit_memory(a: &FitMemoryArgs) -> Result<(), String> {
    let mut inputs: Vec<_> = a.corpus.input.iter().map(PathBuf::as_path).collect();
    inputs.push(&a.model);
    let mut outputs = vec![a.output.as_path()];
    outputs.extend(a.report.iter().map(PathBuf::as_path));
    distinct_outputs(&inputs, &outputs)?;
    if a.output.exists() {
        return Err("memory-read output exists; choose a new artifact path".into());
    }
    let start = Instant::now();
    let model = load_model(&a.model)?;
    let (documents, source) = read_corpus(&a.corpus)?;
    let config = uor_r4_core::native_geometric::MemoryReadFitConfig {
        query_tokens: a.query_tokens,
        source_offsets: a.source_offsets,
        postings_per_address: a.postings_per_address,
        candidate_limit: a.candidates,
        max_positions: a.max_positions,
        epochs: a.epochs,
        max_features: a.max_features,
    };
    let (learned, fit) = if a.word_cues {
        model.fit_memory_read_with_word_cues(&documents, config)
    } else {
        model.fit_memory_read(&documents, config)
    }
    .map_err(err)?;
    let bytes = learned.to_bytes().map_err(err)?;
    atomic_write(&a.output, &bytes)?;
    write_report(
        a.report.as_deref(),
        &serde_json::json!({
        "schema":"uor-r4.native-geometric-memory-read-run/1", "source":source,"fit":fit,
        "baseline_artifact":model.artifact_cid(),"learned_artifact":learned.artifact_cid(),
        "elapsed_ms":elapsed_ms(start),"peak_rss_bytes":peak_rss_bytes()?,"output_bytes":bytes.len(),
        "interpretation":"Memory-read fitting evidence; evaluate the separate artifact on distinct development inputs."}),
    )
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn load_model(path: &Path) -> Result<Model, String> {
    let bytes = bounded_file_read(path, 256 * 1024 * 1024)?;
    Model::from_bytes(&bytes).map_err(err)
}

fn bounded_file_read(path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(err)?;
    if file.metadata().map_err(err)?.len() > maximum as u64 {
        return Err(format!(
            "{} exceeds the serialized envelope limit",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() > maximum {
        return Err("input grew beyond serialized envelope limit".into());
    }
    Ok(bytes)
}
fn print_json(value: &impl Serialize) -> Result<(), String> {
    println!("{}", serde_json::to_string_pretty(value).map_err(err)?);
    Ok(())
}
fn write_report(path: Option<&Path>, value: &impl Serialize) -> Result<(), String> {
    if let Some(path) = path {
        atomic_write(path, &serde_json::to_vec_pretty(value).map_err(err)?)?;
    }
    print_json(value)
}

fn train(a: &TrainArgs) -> Result<(), String> {
    if a.epochs == 0 || a.checkpoint_every == 0 || a.max_seconds == 0 || a.max_rss_mib == 0 {
        return Err("epochs, checkpoint interval and machine budget must be positive".into());
    }
    let mut outputs = vec![a.model.as_path(), a.checkpoint.as_path()];
    outputs.extend(a.report.iter().map(PathBuf::as_path));
    distinct_outputs(
        &a.corpus
            .input
            .iter()
            .map(PathBuf::as_path)
            .collect::<Vec<_>>(),
        &outputs,
    )?;
    if !a.resume && (a.model.exists() || a.checkpoint.exists()) {
        return Err("output exists; choose new paths or use --resume with the checkpoint".into());
    }
    let start = Instant::now();
    let (documents, source) = read_corpus(&a.corpus)?;
    let receipts: Vec<_> = documents
        .iter()
        .map(|d| (d.id.clone(), digest(d.text.as_bytes())))
        .collect();
    let config = Config {
        context_tokens: a.context,
        candidate_limit: a.candidates,
        max_lexical_pieces: a.lexical_pieces,
        max_rows: a.rows,
        max_associations: a.associations,
        ..Config::default()
    };
    let (mut header, mut trainer) = if a.resume {
        let (h, t) = read_checkpoint(&a.checkpoint)?;
        if h.documents != receipts || t.config() != &config {
            return Err("resume requires the same construction document bytes/order and model configuration".into());
        }
        if h.next_document >= documents.len() || h.completed_epochs > a.epochs {
            return Err("checkpoint cursor is incompatible with requested training".into());
        }
        if h.completed_epochs
            .checked_mul(documents.len())
            .and_then(|n| n.checked_add(h.next_document))
            != Some(t.progress().documents_completed)
        {
            return Err(
                "checkpoint cursor does not match committed training document count".into(),
            );
        }
        (h, t)
    } else {
        (
            CheckpointHeader {
                schema: "uor-r4.native-geometric-checkpoint/1".into(),
                next_document: 0,
                completed_epochs: 0,
                elapsed_ms: 0,
                documents: receipts,
            },
            Trainer::new(config, &documents).map_err(err)?,
        )
    };
    let previous_elapsed = header.elapsed_ms;
    let previous_model_bytes = if a.model.exists() {
        usize::try_from(fs::metadata(&a.model).map_err(err)?.len()).map_err(err)?
    } else {
        0
    };
    let checkpoint_limit = a
        .max_output_bytes
        .checked_sub(previous_model_bytes)
        .ok_or("existing model already exceeds combined storage budget")?;
    let mut since_checkpoint = 0usize;
    let mut stop = "epochs_complete";
    while header.completed_epochs < a.epochs {
        if previous_elapsed.saturating_add(elapsed_ms(start)) >= a.max_seconds.saturating_mul(1000)
        {
            stop = "cumulative_time_budget";
            break;
        }
        if peak_rss_bytes()? > a.max_rss_mib.saturating_mul(1_048_576) {
            stop = "process_memory_budget";
            break;
        }
        trainer
            .train_documents(&documents[header.next_document..header.next_document + 1])
            .map_err(err)?;
        header.next_document += 1;
        since_checkpoint += 1;
        if header.next_document == documents.len() {
            header.next_document = 0;
            header.completed_epochs += 1;
        }
        if since_checkpoint >= a.checkpoint_every {
            header.elapsed_ms = previous_elapsed.saturating_add(elapsed_ms(start));
            save_checkpoint(&a.checkpoint, &header, &trainer, checkpoint_limit)?;
            since_checkpoint = 0;
        }
    }
    header.elapsed_ms = previous_elapsed.saturating_add(elapsed_ms(start));
    // Save fitting state before compilation so interruption cannot lose the learned counts.
    save_checkpoint(&a.checkpoint, &header, &trainer, checkpoint_limit)?;
    if trainer.progress().target_positions == 0 {
        return write_report(
            a.report.as_deref(),
            &serde_json::json!({
            "schema":"uor-r4.native-geometric-training-run/1","stop":stop,
            "checkpoint":a.checkpoint,"model_status":"not_fitted",
            "cumulative_elapsed_ms":previous_elapsed.saturating_add(elapsed_ms(start)),
            "source":source,"training":trainer.progress()}),
        );
    }
    let model = trainer.compile().map_err(err)?;
    let model_bytes = model.to_bytes().map_err(err)?;
    let checkpoint_size = fs::metadata(&a.checkpoint).map_err(err)?.len();
    if model_bytes.len() as u64 + checkpoint_size > a.max_output_bytes as u64 {
        return Err(
            "combined model/checkpoint storage budget exhausted; fitting checkpoint is retained"
                .into(),
        );
    }
    atomic_write(&a.model, &model_bytes)?;
    header.elapsed_ms = previous_elapsed.saturating_add(elapsed_ms(start));
    save_checkpoint(
        &a.checkpoint,
        &header,
        &trainer,
        a.max_output_bytes - model_bytes.len(),
    )?;
    let report = serde_json::json!({"schema":"uor-r4.native-geometric-training-run/1",
        "stop":stop,"source":source,"model":a.model,"checkpoint":a.checkpoint,
        "config":trainer.config(),"training":trainer.progress(),
        "completed_epochs":header.completed_epochs,"next_document":header.next_document,
        "cumulative_elapsed_ms":header.elapsed_ms,"max_seconds":a.max_seconds,
        "peak_rss_bytes":peak_rss_bytes()?,"max_rss_mib":a.max_rss_mib,
        "model_bytes":model_bytes.len(),"checkpoint_bytes":fs::metadata(&a.checkpoint).map_err(err)?.len(),
        "budget_boundary":"Checked between bounded documents; checkpoint and compile finalization are charged and may overrun the time boundary.",
        "interpretation":"Fitted native prototype artifact; useful conversation and coding require separate evaluation."});
    write_report(a.report.as_deref(), &report)
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn digest(bytes: &[u8]) -> String {
    // Provenance identity only; this digest is never a model feature.
    blake3::hash(bytes).to_hex().to_string()
}

fn resolved_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(err)?.join(path)
    };
    let mut result = PathBuf::new();
    for part in absolute.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            _ => {
                result.push(part.as_os_str());
                if result.exists() {
                    result = fs::canonicalize(&result).map_err(err)?;
                }
            }
        }
    }
    Ok(result)
}

fn distinct_outputs(inputs: &[&Path], outputs: &[&Path]) -> Result<(), String> {
    let protected: std::collections::BTreeSet<_> = inputs
        .iter()
        .map(|p| resolved_path(p))
        .collect::<Result<_, _>>()?;
    let mut destinations = std::collections::BTreeSet::new();
    for path in outputs {
        let resolved = resolved_path(path)?;
        if protected.contains(&resolved) || !destinations.insert(resolved) {
            return Err("model, checkpoint, report and corpus paths must be distinct".into());
        }
    }
    Ok(())
}

fn read_corpus(a: &CorpusArgs) -> Result<(Vec<Document>, serde_json::Value), String> {
    if a.max_input_bytes == 0 || a.document_bytes < 4 || a.document_bytes > 1_048_576 {
        return Err(
            "input budget must be positive; document-bytes must be between 4 and 1048576".into(),
        );
    }
    let mut remaining = a.max_input_bytes;
    let mut documents = Vec::new();
    let mut sources = Vec::new();
    for path in &a.input {
        if remaining == 0 {
            break;
        }
        let file = fs::File::open(path).map_err(err)?;
        let total_bytes = file.metadata().map_err(err)?.len();
        let mut bytes = Vec::new();
        file.take(remaining as u64)
            .read_to_end(&mut bytes)
            .map_err(err)?;
        remaining -= bytes.len();
        let truncated = (bytes.len() as u64) < total_bytes;
        if truncated {
            if let Err(error) = std::str::from_utf8(&bytes) {
                if error.error_len().is_none() {
                    bytes.truncate(error.valid_up_to());
                }
            }
        }
        let text = std::str::from_utf8(&bytes).map_err(err)?;
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            for (index, line) in text.split_inclusive('\n').enumerate() {
                if truncated && !line.ends_with('\n') {
                    break;
                }
                if line.trim().is_empty() {
                    continue;
                }
                let doc: Document = serde_json::from_str(line)
                    .map_err(|e| format!("{}:{}: {e}", path.display(), index + 1))?;
                if doc.text.len() > a.document_bytes {
                    return Err(format!(
                        "JSONL document {} exceeds --document-bytes",
                        doc.id
                    ));
                }
                documents.push(doc);
            }
        } else {
            let mut offset = 0;
            while offset < text.len() {
                let mut end = (offset + a.document_bytes).min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                documents.push(Document {
                    id: format!("{}:{offset}", path.display()),
                    text: text[offset..end].into(),
                });
                offset = end;
            }
        }
        sources.push(serde_json::json!({"path":path,"read_bytes":bytes.len(),"source_bytes":total_bytes,"prefix_truncated":truncated}));
    }
    if documents.is_empty() {
        return Err("no complete construction/evaluation documents were read".into());
    }
    Ok((
        documents,
        serde_json::json!({"inputs":sources,"input_byte_budget":a.max_input_bytes}),
    ))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(err)?;
    }
    let temp = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name()
            .ok_or("output needs a filename")?
            .to_string_lossy(),
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(err)?;
    let result = (|| {
        file.write_all(bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        fs::rename(&temp, path).map_err(err)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
fn save_checkpoint(
    path: &Path,
    header: &CheckpointHeader,
    trainer: &Trainer,
    limit: usize,
) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(header).map_err(err)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("checkpoint header exceeds envelope limit; prior checkpoint retained".into());
    }
    bytes.push(b'\n');
    bytes.extend(trainer.to_bytes().map_err(err)?);
    if bytes.len() > limit.min(320 * 1024 * 1024) {
        return Err("checkpoint storage budget exhausted; prior checkpoint retained".into());
    }
    atomic_write(path, &bytes)
}
fn read_checkpoint(path: &Path) -> Result<(CheckpointHeader, Trainer), String> {
    let bytes = bounded_file_read(path, 320 * 1024 * 1024)?;
    let boundary = bytes
        .iter()
        .position(|b| *b == b'\n')
        .ok_or("checkpoint header missing")?;
    if boundary > 64 * 1024 * 1024 {
        return Err("checkpoint header exceeds envelope limit".into());
    }
    let header: CheckpointHeader = serde_json::from_slice(&bytes[..boundary]).map_err(err)?;
    if header.schema != "uor-r4.native-geometric-checkpoint/1" {
        return Err("unsupported checkpoint schema".into());
    }
    Ok((
        header,
        Trainer::from_bytes(&bytes[boundary + 1..]).map_err(err)?,
    ))
}

fn peak_rss_bytes() -> Result<u64, String> {
    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes the provided rusage on success.
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error().to_string());
        }
        let usage = unsafe { usage.assume_init() };
        let value = u64::try_from(usage.ru_maxrss).map_err(err)?;
        #[cfg(target_os = "macos")]
        {
            Ok(value)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(value.saturating_mul(1024))
        }
    }
    #[cfg(not(unix))]
    {
        Err("process memory accounting is currently supported on Unix hosts".into())
    }
}

fn chat(a: &ChatArgs) -> Result<(), String> {
    let model = load_model(&a.model)?;
    if !(1..=4096).contains(&a.max_tokens) {
        return Err("max-tokens must be between 1 and 4096".into());
    }
    let mut session = model.session(Control::Full).map_err(err)?;
    session
        .observe(&model, uor_r4_core::native_geometric::BOS)
        .map_err(err)?;
    eprintln!("Native geometric development model. /reset clears the conversation; /exit quits.");
    for line in io::stdin().lock().lines() {
        let line = line.map_err(err)?;
        if line == "/exit" {
            break;
        }
        if line == "/reset" {
            session = model.session(Control::Full).map_err(err)?;
            session
                .observe(&model, uor_r4_core::native_geometric::BOS)
                .map_err(err)?;
            continue;
        }
        for token in model
            .encode(&format!("\nUser: {line}\nAssistant:"))
            .map_err(err)?
        {
            session.observe(&model, token).map_err(err)?;
        }
        let mut output = Vec::new();
        for _ in 0..a.max_tokens {
            let token = session.predict(&model).map_err(err)?.token;
            if token == uor_r4_core::native_geometric::EOS {
                break;
            }
            session.observe(&model, token).map_err(err)?;
            output.push(token);
        }
        println!(
            "{}",
            String::from_utf8_lossy(&model.decode(&output).map_err(err)?)
        );
        io::stdout().flush().map_err(err)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let unique = format!(
                "r4-native-cli-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir(&path).expect("test directory");
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn native_geometric_cli_training_resume_preserves_exact_counts_and_bytes() {
        let dir = Directory::new();
        let input = dir.0.join("train.jsonl");
        fs::write(&input,"{\"id\":\"a\",\"text\":\"The red bird sings. The blue bird flies.\"}\n{\"id\":\"b\",\"text\":\"fn answer() -> u32 { 42 }\\n\"}\n").expect("corpus");
        let mut args = TrainArgs {
            corpus: CorpusArgs {
                input: vec![input],
                max_input_bytes: 4096,
                document_bytes: 1024,
            },
            model: dir.0.join("model.json"),
            checkpoint: dir.0.join("checkpoint.bin"),
            resume: false,
            epochs: 1,
            context: 32,
            candidates: 8,
            lexical_pieces: 64,
            rows: 4096,
            associations: 20000,
            max_seconds: 60,
            max_rss_mib: 4096,
            max_output_bytes: 16_777_216,
            checkpoint_every: 1,
            report: None,
        };
        train(&args).expect("native fit");
        let first = load_model(&args.model).expect("load");
        assert_eq!(first.training().documents_completed, 2);
        assert!(train(&args)
            .expect_err("never overwrite new run")
            .contains("output exists"));
        args.resume = true;
        args.epochs = 2;
        train(&args).expect("resume second epoch");
        let second = load_model(&args.model).expect("reload");
        assert_eq!(
            second.training().target_positions,
            first.training().target_positions * 2
        );
        let resumed = fs::read(&args.model).expect("artifact");
        train(&args).expect("already completed resume is idempotent");
        assert_eq!(resumed, fs::read(&args.model).expect("artifact"));
        args.report = Some(dir.0.join(".").join("model.json"));
        assert!(train(&args)
            .expect_err("report may not overwrite model")
            .contains("must be distinct"));
        assert_eq!(resumed, fs::read(&args.model).expect("protected artifact"));
        args.report = None;
        let eval = [Document {
            id: "dev".into(),
            text: "The green bird flies.".into(),
        }];
        assert!(
            second
                .evaluate(&eval, Control::Full)
                .expect("heldout")
                .positions
                > 0
        );
        let result = second.generate("fn", 8, Control::Full).expect("generate");
        assert!(result.token_ids.len() <= 8);
    }

    #[test]
    fn native_geometric_cli_prefix_reader_preserves_utf8_and_reports_truncation() {
        let dir = Directory::new();
        let input = dir.0.join("text.txt");
        fs::write(&input, "abcλdef").expect("input");
        let args = CorpusArgs {
            input: vec![input],
            max_input_bytes: 4,
            document_bytes: 4,
        };
        let (docs, report) = read_corpus(&args).expect("prefix");
        assert_eq!(docs[0].text, "abc");
        assert_eq!(report["inputs"][0]["prefix_truncated"], true);
    }

    #[test]
    fn native_geometric_cli_checkpoint_rejects_changed_construction() {
        let dir = Directory::new();
        let docs = [Document {
            id: "a".into(),
            text: "hello world".into(),
        }];
        let trainer = Trainer::new(Config::default(), &docs).expect("trainer");
        let header = CheckpointHeader {
            schema: "wrong".into(),
            next_document: 0,
            completed_epochs: 0,
            elapsed_ms: 0,
            documents: vec![],
        };
        let path = dir.0.join("checkpoint");
        save_checkpoint(&path, &header, &trainer, 16_777_216).expect("save");
        assert!(read_checkpoint(&path)
            .expect_err("schema mismatch")
            .contains("unsupported checkpoint"));
    }

    #[test]
    fn native_geometric_cli_preparation_produces_disjoint_complete_documents() {
        let dir = Directory::new();
        let input = dir.0.join("input.txt");
        fs::write(&input, "aaaabbbbccccdddd").expect("source");
        let output = dir.0.join("prepared");
        prepare(&PrepareArgs {
            corpus: CorpusArgs {
                input: vec![input],
                max_input_bytes: 16,
                document_bytes: 4,
            },
            output_directory: output.clone(),
            development_every: 2,
            readout_split: false,
        })
        .expect("prepare");
        let (training, _) = read_corpus(&CorpusArgs {
            input: vec![output.join("train.jsonl")],
            max_input_bytes: 4096,
            document_bytes: 4,
        })
        .expect("training");
        let (development, _) = read_corpus(&CorpusArgs {
            input: vec![output.join("development.jsonl")],
            max_input_bytes: 4096,
            document_bytes: 4,
        })
        .expect("development");
        assert_eq!(
            training.iter().map(|d| d.text.as_str()).collect::<Vec<_>>(),
            vec!["aaaa", "cccc"]
        );
        assert_eq!(
            development
                .iter()
                .map(|d| d.text.as_str())
                .collect::<Vec<_>>(),
            vec!["bbbb", "dddd"]
        );
    }
}
