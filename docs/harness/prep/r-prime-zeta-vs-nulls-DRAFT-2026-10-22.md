# Prime-zeta codebook against random-injective and quota-only-injective nulls at zero updates — DRAFT

- **Status:** DRAFT, zero-update pre-flight only (`PROPOSED, NOT REGISTERED`).
- **Line:** M1 prime-zeta starting codebook ([`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029), item [`r-prime-zeta-vs-nulls`](../../roadmap/items/r-prime-zeta-vs-nulls.md), issue [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881)). **Lab:** antigravity.
- **Design note & decision card:** [`#2881` comment `6353762758`](https://github.com/UOR-Foundation/uor-r4/issues/2881#issuecomment-6353762758) (cross-linked on [`#2029` comment `6353766545`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6353766545)).
- **Parent and context:**
  - Mechanism page: [`docs/mechanisms/prime-zeta-codebook.md`](../../mechanisms/prime-zeta-codebook.md).
  - Construction qualification: [`archive/labs/prime-zeta-construction-2026-10-14/README.md`](../../../archive/labs/prime-zeta-construction-2026-10-14/README.md).
  - Coverage census instrument: [`docs/labs/coverage-census-2026-10-18/README.md`](../../labs/coverage-census-2026-10-18/README.md).
  - Injective pre-flight receipt (`P8` `127 / 3,847` `P_succ`, `427 / 14,287` full): [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`](../../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md).
  - Address-structured cells pre-flight sheet and receipt: [`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](m1-attention-address-structured-cells-DRAFT-2026-10-19.md) and [`docs/labs/address-cells-preflight-2026-10-19/README.md`](../../labs/address-cells-preflight-2026-10-19/README.md).
- **Governing constraints:**
  - **D24:** GPU only for every model quantity. Admission and zero-update scoring run exclusively on the GPU (`init-corpus`, `eval-corpus`, `attention-dev-b3`, `attention-export-token-cells`); the host only builds static integer tables and grades stored receipts.
  - **D23 / D25 / D30:** Zero softmax, squashing, or dense matrix multiplication anywhere.
  - **D32:** Both new injective null modes (`CodebookMode::RandomInjective` and `CodebookMode::QuotaOnlyInjective`) are constructed in 100% pure integer arithmetic over `geometry::ROOTS: [[i8; 8]; 16]` with zero floating-point operations.
  - **D22:** A negative result closes a configuration, never a mechanism. This sheet tests zero-update reach and capacity only; trained attribution remains a separate held question.

## Why

The prime-zeta codebook (`CodebookMode::PrimeZeta`) gives all $4,096$ vocabulary tokens distinct 4-block $E_8$ root vectors (`4,096 / 4,096` unique vectors, `256 / 256` writer cells, `16` tokens per cell). As noted in [`docs/mechanisms/prime-zeta-codebook.md`](../../mechanisms/prime-zeta-codebook.md) (line 38), the two existing controls (`TangentNull` and `NormalNull`) run the same `16`-per-cell quota pass and interior-block uniqueness pass as `PrimeZeta`, and still compute zeta-zero phases.

This pre-registration tests `PrimeZeta` at **zero updates** (`accepted_updates = 0`) against two pure-integer injective nulls that use **no prime numbers and no Riemann zeta zeros**:
1. **`QuotaOnlyInjective { seed }` (`Q0, Q1, Q2` for seeds `0, 1, 2`):** enforces the exact same `16`-tokens-per-cell writer-cell quota (`256 × 16`) and unique interior block roots `(r_1, r_2)` as `PrimeZeta`, but assigns tokens to writer cells and interior root pairs via a deterministic seeded SplitMix64 Fisher–Yates permutation.
2. **`RandomInjective { seed }` (`R0, R1, R2` for seeds `0, 1, 2`):** draws `4,096` strictly distinct 4-block $E_8$ root vectors `(r_0, r_1, r_2, r_3)` uniformly without replacement from the $16^4 = 65,536$ orbit space via a deterministic seeded SplitMix64 integer generator, with **no writer-cell quota**.

## Proof: why raw alias ceiling cannot separate injective codebooks

In [`crates/uor-r4-discrete/src/coverage_census.rs`](../../../crates/uor-r4-discrete/src/coverage_census.rs) (lines 34–87), `alias_representatives(table, spec)` groups tokens with identical full $32$-integer vectors and assigns each group its lowest token ID `representative[t]`. Whenever a codebook is injective (`distinct_vectors == 4,096`), every alias group has size `1` (`representative[t] == t` for all $t \in \{0, \dots, 4095\}$), so `within_cell_tie_mass.tokens_in_alias_groups == 0` and `dev_alias_ceiling_rows == 14,287 / 14,287` on the DEV panel. Raw alias ceiling is therefore algebraically invariant across all `4,096`-injective tables and cannot separate `PrimeZeta`, `QuotaOnlyInjective`, or `RandomInjective`.

## Definitions of "Capacity" and "Reach" at zero updates

1. **Capacity (writer-cell packing & within-cell crowding — `coverage-census --token-cells` and `writer_cell_occupancy`):**
   - `distinct_vectors` (`4,096 / 4,096`) and `dev_alias_ceiling_rows` (`14,287 / 14,287`, invariant sanity gate).
   - `occupied_writer_cells` (out of `256`), `min_cell_occupancy`, and `max_cell_occupancy`.
   - `cell_collision_pairs` $= \sum_{c=0}^{255} \binom{n_c}{2}$: number of unordered token pairs sharing a writer cell and competing on within-cell `token_distance`. By convexity of $\binom{n}{2}$, the minimum over $V = 4,096$ and $256$ cells is $256 \times \binom{16}{2} = 30,720$ (achieved by `PrimeZeta` and `QuotaOnlyInjective`), whereas `RandomInjective` has strictly higher crowding (`32,493–32,802` pairs).
2. **Reach (zero-update GPU reader cell admission & scoring — `eval-corpus` and `attention-dev-b3` at `kept_cells = K = 8`):**
   - **`B3_init(P_succ)` (primary reach metric):** target-admitted rows on the canonical `3,847`-row context-sensitive `P_succ` subset (`admitted_cells - address_rivals == 1`).
   - **`B3_full`:** target-admitted rows across all `14,287` DEV rows.
   - **Net G-Cap:** $\min(\text{dev\_alias\_ceiling\_rows}, \text{B3\_init}(P_{\text{succ}}))$.
   - **Zero-update scoring (`eval-corpus` aggregate):** `top1_correct`, `top8_correct`, and `total_loss_numerator_sum` at `accepted_updates = 0`.

## Arms and pinned tables (`V4096 B4 C192 K8`, checkpoint 0, zero updates)

All seven arms share the identical checkpoint-0 model configuration from [`docs/labs/address-cells-preflight-configs-2026-10-19/evidence/p8.json`](../../labs/address-cells-preflight-configs-2026-10-19/evidence/p8.json) (`Relation{8,1}`, `LearnV1`, `NearestAddressMarginV1`, `ActiveLeafV1`, `InterleavedReadEvery8V1`, `rows_v1`, `seed: 1`, `kept_cells: 8`), differing **only** in the pinned codebook table:

| Arm | CLI mode (`prime-zeta-codebook`) | `table_sha256` | `file_sha256` | Unique vectors | Occupied cells | Min / max cell | Collision pairs |
|---|---|---|---|---:|---:|---:|---:|
| `P8` | `prime_zeta` | `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804` | `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q0` | `quota_only_injective:0` | `0098d4c3df8ce101e4a2dda8d8d28a29ac64f6ed3d521430504d0bb03ffbc18f` | `33b4e459cef8303f9d502f6f3e66327ec9e045026e7c8de896ad3ff350c2c415` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q1` | `quota_only_injective:1` | `13a76c718e7e34f3f0d912a8fc8239b02426c5cf1c54930c3950a0cb59054e44` | `57225bbb63387d9a6f39e7cb6415aa0e317d19534633bb731a3d7559efd4c3b9` | `4,096` | `256` | `16 / 16` | `30,720` |
| `Q2` | `quota_only_injective:2` | `279d72c26c0cded7340862b51abbdf938af3ee914e9474c7f528fb16d4d9db50` | `e03d6aef42608e1db3f4d33a0b034fd2c113cd9c48f366e0b6319d20d4f77175` | `4,096` | `256` | `16 / 16` | `30,720` |
| `R0` | `random_injective:0` | `35c8da6250a5d986ffa5f6758cbf761b45af6fa24d3893114b9693c1b0d9ee37` | `229e93f58ad36cb3f1fc2fa4ab194ab8ce06798a67e39537352b9817cc5df345` | `4,096` | `256` | `8 / 27` | `32,493` |
| `R1` | `random_injective:1` | `337a72bc3f5187125737243e4e066346669f1955a38d799b5447abc4a3eb77c6` | `36fec6323ae1daf15d965ff5cd1161f66b7a4a8a8b0a3261e9094e9c125a9468` | `4,096` | `256` | `3 / 27` | `32,802` |
| `R2` | `random_injective:2` | `21809edac362e40ea39702777c5cc48648cbe6cf2615599ed33ab8bf3103dfdb` | `25ec9bd29ce7fec20721826bab93f46caed8720cda6387cd43c5bddc62f0edea` | `4,096` | `256` | `8 / 26` | `32,659` |

## Bar

`PROPOSED, NOT REGISTERED` (submitted for owner / PM registration on [`#2881` comment `6353762758`](https://github.com/UOR-Foundation/uor-r4/issues/2881#issuecomment-6353762758)):
- **G-Rebuild & G-Init (D31 instrument gate):** On the pod, `prime-zeta-codebook` must rebuild all seven tables to the exact `file_sha256` and `table_sha256` pins above, and each arm's checkpoint-0 `model.table` must match its pin word-for-word (`accepted_updates == 0`, identical `routes` and `gates` across all seven arms).
- **P8 Exactness Gate (D31 instrument gate):** Arm `P8` (`prime_zeta`) must reproduce the 19 October pre-flight numbers exactly: `B3_full == 427 / 14,287`, `B3_init(P_succ) == 127 / 3,847`, and `top1_correct == 38 / 14,287`. Any discrepancy is an instrument fault (D31), never a mechanism reading.
- **G-Occ & Capacity Gate:** All seven arms must report `distinct_vectors == 4,096`, `dev_alias_ceiling_rows == 14,287`, and `within_cell_tie_mass == 0`. `P8` and `Q0..Q2` must report `occupied_cells == 256`, `max_cell_occupancy == 16`, and `cell_collision_pairs == 30,720`; `R0..R2` must report `cell_collision_pairs > 30,720` (`32,493`, `32,802`, `32,659`).
- **Zero-Update Reach Attribution Bar (paired McNemar discordance on the `3,847` `P_succ` rows):**
  - Let $b$ be `P_succ` rows admitted by `P8` only and $c$ be `P_succ` rows admitted by control arm $X \in \{Q_0, Q_1, Q_2, R_0, R_1, R_2\}$ only, with one-sided paired floor $F = \max(24, 2\sqrt{b+c})$ (matching [`m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](m1-attention-address-structured-cells-DRAFT-2026-10-19.md)).
  - **Positive zero-update zeta-phase attribution:** `P8` clears $b - c > F$ against all three `QuotaOnlyInjective` seeds (`Q0, Q1, Q2`) and all three `RandomInjective` seeds (`R0, R1, R2`).
  - **Null / quota-equivalence reading (configuration result under D22):** If $|b - c| \le F$ between `P8` and `QuotaOnlyInjective` across seeds `0, 1, 2`, the zero-update test records that **at checkpoint 0 (`K = 8`), `PrimeZeta`'s reach (`~3.1%`) is governed by its `256 × 16` cell quota and full-vector injectivity rather than by log-prime zeta-phase angles**, closing only the zero-update configuration claim that zeta phases alone raise `K = 8` admission without corpus predecessor placement.

## Base artifact

None. Every arm is initialized at checkpoint 0 (`discrete-stack init-corpus`, `new_model(0)`, `accepted_updates = 0`) from its pinned `uor-r4.codebook-table/1` integer table and staged config in [`docs/labs/r-prime-zeta-vs-nulls/evidence/`](../../labs/r-prime-zeta-vs-nulls/evidence/).

## Data split

- **TRAIN** (`3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`): bound in the driver config for provenance verification only; zero training updates are run and none of `P8`, `Q0..Q2`, or `R0..R2` reads `TRAIN` during construction.
- **DEV** (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `14,287` eligible rows; `3,847` `P_succ` rows): used exclusively for zero-update GPU evaluation (`eval-corpus`, `attention-dev-b3`, `coverage-census --token-cells`).
- **Fresh held-out slices:** untouched.

## Steps

1. Host constructor and CLI exporter (`crates/uor-r4-discrete/src/prime_zeta.rs` and `crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`) and finite host unit tests are implemented and verified on macOS (`6 / 6` unit tests in `prime_zeta.rs` + CLI mode parser test, `0` new `D23`/`D25`/`D32` sites).
2. Obtain a non-author `TEST FITNESS: FIT` review of this sheet (CHARTER §4.5) and owner / PM registration of the comparison bar and pod spend on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881).
3. On one RTX 5090 pod via `scripts/pod/uor-pod` (`uor-pod status` first), execute [`docs/labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh`](../../labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh):
   - Rebuild all seven tables (`P8`, `Q0..Q2`, `R0..R2`) with `prime-zeta-codebook` and verify exact match against the pinned `file_sha256` and `table_sha256` digests above.
   - For each arm `A in p8 q0 q1 q2 r0 r1 r2`, run `attention-build-identity`, `init-corpus`, `eval-corpus`, `attention-dev-b3`, `attention-export-token-cells`, and `coverage-census --token-cells --bar-rows 500`.
   - Compute paired row-by-row `P_succ` admission discordance ($n_{11}, b, c, n_{00}, b - c, F$) between `P8` and each null arm (`Q0..Q2`, `R0..R2`), and between `Q_s` and `R_s`.
   - Archive all outputs off the pod before `uor-pod down`.
4. Record the zero-update reach and capacity numbers as one row in `docs/research/notebook/m1-prime-zeta.md` (worded for the configuration under D22) and set `docs/roadmap/items/r-prime-zeta-vs-nulls.md` to `status: done`.

## Stop

- Any mismatch on table rebuild SHAs, checkpoint-0 table identity, or `P8`'s `427 / 127 / 38` reproduction stops execution immediately as an instrument fault (D31), never as a mechanism result.
- `eval-corpus` refuses any checkpoint with `accepted_updates > 0`; zero training updates occur in this battery.
- Pod spend (`4` pods / `$8/h` cap) and the `30 GiB` local disk floor are hard stops.

## Cost

About 15 minutes on one RTX 5090 pod (`~$0.20` at `$0.80/h`): ~8 minutes for the release CUDA build and ~1 minute per arm across the 7 zero-update arms (`P8`, `Q0..Q2`, `R0..R2`). Zero training updates.

## Minimum test power

- Capacity metrics (`distinct_vectors`, `occupied_cells`, `cell_collision_pairs`) are exact deterministic counts over all `4,096` vocabulary tokens and `256` writer cells.
- Zero-update reach (`B3_full` on `14,287` DEV rows and `B3_init(P_succ)` on `3,847` `P_succ` rows) is evaluated across `3` independent seeds (`0, 1, 2`) for each null family (`QuotaOnlyInjective` and `RandomInjective`), exceeding the CHARTER §4.3 minimum of `>= 2` seeds and `>= 200` rows (`3,847 >= 200`, paired McNemar floor $F = \max(24, 2\sqrt{b+c}) \approx 2\sigma$).

## Verdict

TEST FITNESS: PENDING — drafted by `antigravity` (`antigravity-27`, PR `#3014`); awaiting non-author `TEST FITNESS: FIT` review and owner/PM bar registration and pod-spend approval on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881).
