//! Native desktop client backend (`src-tauri`) for the UOR-R4 geometric language model.
//!
//! Links [`uor_r4_integer`] (`features = ["serving-wasm"]`) and [`uor_r4_object_store`] directly as
//! Rust libraries so the desktop client shares the Studio interface (`web/studio/index.html`) while
//! running the D11 integer stack engine and D19 content-addressed durable memory store natively and
//! offline on client laptops (CHARTER §3.6).
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub mod mcp;
pub(crate) mod process_control;
pub mod sidecar;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::session_codec;
use uor_r4_integer::stack::{
    stack_argmax, IntegerStackModel, StackShape, MAGIC as UORLUT01_MAGIC, STACK_SCHEMA,
};
use uor_r4_object_store::durable_consumer::{DurableObjectMemoryStore, IdentityScope};
use uor_r4_object_store::{Limits, Manifest, Store};

/// Default deterministic seed for the `mk-synthetic-stack` stand-in (`crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`).
pub const SYNTHETIC_STANDIN_SEED: u64 = 5;

/// Default file name of the synthetic `UORLUT01` stand-in artifact.
pub const SYNTHETIC_STANDIN_FILE_NAME: &str = "synthetic-stack.lut";

/// Engine identifier reported to the shared Studio interface when running in the Tauri desktop shell.
pub const DESKTOP_ENGINE_TAG: &str = "uor-r4-integer/native-desktop";

/// Stand-in prompt tokenizer label shared with `web/studio/assets/js/uor_model_worker.js`.
pub const PROMPT_TOKENIZER_LABEL: &str = "UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)";

/// Maximum simultaneous conversation sessions held in memory.
pub const MAX_DESKTOP_SESSIONS: usize = 32;

/// Maximum UTF-8 source file size accepted by `native_read_file` (16 MiB).
pub const MAX_READ_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Maximum UTF-8 content accepted by `native_write_file` (16 MiB).
pub const MAX_WRITE_FILE_BYTES: usize = 16 * 1024 * 1024;

/// Maximum `UORLUT01` model file size inspected by `native_list_local_models` / `native_load_model` (128 MiB).
pub const MAX_MODEL_FILE_BYTES: u64 = 128 * 1024 * 1024;

/// Pinned release record for a `UORLUT01` model artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPin {
    pub name: String,
    pub sha256: String,
    pub synthetic: bool,
}

/// Registry of SHA-256 release pins accepted by the model manager.
///
/// Per `docs/products/desktop-client.md` §Security ("Load only files whose SHA-256 matches a
/// pinned release hash"), the model manager verifies a candidate file's SHA-256 against this
/// registry before passing any bytes to `crates/uor-r4-integer/src/stack/format.rs`.
#[derive(Debug, Clone, Default)]
pub struct PinRegistry {
    pins_by_sha256: BTreeMap<String, ModelPin>,
}

impl PinRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pin registry pre-populated with the deterministic seed-5 synthetic stand-in pin.
    pub fn with_synthetic_standin() -> Self {
        let mut reg = Self::new();
        let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
        let sha = sha256_bytes(&bytes);
        reg.register_pin(SYNTHETIC_STANDIN_FILE_NAME, sha, true);
        reg
    }

    pub fn register_pin(
        &mut self,
        name: impl Into<String>,
        sha256: impl Into<String>,
        synthetic: bool,
    ) {
        let sha = sha256.into().to_ascii_lowercase();
        self.pins_by_sha256.insert(
            sha.clone(),
            ModelPin {
                name: name.into(),
                sha256: sha,
                synthetic,
            },
        );
    }

    pub fn get_by_sha256(&self, sha256: &str) -> Option<&ModelPin> {
        self.pins_by_sha256.get(&sha256.to_ascii_lowercase())
    }
}

/// Entry returned to the Studio model manager by `native_list_local_models`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalModelEntry {
    pub name: String,
    pub path: String,
    pub format: String,
    pub size_bytes: u64,
    pub size_str: String,
    pub sha256: String,
    pub synthetic: bool,
    pub status_label: String,
}

/// Shape summary reported in [`LoadedModelCapabilities`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapeSummary {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp: usize,
    pub pattern: String,
    pub read: String,
    pub rotation: bool,
    pub context: usize,
}

impl From<&StackShape> for ShapeSummary {
    fn from(s: &StackShape) -> Self {
        Self {
            vocab: s.vocab,
            width: s.width,
            heads: s.heads,
            mlp: s.mlp,
            pattern: s.pattern.clone(),
            read: s.read.clone(),
            rotation: s.rotation,
            context: s.context,
        }
    }
}

/// Capabilities reported to the Studio page when a pinned `UORLUT01` stack is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedModelCapabilities {
    pub engine: String,
    pub file_name: String,
    pub artifact_bytes: usize,
    pub artifact_sha256: String,
    pub schema: String,
    pub shape: ShapeSummary,
    pub group: usize,
    pub synthetic: bool,
    pub status_label: String,
    pub tokenizer_label: String,
    pub weights_per_token: u64,
    pub checkpoint_bytes: usize,
}

/// Per-token streaming event emitted during `native_step_tokens` / `native_generate_prompt`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamTokenEvent {
    pub conversation_id: String,
    pub step_index: usize,
    pub token_id: u32,
    pub margin_q16: i32,
    pub position: u32,
    pub chunk: String,
    pub full_text: String,
}

/// Final receipt returned at the end of a native generation turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationReceipt {
    pub conversation_id: String,
    pub token_ids: Vec<u32>,
    pub full_text: String,
    pub weights_per_token: u64,
    pub checkpoint_bytes: usize,
    pub status_label: String,
    pub tokenizer_label: String,
}

/// Receipt returned by `native_save_session_durable` after folding a `UORMEM01` + `UR4SESS1`
/// envelope into [`uor_r4_object_store::Store`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableSessionReceipt {
    pub scope_key: String,
    pub object_version_hex: String,
    pub manifest_bytes: Vec<u8>,
    pub ur4sess1_bytes_len: usize,
    pub store_record_count: usize,
    pub store_retained_bytes: usize,
}

/// Receipt returned by `native_restore_session_durable` after verifying and restoring a session
/// from [`uor_r4_object_store::Store`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoredSessionReceipt {
    pub scope_key: String,
    pub conversation_id: String,
    pub artifact_sha256: String,
    pub position: u64,
    pub tokens: Vec<u32>,
    pub ur4sess1_bytes_len: usize,
    pub non_memory_turns: usize,
}

/// Git repository status report returned by `native_git_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatusReport {
    pub branch: String,
    pub is_clean: bool,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub truncated: bool,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitDiffReport {
    pub diff: String,
    pub truncated: bool,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedTerminalCommand {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

/// Request structure for `native_run_terminal_command`.
///
/// Per `docs/products/desktop-client.md` §Security, terminal command execution is off by default,
/// must display the exact command to the user for confirmation before running, and is restricted
/// to a user-granted directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCommandRequest {
    pub cwd: PathBuf,
    pub command: String,
    pub user_confirmed: bool,
    pub confirmed_command: String,
}

/// Errors returned by the desktop client backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopClientError {
    Io(String),
    UnsupportedModelFormat(String),
    UnpinnedOrTamperedModel {
        actual_sha256: String,
        reason: String,
    },
    StackEngine(String),
    ObjectStore(String),
    PathTraversalRefused(String),
    PathNotGranted(String),
    SymlinkRefused(String),
    FileTooLarge {
        bytes: u64,
        max_bytes: u64,
    },
    TerminalDisabledByDefault,
    TerminalConsentRequired,
    TerminalMetacharacterRefused(String),
    CommandFailed(String),
    SessionNotFound(String),
    SessionLimitReached,
    ModelNotLoaded,
    InvalidInput(String),
}

impl fmt::Display for DesktopClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DesktopClientError {}

/// Compute the lowercase hex SHA-256 of an in-memory byte slice.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Format a byte count using integer arithmetic only (`B` or `KiB`).
pub fn format_size_bytes(size_bytes: u64) -> String {
    if size_bytes >= 1024 {
        let kib = size_bytes >> 10;
        format!("{kib} KiB ({size_bytes} B)")
    } else {
        format!("{size_bytes} B")
    }
}

/// Verify that `bytes` starts with `UORLUT01` and that its whole-file SHA-256 matches both
/// `expected_sha256` (if provided) and an entry in `pins` **before** passing the bytes to the
/// structural parser in `crates/uor-r4-integer/src/stack/format.rs`.
pub fn verify_pinned_uorlut01_bytes(
    bytes: &[u8],
    pins: &PinRegistry,
    expected_sha256: Option<&str>,
) -> Result<(String, ModelPin), DesktopClientError> {
    if bytes.len() < 16 || &bytes[..8] != UORLUT01_MAGIC {
        return Err(DesktopClientError::UnsupportedModelFormat(
            "expected UORLUT01 magic header; non-UORLUT01 model formats are refused".to_string(),
        ));
    }
    let actual_sha256 = sha256_bytes(bytes);
    if let Some(expected) = expected_sha256 {
        if !actual_sha256.eq_ignore_ascii_case(expected.trim()) {
            return Err(DesktopClientError::UnpinnedOrTamperedModel {
                actual_sha256,
                reason: "file SHA-256 does not match caller-supplied pin".to_string(),
            });
        }
    }
    let Some(pin) = pins.get_by_sha256(&actual_sha256) else {
        return Err(DesktopClientError::UnpinnedOrTamperedModel {
            actual_sha256,
            reason: "file SHA-256 is not present in the release pin registry".to_string(),
        });
    };
    Ok((actual_sha256, pin.clone()))
}

/// Encode prompt text or explicit token IDs using the deterministic UTF-8 byte-mod stand-in
/// (`raw_bytes[i] % vocab_size`) shared with `web/studio/assets/js/uor_model_worker.js`.
pub fn encode_prompt_standin(
    prompt_text: &str,
    vocab_size: usize,
) -> Result<Vec<u32>, DesktopClientError> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() || vocab_size == 0 {
        return Err(DesktopClientError::InvalidInput(
            "a non-empty prompt and positive vocabulary size are required".to_string(),
        ));
    }
    let vocab_u32 = u32::try_from(vocab_size)
        .map_err(|_| DesktopClientError::InvalidInput("vocabulary size overflow".to_string()))?;
    Ok(trimmed
        .as_bytes()
        .iter()
        .map(|&b| u32::from(b) % vocab_u32)
        .collect())
}

#[derive(Debug, Clone)]
struct ConversationSessionEntry {
    ur4sess1_bytes: Vec<u8>,
    last_token: Option<u32>,
}

/// Host-testable native desktop client backend state (`src-tauri`).
pub struct DesktopBackend {
    model_search_dirs: Vec<PathBuf>,
    pins: PinRegistry,
    granted_folders: BTreeSet<PathBuf>,
    terminal_enabled: bool,
    host_approved_commands: BTreeMap<(PathBuf, String), std::time::Instant>,
    loaded_model: Option<IntegerStackModel>,
    loaded_capabilities: Option<LoadedModelCapabilities>,
    initial_session_bytes: Option<Vec<u8>>,
    sessions: BTreeMap<String, ConversationSessionEntry>,
    object_store: Store,
    memory_store: DurableObjectMemoryStore,
    durable_store_dir: Option<PathBuf>,
}

impl DesktopBackend {
    /// Construct a new desktop backend with an explicit release [`PinRegistry`].
    ///
    /// File access allowlists (`granted_folders`) are empty by default, `terminal_enabled` is
    /// `false` by default, and `host_approved_commands` is empty.
    pub fn new(pins: PinRegistry) -> Self {
        Self {
            model_search_dirs: Vec::new(),
            pins,
            granted_folders: BTreeSet::new(),
            terminal_enabled: false,
            host_approved_commands: BTreeMap::new(),
            loaded_model: None,
            loaded_capabilities: None,
            initial_session_bytes: None,
            sessions: BTreeMap::new(),
            object_store: Store::new(Limits::default()),
            memory_store: DurableObjectMemoryStore::new(),
            durable_store_dir: None,
        }
    }

    /// Borrow the active [`PinRegistry`].
    pub fn pins(&self) -> &PinRegistry {
        &self.pins
    }

    /// Borrow the mutable [`PinRegistry`].
    pub fn pins_mut(&mut self) -> &mut PinRegistry {
        &mut self.pins
    }

