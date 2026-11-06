# ORDER: cpu-training-pilot, deepseek-198 (full-sequence suffix arbitration)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `0e3924274`.

## Goal

Train reply-target suffix memory over full prompt-plus-reply histories so its
keys and local exact copy inspect the same token stream. Use a confident global
suffix only when its matched depth is greater than the local exact-copy depth;
preserve local copy on ties. Stopping remains the independent true
deepseek-191 boundary.

This is D84 CPU development research. It is exact-memory recall evidence, not
fresh-wording generalization, useful chat, serving integration or GPU
qualification.

## Why this is next

- deepseek-193 replaced local copy and over-stopped.
- deepseek-196 preserved local copy but made global fallback depend on local
  absence. Local exact copy had a candidate at 25,202/25,202 reachable TRAIN
  decisions, so treatment was structurally unreachable.
- Both experiments trained on reply-only history, while local copy sees the
  prompt. This gives the two memories different context surfaces.
- Full-sequence suffix keys expose prompt-conditioned reply continuations and
  allow a same-surface depth arbitration: deeper global context may replace a
  shallow local match; ties and deeper local matches stay local.

## Fixed construction

For every TRAIN conversation:

1. Start history with the prompt tokens.
2. For each normalized reply token, count transitions from every suffix of
   history length 1 through 32 to that reply token.
3. Append the reply token to history and continue.
4. Do not add decoder BOS or any terminal successor.

At each generated step:

1. Apply deepseek-191's true reply-boundary memory.
2. Read local `Longest32To1` on the complete prompt-plus-generated stream.
3. Read the global content table on the same stream.
4. Accept a global prediction only with at least two observations and a strict
   majority successor.
5. Treatment uses global only when its matched depth is strictly deeper than
   the local candidate; otherwise local wins.
6. With no local or global candidate, use the writer.

## Arms

Per public artifact seed:

- `deeper_full_sequence_32`: treatment;
- `deeper_full_sequence_shuffled`: successor-multiset-preserving shuffle;
- `shallow_local_global_4`: global confident suffix may replace a local match
  of depth at most four, regardless of relative depth;
- `full_sequence_32_only`: global confident suffix without local priority;
- `local_longest_32_boundary`;
- `writer_boundary`;
- `reference_length_oracle`: exact-body ceiling.

All arms use the same artifact and true boundary memory.

## Data and power

- table from TRAIN panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2774, protocol 2, 128 conversations,
  generated only after FIT and GO;
- both public dual-c131 raw artifact seeds;
- 64-token generation cap;
- at least 200 reference-reachable replies per artifact seed;
- at least 100 reachable treatment decisions where global depth exceeds local;
- at least one changed decision from local longest and from the shuffled
  treatment per artifact seed;
- fresh-process repeats of both artifact seeds.

## Bar

On both artifact seeds, `deeper_full_sequence_32` must beat local longest,
shallow-local depth-4, shuffled, global-only and writer on correct body
positions by:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

It must keep premature BOS at or below four per panel, make at least 100
reachable deeper-global replacements, and repeat exactly.

Exact complete replies are the primary capability outcome. If the reference
oracle has fewer than ten exact bodies, exact comparison is
`INSUFFICIENT_POWER`; a body pass is retained only as
`BODY_POSITIVE_INSUFFICIENT_COMPLETION`. With at least ten oracle-exact bodies,
treatment must also beat local longest on exact complete replies by the paired
rule.

## Done when

1. The full-sequence table, depth arbiter, evaluator, CLI and focused tests are
   implemented.
2. A FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. TRAIN preflight confirms at least 100 activated deeper-global decisions and
   non-vacuous local/shuffled controls without opening seed 2774.
4. Seed 2774 and both artifacts run with every control and repeat.
5. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
6. Reviewed merge, protected queue merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, table, confidence, depth rule, parity,
determinism or output-root drift; fewer than 200 reachable replies; fewer than
100 deeper-global treatment decisions; a vacuous shuffle; any need to edit
serving, CUDA, float-training, exporter or Claude-owned source; wall time above
30 minutes; or peak RSS above 2 GiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-198/ORDER.md
