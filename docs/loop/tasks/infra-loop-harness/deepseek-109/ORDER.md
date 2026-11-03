# ORDER: infra-loop-harness, deepseek-109 (close head and link accounting)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the final review of PR #3622 at
`4101aeb61047715ed2196c702c067e73a3af05c4`.

The tested content commit and receipt hashes were correct, but the following
commit's description omitted its update to the content-commit line, and the PR
body linked an ORDER path that does not exist on `main` before merge.

## Cancel probe (run first)

At head `4101aeb61`, the closing-commit description omits the content-commit
line, and the PR-body ORDER URL returns 404 before merge. The repair stands.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/deepseek-108/REPORT.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-109/**`

## Done when

1. The active report enumerates every report and receipt change in the final
   accounting commit.
2. It records the exact tested content commit and exact final diff size.
3. The PR body uses an inline ORDER path rather than a pre-merge `main` link.
4. Report, exact-diff, local PR and full gates pass.
5. A fresh exact-head review approves the final registration.

## Stop and ask

Do not change the image item, source code, scope or acceptance criteria.
