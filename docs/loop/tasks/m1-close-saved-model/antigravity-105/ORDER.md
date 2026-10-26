# ORDER: m1-close-saved-model, antigravity-105

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Base: `e9cfa8873`.

## Goal

The item's done-when reads:

> One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel.

This unit (`antigravity-105`, PR `#3301`):
1. Builds the exact-integer five-bar M1 closing-bar evaluator and `FreshTransferGateReceipt` generator (`m1_close` module and `d11-scorer m1-close-grade` in `docs/labs/m1-d11-artifact-scorer/instrument/`) with failing-first negative-gate unit tests (`tests/m1_close.rs`);
2. Fixes the two Stage 3 trunk readout limitations documented in `docs/labs/m1-trunk-assembly-and-training/README.md:36-38` (`crates/uor-r4-discrete/src/trunk.rs`) by adding joint-first hierarchical readout (`(query, read, rec) -> (query, read) -> (query, rec) -> query -> (read, rec) -> read -> rec -> unigram`, matching `#3179` `QC = C1`) and post-rotation full two-stage `OuterL1V1` evaluation (`evaluate_joint_hierarchy_full_writer`), with failing-first unit tests;
3. Pre-registers `docs/harness/prep/m1-close-saved-model-2026-10-26.md` with fresh-context non-author `TEST FITNESS: FIT` and D68 decision review (`DECISION.md`, logged on `#2837` and `#2029`), generates the machine-checked five-bar audit `docs/labs/m1-close-saved-model/evidence/m1-close-grade.json` and `stage-t-fz-fresh-gate-receipt.json` across all saved discrete models (`K16-A`, `K16-R`, `Rung-0 Q4`, `#2831` `A8 Q1`, `#2836` Stage T `T16_fz` & `T16_rel`, `#3267` `WithinCellCodeRerankerV1`, and `#2844` `TrunkAssemblyV1`), writes `docs/labs/m1-close-saved-model/README.md` (including the `CHARTER.md` §4.4 pivot note) and one row in `docs/research/notebook/m1-discrete-attention.md`, and preserves the sealed fresh panel (`0b7e754c…` / `a1cda529…`, `UNAVAILABLE_NOT_RUN`) while Bar (d) (`> 6,300` through `UORLUT01`) remains unmet (`References #2837`).

## Cancel probe (run first)

Ran on `origin/main` (`e9cfa88735f331d004c0dbf152ed569abf551421`):
1. Checked `docs/labs/m1-d11-artifact-scorer/instrument/`, `docs/harness/prep/m1-close-saved-model*`, and `docs/labs/m1-close-saved-model/**`:
   - `d11-scorer` has `export`, `rows`, `score`, `fresh-score`, and `toks`, and `validate_fresh_transfer_gate` (`src/fresh_transfer.rs`) consumes a `uor-r4.m1-fresh-transfer-gate/1` receipt, but no `m1_close` module or `d11-scorer m1-close-grade` command exists to evaluate all five M1 closing bars `(a)`–`(e)` (`CHARTER.md` §4.8, `D39`, `D42 Q6`, `D79`) or emit the `uor-r4.m1-fresh-transfer-gate/1` decision receipt (`classification: instrument`).
   - In `crates/uor-r4-discrete/src/trunk.rs`, `TrunkTables::select_level3_full_trunk` (`:724-764`) weights the coarse marginal query count `64 * c_q` above the joint `(query, read)` (`32 * c_qr`) and `(query, rec)` (`16 * c_qc`) counts and restricts selection to pre-trunk `&row.slots` (`A_W(q_read)`, `5,009 / 14,287 < 6,300`), lacking a joint-first hierarchical readout (`#3179` `QC = C1`) and post-rotation `OuterL1V1` full-writer scorer (`classification: instrument`).
   - Neither `docs/harness/prep/m1-close-saved-model-2026-10-26.md` nor `docs/labs/m1-close-saved-model/README.md` exists yet (`classification: instrument`).
2. Checked the sealed GPU receipts across all saved discrete models on `main` (`#2831`, `#2836`, `#2839`, `#3267`, `#2883`, `#2884`, `#2844`):
   - Bars `(a)` (`5,680` / `5,709` / `6,353 >= 4,894`), `(b)` (`G-Adm`: `5,009 / 14,287 >= 4,894` & `P_succ = 1,216 / 3,847 >= 676` on Stage T `T16_fz` Seeds 1 & 2), and `(c)` (`G-Occ`: `256 / 256 > 16` cells occupied, `16 / 4,096 = 0.39% <= 25%` max cell share) pass on `T16_fz`, while Bar `(d)` (`UORLUT01` integer `top-1` beating `Unigram = 1,226` and `T2 = 6,300` on two seeds) is not yet met (`455` and `371` through 4-table `UORLUT01` in `#2839` at `12,630`/`12,523` and `1,197`/`1,118` decode `tok/s`; `2,140` with `WithinCellCodeRerankerV1` in `#3267`; `2,988`/`2,989` with `TrunkAssemblyV1` in `#2844`), and Bar `(e)` (`fresh_v2` `0b7e754c…` / `a1cda529…`) remains sealed (`UNAVAILABLE_NOT_RUN`, `0` evaluations).
   - **Verdict:** Not cancelled. Proceed with building `m1_close` (`d11-scorer m1-close-grade`), fixing the `trunk.rs` joint-first hierarchy + post-rotation `OuterL1V1` scorer, registering `docs/harness/prep/m1-close-saved-model-2026-10-26.md` with fresh-context FIT + D68 review, and recording the five-bar audit and `CHARTER.md` §4.4 pivot note (`$0` GPU spend under `CHARTER.md` §3.5 / `D24`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-close-saved-model*`
- `docs/labs/m1-close-saved-model/**`
- `docs/loop/tasks/m1-close-saved-model/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

1. `docs/labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs` and `d11-scorer m1-close-grade` evaluate all five M1 closing bars `(a)`–`(e)` in exact integer arithmetic (zero float, zero squashing, zero matmul), emit both the `uor-r4.m1-fresh-transfer-gate/1` receipt and the `uor-r4.m1-close-saved-model/1` five-bar grade report, and pass failing-first negative-gate unit tests (`tests/m1_close.rs`).
2. `crates/uor-r4-discrete/src/trunk.rs` implements `select_level3_joint_hierarchy_admitted`, `select_level3_joint_hierarchy_global`, and `evaluate_joint_hierarchy_full_writer` with failing-first unit tests proving that joint-first hierarchical scoring beats marginal-first weighting on seen `(query, read)` pairs and that post-rotation `OuterL1V1` full-writer scoring (`A_W(q_trunk)`) removes the pre-rotation `A_W(q_read)` shortlist cap, without altering `driver_source_sha256` (`04c51654…`).
3. `docs/harness/prep/m1-close-saved-model-2026-10-26.md` receives fresh-context non-author `TEST FITNESS: FIT` and D68 decision review (`DECISION.md`, logged on `#2837` and `#2029`), `docs/labs/m1-close-saved-model/README.md` and `docs/labs/m1-close-saved-model/evidence/` record the verified five-bar audit and `CHARTER.md` §4.4 pivot note, `docs/research/notebook/m1-discrete-attention.md` adds one row, and all full gates (`uor-harness loop gates --full`) pass (`References #2837`).

## Stop and ask if

- Never stop to wait for Casey (`CHARTER.md` §5.1, D65–D69). If any check touches a `CHARTER.md` §3 constraint or the sealed fresh panel (`0b7e754c…` / `a1cda529…`) before Bars `(a)`–`(d)` pass on two seeds, refuse immediately.
