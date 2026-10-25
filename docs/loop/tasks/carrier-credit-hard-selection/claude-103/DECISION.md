# Decision card (D68): a new sheet for carrier credit on the live learner, and this item's scope

Lab: claude, claude-103. Item: [`carrier-credit-hard-selection`](../../../../roadmap/items/carrier-credit-hard-selection.md) (#2819). Status: **approved**. The fresh decision review returned CHANGES in rounds 1–3 and APPROVE in round 4. Round 4's three wording advisories are applied.

## Why this test, and why now

- **The chain.**
  - M1 closes on the saved-model path (D79). Its training doses are the A8 full dose rerun (#2831) and BVG-K Stage T (#2836), and both train with one of the discrete learner's fixed coordinate schedules, canonical or interleaved (`training.rs`:585–610).
  - This item is `milestone: research`, and no item depends on it. `loop next` lists it as the only item claude can claim now (unblocks 0).
  - The question it answers bears on those doses directly: does choosing *which* coordinate the exact move tries, by the objective's own carrier credit, beat the fixed sweep and an uninformed in-batch choice at a matched dose?
- **The consumer.** A pass gives #2831 and #2836 a measured schedule for their next registered dose. That is a new sheet for each, not an edit to a running one. A control-only win tells them the gain is in-batch selection, not credit. A fail tells them the canonical sweep is not their bottleneck.
- **The mechanism.** `docs/mechanisms/carrier-credit.md` is PARKED. Its reopen conditions 1 ("hard selection by address or integer counterfactual moves") and 5 ("an integer counterfactual form on the discrete path") name exactly this test.

## The facts this rests on

1. **The archived sheet cannot run lawfully.** `archive/labs/unified-geometric-integration-2026-10-11/PREREGISTRATION.md`:16–27 starts from saved437 and runs 128 updates at batch 8.
   - Its objective is the "equal-episode logmeanexp of the existing strongest-wrong pooled negative-log-mass margin0 softplus", which CHARTER §3.2 bans in losses.
   - ON and OFF differ only in "the new v3 field-to-carrier/prototype adjoints" (`:23`), which are float gradients.
   - Its 437/512 start was itself panel fitting: 0 of 128 on fresh rows (`docs/mechanisms/carrier-credit.md`:3).
2. **Its code is compiled out.** `crates/uor-r4-training/src/lib.rs` marks the carrier modules `#[cfg(any())]`, and the runner example requires `historical-reference` (`Cargo.toml`:27, 53–55). Float training is legal only in the separate float module (§3.7, D37, #2823), which is not extracted yet.
3. **The softmax "sampled choice" was never run.** It is `Mode::Sampled` in `whole_path_learning.rs` near 2324, and its file calls itself an "offline CPU reference … not a qualified CUDA producer". The executed arms held the other carriers "at the actual hard codes" (`README.md`:131).
4. **saved437 is only in the iCloud cold store** (`results/codex/codex-m2-fresh-437-20261010.tar`).
5. **The only live learner is `crates/uor-r4-discrete`** (ACTIVE).
   - It tries all 16 legal states of one coordinate against the integer rank-margin hinge, and commits on strict improvement.
   - The canonical sweep visits V·B = 4,096 × 4 vector blocks in order.
   - In the 14 Oct pilot, 1,179 of 1,640 vector proposals had some candidate cost different from identity, and 236 accepted (`archive/labs/a2-pilot-2026-10-14/README.md`:29–31). The DEV count reference was 1,226 top-1 and 4,060 top-8.
   - CUDA `discrete_evaluate` already writes per-row hinges, `top1_correct` and the top-8 admitted ids (`device.rs`:1565–1650).
6. **A new schedule must live inside the crate.** `Device::apply` is `pub(crate)` (`device.rs`:2007), and a move commits only through `PreparedDriver::transition` with a controller admission (`training.rs`:2579–2690). D67 staging has a precedent in `bin/a8-qualify-stage.rs`:513–545.

## Decision (recommended)

**A. A new sheet, honestly named.**
- The done-when is met by a new pre-registered sheet with its own FIT (§4.5: archived float sheets "return only as new sheets"). It tests the discrete counterpart of carrier credit (reopen condition 5).
- It reruns the archived sheet's ON/OFF × two-seed design, not its objective, start state or carriers.
- The archived design was 2 schedules × 2 seeds, and three of its four arms ran. The new design is 4 schedules × 2 seeds, eight arms, all registered and all run.
- The item's done-when text is replaced by the exact line in D.

**B. The hard-selection credit step.** It replaces the sampled choice.
- **The credit signal.** Each batch, CUDA evaluates it with `discrete_evaluate`. A row with a positive integer hinge (token plus address, the geometry's own rank-margin objective over E8-root cells) credits that hinge to its target and to its strongest admitted rival (`top8[0]` when top-1 is wrong). These are the two carriers whose cells the margin is measured between.
- **The selection.** The tried coordinate is a vector block of the token with the largest credit, ties to the lowest id. The block is `completed_batches mod B`.
- **Bound to the GPU (§3.5).** The credit scatter-add and the argmax run as one nvcc-built kernel in `discrete.cu`, with no NVRTC (owner, 13 Oct). The host receives only the 2-word result `(token, credit)`. A test fails if the selection path downloads per-row results.
- Nothing is sampled, and there is no probability, softmax or matrix product.

**C. Arms.** Every arm uses the same objective, 16-state counterfactual kernel, commit rule, batch order, corpus, start state and dose. Each arm runs at seeds 1001 and 2001.
- **ON:** the credit selection in B.
- **Two model-blind controls.** Both use the same block rule. They draw by a named integer hash of `(seed, completed_batches)`, fixed in the sheet. They never read model state, so each is a schedule choice, not sampled credit.
  - **CONTROL-U** draws uniformly over the batch's distinct target tokens. It carries no frequency information.
  - **CONTROL-F** draws uniformly over the batch's rows, then takes that row's target. It carries frequency but not credit.
  - ON's credit sums over rows, so it leans toward frequent tokens. CONTROL-F separates credit from frequency, and CONTROL-U separates both from "any in-batch schedule".
- **OFF:** the canonical sweep, which visits a token's B blocks back to back. ON and both controls share the `mod B` block rule, so their comparisons carry no block-order confound. An ON–OFF gap alone is never read as credit.
- **Leakage.** Credit and draws come from the training batch only, never from DEV or fresh rows.
- **Start state:** a fresh model (seeded-device init) on the registered discrete spec and corpus, at the full 16,404-batch dose that #2831 and #2836 also run. The schedule finding then becomes a candidate arm in their next sheets, tested at their own configuration.

**D. Scope fix, through one `infra-loop-harness` unit.**
- The scope is `crates/uor-r4-discrete/**`. It is the convention of 18 items, and the pin updates reach tests across the crate. A narrower list would force another umbrella unit the first time the guard refuses a pinned test.
- `notebook: docs/research/notebook/m1-discrete-attention.md` is bound.
- The done-when line becomes, exactly:
  > A hard-selection credit step replaces the sampled choice with no softmax, on CUDA and integer-only, and the archived three-arm sheet's ON/OFF × two-seed design is rerun as a new pre-registered sheet with a fresh FIT on the live discrete learner (CHARTER §4.5; carrier-credit reopen condition 5), with model-blind in-batch controls, every registered arm run, and the result recorded on fresh rows.

  It still requires the step, every arm and the recorded result. It changes only what can lawfully be rerun.
- This is this item's own scope fix (D68), following the `023910c63` precedent. It is not a new infrastructure item under D77.

**E. Fresh rows, and the bar.**
- **The fresh slice.**
  - Fresh rows are a held-out slice of the registered TRAIN corpus. The slice is cut out of the train binding before any arm runs, as a separate `Split::Fresh` tokens/mask pair, sealed by its hashes, and kept in the HF store. It is data preparation on the host.
  - The trainer never reads it. The arms are scored on it once, at the end.
  - It is not the M1 fresh panel (binding `4bd108ad…`, the fresh V2 seal `df5d074a…`), which stays sealed for M1 bar (e), and it is not frozen `ft/dev`.
  - It is cut at document boundaries. Windows that duplicate a TRAIN window exactly are removed. It holds at least 200 scored targets (§4.3), with the exact count fixed in the sheet.
  - Refined in the prep sheet (§4), before any FIT. Batches walk TRAIN's eligible targets in order, so a 16,404 × 64 dose reads only indices 0 to 1,049,855 of 2,425,925. The slice is taken from documents past that point, the TRAIN binding stays byte-identical to the one #2831 and #2836 use, and a driver check refuses any cursor that reaches the slice. A finding still enters their next sheets only as a candidate arm.
  - The build adds the `Split::Fresh` binding and its evaluation entry point. `corpus.rs`:56 defines the split, but `training.rs` loads and evaluates only Train and Dev (:272–273, :453–454, :3036, :3068).
- **The bar** is fixed in the prep sheet before any run, and reviewed for FIT.
  - The two pre-named comparisons are read on fresh rows (§4.3), with a margin no narrower than the measured seed noise.
  - The count-table reference (§4.4) is computed on the fresh slice as well as DEV, and is reported beside every arm.
  - Wall time and tokens/s are reported. ON and both controls run the same CUDA evaluation and selection kernel every batch: the controls' draw is a mode of that kernel and ignores its scores. Compute is therefore matched among the three, and only OFF skips the kernel.

## What each outcome unlocks

An instrument failure is classified first (§4.2). It gets a repair and a rerun, and no verdict.

Otherwise there are two pre-named comparisons. Fresh rows decide both. DEV is reported, and a comparison that wins on DEV but not on fresh rows is labelled "panel fit" in the REPORT, without changing its verdict. Each comparison is a **win** (at both seeds, by more than the seed noise), a **tie or loss**, or **split** (the seeds disagree). The verdict is the pair of results, and each half sets its own decision, so every combination maps to exactly one pair of decisions.

**Credit: ON against both controls.** This is decided per seed. A seed counts for ON only if ON beats both CONTROL-U and CONTROL-F at that seed, by more than the seed noise. **Win** means both seeds count, **tie or loss** means neither does, and **split** means one does. Selection is read per seed in the same way.

| Result | Meaning | Decision |
|---|---|---|
| win | credit adds over frequency and over an uninformed in-batch choice | reopen conditions 1 and 5 met; mechanism PARKED → READY; ON becomes a candidate arm for #2831 and #2836 |
| tie or loss | credit adds nothing over a model-blind choice | configuration negative for credit (D22); mechanism stays PARKED |
| split | inconclusive at this power | no status change; the next configuration adds seeds |

**Selection: CONTROL-F against OFF.** Pre-named: CONTROL-F is the model-blind schedule a dose would adopt. CONTROL-U against OFF is reported only.

| Result | Meaning | Decision |
|---|---|---|
| win | in-batch selection beats the sweep at this dose | CONTROL-F's rule becomes a candidate arm for #2831 and #2836 |
| tie or loss | model-blind in-batch selection does not beat the sweep at this dose | #2831 and #2836 keep their schedule. ON against OFF is reported beside any ON candidacy, so a credit win that still loses to the sweep is visible |
| split | inconclusive at this power | #2831 and #2836 unchanged; the next configuration adds seeds |

A pass still leaves reopen condition 2 (one typed binding and frame) and the "all 4,096 choices scored with physical copy aliases" part of condition 4 open, and they are named in the REPORT. Condition 3 asks for a panel that is not component-only, two seeds, and at least 128 fresh rows. It is met by the corpus, both seeds and the fresh slice of at least 200 targets. Condition 6, the M2 hold, is superseded by D67.

## Rejected

- **Replay the archived sheet:** its banned objective and compiled-out float code rule this out.
- **Port it to the float module:** #2823 is not extracted, and the objective would still be banned.
- **Close the item unrun:** a blocker is fixed, not parked.
- **Narrow the crate scope to named files:** rejected for the umbrella churn above.

## Risks and reversal

- **Pin churn.**
  - Editing `training.rs` moves `driver_source_sha256`, which `prime_zeta.rs`' bin tests pin. The full discrete suite must run, and the pins must move in the same PR.
  - A negative gate is required: a test that the canonical schedule's config and lineage bytes stay byte-identical.
  - The new kernel is pinned by its PTX bytes, never by the compiled-source sha, which changes with every rebuild.
  - A run built from a fixed commit is unaffected.
- **Reversal cost:** low. It is one schedule variant behind config, plus the item-file lines.
