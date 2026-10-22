// Compatibility forwarding entry point for `web/studio/r4_worker.js`.
// Delegates all Web Worker traffic to `assets/js/uor_model_worker.js`
// (`crates/uor-r4-integer` `--features serving-wasm`).
import './assets/js/uor_model_worker.js';
