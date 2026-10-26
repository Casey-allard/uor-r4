# Bucketed CUDA writer: CSR root-pair index qualification (`m1-faster-writer`)

Item [`m1-faster-writer`](../../roadmap/items/m1-faster-writer.md) (`#2840`). Lab: `antigravity`. Order: [`antigravity-90`](../../loop/tasks/m1-faster-writer/antigravity-90/ORDER.md). Decision card: [`DECISION.md`](../../loop/tasks/m1-faster-writer/antigravity-90/DECISION.md) (`D68`). Sheet: [`m1-faster-writer-2026-10-26.md`](../../harness/prep/m1-faster-writer-2026-10-26.md) (`TEST FITNESS: FIT`, `D68`, and `D67` pre-run review at `#2840`).

**Result: `PASSED` (`b1_parity_pass = true`, `b2_comparison_reduction_pass = true`, `negative_controls_pass = true`) across two independent `sm_120` builds (`Build A` and `Build B`) on `1x NVIDIA GeForce RTX 5090` (`vast-57796936`, driver `595.84`, `$0.32`).**

Replacing the full-vocabulary scan in `writer` (`discrete.cu:307-345`) with a GPU-resident 256-cell Compressed Sparse Row (CSR) bucket index (`bucket.ptx` / `bucket.cubin`, 7 admitted `.entry` kernels) eliminates per-row `token_cell` scans (`36,865 -> 0` at `V = 4,096, K = 8`; `147,457 -> 0` at `V = 16,384, K = 8`), caps per-row cell-distance evaluations at `256` (`2,277 -> 256` at `K = 8`; `4,233 -> 256` at `K = 16`), and walks only tokens inside the `K` admitted address buckets, with **`0` output mismatches** across all `5` control groups and **`4 / 4` planted negative-control faults caught**.

This is a finite algorithmic and CUDA kernel qualification at `0` training updates. It establishes exact parity and per-row comparison reduction for `OuterL1V1` (`s[15] == 0`), closes `#2840`, and unblocks `#2887` (`r-vocab-4096-vs-16384`). It is not a trained language capability result.

## 1. Headline throughput (`tokens/s`) and comparison counts (`B1` & `B2`)

### 1.1 Real-read `14,287`-row `DEV` panel (`V = 4,096, B = 4, C = 192, K = 8`, `Relation` `ActiveLeafV1`, `256` occupied cells, `16` tokens/cell)

Scored on the canonical `14,287` eligible response targets (`42,856` payload tokens, `224` batches of up to `64` causal windows of length `C = 192`) on both pinned `A8` (`address-structured-v1.json`, `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) and pinned `PrimeZeta` (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`):

| Codebook (`n = 14,287`) | Mode | `token_cell` / row | `cell_distance` / row | `token_distance` / row | Total evals / row | Reduction vs full scan | Direct writer `rows/s` (Build A / B) | End-to-end `forward` `rows/s` (Build A / B) | End-to-end `evaluate` `rows/s` (Build A / B) | Mismatches (`fwd` / `eval` / `direct`) | `top-1` / `top-8` |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **`A8`** (`6600bdb1…`) | Full-scan `writer` | `36,865` | `2,277` | `128` | `39,270` | `1.00x` | `1,790` / `1,790` | `925` / `925` | `881` / `881` | baseline | `133` / `513` |
| **`A8`** (`6600bdb1…`) | Bucket `IdOrder` | **`0`** | **`256`** | **`128`** | **`384`** | **`102.26x`** (`10,226`) | **`81,533` / `81,537`** (`45.5x`) | **`1,552` / `1,554`** (`1.68x`) | **`1,549` / `1,551`** (`1.76x`) | **`0 / 0 / 0`** | `133` / `513` |
| **`A8`** (`6600bdb1…`) | Bucket `ServingPruned` | **`0`** | **`256`** | **`16`** | **`272`** | **`144.37x`** (`14,437`) | **`106,648` / `106,871`** (`59.6x`) | — | — | **`0`** | `133` / — |
| **`PrimeZeta`** (`83c1380a…`) | Full-scan `writer` | `36,865` | `2,277` | `128` | `39,270` | `1.00x` | `1,762` / `1,762` | `924` / `925` | `876` / `878` | baseline | `38` / `95` |
| **`PrimeZeta`** (`83c1380a…`) | Bucket `IdOrder` | **`0`** | **`256`** | **`128`** | **`384`** | **`102.26x`** (`10,226`) | **`78,677` / `78,553`** (`44.7x`) | **`1,563` / `1,566`** (`1.69x`) | **`1,559` / `1,563`** (`1.78x`) | **`0 / 0 / 0`** | `38` / `95` |
| **`PrimeZeta`** (`83c1380a…`) | Bucket `ServingPruned` | **`0`** | **`256`** | **`83`** | **`339`** | **`115.81x`** (`11,581`) | **`84,832` / `84,602`** (`48.1x`) | — | — | **`0`** | `38` / — |

