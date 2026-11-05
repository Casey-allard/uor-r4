# ORDER: cpu-training-pilot, deepseek-149 (hierarchical copy-key depth)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `4e42ffca5`.

## Goal

Test whether exact copy retrieval improves when the Hamming read backs off from
the longest available suffix key, depth four to one. This screen decides
whether Claude's next copy-read unit should extend beyond depth two.

This is D84 CPU development research. It changes no float training, integer
serving, CUDA or Claude-owned source.

## Fixed construction

For a 192-token scoring window, copy its tokens from `win_start` through the
last eligible target into `x`. For an eligible masked target at corpus
position `p`, let `t = p - 1 - win_start`. The query is the suffix ending at
`x[t]`. At depth `d`, its key is:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

A candidate is `1 <= s < t`, excluding the current query position. The latest
matching `s` predicts `x[s]`; the target is `x[t+1]`. Missing history at the
start of `x` repeats `x[0]`, matching Claude-267's depth-two reference. Search
stays inside the same BOS-delimited document and scoring window.

Compare these fixed and deterministic selectors:

- fixed depths `1`, `2`, `3` and `4`;
- `k2>k1`: latest depth-2 candidate, otherwise latest depth-1 candidate;
- `longest`: latest candidate at the greatest available depth from four to one;
- `shortest`: the fixed-depth-1 control;
- a seeded random-depth selector over available depths, as the non-vacuous
  matched control.

The public DEV result is an already-opened instrument oracle. The implementation
must reproduce Claude-267's fixed-depth counts: 8,243 copyable positions,
depth-1 6,306 matches / 3,859 right, depth-2 4,383 / 3,506, with a 192-token
context. Any mismatch stops the run as an instrument fault. Public DEV is never
used for the candidate bar.

## Panels

1. Public M1 DEV, only for exact instrument parity with Claude-267.
2. `copy-key-depth-tt-2501`: a new 64-conversation `train,train` panel.
3. `copy-key-depth-dd-2502`: a new 64-conversation `dev,dev` panel.

Panels 2 and 3 are drawn after FIT and GO from the pinned m-world-v2 generator
with those fixed seeds. Earlier exploratory readings on `train-control-v3`,
`dev-panel-v3` and `panel-dev-dev` are recorded as exploratory only and do not
enter the decision.

## Bar

The candidate is `longest`.

- It must beat `k2>k1` on true target correctness on both new decision panels.
- The gain must pass the project's paired two-sigma rule over eligible target
  positions: candidate sum greater than control sum and candidate-only
  disagreements at least `max(2, ceil(2*sqrt(control-only)))`.
- It must beat the seeded random-depth selector by the same rule.
- Random-control novelty is counted only on copyable positions and against the
  `longest` candidate it controls.
- Every fixed-depth and selector arm must account for every eligible position,
  and a fresh-process deterministic repeat must match.

## Stop rules

Stop on a public-DEV parity mismatch, input/hash drift, nondeterminism,
out-of-window candidate leakage, a vacuous random control, or any need to
change serving or Claude-owned source.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `.gitattributes`
- `docs/harness/prep/cpu-training-pilot-copy-key-depth-2026-11-05.md`
- `docs/labs/cpu-training-pilot-copy-key-depth-2026-11-05/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-149/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`

## Done when

1. The exact copy-key oracle, boundary/window checks, accounting and controls
   have tests.
2. A fresh FIT and GO precede the registered CPU run.
3. Public DEV reproduces Claude-267's fixed-depth figures before any candidate
   reading is accepted; the two decisions panels are newly drawn at the fixed
   seeds.
4. The result is sealed, classified and delivered through protected main.
