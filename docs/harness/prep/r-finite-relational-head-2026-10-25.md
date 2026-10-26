# Pre-Registration: Stage M2 Finite Relational Head Discriminator (`r-finite-relational-head`, `#2877`)

- **Date:** 2026-10-25 (reconciled 2026-10-26 under D68 decision card [`docs/loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md`](../../loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md))
- **Author:** `antigravity` (ORDER [`docs/loop/tasks/r-finite-relational-head/antigravity-65/ORDER.md`](../../loop/tasks/r-finite-relational-head/antigravity-65/ORDER.md), completed under ORDER [`docs/loop/tasks/r-finite-relational-head/antigravity-96/ORDER.md`](../../loop/tasks/r-finite-relational-head/antigravity-96/ORDER.md))
- **Non-Author FIT & D68 Reviewer:** `antigravity` fresh-context reviewer `f9136838-edd2-41ab-8951-42f47f221ff1` ([`#2877` comment `6396409286`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6396409286)) and D68 reconciliation reviewer `b9f7ec13-d459-41e6-8b54-39ae7c196d9b` ([`#2877` comment `6421371461`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6421371461))
- **Verdict:** **TEST FITNESS: FIT** | **D68 DECISION REVIEW: PASS** | **OUTCOME: `RELATIONAL_WITHIN_ARM_GAIN_TRAILS_FROZEN_ZERO_ADMISSION`** (`mechanism_negative: false`, [`relation-head-grade.json`](../labs/r-finite-relational-head/relation-head-grade.json))
- **Milestone / Line:** M2 (`#2030`) / `m1-discrete-attention` — item [`r-finite-relational-head`](../../roadmap/items/r-finite-relational-head.md) (`#2877`), mechanism [`relation-read`](../../mechanisms/relation-read.md) (`ACTIVE`)

## 1. Question and Base Artifact

In `crates/uor-r4-discrete/src/training.rs` (`read_relation`, `ReadSpec::Relation { kept: 8, sweeps: 1 }`) and `crates/uor-r4-discrete/src/discrete.cu` (`panel_eval_row_k`), the finite relational head aggregates context leaves across the 5-level base-3 tree (`levels 0..4`) using a learned 25-trit signed gate table $g[e, \ell] \in \{-1, 0, +1\}$ (`RelationGates`, $5 \text{ link types} \times 5 \text{ tree levels}$) over the five exact discrete link relations (`RELATION_LINK_TYPES = 5`: parent-child, sibling, sequential successor, same token, same frame) on top of the 7-neighbor local Flock baseline (`FlockV1`).
- **Base artifact & prior qualification:**
  - [`docs/labs/attention-preflight-2026-10-18/README.md`](../attention-preflight-2026-10-18/README.md): finite qualification passed `9/9` controls (`Flock` collapse at $g \equiv 0$, route collapse, sensitivity `25/25`, level-0 sequential-successor copy `2,034 / 3,847` vs `1,351 / 3,847` for `Flock`, `+683` rows).
  - [`docs/labs/m1-a8-finite-cuda-qualification/README.md`](../m1-a8-finite-cuda-qualification/README.md) (`#2830`) and [`docs/labs/m1-a8-full-dose-rerun/README.md`](../m1-a8-full-dose-rerun/README.md) (`#2831`): CUDA qualification passed `5/5` controls (`a9c1228bc508eb12f61a82b35dd2f18722edee64bb1da4270a1fe37a01a2b2b6`), and the paired D40 admission-preserving rerun (`4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`, `kernel_sha256 = d600957a1ebf5f981db2e07b1d8534c3c981f4a1ae0abf9837ce110d586c75a6`, `geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`, `driver_source_sha256 = 31789abea8b955cb0e137257fc9e8b3e5eb42865fec7728396fa377c9d4228ea`) proved that active whole-interval rejection (`floor = 1,187 / 3,847` on the 3,847-row `TRAIN` anchor) raises `FrozenZero` at `Q1` (`4,096` batches) to `DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, and `DEV top1_correct = 871 / 14,287` (`+738` over init `133`, `+408` over best K16 `463`), while `Relation` (`LearnV1`) committed `22` relation-gate updates (`42` sensitive) and reached `1,600 / 3,847` `TRAIN` anchor admission at batch `256` before both arms hit the `K = 1` 64-batch cadence-granularity stop (`STOP_FOR_DIAGNOSIS: consecutive_rejections` at batch `1,600` for `Relation` and batch `5,248` for `FrozenZero`, `mechanism_negative: false`).
- **Question:** Under the Stage T batched-coordinate learning law (`BVG-K`, `K = 16`, `#2836`) and the D40 admission-preserving anchor (`floor = 1,187 / 3,847`), does training the 25-trit finite relational head (`ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationPolicySpec::LearnV1`) improve DEV admission (`b_3(P_succ)`, `b_3(Full)`) and DEV ranking (`top1_correct`, `top8_correct`) over its matched `FrozenZero` ($g \equiv 0 \equiv \text{Flock}$), `Route` (`ReadSpec::Route`), and `SuccessorOnly` ($g[\text{succ}, 0] = +1$ frozen) controls, and where does it stand against both the unconditional count tables (`unigram = 1,226 / 14,287`, `T2 = 6,300 / 14,287`) and the W2 admission-matched count tables (`T_code = 3,414 / 14,287`, `T_restr` on admitted rows)?