Across the `14,287`-row `DEV` panel, total writer evaluations drop from `561,050,490` (`39,270 / row`) to `5,486,208` (`384 / row`, `IdOrder`) and `3,886,064` (`272 / row`, `ServingPruned` on `A8`). CSR index construction (`discrete_bucket_build`) on `V = 4,096` takes `657–663 us` once per codebook epoch.

### 1.2 Direct GPU writer sweep across `V ∈ {64, 256, 4096, 16384}` (`72` seeded cases = `2,304` rows + `36` virtual-move cases + uniform `V = 4,096` and `V = 16,384`)

| Scale (`B = 4`) | `K` | Max `|B[c]|` | Full-scan evals / row (`tc + cd + td`) | `IdOrder` evals / row (`0 + 256 + td`) | `IdOrder` reduction | `ServingPruned` evals / row | `ServingPruned` reduction | Full-scan `rows/s` (Build A / B) | `IdOrder` `rows/s` (Build A / B) | `ServingPruned` `rows/s` (Build A / B) | Mismatches |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Seeded `V = 64` | `8` | `12` | `828` (`577 + 228 + 23`) | `279` (`0 + 256 + 23`) | `2.96x` | `273` (`0 + 256 + 17`) | `3.03x` | `31,199` / `31,057` | `127,320` / `125,326` | `128,686` / `127,320` | `0` |
| Seeded `V = 256` | `8` | `28` | `2,653` (`2,305 + 267 + 81`) | `337` (`0 + 256 + 81`) | `7.86x` | `300` (`0 + 256 + 44`) | `8.84x` | `11,147` / `11,091` | `88,397` / `87,992` | `98,969` / `98,360` | `0` |
| Seeded `V = 4,096` | `1` | `46` | `8,721` (`8,193 + 513 + 15`) | `271` (`0 + 256 + 15`) | `32.08x` | `271` (`0 + 256 + 15`) | `32.08x` | `2,951` / `2,943` | `115,523` / `111,627` | `118,081` / `118,081` | `0` |
| Seeded `V = 4,096` | `8` | `36` | `39,268` (`36,865 + 2,277 + 126`) | `382` (`0 + 256 + 126`) | **`102.74x`** | `365` (`0 + 256 + 109`) | **`107.53x`** | `855` / `853` | `47,904` / `47,151` | `51,282` / `51,336` | `0` |
| Seeded `V = 4,096` | `16` | `46` | `74,128` (`69,633 + 4,233 + 262`) | `518` (`0 + 256 + 262`) | **`143.10x`** | `443` (`0 + 256 + 187`) | **`167.06x`** | `479` / `477` | `28,941` / `28,751` | `31,840` / `31,714` | `0` |
| Uniform `V = 4,096` | `8` | `16` | `39,270` (`36,865 + 2,277 + 128`) | `384` (`0 + 256 + 128`) | **`102.26x`** | — | — | — | — | — | `0` |
| Seeded `V = 16,384` | `1` | `105` | `33,344` (`32,769 + 513 + 62`) | `318` (`0 + 256 + 62`) | **`104.54x`** | `318` (`0 + 256 + 62`) | **`104.54x`** | `756` / `754` | `84,063` / `83,478` | `86,486` / `85,637` | `0` |
| Seeded `V = 16,384` | `8` | `100` | `150,243` (`147,457 + 2,277 + 509`) | `765` (`0 + 256 + 509`) | **`196.15x`** | `592` (`0 + 256 + 336`) | **`253.50x`** | `219` / `218` | `22,977` / `22,884` (`104.9x`) | `26,936` / `26,845` (`123.0x`) | `0` |
| Seeded `V = 16,384` | `16` | `107` | `283,788` (`278,529 + 4,233 + 1,026`) | `1,282` (`0 + 256 + 1,026`) | **`221.28x`** | `666` (`0 + 256 + 410`) | **`425.53x`** | `122` / `122` | `13,038` / `12,979` (`106.9x`) | `16,689` / `16,655` (`136.8x`) | `0` |
| Uniform `V = 16,384` | `8` | `64` | `150,246` (`147,457 + 2,277 + 512`) | `768` (`0 + 256 + 512`) | **`195.63x`** | — | — | — | — | — | `0` |

