# ORDER: cpu-training-pilot, deepseek-154 (release copy-reply claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `ab49404cf`.

## Goal

Release the completed `deepseek/copy-key-longest-reply` coordination claim and
record the exact-head merge and fresh-main verification for PR #3697. This is
bookkeeping only.

The merged result is selection-surrogate positive but termination-limited: the
body-position comparisons passed, while no arm completed a reply. No model,
language or serving claim follows.

## Done when

1. The claim is removed from `docs/harness/claims.json`.
2. Deepseek-153 records exact-head review, protected merge and fresh-main
   verification.
3. Claims JSON and task records validate.
4. Exact-head review and protected merge complete.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-153/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-154/**`

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
