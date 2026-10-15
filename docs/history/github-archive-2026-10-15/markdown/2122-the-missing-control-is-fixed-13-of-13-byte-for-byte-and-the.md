# #2122 — The missing control is FIXED: 13 of 13 byte for byte, and the split is still VOID (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:26:09Z
- **Updated:** 2026-10-10T03:23:32Z
- **Closed:** 2026-10-10T01:27:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2122
- **Merged:** 2026-10-10T01:27:32Z
- **Merge commit:** 009533af11a3ed9db8a94a1ded3453c5ab89ff77
- **Base / Head:** main / UOR-Foundation:numeric-trace-reproduction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## 0. The structural qualifier stays visible
**The artifact that answered the v5 "memory" panel carries NO MEMORY OPERATOR — only a copy pointer** — so **"10 of 40 on the memory category" is 10 of 40 for a copy-pointer dialogue stack on requests *named* memory rows.**

## 1. THE MISSING CONTROL IS FIXED: 13 of 13 BYTE FOR BYTE
**Root cause, found by reading `reply_panel` rather than guessing: the panel calls the reply closure ONCE PER TURN, NOT ONCE PER ROW.**

```rust
for request in requests {
    for (turn, user) in request.user_turns.iter().enumerate() {
        let generated = reply(&history, max_new_tokens)?;   // once per turn
```

With 13 three-turn rows the previous probe took the **first 13 calls** — the first few rows' *turns* — as if they were the 13 rows' replies, **which is exactly why `mem-040` and `mem-006` appeared to share a byte-identical sequence: they were not those rows' replies at all.** The fix groups calls by request, takes each row's **last turn** (the reply the check grades), and asserts the call count equals the total turn count so mis-grouping fails loudly.

**Acceptance condition, stated before the build and met exactly: ALL THIRTEEN rows reproduce their sealed replies byte for byte — 13 of 13 — and `mem-040`'s is `[2997, 1700, 386, 2605, 284, 2471, 435, 223, 26, 22, 16, 1]`, the reply containing the stored `84`.** **So the sealed replies ARE reproducible from the artifact by this path**, and the previous piece's "history-insensitive replies" are explained and gone.

## 2. THE SPLIT IS STILL VOID, AND NOT REPORTED
The fixed run produced `READER: 10, PARTIAL: 3` — **not a result either.** A second defect was found: the values file is tab-separated and the probe took **everything after the first tab** as the value, so `digit_ids` came out as **12-token sequences for two-digit values**. Every class assignment is keyed on the wrong token set and **means nothing**. The extraction fix is written, but **the reported numbers come from the pre-fix binary** — so **no reader-versus-emitter classification is claimed, and nothing about pointer selection may be read out of the void run.**

## 3. The four loud-failure conditions
| condition | status |
|---|---|
| 1. Both classes non-empty or INCONCLUSIVE | **not reached** — the deciding run is void |
| 2. `mem-040` positive control | **satisfied at the generation level**; unreadable at the *selection* level from the void run |
| 3. Per-step records for two rows | **not reported** — keyed on the wrong token set |
| 4. Token ids, never strings | **held** throughout |

## 4. The ledger
Token count **REFUTED** · minimal pairs/digit order **REFUTED** · value addressability **REFUTED** · **the learned read/emit path OPEN** — no memory reader on this artifact, so the question is the pointer's; the instrument now reproduces the sealed replies byte for byte (13 of 13), with one defect in its value extraction remaining.

## Delivery
Record [`docs/labs/numeric-trace-reproduction-2026-10-09/README.md`](docs/labs/numeric-trace-reproduction-2026-10-09/README.md); the bin's alignment fix; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store before any local deletion: `numeric-trace-reproduction-2026-10-09.tar`, 97,792 bytes, md5 `c85ac243578086e31e8ecdffb2d0e73f`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

