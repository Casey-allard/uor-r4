# Address-Structured Injective Cells (`A8` / `S8` / `P8` / `P16` / `A16`) Table & Pre-Flight Consumer Verifier (19 October 2026)

- **Line:** `m1-attention-address-structured-cells-consumer-verifier` — **Count:** 0/3 (verification tooling; does not count against any 3-negative mechanism line per D21) — **Headline:** `1.20` (`#2029` / `#2028`, unchanged)
- **Deliverable:** Host-only deterministic consumer verifier [`verify_address_cells_preflight.py`](verify_address_cells_preflight.py) (`57ae1c9349b178267936dfa89e55c4aba7cb8c8aa26cf563265e885c2d795c32`, `85,835` B) for the Address-Structured Injective Cells registration sheet [`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md) (`b94d5ae02ca433de67d81d4315a03676f931bac914625133d8973afec8b41275`, `#2738` / `#2748`), offline codebook constructor [`crates/uor-r4-discrete/src/address_cells.rs`](../../../crates/uor-r4-discrete/src/address_cells.rs) (`22dec8e967f7d2bfd410f83067ee69646993bffe0aa3be4996375896e33fa75e`, `#2742`), exporter [`crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`](../../../crates/uor-r4-discrete/src/bin/address-cells-codebook.rs) (`691349582b563dde327c0613cb72a282ce84fd96ed0dc226a409d0a25c19744c`, `#2742`), and staged five-arm pre-flight configs [`docs/labs/address-cells-preflight-configs-2026-10-19/`](../address-cells-preflight-configs-2026-10-19/README.md) (`#2746`).
- **Scope & Holds:** Host-only (`cuda_context_created: false`, `model_constructed: false`, `training: "HELD"`, `$0.00` GPU/rental spend). Does not release any D4 training hold or dispatch a pod.

---

## 1. Host-Only Reproduction of `A8` and `S8` Tables from Pinned TRAIN

Both the Rust binary (`cargo run --locked -p uor-r4-discrete --bin address-cells-codebook`) and the independent pure-Python reference in [`verify_address_cells_preflight.py`](verify_address_cells_preflight.py) (`reproduce_and_verify_from_train`) were executed on host CPU against the pinned `mw-balp` TRAIN split (`tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, `7,512,797` payload tokens, `2,425,925` eligible response-masked non-BOS target positions, `train_tokens_sha256` `3082a50cc92a2a37b653002e546a6fc5edd50b5c2966151b832f83e289b07945`, `counts_sha256` `755a5a81ddbf0f9d81b4d0bd88e54435b96f5485f87ad92a22d235651fe6ca44`) and `PrimeZeta` fallback table (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`), reproducing the `#2746` / `#2748` pins byte-for-byte:

| Arm / Table | Mode | File SHA-256 (`uor-r4.codebook-table/1`) | Bytes | `table_sha256` (`131,072` `i32` LE) | Distinct vectors | Occupied cells (`quota=16`) | Zero-mass tokens | TRAIN predecessor L1 total (`2,425,925` targets) | Mean L1 / target |
| :--- | :--- | :--- | ---: | :--- | ---: | ---: | ---: | ---: | ---: |
| **`P8`** (`prime-zeta-v4096-b4-c192-k8.json`) | `prime_zeta` | `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` | `278,619` | `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804` | `4,096` | `256` | — | `33,871,344` | `13.962239` |
| **`A8`** (`address-structured-v1.json`) | `address_structured_v1` | `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` | `278,778` | `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` | `4,096` | `256` | `2,749` | `22,374,488` (`-33.94%` vs `P8`, `-25.12%` vs `S8`) | `9.223075` |
| **`S8`** (`address-shuffled-v1-seed-20261019.json`) | `address_shuffled_v1_seed_20261019` | `a185bdca07077fe7157e947ff1fa80919e6f99165a4dd5a04061a97873c92718` | `278,790` | `d2fb8680a16e8017f6f0e6518df6810f6206023b0d14388dc076d4cfde2ebc5b` | `4,096` | `256` | `3,147` | `29,881,384` (`-11.78%` vs `P8`) | `12.317522` |

---

## 2. Staged 5-Arm Configuration & Lineage Verification (`#2746` / `#2748`)

- `driver_source_sha256` (11 concatenated source files in `crates/uor-r4-discrete/src/`): `44f3b50ab80a98931996c07dd1ab2f661b11ed0a3e9493b5e10ae9fd90632bdc`
- `docs/labs/address-cells-preflight-configs-2026-10-19/evidence/SHA256SUMS`: `895ec8fb8bd875d7009a0d91e82c838f5a88e636556c4e19614ed6530928c4fb` (`372` B; no self-entry)

| Arm | Config File | File SHA-256 | Bytes | `config_sha256` | `lineage_sha256` | `spec.kept_cells` | `read_spec.kept` | `attention_learning::scope` (`K=8` arms only) |
| :--- | :--- | :--- | ---: | :--- | :--- | ---: | ---: | :--- |
| **`P8`** | `evidence/p8.json` | `7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda` | `1,926` | `0c3404d43109eafa4230e7245caf4219283417894f6a5adf77b862a326c0d5cf` | `3849a4bdf93c847baef835c60253421fa14283fb15c47dbcd1c18fca23916025` | `8` | `8` | `m1-attention-learning-injective-83c1380a8f50-relation-k8-seed1-batches0..16404` |
| **`A8`** | `evidence/a8.json` | `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e` | `1,924` | `f223d78f45fe26dee4080ae7ca9a5b9ff12d9ef0df847375671d52db733fbecd` | `cf6cf50f76eb9f6950fc518258a878df4cf8e44c9ec0c010e85ceb50c8cb8940` | `8` | `8` | `m1-attention-learning-injective-6600bdb1306d-relation-k8-seed1-batches0..16404` |
| **`S8`** | `evidence/s8.json` | `b22c6c04ff160b741f1c6c2e8f7806f10ac8e5e3c2027ff72fef9d432f99bfad` | `1,936` | `16e37144c966925b36477efee1ee2bbedeb8a18b500454cd50a924ce5d743db5` | `fda6bd48d0d655631e17c284297b5c7d4b3cc8f8b23990f8be97cb92e246f761` | `8` | `8` | `m1-attention-learning-injective-a185bdca0707-relation-k8-seed1-batches0..16404` |
| **`P16`** | `evidence/p16.json` | `656fe13e4658e42446a79f44aa1da88fa1cb166e63a2caac81918e1afb03c0ea` | `1,927` | `0d7438c845fdc8eb4589ca61e707f21f7d4525081e4bc5041a3bd44938929c2d` | `8cbb8df7a73477a01a96698340a44a69432cb9140a7a71e03321a7e5537852c1` | `16` | `8` | Diagnostic bracket (`init-corpus`/`eval-corpus`; refused by `attention_learning::validate_config`) |
| **`A16`** | `evidence/a16.json` | `204733c9a21c370eaa9d725d7bb54da7d1d4a7d36877409dc97aef4291bf11a5` | `1,925` | `93623108cbbf11bb1e96263c89702ce259e060093102e38f89f83aad8c80480c` | `0340e25ff976491d5f01ef3bc824980812c49a84ff922450cee58ae95855ddf5` | `16` | `8` | Diagnostic bracket (`init-corpus`/`eval-corpus`; refused by `attention_learning::validate_config`) |

---

## 3. Pre-Flight Gates & Self-Test Coverage (`26/26` PASS)

[`verify_address_cells_preflight.py`](verify_address_cells_preflight.py) enforces every pre-flight gate and decision rule from [`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](../../harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md) (`b94d5ae02ca433de67d81d4315a03676f931bac914625133d8973afec8b41275`):

1. **Pod table rebuild & `build-identity` gate:** Verifies pod-rebuilt `P8` (`83c1380a…`), `A8` (`6600bdb1…`), `S8` (`a185bdca…`), and `counts_sha256` (`755a5a81…`), plus `driver_source_sha256` (`44f3b50a…`), `training: "HELD"`, `cuda_context_created: false`, and absence of `scope`.
2. **`G-Init` + negatives (a) and (b) + `G-Occ` across `P8`, `A8`, `S8`:** Verifies zero-update checkpoint-0 `cursor`, host table SHA refusal (`invalid discrete model: content hash differs`), GPU status-6 non-root refusal, and `G-Occ` (`occupied_cells == 256 > 16`, `distinct_vectors == 4096`, `alias_group_size_histogram == {"1": 4096}`, zero within-cell tie mass, `cell_occupancy_histogram == {"16": 256}`, `cell_collision_pairs == 30,720`).
3. **`P8` exactness gate (`D31`):** Requires `P8` checkpoint 0 to reproduce `427` full-panel and `127` `P_succ` target-admitted rows (`300` complement). Any mismatch returns `INSTRUMENT_FAULT_D31` (`INSTRUMENT_FAULT_P8_EXACTNESS_MISS`, `mechanism_negative: false`), never a mechanism negative.
4. **`K=16` bracket join precondition & `P8` formula check (`D31`):** Grades `eval-corpus` per-row records joined on `(target_position, target_token)` to `is_psucc`; requires `14,287` unique keys, `3,847` `P_succ` rows, `admitted_cells - address_rivals in {0, 1}`, and verifies that `admitted_cells - address_rivals == 1` on `P8` reproduces `(427, 127)` before grading `P16`/`A16` (or accepts `DROPPED_DIAGNOSTIC_NOT_RUN` per `#2748` Registration bindings if `eval-corpus` refuses `K=16`). `bracket_can_promote` is always `false`.
5. **Paired one-sided McNemar discordance on `P_succ` (`3,847` rows):** Verifies `n11 + b + c + n00 == 3,847`, marginals `n11 + b == treatment_admitted` and `n11 + c == control_admitted`, and evaluates `b - c > F` where `F = max(24, 2 * sqrt(b + c))` via exact integer inequality `(b - c > 24) and ((b - c)^2 > 4 * (b + c))`, classifying attribution into `PREDECESSOR_RELATION` (`A8 > P8` and `A8 > S8`), `FREQUENCY_PACKING_ONLY` (`A8 > P8` only), or `PLACEMENT_DID_NOT_MOVE_ADMISSION`.
6. **`A8` promotion battery:** Requires `B3_init(P_succ) >= 500` and `net_g_cap > max(500, top1_control(cp0) + 24)` (`A8_PROMOTED_TO_TRAINING_SHEET_DRAFT` vs `TERMINAL_PREFLIGHT_FAIL_A8_BELOW_GATE` / `CONFIGURATION_CLOSED_D22`, `mechanism_negative: false`).

## 4. Verification Commands

```sh
# 1. Run 26 positive/negative self-tests and verify staged configs + repository bindings
python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_address_cells_preflight.py \
  --self-test \
  --configs-dir docs/labs/address-cells-preflight-configs-2026-10-19 \
  --repo-root .

# 2. Also reproduce A8 and S8 tables byte-for-byte from local cached TRAIN pins
python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_address_cells_preflight.py \
  --self-test \
  --configs-dir docs/labs/address-cells-preflight-configs-2026-10-19 \
  --repo-root . \
  --train-tokens /Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v1/mw-balp/train/tokens.u16 \
  --train-mask /Users/casey.allard/uor-r4-local/codex-attention-preflight-inputs-v1/mw-balp/train/response_mask.u8

# 3. Grade a completed 5-arm pre-flight receipt directory
python3 docs/labs/address-cells-consumer-verifier-2026-10-19/verify_address_cells_preflight.py \
  --preflight-dir /path/to/preflight-receipt-dir
```
