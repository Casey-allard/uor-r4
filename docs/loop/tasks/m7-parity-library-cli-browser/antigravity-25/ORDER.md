# ORDER: m7-parity-library-cli-browser, antigravity-25

Lab: antigravity. Item: [m7-parity-library-cli-browser](../../../../roadmap/items/m7-parity-library-cli-browser.md). Base: `bca96e43d`.

## Goal

The item's done-when reads:

> M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. Today there is no accepted artifact, so a harness that passes on the synthetic stack (`mk-synthetic-stack`) shows the harness works and is not a milestone result. The bar is not yet registered for how many prompts, how many tokens per prompt or which sampler settings the comparison covers; registering it is step 2 below.

## Cancel probe (run first)

- `docs/labs/m7-parity-library-cli-browser/` does not exist yet (`0` comments on Issue `#2867`).
- `uor-chat --stack` (`crates/uor-r4-integer/src/bin/uor-chat.rs`) lacks `--prompt-ids <csv>`, `--save-session <path>`, and `--restore-session <path>` flags for `UR4SESS1` snapshots.
- No harness compares tokens and `UR4SESS1` snapshots across the Rust library, `uor-chat` CLI, and Browser WASM / Studio worker. Proceeding.

## Files in scope

- `crates/uor-r4-integer/src/bin/uor-chat.rs` (estimated ~160 lines: `--prompt-ids`, `--save-session`, `--restore-session` on `--stack`)
- `crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs` (estimated ~280 lines: 3-leg parity + fresh-process restore + planted-fault controls)
- `web/studio/verify_m7_parity.mjs` (estimated ~140 lines: Node WASM + `uor_model_worker.js` driver)
- `docs/labs/m7-parity-library-cli-browser/README.md` (estimated ~95 lines: lab record and staged `m4-serving.md` row)
- `docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md` (estimated ~80 lines: frozen input hashes and proposed M7 parity bar)
- `docs/products/web-client.md` (estimated ~10 lines: link parity harness)

## Done when

- Design note and bar-registration request are posted on Issue `#2867`.
- `uor-chat --stack` supports `--prompt-ids`, `--save-session`, and `--restore-session` via `UR4SESS1` (`session_codec`), with zero softmax, zero matmul, and zero float sampling.
- `m7_parity_library_cli_browser` and `verify_m7_parity.mjs` pass on the `seed = 5` `mk-synthetic-stack` (`48` uninterrupted tokens and `32 + 16` fresh-process restore across all 8 prompts) and fail under planted faults.
- `docs/labs/m7-parity-library-cli-browser/` records the frozen panel hashes, receipts, and staged `m4-serving.md` row (`References #2867`).

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**`, `web/**`, `docs/products/**`, or `docs/labs/m7-parity-library-cli-browser/**` needs modification.
- Registering the final M7 acceptance bar requires an owner/PM decision.
