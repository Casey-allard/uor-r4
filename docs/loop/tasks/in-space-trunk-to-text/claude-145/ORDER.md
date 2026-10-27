# ORDER: in-space-trunk-to-text, claude-145 (a content-keyed read inside the trunk)

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `a9806b378`.

## Goal

**Let a reply opening find the prompt's words by what they are, then let the trunk's later layers use them.**

claude-144 closed one Flock read configuration (cell D at both seeds). It keyed on the rotated hidden point and added the next token's point after the last layer. Done-when (1) holds on every arm. Done-when (2), `R16 ≥ 0.05`, is the bar no arm meets (best `0.0171`).

**The read, `crates/uor-r4-float-train/src/flock_read_content.rs` (new), switched by `--content-read K`, default 0 (off).**

1. **Where:** in `StackModel::hidden`, after layer 3 of 6, so layers 4–6 transform what it adds.
2. **Key:** each earlier position's own token point, `e(x_s)`, rotated lane by lane by a learned unit quaternion. Each 4-lane block is put on the unit sphere S³, so the key is a direction, not a magnitude.
3. **Query:** the residual point `x_t` at layer 3, rotated and put on S³ the same way.
4. **Choice:** the `K` nearest earlier positions `s < t` in mean L1, hard and detached. This reuses claude-144's choice.
5. **Payload:** the earlier position's residual point `x_s` at layer 3.
6. **Consensus and credit:** the plain mean, with claude-144's straight-through factor.
7. **Add:** `x_t + w ⊙ c_t`, with `w` starting at 0.
8. **Optimizer:** its own AdamW, as in claude-144.

The read is saved beside each checkpoint, and every reader uses it.

## Cancel probe (run first)

Cancel if a content-keyed float read exists, or another open order changes the float-train runner.

Premise count (laptop, a count and not a model):
- Build TRAIN's commonest reply opening for each key: the first `k` prompt tokens the window shows, or the last `k`.
- Score it on DEV's 1,226 openings.

If no key beats the twins' `271` by much, the openings route is weak, and reach (2) becomes the read's whole case. The result is recorded either way. It is not a cancel.

Result (2026-10-27T09:05Z, base `a9806b378`; `train-in-space --opening-bound 1`, 0.5 s on the laptop; 212,968 TRAIN openings, DEV's 1,226):

- **No key: `161`.** This equals DEV's commonest-opening count, which checks the instrument.
- **Last 1–4 tokens: `161`.** The tokens before an opening are always the same marker, so a short reach sees nothing.
- **First `k` tokens:**

  | `k` | 1–3 | 4 | 5 | 6 | 7 | 8 |
  |---|---|---|---|---|---|---|
  | DEV openings named | `146` | `227` | `373` | `460` | `556` | `607` |

- **The words carry it.** The commonest keys are "User: What is" (opening "It"), "Please remember:" and "Remember" ("G…"), and "What was" ("That").

**Not cancelled, and the premise holds.** A read that finds the prompt's words could name up to about half the openings, against the twins' `200–271`.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read*.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `docs/harness/prep/in-space-trunk-to-text-content-*`
- `docs/research/notebook/float-training.md` (rows)
- `docs/loop/tasks/in-space-trunk-to-text/**` (this task folder)

## Done when

1. The premise count is recorded, with its key table's top rows decoded.
2. The read builds, and its CUDA tests are green on the pod. They cover:
   - the choice is nearest, causal and never itself;
   - `w = 0` leaves `hidden` unchanged;
   - keys come from token points, not from state;
   - the credit's sign, with a negative gate.
3. A probe ladder shows the read trains: read against no read.
4. A two-seed run, read against no read, under its own prep sheet. It gets one fresh FIT and pre-run review (light gates, per the owner's 27 Oct direction), and a third seed if the seeds split.
5. The REPORT puts negatives first and leads with tokens/s. It shows the replies and says whether the item's done-when is met.

## Stop and decide (never stop and wait)

- A read that stalls training: classify instrument or mechanism first, and record it.
- A missing number gives no cell: rerun that step.
