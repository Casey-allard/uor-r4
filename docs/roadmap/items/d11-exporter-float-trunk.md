---
id: d11-exporter-float-trunk
title: Write the D11 exporter for a D37 float trunk
milestone: M4
area: serving
status: ready
open_to: lab
needs_gpu: true
depends_on: [float-training-module, m1-d11-exporter-discrete, owner-decision-float-path-scope]
scope: [crates/uor-r4-training/**, crates/uor-r4-integer/**, crates/uor-r4-float-train/**, docs/harness/prep/d11-exporter-float-trunk*, docs/labs/d11-exporter-float-trunk/**, docs/loop/tasks/d11-exporter-float-trunk/**, docs/research/notebook/m4-serving.md]
---

## Why it matters

Without it no float-trained trunk can be served, so Stages 3–5 of the float path stop at training. D11 is the serving contract: a whole-number file run with adds, shifts and table look-ups. The exporter is the step between a trained trunk and that file, and it is where the export gap is measured. See [serving and export](../../model/serving-and-export.md). The float path itself is a hypothesis test, not reuse.

## What is missing

**State on 26 Oct 2026 (claude-120, #3253; D68 owner log on #2820).** antigravity-75 (#3221) merged a float-trunk UORLUT01 exporter, `export_float_trunk_uorlut01`, and its finite controls. It then held this item until the Stage 3 float chain (#2886 → #2890 → #2824 → #2891) produced two-seed checkpoints. That chain is done, and this item is released. Three things are left, and none waits on another lab or the owner.

- **No Stage 3 checkpoint was saved.**
  - The four records' store folders hold receipts, logs and binaries only.
  - `StackModel::save` and `StackModel::load` exist (`crates/uor-r4-float-train/src/geometric_stack_native.rs:1293`, `:1337`), but no sheet runner calls them; only a writer test does.
  - What is missing is a save call in a runner, and a converter from a saved model to the exporter's input.
- **The merged exporter covers a different trunk from the one Stage 3 trained.**
  - `FloatTrunkSnapshot`, `FloatTrunkLayer` and `FloatTrunkRecurrence` (`crates/uor-r4-integer/src/stack/format.rs:690-772`) hold dense mixer, MLP, read and head matrices. Nothing in `stack/format.rs` or `stack/session.rs` mentions SpiralCore/FBS or the vector writer.
  - The Stage 3 sheets trained a different trunk (`build_hamilton_model`, `crates/uor-r4-float-train/src/hamilton_sheet.rs:1640-1658`): six recurrence layers, the Hamilton gate, geometric recurrence gates, and SpiralCore/FBS projections. Each dense map in `QUATERNION_MAPS` (`geometric_stack_native.rs:402`) becomes a `[outputs, stages, 8]` routing tensor. The head is the native vector writer, keeping 8 cells.
  - So the work left is an FBS `(atom, action)` projection path and a vector-writer head, in the exporter and in the integer runtime, with finite CPU controls.
  - Prior art to reuse: the recurrence-fit lab's integer exporter (`docs/labs/recurrence-gates-six-recurrence-fit/README.md:94`) and `pure_i32_q12_writer_oracle` (`hamilton_sheet.rs:1245`).
- **No GPU evaluator exists for a UORLUT01 stack container.** CHARTER §3.6 keeps panel scoring of an exported artifact on the GPU. The work needs either a GPU evaluator of the container, or a bit-exact parity proof between the sheets' GPU export mode and the container.

**Scope note (claude-121, #3254; D68 owner log on #2820).** `docs/mechanisms/lut-serving.md` is not in this item's scope. It is also in `infra-loop-harness`'s scope, which Codex's idle draft #3169 holds, and that overlap refused this item's claim. This item records its results in its lab record (`docs/labs/d11-exporter-float-trunk/**`) and in `docs/research/notebook/m4-serving.md`. The mechanism-page row and status for this result are written by an infra-loop-harness item-maintenance unit once the umbrella is unclaimed. Every exporter REPORT lists this as open.

**What the earlier text below listed:**

- Export and quantization-aware training refuse the recurrence mode.
- The Hamilton gate "still needs the icosian snap".
- `crates/uor-r4-training/src/stack_export.rs` is compiled out with `#[cfg(any())]` (`crates/uor-r4-training/src/lib.rs:197-198`). That crate is listed as held and carries the historical dense and sigmoid stack.
- Integer schemas /1–/2 serve softmax, so they cannot be reused.
- The numeric export-gap band is not yet registered. [CHARTER](../../../CHARTER.md) §3.7 only says a gap above the two-seed noise band disqualifies a float candidate.

## Done when

A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

## How to start

1. Confirm the three dependencies on the issue. `owner-decision-float-path-scope` is the owner's call, and no float trunk exists until `float-training-module` lands and a trunk is trained (Stage 3). Do not start code before the owner decision is recorded. Then read `docs/mechanisms/lut-serving.md`, `docs/mechanisms/recurrence-gates.md`, `docs/mechanisms/hamilton-gate.md`, and the Stage 4 text at `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:134-137`.
2. Post a design note on the issue first. It must give:
   - the snapshot format the float module hands to the exporter;
   - how dyadic recurrence with quaternion transport becomes adds, shifts and table look-ups;
   - the icosian snap for the Hamilton gate;
   - why neither the exporter nor the audits use softmax, squashing or dense matrix multiplication;
   - finite CPU controls: a tiny hand-built trunk whose float forward and exported integer forward can be compared on every input of a small vocabulary.
3. Do not edit `crates/uor-r4-training/src/stack_export.rs` or bring it back into that crate. Read it for the quantization steps only. Put new code in the float module's crate or beside the UORLUT01 reader in `crates/uor-r4-integer/src/stack/format.rs`, and say where in the note. Write a failing test first: the control trunk exports and the integer forward matches the recorded float forward within the stated tolerance.
4. Draft a one-page pre-registration for the export-gap measurement: the evaluator, two seeds, the band as the measured two-seed noise band, and the audits that must be green. `scripts/audit_zero_matmul_serving.py` is the existing zero-multiply audit: run it as `python3 scripts/audit_zero_matmul_serving.py <path to the built rlib or binary> --stack`, with `objdump` or `otool` installed. `--stack` audits `uor_r4_integer::stack`. Add no new Python. A non-author FIT review comes first. Builds and finite host tests need no pre-registration; a GPU training or evaluation run, and any run a milestone claim rests on, does. The export gap is a panel-scored evaluation of the exported artifact, so it is GPU research compute (CHARTER §3.5 and §3.6). Only the finite controls on the hand-built trunk and the tokens-per-second measurement on a client device run on the laptop. The synthesis Stage 4 text says the gap is measured on the laptop; CHARTER §3.6 governs.

   **Training the trunk, and the export-gap run, go ahead on the lab's own D67 pre-run review** (CHARTER §5.1), under a new one-page sheet with a fresh non-author FIT. That replaces the typed owner release; D56 and D67 retired it.

   **The Stage 4 sheet FIT'd for antigravity-75 (#2820 comment `6398915445`) does not carry over:**
   - its float arm assumes a flock rank read, which the Stage 3 trunk has none of, and which the float module refuses;
   - its planted fault negates `rec_out` nibbles, which an FBS trunk does not have;
   - its 398-row band must be measured again on the new runs.

   The new sheet must name the arm it exports and say what each outcome decides. Every Stage 3 configuration missed its fit bars, and their export gaps differ by arm:
   - every Hamilton-sheet arm passed its own export gap, `Hamilton_IcosianHard` with the smallest, 11–22 top-1 rows (`docs/labs/hamilton-gate-offline-float-sheet/README.md`);
   - the FBS one-atom arm failed its top-1 export bar (`docs/labs/spiralcore-fbs-offline-float-sheet/README.md`).
5. Implement recurrence export, the icosian snap and the audits. Run `~/.cargo/bin/cargo test -p <crate>` for the crate you touched and `uor-harness check`. If the crate is `uor-r4-integer`, add `--features serving-wasm`: the `stack` module (including `stack/format.rs`) and the `uor-r4-stack` and `mk-synthetic-stack` bins compile only with that feature. `mk-synthetic-stack [SEED] [OUT_PATH]` writes a small synthetic UORLUT01 stack that can seed the control fixture.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **No dense materialization** (§3.3, §3.6): never turn FBS routes into dense matrices so that the merged dense path can take them. A table-read matrix-vector product is a dense multiply in disguise (`docs/mechanisms/lut-serving.md:56`). The merged dense path stays a finite control only. The writer reads a bounded candidate set, and the export must keep it bounded: no full-vocabulary head scan.
- **One-way dependency:** the converter lives in `crates/uor-r4-float-train` and depends on `crates/uor-r4-integer`, never the reverse (§3.4, D11 R1).
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:135`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/model/architecture.md:66`](../../../docs/model/architecture.md)
- [`docs/start/transformer-vs-geometry.md:61`](../../../docs/start/transformer-vs-geometry.md)
