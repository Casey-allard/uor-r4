# ORDER: m1-close-saved-model, antigravity-106

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `4b77eb7f5`.

## Goal

The item's done-when reads:

> One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

Building on `antigravity-105` (`1899abc43`), execute the Path A discrete joint-trunk evaluation and integer artifact export audit:
1. Evaluate the joint-conditioning hierarchy (`select_level3_joint_hierarchy_global` vs query-backed joint conditioning where unconditioned marginals `c_rr`, `c_r`, `c_c` do not override `c_q`) and collision-free vs Stage T `OuterL1V1` codebooks on the stored GPU `1,049,856`-row `mw-balp/train` and `14,287`-row `ft/dev` streams (`Seed 1` and `Seed 2` recurrence states + `fresh_v2`).
2. Implement and test the corrected query-backed joint trunk selector and self-contained integer trunk artifact export/verification in `crates/uor-r4-discrete/src/trunk.rs`.
3. Record the exact `ft/dev`, `P_succ`, and `fresh_v2` measurements in `docs/labs/m1-close-saved-model/` and `docs/research/notebook/m1-discrete-attention.md`, closing `#2837` if all five bars `(a)–(e)` pass on the sealed GPU artifacts or documenting the exact remaining GPU step if a pod run is required.

## Cancel probe (run first)

Verify on the stored `w2-capacity-2026-10-25` GPU capture (`1,049,856` TRAIN rows in `train-stream.bin` + `query-sidecar.bin`, `14,287` `ft/dev` rows in `dev-capture.json`) and `stage-t-20261025` checkpoints that:
1. Unconditional weighting of marginal `c_rr` (`16x`), `c_r` (`8x`), and `c_c` (`4x`) above `1 * c_q` in `select_level3_joint_hierarchy_global` lets unconditioned `read_code`/`rec_code` marginals override `query_token` ($x_{L-1}$) on common target tokens (`instrument` finding in the selector's fallback order), whereas placing `c_q` above unconditioned `(c_rr, c_r, c_c)` while keeping conditioned `(c_qr, c_qc)` above `c_q` preserves the `6,300` query-token floor and tests whether joint `(query_token, read_code, rec_code)` conditioning lifts top-1 above `6,324` on `ft/dev` and `1,753` on `P_succ`.
2. In `T16_fz` (`final-checkpoint.json`), `88` codebook vector collisions (`4,008 / 4,096` distinct 32-int vectors) cause `OuterL1V1` tie-breaking to drop collided higher-ID tokens even after exact `DiscreteHamiltonGateV1` rotation, whereas the collision-free initial `4,096 / 4,096` codebook (`checkpoint-00000.json`) has `0` tie-break collisions.

Result (`2026-10-26`): **PASS** (`instrument` finding confirmed; `select_level3_joint_hierarchy_global` fallback order and codebook collision tie-breaking isolated before any pod spend).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-close-saved-model*`
- `docs/labs/m1-close-saved-model/**`
- `docs/loop/tasks/m1-close-saved-model/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

- The corrected query-backed joint trunk selector and collision-free `OuterL1V1` verification are implemented and unit-tested in `crates/uor-r4-discrete/src/trunk.rs`.
- Stored GPU streams (`1,049,856` TRAIN rows, `14,287` `ft/dev` rows, `14,287` `fresh_v2` rows) are graded across `Seed 1` and `Seed 2`, with all findings recorded in `docs/labs/m1-close-saved-model/`, `docs/research/notebook/m1-discrete-attention.md`, and `REPORT.md`.
- `uor-harness loop gates --full` is green.

## Stop and ask if

- Never stop and wait (`CHARTER.md` §5.1); resolve any bar, classification, or next-step decision via an in-session D68 decision review logged on `#2837`.
