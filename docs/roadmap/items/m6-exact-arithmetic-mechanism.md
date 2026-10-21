---
id: m6-exact-arithmetic-mechanism
title: Design exact arithmetic on the geometric path
milestone: M6
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-core/**, crates/uor-r4-integer/**, docs/geometry/spiralcore.md, docs/mechanisms/exact-addressed-memory.md, docs/mechanisms/spiralcore-fbs.md, docs/mechanisms/ternary-xor.md, docs/labs/m6-exact-arithmetic-mechanism/**]
---

## Why it matters

M6 asks for executable reasoning and coding on the same model, and the project names exact arithmetic as the visible gap. Multi-step compositions and code tasks both need numbers to come out exactly right, and nothing in the repository yet says how the geometric stack does that. A mechanism here feeds the frozen M6 panel and the final M6 qualification. It sits on the same typed operators as the rest of the model; see [how SpiralCore transport works](../../mechanisms/spiralcore-fbs.md).

## What is missing

Exact arithmetic is named as the visible gap for reasoning and coding (`STATUS.md`, `docs/start/where-were-going.md`), but the sources specify no mechanism. It must stay on the same geometric stack with typed operators (SpiralCore Cl(0,6)/E8/FBS, icosian transport, exact addressed memory), stay exportable under the D11 serving contract, and obey the bans on matmul and squashing. Issue #2034 adopts no second model and no external code-generation teacher (D28).

## Done when

A mechanism with finite controls on exact arithmetic items, integrated through the shared operators with no second model or external teacher. The bar is not yet registered: how many items, which operations and what pass line are all undefined in the sources. Registering them is part of step 3 below.

## How to start

1. Read what exists. Open `docs/mechanisms/spiralcore-fbs.md`, `docs/geometry/spiralcore.md`, `docs/mechanisms/exact-addressed-memory.md`, `docs/mechanisms/ternary-xor.md` and `docs/start/transformer-vs-geometry.md`. Read the M6 scope with `gh issue view 2034 --repo UOR-Foundation/uor-r4`. The sources do not say how numbers are represented today, so write down what you find rather than assuming.
2. Post a design note on #2034 as your first deliverable. It must give (a) the geometric construction: how a number and an operation such as addition with carry are represented as atoms, binder paths or addressed-memory entries, and which typed operator does each step; (b) why no step needs softmax or squashing (CHARTER §3.2) or dense matrix multiplication (§3.3), including hand loops; (c) how the result serves as integers under D11; (d) which finite controls would show it works or fails.
3. In the same note, propose the bar for the controls (operations, operand range, item count, pass line) as a decision card to the PM or owner. A bar is not yours to set (CHARTER §5.2).
4. Build the finite CPU controls in Rust in the crate that owns the operator you extend (`ls crates/`; candidates are `crates/uor-r4-core` and `crates/uor-r4-integer`; say which in the note). Each control is a Rust test with exact expected values, including cases that must fail. Run `~/.cargo/bin/cargo test -p <crate>` (for `uor-r4-integer` add `--features serving-wasm`, because the serving modules compile only with it) and `uor-harness check` (see `docs/harness/README.md`). Finite tests need no pre-registration (CHARTER §4.5).
5. Keep the construction independent of any model teacher: no distillation, no donor, no neural grader (CHARTER §3.10). Check items for overlap with the M6 panel once `m6-reasoning-panel-freeze` has frozen it, so the controls are not the panel.
6. If the controls pass, draft a one-page pre-registration sheet for any GPU run that would train or score the mechanism (bar, noise, controls, seeds, cost, what each outcome decides). It needs a non-author TEST FITNESS: FIT review before it runs, and a release typed by Casey; this item does not release anything.
7. Open a PR that says `Closes #<this issue>` when the note and controls are in, or `References` if only part is done. There is no M6 notebook page yet. Add `docs/research/notebook/m6-reasoning-coding.md` in the format of `docs/research/notebook/README.md` and record one row per control result. A negative closes that configuration, never the mechanism.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:21`](../../../STATUS.md)
- [`docs/start/where-were-going.md:63`](../../../docs/start/where-were-going.md)
- issue#2034
