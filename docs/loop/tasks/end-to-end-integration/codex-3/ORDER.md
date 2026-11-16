# ORDER: end-to-end-integration, codex-3

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-2](../codex-2/ORDER.md). Reviewed head: `74d487308`.

## Why this unit exists

The owner asked, twice, not to forget the CIRISKernel work. ADR-0007 attached it to `Context`: a
bounded ring whose spans may be overwritten **only when settled**, with a stub that keeps exactly
what the declared read consumes. Before any session wiring, this unit fixes that contract with two
exact demonstrations and one witness pair:

1. a **fold read** (a commutative sum) is preserved exactly by a ring plus a checkpoint stub;
2. a **selection read** (top-k) is *not* preserved by that stub — it needs a mergeable bounded
   sketch — and with the sketch it is exact;
3. an unsettled overwrite has a witness pair: two histories the stub cannot distinguish that a
   later read sends to different answers. That is why the gate must refuse it.

This is the kernel's actual contribution (settlement, not rolling counters), stated in the form our
serving path can use.

## Claim record

`loop claim end-to-end-integration --lab-topic context-window-probe` was refused with exit 2 because
`crates/uor-r4-integer/src/stack/**` overlaps claude-393's PR #4014. This unit adds only a new test
file and docs, touches none of that path, adds no dependency and does not change `Cargo.lock`; it
continues under the owner's D85 direction.

## Method

- A `Span` carries an id and a value; the ring has a fixed capacity, a cursor, and a checkpoint.
- `push(span, settled)` writes the next slot; if the slot is occupied and **unsettled**, it returns
  the blocking id and changes nothing. If settled, the span's stub is folded into the checkpoint and
  the slot is reused.
- The fold read is `checkpoint + Σ live values`.
- The selection read is the top-k by value over all spans.
- The stub is a mergeable bounded top-k (insert-and-keep-k, ties by id), so it is associative and
  commutative; it is the analogue of the kernel's fixed-size row stub.
- Tests replay a stream of several wraps and compare both reads against a full-history record at
  every step; then exhibit the witness pair for a selection read whose stub kept only the sum.

## Files in scope

- `crates/uor-r4-integer/tests/context_window_probe.rs` (new file)
- `docs/loop/tasks/end-to-end-integration/codex-3/**`

Out of scope: source, schema, session wiring, the read interface, any dependency, any GPU run, and
any claim about a model.

## Done when

1. The fold read is bit-identical to full history at every step across at least three wraps.
2. The selection read with the mergeable sketch is bit-identical; with a sum-only stub it is not,
   and the failing pair is printed as the witness.
3. An unsettled push is refused, leaves the ring unchanged, and the witness pair shows the
   divergence it prevents.
4. The fold and sketch merges are associative and commutative on a sample; the ring is bounded.
5. The test passes; `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if the contract needs a source change or a schema field, or if the demonstration cannot be
made exact without approximations. Do not wire the ring into `IntegerStackSession` here.

STOP and report when done.
