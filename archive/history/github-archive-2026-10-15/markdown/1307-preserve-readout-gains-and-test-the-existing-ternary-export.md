# #1307 — Preserve readout gains and test the existing ternary export

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T08:07:35Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-09-20T08:08:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1307
- **Merged:** 2026-09-20T08:08:41Z
- **Merge commit:** 34d822e5da33f015ecc7e035cfb4bef9de83cdab
- **Base / Head:** main / UOR-Foundation:codex/readout-result-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1306 improves the empirical output head, but its finite floating-head comparison does not establish feature rank/capacity as the dominant bottleneck. This review verifies the saved-data gains, corrects teacher/exposure/KL and artifact-identity claims, and replaces the unsupported width recommendation with one fixed-feature ternary export experiment.

The complete DeepSeek prompt compares the existing quantizer with one fit-activation-aware dyadic ternary calibration of the retained floating head. It specifies numerical-preserving tokenizer metadata repair, common-objective replay, reloaded CPL2 evaluation, non-vacuous contextual controls, actual generation, bounded resources and protected delivery. No new model run or serving change is included in this PR.

README, the canonical plan/dependencies, current state, direction, project map, handoff, evidence and resource interpretation now agree. Raw receipts and retained artifacts are preserved. The latest cumulative charge is retained while its full measured-wall provenance remains unverified.

Validation: independently reconstructed saved loss means and paired document intervals; checked retained member sets/sizes/hashes and reviewed source; focused independent review of the projection mathematics; claim-wording gate, new relative-link check and git diff --check passed. No Rust build/training/model forward was run. Historical queue statuses are compatibility acknowledgements.

References #973, #820, #963, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

