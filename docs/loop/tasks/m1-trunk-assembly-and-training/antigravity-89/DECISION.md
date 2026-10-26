# DECISION (D68, D67, and FIT): Stage 3 Trunk Assembly and Training (`m1-trunk-assembly-and-training`, `#2844`)

Lab: `antigravity`. Unit: `antigravity-89`. Item: [`m1-trunk-assembly-and-training`](../../../../roadmap/items/m1-trunk-assembly-and-training.md) (`#2844`, Milestone M2 `#2030`, effective milestone M1 via `#2833`). Reviewed at commit `6a7848e40ba414a5d0d08a5cba69aca9ece905af`.

## 1. The Question

Now that all four dependencies of `m1-trunk-assembly-and-training` (`#2844`) are `done` on `main` (`m1-position-mechanism` `#2843`, `m1-normalization-mechanism` `#2842`, `m1-bvg-k-stage-t-training` `#2836`, and `m1-d11-exporter-discrete` `#2838`), how should we operationalize `#2844`'s Stage 3 bar (*"beat Stage 2's best by more than the seed-to-seed noise, with a fresh-panel number"*) across the two seeds (`Seed 1`, `Seed 2`) and two attention policies (`T16_fz`, `T16_rel`) from `#2836`, and should the GPU trunk training and evaluation run (`trunk-run`) proceed now?

## 2. The Decision

1. **Operationalize the Stage 3 bar (`trunk.rs::grade_trunk_arms`, `docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md` §5) conservatively against all three Stage 2 tiers simultaneously across both seeds (`Seed 1`, `Seed 2`) on both `ft/dev` (`14,287` rows) and the sealed fresh transfer panel `fresh_v2` (`14,287` rows)**:
   - Every trunk readout level (`Level 0` through `Level 3`) emits a **single 32-int trunk output vector** $q^{\text{trunk}}_L(x) \in \mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$ per row, scored directly by the unmodified `OuterL1V1` writer (`CHARTER.md` §3.12).
   - To pass (`STAGE3_PASS_BEATS_STAGE2_BEST_ON_DEV_AND_FRESH`), all 5 Step 0 GPU controls (`G-T1`..`G-T5`) must pass (`0` mismatches), all 4 arms must pass `D11` (`uor-r4.lut-discrete/1`) integer artifact round-trip verification (`trunk-model.d11`), and for each policy (`T16_fz`, `T16_rel`) across both seeds (`Seed 1`, `Seed 2`):
     1. `Level 1` (`hamilton_read_only`) beats `Level 0` (`writer_raw`: `455` on `T16_fz`, `371` on `T16_rel`) by $> \max(\text{seed\_noise}, \lceil 2\sqrt{b+c}\rceil)$ on both `ft/dev` and `fresh_v2`;
     2. `Level 2` (`hamilton_read_recurrence`) beats `Level 1` (`T_restr(A_W)` equivalent: `2,140` on `T16_fz`, `1,602` on `T16_rel`) and `Level 2 [C_rec_null]` by $> \text{seed\_noise}$ on both `ft/dev` and `fresh_v2`;
     3. `Level 3` (`hamilton_full_trunk`) beats Stage 2's highest restricted table (`G_restr(A_W)`: `3,084` on `T16_fz`, `2,364` on `T16_rel`) and `Level 3 [C_rec_null]` by $> \text{seed\_noise}$ (and beats `Level 1` by $> \max(\text{seed\_noise}, \lceil 2\sqrt{b+c}\rceil)$) on both `ft/dev` and `fresh_v2`.
2. **Proceed immediately with the `1x RTX 5090` GPU execution (`trunk-run step0`, `trunk-run arm` on all 4 Stage T arms, and `trunk-run grade`) under `docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md` (`sha256 = bbb641f9115634b03d319b8937d534ed987ebfa18f3d3e73df8ff45d9f554ce0`; pre-stamp `b49051c0f5ef3294089b21dd405d247341c1cf28bd179b6807870eb9373afb07`).**

## 3. Why (Whole-Project Context & M1 $\to$ M2 Chain)

1. **Whole-Project Unblock Path (`PLAN.md` `D77`, `D79`)**:
   - `#2844` (`m1-trunk-assembly-and-training`) is the direct prerequisite of M1 item `#2833` (`m1-address-as-read`), M2 item `#2851` (`m2-pin-train-mask-sheet` discrete branch, which unblocks `m2-train-grounded-replies`), and downstream M2/M6/M7 trunk items (`#2867`, `#2865`).
2. **Why Stage 2 (`#2836`) Pointed Directly to Single-Vector Trunk Assembly**:
   - In `#2836`, `BVG-K` (`K = 16`) raised full-DEV target cell admission to `5,009 / 14,287` (`35.06%`) on `T16_fz` and distinct TRAIN read codes to `19,385` (`T16_fz`) and `27,132` (`T16_rel`), proving that the checkpoint's admitted shortlist $A_W(x)$ and read code $c_{\text{read}}(x)$ already support `2,140 / 14,287` (`T_restr(A_W)`) and `3,084 / 14,287` (`G_restr(A_W)`) exact hits, while the raw `OuterL1V1` writer on the untransported read vector $q^{\text{read}}(x)$ scored `455` (`T16_fz`) and `371` (`T16_rel`).
   - Assembling causal prefix-holonomy recurrence (`DiscreteQuaternionRecurrenceV1`), multiplicity-matched orbit-root snap normalization (`OrbitRootSnapNormV1`), and the transitive non-linear geometric feed-forward gate (`DiscreteHamiltonGateV1`) rotates $q^{\text{read}}(x)$ into a single canonical orbit vector $q^{\text{trunk}}_L(x) = \mathrm{codebook}[y^*_L(x)] \in \mathcal{O}_{16}^4$ scored by the unmodified `OuterL1V1` writer, testing whether the assembled geometric trunk recovers and exceeds Stage 2's within-shortlist capacity on both `ft/dev` and `fresh_v2`.

## 4. Cost to Reverse

Reversible by git revert of the `#2844` commits; `training::driver_source_sha256` (`04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`) and all existing discrete kernel and training paths are untouched.

## 5. Independent Fresh-Context Review Record (`FIT`, `D68`, `D67`)

- **Reviewer**: `antigravity` fresh-context non-author subagent (`65458246-8c07-489e-85d5-935d01db8259`) at commit `6a7848e40ba414a5d0d08a5cba69aca9ece905af`
- **Prep Sheet**: [`docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md`](../../../harness/prep/m1-trunk-assembly-and-training-2026-10-25.md) (`sha256 = bbb641f9115634b03d319b8937d534ed987ebfa18f3d3e73df8ff45d9f554ce0`; pre-stamp `b49051c0f5ef3294089b21dd405d247341c1cf28bd179b6807870eb9373afb07`)
- **Verdicts**:
  - `TEST FITNESS: FIT`
  - `DECISION REVIEW (D68): APPROVE`
  - `PRE-RUN REVIEW (D67): APPROVE`
