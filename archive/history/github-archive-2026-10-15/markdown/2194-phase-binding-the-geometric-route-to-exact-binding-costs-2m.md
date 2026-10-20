# #2194 — Phase binding: the geometric route to exact binding costs 2M parameters and 13.4x the training time and reads worse (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T07:29:11Z
- **Updated:** 2026-10-11T07:35:55Z
- **Closed:** 2026-10-11T07:32:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2194
- **Merged:** 2026-10-11T07:32:36Z
- **Merge commit:** c9a1921c47b267283cf1983813abb50837fcab9c
- **Base / Head:** main / UOR-Foundation:deepseek/phase-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**D22 §2 says a KEEP is iterated, and the read-binding brief names the iteration: *"Phase binding (Step 7d/9, #820; `ReadLineage`) is the geometric route to the same goal: values carry a fixed phase or lineage, so binding becomes exact rather than learned similarity."* Trained on top of the keeper at the same seeds, store and recipe — so the lineage flag is the only difference — it comes out as a measured negative: **+2.0 M parameters (28.96 → 30.97 M), 13.4× the wall time per training step (0.058 → 0.777 s), identical bound/competing mass, and a worse panel on both matched pairs** (memory 19 → **17** and 24 → **20**; wrong-value 12 → **14** and 5 → **14**). References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: read binding, iterated (D22 order 4) · count 0/3 · headline before memory 19/24 (keeper, matched seeds) → after **17/20** with wrong-value 12/5 → **14/14**

## Quality, paired against the keeper

| pair | data order | phase binding | keeper | Δ memory | phase binding WV | keeper WV | Δ WV |
|---|---|---:|---:|---:|---:|---:|---:|
| **P1 vs S1** | 20261009 | 17/40 | 19/40 | −2 | 14 | 12 | +2 |
| **P2 vs S2** | 20261010 | 20/40 | **24/40** | **−4** | 14 | **5** | **+9** |

**Bar NOT MET — worse on both pairs on the failure mode it was meant to improve.** The mechanism's own objective is unchanged: bound mass 0.898 / 0.897 against 0.896 / 0.899; competing 0.088 / 0.088 against 0.090 / 0.086. **REJECT this configuration; the keeper (read binding alone) stands; the mechanism stays open** (D22 §1).

## The compute ledger — the first one reported in this lab

| | keeper | **+ phase binding** | change |
|---|---:|---:|---:|
| parameters | 28,958,761 | **30,968,449** | **+2.0 M** (read 4.0 M → 6.0 M) |
| **wall per training step** | **0.058 s** | **0.777 s** | **13.4×** |
| memory, matched seeds | 19, 24 | 17, 20 | worse |
| wrong-value | 12, 5 | 14, 14 | worse |
| bound / competing | 0.896/0.090, 0.899/0.086 | 0.898/0.088, 0.897/0.088 | unchanged |

**Why the 13.4×:** the phase-binding read has no fused CUDA kernel, so its step runs the exact CPU forward on host copies — the stack's rule for kernel-less configurations. That is a cost of this implementation, not of the mathematics; a kernel could remove it and the quality result would stand.

**Under a quality-only metric this trade would have been invisible.** Reporting the ledger beside the quality number is the practice the [geometry audit](docs/integration/geometry-audit-2026-10-11/README.md) asks the project to adopt, started here.

## Checks

- [x] \`python3 scripts/check_claim_wording.py\` **pass** at this head; record + \`current-state\` entry + \`STATUS.md\` row + README mechanism bullet.
- [x] No new code: the arms use existing flags (\`read_lineage=phase_bind\`, \`read_binding_supervision=0.1\`).

**Declared deviations:** the pre-registration specified **three** seeds; **two** were run because the bar needs ≥ 2 of 3 to pass and both pairs were worse, so a third could not change the decision. Frozen panel is 40 memory rows (±3, standing deviation). \`phase_bind_free\`, \`rot\` and \`carrier\` are untried variants. No served-cost measurement, so the 13.4× is a training-time cost.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

