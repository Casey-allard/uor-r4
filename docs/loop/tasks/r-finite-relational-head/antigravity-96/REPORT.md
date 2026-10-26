# REPORT: r-finite-relational-head, antigravity-96

Lab: antigravity. Item: [r-finite-relational-head](../../../../roadmap/items/r-finite-relational-head.md) (#2877). ORDER: [ORDER.md](ORDER.md). Decision card: [DECISION.md](DECISION.md).
Closes #2877.

## What did not work

- **Between-arm comparison against `C_frozen_zero` (`T16_fz`) on shared outer-L1 cell shortlist admission (`configuration negative`, `mechanism_negative: false`):**
  - Across both seeds (`seed = 1` and `seed = 2`) in the Stage T `BVG-K` (`K = 16`) + D40 admission-anchor GPU training run (`#2836`, `2x RTX 5090` `vast-57710022`, `stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`), modifying the 25 relation trits (`ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationPolicySpec::LearnV1`) during shared outer-L1 cell shortlist selection perturbed the predecessor-aligned `A8-Outer` query anchor $x_{L-1}$ (consistent with the zero-update finding in `#2885` `r-read-off-vs-read-on`).
  - Consequently, `H_relation` (`T16_rel`) admitted `3,827 / 14,287` `DEV` rows (`1,144 / 3,847` `P_succ`) vs `5,009 / 14,287` (`1,216 / 3,847` `P_succ`) on the matched `C_frozen_zero` control (`T16_fz`, $\Delta = -1,182$ `Full`, $-72$ `P_succ`), and trailed `C_frozen_zero` on raw `OuterL1V1` `DEV top-1` (`371` vs `455`, $\Delta = -84$; `top-8` `1,124` vs `1,484`), full-panel read-code table $T(c(x))$ (`3,680` vs `4,085`), admission-matched count tables `T_restr(A_W)` (`1,602` vs `2,140`, $\Delta = -538$), `G_restr(A_W)` (`2,364` vs `3,084`, $\Delta = -720$), `U_restr(A_W)` (`942` vs `1,127`), `T_sym_restr(A_W)` (`1,377` vs `1,909`), within-cell read-code reranker (`1,602` vs `2,140` in `#3267`), and Stage 3 Trunk `Level 3` `top-1` (`2,227 / 2,226` vs `2,988 / 2,989` on `ft/dev` and `2,427` vs `3,163` on `fresh_v2` in `#2844`).
  - Under `CHARTER.md` §4.1 (D22), this closes only the uncoupled single-pass `Relation + A8-Outer` shortlist configuration where relation-perturbed $q^{\text{read}}(x)$ drives outer-L1 cell shortlist admission, never the `relation-read` mechanism (`ACTIVE`).
- **Additional GPU training of `C_route` or `C_successor_only` refused as outcome-invariant (`CHARTER.md` §3.5, §5.10 D67/D68):**
  - Both positive rows of `docs/harness/prep/r-finite-relational-head-2026-10-25.md` §3 require $\text{H\_relation} \ge \max(\text{C\_frozen\_zero}, \text{C\_route}) + F_{\text{full}}$ on both seeds. Because $\max(\text{C\_frozen\_zero}, \text{C\_route}) \ge \text{C\_frozen\_zero} > \text{H\_relation}$ already holds strictly across both seeds on every admission and ranking metric, a `$5.05` GPU training run of `C_route` or `C_successor_only` cannot change the decision table verdict and was refused by D68 decision review (`DECISION.md`, reviewer `b9f7ec13-d459-41e6-8b54-39ae7c196d9b`, `#2877` comment `6421371461`).
- **TDD negative verification (`stage_t::tests::relation_head_grader_verifies_gates_within_arm_lift_and_outcome_invariance`):**
  - Verified that `cargo test --offline -p uor-r4-discrete --lib stage_t::tests::relation_head_grader_verifies_gates_within_arm_lift_and_outcome_invariance -- --exact` fails when `grade_relation_head` is set aside, passes when restored, and refuses inputs that breach `Step 0`, `Gate D40`, `Gate S`, or outcome invariance.

## What changed

- **Size:** `10` files modified/added (`2` Rust code/test files: `crates/uor-r4-discrete/src/stage_t.rs` `+637` lines, `crates/uor-r4-discrete/src/bin/stage-t-run.rs` `+338/-3` lines; `8` documentation/receipt/task files: `docs/harness/prep/r-finite-relational-head-2026-10-25.md`, `docs/labs/r-finite-relational-head/README.md`, `docs/labs/r-finite-relational-head/relation-head-grade.json`, `docs/research/notebook/m1-discrete-attention.md`, `docs/roadmap/items/r-finite-relational-head.md`, `docs/roadmap/README.md`, and `docs/loop/tasks/r-finite-relational-head/antigravity-96/{ORDER.md,DECISION.md,REPORT.md}`).
- **Suborders:**
  - `local/antigravity/antigravity-96/orders/1.md`: fresh-context D68 decision review (`b9f7ec13-d459-41e6-8b54-39ae7c196d9b`, `DECISION REVIEW VERDICT: APPROVE`).
  - `local/antigravity/antigravity-96/orders/2.md`: fresh-context D66 exact-head code reviewer (`4d6c6935-aa59-4ef6-8141-07f261628e87`) and claims reviewer (`9612a098-96cf-405e-a232-897cc13ca03c`).

## Measurements

- **Command:** `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p uor-r4-discrete --bin stage-t-run -- grade-relation-head --out docs/labs/r-finite-relational-head/relation-head-grade.json`
- **Conditions:** Two-seed RTX 5090 GPU receipts from `#2836` (`stage-t-20261025.tar.gz` `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`, `grade.json` `7917ec85...`, `step0-receipt.json` `0ca7d955...`), `#2844` (`trunk-20261025.tar.gz` `4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35`, `grade.json` `8413be77...`), and `#3267` (`stage-t-decomposition.json` `e9dd940c...`), on `train_stream.bin` (`7c74a4ef...`, `1,049,856` rows), `p_succ_train_v1.txt` (`f85466ff...`, `3,847` rows, D40 floor `1,187`), `dev_panel_v1.bin` (`29e536d2...`, `14,287` rows, `3,847` `P_succ` rows), and `fresh_v2` (`14,287` rows).
- **Results (`relation-head-grade.json` `fa8c7e02ae0ebe7c1fb0ffc7702923031373baaa6670e87df45d5a16fa66d594`):**
  1. **Instrument gates (`2 / 2` seeds PASS):** `Gate C1–C5` + `Step 0` (`5/5 PASS`), `Gate D40` (`TRAIN` anchor `1,211 -> peak 1,849 -> final 1,286 >= 1,187` on `T16_rel`, `1,211 -> peak 1,823 -> final 1,342 >= 1,187` on `T16_fz`; `DEV b_3(P_succ) = 1,144` and `1,216 >= 676`), and `Gate S` (`186 >= 16` committed relation block updates and `425 >= 16` committed relation moves per seed on `T16_rel`, `0` on `T16_fz`; `11,562` / `11,201 context tok/s`).
  2. **Within-arm relational gain (`H_relation` = `T16_rel` Seeds 1 & 2):** Lifts `DEV top-1` `2.79x` over `K = 1` `T1_rel` (`133 -> 371`, `+238`), beats its own `relation_off` ablation on `DEV b_3(Full)` (`3,827` vs `3,753`, `+74`), `DEV b_3(P_succ)` (`1,144` vs `1,129`, `+15`), `DEV top-1` (`371` vs `327`, `+44`), `DEV top-8` (`1,124` vs `1,063`, `+61`), and `strict_wins` (`172` vs `123`, `+49`), expands distinct read codes to `27,132` TRAIN / `4,429` DEV (vs `19,385` / `4,067` on `T16_fz`), and achieves a larger Stage 3 `Level 2` recurrence lift (`+67` / `+53` rows on `ft/dev` vs `+33` on `T16_fz`).
  3. **Between-arm and count-table comparison (`H_relation` vs `C_frozen_zero`, `n = 14,287` DEV):** Unconditional `U = 1,226` (`top-8 = 4,060`), `T2 = G = 6,300` (`top-8 = 11,263`), $T(c(x)) = 3,680$ (`T16_rel`) vs `4,085` (`T16_fz`); admission `DEV b_3(Full) = 3,827` vs `5,009` (`-1,182`), `b_3(P_succ) = 1,144` vs `1,216` (`-72`); writer `top-1 = 371` (`P_succ = 78`) vs `455` (`P_succ = 107`), `top-8 = 1,124` vs `1,484`; admission-matched `U_restr(A_W) = 942` vs `1,127`, `T_sym_restr(A_W) = 1,377` vs `1,909`, `T_restr(A_W) = 1,602` (`top-8 = 3,007`, `P_succ = 327`) vs `2,140` (`top-8 = 4,006`, `P_succ = 376`), `G_restr(A_W) = 2,364` (`top-8 = 3,529`, `P_succ = 496`) vs `3,084` (`top-8 = 4,674`, `P_succ = 517`); within-cell reranker `1,602` ($d = +1,231 > 74$) vs `2,140` ($d = +1,685 > 86$, with `1,190 / 1,291 = 92.18%` missed `T_restr` hits lost within target's own cell); Stage 3 Trunk `Level 3` `2,227 / 2,226` vs `2,988 / 2,989` (`ft/dev`) and `2,427` vs `3,163` (`fresh_v2`).
  4. **Verdict:** `RELATIONAL_WITHIN_ARM_GAIN_TRAILS_FROZEN_ZERO_ADMISSION` (`mechanism_negative: false`, `outcome_invariance_verified: true`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-discrete --no-fail-fast`: PASS.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p uor-r4-discrete --bin stage-t-run -- grade-relation-head --out docs/labs/r-finite-relational-head/relation-head-grade.json`: PASS (`fa8c7e02ae0ebe7c1fb0ffc7702923031373baaa6670e87df45d5a16fa66d594`).
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`: PASS.
- `uor-harness loop report --check --order docs/loop/tasks/r-finite-relational-head/antigravity-96/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS.
- `uor-harness loop gates --full`: PASS (`4/4` gates green).

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`.

## Still open

- `#2877` (`r-finite-relational-head`) is `done` (`Closes #2877`). Next items on the `m1-discrete-attention` line (`#2884` `r-ranking-with-admission-intact`, `#2883` `r-rank-objective-keeps-cells`, and `#2834` `m1-address-key-full-vector`) evaluate decoupled shortlist admission and within-cell ranking.

## Plain summary

This unit (`antigravity-96`) completes `r-finite-relational-head` (`#2877`, *"Train the finite relational head"*) by grading the two-seed RTX 5090 GPU runs of the 25-trit finite relational head (`H_relation` = `T16_rel`, where the model learns ternary weights over five discrete tree-link relations across five tree levels) against the matched `C_frozen_zero` control (`T16_fz`, where those 25 relation trits stay frozen at zero) and the lawful count tables (`U` unigram, `T2` next-word bigram, $T(c(x))$ exact read-code table, and their shortlist-restricted counterparts `U_restr(A_W)`, `T_sym_restr(A_W)`, `T_restr(A_W)`, `G_restr(A_W)`). All instrument checks (`Step 0`, `Gate D40` copy-successor floor `1,187 / 3,847`, and `Gate S` with `186` committed relation batches / `425` relation moves per seed) pass on both seeds. Within its own arm, learning the 25 relation trits lifts `DEV top-1` `2.79x` over `K = 1` (`133 -> 371`), beats its `relation_off` ablation (`+74` admitted rows, `+44` `top-1`), expands distinct read codes by `+40%` (`27,132` vs `19,385`), and yields a larger Stage 3 recurrence lift (`+67 / +53` vs `+33`). However, because modifying the relation trits during single-pass outer-L1 cell shortlist selection perturbs the query token's predecessor-aligned cell address (`3,827` vs `5,009` admitted `DEV` rows), `H_relation` trails `C_frozen_zero` on `DEV` admission (`-1,182`), raw writer `top-1` (`371` vs `455`), `T_restr(A_W)` (`1,602` vs `2,140`), `G_restr(A_W)` (`2,364` vs `3,084`), and Stage 3 Trunk `Level 3` (`2,227 / 2,226` vs `2,988 / 2,989` on `ft/dev` and `2,427` vs `3,163` on `fresh_v2`), closing only the single-pass outer-L1 shortlist configuration (`CHARTER.md` §4.1) while keeping `relation-read` `ACTIVE`.
