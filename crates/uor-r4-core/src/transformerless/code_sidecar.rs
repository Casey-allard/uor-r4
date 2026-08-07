//! Per-record graded-code sidecar (issue #469 lever A).
//!
//! # Why this exists
//!
//! Every consumer of the transformerless pipeline — the graded store
//! build, the #460 capacity instrument, each scoring harness — needs the
//! same thing first: the graded class code of every corpus record,
//! `assign_for_bundle(bundle_plain(i))` for `i in 0..n`. That pass is the
//! dominant cost of a measurement run on a large corpus, and every
//! consumer recomputes it from scratch because nothing persists it (the
//! compile persists TOKEN codes only, in `hierarchical_codes.json`).
//!
//! This module persists that one pass and reads it back.
//!
//! # Why it cannot change a number
//!
//! The sidecar is a CACHE OF AN EXISTING DETERMINISTIC COMPUTATION, never
//! a new definition of it. Nothing here computes a code: the writer takes
//! codes a caller already derived through the ordinary runtime path, and
//! the reader either returns exactly those bytes or returns `None` and the
//! caller computes as before. There is no third outcome, no interpolation,
//! no partial load, and no code arithmetic anywhere in this file.
//!
//! A load succeeds only when ALL of the following hold:
//!
//! - the magic is [`MAGIC`] and the version is [`VERSION`];
//! - the stored artifact κ equals the caller's artifact κ;
//! - the stored corpus κ equals the caller's corpus κ;
//! - the stored record count equals the caller's record count;
//! - the stored stage count equals [`STAGES`];
//! - the file length is exactly the length the header implies;
//! - the blake3 digest of the packed code block matches the stored digest.
//!
//! A stale, foreign, truncated or corrupted sidecar therefore cannot be
//! loaded silently: it fails one of those checks and the caller falls back
//! to computing. That is the whole safety property.
//!
//! # Format `R4CS`, version 1
//!
//! All integers little-endian; the layout is fixed, so the bytes are a
//! function of the codes and the two κs alone (κ-pinnable in turn).
//!
//! - `0..4`   magic `R4CS`
//! - `4..8`   u32 version
//! - `8..12`  u32 stage count
//! - `12..20` u64 record count `n`
//! - then u16 artifact-κ length, then that many UTF-8 bytes
//! - then u16 corpus-κ length, then that many UTF-8 bytes
//! - then 32 bytes: blake3 of the packed code block
//! - then `n × stages` code bytes, record index implicit in order
//!
//! At `STAGES = 4` that is four bytes per record — about 8.5 MB for a
//! 2.11M-record corpus.
//!
//! # Paths
//!
//! The default path mirrors [`ART_PATH`]; `R4_CODES_PATH` overrides it, in
//! the established `R4_*` style. A default-path collision between two
//! different corpora is a cache MISS (the κs differ), never a wrong load.

use super::compiler::{Compiled, Corpus, STAGES};
use super::runtime;

/// Container magic of the per-record code sidecar.
pub const MAGIC: &[u8; 4] = b"R4CS";

/// Container version. Bumped whenever the layout below changes; an old
/// file then fails the version check and the caller recomputes.
pub const VERSION: u32 = 1;

/// Default sidecar path, mirroring `compiler::ART_PATH`.
pub const CODES_PATH: &str = "/tmp/tless_codes.bin";

/// Environment override for [`CODES_PATH`].
pub const CODES_PATH_ENV: &str = "R4_CODES_PATH";

/// Fixed header bytes before the two length-prefixed κ strings.
const HEADER_FIXED: usize = 20;

/// Digest width of the packed code block (blake3).
const DIGEST_BYTES: usize = 32;

/// Resolve the sidecar path: `R4_CODES_PATH` when set and non-empty,
/// otherwise [`CODES_PATH`].
pub fn codes_path() -> String {
    match std::env::var(CODES_PATH_ENV) {
        Ok(value) if !value.trim().is_empty() => value.trim().to_owned(),
        _ => CODES_PATH.to_owned(),
    }
}

