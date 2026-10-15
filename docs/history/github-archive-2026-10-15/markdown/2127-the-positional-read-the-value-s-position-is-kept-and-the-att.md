# #2127 — The positional read: the value's position is KEPT and the attention never lands on it (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:53:36Z
- **Updated:** 2026-10-10T03:23:38Z
- **Closed:** 2026-10-10T01:55:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2127
- **Merged:** 2026-10-10T01:55:20Z
- **Merge commit:** 6ebe4b89326b583514e834d86b5b949665ee12a7
- **Base / Head:** main / UOR-Foundation:pointer-positional-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Read on the CONTROL arm
Arm A — default knobs, byte-for-byte **13 of 13**, split **10/2/1** — so this is a statement about **the artifact**, not about a modified decoder. The treatment arm's unmeasured distribution is **not used**.

**Structural qualifier: this artifact has NO MEMORY READER — a pointer fix is a pointer fix.**

## The answer: (b), in a stronger form
At default knobs the pointer's `select` is `None`, which the code documents as keeping **every source** — so **the keep set is the whole window and the second digit's position is not excluded from it; the attention simply never places its mass there.**

| | rows | attention at that step |
|---|---:|---|
| argmax **ever** lands on the value's **FIRST** digit | **3 of 13** | 0.8533, 0.5141, 0.4743 |
| argmax **ever** lands on the value's **SECOND** digit | **0 of 13** | — |

**The second digit is never the argmax on any row**, and on the 10 READER rows **neither digit is attended at any step** — it is not that the first is attended while the second is dropped; **neither is attended.**

## The per-step evidence (verbatim, two READER rows)
`mem-005` (value `41`, ids `[22, 19]`, 14 steps): every step selects a window id that is **not** the value — `src=8:id223@0.378`, `5:1498@0.768`, `6:2369@0.652`, `5:1498@0.400`, `47:1156@0.454`, `8:223@0.920`, `43:2728@0.626`, `5:1498@0.323`, `45:1044@0.231`, `67:1790@0.321`, `51:754@0.465`, `5:1498@0.190`, `64:772@0.407`, `8:223@0.702`.

`mem-037` (value `98`, ids `[27, 26]`, 14 steps): the **same positions in the same order** — `8:223@0.351`, `5:1498@0.801`, `6:2369@0.712`, `5:1498@0.467`, `47:1156@0.432`, `8:223@0.906`, `43:2728@0.609`, `5:1498@0.380`, `45:1044@0.231`, `67:1790@0.348`, `51:754@0.463`, `5:1498@0.172`, `5:1498@0.418`, `8:223@0.706`.

## The finding the tables give for free: the pattern is nearly IDENTICAL across rows
Two rows with different values, keys and histories choose **the same source positions in the same order**, with attention tracking closely (0.3779 vs 0.3505; 0.7675 vs 0.8009; 0.9202 vs 0.9064). **The pointer's positional behaviour is essentially history-insensitive on this artifact: it is not failing to find the value, it is looking at a fixed pattern of positions and the value is not one of them.**

**It also explains the knob result:** `TopK(2)` + gate 0.9 moved coverage to **exactly 3 of 13** — precisely the rows where the argmax already landed on the first digit — because **raising the gate does not change where the pointer looks, only how much the copy distribution counts once it is there.**

## What it does not answer
Why the attention pattern is positional and row-invariant — whether the head was trained on targets that never required a two-token run, or whether the mask/window geometry penalises it. **That is the next read, not another knob.** No capability change is claimed: no judge was run, v5's `check_pass` was not re-measured.

## Ledger
Token count **REFUTED** · minimal pairs/digit order **REFUTED** · value addressability **REFUTED** · **the pointer's single-source shape REFUTED IN ITS SIMPLE FORM AND REPLACED** — single-source selection is not the binding constraint, the **attention's positional target** is · read/emit path **SPLIT**, localized to **attention placement rather than keep-set coverage**.

## Delivery
Record [`docs/labs/pointer-positional-read-2026-10-09/README.md`](docs/labs/pointer-positional-read-2026-10-09/README.md) with the full per-step tables; the bin now records **every step**, not just the audit subset; newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `pointer-positional-read-2026-10-09.tar`, 81,408 bytes, md5 `59bc0a05c6f9085c8e3a9da3c924a6cb`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch; scratch kept inside the worktree.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

