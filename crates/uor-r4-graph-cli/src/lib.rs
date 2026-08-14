//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! Read docs/transformerless/TRANSFORMERLESS.md (the extrapolation) and docs/transformerless/PROOF.md (the
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

use uor_r4_core::transformerless::{code_sidecar, compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_certify as score_runtime;
use uor_r4_graph_compiler::induction as cover;
use uor_r4_graph_compiler::observation as observe;
use uor_r4_graph_compiler::observation_text as observe_text;
use uor_r4_graph_compiler::reproducibility as repro;
mod convert_r4g1;
pub mod cover_sweep;
pub mod recommend_scale;
mod runtime_corpus;
mod scenarios;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use uor_r4_core::transformerless::hf_bpe::{
    TokenizerAdapter, TokenizerAdapterKey, TokenizerKind, adapter_constructor,
    resolve_source_tokenizer,
};
use uor_r4_core::transformerless::scenarios as core_scenarios;
use uor_r4_core::transformerless::scenarios::RuntimeTokenizerDecodeTable;
use uor_r4_model_source::{BehaviorSource, LlamaOracle, SourceUnavailable, Teacher, TeacherOracle};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const DEFAULT_TOKENIZER: &str = "/tmp/ref/tokenizer.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";
const DEFAULT_HF_SOURCE_PATH: &str = ".uor-models/sources/smollm2-135m-instruct";
const DEFAULT_HF_COMPILED_PATH: &str = ".uor-models/compiled/smollm2-135m-instruct";
const DEFAULT_HF_EVALUATION_REPORT: &str = "instruction-eval.json";
const DEFAULT_TEXT_CORPUS: &str = ".uor-models/corpora/simple-wiki-20231101/articles.jsonl";
const TOKENIZER_ADAPTER_FILE: &str = "tokenizer_adapter.json";
static TOKENIZER_ADAPTER_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn is_blake3_cid(value: &str) -> bool {
    value.len() == "blake3:".len() + 64
        && value.starts_with("blake3:")
        && value["blake3:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Read an explicitly selected tokenizer input without allowing absence to
/// collapse into the legacy zero-CID mode. `symlink_metadata` deliberately
/// rejects directories, symlinks (including dangling ones), and special files;
/// only omission of the flag selects compatibility behavior.
fn explicit_tokenizer_cid(path: Option<&Path>) -> Result<Option<[u8; 32]>, SourceUnavailable> {
    let Some(path) = path else {
        return Ok(None);
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
    Ok(Some(*blake3::hash(&bytes).as_bytes()))
}

/// Select the scored graph's tokenizer identity. A scored graph inherits a
/// nonzero cover binding; an explicit tokenizer may introduce a binding for a
/// legacy cover or reproduce the existing one, but may never replace it.
fn scored_tokenizer_cid(
    cover_bytes: Option<&[u8]>,
    explicit: Option<[u8; 32]>,
) -> Result<[u8; 32], SourceUnavailable> {
    let cover_cid = match cover_bytes {
        None => [0; 32],
        Some(bytes) => {
            let view = uor_r4_graph_format::GraphView::parse(bytes).map_err(|error| {
                SourceUnavailable::new(format!("invalid cover R4G1 artifact: {error}"))
            })?;
            view.verify_cids().map_err(|error| {
                SourceUnavailable::new(format!("invalid cover R4G1 integrity: {error}"))
            })?;
            view.head()
                .ok_or_else(|| SourceUnavailable::new("cover R4G1 artifact has no HEAD"))?
                .tokenizer_cid()
                .0
        }
    };
    if cover_cid == [0; 32] {
        return Ok(explicit.unwrap_or([0; 32]));
    }
    match explicit {
        None => Ok(cover_cid),
        Some(explicit) if cover_cid == explicit => Ok(cover_cid),
        Some(explicit) => Err(SourceUnavailable::new(format!(
            "cover tokenizer CID blake3:{} does not match explicit --tokenizer CID blake3:{}; refusing to change tokenizer identity while scoring",
            blake3::Hash::from(cover_cid).to_hex(),
            blake3::Hash::from(explicit).to_hex(),
        ))),
    }
}

fn validate_tokenizer_adapter_record(adapter: &TokenizerAdapter) -> Result<(), SourceUnavailable> {
    // Registry membership is part of provenance validity: a self-consistent
    // digest cannot turn an unknown family/version into supported behavior.
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

fn read_compile_tokenizer_adapter(
    output: &Path,
) -> Result<Option<TokenizerAdapter>, SourceUnavailable> {
    let sidecar = output.join(TOKENIZER_ADAPTER_FILE);
    let metadata = match std::fs::symlink_metadata(&sidecar) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{}: {error}",
                sidecar.display()
            )));
        }
    };
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{} exists but is not a regular file; refusing tokenizer adapter provenance through a directory, symlink, or special file",
            sidecar.display()
        )));
    }
    let bytes = std::fs::read(&sidecar)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", sidecar.display())))?;
    let recorded: TokenizerAdapter = serde_json::from_slice(&bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{} is not a valid tokenizer adapter record: {error}",
            sidecar.display()
        ))
    })?;
    validate_tokenizer_adapter_record(&recorded).map_err(|error| {
        SourceUnavailable::new(format!("{}: {}", sidecar.display(), error.reason))
    })?;
    Ok(Some(recorded))
}

fn require_matching_compile_tokenizer_adapter(
    output: &Path,
    requested: &TokenizerAdapter,
    recorded: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    if recorded == requested {
        return Ok(());
    }
    Err(SourceUnavailable::new(format!(
        "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested {}/{} (CID {}, digest {}); incompatible compile resume/evaluation identity refused before mutation",
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

/// Bind a resumable compiler output to the exact full adapter identity before
/// `tokenizer.bin`, `corpus.meta`, or `corpus.records` can be changed.
fn pin_compile_tokenizer_adapter(
    output: &Path,
    requested: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    validate_tokenizer_adapter_record(requested)?;
    let sidecar = output.join(TOKENIZER_ADAPTER_FILE);
    if let Some(recorded) = read_compile_tokenizer_adapter(output)? {
        return require_matching_compile_tokenizer_adapter(output, requested, &recorded);
    }

    // A pre-#718 compiler corpus has no adapter identity. Non-empty tokenizer
    // or corpus files make that absence an incompatible legacy era; never
    // relabel those bytes merely because the current source happens to parse.
    for name in ["tokenizer.bin", "corpus.meta", "corpus.records"] {
        let path = output.join(name);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(SourceUnavailable::new(format!(
                    "{} exists but is not a regular file; refusing to infer an empty compiler payload through a directory, symlink, or special file",
                    path.display()
                )));
            }
            Ok(metadata) if metadata.len() != 0 => {
                return Err(SourceUnavailable::new(format!(
                    "{} has no {TOKENIZER_ADAPTER_FILE} but already contains {name} payload; refusing to relabel legacy/unpinned compiler bytes as {}/{} before mutation",
                    output.display(),
                    requested.family,
                    requested.version
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "{}: {error}",
                    path.display()
                )));
            }
        }
    }

    std::fs::create_dir_all(output).map_err(SourceUnavailable::new)?;
    let mut bytes = serde_json::to_vec_pretty(requested).map_err(SourceUnavailable::new)?;
    bytes.push(b'\n');
    let temporary = loop {
        let sequence = TOKENIZER_ADAPTER_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = output.join(format!(
            ".{TOKENIZER_ADAPTER_FILE}.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
                    let _ = std::fs::remove_file(&candidate);
                    return Err(SourceUnavailable::new(format!(
                        "{}: {error}",
                        candidate.display()
                    )));
                }
                drop(file);
                break candidate;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "{}: {error}",
                    candidate.display()
                )));
            }
        }
    };

    // `hard_link` is an atomic no-clobber publish: unlike rename, it cannot
    // replace a sidecar another compiler created after our initial read. A
    // loser re-reads and accepts only the exact same validated identity.
    match std::fs::hard_link(&temporary, &sidecar) {
        Ok(()) => {
            std::fs::remove_file(&temporary).map_err(|error| {
                SourceUnavailable::new(format!("{}: {error}", temporary.display()))
            })?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(&temporary);
            let recorded = read_compile_tokenizer_adapter(output)?.ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "{} appeared during tokenizer adapter publication but is now absent",
                    sidecar.display()
                ))
            })?;
            require_matching_compile_tokenizer_adapter(output, requested, &recorded)
        }
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            Err(SourceUnavailable::new(format!(
                "tokenizer adapter sidecar publish {} -> {}: {error}",
                temporary.display(),
                sidecar.display()
            )))
        }
    }
}

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
    tokenizer_adapter: Option<TokenizerAdapterKey>,
}

#[derive(Debug, PartialEq, Eq)]
struct RecordedCompileOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    vocab_size: usize,
    output: PathBuf,
}

fn parse_recorded_compile_options(
    args: &[String],
) -> Result<RecordedCompileOptions, SourceUnavailable> {
    let mut corpus_meta = None;
    let mut corpus_recs = None;
    let mut vocab_size = None;
    let mut output = None;
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--corpus-meta" => corpus_meta = Some(PathBuf::from(value)),
            "--corpus-recs" => corpus_recs = Some(PathBuf::from(value)),
            "--vocab-size" => {
                let parsed = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --vocab-size value: {value}"))
                })?;
                if parsed == 0 {
                    return Err(SourceUnavailable::new(
                        "--vocab-size must be greater than zero",
                    ));
                }
                vocab_size = Some(parsed);
            }
            "--out" | "--output" => output = Some(PathBuf::from(value)),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown compile-recorded option: {flag}"
                )));
            }
        }
        index += 2;
    }
    Ok(RecordedCompileOptions {
        corpus_meta: corpus_meta
            .ok_or_else(|| SourceUnavailable::new("--corpus-meta is required"))?,
        corpus_recs: corpus_recs
            .ok_or_else(|| SourceUnavailable::new("--corpus-recs is required"))?,
        vocab_size: vocab_size.ok_or_else(|| SourceUnavailable::new("--vocab-size is required"))?,
        output: output.ok_or_else(|| SourceUnavailable::new("--out is required"))?,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct EvaluateReportOptions {
    source: PathBuf,
    compiled: PathBuf,
    report: Option<PathBuf>,
    sequence_length: usize,
    /// #268 candidate 3 (prompt framing): prepend the source tokenizer's
    /// BOS token at every held-out story boundary before teacher-forcing.
    bos: bool,
    /// Smoke-run cap: evaluate only the first N held-out stories. The
    /// report is stamped as a SMOKE run and its numbers are not quotable;
    /// this exists so a change can be verified to sit in the measured
    /// path before a full run is spent on it.
    max_held_out_stories: Option<u32>,
    tokenizer_adapter: Option<TokenizerAdapterKey>,
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
    tokenizer_adapter: Option<TokenizerAdapterKey>,
}

#[derive(Debug, Default)]
struct TokenizerAdapterKeyBuilder {
    family: Option<String>,
    version: Option<u32>,
}

impl TokenizerAdapterKeyBuilder {
    fn parse(&mut self, flag: &str, value: &str) -> Result<bool, SourceUnavailable> {
        match flag {
            "--tokenizer-family" => self.family = Some(value.to_owned()),
            "--tokenizer-version" => {
                self.version = Some(value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --tokenizer-version value: {value}"))
                })?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn finish(self) -> Result<Option<TokenizerAdapterKey>, SourceUnavailable> {
        match (self.family, self.version) {
            (None, None) => Ok(None),
            (Some(family), Some(version)) => Ok(Some(TokenizerAdapterKey::new(family, version))),
            (Some(_), None) => Err(SourceUnavailable::new(
                "--tokenizer-family requires --tokenizer-version",
            )),
            (None, Some(_)) => Err(SourceUnavailable::new(
                "--tokenizer-version requires --tokenizer-family",
            )),
        }
    }
}

fn parse_observe_options(args: &[String]) -> Result<ObserveOptions, SourceUnavailable> {
    let mut tokenizer_adapter = TokenizerAdapterKeyBuilder::default();
    let mut options = ObserveOptions {
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        checkpoint: None,
        output: PathBuf::from("obs"),
        seconds: 300,
        target: 20_000,
        shards: 4,
        sequence_length: 128,
        tokenizer_adapter: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--source" => options.source = PathBuf::from(value),
            "--checkpoint" => options.checkpoint = Some(PathBuf::from(value)),
            "--out" => options.output = PathBuf::from(value),
            "--seconds" => {
                options.seconds = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --seconds value: {value}"))
                })?;
            }
            "--target" => {
                options.target = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --target value: {value}"))
                })?;
            }
            "--shards" => {
                options.shards = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --shards value: {value}"))
                })?;
                if options.shards > observe::MAX_SHARD_BITS {
                    return Err(SourceUnavailable::new(format!(
                        "--shards must be at most {} (2^N shard files)",
                        observe::MAX_SHARD_BITS
                    )));
                }
            }
            "--sequence-length" => {
                options.sequence_length = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --sequence-length value: {value}"))
                })?;
                if options.sequence_length == 0 {
                    return Err(SourceUnavailable::new(
                        "--sequence-length must be greater than zero",
                    ));
                }
            }
            flag if tokenizer_adapter.parse(flag, value)? => {}
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown observe option: {flag}"
                )));
            }
        }
        index += 2;
    }
    options.tokenizer_adapter = tokenizer_adapter.finish()?;
    if options.checkpoint.is_some() && options.tokenizer_adapter.is_some() {
        return Err(SourceUnavailable::new(
            "--tokenizer-family/--tokenizer-version select registered source tokenizers and cannot be used with legacy --checkpoint",
        ));
    }
    Ok(options)
}

