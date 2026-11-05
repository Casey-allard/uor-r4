# ORDER: cpu-training-pilot, deepseek-153 (longest-first copy in complete replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e06635e3a`.

## Goal

Test whether deepseek-149's longest-first exact suffix selection improves
complete generated replies, rather than only isolated next-token decisions.
This decides whether Claude's next trained copy read should preserve depth
three and four, or whether copy selection is already limited by reply
termination and should wait for that repair.

This is D84 CPU development research on the Ryzen 9 9950X. It changes no float
training, integer serving, CUDA or Claude-owned source.

## Fixed construction

At every generated decision, use the last 192 tokens of the exact stream
consisting of the prompt and all generated tokens so far. For a query ending at
the current final token, search backward inside that fixed window for the latest
exact suffix key at each depth one through four:

```text
query_d(t) = x[t], x[t-1], ..., x[t-d+1]
key_d(s)   = x[s-1], x[s-2], ..., x[s-d]
```

Here `0 < s < t`; the candidate token is `x[s]`, the latest match at a depth
wins, and missing history at the start repeats `x[0]`, matching deepseek-149.

The five arms are:

- `longest_4_to_1`: the deepest available exact match, then the first
  generated token from the artifact when none exists;
- `k2_backoff_k1`: depth two, else depth one, else the artifact token;
- `fixed_depth_1`: depth one, else the artifact token;
- `random_available_depth`: a seeded uniform depth over the depths that have a
  match, else the artifact token;
- `writer`: the unmodified integer artifact token.

The policy may replace only the next token chosen by the artifact's existing
writer. It does not alter the forward state, memory, point key, artifact,
stopping contract, serving format or writer tables. The selected token, writer
token, selected depth and fallback state are recorded for every decision. The
same policy is applied to every artifact seed.

## Inputs

- Public trained artifacts:
  - seed 1:
    `local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
    SHA-256
    `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
  - seed 2:
    `local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
    SHA-256
    `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Tokenizer:
  `local/deepseek/m2-evidence/tokenizer.json`,
  SHA-256
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.
- Attempt 1 stopped before producing a result because its generated panels
  contained 46- and 49-token assistant replies under a 32-token cap.
- The corrected attempt uses two new m-world-v2 panels, drawn after FIT and GO
  with the pinned
  `heldout-panel` generator:
  - `copy-key-reply-tt-2711`: `cell=train,train`, 128 conversations, seed
    `2711`;
  - `copy-key-reply-dd-2712`: `cell=dev,dev`, 128 conversations, seed `2712`.

Panel generation uses protocol 2 and the pinned tokenizer. The decision run
uses every recovered prompt/reply example from each panel, with the maximum
supported 64-token generation cap. Each artifact seed and panel must supply at least 200
reference-reachable examples; paired comparisons use only those examples.
Otherwise the run stops as a power or instrument fault before a result is read.

## Measurements

For each artifact seed, panel and arm, report:

- examples and reference-reachable examples;
- exact-reference replies, requiring token identity and BOS termination;
- BOS-complete replies, opening-correct replies and non-premature BOS stops;
- correct generated body positions and total reference body positions;
- generated tokens and generated tokens per second;
- copy overrides, exact-suffix matches, writer fallbacks and selected depth;
- random-control changes from longest-first.

Paired comparisons are over the same prompt/reply example. They use the
project's two-sigma rule:

```text
candidate sum > control sum
candidate-only >= max(2, ceil(2 * sqrt(control-only)))
```

## Bar

This is a configuration screen, not a language or serving result.

- **Complete-reply transfer passes** only if `longest_4_to_1` beats
  `k2_backoff_k1`, `fixed_depth_1`, random depth and writer on exact-reference
  replies and non-premature BOS completions, by the paired rule, for both
  artifact seeds on both panels.
- **Selection-surrogate transfer is positive but termination-limited** if the
  complete-reply bar fails while longest-first beats `k2_backoff_k1`,
  `fixed_depth_1` and random depth on correct reference body positions by the
  paired rule for both artifact seeds on both panels, with no loss of
  non-premature BOS stops against writer.
- **Negative** is every other outcome. It closes only this fixed complete-reply
  writer-override configuration, never exact copy, depth three/four or the
  geometry generally.

The random arm is non-vacuous only if it changes at least one longest-first
decision on each panel. A fresh process must reproduce the full report byte for
byte after paths and timing fields are excluded.

## Stop rules

Stop on an artifact, tokenizer or panel hash mismatch; fewer than 200
reference-reachable examples for any artifact-seed and panel cell;
nondeterminism; a vacuous random control; a prompt outside the artifact context
or a reference longer than the 64-token cap; a malformed or unsupported policy
trace; a resource breach; or any need to change serving, CUDA, float training or
Claude-owned source. A recovered example whose prompt plus reply leaves no room
to request BOS remains in the input census but is excluded from paired
comparisons.

## Instrument gate

Finite tests must establish latest-match selection, depth precedence,
`k2>k1` fallback, deterministic random selection, writer fallback, BOS/cap/
context stopping and exact accounting. A known synthetic artifact must prove
that a selected copy token is fed to the same CPU forward path. The public
artifacts must parse and their pinned hashes must match before panel generation.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-key-longest-reply-2026-11-05.md`
- `docs/harness/prep/cpu-training-pilot-copy-key-longest-reply-64-2026-11-05.md`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-153/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`
- `docs/harness/ledger/deepseek-2026-11.jsonl`

## Done when

1. The policy, generated-reply evaluator, accounting and finite controls have
   tests.
2. A fresh-context FIT and independent GO precede the registered CPU run.
3. Both artifact seeds run on both newly generated panels, and a fresh-process
   repeat matches after path and timing fields are excluded.
4. Evidence, notebook row, REPORT, exact-head review, protected merge and
   fresh-main verification complete.
