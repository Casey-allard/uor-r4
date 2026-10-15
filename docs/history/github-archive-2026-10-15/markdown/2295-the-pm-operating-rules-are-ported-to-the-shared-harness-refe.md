# #2295 — The PM operating rules are ported to the shared harness (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:04:30Z
- **Updated:** 2026-10-13T05:07:33Z
- **Closed:** 2026-10-13T05:06:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2295
- **Merged:** 2026-10-13T05:06:59Z
- **Merge commit:** 84287b397a0a540cbbdcefd41995921a04682136
- **Base / Head:** main / UOR-Foundation:kimi/pm-rules-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: no model line — governance · count 0/3 · headline 37/232 → 37/232 (unchanged)

## The PM operating rules are ported to the shared harness

Docs/registry only. Record = the rules themselves: `docs/harness/pm-rules.md`.
Decision: **NOT YET PROMOTED** — no model line, no compute, no count reset, headline unchanged (M1 reply 37/232, v5 memory 20/40). No `scripts/harness/*.py` file is touched (the harness code is DeepSeek's).

### What changed and why

Owner direction: the other labs are expecting the PM's rules ported over to the shared workflow. `docs/harness/pm-rules.md` records nine owner-ratified rules, each with its rule, why and the harness surface that carries it: **R1** one owner per branch; **R2** one live branch per active PR; **R3** status discipline (heartbeat every 90 minutes or 3 merged PRs); **R4** no model compute without a `TEST FITNESS: FIT` prep sheet — the live example is the rung-0 sheet on M1 #2029; **R5** ratchets green on every PR; **R6** preservation; **R7** collision resolution by the PM; **R8** delivery truthfulness; **R9** the roster.

Also:

- `docs/harness/claims.json` — the two stale kimi claims (`kimi/d27-rust-first-2026-10-13` and `kimi/rung0-sheet-bars-2026-10-13`; both branches merged and deleted) are removed, and the file carries this branch's own claim — R1's lesson is that a claim is visible repo-wide only once its committed change merges. Codex's restored `codex/geometric-head-shared-writer-20261013` claim currently exists only in the owner checkout's uncommitted working tree; Codex/DeepSeek will commit it from there, and it is deliberately not added here.
- `docs/harness/state-formats.json` — deliberately re-frozen. The claims key moves from the released d27 branch to this branch's claim, marked `optional`, because a branch key is transient: present while its branch lives, absent after release, so a future release is not format drift. (Without the re-freeze, removing a pinned key is a `harness.state_formats` error by design.)
- `docs/labs/README.md` — one roster line: Codex is also Sol (alias, same seat); Antigravity is the fifth lab, currently offline. `docs/harness/README.md` — one registry row for `pm-rules.md`.
- A newest-first `docs/integration/current-state.md` entry; claim and heartbeat receipts ride along.

### Verification

- [x] Checks run at this exact head (`476f93812`, with the same checks passing at the pre-merge branch head `636fba2e6`) — commands and outcomes below
- [x] `python3 scripts/harness/cli.py check` — 0 errors; the warns are pre-existing (the rung-0 sheet's expected `TEST FITNESS: PENDING`, other milestones' unfrozen panels, another lab's orphan branch, the caps notice)
- [x] `python3 scripts/harness/cli.py lint check --body-file <this body>` — 0 errors
- [x] `python3 scripts/harness/statecheck.py` — claims ok (the deliberate re-freeze)
- [x] `python3 scripts/harness/cli.py multilab claims` — run before and after the edit; claims.json stays valid JSON the CLI reads, and this branch's claim is visible
- [x] `python3 scripts/check_claim_wording.py` — PASS
- [x] `python3 scripts/check_no_softmax.py` — OK, 1649 (baseline 1649)
- [x] `python3 scripts/check_no_matmul.py` — OK, 93 sites / 26 files (baseline 93)
- [x] Docs/registry only — no Rust source and no `scripts/harness/*.py` touched
- [x] Files a deliberate lint warn: `prlint.record_path` wants a dated `docs/labs/...` record; here the rules doc is the record (as with the sheet in #2272) and the current-state entry points at it

FIT does not apply (no model compute). References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

