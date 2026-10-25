# Prime-zeta codebook vs random-injective and quota-only-injective nulls at zero updates (`r-prime-zeta-vs-nulls`)

- **Lab / Task:** `antigravity` (`antigravity-27`, `antigravity-28`, `antigravity-36`; non-author audit by `kimi` in `kimi-25`)
- **Roadmap item:** [`docs/roadmap/items/r-prime-zeta-vs-nulls.md`](../../roadmap/items/r-prime-zeta-vs-nulls.md) (Issue [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881), Milestone M1 [`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029))
- **Design note & decision card:** [`#2881` comment `6353762758`](https://github.com/UOR-Foundation/uor-r4/issues/2881#issuecomment-6353762758) (cross-linked on [`#2029` comment `6353766545`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6353766545))
- **Pre-registration sheet:** [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) (`TEST FITNESS: NOT_FIT` from `kimi-25` `#3067`; prerequisite `(b)` total outcome rule resolved in `antigravity-36`; awaits only prerequisite `(a)`, owner/PM registration of the comparison bar and `~$0.20` pod spend on `#2881`)
- **Result (claude-112, 25 Oct):** run and graded. Under the bar registered by the D68 card, it is a **configuration negative (loss)**. See [§5](#5-results-claude-112). The status line below is antigravity's, as it stood before the run.
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

---

## 5. Results (claude-112)

**Configuration negative (loss), D22.** P8 wins against none of Q0–Q2 and loses to two of them. Graded by the registered instrument (`instrument/`, `pz-nulls-grade`) in [`grade.json`](grade.json), sha256 `14cc2c62882069361c22632b94f867be480fda9bd822adff0490990b5ec37975`.

**Records:**
- the D68 card `docs/loop/tasks/r-prime-zeta-vs-nulls/claude-112/DECISION.md`;
- the sheet [`r-prime-zeta-vs-nulls-2026-10-25.md`](../../harness/prep/r-prime-zeta-vs-nulls-2026-10-25.md), FIT in round 2 with the D67 pre-run review passed (#2881 `6401883120`);
- the job [`pz-nulls-job.sh`](pz-nulls-job.sh), sha256 `820ee916…`, run at source `d1eea395d`.

**Run:**
- One pod, `vast-57710234` (2×5090, one GPU used), about 9 minutes at $2.176/h, about $0.33. A 2×5090 was what the provider gave.
- The outputs are in the HF store at `results/claude/prime-zeta-nulls-2026-10-25/run/`. All 68 checksummed files verified against SHA256SUMS after download, before the pod came down. The folder holds 70 files, counting `SHA256SUMS` and `job.log`.
- The pod was leased at 19:06Z and deleted at 19:15Z.

**Instrument gates, all passed in the job:**
- G-Rebuild: all 7 tables equal their pins.
- G-Init: checkpoint 0, each table word for word, and identical routes and gates.
- G-Occ: as below.
- G-Exact: P8 read `427 127 38`, before any null was evaluated.

| Arm | P_succ admitted (of 3,847) | Full admitted (of 14,287) | Top-1 | Top-8 | Min / max per cell | Collision pairs |
|---|---:|---:|---:|---:|---:|---:|
| P8 | 127 | 427 | 38 | 95 | 16 / 16 | 30,720 |
| Q0 | 178 | 564 | 36 | 74 | 16 / 16 | 30,720 |
| Q1 | 184 | 560 | 49 | 133 | 16 / 16 | 30,720 |
| Q2 | 126 | 696 | 52 | 158 | 16 / 16 | 30,720 |
| R0 | 183 | 553 | 24 | 72 | 8 / 27 | 32,493 |
| R1 | 188 | 485 | 25 | 116 | 3 / 27 | 32,802 |
| R2 | 96 | 373 | 27 | 73 | 8 / 26 | 32,659 |

Every arm has 4,096 distinct vectors, alias ceiling 14,287, zero tie mass and 256 occupied cells.

**Paired on P_succ.** b counts rows P8 admits and the null does not; c counts the reverse. The floor F is max(24, 2√(b + c)).

| P8 against | b | c | Outcome |
|---|---:|---:|---|
| Q0 | 122 | 173 | loss |
| Q1 | 122 | 179 | loss |
| Q2 | 122 | 121 | tie |
| R0 | 119 | 175 | loss |
| R1 | 125 | 186 | loss |
| R2 | 125 | 94 | win |

**Matched-seed crowding, reported only:** Q0 against R0 is 169 / 174 (tie), Q1 against R1 is 159 / 163 (tie), and Q2 against R2 is 122 / 92 (win for Q2).

**Read with care. These points are descriptive and decide nothing.**
- **P8 sits near chance, below four of the six nulls.** The K = 8 of 256 cells chance rate is about 120. P8 admits 127. Q0, Q1, R0 and R1 admit 178 to 188, about 1.5 times chance. Only Q2 (126) and R2 (96) are at or below P8.
- **Both losses to Q clear the floor by a wide margin:** 51 and 57 rows, against F of about 34. The one win, against R2, clears it by about 1.4.
- **The overlap of admitted rows** is at the level expected if admission were independent of the table. P8 shares 2 to 8 of its 127 admitted rows with any null, against about 3 to 6 expected (127 × the null's P_succ count / 3,847).

**Filled notebook row.** The installed loop guard refuses `docs/research/notebook/m1-prime-zeta.md`. That page is outside this item's scope, and the fix that admits an item's own notebook page (#3189) is not in the installed binary. The row below is the one to add there.

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-25 | Zero-update admission at K = 8: prime-zeta (P8) against quota-only-injective Q0–Q2 (256 × 16 quota, seeded shuffle) and random-injective R0–R2 (no quota), checkpoint 0, seed 1, 14,287-row DEV panel with 3,847 P_succ rows; 1 RTX 5090, source `d1eea395d`, about $0.33 (claude-112) | Paired P_succ b/c: Q0 122/173 (loss), Q1 122/179 (loss), Q2 122/121 (tie), R0 119/175 (loss), R1 125/186 (loss), R2 125/94 (win). Reach P_succ (full): P8 127 (427); Q 178, 184, 126 (564, 560, 696); R 183, 188, 96 (553, 485, 373). Every arm 4,096 distinct vectors and 256 cells; collision pairs 30,720 (P8, Q) and 32,493–32,802 (R) | D68 card, registered by sheet FIT: win if b − c > max(24, 2√(b + c)), loss the reverse; no Q win is negative. Gates G-Rebuild, G-Init, G-Occ and G-Exact (427/127/38) passed | negative (loss), for this configuration: checkpoint 0, K = 8, three seeds per null family (D22). The mechanism stays ACTIVE | [r-prime-zeta-vs-nulls](../../labs/r-prime-zeta-vs-nulls/README.md) |
