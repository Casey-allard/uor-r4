---
id: m6-causal-ablation-instrument
title: Build the intermediate-state ablation instrument for M6
milestone: M6
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: [m6-reasoning-panel-freeze, m1-trunk-assembly-and-training]
---

## Why it matters

M6 ([#2034](https://github.com/UOR-Foundation/uor-r4/issues/2034)) asks for executable reasoning and coding on the same model. This instrument is half of its acceptance criteria: it is how the project shows the answers come from the geometry and not from memorized templates. It runs on the frozen panel from `m6-reasoning-panel-freeze` and on the trunk described in [the geometric stack](../../mechanisms/geometric-stack.md) and [exact addressed memory](../../mechanisms/exact-addressed-memory.md).

## What is missing

M6 needs causal intermediate-state ablation showing reasoning and code generation depend on geometric state transport and addressed memory rather than memorized templates. No instrument exists. It needs model hooks that depend on the trunk's state design, which is still being decided under M1. The sources give no numeric bar for the size of the ablation effect; the bar is not yet registered.

## Done when

An ablation instrument registered before any run, able to knock out state transport and addressed memory and report the effect on the frozen panel. Registering the effect-size bar is part of this item (step 4).

## How to start

1. Read the body of issue #2034 (`gh issue view 2034`), then the sibling items `docs/roadmap/items/m6-reasoning-panel-freeze.md`, `m6-qualify-reasoning-coding.md` and `m1-trunk-assembly-and-training.md`, then `docs/mechanisms/geometric-stack.md`, `docs/mechanisms/exact-addressed-memory.md` and `docs/model/architecture.md`. Read `CHARTER.md` §3.10: panels are graded deterministically, and neural graders or comparators are not allowed.
2. Post a one-line intent note, then a design note on this issue before any code. It must define: (a) what "intermediate state" is in the trunk and where state transport and addressed-memory reads happen; (b) each knockout as a geometric operation at those points (what replaces the state or the read), with the reason it is geometric; (c) why none of it needs softmax, normalization by exponential or dense matmul; (d) the finite CPU controls: a no-knockout hook that is bit-identical to the unhooked path, and a tiny synthetic trunk where each knockout provably changes the output.
3. Ask on the issue which crate holds the trunk state (list `crates/` with `ls crates/`). The hook points depend on the trunk's state design, so the note should state which M1 trunk design it assumes. If that design is not settled, post the note as a draft against the assumption and mark it so.
4. Write a one-page pre-registration sheet: the knockout arms (state transport off, addressed memory off, both), the frozen panel and its sha256 from `m6-reasoning-panel-freeze`, the metric reported per task family (compositions, altered constraints, code tasks), the effect-size bar (not yet registered; propose one and send it to the PM as a decision card), seeds and stop rules. Get a non-author FIT review before any GPU run (CHARTER §4.5). Panel scoring runs on the GPU (§3.5) through `scripts/pod/uor-pod`.
5. Implement the hooks and the instrument in Rust. Add the control tests from step 2 and run them with `~/.cargo/bin/cargo test -p <crate>`; run `uor-harness check` before pushing.
6. Open a PR that says `Closes #<this issue>` once the instrument is registered, or `References #<this issue>` if only the design or hooks land. Record the result as one row in a notebook. No M6 notebook page exists yet, so confirm the page name with the PM and follow `docs/research/notebook/README.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2034
- [`ROADMAP.md:19`](../../../ROADMAP.md)
