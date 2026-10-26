# Decision Card: `m1-close-saved-model` (`#2837` / `#2029`, `antigravity-105`)

- **Date:** 2026-10-26
- **Lab:** `antigravity` (`antigravity-105`, draft PR `#3301`)
- **Item:** `m1-close-saved-model` (`#2837`, Step 8 of `PLAN.md`), Milestone M1 (`#2029`)
- **Rules:** `CHARTER.md` §2 M1, §4.1–§4.5, §4.8 (`D39`), §5.1 (`D67`, `D68`)

## 1. Whole-project context read first (`D68`)

Read `CHARTER.md`, `PLAN.md`, `docs/integration/DECISIONS.md` (`D22`, `D32`, `D37`, `D39`, `D66`–`D69`), `docs/roadmap/items/m1-close-saved-model.md` (`#2837`), `docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`), `docs/roadmap/items/d11-exporter-float-trunk.md` (`#2818`), `docs/research/notebook/m1-discrete-attention.md`, `docs/labs/m1-export-gap-and-toks/README.md` (`#2839`), `docs/labs/m1-fresh-panel-transfer/README.md` (`#2841`), `docs/labs/m1-trunk-assembly-and-training/README.md` (`#2844`), `docs/labs/r-within-cell-ranking/README.md` (`#3267`), and `docs/labs/r-ranking-with-admission-intact/README.md` (`#2884` / `#3295`).

## 2. Decision

1. **Five-bar M1 verdict (`m1_close_pass = false`, `OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`):**
   - Keep `m1-close-saved-model` (`#2837`) and Milestone M1 (`#2029`) **OPEN** (`status: ready` in `docs/roadmap/items/m1-close-saved-model.md`, `References #2837` in `REPORT.md`).
   - Under `CHARTER.md` §2 and `m1-close-saved-model.md`, M1 closes only when all five closing bars (a)–(e) pass simultaneously across `>= 2` seeds (`NOT_RUN` is never `PASS`):
     - **Bars (a), (b), and (c) PASS** on the two-seed exported Stage T `UORLUT01` artifacts (`T16_fz` `8a2c3cb4…` / `33ce2c55…`, `455 / 14,287` top-1, `0 / 14,287` export gap, `B4` pass, `12,630` / `12,523 tok/s` laptop decode, `232,879` / `232,126 tok/s` GPU response; `T16_rel` `166403ca…` / `3e52357b…`, `371 / 14,287` top-1, `0 / 14,287` export gap, `B4` pass, `1,197` / `1,118 tok/s` laptop decode, `232,879` / `233,284 tok/s` GPU response).
     - **Bar (d) FAILS** across all 8 saved discrete models on `main` (`455` and `371` on 4-table `UORLUT01`; `2,135` on `#3267` `WithinCellCodeRerankerV1`; `2,988`–`2,989` and `2,227`–`2,226` on `#2844` `TrunkAssemblyV1` vs required `DEV_T2_TOP1 + 24 = 6,324 / 14,287`).
     - **Bar (e) (`Fresh V2` transfer) remains `UNAVAILABLE_NOT_RUN`** with the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`, seal `df5d074a…`) unopened (`dev_decision = "FAIL_BAR_D_HOLD_FRESH_SEAL"`, refused by `validate_fresh_transfer_gate`).
2. **Classification (`instrument` + `mechanism` configuration bound under `D22`):**
   - `classification: instrument` for the two tooling/export gaps repaired in this unit (`d11-scorer m1-close-grade` in `docs/labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs` and `select_level3_joint_hierarchy_admitted` / `select_level3_joint_hierarchy_global` / `evaluate_joint_hierarchy_full_writer` in `crates/uor-r4-discrete/src/trunk.rs`).
   - `classification: mechanism` configuration negative (`D22`) on the 4-table Stage 1/2 `UORLUT01` configuration (`discrete.table`, `discrete.cells`, `discrete.routes`, `discrete.gates`) scored by `IntegerDiscreteModel::evaluate_prefix`, which is structurally bounded above on `ft/dev` by pre-rotation 8-cell shortlist admission $A_W(q_{\text{read}}) = 5,009 < 6,300$ (`G_restr(A_W) = 3,084 < 6,300`) and unconditioned 16-bit read-code collision $T(c_{\text{read}}) = 4,085 < 6,300$.
3. **Registered `CHARTER.md` §4.4 two-path pivot to clear Bar (d) and unlock Bar (e):**
   - **Path A (Discrete post-rotation joint-hierarchy Hamilton trunk exported to D11):** Score `evaluate_joint_hierarchy_full_writer` (`64 * c_qr + 32 * c_qc + 8 * c_rr + 2 * c_r + c_q` + `DiscreteHamiltonGateV1` rotation + post-rotation `writer_score_full_outer_l1(&q_trunk)`) across two seeds on `ft/dev` and extend the D11 export/reader schema to include the trunk tables.
   - **Path B (Offline float-trained SpiralCore / FBS trunk exported via `#2818`):** Train the multi-layer geometric trunk (`SpiralCore` / `FBS`) offline in float (`CHARTER.md` §3.6 / `D37`) and export to `UORLUT01` via `d11-exporter-float-trunk` (`#2818`).
