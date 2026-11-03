# ORDER: infra-loop-harness, deepseek-108 (correct the recorded base)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the final CHANGES finding on PR #3622 at
`674d471c13a8a1dc24fe2f946573e7cead5f40c7`: every new task record and the PR
body named a non-existent base commit with transposed trailing digits.

## Cancel probe (run first)

At head `674d471c`, the recorded base
`9e0403b13e72cdc9cb840e9b62898ec33e0d614d` is absent from the object database,
while `git merge-base HEAD origin/main` is
`9e0403b132da34752499c657def95dfce3235538`. The repair stands.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/deepseek-104/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-105/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-106/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-107/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-108/**`

## Done when

1. Every affected ORDER and REPORT records merge base
   `9e0403b132da34752499c657def95dfce3235538`.
2. The PR body carries the same base.
3. The active report records the tested content commit, receipt/accounting commit
   behavior, final size and exact checks.
4. Report, exact-diff, local PR and full gates pass.
5. A fresh exact-head review approves the corrected record.

## Stop and ask

Change only task records and PR metadata. Do not change the image item,
implementation plan or tracked code.
