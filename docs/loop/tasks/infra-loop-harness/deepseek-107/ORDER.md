# ORDER: infra-loop-harness, deepseek-107 (finish head accounting)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the final CHANGES finding on PR #3622 at
`d9a646718a44e88caed011562375d598182a106c`: the active report said the final
commit added only receipts, but that commit also changed the report's size
line.

## Cancel probe (run first)

At head `d9a646718`, comparing that commit with its parent shows both
`REPORT.md` and `receipts.jsonl` changed. The receipts-only description is
therefore still wrong and the repair stands.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/deepseek-106/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-107/**`

## Done when

1. The active report names the exact commit where full gates ran.
2. It states that the following receipt/accounting commit changes both
   `receipts.jsonl` and the report size line.
3. The active report contains the final cumulative diff size.
4. Report, exact-diff, local PR and full gates pass.
5. A fresh exact-head review approves the corrected record.

## Stop and ask

Do not change the image item, scopes, requirements or implementation plans.
