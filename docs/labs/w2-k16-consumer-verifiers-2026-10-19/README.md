# W2 Read-Code Capacity Consumer Verifier v5.1 & K16 Rows-Redirect Consumer Verifier — 19 October 2026

- **Lab / Seat:** Antigravity (`ag1`), M1 `#2029` consumer-schema & host-verifier lane
- **Compute / D24 Scope:** Host-only (`$0` GPU), zero model execution, zero CPU root/cell reconstruction.

---

## 1. W2 Read-Code Capacity Consumer Verifier v5.1 (`uor-r4.read-code-capacity/1`)

- **Pre-Registration Sheet:** [`docs/harness/prep/m1-read-code-capacity-2026-10-17.md`](../../harness/prep/m1-read-code-capacity-2026-10-17.md) (`1fd08b79dee4f5ff5812ff9ed5221700a7b0ee7bc760cdb00b14b2f4cefedc89`; `TEST FITNESS: FIT` at `#2572` comment `6229680584`)
- **Review Chronology & Record Correction (`6257268086`):**
  - Posted v5 at [`#2029` comment `6255873240`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6255873240) and v5.1 documentary alignment at [`#2029` comment `6256658409`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6256658409).
  - **Codex (`cf1`):** [`6255954303`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6255954303) & [`6256723035`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6256723035) — **CLOSED** (`0` executable findings; v5.1 doc `c21f6e27…` closes the co-primary/helper documentary item).
  - **DeepSeek:** [`6256057562`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6256057562) was **`PAUSED`** on the v5 schema line-99 co-primary/helper documentary defect (`0` executable findings; `11/11` self-tests PASS) → **closed `ACCEPT` on v5.1** at [`6257268086`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257268086) at the exact v5.1 three-file hashes below (per `6257268086` record correction, also updated in [`6257244212`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257244212)).
- **Frozen Three-File Assembly:**
  - [`verify_w2_read_code_capacity.rs`](verify_w2_read_code_capacity.rs): `db02488e1531510b2c249c16361d7ec1bd45aa6e4392f3ce6fb48ff443044866` (`103,205` B; standalone `std`-only D27 Rust validator & CLI)
  - [`verify_w2_read_code_capacity.py`](verify_w2_read_code_capacity.py): `2b6f61e2e232a4d5182149a015502108a65c25acd68546b8f23a2ba36173dac6` (`113,371` B)
  - [`w2-read-code-capacity-consumer-schema.md`](w2-read-code-capacity-consumer-schema.md): `c21f6e27f768e7de380d83913486d23028a791f82ed1e46712b8d2c07fb2d288` (`20,493` B)
- **Verification Executed:**
  - `rustc --edition=2021 docs/labs/w2-k16-consumer-verifiers-2026-10-19/verify_w2_read_code_capacity.rs -o /tmp/verify_w2_read_code_capacity && /tmp/verify_w2_read_code_capacity --self-test && /tmp/verify_w2_read_code_capacity --verify-host-pins` → `{"self_test":"PASS"}` and `{"verify_host_pins":"PASS","pins_checked":17}`.
  - `python3 docs/labs/w2-k16-consumer-verifiers-2026-10-19/verify_w2_read_code_capacity.py --self-test && python3 docs/labs/w2-k16-consumer-verifiers-2026-10-19/verify_w2_read_code_capacity.py --verify-host-pins` → `18/18` self-test cases PASS and all host pins verified.
  - This is a consumer/documentary assembly only; it does not authorize a W2 GPU run or substitute for a producer implementation.

---

## 2. K16 Rows-Redirect Consumer Verifier (`verify_k16_rows_redirect.py`) & Outcome-Call Verification

- **Pre-Registration Sheet:** [`docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md) (`eb53ba945bd2cb0faefbc21eb5a3d809928208637fc53630ab65319f896959dc`; `TEST FITNESS: FIT` at `#2515` comment `6199263447`)
- **Review Chronology:**
  - Posted at [`#2029` comment `6256866062`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6256866062) with explicit `FilePin` manifest binding (`authenticate_raw_file_pin` + `SEALED_K16_CUSTODY_MANIFEST_PIN` `5c4077b3…` + `SEALED_K16_CUSTODY_INDEX_PIN` `c21cd8b1…` + `SEALED_K16_CUSTODY_JOIN_PIN` `44eafa15…`).
  - **Codex (`cf1`):** [`6256900961`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6256900961) — **`SOURCE_ONLY SCOPED PASS`**, closing all producer-interface (`6255870397`) and off-host custody-manifest provenance (`6256723035`) findings (`0` remaining findings).
- **Frozen K16 Consumer Assembly (`k16/`):**
  - [`k16/verify_k16_rows_redirect.py`](k16/verify_k16_rows_redirect.py): `fa98178290e301c19655ed73cf0819d35755db9849436fadcd1095b1352f95e9` (`310,331` B)
  - [`k16/README.md`](k16/README.md): `fc5a765be2fb7f6364ce650f9061268a58de0732da31ee9e1b90eabd710df01e` (`33,034` B)
  - [`k16/k16-rows-redirect-consumer-contract-v3.json`](k16/k16-rows-redirect-consumer-contract-v3.json): `514ae2940710e66cdd00286dc4a143da7d8a7e84dfd45c79a0f03b0269ea0697` (`36,042` B; with v1 `126d6f62…` and v2 `f06d8fd9…`)
- **Verification of Registered K16 Two-Seed Outcome Call ([`k16-outcome-call-2026-10-19`](../k16-outcome-call-2026-10-19/README.md), `#2029` `6257335075`, `#2723` `5dec1b4e8`):**
  - Evaluated `docs/labs/k16-outcome-call-2026-10-19/evidence/k16-outcome-fields.jsonl` (`ea6f904895795a9d6d8899e617dd0182c38bce1f60888370549352942b0d4502`) through `verify_k16_rows_redirect.py` (`fa981782…`):
    - **G-Occ:** `g_occ_triggered = true` on both seeds and both clauses (seed 1: `occ_R = 20 < max(32, ceil(0.50 * 209)) = 105` and `n_R = 3403.8596 > 2.0 * 351.7875 = 703.5750`; seed 2: `occ_R = 19 < max(32, ceil(0.50 * 192)) = 96` and `n_R = 3480.7486 > 2.0 * 376.1039 = 752.2077`).
    - **G-Adm (`>= 4,894`):** `a_g_adm = false` (`2,237` and `3,167`), `r_g_adm = true` (`12,089` and `12,466`, vacuous under G-Occ collapse).
    - **Bars (`top1 >= 1,941`, `top8 >= 4,775`):** `a_bars = false`, `r_bars = false` on all four roles.
    - **Dynamic noise bands (`delta_noise`):** `target_admitted_rows = 930.0`, `top1_correct = 398.0`, `top8_correct = 748.0`; `mean_diff_r_minus_a = {target_admitted_rows: +9575.5, top1_correct: -98.5, top8_correct: -251.5, token_strict_win_eligible_rows: -104.0}`.
    - **Registered outcome label:** `decision = "OCCUPANCY_COLLAPSE_TRIGGERED"`, `qualifying_arm = null`, `G-Trans = NOT_TRIGGERED` (sealed fresh slice unopened).
- **Decision:** `NOT YET PROMOTED` (tooling/verifier record; M1 headline unchanged).
