---
id: owner-decision-board-status-disputes
title: Settle the disputed D11 and Hopf observation statuses on the board
milestone: infra
area: docs
status: decision
open_to: lab
needs_gpu: false
depends_on: []
assignee: Casey-allard
---

## Why it matters

It keeps the board and its counts (10 ACTIVE, 4 READY, 13 PARKED) accurate.

## What is missing

The board keeps D11 integer serving ACTIVE, as its own page says, but the audit would call it READY because no lawful model has been exported through it. Hopf observation is PARKED, yet its kept-fibre map is built and unit-tested and only sector routing failed, so splitting the two forms may be more accurate. A status change is the owner's call.

## Done when

The owner records the status of D11 serving and whether Hopf is split, and the board and counts are updated to match.

## How to start

Open these first:

- `docs/mechanisms/README.md`
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/hopf-observation.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Owner decision** (§5.1): only the owner decides this. The issue lays out the options and their consequences; labs do not act on their own reading of it.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:23`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:155-156`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:47`](../../../docs/start/transformer-vs-geometry.md)

