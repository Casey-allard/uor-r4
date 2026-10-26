# REPORT: m1-close-saved-model, antigravity-105

Lab: antigravity. Item: [m1-close-saved-model](../../../../roadmap/items/m1-close-saved-model.md). Order: [ORDER.md](ORDER.md).

References #2837

## What did not work

- **No saved discrete model on `main` satisfies all five M1 closing bars (a)–(e) simultaneously (`m1_close_pass = false`, `OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED`), so `#2837` and `#2029` remain open and `Fresh V2` remains sealed (`UNAVAILABLE_NOT_RUN`):**
  - Across the two-seed exported Stage T `UORLUT01` integer artifacts (`T16_fz` `8a2c3cb4da4c80643ed992f886ff858ba2d739bc901ab23c84805619da1dd7f5` and `33ce2c5531ceac304823879c84417811408446c509665ef4c2f62196859078fd`; `T16_rel` `166403ca59268a16b0c5c7f7c225a15a3f1dbef797dfba4d38daec5c9b04fb5d` and `3e52357b3b7fdcb123db339c5767b5685d7ccbc80511a8431b538ce0936de3a0`), **Bars (a), (b), and (c) PASS** (`533,248` B accepted by `IntegerDiscreteModel::from_bytes`, `0 / 14,287` row mismatches between `d11-scorer score` and `discrete-stack eval-corpus`, planted-fault `B4` pass, laptop `12,630` / `12,523 tok/s` and `1,197` / `1,118 tok/s` decode, GPU `232,879` / `232,126` / `233,284 tok/s` response), but **Bar (d) FAILS** on `ft/dev` (`14,287` rows): `455 / 14,287` (`T16_fz`, deficit `-5,869` vs required `DEV_T2_TOP1 + 24 = 6,324`) and `371 / 14,287` (`T16_rel`, deficit `-5,953` vs `6,324`).
  - In the 4-table Stage 1/2 `UORLUT01` schema (`discrete.table`, `discrete.cells`, `discrete.routes`, `discrete.gates`) scored by `IntegerDiscreteModel::evaluate_prefix`, every prefix $x$ is mapped to a 16-bit read code $c_{\text{read}}(x) \in \mathcal{O}_{16}^4$ before 8-cell shortlist admission $A_W(q_{\text{read}})$ (`5,009 / 14,287 = 35.06% < 6,300`; $G_{\text{restr}}(A_W) = 3,084 < 6,300$) and unconditioned read-code table ceiling $T(c_{\text{read}}) = 4,085 < 6,300$ (`3,414` on `Rung-0 Q4`, projected ceiling `3,552` in `#3179` `QB = B2`).
  - `#3267` `WithinCellCodeRerankerV1` (`2,135 / 14,287`, `99.77%` of `T_restr(A_W) = 2,140`) and `#2844` `TrunkAssemblyV1` (`Level 3 = 2,988`–`2,989 / 14,287` on `T16_fz` and `2,227`–`2,226 / 14,287` on `T16_rel`) beat `Unigram` (`1,226 / 14,287`) by `+909` to `+1,763` rows on `ft/dev`, but their side-tables (`M_code` `203,316` B and `TrunkTables`) are omitted from the 4-table `UORLUT01` export and remain bounded below `T2 = 6,300` by pre-rotation $A_W(q_{\text{read}}) = 5,009$ and `#2844`'s marginal `64 * c_q` weight inversion.
