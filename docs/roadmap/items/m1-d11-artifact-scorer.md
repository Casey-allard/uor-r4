---
id: m1-d11-artifact-scorer
title: Build the D11 exporter CLI, GPU panel scorer and tok/s harness for step 6
milestone: M1
area: evaluation
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-d11-exporter-discrete]
notebook: docs/research/notebook/m4-serving.md
scope: [docs/labs/m1-d11-artifact-scorer/**, docs/harness/prep/m1-d11-artifact-scorer*, docs/loop/tasks/m1-d11-artifact-scorer/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

The M1 bar is read "through the D11 export, on each of two seeds … on the same DEV panel, with tokens/s reported" (CHARTER §4.8). PLAN.md step 6 (`m1-export-gap-and-toks`, #2839) measures exactly that, and the M1 close (step 8) rests on it.

CHARTER §3.6 makes "panel scoring, qualification and milestone evaluation of an exported artifact" GPU research compute. Only product serving, and timing it, runs on the laptop. Step 6 is held until step 4 (BVG-K Stage T, #2836) saves its models. When they land, its run needs three tools that do not exist on `main`. This item builds them, so that step 6 starts the moment step 4 is done.

## What is missing

1. **An exporter CLI.**
   - `export_d11(&Snapshot) -> Result<Vec<u8>>` (`crates/uor-r4-discrete/src/d11_export.rs:227`) is a library function with no caller.
   - The step-6 sheet (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md:32, :81`) names a `discrete-stack export-d11` subcommand and `export_d11_to_path`. Neither is on `main`, and the "#2998" it cites is a different PR.
2. **A GPU panel scorer for an exported artifact.**
   - `crates/uor-r4-integer` has no `.cu` file and no `build.rs`.
   - Its clean-room evaluator, `IntegerDiscreteModel::evaluate_prefix` (`crates/uor-r4-integer/src/discrete_d11.rs:1165`), scores one prefix on the host.
   - The step-6 sheet plans to compare it row by row against `discrete-stack eval-corpus` on 14,287 rows, on the leased pod's host CPU and not its GPU. Panel scoring of an exported artifact is GPU research compute (§3.5, §3.6).
3. **A serving tok/s harness** for `uor-r4.lut-discrete/1` files that reports prompt-reading (context prefill) and reply-writing (decode) separately. The protocol is the step-6 sheet's: 1 warmup and 20 timed runs, a 192-token prefill and a 64-token decode, median, nearest-rank p95 and all raw samples.

## Done when

All three are built as a detached instrument in `docs/labs/m1-d11-artifact-scorer/instrument/`: its own `[workspace]`, with path dependencies on `uor-r4-discrete` and `uor-r4-integer` and no edit to either. That keeps `forbid(unsafe_code)` in the serving crate (§3.13) and takes no crate lock.
- **`export`** turns a `DriverCheckpoint` JSON into a `UORLUT01` file. The file is byte-identical to `export_d11` on the same snapshot, and `IntegerDiscreteModel::from_bytes` accepts it.
- **`score`** runs a CUDA, clean-room port of `IntegerDiscreteModel::evaluate_prefix` over the registered DEV panel. The rows come from `uor_r4_discrete`'s own corpus windows, so they are the same rows `eval-corpus` scores. It writes per-row prediction, top-8 and admission, and integer top-1, top-8 and admitted totals.
  - The kernel is built ahead of time by nvcc, with no NVRTC, no float and no squashing (§3.2–§3.4).
  - A finite test shows the CUDA results bit-identical to `evaluate_prefix` on fixtures, and fails when the port is set aside.
  - A GPU qualification on one RTX 5090 shows parity on a real exported checkpoint. The A8 FrozenZero `continuation-05248.json`, inside `results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz` (store, with its `SHA256SUMS`), qualifies the instrument only, and is never step 6's reported measurement.
- **`toks`** times `IntegerDiscreteModel` serving on the laptop by the step-6 protocol, and records the hardware.
- **A note on #2839** maps each tool line in the step-6 sheet to the instrument command that replaces it. Those lines are `:32` (the exporter "merged in #2998"), `:81` (`discrete-stack export-d11`) and `:82` (the `eval-corpus`/`IntegerDiscreteModel` comparison). This item does not edit that sheet.

## How to start

1. Read the step-6 sheet (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`), its task folders (`docs/loop/tasks/m1-export-gap-and-toks/`) and #2839. This item serves that sheet and does not change it.
2. Read `crates/uor-r4-integer/src/discrete_d11.rs`: `evaluate_prefix` at 1165 and the read modes it dispatches. Then read `crates/uor-r4-discrete/src/d11_export.rs` and the round-trip tests from `m1-d11-exporter-discrete`.
3. Read the detached-instrument precedents, `docs/labs/m1-w2-code-utilization/instrument/` and `docs/labs/r-prime-zeta-vs-nulls/instrument/`. Neither builds CUDA: they set the precedent for detaching only. The ahead-of-time nvcc PTX build to follow is `crates/uor-r4-discrete/build.rs`.
4. Type-check the CUDA on the Mac with a stub nvcc (the run file, part 4 A.5). Run the finite parity and the GPU qualification on a leased pod under a D67 pre-run review.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5, §3.6): panel scoring of an exported artifact runs on a GPU. Serving it and timing it run on the client device.
- **Pre-registration first** (§4.5): this item builds and qualifies tools, which needs no sheet. A measurement that a milestone claim rests on runs under step 6's FIT'd sheet.
- **Rust first** (§3.11): new code is Rust (and CUDA built by nvcc). No new Python.
- **Records** (§4.1, §5.6): each result is one row in `docs/research/notebook/m4-serving.md`.

## Sources

- [`CHARTER.md` §3.6, §4.8](../../../CHARTER.md)
- [`PLAN.md` step 6](../../../PLAN.md)
- [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md)
- [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs)
- [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs)
- [`docs/loop/tasks/infra-loop-harness/claude-115/DECISION.md`](../../loop/tasks/infra-loop-harness/claude-115/DECISION.md)