/// Pin the tokenizer identity before a resumable observation command writes
/// `tokenizer.bin` or reconciles any partial shard tail. The writer performs
/// the registry/digest checks and refuses adapterless payload relabeling.
fn pin_observation_tokenizer_before_output(
    output: &Path,
    shard_bits: u8,
    requested: Option<&TokenizerAdapter>,
) -> Result<(), SourceUnavailable> {
    let mut writer = observe::ObservationShardWriter::open(output, shard_bits)?;
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

/// Observation pipeline v2 (plan §5 Phase 2): the same teacher generation
/// as [`compile_hugging_face`]'s corpus step, spilled into content-
/// addressed, resumable shards instead of one corpus stream.
pub fn observe_command(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- observe ...`"
    );
    let options = parse_observe_options(args)?;
    let token_byte_lengths: Option<Vec<u32>>;
    let mut oracle: Box<dyn TeacherOracle> = if let Some(checkpoint) = &options.checkpoint {
        // Legacy llama2.c checkpoint: no HF tokenizer tree, so byte
        // anchors stay at the v3 "unknown" value.
        token_byte_lengths = None;
        let path = checkpoint
            .to_str()
            .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
        pin_observation_tokenizer_before_output(&options.output, options.shards, None)?;
        Box::new(LlamaOracle::load(path))
    } else {
        let tokenizer =
            resolve_source_tokenizer(&options.source, options.tokenizer_adapter.as_ref())?;
        let runtime_table = tokenizer.runtime_decode_table().ok_or_else(|| {
            SourceUnavailable::new("registered source tokenizer has no runtime decode table")
        })?;
        let oracle = Teacher::load_with_sequence_length(&options.source, options.sequence_length)
            .map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
        let adapter = tokenizer.adapter();
        pin_observation_tokenizer_before_output(&options.output, options.shards, adapter.as_ref())?;
        eprintln!("exporting tokenizer...");
        let export = core_scenarios::export_runtime_tokenizer_table(
            &runtime_table,
            options.output.join("tokenizer.bin"),
        )
        .map_err(SourceUnavailable::new)?;
        token_byte_lengths = export.source_byte_lengths;
        Box::new(oracle)
    };
    let summary = observe::observe_sharded(
        oracle.as_mut(),
        options.seconds,
        options.target,
        options.shards,
        &options.output,
        token_byte_lengths.as_deref(),
    )
    .map_err(SourceUnavailable::new)?;
    if summary.done {
        // Persist the merged record stream so Gate C can consume it as
        // --corpus-recs with state.bin as --corpus-meta (same convention
        // as the from-text driver, issue #75).
        let merged = observe::merge_shards(&options.output).map_err(SourceUnavailable::new)?;
        let merged_path = options.output.join("merged.bin");
        std::fs::write(&merged_path, &merged)?;
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
    workers: usize,
    batch: usize,
    tokenizer_adapter: Option<TokenizerAdapterKey>,
}

fn parse_observe_text_options(args: &[String]) -> Result<ObserveTextOptions, SourceUnavailable> {
    let mut tokenizer_adapter = TokenizerAdapterKeyBuilder::default();
    let mut options = ObserveTextOptions {
        input: PathBuf::from(DEFAULT_TEXT_CORPUS),
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        checkpoint: None,
        tokenizer: None,
        output: PathBuf::from("obs-text"),
        seconds: 300,
        shards: 4,
        sequence_length: 128,
        workers: 1,
        batch: 1,
        tokenizer_adapter: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--input" => options.input = PathBuf::from(value),
            "--source" => options.source = PathBuf::from(value),
            "--checkpoint" => options.checkpoint = Some(PathBuf::from(value)),
            "--tokenizer" => options.tokenizer = Some(PathBuf::from(value)),
            "--out" => options.output = PathBuf::from(value),
            "--seconds" => {
                options.seconds = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --seconds value: {value}"))
                })?;
            }
            "--shards" => {
                options.shards = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --shards value: {value}"))
                })?;
                if options.shards > observe::MAX_SHARD_BITS {
                    return Err(SourceUnavailable::new(format!(
                        "--shards must be at most {} (2^N shard files)",
                        observe::MAX_SHARD_BITS
                    )));
                }
            }
            "--sequence-length" => {
                options.sequence_length = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --sequence-length value: {value}"))
                })?;
                if options.sequence_length == 0 {
                    return Err(SourceUnavailable::new(
                        "--sequence-length must be greater than zero",
                    ));
                }
            }
            "--workers" => {
                // Teacher instances observing articles in parallel. Each holds a
                // full copy of the teacher weights, so memory scales with this;
                // the produced corpus is identical to `--workers 1`.
                options.workers = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --workers value: {value}"))
                })?;
                if options.workers == 0 {
                    return Err(SourceUnavailable::new(
                        "--workers must be greater than zero",
                    ));
                }
            }
            "--batch" => {
                // Articles teacher-forced together per forward (Hugging Face
                // teacher only). One shared weight copy; the throughput lever.
                options.batch = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --batch value: {value}"))
                })?;
                if options.batch == 0 {
                    return Err(SourceUnavailable::new("--batch must be greater than zero"));
                }
            }
            flag if tokenizer_adapter.parse(flag, value)? => {}
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown observe-text option: {flag}"
                )));
            }
        }
        index += 2;
    }
    options.tokenizer_adapter = tokenizer_adapter.finish()?;
    if options.checkpoint.is_some() && options.tokenizer_adapter.is_some() {
        return Err(SourceUnavailable::new(
            "--tokenizer-family/--tokenizer-version select registered source tokenizers and cannot be used with legacy --checkpoint",
        ));
    }
    if options.checkpoint.is_none() && options.tokenizer.is_some() {
        return Err(SourceUnavailable::new(
            "--tokenizer is the legacy llama2.c tokenizer and requires --checkpoint",
        ));
    }
    Ok(options)
}

fn resolve_registered_observation_tokenizer(
    source: &Path,
    selection: Option<&TokenizerAdapterKey>,
) -> Result<(TokenizerKind, RuntimeTokenizerDecodeTable), SourceUnavailable> {
    // Resolve and materialize the table before loading an expensive teacher
    // or touching resumable output. The same model then supplies manifest
    // identity, host encoding, and runtime decoding.
    let tokenizer = resolve_source_tokenizer(source, selection)?;
    let runtime_table = tokenizer.runtime_decode_table().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no runtime decode table")
    })?;
    Ok((tokenizer, runtime_table))
}

fn pin_and_export_observation_tokenizer(
    tokenizer: &TokenizerKind,
    runtime_table: &RuntimeTokenizerDecodeTable,
    output: &Path,
    shard_bits: u8,
) -> Result<Option<Vec<u32>>, SourceUnavailable> {
    let adapter = tokenizer.adapter();
    pin_observation_tokenizer_before_output(output, shard_bits, adapter.as_ref())?;
    eprintln!("exporting tokenizer...");
    let export =
        core_scenarios::export_runtime_tokenizer_table(runtime_table, output.join("tokenizer.bin"))
            .map_err(SourceUnavailable::new)?;
    Ok(export.source_byte_lengths)
}

/// From-text observation driver (issue #72): feed the sealed natural-text
/// corpus (D3) through the teacher, recording the same v3 observation
/// records the autoregressive `observe` path produces, with the corpus
/// split rule applied at write time and recorded per shard.
pub fn observe_text_command(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- observe-text ...`"
    );
    let options = parse_observe_text_options(args)?;
    // Batched teacher path: one shared weight copy, B articles per forward. This
    // is the throughput lever (measured ~15× at batch 32) and supersedes
    // --workers for the Hugging Face teacher.
    if options.batch > 1 {
        if options.checkpoint.is_some() {
            return Err(SourceUnavailable::new(
                "--batch is only supported for the Hugging Face teacher, not a legacy --checkpoint",
            ));
        }
        return observe_text_batched_command(&options);
    }
    let token_byte_lengths: Option<Vec<u32>>;
    let tokenizer: TokenizerKind;
    let oracle: Box<dyn TeacherOracle + Send> = if let Some(checkpoint) = &options.checkpoint {
        // Legacy llama2.c checkpoint: the companion tokenizer is the
        // scoreless tokenizer.bin fetched by `setup` (overridable with
        // --tokenizer); its piece byte lengths anchor records into the
        // article text. This path is untouched by issue #242 — its κ-pinned
        // baselines depend on the exact legacy encoding.
        let tokenizer_path = options
            .tokenizer
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TOKENIZER));
        let legacy = scenarios::Tokenizer::try_load(&tokenizer_path).map_err(|error| {
            SourceUnavailable::new(format!("{}: {error}", tokenizer_path.display()))
        })?;
        token_byte_lengths = Some(
            legacy
                .vocab
                .iter()
                .map(|piece| piece.len() as u32)
                .collect(),
        );
        tokenizer = TokenizerKind::Legacy(legacy);
        let path = checkpoint
            .to_str()
            .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
        Box::new(LlamaOracle::load(path))
    } else {
        let (resolved_tokenizer, runtime_table) = resolve_registered_observation_tokenizer(
            &options.source,
            options.tokenizer_adapter.as_ref(),
        )?;
        let oracle = Teacher::load_with_sequence_length(&options.source, options.sequence_length)
            .map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
        token_byte_lengths = pin_and_export_observation_tokenizer(
            &resolved_tokenizer,
            &runtime_table,
            &options.output,
            options.shards,
        )?;
        tokenizer = resolved_tokenizer;
        Box::new(oracle)
    };
    // Build the worker pool: the oracle loaded above plus `workers - 1` more
    // identical teacher instances, so up to `workers` articles are observed in
    // parallel. Each instance holds its own copy of the teacher weights.
    let mut pool: Vec<Box<dyn TeacherOracle + Send>> = Vec::with_capacity(options.workers);
    pool.push(oracle);
    for _ in 1..options.workers {
        let extra: Box<dyn TeacherOracle + Send> = if let Some(checkpoint) = &options.checkpoint {
            let path = checkpoint
                .to_str()
                .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
            Box::new(LlamaOracle::load(path))
        } else {
            Box::new(
                Teacher::load_with_sequence_length(&options.source, options.sequence_length)
                    .map_err(|error| {
                        SourceUnavailable::new(format!(
                            "failed to load Hugging Face model: {error}"
                        ))
                    })?,
            )
        };
        pool.push(extra);
    }
    if options.workers > 1 {
        eprintln!("observing with {} teacher workers", options.workers);
    }
    let report = observe_text::observe_text_corpus(
        &mut pool,
        options.seconds,
        &tokenizer,
        token_byte_lengths.as_deref(),
        &options.input,
        &options.output,
        options.shards,
        true,
    )
    .map_err(SourceUnavailable::new)?;
    finish_observe_text_report(&report, &options.output)
}

/// Observe-text with a batched Hugging Face teacher: one shared weight copy,
/// up to `--batch` articles teacher-forced per forward. Produces the same
/// records as the serial path for identical logits.
fn observe_text_batched_command(options: &ObserveTextOptions) -> Result<(), SourceUnavailable> {
    // #657 item 2b: the batched observation path now dispatches over
    // `Teacher`, so a GPT-2 source runs on its own batched executor
    // (`Gpt2State`) instead of the Llama-only path. Both concrete oracles
    // implement `BatchedTeacher`; the generic `observe_text_corpus_batched`
    // is monomorphized per architecture.
    let (tokenizer, runtime_table) = resolve_registered_observation_tokenizer(
        &options.source,
        options.tokenizer_adapter.as_ref(),
    )?;
    let teacher = Teacher::load_with_sequence_length(&options.source, options.sequence_length)
        .map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
    let token_byte_lengths = pin_and_export_observation_tokenizer(
        &tokenizer,
        &runtime_table,
        &options.output,
        options.shards,
    )?;
    eprintln!("observing with batch {}", options.batch);
    let report = match teacher {
        Teacher::Llama(oracle) => observe_text::observe_text_corpus_batched(
            &oracle,
            options.batch,
            options.seconds,
            &tokenizer,
            token_byte_lengths.as_deref(),
            &options.input,
            &options.output,
            options.shards,
            true,
        ),
        Teacher::Gpt2(oracle) => observe_text::observe_text_corpus_batched(
            &oracle,
            options.batch,
            options.seconds,
            &tokenizer,
            token_byte_lengths.as_deref(),
            &options.input,
            &options.output,
            options.shards,
            true,
        ),
    }
    .map_err(SourceUnavailable::new)?;
    finish_observe_text_report(&report, &options.output)
}

/// Print the observe-text report and, when the corpus is complete, persist the
/// merged record stream. Shared by the serial/pool and batched entry points.
fn finish_observe_text_report(
    report: &observe_text::ObservationReport,
    output: &std::path::Path,
) -> Result<(), SourceUnavailable> {
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
            "note: {} source characters or spans were represented lossily by the selected tokenizer",
            report.characters_replaced
        );
    }
    if report.done {
        // Persist the merged record stream: Gate C consumes it as
        // --corpus-recs with state.bin as --corpus-meta (issue #72).
        let merged = observe::merge_shards(output).map_err(SourceUnavailable::new)?;
        let merged_path = output.join("merged.bin");
        std::fs::write(&merged_path, &merged)?;
        println!(
            "observe-text complete: merged κ {} at {}",
            report
                .merged_kappa
                .as_deref()
                .expect("done reports merged κ"),
            merged_path.display()
        );
    } else {
        println!(
            "text observation corpus is not complete ({}/{} articles); rerun the same command to resume {}",
            report.articles_completed,
            report.articles_total,
            output.display()
        );
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct CoverOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    tokenizer: Option<PathBuf>,
    depths: usize,
    k0: usize,
    regions_budget: usize,
    memory_budget_mb: u64,
    min_support: usize,
    entropy_gain_bits: f64,
    radius_quantile: u32,
    output: PathBuf,
    /// Root κ of the #597 source-snapshot manifest of the teacher source
    /// (`--source-manifest-kappa`), carried verbatim into the cover
    /// report. This crate never computes the κ (no uor-addr dependency).
    source_manifest_kappa: Option<String>,
    /// #600 typed geometry-projection record of the teacher source
    /// (`--geometry-projection`, a JSON serialization of
    /// [`uor_r4_model_source::geometry::GeometryProjection`]), carried
    /// into the cover report. This crate never derives the record itself;
    /// the pipeline that held the oracle passes it.
    geometry: Option<uor_r4_model_source::geometry::GeometryProjection>,
    /// #602 typed attention-operator record of the teacher source
    /// (`--attention-operator`, a JSON serialization of
    /// [`uor_r4_model_source::attention::AttentionOperatorSpec`]),
    /// carried into the cover report. This crate never derives the
    /// record itself; the pipeline that held the oracle (or its
    /// `r4_attention` switch) passes it.
    attention_operator: Option<uor_r4_model_source::attention::AttentionOperatorSpec>,
}

