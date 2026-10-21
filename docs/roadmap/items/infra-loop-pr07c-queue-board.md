---
id: infra-loop-pr07c-queue-board
title: Loop PR 7c: loop queue and loop board
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr05-policy-check, infra-loop-pr06-review-records]
---

## Why it matters

The policy gate is advisory on GitHub (D54), so the supervisor's queue step is the lock. The board lets anyone see all five labs at once.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`queue`, `board`".

## Done when

`loop queue` queues a PR only when the policy check is green at the head, the APPROVE that PR 6's gate accepts is at the head (Claude's for a loop setup PR under D59; otherwise another vendor's), no later CHANGES that the gate counts exists, and no owner or PM comment came after the approval. It refuses required checks that did not come from the policy job, and `.github/**` changes made without the owner's approval. `loop board` is a read-only view of the five labs. The planted fault turns the check red: **a later CHANGES record blocks queueing, unless it comes from a lab other than Claude on a setup PR (D59).**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to codex or deepseek. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read step 12 of the loop and the P-4 result on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/statusdelivery.rs` and `crates/uor-r4-harness/src/status.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 7c` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D54, D55
