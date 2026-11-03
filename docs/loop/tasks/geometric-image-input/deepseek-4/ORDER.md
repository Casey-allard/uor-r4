# ORDER: geometric-image-input, deepseek-4 (merged-head evidence repair)

Lab: deepseek. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619). Base: `8fb9f3719`.

## Goal

Repair record drift found by the fresh exact-head reviews after PR #3628
merged. The image implementation passed; its report retained 451 integer
tests even though current `main` has 452 passing and 1 ignored, and it stated
that all loop gates passed on the first attempt even though a retained fake-gh
`ETXTBSY` flake required an immediate rerun.

## Design

- Keep the historical 451-test receipt unchanged.
- Add an exact receipt for the merged-head integer result: 452 passed, 1
  ignored.
- Retain the failed gate attempt and its passing immediate rerun.
- Correct only the current REPORT, notebook count, checksum manifest and
  evidence needed for those facts. Do not change image code, model behavior,
  capabilities or historical results.

## Tests first

1. Re-run the integer library at the post-merge source tree and require the
   recorded count.
2. Verify every evidence checksum.
3. Run report validation, local PR policy and all loop gates.

## Done when

1. Current REPORT and notebook state 452 integer tests while preserving the
   earlier 451 receipt.
2. The `ETXTBSY` first attempt and green immediate rerun are both retained.
3. Exact-head review, protected merge and fresh-main verification pass.

## Cancel probes

- `main` contains the complete #3628 implementation at base `8fb9f3719`.
- The remaining differences are records only.
- The historical evidence files are still present and must remain unchanged.

## Stop and report

Stop if the repair would rewrite a historical receipt, change image behavior,
or claim a capability beyond the experimental R4 input path.
