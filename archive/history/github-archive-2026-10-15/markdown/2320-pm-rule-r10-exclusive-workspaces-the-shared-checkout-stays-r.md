# #2320 — PM rule R10: exclusive workspaces, the shared checkout stays read-only (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T12:06:43Z
- **Updated:** 2026-10-13T12:08:17Z
- **Closed:** 2026-10-13T12:07:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2320
- **Merged:** 2026-10-13T12:07:54Z
- **Merge commit:** 9556c1bd3be6a7afab62d4e4f45aec51f0885b17
- **Base / Head:** main / UOR-Foundation:kimi/pm-rule-r10-workspaces-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## PM rule R10: exclusive workspaces — the shared checkout stays read-only

Docs-only. Record = `docs/harness/pm-rules.md` itself; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (reply 37/232, v5 memory 20/40); the milestone's active pre-registration is the rung-0 sheet, and this PR does not touch it.

### What changed and why

- `docs/harness/pm-rules.md` gains **R10 — Exclusive workspaces: the shared checkout is read-only** (appended verbatim as specified): the owner checkout stays on clean `origin/main` and is read-only for every lab; work happens in a claimed, isolated worktree; builds use a per-worktree `CARGO_TARGET_DIR`; harness writes run from your own worktree so receipts ride your delivery PR; coordination state is authoritative only once merged. Motivated by today's repeated collisions — heartbeat/presence writes landing in another session's checkout, the claims store clobbered three times by concurrent writers, and a ledger shard losing receipts in a merge (#2305), the same class seen during the M1 #2029 delivery runs.
- `docs/harness/README.md` — the registry row now reads (R1–R10); it was the only live "R1–R9" reference (checked with a repo grep).
- `docs/harness/claims.json` — releases `kimi/rung0-sheet-amend3` (branch merged as #2306 and deleted) and carries this branch's claim; claude's entry kept byte-identical.
- A newest-first `docs/integration/current-state.md` line (two sentences); claim/heartbeat receipts ride along.

### Verification

- [x] Checks run at this exact head (`c225d4b8d`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors; warns are claude's stale claim awaiting its holder's release plus the standing notices
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Docs only — no Rust source and no `scripts/harness/*.py` touched
- [x] Files two deliberate lint warns: `prlint.count_matches_registry` ("no model line" is not an M1 registry line — governance work) and `prlint.record_path` (the rules doc itself is the record, as in #2295)

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

