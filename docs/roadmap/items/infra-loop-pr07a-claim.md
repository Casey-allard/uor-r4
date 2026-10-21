---
id: infra-loop-pr07a-claim
title: Loop PR 7a: loop claim, suborder and ask
milestone: infra
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr02-item-keys]
---

## Why it matters

Two labs on one item, or on one file, waste work and eject each other from the queue. A claim made as a draft PR is visible at once, and PR numbers settle races.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`claim`, `suborder`, `ask`".

## Done when

`loop claim` is refused unless the item is ready and unclaimed, its scope overlaps no open order (notebook pages excepted) and the lab is under its open-PR limit; it creates the worktree, commits the ORDER, opens a draft PR and lists again. `loop suborder` refuses a scope outside the parent's. `loop ask --scope` and `--blocked` write the two stop-and-ask cards. The planted fault turns the check red: **an overlapping claim is refused.**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to codex or deepseek. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read step 4 of the loop, "Scope, without a line cap" and "Subagents" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/multilab.rs`, `crates/uor-r4-harness/src/multilabrules.rs` and `docs/harness/claims.json` (today's claims).
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 7a` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
