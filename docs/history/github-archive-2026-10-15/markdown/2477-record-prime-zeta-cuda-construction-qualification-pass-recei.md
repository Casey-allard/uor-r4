# #2477 — Record prime-zeta CUDA construction qualification PASS receipts and pod teardown

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T15:22:22Z
- **Updated:** 2026-10-14T15:29:14Z
- **Closed:** 2026-10-14T15:27:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2477
- **Merged:** 2026-10-14T15:27:19Z
- **Merge commit:** 9048df01b20d57ef7312e117a687f056f2a0da9b
- **Base / Head:** main / UOR-Foundation:antigravity/ag1-prime-zeta-cuda-receipts
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Archives the bounded CUDA construction qualification receipts for **Prime-Rank Indexing + Complex-Plane Zeta-Phase ($\mathbb{C}^4 \to \mathbb{R}^4$ / $\mathbb{C}^8 \to \mathbb{R}^8$)** executed on pod `vast-55945260` (`1×RTX 5090`, driver `580.95.05`, `nvcc 12.8.93`, `UOR_NVCC_ARCH=compute_120`, `rustc 1.97.1`) over merged `origin/main` head `912a8a0690d0427fac0e4abed4bdc9636841ea05` per owner release [`#2029` comment `6170312990`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990):

1. **Zero-Float PTX Admission (`admit_ptx`):** `verified_entries: 11`, `zero_float_instructions: true`.
2. **Base CUDA Controls (`17/17 PASS`):**
   - Route controls (`11/11 PASS`): [`docs/labs/prime-zeta-construction-2026-10-14/receipts/route-controls/controls.json`](docs/labs/prime-zeta-construction-2026-10-14/receipts/route-controls/controls.json) (`sha256: c42c722b58ed7271be76f04430396bb93e816c2b34e3c5cb923ebe51ba6da25e`).
   - Flock controls (`6/6 PASS`): [`docs/labs/prime-zeta-construction-2026-10-14/receipts/flock-controls/flock-controls.json`](docs/labs/prime-zeta-construction-2026-10-14/receipts/flock-controls/flock-controls.json) (`sha256: cd4322d4d9a1082297e5f9a6c754c234caa33c0b9aba929801484deff9359932`).
3. **Prime-Zeta CUDA Controls (`6/6 PASS`, [`prime-zeta-qualification.json`](docs/labs/prime-zeta-construction-2026-10-14/receipts/prime-zeta-qualification.json) `sha256: 082ed664c60000aa842d479f579f4a0a431eda821970cfd243a2df99965e5951`):**
   - `C18` (`prime_zeta_v4096_b4_cuda_snapshot_validation_and_zero_collision`): `PASS` (`unique_table_vectors: 4096`, `occupied_writer_cells: 256`, `epoch: 0`).
   - `C19` (`prime_zeta_v4096_b2_cuda_snapshot_validation`): `PASS` (`losses: [640, 640, 544, 640]`, `predictions: [0, 2, 210, 13]`, `epoch: 0`).
   - `C20` (`prime_zeta_seed_reproducibility_0_1_2_cuda`): `PASS` (`losses: [1280, 1216, 1248, 896]`, `predictions: [3641, 2, 1689, 13]`, `total_loss_numerator_sum: 4640` bit-exact across seeds `0, 1, 2`).
   - `C21` (`prime_zeta_tangent_and_normal_null_cuda_separation`): `PASS` (`prime_zeta_losses: [1280, 1216, 1248, 896]`, `tangent_null_losses: [1248, 1088, 1248, 1280]`, `normal_null_seed0_losses: [1248, 1248, 1248, 1216]`, `normal_null_seed1_losses: [1088, 1280, 1248, 1184]`, `normal_null_seed2_losses: [1248, 1280, 896, 1184]`).
   - `C22` (`prime_zeta_route_counterfactual_nonvacuity_v4096`): `PASS` (`vector_block0_proposal`: `4640 -> 4288`, `candidate: 1`, `sensitive: true`; `route0_proposal`: `4640 -> 4448`, `candidate: 2`, `sensitive: true`).
   - `C23` (`prime_zeta_flock_read_and_gate_counterfactual_v4096`): `PASS` (`flock_predictions: [4095, 97, 1023, 113]`, `gate0_proposal`: `5120 -> 4992`, `candidate: 2`, `sensitive: true`).
4. **Cross-Architecture Host Codebook Bit-Exactness:** `host-codebook-qualification.json` (`41e82887…`) and `v4096-witnesses.json` (`e7f4c636…`) generated on `x86_64-unknown-linux-gnu` (`vast-55945260`) are 100% byte-identical to `aarch64-apple-darwin`.
5. **Pod Lifecycle & Spend:** `applied_updates: 0`, `corpus_rows_consumed: 0`, `training: "HELD"`. Lease released and pod `vast-55945260` destroyed after `829s` at `$0.563/h` (`$0.1297` total spend).

Line: m1-prime-zeta-complex-routing · count 0/3 unchanged (construction qualification, zero training) · headline 37/232 → 37/232 (unchanged).

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6171805786 — 2026-10-14T15:22:24Z — Casey-allard

**Antigravity (`antigravity/ag1`) exact-head review: `APPROVE` at `824ae4b074b4f2e24a72b5369aa3aad95a16a800`.** Receipts-only PR archiving the CUDA construction qualification outputs (`prime-zeta-qualification.json` `082ed664…`, `route-controls/controls.json` `c42c722b…`, `flock-controls/flock-controls.json` `cd4322d4…`, `cuda-qualify-job.log` `04eaa4f8…`, `SHA256SUMS`). Zero `crates/` changes; frozen D5/completion hashes verified (`source_sha256: e1836149…`, `kernel_sha256: 931a511b…`, `geometry_sha256: 46ddbb76…`, `driver_source_sha256: cb090e85…`); pod `vast-55945260` released and destroyed (`829s`, `$0.1297`). Proceeding through protected merge.

### comment 6171878660 — 2026-10-14T15:25:51Z — Casey-allard

**Antigravity (`antigravity/ag1`) exact-head review: `APPROVE` at `f86b25742ef9154c27184ebbe5375c4fcbffd0e3`.** Merged `origin/main` (`334eac5be`) cleanly; receipts-only PR (`prime-zeta-qualification.json` `082ed664…`, `route-controls/controls.json` `c42c722b…`, `flock-controls/flock-controls.json` `cd4322d4…`, `cuda-qualify-job.log` `04eaa4f8…`, `SHA256SUMS`). Proceeding through protected merge.


---
_Comment/review ids in this file: 6171805786, 6171878660_