    /// Add a directory to scan during `native_list_local_models` (deduplicated).
    pub fn add_model_search_dir(&mut self, dir: impl Into<PathBuf>) {
        let candidate = dir.into();
        let normalized = fs::canonicalize(&candidate).unwrap_or(candidate);
        if !self.model_search_dirs.contains(&normalized) {
            self.model_search_dirs.push(normalized);
        }
    }

    /// Grant and register a model directory on the same authority as project folders.
    pub fn grant_model_search_dir(
        &mut self,
        dir: impl Into<PathBuf>,
    ) -> Result<PathBuf, DesktopClientError> {
        let canonical = self.native_grant_folder(&dir.into())?;
        self.add_model_search_dir(&canonical);
        Ok(canonical)
    }

    /// Register a page-supplied model directory only after verifying an existing grant.
    pub fn add_granted_model_search_dir(
        &mut self,
        dir: impl Into<PathBuf>,
    ) -> Result<PathBuf, DesktopClientError> {
        let canonical = self.verify_granted_existing_path(&dir.into())?;
        if !canonical.is_dir() {
            return Err(DesktopClientError::InvalidInput(
                "model search path is not a directory".to_string(),
            ));
        }
        self.add_model_search_dir(&canonical);
        Ok(canonical)
    }

    /// Borrow the registered model search directories.
    pub fn model_search_dirs(&self) -> &[PathBuf] {
        &self.model_search_dirs
    }

    /// Configure an on-disk directory inside `self.granted_folders` for durable object-store
    /// persistence across restarts.
    ///
    /// Refuses any `dir` outside `self.granted_folders` or any symlink along the path.
    /// If `records.uorobj01` already exists in `dir`, loads and validates its content-addressed
    /// records into `self.object_store`.
    pub fn set_durable_store_dir(
        &mut self,
        dir: impl AsRef<Path>,
    ) -> Result<PathBuf, DesktopClientError> {
        let path = dir.as_ref();
        reject_parent_dir_traversal(path)?;
        reject_if_symlink(path)?;

        let canonical = if path.exists() {
            self.verify_granted_existing_path(path)?
        } else {
            let parent = path.parent().ok_or_else(|| {
                DesktopClientError::PathNotGranted(
                    "durable store path has no parent directory".to_string(),
                )
            })?;
            let dir_name = path.file_name().ok_or_else(|| {
                DesktopClientError::InvalidInput(
                    "durable store path has no directory name".to_string(),
                )
            })?;
            let canonical_parent = self.verify_granted_existing_path(parent)?;
            let target_dir = canonical_parent.join(dir_name);
            reject_if_symlink(&target_dir)?;
            fs::create_dir_all(&target_dir).map_err(|e| DesktopClientError::Io(e.to_string()))?;
            fs::canonicalize(&target_dir).map_err(|e| DesktopClientError::Io(e.to_string()))?
        };

        let records_path = canonical.join("records.uorobj01");
        reject_if_symlink(&records_path)?;
        if records_path.is_file() {
            self.object_store
                .load(&records_path)
                .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        }
        self.durable_store_dir = Some(canonical.clone());
        Ok(canonical)
    }

    /// Borrow the loaded model's capabilities, if a stack is currently loaded.
    pub fn loaded_capabilities(&self) -> Option<&LoadedModelCapabilities> {
        self.loaded_capabilities.as_ref()
    }

    /// Borrow the underlying [`Store`] for inspection in host tests.
    pub fn object_store(&self) -> &Store {
        &self.object_store
    }

