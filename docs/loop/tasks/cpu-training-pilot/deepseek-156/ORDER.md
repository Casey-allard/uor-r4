# ORDER: cpu-training-pilot, deepseek-156 (release termination-oracle claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `d5246431b`.

## Goal

Release the completed `deepseek/copy-key-termination-oracle` coordination
claim and record the exact-head merge and fresh-main verification for PR
#3699. This is bookkeeping only.

The merged result is a bounded termination ceiling: with oracle BOS at the
reference length, longest-first completed 11 train and 13 dev replies. No
deployable decoder, language or serving claim follows.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-155/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-156/**`

## Done when

1. The claim is removed from `docs/harness/claims.json`.
2. Deepseek-155 records exact-head review, protected merge and fresh-main
   verification.
3. Claims JSON and task records validate.
4. Exact-head review and protected merge complete.

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
