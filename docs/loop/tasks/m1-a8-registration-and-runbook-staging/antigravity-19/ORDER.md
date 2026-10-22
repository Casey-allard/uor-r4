# ORDER: m1-a8-registration-and-runbook-staging, antigravity-19

Lab: antigravity. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md). Base: `767f653b2`.

## Goal

The item's done-when reads:

> Finite-qualification sheet registered with its own binding and its own FIT, with the admission-preserving sheet identity (`fcb770e6…`) unchanged (#2785 is already merged); runbook and executor staged; the exact owner-release command staged for Casey to type.

## Cancel probe (run first)

Ran `shasum -a 256 docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` (`fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`), confirmed `gh pr view 2785 --json state,mergedAt` is `MERGED` (`2026-10-20T16:48:41Z`, `c6eb05265`), and verified that neither `docs/harness/prep/m1-a8-registration-and-runbook-staging*` nor `docs/labs/m1-a8-registration-and-runbook-staging/` exists on `origin/main`. Continue.

## Files in scope

- `docs/harness/prep/m1-a8-registration-and-runbook-staging*`
- `docs/labs/m1-a8-registration-and-runbook-staging/**`
- `docs/roadmap/items/m1-a8-registration-and-runbook-staging.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/m1-a8-registration-and-runbook-staging/antigravity-19/**`

Expected diff size: ~420 lines across 4 markdown/staged files (`docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md`, `docs/labs/m1-a8-registration-and-runbook-staging/README.md`, `docs/roadmap/items/m1-a8-registration-and-runbook-staging.md`, `docs/roadmap/README.md`).

## Done when

1. One-page finite-qualification registration sheet `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md` is committed with all 8 required `prep.rs::REQUIRED_BLOCKS` sections (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: FIT`), pinning source `fd409fcdf3bc43d607e768bae6f8cf403aa75901` (`#2782`), protocol `train61-synthetic1-interval64-two-arm-throughput-v1`, scope `m1-attention-active-audit-finite-source-qualification-v1`, TRAIN anchor `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`, bounded-update and restoration controls, and the readiness bar `R = added_gate_cost / ungated_two_arm_dose_cost <= 15%` from `fcb770e6…` §4 and `docs/labs/admission-preserving-qualification-2026-10-19/README.md`.
2. `docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md` remains byte-identical at SHA-256 `fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`.
3. `docs/labs/m1-a8-registration-and-runbook-staging/README.md` stages the complete audited-dose and finite-qualification runbook, executor sequence, six-column research notebook row, and exact owner-release commands (`attention-audit-qualification-owner-release` and `attention-audit-owner-release`) for Casey to type personally with `UOR_LAB` unset.
4. A non-author fresh-context reviewer verifies `TEST FITNESS: FIT` on the finite-qualification sheet and approves the PR at the exact head.

## Stop and ask if

- Any file outside the declared scope must change.
- Any GPU pod, live release, or spend is required.
