//! WebAssembly `wasm-bindgen` bindings exposing [`WasmMvmSandbox`] to browser Web Workers.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::sandbox::MvmSandbox;

#[derive(Serialize)]
struct GuestIdentityPayload {
    architecture: &'static str,
    guest_cid: u16,
    cwd: String,
}

/// Browser WebAssembly handle owning one `MvmSandbox` RV64IMAC MicroVM instance.
#[wasm_bindgen]
pub struct WasmMvmSandbox {
    inner: MvmSandbox,
}

#[wasm_bindgen]
impl WasmMvmSandbox {
    /// Boot a fresh RV64IMAC MicroVM instance in WebAssembly.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmMvmSandbox, JsValue> {
        let inner = MvmSandbox::new().map_err(|err| JsValue::from_str(&err))?;
        Ok(Self { inner })
    }

    /// Execute a command or pipeline in `cwd` (or current cwd when empty) and return JSON `ExecResult`.
    pub fn exec_json(&mut self, cmd: &str, cwd: &str) -> String {
        let res = self.inner.exec(cmd, cwd);
        serde_json::to_string(&res).unwrap_or_else(|e| {
            format!(
                "{{\"stdout\":\"\",\"stderr\":\"serialization error: {e}\",\"exit_code\":1,\"cwd\":\"/workspace\",\"instructions_retired\":0,\"compiled_elf_machine\":null,\"cow_state_sha256\":\"\"}}"
            )
        })
    }

    /// Current guest working directory.
    pub fn cwd(&self) -> String {
        self.inner.cwd().to_string()
    }

    /// Read guest file bytes by path.
    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.inner.read_file(path)
    }

    /// Write guest file bytes by path.
    pub fn write_file(&mut self, path: &str, bytes: &[u8]) -> Result<(), JsValue> {
        self.inner
            .write_file(path, bytes)
            .map_err(|err| JsValue::from_str(&err))
    }

    /// Remove a non-protected guest file by path.
    pub fn remove_file(&mut self, path: &str) -> Result<(), JsValue> {
        self.inner
            .remove_file(path)
            .map_err(|err| JsValue::from_str(&err))
    }

    /// List guest files under `prefix` as a JSON array of paths.
    pub fn list_files_json(&self, prefix: &str) -> String {
        let paths = self.inner.list_files(prefix);
        serde_json::to_string(&paths).unwrap_or_else(|_| "[]".to_string())
    }

    /// Export a binary CoW snapshot for OPFS persistence.
    pub fn export_cow_snapshot(&self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .export_cow_snapshot()
            .map_err(|err| JsValue::from_str(&err))
    }

    /// Restore a binary CoW snapshot from OPFS persistence.
    pub fn import_cow_snapshot(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        self.inner
            .import_cow_snapshot(bytes)
            .map_err(|err| JsValue::from_str(&err))
    }

    /// Return `{ architecture, guest_cid, cwd }` as JSON.
    pub fn guest_identity_json(&self) -> String {
        let (architecture, guest_cid) = self.inner.guest_identity();
        let payload = GuestIdentityPayload {
            architecture,
            guest_cid,
            cwd: self.inner.cwd().to_string(),
        };
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    }
}