## 2. Arms, Data Split, and Steps

All four model arms run on the GPU (`discrete-stack`, `crates/uor-r4-discrete`, `CHARTER.md` §3.5) with identical initialization, identical 2,048-token discrete vocabulary (`artifacts/tokenizer-v1/vocab-2048.json`), identical Stage T `BVG-K` (`K = 16`) schedule (`16,404` batches), and identical two seeds (`seed = 1, 2`):

| Arm | Read / Relation Spec | Active Parameters | Scientific Role |
| :--- | :--- | :--- | :--- |
| **`H_relation` (Treatment)** | `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationPolicySpec::LearnV1` | Shared writer (`2,048 x 6` trits) + combine tables (`3 x 729` trits) + **25 learned relation trits** $g[e, \ell] \in \{-1, 0, +1\}$ | Tests full 5-link $\times$ 5-level learned discrete relational routing |
| **`C_frozen_zero` (Matched Null Control)** | `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationPolicySpec::FrozenZeroV1` ($g \equiv 0$) | Shared writer + combine tables ($g[e, \ell] \equiv 0$ frozen; bitwise identical to `FlockV1`) | Isolates the exact effect of the 25 relation trits on the identical kernel and schedule |
| **`C_route` (Tree-Route Control)** | `ReadSpec::Route { kept: 8 }`, `RelationPolicySpec::Disabled` | Shared writer + combine tables | Tests whether relational link gating beats fixed base-3 tree-route pooling |
| **`C_successor_only` (1-Link Ablation)** | `ReadSpec::Relation { kept: 8, sweeps: 1 }`, frozen $g[\text{succ}, 0] = +1$, all other $24$ trits $0$ | Shared writer + combine tables ($g$ frozen to the preflight level-0 copy configuration) | Tests whether learning all 25 link-level trits beats the hand-set level-0 sequential-successor gate (`2,034 / 3,847` init `P_succ`) |

- **Data split and SHA-256 pins (`caseyallard/uor-r4-store:inputs/m1-a8-2026-10-25/`):**
  - `train_stream.bin`: `7c74a4ef45f93b02963ff8ca1fa1d61d7bb1138eb79d292dc2b2ed43cf3376bb` (`1,053,696` B)
  - `p_succ_train_v1.txt`: `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` (`3,847` rows, baseline `1,211 / 3,847`, active floor `1,187 / 3,847`)
  - `dev_panel_v1.bin`: `29e536d2d8ff1371a5194b838dc0791dd47df7a42a165fa226a99d95620ec288` (`14,287` rows)
  - `p_succ_dev_v1.txt`: `e165f039c30a0a6bf5e74f9730e1c9d03f827c9d450201f31b7d318d18ff5b07` (`3,847` rows)
  - `fresh_panel_v2.bin`: `a643be37f1f007891a59dc42705158c3db52c4cf2322585cb8e3336e82b4cf30` (`2,095` rows; evaluated once at terminal checkpoint)
