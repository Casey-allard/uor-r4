# ORDER: cpu-training-pilot, deepseek-193 (trained reply-suffix continuation memory)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `1f1e3a75`.

## Goal

Test whether a discrete suffix-to-next-token memory trained only from TRAIN
replies improves complete-reply body reconstruction over the current local
`Longest32To1` exact-copy selector. The table records every reply suffix of
length 1 through 32 and its observed continuations, including decoder BOS for
the terminal transition. At generation, the longest suffix present in the
table selects its most frequent successor, ties to the smaller token id, and
falls back to the artifact writer when no suffix is known.

This is D84 CPU development research. It is exact-memory recall evidence, not
fresh-wording generalization, useful-chat qualification, serving integration or
GPU qualification.

## Why this is next

- deepseek-191 exercised reply-boundary stopping, but the oracle completed only
  seven fresh bodies. Body reconstruction is now the measured limiting factor.
- `Longest32To1` copies only from the current prompt/generated stream and keeps
  the latest source occurrence at the deepest exact depth.
- The content readouts tested so far learn from generated geometric states and
  can arbitrate only a narrow candidate set; several lose to `Longest32To1` or
  lack training power.
- No prior unit trains a global discrete reply-suffix continuation table and
  compares its longest-context plurality against a matched successor shuffle,
  a depth-one table, local exact copy and the writer.

## Fixed construction

Add an isolated suffix-memory module and evaluator in `uor-r4-cpu-train`.

For every normalized TRAIN reply `r`:

1. For each observed reply position `i`, count the transition from every suffix
   ending at `r[i]`, at lengths `1..=min(32, i + 1)`, to `r[i + 1]`.
2. For the final reply position, count the same suffixes to decoder BOS.
3. Preserve exact integer counts; no floating point is used.

At each generated step:

1. Inspect only the generated reply suffix, not the prompt.
2. Search suffix depths from longest to shortest.
3. At the first known depth, select the most frequent successor; break equal
   counts toward the smaller token id.
4. Emit decoder BOS when that successor is BOS; otherwise continue through the
   exact CPU FBS writer for the next state.
5. Fall back to the writer token when no suffix is known.

## Arms

Per public artifact seed:

- `trained_suffix_32`: the treatment above;
- `trained_suffix_depth_1`: the same table restricted to depth one;
- `shuffled_suffix_32`: the treatment with successor labels redistributed
  deterministically across equal-depth observations while preserving each
  suffix's observation count, every target count and the global successor
  multiset at that depth;
- `local_longest_32_boundary`: the established local `Longest32To1` body with
  deepseek-191's true reply-boundary memory;
- `writer_boundary`: artifact writer with the same true boundary memory;
- `reference_length_oracle`: local longest body stopped at the reference
  length, reported only as the exact-body ceiling.

The treatment may override the writer only. It does not alter the artifact,
serving session, Hamming read, point key, CUDA or float-training code.

## Data and power

- derive the suffix table only from the fixed TRAIN panel `tt-2711`;
- evaluate a new unopened `cell=dev,dev`, seed-2772, protocol-2, 128-conversation
  m-world-v2 panel generated only after FIT and GO;
- both public dual-c131 raw artifact seeds;
- 64-token generation cap;
- at least 200 reference-reachable replies per artifact seed and arm;
- at least one treatment decision changed from local longest and from the
  shuffled table per artifact seed;
- at least 100 known-suffix decisions per artifact seed;
- fresh-process repeat of both artifact seeds.

## Bar

For both artifact seeds on the fresh panel, `trained_suffix_32` must beat each
of `local_longest_32_boundary`, `trained_suffix_depth_1`,
`shuffled_suffix_32` and `writer_boundary` on correct body positions by the
existing paired rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must also keep premature BOS at or below four per panel. A non-vacuous
shuffle and exact replay are required.

Exact complete replies are read as the primary capability outcome. If the
reference-length oracle has fewer than ten exact bodies, the exact comparison
is reported `INSUFFICIENT_POWER`; a body-position pass is then retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`, not as a complete-reply or language
result. If the oracle has at least ten exact bodies, the treatment must also
beat local longest on exact complete replies by the paired rule.

## Done when

1. The isolated module, evaluator, CLI and focused tests are implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. TRAIN-only preflight confirms both treatment and shuffled control are
   non-vacuous without opening the fresh panel.
4. The fresh panel and both artifact seeds run with every control and repeat.
5. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
6. Exact-head review, protected merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, suffix-table, parity, determinism or output-root
drift; fewer than 200 reachable replies; fewer than 100 known-suffix treatment
decisions; a vacuous successor shuffle; any need to edit serving, CUDA,
float-training, exporter or Claude-owned source; wall time above 30 minutes; or
peak RSS above 2 GiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-193/ORDER.md
