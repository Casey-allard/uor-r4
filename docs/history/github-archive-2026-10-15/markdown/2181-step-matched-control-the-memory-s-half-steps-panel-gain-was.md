# #2181 — Step-matched control: the memory's half-steps panel gain was the shorter fit, not the memory (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T21:49:50Z
- **Updated:** 2026-10-10T21:55:25Z
- **Closed:** 2026-10-10T21:52:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2181
- **Merged:** 2026-10-10T21:52:47Z
- **Merge commit:** e82b2c9c84ab8bf517ee8db1bef237728a75d394
- **Base / Head:** main / UOR-Foundation:deepseek/addressed-memory-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The control the last cycle named settles its loose end: the product-key memory's one better panel reading (22/40 at half the steps) was the **shorter fit**, not the memory — the identical recipe with **no memory** at the same 1,000 steps reads **22/40** too (wrong-value 12 against the memory arm's 11).** The memory configuration therefore contributes nothing measurable on the frozen v5 panel at either step count, and at the anchor's steps it sits one row below the no-memory anchor while costing **+0.026 BPB** on the chat held-out stream. A rejected *configuration*, not a closed mechanism (D22 §1). References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory (D22 order 2) · count 0/3 · headline before reply 30/232 + memory 19/40 → after: control **22/40 with no memory at the same steps**

## The paired reading

| arm | memory | steps | device | v5 memory  | unknowable | wrong-value | derangement |
|---|---|---|---:|---:|---:|---:|---:|
| **C1 / C2 (control, no memory)** | — | 1,000 | GPU | **22/40** | 3/24 | **12** | 0 |
| cpuM1 (memory) | yes | 1,000 | CPU | 22/40 | 2/24 | 11 | 2 |
| G1 / G2 (memory) | yes | 2,000 | GPU | 19/40 | 0/24 | 17 / 15 | 1 / 2 |
| D10 (adopted base, no memory) | — | 2,000 | GPU | 20/40 | 0/24 | 17 | 0 |

**Two measurements kept.** The control's two seeds are **bit-identical** (`90c382d6…`, 63 s wall each): without an added memory, `seed=` has nothing to initialise and every weight comes from the checkpoint — so the memory arms' seed variation is entirely the added memory's, the property [#2170](https://github.com/UOR-Foundation/uor-r4/pull/2170) restored. And the memory's host bridge costs **≈ 0.13 s/step** (1,000 steps: 63 s without it, ~190 s with it), small next to the trunk and the reason the *kernel* stays the honest follow-up.

## What the next cycle does, decided by this reading

The memory's **placement and size** at the anchor's steps — a **read** layer (`a` of `rrarrarrar`) and a larger sub-key set, one seed each against the same frozen panel and bar — then the brief's **exact-key arm** (prime/semiprime addressed store) with the tagger-versus-read split that this line's product-key configuration declared not applicable.

## Checklist

- [x] Checks at the exact head `846eb4e6cd596921108203386f7b4d261c89d250`: `python3 scripts/check_claim_wording.py` **pass** (docs-only: the record, a `current-state` entry, the README's two mechanism sections).
- [x] Research record: `docs/labs/addressed-memory-2026-10-10/README.md` (the control section appended to this mechanism's record)
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] README **How it works** and **Results so far** updated where the result changed (D22 §6)
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

