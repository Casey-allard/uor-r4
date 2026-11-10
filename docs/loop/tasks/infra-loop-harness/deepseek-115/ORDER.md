# ORDER: infra-loop-harness, deepseek-115 (release the merged stack-contract claim)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (record maintenance). Base: `adf38bcfa`.

## Goal

Remove the stale `deepseek/stack-contract` claim left in the committed coordination
store after PR #3919 merged and its branch was deleted.

## Cancel probe (run first)

`origin/main:docs/harness/claims.json` still contains
`deepseek/stack-contract`; the remote branch is absent and PR #3919 is merged.
The record is stale, and `multilab.claim_not_merged` would warn until it is
released.

## Files in scope

- `docs/harness/claims.json`
- `docs/loop/tasks/infra-loop-harness/deepseek-115/**`

## Done when

1. The merged branch's claim is absent from the committed coordination store.
2. No other claim changes.
3. Report, local PR policy, exact-head review, merge and fresh-main verification pass.

## Stop and report

Stop if the removal would change another lab's claim, if the branch is not already
merged, or if the release needs a source or model change.
