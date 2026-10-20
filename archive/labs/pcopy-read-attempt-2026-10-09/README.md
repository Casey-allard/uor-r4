# Step 1 cannot be read from outside the crate: the mixture row is PRIVATE, and the smallest addition is named

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, no knob, no model saved,
$0.** No v5 re-run.

## What step 1 asked for

Report **`p_copy` at the value's two digit positions on the existing weights, before any update**, with
`pointer_nll` and `gate_bce`, per row, for the 13 numeric rows plus word rows as contrast — the read
that decides whether the training piece is necessary at all.

## What I found: the numbers exist, and they are not reachable

The mixture evaluation holds exactly the needed quantities, in
`crates/uor-r4-training/src/geometric_stack.rs`:

```rust
/// One scored position of the mixture.
struct MixtureRow {
    /// The pointer's attention over sources `0..=t`.
    attention: Vec<f64>,
    /// `p_copy(target | t)`: exactly 0 when no source with attention holds the target.
    copy: f64,
    /// The gate `g_t`.
    gate: f64,
    /// `log((1 - g) softmax(z)[target] + g p_copy)`, with no floor on `p_copy`.
    log_mixture: f64,
    /// The shares of the mixture that came from each branch (sum to 1).
    generate_share: f64,
    copy_share: f64,
    /// Whether any source `0..=t` holds the target id (its attention aside).
    present: bool,
    ...
}
```

**`MixtureRow` is a private struct with no public accessor.** `attention` is the **full attention
vector over sources `0..=t`**, which is precisely what is needed to evaluate
`p_copy(target|t) = Σ_j a_tj [x_j = target]` at the value's digit positions — and precisely what
cannot be obtained from outside the crate.

**So step 1 is not "minutes of CPU" as the plan estimated; it needs a small addition first, and the
estimate in the plan was wrong.** I am recording that as the plan's error rather than working around it
with a proxy.

## The smallest addition, named exactly

A public accessor (or a crate-internal test) that, for a given `(ids, target)`, returns the mixture row:
at minimum **`attention`, `copy`, `gate`, `present`, `generate_share`, `copy_share`**. Then step 1 is:
for each of the 13 numeric rows, take the label's `query` position, set `target` to each of the value's
two digit ids, and report `copy` at both — plus the same at the distractor's positions and at the frame
positions the decode identified (`223` the bare space, `1498`, `2369`, `1156`, `2728`, `1044`, `754`,
`772`, `1790`), and the same totals for a few word rows as the control.

**That is a bounded addition: no new algorithm, no training, no weights touched — it exposes a value the
code already computes.**

## What CAN be said now, as a bound rather than the number

The trace already gives, per step, the pointer's **argmax source and its attention** — and the decode
established that **on 10 of 13 rows the argmax never lands on either stored digit, and on no row does it
land on the second.** Since `p_copy` sums the attention over positions holding the target,
`p_copy(digit) ≤ attention at the argmax` on those steps. **So the copy mass at the value's positions is
bounded above by an attention that is itself going elsewhere — but that is a bound, not the reading, and
it is reported as a bound.**

**It is not enough to decide the training piece.** The three pre-registered outcomes need `copy` itself:
high at the digits (nothing to train), low at the digits against a higher baseline elsewhere (the frame
reading survives), or low everywhere (the gate is broadly weak and the labels should say something
different).

## The ledger

Unchanged, and the read adds nothing to it: token count **REFUTED**; digit order **REFUTED**; value
addressability **REFUTED**; single-source shape **REFUTED in its simple form and replaced**;
"history-insensitive" **CORRECTED** to the frame's invariance; **the pointer attends the frame and never
the varying slot — MEASURED on 13 rows**. **This piece adds one operational fact: the p_copy read needs
an accessor before it can be run.**

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Add the public mixture-row accessor** (the fields are listed above), then run step 1 unchanged —
per-row `copy` at the value's two digits, at the distractor's positions and at the frame positions, with
word rows as the control. **The training plan stays on the shelf, not deleted:** it is not shelved
because the read said "nothing to train", but because the read **cannot be taken yet**, and the next
person needs both facts.

**Later note (2026-10-09), from the [copy-mass read](../pcopy-mass-read-2026-10-09/README.md): the read
was taken, and it did not need the mixture-row accessor.** `StackModel::score_targets` already exposes
`PointerRowStats { gate, copy_mass, hit, reachable }` and is already called on the reply path, so step 1
ran through a caller only. Gate passed: `hit` equals the recorded trace's `matches_source` at 166 of 166
steps across 13 rows. So the shelf's first reason ("the read cannot be taken yet") is now closed — and
the third reason is partially earned: the copy mass is 0.60–0.90 at the digits on the 3 rows whose
argmax already lands on a stored digit, and ≤ 0.20 against a 0.34–1.00 frame baseline on the other 10.
