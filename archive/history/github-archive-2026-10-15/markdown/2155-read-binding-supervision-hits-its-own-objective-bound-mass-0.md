# #2155 — Read-binding supervision hits its own objective (bound mass 0.73 -> 0.90) and leaves the frozen panel flat (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T11:46:28Z
- **Updated:** 2026-10-10T11:50:36Z
- **Closed:** 2026-10-10T11:48:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2155
- **Merged:** 2026-10-10T11:48:14Z
- **Merge commit:** 61a0dcce60ed0569db1da985670d28a782a684f4
- **Base / Head:** main / UOR-Foundation:deepseek/read-binding-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: `read_binding_supervision` (the `ReadSupervisionGroup bound/competing` objective D21 named and the last cycle did not test) reaches its own target — binding-head mass on the asked value's positions **0.73 → 0.90** (W = 0.1) and **0.93** (W = 0.5), competing mass halved — and leaves the frozen panel flat-to-down: v5 memory **20/40 (anchor) → 19/40 → 17/40**, with the targeted failure mode moving instead (**wrong-value rows 17 → 12**, nine repaired and four broken). The reply guard is clean (39/232, p = 0.62 vs the base). Bar not met → **REJECT**, count **2/3**.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer-target supervised fine-tune · count 2/3 · headline before reply 37/232 `acceptable` + memory 20/40 `check_pass` → after reply **39/232** + memory **19/40** (W = 0.1; W = 0.5 gives 17/40)

## What and why

**Cycle 2** of the standing goal. Cycle 1's frontier showed the memory deficit is not a read that misses the value but a read that binds the wrong one: of the 40 v5 rows, the 15 that both new doses fail are **every one a wrong-value failure** — the reply names the distractor's value (`expected barbara → "Your kitten is named Leon."`). This cycle trains the objective whose labels say exactly that — the asked value's history positions as `bound`, the other stated values as `competing` — on cycle 1's **unchanged 10 % mixture**, pre-registered on #2029 before any compute.

## Result

| arm | `read_binding_supervision` | binding NLL (train) | bound / competing mass | **v5 memory `check_pass`** | unknowable | **wrong-value failures** | reply `fluent_and_relevant` |
|---|---:|---:|---:|---:|---:|---:|---:|
| D10 (cycle-1 anchor, same mixture) | 0.0 | — | — | **20/40** | 0/24 | 17 | 37/232 |
| **B1 (primary)** | **0.1** | 0.673 → **0.183** | 0.732 → **0.896** / 0.140 → 0.090 | **19/40** | 5/24 | **12** | **39/232** |
| B5 | 0.5 | 0.695 → **0.132** | 0.739 → **0.934** / 0.150 → 0.056 | **17/40** | 4/24 | 14 | not scored (guard covers the primary) |

Row-paired against the anchor: **nine wrong-value rows repaired** (`mem-002`, `-014`, `-018`, `-022`, `-027`, `-030`, `-032`, `-034`, `-040`), **four correct rows broken** (`-007`, `-010`, `-013`, `-028`). Reply panel: **39/232, statistically indistinguishable from the base's 43 (20/16, p = 0.62) and from the anchor's 37 (14/16, p = 0.86)**; `heldout_first_turn` 35 against the base's 34 is the best of any fine-tuned artifact on this panel.

**Decision: REJECT.** The pre-registered bar needs memory ≥ 21/40 **and** reply ≥ 29/232 at once; B1 is at 19/40 and B5 at 17/40, so the bar cannot be met. Count **2/3** under the rule stated on #2029 before the numbers existed (both halves must be at least at their last-cycle levels; memory is not) — **one more merged PR on this line without a moved headline makes 3/3 and requires a pivot card**. Criterion 1 remains **NOT MET** on both halves (19/40 against ≥ 34/40; 39/232 against ≥ 116/232); the base artifact's 10/40 and 43/232 are unchanged.

**What it establishes:** the objective is aimed correctly and is not enough — the failure mode the panel is made of *is* reachable by a training objective (9 rows repaired) and this shape of it trades modes rather than reducing them (4 broken). With the copy gate (#2145) also flat on the panel, **the data mixture remains the only lever this line has moved** (cycle 1: 10/40 → 20/40).

## Checklist

- [x] Checks at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched — the trainer and grader are unchanged `main` builds, and the round's checks are the panel's own: frozen row checks, derangement controls, the byte-identity of the regenerated label store, and paired exact McNemar on the reply panel).
- [x] Research record: `docs/labs/read-binding-2026-10-10/README.md`
- [x] Newest-first `docs/integration/current-state.md` entry with **Next:**
- [x] `STATUS.md` row and M1 cell; #2028 M1 table posted after the merge
- [x] Evidence bundle: cloud-store `read-binding-2026-10-10`, **2,241,536 bytes, md5 `04172efbba05bdfadf60e9848c3a8b28`**
- [ ] After merge: verify on fresh `origin/main`, delete branch and worktree, `scripts/storage/uor-hygiene --apply`, final card on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6097161773 — 2026-10-10T11:46:30Z — Casey-allard

**Validation at the exact head `f39d3b037543876ccd842e99b1e89e5abc530c3d`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` row and cell; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "no unlinked machine-verified/exact-equivalence wording" |
| label store byte-identity | `sha256sum` of the regenerated vs cycle-1 store | `tokens.u16` **`28f7bacccd099b18…`** and `response_mask.u8` **`444eaae67eb4ef98…`** identical — the only difference from the anchor is the objective |
| objective reached | arm reports' curves | binding NLL 0.673 → **0.183** (W = 0.1) and 0.695 → **0.132** (W = 0.5); bound mass 0.732 → **0.896** / 0.739 → **0.934**; competing mass 0.140 → 0.090 / 0.150 → 0.056 |
| v5 acceptance | `chat-grade grade-replies`, frozen `conversational-v5-checks.tsv` | memory **19/40** (B1) and **17/40** (B5) against the anchor's 20/40; unknowable 5/24 and 4/24; derangement controls 1 and 2 of 64 |
| failure causes | judge-free classification of the same replies against the same frozen checks | wrong-value **17 → 12 → 14**; distractor-key 0 → 2 → 3; no/other 3 → 7 → 6 |
| reply guard (primary) | `chat-grade grade-replies`, 232 rows, cap 64 | **39/232** `fluent_and_relevant`; base vs B1 20/16, **p = 0.62**; anchor vs B1 14/16, p = 0.86; derangement 4 |

Evidence bundle: cloud-store object `read-binding-2026-10-10`, **2,241,536 bytes, md5 `04172efbba05bdfadf60e9848c3a8b28`**.



---
_Comment/review ids in this file: 6097161773_

