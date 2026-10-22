# ORDER: m7-studio-connect-wasm-engine, antigravity-20

Lab: antigravity. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md). Base: `2ec16d513`.

## Goal

The item's done-when reads:

> M7 criterion 3: the Studio runs entirely client-side in a browser Web Worker with truthful capability reporting and zero remote model calls; the `.wasm` module passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7 PASS, mul=0, div=0, float=0). The 18 October record counts 7 serving functions, not whole modules.

## Cancel probe (run first)

Verified before starting implementation:
- `uor-harness loop next` and `uor-harness loop arrive` from `/Users/casey.allard/uor-r4/.worktrees/antigravity-main-verify` report `kill-switch: off`.
- Issue `#2868` (`m7-studio-connect-wasm-engine`) is open, `status: ready`, `gpu: false`, with zero active claims in `uor-harness team claims`.
- Pre-implementation design note posted on [`#2868`](https://github.com/UOR-Foundation/uor-r4/issues/2868#issuecomment-6345487875) and [`#2035`](https://github.com/UOR-Foundation/uor-r4/issues/2035#issuecomment-6345488418).

## Files in scope

- `web/**`
- `crates/uor-r4-integer/**`
- `docs/products/**`
- `docs/labs/m7-studio-connect-wasm-engine/**`

## Done when

1. `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm` and `cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack` produce the serving WASM bundle and synthetic stack fixture, and `docs/wasm-serving/verify.mjs` passes (`ALL CHECKS PASSED`).
2. `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap` passes (`7/7 PASS`, `mul=0`, `div=0`, `float=0`).
3. `web/studio/assets/js/uor_model_worker.js`, `web/studio/r4_worker.js`, and `web/studio/index.html` connect to `crates/uor-r4-integer` (`init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `allocEvents`, `leakedModels`), report truthful header/engine capabilities (including `"synthetic stand-in"` for the synthetic fixture), keep the D36 status banner accurate (`0/8` milestones, `35.1%` tok/s), and disable all remote inference/search pathways (`fetchLiveWebKnowledge` / `toggleWebSearch`).
4. An automated Node worker test (`web/studio/verify_studio_worker.mjs`) loads the synthetic stack through the Studio worker, verifies the exact 32-token output sequence against `docs/wasm-serving/verify.mjs` (`[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]`), verifies `saveSession`/`restoreSession` mid-stream continuation bit-identity, and asserts zero remote network calls.
5. Verification is recorded in `docs/products/web-client.md` and `docs/labs/m7-studio-connect-wasm-engine/README.md`.

## Stop and ask if

- Unfreezing GitHub Pages deployment (`.github/workflows/deploy.yml` or D36) is needed.
- Any file outside `web/**`, `crates/uor-r4-integer/**`, `docs/products/**`, `docs/labs/m7-studio-connect-wasm-engine/**`, or the roadmap/task files needs modification.