### 1.3 Seeded `Device` end-to-end parity (`forward`, `evaluate`, `propose`)

Across all `12` seeded `Device` models (`V ∈ {64, 256, 4096}`, `B = 4, C = 8, K ∈ {8, 16}`, `Flock` `ActiveLeafV1`, both `NearestAddressMarginV1` and `OccupiedCellRankHingeV1`), `forward` (`16` rows), `evaluate` (`16` rows), and `propose` (`5` parameters: `VectorBlock` at block `0`, `1`, `B - 1`, plus `Gate(0)` and `Gate(1)`) matched the full-scan `Device` with **`0` mismatches**:
- At `V = 4,096, K = 8` (seeds `31`, `32`), `5`-parameter `propose` dropped from `4,766,257`–`4,771,599 us` to `218,513`–`219,428 us` (**`21.8x` faster**), `forward` dropped from `34,164`–`34,374 us` to `1,624`–`1,632 us` (**`21.0x` faster**), and `evaluate` dropped from `39,169`–`39,388 us` to `1,780`–`1,781 us` (**`22.0x` faster**).
- At `V = 4,096, K = 16` (seeds `33`, `34`), `5`-parameter `propose` dropped from `9,014,276`–`9,015,570 us` to `422,170`–`422,835 us` (**`21.3x` faster**), `forward` dropped from `60,133`–`61,422 us` to `2,502`–`2,511 us` (**`24.0x–24.5x` faster**), and `evaluate` dropped from `64,988`–`66,330 us` to `2,706 us` (**`24.0x–24.5x` faster**).

### 1.4 Edge cases, `64`-step incremental CSR update (`B2e`), and planted faults (`B3`)

- **Edge cases (`0` mismatches):**
  - Single-cell collapse (`V = 256, B = 3, K = 8`, all `256` tokens in cell `0`): `0` mismatches, `max_bucket_size = 256`, `token_distance_evals = 256`;
  - Fewer than `K` occupied cells (`3` occupied cells at `K = 8`): `0` mismatches, `admitted_count = 3`;
  - `ABSENT` non-root token refusal (`10` cases across `NO_MOVE` and `4` virtual moves in `IdOrder` and `ServingPruned`): `absent_count == 1`, `status == 8`, `prediction == ABSENT`, `0` mismatches;
  - Constructed cell-admission ties and same-cell / cross-cell token ties (`K ∈ {1, 2, 16}`): `0` mismatches;
  - Virtual `kind == 1` moves into an admitted cell, out of an admitted cell leaving a member behind, and out of an admitted cell emptying it: `0` mismatches, `token_cell_calls == 1`.
- **`64`-step sequential committed move loop (`B2e`):**
  - On `(V = 256, B = 4, C = 8, K = 8)` across `64` sequential steps of `commit_proposal_in_qualification` (`discrete_apply`) + `discrete_bucket_update`: `8` accepted commits, `8` cell changes, **`0` CSR mismatches** against a fresh `discrete_bucket_build` after every step `1..=64`, **`0` writer mismatches** on `16` query rows after every step, `max_entries_written = 338 <= 770` (`2 * V + 258`), and total GPU update time `2,204 us` (Build A) / `2,209 us` (Build B) (`~34.5 us/step`).
- **Four planted negative-control faults (`B3`, `4 / 4` caught on GPU):**
  1. `FAULT_STALE_BUCKET (1)` (`stale_bucket_after_move`): `mismatches_detected = 1`, `caught = true`;
  2. `FAULT_REVERSED_TIE (2)` (`reversed_tie_break`): `mismatches_detected = 1`, `caught = true`;
  3. `FAULT_OVER_PRUNED (3)` (`over_aggressive_serving_prune`): `mismatches_detected = 1`, `caught = true`;
  4. `FAULT_CORRUPTED_CSR (4)` (`corrupted_csr_offset`): `mismatches_detected = 1`, `caught = true`.

