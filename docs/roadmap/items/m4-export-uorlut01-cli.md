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

M4 (issue #2032) is the step that turns the selected M1-to-M3 work into one accepted served artifact, which M5 to M8 all depend on. The integer serving engine and `uor-chat` exist, and the discrete M1 pair already proves table-native UORLUT01 export and D11 serving at its declared scope. That does not qualify M4's selected model, quality, CLI, session and full audit path. Background: [serving and export](../../model/serving-and-export.md).

## What is missing

**November 17, 2026.** The M1 D11 artifacts and CLI provide an earlier serving case. M4 still requires a selected M3-qualified model, the held-out 0.90 BPB bar, static audits, exact restore and throughput. On the D85 default, native raw-word parity and lossless state serialization supersede the historical float export-gap route. [PR #4000](https://github.com/UOR-Foundation/uor-r4/pull/4000) · [PR #4046](https://github.com/UOR-Foundation/uor-r4/pull/4046) · [Issue #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032)

Package the selected M3-qualified native state in a single UORLUT01 file, serve it through `uor-chat`, and score it on the held-out byte panel. The discrete M1 pair already demonstrated exact export agreement on 14,287 DEV rows. M4 requires its own accepted artifact and score.

The explicit `System: Memory: {value}.` serving turn now exists. On both public
FBS seeds, adding it changed the model's reply ids in every matched arm, but
neither factual answer contained the supplied value and the positive and
`Memory: none.` controls collapsed to the same malformed reply. The serving
format is connected; automatic statement extraction and trained recall quality
remain open. See [the report](../../loop/tasks/m4-export-uorlut01-cli/deepseek-1/REPORT.md).

`uor-chat --stack` now defaults to protocol 2, the encoding used by the pinned
training corpus. Protocol 1 remains selectable for old artifacts. This repairs
the default-tokenization instrument found by claude-259; it does not turn the
current public artifacts into a useful chat model. See
[the report](../../loop/tasks/m4-export-uorlut01-cli/deepseek-3/REPORT.md).

`fbs-toks --copy-codes` now refuses non-one-token Hamming keys before parsing
artifact sections. The legacy checker's reconstruction is one-token only; a
pair-aware checker remains separate work after the ordered key lands. See
[the report](../../loop/tasks/m4-export-uorlut01-cli/deepseek-4/REPORT.md).

The exporter now applies the 256-wide per-attention-head limit only to
attention stacks. All-recurrence FBS trunks at widths 512 and 1,024 export and
serve in exact synthetic controls, matching the native writer's existing width
limit and the Hamming read's separate head geometry. Claude's real width-512
checkpoints still need re-export after merge. See
[the wide-export report](../../loop/tasks/m4-export-uorlut01-cli/deepseek-2/REPORT.md).

The native and WASM serving audits pass at their declared scopes. The final M4 build must retain 0 float, 0 softmax or squashing and 0 dense matmul; integer multiply and divide are allowed and audited under D80/D81.

The M1 pair settles that a discrete model can use table-native UORLUT01. What remains open is the selected M4 model's accepted CLI/session path and the held-out byte panel behind the unchanged 0.90 bits-per-byte bar.

## Done when

M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. In addition:

- The artifact losslessly serializes the native state updated in training. On identical prompts and starting state, the shared typed operators replay raw route choices, carry, read choices, memory addresses and emitted token IDs bit for bit (D85).
- The static audits are green.
- A fresh-process restore is bit-identical.

## How to start

1. **Read the contract and the code.** Open `docs/model/serving-and-export.md`, `docs/mechanisms/lut-serving.md`, `crates/uor-r4-lut/src/format.rs` (the UORLUT01 layout), `crates/uor-r4-integer/src/bin/uor-chat.rs` and `crates/uor-r4-integer/README.md`. `uor-chat` and `mk-synthetic-stack` are built only with `--features serving-wasm` (the README's `cargo build` line omits it). Read the headers of `scripts/audit_zero_matmul_serving.py` and `scripts/audit_zero_matmul_wasm.py` for their arguments, and run both once, from the repository root, to record today's baseline. They are existing Python scripts and need built inputs. The native audit reads `target/release/libuor_r4_integer.rlib`, `target/release/uor-r4-integer` and `target/release/uor-chat` unless you pass a path, so first run `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm` (a CPU build, allowed). The WebAssembly audit needs the module that `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm` writes (by default under `crates/uor-r4-integer/pkg/`), then `python3 scripts/audit_zero_matmul_wasm.py <that .wasm> --serving --call-graph --tap` (`docs/labs/wasm-serving-runtime-2026-10-18/README.md:32-33`).
2. **First deliverable: a design note on the issue.** Start from the M1 pair's demonstrated table-native UORLUT01 mapping and say how the selected M4 model loads through `uor-chat`. Keep the serve path at 0 float, 0 softmax or squashing and 0 dense matmul, with integer multiply/divide allowed and audited under D80/D81. Require fresh-process restore to be bit-identical.
3. **Write a finite control as a failing test.** In `crates/uor-r4-integer/tests/`, export a toy model (see `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`), start `uor-chat` on it in two fresh processes, and require identical token output. Run it with `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm`. Start the CLI with `~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin uor-chat -- --stack <ARTIFACT.lut> --tokenizer <TOKENIZER.json>`.
4. **Draft the pre-registration sheet.** Pin the M3-qualified native state, the serialized artifact, the tokenizer, the held-out byte panel by SHA-256, and the scorer from `m4-lawful-likelihood-bpb`. Register the identical starting state and prompts for the raw-word train/serve replay, the fields compared at each step, and the fresh-process restore check. Use the existing ≤0.90 BPB bar and record decode and context throughput, cost and stop rules. Obtain a non-author FIT.
5. **Run the D67 pre-run review.** After the registered FIT and the lab's own pre-run review pass, run the export and evaluation on a GPU inside the caps (outside contributors attach hardware specs), then rerun both audits on the final build. No blanket owner release applies.
6. **Open a PR** that says `Closes #<this issue>` (or `References` if partial), and record the result as one row in `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md`](../../../ROADMAP.md) and [`PLAN.md`](../../../PLAN.md)
- [Current architecture](../../model/architecture.md) and [serving/export](../../model/serving-and-export.md) for the M1 pair's demonstrated table-native export and its exact scope
- `docs/labs/wasm-serving-runtime-2026-10-18/README.md:32-33` (WebAssembly build and audit commands)
- [Charter §§3.7 and 5.1](../../../CHARTER.md) for D85 native replay and the D67 pre-run review. The historical D37 recipe compared a float candidate and its exported integer artifact on the same evaluator. Its measured gaps are preserved in the [M4 serving notebook](../../research/notebook/m4-serving.md).
- issue#2032

