# Prime-Zeta Complex Routing Construction Qualification — 2026-10-14

- **Lab / Session:** `antigravity/ag1`
- **Issues:** [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028), [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032)
- **Pre-registration sheet:** [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md) (`sha256: 26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7`, `TEST FITNESS: FIT`)
- **Mechanism brief:** [`docs/mechanisms/prime-zeta-complex-routing.md`](../../mechanisms/prime-zeta-complex-routing.md)
- **Owner release:** [#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990) — cleared for finite construction and verification qualification after exact-head review, pinned strictly to the `TEST FITNESS: FIT` sheet (zero training, zero corpus dose, zero `.apply(` calls, `f64` confined to one-time host-side offline codebook generation).
- **Implementation PR:** [#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473) (merged into `origin/main` at `912a8a0690d0427fac0e4abed4bdc9636841ea05`).

---

## 1. Summary

This unit implements and executes the **Prime-Rank Indexing + Complex-Plane Zeta-Phase ($\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$) construction qualification** registered in [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md):

1. **Host-side one-time offline codebook generator (`crates/uor-r4-discrete/src/prime_zeta.rs`):**
   - Bijective prime atom assignment $r(v) = p_{\text{id}(v)+1}$ (`first_n_primes` up to $V=8192$; $p_1 = 2, p_{256} = 1619, p_{4096} = 38873$).
   - Complex-plane log-prime zeta-phase evaluation $\theta_{b,c}(p) = \gamma_{4b+c} \ln p \pmod{2\pi}$ using the Riemann zeta ordinates $\gamma_k$ from `crates/uor-r4-core/src/zeta_zeros.rs` (`include!`d directly into `prime_zeta.rs` so `uor-r4-core` is never added to `uor-r4-discrete`'s `[dependencies]`).
   - Exact integer quantization boundary:
     - `PhaseQ29` ($i32$): $\Phi_{b,c}(p) = \operatorname{round}\!\left(\frac{\theta_{b,c}^{\text{signed}}(p)}{\pi} \cdot 2^{29}\right) \in [-2^{29}, 2^{29}]$.
     - `UnitS3Q30` ($[i32; 4]$): two $\mathbb{C}^2 \to \mathbb{R}^4$ spinor projections per block scaled by $\frac{1}{\sqrt{2}}$ and quantized to signed Q30 integers with exact norm renormalization.
     - $E_8$ Orbit Anchor Root Quantization (`Snapshot.table`): embeds the 4 complex phasors per block into the 4 orthogonal 2D root planes of `geometry::ROOTS: [[i8; 8]; 16]` (`(0,1), (2,4), (3,7), (5,6)`), scores candidate orbit roots by exact integer `i64` inner product, balances writer cells `(root_0, root_{B-1})` across all `256` cells (`16` tokens/cell at $V=4096$), and assigns unique interior block roots `(r_1, …, r_{B-2})` within each writer cell for $B \ge 3$ so all $4,096$ vocabulary tokens receive strictly unique `Snapshot.table` vectors (`0` full-vector collisions at $V=4096, B=4$).
2. **Matched D22 §3 Null Controls (`CodebookMode::TangentNull` & `CodebookMode::NormalNull`):**
   - **Tangent Null ($\gamma_k \to \gamma_k + \delta$, $\delta = 1.0$):** preserves prime ranks $p_{\text{id}(v)+1}$ and the quantization pipeline while shifting every zeta ordinate by $+1.0$.
   - **Normal Null (Seeded Prime Permutation + Off-Critical-Line Perturbation):** permutes prime assignments $\pi_s(p)$ via deterministic SplitMix64 + Fisher–Yates shuffle per seed $s \in \{0, 1, 2\}$ and applies off-critical-line radial/phase distortion before running the identical quantization boundary.
3. **Qualification Binary (`crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs`):**
   - `prime-zeta-qualify host-verify NEW_REPORT_DIR`: executes deterministic host codebook verification across `seed = 0, 1, 2`, verifies `0` full-vector collisions (`4096/4096` unique vectors on `V4096 B4`, `4096/4096` unique `PhaseQ29`/`UnitS3Q30` vectors and `256/256` saturated orbit cells on `V4096 B2`), verifies structural Hamming/L1/writer-cell separation against both nulls, and writes [`receipts/host-codebook-qualification.json`](receipts/host-codebook-qualification.json) and [`receipts/v4096-witnesses.json`](receipts/v4096-witnesses.json).
   - `prime-zeta-qualify qualify NEW_REPORT_DIR` (`--features cuda`): verifies `0` floating-point/tensor-core instructions in `discrete.ptx`, runs `host-verify`, executes all `17/17` base CUDA controls (`11` Route + `6` Flock), and executes `6/6` Prime-Zeta CUDA controls (`C18..C23`) on `V4096 B4` and `V4096 B2` (`discrete_validate`, `forward`, `evaluate`, `propose`, tangent/normal null separation, cross-seed determinism) with `0` `.apply(` calls.
4. **Zero Impact on Frozen Rung-0 Completion Hashes:**
   - All 10 files hashed by `device::source_sha256()` and `training::driver_source_sha256()` (`cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4`) are **100% untouched**, verified by unit test `frozen_d5_driver_source_hash_is_preserved_untouched`.

---

## 2. Host Codebook Qualification Receipts (`V4096` & `V256`)

Generated identically across both `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu` (`host-codebook-qualification.json` `sha256: 41e828872cb9ef7b3b9cc921481cd916b9258153e9d381210258eb2ec0d3a52c` and `v4096-witnesses.json` `sha256: e7f4c63667e14873b6d03d370d231c353724618bb5d17d5a61c444fc789f1d0d` are **bit-exact across both architectures**):

| Codebook | Vocabulary $V$ | Blocks $B$ | Unique `PhaseQ29` | Unique `UnitS3Q30` | Unique `Snapshot.table` | Occupied Writer Cells | `table_sha256` |
|---|---:|---:|---:|---:|---:|---:|---|
| `v4096_b4_prime_zeta` (`seed 0, 1, 2`) | `4096` | `4` (`32D`) | `4096/4096` | `4096/4096` | **`4096/4096` (`0` collisions)** | `256/256` | `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804` |
| `v4096_b2_prime_zeta` (`seed 0, 1, 2`) | `4096` | `2` (`16D`) | `4096/4096` | `4096/4096` | `256/256` ($16^2$ orbit saturated, quota `16`) | `256/256` | `41dcb6be3962b1e723c4d819eb93b2502fc3ef6135a735bf55ae3fe8a0efd023` |
| `v256_b2_prime_zeta` (`seed 0, 1, 2`) | `256` | `2` (`16D`) | `256/256` | `256/256` | **`256/256` (`0` collisions)** | `256/256` | `bdbe94993d7f27b9a367374df966a007cea699639ac8aae85b3a9a09bce8a4fc` |
| `v4096_b4_tangent_null_delta_1_0` | `4096` | `4` (`32D`) | `4096/4096` | `4096/4096` | **`4096/4096` (`0` collisions)** | `256/256` | `bae145fe4a348ee27f846dbc90075e80559f59a7c4d7931dfc6c58c178c524e1` |
| `v4096_b4_normal_null_seed_0` | `4096` | `4` (`32D`) | `4096/4096` | `4096/4096` | **`4096/4096` (`0` collisions)** | `256/256` | `a1031b64712a0a944921d92971deac44039dc6fa08986c141a47bb3568e6f10e` |
| `v4096_b4_normal_null_seed_1` | `4096` | `4` (`32D`) | `4096/4096` | `4096/4096` | **`4096/4096` (`0` collisions)** | `256/256` | `7c35797e2c64792cad0145b30a955f3f8ffd8d0191213df8a9a645f191fad0fa` |
| `v4096_b4_normal_null_seed_2` | `4096` | `4` (`32D`) | `4096/4096` | `4096/4096` | **`4096/4096` (`0` collisions)** | `256/256` | `8d3e69332bf2974b7748b79d8ddd2541d726ee86d505ed98961e6ae6dd473346` |

### Matched Null Structural Separation (`V4096`)

| Comparison Pair | Differing Tokens ($/4096$) | Differing Root Blocks | Coordinate Hamming Distance | Coordinate L1 Distance | Differing Writer Cells ($/4096$) |
|---|---:|---:|---:|---:|---:|
| `V4096 B4`: Prime-Zeta vs Tangent Null ($\delta = 1.0$) | `4090 / 4096` (`99.85%`) | `16,221 / 16,384` | `55,088` | `123,960` | `4074 / 4096` |
| `V4096 B4`: Prime-Zeta vs Normal Null (`seed 0`) | `4096 / 4096` (`100.00%`) | `15,397 / 16,384` | `53,551` | `115,100` | `4088 / 4096` |
| `V4096 B4`: Prime-Zeta vs Normal Null (`seed 1`) | `4096 / 4096` (`100.00%`) | `15,312 / 16,384` | `53,390` | `114,600` | `4081 / 4096` |
| `V4096 B4`: Prime-Zeta vs Normal Null (`seed 2`) | `4096 / 4096` (`100.00%`) | `15,334 / 16,384` | `53,122` | `114,312` | `4084 / 4096` |
| `V4096 B4`: Tangent Null vs Normal Null (`seed 0`) | `4096 / 4096` (`100.00%`) | `15,408 / 16,384` | `53,269` | `114,836` | `4082 / 4096` |
| `V4096 B2`: Prime-Zeta vs Tangent Null ($\delta = 1.0$) | `4079 / 4096` (`99.58%`) | `8,088 / 8,192` | `27,730` | `62,448` | `4079 / 4096` |
| `V4096 B2`: Prime-Zeta vs Normal Null (`seed 0`) | `4073 / 4096` (`99.44%`) | `7,658 / 8,192` | `26,522` | `57,080` | `4073 / 4096` |

---

## 3. Bounded CUDA Construction Qualification (`device=cuda`, Pod `vast-55945260`)

Executed on merged `origin/main` head `912a8a0690d0427fac0e4abed4bdc9636841ea05` per owner release [#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990):

- **Pod:** `vast-55945260` (`1×NVIDIA GeForce RTX 5090`, driver `580.95.05`, compute capability `12.0`, `32,607 MiB` VRAM, Ubuntu 24.04).
- **Toolchain:** `nvcc 12.8.93` (`V12.8.93`, `UOR_NVCC_ARCH=compute_120`), `rustc 1.97.1 (8bab26f4f 2026-07-14)`.
- **Executable:** `/root/build/target-sm120/release/prime-zeta-qualify` (`sha256: 953f8f085743aab68e9613c1acbbc6d41774bc71be50178fe75b40b05e0655f5`).
- **Compiled Identity (matching frozen D5 completion pins):**
  - `source_sha256`: `e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a`
  - `kernel_sha256`: `931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b`
  - `geometry_sha256`: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`
  - `driver_source_sha256`: `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4`
- **PTX Admission (`admit_ptx`):** `verified_entries: 11`, `zero_float_instructions: true`.
- **Base CUDA Controls (`17/17 PASS`):**
  - Route controls (`11/11 PASS`): [`receipts/route-controls/controls.json`](receipts/route-controls/controls.json) (`sha256: c42c722b58ed7271be76f04430396bb93e816c2b34e3c5cb923ebe51ba6da25e`).
  - Flock controls (`6/6 PASS`): [`receipts/flock-controls/flock-controls.json`](receipts/flock-controls/flock-controls.json) (`sha256: cd4322d4d9a1082297e5f9a6c754c234caa33c0b9aba929801484deff9359932`).
- **Prime-Zeta CUDA Controls (`6/6 PASS`, [`receipts/prime-zeta-qualification.json`](receipts/prime-zeta-qualification.json) `sha256: 082ed664c60000aa842d479f579f4a0a431eda821970cfd243a2df99965e5951`):**
  - `C18` (`prime_zeta_v4096_b4_cuda_snapshot_validation_and_zero_collision`): **`PASS`** (`unique_table_vectors: 4096`, `occupied_writer_cells: 256`, `epoch: 0`).
  - `C19` (`prime_zeta_v4096_b2_cuda_snapshot_validation`): **`PASS`** (`losses: [640, 640, 544, 640]`, `predictions: [0, 2, 210, 13]`, `epoch: 0`).
  - `C20` (`prime_zeta_seed_reproducibility_0_1_2_cuda`): **`PASS`** (`losses: [1280, 1216, 1248, 896]`, `predictions: [3641, 2, 1689, 13]`, `total_loss_numerator_sum: 4640` bit-exact across seeds `0, 1, 2`).
  - `C21` (`prime_zeta_tangent_and_normal_null_cuda_separation`): **`PASS`** (`prime_zeta_losses: [1280, 1216, 1248, 896]`, `tangent_null_losses: [1248, 1088, 1248, 1280]`, `normal_null_seed0_losses: [1248, 1248, 1248, 1216]`, `normal_null_seed1_losses: [1088, 1280, 1248, 1184]`, `normal_null_seed2_losses: [1248, 1280, 896, 1184]`).
  - `C22` (`prime_zeta_route_counterfactual_nonvacuity_v4096`): **`PASS`** (`vector_block0_proposal`: `4640 -> 4288`, `candidate: 1`, `sensitive: true`; `route0_proposal`: `4640 -> 4448`, `candidate: 2`, `sensitive: true`; `epoch: 0`).
  - `C23` (`prime_zeta_flock_read_and_gate_counterfactual_v4096`): **`PASS`** (`flock_predictions: [4095, 97, 1023, 113]`, `gate0_proposal`: `5120 -> 4992`, `candidate: 2`, `sensitive: true`; `epoch: 0`).
- **Training & Corpus Dose:** `applied_updates: 0`, `corpus_rows_consumed: 0`, `training: "HELD"`.
- **Pod Cost & Lifecycle Receipt:**
  - Lease released (`2026-10-14T15:16:09Z`), results archived (`144 KB`), and pod `vast-55945260` destroyed (`2026-10-14T15:16:28Z`, `uptime_s: 829` at `$0.563/h` = **`$0.1297` total spend**).

---

## 4. Receipt Manifest ([`receipts/SHA256SUMS`](receipts/SHA256SUMS))

```text
41e828872cb9ef7b3b9cc921481cd916b9258153e9d381210258eb2ec0d3a52c  host-codebook-qualification.json
e7f4c63667e14873b6d03d370d231c353724618bb5d17d5a61c444fc789f1d0d  v4096-witnesses.json
082ed664c60000aa842d479f579f4a0a431eda821970cfd243a2df99965e5951  prime-zeta-qualification.json
c42c722b58ed7271be76f04430396bb93e816c2b34e3c5cb923ebe51ba6da25e  route-controls/controls.json
cd4322d4d9a1082297e5f9a6c754c234caa33c0b9aba929801484deff9359932  flock-controls/flock-controls.json
04eaa4f81c2ba366ea494c49d3a492b065588f00fc69de416ad4182d6424ead6  cuda-qualify-job.log
```
