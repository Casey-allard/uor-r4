# Prime-zeta codebook vs random-injective and quota-only-injective nulls at zero updates (`r-prime-zeta-vs-nulls`)

- **Lab / Task:** `antigravity` (`antigravity-27`, `antigravity-28`, `antigravity-36`; non-author audit by `kimi` in `kimi-25`)
- **Roadmap item:** [`docs/roadmap/items/r-prime-zeta-vs-nulls.md`](../../roadmap/items/r-prime-zeta-vs-nulls.md) (Issue [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881), Milestone M1 [`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029))
- **Design note & decision card:** [`#2881` comment `6353762758`](https://github.com/UOR-Foundation/uor-r4/issues/2881#issuecomment-6353762758) (cross-linked on [`#2029` comment `6353766545`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6353766545))
- **Pre-registration sheet:** [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) (`TEST FITNESS: NOT_FIT` from `kimi-25` `#3067`; prerequisite `(b)` total outcome rule resolved in `antigravity-36`; awaits only prerequisite `(a)`, owner/PM registration of the comparison bar and `~$0.20` pod spend on `#2881`)
- **Source modules:** [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs), [`crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`](../../../crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs)
- **Status:** Steps 1–4 of [`r-prime-zeta-vs-nulls.md`](../../roadmap/items/r-prime-zeta-vs-nulls.md) complete and host-verified by Kimi (`kimi-25`) and DeepSeek (`#3067`: `7 / 7` tests in `--bin prime-zeta-codebook`, all 7 table SHAs regenerated bit-exact on host, zero added `d32`/`d23`/`d25` ratchet sites, total outcome rule in `## Bar`). Step 5 (zero-update GPU battery on RTX 5090, `~$0.20`) awaits owner/PM bar registration and pod-spend approval on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881) followed by the final `TEST FITNESS: FIT` stamp.

---

## 1. Construction of the Two Pure-Integer Injective Nulls

As noted on the [prime-zeta codebook mechanism page](../../mechanisms/prime-zeta-codebook.md) (line 38), the two existing controls in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs) (`CodebookMode::TangentNull` and `CodebookMode::NormalNull`) still evaluate Riemann zeta phases and run the same `16`-per-cell writer-cell quota pass and interior-block uniqueness pass as `CodebookMode::PrimeZeta`.

To test whether `PrimeZeta`'s log-prime / zeta-zero phases ($\theta_k(p) = \gamma_k \ln p$) contribute anything at zero updates beyond full-vector injectivity (`4,096 / 4,096` distinct vectors) and flat cell balancing (`256` cells $\times$ `16` tokens), [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs) adds two pure-integer modes to `CodebookMode`, exported by [`crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`](../../../crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs):

1. **`CodebookMode::QuotaOnlyInjective { seed: u32 }` (`quota_only_injective:SEED`):**
   - Places all $V = 4,096$ vocabulary tokens across the $256$ writer cells $c = (r_0, r_{B-1}) \in \{0..16\} \times \{0..16\}$ with the exact same flat quota of `16` tokens per cell as `PrimeZeta`, using a deterministic seeded SplitMix64 (`next_seeded_u64`) Fisher–Yates shuffle of the quota-balanced multiset `[0..256; 16]`.
   - Within each writer cell $c$, assigns each of the `16` tokens a strictly unique interior block root pair $(r_1, r_2) \in \{0..16\}^2$ via a per-cell seeded SplitMix64 Fisher–Yates shuffle.
   - Uses **zero** prime numbers and **zero** zeta-zero ordinates for placement, and executes in 100% integer arithmetic over `geometry::ROOTS: [[i8; 8]; 16]`.
2. **`CodebookMode::RandomInjective { seed: u32 }` (`random_injective:SEED`):**
   - Draws $V = 4,096$ strictly distinct $B$-block root tuples $(r_0, \dots, r_{B-1}) \in \{0..16\}^B$ uniformly without replacement from the $16^4 = 65,536$ orbit space using `next_seeded_u64`, with **no writer-cell quota**.
   - Writer-cell counts follow natural multinomial dispersion around mean `16` (`min_cell_occupancy = 3..8`, `max_cell_occupancy = 26..27` across seeds `0, 1, 2`).

### Constraint Audit (`D23`, `D24`, `D25`, `D32`)

- **Zero new floating-point sites (`D32`):** `uor-harness ratchet d32` remains at the frozen baseline (`31` sites in `prime_zeta.rs`, `0` added).
- **Zero softmax (`D23`) and zero matmul (`D25`):** `uor-harness ratchet d23` and `uor-harness ratchet d25` report `0` added sites.
- **Unchanged driver source hash:** `uor_r4_discrete::training::driver_source_sha256()` remains at `b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24`.

---

## 2. Proof: Invariance of Raw `coverage_census` Alias Ceiling Across Injective Codebooks

