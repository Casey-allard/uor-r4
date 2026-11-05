# ORDER: cpu-training-pilot, deepseek-152 (close copy-key router)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `292a57213`.

## Goal

Release the completed `deepseek/copy-key-router` coordination claim and record
the post-merge verification for PR #3693. This is bookkeeping only.

The delivered unit rejected the fitted depth-and-age precision router because
it lost to longest-first. No model, language or serving claim follows.

## Done when

1. The claim is removed from `docs/harness/claims.json`.
2. Deepseek-151 records exact-head review, protected merge and fresh-main
   verification.
3. Claims JSON and task records validate.
4. Exact-head review and protected merge complete.

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
