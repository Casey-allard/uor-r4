# ORDER: infra-loop-harness, deepseek-106 (finish image-item fidelity)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the fresh-context CHANGES review of PR #3622 at
`0f9fa0d20c1849388a8538e7ad992b5b9d774ee2`.

The implementation scope is now valid. Remaining errors are record fidelity:
the active report's diff size is stale, its gate-head wording is ambiguous, two
issue work-card requirements were generalized away, and copied relative links
in the PR body do not resolve outside the repository.

## Cancel probe (run first)

At head `d9a646718`, the report's size and head accounting are stale, two issue
requirements are generalized, and copied PR-body links fail outside the
repository. The repair stands.

## Files in scope

- `docs/roadmap/items/geometric-image-input.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-105/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-106/**`

## Done when

1. The item restores the issue's exact-usage and truthful 501/413 requirements.
2. The active report records the exact final diff and distinguishes the
   implementation head from the receipt-only commit.
3. The active report names the runnable link-check command.
4. The PR body uses absolute URLs for task and item references.
5. Roadmap, link, report, exact-diff, local PR and full gates pass.
6. A fresh exact-head review approves the final registration.

## Stop and ask

Do not implement image decomposition or weaken any signed-off requirement.
