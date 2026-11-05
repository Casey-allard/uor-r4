# ORDER: cpu-training-pilot, deepseek-196 (confident content suffix fallback)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `ebbe084e9`.

## Goal

Separate content continuation from stopping and preserve the established local
`Longest32To1` body. Train a reply-only suffix table with no decoder-BOS target,
then use a confident global suffix only when local exact copy has no candidate.
The true deepseek-191 boundary remains the only stopping path.

This is D84 CPU development research. It is exact-memory recall evidence, not
fresh-wording generalization, useful chat, serving integration or GPU
qualification.

## Why this is next

- deepseek-193's table included BOS transitions and globally replaced local
  longest-32. It reached 57/86 correct body positions against 514/514 for
  local copy and made 14/227 premature stops.
- That result confounds two changes: removing the strong local body and adding
  termination to the same memory.
- This unit leaves local copy in control and removes BOS from the trained suffix
  table. A confident suffix may act only where local exact copy is absent.
- The match isolates whether global content recurrence can repair local
  fallback without displacing the body signal.

## Fixed construction

Add `derive_suffix_content_memory` and a separate fallback evaluator in
`uor-r4-cpu-train`.

For each normalized TRAIN reply:

1. Count every suffix of length 1 through 32 ending at a reply position whose
   successor exists.
2. Do not count decoder BOS or any terminal transition.
3. Preserve exact integer target counts.

At generation:

1. Apply deepseek-191's true reply-boundary memory before each token.
2. Run local `Longest32To1`; use it whenever it has an exact candidate.
3. Otherwise search generated-reply suffixes from depth 32 down to one.
4. Use a global suffix only when it has at least two observations and its
   plurality target is strictly more than half of them.
5. Otherwise fall back to the writer.

## Arms

Per public artifact seed:

- `local_then_content_32`: treatment;
- `local_then_content_depth_1`: depth-one confidence fallback;
- `local_then_shuffled_content_32`: successor-multiset-preserving shuffle;
- `content_32_only`: global confident suffix replacement;
- `local_longest_32_boundary`: established local body;
- `writer_boundary`;
- `reference_length_oracle`: exact-body ceiling.

All arms use the same true reply-boundary memory and artifact. No arm receives a
BOS target from the content table.

## Data and power

- table from TRAIN panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2773, protocol 2, 128 conversations,
  generated only after FIT and GO;
- both public dual-c131 raw artifact seeds;
- 64-token generation cap;
- at least 200 reference-reachable replies per artifact seed;
- at least 100 reachable confident-suffix fallback decisions in the treatment;
- at least one changed decision from local longest and from the shuffled
  fallback per artifact seed;
- fresh-process repeats of both artifact seeds.

## Bar

On both artifact seeds, `local_then_content_32` must beat local longest,
depth-one fallback, shuffled fallback, content-only and writer on correct body
positions by the paired rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must keep premature BOS at or below four per panel, activate at least 100
reachable confident-suffix fallbacks, and repeat exactly.

Exact complete replies are the primary capability outcome. If the reference
oracle has fewer than ten exact bodies, the exact comparison is
`INSUFFICIENT_POWER`; a body pass is retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`. With at least ten oracle-exact bodies,
the treatment must also beat local longest on exact complete replies by the
paired rule.

## Done when

1. The content-only table, fallback evaluator, CLI and focused tests are
   implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. TRAIN-only preflight confirms activated, non-vacuous true and shuffled
   fallback decisions without opening the fresh panel.
4. Seed 2773 and both artifacts run with every control and repeat.
5. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
6. Reviewed merge, protected queue merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, table, confidence rule, parity, determinism or
output-root drift; fewer than 200 reachable replies; fewer than 100 confident
fallback decisions; a vacuous shuffle; any need to edit serving, CUDA,
float-training, exporter or Claude-owned source; wall time above 30 minutes; or
peak RSS above 2 GiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-196/ORDER.md
