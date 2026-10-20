//! Native Geometric Language Model WebAssembly runtime facade.
//!
//! Exposes thread-safe in-browser inference, streaming, cancellation,
//! and durable memory controls over `uor_r4_api::native_capability_api`.

use std::sync::{Mutex, OnceLock};
use uor_r4_api::native_capability_api::{NativeModel, WasmModelRuntime};

static NATIVE_RUNTIME: OnceLock<Mutex<WasmModelRuntime>> = OnceLock::new();

pub fn get_runtime() -> Result<&'static Mutex<WasmModelRuntime>, String> {
    NATIVE_RUNTIME
        .get()
        .ok_or_else(|| "No native model loaded; supply a retained artifact first".into())
}

pub fn init(model_bytes: &[u8]) -> Result<String, String> {
    // Validate before replacing the runtime, preserving the previous model on error.
    let model = NativeModel::load_from_bytes(model_bytes)
        .map_err(|e| format!("Failed to load native geometric model: {e}"))?;
    let runtime = WasmModelRuntime::new(model);
    if let Some(rt) = NATIVE_RUNTIME.get() {
        *rt.lock()
            .map_err(|e| format!("Runtime lock poisoned: {e}"))? = runtime;
    } else {
        NATIVE_RUNTIME
            .set(Mutex::new(runtime))
            .map_err(|_| "Concurrent native model initialization".to_string())?;
    }
    let guard = get_runtime()?
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    Ok(guard.wasm_get_capabilities())
}

pub fn create_session(session_id: &str, user_id: &str, project_id: &str) -> Result<u32, String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_create_session(session_id, user_id, project_id)
        .map_err(|e| format!("Session creation failed: {e}"))
}

pub fn ingest(handle: u32, text: &str) -> Result<String, String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_ingest(handle, text)
        .map_err(|e| format!("Ingest failed: {e}"))
}

pub fn generate_step(handle: u32, max_tokens: usize) -> Result<String, String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_generate_step(handle, max_tokens)
        .map_err(|e| format!("Generation step failed: {e}"))
}

pub fn finish_generation(handle: u32) -> Result<String, String> {
    let guard = get_runtime()?
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_finish_generation(handle)
        .map_err(|e| format!("Finish generation failed: {e}"))
}

pub fn cancel(handle: u32) -> Result<(), String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_cancel(handle)
        .map_err(|e| format!("Cancel failed: {e}"))
}

pub fn export_session(handle: u32) -> Result<Vec<u8>, String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_export_session(handle)
        .map_err(|e| format!("Export failed: {e}"))
}

pub fn import_session(handle: u32, bytes: &[u8]) -> Result<(), String> {
    let rt = get_runtime()?;
    let guard = rt
        .lock()
        .map_err(|e| format!("Runtime lock poisoned: {e}"))?;
    guard
        .wasm_import_session(handle, bytes)
        .map_err(|e| format!("Import failed: {e}"))
}

pub fn free_session(handle: u32) {
    if let Ok(rt) = get_runtime() {
        if let Ok(guard) = rt.lock() {
            guard.wasm_free_session(handle);
        }
    }
}

pub fn capabilities() -> String {
    if let Ok(rt) = get_runtime() {
        if let Ok(guard) = rt.lock() {
            return guard.wasm_get_capabilities();
        }
    }
    String::new()
}

// ============================================================================
// WASM-bindgen interface exports
// ============================================================================

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_init(model_bytes: &[u8]) -> Result<String, JsValue> {
    init(model_bytes).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_create_session(
    session_id: &str,
    user_id: &str,
    project_id: &str,
) -> Result<u32, JsValue> {
    create_session(session_id, user_id, project_id).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_ingest(handle: u32, text: &str) -> Result<String, JsValue> {
    ingest(handle, text).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_generate_step(handle: u32, max_tokens: usize) -> Result<String, JsValue> {
    generate_step(handle, max_tokens).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_finish_generation(handle: u32) -> Result<String, JsValue> {
    finish_generation(handle).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_cancel(handle: u32) -> Result<(), JsValue> {
    cancel(handle).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_export_session(handle: u32) -> Result<Vec<u8>, JsValue> {
    export_session(handle).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_import_session(handle: u32, bytes: &[u8]) -> Result<(), JsValue> {
    import_session(handle, bytes).map_err(|e| JsValue::from_str(&e))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_free_session(handle: u32) {
    free_session(handle);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn native_geometric_capabilities() -> String {
    capabilities()
}