    /// Scan `model_search_dirs` for pinned `UORLUT01` model files.
    ///
    /// Reads only the first 8 magic bytes (`UORLUT01`) before reading the rest of a candidate
    /// file, skips symlinks and files exceeding [`MAX_MODEL_FILE_BYTES`], and skips unpinned or
    /// non-`UORLUT01` files so an unrelated `.bin` or `.gguf` file in a directory never hides a
    /// valid pinned `UORLUT01` model. Use [`DesktopBackend::native_load_model`] to validate and
    /// surface explicit pin/format errors for a specific file.
    pub fn native_list_local_models(&self) -> Result<Vec<LocalModelEntry>, DesktopClientError> {
        use std::io::Read;
        let mut entries = Vec::new();
        let mut seen_paths = BTreeSet::new();

        for dir in &self.model_search_dirs {
            if !dir.is_dir() {
                continue;
            }
            let read_dir = fs::read_dir(dir).map_err(|e| DesktopClientError::Io(e.to_string()))?;
            let mut paths: Vec<PathBuf> = read_dir
                .filter_map(|res| res.ok().map(|e| e.path()))
                .collect();
            paths.sort();

            for path in paths {
                let Ok(meta) = fs::symlink_metadata(&path) else {
                    continue;
                };
                if meta.file_type().is_symlink() || !meta.is_file() {
                    continue;
                }
                let size_bytes = meta.len();
                if !(16..=MAX_MODEL_FILE_BYTES).contains(&size_bytes) {
                    continue;
                }
                let is_candidate = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("lut")
                            || ext.eq_ignore_ascii_case("bin")
                            || ext.eq_ignore_ascii_case("gguf")
                            || ext.eq_ignore_ascii_case("safetensors")
                    });
                if !is_candidate {
                    continue;
                }
                let Ok(canonical_path) = fs::canonicalize(&path) else {
                    continue;
                };
                if !seen_paths.insert(canonical_path) {
                    continue;
                }

                // Stream-check the 8-byte UORLUT01 magic header before reading the full file.
                let Ok(mut file) = fs::File::open(&path) else {
                    continue;
                };
                let mut magic = [0u8; 8];
                if file.read_exact(&mut magic).is_err() || &magic != UORLUT01_MAGIC {
                    continue;
                }

                let mut bytes = Vec::with_capacity(size_bytes as usize);
                bytes.extend_from_slice(&magic);
                if file.read_to_end(&mut bytes).is_err() {
                    continue;
                }
                let Ok((sha256, pin)) = verify_pinned_uorlut01_bytes(&bytes, &self.pins, None)
                else {
                    continue;
                };
                let Ok(model) = IntegerStackModel::parse(&bytes) else {
                    continue;
                };
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&pin.name)
                    .to_string();
                let status_label = if pin.synthetic {
                    "synthetic stand-in".to_string()
                } else {
                    format!("pinned UORLUT01 ({} w/tok)", model.weights_per_token())
                };
                entries.push(LocalModelEntry {
                    name: file_name,
                    path: path.to_string_lossy().into_owned(),
                    format: "UORLUT01".to_string(),
                    size_bytes,
                    size_str: format_size_bytes(size_bytes),
                    sha256,
                    synthetic: pin.synthetic,
                    status_label,
                });
            }
        }
        Ok(entries)
    }

    /// Load a `UORLUT01` model file from disk after checking its 8-byte magic header and
    /// verifying its SHA-256 against [`PinRegistry`].
    pub fn native_load_model(
        &mut self,
        file_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<LoadedModelCapabilities, DesktopClientError> {
        use std::io::Read;
        let canonical = self.verify_granted_existing_path(file_path)?;
        reject_if_symlink(&canonical)?;
        let meta =
            fs::symlink_metadata(&canonical).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        if meta.len() > MAX_MODEL_FILE_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: meta.len(),
                max_bytes: MAX_MODEL_FILE_BYTES,
            });
        }
        let mut file =
            fs::File::open(&canonical).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        let mut header = [0u8; 8];
        if file.read_exact(&mut header).is_err() || &header != UORLUT01_MAGIC {
            return Err(DesktopClientError::UnsupportedModelFormat(
                "expected UORLUT01 magic header; non-UORLUT01 model formats are refused"
                    .to_string(),
            ));
        }
        let mut bytes = Vec::with_capacity(meta.len() as usize);
        bytes.extend_from_slice(&header);
        file.read_to_end(&mut bytes)
            .map_err(|e| DesktopClientError::Io(e.to_string()))?;
        let file_name = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(SYNTHETIC_STANDIN_FILE_NAME);
        self.native_load_model_bytes(file_name, &bytes, expected_sha256)
    }

    /// Load a `UORLUT01` model from raw bytes after verifying its SHA-256 against [`PinRegistry`].
    pub fn native_load_model_bytes(
        &mut self,
        file_name: &str,
        bytes: &[u8],
        expected_sha256: Option<&str>,
    ) -> Result<LoadedModelCapabilities, DesktopClientError> {
        let (sha256, pin) = verify_pinned_uorlut01_bytes(bytes, &self.pins, expected_sha256)?;
        let model = IntegerStackModel::parse(bytes)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
        let blank_session = model.session();
        let blank_bytes = session_codec::encode(&blank_session.save_state());
        let capabilities = LoadedModelCapabilities {
            engine: DESKTOP_ENGINE_TAG.to_string(),
            file_name: file_name.to_string(),
            artifact_bytes: bytes.len(),
            artifact_sha256: sha256,
            schema: STACK_SCHEMA.to_string(),
            shape: ShapeSummary::from(model.shape()),
            group: uor_r4_integer::stack::GROUP,
            synthetic: pin.synthetic,
            status_label: if pin.synthetic {
                "synthetic stand-in".to_string()
            } else {
                "pinned stack artifact".to_string()
            },
            tokenizer_label: PROMPT_TOKENIZER_LABEL.to_string(),
            weights_per_token: model.weights_per_token(),
            checkpoint_bytes: blank_bytes.len(),
        };
        self.loaded_model = Some(model);
        self.loaded_capabilities = Some(capabilities.clone());
        self.initial_session_bytes = Some(blank_bytes);
        self.sessions.clear();
        Ok(capabilities)
    }

    fn ensure_session_entry(
        &mut self,
        conversation_id: &str,
    ) -> Result<ConversationSessionEntry, DesktopClientError> {
        if conversation_id.trim().is_empty() {
            return Err(DesktopClientError::InvalidInput(
                "conversation_id must not be empty".to_string(),
            ));
        }
        let initial = self
            .initial_session_bytes
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        if let Some(existing) = self.sessions.get(conversation_id) {
            return Ok(existing.clone());
        }
        if self.sessions.len() >= MAX_DESKTOP_SESSIONS {
            return Err(DesktopClientError::SessionLimitReached);
        }
        let fresh = ConversationSessionEntry {
            ur4sess1_bytes: initial.clone(),
            last_token: None,
        };
        self.sessions
            .insert(conversation_id.to_string(), fresh.clone());
        Ok(fresh)
    }

    /// Step the native [`IntegerStackSession`] for `conversation_id` over `prompt_tokens` and
    /// generate `max_new_tokens` greedy tokens using integer argmax, streaming each token via
    /// `on_token`.
    pub fn native_step_tokens<F>(
        &mut self,
        conversation_id: &str,
        prompt_tokens: &[u32],
        max_new_tokens: usize,
        mut on_token: F,
    ) -> Result<GenerationReceipt, DesktopClientError>
    where
        F: FnMut(StreamTokenEvent),
    {
        if prompt_tokens.is_empty() {
            return Err(DesktopClientError::InvalidInput(
                "prompt_tokens must not be empty".to_string(),
            ));
        }
        if max_new_tokens == 0 || max_new_tokens > 4096 {
            return Err(DesktopClientError::InvalidInput(
                "max_new_tokens must be in 1..=4096".to_string(),
            ));
        }
        let entry = self.ensure_session_entry(conversation_id)?;
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let caps = self
            .loaded_capabilities
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;

        let decoded_state = session_codec::decode(&entry.ur4sess1_bytes)
            .map_err(DesktopClientError::StackEngine)?;
        let mut session = model.session();
        session
            .restore_state(&decoded_state)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;

        let mut tok = prompt_tokens[0];
        for &next_prompt_tok in &prompt_tokens[1..] {
            session
                .step(tok)
                .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
            tok = next_prompt_tok;
        }

        let mut token_ids = Vec::with_capacity(max_new_tokens);
        let mut full_text = String::new();

        for step_index in 0..max_new_tokens {
            let logits = session
                .step(tok)
                .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
            let next = stack_argmax(logits) as u32;
            let (mut top1, mut top2) = (i32::MIN, i32::MIN);
            for &v in logits {
                if v > top1 {
                    top2 = top1;
                    top1 = v;
                } else if v > top2 {
                    top2 = v;
                }
            }
            let margin_q16 = top1.saturating_sub(top2);
            let position = session.position() as u32;

            tok = next;
            token_ids.push(tok);
            let chunk = if step_index == 0 {
                tok.to_string()
            } else {
                format!(" {tok}")
            };
            full_text.push_str(&chunk);

            on_token(StreamTokenEvent {
                conversation_id: conversation_id.to_string(),
                step_index,
                token_id: tok,
                margin_q16,
                position,
                chunk,
                full_text: full_text.clone(),
            });
        }

        let saved_bytes = session_codec::encode(&session.save_state());
        let checkpoint_bytes = saved_bytes.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: saved_bytes,
                last_token: Some(tok),
            },
        );

        Ok(GenerationReceipt {
            conversation_id: conversation_id.to_string(),
            token_ids,
            full_text,
            weights_per_token: caps.weights_per_token,
            checkpoint_bytes,
            status_label: caps.status_label.clone(),
            tokenizer_label: caps.tokenizer_label.clone(),
        })
    }

    /// Encode `prompt_text` with the UTF-8 byte-mod stand-in and step `max_new_tokens` tokens.
    pub fn native_generate_prompt<F>(
        &mut self,
        conversation_id: &str,
        prompt_text: &str,
        max_new_tokens: usize,
        on_token: F,
    ) -> Result<GenerationReceipt, DesktopClientError>
    where
        F: FnMut(StreamTokenEvent),
    {
        let vocab = self
            .loaded_capabilities
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?
            .shape
            .vocab;
        let prompt_tokens = encode_prompt_standin(prompt_text, vocab)?;
        self.native_step_tokens(conversation_id, &prompt_tokens, max_new_tokens, on_token)
    }

    /// Export the raw `UR4SESS1` session snapshot bytes and last token for `conversation_id`.
    pub fn native_export_session(
        &mut self,
        conversation_id: &str,
    ) -> Result<(Vec<u8>, Option<u32>), DesktopClientError> {
        let entry = self.ensure_session_entry(conversation_id)?;
        Ok((entry.ur4sess1_bytes, entry.last_token))
    }

    /// Import and validate a `UR4SESS1` session snapshot for `conversation_id`.
    pub fn native_import_session(
        &mut self,
        conversation_id: &str,
        ur4sess1_bytes: &[u8],
        last_token: Option<u32>,
    ) -> Result<usize, DesktopClientError> {
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let decoded =
            session_codec::decode(ur4sess1_bytes).map_err(DesktopClientError::StackEngine)?;
        let mut session = model.session();
        session
            .restore_state(&decoded)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
        let verified = session_codec::encode(&session.save_state());
        let len = verified.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: verified,
                last_token,
            },
        );
        Ok(len)
    }

    /// Reset or close `conversation_id`, restoring its state to the initial blank session.
    pub fn native_reset_session(
        &mut self,
        conversation_id: &str,
    ) -> Result<(), DesktopClientError> {
        if self.loaded_model.is_none() {
            return Err(DesktopClientError::ModelNotLoaded);
        }
        self.sessions.remove(conversation_id);
        Ok(())
    }

    /// Persist the active `UR4SESS1` snapshot for `(user_id, project_id, conversation_id)` into
    /// [`uor_r4_object_store::Store`] via [`DurableObjectMemoryStore::export_scope`] (`UORMEM01`).
    ///
    /// When `durable_store_dir` is configured, atomically saves the deduplicated object-store
    /// records (`records.uorobj01`) and the scope's pinned [`Manifest`] (`scopes/<scope>.uorman01`)
    /// to disk so a subsequent process can restore the conversation across restarts.
    pub fn native_save_session_durable(
        &mut self,
        user_id: &str,
        project_id: &str,
        conversation_id: &str,
    ) -> Result<DurableSessionReceipt, DesktopClientError> {
        let scope = IdentityScope::new(user_id, project_id, conversation_id)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let entry = self.ensure_session_entry(conversation_id)?;
        self.memory_store
            .set_ur4sess1_snapshot(&scope, &entry.ur4sess1_bytes)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let manifest = self
            .memory_store
            .export_scope(&mut self.object_store, &scope)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let manifest_bytes = manifest.encode();

        if let Some(ref dir) = self.durable_store_dir {
            let records_path = dir.join("records.uorobj01");
            let scopes_dir = dir.join("scopes");
            let manifest_path = scopes_dir.join(scope_manifest_filename(&scope));
            reject_if_symlink(&scopes_dir)?;
            fs::create_dir_all(&scopes_dir).map_err(|e| DesktopClientError::Io(e.to_string()))?;
            self.verify_granted_write_path(&records_path)?;
            self.verify_granted_write_path(&manifest_path)?;
            self.object_store
                .save(&records_path)
                .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
            write_atomic_file(&manifest_path, &manifest_bytes)?;
        }

        Ok(DurableSessionReceipt {
            scope_key: scope.key(),
            object_version_hex: hex::encode(manifest.version()),
            manifest_bytes,
            ur4sess1_bytes_len: entry.ur4sess1_bytes.len(),
            store_record_count: self.object_store.record_count(),
            store_retained_bytes: self.object_store.retained_record_bytes(),
        })
    }

    /// Rebuild and verify the `UORMEM01` + `UR4SESS1` snapshot for `(user_id, project_id, conversation_id)`
    /// from [`uor_r4_object_store::Store`] and restore the conversation's [`IntegerStackSession`].
    pub fn native_restore_session_durable(
        &mut self,
        user_id: &str,
        project_id: &str,
        conversation_id: &str,
        manifest_bytes: Option<&[u8]>,
    ) -> Result<RestoredSessionReceipt, DesktopClientError> {
        let scope = IdentityScope::new(user_id, project_id, conversation_id)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;

        let owned_manifest_bytes: Vec<u8>;
        let raw_manifest: &[u8] = match manifest_bytes {
            Some(b) => b,
            None => {
                let dir = self.durable_store_dir.as_ref().ok_or_else(|| {
                    DesktopClientError::InvalidInput(
                        "either manifest_bytes or durable_store_dir is required".to_string(),
                    )
                })?;
                let scopes_dir = dir.join("scopes");
                reject_if_symlink(&scopes_dir)?;
                let manifest_path = scopes_dir.join(scope_manifest_filename(&scope));
                self.verify_granted_existing_path(&manifest_path)?;
                owned_manifest_bytes =
                    fs::read(&manifest_path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
                &owned_manifest_bytes
            }
        };

        let manifest = Manifest::decode(raw_manifest, self.object_store.limits())
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        self.memory_store
            .import_scope_expecting(&self.object_store, &manifest, &scope)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;

        let ur4sess1_bytes = self
            .memory_store
            .ur4sess1_snapshot(&scope)
            .ok_or_else(|| {
                DesktopClientError::ObjectStore("missing UR4SESS1 snapshot in scope".to_string())
            })?
            .to_vec();

        let decoded =
            session_codec::decode(&ur4sess1_bytes).map_err(DesktopClientError::StackEngine)?;
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let mut session = model.session();
        session
            .restore_state(&decoded)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;

        let last_token = decoded.tokens.last().copied();
        let verified_bytes = session_codec::encode(&session.save_state());
        let ur4sess1_bytes_len = verified_bytes.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: verified_bytes,
                last_token,
            },
        );

        Ok(RestoredSessionReceipt {
            scope_key: scope.key(),
            conversation_id: conversation_id.to_string(),
            artifact_sha256: decoded.artifact_sha256,
            position: decoded.position,
            tokens: decoded.tokens,
            ur4sess1_bytes_len,
            non_memory_turns: self.memory_store.non_memory_turn_count(&scope),
        })
    }

    /// Explicitly grant read/write access to a local project directory (refusing symlinks).
    pub fn native_grant_folder(
        &mut self,
        folder_path: &Path,
    ) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(folder_path)?;
        reject_if_symlink(folder_path)?;
        let canonical =
            fs::canonicalize(folder_path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        if !canonical.is_dir() {
            return Err(DesktopClientError::InvalidInput(format!(
                "granted path is not a directory: {}",
                canonical.display()
            )));
        }
        self.granted_folders.insert(canonical.clone());
        Ok(canonical)
    }

    /// Borrow the set of canonicalized granted folders.
    pub fn granted_folders(&self) -> &BTreeSet<PathBuf> {
        &self.granted_folders
    }

    pub fn verify_granted_existing_path(
        &self,
        target: &Path,
    ) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(target)?;
        reject_if_symlink(target)?;
        let canonical = fs::canonicalize(target).map_err(|_| {
            DesktopClientError::PathNotGranted(format!(
                "path does not exist or is outside granted folders: {}",
                target.display()
            ))
        })?;
        let Some(root) = self
            .granted_folders
            .iter()
            .find(|r| canonical.starts_with(*r))
        else {
            return Err(DesktopClientError::PathNotGranted(format!(
                "access outside granted folders is refused: {}",
                canonical.display()
            )));
        };
        reject_symlinks_between_root_and_target(root, target)?;
        Ok(canonical)
    }

    pub fn verify_granted_write_path(&self, target: &Path) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(target)?;
        // Always check symlink_metadata first so dangling symlinks (`target.exists() == false`)
        // are refused before falling back to parent canonicalization.
        reject_if_symlink(target)?;
        if target.exists() {
            return self.verify_granted_existing_path(target);
        }
        let parent = target.parent().ok_or_else(|| {
            DesktopClientError::PathNotGranted("target path has no parent directory".to_string())
        })?;
        let file_name = target.file_name().ok_or_else(|| {
            DesktopClientError::InvalidInput("target path has no file name".to_string())
        })?;
        let canonical_parent = self.verify_granted_existing_path(parent)?;
        let joined = canonical_parent.join(file_name);
        reject_if_symlink(&joined)?;
        Ok(joined)
    }

    /// Read a UTF-8 file located inside a user-granted folder (capped at [`MAX_READ_FILE_BYTES`]).
    pub fn native_read_file(&self, file_path: &Path) -> Result<String, DesktopClientError> {
        let canonical = self.verify_granted_existing_path(file_path)?;
        let meta =
            fs::symlink_metadata(&canonical).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        if meta.len() > MAX_READ_FILE_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: meta.len(),
                max_bytes: MAX_READ_FILE_BYTES,
            });
        }
        fs::read_to_string(&canonical).map_err(|e| DesktopClientError::Io(e.to_string()))
    }

    /// Write a UTF-8 file inside a user-granted folder (refusing symlinks and dangling symlinks).
    pub fn native_write_file(
        &self,
        file_path: &Path,
        content: &str,
    ) -> Result<(), DesktopClientError> {
        if content.len() > MAX_WRITE_FILE_BYTES {
            return Err(DesktopClientError::FileTooLarge {
                bytes: content.len() as u64,
                max_bytes: MAX_WRITE_FILE_BYTES as u64,
            });
        }
        let safe_path = self.verify_granted_write_path(file_path)?;
        let _git_guard = lock_git_mutation_guard()?;
        fs::write(&safe_path, content).map_err(|e| DesktopClientError::Io(e.to_string()))
    }

    /// List relative file paths inside a user-granted folder up to `max_depth` levels deep
    /// without invoking any shell command.
    pub fn native_list_directory(
        &self,
        folder_path: &Path,
        max_depth: usize,
    ) -> Result<Vec<String>, DesktopClientError> {
        let root = self.verify_granted_existing_path(folder_path)?;
        let mut out = Vec::new();
        collect_directory_files(&root, &root, 0, max_depth.min(8), &mut out)?;
        out.sort();
        Ok(out)
    }

    /// Inspect Git branch and status for a repository inside a user-granted folder.
    ///
    /// Passes `-c core.fsmonitor=false -c core.hooksPath=/dev/null` so untrusted `.git/config`
    /// fsmonitor hooks never execute code.
    pub fn native_git_status(
        &self,
        repo_path: &Path,
    ) -> Result<GitStatusReport, DesktopClientError> {
        let canonical_repo = self.verify_granted_existing_path(repo_path)?;
        git_status_at(&canonical_repo)
    }

    pub fn native_git_diff(&self, repo_path: &Path) -> Result<GitDiffReport, DesktopClientError> {
        let canonical_repo = self.verify_granted_existing_path(repo_path)?;
        git_diff_at(&canonical_repo)
    }

    /// Host-owned control: enable or disable `native_run_terminal_command` (off by default).
    ///
    /// Not exposed to unprivileged web-page JSON-RPC callers so a page script cannot turn the
    /// terminal on without host opt-in.
    pub fn native_set_terminal_enabled(&mut self, enabled: bool) {
        self.terminal_enabled = enabled;
        if !enabled {
            self.host_approved_commands.clear();
        }
    }

    /// Return whether terminal command execution is currently enabled.
    pub fn terminal_enabled(&self) -> bool {
        self.terminal_enabled
    }

    /// Record a one-shot host-side user confirmation for exact `(cwd, command)`.
    ///
    /// In the desktop shell, confirmation comes from the native host dialog / CLI prompt rather
    /// than trusting a page-supplied boolean alone.
    pub fn host_approve_terminal_command(
        &mut self,
        cwd: &Path,
        command: &str,
    ) -> Result<(), DesktopClientError> {
        if !self.terminal_enabled {
            return Err(DesktopClientError::TerminalDisabledByDefault);
        }
        let canonical_cwd = self.verify_granted_existing_path(cwd)?;
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(DesktopClientError::TerminalConsentRequired);
        }
        validate_consent_text(trimmed)?;
        self.host_approved_commands.insert(
            (canonical_cwd, trimmed.to_string()),
            std::time::Instant::now(),
        );
        Ok(())
    }

    /// Validate consent and paths, then return the exact direct-spawn command.
    pub fn prepare_terminal_command(
        &mut self,
        req: &TerminalCommandRequest,
    ) -> Result<PreparedTerminalCommand, DesktopClientError> {
        if !self.terminal_enabled {
            return Err(DesktopClientError::TerminalDisabledByDefault);
        }
        let trimmed = req.command.trim();
        if trimmed.is_empty() || !req.user_confirmed || req.confirmed_command.trim() != trimmed {
            return Err(DesktopClientError::TerminalConsentRequired);
        }
        for forbidden in [';', '|', '&', '`', '$', '>', '<', '\n', '\r'] {
            if trimmed.contains(forbidden) {
                return Err(DesktopClientError::TerminalMetacharacterRefused(
                    forbidden.to_string(),
                ));
            }
        }
        let canonical_cwd = self.verify_granted_existing_path(&req.cwd)?;
        if !self
            .host_approved_commands
            .remove(&(canonical_cwd.clone(), trimmed.to_string()))
            .is_some_and(|approved_at| approved_at.elapsed() <= std::time::Duration::from_secs(60))
        {
            return Err(DesktopClientError::TerminalConsentRequired);
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let Some((program, args)) = parts.split_first() else {
            return Err(DesktopClientError::TerminalConsentRequired);
        };
        validate_terminal_program(program)?;
        validate_terminal_argument_shape(program, args)?;

        for &arg in args {
            if arg.starts_with('-') && (arg.contains('/') || arg.contains('\\')) {
                let separator = arg.find(['/', '\\']).unwrap_or(0);
                self.verify_granted_write_path(Path::new(&arg[separator..]))?;
                continue;
            }
            let candidate_str = match arg.split_once('=') {
                Some((_, val)) if arg.starts_with('-') => val,
                _ => arg,
            };
            if candidate_str.contains("..") {
                return Err(DesktopClientError::PathTraversalRefused(arg.to_string()));
            }
            if candidate_str.starts_with('~') || candidate_str.contains("/~") {
                return Err(DesktopClientError::PathNotGranted(arg.to_string()));
            }
            if candidate_str.starts_with('/') || candidate_str.starts_with('\\') {
                self.verify_granted_write_path(Path::new(candidate_str))?;
            } else if !candidate_str.starts_with('-') {
                let joined = canonical_cwd.join(candidate_str);
                reject_if_symlink(&joined)?;
                if candidate_str.contains('/') || candidate_str.contains('\\') || joined.exists() {
                    self.verify_granted_write_path(&joined)?;
                }
            }
        }
        Ok(PreparedTerminalCommand {
            program: (*program).to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            cwd: canonical_cwd,
        })
    }

    pub fn native_run_terminal_command(
        &mut self,
        req: &TerminalCommandRequest,
    ) -> Result<String, DesktopClientError> {
        let prepared = self.prepare_terminal_command(req)?;
        if prepared.program == "git" {
            ensure_git_repository_root(&prepared.cwd)?;
        }
        run_prepared_terminal_command(&prepared)
    }
}

