---
id: infra-loop-pr04-gates
title: Loop PR 4: loop gates runner, gates.json, ledger and breaks
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
---

## Why it matters

"Done" is proved by checks anyone can rerun. A check that cannot go red proves nothing, so every gate needs a declared fault that turns it red.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "`loop gates`: runner, `gates.json`, ledger with working-tree hashes, REFUSED and UNKNOWN, breaks".

## Done when

`loop gates` reads `docs/harness/gates.json`, in which every entry declares a break. It follows the run rules on the loop page: a gate that cannot run is REFUSED, later gates read UNKNOWN after a red, the true exit code is kept, and a change to a gate's read files during the run voids it. The ledger `local/<lab>/gates.jsonl` marks a result STALE after an uncommitted edit. The planted fault turns the check red: **`--break all` turns each gate red for its own reason.**

## How to start

1. Team (D52): codex or deepseek builds it. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59).
2. Read "Gates" on [the loop page](../../loop/README.md).
3. Open `crates/uor-r4-harness/src/main.rs`, `crates/uor-r4-harness/src/checkdispatch.rs` and `crates/uor-r4-harness/src/ratchet.rs` for today's checks.
4. List today's gates in `gates.json`, each with its break.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 4` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `References #<the loop issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
