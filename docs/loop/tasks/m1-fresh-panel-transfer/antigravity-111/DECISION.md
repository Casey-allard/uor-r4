# D68 Decision Card: Close `m1-fresh-panel-transfer` (`#2841`, `status: done`) on `PASS_BAR_E_FRESH_TRANSFER`

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-111`, PR `#3318`)
- **Item:** `m1-fresh-panel-transfer` (`#2841`, Step 7 of [`PLAN.md`](../../../../../PLAN.md))
- **Pre-registration sheet:** [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md)
- **Prior FIT & D67 Pre-Run Reviews:** `#2841` (`#issuecomment-6423732020`, `#issuecomment-6432026082`)

## 1. Whole-Project Chain and Decision Question

1. `m1-fresh-panel-transfer` (`#2841`, Step 7 of `PLAN.md`) requires evaluating the two-seed DEV bar-(a)–(d)-passing 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `04652a3d…` and `1b49e8bd…`) integer model on the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`, `14,479` eligible targets, `3,594` `rposition` `P_succ` rows) once via `d11-scorer fresh-score` after `#2841`'s FIT + D67 pre-run review (`#issuecomment-6432026082`), and closing `#2841` (`status: done`) when `PASS_BAR_E_FRESH_TRANSFER` is recorded.
2. In `antigravity-110` (PR `#3312`), `d11-scorer fresh-score` was executed across two ahead-of-time CUDA builds (`Build A` `ce5843b00dee006cf89a1028f80b5125221393dbfb549b09c2273f3e6d3660f4` and `Build B` `0357abf8ca70cd463b7976eebef5ddea2dbfb7d9128ef1246246f665e25fed17`, `ptx_sha256` `5eca2621…`, `cubin_sha256` `c513559a…`) on `1x RTX 5090` (`vast-57903382`) and archived in `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/m1-close-saved-model-20261026.tar` (`a758ca0962d68c43d2796fcf7e43e3af57dccf0d82849f3b11a9f4e2294d1506`):
   - `G-Fresh-Gate`: **PASS** (`d11_dev_top1 = 7,002` / `7,001 >= 6,324`, `psucc_d11_top1 = 1,779` / `1,770 >= 1,265`, `0` export gap, `B4` pass).
   - `G-Fresh-Full` (`14,479` rows, `U_fresh = 1,279`, `T2_fresh = 6,582`, `two_seed_spread_full = 13`): **PASS** (`seed_1 = 7,215` [`+5,936` vs `U_fresh`, `+633` vs `T2_fresh`, floor `82`]; `seed_2 = 7,228` [`+5,949` vs `U_fresh`, `+646` vs `T2_fresh`, floor `82`]).
   - `G-Fresh-Psucc` (`3,594` `rposition` rows, `U_psucc = 229`, `T2_psucc = 1,240`, `two_seed_spread_psucc = 2`): **PASS** (`seed_1 = 1,713` [`+473` vs `T2_psucc`, floor `52`]; `seed_2 = 1,711` [`+471` vs `T2_psucc`, floor `52`]; `windows(2)` `4,900`-row variant: `2,333` / `2,329` vs `1,863` [`+470` / `+466`, floor `53`]).
   - `G-Fresh-Integrity`: **PASS** (`refused_rows = 0`, `reference_mismatches = 0`, `b4_pass = true`).
3. **Decision (`CHARTER.md` §5.1, §5.2, D68):**
   - Commit the sealed `Fresh V2` gate receipt and two-seed transfer reports into `docs/labs/m1-fresh-panel-transfer/evidence/`, reconcile the prep-sheet `TRAIN`/`DEV` SHA-256 pins in `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md` (`classification: instrument`), update `docs/labs/m1-fresh-panel-transfer/README.md`, `docs/start/faq.md`, `docs/research/open-questions.md`, and `docs/research/notebook/m1-discrete-attention.md`, and transition `docs/roadmap/items/m1-fresh-panel-transfer.md` from `status: ready` to `status: done` (`Closes #2841`).
