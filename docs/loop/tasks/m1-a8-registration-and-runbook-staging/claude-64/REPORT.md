# REPORT: m1-a8-registration-and-runbook-staging, claude-64 (correct the A8 step-1.1 identity pins to measured values)

Lab: claude. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md). The item status is unchanged.

## What did not work, first

1. **The A8 runbook's own step-1.1 identity check would have stopped the first A8 pod session.** Both pinned values were wrong for the commit the runbook checks out (`fd409fcdf`):
   - **Geometry.** The pin `46ddbb76d4f7…` appears in only 3 places on main. Every other record (134), the dose verifier for the same commit, and the binary itself report `46ddbb76b7fbe6ac…`. It looks like a short hash expanded by hand.
   - **Source.** The pin `a5d59806…` is the dose verifier's 3-file `compiled_source_sha256`, not the binary's `compiled_identity.source_sha256`, which covers about 17 source files plus the nvcc PTX.
2. **My first note to the PM was wrong.** I said merging #3071 would need re-staging. It doesn't, because the runbook builds the pinned commit, not main. I corrected it on #2842 (6371771003).
3. **The source digest depends on the nvcc version**, because it covers the PTX. The measured value holds for nvcc 12.8, the `uor-pod` bootstrap's version.

## What changed

| File | Change |
|---|---|
| `docs/labs/m1-a8-registration-and-runbook-staging/README.md` | Step 1.1 now pins `compiled_identity.source_sha256 == 177f1b3d…` and `geometry_sha256 == 46ddbb76b7fb…`, with a dated erratum giving the measurement's provenance. |
| `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md` | The geometry pin is corrected. The measured `compiled_identity.source_sha256` is added beside the verifier's 3-file hash, labelled as a different quantity. The `TEST FITNESS: FIT` block is untouched. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Pod run** (Casey's approval: #2842, 6372197194; one RTX 5090, about 20 minutes). `fd409fcdf` was built with `--features cuda` (nvcc 12.8.93, `sm_120`). `discrete-stack build-identity` reported:
  - `compiled.source_sha256 = 177f1b3da11eddbe2cf90d8172c48d33fafca95b533ef8898f3e8d2c245edcfc`
  - `compiled.geometry_sha256 = 46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`
  - `compiled.kernel_sha256 = 5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6`
- **Results** are in the HF store at `results/claude/craw-a8id-20261023/`, with `SHA256SUMS`. The same run passed the `C_raw` and unit-snap device tests at main `fe7e23da2`.
- **No pins break.** Nothing on main pins this sheet's or this README's file hash, so the edit breaks no pin.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `git diff --check`.

## Skills used

- **`solution-integrity`:** the pins come from a measurement at the exact commit, not from another record.
- **`scope-guard`:** only the two pinned lines and an erratum changed; the FIT block, the bar and every other pin are untouched.

## Still open

- Whether the sheet's FIT should be re-confirmed after this erratum is the PM's call (#2842).

## Plain summary

The A8 runbook would have stopped at its first identity check because two of its fingerprints were wrong. Both now carry the values the pinned commit's binary actually reports, measured on a GPU.
