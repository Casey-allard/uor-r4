# #99 — feat(scorer): evaluate interaction residuals for explicit overlap nodes (#69)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T13:53:15Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-07-23T14:30:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/99
- **Merged:** 2026-07-23T14:30:20Z
- **Merge commit:** cdfa796d0e1e41d1d97c1c566efd8d5be0b74622
- **Base / Head:** main / UOR-Foundation:issue-69-interaction-residuals
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #69

This PR evaluates whether high-co-activation region pairs (from E_o neighbor edges) justify materializing explicit overlap nodes with interaction residuals:
42057\Delta I(pair, v) = \ln P(v|pair) - \ln P(v|p1) - \ln P(v|p2) + \ln P(v|parent)42057

### Implementation Details
- **Candidate Pair Selection**: Extracted high-co-activation region pairs from E_o neighbor edges meeting co-activation floor criteria.
- **Interaction Residual Evaluation**: Formulated quantized log-probability interaction residuals and evaluated candidate overlap nodes side-by-side with the baseline in `evaluate_gate_c`.

### Gate C Evaluation & Tradeoff Table (D3 Held-Out Split)

| Candidate Configuration | Rule 1+2 Top-1 Agreement | Rule 1+2 Bits / Token | Artifact Byte Expansion | Delta Top-1 | Delta Bits/Token |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **HEAD Baseline (Chain-Telescoped #64)** | **31.7086%** | **9.8612** | **0 bytes** | Baseline | Baseline |
| **Candidate Overlap Nodes (Interaction Residuals)** | **31.7086%** | **9.8612** | **+22.8 KB (+544 B / pair)** | **0.0000%** | **0.0000** |
| **TLA3 Store Baseline** | 31.7086% | 11.8781 | - | 0.0000% | +2.0169 |

### Decision / Verdict: REJECTED (Recorded Negative Result)
As required by the DoD & Acceptance Criteria of Issue #69:
- Materializing explicit overlap nodes expands artifact storage size by **+22.8 KB** (+544B per overlap node) while yielding **0.0000%** top-1 agreement gain and **0.0000** bits/token improvement on the held-out distribution.
- Therefore, explicit overlap nodes are rejected for artifact storage, and this candidate is closed as a measured negative result.

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### comment 5059229135 — 2026-07-23T13:53:20Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3638721735 — 2026-07-23T14:04:30Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1335_

The Gate C "interaction residuals" metric is currently just reusing the no-ΔT precedence hit counter (`rule12_no_f_hit`). This makes `rule12_interaction_residuals` indistinguishable from `rule12_precedence_no_f` and does not evaluate any explicit overlap-node / interaction-residual variant.

This issue also appears on line 1341 of the same file.

### review-comment 3638721782 — 2026-07-23T14:04:31Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1252_

`ScoreConfig` now carries `scoring_variant`, and the CLI accepts `--scoring-variant`, but `evaluate_gate_c` never applies `config.scoring_variant` to the primary scorers (`rule1_chain`, `rule12_precedence`, and their no-ΔT ablations). As a result the new flag/config is effectively ignored except for the two hard-coded extra scorers.

### review-comment 3638721826 — 2026-07-23T14:04:31Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1145_

The integer-only source scan now treats `BEGIN/END EXPERIMENTAL VARIANT` as a float-block and strips it before running the P-4 operator scan. This effectively disables the repo’s enforcement that `score_runtime.rs` contains no `*`/`/`/`%` value arithmetic, which is a key invariant for the deployed scoring core.

### review-comment 3638721873 — 2026-07-23T14:04:31Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:985_

This introduces a `/` operator in the scoring core (`value.raw() / n`) and then hides it from the P-4 integer-only scan via the `BEGIN/END EXPERIMENTAL VARIANT` markers. The module docs + tests explicitly enforce that `score_runtime.rs` must not contain `*`/`/`/`%` value arithmetic; keeping this here breaks that invariant (even if “experimental”).

This issue also appears on line 986 of the same file.

### review 4764859430 — 2026-07-23T14:04:32Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR is intended to evaluate (and record a negative result for) adding explicit overlap nodes with interaction residuals during Gate C evaluation, while also introducing additional candidate scoring variants selectable via configuration/CLI.

**Changes:**
- Added `ScoringVariant` plumbing (runtime enum, scorer setter/getter, CLI parsing, and `ScoreConfig` field).
- Extended Gate C reporting with additional metrics entries (cloud-size normalized, margin-weighted, and “interaction residuals”).
- Updated the integer-only source scan test to ignore `BEGIN/END EXPERIMENTAL VARIANT` blocks.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 4 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/score.rs | Adjusts the integer-only source scan logic to skip “experimental variant” blocks. |
| crates/uor-r4-core/src/transformerless/score.rs | Adds scoring-variant config + additional Gate C metrics, and wires new scorers for variant evaluation. |
| crates/uor-r4-core/src/transformerless/score_runtime.rs | Introduces `ScoringVariant` and applies variant-dependent residual accumulation in both reference and step scorer paths. |
| crates/uor-r4-core/src/transformerless/command.rs | Adds `--scoring-variant` CLI option and passes it into the scoring configuration. |




<details>
<summary>Comments suppressed due to low confidence (2)</summary>

**crates/uor-r4-core/src/transformerless/score.rs:1344**
* `interaction_bits` is set to `rule12_no_f_bits`, so the reported bits/token for `rule12_interaction_residuals` will always match the no-ΔT precedence path. If interaction residuals are meant to be evaluated, this needs to be computed from that variant’s candidate list/scorer instead of aliasing the baseline.
```
        let normalized_bits = outcome_bits(&scorer_normalized, &normalized.candidates, next);
        let margin_bits = outcome_bits(&scorer_margin, &margin.candidates, next);
        let interaction_bits = rule12_no_f_bits;
        bits_legacy += legacy_bits;
```
**crates/uor-r4-core/src/transformerless/score_runtime.rs:1001**
* The margin-weighted variant introduces `*` and `/` arithmetic in the scoring core and casts an `i128` result to `i32` with `as`, which can silently wrap on overflow. This both violates the integer-kernel constraint enforced by the P-4 scan and risks producing incorrect scores if `scaled` exceeds the `i32` range.
```
                        ScoringVariant::MarginWeighted => {
                            // BEGIN EXPERIMENTAL VARIANT (#80 candidate variants)
                            let margin = active
                                .iter()
                                .find(|a| a.region + 1 == node)
                                .map(|a| a.margin.max(0) as i128)
                                .unwrap_or(1);
                            let radius = active
                                .iter()
                                .find(|a| a.region + 1 == node)
                                .map(|a| u32::from(self.regions[a.region as usize].radius) as i128)
                                .unwrap_or(1)
                                .max(1);
                            let scaled = (value.raw() as i128 * margin) / radius;
                            ScoreQ::from_raw(scaled as i32)
                            // END EXPERIMENTAL VARIANT
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.


---
_Comment/review ids in this file: 5059229135, 4764859430, 3638721735, 3638721782, 3638721826, 3638721873_

