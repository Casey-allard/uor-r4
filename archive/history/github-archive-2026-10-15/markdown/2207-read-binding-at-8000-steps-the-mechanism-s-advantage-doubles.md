# #2207 — Read binding at 8000 steps: the mechanism's advantage doubles and the paired test turns significant (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T18:40:42Z
- **Updated:** 2026-10-11T18:49:30Z
- **Closed:** 2026-10-11T18:44:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2207
- **Merged:** 2026-10-11T18:44:58Z
- **Merge commit:** bcf8da7064047426632bd11db5d78e35e8e888f1
- **Base / Head:** main / UOR-Foundation:deepseek/read-binding-scaled
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The necessary next run was not another mechanism — it was the same signal trained four times longer, so the comparison is mechanism-versus-no-mechanism at equal budget.** At 8,000 steps read binding's advantage **doubles**: wrong-value failures **49 → 23 over 120 rows (−53 %)**, memory **48 → 64 of 120 (+16)**, every seed positive, **two of three individually significant** (p = 0.031, 0.035; **Fisher χ² = 18.4** on 6 df), 36 rows repaired against 10 broken. Its own objective sharpens with the budget too: bound mass **0.969–0.971** against 0.896–0.905 and competing mass **0.026–0.027** against 0.081–0.090. **KEEP + iterate.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: read binding, scaled (D22 order 4 iteration) · count 0/3, **KEEP** · headline before (2,000-step keeper) memory 65/120, wrong-value 30 against controls' 57/30 → after (8,000 steps) memory **64/120**, wrong-value **23** against controls' **48/49**

## The pod, used the way it should be

One 2 × RTX 4090 pod, **both GPUs at 100 % for the whole batch** (six 8,000-step fits at 573–582 s, three per GPU), inputs checked on the volume *before* the run, and **released and deleted at the last fit — $0/h since, ≈ $1.90 total**. A first attempt lost 4 minutes to `CUDA_ERROR_INVALID_PTX` (the bootstrap had built an `sm_120` binary for 4090s); it was fixed by re-bootstrapping for the pod's own arch and clearing the stale report roots, and that is recorded in the record rather than hidden.

## Results

| pair | data order | read binding WV | control WV | repaired | broken | **p** | read binding memory | control memory |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| R1 vs K1 | 20261009 | **8** | 15 | 10 | 3 | 0.0923 | **20/40** | 16/40 |
| **R2 vs K2** | 20261010 | **10** | 20 | 14 | 4 | **0.0309** | **20/40** | 13/40 |
| **R3 vs K3** | 20261011 | **5** | 14 | 12 | 3 | **0.0352** | **24/40** | 19/40 |
| **aggregate** | — | **23** | **49** | **36** | **10** | **Fisher χ² = 18.4 (6 df)** | **64/120** | **48/120** |

| run | bound mass | competing mass | dev NLL | memory /120 | wrong-value /120 |
|---|---|---|---:|---:|---:|
| keeper, 2,000 steps | 0.896 / 0.899 / 0.905 | 0.090 / 0.086 / 0.081 | 0.363 / 0.394 / 0.309 | **65** | **30** |
| **this run, 8,000 steps** | **0.9689 / 0.9702 / 0.9706** | **0.0268 / 0.0256 / 0.0257** | **0.240 / 0.235 / 0.325** | **64** | **23** |
| controls, 8,000 steps | — | — | 0.266 / 0.302 / 0.288 | 48 | 49 |

**What it establishes.** (1) **The mechanism scales with budget and its advantage grows**: at 2,000 steps it bought +8 memory rows over its controls and −17 wrong-value failures; at 8,000 steps, **+16 rows and −26 failures** — value increasing with training rather than saturating. (2) **The paired test turns significant**: two of three seeds individually, χ² = 18.4 combined, against one seed at 2,000 steps. (3) **The controls get *worse* as they train longer without the signal** (memory 57 → 48, wrong-value 30 → 49), which is the clearest statement of what the supervision does. And unlike the [competing-mass margin](https://github.com/UOR-Foundation/uor-r4/pull/2197) — where the masses improved and the panel did not — here the masses improve *because* the same signal trains longer, and the answers follow.

**Stated plainly:** the arms' absolute memory is **64/120 against the 2,000-step keeper's 65/120 — one row under, inside the ±3-row noise** — while the controls fell to 48, so the comparison that carries the mechanism is **+8 → +16 rows of 120**.

## Checks

- [x] `python3 scripts/check_claim_wording.py` **pass**; record, `current-state` entry, `STATUS.md` row.
- [x] Matched controls re-run at the same budget in the same session (no dependence on a model that might not survive); the paired test is row-by-row.

**Declared:** three seeds on a 40-row panel; **BPB not reported** — a diagnostic under the owner's metric direction, and this pod's volume did not carry the stream (development NLLs stand in); the reply panel is graded separately; nothing about serving changed, so no token-rate claim is made here.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

