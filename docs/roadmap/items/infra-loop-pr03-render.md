---
id: infra-loop-pr03-render
title: Loop PR 3: loop render for every view, with --check at the stamp
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr02-item-keys]
---

## Why it matters

Records that drifted out of date have cost the project more than any other failure (lesson L-001). Views rendered from committed inputs, and checked against a re-render, cannot drift quietly.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`loop render` for every view, `--check` at the stamp, determinism tests".

## Done when

`loop render` writes STATE, HANDOFF, LESSONS, the AGENTS.md top-ten block, `docs/roadmap/README.md` and `docs/loop/baseline.json` from committed inputs only, with no clock and no GitHub. `--check` re-renders at the stamp and needs an exact match, and two renders of one commit are byte-identical. The planted fault turns the check red: **a hand-edited STATE fails.**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to codex or deepseek. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read "The notes" on [the loop page](../../loop/README.md), including the renderer's inputs.
3. Open `xtask/src/roadmap.rs` (today's roadmap render), `crates/uor-r4-harness/src/main.rs`, `crates/uor-r4-harness/src/status.rs` and `STATUS.md`.
4. Add the subcommand with determinism tests.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 3` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
