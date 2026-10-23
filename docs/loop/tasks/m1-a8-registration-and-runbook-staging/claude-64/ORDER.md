# ORDER: m1-a8-registration-and-runbook-staging, claude-64 (correct the A8 step-1.1 identity pins to measured values)

Lab: claude. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md). Base: `origin/main`. `References` the A8 line; the item status is unchanged.

## Goal

Step 1.1 of the A8 runbook checks out the pinned commit `fd409fcdf3bc43d607e768bae6f8cf403aa75901` and requires (`docs/labs/m1-a8-registration-and-runbook-staging/README.md:60-61`):

- `compiled_identity.source_sha256 == "a5d59806…"`
- `compiled_identity.geometry_sha256 == "46ddbb76d4f7…"`

The runbook calls any mismatch an instrument blocker. Neither value matches what that commit's binary reports. On 2026-10-23 a pod run measured both, with Casey's approval (#2842, 6372197194; results in the HF store at `results/claude/craw-a8id-20261023/`, with `SHA256SUMS`). It built `fd409fcdf` with `--features cuda` (nvcc 12.8.93, `sm_120`) and ran `discrete-stack build-identity`:

- `compiled.source_sha256 = 177f1b3da11eddbe2cf90d8172c48d33fafca95b533ef8898f3e8d2c245edcfc`
- `compiled.geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`

The geometry value also matches 134 other records on main, including `verify_a8_address_cells_dose.py:453`, which was written for the same commit. `a5d59806…` matches the dose verifier's separate 3-file `compiled_source_sha256` (`verify_a8_address_cells_dose.py:1417`), a different quantity.

This unit corrects the two step-1.1 pins and records the erratum in the staging sheet.

## Cancel probe (run first)

At the base, do the two step-1.1 lines still read `a5d59806…` and `46ddbb76d4f7…`? If either is already corrected, the task is void. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-registration-and-runbook-staging/**`
- `docs/harness/prep/m1-a8-registration-and-runbook-staging*`
- This task folder

## Done when

1. **Runbook pins.** `README.md:60-61` carry the measured `177f1b3d…` and `46ddbb76b7fb…`, with their provenance (commit, nvcc version, HF path) beside them.
2. **Sheet erratum.** The sheet's geometry pin is corrected, and a dated erratum line records the measured `compiled_identity.source_sha256`. It notes that `a5d59806…` is the dose verifier's 3-file hash, a different quantity. The `TEST FITNESS: FIT` block is untouched.
3. **Checks pass:** `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Stop and ask if

- Any other step-1.1 value would need changing.
- A file outside scope must change.
