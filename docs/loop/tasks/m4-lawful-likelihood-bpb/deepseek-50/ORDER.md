# ORDER: m4-lawful-likelihood-bpb, deepseek-50 (residual record clean-up)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `903dbf701`. `References #2861`.

## Goal

PR #3060 merged at `23eb1c275` with the pre-registration sheet and the single-interval outcome rule. Three review residuals were committed after that merge and did not land: (1) the lab design note's control 7 still says zero-count mass is "infinite and is reported as such", contradicting the merged `MassZero` control and the sheet; (2) `deepseek-49/REPORT.md` still says "no D40 release exists" where CHARTER §5.1 means "no owner release exists"; (3) the sheet header does not cite the design PR #2940. This unit lands exactly those three doc corrections.

## Cancel probe

Class instrument (docs). At the base: `git show origin/main:docs/labs/m4-lawful-likelihood-bpb/README.md | grep -c 'infinite and is reported'` = 1; the sheet header lists #2945 but not #2940; the REPORT says D40. Task stands.

## Files in scope

- `docs/labs/m4-lawful-likelihood-bpb/**`, `docs/harness/prep/m4-lawful-likelihood-bpb*`, `docs/loop/tasks/m4-lawful-likelihood-bpb/**`

## Done when

- The three residuals are corrected, checks pass (`prep-check` PENDING/complete, report, pr-check, gates), a docs-only D46 fresh-context record is at the head, and the PR is merged and verified.
- No content, bar, stream hash or FIT-status change; the sheet stays `TEST FITNESS: PENDING`.