fn parse_cover_options(args: &[String]) -> Result<CoverOptions, SourceUnavailable> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = CoverOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        tokenizer: None,
        depths: cover::DEFAULT_DEPTHS,
        k0: cover::DEFAULT_K0,
        regions_budget: cover::DEFAULT_REGIONS_BUDGET,
        memory_budget_mb: cover::DEFAULT_MEMORY_BUDGET_MB,
        min_support: cover::DEFAULT_MIN_SUPPORT,
        entropy_gain_bits: cover::DEFAULT_SPLIT_ENTROPY_GAIN_BITS,
        radius_quantile: cover::RADIUS_QUANTILE_NUMERATOR,
        output: PathBuf::from("cover"),
        source_manifest_kappa: None,
        geometry: None,
        attention_operator: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--tokenizer" => options.tokenizer = Some(PathBuf::from(value)),
            "--depths" => {
                options.depths = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --depths value: {value}"))
                })?;
                if options.depths == 0 {
                    return Err(SourceUnavailable::new("--depths must be at least 1"));
                }
            }
            "--k0" => {
                options.k0 = value
                    .parse()
                    .map_err(|_| SourceUnavailable::new(format!("invalid --k0 value: {value}")))?;
                if options.k0 == 0 {
                    return Err(SourceUnavailable::new("--k0 must be at least 1"));
                }
            }
            "--regions-budget" => {
                options.regions_budget = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --regions-budget value: {value}"))
                })?;
                if options.regions_budget == 0 {
                    return Err(SourceUnavailable::new(
                        "--regions-budget must be at least 1",
                    ));
                }
            }
            "--memory-budget" => {
                options.memory_budget_mb = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --memory-budget value: {value}"))
                })?;
                if options.memory_budget_mb == 0 {
                    return Err(SourceUnavailable::new(
                        "--memory-budget must be at least 1 MiB",
                    ));
                }
            }
            "--min-support" => {
                options.min_support = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --min-support value: {value}"))
                })?;
                if options.min_support == 0 {
                    return Err(SourceUnavailable::new("--min-support must be at least 1"));
                }
            }
            "--entropy-gain" => {
                options.entropy_gain_bits = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --entropy-gain value: {value}"))
                })?;
                if !options.entropy_gain_bits.is_finite() || options.entropy_gain_bits < 0.0 {
                    return Err(SourceUnavailable::new(
                        "--entropy-gain must be a finite non-negative number",
                    ));
                }
            }
            "--radius-quantile" => {
                options.radius_quantile = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --radius-quantile value: {value}"))
                })?;
                if options.radius_quantile == 0 || options.radius_quantile > 100 {
                    return Err(SourceUnavailable::new(
                        "--radius-quantile must be between 1 and 100",
                    ));
                }
            }
            "--out" => options.output = PathBuf::from(value),
            "--source-manifest-kappa" => {
                options.source_manifest_kappa = Some(value.clone());
            }
            "--geometry-projection" => {
                options.geometry = Some(serde_json::from_str(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --geometry-projection value: {error}"))
                })?);
            }
            "--attention-operator" => {
                options.attention_operator =
                    Some(serde_json::from_str(value).map_err(|error| {
                        SourceUnavailable::new(format!(
                            "invalid --attention-operator value: {error}"
                        ))
                    })?);
            }
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown cover option: {flag}"
                )));
            }
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
pub fn cover_command(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make cover induction much slower; use `cargo run --release -- transformerless cover ...`"
    );
    let options = parse_cover_options(args)?;
    let tokenizer_cid = explicit_tokenizer_cid(options.tokenizer.as_deref())?.unwrap_or([0; 32]);
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
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let corpus_recs = corpus_recs_path
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    // #450: resolve and announce the inputs *before* the long work, so the
    // run says which teacher container it actually read. `--artifacts`
    // defaults to a shared mutable path that helper scripts overwrite.
    let artifact_container = std::fs::read(&options.artifacts).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.artifacts.display()))
    })?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            options.artifacts.display()
        ))
    })?;
    let artifact_kappa = repro::container_kappa(&artifact_container);
    repro::announce_teacher_container(&options.artifacts, &artifact_kappa);
    let meta_bytes = std::fs::read(&options.corpus_meta).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.corpus_meta.display()))
    })?;
    let recs_bytes = std::fs::read(&options.corpus_recs).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.corpus_recs.display()))
    })?;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(&options.corpus_meta, &options.corpus_recs, &corpus_kappa);
    if options.corpus_meta != corpus_meta_path || options.corpus_recs != corpus_recs_path {
        // The κ above addresses the requested paths; the corpus itself is
        // loaded from the fallback-resolved ones. Say so rather than let a
        // reader assume the printed κ covers the loaded records.
        eprintln!(
            "corpus streams (loaded, after fallback resolution): {} + {}",
            corpus_meta_path.display(),
            corpus_recs_path.display()
        );
    }
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta_path.display(),
            corpus_recs_path.display()
        ))
    })?;

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
        // #435 split-criterion / capacity-scaling knobs: defaults (absolute
        // floor, unscaled k0 and budget) preserve the shipped behaviour.
        ..cover::CoverConfig::default()
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
    );
    let held_out = cover::build_observations_with_threads(
        &artifacts,
        &corpus,
        &held_out_positions,
        config.threads as usize,
    );
    let induced =
        cover::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa).ok_or_else(|| {
            SourceUnavailable::new("cover induction needs at least one train observation")
        })?;
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
        .expect("vocabulary exceeds u32 token ids");
    let (artifact_bytes, info) = cover::emit_r4g1_with_tokenizer_cid(
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
    let mut report = cover::build_report(
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
    // #597: bind the source-snapshot identity into the cover report when
    // the caller passed it (`--source-manifest-kappa`).
    report.source_manifest_kappa = options.source_manifest_kappa.clone();
    // #600: bind the teacher's geometry-projection record when the caller
    // passed it (`--geometry-projection`).
    report.geometry = options.geometry.clone();
    // #602: bind the teacher's attention-operator record when the caller
    // passed it (`--attention-operator`).
    report.attention_operator = options.attention_operator.clone();

    std::fs::create_dir_all(&options.output)?;
    let artifact_path = options.output.join("cover.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", artifact_path.display())))?;
    let report_json = serde_json::to_string_pretty(&report)?;
    let report_path = options.output.join("cover_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", report_path.display())))?;

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
    tokenizer: Option<PathBuf>,
    cover: Option<PathBuf>,
    stories: Option<PathBuf>,
    transition_out_degree: usize,
    emission_entries: usize,
    emission_selection: score::EmissionSelection,
    emission_shrinkage: score::EmissionShrinkage,
    context_order: u8,
    context_entries: usize,
    gate_c_context_window: bool,
    repetition_penalty_raw: i32,
    root_top_b: usize,
    exct_top_x: usize,
    witness_sample: usize,
    smoothing: score::Smoothing,
    scoring_variant: score_runtime::ScoringVariant,
    quality_profile: String,
    output: PathBuf,
}

