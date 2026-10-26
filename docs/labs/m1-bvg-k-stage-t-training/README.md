# M1 BVG-K Stage T Batched-Coordinate Training (`m1-bvg-k-stage-t-training`, `#2836`)

- **Date**: 2026-10-25
- **Lab**: `antigravity` (`antigravity-84`, PR `#3239`)
- **Milestone**: M1 (`#2029`), Step 4 in [`PLAN.md`](../../../PLAN.md)
- **Roadmap Item**: [`docs/roadmap/items/m1-bvg-k-stage-t-training.md`](../../roadmap/items/m1-bvg-k-stage-t-training.md) (`#2836`)
- **Pre-Registration Sheet**: [`docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md) (`sha256 = 30dcf0f0b69231531c45e3ac67f62f9de42c88307f5fb7027d0e7cf4b50ef4e7`)
- **Reviews**:
  - `TEST FITNESS: FIT` + `D68 DECISION REVIEW: PASS` + `D67 PRE-RUN REVIEW: PASS` by fresh-context reviewer `f96e37ea-4794-4e07-b3f4-2ad7b21a5f3c` ([`#2836` comment `6401853483`](https://github.com/UOR-Foundation/uor-r4/issues/2836#issuecomment-6401853483); [`DECISION.md`](../../loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/DECISION.md))
- **GPU Pod & Cost**: `vast-57710022` (`2x NVIDIA GeForce RTX 5090`, `$2.171/h`, leased via `scripts/pod/uor-pod`, `~5.5h` total wall time $\approx \$11.94$, within the registered `~$12` estimate and `$15.00` cap)
- **HF Store Archive**: `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/` (with `SHA256SUMS`)

---

## 1. Executive Summary & Stage 2 Verdict

We built the four Stage T instrument components required by [`docs/roadmap/items/m1-bvg-k-stage-t-training.md`](../../roadmap/items/m1-bvg-k-stage-t-training.md) (`crates/uor-r4-discrete/src/stage_t.rs`, `crates/uor-r4-discrete/src/bin/stage-t-run.rs`, and the 8 Seed-1/Seed-2 `DriverConfig` manifests in [`configs/`](configs/)), verified all five mandatory Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d` in [`step0-receipt.json`](step0-receipt.json), `5 / 5 PASS`), and executed the two-seed `BVG-K` (`K = 16`) Stage T runs (`T16_fz` and `T16_rel`, Seeds `1` and `2`) plus Step 3 checkpoint-restricted W2 count-table evaluations (`1,049,856` TRAIN rows and `14,287` DEV rows per arm-seed) on `2x RTX 5090`.

- **Registered Stage 2 Verdict ([`grade.json`](grade.json))**: **`DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS`** (`stage2_bar_met_two_seeds = false`, `mechanism_negative = false`, `fresh_evaluations = 0`).
- **What `BVG-K` (`K = 16`) improved over `K = 1`**:
  1. **Admission expansion (`G-Adm` PASS on `T16_fz` across both seeds)**: On `T16_fz`, `K = 16` block learning under active D40 64-batch rollback raised full-DEV target cell admission from `4,244 / 14,287` (`29.71%`) at checkpoint 0 (and `4,292 / 14,287` at `#2831` `K = 1` Q1) to **`5,009 / 14,287` (`35.06%`, `+765` rows over baseline, `+717` rows over `#2831` Q1)** and `P_succ` admission to **`1,216 / 3,847` (`>= 676` PASS)** across `420` committed block updates (`854` committed coordinate moves, `123` multi-coordinate joint commits), peaking at **`1,823 / 3,847`** (`+612` over baseline `1,211`) on the TRAIN copy-successor anchor (`1,849 / 3,847` peak on `T16_rel`).
  2. **Read-code richness & checkpoint-restricted W2 tables (`T_restr(A_W)` and `G_restr(A_W)` beat `U = 1,226` and baseline `W2` tables on both seeds)**:
     - On `T16_fz` (`epoch = 420`, `854` moves), distinct 16-bit read codes rose from `14,412` (`K = 1` Q4 in `#2845`) to **`19,385`** on TRAIN (`+4,973`) and from `3,713` to **`4,067`** on DEV (`+354`). On `T16_rel` (`epoch = 678`, `1,505` moves including `425` relation gate moves), distinct read codes rose to **`27,132`** on TRAIN and **`4,429`** on DEV.
     - Restricted to the checkpoint's admitted cells $A_W$, the exact read-code count table **`T_restr(A_W)`** scores **`2,140 / 14,287` (`14.98%`) top-1** and **`4,006 / 14,287` (`28.04%`) top-8** on `T16_fz` (`1,602` top-1 / `3,007` top-8 on `T16_rel`), and the admission-restricted bigram **`G_restr(A_W)`** scores **`3,084 / 14,287` (`21.59%`) top-1** and **`4,674 / 14,287` (`32.72%`) top-8** on `T16_fz` (`2,364` top-1 / `3,529` top-8 on `T16_rel`) — nearly doubling `#3179`'s `K = 1` Q4 baseline tables (`T_restr(Q4) = 1,138`, `G_restr(Q4) = 1,482`) and clearing `U = 1,226` by `+914` to `+1,858` rows.
  3. **Relation-gate learning gain on `T16_rel`**: Whereas `#2831` `K = 1` `Relation` (`T1_rel`) stopped at batch `1,600` (`132` updates, `22` relation moves, DEV `top-1 = 133`), `T16_rel` ran all `4,096` batches to `Q1` (`678` committed block updates, `1,505` committed moves = `545` vector + `535` combine + **`425` relation moves**, `199` joint commits), lifting DEV `top-1` `2.79x` to **`371 / 14,287`** (`top-8 = 1,124`, strict wins `172`), with the `relation_off` ablation confirming that zeroing the `425` learned relation moves drops DEV admission by `74` rows (`3,827 -> 3,753`), `top-1` by `44` rows (`371 -> 327`), `top-8` by `61` rows (`1,124 -> 1,063`), and strict wins by `49` rows (`172 -> 123`).
- **Why the Stage 2 writer `top-1` bar was not met (`455` on `T16_fz`, `371` on `T16_rel` vs `U = 1,226`)**:
  - While `T16_fz` raised DEV `top-8` to **`1,484 / 14,287`** (`+29` over `#2831` Q1's `1,455`) and admitted `5,009 / 14,287` target cells (`T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`), the parameterless `OuterL1V1` `NearestAddressMarginV1` writer (`(8*T + w*A)/(8*w)`) only moves the `4,096 x 32` token embedding table and `65` read gates; it has no separate within-cell output table or full-vector tie-breaker. Moving shared token vectors to improve `854` coordinate hinges across `K = 16` blocks pulled more target cells into the 8-cell outer-L1 shortlist (`5,009` rows) and increased `top-8` (`1,484`), but left within-cell rank-1 token selection at `455 / 14,287` (`9.08%` of the `5,009` admitted rows, vs `42.72%` for `T_restr(A_W)` and `61.57%` for `G_restr(A_W)` on the exact same `5,009` admitted cells) before hitting `consecutive_full_rejections = 3` at batch `3,008`.

---

## 2. Code, Config, and Artifact SHA-256 Pins

| Artifact / Pin | Path | SHA-256 |
|---|---|---|
| Compiled `uor-r4-discrete` `source_sha256` | `crates/uor-r4-discrete` (`cec8dbeea`) | `be02d3eff523d73bde5a5e15533cd0a1e6e65eb596416fa85f21184e8fad6be8` |
| Compiled CUDA `kernel_sha256` | `discrete.ptx` (`sm_120`, CUDA 12.8) | `0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6` |
| Compiled `geometry_sha256` | `uor-r4-core` geometry tables | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Unchanged `driver_source_sha256` | `crates/uor-r4-discrete/src/training.rs` | `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` |
| Pre-registration sheet | `docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md` | `30dcf0f0b69231531c45e3ac67f62f9de42c88307f5fb7027d0e7cf4b50ef4e7` |
| TRAIN copy-successor anchor manifest | `anchor-manifest.json` (`3,847` rows, `manifest_sha256 = f85466ff...`) | `885789a71420751660c69c82ec01df7c6bfc285dc4a87e28648e1ead050e909b` |
| `T16_fz` Seed 1 config | [`configs/t16-frozen-zero-seed1.json`](configs/t16-frozen-zero-seed1.json) | `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d` |
| `T16_fz` Seed 2 config | [`configs/t16-frozen-zero-seed2.json`](configs/t16-frozen-zero-seed2.json) | `d169d4ef347f28f4a289d131c3664e56a40a62a29774bf87cf0313ab358592cf` |
| `T16_rel` Seed 1 config | [`configs/t16-relation-seed1.json`](configs/t16-relation-seed1.json) | `ab41788a5e72a1f11370f875b090653e0d1b36be091393861b0e147688e9d746` |
| `T16_rel` Seed 2 config | [`configs/t16-relation-seed2.json`](configs/t16-relation-seed2.json) | `a0bfa6c5fe7497d85d2a3d8a68d86157a8dcaa72a5d5b7eca193d904c480f263` |
| `T1_fz` Seed 1 / Seed 2 configs | [`configs/t1-frozen-zero-seed1.json`](configs/t1-frozen-zero-seed1.json) / [`seed2`](configs/t1-frozen-zero-seed2.json) | `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479` / `461ecd65da5318e036ed16e038425fcd794d7ea5d5213e6413a8040b48e2e70c` |
| `T1_rel` Seed 1 / Seed 2 configs | [`configs/t1-relation-seed1.json`](configs/t1-relation-seed1.json) / [`seed2`](configs/t1-relation-seed2.json) | `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e` / `dd8942a2e210bd3a411ee4d4c6fa685edd5ecaa8596bd8e36489fdddc534abd8` |
| Step 0 GPU controls receipt | [`step0-receipt.json`](step0-receipt.json) | `0ca7d95568adf216ba3f2abda16fbcbe3fc73bd72b96c7d0665c694000d5be50` |
| `T16_fz` Seed 1 `report.json` & `w2-summary.json` | HF store & [`t16-frozen-zero-seed1/w2-summary.json`](t16-frozen-zero-seed1/w2-summary.json) | `127d9044a9d83eb4d889762cfa7e6488f4368b364dd04bf46d83ea51af767325` / `19009681fc7f81cd3af581b77b10e418d57d1437eb07f7ecd139cfcd5187f22c` |
| `T16_fz` Seed 2 `report.json` & `w2-summary.json` | HF store & [`t16-frozen-zero-seed2/w2-summary.json`](t16-frozen-zero-seed2/w2-summary.json) | `28c0d58e90ba306cc413dc2e2f2b1d0877cacdbe711f2cf2c9a51846b67cb1d7` / `d2067572cf746b8b2585f170548ca02c253dc51c8373339594d1825afc50c6e0` |
| `T16_rel` Seed 1 `report.json` & `w2-summary.json` | HF store & [`t16-relation-seed1/w2-summary.json`](t16-relation-seed1/w2-summary.json) | `756e76db88b3dce7a2ca42580e97f07244b444e35c0ebb16a0556cb4888afde5` / `565e0aa10c6ec93fdaa8b70e510f23dfd612d2602826ff9176b3986861089e10` |
| `T16_rel` Seed 2 `report.json` & `w2-summary.json` | HF store & [`t16-relation-seed2/w2-summary.json`](t16-relation-seed2/w2-summary.json) | `c0bf457d3267378efe64a99dfa4217b514ad3fb4da04210372fe7ed1681c1326` / `b7f6e7e76835e091e55304ea4a503ac8b4a522700a0bc67ec29e7289451ffe62` |
| Stage 2 two-seed grade receipt | [`grade.json`](grade.json) | `7917ec85941d919db14ab67567fe824c87d6d2edf18bbd54138bf320b6ea4c81` |
| HF store archive (`stage-t-20261025.tar.gz`) | `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz` | `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6` |

---

## 3. Step 0 GPU Instrument Controls (`step0-receipt.json` — `5 / 5 PASS`)

Before any training metric was read, `stage-t-run step0` verified the three Stage C2 uncovered items (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md` §"What this does not prove") and driver-level invariants on `vast-57710022`:

| Control | Scope & Invariant Checked | Result | Verdict |
|---|---|---|---|
| **`AKd`** | Across `32` single-move commits (`\|commit_ordinals\| == 1`), `apply_block` (`bvgk.cu`) vs ordinary `K = 1` `Device::apply` (`discrete.cu`) on a shadow device from the pre-commit snapshot | `32` commits checked, `0` mismatches (`table`, `routes`, `gates`, `epoch`, and `model_snapshot_sha256` bitwise identical) | **PASS** |
| **`V1d`** | Across the first `24` production A8 batches (`Vector`, `Combine`, straddle `16..32`, `Relation`), `transition_block(K = 1)` vs `transition(K = 1)` | `24` batches checked, `0` mismatches, final snapshot `5de50c024f5507a2805aeda6942d78effc8b5d12396d298d7deea368cc13bdad` | **PASS** |
| **`KId`** | Across `128` `T16_fz` batches (plus every committed batch of all 4 training runs), `model.epoch() == accepted_updates`, `committed_moves == sum(moves)`, fresh device `score_rows_k` post-commit loss equals `committed_loss < l_before`, and all-ineligible `Relation` blocks (`32..45`) are exact no-ops | `128` step-0 batches + `14,208` training batches checked, `39` step-0 commits rescored, `kid_ineligible_noop_verified = true`, `0` mismatches | **PASS** |
| **`V8d`** | On `T16_fz` (`FrozenZeroV1`), `0` relation ordinals committed (`Gate(20..65)`) and all `45` relation trits equal `0` at every batch and 64-batch audit | `128` step-0 batches + `3,008` training batches per seed checked, `0` relation commits, `0` non-zero relation trits | **PASS** |
| **`V9d`** | Restoring the batch-`64` checkpoint of `T16_fz` and running batches `64..128` matches an uninterrupted `128`-batch run bitwise | Uninterrupted & resumed batch-`128` snapshots both equal `3545fe65c5c35f748dc6f9115d30b3fd5ec415c1253a63a4cd2f47466585d12d` (`epoch = 39`, `committed_moves = 79`) | **PASS** |

- **Step 0 Throughput Probe**: `128` `T16_fz` batches (`8,192` rows $\times$ `192` context $= 1,572,864$ context tokens) completed in `27,519 ms` (`215.0 ms/batch`, **`57,155.6 context tok/s`**; projected `16,404`-batch training wall time `3,526 s`).

---

## 4. Two-Seed Stage T Training Measurements (`T16_fz` and `T16_rel` vs `T1` Controls)

Because `CodebookInit::PinnedTableV1` (`address-structured-v1.json`) and `ReadInitialization::ActiveLeafV1` initialize `table` and `gates` deterministically and `ReadSpec::Relation` never reads the seeded `routes` array (`discrete.cu:421`), Seed 1 and Seed 2 follow the exact same integer trajectory on `table` and `gates` while carrying distinct `seed` and `routes` bindings (and therefore distinct `model_snapshot_sha256` and checkpoint digests).

| Metric | `T16_fz` Seed 1 | `T16_fz` Seed 2 | `T16_rel` Seed 1 | `T16_rel` Seed 2 | `#2831` `T1_fz` (`K=1` S1) | `#2831` `T1_rel` (`K=1` S1) |
|---|---:|---:|---:|---:|---:|---:|
| **Block size $K$ / Policy** | `16` / `FrozenZeroV1` | `16` / `FrozenZeroV1` | `16` / `LearnV1` | `16` / `LearnV1` | `1` / `FrozenZeroV1` | `1` / `LearnV1` |
| **Attempted batches** | `3,008` | `3,008` | `4,096` (`Q1`) | `4,096` (`Q1`) | `5,248` (`Q1` at `4,096`) | `1,600` |
| **Terminal status** | `DIAG_STOP_CONSEC_REJ` | `DIAG_STOP_CONSEC_REJ` | `DIAG_STOP_Q1` | `DIAG_STOP_Q1` | `STOP_FOR_DIAGNOSIS` | `STOP_FOR_DIAGNOSIS` |
| **D40 audits (`total / rejected`)** | `48 / 5` | `48 / 5` | `65 / 10` | `65 / 10` | `83 / 15` | `26 / 5` |
| **TRAIN anchor (`init / peak / final`)** | `1,211 / 1,823 / 1,342` | `1,211 / 1,823 / 1,342` | `1,211 / 1,849 / 1,286` | `1,211 / 1,849 / 1,286` | `1,211 / — / 1,249` | `1,211 / — / 1,232` |
| **Committed block updates (`epoch`)** | `420` | `420` | `678` | `678` | `274` | `132` |
| **Committed coordinate moves** | `854` | `854` | `1,505` | `1,505` | `274` | `132` |
| **Committed joint commits ($\|C_t\| \ge 2$)** | `123` | `123` | `199` | `199` | `0` (`K = 1`) | `0` (`K = 1`) |
| **Vector batches / moves committed** | `240 / 399` | `240 / 399` | `310 / 545` | `310 / 545` | `198 / 198` | `82 / 82` |
| **Combine batches / moves committed** | `180 / 455` | `180 / 455` | `208 / 535` | `208 / 535` | `76 / 76` | `28 / 28` |
| **Relation batches / moves committed** | `0 / 0` (`3,125` refused) | `0 / 0` (`3,125` refused) | `186 / 425` | `186 / 425` | `0 / 0` | `22 / 22` |
| **Straddle (`16..32`) batches attempted** | `125` | `125` | `171` | `171` | `0` (`K = 1`) | `0` (`K = 1`) |
| **Baseline DEV admitted (`Full / P_succ`)** | `4,244 / 1,351` | `4,244 / 1,351` | `4,244 / 1,351` | `4,244 / 1,351` | `4,244 / 1,351` | `4,244 / 1,351` |
| **Final DEV admitted (`Full / 14,287`)** | **`5,009` (`35.06%`)** | **`5,009` (`35.06%`)** | `3,827` (`26.79%`) | `3,827` (`26.79%`) | `4,292` (at Q1) | `4,244` (cp0) |
| **Final DEV `P_succ` admitted (`/ 3,847`)** | **`1,216` (`31.61%`)** | **`1,216` (`31.61%`)** | `1,144` (`29.74%`) | `1,144` (`29.74%`) | `1,106` (at Q1) | `1,351` (cp0) |
| **Final DEV complement admitted (`/ 10,440`)** | **`3,793` (`36.33%`)** | **`3,793` (`36.33%`)** | `2,683` (`25.70%`) | `2,683` (`25.70%`) | `3,186` (at Q1) | `2,893` (cp0) |
| **`G-Adm` (`>= 4,220` full & `>= 676` `P_succ`)** | **PASS** | **PASS** | FAIL (`3,827 < 4,220`) | FAIL (`3,827 < 4,220`) | PASS (at Q1) | — |
| **Final DEV writer `top1_correct` (`/ 14,287`)** | **`455` (`3.18%`)** | **`455` (`3.18%`)** | **`371` (`2.60%`)** | **`371` (`2.60%`)** | `871` (at Q1) | `133` (cp0) |
| **Final DEV `P_succ` writer `top1` (`/ 3,847`)** | `107` (`2.78%`) | `107` (`2.78%`) | `78` (`2.03%`) | `78` (`2.03%`) | — | `62` (cp0) |
| **Final DEV writer `top8_correct` (`/ 14,287`)** | **`1,484` (`10.39%`)** | **`1,484` (`10.39%`)** | **`1,124` (`7.87%`)** | **`1,124` (`7.87%`)** | `1,455` (at Q1) | `513` (cp0) |
| **Final DEV `P_succ` writer `top8` (`/ 3,847`)** | `318` (`8.27%`) | `318` (`8.27%`) | `243` (`6.32%`) | `243` (`6.32%`) | — | `202` (cp0) |
| **Final DEV `admitted_strict_win_rows`** | `93` | `93` | `172` | `172` | `325` (at Q1) | `133` (cp0) |
| **`relation_off` DEV (`admitted / top1 / top8 / strict`)** | — | — | `3,753 / 327 / 1,063 / 123` | `3,753 / 327 / 1,063 / 123` | — | — |
| **Training wall time (`s` / context `tok/s`)** | `726.1 s` (`50,910 tok/s`) | `726.8 s` (`50,860 tok/s`) | `4,353.1 s` (`11,562 tok/s`) | `4,493.7 s` (`11,201 tok/s`) | `~660 s` | `~450 s` |
| **Total wall time incl. `w2-eval` (`s`)** | `2,016.3 s` | `2,018.7 s` | `10,079.3 s` | `10,405.3 s` | — | — |

---

## 5. Step 3 Checkpoint-Restricted W2 Count-Table Evaluation (`w2-summary.json`)

On each completed seed's final passing checkpoint, `stage-t-run` captured all `1,049,856` model-matched TRAIN rows (`16,404` batches $\times$ `64` rows) and `14,287` DEV rows on GPU and built the exact read-code count table $T(c(x))$, unigram $U$, bigram $G(x_{\text{last}})$, and E8-symmetry-pooled $T_{\text{sym}}$, both unrestricted and restricted to each row's checkpoint-admitted candidate cells $A_W(x) = \{y : \mathrm{cell}(y) \in \mathrm{shortlist}_8(x)\}$:

| W2 Table / Metric | `#2845` / `#3179` (`K = 1` Q4 Baseline) | `T16_fz` Seed 1 & Seed 2 (`epoch = 420`) | `T16_rel` Seed 1 & Seed 2 (`epoch = 678`) |
|---|---:|---:|---:|
| **TRAIN rows captured** | `1,049,856` | `1,049,856` | `1,049,856` |
| **Distinct TRAIN read codes** | `14,412` | **`19,385`** (`+4,973`) | **`27,132`** (`+12,720`) |
| **Distinct DEV read codes** | `3,713` | **`4,067`** (`+354`) | **`4,429`** (`+716`) |
| **DEV target-admitted rows ($\|A_W\|$)** | `1,920 / 14,287` (`13.44%`) | **`5,009 / 14,287` (`35.06%`)** | **`3,827 / 14,287` (`26.79%`)** |
| **DEV `P_succ` target-admitted rows** | — | **`1,216 / 3,847` (`31.61%`)** | **`1,144 / 3,847` (`29.74%`)** |
| **Full-panel unigram $U$ `hit1`** | `1,226` (`8.58%`) | `1,226` (`8.58%`) | `1,226` (`8.58%`) |
| **Full-panel bigram $T_2 = G$ `hit1`** | `6,300` (`44.10%`) | `6,300` (`44.10%`) | `6,300` (`44.10%`) |
| **Full-panel read-code table $T(c(x))$ `hit1`** | `3,414` (`23.90%`) | **`4,085` (`28.59%`, `+671`)** | **`3,680` (`25.76%`, `+266`)** |
| **Checkpoint-restricted `U_restr(A_W)` `hit1`** | `410` | **`1,127`** (`+717`) | **`942`** (`+532`) |
| **Checkpoint-restricted `T_sym_restr(A_W)` `hit1`** | — | **`1,909`** | **`1,377`** |
| **Checkpoint-restricted `T_restr(A_W)` `hit1` (all DEV)** | `1,138` (`7.97%`) | **`2,140` (`14.98%`, `+1,002`)** | **`1,602` (`11.21%`, `+464`)** |
| **Checkpoint-restricted `T_restr(A_W)` `hit8` (all DEV)** | `2,223` (`15.56%`) | **`4,006` (`28.04%`, `+1,783`)** | **`3,007` (`21.05%`, `+784`)** |
| **Checkpoint-restricted `G_restr(A_W)` `hit1` (all DEV)** | `1,482` (`10.37%`) | **`3,084` (`21.59%`, `+1,602`)** | **`2,364` (`16.55%`, `+882`)** |
| **Checkpoint-restricted `G_restr(A_W)` `hit8` (all DEV)** | `2,331` (`16.32%`) | **`4,674` (`32.72%`, `+2,343`)** | **`3,529` (`24.70%`, `+1,198`)** |
| **`P_succ` restricted `T_restr(A_W)` `hit1` (`/ 3,847`)** | — | **`376`** | **`327`** |
| **`P_succ` restricted `G_restr(A_W)` `hit1` (`/ 3,847`)** | — | **`517`** | **`496`** |
| **Trained writer `W` `hit1` / `hit8` (all DEV)** | `149 / 1,148` | **`455 / 1,484`** | **`371 / 1,124`** |

---

## 6. Diagnosis & What This Unlocks Next

1. **Both admission (`5,009 / 14,287`) and read-code capacity (`T(c(x)) = 4,085`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`) clear the Stage 2 bars by large margins on `T16_fz`**:
   - When paired with the `A8-Outer` codebook and `BVG-K` (`K = 16`), the discrete model's 8-cell shortlist (`5,009` admitted DEV rows) and 16-bit read code (`19,385` distinct TRAIN codes on `T16_fz`, `27,132` on `T16_rel`) already carry enough information to predict **`2,140 / 14,287`** (`T_restr(A_W)`) to **`3,084 / 14,287`** (`G_restr(A_W)`) DEV tokens inside the admitted shortlist — far above the unigram bar (`1,226`) and `#3179`'s baseline restricted tables (`1,138` and `1,482`).
2. **Why the parameterless `OuterL1V1` writer reaches `455` (`T16_fz`) and `371` (`T16_rel`) `top-1`**:
   - In the current writer architecture, a single `4,096 x 32` integer table `table[v]` serves three roles simultaneously: input token embedding, address cell (`address_cell_id(table[v])`), and output token prototype scored by `OuterL1V1` (`L1` on outer coordinates `0..16`).
   - Under `K = 16` block proposals (`854` committed moves on `T16_fz`, `1,505` on `T16_rel`), moving `table[v]` and `Combine`/`Relation` gates on `64`-row batches expands cell shortlist admission (`4,244 -> 5,009` on `T16_fz`) and lifts `top-8` (`513 -> 1,484`), but within each admitted cell (`~16` rival tokens sharing the same 8-bit address cell), `OuterL1V1` distance to the single read vector `q` selects the true token first on only `455 / 5,009` (`9.08%`) of admitted rows, compared with `2,140 / 5,009` (`42.72%`) for `T_restr(A_W)` and `3,084 / 5,009` (`61.57%`) for `G_restr(A_W)`.
3. **Unlocked next work (per Section 3 of the FIT sheet and [`DECISION.md`](../../loop/tasks/m1-bvg-k-stage-t-training/antigravity-84/DECISION.md))**:
   - Closes `#2836` (`m1-bvg-k-stage-t-training`, `status: done`) with `fresh_evaluations = 0` (`fresh_v2` remains untouched).
   - Routes next work on the M1 discrete path to:
     - Registering training and D11 export on `#2834`'s full-vector cell-envelope lineage (`FV8-Env`, `5,709 / 14,287` DEV admitted at checkpoint 0) and/or binding the checkpoint-restricted integer read-code / successor lookup table (`T_restr(A_W)` / `G_restr(A_W)`, which already scores `2,140` / `3,084` on `T16_fz`'s `5,009` admitted cells using only D11 integer table lookups);
     - `m1-wider-code-carrier` (`#2847`).
