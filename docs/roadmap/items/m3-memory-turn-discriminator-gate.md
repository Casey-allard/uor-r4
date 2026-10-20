---
id: m3-memory-turn-discriminator-gate
title: Add a memory-turn discriminator or margin gate
milestone: M3
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: [m3-nonrelation-prose-panel]
---

## Why it matters

Prevents silent corruption of stored facts. M3 cannot be qualified while small talk can compile to a write that overwrites a stored fact. This gate sits between the compiler's read of a turn and the exact store, as described in [the relation read](../../mechanisms/relation-read.md).

## What is missing

The required fix from #1718 is a discriminator or margin gate so that non-memory prose cannot compile to ASSERT or CORRECT, measured before qualifying M3. world=v2 uses compiler-save-op-v25-rawtable and op-model-v25. The gate does not exist yet. The panel that measures it is the [non-relation prose panel](m3-nonrelation-prose-panel.md), and its pass bar is not yet registered.

## Done when

M3 criterion 3: "Non-memory conversational prose does not silently overwrite stored facts (#1718)", measured on the pre-registered panel.

## How to start

1. Read `crates/uor-r4-training/src/relation_compiler.rs`: `SavedCompiler::action` and `table_action` (around lines 2205–2320), `classify_row`, and the `ActRule` and `OpPolicy` types. Find how the table and span heads give a relation and an act, and where a margin between the top class and the runner-up could be read. The module already holds a `Softmax` fit type (line 353) that the gate must not reuse. Do not add softmax, sigmoid or a normalized exponential to get a "probability" (CHARTER §3.2); a margin must come from the geometry the heads already use, such as a rank or distance gap.
2. Wait for or help finish the panel in [m3-nonrelation-prose-panel](m3-nonrelation-prose-panel.md). Its baseline false-assert and false-overwrite rates are the number this gate must beat.
3. Post a design note on the issue. It must state: the geometric construction of the discriminator or margin gate; how it avoids softmax and matmul; where in `action` it acts (it must return `Unresolved`, never change a write into a read); the finite CPU controls (a hand-built compiler where a statement clears the margin and a small-talk turn does not, and a check that real statements still write); and the failure it must not cause, which is refusing true statements.
4. Add the gate and its CPU tests in `relation_compiler.rs`. Run `~/.cargo/bin/cargo test -p uor-r4-training relation_compiler` and `uor-harness check`.
5. Draft a one-page pre-registration sheet: panel hash, grader, baseline rates from the panel item, the gate's setting, the registered bar, and the stop rules. Ask for a non-author TEST FITNESS: FIT review. This is a GPU evaluation on the world=v2 artifacts (`compiler-save-op-v25-rawtable`, `op-model-v25`) and needs that FIT, an owner release typed by Casey, and a lease through `scripts/pod/uor-pod` (CHARTER §4.5, §5.1). Outside contributors may use their own GPU and attach hardware specs.
6. After the release, run the sheet, keep the artifacts, and report both rates before and after the gate.
7. Open a PR that says `Closes #<this issue>` (or `References` if partial). Record the result as one row in `docs/research/notebook/m3-durable-memory.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2031 (folds #1718)
- `crates/uor-r4-training/src/relation_compiler.rs` (`action` and `table_action`, lines 2205–2320)
