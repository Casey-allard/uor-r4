# ORDER: cpu-training-pilot, deepseek-158 (release generated-stop claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `663129a83`.

## Goal

Release the completed `deepseek/copy-generated-stop` coordination claim and
record the exact-head merge and fresh-main verification for PR #3701.

The merged result rejects the fixed all-reference-length terminal labeling with
eight signed Hamming prototypes. It is a bounded negative, not a rejection of
geometric termination.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-157/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-158/**`

## Done when

1. The claim is removed from `docs/harness/claims.json`.
2. Deepseek-157 records exact-head review, protected merge and fresh-main
   verification.
3. Claims JSON and task records validate.
4. Exact-head review and protected merge complete.

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
