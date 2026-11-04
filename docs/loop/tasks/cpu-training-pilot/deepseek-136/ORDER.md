# ORDER: cpu-training-pilot, deepseek-136 (Hamming content readout)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `74919a81a`.

## Goal

After three admission/writer failures, pivot to a different geometry: an exact
256-bit Hamming nearest-example content readout over actual generated-state
writer queries. It predicts the next content token directly from training
trajectories and bypasses writer-cell admission.

This is D84 CPU development research. It is not yet an integer-serving change,
GPU qualification or useful-chat claim.

## Cancel probe

Deepseek-132 classified terminal/nonterminal endings; point gates targeted
reply openings; deepseek-133/134/135 repaired or moved writer admission.
None predicts content continuation from generated-state signatures. Claude PR
#3664 remains limited to its GPU scoring sheet.

## Fixed construction

Use the public 98/103 repeated-key split and training-derived memory. Derive
rows only from actual generated training trajectories: each complete 256-Q12
query becomes a sign signature and contributes the reference content token at
the same offset.

Fit one training-only maximum Hamming distance covering at least 90% of
leave-one-out training rows. At decode, choose the commonest target among
training rows at the minimum distance; ties choose the smaller token. Fall back
to the exact writer token when distance exceeds the fitted threshold.

## Arms and bar

Run writer fallback, true Hamming readout and a deterministic shuffled-target
readout using the same signatures and threshold.

Both seeds must pass:

- training has at least 640 rows and both target diversity and 90% fitted
  coverage;
- true held-out aligned generated top-1 exceeds writer and shuffled by the
  prompt-level paired threshold `max(2, ceil(3 * sqrt(b + c)))`;
- true held-out matching-prefix depth exceeds both controls by the same paired
  rule and has a positive aggregate gain;
- the shuffled target multiset differs; and
- seed-1 repeat matches every non-timing field and result digest.

Report exact references, stops, opening accuracy, nearest-distance coverage,
distinct/repeated replies, workers, wall and RSS. Exact replies are reported,
not gated.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-content-readout-2026-11-04.md`
- `docs/labs/cpu-training-pilot-content-readout-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-136/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned changes.

## Done when

1. A CPU-only exact signature/readout/evaluator derives generated rows, fits
   the coverage threshold, and runs true/shuffled/free-reply arms.
2. Tests cover packing, XOR distance, ties, threshold fit, fallback, shuffle,
   alignment and deterministic generation.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve either outcome; notebook and roadmap
   classify the result.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Fewer than 640 generated training rows or 90% coverage is achievable.
- Shuffling leaves the target mapping unchanged.
- The readout cannot be evaluated without a serving or Claude-owned change.
