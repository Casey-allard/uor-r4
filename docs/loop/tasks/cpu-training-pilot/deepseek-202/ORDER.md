# ORDER: cpu-training-pilot, deepseek-202 (continuation closure)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `d56427955`.

## Goal

Replace per-token stop classification with a completion-level continuation
objective. When the fixed deepseek-198 content source selects a confident
global suffix, accept a whole observed remaining-reply continuation only when
that same suffix has a strict-majority complete continuation. Emit that
continuation as a bounded closure and terminate at its natural end.

This tests whether predicting completion jointly with content removes the
premature-stop failure without abandoning the positive exact-memory content
selector. It is D84 CPU development research, not fresh-wording
generalization, serving integration or GPU qualification.

## Why this is new

- deepseek-200 changed terminal keys to full prompt-plus-reply history, but
  full-history and reply-only boundaries made identical stop decisions.
- Every premature stop followed a global-content decision; another stop-table
  retune or source-label gate is not a different objective.
- This unit predicts the complete remaining reply from the selected global
  suffix, so content and termination are one decision rather than a token
  followed by a separate BOS gate.
- A matched control permutes complete continuations among equal-depth suffixes.

## Fixed construction

For each TRAIN conversation:

1. Preserve the exact prompt and normalized reply.
2. At the prompt boundary and after every generated reply token, count each
   full-history suffix of depth four through 32.
3. The value is the complete remaining normalized reply, including the empty
   continuation at the final token.

At each generated decision:

1. Compute deepseek-198 local `Longest32To1` and confident content prediction
   on the complete stream.
2. Use global content only when its matched depth is strictly deeper than
   local; equal-depth ties remain local.
3. If treatment selects global, look up a confident whole continuation at
   that same suffix depth. A continuation requires at least two observations
   and a strict-majority complete continuation whose first token agrees with
   the selected global token. An empty terminal continuation is allowed.
4. Emit that complete continuation when it fits the cap and context, then
   stop with BOS. Otherwise continue tokenwise from the selected source.
5. With no global selection, preserve tokenwise local, then writer.

No dense matmul, softmax, squashing activation or float serving path is used.

## Arms

Per artifact seed:

1. `source_closure`: treatment;
2. `tokenwise_content`: deepseek-198 content/local arbitration with the
   deepseek-191 per-token reply boundary;
3. `source_closure_shuffled`: equal-depth continuation shuffle;
4. `closure_only`: confident whole continuation without local/content
   arbitration, then writer;
5. `content_no_boundary`;
6. `local_reply_boundary`;
7. `writer_boundary`;
8. `content_reference_oracle`.

## Data and power

- TRAIN from panel `tt-2711` only;
- fresh unopened `cell=dev,dev`, seed 2776, protocol 2, 128 conversations;
- both public dual-c131 raw artifact seeds;
- 64-token cap;
- at least 200 reference-reachable replies per artifact seed;
- at least 100 closure rows whose complete reply differs from tokenwise content
  or local reply output;
- non-vacuous closure shuffle and at least one changed complete reply versus
  tokenwise content, closure-only, no-boundary, shuffled and local;
- fresh-process repeats of both artifact seeds.

## Bar

On both artifacts, `source_closure` must:

- keep premature BOS at or below four;
- produce at least `max(5, ceil(content_oracle_exact / 2))` exact replies;
- beat tokenwise content, closure-only and shuffled closure on exact replies
  by the paired two-sigma rule;
- keep correct body positions at or above `content_no_boundary` and
  `local_reply_boundary`, and at or above `closure_only`;
- make at least 100 closure rows whose complete reply differs from tokenwise
  or local output;
- differ from tokenwise, closure-only, no-boundary, local and shuffled on at
  least one reachable decision;
- repeat exactly after timing removal.

If the oracle has fewer than ten exact bodies, exact comparison is
`INSUFFICIENT_POWER`. A vacuous continuation source or shuffle is
`CONTROL_FAULT`.

## Compute class

Local CPU development on the Ryzen 9 9950X. No GPU, pod, paid API or external
model. Serving remains integer-only and unchanged.

## Cost

- preparation and tests: 10 minutes;
- release build: 1 minute;
- generator build and fresh panel: 2 minutes;
- TRAIN preflight: 2 minutes;
- four artifact runs and two repeats: 2 minutes;
- evidence and delivery: 15 minutes;
- total bounded increment: 32 minutes;
- whole run: 30-minute deadline, 2 GiB per evaluator, 32 Rayon workers;
- retained evidence: at most 32 MiB; fresh target cap: 8 GiB.

## Done when

1. Continuation table, evaluator, CLI and focused tests are implemented.
2. FIT-reviewed pre-registration and fresh D67 GO are recorded.
3. TRAIN preflight shows confident, non-vacuous closure activation.
4. Seed 2776 and both artifacts run with every control and repeat.
5. REPORT, compact evidence, notebook, item/current state and research program
   record the result.
6. Reviewed merge, protected queue merge, fresh-main verification and claim
   release complete.

## Stop rule

Stop on source, artifact, panel, table, continuation, parity, determinism or
output-root drift; fewer than 200 reachable replies; fewer than 100 closure
selections; a vacuous shuffle; any need to edit serving, CUDA, float-training,
exporter or Claude-owned source; wall time above 30 minutes; peak RSS above
2 GiB; or retained evidence above 32 MiB.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-202/ORDER.md
