# A8 Admission-Preserving Full-Dose Rerun (`#2831`)

- **Lab:** `antigravity` ([`#2831`](https://github.com/UOR-Foundation/uor-r4/issues/2831), PR [`#3192`](https://github.com/UOR-Foundation/uor-r4/pull/3192))
- **Milestone:** M1 (`#2029`) — `m1-discrete-attention`
- **Status:** **DIAGNOSTIC STOP (`STOP_FOR_DIAGNOSIS`, `stop_reasons: ["consecutive_rejections"]` on both `Relation` at batch `1,600` and `FrozenZero` at batch `5,248`; `FrozenZero` passed `Q1` at batch `4,096` with `DEV b_3(P_succ) = 1,106 / 3,847`, `b_3(Full) = 4,292 / 14,287`, and `DEV top1_correct = 871 / 14,287`; classified as an instrument/cadence-granularity stop under `K = 1` greedy single-batch updates + `64`-batch whole-interval rollback, `mechanism_negative: false`; fresh slice `fresh_v2` remains sealed with `0` evaluations).**
- **Pre-Registration Sheet:** [`../../harness/prep/m1-a8-full-dose-rerun-2026-10-25.md`](../../harness/prep/m1-a8-full-dose-rerun-2026-10-25.md) — **TEST FITNESS: FIT**, **D68 DECISION REVIEW: PASS**, **D67 PRE-RUN REVIEW: PASS** ([`#2831` comment `6394635218`](https://github.com/UOR-Foundation/uor-r4/issues/2831#issuecomment-6394635218))
- **Immutable Admission-Preserving Dose Sheet:** [`../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md) — SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450` (`FIT_SHEET_SHA256`)
- **Stage 1 Finite Qualification Evidence (`#2830`):** [`../m1-a8-finite-cuda-qualification/README.md`](../m1-a8-finite-cuda-qualification/README.md) (`a8-finite-cuda-qual-20261025.tar.gz`, SHA-256 `a9c1228bc508eb12f61a82b35dd2f18722edee64bb1da4270a1fe37a01a2b2b6`)
- **Source & Binary Identity:**
  - `reviewed_source_commit`: `da940043252432fcbd90e0432e3870b39e92c625`
  - `discrete-stack` `executable_sha256`: `b54d458af2f63990efea390c1f8e111e51454e73257c022565dcb1b89c561728`
  - `source_sha256`: `0a565024678e04d1be1007b1de556820581107ff86677cdb33ca82a768754da1`
  - `kernel_sha256`: `d600957a1ebf5f981db2e07b1d8534c3c981f4a1ae0abf9837ce110d586c75a6`
  - `geometry_sha256`: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`
  - `driver_source_sha256`: `31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea`
- **Pod & Spend:** `vast-57627077` (`2 x NVIDIA GeForce RTX 5090`, driver `580.95.05`, CUDA `12.8`, `$2.021/h`, leased `2026-10-25T06:59:51Z` to `2026-10-25T07:47:56Z`, `~48 min`, cost proxy `~$1.62`, `0` pods left running)
- **HF Store Custody:** `caseyallard/uor-r4-store:results/antigravity/a8-full-dose-rerun-2026-10-25/`
  - `a8-full-dose-rerun-20261025.tar.gz`: SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55` (`34,667,622` B, commit `8618073cc44138aa823ad82314916c7624d72722`)
  - `SHA256SUMS`: SHA-256 `ae90a8191475e3d0b77db484b658f6a3de6e6cd9facf03b8c354cbdef14cd119` (commit `dc22e9dbf664d49c35ba5ececc5b7ad1e434dd81`)
- **Committed Verification Summary:** [`dose-summary.json`](dose-summary.json) (SHA-256 `daad45e0c165472465823c7e22b0ba8e7832a7570fa4b1fea9ff8b03d9fec3c2`, generated and verified by `a8-qualify-stage verify-dose`)

---

## 1. Paired Arm Summary

Both arms ran in parallel (`CUDA_VISIBLE_DEVICES=0` for `Relation`, `CUDA_VISIBLE_DEVICES=1` for `FrozenZero`) under `discrete-stack attention-audit` with the registered active whole-interval rejection policy (`cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`, `consecutive_full_rejections = 3`, `audits_per_arm = 258`, fixed 3,847-row TRAIN anchor manifest `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de`, baseline TRAIN anchor `b_3(P_succ) = 1,211 / 3,847`, active audit floor `1,211 - 24 = 1,187 / 3,847`):

| Metric / Pin | Treatment Arm (`Relation`, `LearnV1`, Seed 1) | Control Arm (`FrozenZero`, `FrozenZeroV1`, Seed 1) |
| :--- | :--- | :--- |
| **Config SHA-256** | `f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd` | `c3fbffe08d13008b8b7ab400a4ef946f2c49632906d88e431d5e80314b9f5536` |
| **Lineage SHA-256** | `5ff9abad7291488769753e6fdc43b8472da2406425adf38cd648f3ec423439fc` | `e739ebfb886ef8ba70879ec7aaef1d0d616c8ea3a3c5653afab4c0c7dabda31c` |
| **Plan (`*-plan.json`) SHA-256** | `7ca3c1322fab29fb3e63ec96b2662387bc1e7cb542c4706b3ca112823427cd11` | `4d3572b24ae8e65e60dfb1fdf32c1eb80c02bb16d174a322877ccc09cde6ada7` |
| **Admission (`*-admission.json`) SHA-256** | `64b9b1f817c5c8a9a5126d2703081ff06a7d68478b6b3985f7d2652642ad3997` | `6e65a2a63cb38f32ad9bb1689685e2bfb87b05913746bcb6bc4a9d607e63db50` |
| **Report (`report.json`) SHA-256** | `eb02da3b46c72591e062025a7f1ecb0fecc8e493947d8efc7fd4e3e1e92ecbc1` | `ca5f946945763142fa45e461014a7cc765cb4b53b5850143013a61ed8049a421` |
| **Baseline DEV (`baseline-dev.json`) SHA-256** | `7a8d7535cb120248c870d3e578a35fd86075b9b7aa5a65c51cb7c274c4a658cb` | `884b9a8f3e90ad910879b106339ea721e3999458f35706ae7272f23160b5d72f` |
| **Initial DEV `b_3(P_succ)` / `b_3(Full)` / `top1`** | `1,351 / 3,847`, `4,244 / 14,287`, `133 / 14,287` | `1,351 / 3,847`, `4,244 / 14,287`, `133 / 14,287` |
| **Initial TRAIN Anchor `b_3(P_succ)` (Floor `1,187`)** | `1,211 / 3,847` | `1,211 / 3,847` |
| **Peak TRAIN Anchor `b_3(P_succ)`** | `1,607 / 3,847` (batch `64`); `1,600 / 3,847` (batch `256` after relation gate commits) | `1,607 / 3,847` (batch `64`) |
| **Q1 (`batch = 4,096`) Status** | Not reached (stopped at batch `1,600`) | **PASS** (`diagnostic_stop: false` at batch `4,096`, `epoch = 258`, `q1.json` SHA-256 `1e436e66a243cf7f9367c14372ef44ff1b9459fce98bf172100f28c5323f5ef7`) |
| **Q1 DEV `b_3(P_succ)` / `b_3(Full)`** | — | **`1,106 / 3,847`** (`>= 676 = ceil(1,351 / 2)` and `>= 500`; vs `857` in ungated `#2831` parent), **`4,292 / 14,287`** (`+48` over initial `4,244`) |
| **Q1 DEV `top1_correct` / `top8_correct`** | — | **`871 / 14,287`** (`+738` over initial `133`, `6.55x`), **`1,455 / 14,287`** |
| **Terminal Status & Reason** | `STOP_FOR_DIAGNOSIS` (`["consecutive_rejections"]`) | `STOP_FOR_DIAGNOSIS` (`["consecutive_rejections"]`) |
| **Completed Attempted Batches** | `1,600 / 16,404` (`25` interval audits, `audits_count = 26`, `5` rejected) | `5,248 / 16,404` (`82` interval audits, `audits_count = 83`, `15` rejected) |
| **Committed Accepted Updates (`epoch`)** | `132` (`82` vector, `28` combine, `22` relation) | `274` (`190` vector, `84` combine, `0` relation; `356` policy-refused) |
| **Rejected Provisional Updates (Rolled Back)** | `33` (`12` vector, `14` combine, `7` relation) across `5` intervals | `74` (`31` vector, `43` combine, `0` relation) across `15` intervals |
| **Sensitive Proposals (`[vector, combine, relation]`)** | `[202, 99, 42]` | `[478, 300, 182]` |
| **Final Restored Passing Checkpoint** | `continuation-01600.json` (`fccbaab5bc607893e4bfda8a7e324713e9161b1ea7a4c1f7e69c1322feea44ea`, `epoch = 132`, TRAIN anchor `admitted = 1,232 >= 1,187`) | `continuation-05248.json` (`70935d37234a868ceeeff66a132fb4584e27a67710f06ec7b87db39358fbc36b`, `epoch = 274`, TRAIN anchor `admitted = 1,249 >= 1,187`) |
| **Wall-Clock Elapsed (`elapsed_ns`)** | `1,443,815,125,983 ns` (`1,443.82 s` = `24.06 min`) | `1,099,022,329,790 ns` (`1,099.02 s` = `18.32 min`) |
| **Fresh Slice (`fresh_v2`)** | `SEALED_PENDING_PAIRED_DEV_DECISION` (`evaluations: 0`) | `SEALED_PENDING_PAIRED_DEV_DECISION` (`evaluations: 0`) |

---

## 2. Q1 (`4,096` Batches) Proof on `FrozenZero`

In the ungated A8 run of 19 October (`docs/labs/a8-dose-2026-10-19/README.md`), `b_3(P_succ)` eroded from `1,351 / 3,847` to `551` (`Relation`) and `857` (`FrozenZero`) by `Q1` (`4,096` batches). Under the active whole-interval rejection gate (`D40`), `FrozenZero` reached `Q1` (`4,096` batches, `epoch = 258`, after rolling back `8` individual breaching intervals between batches `1,472` and `3,776`) with:

- **TRAIN anchor `b_3(P_succ)` at batch `4,096`:** `1,258 / 3,847` (`+47` rows above the initial `1,211 / 3,847` baseline, `+71` rows above the `1,187` floor).
- **DEV `b_3(P_succ)` at batch `4,096` (`q1-dev.json`, SHA-256 `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`):** **`1,106 / 3,847`** (`+249` rows above the ungated `FrozenZero` Q1 score of `857`, and well above both Q1 bars `ceil(1,351 / 2) = 676` and `500`).
- **DEV `b_3(Full)` at batch `4,096`:** **`4,292 / 14,287`** (`+48` rows above the untrained initial `4,244 / 14,287`).
- **DEV `top1_correct` at batch `4,096`:** **`871 / 14,287`** (up from `133 / 14,287` at epoch `0`, a `+738` row / `6.55x` gain, and `+408` rows above the best previous trained K16 discrete model's `463 / 14,287`!).
- **DEV `top8_correct` at batch `4,096`:** **`1,455 / 14,287`**, `admitted_strict_win_rows = 679`.

---

## 3. Instrument-vs-Mechanism Classification of the `consecutive_rejections` Stop

Per CHARTER §4.1–§4.2 and Section 3 of [`m1-a8-full-dose-rerun-2026-10-25.md`](../../harness/prep/m1-a8-full-dose-rerun-2026-10-25.md), a `STOP_FOR_DIAGNOSIS` outcome must be classified as **instrument** versus **mechanism** before calling any result:

### 3.1 Classification: **Instrument / Cadence-Granularity Stop (`mechanism_negative: false`)**

1. **Active whole-interval rejection (`D40`) preserves admission and enables ranking gains:**
   - Every single breaching interval in both `Relation` (`5` intervals) and `FrozenZero` (`15` intervals) was caught at the 64-batch boundary and rolled back bit-identically to the last passing checkpoint (`TRAIN anchor admitted = 1,232 >= 1,187` at termination of `Relation`, and `1,249 >= 1,187` at termination of `FrozenZero`).
   - Where `FrozenZero` reached the `Q1` (`4,096`-batch) DEV evaluation, protecting admission on the 3,847-row TRAIN anchor (`1,258 / 3,847`) simultaneously held DEV `b_3(P_succ) = 1,106 / 3,847`, raised DEV `b_3(Full)` from `4,244` to `4,292 / 14,287`, and raised DEV `top1_correct` from `133` to **`871 / 14,287`**.
2. **Why `consecutive_full_rejections = 3` fired (`Relation` at batch `1,600`, `FrozenZero` at batch `5,248`):**
   - Under the `K = 1` single-coordinate schedule, each batch scores candidate moves on a single **64-row local TRAIN batch** (`loss_after < loss_before` on those 64 rows), whereas the admission audit checks `b_3(P_succ)` on the **3,847-row global TRAIN anchor** only once every **64 batches** (`cadence = 64`, `reject_whole_interval = true`).
   - As `epoch` grows (`epoch = 132` in `Relation`, `epoch = 274` in `FrozenZero`), the model's `TRAIN anchor admitted` count settles near `1,232–1,249` (`45–62` rows above the `1,187` floor), and each 64-batch interval accumulates `4–11` provisional coordinate updates (`132 -> 141`, `132 -> 139`, `132 -> 139` in `Relation` batches `1408..1600`; `274 -> 278`, `274 -> 281`, `274 -> 278` in `FrozenZero` batches `5056..5248`).
   - When even **one** of the `4–11` greedy 64-row updates inside a 64-batch interval moves a shared writer vector or gate that drops global TRAIN anchor admission by `> 45–62` rows, the coarse 64-batch whole-interval gate rejects the **entire** 64-batch interval (rolling back all `4–11` updates, including the admission-neutral or admission-improving ones).
   - Once three consecutive 64-batch windows each contain at least one such locally greedy but globally admission-eroding coordinate move (`batches 1408..1600` in `Relation` with `admitted = 842, 1104, 768 < 1187`; `batches 5056..5248` in `FrozenZero` with `admitted = 997, 1169, 1170 < 1187`), `consecutive_full_rejections = 3` fires and halts the run.
3. **Why this is an instrument/controller granularity limit, not a mechanism negative:**
   - Neither `relation-read` (`Relation`) nor `address-cells` (`A8`) failed: `Relation` committed `22` relation-gate updates (`42` sensitive) and reached `1,600 / 3,847` TRAIN anchor admission at batch `256`, and `FrozenZero` reached `871 / 14,287` DEV top-1 at `Q1` (`4,096` batches) with `4,292 / 14,287` full-panel admission.
   - Rather, pairing **64-row local greedy proposals** with **64-batch coarse whole-interval rollback** (`consecutive_full_rejections = 3`) creates a coupling where one bad coordinate update per 64 batches kills the entire 64-batch window. Moving to batched coordinate updates (`BVG-K` Stage T, `#2836`) or per-move / narrower-window admission screening avoids discarding 63 batches of good updates when a single coordinate move breaches the anchor floor.

---

## 4. Complete Interval Trajectories

### 4.1 `Relation` (`LearnV1`, Seed 1) — `25` Audited Intervals (`batches 64..1,600`)

| Batch | Start Epoch | Inspected Epoch | Post-Audit Epoch | TRAIN Anchor `admitted` (`floor = 1,187`) | Rejected |
| ---: | ---: | ---: | ---: | ---: | :--- |
| `64` | `0` | `8` | `8` | `1,607` | `false` |
| `128` | `8` | `23` | `23` | `1,552` | `false` |
| `192` | `23` | `37` | `37` | `1,496` | `false` |
| `256` | `37` | `42` | `42` | `1,600` | `false` |
| `320` | `42` | `47` | `47` | `1,576` | `false` |
| `384` | `47` | `58` | `58` | `1,444` | `false` |
| `448` | `58` | `71` | `71` | `1,416` | `false` |
| `512` | `71` | `74` | `74` | `1,269` | `false` |
| `576` | `74` | `81` | `74` | `1,186` | **`true`** (rolled back `7` updates) |
| `640` | `74` | `78` | `78` | `1,269` | `false` |
| `704` | `78` | `78` | `78` | `1,269` | `false` |
| `768` | `78` | `78` | `78` | `1,269` | `false` |
| `832` | `78` | `84` | `84` | `1,269` | `false` |
| `896` | `84` | `87` | `84` | `1,186` | **`true`** (rolled back `3` updates) |
| `960` | `84` | `90` | `90` | `1,269` | `false` |
| `1,024` | `90` | `90` | `90` | `1,269` | `false` |
| `1,088` | `90` | `90` | `90` | `1,269` | `false` |
| `1,152` | `90` | `94` | `94` | `1,269` | `false` |
| `1,216` | `94` | `103` | `103` | `1,269` | `false` |
| `1,280` | `103` | `110` | `110` | `1,264` | `false` |
| `1,344` | `110` | `121` | `121` | `1,257` | `false` |
| `1,408` | `121` | `132` | `132` | `1,232` | `false` |
| `1,472` | `132` | `141` | `132` | `842` | **`true`** (`consec = 1`, rolled back `9` updates) |
| `1,536` | `132` | `139` | `132` | `1,104` | **`true`** (`consec = 2`, rolled back `7` updates) |
| `1,600` | `132` | `139` | `132` | `768` | **`true`** (`consec = 3` -> `STOP_FOR_DIAGNOSIS`, rolled back `7` updates) |

### 4.2 `FrozenZero` (`FrozenZeroV1`, Seed 1) — Key Milestones Across `82` Audited Intervals (`batches 64..5,248`)

- **Initial (`batch 0`):** `epoch = 0`, TRAIN anchor `admitted = 1,211 / 3,847`, DEV `b_3(P_succ) = 1,351 / 3,847`, DEV `b_3(Full) = 4,244 / 14,287`, DEV `top1 = 133 / 14,287`.
- **Peak TRAIN anchor (`batch 64`):** `epoch = 8`, TRAIN anchor `admitted = 1,607 / 3,847`.
- **Pre-Q1 rejections rolled back cleanly (`8` intervals):** batches `1,472` (`admitted = 961`), `2,176` (`1,077`), `2,240` (`1,097`), `2,688` (`1,046`), `2,944` (`1,080`), `3,072` (`1,024`), `3,392` (`1,011`), and `3,776` (`1,072`), each time recovering on a subsequent passing interval.
- **Q1 (`batch 4,096`):** `epoch = 258`, TRAIN anchor `admitted = 1,258 / 3,847`, DEV `b_3(P_succ) = 1,106 / 3,847` (`PASS`), DEV `b_3(Full) = 4,292 / 14,287`, DEV `top1 = 871 / 14,287`, DEV `top8 = 1,455 / 14,287`.
- **Post-Q1 (`batches 4,160..5,248`):** single-interval rejections at batches `4,160` (`1,179`), `4,352` (`1,030`), `4,480` (`1,181`), `4,864` (`1,108`), followed by passing intervals at `admitted = 1,249 / 3,847` through batch `5,056` (`epoch = 274`), then three consecutive rejected intervals at batches `5,120` (`997`), `5,184` (`1,169`), and `5,248` (`1,170`), triggering `STOP_FOR_DIAGNOSIS` (`consecutive_rejections`) with restored checkpoint `continuation-05248.json` at `epoch = 274` (`TRAIN anchor admitted = 1,249 >= 1,187`).

---

## 5. Host Reproduction & Verification Commands

```sh
# Run unit tests (including committed dose-summary.json verification)
~/.cargo/bin/cargo test --locked -p uor-r4-discrete --bin a8-qualify-stage --no-fail-fast

# Re-verify the extracted HF store bundle (caseyallard/uor-r4-store:results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz)
~/.cargo/bin/cargo run --locked -p uor-r4-discrete --bin a8-qualify-stage -- verify-dose \
  --relation-out <EXTRACTED_A8_DOSE_DIR>/relation-out \
  --frozen-zero-out <EXTRACTED_A8_DOSE_DIR>/frozen-zero-out \
  --summary-out docs/labs/m1-a8-full-dose-rerun/dose-summary.json
```
