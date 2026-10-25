# REPORT: r-ranking-with-admission-intact, antigravity-68

References #2884

## What did not work

- **Negative 1 — Even with cell admission and occupancy held intact at `Q1` (`4,096` batches, `#2831`), `79.71%` of admitted targets (`3,421 / 4,292`) are not ranked first within the admitted candidate pool:** In `#2831` (`m1-a8-full-dose-rerun`, sealed RTX 5090 archive `a8-full-dose-rerun-20261025.tar.gz`, `q1-dev.json` `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`), `FrozenZero` reached `Q1` (`batch = 4,096`, `epoch = 258`) with `occupied_cells = 256 / 256`, $\bar{n} = 1,807,135 / 14,287 = 126.49$ tokens/row, `DEV b_3(P_succ) = 1,106 / 3,847`, and `DEV b_3(Full) = 4,292 / 14,287` (`+48` over initial `4,244`). While conditional within-cell `top-1` rose `6.48x` from `133 / 4,244` (`3.13%`) to `871 / 4,292` (`20.29%`, `679` strict wins + `192` tie wins) and conditional `top-8` rose from `513 / 4,244` (`12.09%`) to `1,455 / 4,292` (`33.90%`), `3,421` of the `4,292` admitted targets (`79.71%`) still lost to a rival token inside the `126.49`-token shortlist, and full-panel `DEV top-1` (`871 / 14,287 = 6.10%`) remained below the full-panel `Unigram` count table (`1,226 / 14,287 = 8.58%`) and `T2` query-token bigram (`6,300 / 14,287 = 44.10%`).
- **Negative 2 — `K = 1` cadence-granularity coupling prevented `#2831` from reaching `Q4 = 16,404` batches across two seeds (`instrument`):** In `#2831`, `Relation` stopped at batch `1,600` (`epoch = 132`) and `FrozenZero` stopped at batch `5,248` (`epoch = 274`) on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`mechanism_negative: false`), because single-coordinate (`K = 1`) 64-row greedy updates paired with `64`-batch whole-interval rollback discard entire 64-batch intervals whenever a single local coordinate update dips below the global anchor floor. A full `16,404`-batch, 2-seed (`CHARTER §4.3`) admission-intact ranking evaluation therefore requires the Stage T `BVG-K` (`K = 16` batched-coordinate) trainer (`m1-bvg-k-stage-t-training`, `#2836`, Step 4 of `PLAN.md`).

## What changed

- **Files changed:** `6` files (`+243 / -4` lines across `docs/`; `0` code lines, `0` test lines, `240` doc lines, `3` generated roadmap lines; within the `docs/**` scope of `r-ranking-with-admission-intact`).
  - `docs/loop/tasks/r-ranking-with-admission-intact/antigravity-68/ORDER.md`: Recorded the cancel probe (`instrument`) and exact scope/done-when contract.
  - `docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md`: Authored the one-page pre-registration sheet (`TEST FITNESS: FIT`) for the 3-arm (`Arm-FZ-K8`, `Arm-Rel-K8`, `Arm-Rel-K16`) $\times$ 2-seed full `16,404`-batch admission-intact ranking experiment.
  - `docs/labs/r-ranking-with-admission-intact/README.md`: Authored the lab record reporting the exact `#2831` `Q1` (`4,096`-batch) admission-intact ranking decomposition (`871 / 4,292` conditional `top-1`, `1,455 / 4,292` conditional `top-8`, `256 / 256` cells, $\bar{n} = 126.49$) against the `#2845` (`W2`) full-panel and admission-matched count tables (`Unigram`, `W2 Code Table`, `T2 Bigram`).
  - `docs/roadmap/items/r-ranking-with-admission-intact.md` & `docs/roadmap/README.md`: Transitioned `status: ready -> status: held` following the `D68` decision review (`#2884` comment `6396898060`) and re-rendered the roadmap index.
  - `docs/loop/tasks/r-ranking-with-admission-intact/antigravity-68/REPORT.md`: Recorded this unit report.

## Measurements