- **Lawful count-table comparators (`CHARTER.md` §4.4, §4.8, and W2 `#2845`):**
  1. Unconditional `DEV` tables (`n = 14,287`): `U` (unigram) `top-1 = 1,226 / 14,287`, `top-8 = 4,060 / 14,287`; `T2` / `G` (query-token bigram) `top-1 = 6,300 / 14,287`, `top-8 = 11,263 / 14,287`; `T_code` (W2 exact read-code table) `top-1 = 3,414 / 14,287`, `top-8 = 7,558 / 14,287`.
  2. Admission-matched `DEV` tables (`T_restr`, restricted to each arm's admitted cell set $\mathcal{A}(x)$ on the $N_{\text{adm}} = b_3(\text{Full})$ admitted rows, per `docs/labs/m1-w2-capacity-capture/README.md` §4.3 item 4): reports `top-1` and `top-8` of the model writer `W`, `T_restr(code)`, `T_restr(T2)`, and `T_restr(U)` on the exact $b_3(\text{Full})$ rows admitted by that arm.
- **Execution steps:**
  1. **Step 0:** Run finite CUDA controls `c1_flock_collapse`..`c5_determinism` (`5/5` required).
  2. **Step 1:** Run the 4 arms $\times$ 2 seeds (`seed = 1, 2`) for `16,404` batches under `BVG-K` (`K = 16`) + D40 admission anchor (`floor = 1,187 / 3,847`), scoring `DEV` (`14,287` rows + `3,847`-row `P_succ`) and the unconditional + admission-matched count tables at `Q1` (`4,096`) and `Final` (`16,404`).
  3. **Step 2:** Evaluate the terminal checkpoint once on the sealed 2,095-row `fresh_panel_v2.bin` transfer slice.

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum test power (`CHARTER.md` §4.3):** Two seeds (`seed = 1, 2`) across `14,287` `DEV` rows (`3,847` `P_succ` rows) and `2,095` fresh rows (`fresh_panel_v2.bin`). Paired McNemar noise floors: $F_{\text{succ}} = \max(24, \lceil 2\sqrt{b + c} \rceil)$ on `P_succ` (`n = 3,847`) and $F_{\text{full}} = \max(24, \lceil 2\sqrt{b + c} \rceil)$ on `Full` (`n = 14,287`).
- **Instrument gates:** `Gate C1–C5` (`5/5` finite CUDA controls pass); `Gate D40` (`TRAIN` anchor $\ge 1,187 / 3,847$ at every accepted checkpoint, `DEV b_3(P_succ) >= 676 / 3,847` at `Q1` and `Final`); `Gate S` (`accepted_by_kind[2] >= 16` committed relation-gate updates per seed on `H_relation`).

| Outcome | Condition (Must Hold on Each of `seed = 1` and `seed = 2`) | Meaning & What It Unlocks |
| :--- | :--- | :--- |
| **`PASS_M1_AND_RELATIONAL`** | `H_relation` beats `max(C_frozen_zero, C_route)` by $\ge F_{\text{full}}$ on `DEV top1_correct` (or $\ge F_{\text{succ}}$ on `DEV b_3(P_succ)` without losing $> F_{\text{full}}$ on `DEV top1_correct`), beats `C_successor_only` on `DEV top1_correct`, and beats `U = 1,226` and `T2 = 6,300` on `DEV top1_correct` with `fresh_v2` transfer within noise | Closes `#2877` as `done`, promotes `ReadSpec::Relation` as default M1/M2 read head, feeds `#2839` and `#2844` |
| **`PASS_RELATIONAL_GAIN_SUB_T2`** | `H_relation` beats `max(C_frozen_zero, C_route)` by $\ge F_{\text{full}}$ on `DEV top1_correct` (or $\ge F_{\text{succ}}$ on `DEV b_3(P_succ)` with non-inferior `DEV top1_correct`) and beats `W` on `T_restr`, with `DEV top1_correct < 6,300` (`T2`) | Closes `#2877` as `done` (admission + ranking reported against count tables with positive relational lift); localizes remaining gap to `T_restr` vs `b_3(Full)` for `#2844` |
| **`NULL_EQUIVALENT_TO_SUCCESSOR_OR_FLOCK`** | Instrument gates pass, and `H_relation` stays within $\pm F_{\text{full}}$ of `C_frozen_zero` (or `C_successor_only`) on both seeds | Closes `#2877` as `done` on its registered report; closes the 25-trit `sweeps = 1` config (`CHARTER.md` §4.1) in favor of simpler winner |
| **`INSTRUMENT_STOP`** | Any of `Gate C1–C5`, `Gate D40`, or `Gate S` fails, or `STOP_FOR_DIAGNOSIS` fires before full dose | Classified **instrument** (`CHARTER.md` §4.2); repair schedule/cadence coupling before calling any mechanism result |

## 4. Cost and Stop Conditions

- **Cost:** `1` pod (`2 x RTX 5090`, `~$2.02/h` via `scripts/pod/uor-pod`), `~2.5 hours` wall-clock (`8` arm-seed runs at `~30 min` per pair), estimated spend **`~$5.05`** (inside `<= 4` pods and `<= $8.00/h` fleet caps).
- **Stop rule:** (1) Immediate stop if any of `Gate C1–C5`, `Gate D40`, or `Gate S` fails (`0` mechanism claims). (2) Do not evaluate `fresh_panel_v2.bin` before `Final` (`16,404` batches). (3) Hold pod launch until `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`) completes.

