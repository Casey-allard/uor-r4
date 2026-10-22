---
id: infra-loop-pr12-git-hooks
title: Loop PR 12: git hooks, pr-check before push, Lab and Order trailers
milestone: infra
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr05-policy-check, infra-loop-pr06-review-records]
scope: [crates/uor-r4-harness/src/githelp.rs, crates/uor-r4-harness/src/version.rs, crates/uor-r4-harness/src/loophooks*.rs, crates/uor-r4-harness/tests/loop_hooks*.rs, crates/uor-r4-harness/tests/git_hooks*.rs, rules/*-hook.sh]
---

## Why it matters

Checks that run before a commit or push catch faults before they reach a PR, and trailers tie each commit to its lab and order after the squash merge.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "Git hooks: binary current; `pr-check --local` before push; `Lab:` and `Order:` trailers".

## Done when

A commit hook checks that the binary is current and that `Lab:` and `Order:` trailers are present; a pre-push hook runs `pr-check --local`. Trailers are tags, not proof (D55). The planted fault turns the check red: **a commit without trailers is refused.**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone; the original split gave it to codex or deepseek. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read the P-3 result and "Identity is by tags" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/githelp.rs` and `crates/uor-r4-harness/src/version.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 12` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `Closes #2917` and `References #2904`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D55