In [`crates/uor-r4-discrete/src/coverage_census.rs`](../../../crates/uor-r4-discrete/src/coverage_census.rs) (lines 34–87), `alias_representatives(table, spec)` partitions vocabulary tokens into equivalence classes of identical full $32$-integer vectors and names each class by its lowest token ID `representative[t]`.
- Whenever a codebook is injective (`distinct_vectors == 4,096`), every equivalence class is a singleton (`representative[t] == t` for all $t \in \{0, \dots, 4095\}$).
- Therefore, `within_cell_tie_mass.tokens_in_alias_groups == 0`, `within_cell_tie_mass.dev_rows_targeting_non_representative == 0`, and `dev_alias_ceiling_rows == 14,287` for **every** injective codebook (`PrimeZeta`, `TangentNull`, `NormalNull`, `QuotaOnlyInjective`, `RandomInjective`), verified by unit test `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs).
- Consequently, raw `dev_alias_ceiling_rows` cannot separate two injective codebooks. What separates them at zero updates are **writer-cell crowding capacity** (`cell_collision_pairs`, `max_cell_occupancy`) and **GPU reader-admitted target reach** (`B3_init(P_succ)`, `B3_full`, `Net G-Cap`, and checkpoint-0 `top1_correct` / `top8_correct`).

---

## 3. Host Verification & Pinned Codebook Digests (`V4096 B4 C192 K8`)

Exported via `prime-zeta-codebook` on [`evidence/spec-v4096-b4-c192-k8.json`](evidence/spec-v4096-b4-c192-k8.json):

| Arm | CLI Mode | `table_sha256` | `file_sha256` | Unique Vectors | Occupied Cells | Min / Max Cell | Cell Collision Pairs |
|---|---|---|---|---:|---:|---:|---:|
| `P8` | `prime_zeta` | `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804` | `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q0` | `quota_only_injective:0` | `0098d4c3df8ce101e4a2dda8d8d28a29ac64f6ed3d521430504d0bb03ffbc18f` | `33b4e459cef8303f9d502f6f3e66327ec9e045026e7c8de896ad3ff350c2c415` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q1` | `quota_only_injective:1` | `13a76c718e7e34f3f0d912a8fc8239b02426c5cf1c54930c3950a0cb59054e44` | `57225bbb63387d9a6f39e7cb6415aa0e317d19534633bb731a3d7559efd4c3b9` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q2` | `quota_only_injective:2` | `279d72c26c0cded7340862b51abbdf938af3ee914e9474c7f528fb16d4d9db50` | `e03d6aef42608e1db3f4d33a0b034fd2c113cd9c48f366e0b6319d20d4f77175` | `4,096` | `256` | `16 / 16` | `30,720` |
| `R0` | `random_injective:0` | `35c8da6250a5d986ffa5f6758cbf761b45af6fa24d3893114b9693c1b0d9ee37` | `229e93f58ad36cb3f1fc2fa4ab194ab8ce06798a67e39537352b9817cc5df345` | `4,096` | `256` | `8 / 27` | `32,493` |
| `R1` | `random_injective:1` | `337a72bc3f5187125737243e4e066346669f1955a38d799b5447abc4a3eb77c6` | `36fec6323ae1daf15d965ff5cd1161f66b7a4a8a8b0a3261e9094e9c125a9468` | `4,096` | `256` | `3 / 27` | `32,802` |
| `R2` | `random_injective:2` | `21809edac362e40ea39702777c5cc48648cbe6cf2615599ed33ab8bf3103dfdb` | `25ec9bd29ce7fec20721826bab93f46caed8720cda6387cd43c5bddc62f0edea` | `4,096` | `256` | `8 / 26` | `32,659` |

---

## 4. Staged Zero-Update GPU Battery & Staged Notebook Row

### Staged Evidence Files ([`evidence/SHA256SUMS`](evidence/SHA256SUMS))

- [`evidence/p8.json`](evidence/p8.json): `PrimeZeta` arm config (`83c1380a…`).
- [`evidence/q0.json`](evidence/q0.json), [`evidence/q1.json`](evidence/q1.json), [`evidence/q2.json`](evidence/q2.json): `QuotaOnlyInjective` arm configs for seeds `0, 1, 2`.
- [`evidence/r0.json`](evidence/r0.json), [`evidence/r1.json`](evidence/r1.json), [`evidence/r2.json`](evidence/r2.json): `RandomInjective` arm configs for seeds `0, 1, 2`.
- [`evidence/spec-v4096-b4-c192-k8.json`](evidence/spec-v4096-b4-c192-k8.json): `Spec` JSON input to `prime-zeta-codebook`.
- [`evidence/pz-nulls-battery.sh`](evidence/pz-nulls-battery.sh): 7-arm zero-update pod battery script (`init-corpus`, `eval-corpus`, `attention-dev-b3`, `attention-export-token-cells`, `coverage-census --token-cells`).

### Staged Notebook Row for [`docs/research/notebook/m1-prime-zeta.md`](../../research/notebook/m1-prime-zeta.md) (to backfill after GPU battery execution)

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| `<YYYY-MM-DD>` | Zero-update DEV comparison (`14,287` rows, `3,847` `P_succ`, `K = 8`) of `PrimeZeta` (`P8`) against pure-integer `QuotaOnlyInjective` (`Q0..Q2`, `256 × 16` cell quota) and `RandomInjective` (`R0..R2`, no cell quota) across seeds `0, 1, 2` (`0` updates applied). | Capacity: all 7 arms `4,096 / 4,096` distinct vectors, `14,287 / 14,287` alias ceiling, `0` tie mass; `P8` and `Q0..Q2` `30,720` cell collision pairs (`16 / 16` per cell) vs `R0..R2` `32,493–32,802` pairs (`min 3..8, max 26..27`). Zero-update reach (`P_succ` / full): `P8` `127 / 3,847` (`427 / 14,287`), `Q0..Q2` `<q_psucc> / 3,847` (`<q_full> / 14,287`), `R0..R2` `<r_psucc> / 3,847` (`<r_full> / 14,287`). | `P8` exactness (`427 / 127 / 38`); one-sided paired McNemar discordance $b - c > F = \max(24, 2\sqrt{b+c})$ across all 3 seeds (`PROPOSED, NOT REGISTERED` until confirmed on `#2881`). | `<positive | negative | inconclusive>` (for zero-update K=8 configuration under D22; trained attribution remains held) | [`r-prime-zeta-vs-nulls`](README.md) |
