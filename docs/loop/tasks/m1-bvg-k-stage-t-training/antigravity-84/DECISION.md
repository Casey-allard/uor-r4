# DECISION (D68): Stage T (`m1-bvg-k-stage-t-training`, `#2836`) Runs Now After `#2834`, with Unmet W4 Routing Dropped

Lab: `antigravity`. Unit: `antigravity-84`. Item: [`m1-bvg-k-stage-t-training`](../../../../roadmap/items/m1-bvg-k-stage-t-training.md) (`#2836`, Milestone M1 `#2029`). Base: `d5453ca30` (merged forward to `origin/main` `71fc67415`).

## 1. The Question

In `claude-107` ([`docs/loop/tasks/m1-bvg-k-stage-t-training/claude-107/DECISION.md`](../claude-107/DECISION.md)), Stage T was sequenced behind `m1-address-key-full-vector` (`#2834`) with two explicit conditions (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:21-28`, `claude-107/DECISION.md:32-34`):
1. *"Stage T stays on the M1 close (D79). When it runs, its decision review weighs `#2834`'s result first."*
2. *"The sheet routes a partial outcome to `m1-w4-float-credit-sheet` (`#2846`). `PLAN.md` runs W4 only if W2 gives `H_law` and W3 fails (`PLAN.md:58`), and W2 did not give `H_law`. Whoever runs Stage T should drop or re-justify that routing."*
3. *"Instrument work owed before any arm runs: the D40-audited runner is `K = 1` and seed 1 only, and needs a block evidence mode (a `K = 16` read block can straddle the Combine/Relation gates); seed-2 A8 configs; the sheet's five Step 0 controls; the checkpoint-restricted W2 tables."*

Now that `m1-address-key-full-vector` (`#2834`) is `done` (`claude-110`, PR `#3235`, `d5453ca30`), how does `#2834`'s result weigh on Stage T (`#2836`), should Stage T run now on the qualified `A8-Outer` lineage after completing the four instrument items, and how should the Stage T pre-registration sheet's outcome routing be updated?

## 2. The Decision

1. **Stage T (`#2836`) runs now on the qualified `A8-Outer` lineage (`address-structured-v1.json`, `sha256 = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`)**, immediately after building and verifying the four owed instrument items in `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs`.
2. **The Stage T pre-registration sheet (`docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`, matching `#2836`'s `scope:` glob `docs/harness/prep/m1-bvg-k-stage-t-training*`) drops the routing of partial or negative outcomes to `m1-w4-float-credit-sheet` (`#2846`)**, because W2 resolved `H_mixed` (`T = 3,414 <= M + F`), so `PLAN.md:58,93`'s `H_law` precondition for `W4` is not met. Instead, if Stage T returns a partial or configuration-negative outcome, next work routes to registering training and serving under `#2834`'s full-vector cell-envelope metric (`FV8-Env`) and to `m1-wider-code-carrier` (`#2847`).

## 3. Why (Whole-Project Context & Weighing `#2834`)

1. **What `#2834` (`claude-110`, `docs/loop/tasks/m1-address-key-full-vector/claude-110/REPORT.md:85-100,126-129`) established — and what it explicitly leaves to writer learning:**
   - In `#2834`, the zero-update full-vector cell-envelope metric (`FV8-Env`, `cell_envelope_l1_v1` on `FullVectorV1` table `2c886736…`) passed the Stage 1 admission bar: `P_succ` admitted rose from `1,351 / 3,847` (`A8-Outer`) to `1,980 / 3,847`, full-DEV admitted rose from `4,244 / 14,287` to `5,709 / 14,287`, and `top-8` rose from `513` to `1,126 / 14,287`.
   - At the same time, **checkpoint-0 `top-1` fell from `133` (`A8-Outer`) to `118 / 14,287` (`FV8-Env` and `FV8-TwoScale`)** (`claude-110/REPORT.md:8-11,89-91`): *"The writer now admits more target cells, but more of its picks among them are wrong."*
   - Furthermore, **training and D11 serving under a non-outer address metric are refused in code** (`claude-110/REPORT.md:37-39,48,127`): `CorpusUpdateAdmission::guarded` (`crates/uor-r4-discrete/src/training.rs:2978`) and the D11 exporter (`crates/uor-r4-discrete/src/export.rs`) enforce `zero_update_only(driver.config.address_metric)` (`AddressMetric::OuterL1V1` only).
   - Thus `#2834` proves two complementary facts:
     - (a) For closing M1 against the full-panel `T2 = 6,300 / 14,287` bar (`CHARTER.md §4.8`), full-vector address placement + metric (`#2834`) and/or wider carrier (`#2847`) are needed to lift ceiling reach toward `6,300+`.
     - (b) Increasing cell admission alone (`4,244 -> 5,709`) does **not** improve `top-1` (`133 -> 118`) without a writer learning law that ranks admitted candidates inside the shortlist.

2. **Why `A8-Outer` already has `~3x` the admission needed to test the Stage 2 bar (`#2836`):**
   - `#2836`'s done-when is the **Stage 2 bar** (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:32`): *"beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds."*
   - On the qualified `A8-Outer` base codebook (`address-structured-v1.json`, `6600bdb1…`), `4,244 / 14,287` DEV rows at checkpoint 0 (`4,292 / 14,287` at `#2831` `Q1`) already admit the target cell into the 8-cell shortlist (`128` candidates).
   - That `4,244–4,292` admitted ceiling is **more than `3.4x` the unigram bar (`U = 1,226 / 14,287`)** and **`2.8x–3.7x` the W2 baseline admission-matched tables (`T_restr(Q4) = 1,138 / 14,287` and `G_restr(Q4) = 1,482 / 14,287`)** (`docs/labs/m1-w2-code-utilization/README.md:31-38`).
   - In `#2831` (`m1-a8-full-dose-rerun`, `docs/loop/tasks/m1-a8-full-dose-rerun/antigravity-57/REPORT.md:7-9,33-42`), `A8-Outer` (`FrozenZeroV1`) already climbed `6.55x` from `133` to **`871 / 14,287`** at `Q1` (`batch = 4,096`) while keeping DEV admission at `4,292 / 14,287`. What stopped `#2831` before `Q4` was not shortlist reach (`4,292 >> 1,482`), but `consecutive_rejections` under `K = 1` single-coordinate 64-row greedy updates coupled with 64-batch whole-interval rollback (`AuditPolicy::registered()`).
   - Simultaneously, W2 (`docs/labs/m1-w2-capacity-capture/README.md:58-61`) measured `989` rows where the target is admitted by the writer and predicted by the read-code table `T` (`1,138` rows on `A_W`) yet missed by the `K = 1`-trained Q4 output map (`149` rows on `A_W`).
   - Testing `BVG-K` (`K = 16` coordinate block scoring + joint verify + atomic block commit) on `A8-Outer` is a clean, unconfounded test of whether `16x` broader per-batch coordinate coverage and verified multi-coordinate commits overcome `#2831`'s `K = 1` rollback stop (`871 / 14,287`) and clear the Stage 2 bars (`U = 1,226`, `T_restr`, `G_restr`) on two seeds.

3. **Why `m1-w4-float-credit-sheet` (`#2846`) routing is dropped from the Stage T sheet:**
   - `PLAN.md:58,93` states: *"W4 is one float-credit sheet, run only if W2 gives H_law and W3 fails (conditional, D37)."*
   - In `#2845` (`docs/labs/m1-w2-capacity-capture/README.md:27-29,54`) and `#3179` (`docs/labs/m1-w2-code-utilization/README.md:57`), W2 resolved `H_mixed` (`T = 3,414 <= M + F = 6,435`), ruling out `H_law`.
   - Therefore `claude-107/DECISION.md:34` is exact: the `H_law` precondition for `#2846` has failed, so `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md` removes `#2846` from the `PARTIAL_UNIGRAM_ONLY` and `DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS / H_TIE` outcome rows and routes them instead to registering training on `#2834`'s `FV8-Env` lineage and to `m1-wider-code-carrier` (`#2847`).

4. **All dependencies of `#2836` in `PLAN.md` and `docs/roadmap/items/m1-bvg-k-stage-t-training.md:9` are now `done`, and `#2836` is the top blocker on M1 (`unblocks 12`):**
   - `m1-w2-capacity-capture` (`#2845`): `done`
   - `m1-bvg-k-block-apply-c2` (`#2835`): `done`
   - `m1-a8-full-dose-rerun` (`#2831`): `done`
   - `m1-w2-code-utilization` (`#3179`): `done`
   - `m1-address-key-full-vector` (`#2834`): `done`
   - Under `CHARTER.md §5.1` (D67, D68) and `PLAN.md:61` (D79), `#2836` is Step 4 of the M1 close path and unblocks 12 downstream items (`#2839`, `#2841`, `#2837`, `#2844`, and M1–M8).

## 4. Rejected Alternatives

- **Defer Stage T again until full-vector-metric training (`FV8-Env`) is registered and qualified:**
  - Rejected because: (a) `#2834` is `done` and `FV8-Env` training is not an existing item on the M1 close path (`PLAN.md:56-65`); (b) `A8-Outer` already admits `4,244–4,292 / 14,287` DEV rows (`~3x` the Stage 2 bars `1,226` and `1,138 / 1,482`), so Stage 2 (`#2836`) is not bottlenecked on `A8-Outer`'s admission ceiling; and (c) `#2831`'s `K = 1` seed-1 baseline (`871 / 14,287` at `Q1`, `ca5f9469…` / `eb02da3b…`) is on `A8-Outer`, so running `T16` on `A8-Outer` preserves the exact single-variable paired contrast (`K = 16` vs `K = 1`).
- **Keep the `m1-w4-float-credit-sheet` (`#2846`) fallback routing in the Stage T sheet:**
  - Rejected because `PLAN.md:58,93` conditions `W4` on `H_law`, which W2 (`#2845` / `#3179`) ruled out (`claude-107/DECISION.md:34`).
- **Run Stage T without first completing the four instrument items (`docs/roadmap/items/m1-bvg-k-stage-t-training.md:24-28`):**
  - Rejected under `CHARTER.md §4.2` (*"Instrument first"*): the block-evidence D40 controller (`K = 16` straddling read block `16..32`), seed-2 configs, the five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`), and the checkpoint-restricted W2 table evaluator (`T_restr(A_W)` and `G_restr(A_W)`) must all be built and verified before any Stage T training metric is read.

## 5. Cost to Reverse

Reversible by git revert of the sheet/instrument commits or by re-running under a future full-vector-metric training sheet; no existing `crates/uor-r4-discrete` behavior or `driver_source_sha256` pin is altered.

## 6. Review Record (D68, D67, and `TEST FITNESS: FIT`)

- **Reviewer:** `antigravity` fresh-context non-author reviewer (`f96e37ea-4794-4e07-b3f4-2ad7b21a5f3c`) at `6d19043f746a6e2824c6363fa72ed13eca211469`
- **Verdicts:**
  - `TEST FITNESS: FIT` on `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`
  - `D68 DECISION REVIEW: PASS`
  - `D67 PRE-RUN REVIEW: PASS`
- **Owner Log Receipt:** [`#2836` comment `6401853483`](https://github.com/UOR-Foundation/uor-r4/issues/2836#issuecomment-6401853483)