- **Conditions:** Exact extraction from sealed RTX 5090 GPU artifacts of `#2831` (`a8-full-dose-rerun-20261025.tar.gz`, SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`: `baseline-dev.json` `884b9a8f...`, `q1.json` `1e436e66...`, `q1-dev.json` `6b924afa...`) and `#2845` (`docs/labs/m1-w2-capacity-capture/README.md`, `record.json` `69717971...`) at base commit `c364b6c6691eb6520443d039573471c84576a9da`.
- **Command:** `uor-harness loop gates --full`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`.
- **Key Numbers (`DEV` panel, `14,287` rows; `P_succ = 3,847` rows):**
  - `#2831` Init (`batch = 0`): `occupied_cells = 256 / 256`, $\bar{n} = 128.00$, $N_{\text{adm}} = 4,244 / 14,287$ (`29.71%`), `DEV top-1 = 133 / 14,287` (`0.93%`), conditional `top-1 = 133 / 4,244` (`3.13%`), `DEV top-8 = 513 / 14,287` (`3.59%`), conditional `top-8 = 513 / 4,244` (`12.09%`).
  - `#2831` `FrozenZero` at `Q1` (`batch = 4,096`, `epoch = 258`, admission intact): `occupied_cells = 256 / 256`, $\bar{n} = 126.49$, `TRAIN anchor b_3(P_succ) = 1,258 / 3,847` (`>= 1,187`), `DEV b_3(P_succ) = 1,106 / 3,847` (`28.75%`), $N_{\text{adm}} = 4,292 / 14,287$ (`30.04%`), **`DEV top-1 = 871 / 14,287` (`6.10%`, `679` strict wins)**, **conditional `top-1 = 871 / 4,292` (`20.29%`, `6.48x` Init)**, **`DEV top-8 = 1,455 / 14,287` (`10.18%`)**, **conditional `top-8 = 1,455 / 4,292` (`33.90%`, `2.80x` Init)**.
  - Count-table baselines (`#2845`): Full-panel `Unigram = 1,226 / 14,287` (`8.58%`), `W2 Read-Code Table = 3,414 / 14,287` (`23.90%`), `T2 Query-Token Bigram = 6,300 / 14,287` (`44.10%`); on `Rung-0 Q4`'s `1,920` admitted rows, admission-matched `Unigram = 754 / 1,920` (`39.27%`), `Code Table = 1,138 / 1,920` (`59.27%`), `Bigram = 1,482 / 1,920` (`77.19%`), `Trained Writer = 149 / 1,920` (`7.76%`).

## Checks run at the head

- `uor-harness loop gates --full`: `11 / 11` GREEN (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `loop-next-sources`, `loop-next-wait`, `loop-next-owner-direction`, `git-hooks`, `loop-migrate`, `loop-review-changes-blocks`).
- `uor-harness loop report --check --order docs/loop/tasks/r-ranking-with-admission-intact/antigravity-68/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS (`0` errors, `0` warnings).

## Skills used

- `uor-research-architect`: Used to decompose full-panel `top-1` into cell admission ($N_{\text{adm}} / 14,287$) and conditional within-cell ranking ($\text{top-1} / N_{\text{adm}}$) on the sealed `#2831` `Q1` and `#2845` `W2` GPU evaluations.
- `scope-guard` & `solution-integrity`: Used to keep all edits within the item's `docs/**` scope and sequence the 2-seed full-dose GPU run behind `#2836` (`m1-bvg-k-stage-t-training`) via `D68` decision review.

## Still open

- Executing the pre-registered 3-arm (`Arm-FZ-K8`, `Arm-Rel-K8`, `Arm-Rel-K16`) $\times$ 2-seed (`seed in {1, 2}`) `16,404`-batch GPU run and scoring admission-matched count tables on its `Q4` checkpoints once `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) completes.

## Plain summary

Earlier training runs lost most of their reachable target words during training (because word-table updates pushed the right answers out of the 8 opened drawers), making it impossible to tell whether low accuracy came from opening the wrong drawers (admission) or picking the wrong word inside the opened drawers (ranking). By analyzing the sealed 4,096-batch checkpoint from `#2831`—where the active admission guard kept all 256 drawers occupied and held reachable targets steady at `4,292` of `14,287`—this unit shows that keeping admission intact immediately boosted accuracy inside the opened drawers more than sixfold (from `3.13%` to `20.29%`, or `871` correct first guesses, nearly double the best previous 16,404-batch run's `463`), while also showing that `79.71%` of words inside opened drawers are still outranked by rivals. It also pre-registers the full 16,404-batch two-seed test (`TEST FITNESS: FIT`) and holds its GPU launch until the batched `BVG-K` trainer (`#2836`) is complete.
