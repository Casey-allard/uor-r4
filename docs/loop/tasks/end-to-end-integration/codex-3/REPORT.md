# REPORT: end-to-end-integration, codex-3

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic context-window-probe` was refused with exit 2
  because `crates/uor-r4-integer/src/stack/**` overlaps claude-393's PR #4014. This unit adds only a
  new test file and docs, touches none of that path, adds no dependency and does not change
  `Cargo.lock`; it continues under the owner's D85 direction.
- **The first refusal test aimed at the wrong slot.** With two slots, the third push reuses the
  *settled* slot 0, so the unsettled span in slot 1 is only reached by the fourth push. The test
  failed with `Ok(())` versus `Err(2)`; the ring was right and the test was wrong. It now pushes
  once more and asserts the refusal and the unchanged state.
- **This is a probe, not the session.** Nothing is wired into `IntegerStackSession`; the reads are a
  toy sum and top-k, not the Hamming or flock reads; no source, schema or artifact changes.
- **The sketch has a hard precondition:** its capacity must be at least the read's `k`, or the
  dropped candidates cannot answer. The probe documents it and does not enforce it at runtime.
- **The audit vector is unbounded.** `Ring.dropped` keeps the dropped ids so the test can count
  wraps; the real ring would not keep it. It is the one unbounded part of the probe.
- **No model claim, no cost measurement.** The demonstration shows exactness and the witness pair;
  it does not measure a real session's bytes or tokens.

## What changed

- `crates/uor-r4-integer/tests/context_window_probe.rs` (new, test-only):
  - a bounded ring of slots, a cursor, and a stub per dropped span;
  - `push(span, settled)` refuses an unsettled eviction with the blocking id and leaves the ring
    unchanged; a settled span is folded into the stub and dropped;
  - a mergeable bounded top-k stub (insert-and-keep-k, ties to the smaller id) that is associative
    and commutative;
  - a fold read `checkpoint + Σ live` and a selection read `top-k(stub ∪ live)`.

Size: 294 lines of test and the ORDER/REPORT.

## Measurements

- **Fold read:** exact against full history at every step of a 40-span stream through a capacity-8
  ring, i.e. 32 drops and at least three wraps.
- **Selection read:** exact against full history at every step with a sketch of 4, i.e. the dropped
  candidates still answer a top-3 read.
- **Witness pair:** histories `[3, -1, 0]` and `[1, 1, 0]` share the sum `2` and the live span but
  their top-1 differ (`3` vs `1`); a sum-only stub cannot distinguish them, a one-candidate sketch
  can.
- **Refusal:** an unsettled slot returns `Err(id)` and the ring's sum and top-k are unchanged.
- **Merge:** associative and commutative on a sample, and equal to the full-history top-k.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --test context_window_probe` — `3` passed.
- `rustfmt --check` on the new file — clean.
- `cargo clippy --locked --no-deps -p uor-r4-integer --test context_window_probe` — no warning in
  the new file.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the probe tests the two properties the contract needs (fold exactness and
  selection exactness only with a sketch) and exhibits the failure, rather than asserting the
  mechanism is safe.
- `scope-guard`: a new test file only.
- `test-freeze`: a new file; no existing test changed.

## Still open

- **Session wiring:** a `Context` behind the `Read` stage, with the actual Hamming/flock reads, a
  stub sized to the read, and the overflow policy replacing the session's refusal, the float
  slide, the discrete drop and the `StackChat` reset.
- **Settlement criterion for our reads:** a span may be dropped when its contribution cannot change
  the selection. For a fold read that is trivial; for a top-k read it is the sketch precondition.
  The exact criterion must be written per read before wiring.
- **Serialization** of the stub into the session snapshot, and `restore_state` for the mechanisms it
  currently refuses.
- **I1** the `Read` interface, now with a concrete reason for its `ReadOut` shape: `support`,
  `weights_q31` and the stub's candidate set.

## Plain summary

The CIRISKernel transfer is now precise. A bounded ring may overwrite a slot only when the span is
settled; a fold read survives exactly with a sum stub; a selection read needs a mergeable top-k
sketch of at least its `k`, and without it there is a witness pair: two histories with the same
stub and different answers. That is the rule the serving context will use, and it is the first
piece of the kernel that is directly checkable in our arithmetic.
