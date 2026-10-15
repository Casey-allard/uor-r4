# #2200 — Measured tokens/s on GPU: the addressed read is 1.65x slower than the read it replaces (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T15:20:05Z
- **Updated:** 2026-10-11T15:28:45Z
- **Closed:** 2026-10-11T15:24:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2200
- **Merged:** 2026-10-11T15:24:19Z
- **Merge commit:** ac4de2f329be900ec93c2361e225c10221c2fae8
- **Base / Head:** main / UOR-Foundation:deepseek/tokens-per-second
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Measured the way the product is used — generated tokens per second, on the GPU, on a pod, three repeats — the addressed read is 1.65× slower than the read it replaces.** `Ctl1/Ctl2` (the softmax read) **636.6 / 637.7 / 638.4** and **637.4 / 635.8 / 636.9** ids/s against `L1/L2/L3` (product-key memory replacing the read) **389.7 / 388.1 / 386.8**, **383.8 / 385.5 / 385.3**, **384.9 / 383.8 / 384.3**. Training is 2.4× slower too (245 s against 104 s, same 2,000 steps). **The address's −90 % MACs at the replaced layer (−97 % at context 4,096) is correct arithmetic and invisible in tokens/s on this hardware** — the read is matmuls, the address is scoring plus a gather, and this configuration is not fused. References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: exact addressed memory at the read / metric direction · count 0/3 · headline before (analytic) −90 % MACs at the replaced layer → after (measured) **1.65× slower per token**

## The measurement

| arm | configuration | ids/s (repeats 1 / 2 / 3) | generated ids |
|---|---|---:|---:|
| **Ctl1** | read (control), seed 20261009 | 636.6 / 637.7 / 638.4 | 1,806 |
| **Ctl2** | read (control), seed 20261010 | 637.4 / 635.8 / 636.9 | 1,756 |
| **L1** | addressed read, seed 20261009 | 389.7 / 388.1 / 386.8 | 1,976 |
| **L2** | addressed read, seed 20261010 | 383.8 / 385.5 / 385.3 | 1,845 |
| **L3** | addressed read, seed 20261011 | 384.9 / 383.8 / 384.3 | 1,772 |

Three repeats per arm, same harness, same panel, same tokenizer (`d36d3e87…`), spread **≤ 0.5 %** within an arm — the ratio is stable, not a single-run artifact. **CPU-side rates were discarded as unusable**: two repeats of one arm on the shared laptop read 6.9 and 17.0 ids/s, a 2.5× spread that swamps the effect, which is why this is GPU-on-pod per your direction.

## What changes in this lab's reporting, from here

| | before | **now** |
|---|---|---|
| headline | BPB against the line's held-out stream | **measured tokens/s (GPU, pod, ≥ 3 repeats)** always beside the project's own **panel acceptance** |
| quality gate | BPB | **the panel**; BPB is a **diagnostic**, not a bar |
| cost ledger | MACs/token, parameters, wall | the same **plus the measured rate**, so an analytic saving that does not appear in time is visible as exactly that |

*(The project-wide criteria are yours to change — this is how this lab reports until told otherwise.)*

## What does not change, and I would rather state it than smooth it over

The addressed read still lost **7 memory rows of 120**, still costs **+0.10 BPB** as a diagnostic, and now also loses **1.65× on throughput** and **2.4× on training**. The only column it wins is the analytic MAC count — the column this measurement shows is not a proxy for time on GPU.

**The mechanism is not closed** (D22 §1). Its compute case belongs to the **native multiplier-free serving path** (D11), where a table read replaces multiplies and there is no matrix unit to waste; that is the next instrument, together with a long-context rate measurement where the read's cost grows with context (−97 % at 4,096) and the address's does not.

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass**; record, `current-state` entry, `STATUS.md` row.
- [x] Every number is from a report on the pod volume (`rate-gpu/`, `panel.cost.ids_per_second`), pulled and bundled; no figure is estimated.

**Limitations:** the f32 GPU harness is not the product's serving path (D11 is); context ≤ 384 for the panel's replies, where the read is cheapest; one pod, one GPU model, three repeats.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

