---
id: infra-loop-pr09-client-adapters
title: Loop PR 9: loop guard, install and onboard for each client
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr07b-session]
---

## Why it matters

Hooks are the fence that stops an out-of-scope write while a session runs. Each of the five clients wires hooks, skills and rules differently, so each needs its own adapter and an onboarding check.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`guard`, `install`, `onboard`; the local copyright check; committed onboarding receipts".

## Done when

`loop guard` is the one hook entry point and exits at once outside the repository and its worktrees. In Claude Code it scopes subagents, because the tool hook names the calling subagent (P-1). `loop install` is run by the owner, backs up each config it edits and writes only to gitignored `local/`. `loop onboard` prints PASS, FAIL or REFUSED per check, with a committed receipt per client. The guard never runs a release command for a lab (D56). The copyright check runs on the laptop only. The planted fault turns the check red: **each client's canary write is refused.**

## How to start

1. Team (D52): antigravity builds it; the review comes from a lab of a different vendor than the author.
2. Read "Install and onboarding" and the P-1 and P-2 results on [the loop page](../../loop/README.md). Codex, Kimi, Antigravity and DeepSeek still owe their client probes (#2028); an adapter for an unprobed client waits for its probe.
3. Open `crates/uor-r4-harness/src/presence.rs`, `crates/uor-r4-harness/src/onboard.rs` and `crates/uor-r4-harness/src/skills.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask a lab of a different vendor for the exact-head review, and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review from another vendor, merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D56
