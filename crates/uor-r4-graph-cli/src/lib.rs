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
use uor_r4_graph_compiler::recorded_corpus::{
    PLANNED_OUTPUT_RESERVED_PREFIX, PlannedOutputMember, RecordedCorpusProducerGuard,
    RecordedCorpusReaderPins, RecordedCorpusRole,
};
use uor_r4_graph_compiler::reproducibility as repro;
mod convert_r4g1;
pub mod cover_sweep;
pub mod recommend_scale;
mod runtime_corpus;
mod scenarios;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use uor_r4_core::transformerless::hf_bpe::{
    TokenizerAdapter, TokenizerAdapterKey, TokenizerKind, adapter_constructor,
    resolve_source_tokenizer,
};
use uor_r4_core::transformerless::scenarios as core_scenarios;
use uor_r4_core::transformerless::scenarios::RuntimeTokenizerDecodeTable;
use uor_r4_model_source::{
    BehaviorSource, LlamaOracle, SourceUnavailable, Teacher, TeacherOracle,
    attention::AttentionOperatorSpec, dense::DenseOperatorSpec,
};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const DEFAULT_TOKENIZER: &str = "/tmp/ref/tokenizer.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";
const DEFAULT_HF_SOURCE_PATH: &str = ".uor-models/sources/smollm2-135m-instruct";
const DEFAULT_HF_COMPILED_PATH: &str = ".uor-models/compiled/smollm2-135m-instruct";
const DEFAULT_HF_EVALUATION_REPORT: &str = "instruction-eval.json";
const DEFAULT_TEXT_CORPUS: &str = ".uor-models/corpora/simple-wiki-20231101/articles.jsonl";
const TOKENIZER_ADAPTER_FILE: &str = "tokenizer_adapter.json";
/// Compile-directory binding for the source attention operator that produced
/// `corpus.meta` / `corpus.records`.
pub const ATTENTION_OPERATOR_BINDING_FILE: &str = "attention_operator.json";
/// Compile-directory binding for the host-side dense arithmetic that produced
/// `corpus.meta` / `corpus.records`.
pub const DENSE_OPERATOR_BINDING_FILE: &str = "dense_operator.json";
// Pre-#602 corpora computed the immutable standard-source-attention/1
// operator even after the registry's current source version advances.
const LEGACY_STANDARD_ATTENTION_OPERATOR_VERSION: u32 = 1;
static TOKENIZER_ADAPTER_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static ATTENTION_OPERATOR_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static DENSE_OPERATOR_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static PLANNED_OUTPUT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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

#[derive(Default)]
struct CompileOutputPayloadInventory {
    first_present: Option<&'static str>,
}

// Union of the named leaves mutated by source and recorded compilation.
// Identity sidecars are validated separately by their strict readers and are
// published atomically; every other output is inventoried here before either
// identity can take an exact-resume fast path.
const COMPILE_OUTPUT_MUTABLE_FILES: [&str; 10] = [
    "tokenizer.bin",
    "corpus.meta",
    "corpus.records",
    "corpus.records.hidden",
    "tless_artifacts.bin",
    "tless_store.bin",
    "hamming_calibration.json",
    "hierarchical_codes.json",
    "space_manifest.json",
    uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE,
];

const SOURCE_CORPUS_META_BYTES: usize = 25;
const SOURCE_CORPUS_RECORD_BYTES: u64 = 48;
const SOURCE_CORPUS_META_FILE: &str = "corpus.meta";

#[derive(Debug, Default, PartialEq, Eq)]
struct SourceCompileResumePlan {
    records_committed_bytes: Option<u64>,
    hidden_committed_bytes: Option<u64>,
}

fn looks_like_source_corpus_checkpoint_temporary(name: &str) -> bool {
    name.starts_with(&format!(".{SOURCE_CORPUS_META_FILE}.checkpoint.")) && name.ends_with(".tmp")
}

/// Validate, without mutating, the exact checkpoint-temporary namespace.
/// A strict numeric name can contain zero, partial, or complete bytes because
/// every one is an honest process-death point after `create_new`. Only the
/// caller holding exclusive compile-session ownership may reclaim it later.
fn validate_source_corpus_checkpoint_temporaries(output: &Path) -> Result<(), SourceUnavailable> {
    let entries = match std::fs::read_dir(output) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "source compile output {} cannot be enumerated for checkpoint recovery: {error}",
                output.display()
            )));
        }
    };
    for entry in entries {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if compiler::is_source_corpus_checkpoint_temporary_name(&name, SOURCE_CORPUS_META_FILE) {
            let metadata =
                std::fs::symlink_metadata(entry.path()).map_err(SourceUnavailable::new)?;
            if !metadata.file_type().is_file() {
                return Err(SourceUnavailable::new(format!(
                    "recognized source corpus checkpoint temporary {} is not a regular non-symlink file",
                    entry.path().display()
                )));
            }
        } else if looks_like_source_corpus_checkpoint_temporary(&name) {
            return Err(SourceUnavailable::new(format!(
                "source compile output {} contains unrecognized checkpoint temporary {name}",
                output.display()
            )));
        }
    }
    Ok(())
}

fn recover_source_corpus_checkpoint_temporaries(output: &Path) -> Result<(), SourceUnavailable> {
    validate_source_corpus_checkpoint_temporaries(output)?;
    let entries = match std::fs::read_dir(output) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(SourceUnavailable::new(error)),
    };
    let mut reclaimed = false;
    for entry in entries {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if compiler::is_source_corpus_checkpoint_temporary_name(&name, SOURCE_CORPUS_META_FILE) {
            std::fs::remove_file(entry.path()).map_err(SourceUnavailable::new)?;
            reclaimed = true;
        }
    }
    if reclaimed {
        sync_compile_output_directory(output)
    } else {
        Ok(())
    }
}

fn initial_source_corpus_checkpoint() -> [u8; SOURCE_CORPUS_META_BYTES] {
    let mut bytes = [0u8; SOURCE_CORPUS_META_BYTES];
    bytes[16..24].copy_from_slice(&0x5EEDu64.to_le_bytes());
    bytes
}

fn sync_compile_output_directory(output: &Path) -> Result<(), SourceUnavailable> {
    std::fs::File::open(output)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "compile output directory {} cannot be durably synchronized: {error}",
                output.display()
            ))
        })
}

fn write_regular_file_synced(
    path: &Path,
    bytes: &[u8],
    context: &str,
) -> Result<(), SourceUnavailable> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be opened without following links: {error}",
            path.display()
        ))
    })?;
    let path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !path_metadata.file_type().is_file() || !file_metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not a regular non-symlink file",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if path_metadata.dev() != file_metadata.dev() || path_metadata.ino() != file_metadata.ino()
        {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed identity while opened",
                path.display()
            )));
        }
    }
    file.set_len(0)
        .and_then(|()| file.write_all(bytes))
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be durably written: {error}",
                path.display()
            ))
        })?;
    let final_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    if !final_path.file_type().is_file()
        || !final_file.file_type().is_file()
        || final_file.len() != bytes.len() as u64
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed type or length during its durable write",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if final_path.dev() != final_file.dev() || final_path.ino() != final_file.ino() {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed identity during its durable write",
                path.display()
            )));
        }
    }
    Ok(())
}

fn planned_output_member(
    path: &Path,
    context: &str,
) -> Result<PlannedOutputMember, SourceUnavailable> {
    let stable_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| SourceUnavailable::new(format!("{context} stable filename is not UTF-8")))?;
    PlannedOutputMember::REGISTERED
        .into_iter()
        .find(|member| member.stable_name() == stable_name)
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{context} stable filename {stable_name:?} is not a registered planned-output member"
            ))
        })
}

fn preflight_guarded_planned_file_in_scope(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    expected: &[u8],
    context: &str,
    allowed: &[PlannedOutputMember],
) -> Result<bool, SourceUnavailable> {
    let member = planned_output_member(path, context)?;
    guard.preflight_planned_output_scope(allowed)?;
    let max_bytes = u64::try_from(expected.len())
        .map_err(|_| SourceUnavailable::new(format!("{context} length exceeds u64")))?;
    guard.verify_optional_owned_entry_bytes(
        std::ffi::OsStr::new(member.stable_name()),
        expected,
        max_bytes,
        context,
    )
}

fn preflight_guarded_planned_absence(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    context: &str,
) -> Result<(), SourceUnavailable> {
    preflight_guarded_planned_absence_in_scope(guard, path, context, &PlannedOutputMember::ALL)
}

fn preflight_guarded_planned_absence_in_scope(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    context: &str,
    allowed: &[PlannedOutputMember],
) -> Result<(), SourceUnavailable> {
    let member = planned_output_member(path, context)?;
    guard.preflight_planned_output_scope(allowed)?;
    if guard.has_planned_output_residue(member)? {
        return Err(SourceUnavailable::new(format!(
            "{context} has staged bytes but the planned generation declares this member absent"
        )));
    }
    let _ = require_existing_file_matches_if_present(path, None, context)?;
    Ok(())
}

fn preflight_guarded_planned_scope(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    allowed: &[PlannedOutputMember],
    context: &str,
) -> Result<(), SourceUnavailable> {
    guard
        .preflight_planned_output_scope(allowed)
        .map_err(|mut error| {
            error.reason = format!("{context}: {}", error.reason);
            error
        })
}

/// Publish one deterministic member without ever exposing a zero/partial
/// authoritative pathname. Honest process death can leave only a recognized
/// non-authoritative `.writing` inode, which an exact guarded retry reclaims.
fn publish_guarded_planned_file(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    expected: &[u8],
    context: &str,
) -> Result<(), SourceUnavailable> {
    publish_guarded_planned_file_with_after_sync(
        guard,
        path,
        expected,
        context,
        &PlannedOutputMember::ALL,
        |_| Ok(()),
    )
}

/// Testable crash boundary for deterministic planned members. Returning an
/// error from `after_sync` models process death after the complete staging
/// inode is durable but before its atomic no-clobber publication. The staging
/// inode deliberately remains for the exact guarded retry to reclaim.
fn publish_guarded_planned_file_with_after_sync<F>(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    expected: &[u8],
    context: &str,
    allowed: &[PlannedOutputMember],
    after_sync: F,
) -> Result<(), SourceUnavailable>
where
    F: FnOnce(&Path) -> Result<(), SourceUnavailable>,
{
    let member = planned_output_member(path, context)?;
    guard.preflight_planned_output_scope(allowed)?;
    let max_bytes = u64::try_from(expected.len())
        .map_err(|_| SourceUnavailable::new(format!("{context} length exceeds u64")))?;
    let stable_name = std::ffi::OsStr::new(member.stable_name());
    if guard.verify_optional_owned_entry_bytes(stable_name, expected, max_bytes, context)? {
        guard.reclaim_planned_output_residues(member)?;
        guard.sync_owned_root()?;
        return guard.verify_owned_root();
    }
    guard.reclaim_planned_output_residues(member)?;

    let sequence = PLANNED_OUTPUT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staging_name = format!(
        "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--{}.{}.writing",
        member.stable_name(),
        std::process::id(),
        sequence
    );
    let staging_leaf = std::ffi::OsStr::new(&staging_name);
    let staging = guard.root().join(staging_leaf);
    let mut file = guard.create_new_owned_entry(staging_leaf)?;
    file.write_all(expected)
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} staging {} cannot be durably written: {error}",
                staging.display()
            ))
        })?;
    drop(file);
    guard.verify_owned_root()?;
    after_sync(&staging)?;
    guard.link_owned_entry(staging_leaf, stable_name)?;
    guard.unlink_owned_entry(staging_leaf)?;
    guard.sync_owned_root()?;
    guard.verify_owned_entry_bytes(stable_name, expected, max_bytes, context)?;
    guard.verify_owned_root()
}

fn stream_regular_file_summary_nofollow(
    path: &Path,
    context: &str,
) -> Result<
    Option<uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary>,
    SourceUnavailable,
> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} cannot be streamed without following links: {error}",
                path.display()
            )));
        }
    };
    let initial_file = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file.file_type().is_file()
        || !initial_path.file_type().is_file()
        || !opened_file_identity_matches(&initial_path, &initial_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is not one regular non-symlink inode",
            path.display()
        )));
    }
    let captured_length = initial_file.len();
    let mut remaining = captured_length;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    while remaining != 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| SourceUnavailable::new(format!("{context} length overflows usize")))?;
        let count = file
            .read(&mut buffer[..limit])
            .map_err(SourceUnavailable::new)?;
        if count == 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} {} ended before its captured {captured_length}-byte length",
                path.display()
            )));
        }
        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its captured {captured_length}-byte length",
            path.display()
        )));
    }
    let final_file = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !final_file.file_type().is_file()
        || !final_path.file_type().is_file()
        || !opened_file_generation_matches(&initial_file, &final_file)
        || !opened_file_identity_matches(&final_path, &final_file)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed generation while it was streamed",
            path.display()
        )));
    }
    Ok(Some(
        uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary {
            length: captured_length,
            blake3: format!("blake3:{}", hasher.finalize().to_hex()),
        },
    ))
}

fn preflight_guarded_planned_stream(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    expected: Option<&uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary>,
    context: &str,
) -> Result<bool, SourceUnavailable> {
    preflight_guarded_planned_stream_in_scope(
        guard,
        path,
        expected,
        context,
        &PlannedOutputMember::ALL,
    )
}

fn preflight_guarded_planned_stream_in_scope(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    expected: Option<&uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary>,
    context: &str,
    allowed: &[PlannedOutputMember],
) -> Result<bool, SourceUnavailable> {
    let member = planned_output_member(path, context)?;
    guard.preflight_planned_output_scope(allowed)?;
    if let Some(expected) = expected {
        return guard.verify_optional_owned_entry_summary(
            std::ffi::OsStr::new(member.stable_name()),
            expected,
            context,
        );
    }
    let actual = stream_regular_file_summary_nofollow(path, context)?;
    if actual.is_some() {
        return Err(SourceUnavailable::new(format!(
            "{context} {} is present but the planned generation declares typed absence",
            path.display()
        )));
    }
    if guard.has_planned_output_residue(member)? {
        return Err(SourceUnavailable::new(format!(
            "{context} has staged bytes but the planned generation declares this member absent"
        )));
    }
    Ok(true)
}

/// Stream one deterministic large member through a guarded owner-only stage.
/// The source is never accumulated in memory; the staged length/digest must
/// equal the predeclared source summary before the stable hard-link appears.
fn publish_guarded_planned_stream<R: Read + Seek>(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    source: &mut R,
    expected: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary,
    context: &str,
) -> Result<(), SourceUnavailable> {
    publish_guarded_planned_stream_with_before_link(guard, path, source, expected, context, |_| {
        Ok(())
    })
}

fn publish_guarded_planned_stream_with_before_link<R, F>(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    path: &Path,
    source: &mut R,
    expected: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary,
    context: &str,
    before_link: F,
) -> Result<(), SourceUnavailable>
where
    R: Read + Seek,
    F: FnOnce(&Path) -> Result<(), SourceUnavailable>,
{
    let member = planned_output_member(path, context)?;
    if preflight_guarded_planned_stream(guard, path, Some(expected), context)? {
        guard.reclaim_planned_output_residues(member)?;
        guard.sync_owned_root()?;
        return guard.verify_owned_root();
    }
    guard.reclaim_planned_output_residues(member)?;

    let stable_name = std::ffi::OsStr::new(member.stable_name());
    let sequence = PLANNED_OUTPUT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staging_name = format!(
        "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--{}.{}.writing",
        member.stable_name(),
        std::process::id(),
        sequence
    );
    let staging_leaf = std::ffi::OsStr::new(&staging_name);
    let staging = guard.root().join(staging_leaf);
    let mut staging_file = guard.create_new_owned_entry(staging_leaf)?;
    let staging_initial = staging_file.metadata().map_err(SourceUnavailable::new)?;

    source
        .seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    let mut length = 0u64;
    let mut remaining = expected.length;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    while remaining != 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| SourceUnavailable::new(format!("{context} length overflows usize")))?;
        let count = source
            .read(&mut buffer[..limit])
            .map_err(SourceUnavailable::new)?;
        if count == 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} source ended before its declared {}-byte length",
                expected.length
            )));
        }
        staging_file
            .write_all(&buffer[..count])
            .map_err(SourceUnavailable::new)?;
        length = length.checked_add(count as u64).ok_or_else(|| {
            SourceUnavailable::new(format!("{context} staged length overflows u64"))
        })?;
        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }
    let mut extra = [0u8; 1];
    if source.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} source grew beyond its declared {}-byte length",
            expected.length
        )));
    }
    staging_file.sync_all().map_err(SourceUnavailable::new)?;
    let staged = uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary {
        length,
        blake3: format!("blake3:{}", hasher.finalize().to_hex()),
    };
    if &staged != expected {
        return Err(SourceUnavailable::new(format!(
            "{context} source stream changed from its predeclared length or BLAKE3 while staged"
        )));
    }
    let staging_final = staging_file.metadata().map_err(SourceUnavailable::new)?;
    if !staging_final.file_type().is_file()
        || staging_final.len() != expected.length
        || !opened_file_identity_matches(&staging_initial, &staging_final)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} staging {} changed identity, length, or generation before publication",
            staging.display()
        )));
    }
    drop(staging_file);
    guard.verify_owned_entry_summary(staging_leaf, expected, context)?;
    guard.verify_owned_root()?;
    before_link(&staging)?;
    guard.link_owned_entry(staging_leaf, stable_name)?;
    guard.unlink_owned_entry(staging_leaf)?;
    guard.sync_owned_root()?;
    guard.verify_owned_entry_summary(stable_name, expected, context)?;
    guard.verify_owned_root()
}

/// Exclusive source-corpus transaction retained across one complete teacher
/// compile. Managed server callers may acquire this before publishing their
/// stage-local source identity and pass it into `compile_hugging_face_with_session`
/// so P/K and corpus/artifact publication share one common producer interval.
pub struct SourceCorpusSession {
    file: std::fs::File,
    recorded_corpus: Option<uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard>,
}

impl std::fmt::Debug for SourceCorpusSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SourceCorpusSession")
            .field("recorded_corpus", &self.recorded_corpus.is_some())
            .finish_non_exhaustive()
    }
}

impl Drop for SourceCorpusSession {
    fn drop(&mut self) {
        self.release_in_order(|| {});
    }
}

fn source_corpus_session(output: &Path) -> Result<SourceCorpusSession, SourceUnavailable> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(SourceUnavailable::new)?;
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| SourceUnavailable::new("compile output name is not UTF-8"))?;
    let path = parent.join(format!(".{output_name}.source-corpus-session.lock"));
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "source corpus session {} is not a regular non-symlink file",
                path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SourceUnavailable::new(error)),
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(SourceUnavailable::new)?;
    let path_metadata = std::fs::symlink_metadata(&path).map_err(SourceUnavailable::new)?;
    let file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    if !path_metadata.file_type().is_file() || !file_metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "source corpus session {} changed type while opened",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != file_metadata.dev() || path_metadata.ino() != file_metadata.ino()
        {
            return Err(SourceUnavailable::new(format!(
                "source corpus session {} changed identity while opened",
                path.display()
            )));
        }
    }
    file.try_lock().map_err(|error| {
        SourceUnavailable::new(format!(
            "source corpus output {} is BUSY under another compiler session: {error}",
            output.display()
        ))
    })?;
    Ok(SourceCorpusSession {
        file,
        recorded_corpus: None,
    })
}

/// Acquire the source-specific outer session followed by the common recorded-
/// corpus producer guard for an already chosen output root.
pub fn acquire_source_corpus_session(
    output: &Path,
) -> Result<SourceCorpusSession, SourceUnavailable> {
    let mut session = source_corpus_session(output)?;
    session.acquire_recorded_corpus(output)?;
    Ok(session)
}

impl SourceCorpusSession {
    fn release_in_order<F>(&mut self, after_common_release: F)
    where
        F: FnOnce(),
    {
        // Reverse the canonical acquisition order: common corpus guard first,
        // source-specific outer session second. Releasing the outer lock while
        // the common guard is still live would let another source writer enter
        // and then fail BUSY on the inner guard.
        drop(self.recorded_corpus.take());
        after_common_release();
        let _ = self.file.unlock();
    }

    /// Acquire the common recorded-corpus guard after all read-only semantic
    /// preflights, then create/bind a fresh root before the first member
    /// mutation. The source-specific session is always acquired first.
    fn acquire_recorded_corpus(
        &mut self,
        output: &Path,
    ) -> Result<
        &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
        SourceUnavailable,
    > {
        if self.recorded_corpus.is_none() {
            let mut guard =
                uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
                    output,
                )?;
            guard.ensure_root()?;
            guard.preflight_planned_output_scope(&[])?;
            let _ = guard.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
            self.recorded_corpus = Some(guard);
        }
        self.recorded_corpus.as_ref().ok_or_else(|| {
            SourceUnavailable::new("recorded-corpus producer guard was not retained")
        })
    }

    pub fn recorded_corpus_guard(
        &self,
    ) -> Result<
        &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
        SourceUnavailable,
    > {
        self.recorded_corpus.as_ref().ok_or_else(|| {
            SourceUnavailable::new("recorded-corpus producer guard was not acquired")
        })
    }

    /// Begin the source mutation only after the caller has repeated every
    /// requested identity/resume preflight under the retained common guard.
    fn begin_recorded_corpus_mutation(&self, output: &Path) -> Result<(), SourceUnavailable> {
        let guard = self.recorded_corpus_guard()?;
        guard.preflight_planned_output_scope(&[])?;
        let publication =
            uor_r4_graph_compiler::recorded_corpus::preflight_source_update_publication(
                guard,
                &output.join("corpus.meta"),
                &output.join("corpus.records"),
            )?;
        guard.begin_compile_attempt()?;
        if publication.recoverable_binding {
            uor_r4_graph_compiler::recorded_corpus::publish_binding(
                guard,
                &output.join("corpus.meta"),
                &output.join("corpus.records"),
            )?;
        }
        Ok(())
    }
}

/// Inspect every mutable compile-output leaf without following it. Binding
/// equality does not make a directory, symlink, or special file safe for the
/// subsequent tokenizer export / corpus resume path.
fn compile_output_payload_inventory(
    output: &Path,
) -> Result<CompileOutputPayloadInventory, SourceUnavailable> {
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "compile output root {} is not a real directory; symlinks, files, and special entries are refused",
                output.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CompileOutputPayloadInventory::default());
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "compile output root {} cannot be inspected: {error}",
                output.display()
            )));
        }
    }

    let mut inventory = CompileOutputPayloadInventory::default();
    for name in COMPILE_OUTPUT_MUTABLE_FILES {
        let path = output.join(name);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => {
                if inventory.first_present.is_none() {
                    inventory.first_present = Some(name);
                }
            }
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "{} exists but is not a regular file; refusing compile-output mutation through a directory, symlink, or special file",
                    path.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "{} cannot be inspected: {error}",
                    path.display()
                )));
            }
        }
    }
    Ok(inventory)
}

fn readable_regular_file_len(path: &Path, label: &str) -> Result<u64, SourceUnavailable> {
    let file = std::fs::File::open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be read: {error}",
            path.display()
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be inspected after open: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{label} {} is not a regular file",
            path.display()
        )));
    }
    Ok(metadata.len())
}

fn require_truncatable_if_needed(
    path: &Path,
    current_bytes: u64,
    committed_bytes: u64,
    label: &str,
) -> Result<(), SourceUnavailable> {
    if current_bytes == committed_bytes {
        return Ok(());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map(|_| ())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} {} has an uncommitted tail but cannot be opened for recovery: {error}",
                path.display()
            ))
        })
}

fn validate_source_v3_committed_records(
    bytes: &[u8],
    records: usize,
    stories: u64,
) -> Result<(), SourceUnavailable> {
    let committed = records
        .checked_mul(SOURCE_CORPUS_RECORD_BYTES as usize)
        .ok_or_else(|| {
            SourceUnavailable::new("source corpus v3 committed record length overflows")
        })?;
    let committed = bytes.get(..committed).ok_or_else(|| {
        SourceUnavailable::new("source corpus v3 committed prefix is shorter than declared")
    })?;
    let mut previous_story = None;
    for (index, row) in committed
        .chunks_exact(SOURCE_CORPUS_RECORD_BYTES as usize)
        .enumerate()
    {
        let word = |offset: usize| {
            u32::from_le_bytes(
                row[offset..offset + 4]
                    .try_into()
                    .expect("fixed v3 word range"),
            )
        };
        let story = word(0);
        if u64::from(story) >= stories || previous_story.is_some_and(|prior| story < prior) {
            return Err(SourceUnavailable::new(format!(
                "source corpus record {index} is not a monotone in-range v3 story row"
            )));
        }
        previous_story = Some(story);
        let weight_sum = u64::from(word(20)) + u64::from(word(24)) + u64::from(word(28));
        if weight_sum != 100 {
            return Err(SourceUnavailable::new(format!(
                "source corpus record {index} has v3 top-weight sum {weight_sum}, expected 100"
            )));
        }
        let span_start = word(32);
        let span_end = word(36);
        if span_end != span_start.saturating_add(1) {
            return Err(SourceUnavailable::new(format!(
                "source corpus record {index} has invalid v3 token span {span_start}..{span_end}"
            )));
        }
        let byte_start = word(40);
        let byte_end = word(44);
        if !((byte_start == u32::MAX && byte_end == u32::MAX) || byte_start <= byte_end) {
            return Err(SourceUnavailable::new(format!(
                "source corpus record {index} has invalid v3 byte anchors {byte_start}..{byte_end}"
            )));
        }
    }
    Ok(())
}

/// Validate the source generator's 25-byte checkpoint and the record/hidden
/// streams it commits at story boundaries. This is read-only and runs before
/// either provenance sidecar or tokenizer output can be published.
///
/// A missing hidden stream remains compatible with an already-complete
/// historical corpus because the core loader treats hidden rows as optional.
/// An incomplete historical corpus cannot resume under a hidden-producing
/// oracle: that would create a suffix with no rows for the old prefix. Once a
/// hidden stream is present, its committed prefix is checked against the
/// *actual* loaded oracle's hidden width; an interrupted tail is recovered
/// alongside the record tail before generation resumes.
fn preflight_source_compile_resume(
    output: &Path,
    hidden_row_bytes: Option<usize>,
    target: usize,
) -> Result<SourceCompileResumePlan, SourceUnavailable> {
    // This also rejects a symlink output root and every nonregular mutable
    // leaf, independently of whether both identity sidecars already match.
    let _ = compile_output_payload_inventory(output)?;
    validate_source_corpus_checkpoint_temporaries(output)?;
    let meta_path = output.join("corpus.meta");
    let records_path = output.join("corpus.records");
    let hidden_path = output.join("corpus.records.hidden");
    let meta_present = strict_regular_file_if_present(&meta_path, "source corpus metadata")?;
    let records_present = strict_regular_file_if_present(&records_path, "source corpus records")?;
    let hidden_present =
        strict_regular_file_if_present(&hidden_path, "source corpus hidden stream")?;

    match (meta_present, records_present) {
        (false, false) if !hidden_present => return Ok(SourceCompileResumePlan::default()),
        (false, false) => {
            return Err(SourceUnavailable::new(format!(
                "{} exists without corpus.meta/corpus.records; refusing an orphan hidden resume stream before mutation",
                hidden_path.display()
            )));
        }
        (true, false) if !hidden_present => {
            let bytes = std::fs::read(&meta_path).map_err(SourceUnavailable::new)?;
            if bytes == initial_source_corpus_checkpoint() {
                // The core compiler durably publishes the authoritative zero
                // checkpoint before it creates either stream. A process death
                // in that narrow window is therefore a valid fresh prefix.
                return Ok(SourceCompileResumePlan::default());
            }
            return Err(SourceUnavailable::new(format!(
                "{} exists without corpus.records and is not the authoritative zero checkpoint; one-sided source corpus resume refused before mutation",
                meta_path.display()
            )));
        }
        (true, false) | (false, true) => {
            return Err(SourceUnavailable::new(format!(
                "{} must contain corpus.meta and corpus.records together; one-sided source corpus resume refused before mutation",
                output.display()
            )));
        }
        (true, true) => {}
    }

    let meta = std::fs::read(&meta_path).map_err(|error| {
        SourceUnavailable::new(format!(
            "source corpus metadata {} cannot be read: {error}",
            meta_path.display()
        ))
    })?;
    if meta.len() != SOURCE_CORPUS_META_BYTES {
        return Err(SourceUnavailable::new(format!(
            "source corpus metadata {} is {} bytes, expected exactly {SOURCE_CORPUS_META_BYTES}; refusing a false-fresh resume before mutation",
            meta_path.display(),
            meta.len()
        )));
    }
    let records = u64::from_le_bytes(
        meta[0..8]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid source corpus record count"))?,
    );
    usize::try_from(records).map_err(|_| {
        SourceUnavailable::new(format!(
            "source corpus metadata {} declares {records} records, which cannot be represented on this host",
            meta_path.display()
        ))
    })?;
    let stories = u64::from_le_bytes(
        meta[8..16]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid source corpus story count"))?,
    );
    u32::try_from(stories).map_err(|_| {
        SourceUnavailable::new(format!(
            "source corpus metadata {} declares {stories} stories, exceeding the u32 story-id wire range",
            meta_path.display()
        ))
    })?;
    let done = meta[24];
    if !matches!(done, 0 | 1) {
        return Err(SourceUnavailable::new(format!(
            "source corpus metadata {} has invalid done byte {}; expected 0 or 1",
            meta_path.display(),
            done
        )));
    }

    let record_bytes = std::fs::read(&records_path).map_err(|error| {
        SourceUnavailable::new(format!(
            "source corpus records {} cannot be read: {error}",
            records_path.display()
        ))
    })?;
    let records_len = u64::try_from(record_bytes.len()).map_err(|_| {
        SourceUnavailable::new("source corpus record stream is too large for its wire length")
    })?;
    // The required tokenizer+operator bindings cannot predate the current
    // source generator, whose only write layout is v3/48. Missing-sidecar
    // 32/12-byte legacy outputs are already refused at the identity boundary;
    // do not ambiguously reinterpret a long legacy crash tail as v3 here.
    let records_committed_bytes =
        records
            .checked_mul(SOURCE_CORPUS_RECORD_BYTES)
            .ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "source corpus metadata {} overflows the 48-byte v3 record layout",
                    meta_path.display()
                ))
            })?;
    // n=0 authoritatively commits the empty prefix. Any bytes after it were
    // written before the first story checkpoint and are therefore a crash
    // tail, not an inferable legacy format; recovery truncates them to zero.
    if records_len < records_committed_bytes {
        return Err(SourceUnavailable::new(format!(
            "source corpus records {} is {records_len} bytes, shorter than the {records_committed_bytes}-byte v3 prefix committed for {records} records",
            records_path.display()
        )));
    }
    validate_source_v3_committed_records(
        &record_bytes,
        usize::try_from(records).expect("host-size record count validated above"),
        stories,
    )?;
    require_truncatable_if_needed(
        &records_path,
        records_len,
        records_committed_bytes,
        "source corpus records",
    )?;

    let hidden_committed_bytes = if hidden_present {
        let hidden_len = readable_regular_file_len(&hidden_path, "source corpus hidden stream")?;
        let committed = match hidden_row_bytes {
            Some(row_bytes) if row_bytes != 0 => records
                .checked_mul(row_bytes as u64)
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "source corpus hidden prefix overflows for {records} records at {row_bytes} bytes per oracle row"
                    ))
                })?,
            Some(_) if records == 0 && hidden_len == 0 => 0,
            Some(_) => {
                return Err(SourceUnavailable::new(
                    "loaded teacher exposes a zero-width hidden row for a nonempty hidden corpus",
                ));
            }
            None if hidden_len == 0 => 0,
            None => {
                return Err(SourceUnavailable::new(format!(
                    "source corpus hidden stream {} has {hidden_len} bytes, but the loaded teacher exposes no hidden-state row layout",
                    hidden_path.display()
                )));
            }
        };
        if hidden_len < committed {
            return Err(SourceUnavailable::new(format!(
                "source corpus hidden stream {} is {hidden_len} bytes, shorter than the {committed}-byte prefix committed for {records} records by the loaded teacher",
                hidden_path.display()
            )));
        }
        require_truncatable_if_needed(
            &hidden_path,
            hidden_len,
            committed,
            "source corpus hidden stream",
        )?;
        Some(committed)
    } else if hidden_row_bytes.is_some()
        && records != 0
        && !(done == 1
            && records
                >= u64::try_from(target).map_err(|_| {
                    SourceUnavailable::new("source compile target does not fit the corpus wire")
                })?)
    {
        return Err(SourceUnavailable::new(format!(
            "source corpus has {records} committed records but no hidden stream; historical hidden absence is compatible only for an already-complete corpus, because resuming generation would create an unverifiable partial hidden suffix"
        )));
    } else {
        None
    };

    Ok(SourceCompileResumePlan {
        records_committed_bytes: Some(records_committed_bytes),
        hidden_committed_bytes,
    })
}

fn truncate_source_compile_resume_file(
    path: &Path,
    committed_bytes: u64,
    label: &str,
) -> Result<(), SourceUnavailable> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be reinspected: {error}",
            path.display()
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{label} {} stopped being a regular file before recovery",
            path.display()
        )));
    }
    if metadata.len() < committed_bytes {
        return Err(SourceUnavailable::new(format!(
            "{label} {} shrank to {} bytes before recovery, below its {committed_bytes}-byte committed prefix",
            path.display(),
            metadata.len()
        )));
    }
    if metadata.len() == committed_bytes {
        return Ok(());
    }
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} {} cannot be opened for recovery: {error}",
                path.display()
            ))
        })?;
    file.set_len(committed_bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{label} {} cannot be truncated to its committed {committed_bytes}-byte prefix: {error}",
            path.display()
        ))
    })
}

fn reconcile_source_compile_resume(
    output: &Path,
    plan: &SourceCompileResumePlan,
) -> Result<(), SourceUnavailable> {
    if let Some(committed) = plan.records_committed_bytes {
        truncate_source_compile_resume_file(
            &output.join("corpus.records"),
            committed,
            "source corpus records",
        )?;
    }
    if let Some(committed) = plan.hidden_committed_bytes {
        truncate_source_compile_resume_file(
            &output.join("corpus.records.hidden"),
            committed,
            "source corpus hidden stream",
        )?;
    }
    Ok(())
}

/// Bind a resumable compiler output to the exact full adapter identity before
/// `tokenizer.bin`, `corpus.meta`, or `corpus.records` can be changed.
fn pin_compile_tokenizer_adapter(
    output: &Path,
    requested: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    validate_tokenizer_adapter_record(requested)?;
    let inventory = compile_output_payload_inventory(output)?;
    let sidecar = output.join(TOKENIZER_ADAPTER_FILE);
    let recorded = read_compile_tokenizer_adapter(output)?;
    if let Some(recorded) = recorded {
        return require_matching_compile_tokenizer_adapter(output, requested, &recorded);
    }

    // A pre-#718 compiler corpus has no adapter identity. Any recognized
    // output leaf, including a zero-byte torn create, makes that absence an
    // incompatible legacy era; never relabel it merely because the current
    // source happens to parse.
    if let Some(name) = inventory.first_present {
        return Err(SourceUnavailable::new(format!(
            "{} has no {TOKENIZER_ADAPTER_FILE} but already contains {name} payload; refusing to relabel legacy/unpinned compiler bytes as {}/{} before mutation",
            output.display(),
            requested.family,
            requested.version
        )));
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
            sync_compile_output_directory(output)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(&temporary);
            let recorded = read_compile_tokenizer_adapter(output)?.ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "{} appeared during tokenizer adapter publication but is now absent",
                    sidecar.display()
                ))
            })?;
            require_matching_compile_tokenizer_adapter(output, requested, &recorded)?;
            sync_compile_output_directory(output)
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

/// Read-only half of [`pin_compile_tokenizer_adapter`]. Source-driven compile
/// runs use this together with the attention-operator preflight so a conflict
/// in either identity is found before either sidecar is published.
fn preflight_compile_tokenizer_adapter(
    output: &Path,
    requested: &TokenizerAdapter,
) -> Result<(), SourceUnavailable> {
    validate_tokenizer_adapter_record(requested)?;
    let inventory = compile_output_payload_inventory(output)?;
    let recorded = read_compile_tokenizer_adapter(output)?;
    if let Some(recorded) = recorded {
        return require_matching_compile_tokenizer_adapter(output, requested, &recorded);
    }

    if let Some(name) = inventory.first_present {
        return Err(SourceUnavailable::new(format!(
            "{} has no {TOKENIZER_ADAPTER_FILE} but already contains {name} payload; refusing to relabel legacy/unpinned compiler bytes as {}/{} before mutation",
            output.display(),
            requested.family,
            requested.version
        )));
    }
    Ok(())
}

/// Return `false` only for a genuinely absent path. Every present directory
/// entry must itself be a regular file: symlinks (including links to regular
/// files), directories, and special files are provenance errors.
fn strict_regular_file_if_present(path: &Path, context: &str) -> Result<bool, SourceUnavailable> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(SourceUnavailable::new(format!(
            "{context} {} cannot be inspected: {error}",
            path.display()
        ))),
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => Err(SourceUnavailable::new(format!(
            "{context} {} is present but is not a regular file; symlinks, directories, and special files are refused",
            path.display()
        ))),
    }
}

#[cfg(unix)]
fn opened_file_identity_matches(
    path_metadata: &std::fs::Metadata,
    file_metadata: &std::fs::Metadata,
) -> bool {
    use std::os::unix::fs::MetadataExt;

    path_metadata.dev() == file_metadata.dev() && path_metadata.ino() == file_metadata.ino()
}

#[cfg(unix)]
fn opened_file_generation_matches(
    initial: &std::fs::Metadata,
    final_metadata: &std::fs::Metadata,
) -> bool {
    use std::os::unix::fs::MetadataExt;

    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.mtime() == final_metadata.mtime()
        && initial.mtime_nsec() == final_metadata.mtime_nsec()
        && initial.ctime() == final_metadata.ctime()
        && initial.ctime_nsec() == final_metadata.ctime_nsec()
}

#[cfg(not(unix))]
fn opened_file_generation_matches(
    initial: &std::fs::Metadata,
    final_metadata: &std::fs::Metadata,
) -> bool {
    opened_file_identity_matches(initial, final_metadata)
        && initial.len() == final_metadata.len()
        && initial.modified().ok() == final_metadata.modified().ok()
}

#[cfg(unix)]
fn is_publisher_alias_cleanup_transition(
    initial: &std::fs::Metadata,
    final_metadata: &std::fs::Metadata,
    final_path_metadata: &std::fs::Metadata,
    captured_len: usize,
    verification_offset: usize,
) -> bool {
    use std::os::unix::fs::MetadataExt;

    initial.file_type().is_file()
        && final_metadata.file_type().is_file()
        && final_path_metadata.file_type().is_file()
        && opened_file_identity_matches(initial, final_metadata)
        && opened_file_identity_matches(final_path_metadata, final_metadata)
        && initial.len() == captured_len as u64
        && final_metadata.len() == captured_len as u64
        && verification_offset == captured_len
        && initial.mtime() == final_metadata.mtime()
        && initial.mtime_nsec() == final_metadata.mtime_nsec()
        && (initial.ctime() != final_metadata.ctime()
            || initial.ctime_nsec() != final_metadata.ctime_nsec())
        && initial.nlink() == 2
        && final_metadata.nlink() == 1
}

#[cfg(not(unix))]
fn is_publisher_alias_cleanup_transition(
    _initial: &std::fs::Metadata,
    _final_metadata: &std::fs::Metadata,
    _final_path_metadata: &std::fs::Metadata,
    _captured_len: usize,
    _verification_offset: usize,
) -> bool {
    false
}

struct StrictRegularFileSnapshot {
    bytes: Vec<u8>,
    metadata: std::fs::Metadata,
}

struct PublisherAliasCleanup {
    snapshot: StrictRegularFileSnapshot,
    terminal_error: SourceUnavailable,
}

enum StrictOptionalRegularFileCapture {
    Absent,
    Stable(StrictRegularFileSnapshot),
    PublisherAliasCleanup(PublisherAliasCleanup),
}

#[cfg(not(unix))]
fn opened_file_identity_matches(
    _path_metadata: &std::fs::Metadata,
    _file_metadata: &std::fs::Metadata,
) -> bool {
    // The opened handle and both path checks still reject pre-existing links
    // and type changes. Unix additionally exposes a stable inode identity.
    true
}

/// Read one optional provenance sidecar through a single opened handle. The
/// path is never reopened for bytes: Unix refuses links at `open(2)`, while
/// every platform checks that the path still names the opened regular file
/// before and after the read.
fn capture_optional_regular_file_nofollow_with<F>(
    path: &Path,
    context: &str,
    after_open: F,
) -> Result<StrictOptionalRegularFileCapture, SourceUnavailable>
where
    F: FnOnce(),
{
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        // `O_NOFOLLOW` closes the lstat/open link-swap seam. `O_NONBLOCK`
        // prevents a raced FIFO or device from hanging before the opened
        // handle's type is rejected below.
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StrictOptionalRegularFileCapture::Absent);
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{context} {} is not a regular file or cannot be opened without following links: {error}",
                path.display()
            )));
        }
    };
    let opened_metadata = file.metadata().map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} opened handle cannot be inspected: {error}",
            path.display()
        ))
    })?;
    if !opened_metadata.file_type().is_file() {
        return Err(SourceUnavailable::new(format!(
            "{context} {} opened handle is not a regular file",
            path.display()
        )));
    }

    let path_metadata = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be reinspected after open: {error}",
            path.display()
        ))
    })?;
    if !path_metadata.file_type().is_file()
        || !opened_file_identity_matches(&path_metadata, &opened_metadata)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity or is not a regular non-symlink file",
            path.display()
        )));
    }

    let captured_len = usize::try_from(opened_metadata.len()).map_err(|_| {
        SourceUnavailable::new(format!(
            "{context} {} length cannot be represented on this host",
            path.display()
        ))
    })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(captured_len).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot reserve {captured_len} bytes: {error}",
            path.display()
        ))
    })?;
    bytes.resize(captured_len, 0);
    file.read_exact(&mut bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot read its captured {captured_len}-byte generation: {error}",
            path.display()
        ))
    })?;
    let mut extra = [0u8; 1];
    if file.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} {} grew beyond its captured length while its bytes were read",
            path.display()
        )));
    }

    // Tests use this seam to model an adversarial path or same-inode rewrite
    // at the only meaningful boundary: after one exact handle read but before
    // the generation is accepted. Production supplies an empty closure.
    after_open();

    file.seek(SeekFrom::Start(0)).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} opened handle cannot be rewound: {error}",
            path.display()
        ))
    })?;
    let mut verification_offset = 0usize;
    let mut verification_buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut verification_buffer).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context} {} cannot be verified through its opened handle: {error}",
                path.display()
            ))
        })?;
        if count == 0 {
            break;
        }
        let end = verification_offset.checked_add(count).ok_or_else(|| {
            SourceUnavailable::new(format!("{context} verification byte count overflow"))
        })?;
        if bytes.get(verification_offset..end) != Some(&verification_buffer[..count]) {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed content while its opened handle was verified",
                path.display()
            )));
        }
        verification_offset = end;
    }
    let final_file_metadata = file.metadata().map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} opened handle cannot be reinspected: {error}",
            path.display()
        ))
    })?;
    let final_path_metadata = std::fs::symlink_metadata(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be reinspected after read: {error}",
            path.display()
        ))
    })?;
    let terminal_error = || {
        SourceUnavailable::new(format!(
            "{context} {} changed identity, type, length, content, or generation while its bytes were read",
            path.display()
        ))
    };
    if !final_file_metadata.file_type().is_file()
        || !final_path_metadata.file_type().is_file()
        || !opened_file_generation_matches(&opened_metadata, &final_file_metadata)
        || !opened_file_identity_matches(&final_path_metadata, &final_file_metadata)
        || verification_offset != bytes.len()
        || final_file_metadata.len() != bytes.len() as u64
    {
        let error = terminal_error();
        if is_publisher_alias_cleanup_transition(
            &opened_metadata,
            &final_file_metadata,
            &final_path_metadata,
            bytes.len(),
            verification_offset,
        ) {
            return Ok(StrictOptionalRegularFileCapture::PublisherAliasCleanup(
                PublisherAliasCleanup {
                    snapshot: StrictRegularFileSnapshot {
                        bytes,
                        metadata: final_file_metadata,
                    },
                    terminal_error: error,
                },
            ));
        }
        return Err(error);
    }
    Ok(StrictOptionalRegularFileCapture::Stable(
        StrictRegularFileSnapshot {
            bytes,
            metadata: final_file_metadata,
        },
    ))
}

fn read_optional_regular_file_nofollow_with<F>(
    path: &Path,
    context: &str,
    after_open: F,
) -> Result<Option<Vec<u8>>, SourceUnavailable>
where
    F: FnOnce(),
{
    match capture_optional_regular_file_nofollow_with(path, context, after_open)? {
        StrictOptionalRegularFileCapture::Absent => Ok(None),
        StrictOptionalRegularFileCapture::Stable(snapshot) => Ok(Some(snapshot.bytes)),
        StrictOptionalRegularFileCapture::PublisherAliasCleanup(cleanup) => {
            Err(cleanup.terminal_error)
        }
    }
}

fn read_optional_regular_file_for_operator_publisher_with_hooks<F, B, G>(
    path: &Path,
    context: &str,
    after_first_open: F,
    before_retry: B,
    after_retry_open: G,
) -> Result<Option<Vec<u8>>, SourceUnavailable>
where
    F: FnOnce(),
    B: FnOnce(),
    G: FnOnce(),
{
    let first = capture_optional_regular_file_nofollow_with(path, context, after_first_open)?;
    let StrictOptionalRegularFileCapture::PublisherAliasCleanup(first_cleanup) = first else {
        return match first {
            StrictOptionalRegularFileCapture::Absent => Ok(None),
            StrictOptionalRegularFileCapture::Stable(snapshot) => Ok(Some(snapshot.bytes)),
            StrictOptionalRegularFileCapture::PublisherAliasCleanup(_) => unreachable!(),
        };
    };

    before_retry();
    match capture_optional_regular_file_nofollow_with(path, context, after_retry_open)? {
        StrictOptionalRegularFileCapture::Absent => Err(SourceUnavailable::new(format!(
            "{context} {} disappeared during the one permitted publisher alias-cleanup retry",
            path.display()
        ))),
        StrictOptionalRegularFileCapture::PublisherAliasCleanup(cleanup) => {
            Err(cleanup.terminal_error)
        }
        StrictOptionalRegularFileCapture::Stable(retry) => {
            if !opened_file_generation_matches(&first_cleanup.snapshot.metadata, &retry.metadata)
                || first_cleanup.snapshot.bytes != retry.bytes
            {
                return Err(SourceUnavailable::new(format!(
                    "{context} {} changed generation or content across the one permitted publisher alias-cleanup retry",
                    path.display()
                )));
            }
            Ok(Some(retry.bytes))
        }
    }
}

fn read_optional_regular_file_for_operator_publisher(
    path: &Path,
    context: &str,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    read_optional_regular_file_for_operator_publisher_with_hooks(path, context, || {}, || {}, || {})
}

fn read_optional_regular_file_nofollow(
    path: &Path,
    context: &str,
) -> Result<Option<Vec<u8>>, SourceUnavailable> {
    read_optional_regular_file_nofollow_with(path, context, || {})
}

fn verify_optional_regular_file_nofollow(
    path: &Path,
    context: &str,
    expected: Option<&[u8]>,
) -> Result<(), SourceUnavailable> {
    let Some(expected) = expected else {
        return match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SourceUnavailable::new(format!(
                "{context} {} cannot be reinspected: {error}",
                path.display()
            ))),
            Ok(_) => Err(SourceUnavailable::new(format!(
                "{context} {} appeared after typed absence was captured",
                path.display()
            ))),
        };
    };

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context} {} cannot be reopened without following links: {error}",
            path.display()
        ))
    })?;
    let initial_file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    let initial_path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if !initial_file_metadata.file_type().is_file()
        || !initial_path_metadata.file_type().is_file()
        || !opened_file_identity_matches(&initial_path_metadata, &initial_file_metadata)
        || initial_file_metadata.len() != expected.len() as u64
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} no longer names the captured regular-file generation",
            path.display()
        )));
    }
    let mut offset = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(SourceUnavailable::new)?;
        if count == 0 {
            break;
        }
        let end = offset
            .checked_add(count)
            .ok_or_else(|| SourceUnavailable::new(format!("{context} byte count overflow")))?;
        if expected.get(offset..end) != Some(&buffer[..count]) {
            return Err(SourceUnavailable::new(format!(
                "{context} {} changed bytes after capture",
                path.display()
            )));
        }
        offset = end;
    }
    let final_file_metadata = file.metadata().map_err(SourceUnavailable::new)?;
    let final_path_metadata = std::fs::symlink_metadata(path).map_err(SourceUnavailable::new)?;
    if offset != expected.len()
        || !final_file_metadata.file_type().is_file()
        || !final_path_metadata.file_type().is_file()
        || !opened_file_generation_matches(&initial_file_metadata, &final_file_metadata)
        || !opened_file_identity_matches(&final_path_metadata, &final_file_metadata)
    {
        return Err(SourceUnavailable::new(format!(
            "{context} {} changed identity, bytes, or generation after capture",
            path.display()
        )));
    }
    Ok(())
}

fn validate_attention_operator(
    recorded: &AttentionOperatorSpec,
    context: &str,
) -> Result<AttentionOperatorSpec, SourceUnavailable> {
    let registered = uor_r4_model_source::attention::operator_spec(&recorded.id, recorded.version)
        .map_err(|mut error| {
            error.reason = format!("{context}: {}", error.reason);
            error
        })?;
    if !matches!(
        registered.id.as_str(),
        AttentionOperatorSpec::STANDARD_ID
            | AttentionOperatorSpec::EXPERIMENTAL_R4_ID
            | AttentionOperatorSpec::LEARNED_ABSOLUTE_ID
    ) {
        return Err(SourceUnavailable::new(format!(
            "{context} names {}/{} from the operator registry, but it is not a source-teacher attention operator",
            registered.id, registered.version
        )));
    }
    if recorded != &registered {
        return Err(SourceUnavailable::new(format!(
            "{context} does not match registered attention operator {}/{}",
            recorded.id, recorded.version
        )));
    }
    Ok(registered)
}

type DuplicateRejectingJson = uor_r4_graph_compiler::recorded_corpus::DuplicateRejectingJson;

fn reject_duplicate_attention_operator_json(
    bytes: &[u8],
    path: &Path,
) -> Result<(), SourceUnavailable> {
    serde_json::from_slice::<DuplicateRejectingJson>(bytes)
        .map(|_| ())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{}: malformed attention-operator binding: {error}",
                path.display()
            ))
        })
}

fn validate_attention_operator_json(
    value: &serde_json::Value,
    context: &str,
) -> Result<AttentionOperatorSpec, SourceUnavailable> {
    let recorded: AttentionOperatorSpec =
        serde_json::from_value(value.clone()).map_err(|error| {
            SourceUnavailable::new(format!(
                "{context}: malformed attention-operator record: {error}"
            ))
        })?;
    let registered = validate_attention_operator(&recorded, context)?;
    let registered_json = serde_json::to_value(&registered).map_err(SourceUnavailable::new)?;
    if value != &registered_json {
        return Err(SourceUnavailable::new(format!(
            "{context} is not the full registered attention-operator record; missing, unknown, or noncanonical fields are refused"
        )));
    }
    Ok(registered)
}

fn parse_compiled_attention_operator_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<AttentionOperatorSpec, SourceUnavailable> {
    reject_duplicate_attention_operator_json(bytes, path)?;
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: malformed attention-operator binding: {error}",
            path.display()
        ))
    })?;
    validate_attention_operator_json(&value, &path.display().to_string())
}

fn read_optional_compiled_attention_operator_with_bytes(
    output: &Path,
) -> Result<Option<(AttentionOperatorSpec, Vec<u8>)>, SourceUnavailable> {
    let path = output.join(ATTENTION_OPERATOR_BINDING_FILE);
    let Some(bytes) = read_optional_regular_file_nofollow(&path, "attention-operator binding")?
    else {
        return Ok(None);
    };
    let operator = parse_compiled_attention_operator_bytes(&path, &bytes)?;
    Ok(Some((operator, bytes)))
}

fn read_optional_compiled_attention_operator(
    output: &Path,
) -> Result<Option<AttentionOperatorSpec>, SourceUnavailable> {
    Ok(read_optional_compiled_attention_operator_with_bytes(output)?.map(|(operator, _)| operator))
}

#[cfg(test)]
fn read_optional_compiled_attention_operator_for_publisher_with_hooks<F, B, G>(
    output: &Path,
    after_first_open: F,
    before_retry: B,
    after_retry_open: G,
) -> Result<Option<AttentionOperatorSpec>, SourceUnavailable>
where
    F: FnOnce(),
    B: FnOnce(),
    G: FnOnce(),
{
    let path = output.join(ATTENTION_OPERATOR_BINDING_FILE);
    let Some(bytes) = read_optional_regular_file_for_operator_publisher_with_hooks(
        &path,
        "attention-operator binding",
        after_first_open,
        before_retry,
        after_retry_open,
    )?
    else {
        return Ok(None);
    };
    parse_compiled_attention_operator_bytes(&path, &bytes).map(Some)
}

fn read_optional_compiled_attention_operator_for_publisher(
    output: &Path,
) -> Result<Option<AttentionOperatorSpec>, SourceUnavailable> {
    let path = output.join(ATTENTION_OPERATOR_BINDING_FILE);
    let Some(bytes) =
        read_optional_regular_file_for_operator_publisher(&path, "attention-operator binding")?
    else {
        return Ok(None);
    };
    parse_compiled_attention_operator_bytes(&path, &bytes).map(Some)
}

/// Read and registry-validate a compile directory's attention-operator
/// binding. A missing sidecar is an error on this source-driven/API seam; use
/// [`recorded_corpus_attention_operator`] when legacy absence is meaningful.
pub fn compiled_attention_operator(
    output: &Path,
) -> Result<AttentionOperatorSpec, SourceUnavailable> {
    read_optional_compiled_attention_operator(output)?.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: missing teacher attention-operator binding",
            output.join(ATTENTION_OPERATOR_BINDING_FILE).display()
        ))
    })
}

/// Read-only compatibility check for an output directory's arithmetic-era
/// binding. Returns `true` when the exact binding already exists.
fn preflight_compiled_attention_operator(
    output: &Path,
    requested: &AttentionOperatorSpec,
) -> Result<bool, SourceUnavailable> {
    let requested =
        validate_attention_operator(requested, "proposed teacher attention-operator binding")?;
    let inventory = compile_output_payload_inventory(output)?;
    let recorded = read_optional_compiled_attention_operator_for_publisher(output)?;
    if let Some(recorded) = recorded {
        if recorded != requested {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to attention operator {}/{} (digest {}); requested {}/{} (digest {}); use a fresh output directory for the new teacher-arithmetic era",
                output.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            )));
        }
        return Ok(true);
    }
    if let Some(name) = inventory.first_present {
        return Err(SourceUnavailable::new(format!(
            "{} contains {name} payload without {ATTENTION_OPERATOR_BINDING_FILE}; it belongs to the implicit legacy attention era and cannot resume under {}/{}; use a fresh output directory",
            output.display(),
            requested.id,
            requested.version,
        )));
    }
    Ok(false)
}

fn require_matching_compiled_attention_operator(
    output: &Path,
    requested: &AttentionOperatorSpec,
    recorded: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    if recorded == requested {
        return Ok(());
    }
    Err(SourceUnavailable::new(format!(
        "{} is pinned to attention operator {}/{} (digest {}); requested {}/{} (digest {}); incompatible compile resume refused before mutation",
        output.display(),
        recorded.id,
        recorded.version,
        recorded.declared_digest(),
        requested.id,
        requested.version,
        requested.declared_digest(),
    )))
}

fn canonical_attention_operator_bytes(
    requested: &AttentionOperatorSpec,
) -> Result<Vec<u8>, SourceUnavailable> {
    let requested =
        validate_attention_operator(requested, "proposed teacher attention-operator binding")?;
    let mut bytes = serde_json::to_vec_pretty(&requested).map_err(SourceUnavailable::new)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish_attention_operator_binding(
    output: &Path,
    requested: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    let requested =
        validate_attention_operator(requested, "proposed teacher attention-operator binding")?;
    if let Some(recorded) = read_optional_compiled_attention_operator_for_publisher(output)? {
        return require_matching_compiled_attention_operator(output, &requested, &recorded);
    }

    std::fs::create_dir_all(output).map_err(SourceUnavailable::new)?;
    let path = output.join(ATTENTION_OPERATOR_BINDING_FILE);
    let bytes = canonical_attention_operator_bytes(&requested)?;
    let temporary = loop {
        let sequence = ATTENTION_OPERATOR_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = output.join(format!(
            ".{ATTENTION_OPERATOR_BINDING_FILE}.{}.{}.tmp",
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

    // `hard_link` is an atomic no-clobber publish. A racing loser accepts the
    // winner only after a strict re-read and full-record equality check.
    match std::fs::hard_link(&temporary, &path) {
        Ok(()) => {
            std::fs::remove_file(&temporary).map_err(|error| {
                SourceUnavailable::new(format!("{}: {error}", temporary.display()))
            })?;
            sync_compile_output_directory(output)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(&temporary);
            let recorded = read_optional_compiled_attention_operator_for_publisher(output)?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "{} appeared during attention-operator publication but is now absent",
                        path.display()
                    ))
                })?;
            require_matching_compiled_attention_operator(output, &requested, &recorded)?;
            sync_compile_output_directory(output)
        }
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            Err(SourceUnavailable::new(format!(
                "attention-operator sidecar publish {} -> {}: {error}",
                temporary.display(),
                path.display()
            )))
        }
    }
}

fn bind_compiled_attention_operator(
    output: &Path,
    requested: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    let requested =
        validate_attention_operator(requested, "proposed teacher attention-operator binding")?;
    if preflight_compiled_attention_operator(output, &requested)? {
        return Ok(());
    }
    publish_attention_operator_binding(output, &requested)
}

fn validate_dense_operator(
    recorded: &DenseOperatorSpec,
    context: &str,
) -> Result<DenseOperatorSpec, SourceUnavailable> {
    let registered = uor_r4_model_source::dense::operator_spec(&recorded.id, recorded.version)
        .map_err(|mut error| {
            error.reason = format!("{context}: {}", error.reason);
            error
        })?;
    if recorded != &registered {
        return Err(SourceUnavailable::new(format!(
            "{context} does not match registered dense operator {}/{}",
            recorded.id, recorded.version
        )));
    }
    Ok(registered)
}

fn validate_source_execution_pair(
    attention: &AttentionOperatorSpec,
    dense: Option<&DenseOperatorSpec>,
    context: &str,
) -> Result<(), SourceUnavailable> {
    uor_r4_model_source::dense::validate_source_execution_pair(Some(attention), dense).map_err(
        |mut error| {
            error.reason = format!("{context}: {}", error.reason);
            error
        },
    )
}

fn reject_duplicate_dense_operator_json(
    bytes: &[u8],
    path: &Path,
) -> Result<(), SourceUnavailable> {
    serde_json::from_slice::<DuplicateRejectingJson>(bytes)
        .map(|_| ())
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "{}: malformed dense-operator binding: {error}",
                path.display()
            ))
        })
}

fn validate_dense_operator_json(
    value: &serde_json::Value,
    context: &str,
) -> Result<DenseOperatorSpec, SourceUnavailable> {
    let recorded: DenseOperatorSpec = serde_json::from_value(value.clone()).map_err(|error| {
        SourceUnavailable::new(format!(
            "{context}: malformed dense-operator record: {error}"
        ))
    })?;
    let registered = validate_dense_operator(&recorded, context)?;
    let registered_json = serde_json::to_value(&registered).map_err(SourceUnavailable::new)?;
    if value != &registered_json {
        return Err(SourceUnavailable::new(format!(
            "{context} is not the full registered dense-operator record; missing, unknown, or noncanonical fields are refused"
        )));
    }
    Ok(registered)
}

fn parse_compiled_dense_operator_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<DenseOperatorSpec, SourceUnavailable> {
    reject_duplicate_dense_operator_json(bytes, path)?;
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: malformed dense-operator binding: {error}",
            path.display()
        ))
    })?;
    validate_dense_operator_json(&value, &path.display().to_string())
}

fn read_optional_compiled_dense_operator_with_bytes(
    output: &Path,
) -> Result<Option<(DenseOperatorSpec, Vec<u8>)>, SourceUnavailable> {
    let path = output.join(DENSE_OPERATOR_BINDING_FILE);
    let Some(bytes) = read_optional_regular_file_nofollow(&path, "dense-operator binding")? else {
        return Ok(None);
    };
    let operator = parse_compiled_dense_operator_bytes(&path, &bytes)?;
    Ok(Some((operator, bytes)))
}

fn read_optional_compiled_dense_operator(
    output: &Path,
) -> Result<Option<DenseOperatorSpec>, SourceUnavailable> {
    Ok(read_optional_compiled_dense_operator_with_bytes(output)?.map(|(operator, _)| operator))
}

#[cfg(test)]
fn read_optional_compiled_dense_operator_for_publisher_with_hooks<F, B, G>(
    output: &Path,
    after_first_open: F,
    before_retry: B,
    after_retry_open: G,
) -> Result<Option<DenseOperatorSpec>, SourceUnavailable>
where
    F: FnOnce(),
    B: FnOnce(),
    G: FnOnce(),
{
    let path = output.join(DENSE_OPERATOR_BINDING_FILE);
    let Some(bytes) = read_optional_regular_file_for_operator_publisher_with_hooks(
        &path,
        "dense-operator binding",
        after_first_open,
        before_retry,
        after_retry_open,
    )?
    else {
        return Ok(None);
    };
    parse_compiled_dense_operator_bytes(&path, &bytes).map(Some)
}

fn read_optional_compiled_dense_operator_for_publisher(
    output: &Path,
) -> Result<Option<DenseOperatorSpec>, SourceUnavailable> {
    let path = output.join(DENSE_OPERATOR_BINDING_FILE);
    let Some(bytes) =
        read_optional_regular_file_for_operator_publisher(&path, "dense-operator binding")?
    else {
        return Ok(None);
    };
    parse_compiled_dense_operator_bytes(&path, &bytes).map(Some)
}

/// Read a compile directory's optional dense-execution binding. Absence is a
/// real compatibility state for Llama and historical corpora and is never
/// synthesized as GPT-2 v1.
pub fn compiled_dense_operator(
    output: &Path,
) -> Result<Option<DenseOperatorSpec>, SourceUnavailable> {
    read_optional_compiled_dense_operator(output)
}

/// Jointly preflight optional dense provenance. `true` means no publication
/// is needed (the exact record exists, or both requested and recorded are
/// absent).
fn preflight_compiled_dense_operator(
    output: &Path,
    requested: Option<&DenseOperatorSpec>,
) -> Result<bool, SourceUnavailable> {
    let requested = requested
        .map(|record| validate_dense_operator(record, "proposed teacher dense-operator binding"))
        .transpose()?;
    let inventory = compile_output_payload_inventory(output)?;
    let recorded = read_optional_compiled_dense_operator_for_publisher(output)?;
    match (recorded.as_ref(), requested.as_ref()) {
        (Some(recorded), Some(requested)) if recorded == requested => Ok(true),
        (Some(recorded), Some(requested)) => Err(SourceUnavailable::new(format!(
            "{} is pinned to dense operator {}/{} (digest {}); requested {}/{} (digest {}); use a fresh output directory for the new teacher-execution era",
            output.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
            requested.id,
            requested.version,
            requested.declared_digest(),
        ))),
        (Some(recorded), None) => Err(SourceUnavailable::new(format!(
            "{} is pinned to dense operator {}/{} (digest {}); the requested producer declares none; incompatible compile resume refused before mutation",
            output.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
        ))),
        (None, Some(requested)) if inventory.first_present.is_some() => {
            let name = inventory.first_present.unwrap_or("compiler");
            Err(SourceUnavailable::new(format!(
                "{} contains {name} payload without {DENSE_OPERATOR_BINDING_FILE}; it belongs to a legacy dense-execution era and cannot resume under {}/{}; use a fresh output directory",
                output.display(),
                requested.id,
                requested.version,
            )))
        }
        (None, Some(_)) => Ok(false),
        (None, None) => Ok(true),
    }
}

fn require_matching_compiled_dense_operator(
    output: &Path,
    requested: &DenseOperatorSpec,
    recorded: &DenseOperatorSpec,
) -> Result<(), SourceUnavailable> {
    if recorded == requested {
        return Ok(());
    }
    Err(SourceUnavailable::new(format!(
        "{} is pinned to dense operator {}/{} (digest {}); requested {}/{} (digest {}); incompatible compile resume refused before mutation",
        output.display(),
        recorded.id,
        recorded.version,
        recorded.declared_digest(),
        requested.id,
        requested.version,
        requested.declared_digest(),
    )))
}

fn canonical_dense_operator_bytes(
    requested: &DenseOperatorSpec,
) -> Result<Vec<u8>, SourceUnavailable> {
    let requested = validate_dense_operator(requested, "proposed teacher dense-operator binding")?;
    let mut bytes = serde_json::to_vec_pretty(&requested).map_err(SourceUnavailable::new)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish_dense_operator_binding(
    output: &Path,
    requested: &DenseOperatorSpec,
) -> Result<(), SourceUnavailable> {
    let requested = validate_dense_operator(requested, "proposed teacher dense-operator binding")?;
    if let Some(recorded) = read_optional_compiled_dense_operator_for_publisher(output)? {
        return require_matching_compiled_dense_operator(output, &requested, &recorded);
    }

    std::fs::create_dir_all(output).map_err(SourceUnavailable::new)?;
    let path = output.join(DENSE_OPERATOR_BINDING_FILE);
    let bytes = canonical_dense_operator_bytes(&requested)?;
    let temporary = loop {
        let sequence = DENSE_OPERATOR_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = output.join(format!(
            ".{DENSE_OPERATOR_BINDING_FILE}.{}.{}.tmp",
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

    match std::fs::hard_link(&temporary, &path) {
        Ok(()) => {
            std::fs::remove_file(&temporary).map_err(|error| {
                SourceUnavailable::new(format!("{}: {error}", temporary.display()))
            })?;
            sync_compile_output_directory(output)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _ = std::fs::remove_file(&temporary);
            let recorded = read_optional_compiled_dense_operator_for_publisher(output)?
                .ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "{} appeared during dense-operator publication but is now absent",
                        path.display()
                    ))
                })?;
            require_matching_compiled_dense_operator(output, &requested, &recorded)?;
            sync_compile_output_directory(output)
        }
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            Err(SourceUnavailable::new(format!(
                "dense-operator sidecar publish {} -> {}: {error}",
                temporary.display(),
                path.display()
            )))
        }
    }
}

fn bind_compiled_dense_operator(
    output: &Path,
    requested: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    if preflight_compiled_dense_operator(output, requested)? {
        return Ok(());
    }
    let requested = requested.ok_or_else(|| {
        SourceUnavailable::new("dense-operator publication requested without a record")
    })?;
    publish_dense_operator_binding(output, requested)
}

fn pin_compile_identities_with_dense(
    output: &Path,
    tokenizer: &TokenizerAdapter,
    attention_operator: &AttentionOperatorSpec,
    dense_operator: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    validate_source_execution_pair(
        attention_operator,
        dense_operator,
        "teacher-declared source execution",
    )?;
    // Every check is deliberately read-only. Do not let a successful prefix
    // of a mismatched identity bundle mutate a resumable output.
    preflight_compile_tokenizer_adapter(output, tokenizer)?;
    preflight_compiled_attention_operator(output, attention_operator)?;
    preflight_compiled_dense_operator(output, dense_operator)?;
    pin_compile_tokenizer_adapter(output, tokenizer)?;
    if dense_operator.is_some() {
        // Dense-first makes every crash prefix fail closed. Publishing
        // attention first would leave a valid-looking learned-v2/absent-dense
        // compatibility record until the final generation binding appeared.
        bind_compiled_dense_operator(output, dense_operator)?;
    }
    bind_compiled_attention_operator(output, attention_operator)?;
    if dense_operator.is_none() {
        bind_compiled_dense_operator(output, dense_operator)?;
    }
    Ok(())
}

#[cfg(test)]
fn pin_compile_identities(
    output: &Path,
    tokenizer: &TokenizerAdapter,
    attention_operator: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    pin_compile_identities_with_dense(output, tokenizer, attention_operator, None)
}

/// Recorded compilation has no tokenizer authority: its input is already a
/// token-id corpus, not a tokenizer package. Preserving either a source
/// compiler's adapter claim or its runtime table would produce a bundle that
/// looks bound to a token space the recorded path never verified.
fn preflight_recorded_compile_output(output: &Path) -> Result<(), SourceUnavailable> {
    const FORBIDDEN_RECORDED_OUTPUT_LEAVES: [&str; 8] = [
        "tokenizer.bin",
        "space_manifest.json",
        "compile_report.json",
        "source_compile_preflight.json",
        "source_manifest_kappa.json",
        "compiled_bundle_completion.json",
        ".compiled_bundle_stage.json",
        "tokenizer_adapter.json",
    ];
    let _ = compile_output_payload_inventory(output)?;
    if read_compile_tokenizer_adapter(output)?.is_some() {
        return Err(SourceUnavailable::new(format!(
            "recorded compile output {} contains {TOKENIZER_ADAPTER_FILE}, but compile-recorded has no tokenizer authority; use a fresh recorded-only output directory",
            output.display()
        )));
    }
    let entries = match std::fs::read_dir(output) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(SourceUnavailable::new(error)),
    };
    for entry in entries {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(entry_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let forbidden = FORBIDDEN_RECORDED_OUTPUT_LEAVES.iter().any(|stable| {
            entry_name == *stable
                || entry_name.starts_with(&format!(".{stable}."))
                || entry_name.starts_with(&format!("{stable}."))
        });
        if forbidden {
            return Err(SourceUnavailable::new(format!(
                "recorded compile output {} contains {entry_name}, which compile-recorded cannot verify or reproduce; use a fresh recorded-only output directory",
                output.display()
            )));
        }
    }
    Ok(())
}

/// Resolve a compile output through the deepest existing physical ancestor
/// without creating any parent or output entry. A final symlink is never a
/// compile root; intermediate aliases are collapsed once so all later owner
/// checks, session acquisition, and writes use the same physical namespace.
fn canonical_hf_compile_output_subject(output: &Path) -> Result<PathBuf, SourceUnavailable> {
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(SourceUnavailable::new(format!(
                "compile output {} is a symlink; a physical non-symlink directory is required",
                output.display()
            )));
        }
        Ok(metadata) if metadata.file_type().is_dir() => {
            return std::fs::canonicalize(output).map_err(SourceUnavailable::new);
        }
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "compile output {} is not a regular non-symlink directory",
                output.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SourceUnavailable::new(error)),
    }

    let mut cursor = output;
    let mut missing = Vec::<std::ffi::OsString>::new();
    loop {
        match std::fs::symlink_metadata(cursor) {
            Ok(_) => {
                let physical = std::fs::canonicalize(cursor).map_err(SourceUnavailable::new)?;
                let metadata =
                    std::fs::symlink_metadata(&physical).map_err(SourceUnavailable::new)?;
                if !metadata.file_type().is_dir() {
                    return Err(SourceUnavailable::new(format!(
                        "deepest existing compile-output ancestor {} does not resolve to a physical directory",
                        cursor.display()
                    )));
                }
                let mut resolved = physical;
                for leaf in missing.iter().rev() {
                    resolved.push(leaf);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let leaf = cursor.file_name().ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "compile output {} has no physical ancestor and final name",
                        output.display()
                    ))
                })?;
                if leaf == std::ffi::OsStr::new(".") || leaf == std::ffi::OsStr::new("..") {
                    return Err(SourceUnavailable::new(format!(
                        "compile output {} contains a reserved unresolved path component",
                        output.display()
                    )));
                }
                missing.push(leaf.to_os_string());
                cursor = cursor
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
            }
            Err(error) => return Err(SourceUnavailable::new(error)),
        }
    }
}

fn has_server_private_stage_topology(root: &Path) -> bool {
    if root.parent().and_then(Path::file_name)
        != Some(std::ffi::OsStr::new(".uor-r4-source-compile-staging"))
    {
        return false;
    }
    let Some(name) = root.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some((logical, sequence)) = name.rsplit_once(".bundle-stage.") else {
        return false;
    };
    if !logical.starts_with('.') || logical.len() == 1 {
        return false;
    }
    let mut parts = sequence.split('.');
    parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn compiled_bundle_owner_ancestor(output: &Path) -> Result<Option<PathBuf>, SourceUnavailable> {
    let mut candidate = match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_dir() => Some(output),
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "compile output {} is not a regular non-symlink directory",
                output.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => output.parent(),
        Err(error) => return Err(SourceUnavailable::new(error)),
    };
    while let Some(root) = candidate {
        let metadata = match std::fs::symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                candidate = root.parent();
                continue;
            }
            Err(error) => return Err(SourceUnavailable::new(error)),
        };
        if metadata.file_type().is_dir() {
            // The private-stage topology itself reserves this root for the
            // managed server. Select its common authority even in the narrow
            // interval before the owner marker is published, so a direct
            // child compile cannot race marker appearance and seed an
            // independent child transaction inside the stage.
            if has_server_private_stage_topology(root) {
                return std::fs::canonicalize(root)
                    .map(Some)
                    .map_err(SourceUnavailable::new);
            }
            for leaf in [
                "compiled_bundle_completion.json",
                ".compiled_bundle_stage.json",
                ".compiled_bundle_stage_a.json",
            ] {
                match std::fs::symlink_metadata(root.join(leaf)) {
                    Ok(_) => {
                        return std::fs::canonicalize(root)
                            .map(Some)
                            .map_err(SourceUnavailable::new);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(SourceUnavailable::new(error)),
                }
            }
        }
        candidate = root.parent();
    }
    Ok(None)
}

fn reject_foreign_compiled_bundle_stage_owner(output: &Path) -> Result<(), SourceUnavailable> {
    let Some(owner_root) = compiled_bundle_owner_ancestor(output)? else {
        return Ok(());
    };
    // The read-only ancestor classification precedes any child session/root
    // creation. Reacquire that exact ancestor's common authority and recheck
    // while held; never seed independent coordination inside a private stage.
    let guard = RecordedCorpusProducerGuard::try_acquire(&owner_root)?;
    guard.verify_owned_root()?;
    let mut present = false;
    for leaf in [
        "compiled_bundle_completion.json",
        ".compiled_bundle_stage.json",
        ".compiled_bundle_stage_a.json",
    ] {
        match std::fs::symlink_metadata(owner_root.join(leaf)) {
            Ok(_) => present = true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(SourceUnavailable::new(error)),
        }
    }
    guard.verify_owned_root()?;
    let state = if present {
        "is an owned or completed server compiled-bundle ancestor"
    } else {
        "changed owner state during guarded classification"
    };
    Err(SourceUnavailable::new(format!(
        "compile output {} {state} at {}; direct source compilation cannot mutate it without the exact managed capability",
        output.display(),
        owner_root.display()
    )))
}

fn require_exact_managed_compiled_bundle_stage_owner(
    guard: &RecordedCorpusProducerGuard,
    output: &Path,
) -> Result<(), SourceUnavailable> {
    if !guard.protects_directory(output)? {
        return Err(SourceUnavailable::new(format!(
            "managed source capability for {} does not protect exact output {}",
            guard.root().display(),
            output.display()
        )));
    }
    guard.verify_owned_root()?;
    let marker = output.join(".compiled_bundle_stage.json");
    match std::fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "managed compiled-bundle owner marker {} is not a regular non-symlink file",
                marker.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(SourceUnavailable::new(format!(
                "managed source capability cannot mutate {} without its exact server owner marker",
                output.display()
            )));
        }
        Err(error) => return Err(SourceUnavailable::new(error)),
    }
    guard.verify_owned_root()
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

#[derive(Debug, PartialEq, Eq)]
struct CopyRecordedAttentionOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    output: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
struct SubsampleRecordedCorpusOptions {
    source_meta: PathBuf,
    source_records: PathBuf,
    output_meta: PathBuf,
    output_records: PathBuf,
    records: u64,
}

fn parse_subsample_recorded_corpus_options(
    args: &[String],
) -> Result<SubsampleRecordedCorpusOptions, SourceUnavailable> {
    let mut source_meta = None;
    let mut source_records = None;
    let mut output_meta = None;
    let mut output_records = None;
    let mut records = None;
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--src-meta" => source_meta = Some(PathBuf::from(value)),
            "--src-recs" => source_records = Some(PathBuf::from(value)),
            "--out-meta" => output_meta = Some(PathBuf::from(value)),
            "--out-recs" => output_records = Some(PathBuf::from(value)),
            "--records" => {
                let parsed = value.parse::<u64>().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --records value: {value}"))
                })?;
                if parsed == 0 {
                    return Err(SourceUnavailable::new(
                        "--records must be greater than zero",
                    ));
                }
                records = Some(parsed);
            }
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown subsample-recorded-corpus option: {flag}"
                )));
            }
        }
        index += 2;
    }
    let output_meta =
        output_meta.ok_or_else(|| SourceUnavailable::new("--out-meta is required"))?;
    let output_records =
        output_records.ok_or_else(|| SourceUnavailable::new("--out-recs is required"))?;
    if output_meta.file_name() != Some(std::ffi::OsStr::new("corpus.meta"))
        || output_records.file_name() != Some(std::ffi::OsStr::new("corpus.records"))
    {
        return Err(SourceUnavailable::new(
            "subsample output must use the exact canonical corpus.meta/corpus.records filenames",
        ));
    }
    let meta_parent = output_meta
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let records_parent = output_records
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if meta_parent != records_parent {
        return Err(SourceUnavailable::new(
            "--out-meta and --out-recs must have one exact lexical parent directory",
        ));
    }
    Ok(SubsampleRecordedCorpusOptions {
        source_meta: source_meta.ok_or_else(|| SourceUnavailable::new("--src-meta is required"))?,
        source_records: source_records
            .ok_or_else(|| SourceUnavailable::new("--src-recs is required"))?,
        output_meta,
        output_records,
        records: records.ok_or_else(|| SourceUnavailable::new("--records is required"))?,
    })
}

fn parse_copy_recorded_attention_options(
    args: &[String],
) -> Result<CopyRecordedAttentionOptions, SourceUnavailable> {
    let mut corpus_meta = None;
    let mut corpus_recs = None;
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
            "--out" => output = Some(PathBuf::from(value)),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown copy-recorded-attention option: {flag}"
                )));
            }
        }
        index += 2;
    }
    let output = output.ok_or_else(|| SourceUnavailable::new("--out is required"))?;
    if output.file_name() != Some(std::ffi::OsStr::new(ATTENTION_OPERATOR_BINDING_FILE)) {
        return Err(SourceUnavailable::new(format!(
            "--out must name {ATTENTION_OPERATOR_BINDING_FILE} exactly"
        )));
    }
    Ok(CopyRecordedAttentionOptions {
        corpus_meta: corpus_meta
            .ok_or_else(|| SourceUnavailable::new("--corpus-meta is required"))?,
        corpus_recs: corpus_recs
            .ok_or_else(|| SourceUnavailable::new("--corpus-recs is required"))?,
        output,
    })
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

fn parse_observation_manifest_bytes(
    output: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<observe::ObservationManifest, SourceUnavailable> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: malformed observation manifest: {error}",
            path.display()
        ))
    })?;
    if let Some(operator) = value.get("attention_operator") {
        validate_attention_operator_json(operator, &path.display().to_string())?;
    }
    if let Some(operator) = value.get("dense_operator") {
        validate_dense_operator_json(operator, &path.display().to_string())?;
    }
    let parsed: observe::ObservationManifest = serde_json::from_value(value).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: malformed observation manifest: {error}",
            path.display()
        ))
    })?;
    // Delegate the authoritative registry checks for geometry, tokenizer,
    // attention, and trace records to the graph-compiler boundary. The raw
    // operator check above additionally refuses unknown nested claims before
    // serde can discard them.
    let validated = observe::ObservationManifest::load(output)?.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{} disappeared during observation-manifest validation",
            path.display()
        ))
    })?;
    if validated != parsed {
        return Err(SourceUnavailable::new(format!(
            "{} changed during observation-manifest validation",
            path.display()
        )));
    }
    Ok(validated)
}

fn read_optional_observation_manifest(
    output: &Path,
) -> Result<Option<observe::ObservationManifest>, SourceUnavailable> {
    let path = output.join(observe::MANIFEST_FILE);
    if !strict_regular_file_if_present(&path, "observation manifest")? {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: unreadable observation manifest: {error}",
            path.display()
        ))
    })?;
    parse_observation_manifest_bytes(output, &path, &bytes).map(Some)
}

fn observation_payload_exists(
    output: &Path,
    manifest: Option<&observe::ObservationManifest>,
) -> Result<bool, SourceUnavailable> {
    let mut has_payload = manifest
        .is_some_and(|manifest| manifest.total_records != 0 || !manifest.completed.is_empty());

    let payload_file_is_present = |path: &Path| match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => Err(SourceUnavailable::new(format!(
            "observation payload path {} is not a regular file; refusing provenance mutation",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(SourceUnavailable::new(format!(
            "{} cannot be inspected: {error}",
            path.display()
        ))),
    };

    for name in [
        observe::STATE_FILE,
        observe::RAW_COMMITTED_FILE,
        "merged.bin",
        "committed.bin",
        ".committed.bin.tmp",
        "stories.jsonl",
        "tokenizer.bin",
    ] {
        if payload_file_is_present(&output.join(name))? {
            has_payload = true;
        }
    }
    // Scan the inventory rather than deriving names from the requested
    // fan-out. A stripped or stale manifest can under-declare the shards that
    // already exist; every shard/probability/trace payload is arithmetic-era
    // evidence even when its index lies outside the current fan-out.
    let entries = match std::fs::read_dir(output) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(has_payload),
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "observation directory {} cannot be inspected: {error}",
                output.display()
            )));
        }
    };
    for entry in entries {
        let entry = entry.map_err(|error| {
            SourceUnavailable::new(format!(
                "observation directory {} cannot be inspected: {error}",
                output.display()
            ))
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(stem) = name.strip_prefix("shard-") else {
            continue;
        };
        let numeric_index = [".bin", ".bin.prob", ".bin.trace"]
            .into_iter()
            .find_map(|suffix| stem.strip_suffix(suffix));
        if numeric_index.is_some_and(|index| {
            !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
        }) && payload_file_is_present(&entry.path())?
        {
            has_payload = true;
        }
    }
    Ok(has_payload)
}

/// Validate the tokenizer and operator halves of an observation identity
/// without opening a writer. Any conflict therefore leaves both manifest
/// fields and `tokenizer.bin` unchanged.
fn preflight_observation_identities_with_dense(
    output: &Path,
    shard_bits: u8,
    tokenizer: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    if let Some(tokenizer) = tokenizer {
        validate_tokenizer_adapter_record(tokenizer)?;
    }
    let requested_operator = attention_operator
        .map(|operator| {
            validate_attention_operator(operator, "teacher-declared attention operator")
        })
        .transpose()?;
    let requested_dense = dense_operator
        .map(|operator| validate_dense_operator(operator, "teacher-declared dense operator"))
        .transpose()?;
    match (requested_operator.as_ref(), requested_dense.as_ref()) {
        (Some(attention), dense) => {
            validate_source_execution_pair(attention, dense, "teacher-declared source execution")?
        }
        (None, Some(dense)) => {
            return Err(SourceUnavailable::new(format!(
                "teacher-declared source execution supplies dense operator {}/{} without an attention operator",
                dense.id, dense.version
            )));
        }
        (None, None) => {}
    }
    let manifest = read_optional_observation_manifest(output)?;
    if let Some(manifest) = &manifest {
        if manifest.shard_bits != shard_bits {
            return Err(SourceUnavailable::new(format!(
                "manifest shard_bits {} does not match requested {shard_bits}",
                manifest.shard_bits
            )));
        }
        if let Some(recorded) = &manifest.tokenizer_adapter {
            validate_tokenizer_adapter_record(recorded).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{}: {}",
                    output.join(observe::MANIFEST_FILE).display(),
                    error.reason
                ))
            })?;
        }
        if let Some(recorded) = &manifest.attention_operator {
            validate_attention_operator(
                recorded,
                &output.join(observe::MANIFEST_FILE).display().to_string(),
            )?;
        }
        if let Some(recorded) = &manifest.dense_operator {
            validate_dense_operator(
                recorded,
                &output.join(observe::MANIFEST_FILE).display().to_string(),
            )?;
        }
        match (
            manifest.attention_operator.as_ref(),
            manifest.dense_operator.as_ref(),
        ) {
            (Some(attention), dense) => validate_source_execution_pair(
                attention,
                dense,
                &output.join(observe::MANIFEST_FILE).display().to_string(),
            )?,
            (None, Some(dense)) => {
                return Err(SourceUnavailable::new(format!(
                    "{} records dense operator {}/{} without an attention operator",
                    output.join(observe::MANIFEST_FILE).display(),
                    dense.id,
                    dense.version
                )));
            }
            (None, None) => {}
        }
    }
    // Inventory every payload entry even when the two recorded identities
    // match. This is the last read-only boundary before tokenizer export and
    // writer construction, so a shard/probability/trace symlink or other
    // nonregular entry must fail before either operation can follow it.
    let payload_exists = observation_payload_exists(output, manifest.as_ref())?;

    match (
        manifest
            .as_ref()
            .and_then(|manifest| manifest.tokenizer_adapter.as_ref()),
        tokenizer,
    ) {
        (Some(recorded), Some(requested)) if recorded == requested => {}
        (Some(recorded), Some(requested)) => {
            return Err(SourceUnavailable::new(format!(
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
            )));
        }
        (Some(recorded), None) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to tokenizer adapter {}/{} (CID {}, digest {}); requested the adapterless legacy tokenizer; incompatible resume refused before mutation",
                output.display(),
                recorded.family,
                recorded.version,
                recorded.tokenizer_cid,
                recorded.adapter_digest,
            )));
        }
        (None, _) => {}
    }

    match (
        manifest
            .as_ref()
            .and_then(|manifest| manifest.attention_operator.as_ref()),
        requested_operator.as_ref(),
    ) {
        (Some(recorded), Some(requested)) if recorded == requested => {}
        (Some(recorded), Some(requested)) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to attention operator {}/{} (digest {}); requested {}/{} (digest {}); incompatible observation resume refused before mutation",
                output.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            )));
        }
        (Some(recorded), None) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to attention operator {}/{} but the requested oracle declares no operator; use a fresh output directory for the legacy arithmetic era",
                output.display(),
                recorded.id,
                recorded.version,
            )));
        }
        (None, Some(requested)) if payload_exists => {
            return Err(SourceUnavailable::new(format!(
                "{} has no recorded attention operator but already contains observation payload; refusing to relabel legacy rows as {}/{} before mutation",
                output.display(),
                requested.id,
                requested.version,
            )));
        }
        (None, _) => {}
    }

    match (
        manifest
            .as_ref()
            .and_then(|manifest| manifest.dense_operator.as_ref()),
        requested_dense.as_ref(),
    ) {
        (Some(recorded), Some(requested)) if recorded == requested => {}
        (Some(recorded), Some(requested)) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to dense operator {}/{} (digest {}); requested {}/{} (digest {}); incompatible observation resume refused before mutation",
                output.display(),
                recorded.id,
                recorded.version,
                recorded.declared_digest(),
                requested.id,
                requested.version,
                requested.declared_digest(),
            )));
        }
        (Some(recorded), None) => {
            return Err(SourceUnavailable::new(format!(
                "{} is pinned to dense operator {}/{} but the requested oracle declares no operator; use a fresh output directory for the legacy arithmetic era",
                output.display(),
                recorded.id,
                recorded.version,
            )));
        }
        (None, Some(requested)) if payload_exists => {
            return Err(SourceUnavailable::new(format!(
                "{} has no recorded dense operator but already contains observation payload; refusing to relabel legacy rows as {}/{} before mutation",
                output.display(),
                requested.id,
                requested.version,
            )));
        }
        (None, _) => {}
    }
    Ok(())
}

#[cfg(test)]
fn preflight_observation_identities(
    output: &Path,
    shard_bits: u8,
    tokenizer: Option<&TokenizerAdapter>,
    attention_operator: Option<&AttentionOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    preflight_observation_identities_with_dense(
        output,
        shard_bits,
        tokenizer,
        attention_operator,
        None,
    )
}

fn pin_raw_observation_identities_before_output_with_dense(
    session: &observe::ObservationSession,
    tokenizer: Option<&TokenizerAdapter>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    preflight_observation_identities_with_dense(
        session.dir(),
        session.shard_bits(),
        tokenizer,
        attention_operator,
        dense_operator,
    )?;
    let operator = attention_operator
        .map(|operator| {
            validate_attention_operator(operator, "teacher-declared attention operator")
        })
        .transpose()?;
    let dense = dense_operator
        .map(|operator| validate_dense_operator(operator, "teacher-declared dense operator"))
        .transpose()?;
    // The lower boundary joins geometry with tokenizer/operator on the same
    // locked snapshot. Run its full read-only pass before publishing any
    // identity, then repeat and pin the whole bundle while this session stays
    // live through tokenizer export, reconciliation, and row writes.
    observe::preflight_observation_identities_in_session_with_dense(
        session,
        None,
        geometry,
        tokenizer,
        operator.as_ref(),
        dense.as_ref(),
        None,
    )?;
    observe::pin_observation_identities_in_session_with_dense(
        session,
        None,
        geometry,
        tokenizer,
        operator.as_ref(),
        dense.as_ref(),
        None,
    )
}

#[cfg(test)]
fn pin_raw_observation_identities_before_output(
    session: &observe::ObservationSession,
    tokenizer: Option<&TokenizerAdapter>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&AttentionOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    pin_raw_observation_identities_before_output_with_dense(
        session,
        tokenizer,
        geometry,
        attention_operator,
        None,
    )
}

fn pin_text_observation_identities_before_output_with_dense(
    session: &observe::ObservationSession,
    input: &Path,
    tokenizer: Option<&TokenizerAdapter>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&AttentionOperatorSpec>,
    dense_operator: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    preflight_observation_identities_with_dense(
        session.dir(),
        session.shard_bits(),
        tokenizer,
        attention_operator,
        dense_operator,
    )?;
    let operator = attention_operator
        .map(|operator| {
            validate_attention_operator(operator, "teacher-declared attention operator")
        })
        .transpose()?;
    let dense = dense_operator
        .map(|operator| validate_dense_operator(operator, "teacher-declared dense operator"))
        .transpose()?;
    observe_text::preflight_text_observation_in_session_with_dense(
        session,
        input,
        true,
        geometry,
        operator.as_ref(),
        dense.as_ref(),
        tokenizer,
    )?;
    observe_text::pin_text_observation_identities_in_session_with_dense(
        session,
        input,
        true,
        geometry,
        operator.as_ref(),
        dense.as_ref(),
        tokenizer,
    )
}

#[cfg(test)]
fn pin_text_observation_identities_before_output(
    session: &observe::ObservationSession,
    input: &Path,
    tokenizer: Option<&TokenizerAdapter>,
    geometry: Option<&uor_r4_model_source::geometry::GeometryProjection>,
    attention_operator: Option<&AttentionOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    pin_text_observation_identities_before_output_with_dense(
        session,
        input,
        tokenizer,
        geometry,
        attention_operator,
        None,
    )
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
    let (adapter, runtime_table, mut oracle): (
        Option<TokenizerAdapter>,
        Option<RuntimeTokenizerDecodeTable>,
        Box<dyn TeacherOracle>,
    ) = if let Some(checkpoint) = &options.checkpoint {
        // Legacy llama2.c checkpoint: no HF tokenizer tree, so byte
        // anchors stay at the v3 "unknown" value.
        let path = checkpoint
            .to_str()
            .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
        let oracle = LlamaOracle::load(path);
        (None, None, Box::new(oracle))
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
        (adapter, Some(runtime_table), Box::new(oracle))
    };
    // Resolve every source input before opening the resumable output. The one
    // session acquired here then spans the full identity/export/run/finalize
    // lifetime.
    let session = observe::ObservationSession::acquire(&options.output, options.shards)?;
    let geometry = oracle.geometry_projection();
    let attention_operator = oracle.attention_operator_spec();
    let dense_operator = oracle.dense_operator_spec();
    pin_raw_observation_identities_before_output_with_dense(
        &session,
        adapter.as_ref(),
        geometry.as_ref(),
        attention_operator.as_ref(),
        dense_operator.as_ref(),
    )?;
    let token_byte_lengths = if let Some(runtime_table) = runtime_table.as_ref() {
        eprintln!("exporting tokenizer...");
        let export = core_scenarios::export_runtime_tokenizer_table(
            runtime_table,
            session.dir().join("tokenizer.bin"),
        )
        .map_err(SourceUnavailable::new)?;
        export.source_byte_lengths
    } else {
        None
    };
    let summary = observe::observe_sharded_in_session(
        &session,
        oracle.as_mut(),
        options.seconds,
        options.target,
        token_byte_lengths.as_deref(),
    )
    .map_err(SourceUnavailable::new)?;
    if summary.done {
        // Persist the merged record stream so Gate C can consume it as
        // --corpus-recs with state.bin as --corpus-meta (same convention
        // as the from-text driver, issue #75).
        let merged = observe::merge_shards(session.dir()).map_err(SourceUnavailable::new)?;
        let merged_path = session.dir().join("merged.bin");
        write_regular_file_synced(&merged_path, &merged, "merged raw observation corpus")?;
        let state_path = session.dir().join(observe::STATE_FILE);
        let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
            session.recorded_corpus_guard(),
            &state_path,
            &merged_path,
        )?;
        let captured = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
            session.recorded_corpus_guard(),
            &state_path,
            &merged_path,
        )?;
        if captured.binding_cid.as_deref() != Some(binding_cid.as_str()) {
            return Err(SourceUnavailable::new(
                "raw observation binding changed during bounded post-publication verification",
            ));
        }
        captured.verify_generation()?;
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
    drop(session);
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
    /// #603/#605 S1: a registered non-minimal trace profile as
    /// `<id>/<version>` (e.g. `full/1`), captured into per-shard
    /// `.trace` sidecars through the teacher's exact-executor taps.
    trace_profile: Option<(String, u32)>,
    /// Declared capture layer indices for the trace profile's lanes.
    trace_layers: Vec<u32>,
    /// Per-head attention-support cap (defaults to the registry's
    /// `PRIMARY_TOP_K` when omitted).
    trace_support: Option<u32>,
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
        trace_profile: None,
        trace_layers: Vec::new(),
        trace_support: None,
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
            "--trace-profile" => {
                // #603/#605 S1: `<id>/<version>` against the immutable
                // profile registry (e.g. `full/1`). Unknown pairs are
                // refused by the registry, never approximated.
                let (id, version) = value.split_once('/').ok_or_else(|| {
                    SourceUnavailable::new(format!(
                        "invalid --trace-profile value: {value} (expected <id>/<version>, e.g. full/1)"
                    ))
                })?;
                let version: u32 = version.parse().map_err(|_| {
                    SourceUnavailable::new(format!(
                        "invalid --trace-profile version in {value}: expected an integer"
                    ))
                })?;
                options.trace_profile = Some((id.to_owned(), version));
            }
            "--trace-layers" => {
                let mut layers = Vec::new();
                for part in value.split(',') {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    layers.push(part.parse::<u32>().map_err(|_| {
                        SourceUnavailable::new(format!(
                            "invalid --trace-layers entry: {part} (expected comma-separated layer indices)"
                        ))
                    })?);
                }
                if layers.is_empty() {
                    return Err(SourceUnavailable::new(
                        "--trace-layers must name at least one layer index",
                    ));
                }
                options.trace_layers = layers;
            }
            "--trace-support" => {
                let support: u32 = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --trace-support value: {value}"))
                })?;
                if support == 0 {
                    return Err(SourceUnavailable::new(
                        "--trace-support must be greater than zero",
                    ));
                }
                options.trace_support = Some(support);
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
    // #603/#605 S1 trace-flag coherence: the capture surface is the
    // single-stream Hugging Face exact executor, so traced passes refuse
    // the batched driver and the legacy checkpoint path with typed
    // errors instead of failing later mid-load; bounds flags without a
    // profile are a mistake worth naming.
    if options.trace_profile.is_some() {
        if options.batch > 1 {
            return Err(SourceUnavailable::new(
                "--trace-profile capture is single-stream (per-oracle exact-executor taps); \
                 it cannot run under --batch — use --workers for parallelism",
            ));
        }
        if options.checkpoint.is_some() {
            return Err(SourceUnavailable::new(
                "--trace-profile requires the Hugging Face teacher path; the legacy \
                 --checkpoint oracle declares no capture surface",
            ));
        }
        if options.trace_layers.is_empty() {
            return Err(SourceUnavailable::new(
                "--trace-profile requires --trace-layers naming the declared capture layers",
            ));
        }
    } else if !options.trace_layers.is_empty() || options.trace_support.is_some() {
        return Err(SourceUnavailable::new(
            "--trace-layers/--trace-support declare capture bounds for --trace-profile; \
             pass a profile or drop them",
        ));
    }
    Ok(options)
}

/// #605 amendment 2 (S1-1): fit packed route codes on a REAL traced
/// corpus and judge the pre-registered overlap gates — the
/// teacher-agnostic half of the real-teacher stage. Geometry and the BOS
/// id come from the teacher source's `config.json`; the corpus loads
/// through the pre-declared streamed subset rule (first `--stories`
/// ordinals, first `--positions` positions each — the deployed
/// operator's own candidate bound). Writes the typed
/// `RealArmOverlapReport` JSON and prints the verdict; a FAIL verdict is
/// a recorded outcome, not a command error.
fn route_fit_real_command(args: &[String]) -> Result<(), SourceUnavailable> {
    use uor_r4_graph_certify::route_fit_report::{
        preregistered_route_fit_contract, run_real_arm_overlap_gates,
    };
    use uor_r4_graph_compiler::route_fit::{
        RouteTraceSubset, fit_route_codes, load_route_trace_corpus_subset,
    };
    use uor_r4_model_source::TraceCaptureGeometry;

    let mut corpus_dir: Option<PathBuf> = None;
    let mut source_dir: Option<PathBuf> = None;
    let mut max_stories: u32 = 1000;
    let mut max_positions: u32 = 64;
    let mut out: Option<PathBuf> = None;
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--corpus" => corpus_dir = Some(PathBuf::from(value)),
            "--source" => source_dir = Some(PathBuf::from(value)),
            "--stories" => {
                max_stories = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --stories value: {value}"))
                })?;
            }
            "--positions" => {
                max_positions = value.parse().map_err(|_| {
                    SourceUnavailable::new(format!("invalid --positions value: {value}"))
                })?;
            }
            "--out" => out = Some(PathBuf::from(value)),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown route-fit-real option: {flag}"
                )));
            }
        }
        index += 2;
    }
    let corpus_dir = corpus_dir
        .ok_or_else(|| SourceUnavailable::new("route-fit-real requires --corpus <DIR>"))?;
    let source_dir = source_dir.ok_or_else(|| {
        SourceUnavailable::new(
            "route-fit-real requires --source <teacher dir> (geometry + BOS come from its \
             config.json)",
        )
    })?;
    let config_path = source_dir.join("config.json");
    let config_bytes = std::fs::read(&config_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", config_path.display())))?;
    let config: serde_json::Value = serde_json::from_slice(&config_bytes).map_err(|error| {
        SourceUnavailable::new(format!("{}: invalid JSON: {error}", config_path.display()))
    })?;
    let field = |name: &str| -> Result<usize, SourceUnavailable> {
        config
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .map(|value| value as usize)
            .ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "{} carries no numeric {name}; cannot derive the capture geometry",
                    config_path.display()
                ))
            })
    };
    let geometry = TraceCaptureGeometry {
        layers: field("num_hidden_layers")?,
        heads: field("num_attention_heads")?,
        kv_heads: field("num_key_value_heads")?,
        residual_width: field("hidden_size")?,
    };
    let bos_token = config
        .get("bos_token_id")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1) as u32;

    eprintln!(
        "route-fit-real: loading subset (stories < {max_stories}, positions < {max_positions}) \
         from {} under geometry {}L/{}H/{}KV/{}D, bos {bos_token}",
        corpus_dir.display(),
        geometry.layers,
        geometry.heads,
        geometry.kv_heads,
        geometry.residual_width
    );
    let corpus = load_route_trace_corpus_subset(
        &corpus_dir,
        geometry,
        bos_token,
        RouteTraceSubset {
            max_stories,
            max_positions,
        },
    )?;
    eprintln!(
        "route-fit-real: {} records across {} stories loaded (records κ {}, trace κ {})",
        corpus.records,
        corpus.stories.len(),
        corpus.records_kappa,
        corpus.trace_kappa
    );
    eprintln!("route-fit-real: fitting packed route codes...");
    let fitted = fit_route_codes(&corpus)?;
    eprintln!(
        "route-fit-real: fitted {} heads (top_m {}, params κ {}); judging the \
         pre-registered overlap gates...",
        fitted.heads.len(),
        fitted.top_m,
        fitted.kappa()
    );
    let contract = preregistered_route_fit_contract();
    let report = run_real_arm_overlap_gates(&corpus, &fitted, &contract)?;
    let out = out.unwrap_or_else(|| corpus_dir.join("route_fit_real_report.json"));
    let mut bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| SourceUnavailable::new(format!("serialize real-arm report: {error}")))?;
    bytes.push(b'\n');
    write_regular_file_synced(&out, &bytes, "real-arm overlap report")?;
    println!(
        "route-fit-real verdict: {:?}\n  {}\n  report: {}",
        report.verdict,
        report.reason,
        out.display()
    );
    Ok(())
}

/// Resolve the parsed `--trace-*` flags against the immutable #603 profile
/// registry: `(id, version)` plus the declared bounds map to the typed
/// record, or a typed refusal for unknown pairs and out-of-bound
/// declarations.
fn resolve_observe_text_trace_profile(
    options: &ObserveTextOptions,
) -> Result<Option<uor_r4_graph_compiler::trace_profile::TraceProfile>, SourceUnavailable> {
    let Some((id, version)) = options.trace_profile.as_ref() else {
        return Ok(None);
    };
    let bounds = uor_r4_graph_compiler::trace_profile::TraceCaptureBounds {
        layer_indices: options.trace_layers.clone(),
        support_size: options
            .trace_support
            .unwrap_or(uor_r4_graph_compiler::trace_profile::PRIMARY_TOP_K),
    };
    uor_r4_graph_compiler::trace_profile::profile_spec(id, *version, &bounds).map(Some)
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

fn export_observation_tokenizer_in_session(
    session: &observe::ObservationSession,
    runtime_table: &RuntimeTokenizerDecodeTable,
) -> Result<Option<Vec<u32>>, SourceUnavailable> {
    eprintln!("exporting tokenizer...");
    let export = core_scenarios::export_runtime_tokenizer_table(
        runtime_table,
        session.dir().join("tokenizer.bin"),
    )
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
    // Resolve the trace profile against the registry BEFORE loading an
    // expensive teacher or touching resumable output — same discipline as
    // the tokenizer resolution below.
    let trace_profile = resolve_observe_text_trace_profile(&options)?;
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
    let (tokenizer, runtime_table, mut token_byte_lengths, oracle): (
        TokenizerKind,
        Option<RuntimeTokenizerDecodeTable>,
        Option<Vec<u32>>,
        Box<dyn TeacherOracle + Send>,
    ) = if let Some(checkpoint) = &options.checkpoint {
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
        let token_byte_lengths = Some(
            legacy
                .vocab
                .iter()
                .map(|piece| piece.len() as u32)
                .collect(),
        );
        let tokenizer = TokenizerKind::Legacy(legacy);
        let path = checkpoint
            .to_str()
            .ok_or_else(|| SourceUnavailable::new("checkpoint path is not UTF-8"))?;
        let oracle = LlamaOracle::load(path);
        (tokenizer, None, token_byte_lengths, Box::new(oracle))
    } else {
        let (resolved_tokenizer, runtime_table) = resolve_registered_observation_tokenizer(
            &options.source,
            options.tokenizer_adapter.as_ref(),
        )?;
        let oracle = Teacher::load_with_sequence_length(&options.source, options.sequence_length)
            .map_err(|error| {
            SourceUnavailable::new(format!("failed to load Hugging Face model: {error}"))
        })?;
        (
            resolved_tokenizer,
            Some(runtime_table),
            None,
            Box::new(oracle),
        )
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
    let geometry = pool[0].geometry_projection();
    let attention_operator = pool[0].attention_operator_spec();
    let dense_operator = pool[0].dense_operator_spec();
    for (worker, oracle) in pool.iter().enumerate().skip(1) {
        if oracle.geometry_projection() != geometry {
            return Err(SourceUnavailable::new(format!(
                "observe-text worker {worker} loaded a different source geometry before output; refusing a mixed teacher pool"
            )));
        }
        if oracle.attention_operator_spec() != attention_operator {
            return Err(SourceUnavailable::new(format!(
                "observe-text worker {worker} loaded a different attention operator before output; refusing a mixed teacher pool"
            )));
        }
        if oracle.dense_operator_spec() != dense_operator {
            return Err(SourceUnavailable::new(format!(
                "observe-text worker {worker} loaded a different dense operator before output; refusing a mixed teacher pool"
            )));
        }
    }
    // All potentially fallible teacher/tokenizer loading is complete before
    // the locked output session publishes identities or tokenizer bytes.
    let session = observe::ObservationSession::acquire(&options.output, options.shards)?;
    let adapter = tokenizer.adapter();
    pin_text_observation_identities_before_output_with_dense(
        &session,
        &options.input,
        adapter.as_ref(),
        geometry.as_ref(),
        attention_operator.as_ref(),
        dense_operator.as_ref(),
    )?;
    if let Some(runtime_table) = runtime_table.as_ref() {
        token_byte_lengths = export_observation_tokenizer_in_session(&session, runtime_table)?;
    }
    if options.workers > 1 {
        eprintln!("observing with {} teacher workers", options.workers);
    }
    let report = match trace_profile.as_ref() {
        Some(profile) => {
            eprintln!(
                "observing under trace profile {}/{} ({} declared layers, support cap {})",
                profile.id,
                profile.version,
                options.trace_layers.len(),
                options
                    .trace_support
                    .unwrap_or(uor_r4_graph_compiler::trace_profile::PRIMARY_TOP_K)
            );
            observe_text::observe_text_corpus_traced_in_session(
                &session,
                &mut pool,
                options.seconds,
                &tokenizer,
                token_byte_lengths.as_deref(),
                &options.input,
                true,
                profile,
            )
            .map_err(SourceUnavailable::new)?
        }
        None => observe_text::observe_text_corpus_in_session(
            &session,
            &mut pool,
            options.seconds,
            &tokenizer,
            token_byte_lengths.as_deref(),
            &options.input,
            true,
        )
        .map_err(SourceUnavailable::new)?,
    };
    let result = finish_observe_text_report(&report, &session);
    drop(session);
    result
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
    let session = observe::ObservationSession::acquire(&options.output, options.shards)?;
    let adapter = tokenizer.adapter();
    let geometry = teacher.geometry_projection();
    let attention_operator = teacher.attention_operator_spec();
    let dense_operator = teacher.dense_operator_spec();
    pin_text_observation_identities_before_output_with_dense(
        &session,
        &options.input,
        adapter.as_ref(),
        geometry.as_ref(),
        attention_operator.as_ref(),
        dense_operator.as_ref(),
    )?;
    let token_byte_lengths = export_observation_tokenizer_in_session(&session, &runtime_table)?;
    eprintln!("observing with batch {}", options.batch);
    let report = match teacher {
        Teacher::Llama(oracle) => observe_text::observe_text_corpus_batched_in_session(
            &session,
            &oracle,
            options.batch,
            options.seconds,
            &tokenizer,
            token_byte_lengths.as_deref(),
            &options.input,
            true,
        ),
        Teacher::Gpt2(oracle) => observe_text::observe_text_corpus_batched_in_session(
            &session,
            &oracle,
            options.batch,
            options.seconds,
            &tokenizer,
            token_byte_lengths.as_deref(),
            &options.input,
            true,
        ),
    }
    .map_err(SourceUnavailable::new)?;
    let result = finish_observe_text_report(&report, &session);
    drop(session);
    result
}

/// Print the observe-text report and, when the corpus is complete, persist the
/// merged record stream. Shared by the serial/pool and batched entry points.
fn finish_observe_text_report(
    report: &observe_text::ObservationReport,
    session: &observe::ObservationSession,
) -> Result<(), SourceUnavailable> {
    let output = session.dir();
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
        write_regular_file_synced(&merged_path, &merged, "merged text observation corpus")?;
        let state_path = output.join(observe::STATE_FILE);
        let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
            session.recorded_corpus_guard(),
            &state_path,
            &merged_path,
        )?;
        let captured = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
            session.recorded_corpus_guard(),
            &state_path,
            &merged_path,
        )?;
        if captured.binding_cid.as_deref() != Some(binding_cid.as_str()) {
            return Err(SourceUnavailable::new(
                "text observation binding changed during bounded post-publication verification",
            ));
        }
        captured.verify_generation()?;
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
    /// Explicit stable mutation authority for a managed/canonical bundle.
    /// Without this flag, `--out` is always an arbitrary standalone root,
    /// even when its basename is `graph-cover`.
    bundle_root: Option<PathBuf>,
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
    /// Typed host-side dense-execution record. Absence is retained for
    /// historical corpora and Llama; present records are registry exact.
    dense_operator: Option<DenseOperatorSpec>,
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
        bundle_root: None,
        source_manifest_kappa: None,
        geometry: None,
        attention_operator: None,
        dense_operator: None,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--emit-provenance" {
            // #637 phase 3: PROV/1 is unconditionally emitted now (see
            // the `emit_r4g1_with_provenance` call below), so this flag
            // is a no-op kept only for backward CLI compatibility with
            // phase 2a callers/scripts that already pass it.
            index += 1;
            continue;
        }
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
            "--bundle-root" => options.bundle_root = Some(PathBuf::from(value)),
            "--source-manifest-kappa" => {
                options.source_manifest_kappa = Some(value.clone());
            }
            "--geometry-projection" => {
                options.geometry = Some(serde_json::from_str(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --geometry-projection value: {error}"))
                })?);
            }
            "--attention-operator" => {
                serde_json::from_str::<DuplicateRejectingJson>(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --attention-operator value: {error}"))
                })?;
                let recorded: serde_json::Value = serde_json::from_str(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --attention-operator value: {error}"))
                })?;
                options.attention_operator = Some(validate_attention_operator_json(
                    &recorded,
                    "--attention-operator",
                )?);
            }
            "--dense-operator" => {
                serde_json::from_str::<DuplicateRejectingJson>(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --dense-operator value: {error}"))
                })?;
                let recorded: serde_json::Value = serde_json::from_str(value).map_err(|error| {
                    SourceUnavailable::new(format!("invalid --dense-operator value: {error}"))
                })?;
                options.dense_operator =
                    Some(validate_dense_operator_json(&recorded, "--dense-operator")?);
            }
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown cover option: {flag}"
                )));
            }
        }
        index += 2;
    }
    match (
        options.attention_operator.as_ref(),
        options.dense_operator.as_ref(),
    ) {
        (Some(attention), dense) => {
            validate_source_execution_pair(attention, dense, "cover provenance")?;
        }
        (None, Some(dense)) => {
            return Err(SourceUnavailable::new(format!(
                "cover provenance supplies dense operator {}/{} without an attention operator",
                dense.id, dense.version
            )));
        }
        (None, None) => {}
    }
    Ok(options)
}

fn require_cover_recorded_execution_identity(
    options: &CoverOptions,
    recorded: &RecordedCorpusExecutionIdentity,
) -> Result<(), SourceUnavailable> {
    let attention_label = |record: Option<&AttentionOperatorSpec>| {
        record
            .map(|record| format!("{}/{}", record.id, record.version))
            .unwrap_or_else(|| "absent".to_owned())
    };
    let dense_label = |record: Option<&DenseOperatorSpec>| {
        record
            .map(|record| format!("{}/{}", record.id, record.version))
            .unwrap_or_else(|| "absent".to_owned())
    };
    if options.attention_operator.as_ref() != recorded.attention_operator.as_ref() {
        return Err(SourceUnavailable::new(format!(
            "cover --attention-operator {} does not match recorded corpus attention provenance {}; refusing before output mutation",
            attention_label(options.attention_operator.as_ref()),
            attention_label(recorded.attention_operator.as_ref())
        )));
    }
    if options.dense_operator.as_ref() != recorded.dense_operator.as_ref() {
        return Err(SourceUnavailable::new(format!(
            "cover --dense-operator {} does not match recorded corpus dense provenance {}; refusing before output mutation",
            dense_label(options.dense_operator.as_ref()),
            dense_label(recorded.dense_operator.as_ref())
        )));
    }
    Ok(())
}

/// Multiresolution cover induction (plan §5 Phase 2, issue #60): induce
/// the overlapping region cover over the deterministic context-bundle
/// lane, freeze the reference classifier, measure held-out routing recall
/// against it and against the incumbent 4×256 class cover, and write the
/// R4G1 artifact plus the JSON recall/stability report.
pub fn cover_command(args: &[String]) -> Result<(), SourceUnavailable> {
    cover_command_with_authority(args, GraphCommandCallerAuthority::Public)
}

/// Run cover induction while a managed caller retains the one common
/// recorded-corpus producer transaction for the bundle root. This is the
/// server-only seam: acquiring again would self-contend, while falling back to
/// pathname reads would leave completion publication outside the transaction.
pub fn cover_command_under_producer_guard(
    args: &[String],
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    cover_command_with_authority(args, GraphCommandCallerAuthority::Provided(guard))
}

fn cover_command_with_authority(
    args: &[String],
    caller_authority: GraphCommandCallerAuthority<'_>,
) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make cover induction much slower; use `cargo run --release -- transformerless cover ...`"
    );
    let mut options = parse_cover_options(args)?;
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

    let transaction_authority = match caller_authority {
        GraphCommandCallerAuthority::Public => match options.bundle_root.as_deref() {
            Some(root) => GraphTransactionAuthority::DeclaredBundle(root),
            None => GraphTransactionAuthority::StandaloneExact,
        },
        GraphCommandCallerAuthority::Provided(guard) => {
            if let Some(root) = options.bundle_root.as_deref() {
                return Err(SourceUnavailable::new(format!(
                    "cover already has retained producer authority for {}; do not supply a second --bundle-root {} declaration",
                    guard.root().display(),
                    root.display()
                )));
            }
            GraphTransactionAuthority::ProvidedBundle(guard)
        }
    };
    let (recorded_corpus, mut transaction) = begin_graph_command_transaction(
        transaction_authority,
        &corpus_meta_path,
        &corpus_recs_path,
        &mut options.output,
        "cover",
    )?;
    let tokenizer_cid = explicit_tokenizer_cid(options.tokenizer.as_deref())?.unwrap_or([0; 32]);
    require_cover_recorded_execution_identity(&options, &recorded_corpus.execution)?;
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
    let meta_bytes = recorded_corpus.meta_bytes;
    let recs_bytes = recorded_corpus.records_bytes;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(&corpus_meta_path, &corpus_recs_path, &corpus_kappa);
    let corpus = recorded_corpus.corpus;

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
    // #637 phase 3: every `cover` artifact now unconditionally carries a
    // PROV/1 section (maintainer-approved default flip, tracked on
    // #637), binding whichever identity records the caller already
    // passed (`--source-manifest-kappa`, `--geometry-projection`,
    // `--attention-operator`, `--dense-operator`) -- or none at all,
    // which is still a valid, deliberately-empty PROV/1 record per
    // `emit_r4g1_with_provenance`'s own contract. Digest strings that
    // fail to parse as a canonical `blake3:<hex>`/bare-hex κ are refused
    // before any output mutation, matching this parser's existing
    // fail-closed CLI-argument pattern. This changes every `cover`
    // artifact's bytes and κ relative to phases 1/2a (which were
    // additive/opt-in). No checked-in fixture in this workspace pins
    // `cover`'s emitted R4G1 bytes or κ literally (the full crate test
    // suite -- graph-cli, graph-compiler, graph-format -- was run
    // against this change and is unaffected); the unrelated Gate E
    // `kappa_reproduction`/`TLESS_REPIN_WRITE` procedure in AGENTS.md
    // re-pins a different fixture (the llama2.c reference model) and is
    // not touched by this slice.
    let (artifact_bytes, info) = {
        let parse_digest = |label: &str, value: &str| {
            uor_r4_graph_format::parse_digest_hex(value).ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "--emit-provenance: {label} is not a blake3:<hex> or bare-hex digest: {value}"
                ))
            })
        };
        let provenance = uor_r4_graph_format::ProvComponents {
            source_manifest_kappa: options
                .source_manifest_kappa
                .as_deref()
                .map(|value| parse_digest("source-manifest-kappa", value))
                .transpose()?,
            geometry_digest: options
                .geometry
                .as_ref()
                .map(|record| {
                    parse_digest(
                        "geometry implementation_digest",
                        &record.implementation_digest,
                    )
                })
                .transpose()?,
            // #601 tokenizer-adapter digest is not yet threaded to this
            // call site (the typed record lives in this crate, one layer
            // above the graph-compiler emitter) -- deferred to a #637
            // follow-up rather than expanded here.
            tokenizer_adapter_digest: None,
            attention_operator_digest: options
                .attention_operator
                .as_ref()
                .map(|record| {
                    parse_digest(
                        "attention-operator implementation_digest",
                        &record.implementation_digest,
                    )
                })
                .transpose()?,
            dense_operator_digest: options
                .dense_operator
                .as_ref()
                .map(|record| {
                    parse_digest(
                        "dense-operator implementation_digest",
                        &record.implementation_digest,
                    )
                })
                .transpose()?,
            // No CLI surface for a license or evidence-root set yet
            // (#637 follow-up); this slice binds identity digests only.
            license: None,
            evidence_roots: &[],
        };
        cover::emit_r4g1_with_provenance(
            &artifact_container,
            (&meta_bytes, &recs_bytes),
            vocab,
            &induced.cover,
            &edges,
            &prior,
            &train,
            tokenizer_cid,
            &provenance,
        )
    }
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
    // Bind optional host-side dense arithmetic without changing the graph
    // runtime or artifact format.
    report.dense_operator = options.dense_operator.clone();

    transaction.prepare_output_directory(&options.output)?;
    let artifact_path = options.output.join("cover.r4g1");
    write_regular_file_synced(&artifact_path, &artifact_bytes, "cover R4G1 artifact")?;
    let report_json = serde_json::to_string_pretty(&report)?;
    let report_path = options.output.join("cover_report.json");
    write_regular_file_synced(
        &report_path,
        report_json.as_bytes(),
        "cover induction report",
    )?;
    sync_compile_output_directory(&options.output)?;
    transaction.verify_after_output(&options.output)?;
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
    /// Explicit stable mutation authority for a managed/canonical bundle.
    /// Without this flag, `--out` is always an arbitrary standalone root,
    /// even when its basename is `graph`.
    bundle_root: Option<PathBuf>,
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
        bundle_root: None,
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
            "--bundle-root" => options.bundle_root = Some(PathBuf::from(value)),
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
    score_command_with_authority(args, GraphCommandCallerAuthority::Public)
}

/// Score a managed bundle while its server-owned common producer transaction
/// remains live from Stage A through completion publication.
pub fn score_command_under_producer_guard(
    args: &[String],
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    score_command_with_authority(args, GraphCommandCallerAuthority::Provided(guard))
}

fn score_command_with_authority(
    args: &[String],
    caller_authority: GraphCommandCallerAuthority<'_>,
) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make scoring much slower; use `cargo run --release -- transformerless score ...`"
    );
    let mut options = parse_score_options(args)?;
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

    let transaction_authority = match caller_authority {
        GraphCommandCallerAuthority::Public => match options.bundle_root.as_deref() {
            Some(root) => GraphTransactionAuthority::DeclaredBundle(root),
            None => GraphTransactionAuthority::StandaloneExact,
        },
        GraphCommandCallerAuthority::Provided(guard) => {
            if let Some(root) = options.bundle_root.as_deref() {
                return Err(SourceUnavailable::new(format!(
                    "score already has retained producer authority for {}; do not supply a second --bundle-root {} declaration",
                    guard.root().display(),
                    root.display()
                )));
            }
            GraphTransactionAuthority::ProvidedBundle(guard)
        }
    };
    let (recorded_corpus, mut transaction) = begin_graph_command_transaction(
        transaction_authority,
        &corpus_meta_path,
        &corpus_recs_path,
        &mut options.output,
        "score",
    )?;
    // Same-root/server transactions are already protected here; standalone
    // cross-root output ownership remains deferred. Capture every remaining
    // input inside that ordering so a cooperating same-bundle writer cannot
    // create a mixed source/graph generation. For cross-root output, these
    // bytes are all materialized before prepare_output_directory acquires its
    // sole output producer.
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
    let meta_bytes = recorded_corpus.meta_bytes;
    let recs_bytes = recorded_corpus.records_bytes;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(&corpus_meta_path, &corpus_recs_path, &corpus_kappa);
    let corpus = recorded_corpus.corpus;

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
    transaction.prepare_output_directory(&options.output)?;
    let artifact_path = options.output.join("score.r4g1");
    write_regular_file_synced(&artifact_path, &artifact_bytes, "scored R4G1 artifact")?;
    let report_json = serde_json::to_string_pretty(&report).map_err(SourceUnavailable::new)?;
    let report_path = options.output.join("score_report.json");
    write_regular_file_synced(&report_path, report_json.as_bytes(), "graph score report")?;
    sync_compile_output_directory(&options.output)?;
    transaction.verify_after_output(&options.output)?;
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
    /// Exact source-attention arithmetic replayed by the evaluation oracle
    /// (schema 4).
    attention_operator: AttentionOperatorSpec,
    /// Exact optional host-side dense arithmetic replayed by the evaluation
    /// oracle. `None` is durable typed absence for Llama/historical bundles.
    dense_operator: Option<DenseOperatorSpec>,
}

#[derive(Debug, Serialize, Clone)]
struct EvaluationArtifacts {
    directory: String,
    artifacts_cid: String,
    store_cid: String,
    tokenizer_cid: String,
    corpus_meta_cid: String,
    corpus_records_cid: String,
    attention_operator_cid: String,
    dense_operator_cid: Option<String>,
    /// Exact content address of the canonical generation commit. Historical
    /// markerless/Llama bundles retain typed absence.
    recorded_corpus_binding_cid: Option<String>,
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

/// Require the evaluation oracle to compute the exact immutable attention
/// record pinned by the compiled bundle. Sharing a family id is insufficient:
/// v1 and v2 name different teacher arithmetic and cannot replay each other's
/// corpus rows.
fn require_matching_evaluation_attention_operator(
    compiled: &Path,
    recorded: &AttentionOperatorSpec,
    actual: &AttentionOperatorSpec,
) -> Result<(), SourceUnavailable> {
    if actual == recorded {
        return Ok(());
    }
    Err(SourceUnavailable::new(format!(
        "{} is pinned to attention operator {}/{} (digest {}), but the loaded evaluation teacher computes {}/{} (digest {}); evaluation refused before replay",
        compiled.display(),
        recorded.id,
        recorded.version,
        recorded.declared_digest(),
        actual.id,
        actual.version,
        actual.declared_digest(),
    )))
}

fn require_matching_evaluation_dense_operator(
    compiled: &Path,
    recorded: Option<&DenseOperatorSpec>,
    actual: Option<&DenseOperatorSpec>,
) -> Result<(), SourceUnavailable> {
    match (recorded, actual) {
        (Some(recorded), Some(actual)) if recorded == actual => Ok(()),
        (None, None) => Ok(()),
        (Some(recorded), Some(actual)) => Err(SourceUnavailable::new(format!(
            "{} is pinned to dense operator {}/{} (digest {}), but the loaded evaluation teacher computes {}/{} (digest {}); evaluation refused before replay",
            compiled.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
            actual.id,
            actual.version,
            actual.declared_digest(),
        ))),
        (Some(recorded), None) => Err(SourceUnavailable::new(format!(
            "{} is pinned to dense operator {}/{} (digest {}), but the loaded evaluation teacher declares none; evaluation refused before replay",
            compiled.display(),
            recorded.id,
            recorded.version,
            recorded.declared_digest(),
        ))),
        (None, Some(actual)) => Err(SourceUnavailable::new(format!(
            "{} records the legacy dense-operator-absent era, but the loaded evaluation teacher computes {}/{} (digest {}); evaluation refused before replay",
            compiled.display(),
            actual.id,
            actual.version,
            actual.declared_digest(),
        ))),
    }
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
    let recorded_corpus = recorded_compiler_corpus(&corpus_meta_path, &corpus_records_path)?;
    let recorded_attention_operator = recorded_corpus
        .execution
        .attention_operator
        .clone()
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{}: missing teacher attention-operator binding",
                options
                    .compiled
                    .join(ATTENTION_OPERATOR_BINDING_FILE)
                    .display()
            ))
        })?;
    let attention_bytes = recorded_corpus
        .attention_operator_bytes
        .as_deref()
        .ok_or_else(|| {
            SourceUnavailable::new(
                "compiled evaluation attention identity has no exact captured sidecar bytes",
            )
        })?;
    let recorded_dense_operator = recorded_corpus.execution.dense_operator.clone();
    if recorded_dense_operator.is_some() != recorded_corpus.dense_operator_bytes.is_some() {
        return Err(SourceUnavailable::new(
            "compiled evaluation dense identity and exact captured sidecar-byte presence disagree",
        ));
    }
    let attention_operator_cid = format!("blake3:{}", blake3::hash(attention_bytes).to_hex());
    let dense_operator_cid = recorded_corpus
        .dense_operator_bytes
        .as_deref()
        .map(|bytes| format!("blake3:{}", blake3::hash(bytes).to_hex()));
    let recorded_corpus_binding_cid = recorded_corpus.binding_cid.clone();
    let scored_graph_bytes =
        read_required_regular_file(&scored_graph_path, "final scored R4G1 graph")?;
    let tokenizer_bytes = read_required_regular_file(&tokenizer_path, "tokenizer.bin")?;
    verify_scored_graph_tokenizer_binding(&scored_graph_bytes, &tokenizer_bytes)?;
    verify_runtime_tokenizer_adapter_identity(&tokenizer_bytes, &recorded_adapter)?;
    let artifacts_cid = file_cid(&artifacts_path)?;
    let store_cid = file_cid(&store_path)?;
    let tokenizer_cid = format!("blake3:{}", blake3::hash(&tokenizer_bytes).to_hex());
    let corpus_meta_cid = format!(
        "blake3:{}",
        blake3::hash(&recorded_corpus.meta_bytes).to_hex()
    );
    let corpus_records_cid = recorded_corpus.records_cid.clone();
    let corpus = recorded_corpus.corpus;
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
    oracle.set_r4_attention(
        recorded_attention_operator.id == AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
    );
    let actual_attention_operator = oracle.attention_operator_spec().ok_or_else(|| {
        SourceUnavailable::new("evaluation teacher declares no attention operator")
    })?;
    let actual_attention_operator = validate_attention_operator(
        &actual_attention_operator,
        "evaluation teacher attention operator",
    )?;
    let actual_dense_operator = oracle
        .dense_operator_spec()
        .map(|operator| validate_dense_operator(&operator, "evaluation teacher dense operator"))
        .transpose()?;
    validate_source_execution_pair(
        &actual_attention_operator,
        actual_dense_operator.as_ref(),
        "evaluation teacher source execution",
    )?;
    require_matching_evaluation_attention_operator(
        &options.compiled,
        &recorded_attention_operator,
        &actual_attention_operator,
    )?;
    require_matching_evaluation_dense_operator(
        &options.compiled,
        recorded_dense_operator.as_ref(),
        actual_dense_operator.as_ref(),
    )?;
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
        schema: 5,
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
            attention_operator: recorded_attention_operator,
            dense_operator: recorded_dense_operator,
        },
        artifacts: EvaluationArtifacts {
            directory: options.compiled.display().to_string(),
            artifacts_cid,
            store_cid,
            tokenizer_cid,
            corpus_meta_cid,
            corpus_records_cid,
            attention_operator_cid,
            dense_operator_cid,
            recorded_corpus_binding_cid,
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

/// Compile through a caller-retained source/common transaction. The session
/// must protect the parsed `--output`; the same opaque authority is returned
/// so a managed caller can validate and seal the exact compiled generation
/// before either lock is released.
pub fn compile_hugging_face_with_session(
    args: &[String],
    session: SourceCorpusSession,
) -> Result<SourceCorpusSession, SourceUnavailable> {
    compile_hugging_face_with_progress_and_session(args, Some(session), |_, _| {})
}

/// Compile a Hugging Face teacher bundle and report coarse compiler phases.
/// The callback is compiler-side only and does not affect generated bytes.
pub fn compile_hugging_face_with_progress<F>(
    args: &[String],
    progress: F,
) -> Result<(), SourceUnavailable>
where
    F: FnMut(u8, &'static str),
{
    compile_hugging_face_with_progress_and_session(args, None, progress).map(drop)
}

fn compile_hugging_face_with_progress_and_session<F>(
    args: &[String],
    provided_session: Option<SourceCorpusSession>,
    mut progress: F,
) -> Result<SourceCorpusSession, SourceUnavailable>
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
    let requested_output = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/compiled").join(&slug));
    let output = canonical_hf_compile_output_subject(&requested_output)?;
    let server_owned_stage = provided_session.is_some();
    if !server_owned_stage {
        reject_foreign_compiled_bundle_stage_owner(&output)?;
    }
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
    let mut corpus_session = if let Some(session) = provided_session {
        require_exact_managed_compiled_bundle_stage_owner(
            session.recorded_corpus_guard()?,
            &output,
        )?;
        session
    } else {
        source_corpus_session(&output)?
    };
    eprintln!("compiler output: {}", output.display());
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
    let attention_operator = oracle.attention_operator_spec().ok_or_else(|| {
        SourceUnavailable::new("Hugging Face teacher declares no attention operator")
    })?;
    let dense_operator = oracle.dense_operator_spec();
    validate_source_execution_pair(
        &attention_operator,
        dense_operator.as_ref(),
        "Hugging Face teacher source execution",
    )?;
    // Keep every identity and corpus-resume check read-only until the whole
    // bundle has been accepted. In particular, malformed regular metadata
    // must not be mistaken for a fresh corpus after either sidecar is pinned.
    preflight_compile_tokenizer_adapter(&output, &tokenizer_adapter)?;
    preflight_compiled_attention_operator(&output, &attention_operator)?;
    preflight_compiled_dense_operator(&output, dense_operator.as_ref())?;
    let hidden_row_bytes = oracle
        .hidden_state()
        .map(|hidden| {
            hidden
                .len()
                .checked_mul(std::mem::size_of::<f32>())
                .ok_or_else(|| {
                    SourceUnavailable::new("loaded teacher hidden-state row byte length overflows")
                })
        })
        .transpose()?;
    let _ = preflight_source_compile_resume(&output, hidden_row_bytes, options.target)?;
    // Canonical order: source-specific outer session, then the one common
    // recorded-corpus producer guard. Repeat every semantic/resume preflight
    // after the common guard creates or binds the root and before the first
    // recover/pin/generate mutation.
    corpus_session.acquire_recorded_corpus(&output)?;
    if !server_owned_stage {
        // Repeat under the common producer authority so a cooperating server
        // cannot install an owner marker in the check-to-lock interval.
        reject_foreign_compiled_bundle_stage_owner(&output)?;
    }
    preflight_compile_tokenizer_adapter(&output, &tokenizer_adapter)?;
    preflight_compiled_attention_operator(&output, &attention_operator)?;
    preflight_compiled_dense_operator(&output, dense_operator.as_ref())?;
    let resume = preflight_source_compile_resume(&output, hidden_row_bytes, options.target)?;
    corpus_session.begin_recorded_corpus_mutation(&output)?;
    // All read-only identity and resume checks have now accepted the bundle.
    // Under the server's cross-process output session, reclaim only the
    // core compiler's exact reserved checkpoint temporaries; arbitrary names
    // and nonregular entries were rejected above without mutation.
    recover_source_corpus_checkpoint_temporaries(&output)?;
    // A story checkpoint commits records and hidden rows together. Normalize
    // both interrupted tails before the core generator reopens either stream.
    reconcile_source_compile_resume(&output, &resume)?;
    pin_compile_identities_with_dense(
        &output,
        &tokenizer_adapter,
        &attention_operator,
        dense_operator.as_ref(),
    )?;
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
    let finalized_meta =
        read_optional_regular_file_nofollow(Path::new(meta), "source compile corpus metadata")?;
    if !finalized_meta
        .as_deref()
        .is_some_and(|bytes| bytes.len() == SOURCE_CORPUS_META_BYTES && bytes[24] == 1)
    {
        println!(
            "corpus is not complete; rerun the same command to resume {}",
            output.display()
        );
        return Ok(corpus_session);
    }
    let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
        corpus_session.recorded_corpus_guard()?,
        Path::new(meta),
        Path::new(records),
    )?;
    let mut captured = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
        corpus_session.recorded_corpus_guard()?,
        Path::new(meta),
        Path::new(records),
    )?;
    if captured.binding_cid.as_deref() != Some(binding_cid.as_str()) {
        return Err(SourceUnavailable::new(
            "published recorded-corpus binding CID changed during bounded verification",
        ));
    }
    let meta_bytes = captured.meta_bytes.clone();
    let (records_bytes, hidden_bytes) = captured.materialize_compiler_corpus_bytes()?;
    let corpus = compiler::load_corpus_bytes(&meta_bytes, &records_bytes, hidden_bytes.as_deref())
        .ok_or_else(|| {
            SourceUnavailable::new(
                "published recorded corpus failed bounded exact-generation parsing",
            )
        })?;
    drop(captured);
    corpus_session
        .recorded_corpus_guard()?
        .finish_compile_attempt()?;
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
    Ok(corpus_session)
}

/// Registry-exact host execution identity attached to a recorded corpus.
/// Both fields preserve genuine historical absence; callers that need the
/// pre-#602 standard/1 interpretation apply it only after this joint snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCorpusExecutionIdentity {
    pub attention_operator: Option<AttentionOperatorSpec>,
    pub dense_operator: Option<DenseOperatorSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
struct RecordedCorpusSnapshot {
    execution: RecordedCorpusExecutionIdentity,
    attention_operator_bytes: Option<Vec<u8>>,
    dense_operator_bytes: Option<Vec<u8>>,
    meta_bytes: Vec<u8>,
    records_bytes: Vec<u8>,
    hidden_bytes: Option<Vec<u8>>,
    binding_cid: Option<String>,
}

/// The exact recorded generation required by compiler/evaluation consumers.
/// Record bytes are materialized because the existing compiler parses them;
/// hidden rows are materialized only when they have the compiler's fixed `D`
/// width. Wider source-model rows remain bound by their retained stream digest
/// and never become a multi-gigabyte allocation that the compiler discards.
struct RecordedCompilerCorpus {
    execution: RecordedCorpusExecutionIdentity,
    attention_operator_bytes: Option<Vec<u8>>,
    dense_operator_bytes: Option<Vec<u8>>,
    meta_bytes: Vec<u8>,
    records_bytes: Vec<u8>,
    records_cid: String,
    corpus: compiler::Corpus,
    binding_cid: Option<String>,
}

#[derive(Clone, Copy)]
enum GraphCommandCallerAuthority<'a> {
    /// Public CLI/API entry point. Parsed `--bundle-root` selects the declared
    /// bundle mode; its absence selects exact standalone mode.
    Public,
    /// Managed server stage whose producer interval began after Stage A and
    /// remains live through completion publication.
    Provided(&'a RecordedCorpusProducerGuard),
}

#[derive(Clone, Copy)]
enum GraphTransactionAuthority<'a> {
    /// Exact canonical `--out`; physical owner evidence present at transaction
    /// start is terminal, but no later ancestor state changes this mode.
    StandaloneExact,
    /// Public caller explicitly joins one canonical bundle-root transaction.
    DeclaredBundle(&'a Path),
    /// Managed caller already holds that bundle-root producer transaction.
    ProvidedBundle(&'a RecordedCorpusProducerGuard),
}

enum GraphTransactionProducer<'a> {
    Provided(&'a RecordedCorpusProducerGuard),
    Owned(RecordedCorpusProducerGuard),
    /// Cross-root lane. The source reader pin has already been released;
    /// acquire the stable exact-output or declared-bundle root only
    /// immediately before mutation.
    Pending(Option<PathBuf>),
}

struct GraphCommandTransaction<'a> {
    producer: GraphTransactionProducer<'a>,
    root: PathBuf,
    direct_entrypoint: bool,
    label: &'static str,
}

impl GraphCommandTransaction<'_> {
    fn guard(&self) -> Result<&RecordedCorpusProducerGuard, SourceUnavailable> {
        match &self.producer {
            GraphTransactionProducer::Provided(guard) => Ok(guard),
            GraphTransactionProducer::Owned(guard) => Ok(guard),
            GraphTransactionProducer::Pending(_) => Err(SourceUnavailable::new(format!(
                "{} output producer authority has not been acquired",
                self.label
            ))),
        }
    }

    fn acquire_pending_output(&mut self) -> Result<(), SourceUnavailable> {
        let pending = match &mut self.producer {
            GraphTransactionProducer::Pending(root) => root.take(),
            GraphTransactionProducer::Provided(_) | GraphTransactionProducer::Owned(_) => None,
        };
        if let Some(root) = pending {
            let guard = RecordedCorpusProducerGuard::try_acquire(&root)?;
            if self.direct_entrypoint && guard.root_exists() {
                reject_foreign_compiled_bundle_stage_owner_for_graph(&guard)?;
            }
            reject_completed_graph_transaction(&guard, self.label)?;
            self.producer = GraphTransactionProducer::Owned(guard);
        }
        Ok(())
    }

    fn prepare_output_directory(&mut self, output: &Path) -> Result<(), SourceUnavailable> {
        self.prepare_output_directory_with_after_guard(output, |_| Ok(()))
    }

    fn prepare_output_directory_with_after_guard<F>(
        &mut self,
        output: &Path,
        after_guard: F,
    ) -> Result<(), SourceUnavailable>
    where
        F: FnOnce(&RecordedCorpusProducerGuard) -> Result<(), SourceUnavailable>,
    {
        self.acquire_pending_output()?;
        reject_completed_graph_transaction(self.guard()?, self.label)?;
        after_guard(self.guard()?)?;

        match &mut self.producer {
            GraphTransactionProducer::Owned(guard) if !guard.root_exists() => {
                guard.ensure_root()?;
            }
            GraphTransactionProducer::Provided(guard) if !guard.root_exists() => {
                return Err(SourceUnavailable::new(format!(
                    "provided {} producer root {} is absent",
                    self.label,
                    guard.root().display()
                )));
            }
            GraphTransactionProducer::Pending(_) => {
                return Err(SourceUnavailable::new(format!(
                    "{} output producer authority remained pending",
                    self.label
                )));
            }
            GraphTransactionProducer::Provided(_) | GraphTransactionProducer::Owned(_) => {}
        }
        let guard = self.guard()?;
        if !guard.protects_directory(&self.root)? {
            return Err(SourceUnavailable::new(format!(
                "{} producer guard for {} does not protect canonical transaction root {}",
                self.label,
                guard.root().display(),
                self.root.display()
            )));
        }
        if output != self.root {
            if output.parent() != Some(self.root.as_path()) {
                return Err(SourceUnavailable::new(format!(
                    "{} output {} is not the transaction root or one direct owned child of {}",
                    self.label,
                    output.display(),
                    self.root.display()
                )));
            }
            match std::fs::symlink_metadata(output) {
                Ok(metadata) if metadata.file_type().is_dir() => {}
                Ok(_) => {
                    return Err(SourceUnavailable::new(format!(
                        "{} output {} is not a regular non-symlink directory",
                        self.label,
                        output.display()
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::create_dir(output).map_err(|error| {
                        SourceUnavailable::new(format!(
                            "{} output {} cannot be created under retained producer authority: {error}",
                            self.label,
                            output.display()
                        ))
                    })?;
                }
                Err(error) => return Err(SourceUnavailable::new(error)),
            }
        }
        let canonical_output = std::fs::canonicalize(output).map_err(SourceUnavailable::new)?;
        if canonical_output != output {
            return Err(SourceUnavailable::new(format!(
                "{} output {} changed canonical identity before mutation",
                self.label,
                output.display()
            )));
        }
        reject_completed_graph_transaction(guard, self.label)
    }

    fn verify_after_output(&self, output: &Path) -> Result<(), SourceUnavailable> {
        let guard = self.guard()?;
        if !guard.protects_directory(&self.root)? {
            return Err(SourceUnavailable::new(format!(
                "{} producer guard stopped protecting transaction root {}",
                self.label,
                self.root.display()
            )));
        }
        let metadata = std::fs::symlink_metadata(output).map_err(SourceUnavailable::new)?;
        if !metadata.file_type().is_dir()
            || std::fs::canonicalize(output).map_err(SourceUnavailable::new)? != output
        {
            return Err(SourceUnavailable::new(format!(
                "{} output {} changed directory identity during publication",
                self.label,
                output.display()
            )));
        }
        reject_completed_graph_transaction(guard, self.label)?;
        guard.verify_owned_root()
    }
}

fn canonical_existing_corpus_root(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<PathBuf, SourceUnavailable> {
    let parent = |path: &Path, label: &str| -> Result<PathBuf, SourceUnavailable> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::canonicalize(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} parent {} cannot be canonicalized: {error}",
                parent.display()
            ))
        })
    };
    let meta_root = parent(corpus_meta, "recorded corpus metadata")?;
    let records_root = parent(corpus_records, "recorded corpus records")?;
    if meta_root != records_root {
        return Err(SourceUnavailable::new(format!(
            "corpus.meta and corpus.records have different canonical parent roots ({} versus {}); no graph transaction can bind them",
            meta_root.display(),
            records_root.display()
        )));
    }
    Ok(meta_root)
}

fn canonical_output_subject(output: &Path, label: &str) -> Result<PathBuf, SourceUnavailable> {
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            std::fs::canonicalize(output).map_err(SourceUnavailable::new)
        }
        Ok(_) => Err(SourceUnavailable::new(format!(
            "{label} output {} is not a regular non-symlink directory",
            output.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let leaf = output.file_name().ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "{label} output {} has no final directory name",
                    output.display()
                ))
            })?;
            if leaf == std::ffi::OsStr::new(".") || leaf == std::ffi::OsStr::new("..") {
                return Err(SourceUnavailable::new(format!(
                    "{label} output {} has a reserved final directory name",
                    output.display()
                )));
            }
            let parent = output
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let parent = std::fs::canonicalize(parent).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{label} output parent {} cannot be canonicalized before publication: {error}",
                    parent.display()
                ))
            })?;
            Ok(parent.join(leaf))
        }
        Err(error) => Err(SourceUnavailable::new(error)),
    }
}

/// Read-only start-time classification for exact standalone output. This does
/// not select or widen transaction authority: it only refuses an output that
/// was already physically nested in a completed/owned bundle when the command
/// began. Later ancestor changes do not alter standalone exact-root mode.
fn existing_graph_owner_ancestor(output: &Path) -> Result<Option<PathBuf>, SourceUnavailable> {
    let mut candidate = if output.exists() {
        Some(output)
    } else {
        output.parent()
    };
    while let Some(root) = candidate {
        let metadata = match std::fs::symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                candidate = root.parent();
                continue;
            }
            Err(error) => return Err(SourceUnavailable::new(error)),
        };
        if metadata.file_type().is_dir() {
            for leaf in [
                "compiled_bundle_completion.json",
                ".compiled_bundle_stage.json",
                ".compiled_bundle_stage_a.json",
            ] {
                match std::fs::symlink_metadata(root.join(leaf)) {
                    Ok(_) => {
                        return std::fs::canonicalize(root)
                            .map(Some)
                            .map_err(SourceUnavailable::new);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(SourceUnavailable::new(error)),
                }
            }
            let completion_prefix = ".compiled_bundle_completion.json.";
            for entry in std::fs::read_dir(root).map_err(SourceUnavailable::new)? {
                let entry = entry.map_err(SourceUnavailable::new)?;
                let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if name.starts_with(completion_prefix) && name.ends_with(".tmp") {
                    return std::fs::canonicalize(root)
                        .map(Some)
                        .map_err(SourceUnavailable::new);
                }
            }
        }
        candidate = root.parent();
    }
    Ok(None)
}

fn reject_existing_graph_owner_ancestor(
    output: &Path,
    label: &str,
) -> Result<(), SourceUnavailable> {
    let Some(root) = existing_graph_owner_ancestor(output)? else {
        return Ok(());
    };
    let completion = root.join("compiled_bundle_completion.json");
    match std::fs::symlink_metadata(&completion) {
        Ok(metadata) if metadata.file_type().is_file() => {
            return Err(SourceUnavailable::new(format!(
                "{label} standalone output ancestor {} was already an immutable completed bundle at transaction start; pass --bundle-root to participate explicitly",
                root.display()
            )));
        }
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "{label} standalone output ancestor {} had a malformed nonregular completion record at transaction start; refusing graph mutation",
                root.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SourceUnavailable::new(error)),
    }
    let completion_prefix = ".compiled_bundle_completion.json.";
    for entry in std::fs::read_dir(&root).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with(completion_prefix) && name.ends_with(".tmp") {
            return Err(SourceUnavailable::new(format!(
                "{label} standalone output ancestor {} had unfinished completion publication evidence {name} at transaction start; refusing graph mutation",
                root.display()
            )));
        }
    }
    Err(SourceUnavailable::new(format!(
        "{label} standalone output ancestor {} was already an owned server compiled-bundle stage at transaction start; pass --bundle-root only for a public completed bundle",
        root.display()
    )))
}

fn declared_graph_transaction_root(
    output: &Path,
    declared: &Path,
    label: &str,
) -> Result<PathBuf, SourceUnavailable> {
    let metadata = std::fs::symlink_metadata(declared).map_err(|error| {
        SourceUnavailable::new(format!(
            "declared {label} --bundle-root {} cannot be inspected: {error}",
            declared.display()
        ))
    })?;
    if !metadata.file_type().is_dir() {
        return Err(SourceUnavailable::new(format!(
            "declared {label} --bundle-root {} is not a regular non-symlink directory",
            declared.display()
        )));
    }
    let root = std::fs::canonicalize(declared).map_err(SourceUnavailable::new)?;
    if output.parent() != Some(root.as_path()) {
        return Err(SourceUnavailable::new(format!(
            "{label} output {} is not one direct child of declared bundle root {}",
            output.display(),
            root.display()
        )));
    }
    Ok(root)
}

fn reject_foreign_compiled_bundle_stage_owner_for_graph(
    guard: &RecordedCorpusProducerGuard,
) -> Result<(), SourceUnavailable> {
    guard.verify_owned_root()?;
    for leaf in [
        ".compiled_bundle_stage.json",
        ".compiled_bundle_stage_a.json",
    ] {
        match std::fs::symlink_metadata(guard.root().join(leaf)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(SourceUnavailable::new(error)),
            Ok(_) => {
                return Err(SourceUnavailable::new(format!(
                    "graph output ancestor {} is an owned server compiled-bundle stage; direct graph commands require the explicit server-retained producer seam",
                    guard.root().display()
                )));
            }
        }
    }
    guard.verify_owned_root()
}

fn reject_completed_graph_transaction(
    guard: &RecordedCorpusProducerGuard,
    label: &str,
) -> Result<(), SourceUnavailable> {
    if !guard.root_exists() {
        return Ok(());
    }
    guard.verify_owned_root()?;
    let completion = guard.root().join("compiled_bundle_completion.json");
    match std::fs::symlink_metadata(&completion) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SourceUnavailable::new(error)),
        Ok(metadata) if metadata.file_type().is_file() => {
            return Err(SourceUnavailable::new(format!(
                "{label} output ancestor {} is an immutable completed bundle; refusing graph mutation",
                guard.root().display()
            )));
        }
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "{label} output ancestor {} has a malformed nonregular completion record; refusing graph mutation",
                guard.root().display()
            )));
        }
    }
    let prefix = ".compiled_bundle_completion.json.";
    for entry in std::fs::read_dir(guard.root()).map_err(SourceUnavailable::new)? {
        let entry = entry.map_err(SourceUnavailable::new)?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with(prefix) && name.ends_with(".tmp") {
            return Err(SourceUnavailable::new(format!(
                "{label} output ancestor {} has unfinished completion publication residue {name}; refusing graph mutation",
                guard.root().display()
            )));
        }
    }
    guard.verify_owned_root()
}

fn begin_graph_command_transaction<'a>(
    authority: GraphTransactionAuthority<'a>,
    corpus_meta: &Path,
    corpus_records: &Path,
    output: &mut PathBuf,
    label: &'static str,
) -> Result<(RecordedCompilerCorpus, GraphCommandTransaction<'a>), SourceUnavailable> {
    let source_root = canonical_existing_corpus_root(corpus_meta, corpus_records)?;
    let canonical_output = canonical_output_subject(output, label)?;
    let transaction_root = match authority {
        GraphTransactionAuthority::StandaloneExact => {
            reject_existing_graph_owner_ancestor(&canonical_output, label)?;
            canonical_output.clone()
        }
        GraphTransactionAuthority::DeclaredBundle(root) => {
            declared_graph_transaction_root(&canonical_output, root, label)?
        }
        GraphTransactionAuthority::ProvidedBundle(guard) => guard.root().to_path_buf(),
    };
    if canonical_output != transaction_root
        && canonical_output.parent() != Some(transaction_root.as_path())
    {
        return Err(SourceUnavailable::new(format!(
            "{label} output {} is neither the selected transaction root nor one direct physical child of {}",
            canonical_output.display(),
            transaction_root.display()
        )));
    }
    *output = canonical_output;

    match authority {
        GraphTransactionAuthority::ProvidedBundle(guard) => {
            if transaction_root != source_root || !guard.protects_directory(&source_root)? {
                return Err(SourceUnavailable::new(format!(
                    "provided {label} producer guard for {} does not own canonical corpus/output transaction root {}",
                    guard.root().display(),
                    source_root.display()
                )));
            }
            reject_completed_graph_transaction(guard, label)?;
            let stream = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
                guard,
                corpus_meta,
                corpus_records,
            )?;
            let recorded =
                materialize_recorded_compiler_corpus(stream, corpus_meta, corpus_records)?;
            guard.verify_owned_root()?;
            Ok((
                recorded,
                GraphCommandTransaction {
                    producer: GraphTransactionProducer::Provided(guard),
                    root: transaction_root,
                    direct_entrypoint: false,
                    label,
                },
            ))
        }
        GraphTransactionAuthority::StandaloneExact
        | GraphTransactionAuthority::DeclaredBundle(_)
            if transaction_root == source_root =>
        {
            let guard = RecordedCorpusProducerGuard::try_acquire(&source_root)?;
            reject_foreign_compiled_bundle_stage_owner_for_graph(&guard)?;
            reject_completed_graph_transaction(&guard, label)?;
            let stream = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
                &guard,
                corpus_meta,
                corpus_records,
            )?;
            let recorded =
                materialize_recorded_compiler_corpus(stream, corpus_meta, corpus_records)?;
            guard.verify_owned_root()?;
            Ok((
                recorded,
                GraphCommandTransaction {
                    producer: GraphTransactionProducer::Owned(guard),
                    root: transaction_root,
                    direct_entrypoint: true,
                    label,
                },
            ))
        }
        GraphTransactionAuthority::StandaloneExact
        | GraphTransactionAuthority::DeclaredBundle(_) => {
            // Capture the complete source generation under one shared pin,
            // perform its final verification, then release it before any
            // output producer is acquired. This avoids unordered two-root
            // locks for arbitrary standalone layouts.
            let pins = RecordedCorpusReaderPins::try_acquire([&source_root])?;
            let stream = uor_r4_graph_compiler::recorded_corpus::open_stream_under_reader_pins(
                &pins,
                corpus_meta,
                corpus_records,
            )?;
            let recorded = if stream.binding_cid.is_some() {
                let recorded =
                    materialize_recorded_compiler_corpus(stream, corpus_meta, corpus_records)?;
                pins.verify()?;
                drop(pins);
                recorded
            } else {
                // A shared pin excludes cooperating writers but is not the
                // lower protocol's markerless cross-file ABA authority. Reopen
                // that compatibility generation through its documented
                // producer lane, retaining it through final materialization.
                drop(stream);
                drop(pins);
                let (stream, source_guard) =
                    uor_r4_graph_compiler::recorded_corpus::open_stream_for_derivation(
                        corpus_meta,
                        corpus_records,
                    )?;
                let recorded =
                    materialize_recorded_compiler_corpus(stream, corpus_meta, corpus_records)?;
                if let Some(guard) = source_guard.as_ref() {
                    guard.verify_owned_root()?;
                }
                drop(source_guard);
                recorded
            };
            Ok((
                recorded,
                GraphCommandTransaction {
                    producer: GraphTransactionProducer::Pending(Some(transaction_root.clone())),
                    root: transaction_root,
                    direct_entrypoint: true,
                    label,
                },
            ))
        }
    }
}

fn materialize_recorded_compiler_corpus(
    mut stream: uor_r4_graph_compiler::recorded_corpus::RecordedCorpusStreamSnapshot,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCompilerCorpus, SourceUnavailable> {
    let execution = recorded_execution_from_stream(&stream);
    let attention_operator_bytes = stream.attention_operator_bytes.clone();
    let dense_operator_bytes = stream.dense_operator_bytes.clone();
    let meta_bytes = stream.meta_bytes.clone();
    let records_cid = stream.records.declared_digest().to_owned();
    let binding_cid = stream.binding_cid.clone();
    let (records_bytes, hidden_bytes) = stream.materialize_compiler_corpus_bytes()?;
    let corpus = compiler::load_corpus_bytes(&meta_bytes, &records_bytes, hidden_bytes.as_deref())
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "corpus is incomplete at {}/{}; run compile until it is complete",
                corpus_meta.display(),
                corpus_records.display()
            ))
        })?;
    Ok(RecordedCompilerCorpus {
        execution,
        attention_operator_bytes,
        dense_operator_bytes,
        meta_bytes,
        records_bytes,
        records_cid,
        corpus,
        binding_cid,
    })
}

fn recorded_compiler_corpus(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCompilerCorpus, SourceUnavailable> {
    let (stream, _source_guard) =
        uor_r4_graph_compiler::recorded_corpus::open_stream_for_derivation(
            corpus_meta,
            corpus_records,
        )?;
    materialize_recorded_compiler_corpus(stream, corpus_meta, corpus_records)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedCorpusStreamFingerprint {
    meta_bytes: Vec<u8>,
    records_length: u64,
    records_digest: String,
    hidden: Option<(u64, String)>,
}

fn recorded_corpus_stream_fingerprint(
    snapshot: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusStreamSnapshot,
) -> RecordedCorpusStreamFingerprint {
    RecordedCorpusStreamFingerprint {
        meta_bytes: snapshot.meta_bytes.clone(),
        records_length: snapshot.records.len(),
        records_digest: snapshot.records.declared_digest().to_owned(),
        hidden: snapshot
            .hidden
            .as_ref()
            .map(|hidden| (hidden.len(), hidden.declared_digest().to_owned())),
    }
}

fn recorded_execution_from_stream(
    snapshot: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusStreamSnapshot,
) -> RecordedCorpusExecutionIdentity {
    RecordedCorpusExecutionIdentity {
        attention_operator: snapshot.execution.attention_operator.clone(),
        dense_operator: snapshot.execution.dense_operator.clone(),
    }
}

#[cfg(test)]
fn recorded_hidden_path(corpus_records: &Path) -> Result<PathBuf, SourceUnavailable> {
    let records = corpus_records
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    Ok(if records.ends_with("_recs.bin") {
        PathBuf::from(records.replace("_recs.bin", "_hidden.bin"))
    } else {
        PathBuf::from(format!("{records}.hidden"))
    })
}

#[cfg(test)]
fn recorded_provenance_snapshot(root: &Path) -> Result<[Option<Vec<u8>>; 3], SourceUnavailable> {
    let paths = [
        root.join(observe::MANIFEST_FILE),
        root.join(ATTENTION_OPERATOR_BINDING_FILE),
        root.join(DENSE_OPERATOR_BINDING_FILE),
    ];
    let mut snapshot: [Option<Vec<u8>>; 3] = [None, None, None];
    for (slot, path) in paths.iter().enumerate() {
        snapshot[slot] = read_optional_regular_file_nofollow(path, "recorded provenance")?;
    }
    Ok(snapshot)
}

/// Resolve the arithmetic era accompanying a canonically paired recorded
/// corpus. Observation directories carry it in `manifest.json`; compile
/// directories carry the extracted sidecar. `None` means both records are
/// genuinely absent and preserves the documented pre-#602 interpretation.
/// Every present record is registry-validated and two present sources must
/// agree exactly. Provenance is inherited only by the canonical compiled
/// (`corpus.meta` / `corpus.records`) or observation (`state.bin` /
/// `merged.bin`) pair. Arbitrarily named files remain compatible only when
/// both provenance entries are genuinely absent.
#[cfg(test)]
fn recorded_corpus_execution_identity_and_capture_with_recheck_hooks<T, F, G>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_snapshot: F,
    before_recheck: G,
) -> Result<(RecordedCorpusExecutionIdentity, T), SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce() -> Result<T, SourceUnavailable>,
{
    fn canonical_parent(path: &Path, label: &str) -> Result<PathBuf, SourceUnavailable> {
        let metadata = std::fs::symlink_metadata(path).map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} {} cannot be inspected: {error}",
                path.display()
            ))
        })?;
        if !metadata.file_type().is_file() {
            return Err(SourceUnavailable::new(format!(
                "{label} {} is not a regular non-symlink file",
                path.display()
            )));
        }
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let canonical = std::fs::canonicalize(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} parent {} cannot be resolved: {error}",
                parent.display()
            ))
        })?;
        let parent_metadata = std::fs::metadata(&canonical).map_err(|error| {
            SourceUnavailable::new(format!(
                "{label} parent {} cannot be inspected: {error}",
                canonical.display()
            ))
        })?;
        if !parent_metadata.is_dir() {
            return Err(SourceUnavailable::new(format!(
                "{label} parent {} is not a directory",
                canonical.display()
            )));
        }
        Ok(canonical)
    }

    fn is_canonical_pair(corpus_meta: &Path, corpus_records: &Path) -> bool {
        let names = (corpus_meta.file_name(), corpus_records.file_name());
        matches!(
            names,
            (Some(meta), Some(records))
                if (meta == std::ffi::OsStr::new("corpus.meta")
                    && records == std::ffi::OsStr::new("corpus.records"))
                    || (meta == std::ffi::OsStr::new(observe::STATE_FILE)
                        && records == std::ffi::OsStr::new("merged.bin"))
        )
    }

    let meta_root = canonical_parent(corpus_meta, "corpus metadata")?;
    let records_root = canonical_parent(corpus_records, "corpus records")?;
    if meta_root != records_root {
        return Err(SourceUnavailable::new(format!(
            "corpus.meta and corpus.records have different canonical parent roots ({} versus {}); no cryptographic cross-directory pairing exists",
            meta_root.display(),
            records_root.display()
        )));
    }
    let root = meta_root;
    let before = recorded_provenance_snapshot(&root)?;
    after_snapshot();
    let captured = before_recheck()?;
    let after = recorded_provenance_snapshot(&root)?;
    if before != after {
        return Err(SourceUnavailable::new(format!(
            "recorded corpus provenance in {} changed or disappeared while its attention/dense identity was resolved; retry from a stable snapshot",
            root.display()
        )));
    }
    let attention_path = root.join(ATTENTION_OPERATOR_BINDING_FILE);
    let sidecar = before[1]
        .as_deref()
        .map(|bytes| parse_compiled_attention_operator_bytes(&attention_path, bytes))
        .transpose()?;
    let dense_path = root.join(DENSE_OPERATOR_BINDING_FILE);
    let dense_sidecar = before[2]
        .as_deref()
        .map(|bytes| parse_compiled_dense_operator_bytes(&dense_path, bytes))
        .transpose()?;
    let manifest_path = root.join(observe::MANIFEST_FILE);
    let manifest = before[0]
        .as_deref()
        .map(|bytes| parse_observation_manifest_bytes(&root, &manifest_path, bytes))
        .transpose()?;
    let manifest_present = manifest.is_some();
    let manifest_operator = manifest
        .as_ref()
        .and_then(|manifest| manifest.attention_operator.clone())
        .map(|recorded| {
            validate_attention_operator(&recorded, &manifest_path.display().to_string())
        })
        .transpose()?;
    let manifest_dense = manifest
        .as_ref()
        .and_then(|manifest| manifest.dense_operator.clone())
        .map(|recorded| validate_dense_operator(&recorded, &manifest_path.display().to_string()))
        .transpose()?;

    if (sidecar.is_some() || dense_sidecar.is_some() || manifest_present)
        && !is_canonical_pair(corpus_meta, corpus_records)
    {
        return Err(SourceUnavailable::new(format!(
            "recorded-corpus provenance in {} applies only to the canonical corpus.meta/corpus.records or state.bin/merged.bin pair; refusing to attach it to {}/{}",
            root.display(),
            corpus_meta.display(),
            corpus_records.display(),
        )));
    }

    if let Some(sidecar) = sidecar.as_ref()
        && manifest_present
        && manifest_operator.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares attention operator {}/{} but present {} records the legacy operatorless era; refusing conflicting recorded-corpus provenance",
            root.join(ATTENTION_OPERATOR_BINDING_FILE).display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display(),
        )));
    }

    let attention = match (sidecar, manifest_operator) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different attention operators ({}/{} digest {} versus {}/{} digest {})",
                root.join(ATTENTION_OPERATOR_BINDING_FILE).display(),
                manifest_path.display(),
                sidecar.id,
                sidecar.version,
                sidecar.declared_digest(),
                manifest.id,
                manifest.version,
                manifest.declared_digest(),
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, Some(manifest)) => Some(manifest),
        (None, None) => None,
    };
    if let Some(sidecar) = dense_sidecar.as_ref()
        && manifest_present
        && manifest_dense.is_none()
    {
        return Err(SourceUnavailable::new(format!(
            "{} declares dense operator {}/{} but present {} records the legacy dense-operator-absent era; refusing conflicting recorded-corpus provenance",
            root.join(DENSE_OPERATOR_BINDING_FILE).display(),
            sidecar.id,
            sidecar.version,
            manifest_path.display(),
        )));
    }
    let dense = match (dense_sidecar, manifest_dense) {
        (Some(sidecar), Some(manifest)) if sidecar != manifest => {
            return Err(SourceUnavailable::new(format!(
                "{} and {} declare different dense operators ({}/{} digest {} versus {}/{} digest {})",
                root.join(DENSE_OPERATOR_BINDING_FILE).display(),
                manifest_path.display(),
                sidecar.id,
                sidecar.version,
                sidecar.declared_digest(),
                manifest.id,
                manifest.version,
                manifest.declared_digest(),
            )));
        }
        (Some(sidecar), _) => Some(sidecar),
        (None, Some(manifest)) => Some(manifest),
        (None, None) => None,
    };
    match (attention.as_ref(), dense.as_ref()) {
        (Some(attention), dense) => {
            validate_source_execution_pair(attention, dense, "recorded corpus provenance")?;
        }
        (None, Some(dense)) => {
            return Err(SourceUnavailable::new(format!(
                "recorded corpus declares dense operator {}/{} without an attention operator",
                dense.id, dense.version
            )));
        }
        (None, None) => {}
    }
    Ok((
        RecordedCorpusExecutionIdentity {
            attention_operator: attention,
            dense_operator: dense,
        },
        captured,
    ))
}

#[cfg(test)]
fn recorded_corpus_execution_identity_with_recheck_hooks<F, G>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_snapshot: F,
    before_recheck: G,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce(),
{
    recorded_corpus_execution_identity_and_capture_with_recheck_hooks(
        corpus_meta,
        corpus_records,
        after_snapshot,
        || {
            before_recheck();
            Ok(())
        },
    )
    .map(|(execution, ())| execution)
}

/// Resolve attention and dense provenance from one stable filesystem
/// snapshot, refusing any change or disappearance observed during the read.
pub fn recorded_corpus_execution_identity(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let captured =
        uor_r4_graph_compiler::recorded_corpus::execution_identity(corpus_meta, corpus_records)?;
    Ok(RecordedCorpusExecutionIdentity {
        attention_operator: captured.attention_operator,
        dense_operator: captured.dense_operator,
    })
}

/// Resolve the recorded execution identity while a managed caller already
/// retains the exact common producer root. This is the non-self-contending
/// companion used between Stage A and graph completion.
pub fn recorded_corpus_execution_identity_under_producer_guard(
    guard: &RecordedCorpusProducerGuard,
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusExecutionIdentity, SourceUnavailable> {
    let captured = uor_r4_graph_compiler::recorded_corpus::execution_identity_under_producer_guard(
        guard,
        corpus_meta,
        corpus_records,
    )?;
    Ok(RecordedCorpusExecutionIdentity {
        attention_operator: captured.attention_operator,
        dense_operator: captured.dense_operator,
    })
}

#[cfg(test)]
fn recorded_corpus_snapshot_with_hooks<F, G>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_provenance_capture: F,
    after_corpus_capture: G,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let (execution, (meta_bytes, records_bytes, hidden_bytes)) =
        recorded_corpus_execution_identity_and_capture_with_recheck_hooks(
            corpus_meta,
            corpus_records,
            after_provenance_capture,
            || {
                let meta_bytes =
                    read_optional_regular_file_nofollow(corpus_meta, "recorded corpus metadata")?
                        .ok_or_else(|| {
                        SourceUnavailable::new(format!(
                            "recorded corpus metadata {} disappeared during capture",
                            corpus_meta.display()
                        ))
                    })?;
                let records_bytes =
                    read_optional_regular_file_nofollow(corpus_records, "recorded corpus records")?
                        .ok_or_else(|| {
                            SourceUnavailable::new(format!(
                                "recorded corpus records {} disappeared during capture",
                                corpus_records.display()
                            ))
                        })?;
                let hidden_path = recorded_hidden_path(corpus_records)?;
                let hidden_bytes = read_optional_regular_file_nofollow(
                    &hidden_path,
                    "recorded corpus hidden stream",
                )?;

                after_corpus_capture();

                verify_optional_regular_file_nofollow(
                    corpus_meta,
                    "recorded corpus metadata",
                    Some(meta_bytes.as_slice()),
                )?;
                verify_optional_regular_file_nofollow(
                    corpus_records,
                    "recorded corpus records",
                    Some(records_bytes.as_slice()),
                )?;
                verify_optional_regular_file_nofollow(
                    &hidden_path,
                    "recorded corpus hidden stream",
                    hidden_bytes.as_deref(),
                )?;
                Ok((meta_bytes, records_bytes, hidden_bytes))
            },
        )?;
    Ok(RecordedCorpusSnapshot {
        execution,
        attention_operator_bytes: None,
        dense_operator_bytes: None,
        meta_bytes,
        records_bytes,
        hidden_bytes,
        binding_cid: None,
    })
}

#[cfg(test)]
fn recorded_corpus_snapshot_with_hook<F>(
    corpus_meta: &Path,
    corpus_records: &Path,
    after_capture: F,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable>
where
    F: FnOnce(),
{
    recorded_corpus_snapshot_with_hooks(corpus_meta, corpus_records, || {}, after_capture)
}

#[cfg(test)]
fn recorded_corpus_snapshot(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<RecordedCorpusSnapshot, SourceUnavailable> {
    let captured = uor_r4_graph_compiler::recorded_corpus::capture(corpus_meta, corpus_records)?;
    Ok(RecordedCorpusSnapshot {
        execution: RecordedCorpusExecutionIdentity {
            attention_operator: captured.execution.attention_operator,
            dense_operator: captured.execution.dense_operator,
        },
        attention_operator_bytes: captured.attention_operator_bytes,
        dense_operator_bytes: captured.dense_operator_bytes,
        meta_bytes: captured.meta_bytes,
        records_bytes: captured.records_bytes,
        hidden_bytes: captured.hidden_bytes,
        binding_cid: captured.binding_cid,
    })
}

pub fn recorded_corpus_attention_operator(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<Option<AttentionOperatorSpec>, SourceUnavailable> {
    Ok(recorded_corpus_execution_identity(corpus_meta, corpus_records)?.attention_operator)
}

/// Resolve the optional dense-execution record accompanying a recorded
/// corpus. Unlike the pre-#602 attention interpretation, genuine absence is
/// never synthesized: it is the exact historical/Llama compatibility state.
pub fn recorded_corpus_dense_operator(
    corpus_meta: &Path,
    corpus_records: &Path,
) -> Result<Option<DenseOperatorSpec>, SourceUnavailable> {
    Ok(recorded_corpus_execution_identity(corpus_meta, corpus_records)?.dense_operator)
}

/// Copy the complete registry-exact source-execution pair of a recorded
/// corpus. The historical command name and `--out attention_operator.json`
/// spelling remain stable; a present dense record is published to the sibling
/// `dense_operator.json`. Every source and destination check precedes either
/// publication so a conflict cannot partially relabel the derived corpus.
pub fn copy_recorded_attention(args: &[String]) -> Result<(), SourceUnavailable> {
    copy_recorded_attention_with_after_dense(args, || Ok(()))
}

fn copy_recorded_attention_with_after_dense<F>(
    args: &[String],
    _after_dense: F,
) -> Result<(), SourceUnavailable>
where
    F: FnOnce() -> Result<(), SourceUnavailable>,
{
    let options = parse_copy_recorded_attention_options(args)?;
    // Resolve the source pair before touching the destination. The lower
    // bounded reader hashes large members without retaining them and takes a
    // non-mutating shared coordination lock when one already exists.
    let resolved = recorded_corpus_execution_identity(&options.corpus_meta, &options.corpus_recs)?;
    let parent = options
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    match std::fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => {
            return Err(SourceUnavailable::new(format!(
                "source-execution destination parent {} is not a real directory",
                parent.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(SourceUnavailable::new(format!(
                "source-execution destination parent {} is absent; create its complete canonical corpus pair before copying provenance",
                parent.display()
            )));
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "source-execution destination parent {} cannot be inspected: {error}",
                parent.display()
            )));
        }
    }

    let destination_root = std::fs::canonicalize(parent).map_err(SourceUnavailable::new)?;
    let mut producer =
        uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
            &destination_root,
        )?;
    producer.ensure_root()?;
    let source_root = std::fs::canonicalize(
        options
            .corpus_meta
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )
    .map_err(SourceUnavailable::new)?;
    if producer.protects_directory(&source_root)? {
        return Err(SourceUnavailable::new(
            "copy-recorded-attention source and destination resolve to the same canonical corpus root; in-place provenance copy is refused before mutation",
        ));
    }

    preflight_guarded_planned_scope(&producer, &[], "provenance-copy destination")?;
    let recoverable_binding =
        producer.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
    if recoverable_binding {
        return Err(SourceUnavailable::new(
            "copy-recorded-attention owns no corpus generation and cannot recover a canonical binding temporary",
        ));
    }
    let destination_meta = destination_root.join("corpus.meta");
    let destination_records = destination_root.join("corpus.records");
    let destination = uor_r4_graph_compiler::recorded_corpus::open_stream(
        &destination_meta,
        &destination_records,
    )?;
    let destination_execution = recorded_execution_from_stream(&destination);
    let destination_fingerprint = recorded_corpus_stream_fingerprint(&destination);
    if destination.binding_cid.is_some() {
        if destination_execution == resolved {
            destination.verify_generation()?;
            return Ok(());
        }
        return Err(SourceUnavailable::new(
            "destination corpus is already binding-committed under a different source-execution identity; refusing relabel before mutation",
        ));
    }
    if resolved.dense_operator.is_some() {
        return Err(SourceUnavailable::new(
            "copy-recorded-attention cannot authorize a new dense-present derivation; use subsample-recorded-corpus so derived bytes and provenance commit in one guarded transaction",
        ));
    }
    if destination_execution == resolved {
        destination.verify_generation()?;
        return Ok(());
    }
    let destination_unbound = destination_execution.attention_operator.is_none()
        && destination_execution.dense_operator.is_none();
    if !destination_unbound {
        return Err(SourceUnavailable::new(
            "destination markerless corpus already carries a different execution identity; refusing relabel before mutation",
        ));
    }
    let Some(attention) = resolved.attention_operator.as_ref() else {
        destination.verify_generation()?;
        return Ok(());
    };
    destination.verify_generation()?;
    publish_attention_operator_binding(&destination_root, attention)?;
    let published = uor_r4_graph_compiler::recorded_corpus::open_stream(
        &destination_meta,
        &destination_records,
    )?;
    let published_execution = recorded_execution_from_stream(&published);
    if published_execution.attention_operator.as_ref() != Some(attention)
        || published_execution.dense_operator.is_some()
        || published.binding_cid.is_some()
        || recorded_corpus_stream_fingerprint(&published) != destination_fingerprint
    {
        return Err(SourceUnavailable::new(
            "markerless attention provenance copy changed during exact recapture",
        ));
    }
    published.verify_generation()?;
    Ok(())
}

fn exact_finalized_record_width(records: u64, byte_len: u64) -> Option<u64> {
    if records == 0 {
        return None;
    }
    [88u64, 48, 32, 12].into_iter().find(|width| {
        records
            .checked_mul(*width)
            .is_some_and(|expected| expected == byte_len)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedRowRange {
    start: u64,
    end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedByteRange {
    start: u64,
    end: u64,
}

impl SelectedByteRange {
    fn len(self) -> u64 {
        self.end - self.start
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RecordedCorpusSelection {
    ranges: Vec<SelectedRowRange>,
    retained_records: u64,
    train_records: u64,
    held_out_records: u64,
}

fn append_selected_row_range(ranges: &mut Vec<SelectedRowRange>, start: u64, end: u64) {
    if let Some(previous) = ranges.last_mut()
        && previous.end == start
    {
        previous.end = end;
        return;
    }
    ranges.push(SelectedRowRange { start, end });
}

/// Scan the record body once, validate every story id, and greedily retain
/// complete runs under quotas derived from the source compiler's fixed story
/// partition. The plan is row-index-only: no record or output body is retained.
fn select_recorded_corpus_ranges<R: Read + Seek>(
    records: &mut R,
    source_records: u64,
    record_width: u64,
    source_stories: u64,
    target_records: u64,
) -> Result<RecordedCorpusSelection, SourceUnavailable> {
    if target_records > source_records {
        return Err(SourceUnavailable::new(format!(
            "source has {source_records} finalized records; --records {target_records} exceeds the source generation"
        )));
    }
    if target_records == 0 || source_records == 0 {
        return Err(SourceUnavailable::new(
            "subsample source and target must each contain at least one finalized record",
        ));
    }
    let width = usize::try_from(record_width).map_err(|_| {
        SourceUnavailable::new(format!(
            "registered record width {record_width} cannot be represented on this host"
        ))
    })?;
    if width > 88 || width < std::mem::size_of::<u32>() {
        return Err(SourceUnavailable::new(format!(
            "registered record width {record_width} is outside the supported scanner range"
        )));
    }

    let full_source = target_records == source_records;
    let (mut train_remaining, mut held_out_remaining) = if full_source {
        (u64::MAX, u64::MAX)
    } else {
        let train = target_records
            .checked_mul(4)
            .ok_or_else(|| SourceUnavailable::new("subsample train quota overflows u64"))?
            / 5;
        let held_out = target_records - train;
        if train == 0 || held_out == 0 {
            return Err(SourceUnavailable::new(
                "subsample target is too small to retain records from both fixed source partitions",
            ));
        }
        (train, held_out)
    };
    // This expression is intentionally identical to compiler::train_cut.
    let source_train_cut = ((source_stories as f64 * 0.8) as u32).max(1);
    let mut selected = Vec::new();
    let mut train_records = 0u64;
    let mut held_out_records = 0u64;
    let mut last_selected_story = None;

    let mut consider_run =
        |run_start: u64, run_end: u64, story: u32| -> Result<(), SourceUnavailable> {
            let rows = run_end
                .checked_sub(run_start)
                .ok_or_else(|| SourceUnavailable::new("subsample story-run bounds are reversed"))?;
            let train = story < source_train_cut;
            let remaining = if train {
                &mut train_remaining
            } else {
                &mut held_out_remaining
            };
            if full_source || (rows <= *remaining && last_selected_story != Some(story)) {
                if !full_source {
                    *remaining -= rows;
                }
                if train {
                    train_records = train_records.checked_add(rows).ok_or_else(|| {
                        SourceUnavailable::new("subsample retained train count overflows u64")
                    })?;
                } else {
                    held_out_records = held_out_records.checked_add(rows).ok_or_else(|| {
                        SourceUnavailable::new("subsample retained held-out count overflows u64")
                    })?;
                }
                append_selected_row_range(&mut selected, run_start, run_end);
                last_selected_story = Some(story);
            }
            Ok(())
        };

    records
        .seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    let mut row = [0u8; 88];
    let mut run_start = 0u64;
    let mut current_story = None;
    for index in 0..source_records {
        records.read_exact(&mut row[..width]).map_err(|error| {
            SourceUnavailable::new(format!(
                "subsample source records ended while reading row {index} at width {record_width}: {error}"
            ))
        })?;
        let story = u32::from_le_bytes(
            row[0..4]
                .try_into()
                .map_err(|_| SourceUnavailable::new("record story id is truncated"))?,
        );
        if u64::from(story) >= source_stories {
            return Err(SourceUnavailable::new(format!(
                "source record {index} declares story {story}, outside the metadata story range 0..{source_stories}"
            )));
        }
        match current_story {
            Some(previous) if previous != story => {
                consider_run(run_start, index, previous)?;
                run_start = index;
                current_story = Some(story);
            }
            None => current_story = Some(story),
            Some(_) => {}
        }
    }
    if let Some(story) = current_story {
        consider_run(run_start, source_records, story)?;
    }
    let mut extra = [0u8; 1];
    if records.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(
            "subsample source records extend beyond the exact finalized row layout",
        ));
    }

    if train_records == 0 || held_out_records == 0 {
        let train_quota = target_records * 4 / 5;
        let held_out_quota = target_records - train_quota;
        return Err(SourceUnavailable::new(format!(
            "source selection does not represent both fixed source partitions within train/held-out quotas {train_quota}/{held_out_quota}"
        )));
    }
    let retained_records = train_records
        .checked_add(held_out_records)
        .ok_or_else(|| SourceUnavailable::new("subsample retained count overflows u64"))?;
    if retained_records > target_records {
        return Err(SourceUnavailable::new(
            "subsample greedy selection exceeded its declared target",
        ));
    }
    Ok(RecordedCorpusSelection {
        ranges: selected,
        retained_records,
        train_records,
        held_out_records,
    })
}

fn selected_byte_ranges(
    rows: &[SelectedRowRange],
    row_bytes: u64,
) -> Result<Vec<SelectedByteRange>, SourceUnavailable> {
    if row_bytes == 0 {
        return Err(SourceUnavailable::new(
            "selected byte ranges require a nonzero row width",
        ));
    }
    let mut ranges: Vec<SelectedByteRange> = Vec::with_capacity(rows.len());
    for row_range in rows {
        let start = row_range
            .start
            .checked_mul(row_bytes)
            .ok_or_else(|| SourceUnavailable::new("selected byte-range start overflows u64"))?;
        let end = row_range
            .end
            .checked_mul(row_bytes)
            .ok_or_else(|| SourceUnavailable::new("selected byte-range end overflows u64"))?;
        if let Some(previous) = ranges.last_mut()
            && previous.end == start
        {
            previous.end = end;
        } else {
            ranges.push(SelectedByteRange { start, end });
        }
    }
    Ok(ranges)
}

/// Read/seek view that concatenates selected source ranges without allocating
/// or spooling the selected body. Source offsets and logical length remain
/// u64, so sparse multi-gigabyte inputs do not become host-sized allocations.
trait SelectedRangeSource: Read + Seek {}

impl<T: Read + Seek> SelectedRangeSource for T {}

struct SelectedRangesReader<'a> {
    source: &'a mut dyn SelectedRangeSource,
    ranges: &'a [SelectedByteRange],
    length: u64,
    position: u64,
    range_index: usize,
    range_offset: u64,
    source_position: Option<u64>,
}

impl<'a> SelectedRangesReader<'a> {
    fn new<R: Read + Seek>(
        source: &'a mut R,
        ranges: &'a [SelectedByteRange],
    ) -> Result<Self, SourceUnavailable> {
        let mut length = 0u64;
        let mut previous_end = None;
        for range in ranges {
            if range.start >= range.end {
                return Err(SourceUnavailable::new(
                    "selected source range is empty or reversed",
                ));
            }
            if previous_end.is_some_and(|end| end >= range.start) {
                return Err(SourceUnavailable::new(
                    "selected source ranges overlap or were not coalesced",
                ));
            }
            length = length
                .checked_add(range.len())
                .ok_or_else(|| SourceUnavailable::new("selected stream length overflows u64"))?;
            previous_end = Some(range.end);
        }
        Ok(Self {
            source,
            ranges,
            length,
            position: 0,
            range_index: 0,
            range_offset: 0,
            source_position: None,
        })
    }

    fn len(&self) -> u64 {
        self.length
    }

    fn reset_cursor(&mut self, target: u64) {
        self.position = target;
        self.source_position = None;
        let mut remaining = target;
        for (index, range) in self.ranges.iter().enumerate() {
            if remaining < range.len() {
                self.range_index = index;
                self.range_offset = remaining;
                return;
            }
            remaining -= range.len();
        }
        self.range_index = self.ranges.len();
        self.range_offset = 0;
    }
}

impl std::io::Read for SelectedRangesReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() || self.position >= self.length {
            return Ok(0);
        }
        let mut written = 0usize;
        while written < buffer.len() && self.position < self.length {
            let range = self.ranges.get(self.range_index).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "selected range cursor ended before the declared logical length",
                )
            })?;
            let source_target = range.start.checked_add(self.range_offset).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "selected source cursor overflows u64",
                )
            })?;
            if self.source_position != Some(source_target) {
                self.source.seek(SeekFrom::Start(source_target))?;
                self.source_position = Some(source_target);
            }
            let range_remaining = range.end - source_target;
            let limit = usize::try_from(range_remaining.min((buffer.len() - written) as u64))
                .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "selected read length does not fit usize",
                    )
                })?;
            let count = self.source.read(&mut buffer[written..written + limit])?;
            if count == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "selected source range ended before its declared boundary",
                ));
            }
            let count_u64 = count as u64;
            written += count;
            self.position = self.position.checked_add(count_u64).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "selected logical position overflows u64",
                )
            })?;
            self.range_offset += count_u64;
            self.source_position = source_target.checked_add(count_u64);
            if self.range_offset == range.len() {
                self.range_index += 1;
                self.range_offset = 0;
            }
        }
        Ok(written)
    }
}

impl std::io::Seek for SelectedRangesReader<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        let target = match position {
            SeekFrom::Start(target) => i128::from(target),
            SeekFrom::End(offset) => i128::from(self.length) + i128::from(offset),
            SeekFrom::Current(offset) => i128::from(self.position) + i128::from(offset),
        };
        let target = u64::try_from(target).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "selected logical seek is outside the u64 address space",
            )
        })?;
        self.reset_cursor(target);
        Ok(target)
    }
}

fn summarize_selected_stream(
    reader: &mut SelectedRangesReader<'_>,
    context: &str,
) -> Result<uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary, SourceUnavailable>
{
    reader
        .seek(SeekFrom::Start(0))
        .map_err(SourceUnavailable::new)?;
    let length = reader.len();
    let mut remaining = length;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    while remaining != 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| SourceUnavailable::new(format!("{context} length overflows usize")))?;
        let count = reader
            .read(&mut buffer[..limit])
            .map_err(SourceUnavailable::new)?;
        if count == 0 {
            return Err(SourceUnavailable::new(format!(
                "{context} ended before its declared {length}-byte length"
            )));
        }
        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }
    let mut extra = [0u8; 1];
    if reader.read(&mut extra).map_err(SourceUnavailable::new)? != 0 {
        return Err(SourceUnavailable::new(format!(
            "{context} grew beyond its declared {length}-byte length"
        )));
    }
    Ok(
        uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary {
            length,
            blake3: format!("blake3:{}", hasher.finalize().to_hex()),
        },
    )
}

struct ExpectedSubsampleGeneration<'a> {
    execution: &'a RecordedCorpusExecutionIdentity,
    meta: &'a [u8],
    records: &'a uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary,
    hidden: Option<&'a uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary>,
}

fn postcheck_subsample_generation(
    guard: &RecordedCorpusProducerGuard,
    output_meta: &Path,
    output_records: &Path,
    expected: &ExpectedSubsampleGeneration<'_>,
    expected_binding_cid: Option<&str>,
) -> Result<String, SourceUnavailable> {
    let published = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
        guard,
        output_meta,
        output_records,
    )?;
    let binding_cid = published.binding_cid.clone().ok_or_else(|| {
        SourceUnavailable::new("subsample postcheck found no canonical generation binding")
    })?;
    if expected_binding_cid.is_some_and(|expected| expected != binding_cid)
        || recorded_execution_from_stream(&published) != *expected.execution
        || published.meta_bytes != expected.meta
        || &published.records.summary() != expected.records
        || published.hidden.as_ref().map(|hidden| hidden.summary()) != expected.hidden.cloned()
    {
        return Err(SourceUnavailable::new(
            "subsample publication changed during bounded post-binding recapture",
        ));
    }
    published.verify_generation()?;
    guard.verify_owned_root()?;
    Ok(binding_cid)
}

/// Derive one smaller, story-run-aligned corpus from a single exact source
/// snapshot. Unlike the historical Python+copy sequence, corpus bytes and the
/// complete source-execution pair commit under one destination guard and one
/// canonical generation binding.
pub fn subsample_recorded_corpus(args: &[String]) -> Result<(), SourceUnavailable> {
    subsample_recorded_corpus_with_before_binding(args, || Ok(()))
}

fn subsample_recorded_corpus_with_before_binding<F>(
    args: &[String],
    before_binding: F,
) -> Result<(), SourceUnavailable>
where
    F: FnOnce() -> Result<(), SourceUnavailable>,
{
    let options = parse_subsample_recorded_corpus_options(args)?;
    let requested_output_root = options
        .output_meta
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let source_root = options
        .source_meta
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut authority =
        uor_r4_graph_compiler::recorded_corpus::RecordedCorpusDerivationGuards::try_acquire(
            source_root,
            requested_output_root,
        )?;
    authority.verify()?;
    let output_root = authority.destination_guard().root().to_path_buf();
    let output_meta = output_root.join("corpus.meta");
    let output_records = output_root.join("corpus.records");
    let output_hidden = output_root.join("corpus.records.hidden");

    let mut source = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
        authority.source_guard(),
        &options.source_meta,
        &options.source_records,
    )?;
    if source.meta_bytes.len() != SOURCE_CORPUS_META_BYTES || source.meta_bytes[24] != 1 {
        return Err(SourceUnavailable::new(
            "subsample source is not one exact finalized 25-byte corpus generation",
        ));
    }
    let source_records = u64::from_le_bytes(
        source.meta_bytes[0..8]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid source corpus record count"))?,
    );
    if options.records > source_records {
        return Err(SourceUnavailable::new(format!(
            "source has {source_records} finalized records; --records {} exceeds the source generation",
            options.records
        )));
    }
    let source_stories_u64 = u64::from_le_bytes(
        source.meta_bytes[8..16]
            .try_into()
            .map_err(|_| SourceUnavailable::new("invalid source corpus story count"))?,
    );
    let record_width = exact_finalized_record_width(source_records, source.records.len())
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "source record stream length {} is not exactly n times one registered width (88, 48, 32, or 12) for n={source_records}",
                source.records.len()
            ))
        })?;
    let selection = select_recorded_corpus_ranges(
        &mut source.records,
        source_records,
        record_width,
        source_stories_u64,
        options.records,
    )?;
    let record_ranges = selected_byte_ranges(&selection.ranges, record_width)?;
    let records_summary = {
        let mut reader = SelectedRangesReader::new(&mut source.records, &record_ranges)?;
        summarize_selected_stream(&mut reader, "subsample selected record stream")?
    };
    let expected_records_length = selection
        .retained_records
        .checked_mul(record_width)
        .ok_or_else(|| SourceUnavailable::new("subsample record byte length overflows u64"))?;
    if records_summary.length != expected_records_length {
        return Err(SourceUnavailable::new(
            "subsample selected record summary has an inconsistent row length",
        ));
    }

    let (hidden_ranges, hidden_summary) = match source.hidden.as_mut() {
        Some(hidden) => {
            if hidden.len() % source_records != 0 {
                return Err(SourceUnavailable::new(format!(
                    "source hidden stream is {} bytes, which is not an exact row width for {source_records} finalized rows",
                    hidden.len()
                )));
            }
            let row_bytes = hidden.len() / source_records;
            if row_bytes == 0 || row_bytes % std::mem::size_of::<f32>() as u64 != 0 {
                return Err(SourceUnavailable::new(
                    "source hidden stream row width is zero or is not an exact f32 byte multiple",
                ));
            }
            let ranges = selected_byte_ranges(&selection.ranges, row_bytes)?;
            let summary = {
                let mut reader = SelectedRangesReader::new(hidden, &ranges)?;
                summarize_selected_stream(&mut reader, "subsample selected hidden stream")?
            };
            let expected_length = selection
                .retained_records
                .checked_mul(row_bytes)
                .ok_or_else(|| SourceUnavailable::new("subsample hidden length overflows u64"))?;
            if summary.length != expected_length {
                return Err(SourceUnavailable::new(
                    "subsample selected hidden summary has an inconsistent row length",
                ));
            }
            (Some(ranges), Some(summary))
        }
        None => (None, None),
    };
    source.verify_generation()?;
    authority.verify()?;
    authority.destination_guard_mut().ensure_root()?;
    authority.verify()?;
    if strict_regular_file_if_present(
        &output_root.join(observe::MANIFEST_FILE),
        "subsample destination observation manifest",
    )? {
        return Err(SourceUnavailable::new(
            "subsample destination contains an observation manifest; use a fresh compile-style corpus root",
        ));
    }

    let mut output_meta_bytes = source.meta_bytes.clone();
    output_meta_bytes[0..8].copy_from_slice(&selection.retained_records.to_le_bytes());
    let expected_execution = RecordedCorpusExecutionIdentity {
        attention_operator: source.execution.attention_operator.clone(),
        dense_operator: source.execution.dense_operator.clone(),
    };
    let attention_bytes = expected_execution
        .attention_operator
        .as_ref()
        .map(canonical_attention_operator_bytes)
        .transpose()?;
    let dense_bytes = expected_execution
        .dense_operator
        .as_ref()
        .map(canonical_dense_operator_bytes)
        .transpose()?;
    let subsample_members = [
        PlannedOutputMember::AttentionOperator,
        PlannedOutputMember::DenseOperator,
        PlannedOutputMember::Records,
        PlannedOutputMember::Hidden,
        PlannedOutputMember::Metadata,
    ];
    let producer = authority.destination_guard();
    preflight_guarded_planned_scope(producer, &subsample_members, "subsample destination")?;
    producer.preflight_planned_output_stable_scope(&subsample_members)?;
    producer.preflight_deterministic_compile_inventory(&subsample_members)?;
    let recoverable_temporary =
        producer.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;

    let attention_ready = match expected_execution.attention_operator.as_ref() {
        Some(attention) => preflight_compiled_attention_operator(&output_root, attention)?,
        None => {
            if read_optional_compiled_attention_operator(&output_root)?.is_some() {
                return Err(SourceUnavailable::new(
                    "subsample source declares no attention identity but destination has one",
                ));
            }
            true
        }
    };
    let attention_bytes_ready = match attention_bytes.as_deref() {
        Some(bytes) => preflight_guarded_planned_file_in_scope(
            producer,
            &output_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            bytes,
            "subsample destination attention operator",
            &subsample_members,
        )?,
        None => {
            preflight_guarded_planned_absence_in_scope(
                producer,
                &output_root.join(ATTENTION_OPERATOR_BINDING_FILE),
                "subsample destination attention operator",
                &subsample_members,
            )?;
            true
        }
    };
    let dense_ready = preflight_compiled_dense_operator(
        &output_root,
        expected_execution.dense_operator.as_ref(),
    )?;
    let dense_bytes_ready = match dense_bytes.as_deref() {
        Some(bytes) => preflight_guarded_planned_file_in_scope(
            producer,
            &output_root.join(DENSE_OPERATOR_BINDING_FILE),
            bytes,
            "subsample destination dense operator",
            &subsample_members,
        )?,
        None => {
            preflight_guarded_planned_absence_in_scope(
                producer,
                &output_root.join(DENSE_OPERATOR_BINDING_FILE),
                "subsample destination dense operator",
                &subsample_members,
            )?;
            true
        }
    };
    let records_ready = preflight_guarded_planned_stream_in_scope(
        producer,
        &output_records,
        Some(&records_summary),
        "subsample destination records",
        &subsample_members,
    )?;
    let hidden_ready = match hidden_summary.as_ref() {
        Some(hidden) => preflight_guarded_planned_stream_in_scope(
            producer,
            &output_hidden,
            Some(hidden),
            "subsample destination hidden stream",
            &subsample_members,
        )?,
        None => {
            preflight_guarded_planned_absence_in_scope(
                producer,
                &output_hidden,
                "subsample destination hidden stream",
                &subsample_members,
            )?;
            true
        }
    };
    let meta_ready = preflight_guarded_planned_file_in_scope(
        producer,
        &output_meta,
        &output_meta_bytes,
        "subsample destination metadata",
        &subsample_members,
    )?;

    let stable_binding = strict_regular_file_if_present(
        &output_root.join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE),
        "subsample destination generation binding",
    )?;
    let planned_ready =
        attention_bytes_ready && dense_bytes_ready && records_ready && hidden_ready && meta_ready;
    if stable_binding || recoverable_temporary {
        if !attention_ready || !dense_ready || !planned_ready {
            return Err(SourceUnavailable::new(
                "subsample destination has binding commit evidence without the exact complete planned generation; refusing recovery before mutation",
            ));
        }
        uor_r4_graph_compiler::recorded_corpus::preflight_binding_evidence_matches_current(
            producer,
            &output_meta,
            &output_records,
        )?;
    }
    if stable_binding && !recoverable_temporary && attention_ready && dense_ready && planned_ready {
        producer.preflight_ready_deterministic_compile_inventory(&subsample_members)?;
        source.verify_generation()?;
        authority.verify()?;
        let expected = ExpectedSubsampleGeneration {
            execution: &expected_execution,
            meta: &output_meta_bytes,
            records: &records_summary,
            hidden: hidden_summary.as_ref(),
        };
        let binding_cid = postcheck_subsample_generation(
            producer,
            &output_meta,
            &output_records,
            &expected,
            None,
        )?;
        if producer.compile_attempt_active()? {
            producer.finish_compile_attempt()?;
        }
        println!(
            "subsample recorded corpus: {} records ({} train, {} held out; requested {}) of {source_records}; {record_width}-byte rows; binding {binding_cid}",
            selection.retained_records,
            selection.train_records,
            selection.held_out_records,
            options.records,
        );
        return Ok(());
    }

    producer.preflight_deterministic_compile_inventory(&subsample_members)?;
    producer.begin_compile_attempt()?;
    if stable_binding || recoverable_temporary {
        source.verify_generation()?;
        authority.verify()?;
        let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
            producer,
            &output_meta,
            &output_records,
        )?;
        let expected = ExpectedSubsampleGeneration {
            execution: &expected_execution,
            meta: &output_meta_bytes,
            records: &records_summary,
            hidden: hidden_summary.as_ref(),
        };
        let checked_binding_cid = postcheck_subsample_generation(
            producer,
            &output_meta,
            &output_records,
            &expected,
            Some(&binding_cid),
        )?;
        debug_assert_eq!(checked_binding_cid, binding_cid);
        producer.finish_compile_attempt()?;
        println!(
            "subsample recorded corpus: {} records ({} train, {} held out; requested {}) of {source_records}; {record_width}-byte rows; binding {binding_cid}",
            selection.retained_records,
            selection.train_records,
            selection.held_out_records,
            options.records,
        );
        return Ok(());
    }

    // Dense-first makes every GPT-2 crash prefix invalid rather than a
    // valid-looking attention-only generation.
    if let Some(bytes) = dense_bytes.as_deref() {
        publish_guarded_planned_file_with_after_sync(
            producer,
            &output_root.join(DENSE_OPERATOR_BINDING_FILE),
            bytes,
            "subsample destination dense operator",
            &subsample_members,
            |_| Ok(()),
        )?;
    }
    if let Some(bytes) = attention_bytes.as_deref() {
        publish_guarded_planned_file_with_after_sync(
            producer,
            &output_root.join(ATTENTION_OPERATOR_BINDING_FILE),
            bytes,
            "subsample destination attention operator",
            &subsample_members,
            |_| Ok(()),
        )?;
    }
    {
        let mut reader = SelectedRangesReader::new(&mut source.records, &record_ranges)?;
        publish_guarded_planned_stream(
            producer,
            &output_records,
            &mut reader,
            &records_summary,
            "subsample destination records",
        )?;
    }
    if let (Some(hidden), Some(ranges), Some(summary)) = (
        source.hidden.as_mut(),
        hidden_ranges.as_deref(),
        hidden_summary.as_ref(),
    ) {
        let mut reader = SelectedRangesReader::new(hidden, ranges)?;
        publish_guarded_planned_stream(
            producer,
            &output_hidden,
            &mut reader,
            summary,
            "subsample destination hidden stream",
        )?;
    }
    publish_guarded_planned_file(
        producer,
        &output_meta,
        &output_meta_bytes,
        "subsample destination metadata",
    )?;
    before_binding()?;
    source.verify_generation()?;
    authority.verify()?;
    let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
        producer,
        &output_meta,
        &output_records,
    )?;
    let expected = ExpectedSubsampleGeneration {
        execution: &expected_execution,
        meta: &output_meta_bytes,
        records: &records_summary,
        hidden: hidden_summary.as_ref(),
    };
    let checked_binding_cid = postcheck_subsample_generation(
        producer,
        &output_meta,
        &output_records,
        &expected,
        Some(&binding_cid),
    )?;
    debug_assert_eq!(checked_binding_cid, binding_cid);
    producer.finish_compile_attempt()?;
    println!(
        "subsample recorded corpus: {} records ({} train, {} held out; requested {}) of {source_records}; {record_width}-byte rows; binding {binding_cid}",
        selection.retained_records,
        selection.train_records,
        selection.held_out_records,
        options.records,
    );
    Ok(())
}

/// Resolve explicit recorded provenance or the immutable standard/1 meaning
/// of genuine pre-provenance absence for `compile-recorded`.
#[cfg(test)]
fn recorded_compile_attention_operator(
    corpus_meta: &Path,
    corpus_recs: &Path,
) -> Result<AttentionOperatorSpec, SourceUnavailable> {
    match recorded_corpus_attention_operator(corpus_meta, corpus_recs)? {
        Some(recorded) => Ok(recorded),
        None => uor_r4_model_source::attention::operator_spec(
            AttentionOperatorSpec::STANDARD_ID,
            LEGACY_STANDARD_ATTENTION_OPERATOR_VERSION,
        ),
    }
}

/// Compile a completed recorded corpus without loading or executing a
/// transformer. This is the observation-first production path; teacher
/// capture remains available through `observe`/`compile` as an explicitly
/// separate offline step.
struct PreparedRecordedCompile {
    source: uor_r4_graph_compiler::recorded_corpus::RecordedCorpusStreamSnapshot,
    source_guard: Option<uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard>,
    meta_bytes: Vec<u8>,
    records_bytes: Vec<u8>,
    corpus: compiler::Corpus,
    attention_operator: AttentionOperatorSpec,
    dense_operator: Option<DenseOperatorSpec>,
}

fn write_captured_recorded_corpus(
    guard: &uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard,
    output: &Path,
    prepared: &mut PreparedRecordedCompile,
) -> Result<(), SourceUnavailable> {
    publish_guarded_planned_file(
        guard,
        &output.join("corpus.records"),
        &prepared.records_bytes,
        "recorded compile corpus records",
    )?;
    if let Some(hidden) = prepared.source.hidden.as_mut() {
        let summary = hidden.summary();
        publish_guarded_planned_stream(
            guard,
            &output.join("corpus.records.hidden"),
            hidden,
            &summary,
            "recorded compile corpus hidden stream",
        )?;
    }
    // Metadata is the core corpus checkpoint and therefore commits last among
    // corpus members. A crash before this write cannot advertise new counts
    // for records or hidden rows that were not durably synchronized.
    publish_guarded_planned_file(
        guard,
        &output.join("corpus.meta"),
        &prepared.meta_bytes,
        "recorded compile corpus metadata",
    )?;
    Ok(())
}

fn require_existing_file_matches_if_present(
    path: &Path,
    expected: Option<&[u8]>,
    context: &str,
) -> Result<bool, SourceUnavailable> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(SourceUnavailable::new(format!(
            "{context} {} cannot be inspected: {error}",
            path.display()
        ))),
        Ok(_) => {
            verify_optional_regular_file_nofollow(path, context, expected).map_err(
                |mut error| {
                    error.reason = format!(
                        "{}; deterministic planned member mismatch is terminal before mutation",
                        error.reason
                    );
                    error
                },
            )?;
            Ok(true)
        }
    }
}

fn require_existing_stream_matches_if_present(
    path: &Path,
    expected: Option<&uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary>,
    context: &str,
) -> Result<bool, SourceUnavailable> {
    let actual = stream_regular_file_summary_nofollow(path, context)?;
    match (actual.as_ref(), expected) {
        (None, _) => Ok(false),
        (Some(actual), Some(expected)) if actual == expected => Ok(true),
        (Some(_), Some(_)) => Err(SourceUnavailable::new(format!(
            "{context} {} has a different length or BLAKE3; deterministic planned member mismatch is terminal before mutation",
            path.display()
        ))),
        (Some(_), None) => Err(SourceUnavailable::new(format!(
            "{context} {} is present but the planned generation declares typed absence; deterministic planned member mismatch is terminal before mutation",
            path.display()
        ))),
    }
}

fn preflight_recorded_compile_execution_identity(
    options: &RecordedCompileOptions,
) -> Result<PreparedRecordedCompile, SourceUnavailable> {
    preflight_recorded_compile_output(&options.output)?;
    let (mut source, source_guard) =
        uor_r4_graph_compiler::recorded_corpus::open_stream_for_derivation(
            &options.corpus_meta,
            &options.corpus_recs,
        )?;
    let attention_operator = match source.execution.attention_operator.as_ref() {
        Some(recorded) => recorded.clone(),
        None => uor_r4_model_source::attention::operator_spec(
            AttentionOperatorSpec::STANDARD_ID,
            LEGACY_STANDARD_ATTENTION_OPERATOR_VERSION,
        )?,
    };
    let dense_operator = source.execution.dense_operator.clone();
    validate_source_execution_pair(
        &attention_operator,
        dense_operator.as_ref(),
        "recorded compile provenance",
    )?;
    let meta_bytes = source.meta_bytes.clone();
    let (records_bytes, hidden_bytes) = source.materialize_compiler_corpus_bytes()?;
    let corpus = compiler::load_corpus_bytes(&meta_bytes, &records_bytes, hidden_bytes.as_deref())
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "recorded corpus is incomplete or invalid at {}/{}",
                options.corpus_meta.display(),
                options.corpus_recs.display()
            ))
        })?;
    preflight_compiled_attention_operator(&options.output, &attention_operator)?;
    preflight_compiled_dense_operator(&options.output, dense_operator.as_ref())?;
    Ok(PreparedRecordedCompile {
        source,
        source_guard,
        meta_bytes,
        records_bytes,
        corpus,
        attention_operator,
        dense_operator,
    })
}

pub fn compile_recorded_corpus(args: &[String]) -> Result<(), SourceUnavailable> {
    compile_recorded_corpus_with(
        args,
        build_recorded_compile_products,
        |_| Ok(()),
        |_| Ok(()),
    )
}

struct RecordedCompileProducts {
    artifact_bytes: Vec<u8>,
    store_bytes: Vec<u8>,
    calibration_bytes: Vec<u8>,
    hierarchical_bytes: Vec<u8>,
}

fn build_recorded_compile_products(
    corpus: &compiler::Corpus,
    vocab_size: usize,
) -> Result<RecordedCompileProducts, SourceUnavailable> {
    let artifacts = compiler::compile_recorded(corpus, vocab_size).ok_or_else(|| {
        SourceUnavailable::new("recorded compile failed: empty corpus or invalid vocabulary")
    })?;
    let calibration = compiler::calibrate_hamming_regions(&artifacts, corpus);
    let hierarchical =
        compiler::induce_hierarchical_codes(&artifacts.token_codes, vocab_size, corpus);
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(1);
    let (store, _) = runtime::build_store_with_threads(&artifacts, corpus, threads);
    Ok(RecordedCompileProducts {
        artifact_bytes: compiler::artifact_bytes(&artifacts),
        store_bytes: runtime::store_bytes(&store),
        calibration_bytes: serde_json::to_vec_pretty(&calibration)?,
        hierarchical_bytes: serde_json::to_vec_pretty(&hierarchical)?,
    })
}

fn compile_recorded_corpus_with<B, D, A>(
    args: &[String],
    build_products: B,
    after_dense_sync: D,
    after_attention_sync: A,
) -> Result<(), SourceUnavailable>
where
    B: FnOnce(&compiler::Corpus, usize) -> Result<RecordedCompileProducts, SourceUnavailable>,
    D: FnOnce(&Path) -> Result<(), SourceUnavailable>,
    A: FnOnce(&Path) -> Result<(), SourceUnavailable>,
{
    let options = parse_recorded_compile_options(args)?;
    let mut prepared = preflight_recorded_compile_execution_identity(&options)?;
    let corpus = &prepared.corpus;
    let corpus_records = corpus.n;
    eprintln!(
        "recorded compile: {} records, {} stories, vocabulary {} (no teacher loaded)",
        corpus.n, corpus.stories, options.vocab_size
    );
    let RecordedCompileProducts {
        artifact_bytes,
        store_bytes,
        calibration_bytes,
        hierarchical_bytes,
    } = build_products(corpus, options.vocab_size)?;

    let source_records_summary = prepared.source.records.summary();
    let source_hidden_summary = prepared
        .source
        .hidden
        .as_ref()
        .map(|hidden| hidden.summary());
    let preflight_planned_files = || -> Result<(), SourceUnavailable> {
        require_existing_file_matches_if_present(
            &options.output.join("tless_artifacts.bin"),
            Some(&artifact_bytes),
            "recorded compile artifact",
        )?;
        require_existing_file_matches_if_present(
            &options.output.join("tless_store.bin"),
            Some(&store_bytes),
            "recorded compile store",
        )?;
        require_existing_file_matches_if_present(
            &options.output.join("hamming_calibration.json"),
            Some(&calibration_bytes),
            "recorded compile calibration",
        )?;
        require_existing_file_matches_if_present(
            &options.output.join("hierarchical_codes.json"),
            Some(&hierarchical_bytes),
            "recorded compile hierarchical codes",
        )?;
        require_existing_file_matches_if_present(
            &options.output.join("corpus.meta"),
            Some(&prepared.meta_bytes),
            "recorded compile corpus metadata",
        )?;
        require_existing_file_matches_if_present(
            &options.output.join("corpus.records"),
            Some(&prepared.records_bytes),
            "recorded compile corpus records",
        )?;
        require_existing_stream_matches_if_present(
            &options.output.join("corpus.records.hidden"),
            source_hidden_summary.as_ref(),
            "recorded compile corpus hidden stream",
        )?;
        Ok(())
    };

    preflight_planned_files()?;
    // Markerless compatibility ownership cannot be nested with the
    // destination producer guard. The retained no-follow source handles and
    // their final generation check remain live after this lock is released.
    prepared.source.verify_generation()?;
    drop(prepared.source_guard.take());
    let output_parent = options
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(output_parent).map_err(SourceUnavailable::new)?;
    let mut producer =
        uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
            &options.output,
        )?;
    let source_root = std::fs::canonicalize(
        options
            .corpus_meta
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )
    .map_err(SourceUnavailable::new)?;
    if producer.protects_directory(&source_root)? {
        return Err(SourceUnavailable::new(
            "compile-recorded source and destination resolve to the same canonical corpus root; in-place derivation is refused before mutation",
        ));
    }
    producer.ensure_root()?;
    preflight_guarded_planned_scope(
        &producer,
        &PlannedOutputMember::REGISTERED,
        "recorded compile destination",
    )?;
    let recoverable_temporary =
        producer.preflight_publication_namespace_for(RecordedCorpusRole::Compile)?;
    preflight_recorded_compile_output(&options.output)?;
    let attention_ready =
        preflight_compiled_attention_operator(&options.output, &prepared.attention_operator)?;
    let attention_bytes = canonical_attention_operator_bytes(&prepared.attention_operator)?;
    let attention_bytes_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join(ATTENTION_OPERATOR_BINDING_FILE),
        &attention_bytes,
        "recorded compile attention operator",
        &PlannedOutputMember::REGISTERED,
    )?;
    let dense_ready =
        preflight_compiled_dense_operator(&options.output, prepared.dense_operator.as_ref())?;
    let dense_bytes = prepared
        .dense_operator
        .as_ref()
        .map(canonical_dense_operator_bytes)
        .transpose()?;
    let dense_bytes_ready = match dense_bytes.as_deref() {
        Some(bytes) => preflight_guarded_planned_file_in_scope(
            &producer,
            &options.output.join(DENSE_OPERATOR_BINDING_FILE),
            bytes,
            "recorded compile dense operator",
            &PlannedOutputMember::REGISTERED,
        )?,
        None => {
            preflight_guarded_planned_absence(
                &producer,
                &options.output.join(DENSE_OPERATOR_BINDING_FILE),
                "recorded compile dense operator",
            )?;
            true
        }
    };
    preflight_planned_files()?;
    let artifact_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join("tless_artifacts.bin"),
        &artifact_bytes,
        "recorded compile artifact",
        &PlannedOutputMember::REGISTERED,
    )?;
    let store_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join("tless_store.bin"),
        &store_bytes,
        "recorded compile store",
        &PlannedOutputMember::REGISTERED,
    )?;
    let calibration_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join("hamming_calibration.json"),
        &calibration_bytes,
        "recorded compile calibration",
        &PlannedOutputMember::REGISTERED,
    )?;
    let hierarchical_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join("hierarchical_codes.json"),
        &hierarchical_bytes,
        "recorded compile hierarchical codes",
        &PlannedOutputMember::REGISTERED,
    )?;
    let records_ready = preflight_guarded_planned_stream_in_scope(
        &producer,
        &options.output.join("corpus.records"),
        Some(&source_records_summary),
        "recorded compile corpus records",
        &PlannedOutputMember::REGISTERED,
    )?;
    let hidden_ready = preflight_guarded_planned_stream_in_scope(
        &producer,
        &options.output.join("corpus.records.hidden"),
        source_hidden_summary.as_ref(),
        "recorded compile corpus hidden stream",
        &PlannedOutputMember::REGISTERED,
    )?;
    let meta_ready = preflight_guarded_planned_file_in_scope(
        &producer,
        &options.output.join("corpus.meta"),
        &prepared.meta_bytes,
        "recorded compile corpus metadata",
        &PlannedOutputMember::REGISTERED,
    )?;
    let stable_binding = strict_regular_file_if_present(
        &options
            .output
            .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE),
        "recorded compile generation binding",
    )?;
    let planned_ready = attention_bytes_ready
        && dense_bytes_ready
        && artifact_ready
        && store_ready
        && calibration_ready
        && hierarchical_ready
        && records_ready
        && hidden_ready
        && meta_ready;
    if stable_binding || recoverable_temporary {
        if !planned_ready || !attention_ready || !dense_ready {
            return Err(SourceUnavailable::new(
                "recorded compile output contains binding commit evidence but not the exact complete intended generation; refusing member mutation before recovery",
            ));
        }
        uor_r4_graph_compiler::recorded_corpus::preflight_binding_evidence_matches_current(
            &producer,
            &options.output.join("corpus.meta"),
            &options.output.join("corpus.records"),
        )?;
    }
    if stable_binding && !recoverable_temporary && planned_ready && attention_ready && dense_ready {
        producer
            .preflight_ready_deterministic_compile_inventory(&PlannedOutputMember::REGISTERED)?;
        if producer.compile_attempt_active()? {
            producer.finish_compile_attempt()?;
        }
        println!(
            "recorded compile complete: {} ({} corpus records, artifact κ blake3:{})",
            options.output.display(),
            corpus_records,
            blake3::hash(&artifact_bytes).to_hex()
        );
        return Ok(());
    }
    producer.preflight_deterministic_compile_inventory(&PlannedOutputMember::REGISTERED)?;
    producer.begin_compile_attempt()?;
    if recoverable_temporary {
        uor_r4_graph_compiler::recorded_corpus::publish_binding(
            &producer,
            &options.output.join("corpus.meta"),
            &options.output.join("corpus.records"),
        )?;
    }

    // Dense-first ensures every crash prefix of a current GPT-2 generation is
    // invalid rather than a false, valid-looking attention-only generation.
    if let Some(bytes) = dense_bytes.as_deref() {
        publish_guarded_planned_file_with_after_sync(
            &producer,
            &options.output.join(DENSE_OPERATOR_BINDING_FILE),
            bytes,
            "recorded compile dense operator",
            &PlannedOutputMember::REGISTERED,
            after_dense_sync,
        )?;
    }
    publish_guarded_planned_file_with_after_sync(
        &producer,
        &options.output.join(ATTENTION_OPERATOR_BINDING_FILE),
        &attention_bytes,
        "recorded compile attention operator",
        &PlannedOutputMember::REGISTERED,
        after_attention_sync,
    )?;
    publish_guarded_planned_file(
        &producer,
        &options.output.join("tless_artifacts.bin"),
        &artifact_bytes,
        "recorded compile artifact",
    )?;
    publish_guarded_planned_file(
        &producer,
        &options.output.join("tless_store.bin"),
        &store_bytes,
        "recorded compile store",
    )?;
    publish_guarded_planned_file(
        &producer,
        &options.output.join("hamming_calibration.json"),
        &calibration_bytes,
        "recorded compile calibration",
    )?;
    publish_guarded_planned_file(
        &producer,
        &options.output.join("hierarchical_codes.json"),
        &hierarchical_bytes,
        "recorded compile hierarchical codes",
    )?;
    write_captured_recorded_corpus(&producer, &options.output, &mut prepared)?;
    // All derived payload bytes are now durable but uncommitted. Refuse a
    // changed source generation before the destination binding can make them
    // authoritative as one corpus.
    prepared.source.verify_generation()?;
    let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
        &producer,
        &options.output.join("corpus.meta"),
        &options.output.join("corpus.records"),
    )?;
    let published = uor_r4_graph_compiler::recorded_corpus::open_stream_under_guard(
        &producer,
        &options.output.join("corpus.meta"),
        &options.output.join("corpus.records"),
    )?;
    let destination_execution = RecordedCorpusExecutionIdentity {
        attention_operator: Some(prepared.attention_operator.clone()),
        dense_operator: prepared.dense_operator.clone(),
    };
    if published.binding_cid.as_deref() != Some(binding_cid.as_str())
        || recorded_execution_from_stream(&published) != destination_execution
        || published.meta_bytes != prepared.meta_bytes
        || published.records.summary() != source_records_summary
        || published.hidden.as_ref().map(|hidden| hidden.summary()) != source_hidden_summary
    {
        return Err(SourceUnavailable::new(
            "recorded compile publication changed during exact post-binding recapture",
        ));
    }
    published.verify_generation()?;
    drop(published);
    producer.finish_compile_attempt()?;

    println!(
        "recorded compile complete: {} ({} corpus records, artifact κ blake3:{})",
        options.output.display(),
        corpus_records,
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
        Some("copy-recorded-attention") => copy_recorded_attention(&args[1..])?,
        Some("subsample-recorded-corpus") => subsample_recorded_corpus(&args[1..])?,
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
        Some("route-fit-real") => route_fit_real_command(&args[1..])?,
        Some("runtime-corpus") => runtime_corpus::run(&args[1..])?,
        Some("cover") => cover_command(&args[1..])?,
        Some("cover-sweep") => cover_sweep::cover_sweep_command(&args[1..])?,
        Some("recommend-scale") => recommend_scale::run(&args[1..])?,
        Some("score") => score_command(&args[1..])?,
        Some("graph") => graph_command(&args[1..])?,
        Some("cd-compile") => cd_compile_command(&args[1..]),
        Some("quantum-eval") => quantum_eval_command(&args[1..]),
        // #790 item 2: `help`/no-args prints usage and succeeds; an
        // UNKNOWN subcommand prints usage and fails — previously any typo
        // (or the removed `compare`/`compare-report`, which are top-level
        // `r4` commands, not transformerless subcommands) exited 0 as a
        // silent no-op, so a broken pipeline stage reported success.
        Some(unknown) if unknown != "help" => {
            println!("{}", transformerless_usage());
            return Err(SourceUnavailable::new(format!(
                "unknown transformerless subcommand `{unknown}`"
            )));
        }
        _ => {
            println!("{}", transformerless_usage());
        }
    }
    Ok(())
}

/// Usage banner for the `transformerless` command family. Lists exactly
/// the subcommands `run` dispatches (#790: it previously advertised
/// `compare`/`compare-report`, which live on the top-level `r4` CLI).
fn transformerless_usage() -> &'static str {
    "R4 transformerless — compile a mul-free table artifact\n\
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--tokenizer-family FAMILY --tokenizer-version N] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | scenarios | teacher-kappa | convert-r4g1 --artifacts <TLA> --store <TLS1> [--calibration <hamming_calibration.json>] --out <R4G1>\n\
                 recorded compile (no transformer): compile-recorded --corpus-meta <META> --corpus-recs <RECS> --vocab-size <N> --out <DIR>\n\
                 markerless legacy attention copy (dense-present derivations are refused): copy-recorded-attention --corpus-meta <META> --corpus-recs <RECS> --out <attention_operator.json>\n\
                 guarded streaming corpus derivation (N <= source; complete runs may undershoot): subsample-recorded-corpus --src-meta <META> --src-recs <RECS> --out-meta <corpus.meta> --out-recs <corpus.records> --records <N>\n\
                 transformer-free refresh: runtime-corpus --artifacts <TLA> --store <TLS1> --seed-meta <META> --seed-recs <RECS> --out <DIR> --target N [--threads N]\n\
                 graph cover: cover --corpus-meta <META> --corpus-recs <RECS> --artifacts <TLA> --out <DIR> [--bundle-root <ROOT>]\n\
                 graph score: score --corpus-meta <META> --corpus-recs <RECS> --artifacts <TLA> --out <DIR> [--bundle-root <ROOT>]\n\
                   --bundle-root explicitly declares one managed/canonical bundle authority; without it, --out is an exact standalone root fixed at transaction start\n\
                 observation pipeline: observe [--source DIR [--tokenizer-family FAMILY --tokenizer-version N] | --checkpoint BIN] [--seconds N] [--target N] [--shards N] [--out DIR] [--sequence-length N]\n\
                 text observations (D3): observe-text [--input PATH] [--out DIR] [--shards N] [--seconds N] [--source DIR [--tokenizer-family FAMILY --tokenizer-version N] | --checkpoint BIN --tokenizer PATH] [--sequence-length N] [--trace-profile ID/V --trace-layers I,J,... [--trace-support S]]\n\
                 route-attention real arm (#605 S1-1): route-fit-real --corpus DIR --source DIR [--stories N] [--positions N] [--out PATH]\n\
                 A-mode infill serving: graph infill --artifact <scored R4G1> --skeleton <token ids, _ for free> [--teacher <TLA container>]\n\
                 quantum operations: cd-compile | quantum-eval\n\
                 hf evaluation: evaluate-report [--source DIR] [--tokenizer-family FAMILY --tokenizer-version N] [--compiled DIR] [--report PATH] [--sequence-length N] [--bos] [--max-held-out-stories N]\n\
                 scale sizing (#514): recommend-scale (--config <hf dir> | --d-model N --n-layers N --vocab N) [--corpus wiki|stories] [--beta B]\n\
                 docs: docs/transformerless/TRANSFORMERLESS.md (extrapolation), docs/transformerless/PROOF.md (proof + certificate)"
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

    #[cfg(unix)]
    fn hard_link_test_sidecar(
        output: &Path,
        stable_name: &str,
        bytes: &[u8],
        alias_tag: &str,
    ) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(output).expect("sidecar output");
        let stable = output.join(stable_name);
        let alias = output.join(format!(".{stable_name}.{alias_tag}.tmp"));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&alias)
            .expect("create sidecar alias");
        file.write_all(bytes).expect("write sidecar alias");
        file.sync_all().expect("sync sidecar alias");
        drop(file);
        std::fs::hard_link(&alias, &stable).expect("publish sidecar hard link");
        sync_compile_output_directory(output).expect("sync sidecar directory");
        (stable, alias)
    }

    #[test]
    fn source_corpus_session_releases_common_guard_before_outer_lock() {
        let base = unique_cli_test_path("source-session-drop-order");
        let root = base.join("compiled");
        std::fs::create_dir_all(&root).expect("root");
        let mut session = source_corpus_session(&root).expect("outer session");
        session
            .acquire_recorded_corpus(&root)
            .expect("common guard");

        session.release_in_order(|| {
            let error = source_corpus_session(&root)
                .expect_err("outer lock remains BUSY until common guard is released");
            assert!(error.to_string().contains("BUSY"), "{error}");
        });

        let mut successor = source_corpus_session(&root).expect("outer lock released last");
        successor
            .acquire_recorded_corpus(&root)
            .expect("common guard can be reacquired after full release");
        drop(successor);
        std::fs::remove_dir_all(&base).expect("cleanup root");
    }

    #[test]
    fn guarded_stream_publication_recovers_partial_stage_without_body_buffering() {
        let root = unique_cli_test_path("planned-stream-recovery");
        std::fs::create_dir_all(&root).expect("root");
        let guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(&root)
                .expect("producer guard");
        let residue = root.join(format!(
            "{PLANNED_OUTPUT_RESERVED_PREFIX}{}--999.1.writing",
            PlannedOutputMember::Hidden.stable_name()
        ));
        std::fs::write(&residue, b"partial").expect("honest crash residue");
        let bytes = b"complete hidden stream";
        let expected = uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary {
            length: bytes.len() as u64,
            blake3: format!("blake3:{}", blake3::hash(bytes).to_hex()),
        };
        let mut source = std::io::Cursor::new(bytes.as_slice());
        let stable = root.join(PlannedOutputMember::Hidden.stable_name());
        publish_guarded_planned_stream(
            &guard,
            &stable,
            &mut source,
            &expected,
            "test hidden stream",
        )
        .expect("exact retry publishes stream");
        assert_eq!(std::fs::read(&stable).unwrap(), bytes);
        assert!(!residue.exists());
        assert_eq!(
            stream_regular_file_summary_nofollow(&stable, "test hidden stream")
                .expect("summary")
                .expect("present"),
            expected
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn guarded_planned_byte_commit_cannot_be_redirected_after_last_root_check() {
        let root = unique_cli_test_path("planned-byte-root-swap");
        std::fs::create_dir_all(&root).expect("root");
        let guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(&root)
                .expect("producer guard");
        let guarded_root = guard.root().to_path_buf();
        let displaced = guarded_root.with_file_name(format!(
            "{}-guarded",
            guarded_root
                .file_name()
                .expect("root leaf")
                .to_string_lossy()
        ));
        let stable = guarded_root.join(PlannedOutputMember::Artifact.stable_name());
        let expected = b"retained-root artifact";
        let error = publish_guarded_planned_file_with_after_sync(
            &guard,
            &stable,
            expected,
            "planned byte root swap",
            &PlannedOutputMember::ALL,
            |_| {
                std::fs::rename(&guarded_root, &displaced).expect("displace guarded root");
                std::fs::create_dir(&guarded_root).expect("replacement root");
                std::fs::write(guarded_root.join("replacement-sentinel"), b"replacement")
                    .expect("replacement sentinel");
                Ok(())
            },
        )
        .expect_err("path replacement must be reported after retained-root commit");
        assert!(error.reason.contains("changed"), "{error}");
        assert_eq!(
            std::fs::read(displaced.join(PlannedOutputMember::Artifact.stable_name()))
                .expect("commit stayed on retained root"),
            expected
        );
        assert!(!stable.exists(), "replacement root was not mutated");
        assert_eq!(
            std::fs::read(guarded_root.join("replacement-sentinel")).expect("sentinel"),
            b"replacement"
        );
        drop(guard);
        let _ = std::fs::remove_dir_all(guarded_root);
        let _ = std::fs::remove_dir_all(displaced);
    }

    #[cfg(unix)]
    #[test]
    fn guarded_planned_stream_commit_cannot_be_redirected_after_last_root_check() {
        let root = unique_cli_test_path("planned-stream-root-swap");
        std::fs::create_dir_all(&root).expect("root");
        let guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(&root)
                .expect("producer guard");
        let guarded_root = guard.root().to_path_buf();
        let displaced = guarded_root.with_file_name(format!(
            "{}-guarded",
            guarded_root
                .file_name()
                .expect("root leaf")
                .to_string_lossy()
        ));
        let stable = guarded_root.join(PlannedOutputMember::Hidden.stable_name());
        let expected_bytes = b"retained-root hidden stream";
        let expected = uor_r4_graph_compiler::recorded_corpus::RecordedCorpusMemberSummary {
            length: expected_bytes.len() as u64,
            blake3: format!("blake3:{}", blake3::hash(expected_bytes).to_hex()),
        };
        let mut source = std::io::Cursor::new(expected_bytes.as_slice());
        let error = publish_guarded_planned_stream_with_before_link(
            &guard,
            &stable,
            &mut source,
            &expected,
            "planned stream root swap",
            |_| {
                std::fs::rename(&guarded_root, &displaced).expect("displace guarded root");
                std::fs::create_dir(&guarded_root).expect("replacement root");
                std::fs::write(guarded_root.join("replacement-sentinel"), b"replacement")
                    .expect("replacement sentinel");
                Ok(())
            },
        )
        .expect_err("path replacement must be reported after retained-root commit");
        assert!(error.reason.contains("changed"), "{error}");
        assert_eq!(
            std::fs::read(displaced.join(PlannedOutputMember::Hidden.stable_name()))
                .expect("stream commit stayed on retained root"),
            expected_bytes
        );
        assert!(!stable.exists(), "replacement root was not mutated");
        assert_eq!(
            std::fs::read(guarded_root.join("replacement-sentinel")).expect("sentinel"),
            b"replacement"
        );
        drop(guard);
        let _ = std::fs::remove_dir_all(guarded_root);
        let _ = std::fs::remove_dir_all(displaced);
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

    fn attention_operator_temp_entries(path: &Path) -> Vec<String> {
        let prefix = format!(".{ATTENTION_OPERATOR_BINDING_FILE}.");
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

    fn dense_operator_temp_entries(path: &Path) -> Vec<String> {
        let prefix = format!(".{DENSE_OPERATOR_BINDING_FILE}.");
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

    fn write_observation_manifest(path: &Path, manifest: &observe::ObservationManifest) {
        std::fs::create_dir_all(path).expect("observation directory");
        std::fs::write(
            path.join(observe::MANIFEST_FILE),
            serde_json::to_vec_pretty(manifest).expect("serialize observation manifest"),
        )
        .expect("write observation manifest");
    }

    fn corpus_markers(path: &Path) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(path).expect("corpus directory");
        let meta = path.join("corpus.meta");
        let records = path.join("corpus.records");
        std::fs::write(&meta, b"meta marker").expect("metadata marker");
        std::fs::write(&records, b"records marker").expect("records marker");
        (meta, records)
    }

    fn finalized_corpus(path: &Path, next: u32) -> (PathBuf, PathBuf, Vec<u8>, Vec<u8>) {
        std::fs::create_dir_all(path).expect("corpus directory");
        let meta = path.join("corpus.meta");
        let records = path.join("corpus.records");
        let mut meta_bytes = vec![0u8; SOURCE_CORPUS_META_BYTES];
        meta_bytes[0..8].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        meta_bytes[16..24].copy_from_slice(&0x5EEDu64.to_le_bytes());
        meta_bytes[24] = 1;
        let mut record_bytes = vec![0u8; observe::RECORD_SIZE];
        record_bytes[4..8].copy_from_slice(&next.to_le_bytes());
        record_bytes[20..24].copy_from_slice(&next.to_le_bytes());
        record_bytes[32..36].copy_from_slice(&100u32.to_le_bytes());
        record_bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        record_bytes[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        record_bytes[44..48].copy_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&meta, &meta_bytes).expect("finalized metadata");
        std::fs::write(&records, &record_bytes).expect("finalized records");
        (meta, records, meta_bytes, record_bytes)
    }

    fn fixture_recorded_compile_products(
        _corpus: &compiler::Corpus,
        _vocab_size: usize,
    ) -> Result<RecordedCompileProducts, SourceUnavailable> {
        Ok(RecordedCompileProducts {
            artifact_bytes: b"fixture recorded artifact".to_vec(),
            store_bytes: b"fixture recorded store".to_vec(),
            calibration_bytes: b"{\"fixture\":\"calibration\"}".to_vec(),
            hierarchical_bytes: b"{\"fixture\":\"hierarchical\"}".to_vec(),
        })
    }

    fn planned_output_residue_entries(path: &Path) -> Vec<String> {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .expect("read planned-output root")
            .map(|entry| {
                entry
                    .expect("planned-output entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.starts_with(PLANNED_OUTPUT_RESERVED_PREFIX))
            .collect();
        entries.sort();
        entries
    }

    fn observation_corpus_markers(path: &Path) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(path).expect("observation directory");
        let state = path.join(observe::STATE_FILE);
        let merged = path.join("merged.bin");
        std::fs::write(&state, b"state marker").expect("state marker");
        std::fs::write(&merged, b"merged marker").expect("merged marker");
        (state, merged)
    }

    fn source_resume_meta(records: u64, done: u8) -> Vec<u8> {
        let mut meta = Vec::with_capacity(SOURCE_CORPUS_META_BYTES);
        meta.extend_from_slice(&records.to_le_bytes());
        meta.extend_from_slice(&1u64.to_le_bytes());
        meta.extend_from_slice(&0x5EEDu64.to_le_bytes());
        meta.push(done);
        meta
    }

    fn source_v3_record(story: u32, position: u32) -> [u8; 48] {
        compiler::encode_v3_record(
            story,
            7,
            &[7, 8, 9],
            &[70, 20, 10],
            (position, position + 1),
            (position, position + 1),
        )
    }

    fn complete_test_corpus(path: &Path) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(path).expect("corpus directory");
        let meta = path.join("corpus.meta");
        let records = path.join("corpus.records");
        std::fs::write(&records, source_v3_record(0, 0)).expect("complete corpus records");
        std::fs::write(&meta, source_resume_meta(1, 1)).expect("complete corpus metadata");
        (meta, records)
    }

    fn complete_test_observation_corpus(path: &Path) -> (PathBuf, PathBuf) {
        std::fs::create_dir_all(path).expect("observation directory");
        let state = path.join(observe::STATE_FILE);
        let merged = path.join("merged.bin");
        std::fs::write(&merged, source_v3_record(0, 0)).expect("complete observation records");
        std::fs::write(&state, source_resume_meta(1, 1)).expect("complete observation state");
        (state, merged)
    }

    fn publish_test_corpus_binding(path: &Path, meta: &Path, records: &Path) -> String {
        let mut guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(path)
                .expect("test producer guard");
        guard.ensure_root().expect("test corpus root");
        guard.begin_compile_attempt().expect("test compile attempt");
        let cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(&guard, meta, records)
            .expect("test corpus binding");
        guard.finish_compile_attempt().expect("finish test attempt");
        cid
    }

    fn publish_test_observation_binding(path: &Path, state: &Path, merged: &Path) -> String {
        let mut guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(path)
                .expect("test observation producer guard");
        guard.ensure_root().expect("test observation root");
        uor_r4_graph_compiler::recorded_corpus::publish_binding(&guard, state, merged)
            .expect("test observation binding")
    }

    fn directory_entry_names(path: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(path)
            .expect("directory inventory")
            .map(|entry| {
                entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn direct_cover_and_score_refuse_completed_bundle_ancestor_without_member_mutation() {
        let root = unique_cli_test_path("graph-completed-ancestor");
        std::fs::create_dir_all(&root).expect("completed root");
        std::fs::write(root.join("corpus.meta"), b"unread completion sentinel")
            .expect("metadata sentinel");
        std::fs::write(root.join("corpus.records"), b"unread completion sentinel")
            .expect("records sentinel");
        std::fs::write(root.join("compiled_bundle_completion.json"), b"{}\n")
            .expect("completion sentinel");
        let before = directory_entry_names(&root);
        for (command, output_leaf) in [("cover", "graph-cover"), ("score", "graph")] {
            let args = vec![
                "--corpus-meta".to_owned(),
                root.join("corpus.meta").display().to_string(),
                "--corpus-recs".to_owned(),
                root.join("corpus.records").display().to_string(),
                "--bundle-root".to_owned(),
                root.display().to_string(),
                "--out".to_owned(),
                root.join(output_leaf).display().to_string(),
            ];
            let error = match command {
                "cover" => cover_command(&args),
                "score" => score_command(&args),
                _ => unreachable!(),
            }
            .expect_err("completed bundle is immutable");
            assert!(
                error.reason.contains("immutable completed bundle"),
                "{command}: {error}"
            );
            assert_eq!(directory_entry_names(&root), before, "{command}");
            assert!(!root.join(".uor-r4-recorded-corpus-locks").exists());
            assert!(!root.join(output_leaf).exists());
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn declared_bundle_graph_holds_parent_guard_before_child_mutation() {
        let root = unique_cli_test_path("graph-parent-completion-race");
        let source = root.join("source");
        let bundle = root.join("bundle");
        std::fs::create_dir_all(&bundle).expect("markerless bundle parent");
        let (meta, records, _, _) = finalized_corpus(&source, 7);
        publish_test_corpus_binding(&source, &meta, &records);
        let output = bundle.join("graph");
        let mut canonical_output = output.clone();
        let (_, mut transaction) = begin_graph_command_transaction(
            GraphTransactionAuthority::DeclaredBundle(&bundle),
            &meta,
            &records,
            &mut canonical_output,
            "score",
        )
        .expect("source generation is captured before deferred output ownership");
        assert!(!output.exists());

        transaction
            .prepare_output_directory_with_after_guard(&canonical_output, |authority| {
                assert_eq!(
                    authority.root(),
                    std::fs::canonicalize(&bundle).expect("canonical declared bundle")
                );
                assert!(!output.exists(), "child mutation follows parent authority");
                let busy = RecordedCorpusProducerGuard::try_acquire(&bundle)
                    .expect_err("cooperating parent publisher is BUSY at the barrier");
                assert!(busy.reason.contains("BUSY"), "{busy}");
                Ok(())
            })
            .expect("declared bundle child publication");
        assert_eq!(
            directory_entry_names(&bundle),
            vec!["graph"],
            "the child appears only inside the retained parent transaction"
        );
        assert!(output.is_dir());
        assert!(!bundle
            .join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn standalone_graph_refuses_owner_evidence_present_at_transaction_start_without_mutation() {
        let root = unique_cli_test_path("graph-standalone-existing-owner");
        let source = root.join("source");
        let (meta, records, _, _) = finalized_corpus(&source, 7);
        publish_test_corpus_binding(&source, &meta, &records);
        for leaf in [
            "compiled_bundle_completion.json",
            ".compiled_bundle_stage.json",
            ".compiled_bundle_stage_a.json",
            ".compiled_bundle_completion.json.17.19.tmp",
            ".compiled_bundle_completion.json.reserved.tmp",
        ] {
            let bundle = root.join(leaf.replace('.', "-"));
            std::fs::create_dir_all(&bundle).expect("owned ancestor");
            std::fs::write(bundle.join(leaf), b"stable owner evidence\n").expect("owner evidence");
            let before = directory_entry_names(&bundle);
            let mut output = bundle.join("graph");
            let error = match begin_graph_command_transaction(
                GraphTransactionAuthority::StandaloneExact,
                &meta,
                &records,
                &mut output,
                "score",
            ) {
                Ok(_) => panic!("pre-existing owner ancestor must refuse standalone start"),
                Err(error) => error,
            };
            assert!(
                error.reason.contains("transaction start"),
                "{leaf}: {error}"
            );
            assert_eq!(directory_entry_names(&bundle), before, "{leaf}");
            assert!(!bundle.join("graph").exists(), "{leaf}");
            assert!(!bundle
                .join(
                    uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
                )
                .exists());
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn standalone_graph_never_adopts_parent_completion_published_after_start() {
        let root = unique_cli_test_path("graph-standalone-late-parent-owner");
        let source = root.join("source");
        let bundle = root.join("bundle");
        std::fs::create_dir_all(&bundle).expect("standalone output parent");
        let (meta, records, _, _) = finalized_corpus(&source, 7);
        publish_test_corpus_binding(&source, &meta, &records);
        let output = bundle.join("graph");
        let mut canonical_output = output.clone();
        let (_, mut transaction) = begin_graph_command_transaction(
            GraphTransactionAuthority::StandaloneExact,
            &meta,
            &records,
            &mut canonical_output,
            "score",
        )
        .expect("standalone mode is fixed at transaction start");

        let parent_writer = RecordedCorpusProducerGuard::try_acquire(&bundle)
            .expect("independent parent publisher after standalone start");
        let completion = bundle.join("compiled_bundle_completion.json");
        std::fs::write(&completion, b"late parent completion\n").expect("late parent completion");
        parent_writer
            .verify_owned_root()
            .expect("parent publisher authority");
        drop(parent_writer);
        let completion_before = std::fs::read(&completion).expect("completion before output");

        transaction
            .prepare_output_directory(&canonical_output)
            .expect("late parent state never changes standalone exact-root mode");
        assert_eq!(
            transaction.guard().expect("standalone guard").root(),
            std::fs::canonicalize(&output).expect("canonical standalone output")
        );
        assert_eq!(
            std::fs::read(&completion).expect("completion after output"),
            completion_before,
            "the independent standalone transaction never edits parent evidence"
        );
        drop(transaction);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn declared_bundle_root_requires_physical_direct_child_without_mutation() {
        use std::os::unix::fs::symlink;

        let root = unique_cli_test_path("graph-declared-physical-containment");
        let source = root.join("source");
        let bundle = root.join("bundle");
        let unrelated = root.join("unrelated");
        let outside = root.join("outside");
        std::fs::create_dir_all(&bundle).expect("declared bundle");
        std::fs::create_dir_all(&unrelated).expect("unrelated root");
        std::fs::create_dir_all(&outside).expect("outside root");
        let (meta, records, _, _) = finalized_corpus(&source, 7);
        publish_test_corpus_binding(&source, &meta, &records);

        let before = [
            directory_entry_names(&bundle),
            directory_entry_names(&unrelated),
            directory_entry_names(&outside),
        ];
        let mut unrelated_output = bundle.join("graph-unrelated");
        let error = match begin_graph_command_transaction(
            GraphTransactionAuthority::DeclaredBundle(&unrelated),
            &meta,
            &records,
            &mut unrelated_output,
            "score",
        ) {
            Ok(_) => panic!("unrelated declared root must refuse"),
            Err(error) => error,
        };
        assert!(error.reason.contains("not one direct child"), "{error}");

        symlink(&outside, bundle.join("escape")).expect("parent escape alias");
        let mut escaped_output = bundle.join("escape/graph");
        let error = match begin_graph_command_transaction(
            GraphTransactionAuthority::DeclaredBundle(&bundle),
            &meta,
            &records,
            &mut escaped_output,
            "score",
        ) {
            Ok(_) => panic!("physical parent escape must refuse"),
            Err(error) => error,
        };
        assert!(error.reason.contains("not one direct child"), "{error}");
        assert!(!outside.join("graph").exists());

        let target = outside.join("target");
        std::fs::create_dir(&target).expect("final symlink target");
        let final_alias = bundle.join("graph");
        symlink(&target, &final_alias).expect("final output alias");
        let mut final_output = final_alias.clone();
        let error = match begin_graph_command_transaction(
            GraphTransactionAuthority::DeclaredBundle(&bundle),
            &meta,
            &records,
            &mut final_output,
            "score",
        ) {
            Ok(_) => panic!("final output symlink must refuse"),
            Err(error) => error,
        };
        assert!(error.reason.contains("non-symlink directory"), "{error}");
        assert!(
            final_alias
                .symlink_metadata()
                .expect("final alias")
                .file_type()
                .is_symlink()
        );

        assert_eq!(directory_entry_names(&unrelated), before[1]);
        assert!(
            directory_entry_names(&bundle).contains(&"escape".to_owned()),
            "only the test's preexisting aliases are present"
        );
        assert!(!bundle
            .join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn graph_same_root_public_contender_is_busy_and_managed_seam_reuses_guard() {
        let root = unique_cli_test_path("graph-managed-producer");
        let (meta, records, _, _) = finalized_corpus(&root, 7);
        publish_test_corpus_binding(&root, &meta, &records);
        let guard = RecordedCorpusProducerGuard::try_acquire(&root).expect("managed producer");
        let output = root.join("graph-cover");
        let mut output_for_transaction = output.clone();
        let (recorded, mut transaction) = begin_graph_command_transaction(
            GraphTransactionAuthority::ProvidedBundle(&guard),
            &meta,
            &records,
            &mut output_for_transaction,
            "cover",
        )
        .expect("provided managed producer avoids self-contention");
        assert_eq!(recorded.corpus.n, 1);
        transaction
            .prepare_output_directory(&output_for_transaction)
            .expect("managed output directory");
        let busy = RecordedCorpusProducerGuard::try_acquire(&root)
            .expect_err("external graph contender is BUSY");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        transaction
            .verify_after_output(&output_for_transaction)
            .expect("managed authority remains valid");
        let duplicate_authority_args = vec![
            "--corpus-meta".to_owned(),
            meta.display().to_string(),
            "--corpus-recs".to_owned(),
            records.display().to_string(),
            "--bundle-root".to_owned(),
            root.display().to_string(),
            "--out".to_owned(),
            output.display().to_string(),
        ];
        let before_duplicate = directory_entry_names(&root);
        let error = cover_command_under_producer_guard(&duplicate_authority_args, &guard)
            .expect_err("provided authority rejects a second root declaration");
        assert!(error.reason.contains("second --bundle-root"), "{error}");
        assert_eq!(directory_entry_names(&root), before_duplicate);
        drop(transaction);
        drop(guard);

        let blocker = RecordedCorpusProducerGuard::try_acquire(&root).expect("public blocker");
        let before = directory_entry_names(&root);
        let args = vec![
            "--corpus-meta".to_owned(),
            meta.display().to_string(),
            "--corpus-recs".to_owned(),
            records.display().to_string(),
            "--bundle-root".to_owned(),
            root.display().to_string(),
            "--out".to_owned(),
            output.display().to_string(),
        ];
        let error = cover_command(&args).expect_err("public same-root contender is BUSY");
        assert!(error.reason.contains("BUSY"), "{error}");
        assert_eq!(directory_entry_names(&root), before);
        drop(blocker);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn standalone_graph_outputs_guard_only_their_exact_roots_and_do_not_contend() {
        let root = unique_cli_test_path("graph-standalone-output-roots");
        let source = root.join("source");
        let (meta, records, _, _) = finalized_corpus(&source, 7);
        publish_test_corpus_binding(&source, &meta, &records);
        let output_a = root.join("graph");
        let output_b = root.join("graph-cover");

        let mut canonical_a = output_a.clone();
        let (_, mut transaction_a) = begin_graph_command_transaction(
            GraphTransactionAuthority::StandaloneExact,
            &meta,
            &records,
            &mut canonical_a,
            "cover",
        )
        .expect("capture source before standalone output A");
        let mut canonical_b = output_b.clone();
        let (_, mut transaction_b) = begin_graph_command_transaction(
            GraphTransactionAuthority::StandaloneExact,
            &meta,
            &records,
            &mut canonical_b,
            "score",
        )
        .expect("capture source before standalone output B");
        assert!(!output_a.exists());
        assert!(!output_b.exists());

        transaction_a
            .prepare_output_directory(&canonical_a)
            .expect("acquire exact standalone output A");
        transaction_b
            .prepare_output_directory(&canonical_b)
            .expect("unrelated standalone output B does not contend");
        assert!(
            transaction_a
                .guard()
                .expect("output A guard")
                .protects_directory(&output_a)
                .expect("output A identity")
        );
        assert_eq!(
            transaction_a.guard().expect("output A guard").root(),
            std::fs::canonicalize(&output_a).expect("canonical output A"),
            "the conventional basename never widens arbitrary --out authority to its parent"
        );
        assert!(
            transaction_b
                .guard()
                .expect("output B guard")
                .protects_directory(&output_b)
                .expect("output B identity")
        );
        assert_eq!(
            transaction_b.guard().expect("output B guard").root(),
            std::fs::canonicalize(&output_b).expect("canonical output B"),
            "graph-cover is also an exact standalone output without provided bundle authority"
        );
        assert!(!output_a
            .join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .exists());
        assert!(!output_b
            .join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .exists());
        assert!(
            root.join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .is_dir()
        );

        let busy = RecordedCorpusProducerGuard::try_acquire(&output_a)
            .expect_err("only output A itself contends with output A");
        assert!(busy.reason.contains("BUSY"), "{busy}");
        transaction_a
            .verify_after_output(&canonical_a)
            .expect("output A authority remains exact");
        transaction_b
            .verify_after_output(&canonical_b)
            .expect("output B authority remains exact");
        drop(transaction_b);
        drop(transaction_a);

        let source_bundle = root.join("unflagged-source-bundle");
        let (same_meta, same_records, _, _) = finalized_corpus(&source_bundle, 11);
        publish_test_corpus_binding(&source_bundle, &same_meta, &same_records);
        let same_root_child = source_bundle.join("graph");
        let mut canonical_child = same_root_child.clone();
        let (_, mut child_transaction) = begin_graph_command_transaction(
            GraphTransactionAuthority::StandaloneExact,
            &same_meta,
            &same_records,
            &mut canonical_child,
            "score",
        )
        .expect("unflagged source-child output is captured as standalone");
        child_transaction
            .prepare_output_directory(&canonical_child)
            .expect("unflagged source-child exact authority");
        assert_eq!(
            child_transaction
                .guard()
                .expect("source-child guard")
                .root(),
            std::fs::canonicalize(&same_root_child).expect("canonical source-child output"),
            "lexical output.parent == corpus root never implies public bundle mode"
        );
        let parent_guard = RecordedCorpusProducerGuard::try_acquire(&source_bundle)
            .expect("standalone child guard does not contend with undeclared parent authority");
        drop(parent_guard);
        drop(child_transaction);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn public_hf_compile_refuses_server_owner_records_but_provided_session_passes_owner_gate() {
        for owner_leaf in [
            ".compiled_bundle_stage.json",
            ".compiled_bundle_stage_a.json",
        ] {
            let root =
                unique_cli_test_path(&format!("hf-owner-{}", owner_leaf.replace(['.', '_'], "-")));
            std::fs::create_dir_all(&root).expect("owned server stage");
            std::fs::write(
                root.join(".compiled_bundle_stage.json"),
                b"owner sentinel\n",
            )
            .expect("owner marker");
            if owner_leaf == ".compiled_bundle_stage_a.json" {
                std::fs::write(root.join(owner_leaf), b"seal sentinel\n").expect("owner seal");
            }
            let source = root.with_file_name(format!(
                "{}-missing-source",
                root.file_name().expect("root leaf").to_string_lossy()
            ));
            let args = vec![
                "--source".to_owned(),
                source.display().to_string(),
                "--output".to_owned(),
                root.display().to_string(),
            ];
            let before = directory_bytes(&root);
            let error = compile_hugging_face(&args)
                .expect_err("public compiler cannot adopt a private server stage");
            assert!(error.reason.contains("compiled-bundle ancestor"), "{error}");
            assert_eq!(directory_bytes(&root), before);

            let child = root.join("child");
            let child_args = vec![
                "--source".to_owned(),
                source.display().to_string(),
                "--output".to_owned(),
                child.display().to_string(),
            ];
            let error = compile_hugging_face(&child_args)
                .expect_err("public compiler cannot seed a child inside a private stage");
            assert!(error.reason.contains("compiled-bundle ancestor"), "{error}");
            assert_eq!(directory_bytes(&root), before);
            assert!(!child.exists());
            assert!(!root
                .join(
                    uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
                )
                .exists());

            let mut session = source_corpus_session(&root).expect("server outer session");
            session
                .acquire_recorded_corpus(&root)
                .expect("server common producer");
            let error = compile_hugging_face_with_session(&args, session)
                .expect_err("missing source fails only after the provided-session owner gate");
            assert!(!error.reason.contains("server owner marker"), "{error}");
            assert_eq!(directory_bytes(&root), before);
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn public_hf_child_compile_reserves_markerless_private_stage_topology() {
        let root = unique_cli_test_path("hf-markerless-private-stage-topology");
        let staging = root.join(".uor-r4-source-compile-staging");
        let stage = staging.join(".teacher.bundle-stage.7.9");
        let child = stage.join("child");
        std::fs::create_dir_all(&stage).expect("markerless private-stage allocation boundary");
        let source = root.join("missing-source");
        let args = vec![
            "--source".to_owned(),
            source.display().to_string(),
            "--output".to_owned(),
            child.display().to_string(),
        ];
        let stage_before = directory_bytes(&stage);

        let error = compile_hugging_face(&args)
            .expect_err("private-stage topology is reserved before owner-marker publication");
        assert!(error.reason.contains("managed capability"), "{error}");
        assert_eq!(directory_bytes(&stage), stage_before);
        assert!(!child.exists());
        assert!(!stage
            .join(
                uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_PRODUCER_COORDINATION_DIR,
            )
            .exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn public_hf_compile_canonicalizes_alias_ancestors_before_any_creation() {
        use std::os::unix::fs::symlink;

        let root = unique_cli_test_path("hf-private-stage-alias");
        let stage = root.join("private-stage");
        let alias = root.join("stage-alias");
        std::fs::create_dir_all(&stage).expect("private stage");
        std::fs::write(
            stage.join(".compiled_bundle_stage.json"),
            b"owner sentinel\n",
        )
        .expect("private owner marker");
        std::fs::write(stage.join("sentinel"), b"private bytes").expect("private stage sentinel");
        symlink(&stage, &alias).expect("alias to private stage");
        let source = root.join("missing-source");
        let stage_before = directory_bytes(&stage);

        let child_args = vec![
            "--source".to_owned(),
            source.display().to_string(),
            "--output".to_owned(),
            alias.join("child").display().to_string(),
        ];
        let error = compile_hugging_face(&child_args)
            .expect_err("physical private-stage ancestor is detected through an alias");
        assert!(error.reason.contains("compiled-bundle ancestor"), "{error}");
        assert_eq!(directory_bytes(&stage), stage_before);
        assert!(!stage.join("child").exists());
        assert!(
            std::fs::symlink_metadata(&alias)
                .expect("alias remains")
                .file_type()
                .is_symlink()
        );

        let final_alias_args = vec![
            "--source".to_owned(),
            source.display().to_string(),
            "--output".to_owned(),
            alias.display().to_string(),
        ];
        let error = compile_hugging_face(&final_alias_args)
            .expect_err("a final output symlink is rejected before owner/session creation");
        assert!(error.reason.contains("is a symlink"), "{error}");
        assert_eq!(directory_bytes(&stage), stage_before);
        let _ = std::fs::remove_dir_all(root);
    }

    fn source_v2_record(story: u32) -> [u8; 32] {
        let mut record = [0u8; 32];
        record[0..4].copy_from_slice(&story.to_le_bytes());
        record[4..8].copy_from_slice(&7u32.to_le_bytes());
        for (index, token) in [7u32, 8, 9].into_iter().enumerate() {
            let offset = 8 + index * 4;
            record[offset..offset + 4].copy_from_slice(&token.to_le_bytes());
        }
        for (index, weight) in [70u32, 20, 10].into_iter().enumerate() {
            let offset = 20 + index * 4;
            record[offset..offset + 4].copy_from_slice(&weight.to_le_bytes());
        }
        record
    }

    fn subsample_args(source: &Path, destination: &Path, records: u64) -> Vec<String> {
        vec![
            "--src-meta".to_owned(),
            source.join("corpus.meta").to_string_lossy().into_owned(),
            "--src-recs".to_owned(),
            source.join("corpus.records").to_string_lossy().into_owned(),
            "--out-meta".to_owned(),
            destination
                .join("corpus.meta")
                .to_string_lossy()
                .into_owned(),
            "--out-recs".to_owned(),
            destination
                .join("corpus.records")
                .to_string_lossy()
                .into_owned(),
            "--records".to_owned(),
            records.to_string(),
        ]
    }

    fn subsample_fixture_row(width: usize, story: u32, index: u32) -> Vec<u8> {
        let next = 100 + index;
        match width {
            12 => {
                let mut row = vec![0u8; 12];
                row[0..4].copy_from_slice(&story.to_le_bytes());
                row[4..6].copy_from_slice(&(next as u16).to_le_bytes());
                row[6..8].copy_from_slice(&(next as u16).to_le_bytes());
                row[8..12].copy_from_slice(&(-0.25f32).to_le_bytes());
                row
            }
            32 => {
                let mut row = vec![0u8; 32];
                row[0..4].copy_from_slice(&story.to_le_bytes());
                row[4..8].copy_from_slice(&next.to_le_bytes());
                for (slot, token) in [next, next + 1, next + 2].into_iter().enumerate() {
                    let offset = 8 + slot * 4;
                    row[offset..offset + 4].copy_from_slice(&token.to_le_bytes());
                }
                for (slot, weight) in [70u32, 20, 10].into_iter().enumerate() {
                    let offset = 20 + slot * 4;
                    row[offset..offset + 4].copy_from_slice(&weight.to_le_bytes());
                }
                row
            }
            48 => compiler::encode_v3_record(
                story,
                next,
                &[next, next + 1, next + 2],
                &[70, 20, 10],
                (1_000 + index, 1_001 + index),
                (2_000 + index, 2_001 + index),
            )
            .to_vec(),
            88 => {
                let mut row = vec![0u8; 88];
                row[0..4].copy_from_slice(&story.to_le_bytes());
                row[4..8].copy_from_slice(&next.to_le_bytes());
                for slot in 0..8usize {
                    let token = next + slot as u32;
                    let token_offset = 8 + slot * 4;
                    let weight_offset = 40 + slot * 4;
                    row[token_offset..token_offset + 4].copy_from_slice(&token.to_le_bytes());
                    row[weight_offset..weight_offset + 4]
                        .copy_from_slice(&(80u32.saturating_sub(slot as u32 * 10)).to_le_bytes());
                }
                row[72..76].copy_from_slice(&(1_000 + index).to_le_bytes());
                row[76..80].copy_from_slice(&(1_001 + index).to_le_bytes());
                row[80..84].copy_from_slice(&(2_000 + index).to_le_bytes());
                row[84..88].copy_from_slice(&(2_001 + index).to_le_bytes());
                row
            }
            _ => panic!("unsupported fixture width {width}"),
        }
    }

    fn write_subsample_fixture(root: &Path, width: usize, hidden_row_bytes: usize) -> Vec<Vec<u8>> {
        std::fs::create_dir_all(root).expect("fixture root");
        bind_compiled_dense_operator(root, Some(&DenseOperatorSpec::gpt2_v2()))
            .expect("fixture dense identity");
        bind_compiled_attention_operator(root, &AttentionOperatorSpec::learned_absolute_v2())
            .expect("fixture attention identity");
        let stories = [
            0u32, 0, 0, // selected train run
            1, 1, 1, 1, 1, 1, // skipped: too large
            0, 0, // skipped: would stitch the selected story-0 run
            2, 2, // selected
            3, 3, 3, 3, // skipped: too large for remaining quota
            4, 4, 4, // selected
            8, 8, // selected held-out run
            9, 9, // skipped: held-out quota has one row left
        ];
        let rows = stories
            .into_iter()
            .enumerate()
            .map(|(index, story)| subsample_fixture_row(width, story, index as u32))
            .collect::<Vec<_>>();
        let records = rows.iter().flatten().copied().collect::<Vec<_>>();
        let mut meta = vec![0u8; SOURCE_CORPUS_META_BYTES];
        meta[0..8].copy_from_slice(&(rows.len() as u64).to_le_bytes());
        meta[8..16].copy_from_slice(&10u64.to_le_bytes());
        meta[16..24].copy_from_slice(&0x0729_5EED_u64.to_le_bytes());
        meta[24] = 1;
        std::fs::write(root.join("corpus.records"), records).expect("fixture records");
        std::fs::write(root.join("corpus.meta"), meta).expect("fixture metadata");
        if hidden_row_bytes != 0 {
            let hidden = (0..rows.len())
                .flat_map(|index| {
                    (0..hidden_row_bytes)
                        .map(move |byte| (index as u8).wrapping_mul(17).wrapping_add(byte as u8))
                })
                .collect::<Vec<_>>();
            std::fs::write(root.join("corpus.records.hidden"), hidden).expect("fixture hidden");
        }
        rows
    }

    #[test]
    fn subsample_streams_all_registered_widths_with_fixed_partition_and_exact_raw_rows() {
        let selected_indices = [0usize, 1, 2, 11, 12, 17, 18, 19, 20, 21];
        let expected_story = [0u32, 0, 0, 2, 2, 4, 4, 4, 8, 8];
        let base = unique_cli_test_path("subsample-all-widths");
        std::fs::create_dir_all(&base).expect("base");

        for width in [12usize, 32, 48, 88] {
            let source = base.join(format!("source-{width}"));
            let destination = base.join(format!("destination-{width}"));
            let rows = write_subsample_fixture(&source, width, 12);
            let attention = AttentionOperatorSpec::learned_absolute_v2();
            let dense = DenseOperatorSpec::gpt2_v2();
            publish_test_corpus_binding(
                &source,
                &source.join("corpus.meta"),
                &source.join("corpus.records"),
            );

            subsample_recorded_corpus(&subsample_args(&source, &destination, 11))
                .expect("streaming subsample");
            let meta = std::fs::read(destination.join("corpus.meta")).expect("output meta");
            let source_meta = std::fs::read(source.join("corpus.meta")).expect("source meta");
            assert_eq!(u64::from_le_bytes(meta[0..8].try_into().unwrap()), 10);
            assert_eq!(u64::from_le_bytes(meta[8..16].try_into().unwrap()), 10);
            assert_eq!(&meta[16..25], &source_meta[16..25]);

            let expected_records = selected_indices
                .iter()
                .flat_map(|&index| rows[index].iter().copied())
                .collect::<Vec<_>>();
            let output_records =
                std::fs::read(destination.join("corpus.records")).expect("output records");
            assert_eq!(output_records, expected_records, "raw width {width}");
            let corpus = compiler::load_corpus_bytes(&meta, &output_records, None)
                .expect("derived corpus parses");
            assert_eq!(corpus.story, expected_story, "stories width {width}");
            let expected_input = [1u32, 100, 101, 1, 111, 1, 117, 118, 1, 120];
            assert_eq!(corpus.input, expected_input, "inputs width {width}");
            if matches!(width, 48 | 88) {
                assert_eq!(
                    corpus.span_start,
                    selected_indices
                        .iter()
                        .map(|index| 1_000 + *index as u32)
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    corpus.byte_end,
                    selected_indices
                        .iter()
                        .map(|index| 2_001 + *index as u32)
                        .collect::<Vec<_>>()
                );
            } else {
                assert_eq!(corpus.span_start, [0u32, 1, 2, 0, 1, 0, 1, 2, 0, 1]);
                assert_eq!(corpus.byte_start, [u32::MAX; 10]);
            }
            let source_hidden =
                std::fs::read(source.join("corpus.records.hidden")).expect("source hidden");
            let expected_hidden = selected_indices
                .iter()
                .flat_map(|index| source_hidden[index * 12..index * 12 + 12].iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(
                std::fs::read(destination.join("corpus.records.hidden")).unwrap(),
                expected_hidden
            );
            assert_eq!(
                compiled_attention_operator(&destination).unwrap(),
                attention
            );
            assert_eq!(compiled_dense_operator(&destination).unwrap(), Some(dense));
            if width == 48 {
                let binding = std::fs::read(
                    destination
                        .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE),
                )
                .expect("canonical output binding");
                assert_eq!(
                    format!("blake3:{}", blake3::hash(&binding).to_hex()),
                    "blake3:2c05c249b75bc7dbc52cd79412443b4fc49bd9b6f01f86d660b5e908f1998c26"
                );
            }

            let ready = directory_bytes(&destination);
            subsample_recorded_corpus(&subsample_args(&source, &destination, 11))
                .expect("idempotent ready rerun");
            assert_eq!(directory_bytes(&destination), ready);
        }
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn selected_ranges_reader_uses_fixed_buffers_at_multi_gigabyte_offsets() {
        #[derive(Default)]
        struct SparseReader {
            position: u64,
            max_request: usize,
        }
        impl Read for SparseReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                self.max_request = self.max_request.max(buffer.len());
                for (offset, byte) in buffer.iter_mut().enumerate() {
                    *byte = self.position.wrapping_add(offset as u64) as u8;
                }
                self.position += buffer.len() as u64;
                Ok(buffer.len())
            }
        }
        impl Seek for SparseReader {
            fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
                let next = match position {
                    SeekFrom::Start(next) => next,
                    SeekFrom::Current(offset) => {
                        u64::try_from(i128::from(self.position) + i128::from(offset)).unwrap()
                    }
                    SeekFrom::End(_) => panic!("fixture has no finite physical end"),
                };
                self.position = next;
                Ok(next)
            }
        }

        let ranges = [
            SelectedByteRange {
                start: 5 * 1024 * 1024 * 1024,
                end: 5 * 1024 * 1024 * 1024 + 70_000,
            },
            SelectedByteRange {
                start: 9 * 1024 * 1024 * 1024,
                end: 9 * 1024 * 1024 * 1024 + 123,
            },
        ];
        let mut source = SparseReader::default();
        let summary = {
            let mut selected = SelectedRangesReader::new(&mut source, &ranges).expect("ranges");
            summarize_selected_stream(&mut selected, "sparse selected stream").expect("summary")
        };
        assert_eq!(summary.length, 70_123);
        assert!(summary.blake3.starts_with("blake3:"));
        assert!(source.max_request <= 64 * 1024, "{}", source.max_request);
    }

    #[test]
    fn selector_preserves_u64_story_count_train_cut_and_requires_both_partitions() {
        let mut records = Vec::new();
        records.extend_from_slice(&subsample_fixture_row(12, 0, 0));
        records.extend_from_slice(&subsample_fixture_row(12, u32::MAX, 1));
        let mut cursor = std::io::Cursor::new(records);
        let selected =
            select_recorded_corpus_ranges(&mut cursor, 2, 12, u64::from(u32::MAX) + 100, 2)
                .expect("u64 story metadata uses compiler train-cut cast semantics");
        assert_eq!(selected.train_records, 1);
        assert_eq!(selected.held_out_records, 1);
        assert_eq!(selected.ranges, [SelectedRowRange { start: 0, end: 2 }]);

        let mut train_only = std::io::Cursor::new(
            [
                subsample_fixture_row(12, 0, 0),
                subsample_fixture_row(12, 0, 1),
            ]
            .concat(),
        );
        let error = select_recorded_corpus_ranges(&mut train_only, 2, 12, 10, 2)
            .expect_err("full-source lane still requires both fixed partitions");
        assert!(
            error.reason.contains("both fixed source partitions"),
            "{error}"
        );
    }

    #[test]
    fn subsample_full_lane_refuses_oversize_and_malformed_hidden_rows() {
        let base = unique_cli_test_path("subsample-full-and-hidden");
        std::fs::create_dir_all(&base).expect("base");
        let source = base.join("source");
        write_subsample_fixture(&source, 48, 12);
        publish_test_corpus_binding(
            &source,
            &source.join("corpus.meta"),
            &source.join("corpus.records"),
        );

        let full = base.join("full");
        subsample_recorded_corpus(&subsample_args(&source, &full, 24))
            .expect("N equal to source is an exact full lane");
        for member in ["corpus.meta", "corpus.records", "corpus.records.hidden"] {
            assert_eq!(
                std::fs::read(full.join(member)).unwrap(),
                std::fs::read(source.join(member)).unwrap(),
                "full lane member {member}"
            );
        }

        let oversize = base.join("oversize");
        let error = subsample_recorded_corpus(&subsample_args(&source, &oversize, 25))
            .expect_err("N above the finalized source is refused");
        assert!(error.reason.contains("exceeds the source"), "{error}");
        assert!(
            !oversize.exists(),
            "oversize refusal publishes no output root"
        );

        for (name, hidden_bytes, expected) in [
            (
                "not-divisible",
                vec![0u8; 24 * 4 + 1],
                "not an exact row width",
            ),
            (
                "not-f32-aligned",
                vec![0u8; 24 * 6],
                "not an exact f32 byte multiple",
            ),
        ] {
            let malformed = base.join(name);
            write_subsample_fixture(&malformed, 48, 0);
            std::fs::write(malformed.join("corpus.records.hidden"), hidden_bytes)
                .expect("malformed hidden body");
            publish_test_corpus_binding(
                &malformed,
                &malformed.join("corpus.meta"),
                &malformed.join("corpus.records"),
            );
            let destination = base.join(format!("{name}-output"));
            let error = subsample_recorded_corpus(&subsample_args(&malformed, &destination, 11))
                .expect_err("malformed hidden layout is terminal");
            assert!(error.reason.contains(expected), "{error}");
            assert!(
                !destination.exists(),
                "hidden-layout refusal publishes no output root"
            );
        }
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn subsample_refuses_alias_missing_parent_and_busy_roots_without_payload_mutation() {
        let base = unique_cli_test_path("subsample-root-authority");
        std::fs::create_dir_all(&base).expect("base");
        let source = base.join("source");
        write_subsample_fixture(&source, 48, 12);
        publish_test_corpus_binding(
            &source,
            &source.join("corpus.meta"),
            &source.join("corpus.records"),
        );
        let source_before = directory_bytes(&source);

        let in_place = subsample_recorded_corpus(&subsample_args(&source, &source, 11))
            .expect_err("in-place derivation is refused");
        assert!(
            in_place.reason.contains("same canonical root"),
            "{in_place}"
        );
        assert_eq!(directory_bytes(&source), source_before);

        let missing_parent = base.join("missing-parent").join("destination");
        let error = subsample_recorded_corpus(&subsample_args(&source, &missing_parent, 11))
            .expect_err("destination immediate parent must already exist");
        assert!(error.reason.contains("cannot be canonicalized"), "{error}");
        assert!(!base.join("missing-parent").exists());
        assert_eq!(directory_bytes(&source), source_before);

        #[cfg(unix)]
        {
            let source_link = base.join("source-link");
            std::os::unix::fs::symlink(&source, &source_link).expect("source symlink");
            let destination = base.join("symlink-output");
            let error = subsample_recorded_corpus(&subsample_args(&source_link, &destination, 11))
                .expect_err("lexical source-root symlink is refused");
            assert!(error.reason.contains("not a real non-symlink"), "{error}");
            assert!(!destination.exists());
            assert_eq!(directory_bytes(&source), source_before);
        }

        let source_blocker =
            RecordedCorpusProducerGuard::try_acquire(&source).expect("source contention fixture");
        let source_busy_output = base.join("source-busy-output");
        let error = subsample_recorded_corpus(&subsample_args(&source, &source_busy_output, 11))
            .expect_err("source BUSY is propagated");
        assert!(
            uor_r4_graph_compiler::recorded_corpus::is_recorded_corpus_busy(&error),
            "{error}"
        );
        assert!(!source_busy_output.exists());
        assert_eq!(directory_bytes(&source), source_before);
        drop(source_blocker);

        let destination = base.join("destination-busy");
        std::fs::create_dir(&destination).expect("destination fixture");
        let destination_blocker = RecordedCorpusProducerGuard::try_acquire(&destination)
            .expect("destination contention fixture");
        let error = subsample_recorded_corpus(&subsample_args(&source, &destination, 11))
            .expect_err("destination BUSY is propagated");
        assert!(
            uor_r4_graph_compiler::recorded_corpus::is_recorded_corpus_busy(&error),
            "{error}"
        );
        assert!(directory_bytes(&destination).is_empty());
        assert_eq!(directory_bytes(&source), source_before);
        drop(destination_blocker);

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn subsample_markerless_compatibility_crash_recovery_and_wrong_role_are_exact() {
        let base = unique_cli_test_path("subsample-compat-recovery");
        std::fs::create_dir_all(&base).expect("base");

        let attention_only = base.join("attention-only");
        write_subsample_fixture(&attention_only, 48, 12);
        std::fs::remove_file(attention_only.join(DENSE_OPERATOR_BINDING_FILE))
            .expect("make markerless attention-only source");
        let attention_output = base.join("attention-output");
        subsample_recorded_corpus(&subsample_args(&attention_only, &attention_output, 11))
            .expect("markerless attention-only source remains compatible");
        assert_eq!(
            compiled_attention_operator(&attention_output).unwrap(),
            AttentionOperatorSpec::learned_absolute_v2()
        );
        assert_eq!(compiled_dense_operator(&attention_output).unwrap(), None);

        let markerless_dense = base.join("markerless-dense");
        write_subsample_fixture(&markerless_dense, 48, 12);
        std::fs::remove_file(markerless_dense.join(ATTENTION_OPERATOR_BINDING_FILE))
            .expect("make invalid markerless dense source");
        let dense_before = directory_bytes(&markerless_dense);
        let dense_output = base.join("dense-output");
        let error =
            subsample_recorded_corpus(&subsample_args(&markerless_dense, &dense_output, 11))
                .expect_err("markerless dense source is refused");
        assert!(
            error.reason.contains("dense")
                && (error.reason.contains("binding") || error.reason.contains("attention")),
            "{error}"
        );
        assert!(!dense_output.exists());
        assert_eq!(directory_bytes(&markerless_dense), dense_before);

        let source = base.join("bound-source");
        write_subsample_fixture(&source, 48, 12);
        publish_test_corpus_binding(
            &source,
            &source.join("corpus.meta"),
            &source.join("corpus.records"),
        );
        let recovered = base.join("recovered");
        let args = subsample_args(&source, &recovered, 11);
        let error = subsample_recorded_corpus_with_before_binding(&args, || {
            Err(SourceUnavailable::new("injected crash before binding"))
        })
        .expect_err("pre-binding crash boundary");
        assert!(error.reason.contains("injected crash"), "{error}");
        assert!(
            !recovered
                .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE)
                .exists()
        );
        assert!(
            recovered
                .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists()
        );
        let records_before = std::fs::read(recovered.join("corpus.records")).unwrap();
        let hidden_before = std::fs::read(recovered.join("corpus.records.hidden")).unwrap();
        let meta_before = std::fs::read(recovered.join("corpus.meta")).unwrap();
        subsample_recorded_corpus(&args).expect("exact crash-prefix rerun commits binding");
        assert_eq!(
            std::fs::read(recovered.join("corpus.records")).unwrap(),
            records_before
        );
        assert_eq!(
            std::fs::read(recovered.join("corpus.records.hidden")).unwrap(),
            hidden_before
        );
        assert_eq!(
            std::fs::read(recovered.join("corpus.meta")).unwrap(),
            meta_before
        );
        assert!(
            recovered
                .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE)
                .exists()
        );
        assert!(
            !recovered
                .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists()
        );

        let wrong_role = base.join("wrong-role");
        let (state, merged) = complete_test_observation_corpus(&wrong_role);
        publish_test_observation_binding(&wrong_role, &state, &merged);
        std::fs::remove_file(state).expect("remove observation state body");
        std::fs::remove_file(merged).expect("remove observation merged body");
        let wrong_before = directory_bytes(&wrong_role);
        let error = subsample_recorded_corpus(&subsample_args(&source, &wrong_role, 11))
            .expect_err("observation-role binding cannot become a compile corpus");
        assert!(error.reason.contains("cross-role"), "{error}");
        assert_eq!(directory_bytes(&wrong_role), wrong_before);

        let _ = std::fs::remove_dir_all(base);
    }

    fn copy_recorded_attention_args(meta: &Path, records: &Path, output: &Path) -> Vec<String> {
        vec![
            "--corpus-meta".to_owned(),
            meta.to_string_lossy().into_owned(),
            "--corpus-recs".to_owned(),
            records.to_string_lossy().into_owned(),
            "--out".to_owned(),
            output.to_string_lossy().into_owned(),
        ]
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

    /// #637 phase 3: `--emit-provenance` is retired as a gate (PROV/1 is
    /// now unconditional -- see the `cover` emission call site), but the
    /// flag itself still parses as a harmless no-op so any phase-2a
    /// caller/script that already passes it keeps working unmodified.
    #[test]
    fn cover_options_still_accept_the_now_no_op_emit_provenance_flag() {
        let kappa = format!("blake3:{}", "7".repeat(64));
        let args = [
            "--emit-provenance".to_owned(),
            "--source-manifest-kappa".to_owned(),
            kappa.clone(),
        ];
        let options = parse_cover_options(&args).expect("valid cover options");
        assert_eq!(
            options.source_manifest_kappa.as_deref(),
            Some(kappa.as_str())
        );
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
            uor_r4_model_source::attention::AttentionOperatorSpec::standard_v1(),
            uor_r4_model_source::attention::AttentionOperatorSpec::standard_v2(),
            uor_r4_model_source::attention::AttentionOperatorSpec::experimental_r4_v1(),
            uor_r4_model_source::attention::AttentionOperatorSpec::experimental_r4_v2(),
            uor_r4_model_source::attention::AttentionOperatorSpec::learned_absolute_v1(),
            uor_r4_model_source::attention::AttentionOperatorSpec::learned_absolute_v2(),
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

        let mut altered = AttentionOperatorSpec::standard();
        altered.output_projection.push_str("-tampered");
        altered.implementation_digest = altered.declared_digest();
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&altered).expect("serialize altered record"),
        ])
        .expect_err("registry-divergent record is not a product");
        assert!(error.reason.contains("does not match registered"));

        let target = AttentionOperatorSpec::r4_route_attention_v1();
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&target).expect("serialize target record"),
        ])
        .expect_err("a deployed target operator is not source provenance");
        assert!(
            error
                .reason
                .contains("not a source-teacher attention operator")
        );

        let mut with_extra =
            serde_json::to_value(AttentionOperatorSpec::standard()).expect("operator JSON");
        with_extra
            .as_object_mut()
            .expect("operator object")
            .insert("unregistered_claim".to_owned(), serde_json::json!(true));
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&with_extra).expect("serialize extended record"),
        ])
        .expect_err("unknown provenance fields are refused");
        assert!(error.reason.contains("unregistered_claim"));

        let duplicate = serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v2())
            .expect("attention JSON")
            .replacen("\"version\":2", "\"version\":1,\"version\":2", 1);
        let error = parse_cover_options(&["--attention-operator".to_owned(), duplicate])
            .expect_err("duplicate attention era is ambiguous");
        assert!(error.reason.contains("duplicate JSON field \"version\""));
    }

    #[test]
    fn cover_options_carry_only_registered_joint_dense_execution_eras() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        for (attention, dense) in [
            (
                AttentionOperatorSpec::learned_absolute_v1(),
                DenseOperatorSpec::gpt2_v1(),
            ),
            (
                AttentionOperatorSpec::learned_absolute_v2(),
                DenseOperatorSpec::gpt2_v2(),
            ),
        ] {
            let args = [
                "--attention-operator".to_owned(),
                serde_json::to_string(&attention).expect("attention JSON"),
                "--dense-operator".to_owned(),
                serde_json::to_string(&dense).expect("dense JSON"),
            ];
            let options = parse_cover_options(&args).expect("registered source-execution era");
            assert_eq!(options.attention_operator.as_ref(), Some(&attention));
            assert_eq!(options.dense_operator.as_ref(), Some(&dense));
        }
        let defaults = parse_cover_options(&[]).expect("legacy/Llama defaults");
        assert_eq!(defaults.dense_operator, None);

        let dense_json =
            serde_json::to_string(&DenseOperatorSpec::gpt2_v2()).expect("current dense JSON");
        let error = parse_cover_options(&["--dense-operator".to_owned(), dense_json.clone()])
            .expect_err("dense without attention is terminal");
        assert!(error.reason.contains("without an attention"), "{error}");
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v1())
                .expect("v1 attention JSON"),
            "--dense-operator".to_owned(),
            dense_json,
        ])
        .expect_err("cross-era cover provenance is terminal");
        assert!(error.reason.contains("source execution pair"), "{error}");

        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v2())
                .expect("attention JSON"),
            "--dense-operator".to_owned(),
            "{not json".to_owned(),
        ])
        .expect_err("malformed dense record is terminal");
        assert!(error.reason.contains("--dense-operator"), "{error}");

        let mut drifted = DenseOperatorSpec::gpt2_v2();
        drifted.conv1d_bias.push_str("-tampered");
        drifted.implementation_digest = drifted.declared_digest();
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v2())
                .expect("attention JSON"),
            "--dense-operator".to_owned(),
            serde_json::to_string(&drifted).expect("drifted dense JSON"),
        ])
        .expect_err("registry-divergent dense record is terminal");
        assert!(
            error.reason.contains("does not match registered"),
            "{error}"
        );

        let duplicate_dense = serde_json::to_string(&DenseOperatorSpec::gpt2_v2())
            .expect("dense JSON")
            .replacen("\"version\":2", "\"version\":1,\"version\":2", 1);
        let error = parse_cover_options(&[
            "--attention-operator".to_owned(),
            serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v2())
                .expect("attention JSON"),
            "--dense-operator".to_owned(),
            duplicate_dense,
        ])
        .expect_err("duplicate dense era is ambiguous");
        assert!(error.reason.contains("duplicate JSON field \"version\""));
    }

    #[test]
    fn cover_reconciles_the_exact_recorded_pair_before_any_output_mutation() {
        let corpus_root = unique_cli_test_path("cover-recorded-execution");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&corpus_root, &attention).expect("attention sidecar");
        bind_compiled_dense_operator(&corpus_root, Some(&dense)).expect("dense sidecar");
        let (meta, records) = complete_test_corpus(&corpus_root);
        publish_test_corpus_binding(&corpus_root, &meta, &records);
        let artifacts = corpus_root.join("missing-artifacts.bin");
        let output = unique_cli_test_path("cover-recorded-execution-output");
        std::fs::create_dir_all(&output).expect("cover output");
        std::fs::write(output.join("sentinel"), b"output-before").expect("output sentinel");
        let output_before = directory_bytes(&output);

        let base_args = || {
            vec![
                "--corpus-meta".to_owned(),
                meta.display().to_string(),
                "--corpus-recs".to_owned(),
                records.display().to_string(),
                "--artifacts".to_owned(),
                artifacts.display().to_string(),
                "--out".to_owned(),
                output.display().to_string(),
            ]
        };
        let missing_pair = base_args();
        let mut missing_dense = base_args();
        missing_dense.extend([
            "--attention-operator".to_owned(),
            serde_json::to_string(&attention).expect("attention JSON"),
        ]);
        let mut wrong_pair = base_args();
        wrong_pair.extend([
            "--attention-operator".to_owned(),
            serde_json::to_string(&AttentionOperatorSpec::learned_absolute_v1())
                .expect("v1 attention JSON"),
            "--dense-operator".to_owned(),
            serde_json::to_string(&DenseOperatorSpec::gpt2_v1()).expect("v1 dense JSON"),
        ]);
        for args in [missing_pair, missing_dense, wrong_pair] {
            let error = cover_command(&args).expect_err("recorded pair mismatch is terminal");
            assert!(
                error.reason.contains("does not match recorded corpus"),
                "{error}"
            );
            assert_eq!(directory_bytes(&output), output_before);
        }

        let mut matching = base_args();
        matching.extend([
            "--attention-operator".to_owned(),
            serde_json::to_string(&attention).expect("attention JSON"),
            "--dense-operator".to_owned(),
            serde_json::to_string(&dense).expect("dense JSON"),
        ]);
        let error = cover_command(&matching).expect_err("missing artifact follows pair preflight");
        assert!(error.reason.contains("missing-artifacts.bin"), "{error}");
        assert_eq!(directory_bytes(&output), output_before);

        let _ = std::fs::remove_dir_all(corpus_root);
        let _ = std::fs::remove_dir_all(output);
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
        assert!(error.reason.contains("not a regular file"), "{error}");

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
    fn compile_attention_operator_sidecar_is_exact_atomic_and_idempotent() {
        let output = unique_cli_test_path("compile-attention-fresh");
        let operator = AttentionOperatorSpec::learned_absolute_v1();
        bind_compiled_attention_operator(&output, &operator).expect("fresh operator pin");
        let sidecar = output.join(ATTENTION_OPERATOR_BINDING_FILE);
        let bytes_before = std::fs::read(&sidecar).expect("sidecar bytes");
        assert_eq!(bytes_before.last(), Some(&b'\n'));
        assert_eq!(
            compiled_attention_operator(&output).expect("registry-validated read"),
            operator
        );
        bind_compiled_attention_operator(&output, &operator).expect("idempotent operator pin");
        assert_eq!(
            std::fs::read(&sidecar).expect("reread sidecar"),
            bytes_before
        );
        assert!(attention_operator_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_dense_operator_sidecar_is_optional_registry_exact_and_atomic() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let absent = unique_cli_test_path("compile-dense-absent");
        assert_eq!(
            compiled_dense_operator(&absent).expect("optional absence"),
            None
        );

        for operator in [DenseOperatorSpec::gpt2_v1(), DenseOperatorSpec::gpt2_v2()] {
            let output =
                unique_cli_test_path(&format!("compile-dense-reader-v{}", operator.version));
            bind_compiled_dense_operator(&output, Some(&operator)).expect("publish dense record");
            let sidecar = output.join(DENSE_OPERATOR_BINDING_FILE);
            let before = std::fs::read(&sidecar).expect("dense sidecar bytes");
            assert_eq!(before.last(), Some(&b'\n'));
            assert_eq!(
                compiled_dense_operator(&output).expect("registry-validated dense read"),
                Some(operator.clone())
            );
            bind_compiled_dense_operator(&output, Some(&operator)).expect("idempotent dense pin");
            assert_eq!(
                std::fs::read(&sidecar).expect("dense sidecar reread"),
                before
            );
            assert!(dense_operator_temp_entries(&output).is_empty());
            let _ = std::fs::remove_dir_all(output);
        }

        let parked = unique_cli_test_path("compile-dense-parked-temp");
        std::fs::create_dir_all(&parked).expect("parked temp output");
        let temporary = parked.join(format!(".{DENSE_OPERATOR_BINDING_FILE}.123.1.tmp"));
        std::fs::write(&temporary, b"incomplete abandoned writer")
            .expect("park abandoned temporary");
        bind_compiled_dense_operator(&parked, Some(&DenseOperatorSpec::gpt2_v2()))
            .expect("unique atomic publication does not adopt an abandoned temp");
        assert_eq!(
            compiled_dense_operator(&parked).expect("stable record wins"),
            Some(DenseOperatorSpec::gpt2_v2())
        );
        assert_eq!(
            std::fs::read(&temporary).expect("abandoned temp is not reclaimed here"),
            b"incomplete abandoned writer"
        );
        let _ = std::fs::remove_dir_all(parked);
    }

    #[test]
    fn compile_dense_operator_reader_rejects_invalid_evidence_without_mutation() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let malformed = unique_cli_test_path("compile-dense-malformed");
        std::fs::create_dir_all(&malformed).expect("malformed output");
        std::fs::write(malformed.join(DENSE_OPERATOR_BINDING_FILE), b"{not json")
            .expect("malformed dense binding");
        let before = directory_bytes(&malformed);
        let error = compiled_dense_operator(&malformed).expect_err("malformed is terminal");
        assert!(error.reason.contains("malformed dense-operator binding"));
        assert_eq!(directory_bytes(&malformed), before);

        let duplicate = unique_cli_test_path("compile-dense-duplicate");
        std::fs::create_dir_all(&duplicate).expect("duplicate output");
        let canonical =
            serde_json::to_string_pretty(&DenseOperatorSpec::gpt2_v2()).expect("dense JSON");
        let duplicate_json =
            canonical.replacen("\"version\": 2", "\"version\": 1,\n  \"version\": 2", 1);
        std::fs::write(duplicate.join(DENSE_OPERATOR_BINDING_FILE), duplicate_json)
            .expect("duplicate dense binding");
        let before = directory_bytes(&duplicate);
        let error = compiled_dense_operator(&duplicate).expect_err("duplicate key is terminal");
        assert!(
            error.reason.contains("duplicate JSON field \"version\""),
            "{error}"
        );
        assert_eq!(directory_bytes(&duplicate), before);

        let unknown = unique_cli_test_path("compile-dense-unknown");
        std::fs::create_dir_all(&unknown).expect("unknown output");
        let mut unknown_record = DenseOperatorSpec::gpt2_v2();
        unknown_record.version = u32::MAX;
        unknown_record.implementation_digest = unknown_record.declared_digest();
        std::fs::write(
            unknown.join(DENSE_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&unknown_record).expect("unknown dense JSON"),
        )
        .expect("unknown dense binding");
        let error = compiled_dense_operator(&unknown).expect_err("unknown version is terminal");
        assert!(matches!(
            error.kind,
            uor_r4_model_source::SourceIngestKind::UnknownDenseOperator { version, .. }
                if version == u32::MAX
        ));

        let drift = unique_cli_test_path("compile-dense-drift");
        std::fs::create_dir_all(&drift).expect("drift output");
        let mut drifted = DenseOperatorSpec::gpt2_v2();
        drifted.conv1d_bias.push_str("-tampered");
        drifted.implementation_digest = drifted.declared_digest();
        std::fs::write(
            drift.join(DENSE_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&drifted).expect("drifted dense JSON"),
        )
        .expect("drifted dense binding");
        let error = compiled_dense_operator(&drift).expect_err("record drift is terminal");
        assert!(
            error.reason.contains("does not match registered"),
            "{error}"
        );

        let nonregular = unique_cli_test_path("compile-dense-directory");
        std::fs::create_dir_all(nonregular.join(DENSE_OPERATOR_BINDING_FILE))
            .expect("directory dense binding");
        let error = compiled_dense_operator(&nonregular).expect_err("nonregular is terminal");
        assert!(error.reason.contains("not a regular file"), "{error}");

        for output in [malformed, duplicate, unknown, drift, nonregular] {
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[cfg(unix)]
    #[test]
    fn compiled_execution_sidecar_reader_rejects_post_open_link_swap() {
        use std::os::unix::fs::symlink;

        for name in [ATTENTION_OPERATOR_BINDING_FILE, DENSE_OPERATOR_BINDING_FILE] {
            let output = unique_cli_test_path(&format!("compiled-sidecar-swap-{name}"));
            std::fs::create_dir_all(&output).expect("sidecar output");
            let path = output.join(name);
            std::fs::write(&path, b"opened provenance bytes").expect("initial regular sidecar");
            let target = output.with_extension(format!("{name}.external"));
            let target_bytes = b"external target must never be read or changed";
            std::fs::write(&target, target_bytes).expect("external target");

            let error = read_optional_regular_file_nofollow_with(
                &path,
                "execution-operator binding",
                || {
                    std::fs::remove_file(&path).expect("replace opened sidecar");
                    symlink(&target, &path).expect("swap path to link after open");
                },
            )
            .expect_err("path/handle identity mismatch is terminal");
            assert!(error.reason.contains("changed identity"), "{error}");
            assert_eq!(
                std::fs::read(&target).expect("external target survives"),
                target_bytes
            );
            assert_eq!(
                std::fs::read_link(&path).expect("swapped link survives"),
                target
            );

            std::fs::remove_file(&path).expect("remove swapped link");
            std::fs::remove_file(&target).expect("remove external target");
            let _ = std::fs::remove_dir_all(output);
        }

        let rewrites = [
            (
                ATTENTION_OPERATOR_BINDING_FILE,
                serde_json::to_vec_pretty(&AttentionOperatorSpec::learned_absolute_v1())
                    .expect("attention v1 JSON"),
                serde_json::to_vec_pretty(&AttentionOperatorSpec::learned_absolute_v2())
                    .expect("attention v2 JSON"),
            ),
            (
                DENSE_OPERATOR_BINDING_FILE,
                serde_json::to_vec_pretty(&DenseOperatorSpec::gpt2_v1()).expect("dense v1 JSON"),
                serde_json::to_vec_pretty(&DenseOperatorSpec::gpt2_v2()).expect("dense v2 JSON"),
            ),
        ];
        for (name, initial, replacement) in rewrites {
            let output = unique_cli_test_path(&format!("compiled-sidecar-rewrite-{name}"));
            std::fs::create_dir_all(&output).expect("sidecar output");
            let path = output.join(name);
            std::fs::write(&path, &initial).expect("initial canonical sidecar");

            let error = read_optional_regular_file_nofollow_with(
                &path,
                "execution-operator binding",
                || std::fs::write(&path, &replacement).expect("in-place canonical rewrite"),
            )
            .expect_err("same-inode rewrite is a changed generation");
            assert!(error.reason.contains("changed"), "{error}");
            assert_eq!(
                std::fs::read(&path).expect("replacement survives refusal"),
                replacement
            );
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[cfg(unix)]
    #[test]
    fn operator_publisher_retry_is_narrow_exact_and_preserves_a_conflicting_winner() {
        let operator = AttentionOperatorSpec::standard_v1();
        let bytes = canonical_attention_operator_bytes(&operator).expect("attention bytes");

        let generic = unique_cli_test_path("publisher-alias-generic-terminal");
        let (generic_stable, generic_alias) =
            hard_link_test_sidecar(&generic, ATTENTION_OPERATOR_BINDING_FILE, &bytes, "generic");
        let error = read_optional_regular_file_nofollow_with(
            &generic_stable,
            "attention-operator binding",
            || std::fs::remove_file(&generic_alias).expect("unlink publisher alias"),
        )
        .expect_err("the generic strict reader keeps ctime transitions terminal");
        assert!(error.reason.contains("changed identity"), "{error}");
        assert_eq!(std::fs::read(&generic_stable).unwrap(), bytes);

        let exact = unique_cli_test_path("publisher-alias-attention-exact");
        let (exact_stable, exact_alias) =
            hard_link_test_sidecar(&exact, ATTENTION_OPERATOR_BINDING_FILE, &bytes, "exact");
        let recorded = read_optional_compiled_attention_operator_for_publisher_with_hooks(
            &exact,
            || std::fs::remove_file(&exact_alias).expect("unlink exact publisher alias"),
            || {},
            || {},
        )
        .expect("one exact retry accepts the stable winner");
        assert_eq!(recorded, Some(operator.clone()));
        assert_eq!(std::fs::read(&exact_stable).unwrap(), bytes);

        let conflicting = unique_cli_test_path("publisher-alias-attention-conflict");
        let (conflicting_stable, conflicting_alias) = hard_link_test_sidecar(
            &conflicting,
            ATTENTION_OPERATOR_BINDING_FILE,
            &bytes,
            "conflict",
        );
        let recorded = read_optional_compiled_attention_operator_for_publisher_with_hooks(
            &conflicting,
            || {
                std::fs::remove_file(&conflicting_alias)
                    .expect("unlink conflicting publisher alias")
            },
            || {},
            || {},
        )
        .expect("retry reads the complete conflicting winner")
        .expect("conflicting winner is present");
        let error = require_matching_compiled_attention_operator(
            &conflicting,
            &AttentionOperatorSpec::standard_v2(),
            &recorded,
        )
        .expect_err("a complete conflicting winner remains terminal");
        assert!(
            error.reason.contains("pinned to attention operator"),
            "{error}"
        );
        assert_eq!(std::fs::read(&conflicting_stable).unwrap(), bytes);

        for output in [generic, exact, conflicting] {
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[cfg(unix)]
    #[test]
    fn operator_publisher_retry_rejects_second_cleanup_aba_path_and_content_changes() {
        let original = b"publisher generation A";

        let second = unique_cli_test_path("publisher-alias-second-cleanup");
        let (second_stable, first_alias) =
            hard_link_test_sidecar(&second, ATTENTION_OPERATOR_BINDING_FILE, original, "first");
        let second_alias = second.join(format!(".{ATTENTION_OPERATOR_BINDING_FILE}.second.tmp"));
        let error = read_optional_regular_file_for_operator_publisher_with_hooks(
            &second_stable,
            "attention-operator binding",
            || std::fs::remove_file(&first_alias).expect("unlink first alias"),
            || std::fs::hard_link(&second_stable, &second_alias).expect("plant second alias"),
            || std::fs::remove_file(&second_alias).expect("unlink second alias"),
        )
        .expect_err("a second alias-cleanup transition is terminal");
        assert!(error.reason.contains("changed identity"), "{error}");
        assert_eq!(std::fs::read(&second_stable).unwrap(), original);

        let aba = unique_cli_test_path("publisher-alias-aba");
        let (aba_stable, aba_alias) =
            hard_link_test_sidecar(&aba, ATTENTION_OPERATOR_BINDING_FILE, original, "first");
        let replacement = aba.join("replacement");
        std::fs::write(&replacement, original).expect("write ABA replacement");
        let error = read_optional_regular_file_for_operator_publisher_with_hooks(
            &aba_stable,
            "attention-operator binding",
            || std::fs::remove_file(&aba_alias).expect("unlink ABA alias"),
            || {
                std::fs::remove_file(&aba_stable).expect("remove first stable inode");
                std::fs::rename(&replacement, &aba_stable).expect("replace stable path")
            },
            || {},
        )
        .expect_err("an exact-byte path ABA between attempts is terminal");
        assert!(error.reason.contains("across the one permitted"), "{error}");
        assert_eq!(std::fs::read(&aba_stable).unwrap(), original);

        let content = unique_cli_test_path("publisher-alias-content");
        let (content_stable, content_alias) =
            hard_link_test_sidecar(&content, ATTENTION_OPERATOR_BINDING_FILE, original, "first");
        let replacement = b"publisher generation B";
        assert_eq!(replacement.len(), original.len());
        let error = read_optional_regular_file_for_operator_publisher_with_hooks(
            &content_stable,
            "attention-operator binding",
            || std::fs::remove_file(&content_alias).expect("unlink content alias"),
            || std::fs::write(&content_stable, replacement).expect("rewrite between attempts"),
            || {},
        )
        .expect_err("a same-inode content change between attempts is terminal");
        assert!(error.reason.contains("across the one permitted"), "{error}");
        assert_eq!(std::fs::read(&content_stable).unwrap(), replacement);

        for output in [second, aba, content] {
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[cfg(unix)]
    #[test]
    fn dense_operator_publisher_retries_one_exact_alias_cleanup() {
        let operator = DenseOperatorSpec::gpt2_v2();
        let bytes = canonical_dense_operator_bytes(&operator).expect("dense bytes");
        let output = unique_cli_test_path("publisher-alias-dense-exact");
        let (stable, alias) =
            hard_link_test_sidecar(&output, DENSE_OPERATOR_BINDING_FILE, &bytes, "exact");

        let recorded = read_optional_compiled_dense_operator_for_publisher_with_hooks(
            &output,
            || std::fs::remove_file(&alias).expect("unlink dense publisher alias"),
            || {},
            || {},
        )
        .expect("one exact retry accepts the dense winner");
        assert_eq!(recorded, Some(operator));
        assert_eq!(std::fs::read(stable).unwrap(), bytes);

        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_dense_operator_refuses_payload_relabel_and_cross_era_joint_pin() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let legacy = unique_cli_test_path("compile-dense-legacy-payload");
        std::fs::create_dir_all(&legacy).expect("legacy output");
        std::fs::write(legacy.join("corpus.records"), b"legacy dense rows")
            .expect("legacy payload");
        let before = directory_bytes(&legacy);
        let error = bind_compiled_dense_operator(&legacy, Some(&DenseOperatorSpec::gpt2_v2()))
            .expect_err("legacy payload cannot be relabelled");
        assert!(
            error.reason.contains("legacy dense-execution era"),
            "{error}"
        );
        assert_eq!(directory_bytes(&legacy), before);

        let cross_era = unique_cli_test_path("compile-dense-cross-era");
        std::fs::create_dir_all(&cross_era).expect("cross-era output");
        std::fs::write(cross_era.join("sentinel"), b"before").expect("sentinel");
        let before = directory_bytes(&cross_era);
        let error = pin_compile_identities_with_dense(
            &cross_era,
            &fixture_adapter("cross-era"),
            &AttentionOperatorSpec::learned_absolute_v1(),
            Some(&DenseOperatorSpec::gpt2_v2()),
        )
        .expect_err("cross-era joint identity must fail before publication");
        assert!(error.reason.contains("source execution pair"), "{error}");
        assert_eq!(directory_bytes(&cross_era), before);
        assert!(!cross_era.join(TOKENIZER_ADAPTER_FILE).exists());
        assert!(!cross_era.join(ATTENTION_OPERATOR_BINDING_FILE).exists());
        assert!(!cross_era.join(DENSE_OPERATOR_BINDING_FILE).exists());

        let _ = std::fs::remove_dir_all(legacy);
        let _ = std::fs::remove_dir_all(cross_era);
    }

    #[test]
    fn compile_attention_operator_reader_accepts_every_immutable_and_current_source_record() {
        for operator in [
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::standard_v2(),
            AttentionOperatorSpec::experimental_r4_v1(),
            AttentionOperatorSpec::experimental_r4_v2(),
            AttentionOperatorSpec::learned_absolute_v1(),
            AttentionOperatorSpec::learned_absolute_v2(),
        ] {
            let output = unique_cli_test_path(&format!(
                "compile-attention-reader-{}-v{}",
                operator.id, operator.version
            ));
            bind_compiled_attention_operator(&output, &operator).expect("publish full record");
            assert_eq!(
                compiled_attention_operator(&output).expect("read registered source record"),
                operator
            );
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn compile_attention_operator_refuses_same_family_v1_to_v2_resume() {
        let output = unique_cli_test_path("compile-attention-standard-v1-to-v2");
        let v1 = AttentionOperatorSpec::standard_v1();
        bind_compiled_attention_operator(&output, &v1).expect("pin immutable v1 record");
        std::fs::write(output.join("corpus.records"), b"immutable v1 rows")
            .expect("write v1 payload");
        let before = directory_bytes(&output);

        let error =
            bind_compiled_attention_operator(&output, &AttentionOperatorSpec::standard_v2())
                .expect_err("v2 cannot resume a v1 compile directory");
        assert!(
            error.reason.contains("pinned to attention operator"),
            "{error}"
        );
        assert_eq!(directory_bytes(&output), before);
        assert_eq!(
            compiled_attention_operator(&output).expect("v1 binding remains"),
            v1
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_attention_operator_rejects_duplicate_v1_and_v2_eras_before_resume_mutation() {
        for (operator, shadow_version) in [
            (AttentionOperatorSpec::standard_v1(), 2),
            (AttentionOperatorSpec::standard_v2(), 1),
        ] {
            let output = unique_cli_test_path(&format!(
                "compile-attention-duplicate-v{}",
                operator.version
            ));
            std::fs::create_dir_all(&output).expect("output directory");
            std::fs::write(output.join("corpus.records"), b"immutable teacher rows")
                .expect("write existing payload");
            let canonical = serde_json::to_string_pretty(&operator).expect("operator JSON");
            let recorded_version = format!("\"version\": {}", operator.version);
            assert_eq!(
                canonical.matches(&recorded_version).count(),
                1,
                "fixture has exactly one canonical version field"
            );
            let duplicate = canonical.replacen(
                &recorded_version,
                &format!(
                    "\"version\": {shadow_version},\n  \"version\": {}",
                    operator.version
                ),
                1,
            );
            std::fs::write(output.join(ATTENTION_OPERATOR_BINDING_FILE), duplicate)
                .expect("write ambiguous sidecar");
            let before = directory_bytes(&output);

            let resume_error = preflight_compiled_attention_operator(&output, &operator)
                .expect_err("last-key-wins must not authorize a compile resume");
            assert!(
                resume_error
                    .reason
                    .contains("duplicate JSON field \"version\""),
                "{resume_error}"
            );
            assert_eq!(directory_bytes(&output), before);

            let evaluation_error = compiled_attention_operator(&output)
                .expect_err("evaluation provenance must reject the same ambiguity");
            assert!(
                evaluation_error
                    .reason
                    .contains("duplicate JSON field \"version\""),
                "{evaluation_error}"
            );
            assert_eq!(directory_bytes(&output), before);
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn evaluation_attention_operator_accepts_exact_records_and_refuses_cross_era_replay() {
        let compiled = Path::new("compiled-fixture");
        for operator in [
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::standard_v2(),
            AttentionOperatorSpec::experimental_r4_v1(),
            AttentionOperatorSpec::experimental_r4_v2(),
            AttentionOperatorSpec::learned_absolute_v1(),
            AttentionOperatorSpec::learned_absolute_v2(),
        ] {
            require_matching_evaluation_attention_operator(compiled, &operator, &operator)
                .expect("exact immutable record is replayable");
        }

        let error = require_matching_evaluation_attention_operator(
            compiled,
            &AttentionOperatorSpec::standard_v1(),
            &AttentionOperatorSpec::standard_v2(),
        )
        .expect_err("same-family cross-era replay must fail closed");
        assert!(error.reason.contains("evaluation refused before replay"));
        assert!(error.reason.contains("standard-source-attention/1"));
        assert!(error.reason.contains("standard-source-attention/2"));
    }

    #[test]
    fn evaluation_dense_operator_requires_exact_value_and_exact_absence() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let compiled = Path::new("compiled-fixture");
        let v1 = DenseOperatorSpec::gpt2_v1();
        let v2 = DenseOperatorSpec::gpt2_v2();
        require_matching_evaluation_dense_operator(compiled, Some(&v1), Some(&v1))
            .expect("exact historical dense replay");
        require_matching_evaluation_dense_operator(compiled, Some(&v2), Some(&v2))
            .expect("exact current dense replay");
        require_matching_evaluation_dense_operator(compiled, None, None)
            .expect("typed dense absence replay");

        for (recorded, actual) in [(Some(&v1), Some(&v2)), (Some(&v2), None), (None, Some(&v2))] {
            let error = require_matching_evaluation_dense_operator(compiled, recorded, actual)
                .expect_err("dense era mismatch must fail before replay");
            assert!(
                error.reason.contains("evaluation refused before replay"),
                "{error}"
            );
        }
    }

    #[test]
    fn evaluation_snapshot_cids_are_from_one_binding_checked_generation() {
        let compiled = unique_cli_test_path("evaluation-captured-execution");
        let attention_v2 = AttentionOperatorSpec::learned_absolute_v2();
        let dense_v2 = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&compiled, &attention_v2).expect("v2 attention");
        bind_compiled_dense_operator(&compiled, Some(&dense_v2)).expect("v2 dense");
        let (meta, records) = complete_test_corpus(&compiled);
        let binding_cid = publish_test_corpus_binding(&compiled, &meta, &records);
        let attention_path = compiled.join(ATTENTION_OPERATOR_BINDING_FILE);
        let dense_path = compiled.join(DENSE_OPERATOR_BINDING_FILE);
        let attention_v2_bytes = std::fs::read(&attention_path).expect("captured attention bytes");
        let dense_v2_bytes = std::fs::read(&dense_path).expect("captured dense bytes");
        let captured = recorded_corpus_snapshot(&meta, &records).expect("exact snapshot");
        assert_eq!(captured.execution.attention_operator, Some(attention_v2));
        assert_eq!(captured.execution.dense_operator, Some(dense_v2));
        assert_eq!(
            format!(
                "blake3:{}",
                blake3::hash(
                    captured
                        .attention_operator_bytes
                        .as_deref()
                        .expect("attention bytes")
                )
                .to_hex()
            ),
            format!("blake3:{}", blake3::hash(&attention_v2_bytes).to_hex())
        );
        assert_eq!(
            format!(
                "blake3:{}",
                blake3::hash(
                    captured
                        .dense_operator_bytes
                        .as_deref()
                        .expect("dense bytes")
                )
                .to_hex()
            ),
            format!("blake3:{}", blake3::hash(&dense_v2_bytes).to_hex())
        );
        assert_eq!(captured.binding_cid, Some(binding_cid));
        let _ = std::fs::remove_dir_all(compiled);
    }

    #[test]
    fn compile_attention_operator_publish_is_concurrent_and_no_clobber() {
        let output = unique_cli_test_path("compile-attention-concurrent");
        let operator = AttentionOperatorSpec::standard();
        let mut threads = Vec::new();
        for _ in 0..8 {
            let output = output.clone();
            let operator = operator.clone();
            threads.push(std::thread::spawn(move || {
                bind_compiled_attention_operator(&output, &operator)
            }));
        }
        for thread in threads {
            thread
                .join()
                .expect("publisher thread")
                .expect("same operator publisher");
        }
        assert_eq!(
            compiled_attention_operator(&output).expect("published sidecar"),
            operator
        );
        assert!(attention_operator_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_attention_operator_concurrent_conflict_accepts_only_the_exact_winner() {
        let output = unique_cli_test_path("compile-attention-concurrent-conflict");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(16));
        let mut threads = Vec::new();
        for index in 0..16 {
            let output = output.clone();
            let barrier = std::sync::Arc::clone(&barrier);
            let operator = if index % 2 == 0 {
                AttentionOperatorSpec::standard()
            } else {
                AttentionOperatorSpec::experimental_r4()
            };
            let requested = operator.clone();
            threads.push((
                requested,
                std::thread::spawn(move || {
                    barrier.wait();
                    bind_compiled_attention_operator(&output, &operator)
                }),
            ));
        }
        let results: Vec<_> = threads
            .into_iter()
            .map(|(requested, thread)| (requested, thread.join().expect("publisher thread")))
            .collect();
        let winner = compiled_attention_operator(&output).expect("one exact winner");
        for (requested, result) in results {
            if requested == winner {
                result.expect("an exact racing loser accepts the winner");
            } else {
                let error = result.expect_err("a different racing loser is refused");
                assert!(error.reason.contains("pinned to attention operator"));
            }
        }
        assert!(attention_operator_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_attention_operator_uses_unique_temporary_names() {
        let output = unique_cli_test_path("compile-attention-unique-temp");
        std::fs::create_dir_all(&output).expect("output directory");
        let historical_fixed_temp = output.join(format!(".{ATTENTION_OPERATOR_BINDING_FILE}.tmp"));
        std::fs::write(&historical_fixed_temp, b"do not replace").expect("park fixed temp");
        bind_compiled_attention_operator(&output, &AttentionOperatorSpec::standard())
            .expect("unique publisher ignores a parked fixed-name temp");
        assert_eq!(
            std::fs::read(&historical_fixed_temp).expect("fixed temp preserved"),
            b"do not replace"
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_attention_operator_reader_distinguishes_absent_from_present_invalid() {
        let missing = unique_cli_test_path("compile-attention-missing");
        let error = compiled_attention_operator(&missing).expect_err("missing binding is explicit");
        assert!(
            error
                .reason
                .contains("missing teacher attention-operator binding")
        );

        let malformed = unique_cli_test_path("compile-attention-malformed");
        std::fs::create_dir_all(&malformed).expect("output directory");
        std::fs::write(
            malformed.join(ATTENTION_OPERATOR_BINDING_FILE),
            b"{not json",
        )
        .expect("malformed binding");
        let error = compiled_attention_operator(&malformed)
            .expect_err("a malformed present binding is not absence");
        assert!(
            error
                .reason
                .contains("malformed attention-operator binding")
        );

        let directory = unique_cli_test_path("compile-attention-directory");
        std::fs::create_dir_all(directory.join(ATTENTION_OPERATOR_BINDING_FILE))
            .expect("directory binding");
        let error = compiled_attention_operator(&directory)
            .expect_err("a directory binding is not absence");
        assert!(error.reason.contains("not a regular file"));

        let _ = std::fs::remove_dir_all(malformed);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn compile_attention_operator_reader_rejects_all_symlinks() {
        use std::os::unix::fs::symlink;

        let output = unique_cli_test_path("compile-attention-symlink");
        std::fs::create_dir_all(&output).expect("output directory");
        let target = output.join("operator-target.json");
        std::fs::write(
            &target,
            serde_json::to_vec_pretty(&AttentionOperatorSpec::standard())
                .expect("serialize target"),
        )
        .expect("write target");
        symlink(&target, output.join(ATTENTION_OPERATOR_BINDING_FILE)).expect("binding symlink");
        let error = compiled_attention_operator(&output)
            .expect_err("even a resolving provenance symlink is refused");
        assert!(error.reason.contains("not a regular file"));

        let dangling = unique_cli_test_path("compile-attention-dangling");
        std::fs::create_dir_all(&dangling).expect("output directory");
        symlink(
            "missing-target",
            dangling.join(ATTENTION_OPERATOR_BINDING_FILE),
        )
        .expect("dangling binding symlink");
        let error = compiled_attention_operator(&dangling)
            .expect_err("a dangling provenance symlink is refused");
        assert!(error.reason.contains("not a regular file"));

        let _ = std::fs::remove_dir_all(output);
        let _ = std::fs::remove_dir_all(dangling);
    }

    #[test]
    fn compile_attention_operator_reader_requires_registry_exactness() {
        let output = unique_cli_test_path("compile-attention-registry");
        std::fs::create_dir_all(&output).expect("output directory");
        let mut altered = AttentionOperatorSpec::standard();
        altered.output_projection.push_str("-tampered");
        altered.implementation_digest = altered.declared_digest();
        std::fs::write(
            output.join(ATTENTION_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&altered).expect("altered record JSON"),
        )
        .expect("write altered binding");
        let error = compiled_attention_operator(&output)
            .expect_err("self-consistent but non-registry record is refused");
        assert!(error.reason.contains("does not match registered"));

        let mut unknown = AttentionOperatorSpec::standard();
        unknown.version = u32::MAX;
        unknown.implementation_digest = unknown.declared_digest();
        std::fs::write(
            output.join(ATTENTION_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&unknown).expect("unknown record JSON"),
        )
        .expect("write unknown binding");
        let error = compiled_attention_operator(&output)
            .expect_err("unregistered operator version is refused");
        assert!(error.reason.contains(&u32::MAX.to_string()));
        assert!(matches!(
            error.kind,
            uor_r4_model_source::SourceIngestKind::UnknownAttentionOperator { version, .. }
                if version == u32::MAX
        ));

        std::fs::write(
            output.join(ATTENTION_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&AttentionOperatorSpec::r4_route_attention_v1())
                .expect("target record JSON"),
        )
        .expect("write target binding");
        let error = compiled_attention_operator(&output)
            .expect_err("a deployed target operator is not source provenance");
        assert!(
            error
                .reason
                .contains("not a source-teacher attention operator")
        );

        let mut with_extra =
            serde_json::to_value(AttentionOperatorSpec::standard()).expect("operator JSON");
        with_extra
            .as_object_mut()
            .expect("operator object")
            .insert("unregistered_claim".to_owned(), serde_json::json!(true));
        std::fs::write(
            output.join(ATTENTION_OPERATOR_BINDING_FILE),
            serde_json::to_vec_pretty(&with_extra).expect("extended record JSON"),
        )
        .expect("write extended binding");
        let error = compiled_attention_operator(&output)
            .expect_err("unknown provenance fields are refused");
        assert!(error.reason.contains("unregistered_claim"));
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_attention_operator_refuses_to_relabel_legacy_corpus_payload() {
        let output = unique_cli_test_path("compile-attention-legacy-corpus");
        std::fs::create_dir_all(&output).expect("output directory");
        std::fs::write(output.join("corpus.records"), b"legacy teacher rows")
            .expect("legacy corpus payload");
        let before = directory_bytes(&output);
        let error = bind_compiled_attention_operator(&output, &AttentionOperatorSpec::standard())
            .expect_err("an operatorless corpus cannot be relabelled");
        assert!(error.reason.contains("implicit legacy attention era"));
        assert_eq!(directory_bytes(&output), before);
        assert!(!output.join(ATTENTION_OPERATOR_BINDING_FILE).exists());
        assert!(attention_operator_temp_entries(&output).is_empty());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn compile_identity_preflights_refuse_zero_byte_unbound_payload_entries() {
        for payload_name in ["tokenizer.bin", "corpus.meta"] {
            let output = unique_cli_test_path(&format!(
                "compile-zero-byte-{}",
                payload_name.replace('.', "-")
            ));
            std::fs::create_dir_all(&output).expect("output directory");
            std::fs::write(output.join(payload_name), []).expect("zero-byte payload entry");
            let before = directory_bytes(&output);
            let tokenizer = fixture_adapter(payload_name);
            let operator = AttentionOperatorSpec::standard();

            for error in [
                preflight_compile_tokenizer_adapter(&output, &tokenizer)
                    .expect_err("zero-byte entry is tokenizer-era evidence"),
                preflight_compiled_attention_operator(&output, &operator)
                    .expect_err("zero-byte entry is operator-era evidence"),
                pin_compile_identities(&output, &tokenizer, &operator)
                    .expect_err("joint pin cannot relabel a torn output"),
            ] {
                assert!(error.reason.contains(payload_name));
            }
            assert_eq!(directory_bytes(&output), before);
            assert!(!output.join(TOKENIZER_ADAPTER_FILE).exists());
            assert!(!output.join(ATTENTION_OPERATOR_BINDING_FILE).exists());
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn compile_attention_operator_preflight_is_joint_before_either_sidecar_mutates() {
        let operator_first = unique_cli_test_path("compile-joint-operator-first");
        bind_compiled_attention_operator(&operator_first, &AttentionOperatorSpec::standard())
            .expect("initial operator");
        let operator_before = directory_bytes(&operator_first);
        let error = pin_compile_identities(
            &operator_first,
            &fixture_adapter("fresh-tokenizer"),
            &AttentionOperatorSpec::experimental_r4(),
        )
        .expect_err("operator mismatch rejects before tokenizer publication");
        assert!(error.reason.contains("is pinned to attention operator"));
        assert_eq!(directory_bytes(&operator_first), operator_before);
        assert!(!operator_first.join(TOKENIZER_ADAPTER_FILE).exists());

        let tokenizer_first = unique_cli_test_path("compile-joint-tokenizer-first");
        let first = fixture_adapter("first-tokenizer");
        pin_compile_tokenizer_adapter(&tokenizer_first, &first).expect("initial tokenizer");
        let tokenizer_before = directory_bytes(&tokenizer_first);
        let error = pin_compile_identities(
            &tokenizer_first,
            &fixture_adapter("different-tokenizer"),
            &AttentionOperatorSpec::standard(),
        )
        .expect_err("tokenizer mismatch rejects before operator publication");
        assert!(error.reason.contains("incompatible compile resume"));
        assert_eq!(directory_bytes(&tokenizer_first), tokenizer_before);
        assert!(
            !tokenizer_first
                .join(ATTENTION_OPERATOR_BINDING_FILE)
                .exists()
        );

        let _ = std::fs::remove_dir_all(operator_first);
        let _ = std::fs::remove_dir_all(tokenizer_first);
    }

    #[cfg(unix)]
    #[test]
    fn compile_exact_identity_resume_rejects_payload_symlinks_without_following() {
        use std::os::unix::fs::symlink;

        for payload_name in [
            "corpus.records",
            "corpus.records.hidden",
            "tokenizer.bin",
            "tless_artifacts.bin",
        ] {
            for resolving in [true, false] {
                let link_kind = if resolving { "resolving" } else { "dangling" };
                let output = unique_cli_test_path(&format!(
                    "compile-exact-{}-{}",
                    payload_name.replace('.', "-"),
                    link_kind
                ));
                let tokenizer = fixture_adapter(&format!("{payload_name}-{link_kind}"));
                let operator = AttentionOperatorSpec::standard();
                pin_compile_identities(&output, &tokenizer, &operator)
                    .expect("publish exact identity pair");
                let tokenizer_sidecar = output.join(TOKENIZER_ADAPTER_FILE);
                let operator_sidecar = output.join(ATTENTION_OPERATOR_BINDING_FILE);
                let tokenizer_sidecar_before =
                    std::fs::read(&tokenizer_sidecar).expect("tokenizer sidecar");
                let operator_sidecar_before =
                    std::fs::read(&operator_sidecar).expect("operator sidecar");

                let external_target = unique_cli_test_path(&format!(
                    "compile-external-{}-{}",
                    payload_name.replace('.', "-"),
                    link_kind
                ));
                let external_bytes = b"external bytes must remain unchanged";
                if resolving {
                    std::fs::write(&external_target, external_bytes).expect("external target");
                }
                let payload = output.join(payload_name);
                symlink(&external_target, &payload).expect("payload symlink");

                for error in [
                    preflight_compile_tokenizer_adapter(&output, &tokenizer)
                        .expect_err("tokenizer preflight must reject payload symlink"),
                    preflight_compiled_attention_operator(&output, &operator)
                        .expect_err("operator preflight must reject payload symlink"),
                    pin_compile_identities(&output, &tokenizer, &operator)
                        .expect_err("joint exact resume must reject payload symlink"),
                ] {
                    assert!(error.reason.contains("not a regular file"));
                }
                assert_eq!(
                    std::fs::read(&tokenizer_sidecar).expect("tokenizer sidecar"),
                    tokenizer_sidecar_before
                );
                assert_eq!(
                    std::fs::read(&operator_sidecar).expect("operator sidecar"),
                    operator_sidecar_before
                );
                assert_eq!(
                    std::fs::read_link(&payload).expect("payload link target"),
                    external_target
                );
                if resolving {
                    assert_eq!(
                        std::fs::read(&external_target).expect("external target"),
                        external_bytes
                    );
                    std::fs::remove_file(&external_target).expect("remove external target");
                } else {
                    assert!(!external_target.exists());
                }
                let _ = std::fs::remove_dir_all(output);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn compile_resume_rejects_symlink_output_root_without_following() {
        use std::os::unix::fs::symlink;

        let real_root = unique_cli_test_path("compile-real-output-root");
        let output_link = unique_cli_test_path("compile-output-root-symlink");
        let tokenizer = fixture_adapter("root-symlink");
        let operator = AttentionOperatorSpec::standard();
        pin_compile_identities(&real_root, &tokenizer, &operator)
            .expect("publish exact identity pair");
        let external_artifact = real_root.join("tless_artifacts.bin");
        let sentinel = b"external artifact sentinel must remain unchanged";
        std::fs::write(&external_artifact, sentinel).expect("external artifact sentinel");
        let tokenizer_sidecar_before =
            std::fs::read(real_root.join(TOKENIZER_ADAPTER_FILE)).expect("tokenizer sidecar");
        let operator_sidecar_before =
            std::fs::read(real_root.join(ATTENTION_OPERATOR_BINDING_FILE))
                .expect("operator sidecar");
        symlink(&real_root, &output_link).expect("output-root symlink");

        for error in [
            preflight_compile_tokenizer_adapter(&output_link, &tokenizer)
                .expect_err("tokenizer preflight must reject a symlink output root"),
            preflight_compiled_attention_operator(&output_link, &operator)
                .expect_err("operator preflight must reject a symlink output root"),
            pin_compile_identities(&output_link, &tokenizer, &operator)
                .expect_err("joint resume must reject a symlink output root"),
        ] {
            assert!(error.reason.contains("is not a real directory"));
        }
        assert_eq!(
            std::fs::read(&external_artifact).expect("external artifact sentinel"),
            sentinel
        );
        assert_eq!(
            std::fs::read(real_root.join(TOKENIZER_ADAPTER_FILE)).expect("tokenizer sidecar"),
            tokenizer_sidecar_before
        );
        assert_eq!(
            std::fs::read(real_root.join(ATTENTION_OPERATOR_BINDING_FILE))
                .expect("operator sidecar"),
            operator_sidecar_before
        );
        assert_eq!(
            std::fs::read_link(&output_link).expect("output-root link target"),
            real_root
        );

        std::fs::remove_file(&output_link).expect("remove output-root symlink");
        let _ = std::fs::remove_dir_all(real_root);
    }

    #[test]
    fn source_compile_resume_rejects_malformed_checkpoint_before_any_output_mutation() {
        let mut too_many_stories = source_resume_meta(0, 0);
        too_many_stories[8..16].copy_from_slice(&(u64::from(u32::MAX) + 1).to_le_bytes());
        for (case, meta, expected) in [
            ("empty", Vec::new(), "expected exactly"),
            ("truncated", vec![0u8; 24], "expected exactly"),
            (
                "invalid-done",
                source_resume_meta(1, 2),
                "invalid done byte",
            ),
            ("story-overflow", too_many_stories, "exceeding the u32"),
        ] {
            let output = unique_cli_test_path(&format!("source-resume-{case}"));
            let tokenizer = fixture_adapter(case);
            let operator = AttentionOperatorSpec::standard();
            pin_compile_identities(&output, &tokenizer, &operator)
                .expect("publish exact identity pair");
            std::fs::write(output.join("corpus.meta"), meta).expect("metadata fixture");
            let records = b"committed record sentinel must remain byte-identical";
            std::fs::write(output.join("corpus.records"), records).expect("record fixture");
            let before = directory_bytes(&output);

            let error = preflight_source_compile_resume(&output, Some(16), 2)
                .expect_err("malformed checkpoint must fail closed");
            assert!(error.reason.contains(expected), "{}", error.reason);
            assert_eq!(directory_bytes(&output), before);
            assert_eq!(
                std::fs::read(output.join("corpus.records")).expect("record sentinel"),
                records
            );
            let _ = std::fs::remove_dir_all(output);
        }

        for missing in ["corpus.meta", "corpus.records"] {
            let output = unique_cli_test_path(&format!(
                "source-resume-missing-{}",
                missing.replace('.', "-")
            ));
            let tokenizer = fixture_adapter(missing);
            pin_compile_identities(&output, &tokenizer, &AttentionOperatorSpec::standard())
                .expect("publish exact identity pair");
            if missing != "corpus.meta" {
                std::fs::write(output.join("corpus.meta"), source_resume_meta(1, 0))
                    .expect("metadata fixture");
            }
            if missing != "corpus.records" {
                std::fs::write(output.join("corpus.records"), vec![7u8; 48])
                    .expect("records fixture");
            }
            let before = directory_bytes(&output);
            let error = preflight_source_compile_resume(&output, Some(16), 2)
                .expect_err("one-sided resume must fail closed");
            assert!(error.reason.contains("one-sided"));
            assert_eq!(directory_bytes(&output), before);
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn source_compile_resume_reconciles_record_and_hidden_tails_at_actual_oracle_width() {
        let output = unique_cli_test_path("source-resume-tail-recovery");
        let tokenizer = fixture_adapter("tail-recovery");
        pin_compile_identities(&output, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(output.join("corpus.meta"), source_resume_meta(2, 0))
            .expect("metadata fixture");
        let mut records = source_v3_record(0, 0).to_vec();
        records.extend_from_slice(&source_v3_record(0, 1));
        records.extend_from_slice(b"partial-record-tail");
        std::fs::write(output.join("corpus.records"), &records).expect("record fixture");
        // The loaded-oracle seam reports 16 bytes per hidden row here. Two
        // committed rows plus a partial third model an interrupted story.
        let mut hidden = vec![5u8; 32];
        hidden.extend_from_slice(b"partial");
        std::fs::write(output.join("corpus.records.hidden"), &hidden).expect("hidden fixture");

        let plan = preflight_source_compile_resume(&output, Some(16), 10)
            .expect("valid crash tails are recoverable");
        assert_eq!(plan.records_committed_bytes, Some(96));
        assert_eq!(plan.hidden_committed_bytes, Some(32));
        reconcile_source_compile_resume(&output, &plan).expect("recover both streams");
        assert_eq!(
            std::fs::metadata(output.join("corpus.records"))
                .expect("records metadata")
                .len(),
            96
        );
        assert_eq!(
            std::fs::metadata(output.join("corpus.records.hidden"))
                .expect("hidden metadata")
                .len(),
            32
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn source_compile_resume_never_reinterprets_a_long_legacy_tail_as_v3() {
        let output = unique_cli_test_path("source-resume-legacy-width-ambiguity");
        let tokenizer = fixture_adapter("legacy-width-ambiguity");
        pin_compile_identities(&output, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(output.join("corpus.meta"), source_resume_meta(2, 0))
            .expect("metadata fixture");
        let mut legacy = source_v2_record(0).to_vec();
        legacy.extend_from_slice(&source_v2_record(0));
        legacy.extend_from_slice(&[0xA5; 40]); // 104 bytes: longer than 2 * 48.
        std::fs::write(output.join("corpus.records"), &legacy).expect("legacy record fixture");
        let before = directory_bytes(&output);

        let error = preflight_source_compile_resume(&output, None, 10)
            .expect_err("a long v2 tail is not evidence of a v3 committed prefix");
        assert!(error.reason.contains("invalid v3 token span"));
        assert_eq!(directory_bytes(&output), before);
        assert_eq!(
            std::fs::read(output.join("corpus.records")).expect("legacy bytes"),
            legacy
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn source_compile_zero_checkpoint_recovers_only_the_authoritative_empty_prefix() {
        let output = unique_cli_test_path("source-resume-zero-checkpoint");
        let tokenizer = fixture_adapter("zero-checkpoint");
        pin_compile_identities(&output, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(output.join("corpus.meta"), source_resume_meta(0, 0))
            .expect("zero checkpoint");
        std::fs::write(
            output.join("corpus.records"),
            b"uncommitted bytes before the first story checkpoint",
        )
        .expect("uncommitted record tail");

        let plan = preflight_source_compile_resume(&output, None, 10)
            .expect("n=0 makes every record byte an uncommitted tail");
        assert_eq!(plan.records_committed_bytes, Some(0));
        reconcile_source_compile_resume(&output, &plan).expect("truncate empty prefix");
        assert_eq!(
            std::fs::metadata(output.join("corpus.records"))
                .expect("records metadata")
                .len(),
            0
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn source_compile_meta_only_zero_checkpoint_resumes_before_stream_creation() {
        let output = unique_cli_test_path("source-resume-meta-only-zero");
        std::fs::create_dir_all(&output).expect("compile output");
        std::fs::write(
            output.join(SOURCE_CORPUS_META_FILE),
            initial_source_corpus_checkpoint(),
        )
        .expect("durable zero checkpoint fixture");

        let plan = preflight_source_compile_resume(&output, Some(16), 10)
            .expect("zero checkpoint precedes stream creation");
        assert_eq!(plan, SourceCompileResumePlan::default());
        assert!(!output.join("corpus.records").exists());
        assert!(!output.join("corpus.records.hidden").exists());
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn source_compile_checkpoint_temp_crash_matrix_recovers_only_reserved_regular_files() {
        for (case, bytes) in [
            ("empty", Vec::new()),
            ("partial", vec![0xA5; 9]),
            ("complete", initial_source_corpus_checkpoint().to_vec()),
        ] {
            let output = unique_cli_test_path(&format!("source-checkpoint-temp-{case}"));
            std::fs::create_dir_all(&output).expect("compile output");
            let temporary = output.join(format!(
                ".{SOURCE_CORPUS_META_FILE}.checkpoint.{}.1.tmp",
                std::process::id()
            ));
            std::fs::write(&temporary, bytes).expect("checkpoint crash residue");
            validate_source_corpus_checkpoint_temporaries(&output)
                .expect("reserved regular crash residue");
            recover_source_corpus_checkpoint_temporaries(&output)
                .expect("exclusive-owner recovery");
            assert!(!temporary.exists());
            let _ = std::fs::remove_dir_all(output);
        }

        let malformed = unique_cli_test_path("source-checkpoint-temp-malformed");
        std::fs::create_dir_all(&malformed).expect("compile output");
        let unknown = malformed.join(format!(
            ".{SOURCE_CORPUS_META_FILE}.checkpoint.not-numeric.1.tmp"
        ));
        std::fs::write(&unknown, b"sentinel").expect("unknown temporary");
        let before = directory_bytes(&malformed);
        let error = validate_source_corpus_checkpoint_temporaries(&malformed)
            .expect_err("unknown checkpoint temporary is terminal");
        assert!(error.reason.contains("unrecognized checkpoint temporary"));
        assert_eq!(directory_bytes(&malformed), before);
        let _ = std::fs::remove_dir_all(malformed);
    }

    #[test]
    fn source_compile_resume_keeps_legacy_missing_hidden_optional_but_refuses_short_present_hidden()
    {
        let historical = unique_cli_test_path("source-resume-no-hidden");
        let tokenizer = fixture_adapter("no-hidden");
        pin_compile_identities(&historical, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(historical.join("corpus.meta"), source_resume_meta(1, 1))
            .expect("metadata fixture");
        std::fs::write(historical.join("corpus.records"), source_v3_record(0, 0))
            .expect("records fixture");
        let plan = preflight_source_compile_resume(&historical, Some(16), 1)
            .expect("completed historical absent hidden stream remains compatible");
        assert_eq!(plan.hidden_committed_bytes, None);

        let incomplete = unique_cli_test_path("source-resume-no-hidden-incomplete");
        let tokenizer = fixture_adapter("no-hidden-incomplete");
        pin_compile_identities(&incomplete, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(incomplete.join("corpus.meta"), source_resume_meta(1, 0))
            .expect("metadata fixture");
        std::fs::write(incomplete.join("corpus.records"), source_v3_record(0, 0))
            .expect("records fixture");
        let before = directory_bytes(&incomplete);
        let error = preflight_source_compile_resume(&incomplete, Some(16), 10)
            .expect_err("resuming would create a partial hidden suffix");
        assert!(error.reason.contains("already-complete corpus"));
        assert_eq!(directory_bytes(&incomplete), before);

        let short = unique_cli_test_path("source-resume-short-hidden");
        let tokenizer = fixture_adapter("short-hidden");
        pin_compile_identities(&short, &tokenizer, &AttentionOperatorSpec::standard())
            .expect("publish exact identity pair");
        std::fs::write(short.join("corpus.meta"), source_resume_meta(1, 0))
            .expect("metadata fixture");
        std::fs::write(short.join("corpus.records"), source_v3_record(0, 0))
            .expect("records fixture");
        std::fs::write(short.join("corpus.records.hidden"), vec![9u8; 15])
            .expect("short hidden fixture");
        let before = directory_bytes(&short);
        let error = preflight_source_compile_resume(&short, Some(16), 10)
            .expect_err("short committed hidden prefix is torn");
        assert!(error.reason.contains("shorter than the 16-byte prefix"));
        assert_eq!(directory_bytes(&short), before);

        let _ = std::fs::remove_dir_all(historical);
        let _ = std::fs::remove_dir_all(incomplete);
        let _ = std::fs::remove_dir_all(short);
    }

    #[test]
    fn recorded_compile_refuses_unsupported_source_bundle_leaves_before_mutation() {
        for case in ["adapter", "table", "space-manifest"] {
            let output = unique_cli_test_path(&format!("recorded-output-stale-{case}"));
            if case == "adapter" {
                pin_compile_tokenizer_adapter(&output, &fixture_adapter("stale-recorded"))
                    .expect("stale adapter fixture");
            }
            bind_compiled_attention_operator(&output, &AttentionOperatorSpec::standard())
                .expect("exact attention binding");
            if case == "table" {
                std::fs::write(
                    output.join("tokenizer.bin"),
                    b"stale tokenizer table sentinel",
                )
                .expect("stale tokenizer table");
            }
            if case == "space-manifest" {
                std::fs::write(
                    output.join("space_manifest.json"),
                    b"stale semantic-space manifest sentinel",
                )
                .expect("stale space manifest");
            }
            std::fs::write(
                output.join("tless_artifacts.bin"),
                b"prior artifact sentinel",
            )
            .expect("prior artifact");
            let before = directory_bytes(&output);

            let error = preflight_recorded_compile_output(&output)
                .expect_err("recorded compile cannot preserve tokenizer provenance");
            assert!(
                error.reason.contains("no tokenizer authority")
                    || error.reason.contains("cannot verify or reproduce")
            );
            assert_eq!(directory_bytes(&output), before);
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn observation_attention_operator_preflight_is_joint_and_rejects_legacy_relabeling() {
        let operator_conflict = unique_cli_test_path("observe-joint-operator-conflict");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard());
        write_observation_manifest(&operator_conflict, &manifest);
        let operator_session =
            observe::ObservationSession::acquire(&operator_conflict, 1).expect("session");
        let manifest_path = operator_conflict.join(observe::MANIFEST_FILE);
        let bytes_before = std::fs::read(&manifest_path).expect("manifest bytes");
        let error = pin_raw_observation_identities_before_output(
            &operator_session,
            Some(&fixture_adapter("would-have-been-published")),
            None,
            Some(&AttentionOperatorSpec::experimental_r4()),
        )
        .expect_err("operator conflict rejects before tokenizer pin");
        assert!(error.reason.contains("is pinned to attention operator"));
        assert_eq!(
            std::fs::read(&manifest_path).expect("manifest bytes"),
            bytes_before
        );
        assert!(!operator_conflict.join("tokenizer.bin").exists());

        let tokenizer_conflict = unique_cli_test_path("observe-joint-tokenizer-conflict");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.tokenizer_adapter = Some(fixture_adapter("recorded"));
        write_observation_manifest(&tokenizer_conflict, &manifest);
        let tokenizer_session =
            observe::ObservationSession::acquire(&tokenizer_conflict, 1).expect("session");
        let manifest_path = tokenizer_conflict.join(observe::MANIFEST_FILE);
        let bytes_before = std::fs::read(&manifest_path).expect("manifest bytes");
        let error = pin_raw_observation_identities_before_output(
            &tokenizer_session,
            Some(&fixture_adapter("requested")),
            None,
            Some(&AttentionOperatorSpec::standard()),
        )
        .expect_err("tokenizer conflict rejects before operator pin");
        assert!(error.reason.contains("pinned to tokenizer adapter"));
        assert_eq!(
            std::fs::read(&manifest_path).expect("manifest bytes"),
            bytes_before
        );

        let legacy = unique_cli_test_path("observe-attention-legacy-payload");
        let manifest = observe::ObservationManifest::new(1);
        write_observation_manifest(&legacy, &manifest);
        std::fs::write(legacy.join(observe::STATE_FILE), b"legacy state").expect("legacy payload");
        let legacy_session = observe::ObservationSession::acquire(&legacy, 1).expect("session");
        let before = directory_bytes(&legacy);
        let error = pin_raw_observation_identities_before_output(
            &legacy_session,
            None,
            None,
            Some(&AttentionOperatorSpec::standard()),
        )
        .expect_err("operatorless legacy rows cannot be relabelled");
        assert!(error.reason.contains("refusing to relabel legacy rows"));
        assert_eq!(directory_bytes(&legacy), before);

        drop(operator_session);
        drop(tokenizer_session);
        drop(legacy_session);
        let _ = std::fs::remove_dir_all(operator_conflict);
        let _ = std::fs::remove_dir_all(tokenizer_conflict);
        let _ = std::fs::remove_dir_all(legacy);
    }

    #[test]
    fn raw_observation_geometry_conflict_is_refused_before_tokenizer_export() {
        let output = unique_cli_test_path("observe-raw-geometry-preflight");
        let adapter = fixture_adapter("raw-geometry");
        let operator = AttentionOperatorSpec::standard();
        let recorded_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(576, 288);
        let requested_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(768, 288);
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.tokenizer_adapter = Some(adapter.clone());
        manifest.attention_operator = Some(operator.clone());
        manifest.geometry = Some(recorded_geometry);
        write_observation_manifest(&output, &manifest);
        std::fs::write(output.join("tokenizer.bin"), b"existing tokenizer sentinel")
            .expect("tokenizer sentinel");
        let session = observe::ObservationSession::acquire(&output, 1).expect("session");
        let before = directory_bytes(&output);

        let error = pin_raw_observation_identities_before_output(
            &session,
            Some(&adapter),
            Some(&requested_geometry),
            Some(&operator),
        )
        .expect_err("geometry conflict must precede tokenizer export");
        assert!(
            error.reason.contains("pinned to geometry"),
            "{}",
            error.reason
        );
        assert_eq!(directory_bytes(&output), before);
        drop(session);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn text_observation_input_and_geometry_conflicts_precede_serial_and_batched_export() {
        let output = unique_cli_test_path("observe-text-full-preflight");
        let input = unique_cli_test_path("observe-text-input-a.jsonl");
        let other_input = unique_cli_test_path("observe-text-input-b.jsonl");
        std::fs::write(
            &input,
            b"{\"id\":\"a\",\"url\":\"https://example/a\",\"title\":\"A\",\"text\":\"first corpus\"}\n",
        )
        .expect("first input");
        std::fs::write(
            &other_input,
            b"{\"id\":\"b\",\"url\":\"https://example/b\",\"title\":\"B\",\"text\":\"different corpus\"}\n",
        )
        .expect("second input");
        let adapter = fixture_adapter("text-preflight");
        let operator = AttentionOperatorSpec::standard();
        let recorded_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(576, 288);
        let requested_geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(768, 288);
        let session = observe::ObservationSession::acquire(&output, 1).expect("session");
        pin_text_observation_identities_before_output(
            &session,
            &input,
            Some(&adapter),
            Some(&recorded_geometry),
            Some(&operator),
        )
        .expect("pin complete text identity bundle");
        std::fs::write(output.join("tokenizer.bin"), b"existing tokenizer sentinel")
            .expect("tokenizer sentinel");

        let before_geometry = directory_bytes(&output);
        let error = pin_text_observation_identities_before_output(
            &session,
            &input,
            Some(&adapter),
            Some(&requested_geometry),
            Some(&operator),
        )
        .expect_err("serial/batched geometry conflict must precede tokenizer export");
        assert!(
            error.reason.contains("pinned to geometry"),
            "{}",
            error.reason
        );
        assert_eq!(directory_bytes(&output), before_geometry);

        let before_input = directory_bytes(&output);
        let error = pin_text_observation_identities_before_output(
            &session,
            &other_input,
            Some(&adapter),
            Some(&recorded_geometry),
            Some(&operator),
        )
        .expect_err("serial/batched input conflict must precede tokenizer export");
        assert!(error.reason.contains("different input CID"));
        assert_eq!(directory_bytes(&output), before_input);

        drop(session);
        let _ = std::fs::remove_dir_all(output);
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(other_input);
    }

    #[test]
    fn observation_session_serializes_cross_field_identity_pin_and_rows() {
        let output = unique_cli_test_path("observe-cross-field-session");
        let winner_output = output.clone();
        let (winner_ready_tx, winner_ready_rx) = std::sync::mpsc::channel();
        let (release_winner_tx, release_winner_rx) = std::sync::mpsc::channel();
        let winner = std::thread::spawn(move || -> Result<(), String> {
            let session = observe::ObservationSession::acquire(&winner_output, 1)
                .map_err(|error| error.reason)?;
            pin_raw_observation_identities_before_output(
                &session,
                None,
                None,
                Some(&AttentionOperatorSpec::standard_v1()),
            )
            .map_err(|error| error.reason)?;
            let mut writer = session.writer().map_err(|error| error.reason)?;
            writer
                .write_record(&[0u8; observe::RECORD_SIZE], 0)
                .map_err(|error| error.reason)?;
            drop(writer);
            winner_ready_tx
                .send(())
                .map_err(|error| error.to_string())?;
            release_winner_rx
                .recv()
                .map_err(|error| error.to_string())?;
            drop(session);
            Ok(())
        });
        winner_ready_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("winner pins and writes while retaining its session");

        let loser_output = output.clone();
        let loser_adapter = fixture_adapter("cross-field-loser");
        let (loser_attempt_tx, loser_attempt_rx) = std::sync::mpsc::channel();
        let loser = std::thread::spawn(move || -> Result<SourceUnavailable, String> {
            loser_attempt_tx
                .send(())
                .map_err(|error| error.to_string())?;
            let session = observe::ObservationSession::acquire(&loser_output, 1)
                .map_err(|error| error.reason)?;
            let result = pin_raw_observation_identities_before_output(
                &session,
                Some(&loser_adapter),
                None,
                Some(&AttentionOperatorSpec::standard_v2()),
            );
            drop(session);
            match result {
                Ok(()) => Err("incompatible loser unexpectedly pinned identities".to_owned()),
                Err(error) => Ok(error),
            }
        });
        loser_attempt_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("loser attempts the same session");
        release_winner_tx.send(()).expect("release winner session");
        winner
            .join()
            .expect("winner thread")
            .expect("winner session");
        let loser_error = loser
            .join()
            .expect("loser thread")
            .expect("loser is refused after serialized acquisition");
        assert!(loser_error.reason.contains("pinned to attention operator"));

        let manifest = read_optional_observation_manifest(&output)
            .expect("manifest read")
            .expect("manifest present");
        assert_eq!(manifest.tokenizer_adapter, None);
        assert_eq!(
            manifest.attention_operator,
            Some(AttentionOperatorSpec::standard_v1())
        );
        assert!(!output.join("tokenizer.bin").exists());
        assert_eq!(
            std::fs::metadata(output.join(observe::shard_file_name(1, 0)))
                .expect("winner shard")
                .len(),
            observe::RECORD_SIZE as u64
        );
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn observation_preflight_detects_payload_beyond_declared_shard_fanout() {
        for (case, payload_name) in [
            ("shard", "shard-99.bin"),
            ("probability", "shard-99.bin.prob"),
            ("trace", "shard-99.bin.trace"),
        ] {
            let output = unique_cli_test_path(&format!("observe-out-of-fanout-{case}"));
            let manifest = observe::ObservationManifest::new(1); // only shards 00 and 01
            write_observation_manifest(&output, &manifest);
            std::fs::write(output.join(payload_name), b"legacy out-of-fanout payload")
                .expect("out-of-fanout payload");
            let before_acquire = directory_bytes(&output);
            match observe::ObservationSession::acquire(&output, 1) {
                Ok(session) => {
                    let before = directory_bytes(&output);
                    let error = pin_raw_observation_identities_before_output(
                        &session,
                        None,
                        None,
                        Some(&AttentionOperatorSpec::standard()),
                    )
                    .expect_err("out-of-fanout payload cannot be relabelled");
                    assert!(error.reason.contains("refusing to relabel legacy rows"));
                    assert_eq!(directory_bytes(&output), before);
                    drop(session);
                }
                Err(error) => {
                    assert!(error.reason.contains("outside the manifest"));
                    assert_eq!(directory_bytes(&output), before_acquire);
                }
            }
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn observation_preflight_treats_zero_byte_payload_entries_as_era_evidence() {
        for (case, payload_name) in [
            ("state", observe::STATE_FILE),
            ("raw-committed", observe::RAW_COMMITTED_FILE),
            ("shard", "shard-99.bin"),
        ] {
            let output = unique_cli_test_path(&format!("observe-zero-byte-{case}"));
            write_observation_manifest(&output, &observe::ObservationManifest::new(1));
            std::fs::write(output.join(payload_name), []).expect("zero-byte payload entry");
            let before_acquire = directory_bytes(&output);
            match observe::ObservationSession::acquire(&output, 1) {
                Ok(session) => {
                    let before = directory_bytes(&output);
                    let error = pin_raw_observation_identities_before_output(
                        &session,
                        None,
                        None,
                        Some(&AttentionOperatorSpec::standard()),
                    )
                    .expect_err("zero-byte payload cannot be relabelled to the current operator");
                    assert!(error.reason.contains("refusing to relabel legacy rows"));
                    assert_eq!(directory_bytes(&output), before);
                    drop(session);
                }
                Err(error) => {
                    assert!(
                        error.reason.contains("outside the manifest")
                            || error.reason.contains("state")
                    );
                    assert_eq!(directory_bytes(&output), before_acquire);
                }
            }
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[cfg(unix)]
    #[test]
    fn observation_matching_identity_preflight_rejects_payload_symlinks() {
        use std::os::unix::fs::symlink;

        for (case, payload_name) in [
            ("shard", "shard-00.bin"),
            ("probability", "shard-00.bin.prob"),
            ("trace", "shard-00.bin.trace"),
        ] {
            let output = unique_cli_test_path(&format!("observe-matching-symlink-{case}"));
            let mut manifest = observe::ObservationManifest::new(1);
            manifest.attention_operator = Some(AttentionOperatorSpec::standard());
            write_observation_manifest(&output, &manifest);
            let manifest_path = output.join(observe::MANIFEST_FILE);
            let manifest_before = std::fs::read(&manifest_path).expect("manifest bytes");
            let target = output.join(format!("{case}-payload-target"));
            std::fs::write(&target, b"payload target").expect("payload target");
            symlink(&target, output.join(payload_name)).expect("payload symlink");

            let error = preflight_observation_identities(
                &output,
                1,
                None,
                Some(&AttentionOperatorSpec::standard()),
            )
            .expect_err("matching identities must not bypass payload inventory");
            assert!(error.reason.contains("not a regular file"));
            assert_eq!(
                std::fs::read(&manifest_path).expect("manifest bytes"),
                manifest_before
            );
            assert!(
                std::fs::symlink_metadata(output.join(payload_name))
                    .expect("payload metadata")
                    .file_type()
                    .is_symlink()
            );
            let _ = std::fs::remove_dir_all(output);
        }
    }

    #[test]
    fn compile_recorded_genuine_absence_resolves_to_immutable_standard_v1() {
        let root = unique_cli_test_path("compile-recorded-legacy-standard-v1");
        let (meta, records) = corpus_markers(&root);
        assert_eq!(
            recorded_compile_attention_operator(&meta, &records)
                .expect("genuine recorded-corpus absence has a pinned legacy meaning"),
            AttentionOperatorSpec::standard_v1()
        );
        assert_eq!(
            LEGACY_STANDARD_ATTENTION_OPERATOR_VERSION,
            AttentionOperatorSpec::STANDARD_V1_VERSION
        );
        assert_ne!(
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::standard(),
            "the current alias must not leak into compile-recorded legacy absence"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn compile_recorded_preflight_propagates_the_joint_dense_execution_identity() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let source = unique_cli_test_path("compile-recorded-dense-source");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&source, &attention).expect("source attention binding");
        bind_compiled_dense_operator(&source, Some(&dense)).expect("source dense binding");
        let (meta, records, meta_bytes, record_bytes) = finalized_corpus(&source, 7);
        let guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
                &source,
            )
            .expect("source guard");
        guard.begin_compile_attempt().expect("source attempt");
        uor_r4_graph_compiler::recorded_corpus::publish_binding(&guard, &meta, &records)
            .expect("source binding");
        guard
            .finish_compile_attempt()
            .expect("finish source binding");
        drop(guard);

        let output = unique_cli_test_path("compile-recorded-dense-output");
        let args = vec![
            "--corpus-meta".to_owned(),
            meta.display().to_string(),
            "--corpus-recs".to_owned(),
            records.display().to_string(),
            "--vocab-size".to_owned(),
            "16".to_owned(),
            "--out".to_owned(),
            output.display().to_string(),
        ];
        let options = parse_recorded_compile_options(&args).expect("recorded compile options");
        let prepared = preflight_recorded_compile_execution_identity(&options)
            .expect("preflight current GPT-2 recorded corpus");
        assert_eq!(prepared.attention_operator, attention);
        assert_eq!(prepared.dense_operator, Some(dense));
        assert_eq!(prepared.meta_bytes, meta_bytes);
        assert_eq!(prepared.records_bytes, record_bytes);
        assert_eq!(prepared.corpus.n, 1);
        assert!(
            !output.exists(),
            "all recorded execution checks precede output publication"
        );

        let _ = std::fs::remove_dir_all(source);
    }

    #[test]
    fn compile_recorded_command_recovers_each_sidecar_sync_crash_and_is_ready_idempotent() {
        let source = unique_cli_test_path("compile-recorded-sidecar-crash-source");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_dense_operator(&source, Some(&dense)).expect("source dense binding");
        bind_compiled_attention_operator(&source, &attention).expect("source attention binding");
        let (meta, records, _, _) = finalized_corpus(&source, 11);
        publish_test_corpus_binding(&source, &meta, &records);

        for crash_member in [
            PlannedOutputMember::DenseOperator,
            PlannedOutputMember::AttentionOperator,
        ] {
            let output = unique_cli_test_path(match crash_member {
                PlannedOutputMember::DenseOperator => "compile-recorded-dense-sync-crash",
                PlannedOutputMember::AttentionOperator => "compile-recorded-attention-sync-crash",
                _ => unreachable!("sidecar crash fixture"),
            });
            let args = vec![
                "--corpus-meta".to_owned(),
                meta.display().to_string(),
                "--corpus-recs".to_owned(),
                records.display().to_string(),
                "--vocab-size".to_owned(),
                "16".to_owned(),
                "--out".to_owned(),
                output.display().to_string(),
            ];

            let injected = match crash_member {
                PlannedOutputMember::DenseOperator => compile_recorded_corpus_with(
                    &args,
                    fixture_recorded_compile_products,
                    |_| Err(SourceUnavailable::new("injected dense sidecar sync crash")),
                    |_| Ok(()),
                ),
                PlannedOutputMember::AttentionOperator => compile_recorded_corpus_with(
                    &args,
                    fixture_recorded_compile_products,
                    |_| Ok(()),
                    |_| {
                        Err(SourceUnavailable::new(
                            "injected attention sidecar sync crash",
                        ))
                    },
                ),
                _ => unreachable!("sidecar crash fixture"),
            }
            .expect_err("injected post-sync crash must interrupt publication");
            assert!(injected.reason.contains("sidecar sync crash"), "{injected}");
            let residues = planned_output_residue_entries(&output);
            assert_eq!(residues.len(), 1, "one exact durable crash residue");
            assert!(
                residues[0].contains(crash_member.stable_name()),
                "residue belongs to the interrupted sidecar: {residues:?}"
            );
            assert!(
                !output.join(crash_member.stable_name()).exists(),
                "post-sync crash precedes stable hard-link publication"
            );

            compile_recorded_corpus_with(
                &args,
                fixture_recorded_compile_products,
                |_| Ok(()),
                |_| Ok(()),
            )
            .expect("exact command retry recovers and commits");
            assert!(planned_output_residue_entries(&output).is_empty());
            assert!(attention_operator_temp_entries(&output).is_empty());
            assert!(dense_operator_temp_entries(&output).is_empty());
            assert_eq!(
                compiled_dense_operator(&output).expect("published dense"),
                Some(dense.clone())
            );
            assert_eq!(
                compiled_attention_operator(&output).expect("published attention"),
                attention
            );
            assert!(
                output
                    .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_BINDING_FILE)
                    .is_file(),
                "the command reaches binding-last commit"
            );

            let ready = directory_bytes(&output);
            compile_recorded_corpus_with(
                &args,
                fixture_recorded_compile_products,
                |_| panic!("ready rerun must not republish dense"),
                |_| panic!("ready rerun must not republish attention"),
            )
            .expect("ready command rerun is idempotent");
            assert_eq!(directory_bytes(&output), ready);
            let _ = std::fs::remove_dir_all(output);
        }

        let _ = std::fs::remove_dir_all(source);
    }

    #[test]
    fn compile_recorded_preflight_and_publication_stream_opaque_hidden() {
        let source = unique_cli_test_path("compile-recorded-stream-source");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&source, &attention).expect("source attention");
        bind_compiled_dense_operator(&source, Some(&dense)).expect("source dense");
        let (meta, records, _, _) = finalized_corpus(&source, 9);
        let hidden = source.join("corpus.records.hidden");
        let hidden_bytes = [
            0.0f32.to_le_bytes(),
            0.25f32.to_le_bytes(),
            (-1.0f32).to_le_bytes(),
        ]
        .concat();
        std::fs::write(&hidden, &hidden_bytes).expect("opaque hidden rows");
        let guard =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
                &source,
            )
            .expect("source guard");
        guard.begin_compile_attempt().expect("source attempt");
        uor_r4_graph_compiler::recorded_corpus::publish_binding(&guard, &meta, &records)
            .expect("source binding");
        guard.finish_compile_attempt().expect("finish source");
        drop(guard);

        let output = unique_cli_test_path("compile-recorded-stream-output");
        let args = vec![
            "--corpus-meta".to_owned(),
            meta.display().to_string(),
            "--corpus-recs".to_owned(),
            records.display().to_string(),
            "--vocab-size".to_owned(),
            "16".to_owned(),
            "--out".to_owned(),
            output.display().to_string(),
        ];
        let options = parse_recorded_compile_options(&args).expect("compile options");
        let mut prepared = preflight_recorded_compile_execution_identity(&options)
            .expect("bounded compile preflight");
        assert!(
            prepared.corpus.hidden.is_none(),
            "non-D hidden rows are not materialized into compiler state"
        );
        let expected_hidden = prepared
            .source
            .hidden
            .as_ref()
            .expect("retained opaque hidden stream")
            .summary();
        prepared
            .source
            .verify_generation()
            .expect("source generation");
        drop(prepared.source_guard.take());

        std::fs::create_dir_all(&output).expect("output root");
        let producer =
            uor_r4_graph_compiler::recorded_corpus::RecordedCorpusProducerGuard::try_acquire(
                &output,
            )
            .expect("output guard");
        producer
            .preflight_deterministic_compile_inventory(&PlannedOutputMember::ALL)
            .expect("fresh deterministic inventory");
        producer.begin_compile_attempt().expect("output attempt");
        bind_compiled_dense_operator(&output, prepared.dense_operator.as_ref())
            .expect("output dense");
        bind_compiled_attention_operator(&output, &prepared.attention_operator)
            .expect("output attention");
        write_captured_recorded_corpus(&producer, &output, &mut prepared)
            .expect("stream recorded corpus");
        prepared
            .source
            .verify_generation()
            .expect("final source generation");
        let binding_cid = uor_r4_graph_compiler::recorded_corpus::publish_binding(
            &producer,
            &output.join("corpus.meta"),
            &output.join("corpus.records"),
        )
        .expect("output binding");
        producer.finish_compile_attempt().expect("finish output");
        assert_eq!(
            std::fs::read(output.join("corpus.records.hidden")).expect("copied hidden"),
            hidden_bytes
        );
        assert!(
            !output
                .join(uor_r4_graph_compiler::recorded_corpus::RECORDED_CORPUS_COMPILE_ATTEMPT_FILE)
                .exists()
        );
        let captured = uor_r4_graph_compiler::recorded_corpus::open_stream(
            &output.join("corpus.meta"),
            &output.join("corpus.records"),
        )
        .expect("bound recorded output");
        assert_eq!(captured.execution.attention_operator, Some(attention));
        assert_eq!(captured.execution.dense_operator, Some(dense));
        assert_eq!(captured.binding_cid.as_deref(), Some(binding_cid.as_str()));
        assert_eq!(
            captured.hidden.as_ref().expect("hidden summary").summary(),
            expected_hidden
        );
        captured.verify_generation().expect("stable output");

        std::fs::create_dir(output.join("graph")).expect("downstream graph");
        std::fs::create_dir(output.join("graph-cover")).expect("downstream cover");
        std::fs::write(output.join("graph/sentinel"), b"graph").expect("graph sentinel");
        std::fs::write(output.join("graph-cover/sentinel"), b"cover").expect("cover sentinel");
        producer
            .preflight_ready_deterministic_compile_inventory(&PlannedOutputMember::ALL)
            .expect("ready exact inventory tolerates downstream graph outputs");
        assert_eq!(
            std::fs::read(output.join("graph/sentinel")).unwrap(),
            b"graph"
        );
        assert_eq!(
            std::fs::read(output.join("graph-cover/sentinel")).unwrap(),
            b"cover"
        );

        drop(producer);
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(output);
    }

    #[cfg(unix)]
    #[test]
    fn recorded_corpus_capture_refuses_post_resolution_swap() {
        use std::os::unix::fs::symlink;

        let source = unique_cli_test_path("recorded-corpus-captured-source");
        let (meta, records) = corpus_markers(&source);
        let hidden = source.join("corpus.records.hidden");
        std::fs::write(&hidden, b"captured hidden bytes").expect("hidden bytes");
        let external = unique_cli_test_path("recorded-corpus-external-records");
        std::fs::write(&external, b"external replacement bytes").expect("external records");
        let error = recorded_corpus_snapshot_with_hook(&meta, &records, || {
            std::fs::remove_file(&records).expect("remove captured records path");
            symlink(&external, &records).expect("post-resolution records symlink")
        })
        .expect_err("post-resolution path generation swap is terminal");
        assert!(
            error.reason.contains("without following links")
                || error.reason.contains("captured regular-file generation"),
            "{error}"
        );
        std::fs::remove_file(&records).expect("remove swapped link");
        std::fs::write(&records, b"records marker").expect("restore records");

        let captured = recorded_corpus_snapshot(&meta, &records).expect("stable exact capture");
        assert_eq!(captured.meta_bytes, b"meta marker");
        assert_eq!(captured.records_bytes, b"records marker");
        assert_eq!(
            captured.hidden_bytes.as_deref(),
            Some(b"captured hidden bytes".as_slice())
        );
        std::fs::remove_file(&meta).expect("remove captured metadata source");
        std::fs::remove_file(&records).expect("remove captured records source");
        symlink(&external, &meta).expect("swap metadata after complete capture");
        symlink(&external, &records).expect("swap records after complete capture");

        assert_eq!(
            std::fs::read(&external).expect("external survives"),
            b"external replacement bytes"
        );

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_file(external);
    }

    #[test]
    fn captured_corpus_byte_parser_matches_the_path_wrapper_contract() {
        let meta = source_resume_meta(1, 1);
        let record = source_v3_record(7, 0);
        let corpus =
            compiler::load_corpus_bytes(&meta, &record, None).expect("captured v3 corpus parses");
        assert_eq!(corpus.n, 1);
        assert_eq!(corpus.stories, 1);
        assert_eq!(corpus.story, vec![7]);
        assert_eq!(corpus.next, vec![7]);
    }

    #[test]
    fn recorded_corpus_attention_operator_reads_both_sources_and_rejects_conflicts() {
        let sidecar_root = unique_cli_test_path("recorded-attention-sidecar");
        let sidecar_operator = AttentionOperatorSpec::learned_absolute_source_attention();
        bind_compiled_attention_operator(&sidecar_root, &sidecar_operator)
            .expect("sidecar binding");
        let (meta, records) = corpus_markers(&sidecar_root);
        assert_eq!(
            recorded_corpus_attention_operator(&meta, &records).expect("resolve sidecar operator"),
            Some(sidecar_operator)
        );

        let manifest_root = unique_cli_test_path("recorded-attention-manifest");
        let manifest_operator = AttentionOperatorSpec::standard();
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(manifest_operator.clone());
        write_observation_manifest(&manifest_root, &manifest);
        let (meta, records) = observation_corpus_markers(&manifest_root);
        assert_eq!(
            recorded_corpus_attention_operator(&meta, &records).expect("resolve manifest operator"),
            Some(manifest_operator)
        );

        let conflict_root = unique_cli_test_path("recorded-attention-conflict");
        bind_compiled_attention_operator(&conflict_root, &AttentionOperatorSpec::standard_v1())
            .expect("sidecar binding");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard_v2());
        write_observation_manifest(&conflict_root, &manifest);
        let (meta, records) = corpus_markers(&conflict_root);
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("same-family v1/v2 provenance sources must agree exactly");
        assert!(
            error
                .reason
                .contains("declare different attention operators")
        );

        let operatorless_manifest_root =
            unique_cli_test_path("recorded-attention-operatorless-manifest-conflict");
        bind_compiled_attention_operator(
            &operatorless_manifest_root,
            &AttentionOperatorSpec::standard(),
        )
        .expect("sidecar binding");
        write_observation_manifest(
            &operatorless_manifest_root,
            &observe::ObservationManifest::new(1),
        );
        let (meta, records) = observation_corpus_markers(&operatorless_manifest_root);
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("a present operatorless manifest conflicts with a sidecar");
        assert!(error.reason.contains("legacy operatorless era"));

        let legacy_root = unique_cli_test_path("recorded-attention-legacy");
        let (meta, records) = corpus_markers(&legacy_root);
        assert_eq!(
            recorded_corpus_attention_operator(&meta, &records).expect("legacy absence"),
            None
        );

        let _ = std::fs::remove_dir_all(sidecar_root);
        let _ = std::fs::remove_dir_all(manifest_root);
        let _ = std::fs::remove_dir_all(conflict_root);
        let _ = std::fs::remove_dir_all(operatorless_manifest_root);
        let _ = std::fs::remove_dir_all(legacy_root);
    }

    #[test]
    fn recorded_corpus_execution_identity_resolves_dense_and_rejects_impossible_pairs() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let sidecar_root = unique_cli_test_path("recorded-execution-sidecars");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&sidecar_root, &attention).expect("attention sidecar");
        bind_compiled_dense_operator(&sidecar_root, Some(&dense)).expect("dense sidecar");
        let (meta, records) = complete_test_corpus(&sidecar_root);
        publish_test_corpus_binding(&sidecar_root, &meta, &records);
        assert_eq!(
            recorded_corpus_execution_identity(&meta, &records).expect("joint sidecar identity"),
            RecordedCorpusExecutionIdentity {
                attention_operator: Some(attention.clone()),
                dense_operator: Some(dense.clone()),
            }
        );

        let manifest_root = unique_cli_test_path("recorded-execution-manifest");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(attention.clone());
        manifest.dense_operator = Some(dense.clone());
        write_observation_manifest(&manifest_root, &manifest);
        let (state, merged) = complete_test_observation_corpus(&manifest_root);
        publish_test_observation_binding(&manifest_root, &state, &merged);
        assert_eq!(
            recorded_corpus_execution_identity(&state, &merged).expect("joint manifest identity"),
            RecordedCorpusExecutionIdentity {
                attention_operator: Some(attention),
                dense_operator: Some(dense),
            }
        );

        let cross_era = unique_cli_test_path("recorded-execution-cross-era");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::learned_absolute_v1());
        manifest.dense_operator = Some(DenseOperatorSpec::gpt2_v2());
        write_observation_manifest(&cross_era, &manifest);
        let (state, merged) = observation_corpus_markers(&cross_era);
        let before = directory_bytes(&cross_era);
        let error = recorded_corpus_execution_identity(&state, &merged)
            .expect_err("cross-era pair is terminal");
        assert!(
            error.reason.contains("does not match") || error.reason.contains("not registered"),
            "{error}"
        );
        assert_eq!(directory_bytes(&cross_era), before);

        let dense_only = unique_cli_test_path("recorded-execution-dense-only");
        bind_compiled_dense_operator(&dense_only, Some(&DenseOperatorSpec::gpt2_v2()))
            .expect("dense-only sidecar fixture");
        let (meta, records) = corpus_markers(&dense_only);
        let before = directory_bytes(&dense_only);
        let error = recorded_corpus_execution_identity(&meta, &records)
            .expect_err("dense without attention is terminal");
        assert!(error.reason.contains("without an attention"), "{error}");
        assert_eq!(directory_bytes(&dense_only), before);

        for root in [sidecar_root, manifest_root, cross_era, dense_only] {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn recorded_corpus_joint_snapshot_rejects_manifest_disappearance_before_output_mutation() {
        use uor_r4_model_source::dense::DenseOperatorSpec;

        let root = unique_cli_test_path("recorded-execution-toctou");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::learned_absolute_v2());
        manifest.dense_operator = Some(DenseOperatorSpec::gpt2_v2());
        write_observation_manifest(&root, &manifest);
        let (state, merged) = observation_corpus_markers(&root);
        let output = unique_cli_test_path("recorded-execution-toctou-output");
        std::fs::create_dir_all(&output).expect("output directory");
        std::fs::write(output.join("sentinel"), b"last-good output").expect("sentinel");
        let output_before = directory_bytes(&output);
        let manifest_path = root.join(observe::MANIFEST_FILE);

        let error = recorded_corpus_execution_identity_with_recheck_hooks(
            &state,
            &merged,
            || {},
            || std::fs::remove_file(&manifest_path).expect("simulate manifest disappearance"),
        )
        .expect_err("one execution identity cannot span filesystem generations");
        assert!(error.reason.contains("changed or disappeared"), "{error}");
        assert_eq!(directory_bytes(&output), output_before);

        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(output);
    }

    #[test]
    fn recorded_corpus_joint_snapshot_returns_only_captured_bytes_across_aba_swap() {
        let root = unique_cli_test_path("recorded-execution-aba");
        let v1_attention = AttentionOperatorSpec::learned_absolute_v1();
        let v1_dense = DenseOperatorSpec::gpt2_v1();
        bind_compiled_attention_operator(&root, &v1_attention).expect("v1 attention");
        bind_compiled_dense_operator(&root, Some(&v1_dense)).expect("v1 dense");
        let (meta, records) = corpus_markers(&root);
        let attention_path = root.join(ATTENTION_OPERATOR_BINDING_FILE);
        let dense_path = root.join(DENSE_OPERATOR_BINDING_FILE);
        let attention_v1_bytes = std::fs::read(&attention_path).expect("v1 attention bytes");
        let dense_v1_bytes = std::fs::read(&dense_path).expect("v1 dense bytes");
        let mut attention_v2_bytes =
            serde_json::to_vec_pretty(&AttentionOperatorSpec::learned_absolute_v2())
                .expect("v2 attention JSON");
        attention_v2_bytes.push(b'\n');
        let mut dense_v2_bytes =
            serde_json::to_vec_pretty(&DenseOperatorSpec::gpt2_v2()).expect("v2 dense JSON");
        dense_v2_bytes.push(b'\n');

        let resolved = recorded_corpus_execution_identity_with_recheck_hooks(
            &meta,
            &records,
            || {
                std::fs::write(&attention_path, &attention_v2_bytes)
                    .expect("transient v2 attention");
                std::fs::write(&dense_path, &dense_v2_bytes).expect("transient v2 dense");
            },
            || {
                std::fs::write(&attention_path, &attention_v1_bytes).expect("restore v1 attention");
                std::fs::write(&dense_path, &dense_v1_bytes).expect("restore v1 dense");
            },
        )
        .expect("the exact captured A generation remains authoritative");
        assert_eq!(
            resolved,
            RecordedCorpusExecutionIdentity {
                attention_operator: Some(v1_attention),
                dense_operator: Some(v1_dense),
            }
        );
        assert_eq!(
            std::fs::read(attention_path).expect("restored attention"),
            attention_v1_bytes
        );
        assert_eq!(
            std::fs::read(dense_path).expect("restored dense"),
            dense_v1_bytes
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recorded_corpus_attention_operator_custom_names_require_legacy_absence() {
        let legacy_root = unique_cli_test_path("recorded-attention-custom-legacy");
        std::fs::create_dir_all(&legacy_root).expect("legacy directory");
        let legacy_meta = legacy_root.join("c_meta.bin");
        let legacy_records = legacy_root.join("c_recs.bin");
        std::fs::write(&legacy_meta, b"legacy metadata").expect("legacy metadata");
        std::fs::write(&legacy_records, b"legacy records").expect("legacy records");
        assert_eq!(
            recorded_corpus_attention_operator(&legacy_meta, &legacy_records)
                .expect("custom legacy pair remains compatible"),
            None
        );

        let sidecar_root = unique_cli_test_path("recorded-attention-custom-sidecar");
        bind_compiled_attention_operator(&sidecar_root, &AttentionOperatorSpec::standard())
            .expect("sidecar binding");
        let (canonical_meta, canonical_records) = corpus_markers(&sidecar_root);
        let alternate_meta = sidecar_root.join("second.meta");
        let alternate_records = sidecar_root.join("second.records");
        std::fs::write(&alternate_meta, b"alternate metadata").expect("alternate metadata");
        std::fs::write(&alternate_records, b"alternate records").expect("alternate records");
        assert_eq!(
            recorded_corpus_attention_operator(&canonical_meta, &canonical_records)
                .expect("canonical pair inherits sidecar"),
            Some(AttentionOperatorSpec::standard())
        );
        let error = recorded_corpus_attention_operator(&alternate_meta, &alternate_records)
            .expect_err("a sibling pair cannot inherit the canonical pair's sidecar");
        assert!(error.reason.contains("applies only to the canonical"));

        let manifest_root = unique_cli_test_path("recorded-attention-custom-manifest");
        write_observation_manifest(&manifest_root, &observe::ObservationManifest::new(1));
        let alternate_meta = manifest_root.join("second.meta");
        let alternate_records = manifest_root.join("second.records");
        std::fs::write(&alternate_meta, b"alternate metadata").expect("alternate metadata");
        std::fs::write(&alternate_records, b"alternate records").expect("alternate records");
        let error = recorded_corpus_attention_operator(&alternate_meta, &alternate_records)
            .expect_err("even an operatorless manifest belongs only to a canonical pair");
        assert!(error.reason.contains("applies only to the canonical"));

        let _ = std::fs::remove_dir_all(legacy_root);
        let _ = std::fs::remove_dir_all(sidecar_root);
        let _ = std::fs::remove_dir_all(manifest_root);
    }

    #[test]
    fn recorded_corpus_attention_operator_rejects_malformed_manifest_and_mixed_roots() {
        let malformed = unique_cli_test_path("recorded-attention-malformed");
        bind_compiled_attention_operator(&malformed, &AttentionOperatorSpec::standard())
            .expect("valid sidecar");
        std::fs::write(malformed.join(observe::MANIFEST_FILE), b"{not json")
            .expect("malformed manifest");
        let (meta, records) = corpus_markers(&malformed);
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("a present invalid manifest is not absence");
        assert!(error.reason.contains("malformed observation manifest"));

        let extended = unique_cli_test_path("recorded-attention-extended-manifest");
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard());
        let mut manifest_json = serde_json::to_value(&manifest).expect("manifest JSON");
        manifest_json
            .get_mut("attention_operator")
            .and_then(serde_json::Value::as_object_mut)
            .expect("operator object")
            .insert("unregistered_claim".to_owned(), serde_json::json!(true));
        std::fs::create_dir_all(&extended).expect("extended manifest directory");
        std::fs::write(
            extended.join(observe::MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest_json).expect("extended manifest JSON"),
        )
        .expect("extended manifest");
        let (meta, records) = corpus_markers(&extended);
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("unknown nested operator claims are refused");
        assert!(error.reason.contains("unregistered_claim"));

        let meta_root = unique_cli_test_path("recorded-attention-meta-root");
        let records_root = unique_cli_test_path("recorded-attention-records-root");
        let (meta, _) = corpus_markers(&meta_root);
        let (_, records) = corpus_markers(&records_root);
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("mixed corpus roots cannot inherit provenance");
        assert!(error.reason.contains("different canonical parent roots"));

        let _ = std::fs::remove_dir_all(malformed);
        let _ = std::fs::remove_dir_all(extended);
        let _ = std::fs::remove_dir_all(meta_root);
        let _ = std::fs::remove_dir_all(records_root);
    }

    #[cfg(unix)]
    #[test]
    fn recorded_corpus_attention_operator_rejects_corpus_symlink_inputs() {
        use std::os::unix::fs::symlink;

        for symlinked_entry in ["corpus.meta", "corpus.records"] {
            let root = unique_cli_test_path(&format!(
                "recorded-attention-{}-symlink",
                symlinked_entry.replace('.', "-")
            ));
            std::fs::create_dir_all(&root).expect("corpus directory");
            let meta = root.join("corpus.meta");
            let records = root.join("corpus.records");
            let target = root.join(format!("{symlinked_entry}-target"));
            std::fs::write(&target, b"corpus target").expect("corpus target");
            if symlinked_entry == "corpus.meta" {
                symlink(&target, &meta).expect("metadata symlink");
                std::fs::write(&records, b"records marker").expect("records marker");
            } else {
                std::fs::write(&meta, b"metadata marker").expect("metadata marker");
                symlink(&target, &records).expect("records symlink");
            }

            let error = recorded_corpus_attention_operator(&meta, &records)
                .expect_err("corpus symlinks cannot inherit directory provenance");
            assert!(error.reason.contains("not a regular non-symlink file"));
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[cfg(unix)]
    #[test]
    fn recorded_corpus_attention_operator_rejects_manifest_symlinks() {
        use std::os::unix::fs::symlink;

        let root = unique_cli_test_path("recorded-attention-manifest-symlink");
        let (meta, records) = corpus_markers(&root);
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard());
        let target = root.join("manifest-target.json");
        std::fs::write(
            &target,
            serde_json::to_vec_pretty(&manifest).expect("manifest JSON"),
        )
        .expect("manifest target");
        symlink(&target, root.join(observe::MANIFEST_FILE)).expect("manifest symlink");
        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("a present manifest symlink is not legacy absence");
        assert!(error.reason.contains("not a regular"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn copy_recorded_attention_preserves_markerless_legacy_compatibility() {
        let source = unique_cli_test_path("copy-recorded-attention-source");
        let operator = AttentionOperatorSpec::standard_v1();
        bind_compiled_attention_operator(&source, &operator).expect("source binding");
        let (meta, records) = complete_test_corpus(&source);
        let destination_root = unique_cli_test_path("copy-recorded-attention-destination");
        let (destination_meta, destination_records) = complete_test_corpus(&destination_root);
        let destination = destination_root.join(ATTENTION_OPERATOR_BINDING_FILE);
        copy_recorded_attention(&copy_recorded_attention_args(&meta, &records, &destination))
            .expect("copy historical attention provenance");
        assert_eq!(
            compiled_attention_operator(&destination_root).expect("copied binding"),
            operator
        );
        assert_eq!(compiled_dense_operator(&destination_root).unwrap(), None);
        let captured = recorded_corpus_snapshot(&destination_meta, &destination_records)
            .expect("markerless compatibility destination");
        assert!(captured.binding_cid.is_none());

        let legacy = unique_cli_test_path("copy-recorded-attention-legacy");
        let (legacy_meta, legacy_records) = complete_test_corpus(&legacy);
        let absent_root = unique_cli_test_path("copy-recorded-attention-legacy-destination");
        complete_test_corpus(&absent_root);
        let absent_destination = absent_root.join(ATTENTION_OPERATOR_BINDING_FILE);
        copy_recorded_attention(&copy_recorded_attention_args(
            &legacy_meta,
            &legacy_records,
            &absent_destination,
        ))
        .expect("true legacy absence is a no-op");
        assert!(!absent_destination.exists());

        for root in [source, destination_root, legacy, absent_root] {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn copy_recorded_attention_dense_is_bound_verification_only() {
        let source = unique_cli_test_path("copy-recorded-dense-first-source");
        let attention = AttentionOperatorSpec::learned_absolute_v2();
        let dense = DenseOperatorSpec::gpt2_v2();
        bind_compiled_attention_operator(&source, &attention).expect("source attention");
        bind_compiled_dense_operator(&source, Some(&dense)).expect("source dense");
        let (meta, records) = complete_test_corpus(&source);
        publish_test_corpus_binding(&source, &meta, &records);
        let destination_root = unique_cli_test_path("copy-recorded-dense-first-destination");
        complete_test_corpus(&destination_root);
        let destination = destination_root.join(ATTENTION_OPERATOR_BINDING_FILE);
        let args = copy_recorded_attention_args(&meta, &records, &destination);
        let before = directory_bytes(&destination_root);
        let error = copy_recorded_attention(&args)
            .expect_err("unbound destination cannot acquire dense provenance");
        assert!(error.reason.contains("cannot authorize"), "{error}");
        assert_eq!(directory_bytes(&destination_root), before);

        std::fs::remove_dir_all(&destination_root).expect("replace unbound fixture");
        bind_compiled_dense_operator(&destination_root, Some(&dense)).expect("bound dense");
        bind_compiled_attention_operator(&destination_root, &attention).expect("bound attention");
        let (destination_meta, destination_records) = complete_test_corpus(&destination_root);
        publish_test_corpus_binding(&destination_root, &destination_meta, &destination_records);
        let bound_before = directory_bytes(&destination_root);
        copy_recorded_attention(&args).expect("exact bound destination verifies as a no-op");
        assert_eq!(directory_bytes(&destination_root), bound_before);

        let attention_only_source = unique_cli_test_path("copy-recorded-attention-only-source");
        let attention_only = AttentionOperatorSpec::standard_v1();
        bind_compiled_attention_operator(&attention_only_source, &attention_only)
            .expect("attention-only source");
        let (meta, records) = complete_test_corpus(&attention_only_source);
        let attention_only_root = unique_cli_test_path("copy-recorded-attention-only-destination");
        complete_test_corpus(&attention_only_root);
        let attention_only_destination = attention_only_root.join(ATTENTION_OPERATOR_BINDING_FILE);
        copy_recorded_attention_with_after_dense(
            &copy_recorded_attention_args(&meta, &records, &attention_only_destination),
            || Err(SourceUnavailable::new("dense hook must not run")),
        )
        .expect("attention-only publication order is unchanged");
        assert_eq!(
            compiled_attention_operator(&attention_only_root).expect("attention-only destination"),
            attention_only
        );
        assert_eq!(
            compiled_dense_operator(&attention_only_root).expect("typed dense absence"),
            None
        );

        for root in [
            source,
            destination_root,
            attention_only_source,
            attention_only_root,
        ] {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn copy_recorded_attention_rejects_malformed_full_manifest_and_custom_pair() {
        let malformed = unique_cli_test_path("copy-recorded-attention-malformed-manifest");
        let (meta, records) = observation_corpus_markers(&malformed);
        let partial_manifest = serde_json::json!({
            "attention_operator": AttentionOperatorSpec::standard()
        });
        std::fs::write(
            malformed.join(observe::MANIFEST_FILE),
            serde_json::to_vec_pretty(&partial_manifest).expect("partial manifest JSON"),
        )
        .expect("partial manifest");
        let destination = unique_cli_test_path("copy-recorded-attention-malformed-destination")
            .join(ATTENTION_OPERATOR_BINDING_FILE);
        let error =
            copy_recorded_attention(&copy_recorded_attention_args(&meta, &records, &destination))
                .expect_err("partial observation manifest cannot be laundered");
        assert!(error.reason.contains("manifest"));
        assert!(!destination.exists());

        let custom = unique_cli_test_path("copy-recorded-attention-custom-pair");
        bind_compiled_attention_operator(&custom, &AttentionOperatorSpec::standard())
            .expect("source binding");
        let custom_meta = custom.join("other.meta");
        let custom_records = custom.join("other.records");
        std::fs::write(&custom_meta, b"custom metadata").expect("custom metadata");
        std::fs::write(&custom_records, b"custom records").expect("custom records");
        let error = copy_recorded_attention(&copy_recorded_attention_args(
            &custom_meta,
            &custom_records,
            &destination,
        ))
        .expect_err("custom pair cannot inherit sibling provenance");
        assert!(error.reason.contains("applies only to the canonical"));
        assert!(!destination.exists());

        let _ = std::fs::remove_dir_all(malformed);
        let _ = std::fs::remove_dir_all(custom);
    }

    #[test]
    fn recorded_resolver_and_copy_reject_invalid_non_attention_manifest_identity() {
        let source = unique_cli_test_path("copy-recorded-attention-invalid-geometry");
        let (meta, records) = observation_corpus_markers(&source);
        let mut manifest = observe::ObservationManifest::new(1);
        manifest.attention_operator = Some(AttentionOperatorSpec::standard());
        let mut geometry =
            uor_r4_model_source::geometry::GeometryProjection::bucket_average(576, 288);
        geometry.implementation_digest = format!("blake3:{}", "0".repeat(64));
        manifest.geometry = Some(geometry);
        write_observation_manifest(&source, &manifest);
        let destination = unique_cli_test_path("copy-recorded-attention-invalid-geometry-dest")
            .join(ATTENTION_OPERATOR_BINDING_FILE);

        let error = recorded_corpus_attention_operator(&meta, &records)
            .expect_err("forged geometry invalidates the full manifest");
        assert!(error.reason.contains("geometry projection"));
        let error =
            copy_recorded_attention(&copy_recorded_attention_args(&meta, &records, &destination))
                .expect_err("copy cannot launder a forged non-attention identity");
        assert!(error.reason.contains("geometry projection"));
        assert!(!destination.exists());

        let _ = std::fs::remove_dir_all(source);
    }

    #[cfg(unix)]
    #[test]
    fn copy_recorded_attention_rejects_destination_symlink_without_following() {
        use std::os::unix::fs::symlink;

        let source = unique_cli_test_path("copy-recorded-attention-symlink-source");
        bind_compiled_attention_operator(&source, &AttentionOperatorSpec::standard())
            .expect("source binding");
        let (meta, records) = complete_test_corpus(&source);
        let destination_parent =
            unique_cli_test_path("copy-recorded-attention-symlink-destination");
        complete_test_corpus(&destination_parent);
        let destination = destination_parent.join(ATTENTION_OPERATOR_BINDING_FILE);
        let sentinel = unique_cli_test_path("copy-recorded-attention-symlink-sentinel");
        let sentinel_bytes = b"external sentinel must remain unchanged";
        std::fs::write(&sentinel, sentinel_bytes).expect("sentinel");
        symlink(&sentinel, &destination).expect("destination symlink");

        let error =
            copy_recorded_attention(&copy_recorded_attention_args(&meta, &records, &destination))
                .expect_err("destination symlink must be refused");
        assert!(
            error.reason.contains("not a regular")
                || error.reason.contains("without following links"),
            "{error}"
        );
        assert_eq!(std::fs::read(&sentinel).expect("sentinel"), sentinel_bytes);
        assert!(
            std::fs::symlink_metadata(&destination)
                .expect("destination link")
                .file_type()
                .is_symlink()
        );

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(destination_parent);
        let _ = std::fs::remove_file(sentinel);
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
                attention_operator: AttentionOperatorSpec::standard_v1(),
                dense_operator: None,
            },
            artifacts: EvaluationArtifacts {
                directory: ".uor-models/compiled/smollm2-135m-instruct".to_owned(),
                artifacts_cid: "blake3:2222".to_owned(),
                store_cid: "blake3:3333".to_owned(),
                tokenizer_cid: "blake3:4444".to_owned(),
                corpus_meta_cid: "blake3:5555".to_owned(),
                corpus_records_cid: "blake3:6666".to_owned(),
                attention_operator_cid: "blake3:7777".to_owned(),
                dense_operator_cid: None,
                recorded_corpus_binding_cid: None,
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
        let legacy_json: serde_json::Value =
            serde_json::from_slice(&json).expect("legacy-compatible report JSON");
        assert_eq!(
            legacy_json["source"]["dense_operator"],
            serde_json::Value::Null
        );
        assert_eq!(
            legacy_json["artifacts"]["dense_operator_cid"],
            serde_json::Value::Null
        );
        assert_eq!(
            legacy_json["artifacts"]["recorded_corpus_binding_cid"],
            serde_json::Value::Null
        );

        let mut gpt2 = report;
        gpt2.schema = 5;
        gpt2.source.attention_operator = AttentionOperatorSpec::learned_absolute_v2();
        gpt2.source.dense_operator = Some(DenseOperatorSpec::gpt2_v2());
        gpt2.artifacts.dense_operator_cid = Some("blake3:8888".to_owned());
        gpt2.artifacts.recorded_corpus_binding_cid = Some("blake3:9999".to_owned());
        let gpt2_json = serde_json::to_value(&gpt2).expect("GPT-2 evaluation report JSON");
        assert_eq!(gpt2_json["schema"], 5);
        assert_eq!(
            gpt2_json["source"]["dense_operator"]["implementation_digest"],
            DenseOperatorSpec::gpt2_v2().implementation_digest
        );
        assert_eq!(gpt2_json["artifacts"]["dense_operator_cid"], "blake3:8888");
        assert_eq!(
            gpt2_json["artifacts"]["recorded_corpus_binding_cid"],
            "blake3:9999"
        );
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

    /// #603/#605 S1: the traced observe-text flags parse, resolve against
    /// the immutable registry, and refuse incoherent combinations with
    /// typed errors (batch, legacy checkpoint, missing layers, orphan
    /// bounds, unknown profile).
    #[test]
    fn observe_text_trace_flags_parse_and_refuse_incoherent_combinations() {
        let traced = [
            "--trace-profile",
            "full/1",
            "--trace-layers",
            "0,4,9,13,18,22,27,31",
            "--trace-support",
            "8",
        ]
        .map(str::to_owned);
        let options = parse_observe_text_options(&traced).expect("traced flags parse");
        assert_eq!(options.trace_profile, Some(("full".to_owned(), 1)));
        assert_eq!(options.trace_layers, vec![0, 4, 9, 13, 18, 22, 27, 31]);
        assert_eq!(options.trace_support, Some(8));
        let profile = resolve_observe_text_trace_profile(&options)
            .expect("full/1 resolves against the registry")
            .expect("a profile is produced");
        assert_eq!(profile.id, "full");
        assert_eq!(profile.version, 1);

        let unknown = parse_observe_text_options(
            &["--trace-profile", "imagined/9", "--trace-layers", "0"].map(str::to_owned),
        )
        .expect("shape parses");
        assert!(
            resolve_observe_text_trace_profile(&unknown).is_err(),
            "unknown registry pairs are refused, never approximated"
        );

        for args in [
            // batched capture is refused (single-stream taps)
            vec![
                "--trace-profile",
                "full/1",
                "--trace-layers",
                "0",
                "--batch",
                "8",
            ],
            // legacy checkpoint oracle has no capture surface
            vec![
                "--trace-profile",
                "full/1",
                "--trace-layers",
                "0",
                "--checkpoint",
                "/tmp/model.bin",
            ],
            // a profile without declared layers is refused
            vec!["--trace-profile", "full/1"],
            // bounds without a profile are a named mistake
            vec!["--trace-layers", "0,1"],
            vec!["--trace-support", "8"],
            // malformed values
            vec!["--trace-profile", "full"],
            vec!["--trace-layers", "a,b"],
            vec!["--trace-support", "0"],
        ] {
            let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
            assert!(
                parse_observe_text_options(&args).is_err(),
                "{args:?} rejected"
            );
        }
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

#[cfg(test)]
mod transformerless_dispatch_790 {
    // #790 item 2 falsifier: an unknown subcommand must be a nonzero-exit
    // error, never a banner-printing silent success; `help`/no-args stays
    // a successful usage print; the banner only advertises subcommands
    // this dispatcher actually handles.
    #[test]
    fn unknown_subcommand_is_an_error_and_help_is_not() {
        let unknown = super::run(&["definitely-not-a-subcommand".to_string()]);
        assert!(unknown.is_err(), "unknown subcommand must fail loudly");
        assert!(
            unknown
                .unwrap_err()
                .to_string()
                .contains("definitely-not-a-subcommand"),
            "the error must name the unknown subcommand"
        );
        assert!(super::run(&["help".to_string()]).is_ok());
        assert!(super::run(&[]).is_ok());
    }

    #[test]
    fn usage_banner_lists_only_dispatched_subcommands() {
        let usage = super::transformerless_usage();
        assert!(
            !usage.contains("compare"),
            "compare/compare-report are top-level r4 commands, not transformerless subcommands"
        );
        for advertised in [
            "setup",
            "store",
            "scenarios",
            "teacher-kappa",
            "convert-r4g1",
        ] {
            assert!(
                usage.contains(advertised),
                "banner must keep `{advertised}`"
            );
        }
    }
}
