# ORDER: infra-loop-pr08-migration, kimi-9 (notes migration to STATE, HANDOFF, PLAN)

Lab: kimi. Item: [infra-loop-pr08-migration](../../../../roadmap/items/infra-loop-pr08-migration.md) (#2913).
Setup row: 8. Base: `fb6df367b`.

## Owner approval

**"ok well I approve of #1"** — Casey in the kimi chat, received 2026-10-22T16:49Z, assigning PR 8 to kimi (#1 on my pending list was "PR 8 (#2913) yes/no") and thereby approving this PR's changes to the owner paths `AGENTS.md` and `docs/integration/DECISIONS.md` (owner-paths.json). The item's D52 (claude builds) and D59 (claude-only review) notes are superseded: by the owner's assignment, and by D59's end (13:56Z) — review is by another vendor.

## Goal

The item's done-when: STATE, HANDOFF and PLAN replace `STATUS.md`, `NOW.md` and the roadmap page's route text, old files moved byte-identical to archive/ with pointers (§4.7); AGENTS.md within the 80-line gate; DECISIONS rows gain "rejected" and "cost to reverse" columns; lessons keyed L-NNN (L-001 = records drifted); FILEMAP rows per folder (two levels); duplicate PM rules moved to pm-rules.md; milestone bars match CHARTER §4.8. Planted fault: `xtask links` fails a planted broken link.

## Cancel probe (run first)

Class instrument (docs + tooling, host only). At base `fb6df367b`: STATE.md, HANDOFF.md, PLAN.md, FILEMAP.md, LESSONS.md, pm-rules.md absent; AGENTS.md 71 lines, no lessons markers; 24 mistakes files, no `id: L-`; STATUS.md/NOW.md hand-edited. The task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopmigrate*.rs`, `crates/uor-r4-harness/tests/loop_migrate*.rs`
- `AGENTS.md`, `STATUS.md`, `STATE.md`, `HANDOFF.md`, `PLAN.md`, `archive/**`, `docs/harness/LESSONS.md`, `docs/harness/mistakes/**`, `docs/harness/pm-rules.md`, `docs/harness/claims.json`, `docs/harness/milestones.json`, `docs/integration/DECISIONS.md`, `docs/integration/NOW.md`, `docs/start/FILEMAP.md`, `docs/roadmap/items/*.md`, this task folder
- Rendered/shared riders, always allowed: `docs/roadmap/README.md`, `docs/harness/gates.json`, `docs/harness/README.md`, `docs/loop/README.md`, `src/lib.rs`/`main.rs`/`loopgates.rs` (small registration edits).

## Done when

1. STATE.md and HANDOFF.md committed as stamped rendered views (`loop render --check` green); PLAN.md PM-written with Goal, `## Premises` (Premise/Probe/Status headers, open cell + backticked command — the deepseek-25 reader), `## Owner directions` (Direction/Status/Date/Note — the deepseek-27 reader), the backwards M1–M8 route, approved sequences, not-doing list.
2. STATUS.md and docs/integration/NOW.md moved byte-identical into `archive/` (dated folder), one-line pointers left.
3. AGENTS.md gains the `loop:lessons-top-ten` markers with the rendered top-ten and stays ≤ 80 lines (duplicates moved to pm-rules.md).
4. DECISIONS.md's format gains "rejected" and "cost to reverse" for new rows (existing rows untouched, append-only).
5. Every mistakes file has `id: L-NNN`; LESSONS.md renders from them.
6. FILEMAP.md covers every directory two levels deep; a missing row or a gone path fails.
7. milestones.json bars verified to match CHARTER §4.8 (reconcile only; the pending bar cards are NOT set here).
8. `loop migrate --check` (new loopmigrate module) verifies byte-identical archive copies, pointers, markers, L-ids, read-path budget, DECISIONS columns, FILEMAP coverage; tests in `tests/loop_migrate*.rs`; a `loop-migrate` gate whose planted fault (broken link planted in PLAN.md) turns `xtask links` red.
9. `loop gates --full` GREEN, `--break all` CAUGHT, `report --check`/`pr-check --local` PASS, receipts bound at the final head; `cargo test -p uor-r4-harness` green.

## Stop and ask if

- Any bar's content would change, or any owner path beyond the two approved ones must change.
- A note line would drop without a byte-identical archive home.
- PLAN.md's table shapes would diverge from the deepseek-25/27 readers.
- The claims.json pr-check allowlist (the carded code fix) seems needed here — it is outside this item's scope and stays carded; only dead-entry cleanup rides (entries whose branches are gone from GitHub).
