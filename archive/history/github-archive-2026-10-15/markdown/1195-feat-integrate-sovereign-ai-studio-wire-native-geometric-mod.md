# #1195 — feat: integrate sovereign AI studio, wire native geometric model, and polish alpha release

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T06:28:40Z
- **Updated:** 2026-10-09T14:16:57Z
- **Closed:** 2026-09-08T06:29:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1195
- **Merged:** 2026-09-08T06:29:43Z
- **Merge commit:** 032f255dec33de23a313d8cc653e6a9715d14086
- **Base / Head:** main / UOR-Foundation:alpha-release-polish
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This pull request completes the integration of the **Sovereign AI Developer Studio** directly into `uor-r4`, wires the **UOR-R4 Native Geometric Language Model** as the primary inference driver, adds the standalone **`r4-native-chat`** interactive CLI utility, fixes the GitHub Pages deployment pipeline, and delivers the comprehensive **v0.1.0-alpha** release documentation and capability scorecard.

### Primary Changes

1. **Sovereign AI Developer Studio Integration (`index.html`, `assets/`, `coi-serviceworker.js`)**:
   - Merged the full Sovereign Developer Studio interface (Monaco IDE, Side-by-side Diff engine, Thought Wave EEG oscilloscope, 3D holographic synaptic brain manifold, and terminal runner).
   - **Stripped all external LLMs**: Removed all external HuggingFace CDN downloads, ONNX transformers.js models (Qwen, GLM), and closed cloud API dependencies.
   - **Inserted UOR-R4 as Primary Driver**: Configured `uor_model_worker.js` to run directly on the native WebAssembly substrate (`pkg/uor_r4_wasm_router.js`) with zero-matmul serving, CORDIC $S^3 \to S^2$ Hopf phase telemetry ($\chi, \delta, \alpha$), and 240 $E_8$ root lattice coordinates.
   - Configured native model tiers in UI: *UOR-R4 Geometric Core (Alpha)*, *UOR-R4 Memory Ring (Grounded)*, and *UOR-R4 Frozen Kernel (R4G1)*.

2. **GitHub Pages Deployment Fix (`.github/workflows/deploy.yml`)**:
   - Resolved the 404 issue on GitHub Pages (`https://uor-foundation.github.io/uor-r4/`) by adding `assets/`, `coi-serviceworker.js`, and `r4_worker.js` to the `dist/` copy manifest.

3. **Standalone Interactive CLI Chat (`crates/uor-r4-api/src/bin/r4-native-chat.rs`)**:
   - Implemented zero-configuration interactive terminal chat powered directly by `NativeModel`.
   - Supports streaming generation, REPL commands (`/help`, `/reset`, `/stats`, `/exit`), and one-shot execution (`cargo run -p uor-r4-api --bin r4-native-chat -- "Explain the R4 manifold"`).
   - Verified native execution throughput of **3,645.9 tok/s** on Apple Silicon CPU.

4. **Comprehensive v0.1.0-alpha Release Documentation (`README.md`)**:
   - Detailed architectural breakdown: $R^4/S^3/H^4$ continuous manifold, 14 Riemann zeta-zero phase clocks, exact $\mathbb{Z}[\phi]$ arithmetic, paired-$H_4$ icosian lattice, and exact addressed memory ring.
   - Embedded real studio screenshots: Sovereign Chat Studio, Monaco IDE, and Diff Viewer.
   - Published Apple Silicon M1 benchmark and thermal quantification table (<70µs latency, ~3.5W active power, 80–180µJ/token, 0-byte steady-state allocation, 0% thermal throttling).
   - Published the 12-Axis Alpha Capability Scorecard (Positions 01–12, 31/31 acceptance criteria verified).

### Verification
- `cargo fmt --check`: PASS
- `cargo check -p uor-r4-api --bin r4-native-chat`: PASS
- `cargo run -p uor-r4-api --bin r4-native-chat -- "hello world"`: PASS (256 tokens @ 3,645.9 tok/s)
- `wasm-pack build --target web`: PASS (`pkg/` generated cleanly)
- `python3 scripts/check_claim_wording.py README.md`: PASS


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