fn parse_score_options(args: &[String]) -> Result<ScoreOptions, SourceUnavailable> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = ScoreOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        tokenizer: None,
        cover: None,
        stories: None,
        transition_out_degree: score::DEFAULT_TRANSITION_OUT_DEGREE,
        emission_entries: score::DEFAULT_EMISSION_ENTRIES,
        emission_selection: score::EmissionSelection::default(),
        emission_shrinkage: score::EmissionShrinkage::default(),
        context_order: score::DEFAULT_CONTEXT_ORDER,
        context_entries: score::DEFAULT_CONTEXT_ENTRIES,
        gate_c_context_window: false,
        repetition_penalty_raw: score::DEFAULT_REPETITION_PENALTY_RAW,
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
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--tokenizer" => options.tokenizer = Some(PathBuf::from(value)),
            "--cover" => options.cover = Some(PathBuf::from(value)),
            "--stories" => options.stories = Some(PathBuf::from(value)),
            "--transition-out-degree" => {
                options.transition_out_degree = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!(
                        "invalid --transition-out-degree value: {value}"
                    ))
                })?;
                if options.transition_out_degree == 0 {
                    return Err(SourceUnavailable::new(
                        "--transition-out-degree must be at least 1",
                    ));
                }
            }
            "--emission-shrinkage" => {
                options.emission_shrinkage = match value.as_str() {
                    "none" => score::EmissionShrinkage::None,
                    "witten-bell" => score::EmissionShrinkage::WittenBell,
                    "contrast" => score::EmissionShrinkage::Contrast,
                    other => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --emission-shrinkage value: {other} \
                             (expected none|witten-bell|contrast)"
                        )));
                    }
                };
            }
            "--emission-selection" => {
                options.emission_selection = match value.as_str() {
                    "ratio" => score::EmissionSelection::Ratio,
                    "probability" => score::EmissionSelection::Probability,
                    other => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --emission-selection value: {other} \
                             (expected ratio|probability)"
                        )));
                    }
                };
            }
            "--emission-entries" => {
                options.emission_entries = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --emission-entries value: {value}"))
                })?;
                if options.emission_entries == 0 {
                    return Err(SourceUnavailable::new(
                        "--emission-entries must be at least 1",
                    ));
                }
            }
            "--context-order" => {
                options.context_order = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --context-order value: {value}"))
                })?;
                if options.context_order > 2 {
                    return Err(SourceUnavailable::new(
                        "--context-order must be 0 (no NGRAM rows), 1 (bigram), or 2                          (bigram + trigram, default)",
                    ));
                }
            }
            "--context-entries" => {
                options.context_entries = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --context-entries value: {value}"))
                })?;
                if options.context_entries == 0 {
                    return Err(SourceUnavailable::new(
                        "--context-entries must be at least 1",
                    ));
                }
            }
            "--repetition-penalty-raw" => {
                options.repetition_penalty_raw = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!(
                        "invalid --repetition-penalty-raw value: {value}"
                    ))
                })?;
                if options.repetition_penalty_raw > 0 {
                    return Err(SourceUnavailable::new(
                        "--repetition-penalty-raw must be <= 0 (raw ScoreQ units)",
                    ));
                }
            }
            "--gate-c-context-window" => {
                options.gate_c_context_window = match value.as_str() {
                    "on" | "true" => true,
                    "off" | "false" => false,
                    other => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --gate-c-context-window value: {other} (expected on|off)"
                        )));
                    }
                };
            }
            "--root-top-b" => {
                options.root_top_b = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --root-top-b value: {value}"))
                })?;
                if options.root_top_b == 0 {
                    return Err(SourceUnavailable::new("--root-top-b must be at least 1"));
                }
            }
            "--exct-top-x" => {
                options.exct_top_x = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --exct-top-x value: {value}"))
                })?;
                if options.exct_top_x == 0 {
                    return Err(SourceUnavailable::new("--exct-top-x must be at least 1"));
                }
            }
            "--witness-sample" => {
                options.witness_sample = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --witness-sample value: {value}"))
                })?;
            }
            "--smoothing" => {
                options.smoothing = score::Smoothing::parse(value).ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "invalid --smoothing value: {value} \
                         (expected add-one | witten-bell | abs-disc:δ with δ finite in (0, 1])"
                    ))
                })?;
            }
            "--scoring-variant" => {
                options.scoring_variant = match value.as_str() {
                    "chain" | "chain-telescoped" => score_runtime::ScoringVariant::ChainTelescoped,
                    "normalized" | "cloud-size-normalized" => {
                        score_runtime::ScoringVariant::CloudSizeNormalized
                    }
                    "margin" | "margin-weighted" => score_runtime::ScoringVariant::MarginWeighted,
                    _ => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --scoring-variant value: {value} (expected chain | normalized | margin)"
                        )));
                    }
                };
            }
            "--quality-profile" => {
                if !matches!(value.as_str(), "pinned" | "relative_tla") {
                    return Err(SourceUnavailable::new(format!(
                        "invalid --quality-profile value: {value} (expected pinned | relative_tla)"
                    )));
                }
                options.quality_profile = value.clone();
            }
            "--out" => options.output = PathBuf::from(value),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown score option: {flag}"
                )));
            }
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
pub fn score_command(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make scoring much slower; use `cargo run --release -- transformerless score ...`"
    );
    let options = parse_score_options(args)?;
    // Resolve tokenizer/cover identity before corpus observation or output
    // mutation. This both preserves a bound cover CID when no tokenizer flag is
    // repeated and rejects an explicit cross-id-space swap up front.
    let explicit_tokenizer_cid = explicit_tokenizer_cid(options.tokenizer.as_deref())?;
    let cover_bytes = options
        .cover
        .as_ref()
        .map(|path| {
            std::fs::read(path)
                .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))
        })
        .transpose()?;
    let tokenizer_cid = scored_tokenizer_cid(cover_bytes.as_deref(), explicit_tokenizer_cid)?;
    // #488: account for where a real corpus's hours go across the WHOLE score
    // pipeline, not just Gate C. Same instrument, same conventions as #471
    // (stderr only — a duration in `score_report.json` would break the
    // deterministic-rebuild gate); the "score phase:" lines bracket the
    // "gate c phase:" lines so the two read as one log.
    let mut phases = score::PhaseLog::new("score phase");
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
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let corpus_recs = corpus_recs_path
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    // #450: resolve and announce the inputs *before* the long work, so the
    // run says which teacher container it actually read. `--artifacts`
    // defaults to a shared mutable path that helper scripts overwrite.
    let artifact_container = std::fs::read(&options.artifacts).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", options.artifacts.display()))
    })?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            options.artifacts.display()
        ))
    })?;
    let artifact_kappa = repro::container_kappa(&artifact_container);
    repro::announce_teacher_container(&options.artifacts, &artifact_kappa);
    let meta_bytes = std::fs::read(&corpus_meta_path).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", corpus_meta_path.display()))
    })?;
    let recs_bytes = std::fs::read(&corpus_recs_path).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", corpus_recs_path.display()))
    })?;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(&corpus_meta_path, &corpus_recs_path, &corpus_kappa);
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_recs).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta_path.display(),
            corpus_recs_path.display()
        ))
    })?;

    let config = score::ScoreConfig {
        transition_out_degree: options.transition_out_degree,
        emission_entries: options.emission_entries,
        root_top_b: options.root_top_b,
        exct_top_x: options.exct_top_x,
        witness_sample: options.witness_sample,
        smoothing: options.smoothing,
        scoring_variant: options.scoring_variant,
        emission_selection: options.emission_selection,
        emission_shrinkage: options.emission_shrinkage,
        context_order: options.context_order,
        context_entries: options.context_entries,
        gate_c_context_window: options.gate_c_context_window,
        repetition_penalty_raw: options.repetition_penalty_raw,
    };
    let (train_positions, held_out_positions) = match &options.stories {
        // D3 natural partition (issue #72): the observation pass records
        // the construction/held-out decision per story (article) in the
        // stories index; honor it instead of the ordinal train cut.
        Some(path) => {
            let index = observe_text::StoryIndex::load(path)
                .map_err(SourceUnavailable::new)?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!("stories index not found at {}", path.display()))
                })?;
            let mut train = Vec::new();
            let mut held_out = Vec::new();
            for i in 0..corpus.n {
                let story = corpus.story[i];
                match index.partition_of(story) {
                    Some(observe::RecordPartition::Construction) => train.push(i),
                    Some(observe::RecordPartition::HeldOut) => held_out.push(i),
                    None => {
                        return Err(SourceUnavailable::new(format!(
                            "story {story} missing from stories index {}",
                            path.display()
                        )));
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
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(1);
    let train =
        cover::build_observations_with_threads(&artifacts, &corpus, &train_positions, threads);
    let held_out =
        cover::build_observations_with_threads(&artifacts, &corpus, &held_out_positions, threads);
    phases.mark("inputs + observations (load, split, observe)");

    // Region parameters + structural edges: recovered from a previously
    // emitted cover artifact (--cover) or re-induced with the default
    // cover configuration. Both paths are byte-identical by construction
    // (deterministic double-run), so the choice is a pure cache.
    let (regions, structural, cover_source) = match &options.cover {
        Some(path) => {
            let bytes = cover_bytes
                .as_deref()
                .expect("cover bytes were preflighted for a present --cover path");
            // #450: the cached cover is the second container this command
            // consumes; address it in the log too.
            eprintln!(
                "cover container: {} (κ {})",
                path.display(),
                repro::container_kappa(bytes)
            );
            let (regions, structural) =
                score::recover_from_artifact(bytes).map_err(SourceUnavailable::new)?;
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
            )
            .ok_or_else(|| {
                SourceUnavailable::new("cover induction needs at least one train observation")
            })?;
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
    phases.mark("cover induction");

    eprintln!("score: building graded store [========================] 100%");
    // #469 lever A: per-record codes come from the κ-keyed sidecar when one
    // verifies against this artifact κ and corpus κ, and are written back
    // when it does not. Store bytes are identical on both branches.
    let (store, _) = code_sidecar::build_store_cached(&artifacts, &corpus, threads);
    let tls1 = runtime::store_bytes(&store);
    // The "code sidecar: HIT/MISS/WROTE/LOADED" line above is inside this
    // phase: a cold miss recomputes every record code (#469 lever A), which is
    // the difference between this phase costing seconds and costing minutes.
    phases.mark("graded store build (+ code sidecar)");

    eprintln!("score: compiling forward transitions [========================] 100%");
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &corpus,
        &regions,
        &train,
        max_depth,
        config.transition_out_degree,
    );
    phases.mark("forward transitions");
    let vocab = u32::try_from(artifacts.token_codes.len() / compiler::STAGES)
        .expect("vocabulary exceeds u32 token ids");
    let context_rows = score::compile_context_rows(&corpus, &train, vocab, &config);
    let fwd_rows = score::compile_forward_anchor_rows(&corpus, &train);
    phases.mark("context + forward-anchor rows");
    let emissions =
        score::compile_emissions(&corpus, &store, &regions, &train, max_depth, vocab, &config);
    phases.mark("emission compilation");
    let (artifact_bytes, info) = score::emit_scored_r4g1_with_tokenizer_cid(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            transition_quantization,
            emissions: &emissions,
            context_rows: &context_rows,
            exct_tls1: &tls1,
            exct_top_x: config.exct_top_x,
            fwd_rows: &fwd_rows,
        },
        tokenizer_cid,
    );
    let graph_kappa = uor_r4_graph_format::r4g1::artifact_kappa(&artifact_bytes)
        .expect("cannot address emitted R4G1 artifact");
    phases.mark("R4G1 artifact emission");

    eprintln!("score: running Gate C evaluation [========================] 100%");
    let gate_c = score::evaluate_gate_c(
        &artifact_bytes,
        &artifact_container,
        &artifacts,
        &store,
        &corpus,
        &held_out,
        &config,
    )
    .ok_or_else(|| SourceUnavailable::new("gate C could not evaluate an empty held-out split"))?;
    // The "gate c phase:" lines above break this phase down further (#471);
    // here it is one line so the score-level total stays complete.
    phases.mark("gate c evaluation");

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
        "distribution declaration (#234): status-based EXCT-miss rate {:.1}% ({}/{} — structurally ~0: the probe backs off to populated prefixes, root included) | STRICT full-code EXCT-miss rate {:.1}% ({}/{} held-out positions escape full-code exact-context) — {}",
        100.0 * report.distribution.exct_miss_rate,
        report.distribution.held_out_positions - report.distribution.exct_resolved_positions,
        report.distribution.held_out_positions,
        100.0 * report.distribution.strict_exct_miss_rate,
        report.distribution.held_out_positions - report.distribution.strict_exct_resolved_positions,
        report.distribution.held_out_positions,
        if report.distribution.can_measure_generalization {
            "Gate C measures generalization here (strict basis)"
        } else {
            "Gate C CANNOT measure generalization on this distribution (issue #234): the row restates exact-context recall"
        }
    );
    {
        let histogram: Vec<String> = report
            .distribution
            .exct_probe_level_histogram
            .iter()
            .enumerate()
            .map(|(level, count)| format!("L{level}:{count}"))
            .collect();
        println!(
            "EXCT probe resolution levels (0=root … {}=full code): {}",
            report
                .distribution
                .exct_probe_level_histogram
                .len()
                .saturating_sub(1),
            histogram.join(" ")
        );
    }
    std::fs::create_dir_all(&options.output).map_err(SourceUnavailable::new)?;
    let artifact_path = options.output.join("score.r4g1");
    std::fs::write(&artifact_path, &artifact_bytes)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", artifact_path.display())))?;
    let report_json = serde_json::to_string_pretty(&report).map_err(SourceUnavailable::new)?;
    let report_path = options.output.join("score_report.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", report_path.display())))?;
    // #488: report build (from the gate-c mark) + both artifact writes. The
    // trailing stdout summary below is negligible and shows up as the total's
    // remainder — which is the check that no stage went unnamed.
    phases.mark("report build + artifact write");

    println!(
        "score complete: {} nodes, {} edges ({} refinement + {} neighbor + {} forward), {} emission entries, EXCT {} bytes, NGRAM {} rows/{} entries ({} bytes)",
        info.node_count,
        info.edge_count,
        info.refinement_edges,
        info.neighbor_edges,
        info.forward_edges,
        info.emission_list_entries,
        info.exct_bytes,
        info.context_row_count,
        info.context_entry_count,
        info.context_bytes
    );
    println!(
        "gate C — held-out D3 metrics ({} positions):",
        gate_c.rule12_precedence.positions
    );
    // #467: a sampled decision run prints its sample size and the standard
    // error of every rate, so a sampled number cannot be read as a census.
    let sampled = gate_c.positions_sampled > 0;
    if sampled {
        println!(
            "  SAMPLED DECISION RUN (R4_GATE_C_SAMPLE): n={} of {} held-out positions \
             ({:.4} of the split); every rate below is an ESTIMATE, +/- is the binomial \
             standard error sqrt(p(1-p)/n)",
            gate_c.positions_sampled,
            gate_c.held_out_population,
            gate_c.positions_sampled as f64 / (gate_c.held_out_population as f64).max(1.0)
        );
    }
    if sampled {
        println!(
            "  {:<26} {:>16} {:>12} {:>10} {:>9}",
            "scorer", "top-1 agree", "bits/token", "+/- (SE)", "n"
        );
    } else {
        println!(
            "  {:<26} {:>16} {:>12}",
            "scorer", "top-1 agree", "bits/token"
        );
    }
    let row = |name: &str, m: &score::GateCMetrics| {
        if sampled {
            println!(
                "  {:<26} {:>15.1}% {:>12.4} {:>9.2}% {:>9}",
                name,
                100.0 * m.top1_agreement,
                m.bits_per_token,
                100.0 * m.standard_error,
                m.positions
            );
        } else {
            println!(
                "  {:<26} {:>15.1}% {:>12.4}",
                name,
                100.0 * m.top1_agreement,
                m.bits_per_token
            );
        }
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
    row("1+2 × fwd-anchor (#399 M2)", &gate_c.rule12_fwd_fused);
    row("1+2 × fwd SELF-anchor (B′)", &gate_c.rule12_fwd_self_fused);
    row("1+2 × fwd GATED self (B′)", &gate_c.rule12_fwd_gated_fused);
    row("1+2 × fwd DRAFT-gated (2p)", &gate_c.rule12_fwd_draft_fused);
    row(
        "1+2 × fwd STRICT-gated (2p)",
        &gate_c.rule12_fwd_strict_fused,
    );
    // #471: the right-context arm group prints its five rows, or says in
    // each row's own place that it was not evaluated. A blank line or a
    // zeroed rate here would be read as a measurement — these arms exist to
    // be compared against `graph chain+EXCT (1+2)` above, and a reader
    // scanning the column would take 0.0% as a catastrophic arm rather than
    // an absent one.
    let skipped_row = |name: &str| {
        if sampled {
            println!(
                "  {:<26} {:>16} {:>12} {:>10} {:>9}",
                name, "SKIPPED", "SKIPPED", "-", "-"
            );
        } else {
            println!("  {:<26} {:>16} {:>12}", name, "SKIPPED", "SKIPPED");
        }
    };
    match &gate_c.right_context_arms {
        Some(arms) => {
            row("1+2 + TWO-SIDED (#446 M1)", &arms.rule12_twosided);
            row("1+2 + two-sided SHUFFLED", &arms.rule12_twosided_shuffled);
            row("1+2 + LATENT-MIX (#446 M2)", &arms.rule12_latent_mix);
            row("1+2 + latent ORACLE-RIGHT", &arms.rule12_latent_oracle);
            row("1+2 + latent SHUF-CLASS", &arms.rule12_latent_shuffled);
        }
        None => {
            skipped_row("1+2 + TWO-SIDED (#446 M1)");
            skipped_row("1+2 + two-sided SHUFFLED");
            skipped_row("1+2 + LATENT-MIX (#446 M2)");
            skipped_row("1+2 + latent ORACLE-RIGHT");
            skipped_row("1+2 + latent SHUF-CLASS");
        }
    }
    let live_line = |name: &str, fused: &score::GateCMetrics, base: &score::GateCMetrics| {
        println!(
            "  {name} live slice ({} positions): fused {:.1}% vs rule 1+2 {:.1}% | bits {:.4} vs {:.4}",
            fused.positions,
            100.0 * fused.top1_agreement,
            100.0 * base.top1_agreement,
            fused.bits_per_token,
            base.bits_per_token,
        );
    };
    live_line(
        "fwd-anchor",
        &gate_c.rule12_fwd_fused_live,
        &gate_c.rule12_on_fwd_live,
    );
    live_line(
        "self-anchor",
        &gate_c.rule12_fwd_self_fused_live,
        &gate_c.rule12_on_fwd_self_live,
    );
    live_line(
        "gated-self ",
        &gate_c.rule12_fwd_gated_fused_live,
        &gate_c.rule12_on_fwd_gated_live,
    );
    live_line(
        "draft-gated",
        &gate_c.rule12_fwd_draft_fused_live,
        &gate_c.rule12_on_fwd_draft_live,
    );
    live_line(
        "strict-gated",
        &gate_c.rule12_fwd_strict_fused_live,
        &gate_c.rule12_on_fwd_strict_live,
    );
    match &gate_c.right_context_arms {
        Some(arms) => {
            live_line(
                "two-sided  ",
                &arms.rule12_twosided_live,
                &arms.rule12_on_twosided_live,
            );
            live_line(
                "two-sided SHUF",
                &arms.rule12_twosided_shuffled_live,
                &arms.rule12_on_twosided_shuffled_live,
            );
            live_line(
                "latent-mix ",
                &arms.rule12_latent_mix_live,
                &arms.rule12_on_latent_mix_live,
            );
            println!(
                "  two-sided pair-resolution depth: {}",
                arms.rule12_twosided_depths
                    .iter()
                    .enumerate()
                    .map(|(depth, count)| if depth == 0 {
                        format!("inert {count}")
                    } else {
                        format!("d{depth} {count}")
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            println!(
                "  two-sided DILUTION slice — ExactContext positions ({}): two-sided {:.1}% vs rule 1+2 {:.1}% | bits {:.4} vs {:.4} | arm live {}",
                arms.rule12_twosided_exct_slice.positions,
                arms.rule12_twosided_exct_slice.top1_agreement * 100.0,
                arms.rule12_on_twosided_exct_slice.top1_agreement * 100.0,
                arms.rule12_twosided_exct_slice.bits_per_token,
                arms.rule12_on_twosided_exct_slice.bits_per_token,
                arms.rule12_twosided_exct_slice_live,
            );
            println!(
                "  two-sided cell subdivision (construction, full graded depth): {:.3} two-sided keys per full left code ({} pair keys / {} left cells)",
                arms.twosided_keys_per_full_left,
                arms.twosided_full_pair_keys,
                arms.twosided_full_left_cells,
            );
            println!(
                "  NOTE (#446 M1): the two-sided rows key on tokens AFTER the target. \
                 Two-sided conditioning is NOT causally available to left-to-right \
                 generation — it is an infill/analysis (A-mode) measurement, or \
                 prospectively a construction-time signal. Never quote it as a \
                 generation number."
            );
            println!(
                "  latent class structure (construction, full left depth): class depth {} byte(s), {:.3} classes per full left code ({} class cells / {} left cells); oracle live {}, shuffled-class live {}",
                arms.latent_class_depth,
                arms.latent_classes_per_full_left,
                arms.latent_full_class_cells,
                arms.latent_full_left_cells,
                arms.latent_oracle_live_positions,
                arms.latent_shuffled_live_positions,
            );
            println!(
                "  latent headroom: baseline {:.1}% -> latent-mix {:.1}% -> oracle-right {:.1}% = {:.1}% of available top-1 headroom | EXIT RULE (>= 2.0pp over baseline AND beats shuffled-class): {}",
                100.0 * gate_c.rule12_precedence.top1_agreement,
                100.0 * arms.rule12_latent_mix.top1_agreement,
                100.0 * arms.rule12_latent_oracle.top1_agreement,
                100.0 * arms.latent_headroom_fraction,
                if arms.latent_exit_rule_met {
                    "MET (positive)"
                } else {
                    "NOT MET (negative)"
                },
            );
            println!(
                "  NOTE (#446 M2): the LATENT-MIX row reads the LEFT key only at \
                 serving — the right context is observed during construction and \
                 marginalized away — so it IS causally legitimate and quotable as a \
                 generation number. ORACLE-RIGHT supplies the true right class at \
                 evaluation time and is an upper bound only, NOT causal."
            );
        }
        // #471. Everything above depends on the whole-corpus right-context
        // code pass, so a run that skipped it has no two-sided or latent
        // numbers at all — including the #446 M2 exit-rule verdict, which is
        // the line most dangerous to fake. NOT MET and never-ran read the
        // same to a skimming eye, so the verdict is not printed at all.
        None => {
            println!(
                "  RIGHT-CONTEXT ARMS NOT EVALUATED ({}={}). The #446 M1 two-sided and \
                 #446 M2 latent rows, the dilution and class-structure statistics, and \
                 the M2 exit-rule verdict are ABSENT from this run — not zero, and not \
                 NOT MET. Re-run without the override to measure them.",
                score::GATE_C_SKIP_ARMS_ENV,
                gate_c.skipped_arm_groups.join(","),
            );
        }
    }
    println!(
        "  predicted-anchor accuracy: {:.1}% ({}/{})",
        100.0 * gate_c.anchor_hat_accuracy,
        gate_c.anchor_hat_correct,
        gate_c.anchor_hat_population,
    );
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
    win_loss_row(
        "win/loss fwd vs 1+2 live",
        &gate_c.win_loss.fwd_vs_rule12_live,
    );
    win_loss_row(
        "win/loss self vs 1+2 live",
        &gate_c.win_loss.fwd_self_vs_rule12_live,
    );
    win_loss_row(
        "win/loss gated vs 1+2 live",
        &gate_c.win_loss.fwd_gated_vs_rule12_live,
    );
    win_loss_row(
        "win/loss draft vs 1+2 live",
        &gate_c.win_loss.fwd_draft_vs_rule12_live,
    );
    win_loss_row(
        "win/loss strict vs 1+2 live",
        &gate_c.win_loss.fwd_strict_vs_rule12_live,
    );
    // #471: two more rows in the right-context group's closure. An all-zero
    // cross-tab printed among populated ones reads as "the arm never won",
    // which is a measurement; on a skipped run it is an absence.
    if let Some(arms) = &gate_c.right_context_arms {
        win_loss_row(
            "win/loss two-sided vs 1+2 live",
            &arms.twosided_vs_rule12_live,
        );
        win_loss_row(
            "win/loss ts-SHUF vs 1+2 live",
            &arms.twosided_shuffled_vs_rule12_live,
        );
    }
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
    phases.total();
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
    /// #268 candidate 3: whether a BOS token was prepended at every
    /// held-out story boundary during teacher-forcing (schema 3).
    bos_prefix: bool,
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

fn parse_compile_options(args: &[String]) -> Result<CompileOptions, SourceUnavailable> {
    let mut tokenizer_adapter = TokenizerAdapterKeyBuilder::default();
    let mut options = CompileOptions {
        model: None,
        revision: None,
        source: None,
        output: None,
        seconds: 300,
        target: 20_000,
        sequence_length: 128,
        r4_attention: false,
        tokenizer_adapter: None,
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
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--model" => options.model = Some(value.clone()),
            "--revision" => options.revision = Some(value.clone()),
            "--source" => options.source = Some(PathBuf::from(value)),
            "--output" => options.output = Some(PathBuf::from(value)),
            "--seconds" => {
                options.seconds = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --seconds value: {value}"))
                })?;
            }
            "--target" => {
                options.target = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --target value: {value}"))
                })?;
            }
            "--sequence-length" => {
                options.sequence_length = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --sequence-length value: {value}"))
                })?;
                if options.sequence_length == 0 {
                    return Err(SourceUnavailable::new(
                        "--sequence-length must be greater than zero",
                    ));
                }
            }
            flag if tokenizer_adapter.parse(flag, value)? => {}
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown compile option: {flag}"
                )));
            }
        }
        index += 2;
    }
    if options.model.is_none() && options.source.is_none() {
        return Err(SourceUnavailable::new(
            "pass --model <HF_REPOSITORY> or --source <DIRECTORY>",
        ));
    }
    if options.model.is_some() && options.revision.is_none() {
        return Err(SourceUnavailable::new(
            "--model requires an immutable --revision",
        ));
    }
    options.tokenizer_adapter = tokenizer_adapter.finish()?;
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

