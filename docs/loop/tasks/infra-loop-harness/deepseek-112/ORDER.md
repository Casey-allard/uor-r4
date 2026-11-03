# ORDER: infra-loop-harness, deepseek-112 (complete ORDER cancel probes)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the final CHANGES finding on PR #3622 at
`595e605f2bae8f275b3db41c984a9906962f8fab`: the repair ORDER chain carried
the required setup row but omitted each order's cancel probe.

## Cancel probe (run first)

At head `595e605f2`, the active ORDER and repair ORDERs 105–111 have no
`## Cancel probe (run first)` section, while the charter and ORDER template
require one. The registration content itself is complete. The unit stands
unless an existing exception permits repair orders to omit this required
section.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/deepseek-104/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-105/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-106/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-107/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-108/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-109/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-110/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-111/ORDER.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-111/REPORT.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-112/**`

## Done when

1. Every ORDER from deepseek-104 through deepseek-112 contains a non-empty
   `## Cancel probe (run first)` section.
2. The active report records the correction and exact final size.
3. Report, exact-diff, local PR, roadmap and full gates pass.
4. A fresh exact-head review is postable and counts.

## Stop and ask

Do not change the image item, roadmap, requirements, scope or implementation
plan.