## 2. Kernel resource report (`sm_120`, `ptxas`)

Compiled ahead of time by `nvcc -ptx -std=c++17 -arch=compute_120` and `nvcc -cubin -arch=sm_120 --resource-usage` in `crates/uor-r4-discrete/build.rs` with zero float/squashing/matmul instructions (`admit_ptx_inventory(&bucket_source::ENTRIES)`):

| Kernel `.entry` | Registers | Stack frame (B) | Spill stores / loads (B) | Barriers |
|---|---|---|---|---|
| `discrete_bucket_build` | `40` | `1,024` | `0 / 0` | `1` |
| `discrete_bucket_update` | `40` | `0` | `0 / 0` | `0` |
| `discrete_bucket_writer_eval` | `64` | `2,816` | `0 / 0` | `0` |
| `discrete_fullscan_writer_eval` (baseline) | `255` | `4,912` | `344 / 180` | `0` |
| `discrete_bucket_forward` | `76` | `22,080` | `0 / 0` | `0` |
| `discrete_bucket_counterfactual` | `96` | `22,560` | `0 / 0` | `0` |
| `discrete_bucket_evaluate` | `75` | `22,144` | `0 / 0` | `0` |

## 3. Artifacts and digests

Archived in the HF store `caseyallard/uor-r4-store` under `results/antigravity/m1-faster-writer-2026-10-26/` and retained locally in this directory:

| Artifact | SHA-256 |
|---|---|
| [`qualification-build-a.json`](qualification-build-a.json) | `37419bdc303b79ac4beac0fab0ea292d74dc34ac9623754457e9107d89a40fd0` |
| [`qualification-build-b.json`](qualification-build-b.json) | `df583e45e1ca857cb698faf23c5a266fdc6abd3073be5f897662e432c82262bb` |
| `bucket-build-a.ptx` / `bucket-build-b.ptx` (identical) | `e53dbb16cdbbf0a3c7d90e4df17db9934141c0b55949fd0a3d15b2f490410a41` |
| `bucket-build-a.cubin` / `bucket-build-b.cubin` (identical) | `22e82fe4eb36e8e6e8f54b8062c40ac6bcea81d55685a7e2ad7e3e427c23c556` |
| `bucket.cu` (`bucket_image.source_sha256`, identical) | `7222ef058a99178fb7080ba8ac64d52d15d3fb078c098d606e8d3ebd78620962` |
| `driver_source_sha256` (unchanged) | `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` |
| `ordinary.cubin` (`ordinary_sha256`, unchanged) | `cf8118c0fa92278a67222b8b05dbee5b0af0c62720440fe7f44b95759aa97ad2` |
| `bin-a/writer-bucket-qualify` | `64b37774c87ec952b8edcb05a9ea6b6129678bd39bee26908f0c9c712ae9e63c` |
| `bin-b/writer-bucket-qualify` | `0bc9d5f81bd10677dffb7eb2bc6599a3ab60e2a99be2ed99f3e737b4e4684288` |
| `results.tar` (HF store archive bundle) | `573fe6ea4b9df3c608ef9cd53332591d220619804d72076352b820ac5feb9ebf` |

## 4. Reproduce and verify

```sh
# Host verification of the retained Build A & Build B receipts and kernel generation:
CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --no-fail-fast

# Ahead-of-time CUDA build and qualification on an RTX 5090 (compute_120 / sm_120):
UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target-sm120 ~/.cargo/bin/cargo build \
  --release --locked -p uor-r4-discrete --features cuda --bin writer-bucket-qualify --bin prime-zeta-codebook
./local/target-sm120/release/prime-zeta-codebook \
  docs/labs/r-prime-zeta-vs-nulls/evidence/spec-v4096-b4-c192-k8.json 0 /tmp/p8.json prime_zeta
./local/target-sm120/release/writer-bucket-qualify \
  <ft/dev/tokens.u16> <ft/dev/response_mask.u8> <a8-rebuilt.json> /tmp/p8.json /tmp/qualification.json
```
