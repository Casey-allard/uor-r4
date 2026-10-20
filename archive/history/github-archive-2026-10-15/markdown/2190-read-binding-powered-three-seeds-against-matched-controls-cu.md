# #2190 — Read binding, powered: three seeds against matched controls cut wrong-value failures 47 -> 30 over 120 and lift memory 57 -> 65 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T05:50:02Z
- **Updated:** 2026-10-11T05:57:01Z
- **Closed:** 2026-10-11T05:54:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2190
- **Merged:** 2026-10-11T05:54:20Z
- **Merge commit:** ee2df21b3e4529b24374f08e1e8784f0864b1721
- **Base / Head:** main / UOR-Foundation:deepseek/read-binding-powered
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**D22 order 4, reopened with power — and the first KEEP on this line.** Supervised value binding, **from step 0** of the fine-tune, **three seeds** (data orders), each against a **matched W=0 control**, wrong-value failures **paired row by row**: bound mass **0.732 → 0.896 / 0.899 / 0.905**, competing mass **0.140 → 0.090 / 0.086 / 0.081**, **wrong-value failures 47 → 30 over 120 rows** (−5 with 9 repaired and 4 broken; **−11 with 12 repaired and 1 broken, p = 0.0034**; −1), **memory `check_pass` 57 → 65 of 120**, and every supervised arm's BPB **below** the anchor's. References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: read binding, powered (D22 order 4) · count 0/3, **KEEP resets the count** · headline before reply 30/232 + memory 20/40 → after **65/120 over three seeds against controls' 57/120**, wrong-value **30 against 47**

## Why this order, and why now

The [answer-path attribution](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6105227220) measured that **the read produces the answers** — removing the facts takes the memory panel to **0/40**, switching the copy head off costs **one row** — and this line's failures are *wrong values*. Read binding is the signal aimed exactly there, and its [brief](docs/mechanisms/read-binding.md) names what the last test lacked: *"one seed and a 40-row panel … it was bolted on late."*

## Design, pre-registered before compute

Supervised from **step 0** from the chat base, on the adopted `mix-10` store with its own label sidecar (`source=recall`), D10's recipe, `read_binding_supervision=0.1`; **three seeds = three data orders** (`data_seed` 20261009/10/11) and **three matched W=0 controls** at the same orders; one extra arm supervising the **first** read layer instead of the last as the head-attribution contrast. Bar: ≥ 5 fewer wrong-value failures than the matched control on ≥ 2 of 3 seeds, the third not worse, memory not below the control, BPB within 0.01.

## Results

| pair | data order | supervised memory | control memory | Δ | supervised WV | control WV | repaired | broken | **p** (exact McNemar) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| S1 vs Ctl1 | 20261009 | 19/40 | 20/40 | −1 | 12 | 17 | 9 | 4 | 0.2668 |
| **S2 vs Ctl2** | 20261010 | **24/40** | 20/40 | **+4** | **5** | 16 | **12** | **1** | **0.0034** |
| **S3 vs Ctl3** | 20261011 | **22/40** | 17/40 | **+5** | 13 | 14 | 6 | 5 | 1.0000 |
| **aggregate** | — | **65/120** | **57/120** | **+8** | **30** | **47** | **27** | **10** | — |

**Bar: MET** → **KEEP + iterate** (D22 §2), count resets. Float BPB (whole held-out stream, one lens): anchor 1.17425; **S1 1.17142, S2 1.17067, S3 1.17060** — all *below* the anchor; L2 (first read layer) 1.17772; the controls' own BPBs are still running and are named pending.

**The head attribution came out as the brief asked:** supervising the **first** read layer reaches bound mass **0.786** with **15** wrong-value failures at the seed where the **last** reaches **0.896** with **12** — the bound head that moves the answers is a late read layer.

## Declared limitations

- The bound/competing masses are **train-side** (`train_binding_*`); the report does not expose held-out ones.
- **Two of the three paired tests are not significant** (p = 0.27, 1.00); the significant one is the seed that repairs twelve and breaks one. The aggregate −17 of 120 is the honest summary.
- **One seed sits one row below its control** on the headline (19 vs 20, inside the ±3-row noise); the frozen v5 panel holds **40 memory rows** where the brief asks ≥ 120 (standing deviation, owner question).
- The head check is a **layer contrast**, not a serving-time ablation — the named next instrument.
- **Ctl3's official grading and the controls' BPBs are pending**; the paired test uses the judge-free row classification, complete for all six arms.

## Checklist

- [x] `python3 scripts/check_claim_wording.py` **pass** at this head.
- [x] Record `docs/labs/read-binding-powered-2026-10-11/README.md`; newest `current-state` entry; `STATUS.md` row and cell; README **How it works** and **Results so far** updated where the mechanism result changed (D22 §6).
- [x] Cost: one pod, seven fits ≈ 14 minutes of GPU (≈ .20), laptop CPU for the six panel sweeps and gradings.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

