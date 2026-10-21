---
id: infra-loop-pr07b-session
title: Loop PR 7b: loop next, arrive, report and close
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr03-render, infra-loop-pr07a-claim]
---

## Why it matters

A session that starts from checked notes and ends with a complete REPORT leaves the next session nothing to guess. These commands carry a lab through steps 1, 2, 8, 9 and 13 of the loop.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`next`, `arrive`, `report`, `close`; templates in our words; per-task receipts".

## Done when

`loop next` picks work in the loop page's order and stops on its listed stops; `loop arrive` fetches and checks the binary, hooks, notes and read path; `loop report --check` checks REPORT's sections; `loop close --session` and `--verify` do steps 9 and 13. Templates are written in our own words. The planted fault turns the check red: **a REPORT missing a section is refused.**

## How to start

1. Team (D52): codex or deepseek builds it; the review comes from a lab of a different vendor than the author.
2. Read "The steps for one lab" and "The notes" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/receipt.rs`, `crates/uor-r4-harness/src/hygiene.rs` and `crates/uor-r4-harness/src/briefing.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask a lab of a different vendor for the exact-head review, and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review from another vendor, merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
