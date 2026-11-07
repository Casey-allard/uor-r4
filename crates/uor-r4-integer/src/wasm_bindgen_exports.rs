//! `wasm-bindgen` exports for the #1172 Studio worker.
//!
//! This is the JavaScript-callable surface over [`crate::wasm_serve`]. It
//! owns no logic beyond argument marshalling; every function forwards. The
//! worker contract (`r4_worker.js`): `init()` once, `load_stack(bytes)`,
//! then `step(handle, token)` in a loop, `save`/`restore` for the
//! reload-identical test, `weights_per_token` for the R3 report, and
//! `alloc_events` to prove zero per-token allocation.
//!
//! The turn memory (`uor-r4.lut-stack/7`, units 2-3 of `turn-memory-integer`): the JS caller
//! reads `memoryInfo` after loading, calls `markReplyStart` just before the step that ingests the
//! prompt's last token (so that row is the opening table's row), and does not restore a memory
//! session, which is refused until its turn state is saved.
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
    Ok(vec![out.next as i32, out.margin_q16, out.position as i32])
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

/// The turn memory the loaded model serves, as a JSON object string, or the string `null`
/// (`JSON.parse` gives the object or null). Its fields: `present`, `k`, `key_window`,
/// `reply_turn`, `opening_entries`, `reply_entries`, `reply_key_len`, `weight_exp`.
#[wasm_bindgen(js_name = memoryInfo)]
pub fn memory_info(handle: u32) -> Result<String, JsError> {
    crate::wasm_serve::wasm_memory_info(handle).map_err(|e| JsError::new(&e))
}

/// Mark that the reply begins at the next token stepped (units 2-3 of `turn-memory-integer`):
/// the next `step` call ingests the prompt's last token, so its row is the opening table's row.
#[wasm_bindgen(js_name = markReplyStart)]
pub fn mark_reply_start(handle: u32) -> Result<(), JsError> {
    crate::wasm_serve::wasm_mark_reply_start(handle).map_err(|e| JsError::new(&e))
}

/// Clear the reply-start signal.
#[wasm_bindgen(js_name = clearReplyStart)]
pub fn clear_reply_start(handle: u32) -> Result<(), JsError> {
    crate::wasm_serve::wasm_clear_reply_start(handle).map_err(|e| JsError::new(&e))
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

/// Load a `uor-r4.lut-discrete/1` M1 artifact (the trained discrete runtime).
#[wasm_bindgen(js_name = loadDiscrete)]
pub fn load_discrete(artifact: &[u8]) -> Result<u32, JsError> {
    crate::wasm_serve::wasm_load_discrete(artifact).map_err(|e| JsError::new(&e))
}

/// Evaluate a causal prefix; returns JSON with the top-1/top-8 next tokens.
#[wasm_bindgen(js_name = predictDiscrete)]
pub fn predict_discrete(handle: u32, ids: &[u32]) -> Result<String, JsError> {
    crate::wasm_serve::wasm_discrete_predict(handle, ids).map_err(|e| JsError::new(&e))
}

/// Static facts about a loaded discrete model, as JSON.
#[wasm_bindgen(js_name = discreteInfo)]
pub fn discrete_info(handle: u32) -> Result<String, JsError> {
    crate::wasm_serve::wasm_discrete_info(handle).map_err(|e| JsError::new(&e))
}

/// Answer a strict decimal expression without invoking a model or its token accounting.
#[wasm_bindgen(js_name = exactArithmetic)]
pub fn exact_arithmetic(prompt: &str) -> Result<String, JsError> {
    let result = match crate::m6_exact_arithmetic::exact_arithmetic_tool(prompt) {
        None => serde_json::json!({"handled": false}),
        Some(Ok(answer)) => serde_json::json!({
            "handled": true,
            "expression": answer.expression,
            "result": answer.result,
            "text": answer.text,
            "tool": "m6-exact-arithmetic"
        }),
        Some(Err(error)) => serde_json::json!({"handled": true, "error": error.to_string()}),
    };
    serde_json::to_string(&result).map_err(|error| JsError::new(&error.to_string()))
}
