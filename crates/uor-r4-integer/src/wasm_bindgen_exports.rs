//! `wasm-bindgen` exports for the #1172 Studio worker.
//!
//! This is the JavaScript-callable surface over [`crate::wasm_serve`]. It
//! owns no logic beyond argument marshalling; every function forwards. The
//! worker contract (`r4_worker.js`): `init()` once, `load_stack(bytes)`,
//! then `step(handle, token)` in a loop, `save`/`restore` for the
//! reload-identical test, `weights_per_token` for the R3 report, and
//! `alloc_events` to prove zero per-token allocation.
//!
//! Host clock: `tick_clock` is called by the JS glue from
//! `performance.now()` at the start of every generate/step batch so the
//! generation telemetry (`duration_nanos_exact`) carries measured values.
//! If the host never calls it, those fields report exact zeroes — the
//! "unmeasured" contract in `wasm_host.rs`.

#![cfg(all(target_arch = "wasm32", feature = "serving-wasm"))]

use wasm_bindgen::prelude::*;

/// Module init. Registers the host clock seam and the panic hook so a panic
/// surfaces as a JS exception with a message, not a silent trap.
#[wasm_bindgen(start)]
pub fn init() {
    crate::wasm_host::set_host_clock();
}

/// Push a monotonic host clock reading (nanoseconds). The JS glue calls this
/// from `performance.now()` at the start of every step batch.
#[wasm_bindgen(js_name = tickClock)]
pub fn tick_clock(ns: u64) {
    crate::wasm_host::tick_host_clock(ns);
}

/// Load a `UORLUT01` stack artifact from bytes. Returns the session handle.
#[wasm_bindgen(js_name = loadStack)]
pub fn load_stack(artifact: &[u8]) -> Result<u32, JsError> {
    crate::wasm_serve::wasm_load_stack(artifact).map_err(|e| JsError::new(&e))
}

/// Feed one token; returns `[next_token, margin_q16, position]`.
#[wasm_bindgen(js_name = step)]
pub fn step(handle: u32, token: u32) -> Result<Vec<i32>, JsError> {
    let out = crate::wasm_serve::wasm_step(handle, token).map_err(|e| JsError::new(&e))?;
    Ok(vec![
        out.next as i32,
        out.margin_q16,
        out.position as i32,
    ])
}

/// Serialize the session state.
#[wasm_bindgen(js_name = saveSession)]
pub fn save_session(handle: u32) -> Result<Vec<u8>, JsError> {
    crate::wasm_serve::wasm_save(handle).map_err(|e| JsError::new(&e))
}

/// Restore a serialized session into the same loaded model.
#[wasm_bindgen(js_name = restoreSession)]
pub fn restore_session(handle: u32, state: &[u8]) -> Result<(), JsError> {
    crate::wasm_serve::wasm_restore(handle, state).map_err(|e| JsError::new(&e))
}

/// Weights touched per token (R3 per-token parameter reads).
#[wasm_bindgen(js_name = weightsPerToken)]
pub fn weights_per_token(handle: u32) -> Result<u64, JsError> {
    crate::wasm_serve::wasm_weights_per_token(handle).map_err(|e| JsError::new(&e))
}

/// Allocation events since module init (loads only).
#[wasm_bindgen(js_name = allocEvents)]
pub fn alloc_events() -> u64 {
    crate::wasm_serve::wasm_alloc_events()
}

/// Models leaked for the module lifetime (one per `loadStack`).
#[wasm_bindgen(js_name = leakedModels)]
pub fn leaked_models() -> u64 {
    crate::wasm_serve::wasm_leaked_models()
}
