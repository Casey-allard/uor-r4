---
id: infra-loop-pr05-policy-check
title: Loop PR 5: loop pr-check and a policy job feeding the required checks
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr02-item-keys, infra-loop-pr03-render, infra-loop-pr04-gates]
---

## Why it matters

The loop's rules need one check that judges every PR the same way, running the base branch's code. There is no org admin, so there is no GitHub App and no ruleset change (D54): the policy result must reach the checks that are already required.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`loop pr-check`; `gate.yml` with the policy job advisory".

## Done when

`loop pr-check` checks order, scope at the base, commit order, review record, REPORT, owner paths, rule sources, the rendered-file lock, read path and FILEMAP. The policy job runs base code on PR events and feeds the `required-transport` job in `.github/workflows/ci.yml`, which publishes the five required checks (probe P-4). `pr-check` refuses a PR whose required checks did not come from the policy job, or that changes `.github/**` without the owner's approval; `loop queue` (PR 7c) applies the same refusal. The planted fault turns the check red: **one fixture PR per rule fails.**

## How to start

1. Team (D52): codex or deepseek builds it. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read the P-4 result and "CI (to build)" on [the loop page](../../loop/README.md).
3. Open `.github/workflows/ci.yml` (the `required-transport` job), `crates/uor-r4-harness/src/prgates.rs` and `crates/uor-r4-harness/src/prlintrules.rs`.
4. This PR changes `.github/**`, so ask the owner in your own chat and record the approval verbatim with the time.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 5` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D54
