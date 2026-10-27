# ORDER: d11-exporter-float-trunk, kimi-46

Lab: kimi. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `1b9b87b39`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

## Cancel probe (run first)

claude-136's stored-output bound shows a perfect exporter cannot change any of the six verdicts, so continuous localisation is not the next informative run. The pivot's remaining test is a dose-plus-coverage change to 2,048 dual-forward steps: both claude-131 losses were still falling at step 1,024, and the next 1,024 deterministic batches are new windows from the same corpus, so this is not a pure optimizer-step isolation. Source inspection finds `run_fbs_arm_with` already accepts a step count, but `run_d11_train_probe`/`run_d11_train_export` hard-code `STEPS = 1,024`, `TRAIN_BATCHES`, and `EVAL_STEPS`, and the CLI has no `--steps`. Repeating claude-131 would add no information. `classification: instrument`. **Not cancelled: add the narrow step-count binding, then run the 2,048-step dual configuration.**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

Prefer changes limited to `spiralcore_fbs_sheet.rs` and its existing CLI. Keep `TrainForward::Dual`, the 1:1 loss, both learning rates, weight decay, clipping, data, seeds, exporter and evaluator unchanged. Do not edit the retired `crates/uor-r4-training/src/stack_export.rs` or change the unrelated Q12 clamp mask.

## Done when

- Failing-first controls pin `--steps` defaulting to 1,024, longer-dose acceptance, sub-1,024 refusal, final evaluation placement, batch count and probe projection; restored behavior passes all affected crate suites.
- A new one-page sheet fixes 2,048 steps at unchanged per-step learning rates, the Stage 4b gap bar, claude-129 artifact floors 1,907/2,294, seed-spread/loss-band readings, instrument checks, outcomes and costs. Fresh non-author FIT and D67 precede dispatch.
- RTX 5090 probe and two-seed run save models, snapshots, artifacts, verified GPU evaluations and predictions with checksums and remote readback. Report F/A top-1, paired b/c/F_I, loss gap, seed spread, artifact floors, D-vs-A residual and full costs.
- Apply the registered outcome: unflagged PASS gets product decode/context tokens/s, zero-multiply audits and integer serving tests before `Closes #2820`; flagged PASS gets D68; mixed/FAIL closes only this configuration and leaves the item open with a decision record. Do not claim a mechanism negative.
- Add REPORT and one notebook row, pass full gates/local policy and fresh exact-head code/evidence reviews, queue, verify on fresh main and clean up.

## Stop and decide if

- Any change besides dose and its evaluation/projection plumbing is needed, or the 1:1 loss, learning rates, bridges, data, seed definitions or bar would change; re-scope with a new ORDER/sheet.
- Probe shows non-finite loss, no learning, identity mismatch or projection over one hour/seed; classify instrument versus mechanism before the full run.
- Any host model/parity fallback, dense matmul, softmax/squashing, non-geometric surrogate, or unverified restored path appears.
- Apply the sheet's exact precedence: A failing while the perfect-exporter counterfactual D passes is exporter-unresolved; A and D both failing is configuration FAIL, including mixed outcomes. Never substitute the retired coarse |net(D,A)| branch, and never read an unflagged PASS without the artifact floors.
- Fleet/disk caps fail, or another lab's files, pod, lease or scope would be touched.
