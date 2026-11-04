# ORDER: cpu-training-pilot, deepseek-108 (point vote complete replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `a66028b0e`.

## Goal

Measure whether deepseek-106's served eight-row point vote changes free
complete replies, not only the teacher-forced opening row. Use the existing
fixed 60/60 unseen-key split and deterministic CPU FBS session on both public
seeds. Compare point-off, nearest-row, eight-row vote and shuffled-target vote.

This is a bounded CPU development-readiness evaluation. An exact-reference
match is not semantic correctness, and no result can qualify useful chat.

## Design

- Extend `cpu-point-transfer-driver` with a bounded greedy reply pass.
- Before each evaluation prompt's last token, mark reply start.
- Feed the final prompt token, then feed each generated token back.
- Stop through the established integer `reply_stop` contract: BOS ends the
  reply without being included, otherwise the cap or context ends it.
- Record, per arm:
  - replies attempted;
  - BOS-complete, cap-stopped and context-stopped replies;
  - exact reference replies;
  - correct first token;
  - non-empty and distinct replies;
  - replies containing a repeated 4-gram;
  - generated and reference token totals;
  - per-arm wall time and generated tokens/s.
- Keep the four matched artifacts already produced by the driver.
- Keep independent examples as the only Rayon unit.

## Tests first

1. A finite reply passes prompt tokens once, feeds only generated tokens, stops
   at BOS, and reproduces a known exact reply.
2. The cap and context limit stop deterministically.
3. Exact-reference normalization removes one trailing BOS from the stored
   reference; a reference longer than the cap is classified unreachable;
   distinct, repeated-4-gram and cost counters count as named.
4. All counters aggregate without overflow.

## Done when

1. The free-generation instrument passes finite controls before public data.
2. Both fixed public seeds run without changing the split, cap or controls.
3. The REPORT records whether the eight-row vote improves complete replies,
   BOS completion, first tokens, exact references or repetition, with the
   exact negative outcomes first.
4. CPU/serving parity remains exact for all teacher-forced arms.
5. Tests, strict CPU Clippy, formatting, evidence, report, exact-head review,
   protected merge and fresh-main verification pass.

## Cancel probe

- `main` is clean at `a66028b0e`.
- Claude's #3629 only changes `train-in-space` and its docs, not this driver.
- The fixed deepseek-105 inputs and split remain available.
- `CpuFbsForward` exposes bounded stepping and context exhaustion.

## Stop and report

Stop on split drift, nondeterministic generation, counter overflow, or a
proposal to call exact-reference matches semantic correctness. Do not tune the
split, vote width, reply cap, random seed or controls after observing public
results.
