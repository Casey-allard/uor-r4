# REPORT: one-model-end-to-end, claude-391 (a native update rule for the Q31 read's gates)

Lab: claude. Item: one-model-end-to-end (#3407). ORDER: [ORDER.md](ORDER.md). PR #4009.
Commits:
- `15230f0b2`: the ORDER;
- `fdc83c08c`: the rule and its tests;
- `154e9236f`: the margin test;
- `eae2f77a8`: the first REPORT;
- `637723b27`: the review fixes.

## What did not work, first

1. **The rule as the ORDER wrote it was too slow at the served scale.** At the residual claude-390 measured (about 2^18), each accepted step lowered the loss by about 0.1% (131,072 → 130,904 at step shift 0), so the fixture needed about 780 steps. The ORDER expected the credit's natural size to clear the quantum, and that was wrong:
   - the credit scales with the block, about 2^17 per gate word;
   - the gate's effect on an address scales with the block too, by `|x| / 2^31`;
   - together the step shrinks with `|x|^2`, about 2^13 short at 2^18.

   The fix stays inside the update: before the conjugate product, the rows are lifted by one exact power of two per batch, so the largest word is at most 2^30 (`q31_read_credit_shift`). The fixture then settles in 4 steps at shift 4, and in 4 steps at scales 2^10, 2^18 and 2^28 alike. This is the second of claude-390 Still open 1's two options (updates sized above the quantum). The operator is not changed.
2. **Step shift 0 can overshoot.** Two windows whose rows sit in swapped order raise the batch loss from 131,072 to 393,216 at shift 0. The acceptance rule rejects that candidate. No step-size schedule is built: a rejected step is returned unchanged, and the caller chooses the next shift.
3. **My first overshoot fixture did not overshoot.** 64 identical windows at shift 0 saturate the gate words, and the loss still drops. A sweep found the two-window case above.
4. **The fresh review at `eae2f77a8` found two bugs.** Both are fixed, and each fix's test failed first at that head.
   - **The credit could wrap and flip its sign.** It went through `q31_hamilton_rotate`, whose four products sum in wrapping `u64`. A residual of two units on four aligned lanes against a block lifted to `2^30` makes `4 · 2^61 = 2^63`. The test's credit came out −2,147,483,648 where +2^33 was due. The conjugate product is now exact in `i128`, and the step divides by `2^(shift + 1)`.
   - **At margin 0, a zero loss could leave the pick wrong.** When the target and a later rival are at equal distance, the hinge is 0, but serving sends the tie to the later row. The test's loss was 0 where 1 was due. The margin is now at least 1 when the rival is later than the target.
5. **Only the gates train.** The weights need a downstream error, and the stack has no native loss yet. The gain stays fixed (claude-389 Still open 2 left that choice to this unit).

## What changed

`crates/uor-r4-native-vector/src/read_train.rs` (new) adds `Q31FirstPickRead::train_pick_step` and `q31_read_credit_shift`. `read.rs` widens `ADDRESS_LANES` and `read_distance` to `pub(crate)`, and `lib.rs` declares and re-exports the module.

1. **Loss.** Per row with a target: `max(0, d(q, k*) − d(q, k_r) + m)`, summed exactly in `u128`. `m` is at least 1 when the rival is later than the target. `d` is the read's own L1 address distance. `k_r` is the nearest other earlier key, and a tie goes to the latest row, as in the read. The step reports each row's rival (`r + 1`).
2. **Credit.** The L1 subgradient per address word is a sign:
   - the query gets `sign(q − k*) − sign(q − k_r)`;
   - the target key gets `−sign(q − k*)`;
   - the rival key gets `+sign(q − k_r)`.

   For each block, that residual, in sign units, is multiplied exactly in `i128` by the conjugate of the block lifted by the credit shift. This is the rule of `q31_hamilton_train_step` at `2^30` per residual unit, without its Q31 rounding and `u64` wrap. Credits sum exactly in `i128`. Each gate becomes `sat(gate − clamp(credit >> (step_shift + 1), ±2^32))`.
3. **Accept.** The candidate is built with `Q31FirstPickRead::new`, so every word is re-admitted. It is kept only if the batch loss strictly drops; otherwise the step returns the read unchanged. The picks before and after come from `apply`, the serving operator.
4. **Refusals.** `apply`'s own refusals come first. Then: a targets length other than `batch * time`; a target that is not an earlier row of its window (this includes row 0 and the row itself); a step shift above 31.

## Measurements

Laptop, finite fixtures, width 16, identity gates, gain 4. No GPU and no training run.

| Fixture | Number |
|---|---|
| Row 2 picks row 0, target row 1, scale 2^18, shift 4 | loss 131,072 → 88,064 → 45,056 → 2,048 → 0 in 4 accepted steps; picks `[0, 1, 1]` → `[0, 1, 2]` |
| The same at shift 8 | 49 accepted steps (pinned by a test) |
| The same without the credit shift, shift 0 | −168 per step (about 780 steps): a one-off probe of the first, uncommitted version, not pinned |
| Block `[S; 4]` lifted to 2^30, residual ±2 on every lane | credit word 0 is exactly ±2^33 per gate; ±2^34 at a word of 2^31 |
| Target row 0 and rival row 1 at equal distance, margin 0 | loss 1, not 0; training makes row 0 the pick |
| Scales 2^10, 2^18, 2^28 at shift 4 | 4 steps each |
| Two swapped windows, shift 0 | candidate loss 131,072 → 393,216, rejected, read unchanged |
| Margin `S` on a right pick at distance gap `S/2` | loss `S/2` before; accepted, lower after; picks unchanged |

**Replay.** The trained read's `apply` picks equal the step's recorded picks. The read survives `to_bytes`/`from_bytes` unchanged, and the restored read's incremental `step` gives the same picks, `u32` for `u32`.

## Checks run at the head

At `637723b27`, on the laptop. The only change after it is this REPORT and the notebook row.

- `cargo test -p uor-r4-native-vector`: 38 passed, 0 failed (10 in `read_train`).
- `cargo clippy -p uor-r4-native-vector --all-targets`: nothing in the changed files. One existing warning remains at `read_dump.rs:200`.
- `rustfmt --edition 2021` on `read_train.rs` only (a new file).
- **Negative gates.** For each, I confirmed the change was applied with `git diff --stat`, ran `read_train`'s tests, then restored and re-ran green:
  1. accept every candidate: 4 tests fail (overshoot, rows without a target, scale, replay);
  2. drop the conjugate: 3 fail (overshoot, scale, replay);
  3. a rival tie goes to the earliest row: the tie test fails;
  4. credit shift 0: 3 fail (scale, overshoot, replay);
  5. a target one row later is admitted: the refusals test fails;
  6. drop the margin term: the margin test fails;
  7. the credit through the wrapping rotate, at `eae2f77a8` before the fix: the aligned-credit test fails (−2,147,483,648 against 2^33);
  8. no minimum margin for a later rival, at `eae2f77a8` before the fix: the tie test fails (loss 0 against 1).
- **The notebook row** is in `float-training.md`, beside claude-389's and claude-390's rows for this read. The ORDER's file list names `cpu-training.md`; that line of the ORDER was wrong.
- **The fresh review** at `eae2f77a8` asked for changes: the two bugs above, pinning the REPORT's numbers, and the notebook file. All are addressed here.
- `uor-harness loop pr-check --local`: see the PR record.

## Skills used

`native-space-cycle`: the ORDER's five items. The forward, the gradient and the update stay in Q31 and `i64`/`i128` words, and the update calls the read's own address and distance code.

## Still open

1. **The CUDA twin of the update, and its bit parity** against this host rule on wick. Training qualifies on GPU (§3.5), so this comes before any run.
2. **A pre-registered training run** of the gates on served states with target picks, judged on held-out picks. It needs a sheet with a non-author FIT (§4.5). The gain for that run is set at initialization (claude-389 measured `g + 4` for ×16).
3. **The weights' update.** It needs a native downstream loss from the stack. None exists yet.
4. **A step-size schedule.** The caller retries a rejected step with a larger shift. A rule for that, such as halving on reject, belongs to the run.
5. **The credit shift is one per batch.** One large row lowers the shift for every row in the batch. A per-row shift is the alternative if a run shows skewed batches.
6. **Cost.** Each step runs the forward three times: `apply` before, candidate addresses, `apply` after. The device twin can share the before pass.
7. **From claude-390:** Still open 4 to 10 stand, including restore with a read, `/13` beside `/14`, and the per-step table scan.

## Plain summary

The read that copies the best-matching earlier fact can now learn which fact to copy, in the same integer words it is served in. On a small test case, a wrong pick became right in four steps. The trained read then picks the same rows when it is served, saved and reloaded. A one-off probe of the first version showed it learning several thousand times too slowly at the real model's scale. One exact power-of-two scaling inside the update fixed that without touching the served operator. The review then found that a large step could flip its own sign, and that an exact tie could look solved while serving still picked the wrong row. Both are fixed and tested. Next comes the GPU twin of this update, then a real training run.
