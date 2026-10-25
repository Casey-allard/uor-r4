# REPORT: m1-w2-code-utilization, claude-89

Lab: claude. Order: [ORDER.md](ORDER.md).
References #3179 (the item stays `ready`; the diagnostic's run is the next unit).

## What did not work, first

- **The first draft was NOT_FIT.** The non-author FIT reviewer found one blocker and seven should-fix items.
  - **The blocker.** Instrument check 8 bounded arm R's changed parameters by accepted updates, and any failure stopped every band.
    - The reviewer read `training.rs`'s `committed_moves` range (accepted to K × accepted) as applying here. That range exists only when `coordinate_block > 1`. The 15 Oct Q-run predates BVG-K, so it moves one parameter per accepted update. The reviewer agreed in round 2 that this was its misreading.
    - The fix it asked for was still right, and is in. The bound now reads the cursor: `accepted_updates` when there is no `block` field, `committed_moves` when there is. A failure makes only R NOT_RUN.
  - **A1 overclaimed.** U always predicts token 1, so a fixed list of the most frequent cells (A_U) may well admit more target cells than the writer's 1,920. A_T could then beat A_W only by ranking cells by frequency. A1 is now split:
    - A1a: A_T beats both A_W and A_U.
    - A1b: A_T beats A_W but not A_U, and A_U is the reference to beat.
  - **QA was not load-matched.** A mass-ranked admission set can admit more targets by admitting more candidate tokens. Candidate load is now reported for every admission set.
  - **QB was decided before any data.** B1 needs T(N_1) ≤ 1,903, a 44% drop from halving the data. The threshold is now stated, and B2 is named the expected outcome: its job is to decline G1 on a measured curve.
  - **QB had no exit when underpowered.** It is now exempt from precedence 1, and a noise-level gain gives d_T = 0, so B2.
  - **C2 and B2 overclaimed.** C2 read "neither beats" as "the read must change". Both rows now say failure to detect, and leave the read question to a decision card.
  - **R was cited as evidence with no threshold.** It is descriptive only now.
  - **First-row prefixes could confound data size with corpus order.** The data-scaling arm now uses stride-2^j subsamples, with the prefixes kept as descriptive figures. Exact byte sizes for the stream and the sidecar were added to the pin checks.
- **Round 2 returned FIT with three optional nits**, all applied after the verdict:
  - A2 is reworded as a failure to detect.
  - C1 is softened to "one input favouring".
  - An underpowered A_T-against-A_U pair gives "A1, unsplit" instead of making all of QA UNDERPOWERED.
- **The exact-head review at `776b2d8c1` found that the post-FIT "unsplit" reading was itself ambiguous** (APPROVE, two should-fix items).
  - That reading had no row and no "Next". It is now row A1u, and A1a and A1b say "powered", so the four A1 rows stay disjoint.
  - Read literally, precedence 1 could void QA through the A_T-against-A_U pair even when A2 or A3 fired. That pair is now decisive only when A_T beats A_W.
  - Its two nits are also in: the resubstitution top-1 is defined, which the leave-one-out bound needed, and the sidecar's path is given as `w2-out-b/`.
  - None of this changes a threshold or any outcome's routing.
- **`loop next` does not name this item first today.** It offers one item per call, ranked by milestone, then unblock count, then id. This item and `m1-address-as-read` are both M1 with 0 unblocks, so `m1-address-as-read` wins on id. The item is in the claimable set (ready, scoped, its only dependency done, read from `origin/main`). It is offered once the items ranked above it are taken, or sooner if a later decision makes another item depend on it.

## What changed

Size: docs 3 files (+309/−0: the sheet 186, the ORDER 43, this REPORT 80); code 0; tests 0. The one advisory WARN on the first push (rule-sources, D76, on the heading `## Stop rule`) is cleared by naming that block `## Stop conditions`; the prep shape rule needs only "stop" in the heading.

- `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md` (new, 185 lines). It contains:
  - the question, and QA, QB and QC;
  - inputs and notation, with W2's paired floor in squares;
  - the decisive and descriptive arms, each with an exact integer definition;
  - the decision table, with what each outcome unlocks;
  - the instrument checks;
  - the GPU arms named but not run (G1 full-TRAIN capture, G2 init snapshot);
  - the harness's required blocks and both TEST FITNESS lines.
- This task folder.

## Measurements

None. This unit runs nothing. The sheet's arithmetic was checked by the FIT reviewer:
- F² = 4 · 4,572 = 18,288;
- T_proj ≥ 6,436 means d_T ≥ 1,511, that is T(N_1) ≤ 1,903;
- 1,049,856 = 16 × 65,616;
- the byte sizes 4,199,424 B and 2,099,712 B;
- Σ k_r = 114,296 = 14,287 × 8.

## Checks run at the head

Recorded in the PR's review record.
- `uor-harness check --only prep`: exit 0, and no finding names the new sheet.
- `loop report --check` and `loop pr-check --local`.

## Skills used

- holding-the-goal: the M1 bar is a comparison with T2 = G = 6,300, and the model's prediction is a function of its code. Those two facts are why the questions are QA, QB and QC.
- solution-integrity: every band is labelled measurement or proof, and failures to detect are not read as negatives (CHARTER §4.1, §4.6).

## Still open

- **The run.** Build the Rust host tool (W2's `w2_capacity.rs` tables, tie rules, floors and McNemar are reusable), run it once on the stored capture, seal the record, write one notebook row, then register or decline G1 per QB. The item offers this to the next lab that `loop next` reaches it for.
- **Stage T's gate** ("only warranted if W2 blames the learning law (H_law)") is untouched. Any change is a decision review's call, with this run's bands as inputs.

## Plain summary

W2 showed that a table keyed by the model's 16-bit "read code" predicts the next word 3,414 times out of 14,287 on the development panel. The model itself manages 149, and a simple previous-word table 6,300. The milestone needs the model to beat 6,300.

This sheet fixes, in advance, three questions about the stored W2 data, along with the exact counts and thresholds that answer them:
- Could the code choose candidate word groups better than the model's address key does?
- Would more training data lift the code table toward 6,300?
- Does the code know anything the previous word does not?

A reviewer that had not seen the authoring first rejected the sheet, then passed it once the fixes were in. Nothing ran on a GPU; the questions are answered later, on a laptop, from data already saved.
