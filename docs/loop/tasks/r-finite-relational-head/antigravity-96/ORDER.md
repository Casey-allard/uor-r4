# ORDER: r-finite-relational-head, antigravity-96

Lab: antigravity. Item: [r-finite-relational-head](../../../../roadmap/items/r-finite-relational-head.md) (#2877). Base: `4cdefbe4a0d9e1b69255019fc3b31e6ad00a92ff`.

## Goal

The item's done-when reads:

> A registered run of the finite relational head with admission and ranking reported against the count tables.

Complete `#2877` (`r-finite-relational-head`) by:
1. Running a fresh-context D68 decision review (`DECISION.md`) reconciling `docs/harness/prep/r-finite-relational-head-2026-10-25.md` with the completed two-seed RTX 5090 GPU runs in `#2836` (`m1-bvg-k-stage-t-training`, `T16_rel` vs `T16_fz`), `#2844` (`m1-trunk-assembly-and-training`, `Trunk-Rel` vs `Trunk-FZ` on `ft/dev` and `fresh_v2`), `#3267` (`m1-within-cell-writer`, `stage-t-decomposition.json`), `#2885` (`r-read-off-vs-read-on`), and `#2830` (`m1-a8-finite-cuda-qualification`).
2. Extending `crates/uor-r4-discrete/src/stage_t.rs` and `crates/uor-r4-discrete/src/bin/stage-t-run.rs` (`grade-relation-head`) with exact integer grading (`RelationHeadGradeReport`) of the two-seed `H_relation` (`T16_rel`) vs `C_frozen_zero` (`T16_fz`) GPU runs, instrument gates (`Gate C1–C5` / `Step 0`, `Gate D40`, `Gate S`), within-arm `relation_off` ablation, unconditional and admission-matched count tables (`U`, `T2`, `T(c(x))`, `U_restr(A_W)`, `T_sym_restr(A_W)`, `T_restr(A_W)`, `G_restr(A_W)`), within-cell rerankers (`#3267`), and Stage 3 trunk transfer (`Level 0..3` on `ft/dev` and `fresh_v2`).
3. Writing `docs/labs/r-finite-relational-head/relation-head-grade.json`, updating `docs/labs/r-finite-relational-head/README.md` and `docs/harness/prep/r-finite-relational-head-2026-10-25.md`, recording the result row in `docs/research/notebook/m1-discrete-attention.md`, and setting `docs/roadmap/items/r-finite-relational-head.md` to `status: done`.

## Cancel probe (run first)

Class: `instrument`. Run at `4cdefbe4a0d9e1b69255019fc3b31e6ad00a92ff`:
1. Verified that `#2836` (`stage-t-20261025.tar.gz` `c004db50...`), `#2844` (`trunk-20261025.tar.gz` `4f4d4817...`), and `#3267` (`stage-t-decomposition.json`) already executed the two-seed `BVG-K` (`K = 16`) + D40 GPU training and evaluation of `H_relation` (`T16_rel` Seeds 1 & 2: `4,096` batches, `678` updates, `1,505` moves = `545` vec + `535` comb + `425` rel across `186` rel batches) and `C_frozen_zero` (`T16_fz` Seeds 1 & 2: `3,008` batches, `420` updates, `854` moves).
2. On both seeds, `H_relation` (`DEV b_3(Full) = 3,827`, `b_3(P_succ) = 1,144`, `top-1 = 371`, `T_restr(A_W) = 1,602`, `G_restr(A_W) = 2,364`, Trunk `Level 3 = 2,227 / 2,226` DEV & `2,427` `fresh_v2`) trails `C_frozen_zero` (`DEV b_3(Full) = 5,009`, `b_3(P_succ) = 1,216`, `top-1 = 455`, `T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084`, Trunk `Level 3 = 2,988 / 2,989` DEV & `3,163` `fresh_v2`) while beating its own `relation_off` ablation (`+74` admitted, `+44` `top-1`) and expanding distinct read codes (`27,132` vs `19,385` TRAIN).
3. Because $\max(C_{\text{frozen\_zero}}, C_{\text{route}}) \ge C_{\text{frozen\_zero}} > H_{\text{relation}}$ on both seeds, additional GPU training of `C_route` or `C_successor_only` cannot change the decision table outcome (`CHARTER.md` §3.5, §5.10 D67). Proceed to D68 decision review and exact integer grading.

## Files in scope

- `crates/uor-r4-discrete/src/stage_t.rs`
- `crates/uor-r4-discrete/src/bin/stage-t-run.rs`
- `docs/harness/prep/r-finite-relational-head-2026-10-25.md`
- `docs/labs/r-finite-relational-head/**`
- `docs/loop/tasks/r-finite-relational-head/antigravity-96/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/roadmap/items/r-finite-relational-head.md`
- `docs/roadmap/README.md`

## Done when

- D68 decision review approves and is logged on `#2877`; `stage-t-run grade-relation-head` writes `docs/labs/r-finite-relational-head/relation-head-grade.json`; unit tests, `loop gates --full`, and D66 exact-head review pass; `#2877` is set to `status: done` (`Closes #2877`).

## Stop and ask if

- Never stop and wait; decide via D68 review.
