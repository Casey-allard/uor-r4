# #2130 — Step 1 cannot be read from outside the crate: the mixture row is PRIVATE (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T02:10:31Z
- **Updated:** 2026-10-10T03:23:35Z
- **Closed:** 2026-10-10T02:12:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2130
- **Merged:** 2026-10-10T02:12:14Z
- **Merge commit:** 140157e2560dac9be7f9812cd475af53e19dee39
- **Base / Head:** main / UOR-Foundation:pcopy-read-attempt
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What step 1 asked for
Report **`p_copy` at the value's two digit positions on the existing weights, before any update**, per row, with word rows as the control — the read that decides whether the training piece is necessary at all.

**CPU only, no pod, no training, no knob, no model saved, $0.** No v5 re-run.

## The numbers exist and are NOT reachable
The mixture evaluation holds exactly what is needed, in `crates/uor-r4-training/src/geometric_stack.rs`:

```rust
struct MixtureRow {
    attention: Vec<f64>,   // the full attention over sources 0..=t
    copy: f64,             // p_copy(target | t)
    gate: f64,
    log_mixture: f64,
    generate_share: f64,
    copy_share: f64,
    present: bool,
    ...
}
```

**`MixtureRow` is a private struct with no public accessor.** `attention` is the full attention vector over sources `0..=t` — exactly what is needed to evaluate `p_copy(target|t) = Σ_j a_tj [x_j = target]` at the value's digit positions, and exactly what cannot be obtained from outside the crate.

**So step 1 is not "minutes of CPU" as the plan estimated; it needs a small addition first, and the plan's estimate was wrong.** Recorded as the plan's error rather than worked around with a proxy.

## The smallest addition, named exactly
A public accessor (or crate-internal test) returning the mixture row for `(ids, target)` — at minimum **`attention`, `copy`, `gate`, `present`, `generate_share`, `copy_share`**. Then step 1 runs unchanged: per row of the 13 numeric rows, take the label's `query` position and report `copy` for each of the value's two digit ids, plus the distractor's positions and the frame positions the decode identified (`223` the bare space, `1498`, `2369`, `1156`, `2728`, `1044`, `754`, `772`, `1790`), with word rows as the control. **A bounded addition: no new algorithm, no training, no weights touched — it exposes a value the code already computes.**

## What can be said now is a BOUND, not the reading
The trace gives the pointer's argmax source and its attention per step, and the decode established that **on 10 of 13 rows the argmax never lands on either stored digit and on no row does it land on the second**. Since `p_copy` sums attention over positions holding the target, **`p_copy(digit) ≤ attention at the argmax`** on those steps — an upper bound that is itself going elsewhere.

**That is not enough to decide the training piece.** The three pre-registered outcomes need `copy` itself: **high at the digits** (nothing to train), **low at the digits against a higher baseline elsewhere** (the frame reading survives), or **low everywhere** (the gate is broadly weak and the labels should say something different).

## Ledger — unchanged, and this read adds nothing to it
Token count **REFUTED** · digit order **REFUTED** · value addressability **REFUTED** · single-source shape **REFUTED in its simple form and replaced** · "history-insensitive" **CORRECTED** · **the pointer attends the frame and never the varying slot, MEASURED on 13 rows**. This piece adds one **operational** fact: the `p_copy` read needs an accessor before it can be run.

## Delivery
Record [`docs/labs/pcopy-read-attempt-2026-10-09/README.md`](docs/labs/pcopy-read-attempt-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `pcopy-read-attempt-2026-10-09.tar`, 13,824 bytes, md5 `61b84225f2058322d67a2e3f4c0ee8b0`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch; scratch inside the worktree.

**Next: add the public mixture-row accessor, then run step 1 unchanged** — per-row `copy` at the value's two digits, the distractor's positions and the frame positions, with word rows as the control.

**The training plan stays on the shelf, not deleted:** it is shelved because the read **cannot be taken yet**, not because the read said "nothing to train", and the next person needs both facts.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

