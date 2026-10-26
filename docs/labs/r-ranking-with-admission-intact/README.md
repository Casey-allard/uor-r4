# Ranking With Admission Intact (`r-ranking-with-admission-intact`, `#2884`)

- **Issue / Roadmap Item:** [`#2884`](https://github.com/UOR-Foundation/uor-r4/issues/2884) (`docs/roadmap/items/r-ranking-with-admission-intact.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/m1-discrete-attention.md`)
- **Labs:** `antigravity` preparation (ORDER [`antigravity-68`](../../loop/tasks/r-ranking-with-admission-intact/antigravity-68/ORDER.md)); `kimi` diagnostic close (ORDER [`kimi-44`](../../loop/tasks/r-ranking-with-admission-intact/kimi-44/ORDER.md), decision [`kimi-43/DECISION.md`](../../loop/tasks/r-ranking-with-admission-intact/kimi-43/DECISION.md))
- **Pre-Registration Sheet:** [`docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md`](../../harness/prep/r-ranking-with-admission-intact-2026-10-25.md) (`TEST FITNESS: FIT`, [`#2884` comment `6396898060`](https://github.com/UOR-Foundation/uor-r4/issues/2884#issuecomment-6396898060)); its six-run full-dose ladder is **`NOT_RUN`** and remains unchanged history
- **Current Status:** `done` at the item's unchanged diagnostic done-when only (D68 review, [`#2884` comment `6424978255`](https://github.com/UOR-Foundation/uor-r4/issues/2884#issuecomment-6424978255)). Q1 ranking is below the same-admitted-row unigram control; no full-dose, independent-seed, fresh-transfer, above-unigram or M1 claim follows

---

## 1. Exact Admission-Intact Ranking Decomposition at `Q1` (`4,096` Batches, `#2831`)

Prior to `#2831` (`m1-a8-full-dose-rerun`, PR `#3192`), every non-collapsing discrete training run lost most of its target admission during training (`Rung-0` dropped from `5,680` to `1,920 / 14,287`; `K16-A` ended at `2,237` and `3,167 / 14,287`; ungated `A8` dropped from `1,351` to `551` and `857 / 3,847` on `P_succ`), whereas `K16-R` (`OccupiedCellRankHingeV1`) preserved nominal admission (`12,089` and `12,466 / 14,287`) only by collapsing onto `20` and `19` occupied cells ($\bar{n} \approx 3,440$ tokens/row).

In `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`, sealed RTX 5090 archive `a8-full-dose-rerun-20261025.tar.gz`, SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`), pairing the injective `PrimeZeta` + `AddressCellsA8` initialization (`256 / 256` cells occupied at `16` tokens/cell) with the active `D40` whole-interval admission-anchor controller (`floor = 1,187 / 3,847`) produced the project's **first checkpoint (`FrozenZero` at `Q1 = 4,096` batches, `epoch = 258`) with gate-relative admission and cell occupancy simultaneously intact**:

| Metric (`DEV` Panel, `14,287` rows; `P_succ = 3,847` rows) | `#2831` Init (`batch = 0`, `baseline-dev.json` `884b9a8f...`) | `#2831` `Q1` (`batch = 4,096`, `q1-dev.json` `6b924afa...`) | Delta / Ratio (`Q1` vs Init) | `K16-A` Seed 1 (`16,404` batches) | `K16-A` Seed 2 (`16,404` batches) | `Rung-0 Q4` (`16,404` batches, `#2845`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| **Occupied Cells (`of 256`)** | `256` | **`256`** | `0` (`100%` intact) | `209` | `192` | `<= 16` |
| **Mean Admitted Tokens / Row ($\bar{n}$)** | `128.00` (`1,828,736 / 14,287`) | **`126.49`** (`1,807,135 / 14,287`) | `-1.51` tokens/row | `351.8` | `376.1` | — |
| **TRAIN Anchor `b_3(P_succ)` (`floor = 1,187`)** | `1,211 / 3,847` (`31.48%`) | **`1,258 / 3,847`** (`32.70%`) | **`+47` rows** | — | — | — |
| **DEV Copy-Target Admission `b_3(P_succ)`** | `1,351 / 3,847` (`35.12%`) | **`1,106 / 3,847`** (`28.75%`) | `-245` (`+249` vs ungated `857`) | — | — | — |
| **DEV Complement Admission (`10,440` rows)** | `2,893 / 10,440` (`27.71%`) | **`3,186 / 10,440`** (`30.52%`) | **`+293` rows** | — | — | — |
| **DEV Full-Panel Admission $N_{\text{adm}} = b_3(\text{Full})$** | `4,244 / 14,287` (`29.71%`) | **`4,292 / 14,287`** (`30.04%`) | **`+48` rows** | `2,237` (`15.66%`) | `3,167` (`22.17%`) | `1,920` (`13.44%`) |
| **Full-Panel `DEV top-1` (`/ 14,287`)** | `133` (`0.93%`) | **`871` (`6.10%`)** | **`+738` (`6.55x`)** | `65` (`0.45%`) | `463` (`3.24%`) | `149` (`1.04%`) |
| **Strict-Win `DEV top-1` (`admitted_strict_win_rows`)** | `133` | **`679`** (`+192` tie-resolved) | **`+546` (`5.11x`)** | — | — | — |
| **Conditional `top-1` Given Admitted ($\text{top-1} / N_{\text{adm}}$)** | `133 / 4,244` (**`3.13%`**) | **`871 / 4,292` (`20.29%`)** | **`+17.16 pp` (`6.48x`)** | `65 / 2,237` (`2.91%`) | `463 / 3,167` (`14.62%`) | `149 / 1,920` (`7.76%`) |
| **Full-Panel `DEV top-8` (`/ 14,287`)** | `513` (`3.59%`) | **`1,455` (`10.18%`)** | **`+942` (`2.84x`)** | — | — | `747` (`5.23%`) |
| **Conditional `top-8` Given Admitted ($\text{top-8} / N_{\text{adm}}$)** | `513 / 4,244` (**`12.09%`**) | **`1,455 / 4,292` (`33.90%`)** | **`+21.81 pp` (`2.80x`)** | — | — | `747 / 1,920` (`38.91%`) |
| **Token Hinge Sum (`token_hinge_sum`)** | `768,188` | **`692,984`** | **`-75,204` (`-9.79%`)** | — | — | — |
| **Cell Rank Hinge Sum (`cell_rank_hinge_sum`)** | `880,738` | **`822,475`** | **`-58,263` (`-6.62%`)** | — | — | — |

---

## 2. Comparison Against Full-Panel and Cross-Checkpoint Count Tables

Combining the `#2831` `Q1` (`4,096`-batch) evaluation with the `#2845` (`m1-w2-capacity-capture`, `docs/labs/m1-w2-capacity-capture/README.md`) count-table benchmarks separates the admission ceiling from within-cell ranking. The `1,920`-row conditional columns below belong to Rung-0 Q4, **not** to Q1's `4,292` admitted rows; they are cross-checkpoint context, not an admission-matched Q1 result.

| Comparator / Model | Full-Panel `DEV top-1` (`/ 14,287`) | Full-Panel `DEV top-8` (`/ 14,287`) | Conditional `top-1` on Admitted Rows | Conditional `top-8` on Admitted Rows |
| :--- | ---: | ---: | ---: | ---: |
| **Full-Panel `T2` Query-Token Bigram (`#2845`)** | `6,300` (`44.10%`) | `11,263` (`78.83%`) | `1,482 / 1,920` (`77.19%` on Rung-0 `Q4` admitted) | `1,895 / 1,920` (`98.70%`) |
| **W2 Exact Read-Code Table (`#2845`)** | `3,414` (`23.90%`) | `7,558` (`52.90%`) | `1,138 / 1,920` (`59.27%` on Rung-0 `Q4` admitted) | `1,782 / 1,920` (`92.81%`) |
| **Full-Panel `Unigram` Count Table (`#2845`, CHARTER §4.8)** | `1,226` (`8.58%`) | `4,060` (`28.42%`) | `754 / 1,920` (`39.27%` on Rung-0 `Q4` admitted) | `1,586 / 1,920` (`82.60%`) |
| **Same-Q1-row unigram argmax control (host count of stored GPU rows)** | `1,226` (`8.58%`) | — | **`972 / 4,292` (`22.65%`)** | — |
| **`#2831` `FrozenZero` at `Q1` (`4,096` batches, `258` updates)** | **`871` (`6.10%`)** | **`1,455` (`10.18%`)** | **`871 / 4,292` (`20.29%` on `#2831` `Q1` admitted)** | **`1,455 / 4,292` (`33.90%`)** |
| **`K16-A` Seed 2 (`16,404` batches, `130` updates)** | `463` (`3.24%`) | — | `463 / 3,167` (`14.62%` on `K16-A` s2 admitted) | — |
| **`Rung-0 Q4` Trained Writer (`16,404` batches, `#2845`)** | `149` (`1.04%`) | `747` (`5.23%`) | `149 / 1,920` (`7.76%` on Rung-0 `Q4` admitted) | `747 / 1,920` (`38.91%`) |
| **`#2831` Untrained Init (`0` batches)** | `133` (`0.93%`) | `513` (`3.59%`) | `133 / 4,244` (`3.13%` on `#2831` Init admitted) | `513 / 4,244` (`12.09%`) |

### Key Diagnostic Findings (Measurement)

1. **The Q1 writer loses to the simplest same-row control:** host counting of the sealed Q1 GPU rows finds `972 / 4,292 = 22.65%` admitted targets equal to token 1, the full-panel unigram argmax. The writer's `871 / 4,292 = 20.29%` is `101` rows lower. Training improved from `133 / 4,244` at initialization, but this does not establish an advantage over the count table or isolate admission protection as the cause.
2. **Both channels remain active bottlenecks at `Q1`:**
   - **Admission loss (`9,995 / 14,287` rows = `69.96%`):** At $K = 8$ kept cells, `9,995` DEV targets lie outside the `8` opened drawers and are arithmetically unreachable regardless of within-cell ranking. Tying full-panel T2 requires at least `6,300 / 14,287 = 44.10%` admission even with a perfect ranker; at `4,292` admitted rows it would require an impossible `146.78%` conditional top-1. Tying Unigram requires `28.56%`.
   - **Within-cell ranking loss (`3,421 / 4,292` admitted rows = `79.71%`):** Among the `4,292` rows where the correct drawer *is* opened, the L1 token-distance writer ranks the target token first in `871` rows (`20.29%`) and in the top 8 in `1,455` rows (`33.90%`), missing `3,421` admitted targets (`2,837` of which are outside top-8 even within the admitted cells). The reported mean `126.49` is admitted tokens per **DEV row** (`1,807,135 / 14,287`), not per admitted-target row.

---

## 3. Full-Dose Ladder: `NOT_RUN`

In `#2831`, neither arm reached `Q4` (`16,404` batches): `Relation` stopped at batch `1,600` and `FrozenZero` at `5,248` on `consecutive_rejections`. The later Stage T runs did not qualify the pre-registered full-dose ladder: `T16_fz` stopped at `3,008`, `T16_rel` stopped at Q1, and the two seed bindings produced identical table/gate trajectories. They are supporting context, not independent two-seed or Q4 evidence.

The D67 pre-run review failed (`#2884` comment `6424741498`): `Arm-Rel-K16` is rejected by the A8-only validator and baseline, prior measured Relation throughput projects above the registered 35-minute gate, and the three-arm grading outputs are absent. No pod launched and no fresh evaluation occurred. The D68 diagnostic close (`#2884` comment `6424978255`) leaves the original FIT unchanged and the exact ladder unexecuted. Any future version needs a repaired design and implementation, a new sheet and fresh FIT/D67; the existing #2878 coverage-schedule item is only the related longer-training route.