const MAX_PROCESS_OUTPUT_BYTES: usize = 64 * 1024;
const PROCESS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
static GIT_MUTATION_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_git_mutation_guard() -> Result<std::sync::MutexGuard<'static, ()>, DesktopClientError> {
    GIT_MUTATION_GUARD.lock().map_err(|_| {
        DesktopClientError::CommandFailed("Git mutation guard is unavailable".to_string())
    })
}

struct BoundedProcessOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: std::process::ExitStatus,
    truncated: bool,
    timed_out: bool,
}

fn forward_process_output<R: std::io::Read>(
    mut reader: R,
    is_stdout: bool,
    sender: std::sync::mpsc::SyncSender<(bool, Vec<u8>)>,
) {
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if sender.send((is_stdout, buffer[..read].to_vec())).is_err() {
                    break;
                }
            }
        }
    }
}

fn run_bounded_process(
    command: &mut Command,
    timeout: std::time::Duration,
    max_bytes: usize,
) -> Result<BoundedProcessOutput, DesktopClientError> {
    use std::process::Stdio;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::{Duration, Instant};

    process_control::prepare(command);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| DesktopClientError::CommandFailed("stdout pipe unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| DesktopClientError::CommandFailed("stderr pipe unavailable".to_string()))?;
    let (sender, receiver) = std::sync::mpsc::sync_channel::<(bool, Vec<u8>)>(16);
    let stdout_sender = sender.clone();
    let stdout_thread = std::thread::spawn(move || {
        forward_process_output(stdout, true, stdout_sender);
    });
    let stderr_thread = std::thread::spawn(move || {
        forward_process_output(stderr, false, sender);
    });

    let deadline = Instant::now() + timeout;
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut retained = 0usize;
    let mut truncated = false;
    let mut timed_out = false;
    let mut killed = false;
    let mut readers_disconnected = false;
    loop {
        if !killed && retained >= max_bytes {
            truncated = true;
            process_control::terminate(&mut child);
            killed = true;
        }
        if !killed && Instant::now() >= deadline {
            timed_out = true;
            process_control::terminate(&mut child);
            killed = true;
        }
        if killed
            && child
                .try_wait()
                .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?
                .is_some()
        {
            break;
        }
        match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok((is_stdout, bytes)) => {
                let keep = max_bytes.saturating_sub(retained).min(bytes.len());
                if is_stdout {
                    stdout_bytes.extend_from_slice(&bytes[..keep]);
                } else {
                    stderr_bytes.extend_from_slice(&bytes[..keep]);
                }
                retained += keep;
                if keep < bytes.len() && !killed {
                    truncated = true;
                    process_control::terminate(&mut child);
                    killed = true;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                readers_disconnected = true;
                break;
            }
        }
    }
    let exited = child
        .try_wait()
        .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?
        .is_some();
    if !exited {
        timed_out = true;
        process_control::terminate(&mut child);
    }
    if !readers_disconnected && !killed {
        timed_out = true;
    }
    let status = child
        .wait()
        .map_err(|error| DesktopClientError::CommandFailed(error.to_string()))?;
    if readers_disconnected {
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();
    }
    Ok(BoundedProcessOutput {
        stdout: stdout_bytes,
        stderr: stderr_bytes,
        status,
        truncated,
        timed_out,
    })
}

fn hardened_git_command(repo: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .args([
            "--no-pager",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "filter.*.clean=cat",
            "-c",
            "filter.*.smudge=cat",
            "-c",
            "filter.*.process=cat",
            "-c",
            "core.pager=cat",
            "-c",
            "pager.status=false",
            "-c",
            "pager.diff=false",
            "-C",
        ])
        .arg(repo)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("LC_ALL", "C");
    for variable in [
        "GIT_CONFIG_PARAMETERS",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_EXTERNAL_DIFF",
    ] {
        command.env_remove(variable);
    }
    command
}

fn ensure_no_git_filters(repo: &Path) -> Result<(), DesktopClientError> {
    for scope in ["--local", "--worktree"] {
        let output = run_bounded_process(
            hardened_git_command(repo).args([
                "config",
                scope,
                "--includes",
                "--get-regexp",
                "^filter\\.",
            ]),
            PROCESS_TIMEOUT,
            MAX_PROCESS_OUTPUT_BYTES,
        )?;
        if output.status.success() {
            if !output.stdout.is_empty() {
                return Err(DesktopClientError::InvalidInput(
                    "Git filter drivers are not allowed in guarded repositories".to_string(),
                ));
            }
            continue;
        }
        if output.status.code() != Some(1) {
            return Err(DesktopClientError::CommandFailed(
                "could not verify Git filter configuration".to_string(),
            ));
        }
    }
    Ok(())
}

fn ensure_git_repository_root(repo: &Path) -> Result<(), DesktopClientError> {
    let output = run_bounded_process(
        hardened_git_command(repo).args(["rev-parse", "--show-toplevel"]),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if !output.status.success() {
        return Err(DesktopClientError::PathNotGranted(
            "path is not a Git repository root".to_string(),
        ));
    }
    let top = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let canonical = fs::canonicalize(&top).map_err(|_| {
        DesktopClientError::PathNotGranted("Git repository root is unavailable".to_string())
    })?;
    if canonical != repo {
        return Err(DesktopClientError::PathNotGranted(format!(
            "Git repository discovery escapes the granted directory: {top}"
        )));
    }
    ensure_no_git_filters(repo)
}

fn git_status_at(repo: &Path) -> Result<GitStatusReport, DesktopClientError> {
    let _git_guard = lock_git_mutation_guard()?;
    ensure_git_repository_root(repo)?;
    let status_out = run_bounded_process(
        hardened_git_command(repo).args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
        ]),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if !status_out.status.success() && !status_out.truncated && !status_out.timed_out {
        return Err(DesktopClientError::CommandFailed(
            String::from_utf8_lossy(&status_out.stderr)
                .trim()
                .to_string(),
        ));
    }
    let branch_out = run_bounded_process(
        hardened_git_command(repo).args(["rev-parse", "--abbrev-ref", "HEAD"]),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    let branch = if branch_out.status.success() {
        String::from_utf8_lossy(&branch_out.stdout)
            .trim()
            .to_string()
    } else if branch_out.timed_out {
        "HEAD (timeout)".to_string()
    } else {
        "HEAD (unborn)".to_string()
    };

    let mut modified_files = Vec::new();
    let mut untracked_files = Vec::new();
    for record in String::from_utf8_lossy(&status_out.stdout).split('\0') {
        if record.len() < 4 {
            continue;
        }
        let code = &record[..2];
        let file = record[3..].to_string();
        if code == "??" {
            untracked_files.push(file);
        } else {
            modified_files.push(file);
        }
    }
    let truncated = status_out.truncated || branch_out.truncated;
    let timed_out = status_out.timed_out || branch_out.timed_out;
    let is_clean =
        !truncated && !timed_out && modified_files.is_empty() && untracked_files.is_empty();
    Ok(GitStatusReport {
        branch,
        is_clean,
        modified_files,
        untracked_files,
        truncated,
        timed_out,
    })
}

fn git_diff_at(repo: &Path) -> Result<GitDiffReport, DesktopClientError> {
    let _git_guard = lock_git_mutation_guard()?;
    ensure_git_repository_root(repo)?;
    let output = run_bounded_process(
        hardened_git_command(repo).args([
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--relative",
            "--",
        ]),
        PROCESS_TIMEOUT,
        MAX_PROCESS_OUTPUT_BYTES,
    )?;
    if !output.status.success() && !output.truncated && !output.timed_out {
        return Err(DesktopClientError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(GitDiffReport {
        diff: String::from_utf8_lossy(&output.stdout).into_owned(),
        truncated: output.truncated,
        timed_out: output.timed_out,
    })
}

fn validate_consent_text(value: &str) -> Result<(), DesktopClientError> {
    if value.chars().any(|character| {
        character.is_control()
            || matches!(character, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{feff}')
    }) {
        return Err(DesktopClientError::InvalidInput(
            "control and bidirectional formatting characters are refused in host consent".to_string(),
        ));
    }
    Ok(())
}

fn validate_terminal_argument_shape(
    program: &str,
    args: &[&str],
) -> Result<(), DesktopClientError> {
    for arg in args {
        if matches!(*arg, "-exec" | "-execdir" | "-ok" | "-okdir")
            || (program == "rg" && (*arg == "--pre" || arg.starts_with("--pre=")))
            || (program == "git"
                && (arg.starts_with("-c")
                    || arg.starts_with("-C")
                    || matches!(
                        *arg,
                        "--config" | "--config-env" | "--exec-path" | "--git-dir" | "--work-tree"
                    )
                    || [
                        "--config=",
                        "--config-env=",
                        "--exec-path=",
                        "--git-dir=",
                        "--work-tree=",
                    ]
                    .iter()
                    .any(|prefix| arg.starts_with(prefix))))
        {
            return Err(DesktopClientError::InvalidInput(format!(
                "terminal execution flag is refused: {arg}"
            )));
        }
        let attached_file_value = match program {
            "grep" | "rg" => arg.starts_with("-f") && arg.len() > 2,
            "find" => {
                (arg.starts_with("-fprint") && arg.len() > "-fprint".len())
                    || (arg.starts_with("-fls") && arg.len() > "-fls".len())
            }
            _ => false,
        };
        if attached_file_value {
            return Err(DesktopClientError::InvalidInput(format!(
                "file-taking option requires a separately validated path: {arg}"
            )));
        }
    }
    Ok(())
}

pub fn validate_terminal_program(program: &str) -> Result<(), DesktopClientError> {
    const ALLOWED: [&str; 14] = [
        "cargo",
        "git",
        "ls",
        "pwd",
        "cat",
        "head",
        "tail",
        "wc",
        "grep",
        "rg",
        "find",
        "echo",
        "uor-chat",
        "uor-harness",
    ];
    if program.contains('/') || program.contains('\\') || program.contains("..") {
        return Err(DesktopClientError::PathTraversalRefused(
            program.to_string(),
        ));
    }
    if !ALLOWED.contains(&program) {
        return Err(DesktopClientError::InvalidInput(format!(
            "terminal program is not allowed: {program}"
        )));
    }
    Ok(())
}

fn run_prepared_terminal_command(
    prepared: &PreparedTerminalCommand,
) -> Result<String, DesktopClientError> {
    let _git_guard = if prepared.program == "git" {
        let guard = lock_git_mutation_guard()?;
        ensure_git_repository_root(&prepared.cwd)?;
        Some(guard)
    } else {
        None
    };
    let mut command = if prepared.program == "git" {
        hardened_git_command(&prepared.cwd)
    } else {
        Command::new(&prepared.program)
    };
    command.args(&prepared.args).current_dir(&prepared.cwd);
    let output = run_bounded_process(&mut command, PROCESS_TIMEOUT, MAX_PROCESS_OUTPUT_BYTES)?;
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        if !combined.is_empty() && !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push_str(&stderr);
    }
    if output.truncated {
        combined.push_str("\n[output truncated at 65536 bytes; process terminated]\n");
    }
    if output.timed_out {
        combined.push_str("\n[process timed out after 30 seconds and was terminated]\n");
    }
    if !output.status.success() && !output.truncated && !output.timed_out {
        return Err(DesktopClientError::CommandFailed(combined));
    }
    Ok(combined)
}

#[cfg(test)]
mod bounded_process_tests {
    use super::*;

    #[test]
    fn process_timeout_kills_and_reports() {
        let mut command = Command::new("sleep");
        command.arg("5");
        let output = run_bounded_process(&mut command, std::time::Duration::from_millis(25), 1024)
            .expect("bounded sleep");
        assert!(output.timed_out);
        assert!(!output.status.success());
    }

    #[test]
    fn process_timeout_kills_term_ignoring_descendants_and_closes_pipes() {
        let root = std::env::temp_dir().join(format!("uor-grandchild-term-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create TERM-ignoring fixture");
        let pid_path = root.join("sleep.pid");
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(format!(
            "nohup sh -c 'trap \"\" TERM; sleep 30' >/dev/null 2>&1 & echo $! > '{}'; exec 1>&- 2>&-; trap \"\" TERM; sleep 30",
            pid_path.display()
        ));
        let output = run_bounded_process(&mut command, std::time::Duration::from_millis(25), 1024)
            .expect("bounded shell with TERM-ignoring descendant");
        assert!(output.timed_out);
        let pid = fs::read_to_string(&pid_path)
            .expect("TERM-ignoring descendant pid")
            .trim()
            .to_string();
        let alive = Command::new("ps")
            .arg("-p")
            .arg(&pid)
            .status()
            .expect("ps")
            .success();
        assert!(
            !alive,
            "TERM-ignoring descendant {pid} survived process timeout"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn process_timeout_kills_descendants() {
        let root = std::env::temp_dir().join(format!("uor-grandchild-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create grandchild fixture");
        let pid_path = root.join("sleep.pid");
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(format!(
            "nohup sleep 30 >/dev/null 2>&1 & echo $! > '{}'; wait",
            pid_path.display()
        ));
        let output = run_bounded_process(&mut command, std::time::Duration::from_millis(25), 1024)
            .expect("bounded shell with descendant");
        assert!(output.timed_out);
        let pid = fs::read_to_string(&pid_path)
            .expect("descendant pid")
            .trim()
            .to_string();
        let alive = Command::new("ps")
            .arg("-p")
            .arg(&pid)
            .status()
            .expect("ps")
            .success();
        assert!(!alive, "descendant {pid} survived process timeout");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn exited_parent_with_pipe_holding_descendant_returns_bounded() {
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg("sleep 30 & exit 0");
        let output = run_bounded_process(&mut command, std::time::Duration::from_millis(50), 1024)
            .expect("exited parent with open descendant pipes must return bounded");
        assert!(output.timed_out, "open descendant pipes hid the timeout");
    }

    #[test]
    fn output_cap_does_not_impersonate_a_wall_clock_timeout() {
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg("yes x");
        let output = run_bounded_process(&mut command, std::time::Duration::from_secs(1), 64)
            .expect("output cap must terminate without a timeout claim");
        assert!(output.truncated, "output cap was not reported");
        assert!(!output.timed_out, "output cap was mislabeled as a timeout");
    }
}

fn scope_manifest_filename(scope: &IdentityScope) -> String {
    let sanitize = |s: &str| -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .take(48)
            .collect()
    };
    let digest = hex::encode(Sha256::digest(scope.key().as_bytes()));
    format!(
        "{}__{}__{}__{}.uorman01",
        sanitize(&scope.user_id),
        sanitize(&scope.project_id),
        sanitize(&scope.session_id),
        &digest[..16],
    )
}

fn write_atomic_file(path: &Path, bytes: &[u8]) -> Result<(), DesktopClientError> {
    let name = path.file_name().ok_or_else(|| {
        DesktopClientError::InvalidInput("durable manifest filename is unavailable".to_string())
    })?;
    let temporary = path.with_file_name(format!("{}.tmp", name.to_string_lossy()));
    reject_if_symlink(&temporary)?;
    fs::write(&temporary, bytes).map_err(|error| DesktopClientError::Io(error.to_string()))?;
    fs::rename(&temporary, path).map_err(|error| DesktopClientError::Io(error.to_string()))
}

pub fn reject_parent_dir_traversal(path: &Path) -> Result<(), DesktopClientError> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(DesktopClientError::PathTraversalRefused(
            path.to_string_lossy().into_owned(),
        ));
    }
    Ok(())
}

pub fn reject_if_symlink(path: &Path) -> Result<(), DesktopClientError> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err(DesktopClientError::SymlinkRefused(
                path.to_string_lossy().into_owned(),
            ));
        }
    }
    Ok(())
}

fn reject_symlinks_between_root_and_target(
    canonical_root: &Path,
    target: &Path,
) -> Result<(), DesktopClientError> {
    reject_if_symlink(target)?;
    let mut cur = target.parent();
    while let Some(p) = cur {
        if let Ok(can_p) = fs::canonicalize(p) {
            if can_p == canonical_root || !can_p.starts_with(canonical_root) {
                break;
            }
        }
        reject_if_symlink(p)?;
        cur = p.parent();
    }
    Ok(())
}

fn collect_directory_files(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<String>,
) -> Result<(), DesktopClientError> {
    if depth > max_depth {
        return Ok(());
    }
    let read_dir = fs::read_dir(current).map_err(|e| DesktopClientError::Io(e.to_string()))?;
    let mut entries: Vec<PathBuf> = read_dir
        .filter_map(|e| e.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for path in entries {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || matches!(name, "target" | "node_modules" | "dist" | "local") {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_directory_files(root, &path, depth + 1, max_depth, out)?;
        } else if meta.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_str = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(rel_str);
            }
        }
    }
    Ok(())
}

// =========================================================================
// Deterministic synthetic `UORLUT01` stand-in builder, compiled directly
// from `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs` with zero
// code duplication.
// =========================================================================

mod mk_synthetic_stack_bin {
    #![allow(dead_code, clippy::map_identity)]
    include!(concat!(env!("OUT_DIR"), "/mk_synthetic_stack_included.rs"));

    pub(super) fn build(seed: u64) -> Vec<u8> {
        artifact(seed)
    }
}

/// Build the deterministic synthetic `UORLUT01` stack stand-in for `seed` (default `5`),
/// compiled directly from `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`.
pub fn build_synthetic_stack_artifact(seed: u64) -> Vec<u8> {
    mk_synthetic_stack_bin::build(seed)
}

// =========================================================================
// JSON-RPC command dispatcher & loopback Studio host (`--stdio`, `--serve-studio`)
// =========================================================================

/// Dispatch a single page-invocable desktop command against [`DesktopBackend`].
///
/// Note that `native_grant_folder`, `native_set_terminal_enabled`, and
/// `host_approve_terminal_command` are host-owned methods and are intentionally not exposed to
/// unprivileged page RPC calls.
pub fn dispatch_rpc_command(
    backend: &mut DesktopBackend,
    cmd: &str,
    args: &Value,
) -> Result<Value, String> {
    match cmd {
        "native_list_local_models" => {
            if let Some(dir) = args.get("modelDir").and_then(Value::as_str) {
                backend
                    .add_granted_model_search_dir(PathBuf::from(dir))
                    .map_err(|e| e.to_string())?;
            }
            let models = backend
                .native_list_local_models()
                .map_err(|e| e.to_string())?;
            serde_json::to_value(models).map_err(|e| e.to_string())
        }
        "native_load_synthetic_standin" => {
            let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
            let sha = sha256_bytes(&bytes);
            let caps = backend
                .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
                .map_err(|e| e.to_string())?;
            serde_json::to_value(caps).map_err(|e| e.to_string())
        }
        "native_load_model" => {
            if let Some(raw_bytes) = args.get("bytes").and_then(Value::as_array) {
                let mut buf = Vec::with_capacity(raw_bytes.len());
                for v in raw_bytes {
                    let b = v
                        .as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(|| "invalid byte in bytes array".to_string())?;
                    buf.push(b);
                }
                let file_name = args
                    .get("fileName")
                    .and_then(Value::as_str)
                    .unwrap_or(SYNTHETIC_STANDIN_FILE_NAME);
                let expected = args.get("expectedSha256").and_then(Value::as_str);
                let caps = backend
                    .native_load_model_bytes(file_name, &buf, expected)
                    .map_err(|e| e.to_string())?;
                return serde_json::to_value(caps).map_err(|e| e.to_string());
            }
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath or bytes".to_string())?;
            let expected = args.get("expectedSha256").and_then(Value::as_str);
            let caps = backend
                .native_load_model(Path::new(file_path), expected)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(caps).map_err(|e| e.to_string())
        }
        "native_step_tokens" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let max_new = args
                .get("maxNewTokens")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(32);
            let mut prompt_tokens = Vec::new();
            if let Some(arr) = args.get("tokenIds").and_then(Value::as_array) {
                for v in arr {
                    let t = v
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| "invalid token id".to_string())?;
                    prompt_tokens.push(t);
                }
            }
            let mut events = Vec::new();
            let receipt = backend
                .native_step_tokens(conv_id, &prompt_tokens, max_new, |ev| events.push(ev))
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "receipt": receipt,
                "events": events,
            }))
        }
        "native_generate_prompt" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let prompt_text = args
                .get("promptText")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing promptText".to_string())?;
            let max_new = args
                .get("maxNewTokens")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(32);
            let mut events = Vec::new();
            let receipt = backend
                .native_generate_prompt(conv_id, prompt_text, max_new, |ev| events.push(ev))
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "receipt": receipt,
                "events": events,
            }))
        }
        "native_export_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let (bytes, last_token) = backend
                .native_export_session(conv_id)
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "conversationId": conv_id,
                "bytes": bytes,
                "lastToken": last_token,
            }))
        }
        "native_import_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing conversationId".to_string())?;
            let arr = args
                .get("bytes")
                .and_then(Value::as_array)
                .ok_or_else(|| "missing bytes".to_string())?;
            let mut buf = Vec::with_capacity(arr.len());
            for v in arr {
                let b = v
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| "invalid byte".to_string())?;
                buf.push(b);
            }
            let last_token = args
                .get("lastToken")
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok());
            let verified_len = backend
                .native_import_session(conv_id, &buf, last_token)
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "conversationId": conv_id,
                "bytes": verified_len,
            }))
        }
        "native_reset_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            backend
                .native_reset_session(conv_id)
                .map_err(|e| e.to_string())?;
            Ok(json!({ "conversationId": conv_id, "reset": true }))
        }
        "native_save_session_durable" => {
            let user_id = args
                .get("userId")
                .and_then(Value::as_str)
                .unwrap_or("local-user");
            let project_id = args
                .get("projectId")
                .and_then(Value::as_str)
                .unwrap_or("studio");
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            if let Some(dir) = args.get("storeDir").and_then(Value::as_str) {
                backend
                    .set_durable_store_dir(dir)
                    .map_err(|e| e.to_string())?;
            }
            let receipt = backend
                .native_save_session_durable(user_id, project_id, conv_id)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(receipt).map_err(|e| e.to_string())
        }
        "native_restore_session_durable" => {
            let user_id = args
                .get("userId")
                .and_then(Value::as_str)
                .unwrap_or("local-user");
            let project_id = args
                .get("projectId")
                .and_then(Value::as_str)
                .unwrap_or("studio");
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            if let Some(dir) = args.get("storeDir").and_then(Value::as_str) {
                backend
                    .set_durable_store_dir(dir)
                    .map_err(|e| e.to_string())?;
            }
            let manifest_vec: Option<Vec<u8>> =
                match args.get("manifestBytes").and_then(Value::as_array) {
                    Some(arr) => {
                        let mut buf = Vec::with_capacity(arr.len());
                        for v in arr {
                            let b = v
                                .as_u64()
                                .and_then(|n| u8::try_from(n).ok())
                                .ok_or_else(|| "invalid manifest byte".to_string())?;
                            buf.push(b);
                        }
                        Some(buf)
                    }
                    None => None,
                };
            let receipt = backend
                .native_restore_session_durable(
                    user_id,
                    project_id,
                    conv_id,
                    manifest_vec.as_deref(),
                )
                .map_err(|e| e.to_string())?;
            serde_json::to_value(receipt).map_err(|e| e.to_string())
        }
        "native_list_granted_folders" => {
            let folders: Vec<String> = backend
                .granted_folders()
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            Ok(json!({ "folders": folders }))
        }
        "native_grant_folder" => Err(
            "PathNotGranted: folder grants are host-owned (--grant <dir>) and cannot be granted by page RPC"
                .to_string(),
        ),
        "native_read_file" => {
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath".to_string())?;
            let text = backend
                .native_read_file(Path::new(file_path))
                .map_err(|e| e.to_string())?;
            Ok(Value::String(text))
        }
        "native_write_file" => {
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath".to_string())?;
            let content = args
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing content".to_string())?;
            backend
                .native_write_file(Path::new(file_path), content)
                .map_err(|e| e.to_string())?;
            Ok(json!({ "written": true }))
        }
        "native_list_directory" => {
            let folder_path = args
                .get("folderPath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing folderPath".to_string())?;
            let max_depth = args
                .get("maxDepth")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(4);
            let files = backend
                .native_list_directory(Path::new(folder_path), max_depth)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(files).map_err(|e| e.to_string())
        }
        "native_git_status" => {
            let repo_path = args
                .get("repoPath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing repoPath".to_string())?;
            let report = backend
                .native_git_status(Path::new(repo_path))
                .map_err(|e| e.to_string())?;
            serde_json::to_value(report).map_err(|e| e.to_string())
        }
        "native_run_terminal_command" => {
            let cwd = args
                .get("cwd")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing cwd".to_string())?;
            let command = args
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing command".to_string())?;
            let user_confirmed = args
                .get("userConfirmed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let confirmed_command = args
                .get("confirmedCommand")
                .and_then(Value::as_str)
                .unwrap_or("");
            let req = TerminalCommandRequest {
                cwd: PathBuf::from(cwd),
                command: command.to_string(),
                user_confirmed,
                confirmed_command: confirmed_command.to_string(),
            };
            let output = backend
                .native_run_terminal_command(&req)
                .map_err(|e| e.to_string())?;
            Ok(Value::String(output))
        }
        "native_set_terminal_enabled" => Err(
            "TerminalDisabledByDefault: terminal enablement is host-owned and cannot be toggled by page RPC"
                .to_string(),
        ),
        other => Err(format!("unknown desktop command: {other}")),
    }
}

