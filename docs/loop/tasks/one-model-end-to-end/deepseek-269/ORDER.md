# ORDER: one-model-end-to-end, deepseek-269 (reply-control trace)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `c0f7db014`.

## Goal

Add an observation-only reply-control trace to the raw-id instrument. At every
generated reply position it records:

- nearest top-one and runner-up distance, and their margin;
- the EOS token's writer-table rank and distance;
- whether the turn memory, point key, Hamming copy heads or Hamming type heads
  fired, with their keys or kept positions;
- short-cycle state and reply position;
- an `opaque`, `reflective` or `permeable` control class.

The trace must not change a token. Each traced reply is replayed untraced
through the product session and must match id for id, or the instrument fails.

This unit changes no serving arithmetic, artifact, trainer, exporter or bar.

## Cancel probe

At the base, `raw-id-replies` can emit product replies and exact early-stop
visit counts, but it records no per-reply-position control row. Its replay
cannot show that a tracing probe selects the same token as the served step.
The unit stands.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`
- `crates/uor-r4-integer/src/generation.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-269/**`

## Done when

1. `raw-id-replies --trace-control 1 --trace-out FILE` writes one deterministic
   JSONL row per generated reply token.
2. The record includes the required nearest, EOS, memory/copy/type, cycle,
   position and class fields.
3. The probe-selection token equals the product-served token at every row; any
   mismatch fails the run.
4. Every traced reply equals a fresh untraced product replay, including EOS or
   the token cap.
5. The same trace enabled and disabled emits identical reply files and no
   serving token changes.
6. Focused unit tests cover ranking, class priority, cycle recording,
   memory-state conversion and mismatch refusal.
7. Existing integer tests, formatting, Clippy on changed crates, report check
   and local PR check pass, or a pre-existing warning is named explicitly.

## Stop and report

Stop if the trace needs a serving arithmetic change, cannot use the public
product session, or discovers that traced and untraced replies differ.
