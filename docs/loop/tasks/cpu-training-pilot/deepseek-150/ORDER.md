# ORDER: cpu-training-pilot, deepseek-150 (close copy-key depth)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `ab0a759cd`.

## Goal

Release the completed `deepseek/copy-key-depth` coordination claim and record
the post-merge verification for PR #3690. This is bookkeeping only.

The delivered unit selected the longest-first copy key at depths four to one
for a trained-read test. No model, language or serving claim follows.

## Done when

1. The claim is removed from `docs/harness/claims.json`.
2. Deepseek-149 records exact-head review, protected merge and fresh-main
   verification.
3. Claims JSON and task records validate.
4. Exact-head review and protected merge complete.

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
