---
id: m1-position-mechanism
title: Design a lawful position mechanism
milestone: M1
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [crates/uor-r4-discrete/**, docs/geometry/e8-h4-and-the-600-cell.md, docs/geometry/quaternions-and-rotation.md, docs/harness/prep/m1-position-mechanism*, docs/mechanisms/recurrence-gates.md, docs/labs/m1-position-mechanism/**]
---

## Why it matters

A transformer adds a signal that says where each token sits in the sequence. This model has no such mechanism, so the Stage 3 trunk cannot be assembled without one. This is a design task that needs no GPU and is open to anyone. See [the transformer-to-geometry map](../../start/transformer-vs-geometry.md) (Gap 1).

## What is missing

No position mechanism is assigned, and no lawful evidence exists for one. The current model keeps the most recent tokens in order at the leaves of a small tree, so order is in its layout, but nothing is learned about it. The only related result is about carrying a running state by quaternion rotation: it beat identity by 0.071 and 0.077 nats in an `rrarra` core at 1,000 updates. That core had softmax attention, a sigmoid/exp gate and dense MLPs, ran in the lab sandbox, and its identity arm had 332,928 fewer parameters. The record itself asks for a parameter-matched real-gate control, and the quaternion arm was no better than Householder at one paired seed. "Rotation can carry order" is a **hypothesis**. The route-holonomy and icosian-holonomy reads are held proposals (`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:15`). The census lists the icosian holonomy lanes as a dormant design with no model test (`docs/labs/course-correction-2026-10-18/audits/mechanism-census.md:82`). The synthesis row points to a `ROADMAP.md` retained table, but the current `ROADMAP.md` has no such table.

## Done when

A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. The bar for "helps order" is not yet registered; step 4 registers it.

## How to start

1. Read the evidence and its limits. Open `archive/integration/transport-attribution-d1-2026-09-28.md`, `docs/mechanisms/recurrence-gates.md`, `docs/research/findings.md`, `docs/start/transformer-vs-geometry.md` (Gap 1), `docs/geometry/e8-h4-and-the-600-cell.md` (the section "A design off the current path: icosian holonomy lanes"), `docs/labs/course-correction-2026-10-18/audits/mechanism-census.md` (line 82), `docs/geometry/quaternions-and-rotation.md` and `docs/model/architecture.md`.
2. Post a design note on the issue as your first deliverable. It must give the geometric construction (for example quaternion transport along the sequence, or a holonomy read; the sources assign none), and the gate or update rule it uses with no sigmoid, exponential or softmax. It must show how it avoids matmul, and how it exports as whole numbers. It must also define "helps order": for example, the gain must vanish when the context order is shuffled, and the control arm must have matched parameters.
3. Write finite CPU controls in Rust, in the crate your note names (for the discrete path, `crates/uor-r4-discrete`). Examples to propose: the identity case leaves the state unchanged; composition along a sequence is checked against a direct computation; the state stays within its integer range. Run `~/.cargo/bin/cargo test -p <crate>`.
4. Draft the one-page pre-registration sheet in `docs/harness/prep/` (format model: `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`). Register the order test, the bar and noise, the matched-parameter control, seeds and cost. The trunk does not exist yet (see `m1-trunk-assembly-and-training`), so say which trunk or proxy it runs on. A non-author must give TEST FITNESS: FIT before any GPU run (CHARTER §4.5).
5. Open a PR that says `Closes #<this issue>` when the mechanism, controls and sheet are in, or `References #<this issue>` if partial. Record the result as one row in the research notebook for its line; if no page in `docs/research/notebook/` fits, add one in the format of `docs/research/notebook/README.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:15`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:132`](../../../docs/mechanisms/README.md)
- [`docs/model/architecture.md:63`](../../../docs/model/architecture.md)
- [`docs/research/open-questions.md:84`](../../../docs/research/open-questions.md)
- [`docs/start/transformer-vs-geometry.md:51`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/start/where-we-are.md:60`](../../../docs/start/where-we-are.md)
- [`docs/start/where-were-going.md:48`](../../../docs/start/where-were-going.md)