## 5. Review Verdict

TEST FITNESS: FIT - research #2877 r-finite-relational-head
sheet: docs/harness/prep/r-finite-relational-head-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer f9136838-edd2-41ab-8951-42f47f221ff1; https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6396409286)
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).

## 6. Post-Stage-T D68 Reconciliation (`antigravity-96`, 2026-10-26)

Under [`DECISION.md`](../../loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md) (approved by fresh-context reviewer `b9f7ec13-d459-41e6-8b54-39ae7c196d9b`, [`#2877` comment `6421371461`](https://github.com/UOR-Foundation/uor-r4/issues/2877#issuecomment-6421371461)), the two-seed RTX 5090 GPU runs in [`#2836`](../labs/m1-bvg-k-stage-t-training/README.md) (`stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`), [`#2844`](../labs/m1-trunk-assembly-and-training/README.md) (`trunk-20261025.tar.gz` `4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35`), and [`#3267`](../labs/m1-within-cell-writer/README.md) (`stage-t-decomposition.json`) already executed `H_relation` (`T16_rel` Seeds 1 & 2) and `C_frozen_zero` (`T16_fz` Seeds 1 & 2) across all instrument gates (`Gate C1–C5` / `Step 0`, `Gate D40`, `Gate S`), count tables (`U`, `T2`, `T(c(x))`, `U_restr(A_W)`, `T_sym_restr(A_W)`, `T_restr(A_W)`, `G_restr(A_W)`), within-cell rerankers, and `fresh_v2` transfer. Because $\max(C_{\text{frozen\_zero}}, C_{\text{route}}) \ge C_{\text{frozen\_zero}} > H_{\text{relation}}$ holds strictly across both seeds on every admission and ranking metric (`3,827 < 5,009` `DEV b_3(Full)`, `1,144 < 1,216` `DEV b_3(P_succ)`, `371 < 455` `DEV top-1`, `1,602 < 2,140` `T_restr(A_W)`, `2,364 < 3,084` `G_restr(A_W)`, `2,227 < 2,989` Trunk `Level 3` DEV, `2,427 < 3,163` Trunk `Level 3` `fresh_v2`), additional GPU spend on `C_route` or `C_successor_only` is outcome-invariant and refused under `CHARTER.md` §3.5 and §5.10 (D67), resolving `#2877` to **`RELATIONAL_WITHIN_ARM_GAIN_TRAILS_FROZEN_ZERO_ADMISSION`** (`mechanism_negative: false`, [`relation-head-grade.json`](../labs/r-finite-relational-head/relation-head-grade.json)).
