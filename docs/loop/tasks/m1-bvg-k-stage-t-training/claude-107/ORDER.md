# ORDER: m1-bvg-k-stage-t-training, claude-107

Lab: claude. Item: [m1-bvg-k-stage-t-training](../../../../roadmap/items/m1-bvg-k-stage-t-training.md) (#2836). Base: `cada129ea`.

## Goal

Run the registered, FIT Stage T sheet, [`m1-bvgk-stage-t-training-2026-10-25.md`](../../../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md) (sha256 `c43c4ea1…`; FIT on #2835, `6395412609`), and record its outcome against the item's done-when:

> Stage 2 bar: 'beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds.'

That needs four pieces of instrument work first, because the D40-audited runner is K = 1 and seed 1 only (`attention_audited::validate_config`):
1. **The audited block path.** `attention_audited` drives `transition_block` at K = 16 under the unchanged `AuditPolicy::registered()` controller, for the sheet's `T16_fz` and `T16_rel` arms, at seeds 1 and 2. The K = 1 path stays byte for byte.
2. **Seed-2 A8 configs** for both relation policies, made exactly like the seed-1 configs in `docs/labs/a8-training-configs-2026-10-19/evidence/`.
3. **The sheet's five Step 0 controls** (`AKd`, `V1d`, `KId`, `V8d`, `V9d`, §4) as a CUDA check that runs before any training metric is read.
4. **Step 3's evaluation:** DEV top-1, top-8 and admitted rows per checkpoint, and the W2 admission-matched tables restricted to each checkpoint's admitted rows (`T_restr(A_W)`, `G_restr(A_W)`), by the W2 instrument's rules.

Then: the D67 pre-run review, the probe ladder, the run, and the REPORT with the notebook row.

## Cancel probe (run first)

Run at `cada129ea`. Class: instrument (the runner lacks the block path), not mechanism.

| Check | Result |
|---|---|
| The four dependencies are `status: done` (`m1-w2-capacity-capture`, `m1-bvg-k-block-apply-c2`, `m1-a8-full-dose-rerun`, `m1-w2-code-utilization`) | **Holds** |
| The sheet is unchanged since its FIT: one commit, `14fc2cbe4` (#3204) | **Holds** |
| Its inputs are in the HF store: the A8 dose archive `results/antigravity/a8-full-dose-rerun-2026-10-25/` and the W2 records `results/claude/w2-capacity-2026-10-25/` and `w2-utilization-2026-10-25/` | **Holds** |
| The audited runner can run a T16 arm today | **Fails:** `validate_config` refuses `coordinate_block != ONE` and `seed != 1`. Instrument gap, so the goal's four pieces are built. |
| Stage T is still warranted after W2 and the code-utilization diagnostic | **Fails.** Decided in [DECISION.md](DECISION.md) (D68): W2's row 3 says "do not change the learning law yet", and the diagnostic routes A1a to #2834. Stage T waits on #2834. **The unit is cancelled here, before any build.** |

## Files in scope

- `crates/uor-r4-discrete/**` (the audited block path, the Step 0 controls, the evaluation)
- `docs/harness/prep/m1-bvg-k-stage-t-training*`
- `docs/labs/m1-bvg-k-stage-t-training/**` (configs, plans, release, results)
- `docs/loop/tasks/m1-bvg-k-stage-t-training/**`
- the item's notebook row in `docs/research/notebook/m1-discrete-attention.md`

## Done when

- The four instrument pieces are built, with tests that fail without them, and pass a fresh exact-head review.
- Step 0 passes on the GPU, and the D67 pre-run review passes.
- `T16_fz` and `T16_rel` run on seeds 1 and 2 to the sheet's stop rules.
- The outcome is graded against the sheet's §3 decision table and recorded on fresh-context-reviewed claims, with one notebook row.
- The item is set `done` and the REPORT says `Closes #2836` for any of the sheet's three outcomes; each one closes the item on its recorded two-seed measurement.

## Stop and ask if

Stop and decide, never stop and wait.

- A Step 0 control fails: that is an instrument fault. Fix it, and read no training metric first.
- The build shows that the sheet's design cannot be run as registered. A changed design needs a new FIT; record it as a D68 card.
- The projected cost passes the sheet's $15 cap.
