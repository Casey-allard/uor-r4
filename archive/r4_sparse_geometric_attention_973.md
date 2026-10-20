# #973 sparse geometric recurrent attention

**Date:** 2026-09-04
**Status:** `SPARSE_GEOMETRIC_MECHANISM_EXECUTED_LANGUAGE_UTILITY_UNESTABLISHED`
**Implementation revision:** `e93822e944d1a8edfbc314153abc4585f2dcafc5`

## Result

`R4SparseGeometricCandidateSoftmaxKVBindingV1` now selects a bounded subset of
the accepted fixed recurrent memory before reading K/V payloads. The eligible
pool remains eight exact live records plus four H4-local summary banks. The new
policy admits at most eight persistent records and always appends the transient
current record, so learned Q/K softmax sees at most nine sources instead of
thirteen.

The persistent budget equals the accepted live-window size. States containing
only the eight live records therefore preserve the comparator exactly. Sparse
behavior starts on the first read after an eviction has created a competing
summary. No fit or evaluation was performed.

## Selector and evidence boundary

For every occupied source frame `s` and current frame `q`, the selector obtains
the exact relative H4 element `s^-1 * q` from the sidecar's inverse and product
tables. It visits the nine signed S3 angular shells from coincident to
antipodal. If a shell crosses the remaining budget, it greedily maximizes the
minimum pairwise signed-S3 separation of the complete relative H4 roots, then
breaks remaining ties by causal age and physical slot. Only the selected slots
are gathered and transported. Their existing learned Q/K scores, softmax, and
value aggregation remain unchanged.

- **Exact construction:** the H4 inverse/product/root lookup, nine-shell order,
  deterministic tie rule, eight-record persistent budget, and nine-source read
  ceiling. Independent review reproduced the scalar-shell relation for all
  14,400 ordered pairs in the canonical H4 sidecar.
- **Measured behavior:** the focused mutation check and two source-backed
  prompt executions below. They establish that selection executes before K/V
  reads, summaries can be admitted, and work stays bounded.
- **Unverified hypothesis:** H4 shell proximity and full-root maximin diversity
  identify useful language memory.

The exact `q0/q1` heatmap, activation, chirality, cosine polarity, typed-null
projection, and all four `Z[phi]` root coordinates remain in the trace. The
heatmap is not an admission score. A typed-null two-coordinate chart does not
reject an otherwise valid full H4 root.

This is materially different from the older tested selectors, but it does not
erase their negative results. Shortest Cayley distance tied all six historical
queries in #967. The #970 `q0/q1` heatmap construction transferred strict
selection on 0/6 held-out queries and exposed eight incompatible classes. Those
results still prohibit treating either representation as established semantic
relevance.

## Frozen no-fit comparison

The new arm used the accepted artifact, tokenizer, exact-H4 geometry and frame
sidecar, seed 9738, top-k 40, temperature 0.8, and sixteen generated tokens.
The fixed recurrent comparator was read from its preserved JSON and was not
rerun.

| Prompt | Fixed recurrent continuation | Sparse continuation | Common generated prefix | Sparse decisions | Selected summaries | Score materialization |
|---|---|---|---:|---:|---:|---:|
| `A purple turtle found a clock in the garden` | `, there was a time, there was a little girl named but so she saw` | `, there was a time, there was a little girl named I saw a little` | 12 | 16 | 26 | 1,776 -> 1,512 (-14.86%) |
| `Albert Einstein was born in` | ` his friend, a time, there was a little girl named and and a time` | ` his friend, and Lily were very sad. He said, "So and` | 3 | 19 | 29 | 2,048 -> 1,728 (-15.63%) |

Across both executions, 35 decisions required sparse selection. The geometric
set differed from age-only top-eight on 33 of them and admitted 225 live plus 55
summary records. Aggregate materialized attention scores fell from 3,824 to
3,240 (-15.27% over whole prompts, including their initially non-sparse
prefixes). Peak attention sources fell from 13 to 9 (-30.77%). Both runs reached
12 eligible persistent records while selecting no more than eight.

Both runs reported zero complete-prefix scans, unselected K/V reads, teacher
calls, provider calls, future reads, and forbidden reads. The recurrent K/V
state remains 2,304 f32 values / 9,216 bytes. The two prompt trajectories are
uneven: one retained the prior 12-token common prefix and the other retained
three. That is a useful limit, not a language-quality verdict.

Preserved artifacts:

- sparse turtle raw JSON: SHA-256
  `b2ca5780bd5fc91faeab0ac9ee58737aff5fb0931f783ffdfd256d7fc82a7b4d`
- sparse Einstein raw JSON: SHA-256
  `3bcbddecda5e856d2cba095373f37769d6da6f16b7434c0ff4fb4ff793761018`
- comparison summary: SHA-256
  `d592b5bb9c430a509c465ac8dbb38c706a240fb980df18dee2c4a98ed560d396`

They remain outside Git under
`.uor-models/research/issue-973-sparse-geometric-r4-v1/comparison/`.

## Focused verification and review

```text
PYTHONPATH=tools/r4-softmax-trainer/src \
  .uor-models/research/issue-1014/venv/bin/python -m unittest \
  tools/r4-softmax-trainer/tests/test_fixed_recurrent_kv_binding.py

Ran 3 tests in 0.161s
OK
```

The focused test preserves fixed-reader parity through the eviction-producing
decision, verifies the exact inverse/product/root trace and shell order, and
mutates one omitted versus one admitted payload. The omitted mutation leaves
the sparse logits bit-identical; the admitted mutation changes them.

Independent source and artifact review found no blocking defect after one
per-lane trace-accounting correction. It also checked divergent 16-lane
selection and reconciled all final raw hashes.

## Limits and next action

The learned Q/K/V/O projections, bounded softmax, RMSNorm, SwiGLU, vocabulary
head, f32 tensors, allocation, and 120-token RoPE ceiling remain. Two prompts do
not establish useful retrieval, language quality, long-context retention,
geometric advantage, architectural alpha, reasoning, coding, table-native
serving, or release readiness.

The sparse geometric attention mechanical checkpoint is complete. #973 remains
open and owns the next build stage: implement one versioned nonlinear geometric
block while retaining this sparse reader and the dense SwiGLU block as
comparators. Selector-quality attribution belongs with scale/data work unless a
direct regression blocks that implementation.
