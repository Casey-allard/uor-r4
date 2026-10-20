# #2189 — Which path produces the answer: the read does, the copy head does not (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T03:46:20Z
- **Updated:** 2026-10-11T03:52:39Z
- **Closed:** 2026-10-11T03:50:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2189
- **Merged:** 2026-10-11T03:50:18Z
- **Merge commit:** fd7246cec16db9dac16325551093f6cfd5f1150e
- **Base / Head:** main / UOR-Foundation:deepseek/answer-path-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The question this line never asked before building on either path: which mechanism produces the answer token? Answered in four serving-time conditions on one artifact, one frozen panel, no training:** removing the facts takes the memory panel to **0/40** (all 40 rows fail with a *missing* value); switching the copy head off costs **one row** (23 → 22, inside noise) while removing ~6 % of the mixture; forcing the copy to half the mixture costs **sixteen rows** (23 → 7). **The answers come from the context through the read. The copy head does not produce them.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: answer-path attribution (pointer brief, D22 order 3) · count 0/3 · headline before reply 30/232 + memory 23/40 → after: **copy off 22/40**, **facts removed 0/40**

| # | condition | how | memory | wrong-value | no/other |  |
|---|---|---|---:|---:|---:|---:|
| A | normal (copy at its learned ~0.06 gate) | — | **23/40** | 9 | 8 | **17** |
| B | **copy head off** | `pointer_gate_ceiling=0.0` | **22/40** | 7 | 11 | 9 |
| C | **facts removed** (question turn only) | `panel-qonly.json` | **0/40** | 0 | **40** | 2 |
| D | copy forced to half the mixture | `pointer_gate_floor=0.5` | 7/40 | 21 | 5 | 1 |
| E | copy forced to all of it | `pointer_gate_floor=1.0` | 9/40 | 18 | 4 | 4 |

**What each reading establishes.**

1. **The answers come from the context through the read (C).** With the facts gone, every memory row fails and every failure is a missing value — the model is not reciting from pretraining, and grounding on this panel is real. Its carrier is the **softmax read**.
2. **The copy head is not the answer path (B).** One row, inside the ±3-row noise, for 6 % of the mixture. The gate is not suppressing a correct answer. The judge column shows what the copy does buy: `acceptable` **17 → 9** with the copy off — **local fluency, not the value**.
3. **Forcing the copy on is destructive because it displaces the reader (D, E)** — a branch worth ~1 row cannot carry 50 % of the output without pushing out the branch worth 23.
4. **The pointer line's metrics and the panel were measuring different things.** The order-3 result raised the pointer's hit rate **0.377 → 0.54** across three seeds at zero likelihood cost with the panel flat; A–C say why. Those results stand as read-outs of a ~6 % branch (`dev_pointer_mean_gate` 0.056–0.064) and cannot be evidence about the answers.

**What follows.** The next mechanism on this line must act on the **read** — 16.6 % of the model's MACs at context 384 and **40.5 % at 4,096**, so it has both a target (it is the answer path) and a compute argument (its cost grows with context; a lookup's does not) — or the output path must be rebuilt so a copy *can* be the answer. Enabling the existing copy head cannot do it, and the oracle says so directly.

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass** at this head; docs-only (one record, one `current-state` entry, one `STATUS.md` row and cell).
- [x] The two interventions are **serving-time read-out changes recorded in each `replies.json`** — no parameter, no gradient, no training path. Instruments: [#2187](https://github.com/UOR-Foundation/uor-r4/pull/2187) (gate floor), [#2188](https://github.com/UOR-Foundation/uor-r4/pull/2188) (gate ceiling).
- [x] No pod, no lease, no external cost: laptop CPU only.

**Limitation stated up front:** one artifact, one seed, 40 memory rows; condition C removes the facts rather than masking them, so it measures dependence on the prompt, not the read's internals — a span-mask ablation is the named next instrument.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