/// Parse and execute a single JSON-RPC line (`{"id": ..., "command": ..., "args": ...}`).
pub fn handle_rpc_line(backend: &mut DesktopBackend, line: &str) -> Value {
    match serde_json::from_str::<Value>(line) {
        Ok(req) => {
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let cmd = req.get("command").and_then(Value::as_str).unwrap_or("");
            let payload = req.get("args").cloned().unwrap_or_else(|| json!({}));
            match dispatch_rpc_command(backend, cmd, &payload) {
                Ok(result) => json!({ "id": id, "ok": true, "result": result }),
                Err(error) => json!({ "id": id, "ok": false, "error": error }),
            }
        }
        Err(e) => json!({ "id": Value::Null, "ok": false, "error": e.to_string() }),
    }
}

/// Run the end-to-end synthetic stand-in and durable memory self-test and return a JSON receipt.
pub fn run_self_test_receipt() -> Result<Value, String> {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let sha = sha256_bytes(&bytes);
    let caps = backend
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .map_err(|e| e.to_string())?;
    let mut streamed = Vec::new();
    let receipt = backend
        .native_step_tokens("self-test", &[0], 32, |ev| streamed.push(ev.token_id))
        .map_err(|e| e.to_string())?;
    let durable = backend
        .native_save_session_durable("local-user", "studio", "self-test")
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "verdict": "PASS",
        "engine": caps.engine,
        "artifactBytes": caps.artifact_bytes,
        "artifactSha256": caps.artifact_sha256,
        "weightsPerToken": caps.weights_per_token,
        "tokens32": receipt.token_ids,
        "streamedCount": streamed.len(),
        "durableScope": durable.scope_key,
        "objectVersion": durable.object_version_hex,
        "ur4sess1Bytes": durable.ur4sess1_bytes_len,
        "storeRecords": durable.store_record_count,
    }))
}