- **Two instrument gaps found and fixed (`classification: instrument`):**
  1. `docs/labs/m1-d11-artifact-scorer/instrument/` (`d11-scorer`) lacked a five-bar M1 closing-bar evaluator (`m1-close-grade`) auditing Bars (a)–(e) in exact integer arithmetic and enforcing that `NOT_RUN` (`fresh_transfer_report: None`) is never `PASS` while refusing any premature unsealing of `Fresh V2` (`0b7e754c…` / `a1cda529…`) when `bars_a_to_d_pass == false`. Our failing-first test (`negative_gate_checks_on_each_of_five_m1_bars_and_prospective_fresh_seal` in [`tests/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/m1_close.rs)) failed when `m1_close_pass` ignored `bar_e` and passed once `bars_a_to_d_pass && bar_e` was restored.
  2. In [`crates/uor-r4-discrete/src/trunk.rs`](../../../../crates/uor-r4-discrete/src/trunk.rs) (`#2844`), `select_level3_full_trunk` weighted marginal query count `64 * C_query(q_tok, y)` above joint `32 * C_query,read((q_tok, c_read), y)` and `16 * C_query,rec((q_tok, c_rec), y)` and restricted candidates to pre-rotation `&row.slots` ($A_W(q_{\text{read}})$). Our failing-first test (`joint_hierarchy_fixes_level3_marginal_query_inversion_and_pre_rotation_shortlist_cap` in [`crates/uor-r4-discrete/src/trunk.rs`](../../../../crates/uor-r4-discrete/src/trunk.rs)) failed (`left: 0, right: 17`) when stubbed to `select_level3_full_trunk` and passed once the joint-first hierarchy (`64 * c_qr + 32 * c_qc + 8 * c_rr + 2 * c_r + c_q`) and post-rotation full `OuterL1V1` writer (`evaluate_joint_hierarchy_full_writer`) were active.

## What changed

- **Step 1 — Joint-first hierarchy and post-rotation full `OuterL1V1` writer in [`crates/uor-r4-discrete/src/trunk.rs`](../../../../crates/uor-r4-discrete/src/trunk.rs):**
  - Added `TrunkTables::level3_joint_hierarchy_score`, `TrunkTables::select_level3_joint_hierarchy_admitted`, `TrunkTables::select_level3_joint_hierarchy_global`, and `evaluate_joint_hierarchy_full_writer` (pure integer arithmetic, zero float tokens or tuple-index literals, preserving `DRIVER_SOURCE_SHA256 = "04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d"`).
- **Step 2 — Five-bar M1 closing-bar evaluator in [`docs/labs/m1-d11-artifact-scorer/instrument/`](../../../labs/m1-d11-artifact-scorer/instrument/):**
  - Added [`src/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/m1_close.rs), [`tests/m1_close.rs`](../../../labs/m1-d11-artifact-scorer/instrument/tests/m1_close.rs), and `d11-scorer m1-close-grade OUT_GRADE_JSON OUT_FRESH_GATE_JSON` in [`src/main.rs`](../../../labs/m1-d11-artifact-scorer/instrument/src/main.rs), and generated [`docs/labs/m1-close-saved-model/evidence/m1-close-grade.json`](../../../labs/m1-close-saved-model/evidence/m1-close-grade.json) and [`docs/labs/m1-close-saved-model/evidence/stage-t-fz-fresh-gate-receipt.json`](../../../labs/m1-close-saved-model/evidence/stage-t-fz-fresh-gate-receipt.json).
- **Step 3 — Pre-registration sheet, D68 decision card, lab note, and notebook row:**
  - Created [`docs/harness/prep/m1-close-saved-model-2026-10-26.md`](../../../harness/prep/m1-close-saved-model-2026-10-26.md), [`docs/loop/tasks/m1-close-saved-model/antigravity-105/DECISION.md`](DECISION.md), [`docs/labs/m1-close-saved-model/README.md`](../../../labs/m1-close-saved-model/README.md), and added one row to [`docs/research/notebook/m1-discrete-attention.md`](../../../research/notebook/m1-discrete-attention.md).

## Measurements

- **Headline throughput (`CHARTER.md` §3.12, exported `UORLUT01` artifacts `533,248` B):**
  - `stage-t-t16-fz` (`FrozenZeroV1`, `Flock` read, `B = 4`, `C = 256`): Seed 1 (`8a2c3cb4…`) `12,630 tok/s` laptop decode (`12,551 tok/s` prefill), `232,879 tok/s` GPU response (`56,836,282 tok/s` GPU context); Seed 2 (`33ce2c55…`) `12,523 tok/s` laptop decode (`12,663 tok/s` prefill), `232,126 tok/s` GPU response (`56,652,733 tok/s` GPU context).
  - `stage-t-t16-rel` (`LearnV1`, `Relation` read, `B = 4`, `C = 256`): Seed 1 (`166403ca…`) `1,197 tok/s` laptop decode (`1,237 tok/s` prefill), `232,879 tok/s` GPU response (`56,836,282 tok/s` GPU context); Seed 2 (`3e52357b…`) `1,118 tok/s` laptop decode (`1,179 tok/s` prefill), `233,284 tok/s` GPU response (`56,935,299 tok/s` GPU context).
- **Five-bar M1 closing audit (`m1-close-grade.json`):**
  - `m1_close_pass = false`, `m1_milestone_status = "OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED"`, `fresh_v2_seal_intact = true`.
  - `stage-t-t16-fz`: `bar_a = true`, `bar_b = true` (`0 / 14,287` row mismatches, `B4` pass), `bar_c = true`, `bar_d = false` (`455 / 14,287 < 6,324`, deficit `5,869`), `bar_e = false` (`UNAVAILABLE_NOT_RUN`, `dev_decision = "FAIL_BAR_D_HOLD_FRESH_SEAL"`).
  - `stage-t-t16-rel`: `bar_a = true`, `bar_b = true` (`0 / 14,287` row mismatches, `B4` pass), `bar_c = true`, `bar_d = false` (`371 / 14,287 < 6,324`, deficit `5,953`), `bar_e = false` (`UNAVAILABLE_NOT_RUN`, `dev_decision = "FAIL_BAR_D_HOLD_FRESH_SEAL"`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --lib --no-fail-fast trunk`: `6 / 6` passed.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-discrete --lib --no-fail-fast driver_source_sha256_matches_the_pin`: `1 / 1` passed.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --target aarch64-apple-darwin --no-fail-fast`: `20 / 20` passed (`2` `m1_close`, `10` `parity`, `5` `stage1`, `3` `toks`).
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/antigravity-105/ORDER.md`: `PASS`.
- `uor-harness loop gates --full`: `11 / 11` gates `GREEN`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`.

## Still open

- Executing the `CHARTER.md` §4.4 pivot (Path A discrete post-rotation joint-hierarchy Hamilton trunk exported to D11; Path B offline float-trained SpiralCore/FBS trunk exported via `#2818` `d11-exporter-float-trunk`) to clear Bar (d) (`>= 6,324 / 14,287` on `ft/dev` across two seeds), running `d11-scorer fresh-score` (`#2841`) once on the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`), and closing `#2837` and `#2029` when all five bars (a)–(e) pass.

## Plain summary

We built `d11-scorer m1-close-grade` (`src/m1_close.rs` + `tests/m1_close.rs`) to audit all 8 saved discrete models on `main` against M1 closing bars (a)–(e) in exact integer arithmetic, added the joint-first `Level 3` hierarchy and post-rotation full `OuterL1V1` writer (`evaluate_joint_hierarchy_full_writer`) to `crates/uor-r4-discrete/src/trunk.rs`, confirmed that Stage T `T16_fz` (`455 / 14,287`, `12,630` / `12,523 tok/s`) and `T16_rel` (`371 / 14,287`, `1,197` / `1,118 tok/s`) pass Bars (a)–(c) at zero export gap while failing Bar (d) (`>= 6,324 / 14,287`), kept `#2837` and `#2029` open with the sealed `Fresh V2` slice intact (`UNAVAILABLE_NOT_RUN`), and registered the `CHARTER.md` §4.4 two-path pivot.
