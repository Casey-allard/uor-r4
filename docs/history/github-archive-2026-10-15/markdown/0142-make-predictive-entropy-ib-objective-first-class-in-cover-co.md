# #142 — Make predictive entropy + IB objective first-class in cover compilation reports

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T14:10:55Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-07-24T17:02:08Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/142
- **Merged:** 2026-07-24T17:02:08Z
- **Merge commit:** 669f56681542684a6eae3568d8373f8ddfc243f8
- **Base / Head:** main / UOR-Foundation:copilot/make-predictive-entropy-objectives
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This updates graph cover induction to optimize and audit against predictive utility + compact reusable state, not geometry-only split gain. It introduces a versioned objective model with explicit entropy/IB/cost components and surfaces train-vs-held-out tradeoffs directly in compile reports.

- **Objective model (compiler-side, versioned)**
  - Added `ObjectiveConfig` (schemaed) to `CoverConfig` with:
    - predictive entropy `H(A|R)` proxy
    - future-state entropy `H(S_future|R)` proxy
    - information bottleneck proxy `I(Z;X) - beta * I(Z;Y_future)`
    - teacher-loss/runtime/artifact-size/bytes-read/structural-complexity weights
    - estimator metadata + declared partition usage (`fit=train`, report includes `train,held_out`)
  - Kept all float-bearing objective logic in compiler/report codepaths only.

- **Split decision scoring + deterministic tie behavior**
  - Added objective decomposition and weighted scoring for keep-vs-split comparisons.
  - Recorded per-region decision audits (`RegionDecisionAudit`) with component-level keep/split values.
  - Preserved entropy gate and added deterministic objective tie-breaking: ties resolve to `keep`.

- **Report surface area for auditable tradeoffs**
  - Extended `CoverReport` with `objective` block containing:
    - objective config + estimator metadata
    - separate `train` and `held_out` component vectors
    - explicit `held_out - train` deltas
    - per-region split/keep decision trace
    - migration note for future objective schema evolution

- **Deterministic mini-corpus coverage**
  - Added a focused test validating:
    - objective decomposition emission
    - stable tie-breaking behavior for region decisions under equal weighted score

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectiveConfig {
    pub schema: u32,
    pub ib_beta: f64,
    pub weights: ObjectiveWeights,
    pub estimators: ObjectiveEstimatorMetadata,
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #127

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3646526095 — 2026-07-24T15:43:25Z — Copilot

_on crates/uor-r4-graph-compiler/src/induction.rs:787_

`conditional_entropy_for_key` iterates with `observations.iter().zip(assignments.iter())` but normalizes by `observations.len()`. If `assignments.len() != observations.len()`, the loop will silently drop the tail and the probabilities will be normalized by the wrong total, producing incorrect entropies (and potentially misleading objective scores). It’s safer to either assert 1:1 lengths or compute `n = min(len)` and normalize by `n`.

### review 4774684209 — 2026-07-24T15:43:26Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Introduces a versioned, compiler-side objective model for cover induction so region split decisions are scored and audited against predictive-utility/IB/cost components (train vs held-out), rather than geometry-only split gain, and surfaces these decomposed tradeoffs in `cover_report.json`.

**Changes:**
- Added schemaed `ObjectiveConfig`/weights/estimator metadata and objective component decomposition, plus deterministic “ties resolve to keep” behavior and per-region decision auditing.
- Extended `CoverReport` with an `objective` block including config, train/held-out component vectors, deltas, and per-region decision traces.
- Added a deterministic mini-corpus test covering objective decomposition emission and tie-breaking; updated README to document the new report surface.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| README.md | Documents the new versioned objective block in `cover_report.json` and its train vs held-out component reporting. |
| crates/uor-r4-graph-compiler/tests/induction.rs | Adds a deterministic test for objective decomposition emission and objective-tie → keep behavior. |
| crates/uor-r4-graph-compiler/src/induction.rs | Implements objective config/types, per-split objective scoring + deterministic tie behavior, and adds objective data to the cover compilation report. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646526120 — 2026-07-24T15:43:26Z — Copilot

_on crates/uor-r4-graph-compiler/src/induction.rs:1263_

In the per-split objective audit, `keep_components` currently charges non-zero `runtime_cost_units`, `bytes_read`, and `structural_complexity` (1.0 / SIG_BYTES / 1.0) even though the other cost term (`artifact_size_bytes`) is explicitly modeled as a *delta* (0.0 for keep, >0 for split). This makes the objective’s cost accounting inconsistent and can skew keep-vs-split comparisons (keep is penalized for costs that don’t occur when you don’t add child regions). If these cost proxies are intended to be split deltas (as implied by the artifact-size estimator wording), keep should be 0.0 for all delta-cost fields.

### review-comment 3646526151 — 2026-07-24T15:43:26Z — Copilot

_on crates/uor-r4-graph-compiler/src/induction.rs:1248_

`parent_observations` is built by cloning each `Observation` (including the `Vec<f32>` embedding) for every eligible region split. In deeper inductions this can become a large amount of allocation/copying on a hot loop and will dominate runtime/memory even though the objective only needs a few scalar fields (`next`, `prev`) plus the assignment labels. Consider computing objective entropies from `parent_members` indices (or iterating by reference) to avoid cloning observation vectors.


---
_Comment/review ids in this file: 4774684209, 3646526095, 3646526120, 3646526151_

