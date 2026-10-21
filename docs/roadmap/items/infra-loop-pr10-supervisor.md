---
id: infra-loop-pr10-supervisor
title: Loop PR 10: loop drive, watchers, kill switch and the soft check-in
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr07b-session, infra-loop-pr07c-queue-board, infra-loop-pr09-client-adapters]
---

## Why it matters

The supervisor keeps each lab working without a person watching: it starts fresh sessions, does the mechanical steps, and stops cleanly on the owner's signal.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`drive`, watchers, kill switch, render refresh, compaction hooks, the soft check-in".

## Done when

`loop drive` starts a fresh headless session per unit with the lab's pinned model and stops on the loop page's stops, including the kill switch. A watcher outlives the session for a leased GPU run and never cancels a healthy run. The 2-hour check-in is soft (D53): real progress continues with a note on the issue, a spiraled or stalled session closes, and the PM prompts any lab that stopped. The supervisor stops only its own pids and does not count Claude Code's after-stop summary session as a lab session. The planted fault turns the check red: **a fake stalled client past 2 hours is closed and carded, while a fake progressing one continues with a note.**

## How to start

1. Team (D52): codex or deepseek builds it; the review comes from a lab of a different vendor than the author.
2. Test first, and record in REPORT: a foreground tool's process group, signalling a whole group, and Claude Code's after-stop summary session (the P-2 gaps).
3. Read "The 2-hour check-in and the stops" on [the loop page](../../loop/README.md).
4. Open `scripts/pod/uor-pod` (leases), `crates/uor-r4-harness/src/heartbeatcard.rs` and `crates/uor-r4-harness/src/githubcards.rs`.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask a lab of a different vendor for the exact-head review, and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review from another vendor, merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D53
