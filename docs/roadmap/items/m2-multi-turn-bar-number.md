---
id: m2-multi-turn-bar-number
title: Register a number for the multi-turn reply bar blocked under R14
milestone: M2
area: evaluation
status: decision
open_to: lab
needs_gpu: false
depends_on: []
assignee: Casey-allard
---

## Why it matters

M2 cannot gate multi-turn behaviour without it.

## What is missing

The multi-turn reply bar is BLOCKED under R14 (D39 item 1), so the multi-turn check needs a registered number before it can gate M2. No number is registered.

## Done when

A threshold is registered with deterministic graders (D28) and the R14 block is lifted for that check.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `ROADMAP.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Owner decision** (§5.1): only the owner decides this. The issue lays out the options and their consequences; labs do not act on their own reading of it.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:141`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

