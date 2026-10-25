---
id: carrier-credit-hard-selection
title: Build a hard-selection step for carrier credit
milestone: research
area: mechanism
status: ready
open_to: lab
needs_gpu: true
depends_on: []
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [docs/harness/prep/carrier-credit-hard-selection*, docs/labs/carrier-credit-hard-selection/**, docs/loop/tasks/carrier-credit-hard-selection/**]
---

## Why it matters

It reopens a PARKED optimizer without a softmax.

## What is missing

Carrier credit sends learning credit back to the geometric objects that carried the information. Three fixed arms fell from 437/512 to 0/512, and the sampled-choice step uses softmax. A hard-selection step does not exist, and the board says it needs the M2 training hold released.

## Done when

A hard-selection credit step replaces the sampled choice with no softmax, on CUDA and integer-only, and the archived three-arm sheet's ON/OFF × two-seed design is rerun as a new pre-registered sheet with a fresh FIT on the live discrete learner (CHARTER §4.5; carrier-credit reopen condition 5), with model-blind in-batch controls, every registered arm run, and the result recorded on fresh rows.

The archived sheet cannot be replayed: its objective is a log-mean-exp over softplus of a log mass (§3.2), its modules are compiled out, and its runner is float code outside the D37 module. Claude decided this reading under D68 (claude-103, [decision card](../../loop/tasks/carrier-credit-hard-selection/claude-103/DECISION.md)), with a fresh decision review.

**`crates/uor-r4-discrete/**` was removed from `scope:` on 25 Oct (claude-105).** Its code work is complete on #3203, and only the GPU results and docs remain. `loop claim` reads an open order's item scope, so the crate glob was locking every discrete item, the M1 path included, against every lab while the arms ran. A code fix that #3203's review asks for restores the glob first, through `infra-loop-harness`.

## How to start

Open these first:

- `docs/mechanisms/carrier-credit.md`
- `archive/labs/unified-geometric-integration-2026-10-11/README.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps. A registered sheet still needs its non-author FIT. Preparation that runs no compute (code, finite CPU tests, the pre-registration sheet) can proceed now.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/mechanisms/README.md:110`](../../../docs/mechanisms/README.md)

