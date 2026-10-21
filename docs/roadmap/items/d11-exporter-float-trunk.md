---
id: d11-exporter-float-trunk
title: Write the D11 exporter for a D37 float trunk
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [float-training-module, m1-d11-exporter-discrete, owner-decision-float-path-scope]
scope: [crates/uor-r4-training/**, crates/uor-r4-integer/**, docs/mechanisms/lut-serving.md]
---

## Why it matters

Without it no float-trained trunk can be served, so Stages 3–5 of the float path stop at training. D11 is the serving contract: a whole-number file run with adds, shifts and table look-ups. The exporter is the step between a trained trunk and that file, and it is where the export gap is measured. See [serving and export](../../model/serving-and-export.md). The float path itself is a hypothesis test, not reuse.

## What is missing

No exporter exists for a float trunk.

- Export and quantization-aware training refuse the recurrence mode.
- The Hamilton gate "still needs the icosian snap".
- `crates/uor-r4-training/src/stack_export.rs` is compiled out with `#[cfg(any())]` (`crates/uor-r4-training/src/lib.rs:197-198`). That crate is listed as held and carries the historical dense and sigmoid stack.
- Integer schemas /1–/2 serve softmax, so they cannot be reused.
- The numeric export-gap band is not yet registered. [CHARTER](../../../CHARTER.md) §3.7 only says a gap above the two-seed noise band disqualifies a float candidate.

## Done when

A float-trunk snapshot with recurrence and the Hamilton gate exports to D11 with the audits green and the export gap measured inside the band. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

## How to start

1. Confirm the three dependencies on the issue. `owner-decision-float-path-scope` is the owner's call, and no float trunk exists until `float-training-module` lands and a trunk is trained (Stage 3). Do not start code before the owner decision is recorded. Then read `docs/mechanisms/lut-serving.md`, `docs/mechanisms/recurrence-gates.md`, `docs/mechanisms/hamilton-gate.md`, and the Stage 4 text at `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:134-137`.
2. Post a design note on the issue first. It must give:
   - the snapshot format the float module hands to the exporter;
   - how dyadic recurrence with quaternion transport becomes adds, shifts and table look-ups;
   - the icosian snap for the Hamilton gate;
   - why neither the exporter nor the audits use softmax, squashing or dense matrix multiplication;
   - finite CPU controls: a tiny hand-built trunk whose float forward and exported integer forward can be compared on every input of a small vocabulary.
3. Do not edit `crates/uor-r4-training/src/stack_export.rs` or bring it back into that crate. Read it for the quantization steps only. Put new code in the float module's crate or beside the UORLUT01 reader in `crates/uor-r4-integer/src/stack/format.rs`, and say where in the note. Write a failing test first: the control trunk exports and the integer forward matches the recorded float forward within the stated tolerance.
4. Draft a one-page pre-registration for the export-gap measurement: the evaluator, two seeds, the band as the measured two-seed noise band, and the audits that must be green. `scripts/audit_zero_matmul_serving.py` is the existing zero-multiply audit: run it as `python3 scripts/audit_zero_matmul_serving.py <path to the built rlib or binary> --stack`, with `objdump` or `otool` installed. `--stack` audits `uor_r4_integer::stack`. Add no new Python. A non-author FIT review comes first. Builds and finite host tests need no pre-registration; a GPU training or evaluation run, and any run a milestone claim rests on, does. The export gap is a panel-scored evaluation of the exported artifact, so it is GPU research compute (CHARTER §3.5 and §3.6). Only the finite controls on the hand-built trunk and the tokens-per-second measurement on a client device run on the laptop. The synthesis Stage 4 text says the gap is measured on the laptop; CHARTER §3.6 governs. Training the trunk stays HELD until Casey types the owner release himself: a FIT review releases nothing, and no lab types it.
5. Implement recurrence export, the icosian snap and the audits. Run `~/.cargo/bin/cargo test -p <crate>` for the crate you touched and `uor-harness check`. If the crate is `uor-r4-integer`, add `--features serving-wasm`: the `stack` module (including `stack/format.rs`) and the `uor-r4-stack` and `mk-synthetic-stack` bins compile only with that feature. `mk-synthetic-stack [SEED] [OUT_PATH]` writes a small synthetic UORLUT01 stack that can seed the control fixture.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:66`](../../../docs/model/architecture.md)
- [`docs/start/transformer-vs-geometry.md:61`](../../../docs/start/transformer-vs-geometry.md)
