# #147 — feat(compiler): introduce unsupervised intervention and counterfactual behavioral probes (#128)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T15:00:17Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-24T15:59:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/147
- **Merged:** 2026-07-24T15:59:40Z
- **Merge commit:** 8e83b41a12966907c8613e56e872e471656c06c3
- **Base / Head:** main / UOR-Foundation:issue-128-behavioral-probes
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #128. Implementation follows PDF §§7, 11, 17 and `docs/formal_vocabulary.md` §3.

## Key Changes
1. **Intervention Record**: Implemented content-addressed `InterventionRecord` tracking baseline observation, affected span $(start, end)$, and expected relation (`Invariant` vs `Sensitive`).
2. **Behavioral Probe Harness**: Implemented `BehavioralProbeHarness` evaluating `invariance_score` and `sensitivity_score` under 5 intervention kinds (`ContextAblation`, `SurfaceVariation`, `EntitySubstitution`, `TemporalChange`, `GoalChange`).
3. **Anti-Memorization Guard**: Added `memorization_check_passed` rejecting non-generalizing surface lookup tables that exhibit zero sensitivity under goal/context interventions.
4. **Cucumber BDD Suite**: Added `features/suites/behavioral_probes.feature` with 3 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 28 BDD scenarios passed (88 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (7 entries; chronological: conversation comments, reviews, review comments)

### comment 5071267208 — 2026-07-24T15:00:22Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3646343605 — 2026-07-24T15:11:42Z — Copilot

_on crates/uor-r4-graph-compiler/src/behavioral_probes.rs:155_

`InterventionRecord::new` builds the probe id from `obs`, `kind`, `affected_span`, and `baseline_output.len()` only, so two records with different `expected_relation` and/or different baseline/intervention outputs can still get the same `id`. That breaks the "content-addressed" property and can cause incorrect deduplication/caching or mis-attribution in reports/certificates.

Include `expected_relation` and a deterministic encoding of both output vectors in the ID payload (ideally using the same blake3-based content-address convention used elsewhere in this crate).

### review-comment 3646343630 — 2026-07-24T15:11:42Z — Copilot

_on crates/uor-r4-graph-compiler/src/behavioral_probes.rs:253_

The anti-memorization guard currently checks `sensitivity_score > 0.0`, which can still pass even if *GoalChange*/*ContextAblation* probes have zero divergence (e.g. another unrelated Sensitive probe passes). This doesn’t match the comment/PR description (“zero sensitivity under goal/context interventions”) and can miss the intended memorization failure mode.

Instead, treat any Sensitive `GoalChange`/`ContextAblation` probe with divergence below `sensitivity_threshold` as a memorization failure, and report the failing probe id.

### review 4774452792 — 2026-07-24T15:11:43Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Introduces an initial behavioral-probing layer in `uor-r4-graph-compiler` to record and evaluate counterfactual interventions (invariance vs sensitivity), plus a BDD feature suite to exercise the harness and anti-memorization guard in an end-to-end style.

**Changes:**
- Added `behavioral_probes` module with `InterventionRecord` + suite evaluation harness and error/report types.
- Exposed the new module via `crates/uor-r4-graph-compiler/src/lib.rs`.
- Added a Cucumber `.feature` file and corresponding step implementations in `tests/bdd.rs` for probe scenarios.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-graph-compiler/src/behavioral_probes.rs | New intervention record + probe suite harness and unit tests. |
| crates/uor-r4-graph-compiler/src/lib.rs | Exports the new `behavioral_probes` module. |
| tests/bdd.rs | Adds Cucumber world fields and BDD step definitions for probe scenarios. |
| features/suites/behavioral_probes.feature | New enforced feature scenarios covering invariance/sensitivity, memorization guard, and span bounds. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646343652 — 2026-07-24T15:11:43Z — Copilot

_on crates/uor-r4-graph-compiler/src/behavioral_probes.rs:106_

The PR description / Issue #128 acceptance criteria say probe records are "serializable" and "replayable", but `InterventionRecord`/`BehavioralProbeReport` (and the enums they contain) don’t implement `Serialize`/`Deserialize`. This diverges from the pattern used elsewhere in this crate (e.g. `observation.rs` records are `Serialize`/`Deserialize`).

Consider deriving serde traits (and adding the serde import) for the record/report types so they can be persisted into certificates/reports without a bespoke encoding.

### comment 5071598827 — 2026-07-24T15:35:35Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5071655941 — 2026-07-24T15:41:40Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved the merge conflicts with `main` in `865c409` (semantic state space fields/steps in `tests/bdd.rs` merged alongside the behavioral probe additions). Verified `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test --workspace --all-targets` (33/33 BDD scenarios) pass post-merge.


---
_Comment/review ids in this file: 5071267208, 5071598827, 5071655941, 4774452792, 3646343605, 3646343630, 3646343652_

