---
id: m1-export-gap-and-toks
title: Measure export gap and tokens per second on a trained model
milestone: M1
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m1-d11-exporter-discrete]
---

## Why it matters

M1 bar (d) is defined through the export, and M5 builds on the same measurement. This is Stage 4 of the plan: the first time a model trained under today's rules is read through the integer artifact and timed. See [the lookup-serving page](../../mechanisms/lut-serving.md).

## What is missing

Tokens per second for a model trained under today's rules has never been measured, and the export gap is unmeasured. Stage 4 needs both prompt-reading and reply-writing rates on a laptop, and integer top-1 scored through the D11 export on the DEV panel (14,287 rows in the K16 receipts, `docs/integration/NOW.md` line 78). The export-gap rule (D39 item 5) is: a float candidate is disqualified when its export gap exceeds the two-seed band on the same evaluator, fixed per sheet at FIT time. The numeric band is not yet registered.

Two inputs do not exist yet. The exporter is the item `m1-d11-exporter-discrete`. A saved trained discrete model is also needed, from the A8 rerun (`m1-a8-full-dose-rerun`), BVG-K Stage T (`m1-bvg-k-stage-t-training`) or the trunk (`m1-trunk-assembly-and-training`). This item cannot run until both exist.

## Done when

Stage 4 bar: 'the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply.' Decode and context rates are reported separately.

## How to start

1. Read `docs/mechanisms/lut-serving.md`, `docs/integration/NOW.md` lines 96–97 (the export-gap rule), `docs/labs/wasm-serving-runtime-2026-10-18/README.md`, `docs/labs/k16-outcome-call-2026-10-19/README.md` (the DEV panel and its two-seed receipts) and the runtime in `crates/uor-r4-integer`. Check the progress of `m1-d11-exporter-discrete` and of a saved trained model; do not start a run without both.
2. First deliverable, posted on the issue: a draft pre-registration sheet, one page, modelled on `docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`. It must define:
   - the panel: the DEV panel file and its SHA-256, taken from the K16 outcome record and frozen before the run;
   - the oracle: the training-side evaluator and the integer scorer through the D11 file on the same rows, so the gap is measured on the same evaluator;
   - the gap statistic (for example the difference in top-1 count on the same rows) and the noise band, taken from two seeds on that evaluator and fixed in the sheet at FIT time;
   - the timing protocol: laptop model recorded, a warmup, then N timed runs with median, nearest-rank p95 and all raw samples, for the prompt-reading rate and the reply-writing rate separately;
   - which side runs where. Evaluation runs on the GPU (§3.5), while serving the artifact on a laptop is the product (§3.6). Say which parts run on the pod and which on the laptop, and ask the FIT reviewer to agree.
3. Request a non-author TEST FITNESS: FIT review. Do not run before it passes.
4. After FIT: export the saved model with the `m1-d11-exporter-discrete` tool. Run everything below from the repository root. Build the native runtime with `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm` (the `uor-chat`, `verify-integer` and `uor-r4-stack` binaries all need `serving-wasm`, and without it only the library builds). Build the WebAssembly module as in the [WASM runtime record](../../labs/wasm-serving-runtime-2026-10-18/README.md): `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`. Then run the native audit, `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`, and the WebAssembly audit, `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap` (both are existing scripts; add no new Python). Both must report 0 float and 0 mul/div. Score the DEV panel through the artifact and time both rates on the laptop.
5. Keep the raw outputs and hashes beside the sheet. Report the gap against the band, and tokens per second for both rates. A gap outside the band is a result on that configuration, not a verdict on the mechanism.
6. Open a PR that says `Closes #<this issue>` (or `References` if partial). Add one row to `docs/research/notebook/m4-serving.md` with the number, the bar fixed in the sheet, the export gap and the verdict word.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:96-97`](../../../docs/integration/NOW.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:136`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:165`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:23`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:94`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:55`](../../../docs/model/architecture.md)
- [`docs/start/where-we-are.md:58`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:50`](../../../docs/start/where-were-going.md)
