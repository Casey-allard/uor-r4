# Ranking With Admission Intact (`r-ranking-with-admission-intact`, `#2884`)

- **Issue / Roadmap Item:** [`#2884`](https://github.com/UOR-Foundation/uor-r4/issues/2884) (`docs/roadmap/items/r-ranking-with-admission-intact.md`)
- **Milestone / Area:** `research` / `training` (`docs/research/notebook/m1-discrete-attention.md`)
- **Lab:** `antigravity` (ORDER [`docs/loop/tasks/r-ranking-with-admission-intact/antigravity-68/ORDER.md`](../../loop/tasks/r-ranking-with-admission-intact/antigravity-68/ORDER.md))
- **Pre-Registration Sheet:** [`docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md`](../../harness/prep/r-ranking-with-admission-intact-2026-10-25.md) (`TEST FITNESS: FIT`, [`#2884` comment `6396898060`](https://github.com/UOR-Foundation/uor-r4/issues/2884#issuecomment-6396898060))
- **Current Status:** `held` (`status: ready -> status: held` via D68 decision review by `0d7e3ab3-18b0-4f86-9020-0fbbd27df5f4`; `#2831` `Q1` admission-intact ranking decomposition and `#2845` count-table comparison recorded; full `16,404`-batch 2-seed GPU run gated on `#2836` `m1-bvg-k-stage-t-training`)

---

## 1. Exact Admission-Intact Ranking Decomposition at `Q1` (`4,096` Batches, `#2831`)

Prior to `#2831` (`m1-a8-full-dose-rerun`, PR `#3192`), every non-collapsing discrete training run lost most of its target admission during training (`Rung-0` dropped from `5,680` to `1,920 / 14,287`; `K16-A` ended at `2,237` and `3,167 / 14,287`; ungated `A8` dropped from `1,351` to `551` and `857 / 3,847` on `P_succ`), whereas `K16-R` (`OccupiedCellRankHingeV1`) preserved nominal admission (`12,089` and `12,466 / 14,287`) only by collapsing onto `20` and `19` occupied cells ($\bar{n} \approx 3,440$ tokens/row).

In `#2831` (`docs/labs/m1-a8-full-dose-rerun/README.md`, sealed RTX 5090 archive `a8-full-dose-rerun-20261025.tar.gz`, SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`), pairing the injective `PrimeZeta` + `AddressCellsA8` initialization (`256 / 256` cells occupied at `16` tokens/cell) with the active `D40` whole-interval admission-anchor controller (`floor = 1,187 / 3,847`) produced the project's **first checkpoint (`FrozenZero` at `Q1 = 4,096` batches, `epoch = 258`) with admission and cell occupancy simultaneously intact**:

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

## 2. Comparison Against Full-Panel and Admission-Matched Count Tables

Combining the `#2831` `Q1` (`4,096`-batch) evaluation with the `#2845` (`m1-w2-capacity-capture`, `docs/labs/m1-w2-capacity-capture/README.md`) count-table benchmarks separates the admission ceiling from within-cell ranking:

| Comparator / Model | Full-Panel `DEV top-1` (`/ 14,287`) | Full-Panel `DEV top-8` (`/ 14,287`) | Conditional `top-1` on Admitted Rows | Conditional `top-8` on Admitted Rows |
| :--- | ---: | ---: | ---: | ---: |
| **Full-Panel `T2` Query-Token Bigram (`#2845`)** | `6,300` (`44.10%`) | `11,263` (`78.83%`) | `1,482 / 1,920` (`77.19%` on Rung-0 `Q4` admitted) | `1,895 / 1,920` (`98.70%`) |
| **W2 Exact Read-Code Table (`#2845`)** | `3,414` (`23.90%`) | `7,558` (`52.90%`) | `1,138 / 1,920` (`59.27%` on Rung-0 `Q4` admitted) | `1,782 / 1,920` (`92.81%`) |
| **Full-Panel `Unigram` Count Table (`#2845`, CHARTER §4.8)** | `1,226` (`8.58%`) | `4,060` (`28.42%`) | `754 / 1,920` (`39.27%` on Rung-0 `Q4` admitted) | `1,586 / 1,920` (`82.60%`) |
| **`#2831` `FrozenZero` at `Q1` (`4,096` batches, `258` updates)** | **`871` (`6.10%`)** | **`1,455` (`10.18%`)** | **`871 / 4,292` (`20.29%` on `#2831` `Q1` admitted)** | **`1,455 / 4,292` (`33.90%`)** |
| **`K16-A` Seed 2 (`16,404` batches, `130` updates)** | `463` (`3.24%`) | — | `463 / 3,167` (`14.62%` on `K16-A` s2 admitted) | — |
| **`Rung-0 Q4` Trained Writer (`16,404` batches, `#2845`)** | `149` (`1.04%`) | `747` (`5.23%`) | `149 / 1,920` (`7.76%` on Rung-0 `Q4` admitted) | `747 / 1,920` (`38.91%`) |
| **`#2831` Untrained Init (`0` batches)** | `133` (`0.93%`) | `513` (`3.59%`) | `133 / 4,244` (`3.13%` on `#2831` Init admitted) | `513 / 4,244` (`12.09%`) |

### Key Diagnostic Findings (Measurement)

1. **Holding admission intact immediately unlocks ranking learning:** At `Q1` (`4,096` batches, only `1/4` of a full `16,404`-batch dose and `258` committed coordinate updates), keeping $N_{\text{adm}} = 4,292 / 14,287$ and `occupied_cells = 256 / 256` lifted conditional within-cell `top-1` from `3.13%` (`133 / 4,244`) to **`20.29%` (`871 / 4,292`)**—already `2.61x` higher conditional accuracy and `5.85x` higher absolute `top-1` than `Rung-0 Q4` (`149 / 1,920 = 7.76%`), and `1.88x` higher absolute `top-1` than the best full-dose `K16-A` run (`463 / 3,167 = 14.62%`).
2. **Both channels remain active bottlenecks at `Q1`:**
   - **Admission loss (`9,995 / 14,287` rows = `69.96%`):** At $K = 8$ kept cells, `9,995` DEV targets lie outside the `8` opened drawers and are arithmetically unreachable regardless of within-cell ranking.
   - **Within-cell ranking loss (`3,421 / 4,292` admitted rows = `79.71%`):** Among the `4,292` rows where the correct drawer *is* opened ($\bar{n} = 126.49$ candidate tokens), the L1 token-distance writer ranks the target token first in `871` rows (`20.29%`) and in the top 8 in `1,455` rows (`33.90%`), missing `3,421` admitted targets (`2,837` of which are outside top-8 even within the admitted cells).

---

## 3. Why Full-Dose `16,404`-Batch Execution Is Held (`status: held`)

In `#2831`, neither arm reached `Q4` (`16,404` batches): `Relation` stopped at batch `1,600` (`epoch = 132`) and `FrozenZero` stopped at batch `5,248` (`epoch = 274`) on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`mechanism_negative: false`), caused by coupling `K = 1` single-coordinate greedy 64-row updates with `64`-batch whole-interval rollback. Running the pre-registered 3-arm (`Arm-FZ-K8`, `Arm-Rel-K8`, `Arm-Rel-K16`) $\times$ 2-seed (`seed in {1, 2}`) full `16,404`-batch experiment is therefore held (`status: held`) until `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) qualifies the `BVG-K` (`K = 16` batched-coordinate) full-dose trainer.