fn hash_u32_slice(hasher: &mut blake3::Hasher, values: &[u32]) {
    let mut scratch = [0u8; 4096];
    let mut filled = 0usize;
    for &value in values {
        scratch[filled..filled + 4].copy_from_slice(&value.to_le_bytes());
        filled += 4;
        if filled == scratch.len() {
            hasher.update(&scratch);
            filled = 0;
        }
    }
    if filled > 0 {
        hasher.update(&scratch[..filled]);
    }
}

/// κ-label of a corpus's CONTENT (`blake3:<hex>`), the sidecar's corpus key.
///
/// Derived from the in-memory corpus rather than from the file bytes it was
/// loaded from, so it addresses what the codes actually depend on and is
/// available to every consumer regardless of how the corpus was obtained.
/// It covers strictly more than the code derivation reads (which is
/// `story` and `input` only), so a corpus that differs anywhere is a MISS —
/// the conservative direction.
pub fn corpus_content_kappa(c: &Corpus) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"r4-corpus-content-v1");
    hasher.update(&(c.n as u64).to_le_bytes());
    hasher.update(&c.stories.to_le_bytes());
    for column in [
        &c.story,
        &c.input,
        &c.next,
        &c.t_argmax,
        &c.span_start,
        &c.span_end,
        &c.byte_start,
        &c.byte_end,
    ] {
        hasher.update(&(column.len() as u64).to_le_bytes());
        hash_u32_slice(&mut hasher, column);
    }
    hasher.update(&(c.top_tokens.len() as u64).to_le_bytes());
    for row in &c.top_tokens {
        hash_u32_slice(&mut hasher, row);
    }
    hasher.update(&(c.top_weights.len() as u64).to_le_bytes());
    for row in &c.top_weights {
        hash_u32_slice(&mut hasher, row);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// The two κs the sidecar is keyed by, for a given artifact and corpus.
pub fn sidecar_keys(art: &Compiled, c: &Corpus) -> (String, String) {
    (
        super::compiler::artifact_kappa(art),
        corpus_content_kappa(c),
    )
}

/// Serialize a sidecar. Byte layout is fixed (see the module docs), so the
/// output is a deterministic function of its inputs.
pub fn sidecar_bytes(artifact_kappa: &str, corpus_kappa: &str, codes: &[[u8; STAGES]]) -> Vec<u8> {
    let mut packed = Vec::with_capacity(codes.len() * STAGES);
    for code in codes {
        packed.extend_from_slice(code);
    }
    let digest = blake3::hash(&packed);
    let art_key = artifact_kappa.as_bytes();
    let corpus_key = corpus_kappa.as_bytes();
    let mut b = Vec::with_capacity(
        HEADER_FIXED + 2 + art_key.len() + 2 + corpus_key.len() + DIGEST_BYTES + packed.len(),
    );
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&VERSION.to_le_bytes());
    b.extend_from_slice(&(STAGES as u32).to_le_bytes());
    b.extend_from_slice(&(codes.len() as u64).to_le_bytes());
    b.extend_from_slice(&(art_key.len() as u16).to_le_bytes());
    b.extend_from_slice(art_key);
    b.extend_from_slice(&(corpus_key.len() as u16).to_le_bytes());
    b.extend_from_slice(corpus_key);
    b.extend_from_slice(digest.as_bytes());
    b.extend_from_slice(&packed);
    b
}

/// Why a sidecar was not usable. Reported in the provenance log so a MISS
/// is never mysterious.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarReject {
    Magic,
    Version(u32),
    Stages(u32),
    RecordCount(u64),
    ArtifactKappa(String),
    CorpusKappa(String),
    Truncated,
    Digest,
}

