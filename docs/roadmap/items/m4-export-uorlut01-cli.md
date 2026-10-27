---
id: m4-export-uorlut01-cli
title: Serve the chosen model as UORLUT01 through the uor-chat CLI
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory]
notebook: docs/research/notebook/m4-serving.md
scope: [crates/uor-r4-integer/**, crates/uor-r4-discrete/**, crates/uor-r4-lut/**, docs/mechanisms/lut-serving.md, docs/model/serving-and-export.md, docs/harness/prep/m4-export-uorlut01-cli*, docs/labs/m4-export-uorlut01-cli/**, docs/loop/tasks/m4-export-uorlut01-cli/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

M4 (issue #2032) is the step that turns the M1 to M3 work into one served artifact, which M5 to M8 all depend on. The integer serving engine and the `uor-chat` command line exist, but no trained model has been exported to them. Background: [serving and export](../../model/serving-and-export.md).

## What is missing

One chosen model must be exported as a single UORLUT01 file, serve through the `uor-chat` CLI, report its training-to-D11 export gap and meet the held-out bar; none exists. `STATUS.md:19` records that no model trained under today's rules has been exported, and `docs/model/architecture.md:81` records "no exporter". M4 is held.

The native and WASM serving audits pass on today's build (44 native symbol ranges, 7 of 7 WebAssembly serving functions; `docs/model/serving-and-export.md:51-52`) and must stay at 0 float, 0 softmax or squashing, 0 matmul and 0 mul/div on the final build.

Two things are open. First, `crates/uor-r4-integer/src/bin/uor-chat.rs` describes `--stack` as serving a UORLUT01 geometric-stack artifact, while the geometric stack is PARKED and the live model is the discrete one. Whether a discrete model fits UORLUT01, or needs new sections, is not settled in the sources. Second, the held-out panel behind the 0.90 bits-per-byte bar is not yet registered.

## Done when

M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. In addition:

- The export gap is reported (D37).
- The static audits are green.
- A fresh-process restore is bit-identical.

## How to start

1. **Read the contract and the code.** Open `docs/model/serving-and-export.md`, `docs/mechanisms/lut-serving.md`, `crates/uor-r4-lut/src/format.rs` (the UORLUT01 layout), `crates/uor-r4-integer/src/bin/uor-chat.rs` and `crates/uor-r4-integer/README.md`. `uor-chat` and `mk-synthetic-stack` are built only with `--features serving-wasm` (the README's `cargo build` line omits it). Read the headers of `scripts/audit_zero_matmul_serving.py` and `scripts/audit_zero_matmul_wasm.py` for their arguments, and run both once, from the repository root, to record today's baseline. They are existing Python scripts and need built inputs. The native audit reads `target/release/libuor_r4_integer.rlib`, `target/release/uor-r4-integer` and `target/release/uor-chat` unless you pass a path, so first run `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm` (a CPU build, allowed). The WebAssembly audit needs the module that `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm` writes (by default under `crates/uor-r4-integer/pkg/`), then `python3 scripts/audit_zero_matmul_wasm.py <that .wasm> --serving --call-graph --tap` (`docs/labs/wasm-serving-runtime-2026-10-18/README.md:32-33`).
2. **First deliverable: a design note on the issue.** Say how the output of `m1-d11-exporter-discrete` maps into UORLUT01 and how `uor-chat` loads it. Say how the serve path stays at 0 float, 0 softmax or squashing, 0 matmul and 0 mul/div, and how a fresh-process restore is bit-identical. Confirm with `m1-d11-exporter-discrete` that both items target the same container.
3. **Write a finite control as a failing test.** In `crates/uor-r4-integer/tests/`, export a toy model (see `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`), start `uor-chat` on it in two fresh processes, and require identical token output. Run it with `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm`. Start the CLI with `~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin uor-chat -- --stack <ARTIFACT.lut> --tokenizer <TOKENIZER.json>`.
4. **Draft the pre-registration sheet.** Name the chosen artifact, the frozen held-out byte panel with its SHA-256, and the scorer from `m4-lawful-likelihood-bpb`. Register the 0.90 bar, the export-gap method (float candidate against integer artifact on the same evaluator, `CHARTER.md` section 3.7), the tokens-per-second report, the stop rule and the cost. Ask a non-author for a TEST FITNESS: FIT review.
5. **Wait for the owner.** M4 is held, and one typed owner release covers one sheet; Casey types it. After release, run the export and evaluation on a GPU (outside contributors attach hardware specs), then rerun both audits on the final build.
6. **Open a PR** that says `Closes #<this issue>` (or `References` if partial), and record the result as one row in `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:17`](../../../ROADMAP.md)
- [`STATUS.md:19`](../../../STATUS.md)
- `docs/model/serving-and-export.md:51-52` (audit counts)
- `docs/labs/wasm-serving-runtime-2026-10-18/README.md:32-33` (WebAssembly build and audit commands)
- `CHARTER.md` section 3.7 (float only offline, with an export gap)
- issue#2032

