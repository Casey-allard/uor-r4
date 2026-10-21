---
id: infra-loop-pr11-progress-window
title: Loop PR 11: loop gates --watch and warnings raised to errors
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr04-gates]
---

## Why it matters

A person running the full gates wants to see progress and reds as they land. Some checks that only warn today must become errors once the loop relies on them.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`--watch`; warnings raised to errors".

## Done when

`loop gates --watch` shows gate N of M, reds as they land and a finish estimate from the ledger. It opens only when a person starts a full run in a terminal, never in CI or a headless run, and closes after the run. The named warnings are raised to errors. The planted fault turns the check red: **the window gives up after 5 minutes without change.**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to codex or deepseek. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read "Gates" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/preprules.rs`, `crates/uor-r4-harness/src/prlintrules.rs` and `crates/uor-r4-harness/src/mistakes.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 11` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