impl std::fmt::Display for SidecarReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SidecarReject::Magic => write!(f, "not an {} container", MAGIC.escape_ascii()),
            SidecarReject::Version(v) => write!(f, "version {v} (expected {VERSION})"),
            SidecarReject::Stages(s) => write!(f, "stages {s} (expected {STAGES})"),
            SidecarReject::RecordCount(n) => write!(f, "record count {n} does not match corpus"),
            SidecarReject::ArtifactKappa(k) => write!(f, "artifact κ {k} does not match"),
            SidecarReject::CorpusKappa(k) => write!(f, "corpus κ {k} does not match"),
            SidecarReject::Truncated => write!(f, "truncated or over-long container"),
            SidecarReject::Digest => write!(f, "code-block digest mismatch"),
        }
    }
}

fn read_u16(b: &[u8], at: usize) -> Option<usize> {
    let raw = b.get(at..at + 2)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]) as usize)
}

/// Parse a sidecar, returning the codes only when every check in the module
/// docs passes. Any failure yields the reason instead — never codes.
pub fn parse_sidecar(
    bytes: &[u8],
    artifact_kappa: &str,
    corpus_kappa: &str,
    records: usize,
) -> Result<Vec<[u8; STAGES]>, SidecarReject> {
    if bytes.len() < HEADER_FIXED || &bytes[0..4] != MAGIC {
        return Err(SidecarReject::Magic);
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(SidecarReject::Version(version));
    }
    let stages = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if stages as usize != STAGES {
        return Err(SidecarReject::Stages(stages));
    }
    let count = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
    if count != records as u64 {
        return Err(SidecarReject::RecordCount(count));
    }
    let mut offset = HEADER_FIXED;
    let art_len = read_u16(bytes, offset).ok_or(SidecarReject::Truncated)?;
    offset += 2;
    let art_key = bytes
        .get(offset..offset + art_len)
        .ok_or(SidecarReject::Truncated)?;
    offset += art_len;
    let corpus_len = read_u16(bytes, offset).ok_or(SidecarReject::Truncated)?;
    offset += 2;
    let corpus_key = bytes
        .get(offset..offset + corpus_len)
        .ok_or(SidecarReject::Truncated)?;
    offset += corpus_len;
    if art_key != artifact_kappa.as_bytes() {
        return Err(SidecarReject::ArtifactKappa(
            String::from_utf8_lossy(art_key).into_owned(),
        ));
    }
    if corpus_key != corpus_kappa.as_bytes() {
        return Err(SidecarReject::CorpusKappa(
            String::from_utf8_lossy(corpus_key).into_owned(),
        ));
    }
    let digest = bytes
        .get(offset..offset + DIGEST_BYTES)
        .ok_or(SidecarReject::Truncated)?
        .to_vec();
    offset += DIGEST_BYTES;
    let packed_len = records
        .checked_mul(STAGES)
        .ok_or(SidecarReject::RecordCount(count))?;
    let packed = bytes
        .get(offset..offset + packed_len)
        .ok_or(SidecarReject::Truncated)?;
    if offset + packed_len != bytes.len() {
        return Err(SidecarReject::Truncated);
    }
    if blake3::hash(packed).as_bytes()[..] != digest[..] {
        return Err(SidecarReject::Digest);
    }
    let mut codes = Vec::with_capacity(records);
    for chunk in packed.chunks_exact(STAGES) {
        let mut code = [0u8; STAGES];
        code.copy_from_slice(chunk);
        codes.push(code);
    }
    Ok(codes)
}

/// Read the sidecar at [`codes_path`] and verify it against the given keys.
/// `None` means "compute as today"; the reason is logged (#450 precedent:
/// one greppable stderr line naming the path and the κs).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_codes(
    artifact_kappa: &str,
    corpus_kappa: &str,
    records: usize,
) -> Option<Vec<[u8; STAGES]>> {
    let path = codes_path();
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!(
                "code sidecar: MISS {path} ({error}); computing {records} record codes \
                 (artifact κ {artifact_kappa}, corpus κ {corpus_kappa})"
            );
            return None;
        }
    };
    match parse_sidecar(&bytes, artifact_kappa, corpus_kappa, records) {
        Ok(codes) => {
            eprintln!(
                "code sidecar: LOADED {path} ({} records, {} bytes, artifact κ \
                 {artifact_kappa}, corpus κ {corpus_kappa})",
                codes.len(),
                bytes.len()
            );
            Some(codes)
        }
        Err(reason) => {
            eprintln!(
                "code sidecar: MISS {path} ({reason}); computing {records} record codes \
                 (artifact κ {artifact_kappa}, corpus κ {corpus_kappa})"
            );
            None
        }
    }
}

