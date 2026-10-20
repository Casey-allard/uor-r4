---
id: m1-normalization-mechanism
title: Design a lawful normalization mechanism
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

A transformer rescales its signals between layers so they stay in a workable range. This model has nothing in that slot, and the Stage 3 trunk (feed-forward layer, recurrence and the Flock and Relation reads) needs one. This is a design task that needs no GPU and is open to anyone. See [the transformer-to-geometry map](../../start/transformer-vs-geometry.md) (Gap 2).

## What is missing

No normalization is designed, and the current model has none. Snapping each block to one of its 16 fixed E8 roots, or the unit-quaternion Hamilton gate, might play this role, but no record says so or tests it; that is a **hypothesis**. The Hamilton gate's own claim, that a norm-preserving gate can suppress a signal, is also a hypothesis (a dossier reading only). Any mechanism must meet CHARTER §3.1-§3.3: geometry only, no softmax or squashing, no matmul.

## Done when

A normalization mechanism with finite controls obeying CHARTER §3, plus a pre-registered test of its effect in the trunk. The size of effect that would count is not yet registered; step 4 registers it.

## How to start

1. Read the gap and the lawful parts. Open `docs/mechanisms/README.md` (the Normalization row, near line 142), `docs/mechanisms/hamilton-gate.md`, `docs/start/transformer-vs-geometry.md` (Gap 2), `docs/model/architecture.md` (the token table of 4 blocks of 16 roots; the "missing parts" table), `docs/geometry/e8-h4-and-the-600-cell.md`, `docs/geometry/quaternions-and-rotation.md` and `CHARTER.md` §3.
2. Post a design note on the issue as your first deliverable. It must name the quantity to keep stable (for example block norms of the integer vector), the geometric construction, and why it is lawful: integer arithmetic only, with no exponential, softmax, sigmoid, tanh or other squashing, and no matmul. It must also state what information the construction loses (a root is one of 16, so 4 bits per block) and how it exports as a whole-number D11 operation.
3. Write finite CPU controls in Rust before any trunk run, in the crate your note names (for the discrete path, `crates/uor-r4-discrete`). Examples to propose: the output is stable when applied twice; the range is bounded; ties break deterministically. Run `~/.cargo/bin/cargo test -p <crate>`.
4. Draft the one-page pre-registration sheet in `docs/harness/prep/` (format model: `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`). The arms are the trunk with and without the mechanism. The trunk does not exist yet (see `m1-trunk-assembly-and-training`), so say which trunk or proxy the test uses. Register the bar and noise, controls, seeds and cost. A non-author must give TEST FITNESS: FIT before any GPU run (CHARTER §4.5).
5. Open a PR that says `Closes #<this issue>` when the mechanism, controls and sheet are in, or `References #<this issue>` if partial. Record the result as one row in the research notebook for its line; if no page in `docs/research/notebook/` fits, add one in the format of `docs/research/notebook/README.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:18`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:142`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:64`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:85`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:53`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:60`](../../../docs/start/where-we-are.md)

