# #1695 — Fit geometric native route credit and retain the discrete update regressions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T19:33:48Z
- **Updated:** 2026-10-09T13:17:27Z
- **Closed:** 2026-10-04T19:35:30Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1695
- **Merged:** 2026-10-04T19:35:29Z
- **Merge commit:** de3fa0e868c52b8eb2b8e25dba4de6e0e60545ab
- **Base / Head:** main / UOR-Foundation:codex/geometric-native-route-fit-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The current geometric bridge had native action credit but no completed fit using it. Add the bounded Rust paired fitter and a seeded transfer preparer that excludes exposed panels, then retain the actual fitted regressions and update the active next step.

Both64-update fits change packed H4 actions but regress against the accepted parent (CE0.492382,31/64 complete): Joint64 CE0.739038,20/64; Role64 CE0.810202,15/64. Baseline-inclusive selection retains0; selected/parent fresh0/32. Nonselected learned fresh evaluation is NOT_RUN. No learned model is adopted. Historical source/research reviews identify existing discrete hard-trajectory/native acceptance machinery for the next bridge adapter and retain directed relative geometry as a separate comparison.

Validation: initial committed source90a8f575 passes6 fit+4 preparer tests and release build on LinuxCPU; after a pre-update strict offline-label decoder failure, repaired09b226b3 passes7 fit tests/release. Frozen pool provenance is now checked without changing runtime inputs or panel selection. Both fits finish exit0 on CPU48, noGPU/runners/laptopmodelCPU. Saved paired review passes780,994 checks with explicit reconstruction limits; exact returns retain250files128,492,086B. Fit/preparer source bytes are unchanged from executed revisions. Source-bound commands, costs, negative artifacts and review limits are linked in the evidence packet.

References #1552. Full grounded-generation acceptance remains open; shared session source is untouched.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5983629024 — 2026-10-04T19:34:36Z — Casey-allard

{
  "schema": "uor-r4.independent-exact-head-delivery-review/1",
  "reviewed_head": "0f1f333e7853ffffbff0f22744d06752875ce630",
  "base": "570349f28a627d9841b804ee9fdeaa750f9c619e",
  "executed_fit_source_commit": "09b226b35699e3b00f110b2c6d5b22e6cf89ad2c",
  "status": "APPROVED_SCOPED_DELIVERY",
  "checks": [
    {
      "check": "retained binding paired-independent-audit.json",
      "pass": true
    },
    {
      "check": "retained binding panel-preparation-review.json",
      "pass": true
    },
    {
      "check": "retained binding joint-2-full-return-verification.json",
      "pass": true
    },
    {
      "check": "retained binding role-2-full-return-verification.json",
      "pass": true
    },
    {
      "check": "retained binding mode64-comparison.json",
      "pass": true
    },
    {
      "check": "retained binding prior-credit-row-comparison.json",
      "pass": true
    },
    {
      "check": "retained binding prior-credit-row-interpretation.md",
      "pass": true
    },
    {
      "check": "retained binding historical-engines-review.md",
      "pass": true
    },
    {
      "check": "retained binding historical-geometry-review.md",
      "pass": true
    },
    {
      "check": "retained binding historical-retirement-audit.md",
      "pass": true
    },
    {
      "check": "retained binding historical-review-coverage.json",
      "pass": true
    },
    {
      "check": "retained binding next-step-mathematical-review.md",
      "pass": true
    },
    {
      "check": "retained binding next-step-engineering-review.md",
      "pass": true
    },
    {
      "check": "retained binding conditional-research-review.md",
      "pass": true
    },
    {
      "check": "retained binding ledger-charge.json",
      "pass": true
    },
    {
      "check": "retained binding resource-projection.json",
      "pass": true
    },
    {
      "check": "all worker elapsed arithmetic",
      "pass": true
    },
    {
      "check": "returned totals",
      "pass": true
    },
    {
      "check": "executed source equality crates/uor-r4-training/examples/geometric-native-route-fit.rs",
      "pass": true
    },
    {
      "check": "executed source equality crates/uor-r4-training/examples/geometric-native-route-fit-panel.rs",
      "pass": true
    },
    {
      "check": "exact scope six paths",
      "pass": true
    },
    {
      "check": "baseline selected joint-copy",
      "pass": true
    },
    {
      "check": "baseline selected role-surface",
      "pass": true
    }
  ],
  "findings": [],
  "manual_review": [
    "Six changed paths: two unchanged executed example sources, report, evidence, current-state, project-track.",
    "Full development CE/completions and first Q4 crossing at42 consistent with completed paired review; no fitted feedback promotion.",
    "Fresh claims explicitly restricted to selected parent; learned48/64 fresh NOT_RUN.",
    "Initial pool_index execution failure and auditor serde-order repair distinct from mechanism outcomes.",
    "Complete150-minute ledger charge separate from worker time; Linux CPU/noGPU and unavailable Metal explicit.",
    "Historical all-family inventory explicitly excludes exhaustive source/evidence verification.",
    "Next hard-native acceptance and relative-H4 comparison are prospective, without transformer or collision gates."
  ],
  "limits": [
    "Reuses completed arithmetic/source reviews; no native, optimizer, BPE, Cargo or model reexecution.",
    "Does not independently certify omitted utility/gradient/Adam arrays or whole-path serving energy."
  ],
  "source_file_sha256": {
    "crates/uor-r4-training/examples/geometric-native-route-fit.rs": "2561a129dccce6f024d15dd88ef0053c515241f0196680913e5eed1ac915e8db",
    "crates/uor-r4-training/examples/geometric-native-route-fit-panel.rs": "5257479ef3d616e326e0c77443799e92dd10c0fbf4bf1b4e1b8182ea932d400b"
  }
}



---
_Comment/review ids in this file: 5983629024_

