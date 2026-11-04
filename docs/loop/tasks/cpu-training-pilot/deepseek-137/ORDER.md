# ORDER: cpu-training-pilot, deepseek-137 (Z/256Z content signature)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `8cf088bd9`.

## Goal

Test whether preserving the low byte of every Q12 generated-query coordinate
in a 2,048-bit modulo-256 code improves the positive content readout enough to
reduce its severe signature ambiguity and extend matching-prefix depth.

The current 256-bit sign code gives each coordinate one bit. The candidate
keeps the complete low-byte residue `value mod 256` for each of 256
coordinates. XOR plus popcount remains the only distance operation: no
matmul, softmax or float arithmetic.

This is D84 CPU development research. It does not change integer serving,
CUDA, float training or Claude's end-to-end track.

## Cancel probe

Deepseek-136 established a positive aligned-content override with the same
generated-state rows, 90% leave-one-out threshold, writer fallback and
effective-mapping shuffled control. Its sign code had only `199/894` and
`201/894` leave-one-out hits and `892/894` and `879/894` rows with multiple
targets at the minimum distance. Claude PR #3664 contains only P3 scoring
sheet/ORDER changes; no integer or CPU-training file overlaps this unit.

## Fixed construction

Use the public 98/103 repeated-key split, training-derived memory and actual
generated training trajectories exactly as deepseek-136.

For each decision, compare these two signatures over the same Q12 query:

1. `sign`: one bit per coordinate, set when the coordinate is non-negative;
   and
2. `code`: the low eight bits of each coordinate, stored at its coordinate
   position, for 2,048 bits total.

Fit each signature's maximum Hamming distance over at least 90% of its
leave-one-out training rows. At decode, choose the commonest target among
rows at the minimum distance and break equal-count target ties by smaller
token. Fall back to the exact writer when distance exceeds that signature's
fitted threshold.

## Arms and bar

Choose each mode's deterministic target rotation only after verifying that it
changes the effective commonest-target map keyed by unique signature.

Run, over complete 16-token generated replies:

1. exact writer;
2. true sign readout with writer fallback;
3. shuffled sign readout;
4. true modulo-256 code readout with writer fallback; and
5. shuffled code readout.

Both seeds must pass:

- each true mode has at least 640 generated rows, 16 targets and 90%
  leave-one-out threshold coverage;
- each shuffled mode changes at least one effective signature assignment;
- code held-out aligned top-1 exceeds writer and code-shuffled by the
  prompt-level rule `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`
  and has a positive aggregate gain;
- code matching-prefix depth exceeds writer and code-shuffled by the same
  rule and has a positive aggregate gain;
- code top-1 and matching depth are not below sign on either seed, with at
  least one strictly above per seed;
- code leave-one-out top-1 exceeds sign on both seeds; and
- seed-1 repeat matches every non-timing field and evidence file.

Report exact references, stops, opening accuracy, ambiguity, fired and
fallback decisions, distinct/repeated replies, workers, wall and RSS. Exact
replies are reported, not gated.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-content-code-2026-11-04.md`
- `docs/labs/cpu-training-pilot-content-code-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-137/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned changes.

## Done when

1. The existing content-readout path deterministically supports both sign and
   modulo-256 code signatures without changing the writer's arithmetic.
2. Tests cover both packings, XOR distance, threshold fitting, fallback, ties,
   effective-map shuffle refusal, alignment and worker-count determinism.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve either outcome; notebook and roadmap
   classify whether the code improves or only reproduces the sign result.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- A mode cannot fit 90% coverage or has fewer than 640 generated rows.
- Either shuffled control leaves the effective map unchanged.
- The comparison needs integer, serving, CUDA or Claude-owned changes.
