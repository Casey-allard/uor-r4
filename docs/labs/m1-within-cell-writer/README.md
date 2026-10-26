# Within-Cell Token Selection: Stage T Error Decomposition, Finite CUDA Qualification, and Zero-Update `B0` (`m1-within-cell-writer`, `#3267`)

- **Date**: 2026-10-26
- **Lab**: `antigravity` (`antigravity-93`, PR `#3280`)
- **Roadmap Item**: [`docs/roadmap/items/m1-within-cell-writer.md`](../../roadmap/items/m1-within-cell-writer.md) (`#3267`, referencing [`#3128`](https://github.com/UOR-Foundation/uor-r4/issues/3128) and [`#3129`](https://github.com/UOR-Foundation/uor-r4/issues/3129))
- **Pre-Registration Sheet**: [`docs/harness/prep/m1-within-cell-writer-2026-10-26.md`](../../harness/prep/m1-within-cell-writer-2026-10-26.md) ([`TEST FITNESS: FIT` + `D67 PRE-RUN REVIEW: PASS`](https://github.com/UOR-Foundation/uor-r4/issues/3267#issuecomment-6417919141))
- **Order**: [`docs/loop/tasks/m1-within-cell-writer/antigravity-93/ORDER.md`](../../loop/tasks/m1-within-cell-writer/antigravity-93/ORDER.md)
- **Hardware**: `1x NVIDIA GeForce RTX 5090` (`vast-57824058`, `$0.536/h`, `~28 min` active build + two-build run, `~$0.25`, `0` pods left running)
- **HF Store Archive**: `caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/` ([`SHA256SUMS`](./SHA256SUMS))
- **Verdict**: **Negative on epoch-0 two-pass interior-key framing (`NEGATIVE_TWOPASS_FRAMING_POSITIVE_WITHIN_CELL_RERANKER`, closes only the tested epoch-0 configuration under `D22`); positive on all 6 negative gates + 6 finite CUDA controls (`C1`–`C6`), positive on `ReadLawV2` + `GlobalWriterFrameV1` epoch-0 admission (`4,412` vs `4,244`) and `top-1` (`185` vs `119`, $d = +66 > 29$), and positive on the TRAIN-only within-cell read-code reranker (`2,140` vs `455` on Stage T `T16_fz`, $d = +1,685 > 86$; `1,107` vs `119` on epoch-0 `FullVectorV1`, $d = +988 > 68$). Closes `#3267`; `#3128` and `#3129` remain open (`References #3128`, `References #3129`).**

---

## 1. Executive Summary & Headline Metrics

Following `CHARTER.md §3.9` and `PART 5`, decode throughput (`tokens/s` and `rows/s` on `1x RTX 5090` across the `14,287` frozen `ft/dev` evaluation rows at context length `C = 192`, `V = 4,096`, `B = 4`, `K = 8`) leads every result:

| Phase / Evaluation Arm | Decode `tokens/s` (payload, `3x` rows/s) | Decode `rows/s` (`14,287` DEV rows) | Primary / Key Accuracy Readout | Verdict |
|---|---:|---:|---|---|
| **Finite CUDA Qualification (`within-cell-qualification.json`)** | — (`371 ms` Build 1 / `310 ms` Build 2) | — | `6 / 6` negative gates (`fails_when_set_aside = true`, `passes_when_active = true`) + `6 / 6` finite controls (`C1`–`C6`, `256/256` V2 orbit, `0/16` old-lineage parity diff) | **PASS (`all_controls_passed = true`)** |
| **Stage T `T16_fz` (`epoch = 420`) Error Decomposition** | — (exact replay of sealed `5,009` admitted cells) | — | `OuterL1V1` `top-1 = 455`, `top-8 = 1,484` (`3,525` admitted outside `top-8`; `4,208` lose within own cell) vs `R_cell(c)` **`2,140`** ($d = +1,685 > 86$) and `R_cell(c_perm)` `753` ($d = +1,387 > 80$) | **Mechanism isolated: `90.48%` (`1,597 / 1,765`) of missed `T_restr` hits lose within their own cell under `OuterL1V1`** |
| **Stage T `T16_rel` (`epoch = 678`) Error Decomposition** | — (exact replay of sealed `3,827` admitted cells) | — | `OuterL1V1` `top-1 = 371`, `top-8 = 1,124` (`2,703` admitted outside `top-8`; `3,226` lose within own cell) vs `R_cell(c)` **`1,602`** ($d = +1,231 > 74$) and `R_cell(c_perm)` `554` ($d = +1,048 > 74$) | **Mechanism isolated: `92.18%` (`1,190 / 1,291`) of missed `T_restr` hits lose within their own cell under `OuterL1V1`** |
| **`B0` Primary V1: `twopass_endpoint_output_v1` vs `nearest_endpoint_single_pass_v1`** | `4,181 tok/s` (`10,250 ms`) | `1,393 rows/s` | Intersection (`3,334` rows) 8-bit interior-key `(r_1, r_2)`: **`266` vs `260`** ($b = 19, c = 13, d = +6 \le 24$, $d^2 = 36 \le 128$); `top-1 = 122` vs `122` (`99` vs `96` intersection); `non_inert = 3,623` | **NEGATIVE (`clears_strict_paired_rule = false`; also ties `frozen_constant_frame_v1` `241`, $d = +25 \le 29$)** |
| **`B0` Primary V2: `twopass_endpoint_output_v2` vs `nearest_endpoint_single_pass_v2`** | `4,166 tok/s` (`10,286 ms`) | `1,388 rows/s` | Intersection (`3,334` rows) 8-bit interior-key `(r_1, r_2)`: **`268` vs `264`** ($b = 47, c = 43, d = +4 \le 24$, $d^2 = 16 \le 360$); `top-1 = 164` vs `169` (`144` vs `150` intersection); `non_inert = 8,287` | **NEGATIVE (`clears_strict_paired_rule = false`; ties `current_single_pass_v2` `260`, $d = +8 \le 26$)** |
| **`B0` Global Writer Frame: `twopass_global_writer_frame_v2`** | `4,191 tok/s` (`10,224 ms`) | `1,397 rows/s` | Admitted **`4,412 / 14,287`** (`P_succ = 1,426 / 3,847`), `top-1 = 185` vs `current_single_pass_v1` `119` ($b = 132, c = 66, d = +66 > 29$); interior-key intersection `274 / 3,334` | **Positive on epoch-0 admission (`+168`) and `top-1` (`+66`), but interior-key `274 / 3,334` stays near `264`** |
| **`B0` Within-Cell Reranker: `reranker_t_rebuilt_pooled_v1` ($R_{\text{cell}}(T_{\text{rebuilt}})$)** | `4,046 tok/s` (`10,590 ms`) | `1,349 rows/s` | Total `top-1` **`1,107 / 14,287`** (`797 / 3,334` intersection, `top-8 = 2,931`, `cell_prio = 1,108`, `top1_cell_only = 577`) vs `current_single_pass_v1` `119` ($d = +988 > 68$) and `permuted_null` `191` ($d = +916 > 66$) | **POSITIVE (`9.30x` over `OuterL1V1`; `clears_strict_paired_rule = true`)** |

---

## 2. Saved Stage T (`#2836`) Error Decomposition (`stage-t-decomposition.json`)

Before running `B0`, [`within-cell-writer-run decompose-stage-t`](../../../crates/uor-r4-discrete/src/within_cell_writer.rs) loaded the sealed Stage T (`#2836`) checkpoints (`T16_fz` Seed 1 at `epoch = 420` and `T16_rel` Seed 1 at `epoch = 678`), verified Gate R exact reproduction of their sealed `w2-summary.json` / `final-dev.json` metrics on GPU (`5,009 / 1,216 / 455 / 1,484 / 2,140 / 3,084` on `T16_fz`; `3,827 / 1,144 / 371 / 1,124 / 1,602 / 2,364` on `T16_rel`), and decomposed every admitted DEV row across shortlist coverage, within-cell vs cross-cell L1 competition, DEV read-code coverage, and TRAIN-only within-cell rerankers:

| Metric / Partition | `T16_fz` Seed 1 (`epoch = 420`, `854` moves) | `T16_rel` Seed 1 (`epoch = 678`, `1,505` moves) |
|---|---:|---:|
| Checkpoint SHA-256 | `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4` | `558b8810dc888d3acd2d5d135b9137062ce69f74e4148a822e1b59b9cfccb944` |
| Snapshot SHA-256 | `0608b4c8e0f3de4f67e7f62975306830fcdf8e665f10990ca1575b664242378c` | `34a0f33781c37ecde8e7be6fa5dd0a82203df06778972c5a3ee6ae8195588a28` |
| Total `ft/dev` rows / `mw-balp/train` model-matched rows | `14,287` / `1,049,856` | `14,287` / `1,049,856` |
| Distinct read codes (`TRAIN` / `DEV`) | `19,385` / `4,067` | `27,132` / `4,429` |
| **Cell admission (`dev_admitted_rows` / `dev_unadmitted_rows`)** | **`5,009`** (`35.06%`) / `9,278` (`P_succ`: `1,216 / 3,847`) | **`3,827`** (`26.79%`) / `10,460` (`P_succ`: `1,144 / 3,847`) |
| **Writer `OuterL1V1` `top-1` / `top-8` / `top-2..8` hits** | **`455`** / **`1,484`** / `1,029` | **`371`** / **`1,124`** / `753` |
| **Admitted rows outside writer `top-8` (`admitted_outside_writer_top8_rows`)** | **`3,525`** (`70.37%` of admitted rows) | **`2,703`** (`70.63%` of admitted rows) |
| Admitted rows where target token is rank-1 inside its own cell | `801` (`455` global `top-1` + `346` lose cross-cell) | `601` (`371` global `top-1` + `230` lose cross-cell) |
| **Admitted rows where target token loses within its own cell** | **`4,208`** (`84.01%`; `3,668` lose both within & cross-cell) | **`3,226`** (`84.30%`; `2,790` lose both within & cross-cell) |
| **`T_restr(A_W)` `top-1` / `top-8` hits on admitted cells** | **`2,140`** / **`4,006`** | **`1,602`** / **`3,007`** |
| Paired `T_restr(A_W)` vs `OuterL1V1`: `(both a, T_only b, W_only c, net d = b - c)` | `(375, 1765, 80, +1685)` (`F = 86`, **PASS**) | `(311, 1291, 60, +1231)` (`F = 74`, **PASS**) |
| Breakdown of `b` (`T_restr` hits missed by `OuterL1V1`) by `top-8` coverage | `1,117` outside `top-8` (`63.29%`) + `648` in `top-2..8` (`36.71%`) | `805` outside `top-8` (`62.35%`) + `486` in `top-2..8` (`37.65%`) |
| Breakdown of `b` by within-cell vs cross-cell L1 loss | `415` within-cell only + `168` cross-cell only + `1,182` both (`90.48%` lose within own cell) | `324` within-cell only + `101` cross-cell only + `866` both (`92.18%` lose within own cell) |
| **DEV read-code coverage on admitted rows (`seen in TRAIN` / `count >= 2` / `unseen`)** | **`5,007 / 5,009`** (`99.96%`) / `5,003` / `2` (`2,140` hits on seen, `0` on unseen) | **`3,821 / 3,827`** (`99.84%`) / `3,808` / `6` (`1,601` hits on seen, `1` on unseen) |
| **Within-cell reranker `reranker_code_top1_cell_only` (rank-1 cell `slots[0]` only)** | **`1,706`** (`3.75x` over `455`) | **`1,095`** (`2.95x` over `371`) |
| **Within-cell reranker `reranker_code_cell_prio` (cell rank then read-code count)** | **`2,142`** | **`1,597`** |
| **Within-cell reranker `reranker_code_pooled` ($R_{\text{cell}}(c)$ across 8 admitted cells)** | **`2,140`** (`top-8 = 4,006`) | **`1,602`** (`top-8 = 3,007`) |
| **Permuted-code null `reranker_code_permuted_null` ($R_{\text{cell}}(c_{\text{perm}})$, seed `20261026`)** | `753` ($b = 1,477, c = 90, d = +1,387 > F = 80$, **PASS**) | `554` ($b = 1,199, c = 151, d = +1,048 > F = 74$, **PASS**) |
| Within-cell bigram reranker `reranker_bigram_top1_cell_only` / `pooled` ($R_{\text{cell}}(g)$) | `2,162` / **`3,084`** (`top-8 = 4,674`) | `1,423` / **`2,364`** (`top-8 = 3,529`) |
| Within-cell unigram reranker `reranker_unigram_top1_cell_only` / `pooled` ($R_{\text{cell}}(u)$) | `1,073` / `1,127` | `815` / `942` |

### Key Diagnostic Finding from Stage T Decomposition

1. **The gap is not read-code sparsity or cell admission**: `99.96%` (`5,007 / 5,009`) of `T16_fz`'s admitted DEV rows have a read code seen in TRAIN (`5,003` with count $\ge 2$). Even when restricted strictly to the **single rank-1 admitted cell** (`slots[0]`), the TRAIN-only read-code reranker jumps from `455` to **`1,706`** (`+1,251` rows), and across the 8 admitted cells it reaches **`2,142`** (`cell_prio`) and **`2,140`** (`pooled`).
2. **Why `OuterL1V1` scores `455`**: Under `B = 4` endpoint cell addressing `c(y) = (r_0(y), r_3(y))`, every token inside cell `c = (a, b)` shares identical endpoint blocks `x_0 = o_a` and `x_3 = o_b`. Within-cell L1 distance $\|q - x_y\|_1$ therefore depends exclusively on interior blocks `b in {1, 2}` (`8` bits: `(r_1(y), r_2(y))`), while cross-cell L1 distance sums endpoint L1 and interior L1. On `90.48%` (`1,597 / 1,765`) of the rows where `T_restr(A_W)` succeeds and `OuterL1V1` fails, the target token loses to a rival token inside its own cell under interior L1 distance (`3,525 / 5,009` admitted targets are pushed outside `top-8` altogether).

---

## 3. Negative Gates & Finite CUDA Qualification (`within-cell-qualification.json`)

All 6 negative refusal gates and all 6 finite CUDA controls (`C1`–`C6`) were executed on `1x RTX 5090` across two independent release builds ([`within-cell-qualification.json`](./within-cell-qualification.json), `371 ms` Build 1 / `310 ms` Build 2):

| Gate / Control | Invariant Verified on GPU | Set-Aside Fault Result | Active Gate / Control Result | Verdict |
|---|---|---|---|---|
| **`G-Refuse-Blocks`** | Refuse block counts `s[1] != 4` (`status = 15`) with zero state mutation | `fails_when_set_aside = true` (`blocks = 3` succeeded when bypass bit `1` set aside) | `passes_when_active = true` (`status 15` refusal) | **PASS** |
| **`G-Refuse-InertPosition`** | Refuse inert position representation `s[14] >= 3` (`status = 16`) | `fails_when_set_aside = true` (`s[14] = 3` succeeded when bypass bit `2` set aside) | `passes_when_active = true` (`status 16` refusal) | **PASS** |
| **`G-Refuse-IdentityBypass`** | Refuse `R-Off` route identity bypass `s[13] != 0` (`status = 17`) | `fails_when_set_aside = true` (`s[13] = 1` succeeded when bypass bit `4` set aside) | `passes_when_active = true` (`status 17` refusal) | **PASS** |
| **`G-Refuse-RelationBound`** | Refuse relation score magnitude exceeding `max_rel_bound` (`status = 14`) | `fails_when_set_aside = true` (`max_rel_bound = 1` with `\|rel\| = 16` succeeded when bypass bit `8` set aside) | `passes_when_active = true` (`status 14` refusal) | **PASS** |
| **`G-Refuse-InvalidGateTrit`** | Refuse non-trit gate values outside `-1..=1` on host and GPU (`status = 10`) | `fails_when_set_aside = true` (`gate = 2` succeeded when bypass bit `16` set aside) | `passes_when_active = true` (`status 10` refusal on GPU & host) | **PASS** |
| **`G-Refuse-ZeroNonInertness`** | Refuse vacuous two-pass configuration when non-inert row count equals `0` | `fails_when_set_aside = true` (`inert_count = 0` flagged as vacuous refusal) | `passes_when_active = true` (`active_count = 1` admitted) | **PASS** |
| **`C1` (`256/256` Orbit Snap)** | Exhaustive $16 \times 16 = 256$ root pairs (`240` distinct $v \neq r_q$) on host & GPU (`discrete_within_cell_orbit_snap_check`) | — | `ReadLawV1` unweighted `r_q + v`: **`129 / 256`** (`113 / 240` distinct; `112` lose to lower-index $r_q$, `15` antipodal zero collapse); `ReadLawV2` positive `r_q + 2*v`: **`256 / 256`** (`240 / 240` distinct); `ReadLawV2` negative `r_q - 2*v`: **`0 / 256`** equal to $v$, **`256 / 256`** recover $-v$; `c1_host_gpu_orbit_exact_match = true` | **PASS** |
| **`C2` (Multi-Level & `-1` Gates)** | Active tree carry across multiple levels (`terms > 2`) and antipodal `-1` gate flip on GPU | — | Distinct query token (`tok = 2`) vs context (`tok = 1`): `terms(pos, neg, multi) = (3, 3, 5)`, `antipodal_blocks = 4 / 4` | **PASS** |
| **`C3` (`s[12] = 1/2` Residual)** | `SnapScale::UnitSnap1` (`s[12] = 1`) and `SnapScale::RawResidual` (`s[12] = 2`) on GPU | — | `UnitSnap1` decodes to valid orbit roots across all 4 blocks; `RawResidual` preserves exact unsnapped $\mathbb{Z}^8$ residual (`L1 > 32`) | **PASS** |
| **`C4` (Bounded Relation & Copy)** | Recent-8 `SameToken` / `SameFrame` cap (`rel <= 64`) + per-sweep reset + `[3, 1, 2, 3, 1] -> 2` single-copy succession | — | Repeated matches cap at `rel_node10 = 64`; `ReadLawV1` copy `top1 = false` vs `ReadLawV2` copy `top1 = true` (`within_cell_rivals = 0, cross_cell_rivals = 0`) | **PASS** |
| **`C5` (Two-Pass & Single-Cell)** | Frame-aligned copy `[1, 2, 3, 1] -> 2` across distinct endpoint frames + 16-token single-cell `FullVectorV1` separation | — | `interior_pair(cur = 237, near = 228, twopass_rel = 61, twopass_global = 52, target = 52)`; `FullVectorV1` cell `0` has `16 / 16` distinct interior pairs `(r_1, r_2)` | **PASS** |
| **`C6` (Frame Remap & Parity)** | `16 -> 16` TRAIN-fit frame remap validation gain + bit-exact old-lineage GPU parity against `Device::forward` and `Device::evaluate` | — | Holdout validation hits `(80/80, 80/80)` vs identity `(0, 0)`; old-lineage GPU parity mismatches **`0 / 16`** across `Flock` and `Relation` | **PASS** |

---

## 4. Zero-Update `B0` Evaluation on Re-Derived Epoch-0 `FullVectorV1` (`b0-grade.json`)

All 21 `B0` arms were evaluated on the re-derived epoch-0 `FullVectorV1` (`B = 4`, `sha256 = 2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2`, `G-Occ` verified) under `OuterL1V1` on the `14,287` frozen DEV rows (`intersection_admitted_rows = 3,334`):

| Arm | Law / Framing / Transport | Admitted (`Full` / `P_succ` / `Inter`) | 8-Bit Interior Key `(r_1, r_2)` (`Inter 3,334` / `All 14,287`) | `Top-1` (`All` / `Inter` / `Own-Cell Rank-1`) | `Top-8` (`All`) | Codes (`TRAIN` / `DEV` / `DEV Seen`) | Non-Inert vs `(Cur, Near)` (Interior / Top-1) | Comps/Row `(Cell, Tok)` & Resident Bytes | Decode `tok/s` (`rows/s`) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `current_single_pass_v1` | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `305 / 719` | `119 / 97 / 464` | `1,087` | `5,402 / 1,238 / 14,273` | `(0, 4556)` / `(0, 3699)` | `(256, 128)` / `524,548 B` | `4,179` (`1,393`) |
| `nearest_endpoint_single_pass_v1` | `V1` / `Nearest` / `Rel` | `4,244 / 1,351 / 3,334` | `260 / 629` | `122 / 96 / 417` | `980` | `5,402 / 1,168 / 14,060` | `(4556, 0)` / `(3699, 0)` | `(256, 128)` / `524,548 B` | `4,180` (`1,393`) |
| **`twopass_endpoint_output_v1`** | `V1` / `TwoPass` / `Rel` | `4,244 / 1,351 / 3,334` | **`266 / 691`** | `122 / 99 / 419` | `1,012` | `5,402 / 1,076 / 14,033` | `(5483, 3623)` / `(4939, 3197)` | `(256, 128)` / `524,548 B` | `4,181` (`1,393`) |
| `frozen_constant_frame_v1` | `V1` / `Frozen(0)` / `Rel` | `4,244 / 1,351 / 3,334` | `241 / 589` | `104 / 89 / 495` | `964` | `5,402 / 1,394 / 13,726` | `(8644, 8303)` / `(7581, 7100)` | `(256, 128)` / `524,548 B` | `4,198` (`1,399`) |
| `permuted_frame_v1` | `V1` / `Permuted` / `Rel` | `4,244 / 1,351 / 3,334` | `185 / 401` | `115 / 97 / 516` | `1,072` | `5,402 / 1,072 / 12,892` | `(11335, 11232)` / `(9683, 9587)` | `(256, 128)` / `524,676 B` | `4,182` (`1,394`) |
| `train_remap_16_v1` | `V1` / `Remap16` / `Rel` | `4,244 / 1,351 / 3,334` | `250 / 645` | `109 / 91 / 395` | `938` | `5,402 / 1,043 / 14,043` | `(5586, 3522)` / `(4944, 3025)` | `(256, 128)` / `524,676 B` | `4,185` (`1,395`) |
| `current_single_pass_v2` | `V2` / `Current` / `Rel` | `4,066 / 1,383 / 3,334` | `260 / 617` | `148 / 128 / 388` | `886` | `13,887 / 2,382 / 14,232` | `(0, 6624)` / `(0, 5379)` | `(256, 128)` / `524,548 B` | `4,164` (`1,388`) |
| `nearest_endpoint_single_pass_v2` | `V2` / `Nearest` / `Rel` | `4,066 / 1,383 / 3,334` | `264 / 574` | `169 / 150 / 375` | `850` | `13,887 / 2,275 / 13,142` | `(6624, 0)` / `(5379, 0)` | `(256, 128)` / `524,548 B` | `4,164` (`1,388`) |
| **`twopass_endpoint_output_v2`** | `V2` / `TwoPass` / `Rel` | `4,066 / 1,383 / 3,334` | **`268 / 540`** | `164 / 144 / 391` | `882` | `13,887 / 1,805 / 13,262` | `(8903, 8287)` / `(8382, 7707)` | `(256, 128)` / `524,548 B` | `4,166` (`1,388`) |
| `frozen_constant_frame_v2` | `V2` / `Frozen(0)` / `Rel` | `4,066 / 1,383 / 3,334` | `60 / 196` | `87 / 73 / 288` | `580` | `13,887 / 2,737 / 11,763` | `(13105, 13042)` / `(11886, 11772)` | `(256, 128)` / `524,548 B` | `4,183` (`1,394`) |
| `permuted_frame_v2` | `V2` / `Permuted` / `Rel` | `4,066 / 1,383 / 3,334` | `43 / 95` | `82 / 77 / 306` | `683` | `13,887 / 1,879 / 9,611` | `(14148, 14074)` / `(13461, 13282)` | `(256, 128)` / `524,676 B` | `4,168` (`1,389`) |
| `train_remap_16_v2` | `V2` / `Remap16` / `Rel` | `4,066 / 1,383 / 3,334` | `214 / 624` | `149 / 127 / 345` | `763` | `13,887 / 2,084 / 13,334` | `(8639, 6667)` / `(7857, 6081)` | `(256, 128)` / `524,676 B` | `4,170` (`1,390`) |
| **`twopass_global_writer_frame_v2`** | `V2` / `TwoPass` / `Global` | **`4,412 / 1,426 / 3,334`** | `274 / 618` | **`185 / 150 / 413`** | `959` | `4,309 / 885 / 14,266` | `(8953, 8515)` / `(9264, 8501)` | `(256, 128)` / `524,548 B` | `4,191` (`1,397`) |
| `lookup_t_frozen_restr_v1` ($T_{\text{frozen}}$) | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `160 / 197` | `197 / 160 / 197` | `1,339` | `1,518 / 1,238 / 901` | `(197, 197)` | `(256, 128)` / `1,214,376 B` | `4,070` (`1,356`) |
| **`reranker_t_rebuilt_pooled_v1`** ($R_{\text{cell}}(T_{\text{rebuilt}})$) | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | **`797 / 1,107`** | **`1,107 / 797 / 1,107`** | **`2,931`** | `5,402 / 1,238 / 14,273` | `(1107, 1107)` | `(256, 128)` / `1,398,240 B` | `4,046` (`1,349`) |
| `reranker_t_rebuilt_top1_cell_only_v1` | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `399 / 577` | `577 / 399 / 577` | `2,931` | `5,402 / 1,238 / 14,273` | `(577, 577)` | `(256, 128)` / `1,398,240 B` | `4,046` (`1,349`) |
| **`reranker_t_rebuilt_cell_prio_v1`** | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | **`797 / 1,108`** | **`1,108 / 797 / 1,108`** | **`2,931`** | `5,402 / 1,238 / 14,273` | `(1108, 1108)` | `(256, 128)` / `1,398,240 B` | `4,046` (`1,349`) |
| `reranker_code_permuted_null_v1` ($R_{\text{cell}}(c_{\text{perm}})$) | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `153 / 191` | `191 / 153 / 191` | `1,335` | `5,402 / 1,238 / 820` | `(191, 191)` | `(256, 128)` / `1,398,240 B` | `4,072` (`1,357`) |
| **`reranker_t_rebuilt_pooled_v2`** ($R_{\text{cell}}(T_{\text{rebuilt}})$ V2) | `V2` / `Current` / `Rel` | `4,066 / 1,383 / 3,334` | `650 / 949` | **`949 / 650 / 949`** | `2,497` | `13,887 / 2,382 / 14,232` | `(949, 949)` | `(256, 128)` / `2,186,052 B` | `4,034` (`1,344`) |
| `lookup_g_bigram_restr_v1` ($G_{\text{restr}}$) | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `1,324 / 1,857` | **`1,857 / 1,324 / 1,857`** | `3,683` | `1,266 / 1,238 / 14,287` | `(1857, 1857)` | `(256, 128)` / `892,488 B` | `4,052` (`1,351`) |
| `lookup_u_unigram_restr_v1` ($U_{\text{restr}}$) | `V1` / `Current` / `Rel` | `4,244 / 1,351 / 3,334` | `150 / 188` | `188 / 150 / 188` | `1,339` | `1 / 1,238 / 14,287` | `(188, 188)` | `(256, 128)` / `572,520 B` | `4,051` (`1,350`) |

### Strict Paired Comparisons (`d > 24 && d^2 > 4(b+c)`)

| Comparison Label | Candidate vs Reference | Population ($N$) | Hits `(Cand, Ref)` | `(both a, Cand b, Ref c, neither d)` | Net $d = b - c$ vs Floor $F = \max(24, \lceil 2\sqrt{b+c}\rceil)$ | Clears Strict Rule? |
|---|---|---:|---:|---|---:|---|
| **`primary_v1_twopass_vs_nearest_interior_key`** | `twopass_v1` vs `nearest_v1` (8-bit interior key) | `3,334` (inter) | `(266, 260)` | `(247, 19, 13, 3055)` | **`+6` vs `24`** ($d^2 = 36 \le 128$) | **`false` (PRIMARY NEGATIVE)** |
| **`primary_v2_twopass_vs_nearest_interior_key`** | `twopass_v2` vs `nearest_v2` (8-bit interior key) | `3,334` (inter) | `(268, 264)` | `(221, 47, 43, 3023)` | **`+4` vs `24`** ($d^2 = 16 \le 360$) | **`false` (PRIMARY NEGATIVE)** |
| `v1_twopass_vs_current_interior_key_intersection` | `twopass_v1` vs `current_v1` (8-bit interior key) | `3,334` (inter) | `(266, 305)` | `(226, 40, 79, 2989)` | `-39` vs `24` | `false` |
| `v2_twopass_vs_current_interior_key_intersection` | `twopass_v2` vs `current_v2` (8-bit interior key) | `3,334` (inter) | `(268, 260)` | `(184, 84, 76, 2990)` | `+8` vs `26` | `false` |
| `v1_twopass_vs_frozen_frame_interior_key_intersection` | `twopass_v1` vs `frozen_v1` (8-bit interior key) | `3,334` (inter) | `(266, 241)` | `(150, 116, 91, 2977)` | `+25` vs `29` ($d^2 = 625 \le 828$) | `false` (ties frozen frame) |
| `v1_twopass_vs_permuted_frame_interior_key_intersection` | `twopass_v1` vs `permuted_v1` (8-bit interior key) | `3,334` (inter) | `(266, 185)` | `(62, 204, 123, 2945)` | `+81` vs `37` | `true` |
| `v1_train_remap16_vs_nearest_interior_key_intersection` | `train_remap_16_v1` vs `nearest_v1` (8-bit interior key) | `3,334` (inter) | `(250, 260)` | `(234, 16, 26, 3058)` | `-10` vs `24` | `false` |
| **`v2_global_vs_v1_current_top1_all`** | `twopass_global_writer_frame_v2` vs `current_v1` (`top-1`) | `14,287` (all) | `(185, 119)` | `(53, 132, 66, 14036)` | **`+66` vs `29`** ($d^2 = 4356 > 792$) | **`true`** |
| **`reranker_v1_vs_writer_v1_top1_all`** | `reranker_t_rebuilt_pooled_v1` vs `current_v1` (`top-1`) | `14,287` (all) | `(1107, 119)` | `(38, 1069, 81, 13099)` | **`+988` vs `68`** | **`true`** |
| **`reranker_v1_vs_permuted_null_top1_all`** | `reranker_t_rebuilt_pooled_v1` vs `permuted_null_v1` (`top-1`) | `14,287` (all) | `(1107, 191)` | `(111, 996, 80, 13100)` | **`+916` vs `66`** | **`true`** |
| **`reranker_v1_vs_permuted_null_top1_intersection`** | `reranker_t_rebuilt_pooled_v1` vs `permuted_null_v1` (`top-1`) | `3,334` (inter) | `(797, 153)` | `(88, 709, 65, 2472)` | **`+644` vs `56`** | **`true`** |
| **`reranker_v2_vs_writer_v2_top1_all`** | `reranker_t_rebuilt_pooled_v2` vs `current_v2` (`top-1`) | `14,287` (all) | `(949, 148)` | `(44, 905, 104, 13234)` | **`+801` vs `64`** | **`true`** |
| `reranker_v1_vs_g_bigram_restr_top1_all` | `reranker_t_rebuilt_pooled_v1` vs `lookup_g_bigram_restr_v1` (`top-1`) | `14,287` (all) | `(1107, 1857)` | `(846, 261, 1011, 12169)` | `-750` vs `72` | `false` |

---

## 5. Decision & Unlocked Next Step (`#3267` Step 5)

Per [`docs/roadmap/items/m1-within-cell-writer.md`](../../roadmap/items/m1-within-cell-writer.md) ("How to start" step 5 and "Done when" bullet 5):
1. **Epoch-0 Two-Pass Interior-Key Framing (`TwoPassEndpointOutputV1`) Stopped (`D22` Configuration Closure)**:
   - On the re-derived epoch-0 `FullVectorV1` table under `ActiveLeafV1` (`Flock` leaf-only carry), Pass 1's endpoint outputs `(out_0, out_3)` match the cursor token's endpoints `(r_0(x_{L-1}), r_3(x_{L-1}))` whenever $q_0$ and $q_3$ snap back to their anchor roots, and when they differ, an untrained 4-bit endpoint root `out_0` or `out_3` carries no mutual information with the target token's 8-bit interior key `(r_1(y^*), r_2(y^*))` (`266` vs `260` in V1, $d = +6 \le 24$; `268` vs `264` in V2, $d = +4 \le 24$; and `266` vs `241` against `frozen_constant_frame_v1`, $d = +25 \le 29$).
   - Under `D22`, this closes **only** the tested epoch-0 `TwoPassEndpointOutputV1` configuration on `FullVectorV1` under `OuterL1V1`, and does **not** close `#3128` or `#3129` (`References #3128`, `References #3129`).
2. **Exact-Memory Within-Cell Writer Reranker / Lookup Path Confirmed as Positive Successor**:
   - Both on trained Stage T checkpoints (`T16_fz`: `455 -> 2,140`, $d = +1,685 > 86$; `T16_rel`: `371 -> 1,602`, $d = +1,231 > 74$) and on zero-update epoch-0 `FullVectorV1` (`119 -> 1,107`, $d = +988 > 68$), indexing an exact TRAIN-built count/read-code table inside the writer's admitted cells recovers the `1,685`-row Stage T within-cell selection loss without any additional read pass or shortlist expansion.
   - This directly feeds the downstream M1 exact-memory / saved-model closure path (`#2833` `m1-address-as-read-oracle`, `#2839` `m1-export-gap-and-toks`, `#2841` `m1-fresh-panel-transfer`, and `#2837` `m1-close-saved-model`).

---

## 6. Reproducibility & Pinned Digests

- **Source & Kernel Digests**:
  - `within_cell_writer_source.rs` (`kernel_sha256`): `0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6`
  - `geometry_sha256`: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`
  - `driver_source_sha256` (unchanged): `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`
  - `within_cell_writer.ptx`: `b6ddae94a4bb4319be6c4eef17f1cacd81cca1a073033a745b2717abab30b861`
  - `within_cell_writer.cubin`: `13b06bfb0dffe557c5f82bbce4b9a48e3eb8470583a62c09207e85041bf3ba61`
  - `within_cell_writer.meta`: `34bf1e8f869dd6eaa52c18458cbb7fc85c5116fee80ec49ae11e521ea7d1f1ab`
- **Artifacts in this Folder & HF Store (`caseyallard/uor-r4-store:results/antigravity/m1-within-cell-writer-2026-10-26/`)**:
  - [`within-cell-qualification.json`](./within-cell-qualification.json): `3e22a8e3e6dcd445786ebb00258fcdb5ac6ac7787660033e9727ba0750ddcedc` (Build 1) / `9994f0b1eba2d0afd6a4936831a0ff6462579cd4f12f059eb12d82c939a3d9be` (Build 2)
  - [`stage-t-decomposition.json`](./stage-t-decomposition.json): `e9dd940c0f58a1c494bb5d8f38a485528b6693bcaa67b23df3f1d867bec76cc6` (byte-identical across Build 1 and Build 2)
  - [`b0-grade.json`](./b0-grade.json): `68d397da4ff146cf88799216de2763cddbbb392d4d1fb49d240abc202d68676c` (Build 1) / `b16e80205256700bcc25156d7c3ed646c9305f9a06a935d25796ba4cb21fb6ae` (Build 2)
  - [`within-cell-writer-job.sh`](./within-cell-writer-job.sh) and [`SHA256SUMS`](./SHA256SUMS)
