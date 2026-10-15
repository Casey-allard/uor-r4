# #2114 — Why the numeric memory rows fail: the value is not delivered, and it is not confusion (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T00:36:16Z
- **Updated:** 2026-10-10T00:40:47Z
- **Closed:** 2026-10-10T00:38:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2114
- **Merged:** 2026-10-10T00:38:02Z
- **Merge commit:** cc61324ec0cbe5dbae2896148caf90701475e259
- **Base / Head:** main / UOR-Foundation:v5-numeric-diagnosis
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

Diagnosis of the v5 declared run's clustered failure. **No model run, no v5 re-run, CPU only, no pod, $0.** Every number recomputed from the sealed v5 replies, the frozen checks file and the frozen tokenizer.

## The ledger, at the strength each claim actually earned

**MEASURED — the vocabulary has no multi-digit token.** From the frozen tokenizer `d36d3e87…`: **4096 tokens, 10 pure single-digit tokens (`0`–`9`), 0 multi-digit tokens, 3 superscript digit slots (`¹ ² ³`)**. Three of thirteen digit slots are idle while no multi-digit token exists — reclaimable budget for any number-aware tokenizer.

**MEASURED — numeric rows fail far more often.**

| row type | rows | failed | rate |
|---|---:|---:|---:|
| numeric value | 13 | **12** | **92.3%** |
| word value | 27 | **18** | **66.7%** |

`expected_value` passes **40 of 40**, so the check is not the obstacle.

**MEASURED CONTROL, REPLICATED — within-key comparisons hold context, kind and history fixed:**

| key | n | values | type | pass |
|---|---:|---|---|---:|
| `brother\|brother's` | 6 | 41 68 71 54 98 84 | numeric | **1/6** |
| `grandpa\|grandpa's` | 3 | Tyler Phil Luis | **word** | **2/3** |
| `piano\|piano's` | 3 | 75 89 71 | numeric | **0/3** |
| `treehouse\|treehouse's` | 2 | 85 99 | numeric | **0/2** |

The same key carries opposite outcomes by value type — which kills "the brother rows are just harder". Numeric totals 1 of 11 by two independent computations.

**REFUTED — "token count explains the cluster."** Numbers tokenize in **2 tokens** against **3.74** for words, so length runs the wrong way.

**REFUTED — "minimal pairs / digit order."** `mem-040`'s planted distractor `74` is a **transposition** of its expected `84` — the most confusable pair constructible — and it **passes**; `mem-037` (`98` vs `76`, no shared digit position) fails.

## The decisive cut: the failing numeric replies do not emit a value at all

| what a failing numeric row emits | rows |
|---|---:|
| **no digit anywhere** | **10** |
| a digit, but not the stored value | **2** |
| names the planted distractor | **0** |
| **total** | **12** |

**All 8 planted-distractor confusions are WORD rows.** The one passing numeric row emits the stored value exactly: `mem-040` → *"Your sister's locker number is 84."* The two that emit a digit both answer *"Your sister's locker number is 1."* for stored values `68` and `54`. And the expected value was in the context for **40 of 40 rows (13 of 13 numeric)** — this is not context loss.

Fluent, correctly framed sentences **with the value slot empty or filled by something never stored**: the value does not arrive.

## The six keyless rows: not a panel defect

`mem-011/012/013/027/028/029`, **all 3-turn**, four word and **two numeric**. `expected_value` passes all six — none is unpassable. They fail for their value type's reason (2 emit no number, 4 name no value). **The no-key set overlaps the numeric set (2 of 6)** — do not add 1-of-11 and 0-of-6 as disjoint causes. The clean statement: **numeric 92.3% against word 66.7%**, keyless splitting 4 word / 2 numeric.

## Magnitude vs type

Not magnitude — every numeric value on this panel is **two-digit**, so there is no multi-digit contrast to measure. Not position — the value is in an earlier turn for all 40 rows, and the 3-turn keyless rows split by type, not length. **It is the value type, and within it the deficit is emission, not selection.**

## Two named options, neither started

1. **A number-aware tokenizer** giving a value one token and therefore **one address** — now the *direct* fix rather than the cheap one; the three idle superscript slots and the absent multi-digit tokens are its budget.
2. **An explicit numeric slot in the memory path** — the more native version, addressing a value *as a value*.

The `mem-040` specimen decides which to try first, and either gets its own pre-registered piece on a **fresh sealed panel**. **v5 is not re-run** — it is development evidence now and its 10 of 40 stands.

## Delivery

Record [`docs/labs/v5-numeric-diagnosis-2026-10-09/README.md`](docs/labs/v5-numeric-diagnosis-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on `cloud-store` before any local deletion: `v5-numeric-diagnosis-2026-10-09.tar`, 25,600 bytes, md5 `1b7b37d5c039e5412653228067c47428`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

