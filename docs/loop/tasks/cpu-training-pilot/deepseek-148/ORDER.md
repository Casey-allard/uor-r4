# ORDER: cpu-training-pilot, deepseek-148 (release merged claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `67736f850`.

## Goal

Release the completed `deepseek/trajectory-density` coordination claim after
PR #3687 merged. The experiment itself is already delivered and verified:

- fresh `main` contains implementation source pin `1bf1a72fa`;
- the merged evidence reports 16/84 endpoint recall and 11/1,484 false stops;
- fresh-main release tests pass 261 library and 10 driver tests with 4 expected
  ignores;
- the evidence manifest and resource ledger match `main`.

This unit changes only `docs/harness/claims.json`. It runs no model compute and
does not alter a result, bar, serving path or Claude-owned source.

## Done when

1. The merged branch claim is removed.
2. The claims JSON remains valid.
3. Exact-head review and protected merge complete.
4. Fresh `main` no longer lists the completed claim.

## Stop rules

Stop if the claim owner or branch differs, another live lane holds the same
claim, or the edit changes anything beyond coordination metadata.
