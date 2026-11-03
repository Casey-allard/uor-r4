# ORDER: infra-loop-harness, deepseek-110 (refresh the recovered item view)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Resolve the final CHANGES finding on PR #3622 at
`39f5cd99246c35b5152ae427ed123bf8e427f344`: the recovered image item is in
the source set, but the generated roadmap view still reports 23 ready/138 total
and omits the item and dependency edge.

## Cancel probe (run first)

At head `39f5cd992`, `roadmap render --check` fails because the view reports
23 ready/138 total, while the source set contains 24 ready/139 items including
the image item. The repair stands.

## Files in scope

- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-109/REPORT.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-110/**`

## Done when

1. The roadmap renderer is used rather than a hand edit.
2. `roadmap render --check` and the docs archive/link workflow pass.
3. The active report records that rendered-view and generated-output policy was
   followed, including any advisory status.
4. Report, exact-diff, local PR and full gates pass or any policy refusal is
   explicitly escalated rather than bypassed.
5. A fresh exact-head review approves the final registration.

## Stop and ask

Stop if repository policy refuses a non-render-PR change to
`docs/roadmap/README.md`; record the deferred render dependency instead of
bypassing the policy.
