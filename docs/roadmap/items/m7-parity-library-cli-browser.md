---
id: m7-parity-library-cli-browser
title: Build the library, CLI and browser bit-identity parity harness
milestone: M7
area: serving
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m7-studio-connect-wasm-engine, m4-export-uorlut01-cli]
scope: [crates/uor-r4-integer/**, web/**, docs/products/**, docs/labs/m7-parity-library-cli-browser/**]
---

## Why it matters

This is the closing bar of M7 (issue #2035): the same model must give the same tokens whether it is called from Rust, from the `uor-chat` command line or from a browser worker. It also tests that a session saved in one process restores identically in a fresh one. It sits at the end of the serving path, after the model is exported to `UORLUT01` and before the alpha release package (M8). See [serving and export](../../model/serving-and-export.md).

## What is missing

No harness yet compares tokens and session snapshots across the Rust library (`IntegerCapabilityApi`), the `uor-chat` command line and the browser worker on one artifact. The Node save and restore check (`docs/wasm-serving/verify.mjs`) passes only on a synthetic stand-in model, and it does not run from a clean clone: it reads `uor_r4_integer_bg.wasm` and `uor_r4_integer.js` (the `wasm-pack` output) and `synthetic-stack.lut` from the current directory, and none of the three is committed.

## Done when

M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. Today there is no accepted artifact, so a harness that passes on the synthetic stack (`mk-synthetic-stack`) shows the harness works and is not a milestone result. The bar is not yet registered for how many prompts, how many tokens per prompt or which sampler settings the comparison covers; registering it is step 2 below.

## How to start

1. Read `crates/uor-r4-integer/src/capability_api.rs` and `crates/uor-r4-integer/tests/capability_api_tests.rs` (how a session is created, stepped, saved with `save_session` and restored with `restore_session`). Then read `crates/uor-r4-integer/src/bin/uor-chat.rs`, `crates/uor-r4-integer/src/wasm_serve.rs` and `crates/uor-r4-integer/src/session_codec.rs`. Note that `IntegerCapabilityApi::new` takes a `Bundle`, while the browser `loadStack` takes `UORLUT01` bytes and saves sessions in the `UR4SESS1` byte format. Run the baseline with `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm` (the 18 October record reports 165 passing).
2. Post a design note on the issue. It must say: which Rust entry point is the reference for `UORLUT01` (the library API or the stack path used by `uor-chat`), what "identical snapshot" means across the library snapshot and the `UR4SESS1` bytes, and the exact comparison: a fixed prompt list, a fixed token count, the sampler policy (greedy, temperature 0, as the browser engine selects greedily) and read mode, tokens compared as integer ids, snapshots compared as bytes. State that the harness uses no softmax, no matmul and no floating-point sampling, and that CPU controls are finite and deterministic. Ask the PM to register the prompt list and token count as the bar.
3. Write the failing test first: a Rust test that loads the synthetic stack from `mk-synthetic-stack` (`~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack`; it writes `synthetic-stack.lut` in the current directory unless a second argument gives a path), runs the library and `uor-chat` paths (`uor-chat --stack <artifact>` also requires `--tokenizer <tokenizer.json>`, so the harness must supply one), and compares ids and snapshots. Keep it a host check; it needs no GPU.
4. Add the browser leg. Build the engine as in the 18 October record (`RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`), then drive it from Node with a Rust-first harness that follows the call sequence in `docs/wasm-serving/verify.mjs` (that script reads `./uor_r4_integer_bg.wasm`, `./uor_r4_integer.js` and `./synthetic-stack.lut` from the current directory, so run it from a folder that holds the `wasm-pack` output, which lands in `crates/uor-r4-integer/pkg/`, and the synthetic stack file) (`loadStack`, `step`, `saveSession`, `restoreSession`). Keep any small `.mjs` shim to a minimum, and add no Python. Repeat in the Studio worker once `m7-studio-connect-wasm-engine` lands.
5. Add a fresh-process restore check: save in process A, restore in process B, continue, and compare against one uninterrupted run, in all three paths.
6. Freeze and hash the inputs (prompt list, artifact, build commit) in a one-page pre-registration. Run the harness on the synthetic stack and post the result. Re-run on the accepted artifact once `m4-export-uorlut01-cli` produces one.
7. Open a PR that says `References #<this issue>` (it says `Closes` only after the accepted artifact passes), and add one row to `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:20`](../../../ROADMAP.md)
- issue#2032
- issue#2035
- [`docs/labs/wasm-serving-runtime-2026-10-18/README.md`](../../labs/wasm-serving-runtime-2026-10-18/README.md)
