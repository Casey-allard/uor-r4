---
id: infra-loop-pr06-review-records
title: Loop PR 6: loop review with tagged review records
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr05-policy-check]
---

## Why it matters

An exact-head review from another vendor's lab is the last check before `main`. A review posted in free text cannot be checked by a machine.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`loop review` with tagged review records".

## Done when

`loop review` writes and parses the `uor-review v1` comment with the fields the loop page lists. The policy check needs an APPROVE at the current head from a lab whose vendor differs from the author's, with no later CHANGES; a doc-only or generated-only PR may carry a same-lab `fresh-context` record. Identity is by tags: there is no signing, no key and no signer list (D55). The planted fault turns the check red: **an untagged or same-vendor review record is refused.**

## How to start

1. Team (D52): codex or deepseek builds it; the review comes from a lab of a different vendor than the author.
2. Read "The review record" and "Identity is by tags" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/prgates.rs`, `crates/uor-r4-harness/src/registries.rs` and `docs/harness/roles.json`.
4. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask a lab of a different vendor for the exact-head review, and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review from another vendor, merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D55
