# ORDER: one-model-end-to-end, claude-265 (train and serve as served under the unsnapped D11 contract, then re-read I-3)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `74e6cf369`. This ORDER is hand-written under the umbrella reading (cancel probe 2).

## Goal

claude-263 and claude-264 found that the integer first token misses the float one on 100–137 of 310 DEV prompts (bar 8), after two repairs.

**The served bridge snaps.** `with_served_surrogate_training` calls `with_d11_training`, which snaps both Hamilton gates and the recurrence transition to the 120 icosians. So a run trained as served had to be exported snapped (an icosian `transport_snap` in the header), as claude-263 did.

**The project's own R6 result says the snap is the gap:**
- **claude-180:** the gate snap carries most of the D11 forward's gap from float.
- **claude-181:** even with the same snaps, an exact f64 reference and the integer session split on 33 of 2,048 positions at snap near-ties; unsnapped, on none.
- **claude-182:** the unsnapped contract (`lut-stack/5`, gates and transition served as exact unit quaternions) closes the export gap. PLAN.md records R6 as done on that contract.
- **claude-259:** trained with only the export's table rounding, scored plain and exported raw, its first tokens differed from the float on 0–1 of 310.

A snap is a discrete choice: two sides a hair apart before it can land on icosians tens of degrees apart, at any of thousands of snaps per token. No amount of matching 2^-16 roundings removes that.

**This unit:**
- makes the served bridge use the unsnapped D11 contract:
  - the gates are continuous (`Hamilton_Active`) and the transition is unit-normalized;
  - the Q12 query bridge, the Q16 residual, the i16 activations and the projection rounding stay;
- makes the served checkpoint's D11 panel and loss follow that contract;
- trains claude-262's 256-step served recipe at seeds 1 and 2 under it, and scores each, seed 1 twice on the same host;
- exports raw, serves on the laptop, and reads I-3.

**Plan step (the claim check):** order step 4, one model trained, exported, served and scored end to end. I-3 is the door to the full-length served run.

**What it changes in the model:** what a model trained as served trains on and serves.
- Its Hamilton gates and recurrence transition are exact unit quaternions in the training forward and in the artifact (`raw_gates`, `raw_transition`), in place of icosian snaps.
- The Hamming read's backward no longer runs through the trunk's snap (`flock_read_hamming::as_trained`'s noted gap).

## Cancel probe

1. **Already done?** No.
   - On main, `with_served_surrogate_training` (`hamilton_sheet.rs:765`) calls `with_d11_training`, which calls `with_d11_mode` (all snaps).
   - `fbs-export` defaults to snapped and does not read how the run trained.
2. **The lock.**
   - **The refusal**, 13:17Z, verbatim: "loop claim: refused: item one-model-end-to-end is already claimed by PR #3599 (docs/loop/tasks/one-model-end-to-end/deepseek-12/ORDER.md)". #3583's ORDER could not be read (HTTP 500) and was treated as an open order.
   - **#3599** (deepseek-12, draft at `8474055c1`, opened 13:17Z) is in scope for `docs/labs/one-model-end-to-end/**`, its task folder, `float-training.md` and the item file.
   - **#3583** (deepseek-9, draft at `90bfc731a`, unchanged since 09:52Z) has a real diff from merge base `e2470ec4d` in `crates/` of `train-in-space.rs` and `in_space_long.rs` only. Its `hamilton_sheet.rs` is blob `66f85c842`, main's own version at claude-262 (`fac0131bc`).
   - **The overlap:** no code file. Only the notebook and the item file are shared, each taking added text only.
   - **The basis:** the umbrella reading, claude-263's decision review (ALLOW, 09:48Z, logged on #2028).
3. **Inputs.** The DEV data pins, and claude-262's recipe (its sheet's R block), are in the store and the repo, as claude-264 used them.

Classed instrument: an I-3 miss is a parity fault (§4.2). The change is a model change.

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`: one assertion of `the_training_forward_selects_and_restores_the_served_bridges` only, the transition flag the served contract turns off.
  - #3583 also edits this file. Its last hunk inserts at base line 14017, about 680 lines before this test, and its diff names neither the test nor the snap flag.
- `docs/harness/prep/one-model-end-to-end-i3-unsnapped*`
- `docs/loop/tasks/one-model-end-to-end/claude-265/**`
- `docs/research/notebook/float-training.md` (added rows only)
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Under the served bridge, the gate arm is `Hamilton_Active` and the transition is not snapped. The Q12 query bridge, the Q16 residual and the i16 activations stay on, and every flag is restored after `Ok`, `Err` and a panic. A host test fails with the change set aside and passes with it.
2. The served DEV panel and loss follow the contract: under the served bridge, `with_d11_mode` is `SERVED_SNAPS`.
   - The served checkpoint's `evaluate_dev_panel_d11` and `panel_loss_d11` call `with_d11_mode`, so they need no edit.
   - The I-3 serve step exports with `--raw-gates --raw-transition`. It refuses any artifact whose header shape lacks `raw_gates: true` or `raw_transition: true`, or that carries a `transport_snap`.
   - The schema does not say this: a stack with the turn memory is `uor-r4.lut-stack/7` (`STACK_FBS_MEMORY_SCHEMA`) whether its gates are raw or snapped. claude-263's snapped exports were `/7` with an icosian `transport_snap`.
   - A guard inside `fbs-export` is out of the item's scope (`crates/uor-r4-integer/src/bin/` is not listed), so it stays open in the REPORT.
3. The prep sheet is FIT, and its pre-run review is PASS.
4. On one 5090 at the PR head:
   - the CUDA library suite passes;
   - seeds 1 and 2 train 256 steps as served and are scored, seed 1 twice;
   - the outputs are sealed and read back, and the pod is down.
5. I-3 is read at both seeds by the sheet's bar, against raw exports served on the laptop. The run-to-run count is read from seed 1's two scores.
6. A REPORT with negatives first and tokens per second first, one notebook row, and the item's line updated.

## Stop and ask if

Each is decided in the unit, never waited on:
- **The change alters a non-served forward** (plain, Export, D11): fix it before any pod.
- **A served training run diverges or loses its checkpoint:** classify it as instrument or mechanism before anything else.
- **I-3 misses:** the next unit reads the remaining continuous sites, with the run-to-run count in hand.
- **#3583's or #3599's files grow into scope:** re-check them before every push.
