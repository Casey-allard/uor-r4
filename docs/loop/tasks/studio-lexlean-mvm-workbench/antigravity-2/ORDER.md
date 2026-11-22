# ORDER for porting Foundry Studio, real UOR-R4 WASM serving, and RV64IMAC MVM terminal

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `a33cd2850`. Owner direction (2026-11-22): archive the retired Preact/Claude-Code-Desktop Web Studio (`web/studio/`) while preserving desktop/contract files, port `UOR-Foundation/foundry` (`ea917b8d`) into `web/studio/foundry/`, wire real UOR-R4 serving (`crates/uor-r4-integer` `serving-wasm` + OPFS cache + public M1 + local `.uorlut`) and the `uor-r4-mvm` RV64IMAC Micro-Linux interactive terminal (`WasmMvmSandbox` + OPFS CoW + `<tool_call>`/`<tool_response>` loop) into Foundry, and update staging/publishing scripts for `gh-pages` `/studio/`.

## Cancel probe

Verify zero committed binaries (§3.13), byte-identical archival (`archive/CATALOG.md` + `xtask archive-verify`), D23/D25/D32 ratchets, `audit_zero_matmul_wasm.py`, `pages_branch_policy`, `repo-conformance`, and real `uor_model_worker.js` + `WasmMvmSandbox` browser execution. Cancel immediately on any softmax/squashing, dense matmul, hardcoded model reply, AST-eval shortcut, or committed binary.

## Scope

Inside `web/studio/**`, `archive/web-studio-preact-2026-11/**`, `archive/{catalog.jsonl,CATALOG.md,catalog/m7-distribution.md}`, `scripts/studio/**`, `scripts/stage-studio-site.sh`, `scripts/tests/**`, `crates/uor-r4-wasm-router/tests/bdd.rs`, `Cargo.toml`, `.gitignore`, `PLAN.md`, `docs/products/web-client.md`, and `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-2/**`:
- Archive retired Preact `web/studio/` files byte-identical while keeping `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `r4_worker.js`, `coi-serviceworker.js`, `verify_m7_parity.mjs`, `verify_studio_worker.mjs`, `tools/make_tokenizer.mjs`, and `build_serve_wasm.sh`.
- Import text-only `UOR-Foundation/foundry` (`ea917b8d`) + `foundry-assets.lock.json` & `scripts/studio/fetch-foundry-assets.mjs` for locked binaries.
- Wire real `uor-r4-integer` WASM serving + OPFS cache (`models.json` + public M1 + local `.uorlut`) and `uor-r4-mvm` WASM RV64IMAC terminal + OPFS CoW persistence + `<tool_call>` harness + Foundry `TerminalPanel` tab.
- Build and verify via `build_foundry_studio.sh`, headless Chrome CDP `verify_foundry_studio.mjs`, M7 parity, Rust workspace tests, and dry-run `build-and-publish.sh`.

## Done check

- `build_foundry_studio.sh`, Chrome CDP `verify_foundry_studio.mjs`, `verify_m7_parity.mjs`, `audit_zero_matmul_wasm.py`, `cargo test` (`uor-r4-mvm`, `uor-r4-wasm-router`, `pages_branch_policy`, `repo-conformance`), `wasm-pack` on `uor-r4-wasm-router`, `xtask docs-catalog --check`, `xtask archive-verify`, `xtask links`, ratchets, `uor-r4-claim-wording`, dry-run `build-and-publish.sh`, and `uor-harness loop pr-check` all pass.

## Stop and decide if

Stop if Foundry Vite transforms, `stamp_shell.py`, `pages_branch_policy`, or `repo-conformance` fails twice, or if headless Chrome CDP E2E is blocked.
