# #38 — Emit per-region Hamming histograms and calibrated acceptance radii in transformerless compiler output

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-21T23:09:48Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-07-21T23:45:39Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/38
- **Merged:** 2026-07-21T23:45:39Z
- **Merge commit:** 5e898fb11a6d5ccf12c4f3df002957caa68a30c2
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundationuor-r4-15
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The transformerless compiler previously emitted class signatures but no calibration artifact for masked-Hamming region membership. This change adds compiler output for per-region Hamming distance distributions and calibrated acceptance radii so downstream routing can consume explicit region thresholds.

- **Calibration report model + computation**
  - Added `HammingCalibrationReport` and `RegionHammingCalibration` in `transformerless::compiler`.
  - Added calibration pipeline that:
    - derives context signatures from held-out corpus positions (falls back to full corpus when held-out is empty),
    - assigns each signature to nearest class per stage,
    - accumulates per-class Hamming histograms,
    - computes acceptance radii via quantile cutoff (95th percentile).
  - Report includes `stage`, `class`, `mask_bits`, `sample_count`, `acceptance_radius`, and full `hamming_histogram`.

- **Compiler output artifact**
  - `compile_hugging_face` now emits `hamming_calibration.json` alongside existing outputs.
  - Keeps existing artifact flow unchanged (`hierarchical_codes.json`, `tless_artifacts.bin`, `tless_store.bin`, manifest).

- **Focused coverage**
  - Added test coverage for calibration emission shape and basic histogram/radius behavior:
    - verifies report metadata,
    - verifies region count (`STAGES * K`),
    - verifies per-region sample counts and zero-distance histogram bins in a controlled fixture.

```rust
let calibration = compiler::calibrate_hamming_regions(&artifacts, &corpus);
let calibration_json = serde_json::to_string_pretty(&calibration)?;
std::fs::write(output.join("hamming_calibration.json"), calibration_json)?;
```

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3626356411 — 2026-07-21T23:22:51Z — Copilot

_on crates/uor-r4-core/src/transformerless/compiler.rs:395_

`quantile_radius` uses `u32` products with `saturating_mul`, which can silently clamp for larger sample counts and return an incorrect quantile cutoff (e.g., totals > ~42M with denominator=100 can trigger saturation). Switching the intermediate arithmetic to `u64` (and computing a target count via division/ceil) avoids overflow and keeps the quantile logic correct.

### review-comment 3626356426 — 2026-07-21T23:22:51Z — Copilot

_on crates/uor-r4-core/src/transformerless/compiler.rs:404_

`calibrate_hamming_regions_from_signatures` can panic with an out-of-bounds histogram index if any `stage_sigs` slice is shorter than `SIG_BYTES` (the inner `chunks_exact(SIG_BYTES)` loop runs zero iterations, leaving `best_dist = u32::MAX`, which is then used as an index). Since this is `pub`, it should defensively validate the input layout up front (or otherwise handle the empty/short case).

### review 4749752996 — 2026-07-21T23:22:52Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a calibration artifact to the transformerless compiler pipeline so downstream components can consume explicit masked-Hamming region thresholds, based on per-region Hamming distance histograms and a quantile-derived acceptance radius.

**Changes:**
- Introduces `HammingCalibrationReport` / `RegionHammingCalibration` and a calibration pipeline that builds per-stage/per-class Hamming histograms and computes acceptance radii.
- Emits `hamming_calibration.json` during `compile_hugging_face` runs alongside existing compiler artifacts.
- Adds an integration test covering calibration report shape and basic histogram/radius behavior using a small controlled fixture.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/compiler.rs | Adds calibration report types + histogram accumulation + quantile-based acceptance radius computation. |
| crates/uor-r4-core/src/transformerless/command.rs | Writes `hamming_calibration.json` to the compiler output directory during `compile_hugging_face`. |
| crates/uor-r4-core/tests/hierarchical_compiler.rs | Adds a focused test ensuring calibration report structure and basic histogram/radius properties. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626356454 — 2026-07-21T23:22:52Z — Copilot

_on crates/uor-r4-core/tests/hierarchical_compiler.rs:71_

This test hard-codes `288` for `signature_bits`, which is derived from the compiler constants (`D` / `SIG_BYTES`). Using `SIG_BYTES * 8` keeps the test correct if `D` changes in the future (while still validating the reported geometry).


---
_Comment/review ids in this file: 4749752996, 3626356411, 3626356426, 3626356454_

