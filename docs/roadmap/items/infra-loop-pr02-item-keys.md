---
id: infra-loop-pr02-item-keys
title: Loop PR 2: item keys, Cancel if, done-when class and validator
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

The loop's scope check, its cancel probe and its review rule all read facts from the roadmap item. Today an item states none of them, so nothing can compare an order's scope with its item's.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "Item keys, 'Cancel if', done-when class, validator".

## Done when

`roadmap check` accepts and validates `scope:` (globs), `size:`, `lane:`, `rank:`, `notebook:`, `done_class:` (`host` or `gpu`) and a `## Cancel if` section, as the loop page's scope section lists them; they become required once the PM backfills the ready items. The planted fault turns the check red: **an item scoped `**` is refused.**

## How to start

1. Team (D52): codex or deepseek builds it; the review comes from a lab of a different vendor than the author.
2. Read "Scope, without a line cap" on [the loop page](../../loop/README.md).
3. Open `xtask/src/roadmap.rs` (`REQUIRED_KEYS`, `SECTIONS`, `AREAS`) and two items in `docs/roadmap/items/`.
4. Add the keys, the section and unit tests; run `roadmap check` and `roadmap render --check`.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, ask a lab of a different vendor for the exact-head review, and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review from another vendor, merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
