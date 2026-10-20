---
id: m5-laptop-full-path-measurement
title: Measure M1-class laptop cost on the accepted served model
milestone: M5
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: [m4-export-uorlut01-cli]
---

## Why it matters

An integer path is not an efficiency result until it is measured (CHARTER §3.9: "Efficiency is measured"). M5 ([#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033)) turns the serving claim into numbers on an ordinary laptop. It runs on the model M4 exports, so it can only close after M4 delivers the artifact. See [serving and export](../../model/serving-and-export.md) and [lookup-table serving](../../mechanisms/lut-serving.md).

## What is missing

m5-cost is 0/1 qualified. Only synthetic-stack measurements exist: macmon M1 runs (4.13 vs 0.96 mJ/tok) and a provisional Linux RAPL receipt (17,190 tok/s, 3.172 mJ/tok gross, 1.530 mJ/tok net, on a 15,968 weights-per-token synthetic fixture, not a trained model). The maintainer annotation in the protocol (section 4) records two defects in that receipt: `work_rate_w` (40.5 W) disagrees with gross energy times tok/s (about 54.5 W), and the confidence interval used a z-value for three samples where a t-based interval gives about ±0.080 mJ instead of ±0.037 mJ. The owner's M1 run on the accepted M4 artifact is the authoritative record.

## Done when

M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". Latency (ms/tok), peak RSS, energy (mJ/tok) and weights_per_token over total parameters are measured and reproducible across ingestion, routing, memory lookup and emission, with no hidden host float fallback or uncounted external daemon.

## How to start

1. Read `archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md` (protocol, scope and the 2026-10-18 maintainer annotation), `archive/ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md` and the unedited receipt `archive/ops/measurement_report_rnd.json`. The files were archived byte-identical, so cite the `archive/ops/` paths. Read `scripts/energy_per_token.py`, the existing macmon harness; do not add new Python.
2. Read the Rust side that will be measured: `crates/uor-r4-integer/src/bin/uor-r4-stack.rs`, `uor-chat.rs` and `mk-synthetic-stack.rs`. Build with `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm --bins` (every binary in that crate requires the `serving-wasm` feature). Note where ingestion, routing, memory lookup and emission happen, because the bar asks for the cost across all four.
3. Post a design note on this issue: the per-phase timing and counting points, how peak RSS is read, how energy is read (macmon on M1; RAPL only as a second instrument), how `weights_per_token / total parameters` is computed, and how the run proves there is no host float fallback and no uncounted external daemon (for example, a process list and a link-time or disassembly check recorded with the receipt).
4. Draft a protocol v3 sheet (a new file under `docs/`, not an edit of the archived v2): it fixes the artifact hash, the machine, warm-up, run count, the t-based interval for small n, and corrects the two defects above. The numeric M5 bar is not yet registered beyond "measured", so the sheet must say so and ask the PM for a decision card if a threshold is wanted. Labs stage the exact commands; the owner runs the authoritative M1 measurement.
5. Rehearse the whole procedure on the synthetic fixture (`mk-synthetic-stack`) and label every number provisional. Keep the output as one receipt file with its sha256. Measuring serving on a client device is allowed (CHARTER §3.6); training and panel scoring are not part of this item.
6. When M4's artifact exists (`m4-export-uorlut01-cli`), stage the commands for the owner's M1 run on it and attach the receipt.
7. Open a PR that says `Closes #<this issue>` after the owner's M1 receipt lands, or `References #<this issue>` for the protocol and rehearsal. Record the result as one row in `docs/research/notebook/m4-serving.md` (no M5 notebook page exists yet).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:18`](../../../ROADMAP.md)
- issue#2033
- [`archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`](../../../archive/ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md)
