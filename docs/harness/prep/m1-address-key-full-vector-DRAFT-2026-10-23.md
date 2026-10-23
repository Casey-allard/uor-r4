# Full-vector / interior-block address metric on the A8 placement: zero-update GPU pre-flight — DRAFT

- **Status:** DRAFT, pre-flight only (`zero training updates`). **TEST FITNESS: PENDING**
- **Roadmap item:** [`docs/roadmap/items/m1-address-key-full-vector.md`](../../roadmap/items/m1-address-key-full-vector.md) (`#2834`, Milestone `#2029`)
- **Design note & finite CPU controls:** [`docs/labs/m1-address-key-full-vector/README.md`](../../labs/m1-address-key-full-vector/README.md) (`11 / 11` host controls pass in `crates/uor-r4-discrete/src/address_cells.rs`)
- **Parent sheet:** [`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](m1-attention-address-structured-cells-DRAFT-2026-10-19.md) (executed in [`docs/labs/address-cells-preflight-2026-10-19/README.md`](../../labs/address-cells-preflight-2026-10-19/README.md), where `A8` reached `1,351 / 3,847` `P_succ` admission and `4,244 / 14,287` full-DEV admission at checkpoint 0 under the 5-value outer-block `address_distance`)
- **Rules:**
  - CHARTER §3.1–§3.3 (D23, D25, D30): geometry only; zero softmax, squashing, or matmul anywhere.
  - CHARTER §3.4–§3.5 (D24): GPU-only model evaluation and pre-built `nvcc` CUDA kernels; host code builds candidate tables from TRAIN counts and runs finite synthetic controls only.
  - CHARTER §4.1–§4.5 (D22, D31): one-page pre-registration and non-author `TEST FITNESS: FIT` before any GPU run; a negative closes a configuration only.

---

## Arms (checkpoint 0, zero updates, seed 1, `Relation { kept: 8, sweeps: 1 }`, `ActiveLeafV1`, `rows_v1`)

| Arm | Codebook Table Mode (`address-cells-codebook`) | Writer Address Metric (`discrete.cu`) | Writer $K$ | Role |
|---|---|---|---:|---|
| **A8-Outer** | `address_structured_v1` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) | 2-block outer `address_distance` (`5` levels `{0,4,8,12,16}`) | `8` | **Exactness & baseline control:** must reproduce `4,244 / 14,287` full-DEV and `1,351 / 3,847` `P_succ` target-admitted rows and `top1 = 133` exactly (D31 instrument gate). |
| **FV8-Env** | `address_full_vector_v1` (rebuilt from TRAIN `3dee858f…` / `b5e7352d…` via `address-cells-codebook CONFIG OUT full-vector`) | Unweighted $B$-block cell-envelope minimum $d_{\text{env}}(q, c) = \min_{t \in c} \text{token\_distance}(q, t)$ | `8` | **Treatment 1 (parameter-free full-vector envelope).** |
| **FV8-TwoScale** | `address_full_vector_v1` | Two-scale full-vector distance `(d_env(q, c) << 8) + sum_{t in c} token_distance(q, t)` | `8` | **Treatment 2 (two-scale full-vector envelope + cell sum tie-break).** |
| **FVS8-TwoScale** | `address_full_vector_shuffled_v1_seed_20261019` (seeded predecessor relabel control) | Two-scale full-vector distance | `8` | **Shuffled packing control:** separates predecessor-pulled interior/outer geometry from quota balance and tie-break order. |
| **FV16-TwoScale** | `address_full_vector_v1` | Two-scale full-vector distance | `16` | **Bracket control:** reports $K = 16$ scaling alongside `A16` (`6,023 / 14,287` full, `2,178 / 3,847` `P_succ`); decides no gate by itself. |

---

## Bar

**Status: PROPOSED, NOT REGISTERED** (awaits owner/PM registration on `#2834` and non-author `TEST FITNESS: FIT` before any CUDA kernel change is qualified or run on a GPU pod).

Per [`docs/roadmap/items/m1-address-key-full-vector.md`](../../roadmap/items/m1-address-key-full-vector.md) (`## Done when`):
1. **G-Exact (`A8-Outer` D31 instrument gate):** `A8-Outer` reproduces `4,244 / 14,287` full-DEV admitted rows, `1,351 / 3,847` `P_succ` admitted rows, and `top1_correct == 133` at checkpoint 0 word-for-word. Any discrepancy stops as an instrument fault (D31).
2. **G-Occ & G-Cap (Stage 1 capacity & tie-mass gate):** On `FV8-Env` and `FV8-TwoScale`, `coverage-census --token-cells` reports `distinct_vectors == 4,096`, `occupied_cells == 256`, `within_cell_tie_mass.tokens_in_alias_groups == 0`, `within_cell_tie_mass.dev_rows_targeting_non_representative == 0`, and `dev_alias_ceiling_rows == 14,287 > 1,941` (`ALIAS_CEILING_ABOVE_BAR`), with net winnable positions $\min(\text{dev\_alias\_ceiling\_rows}, \text{full\_DEV\_admitted\_rows}) > 1,941$.
3. **Stage 1 starting reach bar (primary treatment gate):** At checkpoint 0 (`accepted_updates == 0`) at $K = 8$, the treatment arm (`FV8-Env` or `FV8-TwoScale`) achieves:
   - `B3_init(P_succ) > 1,351 / 3,847` (strictly above `A8`'s `1,351 / 3,847`), AND
   - Paired one-sided McNemar discordance against `A8-Outer` on the `3,847` `P_succ` rows clears $b - c > F = \max(24, 2\sqrt{b + c})$, where $b$ is rows admitted by `FV8` only and $c$ is rows admitted by `A8-Outer` only, AND
   - Paired one-sided McNemar discordance against `FVS8-TwoScale` clears $b_{\text{shuf}} - c_{\text{shuf}} > \max(24, 2\sqrt{b_{\text{shuf}} + c_{\text{shuf}}})$ (attributing the gain to predecessor-pulled full-vector geometry rather than balanced interior packing alone).

---

## Base artifact

None (`accepted_updates == 0`). Every arm is initialized at checkpoint 0 (`init-corpus`, `new_model(0)`) from its pinned `uor-r4.codebook-table/1` JSON file and bound by `table_sha256`, `file_sha256`, `config_sha256`, and `lineage_sha256`.

---

## Data split

- **TRAIN** (`mw-balp/train/tokens.u16` SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, counts SHA-256 `755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44`): read on the host by `address-cells-codebook` to construct `address_full_vector_v1` and `address_full_vector_shuffled_v1_seed_20261019`.
- **DEV** (`ft/dev/tokens.u16` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `response_mask.u8` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; `14,287` eligible rows, `3,847` `P_succ` rows): scored once per arm at checkpoint 0 on the GPU. Never read during table construction.
- **Fresh sealed transfer slice** (`0b7e754c…` / `a1cda529…` / seal `df5d074a…`): untouched.

---

## Steps

1. **Host metric & finite CPU controls (completed in `antigravity-37`):** `crates/uor-r4-discrete/src/address_cells.rs` implements `AddressCellsMode::FullVectorV1`, `AddressCellsMode::FullVectorShuffledV1`, `cell_outer_l1`, `token_full_l1`, `cell_envelope_l1`, `full_vector_cell_l1`, and `admit_cells_by_distances` (`11 / 11` unit tests pass via `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells`).
2. **CUDA kernel & finite CUDA qualification (after host review):** Add the opt-in writer address-metric dispatch in `crates/uor-r4-discrete/src/discrete.cu` (`outer_l1_v1` default vs `cell_envelope_l1_v1` vs `full_vector_cell_l1_v1`), verify on host with the stub-`nvcc` `--features cuda` type check, and add finite CUDA qualification controls verifying exact match between host `cell_envelope_l1` / `full_vector_cell_l1` and the device kernel.
3. **Non-author `TEST FITNESS: FIT` & owner/PM registration:** Obtain non-author `TEST FITNESS: FIT` on this sheet and owner/PM registration of the Bar and pod spend on `#2834`.
4. **Zero-update GPU pre-flight on one leased RTX 5090 (`scripts/pod/uor-pod`):**
   - Run `address-cells-codebook` on the pinned TRAIN split (`3dee858f…`) to export and hash `address_full_vector_v1.json` and `address_full_vector_shuffled_v1_seed_20261019.json`, and verify `A8` rebuilds `6600bdb1…` exactly.
   - Run the finite CUDA qualification binary first.
   - For each of the 5 arms (`A8-Outer`, `FV8-Env`, `FV8-TwoScale`, `FVS8-TwoScale`, `FV16-TwoScale`), run `init-corpus`, `eval-corpus`, `attention-dev-b3` ($K=8$ arms), and `attention-export-token-cells` + `coverage-census --token-cells --bar 1941`.
   - Archive all outputs and `FILES.sha256` off the pod before `uor-pod down`, and record one row in `docs/research/notebook/m1-discrete-attention.md`.

---

## Stop

- Any failure of `A8-Outer` to reproduce `4,244 / 14,287` full-DEV admission, `1,351 / 3,847` `P_succ` admission, or `top1 == 133`, or any mismatch between host and device full-vector cell distances in finite CUDA qualification, stops immediately as an instrument fault (D31), never as a mechanism negative.
- If both `FV8-Env` and `FV8-TwoScale` fail to clear `1,351 / 3,847` + $F$ on `P_succ` or fail `full_DEV_admitted_rows > 1,941`, close the `(256-cell outer partition, B=4 full-vector cell metric at K=8)` configuration under D22 and route to `m1-wider-code-carrier` (`#2847`).
- Hard ceiling: 30 minutes wall-clock (`~$0.40` on one RTX 5090) and 30 GiB free disk floor.

---

## Cost

- **GPU compute:** 1x RTX 5090 via `scripts/pod/uor-pod` (`uor-pod status` first) for `< 25 min` wall-clock (`~10 min` build + finite qualification, `< 1 min` per arm $\times$ 5 zero-update arms) = **`~$0.35–$0.40`** at `~$0.82/h`. Zero training updates are executed.

---

## Minimum test power

- Every arm is scored deterministically at checkpoint 0 across the full **14,287-row** DEV panel and the **3,847-row** `P_succ` copy-successor sub-panel ($\ge 200$ rows per CHARTER §4.3).
- Paired binary contrasts (`FV8` vs `A8-Outer` and `FV8` vs `FVS8-TwoScale`) use the exact McNemar discordance floor $F = \max(24, 2\sqrt{b + c})$ (`>= 2 SE` of the paired row difference on `3,847` rows).

---

## Verdict

TEST FITNESS: PENDING - M1 #2029 m1-address-key-full-vector
