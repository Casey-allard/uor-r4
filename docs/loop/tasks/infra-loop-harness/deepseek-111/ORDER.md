# ORDER: infra-loop-harness, deepseek-111 (record setup row)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the review-record refusal on PR #3622 at
`83011dc666648cf4931513969b855573a94bd550`. The manually written
maintenance ORDER chain omitted the required `Setup row: 2` field.

## Cancel probe (run first)

At head `83011dc66`, `loop review` refuses the exact-head record because the
active ORDER and its predecessors name no setup row. The repair stands.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/deepseek-104/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-105/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-106/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-107/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-108/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-109/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-110/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-110/REPORT.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-111/**`

## Done when

1. Every ORDER in the chain names the valid setup row 2.
2. The active REPORT records the correction and exact final size.
3. Report, exact-diff, local PR and full gates pass.
4. A fresh exact-head review is postable and counts.

## Stop and ask

Do not change the image item, requirements, scope or implementation plan.
