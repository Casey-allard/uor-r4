# Integration Translation Plan

## Objective
Move the current routed frontier out of the proxy-regression harness and into a more model-like systems path without reopening geometry prematurely.

## Chosen First Path
Token-memory retrieval preselection.

## Why This Path
1. It is closer to a real model integration point than bucket-mean regression.
2. It measures the thing the project claims to improve: routing enough of the search space away that fewer candidates need expensive comparison.
3. The repo already has the required ingredients:
   - LM token cache
   - hashed context features
   - stable routed frontiers
   - route/memory coordinate support
4. It can preserve the existing JSON-summary, parser, summarize, and gate-note workflow.

## Minimal Harness
1. Build a token-memory bank from LM proxy training items.
2. Use exact dense retrieval as the control.
3. Use routed preselection to probe one or more route buckets, then perform exact local retrieval only inside the selected candidate set.
4. Predict:
   - hashed target vector mean for MSE continuity with the existing contract
   - token top-1 via majority vote for a more model-like retrieval metric
5. Log:
   - quality: `test_mse_after`, `test_top1_after`
   - systems: `total_sec`, `retrieval_candidate_count_mean`, `retrieval_candidate_fraction_mean`, `retrieval_probe_bucket_mean`
   - route health and Poincare alignment for routed branches

## First Screen
Compare:
- `DENSE`
- `HOPF_RET_P1`
- `HOPF_RET_P2`
- `HOPF_PHI2_RET_P1`
- `HOPF_PHI2_RET_P2`

with:
- `task_script=tasks/router_retrieval_eval.py`
- `baseline_route_id=DENSE`
- current cheap frontier chart schedule (`IT40_P2` family)
- medium translation subset before escalation to near-full scale

## First Decision Rule
Keep the path only if at least one routed branch:
1. stays close enough to dense retrieval on quality to remain credible, and
2. materially reduces candidate fraction, and
3. does not erase the current systems advantage with chart or routing overhead.

## Stop Rule
If the first retrieval screen fails because dense exact retrieval remains dominant even after candidate pruning, the next step is not new geometry. The next step is cost decomposition inside the translated harness.
