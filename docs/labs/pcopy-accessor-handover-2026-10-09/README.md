# The accessor is bounded, and here is exactly where it goes — not completed

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, no knob, no model saved,
$0.** No v5 re-run.

## What this piece was asked to do

Build the smallest exposure of the mixture row and then run step 1 unchanged: per-row `copy` at the
value's two digit ids, at the distractor's positions and at the frame positions the decode identified,
with word rows as the control.

## What I found, and why it is not a one-line accessor

The mixture row is **not merely a private struct** — it is **derived from the loss/generation path's
hidden states**, and it is constructed at exactly two sites in
`crates/uor-r4-training/src/geometric_stack.rs`:

**Site 1 — the loss path (`pointer_loss_from_hidden`), where the row is a training artifact:**

```rust
let logits = self.linear(&hidden, p.head()?)?;
let side = self.pointer_side(p, hidden)?;
let beta = self.pointer_beta(p)?;
Ok(logits.contiguous()?.apply_op3(&side.contiguous()?, &beta.contiguous()?, PointerMixture {
    time, dim: pointer.dim, score: pointer.score, identity: pointer.identity,
    select: pointer.select, route: pointer.route, ids: ids.to_vec(),
    keys: self.pointer_route_keys(ids), targets: targets.to_vec(), ...
```

**Site 2 — the GENERATION path, which is the one that matters here:**

```rust
.hidden_hooked(&p, ids, batch, time, &mut None)?.detach();
let logits = self.linear(&hidden, p.head()?)?;
let side = self.pointer_side(&p, &hidden)?;
let beta = one_value(&self.pointer_beta(&p)?)?;
...
let op = PointerMixture { time, dim: pointer.dim, score: pointer.score, ... };
```

**Site 2 already builds the mixture at inference time**, from a hidden state the model computes anyway,
and `MixtureRow::evaluate` already returns exactly the needed fields (`attention`, `copy`, `gate`,
`present`, `generate_share`, `copy_share`).

**So the accessor IS bounded — but it is a small new code path, not a visibility change:** a public
function beside Site 2 that returns the `MixtureRow` for a given `(ids, target)` instead of only the
scores. Two things make it more than a one-liner: the private `PointerMixture`/`MixtureRow` types must be
surfaced through a small public struct, and the function must be placed on the generation path so it uses
the same hidden state the decoder used — otherwise it would measure a different mixture than the one
that produced the replies, which is the failure mode the voided trace already taught us to avoid.

**I did not complete it in this piece.** My remaining runway is not enough for a change to the model's
generation path plus a build plus the run plus the record, and a half-built accessor that measures a
*different* mixture than the decoder used would produce exactly the plausible-but-wrong numbers this line
has spent eleven pieces learning to refuse. **So the piece lands as the handover, not as the read.**

## The exact change the next attempt needs

1. **A small public struct** carrying `attention: Vec<f64>`, `copy: f64`, `gate: f64`, `present: bool`,
   `generate_share: f64`, `copy_share: f64` — the six fields step 1 needs, nothing more.
2. **A public function on the generation path** (Site 2), `pointer_mixture_row(&self, ids: &[u32],
   target: u32) -> Result<Option<PublicRow>>`, reusing the existing `hidden_hooked` → `linear` →
   `pointer_side` → `pointer_beta` → `PointerMixture` → `MixtureRow::evaluate` chain **unchanged**.
3. **A check that it measures the decoder's mixture**: the row's `copy` at a step whose argmax the trace
   already recorded must be consistent with the trace's `attention` at that same source — i.e. the new
   path is validated against the instrument that reproduces the sealed replies 13 of 13, not trusted on
   its own.
4. **Then step 1 unchanged**: per-row `copy` at the value's two digits, the distractor's positions, the
   frame positions (`223`, `1498`, `2369`, `1156`, `2728`, `1044`, `754`, `772`, `1790`), and word rows
   as the control — per-row numbers, not aggregates.

**Choose the exposure deliberately:** a `pub fn` on `StackModel` is the smallest thing that lets a
**binary** run step 1, and it should be named so a future reader does not mistake it for a supported
serving interface — it is a read-out for an instrument, and the record should say so at the declaration.

## The bound stays, and it is still only a bound

`p_copy(digit) ≤ attention at the argmax` on steps where the argmax is elsewhere — a ceiling, not the
reading. It stays published beside the (still absent) measurement so a reader can see whether the
ceiling was tight once the numbers exist.

## The shelving rule, with BOTH reasons in sequence

The training plan stays on the shelf, and the record now carries the complete history of why it was
never run:

1. **First reason — "the read could not be taken yet"** (2026-10-09, `pcopy-read-attempt`): `MixtureRow`
   is private, so step 1 was not runnable as the plan wrote it. **The plan's own first step was not
   runnable as written, which was the plan's error.**
2. **Second reason — "the accessor is a small new code path, not a visibility change"** (this piece):
   the row is derived from hidden states at two construction sites, and the one that matters is the
   generation path; getting it wrong measures a different mixture than the decoder used.

**Neither reason is "the read said nothing to train."** That reason is still unearned, and the plan is
**not deleted** — the next person needs all three possibilities on the record.

## The ledger

Unchanged: token count **REFUTED**; digit order **REFUTED**; value addressability **REFUTED**;
single-source shape **REFUTED in its simple form and replaced**; "history-insensitive" **CORRECTED** to
the frame's invariance; **the pointer attends the frame and never the varying slot — MEASURED on 13
rows.** This piece adds a **code-level fact**: the mixture row is reachable only through the hidden-state
path, and the accessor is a small new code path to be built on the generation side.

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Build the accessor at Site 2 as specified, validate it against the 13-of-13 trace, then run step 1
unchanged.** It remains CPU-only, no pod, no training, no weights touched — and the three pre-registered
outcomes stay distinguishable: `copy` high at the digits (nothing to train → the emitter), low at the
digits against a higher baseline elsewhere (the frame reading survives → single-rung labelled run), or
low everywhere (the gate is broadly weak → the labels should say something different).