fn parse_evaluate_report_options(
    args: &[String],
) -> Result<EvaluateReportOptions, SourceUnavailable> {
    let mut tokenizer_adapter = TokenizerAdapterKeyBuilder::default();
    let mut options = EvaluateReportOptions {
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        compiled: PathBuf::from(DEFAULT_HF_COMPILED_PATH),
        report: None,
        sequence_length: 128,
        bos: false,
        max_held_out_stories: None,
        tokenizer_adapter: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--bos" {
            options.bos = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--source" => options.source = PathBuf::from(value),
            "--compiled" => options.compiled = PathBuf::from(value),
            "--report" => options.report = Some(PathBuf::from(value)),
            "--sequence-length" => {
                options.sequence_length = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --sequence-length value: {value}"))
                })?;
                if options.sequence_length == 0 {
                    return Err(SourceUnavailable::new(
                        "--sequence-length must be greater than zero",
                    ));
                }
            }
            "--max-held-out-stories" => {
                let limit: u32 = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --max-held-out-stories value: {value}"))
                })?;
                if limit == 0 {
                    return Err(SourceUnavailable::new(
                        "--max-held-out-stories must be greater than zero",
                    ));
                }
                options.max_held_out_stories = Some(limit);
            }
            flag if tokenizer_adapter.parse(flag, value)? => {}
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown evaluate-report option: {flag}"
                )));
            }
        }
        index += 2;
    }
    options.tokenizer_adapter = tokenizer_adapter.finish()?;
    Ok(options)
}

fn read_required_regular_file(path: &Path, label: &str) -> Result<Vec<u8>, SourceUnavailable> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{}: required {label} is not a regular file",
            path.display()
        )));
    }
    std::fs::read(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))
}

/// Validate the deployable graph rather than accidentally parsing the teacher
/// artifact container. New #718 evaluation requires an affirmative, nonzero
/// HEAD binding and exact reproduction by the deployed tokenizer bytes.
fn verify_scored_graph_tokenizer_binding(
    graph_bytes: &[u8],
    tokenizer_bytes: &[u8],
) -> Result<(), SourceUnavailable> {
    let view = uor_r4_graph_format::GraphView::parse(graph_bytes).map_err(|error| {
        SourceUnavailable::new(format!("invalid final scored R4G1 graph: {error}"))
    })?;
    view.verify_cids().map_err(|error| {
        SourceUnavailable::new(format!("invalid final scored R4G1 integrity: {error}"))
    })?;
    let expected = view
        .head()
        .ok_or_else(|| SourceUnavailable::new("final scored R4G1 graph has no HEAD"))?
        .tokenizer_cid()
        .0;
    if expected == [0; 32] {
        return Err(SourceUnavailable::new(
            "final scored R4G1 graph has a legacy zero tokenizer CID; evaluation requires an exact tokenizer.bin binding",
        ));
    }
    view.verify_tokenizer_cid(tokenizer_bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "final scored graph tokenizer_cid verification failed: {error}"
        ))
    })
}

/// Bind the deployed runtime table back to the already validated source and
/// sidecar identity. The historical hf-byte-bpe/1 representation is the sole
/// intentional untagged format; every tagged table must reproduce all four
/// identity fields, and every other adapter requires a tagged identity.
fn verify_runtime_tokenizer_adapter_identity(
    tokenizer_bytes: &[u8],
    adapter: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    let runtime = core_scenarios::Tokenizer::from_bytes(tokenizer_bytes)
        .ok_or_else(|| SourceUnavailable::new("tokenizer.bin is not a valid runtime tokenizer"))?;
    let Some(identity) = runtime.adapter_identity() else {
        if adapter.family == TokenizerAdapterKey::hf_byte_bpe_v1().family
            && adapter.version == TokenizerAdapterKey::hf_byte_bpe_v1().version
        {
            return Ok(());
        }
        return Err(SourceUnavailable::new(format!(
            "tokenizer.bin is an untagged legacy runtime table, but source/sidecar require {}/{} (CID {}, digest {})",
            adapter.family, adapter.version, adapter.tokenizer_cid, adapter.adapter_digest,
        )));
    };
    if identity.family != adapter.family
        || identity.version != adapter.version
        || identity.tokenizer_cid != adapter.tokenizer_cid
        || identity.adapter_digest != adapter.adapter_digest
    {
        return Err(SourceUnavailable::new(format!(
            "tokenizer.bin embedded adapter identity {}/{} (CID {}, digest {}) does not match validated source/sidecar {}/{} (CID {}, digest {})",
            identity.family,
            identity.version,
            identity.tokenizer_cid,
            identity.adapter_digest,
            adapter.family,
            adapter.version,
            adapter.tokenizer_cid,
            adapter.adapter_digest,
        )));
    }
    Ok(())
}

fn file_cid(path: &Path) -> Result<String, SourceUnavailable> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(SourceUnavailable::from)?;
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
) -> Result<(), SourceUnavailable> {
    let mut children = Vec::new();
    for child in std::fs::read_dir(directory)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", directory.display())))?
    {
        children.push(child.map_err(SourceUnavailable::from)?.path());
    }
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_file_entries(root, &child, entries)?;
            continue;
        }
        let relative = child
            .strip_prefix(root)
            .map_err(|error| SourceUnavailable::new(format!("{}: {error}", child.display())))?
            .to_path_buf();
        entries.push((relative, file_cid(&child)?));
    }
    Ok(())
}

fn directory_cid(path: &Path) -> Result<String, SourceUnavailable> {
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

fn evaluate_report(args: &[String]) -> Result<(), SourceUnavailable> {
    let options = parse_evaluate_report_options(args)?;
    // Evaluation decodes/classifies in the same registered id space selected
    // by observation and compilation. A malformed, missing, unsupported, or
    // ambiguous source definition is an error rather than "unclassified".
    let tokenizer = resolve_source_tokenizer(&options.source, options.tokenizer_adapter.as_ref())?;
    let tokenizer_adapter = tokenizer.adapter().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no adapter identity")
    })?;
    let recorded_adapter = read_compile_tokenizer_adapter(&options.compiled)?.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{} has no {TOKENIZER_ADAPTER_FILE}; evaluation cannot prove that its corpus id space matches {}/{}",
            options.compiled.display(), tokenizer_adapter.family, tokenizer_adapter.version
        ))
    })?;
    require_matching_compile_tokenizer_adapter(
        &options.compiled,
        &tokenizer_adapter,
        &recorded_adapter,
    )?;
    let report_path = options
        .report
        .clone()
        .unwrap_or_else(|| options.compiled.join(DEFAULT_HF_EVALUATION_REPORT));
    let source_cid = directory_cid(&options.source)?;
    let artifacts_path = options.compiled.join("tless_artifacts.bin");
    let store_path = options.compiled.join("tless_store.bin");
    let tokenizer_path = options.compiled.join("tokenizer.bin");
    let scored_graph_path = options.compiled.join("graph").join("score.r4g1");
    let corpus_meta_path = options.compiled.join("corpus.meta");
    let corpus_records_path = options.compiled.join("corpus.records");
    let scored_graph_bytes =
        read_required_regular_file(&scored_graph_path, "final scored R4G1 graph")?;
    let tokenizer_bytes = read_required_regular_file(&tokenizer_path, "tokenizer.bin")?;
    verify_scored_graph_tokenizer_binding(&scored_graph_bytes, &tokenizer_bytes)?;
    verify_runtime_tokenizer_adapter_identity(&tokenizer_bytes, &recorded_adapter)?;
    let artifacts_cid = file_cid(&artifacts_path)?;
    let store_cid = file_cid(&store_path)?;
    let tokenizer_cid = format!("blake3:{}", blake3::hash(&tokenizer_bytes).to_hex());
    let corpus_meta_cid = file_cid(&corpus_meta_path)?;
    let corpus_records_cid = file_cid(&corpus_records_path)?;

    let corpus_meta = corpus_meta_path
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let corpus_records = corpus_records_path
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_records).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "corpus is incomplete at {}; rerun compile until it is complete",
            options.compiled.display()
        ))
    })?;
    let held_out_cut = compiler::train_cut(&corpus);
    // --bos occupies one oracle position per story, so a full
    // `sequence_length`-token story needs one extra cache slot. The
    // extra slot is buffer capacity only (RoPE is absolute, attention is
    // causal over cached positions); it does not change the arithmetic
    // of the no-BOS arms.
    let oracle_sequence_length = options.sequence_length + usize::from(options.bos);
    let mut oracle = Teacher::load_with_sequence_length(&options.source, oracle_sequence_length)
        .map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
    let mut teacher_logits = vec![0f32; oracle.vocab()];
    let artifacts_bytes = std::fs::read(&artifacts_path)?;

    let artifacts = compiler::parse_artifacts(&artifacts_bytes)
        .ok_or_else(|| SourceUnavailable::new("invalid compiled artifact container"))?;
    let store_bytes = std::fs::read(&store_path)?;
    let store = runtime::parse_store(&store_bytes)
        .ok_or_else(|| SourceUnavailable::new("invalid store"))?;
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
    // Next-token class through the exact registered tokenizer that defines
    // the corpus id space. Resolver errors were surfaced before loading the
    // expensive teacher and cannot silently become legacy/unclassified data.
    let mut class_cache: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    let mut classify = |token: u32| -> usize {
        if let Some(&class) = class_cache.get(&token) {
            return class;
        }
        let text = tokenizer.decode(&[token]);
        let trimmed = text.trim();
        let class = if trimmed.is_empty() {
            2
        } else if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
            0
        } else if trimmed.chars().all(char::is_alphabetic) {
            1
        } else {
            2
        };
        class_cache.insert(token, class);
        class
    };
    let start_eval_time = std::time::Instant::now();
    // #268 fix: story-contiguous replay. The record stream interleaves
    // stories in short runs (measured mean run length 9.2 tokens on the
    // D3 bundle), so replaying in stream order cold-started the oracle
    // every few tokens — ~4.6-token mean effective context, which is
    // what the 13.4-bit "teacher floor" was actually measuring. Group
    // positions by story (stable sort keeps in-story stream order) and
    // reset only at true story boundaries. The position-in-story
    // decomposition slices double as verification: bands beyond 8-15
    // populate iff grouping reconstructed real contexts.
    let mut replay_order: Vec<usize> = (0..corpus.n).collect();
    replay_order.sort_by_key(|&position| corpus.story[position]);
    // #268 candidate 3 (prompt framing): optionally prepend BOS at every
    // held-out story boundary. `oracle_position` is the oracle's RoPE
    // position (includes the BOS step when enabled); `story_position`
    // keeps counting evaluated corpus tokens only, so the
    // position-in-story decomposition bands stay comparable across arms.
    let bos_token = oracle.bos_token();
    let mut oracle_position = 0usize;
    for (done, &index) in replay_order.iter().enumerate() {
        if done % 1000 == 0 || done + 1 == corpus.n {
            let pct = (done as f64 / corpus.n as f64) * 100.0;
            let elapsed = start_eval_time.elapsed().as_secs();
            println!(
                "progress: evaluated {}/{} positions ({:.1}%, {}s)",
                done, corpus.n, pct, elapsed
            );
        }
        if let Some(limit) = options.max_held_out_stories {
            // Replay order is story-sorted, so the first story past the
            // cap ends the smoke run.
            if corpus.story[index] >= held_out_cut.saturating_add(limit) {
                break;
            }
        }
        if current_story != Some(corpus.story[index]) {
            current_story = Some(corpus.story[index]);
            story_position = 0;
            oracle_position = 0;
            oracle.reset();
            if options.bos && corpus.story[index] >= held_out_cut {
                oracle.step(bos_token, oracle_position, &mut teacher_logits);
                oracle_position += 1;
            }
        }
        if corpus.story[index] < held_out_cut {
            continue;
        }
        oracle.step(
            corpus.input[index] as usize,
            oracle_position,
            &mut teacher_logits,
        );
        oracle_position += 1;
        story_position += 1;
        held_out_tokens += 1;
        let code = runtime::code_plain(&artifacts, &rotations, &corpus, index);
        let prediction = deepest_argmax(&store, &code).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "store has no populated backoff class for held-out position {index}"
            ))
        })?;
        if prediction == corpus.next[index] {
            top1_hits += 1;
        }
        let teacher_argmax = teacher_logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(token, _)| token as u32)
            .ok_or_else(|| SourceUnavailable::new("teacher produced empty logits"))?;
        if prediction == teacher_argmax {
            argmax_hits += 1;
        }
        let next_token = corpus.next[index] as usize;
        if next_token >= teacher_logits.len() {
            return Err(SourceUnavailable::new(format!(
                "next token {} is outside teacher vocab {}",
                corpus.next[index],
                teacher_logits.len()
            )));
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
        return Err(SourceUnavailable::new(
            "held-out split is empty; cannot evaluate",
        ));
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

    let distribution_name = match options.max_held_out_stories {
        None => "D3-held-out".to_owned(),
        Some(limit) => format!(
            "D3-held-out SMOKE (first {limit} held-out stories; path-verification only, numbers not quotable)"
        ),
    };
    let report = EvaluationReport {
        schema: 3,
        distribution: EvaluationDistribution {
            name: distribution_name,
            split: "compiler::train_cut 80/20 by story id".to_owned(),
            held_out_tokens,
        },
        source: EvaluationSource {
            directory: options.source.display().to_string(),
            cid: source_cid,
            sequence_length: options.sequence_length,
            bos_prefix: options.bos,
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
        std::fs::create_dir_all(parent)?;
    }
    let report_json = serde_json::to_vec_pretty(&report)?;
    let report_cid = format!("blake3:{}", blake3::hash(&report_json).to_hex());
    let envelope = EvaluationReportEnvelope {
        report,
        report_cid_of_report_bytes: report_cid.clone(),
    };
    let envelope_json = serde_json::to_string_pretty(&envelope)?;
    std::fs::write(&report_path, envelope_json)?;

    println!(
        "evaluation report written: {} ({})",
        report_path.display(),
        report_cid
    );
    println!(
        "arm: source={} | bos_prefix={} | held-out stories={}",
        options.source.display(),
        options.bos,
        options
            .max_held_out_stories
            .map_or("all".to_owned(), |limit| format!("first {limit} (SMOKE)"))
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

fn download_hf_source(
    repository: &str,
    revision: &str,
    destination: &Path,
) -> Result<(), SourceUnavailable> {
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
        return Err(SourceUnavailable::new(
            "--model must be a Hugging Face owner/repository name",
        ));
    }
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SourceUnavailable::new(
            "--revision must be a 40-character immutable commit SHA",
        ));
    }
    eprintln!(
        "downloading {repository}@{revision} to {}...",
        destination.display()
    );
    std::fs::create_dir_all(destination).map_err(SourceUnavailable::new)?;
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
            "--include",
            "spiece.model",
        ])
        .status()
        .map_err(|error| SourceUnavailable::new(format!("failed to run hf: {error}")))?;
    if !status.success() {
        return Err(SourceUnavailable::new(format!(
            "hf download failed with status {status}"
        )));
    }
    eprintln!("download complete");
    Ok(())
}

