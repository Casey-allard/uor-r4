---
id: m2-multi-turn-bar-number
title: Register a number for the multi-turn reply bar blocked under R14
milestone: M2
area: evaluation
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [docs/harness/milestones.json, docs/harness/panels/m2-multi-turn*.json, docs/harness/panels/v5-reply.json, docs/harness/prep/m2-multi-turn*, crates/uor-r4-harness/src/panels.rs, crates/uor-r4-harness/src/panel_m2_multi_turn.rs, crates/uor-r4-harness/tests/panel_m2_multi_turn.rs, docs/research/notebook/m2-grounded-reply.md, docs/roadmap/items/m2-multi-turn-bar-number.md, docs/loop/tasks/m2-multi-turn-bar-number/**]
---

## Why it matters

M2 cannot gate multi-turn behaviour without it.

## What is missing

**November 17, 2026.** The October 24 owner direction assigns the lab a recommended numerical threshold and deterministic graders. A D68 whole-project decision review records the resulting card under the owner's standing approval. Until then, the multi-turn check remains outside the gating result. [Issue #2850](https://github.com/UOR-Foundation/uor-r4/issues/2850#issuecomment-6379433704) · [CHARTER.md](https://github.com/UOR-Foundation/uor-r4/blob/main/CHARTER.md)

The multi-turn reply bar is BLOCKED under R14 (D39 item 1), so the multi-turn check needs a registered number before it can gate M2. No number is registered.

## Done when

A threshold is registered with deterministic graders (D28) and the R14 block is lifted for that check.

## How to start

Inspect the suspended diagnostic, its noise and deterministic graders in [the historical multi-turn panel](../../harness/panels/v5-reply.json). Use those data to propose a threshold and document the named panel, exact grader and controls on a D68 card. Keep multi-turn outside the M2 gate until registration and retain the existing saved and fresh reply bars. [M2 notebook](../../research/notebook/m2-grounded-reply.md) · [Milestone registry](../../harness/milestones.json)

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Decision review** (§5.1). The lab drafts the numerical bar and deterministic graders, obtains its own decision review and logs the card for the owner.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:141`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

