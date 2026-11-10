# REPORT: infra-loop-harness, deepseek-115

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `adf38bcfa`.

## What did not work, first

PR #3919 was merged and its local and remote branch were removed, but its claim
remained in `docs/harness/claims.json`. `loop close --verify` proved that the
worktree and branch were safe to delete but did not rewrite the merged claim.
The committed store therefore disagreed with the branch state.

## What changed

`uor-harness team release --branch deepseek/stack-contract --lab deepseek`
removed exactly one claim from `docs/harness/claims.json`. No source, model,
scope or other lab's claim changed.

## Measurements

No model measurement applies. The change removes one stale coordination record.

## Checks run at the head

- `uor-harness team release --branch deepseek/stack-contract --lab deepseek`:
  released the claim.
- `jq` confirms the branch key is absent and claims remain valid JSON.
- `uor-harness loop report --check`, local PR policy and the exact-head review
  run after this report commit.

## Skills used

None. This is a coordination-record repair.

## Still open

The broader claim store still contains older merged claims from other units.
They are outside this order and are not changed here.

## Plain summary

The stack-contract work merged, but its temporary “this branch is being worked
on” marker survived in the repository. This change removes only that marker so
the coordination record matches `main`.