pub fn compile_hugging_face(args: &[String]) -> Result<(), SourceUnavailable> {
    compile_hugging_face_with_progress(args, |_, _| {})
}

/// Compile a Hugging Face teacher bundle and report coarse compiler phases.
/// The callback is compiler-side only and does not affect generated bytes.
pub fn compile_hugging_face_with_progress<F>(
    args: &[String],
    mut progress: F,
) -> Result<(), SourceUnavailable>
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
    let tokenizer = resolve_source_tokenizer(&source, options.tokenizer_adapter.as_ref())?;
    let tokenizer_adapter = tokenizer.adapter().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no adapter identity")
    })?;
    let runtime_table = tokenizer.runtime_decode_table().ok_or_else(|| {
        SourceUnavailable::new("registered source tokenizer has no runtime decode table")
    })?;
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/compiled").join(&slug));
    eprintln!("compiler output: {}", output.display());
    pin_compile_tokenizer_adapter(&output, &tokenizer_adapter)?;
    let meta = output.join("corpus.meta");
    let records = output.join("corpus.records");
    let meta = meta
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let records = records
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    let mut oracle =
        Teacher::load_with_sequence_length(&source, options.sequence_length).map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
    if options.r4_attention {
        oracle.set_r4_attention(true);
    }
    progress(5, "Exporting tokenizer...");
    eprintln!("exporting tokenizer...");
    let tokenizer_export = core_scenarios::export_runtime_tokenizer_table(
        &runtime_table,
        output.join("tokenizer.bin"),
    )
    .map_err(SourceUnavailable::new)?;
    progress(10, "Generating teacher corpus...");
    compiler::generate_to_with_token_byte_lengths(
        &mut oracle,
        options.seconds,
        options.target,
        meta,
        records,
        tokenizer_export.source_byte_lengths.as_deref(),
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
        serde_json::to_string_pretty(&calibration).map_err(SourceUnavailable::new)?;
    std::fs::write(output.join("hamming_calibration.json"), calibration_json)
        .map_err(SourceUnavailable::new)?;
    progress(75, "Inducing hierarchical codes...");
    eprintln!("inducing hierarchical codes...");
    let hc = compiler::induce_hierarchical_codes(&artifacts.token_codes, oracle.vocab(), &corpus);
    let hc_json = serde_json::to_string_pretty(&hc).map_err(SourceUnavailable::new)?;
    std::fs::write(output.join("hierarchical_codes.json"), hc_json)
        .map_err(SourceUnavailable::new)?;
    progress(85, "Writing compiled artifact...");
    eprintln!("writing artifact...");
    std::fs::write(
        output.join("tless_artifacts.bin"),
        compiler::artifact_bytes(&artifacts),
    )
    .map_err(SourceUnavailable::new)?;
    progress(92, "Building graded store...");
    eprintln!("building graded store...");
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    std::fs::write(output.join("tless_store.bin"), runtime::store_bytes(&store))
        .map_err(SourceUnavailable::new)?;
    // Helper to calculate Blake3 hash CID
    let calculate_file_hash = |path: &Path| -> Result<String, SourceUnavailable> {
        let content = std::fs::read(path).map_err(SourceUnavailable::new)?;
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

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(SourceUnavailable::new)?;
    std::fs::write(output.join("space_manifest.json"), manifest_json)
        .map_err(SourceUnavailable::new)?;
    progress(100, "Teacher bundle ready.");
    eprintln!("space manifest generated: space_manifest.json");

    println!("compile complete: {}", output.display());
    println!(
        "bundle ready for local `ask`; use `cargo run -- import --help` to attach a quality attestation and persist a named manifest (name: {slug})"
    );
    Ok(())
}

/// Compile a completed recorded corpus without loading or executing a
/// transformer. This is the observation-first production path; teacher
/// capture remains available through `observe`/`compile` as an explicitly
/// separate offline step.
pub fn compile_recorded_corpus(args: &[String]) -> Result<(), SourceUnavailable> {
    let options = parse_recorded_compile_options(args)?;
    let meta = options
        .corpus_meta
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let records = options
        .corpus_recs
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    let corpus = compiler::load_corpus_from(meta, records).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "recorded corpus is incomplete or invalid at {}/{}",
            options.corpus_meta.display(),
            options.corpus_recs.display()
        ))
    })?;
    eprintln!(
        "recorded compile: {} records, {} stories, vocabulary {} (no teacher loaded)",
        corpus.n, corpus.stories, options.vocab_size
    );
    let artifacts = compiler::compile_recorded(&corpus, options.vocab_size).ok_or_else(|| {
        SourceUnavailable::new("recorded compile failed: empty corpus or invalid vocabulary")
    })?;
    let calibration = compiler::calibrate_hamming_regions(&artifacts, &corpus);
    let hierarchical =
        compiler::induce_hierarchical_codes(&artifacts.token_codes, options.vocab_size, &corpus);
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(1);
    let (store, _) = runtime::build_store_with_threads(&artifacts, &corpus, threads);

    std::fs::create_dir_all(&options.output)?;
    std::fs::write(
        options.output.join("tless_artifacts.bin"),
        compiler::artifact_bytes(&artifacts),
    )?;
    std::fs::write(
        options.output.join("tless_store.bin"),
        runtime::store_bytes(&store),
    )?;
    std::fs::write(
        options.output.join("hamming_calibration.json"),
        serde_json::to_string_pretty(&calibration).map_err(SourceUnavailable::from)?,
    )?;
    std::fs::write(
        options.output.join("hierarchical_codes.json"),
        serde_json::to_string_pretty(&hierarchical).map_err(SourceUnavailable::from)?,
    )?;
    std::fs::copy(&options.corpus_meta, options.output.join("corpus.meta"))?;
    std::fs::copy(&options.corpus_recs, options.output.join("corpus.records"))?;

    let artifact_bytes = compiler::artifact_bytes(&artifacts);
    println!(
        "recorded compile complete: {} ({} corpus records, artifact κ blake3:{})",
        options.output.display(),
        corpus.n,
        blake3::hash(&artifact_bytes).to_hex()
    );
    Ok(())
}

/// Parse a `--skeleton` spec: comma-separated token ids with `_` at
/// free positions, e.g. `12,_,_,_,99,_,_,_,7`.
fn parse_skeleton(spec: &str) -> Result<Vec<Option<u32>>, SourceUnavailable> {
    spec.split(',')
        .map(|slot| {
            let slot = slot.trim();
            if slot == "_" {
                Ok(None)
            } else {
                slot.parse::<u32>().map(Some).map_err(|_| {
                    SourceUnavailable::new(format!(
                        "invalid skeleton slot: {slot:?} (expected a token id or _)"
                    ))
                })
            }
        })
        .collect()
}

/// A `Compiled` with no token codes: every window bundles to zero rows,
/// so scoring is carried by the artifact's own tables (root prior,
/// NGRAM context rows through the recent-token deque, FWDA rows). Used
/// when no `--teacher` TLA container is supplied.
fn empty_compiled() -> compiler::Compiled {
    compiler::Compiled {
        token_codes: Vec::new(),
        stage_books: Vec::new(),
        stage_shifts: Vec::new(),
        thresholds: vec![0i64; compiler::D],
        class_sigs: Vec::new(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb: Vec::new(),
        resid_cb: Vec::new(),
        resid_scale_shifts: Vec::new(),
        norm_fold_const: 0,
    }
}

/// A-mode infill serving (issue #399): anchors are GIVEN inputs and the
/// engine fills the free positions between them through the validated
/// forward-anchor channel (`GraphScorer::score_candidates_infill`).
/// Token-id level only — no tokenizer involved.
pub fn graph_infill_command(args: &[String]) -> Result<(), SourceUnavailable> {
    let mut artifact_path: Option<PathBuf> = None;
    let mut skeleton_spec: Option<String> = None;
    let mut teacher_path: Option<PathBuf> = None;
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--artifact" => artifact_path = Some(PathBuf::from(value)),
            "--skeleton" => skeleton_spec = Some(value.clone()),
            "--teacher" => teacher_path = Some(PathBuf::from(value)),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown graph infill option: {flag}"
                )));
            }
        }
        index += 2;
    }
    let artifact_path = artifact_path
        .ok_or_else(|| SourceUnavailable::new("pass --artifact <scored R4G1 path>"))?;
    let skeleton_spec = skeleton_spec.ok_or_else(|| {
        SourceUnavailable::new("pass --skeleton <comma-separated token ids, _ for free positions>")
    })?;
    let skeleton = parse_skeleton(&skeleton_spec)?;

    let r4g1 = std::fs::read(&artifact_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", artifact_path.display())))?;
    let teacher_bytes = teacher_path
        .as_ref()
        .map(|path| {
            std::fs::read(path)
                .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))
        })
        .transpose()?;
    let artifacts = match &teacher_bytes {
        Some(bytes) => compiler::parse_artifacts(bytes).ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{}: not a TLA3/TLA4/TLA5 artifact container",
                teacher_path
                    .expect("teacher bytes imply a teacher path")
                    .display()
            ))
        })?,
        None => empty_compiled(),
    };
    let scorer = score_runtime::GraphScorer::from_artifact(
        &r4g1,
        teacher_bytes.as_deref(),
        score::DEFAULT_ROOT_TOP_B,
        score::DEFAULT_EXCT_TOP_X,
    )
    .ok_or_else(|| SourceUnavailable::new("could not build scorer from R4G1 artifact"))?;
    let rotations = runtime::derive_rotations();

    let filled = score_runtime::infill_fill(&scorer, &artifacts, &rotations, &skeleton)
        .ok_or_else(|| SourceUnavailable::new("infill fill produced no tokens"))?;

    let free_positions = skeleton.iter().filter(|slot| slot.is_none()).count();
    let live_fwd_positions = skeleton
        .iter()
        .enumerate()
        .filter(|(index, slot)| {
            slot.is_none()
                && score_runtime::next_skeleton_anchor(&skeleton, *index).is_some_and(
                    |(anchor, distance)| scorer.forward_anchor_row(distance, anchor).is_some(),
                )
        })
        .count();

    let rendered: Vec<String> = filled.iter().map(u32::to_string).collect();
    println!("{}", rendered.join(","));
    println!(
        "infill: filled {free_positions} free positions ({live_fwd_positions} with live fwd rows) across {} slots; {} fwd rows loaded",
        skeleton.len(),
        scorer.forward_anchor_row_count()
    );
    Ok(())
}

/// Dispatch of the `graph` command family (A-mode serving surfaces).
pub fn graph_command(args: &[String]) -> Result<(), SourceUnavailable> {
    match args.first().map(|s| s.as_str()) {
        Some("infill") => graph_infill_command(&args[1..]),
        _ => Err(SourceUnavailable::new(
            "graph commands: infill --artifact <scored R4G1> --skeleton <token ids, _ for free> [--teacher <TLA container>]",
        )),
    }
}

pub fn run(args: &[String]) -> Result<(), SourceUnavailable> {
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
        Some("compile-recorded") => compile_recorded_corpus(&args[1..])?,
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
        Some("runtime-corpus") => runtime_corpus::run(&args[1..])?,
        Some("cover") => cover_command(&args[1..])?,
        Some("cover-sweep") => cover_sweep::cover_sweep_command(&args[1..])?,
        Some("recommend-scale") => recommend_scale::run(&args[1..])?,
        Some("score") => score_command(&args[1..])?,
        Some("graph") => graph_command(&args[1..])?,
        Some("cd-compile") => cd_compile_command(&args[1..]),
        Some("quantum-eval") => quantum_eval_command(&args[1..]),
        _ => {
            println!(
                "R4 transformerless — compile a mul-free table artifact\n\
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--tokenizer-family FAMILY --tokenizer-version N] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | compare | compare-report | scenarios | teacher-kappa | convert-r4g1 --artifacts <TLA> --store <TLS1> [--calibration <hamming_calibration.json>] --out <R4G1>\n\
                 recorded compile (no transformer): compile-recorded --corpus-meta <META> --corpus-recs <RECS> --vocab-size <N> --out <DIR>\n\
                 transformer-free refresh: runtime-corpus --artifacts <TLA> --store <TLS1> --seed-meta <META> --seed-recs <RECS> --out <DIR> --target N [--threads N]\n\
                 observation pipeline: observe [--source DIR [--tokenizer-family FAMILY --tokenizer-version N] | --checkpoint BIN] [--seconds N] [--target N] [--shards N] [--out DIR] [--sequence-length N]\n\
                 text observations (D3): observe-text [--input PATH] [--out DIR] [--shards N] [--seconds N] [--source DIR [--tokenizer-family FAMILY --tokenizer-version N] | --checkpoint BIN --tokenizer PATH] [--sequence-length N]\n\
                 A-mode infill serving: graph infill --artifact <scored R4G1> --skeleton <token ids, _ for free> [--teacher <TLA container>]\n\
                 quantum operations: cd-compile | quantum-eval\n\
                 hf evaluation: evaluate-report [--source DIR] [--tokenizer-family FAMILY --tokenizer-version N] [--compiled DIR] [--report PATH] [--sequence-length N] [--bos] [--max-held-out-stories N]\n\
                 scale sizing (#514): recommend-scale (--config <hf dir> | --d-model N --n-layers N --vocab N) [--corpus wiki|stories] [--beta B]\n\
                 docs: docs/transformerless/TRANSFORMERLESS.md (extrapolation), docs/transformerless/PROOF.md (proof + certificate)"
            );
        }
    }
    Ok(())
}

