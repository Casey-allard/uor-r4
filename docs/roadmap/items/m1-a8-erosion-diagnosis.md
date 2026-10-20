---
id: m1-a8-erosion-diagnosis
title: Diagnose A8 reach erosion from retained checkpoints
milestone: M1
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

Tells the controller design whether the loss comes from the update moves, the read or the instrument. A8 is the address-cell run: tokens are placed in cells so the attention read can reach their targets. If reach erodes in training, the read cannot use what placement gave it. See [address cells](../../mechanisms/address-cells.md).

## What is missing

Claude owns a host-only diagnosis of why Relation reach fell from 1,351 to 551 at Q1, using retained checkpoints and move logs. It recomputes no admission and so far establishes no causal admission-loss conclusion (see the latest attribution correction on #2029). No paired DEV verdict, KEEP or fresh success exists; fresh remains sealed.

The sources give these facts:

- The Relation arm stopped at Q1 (batch 4,096) with 551 reachable positions, below half its starting 1,351 (the stop line is 675.5). Status: `Q1_STOP_FOR_DIAGNOSIS`.
- The FrozenZero control completed 16,404 batches, with reach 857 at Q1. Both arms lost reach: the control by 37%, the Relation arm by 59%.
- The two checkpoint-0 models are byte-equal across arms, and the preflight found placement moved checkpoint-0 admission (1,351 for A8 against 127 for P8).
- The A8 record names its cheapest first step: a per-checkpoint census of token-cell placement on the archived 64-batch checkpoints. Placement can be rebuilt from each checkpoint's table on the host; recomputing admission needs device evaluation, which this item does not do.
- A version that protects the reach is built; its GPU qualification is `NOT_RUN` ([controller record](../../labs/admission-anchor-controller-2026-10-19/README.md)).

## Done when

A diagnosis note from the retained checkpoints and move logs that states which causal claims it does and does not support; recorded as one notebook row.

## How to start

1. Claude owns this diagnosis, so say on #2029 that you want to join before starting. Read `docs/labs/a8-dose-2026-10-19/README.md`, `docs/labs/address-cells-preflight-2026-10-19/README.md` and the latest attribution correction (`gh api repos/UOR-Foundation/uor-r4/issues/comments/6261186285 --jq .body`).
2. Find the retained data. The Q1 counters (attempted, candidate-cost-sensitive, accepted) are in `docs/labs/a8-dose-2026-10-19/evidence/relation-q1-summary.json` and `frozen-zero-q1-summary.json`. The 64-batch checkpoints and move logs are in the pod archive on the HF store (`vast-archive/vast-56670997/`, 442 payload files); see `docs/reference/storage.md` for how to fetch only what you need.
3. Post a design note first, as the first deliverable. It must list each question the host data can answer, and for each claim say supported or not supported given that no admission is recomputed. It must separate three suspects: the update moves, the read, and the instrument. State the checkpoint-0 control the census must reproduce: the byte-equal checkpoint-0 tables and the G-Init pass (table equals the pin word for word).
4. Write the census as a host tool in Rust, not Python, read-only on the retained files. Test it first on checkpoint 0 against the recorded placement facts. Then run it per 64-batch checkpoint for both arms and join it with the move-log counters. A run that moves or edits retained files is wrong.
5. Write the note. Do not call a result KEEP, name a paired DEV verdict, or open fresh. A placement census is not admission, and the note must say so. A new GPU run needs a pre-registration with a non-author FIT first and is out of scope here.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m1-discrete-attention.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:38-43`](../../../docs/integration/NOW.md)
- [`docs/start/where-we-are.md:56`](../../../docs/start/where-we-are.md)
