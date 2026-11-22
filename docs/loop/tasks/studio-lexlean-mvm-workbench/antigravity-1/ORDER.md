# ORDER for upgrading `uor-r4-mvm` with LexLean-R4 RV64IMAC shell and WASM target

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `1e60cfab1`. Owner directed porting the LexLean-R4 browser MVM Micro-Linux terminal into UOR-Foundation/foundry inside `uor-r4` while excluding all shortcut/transformer modules (`wasm_ast_eval`, Qwen3/RoPE/SiLU/sigmoid, hardcoded interceptors).

## Cancel probe

Verify every ported module from `/Users/casey.allard/lexlean-r4` (`288e2677`) against `mvm_initramfs.rs` SHA-256 (`071e9c8d...`), `#![forbid(unsafe_code)]`, D23/D25/D32 ratchets, and `uor-r4/Cargo.lock`. Cancel immediately on any `unsafe`, floating-point squashing, hidden AST-eval shortcuts, or committed binaries (§3.13).

## Scope

Inside `crates/uor-r4-mvm/**` and `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-1/**` plus extending `docs/roadmap/items/studio-lexlean-mvm-workbench.md` `scope:` for the companion Foundry Web Studio + R4 integration PR:
- Port `mvm.rs`, `mvm_shell.rs`, `mvm_cc.rs`, `vsock.rs`, and the 5 pure admission state-machine modules (`lexlean_admission.rs`) from `lexlean-r4` `288e2677`, stripping `wasm_ast_eval::execute_workspace_cargo` into honest unavailable exit-127 errors.
- Export `MvmSandbox` (`src/sandbox.rs`) and target-gated `wasm` feature `WasmMvmSandbox` (`src/wasm.rs`) with CoW snapshot export/import while preserving `guest_identity`, `run_collatz`, and `uor-mvm` CLI.

## Done check

- `cargo test --locked -p uor-r4-mvm`, `--all-features`, `--doc`, `uor-mvm status|collatz`, and `wasm-pack build crates/uor-r4-mvm --target web --no-opt -- --locked --features wasm` all pass.
- `uor-harness ratchet d23|d25|d32`, `uor-r4-claim-wording`, and `uor-harness loop pr-check` pass with zero committed binaries.

## Stop and decide if

Stop if `uor-r4-mvm` `wasm32` compilation requires workspace `Cargo.lock` dependency additions outside existing pinned versions, introduces any ratchet regression, or triggers a hard `loop pr-check` blocker.
