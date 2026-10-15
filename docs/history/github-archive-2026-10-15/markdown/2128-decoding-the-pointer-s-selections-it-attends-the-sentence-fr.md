# #2128 — Decoding the pointer's selections: it attends the sentence FRAME and never the varying slot (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:59:17Z
- **Updated:** 2026-10-10T03:23:37Z
- **Closed:** 2026-10-10T02:00:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2128
- **Merged:** 2026-10-10T02:00:56Z
- **Merge commit:** cbb9698e4ec97057935a64b760fd674e532056b4
- **Base / Head:** main / UOR-Foundation:pointer-ids-decode
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## 0. The index check first
**Vocabulary 4096, max id 4095, every selected id in range (223 … 2728).** The trace's indexing is sound; the last three pieces' window readings are **not** in question.

Read on the **control arm**. CPU only, no pod, no training, **no knob**, $0. No v5 re-run. **Structural qualifier: this artifact has NO MEMORY READER.**

## 1. The decode
| id | token | |
|---:|---|---|
| **223** | **`'Ġ'`** | a **bare space** — and it carries the **highest attention in both tables (0.9202, 0.9064)** |
| 1498 | `'Ġbrother'` | content subword |
| 2369 | `'Ġtakes'` | content |
| 1156 | `'Ġbus'` | content |
| 2728 | `'ser'` | content |
| 1044 | `'ĠWh'` | content |
| 754 | `'Ġtake'` | content |
| 772 | `'ĠC'` | content |
| 1790 | `'ass'` | content |

**The value as control:** `mem-005` `41` → `22 → '4'`, `19 → '1'`; `mem-037` `98` → `27 → '9'`, `26 → '8'`. **The value is a run of SINGLE-DIGIT tokens** — the whole reason it is a run. **Structural candidates are never selected:** `<|bos|>` (0), `<|eos|>` (1), `.` (16), `:` (28) appear in **no** step.

## 2. The answer: neither "structural" nor "arbitrary" — it is the FRAME, and the varying slot is skipped
The selected ids are **content subwords, not role markers**, so the structural answer is not what the data gives. But they are not arbitrary either: they are the **recurring sentence frame** of the panel's construction, because **the two rows compared share the key `brother|brother's`** and the panel builds rows on a shared frame per key.

**This corrects my own previous piece, and the record says so: "history-insensitive" was overstated.** The invariance I measured is **the frame's**, not a demonstration that the head ignores history. **The correction strengthens the finding:** the two rows differ in **exactly one respect — the value (`41` vs `98`)** — and the pointer attends the frame they share and **never the slot that differs.** The highest-attention selection in both tables is the **bare space token**.

## 3. Against the pre-registered options
**STRUCTURAL — NO** (content subwords; structural candidates never selected). **CONTENT AT A FIXED OFFSET — PARTLY, and it is the frame's offset**, since the positions recur because the frame recurs. **NEITHER/arbitrary — NO**, the recurrence is explained by the shared frame. **Ids out of range / not decoding — NO.**

**The implication is the owner-level one, not a knob:** the head attends the sentence frame and never the varying slot — a **training-target** property — and it is **not** addressed by `TopK`, the gate, or a wider keep set (the previous piece's knob moved coverage only where the argmax already landed on a digit).

## 4. What it does not settle
It does **not** prove the head was trained on frame-only targets — it measures attention on 13 rows of one artifact; the training-target reading is the natural explanation and is **not** measured here. Nothing about the reply half or the addressed-memory path. **No capability change is claimed:** no judge was run, v5's `check_pass` was not re-measured.

## 5. Ledger
Token count **REFUTED** · minimal pairs/digit order **REFUTED** · value addressability **REFUTED** · single-source shape **REFUTED in its simple form and replaced** · **"history-insensitive" CORRECTED — the invariance is the frame's** · **"the pointer attends the frame and never the varying slot" MEASURED on 13 rows** — the smallest true statement this line has reached.

## Delivery
Record [`docs/labs/pointer-selected-ids-decode-2026-10-09/README.md`](docs/labs/pointer-selected-ids-decode-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store before any local deletion: `pointer-selected-ids-decode-2026-10-09.tar`, 81,408 bytes, md5 `a33917da9a2a09553c076ed58a218cff`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch; scratch inside the worktree.

**Next is a training decision for the owner**, not another measurement and not a knob: a pre-registered training piece with a **fresh sealed panel** (a retrained artifact cannot be compared to v5's 10 of 40) and a **timed calibration run before any estimate**.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

