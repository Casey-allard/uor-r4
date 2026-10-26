# `m1-wider-code-carrier` — Wider Code Carrier ($B = 6, 8$ Blocks): Finite CUDA Qualification and Zero-Training GPU Pre-Flight

- **Roadmap item:** [`docs/roadmap/items/m1-wider-code-carrier.md`](../../roadmap/items/m1-wider-code-carrier.md) (`#2847`, Milestone `#2029`)
- **Pre-registration sheet:** [`docs/harness/prep/m1-wider-code-carrier-2026-10-26.md`](../../harness/prep/m1-wider-code-carrier-2026-10-26.md) (`TEST FITNESS: FIT` and `D67 PRE-RUN REVIEW: PASS`, [#2847 comment](https://github.com/UOR-Foundation/uor-r4/issues/2847#issuecomment-6415495380))
- **GPU execution:** 1x NVIDIA GeForce RTX 5090 (`vast-57809208`, driver `580.95.05`, `32,607 MiB`, `sm_120`), source commit `fe6bcd0678724c2001f527bb856120d67aee5a50` (`crates/` at `99b251ccba823612fe3acc04c083fe584adf9e6a`, `driver_source_sha256 = 04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`), job [`wider-carrier-job.sh`](wider-carrier-job.sh), configs in [`configs/`](configs/).
- **Sealed artifacts:**
  - Qualification JSON: [`wider-carrier-qualification.json`](wider-carrier-qualification.json) (`sha256: 384ce237403b82e8745b5f6fe3f9525fedb5f852e06dc9a0e087c9f34ac2418f`)
  - Suite grade JSON: [`grade.json`](grade.json) (`sha256: 7766aae432500f632d4ffb01f9e7fb4f963c1b56c8ca3d81c0968b292a7300e0`, `overall_outcome: "PASS"`)
  - HF store archive: `caseyallard/uor-r4-store:results/antigravity/m1-wider-code-carrier-2026-10-26/run/` (84 files with verified `SHA256SUMS`) and `results/antigravity/m1-wider-code-carrier-2026-10-26/tables/` (8 pinned tables with verified `SHA256SUMS`). Three pre-treatment instrument stops are preserved at `results/antigravity/m1-wider-code-carrier-2026-10-26/fault{1,2,3}/`.

---

## 1. Headline Decode Throughput (`tokens/s` & `rows/s`) and Negatives First

### 1.1 Decode Speed vs. Carrier Width ($B \in \{4, 6, 8\}$, $V = 4,096, C = 192, K = 8$, `Relation { kept: 8, sweeps: 1 }` `ActiveLeafV1`, RTX 5090 `sm_120`)

| Blocks $B$ (`width = 8B`) | Address Metric (`s[15]`) | 64-Row Batch Median (`us`) | 64-Row Throughput (`rows/s`) | Full 14,287-Row DEV Eval (`ms`) | Full-DEV Decode (`tokens/s`) |
|---:|---|---:|---:|---:|---:|
| `4` (`32`) | `OuterL1V1` (`0`) | `36,434` | `1,756` | `16,642` | `858` |
| `4` (`32`) | `CellEnvelopeL1V1` (`1`) | `38,120` | `1,678` | `17,184` | `831` |
| `4` (`32`) | `FullVectorCellL1V1` (`2`) | `38,121` | `1,678` | `17,215` | `829` |
| `6` (`48`) | `OuterL1V1` (`0`) | `37,972` | `1,685` | `21,384` | `668` |
| `6` (`48`) | `CellEnvelopeL1V1` (`1`) | `40,636` | `1,574` | `22,192` | `643` |
| `6` (`48`) | `FullVectorCellL1V1` (`2`) | `40,395` | `1,584` | `22,182` | `644` |
| `8` (`64`) | `OuterL1V1` (`0`) | `41,450` | `1,544` | `26,065` | `548` |
| `8` (`64`) | `CellEnvelopeL1V1` (`1`) | `44,633` | `1,433` | `27,110` | `527` |
| `8` (`64`) | `FullVectorCellL1V1` (`2`) | `44,631` | `1,433` | `27,102` | `527` |

### 1.2 Negatives and Boundaries First

1. **No `P_succ` cell-admission gain over $B = 4$ under `CellEnvelopeL1V1`:** On the 3,847-row `P_succ` copy-successor sub-panel under `CellEnvelopeL1V1`, $B = 4$ (`FV8-B4-Env`) already admits `1,980 / 3,847` (`51.5%`), whereas $B = 6$ (`WB6-Env`) admits `1,901 / 3,847` (`b / c = 419 / 498`, `b - c = -79`) and $B = 8$ (`WB8-Env`) admits `1,965 / 3,847` (`FV8-B4-Env` vs `WB8-Env`: `b / c = 439 / 424`, `b - c = +15`). Thus `P_succ` cell admission under `CellEnvelopeL1V1` is already saturated at $B = 4$; widening the carrier does not raise `CellEnvelopeL1V1` `P_succ` admission.
2. **Where wider carriers DO improve over $B = 4$:**
   - **Full-DEV cell admission (14,287 rows):** rises from `5,709` (`FV8-B4-Env`) to `6,302` (`WB6-Env`, `+593` rows) and `6,342` (`WB8-Env`, `+633` rows), and under `FullVectorCellL1V1` from `5,666` (`FV8-B4-TwoScale`) to `6,322` (`WB6-TwoScale`, `+656` rows) and `6,353` (`WB8-TwoScale`, `+687` rows).
   - **Two-scale `P_succ` admission at $B = 8$:** `WB8-TwoScale` (`1,977 / 3,847`) beats $B = 4$ `FV8-B4-TwoScale` (`1,802 / 3,847`) by `+175` rows (`b / c = 572 / 397` in favor of `WB8-TwoScale`, clearing the two-sigma floor `F = 63`).
   - **Checkpoint-0 `Top-1` and `Top-8` ranking:** whereas `#2834` at $B = 4$ saw `Top-1` dip from `A8-Outer`'s `133` to `118` (`Top-8 = 1,126–1,179`), widening the carrier to $B = 6$ (`WB6-Env` / `WB6-TwoScale`) raises checkpoint-0 `Top-1` to **`295`** (`Top-8 = 1,852` / `1,678`), and $B = 8$ (`WB8-Env` / `WB8-TwoScale`) reaches `Top-1 = 240` and **`Top-8 = 2,103` / `1,919`**.
3. **Decode throughput cost:** Widening from $B = 4$ (`831 tokens/s`) to $B = 6$ (`643 tokens/s`, `-22.6%`) and $B = 8$ (`527 tokens/s`, `-36.6%`) scales runtime linearly in $B$.
4. **Zero-training scope:** All results are at checkpoint 0 (`applied_updates = 0`, `training: "HELD"`). Non-outer address metrics remain refused by `CorpusUpdateAdmission` and `d11_export` until `#2836`.
5. **Pre-treatment instrument faults (`fault1`–`fault3`):**
   - `fault1`: `G-Q` Control 4 checked `dev_fv.propose().is_err()`, whereas the non-outer metric update guard lives in `training.rs` (`CorpusUpdateAdmission::guarded`), while `d11_export::export_d11` and foreign-batch `inspect_wider_carrier_rows` are the device/export boundary checks. Fixed in `8d41f62d0`.
   - `fault2`: `wider-carrier-job.sh` compared `.controls` raw JSON across Build 1 and Build 2 without stripping `.controls[3].detail.d11_exports[].artifact_sha256` (which embeds `Snapshot::source_sha256` containing `ptxas` compile-time lines across `target-wider-1` vs `target-wider-2`). Fixed in `fdf6f78d2`.
   - `fault3`: `wider-carrier-job.sh` and `grade_wider_suite` initially asserted `WB6-Outer == A8-Outer` on the full corpus arm, overlooking that `address-cells-codebook` computes a $B$-native PrimeZeta fallback (`roots[B-1]` from zeta-zero channels `4(B-1)..4B-1`) and `writer` ranks within-cell tokens by `token_distance` across all $B$ blocks (`G-Q` Control 3 already proves outer-endpoint and cell-admission invariance across $B \in \{4, 6, 8\}$ on a shared fallback). Fixed in `99b251ccb` before any Stage 5 treatment arm ran.

---

## 2. Multi-Pair Wider-Block Placement (`AddressCellsMode::WiderBlocksV1`)

In [`crates/uor-r4-discrete/src/address_cells.rs`](../../../crates/uor-r4-discrete/src/address_cells.rs), `AddressCellsMode::FullVectorV1` (`#2834`) optimizes only a single interior root pair `(block 1, block 2)` and fills blocks `3..B-1` at $B > 4$ with an un-pulled XOR pad (`((ic / 16) ^ (ic % 16) ^ b) & 15`).

`AddressCellsMode::WiderBlocksV1` (and its matched permutation control `WiderBlocksShuffledV1 { seed }`) generalizes interior placement in `address_cells::build(spec, counts, fallback_cells, mode)` to $P = \lfloor (B - 1) / 2 \rfloor$ interior block pairs (`P = 2` pairs = 4 interior blocks at $B = 6$; `P = 3` pairs = 6 interior blocks at $B = 8$):
1. **Outer blocks `(0, B-1)`:** Placed by `address_cells::build` (`quota = V / 256 = 16`, `occupied_cells = 256`), preserving the 256-cell writer layout and `UORLUT01` D11 serving format.
2. **Stage-by-stage residual bidirectional interior pull (`stage = 0..P`):**
   - Every stage `stage` (`blocks 2*stage + 1, 2*stage + 2`) enforces within-cell uniqueness (`!used_stage[cell][ic]`, guaranteeing `distinct_vectors == V == 4,096` and `within_cell_tie_mass == 0`) and global pair quota balance (`load_stage[ic] < interior_quota`), with coprime modular-shear Latin fallback `latin_interior_anchor_stage(cell, stage)`.
   - Across both incoming ($u \to t$) and outgoing ($t \to u$) bigram edges, each stage weights every already-placed neighbor $u$ by `u128::from(count) * u128::from(4 + outer_d + 2 * prev_int_d)`, where `outer_d` is the outer-endpoint L1 distance and `prev_int_d` is the cumulative interior L1 distance across earlier stages (`prev_int_d == 0` at `stage = 0`, and $\sum_{\text{prev} < \text{stage}} \text{cell\_l1}(\text{pic\_other}, \text{pic\_self})$ for `stage >= 1`), concentrating additional interior block pairs on separating high-frequency bigram neighbors that remained distant in earlier block pairs.

---

## 3. Finite CUDA Qualification (`G-Q`, `wider-carrier-qualification.json`)

Compiled ahead of time by `nvcc` for `sm_120` in `build.rs` from [`crates/uor-r4-discrete/src/wider_carrier_source.rs`](../../../crates/uor-r4-discrete/src/wider_carrier_source.rs) (`wider_carrier_image_sha256`: source `bafb56be3078ddf6debd21b8ed6acc63262fba7d5269807e5585131e47fa0696`, PTX `3082c672be5d7b4a90105aad02e3d09e26ea866bbf1f5841b1ea26deff24b517`, cubin `407b0c08f460a11a2ac83c770b00b111bcad593ba56e5a4bd4f95f791cf1a0d8`, resource report `bcf76eb6465b759fa73ff1642af2fadabb49bddcd2adc00d6c77ae814a05a5d9`, identical across Build 1 and Build 2):

| Control Index & Name | Scope | Verified Result | Status |
|---|---|---|---|
| `1. wider_carrier_b6_host_gpu_distance_parity` | $V=512, B=6$ (`width=48`), 16 rows $\times$ 3 metrics (`48` row evals, `24,576` token pairs) | Exact GPU vs. host oracle parity on `q`, `env[256]`, `sum[256]`, `cell_dist[256]`, `admitted_cells[8]`, `Device::evaluate` target-cell admission (`host_target_admitted == gpu_target_admitted`), `token_distances[512]` (`full_l1`, `outer_l1`, `interior_l1`), and zero device state/epoch mutation across all 3 metrics | `PASS` |
| `2. wider_carrier_b8_host_gpu_distance_parity` | $V=512, B=8$ (`width=64`), 16 rows $\times$ 3 metrics (`48` row evals, `24,576` token pairs) | Exact GPU vs. host oracle parity on `q`, `env[256]`, `sum[256]`, `cell_dist[256]`, `admitted_cells[8]`, `Device::evaluate` target-cell admission (`host_target_admitted == gpu_target_admitted`), `token_distances[512]` (`full_l1`, `outer_l1`, `interior_l1`), and zero device state/epoch mutation across all 3 metrics | `PASS` |
| `3. wider_carrier_outer_invariance_across_b4_b6_b8` | $V=512, B \in \{4, 6, 8\}$ on shared fallback under `OuterL1V1` | `outer_query_blocks_identical = true`, `outer_admitted_cells_identical = true`, `target_admitted_rows = 4` across $B = 4, 6, 8$ | `PASS` |
| `4. wider_carrier_d11_export_and_state_restoration` | $B \in \{6, 8\}$ `UORLUT01` export/import + refusals + zero mutation | `artifact_bytes = 101,892` ($B=6$) and `134,684` ($B=8$) round-trip bit-exactly; non-outer D11 export and foreign-batch inspection refused; `state_unchanged = true` (`epoch == 0`) | `PASS` |

---

## 4. Zero-Training 15-Arm GPU Pre-Flight Results (`grade.json`)

All 15 arms were evaluated on the 14,287-row DEV panel (`3,847` `P_succ` rows, `10,440` complement rows) at checkpoint 0 (`seed = 1`, `Relation { kept: 8, sweeps: 1 }`, `ActiveLeafV1`, `K = 8`). Every arm satisfied `G-Occ` (`distinct_vectors = 4,096`, `occupied_cells = 256`, `within_cell_tie_tokens = 0`, `within_cell_tie_dev_rows = 0`, `dev_alias_ceiling_rows = 14,287`).

| Arm | $B$ (`width`) | Metric (`s[15]`) | Mode | `P_succ` Admitted (`/ 3,847`) | Full-DEV Admitted (`/ 14,287`) | Complement Admitted (`/ 10,440`) | `Top-1` | `Top-8` | Strict-Win Rows | Decode (`tokens/s`) |
|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|
| `A8-Outer` | 4 (32) | `OuterL1V1` | `address_structured_v1` | `1,351` | `4,244` | `2,893` | `133` | `513` | `133` | `858` |
| `FV8-B4-Env` | 4 (32) | `CellEnvelopeL1V1` | `address_full_vector_v1` | `1,980` | `5,709` | `3,729` | `118` | `1,126` | `110` | `831` |
| `FV8-B4-TwoScale` | 4 (32) | `FullVectorCellL1V1` | `address_full_vector_v1` | `1,802` | `5,666` | `3,864` | `118` | `1,179` | `110` | `829` |
| `WB6-Outer` | 6 (48) | `OuterL1V1` | `address_wider_blocks_v1` | `1,809` | `5,349` | `3,540` | `265` | `1,476` | `119` | `668` |
| **`WB6-Env`** | **6 (48)** | **`CellEnvelopeL1V1`** | **`address_wider_blocks_v1`** | **`1,901`** | **`6,302`** | **`4,401`** | **`295`** | **`1,852`** | **`136`** | **`643`** |
| **`WB6-TwoScale`** | **6 (48)** | **`FullVectorCellL1V1`** | **`address_wider_blocks_v1`** | **`1,846`** | **`6,322`** | **`4,476`** | **`295`** | **`1,678`** | **`136`** | **`644`** |
| `WBS6-Env` | 6 (48) | `CellEnvelopeL1V1` | `address_wider_blocks_shuffled_v1_seed_20261019` | `728` | `2,125` | `1,397` | `49` | `1,154` | `19` | `636` |
| `WBS6-TwoScale` | 6 (48) | `FullVectorCellL1V1` | `address_wider_blocks_shuffled_v1_seed_20261019` | `825` | `2,277` | `1,452` | `49` | `1,162` | `19` | `637` |
| `WB8-Outer` | 8 (64) | `OuterL1V1` | `address_wider_blocks_v1` | `1,471` | `4,639` | `3,168` | `223` | `1,686` | `109` | `548` |
| **`WB8-Env`** | **8 (64)** | **`CellEnvelopeL1V1`** | **`address_wider_blocks_v1`** | **`1,965`** | **`6,342`** | **`4,377`** | **`240`** | **`2,103`** | **`112`** | **`527`** |
| **`WB8-TwoScale`** | **8 (64)** | **`FullVectorCellL1V1`** | **`address_wider_blocks_v1`** | **`1,977`** | **`6,353`** | **`4,376`** | **`240`** | **`1,919`** | **`112`** | **`527`** |
| `WBS8-Env` | 8 (64) | `CellEnvelopeL1V1` | `address_wider_blocks_shuffled_v1_seed_20261019` | `722` | `1,953` | `1,231` | `60` | `690` | `39` | `519` |
| `WBS8-TwoScale` | 8 (64) | `FullVectorCellL1V1` | `address_wider_blocks_shuffled_v1_seed_20261019` | `789` | `2,067` | `1,278` | `60` | `698` | `39` | `519` |
| `FV8-B8-Pad-Env` | 8 (64) | `CellEnvelopeL1V1` | `address_full_vector_v1` ($B=8$ XOR-pad) | `1,710` | `5,089` | `3,379` | `209` | `1,028` | `131` | `525` |
| `FV8-B8-Pad-TwoScale` | 8 (64) | `FullVectorCellL1V1` | `address_full_vector_v1` ($B=8$ XOR-pad) | `1,716` | `5,305` | `3,589` | `210` | `1,030` | `131` | `524` |

### 4.1 Paired McNemar Discordance on `P_succ` (`3,847` rows; clears when $b - c > F = \max(24, \lceil 2\sqrt{b + c}\rceil)$)

| Comparison ($X$ vs. $Y$) | $b$ ($X$ only) | $c$ ($Y$ only) | $b - c$ | Floor $F$ | Clears | Interpretation |
|---|---:|---:|---:|---:|:---:|---|
| `WB6-Env` vs. `A8-Outer` | `862` | `312` | `+550` | `69` | `true` | Primary Reach bar at $B = 6$ (`PASS`) |
| `WB6-Env` vs. `WBS6-Env` | `1,609` | `436` | `+1,173` | `91` | `true` | Primary Geometry bar vs. shuffled at $B = 6$ (`PASS`) |
| `WB6-Env` vs. `WB6-Outer` | `336` | `244` | `+92` | `49` | `true` | Pure metric lift on fixed $B = 6$ table (`1,901` vs. `1,809`) |
| `WB6-TwoScale` vs. `A8-Outer` | `803` | `308` | `+495` | `67` | `true` | Primary Reach bar at $B = 6$ (`PASS`) |
| `WB6-TwoScale` vs. `WBS6-TwoScale` | `1,534` | `513` | `+1,021` | `91` | `true` | Primary Geometry bar vs. shuffled at $B = 6$ (`PASS`) |
| `WB6-TwoScale` vs. `WB6-Outer` | `301` | `264` | `+37` | `48` | `false` | Within-table two-scale vs. outer on $B = 6$ (`+37 <= 48`) |
| `WB8-Env` vs. `A8-Outer` | `925` | `311` | `+614` | `71` | `true` | Primary Reach bar at $B = 8$ (`PASS`) |
| `WB8-Env` vs. `WBS8-Env` | `1,627` | `384` | `+1,243` | `90` | `true` | Primary Geometry bar vs. shuffled at $B = 8$ (`PASS`) |
| `WB8-Env` vs. `FV8-B8-Pad-Env` | `457` | `202` | `+255` | `52` | `true` | Multi-pair pull (`P=3`) beats single-pair XOR-pad at $B = 8$ (`1,965` vs. `1,710`) |
| `WB8-Env` vs. `WB8-Outer` | `641` | `147` | `+494` | `57` | `true` | Pure metric lift on fixed $B = 8$ table (`1,965` vs. `1,471`) |
| `WB8-TwoScale` vs. `A8-Outer` | `927` | `301` | `+626` | `71` | `true` | Primary Reach bar at $B = 8$ (`PASS`) |
| `WB8-TwoScale` vs. `WBS8-TwoScale` | `1,594` | `406` | `+1,188` | `90` | `true` | Primary Geometry bar vs. shuffled at $B = 8$ (`PASS`) |
| `WB8-TwoScale` vs. `FV8-B8-Pad-TwoScale` | `458` | `197` | `+261` | `52` | `true` | Multi-pair pull (`P=3`) beats single-pair XOR-pad at $B = 8$ (`1,977` vs. `1,716`) |
| `WB8-TwoScale` vs. `WB8-Outer` | `624` | `118` | `+506` | `55` | `true` | Pure metric lift on fixed $B = 8$ table (`1,977` vs. `1,471`) |
| `WB6-Env` vs. `FV8-B4-Env` | `419` | `498` | `-79` | `61` | `false` | $B = 4$ envelope (`1,980`) exceeds $B = 6$ envelope (`1,901`) on `P_succ` |
| `FV8-B4-Env` vs. `WB8-Env` | `439` | `424` | `+15` | `59` | `false` | $B = 4$ (`1,980`) and $B = 8$ (`1,965`) are statistically tied on `P_succ` |
| `WB6-TwoScale` vs. `FV8-B4-TwoScale` | `492` | `448` | `+44` | `62` | `false` | $B = 6$ (`1,846`) vs. $B = 4$ (`1,802`) under two-scale (`+44 <= 62`) |
| `FV8-B4-TwoScale` vs. `WB8-TwoScale` | `397` | `572` | `-175` | `63` | `false` | `WB8-TwoScale` (`1,977`) beats `FV8-B4-TwoScale` (`1,802`) by `+175 > 63` |

---

## 5. Synthesis: What This Resolves for Stage 1 (`#2847`)

1. **Both $B = 6$ (`WB6-Env`, `WB6-TwoScale`) and $B = 8$ (`WB8-Env`, `WB8-TwoScale`) pass `G-Q`, `G-D11`, `G-Occ` (`within_cell_tie_mass == 0`), `G-Cap` (`6,302..6,353 > 1,941`), Reach vs. `A8-Outer` (`1,846..1,977 > 1,351`), and Geometry vs. shuffled (`b - c = +1,021..+1,243`).**
2. **Multi-pair residual bidirectional pull (`WiderBlocksV1`) is essential at $B > 4$:** At $B = 8$, `FullVectorV1`'s single-pair XOR pad (`FV8-B8-Pad-Env = 1,710`, `Full-DEV = 5,089`, `Top-8 = 1,028`) degrades sharply below $B = 4$ (`1,980`, `5,709`, `1,126`), whereas `WiderBlocksV1` (`WB8-Env = 1,965`, `Full-DEV = 6,342`, `Top-8 = 2,103`) recovers full `P_succ` admission (`+255` over XOR-pad) and lifts full-DEV admission by `+1,253` rows and `Top-8` by `+1,075` hits.
3. **Separating metric lift from table placement:** Because `WB6-Outer` and `WB8-Outer` evaluate the exact same pinned tables (`b6-wider-blocks-v1.json`, `b8-wider-blocks-v1.json`) under `OuterL1V1`, the pure address-metric gain on a fixed table is directly measured: `+92` `P_succ` rows (`+953` full-DEV rows) at $B = 6$, and `+494` to `+506` `P_succ` rows (`+1,703` to `+1,714` full-DEV rows) at $B = 8$.
4. **Within-cell tie mass as an explicit Stage 1 gate:** All wider-carrier tables and the synthesis in [`docs/mechanisms/discrete-learning.md`](../../mechanisms/discrete-learning.md) enforce `within_cell_tie_tokens == 0` and `within_cell_tie_dev_rows == 0` alongside `dev_alias_ceiling_rows > 1,941` (`G-Cap` + `G-Occ`).
