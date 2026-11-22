//! Safe public host/WASM facade over the `uor-r4-mvm` RV64IMAC Linux MicroVM.

use serde::{Deserialize, Serialize};

/// Structured command execution result returned by [`MvmSandbox::exec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecResult {
    /// Captured standard output UTF-8 text.
    pub stdout: String,
    /// Captured standard error UTF-8 text.
    pub stderr: String,
    /// Final shell/process exit code (`0` on success, `127` when unavailable/not found).
    pub exit_code: i32,
    /// Guest working directory after the command completes.
    pub cwd: String,
    /// Total RV64IMAC instructions retired during execution.
    pub instructions_retired: u64,
    /// `e_machine` value (`243` for `EM_RISCV`) if this invocation compiled an ELF64 binary.
    pub compiled_elf_machine: Option<u16>,
    /// FIPS 180-4 SHA-256 hex digest of the logical guest filesystem and CoW state.
    pub cow_state_sha256: String,
}

/// Safe public facade over an in-memory RV64IMAC Linux MicroVM sandbox.
#[derive(Clone)]
pub struct MvmSandbox {
    inner: crate::mvm::MvmSandbox,
}

impl MvmSandbox {
    /// Boot a fresh RV64IMAC MicroVM with the pinned `UORMVM01` initramfs (`CID = 3`).
    pub fn new() -> Result<Self, String> {
        let inner = crate::mvm::MvmSandbox::boot_default().map_err(|e| e.to_string())?;
        Ok(Self { inner })
    }

    /// Current guest working directory.
    pub fn cwd(&self) -> &str {
        self.inner.current_cwd()
    }

    /// Guest architecture string and `AF_VSOCK` Context ID.
    pub fn guest_identity(&self) -> (&'static str, u16) {
        (self.inner.architecture(), self.inner.guest_cid())
    }

    /// Execute a command or pipeline inside the MicroVM, updating `cwd` cleanly across `cd`.
    pub fn exec(&mut self, command: &str, cwd: &str) -> ExecResult {
        let effective_cwd = if cwd.trim().is_empty() {
            self.inner.current_cwd().to_string()
        } else {
            cwd.to_string()
        };
        if command.trim().is_empty() {
            return ExecResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                cwd: self.inner.current_cwd().to_string(),
                instructions_retired: 0,
                compiled_elf_machine: None,
                cow_state_sha256: self.inner.cow_state_sha256(),
            };
        }
        match self.inner.exec_command(command, &effective_cwd) {
            Ok(outcome) => ExecResult {
                stdout: outcome.stdout,
                stderr: outcome.stderr,
                exit_code: outcome.exit_code,
                cwd: self.inner.current_cwd().to_string(),
                instructions_retired: outcome.instructions_retired,
                compiled_elf_machine: outcome.compiled_elf_machine,
                cow_state_sha256: outcome.cow_state_sha256,
            },
            Err(err) => ExecResult {
                stdout: String::new(),
                stderr: format!("{err}\n"),
                exit_code: 1,
                cwd: self.inner.current_cwd().to_string(),
                instructions_retired: 0,
                compiled_elf_machine: None,
                cow_state_sha256: self.inner.cow_state_sha256(),
            },
        }
    }

    /// Read a guest file (resolving symlinks inside the sandbox).
    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.inner.read_guest_file(path)
    }

    /// Write a guest file, enforcing protected canonical path admission.
    pub fn write_file(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.inner
            .write_guest_file(path, bytes)
            .map_err(|e| e.to_string())
    }

    /// Remove a regular guest file or symlink, refusing protected canonical paths.
    pub fn remove_file(&mut self, path: &str) -> Result<(), String> {
        self.inner
            .remove_guest_file(path)
            .map_err(|e| e.to_string())
    }

    /// List normalized guest file paths matching `prefix` (`"/"` or `"/workspace"`).
    pub fn list_files(&self, prefix: &str) -> Vec<String> {
        self.inner.list_guest_paths(prefix)
    }

    /// Export a self-contained binary Copy-on-Write snapshot with SHA-256 verification.
    pub fn export_cow_snapshot(&self) -> Result<Vec<u8>, String> {
        self.inner.export_cow_snapshot_bytes()
    }

    /// Restore the MicroVM filesystem and CoW sectors from a snapshot produced by
    /// [`Self::export_cow_snapshot`], verifying magic, lengths, and state digest.
    pub fn import_cow_snapshot(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.inner.import_cow_snapshot_bytes(bytes)
    }
}
