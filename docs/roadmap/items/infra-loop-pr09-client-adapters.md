---
id: infra-loop-pr09-client-adapters
title: Loop PR 9: loop guard, install and onboard for each client
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr07b-session]
scope: [crates/uor-r4-harness/src/loopguard*.rs, crates/uor-r4-harness/src/loopinstall*.rs, crates/uor-r4-harness/src/looponboard*.rs, crates/uor-r4-harness/src/onboard.rs, crates/uor-r4-harness/src/presence.rs, crates/uor-r4-harness/src/skills.rs, crates/uor-r4-harness/tests/loop_guard*.rs, crates/uor-r4-harness/tests/loop_install*.rs, crates/uor-r4-harness/tests/loop_onboard*.rs, docs/harness/onboarding/**]
---

## Why it matters

Hooks are the fence that stops an out-of-scope write while a session runs. Each of the five clients wires hooks, skills and rules differently, so each needs its own adapter and an onboarding check.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`guard`, `install`, `onboard`; the local copyright check; committed onboarding receipts".

## Done when

`loop guard` is the one hook entry point and exits at once outside the repository and its worktrees. In Claude Code it scopes subagents, because the tool hook names the calling subagent (P-1). `loop install` is run by the owner, backs up each config it edits and writes only to gitignored `local/`. `loop onboard` prints PASS, FAIL or REFUSED per check, with a committed receipt per client. While Claude runs alone, this PR ships the Claude Code adapter and its receipt; each other client's adapter and receipt are added when that lab joins (D52, amended). The guard never runs a release command for a lab (D56). The copyright check runs on the laptop only. The planted fault turns the check red: **each client's canary write is refused.**

## How to start

1. Team (D52, amended): claude builds it while Claude runs alone, starting with the Claude Code adapter; the original split gave it to antigravity. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read "Install and onboarding" and the P-1 and P-2 results on [the loop page](../../loop/README.md). Codex, Kimi, Antigravity and DeepSeek still owe their client probes (#2028); an adapter for an unprobed client waits for its probe.
3. Open `crates/uor-r4-harness/src/presence.rs`, `crates/uor-r4-harness/src/onboard.rs` and `crates/uor-r4-harness/src/skills.rs`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 9` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `Closes #2914` and `References #2904`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D56
