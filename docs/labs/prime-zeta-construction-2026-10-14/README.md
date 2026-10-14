# Prime-Zeta Complex Routing Construction Qualification — 2026-10-14

- **Lab / Session:** `antigravity/ag1`
- **Issues:** [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028), [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032)
- **Pre-registration sheet:** [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md) (`sha256: 26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7`, `TEST FITNESS: FIT`)
- **Mechanism brief:** [`docs/mechanisms/prime-zeta-complex-routing.md`](../../mechanisms/prime-zeta-complex-routing.md)
- **Owner release:** [#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990) — cleared for finite construction and verification qualification after exact-head review, pinned strictly to the `TEST FITNESS: FIT` sheet (zero training, zero corpus dose, zero `.apply(` calls, `f64` confined to one-time host-side offline codebook generation).

---

## 1. Summary

This unit implements the **Prime-Rank Indexing + Complex-Plane Zeta-Phase ($\mathbb{C}^4 \to \mathbb{R}^4$ and $\mathbb{C}^8 \to \mathbb{R}^8$) construction qualification** registered in [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md):

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

Generated via:

```sh
CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-discrete --bin prime-zeta-qualify
./local/harness-target/debug/prime-zeta-qualify host-verify docs/labs/prime-zeta-construction-2026-10-14/receipts
```

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

## 3. Verification & Guardrail Checks

- `cargo fmt --check`: clean.
- `CARGO_TARGET_DIR=local/harness-target cargo clippy -p uor-r4-discrete --bin prime-zeta-qualify --tests -- -D warnings`: clean (`0` warnings in `prime_zeta.rs` and `prime-zeta-qualify.rs`).
- `CARGO_TARGET_DIR=local/harness-target cargo test -p uor-r4-discrete`: **30/30 tests pass** (`25` library tests including `apply_calls_are_confined_to_bounded_fixture_and_admitted_transition` + `5` `prime_zeta` tests).
- `uor-harness ratchet d23` & `uor-harness ratchet d25`: pass.
- **Limits, stated:** This PR delivers the complete implementation, unit tests, and host codebook verification receipts (`host-codebook-qualification.json` and `v4096-witnesses.json`) with **zero GPU execution**. Following exact-head review of this implementation PR, the bounded CUDA construction qualification (`prime-zeta-qualify qualify --features cuda`) is cleared to run under owner release [#2029 comment 6170312990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990). Training and M2 rental holds remain `HELD`.