/// Generate a 256-bit (64-hex-character) per-launch token for the loopback Studio host.
pub fn generate_launch_token() -> String {
    use std::io::Read;
    let mut seed_bytes = [0u8; 32];
    let mut filled = false;
    if let Ok(mut f) = fs::File::open("/dev/urandom") {
        if f.read_exact(&mut seed_bytes).is_ok() {
            filled = true;
        }
    }
    let mut hasher = Sha256::new();
    hasher.update(b"UOR-DESKTOP-LAUNCH-TOKEN-V1");
    hasher.update(seed_bytes);
    hasher.update(std::process::id().to_le_bytes());
    if let Ok(dur) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        hasher.update(dur.as_nanos().to_le_bytes());
    }
    if !filled {
        hasher.update(b"fallback-entropy");
    }
    hex::encode(hasher.finalize())
}

fn find_header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim())
}

/// Handle a single loopback HTTP request (`GET /...` for static `web/studio/` assets or
/// `POST /__tauri_invoke` for `window.__TAURI__.core.invoke` JSON-RPC commands).
///
/// Enforces:
/// 1. `Host` header must match `bound_host` (`127.0.0.1:<port>`) on every request (blocks DNS rebinding).
/// 2. On `POST /__tauri_invoke`, `Origin` must equal `http://<bound_host>`, `Content-Type` must
///    start with `application/json` (blocks `no-cors` simple requests), and `X-UOR-Desktop-Token`
///    must match the per-launch `expected_token` (embedded in the served `index.html`).
#[allow(clippy::too_many_arguments)]
pub fn handle_studio_http_request(
    backend: &mut DesktopBackend,
    studio_root: &Path,
    bound_host: &str,
    expected_token: &str,
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> (u16, &'static str, Vec<u8>) {
    let Some(req_host) = find_header(headers, "host") else {
        return (
            403,
            "text/plain; charset=utf-8",
            b"Forbidden: missing Host header".to_vec(),
        );
    };
    if !req_host.eq_ignore_ascii_case(bound_host) {
        return (
            403,
            "text/plain; charset=utf-8",
            b"Forbidden: Host header mismatch".to_vec(),
        );
    }

    if method == "POST" && path == "/__tauri_invoke" {
        let expected_origin = format!("http://{bound_host}");
        let origin_ok = find_header(headers, "origin")
            .is_some_and(|o| o.eq_ignore_ascii_case(&expected_origin));
        let content_type_ok = find_header(headers, "content-type").is_some_and(|ct| {
            ct.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("application/json")
        });
        let token_ok = !expected_token.is_empty()
            && find_header(headers, "x-uor-desktop-token").is_some_and(|t| t == expected_token);

        if !origin_ok || !content_type_ok || !token_ok {
            return (
                403,
                "application/json; charset=utf-8",
                br#"{"ok":false,"error":"Forbidden: invalid Origin, Content-Type, or X-UOR-Desktop-Token"}"#
                    .to_vec(),
            );
        }

        let line = String::from_utf8_lossy(body);
        let resp = handle_rpc_line(backend, &line);
        let bytes = serde_json::to_vec(&resp).unwrap_or_default();
        return (200, "application/json; charset=utf-8", bytes);
    }
    if method != "GET" {
        return (
            405,
            "text/plain; charset=utf-8",
            b"Method Not Allowed".to_vec(),
        );
    }
    let clean_path = path.split('?').next().unwrap_or("/");
    let rel = if clean_path == "/" || clean_path == "/studio" || clean_path == "/studio/" {
        "index.html"
    } else {
        clean_path
            .strip_prefix("/studio/")
            .or_else(|| clean_path.strip_prefix('/'))
            .unwrap_or(clean_path)
    };
    let candidate = studio_root.join(rel);
    if Path::new(rel).is_absolute()
        || reject_parent_dir_traversal(Path::new(rel)).is_err()
        || reject_symlinks_between_root_and_target(studio_root, &candidate).is_err()
    {
        return (403, "text/plain; charset=utf-8", b"Forbidden".to_vec());
    }
    let canonical_root = match fs::canonicalize(studio_root) {
        Ok(root) => root,
        Err(_) => return (404, "text/plain; charset=utf-8", b"Not Found".to_vec()),
    };
    let canonical_candidate = match fs::canonicalize(&candidate) {
        Ok(path) if path.starts_with(&canonical_root) => path,
        Ok(_) => return (403, "text/plain; charset=utf-8", b"Forbidden".to_vec()),
        Err(_) => return (404, "text/plain; charset=utf-8", b"Not Found".to_vec()),
    };
    let Ok(mut bytes) = fs::read(&canonical_candidate) else {
        return (404, "text/plain; charset=utf-8", b"Not Found".to_vec());
    };
    if rel == "index.html" && !expected_token.is_empty() {
        let html = String::from_utf8_lossy(&bytes);
        let bootstrap = format!(
            "<script>window.UOR_DESKTOP_SESSION_TOKEN={:?};window.UOR_ENABLE_LOOPBACK_TAURI=true;</script>\n    <script src=\"./assets/js/uor_offline_vendor.js\"></script>",
            expected_token
        );
        let injected = html.replacen(
            "<script type=\"module\">",
            &format!("{bootstrap}\n    <script type=\"module\">"),
            1,
        );
        bytes = injected.into_bytes();
    }
    let content_type = if rel.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if rel.ends_with(".js") || rel.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if rel.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if rel.ends_with(".svg") {
        "image/svg+xml"
    } else if rel.ends_with(".json") {
        "application/json; charset=utf-8"
    } else {
        "application/octet-stream"
    };
    (200, content_type, bytes)
}

/// Native Tauri v2 WebView shell (`--features gui`) wiring `#[tauri::command]` handlers and
/// `tauri::Builder` over [`DesktopBackend`].
#[cfg(feature = "gui")]
pub mod tauri_gui {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::mcp::McpManager;
    use super::sidecar::{NativeChatEvent, NativeProxyResponse, SidecarProcess};
    use super::{
        dispatch_rpc_command, ensure_git_repository_root, git_diff_at, git_status_at,
        run_prepared_terminal_command, DesktopBackend, GitDiffReport, GitStatusReport, PinRegistry,
        TerminalCommandRequest,
    };
    use serde_json::{json, Value};
    use tauri::Manager;
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

    #[derive(Clone)]
    pub struct TauriDesktopState {
        inner: Arc<Mutex<DesktopBackend>>,
        sidecar: Option<SidecarProcess>,
        mcp: McpManager,
    }

    impl TauriDesktopState {
        pub fn new(backend: DesktopBackend) -> Self {
            Self {
                inner: Arc::new(Mutex::new(backend)),
                sidecar: None,
                mcp: McpManager::default(),
            }
        }

        pub fn with_sidecar(binary: &Path, stack: &Path, tokenizer: &Path) -> Result<Self, String> {
            Self::with_backend_and_sidecar(
                DesktopBackend::new(PinRegistry::with_synthetic_standin()),
                binary,
                stack,
                tokenizer,
            )
        }

        pub fn with_backend_and_sidecar(
            backend: DesktopBackend,
            binary: &Path,
            stack: &Path,
            tokenizer: &Path,
        ) -> Result<Self, String> {
            Ok(Self {
                inner: Arc::new(Mutex::new(backend)),
                sidecar: Some(SidecarProcess::start(binary, stack, tokenizer)?),
                mcp: McpManager::default(),
            })
        }

        pub fn sidecar(&self) -> Result<SidecarProcess, String> {
            self.sidecar
                .clone()
                .ok_or_else(|| "The native uor-chat sidecar is unavailable.".to_string())
        }

        pub fn with_backend<T>(
            &self,
            f: impl FnOnce(&mut DesktopBackend) -> Result<T, String>,
        ) -> Result<T, String> {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "DesktopBackend mutex poisoned".to_string())?;
            f(&mut guard)
        }

        pub fn grant_project_folder(&self, path: &Path) -> Result<PathBuf, String> {
            self.with_backend(|backend| {
                backend
                    .native_grant_folder(path)
                    .map_err(|error| error.to_string())
            })
        }

        pub fn grant_model_folder(&self, path: &Path) -> Result<PathBuf, String> {
            self.with_backend(|backend| {
                backend
                    .grant_model_search_dir(path)
                    .map_err(|error| error.to_string())
            })
        }

        pub fn git_status(&self, repo: &Path) -> Result<GitStatusReport, String> {
            let canonical = self.with_backend(|backend| {
                backend
                    .verify_granted_existing_path(repo)
                    .map_err(|error| error.to_string())
            })?;
            git_status_at(&canonical).map_err(|error| error.to_string())
        }

        pub fn git_diff(&self, repo: &Path) -> Result<GitDiffReport, String> {
            let canonical = self.with_backend(|backend| {
                backend
                    .verify_granted_existing_path(repo)
                    .map_err(|error| error.to_string())
            })?;
            git_diff_at(&canonical).map_err(|error| error.to_string())
        }

        pub fn enable_terminal(&self) -> Result<(), String> {
            self.with_backend(|backend| {
                backend.native_set_terminal_enabled(true);
                Ok(())
            })
        }

        pub fn approve_terminal_command(&self, cwd: &Path, command: &str) -> Result<(), String> {
            self.with_backend(|backend| {
                backend
                    .host_approve_terminal_command(cwd, command)
                    .map_err(|error| error.to_string())
            })
        }

        pub fn run_terminal_command(
            &self,
            request: &TerminalCommandRequest,
        ) -> Result<String, String> {
            let prepared = self.with_backend(|backend| {
                backend
                    .prepare_terminal_command(request)
                    .map_err(|error| error.to_string())
            })?;
            if prepared.program == "git" {
                ensure_git_repository_root(&prepared.cwd).map_err(|error| error.to_string())?;
            }
            run_prepared_terminal_command(&prepared).map_err(|error| error.to_string())
        }

        pub fn connect_mcp_remote(&self, url: &str, credential: &str) -> Result<String, String> {
            self.mcp
                .connect_remote(url, credential, super::mcp::McpLimits::default())
        }

        pub fn connect_mcp_stdio(
            &self,
            executable: &Path,
            args: &[String],
            cwd: &Path,
        ) -> Result<String, String> {
            let canonical_cwd = self.with_backend(|backend| {
                backend
                    .verify_granted_existing_path(cwd)
                    .map_err(|error| error.to_string())
            })?;
            self.mcp.connect_stdio(
                executable,
                args,
                &canonical_cwd,
                super::mcp::McpLimits::default(),
            )
        }

        pub fn list_mcp_tools(&self, session_id: &str) -> Result<Vec<Value>, String> {
            self.mcp.list_tools(session_id)
        }

        pub fn call_mcp_tool(
            &self,
            session_id: &str,
            name: &str,
            args: &Value,
        ) -> Result<Value, String> {
            self.mcp.call_tool(session_id, name, args)
        }

        pub fn disconnect_mcp(&self, session_id: &str) -> Result<(), String> {
            self.mcp.disconnect(session_id)
        }
    }

    fn main_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
        app.get_webview_window("main")
            .ok_or_else(|| "The main Studio window is unavailable.".to_string())
    }

    fn confirmation_dialog(
        app: &tauri::AppHandle,
        message: impl Into<String>,
    ) -> Result<tauri_plugin_dialog::MessageDialogBuilder<tauri::Wry>, String> {
        let window = main_window(app)?;
        Ok(window.dialog().message(message).parent(&window))
    }

    pub(crate) fn studio_navigation_allowed(value: &str) -> bool {
        let Ok(url) = reqwest::Url::parse(value) else {
            return false;
        };
        if !url.username().is_empty() || url.password().is_some() {
            return false;
        }
        match url.scheme() {
            "tauri" => url.host_str() == Some("localhost") && url.port().is_none(),
            "http" | "https" => {
                !cfg!(target_os = "macos")
                    && url.host_str() == Some("tauri.localhost")
                    && url.port().is_none()
            }
            _ => false,
        }
    }

    pub async fn connect_mcp_remote_on_worker(
        state: TauriDesktopState,
        url: String,
        credential: String,
    ) -> Result<String, String> {
        tauri::async_runtime::spawn_blocking(move || state.connect_mcp_remote(&url, &credential))
            .await
            .map_err(|error| format!("native MCP worker failed: {error}"))?
    }

    fn sidecar_input_paths() -> Result<(PathBuf, PathBuf, PathBuf), String> {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        sidecar_input_paths_for(&executable, Path::new(env!("CARGO_MANIFEST_DIR")))
    }

    pub(crate) fn sidecar_input_paths_for(
        executable: &Path,
        manifest_dir: &Path,
    ) -> Result<(PathBuf, PathBuf, PathBuf), String> {
        let macos = executable
            .parent()
            .ok_or_else(|| "The executable has no parent directory.".to_string())?;
        let packaged = executable
            .components()
            .any(|component| component.as_os_str().to_string_lossy().ends_with(".app"));
        if packaged {
            let resources = macos
                .parent()
                .ok_or_else(|| "The executable has no resource directory.".to_string())?
                .join("Resources");
            return Ok((
                macos.join("uor-chat"),
                resources.join("pkg/synthetic-stack.lut"),
                resources.join("pkg/tokenizer.json"),
            ));
        }
        let development = (
            manifest_dir.join("binaries/uor-chat-aarch64-apple-darwin"),
            manifest_dir.join("runtime/studio/pkg/synthetic-stack.lut"),
            manifest_dir.join("runtime/studio/pkg/tokenizer.json"),
        );
        if [&development.0, &development.1, &development.2]
            .iter()
            .all(|path| path.is_file())
        {
            return Ok(development);
        }
        let resources = macos
            .parent()
            .ok_or_else(|| "The executable has no resource directory.".to_string())?
            .join("Resources");
        Ok((
            macos.join("uor-chat"),
            resources.join("pkg/synthetic-stack.lut"),
            resources.join("pkg/tokenizer.json"),
        ))
    }

    /// Test helper that dispatches a command name and JSON payload through [`TauriDesktopState`].
    pub fn invoke_tauri_command_for_test(
        state: &TauriDesktopState,
        cmd: &str,
        args: Value,
    ) -> Result<Value, String> {
        state.with_backend(|b| dispatch_rpc_command(b, cmd, &args))
    }

    #[tauri::command(async)]
    pub fn native_models(
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<NativeProxyResponse, String> {
        state.sidecar()?.models()
    }

    #[tauri::command(async)]
    pub fn native_decompose_image(
        state: tauri::State<'_, TauriDesktopState>,
        width: u32,
        height: u32,
        columns: u32,
        rows: u32,
        rgba: Vec<u8>,
    ) -> Result<NativeProxyResponse, String> {
        state
            .sidecar()?
            .decompose(width, height, columns, rows, &rgba)
    }

    #[tauri::command(async)]
    pub fn host_cancel_chat(
        state: tauri::State<'_, TauriDesktopState>,
        request_id: String,
    ) -> Result<(), String> {
        state.sidecar()?.cancel(&request_id)
    }

    #[tauri::command(async)]
    pub fn native_chat_completions(
        state: tauri::State<'_, TauriDesktopState>,
        request_id: String,
        request: String,
        on_event: tauri::ipc::Channel<NativeChatEvent>,
    ) -> Result<NativeProxyResponse, String> {
        let sidecar = state.sidecar()?;
        let canceller = sidecar.clone();
        let cancellation_id = request_id.clone();
        let (initial_sender, initial_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut initial_sender = Some(initial_sender);
            let mut last_response = None;
            let events = on_event.clone();
            let result = sidecar.stream_chat(&request_id, &request, |event| {
                last_response = Some((event.status, event.headers.clone()));
                if let Some(sender) = initial_sender.take() {
                    let _ = sender.send(Ok(NativeProxyResponse {
                        status: event.status,
                        headers: event.headers.clone(),
                        body: Vec::new(),
                    }));
                }
                let _ = on_event.send(event);
            });
            if let Err(error) = result {
                let (status, headers) = last_response.unwrap_or((500, BTreeMap::new()));
                if let Some(sender) = initial_sender.take() {
                    let _ = sender.send(Err(error.clone()));
                }
                let _ = events.send(NativeChatEvent {
                    status,
                    headers,
                    body_chunk: Vec::new(),
                    terminal: true,
                    error: Some(error),
                });
            }
        });
        match initial_receiver.recv_timeout(Duration::from_secs(15)) {
            Ok(result) => result,
            Err(_) => {
                canceller.cancel(&cancellation_id)?;
                Err("The native chat request did not return its initial response.".to_string())
            }
        }
    }

    #[tauri::command(async)]
    pub fn native_list_local_models(
        state: tauri::State<'_, TauriDesktopState>,
        model_dir: Option<String>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_list_local_models",
                &json!({ "modelDir": model_dir }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_load_synthetic_standin(
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<Value, String> {
        state.with_backend(|b| dispatch_rpc_command(b, "native_load_synthetic_standin", &json!({})))
    }

    #[tauri::command(async)]
    pub fn native_load_model(
        state: tauri::State<'_, TauriDesktopState>,
        file_path: Option<String>,
        bytes: Option<Vec<u8>>,
        file_name: Option<String>,
        expected_sha256: Option<String>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_load_model",
                &json!({
                    "filePath": file_path,
                    "bytes": bytes,
                    "fileName": file_name,
                    "expectedSha256": expected_sha256,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_step_tokens(
        state: tauri::State<'_, TauriDesktopState>,
        conversation_id: Option<String>,
        token_ids: Option<Vec<u32>>,
        max_new_tokens: Option<usize>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_step_tokens",
                &json!({
                    "conversationId": conversation_id,
                    "tokenIds": token_ids,
                    "maxNewTokens": max_new_tokens,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_generate_prompt(
        state: tauri::State<'_, TauriDesktopState>,
        conversation_id: Option<String>,
        prompt_text: String,
        max_new_tokens: Option<usize>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_generate_prompt",
                &json!({
                    "conversationId": conversation_id,
                    "promptText": prompt_text,
                    "maxNewTokens": max_new_tokens,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_export_session(
        state: tauri::State<'_, TauriDesktopState>,
        conversation_id: Option<String>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_export_session",
                &json!({ "conversationId": conversation_id }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_import_session(
        state: tauri::State<'_, TauriDesktopState>,
        conversation_id: String,
        bytes: Vec<u8>,
        last_token: Option<u32>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_import_session",
                &json!({
                    "conversationId": conversation_id,
                    "bytes": bytes,
                    "lastToken": last_token,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_reset_session(
        state: tauri::State<'_, TauriDesktopState>,
        conversation_id: Option<String>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_reset_session",
                &json!({ "conversationId": conversation_id }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_save_session_durable(
        state: tauri::State<'_, TauriDesktopState>,
        user_id: Option<String>,
        project_id: Option<String>,
        conversation_id: Option<String>,
        store_dir: Option<String>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_save_session_durable",
                &json!({
                    "userId": user_id,
                    "projectId": project_id,
                    "conversationId": conversation_id,
                    "storeDir": store_dir,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_restore_session_durable(
        state: tauri::State<'_, TauriDesktopState>,
        user_id: Option<String>,
        project_id: Option<String>,
        conversation_id: Option<String>,
        store_dir: Option<String>,
        manifest_bytes: Option<Vec<u8>>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_restore_session_durable",
                &json!({
                    "userId": user_id,
                    "projectId": project_id,
                    "conversationId": conversation_id,
                    "storeDir": store_dir,
                    "manifestBytes": manifest_bytes,
                }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_list_granted_folders(
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<Value, String> {
        state.with_backend(|b| dispatch_rpc_command(b, "native_list_granted_folders", &json!({})))
    }

    #[tauri::command(async)]
    pub fn native_read_file(
        state: tauri::State<'_, TauriDesktopState>,
        file_path: String,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(b, "native_read_file", &json!({ "filePath": file_path }))
        })
    }

    #[tauri::command(async)]
    pub fn native_write_file(
        state: tauri::State<'_, TauriDesktopState>,
        file_path: String,
        content: String,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_write_file",
                &json!({ "filePath": file_path, "content": content }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_list_directory(
        state: tauri::State<'_, TauriDesktopState>,
        folder_path: String,
        max_depth: Option<usize>,
    ) -> Result<Value, String> {
        state.with_backend(|b| {
            dispatch_rpc_command(
                b,
                "native_list_directory",
                &json!({ "folderPath": folder_path, "maxDepth": max_depth }),
            )
        })
    }

    #[tauri::command(async)]
    pub fn native_git_status(
        state: tauri::State<'_, TauriDesktopState>,
        repo_path: String,
    ) -> Result<Value, String> {
        serde_json::to_value(state.git_status(Path::new(&repo_path))?)
            .map_err(|error| error.to_string())
    }

    #[tauri::command(async)]
    pub fn native_git_diff(
        state: tauri::State<'_, TauriDesktopState>,
        repo_path: String,
    ) -> Result<Value, String> {
        serde_json::to_value(state.git_diff(Path::new(&repo_path))?)
            .map_err(|error| error.to_string())
    }

    #[tauri::command(async)]
    pub async fn native_choose_project_folder(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<Value, String> {
        let Some(path) = app.dialog().file().blocking_pick_folder() else {
            return Ok(Value::Null);
        };
        let path = path.into_path().map_err(|error| error.to_string())?;
        let granted = state.grant_project_folder(&path)?;
        Ok(json!({ "path": granted }))
    }

    #[tauri::command(async)]
    pub async fn native_choose_model_folder(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<Value, String> {
        let Some(path) = app.dialog().file().blocking_pick_folder() else {
            return Ok(Value::Null);
        };
        let path = path.into_path().map_err(|error| error.to_string())?;
        let granted = state.grant_model_folder(&path)?;
        Ok(json!({ "path": granted }))
    }

    #[tauri::command(async)]
    pub async fn native_enable_terminal(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
    ) -> Result<Value, String> {
        let confirmed = confirmation_dialog(&app, "Enable the guarded terminal for this app session? Every command will still require separate approval.")?
            .title("Enable terminal")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show();
        if !confirmed {
            return Ok(json!({ "enabled": false }));
        }
        state.enable_terminal()?;
        Ok(json!({ "enabled": true }))
    }

    #[tauri::command(async)]
    pub async fn native_confirm_terminal_command(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
        cwd: String,
        command: String,
    ) -> Result<Value, String> {
        super::validate_consent_text(&cwd).map_err(|error| error.to_string())?;
        super::validate_consent_text(&command).map_err(|error| error.to_string())?;
        let confirmed = confirmation_dialog(
            &app,
            format!("Run this command once?\n\nWorking directory:\n{cwd}\n\nCommand:\n{command}"),
        )?
        .title("Approve command")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show();
        if !confirmed {
            return Ok(json!({ "approved": false }));
        }
        state.approve_terminal_command(Path::new(&cwd), &command)?;
        Ok(json!({ "approved": true }))
    }

    #[tauri::command(async)]
    pub fn native_run_terminal_command(
        state: tauri::State<'_, TauriDesktopState>,
        cwd: String,
        command: String,
        user_confirmed: Option<bool>,
        confirmed_command: Option<String>,
    ) -> Result<Value, String> {
        let request = TerminalCommandRequest {
            cwd: PathBuf::from(cwd),
            command,
            user_confirmed: user_confirmed.unwrap_or(false),
            confirmed_command: confirmed_command.unwrap_or_default(),
        };
        Ok(Value::String(state.run_terminal_command(&request)?))
    }

    #[tauri::command(async)]
    pub async fn native_mcp_connect_remote(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
        url: String,
        credential: String,
    ) -> Result<Value, String> {
        super::validate_consent_text(&url).map_err(|error| error.to_string())?;
        let credential_notice = if credential.is_empty() {
            ""
        } else {
            "\n\nA credential will be sent to this server."
        };
        let confirmed = confirmation_dialog(
            &app,
            format!("Connect to this MCP server?\n\n{url}{credential_notice}"),
        )?
        .title("Approve remote MCP server")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show();
        if !confirmed {
            return Ok(Value::Null);
        }
        let session_id =
            connect_mcp_remote_on_worker(state.inner().clone(), url, credential).await?;
        Ok(json!({ "sessionId": session_id }))
    }

    #[tauri::command(async)]
    pub fn native_mcp_disconnect(
        state: tauri::State<'_, TauriDesktopState>,
        session_id: String,
    ) -> Result<Value, String> {
        state.disconnect_mcp(&session_id)?;
        Ok(json!({ "disconnected": true }))
    }

    #[tauri::command(async)]
    pub fn native_mcp_list_tools(
        state: tauri::State<'_, TauriDesktopState>,
        session_id: String,
    ) -> Result<Value, String> {
        serde_json::to_value(state.list_mcp_tools(&session_id)?).map_err(|error| error.to_string())
    }

    #[tauri::command(async)]
    pub fn native_mcp_call_tool(
        state: tauri::State<'_, TauriDesktopState>,
        session_id: String,
        name: String,
        args: Value,
    ) -> Result<Value, String> {
        state.call_mcp_tool(&session_id, &name, &args)
    }

    #[tauri::command(async)]
    pub async fn native_mcp_connect_stdio(
        app: tauri::AppHandle,
        state: tauri::State<'_, TauriDesktopState>,
        executable: String,
        args: Vec<String>,
        cwd: String,
    ) -> Result<Value, String> {
        super::validate_consent_text(&executable).map_err(|error| error.to_string())?;
        super::validate_consent_text(&cwd).map_err(|error| error.to_string())?;
        for argument in &args {
            super::validate_consent_text(argument).map_err(|error| error.to_string())?;
        }
        let confirmed = confirmation_dialog(&app, format!(
                "Start this MCP stdio server?\n\nExecutable:\n{executable}\n\nArguments:\n{args:?}\n\nWorking directory:\n{cwd}"
            ))?
            .title("Approve MCP stdio server")
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show();
        if !confirmed {
            return Ok(Value::Null);
        }
        let worker_state = state.inner().clone();
        let session_id = tauri::async_runtime::spawn_blocking(move || {
            worker_state.connect_mcp_stdio(Path::new(&executable), &args, Path::new(&cwd))
        })
        .await
        .map_err(|error| format!("native MCP stdio worker failed: {error}"))??;
        Ok(json!({ "sessionId": session_id }))
    }

    /// Construct the configured [`tauri::Builder`] with managed [`TauriDesktopState`] and all
    /// 30 native/host `#[tauri::command]` handlers registered.
    pub fn build_tauri_app_builder(state: TauriDesktopState) -> tauri::Builder<tauri::Wry> {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .setup(|app| {
                tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("UOR Harness")
                .inner_size(1440.0, 900.0)
                .resizable(true)
                .fullscreen(false)
                .on_navigation(|url| {
                    if studio_navigation_allowed(url.as_str()) {
                        return true;
                    }
                    if matches!(url.scheme(), "http" | "https") {
                        let _ = std::process::Command::new("open")
                            .arg(url.as_str())
                            .status();
                    }
                    false
                })
                .build()?;
                Ok(())
            })
            .manage(state)
            .invoke_handler(tauri::generate_handler![
                native_models,
                native_decompose_image,
                native_chat_completions,
                host_cancel_chat,
                native_list_local_models,
                native_load_synthetic_standin,
                native_load_model,
                native_step_tokens,
                native_generate_prompt,
                native_export_session,
                native_import_session,
                native_reset_session,
                native_save_session_durable,
                native_restore_session_durable,
                native_list_granted_folders,
                native_read_file,
                native_write_file,
                native_list_directory,
                native_git_status,
                native_git_diff,
                native_choose_project_folder,
                native_choose_model_folder,
                native_enable_terminal,
                native_confirm_terminal_command,
                native_run_terminal_command,
                native_mcp_connect_remote,
                native_mcp_list_tools,
                native_mcp_call_tool,
                native_mcp_disconnect,
                native_mcp_connect_stdio,
            ])
    }

    /// Launch the native Tauri v2 desktop window wrapping `web/studio/`.
    pub fn run_tauri_gui(backend: DesktopBackend) -> Result<(), String> {
        let (binary, stack, tokenizer) = sidecar_input_paths()?;
        let state =
            TauriDesktopState::with_backend_and_sidecar(backend, &binary, &stack, &tokenizer)?;
        let exit_sidecar = state.sidecar.clone();
        let app = build_tauri_app_builder(state)
            .build(tauri::generate_context!())
            .map_err(|e| e.to_string())?;
        app.run(move |_app, event| {
            if matches!(
                event,
                tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
            ) {
                if let Some(sidecar) = &exit_sidecar {
                    sidecar.shutdown();
                }
            }
        });
        Ok(())
    }
}

#[cfg(all(test, feature = "gui"))]
mod sidecar_path_tests {
    use super::tauri_gui::{sidecar_input_paths_for, studio_navigation_allowed};
    use std::path::Path;

    #[test]
    fn navigation_is_confined_to_the_actual_platform_asset_origin() {
        assert!(studio_navigation_allowed("tauri://localhost"));
        #[cfg(target_os = "macos")]
        {
            assert!(!studio_navigation_allowed("http://tauri.localhost"));
            assert!(!studio_navigation_allowed("https://tauri.localhost"));
            assert!(!studio_navigation_allowed("http://tauri.localhost:80"));
            assert!(!studio_navigation_allowed("https://tauri.localhost:443"));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(studio_navigation_allowed("http://tauri.localhost"));
            assert!(studio_navigation_allowed("https://tauri.localhost"));
            assert!(studio_navigation_allowed("http://tauri.localhost:80"));
            assert!(studio_navigation_allowed("https://tauri.localhost:443"));
        }
        assert!(!studio_navigation_allowed("https://example.com"));
        assert!(!studio_navigation_allowed("http://127.0.0.1:8000"));
        assert!(!studio_navigation_allowed("tauri://evil"));
        assert!(!studio_navigation_allowed("tauri://user@localhost"));
        assert!(!studio_navigation_allowed("http://tauri.localhost:8000"));
        assert!(!studio_navigation_allowed("https://tauri.localhost:444"));
        assert!(!studio_navigation_allowed("https://user@tauri.localhost"));
        assert!(!studio_navigation_allowed("file:///tmp/private.html"));
    }

    #[test]
    fn packaged_app_prefers_its_own_resources_over_development_inputs() {
        let executable = Path::new("/tmp/UOR Harness.app/Contents/MacOS/uor-desktop-client");
        let (binary, stack, tokenizer) = sidecar_input_paths_for(executable, Path::new("/source"))
            .expect("packaged sidecar paths");
        assert!(binary.ends_with("UOR Harness.app/Contents/MacOS/uor-chat"));
        assert!(stack.ends_with("UOR Harness.app/Contents/Resources/pkg/synthetic-stack.lut"));
        assert!(tokenizer.ends_with("UOR Harness.app/Contents/Resources/pkg/tokenizer.json"));
    }
}