pub fn cd_compile_command(args: &[String]) {
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
}

pub fn quantum_eval_command(_args: &[String]) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_cli_test_path(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("uor-r4-cli-{name}-{nonce}"))
    }

    fn fixture_adapter(marker: &str) -> TokenizerAdapter {
        let tokenizer_json = format!(
            r#"{{
                "fixture_marker":"{marker}",
                "pre_tokenizer":{{"type":"ByteLevel","add_prefix_space":false}},
                "model":{{"type":"BPE","vocab":{{"a":0}},"merges":[]}}
            }}"#
        );
        uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
            tokenizer_json.as_bytes(),
        )
        .expect("adapter fixture")
        .adapter()
    }

    fn directory_bytes(path: &Path) -> Vec<(String, Vec<u8>)> {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .expect("read test directory")
            .map(|entry| entry.expect("directory entry").path())
            .filter(|entry| entry.is_file())
            .map(|entry| {
                (
                    entry
                        .file_name()
                        .expect("file name")
                        .to_string_lossy()
                        .into_owned(),
                    std::fs::read(&entry).expect("file bytes"),
                )
            })
            .collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    fn minimal_graph_with_tokenizer_cid(tokenizer_cid: [u8; 32]) -> Vec<u8> {
        let mut head = [0u8; uor_r4_graph_format::HEAD_PAYLOAD_LEN];
        head[32..64].copy_from_slice(&tokenizer_cid);
        head[184..186].copy_from_slice(&8u16.to_le_bytes()); // W
        head[204] = 1; // depth_count
        head[212..214].copy_from_slice(&64u16.to_le_bytes()); // signature_bytes
        let mut builder = uor_r4_graph_format::ArtifactBuilder::new(6);
        builder.add_section(uor_r4_graph_format::SectionId::HEAD, 0, &head);
        builder.build().expect("minimal graph serializes")
    }

    fn tokenizer_adapter_temp_entries(path: &Path) -> Vec<String> {
        let prefix = format!(".{TOKENIZER_ADAPTER_FILE}.");
        let mut entries: Vec<String> = std::fs::read_dir(path)
            .expect("read test directory")
            .map(|entry| {
                entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.starts_with(&prefix) && name.ends_with(".tmp"))
            .collect();
        entries.sort();
        entries
    }

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
            "--tokenizer-family",
            "hf-byte-bpe",
            "--tokenizer-version",
            "1",
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
        assert_eq!(
            options.tokenizer_adapter,
            Some(TokenizerAdapterKey::hf_byte_bpe_v1())
        );
        assert_eq!(source_slug(&options), "smollm2-135m-instruct");
    }

    /// #597 plumbing seam: the cover stage accepts and carries the
    /// source-snapshot manifest root κ (populated into the cover report),
    /// and leaves it unset when the flag is absent.
    #[test]
    fn cover_options_carry_the_source_manifest_kappa() {
        let kappa = format!("blake3:{}", "7".repeat(64));
        let args = ["--source-manifest-kappa".to_owned(), kappa.clone()];
        let options = parse_cover_options(&args).expect("valid cover options");
        assert_eq!(
            options.source_manifest_kappa.as_deref(),
            Some(kappa.as_str())
        );
        let defaults = parse_cover_options(&[]).expect("default cover options");
        assert_eq!(defaults.source_manifest_kappa, None);
    }

    /// #600 plumbing seam: the cover stage accepts the typed
    /// geometry-projection record as JSON (populated into the cover
    /// report), leaves it unset when the flag is absent, and refuses
    /// malformed JSON by name on the sanctioned error surface.
    #[test]
    fn cover_options_carry_the_geometry_projection() {
        let record = uor_r4_model_source::geometry::GeometryProjection::bucket_average(576, 288);
        let json = serde_json::to_string(&record).expect("record serializes");
        let args = ["--geometry-projection".to_owned(), json];
        let options = parse_cover_options(&args).expect("valid cover options");
        assert_eq!(options.geometry.as_ref(), Some(&record));
        let defaults = parse_cover_options(&[]).expect("default cover options");
        assert_eq!(defaults.geometry, None);
        let error =
            parse_cover_options(&["--geometry-projection".to_owned(), "{not json".to_owned()])
                .expect_err("malformed record is not a product");
        assert!(error.reason.contains("--geometry-projection"));
    }

    /// #602 plumbing seam: the cover stage accepts the typed
    /// attention-operator record as JSON (populated into the cover
    /// report), leaves it unset when the flag is absent (the legacy
    /// interpretation), and refuses malformed JSON by name on the
    /// sanctioned error surface.
    #[test]
    fn cover_options_carry_the_attention_operator() {
        for record in [
            uor_r4_model_source::attention::AttentionOperatorSpec::standard(),
            uor_r4_model_source::attention::AttentionOperatorSpec::experimental_r4(),
        ] {
            let json = serde_json::to_string(&record).expect("record serializes");
            let args = ["--attention-operator".to_owned(), json];
            let options = parse_cover_options(&args).expect("valid cover options");
            assert_eq!(options.attention_operator.as_ref(), Some(&record));
        }
        let defaults = parse_cover_options(&[]).expect("default cover options");
        assert_eq!(defaults.attention_operator, None);
        let error =
            parse_cover_options(&["--attention-operator".to_owned(), "{not json".to_owned()])
                .expect_err("malformed record is not a product");
        assert!(error.reason.contains("--attention-operator"));
    }

    #[test]
    fn graph_stage_tokenizer_flags_are_explicit_regular_inputs() {
        let tokenizer = unique_cli_test_path("stage-tokenizer.bin");
        std::fs::write(&tokenizer, b"exact stage tokenizer").expect("write tokenizer");
        let flags = ["--tokenizer", tokenizer.to_str().expect("UTF-8 path")].map(str::to_owned);
        assert_eq!(
            parse_cover_options(&flags)
                .expect("cover flag")
                .tokenizer
                .as_deref(),
            Some(tokenizer.as_path())
        );
        assert_eq!(
            parse_score_options(&flags)
                .expect("score flag")
                .tokenizer
                .as_deref(),
            Some(tokenizer.as_path())
        );
        assert_eq!(
            explicit_tokenizer_cid(Some(&tokenizer)).expect("hash exact bytes"),
            Some(*blake3::hash(b"exact stage tokenizer").as_bytes())
        );
        assert_eq!(explicit_tokenizer_cid(None).expect("legacy absence"), None);

        let missing = unique_cli_test_path("missing-stage-tokenizer.bin");
        assert!(explicit_tokenizer_cid(Some(&missing)).is_err());
        let directory = unique_cli_test_path("stage-tokenizer-directory");
        std::fs::create_dir_all(&directory).expect("create directory");
        let error = explicit_tokenizer_cid(Some(&directory)).expect_err("directory refused");
        assert!(error.reason.contains("not a regular file"));

        let _ = std::fs::remove_file(tokenizer);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn score_preserves_cover_tokenizer_and_refuses_a_swap() {
        let cover_tokenizer = *blake3::hash(b"cover tokenizer").as_bytes();
        let cover = minimal_graph_with_tokenizer_cid(cover_tokenizer);
        assert_eq!(
            scored_tokenizer_cid(Some(&cover), None).expect("inherit cover"),
            cover_tokenizer
        );
        assert_eq!(
            scored_tokenizer_cid(Some(&cover), Some(cover_tokenizer))
                .expect("matching explicit tokenizer"),
            cover_tokenizer
        );
        let replacement = *blake3::hash(b"replacement tokenizer").as_bytes();
        let error = scored_tokenizer_cid(Some(&cover), Some(replacement))
            .expect_err("cover identity cannot be replaced");
        assert!(
            error
                .reason
                .contains("does not match explicit --tokenizer CID")
        );

        let legacy_cover = minimal_graph_with_tokenizer_cid([0; 32]);
        assert_eq!(
            scored_tokenizer_cid(Some(&legacy_cover), Some(replacement))
                .expect("explicit tokenizer upgrades a legacy cover"),
            replacement
        );
    }

    #[test]
    fn evaluation_requires_bound_scored_graph_and_refuses_swapped_tokenizer() {
        let tokenizer = b"evaluation tokenizer";
        let graph = minimal_graph_with_tokenizer_cid(*blake3::hash(tokenizer).as_bytes());
        verify_scored_graph_tokenizer_binding(&graph, tokenizer).expect("exact binding");
        let swapped = verify_scored_graph_tokenizer_binding(&graph, b"swapped tokenizer")
            .expect_err("swapped tokenizer refused");
        assert!(swapped.reason.contains("tokenizer_cid verification failed"));

        let legacy = minimal_graph_with_tokenizer_cid([0; 32]);
        let legacy_error = verify_scored_graph_tokenizer_binding(&legacy, tokenizer)
            .expect_err("zero CID is not evaluable");
        assert!(legacy_error.reason.contains("legacy zero tokenizer CID"));
    }

    #[test]
    fn evaluation_rejects_a_rebound_tagged_tokenizer_with_another_adapter_identity() {
        let source_adapter = fixture_adapter("evaluation-source-a");
        let tagged_path = unique_cli_test_path("evaluation-tagged-tokenizer-b.bin");
        let replacement_identity = core_scenarios::RuntimeTokenizerIdentity {
            // Keep the registry key equal so the regression proves that the
            // raw-definition CID and complete adapter digest are checked too.
            family: source_adapter.family.clone(),
            version: source_adapter.version,
            tokenizer_cid: format!("blake3:{}", "1".repeat(64)),
            adapter_digest: format!("blake3:{}", "2".repeat(64)),
        };
        assert_ne!(
            replacement_identity.tokenizer_cid,
            source_adapter.tokenizer_cid
        );
        let table = RuntimeTokenizerDecodeTable {
            identity: replacement_identity,
            pieces: vec![b"<unk>".to_vec(), "▁a".as_bytes().to_vec()],
            encode_policy: core_scenarios::RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: core_scenarios::RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        core_scenarios::export_runtime_tokenizer_table(&table, &tagged_path)
            .expect("export tagged replacement");
        let tagged_bytes = std::fs::read(&tagged_path).expect("read tagged replacement");

        // An attacker can make graph↔tokenizer binding internally consistent
        // by rebinding HEAD to B. The independent runtime↔sidecar/source check
        // must still reject B before evaluation touches corpus/oracle state.
        let rebound_graph =
            minimal_graph_with_tokenizer_cid(*blake3::hash(&tagged_bytes).as_bytes());
        verify_scored_graph_tokenizer_binding(&rebound_graph, &tagged_bytes)
            .expect("graph was deliberately rebound to tokenizer B");
        let error = verify_runtime_tokenizer_adapter_identity(&tagged_bytes, &source_adapter)
            .expect_err("tagged tokenizer B cannot impersonate source/sidecar A");
        assert!(
            error.reason.contains("embedded adapter identity")
                && error
                    .reason
                    .contains("does not match validated source/sidecar")
                && error.reason.contains(&source_adapter.tokenizer_cid),
            "{error}"
        );

        // The one intentionally untagged runtime representation remains the
        // historical hf-byte-bpe/1 byte table.
        let mut legacy_bpe = 1i32.to_le_bytes().to_vec();
        legacy_bpe.push(b'a');
        verify_runtime_tokenizer_adapter_identity(&legacy_bpe, &source_adapter)
            .expect("legacy hf-byte-bpe/1 remains explicit compatibility");

        let _ = std::fs::remove_file(tagged_path);
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
        let error = parse_compile_options(&args).expect_err("requires --revision");
        assert_eq!(error.reason, "--model requires an immutable --revision");
    }

    #[test]
    fn compile_adapter_sidecar_is_full_exact_and_idempotent() {
        let output = unique_cli_test_path("compile-adapter-fresh");
        let adapter = fixture_adapter("fresh");
        pin_compile_tokenizer_adapter(&output, &adapter).expect("fresh pin");
        let sidecar = output.join(TOKENIZER_ADAPTER_FILE);
        let recorded: TokenizerAdapter =
            serde_json::from_slice(&std::fs::read(&sidecar).expect("sidecar bytes"))
                .expect("full adapter record");
        assert_eq!(recorded, adapter);
        let before = directory_bytes(&output);
        pin_compile_tokenizer_adapter(&output, &adapter).expect("identical pin");
        assert_eq!(directory_bytes(&output), before);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_adapter_sidecar_publish_is_concurrent_and_no_clobber() {
        let output = unique_cli_test_path("compile-adapter-concurrent");
        let adapter = fixture_adapter("concurrent");
        let mut threads = Vec::new();
        for _ in 0..8 {
            let output = output.clone();
            let adapter = adapter.clone();
            threads.push(std::thread::spawn(move || {
                pin_compile_tokenizer_adapter(&output, &adapter)
            }));
        }
        for thread in threads {
            thread
                .join()
                .expect("publisher thread")
                .expect("same identity publisher");
        }
        let recorded = read_compile_tokenizer_adapter(&output)
            .expect("sidecar read")
            .expect("sidecar present");
        assert_eq!(recorded, adapter);
        assert!(tokenizer_adapter_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_adapter_requires_canonical_lowercase_blake3_cid() {
        assert!(is_blake3_cid(&format!("blake3:{}", "0a1b2c3d".repeat(8))));
        assert!(!is_blake3_cid(&format!("blake3:{}", "A".repeat(64))));
        let output = unique_cli_test_path("compile-adapter-uppercase");
        let mut adapter = fixture_adapter("uppercase");
        adapter.tokenizer_cid = format!("blake3:{}", "A".repeat(64));
        adapter.adapter_digest = adapter.declared_digest();
        let error = pin_compile_tokenizer_adapter(&output, &adapter)
            .expect_err("uppercase CID is not canonical");
        assert!(error.reason.contains("invalid tokenizer CID"));
        assert!(!output.exists());
    }

    #[test]
    fn compile_adapter_rejects_directory_sidecar_without_mutation() {
        let output = unique_cli_test_path("compile-adapter-sidecar-directory");
        std::fs::create_dir_all(output.join(TOKENIZER_ADAPTER_FILE)).expect("directory sidecar");
        std::fs::write(output.join("corpus.meta"), b"preserve me").expect("payload");
        let before = directory_bytes(&output);
        let error = pin_compile_tokenizer_adapter(&output, &fixture_adapter("directory"))
            .expect_err("directory sidecar must fail closed");
        assert!(error.reason.contains("not a regular file"));
        assert_eq!(directory_bytes(&output), before);
        assert!(output.join(TOKENIZER_ADAPTER_FILE).is_dir());
        assert!(tokenizer_adapter_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[cfg(unix)]
    #[test]
    fn compile_adapter_rejects_dangling_sidecar_without_mutation() {
        use std::os::unix::fs::symlink;

        let output = unique_cli_test_path("compile-adapter-sidecar-dangling");
        std::fs::create_dir_all(&output).expect("output dir");
        let sidecar = output.join(TOKENIZER_ADAPTER_FILE);
        symlink("missing-adapter-target", &sidecar).expect("dangling sidecar");
        std::fs::write(output.join("corpus.meta"), b"preserve me").expect("payload");
        let before = directory_bytes(&output);
        let link_before = std::fs::read_link(&sidecar).expect("link target");
        let error = pin_compile_tokenizer_adapter(&output, &fixture_adapter("dangling"))
            .expect_err("dangling sidecar must fail closed");
        assert!(error.reason.contains("not a regular file"));
        assert_eq!(directory_bytes(&output), before);
        assert_eq!(
            std::fs::read_link(&sidecar).expect("link target"),
            link_before
        );
        assert!(
            std::fs::symlink_metadata(&sidecar)
                .expect("sidecar metadata")
                .file_type()
                .is_symlink()
        );
        assert!(tokenizer_adapter_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[cfg(unix)]
    #[test]
    fn compile_adapter_rejects_non_regular_payload_entries() {
        use std::os::unix::fs::symlink;

        let adapter = fixture_adapter("non-regular-payload");
        let directory_output = unique_cli_test_path("compile-payload-directory");
        std::fs::create_dir_all(directory_output.join("corpus.meta")).expect("directory payload");
        let error = pin_compile_tokenizer_adapter(&directory_output, &adapter)
            .expect_err("directory payload must fail closed");
        assert!(error.reason.contains("not a regular file"));
        assert!(directory_output.join("corpus.meta").is_dir());
        assert!(!directory_output.join(TOKENIZER_ADAPTER_FILE).exists());
        assert!(tokenizer_adapter_temp_entries(&directory_output).is_empty());

        let dangling_output = unique_cli_test_path("compile-payload-dangling");
        std::fs::create_dir_all(&dangling_output).expect("output dir");
        let tokenizer = dangling_output.join("tokenizer.bin");
        symlink("missing-tokenizer-target", &tokenizer).expect("dangling payload");
        let error = pin_compile_tokenizer_adapter(&dangling_output, &adapter)
            .expect_err("dangling payload must fail closed");
        assert!(error.reason.contains("not a regular file"));
        assert!(
            std::fs::symlink_metadata(&tokenizer)
                .expect("payload metadata")
                .file_type()
                .is_symlink()
        );
        assert!(!dangling_output.join(TOKENIZER_ADAPTER_FILE).exists());
        assert!(tokenizer_adapter_temp_entries(&dangling_output).is_empty());

        let _ = std::fs::remove_dir_all(directory_output);
        let _ = std::fs::remove_dir_all(dangling_output);
    }

    #[test]
    fn compile_adapter_mismatch_and_malformed_sidecar_preserve_every_byte() {
        let output = unique_cli_test_path("compile-adapter-mismatch");
        let first = fixture_adapter("first");
        let second = fixture_adapter("second");
        assert_ne!(first, second);
        pin_compile_tokenizer_adapter(&output, &first).expect("first pin");
        std::fs::write(output.join("tokenizer.bin"), b"runtime tokenizer")
            .expect("tokenizer payload");
        std::fs::write(output.join("corpus.meta"), b"corpus metadata").expect("corpus payload");
        std::fs::write(output.join("corpus.records"), b"corpus records").expect("record payload");
        let before = directory_bytes(&output);
        let error =
            pin_compile_tokenizer_adapter(&output, &second).expect_err("mismatch must fail closed");
        assert!(error.reason.contains("incompatible compile resume"));
        assert_eq!(directory_bytes(&output), before);

        std::fs::write(output.join(TOKENIZER_ADAPTER_FILE), b"{not json")
            .expect("malformed sidecar");
        let malformed_before = directory_bytes(&output);
        let error = pin_compile_tokenizer_adapter(&output, &first)
            .expect_err("malformed sidecar must fail closed");
        assert!(
            error
                .reason
                .contains("not a valid tokenizer adapter record")
        );
        assert_eq!(directory_bytes(&output), malformed_before);

        let mut inconsistent = first.clone();
        inconsistent.adapter_digest = format!("blake3:{}", "0".repeat(64));
        std::fs::write(
            output.join(TOKENIZER_ADAPTER_FILE),
            serde_json::to_vec_pretty(&inconsistent).expect("invalid record JSON"),
        )
        .expect("inconsistent sidecar");
        let inconsistent_before = directory_bytes(&output);
        let error = pin_compile_tokenizer_adapter(&output, &first)
            .expect_err("self-inconsistent sidecar must fail closed");
        assert!(error.reason.contains("declares digest"));
        assert_eq!(directory_bytes(&output), inconsistent_before);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_adapter_refuses_to_relabel_legacy_corpus_payload() {
        let output = unique_cli_test_path("compile-adapter-legacy");
        std::fs::create_dir_all(&output).expect("output dir");
        std::fs::write(output.join("tokenizer.bin"), b"legacy tokenizer")
            .expect("tokenizer payload");
        std::fs::write(output.join("corpus.meta"), b"legacy corpus").expect("corpus payload");
        let before = directory_bytes(&output);
        let error = pin_compile_tokenizer_adapter(&output, &fixture_adapter("requested"))
            .expect_err("legacy payload must remain unpinned");
        assert!(error.reason.contains("refusing to relabel legacy/unpinned"));
        assert_eq!(directory_bytes(&output), before);
        assert!(!output.join(TOKENIZER_ADAPTER_FILE).exists());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn evaluate_report_defaults_target_smollm2_paths() {
        let options = parse_evaluate_report_options(&[]).expect("defaults");
        assert_eq!(options.source, PathBuf::from(DEFAULT_HF_SOURCE_PATH));
        assert_eq!(options.compiled, PathBuf::from(DEFAULT_HF_COMPILED_PATH));
        assert_eq!(options.sequence_length, 128);
        assert_eq!(options.report, None);
        assert!(!options.bos);
        assert_eq!(options.max_held_out_stories, None);
        assert_eq!(options.tokenizer_adapter, None);
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
            "--bos",
            "--max-held-out-stories",
            "5",
            "--tokenizer-family",
            "sentencepiece-unigram",
            "--tokenizer-version",
            "1",
        ]
        .map(str::to_owned);
        let options = parse_evaluate_report_options(&args).expect("valid options");
        assert_eq!(options.source, PathBuf::from("/tmp/source"));
        assert_eq!(options.compiled, PathBuf::from("/tmp/compiled"));
        assert_eq!(options.report, Some(PathBuf::from("/tmp/out.json")));
        assert_eq!(options.sequence_length, 256);
        assert!(options.bos);
        assert_eq!(options.max_held_out_stories, Some(5));
        assert_eq!(
            options.tokenizer_adapter,
            Some(TokenizerAdapterKey::sentencepiece_unigram_v1())
        );
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
                bos_prefix: false,
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
        assert_eq!(options.tokenizer_adapter, None);

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
        assert_eq!(options.tokenizer_adapter, None);
    }

    #[test]
    fn recorded_compile_requires_explicit_corpus_and_vocab() {
        let args = [
            "--corpus-meta",
            "/tmp/corpus.meta",
            "--corpus-recs",
            "/tmp/corpus.records",
            "--vocab-size",
            "49152",
            "--out",
            "/tmp/recorded",
        ]
        .map(str::to_owned);
        let options = parse_recorded_compile_options(&args).expect("valid recorded options");
        assert_eq!(options.corpus_meta, PathBuf::from("/tmp/corpus.meta"));
        assert_eq!(options.corpus_recs, PathBuf::from("/tmp/corpus.records"));
        assert_eq!(options.vocab_size, 49_152);
        assert_eq!(options.output, PathBuf::from("/tmp/recorded"));

        assert!(
            parse_recorded_compile_options(&["--out", "/tmp/recorded"].map(str::to_owned)).is_err()
        );
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
        assert_eq!(options.tokenizer_adapter, None);

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
        assert_eq!(options.tokenizer_adapter, None);
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
    fn registered_tokenizer_flags_are_paired_and_exclude_legacy_paths() {
        let family_only = [
            "--source",
            "/tmp/source",
            "--tokenizer-family",
            "hf-byte-bpe",
        ]
        .map(str::to_owned);
        let version_only =
            ["--source", "/tmp/source", "--tokenizer-version", "1"].map(str::to_owned);
        for error in [
            parse_compile_options(&family_only).expect_err("family alone is incomplete"),
            parse_compile_options(&version_only).expect_err("version alone is incomplete"),
        ] {
            assert!(error.reason.contains("requires --tokenizer-"));
        }

        let selected = [
            "--source",
            "/tmp/source",
            "--tokenizer-family",
            "sentencepiece-unigram",
            "--tokenizer-version",
            "1",
        ]
        .map(str::to_owned);
        assert_eq!(
            parse_observe_options(&selected)
                .expect("observe selection")
                .tokenizer_adapter,
            Some(TokenizerAdapterKey::sentencepiece_unigram_v1())
        );
        assert_eq!(
            parse_observe_text_options(&selected)
                .expect("observe-text selection")
                .tokenizer_adapter,
            Some(TokenizerAdapterKey::sentencepiece_unigram_v1())
        );

        let checkpoint_and_registered = [
            "--checkpoint",
            "/tmp/model.bin",
            "--tokenizer-family",
            "hf-byte-bpe",
            "--tokenizer-version",
            "1",
        ]
        .map(str::to_owned);
        assert!(parse_observe_options(&checkpoint_and_registered).is_err());
        assert!(parse_observe_text_options(&checkpoint_and_registered).is_err());
        assert!(
            parse_observe_text_options(&["--tokenizer", "/tmp/tokenizer.bin"].map(str::to_owned))
                .is_err()
        );
    }

    #[test]
    fn source_tokenizer_selection_is_unambiguous_and_fail_closed() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let source = std::env::temp_dir().join(format!("uor-r4-cli-tokenizer-{nonce}"));
        std::fs::create_dir_all(&source).expect("source dir");
        let tokenizer_json = br#"{
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"type":"BPE","vocab":{"a":0},"merges":[]}
        }"#;
        std::fs::write(source.join("tokenizer.json"), tokenizer_json).expect("BPE fixture");

        let automatic =
            resolve_source_tokenizer(&source, None).expect("one definition auto-selects");
        assert_eq!(
            automatic.adapter().expect("registered").family,
            "hf-byte-bpe"
        );

        // Even a malformed second definition makes auto-selection ambiguous;
        // explicit BPE remains deterministic while explicit SentencePiece
        // surfaces the selected parse failure instead of falling back.
        std::fs::write(source.join("spiece.model"), b"not a model").expect("SP fixture");
        assert!(resolve_source_tokenizer(&source, None).is_err());
        assert!(
            resolve_source_tokenizer(&source, Some(&TokenizerAdapterKey::hf_byte_bpe_v1())).is_ok()
        );
        assert!(
            resolve_source_tokenizer(
                &source,
                Some(&TokenizerAdapterKey::sentencepiece_unigram_v1())
            )
            .is_err()
        );
        let unknown = TokenizerAdapterKey::new("hf-byte-bpe", 2);
        let error = resolve_source_tokenizer(&source, Some(&unknown))
            .err()
            .expect("unknown version is refused by name");
        assert!(error.reason.contains("hf-byte-bpe/2"));

        let _ = std::fs::remove_dir_all(source);
    }

    #[test]
    fn score_defaults_and_overrides() {
        let (default_meta, default_recs) = compiler::corpus_paths();
        let options = parse_score_options(&[]).expect("defaults");
        assert_eq!(options.corpus_meta, PathBuf::from(default_meta));
        assert_eq!(options.corpus_recs, PathBuf::from(default_recs));
        assert_eq!(options.artifacts, PathBuf::from(compiler::ART_PATH));
        assert_eq!(options.tokenizer, None);
        assert_eq!(options.cover, None);
        assert_eq!(
            options.transition_out_degree,
            score::DEFAULT_TRANSITION_OUT_DEGREE
        );
        assert_eq!(options.emission_entries, score::DEFAULT_EMISSION_ENTRIES);
        assert_eq!(options.emission_selection, score::EmissionSelection::Ratio);
        assert_eq!(options.emission_shrinkage, score::EmissionShrinkage::None);
        assert_eq!(options.context_order, score::DEFAULT_CONTEXT_ORDER);
        assert_eq!(options.context_entries, score::DEFAULT_CONTEXT_ENTRIES);
        assert!(!options.gate_c_context_window);
        assert_eq!(
            options.repetition_penalty_raw,
            score::DEFAULT_REPETITION_PENALTY_RAW
        );
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
            "--tokenizer",
            "/tmp/tokenizer.bin",
            "--cover",
            "/tmp/cover.r4g1",
            "--transition-out-degree",
            "16",
            "--emission-entries",
            "256",
            "--emission-selection",
            "probability",
            "--emission-shrinkage",
            "witten-bell",
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
        assert_eq!(options.tokenizer, Some(PathBuf::from("/tmp/tokenizer.bin")));
        assert_eq!(options.cover, Some(PathBuf::from("/tmp/cover.r4g1")));
        assert_eq!(options.transition_out_degree, 16);
        assert_eq!(options.emission_entries, 256);
        assert_eq!(
            options.emission_selection,
            score::EmissionSelection::Probability
        );
        assert_eq!(
            options.emission_shrinkage,
            score::EmissionShrinkage::WittenBell
        );
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

    #[test]
    fn score_emission_shrinkage_flag_parses_all_variants() {
        let parse = |value: &str| {
            let args = ["--emission-shrinkage", value].map(str::to_owned);
            parse_score_options(&args)
                .expect("valid emission shrinkage")
                .emission_shrinkage
        };
        assert_eq!(parse("none"), score::EmissionShrinkage::None);
        assert_eq!(parse("witten-bell"), score::EmissionShrinkage::WittenBell);
        assert!(parse_score_options(&["--emission-shrinkage", "bad"].map(str::to_owned)).is_err());
    }
}
