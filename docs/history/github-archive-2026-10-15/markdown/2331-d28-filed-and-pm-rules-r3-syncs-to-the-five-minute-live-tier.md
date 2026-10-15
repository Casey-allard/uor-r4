# #2331 — D28 filed, and pm-rules R3 syncs to the five-minute live-tier heartbeat (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T14:02:27Z
- **Updated:** 2026-10-13T14:04:03Z
- **Closed:** 2026-10-13T14:03:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2331
- **Merged:** 2026-10-13T14:03:38Z
- **Merge commit:** 2dc799153cd374adbe71d31657ba603040281afe
- **Base / Head:** main / UOR-Foundation:kimi/d28-r3-sync-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## D28 filed, and pm-rules R3 syncs to the five-minute live-tier heartbeat

Docs-only, two batched items. Record = the changed documents themselves; the newest-first `docs/integration/current-state.md` entry is the change log.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (reply 37/232, v5 memory 20/40).

### What changed and why

- **D28 in `docs/integration/DECISIONS.md`** (next after D27, in the file's section format): the owner's direction, relayed by Claude lab ([#2028 comment 6147245657](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6147245657)) — no transformer or neural-network comparisons of any kind (comparators, controls, baselines, teachers, and neural graders, including the v5 `grade-replies` qwen2.5:7b grader) until the owner declares the native model works end to end; the declaration is the owner's, not a lab's. Panels are scored with deterministic checks only (membership checks, exact-answer oracles); a bar scoreable only by a neural grader today is recorded **BLOCKED** and not run. Comparisons stay inside the native line (prior native rungs, lawful count references). It settles the rung-0b card's open grader question (#2032), supersedes older transformer-baseline implications, and is not retroactive — historical transformer-comparator evidence keeps its scope and nothing is deleted.
- **`docs/harness/pm-rules.md` R3 synced to post-#2327 reality**: check in **at least every five minutes** while active; presence is live-tier; the Python `cli.py heartbeat` delegates to the Rust `uor-harness heartbeat` (the only implementation), whose missing binary is a usage error naming the build command (`CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`); **STALE** marks a lab quiet past the five-minute window; live tier at `~/.local/share/uor-r4/harness`, no checkout writes. The rest of R3 is unchanged.
- A current-state line records both; the entry also notes the rung-0 pre-registration sheet is FIT at `be5d8cd6e` awaiting the owner's training-hold release.
- `docs/harness/claims.json` set to main's current entries plus this branch's claim (released my three stale kimi claims that a store merge had regathered into the worktree file; claude's and codex's entries kept byte-identical).

### Verification

- [x] Checks run at this exact head (`6f9bf90ab`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — **introduces no new errors**: it shows exactly the one known **pre-existing** error, `receipts.parse [docs/harness/ledger/codex-2026-10.jsonl]: line 27 has unknown kind 'receipt'` (routed to DeepSeek, #2028 comment 6146858921). Proven by `git stash` of all four changed files: the identical single error appears without this change set
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors (deliberate `record_path` warn, as in every governance PR)
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] Docs only — no `crates/` changes, nothing under `local/` committed

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

