---
id: infra-loop-pr08-migration
title: Loop PR 8: migrate the notes to STATE, HANDOFF and PLAN
milestone: infra
area: docs
status: ready
open_to: lab
needs_gpu: false
depends_on: [infra-loop-pr03-render]
scope: [crates/uor-r4-harness/src/loopmigrate*.rs, crates/uor-r4-harness/tests/loop_migrate*.rs, crates/uor-r4-harness/tests/loop_prcheck.rs, AGENTS.md, STATUS.md, STATE.md, HANDOFF.md, PLAN.md, archive/**, docs/harness/LESSONS.md, docs/harness/mistakes/**, docs/harness/pm-rules.md, docs/harness/claims.json, docs/harness/milestones.json, docs/integration/DECISIONS.md, docs/integration/NOW.md, docs/start/FILEMAP.md, docs/roadmap/items/*.md, crates/uor-r4-harness/src/mistakes.rs, crates/uor-r4-harness/src/looprender.rs]
---

## Why it matters

Today `STATUS.md`, `NOW.md` and `claims.json` are edited by hand and drift, and several files repeat the same rules. Moving to rendered notes and one list of rule sources removes both problems.

## What is missing

The build plan's row on [the loop page](../../loop/README.md#build-plan): "Migration of the notes; AGENTS.md under the 80-line gate; DECISIONS columns; lesson ids; FILEMAP; `pm-rules.md`; bars reconciled".

## Done when

STATE, HANDOFF and PLAN replace `STATUS.md`, `NOW.md` and the roadmap page, with the old files moved byte-identical to the archive with pointers (§4.7). AGENTS.md stays within the 80-line gate; new DECISIONS rows carry "rejected" and "cost to reverse" columns; lessons have `L-NNN` keys; FILEMAP has a row per folder; duplicates in `pm-rules.md` are removed; milestone bars match CHARTER §4.8. Each charter-touching step has the owner's approval. The planted fault turns the check red: **`xtask links` fails a planted broken link.**

## How to start

1. Team (D52): claude (architecture lead) builds it. This is a loop setup PR, so its exact-head review comes from Claude's fresh-context reviewers (D59); Kimi does not review it while the other labs are not running the loop.
2. Read "The notes" and "Charter changes" on [the loop page](../../loop/README.md).
3. Open `STATUS.md`, `docs/integration/NOW.md`, `docs/harness/claims.json`, `docs/harness/pm-rules.md`, `docs/harness/milestones.json`, `docs/harness/mistakes/` and `AGENTS.md`.
4. Ask the owner in your own chat before each charter-touching step and record the approval verbatim with the time.
5. Carry an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, build and test on the laptop with `~/.cargo/bin/cargo`, put `Setup row: 8` in the ORDER (PR 6's gate reads it), ask Claude for the exact-head review (D59: no other lab is asked), and open the PR with `Closes #2913` and `References #2904`.

## Rules that apply

- **Owner paths in scope:** `AGENTS.md` and `docs/integration/DECISIONS.md` are owner paths (`docs/harness/owner-paths.json`); a PR that changes one needs the owner's approval recorded in its ORDER (`pr-check`'s `owner-paths` rule).
- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for this setup PR, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md) §5.10
- [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) D52