/// Write the sidecar at [`codes_path`], atomically (temp file then rename)
/// so a concurrent reader never sees a partial container. Write failure is
/// reported and otherwise ignored: this is a cache, not an output.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_codes(artifact_kappa: &str, corpus_kappa: &str, codes: &[[u8; STAGES]]) {
    let path = codes_path();
    let bytes = sidecar_bytes(artifact_kappa, corpus_kappa, codes);
    let temp = format!("{path}.tmp.{}", std::process::id());
    let written = std::fs::write(&temp, &bytes).and_then(|()| std::fs::rename(&temp, &path));
    match written {
        Ok(()) => eprintln!(
            "code sidecar: WROTE {path} ({} records, {} bytes, artifact κ {artifact_kappa}, \
             corpus κ {corpus_kappa})",
            codes.len(),
            bytes.len()
        ),
        Err(error) => {
            let _ = std::fs::remove_file(&temp);
            eprintln!("code sidecar: WRITE FAILED {path} ({error})");
        }
    }
}

/// Wasm has no filesystem, so there is no sidecar to consult: the cache
/// degrades to the caller's own derivation. Keeping the signature here
/// rather than making every call site cfg-aware preserves the invariant
/// that this module never derives a code itself and never changes one.
#[cfg(target_arch = "wasm32")]
pub fn corpus_codes_cached<F>(_art: &Compiled, _c: &Corpus, compute: F) -> Vec<[u8; STAGES]>
where
    F: FnOnce() -> Vec<[u8; STAGES]>,
{
    compute()
}

/// The wasm counterpart of [`build_store_cached`]: no sidecar, so the code
/// pass and the store build run exactly as they did before this module
/// existed.
#[cfg(target_arch = "wasm32")]
pub fn build_store_cached(
    art: &Compiled,
    c: &Corpus,
    _threads: usize,
) -> Result<(runtime::Store, Vec<[u8; STAGES]>), String> {
    Ok(runtime::build_store(art, c))
}

/// Per-record codes for a whole corpus: served from the sidecar when it
/// verifies, otherwise produced by `compute` and then written back.
///
/// `compute` is the caller's existing derivation, unchanged — this function
/// never derives a code itself.
#[cfg(not(target_arch = "wasm32"))]
pub fn corpus_codes_cached<F>(art: &Compiled, c: &Corpus, compute: F) -> Vec<[u8; STAGES]>
where
    F: FnOnce() -> Vec<[u8; STAGES]>,
{
    let (artifact_kappa, corpus_kappa) = sidecar_keys(art, c);
    if let Some(codes) = load_codes(&artifact_kappa, &corpus_kappa, c.n) {
        return codes;
    }
    let codes = compute();
    save_codes(&artifact_kappa, &corpus_kappa, &codes);
    codes
}

/// [`runtime::build_store_with_threads`] with the code pass served from the
/// sidecar when it verifies. The store is then built through
/// `runtime::store_from_codes`, the same insertion path, so the store bytes
/// are identical on both branches.
#[cfg(not(target_arch = "wasm32"))]
pub fn build_store_cached(
    art: &Compiled,
    c: &Corpus,
    threads: usize,
) -> Result<(runtime::Store, Vec<[u8; STAGES]>), String> {
    let (artifact_kappa, corpus_kappa) = sidecar_keys(art, c);
    let codes = match load_codes(&artifact_kappa, &corpus_kappa, c.n) {
        Some(codes) => codes,
        None => {
            let codes = runtime::codes_with_threads(art, c, threads)?;
            save_codes(&artifact_kappa, &corpus_kappa, &codes);
            codes
        }
    };
    let store = runtime::store_from_codes(c, &codes);
    Ok((store, codes))
}
